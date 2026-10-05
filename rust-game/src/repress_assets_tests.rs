//! Disclosed boundary layouts; new declarations and responses use real actions.
use crate::{catalog, model::*, rules::*};

fn game() -> Game {
    let mut g = Game::new(
        "repress-assets-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        8,
    )
    .unwrap();
    for s in 1..4 {
        g.join(format!("P{s}"), "watchers".into()).unwrap();
    }
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
    while g.pending.is_some() {
        choose(&mut g, vec![]);
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.cards.clear();
        r.influence = [0; 2];
    }
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    g
}
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
fn fund(g: &mut Game, seat: usize, id: &str, n: usize) {
    for _ in 0..n {
        let c = g.make_card(id, seat);
        g.players[seat].assets.push(c);
    }
}
fn restore(g: &mut Game) {
    let state = serde_json::to_string(g).unwrap();
    let views = (0..4)
        .map(|s| serde_json::to_value(g.view(s)).unwrap())
        .collect::<Vec<_>>();
    *g = Game::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), state);
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
    let before = serde_json::to_string(g).unwrap();
    assert!(g.apply(s, a).is_err());
    assert_eq!(serde_json::to_string(g).unwrap(), before);
    restore(g);
}
fn choose(g: &mut Game, ids: Vec<String>) {
    let p = g.pending.clone().unwrap();
    act(
        g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(ids),
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
fn resolve_choice(g: &mut Game) {
    let p = g.pending.clone().unwrap();
    if matches!(
        p.resolution,
        ChoiceResolution::Forecast { .. }
            | ChoiceResolution::Frame {
                choice: FrameChoice::Forecast { .. },
                ..
            }
    ) {
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                top: Some(ids),
                bottom: Some(vec![]),
                ..Action::new("choose")
            },
        );
    } else if matches!(p.resolution, ChoiceResolution::Bottom { .. }) {
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                bottom: Some(ids),
                ..Action::new("choose")
            },
        );
    } else if matches!(p.resolution, ChoiceResolution::Damage { .. }) {
        let mut allocations = std::collections::BTreeMap::new();
        allocations.insert(p.choice.options[0].id.clone(), p.choice.amount.unwrap());
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                allocations: Some(allocations),
                ..Action::new("choose")
            },
        );
    } else {
        let ids = p
            .choice
            .options
            .iter()
            .take(p.choice.min.unwrap_or(0))
            .map(|o| o.id.clone())
            .collect();
        choose(g, ids);
    }
}
fn play_card(g: &mut Game, seat: usize, id: &str) {
    let kind = if g.players[seat]
        .hand
        .iter()
        .find(|c| c.id == id)
        .is_some_and(|c| catalog::card(&c.definition).kind == "character")
    {
        "deploy"
    } else {
        "play"
    };
    act(
        g,
        seat,
        Action {
            card_id: Some(id.into()),
            region: Some(0),
            ..Action::new(kind)
        },
    );
}
fn until_choice(g: &mut Game, kind: &str) {
    for _ in 0..80 {
        if g.pending.as_ref().is_some_and(|p| p.choice.kind == kind) {
            return;
        }
        if g.pending.is_some() {
            resolve_choice(g);
        } else {
            pass(g);
        }
    }
    panic!("missing {kind}");
}
fn play_target(g: &mut Game, id: &str, target: &str) {
    act(
        g,
        0,
        Action {
            card_id: Some(id.into()),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    );
}
fn resources(g: &Game, s: usize) -> usize {
    g.players[s].assets.iter().filter(|c| !c.exhausted).count()
}
fn drain(g: &mut Game) {
    for _ in 0..100 {
        if g.stack.is_empty() && g.pending.is_none() {
            return;
        }
        if g.pending.is_some() {
            resolve_choice(g);
        } else {
            pass(g);
        }
    }
    panic!("bounded drain");
}

#[test]
fn repress_assets_original_fields_and_card_bound_program_limits() {
    for (id, name, cost, loyalty, subtype, timing) in [
        ("JZ74", "意外事故", 1, vec![], "突发状况", Timing::Fast),
        (
            "JC126",
            "产业扩张",
            3,
            vec!["蓝色"],
            "策略",
            Timing::Standard,
        ),
    ] {
        let d = catalog::card(id);
        assert_eq!(
            (&*d.name, d.cost, &*d.color, &*d.magic),
            (name, cost, "中立", "")
        );
        assert_eq!(d.loyalty, loyalty);
        assert_eq!(d.subtypes, [subtype]);
        assert_eq!(d.kind, "spell");
        assert_eq!(d.defense, None);
        assert!(!d.unique && d.keywords.is_empty());
        assert_eq!(d.permanent_icons, Icons::default());
        assert_eq!(d.temporary_icons, Icons::default());
        let spec = &definition(id).abilities[0];
        assert_eq!(spec.timing, timing);
        assert!(spec.play_only && spec.event.is_none() && spec.costs.is_empty());
        validate_ability(id, spec).unwrap();
        assert!(validate_ability("JC125", spec).is_err());
        let mut bad = spec.clone();
        bad.play_only = false;
        assert!(validate_ability(id, &bad).is_err());
        bad = spec.clone();
        bad.ops.push(spec.ops[0].clone());
        assert!(validate_ability(id, &bad).is_err());
    }
    let mut bad = definition("JZ74").abilities[0].clone();
    bad.ops.swap(0, 1);
    assert!(validate_ability("JZ74", &bad).is_err());
    bad = definition("JZ74").abilities[0].clone();
    bad.targets[0].relation = Relation::FriendlyTeam;
    assert!(validate_ability("JZ74", &bad).is_err());
    bad = definition("JZ74").abilities[0].clone();
    bad.ops[1] = Op::Draw {
        player: PlayerRef::Target(0),
        count: 1,
        end: DeckEnd::Top,
    };
    assert!(validate_ability("JZ74", &bad).is_err());
}

#[test]
fn repress_assets_accident_target_player_chooses_before_actor_draws_all_four_seats() {
    for actor in 0..4 {
        let mut g = game();
        let enemy = if actor < 2 { 2 } else { 0 };
        let team = if enemy < 2 { 0 } else { 1 };
        g.begin_window(Window::Action(1 - team));
        fund(&mut g, actor, "JC125", 2);
        let source = held(&mut g, "JZ74", actor);
        g.regions[2].influence[team] = 2;
        g.regions[4].influence[team] = 1;
        let top = g.players[actor].deck[0].clone();
        let enemy_hand = g.players[enemy].hand.len();
        act(
            &mut g,
            actor,
            Action {
                card_id: Some(source.clone()),
                target_id: Some(format!("p{enemy}")),
                ..Action::new("play")
            },
        );
        assert_eq!(resources(&g, actor), 1);
        until_choice(&mut g, "target");
        assert_eq!(g.pending.as_ref().unwrap().seat, enemy);
        assert!(g.players[actor].hand.is_empty());
        assert_eq!(g.players[actor].deck[0].id, top.id);
        for s in 0..4 {
            assert_eq!(g.view(s).pending_choice.is_some(), s == enemy);
        }
        let p = g.pending.clone().unwrap();
        reject(
            &mut g,
            actor,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec!["region:4".into()]),
                ..Action::new("choose")
            },
        );
        reject(
            &mut g,
            enemy,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        );
        choose(&mut g, vec!["region:4".into()]);
        drain(&mut g);
        assert_eq!(g.regions[2].influence[team], 2);
        assert_eq!(g.regions[4].influence[team], 0);
        assert_eq!(g.players[actor].hand.len(), 1);
        assert_eq!(g.players[actor].hand[0].definition, top.definition);
        assert_ne!(g.players[actor].hand[0].id, top.id);
        assert_eq!(g.players[enemy].hand.len(), enemy_hand);
        assert_eq!(resources(&g, actor), 1);
        assert!(g.players[actor]
            .graveyard
            .iter()
            .any(|c| c.definition == "JZ74" && c.id != source));
    }
}

