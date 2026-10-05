//! Actual admitted card actions. Natural game has no post-construction overrides;
//! boundary scenarios have explicitly marked initial layouts/funding only.
use super::*;
use hegemony_server::model::{Card, ControlLifetime};
fn draft(opponent: bool) -> deck::DeckDraft {
    let mut d = msjc01::draft(25, 50);
    d.cards = if opponent {
        [
            "JC001", "JC002", "JC003", "JC004", "JC005", "JC006", "JC008", "LC01",
        ]
        .into_iter()
        .map(|id| catalog::DeckEntry {
            card_id: id.into(),
            count: 3,
        })
        .chain([catalog::DeckEntry {
            card_id: "JC007".into(),
            count: 1,
        }])
        .collect()
    } else {
        d.cards
            .into_iter()
            .filter(|c| c.card_id != "JC125")
            .map(|mut c| {
                if c.card_id == "JC007" {
                    c.card_id = "WM003".into();
                }
                c
            })
            .collect()
    };
    d.cards.extend(
        ["XQ12", "XQ16", "JC104", "JC129", "JC036", "JZ27"]
            .into_iter()
            .map(|id| catalog::DeckEntry {
                card_id: id.into(),
                count: 3,
            }),
    );
    d.cards.push(catalog::DeckEntry {
        card_id: "JC125".into(),
        count: 7,
    });
    deck::validate(d).unwrap()
}
fn field(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn fund(g: &mut Game, seat: usize, def: &str, count: usize) {
    for _ in 0..count {
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
fn card_on<'a>(g: &'a Game, def: &str) -> Option<&'a Card> {
    g.regions
        .iter()
        .flat_map(|r| r.cards.iter())
        .find(|c| c.definition == def)
}
fn checkpoint(r: &RoomEnvelope, steps: &mut Vec<Value>) -> Value {
    for s in 0..4 {
        let view = r.view(s, r.pacing.last_server_now_ms).game;
        let eligible = r
            .regions
            .iter()
            .flat_map(|region| &region.cards)
            .any(|c| c.definition == "LC01" && !c.face_down && c.controller == s);
        assert_eq!(
            view.private_deck_top.as_ref().map(|c| &c.instance_id),
            if eligible {
                r.players[s].deck.first().map(|c| &c.id)
            } else {
                None
            }
        );
    }
    json!({"step":steps.len()-1,"turn":r.turn,"controls":r.control_effects,"LC01":card_on(&r.game,"LC01").map(|c|json!({"id":c.id,"owner":c.owner,"controller":c.controller,"hidden":c.face_down,"types":r.current_subtypes(c)}))})
}
fn choose_target(r: &mut RoomEnvelope, steps: &mut Vec<Value>, target: &str) {
    let p = r.pending.clone().unwrap();
    apply_game(
        r,
        steps,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![target.into()]),
            ..Action::new("choose")
        },
    );
}
fn advance(r: &mut RoomEnvelope, steps: &mut Vec<Value>, test: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..700 {
        if test(r) {
            return;
        }
        let (seat, action) = if r.pending.is_some() {
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
        apply_game(r, steps, seat, action);
    }
    panic!("bounded control advance");
}
fn available(r: &RoomEnvelope, seat: usize, def: &str, kind: &str, target: Option<&str>) -> Action {
    r.game
        .legal_actions(seat)
        .into_iter()
        .find(|a| {
            a.action.kind == kind
                && a.action.card_id.as_ref().is_some_and(|id| {
                    r.players[seat]
                        .hand
                        .iter()
                        .chain(r.regions.iter().flat_map(|x| &x.cards))
                        .any(|c| c.id == *id && c.definition == def)
                })
                && target.is_none_or(|t| a.action.target_id.as_deref() == Some(t))
                && a.action.region.is_none_or(|region| region == 0)
        })
        .unwrap_or_else(|| {
            panic!(
                "missing {seat} {def} {kind}, turn {} priority {}",
                r.turn, r.priority_team
            )
        })
        .action
}
fn standard(r: &mut RoomEnvelope, steps: &mut Vec<Value>, seat: usize) {
    advance(r, steps, |r| {
        r.pending.is_none()
            && r.stack.is_empty()
            && r.priority_team == r.game.team(seat)
            && r.window == Some(Window::Action(r.game.team(seat)))
    });
}
fn cast(r: &mut RoomEnvelope, steps: &mut Vec<Value>, seat: usize, def: &str, target: &str) {
    let action = available(r, seat, def, "play", Some(target));
    apply_game(r, steps, seat, action);
    attachment_pass_top(r, steps);
}
fn hide_by_lawyer(r: &mut RoomEnvelope, steps: &mut Vec<Value>, target: &str) {
    standard(r, steps, 0);
    let a = available(r, 0, "XQ16", "deploy", None);
    apply_game(r, steps, 0, a);
    attachment_pass_top(r, steps);
    choose_target(r, steps, target);
    attachment_pass_top(r, steps);
}
fn boundary(kind: &str) -> Value {
    let mut g = attachment_initial("teams", "888888888888888888880115");
    let t = field(&mut g, "LC01", 2);
    fund(&mut g, 0, "XQ16", 12);
    fund(&mut g, 0, "JC104", 6);
    fund(&mut g, 0, "JC002", 4);
    fund(&mut g, 2, "XQ16", 12);
    fund(&mut g, 2, "JC104", 6);
    fund(&mut g, 2, "JC002", 5);
    held(&mut g, 0, "JC008");
    held(&mut g, 0, "JC129");
    held(&mut g, 0, "XQ16");
    held(&mut g, 0, "JC005");
    held(&mut g, 2, "JC129");
    held(&mut g, 2, "JC036");
    held(&mut g, 2, "JC006");
    let source = field(&mut g, "JZ27", 0);
    card_on(&g, "JZ27").unwrap();
    g.regions[0]
        .cards
        .iter_mut()
        .find(|c| c.id == source)
        .unwrap()
        .face_down = true;
    let mut r = RoomEnvelope::from_game(g);
    RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();
    let mut steps = vec![step(
        &r,
        "initialFixture",
        json!([serde_json::to_string(&r).unwrap()]),
        0,
    )];
    let mut checkpoints = vec![];
    if kind == "temporary-target-hide" {
        cast(&mut r, &mut steps, 0, "JC008", &t);
        cast(&mut r, &mut steps, 0, "JC129", &t);
        assert_eq!(card_on(&r.game, "LC01").unwrap().controller, 0);
        checkpoints.push(checkpoint(&r, &mut steps));
        hide_by_lawyer(&mut r, &mut steps, &t);
        let c = card_on(&r.game, "LC01").unwrap();
        assert!(c.face_down && c.controller == 2 && c.id != t);
        assert!(r.control_effects.is_empty());
        assert!(r.turn_attribute_modifiers.is_empty());
        checkpoints.push(checkpoint(&r, &mut steps));
        let old = c.id.clone();
        standard(&mut r, &mut steps, 2);
        let a = available(&r, 2, "LC01", "reveal", None);
        apply_game(&mut r, &mut steps, 2, a);
        attachment_pass_top(&mut r, &mut steps);
        let c = card_on(&r.game, "LC01").unwrap();
        assert!(c.id != old && c.controller == 2 && !c.face_down);
        checkpoints.push(checkpoint(&r, &mut steps));
    } else {
        let a = available(&r, 0, "JZ27", "reveal", None);
        apply_game(&mut r, &mut steps, 0, a);
        attachment_pass_top(&mut r, &mut steps);
        choose_target(&mut r, &mut steps, &t);
        attachment_pass_top(&mut r, &mut steps);
        let source = card_on(&r.game, "JZ27").unwrap().id.clone();
        assert_eq!(card_on(&r.game, "LC01").unwrap().controller, 0);
        checkpoints.push(checkpoint(&r, &mut steps));
        if kind == "source-hide" {
            hide_by_lawyer(&mut r, &mut steps, &source);
            assert_eq!(card_on(&r.game, "LC01").unwrap().controller, 2);
            assert!(r.control_effects.is_empty());
            checkpoints.push(checkpoint(&r, &mut steps));
        } else if kind == "source-controller-fixed-recipient" {
            standard(&mut r, &mut steps, 2);
            cast(&mut r, &mut steps, 2, "JC129", &source);
            assert_eq!(card_on(&r.game, "JZ27").unwrap().controller, 2);
            assert_eq!(card_on(&r.game, "LC01").unwrap().controller, 0);
            checkpoints.push(checkpoint(&r, &mut steps));
            let turn = r.turn;
            advance(&mut r, &mut steps, |r| r.turn > turn);
            assert_eq!(card_on(&r.game, "JZ27").unwrap().controller, 0);
            assert_eq!(card_on(&r.game, "LC01").unwrap().controller, 0);
            checkpoints.push(checkpoint(&r, &mut steps));
        } else {
            standard(&mut r, &mut steps, 2);
            cast(&mut r, &mut steps, 2, "JC036", &t);
            let a = r
                .attachments
                .iter()
                .find(|a| a.card.definition == "JC036")
                .unwrap()
                .card
                .id
                .clone();
            assert_eq!(card_on(&r.game, "LC01").unwrap().controller, 2);
            checkpoints.push(checkpoint(&r, &mut steps));
            standard(&mut r, &mut steps, 0);
            cast(&mut r, &mut steps, 0, "JC129", &t);
            assert_eq!(card_on(&r.game, "LC01").unwrap().controller, 0);
            checkpoints.push(checkpoint(&r, &mut steps));
            if kind == "lower-removed-under-upper" {
                cast(&mut r, &mut steps, 0, "JC005", &a);
                assert_eq!(card_on(&r.game, "LC01").unwrap().controller, 0);
            }
            let turn = r.turn;
            advance(&mut r, &mut steps, |r| r.turn > turn);
            assert_eq!(
                card_on(&r.game, "LC01").unwrap().controller,
                if kind == "lower-removed-under-upper" {
                    0
                } else {
                    2
                }
            );
            checkpoints.push(checkpoint(&r, &mut steps));
            if kind != "lower-removed-under-upper" {
                standard(&mut r, &mut steps, 0);
                cast(&mut r, &mut steps, 0, "JC005", &a);
                assert_eq!(card_on(&r.game, "LC01").unwrap().controller, 0);
                checkpoints.push(checkpoint(&r, &mut steps));
            }
            standard(&mut r, &mut steps, 2);
            cast(&mut r, &mut steps, 2, "JC006", &source);
            assert_eq!(card_on(&r.game, "LC01").unwrap().controller, 2);
            assert_eq!(
                r.current_subtypes(card_on(&r.game, "LC01").unwrap()),
                vec!["人类", "法师"]
            );
            checkpoints.push(checkpoint(&r, &mut steps));
        }
    }
    json!({"name":format!("control-{kind}"),"seed":"9007199254740993","syntheticInitialLayout":true,"syntheticInitialFunding":true,"postInitialStateInjection":false,"normalPrintedCardControlOnly":true,"publicNaturalUiAcceptance":false,"checkpoints":checkpoints,"steps":steps})
}

fn reserve(seat: usize, def: &str) -> usize {
    match (seat, def) {
        (0, "JC129" | "JC036" | "JZ27" | "WM003") => 1,
        (0, "XQ16") => 2,
        (2, "LC01" | "JC008") => 1,
        (1, "JC125") => 1,
        _ => 0,
    }
}
fn search_need(
    g: &Game,
    stolen: bool,
    embraced: bool,
    source_started: bool,
    target_hidden: bool,
) -> Option<&'static str> {
    let count = |def| {
        g.players[0]
            .hand
            .iter()
            .filter(|c| c.definition == def)
            .count()
    };
    if g.players[0]
        .assets
        .iter()
        .filter(|c| catalog::card(&c.definition).color == "蓝")
        .count()
        < 3
        && count("XQ12") == 0
        && g.players[0].deck.iter().any(|c| c.definition == "XQ12")
    {
        Some("XQ12")
    } else if !stolen && count("JC129") == 0 {
        Some("JC129")
    } else if !embraced && count("JC036") == 0 {
        Some("JC036")
    } else if !source_started && count("JZ27") == 0 && card_on(g, "JZ27").is_none() {
        Some("JZ27")
    } else if !target_hidden && count("XQ16") < 2 {
        Some("XQ16")
    } else {
        None
    }
}
fn natural_actions(seed: u64) -> Option<Vec<(usize, Action)>> {
    let d = draft(false);
    let enemy = draft(true);
    let mut g = Game::new_with_deck(
        "888888888888888888880116".into(),
        "NATURAL".into(),
        "teams".into(),
        "P0".into(),
        d.clone(),
        seed,
    )
    .unwrap();
    for seat in 1..4 {
        g.join_with_deck(
            format!("P{seat}"),
            if seat == 2 { enemy.clone() } else { d.clone() },
        )
        .unwrap();
    }
    let mut actions = vec![];
    for seat in 0..4 {
        let a = Action::new("ready");
        g.apply(seat, a.clone()).unwrap();
        actions.push((seat, a));
    }
    let a = Action::new("start");
    g.apply(0, a.clone()).unwrap();
    actions.push((0, a));
    let (mut stolen, mut buffed, mut embraced, mut searched, mut source_hidden, mut target_hidden) =
        (false, false, false, false, false, false);
    let mut source_started = false;
    for _ in 0..4096 {
        if g.status != "playing" || g.turn > 22 {
            eprintln!("control seed {seed} stops turn={} status={} stolen={stolen} buffed={buffed} embraced={embraced} source_started={source_started} source_hidden={source_hidden} target_hidden={target_hidden} assets0={:?} hand0={:?} board={:?}",g.turn,g.status,g.players[0].assets.iter().map(|c|&c.definition).collect::<Vec<_>>(),g.players[0].hand.iter().map(|c|&c.definition).collect::<Vec<_>>(),g.regions.iter().flat_map(|r|&r.cards).map(|c|(&c.definition,c.controller,c.face_down)).collect::<Vec<_>>());
            return None;
        }
        if let Some(c) = card_on(&g, "LC01") {
            if !stolen
                && c.controller == 0
                && g.control_effects
                    .iter()
                    .any(|e| matches!(e.lifetime, ControlLifetime::TurnEnd { .. }))
            {
                if !buffed
                    || !g
                        .turn_attribute_modifiers
                        .iter()
                        .any(|m| m.target_instance == c.id)
                {
                    eprintln!("control seed {seed} lacks paid JC008 response");
                    return None;
                }
                stolen = true;
            }
            if stolen && !embraced && g.attachments.iter().any(|a| a.card.definition == "JC036") {
                embraced = true;
            }
            if embraced && c.face_down && c.controller == 2 {
                target_hidden = true;
            }
        }
        if g.control_effects
            .iter()
            .any(|e| matches!(e.lifetime, ControlLifetime::SourceLeaves { .. }))
        {
            source_started = true;
        }
        if source_started
            && card_on(&g, "JZ27").is_some_and(|c| c.face_down)
            && g.control_effects
                .iter()
                .all(|e| !matches!(e.lifetime, ControlLifetime::SourceLeaves { .. }))
        {
            source_hidden = true;
        }
        if target_hidden && source_hidden && g.stack.is_empty() && g.pending.is_none() {
            return Some(actions);
        }
        let selected = if let Some(p) = g.pending.as_ref() {
            if p.choice.kind == "mulligan" {
                Some((
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(vec![]),
                        ..Action::new("choose")
                    },
                ))
            } else {
                let source = match &p.resolution {
                    ChoiceResolution::Declare { declaration, .. } => {
                        Some(declaration.source.card.definition.as_str())
                    }
                    _ => None,
                };
                let target = match source {
                    Some("XQ16") => {
                        if !source_hidden {
                            card_on(&g, "JZ27")
                                .filter(|c| !c.face_down)
                                .map(|c| c.id.clone())
                        } else {
                            card_on(&g, "LC01")
                                .filter(|c| !c.face_down)
                                .map(|c| c.id.clone())
                        }
                    }
                    Some("JZ27") => g
                        .regions
                        .iter()
                        .flat_map(|r| &r.cards)
                        .find(|c| c.definition == "JC125" && c.owner == 1 && !c.face_down)
                        .map(|c| c.id.clone()),
                    _ => {
                        if p.choice.kind == "search" {
                            let need = if p.seat == 2 {
                                Some("LC01")
                            } else {
                                search_need(&g, stolen, embraced, source_started, target_hidden)
                            };
                            p.choice
                                .options
                                .iter()
                                .find(|o| {
                                    o.card.as_ref().and_then(|c| c.card_id.as_deref()) == need
                                })
                                .map(|o| o.id.clone())
                        } else {
                            None
                        }
                    }
                };
                if let Some(id) = target.filter(|id| p.choice.options.iter().any(|o| o.id == *id)) {
                    Some((
                        p.seat,
                        Action {
                            choice_id: Some(p.choice.id.clone()),
                            selected: Some(vec![id]),
                            ..Action::new("choose")
                        },
                    ))
                } else if p.choice.kind == "discard" {
                    let mut options = p.choice.options.clone();
                    options.sort_by_key(|o| {
                        o.card
                            .as_ref()
                            .and_then(|c| c.card_id.as_deref())
                            .map_or(0, |def| reserve(p.seat, def))
                    });
                    Some((
                        p.seat,
                        Action {
                            choice_id: Some(p.choice.id.clone()),
                            selected: Some(
                                options
                                    .into_iter()
                                    .take(p.choice.min.unwrap_or(0))
                                    .map(|o| o.id)
                                    .collect(),
                            ),
                            ..Action::new("choose")
                        },
                    ))
                } else {
                    Some(pick_choice(&g))
                }
            }
        } else {
            let legal = (0..4).map(|s| g.legal_actions(s)).collect::<Vec<_>>();
            let mut selected = None;
            // Opponent responds to the real control transaction with its real JC008.
            if !buffed
                && g.stack
                    .iter()
                    .any(|i| i.card.as_ref().is_some_and(|c| c.definition == "JC129"))
            {
                if let Some(target) = card_on(&g, "LC01") {
                    selected = legal[2]
                        .iter()
                        .find(|a| {
                            a.action.kind == "play"
                                && a.action.target_id.as_deref() == Some(&target.id)
                                && a.action.card_id.as_ref().is_some_and(|id| {
                                    g.players[2]
                                        .hand
                                        .iter()
                                        .any(|c| c.id == *id && c.definition == "JC008")
                                })
                        })
                        .map(|a| (2, a.action.clone()));
                    if selected.is_some() {
                        buffed = true;
                    }
                }
            }
            for seat in [0, 1, 2] {
                if selected.is_some() {
                    break;
                }
                for a in &legal[seat] {
                    let def = a
                        .action
                        .card_id
                        .as_ref()
                        .and_then(|id| {
                            g.players[seat]
                                .hand
                                .iter()
                                .chain(g.regions.iter().flat_map(|r| &r.cards))
                                .find(|c| c.id == *id)
                        })
                        .map(|c| c.definition.as_str());
                    let want = if seat == 0 && def == Some("WM003") {
                        a.action.kind == "deploy" && card_on(&g, "WM003").is_none()
                            || a.action.ability_id.as_deref() == Some("search-any-private")
                                && search_need(&g, stolen, embraced, source_started, target_hidden)
                                    .is_some_and(|need| {
                                        g.players[0].deck.iter().any(|c| c.definition == need)
                                    })
                    } else if seat == 1 {
                        embraced
                            && g.players[0].assets.len() >= 8
                            && def == Some("JC125")
                            && a.action.kind == "deploy"
                            && !g
                                .regions
                                .iter()
                                .flat_map(|r| &r.cards)
                                .any(|c| c.owner == 1 && c.definition == "JC125")
                    } else if seat == 2 {
                        def == Some("LC01")
                            && a.action.kind == "deploy"
                            && card_on(&g, "LC01").is_none()
                    } else if !stolen {
                        def == Some("JC129")
                            && a.action.kind == "play"
                            && g.players[2].assets.iter().filter(|c| !c.exhausted).count() >= 2
                            && g.players[2].hand.iter().any(|c| c.definition == "JC008")
                            && card_on(&g, "LC01").is_some_and(|c| {
                                a.action.target_id.as_deref() == Some(&c.id) && c.controller == 2
                            })
                    } else if !embraced {
                        g.players[0].assets.len() >= 8
                            && def == Some("JC036")
                            && a.action.kind == "play"
                            && card_on(&g, "LC01").is_some_and(|c| {
                                a.action.target_id.as_deref() == Some(&c.id) && c.controller == 2
                            })
                    } else if card_on(&g, "JZ27").is_none() {
                        def == Some("JZ27")
                            && a.action.kind == "conceal"
                            && g.players[0].assets.len() >= 8
                    } else if card_on(&g, "JZ27").is_some_and(|c| c.face_down) && !source_hidden {
                        def == Some("JZ27")
                            && a.action.kind == "reveal"
                            && g.regions[2].cards.iter().any(|c| {
                                c.owner == 1
                                    && !c.face_down
                                    && g.current_subtypes(c).iter().any(|t| t == "人类")
                            })
                    } else {
                        def == Some("XQ16")
                            && a.action.kind == "deploy"
                            && g.control_effects
                                .iter()
                                .any(|e| matches!(e.lifetime, ControlLifetime::SourceLeaves { .. }))
                            || source_hidden
                                && !target_hidden
                                && def == Some("XQ16")
                                && a.action.kind == "deploy"
                    };
                    // Centre region is accessible to all four normal deployment lanes.
                    if want && a.action.region.is_none_or(|r| r == 2) {
                        selected = Some((seat, a.action.clone()));
                        break;
                    }
                }
            }
            if selected.is_none() && !searched && card_on(&g, "LC01").is_none() {
                selected = legal[2]
                    .iter()
                    .find(|a| a.action.ability_id.as_deref() == Some("search-yellow-unique"))
                    .map(|a| (2, a.action.clone()));
                if selected.is_some() {
                    searched = true;
                }
            }
            for seat in [0, 2] {
                if selected.is_some() {
                    break;
                }
                let blue = g.players[seat]
                    .assets
                    .iter()
                    .filter(|c| catalog::card(&c.definition).color == "蓝")
                    .count();
                let purple = g.players[seat]
                    .assets
                    .iter()
                    .filter(|c| catalog::card(&c.definition).color == "紫")
                    .count();
                let mut choices = legal[seat]
                    .iter()
                    .filter(|a| a.action.kind == "asset")
                    .filter_map(|a| {
                        let c = g.players[seat]
                            .hand
                            .iter()
                            .find(|c| Some(&c.id) == a.action.card_id.as_ref())?;
                        if g.players[seat]
                            .hand
                            .iter()
                            .filter(|h| h.definition == c.definition)
                            .count()
                            <= reserve(seat, &c.definition)
                        {
                            return None;
                        }
                        let color = &catalog::card(&c.definition).color;
                        let score = if seat == 0 && blue < 3 && color == "蓝"
                            || seat == 2 && color == "黄"
                        {
                            0
                        } else if seat == 0 && purple < 1 && color == "紫" {
                            1
                        } else {
                            2
                        };
                        Some((score, a.action.clone()))
                    })
                    .collect::<Vec<_>>();
                choices.sort_by_key(|x| x.0);
                if let Some((_, a)) = choices.into_iter().next() {
                    selected = Some((seat, a));
                }
            }
            selected.or_else(|| {
                legal.iter().enumerate().find_map(|(s, aa)| {
                    aa.iter()
                        .find(|a| a.action.kind == "pass")
                        .map(|a| (s, a.action.clone()))
                })
            })
        };
        let Some((seat, a)) = selected else {
            eprintln!(
                "control seed {seed} no legal continuation turn={} pending={:?}",
                g.turn,
                g.pending.as_ref().map(|p| &p.choice.kind)
            );
            return None;
        };
        if let Err(error) = g.apply(seat, a.clone()) {
            eprintln!(
                "control seed {seed} action {} seat {seat} rejected: {error}",
                a.kind
            );
            return None;
        }
        actions.push((seat, a));
    }
    eprintln!("control seed {seed} reached action bound stolen={stolen} embraced={embraced} source_started={source_started} source_hidden={source_hidden} target_hidden={target_hidden} turn={} LC01={:?} assets0={:?} hand0={:?} window={:?}",g.turn,card_on(&g,"LC01").map(|c|(&c.id,c.controller)),g.players[0].assets.iter().map(|c|&c.definition).collect::<Vec<_>>(),g.players[0].hand.iter().map(|c|&c.definition).collect::<Vec<_>>(),g.window);
    None
}
fn natural_case() -> Value {
    // Fixed reproducible seed selected during fixture development; no state override.
    let seed = 5;
    let actions = natural_actions(seed).expect("natural control lifecycle path");
    let d = draft(false);
    let enemy = draft(true);
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "888888888888888888880116".into(),
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
    for seat in 1..4 {
        let next = if seat == 2 { enemy.clone() } else { d.clone() };
        r.game
            .join_with_deck(format!("P{seat}"), next.clone())
            .unwrap();
        r.revision = r.game.version;
        steps.push(step(
            &r,
            "joinGameWithDeck",
            json!([format!("P{seat}"), serde_json::to_string(&next).unwrap()]),
            seat,
        ));
    }
    let mut milestones = vec![];
    let mut observed = [false; 4];
    for (seat, a) in actions {
        apply_game(&mut r, &mut steps, seat, a);
        if let Some(c) = card_on(&r.game, "LC01") {
            if !observed[0]
                && c.controller == 0
                && r.turn_attribute_modifiers
                    .iter()
                    .any(|m| m.target_instance == c.id)
            {
                observed[0] = true;
                milestones.push(checkpoint(&r, &mut steps));
            }
            if !observed[1] && r.attachments.iter().any(|a| a.card.definition == "JC036") {
                observed[1] = true;
                milestones.push(checkpoint(&r, &mut steps));
            }
            if !observed[3] && c.face_down && c.controller == 2 {
                observed[3] = true;
                milestones.push(checkpoint(&r, &mut steps));
            }
        }
        if !observed[2]
            && r.control_effects
                .iter()
                .any(|e| matches!(e.lifetime, ControlLifetime::SourceLeaves { .. }))
        {
            observed[2] = true;
            milestones.push(checkpoint(&r, &mut steps));
        }
    }
    assert!(
        observed.iter().all(|v| *v),
        "seed {seed} observed {observed:?}"
    );
    assert!(r.control_effects.is_empty());
    assert!(card_on(&r.game, "JZ27").unwrap().face_down);
    assert!(card_on(&r.game, "LC01").unwrap().face_down);
    checkpoint(&r, &mut steps);
    eprintln!(
        "natural control seed={seed} steps={} turn={} milestones={:?}",
        steps.len(),
        r.turn,
        observed
    );
    json!({"name":"control-natural-four-seat-real-50-card-decks","seed":seed.to_string(),"syntheticInitialLayout":false,"postConstructionStateOverrides":false,"publicNaturalUiAcceptance":false,"milestones":milestones,"steps":steps})
}
pub(super) fn cases() -> impl Iterator<Item = Value> {
    [
        "temporary-target-hide",
        "source-hide",
        "source-controller-fixed-recipient",
        "three-layers-expiry",
        "lower-removed-under-upper",
    ]
    .into_iter()
    .map(boundary)
    .chain(std::iter::once_with(natural_case))
}
