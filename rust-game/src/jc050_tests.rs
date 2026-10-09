//! Prepared rule layouts, paid production actions and exact persisted room inputs.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, deck, model::*, room::*, rules::*};
fn held(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn cast(g: &mut Game, actor: usize) -> String {
    let id = held(g, "JC050", actor);
    fund(g, actor, "JC047", 5);
    let a = g
        .legal_actions(actor)
        .into_iter()
        .find(|a| a.action.kind == "play" && a.action.card_id.as_deref() == Some(&id))
        .unwrap()
        .action;
    assert!(a.target_id.is_none());
    apply(g, actor, a);
    id
}
fn region(g: &mut Game, r: usize) {
    choose(g, vec![format!("region:{r}")]);
}
#[test]
fn jc050_whole_original_and_non_targeted_standard_program() {
    let c = catalog::card("JC050");
    assert_eq!(
        (&*c.name, &*c.kind, &*c.color, c.cost, &*c.magic),
        ("丧钟回响", "spell", "红", 5, "鲜血")
    );
    assert_eq!(c.loyalty, ["红色", "红色"]);
    assert_eq!(c.subtypes, ["灾难"]);
    assert!(!c.unique && c.keywords.is_empty());
    assert_eq!(deck::copy_limit(c), Some(3));
    let d = definition("JC050");
    assert_eq!(d.abilities.len(), 1);
    let a = &d.abilities[0];
    assert!(a.play_only && a.event.is_none() && a.targets.is_empty() && a.costs.is_empty());
    assert_eq!(a.timing, Timing::Standard);
    assert!(matches!(
        a.ops.as_slice(),
        [Op::ChooseRegion, Op::JC050DestroyChosenRegionCharacters]
    ));
}
#[test]
fn jc050_selects_after_responses_and_destroys_all_four_players_with_owner_graves() {
    for actor in 0..4 {
        let mut g = game(actor);
        let original_region = g.regions[2].card.id.clone();
        let mut ids = vec![];
        for seat in 0..4 {
            let id = board(&mut g, "JC002", seat, 2);
            if seat == 1 {
                g.board_mut(&id).unwrap().controller = 3;
            }
            ids.push(id);
        }
        let other = board(&mut g, "JC002", 1, 3);
        let spell = cast(&mut g, actor);
        assert!(g.pending.is_none());
        assert_eq!(g.regions[2].cards.len(), 4);
        checkpoint(&g);
        pass_top(&mut g);
        assert!(g.pending.is_some());
        assert_eq!(g.pending.as_ref().unwrap().seat, actor);
        checkpoint(&g);
        region(&mut g, 2);
        assert_eq!(g.regions[2].card.id, original_region);
        assert!(g.regions[2].cards.is_empty());
        assert!(g.board(&other).is_some());
        for seat in 0..4 {
            assert!(g.players[seat]
                .graveyard
                .iter()
                .any(|c| c.definition == "JC002" && c.controller == seat));
            assert!(g.board(&ids[seat]).is_none());
        }
        assert!(g.players[actor]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC050"));
        assert!(g
            .stack
            .iter()
            .all(|s| s.card.as_ref().is_none_or(|c| c.id != spell)));
        checkpoint(&g);
    }
}
#[test]
fn jc050_hidden_spells_die_without_death_triggers_and_public_characters_do_trigger() {
    let mut g = game(0);
    let public = board(&mut g, "JZ31", 0, 2);
    let hidden = board(&mut g, "JZ31", 0, 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    let hidden_spell = board(&mut g, "JC047", 1, 2);
    g.board_mut(&hidden_spell).unwrap().face_down = true;
    cast(&mut g, 0);
    pass_top(&mut g);
    region(&mut g, 2);
    assert!(
        g.board(&public).is_none()
            && g.board(&hidden).is_none()
            && g.board(&hidden_spell).is_none()
    );
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JZ31")
            .count(),
        2
    );
    assert_eq!(
        g.players[1]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JC047")
            .count(),
        1
    );
    let mut accepted = 0;
    for _ in 0..20 {
        if let Some(p) = &g.pending {
            if p.choice.min == Some(0) && p.choice.options.iter().any(|o| o.id == "accept") {
                accepted += 1;
                choose(&mut g, vec!["accept".into()]);
            } else {
                choose(&mut g, vec![]);
            }
        } else if !g.stack.is_empty() {
            pass_top(&mut g);
        } else {
            break;
        }
    }
    assert_eq!(accepted, 1);
    assert_eq!(g.regions[2].influence, [1, 0]);
    checkpoint(&g);
}
#[test]
fn jc050_destroy_is_non_damage_and_bypasses_target_shields() {
    let mut g = game(0);
    let id = board(&mut g, "JC070", 1, 2);
    g.board_mut(&id).unwrap().shield = 3;
    g.turn_attribute_modifiers.push(TurnAttributeModifier {
        target_instance: id.clone(),
        prevents_damage: true,
        expires_turn: g.turn,
        defense_bonus: 0,
        kill_bonus: 0,
        grants_retreat: false,
        printed_defense_override: None,
        ordinary_icons: Icons::default(),
        grants_renown: false,
    });
    cast(&mut g, 0);
    pass_top(&mut g);
    region(&mut g, 2);
    assert!(g.board(&id).is_none());
    checkpoint(&g);
}
#[test]
fn jc050_paid_response_move_and_return_escape_and_empty_region_is_valid() {
    let mut g = game(0);
    let moved = board(&mut g, "JC002", 1, 2);
    let returned = board(&mut g, "JC002", 2, 2);
    cast(&mut g, 0);
    let (_, c) = g.remove_board(&moved).unwrap();
    g.regions[3].cards.push(c);
    g.return_hand(&returned);
    pass_top(&mut g);
    region(&mut g, 2);
    assert!(g.board(&moved).is_some());
    assert!(g.players[2].hand.iter().any(|c| c.definition == "JC002"));
    checkpoint(&g);
}
#[test]
fn jc050_simultaneous_dead_observer_sees_whole_wave_before_departure() {
    let mut g = game(0);
    board(&mut g, "JC045", 0, 2);
    board(&mut g, "JC002", 1, 2);
    board(&mut g, "JC002", 2, 2);
    cast(&mut g, 0);
    pass_top(&mut g);
    region(&mut g, 2);
    // Optional declarations from the dead observer survive as three frozen events.
    let mut n = 0;
    for _ in 0..30 {
        if let Some(p) = &g.pending {
            if p.choice.options.iter().any(|o| o.id == "accept") {
                n += 1;
            }
            choose(&mut g, vec![]);
        } else if !g.stack.is_empty() {
            pass_top(&mut g);
        } else {
            break;
        }
    }
    assert_eq!(n, 3);
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JC045")
            .count(),
        1
    );
    checkpoint(&g);
}
#[test]
fn jc050_failed_payment_and_invalid_region_choice_are_atomic() {
    let mut g = game(0);
    let id = held(&mut g, "JC050", 0);
    fund(&mut g, 0, "JC047", 4);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            ..Action::new("play")
        },
    );
    cast(&mut g, 0);
    pass_top(&mut g);
    let p = g.pending.as_ref().unwrap().clone();
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec!["region:999".into()]),
            ..Action::new("choose")
        },
    );
    region(&mut g, 2);
    checkpoint(&g);
}
#[test]
fn jc050_original_region_choice_restores_and_duplicate_command_preserves_receipt() {
    let mut g = game(0);
    board(&mut g, "JC002", 1, 2);
    cast(&mut g, 0);
    pass_top(&mut g);
    let r = envelope(&g);
    let before = serde_json::to_string(&r).unwrap();
    let cmd = RoomCommand {
        command_id: "jc050-region".into(),
        expected_version: r.revision,
        action: SessionAction::Game {
            action: Action {
                choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
                selected: Some(vec!["region:2".into()]),
                ..Action::new("choose")
            },
        },
    };
    let e = r.transition(0, Some(cmd.clone()), 0).unwrap();
    assert!(e.error_code.is_none());
    let restored = RoomEnvelope::from_persisted(&e.state).unwrap();
    let views = (0..4).map(|s| restored.view(s, 0)).collect::<Vec<_>>();
    if let Ok(dir) = std::env::var("JC050_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/choose.json"),serde_json::to_vec(&serde_json::json!({"state":before,"seat":0,"command":cmd,"expected":e,"views":views})).unwrap()).unwrap();
    }
    let duplicate = restored.transition(0, Some(cmd), 600000).unwrap();
    assert_eq!(duplicate.state, e.state);
    assert_eq!(
        serde_json::to_value(duplicate.view).unwrap(),
        serde_json::to_value(restored.view(0, 600000)).unwrap()
    );
    checkpoint(&g);
}
#[test]
fn jc050_rejects_definition_transplants_nested_rekeys_and_changed_selection_order() {
    for dest in ["JC050", "JC002", "JC047"] {
        for kind in 0..5 {
            let mut defs = definitions().clone();
            let mut d = definition("JC050").clone();
            match kind {
                0 => {}
                1 => d.abilities[0].key = "other".into(),
                2 => d.abilities[0].ops.reverse(),
                3 => d.abilities[0].ops = vec![Op::ForEachLivingPlayer(d.abilities[0].ops.clone())],
                _ => d.abilities[0].timing = Timing::Fast,
            }
            defs.insert(dest.into(), d);
            assert_eq!(
                validate_definitions(&defs).is_ok(),
                dest == "JC050" && kind == 0
            );
        }
    }
}
#[test]
fn jc050_persisted_stack_and_choice_reject_op_injection_and_repeat_execution() {
    let mut g = game(0);
    cast(&mut g, 0);
    for kind in 0..5 {
        let mut bad = g.clone();
        let f = bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
        match kind {
            0 => f.steps.reverse(),
            1 => f.ability_key = "Draw".into(),
            2 => f.cursor = 1,
            3 => {
                f.steps[1].op = Op::Draw {
                    player: PlayerRef::Actor,
                    count: 1,
                    end: DeckEnd::Top,
                }
            }
            _ => f.chosen_region = Some(2),
        }
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
        assert!(RoomEnvelope::from_persisted(
            &serde_json::to_string(&RoomEnvelope::from_game(bad.clone())).unwrap()
        )
        .is_err());
        if let Ok(dir) = std::env::var("JC050_INVALID_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(format!("{dir}/stack-{kind}.json"),serde_json::to_vec(&serde_json::json!({"state":serde_json::to_string(&RoomEnvelope::from_game(bad)).unwrap()})).unwrap()).unwrap();
        }
    }
    pass_top(&mut g);
    let mut bad = g.clone();
    if let ChoiceResolution::Frame { frame, .. } = &mut bad.pending.as_mut().unwrap().resolution {
        frame.cursor = 0;
    }
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    checkpoint(&g);
}