#[test]
fn repress_assets_accident_no_influence_still_draws_and_empty_draw_eliminates() {
    for empty in [false, true] {
        let mut g = game();
        fund(&mut g, 0, "JC125", 1);
        let source = held(&mut g, "JZ74", 0);
        if empty {
            g.players[0].deck.clear();
        }
        play_target(&mut g, &source, "p3");
        drain(&mut g);
        assert_eq!(g.players[0].eliminated, empty);
        assert_eq!(g.players[0].hand.len(), usize::from(!empty));
        assert!(g.regions.iter().all(|r| r.influence == [0, 0]));
    }
}

#[test]
fn repress_assets_accident_invalid_bindings_are_atomic_and_no_neutral_loyalty() {
    let mut g = game();
    let source = held(&mut g, "JZ74", 0);
    fund(&mut g, 1, "JC125", 2);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(source.clone()),
            target_id: Some("p2".into()),
            ..Action::new("play")
        },
    );
    fund(&mut g, 0, "JC125", 1);
    for target in [None, Some("p0"), Some("p1"), Some("p8")] {
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                target_id: target.map(str::to_string),
                ..Action::new("play")
            },
        );
    }
    play_target(&mut g, &source, "p2");
    drain(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}

#[test]
fn repress_assets_expansion_blue_loyalty_personal_currency_and_standard_timing() {
    let mut g = game();
    let source = held(&mut g, "JC126", 0);
    fund(&mut g, 0, "JC125", 3);
    fund(&mut g, 1, "JC036", 3);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(source.clone()),
            ..Action::new("play")
        },
    );
    fund(&mut g, 0, "JC036", 1);
    let response = held(&mut g, "JZ74", 0);
    play_target(&mut g, &response, "p2");
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(source.clone()),
            ..Action::new("play")
        },
    );
    drain(&mut g);
    play_card(&mut g, 0, &source);
    assert_eq!(resources(&g, 0), 0);
    drain(&mut g);
    assert_eq!(resources(&g, 0), 1);
    assert_eq!(
        g.players[1].assets.iter().filter(|c| !c.exhausted).count(),
        3
    );
}

