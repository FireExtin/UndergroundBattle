//! Disclosed boundary layouts; every later transition uses real Room commands.
//! Marked response checkpoints are explicit synthetic continuations, never natural UI.
use super::*;
fn initial(name: &str) -> Game {
    let mut g = attachment_initial("teams", &format!("repress-assets-{name}"));
    for p in &mut g.players {
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.influence = [0, 0];
    }
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
    json!({"name":format!("repress-assets-{name}"),"seed":"9007199254740993","syntheticInitialLayout":true,"newNaturalUiCoverage":false,"steps":steps})
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
    panic!("bounded repress/assets drain")
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
    panic!("bounded repress/assets advance")
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

fn hand_play(id: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        ..Action::new("play")
    }
}
fn action_window(g: &mut Game, seat: usize) {
    let team = usize::from(seat >= 2);
    g.window = Some(Window::Action(team));
    g.active_team = team;
    g.priority_team = team;
    g.passed.clear();
    g.team_passed = [false; 2];
}
fn accident(actor: usize, kind: &str) -> Value {
    let mut g = initial(kind);
    action_window(&mut g, actor);
    let enemy = if actor < 2 { 2 } else { 0 };
    let team = usize::from(enemy >= 2);
    fund(&mut g, "JC125", actor, 2);
    let c = held(&mut g, "JZ74", actor);
    if kind != "empty-influence" && kind != "empty-deck" {
        g.regions[2].influence[team] = 2;
        g.regions[4].influence[team] = 1;
    }
    if kind == "empty-deck" {
        g.players[actor].deck.clear();
    }
    let top = g.players[actor].deck.first().cloned();
    let hand_len = g.players[actor].hand.len() - 1;
    let (mut r, mut steps) = room(g);
    if kind == "reject" {
        for id in [format!("p{actor}"), format!("p{}", actor ^ 1), "p8".into()] {
            reject(&mut r, &mut steps, actor, play(&c, &id));
        }
        reject(&mut r, &mut steps, actor, hand_play(&c));
    }
    apply_game(&mut r, &mut steps, actor, play(&c, &format!("p{enemy}")));
    assert_eq!(resources(&r, actor), 1);
    if kind == "empty-influence" || kind == "empty-deck" {
        drain(&mut r, &mut steps);
    } else {
        advance(&mut r, &mut steps, |r| {
            r.pending
                .as_ref()
                .is_some_and(|p| p.choice.kind == "target")
        });
        let p = r.pending.clone().unwrap();
        assert_eq!(p.seat, enemy);
        assert_eq!(r.players[actor].hand.len(), hand_len);
        assert_eq!(r.players[actor].deck[0].id, top.as_ref().unwrap().id);
        for s in 0..4 {
            assert_eq!(
                r.view(s, r.pacing.last_server_now_ms)
                    .pending_choice
                    .is_some(),
                s == enemy
            );
        }
        if kind == "reject" {
            for s in [actor, enemy ^ 1] {
                reject(
                    &mut r,
                    &mut steps,
                    s,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(vec!["region:4".into()]),
                        ..Action::new("choose")
                    },
                );
            }
            for ids in [
                vec![],
                vec!["region:0".into()],
                vec!["region:2".into(), "region:4".into()],
            ] {
                reject(
                    &mut r,
                    &mut steps,
                    enemy,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(ids),
                        ..Action::new("choose")
                    },
                );
            }
        }
        choose(&mut r, &mut steps, vec!["region:4".into()]);
        drain(&mut r, &mut steps);
        assert_eq!(r.regions[2].influence[team], 2);
        assert_eq!(r.regions[4].influence[team], 0);
    }
    if let Some(top) = top {
        assert_eq!(r.players[actor].hand.len(), hand_len + 1);
        assert_eq!(
            r.players[actor].hand.last().unwrap().definition,
            top.definition
        );
        assert_ne!(r.players[actor].hand.last().unwrap().id, top.id);
        assert_eq!(resources(&r, actor), 1);
        assert!(r.players[actor]
            .graveyard
            .iter()
            .any(|c| c.definition == "JZ74"));
    } else {
        assert!(r.players[actor].eliminated);
    }
    result(&format!("accident-seat{actor}-{kind}"), steps)
}
fn expansion(actor: usize, top_def: &str, kind: &str) -> Value {
    let mut g = initial(kind);
    action_window(&mut g, actor);
    fund(&mut g, "JC036", actor, 6);
    let c = held(&mut g, "JC126", actor);
    let h = held(&mut g, "JC125", actor);
    let top = g.make_card(top_def, actor);
    let old = top.id.clone();
    g.players[actor].deck.insert(0, top);
    if kind == "empty" {
        g.players[actor].deck.clear();
    }
    if kind == "dirty-boundary" {
        let top = &mut g.players[actor].deck[0];
        top.face_down = true;
        top.exhausted = true;
        top.damage = 7;
        top.wounds = 4;
        top.shield = 2;
    }
    let (mut r, mut steps) = room(g);
    if kind == "prior-build" {
        apply_game(
            &mut r,
            &mut steps,
            actor,
            Action {
                card_id: Some(h.clone()),
                ..Action::new("asset")
            },
        );
    }
    let before = r.players[actor].assets.len();
    let deck = r.players[actor].deck.len();
    apply_game(&mut r, &mut steps, actor, hand_play(&c));
    assert_eq!(resources(&r, actor), 3 + usize::from(kind == "prior-build"));
    drain(&mut r, &mut steps);
    assert_eq!(r.players[actor].asset_used, kind == "prior-build");
    assert_eq!(
        r.players[actor].hand.len(),
        usize::from(kind != "prior-build")
    );
    if kind == "empty" {
        assert_eq!(r.players[actor].assets.len(), before);
        assert!(!r.players[actor].eliminated);
    } else {
        assert_eq!(r.players[actor].deck.len(), deck - 1);
        let asset = r.players[actor].assets.last().unwrap();
        assert_eq!(asset.definition, top_def);
        assert_ne!(asset.id, old);
        assert_eq!(
            (
                asset.owner,
                asset.controller,
                asset.face_down,
                asset.exhausted,
                asset.damage,
                asset.wounds,
                asset.shield
            ),
            (actor, actor, false, false, 0, 0, 0)
        );
        for s in 0..4 {
            let view = r.view(s, r.pacing.last_server_now_ms);
            let a = view
                .assets
                .iter()
                .find(|v| v.instance_id == asset.id)
                .unwrap();
            assert_eq!(a.kind, "asset");
            assert!(
                a.card_id.is_none()
                    && a.text.is_none()
                    && a.icons.is_none()
                    && a.defense.is_none()
                    && a.current_spirit_protection.is_none()
            );
            assert_eq!(
                a.color.as_deref(),
                Some(catalog::card(top_def).color.as_str())
            );
            assert_eq!(
                a.magic.as_deref(),
                Some(catalog::card(top_def).magic.as_str())
            );
        }
        if kind == "prior-build" {
            reject(
                &mut r,
                &mut steps,
                actor,
                Action {
                    card_id: Some(h.clone()),
                    ..Action::new("asset")
                },
            );
        }
        if kind == "later-build" {
            apply_game(
                &mut r,
                &mut steps,
                actor,
                Action {
                    card_id: Some(h),
                    ..Action::new("asset")
                },
            );
        }
    }
    assert!(r.players[actor]
        .graveyard
        .iter()
        .any(|grave| grave.definition == "JC126" && grave.id != c));
    result(&format!("expansion-seat{actor}-{top_def}-{kind}"), steps)
}

