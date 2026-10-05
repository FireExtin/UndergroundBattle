//! Disclosed boundary layouts; every later transition uses real Room commands.
//! The free-reveal case begins at the real region Win window, not a replacement Op.
use super::*;
use hegemony_server::model::Card;
fn initial(name: &str) -> Game {
    let mut g = attachment_initial("teams", &format!("purple-tools-{name}"));
    for p in &mut g.players {
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.influence = [0, 0];
    }
    fund(&mut g, "JC104", 0, 4);
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
    let r = RoomEnvelope::from_game(g);
    let raw = serde_json::to_string(&r).unwrap();
    assert_eq!(
        serde_json::to_string(&RoomEnvelope::from_persisted(&raw).unwrap()).unwrap(),
        raw
    );
    let steps = vec![step(&r, "initialFixture", json!([raw]), 0)];
    (r, steps)
}
fn result(name: &str, steps: Vec<Value>) -> Value {
    json!({"name":format!("purple-tools-{name}"),"seed":"9007199254740993","syntheticInitialLayout":true,"newNaturalUiCoverage":false,"steps":steps})
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
fn painter_play() -> Value {
    let mut g = initial("painter-play");
    let p = field(&mut g, "JC103", 0);
    let victim = asset(&mut g, "XQ17", 2);
    let spell = held(&mut g, "JC107", 0);
    let (mut r, mut steps) = room(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        activate(&p, "reduce-next-purple", None),
    );
    assert_eq!(resources(&r, 0), 4);
    assert!(board(&r, &p).unwrap().exhausted);
    assert_eq!(r.modifiers.len(), 1);
    assert!(r.stack.is_empty());
    for seat in 0..4 {
        assert!(r
            .view(seat, r.pacing.last_server_now_ms)
            .response_window
            .is_none());
    }
    reject(
        &mut r,
        &mut steps,
        0,
        activate(&p, "reduce-next-purple", None),
    );
    drain(&mut r, &mut steps);
    assert!(r.modifiers[0].paid_reveal);
    assert_eq!(r.game.view(0).hand[0].effective_cost, Some(2));
    apply_game(&mut r, &mut steps, 0, play(&spell, &victim));
    assert_eq!(resources(&r, 0), 2);
    assert_eq!(r.modifiers[0].uses, 0);
    for s in 0..4 {
        let v = r.game.view(s);
        assert_eq!(v.stack.last().unwrap().target_summaries[0].label, "资产");
    }
    while !r.stack.is_empty() {
        assert!(r.pending.is_none());
        pass(&mut r, &mut steps)
    }
    assert!(r.players[2].assets.is_empty());
    let c = r.players[2]
        .graveyard
        .iter()
        .find(|c| c.definition == "XQ17")
        .unwrap();
    assert_ne!(c.id, victim);
    assert!(!c.exhausted);
    assert_eq!(c.controller, 2);
    assert!(!r.log.iter().any(|l| l.text.contains("祭品：死亡")));
    result(
        "painter-exhaust-only-discount-play-asset-character-no-death",
        steps,
    )
}
fn reveal_scope(legacy: bool) -> Value {
    let mut g = initial("reveal-scope");
    let p = field(&mut g, if legacy { "JC112" } else { "JC103" }, 0);
    let h = held(&mut g, "JC104", 0);
    let spare = held(&mut g, "JC125", 0);
    let (mut r, mut steps) = room(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        activate(
            &p,
            if legacy {
                "reduce-next-magic"
            } else {
                "reduce-next-purple"
            },
            None,
        ),
    );
    drain(&mut r, &mut steps);
    assert_eq!(r.modifiers[0].paid_reveal, !legacy);
    if legacy {
        assert!(!serde_json::to_string(&r.modifiers[0])
            .unwrap()
            .contains("paid_reveal"));
    }
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(h),
            region: Some(0),
            ..Action::new("conceal")
        },
    );
    assert_eq!(resources(&r, 0), 3);
    assert_eq!(r.modifiers[0].uses, 1);
    let hidden = r.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC104")
        .unwrap()
        .id
        .clone();
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(spare),
            ..Action::new("asset")
        },
    );
    assert_eq!(r.modifiers[0].uses, 1);
    let before = resources(&r, 0);
    apply_game(
        &mut r,
        &mut steps,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    assert_eq!(resources(&r, 0), before - if legacy { 2 } else { 1 });
    assert_eq!(r.modifiers[0].uses, if legacy { 1 } else { 0 });
    drain(&mut r, &mut steps);
    result(
        if legacy {
            "legacy-reduction-excludes-paid-reveal"
        } else {
            "conceal-asset-do-not-consume-paid-reveal-does"
        },
        steps,
    )
}
fn free_reveal() -> Value {
    let mut g = initial("real-win-free-reveal");
    if let Some(at) = g
        .regions
        .iter()
        .position(|r| r.card.definition == "DQJC108")
    {
        g.regions.swap(0, at);
    } else {
        let at = g
            .world
            .iter()
            .position(|c| c.definition == "DQJC108")
            .unwrap();
        std::mem::swap(&mut g.regions[0].card, &mut g.world[at]);
    }
    g.window = Some(Window::Win(0, 0));
    let p = field(&mut g, "JC103", 0);
    let h = field(&mut g, "JC104", 0);
    g.regions[0].cards.last_mut().unwrap().face_down = true;
    let hidden = g.regions[0].cards.pop().unwrap();
    g.regions[2].cards.push(hidden);
    let (mut r, mut steps) = room(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        activate(&p, "reduce-next-purple", None),
    );
    drain(&mut r, &mut steps);
    assert_eq!(r.modifiers[0].uses, 1);
    let before = resources(&r, 0);
    let turn = r.turn;
    for _ in 0..150 {
        if r.regions
            .iter()
            .flat_map(|r| &r.cards)
            .any(|c| c.definition == "JC104" && !c.face_down)
        {
            break;
        }
        if let Some(p) = r.pending.clone() {
            let (s, mut a) = pick_choice(&r.game);
            if matches!(p.resolution, ChoiceResolution::Declare { .. }) {
                a.selected = Some(vec!["accept".into()]);
            } else if p.choice.options.iter().any(|o| o.id == h) {
                a.selected = Some(vec![h.clone()]);
            }
            apply_game(&mut r, &mut steps, s, a);
        } else {
            pass(&mut r, &mut steps)
        }
    }
    assert!(r
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .any(|c| c.definition == "JC104" && !c.face_down && c.id != h));
    assert_eq!(resources(&r, 0), before);
    assert_eq!(r.turn, turn);
    assert_eq!(r.modifiers[0].uses, 1);
    let mut v = result("real-DQJC108-win-free-reveal-keeps-discount", steps);
    v["preparedRealWinWindow"] = json!(true);
    v
}
fn invalid_discount(kind: &str) -> Value {
    let mut g = initial(kind);
    let p = field(&mut g, "JC103", 0);
    let victim = asset(&mut g, "JC125", 2);
    let spell = held(&mut g, "JC107", 0);
    let nonpurple = held(&mut g, "JC125", 0);
    let other = held(&mut g, "JC104", 2);
    fund(&mut g, "JC104", 2, 2);
    if kind == "loyalty" {
        for c in &mut g.players[0].assets {
            c.definition = "JC075".into();
        }
    }
    if kind == "no-money" {
        for c in &mut g.players[0].assets {
            c.exhausted = true;
        }
    }
    let (mut r, mut steps) = room(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        activate(&p, "reduce-next-purple", None),
    );
    drain(&mut r, &mut steps);
    match kind {
        "bad-target" => reject(&mut r, &mut steps, 0, play(&spell, "missing")),
        "loyalty" => reject(&mut r, &mut steps, 0, play(&spell, &victim)),
        "no-money" => reject(&mut r, &mut steps, 0, play(&spell, &victim)),
        "nonpurple" => {
            apply_game(&mut r, &mut steps, 0, deploy(&nonpurple));
            drain(&mut r, &mut steps);
        }
        "other-actor" => {
            advance(&mut r, &mut steps, |r| {
                r.game.legal_actions(2).iter().any(|a| {
                    a.action.card_id.as_deref() == Some(&other) && a.action.kind == "deploy"
                })
            });
            let before = resources(&r, 2);
            apply_game(&mut r, &mut steps, 2, deploy(&other));
            assert_eq!(resources(&r, 2), before - 2);
            drain(&mut r, &mut steps);
        }
        _ => panic!(),
    }
    assert_eq!(r.modifiers[0].uses, 1);
    result(&format!("discount-not-consumed-{kind}"), steps)
}
fn stacking() -> Value {
    let mut g = initial("stacking");
    let p = field(&mut g, "JC103", 0);
    let q = field(&mut g, "JC103", 0);
    let spell = held(&mut g, "JZ67", 0);
    let (mut r, mut steps) = room(g);
    for id in [p, q] {
        apply_game(
            &mut r,
            &mut steps,
            0,
            activate(&id, "reduce-next-purple", None),
        );
        drain(&mut r, &mut steps)
    }
    apply_game(&mut r, &mut steps, 0, play(&spell, "p2"));
    assert_eq!(resources(&r, 0), 4);
    assert!(r.modifiers.iter().all(|m| m.uses == 0));
    drain(&mut r, &mut steps);
    result("stacked-discounts-clamp-zero-consume-next", steps)
}
fn source_return(expire: bool) -> Value {
    let mut g = initial("source-return");
    let p = field(&mut g, "JC103", 2);
    g.regions[0].cards.last_mut().unwrap().controller = 0;
    let rescuer = field(&mut g, "JC075", 0);
    let (mut r, mut steps) = room(g);
    apply_game(
        &mut r,
        &mut steps,
        0,
        activate(&p, "reduce-next-purple", None),
    );
    apply_game(
        &mut r,
        &mut steps,
        0,
        activate(&rescuer, "rescue", Some(&p)),
    );
    drain(&mut r, &mut steps);
    assert!(board(&r, &p).is_none());
    assert!(r.players[2]
        .hand
        .iter()
        .any(|c| c.definition == "JC103" && c.id != p));
    assert_eq!(r.modifiers[0].actor, 0);
    assert_eq!(resources(&r, 0), 2);
    for s in 0..4 {
        let v = r.game.view(s);
        assert_eq!(
            v.hand.iter().any(|c| c.card_id.as_deref() == Some("JC103")),
            s == 2
        );
    }
    if expire {
        let turn = r.turn;
        advance(&mut r, &mut steps, |r| r.turn > turn);
        assert!(r.modifiers.is_empty());
    }
    result(
        if expire {
            "turn-end-expires-discount"
        } else {
            "returned-source-keeps-declared-controller-discount"
        },
        steps,
    )
}
fn asset_seat(seat: usize) -> Value {
    let mut g = initial("asset-seat");
    let victim = asset(&mut g, "XQ17", seat);
    if seat == 2 {
        let c = g.players[2].assets.last_mut().unwrap();
        c.owner = 3;
    }
    let owner = if seat == 2 { 3 } else { seat };
    let spell = held(&mut g, "JC107", 0);
    let (mut r, mut steps) = room(g);
    for s in 0..4 {
        let v = r.game.view(s);
        let a = v.assets.iter().find(|c| c.instance_id == victim).unwrap();
        assert_eq!(a.kind, "asset");
        assert_eq!(a.name, "资产");
        assert!(a.card_id.is_none());
    }
    let a = r
        .game
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.card_id.as_deref() == Some(&spell)
                && a.action.target_id.as_deref() == Some(&victim)
        })
        .unwrap();
    assert!(a.label.contains("资产"));
    assert!(!a.label.contains("祭品"));
    apply_game(&mut r, &mut steps, 0, a.action);
    for s in 0..4 {
        let v = r.game.view(s);
        let t = &v.stack.last().unwrap().target_summaries[0];
        assert_eq!(t.kind, "asset");
        assert_eq!(t.label, "资产");
        assert_eq!(t.owner.as_deref(), Some(format!("p{owner}").as_str()));
        assert!(t.region.is_none());
    }
    while !r.stack.is_empty() {
        assert!(r.pending.is_none());
        pass(&mut r, &mut steps)
    }
    assert!(r.players[owner]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ17" && c.id != victim && c.controller == owner));
    assert!(!r.players[seat].assets.iter().any(|c| c.id == victim));
    result(
        &format!("collapse-seat-{seat}-asset-owner-fresh-instance-privacy"),
        steps,
    )
}
fn invalid_zone(kind: &str) -> Value {
    let mut g = initial(kind);
    let spell = held(&mut g, "JC107", 0);
    let target = match kind {
        "character" => field(&mut g, "JC125", 2),
        "hidden" => {
            let id = field(&mut g, "JC125", 2);
            g.regions[0].cards.last_mut().unwrap().face_down = true;
            id
        }
        "hand" => held(&mut g, "JC125", 2),
        "grave" => {
            let c = g.make_card("JC125", 2);
            let id = c.id.clone();
            g.players[2].graveyard.push(c);
            id
        }
        "region" => "region:0".into(),
        "player" => "p2".into(),
        "missing" => "missing".into(),
        _ => panic!(),
    };
    let (mut r, mut steps) = room(g);
    reject(&mut r, &mut steps, 0, play(&spell, &target));
    result(&format!("collapse-reject-{kind}-atomically"), steps)
}
fn control_attachment() -> Value {
    let mut g = initial("control-attachment");
    let host = field(&mut g, "JC125", 2);
    let leash = held(&mut g, "JC036", 0);
    let spell = held(&mut g, "JC107", 0);
    fund(&mut g, "JC036", 0, 5);
    let (mut r, mut steps) = room(g);
    apply_game(&mut r, &mut steps, 0, play(&leash, &host));
    drain(&mut r, &mut steps);
    let attachment = r.attachments[0].card.id.clone();
    apply_game(&mut r, &mut steps, 0, play(&spell, &attachment));
    drain(&mut r, &mut steps);
    assert!(r.attachments.is_empty());
    assert!(r.control_effects.is_empty());
    assert_eq!(board(&r, &host).unwrap().controller, 2);
    result("collapse-real-control-attachment-cleans-control", steps)
}
fn armor_host(stale: bool) -> Value {
    let mut g = initial("armor-host");
    let host = field(&mut g, if stale { "JC125" } else { "XQ17" }, 2);
    if !stale {
        g.regions[0].cards.last_mut().unwrap().damage = 1;
    }
    let vest = g.make_card("XQ47", 2);
    let old = vest.id.clone();
    g.attachments.push(Attachment {
        card: vest,
        host_id: host.clone(),
    });
    let spell = held(&mut g, "JC107", 0);
    let rescuer = if stale {
        fund(&mut g, "JC075", 2, 2);
        Some(field(&mut g, "JC075", 2))
    } else {
        None
    };
    let (mut r, mut steps) = room(g);
    apply_game(&mut r, &mut steps, 0, play(&spell, &old));
    if let Some(rescuer) = rescuer {
        advance(&mut r, &mut steps, |r| {
            r.priority_team == 1 && r.pending.is_none()
        });
        apply_game(
            &mut r,
            &mut steps,
            2,
            activate(&rescuer, "rescue", Some(&host)),
        );
        drain(&mut r, &mut steps);
        assert!(r.players[2]
            .hand
            .iter()
            .any(|c| c.definition == "JC125" && c.id != host));
    } else {
        advance(&mut r, &mut steps, |r| {
            r.pending
                .as_ref()
                .is_some_and(|p| p.choice.kind == "trigger")
        });
        let ChoiceResolution::Declare { declaration, .. } = &r.pending.as_ref().unwrap().resolution
        else {
            panic!()
        };
        assert_eq!(declaration.actor, 2);
        assert_eq!(declaration.source.card.id, host);
        drain(&mut r, &mut steps);
    }
    assert_eq!(resources(&r, 0), 1);
    assert!(r.attachments.is_empty());
    assert!(board(&r, &host).is_none());
    assert!(r.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ47" && c.id != old));
    result(
        if stale {
            "real-host-rescue-stales-attachment-no-refund"
        } else {
            "destroy-armor-settles-wounded-host-real-death-trigger"
        },
        steps,
    )
}
pub fn cases() -> impl Iterator<Item = Value> {
    let f: Vec<Box<dyn FnOnce() -> Value>> = vec![
        Box::new(painter_play),
        Box::new(|| reveal_scope(false)),
        Box::new(|| reveal_scope(true)),
        Box::new(free_reveal),
        Box::new(|| invalid_discount("bad-target")),
        Box::new(|| invalid_discount("loyalty")),
        Box::new(|| invalid_discount("no-money")),
        Box::new(|| invalid_discount("nonpurple")),
        Box::new(|| invalid_discount("other-actor")),
        Box::new(stacking),
        Box::new(|| source_return(false)),
        Box::new(|| source_return(true)),
        Box::new(|| asset_seat(0)),
        Box::new(|| asset_seat(1)),
        Box::new(|| asset_seat(2)),
        Box::new(|| asset_seat(3)),
        Box::new(|| invalid_zone("character")),
        Box::new(|| invalid_zone("hidden")),
        Box::new(|| invalid_zone("hand")),
        Box::new(|| invalid_zone("grave")),
        Box::new(|| invalid_zone("region")),
        Box::new(|| invalid_zone("player")),
        Box::new(|| invalid_zone("missing")),
        Box::new(control_attachment),
        Box::new(|| armor_host(false)),
        Box::new(|| armor_host(true)),
    ];
    f.into_iter().map(|f| f())
}