#[test]
fn repress_assets_expansion_fresh_public_asset_all_seats_no_entry_or_character_traits() {
    for actor in 0..4 {
        for top_def in ["JC104", "JZ58", "LC23", "JC126"] {
            let mut g = game();
            g.begin_window(Window::Action(if actor < 2 { 0 } else { 1 }));
            fund(&mut g, actor, "JC036", 3);
            let source = held(&mut g, "JC126", actor);
            let mut top = g.make_card(top_def, actor);
            let old = top.id.clone();
            top.face_down = true;
            top.exhausted = true;
            top.damage = 7;
            top.wounds = 4;
            top.shield = 2;
            g.players[actor].deck.insert(0, top);
            let deck = g.players[actor].deck.len();
            play_card(&mut g, actor, &source);
            drain(&mut g);
            assert_eq!(g.players[actor].deck.len(), deck - 1);
            assert!(g.players[actor].hand.is_empty() && !g.players[actor].eliminated);
            let a = g.players[actor].assets.last().unwrap();
            assert_eq!(a.definition, top_def);
            assert_ne!(a.id, old);
            assert_eq!(
                (
                    a.owner,
                    a.controller,
                    a.face_down,
                    a.exhausted,
                    a.damage,
                    a.wounds,
                    a.shield
                ),
                (actor, actor, false, false, 0, 0, 0)
            );
            assert!(!g.players[actor].asset_used);
            for s in 0..4 {
                let projection = g.view(s);
                let v = projection
                    .assets
                    .iter()
                    .find(|v| v.instance_id == a.id)
                    .unwrap();
                assert!(v.card_id.is_none());
                assert_eq!(v.kind, "asset");
                assert!(
                    v.text.is_none()
                        && v.icons.is_none()
                        && v.defense.is_none()
                        && v.current_spirit_protection.is_none()
                );
                assert_eq!(
                    v.color.as_deref(),
                    Some(catalog::card(top_def).color.as_str())
                );
                assert_eq!(
                    v.magic.as_deref(),
                    Some(catalog::card(top_def).magic.as_str())
                );
            }
        }
    }
}

#[test]
fn repress_assets_expansion_preserves_build_flag_in_both_directions_and_multiple_spells() {
    for prior_build in [false, true] {
        let mut g = game();
        fund(&mut g, 0, "JC036", 6);
        let hand_asset = held(&mut g, "JC125", 0);
        if prior_build {
            act(
                &mut g,
                0,
                Action {
                    card_id: Some(hand_asset.clone()),
                    ..Action::new("asset")
                },
            );
        }
        for _ in 0..2 {
            let source = held(&mut g, "JC126", 0);
            play_card(&mut g, 0, &source);
            drain(&mut g);
            assert_eq!(g.players[0].asset_used, prior_build);
        }
        assert_eq!(g.players[0].assets.len(), 8 + usize::from(prior_build));
        let a = Action {
            card_id: Some(hand_asset),
            ..Action::new("asset")
        };
        if prior_build {
            reject(&mut g, 0, a);
        } else {
            act(&mut g, 0, a);
        }
    }
}

#[test]
fn repress_assets_expansion_empty_deck_does_not_draw_or_eliminate() {
    let mut g = game();
    fund(&mut g, 0, "JC036", 3);
    let source = held(&mut g, "JC126", 0);
    g.players[0].deck.clear();
    play_card(&mut g, 0, &source);
    drain(&mut g);
    assert_eq!(g.players[0].assets.len(), 3);
    assert_eq!(resources(&g, 0), 0);
    assert!(!g.players[0].eliminated && g.players[0].hand.is_empty());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC126"));
}

