#![cfg(feature = "native")]
use hegemony_server::{
    model::{Action, Window},
    room::{QuoteRequest, RoomCommand, RoomEnvelope, SessionAction},
    service::{CreateRoom, JoinRoom, Session, Store},
};
use rusqlite::params;

async fn setup(path: &std::path::Path, mode: &str) -> (Store, Vec<Session>, String, String) {
    let store = Store::open(path).unwrap();
    let host = store
        .create(CreateRoom {
            name: "P0".into(),
            mode: mode.into(),
            deck_id: "responders".into(),
            deck_draft: None,
        })
        .await
        .unwrap();
    let mut seats = vec![host];
    let capacity = if mode == "teams" { 4 } else { 2 };
    for seat in 1..capacity {
        seats.push(
            store
                .join(JoinRoom {
                    invite_code: seats[0].invite_code.clone(),
                    name: format!("P{seat}"),
                    deck_id: "responders".into(),
                    deck_draft: None,
                })
                .await
                .unwrap(),
        );
    }
    drop(store);
    let db = rusqlite::Connection::open(path).unwrap();
    let state: String = db
        .query_row(
            "SELECT state FROM rooms WHERE id=?1",
            [&seats[0].room_id],
            |r| r.get(0),
        )
        .unwrap();
    let mut game = RoomEnvelope::from_persisted(&state).unwrap().game;
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
    // One explicit initial fixture, before any accepted command; credentials are retained.
    game.window = Some(Window::Action(0));
    game.first_team = 0;
    game.priority_team = 0;
    game.active_team = 0;
    for p in &mut game.players {
        p.hand.clear();
        p.assets.clear();
    }
    for seat in 0..capacity {
        let source = game.make_card("JC042", seat);
        game.regions[0].cards.push(source);
        for _ in 0..3 {
            let a = game.make_card("JC091", seat);
            game.players[seat].assets.push(a);
        }
    }
    let enemy = if mode == "teams" { 2 } else { 1 };
    let target = game.regions[0]
        .cards
        .iter()
        .find(|c| c.controller == enemy)
        .unwrap()
        .id
        .clone();
    let murder = game.make_card("JC091", 0);
    let murder_id = murder.id.clone();
    game.players[0].hand.push(murder);
    game.version = (capacity - 1) as u64;
    game.log.clear();
    let initial = serde_json::to_string(&RoomEnvelope::from_game(game)).unwrap();
    db.execute(
        "UPDATE rooms SET state=?2,initial_state=?2,revision=?3 WHERE id=?1",
        params![seats[0].room_id, initial, (capacity - 1) as u64],
    )
    .unwrap();
    db.execute("DELETE FROM journal WHERE room_id=?1", [&seats[0].room_id])
        .unwrap();
    drop(db);
    (Store::open(path).unwrap(), seats, murder_id, target)
}
fn command(id: &str, version: u64, action: SessionAction) -> RoomCommand {
    RoomCommand {
        command_id: id.into(),
        expected_version: version,
        action,
    }
}
async fn start_murder(
    store: &Store,
    seats: &[Session],
    id: String,
    target: String,
) -> hegemony_server::room::RoomView {
    store
        .command_at_now(
            &seats[0].room_id,
            &seats[0].token,
            command(
                "murder",
                (seats.len() - 1) as u64,
                Action {
                    card_id: Some(id),
                    target_id: Some(target),
                    ..Action::new("play")
                }
                .into(),
            ),
            1_000,
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn paid_response_reopen_quote_receipts_and_target_failure_keep_atomic_costs() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("response.sqlite3");
    let (store, seats, murder, target) = setup(&path, "duel").await;
    let first = start_murder(&store, &seats, murder, target.clone()).await;
    let pass = command(
        "pass0",
        first.version,
        SessionAction::PassResponse {
            window_id: first.response_window.clone().unwrap().id,
        },
    );
    let holder = store
        .command_at_now(&seats[0].room_id, &seats[0].token, pass, 1_001)
        .await
        .unwrap();
    let window = holder.response_window.clone().unwrap().id;
    let begin = command(
        "begin1",
        holder.version,
        SessionAction::BeginResponse {
            window_id: window.clone(),
            intent_id: "draft1".into(),
        },
    );
    let begun = store
        .command_at_now(&seats[0].room_id, &seats[1].token, begin.clone(), 1_002)
        .await
        .unwrap();
    let action = Action {
        card_id: Some(target),
        ..Action::new("activate")
    };
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(
        serde_json::to_string(
            &store
                .state_at_now(&seats[0].room_id, &seats[1].token, 0)
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_string(&begun).unwrap()
    );
    let quote = store
        .quote(
            &seats[0].room_id,
            &seats[1].token,
            QuoteRequest {
                window_id: window.clone(),
                intent_id: "draft1".into(),
                draft: Some(action.clone()),
            },
        )
        .await
        .unwrap();
    assert!(quote.ready);
    let before = serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap();
    let invalid = command(
        "invalid-submit",
        begun.version,
        SessionAction::SubmitResponse {
            window_id: window.clone(),
            intent_id: "draft1".into(),
            action: Action::new("activate"),
        },
    );
    assert_eq!(
        store
            .command_at_now(&seats[0].room_id, &seats[1].token, invalid, 1_003)
            .await
            .unwrap_err()
            .error,
        "invalid_action"
    );
    assert_eq!(
        serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap(),
        before
    );
    let submit = command(
        "submit1",
        begun.version,
        SessionAction::SubmitResponse {
            window_id: window.clone(),
            intent_id: "draft1".into(),
            action,
        },
    );
    let paid = store
        .command_at_now(&seats[0].room_id, &seats[1].token, submit.clone(), 1_003)
        .await
        .unwrap();
    assert_eq!(paid.stack.len(), 1);
    assert_eq!(paid.stack[0].target_summaries[0].status, "missing");
    let room = store.replay(&seats[0].room_id).unwrap();
    assert_eq!(room.game.modifiers.len(), 1);
    assert_eq!(room.game.players[1].graveyard.len(), 1);
    assert!(room.game.players[0].assets.iter().all(|a| a.exhausted));
    let cancel = command(
        "too-late-to-cancel",
        paid.version,
        SessionAction::CancelAndPass {
            window_id: window,
            intent_id: "draft1".into(),
        },
    );
    assert_eq!(
        store
            .command_at_now(&seats[0].room_id, &seats[1].token, cancel, 1_004)
            .await
            .unwrap_err()
            .error,
        "window_expired"
    );
    let mut view = paid.clone();
    for (n, seat) in [1, 0].into_iter().enumerate() {
        view = store
            .command_at_now(
                &seats[0].room_id,
                &seats[seat].token,
                command(
                    &format!("final-pass{seat}"),
                    view.version,
                    SessionAction::PassResponse {
                        window_id: view.response_window.as_ref().unwrap().id.clone(),
                    },
                ),
                1_004 + n as u64,
            )
            .await
            .unwrap();
    }
    assert!(view.stack.is_empty());
    assert!(view.log.iter().any(|l| l.text.contains("取消")));
    let final_state = serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap();
    drop(store);
    let store = Store::open(&path).unwrap();
    // Both old receipts precede the huge clock input: no expiry or repeated sacrifice.
    for (cmd, expected) in [(begin, begun), (submit.clone(), paid)] {
        assert_eq!(
            serde_json::to_string(
                &store
                    .command_at_now(&seats[0].room_id, &seats[1].token, cmd, 999_999)
                    .await
                    .unwrap()
            )
            .unwrap(),
            serde_json::to_string(&expected).unwrap()
        );
    }
    assert_eq!(
        serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap(),
        final_state
    );
    // Window/intent membership is part of the original receipt, even long after its deadline.
    for changed_field in ["intent", "window", "expected"] {
        let mut altered = submit.clone();
        match &mut altered.action {
            SessionAction::SubmitResponse {
                window_id,
                intent_id,
                ..
            } => {
                if changed_field == "intent" {
                    *intent_id = "other-intent".into();
                }
                if changed_field == "window" {
                    *window_id = "other-window".into();
                }
            }
            _ => unreachable!(),
        }
        if changed_field == "expected" {
            altered.expected_version += 1;
        }
        assert_eq!(
            store
                .command_at_now(&seats[0].room_id, &seats[1].token, altered, 999_999)
                .await
                .unwrap_err()
                .error,
            "command_id_conflict"
        );
    }
    assert_eq!(
        serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap(),
        final_state
    );
    assert!(store.audit_replay(&seats[0].room_id).unwrap().matches);
}

#[tokio::test]
async fn expired_rejected_begin_commits_only_system_journal_and_poll_stops_at_empty_stack() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("expiry.sqlite3");
    let (store, seats, murder, target) = setup(&path, "duel").await;
    let first = start_murder(&store, &seats, murder, target).await;
    let late = command(
        "late",
        first.version,
        SessionAction::BeginResponse {
            window_id: first.response_window.clone().unwrap().id,
            intent_id: "late".into(),
        },
    );
    let error = store
        .command_at_now(&seats[0].room_id, &seats[0].token, late, 6_000)
        .await
        .unwrap_err();
    assert_eq!(error.error, "window_expired");
    assert_eq!(error.view.unwrap().version, first.version + 1);
    let db = rusqlite::Connection::open(&path).unwrap();
    let receipts: u64 = db
        .query_row(
            "SELECT COUNT(*) FROM commands WHERE room_id=?1 AND command_id='late'",
            [&seats[0].room_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(receipts, 0);
    drop(db);
    drop(store);
    let store = Store::open(&path).unwrap();
    let completed = store
        .state_at_now(&seats[0].room_id, &seats[0].token, 30_000)
        .await
        .unwrap();
    assert!(completed.stack.is_empty());
    assert!(completed.response_window.is_none());
    let version = completed.version;
    let no_cascade = store
        .state_at_now(&seats[0].room_id, &seats[0].token, 9_999_999)
        .await
        .unwrap();
    assert_eq!(no_cascade.version, version);
    assert_eq!(no_cascade.step, completed.step);
    assert_eq!(no_cascade.server_now_ms, 9_999_999);
    assert!(store.audit_replay(&seats[0].room_id).unwrap().matches);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn begin_and_timeout_race_has_one_committed_order_and_replays() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("race.sqlite3");
    let (store, seats, murder, target) = setup(&path, "duel").await;
    let first = start_murder(&store, &seats, murder, target).await;
    let second = Store::open(&path).unwrap();
    let cmd = command(
        "racing-begin",
        first.version,
        SessionAction::BeginResponse {
            window_id: first.response_window.clone().unwrap().id,
            intent_id: "race".into(),
        },
    );
    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(2));
    let b = barrier.clone();
    let room = seats[0].room_id.clone();
    let token = seats[0].token.clone();
    let s = store.clone();
    let begin = tokio::spawn(async move {
        b.wait().await;
        s.command_at_now(&room, &token, cmd, 5_999).await
    });
    let b = barrier;
    let room = seats[0].room_id.clone();
    let token = seats[0].token.clone();
    let tick = tokio::spawn(async move {
        b.wait().await;
        second.state_at_now(&room, &token, 6_000).await
    });
    let result = begin.await.unwrap();
    tick.await.unwrap().unwrap();
    let final_room = store.replay(&seats[0].room_id).unwrap();
    assert_eq!(final_room.revision, first.version + 1);
    if result.is_ok() {
        assert!(matches!(
            final_room.pacing.window.as_ref().unwrap().members[&0],
            hegemony_server::room::Decision::Composing { .. }
        ));
    } else {
        assert_eq!(result.unwrap_err().error, "window_expired");
        assert_eq!(final_room.game.priority_team, 1);
    }
    assert!(store.audit_replay(&seats[0].room_id).unwrap().matches);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_join_cas_assigns_distinct_seats_and_replay_contains_both() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("join.sqlite3");
    let store = Store::open(&path).unwrap();
    let host = store
        .create(CreateRoom {
            name: "Host".into(),
            mode: "teams".into(),
            deck_id: "watchers".into(),
            deck_draft: None,
        })
        .await
        .unwrap();
    let second = Store::open(&path).unwrap();
    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(2));
    let b = barrier.clone();
    let invite = host.invite_code.clone();
    let s = store.clone();
    let one = tokio::spawn(async move {
        b.wait().await;
        s.join(JoinRoom {
            invite_code: invite,
            name: "One".into(),
            deck_id: "watchers".into(),
            deck_draft: None,
        })
        .await
    });
    let two = tokio::spawn(async move {
        barrier.wait().await;
        second
            .join(JoinRoom {
                invite_code: host.invite_code,
                name: "Two".into(),
                deck_id: "watchers".into(),
                deck_draft: None,
            })
            .await
    });
    let a = one.await.unwrap().unwrap();
    let b = two.await.unwrap().unwrap();
    assert_ne!(a.seat, b.seat);
    assert_ne!(a.token, b.token);
    assert_eq!(store.replay(&a.room_id).unwrap().game.players.len(), 3);
    assert!(store.audit_replay(&a.room_id).unwrap().matches);
}

#[tokio::test]
async fn failed_pacing_transaction_preserves_deadline_and_has_no_receipt_or_ack() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("failure.sqlite3");
    let (store, seats, murder, target) = setup(&path, "duel").await;
    let first = start_murder(&store, &seats, murder, target).await;
    let before = serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap();
    let cmd = command(
        "failed-begin",
        first.version,
        SessionAction::BeginResponse {
            window_id: first.response_window.clone().unwrap().id,
            intent_id: "draft".into(),
        },
    );
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute_batch("CREATE TRIGGER fail_session BEFORE INSERT ON journal BEGIN SELECT RAISE(ABORT,'fixture'); END;").unwrap();
    assert_eq!(
        store
            .command_at_now(&seats[0].room_id, &seats[0].token, cmd.clone(), 2_000)
            .await
            .unwrap_err()
            .error,
        "storage_error"
    );
    assert_eq!(
        serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap(),
        before
    );
    let receipts: u64 = db
        .query_row(
            "SELECT COUNT(*) FROM commands WHERE room_id=?1 AND command_id='failed-begin'",
            [&seats[0].room_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(receipts, 0);
    db.execute_batch("DROP TRIGGER fail_session").unwrap();
    drop(db);
    let accepted = store
        .command_at_now(&seats[0].room_id, &seats[0].token, cmd, 2_001)
        .await
        .unwrap();
    assert_eq!(accepted.version, first.version + 1);
    assert!(store.audit_replay(&seats[0].room_id).unwrap().matches);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn teammate_begins_from_one_revision_both_commit_without_rebasing_paid_actions() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("teammates.sqlite3");
    let (store, seats, murder, target) = setup(&path, "teams").await;
    let first = start_murder(&store, &seats, murder, target).await;
    let second = Store::open(&path).unwrap();
    let window = first.response_window.clone().unwrap().id;
    let a = command(
        "parallel-begin0",
        first.version,
        SessionAction::BeginResponse {
            window_id: window.clone(),
            intent_id: "draft0".into(),
        },
    );
    let b = command(
        "parallel-begin1",
        first.version,
        SessionAction::BeginResponse {
            window_id: window,
            intent_id: "draft1".into(),
        },
    );
    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(2));
    let gate = barrier.clone();
    let room = seats[0].room_id.clone();
    let token = seats[0].token.clone();
    let s = store.clone();
    let one = tokio::spawn(async move {
        gate.wait().await;
        s.command_at_now(&room, &token, a, 1_001).await
    });
    let room = seats[0].room_id.clone();
    let token = seats[1].token.clone();
    let two = tokio::spawn(async move {
        barrier.wait().await;
        second.command_at_now(&room, &token, b, 1_001).await
    });
    one.await.unwrap().unwrap();
    two.await.unwrap().unwrap();
    let final_room = store.replay(&seats[0].room_id).unwrap();
    assert_eq!(final_room.revision, first.version + 2);
    let members = &final_room.pacing.window.as_ref().unwrap().members;
    for seat in [0, 1] {
        assert!(matches!(
            members[&seat],
            hegemony_server::room::Decision::Composing { .. }
        ));
    }
    let source = final_room.regions[0]
        .cards
        .iter()
        .find(|c| c.controller == 0)
        .unwrap()
        .id
        .clone();
    let before = serde_json::to_string(&final_room).unwrap();
    let stale = command(
        "stale-formal",
        first.version + 1,
        SessionAction::SubmitResponse {
            window_id: final_room.pacing.window.as_ref().unwrap().id.clone(),
            intent_id: "draft0".into(),
            action: Action {
                card_id: Some(source),
                ..Action::new("activate")
            },
        },
    );
    assert_eq!(
        store
            .command_at_now(&seats[0].room_id, &seats[0].token, stale, 1_002)
            .await
            .unwrap_err()
            .error,
        "version_conflict"
    );
    assert_eq!(
        serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap(),
        before
    );
    assert!(store.audit_replay(&seats[0].room_id).unwrap().matches);
}
