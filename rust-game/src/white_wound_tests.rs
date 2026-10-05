//! Printed LC06; initial boundary layouts are disclosed, all effects use real actions.
use crate::{catalog, model::*, rules::*};

fn initial() -> Game {
    let mut g = Game::new(
        "lc06-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        9,
    )
    .unwrap();
    for s in 1..4 {
        g.join(format!("P{s}"), "watchers".into()).unwrap();
    }
    for s in 0..4 {
        g.apply(s, Action::new("ready")).unwrap();
    }
    g.apply(0, Action::new("start")).unwrap();
    while g.pending.is_some() {
        choose(&mut g, vec![]);
    }
    for s in 0..4 {
        g.players[s].hand.clear();
        g.players[s].assets.clear();
        g.players[s].deck.clear();
        for _ in 0..12 {
            let c = g.make_card("JC125", s);
            g.players[s].deck.push(c);
        }
    }
    for r in &mut g.regions {
        r.cards.clear();
        r.influence = [0, 0];
    }
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    fund(&mut g, 0, "JC091", 6);
    g
}
fn fund(g: &mut Game, s: usize, definition: &str, count: usize) {
    for _ in 0..count {
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
fn restore(g: &mut Game) {
    let raw = serde_json::to_string(g).unwrap();
    let views = (0..4)
        .map(|s| serde_json::to_value(g.view(s)).unwrap())
        .collect::<Vec<_>>();
    *g = Game::from_persisted(&raw).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), raw);
    for s in 0..4 {
        assert_eq!(serde_json::to_value(g.view(s)).unwrap(), views[s]);
    }
}
fn act(g: &mut Game, s: usize, a: Action) {
    restore(g);
    g.apply(s, a).unwrap();
    restore(g);
}
fn reject(g: &mut Game, s: usize, a: Action) {
    let raw = serde_json::to_string(g).unwrap();
    assert!(g.apply(s, a).is_err());
    assert_eq!(serde_json::to_string(g).unwrap(), raw);
    restore(g);
}
fn choose(g: &mut Game, selected: Vec<String>) {
    let p = g.pending.clone().unwrap();
    act(
        g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(selected),
            ..Action::new("choose")
        },
    );
}
fn pass(g: &mut Game) {
    let s = (0..4)
        .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
        .unwrap();
    act(g, s, Action::new("pass"));
}
fn until_trigger(g: &mut Game, definition: &str) {
    for _ in 0..60 {
        if g.pending.as_ref().is_some_and(|p| matches!(&p.resolution, ChoiceResolution::Declare { declaration, .. } if declaration.source.card.definition == definition)) { return; }
        assert!(g.pending.is_none(), "unexpected pending choice");
        pass(g);
    }
    panic!("missing trigger {definition}");
}
fn drain(g: &mut Game) {
    for _ in 0..100 {
        if g.stack.is_empty() && g.pending.is_none() {
            return;
        }
        assert!(g.pending.is_none(), "unexpected pending choice");
        pass(g);
    }
    panic!("stack drain bound");
}
fn play(g: &mut Game, definition: &str, target: &str) {
    let id = held(g, definition, 0);
    act(
        g,
        0,
        Action {
            card_id: Some(id),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    );
}
fn wound(g: &mut Game, target: &str) {
    let jz = field(g, "JZ59", 0, 0);
    play(g, "JC091", &jz);
    until_trigger(g, "JZ59");
    choose(g, vec![target.into()]);
}
fn activate(id: &str, key: &str, target: Option<&str>) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(key.into()),
        target_id: target.map(str::to_string),
        ..Action::new("activate")
    }
}

