//! Internal-feature registry only; every transition uses normal create/join/session commands.
use super::*;

pub(super) fn natural_case() -> Value {
    let seed = "18446744073709551615";
    let make_draft = |society: Option<&str>| {
        let mut d = deck::preset("watchers").unwrap();
        d.id = "fixture-foundation-deck".into();
        d.name = "internal fixture".into();
        d.cards = vec![catalog::DeckEntry {
            card_id: "JC125".into(),
            count: 50,
        }];
        d.society_id = society.map(str::to_owned);
        d
    };
    let draft = make_draft(Some("FIXTURE_SOCIETY_SIX"));
    let mut room = RoomEnvelope::from_game(
        Game::new_with_deck(
            "888888888888888888880001".into(),
            "SOCIETY".into(),
            "teams".into(),
            "P0".into(),
            draft.clone(),
            seed.parse().unwrap(),
        )
        .unwrap(),
    );
    let mut steps = vec![step(
        &room,
        "newGameWithDeck",
        json!([
            room.room_id,
            room.invite_code,
            "teams",
            "P0",
            serde_json::to_string(&draft).unwrap(),
            seed
        ]),
        0,
    )];
    for (seat, id) in [
        (1, None),
        (2, Some("FIXTURE_SOCIETY_FOUR")),
        (3, Some("FIXTURE_SOCIETY_PENDING")),
    ] {
        let d = make_draft(id);
        room.game
            .join_with_deck(format!("P{seat}"), d.clone())
            .unwrap();
        room.revision = room.game.version;
        steps.push(step(
            &room,
            "joinGameWithDeck",
            json!([format!("P{seat}"), serde_json::to_string(&d).unwrap()]),
            seat,
        ));
        assert!(room
            .view(0, 0)
            .society_zones
            .iter()
            .all(|z| z.card.is_none()));
    }
    for seat in 0..4 {
        apply_game(&mut room, &mut steps, seat, Action::new("ready"));
    }
    apply_game(&mut room, &mut steps, 0, Action::new("start"));
    assert_eq!(
        room.players
            .iter()
            .map(|p| p.hand.len())
            .collect::<Vec<_>>(),
        vec![6, 6, 4, 6]
    );
    let source = room.players[0]
        .society_zone
        .card
        .as_ref()
        .unwrap()
        .id
        .clone();
    let pending = room.players[3]
        .society_zone
        .card
        .as_ref()
        .unwrap()
        .id
        .clone();
    let mut paid = false;
    let mut resolved = false;
    for _ in 0..1000 {
        if paid && room.turn >= 2 {
            assert!(
                !room.players[0]
                    .society_zone
                    .card
                    .as_ref()
                    .unwrap()
                    .exhausted
            );
            assert_eq!(
                room.players[0].society_zone.card.as_ref().unwrap().id,
                source
            );
            break;
        }
        if room.pending.is_some() {
            let (seat, a) = pick_choice(&room.game);
            apply_game(&mut room, &mut steps, seat, a);
            continue;
        }
        let legal: Vec<_> = (0..4)
            .map(|seat| {
                room.view(seat, room.pacing.last_server_now_ms)
                    .game
                    .legal_actions
            })
            .collect();
        if !paid {
            if let Some(a) = legal[0].iter().find(|a| {
                a.action.card_id.as_deref() == Some(source.as_str()) && a.action.kind == "activate"
            }) {
                let a = a.action.clone();
                let before = room.players[0].hand.len();
                apply_game(&mut room, &mut steps, 0, a);
                assert!(
                    room.players[0]
                        .society_zone
                        .card
                        .as_ref()
                        .unwrap()
                        .exhausted
                );
                assert!(room.players[0].assets[0].exhausted);
                assert_eq!(room.players[0].hand.len(), before);
                assert_eq!(
                    room.stack
                        .last()
                        .unwrap()
                        .frame
                        .as_ref()
                        .unwrap()
                        .already_paid
                        .len(),
                    2
                );
                paid = true;
                continue;
            }
            if let Some(a) = legal[0].iter().find(|a| a.action.kind == "asset") {
                let a = a.action.clone();
                apply_game(&mut room, &mut steps, 0, a);
                continue;
            }
        }
        if paid && room.stack.is_empty() {
            resolved = true;
        }
        let (seat, a) = legal
            .iter()
            .enumerate()
            .find_map(|(seat, actions)| {
                actions
                    .iter()
                    .find(|a| a.action.kind == "pass")
                    .map(|a| (seat, a.action.clone()))
            })
            .expect("natural pass available");
        apply_game(&mut room, &mut steps, seat, a);
    }
    assert!(paid && resolved && room.turn >= 2);
    for s in 0..4 {
        let v = room.view(s, room.pacing.last_server_now_ms);
        assert!(v
            .society_zones
            .iter()
            .filter_map(|z| z.card.as_ref())
            .all(|c| c.region.is_none() && c.kind == "society" && !c.face_down));
        assert!(v.hand.iter().all(|c| c.owner == format!("p{s}")));
        assert!(!v
            .legal_actions
            .iter()
            .any(|a| a.action.card_id.as_deref() == Some(pending.as_str())));
    }
    let rejected_commands = vec![rejected(
        &room,
        1,
        Action {
            card_id: Some(source),
            ability_id: Some("fixture-draw".into()),
            ..Action::new("activate")
        },
    )];
    json!({"name":"internal-society-foundation-natural-create-start-paid-stack-reset","seed":seed,"steps":steps,"rejectedCommands":rejected_commands,"syntheticCardRegistry":true,"syntheticInitialLayout":false,"realPrintedSocietiesOpened":false})
}
