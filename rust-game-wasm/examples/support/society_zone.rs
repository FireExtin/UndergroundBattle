//! Synthetic initial layouts for real printed society attachments, not browser games.
//! All changes after the initial layout use the shared Room ABI recording harness.
use super::*;

fn case(definition: &str, with_society: bool) -> Value {
    let mut game = attachment_initial(
        "teams",
        &format!("society-zone-{definition}-{with_society}"),
    );
    if with_society {
        game.players[0].society_zone.card = Some(game.make_card("MSJC01", 0));
    }
    for asset in ["JC031", "JC063"] {
        for _ in 0..3 {
            let card = game.make_card(asset, 0);
            game.players[0].assets.push(card);
        }
    }
    let source = game.make_card(definition, 0);
    let source_id = source.id.clone();
    game.players[0].hand.push(source);
    let mut room = RoomEnvelope::from_game(game);
    let initial = serde_json::to_string(&room).unwrap();
    room = RoomEnvelope::from_persisted(&initial).unwrap();
    let societies = serde_json::to_value(
        room.players
            .iter()
            .map(|p| &p.society_zone)
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let mut steps = vec![step(&room, "initialFixture", json!([initial]), 0)];
    let offers = room
        .game
        .legal_actions(0)
        .into_iter()
        .filter(|a| a.action.kind == "play" && a.action.card_id.as_deref() == Some(&source_id))
        .collect::<Vec<_>>();
    assert_eq!(offers.len(), 1);
    let play = offers[0].action.clone();
    assert_eq!(play.target_id.as_deref(), Some("society:p0"));
    assert_eq!(room.team(0), room.team(1));
    if definition == "JZ29" {
        assert!(!room.game.legal_actions(0).iter().any(|a| {
            a.action.kind == "asset" && a.action.card_id.as_deref() == Some(&source_id)
        }));
    }

    // A teammate's zone is not this actor's zone; numeric aliases are invalid.
    for target in ["society:p1", "society:p00"] {
        let command = RoomCommand {
            command_id: format!("society-zone:invalid:{}", steps.len()),
            expected_version: room.revision,
            action: Action {
                target_id: Some(target.into()),
                ..play.clone()
            }
            .into(),
        };
        let now = room.pacing.last_server_now_ms;
        let transition = room.transition(0, Some(command.clone()), now).unwrap();
        assert_eq!(transition.error_code.as_deref(), Some("invalid_action"));
        assert!(!transition.changed);
        record_transition(
            &mut room,
            &mut steps,
            "applyRoom",
            json!([0, command, now.to_string()]),
            transition,
        );
        assert!(room.players[0].assets.iter().all(|c| !c.exhausted));
        assert!(room.players[0].hand.iter().any(|c| c.id == source_id));
    }

    apply_game(&mut room, &mut steps, 0, play.clone());
    let paid_command: RoomCommand =
        serde_json::from_value(steps.last().unwrap()["args"][1].clone()).unwrap();
    let paid = catalog::card(definition).cost as usize;
    assert_eq!(
        room.players[0]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        paid
    );
    assert!(!room.players[0].hand.iter().any(|c| c.id == source_id));
    assert_eq!(room.stack.len(), 1);
    assert!(room.pacing.window.is_some());

    // Repeat the original paid command at the restored paid checkpoint.
    let now = room.pacing.last_server_now_ms;
    let duplicate = room.transition(0, Some(paid_command.clone()), now).unwrap();
    assert_eq!(duplicate.error_code.as_deref(), Some("version_conflict"));
    assert!(!duplicate.changed);
    record_transition(
        &mut room,
        &mut steps,
        "applyRoom",
        json!([0, paid_command, now.to_string()]),
        duplicate,
    );
    assert_eq!(
        room.players[0]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        paid
    );
    assert_eq!(room.stack.len(), 1);

    attachment_pass_top(&mut room, &mut steps);
    if let Some(pending) = room.pending.clone() {
        assert_eq!(definition, "JZ29");
        assert_eq!(pending.choice.kind, "trigger");
        assert_eq!(pending.choice.min, Some(0));
        apply_game(
            &mut room,
            &mut steps,
            pending.seat,
            Action {
                choice_id: Some(pending.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        );
    }
    assert!(room.stack.is_empty() && room.pending.is_none());
    assert_eq!(room.attachments.len(), 1);
    assert_eq!(room.attachments[0].card.definition, definition);
    assert_eq!(room.attachments[0].card.controller, 0);
    assert_eq!(room.attachments[0].host_id, "society:p0");
    assert_eq!(
        room.players[0]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        paid
    );
    assert_eq!(
        serde_json::to_value(
            room.players
                .iter()
                .map(|p| &p.society_zone)
                .collect::<Vec<_>>()
        )
        .unwrap(),
        societies
    );
    for seat in 0..4 {
        let view = room.view(seat, room.pacing.last_server_now_ms).game;
        assert_eq!(view.society_zones.len(), 4);
        assert_eq!(view.society_zones[0].card.is_some(), with_society);
        assert!(view.society_zones.iter().skip(1).all(|z| z.card.is_none()));
        assert_eq!(view.attachments.len(), 1);
        assert_eq!(view.attachments[0].host_id, "society:p0");
        assert!(view.attachments[0].card.region.is_none());
    }
    json!({
        "name":format!("society-zone-{definition}-{}-paid-attachment", if with_society { "with-card" } else { "empty" }),
        "seed":"9007199254740993", "steps":steps,
        "syntheticInitialLayout":true, "syntheticCardRegistry":false,
        "publicNaturalUiAcceptance":false,
        "societyCardSelected":with_society,
    })
}

pub fn cases() -> impl Iterator<Item = Value> {
    ["JC033", "JC034", "JZ29", "JC068"]
        .into_iter()
        .flat_map(|definition| {
            [false, true]
                .into_iter()
                .map(move |with_society| case(definition, with_society))
        })
}
