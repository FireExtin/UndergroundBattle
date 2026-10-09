//! Explicit bounded Native room layouts through paid commands and persisted state.
use crate::jc029_tests::{apply, board, checkpoint, envelope, fund, game, pass_top, reject};
use crate::{catalog, deck, model::*, room::*, rules::*};
fn held(g: &mut Game, owner: usize) -> String {
    let c = g.make_card("XQ27", owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn hidden(g: &mut Game, def: &str, owner: usize, r: usize) -> String {
    let id = board(g, def, owner, r);
    g.board_mut(&id).unwrap().face_down = true;
    id
}
fn cast(g: &mut Game, actor: usize) {
    let id = held(g, actor);
    fund(g, actor, "JC069", 4);
    let a = g
        .legal_actions(actor)
        .into_iter()
        .find(|a| a.action.kind == "play" && a.action.card_id.as_deref() == Some(&id))
        .unwrap()
        .action;
    assert!(a.region.is_none() && a.target_id.is_none());
    apply(g, actor, a);
}
#[test]
fn xq27_complete_original_and_standard_untargeted_definition() {
    let c = catalog::card("XQ27");
    assert_eq!(
        (&*c.name, &*c.kind, &*c.color, c.cost, &*c.magic),
        ("定点清除行动", "spell", "灰", 4, "")
    );
    assert_eq!(c.loyalty, ["灰色", "灰色"]);
    assert_eq!(c.subtypes, ["阴谋"]);
    assert!(!c.unique && c.keywords.is_empty());
    assert_eq!(deck::copy_limit(c), Some(3));
    let a = &definition("XQ27").abilities[0];
    assert!(a.play_only && a.event.is_none() && a.targets.is_empty() && a.costs.is_empty());
    assert_eq!(a.timing, Timing::Standard);
    assert!(matches!(a.ops.as_slice(), [Op::XQ27DestroyAllHidden]));
}
#[test]
fn xq27_all_regions_four_actors_hidden_types_same_names_owner_graves() {
    for actor in 0..4 {
        let mut g = game(actor);
        let mut ids = vec![];
        for r in 0..g.regions.len() {
            for owner in 0..4 {
                let id = hidden(
                    &mut g,
                    if owner % 2 == 0 { "JZ31" } else { "JC050" },
                    owner,
                    r,
                );
                g.board_mut(&id).unwrap().controller = (owner + 1) % 4;
                ids.push(id);
            }
        }
        let public = board(&mut g, "JZ31", 0, 2);
        let before = g
            .regions
            .iter()
            .map(|r| (r.card.id.clone(), r.influence))
            .collect::<Vec<_>>();
        cast(&mut g, actor);
        assert!(ids.iter().all(|id| g.board(id).is_some()));
        checkpoint(&g);
        pass_top(&mut g);
        assert!(ids.iter().all(|id| g.board(id).is_none()));
        assert!(g.board(&public).is_some());
        assert!(g.pending.is_none() && g.effects.is_empty());
        assert_eq!(
            g.regions
                .iter()
                .map(|r| (r.card.id.clone(), r.influence))
                .collect::<Vec<_>>(),
            before
        );
        for owner in 0..4 {
            assert_eq!(
                g.players[owner]
                    .graveyard
                    .iter()
                    .filter(|c| c.definition == if owner % 2 == 0 { "JZ31" } else { "JC050" })
                    .count(),
                g.regions.len()
            );
        }
        checkpoint(&g);
    }
}
#[test]
fn xq27_empty_wave_and_non_damage_ignores_shield_and_barrier() {
    for with_victim in [false, true] {
        let mut g = game(0);
        let victim = if with_victim {
            let id = hidden(&mut g, "JC070", 1, 1);
            g.board_mut(&id).unwrap().shield = 3;
            Some(id)
        } else {
            None
        };
        cast(&mut g, 0);
        pass_top(&mut g);
        assert!(victim.as_ref().is_none_or(|id| g.board(id).is_none()));
        checkpoint(&g);
    }
}
#[test]
fn xq27_uses_resolution_faces_paid_reveal_response_escapes() {
    let mut g = game(0);
    let escape = hidden(&mut g, "JC003", 1, 2);
    let victim = hidden(&mut g, "JC002", 3, 3);
    cast(&mut g, 0);
    fund(&mut g, 1, "JC003", 3);
    let a = g
        .legal_actions(1)
        .into_iter()
        .find(|a| a.action.kind == "reveal" && a.action.card_id.as_deref() == Some(&escape))
        .unwrap()
        .action;
    apply(&mut g, 1, a);
    pass_top(&mut g);
    let revealed = g.regions[2]
        .cards
        .iter()
        .find(|c| c.owner == 1 && c.definition == "JC003")
        .unwrap()
        .id
        .clone();
    assert!(!g.board(&revealed).unwrap().1.face_down);
    pass_top(&mut g);
    assert!(g.board(&revealed).is_some() && g.board(&victim).is_none());
    checkpoint(&g);
}
#[test]
fn xq27_current_region_replacement_and_returned_hidden_escape() {
    let mut g = game(0);
    let escape = hidden(&mut g, "JC002", 1, 2);
    cast(&mut g, 0);
    g.return_hand(&escape);
    let old = g.regions[2].card.id.clone();
    g.regions[2].card = g.make_card("DQJC111", 0);
    let victim = hidden(&mut g, "JC002", 3, 2);
    assert_ne!(old, g.regions[2].card.id);
    checkpoint(&g);
    pass_top(&mut g);
    assert!(g.board(&victim).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.definition == "JC002"));
    checkpoint(&g);
}
#[test]
fn xq27_invalid_payment_predeclared_target_region_and_standard_response_atomic() {
    let mut g = game(0);
    let id = held(&mut g, 0);
    fund(&mut g, 0, "JC069", 3);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            ..Action::new("play")
        },
    );
    fund(&mut g, 0, "JC069", 1);
    for target in [false, true] {
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(id.clone()),
                region: if target { None } else { Some(2) },
                target_id: if target {
                    Some("region:2".into())
                } else {
                    None
                },
                ..Action::new("play")
            },
        );
    }
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            ..Action::new("play")
        },
    );
    let id = held(&mut g, 1);
    fund(&mut g, 1, "JC069", 4);
    reject(
        &mut g,
        1,
        Action {
            card_id: Some(id),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    checkpoint(&g);
}
#[test]
fn xq27_stack_restore_final_response_and_duplicate_receipts() {
    let mut g = game(0);
    hidden(&mut g, "JZ31", 1, 2);
    cast(&mut g, 0);
    loop {
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        let room = envelope(&g);
        let cmd = RoomCommand {
            command_id: format!("xq27-pass-{}", room.revision),
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
            let restored = RoomEnvelope::from_persisted(&e.state).unwrap();
            let duplicate = restored
                .transition(seat, Some(cmd.clone()), 600000)
                .unwrap();
            assert_eq!(duplicate.state, e.state);
            assert_eq!(
                serde_json::to_value(duplicate.view).unwrap(),
                serde_json::to_value(restored.view(seat, 600000)).unwrap()
            );
            if let Ok(dir) = std::env::var("XQ27_FRONTEND_DIR") {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(format!("{dir}/resolve.json"),serde_json::to_vec(&serde_json::json!({"state":state,"seat":seat,"command":cmd,"expected":e,"views":(0..4).map(|s|restored.view(s,0)).collect::<Vec<_>>()})).unwrap()).unwrap();
            }
            break;
        }
        apply(&mut g, seat, Action::new("pass"));
    }
}
#[test]
fn xq27_definition_transplants_rekeys_nested_ops_and_timing_rejected() {
    for dest in ["XQ27", "JC002", "JC050"] {
        for mutation in 0..5 {
            let mut defs = definitions().clone();
            let mut d = definition("XQ27").clone();
            match mutation {
                0 => {}
                1 => d.abilities[0].key = "other".into(),
                2 => d.abilities[0].ops = vec![Op::ForEachLivingPlayer(d.abilities[0].ops.clone())],
                3 => d.abilities[0].timing = Timing::Fast,
                _ => d.abilities[0].play_only = false,
            };
            defs.insert(dest.into(), d);
            assert_eq!(
                validate_definitions(&defs).is_ok(),
                dest == "XQ27" && mutation == 0
            );
        }
    }
}
#[test]
fn xq27_persisted_stack_mutants_rejected_by_game_and_room() {
    let mut g = game(0);
    cast(&mut g, 0);
    let original = envelope(&g);
    for mutation in 0..13 {
        let mut r = original.clone();
        let s = r.game.stack.last_mut().unwrap();
        let f = s.frame.as_mut().unwrap();
        match mutation {
            0 => f.cursor = 1,
            1 => f.steps.push(f.steps[0].clone()),
            2 => f.steps[0].context = 1,
            3 => f.ability_key = "other".into(),
            4 => f.chosen_region = Some(2),
            5 => s.deploy_region = Some(2),
            6 => s.reveal = true,
            7 => s.card.as_mut().unwrap().controller = 1,
            8 => f.source.card.controller = 1,
            9 => s.target = Some("region:2".into()),
            10 => f.steps[0].op = Op::ForEachLivingPlayer(vec![Op::XQ27DestroyAllHidden]),
            11 => s.frame = None,
            _ => s.card.as_mut().unwrap().face_down = true,
        };
        let state = serde_json::to_string(&r).unwrap();
        assert!(RoomEnvelope::from_persisted(&state)
            .unwrap_err()
            .contains("XQ27"));
        assert!(
            Game::from_persisted(&serde_json::to_string(&r.game).unwrap())
                .unwrap_err()
                .contains("XQ27")
        );
        if let Ok(dir) = std::env::var("XQ27_INVALID_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                format!("{dir}/invalid-{mutation}.json"),
                serde_json::to_vec(
                    &serde_json::json!({"state":state,"expectedErrorContains":"XQ27"}),
                )
                .unwrap(),
            )
            .unwrap();
        }
    }
    checkpoint(&g);
}
#[test]
fn xq27_true_paid_hidden_reveal_does_not_cast_standard_sweep() {
    let mut g = game(0);
    let id = hidden(&mut g, "XQ27", 0, 2);
    let survivor = hidden(&mut g, "JC002", 1, 3);
    fund(&mut g, 0, "JC069", 4);
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| a.action.kind == "reveal" && a.action.card_id.as_deref() == Some(&id))
        .unwrap()
        .action;
    apply(&mut g, 0, a);
    checkpoint(&g);
    pass_top(&mut g);
    assert!(g.board(&survivor).is_some());
    checkpoint(&g);
}
