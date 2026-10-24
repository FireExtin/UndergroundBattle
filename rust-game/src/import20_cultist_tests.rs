//! Author tests on explicit legal layouts. Shared helpers round-trip normal
//! commands and all four views; these are not independent/browser acceptance.
use crate::{catalog, import20_cultist as c, model::*, rules::*, unit_support::*};
use crate::jc029_tests::pass_top;

fn activate(source: &str, paid: &str) -> Action {
    Action { card_id: Some(source.into()), ability_id: Some(c::FAST.into()),
        cost_selected: Some(vec![paid.into()]), ..Action::new("activate") }
}
fn priority(g: &mut Game, actor: usize) {
    for _ in 0..8 { if g.can_fast(actor) { return; } pass(g); }
    panic!("missing response priority");
}
fn finish(g: &mut Game) {
    for _ in 0..100 {
        if let Some(p) = &g.pending {
            if p.choice.kind == "batch3-entry-order" { let id = p.choice.options[0].id.clone(); choose(g, vec![id]); }
            else { resolve_choice(g); }
        } else if !g.stack.is_empty() { pass(g); }
        else if !g.effects.is_empty() { g.drive().unwrap(); restore(g); }
        else { return; }
    }
    panic!("normal finite continuation did not finish");
}
fn deploy(g: &mut Game, actor: usize, id: &str) -> String {
    let region = (0..g.regions.len()).find(|r| g.accessible(actor, *r)).unwrap();
    let before = g.regions[region].cards.iter().map(|c| c.id.clone()).collect::<Vec<_>>();
    let source = held(g, id, actor);
    act(g, actor, Action { card_id: Some(source), region: Some(region), ..Action::new("deploy") });
    pass_top(g);
    g.regions[region].cards.iter().find(|c| c.definition == id && c.controller == actor && !before.contains(&c.id)).unwrap().id.clone()
}
fn entry(g: &mut Game) -> (String, String) {
    let source = field(g, c::ID, 0);
    g.players[0].society_zone.secrecy = 1;
    fund(g, 0, "JC042", 1);
    let entered = deploy(g, 0, "JC042");
    assert!(g.pending.as_ref().is_some_and(|p| matches!(&p.resolution,
        ChoiceResolution::Declare { declaration: d, .. } if d.ability.key == c::ENTRY)));
    (source, entered)
}
fn discount(g: &mut Game, source: &str, paid: &str) {
    priority(g, 0);
    let a = activate(source, paid);
    assert!(g.legal_actions(0).iter().any(|x| x.action == a));
    act(g, 0, a);
    assert!(g.board(paid).is_none());
    pass_top(g);
}
fn destruction_response(g: &mut Game, target: &str) {
    fund(g, 2, "JC084", 2);
    let spell = held(g, "BQ090", 2);
    priority(g, 2);
    let a = g.legal_actions(2).into_iter().find(|a| a.action.kind == "play"
        && a.action.card_id.as_deref() == Some(&spell) && a.action.target_id.as_deref() == Some(target)).unwrap().action;
    act(g, 2, a);
    pass_top(g);
    assert!(g.board(target).is_none());
}

#[test]
fn import20_xg13_original_fields_unique_two_loyalty_and_closed_public_pool() {
    let d = catalog::card(c::ID);
    assert_eq!((d.name.as_str(), d.subtitle.as_deref(), d.cost, d.color.as_str(), d.magic.as_str()),
        ("玛丽·林德博格", Some("狂信者"), 2, "红", "鲜血"));
    assert_eq!(d.loyalty, ["红色", "红色"]);
    assert_eq!(d.subtypes, ["人类", "邪教徒"]);
    assert!(d.unique && !d.supported && catalog::runtime_implemented(d));
    assert_eq!(d.permanent_icons, Icons { combat: 1, ..Icons::default() });
    assert_eq!(d.temporary_icons, Icons { influence: 1, ..Icons::default() });
    assert_eq!(d.defense, Some(1));
    assert!(d.text.contains("下一张鸣钟教派牌")); assert!(!d.text.contains("下一张灾难牌"));
    crate::unit_support::assert_current_serial_catalog();
    assert_eq!(catalog::catalog().cards.iter().filter(|d| d.supported).count(), 213);
    assert!(!catalog::catalog().cards.iter().any(|d| d.id == "XQ19"));
    assert_eq!(catalog::ENGINE_VERSION, "rust-v0.2.109-wm057-personal-borrow");
    assert_eq!(definition(c::ID).abilities[1].timing, Timing::Fast);
    assert_eq!(definition(c::ID).abilities[1].response_policy, ResponsePolicy::Respondable);
    validate_definitions(definitions()).unwrap();
}

