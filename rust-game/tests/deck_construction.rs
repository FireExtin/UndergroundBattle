use hegemony_server::{
    catalog::{self, DeckEntry},
    deck::{self, DeckDraft},
    model::{Action, Game},
};

fn draft(id: &str) -> DeckDraft {
    let mut draft = deck::preset("watchers").unwrap();
    draft.id = id.into();
    draft.name = "公开自组名称".into();
    draft.updated_at = "2026-10-02T00:00:00Z".into();
    draft
}
fn entry(id: &str, count: usize) -> DeckEntry {
    DeckEntry {
        card_id: id.into(),
        count,
    }
}

#[test]
fn minimum_unlimited_multifaction_and_canonical_duplicate_entries() {
    let mut d = draft("multi");
    d.cards = vec![
        entry("JC125", 46),
        entry("JC014", 1),
        entry("XQ12", 1),
        entry("JC091", 1),
        entry("JC042", 1),
        entry("JC058", 1),
        entry("JC002", 1),
    ];
    let accepted = deck::validate(d).unwrap();
    assert_eq!(accepted.cards.iter().map(|c| c.count).sum::<usize>(), 52);
    assert_eq!(deck::copy_limit(catalog::card("JC125")), None);
    assert_eq!(deck::copy_limit(catalog::card("LC23")), Some(3)); // 独有 is not deck 唯一.
    let mut d = draft("canonical");
    d.cards = vec![entry("JC125", 20), entry("LC23", 3), entry("JC125", 27)];
    assert_eq!(
        deck::validate(d).unwrap().cards,
        vec![entry("JC125", 47), entry("LC23", 3)]
    );
    assert_eq!(catalog::catalog().deck_build_rules.faction_limit, None);
}

#[test]
fn printed_name_groups_versions_and_unique_is_distinct_from_in_play_unique() {
    // Typed registry fixture represents two printed versions; no extra cards are opened to Game.
    let mut cards = catalog::catalog().cards.clone();
    let mut alias = catalog::card("LC20").clone();
    alias.id = "fixture-other-print".into();
    cards.push(alias.clone());
    let mut d = draft("same-name");
    d.cards = vec![entry("JC125", 47), entry("LC20", 1), entry(&alias.id, 2)];
    assert!(deck::validate_with_cards(d.clone(), &cards).is_ok());
    d.cards[1].count = 2;
    assert!(deck::validate_with_cards(d, &cards)
        .unwrap_err()
        .contains("同名"));
    cards.last_mut().unwrap().keywords = vec!["唯一".into()];
    let mut d = draft("unique");
    d.cards = vec![entry("JC125", 49), entry(&alias.id, 1)];
    assert!(deck::validate_with_cards(d.clone(), &cards).is_ok());
    d.cards.push(entry("LC20", 1));
    assert!(deck::validate_with_cards(d, &cards)
        .unwrap_err()
        .contains("最多1张"));
}

#[test]
fn malformed_disabled_overflow_versions_and_society_are_rejected() {
    for (cards, message) in [
        (vec![entry("JC125", 49)], "50"),
        (vec![entry("JC125", 50), entry("LC20", 0)], "大于0"),
        (vec![entry("JC125", 50), entry("unknown", 1)], "未开放"),
        (vec![entry("JC125", 50), entry("DQJC107", 1)], "不能加入"),
        (vec![entry("JC125", usize::MAX), entry("JC125", 1)], "溢出"),
        (vec![entry("JC125", 2049)], "不是原作规则"),
    ] {
        let mut d = draft("invalid");
        d.cards = cards;
        assert!(deck::validate(d).unwrap_err().contains(message));
    }
    let mut d = draft("capacity");
    d.cards = vec![entry("JC125", 2048)];
    assert!(deck::validate(d.clone()).is_ok());
    d.society_id = Some("not-implemented".into());
    assert!(deck::validate(d).unwrap_err().contains("秘社"));
    for field in ["rulesVersion", "cardPoolVersion", "engineVersion"] {
        let mut value = serde_json::to_value(draft("version")).unwrap();
        value[field] = "old-version".into();
        assert!(deck::validate(serde_json::from_value(value.clone()).unwrap()).is_err());
        value.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<DeckDraft>(value).is_err());
    }
    let mut cards = catalog::catalog().cards.clone();
    cards.iter_mut().find(|c| c.id == "LC20").unwrap().supported = false;
    let mut d = draft("disabled");
    d.cards = vec![entry("JC125", 49), entry("LC20", 1)];
    assert!(deck::validate_with_cards(d, &cards).is_err());
}

