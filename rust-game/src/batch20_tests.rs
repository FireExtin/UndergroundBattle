//! Focused batch scenarios using production actions and persistence at each choice/response.
use crate::{model::*, rules::*, unit_support::*};

fn attach_region(g: &mut Game, definition: &str, actor: usize, r: usize) -> String {
    let c = g.make_card(definition, actor);
    let id = c.id.clone();
    g.attachments.push(Attachment { card: c, host_id: g.regions[r].card.id.clone() });
    id
}
fn trigger(g: &mut Game, id: &str, event: Event) {
    let (r,c) = g.in_play(id).unwrap();
    let source = g.source_snapshot(c, r);
    g.emit_event(c.controller, source, event);
    g.drive().unwrap();
    restore(g);
}
fn prepare(g: &mut Game) {
    g.jc035_preparation = None; // explicit prepared layout at this turn's beginning
    g.begin_window(Window::Prepare);
    g.drive().unwrap();
    restore(g);
}
fn society(g: &mut Game, seat: usize) {
    g.players[seat].society_zone.card = Some(g.make_card("MSJC04", seat));
}

#[test]
fn batch20_jc043_targets_direct_original_region_and_society_but_not_character_host() {
    let mut g = game();
    society(&mut g, 1);
    let satellite = g.make_card("JC068", 1);
    let satellite_id = satellite.id.clone();
    g.attachments.push(Attachment { card: satellite, host_id: "society:p1".into() });
    let local = attach_region(&mut g, "BQ091", 1, 0);
    let distant = attach_region(&mut g, "BQ091", 1, 1);
    let host = field(&mut g, "JC125", 1);
    let equipment = g.make_card("BQ022", 1);
    let equipment_id = equipment.id.clone();
    g.attachments.push(Attachment { card: equipment, host_id: host });
    let arsonist = field(&mut g, "JC043", 0);
    trigger(&mut g, &arsonist, Event::Enter);
    let options = &g.pending.as_ref().unwrap().choice.options;
    assert!(options.iter().any(|o| o.id == local));
    assert!(options.iter().any(|o| o.id == satellite_id));
    assert!(!options.iter().any(|o| o.id == distant || o.id == equipment_id));
    choose(&mut g, vec![satellite_id.clone()]);
    top(&mut g);
    assert!(g.in_play(&satellite_id).is_none());
    assert!(g.players[1].graveyard.iter().any(|c| c.definition == "JC068"));
}

#[test]
fn batch20_bq054_discard_trigger_is_free_and_locks_a_distant_hidden_card() {
    let mut g = game();
    let id = held(&mut g, "BQ054", 0);
    let mut c = g.make_card("JC125", 1); c.face_down = true;
    let target = c.id.clone(); g.regions[2].cards.push(c);
    assert_eq!(resources(&g, 0), 0);
    g.discard_hand_card(0, &id).unwrap();
    g.drive().unwrap();
    restore(&mut g);
    choose(&mut g, vec![target.clone()]);
    top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.lock_markers, 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition == "BQ054" && c.id != id));
}

#[test]
fn batch20_hourglass_recounts_after_death_and_cannot_skip_forced_preparation() {
    let mut g = game();
    let hourglass = attach_region(&mut g, "JC048", 0, 0);
    g.board_mut(&hourglass).unwrap().time_markers = 1;
    let carrier = field(&mut g, "JC125", 1);
    g.board_mut(&carrier).unwrap().time_markers = 1;
    prepare(&mut g);
    let p = g.pending.clone().unwrap();
    assert_eq!(p.choice.min, Some(1));
    reject(&mut g, 0, Action { choice_id: Some(p.choice.id), selected: Some(vec![]), ..Action::new("choose") });
    choose(&mut g, vec![hourglass.clone()]);
    top(&mut g);
    assert!(g.board(&carrier).is_none());
    assert_eq!(g.board(&hourglass).unwrap().1.time_markers, 2);
    g.turn += 1;
    // Explicit next-turn fixture: keep the same boundary as Effect::NextTurn.
    g.first_marker_placements.clear(); g.wm031_observations.clear();
    prepare(&mut g);
    choose(&mut g, vec![hourglass.clone()]);
    top(&mut g);
    assert!(g.board(&hourglass).is_none());
}

