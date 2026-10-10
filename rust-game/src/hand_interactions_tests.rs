//! Explicit starting layouts; declarations, costs, responses and restoration use real actions.
use crate::unit_support::*;
use crate::{catalog, model::*, rules::*};

fn activation(id: &str, key: &str, target: &str, cost: Option<&str>) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(key.into()),
        target_id: Some(target.into()),
        cost_selected: cost.map(|s| vec![s.into()]),
        ..Action::new("activate")
    }
}
#[test]
fn hand_interactions_original_fields() {
    for (id, cost, loyalty, magic, unique) in [
        ("JC114", 2, vec![], "", false),
        ("XQ34", 1, vec!["白色"], "", false),
        ("XQ38", 3, vec!["黑色", "黑色"], "", true),
        ("JZ67", 1, vec!["紫色"], "心灵", false),
    ] {
        let d = catalog::card(id);
        assert_eq!(d.cost, cost);
        assert_eq!(d.loyalty, loyalty);
        assert_eq!(d.magic, magic);
        assert_eq!(d.unique, unique);
        for a in &definition(id).abilities {
            validate_ability(id, a).unwrap();
        }
    }
    assert_eq!(
        catalog::card("JC114").subtitle.as_deref(),
        Some("无知而无畏")
    );
    assert_eq!(
        catalog::card("XQ38").subtitle.as_deref(),
        Some("死灵学专家")
    );
    assert_eq!(catalog::card("XQ38").temporary_icons.investigation, 2);
    assert_eq!(catalog::card("XQ38").permanent_icons.influence, 1);
    assert_eq!(catalog::card("JC114").temporary_icons.investigation, 1);
}
#[test]
fn hand_interactions_hacker_all_targets_reveal_four_views_optional_sacrifice() {
    for s in 0..4 {
        for accept in [false, true] {
            let mut g = game();
            let h = field(&mut g, "JC114", 0);
            fund(&mut g, 0, "JC075", 1);
            let id = held(&mut g, "JC003", s);
            let before = resources(&g, 0);
            act(
                &mut g,
                0,
                activation(&h, "reveal-hand-discard", &format!("p{s}"), None),
            );
            assert_eq!(resources(&g, 0), before - 1);
            assert!(g.board(&h).unwrap().1.exhausted);
            until_choice(&mut g, "revealed-hand-discard");
            for v in 0..4 {
                let view = g.view(v);
                assert_eq!(view.revealed_hands.len(), 1);
                assert_eq!(view.revealed_hands[0].cards[0].instance_id, id);
                assert_eq!(
                    view.revealed_hands[0].cards[0].card_id.as_deref(),
                    Some("JC003")
                );
                assert_eq!(view.pending_choice.is_some(), v == 0);
            }
            let cid = g.pending.as_ref().unwrap().choice.id.clone();
            reject(
                &mut g,
                1,
                Action {
                    choice_id: Some(cid),
                    selected: Some(vec![id.clone()]),
                    ..Action::new("choose")
                },
            );
            choose(&mut g, if accept { vec![id.clone()] } else { vec![] });
            drain(&mut g);
            for v in 0..4 {
                assert!(g.view(v).revealed_hands.is_empty());
            }
            if accept {
                assert!(g.board(&h).is_none());
                assert!(g.players[0]
                    .graveyard
                    .iter()
                    .any(|c| c.definition == "JC114" && c.id != h));
                assert!(g.players[s]
                    .graveyard
                    .iter()
                    .any(|c| c.definition == "JC003" && c.id != id));
            } else {
                assert!(g.board(&h).is_some());
                assert!(g.players[s].hand.iter().any(|c| c.id == id));
            }
        }
    }
}
#[test]
fn hand_interactions_hacker_empty_hand_and_real_source_return() {
    let mut g = game();
    let h = field(&mut g, "JC114", 0);
    fund(&mut g, 0, "JC075", 2);
    act(&mut g, 0, activation(&h, "reveal-hand-discard", "p2", None));
    drain(&mut g);
    assert!(g.board(&h).unwrap().1.exhausted);
    assert!(g.view(1).revealed_hands.is_empty());
    let mut g = game();
    let h = field(&mut g, "JC114", 0);
    let target = held(&mut g, "JC003", 2);
    let rescue = field(&mut g, "JC075", 0);
    fund(&mut g, 0, "JC075", 3);
    fund(&mut g, 2, "JC075", 2);
    act(&mut g, 0, activation(&h, "reveal-hand-discard", "p2", None));
    act(
        &mut g,
        0,
        Action {
            ability_id: Some("rescue".into()),
            card_id: Some(rescue),
            target_id: Some(h.clone()),
            ..Action::new("activate")
        },
    );
    until_choice(&mut g, "revealed-hand-discard");
    assert!(g.board(&h).is_none());
    assert!(g.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "JC114" && c.id != h));
    assert_eq!(g.pending.as_ref().unwrap().choice.max, Some(0));
    let cid = g.pending.as_ref().unwrap().choice.id.clone();
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(cid),
            selected: Some(vec![target.clone()]),
            ..Action::new("choose")
        },
    );
    choose(&mut g, vec![]);
    assert!(g.players[2].hand.iter().any(|c| c.id == target));
}
#[test]
fn hand_interactions_inspiration_forced_discard_holder_owner_accept_skip_and_no_funds() {
    for (accept, n) in [(true, 1), (false, 1), (true, 0)] {
        let mut g = game();
        let h = field(&mut g, "JC114", 0);
        fund(&mut g, 0, "JC075", 1);
        fund(&mut g, 2, "JC075", n);
        let id = held(&mut g, "XQ34", 2);
        g.players[2].hand[0].owner = 3; // Explicit foreign-owned hand boundary.
        deck_cards(&mut g, 2, &["JC003", "JC125"]);
        act(&mut g, 0, activation(&h, "reveal-hand-discard", "p2", None));
        until_choice(&mut g, "revealed-hand-discard");
        choose(&mut g, vec![id.clone()]);
        assert!(g.players[3]
            .graveyard
            .iter()
            .any(|c| c.definition == "XQ34" && c.id != id && c.controller == 3));
        if n == 0 {
            assert!(g.pending.is_none());
            assert!(g.stack.is_empty());
        } else {
            assert_eq!(g.pending.as_ref().unwrap().seat, 2);
            let ChoiceResolution::Declare { declaration, .. } =
                &g.pending.as_ref().unwrap().resolution
            else {
                panic!()
            };
            assert_eq!(
                (
                    declaration.actor,
                    declaration.source.card.controller,
                    declaration.source.card.owner
                ),
                (2, 2, 3)
            );
            assert_eq!(declaration.source.card.id, id);
            choose(
                &mut g,
                if accept {
                    vec!["accept".into()]
                } else {
                    vec![]
                },
            );
            drain(&mut g);
        }
        assert_eq!(g.players[2].hand.len(), usize::from(accept && n > 0));
        assert_eq!(resources(&g, 2), if accept { 0 } else { n });
        for s in [0, 1, 3] {
            assert!(g.view(s).pending_choice.is_none());
            assert!(g.view(s).revealed_hands.is_empty());
        }
    }
}
#[test]
fn hand_interactions_inspiration_normal_play_and_end_limit_discard() {
    let mut g = game();
    let id = held(&mut g, "XQ34", 0);
    fund(&mut g, 0, "JC075", 1);
    deck_cards(&mut g, 0, &["JC003", "JC125"]);
    play_target(&mut g, &id, "p0");
    drain(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ34"));
    assert!(g.pending.is_none());
    let mut g = game();
    fund(&mut g, 2, "JC075", 1);
    let id = held(&mut g, "XQ34", 2);
    deck_cards(&mut g, 2, &["JC003"]);
    g.effects.push_front(Effect::Discard {
        seat: 2,
        amount: 1,
        optional: false,
        redraw: false,
    });
    g.drive().unwrap();
    until_choice(&mut g, "discard");
    choose(&mut g, vec![id]);
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    choose(&mut g, vec!["accept".into()]);
    drain(&mut g);
    assert_eq!(g.players[2].hand.len(), 1);
}
#[test]
fn hand_interactions_recover_hand_cost_atomic_and_exact_prebound_target() {
    let mut g = game();
    let source = field(&mut g, "XQ38", 0);
    let cost = held(&mut g, "JC125", 0);
    let foreign = held(&mut g, "JC125", 1);
    let c = g.make_card("JC003", 0);
    let target = c.id.clone();
    g.players[0].graveyard.push(c);
    fund(&mut g, 0, "JC075", 3);
    for bad in [
        None,
        Some(foreign.as_str()),
        Some(source.as_str()),
        Some("stale"),
    ] {
        reject(
            &mut g,
            0,
            activation(&source, "recover-grave-discard", &target, bad),
        );
    }
    let mut duplicate = activation(&source, "recover-grave-discard", &target, Some(&cost));
    duplicate.cost_selected = Some(vec![cost.clone(), cost.clone()]);
    reject(&mut g, 0, duplicate);
    reject(
        &mut g,
        0,
        activation(&source, "recover-grave-discard", &cost, Some(&cost)),
    );
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| a.action.ability_id.as_deref() == Some("recover-grave-discard"))
        .unwrap();
    assert_eq!(
        a.action.cost_selected.as_ref().unwrap(),
        &vec![cost.clone()]
    );
    assert!(a.label.contains("费用：弃掉"));
    act(&mut g, 0, a.action);
    assert_eq!(resources(&g, 0), 1);
    assert!(g.board(&source).unwrap().1.exhausted);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125" && c.id != cost));
    drain(&mut g);
    assert!(g.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "JC003" && c.id != target));
}
#[test]
fn hand_interactions_recover_cost_inspiration_draw_resolves_before_recovery_and_stale_no_refund() {
    for stale in [false, true] {
        let mut g = game();
        let source = field(&mut g, "XQ38", 0);
        let cost = held(&mut g, "XQ34", 0);
        let c = g.make_card("JC003", 0);
        let target = c.id.clone();
        g.players[0].graveyard.push(c);
        fund(&mut g, 0, "JC075", 3);
        deck_cards(&mut g, 0, &["JC125", "JC125"]);
        act(
            &mut g,
            0,
            activation(&source, "recover-grave-discard", &target, Some(&cost)),
        );
        assert_eq!(g.stack.len(), 1);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
        assert_eq!(resources(&g, 0), 1);
        if stale {
            let c = g.players[0].graveyard.remove(0);
            let c = g.reset_zone_card(c);
            g.players[0].hand.push(c);
            restore(&mut g);
        }
        choose(&mut g, vec!["accept".into()]);
        assert_eq!(resources(&g, 0), 0);
        assert_eq!(g.stack.len(), 2);
        top(&mut g);
        assert_eq!(
            g.players[0]
                .hand
                .iter()
                .filter(|c| c.definition == "JC125")
                .count(),
            1
        );
        drain(&mut g);
        assert_eq!(
            g.players[0]
                .hand
                .iter()
                .filter(|c| c.definition == "JC003")
                .count(),
            1
        );
        assert!(g.board(&source).unwrap().1.exhausted);
    }
}
#[test]
fn hand_interactions_entry_mill_zero_one_three_fresh_ids_no_discard_or_elimination() {
    for n in [0, 1, 3] {
        let mut g = game();
        let source = held(&mut g, "XQ38", 0);
        fund(&mut g, 0, "JC084", 3);
        let old = deck_cards(&mut g, 2, &["XQ34", "JC125", "JC003"][..n]);
        fund(&mut g, 2, "JC075", 1);
        let rng = g.random;
        play_card(&mut g, 0, &source);
        until_choice(&mut g, "trigger");
        choose(&mut g, vec!["p2".into()]);
        drain(&mut g);
        assert_eq!(g.players[2].graveyard.len(), n);
        assert!(g.players[2].deck.is_empty());
        assert!(!g.players[2].eliminated);
        assert_eq!(g.random, rng);
        assert_eq!(resources(&g, 2), 1);
        assert!(g.pending.is_none());
        for c in &g.players[2].graveyard {
            assert!(!old.contains(&c.id));
            assert_eq!(c.owner, 2);
        }
    }
}
#[test]
fn hand_interactions_amnesia_random_private_rest_and_owner_deck_with_forecast() {
    for owner in [2, 3] {
        let mut g = game();
        let spell = held(&mut g, "JZ67", 0);
        fund(&mut g, 0, "JC104", 1);
        let chosen = held(&mut g, "XQ34", 2);
        g.players[2].hand[0].owner = owner;
        fund(&mut g, 2, "JC075", 1);
        let old = deck_cards(&mut g, 0, &["JC003", "JC125", "JC125"]);
        deck_cards(&mut g, owner, &["JC125", "JC125"]);
        let mut twin = g.clone();
        play_target(&mut g, &spell, "p2");
        play_target(&mut twin, &spell, "p2");
        until_choice(&mut g, "investigation");
        until_choice(&mut twin, "investigation");
        assert_eq!(
            serde_json::to_string(&g).unwrap(),
            serde_json::to_string(&twin).unwrap()
        );
        assert_eq!(g.players[owner].deck[0].definition, "XQ34");
        assert_ne!(g.players[owner].deck[0].id, chosen);
        assert_eq!(g.players[2].graveyard.len(), 0);
        assert_eq!(resources(&g, 2), 1);
        assert_eq!(g.pending.as_ref().unwrap().seat, 0);
        for v in 1..4 {
            assert!(g.view(v).pending_choice.is_none());
            let text = serde_json::to_string(&g.view(v)).unwrap();
            for id in &old[..2] {
                assert!(!text.contains(id));
            }
        }
        resolve_choice(&mut g);
        drain(&mut g);
        assert!(g.pending.is_none());
        assert_eq!(
            g.players[0]
                .deck
                .iter()
                .map(|c| c.id.clone())
                .collect::<Vec<_>>(),
            old
        );
    }
}
#[test]
fn hand_interactions_amnesia_empty_target_still_forecasts_without_dream_demon() {
    let mut g = game();
    let spell = held(&mut g, "JZ67", 0);
    fund(&mut g, 0, "JC104", 1);
    deck_cards(&mut g, 0, &["JC003", "JC125"]);
    let rng = g.random;
    play_target(&mut g, &spell, "p2");
    until_choice(&mut g, "investigation");
    assert_eq!(g.random, rng);
    resolve_choice(&mut g);
    drain(&mut g);
    assert!(g.pending.is_none());
    assert!(!g.players[2].eliminated);
}
#[test]
fn hand_interactions_optional_shuffle_prepared_choice_decline_and_accept() {
    // No admitted card has the 梦魔 subtype. Exercise persisted continuation only;
    // this checkpoint does not prove a naturally reachable positive condition.
    for accept in [false, true] {
        let mut g = game();
        let c = g.make_card("JZ67", 0);
        let source = g.source_snapshot(&c, None);
        let a = &definition("JZ67").abilities[0];
        let mut frame = g.make_frame(0, source, a, vec![], vec![], None);
        frame.steps.clear();
        deck_cards(&mut g, 2, &["JC003", "JC125", "XQ34"]);
        let rng = g.random;
        g.choice(
            0,
            "optional-shuffle",
            "是否洗牌".into(),
            vec![ChoiceOption {
                id: "shuffle".into(),
                label: "洗牌".into(),
                card: None,
            }],
            0,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(frame),
                choice: FrameChoice::OptionalShuffle { seat: 2 },
            },
        );
        choose(
            &mut g,
            if accept {
                vec!["shuffle".into()]
            } else {
                vec![]
            },
        );
        assert_eq!(g.random == rng, !accept);
        assert!(g.pending.is_none());
        assert_eq!(g.players[2].deck.len(), 3);
    }
}
#[test]
fn hand_interactions_hacker_naturally_dealt_deploy_pays_only_printed_two_without_ability_target() {
    let mut chosen = None;
    for seed in 1..100 {
        let mut draft = crate::deck::preset("watchers").unwrap();
        draft.cards = vec![
            catalog::DeckEntry {
                card_id: "JC114".into(),
                count: 3,
            },
            catalog::DeckEntry {
                card_id: "JC125".into(),
                count: 47,
            },
        ];
        let mut g = Game::new_with_deck(
            "natural-hacker".into(),
            "LOCAL".into(),
            "teams".into(),
            "P0".into(),
            draft.clone(),
            seed,
        )
        .unwrap();
        for s in 1..4 {
            g.join_with_deck(format!("P{s}"), draft.clone()).unwrap();
        }
        for seat in 0..4 {
            act(&mut g, seat, Action::new("ready"));
        }
        act(&mut g, 0, Action::new("start"));
        while g.pending.is_some() {
            choose(&mut g, vec![]);
        }
        if let Some(id) = g.players[0]
            .hand
            .iter()
            .find(|c| c.definition == "JC114")
            .map(|c| c.id.clone())
        {
            chosen = Some((g, id));
            break;
        }
    }
    let (mut g, id) = chosen.expect("bounded naturally dealt hacker");
    for _ in 0..500 {
        if let Some(a) = g
            .legal_actions(0)
            .into_iter()
            .find(|a| a.action.kind == "deploy" && a.action.card_id.as_deref() == Some(&id))
        {
            assert!(a.action.target_id.is_none() && a.action.ability_id.is_none());
            let before = resources(&g, 0);
            assert_eq!(before, 2);
            act(&mut g, 0, a.action);
            assert_eq!(resources(&g, 0), 0);
            assert!(g.pending.is_none());
            let frame = g.stack.last().unwrap().frame.as_ref().unwrap();
            assert_eq!(frame.ability_key, "deploy");
            assert!(frame.steps.is_empty());
            assert!(matches!(&frame.already_paid[..],[PaidCost::Assets(ids)]if ids.len()==2));
            top(&mut g);
            let c = g
                .regions
                .iter()
                .flat_map(|r| &r.cards)
                .find(|c| c.definition == "JC114" && c.controller == 0)
                .unwrap();
            assert_ne!(c.id, id);
            assert!(!c.exhausted);
            for s in 0..4 {
                assert!(g.view(s).revealed_hands.is_empty());
            }
            eprintln!(
                "natural JC114 seed={} turn={} paid=2 no target/ability/source exhaustion",
                g.seed, g.turn
            );
            return;
        }
        if let Some(p) = g.pending.clone() {
            if p.choice.kind == "discard" {
                let ids = p
                    .choice
                    .options
                    .iter()
                    .filter(|o| o.id != id)
                    .take(p.choice.min.unwrap_or(0))
                    .map(|o| o.id.clone())
                    .collect();
                choose(&mut g, ids);
            } else {
                resolve_choice(&mut g);
            }
            continue;
        }
        if resources(&g, 0) < 2 {
            if let Some(a) = g
                .legal_actions(0)
                .into_iter()
                .find(|a| a.action.kind == "asset" && a.action.card_id.as_deref() != Some(&id))
            {
                act(&mut g, 0, a.action);
                continue;
            }
        }
        pass(&mut g);
    }
    panic!("natural hacker declaration not reached");
}
