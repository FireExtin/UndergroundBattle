//! JZ49 uses explicit offline initial layouts, followed by normal paid commands.
//! These fixtures do not claim a natural UI match or production state edits.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::room::{Decision, RoomCommand, RoomEnvelope, SessionAction};
use crate::{catalog, deck, model::*, rules::*};

fn held(g: &mut Game, definition: &str, owner: usize) -> String {
    let c = g.make_card(definition, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn deploy(g: &mut Game, seat: usize, id: &str, region: usize) {
    let action = g
        .legal_actions(seat)
        .into_iter()
        .find(|a| {
            a.action.kind == "deploy"
                && a.action.card_id.as_deref() == Some(id)
                && a.action.region == Some(region)
        })
        .unwrap()
        .action;
    apply(g, seat, action);
    assert!(g.board(id).is_none());
    pass_top(g);
}
fn source(g: &Game) -> &Declaration {
    match &g.pending.as_ref().unwrap().resolution {
        ChoiceResolution::Declare { declaration, .. } => declaration,
        _ => panic!("expected optional entry declaration"),
    }
}
fn target(g: &mut Game, seat: usize) {
    assert_eq!(source(g).ability.event, Some(Event::Enter));
    choose(g, vec![player_id(seat)]);
    assert_eq!(
        g.stack.last().unwrap().frame.as_ref().unwrap().ability_key,
        "mill-two-entry"
    );
}
fn fresh_board(g: &Game, seat: usize) -> &Card {
    g.regions
        .iter()
        .flat_map(|r| &r.cards)
        .find(|c| c.definition == "JZ49" && c.controller == seat)
        .unwrap()
}
fn initial_entry(actor: usize) -> Game {
    let mut g = game(actor);
    fund(&mut g, actor, "JC084", 2);
    let id = held(&mut g, "JZ49", actor);
    deploy(&mut g, actor, &id, 2);
    g
}

#[test]
fn jz49_original_has_black_loyalty_two_death_domain_and_exact_slow_entry() {
    let c = catalog::card("JZ49");
    assert_eq!(
        (&*c.name, &*c.kind, &*c.color, c.cost, c.defense),
        ("蹒跚行尸", "character", "黑", 1, Some(1))
    );
    assert_eq!(c.loyalty, ["黑色", "黑色"]);
    assert_eq!(c.magic_icon, MagicIcon::Death);
    assert_eq!(c.subtypes, ["不死生物", "行尸"]);
    assert_eq!(
        c.permanent_icons,
        Icons {
            combat: 1,
            ..Icons::default()
        }
    );
    assert_eq!(c.temporary_icons, c.permanent_icons);
    assert_eq!(c.keywords, ["迟缓"]);
    assert!(!c.unique);
    assert_eq!(deck::copy_limit(c), Some(3));
    let d = definition("JZ49");
    assert!(d.traits.slow && !d.traits.public && !d.traits.spirit);
    assert_eq!(d.abilities.len(), 1);
    let a = &d.abilities[0];
    assert_eq!(a.event, Some(Event::Enter));
    assert!(!a.requires_ready_source);
    assert!(a.costs.is_empty() && a.modes.is_empty());
    assert_eq!(a.targets.len(), 1);
    assert_eq!(
        (a.targets[0].zone, a.targets[0].relation, a.targets[0].range),
        (Zone::Player, Relation::Any, Range::Anywhere)
    );
    assert!(matches!(
        a.ops.as_slice(),
        [Op::MoveDeckTopToGraveyard {
            player: PlayerRef::Target(0),
            count: 2
        }]
    ));
}

#[test]
fn jz49_paid_entry_is_exhausted_before_optional_declaration_and_can_decline() {
    for actor in 0..4 {
        let mut g = initial_entry(actor);
        let c = fresh_board(&g, actor);
        assert!(c.exhausted && !c.face_down);
        assert_eq!(g.icons(c, 2), Icons::default());
        assert_eq!(g.current_icons(c, 2).combat, 2);
        assert!(source(&g).source.card.exhausted);
        assert_eq!(source(&g).actor, actor);
        assert_eq!(g.resources(actor), 1);
        let deck = g.players[2].deck.clone();
        checkpoint(&g);
        choose(&mut g, vec![]);
        assert!(g.stack.is_empty() && g.pending.is_none());
        assert_eq!(
            serde_json::to_value(&g.players[2].deck).unwrap(),
            serde_json::to_value(deck).unwrap()
        );
        assert!(fresh_board(&g, actor).exhausted);
    }
}

#[test]
fn jz49_any_living_player_can_be_targeted_and_exact_top_two_are_milled() {
    for actor in 0..4 {
        for victim in 0..4 {
            let mut g = initial_entry(actor);
            let ids: Vec<_> = g.players[victim]
                .deck
                .iter()
                .map(|c| c.id.clone())
                .collect();
            let names: Vec<_> = g.players[victim]
                .deck
                .iter()
                .take(2)
                .map(|c| c.definition.clone())
                .collect();
            let hands: Vec<_> = g.players.iter().map(|p| p.hand.len()).collect();
            target(&mut g, victim);
            checkpoint(&g);
            pass_top(&mut g);
            assert_eq!(
                g.players[victim]
                    .deck
                    .iter()
                    .map(|c| c.id.clone())
                    .collect::<Vec<_>>(),
                ids[2..]
            );
            assert_eq!(
                g.players[victim]
                    .graveyard
                    .iter()
                    .map(|c| c.definition.clone())
                    .collect::<Vec<_>>(),
                names
            );
            assert_eq!(
                g.players.iter().map(|p| p.hand.len()).collect::<Vec<_>>(),
                hands
            );
            assert_eq!(g.status, "playing");
            assert!(fresh_board(&g, actor).exhausted);
        }
    }
}

#[test]
fn jz49_short_or_empty_deck_uses_available_cards_without_draw_failure() {
    for count in 0..=2 {
        let mut g = initial_entry(0);
        g.players[2].deck.truncate(count); // Explicit short-deck initial fixture.
        target(&mut g, 2);
        pass_top(&mut g);
        assert!(g.players[2].deck.is_empty());
        assert_eq!(g.players[2].graveyard.len(), count);
        assert!(!g.players[2].eliminated);
        assert_eq!(g.status, "playing");
    }
}

#[test]
fn jz49_hidden_deployment_is_ready_private_and_does_not_offer_entry_trigger() {
    let mut g = game(0);
    fund(&mut g, 0, "JC125", 1); // Secret deployment has no face-up loyalty requirement.
    let id = held(&mut g, "JZ49", 0);
    let action = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.kind == "conceal"
                && a.action.card_id.as_deref() == Some(&id)
                && a.action.region == Some(2)
        })
        .unwrap()
        .action;
    apply(&mut g, 0, action);
    let c = fresh_board(&g, 0);
    assert!(c.face_down && !c.exhausted);
    assert!(g.pending.is_none() && g.stack.is_empty());
    let own = g.view(0).regions[2].characters[0].clone();
    let enemy = g.view(2).regions[2].characters[0].clone();
    assert_eq!(own.card_id.as_deref(), Some("JZ49"));
    assert!(enemy.card_id.is_none());
    checkpoint(&g);
}