#[test]
fn batch20_mixed_optional_and_forced_preparation_uses_one_order_queue() {
    let mut g = game();
    let hourglass = attach_region(&mut g, "JC048", 0, 0);
    let shelter = attach_region(&mut g, "BQ091", 0, 0);
    prepare(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 2);
    choose(&mut g, vec![hourglass]);
    assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(0));
    assert_eq!(g.pending.as_ref().unwrap().choice.options[0].id, shelter);
    choose(&mut g, vec![]);
    top(&mut g);
    restore(&mut g);
}

#[test]
fn batch20_shelter_mill_is_not_draw_and_seven_graves_grants_investigation_dynamically() {
    let mut g = game();
    let shelter = attach_region(&mut g, "BQ091", 0, 0);
    g.players[0].deck = vec![g.make_card("JC125", 0)];
    for _ in 0..6 { let c = g.make_card("JC125", 0); g.players[0].graveyard.push(c); }
    assert_eq!(g.attachment_current_icons(g.board(&shelter).unwrap().1, 0).investigation, 0);
    prepare(&mut g);
    choose(&mut g, vec![shelter.clone()]);
    top(&mut g);
    assert!(g.players[0].deck.is_empty() && !g.players[0].eliminated);
    assert_eq!(g.attachment_current_icons(g.board(&shelter).unwrap().1, 0).investigation, 1);
    g.players[0].graveyard.pop();
    assert_eq!(g.attachment_current_icons(g.board(&shelter).unwrap().1, 0).investigation, 0);
}

#[test]
fn batch20_wm034_pays_a_real_time_marker_before_global_non_damage_destruction() {
    let mut g = game();
    let urd = field(&mut g, "WM034", 0);
    g.board_mut(&urd).unwrap().time_markers = 1;
    let other = field(&mut g, "JC070", 1);
    g.board_mut(&other).unwrap().shield = 2;
    let hidden = field(&mut g, "JC125", 1);
    g.board_mut(&hidden).unwrap().face_down = true;
    let action = g.legal_actions(0).into_iter().find(|a| a.action.card_id.as_deref() == Some(&urd)
        && a.action.ability_id.as_deref() == Some("batch20-wm034-destroy")).unwrap().action;
    act(&mut g, 0, action);
    assert!(g.board(&urd).unwrap().1.exhausted);
    assert_eq!(g.board(&urd).unwrap().1.time_markers, 0);
    top(&mut g);
    assert!(g.board(&other).is_none());
    assert!(g.board(&hidden).is_some());
    prepare(&mut g);
    choose(&mut g, vec![urd.clone()]);
    top(&mut g);
    assert!(g.board(&urd).is_none());
}

#[test]
fn batch20_satellite_is_playable_visible_and_removable_outside_regions() {
    let mut g = game(); society(&mut g, 0);
    fund(&mut g, 0, "JC063", 3);
    let id = held(&mut g, "JC068", 0);
    let actions = g.legal_actions(0);
    let action = actions.iter().find(|a| a.action.card_id.as_deref() == Some(&id) && a.action.kind == "play").unwrap().action.clone();
    assert_eq!(action.target_id.as_deref(), Some("society:p0"));
    act(&mut g, 0, action); top(&mut g);
    let id = g.attachments[0].card.id.clone();
    assert!(g.board(&id).is_none() && g.in_play(&id).is_some());
    assert!(g.view(0).attachments.iter().any(|a| a.host_id == "society:p0" && a.card.region.is_none()));
    let enemy = (0..4).find(|s| g.team(*s) != g.team(0)).unwrap();
    let teammate = (1..4).find(|s| g.team(*s) == g.team(0)).unwrap();
    assert!(g.jc068_blocks_conceal(enemy) && !g.jc068_blocks_conceal(teammate));
    g.players[0].assets.clear();
    prepare(&mut g);
    choose(&mut g, vec![id.clone()]); top(&mut g);
    assert!(g.in_play(&id).unwrap().1.exhausted && !g.jc068_blocks_conceal(enemy));
    g.remove_dead(&id, RemovalCause::Destroy);
    assert!(g.in_play(&id).is_none()); restore(&mut g);
}

