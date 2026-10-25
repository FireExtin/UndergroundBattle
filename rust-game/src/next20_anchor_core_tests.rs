//! Real BQ009 and BQ033 command paths. Explicit sealed initial layouts do not
//! claim that an implemented hand/deck-to-society producer exists.
use crate::{catalog, model::*, next20_blue_vampires as blue, unit_support::*};
fn require_anchor() {
    assert!(
        catalog::catalog().cards.iter().any(|c| c.id == "BQ009"),
        "verified real BQ009 dependency is absent"
    );
}
fn anchor(g: &mut Game, owner: usize, society: usize) -> String {
    require_anchor();
    let c = g.make_card("BQ009", owner);
    let id = c.id.clone();
    g.sealed_cards.push(SealedCard {
        card: c,
        host_id: format!("society:p{society}"),
    });
    id
}
fn choices(g: &mut Game) {
    for _ in 0..50 {
        if g.pending.is_some() {
            return;
        }
        assert!(!g.stack.is_empty());
        pass(g);
    }
    panic!("choice");
}
#[test]
fn next20_anchor_core_original_printed_no_effect_play_does_not_self_seal() {
    require_anchor();
    let mut g = game();
    let source = held(&mut g, "BQ009", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            ..Action::new("play")
        },
    );
    top(&mut g);
    assert!(g.sealed_cards.is_empty());
    assert!(g.pending.is_none());
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "BQ009")
            .count(),
        1
    );
}
#[test]
fn next20_anchor_core_deploy_and_paid_reveal_consume_actual_anchor_receipt_once() {
    require_anchor();
    for reveal in [false, true] {
        for owner in [0, 2] {
            let mut g = game();
            fund(&mut g, 0, "JC125", 3);
            let paid_anchor = anchor(&mut g, owner, 0);
            let before_decks = g.players.iter().map(|p| p.deck.len()).collect::<Vec<_>>();
            let source = if reveal {
                let id = field(&mut g, "BQ033", 0);
                g.board_mut(&id).unwrap().face_down = true;
                id
            } else {
                held(&mut g, "BQ033", 0)
            };
            let action = Action {
                card_id: Some(source.clone()),
                region: (!reveal).then_some(0),
                cost_selected: Some(vec![paid_anchor.clone()]),
                ..Action::new(if reveal { "reveal" } else { "deploy" })
            };
            assert!(g.legal_actions(0).iter().any(|a| a.action == action));
            act(&mut g, 0, action.clone());
            assert_eq!(g.resources(0), 0);
            assert!(!g.sealed_cards.iter().any(|s| s.card.id == paid_anchor));
            for (seat, p) in g.players.iter().enumerate() {
                assert_eq!(
                    p.deck.len(),
                    before_decks[seat] + usize::from(seat == owner)
                );
            }
            let f = g.stack.last().unwrap().frame.as_ref().unwrap();
            let [PaidCost::Assets(ids), PaidCost::BQ033Anchor(receipt)] = f.already_paid.as_slice()
            else {
                panic!("real anchor receipt")
            };
            assert_eq!(ids.len(), 3);
            assert_eq!(receipt.anchor.id, paid_anchor);
            assert_eq!(receipt.anchor.owner, owner);
            assert_eq!(receipt.payer, 0);
            assert!(g.players[owner]
                .deck
                .iter()
                .any(|c| c.id == receipt.deck_instance
                    && c.definition == "BQ009"
                    && c.controller == owner));
            let original = serde_json::to_value(&g).unwrap();
            let mut bad = original.clone();
            bad["stack"][0]["frame"]["already_paid"][1]["BQ033Anchor"]["payer"] = 2.into();
            assert!(Game::from_persisted(&bad.to_string()).is_err());
            let mut bad = original;
            bad["stack"][0]["card"]["id"] = source.into();
            assert!(Game::from_persisted(&bad.to_string()).is_err());
            reject(&mut g, 0, action);
            choices(&mut g);
            assert!(
                matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key==blue::DISCARD)
            );
            choose(&mut g, vec![]);
            assert_eq!(
                g.regions[0]
                    .cards
                    .iter()
                    .filter(|c| c.definition == "BQ033" && !c.face_down)
                    .count(),
                1
            );
            restore(&mut g);
        }
    }
}
#[test]
fn next20_anchor_core_bad_anchor_zone_or_payment_is_atomic() {
    require_anchor();
    let mut g = game();
    fund(&mut g, 0, "JC125", 3);
    let wrong = anchor(&mut g, 0, 1);
    let correct = anchor(&mut g, 0, 0);
    let source = held(&mut g, "BQ033", 0);
    for selected in [vec![wrong], vec![correct.clone(), correct.clone()], vec![]] {
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                region: Some(0),
                cost_selected: Some(selected),
                ..Action::new("deploy")
            },
        );
    }
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            region: Some(0),
            cost_selected: Some(vec![correct]),
            ..Action::new("deploy")
        },
    );
    assert_eq!(g.sealed_cards.len(), 1);
}
#[test]
fn next20_anchor_core_sealing_on_own_society_emits_real_optional_modes_and_forecast() {
    require_anchor();
    for mode in [None, Some("forecast"), Some("repress")] {
        let mut g = game();
        g.regions[0].influence[1] = 1;
        let card = g.make_card("BQ009", 0);
        // Existing primitive after a validated producer; this is not a newly
        // invented public command or a claim that the producer chain is ready.
        g.seal_card_on_valid_host(card, 0, "society:p0");
        g.drive().unwrap();
        restore(&mut g);
        let p = g.pending.as_ref().unwrap();
        assert_eq!(p.seat, 0);
        assert_eq!(
            p.choice
                .options
                .iter()
                .map(|o| o.id.as_str())
                .collect::<Vec<_>>(),
            ["repress", "forecast"]
        );
        if let Some(mode) = mode {
            choose(&mut g, vec![mode.into()]);
            if mode == "repress" {
                choose(&mut g, vec!["p2".into()]);
            }
            choices(&mut g);
            if mode == "forecast" {
                let p = g.pending.clone().unwrap();
                let top = p.choice.options[0].id.clone();
                act(
                    &mut g,
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id),
                        top: Some(vec![]),
                        bottom: Some(vec![top]),
                        ..Action::new("choose")
                    },
                );
            } else {
                assert_eq!(g.pending.as_ref().unwrap().seat, 2);
                choose(&mut g, vec!["region:0".into()]);
                assert_eq!(g.regions[0].influence[1], 0);
            }
        } else {
            choose(&mut g, vec![]);
        }
        assert!(g.pending.is_none());
        assert_eq!(g.sealed_cards.len(), 1);
        restore(&mut g);
    }
}
#[test]
fn next20_anchor_core_other_hosts_do_not_emit_society_trigger() {
    require_anchor();
    for on_character in [false, true] {
        let mut g = game();
        let host = if on_character {
            field(&mut g, "JC125", 0)
        } else {
            "society:p1".into()
        };
        let c = g.make_card("BQ009", 0);
        g.seal_card_on_valid_host(c, 0, &host);
        g.drive().unwrap();
        assert!(g.pending.is_none());
        restore(&mut g);
    }
}
#[test]
fn next20_anchor_core_copy_limit_is_conditional_and_public_admission_remains_closed() {
    require_anchor();
    let c = catalog::card("BQ009");
    assert!(!c.supported);
    assert_eq!(crate::deck::copy_limit(c), Some(3));
    assert_eq!(crate::deck::copy_limit_for_society(c, Some("MSJC10")), None);
    assert_eq!(
        crate::deck::copy_limit_for_society(c, Some("MSJC09")),
        Some(3)
    );
    // MSJC10 and the actual supply cards remain outside this implementation.
    assert!(crate::society::definition("MSJC10").is_err());
    assert!(["BQ033", "BQ034", "CQ25", "XG25"]
        .iter()
        .all(|id| !catalog::card(id).supported));
}
#[test]
fn next20_anchor_core_recovery_rejects_changed_event_host_mode_and_private_choice() {
    require_anchor();
    let mut g = game();
    let c = g.make_card("BQ009", 0);
    g.seal_card_on_valid_host(c, 0, "society:p0");
    g.drive().unwrap();
    let saved = serde_json::to_value(&g).unwrap();
    for field in ["seat", "max", "host", "mode"] {
        let mut bad = saved.clone();
        match field {
            "seat" => bad["pending"]["seat"] = 2.into(),
            "max" => bad["pending"]["choice"]["max"] = 2.into(),
            "host" => {
                bad["pending"]["resolution"]["Declare"]["declaration"]["source"]
                    ["attachment_host_instance"] = "society:p1".into()
            }
            _ => {
                bad["pending"]["resolution"]["Declare"]["declaration"]["ability"]["modes"][1]
                    ["ops"] = serde_json::json!([])
            }
        }
        assert!(Game::from_persisted(&bad.to_string()).is_err(), "{field}");
    }
    choose(&mut g, vec!["forecast".into()]);
    choices(&mut g);
    let mut bad = serde_json::to_value(&g).unwrap();
    bad["pending"]["choice"]["amount"] = 2.into();
    assert!(Game::from_persisted(&bad.to_string()).is_err());
    let p = g.pending.clone().unwrap();
    reject(
        &mut g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            top: Some(vec![]),
            bottom: Some(vec![p.choice.options[0].id.clone()]),
            cost_selected: Some(vec!["forged".into()]),
            ..Action::new("choose")
        },
    );
}