#[test]
fn jz49_paid_reveal_resets_hidden_identity_and_reapplies_slow() {
    let mut g = game(0);
    fund(&mut g, 0, "JC084", 2);
    let id = board(&mut g, "JZ49", 0, 2);
    g.board_mut(&id).unwrap().face_down = true;
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            ..Action::new("reveal")
        },
    );
    assert!(g.board(&id).is_none());
    pass_top(&mut g);
    let c = fresh_board(&g, 0);
    assert_ne!(c.id, id);
    assert!(!c.face_down && c.exhausted);
    assert_eq!(g.resources(0), 1);
    target(&mut g, 0);
    pass_top(&mut g);
}

#[test]
fn jz49_free_reveal_and_face_up_grave_entry_share_slow_but_hidden_grave_entry_does_not() {
    for grave in [false, true] {
        let mut g = game(0);
        let id = if grave {
            let c = g.make_card("JZ49", 0);
            let id = c.id.clone();
            g.players[0].graveyard.push(c);
            id
        } else {
            let id = board(&mut g, "JZ49", 0, 2);
            g.board_mut(&id).unwrap().face_down = true;
            id
        };
        if grave {
            g.world_graveyard_entry(0, 2, &id).unwrap();
        } else {
            g.world_reveal(0, &id, false).unwrap();
        }
        g.drive().unwrap();
        assert!(fresh_board(&g, 0).exhausted);
        assert_eq!(g.resources(0), 0);
        target(&mut g, 3);
        pass_top(&mut g);
        checkpoint(&g);
    }
    let mut g = game(0);
    fund(&mut g, 0, "JC084", 2);
    let c = g.make_card("JZ49", 0);
    let id = c.id.clone();
    g.players[0].graveyard.push(c);
    let spell = held(&mut g, "JC092", 0);
    let action = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&spell)
                && a.action.target_id.as_deref() == Some(&id)
                && a.action.region == Some(2)
        })
        .unwrap()
        .action;
    apply(&mut g, 0, action);
    pass_top(&mut g);
    assert!(fresh_board(&g, 0).face_down && !fresh_board(&g, 0).exhausted);
    assert!(g.pending.is_none());
    checkpoint(&g);
}

