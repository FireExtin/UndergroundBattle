//! Explicit Native layouts, real paid actions, choices and persisted Room commands.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, deck, model::*, room::*, rules::*};
fn initial(actor: usize) -> Game {
    let mut g = game(actor);
    for r in &mut g.regions {
        r.influence = [0; 2];
    }
    g
}
fn held(g: &mut Game, owner: usize) -> String {
    let c = g.make_card("XQ37", owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn entry(g: &mut Game, owner: usize, actor: usize) -> String {
    let id = board(g, "XQ37", owner, 2);
    g.board_mut(&id).unwrap().controller = actor;
    g.enter_triggers(actor, "XQ37", &id, false);
    g.drive().unwrap();
    checkpoint(g);
    id
}
fn accept(g: &mut Game) {
    choose(g, vec!["accept".into()]);
}
fn fixture(kind: &str, g: &Game) {
    if let Ok(dir) = std::env::var("XQ37_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{kind}.json"),serde_json::to_vec(&serde_json::json!({"state":serde_json::to_string(&r).unwrap(),"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()})).unwrap()).unwrap();
    }
}
#[test]
fn xq37_complete_original_and_closed_entry_definition() {
    let c = catalog::card("XQ37");
    assert_eq!(
        (&*c.name, c.cost, &*c.color, &*c.magic, c.defense),
        ("夜总会看门人", 2, "黑", "", Some(1))
    );
    assert_eq!(c.loyalty, ["黑色"]);
    assert_eq!(c.subtypes, ["人类", "罪犯"]);
    assert!(c.keywords.is_empty() && !c.unique);
    assert_eq!(
        c.permanent_icons,
        Icons {
            combat: 1,
            ..Default::default()
        }
    );
    assert_eq!(c.temporary_icons, Icons::default());
    assert_eq!(deck::copy_limit(c), Some(3));
    let a = &definition("XQ37").abilities[0];
    assert_eq!(a.event, Some(Event::Enter));
    assert_eq!(a.timing, Timing::Fast);
    assert_eq!(a.response_policy, ResponsePolicy::Respondable);
    assert!(
        a.costs.is_empty()
            && a.targets.is_empty()
            && a.modes.is_empty()
            && !a.play_only
            && !a.activation_only
            && a.per_turn_limit.is_none()
    );
    assert!(matches!(
        a.ops.as_slice(),
        [Op::XQ37EntryInfluenceIfPresent]
    ));
}
#[test]
fn xq37_paid_deploy_and_reveal_emit_once_hidden_entry_and_move_do_not() {
    for hidden in [false, true] {
        let mut g = initial(0);
        g.regions[2].influence = [1, 0];
        fund(&mut g, 0, "XQ37", 2);
        let id = held(&mut g, 0);
        apply(
            &mut g,
            0,
            Action {
                card_id: Some(id),
                region: Some(2),
                ..Action::new(if hidden { "conceal" } else { "deploy" })
            },
        );
        if hidden {
            assert!(g.pending.is_none());
            assert_eq!(g.regions[2].influence, [1, 0]);
            let id = g.regions[2].cards[0].id.clone();
            for c in &mut g.players[0].assets {
                c.exhausted = false;
            }
            apply(
                &mut g,
                0,
                Action {
                    card_id: Some(id),
                    ..Action::new("reveal")
                },
            );
        }
        pass_top(&mut g);
        assert!(g.pending.is_some());
        fixture("paid-entry-choice", &g);
        accept(&mut g);
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [2, 0]);
        assert!(g.pending.is_none() && g.stack.is_empty());
        let id = g.regions[2].cards[0].id.clone();
        let (_, c) = g.remove_board(&id).unwrap();
        g.regions[1].cards.push(c);
        g.drive().unwrap();
        checkpoint(&g);
        assert!(g.pending.is_none());
    }
}
#[test]
fn xq37_payment_and_loyalty_fail_atomically_and_trigger_cannot_be_activated() {
    for assets in [0, 1] {
        let mut g = initial(0);
        fund(&mut g, 0, "XQ37", assets);
        let id = held(&mut g, 0);
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(id),
                region: Some(2),
                ..Action::new("deploy")
            },
        );
    }
    let mut g = initial(0);
    fund(&mut g, 0, "JC059", 2);
    let id = held(&mut g, 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    let id = board(&mut g, "XQ37", 0, 2);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            ability_id: Some("entry-existing-influence".into()),
            ..Action::new("activate")
        },
    );
}
#[test]
fn xq37_only_current_friendly_team_count_one_for_all_four_seats() {
    for actor in 0..4 {
        for friendly in [0, 1, 2] {
            let mut g = initial(actor);
            let team = g.team(actor);
            g.regions[2].influence[team] = friendly;
            g.regions[2].influence[1 - team] = 1;
            entry(&mut g, actor, actor);
            accept(&mut g);
            pass_top(&mut g);
            // Existing influence placement cancels enemy influence before adding friendly markers.
            assert_eq!(g.regions[2].influence[team], friendly);
            assert_eq!(g.regions[2].influence[1 - team], u32::from(friendly == 0));
            checkpoint(&g);
        }
    }
}
#[test]
fn xq37_condition_at_resolution_can_become_true_or_false_and_decline_is_noop() {
    for before in [false, true] {
        let mut g = initial(0);
        g.regions[2].influence = [u32::from(before), 0];
        entry(&mut g, 0, 0);
        accept(&mut g);
        // Explicit prepared response perturbation: condition is not frozen with declaration.
        g.regions[2].influence = [u32::from(!before), 0];
        checkpoint(&g);
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [if before { 0 } else { 2 }, 0]);
    }
    let mut g = initial(0);
    g.regions[2].influence = [1, 0];
    entry(&mut g, 0, 0);
    choose(&mut g, vec![]);
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.regions[2].influence, [1, 0]);
    checkpoint(&g);
}
#[test]
fn xq37_frozen_actor_owner_and_original_region_survive_source_departure_or_move() {
    for change in 0..4 {
        let mut g = initial(2);
        g.regions[2].influence = [0, 1];
        g.regions[1].influence = [1, 0];
        let id = entry(&mut g, 0, 2);
        accept(&mut g);
        match change {
            0 => {
                g.return_hand(&id);
                assert!(g.players[0].hand.iter().any(|c| c.definition == "XQ37"));
            }
            1 => {
                let (_, c) = g.remove_board(&id).unwrap();
                g.regions[1].cards.push(c);
            }
            2 => g.remove_dead(&id, RemovalCause::Destroy),
            _ => g.board_mut(&id).unwrap().controller = 0,
        }
        checkpoint(&g);
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [0, 2]);
        assert_eq!(g.regions[1].influence, [1, 0]);
        checkpoint(&g);
    }
}
#[test]
fn xq37_replaced_original_region_never_rewards_same_index_replacement() {
    for after_accept in [false, true] {
        let mut g = initial(0);
        g.regions[2].influence = [1, 0];
        entry(&mut g, 0, 0);
        if after_accept {
            accept(&mut g);
        }
        let old = g.regions[2].card.id.clone();
        g.regions[2].card = g.make_card("DQJC107", 0);
        assert_ne!(old, g.regions[2].card.id);
        checkpoint(&g);
        if !after_accept {
            accept(&mut g);
        }
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [1, 0]);
        checkpoint(&g);
    }
}
#[test]
fn xq37_same_name_independent_instances_one_point_and_win_threshold() {
    let mut g = initial(0);
    g.regions[2].influence = [1, 0];
    let first = entry(&mut g, 0, 0);
    accept(&mut g);
    pass_top(&mut g);
    let second = entry(&mut g, 1, 1);
    assert_ne!(first, second);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [3, 0]);
    let mut g = initial(0);
    g.regions[2].card = g.make_card("DQJC115", 0);
    g.regions[2].influence = [2, 0];
    entry(&mut g, 0, 0);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [3, 0]);
    assert_eq!(g.window, Some(Window::Win(2, 0)));
    fixture("win-threshold", &g);
}
#[test]
fn xq37_declaration_stack_roundtrip_wrong_actor_and_duplicate_choice() {
    let mut g = initial(0);
    g.regions[2].influence = [1, 0];
    entry(&mut g, 0, 0);
    let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    let a = Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(vec!["accept".into()]),
        ..Action::new("choose")
    };
    reject(&mut g, 2, a.clone());
    apply(&mut g, 0, a.clone());
    apply(&mut restored, 0, a.clone());
    reject(&mut g, 0, a);
    assert_eq!(
        serde_json::to_value(&g).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    fixture("accepted-stack", &g);
    let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    pass_top(&mut g);
    pass_top(&mut restored);
    assert_eq!(
        serde_json::to_value(&g).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    assert_eq!(g.regions[2].influence, [2, 0]);
}
#[test]
fn xq37_final_room_command_and_duplicate_receipt_export() {
    let mut g = initial(0);
    g.regions[2].influence = [1, 0];
    entry(&mut g, 0, 0);
    accept(&mut g);
    loop {
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        let room = envelope(&g);
        let cmd = RoomCommand {
            command_id: format!("xq37-pass-{}", room.revision),
            expected_version: room.revision,
            action: SessionAction::PassResponse {
                window_id: room.pacing.window.as_ref().unwrap().id.clone(),
            },
        };
        let state = serde_json::to_string(&room).unwrap();
        let e = room.transition(seat, Some(cmd.clone()), 0).unwrap();
        assert!(e.error_code.is_none());
        if serde_json::from_str::<RoomEnvelope>(&e.state)
            .unwrap()
            .game
            .stack
            .is_empty()
        {
            let r = RoomEnvelope::from_persisted(&e.state).unwrap();
            let duplicate = r.transition(seat, Some(cmd.clone()), 600000).unwrap();
            assert_eq!(duplicate.state, e.state);
            assert_eq!(
                serde_json::to_value(duplicate.view).unwrap(),
                serde_json::to_value(r.view(seat, 600000)).unwrap()
            );
            if let Ok(dir) = std::env::var("XQ37_FRONTEND_DIR") {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(format!("{dir}/resolve.json"),serde_json::to_vec(&serde_json::json!({"state":state,"seat":seat,"command":cmd,"expected":e,"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()})).unwrap()).unwrap();
            }
            break;
        }
        apply(&mut g, seat, Action::new("pass"));
    }
}
#[test]
fn xq37_definition_rekeys_transplants_nested_ops_and_extra_traits_rejected() {
    for dest in ["XQ37", "JC002", "JZ22"] {
        for mutation in 0..7 {
            let mut defs = definitions().clone();
            let mut d = definition("XQ37").clone();
            let a = &mut d.abilities[0];
            match mutation {
                0 => {}
                1 => a.key = "other".into(),
                2 => a.ops = vec![Op::ForEachLivingPlayer(a.ops.clone())],
                3 => a.event = Some(Event::Death),
                4 => a.ops.push(Op::XQ37EntryInfluenceIfPresent),
                5 => a.costs = vec![Cost::Assets(1)],
                _ => d.traits.public = true,
            };
            defs.insert(dest.into(), d);
            assert_eq!(
                validate_definitions(&defs).is_ok(),
                dest == "XQ37" && mutation == 0
            );
        }
    }
}
#[test]
fn xq37_invalid_entry_frame_and_root_hidden_flag_rejected_and_exported() {
    let mut g = initial(0);
    g.regions[2].influence = [1, 0];
    entry(&mut g, 0, 0);
    accept(&mut g);
    let original = envelope(&g);
    for mutation in 0..18 {
        let mut r = original.clone();
        let s = r.game.stack.last_mut().unwrap();
        let f = s.frame.as_mut().unwrap();
        match mutation {
            0 => f.cursor = 1,
            1 => f.steps.push(f.steps[0].clone()),
            2 => f.steps[0].context = 2,
            3 => f.ability_key = "other".into(),
            4 => f.chosen_region = Some(2),
            5 => f.source.card.face_down = true,
            6 => f.source.card.controller = 2,
            7 => f.source.region = None,
            8 => f.source.source_region_instance = None,
            9 => f.source.attachment_host_instance = Some("fake".into()),
            10 => f.already_paid = vec![PaidCost::Assets(vec![])],
            11 => f.guard = GuardState::Accepted,
            12 => s.controller = 2,
            13 => s.id = "fake".into(),
            14 => s.target = Some("region:2".into()),
            15 => f.steps[0].op = Op::ForEachLivingPlayer(vec![Op::XQ37EntryInfluenceIfPresent]),
            16 => s.card = Some(f.source.card.clone()),
            _ => s.reveal = true,
        };
        invalid(mutation, &r);
    }
    let mut g = initial(0);
    fund(&mut g, 0, "XQ37", 2);
    let id = held(&mut g, 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    let mut r = envelope(&g);
    r.game
        .stack
        .last_mut()
        .unwrap()
        .card
        .as_mut()
        .unwrap()
        .face_down = true;
    invalid(18, &r);
    checkpoint(&g);
}
fn invalid(n: usize, r: &RoomEnvelope) {
    let state = serde_json::to_string(r).unwrap();
    assert!(RoomEnvelope::from_persisted(&state)
        .unwrap_err()
        .contains("XQ37"));
    assert!(
        Game::from_persisted(&serde_json::to_string(&r.game).unwrap())
            .unwrap_err()
            .contains("XQ37")
    );
    if let Ok(dir) = std::env::var("XQ37_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/invalid-{n}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state,"expectedErrorContains":"XQ37"}))
                .unwrap(),
        )
        .unwrap();
    }
}
