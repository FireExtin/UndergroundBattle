//! Identity audit only. Boundary layouts are explicitly synthetic; the natural
//! four-seat red case joins, deals, builds assets and deploys through legal actions.
use super::*;
fn field(g: &mut Game, def: &str, owner: usize, controller: usize) -> String {
    let mut c = g.make_card(def, owner);
    c.controller = controller;
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn held(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn fund(g: &mut Game, def: &str, owner: usize, n: usize) {
    for _ in 0..n {
        let c = g.make_card(def, owner);
        g.players[owner].assets.push(c)
    }
}
fn initial(def: &str) -> (Game, Vec<String>) {
    let mut g = attachment_initial("teams", "identity-overlap");
    for p in &mut g.players {
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.influence = [0, 0];
    }
    let ids = (0..4).map(|s| field(&mut g, def, s, s)).collect();
    for def in ["JC042", "JC091", "JC036", "JC003", "JC075", "JC103"] {
        fund(&mut g, def, 0, 6);
    }
    for seat in 1..4 {
        fund(&mut g, "JC042", seat, 1);
    }
    (g, ids)
}
fn room(g: Game) -> (RoomEnvelope, Vec<Value>) {
    let r = RoomEnvelope::from_game(g);
    let steps = vec![step(
        &r,
        "initialFixture",
        json!([serde_json::to_string(&r).unwrap()]),
        0,
    )];
    (r, steps)
}
fn result(name: &str, steps: Vec<Value>) -> Value {
    json!({"name":format!("identity-overlap-{name}"),"seed":"9007199254740993","syntheticInitialLayout":true,"newNaturalUiCoverage":false,"steps":steps})
}
fn board<'a>(r: &'a RoomEnvelope, id: &str) -> Option<&'a hegemony_server::model::Card> {
    r.regions.iter().flat_map(|r| &r.cards).find(|c| c.id == id)
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
    )
}
fn pass(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    let (s, a) = (0..r.players.len())
        .find_map(|s| {
            r.game
                .legal_actions(s)
                .into_iter()
                .find(|a| a.action.kind == "pass")
                .map(|a| (s, a.action))
        })
        .expect("legal pass");
    apply_game(r, steps, s, a)
}
fn drain(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    for _ in 0..200 {
        if r.stack.is_empty() && r.pending.is_none() {
            return;
        }
        if r.pending.is_some() {
            let (s, a) = pick_choice(&r.game);
            apply_game(r, steps, s, a)
        } else {
            pass(r, steps)
        }
    }
    panic!("identity bounded drain")
}
fn advance(r: &mut RoomEnvelope, steps: &mut Vec<Value>, done: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..700 {
        if done(r) {
            return;
        }
        if r.pending.is_some() {
            let (s, a) = pick_choice(&r.game);
            apply_game(r, steps, s, a)
        } else {
            pass(r, steps)
        }
    }
    panic!("identity bounded advance")
}
fn reject(r: &mut RoomEnvelope, steps: &mut Vec<Value>, s: usize, a: Action) {
    let c = RoomCommand {
        command_id: format!("identity-reject:{}", steps.len()),
        expected_version: r.revision,
        action: a.into(),
    };
    let now = r.pacing.last_server_now_ms;
    let t = r.transition(s, Some(c.clone()), now).unwrap();
    assert_eq!(t.outcome, "rejected");
    assert!(!t.changed);
    record_transition(r, steps, "applyRoom", json!([s, c, now.to_string()]), t)
}
fn activate(source: &str, key: &str, target: &str) -> Action {
    Action {
        card_id: Some(source.into()),
        ability_id: Some(key.into()),
        target_id: Some(target.into()),
        ..Action::new("activate")
    }
}
fn play(source: &str, target: &str) -> Action {
    Action {
        card_id: Some(source.into()),
        target_id: Some(target.into()),
        ..Action::new("play")
    }
}
fn buff_damage() -> Value {
    let (mut g, ids) = initial("JC042");
    g.regions[0].cards[0].owner = 3;
    let source = field(&mut g, "LC12", 2, 0);
    let other = field(&mut g, "LC12", 1, 1);
    let blast = held(&mut g, "JC102", 0);
    let (mut r, mut steps) = room(g);
    for id in &ids[1..] {
        reject(
            &mut r,
            &mut steps,
            0,
            activate(&source, "protect-local-character", id),
        );
    }
    reject(
        &mut r,
        &mut steps,
        0,
        activate(&other, "protect-local-character", &ids[0]),
    );
    apply_game(
        &mut r,
        &mut steps,
        0,
        activate(&source, "protect-local-character", &ids[0]),
    );
    drain(&mut r, &mut steps);
    for (i, id) in ids.iter().enumerate() {
        let c = board(&r, id).unwrap();
        assert_eq!(r.game.defense(c, 0), if i == 0 { 2 } else { 1 });
        assert_eq!(c.owner, if i == 0 { 3 } else { i });
    }
    apply_game(&mut r, &mut steps, 0, play(&blast, &ids[0]));
    drain(&mut r, &mut steps);
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(board(&r, id).unwrap().damage, if i == 0 { 1 } else { 0 });
    }
    assert!(board(&r, &source).unwrap().exhausted);
    assert!(!board(&r, &other).unwrap().exhausted);
    result(
        "four-same-red-exact-controller-source-defense-and-damage",
        steps,
    )
}
fn exhaust() -> Value {
    let (mut g, ids) = initial("JC042");
    let source = field(&mut g, "JC002", 0, 0);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    let (mut r, mut steps) = room(g);
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
    choose(&mut r, &mut steps, vec![ids[2].clone()]);
    drain(&mut r, &mut steps);
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(board(&r, id).unwrap().exhausted, i == 2);
    }
    result("four-same-red-reveal-exhaust-exact-enemy-instance", steps)
}
fn wound() -> Value {
    let (mut g, ids) = initial("JC042");
    let source = field(&mut g, "JZ59", 1, 0);
    let murder = held(&mut g, "JC091", 0);
    let (mut r, mut steps) = room(g);
    apply_game(&mut r, &mut steps, 0, play(&murder, &source));
    advance(&mut r, &mut steps, |r| {
        r.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "trigger")
    });
    assert_eq!(r.pending.as_ref().unwrap().seat, 0);
    choose(&mut r, &mut steps, vec![ids[3].clone()]);
    drain(&mut r, &mut steps);
    assert!(board(&r, &ids[3]).is_none());
    for id in &ids[..3] {
        assert_eq!(board(&r, id).unwrap().wounds, 0);
    }
    assert!(r.players[3]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC042" && c.id != ids[3]));
    assert!(r.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "JZ59" && c.id != source));
    result("four-same-red-death-wound-only-selected-instance", steps)
}
fn control_return() -> Value {
    let (mut g, ids) = initial("JC042");
    let control = held(&mut g, "JC129", 0);
    let source = field(&mut g, "LC12", 0, 0);
    let rescue = field(&mut g, "JC075", 0, 0);
    let blast = held(&mut g, "JC102", 0);
    let (mut r, mut steps) = room(g);
    apply_game(&mut r, &mut steps, 0, play(&control, &ids[2]));
    drain(&mut r, &mut steps);
    for (i, id) in ids.iter().enumerate() {
        let c = board(&r, id).unwrap();
        assert_eq!(c.owner, i);
        assert_eq!(c.controller, if i == 2 { 0 } else { i });
    }
    apply_game(
        &mut r,
        &mut steps,
        0,
        activate(&source, "protect-local-character", &ids[2]),
    );
    drain(&mut r, &mut steps);
    assert_eq!(r.game.defense(board(&r, &ids[2]).unwrap(), 0), 2);
    apply_game(&mut r, &mut steps, 0, play(&blast, &ids[2]));
    apply_game(&mut r, &mut steps, 0, activate(&rescue, "rescue", &ids[2]));
    drain(&mut r, &mut steps);
    assert!(board(&r, &ids[2]).is_none());
    let hand = r.players[2]
        .hand
        .iter()
        .find(|c| c.definition == "JC042")
        .unwrap();
    assert_ne!(hand.id, ids[2]);
    assert_eq!(
        (hand.owner, hand.controller, hand.damage, hand.wounds),
        (2, 2, 0, 0)
    );
    let h = hand.id.clone();
    advance(&mut r, &mut steps, |r| {
        r.game
            .legal_actions(2)
            .iter()
            .any(|a| a.action.kind == "deploy" && a.action.card_id.as_deref() == Some(&h))
    });
    let a = r
        .game
        .legal_actions(2)
        .into_iter()
        .find(|a| a.action.kind == "deploy" && a.action.card_id.as_deref() == Some(&h))
        .unwrap()
        .action;
    apply_game(&mut r, &mut steps, 2, a);
    drain(&mut r, &mut steps);
    let fresh = r
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .find(|c| c.definition == "JC042" && c.owner == 2)
        .unwrap();
    assert_ne!(fresh.id, ids[2]);
    assert_ne!(fresh.id, h);
    assert_eq!(
        r.game.defense(
            fresh,
            r.regions
                .iter()
                .position(|region| region.cards.iter().any(|c| c.id == fresh.id))
                .unwrap()
        ),
        1
    );
    assert_eq!((fresh.controller, fresh.damage, fresh.wounds), (2, 0, 0));
    for id in [&ids[0], &ids[1], &ids[3]] {
        let c = board(&r, id).unwrap();
        assert_eq!((c.damage, c.wounds), (0, 0));
    }
    result(
        "real-control-same-red-owner-return-response-stale-target-fresh-redeploy",
        steps,
    )
}
fn simultaneous_death() -> Value {
    let (mut g, ids) = initial("XQ17");
    g.regions[0].cards[3].controller = 0;
    let blast = held(&mut g, "JC047", 0);
    let before = g.players.iter().map(|p| p.deck.len()).collect::<Vec<_>>();
    let (mut r, mut steps) = room(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(blast),
            region: Some(0),
            ..Action::new("play")
        },
    );
    let mut controllers = vec![];
    for _ in 0..160 {
        if r.stack.is_empty() && r.pending.is_none() {
            break;
        }
        if let Some(p) = r.pending.clone() {
            if p.choice.kind == "trigger" {
                controllers.push(p.seat);
                assert_eq!(p.choice.options.len(), 1);
                choose(&mut r, &mut steps, vec![p.choice.options[0].id.clone()]);
            } else {
                let (s, a) = pick_choice(&r.game);
                apply_game(&mut r, &mut steps, s, a)
            }
        } else {
            pass(&mut r, &mut steps)
        }
    }
    controllers.sort();
    assert_eq!(controllers, vec![0, 0, 1, 2]);
    for (s, n) in [2, 1, 1, 0].into_iter().enumerate() {
        assert_eq!(r.players[s].deck.len(), before[s] - n);
        assert_eq!(
            r.players[s]
                .graveyard
                .iter()
                .filter(|c| c.definition == "XQ17")
                .count(),
            1
        );
        assert!(r.players[s]
            .graveyard
            .iter()
            .filter(|c| c.definition == "XQ17")
            .all(|c| c.id != ids[s] && c.owner == s && c.controller == s));
    }
    result(
        "same-red-simultaneous-four-deaths-last-controller-and-owner-graves",
        steps,
    )
}
fn unique_scope() -> Value {
    let (mut g, _) = initial("JC042");
    let own = field(&mut g, "LC23", 0, 0);
    let ally = field(&mut g, "LC23", 1, 1);
    let enemy = field(&mut g, "LC23", 2, 2);
    let held = held(&mut g, "LC23", 0);
    let (mut r, mut steps) = room(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(held),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    advance(&mut r, &mut steps, |r| {
        r.pending
            .as_ref()
            .is_some_and(|p| p.choice.title.contains("独有"))
    });
    let p = r.pending.as_ref().unwrap();
    assert_eq!(p.seat, 0);
    assert_eq!(p.choice.options.len(), 2);
    assert!(p.choice.options.iter().any(|o| o.id == own));
    assert!(p
        .choice
        .options
        .iter()
        .all(|o| o.id != ally && o.id != enemy));
    choose(&mut r, &mut steps, vec![own.clone()]);
    drain(&mut r, &mut steps);
    assert!(board(&r, &own).is_none());
    assert!(board(&r, &ally).is_some());
    assert!(board(&r, &enemy).is_some());
    assert_eq!(
        r.regions[0]
            .cards
            .iter()
            .filter(|c| c.definition == "LC23" && c.controller == 0)
            .count(),
        1
    );
    result(
        "actual-unique-lc23-per-controller-not-color-team-or-opponent",
        steps,
    )
}
fn society_draft(id: &str) -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.society_id = Some(id.into());
    d.cards = vec![];
    if id == "MSJC01" {
        for cid in [
            "JC001", "JC002", "JC003", "JC004", "JC005", "JC006", "JC007", "XQ03",
        ] {
            d.cards.push(catalog::DeckEntry {
                card_id: cid.into(),
                count: 3,
            })
        }
        d.cards.push(catalog::DeckEntry {
            card_id: "JC008".into(),
            count: 1,
        });
        d.cards.push(catalog::DeckEntry {
            card_id: "JC125".into(),
            count: 25,
        });
    } else {
        d.cards.push(catalog::DeckEntry {
            card_id: "JC125".into(),
            count: 50,
        });
    }
    deck::validate(d).unwrap()
}
fn societies(mixed: bool) -> Value {
    let mut g = Game::new_with_deck(
        "identity-society".into(),
        "QA".into(),
        "teams".into(),
        "P0".into(),
        society_draft("MSJC09"),
        9007199254740993,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(
            format!("P{s}"),
            society_draft(if mixed && s >= 2 { "MSJC01" } else { "MSJC09" }),
        )
        .unwrap();
    }
    for s in 0..4 {
        g.apply(s, Action::new("ready")).unwrap();
    }
    g.apply(0, Action::new("start")).unwrap();
    while g.pending.is_some() {
        let p = g.pending.clone().unwrap();
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
    for p in &mut g.players {
        p.assets.clear();
    }
    g.first_team = 0;
    g.active_team = 0;
    g.priority_team = 0;
    g.window = Some(Window::Action(0));
    g.passed.clear();
    for s in 0..4 {
        fund(&mut g, "JC042", s, 1);
        fund(&mut g, if s < 2 { "JC091" } else { "JC036" }, s, 2);
    }
    let ids = g
        .players
        .iter()
        .map(|p| p.society_zone.card.as_ref().unwrap().id.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        ids.iter().collect::<std::collections::BTreeSet<_>>().len(),
        4
    );
    let (mut r, mut steps) = room(g);
    let a = |s: usize| Action {
        card_id: Some(ids[s].clone()),
        ability_id: Some("drawWithInitiative".into()),
        ..Action::new("activate")
    };
    for s in 1..4 {
        reject(&mut r, &mut steps, 0, a(s));
    }
    let legal = r
        .game
        .legal_actions(0)
        .into_iter()
        .find(|l| l.action == a(0))
        .unwrap();
    assert_eq!(legal.source_zone_id.as_deref(), Some("society:p0"));
    let before = r.players.iter().map(|p| p.hand.len()).collect::<Vec<_>>();
    apply_game(&mut r, &mut steps, 0, a(0));
    drain(&mut r, &mut steps);
    assert_eq!(r.players[0].hand.len(), before[0] + 1);
    for s in 1..4 {
        assert_eq!(r.players[s].hand.len(), before[s]);
        assert!(!r.players[s].society_zone.card.as_ref().unwrap().exhausted);
        assert!(r.players[s].assets.iter().all(|c| !c.exhausted));
    }
    assert!(r.players[0].assets.iter().all(|c| c.exhausted));
    for viewer in 0..4 {
        let v = r.view(viewer, r.pacing.last_server_now_ms);
        for (s, z) in v.society_zones.iter().enumerate() {
            let c = z.card.as_ref().unwrap();
            assert_eq!(z.id, format!("society:p{s}"));
            assert_eq!(c.instance_id, ids[s]);
            assert_eq!(
                (&c.owner, &c.controller),
                (&format!("p{s}"), &format!("p{s}"))
            );
        }
    }
    result(
        if mixed {
            "different-real-societies-common-red-assets-personal-payment"
        } else {
            "four-same-real-society-exact-instances-source-authority-and-payment"
        },
        steps,
    )
}
fn red_draft() -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.cards = [
        ("JC042", 3),
        ("XQ17", 3),
        ("JC047", 3),
        ("JC049", 3),
        ("JC125", 38),
    ]
    .into_iter()
    .map(|(id, count)| catalog::DeckEntry {
        card_id: id.into(),
        count,
    })
    .collect();
    deck::validate(d).unwrap()
}
fn natural_plan(seed: u64) -> Option<(Vec<(usize, Action)>, Vec<String>)> {
    let d = red_draft();
    let mut g = Game::new_with_deck(
        "natural-four-red".into(),
        "QA".into(),
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
    while let Some(p) = g.pending.clone() {
        let a = Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![]),
            ..Action::new("choose")
        };
        g.apply(p.seat, a.clone()).unwrap();
        actions.push((p.seat, a));
    }
    let mut characters = vec![];
    let mut payments = vec![];
    for s in 0..4 {
        let c = g.players[s].hand.iter().find(|c| c.definition == "XQ17")?;
        characters.push(c.id.clone());
        let a = g.players[s]
            .hand
            .iter()
            .find(|a| a.id != c.id && catalog::card(&a.definition).color == "红")?;
        payments.push(a.id.clone());
    }
    let mut built = [false; 4];
    let mut deployed = [false; 4];
    for _ in 0..500 {
        if deployed.iter().all(|x| *x) && g.stack.is_empty() && g.pending.is_none() {
            let ids = (0..4)
                .map(|s| {
                    g.regions
                        .iter()
                        .flat_map(|r| &r.cards)
                        .find(|c| c.definition == "XQ17" && c.controller == s && !c.face_down)
                        .unwrap()
                        .id
                        .clone()
                })
                .collect();
            return Some((actions, ids));
        }
        let next = if g.pending.is_some() {
            Some(pick_choice(&g))
        } else {
            (0..4).find_map(|s| {
                let legal = g.legal_actions(s);
                let desired = if !built[s] {
                    "asset"
                } else if !deployed[s] {
                    "deploy"
                } else {
                    return None;
                };
                let id = if !built[s] {
                    &payments[s]
                } else {
                    &characters[s]
                };
                legal
                    .into_iter()
                    .find(|a| a.action.kind == desired && a.action.card_id.as_ref() == Some(id))
                    .map(|a| {
                        if desired == "asset" {
                            built[s] = true
                        } else {
                            deployed[s] = true
                        }
                        (s, a.action)
                    })
            })
        };
        let (s, a) = next.unwrap_or_else(|| {
            (0..4)
                .find_map(|s| {
                    g.legal_actions(s)
                        .into_iter()
                        .find(|a| a.action.kind == "pass")
                        .map(|a| (s, a.action))
                })
                .unwrap()
        });
        g.apply(s, a.clone()).unwrap();
        actions.push((s, a));
    }
    None
}
fn natural() -> Value {
    let (seed, actions, expected) = (1..2000)
        .find_map(|s| natural_plan(s).map(|(a, i)| (s, a, i)))
        .expect("naturally dealt four same red characters and own red assets");
    let d = red_draft();
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "natural-four-red".into(),
            "QA".into(),
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
    for (s, a) in actions {
        apply_game(&mut r, &mut steps, s, a);
    }
    assert_eq!(
        expected
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    for s in 0..4 {
        let c = board(&r, &expected[s]).unwrap();
        assert_eq!((c.owner, c.controller), (s, s));
        assert_eq!(c.definition, "XQ17");
    }
    println!("natural four same red seed={seed}");
    json!({"name":"identity-overlap-natural-four-seat-same-red-real-deal-assets-deploy","seed":seed.to_string(),"syntheticInitialLayout":false,"actualJoinedDealtGameNoStateInjection":true,"newNaturalUiCoverage":false,"steps":steps})
}
pub fn cases() -> impl Iterator<Item = Value> {
    [
        buff_damage(),
        exhaust(),
        wound(),
        control_return(),
        simultaneous_death(),
        unique_scope(),
        societies(false),
        societies(true),
        natural(),
    ]
    .into_iter()
}