#[test]
fn repress_assets_expansion_resolves_current_top_after_real_paid_forecast_reveal() {
    let mut g = game();
    fund(&mut g, 0, "JC036", 5);
    fund(&mut g, 0, "JC104", 1);
    let source = held(&mut g, "JC126", 0);
    let hidden = field(&mut g, "JC104", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    let first = g.players[0].deck[0].id.clone();
    let second = g.players[0].deck[1].clone();
    play_card(&mut g, 0, &source);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    until_choice(&mut g, "trigger");
    choose(&mut g, vec!["accept".into()]);
    until_choice(&mut g, "investigation");
    let p = g.pending.clone().unwrap();
    let remaining = p
        .choice
        .options
        .iter()
        .filter(|o| o.id != second.id)
        .map(|o| o.id.clone())
        .collect();
    act(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id),
            top: Some(vec![second.id.clone()]),
            bottom: Some(remaining),
            ..Action::new("choose")
        },
    );
    drain(&mut g);
    let asset = g.players[0].assets.last().unwrap();
    assert_eq!(asset.definition, second.definition);
    assert_ne!(asset.id, second.id);
    assert!(g.players[0].deck.iter().any(|c| c.id == first));
    assert!(!g.players[0].deck.iter().any(|c| c.id == second.id));
    assert_eq!(resources(&g, 0), 2);
}

#[test]
fn repress_assets_accident_guard_cancels_whole_program_after_real_target_elimination() {
    let mut g = game();
    fund(&mut g, 0, "JC125", 2);
    fund(&mut g, 2, "JC125", 1);
    let a = held(&mut g, "JZ74", 0);
    let b = held(&mut g, "JZ74", 2);
    g.players[2].deck.clear();
    play_target(&mut g, &a, "p2");
    act(&mut g, 0, Action::new("pass"));
    act(&mut g, 1, Action::new("pass"));
    act(
        &mut g,
        2,
        Action {
            card_id: Some(b),
            target_id: Some("p0".into()),
            ..Action::new("play")
        },
    );
    drain(&mut g);
    assert!(g.players[2].eliminated);
    assert!(g.players[0].hand.is_empty());
    assert_eq!(resources(&g, 0), 1);
    assert!(g.log.iter().any(|l| l.text.contains("效果取消")));
}

#[test]
fn repress_assets_expansion_asset_supplies_ready_currency_and_live_color_domain() {
    let mut g = game();
    fund(&mut g, 0, "JC036", 4);
    fund(&mut g, 0, "JC125", 2);
    g.players[0].assets[4].exhausted = true;
    g.players[0].assets[5].exhausted = true;
    let top = g.make_card("JC008", 0);
    g.players[0].deck.insert(0, top);
    let host = field(&mut g, "JC125", 2);
    let attachment = g.make_card("BQ022", 2);
    let target = attachment.id.clone();
    g.attachments.push(Attachment {
        card: attachment,
        host_id: host,
    });
    let c = held(&mut g, "JC126", 0);
    let disintegrate = held(&mut g, "JC005", 0);
    assert!(!g.actor_has_asset_domain(0, &MagicIcon::Mind, 1));
    play_card(&mut g, 0, &c);
    drain(&mut g);
    assert!(g.loyalty(0, "JC005"));
    assert_eq!(resources(&g, 0), 2);
    // The real two-cost spell consumes one old asset and the new asset.
    let buff = held(&mut g, "JC008", 0);
    let own = field(&mut g, "JC125", 0);
    play_target(&mut g, &buff, &own);
    drain(&mut g);
    assert_eq!(resources(&g, 0), 0);
    assert!(g.players[0].assets.last().unwrap().exhausted);
    // Exhaustion does not remove either printed loyalty or the Mind domain.
    assert!(g.loyalty(0, "JC005"));
    assert!(g.actor_has_asset_domain(0, &MagicIcon::Mind, 1));
    assert!(!g.actor_has_asset_domain(1, &MagicIcon::Mind, 1));
    for _ in 0..500 {
        if g.legal_actions(0).iter().any(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&disintegrate)
                && a.action.target_id.as_deref() == Some(&target)
        }) {
            break;
        }
        if g.pending.is_some() {
            resolve_choice(&mut g);
        } else {
            pass(&mut g);
        }
    }
    play_target(&mut g, &disintegrate, &target);
    drain(&mut g);
    assert!(g.attachments.is_empty());
}