#[test]
fn lc06_whole_original_and_finite_wound_event_only() {
    let c = catalog::card("LC06");
    assert_eq!(
        (&*c.name, c.subtitle.as_deref(), &*c.color),
        ("瓦尔德修士", Some("苦行先知"), "白")
    );
    assert_eq!(c.subtypes, ["人类", "僧侣"]);
    assert_eq!(c.cost, 3);
    assert_eq!(c.loyalty, ["白色", "白色"]);
    assert_eq!(c.defense, Some(2));
    assert!(c.unique && c.magic == "神圣" && c.keywords.is_empty());
    assert_eq!(
        c.permanent_icons,
        Icons {
            investigation: 1,
            combat: 0,
            influence: 1
        }
    );
    assert_eq!(c.temporary_icons, Icons::default());
    assert_eq!(c.deck_copy_limit, Some(3));
    let a = crate::rules::definition("LC06").abilities[0].clone();
    assert_eq!(a.event, Some(Event::ReceiveWound));
    assert!(crate::rules::validate_ability("LC06", &a).is_ok());
    assert!(crate::rules::validate_ability("fixture", &a).is_err());
    for bad in [
        "cost",
        "count",
        "ready",
        "immediate",
        "manual",
        "mode",
        "quota",
    ] {
        let mut wrong = a.clone();
        match bad {
            "cost" => wrong.costs.push(Cost::Assets(1)),
            "count" => {
                wrong.ops = vec![Op::Draw {
                    player: PlayerRef::Actor,
                    count: 1,
                    end: DeckEnd::Top,
                }]
            }
            "ready" => wrong.requires_ready_source = true,
            "immediate" => wrong.response_policy = ResponsePolicy::Immediate,
            "manual" => wrong.activation_only = true,
            "mode" => wrong.play_only = true,
            "quota" => wrong.once_per_game = true,
            _ => unreachable!(),
        }
        assert!(
            crate::rules::validate_ability("LC06", &wrong).is_err(),
            "{bad}"
        );
    }
}
#[test]
fn lc06_real_wound_controller_snapshot_fatal_and_four_private_views() {
    for (owner, controller, fatal) in [(0, 0, false), (3, 2, false), (3, 2, true)] {
        let mut g = initial();
        let lc = field(&mut g, "LC06", owner, controller);
        if fatal {
            g.board_mut(&lc).unwrap().wounds = 1;
        }
        let hand = g.players[controller].hand.len();
        let deck = g.players[controller].deck.len();
        wound(&mut g, &lc);
        until_trigger(&mut g, "LC06");
        assert_eq!(g.players[controller].hand.len(), hand);
        let p = g.pending.clone().unwrap();
        assert_eq!(p.seat, controller);
        if let ChoiceResolution::Declare { declaration, .. } = &p.resolution {
            assert_eq!(declaration.source.card.id, lc);
            assert_eq!(declaration.source.card.controller, controller);
            assert_eq!(declaration.source.card.owner, owner);
            assert_eq!(declaration.source.region, Some(0));
        } else {
            panic!("real LC06 declaration");
        }
        for v in 0..4 {
            assert_eq!(g.view(v).pending_choice.is_some(), v == controller);
        }
        if fatal {
            assert!(g.board(&lc).is_none());
            let c = g.players[owner]
                .graveyard
                .iter()
                .find(|c| c.definition == "LC06")
                .unwrap();
            assert_ne!(c.id, lc);
            assert_eq!(
                (c.owner, c.controller, c.wounds, c.damage),
                (owner, owner, 0, 0)
            );
        } else {
            assert_eq!(g.board(&lc).unwrap().1.wounds, 1);
        }
        choose(&mut g, vec!["accept".into()]);
        assert_eq!(
            g.stack.last().unwrap().frame.as_ref().unwrap().actor,
            controller
        );
        assert_eq!(g.players[controller].hand.len(), hand);
        drain(&mut g);
        assert_eq!(g.players[controller].hand.len(), hand + 2);
        assert_eq!(g.players[controller].deck.len(), deck - 2);
        for other in 0..4 {
            if other != controller {
                assert!(g.players[other]
                    .hand
                    .iter()
                    .all(|c| c.definition != "JC125"));
            }
        }
    }
}
#[test]
fn lc06_damage_and_destroy_do_not_trigger_and_no_manual_activation() {
    for fatal in [false, true] {
        let mut g = initial();
        let lc = field(&mut g, "LC06", 0, 0);
        reject(&mut g, 0, activate(&lc, "wound-draw-two", None));
        if fatal {
            play(&mut g, "JC091", &lc);
        } else {
            fund(&mut g, 0, "JC102", 3);
            play(&mut g, "JC102", &lc);
        }
        drain(&mut g);
        assert!(g.players[0].hand.iter().all(|c| c.definition != "JC125"));
        if fatal {
            assert!(g.board(&lc).is_none());
        } else {
            assert_eq!(
                (
                    g.board(&lc).unwrap().1.damage,
                    g.board(&lc).unwrap().1.wounds
                ),
                (1, 0)
            );
        }
    }
}
#[test]
fn lc06_decline_and_declared_source_rescue_preserve_independent_draw_frame() {
    for decline in [false, true] {
        let mut g = initial();
        let lc = field(&mut g, "LC06", 0, 0);
        let rescue = field(&mut g, "JC075", 0, 0);
        fund(&mut g, 0, "JC075", 2);
        wound(&mut g, &lc);
        until_trigger(&mut g, "LC06");
        choose(
            &mut g,
            if decline {
                vec![]
            } else {
                vec!["accept".into()]
            },
        );
        if !decline {
            act(&mut g, 0, activate(&rescue, "rescue", Some(&lc)));
        }
        drain(&mut g);
        assert_eq!(
            g.players[0]
                .hand
                .iter()
                .filter(|c| c.definition == "JC125")
                .count(),
            if decline { 0 } else { 2 }
        );
        if !decline {
            assert!(g.board(&lc).is_none());
            let c = g.players[0]
                .hand
                .iter()
                .find(|c| c.definition == "LC06")
                .unwrap();
            assert_ne!(c.id, lc);
            assert_eq!((c.wounds, c.damage), (0, 0));
        }
    }
}
#[test]
fn lc06_rescue_before_wound_cancels_stale_target_and_emits_no_event() {
    let mut g = initial();
    let lc = field(&mut g, "LC06", 0, 0);
    let rescue = field(&mut g, "JC075", 0, 0);
    fund(&mut g, 0, "JC075", 2);
    wound(&mut g, &lc);
    act(&mut g, 0, activate(&rescue, "rescue", Some(&lc)));
    drain(&mut g);
    assert!(g.players[0].hand.iter().all(|c| c.definition != "JC125"));
    assert_eq!(
        g.players[0]
            .hand
            .iter()
            .find(|c| c.definition == "LC06")
            .unwrap()
            .wounds,
        0
    );
}
#[test]
fn lc06_wounds_ignore_real_damage_prevention_and_repeat_without_ready_cost() {
    let mut g = initial();
    let lc = field(&mut g, "LC06", 0, 0);
    fund(&mut g, 0, "JC075", 1);
    play(&mut g, "JC078", &lc);
    drain(&mut g);
    g.board_mut(&lc).unwrap().exhausted = true;
    for n in 1..=2 {
        wound(&mut g, &lc);
        until_trigger(&mut g, "LC06");
        choose(&mut g, vec!["accept".into()]);
        drain(&mut g);
        assert_eq!(
            g.players[0]
                .hand
                .iter()
                .filter(|c| c.definition == "JC125")
                .count(),
            n * 2
        );
    }
    assert!(g.board(&lc).is_none());
}
#[test]
fn lc06_empty_draw_deck_uses_existing_elimination_without_replaying_wound() {
    let mut g = initial();
    let lc = field(&mut g, "LC06", 0, 0);
    g.players[0].deck.truncate(1);
    wound(&mut g, &lc);
    until_trigger(&mut g, "LC06");
    choose(&mut g, vec!["accept".into()]);
    drain(&mut g);
    assert!(g.players[0].eliminated);
    assert!(g.players[0].hand.is_empty());
    restore(&mut g);
}

