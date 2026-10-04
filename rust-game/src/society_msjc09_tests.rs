//! Printed MSJC09 regressions. Layout mutations here are explicit unit fixtures,
//! separate from normal-command WASM and HTTP/Worker scenarios.
use super::*;
use crate::{
    catalog, deck,
    room::{RoomCommand, RoomEnvelope, SessionAction},
    rules,
};

fn draft(society: Option<&str>) -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.cards = vec![catalog::DeckEntry {
        card_id: "JC125".into(),
        count: 50,
    }];
    d.society_id = society.map(str::to_owned);
    d
}
fn lobby() -> Game {
    let mut g = Game::new_with_deck(
        "printed-msjc09".into(),
        "invite".into(),
        "teams".into(),
        "A".into(),
        draft(Some("MSJC09")),
        9,
    )
    .unwrap();
    for (name, society) in [("B", None), ("C", Some("MSJC09")), ("D", None)] {
        g.join_with_deck(name.into(), draft(society)).unwrap();
    }
    g
}
fn started() -> Game {
    let mut g = lobby();
    for seat in 0..4 {
        g.apply(seat, Action::new("ready")).unwrap();
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
    g
}
fn activation(g: &Game, seat: usize) -> Action {
    Action {
        card_id: Some(
            g.players[seat]
                .society_zone
                .card
                .as_ref()
                .unwrap()
                .id
                .clone(),
        ),
        ability_id: Some("drawWithInitiative".into()),
        ..Action::new("activate")
    }
}
fn fund(g: &mut Game, seat: usize, n: usize) {
    let team = g.team(seat);
    g.window = Some(Window::Action(team));
    g.active_team = team;
    g.priority_team = team;
    g.passed.clear();
    for _ in 0..n {
        let c = g.make_card("JC125", seat);
        g.players[seat].assets.push(c);
    }
}
fn settle(g: &mut Game) {
    for _ in 0..20 {
        if g.stack.is_empty() {
            return;
        }
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .expect("response pass available");
        g.apply(seat, Action::new("pass")).unwrap();
    }
    panic!("bounded paid frame did not resolve");
}
fn rejects_without_mutation(g: &mut Game, seat: usize, a: Action) {
    let before = serde_json::to_string(g).unwrap();
    assert!(g.apply(seat, a).is_err());
    assert_eq!(serde_json::to_string(g).unwrap(), before);
}

#[test]
fn msjc09_printed_registry_deck_and_atomic_start_keep_four_seat_privacy() {
    let c = catalog::catalog();
    assert_eq!(c.cards.len(), 67);
    assert_eq!(c.societies.len(), 2);
    assert!(c.deck_build_rules.society_supported);
    let s = &c.societies[0];
    assert_eq!(
        (&*s.card.id, &*s.card.name, &*s.subtitle, &*s.card.color),
        ("MSJC09", "秘社", "未知的聚会", "中立")
    );
    assert_eq!(s.starting_hand, 6);
    assert!(s.printed_cost.is_none());
    assert!(s.card.unique);
    assert!(s.deck_constraints.is_empty() && s.unresolved_abilities.is_empty());
    assert_eq!(
        s.card.text,
        "行动3，横置：若你具有【先手标志】，则抓一张牌。"
    );
    let spec = &rules::definition("MSJC09").abilities[0];
    assert_eq!(spec.timing, rules::Timing::Standard);
    assert!(spec.targets.is_empty());
    assert_eq!(
        serde_json::to_value(&spec.costs).unwrap(),
        serde_json::json!([{"Assets":3},"ExhaustSource"])
    );
    for closed in ["MSJC16", "FIXTURE_SOCIETY_SIX"] {
        assert!(deck::validate(draft(Some(closed))).is_err());
    }
    let mut small = draft(Some("MSJC09"));
    small.cards[0].count = 49;
    assert!(deck::validate(small).is_err());
    let mut stuffed = draft(None);
    stuffed.cards.push(catalog::DeckEntry {
        card_id: "MSJC09".into(),
        count: 1,
    });
    assert!(deck::validate(stuffed).is_err());
    let mut g = lobby();
    for seat in 0..4 {
        let v = g.view(seat);
        assert!(v.society_zones.iter().all(|z| z.card.is_none()));
        assert_eq!(
            v.your_deck.unwrap().society_id,
            if seat == 0 || seat == 2 {
                Some("MSJC09".into())
            } else {
                None
            }
        );
        g.apply(seat, Action::new("ready")).unwrap();
    }
    g.apply(0, Action::new("start")).unwrap();
    assert_eq!(
        g.players.iter().map(|p| p.hand.len()).collect::<Vec<_>>(),
        vec![6; 4]
    );
    assert_eq!(
        g.players.iter().map(|p| p.deck.len()).collect::<Vec<_>>(),
        vec![44; 4]
    );
    for seat in 0..4 {
        let v = g.view(seat);
        assert_eq!(
            v.society_zones.iter().filter(|z| z.card.is_some()).count(),
            2
        );
        assert!(v
            .society_zones
            .iter()
            .filter_map(|z| z.card.as_ref())
            .all(|card| card.card_id.as_deref() == Some("MSJC09")
                && !card.face_down
                && card.region.is_none()
                && card.cost.is_none()));
        assert!(v.hand.iter().all(|card| card.owner == player_id(seat)));
        assert_eq!(
            v.pending_choice.is_some(),
            seat == g.pending.as_ref().unwrap().seat
        );
        let json = serde_json::to_value(v).unwrap();
        assert!(json
            .get("players")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p.get("societyId").is_none()));
    }
}