#[test]
fn frozen_snapshot_private_views_lobby_changes_and_started_rejections() {
    let mut saved = draft("private-alpha-id");
    saved.cards = deck::preset("responders").unwrap().cards;
    let frozen = deck::validate(saved.clone()).unwrap();
    let mut game = Game::new_with_deck(
        "freeze".into(),
        "FREEZE".into(),
        "duel".into(),
        "P0".into(),
        saved.clone(),
        172,
    )
    .unwrap();
    let mut other = draft("private-beta-id");
    other.cards = deck::preset("keepers").unwrap().cards;
    game.join_with_deck("P1".into(), other.clone()).unwrap();
    saved.cards.clear();
    saved.name = "later edit".into();
    assert_eq!(game.players[0].deck_snapshot.as_ref(), Some(&frozen));
    assert_eq!(game.view(0).your_deck.as_ref(), Some(&frozen));
    let peer = game.view(1);
    assert_eq!(peer.your_deck.unwrap().id, other.id);
    assert_eq!(peer.players[0].deck_id, "custom");
    assert_eq!(peer.players[0].deck_name, frozen.name);
    assert!(!serde_json::to_string(&game.view(1))
        .unwrap()
        .contains("private-alpha-id"));
    assert!(!serde_json::to_string(&game.view(1))
        .unwrap()
        .contains("JC042"));
    assert!(!serde_json::to_string(catalog::catalog())
        .unwrap()
        .contains("private-alpha-id"));
    game.apply(0, Action::new("ready")).unwrap();
    game.apply(
        0,
        Action {
            option: Some("keepers".into()),
            ..Action::new("deck")
        },
    )
    .unwrap();
    assert!(!game.players[0].ready && game.players[0].deck_snapshot.is_none());
    game.apply(
        0,
        Action {
            deck_draft: Some(frozen.clone()),
            ..Action::new("deck")
        },
    )
    .unwrap();
    for seat in 0..2 {
        game.apply(seat, Action::new("ready")).unwrap();
    }
    game.apply(0, Action::new("start")).unwrap();
    for seat in 0..2 {
        let player = &game.players[seat];
        assert_eq!(player.hand.len() + player.deck.len(), 50);
    }
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
    let serialized = serde_json::to_string(&game).unwrap();
    for action in [
        Action {
            deck_draft: Some(draft("new")),
            ..Action::new("deck")
        },
        Action {
            option: Some("hunters".into()),
            ..Action::new("deck")
        },
    ] {
        assert!(game.apply(0, action).is_err());
        assert_eq!(serde_json::to_string(&game).unwrap(), serialized);
    }
    assert_eq!(game.view(0).world_deck_count, 7);
    assert_eq!(
        Game::from_persisted(&serialized).unwrap().players[0]
            .deck_snapshot
            .as_ref(),
        Some(&frozen)
    );
    // Restart-entry fixture: this checks restarting, not a fabricated natural game ending.
    game.status = "finished".into();
    game.apply(0, Action::new("restart")).unwrap();
    assert_eq!(game.players[0].deck_snapshot.as_ref(), Some(&frozen));
    assert_eq!(game.players[0].hand.len() + game.players[0].deck.len(), 50);
}

