//! Explicit offline layouts with ordinary paid commands and persisted views.
//! Primitive movement/control inputs below are not claims of natural UI play.
use crate::jc029_tests::{apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject};
use crate::{catalog, deck, model::*, room::*, rules::*};
use std::collections::BTreeMap;

fn held(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn stats(g: &Game, id: &str) -> (u32, u32) {
    let (r, c) = g.board(id).unwrap();
    (g.current_icons(c, r).influence, g.defense(c, r))
}
fn pair(actor: usize) -> (Game, String, String) {
    let mut g = game(actor);
    let a = board(&mut g, "JZ48", actor, 2);
    let b = board(&mut g, "JZ48", actor, 2);
    (g, a, b)
}
fn fixture(kind: &str, g: &Game) {
    if let Ok(dir) = std::env::var("JZ48_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{kind}.json"), serde_json::to_vec(&serde_json::json!({
            "kind":kind,"state":serde_json::to_string(&r).unwrap(),
            "views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
    checkpoint(g);
}
fn cast(g: &mut Game, actor: usize, def: &str, target: &str) {
    let spell = held(g, def, actor);
    let action = g.legal_actions(actor).into_iter().find(|a|
        a.action.kind == "play" && a.action.card_id.as_deref() == Some(&spell)
        && a.action.target_id.as_deref() == Some(target)).unwrap().action;
    apply(g, actor, action);
    pass_top(g);
}

#[test]
fn jz48_original_is_exact_black_one_no_domain_and_one_closed_modifier() {
    let c = catalog::card("JZ48");
    assert_eq!((&*c.name, &*c.kind, &*c.color, c.cost, c.defense),
        ("街头劫匪", "character", "黑", 1, Some(1)));
    assert_eq!(c.loyalty, ["黑色"]);
    assert_eq!(c.subtypes, ["人类", "罪犯"]);
    assert_eq!(c.magic_icon, MagicIcon::None);
    assert!(c.magic.is_empty() && c.keywords.is_empty() && !c.unique);
    assert_eq!(c.permanent_icons, Icons { combat: 1, ..Icons::default() });
    assert_eq!(c.temporary_icons, Icons::default());
    assert_eq!(deck::copy_limit(c), Some(3));
    let d = definition("JZ48");
    assert!(d.abilities.is_empty() && d.attachment.is_none());
    assert!(matches!(d.modifiers.as_slice(), [StaticModifier::JZ48OtherControlledCriminalInfluenceAndDefense]));
    assert_eq!(catalog::catalog().cards.len(), 102);
    assert!(definition("JZ49").traits.slow);
}

#[test]
fn jz48_cannot_self_satisfy_and_teammates_enemies_hidden_assets_and_other_regions_do_not_count() {
    for actor in 0..4 {
        for other in 0..4 {
            for region in [1, 2, 3] {
                let mut g = game(actor);
                let source = board(&mut g, "JZ48", actor, 2);
                assert_eq!(stats(&g, &source), (0, 1));
                let criminal = board(&mut g, "JC084", other, region);
                let qualifies = actor == other && region == 2;
                assert_eq!(stats(&g, &source), if qualifies { (1, 2) } else { (0, 1) });
                g.board_mut(&criminal).unwrap().exhausted = true;
                assert_eq!(stats(&g, &source), if qualifies { (1, 2) } else { (0, 1) });
                checkpoint(&g);
                g.board_mut(&criminal).unwrap().face_down = true;
                assert_eq!(stats(&g, &source), (0, 1));
                checkpoint(&g);
                g.remove_board(&criminal).unwrap();
                fund(&mut g, actor, "JC084", 2);
                board(&mut g, "JC125", actor, 2);
                assert_eq!(stats(&g, &source), (0, 1));
                checkpoint(&g);
            }
        }
    }
}

#[test]
fn jz48_bonus_is_one_not_criminal_count_and_two_instances_support_each_other() {
    let (mut g, a, b) = pair(0);
    assert_ne!(a, b);
    assert_eq!(stats(&g, &a), (1, 2));
    assert_eq!(stats(&g, &b), (1, 2));
    for _ in 0..4 { board(&mut g, "JC084", 0, 2); }
    assert_eq!(stats(&g, &a), (1, 2));
    assert_eq!(stats(&g, &b), (1, 2));
    fixture("supported-pair", &g);
}

#[test]
fn jz48_permanent_bonus_ignores_initiative_but_exhaustion_suppresses_participation_only() {
    let (mut g, a, b) = pair(0);
    for first in 0..2 {
        g.first_team = first;
        assert_eq!(stats(&g, &a), (1, 2));
        g.board_mut(&a).unwrap().exhausted = true;
        let c = g.board(&a).unwrap().1;
        assert_eq!(g.icons(c, 2), Icons::default());
        assert_eq!(stats(&g, &a), (1, 2));
        assert_eq!(stats(&g, &b), (1, 2));
        fixture("exhausted-supported", &g);
        g.board_mut(&a).unwrap().exhausted = false;
    }
}

#[test]
fn jz48_hidden_source_and_departed_source_never_gain_the_bonus() {
    let (mut g, a, b) = pair(0);
    g.board_mut(&a).unwrap().face_down = true;
    assert!(!g.jz48_other_controlled_criminal_active(g.board(&a).unwrap().1, 2));
    assert_eq!(stats(&g, &b), (0, 1));
    let c = g.board(&a).unwrap().1;
    assert_eq!(g.current_icons(c, 2), Icons { influence: 1, ..Icons::default() });
    assert_eq!(g.defense(c, 2), 1);
    fixture("hidden-source", &g);
    let c = g.remove_board(&a).unwrap().1;
    assert!(!g.jz48_other_controlled_criminal_active(&c, 2));
}

#[test]
fn jz48_owner_is_not_controller_and_both_teams_have_independent_same_name_instances() {
    let mut g = game(0);
    let a = board(&mut g, "JZ48", 1, 2);
    g.add_control(&a, 0, ControlLifetime::TurnEnd { turn: g.turn }, SubtypeChange::None);
    let b = board(&mut g, "JZ48", 2, 2);
    let own = board(&mut g, "JC084", 3, 2);
    g.add_control(&own, 0, ControlLifetime::TurnEnd { turn: g.turn }, SubtypeChange::None);
    assert_eq!(stats(&g, &a), (1, 2));
    assert_eq!(stats(&g, &b), (0, 1));
    board(&mut g, "JC084", 2, 2);
    assert_eq!(stats(&g, &a), (1, 2));
    assert_eq!(stats(&g, &b), (1, 2));
    fixture("borrowed-both-sides", &g);
    g.add_control(&own, 1, ControlLifetime::TurnEnd { turn: g.turn }, SubtypeChange::None);
    assert_eq!(stats(&g, &a), (0, 1));
    assert_eq!(stats(&g, &b), (1, 2));
    checkpoint(&g);
}

#[test]
fn jz48_movement_hide_return_bottom_and_control_loss_update_before_lethal_stable_point() {
    for input in 0..5 {
        let mut g = game(0);
        let source = board(&mut g, "JZ48", 0, 2);
        let criminal = board(&mut g, "JC084", 0, 2);
        g.board_mut(&source).unwrap().damage = 1;
        g.board_mut(&source).unwrap().exhausted = true;
        assert_eq!(stats(&g, &source), (1, 2));
        // Exact existing primitives, explicitly distinct from natural card commands.
        match input {
            0 => { let c = g.remove_board(&criminal).unwrap().1; g.regions[3].cards.push(c); }
            1 => g.board_mut(&criminal).unwrap().face_down = true,
            2 => g.return_hand(&criminal),
            3 => g.to_bottom(&criminal),
            _ => g.add_control(&criminal, 1, ControlLifetime::TurnEnd { turn: g.turn }, SubtypeChange::None),
        }
        assert_eq!(stats(&g, &source), (0, 1));
        g.settle_deaths();
        g.drive().unwrap();
        assert!(g.board(&source).is_none());
        assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "JZ48").count(), 1);
        checkpoint(&g);
    }
}

#[test]
fn jz48_moving_or_changing_source_controller_uses_its_current_region_and_controller() {
    let (mut g, a, _) = pair(0);
    let c = g.remove_board(&a).unwrap().1;
    g.regions[3].cards.push(c);
    assert_eq!(stats(&g, &a), (0, 1));
    let other = board(&mut g, "JC084", 2, 3);
    assert_eq!(stats(&g, &a), (0, 1));
    g.add_control(&a, 2, ControlLifetime::TurnEnd { turn: g.turn }, SubtypeChange::None);
    assert_eq!(stats(&g, &a), (1, 2));
    g.board_mut(&a).unwrap().damage = 1;
    g.add_control(&other, 3, ControlLifetime::TurnEnd { turn: g.turn }, SubtypeChange::None);
    g.settle_deaths();
    assert!(g.board(&a).is_none());
    assert!(g.players[0].graveyard.iter().any(|c| c.definition == "JZ48" && c.controller == 0));
    checkpoint(&g);
}

#[test]
fn jz48_control_expiry_and_source_leave_cleanup_can_cause_death() {
    for source_lifetime in [false, true] {
        let mut g = game(0);
        let a = board(&mut g, "JZ48", 0, 2);
        let criminal = board(&mut g, "JC084", 2, 2);
        let controller_source = board(&mut g, "JC125", 0, 2);
        let lifetime = if source_lifetime { ControlLifetime::SourceLeaves { source_instance: controller_source.clone() } }
            else { ControlLifetime::TurnEnd { turn: g.turn } };
        g.add_control(&criminal, 0, lifetime, SubtypeChange::None);
        g.board_mut(&a).unwrap().damage = 1;
        assert_eq!(stats(&g, &a), (1, 2));
        if source_lifetime { g.remove_dead(&controller_source, RemovalCause::Destroy); }
        else { g.turn += 1; }
        g.settle_deaths();
        g.drive().unwrap();
        assert!(g.board(&a).is_none());
        assert_eq!(g.board(&criminal).unwrap().1.controller, 2);
        checkpoint(&g);
    }
}

#[test]
fn jz48_real_xq16_hide_resets_hidden_damage_and_removes_support_for_the_partner() {
    let (mut g, a, b) = pair(0);
    for id in [&a, &b] { g.board_mut(id).unwrap().damage = 1; }
    fund(&mut g, 0, "XQ16", 2);
    let lawyer = held(&mut g, "XQ16", 0);
    apply(&mut g, 0, Action { card_id: Some(lawyer), region: Some(2), ..Action::new("deploy") });
    pass_top(&mut g);
    fixture("hide-declaration", &g);
    choose(&mut g, vec![a.clone()]);
    fixture("hide-stack", &g);
    pass_top(&mut g);
    assert!(g.board(&a).is_none() && g.board(&b).is_none());
    let hidden = g.regions[2].cards.iter().find(|c| c.definition == "JZ48").unwrap();
    assert!(hidden.face_down && hidden.damage == 0);
    assert_ne!(hidden.id, a);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "JZ48").count(), 1);
    fixture("hide-cascade-final", &g);
}

