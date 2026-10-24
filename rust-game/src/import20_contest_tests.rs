//! Normal paid commands on explicit legal layouts. Persistence roundtrips here
//! cover legitimate continuations, not additional corrupt-save fuzzing.
use crate::{catalog, import20_contest as c, model::*, rules::*, unit_support::*};
use crate::jc029_tests::pass_top;

fn play(id: &str, key: &str, target: Option<&str>) -> Action {
    Action { card_id: Some(id.into()), ability_id: Some(key.into()),
        target_id: target.map(str::to_owned), ..Action::new("play") }
}
fn hand(id: &str) -> Action {
    Action { card_id: Some(id.into()), ability_id: Some(c::HAND.into()), ..Action::new("activate") }
}
fn priority(g: &mut Game, seat: usize) {
    for _ in 0..4 { if g.can_fast(seat) { return; } pass(g); }
    assert!(g.can_fast(seat));
}
fn start_contest(g: &mut Game) -> String {
    fund(g, 0, "JC125", 2);
    let source = held(g, "BQ090", 0);
    act(g, 0, hand(&source));
    assert!(!g.players[0].hand.iter().any(|c| c.id == source));
    pass_top(g);
    source
}

#[test]
fn import20_source_fields_and_closed_public_admission() {
    assert_eq!(catalog::ENGINE_VERSION, "rust-v0.2.108-import20-tax-ban");
    for id in c::CARDS {
        let d = catalog::card(id);
        assert!(!d.supported);
        assert!(catalog::runtime_implemented(d));
        assert_eq!(d.kind, "spell");
    }
    let d = catalog::card("BQ090");
    assert_eq!((d.cost, d.color.as_str(), d.magic.as_str()), (2, "黑", "死亡"));
    assert_eq!(d.loyalty, ["黑色"]); assert!(d.keywords.iter().any(|k| k == "非资产"));
    let d = catalog::card("BQ064");
    assert_eq!((d.cost, d.color.as_str()), (3, "灰")); assert_eq!(d.loyalty, ["灰色", "灰色"]);
    assert!(!d.text.contains("地区势力"));
    let d = definition("BQ090");
    assert_eq!(d.abilities[0].timing, Timing::Fast);
    assert_eq!(d.abilities[1].timing, Timing::Standard);
    validate_definitions(definitions()).unwrap();
}

#[test]
fn import20_bq090_paid_fast_destroy_accepts_each_controller_and_cost3() {
    for seat in 0..4 {
        let mut g = game(); let target = field(&mut g, "JC086", seat);
        fund(&mut g, 0, "JC084", 3); let source = held(&mut g, "BQ090", 0);
        let a = play(&source, c::DESTROY, Some(&target));
        assert!(g.legal_actions(0).iter().any(|x| x.action == a));
        act(&mut g, 0, a); assert_eq!(g.resources(0), 1);
        pass_top(&mut g); assert!(g.board(&target).is_none());
        assert!(g.players[seat].graveyard.iter().any(|c| c.definition == "JC086"));
        assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "BQ090").count(), 1);
    }
}

#[test]
fn import20_bq090_illegal_targets_payloads_loyalty_and_nonasset_rollback() {
    let mut g = game(); fund(&mut g, 0, "JC125", 2);
    let source = held(&mut g, "BQ090", 0); let t = field(&mut g, "JC125", 2);
    reject(&mut g, 0, play(&source, c::DESTROY, Some(&t))); // no black loyalty
    fund(&mut g, 0, "JC084", 1);
    let expensive = field(&mut g, "LC24", 2);
    let hidden = field(&mut g, "JC125", 2); g.board_mut(&hidden).unwrap().face_down = true;
    for target in [expensive, hidden, g.regions[0].card.id.clone(), "society:p0".into(), "stale".into()] {
        reject(&mut g, 0, play(&source, c::DESTROY, Some(&target)));
    }
    let mut extra = play(&source, c::DESTROY, Some(&t)); extra.selected = Some(vec![t.clone()]);
    reject(&mut g, 0, extra);
    reject(&mut g, 2, play(&source, c::DESTROY, Some(&t)));
    reject(&mut g, 0, Action { card_id: Some(source), ..Action::new("asset") });
}

