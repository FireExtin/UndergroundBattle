//! Independent narrow review of 59ac43d..932de1d. These are native fixture
//! checks, not a browser match or the author's full grant/confrontation matrix.
use crate::{model::*, room::*, unit_support as u};

fn both_entries(g: &Game) -> (Game, RoomEnvelope) {
    // These cases deliberately have no active response stack. A real Room
    // response must use its intent protocol; from_game alone cannot make one.
    assert!(g.stack.is_empty());
    let raw = serde_json::to_string(g).unwrap();
    let game = Game::from_persisted(&raw).unwrap();
    let room = RoomEnvelope::from_game(g.clone());
    let room_raw = serde_json::to_string(&room).unwrap();
    let room = RoomEnvelope::from_persisted(&room_raw).unwrap();
    assert_eq!(serde_json::to_string(&game).unwrap(), raw);
    assert_eq!(serde_json::to_string(&room.game).unwrap(), raw);
    for seat in 0..4 {
        let expected = serde_json::to_value(g.view(seat)).unwrap();
        assert_eq!(serde_json::to_value(game.view(seat)).unwrap(), expected);
        assert_eq!(serde_json::to_value(room.view(seat, 0).game).unwrap(), expected);
    }
    (game, room)
}

fn command_both(g: &Game, seat: usize, action: Action, accepted: bool) -> Game {
    let (mut game, room) = both_entries(g);
    let before_game = serde_json::to_string(&game).unwrap();
    let before_room = serde_json::to_string(&room).unwrap();
    let result = game.apply(seat, action.clone());
    let transition = room.transition(seat, Some(RoomCommand {
        command_id: format!("independent-delta-{}", room.revision),
        expected_version: room.revision,
        action: action.into(),
    }), 0).unwrap();
    assert_eq!(result.is_ok(), accepted, "Game result={result:?}");
    assert_eq!(transition.error_code.is_none(), accepted,
        "Room error={:?}", transition.error_message);
    assert_eq!(transition.changed, accepted);
    if !accepted {
        assert_eq!(serde_json::to_string(&game).unwrap(), before_game);
        assert_eq!(transition.state, before_room);
        assert!(transition.journal.is_empty());
    }
    let room_after = RoomEnvelope::from_persisted(&transition.state).unwrap();
    assert_eq!(serde_json::to_value(&room_after.game).unwrap(), serde_json::to_value(&game).unwrap());
    for s in 0..4 {
        assert_eq!(serde_json::to_value(room_after.view(s, 0).game).unwrap(), serde_json::to_value(game.view(s)).unwrap());
    }
    game
}

#[test]
fn independent_abi94_delta_cross_card_keys_are_not_public_activations() {
    for print in ["BQ037", "CQ07", "JZ26", "BQ026"] {
        let mut g = u::game();
        let source = u::field(&mut g, print, 0);
        for key in ["mobility", "renown", "jc089-combat-glory"] {
            assert!(!g.legal_actions(0).iter().any(|a|
                a.action.card_id.as_deref() == Some(source.as_str())
                && a.action.ability_id.as_deref() == Some(key)));
            command_both(&g, 0, Action {
                card_id: Some(source.clone()), ability_id: Some(key.into()),
                target_id: (key == "mobility").then(|| "region:1".into()),
                ..Action::new("activate")
            }, false);
        }
    }
}

fn native_reward_pending(print: &str, glory: bool) -> Game {
    // Explicit current grant/layout fixture. Producers and subsequent choices
    // are real engine paths; obtaining the grant via paid commands is covered
    // in the author's integration matrix, not asserted by this test.
    let mut g = u::game();
    let host = u::field(&mut g, print, 0);
    for _ in 0..2 { u::field(&mut g, "JC059", 0); }
    if glory {
        let card = g.make_card("JC089", 0);
        g.attachments.push(Attachment { card, host_id: host.clone() });
        g.begin_window(Window::After(0, 1));
        g.reward(0, 0, 1, 1).unwrap();
    } else {
        g.turn_attribute_modifiers.push(TurnAttributeModifier {
            target_instance: host.clone(), defense_bonus: 0, kill_bonus: 0,
            grants_retreat: false, printed_defense_override: None,
            ordinary_icons: Icons::default(), grants_renown: true,
            grants_barrier: false, prevents_damage: false, expires_turn: g.turn,
        });
        g.begin_window(Window::After(0, 2));
        g.complete_influence_contest(0);
    }
    let key = if glory { "jc089-combat-glory" } else { "renown" };
    for _ in 0..12 {
        g.drive().unwrap();
        if let Some(p) = &g.pending {
            if matches!(&p.resolution, ChoiceResolution::Declare { declaration, .. }
                if declaration.ability.key == key && declaration.source.card.id == host) {
                return g;
            }
            u::resolve_choice(&mut g);
        }
    }
    panic!("no real reward producer result for {print}/{key}");
}

#[test]
fn independent_abi94_delta_decline_is_exactly_once_after_both_restore_entries() {
    for print in ["BQ037", "CQ07"] { for glory in [false, true] {
        let g = native_reward_pending(print, glory);
        let p = g.pending.as_ref().unwrap();
        let before = g.regions.iter().map(|r| r.influence).collect::<Vec<_>>();
        let decline = Action {
            choice_id: Some(p.choice.id.clone()), selected: Some(vec![]),
            ..Action::new("choose")
        };
        // Ownership and cardinality must remain authoritative after recovery.
        command_both(&g, (p.seat + 1) % 4, decline.clone(), false);
        command_both(&g, p.seat, Action {
            selected: Some(vec!["accept".into(), "accept".into()]), ..decline.clone()
        }, false);
        let after = command_both(&g, p.seat, decline.clone(), true);
        assert_eq!(after.regions.iter().map(|r| r.influence).collect::<Vec<_>>(), before);
        assert!(after.stack.is_empty());
        assert!(after.pending.as_ref().is_none_or(|now| now.choice.id != p.choice.id));
        assert_eq!(after.version, g.version + 1);
        command_both(&after, p.seat, decline.clone(), false);
        command_both(&after, p.seat, Action {
            selected: Some(vec!["accept".into()]), ..decline
        }, false);
    } }
}