fn expansion_repeated() -> Value {
    let mut g = initial("repeat");
    fund(&mut g, "JC036", 0, 5);
    let a = held(&mut g, "JC126", 0);
    let b = held(&mut g, "JC126", 0);
    let top_ids = g.players[0]
        .deck
        .iter()
        .take(2)
        .map(|c| c.id.clone())
        .collect::<Vec<_>>();
    let (mut r, mut steps) = room(g);
    for c in [a, b] {
        apply_game(&mut r, &mut steps, 0, hand_play(&c));
        drain(&mut r, &mut steps);
        assert!(!r.players[0].asset_used);
    }
    assert_eq!(r.players[0].assets.len(), 7);
    assert_eq!(resources(&r, 0), 1);
    assert!(r.players[0].deck.iter().all(|c| !top_ids.contains(&c.id)));
    assert_ne!(r.players[0].assets[5].id, r.players[0].assets[6].id);
    result("two-paid-expansions-one-turn-two-fresh-assets", steps)
}
fn expansion_forecast() -> Value {
    let mut g = initial("forecast");
    fund(&mut g, "JC036", 0, 5);
    fund(&mut g, "JC104", 0, 1);
    let c = held(&mut g, "JC126", 0);
    let hidden = field(&mut g, "JC104", 0);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    let first = g.players[0].deck[0].id.clone();
    let second = g.players[0].deck[1].clone();
    let (mut r, mut steps) = room(g);
    apply_game(&mut r, &mut steps, 0, hand_play(&c));
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    advance(&mut r, &mut steps, |r| {
        r.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "trigger")
    });
    choose(&mut r, &mut steps, vec!["accept".into()]);
    advance(&mut r, &mut steps, |r| {
        r.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "investigation")
    });
    let p = r.pending.clone().unwrap();
    assert_eq!(p.seat, 0);
    for s in 1..4 {
        assert!(r
            .view(s, r.pacing.last_server_now_ms)
            .pending_choice
            .is_none());
    }
    let bottom = p
        .choice
        .options
        .iter()
        .filter(|o| o.id != second.id)
        .map(|o| o.id.clone())
        .collect();
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            choice_id: Some(p.choice.id),
            top: Some(vec![second.id.clone()]),
            bottom: Some(bottom),
            ..Action::new("choose")
        },
    );
    drain(&mut r, &mut steps);
    let asset = r.players[0].assets.last().unwrap();
    assert_eq!(asset.definition, second.definition);
    assert_ne!(asset.id, second.id);
    assert!(r.players[0].deck.iter().any(|c| c.id == first));
    assert!(!r.players[0].deck.iter().any(|c| c.id == second.id));
    assert_eq!(resources(&r, 0), 2);
    result(
        "paid-jc104-response-private-forecast-changes-actual-top",
        steps,
    )
}
fn expansion_requirements() -> Value {
    let mut g = initial("requirements");
    fund(&mut g, "JC125", 0, 3);
    fund(&mut g, "JC036", 1, 4);
    let c = held(&mut g, "JC126", 0);
    let (mut r, mut steps) = room(g);
    reject(&mut r, &mut steps, 0, hand_play(&c));
    result(
        "neutral-expansion-blue-loyalty-not-supplied-by-teammate",
        steps,
    )
}
fn accident_guard() -> Value {
    let mut g = initial("guard");
    fund(&mut g, "JC125", 0, 2);
    fund(&mut g, "JC125", 2, 1);
    let a = held(&mut g, "JZ74", 0);
    let b = held(&mut g, "JZ74", 2);
    g.players[2].deck.clear();
    let (mut r, mut steps) = room(g);
    apply_game(&mut r, &mut steps, 0, play(&a, "p2"));
    pass(&mut r, &mut steps);
    pass(&mut r, &mut steps);
    apply_game(&mut r, &mut steps, 2, play(&b, "p0"));
    drain(&mut r, &mut steps);
    assert!(r.players[2].eliminated && r.players[0].hand.is_empty());
    assert_eq!(resources(&r, 0), 1);
    assert!(r
        .game
        .view(0)
        .log
        .iter()
        .any(|l| l.text.contains("效果取消")));
    result(
        "real-target-empty-deck-draw-elimination-cancels-whole-program",
        steps,
    )
}
fn expansion_payment_and_domain() -> Value {
    let mut g = initial("asset-payment-domain");
    fund(&mut g, "JC036", 0, 4);
    fund(&mut g, "JC125", 0, 2);
    // Two ordinary currency assets start exhausted; the new asset is
    // one of the two remaining currency assets for the real two-cost JC008.
    g.players[0].assets[4].exhausted = true;
    g.players[0].assets[5].exhausted = true;
    let top = g.make_card("JC008", 0);
    g.players[0].deck.insert(0, top);
    let own = field(&mut g, "JC125", 0);
    let enemy_human = field(&mut g, "JC125", 2);
    let attachment = g.make_card("BQ022", 2);
    let target = attachment.id.clone();
    g.attachments.push(hegemony_server::model::Attachment {
        card: attachment,
        host_id: enemy_human,
    });
    let c = held(&mut g, "JC126", 0);
    let buff = held(&mut g, "JC008", 0);
    let destroy = held(&mut g, "JC005", 0);
    let (mut r, mut steps) = room(g);
    apply_game(&mut r, &mut steps, 0, hand_play(&c));
    drain(&mut r, &mut steps);
    assert_eq!(resources(&r, 0), 2);
    apply_game(&mut r, &mut steps, 0, play(&buff, &own));
    drain(&mut r, &mut steps);
    assert_eq!(resources(&r, 0), 0);
    assert!(r.players[0].assets.last().unwrap().exhausted);
    // A normal next round refresh supplies currency; no test state edit.
    advance(&mut r, &mut steps, |r| {
        r.game.legal_actions(0).iter().any(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&destroy)
                && a.action.target_id.as_deref() == Some(&target)
        })
    });
    apply_game(&mut r, &mut steps, 0, play(&destroy, &target));
    drain(&mut r, &mut steps);
    assert!(r.attachments.is_empty());
    result(
        "new-asset-pays-real-two-cost-spell-and-supplies-yellow-loyalty-public-mind",
        steps,
    )
}

