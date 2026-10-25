//! Real checkout commands/persistence with explicit initial layouts. Death
//! replacement is deliberately not claimed: its shared pre-grave continuation
//! is still absent. Sealing producers here are CQ25's real search or setup.
use crate::{catalog, model::*, next20_black_wraiths as black, unit_support::*};

fn at(g: &mut Game, id: &str, seat: usize, region: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.regions[region].cards.push(c);
    id
}
fn grave(g: &mut Game, id: &str, owner: usize) -> String {
    let c = g.make_card(id, owner);
    let id = c.id.clone();
    g.players[owner].graveyard.push(c);
    id
}
fn until_choice(g: &mut Game) {
    for _ in 0..48 {
        if g.pending.is_some() {
            return;
        }
        assert!(!g.stack.is_empty());
        pass(g);
    }
    panic!("expected native pending");
}
fn setup_kill(g: &mut Game, count: usize) {
    fund(g, 2, "JC102", count * 2);
    for _ in 0..count {
        held(g, "JC102", 2);
    }
    g.begin_window(Window::Action(g.team(2)));
}
fn fire(g: &mut Game, target: &str) {
    let id = g.players[2]
        .hand
        .iter()
        .find(|c| c.definition == "JC102")
        .unwrap()
        .id
        .clone();
    let a = g
        .legal_actions(2)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_ref() == Some(&id)
                && a.action.target_id.as_deref() == Some(target)
        })
        .unwrap()
        .action;
    act(g, 2, a);
}

#[test]
fn next20_black_core_all_four_remain_closed_without_fake_anchor() {
    assert_current_serial_catalog();
    for id in ["BQ033", "BQ034", "CQ25", "XG25"] {
        assert!(!catalog::card(id).supported);
    }
    assert!(!catalog::catalog().cards.iter().any(|c| c.id == "BQ009"));
    assert!(catalog::card("CQ25").text.contains("敌方角色"));
}

#[test]
fn next20_black_core_board_move_uses_real_payment_target_filter_and_once_limit() {
    let mut g = game();
    let source = field(&mut g, "XG25", 0);
    let enemy = at(&mut g, "JC125", 2, 1);
    let friendly = at(&mut g, "JC125", 1, 1);
    let far = at(&mut g, "JC125", 2, 3);
    fund(&mut g, 0, "XG25", 2);
    for target in [&friendly, &far] {
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                ability_id: Some(black::MOVE.into()),
                target_id: Some(target.clone()),
                ..Action::new("activate")
            },
        );
    }
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.card_id.as_ref() == Some(&source)
                && a.action.target_id.as_ref() == Some(&enemy)
        })
        .unwrap()
        .action;
    act(&mut g, 0, a.clone());
    assert_eq!(g.resources(0), 1);
    top(&mut g);
    assert_eq!(g.board(&enemy).unwrap().0, 0);
    let another = at(&mut g, "JC125", 2, 1);
    reject(
        &mut g,
        0,
        Action {
            target_id: Some(another),
            ..a
        },
    );
}

#[test]
fn next20_black_core_sealed_owner_pays_not_prior_actor_or_host_controller() {
    let mut g = game();
    let host = field(&mut g, "JC125", 1);
    let card = g.make_card("XG25", 0);
    // Explicit initial sealed layout, produced by the normal sealing primitive.
    g.seal_card_on_valid_host(card, 2, &host);
    g.effects.clear();
    let source = g.sealed_cards[0].card.id.clone();
    let enemy = at(&mut g, "JC125", 2, 1);
    fund(&mut g, 0, "XG25", 2);
    fund(&mut g, 1, "XG25", 1);
    fund(&mut g, 2, "XG25", 1);
    for actor in [1, 2] {
        let mut trial = g.clone();
        trial.begin_window(Window::Action(trial.team(actor)));
        reject(
            &mut trial,
            actor,
            Action {
                card_id: Some(source.clone()),
                ability_id: Some(black::MOVE.into()),
                target_id: Some(enemy.clone()),
                ..Action::new("activate")
            },
        );
    }
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| a.action.card_id.as_ref() == Some(&source))
        .unwrap()
        .action;
    act(&mut g, 0, a.clone());
    top(&mut g);
    assert_eq!(g.resources(0), 1);
    assert_eq!(g.resources(1), 1);
    assert_eq!(g.resources(2), 1);
    assert_eq!(g.board(&enemy).unwrap().0, 0);
    assert!(g.board(&source).is_none());
    let view = g.sealed_card_view(&g.sealed_cards[0].card, 0);
    assert!(view.icons.is_none() && view.magic.is_none());
    let another = at(&mut g, "JC125", 2, 1);
    reject(
        &mut g,
        0,
        Action {
            target_id: Some(another),
            ..a
        },
    );
}