#[test]
fn msjc09_first_and_rear_may_pay_but_only_resolution_time_initiative_draws() {
    // All four declaration/resolution combinations detect both an activation
    // gate and capturing the condition too early. Changing initiative here is
    // a unit fixture, not a claim of a currently playable initiative response.
    for declared_first in [0, 1] {
        for resolved_first in [0, 1] {
            let mut g = started();
            g.first_team = declared_first;
            fund(&mut g, 0, 3);
            let a = activation(&g, 0);
            let id = a.card_id.clone().unwrap();
            let before = g.players[0].hand.len();
            let legal = g
                .legal_actions(0)
                .into_iter()
                .find(|l| l.action == a)
                .expect("initiative is not an activation gate");
            assert_eq!(legal.source_zone_id.as_deref(), Some("society:p0"));
            assert!(legal.action.region.is_none());
            g.apply(0, a.clone()).unwrap();
            assert_eq!(g.players[0].hand.len(), before);
            assert_eq!(
                g.players[0].assets.iter().filter(|c| c.exhausted).count(),
                3
            );
            assert!(g.society_card(&id).unwrap().exhausted);
            let frame = g.stack[0].frame.as_ref().unwrap();
            assert_eq!(frame.actor, 0);
            assert!(frame.source.region.is_none());
            assert_eq!(frame.already_paid.len(), 2);
            let serialized = serde_json::to_string(&g).unwrap();
            assert!(serialized.contains("DrawIfActorHasInitiative"));
            g = Game::from_persisted(&serialized).unwrap();
            g.first_team = resolved_first;
            settle(&mut g);
            assert_eq!(
                g.players[0].hand.len(),
                before + usize::from(resolved_first == 0)
            );
            assert!(g.society_card(&id).unwrap().exhausted);
            assert!(g.players[0].assets.iter().all(|c| c.exhausted));
            rejects_without_mutation(&mut g, 0, a);
        }
    }
}

