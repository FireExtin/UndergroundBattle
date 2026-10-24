//! CQ28 complete paired choices in printed order; author Native fixtures.
use crate::{catalog, import20_gate as gate, jc029_tests::pass_top, model::*, unit_support::*};
fn setup() -> (Game, String) {
    let mut g = game();
    fund(&mut g, 0, "WM057", 4);
    let c = held(&mut g, "CQ28", 0);
    (g, c)
}
fn start(g: &mut Game, id: &str, n: u8) {
    act(
        g,
        0,
        Action {
            card_id: Some(id.into()),
            ..Action::new("play")
        },
    );
    assert!(g.stack.is_empty());
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, gate::DRAFT);
    choose(g, vec![format!("item-{n}")]);
    choose(g, vec![format!("item-{n}")]);
}
fn target(g: &mut Game, id: &str) {
    choose(g, vec![id.into()]);
}
fn finish(g: &mut Game) {
    for _ in 0..80 {
        if let Some(p) = g.pending.clone() {
            let ids = if p.choice.kind == "trigger" {
                vec!["decline".into()]
            } else {
                p.choice
                    .options
                    .iter()
                    .take(p.choice.min.unwrap_or(0))
                    .map(|o| o.id.clone())
                    .collect()
            };
            choose(g, ids)
        } else if !g.stack.is_empty() {
            pass_top(g)
        } else if !g.effects.is_empty() {
            g.drive().unwrap()
        } else {
            return;
        }
    }
    panic!("bounded CQ28 continuation")
}
#[test]
fn cq28_exact_printed_fields_and_ten_canonical_pairs() {
    let d = catalog::card("CQ28");
    assert_eq!(
        (
            &*d.name,
            &*d.color,
            d.cost,
            d.loyalty.len(),
            d.unique,
            d.supported
        ),
        ("咒欲之门", "黑", 4, 4, false, true)
    );
    let def = gate::definition();
    assert_eq!(def.abilities[0].modes.len(), 10);
    assert!(def.abilities[0]
        .modes
        .iter()
        .all(|m| m.key.starts_with("pair-")));
}
#[test]
fn cq28_draw_twice_pays_printed_four_once_after_mode_selection() {
    let (mut g, id) = setup();
    let top = g.players[0]
        .deck
        .iter()
        .take(2)
        .map(|c| c.definition.clone())
        .collect::<Vec<_>>();
    start(&mut g, &id, 3);
    assert_eq!(
        g.players[0].assets.iter().filter(|c| c.exhausted).count(),
        4
    );
    assert_eq!(g.stack.len(), 1);
    pass_top(&mut g);
    assert_eq!(
        g.players[0]
            .hand
            .iter()
            .map(|c| c.definition.clone())
            .collect::<Vec<_>>(),
        top
    );
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "CQ28")
            .count(),
        1
    );
}
#[test]
fn cq28_mill_twice_keeps_two_six_card_instances() {
    let (mut g, id) = setup();
    let before = g.players[0].deck.len();
    let old = g.players[0]
        .deck
        .iter()
        .take(12)
        .map(|c| c.definition.clone())
        .collect::<Vec<_>>();
    start(&mut g, &id, 0);
    finish(&mut g);
    assert_eq!(g.players[0].deck.len(), before - 12);
    let grave = g.players[0]
        .graveyard
        .iter()
        .filter(|c| c.definition != "CQ28")
        .map(|c| c.definition.clone())
        .collect::<Vec<_>>();
    assert_eq!(grave, old);
}
#[test]
fn cq28_reanimate_two_predeclared_original_uids_in_two_regions() {
    let (mut g, id) = setup();
    let a = g.make_card("JC125", 0);
    let aid = a.id.clone();
    g.players[0].graveyard.push(a);
    let b = g.make_card("JC084", 0);
    let bid = b.id.clone();
    g.players[0].graveyard.push(b);
    start(&mut g, &id, 1);
    for t in [&aid, "region:0", &bid, "region:1"] {
        assert_eq!(
            g.players[0].assets.iter().filter(|c| c.exhausted).count(),
            0
        );
        target(&mut g, t);
    }
    assert_eq!(g.stack.len(), 1);
    finish(&mut g);
    assert!(g.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "JC125" && c.id != aid));
    assert!(g.regions[1]
        .cards
        .iter()
        .any(|c| c.definition == "JC084" && c.id != bid));
    assert!(!g.players[0]
        .graveyard
        .iter()
        .any(|c| c.id == aid || c.id == bid));
}
#[test]
fn cq28_repeated_grave_target_does_not_rebind_entered_instance() {
    let (mut g, id) = setup();
    let a = g.make_card("JC125", 0);
    let aid = a.id.clone();
    g.players[0].graveyard.push(a);
    start(&mut g, &id, 1);
    for t in [&aid, "region:0", &aid, "region:1"] {
        target(&mut g, t);
    }
    finish(&mut g);
    assert_eq!(
        g.regions
            .iter()
            .flat_map(|r| &r.cards)
            .filter(|c| c.definition == "JC125")
            .count(),
        1
    );
    assert!(g.regions[0].cards.iter().any(|c| c.definition == "JC125"));
    assert!(g.regions[1].cards.is_empty());
}
#[test]
fn cq28_defense_twice_stacks_and_expires() {
    let (mut g, id) = setup();
    let t = field(&mut g, "JC094", 2);
    let before = g.defense(g.board(&t).unwrap().1, 0);
    assert!(before >= 2);
    start(&mut g, &id, 2);
    target(&mut g, &t);
    target(&mut g, &t);
    finish(&mut g);
    if before == 2 {
        assert!(g.board(&t).is_none())
    } else {
        assert_eq!(g.defense(g.board(&t).unwrap().1, 0), before - 2)
    }
    assert_eq!(
        g.cq28_defense_reductions
            .iter()
            .filter(|m| m.frame.targets[m.slot].id == t)
            .count(),
        2
    );
    g.effects.push_front(Effect::FinishCleanup);
    g.drive().unwrap();
    assert!(g.cq28_defense_reductions.is_empty());
    assert_eq!(g.defense(g.board(&t).unwrap().1, 0), before);
}
#[test]
fn cq28_two_defense_targets_are_frozen_before_response() {
    let (mut g, id) = setup();
    let a = field(&mut g, "JC125", 1);
    let b = field(&mut g, "JC084", 1);
    start(&mut g, &id, 2);
    target(&mut g, &a);
    target(&mut g, &b);
    g.remove_dead(&b, RemovalCause::Destroy);
    pass_top(&mut g);
    assert_eq!(g.board(&a).unwrap().1.definition, "JC125");
    assert!(g.turn_attribute_modifiers.is_empty());
    assert!(g.cq28_defense_reductions.is_empty());
    assert_eq!(
        g.players[0].assets.iter().filter(|c| c.exhausted).count(),
        4
    );
}
#[test]
fn cq28_high_cost_and_other_grave_are_not_candidates() {
    let (mut g, id) = setup();
    let expensive = g.make_card("LC16", 0);
    let bad = expensive.id.clone();
    g.players[0].graveyard.push(expensive);
    let foreign = g.make_card("JC125", 1);
    let fid = foreign.id.clone();
    g.players[1].graveyard.push(foreign);
    let own = g.make_card("JC125", 0);
    g.players[0].graveyard.push(own);
    start(&mut g, &id, 1);
    let options = &g.pending.as_ref().unwrap().choice.options;
    assert!(!options.iter().any(|o| o.id == bad || o.id == fid));
    let p = g.pending.clone().unwrap();
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![bad]),
            ..Action::new("choose")
        },
    );
}
#[test]
fn cq28_rejects_reordered_unknown_and_extra_targets_before_payment() {
    let (mut g, id) = setup();
    for option in ["pair-3-0", "pair-4-4"] {
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(id.clone()),
                ability_id: Some(gate::KEY.into()),
                option: Some(option.into()),
                selected: Some(vec![]),
                ..Action::new("play")
            },
        );
    }
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            ability_id: Some(gate::KEY.into()),
            option: Some("pair-3-3".into()),
            selected: Some(vec!["extra".into()]),
            ..Action::new("play")
        },
    );
    assert!(g.pending.is_none());
    assert!(g.players[0].assets.iter().all(|c| !c.exhausted));
}
#[test]
fn cq28_insufficient_payment_and_spell_ban_cannot_trap_unpaid_draft() {
    let (mut g, id) = setup();
    g.players[0].assets.pop();
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            ..Action::new("play")
        },
    );
    fund(&mut g, 0, "WM057", 1);
    field(&mut g, "XQ22", 1);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            ..Action::new("play")
        },
    );
    assert!(g.pending.is_none());
}

