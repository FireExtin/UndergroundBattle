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