#[test]
fn batch20_satellite_upkeep_can_choose_payment_and_restore_while_waiting() {
    let mut g = game(); society(&mut g, 0); fund(&mut g, 0, "JC063", 1);
    let c = g.make_card("JC068", 0); let id = c.id.clone();
    g.attachments.push(Attachment { card: c, host_id: "society:p0".into() });
    prepare(&mut g); choose(&mut g, vec![id.clone()]);
    while g.pending.is_none() { pass(&mut g); }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "upkeep"); restore(&mut g);
    choose(&mut g, vec!["pay".into()]);
    assert_eq!(resources(&g, 0), 0);
    assert!(!g.in_play(&id).unwrap().1.exhausted);
}

#[test]
fn batch20_bq094_observes_own_death_even_in_its_own_destroy_wave_and_is_not_an_asset() {
    let mut g = game();
    let id = held(&mut g, "BQ094", 0);
    reject(&mut g, 0, Action { card_id: Some(id), ..Action::new("asset") });
    let observer = attach_region(&mut g, "BQ094", 0, 0);
    let own = field(&mut g, "JC125", 0);
    let teammate = field(&mut g, "JC125", 1);
    let hidden = field(&mut g, "JC125", 0); g.board_mut(&hidden).unwrap().face_down = true;
    let deaths = [observer, own, teammate, hidden].iter().map(|id| {
        let (r,c) = g.in_play(id).unwrap(); (id.clone(), RemovalCause::Destroy, g.source_snapshot(c, r))
    }).collect();
    g.remove_death_batch(deaths); g.drive().unwrap(); restore(&mut g);
    assert!(g.pending.is_none(), "BQ094 is mandatory");
    assert_eq!(g.stack.len(), 1);
    top(&mut g);
    assert_eq!(g.regions[0].influence, [1,0]);
}

#[test]
fn batch20_jz52_blocks_hand_faceup_but_allows_conceal_and_real_death_grave_play() {
    let mut g = game(); let vizier = field(&mut g, "JZ52", 0);
    fund(&mut g, 0, "JC031", 12);
    let hand = held(&mut g, "JC031", 0);
    reject(&mut g, 0, Action { card_id: Some(hand.clone()), region: Some(0), ..Action::new("deploy") });
    assert!(g.legal_actions(0).iter().any(|a| a.action.kind == "conceal" && a.action.card_id.as_deref() == Some(&hand)));
    let grave = g.make_card("JC031", 0); let id = grave.id.clone(); g.players[0].graveyard.push(grave);
    act(&mut g, 0, Action { card_id: Some(id), region: Some(0), ..Action::new("deploy") });
    assert!(matches!(g.stack[0].frame.as_ref().unwrap().source.play_source, Some(PlaySource::GraveyardByJZ52(_))));
    g.remove_dead(&vizier, RemovalCause::Destroy); restore(&mut g);
    top(&mut g);
    assert!(g.regions[0].cards.iter().any(|c| c.definition == "JC031" && !c.face_down));
}

#[test]
fn batch20_bq080_entry_return_and_scarab_attachment_trigger_have_separate_receipts() {
    let mut g = game(); let source = field(&mut g, "BQ080", 0);
    let c = g.make_card("JC125", 0); let grave = c.id.clone(); g.players[0].graveyard.push(c);
    trigger(&mut g, &source, Event::Enter); choose(&mut g, vec![grave]); top(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition == "JC125"));
    let attachment = attach_region(&mut g, "BQ094", 0, 0);
    trigger(&mut g, &attachment, Event::Enter);
    assert!(matches!(&g.pending.as_ref().unwrap().resolution, ChoiceResolution::Declare { declaration, .. }
        if declaration.ability.key == "batch20-bq080-hide"));
    choose(&mut g, vec!["accept".into()]);
    let mut forged = g.clone();
    if let Op::BQ080HideForAttachment { observation: Some(o) } = &mut forged.stack[0].frame.as_mut().unwrap().steps[0].op { o.attachment.card.controller = 1; }
    assert!(Game::from_persisted(&serde_json::to_string(&forged).unwrap()).is_err());
    top(&mut g);
    assert!(g.regions[0].cards.iter().any(|c| c.definition == "BQ080" && c.face_down && c.id != source));
}

