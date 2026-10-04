//! Only boundary starting layouts/funding are synthetic. Recorded operations
//! use the real Room ABI, complete journals, exact persistence and four views.
use super::*;
use hegemony_server::{model::Card, rules::Event};

fn field(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn fund(g: &mut Game, seat: usize, def: &str, n: usize) {
    for _ in 0..n {
        let c = g.make_card(def, seat);
        g.players[seat].assets.push(c);
    }
}
fn held(g: &mut Game, seat: usize, def: &str) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn choose(r: &mut RoomEnvelope, steps: &mut Vec<Value>, selected: Vec<String>) {
    let p = r.pending.clone().unwrap();
    apply_game(
        r,
        steps,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(selected),
            ..Action::new("choose")
        },
    );
}
fn is_renown(r: &RoomEnvelope) -> bool {
    matches!(r.pending.as_ref().map(|p| &p.resolution), Some(ChoiceResolution::Declare { declaration, .. }) if declaration.ability.event == Some(Event::RegionConfrontationsEnded))
}
fn advance(r: &mut RoomEnvelope, steps: &mut Vec<Value>, done: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..500 {
        if done(r) {
            return;
        }
        let (seat, a) = if r.pending.is_some() {
            pick_choice(&r.game)
        } else {
            (0..4)
                .find_map(|s| {
                    r.game
                        .legal_actions(s)
                        .into_iter()
                        .find(|a| a.action.kind == "pass")
                        .map(|a| (s, a.action))
                })
                .unwrap()
        };
        apply_game(r, steps, seat, a);
    }
    panic!("bounded renown advance");
}
fn cast(r: &mut RoomEnvelope, steps: &mut Vec<Value>, seat: usize, def: &str, target: &str) {
    let a = r
        .game
        .legal_actions(seat)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.target_id.as_deref() == Some(target)
                && a.action.card_id.as_ref().is_some_and(|id| {
                    r.players[seat]
                        .hand
                        .iter()
                        .any(|c| c.id == *id && c.definition == def)
                })
        })
        .unwrap()
        .action;
    apply_game(r, steps, seat, a);
    attachment_pass_top(r, steps);
}
fn boundary(kind: &str) -> Value {
    let mut g = attachment_initial("teams", "888888888888888888880117");
    g.regions[0].card = g.make_card("DQJC115", 0);
    fund(&mut g, 0, "JC075", 8);
    fund(&mut g, 0, "JC104", 3);
    fund(&mut g, 2, "JC002", 2);
    fund(&mut g, 2, "XQ16", 5);
    let target = field(&mut g, "LC01", 2);
    for def in ["JC070", "JC076", "JC074", "JC129"] {
        held(&mut g, 0, def);
    }
    held(&mut g, 2, "JC006");
    if matches!(kind, "normal-win" | "merged-decline" | "renown-win") {
        g.regions[0].cards.clear();
        if kind == "normal-win" {
            for _ in 0..3 {
                field(&mut g, "JC075", 0);
            }
            field(&mut g, "JC070", 2);
        } else {
            field(&mut g, "JC070", 0);
            field(&mut g, "JC076", 1);
            field(&mut g, "JC070", 2);
            if kind == "renown-win" {
                g.regions[0].influence = [2, 0];
            }
        }
        g.window = Some(Window::Before(0, 2));
    }
    let old_region = g.regions[0].card.id.clone();
    let mut r = RoomEnvelope::from_game(g);
    let mut steps = vec![step(
        &r,
        "initialFixture",
        json!([serde_json::to_string(&r).unwrap()]),
        0,
    )];
    if kind == "normal-win" {
        advance(&mut r, &mut steps, |r| {
            matches!(r.window, Some(Window::Win(0, 0)))
        });
        assert!(r.pending.is_none() && r.stack.is_empty());
        advance(&mut r, &mut steps, |r| r.regions[0].card.id != old_region);
        assert!(!is_renown(&r));
        assert_eq!(r.players[0].score_cards.len(), 1);
        if r.pending.is_some() {
            choose(&mut r, &mut steps, vec![]);
        }
        advance(&mut r, &mut steps, |r| {
            r.window == Some(Window::Before(1, 0))
        });
        assert_eq!(r.regions[0].influence, [0, 0]);
    } else if kind == "merged-decline" || kind == "renown-win" {
        advance(&mut r, &mut steps, is_renown);
        assert_eq!(r.pending.as_ref().unwrap().seat, 0);
        for s in 1..4 {
            assert!(r
                .view(s, r.pacing.last_server_now_ms)
                .game
                .pending_choice
                .is_none());
        }
        if kind == "merged-decline" {
            choose(&mut r, &mut steps, vec![]);
            advance(&mut r, &mut steps, |r| {
                r.window == Some(Window::Before(1, 0))
            });
            assert_eq!(r.regions[0].influence, [0, 0]);
        } else {
            choose(&mut r, &mut steps, vec!["accept".into()]);
            attachment_pass_top(&mut r, &mut steps);
            assert_eq!(r.window, Some(Window::Win(0, 0)));
            advance(&mut r, &mut steps, |r| r.regions[0].card.id != old_region);
            assert_eq!(r.players[0].score_cards.len(), 1);
            assert!(!is_renown(&r));
        }
    } else if kind == "response-identity" {
        let spell = r.players[0]
            .hand
            .iter()
            .find(|c| c.definition == "JC074")
            .unwrap()
            .id
            .clone();
        apply_game(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(spell),
                target_id: Some(target.clone()),
                ..Action::new("play")
            },
        );
        advance(&mut r, &mut steps, |r| r.priority_team == 1);
        cast(&mut r, &mut steps, 2, "JC006", &target);
        attachment_pass_top(&mut r, &mut steps);
        assert!(r.turn_attribute_modifiers.is_empty());
        assert!(!r.regions[0].cards.iter().any(|c| c.id == target));
        advance(&mut r, &mut steps, |r| {
            r.pending.is_none() && r.stack.is_empty() && r.window == Some(Window::Action(1))
        });
        let a = r
            .game
            .legal_actions(2)
            .into_iter()
            .find(|a| {
                a.action.kind == "deploy"
                    && a.action.region == Some(0)
                    && a.action.card_id.as_ref().is_some_and(|id| {
                        r.players[2]
                            .hand
                            .iter()
                            .any(|c| c.id == *id && c.definition == "LC01")
                    })
            })
            .unwrap()
            .action;
        apply_game(&mut r, &mut steps, 2, a);
        attachment_pass_top(&mut r, &mut steps);
        let c = r.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "LC01")
            .unwrap();
        assert_ne!(c.id, target);
        assert!(r.turn_attribute_modifiers.is_empty());
        for s in 0..4 {
            assert_eq!(
                r.view(s, r.pacing.last_server_now_ms).game.regions[0].characters[0].current_renown,
                None
            );
        }
    } else {
        cast(&mut r, &mut steps, 0, "JC074", &target);
        cast(&mut r, &mut steps, 0, "JC129", &target);
        assert_eq!(r.regions[0].cards[0].controller, 0);
        for s in 0..4 {
            assert_eq!(
                r.view(s, r.pacing.last_server_now_ms).game.regions[0].characters[0].current_renown,
                Some(true)
            );
        }
        advance(&mut r, &mut steps, is_renown);
        choose(&mut r, &mut steps, vec!["accept".into()]);
        attachment_pass_top(&mut r, &mut steps);
        assert_eq!(r.regions[0].influence, [1, 0]);
        let turn = r.turn;
        advance(&mut r, &mut steps, |r| r.turn > turn);
        assert!(r.turn_attribute_modifiers.is_empty());
        assert_eq!(r.regions[0].cards[0].controller, 2);
        for s in 0..4 {
            assert_eq!(
                r.view(s, r.pacing.last_server_now_ms).game.regions[0].characters[0].current_renown,
                None
            );
        }
    }
    json!({"name":format!("renown-{kind}"),"seed":"9007199254740993","syntheticInitialLayout":true,"syntheticInitialFunding":true,"syntheticInitialWindow":matches!(kind,"normal-win"|"merged-decline"|"renown-win"),"postInitialStateInjection":false,"publicNaturalUiAcceptance":false,"steps":steps})
}

