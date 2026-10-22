//! Independent recovery contracts on the fixed ABI94 correction.
//! Explicit native layouts; no browser, host authentication, or full-game claim.
use crate::{
    jc029_tests::{envelope, pass_top},
    model::*,
    room::{Decision, PriorityWindow, RoomCommand, RoomEnvelope, SessionAction},
    unit_support::{act, field, fund, game, held, pass},
};
use serde::Serialize;
use serde_json::{json, Value};

fn state<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap()
}

fn hidden(g: &mut Game, definition: &str, seat: usize) -> String {
    let id = field(g, definition, seat);
    g.board_mut(&id).unwrap().face_down = true;
    id
}

fn inspection_pending() -> Game {
    let mut g = game();
    fund(&mut g, 0, "JC057", 1);
    let spell = held(&mut g, "JC062", 0);
    let targets = vec![hidden(&mut g, "JC125", 2), hidden(&mut g, "JC029", 3)];
    act(
        &mut g,
        0,
        Action {
            card_id: Some(spell),
            selected: Some(targets),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    assert!(matches!(
        g.pending.as_ref().unwrap().resolution,
        ChoiceResolution::Frame {
            choice: FrameChoice::JC062Inspection { .. },
            ..
        }
    ));
    g
}

fn inspection_parts(g: &mut Game) -> (&mut ResolutionFrame, &mut Vec<Card>) {
    match &mut g.pending.as_mut().unwrap().resolution {
        ChoiceResolution::Frame {
            frame,
            choice: FrameChoice::JC062Inspection { inspected },
        } => (frame, inspected),
        _ => panic!("fixture must be a private inspection continuation"),
    }
}

fn choose_action(id: &str) -> Action {
    Action {
        choice_id: Some(id.into()),
        selected: Some(vec![]),
        ..Action::new("choose")
    }
}

fn room_command(room: &RoomEnvelope, label: &str, action: SessionAction) -> RoomCommand {
    RoomCommand {
        command_id: format!("independent-{label}-{}", room.revision),
        expected_version: room.revision,
        action,
    }
}

fn reject_choice(g: &mut Game, room: &RoomEnvelope, seat: usize, action: Action) {
    let before_game = state(g);
    let before_room = state(room);
    assert!(g.apply(seat, action.clone()).is_err());
    assert_eq!(state(g), before_game);
    let result = room
        .transition(
            seat,
            Some(room_command(room, "invalid-choice", action.into())),
            0,
        )
        .unwrap();
    assert_eq!(result.outcome, "rejected");
    assert_eq!(result.error_code.as_deref(), Some("invalid_action"));
    assert!(!result.changed);
    assert!(result.journal.is_empty());
    assert_eq!(result.version, room.revision);
    assert_eq!(result.state, before_room);
    RoomEnvelope::from_persisted(&result.state).unwrap();
}

fn pass_room_top(room: &mut RoomEnvelope) {
    let count = room.game.stack.len();
    assert!(count > 0);
    for _ in 0..32 {
        if room.game.stack.len() < count || room.game.pending.is_some() {
            return;
        }
        let window = room.pacing.window.as_ref().unwrap();
        let seat = *window
            .members
            .iter()
            .find(|(_, decision)| matches!(decision, Decision::Undecided { .. }))
            .unwrap()
            .0;
        let result = room
            .transition(
                seat,
                Some(room_command(
                    room,
                    "pass",
                    SessionAction::PassResponse {
                        window_id: window.id.clone(),
                    },
                )),
                0,
            )
            .unwrap();
        assert_eq!(result.outcome, "accepted", "{:?}", result.error_message);
        assert!(result.changed && result.error_code.is_none());
        *room = RoomEnvelope::from_persisted(&result.state).unwrap();
    }
    panic!("restored Room did not resolve one stack item within 32 passes");
}

#[test]
fn independent_recovery_pending_domain_mutations_agree() {
    let original = inspection_pending();
    let valid_room = envelope(&original);
    Game::from_persisted(&state(&original)).unwrap();
    RoomEnvelope::from_persisted(&state(&valid_room)).unwrap();
    let mutations: [(&str, fn(&mut Game)); 5] = [
        ("chooser seat", |g| g.pending.as_mut().unwrap().seat = 1),
        ("choice player", |g| {
            g.pending.as_mut().unwrap().choice.player_id = player_id(1)
        }),
        ("inspection identity", |g| {
            inspection_parts(g).1[0].definition = "JC029".into()
        }),
        ("payment receipt", |g| {
            inspection_parts(g).0.already_paid.clear()
        }),
        ("cursor versus accepted guard", |g| {
            inspection_parts(g).0.cursor = 0
        }),
    ];
    for (label, mutate) in mutations {
        let mut altered = original.clone();
        mutate(&mut altered);
        let mut wrapped = valid_room.clone();
        wrapped.game = altered.clone();
        let game_error =
            Game::from_persisted(&state(&altered)).expect_err(&format!("Game must reject {label}"));
        let room_error = RoomEnvelope::from_persisted(&state(&wrapped))
            .expect_err(&format!("Room must reject {label}"));
        assert_eq!(game_error, room_error, "shared domain error for {label}");
    }
}

#[test]
fn independent_recovery_private_pending_roundtrip_preserves_authority() {
    let original = inspection_pending();
    let original_room = envelope(&original);
    let saved_game = state(&original);
    let saved_room = state(&original_room);
    let mut g = Game::from_persisted(&saved_game).unwrap();
    let mut room = RoomEnvelope::from_persisted(&saved_room).unwrap();
    let targets: Vec<_> = match &g.pending.as_ref().unwrap().resolution {
        ChoiceResolution::Frame { frame, .. } => {
            frame.targets.iter().map(|t| t.id.clone()).collect()
        }
        _ => unreachable!(),
    };

    // Query order must not transfer the actor's private projection to another seat,
    // consume IDs/randomness, or change the recoverable continuation.
    for seat in [3, 0, 2, 1, 0, 1, 3, 2] {
        let view = g.view(seat);
        let room_view = room.view(seat, 0);
        assert_eq!(state(&view), state(&original.view(seat)));
        assert_eq!(state(&room_view), state(&original_room.view(seat, 0)));
        assert_eq!(state(&view), state(&room_view.game));
        assert!(view.private_inspections.is_empty());
        if seat == 0 {
            let previews = &view.pending_choice.as_ref().unwrap().preview_cards;
            assert_eq!(previews.len(), 2);
            assert_eq!(previews[0].card_id.as_deref(), Some("JC125"));
            assert_eq!(previews[1].card_id.as_deref(), Some("JC029"));
        } else {
            assert!(view.pending_choice.is_none());
        }
        for id in &targets {
            let card = view.regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == *id)
                .unwrap();
            assert!(card.face_down);
            if card.controller != player_id(seat) {
                assert!(card.card_id.is_none());
            }
        }
    }
    assert_eq!(state(&g), saved_game);
    assert_eq!(state(&room), saved_room);

    let choice_id = g.pending.as_ref().unwrap().choice.id.clone();
    reject_choice(&mut g, &room, 1, choose_action(&choice_id));
    reject_choice(&mut g, &room, 0, choose_action("already-retired-choice"));
    let deck_before = g.players[0].deck.len();
    let hand_before = g.players[0].hand.len();
    let assets_before = state(&g.players[0].assets);
    let command = room_command(&room, "confirm", choose_action(&choice_id).into());
    g.apply(0, choose_action(&choice_id)).unwrap();
    let result = room.transition(0, Some(command), 0).unwrap();
    assert_eq!(result.outcome, "accepted", "{:?}", result.error_message);
    assert!(result.changed && result.error_code.is_none());
    room = RoomEnvelope::from_persisted(&result.state).unwrap();
    g = Game::from_persisted(&state(&g)).unwrap();
    assert_eq!(state(&g), state(&room.game));
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.players[0].deck.len(), deck_before - 1);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(state(&g.players[0].assets), assets_before);
    assert!(targets.iter().all(|id| g.board(id).unwrap().1.face_down));
    for seat in 0..4 {
        assert!(room.view(seat, 0).pending_choice.is_none());
    }
    // Fresh envelope version and a different command ID isolate spent-choice
    // rejection from the host's receipt/version-conflict protections.
    reject_choice(&mut g, &room, 0, choose_action(&choice_id));
}

