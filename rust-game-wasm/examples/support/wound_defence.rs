//! Disclosed boundary layouts; every later transition uses real Room commands.
//! Marked response checkpoints are explicit synthetic continuations, never natural UI.
use super::*;
use hegemony_server::model::Card;
fn initial(name: &str) -> Game {
    let mut g = attachment_initial("teams", &format!("wound-defence-{name}"));
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
fn asset(g: &mut Game, id: &str, s: usize) -> String {
    fund(g, id, s, 1);
    g.players[s].assets.last().unwrap().id.clone()
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
    json!({"name":format!("wound-defence-{name}"),"seed":"9007199254740993","syntheticInitialLayout":true,"newNaturalUiCoverage":false,"steps":steps})
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
    panic!("bounded purple drain")
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
    panic!("bounded purple advance")
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
fn deploy(id: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        region: Some(0),
        ..Action::new("deploy")
    }
}

fn grant(r: &mut RoomEnvelope, steps: &mut Vec<Value>, h: &str, t: &str) {
    apply_game(r, steps, 0, activate(h, "protect-local-character", Some(t)));
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
fn grant_case(kind: &str) -> Value {
    let mut g = initial(kind);
    let h = field(&mut g, "LC12", if kind == "controller" { 3 } else { 0 });
    if kind == "controller" {
        g.regions[0].cards.last_mut().unwrap().controller = 0;
    }
    let t = field(&mut g, "JC125", if kind == "controller" { 2 } else { 0 });
    if kind == "controller" {
        g.regions[0].cards.last_mut().unwrap().controller = 0;
    }
    let second = if kind == "stack" {
        Some(field(&mut g, "LC12", 0))
    } else {
        None
    };
    let rescue = if matches!(kind, "source-return" | "target-return") {
        Some(field(&mut g, "JC075", 0))
    } else {
        None
    };
    if kind == "no-assets" {
        g.players[0].assets.clear();
    }
    let (mut r, mut steps) = room(g);
    let before = resources(&r, 0);
    grant(&mut r, &mut steps, &h, &t);
    assert_eq!(resources(&r, 0), before);
    assert!(board(&r, &h).unwrap().exhausted);
    if let Some(rescue) = rescue {
        let id = if kind == "source-return" { &h } else { &t };
        apply_game(&mut r, &mut steps, 0, activate(&rescue, "rescue", Some(id)));
    }
    drain(&mut r, &mut steps);
    if kind == "target-return" {
        assert!(r.turn_attribute_modifiers.is_empty());
        assert!(board(&r, &t).is_none());
        let id = r.players[0]
            .hand
            .iter()
            .find(|c| c.definition == "JC125")
            .unwrap()
            .id
            .clone();
        assert_ne!(id, t);
        apply_game(&mut r, &mut steps, 0, deploy(&id));
        drain(&mut r, &mut steps);
        let c = r.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "JC125")
            .unwrap();
        assert_ne!(c.id, t);
        assert_eq!(r.game.defense(c, 0), 1);
    } else {
        if let Some(h) = second {
            grant(&mut r, &mut steps, &h, &t);
            drain(&mut r, &mut steps);
        }
        let expected = if kind == "stack" { 3 } else { 2 };
        assert_eq!(r.game.defense(board(&r, &t).unwrap(), 0), expected);
        for s in 0..4 {
            let v = r.game.view(s);
            let c = v.regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == t)
                .unwrap();
            assert_eq!(c.defense, Some(expected));
            assert_eq!(c.icons, Some(Default::default()));
        }
        assert!(r
            .turn_attribute_modifiers
            .iter()
            .all(|m| m.ordinary_icons == Default::default() && !m.grants_renown));
    }
    if kind == "source-return" {
        assert!(board(&r, &h).is_none());
        assert!(r.players[0]
            .hand
            .iter()
            .any(|c| c.definition == "LC12" && c.id != h));
    }
    result(&format!("hermit-{kind}"), steps)
}
fn denied_target(kind: &str) -> Value {
    let mut g = initial(kind);
    let h = field(&mut g, "LC12", 0);
    let t = match kind {
        "self" => h.clone(),
        "teammate" => field(&mut g, "JC125", 1),
        "enemy" => field(&mut g, "JC125", 2),
        "other-region" => {
            let id = field(&mut g, "JC125", 0);
            let c = g.regions[0].cards.pop().unwrap();
            g.regions[1].cards.push(c);
            id
        }
        "hidden" => {
            let id = field(&mut g, "JC125", 0);
            g.regions[0].cards.last_mut().unwrap().face_down = true;
            id
        }
        "asset" => asset(&mut g, "JC125", 0),
        "hand" => held(&mut g, "JC125", 0),
        "missing" => "missing".into(),
        _ => panic!(),
    };
    let (mut r, mut steps) = room(g);
    reject(
        &mut r,
        &mut steps,
        0,
        activate(&h, "protect-local-character", Some(&t)),
    );
    assert!(!board(&r, &h).unwrap().exhausted);
    assert!(r.turn_attribute_modifiers.is_empty());
    result(&format!("hermit-reject-{kind}"), steps)
}
fn changed_checkpoint(kind: &str) -> Value {
    let mut g = initial(kind);
    let h = field(&mut g, "LC12", 0);
    let t = field(&mut g, "JC125", 0);
    let (mut r, mut preparation) = room(g);
    grant(&mut r, &mut preparation, &h, &t);
    // Disclosed target-change checkpoint keeps the real command-created pacing.
    match kind {
        "moved" => {
            let c = r.game.regions[0].cards.pop().unwrap();
            r.game.regions[1].cards.push(c);
        }
        "controlled" => r.game.regions[0].cards.last_mut().unwrap().controller = 2,
        "hidden" => r.game.regions[0].cards.last_mut().unwrap().face_down = true,
        _ => panic!(),
    };
    let (mut r, mut steps) = checkpoint(r);
    assert!(board(&r, &h).unwrap().exhausted);
    drain(&mut r, &mut steps);
    assert!(r.turn_attribute_modifiers.is_empty());
    let mut v = result(&format!("prepared-hermit-target-{kind}-cancels"), steps);
    v["preparedChangedTargetResponseCheckpoint"] = json!(true);
    v
}
fn wound_case(kind: &str) -> Value {
    let mut g = initial(kind);
    let source = field(&mut g, "JZ59", if kind == "controller" { 3 } else { 0 });
    if kind == "controller" {
        g.regions[0].cards.last_mut().unwrap().controller = 0;
    }
    let target = if kind == "no-target" {
        None
    } else {
        Some(field(
            &mut g,
            if matches!(kind, "lethal" | "saved" | "prevented") {
                "JC125"
            } else {
                "LC01"
            },
            if matches!(kind, "saved" | "prevented") {
                0
            } else {
                2
            },
        ))
    };
    let other = field(&mut g, "LC01", 2);
    let c = g.regions[0].cards.pop().unwrap();
    g.regions[1].cards.push(c);
    if kind == "shield" {
        g.regions[0].cards.last_mut().unwrap().shield = 1;
    }
    let h = if kind == "saved" {
        Some(field(&mut g, "LC12", 0))
    } else {
        None
    };
    let prevent = if kind == "prevented" {
        fund(&mut g, "JC078", 0, 1);
        Some(held(&mut g, "JC078", 0))
    } else {
        None
    };
    let rescue = if kind == "target-return" {
        fund(&mut g, "JC075", 2, 2);
        Some(field(&mut g, "JC075", 2))
    } else {
        None
    };
    let murder = held(&mut g, "JC091", 0);
    let (mut r, mut steps) = room(g);
    if let Some(prevent) = prevent {
        apply_game(
            &mut r,
            &mut steps,
            0,
            play(&prevent, target.as_ref().unwrap()),
        );
        drain(&mut r, &mut steps);
        assert!(r
            .game
            .damage_prevented(board(&r, target.as_ref().unwrap()).unwrap()));
    }
    if kind == "no-target" {
        apply_game(&mut r, &mut steps, 0, play(&murder, &source));
        drain(&mut r, &mut steps);
        assert!(r.pending.is_none());
        return result("death-no-local-character-no-choice", steps);
    }
    kill(&mut r, &mut steps, &murder, &source);
    let p = r.pending.as_ref().unwrap();
    assert_eq!(p.seat, 0);
    let ChoiceResolution::Declare { declaration, .. } = &p.resolution else {
        panic!()
    };
    assert_eq!(declaration.source.card.id, source);
    assert_eq!(declaration.source.region, Some(0));
    assert_eq!(declaration.actor, 0);
    assert_eq!(declaration.source.card.controller, 0);
    assert_eq!(
        declaration.source.card.owner,
        if kind == "controller" { 3 } else { 0 }
    );
    let target = target.unwrap();
    assert!(p.choice.options.iter().any(|o| o.id == target));
    assert!(!p
        .choice
        .options
        .iter()
        .any(|o| o.id == source || o.id == other));
    for viewer in 0..4 {
        let v = r.game.view(viewer);
        assert_eq!(v.pending_choice.is_some(), viewer == 0);
    }
    if kind == "decline" {
        let before = resources(&r, 0);
        choose(&mut r, &mut steps, vec![]);
        drain(&mut r, &mut steps);
        assert_eq!(resources(&r, 0), before);
        assert_eq!(board(&r, &target).unwrap().wounds, 0);
        return result("death-decline-free", steps);
    }
    let before = resources(&r, 0);
    choose(&mut r, &mut steps, vec![target.clone()]);
    assert_eq!(resources(&r, 0), before);
    if let Some(h) = h {
        grant(&mut r, &mut steps, &h, &target);
    }
    if let Some(rescue) = rescue {
        advance(&mut r, &mut steps, |r| {
            r.pending.is_none() && r.priority_team == 1
        });
        apply_game(
            &mut r,
            &mut steps,
            2,
            activate(&rescue, "rescue", Some(&target)),
        );
    }
    drain(&mut r, &mut steps);
    if matches!(kind, "lethal" | "prevented" | "target-return") {
        assert!(board(&r, &target).is_none());
        if kind == "target-return" {
            assert!(r.players[2]
                .hand
                .iter()
                .any(|c| c.definition == "LC01" && c.id != target && c.wounds == 0));
        } else {
            assert!(r.players[if kind == "prevented" { 0 } else { 2 }]
                .graveyard
                .iter()
                .any(|c| c.definition == "JC125" && c.id != target));
        }
    } else if kind == "shield" {
        assert_eq!(board(&r, &target).unwrap().shield, 0);
        assert_eq!(board(&r, &target).unwrap().wounds, 0);
    } else {
        let c = board(&r, &target).unwrap();
        assert_eq!(c.wounds, 1);
        assert_eq!(c.damage, 0);
        assert_eq!(r.game.defense(c, 0), if kind == "saved" { 1 } else { 3 });
        if matches!(kind, "persist" | "saved") {
            let turn = r.turn;
            advance(&mut r, &mut steps, |r| r.turn > turn);
            if kind == "saved" {
                assert!(board(&r, &target).is_none());
            } else {
                assert_eq!(board(&r, &target).unwrap().wounds, 1);
                assert_eq!(board(&r, &target).unwrap().damage, 0);
            }
        }
    }
    result(&format!("real-murder-death-wound-{kind}"), steps)
}
fn sacrificed() -> Value {
    let mut g = initial("sacrificed-cost");
    fund(&mut g, "JC049", 0, 2);
    let source = field(&mut g, "JZ59", 0);
    let target = field(&mut g, "LC01", 2);
    let spell = held(&mut g, "JC049", 0);
    let (mut r, mut steps) = room(g);
    let before = resources(&r, 0);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(spell),
            cost_selected: Some(vec![source.clone()]),
            ..Action::new("play")
        },
    );
    advance(&mut r, &mut steps, |r| {
        r.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "trigger")
    });
    assert_eq!(resources(&r, 0), before - 2);
    choose(&mut r, &mut steps, vec![target.clone()]);
    drain(&mut r, &mut steps);
    assert_eq!(resources(&r, 0), before - 2);
    assert_eq!(board(&r, &target).unwrap().wounds, 1);
    assert_eq!(
        r.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JZ59")
            .count(),
        1
    );
    result("real-sacrifice-cost-independent-death-wound", steps)
}
fn cleanup_case(wound: bool) -> Value {
    let mut g = initial("cleanup");
    let h = field(&mut g, "LC12", 0);
    let target = field(&mut g, if wound { "JZ59" } else { "JC125" }, 0);
    let buddy = field(&mut g, "LC01", 0);
    // Declare and resolve the actual grant before marking a disclosed cleanup checkpoint.
    g.apply(0, activate(&h, "protect-local-character", Some(&target)))
        .unwrap();
    while !g.stack.is_empty() {
        let (s, a) = (0..4)
            .find_map(|s| {
                g.legal_actions(s)
                    .into_iter()
                    .find(|a| a.action.kind == "pass")
                    .map(|a| (s, a.action))
            })
            .unwrap();
        g.apply(s, a).unwrap();
    }
    let c = g.regions[0]
        .cards
        .iter_mut()
        .find(|c| c.id == target)
        .unwrap();
    if wound {
        c.wounds = 1;
    } else {
        c.damage = 1;
    }
    g.window = Some(Window::End);
    g.priority_team = 0;
    g.passed.clear();
    g.team_passed = [false; 2];
    let (mut r, mut steps) = room(g);
    let turn = r.turn;
    let mut observed = false;
    for _ in 0..650 {
        if r.turn > turn {
            break;
        }
        if let Some(p) = r.pending.clone() {
            if let ChoiceResolution::Declare { declaration, .. } = &p.resolution {
                if declaration.source.card.id == target {
                    assert!(wound);
                    assert_eq!(declaration.actor, 0);
                    assert_eq!(declaration.source.card.wounds, 1);
                    observed = true;
                    choose(&mut r, &mut steps, vec![buddy.clone()]);
                    continue;
                }
            }
            let (s, a) = pick_choice(&r.game);
            apply_game(&mut r, &mut steps, s, a);
        } else {
            pass(&mut r, &mut steps)
        }
    }
    assert!(r.turn > turn);
    assert!(r.turn_attribute_modifiers.is_empty());
    if wound {
        assert!(observed);
        assert!(board(&r, &target).is_none());
        assert_eq!(board(&r, &buddy).unwrap().wounds, 1);
    } else {
        let c = board(&r, &target).unwrap();
        assert_eq!(c.damage, 0);
        assert_eq!(r.game.defense(c, 0), 1);
    }
    let mut v = result(
        if wound {
            "prepared-cleanup-wounds-stay-expiry-real-death"
        } else {
            "prepared-cleanup-damage-cleared-with-defense-expiry"
        },
        steps,
    );
    v["preparedActualResolvedGrantAndEndCheckpoint"] = json!(true);
    v
}
pub fn cases() -> impl Iterator<Item = Value> {
    let mut cases: Vec<Box<dyn FnOnce() -> Value>> = vec![];
    for kind in [
        "no-assets",
        "controller",
        "stack",
        "source-return",
        "target-return",
    ] {
        cases.push(Box::new(move || grant_case(kind)));
    }
    for kind in [
        "self",
        "teammate",
        "enemy",
        "other-region",
        "hidden",
        "asset",
        "hand",
        "missing",
    ] {
        cases.push(Box::new(move || denied_target(kind)));
    }
    for kind in ["moved", "controlled", "hidden"] {
        cases.push(Box::new(move || changed_checkpoint(kind)));
    }
    for kind in [
        "normal",
        "controller",
        "no-target",
        "decline",
        "lethal",
        "prevented",
        "shield",
        "target-return",
        "persist",
        "saved",
    ] {
        cases.push(Box::new(move || wound_case(kind)));
    }
    cases.push(Box::new(sacrificed));
    cases.push(Box::new(|| cleanup_case(false)));
    cases.push(Box::new(|| cleanup_case(true)));
    cases.into_iter().map(|f| f())
}
