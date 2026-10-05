//! Actual LC06 event and real JZ59/JC091/JC075/JC078/JC102 commands.
//! Every boundary layout is disclosed; the final case starts from legal decks.
use super::*;
fn fund(g: &mut Game, s: usize, definition: &str, n: usize) {
    for _ in 0..n {
        let c = g.make_card(definition, s);
        g.players[s].assets.push(c);
    }
}
fn field(g: &mut Game, definition: &str, owner: usize, controller: usize) -> String {
    let mut c = g.make_card(definition, owner);
    c.controller = controller;
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn held(g: &mut Game, definition: &str, s: usize) -> String {
    let c = g.make_card(definition, s);
    let id = c.id.clone();
    g.players[s].hand.push(c);
    id
}
fn board<'a>(g: &'a Game, id: &str) -> Option<&'a hegemony_server::model::Card> {
    g.regions.iter().flat_map(|r| &r.cards).find(|c| c.id == id)
}
fn initial(name: &str) -> Game {
    let mut g = attachment_initial("teams", &format!("lc06-{name}"));
    for s in 0..4 {
        g.players[s].deck.clear();
        g.players[s].assets.clear();
        for _ in 0..12 {
            let c = g.make_card("JC125", s);
            g.players[s].deck.push(c);
        }
    }
    fund(&mut g, 0, "JC091", 6);
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
    for _ in 0..150 {
        if done(r) {
            return;
        }
        assert!(r.pending.is_none(), "unexpected choice");
        pass(r, steps);
    }
    panic!("bounded LC06 progress");
}
fn trigger(r: &RoomEnvelope, definition: &str) -> bool {
    r.pending.as_ref().is_some_and(|p| matches!(&p.resolution, ChoiceResolution::Declare { declaration, .. } if declaration.source.card.definition == definition))
}
fn choose(r: &mut RoomEnvelope, steps: &mut Vec<Value>, ids: Vec<String>) {
    let p = r.pending.clone().unwrap();
    apply_game(
        r,
        steps,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(ids),
            ..Action::new("choose")
        },
    );
}
fn drain(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    advance(r, steps, |r| r.stack.is_empty() && r.pending.is_none());
}
fn play(id: &str, target: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        target_id: Some(target.into()),
        ..Action::new("play")
    }
}
fn activate(id: &str, key: &str, target: Option<&str>) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(key.into()),
        target_id: target.map(str::to_string),
        ..Action::new("activate")
    }
}
fn reject(r: &mut RoomEnvelope, steps: &mut Vec<Value>, s: usize, a: Action) {
    let c = RoomCommand {
        command_id: format!("lc06-reject:{}", steps.len()),
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
    let mut g = initial(kind);
    let controller = if matches!(kind, "controller" | "fatal-controller" | "shield") {
        2
    } else {
        0
    };
    let owner = if controller == 2 { 3 } else { 0 };
    let lc = field(&mut g, "LC06", owner, controller);
    if matches!(kind, "fatal-controller" | "healing") {
        g.regions[0].cards.last_mut().unwrap().wounds = 1;
    }
    if kind == "shield" {
        g.regions[0].cards.last_mut().unwrap().shield = 1;
    }
    if kind == "exhausted-two-wounds" {
        g.regions[0].cards.last_mut().unwrap().exhausted = true;
    }
    if kind == "empty-deck" {
        g.players[controller].deck.truncate(1);
    }
    let count = if kind == "exhausted-two-wounds" { 2 } else { 1 };
    let mut sources = vec![];
    let mut murders = vec![];
    for _ in 0..count {
        sources.push(field(&mut g, "JZ59", 0, 0));
        murders.push(held(&mut g, "JC091", 0));
    }
    let rescue = if matches!(kind, "source-return" | "target-return") {
        fund(&mut g, 0, "JC075", 2);
        Some(field(&mut g, "JC075", 0, 0))
    } else {
        None
    };
    let healer = if kind == "healing" {
        Some(field(&mut g, "LC19", 0, 0))
    } else {
        None
    };
    let extra = if matches!(kind, "damage" | "destroy" | "damage-prevention") {
        let definition = if kind == "damage" {
            "JC102"
        } else if kind == "destroy" {
            "JC091"
        } else {
            "JC078"
        };
        if kind != "destroy" {
            fund(&mut g, 0, definition, 3);
        }
        Some(held(&mut g, definition, 0))
    } else {
        None
    };
    let (mut r, mut steps) = checkpoint(RoomEnvelope::from_game(g));
    if kind == "healing" {
        apply_game(
            &mut r,
            &mut steps,
            0,
            activate(healer.as_ref().unwrap(), "heal", Some(&lc)),
        );
        drain(&mut r, &mut steps);
        assert_eq!(board(&r, &lc).unwrap().wounds, 0);
        assert!(r.players[0].hand.iter().all(|c| c.definition != "JC125"));
    } else if kind == "manual" {
        reject(
            &mut r,
            &mut steps,
            controller,
            activate(&lc, "wound-draw-two", None),
        );
    } else if matches!(kind, "damage" | "destroy") {
        apply_game(&mut r, &mut steps, 0, play(extra.as_ref().unwrap(), &lc));
        drain(&mut r, &mut steps);
        assert!(r.players[controller]
            .hand
            .iter()
            .all(|c| c.definition != "JC125"));
        if kind == "damage" {
            let c = board(&r, &lc).unwrap();
            assert_eq!((c.wounds, c.damage), (0, 1));
        } else {
            assert!(board(&r, &lc).is_none());
        }
    } else {
        if kind == "damage-prevention" {
            apply_game(&mut r, &mut steps, 0, play(extra.as_ref().unwrap(), &lc));
            drain(&mut r, &mut steps);
        }
        for i in 0..count {
            apply_game(&mut r, &mut steps, 0, play(&murders[i], &sources[i]));
            advance(&mut r, &mut steps, |r| trigger(r, "JZ59"));
            choose(&mut r, &mut steps, vec![lc.clone()]);
            if kind == "target-return" {
                apply_game(
                    &mut r,
                    &mut steps,
                    0,
                    activate(rescue.as_ref().unwrap(), "rescue", Some(&lc)),
                );
            }
            if matches!(kind, "target-return" | "shield") {
                drain(&mut r, &mut steps);
                assert!(r.players[controller]
                    .hand
                    .iter()
                    .all(|c| c.definition != "JC125"));
                if kind == "shield" {
                    let c = board(&r, &lc).unwrap();
                    assert_eq!((c.wounds, c.shield), (0, 0));
                }
            } else {
                advance(&mut r, &mut steps, |r| trigger(r, "LC06"));
                assert_eq!(r.pending.as_ref().unwrap().seat, controller);
                for viewer in 0..4 {
                    assert_eq!(
                        r.view(viewer, r.pacing.last_server_now_ms)
                            .pending_choice
                            .is_some(),
                        viewer == controller
                    );
                }
                let declaration = match &r.pending.as_ref().unwrap().resolution {
                    ChoiceResolution::Declare { declaration, .. } => declaration,
                    _ => unreachable!(),
                };
                assert_eq!(declaration.source.card.id, lc);
                assert_eq!(
                    (
                        declaration.source.card.owner,
                        declaration.source.card.controller
                    ),
                    (owner, controller)
                );
                assert_eq!(
                    declaration.source.card.wounds,
                    if kind == "fatal-controller" {
                        2
                    } else {
                        (i + 1) as u32
                    }
                );
                if kind == "fatal-controller" {
                    assert!(board(&r, &lc).is_none());
                    let c = r.players[owner]
                        .graveyard
                        .iter()
                        .find(|c| c.definition == "LC06")
                        .unwrap();
                    assert_ne!(c.id, lc);
                    assert_eq!((c.controller, c.wounds, c.damage), (owner, 0, 0));
                }
                let before = r.players[controller].hand.len();
                choose(
                    &mut r,
                    &mut steps,
                    if kind == "decline" {
                        vec![]
                    } else {
                        vec!["accept".into()]
                    },
                );
                assert_eq!(r.players[controller].hand.len(), before);
                if kind == "source-return" {
                    apply_game(
                        &mut r,
                        &mut steps,
                        0,
                        activate(rescue.as_ref().unwrap(), "rescue", Some(&lc)),
                    );
                }
                drain(&mut r, &mut steps);
                if kind == "empty-deck" {
                    assert!(
                        r.players[controller].eliminated && r.players[controller].hand.is_empty()
                    );
                } else {
                    assert_eq!(
                        r.players[controller]
                            .hand
                            .iter()
                            .filter(|c| c.definition == "JC125")
                            .count(),
                        if kind == "decline" { 0 } else { (i + 1) * 2 }
                    );
                }
                if kind == "source-return" {
                    assert!(board(&r, &lc).is_none());
                    let c = r.players[owner]
                        .hand
                        .iter()
                        .find(|c| c.definition == "LC06")
                        .unwrap();
                    assert_ne!(c.id, lc);
                    assert_eq!(c.wounds, 0);
                }
            }
        }
    }
    json!({"name":format!("LC06-{kind}"),"seed":"9007199254740993","steps":steps,"syntheticInitialLayout":true,"syntheticInitialWound":matches!(kind,"fatal-controller"|"healing"),"syntheticInitialShield":kind=="shield","newNaturalUiCoverage":false,"realPrintedJZ59Wound":!matches!(kind,"damage"|"destroy"|"manual"|"healing"),"newCardProgramInjected":false})
}
fn unique(kind: &str) -> Value {
    let mut g = initial(kind);
    g.players[0].assets.clear();
    fund(&mut g, 0, "JC075", 3);
    let old = field(&mut g, "LC06", 3, if kind == "friend" { 1 } else { 0 });
    let id = if kind == "paid-reveal" {
        let id = field(&mut g, "LC06", 0, 0);
        g.regions[0].cards.last_mut().unwrap().face_down = true;
        id
    } else {
        held(&mut g, "LC06", 0)
    };
    let (mut r, mut steps) = checkpoint(RoomEnvelope::from_game(g));
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(id.clone()),
            region: if kind == "paid-reveal" { None } else { Some(0) },
            ..Action::new(if kind == "paid-reveal" {
                "reveal"
            } else {
                "deploy"
            })
        },
    );
    if kind == "friend" {
        drain(&mut r, &mut steps);
        assert!(board(&r, &old).is_some());
    } else {
        advance(&mut r, &mut steps, |r| r.pending.is_some());
        let p = r.pending.clone().unwrap();
        assert_eq!(p.seat, 0);
        assert_eq!(p.choice.options.len(), 2);
        assert!(p.choice.title.contains("同名独有"));
        choose(&mut r, &mut steps, vec![old.clone()]);
        assert!(board(&r, &old).is_none());
        assert!(r.players[3]
            .graveyard
            .iter()
            .any(|c| c.definition == "LC06" && c.id != old));
    }
    assert_eq!(
        r.players[0].assets.iter().filter(|c| c.exhausted).count(),
        3
    );
    let new = r
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .find(|c| c.definition == "LC06" && c.owner == 0)
        .unwrap();
    assert_ne!(new.id, id);
    json!({"name":format!("LC06-gold-unique-{kind}"),"seed":"9007199254740993","steps":steps,"syntheticInitialLayout":true,"normalPaidNewInstance":true,"newNaturalUiCoverage":false})
}
fn natural_deck() -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.id = "lc06-natural".into();
    d.name = "圣贤真实创伤链".into();
    d.society_id = Some("MSJC06".into());
    d.cards.clear();
    for id in [
        "JC075", "JC070", "JC076", "JC074", "JC073", "JC078", "XQ34", "LC12", "LC06", "JZ59",
        "JC091",
    ] {
        d.cards.push(catalog::DeckEntry {
            card_id: id.into(),
            count: 3,
        });
    }
    d.cards.push(catalog::DeckEntry {
        card_id: "JC125".into(),
        count: 17,
    });
    d
}
pub(super) fn find_natural_seed() {
    let d = natural_deck();
    for seed in 9007199254740993..9007199254840993 {
        let mut g = Game::new_with_deck(
            "lc06-seed".into(),
            "LOCAL".into(),
            "teams".into(),
            "P0".into(),
            d.clone(),
            seed,
        )
        .unwrap();
        for s in 1..4 {
            g.join_with_deck(format!("P{s}"), d.clone()).unwrap();
        }
        for s in 0..4 {
            g.apply(s, Action::new("ready")).unwrap();
        }
        g.apply(0, Action::new("start")).unwrap();
        let p = &g.players[0];
        if p.hand.iter().filter(|c| c.definition == "JC091").count() >= 2
            && p.hand.iter().filter(|c| c.definition == "JZ59").count() >= 2
            && p.hand
                .iter()
                .filter(|c| catalog::card(&c.definition).color == "白" && c.definition != "LC06")
                .count()
                >= 2
        {
            println!(
                "LC06 natural seed={seed} opening={:?} next8={:?}",
                p.hand
                    .iter()
                    .map(|c| c.definition.clone())
                    .collect::<Vec<_>>(),
                p.deck
                    .iter()
                    .take(8)
                    .map(|c| c.definition.clone())
                    .collect::<Vec<_>>()
            );
            return;
        }
    }
    panic!("no bounded natural seed");
}
pub(super) fn natural() -> Value {
    let seed = "9007199254743415";
    let d = natural_deck();
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "lc06-natural".into(),
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
    let mut searched = false;
    let mut lc = None;
    let mut jz = None;
    let mut killed = false;
    let mut drawn = false;
    let mut hand_before = 0;
    let mut selected_search = None;
    for iteration in 0..1800 {
        if iteration % 200 == 0 {
            eprintln!("LC06 natural iteration={iteration} turn={} searched={searched} lc={lc:?} jz={jz:?} killed={killed} drawn={drawn} assets={:?} hand={:?}",r.turn,r.players[0].assets.iter().map(|c|c.definition.clone()).collect::<Vec<_>>(),r.players[0].hand.iter().map(|c|c.definition.clone()).collect::<Vec<_>>());
        }
        if drawn && r.stack.is_empty() && r.pending.is_none() {
            assert_eq!(r.players[0].hand.len(), hand_before + 2);
            break;
        }
        if let Some(p) = r.pending.clone() {
            if matches!(&p.resolution, ChoiceResolution::Mulligan) {
                choose(&mut r, &mut steps, vec![]);
            } else if matches!(
                &p.resolution,
                ChoiceResolution::Frame {
                    choice: FrameChoice::Search { .. },
                    ..
                }
            ) && searched
                && lc.is_none()
            {
                let o = p
                    .choice
                    .options
                    .iter()
                    .find(|o| {
                        o.card
                            .as_ref()
                            .is_some_and(|c| c.card_id.as_deref() == Some("LC06"))
                    })
                    .expect("LC06 naturally remains in deck");
                selected_search = Some(o.id.clone());
                choose(&mut r, &mut steps, vec![o.id.clone()]);
            } else if matches!(&p.resolution, ChoiceResolution::Discard { redraw: false }) {
                let mut options = p.choice.options.clone();
                options.sort_by_key(|o| {
                    o.card.as_ref().is_some_and(|c| {
                        matches!(c.card_id.as_deref(), Some("JC091" | "JZ59" | "LC06"))
                    })
                });
                choose(
                    &mut r,
                    &mut steps,
                    options
                        .iter()
                        .take(p.choice.min.unwrap_or(0))
                        .map(|o| o.id.clone())
                        .collect(),
                );
            } else if trigger(&r, "JZ59") {
                choose(&mut r, &mut steps, vec![lc.clone().unwrap()]);
            } else if trigger(&r, "LC06") {
                assert!(killed);
                assert_eq!(p.seat, 0);
                hand_before = r.players[0].hand.len();
                for v in 0..4 {
                    assert_eq!(
                        r.view(v, r.pacing.last_server_now_ms)
                            .pending_choice
                            .is_some(),
                        v == 0
                    );
                }
                choose(&mut r, &mut steps, vec!["accept".into()]);
                drawn = true;
            } else {
                let (s, a) = pick_choice(&r.game);
                apply_game(&mut r, &mut steps, s, a);
            }
            continue;
        }
        let legal = r.game.legal_actions(0);
        let mut acted = false;
        if !searched {
            if let Some(a) = legal
                .iter()
                .find(|a| a.action.ability_id.as_deref() == Some("search-white-unique"))
            {
                apply_game(&mut r, &mut steps, 0, a.action.clone());
                searched = true;
                acted = true;
            }
        }
        let chain_ready = r.players[0].assets.iter().filter(|c| !c.exhausted).count() >= 5
            && ["白", "紫", "黑"].into_iter().all(|color| {
                r.players[0]
                    .assets
                    .iter()
                    .filter(|c| catalog::card(&c.definition).color == color)
                    .count()
                    >= if color == "白" { 2 } else { 1 }
            })
            && r.players[0].hand.iter().any(|c| c.definition == "JC091")
            && (jz.is_some() || r.players[0].hand.iter().any(|c| c.definition == "JZ59"));
        if !acted && searched && r.stack.is_empty() && (lc.is_some() || chain_ready) {
            for (definition, slot) in [("LC06", &mut lc), ("JZ59", &mut jz)] {
                if slot.is_none() {
                    if let Some(a) = legal.iter().find(|a| {
                        a.action.kind == "deploy"
                            && r.players[0].hand.iter().any(|c| {
                                a.action.card_id.as_ref() == Some(&c.id)
                                    && c.definition == definition
                            })
                    }) {
                        let mut a = a.action.clone();
                        a.region = Some(0);
                        apply_game(&mut r, &mut steps, 0, a);
                        advance(&mut r, &mut steps, |r| {
                            r.stack.is_empty() && r.pending.is_none()
                        });
                        *slot = Some(
                            r.regions[0]
                                .cards
                                .iter()
                                .find(|c| c.definition == definition && c.controller == 0)
                                .unwrap()
                                .id
                                .clone(),
                        );
                        acted = true;
                        break;
                    }
                }
            }
            if !acted && lc.is_some() && jz.is_some() && !killed {
                if let Some(a) = legal.iter().find(|a| {
                    a.action.kind == "play"
                        && r.players[0].hand.iter().any(|c| {
                            a.action.card_id.as_ref() == Some(&c.id) && c.definition == "JC091"
                        })
                }) {
                    let mut a = a.action.clone();
                    a.target_id = jz.clone();
                    apply_game(&mut r, &mut steps, 0, a);
                    killed = true;
                    acted = true;
                }
            }
        }
        if !acted && !killed {
            let assets = legal
                .iter()
                .filter(|a| a.action.kind == "asset")
                .collect::<Vec<_>>();
            let colors = [("白", 2), ("紫", 1), ("黑", 1)];
            let needed = colors
                .into_iter()
                .find(|(color, count)| {
                    r.players[0]
                        .assets
                        .iter()
                        .filter(|c| catalog::card(&c.definition).color == *color)
                        .count()
                        < *count
                })
                .map(|(color, _)| color);
            let candidate = assets
                .iter()
                .find(|a| {
                    r.players[0]
                        .hand
                        .iter()
                        .find(|c| a.action.card_id.as_ref() == Some(&c.id))
                        .is_some_and(|c| {
                            Some(catalog::card(&c.definition).color.as_str()) == needed
                                && c.definition != "LC06"
                                && (c.definition != "JZ59"
                                    || jz.is_some()
                                    || r.players[0]
                                        .hand
                                        .iter()
                                        .filter(|h| h.definition == "JZ59")
                                        .count()
                                        > 1)
                                && (c.definition != "JC091"
                                    || r.players[0]
                                        .hand
                                        .iter()
                                        .filter(|h| h.definition == "JC091")
                                        .count()
                                        > 1)
                        })
                })
                .or_else(|| {
                    assets.iter().find(|a| {
                        r.players[0]
                            .hand
                            .iter()
                            .find(|c| a.action.card_id.as_ref() == Some(&c.id))
                            .is_some_and(|c| {
                                !matches!(c.definition.as_str(), "LC06" | "JZ59" | "JC091")
                            })
                    })
                });
            if let Some(a) = candidate {
                apply_game(&mut r, &mut steps, 0, a.action.clone());
                acted = true;
            }
        }
        if !acted {
            pass(&mut r, &mut steps);
        }
    }
    assert!(
        drawn && r.stack.is_empty() && r.pending.is_none(),
        "natural LC06 chain exhausted bound"
    );
    let id = lc.unwrap();
    assert_eq!(board(&r, &id).unwrap().wounds, 1);
    assert!(board(&r, jz.as_ref().unwrap()).is_none());
    assert!(r.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JZ59"));
    json!({"name":"LC06-natural-four-seat-white-search-paid-entry-real-JZ59-death-wound-responsive-draw","seed":seed,"steps":steps,"syntheticInitialLayout":false,"normalRoomCommandsOnly":true,"ordinaryDeckSize":50,"whiteDeckCount":27,"sourceSearchOldInstance":selected_search,"recipientBoardInstance":id,"realPrintedWoundSource":"JZ59","newNaturalUiCoverage":false})
}
pub(super) fn cases() -> impl Iterator<Item = Value> {
    [
        "normal",
        "controller",
        "fatal-controller",
        "decline",
        "source-return",
        "target-return",
        "damage-prevention",
        "exhausted-two-wounds",
        "empty-deck",
        "damage",
        "destroy",
        "manual",
        "shield",
        "healing",
    ]
    .into_iter()
    .map(boundary)
    .chain(
        ["same-controller", "friend", "paid-reveal"]
            .into_iter()
            .map(unique),
    )
    .chain(std::iter::once_with(natural))
}