#[test]
fn batch20_jz57_grants_real_skeleton_sacrifice_and_optional_draw_after_short_mill() {
    let mut g = game(); let host = field(&mut g, "JC125", 0);
    let c = g.make_card("JZ57", 0); let tattoo = c.id.clone();
    g.attachments.push(Attachment { card: c, host_id: host.clone() });
    let (_,c) = g.board(&host).unwrap();
    assert!(g.current_subtypes(c).iter().any(|s| s == "骷髅"));
    deck_cards(&mut g, 2, &["JC125"]); deck_cards(&mut g, 0, &["JC125"]);
    act(&mut g, 0, Action { card_id: Some(tattoo), ability_id: Some("batch20-jz57-mill".into()),
        target_id: Some("p2".into()), cost_selected: Some(vec![host]), ..Action::new("activate") });
    assert!(g.attachments.is_empty(), "paying the host also removes the tattoo");
    while g.pending.is_none() { pass(&mut g); }
    assert!(g.players[2].deck.is_empty() && !g.players[2].eliminated);
    assert_eq!(g.players[2].graveyard.len(), 1); restore(&mut g);
    choose(&mut g, vec!["draw".into()]);
    assert_eq!(g.players[0].hand.len(), 1);
}

#[test]
fn batch20_jc033_reduces_only_own_paid_reveal_and_search_survives_its_sacrifice() {
    let mut g = game(); society(&mut g, 0); fund(&mut g, 0, "JC031", 4);
    let c = g.make_card("JC033", 0); let id = c.id.clone();
    g.attachments.push(Attachment { card: c, host_id: "society:p0".into() });
    let c = g.make_card("JC031", 0);
    assert_eq!(g.paid_reveal_cost(0, &c), crate::catalog::card("JC031").cost - 1);
    assert_eq!(g.paid_reveal_cost(1, &c), crate::catalog::card("JC031").cost);
    assert_eq!(g.effective_cost(0, &c), crate::catalog::card("JC031").cost);
    let eligible = deck_cards(&mut g, 0, &["JC031", "JC125"])[0].clone();
    act(&mut g, 0, Action { card_id: Some(id.clone()), ability_id: Some("batch20-jc033-search".into()), region: Some(1), ..Action::new("activate") });
    assert!(g.in_play(&id).is_none());
    while g.pending.is_none() { pass(&mut g); }
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 1);
    choose(&mut g, vec![eligible]);
    assert!(g.regions[1].cards.iter().any(|c| c.definition == "JC031" && c.face_down));
    assert_eq!(g.players[0].deck.len(), 1);
}

#[test]
fn batch20_cq16_search_preserves_original_region_and_filters_exact_printed_cost() {
    let mut g = game(); let source = field(&mut g, "CQ16", 0); fund(&mut g, 0, "JC063", 1);
    let hidden = field(&mut g, "JC125", 0); g.board_mut(&hidden).unwrap().face_down = true;
    trigger(&mut g, &source, Event::Enter);
    assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o| o.id == source));
    assert!(!g.pending.as_ref().unwrap().choice.options.iter().any(|o| o.id == hidden));
    choose(&mut g, vec![source.clone()]); top(&mut g);
    assert_eq!(g.board(&source).unwrap().1.lock_markers, 1);
    let eligible = deck_cards(&mut g, 0, &["BQ054", "CQ16", "JC125"])[0].clone();
    act(&mut g, 0, Action { card_id: Some(source.clone()), ability_id: Some("batch20-cq16-search".into()), ..Action::new("activate") });
    g.remove_dead(&source, RemovalCause::Destroy);
    while g.pending.is_none() { pass(&mut g); }
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 1);
    choose(&mut g, vec![eligible]);
    assert!(g.regions[0].cards.iter().any(|c| c.definition == "BQ054" && !c.face_down));
    assert_eq!(g.players[0].deck.len(), 2);
}

