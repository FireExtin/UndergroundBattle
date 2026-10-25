//! Explicit unit layouts; all revelation/sealing/draw operations use actual
//! engine actions. These cases do not claim a public construction trajectory.
use crate::{catalog, model::*, rules::*, unit_support::*};
const ID: &str = "BQ001";
const CHOICE: &str = "bq001-star-hand";
fn require_card() {
    assert!(
        catalog::catalog().cards.iter().any(|c| c.id == ID),
        "original BQ001 is not registered"
    );
}
fn yellow() -> String {
    catalog::catalog()
        .cards
        .iter()
        .find(|c| {
            c.supported
                && c.color == "黄"
                && matches!(c.kind.as_str(), "character" | "spell" | "attachment")
        })
        .unwrap()
        .id
        .clone()
}
fn stop_at(g: &mut Game, kind: &str) {
    for _ in 0..100 {
        if let Some(p) = &g.pending {
            assert_eq!(p.choice.kind, kind, "unexpected choice {}", p.choice.title);
            return;
        }
        assert!(!g.stack.is_empty(), "missing choice {kind}");
        pass(g);
    }
    panic!("bounded choice {kind}");
}
fn decline_pending(g: &mut Game) {
    for _ in 0..100 {
        if let Some(p) = &g.pending {
            assert!(
                matches!(p.resolution, ChoiceResolution::Declare { .. }),
                "unexpected choice {}",
                p.choice.kind
            );
            assert_eq!(p.choice.min, Some(0));
            choose(g, vec![]);
        } else if !g.stack.is_empty() {
            pass(g);
        } else {
            return;
        }
    }
    panic!("bounded optional trigger cleanup");
}
#[test]
fn bq001_printed_whole_card_is_registered_closed_and_image_specific() {
    require_card();
    let d = catalog::card(ID);
    assert_eq!(
        (d.cost, d.defense, d.unique, d.supported),
        (1, Some(1), false, false)
    );
    assert_eq!(d.color, "黄");
    assert_eq!(d.loyalty, ["黄色"]);
    assert_eq!(d.magic, "心灵");
    assert_eq!(d.subtypes, ["人类", "法师", "学者"]);
    assert_eq!(
        d.permanent_icons,
        Icons {
            investigation: 0,
            combat: 0,
            influence: 1
        }
    );
    assert_eq!(d.temporary_icons, Icons::default());
    let program = serde_json::to_value(definition(ID)).unwrap();
    assert_eq!(
        program["modifiers"],
        serde_json::json!(["BQ001MysticAnchorShown"])
    );
    assert_eq!(program["abilities"].as_array().unwrap().len(), 1);
    assert_eq!(program["abilities"][0]["event"], "Reveal");
}
#[test]
fn bq001_paid_reveal_selects_only_actor_star_hand_then_seals_and_draws_once() {
    require_card();
    let mut g = game();
    fund(&mut g, 0, &yellow(), 3);
    let source = field(&mut g, ID, 0);
    g.board_mut(&source).unwrap().face_down = true;
    let anchor = held(&mut g, "BQ009", 0);
    let nonstar = held(&mut g, "JC125", 0);
    let other = held(&mut g, "BQ009", 2);
    let money = g.resources(0);
    let deck = g.players[0].deck.len();
    let hand = g.players[0].hand.len();
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    );
    assert_eq!(g.resources(0), money - 1);
    stop_at(&mut g, "trigger");
    choose(&mut g, vec!["accept".into()]);
    stop_at(&mut g, CHOICE);
    let p = g.pending.clone().unwrap();
    assert_eq!(p.seat, 0);
    assert_eq!(
        p.choice
            .options
            .iter()
            .map(|o| o.id.as_str())
            .collect::<Vec<_>>(),
        [anchor.as_str()]
    );
    for (seat, id) in [(0, nonstar), (0, other), (2, anchor.clone())] {
        reject(
            &mut g,
            seat,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec![id]),
                ..Action::new("choose")
            },
        );
    }
    choose(&mut g, vec![anchor.clone()]);
    assert_eq!(g.resources(0), money - 1);
    assert_eq!(g.players[0].deck.len(), deck - 1);
    assert_eq!(g.players[0].hand.len(), hand);
    assert!(!g.players[0].hand.iter().any(|c| c.id == anchor));
    let sealed = g
        .sealed_cards
        .iter()
        .find(|s| s.card.definition == "BQ009")
        .unwrap();
    assert_eq!(sealed.host_id, "society:p0");
    assert_eq!(sealed.card.owner, 0);
    assert_ne!(sealed.card.id, anchor);
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![anchor]),
            ..Action::new("choose")
        },
    );
    decline_pending(&mut g);
    assert_eq!(g.players[0].deck.len(), deck - 1);
    assert_eq!(g.sealed_cards.len(), 1);
    assert!(g.effects.is_empty());
}
#[test]
fn bq001_normal_face_up_deploy_does_not_seal_or_draw() {
    require_card();
    let mut g = game();
    fund(&mut g, 0, &yellow(), 1);
    let source = held(&mut g, ID, 0);
    held(&mut g, "BQ009", 0);
    let deck = g.players[0].deck.len();
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    drain(&mut g);
    assert!(g.sealed_cards.is_empty());
    assert_eq!(g.players[0].deck.len(), deck);
    assert!(g.pending.is_none());
}