#[cfg(feature = "native")]
#[tokio::test]
async fn custom_deck_join_change_receipt_reopen_and_replay_keep_frozen_private_specs() {
    use hegemony_server::service::{Command, CreateRoom, JoinRoom, Store};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("decks.sqlite3");
    let store = Store::open(&path).unwrap();
    let mut alpha = draft("secret-alpha");
    alpha.cards = deck::preset("responders").unwrap().cards;
    let a = store
        .create(CreateRoom {
            name: "Alpha".into(),
            mode: "duel".into(),
            deck_id: String::new(),
            deck_draft: Some(alpha.clone()),
        })
        .await
        .unwrap();
    for field in ["rulesVersion", "cardPoolVersion", "engineVersion"] {
        let mut wrong = serde_json::to_value(draft("wrong-seat-version")).unwrap();
        wrong[field] = "old-version".into();
        assert!(store
            .join(JoinRoom {
                invite_code: a.invite_code.clone(),
                name: "Rejected join".into(),
                deck_id: String::new(),
                deck_draft: Some(serde_json::from_value(wrong).unwrap()),
            })
            .await
            .is_err());
        let unchanged = store.state(&a.room_id, &a.token).await.unwrap();
        assert_eq!(unchanged.version, 0);
        assert_eq!(unchanged.players.len(), 1);
    }
    let mut beta = draft("secret-beta");
    beta.cards = deck::preset("keepers").unwrap().cards;
    let b = store
        .join(JoinRoom {
            invite_code: a.invite_code.clone(),
            name: "Beta".into(),
            deck_id: String::new(),
            deck_draft: Some(beta.clone()),
        })
        .await
        .unwrap();
    alpha.cards.clear();
    assert_eq!(
        store.replay(&a.room_id).unwrap().players[0]
            .deck_snapshot
            .as_ref()
            .unwrap()
            .cards
            .iter()
            .map(|c| c.count)
            .sum::<usize>(),
        50
    );
    assert!(
        !serde_json::to_string(&store.state(&a.room_id, &b.token).await.unwrap())
            .unwrap()
            .contains("secret-alpha")
    );
    assert!(
        !serde_json::to_string(&store.state(&a.room_id, &b.token).await.unwrap())
            .unwrap()
            .contains("JC042")
    );
    let mut invalid = draft("bad");
    invalid.engine_version = "rust-v0.2.3".into();
    assert!(store
        .command(
            &a.room_id,
            &a.token,
            Command {
                command_id: "invalid-deck".into(),
                expected_version: 1,
                action: Action {
                    deck_draft: Some(invalid),
                    ..Action::new("deck")
                }
            }
        )
        .await
        .is_err());
    assert_eq!(store.state(&a.room_id, &a.token).await.unwrap().version, 1);
    let change = Command {
        command_id: "replace-deck".into(),
        expected_version: 1,
        action: Action {
            deck_draft: Some(draft("secret-gamma")),
            ..Action::new("deck")
        },
    };
    let changed = store
        .command(&a.room_id, &a.token, change.clone())
        .await
        .unwrap();
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(
        serde_json::to_string(&store.state(&a.room_id, &a.token).await.unwrap()).unwrap(),
        serde_json::to_string(&changed).unwrap()
    );
    let retry = store.command(&a.room_id, &a.token, change).await.unwrap();
    assert_eq!(
        serde_json::to_string(&retry).unwrap(),
        serde_json::to_string(&changed).unwrap()
    );
    assert_eq!(store.state(&a.room_id, &a.token).await.unwrap().version, 2);
    for (seat, token) in [(0, &a.token), (1, &b.token)] {
        store
            .command(
                &a.room_id,
                token,
                Command {
                    command_id: format!("ready-{seat}"),
                    expected_version: 2 + seat as u64,
                    action: Action::new("ready"),
                },
            )
            .await
            .unwrap();
    }
    store
        .command(
            &a.room_id,
            &a.token,
            Command {
                command_id: "start".into(),
                expected_version: 4,
                action: Action::new("start"),
            },
        )
        .await
        .unwrap();
    let game = store.replay(&a.room_id).unwrap();
    assert_eq!(
        game.players[0].deck_snapshot.as_ref().unwrap().id,
        "secret-gamma"
    );
    assert_eq!(game.players[1].deck_snapshot.as_ref().unwrap().id, beta.id);
    assert!(
        Store::open_read_only(&path)
            .unwrap()
            .audit_replay(&a.room_id)
            .unwrap()
            .matches
    );
    let connection = rusqlite::Connection::open(&path).unwrap();
    let join: String = connection
        .query_row(
            "SELECT entry FROM journal WHERE room_id=?1 AND version=1",
            [&a.room_id],
            |r| r.get(0),
        )
        .unwrap();
    let join: serde_json::Value = serde_json::from_str(&join).unwrap();
    assert_eq!(join["JoinWithDeck"]["deck_draft"]["id"], "secret-beta");
}