fn mixed(g: &mut Game, id: &str, a: u8, b: u8) {
    act(
        g,
        0,
        Action {
            card_id: Some(id.into()),
            ..Action::new("play")
        },
    );
    choose(g, vec![format!("item-{a}")]);
    choose(g, vec![format!("item-{b}")]);
}
fn known_deck(g: &mut Game) {
    g.players[0].deck.clear();
    for id in [
        "JC125", "JC084", "JC014", "JC085", "JC042", "JC112", "JC057", "JC070",
    ] {
        let c = g.make_card(id, 0);
        g.players[0].deck.push(c)
    }
}
#[test]
fn cq28_mixed_reverse_clicks_still_mill_before_draw() {
    for (a, b) in [(3, 0), (0, 3)] {
        let (mut g, id) = setup();
        known_deck(&mut g);
        mixed(&mut g, &id, a, b);
        let f = g.stack.last().unwrap().frame.as_ref().unwrap();
        assert!(matches!(
            f.steps[0].op,
            crate::rules::Op::CQ28Mode { mode: 0, .. }
        ));
        assert!(matches!(
            f.steps[1].op,
            crate::rules::Op::CQ28Mode { mode: 3, .. }
        ));
        finish(&mut g);
        assert_eq!(
            g.players[0]
                .hand
                .iter()
                .map(|c| c.definition.as_str())
                .collect::<Vec<_>>(),
            ["JC057"]
        );
        assert_eq!(
            g.players[0].assets.iter().filter(|c| c.exhausted).count(),
            4
        );
    }
}
#[test]
fn cq28_mixed_mill_revive_freezes_existing_grave_target_before_payment() {
    let (mut g, id) = setup();
    known_deck(&mut g);
    let newly_milled = g.players[0].deck[0].id.clone();
    let old = g.make_card("JC084", 0);
    let old_id = old.id.clone();
    g.players[0].graveyard.push(old);
    mixed(&mut g, &id, 1, 0);
    let p = g.pending.clone().unwrap();
    assert!(!p.choice.options.iter().any(|o| o.id == newly_milled));
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![newly_milled]),
            ..Action::new("choose")
        },
    );
    assert!(g.players[0].assets.iter().all(|c| !c.exhausted));
    target(&mut g, &old_id);
    target(&mut g, "region:0");
    finish(&mut g);
    assert!(g.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "JC084" && c.id != old_id));
    assert!(!g.regions[0].cards.iter().any(|c| c.definition == "JC125"));
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125"));
}
#[test]
fn cq28_mixed_mill_defense_keeps_second_step_receipt() {
    let (mut g, id) = setup();
    known_deck(&mut g);
    let h = field(&mut g, "JC094", 2);
    mixed(&mut g, &id, 2, 0);
    target(&mut g, &h);
    finish(&mut g);
    assert_eq!(g.defense(g.board(&h).unwrap().1, 0), 2);
    let m = &g.cq28_defense_reductions[0];
    assert_eq!((m.op_index, m.slot), (1, 0));
    restore(&mut g);
}
#[test]
fn cq28_mixed_revive_defense_does_not_target_new_entry_and_uses_target_slot2() {
    let (mut g, id) = setup();
    let old = g.make_card("JC084", 0);
    let oid = old.id.clone();
    g.players[0].graveyard.push(old);
    let h = field(&mut g, "JC094", 2);
    mixed(&mut g, &id, 2, 1);
    target(&mut g, &oid);
    target(&mut g, "region:1");
    let p = g.pending.clone().unwrap();
    assert!(!p.choice.options.iter().any(|o| o.id == oid));
    target(&mut g, &h);
    finish(&mut g);
    assert_eq!(g.defense(g.board(&h).unwrap().1, 0), 2);
    let m = &g.cq28_defense_reductions[0];
    assert_eq!((m.op_index, m.slot), (1, 2));
    assert!(g.regions[1].cards.iter().any(|c| c.definition == "JC084"));
    restore(&mut g);
    g.effects.push_front(Effect::FinishCleanup);
    g.drive().unwrap();
    assert_eq!(g.defense(g.board(&h).unwrap().1, 0), 3);
}
#[test]
fn cq28_mixed_revive_then_draw_uses_same_paid_frame() {
    let (mut g, id) = setup();
    known_deck(&mut g);
    let old = g.make_card("JC084", 0);
    let oid = old.id.clone();
    g.players[0].graveyard.push(old);
    mixed(&mut g, &id, 3, 1);
    target(&mut g, &oid);
    target(&mut g, "region:0");
    finish(&mut g);
    assert!(g.regions[0].cards.iter().any(|c| c.definition == "JC084"));
    assert_eq!(g.players[0].hand[0].definition, "JC125");
    assert_eq!(
        g.players[0].assets.iter().filter(|c| c.exhausted).count(),
        4
    );
}
#[test]
fn cq28_mixed_defense_then_draw_performs_normal_lethal_cleanup() {
    let (mut g, id) = setup();
    known_deck(&mut g);
    let h = field(&mut g, "JC125", 2);
    mixed(&mut g, &id, 3, 2);
    target(&mut g, &h);
    finish(&mut g);
    assert!(g.board(&h).is_none());
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125"));
    assert_eq!(g.players[0].hand[0].definition, "JC125");
}

