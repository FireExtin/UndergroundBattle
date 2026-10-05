//! Whole MSJC08 oracle. Real purple/gold searches without program/filter changes.
//! Boundary layouts and cancelled/new-instance/finished checkpoints are disclosed.
use super::*;
const SEARCH: &str = "search-purple-unique";
const DRAW: &str = "drawWithInitiative";
fn draft(n: usize, total: usize) -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.id = "msjc08-oracle".into();
    d.name = "梦境行者25紫构筑".into();
    d.society_id = Some("MSJC08".into());
    d.cards.clear();
    let mut left = n;
    for id in [
        "JC104", "JC102", "JZ67", "JC103", "JC107", "JZ59", "JZ58", "JZ61", "XQ43",
    ] {
        let count = left.min(3);
        if count > 0 {
            d.cards.push(catalog::DeckEntry {
                card_id: id.into(),
                count,
            });
        }
        left -= count;
    }
    assert_eq!(left, 0);
    if total > n {
        d.cards.push(catalog::DeckEntry {
            card_id: "JC125".into(),
            count: total - n,
        });
    }
    d
}
fn source(g: &Game, s: usize) -> String {
    g.players[s].society_zone.card.as_ref().unwrap().id.clone()
}
fn activate(g: &Game, s: usize, key: &str) -> Action {
    Action {
        card_id: Some(source(g, s)),
        ability_id: Some(key.into()),
        ..Action::new("activate")
    }
}
fn fund(g: &mut Game, s: usize, n: usize) {
    for _ in 0..n {
        let c = g.make_card("JC125", s);
        g.players[s].assets.push(c);
    }
}
fn pass(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    let (s, a) = (0..4)
        .find_map(|s| {
            r.game
                .legal_actions(s)
                .into_iter()
                .find(|a| a.action.kind == "pass")
                .map(|a| (s, a.action))
        })
        .unwrap();
    apply_game(r, steps, s, a);
}
fn advance(r: &mut RoomEnvelope, steps: &mut Vec<Value>, done: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..500 {
        if done(r) {
            return;
        }
        if r.pending.is_some() {
            let (s, a) = pick_choice(&r.game);
            apply_game(r, steps, s, a);
        } else {
            pass(r, steps);
        }
    }
    panic!("bounded MSJC08 Room progress");
}
fn deny(r: &mut RoomEnvelope, steps: &mut Vec<Value>, s: usize, a: Action) {
    let c = RoomCommand {
        command_id: format!("oracle:msjc08-reject:{}", steps.len()),
        expected_version: r.revision,
        action: a.into(),
    };
    let now = r.pacing.last_server_now_ms;
    let t = r.transition(s, Some(c.clone()), now).unwrap();
    assert_eq!(t.outcome, "rejected");
    assert!(!t.changed);
    record_transition(r, steps, "applyRoom", json!([s, c, now.to_string()]), t);
}
fn initial() -> Game {
    let mut g = Game::new_with_deck(
        "msjc08-boundary".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        draft(27, 50),
        9007199254740993,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), draft(27, 50)).unwrap();
    }
    for s in 0..4 {
        g.apply(s, Action::new("ready")).unwrap();
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
    for s in 0..4 {
        g.players[s].hand.clear();
        g.players[s].deck.clear();
        g.players[s].assets.clear();
        for _ in 0..12 {
            let c = g.make_card("JC125", s);
            g.players[s].deck.push(c);
        }
    }
    for r in &mut g.regions {
        r.cards.clear();
    }
    g.window = Some(Window::Action(0));
    g.active_team = 0;
    g.priority_team = 0;
    g.passed.clear();
    g.team_passed = [false; 2];
    g
}
fn checkpoint(r: RoomEnvelope) -> (RoomEnvelope, Vec<Value>) {
    let raw = serde_json::to_string(&r).unwrap();
    assert_eq!(
        serde_json::to_string(&RoomEnvelope::from_persisted(&raw).unwrap()).unwrap(),
        raw
    );
    let steps = vec![step(&r, "initialFixture", json!([raw]), 0)];
    (r, steps)
}
fn boundary(kind: &str) -> Value {
    let mut g = initial();
    let actor = if kind == "two-hits-seat3" { 3 } else { 0 };
    let team = usize::from(actor >= 2);
    g.window = Some(Window::Action(team));
    g.active_team = team;
    g.priority_team = team;
    fund(&mut g, actor, if kind == "reject-cost" { 3 } else { 8 });
    fund(&mut g, actor ^ 1, 8);
    if kind == "reject-exhausted" {
        g.players[actor]
            .society_zone
            .card
            .as_mut()
            .unwrap()
            .exhausted = true;
    }
    if kind == "empty-deck" {
        g.players[actor].deck.clear();
    }
    if kind == "draw-rear" {
        g.first_team = 1;
    }
    let hits = if kind == "one-hit" {
        1
    } else if kind == "two-hits-seat3" {
        2
    } else {
        0
    };
    if hits > 0 {
        for id in ["JC104", "LC23", "XQ43"] {
            let c = g.make_card(id, actor);
            g.players[actor].deck.push(c);
        }
        if hits == 2 {
            let c = g.make_card("XQ43", actor);
            g.players[actor].deck.push(c);
        }
        let c = g.make_card("XQ43", actor ^ 1);
        g.players[actor ^ 1].deck.push(c);
    }
    let paid_checkpoint = matches!(
        kind,
        "cancelled-paid-frame" | "new-instance-used" | "finished-restart"
    );
    let mut r = RoomEnvelope::from_game(g);
    if paid_checkpoint {
        let mut preparation = vec![];
        let a = activate(&r, actor, SEARCH);
        apply_game(&mut r, &mut preparation, actor, a);
        if kind == "cancelled-paid-frame" {
            r.game
                .stack
                .last_mut()
                .unwrap()
                .frame
                .as_mut()
                .unwrap()
                .guard = hegemony_server::model::GuardState::Cancelled;
        } else {
            attachment_pass_top(&mut r, &mut preparation);
            if kind == "new-instance-used" {
                let c = r.game.players[actor].society_zone.card.take().unwrap();
                let mut fresh = r.game.make_card(&c.definition, c.owner);
                fresh.exhausted = false;
                r.game.players[actor].society_zone.card = Some(fresh);
            } else {
                r.game.status = "finished".into();
            }
        }
    }
    let (mut r, mut steps) = checkpoint(r);
    if kind.starts_with("reject-") || kind == "new-instance-used" {
        let mut a = activate(&r, actor, SEARCH);
        if kind == "reject-foreign-source" {
            a.card_id = Some(source(&r, actor ^ 1));
        }
        if kind == "reject-yellow-key" {
            a.ability_id = Some("search-yellow-unique".into());
        }
        deny(&mut r, &mut steps, actor, a);
    } else if kind == "finished-restart" {
        apply_game(&mut r, &mut steps, actor, Action::new("restart"));
        assert!(r
            .players
            .iter()
            .all(|p| p.society_zone.used_once_per_game.is_empty() && p.hand.len() == 6));
    } else {
        let is_draw = kind.starts_with("draw-");
        if !paid_checkpoint {
            let a = activate(&r, actor, if is_draw { DRAW } else { SEARCH });
            apply_game(&mut r, &mut steps, actor, a);
        }
        assert_eq!(
            r.players[actor]
                .assets
                .iter()
                .filter(|c| c.exhausted)
                .count(),
            if is_draw { 3 } else { 4 }
        );
        assert!(r.players[actor ^ 1].assets.iter().all(|c| !c.exhausted));
        if !is_draw {
            assert!(r.players[actor]
                .society_zone
                .used_once_per_game
                .contains(SEARCH));
        }
        attachment_pass_top(&mut r, &mut steps);
        if hits > 0 {
            let p = r.pending.clone().unwrap();
            assert_eq!(p.seat, actor);
            assert_eq!(p.choice.options.len(), hits);
            for v in 0..4 {
                assert_eq!(
                    r.view(v, r.pacing.last_server_now_ms)
                        .pending_choice
                        .is_some(),
                    v == actor
                );
            }
            deny(
                &mut r,
                &mut steps,
                actor,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec![]),
                    ..Action::new("choose")
                },
            );
            deny(
                &mut r,
                &mut steps,
                actor ^ 1,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec![p.choice.options[0].id.clone()]),
                    ..Action::new("choose")
                },
            );
            let foreign = r.players[actor ^ 1].deck.last().unwrap().id.clone();
            deny(
                &mut r,
                &mut steps,
                actor,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec![foreign]),
                    ..Action::new("choose")
                },
            );
            let selected = p.choice.options[hits - 1].id.clone();
            let n = r.players[actor].deck.len();
            let rng = r.random;
            apply_game(
                &mut r,
                &mut steps,
                actor,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec![selected.clone()]),
                    ..Action::new("choose")
                },
            );
            let c = r.players[actor].hand.last().unwrap();
            assert_ne!(c.id, selected);
            assert_eq!((c.owner, c.controller), (actor, actor));
            assert!(
                catalog::card(&c.definition).color == "紫" && catalog::card(&c.definition).unique
            );
            assert_eq!(r.players[actor].deck.len(), n - 1);
            assert_ne!(r.random, rng);
            for v in 0..4 {
                assert!(r
                    .view(v, r.pacing.last_server_now_ms)
                    .game
                    .log
                    .iter()
                    .any(|e| e.text.contains("展示检索的")));
            }
        } else if is_draw {
            assert_eq!(
                r.players[actor].hand.len(),
                usize::from(kind == "draw-first")
            );
            assert!(r.players[actor].society_zone.used_once_per_game.is_empty());
        } else {
            assert!(r.players[actor].hand.is_empty() && !r.players[actor].eliminated);
        }
        if kind == "turn-reset-used" {
            let t = r.turn;
            advance(&mut r, &mut steps, |r| {
                r.turn > t && matches!(r.window, Some(Window::Action(0)))
            });
            assert!(!r.players[0].society_zone.card.as_ref().unwrap().exhausted);
            let a = activate(&r, 0, SEARCH);
            deny(&mut r, &mut steps, 0, a);
            assert!(r
                .game
                .legal_actions(0)
                .iter()
                .any(|a| a.action.ability_id.as_deref() == Some(DRAW)));
        }
    }
    json!({"name":format!("MSJC08-{kind}"),"seed":"9007199254740993","steps":steps,"syntheticInitialLayout":true,"syntheticPaidFrameCheckpoint":paid_checkpoint,"syntheticCancelledFrame":kind=="cancelled-paid-frame","syntheticNewSourceInstance":kind=="new-instance-used","syntheticFinishedGame":kind=="finished-restart","syntheticCardRegistry":false,"syntheticFilterChanged":false,"formalPurpleUniqueHits":hits,"publicNaturalUiAcceptance":false})
}
fn construction(n: usize) -> Value {
    let d = draft(n, 50);
    let seed = "18446744073709551615";
    let r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "msjc08-construction".into(),
            "LOCAL".into(),
            "teams".into(),
            "P0".into(),
            d.clone(),
            seed.parse().unwrap(),
        )
        .unwrap(),
    );
    let mut bad_copy = draft(27, 50);
    bad_copy.cards[0].count = 4;
    let mut society_in_deck = draft(25, 50);
    society_in_deck.cards.push(catalog::DeckEntry {
        card_id: "MSJC08".into(),
        count: 1,
    });
    let rejected=[draft(24,50),draft(25,49),bad_copy,society_in_deck].into_iter().map(|d|{
        let error=Game::new_with_deck("reject".into(),"LOCAL".into(),"teams".into(),"P0".into(),d.clone(),1).unwrap_err();json!({"args":["reject","LOCAL","teams","P0",serde_json::to_string(&d).unwrap(),"1"],"error":error})
    }).collect::<Vec<_>>();
    json!({"name":format!("MSJC08-construction-{n}-purple"),"seed":seed,"steps":[step(&r,"newGameWithDeck",json!([r.room_id,r.invite_code,"teams","P0",serde_json::to_string(&d).unwrap(),seed]),0)],"rejectedNewGameWithDeck":rejected,"syntheticInitialLayout":false,"publicNaturalUiAcceptance":false})
}
fn natural() -> Value {
    let seed = "18446744073709551615";
    let d = draft(27, 50);
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "msjc08-natural".into(),
            "LOCAL".into(),
            "teams".into(),
            "P0".into(),
            d.clone(),
            seed.parse().unwrap(),
        )
        .unwrap(),
    );
    let mut steps = vec![step(
        &r,
        "newGameWithDeck",
        json!([
            r.room_id,
            r.invite_code,
            "teams",
            "P0",
            serde_json::to_string(&d).unwrap(),
            seed
        ]),
        0,
    )];
    for s in 1..4 {
        r.game.join_with_deck(format!("P{s}"), d.clone()).unwrap();
        r.revision = r.game.version;
        steps.push(step(
            &r,
            "joinGameWithDeck",
            json!([format!("P{s}"), serde_json::to_string(&d).unwrap()]),
            s,
        ));
    }
    for s in 0..4 {
        apply_game(&mut r, &mut steps, s, Action::new("ready"));
    }
    apply_game(&mut r, &mut steps, 0, Action::new("start"));
    assert!(r.players.iter().all(|p| p.hand.len() == 6));
    assert_eq!(
        (0..4)
            .map(|s| source(&r, s))
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    let mut drawn = [false; 4];
    let mut draw_done = [false; 4];
    let mut searched = [false; 4];
    let mut search_done = [false; 4];
    let mut hand_before = [0; 4];
    let mut gain = [0; 4];
    let mut roles = [false; 2];
    let mut last_cast_turn = 0;
    let mut selections = vec![];
    for _ in 0..2500 {
        if let Some(p) = r.pending.clone() {
            if searched[p.seat] && !search_done[p.seat] {
                assert!(p.choice.options.iter().all(|o| r.players[p.seat]
                    .deck
                    .iter()
                    .find(|c| c.id == o.id)
                    .is_some_and(|c| catalog::card(&c.definition).color == "紫"
                        && catalog::card(&c.definition).unique)));
                for v in 0..4 {
                    assert_eq!(
                        r.view(v, r.pacing.last_server_now_ms)
                            .pending_choice
                            .is_some(),
                        v == p.seat
                    );
                }
                let id = p
                    .choice
                    .options
                    .first()
                    .expect("naturally remaining real purple unique")
                    .id
                    .clone();
                let c = r.players[p.seat]
                    .deck
                    .iter()
                    .find(|c| c.id == id)
                    .unwrap()
                    .clone();
                let deck_before = r.players[p.seat].deck.len();
                let rng = r.random;
                apply_game(
                    &mut r,
                    &mut steps,
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id),
                        selected: Some(vec![id.clone()]),
                        ..Action::new("choose")
                    },
                );
                let held = r.players[p.seat].hand.last().unwrap();
                assert_ne!(held.id, id);
                assert_eq!(held.definition, c.definition);
                assert_eq!((held.owner, held.controller), (p.seat, p.seat));
                assert_eq!(r.players[p.seat].deck.len(), deck_before - 1);
                assert_ne!(r.random, rng);
                selections.push(json!({"seat":p.seat,"cardId":c.definition,"oldInstanceId":id,"newInstanceId":held.id,"ownDeckSearched":true,"allFourViewsPrivateWhileChoosing":true}));
            } else {
                let (s, a) = pick_choice(&r.game);
                apply_game(&mut r, &mut steps, s, a);
            }
            continue;
        }
        if r.stack.is_empty() {
            for s in 0..4 {
                if drawn[s] && !draw_done[s] {
                    assert_eq!(r.players[s].hand.len(), hand_before[s] + gain[s]);
                    draw_done[s] = true;
                }
                if searched[s] && !search_done[s] {
                    assert_eq!(r.players[s].hand.len(), hand_before[s] + 1);
                    search_done[s] = true;
                }
            }
        }
        if search_done.iter().all(|x| *x)
            && r.turn > last_cast_turn
            && r.players
                .iter()
                .all(|p| !p.society_zone.card.as_ref().unwrap().exhausted)
        {
            break;
        }
        let legal = (0..4)
            .map(|s| r.view(s, r.pacing.last_server_now_ms).game.legal_actions)
            .collect::<Vec<_>>();
        let mut acted = false;
        for s in 0..4 {
            let key = if !drawn[s] { DRAW } else { SEARCH };
            if !searched[s] {
                if let Some(a) = legal[s]
                    .iter()
                    .find(|a| a.action.ability_id.as_deref() == Some(key))
                {
                    hand_before[s] = r.players[s].hand.len();
                    gain[s] = usize::from(r.team(s) == r.first_team);
                    let other_assets = r
                        .players
                        .iter()
                        .map(|p| serde_json::to_value(&p.assets).unwrap())
                        .collect::<Vec<_>>();
                    let ready = r.players[s].assets.iter().filter(|c| !c.exhausted).count();
                    apply_game(&mut r, &mut steps, s, a.action.clone());
                    assert_eq!(
                        r.players[s].assets.iter().filter(|c| !c.exhausted).count(),
                        ready - if key == DRAW { 3 } else { 4 }
                    );
                    for other in 0..4 {
                        if other != s {
                            assert_eq!(
                                serde_json::to_value(&r.players[other].assets).unwrap(),
                                other_assets[other]
                            );
                        }
                    }
                    if key == DRAW {
                        drawn[s] = true;
                        roles[gain[s]] = true;
                        assert!(r.players[s].society_zone.used_once_per_game.is_empty());
                    } else {
                        searched[s] = true;
                        last_cast_turn = r.turn;
                        assert!(r.players[s]
                            .society_zone
                            .used_once_per_game
                            .contains(SEARCH));
                        for other in 0..4 {
                            if !searched[other] {
                                assert!(r.players[other]
                                    .society_zone
                                    .used_once_per_game
                                    .is_empty());
                            }
                        }
                    }
                    acted = true;
                    break;
                }
            }
            if !searched[s] {
                let assets = legal[s]
                    .iter()
                    .filter(|a| a.action.kind == "asset")
                    .collect::<Vec<_>>();
                if let Some(a) = assets
                    .iter()
                    .find(|a| {
                        r.players[s]
                            .hand
                            .iter()
                            .find(|c| a.action.card_id.as_ref() == Some(&c.id))
                            .is_some_and(|c| !catalog::card(&c.definition).unique)
                    })
                    .or_else(|| assets.first())
                {
                    apply_game(&mut r, &mut steps, s, a.action.clone());
                    acted = true;
                    break;
                }
            }
        }
        if !acted {
            pass(&mut r, &mut steps);
        }
    }
    assert!(
        draw_done.iter().all(|x| *x)
            && search_done.iter().all(|x| *x)
            && roles == [true, true]
            && r.turn > last_cast_turn
    );
    assert_eq!(selections.len(), 4);
    for s in 0..4 {
        assert_eq!(r.players[s].society_zone.used_once_per_game.len(), 1);
        for v in 0..4 {
            assert_eq!(
                r.view(v, r.pacing.last_server_now_ms).game.society_zones[s]
                    .card
                    .as_ref()
                    .unwrap()
                    .used_once_per_game,
                Some(vec![SEARCH.into()])
            );
        }
    }
    json!({"name":"printed-MSJC08-natural-four-seat-build-pay-draw-purple-search-turn-restore","seed":seed,"steps":steps,"selections":selections,"ordinaryDeckSize":50,"purpleDeckCount":27,"startingHand":6,"formalPurpleUniqueHits":4,"syntheticInitialLayout":false,"syntheticCardRegistry":false,"syntheticFilterChanged":false,"normalRoomCommandsOnly":true,"publicNaturalUiAcceptance":false})
}
pub(super) fn cases() -> impl Iterator<Item = Value> {
    [25, 27]
        .into_iter()
        .map(construction)
        .chain(std::iter::once_with(natural))
        .chain(
            [
                "reject-cost",
                "reject-exhausted",
                "reject-foreign-source",
                "reject-yellow-key",
                "draw-first",
                "draw-rear",
                "empty-search",
                "empty-deck",
                "one-hit",
                "two-hits-seat3",
                "cancelled-paid-frame",
                "new-instance-used",
                "finished-restart",
                "turn-reset-used",
            ]
            .into_iter()
            .map(boundary),
        )
}