#[test]
fn jz49_declared_effect_survives_source_death_and_keeps_other_instance_independent() {
    let mut g = initial_entry(0);
    let first = fresh_board(&g, 0).id.clone();
    let other = board(&mut g, "JZ49", 2, 2);
    target(&mut g, 2);
    let before = g.players[2].deck.len();
    // Explicit target-validity primitive: a responding destroy removes source.
    g.remove_dead(&first, RemovalCause::Destroy);
    g.drive().unwrap();
    checkpoint(&g);
    pass_top(&mut g);
    assert_eq!(g.players[2].deck.len(), before - 2);
    assert!(g.board(&other).is_some_and(|(_, c)| !c.exhausted));
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JZ49")
            .count(),
        1
    );
}

#[test]
fn jz49_borrowed_source_declares_for_controller_and_milled_cards_return_to_owners() {
    let mut g = game(2);
    let source_id = board(&mut g, "JZ49", 0, 2);
    g.board_mut(&source_id).unwrap().controller = 2;
    g.enter_triggers(2, "JZ49", &source_id, false);
    g.drive().unwrap();
    assert_eq!(source(&g).actor, 2);
    // Explicit borrowed deck-card fixture exercises the existing owner destination.
    g.players[3].deck[0].owner = 1;
    let def = g.players[3].deck[0].definition.clone();
    target(&mut g, 3);
    pass_top(&mut g);
    assert_eq!(g.players[1].graveyard[0].definition, def);
    assert_eq!(g.players[1].graveyard[0].controller, 1);
    assert_eq!(g.players[3].graveyard.len(), 1);
}

#[test]
fn jz49_target_player_eliminated_before_resolution_does_not_mill() {
    let mut g = initial_entry(0);
    target(&mut g, 2);
    let before = serde_json::to_value(&g.players[2].deck).unwrap();
    g.players[2].eliminated = true; // Explicit response-time target invalidation fixture.
    pass_top(&mut g);
    assert_eq!(serde_json::to_value(&g.players[2].deck).unwrap(), before);
    assert!(g.players[2].graveyard.is_empty());
}

#[test]
fn jz49_insufficient_black_loyalty_or_assets_are_atomic_and_not_advertised() {
    for variant in 0..3 {
        let mut g = game(0);
        match variant {
            0 => fund(&mut g, 0, "JC125", 2),
            1 => fund(&mut g, 0, "JC084", 1),
            _ => {
                fund(&mut g, 0, "JC084", 2);
                for c in &mut g.players[0].assets {
                    c.exhausted = true;
                }
            }
        }
        let id = held(&mut g, "JZ49", 0);
        assert!(!g
            .legal_actions(0)
            .iter()
            .any(|a| a.action.kind == "deploy" && a.action.card_id.as_deref() == Some(&id)));
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
}

#[test]
fn jz49_next_turn_resets_slow_character_without_retriggering_entry() {
    let mut g = initial_entry(0);
    choose(&mut g, vec![]);
    let id = fresh_board(&g, 0).id.clone();
    g.effects.push_back(Effect::NextTurn); // Exact production reset operation fixture.
    g.drive().unwrap();
    assert!(!g.board(&id).unwrap().1.exhausted);
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.current_icons(g.board(&id).unwrap().1, 2).combat, 1);
    checkpoint(&g);
}