#[test]
fn import20_bq090_fast_response_lost_original_target_does_not_rebind_or_refund() {
    let mut g = game(); let target = field(&mut g, "JC125", 2); let twin = field(&mut g, "JC125", 2);
    fund(&mut g, 0, "JC084", 2); fund(&mut g, 2, "JC084", 2);
    let lower = held(&mut g, "BQ090", 0); let upper = held(&mut g, "BQ090", 2);
    act(&mut g, 0, play(&lower, c::DESTROY, Some(&target)));
    priority(&mut g, 2);
    let a = play(&upper, c::DESTROY, Some(&target));
    assert!(g.legal_actions(2).iter().any(|x| x.action == a));
    act(&mut g, 2, a); pass_top(&mut g); assert_eq!(g.stack.len(), 1);
    assert!(g.board(&target).is_none()); pass_top(&mut g);
    assert!(g.board(&twin).is_some()); assert_eq!((g.resources(0), g.resources(2)), (0, 0));
}

#[test]
fn import20_bq090_attached_character_and_shielded_cost7_stays_illegal() {
    let mut g = game(); let host = field(&mut g, "JC029", 2);
    let bat = g.make_card("BQ026", 2); let target = bat.id.clone();
    g.attachments.push(Attachment { card: bat, host_id: host.clone() });
    fund(&mut g, 0, "JC084", 2); let source = held(&mut g, "BQ090", 0);
    act(&mut g, 0, play(&source, c::DESTROY, Some(&target))); pass_top(&mut g);
    assert!(g.board(&target).is_none()); assert!(g.board(&host).is_some());

    let mut g = game(); let team = g.team(2); g.begin_window(Window::Action(team));
    fund(&mut g, 2, "CQ17", 7); let source = held(&mut g, "BQ059", 2);
    let region = (0..g.regions.len()).find(|r| g.accessible(2, *r)).unwrap();
    act(&mut g, 2, Action { card_id: Some(source), region: Some(region), ..Action::new("deploy") });
    drain(&mut g);
    let target = g.regions[region].cards.iter().find(|c| c.definition == "BQ059").unwrap().id.clone();
    g.begin_window(Window::Action(0));
    fund(&mut g, 0, "JC084", 2); let source = held(&mut g, "BQ090", 0);
    assert_eq!(g.board(&target).unwrap().1.shield, 2);
    reject(&mut g, 0, play(&source, c::DESTROY, Some(&target)));
    assert_eq!(g.board(&target).unwrap().1.shield, 2); assert_eq!(g.resources(0), 2);
}

#[test]
fn import20_bq090_hand_pays2_discards_only_original_without_printed_loyalty() {
    let mut g = game(); fund(&mut g, 0, "JC125", 2);
    let source = held(&mut g, "BQ090", 0); let other = held(&mut g, "JC125", 0);
    let mut forged = hand(&source); forged.cost_selected = Some(vec![other.clone()]);
    reject(&mut g, 0, forged); reject(&mut g, 2, hand(&source));
    assert!(g.legal_actions(0).iter().any(|x| x.action == hand(&source)));
    act(&mut g, 0, hand(&source));
    assert_eq!(g.resources(0), 0); assert!(g.players[0].hand.iter().any(|c| c.id == other));
    assert!(!g.players[0].hand.iter().any(|c| c.id == source));
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "BQ090").count(), 1);
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert!(matches!(&f.already_paid[1], PaidCost::Discarded { old_instance, holder: 0, owner: 0 } if old_instance == &source));
    reject(&mut g, 0, hand(&source));
}

