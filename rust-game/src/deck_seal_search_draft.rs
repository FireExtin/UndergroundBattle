//! Historical test-only preparation retained unchanged alongside the real programs.
//! Actual XQ44/JZ02 cardinality and host rulings live in deck_seal_search/tests.
use crate::catalog::{catalog, CardDefinition};
use crate::jc029_tests::{board, checkpoint, choose, game, pass_top};
use crate::model::*;

fn dream_card(d: &CardDefinition) -> bool {
    d.subtypes.iter().any(|s| s == "梦境")
}

fn space_spell_card(d: &CardDefinition) -> bool {
    d.kind == "spell"
        && d.subtypes.iter().any(|s| s == "空间")
        && d.subtypes.iter().any(|s| s == "法术" || s == "事务-法术")
}

// A test-only positive-path prototype, deliberately without a new runtime Op,
// a catalog entry or a rule for zero selections / an invalid host.
fn seal_selected_deck_card_positive(
    g: &mut Game,
    actor: usize,
    selected: &str,
    host: &str,
    dream: bool,
) {
    assert!(!g.players[actor].eliminated);
    let (_, h) = g.board(host).expect("positive-path host exists");
    assert!(!h.face_down && crate::catalog::card(&h.definition).kind == "character");
    let index = g.players[actor]
        .deck
        .iter()
        .position(|c| c.id == selected)
        .unwrap();
    let c = &g.players[actor].deck[index];
    let d = crate::catalog::card(&c.definition);
    assert!(if dream {
        dream_card(d)
    } else {
        space_spell_card(d)
    });
    let c = g.players[actor].deck.remove(index);
    g.seal_card_on_valid_host(c, actor, host);
    g.shuffle_player(actor);
}

fn deck(g: &mut Game, actor: usize, selected: &str, owner: usize) -> String {
    g.players[actor].deck.clear();
    let c = g.make_card(selected, owner);
    let id = c.id.clone();
    g.players[actor].deck.push(c);
    for _ in 0..5 {
        let c = g.make_card("JC125", actor);
        g.players[actor].deck.push(c);
    }
    id
}

