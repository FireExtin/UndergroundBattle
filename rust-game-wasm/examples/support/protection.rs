//! Explicit starting layouts/funding only for boundary cases. Subsequent steps
//! use the real Room ABI, complete journals, persisted decode and four views.
use super::*;
use hegemony_server::model::Card;

fn field(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn held(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn fund(g: &mut Game, def: &str, seat: usize, n: usize) {
    for _ in 0..n {
        let c = g.make_card(def, seat);
        g.players[seat].assets.push(c);
    }
}
fn advance(r: &mut RoomEnvelope, steps: &mut Vec<Value>, done: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..600 {
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
    panic!("bounded protection advance");
}
fn cast(
    r: &mut RoomEnvelope,
    steps: &mut Vec<Value>,
    seat: usize,
    source: &str,
    target: &str,
    option: Option<&str>,
) {
    apply_game(
        r,
        steps,
        seat,
        Action {
            card_id: Some(source.into()),
            target_id: Some(target.into()),
            option: option.map(str::to_string),
            ..Action::new("play")
        },
    );
    attachment_pass_top(r, steps);
}
fn reject_step(r: &mut RoomEnvelope, steps: &mut Vec<Value>, seat: usize, action: Action) {
    let command = RoomCommand {
        command_id: format!("oracle:rejected:{}", steps.len()),
        expected_version: r.revision,
        action: action.into(),
    };
    let now = r.pacing.last_server_now_ms;
    let transition = r.transition(seat, Some(command.clone()), now).unwrap();
    assert_eq!(transition.outcome, "rejected");
    assert!(!transition.changed);
    record_transition(
        r,
        steps,
        "applyRoom",
        json!([seat, command, now.to_string()]),
        transition,
    );
}
fn boundary(kind: &str) -> Value {
    let mut g = attachment_initial("teams", "888888888888888888880119");
    let host = field(
        &mut g,
        if matches!(kind, "response-return" | "response-hide") {
            "JC003"
        } else {
            "JC125"
        },
        if matches!(kind, "foreign-host" | "response-return" | "response-hide") {
            2
        } else {
            0
        },
    );
    let hermit = if kind == "paid-four" {
        held(&mut g, "JC071", 0)
    } else {
        field(&mut g, "JC071", 0)
    };
    let barrier = field(&mut g, "JZ08", 2);
    let hidden = field(&mut g, "JC125", 2);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    let outside = g.make_card("JC125", 2);
    let outside_id = outside.id.clone();
    g.regions[1].cards.push(outside);
    let net = held(&mut g, "JC073", 0);
    let fire = held(&mut g, "JC102", 0);
    let wall = held(&mut g, "JC132", 0);
    let remove_net = held(&mut g, "JC005", 0);
    let reply = held(
        &mut g,
        if kind == "response-hide" {
            "JC063"
        } else {
            "JC006"
        },
        2,
    );
    fund(&mut g, "JC075", 0, 6);
    fund(&mut g, "JC002", 0, 8);
    fund(&mut g, "JC104", 0, 1); // JC102's printed purple loyalty, independent of Blood.
    fund(&mut g, "JC063", 2, 4);
    fund(&mut g, "JC002", 2, 4);
    if kind == "phase-loyalty" {
        g.players[0].assets.clear();
        fund(&mut g, "JC125", 0, 4);
        fund(&mut g, "JC104", 0, 1);
    }
    let purple_asset = if kind == "fire-purple-loyalty" {
        g.players[0].assets.clear();
        fund(&mut g, "JC002", 0, 2); // Enough money and Blood, but no purple loyalty.
        Some(held(&mut g, "JC104", 0))
    } else {
        None
    };
    let mut r = RoomEnvelope::from_game(g);
    let mut steps = vec![step(
        &r,
        "initialFixture",
        json!([serde_json::to_string(&r).unwrap()]),
        0,
    )];
    if kind == "paid-four" {
        apply_game(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(hermit.clone()),
                region: Some(0),
                ..Action::new("deploy")
            },
        );
        attachment_pass_top(&mut r, &mut steps);
        let hermit = r.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JC071")
            .unwrap()
            .id
            .clone();
        apply_game(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(hermit.clone()),
                target_id: Some(host.clone()),
                ability_id: Some("protect-local-character".into()),
                ..Action::new("activate")
            },
        );
        attachment_pass_top(&mut r, &mut steps);
        cast(&mut r, &mut steps, 0, &fire, &host, None);
        assert_eq!(
            r.regions[0]
                .cards
                .iter()
                .find(|c| c.id == host)
                .unwrap()
                .damage,
            1
        );
        cast(&mut r, &mut steps, 0, &net, &host, None);
        let net = r.attachments.last().unwrap().card.id.clone();
        for s in 0..4 {
            let v = r.view(s, r.pacing.last_server_now_ms).game;
            let c = v.regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == host)
                .unwrap();
            assert_eq!(c.current_barrier, Some(true));
            assert_eq!(c.defense, Some(2));
            assert_eq!(c.icons.unwrap().investigation, 1);
        }
        apply_game(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(wall),
                region: Some(0),
                ..Action::new("play")
            },
        );
        attachment_pass_top(&mut r, &mut steps);
        assert!(
            r.regions[0]
                .cards
                .iter()
                .find(|c| c.id == host)
                .unwrap()
                .exhausted
        );
        assert!(
            r.regions[0]
                .cards
                .iter()
                .find(|c| c.id == barrier)
                .unwrap()
                .exhausted
        );
        assert!(
            !r.regions[0]
                .cards
                .iter()
                .find(|c| c.id == hidden)
                .unwrap()
                .exhausted
        );
        assert!(
            !r.regions[1]
                .cards
                .iter()
                .find(|c| c.id == outside_id)
                .unwrap()
                .exhausted
        );
        cast(&mut r, &mut steps, 0, &remove_net, &net, None);
        assert!(r.attachments.is_empty());
        let turn = r.turn;
        advance(&mut r, &mut steps, |r| r.turn > turn);
        assert!(r.turn_attribute_modifiers.is_empty());
        let c = r.regions[0].cards.iter().find(|c| c.id == host).unwrap();
        assert_eq!(c.damage, 0);
    } else if kind == "foreign-host" {
        cast(&mut r, &mut steps, 0, &net, &host, None);
        let net = r.attachments.last().unwrap().card.id.clone();
        for s in 0..4 {
            let v = r.view(s, r.pacing.last_server_now_ms).game;
            let c = v.regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == host)
                .unwrap();
            assert_eq!(c.current_barrier, Some(true));
            assert_eq!(c.icons.unwrap().investigation, 0);
        }
        reject_step(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(fire),
                target_id: Some(host.clone()),
                ..Action::new("play")
            },
        );
        cast(&mut r, &mut steps, 0, &remove_net, &net, None);
        assert!(r.attachments.is_empty());
    } else if matches!(kind, "response-return" | "response-hide") {
        apply_game(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(fire),
                target_id: Some(host.clone()),
                ..Action::new("play")
            },
        );
        advance(&mut r, &mut steps, |r| r.priority_team == 1);
        cast(
            &mut r,
            &mut steps,
            2,
            &reply,
            &host,
            if kind == "response-hide" {
                Some("hide")
            } else {
                None
            },
        );
        attachment_pass_top(&mut r, &mut steps);
        assert!(!r.regions[0].cards.iter().any(|c| c.id == host));
        assert!(!r.players[2]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC003"));
    } else if kind == "source-return" {
        apply_game(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(hermit.clone()),
                target_id: Some(host.clone()),
                ability_id: Some("protect-local-character".into()),
                ..Action::new("activate")
            },
        );
        advance(&mut r, &mut steps, |r| r.priority_team == 1);
        cast(&mut r, &mut steps, 2, &reply, &hermit, None);
        attachment_pass_top(&mut r, &mut steps);
        assert!(!r.regions[0].cards.iter().any(|c| c.id == hermit));
        assert!(r
            .turn_attribute_modifiers
            .iter()
            .any(|m| m.target_instance == host));
    } else if kind == "fire-purple-loyalty" {
        assert_eq!(
            r.players[0].assets.iter().filter(|c| !c.exhausted).count(),
            2
        );
        assert!(r.players[0].assets.iter().all(|c| {
            catalog::card(&c.definition).magic_icon == rules::MagicIcon::Blood
                && catalog::card(&c.definition).color != "紫"
        }));
        let before = serde_json::to_string(&r).unwrap();
        reject_step(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(fire.clone()),
                target_id: Some(host.clone()),
                ..Action::new("play")
            },
        );
        assert_eq!(serde_json::to_string(&r).unwrap(), before);
        assert_eq!(
            steps.last().unwrap()["transition"]["errorCode"],
            "invalid_action"
        );
        assert_eq!(
            steps.last().unwrap()["transition"]["errorMessage"],
            "忠诚不足"
        );
        apply_game(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: purple_asset,
                ..Action::new("asset")
            },
        );
        assert_eq!(
            r.players[0].assets.iter().filter(|c| !c.exhausted).count(),
            3
        );
        cast(&mut r, &mut steps, 0, &fire, &host, None);
        assert_eq!(
            r.players[0].assets.iter().filter(|c| !c.exhausted).count(),
            1
        );
        assert_eq!(
            r.players[0].assets.iter().filter(|c| c.exhausted).count(),
            2
        );
        assert!(!r.regions[0].cards.iter().any(|c| c.id == host));
    } else {
        assert_eq!(kind, "phase-loyalty");
        reject_step(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(wall),
                region: Some(0),
                ..Action::new("play")
            },
        );
        reject_step(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(net),
                target_id: Some(host.clone()),
                ..Action::new("play")
            },
        );
        cast(&mut r, &mut steps, 0, &fire, &host, None);
        assert!(!r.regions[0].cards.iter().any(|c| c.id == host));
    }
    json!({"name":format!("protection-{kind}"),"seed":"9007199254740993","syntheticInitialLayout":true,
        "syntheticInitialFunding":true,"postInitialStateInjection":false,"steps":steps})
}