#[test]
fn import20_bq090_hand_rejects_insufficient_payment_foreign_source_and_response_window() {
    let mut g = game(); fund(&mut g, 0, "JC125", 1); let source = held(&mut g, "BQ090", 0);
    reject(&mut g, 0, hand(&source));
    fund(&mut g, 0, "JC125", 1); let foreign = held(&mut g, "BQ032", 0);
    reject(&mut g, 0, hand(&foreign));
    let board_source = field(&mut g, "BQ090", 0); reject(&mut g, 0, hand(&board_source));
    fund(&mut g, 0, "JC084", 2); let lower = held(&mut g, "BQ090", 0); let target = field(&mut g, "JC125", 2);
    act(&mut g, 0, play(&lower, c::DESTROY, Some(&target)));
    priority(&mut g, 0); reject(&mut g, 0, hand(&source));
    assert!(!g.legal_actions(0).iter().any(|x| x.action == hand(&source)));
}

#[test]
fn import20_bq064_shown_bonus7_private_commits_preserve_markers_and_history() {
    let mut g = game(); let shown = ["BQ064", "JC125", "JC125", "JC125"];
    for (seat, id) in shown.iter().enumerate() {
        deck_cards(&mut g, seat, &["JC125", "JC125"]); held(&mut g, id, seat);
    }
    g.players[0].society_zone.loss_control = 3;
    start_contest(&mut g);
    for (seat, id) in shown.iter().enumerate() {
        let pending = g.pending.as_ref().unwrap().clone(); assert_eq!(pending.seat, seat);
        for viewer in 0..4 {
            let v = g.view(viewer);
            assert_eq!(v.pending_choice.is_some(), viewer == seat);
        }
        let uid = g.players[seat].hand.iter().find(|c| c.definition == *id).unwrap().id.clone();
        choose(&mut g, vec![uid]);
    }
    while g.pending.as_ref().is_some_and(|p| p.choice.kind == crate::mystic_contest::REWARD_KIND) { choose(&mut g, vec![]); }
    assert!(g.mystic_contest.is_none());
    assert_eq!(g.mystic_history[0].shown[0].value, 7);
    assert_eq!(g.mystic_history[0].totals[g.team(0)], 7);
    assert_eq!(g.mystic_history[0].winner_team, Some(g.team(0)));
    assert_eq!(g.players[0].society_zone.loss_control, 3); restore(&mut g);
}

#[test]
fn import20_bq090_no_card_contest_keeps_costs_and_completed_draw_steps() {
    let mut g = game(); for seat in 0..4 { deck_cards(&mut g, seat, &["JC125"]); }
    g.players[2].deck.clear();
    start_contest(&mut g);
    assert!(g.mystic_contest.is_none() && g.pending.is_none()); assert!(g.mystic_history.is_empty());
    assert_eq!(g.resources(0), 0); assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[1].hand.len(), 1); assert!(!g.players[2].eliminated);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "BQ090").count(), 1);
}

#[test]
fn import20_bq064_zero_and_multiple_removals_use_own_actual_counter_only() {
    for (exposure, secrecy) in [(2, 0), (0, 4)] { for count in [0, 1, 3] {
        let mut g = game(); deck_cards(&mut g, 0, &["JC125", "JC125", "JC125", "JC125"]);
        g.players[0].society_zone.loss_control = count;
        g.players[0].society_zone.exposure = exposure; g.players[0].society_zone.secrecy = secrecy;
        g.players[2].society_zone.loss_control = 2; // existing legal maximum is3
        fund(&mut g, 0, "JC063", 4); let source = held(&mut g, "BQ064", 0);
        act(&mut g, 0, play(&source, c::SAMPLES, None));
        assert_eq!(g.players[0].society_zone.loss_control, count); // response, not yet resolved
        pass_top(&mut g); assert_eq!(g.players[0].society_zone.loss_control, 0);
        assert_eq!(g.players[0].hand.len(), count as usize); assert_eq!(g.resources(0), 1);
        assert_eq!((g.players[0].society_zone.exposure, g.players[0].society_zone.secrecy), (exposure, secrecy));
        assert_eq!(g.players[2].society_zone.loss_control, 2);
    }}
}