#[test]
fn jz48_atomic_cleanup_removes_damage_before_control_expiry_but_wounds_can_kill() {
    for wound in [false, true] {
        let mut g = game(0);
        let a = board(&mut g, "JZ48", 0, 2);
        let criminal = board(&mut g, "JC084", 2, 2);
        g.add_control(&criminal, 0, ControlLifetime::TurnEnd { turn: g.turn }, SubtypeChange::None);
        if wound { g.board_mut(&a).unwrap().wounds = 1; }
        else { g.board_mut(&a).unwrap().damage = 1; }
        assert_eq!(stats(&g, &a).1, if wound { 1 } else { 2 });
        // Existing atomic 4.2.3 cleanup, without adding a new expiry pathway.
        g.effects.push_back(Effect::FinishCleanup);
        g.drive().unwrap();
        assert_eq!(g.board(&a).is_none(), wound);
        if !wound { assert_eq!(g.board(&a).unwrap().1.damage, 0); }
        assert_eq!(g.board(&criminal).unwrap().1.controller, 2);
        checkpoint(&g);
    }
}

#[test]
fn jz48_paid_damage_causes_two_mutually_supported_instances_to_die_in_cascade() {
    for actor in 0..4 {
        let (mut g, a, b) = pair(actor);
        for id in [&a, &b] { g.board_mut(id).unwrap().damage = 1; }
        fund(&mut g, actor, "JC102", 2);
        cast(&mut g, actor, "JC102", &a);
        assert!(g.board(&a).is_none() && g.board(&b).is_none());
        let deaths = g.log.iter().filter(|l| l.text == "街头劫匪：死亡").count();
        assert_eq!(deaths, 2);
        assert_eq!(g.players[actor].graveyard.iter().filter(|c| c.definition == "JZ48").count(), 2);
        fixture("paid-cascade", &g);
    }
}

