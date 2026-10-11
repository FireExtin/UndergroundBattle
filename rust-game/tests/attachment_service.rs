#![cfg(feature = "native")]
use hegemony_server::{
    model::{Action, WinContext, Window},
    room::{RoomCommand, RoomEnvelope, SessionAction},
    service::{CreateRoom, JoinRoom, Session, Store},
};
use rusqlite::params;

// Explicit synthetic *initial* layout. Every later state is produced by real
// authenticated session commands, and the native journal is replayed independently.
async fn fixture(
    path: &std::path::Path,
    mode: &str,
    hk: bool,
) -> (Store, Vec<Session>, String, String) {
    let store = Store::open(path).unwrap();
    let host = store
        .create(CreateRoom {
            name: "P0".into(),
            mode: mode.into(),
            deck_id: "hunters".into(),
            deck_draft: None,
        })
        .await
        .unwrap();
    let mut seats = vec![host];
    let n = if mode == "teams" { 4 } else { 2 };
    for s in 1..n {
        seats.push(
            store
                .join(JoinRoom {
                    invite_code: seats[0].invite_code.clone(),
                    name: format!("P{s}"),
                    deck_id: "hunters".into(),
                    deck_draft: None,
                })
                .await
                .unwrap(),
        );
    }
    drop(store);
    let db = rusqlite::Connection::open(path).unwrap();
    let raw: String = db
        .query_row(
            "SELECT state FROM rooms WHERE id=?1",
            [&seats[0].room_id],
            |r| r.get(0),
        )
        .unwrap();
    let mut g = RoomEnvelope::from_persisted(&raw).unwrap().game;
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
    while let Some(p) = g.pending.clone() {
        g.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    g.first_team = 0;
    g.active_team = 0;
    g.priority_team = 0;
    g.passed.clear();
    g.team_passed = [false; 2];
    for r in &mut g.regions {
        r.cards.clear();
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
    }
    let mut card_id = String::new();
    let mut host_id = String::new();
    if hk {
        if let Some(i) = g
            .regions
            .iter()
            .position(|r| r.card.definition == "DQJC116")
        {
            g.regions.swap(0, i);
        } else {
            let i = g
                .world
                .iter()
                .position(|c| c.definition == "DQJC116")
                .unwrap();
            std::mem::swap(&mut g.regions[0].card, &mut g.world[i]);
        }
        g.window = Some(Window::Win(0, 0));
        g.win_contexts = vec![WinContext {
            region: 0, seat: 0,
            region_instance: g.regions[0].card.id.clone(),
            resume_window: Window::After(0, 2),
        }];
        for s in 0..n {
            g.players[s].deck.clear();
            for _ in 0..2 {
                let c = g.make_card("BQ022", s);
                g.players[s].deck.push(c);
            }
            for _ in 0..4 {
                let c = g.make_card("JC125", s);
                g.players[s].deck.push(c);
            }
        }
    } else {
        g.window = Some(Window::Action(0));
        let c = g.make_card("LC22", 0);
        host_id = c.id.clone();
        g.regions[0].cards.push(c);
        let c = g.make_card("BQ022", 0);
        card_id = c.id.clone();
        g.players[0].hand.push(c);
        let c = g.make_card("JC091", 1);
        g.players[1].hand.push(c);
        for s in 0..2 {
            for _ in 0..if s == 0 { 1 } else { 3 } {
                let c = g.make_card("JC091", s);
                g.players[s].assets.push(c);
            }
        }
    }
    g.version = (n - 1) as u64;
    g.log.clear();
    let initial = serde_json::to_string(&RoomEnvelope::from_game(g)).unwrap();
    db.execute(
        "UPDATE rooms SET state=?2,initial_state=?2,revision=?3 WHERE id=?1",
        params![seats[0].room_id, initial, (n - 1) as u64],
    )
    .unwrap();
    db.execute("DELETE FROM journal WHERE room_id=?1", [&seats[0].room_id])
        .unwrap();
    drop(db);
    (Store::open(path).unwrap(), seats, card_id, host_id)
}
async fn act(
    store: &Store,
    seats: &[Session],
    seat: usize,
    action: Action,
    id: &str,
) -> (hegemony_server::room::RoomView, RoomCommand) {
    let view = store
        .state_at_now(&seats[0].room_id, &seats[seat].token, 0)
        .await
        .unwrap();
    let session = if let Some(w) = view.response_window.clone() {
        assert_eq!(
            action.kind, "pass",
            "paid responses must Begin/Submit explicitly"
        );
        SessionAction::PassResponse { window_id: w.id }
    } else {
        action.into()
    };
    let command = RoomCommand {
        command_id: id.into(),
        expected_version: view.version,
        action: session,
    };
    let next = store
        .command_at_now(
            &seats[0].room_id,
            &seats[seat].token,
            command.clone(),
            1_000,
        )
        .await
        .unwrap();
    (next, command)
}
async fn resolve_top(store: &Store, seats: &[Session], counter: &mut u32) {
    let n = store
        .state_at_now(&seats[0].room_id, &seats[0].token, 0)
        .await
        .unwrap()
        .stack
        .len();
    assert!(n > 0);
    for _ in 0..8 {
        let v = store
            .state_at_now(&seats[0].room_id, &seats[0].token, 0)
            .await
            .unwrap();
        if v.stack.len() < n || v.waiting_choice.is_some() {
            return;
        }
        let w = v.response_window.unwrap();
        let seat = w
            .members
            .iter()
            .find(|m| m.status == "undecided")
            .unwrap()
            .player_id[1..]
            .parse::<usize>()
            .unwrap();
        *counter += 1;
        act(
            store,
            seats,
            seat,
            Action::new("pass"),
            &format!("pass-{counter}"),
        )
        .await;
    }
    panic!("top did not resolve");
}
#[tokio::test]
async fn attachment_paid_stack_and_owner_recycle_restore_with_original_receipt_and_replay() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("attachment.sqlite3");
    let (store, seats, card_id, host_id) = fixture(&path, "duel", false).await;
    let (paid, command) = act(
        &store,
        &seats,
        0,
        Action {
            card_id: Some(card_id.clone()),
            target_id: Some(host_id.clone()),
            ..Action::new("play")
        },
        "equipment",
    )
    .await;
    assert!(paid.attachments.is_empty());
    assert_eq!(paid.stack.len(), 1);
    assert_eq!(
        paid.assets
            .iter()
            .filter(|c| c.owner == "p0" && c.exhausted)
            .count(),
        1
    );
    assert_eq!(paid.stack[0].target_summaries[0].instance_id, host_id);
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(
        serde_json::to_string(
            &store
                .state_at_now(&seats[0].room_id, &seats[0].token, 0)
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_string(&paid).unwrap()
    );
    let receipt = store
        .command_at_now(&seats[0].room_id, &seats[0].token, command.clone(), 99_999)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_string(&receipt).unwrap(),
        serde_json::to_string(&paid).unwrap()
    );
    let mut count = 0;
    resolve_top(&store, &seats, &mut count).await;
    let attached = store
        .state_at_now(&seats[0].room_id, &seats[0].token, 0)
        .await
        .unwrap();
    assert_eq!(attached.attachments.len(), 1);
    let attachment_id = attached.attachments[0].card.instance_id.clone();
    assert_eq!(attached.attachments[0].host_id, host_id);
    assert_eq!(attached.regions[0].icons_by_team[0].combat, 2);
    act(&store, &seats, 0, Action::new("pass"), "end-action0").await;
    let view = store
        .state_at_now(&seats[0].room_id, &seats[1].token, 0)
        .await
        .unwrap();
    let murder = view
        .hand
        .iter()
        .find(|c| c.card_id.as_deref() == Some("JC091"))
        .unwrap()
        .instance_id
        .clone();
    act(
        &store,
        &seats,
        1,
        Action {
            card_id: Some(murder),
            target_id: Some(host_id.clone()),
            ..Action::new("play")
        },
        "murder",
    )
    .await;
    resolve_top(&store, &seats, &mut count).await;
    let final_view = store
        .state_at_now(&seats[0].room_id, &seats[0].token, 0)
        .await
        .unwrap();
    assert!(final_view.attachments.is_empty());
    assert!(final_view.stack.is_empty());
    let recycled = final_view
        .hand
        .iter()
        .find(|c| c.card_id.as_deref() == Some("BQ022"))
        .unwrap();
    assert_ne!(recycled.instance_id, card_id);
    assert_ne!(recycled.instance_id, attachment_id);
    assert!(final_view
        .graveyard
        .iter()
        .any(|c| c.card_id.as_deref() == Some("LC22")));
    assert!(final_view
        .graveyard
        .iter()
        .all(|c| c.card_id.as_deref() != Some("BQ022")));
    let opponent = store
        .state_at_now(&seats[0].room_id, &seats[1].token, 0)
        .await
        .unwrap();
    assert!(opponent
        .hand
        .iter()
        .all(|c| c.card_id.as_deref() != Some("BQ022")));
    drop(store);
    let store = Store::open(&path).unwrap();
    assert!(store.audit_replay(&seats[0].room_id).unwrap().matches);
    assert_eq!(
        serde_json::to_string(
            &store
                .state_at_now(&seats[0].room_id, &seats[0].token, 0)
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_string(&final_view).unwrap()
    );
    assert_eq!(
        serde_json::to_string(
            &store
                .command_at_now(&seats[0].room_id, &seats[0].token, command, 99_999)
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_string(&paid).unwrap()
    );
}
#[tokio::test]
async fn actual_hong_kong_attachment_commitments_are_private_and_resume_once_from_sqlite() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hong-kong.sqlite3");
    let (mut store, seats, _, _) = fixture(&path, "teams", true).await;
    let original = store.replay(&seats[0].room_id).unwrap();
    let decks = original
        .players
        .iter()
        .map(|p| p.deck.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    for s in 0..4 {
        act(
            &store,
            &seats,
            s,
            Action::new("pass"),
            &format!("win-pass-{s}"),
        )
        .await;
    }
    let p = store
        .state_at_now(&seats[0].room_id, &seats[0].token, 0)
        .await
        .unwrap()
        .pending_choice
        .clone()
        .unwrap();
    assert_eq!(p.kind, "trigger");
    act(
        &store,
        &seats,
        0,
        Action {
            choice_id: Some(p.id),
            selected: Some(vec!["accept".into()]),
            ..Action::new("choose")
        },
        "accept-hk",
    )
    .await;
    let mut counter = 0;
    resolve_top(&store, &seats, &mut counter).await;
    let mut first_receipt = None;
    for s in 0..4 {
        let mine = store
            .state_at_now(&seats[0].room_id, &seats[s].token, 0)
            .await
            .unwrap();
        let choice = mine.pending_choice.clone().unwrap();
        assert_eq!(choice.kind, "search");
        assert_eq!(choice.options.len(), 2);
        assert!(choice
            .options
            .iter()
            .all(|o| o.card.as_ref().unwrap().card_id.as_deref() == Some("BQ022")));
        for other in 0..4 {
            if other != s {
                assert!(store
                    .state_at_now(&seats[0].room_id, &seats[other].token, 0)
                    .await
                    .unwrap()
                    .pending_choice
                    .is_none());
            }
        }
        let selected = if s == 1 {
            vec![]
        } else {
            vec![choice.options[0].id.clone()]
        };
        let (receipt, command) = act(
            &store,
            &seats,
            s,
            Action {
                choice_id: Some(choice.id),
                selected: Some(selected),
                ..Action::new("choose")
            },
            &format!("hk-choice-{s}"),
        )
        .await;
        if s == 0 {
            first_receipt = Some((receipt.clone(), command.clone()));
        }
        if s < 3 {
            let state = store.replay(&seats[0].room_id).unwrap();
            assert!(state.players.iter().all(|p| p.hand.is_empty()));
            assert_eq!(
                state
                    .players
                    .iter()
                    .map(|p| p.deck.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                decks
            );
            assert!(state.log.iter().all(|l| !l.text.contains("同时展示检索")));
        }
        drop(store);
        store = Store::open(&path).unwrap();
        let persisted = store
            .command_at_now(&seats[0].room_id, &seats[s].token, command, 99_999)
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_string(&persisted).unwrap(),
            serde_json::to_string(&receipt).unwrap()
        );
        assert!(store.audit_replay(&seats[0].room_id).unwrap().matches);
    }
    let state = store.replay(&seats[0].room_id).unwrap();
    assert_eq!(
        state
            .players
            .iter()
            .map(|p| p.hand.len())
            .collect::<Vec<_>>(),
        vec![1, 0, 1, 1]
    );
    assert_eq!(
        state
            .players
            .iter()
            .map(|p| p.deck.len())
            .collect::<Vec<_>>(),
        vec![5, 6, 5, 5]
    );
    assert_eq!(
        state
            .log
            .iter()
            .filter(|l| l.text.contains("同时展示检索") && l.text.contains("合金指虎"))
            .count(),
        3
    );
    assert!(state
        .players
        .iter()
        .flat_map(|p| p.hand.iter())
        .all(|c| c.definition == "BQ022" && !decks[c.owner].contains(&c.id)));
    for s in 0..4 {
        let v = store
            .state_at_now(&seats[0].room_id, &seats[s].token, 0)
            .await
            .unwrap();
        assert_eq!(v.hand.len(), usize::from(s != 1));
        assert!(v.pending_choice.is_none());
        assert!(v.response_window.is_none());
    }
    let (receipt, command) = first_receipt.unwrap();
    assert_eq!(
        serde_json::to_string(
            &store
                .command_at_now(&seats[0].room_id, &seats[0].token, command, 999_999)
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_string(&receipt).unwrap()
    );
    let sql = rusqlite::Connection::open(&path).unwrap();
    let receipts: u32 = sql
        .query_row(
            "SELECT COUNT(*) FROM commands WHERE room_id=?1 AND command_id LIKE 'hk-choice-%'",
            [&seats[0].room_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(receipts, 4);
}

#[tokio::test]
async fn grave_play_sqlite_preserves_paid_origin_and_full_intent_receipt_across_reopen() {
    use hegemony_server::model::PlaySource;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("grave-play.sqlite3");
    let (store, seats, _, _) = fixture(&path, "teams", false).await;
    let mut g = store.replay(&seats[0].room_id).unwrap().game;
    drop(store);
    for r in &mut g.regions {
        r.cards.clear();
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
        p.graveyard.clear();
    }
    let corpse = g.make_card("JC085", 0);
    let old = corpse.id.clone();
    g.players[0].graveyard.push(corpse);
    for definition in ["JC085", "JC084"] {
        let asset = g.make_card(definition, 0);
        g.players[0].assets.push(asset);
    }
    // Explicit local initial fixture only; subsequent commands are authenticated.
    let initial = serde_json::to_string(&RoomEnvelope::from_game(g)).unwrap();
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute(
        "UPDATE rooms SET state=?2,initial_state=?2 WHERE id=?1",
        params![seats[0].room_id, initial],
    )
    .unwrap();
    drop(db);
    let store = Store::open(&path).unwrap();
    let (paid, command) = act(
        &store,
        &seats,
        0,
        Action {
            card_id: Some(old.clone()),
            region: Some(0),
            ..Action::new("deploy")
        },
        "grave-paid",
    )
    .await;
    assert!(paid.hand.is_empty() && paid.graveyard.is_empty());
    assert_eq!(paid.stack.len(), 1);
    assert_eq!(
        paid.assets
            .iter()
            .filter(|c| c.controller == "p0" && c.exhausted)
            .count(),
        2
    );
    let state = store.replay(&seats[0].room_id).unwrap();
    assert_eq!(
        state.stack[0].frame.as_ref().unwrap().source.play_source,
        Some(PlaySource::Graveyard)
    );
    let before = serde_json::to_string(&state).unwrap();
    let mut altered = command.clone();
    let SessionAction::Game { action } = &mut altered.action else {
        panic!("expected game intent")
    };
    action.region = Some(1);
    assert!(store
        .command_at_now(&seats[0].room_id, &seats[0].token, altered, 1_000)
        .await
        .is_err());
    assert_eq!(
        serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap(),
        before
    );
    drop(store);
    let store = Store::open(&path).unwrap();
    let receipt = store
        .command_at_now(&seats[0].room_id, &seats[0].token, command.clone(), 99_999)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_string(&receipt).unwrap(),
        serde_json::to_string(&paid).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap(),
        before
    );
    let mut counter = 0;
    resolve_top(&store, &seats, &mut counter).await;
    let final_state = store.replay(&seats[0].room_id).unwrap();
    assert_eq!(final_state.regions[0].cards.len(), 1);
    assert_eq!(final_state.regions[0].cards[0].definition, "JC085");
    assert_ne!(final_state.regions[0].cards[0].id, old);
    assert!(!final_state.regions[0].cards[0].face_down);
    assert_eq!(
        final_state
            .icons(&final_state.regions[0].cards[0], 0)
            .influence,
        1
    );
    assert_eq!(
        final_state.players[0]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        2
    );
    assert!(store.audit_replay(&seats[0].room_id).unwrap().matches);
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(
        serde_json::to_string(
            &store
                .command_at_now(&seats[0].room_id, &seats[0].token, command, 99_999)
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_string(&paid).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap(),
        serde_json::to_string(&final_state).unwrap()
    );
}
