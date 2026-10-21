//! A player's headquarters exists whether or not their deck chose a society card.
//! Public commands and paid/live checkpoints must agree on that zone identity.
use crate::{
    catalog,
    model::*,
    room::{RoomCommand, RoomEnvelope},
    unit_support::*,
};

const ATTACHMENTS: [&str; 4] = ["JC033", "JC034", "JZ29", "JC068"];

fn layout(with_society: bool) -> Game {
    let mut g = if with_society {
        crate::msjc01_tests::game()
    } else {
        game()
    };
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    fund(&mut g, 0, "JC031", 3);
    fund(&mut g, 0, "JC063", 3);
    g
}

fn round_trip_room(g: &Game) {
    // These explicit native checkpoints use the shared valid pacing fixture.
    let room = crate::jc029_tests::envelope(g);
    let saved = serde_json::to_string(&room).unwrap();
    let restored = RoomEnvelope::from_persisted(&saved).unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), saved);
    for seat in 0..4 {
        assert_eq!(
            serde_json::to_value(restored.game.view(seat)).unwrap(),
            serde_json::to_value(g.view(seat)).unwrap()
        );
    }
}

fn play_in_room(g: &mut Game, action: Action) {
    restore(g);
    let room = RoomEnvelope::from_game(g.clone());
    let command = RoomCommand {
        command_id: format!("society-zone-play-{}", g.version),
        expected_version: room.revision,
        action: action.into(),
    };
    let paid = room.transition(0, Some(command.clone()), 1).unwrap();
    assert!(paid.error_code.is_none(), "{:?}", paid.error_message);
    assert!(paid.changed);
    assert_eq!(
        serde_json::to_string(&room.replay_events(&paid.journal).unwrap()).unwrap(),
        paid.state
    );
    let restored = RoomEnvelope::from_persisted(&paid.state).unwrap();
    assert!(restored.pacing.window.is_some());
    let duplicate = restored.transition(0, Some(command), 2).unwrap();
    assert_eq!(duplicate.error_code.as_deref(), Some("version_conflict"));
    assert!(!duplicate.changed);
    assert_eq!(duplicate.state, paid.state);
    *g = restored.game;
    restore(g);
}

fn own_play(g: &Game, seat: usize, id: &str) -> Action {
    let plays = g
        .legal_actions(seat)
        .into_iter()
        .filter(|a| a.action.kind == "play" && a.action.card_id.as_deref() == Some(id))
        .collect::<Vec<_>>();
    assert_eq!(plays.len(), 1, "one own headquarters target for {id}");
    assert_eq!(plays[0].action.target_id, Some(format!("society:p{seat}")));
    plays[0].action.clone()
}

fn paid_attachment(definition: &str) {
    for with_society in [false, true] {
        let mut g = layout(with_society);
        let societies = g
            .players
            .iter()
            .map(|p| serde_json::to_value(&p.society_zone).unwrap())
            .collect::<Vec<_>>();
        let id = held(&mut g, definition, 0);
        restore(&mut g);
        let action = own_play(&g, 0, &id);
        if definition == "JZ29" {
            assert!(!g
                .legal_actions(0)
                .iter()
                .any(|a| { a.action.kind == "asset" && a.action.card_id.as_deref() == Some(&id) }));
        }
        let before = resources(&g, 0);
        play_in_room(&mut g, action.clone());
        assert_eq!(
            resources(&g, 0),
            before - catalog::card(definition).cost as usize
        );
        assert!(!g.players[0].hand.iter().any(|c| c.id == id));
        assert_eq!(g.stack.len(), 1);
        round_trip_room(&g); // paid declaration, before resolution
        reject(&mut g, 0, action); // duplicate command cannot pay twice
        drain(&mut g);
        assert_eq!(g.attachments.len(), 1);
        let attached = &g.attachments[0];
        let live_id = attached.card.id.clone();
        assert_eq!(attached.card.definition, definition);
        assert_eq!(attached.host_id, "society:p0");
        assert!(g.attachment_host_valid(attached));
        assert!(g.in_play(&live_id).is_some() && g.board(&live_id).is_none());
        for seat in 0..4 {
            let view = g.view(seat);
            assert_eq!(view.society_zones.len(), 4);
            assert_eq!(view.society_zones[0].card.is_some(), with_society);
            assert_eq!(view.attachments.len(), 1);
            assert_eq!(view.attachments[0].host_id, "society:p0");
            assert!(view.attachments[0].card.region.is_none());
        }
        assert_eq!(
            g.players
                .iter()
                .map(|p| { serde_json::to_value(&p.society_zone).unwrap() })
                .collect::<Vec<_>>(),
            societies,
            "attachment must not create/change a society card or its usage"
        );
        round_trip_room(&g);
        g.remove_dead(&live_id, RemovalCause::Destroy);
        assert!(g.attachments.is_empty());
        assert!(g.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == definition));
        assert_eq!(g.society_host_seat("society:p0"), Some(0));
        restore(&mut g);
        round_trip_room(&g);
    }
}

#[test]
fn society_zone_jc033_pays_attaches_restores_and_leaves_with_optional_society() {
    paid_attachment("JC033");
}
#[test]
fn society_zone_jc034_pays_attaches_restores_and_leaves_with_optional_society() {
    paid_attachment("JC034");
}
#[test]
fn society_zone_jz29_pays_attaches_restores_and_leaves_with_optional_society() {
    paid_attachment("JZ29");
}
#[test]
fn society_zone_jc068_pays_attaches_restores_and_leaves_with_optional_society() {
    paid_attachment("JC068");
}

