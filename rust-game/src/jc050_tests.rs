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
fn invalid_room(name: &str, r: RoomEnvelope) {
    let state = serde_json::to_string(&r).unwrap();
    assert!(
        Game::from_persisted(&serde_json::to_string(&r.game).unwrap())
            .unwrap_err()
            .contains("JC050")
    );
    assert!(RoomEnvelope::from_persisted(&state)
        .unwrap_err()
        .contains("JC050"));
    if let Ok(dir) = std::env::var("JC050_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/{name}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state,"expectedErrorContains":"JC050"}))
                .unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn jc050_persisted_stack_and_choice_reject_op_injection_and_repeat_execution() {
    let mut g = game(0);
    cast(&mut g, 0);
    let original = envelope(&g);
    for kind in 0..9 {
        let mut r = original.clone();
        let item = r.game.stack.last_mut().unwrap();
        match kind {
            5 => item.deploy_region = Some(2),
            6 => item.reveal = true,
            7 => item.target = Some("region:2".into()),
            _ => {
                let f = item.frame.as_mut().unwrap();
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
                    4 => f.chosen_region = Some(2),
                    _ => {
                        f.ability_key = "deploy".into();
                        f.steps.clear();
                    }
                }
            }
        }
        invalid_room(&format!("stack-{kind}"), r);
    }
    pass_top(&mut g);
    let original = envelope(&g);
    for kind in 0..10 {
        let mut r = original.clone();
        let p = r.game.pending.as_mut().unwrap();
        match kind {
            0 => {
                if let ChoiceResolution::Frame { frame, .. } = &mut p.resolution {
                    frame.cursor = 0
                }
            }
            1 => p.choice.kind = "damage".into(),
            2 => p.seat = 1,
            3 => p.choice.player_id = "p1".into(),
            4 => p.choice.allow_decline = Some(true),
            5 => p.choice.max = Some(2),
            6 => p.choice.amount = Some(1),
            7 => p.choice.options[0].id = "region:999".into(),
            8 => p.choice.kind = "order".into(),
            _ => {
                if let ChoiceResolution::Frame { frame, .. } = &mut p.resolution {
                    frame.cursor = 2
                }
            }
        }
        invalid_room(&format!("choice-{kind}"), r);
    }
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
#[test]
fn jc050_predeclared_region_and_target_reject_before_payment_and_quote() {
    let mut g = game(0);
    let id = held(&mut g, "JC050", 0);
    fund(&mut g, 0, "JC047", 5);
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
    board(&mut g, "JC002", 0, 2);
    cast(&mut g, 0);
    let bad = held(&mut g, "JC050", 1);
    fund(&mut g, 1, "JC047", 5);
    // BeginResponse requires a genuinely offered fast action; the bad standard
    // transaction is only the subsequent draft, not a fabricated window offer.
    held(&mut g, "JC006", 1);
    fund(&mut g, 1, "JC006", 2);
    let room = envelope(&g);
    let window = room.pacing.window.as_ref().unwrap().id.clone();
    let begin = RoomCommand {
        command_id: "jc050-quote-begin".into(),
        expected_version: room.revision,
        action: SessionAction::BeginResponse {
            window_id: window.clone(),
            intent_id: "jc050-bad-draft".into(),
        },
    };
    let ready = room.transition(1, Some(begin), 0).unwrap();
    assert!(ready.error_code.is_none(), "{:?}", ready.error_message);
    let composing = RoomEnvelope::from_persisted(&ready.state).unwrap();
    let bad_action = Action {
        card_id: Some(bad),
        region: Some(2),
        ..Action::new("play")
    };
    let request = QuoteRequest {
        window_id: window.clone(),
        intent_id: "jc050-bad-draft".into(),
        draft: Some(bad_action.clone()),
    };
    let q = composing.quote(1, request.clone()).unwrap();
    assert!(!q.ready && q.error.as_ref().unwrap().contains("JC050"));
    assert_eq!(serde_json::to_string(&composing).unwrap(), ready.state);
    if let Ok(dir) = std::env::var("JC050_FRONTEND_DIR") {
        std::fs::write(
            format!("{dir}/quote.json"),
            serde_json::to_vec(
                &serde_json::json!({"state":ready.state,"seat":1,"request":request,"expected":q}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    let cmd = RoomCommand {
        command_id: "jc050-bad-submit".into(),
        expected_version: composing.revision,
        action: SessionAction::SubmitResponse {
            window_id: window,
            intent_id: "jc050-bad-draft".into(),
            action: bad_action,
        },
    };
    let e = composing.transition(1, Some(cmd), 0).unwrap();
    assert!(e.error_code.is_some());
    assert_eq!(e.state, ready.state);
    checkpoint(&g);
}
#[test]
fn jc050_true_hidden_reveal_is_admitted_and_empty_program_aliases_reject() {
    let mut g = game(0);
    let id = board(&mut g, "JC050", 0, 2);
    g.board_mut(&id).unwrap().face_down = true;
    fund(&mut g, 0, "JC047", 5);
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| a.action.kind == "reveal" && a.action.card_id.as_deref() == Some(&id))
        .unwrap()
        .action;
    apply(&mut g, 0, a);
    let original = envelope(&g);
    for kind in 0..6 {
        let mut r = original.clone();
        let item = r.game.stack.last_mut().unwrap();
        match kind {
            0 => item.reveal = false,
            1 => item.deploy_region = Some(1),
            5 => item.card.as_mut().unwrap().controller = 1,
            _ => {
                let f = item.frame.as_mut().unwrap();
                match kind {
                    2 => f.source.card.face_down = false,
                    3 => f.source.region = None,
                    _ => f.ability_key = "deploy".into(),
                }
            }
        };
        invalid_room(&format!("reveal-{kind}"), r);
    }
    pass_top(&mut g);
    checkpoint(&g);
}
#[test]
fn jc050_host_attachments_follow_owner_and_source_departure_causes_second_death_wave() {
    let mut g = game(0);
    let source = board(&mut g, "JC002", 0, 2);
    let mut curse = g.make_card("JC036", 1);
    curse.controller = 0;
    g.attachments.push(Attachment {
        card: curse,
        host_id: source.clone(),
    });
    let criminal = board(&mut g, "JC084", 1, 3);
    g.add_control(
        &criminal,
        0,
        ControlLifetime::SourceLeaves {
            source_instance: source.clone(),
        },
        SubtypeChange::None,
    );
    let dependent = board(&mut g, "JZ48", 0, 3);
    g.board_mut(&dependent).unwrap().damage = 1;
    assert_eq!(g.defense(g.board(&dependent).unwrap().1, 3), 2);
    checkpoint(&g);
    cast(&mut g, 0);
    pass_top(&mut g);
    region(&mut g, 2);
    assert!(g.board(&source).is_none() && g.board(&dependent).is_none());
    assert_eq!(g.board(&criminal).unwrap().1.controller, 1);
    assert!(g.attachments.is_empty());
    assert!(g.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC036"));
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JZ48"));
    checkpoint(&g);
}
