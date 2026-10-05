//! Explicit boundary layouts/checkpoints; all recorded commands use Room ABI.
//! No case in this batch is a natural UI game.
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
fn initial(name: &str) -> Game {
    let mut g = attachment_initial("teams", &format!("hand-interactions-{name}"));
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.regions[0].influence = [0, 0];
    for p in &mut g.players {
        p.graveyard.clear();
    }
    for id in ["JC075", "JC084", "JC104"] {
        fund(&mut g, id, 0, 3);
    }
    set_deck(&mut g, 0, &["JC003", "JC125", "JC125", "JC125"]);
    g
}
fn room(g: Game) -> (RoomEnvelope, Vec<Value>) {
    let r = RoomEnvelope::from_game(g);
    let raw = serde_json::to_string(&r).unwrap();
    assert_eq!(
        serde_json::to_string(&RoomEnvelope::from_persisted(&raw).unwrap()).unwrap(),
        raw
    );
    let steps = vec![step(&r, "initialFixture", json!([raw]), 0)];
    (r, steps)
}
fn hacker(kind: &str, target: usize) -> Value {
    let mut g = initial(kind);
    let h = field(&mut g, "JC114", 0);
    let id = held(
        &mut g,
        if kind.starts_with("inspiration") {
            "XQ34"
        } else {
            "JC003"
        },
        target,
    );
    if kind == "foreign-discard" {
        g.players[target].hand[0].owner = 3;
    }
    if kind.starts_with("inspiration") {
        fund(
            &mut g,
            "JC075",
            target,
            usize::from(kind != "inspiration-unfunded"),
        );
        set_deck(&mut g, target, &["JC125", "JC003"]);
    }
    let rescuer = if kind == "source-return" {
        Some(field(&mut g, "JC075", 0))
    } else {
        None
    };
    let (r0, mut steps) = room(g);
    let mut r = r0;
    apply_game(
        &mut r,
        &mut steps,
        0,
        action(&h, "reveal-hand-discard", Some(&format!("p{target}")), None),
    );
    assert!(board(&r, &h).unwrap().1.exhausted);
    if let Some(rescuer) = rescuer {
        apply_game(
            &mut r,
            &mut steps,
            0,
            action(&rescuer, "rescue", Some(&h), None),
        );
    }
    until_choice(&mut r, &mut steps, "revealed-hand-discard");
    for s in 0..4 {
        let v = r.game.view(s);
        assert_eq!(v.revealed_hands[0].cards[0].instance_id, id);
        assert_eq!(v.pending_choice.is_some(), s == 0);
    }
    let cid = r.pending.as_ref().unwrap().choice.id.clone();
    reject(
        &mut r,
        &mut steps,
        1,
        Action {
            choice_id: Some(cid.clone()),
            selected: Some(vec![id.clone()]),
            ..Action::new("choose")
        },
    );
    let accept = !matches!(kind, "decline" | "source-return");
    if kind == "source-return" {
        assert_eq!(r.pending.as_ref().unwrap().choice.max, Some(0));
        reject(
            &mut r,
            &mut steps,
            0,
            Action {
                choice_id: Some(cid),
                selected: Some(vec![id.clone()]),
                ..Action::new("choose")
            },
        );
    }
    choose(
        &mut r,
        &mut steps,
        if accept { vec![id.clone()] } else { vec![] },
    );
    if kind.starts_with("inspiration") && kind != "inspiration-unfunded" {
        assert_eq!(r.pending.as_ref().unwrap().seat, target);
        choose(
            &mut r,
            &mut steps,
            if kind == "inspiration-skip" {
                vec![]
            } else {
                vec!["accept".into()]
            },
        );
    }
    drain(&mut r, &mut steps);
    for s in 0..4 {
        assert!(r.game.view(s).revealed_hands.is_empty());
    }
    if kind == "foreign-discard" {
        assert!(r.players[3]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC003" && c.id != id));
    }
    if kind == "source-return" {
        assert!(r.players[0]
            .hand
            .iter()
            .any(|c| c.definition == "JC114" && c.id != h));
    }
    if kind.starts_with("inspiration") {
        assert_eq!(
            r.players[target].hand.len(),
            usize::from(kind == "inspiration-accept")
        );
    }
    json!({"seed":"9007199254740993","name":format!("hand-interactions-hacker-{kind}-target-{target}"),"syntheticInitialLayout":true,"steps":steps})
}
fn recover(kind: &str) -> Value {
    let mut g = initial(kind);
    let source = field(&mut g, "XQ38", 0);
    let cost = held(
        &mut g,
        if kind.starts_with("inspiration") {
            "XQ34"
        } else {
            "JC125"
        },
        0,
    );
    let foreign = held(&mut g, "JC125", 1);
    let c = g.make_card("JC003", 0);
    let target = c.id.clone();
    g.players[0].graveyard.push(c);
    if kind == "inspiration-unfunded" {
        g.players[0].assets.truncate(2);
    }
    let before = resources(&g, 0);
    let (mut r, mut steps) = room(g);
    for bad in [
        None,
        Some(foreign.as_str()),
        Some(source.as_str()),
        Some("old-instance"),
    ] {
        reject(
            &mut r,
            &mut steps,
            0,
            action(&source, "recover-grave-discard", Some(&target), bad),
        );
    }
    reject(
        &mut r,
        &mut steps,
        0,
        action(&source, "recover-grave-discard", Some(&cost), Some(&cost)),
    );
    let mut bad = action(&source, "recover-grave-discard", Some(&target), Some(&cost));
    bad.cost_selected = Some(vec![cost.clone(), cost.clone()]);
    reject(&mut r, &mut steps, 0, bad);
    apply_game(
        &mut r,
        &mut steps,
        0,
        action(&source, "recover-grave-discard", Some(&target), Some(&cost)),
    );
    assert_eq!(resources(&r, 0), before - 2);
    assert!(board(&r, &source).unwrap().1.exhausted);
    if kind.starts_with("inspiration") && kind != "inspiration-unfunded" {
        assert_eq!(r.stack.len(), 1);
        assert_eq!(r.pending.as_ref().unwrap().seat, 0);
        choose(
            &mut r,
            &mut steps,
            if kind == "inspiration-skip" {
                vec![]
            } else {
                vec!["accept".into()]
            },
        );
        if kind == "inspiration-accept" {
            assert_eq!(r.stack.len(), 2);
            attachment_pass_top(&mut r, &mut steps);
            assert!(r.players[0].hand.iter().any(|c| c.definition == "JC003")); /* deck first has same printed card; exact target still grave */
            assert!(r.players[0].graveyard.iter().any(|c| c.id == target));
        }
    }
    drain(&mut r, &mut steps);
    assert!(!r.players[0].graveyard.iter().any(|c| c.id == target));
    assert!(r.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "JC003" && c.id != target));
    json!({"seed":"9007199254740993","name":format!("hand-interactions-recover-{kind}"),"syntheticInitialLayout":true,"steps":steps})
}
fn mill(n: usize) -> Value {
    let mut g = initial("mill");
    let source = held(&mut g, "XQ38", 0);
    let old = set_deck(&mut g, 2, &["XQ34", "JC125", "JC003"][..n]);
    fund(&mut g, "JC075", 2, 1);
    let rng = g.random;
    let (mut r, mut steps) = room(g);
    apply_game(&mut r, &mut steps, 0, declaration(&source, "XQ38"));
    until_choice(&mut r, &mut steps, "trigger");
    choose(&mut r, &mut steps, vec!["p2".into()]);
    drain(&mut r, &mut steps);
    assert_eq!(r.players[2].graveyard.len(), n);
    assert!(r.players[2].deck.is_empty());
    assert!(!r.players[2].eliminated);
    assert_eq!(r.random, rng);
    assert_eq!(resources(&r, 2), 1);
    for c in &r.players[2].graveyard {
        assert!(!old.contains(&c.id));
    }
    json!({"seed":"9007199254740993","name":format!("hand-interactions-entry-mill-{n}"),"syntheticInitialLayout":true,"steps":steps})
}
fn amnesia(kind: &str, target: usize) -> Value {
    let mut g = initial(kind);
    let source = held(&mut g, "JZ67", 0);
    let ids = if kind == "empty" {
        vec![]
    } else if kind == "random-three" {
        vec![
            held(&mut g, "XQ34", target),
            held(&mut g, "JC003", target),
            held(&mut g, "JC125", target),
        ]
    } else {
        vec![held(&mut g, "XQ34", target)]
    };
    if kind == "foreign-owner" {
        g.players[target].hand[0].owner = 3;
    }
    fund(&mut g, "JC075", target, 1);
    let owner = if kind == "foreign-owner" { 3 } else { target };
    set_deck(&mut g, owner, &["JC125", "JC125", "JC125"]);
    let oldhand = g.players[target]
        .hand
        .iter()
        .filter(|c| c.id != source)
        .cloned()
        .collect::<Vec<_>>();
    let (mut r, mut steps) = room(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(source),
            target_id: Some(format!("p{target}")),
            ..Action::new("play")
        },
    );
    until_choice(&mut r, &mut steps, "investigation");
    if kind != "empty" {
        assert_eq!(r.players[target].hand.len(), ids.len() - 1);
        let moved = oldhand
            .iter()
            .find(|c| !r.players[target].hand.iter().any(|h| h.id == c.id))
            .unwrap();
        assert_eq!(r.players[moved.owner].deck[0].definition, moved.definition);
        assert_ne!(r.players[moved.owner].deck[0].id, moved.id);
        let text = serde_json::to_string(&r.game.view(1)).unwrap();
        for c in &r.players[target].hand {
            if target != 1 {
                assert!(!text.contains(&c.id));
            }
        }
    }
    for s in 1..4 {
        assert!(r.game.view(s).pending_choice.is_none());
    }
    let p = r.pending.clone().unwrap();
    let ids = p
        .choice
        .options
        .iter()
        .map(|o| o.id.clone())
        .collect::<Vec<_>>();
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            choice_id: Some(p.choice.id),
            top: Some(vec![]),
            bottom: Some(ids),
            ..Action::new("choose")
        },
    );
    drain(&mut r, &mut steps);
    assert!(r.pending.is_none());
    assert_eq!(resources(&r, target), if target == 0 { 9 } else { 1 });
    json!({"seed":"9007199254740993","name":format!("hand-interactions-amnesia-{kind}-target-{target}"),"syntheticInitialLayout":true,"steps":steps})
}
fn prepared_shuffle(accept: bool) -> Value {
    // There is no admitted 梦魔. Persist the post-condition continuation explicitly.
    let mut g = initial("prepared-shuffle");
    let c = g.make_card("JZ67", 0);
    let source = SourceSnapshot {
        card: c,
        region: None,
        attachment_host_instance: None,
        play_source: Some(hegemony_server::model::PlaySource::Hand),
    };
    let frame = hegemony_server::model::ResolutionFrame {
        actor: 0,
        source,
        targets: vec![],
        frame_id: "prepared-frame".into(),
        ability_key: "amnesia".into(),
        already_paid: vec![],
        guard: hegemony_server::model::GuardState::Accepted,
        cursor: 0,
        steps: vec![],
        chosen_region: None,
    };
    let p = hegemony_server::model::Pending {
        seat: 0,
        choice: hegemony_server::model::Choice {
            id: "prepared-shuffle-choice".into(),
            kind: "optional-shuffle".into(),
            title: "是否洗牌".into(),
            description: "保存的洗牌继续选择".into(),
            player_id: "p0".into(),
            options: vec![hegemony_server::model::ChoiceOption {
                id: "shuffle".into(),
                label: "洗牌".into(),
                card: None,
            }],
            min: Some(0),
            max: Some(1),
            amount: None,
            allow_decline: Some(true),
        },
        resolution: ChoiceResolution::Frame {
            frame: Box::new(frame),
            choice: FrameChoice::OptionalShuffle { seat: 2 },
        },
    };
    g.pending = Some(p);
    set_deck(&mut g, 2, &["JC003", "JC125", "XQ34"]);
    let rng = g.random;
    let (mut r, mut steps) = room(g);
    choose(
        &mut r,
        &mut steps,
        if accept {
            vec!["shuffle".into()]
        } else {
            vec![]
        },
    );
    assert_eq!(r.random == rng, !accept);
    json!({"seed":"9007199254740993","name":format!("hand-interactions-prepared-optional-shuffle-{}",if accept{"accept"}else{"decline"}),"syntheticInitialLayout":true,"preparedPostConditionCheckpoint":true,"positiveDreamDemonConditionNotCovered":true,"steps":steps})
}
pub fn cases() -> impl Iterator<Item = Value> {
    [0, 1, 2, 3]
        .into_iter()
        .map(|s| hacker("accept", s))
        .chain(
            [
                "decline",
                "source-return",
                "foreign-discard",
                "inspiration-accept",
                "inspiration-skip",
                "inspiration-unfunded",
            ]
            .into_iter()
            .map(|k| hacker(k, 2)),
        )
        .chain(
            [
                "ordinary",
                "inspiration-accept",
                "inspiration-skip",
                "inspiration-unfunded",
            ]
            .into_iter()
            .map(recover),
        )
        .chain([0, 1, 3].into_iter().map(mill))
        .chain(
            [
                ("one", 2),
                ("random-three", 2),
                ("foreign-owner", 2),
                ("empty", 2),
                ("self", 0),
            ]
            .into_iter()
            .map(|(k, s)| amnesia(k, s)),
        )
        .chain([true, false].into_iter().map(prepared_shuffle))
        .chain(std::iter::once_with(normal_play_and_empty_hacker))
        .chain(std::iter::once_with(prepared_stale_recovery))
}
fn normal_play_and_empty_hacker() -> Value {
    let mut g = initial("normal-play-empty-hacker");
    let source = held(&mut g, "XQ34", 0);
    let h = field(&mut g, "JC114", 0);
    g.players[2].hand.clear();
    let (mut r, mut steps) = room(g);
    apply_game(&mut r, &mut steps, 0, declaration(&source, "XQ34"));
    drain(&mut r, &mut steps);
    assert_eq!(r.players[0].hand.len(), 1);
    assert!(r.pending.is_none());
    let (s, a) = pass_action(&r.game);
    apply_game(&mut r, &mut steps, s, a); // Standard action returns to the next team; find the legal window below.
    for _ in 0..10 {
        if r.game
            .legal_actions(0)
            .iter()
            .any(|a| a.action.card_id.as_deref() == Some(&h))
        {
            break;
        }
        let (s, a) = pass_action(&r.game);
        apply_game(&mut r, &mut steps, s, a);
    }
    apply_game(
        &mut r,
        &mut steps,
        0,
        action(&h, "reveal-hand-discard", Some("p2"), None),
    );
    drain(&mut r, &mut steps);
    assert!(board(&r, &h).unwrap().1.exhausted);
    assert!(r.game.view(1).revealed_hands.is_empty());
    json!({"seed":"9007199254740993","name":"hand-interactions-normal-inspiration-play-empty-hacker","syntheticInitialLayout":true,"steps":steps})
}
fn prepared_stale_recovery() -> Value {
    let mut g = initial("prepared-stale-recovery");
    let source = field(&mut g, "XQ38", 0);
    let cost = held(&mut g, "XQ34", 0);
    let c = g.make_card("JC003", 0);
    let target = c.id.clone();
    g.players[0].graveyard.push(c);
    let before = resources(&g, 0);
    g.apply(
        0,
        action(&source, "recover-grave-discard", Some(&target), Some(&cost)),
    )
    .unwrap();
    let pos = g.players[0]
        .graveyard
        .iter()
        .position(|c| c.id == target)
        .unwrap();
    g.players[0].graveyard.remove(pos);
    let replacement = g.make_card("JC003", 0);
    g.players[0].hand.push(replacement);
    let (mut r, mut steps) = room(g);
    assert_eq!(resources(&r, 0), before - 2);
    choose(&mut r, &mut steps, vec!["accept".into()]);
    drain(&mut r, &mut steps);
    assert_eq!(resources(&r, 0), before - 3);
    assert!(board(&r, &source).unwrap().1.exhausted);
    assert_eq!(
        r.players[0]
            .hand
            .iter()
            .filter(|c| c.definition == "JC003")
            .count(),
        2
    );
    assert!(!r.players[0].graveyard.iter().any(|c| c.id == target));
    json!({"seed":"9007199254740993","name":"hand-interactions-prepared-stale-recovery-trigger-still-draws","syntheticInitialLayout":true,"preparedChangedTargetCheckpoint":true,"steps":steps})
}