#[test]
fn batch20_jc038_combines_granted_and_printed_raid_and_keeps_event_time_grant() {
    let mut g = game(); let leader = field(&mut g, "JC038", 1);
    let source = field(&mut g, "JC029", 0); let victim = field(&mut g, "JZ52", 2);
    trigger(&mut g, &source, Event::Reveal);
    assert!(g.pending.as_ref().unwrap().choice.title.contains("袭击2"));
    choose(&mut g, vec![victim.clone()]);
    g.remove_dead(&leader, RemovalCause::Destroy); restore(&mut g);
    assert_eq!(g.stack.len(), 1); top(&mut g);
    assert_eq!(g.board(&victim).unwrap().1.damage, 2);
}

#[test]
fn batch20_jc038_self_sacrifice_still_draws_after_empty_or_victim_chosen_discard() {
    for empty in [false, true] {
        let mut g = game(); let source = field(&mut g, "JC038", 0); deck_cards(&mut g, 0, &["JC125"]);
        let hand = if empty { None } else { Some(held(&mut g, "JC125", 2)) };
        act(&mut g, 0, Action { card_id: Some(source.clone()), ability_id: Some("batch20-jc038-discard-draw".into()),
            cost_selected: Some(vec![source.clone()]), target_id: Some("p2".into()), ..Action::new("activate") });
        assert!(g.board(&source).is_none());
        if let Some(hand) = hand {
            while g.pending.is_none() { pass(&mut g); }
            assert_eq!(g.pending.as_ref().unwrap().seat, 2); choose(&mut g, vec![hand]);
        } else { top(&mut g); }
        assert_eq!(g.players[0].hand.len(), 1);
    }
}

#[test]
fn batch20_jc040_aggregates_one_discard_event_and_represses_across_regions_once() {
    let mut g = game(); field(&mut g, "JC040", 0);
    let ally = held(&mut g, "JC125", 1); g.discard_hand_card(1, &ally).unwrap();
    assert!(g.effects.is_empty());
    let ids = (0..3).map(|_| held(&mut g, "JC125", 2)).collect::<Vec<_>>();
    g.regions[0].influence = [0,1]; g.regions[1].influence = [0,1];
    g.discard_hand_cards(2, &ids).unwrap();
    assert_eq!(g.effects.len(), 1);
    g.drive().unwrap(); restore(&mut g);
    choose(&mut g, vec!["p3".into()]); // Can repress a different player from the discarder.
    while g.pending.is_none() { pass(&mut g); }
    assert_eq!(g.pending.as_ref().unwrap().seat, 3);
    assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(2)); // X=3, only two remain.
    let ids = g.pending.as_ref().unwrap().choice.options.iter().map(|o| o.id.clone()).collect();
    choose(&mut g, ids);
    assert_eq!(g.regions[0].influence, [0,0]); assert_eq!(g.regions[1].influence, [0,0]);
    let deck = g.players[2].deck.len(); g.mill_cards(2, 2); assert!(deck >= 2 && g.effects.is_empty());
}