fn natural_actions(seed: u64) -> Option<(deck::DeckDraft, Vec<(usize, Action)>)> {
    let mut d = deck::preset("watchers").unwrap();
    d.cards = vec![
        catalog::DeckEntry {
            card_id: "JZ74".into(),
            count: 3,
        },
        catalog::DeckEntry {
            card_id: "JC126".into(),
            count: 3,
        },
        catalog::DeckEntry {
            card_id: "XQ16".into(),
            count: 3,
        },
        catalog::DeckEntry {
            card_id: "JC125".into(),
            count: 41,
        },
    ];
    let mut g = Game::new_with_deck(
        "natural-repress-assets".into(),
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
    let a = g.players[0]
        .hand
        .iter()
        .find(|c| c.definition == "JZ74")?
        .id
        .clone();
    let b = g.players[0]
        .hand
        .iter()
        .find(|c| c.definition == "JC126")?
        .id
        .clone();
    if !g.players[0].hand.iter().any(|c| c.definition == "XQ16") {
        return None;
    }
    let (mut accident, mut expansion) = (false, false);
    for _ in 0..900 {
        if accident && expansion && g.stack.is_empty() && g.pending.is_none() {
            assert!(g.players[0].assets.len() >= 4);
            return Some((d, actions));
        }
        let actions0 = g.legal_actions(0);
        let next = if g.pending.is_some() {
            pick_choice(&g)
        } else if !accident
            && actions0.iter().any(|l| {
                l.action.kind == "play"
                    && l.action.card_id.as_deref() == Some(&a)
                    && l.action.target_id.as_deref() == Some("p2")
            })
        {
            accident = true;
            (
                0,
                actions0
                    .iter()
                    .find(|l| {
                        l.action.kind == "play"
                            && l.action.card_id.as_deref() == Some(&a)
                            && l.action.target_id.as_deref() == Some("p2")
                    })
                    .unwrap()
                    .action
                    .clone(),
            )
        } else if !expansion
            && actions0
                .iter()
                .any(|l| l.action.kind == "play" && l.action.card_id.as_deref() == Some(&b))
        {
            expansion = true;
            (
                0,
                actions0
                    .iter()
                    .find(|l| l.action.kind == "play" && l.action.card_id.as_deref() == Some(&b))
                    .unwrap()
                    .action
                    .clone(),
            )
        } else if !expansion {
            let blue = g.players[0]
                .assets
                .iter()
                .any(|c| catalog::card(&c.definition).color == "蓝");
            let build = actions0.iter().find(|l| {
                l.action.kind == "asset"
                    && l.action.card_id.as_ref().is_some_and(|id| {
                        g.players[0].hand.iter().any(|c| {
                            c.id == *id
                                && c.id != a
                                && c.id != b
                                && if blue {
                                    c.definition == "JC125"
                                } else {
                                    c.definition == "XQ16"
                                }
                        })
                    })
            });
            if let Some(build) = build {
                (0, build.action.clone())
            } else {
                pass_action(&g)
            }
        } else {
            pass_action(&g)
        };
        g.apply(next.0, next.1.clone()).unwrap();
        actions.push(next);
    }
    None
}
fn natural() -> Value {
    let (seed, d, actions) = (1..200)
        .find_map(|s| natural_actions(s).map(|(d, a)| (s, d, a)))
        .expect("naturally dealt real JZ74/JC126 and blue asset");
    let mut r = RoomEnvelope::from_game(
        Game::new_with_deck(
            "natural-repress-assets".into(),
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
    for (s, a) in actions {
        apply_game(&mut r, &mut steps, s, a);
    }
    assert!(r.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JZ74"));
    assert!(r.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC126"));
    json!({"name":"repress-assets-natural-real-50-card-three-normal-build-turns-both-new-spells","seed":seed.to_string(),"syntheticInitialLayout":false,"newNaturalUiCoverage":false,"actualJoinedDealtGameNoStateInjection":true,"steps":steps})
}
pub fn cases() -> impl Iterator<Item = Value> {
    let mut cases = vec![];
    for actor in 0..4 {
        cases.push(accident(actor, "choose"));
    }
    for kind in ["empty-influence", "empty-deck", "reject"] {
        cases.push(accident(0, kind));
    }
    for (actor, def) in [(0, "JC104"), (1, "JZ58"), (2, "LC23"), (3, "JC126")] {
        cases.push(expansion(actor, def, "normal"));
    }
    for kind in ["prior-build", "later-build", "empty", "dirty-boundary"] {
        cases.push(expansion(0, "JC104", kind));
    }
    cases.push(expansion_repeated());
    cases.push(expansion_forecast());
    cases.push(expansion_requirements());
    cases.push(accident_guard());
    cases.push(expansion_payment_and_domain());
    cases.push(natural());
    cases.into_iter()
}