#[test]
fn jz49_region_movement_preserves_readiness_but_leave_and_reentry_exhausts_again() {
    let mut g = initial_entry(0);
    choose(&mut g, vec![]);
    let id = fresh_board(&g, 0).id.clone();
    g.effects.push_back(Effect::NextTurn);
    g.drive().unwrap();
    // Explicit movement primitive fixture: this is EnterRegion, not Enter.
    let (_, c) = g.remove_board(&id).unwrap();
    g.regions[3].cards.push(c);
    let snapshot = g.source_snapshot(g.board(&id).unwrap().1, Some(3));
    g.emit_event(0, snapshot, Event::EnterRegion);
    g.drive().unwrap();
    assert!(!g.board(&id).unwrap().1.exhausted);
    assert!(g.pending.is_none());
    g.remove_dead(&id, RemovalCause::Destroy);
    g.drive().unwrap();
    let dead = g.players[0].graveyard.last().unwrap().id.clone();
    g.world_graveyard_entry(0, 3, &dead).unwrap();
    g.drive().unwrap();
    let c = fresh_board(&g, 0);
    assert_ne!(c.id, id);
    assert!(c.exhausted);
    assert_eq!(source(&g).source.region, Some(3));
    checkpoint(&g);
}

#[test]
fn jz49_asset_building_does_not_execute_face_up_character_keywords() {
    let mut g = game(0);
    let id = held(&mut g, "JZ49", 0);
    let action = g
        .legal_actions(0)
        .into_iter()
        .find(|a| a.action.kind == "asset" && a.action.card_id.as_deref() == Some(&id))
        .unwrap()
        .action;
    apply(&mut g, 0, action);
    assert!(g.players[0].assets.iter().any(|c| c.definition == "JZ49"));
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert!(g
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .all(|c| c.definition != "JZ49"));
}

#[test]
fn jz49_complete_definition_mutations_and_transplants_are_rejected() {
    for variant in 0..12 {
        let mut registry = definitions().clone();
        let d = registry.get_mut("JZ49").unwrap();
        match variant {
            0 => d.traits.slow = false,
            1 => d.traits.spirit = true,
            2 => d.abilities[0].event = None,
            3 => d.abilities[0].event = Some(Event::Reveal),
            4 => d.abilities[0].requires_ready_source = true,
            5 => {
                d.abilities[0].ops = vec![Op::MoveDeckTopToGraveyard {
                    player: PlayerRef::Target(0),
                    count: 3,
                }]
            }
            6 => d.abilities[0].targets[0].relation = Relation::EnemyTeam,
            7 => d.abilities[0].costs = vec![Cost::Assets(1)],
            8 => d.abilities.push(d.abilities[0].clone()),
            9 => d.traits.public = true,
            10 => d.abilities[0].per_turn_limit = Some(1),
            _ => d.graveyard_face_up = true,
        }
        assert!(
            validate_definitions(&registry).is_err(),
            "variant {variant}"
        );
    }
    for id in ["JC125", "XQ38", "JZ31"] {
        let mut registry = definitions().clone();
        registry.insert(id.into(), definition("JZ49").clone());
        assert!(validate_definitions(&registry).is_err());
        assert!(validate_ability(id, &definition("JZ49").abilities[0]).is_err());
        let mut registry = definitions().clone();
        registry.get_mut(id).unwrap().traits.slow = true;
        assert!(validate_definitions(&registry).is_err());
    }
}