#[test]
fn jz48_simultaneous_lethal_set_is_captured_before_departure_and_each_owner_gets_one() {
    let mut g = game(0);
    let a = board(&mut g, "JZ48", 1, 2);
    let b = board(&mut g, "JZ48", 2, 2);
    g.add_control(&a, 0, ControlLifetime::TurnEnd { turn: g.turn }, SubtypeChange::None);
    g.add_control(&b, 0, ControlLifetime::SourceLeaves { source_instance: a.clone() }, SubtypeChange::None);
    assert_eq!(stats(&g, &a), (1, 2));
    assert_eq!(stats(&g, &b), (1, 2));
    g.damage(BTreeMap::from([(a.clone(), 2), (b.clone(), 2)])).unwrap();
    g.drive().unwrap();
    assert!(g.board(&a).is_none() && g.board(&b).is_none());
    for owner in [1, 2] {
        assert_eq!(g.players[owner].graveyard.iter().filter(|c| c.definition == "JZ48").count(), 1);
    }
    checkpoint(&g);
}

#[test]
fn jz48_confrontation_sums_team_icons_but_condition_excludes_teammates_and_scores_exact_threshold() {
    let (mut g, a, b) = pair(0);
    board(&mut g, "JZ48", 1, 2); // same team, unsupported: zero influence.
    assert_eq!(g.contest_counts(2, 2), [2, 0]);
    g.board_mut(&b).unwrap().exhausted = true;
    assert_eq!(g.contest_counts(2, 2), [1, 0]);
    let threshold = catalog::card(&g.regions[2].card.definition).threshold.unwrap();
    g.regions[2].influence = [threshold - 1, 0];
    g.begin_window(Window::Before(2, 2));
    g.close_window().unwrap();
    assert_eq!(g.regions[2].influence[0], threshold);
    g.drive().unwrap();
    assert!(matches!(g.window, Some(Window::Win(2, _))));
    assert_eq!(g.icons(g.board(&a).unwrap().1, 2).influence, 1);
    fixture("threshold-win", &g);
}

