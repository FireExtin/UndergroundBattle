//! Explicit layout and paid-frame checkpoints. Legacy paid-frame hit fixtures search neutral+unique LC23,
//! never fictional yellow targets; new batch tests cover real yellow targets.
use crate::{
    catalog, deck,
    model::*,
    room::{RoomCommand, RoomEnvelope, SessionAction},
    rules::*,
};
const SEARCH: &str = "search-yellow-unique";
const DRAW: &str = "drawWithInitiative";
fn draft(yellow: usize, total: usize) -> deck::DeckDraft {
    let mut left = yellow;
    let mut cards = vec![];
    for id in [
        "JC001", "JC002", "JC003", "JC004", "JC005", "JC006", "JC007", "XQ03", "JC008",
    ] {
        let count = left.min(3);
        if count > 0 {
            cards.push(catalog::DeckEntry {
                card_id: id.into(),
                count,
            });
        }
        left -= count;
    }
    assert_eq!(left, 0);
    if total > yellow {
        cards.push(catalog::DeckEntry {
            card_id: "JC125".into(),
            count: total - yellow,
        });
    }
    deck::DeckDraft {
        id: "msjc01-tests".into(),
        name: "MSJC01机制测试".into(),
        description: String::new(),
        society_id: Some("MSJC01".into()),
        cards,
        rules_version: catalog::RULES_VERSION.into(),
        card_pool_version: catalog::POOL_VERSION.into(),
        engine_version: catalog::ENGINE_VERSION.into(),
        updated_at: String::new(),
    }
}
pub(super) fn game() -> Game {
    let mut g = Game::new_with_deck(
        "msjc01-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        draft(25, 50),
        9,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), draft(25, 50)).unwrap();
    }
    for s in 0..4 {
        g.apply(s, Action::new("ready")).unwrap();
    }
    g.apply(0, Action::new("start")).unwrap();
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
        g.players[s].deck.clear();
        g.players[s].assets.clear();
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
pub(super) fn activate(g: &Game, s: usize, key: &str) -> Action {
    Action {
        card_id: Some(source(g, s)),
        ability_id: Some(key.into()),
        ..Action::new("activate")
    }
}
pub(super) fn fund(g: &mut Game, s: usize, n: usize) {
    for _ in 0..n {
        let c = g.make_card("JC125", s);
        g.players[s].assets.push(c);
    }
}
pub(super) fn pass_top(g: &mut Game) {
    let n = g.stack.len();
    assert!(n > 0);
    for _ in 0..32 {
        if g.stack.len() < n || g.pending.is_some() {
            return;
        }
        let s = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(s, Action::new("pass")).unwrap();
    }
    panic!("bounded frame did not progress");
}
pub(super) fn roundtrip(g: &mut Game) {
    let state = serde_json::to_string(g).unwrap();
    *g = Game::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), state);
}
fn next_turn(g: &mut Game) {
    let t = g.turn;
    for _ in 0..200 {
        if g.turn > t {
            return;
        }
        assert!(g.pending.is_none());
        let s = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(s, Action::new("pass")).unwrap();
    }
    panic!("bounded turn did not progress");
}
pub(super) fn rejected(g: &mut Game, s: usize, a: Action) {
    let before = serde_json::to_string(g).unwrap();
    assert!(g.apply(s, a).is_err());
    assert_eq!(serde_json::to_string(g).unwrap(), before);
}
#[test]
fn msjc01_printed_program_and_registry_admission() {
    let d = crate::society::definition("MSJC01").unwrap();
    assert_eq!(
        (&*d.card.name, &*d.subtitle, &*d.card.color),
        ("帷幕守望", "世界守护者", "黄")
    );
    assert_eq!(d.card.subtypes, ["法师结社"]);
    assert!(d.card.unique && d.printed_cost.is_none());
    assert_eq!(d.starting_hand, 6);
    let specs = &crate::rules::definition("MSJC01").abilities;
    assert_eq!(specs.len(), 2);
    assert_eq!(
        serde_json::to_value(&specs[0]).unwrap(),
        serde_json::to_value(&crate::rules::definition("MSJC09").abilities[0]).unwrap()
    );
    assert!(specs[1].once_per_game && specs[1].targets.is_empty());
    assert_eq!(
        serde_json::to_value(&specs[1].costs).unwrap(),
        serde_json::json!([{"Assets":4},"ExhaustSource"])
    );
    assert!(
        matches!(&specs[1].ops[0],Op::Search{filter:CardFilter::PrintedColorAndUnique{color},to_top:false,optional:false,..} if color=="黄")
    );
    let mut invalid = specs[1].clone();
    invalid.event = Some(Event::Enter);
    assert!(crate::rules::validate_ability("fixture", &invalid).is_err());
    assert_eq!(
        catalog::catalog()
            .cards
            .iter()
            .filter(|c| c.color == "黄" && c.unique)
            .count(),
        2
    );
}
#[test]
fn msjc01_24_25_27_yellow_and_society_excluded_from_both_counts() {
    assert!(deck::validate(draft(24, 50))
        .unwrap_err()
        .contains("当前24张"));
    for yellow in [25, 27] {
        let d = deck::validate(draft(yellow, 50)).unwrap();
        assert_eq!(d.cards.iter().map(|c| c.count).sum::<usize>(), 50);
    }
    assert!(deck::validate(draft(25, 49))
        .unwrap_err()
        .contains("至少需要50"));
    let mut d = draft(25, 50);
    d.cards.push(catalog::DeckEntry {
        card_id: "MSJC01".into(),
        count: 1,
    });
    assert!(deck::validate(d).is_err());
    let mut d = draft(25, 50);
    d.cards
        .iter_mut()
        .find(|c| c.card_id == "JC008")
        .unwrap()
        .count = 4;
    assert!(deck::validate(d).unwrap_err().contains("最多3"));
}
#[test]
fn msjc01_paid_declaration_consumes_before_resolution_four_seats_independent() {
    let mut g = game();
    for s in 0..4 {
        g.begin_window(Window::Action(g.team(s)));
        fund(&mut g, s, 4);
        let a = activate(&g, s, SEARCH);
        g.apply(s, a).unwrap();
        assert_eq!(
            g.players[s]
                .society_zone
                .used_once_per_game
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            [SEARCH]
        );
        for other in s + 1..4 {
            assert!(g.players[other].society_zone.used_once_per_game.is_empty());
        }
        for viewer in 0..4 {
            assert_eq!(
                g.view(viewer).society_zones[s]
                    .card
                    .as_ref()
                    .unwrap()
                    .used_once_per_game,
                Some(vec![SEARCH.into()])
            );
        }
        assert_eq!(
            g.players[s].assets.iter().filter(|c| c.exhausted).count(),
            4
        );
        roundtrip(&mut g);
        pass_top(&mut g);
    }
    assert!(g
        .players
        .iter()
        .all(|p| p.society_zone.used_once_per_game.len() == 1));
}
#[test]
fn msjc01_insufficient_cost_exhausted_source_and_missing_ability_are_atomic() {
    for bad in ["cost", "exhausted", "missing"] {
        let mut g = game();
        fund(&mut g, 0, if bad == "cost" { 3 } else { 4 });
        if bad == "exhausted" {
            g.players[0].society_zone.card.as_mut().unwrap().exhausted = true;
        }
        let mut a = activate(&g, 0, SEARCH);
        if bad == "missing" {
            a.ability_id = None;
        }
        rejected(&mut g, 0, a);
        assert!(g.players[0].society_zone.used_once_per_game.is_empty());
        assert!(g.players[0].assets.iter().all(|c| !c.exhausted));
    }
}
#[test]
fn msjc01_empty_search_allowed_consumed_and_shuffles_without_gain() {
    let mut g = game();
    fund(&mut g, 0, 4);
    let a = activate(&g, 0, SEARCH);
    assert!(g.legal_actions(0).iter().any(|x| x.action == a));
    let n = g.players[0].deck.len();
    let before = g.random;
    g.apply(0, a).unwrap();
    pass_top(&mut g);
    assert_ne!(g.random, before);
    assert_eq!(g.players[0].deck.len(), n);
    assert!(g.players[0].hand.is_empty());
    assert!(g.log.iter().any(|e| e.text.contains("没有符合条件的牌")));
    assert!(g.players[0]
        .society_zone
        .used_once_per_game
        .contains(SEARCH));
}
#[test]
fn msjc01_legal_actions_do_not_query_private_search_availability() {
    let mut g = game();
    fund(&mut g, 0, 4);
    let mut no_deck = g.clone();
    no_deck.players[0].deck.clear();
    assert_eq!(
        serde_json::to_string(&g.legal_actions(0)).unwrap(),
        serde_json::to_string(&no_deck.legal_actions(0)).unwrap()
    );
    let a = activate(&no_deck, 0, SEARCH);
    no_deck.apply(0, a).unwrap();
    pass_top(&mut no_deck);
    assert!(no_deck.players[0]
        .society_zone
        .used_once_per_game
        .contains(SEARCH));
}
#[test]
fn msjc01_printed_color_and_unique_filter_accepts_all_kinds_not_unique_keyword() {
    let f = CardFilter::PrintedColorAndUnique {
        color: "黄".into()
    };
    for kind in ["character", "spell", "attachment"] {
        let mut d = catalog::card("JC008").clone();
        d.kind = kind.into();
        d.unique = true;
        assert!(f.matches(&d));
        d.unique = false;
        d.keywords = vec!["唯一".into()];
        assert!(!f.matches(&d));
        d.unique = true;
        d.color = "中立".into();
        assert!(!f.matches(&d));
    }
    assert_eq!(
        catalog::catalog()
            .cards
            .iter()
            .filter(|d| f.matches(d))
            .map(|d| d.id.as_str())
            .collect::<Vec<_>>(),
        ["WM003", "LC01"]
    );
}
#[test]
fn msjc01_search_hit_fixtures_reuse_private_choice_reveal_fresh_hand_and_shuffle() {
    for hits in [1, 3] {
        let mut g = game();
        fund(&mut g, 0, 4);
        for _ in 0..hits {
            let c = g.make_card("LC23", 0);
            g.players[0].deck.push(c);
        }
        let a = activate(&g, 0, SEARCH);
        g.apply(0, a).unwrap();
        // Explicit paid-frame filter fixture. LC23 remains neutral, not a formal yellow target.
        if let Op::Search { filter, .. } =
            &mut g.stack.last_mut().unwrap().frame.as_mut().unwrap().steps[0].op
        {
            *filter = CardFilter::PrintedColorAndUnique {
                color: "中立".into(),
            };
        }
        pass_top(&mut g);
        let p = g.pending.clone().unwrap();
        assert_eq!(p.choice.options.len(), hits);
        assert_eq!(p.choice.min, Some(1));
        for viewer in 1..4 {
            assert!(g.view(viewer).pending_choice.is_none());
            assert!(g.view(viewer).waiting_choice.is_some());
        }
        roundtrip(&mut g);
        rejected(
            &mut g,
            0,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        );
        let id = p.choice.options[0].id.clone();
        let n = g.players[0].deck.len();
        let rng = g.random;
        g.apply(
            0,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![id.clone()]),
                ..Action::new("choose")
            },
        )
        .unwrap();
        let c = g.players[0]
            .hand
            .iter()
            .find(|c| c.definition == "LC23")
            .unwrap();
        assert_ne!(c.id, id);
        assert_eq!(g.players[0].deck.len(), n - 1);
        assert_ne!(g.random, rng);
        assert!(g.log.iter().any(|e| e.text.contains("展示检索的 安格鲁")));
        assert_eq!(g.players[0].society_zone.used_once_per_game.len(), 1);
    }
}
#[test]
fn msjc01_paid_cancelled_frame_does_not_refund_usage_or_costs() {
    let mut g = game();
    fund(&mut g, 0, 4);
    let a = activate(&g, 0, SEARCH);
    g.apply(0, a).unwrap();
    // Explicit cancellation checkpoint; no admitted counter-card is claimed.
    g.stack.last_mut().unwrap().frame.as_mut().unwrap().guard = GuardState::Cancelled;
    roundtrip(&mut g);
    pass_top(&mut g);
    assert!(g.players[0]
        .society_zone
        .used_once_per_game
        .contains(SEARCH));
    assert_eq!(
        g.players[0].assets.iter().filter(|c| c.exhausted).count(),
        4
    );
    assert!(g.players[0].hand.is_empty());
}
#[test]
fn msjc01_turn_ready_reset_does_not_refund_and_draw_ability_is_independent() {
    let mut g = game();
    fund(&mut g, 0, 8);
    let a = activate(&g, 0, DRAW);
    g.apply(0, a).unwrap();
    pass_top(&mut g);
    assert!(g.players[0].society_zone.used_once_per_game.is_empty());
    next_turn(&mut g);
    g.begin_window(Window::Action(0));
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
    let a = activate(&g, 0, DRAW);
    g.apply(0, a).unwrap();
    pass_top(&mut g);
    assert_eq!(g.players[0].society_zone.used_once_per_game.len(), 1);
}
#[test]
fn msjc01_same_name_new_instance_cannot_bypass_stable_source_budget() {
    let mut g = game();
    fund(&mut g, 0, 8);
    let a = activate(&g, 0, SEARCH);
    g.apply(0, a).unwrap();
    pass_top(&mut g);
    let c = g.players[0].society_zone.card.take().unwrap();
    let old = c.id.clone();
    let mut c = g.fresh(c);
    c.exhausted = false;
    assert_ne!(c.id, old);
    g.players[0].society_zone.card = Some(c);
    roundtrip(&mut g);
    let a = activate(&g, 0, SEARCH);
    rejected(&mut g, 0, a);
    assert!(g.players[0]
        .society_zone
        .used_once_per_game
        .contains(SEARCH));
}
#[test]
fn msjc01_new_game_restart_is_the_only_usage_reset() {
    let mut g = game();
    fund(&mut g, 0, 4);
    let a = activate(&g, 0, SEARCH);
    g.apply(0, a).unwrap();
    pass_top(&mut g);
    let old = source(&g, 0);
    // Explicit finished-game checkpoint; restart is a normal Action.
    g.status = "finished".into();
    g.apply(0, Action::new("restart")).unwrap();
    assert!(g
        .players
        .iter()
        .all(|p| p.society_zone.used_once_per_game.is_empty()));
    assert_ne!(source(&g, 0), old);
    assert_eq!(g.players[0].hand.len(), 6);
}
#[test]
fn msjc01_room_paid_save_journal_replay_and_stale_retry_do_not_reconsume() {
    let mut g = game();
    fund(&mut g, 0, 4);
    let a = activate(&g, 0, SEARCH);
    let room = RoomEnvelope::from_game(g);
    let cmd = RoomCommand {
        command_id: "msjc01-paid".into(),
        expected_version: room.revision,
        action: SessionAction::Game { action: a },
    };
    let t = room.transition(0, Some(cmd.clone()), 1000).unwrap();
    assert_eq!(t.outcome, "accepted");
    assert_eq!(
        serde_json::to_string(&room.replay_events(&t.journal).unwrap()).unwrap(),
        t.state
    );
    let restored = RoomEnvelope::from_persisted(&t.state).unwrap();
    assert_eq!(restored.players[0].society_zone.used_once_per_game.len(), 1);
    let retry = restored.transition(0, Some(cmd), 1000).unwrap();
    assert!(!retry.changed);
    assert_eq!(retry.state, t.state);
}