fn command(
    room: &mut RoomEnvelope,
    steps: &mut Vec<serde_json::Value>,
    seat: usize,
    action: SessionAction,
) {
    let state = serde_json::to_string(room).unwrap();
    let cmd = RoomCommand {
        command_id: format!("jz49-chain-{}", steps.len()),
        expected_version: room.revision,
        action,
    };
    let expected = room.transition(seat, Some(cmd.clone()), 0).unwrap();
    assert!(
        expected.error_code.is_none(),
        "{:?}",
        expected.error_message
    );
    assert_eq!(
        serde_json::to_string(&room.replay_events(&expected.journal).unwrap()).unwrap(),
        expected.state
    );
    *room = RoomEnvelope::from_persisted(&expected.state).unwrap();
    steps.push(
        serde_json::json!({"state":state,"seat":seat,"command":cmd,"expected":expected,
        "views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>() }),
    );
}

#[test]
fn jz49_continuous_paid_response_destroys_source_but_original_mill_resolves_after_restore() {
    let mut g = initial_entry(0);
    let source_id = fresh_board(&g, 0).id.clone();
    fund(&mut g, 2, "JC102", catalog::card("JC102").cost as usize);
    let spell = held(&mut g, "JC102", 2);
    let mut room = envelope(&g);
    let initial = serde_json::to_string(&room).unwrap();
    let initial_deck = room.players[2].deck.len();
    let pending = room.pending.clone().unwrap();
    let mut steps = vec![];
    command(
        &mut room,
        &mut steps,
        0,
        SessionAction::Game {
            action: Action {
                choice_id: Some(pending.choice.id),
                selected: Some(vec!["p2".into()]),
                ..Action::new("choose")
            },
        },
    );
    let underlying = room.stack[0].id.clone();
    while room.pacing.window.as_ref().unwrap().holder_team == 0 {
        let w = room.pacing.window.clone().unwrap();
        let seat = *w
            .members
            .iter()
            .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
            .unwrap()
            .0;
        command(
            &mut room,
            &mut steps,
            seat,
            SessionAction::PassResponse { window_id: w.id },
        );
    }
    let window = room.pacing.window.clone().unwrap();
    assert_eq!(window.holder_team, 1);
    command(
        &mut room,
        &mut steps,
        2,
        SessionAction::BeginResponse {
            window_id: window.id.clone(),
            intent_id: "jz49-destroy-response".into(),
        },
    );
    assert!(matches!(
        room.pacing.window.as_ref().unwrap().members[&2],
        Decision::Composing { .. }
    ));
    let action = room
        .legal_actions(2)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&spell)
                && a.action.target_id.as_deref() == Some(&source_id)
        })
        .unwrap()
        .action;
    command(
        &mut room,
        &mut steps,
        2,
        SessionAction::SubmitResponse {
            window_id: window.id,
            intent_id: "jz49-destroy-response".into(),
            action,
        },
    );
    for _ in 0..24 {
        if room.stack.is_empty() {
            break;
        }
        let w = room.pacing.window.clone().unwrap();
        let seat = *w
            .members
            .iter()
            .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
            .unwrap()
            .0;
        command(
            &mut room,
            &mut steps,
            seat,
            SessionAction::PassResponse { window_id: w.id },
        );
    }
    assert!(room.stack.is_empty() && room.pending.is_none());
    assert!(room.board(&source_id).is_none());
    assert_eq!(room.players[2].deck.len(), initial_deck - 2);
    assert_eq!(
        room.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JZ49")
            .count(),
        1
    );
    assert_eq!(room.resources(2), 0);
    if let Ok(dir) = std::env::var("JZ49_CHAIN_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/paid-response-chain.json"),
            serde_json::to_vec(&serde_json::json!({
            "initialState":initial,"steps":steps,"finalState":serde_json::to_string(&room).unwrap(),
            "underlyingStackInstance":underlying,"sourceInstance":source_id}))
            .unwrap(),
        )
        .unwrap();
    }
}