fn invalid_in_both_loaders(g: &Game, v: serde_json::Value) {
    assert!(
        std::panic::catch_unwind(|| Game::from_persisted(&v.to_string()))
            .unwrap()
            .is_err()
    );
    let mut room = serde_json::to_value(crate::jc029_tests::envelope(g)).unwrap();
    room["game"] = v;
    assert!(
        std::panic::catch_unwind(|| crate::room::RoomEnvelope::from_persisted(&room.to_string()))
            .unwrap()
            .is_err()
    );
}
fn actual_hand_contest(g: &mut Game) {
    for s in 0..4 {
        held(g, "JC125", s);
    }
    fund(g, 0, "JC125", 2);
    let source = held(g, "BQ090", 0);
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.card_id.as_deref() == Some(&source)
                && a.action.ability_id.as_deref() == Some(crate::import20_contest::HAND)
        })
        .unwrap()
        .action;
    act(g, 0, a);
    stop_at(g, crate::mystic_contest::COMMIT_KIND);
    for _ in 0..4 {
        let p = g.pending.as_ref().unwrap();
        assert_eq!(p.choice.kind, crate::mystic_contest::COMMIT_KIND);
        let id = g.players[p.seat]
            .hand
            .iter()
            .find(|c| c.definition == "JC125")
            .unwrap()
            .id
            .clone();
        choose(g, vec![id]);
    }
    for _ in 0..8 {
        if g.pending.is_none() {
            break;
        }
        let p = g.pending.as_ref().unwrap();
        assert_eq!(p.choice.kind, crate::mystic_contest::REWARD_KIND);
        choose(g, vec![]);
    }
    assert!(
        g.mystic_contest.is_none()
            && g.pending.is_none()
            && g.stack.is_empty()
            && g.effects.is_empty()
    );
    restore(g);
}
fn sealed_layout(g: &mut Game, owner: usize, host: &str) {
    let c = g.make_card("BQ009", owner);
    let c = g.reset_zone_card(c);
    g.sealed_cards.push(SealedCard {
        card: c,
        host_id: host.into(),
    });
}
#[test]
fn bq001_continuous_uses_own_shown_seat_all_face_up_copies_and_actual_society_anchors() {
    require_card();
    let mut g = game();
    let first = field(&mut g, ID, 0);
    let exhausted = field(&mut g, ID, 0);
    g.board_mut(&exhausted).unwrap().exhausted = true;
    let hidden = field(&mut g, ID, 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    field(&mut g, ID, 1);
    sealed_layout(&mut g, 0, "society:p0");
    sealed_layout(&mut g, 1, "society:p0");
    sealed_layout(&mut g, 1, "society:p1");
    sealed_layout(&mut g, 0, &first);
    actual_hand_contest(&mut g);
    let o = &g.mystic_history[0];
    assert_eq!(
        o.shown
            .iter()
            .map(|s| (s.seat, s.value))
            .collect::<Vec<_>>(),
        [(0, 4), (1, 1), (2, 0), (3, 0)]
    );
    let w = o.shown[0].anchor_witness.as_ref().unwrap();
    assert_eq!(w.sources.len(), 2);
    assert_eq!(w.anchors.len(), 2);
    assert!(w.sources.iter().any(|s| s.card.exhausted));
    assert_eq!(o.totals, [5, 0]);
    assert!(o.shown[2].anchor_witness.is_none());
    // The printed permanent influence icon is unchanged by the contest modifier.
    assert_eq!(catalog::card(ID).permanent_icons.influence, 1);
}
#[test]
fn bq001_continuous_zero_anchor_and_hidden_source_do_not_add_shown_value() {
    require_card();
    let mut g = game();
    field(&mut g, ID, 0);
    let hidden = field(&mut g, ID, 1);
    g.board_mut(&hidden).unwrap().face_down = true;
    sealed_layout(&mut g, 1, "society:p1");
    actual_hand_contest(&mut g);
    let o = &g.mystic_history[0];
    assert!(o.shown.iter().all(|s| s.value == 0));
    assert!(o.shown[0]
        .anchor_witness
        .as_ref()
        .unwrap()
        .anchors
        .is_empty());
    assert!(o.shown[1].anchor_witness.is_none());
}
#[test]
fn bq001_history_keeps_reveal_time_bonus_after_actual_source_destruction_and_rejects_forged_witness(
) {
    require_card();
    let mut g = game();
    let source = field(&mut g, ID, 0);
    sealed_layout(&mut g, 0, "society:p0");
    actual_hand_contest(&mut g);
    let base = serde_json::to_value(&g).unwrap();
    let saved = base["mystic_history"][0].clone();
    assert_eq!(saved["shown"][0]["value"], 1);
    for case in 0..7 {
        let mut v = base.clone();
        let shown = &mut v["mystic_history"][0]["shown"][0];
        match case {
            0 => shown["anchor_witness"]["sources"][0]["card"]["controller"] = 2.into(),
            1 => {
                let x = shown["anchor_witness"]["sources"][0].clone();
                shown["anchor_witness"]["sources"]
                    .as_array_mut()
                    .unwrap()
                    .push(x);
            }
            2 => {
                let x = shown["anchor_witness"]["anchors"][0].clone();
                shown["anchor_witness"]["anchors"]
                    .as_array_mut()
                    .unwrap()
                    .push(x);
            }
            3 => shown["anchor_witness"]["anchors"][0]["host_id"] = "society:p1".into(),
            4 => shown["anchor_witness"]["anchors"][0]["card"]["exhausted"] = true.into(),
            5 => shown["anchor_witness"]["sources"][0]["card"]["face_down"] = true.into(),
            _ => {
                shown.as_object_mut().unwrap().remove("anchor_witness");
            }
        }
        invalid_in_both_loaders(&g, v);
    }
    g.begin_window(Window::Action(g.team(2)));
    fund(&mut g, 2, "JC084", 2);
    let spell = held(&mut g, "BQ090", 2);
    let a = g
        .legal_actions(2)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&spell)
                && a.action.target_id.as_deref() == Some(&source)
        })
        .unwrap()
        .action;
    act(&mut g, 2, a);
    drain(&mut g);
    assert!(g.board(&source).is_none());
    assert_eq!(serde_json::to_value(&g.mystic_history[0]).unwrap(), saved);
    restore(&mut g);
}
#[test]
fn bq001_no_eligible_star_hand_still_draws_but_declined_trigger_does_not() {
    require_card();
    for accept in [false, true] {
        let mut g = game();
        fund(&mut g, 0, &yellow(), 1);
        let source = field(&mut g, ID, 0);
        g.board_mut(&source).unwrap().face_down = true;
        held(&mut g, "JC125", 0);
        let deck = g.players[0].deck.len();
        let hand = g.players[0].hand.len();
        act(
            &mut g,
            0,
            Action {
                card_id: Some(source),
                ..Action::new("reveal")
            },
        );
        stop_at(&mut g, "trigger");
        choose(
            &mut g,
            if accept {
                vec!["accept".into()]
            } else {
                vec![]
            },
        );
        decline_pending(&mut g);
        assert!(g.sealed_cards.is_empty());
        assert_eq!(g.players[0].deck.len(), deck - usize::from(accept));
        assert_eq!(g.players[0].hand.len(), hand + usize::from(accept));
    }
}
#[test]
fn bq001_reveal_can_seal_other_printed_star_hand_without_inventing_an_anchor() {
    require_card();
    let mut g = game();
    fund(&mut g, 0, &yellow(), 1);
    let source = field(&mut g, ID, 0);
    g.board_mut(&source).unwrap().face_down = true;
    let printed = catalog::catalog()
        .cards
        .iter()
        .find(|c| {
            c.supported
                && c.magic == "星辰"
                && c.id != "BQ009"
                && matches!(c.kind.as_str(), "character" | "spell" | "attachment")
        })
        .unwrap()
        .id
        .clone();
    let star = held(&mut g, &printed, 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    );
    stop_at(&mut g, "trigger");
    choose(&mut g, vec!["accept".into()]);
    stop_at(&mut g, CHOICE);
    choose(&mut g, vec![star]);
    decline_pending(&mut g);
    assert_eq!(g.sealed_cards.len(), 1);
    assert_eq!(g.sealed_cards[0].card.definition, printed);
    assert_ne!(printed, "BQ009");
}
#[test]
fn bq001_accepted_reveal_survives_real_destroy_or_control_response_and_uses_original_actor() {
    require_card();
    for response in ["BQ090", "JZ27"] {
        let mut g = game();
        fund(&mut g, 0, &yellow(), 1);
        let source = field(&mut g, ID, 0);
        g.board_mut(&source).unwrap().face_down = true;
        let anchor = held(&mut g, "BQ009", 0);
        held(&mut g, "BQ009", 2);
        fund(
            &mut g,
            2,
            if response == "BQ090" {
                "JC084"
            } else {
                "JC029"
            },
            catalog::card(response).cost as usize,
        );
        let spell = if response == "BQ090" {
            held(&mut g, response, 2)
        } else {
            let id = field(&mut g, response, 2);
            g.board_mut(&id).unwrap().face_down = true;
            id
        };
        act(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                ..Action::new("reveal")
            },
        );
        stop_at(&mut g, "trigger");
        choose(&mut g, vec!["accept".into()]);
        // Paid revelation replaces the old concealed instance before its trigger.
        let current = g
            .regions
            .iter()
            .flat_map(|r| &r.cards)
            .find(|c| c.definition == ID && c.controller == 0 && !c.face_down)
            .unwrap()
            .id
            .clone();
        assert_ne!(current, source);
        let source = current;
        let mut cast = false;
        for _ in 0..4 {
            if let Some(a) = g.legal_actions(2).into_iter().find(|a| {
                a.action.kind
                    == if response == "BQ090" {
                        "play"
                    } else {
                        "reveal"
                    }
                    && a.action.card_id.as_deref() == Some(&spell)
                    && (response != "BQ090" || a.action.target_id.as_deref() == Some(&source))
            }) {
                act(&mut g, 2, a.action);
                cast = true;
                break;
            }
            pass(&mut g);
        }
        assert!(cast, "actual response {response}");
        for _ in 0..80 {
            if let Some(p) = g.pending.clone() {
                if p.choice.kind == CHOICE {
                    break;
                }
                if p.choice.options.iter().any(|o| o.id == "accept") {
                    choose(&mut g, vec!["accept".into()]);
                } else if p.choice.options.iter().any(|o| o.id == source) {
                    choose(&mut g, vec![source.clone()]);
                } else {
                    panic!("unexpected control response choice {}", p.choice.kind);
                }
            } else {
                pass(&mut g);
            }
        }
        stop_at(&mut g, CHOICE);
        assert_eq!(g.pending.as_ref().unwrap().seat, 0);
        if response == "BQ090" {
            assert!(g.board(&source).is_none());
        } else {
            assert_eq!(g.board(&source).unwrap().1.controller, 2);
        }
        choose(&mut g, vec![anchor]);
        decline_pending(&mut g);
        assert_eq!(g.sealed_cards.len(), 1);
        assert_eq!(g.sealed_cards[0].host_id, "society:p0");
        assert_eq!(g.sealed_cards[0].card.owner, 0);
        assert_eq!(
            g.players[2]
                .hand
                .iter()
                .filter(|c| c.definition == "BQ009")
                .count(),
            1
        );
    }
}
#[test]
fn bq001_pending_restore_and_actions_reject_retargeted_steps_receipts_and_foreign_hand() {
    require_card();
    let mut g = game();
    fund(&mut g, 0, &yellow(), 1);
    let source = field(&mut g, ID, 0);
    g.board_mut(&source).unwrap().face_down = true;
    let anchor = held(&mut g, "BQ009", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    );
    stop_at(&mut g, "trigger");
    choose(&mut g, vec!["accept".into()]);
    stop_at(&mut g, CHOICE);
    let p = g.pending.clone().unwrap();
    let mut a = Action {
        choice_id: Some(p.choice.id),
        selected: Some(vec![anchor]),
        ..Action::new("choose")
    };
    a.amount = Some(1);
    reject(&mut g, 0, a);
    let base = serde_json::to_value(&g).unwrap();
    for case in 0..7 {
        let mut v = base.clone();
        let f = &mut v["pending"]["resolution"]["Frame"]["frame"];
        match case {
            0 => f["cursor"] = 0.into(),
            1 => f["actor"] = 2.into(),
            2 => f["steps"][0]["op"] = "Noop".into(),
            3 => f["already_paid"] = serde_json::json!([{"Assets":[]}]),
            4 => f["source"]["card"]["definition"] = "JC125".into(),
            5 => v["pending"]["choice"]["max"] = 2.into(),
            _ => v["pending"]["choice"]["options"][0]["id"] = "i1".into(),
        }
        invalid_in_both_loaders(&g, v);
    }
}

#[test]
fn bq001_character_transactions_reject_extra_payload_before_payment() {
    require_card();
    for kind in ["deploy", "conceal", "reveal"] {
        for extra in 0..3 {
            let mut g = game();
            fund(&mut g, 0, &yellow(), 3);
            let id = if kind == "reveal" {
                let id = field(&mut g, ID, 0);
                g.board_mut(&id).unwrap().face_down = true;
                id
            } else {
                held(&mut g, ID, 0)
            };
            let mut a = Action {
                card_id: Some(id.clone()),
                region: if kind == "reveal" { None } else { Some(0) },
                ..Action::new(kind)
            };
            match extra {
                0 => a.amount = Some(1),
                1 => a.target_id = Some(id),
                _ => a.cost_selected = Some(vec![]),
            };
            reject(&mut g, 0, a);
        }
    }
}