#[test]
fn next20_black_core_real_control_change_keeps_board_action_with_controller() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let enemy = at(&mut g, "JC125", 0, 1);
    let steal = held(&mut g, "JC129", 2);
    fund(&mut g, 2, "JC102", catalog::card("JC129").cost as usize + 1);
    g.begin_window(Window::Action(g.team(2)));
    act(
        &mut g,
        2,
        Action {
            card_id: Some(steal),
            target_id: Some(ghost.clone()),
            ..Action::new("play")
        },
    );
    top(&mut g);
    assert_eq!(g.board(&ghost).unwrap().1.controller, 2);
    let before = g.resources(2);
    let a = g
        .legal_actions(2)
        .into_iter()
        .find(|a| {
            a.action.card_id.as_ref() == Some(&ghost) && a.action.target_id.as_ref() == Some(&enemy)
        })
        .unwrap()
        .action;
    act(&mut g, 2, a);
    top(&mut g);
    assert_eq!(g.resources(2) + 1, before);
    assert_eq!(g.board(&enemy).unwrap().0, 0);
}

#[test]
fn next20_black_core_target_control_change_is_rechecked_before_payment() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 2);
    let target = at(&mut g, "JC125", 0, 1);
    let steal = held(&mut g, "JC129", 2);
    fund(&mut g, 2, "JC102", catalog::card("JC129").cost as usize + 1);
    g.begin_window(Window::Action(g.team(2)));
    act(
        &mut g,
        2,
        Action {
            card_id: Some(steal),
            target_id: Some(target.clone()),
            ..Action::new("play")
        },
    );
    top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.controller, 2);
    reject(
        &mut g,
        2,
        Action {
            card_id: Some(ghost),
            ability_id: Some(black::MOVE.into()),
            target_id: Some(target),
            ..Action::new("activate")
        },
    );
}

#[test]
fn next20_black_core_actual_death_search_seals_and_enables_owner_action() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    let victim = field(&mut g, "JC125", 2);
    let ghost = grave(&mut g, "XG25", 0);
    let enemy = at(&mut g, "JC125", 2, 1);
    fund(&mut g, 0, "CQ25", 1);
    setup_kill(&mut g, 1);
    fire(&mut g, &victim);
    until_choice(&mut g);
    choose(&mut g, vec!["accept".into()]);
    until_choice(&mut g);
    choose(&mut g, vec![ghost]);
    assert!(g
        .sealed_cards
        .iter()
        .any(|s| s.host_id == tiger && s.card.definition == "XG25"));
    // A normal later action step; all movement itself goes through Game::apply.
    g.begin_window(Window::Action(0));
    let source = g
        .sealed_cards
        .iter()
        .find(|s| s.card.definition == "XG25")
        .unwrap()
        .card
        .id
        .clone();
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.card_id.as_ref() == Some(&source)
                && a.action.target_id.as_ref() == Some(&enemy)
        })
        .unwrap()
        .action;
    act(&mut g, 0, a);
    top(&mut g);
    assert_eq!(g.board(&enemy).unwrap().0, 0);
}

#[test]
fn next20_black_core_departed_tiger_still_searches_shuffles_and_buries_found_card() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    let victim = field(&mut g, "JC125", 2);
    g.players[0].deck.clear();
    let ghost = g.make_card("XG25", 0);
    let old_ghost = ghost.id.clone();
    g.players[0].deck.push(ghost);
    let filler = g.make_card("JC125", 0);
    g.players[0].deck.push(filler);
    setup_kill(&mut g, 2);
    fire(&mut g, &victim);
    until_choice(&mut g);
    choose(&mut g, vec!["accept".into()]);
    fire(&mut g, &tiger);
    top(&mut g);
    assert!(g.board(&tiger).is_none());
    until_choice(&mut g);
    let random = g.random;
    choose(&mut g, vec![old_ghost.clone()]);
    assert!(!g.players[0].deck.iter().any(|c| c.definition == "XG25"));
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XG25" && c.id != old_ghost));
    assert!(g.sealed_cards.is_empty());
    // shuffle_player is called after extraction; one remaining card needs no PRNG draw.
    assert_eq!(g.random, random);
    restore(&mut g);
}

#[test]
fn next20_black_core_search_trigger_can_be_declined_without_creating_a_card() {
    let mut g = game();
    field(&mut g, "CQ25", 0);
    let victim = field(&mut g, "JC125", 2);
    let ghost = grave(&mut g, "XG25", 0);
    setup_kill(&mut g, 1);
    fire(&mut g, &victim);
    until_choice(&mut g);
    choose(&mut g, vec![]);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == ghost));
    assert!(g.sealed_cards.is_empty());
}

#[test]
fn next20_black_core_cq25_real_entry_contest_binds_curse_bonus_and_roundtrips() {
    let mut g = game();
    let source = held(&mut g, "CQ25", 0);
    fund(&mut g, 0, "CQ25", 4);
    for s in 0..4 {
        held(&mut g, "XG25", s);
    }
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    until_choice(&mut g);
    choose(&mut g, vec!["accept".into()]);
    for _ in 0..100 {
        if g.stack.is_empty() && g.pending.is_none() && g.mystic_contest.is_none() {
            break;
        }
        if let Some(p) = g.pending.clone() {
            let selected = match p.choice.kind.as_str() {
                crate::mystic_contest::COMMIT_KIND => vec![g.players[p.seat]
                    .hand
                    .iter()
                    .find(|c| c.definition == "XG25")
                    .unwrap()
                    .id
                    .clone()],
                crate::mystic_contest::REWARD_KIND | "trigger" => vec![],
                other => panic!("unexpected {other}"),
            };
            choose(&mut g, selected);
        } else {
            pass(&mut g);
        }
    }
    assert!(g.mystic_contest.is_none());
    let o = &g.mystic_history[0];
    assert_eq!(o.shown.iter().find(|s| s.seat == 0).unwrap().value, 4);
    for seat in 1..4 {
        assert_eq!(o.shown.iter().find(|s| s.seat == seat).unwrap().value, 2);
    }
    restore(&mut g);
}

