#![cfg(feature = "native")]

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use hegemony_server::{
    model::Action,
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