#[test]
fn jz48_one_available_influence_wins_region_and_exact_team_score_ten() {
    let (mut g, _, b) = pair(0);
    g.board_mut(&b).unwrap().exhausted = true;
    g.regions[2].card = g.make_card("DQJC109", 0);
    g.regions[2].influence = [2, 0];
    for id in ["DQJC107", "DQJC109"] {
        let c = g.make_card(id, 0); g.players[0].score_cards.push(c);
    }
    assert_eq!(g.contest_counts(2, 2), [1, 0]);
    assert_eq!(g.players[0].score_cards.iter().map(|c| catalog::card(&c.definition).points.unwrap_or(0)).sum::<u32>(), 7);
    g.begin_window(Window::Before(2, 2));
    g.close_window().unwrap(); g.drive().unwrap();
    assert_eq!(g.regions[2].influence, [3, 0]);
    assert!(matches!(g.window, Some(Window::Win(2, 0))));
    g.close_window().unwrap(); g.drive().unwrap();
    while let Some(p) = g.pending.clone() {
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
        choose(&mut g, ids);
    }
    assert_eq!(g.players[0].score_cards.iter().map(|c| catalog::card(&c.definition).points.unwrap_or(0)).sum::<u32>(), 10);
    assert_eq!(g.status, "finished"); assert_eq!(g.winner_team, Some(0));
    fixture("score-ten-final", &g);
}

#[test]
fn jz48_paid_deployment_no_optional_trigger_atomic_loyalty_and_cost_rejections() {
    for actor in 0..4 {
        let mut g = game(actor);
        let id = held(&mut g, "JZ48", actor);
        fund(&mut g, actor, "JC125", 1);
        reject(&mut g, actor, Action { card_id: Some(id.clone()), region: Some(2), ..Action::new("deploy") });
        g.players[actor].assets.clear();
        fund(&mut g, actor, "JC084", 1);
        g.players[actor].assets[0].exhausted = true;
        reject(&mut g, actor, Action { card_id: Some(id.clone()), region: Some(2), ..Action::new("deploy") });
        g.players[actor].assets[0].exhausted = false;
        board(&mut g, "JC084", actor, 2);
        apply(&mut g, actor, Action { card_id: Some(id.clone()), region: Some(2), ..Action::new("deploy") });
        assert!(g.board(&id).is_none());
        fixture("deployment-stack", &g);
        pass_top(&mut g);
        assert!(g.pending.is_none() && g.stack.is_empty());
        let live = g.regions[2].cards.iter().find(|c| c.definition == "JZ48").unwrap();
        assert_ne!(live.id, id);
        assert_eq!(stats(&g, &live.id), (1, 2));
        assert_eq!(g.resources(actor), 0);
        assert!(!live.exhausted);
        fixture("paid-deployment", &g);
    }
}