#[test]
fn import20_bq064_no_markers_empty_deck_survives_but_real_overdraw_eliminates() {
    for count in [0, 2] {
        let mut g = game(); deck_cards(&mut g, 0, if count == 0 { &[] } else { &["JC125"] });
        g.players[0].society_zone.loss_control = count;
        fund(&mut g, 0, "JC063", 3); let source = held(&mut g, "BQ064", 0);
        act(&mut g, 0, play(&source, c::SAMPLES, None)); pass_top(&mut g);
        assert_eq!(g.players[0].eliminated, count == 2);
        assert_eq!(g.players[0].society_zone.loss_control, 0);
        assert!(!g.players[1].eliminated);
    }
}

#[test]
fn import20_bq064_bad_loyalty_timing_payload_and_duplicate_do_not_spend() {
    let mut g = game(); fund(&mut g, 0, "JC125", 4); let source = held(&mut g, "BQ064", 0);
    reject(&mut g, 0, play(&source, c::SAMPLES, None));
    fund(&mut g, 0, "JC063", 1); reject(&mut g, 0, play(&source, c::SAMPLES, None));
    fund(&mut g, 0, "JC063", 1);
    let mut extra = play(&source, c::SAMPLES, Some("society:p2")); reject(&mut g, 0, extra.clone());
    extra.target_id = None; extra.selected = Some(vec!["3".into()]); reject(&mut g, 0, extra);
    act(&mut g, 0, play(&source, c::SAMPLES, None));
    reject(&mut g, 0, play(&source, c::SAMPLES, None));
    let other = held(&mut g, "BQ064", 0); priority(&mut g, 0);
    reject(&mut g, 0, play(&other, c::SAMPLES, None));
}

#[test]
fn import20_bq064_can_be_asset_and_has_no_hand_activation() {
    let mut g = game(); let source = held(&mut g, "BQ064", 0);
    reject(&mut g, 0, hand(&source));
    act(&mut g, 0, Action { card_id: Some(source), ..Action::new("asset") });
    assert_eq!(g.players[0].assets.len(), 1);
    let asset = g.players[0].assets[0].id.clone();
    reject(&mut g, 0, Action { card_id: Some(asset), ability_id: Some(c::SAMPLES.into()), ..Action::new("activate") });
}

#[test]
fn import20_registered_programs_reject_foreign_or_changed_ops() {
    let mut a = definition("BQ090").abilities[1].clone();
    assert!(validate_ability("BQ090", &a).is_ok());
    assert!(validate_ability("JC125", &a).is_err());
    a.ops = vec![Op::MysticContest { actor_bonus: 2 }]; assert!(validate_ability("BQ090", &a).is_err());
    let mut a = definition("BQ064").abilities[0].clone();
    a.ops = vec![Op::ForEachLivingPlayer(vec![Op::BQ064TakeSamples])];
    assert!(validate_ability("BQ064", &a).is_err()); assert!(validate_ability("JC125", &a).is_err());
}

#[test]
fn import20_room_commands_pay_once_and_duplicate_receipt_is_rejected() {
    use crate::room::{RoomCommand, RoomEnvelope, SessionAction};
    for id in c::CARDS {
        let mut g = game(); fund(&mut g, 0, "JC063", 3);
        let source = held(&mut g, id, 0); g.players[0].society_zone.loss_control = 2;
        let action = if *id == "BQ090" { hand(&source) } else { play(&source, c::SAMPLES, None) };
        let room = RoomEnvelope::from_game(g);
        let cmd = RoomCommand { command_id: format!("import20-{id}-once"), expected_version: room.revision,
            action: SessionAction::Game { action } };
        let first = room.transition(0, Some(cmd.clone()), 0).unwrap(); assert_eq!(first.outcome, "accepted");
        let room = RoomEnvelope::from_persisted(&first.state).unwrap();
        let saved = serde_json::to_value(&room).unwrap();
        let duplicate = room.transition(0, Some(cmd), 0).unwrap(); assert_eq!(duplicate.outcome, "rejected");
        assert!(!duplicate.changed && duplicate.journal.is_empty());
        assert_eq!(serde_json::to_value(RoomEnvelope::from_persisted(&duplicate.state).unwrap()).unwrap(), saved);
    }
}