#[test]
fn batch20_jc034_requires_its_own_actual_vampire_combat_contribution() {
    let mut g = game(); society(&mut g, 0);
    let c = g.make_card("JC034", 0); let path = c.id.clone();
    g.attachments.push(Attachment { card:c, host_id:"society:p0".into() });
    field(&mut g, "JC029", 1); // A teammate alone does not qualify.
    assert!(g.jc034_combat_observers(0, 0).is_empty());
    let vampire = field(&mut g, "JC029", 0);
    g.board_mut(&vampire).unwrap().exhausted = true;
    assert!(g.jc034_combat_observers(0, 0).is_empty());
    g.board_mut(&vampire).unwrap().exhausted = false;
    let hand = held(&mut g, "JC125", 2);
    // Keep the former incomplete reward setup as an explicit rejection control:
    // it is not a real after-combat window and removes a contributor before the
    // mandatory recipient transition can be offered.
    let mut incomplete = g.clone();
    incomplete.reward(0, 0, 1, 1).unwrap();
    incomplete.remove_dead(&vampire, RemovalCause::Destroy);
    incomplete.remove_dead(&path, RemovalCause::Destroy);
    assert!(incomplete.drive().is_err());
    // This is the observer unit test: freeze the genuine contribution witness,
    // then model departure before its declaration. Normal complete battle and
    // recipient commands are covered by their separate combat tests.
    g.begin_window(Window::After(0, 1));
    let declarations = g.jc034_combat_observers(0, 0);
    assert_eq!(declarations.len(), 1);
    g.effects.extend(declarations.into_iter().map(|declaration| Effect::Declare { declaration }));
    g.remove_dead(&vampire, RemovalCause::Destroy); g.remove_dead(&path, RemovalCause::Destroy);
    g.drive().unwrap(); restore(&mut g);
    while !matches!(&g.pending.as_ref().unwrap().resolution, ChoiceResolution::Declare { declaration,.. } if declaration.source.card.definition == "JC034") { resolve_choice(&mut g); }
    choose(&mut g, vec!["p2".into()]);
    while g.pending.is_none() { pass(&mut g); }
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    choose(&mut g, vec![hand]);
    assert!(g.players[2].hand.is_empty());
}

#[test]
fn batch20_corrected_printed_icons_apply_only_to_current_kernel_and_initiative() {
    let mut g = game();
    for (id, initiative, other) in [
        ("JC029", Icons { investigation:0, combat:1, influence:1 }, Icons::default()),
        ("JZ49", Icons { investigation:0, combat:2, influence:0 }, Icons::default()),
        ("JC044", Icons { investigation:1, combat:1, influence:1 }, Icons { investigation:1, combat:1, influence:1 }),
    ] {
        let instance = field(&mut g, id, 0);
        g.first_team = 0; assert_eq!(g.icons(g.board(&instance).unwrap().1, 0), initiative, "{id} first");
        g.first_team = 1; assert_eq!(g.icons(g.board(&instance).unwrap().1, 0), other, "{id} second");
        g.board_mut(&instance).unwrap().exhausted = true;
        assert_eq!(g.icons(g.board(&instance).unwrap().1, 0), Icons::default());
    }
    restore(&mut g);
}

#[test]
fn batch20_jc067_runs_a_second_full_influence_contest_then_one_renown() {
    let mut g = game(); let mayor = field(&mut g, "JC067", 0);
    // Equal influence totals avoid an Award; renown waits for both comparisons.
    field(&mut g, "JC038", 2);
    assert!(!g.board(&mayor).unwrap().1.exhausted);
    g.begin_window(Window::Before(0,2));
    g.close_window().unwrap(); g.drive().unwrap(); restore(&mut g);
    assert_eq!(g.influence_contests[0].completed, 1); assert!(!g.influence_contests[0].ended);
    assert!(g.pending.is_none());
    g.close_window().unwrap(); assert_eq!(g.window, Some(Window::Before(0,2)));
    g.close_window().unwrap(); g.drive().unwrap(); restore(&mut g);
    assert_eq!(g.influence_contests[0].completed, 2); assert!(g.influence_contests[0].ended);
    assert!(matches!(&g.pending.as_ref().unwrap().resolution, ChoiceResolution::Declare { declaration, .. } if declaration.ability.key == "renown"));
    choose(&mut g, vec!["accept".into()]); top(&mut g);
    assert_eq!(g.regions[0].influence, [1,0]);
    g.close_window().unwrap(); assert_eq!(g.window, Some(Window::Before(1,0)));
}