fn draft() -> deck::DeckDraft {
    let mut d = msjc01::draft(25, 50);
    d.society_id = None;
    d.cards = [
        "JC001", "JC002", "JC003", "JC004", "JC005", "JC006", "JC007", "XQ03",
    ]
    .into_iter()
    .map(|id| catalog::DeckEntry {
        card_id: id.into(),
        count: 3,
    })
    .collect();
    d.cards.push(catalog::DeckEntry {
        card_id: "JC008".into(),
        count: 1,
    });
    d.cards.extend(
        [
            "JC075", "JC070", "JC076", "JC074", "XQ12", "XQ16", "JC036", "JC104",
        ]
        .into_iter()
        .map(|id| catalog::DeckEntry {
            card_id: id.into(),
            count: 3,
        }),
    );
    d.cards.push(catalog::DeckEntry {
        card_id: "JZ27".into(),
        count: 1,
    });
    deck::validate(d).unwrap()
}
fn own_card<'a>(g: &'a Game, seat: usize, a: &Action) -> Option<&'a Card> {
    a.card_id.as_ref().and_then(|id| {
        g.players[seat]
            .hand
            .iter()
            .chain(g.regions.iter().flat_map(|r| &r.cards))
            .find(|c| c.id == *id && c.controller == seat)
    })
}
fn natural_actions(seed: u64) -> Vec<(usize, Action)> {
    let d = draft();
    let mut g = Game::new_with_deck(
        "888888888888888888880118".into(),
        "NATURAL".into(),
        "teams".into(),
        "P0".into(),
        d.clone(),
        seed,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), d.clone()).unwrap();
    }
    let mut actions = vec![];
    for s in 0..4 {
        let a = Action::new("ready");
        g.apply(s, a.clone()).unwrap();
        actions.push((s, a));
    }
    let a = Action::new("start");
    g.apply(0, a.clone()).unwrap();
    actions.push((0, a));
    let (mut deployed, mut revealed, mut granted, mut reward_declared) =
        (false, false, false, false);
    let mut completed_turn = None;
    for _ in 0..1800 {
        deployed |= g.regions[2]
            .cards
            .iter()
            .any(|c| c.controller == 0 && c.definition == "JC070" && !c.face_down);
        revealed |= g.regions[2]
            .cards
            .iter()
            .any(|c| c.controller == 0 && c.definition == "JC076" && !c.face_down);
        granted |= g.turn_attribute_modifiers.iter().any(|m| m.grants_renown);
        if deployed
            && revealed
            && granted
            && reward_declared
            && g.stack.is_empty()
            && g.pending.is_none()
            && g.regions[2].influence[0] > 0
        {
            completed_turn.get_or_insert(g.turn);
        }
        if completed_turn.is_some_and(|turn| g.turn > turn) {
            assert!(g.turn_attribute_modifiers.is_empty());
            return actions;
        }
        assert!(g.status=="playing" && g.turn<=20,"renown natural seed {seed} stalled: turn={} paid=({deployed},{revealed},{granted}) reward={reward_declared}",g.turn);
        let (seat, a) = if let Some(p) = g.pending.as_ref() {
            if p.choice.kind == "mulligan" {
                (
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(vec![]),
                        ..Action::new("choose")
                    },
                )
            } else if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.event==Some(Event::RegionConfrontationsEnded))
            {
                let accept = deployed && revealed && granted;
                reward_declared |= accept;
                (
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(if accept {
                            vec!["accept".into()]
                        } else {
                            vec![]
                        }),
                        ..Action::new("choose")
                    },
                )
            } else {
                pick_choice(&g)
            }
        } else {
            let legal = (0..4).map(|s| g.legal_actions(s)).collect::<Vec<_>>();
            let mut chosen = None;
            for s in 0..4 {
                if g.players[s].assets.len() < 5 {
                    let white = g.players[s]
                        .assets
                        .iter()
                        .filter(|c| catalog::card(&c.definition).color == "白")
                        .count();
                    let asset = legal[s]
                        .iter()
                        .filter(|a| a.action.kind == "asset")
                        .filter_map(|a| own_card(&g, s, &a.action).map(|c| (a, c)))
                        .filter(|(_, c)| {
                            let count = g.players[s]
                                .hand
                                .iter()
                                .filter(|h| h.definition == c.definition)
                                .count();
                            !matches!(c.definition.as_str(), "JC070" | "JC076" | "JC074")
                                || count > 1
                        })
                        .min_by_key(|(_, c)| {
                            if white < 2 && catalog::card(&c.definition).color == "白" {
                                0
                            } else if catalog::card(&c.definition).color != "白" {
                                1
                            } else {
                                2
                            }
                        });
                    if let Some((a, _)) = asset {
                        chosen = Some((s, a.action.clone()));
                        break;
                    }
                }
                if s == 0 {
                    if let Some(a) = legal[s].iter().find(|a| {
                        let def = own_card(&g, s, &a.action).map(|c| c.definition.as_str());
                        let want = match (a.action.kind.as_str(), def) {
                            ("deploy", Some("JC070")) => !deployed,
                            ("conceal", Some("JC076")) => {
                                !revealed
                                    && !g.regions[2].cards.iter().any(|c| c.definition == "JC076")
                            }
                            ("reveal", Some("JC076")) => !revealed,
                            ("play", Some("JC074")) => {
                                !granted
                                    && g.regions[2].cards.iter().any(|c| {
                                        c.definition == "JC070"
                                            && !c.face_down
                                            && c.controller == 0
                                            && a.action.target_id.as_deref() == Some(&c.id)
                                    })
                            }
                            _ => false,
                        };
                        want && a.action.region.is_none_or(|r| r == 2)
                    }) {
                        chosen = Some((s, a.action.clone()));
                        break;
                    }
                }
            }
            chosen.unwrap_or_else(|| {
                (0..4)
                    .find_map(|s| {
                        legal[s]
                            .iter()
                            .find(|a| a.action.kind == "pass")
                            .map(|a| (s, a.action.clone()))
                    })
                    .unwrap()
            })
        };
        g.apply(seat, a.clone()).unwrap();
        actions.push((seat, a));
    }
    panic!("bounded renown natural game");
}
fn natural_case() -> Value {
    let seed = 1u64;
    let actions = natural_actions(seed);
    let d = draft();
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "888888888888888888880118".into(),
            "NATURAL".into(),
            "teams".into(),
            "P0".into(),
            d.clone(),
            seed,
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
            seed.to_string()
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
    let mut milestones = vec![];
    let mut observed = [false; 4];
    let mut hidden_instance = None;
    for (seat, a) in actions {
        let was_renown = r
            .stack
            .last()
            .and_then(|s| s.frame.as_ref())
            .is_some_and(|f| f.ability_key == "renown");
        let old_stack = r.stack.len();
        let old_influence = r.regions.get(2).map(|region| region.influence);
        apply_game(&mut r, &mut steps, seat, a);
        for c in r
            .regions
            .get(2)
            .into_iter()
            .flat_map(|region| &region.cards)
        {
            if c.definition == "JC076" && c.face_down {
                hidden_instance = Some(c.id.clone());
            }
            if c.controller == 0 && !c.face_down && c.definition == "JC070" && !observed[0] {
                observed[0] = true;
                milestones.push(json!({"step":steps.len()-1,"turn":r.turn,"kind":"JC070-paid-deployment","instance":c.id}));
            }
            if c.controller == 0 && !c.face_down && c.definition == "JC076" && !observed[1] {
                assert_ne!(hidden_instance.as_ref(), Some(&c.id));
                assert!(hidden_instance.is_some());
                observed[1] = true;
                milestones.push(json!({"step":steps.len()-1,"turn":r.turn,"kind":"JC076-paid-reveal-new-instance","hidden":hidden_instance,"revealed":c.id}));
            }
        }
        if !observed[2] && r.turn_attribute_modifiers.iter().any(|m| m.grants_renown) {
            observed[2] = true;
            milestones.push(json!({"step":steps.len()-1,"turn":r.turn,"kind":"JC074-resolved-ordinary-investigation-and-renown","modifiers":r.turn_attribute_modifiers}));
        }
        if was_renown && r.stack.len() < old_stack && !observed[3] {
            assert_eq!(r.regions[2].influence[0], old_influence.unwrap()[0] + 1);
            observed[3] = true;
            milestones.push(json!({"step":steps.len()-1,"turn":r.turn,"kind":"one-merged-renown-influence","before":old_influence,"after":r.regions[2].influence}));
        }
    }
    assert_eq!(observed, [true; 4]);
    assert!(r.turn_attribute_modifiers.is_empty());
    assert!(r.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC074"));
    assert!(r.log.iter().any(|l| l.text.contains("声望")));
    json!({"name":"renown-natural-four-seat-paid-three-cards","seed":seed.to_string(),"syntheticInitialLayout":false,"syntheticInitialFunding":false,"postInitialStateInjection":false,"zeroNeutralFiftyCardDeck":true,"publicNaturalUiAcceptance":false,"finalTurn":r.turn,"milestones":milestones,"steps":steps})
}
pub fn cases() -> impl Iterator<Item = Value> {
    [
        "merged-decline",
        "normal-win",
        "renown-win",
        "response-identity",
        "control-cleanup",
    ]
    .into_iter()
    .map(boundary)
    .chain(std::iter::once_with(natural_case))
}
