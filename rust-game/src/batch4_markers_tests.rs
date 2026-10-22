//! Native layouts are explicit; commands and all four views round-trip.
use crate::{catalog, model::*, rules::*, unit_support::*};
use crate::jc029_tests::pass_top;

fn layout() -> (Game, String, String, String) {
    let mut g = game(); fund(&mut g, 0, "XQ21", 3);
    let target = field(&mut g, "JZ51", 0); g.board_mut(&target).unwrap().time_markers = 1;
    let fee = field(&mut g, "JC125", 0); let spell = held(&mut g, "XQ21", 0);
    (g, spell, target, fee)
}
fn command(spell: &str, target: &str, fee: &str) -> Action {
    Action { card_id: Some(spell.into()), ability_id: Some("batch4-xq21-copy-marker".into()), target_id: Some(target.into()), cost_selected: Some(vec![fee.into()]), ..Action::new("play") }
}
fn pending(g: &mut Game) {
    for _ in 0..16 { if g.pending.is_some() { break; } pass(g); }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "xq21-marker");
}
fn finish(g: &mut Game, kind: &str) { pending(g); choose(g, vec![kind.into()]); }
fn rejected_by_both(original: &Game, altered: &Game) {
    let game_error = Game::from_persisted(&serde_json::to_string(altered).unwrap()).unwrap_err();
    let mut room = serde_json::to_value(crate::jc029_tests::envelope(original)).unwrap();
    room["game"] = serde_json::to_value(altered).unwrap();
    let room_error = crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).unwrap_err();
    assert_eq!(game_error, room_error);
}