#[test]
fn batch20_jc067_leaving_and_returning_does_not_reset_completed_contests() {
    let mut g = game(); let mayor = field(&mut g, "JC067", 0); g.board_mut(&mayor).unwrap().exhausted = true;
    g.begin_window(Window::Before(0,2)); g.close_window().unwrap(); g.drive().unwrap();
    g.return_hand(&mayor); field(&mut g, "JC067", 0);
    g.regions[0].cards.iter_mut().for_each(|c| c.exhausted = true);
    g.close_window().unwrap(); g.close_window().unwrap(); g.drive().unwrap(); restore(&mut g);
    assert_eq!(g.influence_contests[0].completed, 2);
    g.close_window().unwrap(); assert_eq!(g.window, Some(Window::Before(1,0)));
}

#[test]
fn batch20_jc067_loss_of_grant_ends_region_without_skipping_final_responses() {
    let mut g = game(); let mayor = field(&mut g, "JC067", 0); g.board_mut(&mayor).unwrap().exhausted = true;
    g.begin_window(Window::Before(0,2)); g.close_window().unwrap(); g.drive().unwrap();
    g.return_hand(&mayor);
    g.close_window().unwrap();
    assert_eq!(g.window, Some(Window::After(0,2)));
    assert!(g.influence_contests[0].ended); assert_eq!(g.effects.len(), 1);
    g.drive().unwrap(); restore(&mut g);
    g.close_window().unwrap(); assert_eq!(g.window, Some(Window::Before(1,0)));
}

#[test]
fn batch20_jc067_scored_region_replacement_cannot_receive_an_extra_comparison() {
    let mut g = game(); field(&mut g, "JC067", 0);
    let old = g.regions[0].card.id.clone();
    g.begin_window(Window::Before(0,2)); g.close_window().unwrap(); g.drive().unwrap();
    assert!(matches!(g.window, Some(Window::Win(0,0))));
    assert!(g.influence_contests[0].ended);
    drain(&mut g);
    // Complete the win response window and its ordinary return/score decisions.
    for _ in 0..40 {
        if g.regions[0].card.id != old { break; }
        if g.pending.is_some() { resolve_choice(&mut g); } else { pass(&mut g); }
    }
    assert_ne!(g.regions[0].card.id, old); restore(&mut g);
    assert!(!g.continue_influence_contests(0));
}

#[test]
fn batch20_optional_observation_rejects_replayed_event_and_forced_choice_rewrites() {
    let mut g = game(); field(&mut g, "JC040", 0); let id = held(&mut g, "JC125", 2);
    g.discard_hand_card(2, &id).unwrap(); g.drive().unwrap(); restore(&mut g);
    let mut bad = g.clone(); bad.pending.as_mut().unwrap().choice.min = Some(1);
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut bad = g.clone();
    if let ChoiceResolution::Declare { declaration, .. } = &bad.pending.as_ref().unwrap().resolution {
        bad.effects.push_back(Effect::Declare { declaration: declaration.clone() });
    }
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
}

#[test]
fn batch20_jc065_independent_targets_bind_before_payment_and_victim_chooses_hidden() {
    let mut g = game(); fund(&mut g, 0, "JC063", 1);
    let enemy = (1..4).find(|s| g.team(*s) != g.team(0)).unwrap();
    let other = (1..4).find(|s| g.team(*s) == g.team(0)).unwrap();
    let other_team = g.team(other);g.regions[2].influence[other_team] = 2;
    let h1 = field(&mut g, "JC125", enemy); g.board_mut(&h1).unwrap().face_down = true;
    let h2 = field(&mut g, "JC125", enemy); g.board_mut(&h2).unwrap().face_down = true;
    let ally = field(&mut g, "JC125", other); g.board_mut(&ally).unwrap().face_down = true;
    let source = held(&mut g, "JC065", 0);
    let action = g.legal_actions(0).into_iter().find(|a| a.action.kind == "play" && a.action.card_id.as_deref()==Some(&source)
        && a.action.region==Some(0) && a.action.target_id.as_deref()==Some(&player_id(enemy))
        && a.action.selected==Some(vec![player_id(other)])).unwrap().action;
    let mut illegal = action.clone();illegal.target_id=Some(player_id(other));reject(&mut g,0,illegal);
    let mut illegal = action.clone();illegal.selected=None;reject(&mut g,0,illegal);
    act(&mut g,0,action);assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().targets.len(),3);
    while g.pending.is_none() { pass(&mut g); }
    let p=g.pending.as_ref().unwrap();assert_eq!(p.seat,enemy);assert_eq!(p.choice.options.len(),2);
    assert!(!p.choice.options.iter().any(|o|o.id==ally));
    choose(&mut g,vec![h2.clone()]);assert!(g.board(&h2).is_none());assert!(g.board(&h1).is_some());
    assert_eq!(g.pending.as_ref().unwrap().seat,other);restore(&mut g);
    choose(&mut g,vec!["region:2:marker:0".into()]);assert_eq!(g.regions[2].influence[g.team(other)],1);
    assert!(g.players[0].graveyard.iter().any(|c|c.definition=="JC065"));
}

