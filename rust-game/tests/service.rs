#![cfg(feature = "native")]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use hegemony_server::{
    model::{Action, ChoiceResolution, FrameChoice, Game, GuardState, Window},
    service::{self, Command, CreateRoom, JoinRoom, Store},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
fn command(id: &str, version: u64, kind: &str) -> Command {
    Command {
        command_id: id.into(),
        expected_version: version,
        action: Action::new(kind),
    }
}
#[tokio::test]
async fn paid_sacrifice_death_trigger_and_accepted_frame_restore_without_repayment() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("paid-frame.sqlite3");
    let store = Store::open(&db).unwrap();
    let (a, b) = pair(&store).await;
    drop(store);
    // Explicit initial room-layout fixture. No database writes occur after the command sequence starts.
    let connection = rusqlite::Connection::open(&db).unwrap();
    let initial: String = connection
        .query_row("SELECT state FROM rooms WHERE id=?1", [&a.room_id], |r| {
            r.get(0)
        })
        .unwrap();
    let mut game = Game::from_persisted(&initial).unwrap();
    for p in &mut game.players {
        p.ready = true;
    }
    game.apply(0, Action::new("start")).unwrap();
    while let Some(p) = game.pending.clone() {
        game.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    game.window = Some(Window::Action(0));
    game.first_team = 0;
    game.active_team = 0;
    game.priority_team = 0;
    game.version = 1;
    game.log.clear();
    let sacrifice = game.make_card("XQ12", 1);
    let source_id = sacrifice.id.clone();
    let mut sacrifice = sacrifice;
    sacrifice.controller = 0;
    game.regions[0].cards.push(sacrifice);
    let spell = game.make_card("JC049", 0);
    let spell_id = spell.id.clone();
    game.players[0].hand = vec![spell];
    let discard = game.make_card("LC20", 1);
    game.players[1].hand = vec![discard];
    for _ in 0..2 {
        let asset = game.make_card("JC042", 0);
        game.players[0].assets.push(asset);
    }
    let serialized = serde_json::to_string(&game).unwrap();
    connection
        .execute(
            "UPDATE rooms SET state=?2,initial_state=?2 WHERE id=?1",
            rusqlite::params![a.room_id, serialized],
        )
        .unwrap();
    connection
        .execute("DELETE FROM journal WHERE room_id=?1", [&a.room_id])
        .unwrap();
    drop(connection);
    let store = Store::open(&db).unwrap();
    let cast = Command {
        command_id: "paid-cast".into(),
        expected_version: 1,
        action: Action {
            card_id: Some(spell_id),
            cost_selected: Some(vec![source_id.clone()]),
            ..Action::new("play")
        },
    };
    let paid = store
        .command(&a.room_id, &a.token, cast.clone())
        .await
        .unwrap();
    assert_eq!(paid.pending_choice.as_ref().unwrap().kind, "trigger");
    assert!(paid.assets.iter().all(|c| c.owner != "p0" || c.exhausted));
    assert_eq!(paid.stack.len(), 1);
    let first = store.replay(&a.room_id).unwrap();
    assert!(first
        .regions
        .iter()
        .all(|r| r.cards.iter().all(|c| c.id != source_id)));
    assert_eq!(first.players[1].graveyard.len(), 1);
    assert_eq!(first.stack[0].frame.as_ref().unwrap().already_paid.len(), 2);
    drop(store);
    let store = Store::open(&db).unwrap();
    let retry = store.command(&a.room_id, &a.token, cast).await.unwrap();
    assert_eq!(
        serde_json::to_string(&retry).unwrap(),
        serde_json::to_string(&paid).unwrap()
    );
    assert_eq!(store.state(&a.room_id, &a.token).await.unwrap().version, 2);
    let choice = paid.pending_choice.unwrap();
    store
        .command(
            &a.room_id,
            &a.token,
            Command {
                command_id: "death-target".into(),
                expected_version: 2,
                action: Action {
                    choice_id: Some(choice.id),
                    selected: Some(vec!["p1".into()]),
                    ..Action::new("choose")
                },
            },
        )
        .await
        .unwrap();
    store
        .command(&a.room_id, &a.token, command("pass-a", 3, "pass"))
        .await
        .unwrap();
    let middle = store
        .command(&a.room_id, &b.token, command("pass-b", 4, "pass"))
        .await
        .unwrap();
    let private = middle.pending_choice.as_ref().unwrap();
    assert_eq!(private.kind, "discard");
    let frame = store
        .replay(&a.room_id)
        .unwrap()
        .pending
        .unwrap()
        .resolution;
    let ChoiceResolution::Frame {
        frame,
        choice: FrameChoice::Discard { .. },
    } = frame
    else {
        panic!("resumable accepted frame required")
    };
    assert!(matches!(frame.guard, GuardState::Accepted));
    assert_eq!(frame.cursor, 1);
    drop(store);
    let store = Store::open(&db).unwrap();
    assert_eq!(
        serde_json::to_string(&store.state(&a.room_id, &b.token).await.unwrap()).unwrap(),
        serde_json::to_string(&middle).unwrap()
    );
    let discard = Command {
        command_id: "discard-once".into(),
        expected_version: 5,
        action: Action {
            choice_id: Some(private.id.clone()),
            selected: Some(vec![private.options[0].id.clone()]),
            ..Action::new("choose")
        },
    };
    store
        .command(&a.room_id, &b.token, discard.clone())
        .await
        .unwrap();
    store
        .command(&a.room_id, &a.token, command("pass-a2", 6, "pass"))
        .await
        .unwrap();
    let complete = store
        .command(&a.room_id, &b.token, command("pass-b2", 7, "pass"))
        .await
        .unwrap();
    assert!(complete.stack.is_empty() && complete.pending_choice.is_none());
    assert_eq!(complete.players[0].hand_count, 2);
    assert_eq!(complete.players[1].hand_count, 0);
    let final_game = store.replay(&a.room_id).unwrap();
    assert_eq!(final_game.players[1].graveyard.len(), 2);
    assert_eq!(final_game.players[0].graveyard.len(), 1);
    assert!(final_game.players[0].assets.iter().all(|c| c.exhausted));
    store.command(&a.room_id, &b.token, discard).await.unwrap();
    assert_eq!(store.state(&a.room_id, &a.token).await.unwrap().version, 8);
    assert!(
        Store::open_read_only(&db)
            .unwrap()
            .audit_replay(&a.room_id)
            .unwrap()
            .matches
    );
}
#[tokio::test]
async fn detective_reveal_choice_and_bound_frame_restore_without_repayment_or_private_leaks() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("detective.sqlite3");
    let store = Store::open(&db).unwrap();
    let a = store
        .create(CreateRoom {
            name: "Detective".into(),
            mode: "duel".into(),
            deck_id: "keepers".into(),
        })
        .await
        .unwrap();
    let b = store
        .join(JoinRoom {
            invite_code: a.invite_code.clone(),
            name: "Hidden owner".into(),
            deck_id: "watchers".into(),
        })
        .await
        .unwrap();
    drop(store);
    // Explicit initial layout fixture, installed before the first command. All
    // transitions below use authenticated versioned commands, never midgame SQL.
    let connection = rusqlite::Connection::open(&db).unwrap();
    let initial: String = connection
        .query_row("SELECT state FROM rooms WHERE id=?1", [&a.room_id], |r| {
            r.get(0)
        })
        .unwrap();
    let mut game = Game::from_persisted(&initial).unwrap();
    for player in &mut game.players {
        player.ready = true;
    }
    game.apply(0, Action::new("start")).unwrap();
    while let Some(pending) = game.pending.clone() {
        game.apply(
            pending.seat,
            Action {
                choice_id: Some(pending.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    game.window = Some(Window::Action(0));
    game.first_team = 0;
    game.active_team = 0;
    game.priority_team = 0;
    game.version = 1;
    game.log.clear();
    for player in &mut game.players {
        player.hand.clear();
    }
    let mut detective = game.make_card("JC058", 0);
    let detective_id = detective.id.clone();
    detective.face_down = true;
    detective.exhausted = true;
    game.regions[0].cards.push(detective);
    let mut target = game.make_card("XQ12", 1);
    let target_id = target.id.clone();
    target.face_down = true;
    game.regions[0].cards.push(target);
    for _ in 0..3 {
        let asset = game.make_card("JC058", 0);
        game.players[0].assets.push(asset);
    }
    let serialized = serde_json::to_string(&game).unwrap();
    connection
        .execute(
            "UPDATE rooms SET state=?2,initial_state=?2 WHERE id=?1",
            rusqlite::params![a.room_id, serialized],
        )
        .unwrap();
    connection
        .execute("DELETE FROM journal WHERE room_id=?1", [&a.room_id])
        .unwrap();
    drop(connection);

    let store = Store::open(&db).unwrap();
    let reveal = Command {
        command_id: "detective-reveal".into(),
        expected_version: 1,
        action: Action {
            card_id: Some(detective_id),
            ..Action::new("reveal")
        },
    };
    let paid = store
        .command(&a.room_id, &a.token, reveal.clone())
        .await
        .unwrap();
    assert_eq!(paid.stack.len(), 1);
    assert!(paid.assets.iter().all(|c| c.exhausted));
    store
        .command(&a.room_id, &a.token, command("reveal-pass-a", 2, "pass"))
        .await
        .unwrap();
    store
        .command(&a.room_id, &b.token, command("reveal-pass-b", 3, "pass"))
        .await
        .unwrap();
    let choosing = store.state(&a.room_id, &a.token).await.unwrap();
    let choice = choosing.pending_choice.as_ref().unwrap();
    assert_eq!(choice.kind, "trigger");
    assert_eq!(choice.options.len(), 1);
    assert_eq!(choice.options[0].id, target_id);
    let hidden = choice.options[0].card.as_ref().unwrap();
    assert_eq!(hidden.name, "暗藏者");
    assert!(hidden.card_id.is_none() && hidden.text.is_none() && hidden.cost.is_none());
    assert!(choosing.stack.is_empty());
    let other = store.state(&a.room_id, &b.token).await.unwrap();
    assert!(other.pending_choice.is_none());
    assert_eq!(other.waiting_choice.as_ref().unwrap().player_id, "p0");
    let choose = Command {
        command_id: "detective-target".into(),
        expected_version: 4,
        action: Action {
            choice_id: Some(choice.id.clone()),
            selected: Some(vec![target_id.clone()]),
            ..Action::new("choose")
        },
    };
    drop(store);

    let store = Store::open(&db).unwrap();
    assert_eq!(
        serde_json::to_string(&store.state(&a.room_id, &a.token).await.unwrap()).unwrap(),
        serde_json::to_string(&choosing).unwrap()
    );
    let retry = store.command(&a.room_id, &a.token, reveal).await.unwrap();
    assert_eq!(
        serde_json::to_string(&retry).unwrap(),
        serde_json::to_string(&paid).unwrap()
    );
    assert_eq!(store.state(&a.room_id, &a.token).await.unwrap().version, 4);
    let wrong_seat = Command {
        command_id: "wrong-chooser".into(),
        ..choose.clone()
    };
    assert_eq!(
        store
            .command(&a.room_id, &b.token, wrong_seat)
            .await
            .unwrap_err()
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(store.state(&a.room_id, &a.token).await.unwrap().version, 4);
    let bound = store
        .command(&a.room_id, &a.token, choose.clone())
        .await
        .unwrap();
    assert_eq!(bound.stack.len(), 1);
    assert_eq!(
        bound.stack[0].target_summaries[0].label,
        "暗藏者·Hidden owner"
    );
    assert!(bound.stack[0].target_summaries[0].valid);
    let replay = store.replay(&a.room_id).unwrap();
    let frame = replay.stack[0].frame.as_ref().unwrap();
    assert!(matches!(frame.guard, GuardState::Unchecked));
    assert_eq!(frame.cursor, 0);
    assert!(frame.already_paid.is_empty()); // The trigger has no extra fee.
    assert!(replay.players[0].assets.iter().all(|c| c.exhausted));
    drop(store);

    let store = Store::open(&db).unwrap();
    assert_eq!(
        serde_json::to_string(&store.state(&a.room_id, &a.token).await.unwrap()).unwrap(),
        serde_json::to_string(&bound).unwrap()
    );
    let retry = store.command(&a.room_id, &a.token, choose).await.unwrap();
    assert_eq!(
        serde_json::to_string(&retry).unwrap(),
        serde_json::to_string(&bound).unwrap()
    );
    assert_eq!(store.state(&a.room_id, &a.token).await.unwrap().version, 5);
    store
        .command(&a.room_id, &a.token, command("destroy-pass-a", 5, "pass"))
        .await
        .unwrap();
    let complete = store
        .command(&a.room_id, &b.token, command("destroy-pass-b", 6, "pass"))
        .await
        .unwrap();
    assert!(
        complete.stack.is_empty()
            && complete.pending_choice.is_none()
            && complete.waiting_choice.is_none()
    );
    let final_game = store.replay(&a.room_id).unwrap();
    assert_eq!(final_game.players[1].graveyard.len(), 1);
    assert_eq!(final_game.players[1].graveyard[0].definition, "XQ12");
    assert_ne!(final_game.players[1].graveyard[0].id, target_id);
    assert!(final_game.players[0].assets.iter().all(|c| c.exhausted));
    let detective = final_game.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC058")
        .unwrap();
    assert!(detective.exhausted && !detective.face_down);
    assert!(
        Store::open_read_only(&db)
            .unwrap()
            .audit_replay(&a.room_id)
            .unwrap()
            .matches
    );
}
#[tokio::test]
async fn legacy_state_is_rejected_explicitly_without_silent_migration() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("legacy.sqlite3");
    let store = Store::open(&db).unwrap();
    let (a, _) = pair(&store).await;
    drop(store);
    let connection = rusqlite::Connection::open(&db).unwrap();
    let stored: String = connection
        .query_row("SELECT state FROM rooms WHERE id=?1", [&a.room_id], |r| {
            r.get(0)
        })
        .unwrap();
    drop(connection);
    for (engine, pool, schema) in [
        ("rust-v0.1.0", "limited-v1", None),
        ("rust-v0.2.0", "limited-v2", Some(2)),
        ("rust-v0.2.1", "limited-v2.1", Some(2)),
        ("rust-v0.2.2", "limited-v2.1", Some(2)),
    ] {
        let mut legacy: serde_json::Value = serde_json::from_str(&stored).unwrap();
        if let Some(schema) = schema {
            legacy["state_schema"] = schema.into();
        } else {
            legacy.as_object_mut().unwrap().remove("state_schema");
        }
        legacy["versions"]["engine"] = engine.into();
        legacy["versions"]["cardPool"] = pool.into();
        let legacy = serde_json::to_string(&legacy).unwrap();
        let connection = rusqlite::Connection::open(&db).unwrap();
        connection
            .execute(
                "UPDATE rooms SET state=?2,initial_state=?2 WHERE id=?1",
                rusqlite::params![a.room_id, legacy],
            )
            .unwrap();
        drop(connection);
        let error = match Store::open(&db) {
            Err(e) => e,
            Ok(_) => panic!("legacy room must not load: {engine}"),
        };
        assert!(error.message.contains("旧局") && error.message.contains("静默迁移"));
        assert!(error
            .message
            .contains(hegemony_server::catalog::ENGINE_VERSION));
        let audit = Store::open_read_only(&db).unwrap();
        assert!(audit
            .audit_replay(&a.room_id)
            .unwrap_err()
            .message
            .contains("旧局"));
        let connection = rusqlite::Connection::open(&db).unwrap();
        let after: String = connection
            .query_row("SELECT state FROM rooms WHERE id=?1", [&a.room_id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(after, legacy);
    }
}
async fn pair(store: &Store) -> (service::Session, service::Session) {
    let a = store
        .create(CreateRoom {
            name: "Alice".into(),
            mode: "duel".into(),
            deck_id: "watchers".into(),
        })
        .await
        .unwrap();
    let b = store
        .join(JoinRoom {
            invite_code: a.invite_code.clone(),
            name: "Bob".into(),
            deck_id: "hunters".into(),
        })
        .await
        .unwrap();
    (a, b)
}
#[tokio::test]
async fn dedupe_conflicts_rejections_auth_and_replay_are_strict() {
    let store = Store::open(":memory:").unwrap();
    let (a, b) = pair(&store).await;
    assert!(store.state(&a.room_id, &b.token).await.is_ok());
    assert_eq!(
        store.state(&a.room_id, "wrong").await.unwrap_err().status,
        StatusCode::UNAUTHORIZED
    );
    let c = command("ready-a", 1, "ready");
    let ready = store
        .command(&a.room_id, &a.token, c.clone())
        .await
        .unwrap();
    assert_eq!(ready.version, 2);
    let ready_b = store
        .command(&a.room_id, &b.token, command("ready-b", 2, "ready"))
        .await
        .unwrap();
    assert_eq!(ready_b.version, 3);
    let retry = store.command(&a.room_id, &a.token, c).await.unwrap();
    assert_eq!(
        serde_json::to_string(&retry).unwrap(),
        serde_json::to_string(&ready).unwrap()
    );
    assert_eq!(store.state(&a.room_id, &a.token).await.unwrap().version, 3);
    assert!(store
        .command(&a.room_id, &b.token, command("ready-a", 3, "ready"))
        .await
        .is_err());
    let err = store
        .command(&a.room_id, &a.token, command("stale", 2, "start"))
        .await
        .unwrap_err();
    assert_eq!(err.status, StatusCode::CONFLICT);
    assert_eq!(err.view.unwrap().version, 3);
    assert!(store
        .command(&a.room_id, &b.token, command("not-host", 3, "start"))
        .await
        .is_err());
    assert_eq!(store.state(&a.room_id, &a.token).await.unwrap().version, 3);
    let start = store
        .command(&a.room_id, &a.token, command("start", 3, "start"))
        .await
        .unwrap();
    assert_eq!(start.version, 4);
    assert_eq!(start.pending_choice.as_ref().unwrap().kind, "mulligan");
    assert!(store
        .state(&a.room_id, &b.token)
        .await
        .unwrap()
        .pending_choice
        .is_none());
    let replay = store.replay(&a.room_id).unwrap();
    assert_eq!(
        serde_json::to_string(&replay.view(0)).unwrap(),
        serde_json::to_string(&start).unwrap()
    );
    let serialized = serde_json::to_string(&start).unwrap();
    assert!(!serialized.contains(&a.token) && !serialized.contains(&b.token));
}
#[tokio::test]
async fn sqlite_reopen_restores_identity_pending_choice_and_dedupe() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("room.sqlite3");
    let store = Store::open(&db).unwrap();
    let (a, b) = pair(&store).await;
    store
        .command(&a.room_id, &a.token, command("r-a", 1, "ready"))
        .await
        .unwrap();
    store
        .command(&a.room_id, &b.token, command("r-b", 2, "ready"))
        .await
        .unwrap();
    let original = store
        .command(&a.room_id, &a.token, command("s", 3, "start"))
        .await
        .unwrap();
    drop(store);
    let recovered = Store::open(&db).unwrap();
    let state = recovered.state(&a.room_id, &a.token).await.unwrap();
    assert_eq!(
        serde_json::to_string(&state).unwrap(),
        serde_json::to_string(&original).unwrap()
    );
    assert_eq!(state.hand.len(), 6);
    assert_eq!(state.players[0].deck_count, 44);
    let p = state.pending_choice.unwrap();
    let choice = Command {
        command_id: "mulligan".into(),
        expected_version: 4,
        action: Action {
            kind: "choose".into(),
            choice_id: Some(p.id),
            selected: Some(vec![p.options[0].id.clone()]),
            ..Action::default()
        },
    };
    let changed = recovered
        .command(&a.room_id, &a.token, choice.clone())
        .await
        .unwrap();
    assert_eq!(changed.version, 5);
    assert_ne!(changed.hand[0].instance_id, original.hand[0].instance_id);
    drop(recovered);
    let again = Store::open(&db).unwrap();
    let retry = again.command(&a.room_id, &a.token, choice).await.unwrap();
    assert_eq!(
        serde_json::to_string(&retry).unwrap(),
        serde_json::to_string(&changed).unwrap()
    );
    assert_eq!(again.replay(&a.room_id).unwrap().version, 5);
}
#[tokio::test]
async fn failed_transaction_does_not_ack_or_mutate_state() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("room.sqlite3");
    let store = Store::open(&db).unwrap();
    let (a, _) = pair(&store).await;
    let before = store.state(&a.room_id, &a.token).await.unwrap();
    let external = rusqlite::Connection::open(&db).unwrap();
    external.execute_batch("CREATE TRIGGER deny_command BEFORE INSERT ON commands BEGIN SELECT RAISE(ABORT,'simulated durable-write failure'); END;").unwrap();
    assert_eq!(
        store
            .command(&a.room_id, &a.token, command("fail", 1, "ready"))
            .await
            .unwrap_err()
            .status,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    let after = store.state(&a.room_id, &a.token).await.unwrap();
    assert_eq!(
        serde_json::to_string(&before).unwrap(),
        serde_json::to_string(&after).unwrap()
    );
    drop(store);
    let recovered = Store::open(&db).unwrap();
    assert_eq!(
        recovered.state(&a.room_id, &a.token).await.unwrap().version,
        1
    );
}
#[tokio::test]
async fn room_catalog_requires_a_token_for_that_room_and_returns_current_pool() {
    let store = Store::open(":memory:").unwrap();
    let (a, b) = pair(&store).await;
    let foreign = store
        .create(CreateRoom {
            name: "Other room".into(),
            mode: "duel".into(),
            deck_id: "keepers".into(),
        })
        .await
        .unwrap();
    let app = service::router(store.clone());
    let endpoint = format!("/api/rooms/{}/catalog", a.room_id);
    for token in [None, Some("wrong"), Some(foreign.token.as_str())] {
        let mut request = Request::builder().uri(&endpoint);
        if let Some(token) = token {
            request = request.header("Authorization", format!("Bearer {token}"));
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
    for token in [&a.token, &b.token] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&endpoint)
                    .header("Authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let catalog: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            catalog["engineVersion"],
            hegemony_server::catalog::ENGINE_VERSION
        );
        assert_eq!(
            catalog["cardPoolVersion"],
            hegemony_server::catalog::POOL_VERSION
        );
        assert!(catalog["cards"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["id"] == "JC058"));
        assert!(!String::from_utf8(bytes.to_vec()).unwrap().contains(token));
    }
    assert_eq!(store.state(&a.room_id, &a.token).await.unwrap().version, 1);
}
#[tokio::test]
async fn authenticated_sse_projects_actor_view_and_updates_after_commit() {
    let store = Store::open(":memory:").unwrap();
    let (a, b) = pair(&store).await;
    let app = service::router(store.clone());
    let unauth = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/rooms/{}/events", a.room_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauth.status(), StatusCode::UNAUTHORIZED);
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/rooms/{}/events", a.room_id))
                .header("Authorization", format!("Bearer {}", b.token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    let mut body = response.into_body();
    let initial = tokio::time::timeout(std::time::Duration::from_secs(1), body.frame())
        .await
        .unwrap()
        .unwrap()
        .unwrap()
        .into_data()
        .unwrap();
    let text = String::from_utf8(initial.to_vec()).unwrap();
    assert!(text.contains("\"you\":\"p1\""));
    assert!(text.contains("id: 1"));
    store
        .command(&a.room_id, &a.token, command("change", 1, "ready"))
        .await
        .unwrap();
    let update = tokio::time::timeout(std::time::Duration::from_secs(1), body.frame())
        .await
        .unwrap()
        .unwrap()
        .unwrap()
        .into_data()
        .unwrap();
    let text = String::from_utf8(update.to_vec()).unwrap();
    assert!(text.contains("id: 2"));
    assert!(text.contains("\"ready\":true"));
    assert!(!text.contains(&a.token) && !text.contains(&b.token));
}

#[tokio::test]
async fn read_only_audit_cli_matches_seed_journal_and_detects_corruption() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("audit.sqlite3");
    let store = Store::open(&db).unwrap();
    let (a, b) = pair(&store).await;
    store
        .command(&a.room_id, &a.token, command("ready-a", 1, "ready"))
        .await
        .unwrap();
    store
        .command(&a.room_id, &b.token, command("ready-b", 2, "ready"))
        .await
        .unwrap();
    store
        .command(&a.room_id, &a.token, command("start", 3, "start"))
        .await
        .unwrap();
    let audit = Store::open_read_only(&db).unwrap();
    let report = audit.audit_replay(&a.room_id).unwrap();
    assert!(report.matches);
    assert_eq!(report.version, 4);
    assert_eq!(report.journal_entries, 4);
    assert_eq!(report.persisted_digest, report.replayed_digest);
    let public = serde_json::to_string(&report).unwrap();
    assert!(
        !public.contains(&a.token)
            && !public.contains(&b.token)
            && !public.contains("Alice")
            && !public.contains("JC125")
    );
    assert!(audit
        .create(CreateRoom {
            name: "read-only".into(),
            mode: "duel".into(),
            deck_id: "watchers".into()
        })
        .await
        .is_err());
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_hegemony-audit"))
        .arg(&db)
        .arg(&a.room_id)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .contains("\"matches\":true"));
    let mut altered = store.replay(&a.room_id).unwrap();
    altered.turn += 1;
    let db_conn = rusqlite::Connection::open(&db).unwrap();
    db_conn
        .execute(
            "UPDATE rooms SET state=?2 WHERE id=?1",
            rusqlite::params![a.room_id, serde_json::to_string(&altered).unwrap()],
        )
        .unwrap();
    let mismatch = audit.audit_replay(&a.room_id).unwrap();
    assert!(!mismatch.matches);
    assert_ne!(mismatch.persisted_digest, mismatch.replayed_digest);
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_hegemony-audit"))
        .arg(&db)
        .arg(&a.room_id)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let missing = dir.path().join("must-not-create.sqlite3");
    assert!(Store::open_read_only(&missing).is_err());
    assert!(!missing.exists());
}
