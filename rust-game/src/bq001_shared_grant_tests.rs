//! Explicit unit layout for the existing CQ15 host grant on closed BQ001.
//! All attach, activation, payment and choice transitions use engine actions
//! and unit_support persistence roundtrips. Not a public construction proof.
use crate::{catalog, import20_swords as swords, jc029_tests::pass_top, model::*, unit_support::*};

#[test]
fn bq001_shared_grant_sword_payment_private_search_and_continuation_use_original_host() {
    assert!(!catalog::card("BQ001").supported);
    let mut g = game();
    let host = field(&mut g, "BQ001", 0);
    fund(&mut g, 0, "LC16", 6);
    let sword = held(&mut g, "CQ15", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(sword),
            target_id: Some(host.clone()),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    let sword = g
        .attachments
        .iter()
        .find(|a| a.card.definition == "CQ15")
        .unwrap()
        .card
        .id
        .clone();
    let target = field(&mut g, "JC125", 2);
    let enemy_hand = held(&mut g, "JC125", 2);
    g.begin_window(Window::Action(g.team(0)));
    let activation = Action {
        card_id: Some(host.clone()),
        ability_id: Some(format!("{}{}", swords::PREFIX, sword)),
        target_id: Some(target.clone()),
        ..Action::new("activate")
    };
    assert!(g.legal_actions(0).iter().any(|a| a.action == activation));
    act(&mut g, 0, activation);
    assert!(g.board(&host).unwrap().1.exhausted);
    assert!(g.attachments.iter().all(|a| a.card.id != sword));
    assert_eq!(
        g.players[0]
            .hand
            .iter()
            .filter(|c| c.definition == "CQ15" && c.id != sword)
            .count(),
        1
    );
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    let pending = g.pending.clone().unwrap();
    assert_eq!(pending.choice.kind, swords::SEARCH);
    assert_eq!(pending.seat, 0);
    let ChoiceResolution::Frame { frame, .. } = &pending.resolution else {
        panic!("expected shared sword frame")
    };
    assert_eq!(frame.source.card.definition, "BQ001");
    assert!(g.validated_cross_card_frame(frame).unwrap());
    assert!(pending.choice.options.iter().any(|o| o.id == enemy_hand));
    for viewer in 1..4 {
        assert!(g.view(viewer).pending_choice.is_none());
    }
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(pending.choice.id.clone()),
            selected: Some(vec![enemy_hand.clone()]),
            amount: Some(1),
            ..Action::new("choose")
        },
    );
    choose(&mut g, vec![enemy_hand.clone()]);
    assert!(g.pending.is_none());
    assert!(g.stack.is_empty());
    assert!(g.effects.is_empty());
    assert_eq!(g.sealed_cards.len(), 1);
    assert_eq!(g.sealed_cards[0].host_id, "society:p0");
    assert_eq!(g.sealed_cards[0].card.owner, 2);
    assert!(!g.players[2].hand.iter().any(|c| c.id == enemy_hand));
    let state = serde_json::to_string(&g).unwrap();
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(pending.choice.id),
            selected: Some(vec![enemy_hand]),
            ..Action::new("choose")
        },
    );
    assert_eq!(serde_json::to_string(&g).unwrap(), state);
    assert!(!catalog::card("BQ001").supported);
}