#[test]
fn batch20_jc065_empty_or_revealed_hidden_still_represses_but_replaced_region_cancels() {
    for mode in 0..3 {
        let mut g=game();fund(&mut g,0,"JC063",1);
        let enemy=(1..4).find(|s|g.team(*s)!=g.team(0)).unwrap();let enemy_team=g.team(enemy);g.regions[1].influence[enemy_team]=1;
        let h=field(&mut g,"JC125",enemy);g.board_mut(&h).unwrap().face_down=mode!=0;
        let source=held(&mut g,"JC065",0);
        act(&mut g,0,Action{card_id:Some(source),region:Some(0),target_id:Some(player_id(enemy)),selected:Some(vec![player_id(enemy)]),..Action::new("play")});
        if mode==1 {g.board_mut(&h).unwrap().face_down=false;} // response result, no hidden remains
        if mode==2 {g.regions[0].card=g.make_card("DQJC109",0);} // exact-region guard primitive
        restore(&mut g);
        if mode==2 {top(&mut g);assert_eq!(g.regions[1].influence[g.team(enemy)],1);assert_eq!(resources(&g,0),0);}
        else {while g.pending.is_none(){pass(&mut g);}assert_eq!(g.pending.as_ref().unwrap().choice.kind,"influence");
            choose(&mut g,vec!["region:1:marker:0".into()]);assert_eq!(g.regions[1].influence[g.team(enemy)],0);}
    }
}

#[test]
fn batch20_jz29_entry_discard_and_own_vampire_reveal_return_are_distinct_optional_events() {
    let mut g=game();society(&mut g,0);fund(&mut g,0,"JC031",2);
    let source=held(&mut g,"JZ29",0);held(&mut g,"JC125",1);
    act(&mut g,0,Action{card_id:Some(source),target_id:Some("society:p0".into()),..Action::new("play")});
    while g.pending.is_none(){pass(&mut g);}
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"trigger");choose(&mut g,vec![player_id(1)]);
    while g.pending.is_none(){pass(&mut g);}let discard=g.pending.as_ref().unwrap().choice.options[0].id.clone();
    choose(&mut g,vec![discard]);assert!(g.players[1].hand.is_empty());
    let ring=g.attachments[0].card.id.clone();
    let teammate=(1..4).find(|s|g.team(*s)==g.team(0)).unwrap();
    let foreign=field(&mut g,"JC031",teammate);trigger(&mut g,&foreign,Event::Reveal);assert!(g.pending.is_none());
    let own=field(&mut g,"JC031",0);trigger(&mut g,&own,Event::Enter);assert!(g.pending.is_none());
    trigger(&mut g,&own,Event::Reveal);assert_eq!(g.pending.as_ref().unwrap().choice.min,Some(0));
    choose(&mut g,vec![]);assert!(g.in_play(&ring).is_some());
    trigger(&mut g,&own,Event::Reveal);choose(&mut g,vec!["accept".into()]);
    g.remove_dead(&own,RemovalCause::Destroy);restore(&mut g);top(&mut g);
    assert!(g.in_play(&ring).is_none());assert!(g.players[0].hand.iter().any(|c|c.definition=="JZ29"&&c.id!=ring));
}