#[test]
fn jz48_complete_definition_rejects_mutations_and_transplants() {
    for variant in 0..9 {
        let mut registry = definitions().clone();
        let d = registry.get_mut("JZ48").unwrap();
        match variant {
            0 => d.modifiers.clear(),
            1 => d.modifiers.push(d.modifiers[0].clone()),
            2 => d.modifiers.push(StaticModifier::OtherFriendlyCharactersDefense(1)),
            3 => d.abilities = definition("JC003").abilities.clone(),
            4 => d.traits.public = true,
            5 => d.traits.slow = true,
            6 => d.graveyard_face_up = true,
            7 => d.attachment = definition("XQ47").attachment.clone(),
            _ => d.traits.guard = 1,
        }
        assert!(validate_definitions(&registry).is_err(), "mutation {variant}");
    }
    for id in ["JC125", "JC084", "JZ49"] {
        let mut registry = definitions().clone();
        registry.insert(id.into(), definition("JZ48").clone());
        assert!(validate_definitions(&registry).is_err());
        let mut registry = definitions().clone();
        registry.get_mut(id).unwrap().modifiers.push(StaticModifier::JZ48OtherControlledCriminalInfluenceAndDefense);
        assert!(validate_definitions(&registry).is_err());
    }
    validate_definitions(definitions()).unwrap();
}