#[test]
fn msjc09_both_members_of_the_first_team_each_pay_their_own_society_and_draw() {
    // Hegemony guide physical p22 / printed p21: teammates jointly hold initiative.
    for first_team in [0, 1] {
        let mut g = lobby();
        for seat in [1, 3] {
            g.apply(
                seat,
                Action {
                    deck_draft: Some(draft(Some("MSJC09"))),
                    ..Action::new("deck")
                },
            )
            .unwrap();
        }
        for seat in 0..4 {
            g.apply(seat, Action::new("ready")).unwrap();
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
        g.first_team = first_team;
        let actors = [first_team * 2, first_team * 2 + 1];
        for actor in actors {
            fund(&mut g, actor, 3);
        }
        let initial_hands: Vec<_> = g.players.iter().map(|p| p.hand.len()).collect();
        for actor in actors {
            let action = activation(&g, actor);
            let id = action.card_id.clone().unwrap();
            let teammate = actor ^ 1;
            assert!(g.legal_actions(actor).iter().any(|a| a.action == action));
            rejects_without_mutation(&mut g, teammate, action.clone());
            g.apply(actor, action).unwrap();
            settle(&mut g);
            assert_eq!(g.players[actor].hand.len(), initial_hands[actor] + 1);
            assert!(g.players[actor].assets.iter().all(|c| c.exhausted));
            assert!(g.society_card(&id).unwrap().exhausted);
            assert_eq!(g.players[actor].assets.len(), 3);
        }
        for seat in 0..4 {
            assert_eq!(
                g.players[seat].hand.len(),
                initial_hands[seat] + usize::from(actors.contains(&seat))
            );
            if !actors.contains(&seat) {
                assert!(g.players[seat].assets.is_empty());
                assert!(
                    !g.players[seat]
                        .society_zone
                        .card
                        .as_ref()
                        .unwrap()
                        .exhausted
                );
            }
        }
    }
}

#[test]
fn msjc09_wrong_seats_standard_timing_insufficient_assets_and_exhaustion_are_atomic() {
    let mut g = started();
    fund(&mut g, 0, 3);
    let a = activation(&g, 0);
    for seat in [1, 2, 3] {
        assert!(!g.legal_actions(seat).iter().any(|l| l.action == a));
        rejects_without_mutation(&mut g, seat, a.clone());
    }
    let mut short = started();
    fund(&mut short, 0, 2);
    let short_action = activation(&short, 0);
    rejects_without_mutation(&mut short, 0, short_action);
    let mut exhausted = started();
    fund(&mut exhausted, 0, 3);
    exhausted.players[0]
        .society_zone
        .card
        .as_mut()
        .unwrap()
        .exhausted = true;
    let exhausted_action = activation(&exhausted, 0);
    rejects_without_mutation(&mut exhausted, 0, exhausted_action);
    g.window = Some(Window::Prepare);
    rejects_without_mutation(&mut g, 0, a.clone());
    g.window = Some(Window::Action(0));
    g.apply(0, a.clone()).unwrap();
    g.players[0].society_zone.card.as_mut().unwrap().exhausted = false;
    fund(&mut g, 0, 3);
    rejects_without_mutation(&mut g, 0, a); // Nonempty stack still forbids standard activation.
}

#[test]
fn msjc09_paid_room_frame_journal_four_views_and_same_instance_reset_survive_restore() {
    let mut g = started();
    fund(&mut g, 0, 3);
    let a = activation(&g, 0);
    let id = a.card_id.clone().unwrap();
    let room = RoomEnvelope::from_game(g);
    let command = RoomCommand {
        command_id: "printed-paid".into(),
        expected_version: room.revision,
        action: SessionAction::Game { action: a },
    };
    let result = room.transition(0, Some(command), 1000).unwrap();
    assert_eq!(result.outcome, "accepted");
    assert_eq!(
        serde_json::to_string(&room.replay_events(&result.journal).unwrap()).unwrap(),
        result.state
    );
    let mut restored = RoomEnvelope::from_persisted(&result.state).unwrap();
    for seat in 0..4 {
        assert_eq!(
            serde_json::to_value(result.view.game.society_zones.clone()).unwrap(),
            serde_json::to_value(restored.game.view(seat).society_zones).unwrap()
        );
    }
    assert!(restored.game.board(&id).is_none());
    assert!(restored.game.remove_board(&id).is_none());
    restored.game.remove_dead(&id, RemovalCause::Destroy);
    restored.game.return_hand(&id);
    restored.game.to_bottom(&id);
    assert!(restored.game.society_card(&id).is_some());
    settle(&mut restored.game);
    restored.game.effects.push_back(Effect::NextTurn);
    restored.game.drive().unwrap();
    assert!(!restored.game.society_card(&id).unwrap().exhausted);
    assert!(restored.players[0].assets.iter().all(|c| !c.exhausted));
    // Direct Game passes above do not advance the surrounding Room revision.
    let second = Game::from_persisted(&serde_json::to_string(&restored.game).unwrap()).unwrap();
    assert!(second.society_card(&id).is_some());
}
