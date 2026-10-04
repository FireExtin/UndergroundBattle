//! Explicit initial layouts; public declarations/choices, and named primitive
//! checkpoints for controller transfer, continuous grants and deck mutations.
use crate::{
    catalog,
    model::*,
    msjc01_tests::{activate, game, pass_top, rejected, roundtrip},
    rules::*,
};
fn fund(g: &mut Game, seat: usize, definition: &str, n: usize) {
    for _ in 0..n {
        let c = g.make_card(definition, seat);
        g.players[seat].assets.push(c);
    }
}
fn field(g: &mut Game, definition: &str, owner: usize, controller: usize) -> String {
    let mut c = g.make_card(definition, owner);
    c.controller = controller;
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn search(source: &str) -> Action {
    Action {
        card_id: Some(source.into()),
        ability_id: Some("search-any-private".into()),
        ..Action::new("activate")
    }
}
fn deck(g: &mut Game, seat: usize, defs: &[&str]) {
    g.players[seat].deck.clear();
    for d in defs {
        let c = g.make_card(d, seat);
        g.players[seat].deck.push(c);
    }
}
fn choose(g: &mut Game, ids: Vec<String>) {
    let p = g.pending.as_ref().unwrap();
    g.apply(
        p.seat,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(ids),
            ..Action::new("choose")
        },
    )
    .unwrap();
}
fn top(g: &Game, seat: usize) {
    let v = g.view(seat);
    assert_eq!(
        v.private_deck_top.as_ref().map(|c| &c.instance_id),
        g.players[seat].deck.first().map(|c| &c.id)
    );
    for other in 0..4 {
        if other != seat {
            assert!(g.view(other).private_deck_top.is_none());
        }
    }
}
#[test]
fn yellow_batch_printed_fields_loyalty_ordinary_and_initiative_icons() {
    let w = catalog::card("WM003");
    assert_eq!(w.name, "千机庙离");
    assert!(w.unique && w.rule_traits.public);
    assert_eq!((w.cost, w.defense), (1, Some(1)));
    assert_eq!(w.loyalty, ["黄色", "星辰", "星辰"]);
    assert_eq!(w.magic, "心灵");
    assert_eq!(w.subtypes, ["人类", "法师", "学生"]);
    assert_eq!(w.permanent_icons, Icons::default());
    assert_eq!(
        w.temporary_icons,
        Icons {
            investigation: 1,
            combat: 0,
            influence: 1
        }
    );
    let l = catalog::card("LC01");
    assert_eq!((l.cost, l.defense), (5, Some(4)));
    assert!(l.unique);
    assert_eq!(l.loyalty, ["黄色", "黄色"]);
    assert_eq!(l.magic, "神圣");
    assert_eq!(
        l.permanent_icons,
        Icons {
            investigation: 2,
            combat: 0,
            influence: 0
        }
    );
    assert_eq!(
        l.temporary_icons,
        Icons {
            investigation: 1,
            combat: 0,
            influence: 0
        }
    );
    assert_eq!(l.deck_copy_limit, Some(3));
    assert_eq!(w.deck_copy_limit, Some(3));
    let mut g = game();
    g.first_team = 0;
    let id = field(&mut g, "WM003", 0, 0);
    let c = g.board(&id).unwrap().1;
    assert_eq!(g.icons(c, 0), w.temporary_icons);
    g.first_team = 1;
    assert_eq!(g.icons(g.board(&id).unwrap().1, 0), Icons::default());
    fund(&mut g, 0, "JC002", 1);
    fund(&mut g, 1, "JC104", 2);
    assert!(!g.loyalty(0, "WM003"));
    fund(&mut g, 0, "JC104", 1);
    assert!(!g.loyalty(0, "WM003"));
    fund(&mut g, 0, "JC104", 1);
    assert!(g.loyalty(0, "WM003"));
}
#[test]
fn yellow_batch_wm003_public_play_private_choice_and_no_other_seat_leak() {
    let mut g = game();
    fund(&mut g, 0, "JC002", 1);
    fund(&mut g, 0, "JC104", 2);
    let c = g.make_card("WM003", 0);
    let hand = c.id.clone();
    g.players[0].hand.push(c);
    rejected(
        &mut g,
        0,
        Action {
            card_id: Some(hand.clone()),
            region: Some(0),
            ..Action::new("conceal")
        },
    );
    g.apply(
        0,
        Action {
            card_id: Some(hand),
            region: Some(0),
            ..Action::new("deploy")
        },
    )
    .unwrap();
    pass_top(&mut g);
    let source = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "WM003")
        .unwrap()
        .id
        .clone();
    fund(&mut g, 0, "JC125", 3);
    deck(&mut g, 0, &["LC01", "JC125", "JC125", "JC125"]);
    g.apply(0, search(&source)).unwrap();
    pass_top(&mut g);
    let p = g.pending.clone().unwrap();
    assert_eq!(p.choice.min, Some(1));
    assert_eq!(p.choice.options.len(), 4);
    let chosen = p
        .choice
        .options
        .iter()
        .find(|o| o.card.as_ref().unwrap().card_id.as_deref() == Some("LC01"))
        .unwrap()
        .id
        .clone();
    for other in 1..4 {
        let v = g.view(other);
        assert!(v.pending_choice.is_none());
        assert!(v.waiting_choice.is_some());
        let encoded = serde_json::to_string(&v).unwrap();
        for secret in ["西比尔", "LC01", chosen.as_str()] {
            assert!(!encoded.contains(secret), "{other}: {secret}");
        }
        rejected(
            &mut g,
            other,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec![chosen.clone()]),
                ..Action::new("choose")
            },
        );
    }
    roundtrip(&mut g);
    let rng = g.random;
    choose(&mut g, vec![chosen.clone()]);
    assert_ne!(g.random, rng);
    let held = g.players[0]
        .hand
        .iter()
        .find(|c| c.definition == "LC01")
        .unwrap();
    assert_ne!(held.id, chosen);
    assert!(g.log.iter().any(|e| e.text.contains("完成私密检索")));
    assert!(!g
        .log
        .iter()
        .any(|e| e.text.contains("西比尔") || e.text.contains("展示检索")));
    for other in 1..4 {
        let v = serde_json::to_string(&g.view(other)).unwrap();
        assert!(!v.contains("西比尔") && !v.contains("LC01"));
    }
}
#[test]
fn yellow_batch_wm003_cost_exhaustion_rejected_atomic_empty_and_optional_search_boundary() {
    for exhausted in [false, true] {
        let mut g = game();
        let id = field(&mut g, "WM003", 0, 0);
        fund(&mut g, 0, "JC125", if exhausted { 5 } else { 4 });
        g.regions[0].cards[0].exhausted = exhausted;
        rejected(&mut g, 0, search(&id));
    }
    for optional in [false, true] {
        let mut g = game();
        let id = field(&mut g, "WM003", 0, 0);
        fund(&mut g, 0, "JC125", 5);
        deck(&mut g, 0, &["LC01", "JC125", "JC125"]);
        g.apply(0, search(&id)).unwrap();
        if optional {
            if let Op::Search { optional, .. } =
                &mut g.stack.last_mut().unwrap().frame.as_mut().unwrap().steps[0].op
            {
                *optional = true;
            }
        }
        pass_top(&mut g);
        assert_eq!(
            g.pending.as_ref().unwrap().choice.min,
            Some(if optional { 0 } else { 1 })
        );
        if optional {
            choose(&mut g, vec![]);
            assert_eq!(g.players[0].deck.len(), 3);
        } else {
            let p = g.pending.clone().unwrap();
            rejected(
                &mut g,
                0,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec![]),
                    ..Action::new("choose")
                },
            );
        }
    }
    let mut g = game();
    let id = field(&mut g, "WM003", 0, 0);
    fund(&mut g, 0, "JC125", 5);
    deck(&mut g, 0, &[]);
    g.apply(0, search(&id)).unwrap();
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert_eq!(g.resources(0), 0);
    assert!(g.board(&id).unwrap().1.exhausted);
}
#[test]
fn yellow_batch_lc01_current_controller_multiple_exhausted_hidden_and_leaving_sources() {
    let mut g = game();
    let a = field(&mut g, "LC01", 0, 1);
    let b = field(&mut g, "LC01", 2, 1);
    deck(&mut g, 1, &["LC23", "JC125"]);
    for c in &mut g.regions[0].cards {
        c.exhausted = true;
    }
    top(&g, 1);
    assert_eq!(
        g.view(1).private_deck_top.unwrap().card_id.as_deref(),
        Some("LC23")
    );
    g.regions[0]
        .cards
        .iter_mut()
        .find(|c| c.id == a)
        .unwrap()
        .face_down = true;
    top(&g, 1);
    g.remove_board(&b).unwrap();
    assert!(g.view(1).private_deck_top.is_none());
    let c = g.regions[0].cards.iter_mut().find(|c| c.id == a).unwrap();
    c.face_down = false;
    c.controller = 3;
    top(&g, 3);
    assert!(g.view(1).private_deck_top.is_none());
    g.remove_board(&a).unwrap();
    assert!((0..4).all(|s| g.view(s).private_deck_top.is_none()));
}
#[test]
fn yellow_batch_lc01_live_top_after_draw_shuffle_search_top_return_and_restore() {
    let mut g = game();
    field(&mut g, "LC01", 0, 0);
    let w = field(&mut g, "WM003", 0, 0);
    deck(&mut g, 0, &["LC23", "JC125", "XQ03", "JC125"]);
    top(&g, 0);
    let before = g.view(0).private_deck_top.unwrap().instance_id;
    g.draw(0, 1).unwrap();
    top(&g, 0);
    assert_ne!(g.view(0).private_deck_top.unwrap().instance_id, before);
    g.shuffle_player(0);
    top(&g, 0);
    roundtrip(&mut g);
    top(&g, 0);
    fund(&mut g, 0, "JC125", 5);
    g.apply(0, search(&w)).unwrap();
    pass_top(&mut g);
    let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
    choose(&mut g, vec![id]);
    top(&g, 0);
    // Explicit ready-source and paid-frame top-destination checkpoint, not WM003's printed path.
    g.regions[0]
        .cards
        .iter_mut()
        .find(|c| c.id == w)
        .unwrap()
        .exhausted = false;
    fund(&mut g, 0, "JC125", 5);
    g.apply(0, search(&w)).unwrap();
    if let Op::Search { to_top, .. } =
        &mut g.stack.last_mut().unwrap().frame.as_mut().unwrap().steps[0].op
    {
        *to_top = true;
    }
    pass_top(&mut g);
    let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
    choose(&mut g, vec![id.clone()]);
    top(&g, 0);
    assert_ne!(g.view(0).private_deck_top.unwrap().instance_id, id);
    deck(&mut g, 0, &[]);
    assert!(g.view(0).private_deck_top.is_none());
}
#[test]
fn yellow_batch_msjc01_searches_real_yellow_unique_and_reveals_then_fresh_hand() {
    let mut g = game();
    fund(&mut g, 0, "JC125", 4);
    deck(&mut g, 0, &["LC01", "WM003", "LC23", "JC125"]);
    let a = activate(&g, 0, "search-yellow-unique");
    g.apply(0, a).unwrap();
    pass_top(&mut g);
    let p = g.pending.clone().unwrap();
    assert_eq!(p.choice.options.len(), 2);
    assert!(p.choice.options.iter().all(|o| matches!(
        o.card.as_ref().unwrap().card_id.as_deref(),
        Some("LC01" | "WM003")
    )));
    let chosen = p
        .choice
        .options
        .iter()
        .find(|o| o.card.as_ref().unwrap().card_id.as_deref() == Some("LC01"))
        .unwrap()
        .id
        .clone();
    choose(&mut g, vec![chosen.clone()]);
    assert_ne!(g.players[0].hand[0].id, chosen);
    assert_eq!(g.players[0].hand[0].definition, "LC01");
    for seat in 0..4 {
        assert!(g
            .view(seat)
            .log
            .iter()
            .any(|e| e.text.contains("展示检索的 西比尔")));
    }
}