fn command(room: &mut RoomEnvelope, steps: &mut Vec<serde_json::Value>, seat: usize, action: SessionAction) {
    let state = serde_json::to_string(room).unwrap();
    let cmd = RoomCommand { command_id: format!("jz48-chain-{}", steps.len()), expected_version: room.revision, action };
    let expected = room.transition(seat, Some(cmd.clone()), 0).unwrap();
    assert!(expected.error_code.is_none(), "{:?}", expected.error_message);
    assert_eq!(serde_json::to_string(&room.replay_events(&expected.journal).unwrap()).unwrap(), expected.state);
    *room = RoomEnvelope::from_persisted(&expected.state).unwrap();
    steps.push(serde_json::json!({"state":state,"seat":seat,"command":cmd,"expected":expected,
        "views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>() }));
}

#[test]
fn jz48_continuous_paid_response_cascade_and_underlying_target_revalidation_survive_restore() {
    let (mut g, a, b) = pair(0);
    for id in [&a, &b] { g.board_mut(id).unwrap().damage = 1; }
    fund(&mut g, 0, "JC102", 2);
    fund(&mut g, 2, "JC102", 2);
    let first = held(&mut g, "JC102", 0);
    let second = held(&mut g, "JC102", 2);
    let mut room = envelope(&g);
    let initial = serde_json::to_string(&room).unwrap();
    let mut steps = vec![];
    command(&mut room, &mut steps, 0, SessionAction::Game { action: Action {
        card_id: Some(first), target_id: Some(b.clone()), ..Action::new("play") } });
    while room.pacing.window.as_ref().unwrap().holder_team == 0 {
        let w = room.pacing.window.clone().unwrap();
        let seat = *w.members.iter().find(|(_, d)| matches!(d, Decision::Undecided { .. })).unwrap().0;
        command(&mut room, &mut steps, seat, SessionAction::PassResponse { window_id: w.id });
    }
    let w = room.pacing.window.clone().unwrap();
    command(&mut room, &mut steps, 2, SessionAction::BeginResponse { window_id: w.id.clone(), intent_id: "jz48-paid-cascade".into() });
    command(&mut room, &mut steps, 2, SessionAction::SubmitResponse {
        window_id: w.id, intent_id: "jz48-paid-cascade".into(), action: Action {
            card_id: Some(second), target_id: Some(a.clone()), ..Action::new("play") } });
    fixture("response-stack", &room.game);
    for _ in 0..30 {
        if room.stack.is_empty() { break; }
        let w = room.pacing.window.clone().unwrap();
        let seat = *w.members.iter().find(|(_, d)| matches!(d, Decision::Undecided { .. })).unwrap().0;
        command(&mut room, &mut steps, seat, SessionAction::PassResponse { window_id: w.id });
    }
    assert!(room.stack.is_empty() && room.pending.is_none());
    assert!(room.board(&a).is_none() && room.board(&b).is_none());
    assert_eq!(room.players[0].graveyard.iter().filter(|c| c.definition == "JZ48").count(), 2);
    assert_eq!(room.resources(0), 0);
    assert_eq!(room.resources(2), 0);
    fixture("response-cascade-final", &room.game);
    if let Ok(dir) = std::env::var("JZ48_CHAIN_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/paid-response-chain.json"), serde_json::to_vec(&serde_json::json!({
            "initialState":initial,"steps":steps,"finalState":serde_json::to_string(&room).unwrap()
        })).unwrap()).unwrap();
    }
}

#[cfg(feature = "native")]
#[tokio::test]
async fn jz48_sqlite_paid_stack_cascade_reopen_duplicate_and_conflict_receipts() {
    use crate::service::{CreateRoom, JoinRoom, Store};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("jz48-local-fixture.sqlite");
    let store = Store::open(&path).unwrap();
    let host = store.create(CreateRoom { name: "P0".into(), mode: "teams".into(), deck_id: "watchers".into(), deck_draft: None }).await.unwrap();
    let mut sessions = vec![host];
    for seat in 1..4 { sessions.push(store.join(JoinRoom { invite_code: sessions[0].invite_code.clone(), name: format!("P{seat}"), deck_id: "watchers".into(), deck_draft: None }).await.unwrap()); }
    drop(store);
    let (mut g, a, b) = pair(0);
    for id in [&a, &b] { g.board_mut(id).unwrap().damage = 1; }
    fund(&mut g, 0, "JC102", 2);
    let spell = held(&mut g, "JC102", 0);
    g.room_id = sessions[0].room_id.clone(); g.invite_code = sessions[0].invite_code.clone();
    let mut room = envelope(&g);
    // Explicit synthetic private Store fixture, never a live database.
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute("UPDATE rooms SET state=?1, revision=?2 WHERE id=?3", rusqlite::params![serde_json::to_string(&room).unwrap(), room.revision, g.room_id]).unwrap(); drop(db);
    let first = RoomCommand { command_id: "jz48-store-paid".into(), expected_version: room.revision,
        action: SessionAction::Game { action: Action { card_id: Some(spell), target_id: Some(a.clone()), ..Action::new("play") } } };
    let mut cmds = vec![(0, first)]; let mut accepted = vec![];
    for _ in 0..20 {
        let (seat, cmd) = cmds.last().unwrap().clone();
        let expected = room.transition(seat, Some(cmd.clone()), 0).unwrap(); assert!(expected.error_code.is_none());
        let store = Store::open(&path).unwrap();
        let view = store.command_at_now(&g.room_id, &sessions[seat].token, cmd.clone(), 0).await.unwrap(); drop(store);
        room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        assert_eq!(serde_json::to_value(&view).unwrap(), serde_json::to_value(room.view(seat, 0)).unwrap());
        accepted.push(serde_json::to_value(view).unwrap());
        let store = Store::open(&path).unwrap();
        let duplicate = store.command_at_now(&g.room_id, &sessions[seat].token, cmd.clone(), 0).await.unwrap();
        assert_eq!(accepted.last().unwrap(), &serde_json::to_value(duplicate).unwrap());
        let mut conflict = cmd.clone(); conflict.expected_version += 1;
        assert_eq!(store.command_at_now(&g.room_id, &sessions[seat].token, conflict, 0).await.unwrap_err().error, "command_id_conflict"); drop(store);
        if room.stack.is_empty() { break; }
        let w = room.pacing.window.clone().unwrap();
        let next_seat = *w.members.iter().find(|(_, d)| matches!(d, Decision::Undecided { .. })).unwrap().0;
        cmds.push((next_seat, RoomCommand { command_id: format!("jz48-store-pass-{}", cmds.len()), expected_version: room.revision, action: SessionAction::PassResponse { window_id: w.id } }));
    }
    assert!(room.stack.is_empty() && room.board(&a).is_none() && room.board(&b).is_none());
    assert_eq!(room.players[0].graveyard.iter().filter(|c| c.definition == "JZ48").count(), 2);
    let store = Store::open(&path).unwrap();
    assert_eq!(serde_json::to_value(store.command_at_now(&g.room_id, &sessions[0].token, cmds[0].1.clone(), 90_000).await.unwrap()).unwrap(), accepted[0]); drop(store);
    let db = rusqlite::Connection::open(&path).unwrap();
    let stored: String = db.query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| r.get(0)).unwrap();
    assert_eq!(stored, serde_json::to_string(&room).unwrap());
    let n: u64 = db.query_row("SELECT COUNT(*) FROM commands WHERE room_id=?1", [&g.room_id], |r| r.get(0)).unwrap();
    assert_eq!(n, cmds.len() as u64);
}