#[test]
fn cq28_mill_rumor_resumes_both_paid_modes_once() {
    for second in [0, 3] {
        for seal in [false, true] {
            let (mut g, gate) = setup();
            let host = field(&mut g, "JC125", 0);
            g.players[0].deck.clear();
            for i in 0..13 {
                let card = g.make_card(if i == 0 { "WM042" } else { "JC125" }, 0);
                g.players[0].deck.push(card);
            }
            mixed(&mut g, &gate, 0, second);
            for _ in 0..16 {
                if g.pending.is_some() { break; }
                pass(&mut g);
            }
            let p = g.pending.clone().expect("CQ28 must reach the legal rumor replacement");
            assert!(matches!(p.resolution, ChoiceResolution::Rumor { .. }));
            assert_eq!(g.resources(0), 0);
            reject(&mut g, 2, Action { choice_id: Some(p.choice.id),
                selected: Some(vec![host.clone()]), ..Action::new("choose") });
            choose(&mut g, if seal { vec![host.clone()] } else { vec![] });
            finish(&mut g);
            let milled = if second == 0 { 12 } else { 6 };
            assert_eq!(g.players[0].deck.len(), 13 - milled - usize::from(second == 3));
            assert_eq!(g.players[0].hand.len(), usize::from(second == 3));
            assert_eq!(g.players[0].graveyard.len(), 1 + milled - usize::from(seal));
            assert_eq!(g.sealed_cards.iter().filter(|s| s.host_id == host).count(), usize::from(seal));
            assert_eq!(g.players[0].assets.iter().filter(|c| c.exhausted).count(), 4);
            assert!(g.pending.is_none() && g.stack.is_empty() && g.effects.is_empty());
            let stable = serde_json::to_string(&g).unwrap();
            g.drive().unwrap();
            assert_eq!(serde_json::to_string(&g).unwrap(), stable);
            restore(&mut g);
        }
    }
}
