//! Disclosed boundary layouts; every later transition uses real Room commands.
//! Marked response checkpoints are explicit synthetic continuations, never natural UI.
use super::*;
use hegemony_server::model::{Attachment, Card};
fn initial(name: &str) -> Game {
    let mut g = attachment_initial("teams", &format!("dream-reveal-{name}"));
    for p in &mut g.players {
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.influence = [0, 0];
    }
    fund(&mut g, "JC091", 0, 6);
    g
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
fn board<'a>(g: &'a Game, id: &str) -> Option<&'a Card> {
    g.regions.iter().flat_map(|r| &r.cards).find(|c| c.id == id)
}
fn resources(g: &Game, s: usize) -> usize {
    g.players[s].assets.iter().filter(|c| !c.exhausted).count()
}
fn room(g: Game) -> (RoomEnvelope, Vec<Value>) {
    checkpoint(RoomEnvelope::from_game(g))
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
fn result(name: &str, steps: Vec<Value>) -> Value {
    json!({"name":format!("dream-reveal-{name}"),"seed":"9007199254740993","syntheticInitialLayout":true,"newNaturalUiCoverage":false,"steps":steps})
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
fn drain(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    for _ in 0..150 {
        if r.stack.is_empty() && r.pending.is_none() {
            return;
        }
        if r.pending.is_some() {
            let (s, a) = pick_choice(&r.game);
            apply_game(r, steps, s, a);
        } else {
            pass(r, steps)
        }
    }
    panic!("bounded dream drain")
}
fn advance(r: &mut RoomEnvelope, steps: &mut Vec<Value>, done: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..650 {
        if done(r) {
            return;
        }
        if r.pending.is_some() {
            let (s, a) = pick_choice(&r.game);
            apply_game(r, steps, s, a);
        } else {
            pass(r, steps)
        }
    }
    panic!("bounded dream advance")
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
fn activate(id: &str, key: &str, target: Option<&str>) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(key.into()),
        target_id: target.map(str::to_string),
        ..Action::new("activate")
    }
}
fn play(id: &str, target: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        target_id: Some(target.into()),
        ..Action::new("play")
    }
}
fn pass_action(g: &Game) -> (usize, Action) {
    (0..g.players.len())
        .find_map(|s| {
            g.legal_actions(s)
                .into_iter()
                .find(|a| a.action.kind == "pass")
                .map(|a| (s, a.action))
        })
        .expect("a real pass must be legal")
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
fn kill(r: &mut RoomEnvelope, steps: &mut Vec<Value>, c: &str, source: &str) {
    apply_game(r, steps, 0, play(c, source));
    advance(r, steps, |r| {
        r.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "trigger")
    });
}
fn spirit(kind: &str) -> Value {
    let mut g = initial(kind);
    let s = field(&mut g, "JZ58", 2);
    if kind == "tie" || kind == "deficit" || kind == "simultaneous" || kind == "attachment" {
        field(&mut g, "LC12", 0);
    }
    if kind == "deficit" {
        field(&mut g, "LC12", 0);
    }
    if kind == "attachment" {
        let host = field(&mut g, "JC125", 2);
        let c = g.make_card("JC073", 2);
        g.attachments.push(Attachment {
            card: c,
            host_id: host,
        });
        fund(&mut g, "LC12", 0, 6);
        let c = field(&mut g, "LC12", 0);
        let card = g.regions[0].cards.pop().unwrap();
        assert_eq!(c, card.id);
        g.regions[1].cards.push(card);
    }
    let friendly = if kind == "simultaneous" {
        let friendly = field(&mut g, "LC12", 2);
        let enemy = g.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "LC12" && c.controller == 0)
            .unwrap()
            .id
            .clone();
        let c = g.make_card("XQ47", 0);
        g.attachments.push(Attachment {
            card: c,
            host_id: enemy,
        });
        Some(friendly)
    } else {
        None
    };
    let card = if kind == "simultaneous" {
        "JC047"
    } else {
        "JC102"
    };
    fund(&mut g, card, 0, 4);
    let spell = held(&mut g, card, 0);
    let (mut r, mut steps) = room(g);
    let protected = !matches!(kind, "tie" | "deficit");
    assert_eq!(r.game.spirit_protected(board(&r, &s).unwrap()), protected);
    apply_game(
        &mut r,
        &mut steps,
        0,
        if kind == "simultaneous" {
            Action {
                card_id: Some(spell),
                region: Some(0),
                ..Action::new("play")
            }
        } else {
            play(&spell, &s)
        },
    );
    drain(&mut r, &mut steps);
    if protected {
        assert_eq!(board(&r, &s).unwrap().damage, 0);
    } else {
        assert!(board(&r, &s).is_none());
        assert!(r.players[2]
            .graveyard
            .iter()
            .any(|c| c.definition == "JZ58" && c.id != s));
    }
    if let Some(f) = friendly {
        assert!(board(&r, &f).is_none());
        assert!(!r.game.spirit_protected(board(&r, &s).unwrap()));
    }
    result(&format!("real-spirit-damage-{kind}"), steps)
}
fn spirit_bypass(wound: bool) -> Value {
    let mut g = initial("spirit-bypass");
    let s = field(&mut g, "JZ58", 2);
    let source = field(&mut g, if wound { "JZ59" } else { "JC088" }, 0);
    let card = if wound {
        Some(held(&mut g, "JC091", 0))
    } else {
        field(&mut g, "LC12", 2);
        fund(&mut g, "JC088", 0, 3);
        g.regions[0]
            .cards
            .iter_mut()
            .find(|c| c.id == source)
            .unwrap()
            .face_down = true;
        None
    };
    let (mut r, mut steps) = room(g);
    assert!(r.game.spirit_protected(board(&r, &s).unwrap()));
    if let Some(c) = card {
        kill(&mut r, &mut steps, &c, &source);
    } else {
        apply_game(
            &mut r,
            &mut steps,
            0,
            Action {
                card_id: Some(source),
                ..Action::new("reveal")
            },
        );
        advance(&mut r, &mut steps, |r| {
            r.pending
                .as_ref()
                .is_some_and(|p| p.choice.kind == "trigger")
        });
    }
    assert!(r.game.spirit_protected(board(&r, &s).unwrap()));
    choose(&mut r, &mut steps, vec![s.clone()]);
    drain(&mut r, &mut steps);
    assert!(board(&r, &s).is_none());
    result(
        if wound {
            "real-spirit-wound-still-kills"
        } else {
            "real-spirit-assassin-destruction-still-kills"
        },
        steps,
    )
}
fn spirit_combat() -> Value {
    let mut g = initial("combat");
    let s = field(&mut g, "JZ58", 0);
    fund(&mut g, "JC008", 0, 2);
    let c = held(&mut g, "JC008", 0);
    let (mut r, mut steps) = room(g);
    apply_game(&mut r, &mut steps, 0, play(&c, &s));
    drain(&mut r, &mut steps);
    assert_eq!(r.game.icons(board(&r, &s).unwrap(), 0).combat, 1);
    assert!(r.game.spirit_protected(board(&r, &s).unwrap()));
    result("real-spirit-granted-combat-participates", steps)
}
fn repression(kind: &str) -> Value {
    let mut g = initial(kind);
    let s = field(&mut g, "JZ58", 0);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    fund(&mut g, "JC103", 0, 1);
    if kind != "empty" && kind != "decline" {
        g.regions[2].influence = [1, 2];
        g.regions[4].influence = [0, 1];
    }
    let rescue = if kind == "source-return" {
        Some(field(&mut g, "JC075", 0))
    } else {
        None
    };
    let (mut r, mut steps) = room(g);
    let before = resources(&r, 0);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(s.clone()),
            ..Action::new("reveal")
        },
    );
    advance(&mut r, &mut steps, |r| {
        r.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "trigger")
    });
    let p = r.pending.clone().unwrap();
    assert_eq!(p.seat, 0);
    assert_eq!(
        p.choice
            .options
            .iter()
            .map(|o| o.id.as_str())
            .collect::<Vec<_>>(),
        vec!["p2", "p3"]
    );
    if kind == "reject" {
        for id in ["p0", "p1"] {
            reject(
                &mut r,
                &mut steps,
                0,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec![id.into()]),
                    ..Action::new("choose")
                },
            );
        }
    }
    if kind == "decline" {
        choose(&mut r, &mut steps, vec![]);
        drain(&mut r, &mut steps);
        assert_eq!(resources(&r, 0), before);
        return result("paid-zero-reveal-decline-free", steps);
    }
    let seat = if kind == "target3" { 3 } else { 2 };
    choose(&mut r, &mut steps, vec![format!("p{seat}")]);
    if let Some(rescue) = rescue {
        let source = r.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JZ58")
            .unwrap()
            .id
            .clone();
        apply_game(
            &mut r,
            &mut steps,
            0,
            activate(&rescue, "rescue", Some(&source)),
        );
    }
    if kind == "empty" {
        drain(&mut r, &mut steps);
        assert!(r.pending.is_none());
        assert_eq!(resources(&r, 0), before);
        return result("paid-zero-reveal-no-enemy-influence", steps);
    }
    advance(&mut r, &mut steps, |r| {
        r.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "target")
    });
    let p = r.pending.clone().unwrap();
    assert_eq!(p.seat, seat);
    assert_eq!(
        p.choice
            .options
            .iter()
            .map(|o| o.id.as_str())
            .collect::<Vec<_>>(),
        vec!["region:2", "region:4"]
    );
    for viewer in 0..4 {
        assert_eq!(
            r.view(viewer, r.pacing.last_server_now_ms)
                .pending_choice
                .is_some(),
            viewer == seat
        );
    }
    if kind == "reject" {
        for actor in [0, if seat == 2 { 3 } else { 2 }] {
            reject(
                &mut r,
                &mut steps,
                actor,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec!["region:4".into()]),
                    ..Action::new("choose")
                },
            );
        }
        reject(
            &mut r,
            &mut steps,
            seat,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec!["region:0".into()]),
                ..Action::new("choose")
            },
        );
    }
    choose(&mut r, &mut steps, vec!["region:4".into()]);
    drain(&mut r, &mut steps);
    assert_eq!(r.regions[2].influence, [1, 2]);
    assert_eq!(r.regions[4].influence, [0, 0]);
    assert_eq!(
        resources(&r, 0),
        before - if kind == "source-return" { 2 } else { 0 }
    );
    if kind == "source-return" {
        assert!(r.players[0]
            .hand
            .iter()
            .any(|c| c.definition == "JZ58" && c.id != s));
    }
    result(&format!("paid-zero-reveal-repress-{kind}"), steps)
}
fn death_search(kind: &str) -> Value {
    let mut g = initial(kind);
    let s = field(&mut g, "JZ61", if kind == "controller" { 3 } else { 0 });
    if kind == "controller" {
        g.regions[0].cards.last_mut().unwrap().controller = 0;
    }
    let n = if kind != "empty" {
        let c = g.make_card("JZ58", 0);
        let n = c.id.clone();
        g.players[0].deck.push(c);
        Some(n)
    } else {
        None
    };
    let c = g.make_card("JZ58", 3);
    let other = c.id.clone();
    g.players[3].deck.push(c);
    let spell = held(&mut g, "JC091", 0);
    let (mut r, mut steps) = room(g);
    kill(&mut r, &mut steps, &spell, &s);
    assert_eq!(r.pending.as_ref().unwrap().seat, 0);
    if kind == "trigger-decline" {
        choose(&mut r, &mut steps, vec![]);
        drain(&mut r, &mut steps);
        assert!(r.players[0].hand.iter().all(|c| c.definition != "JZ58"));
        return result("death-named-search-trigger-decline", steps);
    }
    let id = r.pending.as_ref().unwrap().choice.options[0].id.clone();
    choose(&mut r, &mut steps, vec![id]);
    if kind == "empty" {
        drain(&mut r, &mut steps);
        assert!(r.players[0].hand.iter().all(|c| c.definition != "JZ58"));
        return result("death-named-search-empty-shuffles", steps);
    }
    advance(&mut r, &mut steps, |r| {
        r.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "search")
    });
    let p = r.pending.clone().unwrap();
    assert_eq!(p.seat, 0);
    assert_eq!(p.choice.options.len(), 1);
    assert_eq!(p.choice.options[0].id, n.clone().unwrap());
    for viewer in 0..4 {
        assert_eq!(
            r.view(viewer, r.pacing.last_server_now_ms)
                .pending_choice
                .is_some(),
            viewer == 0
        );
    }
    let before = r.random;
    choose(
        &mut r,
        &mut steps,
        if kind == "search-decline" {
            vec![]
        } else {
            vec![n.clone().unwrap()]
        },
    );
    drain(&mut r, &mut steps);
    assert_ne!(r.random, before);
    assert!(r.players[3].deck.iter().any(|c| c.id == other));
    if kind == "search-decline" {
        assert!(r.players[0].hand.iter().all(|c| c.definition != "JZ58"));
    } else {
        assert!(r.players[0]
            .hand
            .iter()
            .any(|c| c.definition == "JZ58" && c.id != n.clone().unwrap()));
        assert!(r
            .view(2, r.pacing.last_server_now_ms)
            .log
            .iter()
            .any(|l| l.text.contains("展示检索的 噩梦残像")));
    }
    result(&format!("real-death-named-search-{kind}"), steps)
}
fn amnesia(kind: &str) -> Value {
    let mut g = initial(kind);
    let seat = if kind == "teammate" {
        1
    } else if kind == "enemy" {
        2
    } else {
        0
    };
    let s = field(&mut g, "JZ58", seat);
    if kind == "hidden" {
        g.regions[0].cards.last_mut().unwrap().face_down = true;
    }
    fund(&mut g, "JZ67", 0, 2);
    let c = held(&mut g, "JZ67", 0);
    held(&mut g, "JC125", 2);
    let rescue = if kind == "source-return" {
        Some(field(&mut g, "JC075", 0))
    } else {
        None
    };
    let (mut r, mut steps) = room(g);
    apply_game(&mut r, &mut steps, 0, play(&c, "p2"));
    if let Some(rescue) = rescue {
        apply_game(&mut r, &mut steps, 0, activate(&rescue, "rescue", Some(&s)));
    }
    advance(&mut r, &mut steps, |r| {
        r.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "optional-shuffle")
            || r.pending.is_none() && r.stack.is_empty()
    });
    let eligible = matches!(kind, "accept" | "decline");
    assert_eq!(
        r.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "optional-shuffle"),
        eligible
    );
    if eligible {
        let before = r.random;
        let deck = r.players[2]
            .deck
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        choose(
            &mut r,
            &mut steps,
            if kind == "accept" {
                vec!["shuffle".into()]
            } else {
                vec![]
            },
        );
        drain(&mut r, &mut steps);
        if kind == "accept" {
            assert_ne!(r.random, before);
        } else {
            assert_eq!(r.random, before);
            assert_eq!(
                r.players[2]
                    .deck
                    .iter()
                    .map(|c| c.id.clone())
                    .collect::<Vec<_>>(),
                deck
            );
        }
    }
    result(&format!("real-amnesia-genuine-dream-{kind}"), steps)
}
fn natural_actions(seed: u64, accept: bool) -> Option<(deck::DeckDraft, Vec<(usize, Action)>)> {
    let mut d = deck::preset("watchers").unwrap();
    d.cards = vec![
        catalog::DeckEntry {
            card_id: "JZ58".into(),
            count: 3,
        },
        catalog::DeckEntry {
            card_id: "JZ61".into(),
            count: 3,
        },
        catalog::DeckEntry {
            card_id: "JZ67".into(),
            count: 3,
        },
        catalog::DeckEntry {
            card_id: "JC125".into(),
            count: 41,
        },
    ];
    let mut g = Game::new_with_deck(
        "natural-nightmare-amnesia".into(),
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
    while g.pending.is_some() {
        let (s, a) = pick_choice(&g);
        g.apply(s, a.clone()).unwrap();
        actions.push((s, a));
    }
    let nightmare = g.players[0]
        .hand
        .iter()
        .find(|c| c.definition == "JZ58")?
        .id
        .clone();
    let spell = g.players[0]
        .hand
        .iter()
        .find(|c| c.definition == "JZ67")?
        .id
        .clone();
    let payment = g.players[0]
        .hand
        .iter()
        .find(|c| c.id != nightmare && c.id != spell && catalog::card(&c.definition).color == "紫")?
        .id
        .clone();
    let (mut built, mut deployed, mut cast, mut offered) = (false, false, false, false);
    for _ in 0..500 {
        if offered && g.stack.is_empty() && g.pending.is_none() {
            assert!(g
                .regions
                .iter()
                .flat_map(|r| &r.cards)
                .any(|c| c.definition == "JZ58" && c.controller == 0 && !c.face_down));
            return Some((d, actions));
        }
        let (s, a) = if let Some(p) = g.pending.as_ref() {
            if p.choice.kind == "optional-shuffle" {
                assert_eq!(p.seat, 0);
                offered = true;
                (
                    0,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(if accept {
                            vec!["shuffle".into()]
                        } else {
                            vec![]
                        }),
                        ..Action::new("choose")
                    },
                )
            } else {
                pick_choice(&g)
            }
        } else if !built
            && g.legal_actions(0)
                .iter()
                .any(|a| a.action.kind == "asset" && a.action.card_id.as_deref() == Some(&payment))
        {
            built = true;
            (
                0,
                g.legal_actions(0)
                    .into_iter()
                    .find(|a| {
                        a.action.kind == "asset" && a.action.card_id.as_deref() == Some(&payment)
                    })
                    .unwrap()
                    .action,
            )
        } else if built
            && !deployed
            && g.legal_actions(0).iter().any(|a| {
                a.action.kind == "deploy" && a.action.card_id.as_deref() == Some(&nightmare)
            })
        {
            deployed = true;
            (
                0,
                g.legal_actions(0)
                    .into_iter()
                    .find(|a| {
                        a.action.kind == "deploy" && a.action.card_id.as_deref() == Some(&nightmare)
                    })
                    .unwrap()
                    .action,
            )
        } else if deployed
            && !cast
            && g.stack.is_empty()
            && g.legal_actions(0).iter().any(|a| {
                a.action.kind == "play"
                    && a.action.card_id.as_deref() == Some(&spell)
                    && a.action.target_id.as_deref() == Some("p2")
            })
        {
            cast = true;
            (
                0,
                g.legal_actions(0)
                    .into_iter()
                    .find(|a| {
                        a.action.kind == "play"
                            && a.action.card_id.as_deref() == Some(&spell)
                            && a.action.target_id.as_deref() == Some("p2")
                    })
                    .unwrap()
                    .action,
            )
        } else {
            pass_action(&g)
        };
        g.apply(s, a.clone()).unwrap();
        actions.push((s, a));
    }
    None
}
fn natural(accept: bool) -> Value {
    let (seed, d, actions) = (1..200)
        .find_map(|s| natural_actions(s, accept).map(|(d, a)| (s, d, a)))
        .expect("naturally dealt actual Dream Demon and Amnesia");
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "natural-nightmare-amnesia".into(),
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
    let mut offer = false;
    for (s, a) in actions {
        if r.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "optional-shuffle")
        {
            offer = true;
            assert_eq!(r.pending.as_ref().unwrap().seat, 0);
            assert!(r
                .regions
                .iter()
                .flat_map(|r| &r.cards)
                .any(|c| c.definition == "JZ58" && c.controller == 0 && !c.face_down));
        }
        let deploy = a.kind == "deploy";
        apply_game(&mut r, &mut steps, s, a);
        if deploy {
            let f = r.stack.last().unwrap().frame.as_ref().unwrap();
            assert_eq!(f.ability_key, "deploy");
            assert!(f.steps.is_empty());
            assert_eq!(resources(&r, 0), 1);
        }
    }
    assert!(offer);
    json!({"name":format!("dream-reveal-natural-real-50-card-joined-actual-jz58-deploy-amnesia-{}",if accept{"accept"}else{"decline"}),"seed":seed.to_string(),"syntheticInitialLayout":false,"newNaturalUiCoverage":false,"actualJoinedDealtGameNoStateInjection":true,"steps":steps})
}
pub fn cases() -> impl Iterator<Item = Value> {
    let mut list: Vec<Box<dyn FnOnce() -> Value>> = vec![];
    for k in ["advantage", "tie", "deficit", "attachment", "simultaneous"] {
        list.push(Box::new(move || spirit(k)));
    }
    list.push(Box::new(|| spirit_bypass(true)));
    list.push(Box::new(|| spirit_bypass(false)));
    list.push(Box::new(spirit_combat));
    for k in [
        "normal",
        "target3",
        "reject",
        "empty",
        "decline",
        "source-return",
    ] {
        list.push(Box::new(move || repression(k)));
    }
    for k in [
        "found",
        "controller",
        "empty",
        "trigger-decline",
        "search-decline",
    ] {
        list.push(Box::new(move || death_search(k)));
    }
    for k in [
        "accept",
        "decline",
        "hidden",
        "teammate",
        "enemy",
        "source-return",
    ] {
        list.push(Box::new(move || amnesia(k)));
    }
    list.push(Box::new(|| natural(true)));
    list.push(Box::new(|| natural(false)));
    list.into_iter().map(|f| f())
}