#[test]
fn jc050_real_paid_response_return_escapes_and_standard_play_cannot_respond() {
    let mut g = game(0);
    let target = board(&mut g, "JC002", 0, 2);
    let other = board(&mut g, "JC002", 2, 2);
    cast(&mut g, 0);
    let invalid = held(&mut g, "JC050", 1);
    fund(&mut g, 1, "JC047", 5);
    reject(
        &mut g,
        1,
        Action {
            card_id: Some(invalid),
            ..Action::new("play")
        },
    );
    let bounce = held(&mut g, "JC006", 1);
    fund(&mut g, 1, "JC006", 2);
    let a = g
        .legal_actions(1)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&bounce)
                && a.action.target_id.as_deref() == Some(&target)
        })
        .unwrap()
        .action;
    apply(&mut g, 1, a);
    pass_top(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition == "JC002"));
    pass_top(&mut g);
    region(&mut g, 2);
    assert!(g.board(&other).is_none());
    checkpoint(&g);
}
#[test]
fn jc050_resolution_choice_uses_current_replacement_region_and_other_seats_cannot_interrupt() {
    let mut g = game(0);
    let old = g.regions[2].card.id.clone();
    let escaped = board(&mut g, "JC002", 1, 2);
    cast(&mut g, 0);
    // Explicit replacement input during the response interval, not a public room injection.
    g.return_hand(&escaped);
    g.regions[2].card = g.make_card("DQJC111", 0);
    let new = g.regions[2].card.id.clone();
    assert_ne!(old, new);
    let victim = board(&mut g, "JC002", 3, 2);
    pass_top(&mut g);
    checkpoint(&g);
    reject(&mut g, 1, Action::new("pass"));
    region(&mut g, 2);
    assert_eq!(g.regions[2].card.id, new);
    assert!(g.board(&victim).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.definition == "JC002"));
    checkpoint(&g);
}
