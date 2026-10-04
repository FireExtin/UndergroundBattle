//! Default MSJC01 oracle: natural construction/payment/empty searches and explicitly
//! marked paid-frame checkpoints. LC23 hit fixtures change the filter to neutral;
//! no formal yellow unique card or natural yellow search gain is fabricated.
use super::*;
const SEARCH: &str = "search-yellow-unique";
const DRAW: &str = "drawWithInitiative";
fn draft(yellow: usize, total: usize) -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.id = "msjc01-oracle".into();
    d.name = "MSJC01构筑与能力验证".into();
    d.society_id = Some("MSJC01".into());
    d.cards.clear();
    let mut n = yellow;
    for id in [
        "JC001", "JC002", "JC003", "JC004", "JC005", "JC006", "JC007", "XQ03", "JC008",
    ] {
        let count = n.min(3);
        if count > 0 {
            d.cards.push(catalog::DeckEntry {
                card_id: id.into(),
                count,
            });
        }
        n -= count;
    }
    assert_eq!(n, 0);
    if total > yellow {
        d.cards.push(catalog::DeckEntry {
            card_id: "JC125".into(),
            count: total - yellow,
        });
    }
    d
}
fn activate(g: &Game, s: usize, key: &str) -> Action {
    Action {
        card_id: Some(g.players[s].society_zone.card.as_ref().unwrap().id.clone()),
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
fn advance(room: &mut RoomEnvelope, steps: &mut Vec<Value>, done: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..400 {
        if done(room) {
            return;
        }
        if room.pending.is_some() {
            let (s, a) = pick_choice(&room.game);
            apply_game(room, steps, s, a);
        } else {
            let (s, a) = (0..4)
                .find_map(|s| {
                    room.view(s, room.pacing.last_server_now_ms)
                        .game
                        .legal_actions
                        .into_iter()
                        .find(|a| a.action.kind == "pass")
                        .map(|a| (s, a.action))
                })
                .unwrap();
            apply_game(room, steps, s, a);
        }
    }
    panic!("bounded MSJC01 Room progress failed");
}
fn initial() -> Game {
    let mut g = Game::new_with_deck(
        "888888888888888888880101".into(),
        "MSJC01".into(),
        "teams".into(),
        "P0".into(),
        draft(25, 50),
        9007199254740993,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), draft(25, 50)).unwrap();
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
        g.players[s].assets.clear();
        g.players[s].deck.clear();
        for _ in 0..12 {
            let c = g.make_card("JC125", s);
            g.players[s].deck.push(c);
        }
    }
    for r in &mut g.regions {
        r.cards.clear();
    }
    // Explicit initial window fixture; no new production accessor is needed.
    g.window = Some(Window::Action(0));
    g.active_team = 0;
    g.priority_team = 0;
    g.passed.clear();
    g.team_passed = [false; 2];
    g
}
fn case(kind: &str) -> Value {
    let mut g = initial();
    fund(&mut g, 0, if kind == "reject-cost" { 3 } else { 8 });
    if kind == "reject-exhausted" {
        g.players[0].society_zone.card.as_mut().unwrap().exhausted = true;
    }
    if kind == "empty-deck" {
        g.players[0].deck.clear();
    }
    if kind == "draw-rear" {
        g.first_team = 1;
    }
    let hits = if kind == "neutral-unique-one-hit" {
        1
    } else if kind == "neutral-unique-three-hits" || kind == "pending-search-restore" {
        3
    } else {
        0
    };
    for _ in 0..hits {
        let c = g.make_card("LC23", 0);
        g.players[0].deck.push(c);
    }
    let checkpoint = hits > 0
        || matches!(
            kind,
            "cancelled-paid-frame" | "new-instance-used" | "new-game-reset"
        );
    let mut room = RoomEnvelope::from_game(g);
    if checkpoint {
        // Build the paid checkpoint through Room commands so its persisted
        // response window agrees with the active Game stack and priority.
        let mut preparation = vec![];
        let a = activate(&room, 0, SEARCH);
        apply_game(&mut room, &mut preparation, 0, a);
        if hits > 0 {
            if let Op::Search { filter, .. } = &mut room
                .game
                .stack
                .last_mut()
                .unwrap()
                .frame
                .as_mut()
                .unwrap()
                .steps[0]
                .op
            {
                *filter = rules::CardFilter::PrintedColorAndUnique {
                    color: "中立".into(),
                };
            }
        }
        if kind == "cancelled-paid-frame" {
            room.game
                .stack
                .last_mut()
                .unwrap()
                .frame
                .as_mut()
                .unwrap()
                .guard = hegemony_server::model::GuardState::Cancelled;
        }
        if matches!(
            kind,
            "new-instance-used" | "new-game-reset" | "pending-search-restore"
        ) {
            attachment_pass_top(&mut room, &mut preparation);
        }
        if kind == "new-instance-used" {
            let c = room.game.players[0].society_zone.card.take().unwrap();
            let mut c = room.game.make_card(&c.definition, c.owner);
            c.exhausted = false;
            room.game.players[0].society_zone.card = Some(c);
        }
        if kind == "new-game-reset" {
            room.game.status = "finished".into();
        }
    }
    RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).unwrap();
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        0,
    )];
    let mut denied = vec![];
    if kind.starts_with("reject-") || kind == "new-instance-used" {
        denied.push(rejected(&room, 0, activate(&room, 0, SEARCH)));
    } else if kind == "new-game-reset" {
        apply_game(&mut room, &mut steps, 0, Action::new("restart"));
        assert!(room
            .players
            .iter()
            .all(|p| p.society_zone.used_once_per_game.is_empty()));
    } else {
        if !checkpoint {
            let key = if kind.starts_with("draw-") {
                DRAW
            } else {
                SEARCH
            };
            let a = activate(&room, 0, key);
            apply_game(&mut room, &mut steps, 0, a);
            if key == SEARCH {
                assert!(room.players[0]
                    .society_zone
                    .used_once_per_game
                    .contains(SEARCH));
            } else {
                assert!(room.players[0].society_zone.used_once_per_game.is_empty());
            }
        }
        if room.pending.is_none() {
            attachment_pass_top(&mut room, &mut steps);
        }
        if hits > 0 {
            let p = room.pending.clone().unwrap();
            assert_eq!(p.choice.options.len(), hits);
            let rejection = rejected(
                &room,
                0,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec![]),
                    ..Action::new("choose")
                },
            );
            let transition: RoomTransition =
                serde_json::from_value(rejection["expected"].clone()).unwrap();
            record_transition(
                &mut room,
                &mut steps,
                "applyRoom",
                json!([0, rejection["command"], rejection["serverNow"]]),
                transition,
            );
            let id = p.choice.options[0].id.clone();
            let n = room.players[0].deck.len();
            let rng = room.random;
            apply_game(
                &mut room,
                &mut steps,
                0,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec![id.clone()]),
                    ..Action::new("choose")
                },
            );
            let held = room.players[0]
                .hand
                .iter()
                .find(|c| c.definition == "LC23")
                .unwrap();
            assert_ne!(held.id, id);
            assert_eq!(room.players[0].deck.len(), n - 1);
            assert_ne!(room.random, rng);
            assert!(room
                .log
                .iter()
                .any(|e| e.text.contains("展示检索的 安格鲁")));
        } else if kind.starts_with("draw-") {
            assert_eq!(
                room.players[0].hand.len(),
                usize::from(kind == "draw-first")
            );
        } else {
            assert!(room.players[0].hand.is_empty());
            assert!(room.players[0]
                .society_zone
                .used_once_per_game
                .contains(SEARCH));
            if kind != "cancelled-paid-frame" {
                assert!(room.log.iter().any(|e| e.text.contains("没有符合条件的牌")));
            }
        }
        if kind == "turn-reset-used" {
            let t = room.turn;
            advance(&mut room, &mut steps, |r| {
                r.turn > t && matches!(r.window, Some(Window::Action(0)))
            });
            assert!(
                !room.players[0]
                    .society_zone
                    .card
                    .as_ref()
                    .unwrap()
                    .exhausted
            );
            denied.push(rejected(&room, 0, activate(&room, 0, SEARCH)));
            assert!(room
                .view(0, room.pacing.last_server_now_ms)
                .game
                .legal_actions
                .iter()
                .any(|a| a.action.ability_id.as_deref() == Some(DRAW)));
        }
    }
    if steps.len() > 1 {
        let last = steps.last().unwrap();
        let command: RoomCommand = serde_json::from_value(last["args"][1].clone()).unwrap();
        let s = last["args"][0].as_u64().unwrap() as usize;
        let now = last["args"][2].as_str().unwrap().parse::<u64>().unwrap();
        let retry = room.transition(s, Some(command.clone()), now).unwrap();
        assert!(!retry.changed);
        assert_eq!(retry.state, serde_json::to_string(&room).unwrap());
        record_transition(
            &mut room,
            &mut steps,
            "applyRoom",
            json!([s, command, now.to_string()]),
            retry,
        );
    }
    json!({"name":format!("MSJC01-{kind}"),"seed":"9007199254740993","steps":steps,"rejectedCommands":denied,"syntheticInitialLayout":true,"syntheticPaidFrameCheckpoint":checkpoint,"syntheticFilterChangedToNeutralUnique":hits>0,"syntheticCancelledFrame":kind=="cancelled-paid-frame","syntheticNewSourceInstance":kind=="new-instance-used","syntheticFinishedGame":kind=="new-game-reset","formalYellowUniqueHits":0,"publicNaturalUiAcceptance":false})
}
fn natural_case() -> Value {
    let seed = "18446744073709551615";
    let d = draft(25, 50);
    let mut room = RoomEnvelope::from_game(
        Game::new_with_deck(
            "888888888888888888880102".into(),
            "MSJC01".into(),
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
    for s in 1..4 {
        room.game
            .join_with_deck(format!("P{s}"), d.clone())
            .unwrap();
        room.revision = room.game.version;
        steps.push(step(
            &room,
            "joinGameWithDeck",
            json!([format!("P{s}"), serde_json::to_string(&d).unwrap()]),
            s,
        ));
    }
    for s in 0..4 {
        apply_game(&mut room, &mut steps, s, Action::new("ready"));
    }
    apply_game(&mut room, &mut steps, 0, Action::new("start"));
    assert!(room.players.iter().all(|p| p.hand.len() == 6));
    let mut drawn = [false; 4];
    let mut searched = [false; 4];
    let mut draw_resolved = [false; 4];
    let mut search_resolved = [false; 4];
    let mut hand_before = [0; 4];
    let mut expected = [0; 4];
    let mut roles = [false; 2];
    let mut last_cast_turn = 0;
    for _ in 0..2500 {
        if room.pending.is_some() {
            let (s, a) = pick_choice(&room.game);
            apply_game(&mut room, &mut steps, s, a);
            continue;
        }
        if room.stack.is_empty() {
            for s in 0..4 {
                if drawn[s] && !draw_resolved[s] {
                    assert_eq!(room.players[s].hand.len(), hand_before[s] + expected[s]);
                    draw_resolved[s] = true;
                }
                if searched[s] && !search_resolved[s] {
                    assert_eq!(room.players[s].hand.len(), hand_before[s]);
                    search_resolved[s] = true;
                }
            }
        }
        if search_resolved.iter().all(|v| *v) && room.turn > last_cast_turn {
            break;
        }
        let legal: Vec<_> = (0..4)
            .map(|s| {
                room.view(s, room.pacing.last_server_now_ms)
                    .game
                    .legal_actions
            })
            .collect();
        let mut acted = false;
        for s in 0..4 {
            if !drawn[s] {
                if let Some(a) = legal[s].iter().find(|a| {
                    a.action.kind == "activate" && a.action.ability_id.as_deref() == Some(DRAW)
                }) {
                    hand_before[s] = room.players[s].hand.len();
                    expected[s] = usize::from(room.team(s) == room.first_team);
                    roles[expected[s]] = true;
                    apply_game(&mut room, &mut steps, s, a.action.clone());
                    drawn[s] = true;
                    acted = true;
                    break;
                }
            } else if !searched[s] {
                if let Some(a) = legal[s].iter().find(|a| {
                    a.action.kind == "activate" && a.action.ability_id.as_deref() == Some(SEARCH)
                }) {
                    hand_before[s] = room.players[s].hand.len();
                    apply_game(&mut room, &mut steps, s, a.action.clone());
                    assert!(room.players[s]
                        .society_zone
                        .used_once_per_game
                        .contains(SEARCH));
                    searched[s] = true;
                    last_cast_turn = room.turn;
                    acted = true;
                    break;
                }
            }
            if !searched[s] {
                if let Some(a) = legal[s].iter().find(|a| a.action.kind == "asset") {
                    apply_game(&mut room, &mut steps, s, a.action.clone());
                    acted = true;
                    break;
                }
            }
        }
        if acted {
            continue;
        }
        let (s, a) = legal
            .iter()
            .enumerate()
            .find_map(|(s, list)| {
                list.iter()
                    .find(|a| a.action.kind == "pass")
                    .map(|a| (s, a.action.clone()))
            })
            .unwrap();
        apply_game(&mut room, &mut steps, s, a);
    }
    assert!(
        draw_resolved.iter().all(|v| *v)
            && search_resolved.iter().all(|v| *v)
            && roles.iter().all(|v| *v)
            && room.turn > last_cast_turn
    );
    for s in 0..4 {
        assert_eq!(room.players[s].society_zone.used_once_per_game.len(), 1);
        assert!(
            !room.players[s]
                .society_zone
                .card
                .as_ref()
                .unwrap()
                .exhausted
        );
        for viewer in 0..4 {
            assert_eq!(
                room.view(viewer, room.pacing.last_server_now_ms)
                    .game
                    .society_zones[s]
                    .card
                    .as_ref()
                    .unwrap()
                    .used_once_per_game,
                Some(vec![SEARCH.into()])
            );
        }
    }
    json!({"name":"printed-MSJC01-natural-four-seat-build-pay-draw-empty-search-turn-restore","seed":seed,"steps":steps,"syntheticInitialLayout":false,"syntheticCardRegistry":false,"normalRoomCommandsOnly":true,"formalYellowUniqueHits":0,"naturalYellowSearchGainClaimed":false,"publicNaturalUiAcceptance":false})
}
fn construction_case(yellow: usize) -> Value {
    let seed = "18446744073709551615";
    let d = draft(yellow, 50);
    let room = RoomEnvelope::from_game(
        Game::new_with_deck(
            "888888888888888888880103".into(),
            "MSJC01".into(),
            "duel".into(),
            "P0".into(),
            d.clone(),
            seed.parse().unwrap(),
        )
        .unwrap(),
    );
    let rejects=[draft(24,50),draft(25,49)].into_iter().map(|d|{let error=Game::new_with_deck("reject".into(),"LOCAL".into(),"duel".into(),"P0".into(),d.clone(),1).unwrap_err();json!({"args":["reject","LOCAL","duel","P0",serde_json::to_string(&d).unwrap(),"1"],"error":error})}).collect::<Vec<_>>();
    json!({"name":format!("MSJC01-construction-{yellow}-yellow"),"seed":seed,"steps":[step(&room,"newGameWithDeck",json!([room.room_id,room.invite_code,"duel","P0",serde_json::to_string(&d).unwrap(),seed]),0)],"rejectedNewGameWithDeck":rejects,"syntheticInitialLayout":false,"publicNaturalUiAcceptance":false})
}
pub(super) fn cases() -> Vec<Value> {
    let mut out = vec![construction_case(25), construction_case(27), natural_case()];
    out.extend(
        [
            "empty-search",
            "empty-deck",
            "reject-cost",
            "reject-exhausted",
            "draw-first",
            "draw-rear",
            "neutral-unique-one-hit",
            "neutral-unique-three-hits",
            "pending-search-restore",
            "cancelled-paid-frame",
            "new-instance-used",
            "new-game-reset",
            "turn-reset-used",
        ]
        .into_iter()
        .map(case),
    );
    out
}
