//! Printed MSJC09; all setup/payment/pass transitions are normal room commands.
use super::*;

pub(super) fn natural_case() -> Value {
    let seed = "18446744073709551615";
    let draft = |society: bool| {
        let mut d = deck::preset("watchers").unwrap();
        d.id = "printed-msjc09-deck".into();
        d.name = "MSJC09机制验证".into();
        d.cards = vec![catalog::DeckEntry {
            card_id: "JC125".into(),
            count: 50,
        }];
        d.society_id = society.then(|| "MSJC09".into());
        d
    };
    let d = draft(true);
    let mut room = RoomEnvelope::from_game(
        Game::new_with_deck(
            "888888888888888888880009".into(),
            "MSJC09".into(),
            "teams".into(),
            "P0".into(),
            d.clone(),
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
            serde_json::to_string(&d).unwrap(),
            seed
        ]),
        0,
    )];
    for seat in 1..4 {
        let d = draft(seat == 2);
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
    }
    for seat in 0..4 {
        apply_game(&mut room, &mut steps, seat, Action::new("ready"));
    }
    apply_game(&mut room, &mut steps, 0, Action::new("start"));
    assert!(room
        .players
        .iter()
        .all(|p| p.hand.len() == 6 && p.deck.len() == 44));
    let sources: [String; 2] = [0, 2].map(|seat| {
        room.players[seat]
            .society_zone
            .card
            .as_ref()
            .unwrap()
            .id
            .clone()
    });
    let mut paid = [false; 2];
    let mut resolved = [false; 2];
    let mut first_rear = [false; 2];
    let mut hand_before = [0; 2];
    let mut expected_draw = [0; 2];
    for _ in 0..2000 {
        if paid.iter().all(|x| *x) && room.turn >= 4 {
            break;
        }
        if room.pending.is_some() {
            let (seat, a) = pick_choice(&room.game);
            apply_game(&mut room, &mut steps, seat, a);
            continue;
        }
        for (index, seat) in [0, 2].into_iter().enumerate() {
            if paid[index] && !resolved[index] && room.stack.is_empty() {
                assert_eq!(
                    room.players[seat].hand.len(),
                    hand_before[index] + expected_draw[index]
                );
                assert!(room.players[seat]
                    .assets
                    .iter()
                    .take(3)
                    .all(|c| c.exhausted));
                resolved[index] = true;
            }
        }
        let legal: Vec<_> = (0..4)
            .map(|seat| {
                room.view(seat, room.pacing.last_server_now_ms)
                    .game
                    .legal_actions
            })
            .collect();
        let mut acted = false;
        for (index, seat) in [0, 2].into_iter().enumerate() {
            if paid[index] {
                continue;
            }
            if let Some(a) = legal[seat].iter().find(|a| {
                a.action.kind == "activate"
                    && a.action.card_id.as_deref() == Some(sources[index].as_str())
            }) {
                assert_eq!(
                    a.source_zone_id.as_deref(),
                    Some(format!("society:p{seat}").as_str())
                );
                hand_before[index] = room.players[seat].hand.len();
                expected_draw[index] = usize::from(room.game.team(seat) == room.first_team);
                first_rear[expected_draw[index]] = true;
                apply_game(&mut room, &mut steps, seat, a.action.clone());
                paid[index] = true;
                acted = true;
                assert_eq!(room.players[seat].hand.len(), hand_before[index]);
                assert!(
                    room.players[seat]
                        .society_zone
                        .card
                        .as_ref()
                        .unwrap()
                        .exhausted
                );
                assert_eq!(
                    room.players[seat]
                        .assets
                        .iter()
                        .filter(|c| c.exhausted)
                        .count(),
                    3
                );
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
                break;
            }
            if let Some(a) = legal[seat].iter().find(|a| a.action.kind == "asset") {
                apply_game(&mut room, &mut steps, seat, a.action.clone());
                acted = true;
                break;
            }
        }
        if acted {
            continue;
        }
        let (seat, a) = legal
            .iter()
            .enumerate()
            .find_map(|(seat, list)| {
                list.iter()
                    .find(|a| a.action.kind == "pass")
                    .map(|a| (seat, a.action.clone()))
            })
            .expect("normal bounded pass available");
        apply_game(&mut room, &mut steps, seat, a);
    }
    assert!(
        paid.iter().all(|x| *x)
            && resolved.iter().all(|x| *x)
            && first_rear.iter().all(|x| *x)
            && room.turn >= 4
    );
    for (index, seat) in [0, 2].into_iter().enumerate() {
        let s = room.players[seat].society_zone.card.as_ref().unwrap();
        assert_eq!(s.id, sources[index]);
        assert!(!s.exhausted);
    }
    for seat in 0..4 {
        let v = room.view(seat, room.pacing.last_server_now_ms);
        assert!(v.hand.iter().all(|c| c.owner == format!("p{seat}")));
        assert_eq!(
            v.society_zones.iter().filter(|z| z.card.is_some()).count(),
            2
        );
        assert!(v
            .society_zones
            .iter()
            .filter_map(|z| z.card.as_ref())
            .all(|c| !c.face_down && c.cost.is_none() && c.region.is_none()));
    }
    let denied = Action {
        card_id: Some(sources[0].clone()),
        ability_id: Some("drawWithInitiative".into()),
        ..Action::new("activate")
    };
    json!({"name":"printed-MSJC09-normal-four-seat-first-and-rear-paid-resolution-reset","seed":seed,"steps":steps,"rejectedCommands":[rejected(&room,1,denied)],"syntheticInitialLayout":false,"syntheticCardRegistry":false,"realPrintedSocieties":["MSJC09"],"firstAndRearBothActivated":true,"sameInstanceNextTurnReset":true})
}