#[test]
fn import20_xg13_real_deploy_retains_double_loyalty_and_does_not_observe_itself() {
    let mut g = game(); fund(&mut g, 0, "JC042", 1); fund(&mut g, 0, "JC125", 1);
    let hand = held(&mut g, c::ID, 0);
    reject(&mut g, 0, Action { card_id: Some(hand.clone()), region: Some(0), ..Action::new("deploy") });
    fund(&mut g, 0, "JC042", 1); g.players[0].society_zone.secrecy = 1;
    act(&mut g, 0, Action { card_id: Some(hand), region: Some(0), ..Action::new("deploy") });
    assert_eq!(g.resources(0), 1); pass_top(&mut g);
    assert!(g.pending.is_none());
    let actual = g.regions[0].cards.iter().find(|c| c.definition == c::ID).unwrap();
    assert_eq!(actual.time_markers, 0); restore(&mut g);
}

#[test]
fn import20_xg13_entry_gives_exactly_one_time_to_original_own_cultist() {
    let mut g = game(); let (_, target) = entry(&mut g);
    let unrelated = field(&mut g, "JC042", 0);
    choose(&mut g, vec!["accept".into()]); pass_top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.time_markers, 1);
    assert_eq!(g.board(&unrelated).unwrap().1.time_markers, 0);
    assert!(g.pending.is_none()); restore(&mut g);
}

#[test]
fn import20_xg13_entry_can_be_declined_without_rebinding_or_markers() {
    let mut g = game(); let (_, target) = entry(&mut g);
    choose(&mut g, vec![]);
    assert!(g.stack.is_empty() && g.pending.is_none());
    assert_eq!(g.board(&target).unwrap().1.time_markers, 0); restore(&mut g);
}

#[test]
fn import20_xg13_entry_condition_is_personal_and_later_moon_does_not_create_missed_event() {
    let mut g = game(); field(&mut g, c::ID, 0);
    g.players[1].society_zone.secrecy = 2; fund(&mut g, 0, "JC042", 1);
    let target = deploy(&mut g, 0, "JC042");
    assert!(g.pending.is_none()); assert_eq!(g.board(&target).unwrap().1.time_markers, 0);
    g.players[0].society_zone.secrecy = 1; g.drive().unwrap();
    assert!(g.pending.is_none()); assert_eq!(g.board(&target).unwrap().1.time_markers, 0); restore(&mut g);
}

#[test]
fn import20_xg13_later_secrecy_loss_does_not_erase_qualified_entry() {
    let mut g = game(); let (_, target) = entry(&mut g);
    choose(&mut g, vec!["accept".into()]);
    // A normal enemy exposure effect changes the physical moon through the
    // shared society operation, after this trigger was already declared.
    g.add_society_markers(0, 1, 0); restore(&mut g);
    assert_eq!(g.players[0].society_zone.secrecy, 0); pass_top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.time_markers, 1);
}

#[test]
fn import20_xg13_ally_enemy_hidden_and_noncultist_entries_do_not_qualify() {
    for actor in [1, 2, 3] {
        let mut g = game(); field(&mut g, c::ID, 0); g.players[0].society_zone.secrecy = 1;
        g.begin_window(Window::Action(g.team(actor))); fund(&mut g, actor, "JC042", 1);
        let target = deploy(&mut g, actor, "JC042");
        assert!(g.pending.is_none()); assert_eq!(g.board(&target).unwrap().1.time_markers, 0);
    }
    let mut g = game(); field(&mut g, c::ID, 0); g.players[0].society_zone.secrecy = 1;
    let target = deploy(&mut g, 0, "JC125"); assert!(g.pending.is_none()); assert_eq!(g.board(&target).unwrap().1.time_markers, 0);
    let source = held(&mut g, "JC042", 0); fund(&mut g, 0, "JC125", 1);
    act(&mut g, 0, Action { card_id: Some(source), region: Some(0), ..Action::new("conceal") });
    assert!(g.pending.is_none()); restore(&mut g);
}