#[test]
fn deck_seal_draft_dream_filter_includes_transaction_and_character() {
    let matches = catalog()
        .cards
        .iter()
        .filter(|d| dream_card(d))
        .map(|d| d.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(matches, ["XQ41", "XQ45"]);
    assert_eq!(
        catalog()
            .cards
            .iter()
            .find(|d| d.id == "XQ45")
            .unwrap()
            .kind,
        "spell"
    );
}

#[test]
fn deck_seal_draft_space_spell_filter_accepts_both_existing_subtype_encodings() {
    let matches = catalog()
        .cards
        .iter()
        .filter(|d| space_spell_card(d))
        .map(|d| d.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(matches, ["JC005", "XQ03", "JC132"]);
}

#[test]
fn deck_seal_draft_filters_require_printed_subtypes_instead_of_color_or_domain() {
    let registry = &catalog().cards;
    let mut changed = registry.iter().find(|d| d.id == "XQ41").unwrap().clone();
    changed.subtypes.clear();
    assert!(!dream_card(&changed));
    let mut changed = registry.iter().find(|d| d.id == "XQ03").unwrap().clone();
    changed.kind = "character".into();
    assert!(!space_spell_card(&changed));
    changed.kind = "spell".into();
    changed.subtypes = vec!["空间".into(), "阴谋".into()];
    assert!(!space_spell_card(&changed));
    changed.subtypes = vec!["法术".into(), "心灵".into()];
    assert!(!space_spell_card(&changed));
}

#[test]
fn deck_seal_draft_positive_space_spell_goes_directly_to_blank_zone_then_one_shuffle() {
    let mut g = game(0);
    let host = board(&mut g, "LC01", 2, 1); // Existing admitted character as a kernel host.
    let selected = deck(&mut g, 0, "XQ03", 1);
    for viewer in 1..4 {
        assert!(!serde_json::to_string(&g.view(viewer))
            .unwrap()
            .contains(&selected));
    }
    let mut expected = g.clone();
    expected.players[0].deck.remove(0);
    expected.shuffle_player(0);
    seal_selected_deck_card_positive(&mut g, 0, &selected, &host, false);
    assert_eq!(g.random, expected.random);
    assert_eq!(
        serde_json::to_value(&g.players[0].deck).unwrap(),
        serde_json::to_value(&expected.players[0].deck).unwrap()
    );
    assert!(g.players[0].hand.is_empty());
    assert_eq!(g.sealed_cards.len(), 1);
    let s = &g.sealed_cards[0];
    assert_eq!(
        (s.card.owner, s.card.controller, s.host_id.as_str()),
        (1, 1, host.as_str())
    );
    assert_ne!(s.card.id, selected);
    assert!(g.effects.is_empty());
    checkpoint(&g);
    for viewer in 0..4 {
        let v = &g.view(viewer).sealed_cards[0].card;
        assert_eq!(v.card_id.as_deref(), Some("XQ03"));
        assert!(v.icons.is_none() && v.magic.is_none() && v.defense.is_none());
    }
}

#[test]
fn deck_seal_draft_positive_dream_event_uses_frozen_holder_once_and_survives_owner_return() {
    let mut g = game(0);
    let host = board(&mut g, "LC01", 2, 1);
    let selected = deck(&mut g, 0, "XQ41", 1);
    seal_selected_deck_card_positive(&mut g, 0, &selected, &host, true);
    assert!(g.players[0].hand.is_empty() && g.players[1].hand.is_empty());
    g.drive().unwrap();
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
    assert!(g.effects.is_empty());
    checkpoint(&g);
    for viewer in 1..4 {
        assert!(g.view(viewer).pending_choice.is_none());
    }
    choose(&mut g, vec!["accept".into()]);
    let saved = g.sealed_cards[0].card.id.clone();
    g.remove_dead(&host, RemovalCause::Destroy);
    assert!(g.sealed_cards.is_empty());
    assert_eq!(g.players[1].hand.len(), 1);
    assert_eq!(g.players[1].hand[0].definition, "XQ41");
    assert_ne!(g.players[1].hand[0].id, saved);
    checkpoint(&g);
    let before = g.players[0].deck.len();
    pass_top(&mut g);
    assert_eq!(g.players[0].deck.len(), before - 1);
    assert_eq!(g.players[0].hand.len(), 1);
    assert!(g.pending.is_none() && g.effects.is_empty());
    checkpoint(&g);
}

#[cfg(feature = "native")]
#[tokio::test]
async fn deck_seal_draft_positive_event_pause_reopen_and_original_receipt_never_reseal() {
    use crate::room::{RoomCommand, RoomEnvelope, SessionAction};
    use crate::service::{CreateRoom, JoinRoom, Store};

    // Only this test's disposable SQLite DB receives an explicit kernel layout.
    // The producer is a direct test-only call; receipt handling starts with the
    // existing XQ41 event, not a not-yet-implemented XQ44/JZ02 search command.
    for take in ["accept", "decline"] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("deck-seal-positive-offline-fixture.sqlite");
        let store = Store::open(&path).unwrap();
        let mut sessions = vec![store
            .create(CreateRoom {
                name: "P0".into(),
                mode: "teams".into(),
                deck_id: "watchers".into(),
                deck_draft: None,
            })
            .await
            .unwrap()];
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
        let mut g = game(1);
        let host = board(&mut g, "LC01", 2, 1);
        let selected = deck(&mut g, 1, "XQ41", 3);
        seal_selected_deck_card_positive(&mut g, 1, &selected, &host, true);
        g.drive().unwrap();
        g.room_id = sessions[0].room_id.clone();
        g.invite_code = sessions[0].invite_code.clone();
        let choice_id = g.pending.as_ref().unwrap().choice.id.clone();
        let sealed_id = g.sealed_cards[0].card.id.clone();
        let random = g.random;
        let deck_before = serde_json::to_value(&g.players[1].deck).unwrap();
        let mut room = crate::jc029_tests::envelope(&g);
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute(
            "UPDATE rooms SET state=?1,revision=?2 WHERE id=?3",
            rusqlite::params![
                serde_json::to_string(&room).unwrap(),
                room.revision,
                g.room_id
            ],
        )
        .unwrap();
        drop(db);

        let mut final_command = None;
        let mut final_response = None;
        let initial_state = serde_json::to_string(&room).unwrap();
        let mut trace = vec![];
        for (index, action) in [
            SessionAction::PauseRoom,
            SessionAction::ResumeRoom,
            SessionAction::Game {
                action: Action {
                    choice_id: Some(choice_id.clone()),
                    selected: Some(if take == "accept" {
                        vec!["accept".into()]
                    } else {
                        vec![]
                    }),
                    ..Action::new("choose")
                },
            },
        ]
        .into_iter()
        .enumerate()
        {
            let now = if index == 0 {
                100
            } else {
                180_000 + index as u64
            };
            let command = RoomCommand {
                command_id: format!("deck-seal-positive-{take}-{index}"),
                expected_version: room.revision,
                action,
            };
            let expected = room.transition(1, Some(command.clone()), now).unwrap();
            assert!(
                expected.error_code.is_none(),
                "{:?}",
                expected.error_message
            );
            trace.push(serde_json::json!({"seat":1,"command":command,"serverNowMs":now.to_string(),
                "transition":expected,"views":(0..4).map(|s| RoomEnvelope::from_persisted(&expected.state).unwrap().view(s,now)).collect::<Vec<_>>() }));
            room = RoomEnvelope::from_persisted(&expected.state).unwrap();
            let store = Store::open(&path).unwrap();
            let response = store
                .command_at_now(&g.room_id, &sessions[1].token, command.clone(), now)
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_value(&response).unwrap(),
                serde_json::to_value(room.view(1, now)).unwrap()
            );
            drop(store);
            assert_eq!(room.game.sealed_cards.len(), 1);
            assert_eq!(room.game.sealed_cards[0].card.id, sealed_id);
            assert_eq!(room.game.sealed_cards[0].card.owner, 3);
            assert_eq!(room.game.random, random);
            assert_eq!(
                serde_json::to_value(&room.game.players[1].deck).unwrap(),
                deck_before
            );
            for viewer in [0, 2, 3] {
                assert!(room.view(viewer, now).pending_choice.is_none());
            }
            if index < 2 {
                assert_eq!(room.game.pending.as_ref().unwrap().choice.id, choice_id);
            } else {
                assert!(room.game.pending.is_none());
                assert_eq!(room.game.stack.len(), if take == "accept" { 1 } else { 0 });
                final_command = Some(command);
                final_response = Some(response);
            }
        }
        let state_before_repeat = serde_json::to_string(&room).unwrap();
        let command = final_command.unwrap();
        let first = final_response.unwrap();
        let store = Store::open(&path).unwrap();
        let repeated = store
            .command_at_now(&g.room_id, &sessions[1].token, command.clone(), 600_000)
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(repeated).unwrap(),
            serde_json::to_value(first).unwrap()
        );
        let mut conflicting = command;
        conflicting.expected_version += 1;
        assert_eq!(
            store
                .command_at_now(&g.room_id, &sessions[1].token, conflicting, 600_000)
                .await
                .unwrap_err()
                .error,
            "command_id_conflict"
        );
        drop(store);
        let db = rusqlite::Connection::open(&path).unwrap();
        let stored: String = db
            .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(stored, state_before_repeat);
        let receipts: u64 = db
            .query_row(
                "SELECT COUNT(*) FROM commands WHERE room_id=?1",
                [&g.room_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(receipts, 3);
        checkpoint(&room.game);
        if let Ok(dir) = std::env::var("DECK_SEAL_DRAFT_RECEIPT_EVIDENCE_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(format!("{dir}/event-{take}-trace.json"), serde_json::to_vec(&serde_json::json!({
                "scope":"Explicit offline kernel fixture after positive deck seal. Real existing XQ41 event pause/resume/accept-or-decline commands. Store duplicate checks are Native-only assertions; no search command or browser claim.",
                "initialState":initial_state,"steps":trace,"originalReceiptRepeatChecked":true,"receipts":3
            })).unwrap()).unwrap();
        }
    }
}