#[test]
fn next20_black_core_sealed_move_restore_rejects_changed_fee_or_target_region() {
    let mut g = game();
    let host = field(&mut g, "JC125", 1);
    let card = g.make_card("XG25", 0);
    g.seal_card_on_valid_host(card, 0, &host);
    g.effects.clear();
    let source = g.sealed_cards[0].card.id.clone();
    at(&mut g, "JC125", 2, 1);
    fund(&mut g, 0, "XG25", 1);
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| a.action.card_id.as_ref() == Some(&source))
        .unwrap()
        .action;
    act(&mut g, 0, a);
    let original = serde_json::to_value(&g).unwrap();
    let mut bad = original.clone();
    bad["stack"][0]["frame"]["already_paid"] = serde_json::json!([]);
    assert!(Game::from_persisted(&bad.to_string()).is_err());
    let mut bad = original;
    bad["stack"][0]["frame"]["targets"][0]["public"]["region"] = 3.into();
    assert!(Game::from_persisted(&bad.to_string()).is_err());
}

#[test]
fn next20_black_core_conditional_spirit_follows_actual_controller_after_control_spell() {
    let mut g = game();
    let tiger = field(&mut g, "CQ25", 0);
    g.players[0].society_zone.secrecy = 1;
    assert!(g.current_spirit(g.board(&tiger).unwrap().1));
    let steal = held(&mut g, "JC129", 2);
    held(&mut g, "JC102", 2);
    fund(&mut g, 2, "JC102", catalog::card("JC129").cost as usize + 2);
    g.begin_window(Window::Action(g.team(2)));
    act(
        &mut g,
        2,
        Action {
            card_id: Some(steal),
            target_id: Some(tiger.clone()),
            ..Action::new("play")
        },
    );
    top(&mut g);
    assert_eq!(g.board(&tiger).unwrap().1.controller, 2);
    assert!(!g.current_spirit(g.board(&tiger).unwrap().1));
    fire(&mut g, &tiger);
    top(&mut g);
    assert!(g.board(&tiger).is_none());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "CQ25"));
}

#[test]
fn next20_black_core_sealed_move_survives_real_host_departure_with_original_region() {
    let mut g = game();
    let host = field(&mut g, "JC125", 1);
    let card = g.make_card("XG25", 0);
    g.seal_card_on_valid_host(card, 2, &host);
    g.effects.clear();
    let source = g.sealed_cards[0].card.id.clone();
    let enemy = at(&mut g, "JC125", 2, 1);
    fund(&mut g, 0, "XG25", 1);
    fund(&mut g, 2, "JC102", 2);
    held(&mut g, "JC102", 2);
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.card_id.as_ref() == Some(&source)
                && a.action.target_id.as_ref() == Some(&enemy)
        })
        .unwrap()
        .action;
    act(&mut g, 0, a);
    for _ in 0..4 {
        if g.legal_actions(2).iter().any(|a| a.action.kind == "play") {
            break;
        }
        pass(&mut g);
    }
    fire(&mut g, &host);
    top(&mut g);
    assert!(g.board(&host).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.definition == "XG25"));
    top(&mut g);
    assert_eq!(g.board(&enemy).unwrap().0, 0);
    restore(&mut g);
}

#[test]
fn next20_black_core_pregrave_helper_lost_secrecy_has_a_real_grave_fallback() {
    // Helper-only contract: does not claim a production death/response continuation.
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let target = field(&mut g, "JC125", 2);
    g.players[0].society_zone.secrecy = 1;
    let (r, c) = g.board(&ghost).unwrap();
    let source = g.next20_source(c, r);
    let (_, departed) = g.leave_board(&ghost).unwrap();
    g.players[0].society_zone.secrecy = 0;
    g.xg25_seal_instead_of_grave(departed, &source, &target)
        .unwrap();
    assert!(g.sealed_cards.is_empty());
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "XG25")
            .count(),
        1
    );
}

#[test]
fn next20_black_core_ineligible_death_observation_preserves_unrelated_id_stream() {
    // Non-interference contract for the new hook on every existing death wave.
    let mut g = game();
    let dead = field(&mut g, "JC125", 2);
    at(&mut g, "CQ25", 0, 1);
    let (r, c) = g.board(&dead).unwrap();
    let death = g.next20_source(c, r);
    for sources in [vec![], g.cq25_capture_sources()] {
        let before = serde_json::to_value(&g).unwrap();
        g.cq25_observe_death(&sources, &death);
        assert_eq!(serde_json::to_value(&g).unwrap(), before);
    }
}