#[test]
fn import20_xg13_real_reveal_of_own_cultist_is_entry_but_hidden_observer_is_inactive() {
    let mut g = game(); field(&mut g, c::ID, 0); g.players[0].society_zone.secrecy = 1;
    let source = field(&mut g, "JC042", 0); g.board_mut(&source).unwrap().face_down = true;
    fund(&mut g, 0, "JC042", 1);
    act(&mut g, 0, Action { card_id: Some(source), ..Action::new("reveal") }); pass_top(&mut g);
    assert!(g.pending.as_ref().is_some_and(|p| matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.ability.key == c::ENTRY)));
    choose(&mut g, vec!["accept".into()]); pass_top(&mut g);
    assert_eq!(g.regions[0].cards.iter().find(|c| c.definition == "JC042").unwrap().time_markers, 1);
    let mut g = game(); let source = field(&mut g, c::ID, 0); g.board_mut(&source).unwrap().face_down = true;
    g.players[0].society_zone.secrecy = 1; fund(&mut g, 0, "JC042", 1);
    deploy(&mut g, 0, "JC042"); assert!(g.pending.is_none());
}

#[test]
fn import20_xg13_source_destroyed_in_real_response_does_not_cancel_entry() {
    let mut g = game(); let (source, target) = entry(&mut g);
    choose(&mut g, vec!["accept".into()]); destruction_response(&mut g, &source); pass_top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.time_markers, 1); restore(&mut g);
}

#[test]
fn import20_xg13_entered_target_destroyed_in_response_does_not_rebind_same_print() {
    let mut g = game(); let (_, target) = entry(&mut g);
    choose(&mut g, vec!["accept".into()]); destruction_response(&mut g, &target);
    let replacement = field(&mut g, "JC042", 0); pass_top(&mut g);
    assert_eq!(g.board(&replacement).unwrap().1.time_markers, 0); restore(&mut g);
}

#[test]
fn import20_xg13_fast_sacrifice_is_real_other_cultist_and_source_need_not_be_ready() {
    let mut g = game(); let source = field(&mut g, c::ID, 0); let cost = field(&mut g, "JC042", 0);
    g.board_mut(&source).unwrap().exhausted = true;
    discount(&mut g, &source, &cost);
    assert!(g.board(&source).unwrap().1.exhausted);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "JC042").count(), 1);
    assert_eq!(g.modifiers.len(), 1); assert_eq!((g.modifiers[0].amount, g.modifiers[0].uses, g.modifiers[0].paid_reveal), (2, 1, false));
}

#[test]
fn import20_xg13_actual_wm038_granted_cultist_can_pay_then_tattoo_leaves_with_host() {
    let mut g = game(); let source = field(&mut g, c::ID, 0); let host = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "JC042", 1); let tattoo = held(&mut g, "WM038", 0);
    act(&mut g, 0, Action { card_id: Some(tattoo), target_id: Some(host.clone()), ..Action::new("play") });
    finish(&mut g);
    assert!(g.current_subtypes(g.board(&host).unwrap().1).iter().any(|s| s == "邪教徒"));
    discount(&mut g, &source, &host);
    assert!(g.attachments.is_empty()); assert_eq!(g.modifiers[0].uses, 1);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "WM038").count(), 1);
}

#[test]
fn import20_xg13_own_xq18_reveal_and_observer_share_real_entry_order_once() {
    let mut g = game(); field(&mut g, c::ID, 0); g.players[0].society_zone.secrecy = 1;
    let hidden = field(&mut g, "XQ18", 0); g.board_mut(&hidden).unwrap().face_down = true;
    fund(&mut g, 0, "JC042", 2);
    act(&mut g, 0, Action { card_id: Some(hidden), ..Action::new("reveal") }); pass_top(&mut g);
    let mut ordered = false; let mut accepted = 0;
    for _ in 0..100 {
        if let Some(p) = g.pending.clone() {
            if p.choice.kind == "batch3-entry-order" {
                ordered = true;
                let id = p.choice.options.iter().find(|o| o.id.ends_with(c::ENTRY)).unwrap_or(&p.choice.options[0]).id.clone();
                choose(&mut g, vec![id]);
            } else if matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.ability.key == c::ENTRY) {
                accepted += 1; choose(&mut g, vec!["accept".into()]);
            } else if p.choice.allow_decline == Some(true) { choose(&mut g, vec![]); }
            else { resolve_choice(&mut g); }
        } else if !g.stack.is_empty() { pass(&mut g); }
        else if !g.effects.is_empty() { g.drive().unwrap(); restore(&mut g); }
        else { break; }
    }
    assert!(ordered); assert_eq!(accepted, 1);
    assert_eq!(g.regions[0].cards.iter().find(|c| c.definition == "XQ18").unwrap().time_markers, 1);
    assert!(g.entry_order_groups.is_empty()); restore(&mut g);
}

