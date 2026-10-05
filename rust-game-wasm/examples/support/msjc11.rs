//! Real finite MSJC11 programs. InitialFixture cases disclose every prepared
//! board/hand/paid-frame boundary. Natural cases use only Room commands/dealing.
use super::*;
use hegemony_server::model::Card;
const KILL: &str = "grant-kill";
const RETREAT: &str = "grant-region-retreat";
fn draft(red: bool) -> deck::DeckDraft {
    let mut d = deck::preset("responders").unwrap();
    d.id = "msjc11-mixed".into();
    d.name = "执行部真实双色50".into();
    d.society_id = Some("MSJC11".into());
    d.cards = [
        "JC014", "JC016", "JZ08", "BQ022", "JC020", "XQ07", "LC22", "LC23", "JC116", "JC129",
        "JC130", "JC131", "JC126",
    ]
    .into_iter()
    .chain(if red {
        vec!["JC042", "LC19", "XQ49"]
    } else {
        vec!["JC084", "JC086", "JC088"]
    })
    .map(|id| catalog::DeckEntry {
        card_id: id.into(),
        count: 3,
    })
    .collect();
    d.cards.push(catalog::DeckEntry {
        card_id: "JC125".into(),
        count: 2,
    });
    d
}
fn source(g: &Game, s: usize) -> String {
    g.players[s].society_zone.card.as_ref().unwrap().id.clone()
}
fn activate(
    g: &Game,
    s: usize,
    key: &str,
    target: Option<String>,
    region: Option<usize>,
) -> Action {
    Action {
        card_id: Some(source(g, s)),
        ability_id: Some(key.into()),
        target_id: target,
        region,
        ..Action::new("activate")
    }
}
fn board<'a>(g: &'a Game, id: &str) -> Option<&'a Card> {
    g.regions.iter().flat_map(|r| &r.cards).find(|c| c.id == id)
}
fn field(g: &mut Game, id: &str, owner: usize, controller: usize, region: usize) -> String {
    let mut c = g.make_card(id, owner);
    c.controller = controller;
    let id = c.id.clone();
    g.regions[region].cards.push(c);
    id
}
fn fund(g: &mut Game, s: usize, id: &str, n: usize) {
    for _ in 0..n {
        let c = g.make_card(id, s);
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
        .expect("bounded MSJC11 has a pass");
    apply_game(r, steps, s, a);
}
fn advance(r: &mut RoomEnvelope, steps: &mut Vec<Value>, done: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..600 {
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
    panic!("bounded MSJC11 progress");
}
fn deny(r: &mut RoomEnvelope, steps: &mut Vec<Value>, s: usize, a: Action) {
    let c = RoomCommand {
        command_id: format!("msjc11-reject:{}", steps.len()),
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
        "msjc11-boundary".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        draft(false),
        9007199254740993,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), draft(s % 2 == 1))
            .unwrap();
    }
    for s in 0..4 {
        g.apply(s, Action::new("ready")).unwrap();
    }
    g.apply(0, Action::new("start")).unwrap();
    while g.pending.is_some() {
        let (s, a) = pick_choice(&g);
        g.apply(s, a).unwrap();
    }
    for s in 0..4 {
        g.players[s].hand.clear();
        g.players[s].assets.clear();
        g.players[s].deck.clear();
        for _ in 0..20 {
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
fn granted(g: &Game, id: &str, kill: bool) -> bool {
    g.turn_attribute_modifiers.iter().any(|m| {
        m.target_instance == id
            && m.expires_turn == g.turn
            && if kill {
                m.kill_bonus > 0
            } else {
                m.grants_retreat
            }
    })
}
fn construction(red: bool) -> Value {
    let d = draft(red);
    let mut g = Game::new_with_deck(
        "msjc11-admission".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        d.clone(),
        1,
    )
    .unwrap();
    let mut r = RoomEnvelope::from_game(g.clone());
    let mut steps = vec![step(
        &r,
        "newGameWithDeck",
        json!([
            "msjc11-admission",
            "LOCAL",
            "teams",
            "P0",
            serde_json::to_string(&d).unwrap(),
            "1"
        ]),
        0,
    )];
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), d.clone()).unwrap();
        r = RoomEnvelope::from_game(g.clone());
        steps.push(step(
            &r,
            "joinGameWithDeck",
            json!([format!("P{s}"), serde_json::to_string(&d).unwrap()]),
            s,
        ));
    }
    let mut rejects = vec![];
    for bad in [
        "49",
        "same-name-4",
        "blue-vampire",
        "white-human-zero",
        "purple-temporary",
        "foreign-attachment",
        "same-name-cross-ID",
    ] {
        let mut b = d.clone();
        match bad {
            "49" => b.cards.last_mut().unwrap().count = 1,
            "same-name-4" => {
                b.cards[1].count = 4;
                b.cards.last_mut().unwrap().count = 1;
            }
            "same-name-cross-ID" => {
                b.society_id = None;
                b.cards = vec![
                    catalog::DeckEntry {
                        card_id: "JC125".into(),
                        count: 46,
                    },
                    catalog::DeckEntry {
                        card_id: "JC071".into(),
                        count: 2,
                    },
                    catalog::DeckEntry {
                        card_id: "LC12".into(),
                        count: 2,
                    },
                ];
            }
            _ => {
                let id = match bad {
                    "blue-vampire" => "JZ27",
                    "white-human-zero" => "LC12",
                    "purple-temporary" => "JC104",
                    _ => "JC093",
                };
                b.cards = vec![
                    catalog::DeckEntry {
                        card_id: id.into(),
                        count: 3,
                    },
                    catalog::DeckEntry {
                        card_id: "JC125".into(),
                        count: 47,
                    },
                ];
            }
        }
        let raw = serde_json::to_string(&b).unwrap();
        let error = Game::new_with_deck(
            "bad".into(),
            "LOCAL".into(),
            "teams".into(),
            "P0".into(),
            b,
            1,
        )
        .unwrap_err();
        rejects.push(json!({"name":bad,"args":["bad","LOCAL","teams","P0",raw,"1"],"error":error}));
    }
    json!({"name":if red{"msjc11-real-green-red-50-admission"}else{"msjc11-real-green-black-50-admission"},"seed":"1","steps":steps,"syntheticInitialLayout":false,"rejectedNewGameWithDeck":rejects})
}
fn boundary(kind: &str) -> Value {
    let mut g = initial();
    let actor = kind
        .strip_prefix("kill-seat-")
        .map(|s| s.parse::<usize>().unwrap())
        .unwrap_or(0);
    let team = g.team(actor);
    g.window = Some(Window::Action(team));
    g.active_team = team;
    g.priority_team = team;
    fund(
        &mut g,
        actor,
        "JC125",
        if kind == "reject-fee" { 2 } else { 6 },
    );
    fund(&mut g, actor ^ 1, "JC125", 6);
    let region = if kind.starts_with("kill-seat-") { 4 } else { 1 };
    let target = field(
        &mut g,
        if kind == "reject-barrier" {
            "JZ08"
        } else if ["ordinary", "equipment", "lost-icons", "late-icons"].contains(&kind) {
            "LC20"
        } else if kind == "temporary-first" || kind == "temporary-rear" {
            "JC004"
        } else if kind == "converted-spirit-no-combat" {
            "JZ58"
        } else if kind == "hide-reentry" {
            "JC088"
        } else if kind == "movement" {
            "JC014"
        } else if kind == "zero-combat" {
            "JC125"
        } else {
            "LC22"
        },
        if kind.starts_with("kill-seat-") {
            (actor + 2) % 4
        } else {
            0
        },
        if kind.starts_with("kill-seat-") {
            (actor + 2) % 4
        } else {
            0
        },
        region,
    );
    let teammate = field(&mut g, "LC22", actor ^ 1, actor ^ 1, region);
    let borrowed = field(&mut g, "LC22", 2, actor, region);
    let enemy = field(&mut g, "LC22", 0, 2, region);
    if kind == "reject-barrier" {
        g.regions[region]
            .cards
            .iter_mut()
            .find(|c| c.id == target)
            .unwrap()
            .controller = 2;
    }
    if kind == "reject-hidden" {
        g.regions[region]
            .cards
            .iter_mut()
            .find(|c| c.id == target)
            .unwrap()
            .face_down = true;
    }
    if kind == "reject-exhausted" {
        g.players[actor]
            .society_zone
            .card
            .as_mut()
            .unwrap()
            .exhausted = true;
    }
    if kind == "equipment" || kind == "lost-icons" {
        let a = g.make_card("JC116", actor);
        g.attachments.push(Attachment {
            card: a,
            host_id: target.clone(),
        });
    }
    let empower = if kind == "ordinary" || kind == "late-icons" {
        // JC008 is in a disclosed prepared hand, outside the MSJC11 deck;
        // declaration/resolution after the checkpoint use real Room commands.
        fund(&mut g, actor, "JC002", 2);
        let c = g.make_card("JC008", actor);
        let id = c.id.clone();
        g.players[actor].hand.push(c);
        Some(id)
    } else {
        None
    };
    if kind == "converted-spirit-no-combat" {
        let a = g.make_card("XQ43", 2);
        g.attachments.push(Attachment {
            card: a,
            host_id: g.regions[region].card.id.clone(),
        });
    }
    if kind == "temporary-rear" {
        g.first_team = 1;
    } else {
        g.first_team = 0;
    }
    let newcomer = if kind == "retreat-once" {
        let c = g.make_card("LC22", 0);
        let id = c.id.clone();
        g.players[0].hand.push(c);
        Some(id)
    } else {
        None
    };
    let disintegrate = if kind == "lost-icons" {
        fund(&mut g, 0, "JC002", 3);
        let c = g.make_card("JC005", 0);
        let id = c.id.clone();
        g.players[0].hand.push(c);
        Some(id)
    } else {
        None
    };
    if kind == "hide-reentry" {
        fund(&mut g, 0, "JC084", 5);
    }
    if ["combat-kill", "zero-combat", "exhausted-kill"].contains(&kind) {
        g.regions[region]
            .cards
            .iter_mut()
            .find(|c| c.id == teammate)
            .unwrap()
            .exhausted = true;
        if kind != "combat-kill" {
            field(&mut g, "LC22", 0, 0, region);
        }
        if kind == "exhausted-kill" {
            g.regions[region]
                .cards
                .iter_mut()
                .find(|c| c.id == target)
                .unwrap()
                .exhausted = true;
        }
    }
    let kill = kind.starts_with("kill-seat-")
        || kind.starts_with("reject-")
        || [
            "cancel-target",
            "stack-kill",
            "zero-combat",
            "combat-kill",
            "exhausted-kill",
            "expiry",
            "hide-reentry",
        ]
        .contains(&kind);
    if kind == "stack-kill" {
        g.regions[region]
            .cards
            .iter_mut()
            .find(|c| c.id == target)
            .unwrap()
            .definition = "JC016".into();
    }
    let mut prepared_room = None;
    if kind == "region-replaced" || kind == "region-binding-missing" || kind == "cancel-target" {
        let mut room = RoomEnvelope::from_game(g.clone());
        let action = activate(
            &g,
            actor,
            if kind == "cancel-target" {
                KILL
            } else {
                RETREAT
            },
            if kind == "cancel-target" {
                Some(target.clone())
            } else {
                None
            },
            if kind == "cancel-target" {
                None
            } else {
                Some(region)
            },
        );
        apply_game(&mut room, &mut vec![], actor, action);
        g = room.game.clone();
        assert_eq!(
            g.stack.last().unwrap().frame.as_ref().unwrap().targets[0].region_instance,
            if kind == "cancel-target" {
                None
            } else {
                Some(g.regions[region].card.id.clone())
            }
        );
        if kind == "region-replaced" {
            g.regions[region].card = g.make_card("DQJC115", 0);
        } else if kind == "region-binding-missing" {
            g.stack.last_mut().unwrap().frame.as_mut().unwrap().targets[0].region_instance = None;
        } else {
            g.regions[region].cards.retain(|c| c.id != target);
        }
        room.game = g.clone();
        prepared_room = Some(room);
    }
    if kind == "win-owner-attachment" {
        g.regions[region].influence[0] = catalog::card(&g.regions[region].card.definition)
            .threshold
            .unwrap()
            - 1;
        field(&mut g, "LC23", 0, 0, region);
        let a = g.make_card("BQ022", 3);
        g.attachments.push(Attachment {
            card: a,
            host_id: borrowed.clone(),
        });
    }
    let (mut r, mut steps) =
        checkpoint(prepared_room.unwrap_or_else(|| RoomEnvelope::from_game(g)));
    if kind.starts_with("reject-") {
        let mut a = activate(&r, actor, KILL, Some(target.clone()), None);
        if kind == "reject-other-source" {
            a.card_id = Some(source(&r, actor ^ 1));
        }
        deny(&mut r, &mut steps, actor, a);
    } else if ["region-replaced", "region-binding-missing", "cancel-target"].contains(&kind) {
        advance(&mut r, &mut steps, |r| r.stack.is_empty());
        assert!(!granted(&r, &target, true) && !granted(&r, &target, false));
    } else {
        if kind == "ordinary" {
            apply_game(
                &mut r,
                &mut steps,
                actor,
                Action {
                    card_id: empower.clone(),
                    target_id: Some(target.clone()),
                    ..Action::new("play")
                },
            );
            advance(&mut r, &mut steps, |r| r.stack.is_empty());
            assert!(r
                .turn_attribute_modifiers
                .iter()
                .any(|m| m.target_instance == target && m.ordinary_icons.combat == 1));
        }
        let a = activate(
            &r,
            actor,
            if kill { KILL } else { RETREAT },
            if kill { Some(target.clone()) } else { None },
            if kill { None } else { Some(region) },
        );
        apply_game(&mut r, &mut steps, actor, a);
        advance(&mut r, &mut steps, |r| r.stack.is_empty());
        if kill {
            assert!(granted(&r, &target, true));
        } else {
            let expected = ![
                "temporary-first",
                "temporary-rear",
                "converted-spirit-no-combat",
                "late-icons",
            ]
            .contains(&kind);
            assert_eq!(granted(&r, &target, false), expected);
            assert!(granted(&r, &borrowed, false));
            assert!(!granted(&r, &teammate, false));
            if actor == 0 {
                assert!(!granted(&r, &enemy, false));
            }
        }
        if kind == "stack-kill" {
            let a = activate(&r, 1, KILL, Some(target.clone()), None);
            apply_game(&mut r, &mut steps, 1, a);
            advance(&mut r, &mut steps, |r| r.stack.is_empty());
            let view = r.view(0, r.pacing.last_server_now_ms);
            assert_eq!(
                view.game.regions[region]
                    .characters
                    .iter()
                    .find(|c| c.instance_id == target)
                    .unwrap()
                    .current_kill,
                Some(3)
            );
        }
        if kind == "retreat-once" {
            // Initial prepared hand; the newcomer uses an ordinary paid deploy.
            let id = newcomer.unwrap();
            apply_game(
                &mut r,
                &mut steps,
                0,
                Action {
                    card_id: Some(id),
                    region: Some(region),
                    ..Action::new("deploy")
                },
            );
            advance(&mut r, &mut steps, |r| r.stack.is_empty());
            let later = r.regions[region]
                .cards
                .iter()
                .find(|c| {
                    c.controller == 0
                        && c.definition == "LC22"
                        && c.id != target
                        && c.id != borrowed
                })
                .unwrap();
            assert!(!granted(&r, &later.id, false));
        }
        if kind == "win-owner-attachment" {
            // The disclosed initial influence is threshold minus one. Real
            // confrontation/LC23 influence reaches Win without a state edit.
            let old = r.regions[region].card.id.clone();
            advance(&mut r, &mut steps, |r| r.regions[region].card.id != old);
            assert!(r.players[2]
                .hand
                .iter()
                .any(|c| c.definition == "LC22" && c.id != borrowed));
            assert!(r.players[1].deck.iter().any(|c| c.definition == "LC22"));
            assert!(r.players[3].deck.iter().any(|c| c.definition == "BQ022"));
            assert!(!r.turn_attribute_modifiers.iter().any(|m| m.grants_retreat));
        }
        if ["combat-kill", "zero-combat", "exhausted-kill"].contains(&kind) {
            advance(&mut r, &mut steps, |r| {
                r.pending.as_ref().is_some_and(|p| matches!(p.resolution, ChoiceResolution::Damage { region: rr } if rr == region))
            });
            assert_eq!(
                r.pending.as_ref().unwrap().choice.amount,
                Some(if kind == "combat-kill" { 2 } else { 1 })
            );
            let (s, a) = pick_choice(&r.game);
            apply_game(&mut r, &mut steps, s, a);
        }
        if kind == "late-icons" {
            apply_game(
                &mut r,
                &mut steps,
                actor,
                Action {
                    card_id: empower.clone(),
                    target_id: Some(target.clone()),
                    ..Action::new("play")
                },
            );
            advance(&mut r, &mut steps, |r| r.stack.is_empty());
            assert!(r
                .turn_attribute_modifiers
                .iter()
                .any(|m| m.target_instance == target && m.ordinary_icons.combat == 1));
            assert!(!granted(&r, &target, false));
        }
        if kind == "lost-icons" {
            let attachment = r
                .attachments
                .iter()
                .find(|a| a.host_id == target)
                .unwrap()
                .card
                .id
                .clone();
            apply_game(
                &mut r,
                &mut steps,
                0,
                Action {
                    card_id: disintegrate,
                    target_id: Some(attachment.clone()),
                    ..Action::new("play")
                },
            );
            advance(&mut r, &mut steps, |r| r.stack.is_empty());
            assert!(!r.attachments.iter().any(|a| a.card.id == attachment));
            assert!(granted(&r, &target, false));
        }
        if kind == "movement" {
            advance(&mut r, &mut steps, |r| {
                r.pending
                    .as_ref()
                    .is_some_and(|p| p.choice.options.iter().any(|o| o.id == "region:2"))
            });
            let p = r.pending.clone().unwrap();
            apply_game(
                &mut r,
                &mut steps,
                p.seat,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec!["region:2".into()]),
                    ..Action::new("choose")
                },
            );
            advance(&mut r, &mut steps, |r| {
                r.stack.is_empty() && r.pending.is_none()
            });
            assert!(r.regions[2].cards.iter().any(|c| c.id == target));
            assert!(granted(&r, &target, false));
        }
        if kind == "expiry" {
            let t = r.turn;
            advance(&mut r, &mut steps, |r| r.turn > t);
            assert!(!r
                .turn_attribute_modifiers
                .iter()
                .any(|m| m.kill_bonus > 0 || m.grants_retreat));
            assert!(!r.players[0].society_zone.card.as_ref().unwrap().exhausted);
        }
        if kind == "hide-reentry" {
            let key = "hide-self".to_string();
            apply_game(
                &mut r,
                &mut steps,
                0,
                Action {
                    card_id: Some(target.clone()),
                    ability_id: Some(key),
                    ..Action::new("activate")
                },
            );
            advance(&mut r, &mut steps, |r| r.stack.is_empty());
            let hidden = r.regions[region]
                .cards
                .iter()
                .find(|c| c.definition == "JC088")
                .unwrap();
            assert!(hidden.face_down);
            assert!(!granted(&r, &hidden.id, true));
            let hidden_id = hidden.id.clone();
            assert_ne!(hidden_id, target);
            let reveal = r
                .view(0, r.pacing.last_server_now_ms)
                .game
                .legal_actions
                .into_iter()
                .find(|a| {
                    a.action.kind == "reveal" && a.action.card_id.as_ref() == Some(&hidden_id)
                })
                .unwrap()
                .action;
            apply_game(&mut r, &mut steps, 0, reveal);
            advance(&mut r, &mut steps, |r| {
                r.stack.is_empty() && r.pending.is_none()
            });
            let returned = r.regions[region]
                .cards
                .iter()
                .find(|c| c.definition == "JC088")
                .unwrap();
            assert!(!returned.face_down);
            assert_ne!(returned.id, hidden_id);
            assert!(!granted(&r, &returned.id, true));
        }
    }
    json!({"name":format!("msjc11-boundary-{kind}"),"seed":"9007199254740993","steps":steps,"syntheticInitialLayout":true,"syntheticBoundary":"Prepared board/hand/paid frame; prepaid cancelled frames only in the initial checkpoint. All later game progress uses Room commands. No browser or production state injection."})
}
fn natural(red: bool) -> Value {
    let seed = if red {
        "9007199254745011"
    } else {
        "9007199254745010"
    };
    let d = draft(red);
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "msjc11-natural".into(),
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
            "msjc11-natural",
            "LOCAL",
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
    let (mut killed, mut retreated) = ([false; 4], [false; 4]);
    let mut same_region = false;
    let mut different_region = false;
    let mut last = 0;
    for _ in 0..2400 {
        let all = r
            .regions
            .iter()
            .enumerate()
            .flat_map(|(region, r)| {
                r.cards
                    .iter()
                    .filter(|c| !c.face_down)
                    .map(move |c| (region, c))
            })
            .collect::<Vec<_>>();
        for (i, (a, x)) in all.iter().enumerate() {
            for (b, y) in all.iter().skip(i + 1) {
                if x.controller != y.controller
                    && x.definition == y.definition
                    && catalog::card(&x.definition).color == "绿"
                {
                    same_region |= a == b;
                    different_region |= a != b;
                }
            }
        }
        if killed.iter().all(|x| *x)
            && retreated.iter().all(|x| *x)
            && r.stack.is_empty()
            && r.pending.is_none()
            && r.turn > last
        {
            break;
        }
        assert_eq!(r.status, "playing", "natural ended before all four grants");
        if let Some(p) = r.pending.clone() {
            if p.choice.kind == "search" {
                if let Some(o) = p.choice.options.iter().find(|o| {
                    r.players[p.seat]
                        .deck
                        .iter()
                        .find(|c| c.id == o.id)
                        .is_some_and(|c| c.definition == "JC016")
                }) {
                    apply_game(
                        &mut r,
                        &mut steps,
                        p.seat,
                        Action {
                            choice_id: Some(p.choice.id),
                            selected: Some(vec![o.id.clone()]),
                            ..Action::new("choose")
                        },
                    );
                    continue;
                }
            }
            let (s, a) = pick_choice(&r.game);
            apply_game(&mut r, &mut steps, s, a);
            continue;
        }
        let mut chosen = None;
        for s in 0..4 {
            let legal = r.view(s, r.pacing.last_server_now_ms).game.legal_actions;
            let region = if s % 2 == 0 { 1 } else { 3 };
            let has_permanent = r.regions[region]
                .cards
                .iter()
                .any(|c| c.controller == s && !c.face_down && c.definition == "JC016");
            let key = if !killed[s] { KILL } else { RETREAT };
            if !retreated[s] && has_permanent {
                if let Some(a) = legal.iter().find(|a| {
                    a.action.ability_id.as_deref() == Some(key)
                        && if key == RETREAT {
                            a.action.region == Some(region)
                        } else {
                            a.action
                                .target_id
                                .as_ref()
                                .and_then(|id| board(&r, id))
                                .is_some_and(|c| c.controller == s)
                        }
                }) {
                    let before = r
                        .players
                        .iter()
                        .map(|p| serde_json::to_value(&p.assets).unwrap())
                        .collect::<Vec<_>>();
                    let ready = r.players[s].assets.iter().filter(|c| !c.exhausted).count();
                    apply_game(&mut r, &mut steps, s, a.action.clone());
                    assert_eq!(
                        r.players[s].assets.iter().filter(|c| !c.exhausted).count(),
                        ready - 3
                    );
                    for other in 0..4 {
                        if other != s {
                            assert_eq!(
                                serde_json::to_value(&r.players[other].assets).unwrap(),
                                before[other]
                            );
                        }
                    }
                    if key == KILL {
                        killed[s] = true;
                    } else {
                        retreated[s] = true;
                    }
                    last = r.turn;
                    chosen = Some(());
                    break;
                }
            }
            if !retreated[s] {
                let green = r.players[s]
                    .assets
                    .iter()
                    .filter(|c| catalog::card(&c.definition).color == "绿")
                    .count();
                let asset = legal
                    .iter()
                    .filter(|a| {
                        a.action.kind == "asset"
                            && a.action
                                .card_id
                                .as_ref()
                                .and_then(|id| r.players[s].hand.iter().find(|c| &c.id == id))
                                .is_some_and(|c| c.definition != "JC016")
                    })
                    .min_by_key(|a| {
                        r.players[s]
                            .hand
                            .iter()
                            .find(|c| Some(&c.id) == a.action.card_id.as_ref())
                            .map(|c| {
                                (
                                    usize::from(
                                        green < 2 && catalog::card(&c.definition).color != "绿",
                                    ),
                                    usize::from(c.definition == "JC016"),
                                )
                            })
                            .unwrap_or((2, 2))
                    });
                if let Some(a) = asset {
                    apply_game(&mut r, &mut steps, s, a.action.clone());
                    chosen = Some(());
                    break;
                }
                if !has_permanent {
                    if let Some(a) = legal
                        .iter()
                        .filter(|a| a.action.kind == "deploy" && a.action.region == Some(region))
                        .filter(|a| {
                            a.action
                                .card_id
                                .as_ref()
                                .and_then(|id| r.players[s].hand.iter().find(|c| &c.id == id))
                                .is_some_and(|c| c.definition == "JC016")
                        })
                        .min_by_key(|a| {
                            a.action
                                .card_id
                                .as_ref()
                                .and_then(|id| r.players[s].hand.iter().find(|c| &c.id == id))
                                .map(|c| usize::from(c.definition != "JC016"))
                                .unwrap_or(2)
                        })
                    {
                        apply_game(&mut r, &mut steps, s, a.action.clone());
                        chosen = Some(());
                        break;
                    }
                    if let Some(a) = legal.iter().find(|a| {
                        a.action.kind == "play"
                            && a.action
                                .card_id
                                .as_ref()
                                .and_then(|id| r.players[s].hand.iter().find(|c| &c.id == id))
                                .is_some_and(|c| c.definition == "JC131")
                    }) {
                        apply_game(&mut r, &mut steps, s, a.action.clone());
                        chosen = Some(());
                        break;
                    }
                }
            }
        }
        if chosen.is_none() {
            pass(&mut r, &mut steps);
        }
    }
    assert!(
        killed.iter().all(|x| *x) && retreated.iter().all(|x| *x),
        "natural incomplete {:?} {:?}",
        killed,
        retreated
    );
    assert!(
        same_region && different_region,
        "same-color same-name independent controllers must occur in both region relationships: same={same_region}, different={different_region}"
    );
    assert!(!r
        .turn_attribute_modifiers
        .iter()
        .any(|m| m.kill_bonus > 0 || m.grants_retreat));
    json!({"name":if red{"msjc11-natural-four-seat-green-red"}else{"msjc11-natural-four-seat-green-black"},"seed":seed,"steps":steps,"syntheticInitialLayout":false,"allFourPaidBothGrants":true,"sameGreenNameDifferentControllerSameAndDifferentRegions":true,"ordinaryCardsPerSeat":50,"publicBrowserUiAcceptance":false})
}
pub(super) fn natural_cases() -> impl Iterator<Item = Value> {
    [false, true].into_iter().map(natural)
}
pub(super) fn cases() -> impl Iterator<Item = Value> {
    [false, true]
        .into_iter()
        .map(construction)
        .chain(natural_cases())
        .chain(boundary_cases())
}
pub(super) fn boundary_cases() -> impl Iterator<Item = Value> {
    [
        "kill-seat-0",
        "kill-seat-1",
        "kill-seat-2",
        "kill-seat-3",
        "reject-fee",
        "reject-exhausted",
        "reject-other-source",
        "reject-barrier",
        "reject-hidden",
        "cancel-target",
        "stack-kill",
        "zero-combat",
        "combat-kill",
        "exhausted-kill",
        "retreat-once",
        "equipment",
        "lost-icons",
        "movement",
        "ordinary",
        "late-icons",
        "temporary-first",
        "temporary-rear",
        "converted-spirit-no-combat",
        "region-replaced",
        "region-binding-missing",
        "win-owner-attachment",
        "expiry",
        "hide-reentry",
    ]
    .into_iter()
    .map(boundary)
}
