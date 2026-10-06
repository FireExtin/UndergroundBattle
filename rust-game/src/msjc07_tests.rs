//! Real MSJC07 program and real black/gold targets; disclosed layouts only.
use crate::msjc01_tests::{fund, pass_top, rejected};
use crate::{catalog, deck, model::*, rules::*};
const SEARCH: &str = "search-black-unique";
const DRAW: &str = "drawWithInitiative";
const BLACK: [&str; 11] = [
    "JC086", "JC092", "JC091", "JZ54", "JC084", "JC085", "JC088", "BQ083", "JC093", "JC096", "XQ38",
];
fn draft(black: usize, total: usize) -> deck::DeckDraft {
    let mut d = deck::preset("watchers").unwrap();
    d.society_id = Some("MSJC07".into());
    d.cards.clear();
    let mut left = black;
    for id in BLACK {
        let count = left.min(3);
        if count > 0 {
            d.cards.push(catalog::DeckEntry {
                card_id: id.into(),
                count,
            });
        }
        left -= count;
    }
    assert_eq!(left, 0);
    if total > black {
        d.cards.push(catalog::DeckEntry {
            card_id: "JC125".into(),
            count: total - black,
        });
    }
    d
}
fn initial() -> Game {
    let mut g = Game::new_with_deck(
        "msjc07-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        draft(33, 50),
        9,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), draft(33, 50)).unwrap();
    }
    for s in 0..4 {
        g.apply(s, Action::new("ready")).unwrap();
    }
    g.apply(0, Action::new("start")).unwrap();
    assert!(g.players.iter().all(|p| p.hand.len() == 6));
    while let Some(p) = g.pending.clone() {
        g.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
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
    }
    g.begin_window(Window::Action(0));
    g
}
fn source(g: &Game, s: usize) -> String {
    g.players[s].society_zone.card.as_ref().unwrap().id.clone()
}
fn activate(g: &Game, s: usize, key: &str) -> Action {
    Action {
        card_id: Some(source(g, s)),
        ability_id: Some(key.into()),
        ..Action::new("activate")
    }
}
fn restore(g: &mut Game) {
    let views = (0..4)
        .map(|s| serde_json::to_value(g.view(s)).unwrap())
        .collect::<Vec<_>>();
    let state = serde_json::to_string(g).unwrap();
    *g = Game::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), state);
    for s in 0..4 {
        assert_eq!(serde_json::to_value(g.view(s)).unwrap(), views[s]);
    }
}
fn next_turn(g: &mut Game) {
    let t = g.turn;
    for _ in 0..250 {
        if g.turn > t {
            return;
        }
        let s = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(s, Action::new("pass")).unwrap();
    }
    panic!("turn bound");
}
#[test]
fn msjc07_original_whole_card_and_finite_existing_search_program() {
    let d = crate::society::definition("MSJC07").unwrap();
    assert_eq!(
        (&*d.card.name, &*d.subtitle, &*d.card.color),
        ("方碑序列", "恐怖同盟", "黑")
    );
    assert_eq!(d.card.subtypes, ["法师结社"]);
    assert_eq!(d.starting_hand, 6);
    assert!(d.printed_cost.is_none() && d.card.unique);
    assert!(d.card.loyalty.is_empty() && d.card.defense.is_none() && d.card.keywords.is_empty());
    assert_eq!(
        serde_json::to_value(&d.deck_constraints).unwrap(),
        serde_json::json!([{"kind":"minimumColor","color":"黑","count":25}])
    );
    let a = &crate::rules::definition("MSJC07").abilities;
    assert_eq!(a.len(), 2);
    assert_eq!(
        serde_json::to_value(&a[0]).unwrap(),
        serde_json::to_value(&crate::rules::definition("MSJC09").abilities[0]).unwrap()
    );
    assert!(a[1].once_per_game && a[1].event.is_none() && a[1].targets.is_empty());
    assert_eq!(
        serde_json::to_value(&a[1].costs).unwrap(),
        serde_json::json!([{"Assets":4},"ExhaustSource"])
    );
    assert!(
        matches!(&a[1].ops[0],Op::Search{player:PlayerRef::Actor,filter:CardFilter::PrintedColorAndUnique{color},to_top:false,optional:false,visibility:SearchVisibility::Reveal} if color=="黑")
    );
    assert_eq!(catalog::catalog().cards.len(), 99);
    assert_eq!(catalog::catalog().societies.len(), 8);
    assert!(!catalog::catalog().cards.iter().any(|c| c.id == "MSJC07"));
    assert_eq!(
        catalog::catalog()
            .cards
            .iter()
            .filter(|c| c.color == "黑" && c.kind != "region")
            .count(),
        12
    );
    for id in ["JC096", "XQ38"] {
        let c = catalog::card(id);
        assert!(c.unique && c.color == "黑" && c.society.is_empty());
    }
}
#[test]
fn msjc07_construction_actual_black_24_25_33_and_society_outside_50() {
    assert!(deck::validate(draft(24, 50))
        .unwrap_err()
        .contains("当前24张"));
    for n in [25, 33] {
        let d = deck::validate(draft(n, 50)).unwrap();
        assert_eq!(d.cards.iter().map(|e| e.count).sum::<usize>(), 50);
    }
    assert!(deck::validate(draft(25, 49))
        .unwrap_err()
        .contains("至少需要50"));
    let mut d = draft(33, 50);
    d.cards[0].count = 4;
    assert!(deck::validate(d).unwrap_err().contains("最多3"));
    let mut d = draft(25, 50);
    d.cards.push(catalog::DeckEntry {
        card_id: "MSJC07".into(),
        count: 1,
    });
    assert!(deck::validate(d).is_err());
    let mut d = draft(24, 50);
    d.cards.push(catalog::DeckEntry {
        card_id: "JC006".into(),
        count: 1,
    });
    assert!(deck::validate(d).is_err());
}
#[test]
fn msjc07_four_same_societies_real_black_hits_own_deck_private_choice_public_reveal_fresh_hand() {
    let mut g = initial();
    let ids = (0..4)
        .map(|s| source(&g, s))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 4);
    for s in 0..4 {
        g.begin_window(Window::Action(g.team(s)));
        fund(&mut g, s, 4);
        for id in ["JC096", "XQ38", "JC091", "LC23"] {
            let c = g.make_card(id, s);
            g.players[s].deck.push(c);
        }
        let n = g.players[s].deck.len();
        let other = s ^ 1;
        let other_n = g.players[other].deck.len();
        let a = activate(&g, s, SEARCH);
        g.apply(s, a).unwrap();
        assert_eq!(
            g.players[s].assets.iter().filter(|c| c.exhausted).count(),
            4
        );
        assert!(g.players[s].society_zone.card.as_ref().unwrap().exhausted);
        for later in s + 1..4 {
            assert!(g.players[later].society_zone.used_once_per_game.is_empty());
        }
        restore(&mut g);
        pass_top(&mut g);
        let p = g.pending.clone().unwrap();
        assert_eq!(p.seat, s);
        assert_eq!(p.choice.options.len(), 2);
        assert_eq!(p.choice.min, Some(1));
        for v in 0..4 {
            assert_eq!(g.view(v).pending_choice.is_some(), v == s);
        }
        restore(&mut g);
        let choice = |seat, ids: Vec<String>| {
            (
                seat,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(ids),
                    ..Action::new("choose")
                },
            )
        };
        let (v, a) = choice(other, vec![p.choice.options[0].id.clone()]);
        rejected(&mut g, v, a);
        let (v, a) = choice(s, vec![]);
        rejected(&mut g, v, a);
        let foreign = g.players[other].deck[0].id.clone();
        let (v, a) = choice(s, vec![foreign]);
        rejected(&mut g, v, a);
        let selected = p.choice.options[s % 2].id.clone();
        let definition = g.players[s]
            .deck
            .iter()
            .find(|c| c.id == selected)
            .unwrap()
            .definition
            .clone();
        let rng = g.random;
        let (v, a) = choice(s, vec![selected.clone()]);
        g.apply(v, a).unwrap();
        let c = g.players[s].hand.last().unwrap();
        assert_eq!(c.definition, definition);
        assert_ne!(c.id, selected);
        assert_eq!((c.owner, c.controller), (s, s));
        assert_eq!(g.players[s].deck.len(), n - 1);
        assert_eq!(g.players[other].deck.len(), other_n);
        assert_ne!(rng, g.random);
        for v in 0..4 {
            let view = g.view(v);
            assert_eq!(
                view.society_zones[s]
                    .card
                    .as_ref()
                    .unwrap()
                    .used_once_per_game,
                Some(vec![SEARCH.into()])
            );
            assert!(view.log.iter().any(|e| e.text.contains("展示检索的")));
        }
        restore(&mut g);
    }
}
#[test]
fn msjc07_rejections_are_atomic_and_cannot_pay_or_activate_with_teammate() {
    for bad in [
        "cost",
        "exhausted",
        "foreign-source",
        "yellow-key",
        "missing-key",
    ] {
        let mut g = initial();
        fund(&mut g, 0, if bad == "cost" { 3 } else { 4 });
        fund(&mut g, 1, 8);
        let mut a = activate(&g, 0, SEARCH);
        match bad {
            "exhausted" => g.players[0].society_zone.card.as_mut().unwrap().exhausted = true,
            "foreign-source" => a.card_id = Some(source(&g, 1)),
            "yellow-key" => a.ability_id = Some("search-yellow-unique".into()),
            "missing-key" => a.ability_id = Some("invented".into()),
            _ => {}
        }
        rejected(&mut g, 0, a);
        assert!(g
            .players
            .iter()
            .all(|p| p.society_zone.used_once_per_game.is_empty()));
        assert!(g.players[1].assets.iter().all(|c| !c.exhausted));
        restore(&mut g);
    }
}
#[test]
fn msjc07_initiative_draw_and_rear_paid_no_draw_are_personal_and_separate() {
    for first in [0, 1] {
        let mut g = initial();
        g.first_team = first;
        fund(&mut g, 0, 3);
        fund(&mut g, 1, 3);
        let n = g.players[0].deck.len();
        let a = activate(&g, 0, DRAW);
        g.apply(0, a).unwrap();
        restore(&mut g);
        pass_top(&mut g);
        let gain = usize::from(first == 0);
        assert_eq!(g.players[0].hand.len(), gain);
        assert_eq!(g.players[0].deck.len(), n - gain);
        assert!(g.players[0].society_zone.used_once_per_game.is_empty());
        assert_eq!(
            g.players[0].assets.iter().filter(|c| c.exhausted).count(),
            3
        );
        assert!(g.players[1].assets.iter().all(|c| !c.exhausted));
        restore(&mut g);
    }
}
#[test]
fn msjc07_empty_search_shuffles_and_empty_deck_is_not_draw_elimination() {
    for empty in [false, true] {
        let mut g = initial();
        if empty {
            g.players[0].deck.clear();
        }
        fund(&mut g, 0, 4);
        let n = g.players[0].deck.len();
        let rng = g.random;
        let a = activate(&g, 0, SEARCH);
        g.apply(0, a).unwrap();
        pass_top(&mut g);
        assert!(g.players[0].hand.is_empty() && !g.players[0].eliminated && g.pending.is_none());
        assert_eq!(g.players[0].deck.len(), n);
        assert!(g.players[0]
            .society_zone
            .used_once_per_game
            .contains(SEARCH));
        if !empty {
            assert_ne!(rng, g.random);
        }
        restore(&mut g);
    }
}
#[test]
fn msjc07_cancelled_paid_frame_retains_payment_exhaustion_and_quota() {
    let mut g = initial();
    fund(&mut g, 0, 4);
    let c = g.make_card("JC096", 0);
    g.players[0].deck.push(c);
    let a = activate(&g, 0, SEARCH);
    g.apply(0, a).unwrap();
    // Explicit cancellation boundary, not a claim about a natural counter-card.
    g.stack.last_mut().unwrap().frame.as_mut().unwrap().guard = GuardState::Cancelled;
    restore(&mut g);
    pass_top(&mut g);
    assert!(
        g.players[0].hand.is_empty()
            && g.players[0]
                .society_zone
                .used_once_per_game
                .contains(SEARCH)
    );
    assert_eq!(
        g.players[0].assets.iter().filter(|c| c.exhausted).count(),
        4
    );
}
#[test]
fn msjc07_ready_turn_and_new_instance_keep_quota_restart_clears() {
    let mut g = initial();
    fund(&mut g, 0, 8);
    let a = activate(&g, 0, SEARCH);
    g.apply(0, a).unwrap();
    pass_top(&mut g);
    next_turn(&mut g);
    g.begin_window(Window::Action(0));
    assert!(!g.players[0].society_zone.card.as_ref().unwrap().exhausted);
    let a = activate(&g, 0, SEARCH);
    rejected(&mut g, 0, a);
    assert!(g
        .legal_actions(0)
        .iter()
        .any(|a| a.action.ability_id.as_deref() == Some(DRAW)));
    let old = source(&g, 0);
    let c = g.players[0].society_zone.card.take().unwrap();
    let mut c = g.fresh(c);
    c.exhausted = false;
    g.players[0].society_zone.card = Some(c);
    assert_ne!(old, source(&g, 0));
    restore(&mut g);
    let a = activate(&g, 0, SEARCH);
    rejected(&mut g, 0, a);
    let a = activate(&g, 0, DRAW);
    g.apply(0, a).unwrap();
    pass_top(&mut g);
    assert_eq!(g.players[0].society_zone.used_once_per_game.len(), 1);
    g.status = "finished".into();
    g.apply(0, Action::new("restart")).unwrap();
    assert!(g
        .players
        .iter()
        .all(|p| p.society_zone.used_once_per_game.is_empty() && p.hand.len() == 6));
    restore(&mut g);
}
