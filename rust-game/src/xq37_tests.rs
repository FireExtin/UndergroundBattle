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
            if friendly > 0 {
                accept(&mut g);
                pass_top(&mut g);
            } else {
                assert!(g.pending.is_none() && g.stack.is_empty());
            }
            // Existing influence placement cancels enemy influence before adding friendly markers.
            assert_eq!(g.regions[2].influence[team], friendly);
            assert_eq!(g.regions[2].influence[1 - team], u32::from(friendly == 0));
            checkpoint(&g);
        }
    }
}
#[test]
fn xq37_qualified_entry_rechecks_current_count_and_decline_is_noop() {
    for after in [0, 1, 2] {
        let mut g = initial(0);
        g.regions[2].influence = [1, 0];
        entry(&mut g, 0, 0);
        accept(&mut g);
        // Prepared response perturbation after a genuinely qualified entry.
        g.regions[2].influence = [after, 0];
        checkpoint(&g);
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [after + u32::from(after > 0), 0]);
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
fn xq37_entry_gate_is_at_event_not_later_declaration_or_response() {
    for qualified in [false, true] {
        let mut g = initial(0);
        g.regions[2].influence = [u32::from(qualified), 0];
        let id = board(&mut g, "XQ37", 0, 2);
        g.enter_triggers(0, "XQ37", &id, false);
        // A queued reward changes markers before Declare is processed.
        g.regions[2].influence = [u32::from(!qualified), 0];
        g.drive().unwrap();
        checkpoint(&g);
        if qualified {
            assert!(g.pending.is_some());
            accept(&mut g);
            pass_top(&mut g);
            assert_eq!(g.regions[2].influence, [0, 0]);
        } else {
            assert!(g.pending.is_none() && g.stack.is_empty());
            assert_eq!(g.regions[2].influence, [1, 0]);
        }
    }
    let mut g = initial(0);
    entry(&mut g, 0, 0);
    assert!(g.pending.is_none() && g.stack.is_empty());
    g.place_influence(0, 2, 1);
    g.drive().unwrap();
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.regions[2].influence, [1, 0]);
    fixture("zero-entry-no-trigger", &g);
    checkpoint(&g);
}
#[test]
fn xq37_frozen_actor_owner_follow_current_source_region_and_not_departed_source() {
    for change in 0..4 {
        let mut g = initial(2);
        g.regions[2].influence = [0, 1];
        g.regions[1].influence = [0, 1];
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
        assert_eq!(g.regions[2].influence, [0, if change == 3 { 2 } else { 1 }]);
        assert_eq!(g.regions[1].influence, [0, if change == 1 { 2 } else { 1 }]);
        checkpoint(&g);
    }
}
#[test]
fn xq37_move_to_unmarked_region_and_other_same_name_never_substitute_source() {
    for marked in [false, true] {
        let mut g = initial(0);
        g.regions[2].influence = [1, 0];
        g.regions[1].influence = [u32::from(marked), 0];
        let id = entry(&mut g, 0, 0);
        accept(&mut g);
        let impostor = board(&mut g, "XQ37", 0, 2);
        assert_ne!(id, impostor);
        let (_, c) = g.remove_board(&id).unwrap();
        g.regions[1].cards.push(c);
        checkpoint(&g);
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [1, 0]);
        assert_eq!(g.regions[1].influence, [if marked { 2 } else { 0 }, 0]);
        fixture(
            if marked {
                "moved-marked-region"
            } else {
                "moved-zero-region"
            },
            &g,
        );
    }
}
#[test]
fn xq37_original_region_departure_never_rewards_same_index_replacement() {
    for after_accept in [false, true] {
        let mut g = initial(0);
        g.regions[2].influence = [1, 0];
        let id = entry(&mut g, 0, 0);
        if after_accept {
            accept(&mut g);
        }
        // Explicit prepared region departure: its original source leaves first.
        // Replacing only the region card while retaining its inhabitants would
        // not model a real region departure under the current-source ruling.
        g.return_hand(&id);
        let old = g.regions[2].card.id.clone();
        g.regions[2].card = g.make_card("DQJC107", 0);
        assert_ne!(old, g.regions[2].card.id);
        assert!(g.board(&id).is_none());
        checkpoint(&g);
        if !after_accept {
            accept(&mut g);
        }
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [1, 0]);
        checkpoint(&g);
    }
}
fn responsive(g: &mut Game, seat: usize, action: Action) {
    for _ in 0..8 {
        if g.legal_actions(seat)
            .iter()
            .any(|a| a.action.kind == action.kind && a.action.card_id == action.card_id)
        {
            apply(g, seat, action);
            return;
        }
        let other = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        apply(g, other, Action::new("pass"));
    }
    panic!("response action never became legal");
}
#[test]
fn xq37_real_hide_and_paid_reveal_do_not_migrate_old_trigger_to_new_instance() {
    for reveal_again in [false, true] {
        let mut g = initial(0);
        g.regions[2].influence = [1, 0];
        let old = entry(&mut g, 0, 0);
        accept(&mut g);
        let spell = g.make_card("JC063", 0);
        let spell_id = spell.id.clone();
        g.players[0].hand.push(spell);
        fund(&mut g, 0, "JC056", 5);
        responsive(
            &mut g,
            0,
            Action {
                card_id: Some(spell_id),
                target_id: Some(old.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        );
        pass_top(&mut g);
        assert!(g.board(&old).is_none());
        let hidden = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "XQ37")
            .unwrap();
        assert!(hidden.face_down);
        let hidden_id = hidden.id.clone();
        assert_ne!(hidden_id, old);
        checkpoint(&g);
        if reveal_again {
            fund(&mut g, 0, "XQ37", 2);
            responsive(
                &mut g,
                0,
                Action {
                    card_id: Some(hidden_id.clone()),
                    ..Action::new("reveal")
                },
            );
            pass_top(&mut g);
            assert!(g.pending.is_some());
            let current = g.regions[2]
                .cards
                .iter()
                .find(|c| c.definition == "XQ37")
                .unwrap();
            assert!(!current.face_down && current.id != old && current.id != hidden_id);
            accept(&mut g);
            pass_top(&mut g);
            assert_eq!(g.regions[2].influence, [2, 0]);
        }
        pass_top(&mut g);
        assert_eq!(
            g.regions[2].influence,
            [if reveal_again { 2 } else { 1 }, 0]
        );
        assert!(g.pending.is_none() && g.stack.is_empty());
        checkpoint(&g);
    }
}
#[test]
fn xq37_moved_source_survives_original_region_replacement_before_or_after_accept() {
    for accepted in [false, true] {
        let mut g = initial(0);
        g.regions[2].influence = [1, 0];
        g.regions[1].influence = [1, 0];
        let id = entry(&mut g, 0, 0);
        if accepted {
            accept(&mut g);
        }
        let (_, c) = g.remove_board(&id).unwrap();
        g.regions[1].cards.push(c);
        let original_region = g.regions[2].card.id.clone();
        g.regions[2].card = g.make_card("DQJC107", 0);
        assert_ne!(g.regions[2].card.id, original_region);
        checkpoint(&g);
        if !accepted {
            accept(&mut g);
        }
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [1, 0]);
        assert_eq!(g.regions[1].influence, [2, 0]);
        checkpoint(&g);
    }
}
#[test]
fn xq37_opposing_same_colour_source_instances_do_not_replace_departed_source() {
    let mut g = initial(0);
    g.regions[2].influence = [1, 0];
    g.regions[1].influence = [0, 1];
    let first = entry(&mut g, 0, 0);
    accept(&mut g);
    let second = board(&mut g, "XQ37", 2, 1);
    assert_ne!(first, second);
    g.enter_triggers(2, "XQ37", &second, false);
    g.drive().unwrap();
    accept(&mut g);
    g.return_hand(&first);
    checkpoint(&g);
    pass_top(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [1, 0]);
    assert_eq!(g.regions[1].influence, [0, 2]);
    checkpoint(&g);
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
#[test]
fn xq37_pending_trigger_actor_and_optional_accept_metadata_are_closed() {
    for ability in 0..3 {
        let mut g = initial(0);
        g.regions[2].influence = [1, 0];
        let id = entry(&mut g, 0, 0);
        let preview = g.card_view(g.board(&id).unwrap().1, 0, Some(2), None);
        if let ChoiceResolution::Declare { declaration, .. } =
            &mut g.pending.as_mut().unwrap().resolution
        {
            let region = declaration
                .source
                .source_region_instance
                .as_deref()
                .unwrap();
            declaration.ability = match ability {
                0 => definition("XQ37").abilities[0].clone(),
                1 => crate::renown::renown_ability(region),
                _ => jc089_combat_glory_ability(region),
            };
        } else {
            panic!("normal entry must offer declaration");
        }
        // Both the printed trigger and its two existing exact granted rewards remain admitted.
        checkpoint(&g);
        let original = envelope(&g);
        for mutation in 0..15 {
            let mut r = original.clone();
            let p = r.game.pending.as_mut().unwrap();
            match mutation {
                0 => p.seat = 2,
                1 => p.choice.player_id = player_id(2),
                2 => p.choice.id.clear(),
                3 => p.choice.kind = "damage".into(),
                4 => p.choice.min = Some(1),
                5 => p.choice.max = Some(2),
                6 => p.choice.amount = Some(1),
                7 => p.choice.allow_decline = Some(false),
                8 => p.choice.options[0].id = "forged".into(),
                9 => p.choice.options[0].card = Some(preview.clone()),
                10 => p.choice.options.clear(),
                11 => p.choice.options.push(p.choice.options[0].clone()),
                12 => p.choice.preview_cards.push(preview.clone()),
                _ => {
                    if let ChoiceResolution::Declare { stage, .. } = &mut p.resolution {
                        *stage = if mutation == 13 {
                            DeclareChoice::Target
                        } else {
                            DeclareChoice::Mode
                        };
                    }
                }
            }
            invalid(19 + ability * 15 + mutation, &r);
        }
    }
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
