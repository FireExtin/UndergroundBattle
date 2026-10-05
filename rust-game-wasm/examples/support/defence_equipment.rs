//! Boundary fixtures disclose initial setup; every subsequent action uses the Room ABI.
//! The separate natural case uses only legal fifty-card decks and real dealt cards.
use super::*;
use hegemony_server::model::Card;
fn resources(g: &Game, s: usize) -> u32 {
    g.players[s].assets.iter().filter(|c| !c.exhausted).count() as u32
}
fn board<'a>(g: &'a Game, id: &str) -> Option<(usize, &'a Card)> {
    g.regions
        .iter()
        .enumerate()
        .find_map(|(r, region)| region.cards.iter().find(|c| c.id == id).map(|c| (r, c)))
}
fn field(g: &mut Game, id: &str, s: usize) -> String {
    let c = g.make_card(id, s);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn held(g: &mut Game, id: &str, s: usize) -> String {
    let c = g.make_card(id, s);
    let id = c.id.clone();
    g.players[s].hand.push(c);
    id
}
fn fund(g: &mut Game, id: &str, s: usize, n: usize) {
    for _ in 0..n {
        let c = g.make_card(id, s);
        g.players[s].assets.push(c);
    }
}
fn pass_top(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    let n = r.stack.len();
    assert!(n > 0);
    for _ in 0..80 {
        if r.pending.is_some() {
            let (s, a) = pick_choice(&r.game);
            apply_game(r, steps, s, a);
        } else if r.stack.len() < n {
            return;
        } else {
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
    }
    panic!("bounded defence stack");
}
fn advance(r: &mut RoomEnvelope, steps: &mut Vec<Value>, done: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..650 {
        if done(r) {
            return;
        }
        let (s, a) = if r.pending.is_some() {
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
        apply_game(r, steps, s, a);
    }
    panic!("bounded defence advance");
}
fn cast(
    r: &mut RoomEnvelope,
    steps: &mut Vec<Value>,
    s: usize,
    id: &str,
    target: &str,
    option: Option<&str>,
) {
    apply_game(
        r,
        steps,
        s,
        Action {
            card_id: Some(id.into()),
            target_id: Some(target.into()),
            option: option.map(str::to_string),
            ..Action::new("play")
        },
    );
    pass_top(r, steps);
}
fn transfer(id: &str, target: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        target_id: Some(target.into()),
        ability_id: Some("reattach".into()),
        ..Action::new("activate")
    }
}
fn reject(r: &mut RoomEnvelope, steps: &mut Vec<Value>, s: usize, a: Action) {
    let c = RoomCommand {
        command_id: format!("oracle:rejected:{}", steps.len()),
        expected_version: r.revision,
        action: a.into(),
    };
    let now = r.pacing.last_server_now_ms;
    let t = r.transition(s, Some(c.clone()), now).unwrap();
    assert_eq!(t.outcome, "rejected");
    assert!(!t.changed);
    record_transition(r, steps, "applyRoom", json!([s, c, now.to_string()]), t);
}
fn boundary(kind: &str) -> Value {
    let mut g = attachment_initial("teams", "888888888888888888880121");
    let host = field(
        &mut g,
        if kind == "paid-four" {
            "JZ27"
        } else if kind == "shield-return" {
            "JC003"
        } else {
            "JC125"
        },
        2,
    );
    let next = field(&mut g, "JC003", 2);
    let hidden = field(&mut g, "JC125", 2);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    let blocked = field(&mut g, "JZ08", 2);
    let cat_board = field(&mut g, "JC112", 0);
    let c = g.make_card("JC125", 0);
    let outside = c.id.clone();
    g.regions[1].cards.push(c);
    let shield = held(&mut g, "JC078", 0);
    let blood = held(&mut g, "XQ14", 0);
    let vest = held(&mut g, "XQ47", 0);
    let cat = held(&mut g, "JC112", 0);
    let fire = held(&mut g, "JC102", 0);
    let fire2 = held(&mut g, "JC102", 0);
    let control = held(&mut g, "JC036", 0);
    let remove = held(&mut g, "JC005", 0);
    let reply = held(
        &mut g,
        if kind == "shield-hide" {
            "JC063"
        } else if kind == "vest-source" {
            "JC005"
        } else {
            "JC006"
        },
        2,
    );
    fund(&mut g, "XQ16", 0, 8);
    fund(&mut g, "JC002", 0, 2);
    fund(&mut g, "JC075", 0, 4);
    fund(&mut g, "JC104", 0, 4);
    fund(&mut g, "JC002", 2, 4);
    fund(&mut g, "JC063", 2, 4);
    if kind == "cat-loyalty" {
        g.players[0].assets.clear();
        fund(&mut g, "JC075", 0, 3);
    }
    let purple = held(&mut g, "JC104", 0);
    let mut r = RoomEnvelope::from_game(g);
    let mut steps = vec![step(
        &r,
        "initialFixture",
        json!([serde_json::to_string(&r).unwrap()]),
        0,
    )];
    let mut milestones = vec![];
    match kind {
        "paid-four" => {
            cast(&mut r, &mut steps, 0, &blood, &host, None);
            cast(&mut r, &mut steps, 0, &vest, &host, None);
            let vest_id = r.attachments.last().unwrap().card.id.clone();
            apply_game(
                &mut r,
                &mut steps,
                0,
                Action {
                    card_id: Some(cat),
                    region: Some(0),
                    ..Action::new("deploy")
                },
            );
            pass_top(&mut r, &mut steps);
            let cat_id = r.regions[0]
                .cards
                .iter()
                .find(|c| c.definition == "JC112" && c.id != cat_board)
                .unwrap()
                .id
                .clone();
            cast(&mut r, &mut steps, 0, &shield, &host, None);
            apply_game(
                &mut r,
                &mut steps,
                0,
                Action {
                    card_id: Some(cat_id),
                    ability_id: Some("reduce-next-magic".into()),
                    ..Action::new("activate")
                },
            );
            assert!(r.stack.is_empty());
            let before = resources(&r.game, 0);
            cast(&mut r, &mut steps, 0, &fire, &host, None);
            assert_eq!(resources(&r.game, 0), before - 1);
            cast(&mut r, &mut steps, 0, &fire2, &host, None);
            assert_eq!(resources(&r.game, 0), before - 3);
            assert_eq!(board(&r.game, &host).unwrap().1.damage, 0);
            for s in 0..4 {
                let v = r.game.view(s);
                let c = v.regions[0]
                    .characters
                    .iter()
                    .find(|c| c.instance_id == host)
                    .unwrap();
                assert_eq!(c.defense, Some(5));
                assert_eq!(c.icons.unwrap().combat, 3);
                assert_eq!(c.current_damage_prevention, Some(true));
            }
            let before = resources(&r.game, 0);
            apply_game(&mut r, &mut steps, 0, transfer(&vest_id, &outside));
            pass_top(&mut r, &mut steps);
            assert_eq!(resources(&r.game, 0), before - 2);
            assert_eq!(
                r.attachments
                    .iter()
                    .find(|a| a.card.id == vest_id)
                    .unwrap()
                    .host_id,
                outside
            );
            milestones.push(json!({"kind":"paid-four-plus-same-instance-cross-region-transfer","step":steps.len()-1,"vest":vest_id,"host":host}));
            let turn = r.turn;
            advance(&mut r, &mut steps, |r| r.turn > turn);
            assert!(r.turn_attribute_modifiers.is_empty());
            assert!(r.modifiers.is_empty());
        }
        "shield-return" | "shield-hide" => {
            if kind == "shield-hide" {
                cast(&mut r, &mut steps, 0, &shield, &host, None);
            } else {
                apply_game(
                    &mut r,
                    &mut steps,
                    0,
                    Action {
                        card_id: Some(shield),
                        target_id: Some(host.clone()),
                        ..Action::new("play")
                    },
                );
            }
            advance(&mut r, &mut steps, |r| r.priority_team == r.game.team(2));
            cast(
                &mut r,
                &mut steps,
                2,
                &reply,
                &host,
                if kind == "shield-hide" {
                    Some("hide")
                } else {
                    None
                },
            );
            if !r.stack.is_empty() {
                pass_top(&mut r, &mut steps);
            }
            assert!(r.turn_attribute_modifiers.is_empty());
            if kind == "shield-hide" {
                for s in 0..4 {
                    let c = r.game.view(s).regions[0]
                        .characters
                        .iter()
                        .find(|c| c.instance_id != next && c.face_down)
                        .unwrap()
                        .clone();
                    assert!(c.current_damage_prevention.is_none());
                }
            } else {
                assert!(board(&r.game, &host).is_none());
                assert!(r.players[2]
                    .hand
                    .iter()
                    .any(|c| c.definition == "JC003" && c.id != host));
            }
        }
        "vest-target" | "vest-source" | "vest-lost-defense" | "roles-targets" => {
            if kind == "roles-targets" {
                reject(
                    &mut r,
                    &mut steps,
                    0,
                    Action {
                        card_id: Some(vest.clone()),
                        target_id: Some(host.clone()),
                        ability_id: Some("reattach".into()),
                        ..Action::new("play")
                    },
                );
                reject(
                    &mut r,
                    &mut steps,
                    0,
                    Action {
                        card_id: Some(blood),
                        target_id: Some(host.clone()),
                        ..Action::new("play")
                    },
                );
            }
            cast(&mut r, &mut steps, 0, &vest, &host, None);
            let id = r.attachments.last().unwrap().card.id.clone();
            if kind == "roles-targets" {
                for target in [&host, &hidden, &blocked, &cat_board, &id] {
                    reject(&mut r, &mut steps, 0, transfer(&id, target));
                }
                reject(
                    &mut r,
                    &mut steps,
                    0,
                    Action {
                        card_id: Some(id.clone()),
                        target_id: Some(next.clone()),
                        ability_id: Some("attach".into()),
                        ..Action::new("activate")
                    },
                );
                apply_game(&mut r, &mut steps, 0, transfer(&id, &next));
                pass_top(&mut r, &mut steps);
            } else if kind == "vest-lost-defense" {
                cast(&mut r, &mut steps, 0, &fire, &host, None);
                assert_eq!(board(&r.game, &host).unwrap().1.damage, 1);
                apply_game(&mut r, &mut steps, 0, transfer(&id, &next));
                pass_top(&mut r, &mut steps);
                assert!(board(&r.game, &host).is_none());
                assert_eq!(r.attachments[0].card.id, id);
                assert_eq!(r.attachments[0].host_id, next);
            } else {
                let before = resources(&r.game, 0);
                apply_game(&mut r, &mut steps, 0, transfer(&id, &next));
                advance(&mut r, &mut steps, |r| r.priority_team == r.game.team(2));
                cast(
                    &mut r,
                    &mut steps,
                    2,
                    &reply,
                    if kind == "vest-source" { &id } else { &next },
                    None,
                );
                pass_top(&mut r, &mut steps);
                assert_eq!(resources(&r.game, 0), before - 2);
                if kind == "vest-source" {
                    assert!(r.attachments.is_empty());
                } else {
                    assert_eq!(r.attachments[0].host_id, host);
                    assert_eq!(r.attachments[0].card.id, id);
                }
            }
        }
        "blood-type" => {
            cast(&mut r, &mut steps, 0, &control, &host, None);
            let id = r.attachments.last().unwrap().card.id.clone();
            cast(&mut r, &mut steps, 0, &blood, &host, None);
            assert_eq!(r.game.defense(board(&r.game, &host).unwrap().1, 0), 2);
            cast(&mut r, &mut steps, 0, &remove, &id, None);
            assert!(r.attachments.is_empty());
            assert_eq!(board(&r.game, &host).unwrap().1.controller, 2);
            assert_eq!(r.game.defense(board(&r.game, &host).unwrap().1, 0), 1);
        }
        "cat-loyalty" => {
            let before = resources(&r.game, 0);
            apply_game(
                &mut r,
                &mut steps,
                0,
                Action {
                    card_id: Some(cat),
                    region: Some(0),
                    ..Action::new("deploy")
                },
            );
            pass_top(&mut r, &mut steps);
            assert_eq!(resources(&r.game, 0), before - 1);
            let id = r.regions[0]
                .cards
                .iter()
                .find(|c| c.definition == "JC112" && c.id != cat_board)
                .unwrap()
                .id
                .clone();
            apply_game(
                &mut r,
                &mut steps,
                0,
                Action {
                    card_id: Some(id),
                    ability_id: Some("reduce-next-magic".into()),
                    ..Action::new("activate")
                },
            );
            let a = Action {
                card_id: Some(fire),
                target_id: Some(next.clone()),
                ..Action::new("play")
            };
            reject(&mut r, &mut steps, 0, a.clone());
            assert_eq!(r.modifiers[0].uses, 1);
            apply_game(
                &mut r,
                &mut steps,
                0,
                Action {
                    card_id: Some(purple),
                    ..Action::new("asset")
                },
            );
            let before = resources(&r.game, 0);
            apply_game(&mut r, &mut steps, 0, a);
            pass_top(&mut r, &mut steps);
            assert_eq!(resources(&r.game, 0), before - 1);
            assert_eq!(r.modifiers[0].uses, 0);
        }
        _ => unreachable!(),
    }
    json!({"name":format!("defence-equipment-{kind}"),"seed":r.seed.to_string(),"syntheticInitialLayout":true,"syntheticInitialFunding":true,"postInitialStateInjection":false,"publicNaturalUiAcceptance":false,"milestones":milestones,"steps":steps})
}
fn draft() -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.id = "local-defence-equipment".into();
    d.name = "Local defence equipment".into();
    d.society_id = None;
    d.cards = [
        "JC001", "JC002", "JC003", "JC004", "JC005", "JC006", "JC007", "JC075", "JC125", "JC076",
        "JC104", "XQ16", "JC036", "JC078", "XQ14", "XQ47",
    ]
    .into_iter()
    .map(|id| catalog::DeckEntry {
        card_id: id.into(),
        count: 3,
    })
    .collect();
    d.cards
        .retain(|c| c.card_id != "JC004" && c.card_id != "JC076");
    d.cards.push(catalog::DeckEntry {
        card_id: "JC112".into(),
        count: 3,
    });
    d.cards.push(catalog::DeckEntry {
        card_id: "JC102".into(),
        count: 3,
    });
    d.cards.push(catalog::DeckEntry {
        card_id: "JC008".into(),
        count: 2,
    });
    deck::validate(d).unwrap()
}
fn own<'a>(g: &'a Game, s: usize, a: &Action) -> Option<&'a Card> {
    a.card_id.as_ref().and_then(|id| {
        g.players[s]
            .hand
            .iter()
            .chain(g.regions.iter().flat_map(|r| &r.cards))
            .chain(g.attachments.iter().map(|a| &a.card))
            .find(|c| c.id == *id && c.controller == s)
    })
}
fn natural_actions(seed: u64) -> Vec<(usize, Action)> {
    let d = draft();
    let mut g = Game::new_with_deck(
        "888888888888888888880122".into(),
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
    let (mut moved, mut shielded, mut reduced, mut fired) = (false, false, false, false);
    let mut complete_turn = None;
    for _ in 0..3400 {
        let hosts = g.regions[2]
            .cards
            .iter()
            .filter(|c| c.definition == "JC125" && c.controller == 0 && !c.face_down)
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        let first = hosts.first();
        let second = hosts.get(1);
        let control = g.attachments.iter().find(|a| a.card.definition == "JC036");
        let blood = g.attachments.iter().find(|a| a.card.definition == "XQ14");
        let vest = g.attachments.iter().find(|a| a.card.definition == "XQ47");
        let cat = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "JC112" && !c.face_down);
        if fired && g.stack.is_empty() && g.pending.is_none() {
            complete_turn.get_or_insert(g.turn);
        }
        if complete_turn.is_some_and(|t| g.turn > t) {
            assert!(g.turn_attribute_modifiers.is_empty());
            assert!(g.modifiers.is_empty());
            return actions;
        }
        assert!(
            g.status == "playing" && g.turn <= 22,
            "natural defence stalled seed={seed} turn={} flags={:?} board={:?} attachments={:?} hand={:?} assets={:?}",
            g.turn,
            (moved, shielded, reduced, fired),hosts,g.attachments.iter().map(|a|(&a.card.definition,&a.host_id)).collect::<Vec<_>>(),g.players[0].hand.iter().map(|c|&c.definition).collect::<Vec<_>>(),g.players[0].assets.iter().map(|c|&c.definition).collect::<Vec<_>>()
        );
        let (seat, a) = if let Some(p) = &g.pending {
            if p.choice.kind == "mulligan"
                || matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.event==Some(rules::Event::RegionConfrontationsEnded))
            {
                (
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(vec![]),
                        ..Action::new("choose")
                    },
                )
            } else if p.choice.kind == "discard" {
                let critical = ["JC125", "JC036", "JC078", "XQ14", "XQ47", "JC112", "JC102"];
                let mut selected = vec![];
                let mut counts = BTreeMap::new();
                for c in &g.players[p.seat].hand {
                    *counts.entry(c.definition.clone()).or_insert(0usize) += 1;
                }
                for _ in 0..p.choice.min.unwrap_or(0) {
                    let candidate = p
                        .choice
                        .options
                        .iter()
                        .filter(|o| !selected.contains(&o.id))
                        .find(|o| {
                            let c = g.players[p.seat]
                                .hand
                                .iter()
                                .find(|c| c.id == o.id)
                                .unwrap();
                            (!critical.contains(&c.definition.as_str())
                                && !(catalog::card(&c.definition).color == "蓝"
                                    && g.players[p.seat]
                                        .assets
                                        .iter()
                                        .filter(|a| catalog::card(&a.definition).color == "蓝")
                                        .count()
                                        < 2)
                                && !(catalog::card(&c.definition).color == "白"
                                    && g.players[p.seat]
                                        .assets
                                        .iter()
                                        .all(|a| catalog::card(&a.definition).color != "白")))
                                || counts[&c.definition]
                                    > if c.definition == "JC125" { 2 } else { 1 }
                        })
                        .or_else(|| p.choice.options.iter().find(|o| !selected.contains(&o.id)))
                        .unwrap();
                    let c = g.players[p.seat]
                        .hand
                        .iter()
                        .find(|c| c.id == candidate.id)
                        .unwrap();
                    *counts.get_mut(&c.definition).unwrap() -= 1;
                    selected.push(candidate.id.clone());
                }
                (
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(selected),
                        ..Action::new("choose")
                    },
                )
            } else {
                pick_choice(&g)
            }
        } else {
            let legal = (0..4).map(|s| g.legal_actions(s)).collect::<Vec<_>>();
            let mut chosen = None;
            if g.stack.is_empty() {
                for s in 0..4 {
                    let count = |color: &str| {
                        g.players[s]
                            .assets
                            .iter()
                            .filter(|c| catalog::card(&c.definition).color == color)
                            .count()
                    };
                    let (white, blue, purple) = (count("白"), count("蓝"), count("紫"));
                    if g.players[s].assets.len() < 10
                        || (s == 0 && (white < 1 || blue < 2 || purple < 1))
                    {
                        let candidate = legal[s]
                            .iter()
                            .filter(|a| a.action.kind == "asset")
                            .filter_map(|a| own(&g, s, &a.action).map(|c| (a, c)))
                            .filter(|(_, c)| {
                                let needed = match c.definition.as_str() {
                                    "JC125" => 2usize.saturating_sub(hosts.len()),
                                    "JC036" => usize::from(control.is_none()),
                                    "XQ14" => usize::from(blood.is_none()),
                                    "XQ47" => usize::from(vest.is_none()),
                                    "JC078" => usize::from(!shielded),
                                    "JC112" => usize::from(cat.is_none() && !reduced),
                                    "JC102" => usize::from(!fired),
                                    _ => 0,
                                };
                                g.players[s]
                                    .hand
                                    .iter()
                                    .filter(|h| h.definition == c.definition)
                                    .count()
                                    > needed
                            })
                            .min_by_key(|(_, c)| {
                                let color = &catalog::card(&c.definition).color;
                                if blue < 2 && color == "蓝" {
                                    0
                                } else if white < 1 && color == "白" {
                                    1
                                } else if purple < 1 && color == "紫" {
                                    2
                                } else {
                                    3
                                }
                            });
                        if let Some((a, _)) = candidate {
                            chosen = Some((s, a.action.clone()));
                            break;
                        }
                    }
                    if s == 0 {
                        let end_ready = ["JC078", "JC112", "JC102"]
                            .iter()
                            .all(|id| g.players[0].hand.iter().any(|c| c.definition == *id))
                            && resources(&g, 0) >= 3
                            && purple >= 1;
                        let target = first.cloned();
                        if let Some(a) = legal[0].iter().find(|a| {
                            let def = own(&g, 0, &a.action).map(|c| c.definition.as_str());
                            let want = match (a.action.kind.as_str(), def) {
                                ("deploy", Some("JC125")) => hosts.len() < 2,
                                ("play", Some("JC036")) => {
                                    control.is_none() && a.action.target_id == target
                                }
                                ("play", Some("XQ14")) => {
                                    blood.is_none()
                                        && control.is_some()
                                        && a.action.target_id == target
                                }
                                ("play", Some("XQ47")) => {
                                    vest.is_none()
                                        && first.is_some()
                                        && a.action.target_id == target
                                }
                                ("activate", Some("XQ47")) => {
                                    !moved
                                        && blood.is_some()
                                        && second.is_some()
                                        && a.action.target_id.as_ref() == second
                                }
                                ("deploy", Some("JC112")) => {
                                    moved && !reduced && cat.is_none() && end_ready
                                }
                                ("play", Some("JC078")) => {
                                    !shielded && cat.is_some() && a.action.target_id == target
                                }
                                ("activate", Some("JC112")) => shielded && !reduced,
                                ("play", Some("JC102")) => {
                                    reduced && !fired && a.action.target_id == target
                                }
                                _ => false,
                            };
                            want && a.action.region.is_none_or(|r| r == 2)
                        }) {
                            chosen = Some((0, a.action.clone()));
                            break;
                        }
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
        let def = own(&g, seat, &a).map(|c| c.definition.clone());
        let before = resources(&g, seat);
        g.apply(seat, a.clone()).unwrap();
        moved |= a.kind == "activate" && def.as_deref() == Some("XQ47");
        shielded |= a.kind == "play" && def.as_deref() == Some("JC078");
        reduced |= a.kind == "activate" && def.as_deref() == Some("JC112");
        if a.kind == "play" && def.as_deref() == Some("JC102") {
            assert_eq!(resources(&g, seat), before - 1);
            fired = true;
        }
        actions.push((seat, a));
    }
    panic!(
        "bounded natural defence turn={} flags={:?} assets={} hand={:?} attachments={:?}",
        g.turn,
        (moved, shielded, reduced, fired),
        g.players[0].assets.len(),
        g.players[0]
            .hand
            .iter()
            .map(|c| &c.definition)
            .collect::<Vec<_>>(),
        g.attachments
            .iter()
            .map(|a| &a.card.definition)
            .collect::<Vec<_>>()
    );
}
fn natural_case() -> Value {
    let seed = 21;
    let actions = natural_actions(seed);
    let d = draft();
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "888888888888888888880122".into(),
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
    let mut seen = [false; 8];
    let mut pending_transfer = None;
    for (seat, a) in actions {
        let def = own(&r.game, seat, &a).map(|c| c.definition.clone());
        let old = r.stack.len();
        let resolving = r
            .stack
            .last()
            .and_then(|s| s.frame.as_ref())
            .map(|f| f.source.card.definition.clone());
        let prior_uses = r.modifiers.iter().map(|m| m.uses).sum::<u32>();
        let before = resources(&r.game, seat);
        let moving = if a.kind == "activate" && def.as_deref() == Some("XQ47") {
            Some((a.card_id.clone().unwrap(), a.target_id.clone().unwrap()))
        } else {
            None
        };
        apply_game(&mut r, &mut steps, seat, a.clone());
        if r.game.regions.len() <= 2 {
            continue;
        }
        for (index, id) in [(0, "JC036"), (1, "XQ14"), (2, "XQ47")] {
            if !seen[index] && r.attachments.iter().any(|a| a.card.definition == id) {
                seen[index] = true;
                milestones.push(json!({"step":steps.len()-1,"kind":format!("{id}-actual-paid-attachment"),"turn":r.turn}));
            }
        }
        if let Some((id, target)) = moving {
            assert_eq!(resources(&r.game, seat), before - 2);
            milestones.push(json!({"step":steps.len()-1,"kind":"XQ47-paid-standard-transfer-declaration","instance":id,"target":target}));
            pending_transfer = Some((id, target));
        }
        if !seen[7] && old > r.stack.len() && resolving.as_deref() == Some("XQ47") {
            if let Some((id, target)) = &pending_transfer {
                let attachment = r.attachments.iter().find(|a| a.card.id == *id).unwrap();
                assert_eq!(&attachment.host_id, target);
                assert_eq!((attachment.card.owner, attachment.card.controller), (0, 0));
                seen[7] = true;
                milestones.push(json!({"step":steps.len()-1,"kind":"XQ47-resolved-same-instance-transfer","instance":id,"target":target}));
            }
        }
        if !seen[3]
            && r.game.regions[2]
                .cards
                .iter()
                .any(|c| c.definition == "JC112" && !c.face_down)
        {
            seen[3] = true;
            milestones.push(
                json!({"step":steps.len()-1,"kind":"JC112-actual-paid-deployment","turn":r.turn}),
            );
        }
        if !seen[4] && r.turn_attribute_modifiers.iter().any(|m| m.prevents_damage) {
            seen[4] = true;
            for viewer in 0..4 {
                assert!(r.game.view(viewer).regions[2]
                    .characters
                    .iter()
                    .any(|c| c.current_damage_prevention == Some(true)));
            }
            milestones.push(json!({"step":steps.len()-1,"kind":"JC078-resolved-turn-damage-prevention","turn":r.turn}));
        }
        if !seen[5] && a.kind == "activate" && def.as_deref() == Some("JC112") {
            seen[5] = true;
            assert!(r.stack.is_empty());
            assert_eq!(
                r.modifiers.iter().map(|m| m.uses).sum::<u32>(),
                prior_uses + 1
            );
            milestones.push(json!({"step":steps.len()-1,"kind":"JC112-real-sacrifice-immediate-reducer","turn":r.turn}));
        }
        if a.kind == "play" && def.as_deref() == Some("JC102") {
            assert_eq!(resources(&r.game, seat), before - 1);
            assert_eq!(
                r.modifiers.iter().map(|m| m.uses).sum::<u32>(),
                prior_uses - 1
            );
        }
        if !seen[6] && old > r.stack.len() && resolving.as_deref() == Some("JC102") {
            seen[6] = true;
            assert!(r.game.log.iter().any(|l| l.text.contains("本回合伤害防止")));
            let host = r.game.regions[2]
                .cards
                .iter()
                .find(|c| c.definition == "JC125" && r.game.damage_prevented(c))
                .unwrap();
            assert_eq!(host.damage, 0);
            milestones.push(json!({"step":steps.len()-1,"kind":"JC102-real-discounted-damage-prevented","turn":r.turn,"host":host.id}));
        }
    }
    assert_eq!(seen, [true; 8]);
    assert!(r.turn_attribute_modifiers.is_empty());
    assert!(r.modifiers.is_empty());
    json!({"name":"defence-equipment-natural-four-seat-paid-four-cards","seed":seed.to_string(),"syntheticInitialLayout":false,"syntheticInitialFunding":false,"postInitialStateInjection":false,"publicNaturalUiAcceptance":false,"legalFiftyCardDeck":true,"finalTurn":r.turn,"milestones":milestones,"steps":steps})
}
pub fn cases() -> impl Iterator<Item = Value> {
    [
        "paid-four",
        "shield-return",
        "shield-hide",
        "vest-target",
        "vest-source",
        "vest-lost-defense",
        "roles-targets",
        "blood-type",
        "cat-loyalty",
    ]
    .into_iter()
    .map(boundary)
    .chain(std::iter::once_with(natural_case))
}