#[test]
fn import20_xg13_real_room_activation_consumes_one_cost_and_rejects_stale_repeat() {
    use crate::room::{RoomCommand, RoomEnvelope};
    let mut g = game(); let source = field(&mut g, c::ID, 0); let paid = field(&mut g, "JC042", 0);
    let r = crate::jc029_tests::envelope(&g);
    let command = RoomCommand { command_id: "xg13-paid-once".into(), expected_version: r.revision, action: activate(&source, &paid).into() };
    let out = r.transition(0, Some(command.clone()), 2000).unwrap();
    assert!(out.error_code.is_none(), "{:?}", out.error_message);
    let next = RoomEnvelope::from_persisted(&out.state).unwrap();
    assert_eq!(serde_json::to_value(r.replay_events(&out.journal).unwrap()).unwrap(), serde_json::to_value(&next).unwrap());
    assert!(next.game.board(&paid).is_none());
    assert_eq!(next.game.players[0].graveyard.iter().filter(|c| c.definition == "JC042").count(), 1);
    let repeat = next.transition(0, Some(command), 2001).unwrap();
    assert_eq!(repeat.error_code.as_deref(), Some("version_conflict")); assert_eq!(repeat.state, out.state);
    assert_eq!(next.game.seed, g.seed);
}

#[test]
fn import20_xg13_illegal_sacrifices_and_extra_payloads_roll_back() {
    let mut g = game(); let source = field(&mut g, c::ID, 0);
    let ally = field(&mut g, "JC042", 1); let enemy = field(&mut g, "JC042", 2);
    let noncultist = field(&mut g, "JC125", 0); let hidden = field(&mut g, "JC042", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    for id in [&source, &ally, &enemy, &noncultist, &hidden, "unknown"] { reject(&mut g, 0, activate(&source, id)); }
    let good = field(&mut g, "JC042", 0);
    for a in [Action { target_id: Some(enemy), ..activate(&source, &good) },
        Action { region: Some(0), ..activate(&source, &good) },
        Action { cost_selected: Some(vec![good.clone(), good.clone()]), ..activate(&source, &good) }] { reject(&mut g, 0, a); }
    assert_eq!(g.modifiers.len(), 0);
    priority(&mut g, 2); reject(&mut g, 2, activate(&source, &good));
}

#[test]
fn import20_xg13_discount_red_faction_or_blood_including_nonred_but_not_all_red() {
    for id in ["XQ18", "JC029"] {
        let mut g = game(); let source = field(&mut g, c::ID, 0); let cost = field(&mut g, "JC042", 0);
        discount(&mut g, &source, &cost); fund(&mut g, 0, if id == "JC029" { "JC029" } else { "JC042" }, 2);
        let hand = held(&mut g, id, 0); assert_eq!(g.effective_cost(0, &g.players[0].hand.iter().find(|c| c.id == hand).unwrap()), 0);
        act(&mut g, 0, Action { card_id: Some(hand), region: Some(0), ..Action::new("deploy") });
        assert_eq!(g.resources(0), 2); assert_eq!(g.modifiers[0].uses, 0); finish(&mut g);
    }
    let mut g = game(); let source = field(&mut g, c::ID, 0); let cost = field(&mut g, "JC042", 0);
    discount(&mut g, &source, &cost); let hand = held(&mut g, "JC051", 0);
    let c = g.players[0].hand.iter().find(|c| c.id == hand).unwrap();
    assert_eq!(catalog::card("JC051").color, "红"); assert_eq!(g.effective_cost(0, c), 6);
}

#[test]
fn import20_xg13_nonmatching_play_preserves_discount_for_actual_next_qualifying_play() {
    let mut g = game(); let source = field(&mut g, c::ID, 0); let cost = field(&mut g, "JC042", 0);
    discount(&mut g, &source, &cost); fund(&mut g, 0, "JC084", 3);
    let hand = held(&mut g, "JC086", 0);
    act(&mut g, 0, Action { card_id: Some(hand), region: Some(0), ..Action::new("deploy") });
    finish(&mut g); assert_eq!(g.modifiers[0].uses, 1);
    fund(&mut g, 0, "JC042", 2); let hand = held(&mut g, "XQ18", 0);
    act(&mut g, 0, Action { card_id: Some(hand), region: Some(0), ..Action::new("deploy") });
    assert_eq!(g.resources(0), 2); assert_eq!(g.modifiers[0].uses, 0); finish(&mut g);
}

#[test]
fn import20_xg13_discount_never_waives_loyalty_and_failed_play_does_not_consume_it() {
    let mut g = game(); let source = field(&mut g, c::ID, 0); let cost = field(&mut g, "JC042", 0);
    discount(&mut g, &source, &cost); let hand = held(&mut g, "JC029", 0); fund(&mut g, 0, "JC042", 2);
    reject(&mut g, 0, Action { card_id: Some(hand.clone()), region: Some(0), ..Action::new("deploy") });
    assert_eq!(g.modifiers[0].uses, 1); fund(&mut g, 0, "JC029", 2);
    act(&mut g, 0, Action { card_id: Some(hand), region: Some(0), ..Action::new("deploy") });
    assert_eq!(g.modifiers[0].uses, 0); finish(&mut g);
}

#[test]
fn import20_xg13_discount_is_personal_and_not_secret_deploy_or_paid_reveal() {
    let mut g = game(); let source = field(&mut g, c::ID, 0); let cost = field(&mut g, "JC042", 0);
    discount(&mut g, &source, &cost); fund(&mut g, 0, "JC042", 4);
    let hand = held(&mut g, "XQ18", 0);
    act(&mut g, 0, Action { card_id: Some(hand), region: Some(0), ..Action::new("conceal") });
    let hidden = g.regions[0].cards.iter().find(|c| c.definition == "XQ18").unwrap().id.clone();
    assert_eq!(g.modifiers[0].uses, 1);
    act(&mut g, 0, Action { card_id: Some(hidden), ..Action::new("reveal") });
    assert_eq!(g.resources(0), 1); assert_eq!(g.modifiers[0].uses, 1); finish(&mut g);
    fund(&mut g, 1, "JC042", 2); let hand = held(&mut g, "XQ18", 1);
    let region = (0..g.regions.len()).find(|r| g.accessible(1, *r)).unwrap();
    act(&mut g, 1, Action { card_id: Some(hand), region: Some(region), ..Action::new("deploy") });
    assert_eq!(g.resources(1), 0); assert_eq!(g.modifiers[0].uses, 1); finish(&mut g);
}

#[test]
fn import20_xg13_repeated_sacrifices_stack_clamp_and_consume_together_once() {
    let mut g = game(); let source = field(&mut g, c::ID, 0);
    for _ in 0..2 { let cost = field(&mut g, "JC042", 0); discount(&mut g, &source, &cost); }
    assert_eq!(g.modifiers.len(), 2); fund(&mut g, 0, "JC042", 2);
    let hand = held(&mut g, "XQ18", 0);
    act(&mut g, 0, Action { card_id: Some(hand), region: Some(0), ..Action::new("deploy") });
    assert!(g.modifiers.iter().all(|m| m.uses == 0)); assert_eq!(g.resources(0), 2); finish(&mut g);
    let next = held(&mut g, "XQ18", 0);
    assert_eq!(g.effective_cost(0, g.players[0].hand.iter().find(|c| c.id == next).unwrap()), 2);
}

#[test]
fn import20_xg13_cost_paid_before_response_and_source_departure_preserves_discount() {
    let mut g = game(); let source = field(&mut g, c::ID, 0); let cost = field(&mut g, "JC042", 0);
    act(&mut g, 0, activate(&source, &cost)); assert!(g.board(&cost).is_none());
    destruction_response(&mut g, &source); pass_top(&mut g);
    assert_eq!(g.modifiers.len(), 1); assert_eq!(g.modifiers[0].uses, 1); restore(&mut g);
}

#[test]
fn import20_xg13_complete_definition_rejects_transplanted_or_truncated_programs() {
    let mut all = definitions().clone(); all.get_mut(c::ID).unwrap().abilities.pop();
    assert!(validate_definitions(&all).is_err());
    let mut all = definitions().clone(); all.get_mut("JC125").unwrap().abilities.push(c::definition().abilities[0].clone());
    assert!(validate_definitions(&all).is_err());
    let mut all = definitions().clone(); all.get_mut(c::ID).unwrap().abilities[1].costs.clear();
    assert!(validate_definitions(&all).is_err());
    let mut all = definitions().clone(); all.get_mut(c::ID).unwrap().abilities[1].response_policy = ResponsePolicy::Immediate;
    assert!(validate_definitions(&all).is_err());
}