#[cfg(feature = "native")]
#[tokio::test]
async fn jz49_sqlite_choice_and_response_receipts_survive_reopen_without_repeat_effect() {
    use crate::service::{CreateRoom, JoinRoom, Store};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("jz49-local-fixture.sqlite");
    let store = Store::open(&path).unwrap();
    let host = store
        .create(CreateRoom {
            name: "P0".into(),
            mode: "teams".into(),
            deck_id: "watchers".into(),
            deck_draft: None,
        })
        .await
        .unwrap();
    let mut sessions = vec![host];
    for seat in 1..4 {
        sessions.push(
            store
                .join(JoinRoom {
                    invite_code: sessions[0].invite_code.clone(),
                    name: format!("P{seat}"),
                    deck_id: "watchers".into(),
                    deck_draft: None,
                })
                .await
                .unwrap(),
        );
    }
    drop(store);
    let mut g = initial_entry(0);
    g.room_id = sessions[0].room_id.clone();
    g.invite_code = sessions[0].invite_code.clone();
    let mut room = envelope(&g);
    // Private, explicitly synthetic local Store fixture. No production DB is accessed.
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute(
        "UPDATE rooms SET state=?1, revision=?2 WHERE id=?3",
        rusqlite::params![
            serde_json::to_string(&room).unwrap(),
            room.revision,
            g.room_id
        ],
    )
    .unwrap();
    drop(db);
    let p = room.pending.clone().unwrap();
    let first = RoomCommand {
        command_id: "jz49-store-choice".into(),
        expected_version: room.revision,
        action: SessionAction::Game {
            action: Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec!["p2".into()]),
                ..Action::new("choose")
            },
        },
    };
    let mut cmds = vec![(0, first)];
    let mut accepted = vec![];
    let before_deck = room.players[2].deck.len();
    for _ in 0..16 {
        let (seat, cmd) = cmds.last().unwrap().clone();
        let expected = room.transition(seat, Some(cmd.clone()), 0).unwrap();
        assert!(expected.error_code.is_none());
        let store = Store::open(&path).unwrap();
        let view = store
            .command_at_now(&g.room_id, &sessions[seat].token, cmd.clone(), 0)
            .await
            .unwrap();
        drop(store);
        room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        assert_eq!(
            serde_json::to_value(&view).unwrap(),
            serde_json::to_value(room.view(seat, 0)).unwrap()
        );
        accepted.push(serde_json::to_value(view).unwrap());
        let store = Store::open(&path).unwrap();
        let duplicate = store
            .command_at_now(&g.room_id, &sessions[seat].token, cmd.clone(), 0)
            .await
            .unwrap();
        assert_eq!(
            accepted.last().unwrap(),
            &serde_json::to_value(duplicate).unwrap()
        );
        let mut conflict = cmd.clone();
        conflict.expected_version += 1;
        assert_eq!(
            store
                .command_at_now(&g.room_id, &sessions[seat].token, conflict, 0)
                .await
                .unwrap_err()
                .error,
            "command_id_conflict"
        );
        drop(store);
        if room.stack.is_empty() {
            break;
        }
        let w = room.pacing.window.clone().unwrap();
        let next_seat = *w
            .members
            .iter()
            .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
            .unwrap()
            .0;
        cmds.push((
            next_seat,
            RoomCommand {
                command_id: format!("jz49-store-pass-{}", cmds.len()),
                expected_version: room.revision,
                action: SessionAction::PassResponse { window_id: w.id },
            },
        ));
    }
    assert!(room.stack.is_empty());
    assert_eq!(room.players[2].deck.len(), before_deck - 2);
    let store = Store::open(&path).unwrap();
    // Retry the original choice after its effect has finished and deadlines passed.
    assert_eq!(
        serde_json::to_value(
            store
                .command_at_now(&g.room_id, &sessions[0].token, cmds[0].1.clone(), 90_000)
                .await
                .unwrap()
        )
        .unwrap(),
        accepted[0]
    );
    drop(store);
    let db = rusqlite::Connection::open(&path).unwrap();
    let stored: String = db
        .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(stored, serde_json::to_string(&room).unwrap());
    let n: u64 = db
        .query_row(
            "SELECT COUNT(*) FROM commands WHERE room_id=?1",
            [&g.room_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, cmds.len() as u64);
}