#[test]
fn lc06_holy_assets_keep_only_color_domain_and_real_healing_does_not_trigger() {
    let mut g = initial();
    g.players[0].assets.clear();
    fund(&mut g, 0, "LC06", 3);
    assert!(g.actor_has_asset_domain(0, &MagicIcon::Other("神圣".into()), 3));
    assert!(!g.actor_has_asset_domain(1, &MagicIcon::Other("神圣".into()), 1));
    let lc = field(&mut g, "LC06", 0, 0);
    g.board_mut(&lc).unwrap().wounds = 1;
    let healer = field(&mut g, "LC19", 0, 0);
    act(&mut g, 0, activate(&healer, "heal", Some(&lc)));
    drain(&mut g);
    assert_eq!(g.board(&lc).unwrap().1.wounds, 0);
    assert!(g.players[0].hand.is_empty());
    assert_eq!(
        g.players[0].assets.iter().filter(|c| c.exhausted).count(),
        2
    );
    assert!(g.pending.is_none());
}
#[test]
fn lc06_gold_same_name_uses_controller_and_real_paid_enter_sacrifice() {
    for duplicate in [false, true] {
        let mut g = initial();
        g.players[0].assets.clear();
        fund(&mut g, 0, "JC075", 3);
        let old = field(&mut g, "LC06", 3, if duplicate { 0 } else { 1 });
        let new = held(&mut g, "LC06", 0);
        act(
            &mut g,
            0,
            Action {
                card_id: Some(new.clone()),
                region: Some(0),
                ..Action::new("deploy")
            },
        );
        if duplicate {
            for _ in 0..40 {
                if g.pending.is_some() {
                    break;
                }
                pass(&mut g);
            }
            let p = g.pending.clone().unwrap();
            assert_eq!(p.seat, 0);
            assert_eq!(p.choice.options.len(), 2);
            assert!(p.choice.title.contains("同名独有"));
            choose(&mut g, vec![old.clone()]);
            assert!(g.board(&old).is_none());
            assert!(g.players[3]
                .graveyard
                .iter()
                .any(|c| c.definition == "LC06" && c.id != old));
        } else {
            drain(&mut g);
            assert!(g.board(&old).is_some());
        }
        let entered = g
            .regions
            .iter()
            .flat_map(|r| &r.cards)
            .find(|c| c.definition == "LC06" && c.owner == 0)
            .unwrap();
        assert_ne!(entered.id, new);
        assert_eq!(
            g.players[0].assets.iter().filter(|c| c.exhausted).count(),
            3
        );
        assert!(g.pending.is_none());
    }
}
