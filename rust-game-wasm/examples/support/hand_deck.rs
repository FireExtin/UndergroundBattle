//! Explicit boundary layouts/checkpoints; all recorded commands use Room ABI.
//! The separate natural case starts with legal decks and real dealt cards.
use super::*;
use hegemony_server::model::Card;

fn field(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn held(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn fund(g: &mut Game, id: &str, seat: usize, n: usize) {
    for _ in 0..n {
        let c = g.make_card(id, seat);
        g.players[seat].assets.push(c);
    }
}
fn board<'a>(g: &'a Game, id: &str) -> Option<(usize, &'a Card)> {
    g.regions
        .iter()
        .enumerate()
        .find_map(|(r, x)| x.cards.iter().find(|c| c.id == id).map(|c| (r, c)))
}
fn resources(g: &Game, s: usize) -> usize {
    g.players[s].assets.iter().filter(|c| !c.exhausted).count()
}
fn action(id: &str, key: &str, target: Option<&str>, sacrifice: Option<&str>) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(key.into()),
        target_id: target.map(str::to_string),
        cost_selected: sacrifice.map(|id| vec![id.into()]),
        ..Action::new("activate")
    }
}
fn pass_action(g: &Game) -> (usize, Action) {
    (0..4)
        .find_map(|s| {
            g.legal_actions(s)
                .into_iter()
                .find(|a| a.action.kind == "pass")
                .map(|a| (s, a.action))
        })
        .unwrap()
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
fn declaration(id: &str, definition: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        region: Some(0),
        ..Action::new(if catalog::card(definition).kind == "character" {
            "deploy"
        } else {
            "play"
        })
    }
}
fn set_deck(g: &mut Game, seat: usize, ids: &[&str]) -> Vec<String> {
    let cards = ids
        .iter()
        .map(|id| g.make_card(id, seat))
        .collect::<Vec<_>>();
    let old = cards.iter().map(|c| c.id.clone()).collect();
    g.players[seat].deck = cards;
    old
}
fn drain(r: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    for _ in 0..100 {
        if r.stack.is_empty() && r.pending.is_none() {
            return;
        }
        let (s, a) = if r.pending.is_some() {
            pick_choice(&r.game)
        } else {
            pass_action(&r.game)
        };
        apply_game(r, steps, s, a);
    }
    panic!("bounded hand/deck drain");
}
fn until_choice(r: &mut RoomEnvelope, steps: &mut Vec<Value>, kind: &str) {
    for _ in 0..100 {
        if r.pending.as_ref().is_some_and(|p| p.choice.kind == kind) {
            return;
        }
        let (s, a) = if r.pending.is_some() {
            pick_choice(&r.game)
        } else {
            pass_action(&r.game)
        };
        apply_game(r, steps, s, a);
    }
    panic!("missing hand/deck {kind}");
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
fn privacy(r: &RoomEnvelope, owner: usize) {
    for seat in 0..4 {
        if seat != owner {
            assert!(r.game.view(seat).pending_choice.is_none());
            assert!(!serde_json::to_string(&r.game.view(seat))
                .unwrap()
                .contains("西比尔"));
        }
    }
}
fn boundary(kind: &str) -> Value {
    let mut g = attachment_initial("teams", "888888888888888888880150");
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.regions[0].influence = [0, 0];
    for p in &mut g.players {
        p.graveyard.clear();
    }
    for id in ["JC075", "JC014", "JC084", "JC042"] {
        fund(&mut g, id, 0, 4);
    }
    let old = set_deck(
        &mut g,
        0,
        &["LC01", "JC003", "JC125", "JC125", "JC125", "JC125"],
    );
    let research = held(&mut g, "JC130", 0);
    let supplies = held(&mut g, "JC131", 0);
    let dog = field(&mut g, "JC096", if kind == "source-return" { 2 } else { 0 });
    if kind == "source-return" {
        g.regions[0]
            .cards
            .iter_mut()
            .find(|c| c.id == dog)
            .unwrap()
            .controller = 0;
    }
    let victim = field(&mut g, "JC125", 0);
    let teammate = field(&mut g, "JC125", 1);
    let enemy = field(&mut g, "JC125", 2);
    let hidden = field(&mut g, "JC125", 0);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    let offering = field(&mut g, "XQ17", 2);
    g.regions[0]
        .cards
        .iter_mut()
        .find(|c| c.id == offering)
        .unwrap()
        .controller = 0;
    if kind == "empty-search" {
        set_deck(&mut g, 0, &[]);
    }
    if kind == "research-depletion" {
        set_deck(&mut g, 0, &["JC125"]);
    }
    if kind == "controlled-not-owned" {
        g.regions[0]
            .cards
            .iter_mut()
            .find(|c| c.id == enemy)
            .unwrap()
            .controller = 0;
    }
    if kind == "loyalty" {
        g.players[0].assets.clear();
        fund(&mut g, "JC125", 0, 8);
    }
    let rescuer = if kind == "source-return" {
        Some(field(&mut g, "JC075", 0))
    } else {
        None
    };
    let mut r = RoomEnvelope::from_game(g);
    let initial = serde_json::to_string(&r).unwrap();
    assert_eq!(
        serde_json::to_string(&RoomEnvelope::from_persisted(&initial).unwrap()).unwrap(),
        initial
    );
    let mut steps = vec![step(&r, "initialFixture", json!([initial]), 0)];
    let mut milestones = vec![];
    match kind {
        "research-private" => {
            let before = resources(&r.game, 0);
            apply_game(&mut r, &mut steps, 0, declaration(&research, "JC130"));
            assert_eq!(resources(&r.game, 0), before - 2);
            until_choice(&mut r, &mut steps, "discard");
            privacy(&r, 0);
            assert_eq!(r.players[0].deck.len(), 4);
            assert!(r.players[0]
                .hand
                .iter()
                .filter(|c| ["LC01", "JC003"].contains(&c.definition.as_str()))
                .all(|c| !old.contains(&c.id)));
            let p = r.pending.clone().unwrap();
            let id = r.players[0]
                .hand
                .iter()
                .find(|c| c.definition == "LC01")
                .unwrap()
                .id
                .clone();
            for ids in [vec![], vec![id.clone(), id.clone()], vec![old[0].clone()]] {
                reject(
                    &mut r,
                    &mut steps,
                    0,
                    Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(ids),
                        ..Action::new("choose")
                    },
                );
            }
            reject(
                &mut r,
                &mut steps,
                1,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec![id.clone()]),
                    ..Action::new("choose")
                },
            );
            choose(&mut r, &mut steps, vec![id.clone()]);
            drain(&mut r, &mut steps);
            assert!(r.players[0]
                .graveyard
                .iter()
                .any(|c| c.definition == "LC01" && c.id != id));
            milestones.push(json!({"kind":"two-fresh-draws-before-exact-one-private-discard-restore-and-atomic-rejections","step":steps.len()-1}));
        }
        "airdrop-private" => {
            let before = resources(&r.game, 0);
            apply_game(&mut r, &mut steps, 0, declaration(&supplies, "JC131"));
            assert_eq!(resources(&r.game, 0), before - 3);
            until_choice(&mut r, &mut steps, "search");
            privacy(&r, 0);
            let p = r.pending.clone().unwrap();
            assert_eq!(p.choice.options.len(), 6);
            reject(
                &mut r,
                &mut steps,
                0,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec![]),
                    ..Action::new("choose")
                },
            );
            reject(
                &mut r,
                &mut steps,
                2,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec![old[0].clone()]),
                    ..Action::new("choose")
                },
            );
            let random = r.random;
            choose(&mut r, &mut steps, vec![old[0].clone()]);
            drain(&mut r, &mut steps);
            assert_ne!(r.random, random);
            assert!(r.players[0]
                .hand
                .iter()
                .any(|c| c.definition == "LC01" && c.id != old[0]));
            privacy(&r, 0);
            assert!(!r
                .log
                .iter()
                .any(|e| e.text.contains("西比尔") || e.text.contains("展示检索")));
            milestones.push(json!({"kind":"private-full-deck-search-fresh-hand-and-shuffle","step":steps.len()-1}));
        }
        "empty-search" => {
            apply_game(&mut r, &mut steps, 0, declaration(&supplies, "JC131"));
            drain(&mut r, &mut steps);
            assert!(!r.players[0].eliminated);
            assert!(r.pending.is_none());
        }
        "research-depletion" => {
            apply_game(&mut r, &mut steps, 0, declaration(&research, "JC130"));
            drain(&mut r, &mut steps);
            assert!(r.players[0].eliminated);
            assert!(r.players[0].hand.is_empty());
        }
        "dog-invalid" => {
            for cost in [
                None,
                Some(vec![teammate]),
                Some(vec![enemy]),
                Some(vec![hidden]),
                Some(vec![r.players[0].assets[0].id.clone()]),
                Some(vec![victim.clone(), victim.clone()]),
                Some(vec!["stale-instance".into()]),
            ] {
                let mut a = action(&dog, "sacrifice-draw", None, None);
                a.cost_selected = cost;
                reject(&mut r, &mut steps, 0, a);
            }
            let before = resources(&r.game, 0);
            apply_game(
                &mut r,
                &mut steps,
                0,
                action(&dog, "sacrifice-draw", None, Some(&victim)),
            );
            assert_eq!(resources(&r.game, 0), before - 1);
            assert!(board(&r.game, &dog).unwrap().1.exhausted);
            drain(&mut r, &mut steps);
            reject(
                &mut r,
                &mut steps,
                0,
                action(&dog, "sacrifice-draw", None, Some(&dog)),
            );
        }
        "dog-self" | "controlled-not-owned" => {
            let v = if kind == "dog-self" { &dog } else { &enemy };
            let owner = if kind == "dog-self" { 0 } else { 2 };
            let before = resources(&r.game, 0);
            apply_game(
                &mut r,
                &mut steps,
                0,
                action(&dog, "sacrifice-draw", None, Some(v)),
            );
            assert_eq!(resources(&r.game, 0), before - 1);
            assert!(board(&r.game, v).is_none());
            let f = r.stack.last().unwrap().frame.as_ref().unwrap();
            assert_eq!(f.source.card.id, dog);
            assert_eq!(f.actor, 0);
            assert_eq!(f.source.region, Some(0));
            assert_eq!(f.already_paid.len(), 3);
            assert!(r.players[owner].graveyard.iter().any(|c| c.id != *v
                && c.definition == if kind == "dog-self" { "JC096" } else { "JC125" }));
            drain(&mut r, &mut steps);
            assert!(r.players[0]
                .hand
                .iter()
                .any(|c| c.definition == "LC01" && c.id != old[0]));
            milestones.push(json!({"kind":"paid-source-and-sacrifice-exact-instance-owner-controller","step":steps.len()-1}));
        }
        "source-return" => {
            apply_game(
                &mut r,
                &mut steps,
                0,
                action(&dog, "sacrifice-draw", None, Some(&victim)),
            );
            apply_game(
                &mut r,
                &mut steps,
                0,
                action(rescuer.as_ref().unwrap(), "rescue", Some(&dog), None),
            );
            for _ in 0..40 {
                if board(&r.game, &dog).is_none() {
                    break;
                }
                let (s, a) = pass_action(&r.game);
                apply_game(&mut r, &mut steps, s, a);
            }
            assert!(board(&r.game, &dog).is_none());
            let returned = r.players[2]
                .hand
                .iter()
                .find(|c| c.definition == "JC096")
                .unwrap();
            assert_ne!(returned.id, dog);
            assert_eq!(returned.controller, 2);
            let f = r.stack.last().unwrap().frame.as_ref().unwrap();
            assert_eq!(
                (f.actor, f.source.card.controller, f.source.card.owner),
                (0, 0, 2)
            );
            assert_eq!(f.source.card.id, dog);
            drain(&mut r, &mut steps);
            assert!(r.players[0].hand.iter().any(|c| c.definition == "LC01"));
            assert_eq!(r.players[2].hand.len(), 1);
            milestones.push(json!({"kind":"actual-JC075-response-owner-return-fresh-instance-keeps-paid-actor-and-source","step":steps.len()-1}));
        }
        "offering-accept" | "offering-skip" => {
            apply_game(
                &mut r,
                &mut steps,
                0,
                action(&dog, "sacrifice-draw", None, Some(&offering)),
            );
            until_choice(&mut r, &mut steps, "trigger");
            let p = r.pending.clone().unwrap();
            assert_eq!(p.seat, 0);
            let ChoiceResolution::Declare { declaration, .. } = p.resolution else {
                panic!("source death")
            };
            assert_eq!(
                (
                    declaration.actor,
                    declaration.source.card.owner,
                    declaration.source.card.controller,
                    declaration.source.region
                ),
                (0, 2, 0, Some(0))
            );
            assert_eq!(declaration.source.card.id, offering);
            choose(
                &mut r,
                &mut steps,
                if kind == "offering-accept" {
                    vec!["accept".into()]
                } else {
                    vec![]
                },
            );
            if kind == "offering-accept" {
                until_choice(&mut r, &mut steps, "discard");
                privacy(&r, 0);
                let (s, a) = pick_choice(&r.game);
                apply_game(&mut r, &mut steps, s, a);
            }
            drain(&mut r, &mut steps);
            assert_eq!(
                r.players[0].deck.len(),
                if kind == "offering-accept" { 4 } else { 5 }
            );
            assert!(r.players[2]
                .graveyard
                .iter()
                .any(|c| c.definition == "XQ17" && c.id != offering));
            milestones.push(json!({"kind":"last-controller-death-source-private-continuation-or-decline","step":steps.len()-1}));
        }
        "loyalty" => {
            let offering = held(&mut r.game, "XQ17", 0);
            let dog = held(&mut r.game, "JC096", 0);
            // Starting state must already include all four hand instances.
            r.revision = r.game.version;
            steps[0] = step(
                &r,
                "initialFixture",
                json!([serde_json::to_string(&r).unwrap()]),
                0,
            );
            for (id, d) in [
                (&research, "JC130"),
                (&supplies, "JC131"),
                (&dog, "JC096"),
                (&offering, "XQ17"),
            ] {
                reject(&mut r, &mut steps, 0, declaration(id, d));
            }
        }
        "paid-four" => {
            let dog = held(&mut r.game, "JC096", 0);
            let offering = held(&mut r.game, "XQ17", 0);
            r.revision = r.game.version;
            steps[0] = step(
                &r,
                "initialFixture",
                json!([serde_json::to_string(&r).unwrap()]),
                0,
            );
            let before = resources(&r.game, 0);
            for (id, d) in [
                (&dog, "JC096"),
                (&offering, "XQ17"),
                (&research, "JC130"),
                (&supplies, "JC131"),
            ] {
                apply_game(&mut r, &mut steps, 0, declaration(id, d));
                drain(&mut r, &mut steps);
            }
            assert_eq!(resources(&r.game, 0), before - 10);
            milestones.push(
                json!({"kind":"four-original-costs-loyalty-colors-all-paid","step":steps.len()-1}),
            );
        }
        _ => unreachable!(),
    }
    json!({"name":format!("hand-deck-{kind}"),"seed":"9007199254740993","syntheticInitialLayout":true,"steps":steps,"milestones":milestones})
}
fn simultaneous(source_first: bool) -> Value {
    let mut g = attachment_initial("teams", "888888888888888888880151");
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.regions[0].influence = [0, 0];
    for r in &mut g.regions {
        r.cards.clear();
    }
    for id in ["XQ16", "JC042"] {
        fund(&mut g, id, 0, 6);
    }
    set_deck(&mut g, 0, &["LC01", "JC125", "JC125"]);
    let offering = field(&mut g, "XQ17", 2);
    let source = field(&mut g, "JZ27", 0);
    {
        let c = g.regions[0]
            .cards
            .iter_mut()
            .find(|c| c.id == source)
            .unwrap();
        c.face_down = true;
        c.damage = 2;
    }
    // Explicit starting checkpoint: actual JZ27 reveal/control prepared with
    // legal Game actions, then board order pinned before the compared trace.
    g.apply(
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    )
    .unwrap();
    for _ in 0..60 {
        if g.pending.is_some() {
            break;
        }
        let (s, a) = pass_action(&g);
        g.apply(s, a).unwrap();
    }
    let p = g.pending.clone().unwrap();
    assert_eq!(p.seat, 0);
    assert_eq!(p.choice.kind, "trigger");
    g.apply(
        0,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![offering.clone()]),
            ..Action::new("choose")
        },
    )
    .unwrap();
    for _ in 0..60 {
        if g.stack.is_empty() && g.pending.is_none() {
            break;
        }
        let (s, a) = if g.pending.is_some() {
            pick_choice(&g)
        } else {
            pass_action(&g)
        };
        g.apply(s, a).unwrap();
    }
    assert_eq!(board(&g, &offering).unwrap().1.controller, 0);
    let source = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JZ27")
        .unwrap()
        .id
        .clone();
    assert_eq!(board(&g, &source).unwrap().1.damage, 2);
    g.regions[0].cards.sort_by_key(|c| {
        if c.id == source {
            !source_first
        } else {
            source_first
        }
    });
    let spell = held(&mut g, "JC047", 0);
    let mut r = RoomEnvelope::from_game(g);
    let initial = serde_json::to_string(&r).unwrap();
    assert_eq!(
        serde_json::to_string(&RoomEnvelope::from_persisted(&initial).unwrap()).unwrap(),
        initial
    );
    let mut steps = vec![step(&r, "initialFixture", json!([initial]), 0)];
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(spell),
            region: Some(0),
            ..Action::new("play")
        },
    );
    until_choice(&mut r, &mut steps, "trigger");
    assert!(board(&r.game, &source).is_none() && board(&r.game, &offering).is_none());
    assert_eq!(r.pending.as_ref().unwrap().seat, 0);
    let ChoiceResolution::Declare { declaration, .. } = &r.pending.as_ref().unwrap().resolution
    else {
        panic!("death snapshot")
    };
    assert_eq!(
        (
            declaration.actor,
            declaration.source.card.controller,
            declaration.source.card.owner,
            declaration.source.region
        ),
        (0, 0, 2, Some(0))
    );
    assert_eq!(declaration.source.card.id, offering);
    choose(&mut r, &mut steps, vec!["accept".into()]);
    until_choice(&mut r, &mut steps, "discard");
    privacy(&r, 0);
    let (s, a) = pick_choice(&r.game);
    apply_game(&mut r, &mut steps, s, a);
    drain(&mut r, &mut steps);
    assert!(r.control_effects.is_empty() && r.control_baselines.is_empty());
    assert!(r.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ17" && c.id != offering && c.controller == 2));
    assert_eq!(r.players[0].deck.len(), 2);
    json!({"name":format!("hand-deck-real-JZ27-simultaneous-{}",if source_first{"source-first"}else{"offering-first"}),
        "seed":"9007199254740993","syntheticInitialLayout":true,"preparedControlCheckpoint":true,
        "steps":steps,"milestones":[{"kind":"actual-JC047-simultaneous-lethal-set-fixes-death-actor-before-source-control-restoration","step":steps.len()-1}]})
}
pub fn cases() -> impl Iterator<Item = Value> {
    [
        "paid-four",
        "research-private",
        "airdrop-private",
        "empty-search",
        "research-depletion",
        "dog-invalid",
        "dog-self",
        "controlled-not-owned",
        "offering-accept",
        "offering-skip",
        "loyalty",
        "source-return",
    ]
    .into_iter()
    .map(boundary)
    .chain([true, false].into_iter().map(simultaneous))
}