#[test]
fn batch4_markers_fixed_fast_ritual_and_no_public_admission() {
    let c = catalog::card("XQ21"); assert_eq!(c.cost, 1); assert_eq!(c.color, "红"); assert_eq!(c.subtypes, vec!["仪式"]); assert!(c.magic.is_empty()); assert!(!c.supported);
    let d = definition("XQ21"); let a = &d.abilities[0]; assert_eq!(a.timing, Timing::Fast); assert!(a.play_only);
    assert!(matches!(a.costs.as_slice(), [Cost::SacrificeSelectedControlledCharacter]));
    let mut all = crate::rules::definitions().clone(); all.get_mut("JC062").unwrap().abilities[0] = a.clone(); assert!(crate::rules::validate_definitions(&all).is_err());
}
#[test]
fn batch4_markers_real_fee_and_sacrifice_commit_once_then_copy_one() {
    let (mut g, spell, target, fee) = layout(); let a = command(&spell, &target, &fee); let resources = g.resources(0);
    assert!(g.legal_actions(0).iter().any(|v| v.action == a)); act(&mut g, 0, a.clone());
    assert_eq!(g.resources(0), resources - 1); assert!(g.in_play(&fee).is_none()); assert_eq!(g.board(&target).unwrap().1.time_markers, 1);
    assert!(matches!(g.stack[0].frame.as_ref().unwrap().already_paid.as_slice(), [PaidCost::Assets(_), PaidCost::Sacrificed { old_instance, .. }] if old_instance == &fee));
    finish(&mut g, "time"); assert_eq!(g.board(&target).unwrap().1.time_markers, 2); assert_eq!(g.resources(0), resources - 1);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "XQ21").count(), 1); reject(&mut g, 0, a);
}
#[test]
fn batch4_markers_missing_duplicate_foreign_hidden_and_asset_fees_are_atomic() {
    let (mut g, spell, target, fee) = layout(); let enemy = field(&mut g, "JC125", 1); let hidden = field(&mut g, "JC125", 0); g.board_mut(&hidden).unwrap().face_down = true;
    for ids in [None, Some(vec![]), Some(vec![fee.clone(), fee.clone()]), Some(vec![enemy]), Some(vec![hidden]), Some(vec![g.players[0].assets[0].id.clone()])] {
        let mut a = command(&spell, &target, &fee); a.cost_selected = ids; reject(&mut g, 0, a);
    }
    for mutation in 0..3 { let mut bad = g.clone(); if mutation == 0 {bad.players[0].assets.clear();} else if mutation == 1 {bad.players[0].assets.iter_mut().for_each(|c| c.exhausted = true);} else {for c in &mut bad.players[0].assets { c.definition = "JC029".into(); }} reject(&mut bad, 0, command(&spell, &target, &fee)); }
}
#[test]
fn batch4_markers_damage_printed_traits_and_offboard_objects_are_not_markers() {
    let (mut g, spell, target, fee) = layout(); g.board_mut(&target).unwrap().time_markers = 0; g.board_mut(&target).unwrap().damage = 1;
    reject(&mut g, 0, command(&spell, &target, &fee));
    for id in [g.players[0].assets[0].id.clone(), spell.clone(), "region:0".into()] {reject(&mut g, 0, command(&spell, &id, &fee));}
    let printed = field(&mut g, "BQ042", 0); reject(&mut g, 0, command(&spell, &printed, &fee));
}
#[test]
fn batch4_markers_extra_command_fields_and_wrong_actor_are_rejected() {
    let (mut g, spell, target, fee) = layout(); let a = command(&spell, &target, &fee);
    for s in 1..4 { reject(&mut g, s, a.clone()); }
    for n in 0..5 { let mut b = a.clone(); match n {0 => b.region = Some(0),1 => b.selected = Some(vec![target.clone()]),2 => b.option = Some("time".into()),3 => b.allocations = Some(Default::default()),_ => b.ability_id = Some("foreign".into())}; reject(&mut g, 0, b); }
}
#[test]
fn batch4_markers_each_existing_physical_kind_increases_only_that_kind() {
    for kind in ["permanent-combat", "permanent-defense", "wound", "shield", "lock"] {
        let (mut g, spell, mut target, fee) = layout(); if kind == "permanent-defense" { target = field(&mut g, "BQ079", 0); g.board_mut(&target).unwrap().permanent_combat_markers = 1; } let c = g.board_mut(&target).unwrap(); c.time_markers = 0;
        match kind {"permanent-combat" => c.permanent_combat_markers = 1,"permanent-defense" => c.permanent_defense_markers = 1,"wound" => c.wounds = 1,"shield" => c.shield = 1,_ => c.lock_markers = 1};
        act(&mut g, 0, command(&spell, &target, &fee)); finish(&mut g, kind); let c = g.board(&target).unwrap().1;
        assert_eq!(match kind {"permanent-combat" => c.permanent_combat_markers,"permanent-defense" => c.permanent_defense_markers,"wound" => c.wounds,"shield" => c.shield,_ => c.lock_markers}, 2);
        assert_eq!(c.time_markers, 0); assert_eq!(c.damage, 0);
    }
}
#[test]
fn batch4_markers_hidden_lock_target_stays_hidden_in_all_seat_views() {
    let (mut g, spell, _, fee) = layout(); let target = field(&mut g, "JC125", 2); g.board_mut(&target).unwrap().face_down = true; g.board_mut(&target).unwrap().lock_markers = 1;
    act(&mut g, 0, command(&spell, &target, &fee)); finish(&mut g, "lock");
    for s in 0..4 { let v = g.view(s); let c = v.regions[0].characters.iter().find(|c| c.instance_id == target).unwrap(); assert!(c.face_down); assert_eq!(c.lock_markers, Some(2)); if s != 2 {assert!(c.card_id.is_none() && c.text.is_none());} }
}
#[test]
fn batch4_markers_attached_carrier_marks_qualify_host_without_moving_them() {
    let (mut g, spell, _, fee) = layout(); let target = field(&mut g, "JC029", 0);
    let mut bat = g.make_card("BQ026", 0); bat.time_markers = 2; bat.lock_markers = 1; let id = bat.id.clone(); g.attachments.push(Attachment { card: bat, host_id: target.clone() });
    act(&mut g, 0, command(&spell, &target, &fee)); pending(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.iter().map(|o| o.id.as_str()).collect::<Vec<_>>(), vec!["time", "lock"]);
    choose(&mut g, vec!["time".into()]); assert_eq!(g.board(&target).unwrap().1.time_markers, 1); assert_eq!(g.board(&id).unwrap().1.time_markers, 2); assert_eq!(g.effective_time_markers(g.board(&target).unwrap().1), 3);
}
#[test]
fn batch4_markers_attached_character_is_a_real_sacrifice_candidate() {
    let (mut g, spell, target, _) = layout(); let host = field(&mut g, "JC029", 0); let c = g.make_card("BQ026", 0); let fee = c.id.clone(); g.attachments.push(Attachment { card: c, host_id: host });
    let a = command(&spell, &target, &fee); assert!(g.legal_actions(0).iter().any(|v| v.action == a));
    act(&mut g, 0, a); finish(&mut g, "time"); assert!(g.in_play(&fee).is_none()); assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "BQ026").count(), 1);
}
#[test]
fn batch4_markers_choice_cannot_add_absent_kind_quantity_or_pay_again() {
    let (mut g, spell, target, fee) = layout(); act(&mut g, 0, command(&spell, &target, &fee)); pending(&mut g);
    let id = g.pending.as_ref().unwrap().choice.id.clone();
    for selected in [vec![], vec!["lock".into()], vec!["time".into(), "time".into()]] {reject(&mut g, 0, Action { choice_id: Some(id.clone()), selected: Some(selected), ..Action::new("choose") });}
    let mut a = Action { choice_id: Some(id), selected: Some(vec!["time".into()]), ..Action::new("choose") }; a.cost_selected = Some(vec![fee]); reject(&mut g, 0, a);
    choose(&mut g, vec!["time".into()]); assert_eq!(g.board(&target).unwrap().1.time_markers, 2);
}
#[test]
fn batch4_markers_sacrificing_original_target_is_paid_then_honestly_fizzles() {
    let (mut g, spell, target, _) = layout(); let resources = g.resources(0); act(&mut g, 0, command(&spell, &target, &target));
    assert!(g.in_play(&target).is_none()); pass_top(&mut g); assert!(g.pending.is_none()); assert_eq!(g.resources(0), resources - 1); assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "XQ21").count(), 1);
}
#[test]
fn batch4_markers_real_rescue_invalidates_original_uid_without_refund_or_retarget() {
    let (mut g, spell, target, fee) = layout(); let rescuer = field(&mut g, "JC075", 0); act(&mut g, 0, command(&spell, &target, &fee));
    for _ in 0..4 {if g.priority_team == 0 {break;} pass(&mut g);}
    let rescue = g.legal_actions(0).into_iter().find(|v| v.action.card_id.as_deref() == Some(&rescuer) && v.action.target_id.as_deref() == Some(&target)).unwrap().action;
    act(&mut g, 0, rescue); pass_top(&mut g); assert!(g.in_play(&target).is_none()); let resources = g.resources(0); pass_top(&mut g);
    assert!(g.pending.is_none()); assert_eq!(g.resources(0), resources); let returned = g.players[0].hand.iter().find(|c| c.definition == "JZ51").unwrap(); assert_ne!(returned.id, target); assert_eq!(returned.time_markers, 0);
}
#[test]
fn batch4_markers_resolution_uses_current_kinds_and_last_marker_loss_fizzles() {
    for keep_lock in [false, true] { let (mut g, spell, target, fee) = layout(); act(&mut g, 0, command(&spell, &target, &fee));
        // Explicit response-layout change, not a claim about a separate card.
        let c = g.board_mut(&target).unwrap(); c.time_markers = 0; c.lock_markers = u32::from(keep_lock); restore(&mut g);
        if keep_lock {finish(&mut g, "lock"); assert_eq!(g.board(&target).unwrap().1.lock_markers, 2);} else {pass_top(&mut g); assert!(g.pending.is_none());}
        assert_eq!(g.board(&target).unwrap().1.time_markers, 0);
    }
}
#[test]
fn batch4_markers_plan_copy_uses_existing_threshold_and_single_owner_completion() {
    let mut g = game(); fund(&mut g, 0, "WM046", 4); fund(&mut g, 0, "XQ21", 1); g.add_society_markers(0, 0, 1);
    let plan = held(&mut g, "WM046", 0); act(&mut g, 0, Action {card_id: Some(plan), target_id: Some("society:p0".into()), option: Some("secret-plan".into()), ..Action::new("play")}); top(&mut g);
    let target = g.attachments[0].card.id.clone(); act(&mut g, 0, Action {card_id: Some(target.clone()), ability_id: Some(crate::batch4_plans::ADVANCE.into()), ..Action::new("activate")}); top(&mut g);
    let fee = field(&mut g, "JC125", 0); let spell = held(&mut g, "XQ21", 0); act(&mut g, 0, command(&spell, &target, &fee)); finish(&mut g, "advancement");
    let c = g.in_play(&target).unwrap().1; assert!(!c.face_down); assert_eq!(c.advancement_markers, 2);
    assert_eq!(g.stack.iter().filter(|i| i.frame.as_ref().is_some_and(|f| f.ability_key == "batch4-plan-complete" && f.source.card.id == target)).count(), 1);
    top(&mut g); assert!(g.in_play(&target).is_none()); assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "WM046").count(), 1);
}
#[test]
fn batch4_markers_enemy_plan_completion_belongs_to_plan_controller() {
    let (mut g, spell, _, fee) = layout();
    // Explicit legal hidden-plan layout, one step below its printed threshold.
    let mut plan = g.make_card("WM046", 2); plan.face_down = true; plan.advancement_markers = 1; let target = plan.id.clone();
    g.attachments.push(Attachment {card: plan, host_id: "society:p2".into()});
    let resources = g.resources(0); act(&mut g, 0, command(&spell, &target, &fee)); pending(&mut g);
    for seat in [0, 1, 3] {let v = g.view(seat); let c = &v.attachments.iter().find(|a|a.card.instance_id == target).unwrap().card; assert!(c.card_id.is_none()); assert_eq!(c.name, "秘密计划");}
    choose(&mut g, vec!["advancement".into()]); assert_eq!(g.resources(0), resources - 1);
    let f = g.stack.last().unwrap().frame.as_ref().unwrap(); assert_eq!(f.actor, 2); assert_eq!(f.source.card.controller, 2); assert_eq!(f.source.card.id, target);
    top(&mut g); assert_eq!(g.players[2].graveyard.iter().filter(|c|c.definition == "WM046").count(), 1);
}
#[test]
fn batch4_markers_enemy_shield_cancels_whole_copy_after_payment() {
    let (mut g, spell, _, fee) = layout(); let target = field(&mut g, "JC125", 2); g.board_mut(&target).unwrap().shield = 1;
    let resources = g.resources(0); act(&mut g, 0, command(&spell, &target, &fee)); pass_top(&mut g);
    assert!(g.pending.is_none()); assert_eq!(g.board(&target).unwrap().1.shield, 0); assert_eq!(g.resources(0), resources - 1); assert!(g.in_play(&fee).is_none());
    assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition == "XQ21").count(), 1);
}
#[test]
fn batch4_markers_corrupt_program_receipts_shell_and_pending_reject_in_both_entries() {
    let (mut g, spell, target, fee) = layout(); act(&mut g, 0, command(&spell, &target, &fee));
    for n in 0..7 {let mut bad = g.clone(); let f = bad.stack[0].frame.as_mut().unwrap(); match n {0 => f.cursor = 99,1 => f.already_paid.clear(),2 => f.steps[0].op = Op::JZ51GraveEntry,3 => f.chosen_region = Some(0),4 => f.targets[0].spec.kind = EntityKind::Hidden,5 => f.source.observed_exposure = Some(1),_ => f.actor = 2}; rejected_by_both(&g, &bad);}
    let mut bad = g.clone(); bad.stack.push(bad.stack[0].clone()); rejected_by_both(&g, &bad);
    pending(&mut g);
    for n in 0..5 {let mut bad = g.clone(); let p = bad.pending.as_mut().unwrap(); match n {0 => p.seat = 2,1 => p.choice.options[0].id = "lock".into(),2 => p.choice.max = Some(2),3 => p.choice.allow_decline = Some(true),_ => if let ChoiceResolution::Frame {choice: FrameChoice::XQ21Marker {kinds}, ..} = &mut p.resolution {kinds.clear();}} rejected_by_both(&g, &bad);}
}
fn room_step(r: &mut crate::room::RoomEnvelope, seat: usize, action: crate::room::SessionAction, now: u64) -> crate::room::RoomCommand {
    let cmd = crate::room::RoomCommand {command_id: format!("marker-room-{}", r.revision), expected_version: r.revision, action};
    let result = r.transition(seat, Some(cmd.clone()), now).unwrap(); assert!(result.error_code.is_none(), "{:?}", result.error_message);
    *r = crate::room::RoomEnvelope::from_persisted(&result.state).unwrap(); cmd
}
#[test]
fn batch4_markers_room_response_pause_choice_replay_and_large_u64() {
    use crate::room::{RoomEnvelope, SessionAction}; let (mut g, spell, target, fee) = layout(); g.seed = u64::MAX - 37; g.random = g.seed;
    act(&mut g, 0, command(&spell, &target, &fee)); let mut r = crate::jc029_tests::envelope(&g); let resources = r.game.resources(0);
    room_step(&mut r, 2, SessionAction::PauseRoom, 1000); room_step(&mut r, 1, SessionAction::ResumeRoom, 2000);
    for n in 0..16 {if r.game.pending.is_some() {break;} let id = r.pacing.window.as_ref().unwrap().id.clone(); let seat = (0..4).find(|s|r.game.legal_actions(*s).iter().any(|v|v.action.kind == "pass")).unwrap(); room_step(&mut r, seat, SessionAction::PassResponse {window_id: id}, 3000 + n);}
    let choice = r.game.pending.as_ref().unwrap().choice.id.clone(); room_step(&mut r, 3, SessionAction::PauseRoom, 4000); room_step(&mut r, 2, SessionAction::ResumeRoom, 5000);
    let before = serde_json::to_string(&r).unwrap(); let cmd = room_step(&mut r, 0, SessionAction::Game {action: Action {choice_id: Some(choice), selected: Some(vec!["time".into()]), ..Action::new("choose")}}, 6000);
    let replay = RoomEnvelope::from_persisted(&before).unwrap().transition(0, Some(cmd.clone()), 6000).unwrap(); assert_eq!(replay.state, serde_json::to_string(&r).unwrap());
    let duplicate = r.transition(0, Some(cmd), 6001).unwrap(); assert_eq!(duplicate.error_code.as_deref(), Some("version_conflict")); assert_eq!(duplicate.state, serde_json::to_string(&r).unwrap());
    assert_eq!(r.game.resources(0), resources); assert_eq!(r.game.board(&target).unwrap().1.time_markers, 2); assert_eq!(r.game.seed, u64::MAX - 37);
    let restored = RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap(); for s in 0..4 {assert_eq!(serde_json::to_value(r.view(s, 7000)).unwrap(), serde_json::to_value(restored.view(s, 7000)).unwrap());}
}
