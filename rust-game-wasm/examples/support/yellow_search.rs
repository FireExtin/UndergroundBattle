//! Real printed bindings from explicitly marked initial layouts. Natural joined
//! MSJC01 construction/search/deployment is a separate scenario, never a fixture.
use super::*;
fn field(g: &mut Game, def: &str, owner: usize, controller: usize) -> String {
    let mut c = g.make_card(def, owner);
    c.controller = controller;
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn fund(g: &mut Game, n: usize) {
    for _ in 0..n {
        let c = g.make_card("JC002", 0);
        g.players[0].assets.push(c);
    }
}
fn deck(g: &mut Game, s: usize, defs: &[&str]) {
    g.players[s].deck.clear();
    for d in defs {
        let c = g.make_card(d, s);
        g.players[s].deck.push(c);
    }
}
fn activate(source: &str, key: &str) -> Action {
    Action {
        card_id: Some(source.into()),
        ability_id: Some(key.into()),
        ..Action::new("activate")
    }
}
fn checked_start(g: Game) -> (RoomEnvelope, Vec<Value>) {
    let r = RoomEnvelope::from_game(g);
    RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();
    let s = step(
        &r,
        "initialFixture",
        json!([serde_json::to_string(&r).unwrap()]),
        0,
    );
    (r, vec![s])
}
fn check_top(r: &RoomEnvelope, s: usize) {
    assert_eq!(
        r.view(s, r.pacing.last_server_now_ms)
            .game
            .private_deck_top
            .as_ref()
            .map(|c| &c.instance_id),
        r.players[s].deck.first().map(|c| &c.id)
    );
    for other in 0..4 {
        if other != s {
            assert!(r
                .view(other, r.pacing.last_server_now_ms)
                .game
                .private_deck_top
                .is_none());
        }
    }
}
fn choose(r: &mut RoomEnvelope, steps: &mut Vec<Value>, id: Option<String>) {
    let p = r.pending.clone().unwrap();
    apply_game(
        r,
        steps,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(id.into_iter().collect()),
            ..Action::new("choose")
        },
    );
}
fn case(kind: &str) -> Value {
    let mut g = msjc01::initial();
    g.first_team = 0;
    fund(
        &mut g,
        if kind == "private-cost-rejection" {
            4
        } else {
            12
        },
    );
    let w = field(&mut g, "WM003", 0, 0);
    if kind == "private-exhaustion-rejection" {
        g.regions[0].cards[0].exhausted = true;
    }
    let peek = kind.starts_with("top-");
    if peek {
        field(
            &mut g,
            "LC01",
            0,
            if kind == "top-controller" { 1 } else { 0 },
        );
    }
    if kind == "top-exhausted" {
        g.regions[0].cards.last_mut().unwrap().exhausted = true;
    }
    if kind == "top-hidden" {
        g.regions[0].cards.last_mut().unwrap().face_down = true;
    }
    deck(&mut g, 0, &["LC23", "LC01", "WM003", "JC125", "JC125"]);
    if kind == "top-controller" {
        deck(&mut g, 1, &["LC23", "JC125"]);
    }
    if kind == "private-empty" || kind == "top-empty" {
        deck(&mut g, 0, &[]);
    }
    if kind == "top-multiple-leave" {
        field(&mut g, "LC01", 2, 0);
        for _ in 0..2 {
            let c = g.make_card("JC006", 0);
            g.players[0].hand.push(c);
        }
    }
    let (mut r, mut steps) = checked_start(g);
    let mut denied = vec![];
    if kind.ends_with("rejection") {
        denied.push(rejected(&r, 0, activate(&w, "search-any-private")));
    } else if kind == "top-controller" {
        check_top(&r, 1);
    } else if kind == "top-exhausted" {
        check_top(&r, 0);
    } else if kind == "top-hidden" || kind == "top-empty" {
        assert!((0..4).all(|s| r.view(s, 0).game.private_deck_top.is_none()));
    } else if kind == "top-multiple-leave" {
        let targets = r.regions[0]
            .cards
            .iter()
            .filter(|c| c.definition == "LC01")
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        let spells = r.players[0]
            .hand
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        for (i, (source, target)) in spells.into_iter().zip(targets).enumerate() {
            apply_game(
                &mut r,
                &mut steps,
                0,
                Action {
                    card_id: Some(source),
                    target_id: Some(target),
                    ..Action::new("play")
                },
            );
            attachment_pass_top(&mut r, &mut steps);
            if i == 0 {
                check_top(&r, 0);
            } else {
                assert!(r
                    .view(0, r.pacing.last_server_now_ms)
                    .game
                    .private_deck_top
                    .is_none());
            }
        }
    } else {
        let source = if kind == "real-yellow-reveal" || kind == "top-draw" {
            r.players[0].society_zone.card.as_ref().unwrap().id.clone()
        } else {
            w.clone()
        };
        let key = if kind == "real-yellow-reveal" {
            "search-yellow-unique"
        } else if kind == "top-draw" {
            "drawWithInitiative"
        } else {
            "search-any-private"
        };
        apply_game(&mut r, &mut steps, 0, activate(&source, key));
        attachment_pass_top(&mut r, &mut steps);
        if kind == "private-empty" {
            assert!(r.pending.is_none());
        } else if kind == "top-draw" {
            check_top(&r, 0);
            assert_eq!(r.players[0].hand[0].definition, "LC23");
        } else {
            let p = r.pending.clone().unwrap();
            let selected = p
                .choice
                .options
                .iter()
                .find(|o| {
                    o.card.as_ref().unwrap().card_id.as_deref()
                        == Some(if kind == "real-yellow-reveal" {
                            "LC01"
                        } else {
                            "LC23"
                        })
                })
                .unwrap()
                .id
                .clone();
            if kind == "real-yellow-reveal" {
                assert_eq!(p.choice.options.len(), 2);
            } else {
                for other in 1..4 {
                    let view = r.view(other, r.pacing.last_server_now_ms);
                    assert!(view.game.pending_choice.is_none());
                    assert!(!serde_json::to_string(&view).unwrap().contains("LC23"));
                }
            }
            let rejection = rejected(
                &r,
                0,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec![]),
                    ..Action::new("choose")
                },
            );
            record_transition(
                &mut r,
                &mut steps,
                "applyRoom",
                json!([0, rejection["command"], rejection["serverNow"]]),
                serde_json::from_value(rejection["expected"].clone()).unwrap(),
            );
            choose(&mut r, &mut steps, Some(selected.clone()));
            assert!(r.players[0].hand.iter().any(|c| c.id != selected
                && c.definition
                    == if kind == "real-yellow-reveal" {
                        "LC01"
                    } else {
                        "LC23"
                    }));
            for s in 0..4 {
                let view = r.view(s, r.pacing.last_server_now_ms);
                if kind == "real-yellow-reveal" {
                    assert!(view
                        .game
                        .log
                        .iter()
                        .any(|e| e.text.contains("展示检索的 西比尔")));
                } else {
                    assert!(!view
                        .game
                        .log
                        .iter()
                        .any(|e| e.text.contains("安格鲁") || e.text.contains("LC23")));
                }
            }
            if peek {
                check_top(&r, 0);
            }
        }
    }
    json!({"name":format!("yellow-batch-{kind}"),"seed":"9007199254740993","steps":steps,"rejectedCommands":denied,"syntheticInitialLayout":true,"realPrintedCardBindings":true,"publicNaturalUiAcceptance":false})
}
fn natural_case() -> Value {
    let seed = "18446744073709551615";
    let mut d = msjc01::draft(25, 50);
    d.cards
        .iter_mut()
        .find(|c| c.card_id == "JC125")
        .unwrap()
        .count = 19;
    d.cards.push(catalog::DeckEntry {
        card_id: "LC01".into(),
        count: 3,
    });
    d.cards.push(catalog::DeckEntry {
        card_id: "WM003".into(),
        count: 3,
    });
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "888888888888888888880114".into(),
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
    let mut searched = [false; 4];
    let mut resolved = [false; 4];
    let mut before = [0; 4];
    let mut deployed = false;
    for _ in 0..1800 {
        if let Some(p) = r.pending.clone() {
            if p.choice.kind == "search" {
                assert!(!p.choice.options.is_empty());
                let id = p
                    .choice
                    .options
                    .iter()
                    .find(|o| o.card.as_ref().unwrap().card_id.as_deref() == Some("LC01"))
                    .unwrap_or(&p.choice.options[0])
                    .id
                    .clone();
                choose(&mut r, &mut steps, Some(id));
                assert_eq!(r.players[p.seat].hand.len(), before[p.seat] + 1);
                resolved[p.seat] = true;
            } else {
                let (s, a) = pick_choice(&r.game);
                apply_game(&mut r, &mut steps, s, a);
            }
            continue;
        }
        if deployed && r.stack.is_empty() {
            assert!(r
                .view(0, r.pacing.last_server_now_ms)
                .game
                .private_deck_top
                .is_some());
            break;
        }
        let legal = (0..4)
            .map(|s| r.view(s, r.pacing.last_server_now_ms).game.legal_actions)
            .collect::<Vec<_>>();
        let mut acted = false;
        for s in 0..4 {
            if !searched[s] {
                if let Some(a) = legal[s]
                    .iter()
                    .find(|a| a.action.ability_id.as_deref() == Some("search-yellow-unique"))
                {
                    before[s] = r.players[s].hand.len();
                    apply_game(&mut r, &mut steps, s, a.action.clone());
                    searched[s] = true;
                    acted = true;
                    break;
                }
            }
            if resolved.iter().all(|v| *v) && s == 0 && !deployed {
                if let Some(a) = legal[s].iter().find(|a| {
                    a.action.kind == "deploy"
                        && r.players[0].hand.iter().any(|c| {
                            Some(&c.id) == a.action.card_id.as_ref() && c.definition == "LC01"
                        })
                }) {
                    apply_game(&mut r, &mut steps, 0, a.action.clone());
                    deployed = true;
                    acted = true;
                    break;
                }
            }
            if !searched[s] || (resolved.iter().all(|v| *v) && s == 0 && !deployed) {
                if let Some(a) = legal[s]
                    .iter()
                    .filter(|a| a.action.kind == "asset")
                    .find(|a| {
                        r.players[s].hand.iter().any(|c| {
                            Some(&c.id) == a.action.card_id.as_ref()
                                && !catalog::card(&c.definition).unique
                                && catalog::card(&c.definition).color == "黄"
                        })
                    })
                    .or_else(|| {
                        legal[s].iter().find(|a| {
                            a.action.kind == "asset"
                                && r.players[s].hand.iter().any(|c| {
                                    Some(&c.id) == a.action.card_id.as_ref()
                                        && !catalog::card(&c.definition).unique
                                })
                        })
                    })
                {
                    apply_game(&mut r, &mut steps, s, a.action.clone());
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
            .find_map(|(s, as_)| {
                as_.iter()
                    .find(|a| a.action.kind == "pass")
                    .map(|a| (s, a.action.clone()))
            })
            .unwrap();
        apply_game(&mut r, &mut steps, s, a);
    }
    assert!(resolved.iter().all(|v| *v) && deployed && r.stack.is_empty());
    assert!(r
        .view(0, r.pacing.last_server_now_ms)
        .game
        .private_deck_top
        .is_some());
    for s in 1..4 {
        assert!(r
            .view(s, r.pacing.last_server_now_ms)
            .game
            .private_deck_top
            .is_none());
    }
    json!({"name":"printed-yellow-batch-natural-four-seat-real-search-LC01-deploy-private-top-restore","seed":seed,"steps":steps,"syntheticInitialLayout":false,"syntheticCardRegistry":false,"normalRoomCommandsOnly":true,"fourSeatNaturalYellowSearchGain":true,"publicNaturalUiAcceptance":false})
}
fn top_checkpoint(to_top: bool) -> Value {
    let mut g = msjc01::initial();
    field(&mut g, "LC01", 0, 0);
    let w = field(&mut g, "WM003", 0, 0);
    fund(&mut g, 5);
    deck(&mut g, 0, &["LC23", "JC125", "XQ03", "JC125"]);
    let mut r = RoomEnvelope::from_game(g);
    apply_game(&mut r, &mut vec![], 0, activate(&w, "search-any-private"));
    if let Op::Search {
        to_top: dest,
        optional,
        ..
    } = &mut r
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
        *dest = to_top;
        *optional = !to_top;
    }
    RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();
    let mut steps = vec![step(
        &r,
        "initialFixture",
        json!([serde_json::to_string(&r).unwrap()]),
        0,
    )];
    attachment_pass_top(&mut r, &mut steps);
    let rng = r.random;
    let n = r.players[0].deck.len();
    let id = r
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .find(|o| o.card.as_ref().unwrap().card_id.as_deref() == Some("LC23"))
        .unwrap()
        .id
        .clone();
    choose(
        &mut r,
        &mut steps,
        if to_top { Some(id.clone()) } else { None },
    );
    assert_eq!(r.players[0].deck.len(), n);
    assert!(r.players[0].hand.is_empty());
    assert_ne!(r.random, rng);
    check_top(&r, 0);
    if to_top {
        assert_eq!(r.players[0].deck[0].definition, "LC23");
        assert_ne!(r.players[0].deck[0].id, id);
    }
    json!({"name":if to_top{"yellow-batch-top-return-checkpoint"}else{"yellow-batch-top-optional-shuffle-checkpoint"},"seed":"9007199254740993","steps":steps,"syntheticInitialLayout":true,"syntheticPaidFrameCheckpoint":true,"syntheticSearchTopDestination":to_top,"syntheticSearchOptional":!to_top,"publicNaturalUiAcceptance":false})
}
pub(super) fn cases() -> Vec<Value> {
    [
        "private-search",
        "private-empty",
        "private-cost-rejection",
        "private-exhaustion-rejection",
        "real-yellow-reveal",
        "top-search",
        "top-draw",
        "top-controller",
        "top-exhausted",
        "top-hidden",
        "top-empty",
        "top-multiple-leave",
    ]
    .into_iter()
    .map(case)
    .chain([top_checkpoint(true), top_checkpoint(false)])
    .chain(std::iter::once(natural_case()))
    .collect()
}