#[test]
fn society_zone_rejects_foreign_and_noncanonical_targets_without_payment() {
    for definition in ATTACHMENTS {
        let mut g = layout(false);
        let id = held(&mut g, definition, 0);
        let action = own_play(&g, 0, &id);
        assert_eq!(g.team(0), g.team(1)); // teammate remains a different controller
        for target in [
            "society:p1",
            "society:p2",
            "society:p3",
            "society:p4",
            "society:p00",
            "society:p+0",
            "society:p-1",
            "society:p0 ",
            "p0",
            "",
        ] {
            reject(
                &mut g,
                0,
                Action {
                    target_id: Some(target.into()),
                    ..action.clone()
                },
            );
        }
        assert_eq!(resources(&g, 0), 6);
        g.players[1].eliminated = true;
        assert_eq!(g.society_host_seat("society:p1"), None);
        reject(
            &mut g,
            0,
            Action {
                target_id: Some("society:p1".into()),
                ..action.clone()
            },
        );
        g.players[0].eliminated = true;
        assert_eq!(g.society_host_seat("society:p0"), None);
        assert!(g.legal_actions(0).is_empty());
        reject(&mut g, 0, action);
    }
}

#[test]
fn society_zone_uses_the_acting_seat_and_never_a_teammates_headquarters() {
    for definition in ATTACHMENTS {
        let mut g = game();
        fund(&mut g, 1, "JC031", 3);
        fund(&mut g, 1, "JC063", 3);
        let id = held(&mut g, definition, 1);
        let action = own_play(&g, 1, &id);
        reject(
            &mut g,
            1,
            Action {
                target_id: Some("society:p0".into()),
                ..action.clone()
            },
        );
        act(&mut g, 1, action);
        drain(&mut g);
        assert_eq!(g.attachments[0].host_id, "society:p1");
        assert_eq!(g.attachments[0].card.controller, 1);
        round_trip_room(&g);
    }
}

#[test]
fn society_zone_paid_and_live_hosts_survive_optional_card_absence() {
    for definition in ATTACHMENTS {
        let mut g = layout(true);
        let id = held(&mut g, definition, 0);
        let action = own_play(&g, 0, &id);
        act(&mut g, 0, action);
        let society = g.players[0].society_zone.card.take().unwrap();
        assert!(g.society_card(&society.id).is_none());
        assert!(g.society_usage(&society.id).is_none());
        assert!(g.ability_source(&society.id).is_none());
        restore(&mut g);
        round_trip_room(&g);
        drain(&mut g);
        assert_eq!(g.attachments.len(), 1);
        g.settle_attachments();
        assert_eq!(g.attachments.len(), 1);
        assert!(g.players[0].society_zone.card.is_none());
        restore(&mut g);
    }
}

#[test]
fn society_zone_empty_jc033_keeps_paid_ability_source_and_private_search_after_sacrifice() {
    let mut g = layout(false);
    let id = held(&mut g, "JC033", 0);
    let action = own_play(&g, 0, &id);
    act(&mut g, 0, action);
    drain(&mut g);
    let live_id = g.attachments[0].card.id.clone();
    let eligible = deck_cards(&mut g, 0, &["JC031", "JC125"])[0].clone();
    g.begin_window(Window::Action(0));
    let ability = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.card_id.as_deref() == Some(&live_id)
                && a.action.ability_id.as_deref() == Some("batch20-jc033-search")
                && a.action.region == Some(1)
        })
        .unwrap();
    assert_eq!(ability.source_zone_id.as_deref(), Some("society:p0"));
    let before = resources(&g, 0);
    act(&mut g, 0, ability.action);
    assert_eq!(resources(&g, 0), before - 1);
    assert!(g.attachments.is_empty());
    while g.pending.is_none() {
        pass(&mut g);
    }
    round_trip_room(&g);
    for seat in 1..4 {
        assert!(g.view(seat).pending_choice.is_none());
    }
    choose(&mut g, vec![eligible]);
    assert!(g.regions[1]
        .cards
        .iter()
        .any(|c| c.definition == "JC031" && c.face_down));
    assert!(g.players[0].society_zone.card.is_none());
    restore(&mut g);
}

#[test]
fn society_zone_empty_jc068_restores_forced_upkeep_and_conceal_control() {
    let mut g = layout(false);
    let id = held(&mut g, "JC068", 0);
    let action = own_play(&g, 0, &id);
    act(&mut g, 0, action);
    drain(&mut g);
    let live_id = g.attachments[0].card.id.clone();
    assert!(!g.jc068_blocks_conceal(0) && !g.jc068_blocks_conceal(1));
    assert!(g.jc068_blocks_conceal(2) && g.jc068_blocks_conceal(3));
    g.players[0].assets.clear();
    g.jc035_preparation = None;
    g.begin_window(Window::Prepare);
    g.drive().unwrap();
    restore(&mut g);
    round_trip_room(&g);
    choose(&mut g, vec![live_id.clone()]);
    top(&mut g);
    assert!(g.in_play(&live_id).unwrap().1.exhausted);
    assert!(!g.jc068_blocks_conceal(2));
    assert!(g.players[0].society_zone.card.is_none());
    round_trip_room(&g);
}