fn draft() -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.society_id = None;
    d.cards = [
        "JC001", "JC002", "JC003", "JC004", "JC005", "JC006", "JC007", "JC075", "JC070", "JC076",
        "JC074", "JC071", "JC073", "JC104", "JC102", "JC132",
    ]
    .into_iter()
    .map(|id| catalog::DeckEntry {
        card_id: id.into(),
        count: 3,
    })
    .collect();
    d.cards.push(catalog::DeckEntry {
        card_id: "JC008".into(),
        count: 2,
    });
    deck::validate(d).unwrap()
}
fn own<'a>(g: &'a Game, seat: usize, a: &Action) -> Option<&'a Card> {
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
        "888888888888888888880120".into(),
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
    let (mut used_fire, mut used_wall, mut seen_net, mut seen_hermit, mut used_hermit) =
        (false, false, false, false, false);
    let mut finished_turn = None;
    for _ in 0..2400 {
        let host = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "JC070" && c.controller == 0 && !c.face_down)
            .map(|c| c.id.clone());
        let hermit = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "JC071" && c.controller == 0 && !c.face_down)
            .map(|c| c.id.clone());
        seen_hermit |= hermit.is_some();
        seen_net |= g.attachments.iter().any(|a| a.card.definition == "JC073");
        used_hermit |= g
            .turn_attribute_modifiers
            .iter()
            .any(|m| m.defense_bonus == 1 && host.as_ref() == Some(&m.target_instance));
        if used_fire
            && used_wall
            && seen_net
            && seen_hermit
            && used_hermit
            && g.stack.is_empty()
            && g.pending.is_none()
        {
            finished_turn.get_or_insert(g.turn);
        }
        if finished_turn.is_some_and(|turn| g.turn > turn) {
            assert!(g.turn_attribute_modifiers.is_empty());
            return actions;
        }
        assert!(
            g.status == "playing" && g.turn <= 20,
            "natural protection stalled seed={seed} turn={} flags={:?}",
            g.turn,
            (seen_hermit, seen_net, used_hermit, used_fire, used_wall)
        );
        let (seat, a) = if let Some(p) = &g.pending {
            if p.choice.kind == "mulligan" {
                (
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(vec![]),
                        ..Action::new("choose")
                    },
                )
            } else if matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.event==Some(rules::Event::RegionConfrontationsEnded))
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
                let mut counts = BTreeMap::new();
                for c in &g.players[p.seat].hand {
                    *counts.entry(c.definition.clone()).or_insert(0usize) += 1;
                }
                let mut selected = vec![];
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
                            !matches!(
                                c.definition.as_str(),
                                "JC070" | "JC071" | "JC073" | "JC102" | "JC132"
                            ) || counts[&c.definition] > 1
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
            for s in 0..4 {
                let purple = g.players[s]
                    .assets
                    .iter()
                    .filter(|c| catalog::card(&c.definition).color == "紫")
                    .count();
                if g.players[s].assets.len() < 6 || (s == 0 && purple == 0) {
                    let white = g.players[s]
                        .assets
                        .iter()
                        .filter(|c| catalog::card(&c.definition).color == "白")
                        .count();
                    let yellow = g.players[s]
                        .assets
                        .iter()
                        .filter(|c| catalog::card(&c.definition).color == "黄")
                        .count();
                    let candidate = legal[s]
                        .iter()
                        .filter(|a| a.action.kind == "asset")
                        .filter_map(|a| own(&g, s, &a.action).map(|c| (a, c)))
                        .filter(|(_, c)| {
                            g.players[s].assets.len() < 6
                                || catalog::card(&c.definition).color == "紫"
                        })
                        .filter(|(_, c)| {
                            !matches!(
                                c.definition.as_str(),
                                "JC070" | "JC071" | "JC073" | "JC102" | "JC132"
                            ) || g.players[s]
                                .hand
                                .iter()
                                .filter(|h| h.definition == c.definition)
                                .count()
                                > 1
                        })
                        .min_by_key(|(_, c)| {
                            let color = &catalog::card(&c.definition).color;
                            if white < 2 && color == "白" {
                                0
                            } else if yellow < 1 && color == "黄" {
                                1
                            } else if purple < 1 && color == "紫" {
                                2
                            } else if color != "白" {
                                3
                            } else {
                                4
                            }
                        });
                    if let Some((a, _)) = candidate {
                        chosen = Some((s, a.action.clone()));
                        break;
                    }
                }
                if s == 0 {
                    let has_bonus = host.as_ref().is_some_and(|host| {
                        g.turn_attribute_modifiers
                            .iter()
                            .any(|m| m.target_instance == *host && m.defense_bonus > 0)
                    });
                    let fire_ready = legal[s].iter().any(|a| {
                        a.action.kind == "play"
                            && own(&g, s, &a.action).is_some_and(|c| c.definition == "JC102")
                            && a.action.target_id.as_ref() == host.as_ref()
                    });
                    if let Some(a) = legal[s].iter().find(|a| {
                        let def = own(&g, s, &a.action).map(|c| c.definition.as_str());
                        let want = match (a.action.kind.as_str(), def) {
                            ("deploy", Some("JC070")) => host.is_none(),
                            ("deploy", Some("JC071")) => {
                                hermit.is_none()
                                    && host.is_some()
                                    && seen_net
                                    && fire_ready
                                    && g.players[0].hand.iter().any(|c| c.definition == "JC102")
                                    && g.players[0].hand.iter().any(|c| c.definition == "JC132")
                            }
                            ("activate", Some("JC071")) => {
                                !used_fire
                                    && fire_ready
                                    && a.action.target_id.as_ref() == host.as_ref()
                            }
                            ("play", Some("JC073")) => {
                                !seen_net
                                    && host.is_some()
                                    && a.action.target_id.as_ref() == host.as_ref()
                            }
                            ("play", Some("JC102")) => {
                                !used_fire
                                    && has_bonus
                                    && a.action.target_id.as_ref() == host.as_ref()
                            }
                            ("play", Some("JC132")) => {
                                !used_wall
                                    && seen_net
                                    && used_fire
                                    && seen_hermit
                                    && a.action.region == Some(2)
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
        let def = own(&g, seat, &a).map(|c| c.definition.clone());
        g.apply(seat, a.clone()).unwrap();
        used_fire |= a.kind == "play" && def.as_deref() == Some("JC102");
        used_wall |= a.kind == "play" && def.as_deref() == Some("JC132");
        actions.push((seat, a));
    }
    panic!("bounded protection natural game turn={} window={:?} pending={:?} flags={:?} assets={} hand={:?} board={:?}", g.turn, g.window, g.pending.as_ref().map(|p| &p.choice.kind), (seen_hermit,seen_net,used_hermit,used_fire,used_wall), g.players[0].assets.len(), g.players[0].hand.iter().map(|c| &c.definition).collect::<Vec<_>>(), g.regions[2].cards.iter().map(|c| (&c.definition,c.face_down,c.exhausted,c.damage)).collect::<Vec<_>>());
}
fn natural_case() -> Value {
    let seed = 1u64;
    let actions = natural_actions(seed);
    let d = draft();
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "888888888888888888880120".into(),
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
    let mut seen = [false; 5];
    for (seat, a) in actions {
        let resolving = r
            .stack
            .last()
            .and_then(|s| s.frame.as_ref())
            .map(|f| f.source.card.definition.clone());
        let old_stack = r.stack.len();
        apply_game(&mut r, &mut steps, seat, a);
        let Some(region) = r.regions.get(2) else {
            continue;
        };
        let hermit = region
            .cards
            .iter()
            .find(|c| c.definition == "JC071" && !c.face_down);
        if hermit.is_some() && !seen[0] {
            seen[0] = true;
            milestones.push(json!({"step":steps.len()-1,"turn":r.turn,"kind":"JC071-paid-deployment","instance":hermit.unwrap().id}));
        }
        if let Some(net) = r.attachments.iter().find(|a| a.card.definition == "JC073") {
            if !seen[1] {
                seen[1] = true;
                milestones.push(json!({"step":steps.len()-1,"turn":r.turn,"kind":"JC073-paid-attachment","host":net.host_id}));
            }
        }
        if !seen[2]
            && r.turn_attribute_modifiers
                .iter()
                .any(|m| m.defense_bonus == 1)
        {
            seen[2] = true;
            milestones.push(json!({"step":steps.len()-1,"turn":r.turn,"kind":"JC071-paid-exhaustion-defense-grant"}));
        }
        if !seen[3] && old_stack > r.stack.len() && resolving.as_deref() == Some("JC102") {
            let host = r.regions[2]
                .cards
                .iter()
                .find(|c| c.definition == "JC070")
                .unwrap();
            assert_eq!(host.damage, 1);
            seen[3] = true;
            milestones.push(json!({"step":steps.len()-1,"turn":r.turn,"kind":"JC102-paid-one-damage-survivor","instance":host.id}));
        }
        if !seen[4] && old_stack > r.stack.len() && resolving.as_deref() == Some("JC132") {
            assert!(r.regions[2]
                .cards
                .iter()
                .filter(|c| !c.face_down)
                .all(|c| c.exhausted));
            seen[4] = true;
            milestones.push(
                json!({"step":steps.len()-1,"turn":r.turn,"kind":"JC132-paid-region-exhaustion"}),
            );
        }
    }
    assert_eq!(seen, [true; 5]);
    assert!(r.turn_attribute_modifiers.is_empty());
    json!({"name":"protection-natural-four-seat-paid-four-cards","seed":seed.to_string(),"syntheticInitialLayout":false,
        "syntheticInitialFunding":false,"postInitialStateInjection":false,"publicNaturalUiAcceptance":false,
        "legalFiftyCardDeck":true,"finalTurn":r.turn,"milestones":milestones,"steps":steps})
}
pub fn cases() -> impl Iterator<Item = Value> {
    [
        "paid-four",
        "foreign-host",
        "response-return",
        "response-hide",
        "source-return",
        "phase-loyalty",
        "fire-purple-loyalty",
    ]
    .into_iter()
    .map(boundary)
    .chain(std::iter::once_with(natural_case))
}