#[test]
fn independent_recovery_old_target_is_not_rebound_by_restore() {
    let mut original = game();
    fund(&mut original, 0, "JC057", 1);
    let spell = held(&mut original, "JC062", 0);
    let target = hidden(&mut original, "JC125", 2);
    let lookalike = hidden(&mut original, "JC125", 2);
    let deck_before = original.players[0].deck.len();
    act(
        &mut original,
        0,
        Action {
            card_id: Some(spell),
            selected: Some(vec![target.clone()]),
            ..Action::new("play")
        },
    );
    for _ in 0..4 {
        if original.can_fast(2) {
            break;
        }
        pass(&mut original);
    }
    act(
        &mut original,
        2,
        Action {
            card_id: Some(target.clone()),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut original);
    assert!(original.board(&target).is_none());
    assert_eq!(original.stack.len(), 1);
    assert_eq!(
        original.stack[0].frame.as_ref().unwrap().targets[0].id,
        target
    );
    assert!(original.board(&lookalike).unwrap().1.face_down);
    let resources = original.resources(0);
    let random = original.random;
    let mut g = Game::from_persisted(&state(&original)).unwrap();
    let mut room = RoomEnvelope::from_persisted(&state(&envelope(&original))).unwrap();
    // Resume both decoded values, rather than continuing the pre-save fixture.
    pass_top(&mut g);
    pass_room_top(&mut room);
    assert_eq!(state(&g), state(&room.game));
    for resolved in [&g, &room.game] {
        assert!(resolved.stack.is_empty() && resolved.pending.is_none());
        assert_eq!(resolved.resources(0), resources);
        assert_eq!(resolved.players[0].deck.len(), deck_before);
        assert!(resolved.players[0].hand.is_empty());
        assert_eq!(resolved.random, random);
        assert!(resolved.board(&lookalike).unwrap().1.face_down);
        assert_eq!(
            resolved.players[0]
                .graveyard
                .iter()
                .filter(|c| c.definition == "JC062")
                .count(),
            1
        );
    }
    Game::from_persisted(&state(&g)).unwrap();
    RoomEnvelope::from_persisted(&state(&room)).unwrap();
}

#[test]
fn independent_recovery_domain_schema_and_versions_reject() {
    let original = inspection_pending();
    let valid: Value = serde_json::to_value(&original).unwrap();
    let valid_room = serde_json::to_value(envelope(&original)).unwrap();
    let mut cases = vec![];
    let mut missing_schema = valid.clone();
    missing_schema
        .as_object_mut()
        .unwrap()
        .remove("state_schema");
    cases.push(("missing Game schema", missing_schema));
    for (label, pointer, value) in [
        ("old Game schema", "/state_schema", json!(1)),
        ("Room schema is not Game schema", "/state_schema", json!(3)),
        ("old engine", "/versions/engine", json!("old-engine")),
        ("old pool", "/versions/cardPool", json!("old-pool")),
        ("old rules", "/versions/rules", json!("old-rules")),
    ] {
        let mut altered = valid.clone();
        *altered.pointer_mut(pointer).unwrap() = value;
        cases.push((label, altered));
    }
    for (label, altered) in cases {
        let mut wrapped = valid_room.clone();
        wrapped["game"] = altered.clone();
        assert!(Game::from_persisted(&state(&altered)).is_err(), "{label}");
        assert!(
            RoomEnvelope::from_persisted(&state(&wrapped)).is_err(),
            "{label} must not be silently migrated through Room"
        );
    }
}

#[test]
fn independent_recovery_room_metadata_stays_an_envelope_boundary() {
    let original = inspection_pending();
    let room = envelope(&original);
    let valid = serde_json::to_value(&room).unwrap();
    let mut cases = vec![];
    let mut missing_schema = valid.clone();
    missing_schema
        .as_object_mut()
        .unwrap()
        .remove("state_schema");
    cases.push(("missing Room schema", missing_schema));
    for (label, pointer, value) in [
        ("old Room schema", "/state_schema", json!(2)),
        ("old outer engine", "/versions/engine", json!("old-engine")),
        (
            "outer revision mismatch",
            "/revision",
            json!(room.revision + 1),
        ),
    ] {
        let mut altered = valid.clone();
        *altered.pointer_mut(pointer).unwrap() = value;
        cases.push((label, altered));
    }
    let mut extra_window = room.clone();
    extra_window.pacing.window_seq = 1;
    extra_window.pacing.window = Some(PriorityWindow {
        id: "response:1".into(),
        stack_top_id: "cannot-respond-during-private-choice".into(),
        holder_team: original.priority_team,
        members: Default::default(),
    });
    cases.push((
        "response during choice",
        serde_json::to_value(extra_window).unwrap(),
    ));
    for (label, altered) in cases {
        assert_eq!(state(&altered["game"]), state(&valid["game"]));
        Game::from_persisted(&state(&altered["game"]))
            .expect("envelope-only corruption must leave the Game snapshot valid");
        assert!(
            RoomEnvelope::from_persisted(&state(&altered)).is_err(),
            "{label}"
        );
    }
}
