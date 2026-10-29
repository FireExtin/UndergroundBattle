//! Actual apply/choose/response paths for the finite XG25 pre-grave protocol.
use crate::{model::*, unit_support::*};
const SEAL: &str = "next20-xg25-seal";
fn grave_count(g: &Game) -> usize {
    g.players
        .iter()
        .flat_map(|p| &p.graveyard)
        .filter(|c| c.definition == "XG25")
        .count()
}
fn response_kills(g: &mut Game, count: usize) {
    fund(g, 2, "XG25", count * 2);
    for _ in 0..count {
        held(g, "BQ090", 2);
    }
    g.begin_window(Window::Action(g.team(2)));
}
fn kill(g: &mut Game, target: &str) {
    let card = g.players[2]
        .hand
        .iter()
        .find(|c| c.definition == "BQ090")
        .unwrap()
        .id
        .clone();
    let a = g
        .legal_actions(2)
        .into_iter()
        .find(|l| {
            l.action.kind == "play"
                && l.action.card_id.as_ref() == Some(&card)
                && l.action.target_id.as_deref() == Some(target)
        })
        .unwrap()
        .action;
    act(g, 2, a);
}
fn pending(g: &mut Game) {
    for _ in 0..60 {
        if g.pending.is_some() {
            return;
        }
        assert!(!g.stack.is_empty(), "death replacement was not declared");
        pass(g);
    }
    panic!("pending");
}
fn death_choice(g: &mut Game) {
    pending(g);
    assert!(
        matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key==SEAL),
        "must suspend original caller for the actual ghost death trigger"
    );
    assert_eq!(grave_count(g), 0);
    restore(g);
}
fn drain(g: &mut Game) {
    for _ in 0..80 {
        if g.stack.is_empty() && g.pending.is_none() {
            return;
        }
        if g.pending.is_some() {
            choose(g, vec![]);
        } else {
            pass(g);
        }
    }
    panic!("drain");
}
#[test]
fn next20_death_core_real_destroy_holds_card_then_seals_without_grave_visit() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let host = field(&mut g, "JC125", 3);
    g.players[0].society_zone.secrecy = 1;
    response_kills(&mut g, 1);
    kill(&mut g, &ghost);
    death_choice(&mut g);
    choose(&mut g, vec![host.clone()]);
    top(&mut g);
    assert_eq!(grave_count(&g), 0);
    assert_eq!(
        g.sealed_cards
            .iter()
            .filter(|s| s.card.definition == "XG25" && s.host_id == host)
            .count(),
        1
    );
    assert!(g.board(&ghost).is_none());
    restore(&mut g);
}
#[test]
fn next20_death_core_resolution_time_marker_loss_buries_once_and_finishes() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let host = field(&mut g, "JC125", 3);
    g.players[0].society_zone.secrecy = 1;
    response_kills(&mut g, 1);
    kill(&mut g, &ghost);
    death_choice(&mut g);
    choose(&mut g, vec![host]);
    // Explicit shared-rule response perturbation, not a claim that a newly
    // invented card action removes the moon. Exposure cancels secrecy normally.
    g.add_society_markers(0, 1, 0);
    restore(&mut g);
    top(&mut g);
    assert_eq!(grave_count(&g), 1);
    assert!(g.sealed_cards.is_empty());
    restore(&mut g);
}
#[test]
fn next20_death_core_target_destroyed_by_real_response_has_grave_fallback() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let host = field(&mut g, "JC125", 3);
    g.players[0].society_zone.secrecy = 1;
    response_kills(&mut g, 2);
    kill(&mut g, &ghost);
    death_choice(&mut g);
    choose(&mut g, vec![host.clone()]);
    kill(&mut g, &host);
    top(&mut g);
    assert!(g.board(&host).is_none());
    top(&mut g);
    assert_eq!(grave_count(&g), 1);
    assert!(g.sealed_cards.is_empty());
    restore(&mut g);
}
#[test]
fn next20_death_core_decline_and_no_target_both_finish_to_owner_grave() {
    for with_host in [true, false] {
        let mut g = game();
        let ghost = field(&mut g, "XG25", 0);
        if with_host {
            field(&mut g, "JC125", 3);
        }
        g.players[0].society_zone.secrecy = 1;
        response_kills(&mut g, 1);
        kill(&mut g, &ghost);
        if with_host {
            death_choice(&mut g);
            choose(&mut g, vec![]);
        } else {
            drain(&mut g);
        }
        assert_eq!(grave_count(&g), 1);
        assert!(g.sealed_cards.is_empty());
        restore(&mut g);
    }
}
#[test]
fn next20_death_core_actual_jz24_caller_waits_before_next_player_and_resumes_once() {
    for lose_moon in [false, true] {
        let mut g = game();
        let source = field(&mut g, "JZ24", 0);
        g.board_mut(&source).unwrap().face_down = true;
        let ghost = field(&mut g, "XG25", 2);
        let other = field(&mut g, "JC125", 3);
        g.players[2].society_zone.secrecy = 1;
        fund(&mut g, 0, "JC029", 4);
        act(
            &mut g,
            0,
            Action {
                card_id: Some(source.clone()),
                ..Action::new("reveal")
            },
        );
        pending(&mut g);
        choose(&mut g, vec!["accept".into()]);
        pending(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().seat, 2);
        choose(&mut g, vec![ghost]);
        death_choice(&mut g);
        assert!(g.board(&other).is_some());
        let live_source = g
            .regions
            .iter()
            .flat_map(|r| &r.cards)
            .find(|c| c.definition == "JZ24")
            .unwrap()
            .id
            .clone();
        choose(&mut g, vec![live_source]);
        if lose_moon {
            g.add_society_markers(2, 1, 0);
        }
        pending(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "jz24_sacrifice");
        assert_eq!(g.pending.as_ref().unwrap().seat, 3);
        assert_eq!(grave_count(&g), usize::from(lose_moon));
        choose(&mut g, vec![other]);
        assert!(g.pending.is_none());
        assert_eq!(
            g.sealed_cards
                .iter()
                .filter(|s| s.card.definition == "XG25")
                .count(),
            usize::from(!lose_moon)
        );
        restore(&mut g);
    }
}
#[test]
fn next20_death_core_nested_real_response_deaths_keep_distinct_continuations() {
    let mut g = game();
    let a = field(&mut g, "XG25", 0);
    let b = field(&mut g, "XG25", 1);
    let host = field(&mut g, "JC125", 3);
    g.players[0].society_zone.secrecy = 1;
    g.players[1].society_zone.secrecy = 1;
    response_kills(&mut g, 2);
    kill(&mut g, &a);
    death_choice(&mut g);
    choose(&mut g, vec![host.clone()]);
    kill(&mut g, &b);
    death_choice(&mut g);
    choose(&mut g, vec![host.clone()]);
    drain(&mut g);
    assert_eq!(grave_count(&g), 0);
    assert_eq!(
        g.sealed_cards
            .iter()
            .filter(|s| s.card.definition == "XG25" && s.host_id == host)
            .count(),
        2
    );
    restore(&mut g);
}
#[test]
fn next20_death_core_paid_black_transactions_reject_swapped_instance() {
    for id in ["CQ25", "XG25"] {
        for reveal in [false, true] {
            let mut g = game();
            fund(&mut g, 0, id, 4);
            let old = if reveal {
                let c = field(&mut g, id, 0);
                g.board_mut(&c).unwrap().face_down = true;
                c
            } else {
                held(&mut g, id, 0)
            };
            act(
                &mut g,
                0,
                Action {
                    card_id: Some(old.clone()),
                    region: (!reveal).then_some(0),
                    ..Action::new(if reveal { "reveal" } else { "deploy" })
                },
            );
            let mut bad = serde_json::to_value(&g).unwrap();
            bad["stack"][0]["card"]["id"] = old.into();
            assert!(Game::from_persisted(&bad.to_string()).is_err());
        }
    }
}

#[test]
fn next20_death_core_actual_damage_lethal_set_uses_the_same_pregrave_protocol() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let host = field(&mut g, "XG25", 2);
    // Equal domain counts remove Spirit's damage prevention; destruction tests
    // do not stand in for the actual lethal-damage entrance.
    assert!(!g.spirit_protected(g.board(&ghost).unwrap().1));
    g.players[0].society_zone.secrecy = 1;
    fund(&mut g, 2, "JC102", 2);
    let fire = held(&mut g, "JC102", 2);
    g.begin_window(Window::Action(g.team(2)));
    act(
        &mut g,
        2,
        Action {
            card_id: Some(fire),
            target_id: Some(ghost),
            ..Action::new("play")
        },
    );
    death_choice(&mut g);
    choose(&mut g, vec![host]);
    top(&mut g);
    assert_eq!(grave_count(&g), 0);
    assert_eq!(g.sealed_cards.len(), 1);
    restore(&mut g);
}
#[test]
fn next20_death_core_actual_sacrifice_cost_finishes_destination_before_paid_original_effect() {
    for seal in [false, true] {
        let mut g = game();
        let dog = field(&mut g, "JC096", 0);
        let ghost = field(&mut g, "XG25", 0);
        let host = field(&mut g, "JC125", 3);
        g.players[0].society_zone.secrecy = 1;
        fund(&mut g, 0, "JC125", 1);
        act(
            &mut g,
            0,
            Action {
                card_id: Some(dog),
                ability_id: Some("sacrifice-draw".into()),
                cost_selected: Some(vec![ghost]),
                ..Action::new("activate")
            },
        );
        death_choice(&mut g);
        choose(&mut g, if seal { vec![host] } else { vec![] });
        if seal {
            top(&mut g);
        }
        assert!(
            g.death_replacements.is_empty(),
            "a paid original frame is the caller, not an unresolved death response"
        );
        assert_eq!(grave_count(&g), usize::from(!seal));
        assert_eq!(g.players[0].hand.len(), 0);
        assert_eq!(g.stack.len(), 1);
        top(&mut g);
        assert_eq!(g.players[0].hand.len(), 1);
        assert_eq!(g.resources(0), 0);
        restore(&mut g);
    }
}
#[test]
fn next20_death_core_actual_death_then_sealed_action_uses_owner_not_old_or_host_control() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 2);
    g.add_control(
        &ghost,
        0,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    let host = field(&mut g, "JC125", 3);
    let enemy = g.make_card("JC125", 0);
    let enemy_id = enemy.id.clone();
    g.regions[1].cards.push(enemy);
    g.players[0].society_zone.secrecy = 1;
    response_kills(&mut g, 1);
    fund(&mut g, 2, "XG25", 1);
    kill(&mut g, &ghost);
    death_choice(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    choose(&mut g, vec![host]);
    top(&mut g);
    let sealed = g
        .sealed_cards
        .iter()
        .find(|s| s.card.definition == "XG25")
        .unwrap()
        .card
        .id
        .clone();
    for seat in [0, 3] {
        assert!(!g
            .legal_actions(seat)
            .iter()
            .any(|a| a.action.card_id.as_ref() == Some(&sealed)));
    }
    let a = g
        .legal_actions(2)
        .into_iter()
        .find(|a| {
            a.action.card_id.as_ref() == Some(&sealed)
                && a.action.target_id.as_ref() == Some(&enemy_id)
        })
        .unwrap()
        .action;
    act(&mut g, 2, a.clone());
    assert_eq!(g.resources(2), 0);
    reject(&mut g, 2, a);
    top(&mut g);
    assert_eq!(g.board(&enemy_id).unwrap().0, 0);
    restore(&mut g);
}
fn jz24_held_layout() -> Game {
    let mut g = game();
    let source = field(&mut g, "JZ24", 0);
    g.board_mut(&source).unwrap().face_down = true;
    let ghost = field(&mut g, "XG25", 2);
    field(&mut g, "JC125", 3);
    g.players[2].society_zone.secrecy = 1;
    fund(&mut g, 0, "JC029", 4);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    );
    pending(&mut g);
    choose(&mut g, vec!["accept".into()]);
    pending(&mut g);
    choose(&mut g, vec![ghost]);
    death_choice(&mut g);
    g
}
#[test]
fn next20_death_core_restore_rejects_duplicate_held_card_and_changed_source_or_barrier() {
    let mut g = jz24_held_layout();
    let saved = serde_json::to_value(&g).unwrap();
    for key in [
        "owner",
        "card",
        "controller",
        "region",
        "duplicate",
        "missing_finish",
        "duplicate_finish",
        "choice_id",
    ] {
        let mut bad = saved.clone();
        match key {
            "owner" => bad["death_replacements"][0]["card"]["owner"] = 1.into(),
            "choice_id" => bad["death_replacements"][0]["declaration_choice"] = "i1".into(),
            "card" => bad["death_replacements"][0]["card"] = serde_json::Value::Null,
            "controller" => bad["death_replacements"][0]["source"]["card"]["controller"] = 0.into(),
            "region" => bad["death_replacements"][0]["source"]["region"] = 1.into(),
            "duplicate" => {
                let c = bad["death_replacements"][0]["card"].clone();
                bad["players"][2]["graveyard"]
                    .as_array_mut()
                    .unwrap()
                    .push(c);
            }
            _ => {
                let es = bad["effects"].as_array_mut().unwrap();
                let i = es
                    .iter()
                    .position(|e| e.get("DeathFinish").is_some())
                    .unwrap();
                if key == "missing_finish" {
                    es.remove(i);
                } else {
                    es.push(es[i].clone());
                }
            }
        }
        assert!(Game::from_persisted(&bad.to_string()).is_err(), "{key}");
    }
    restore(&mut g);
}
#[test]
fn next20_death_core_restore_rejects_early_or_replayed_original_caller() {
    let g = jz24_held_layout();
    let saved = serde_json::to_value(&g).unwrap();
    for key in ["early", "repeat", "cursor", "cohort", "program"] {
        let mut bad = saved.clone();
        let es = bad["effects"].as_array_mut().unwrap();
        let i = es
            .iter()
            .position(|e| e.get("DeathCaller").is_some())
            .unwrap();
        match key {
            "early" => {
                let e = es.remove(i);
                es.insert(0, e);
            }
            "repeat" => es.push(es[i].clone()),
            "cursor" => es[i]["DeathCaller"]["caller"]["frame"]["cursor"] = 0.into(),
            "cohort" => {
                es[i]["DeathCaller"]["caller"]["kind"]["JZ24"]["remaining_players"] =
                    serde_json::json!([0])
            }
            _ => es[i]["DeathCaller"]["caller"]["frame"]["steps"] = serde_json::json!([]),
        }
        assert!(Game::from_persisted(&bad.to_string()).is_err(), "{key}");
    }
}
#[test]
fn next20_death_core_jz35_does_not_invent_a_host_type_for_ghosts() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    field(&mut g, "WM035", 0);
    assert!(!g
        .current_subtypes(g.board(&ghost).unwrap().1)
        .iter()
        .any(|t| t == "宿主"));
    let returns = g.jz35_sacrifice_hosts(0);
    assert!(returns.iter().all(|r| r.definition != "XG25"));
    assert!(g.board(&ghost).is_some());
}

#[test]
fn next20_death_core_already_stacked_tiger_can_find_declined_or_cancelled_ghost() {
    for lose_target in [false, true] {
        let mut g = game();
        let tiger = field(&mut g, "CQ25", 0);
        let ghost = field(&mut g, "XG25", 0);
        g.add_control(
            &ghost,
            2,
            ControlLifetime::TurnEnd { turn: g.turn },
            SubtypeChange::None,
        );
        g.players[2].society_zone.secrecy = 1;
        response_kills(&mut g, 1);
        let response = if lose_target {
            fund(&mut g, 0, "JC102", 2);
            Some(held(&mut g, "JC102", 0))
        } else {
            None
        };
        kill(&mut g, &ghost);
        pending(&mut g);
        assert!(
            matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key==crate::next20_black_wraiths::SEARCH)
        );
        choose(&mut g, vec!["accept".into()]);
        death_choice(&mut g);
        if lose_target {
            choose(&mut g, vec![tiger.clone()]);
            for _ in 0..8 {
                if g.legal_actions(0)
                    .iter()
                    .any(|a| a.action.kind == "play" && a.action.card_id == response)
                {
                    break;
                }
                pass(&mut g);
            }
            let response_action = g
                .legal_actions(0)
                .into_iter()
                .find(|a| {
                    a.action.kind == "play"
                        && a.action.card_id == response
                        && a.action.target_id.as_ref() == Some(&tiger)
                })
                .expect("the real response must have priority and a legal target")
                .action;
            act(&mut g, 0, response_action);
            top(&mut g);
            assert!(g.board(&tiger).is_none());
            top(&mut g);
        } else {
            choose(&mut g, vec![]);
        }
        assert_eq!(
            grave_count(&g),
            1,
            "fallback precedes resolution of the earlier sibling search frame"
        );
        pending(&mut g);
        let found = g.players[0]
            .graveyard
            .iter()
            .find(|c| c.definition == "XG25")
            .unwrap()
            .id
            .clone();
        assert!(g
            .pending
            .as_ref()
            .unwrap()
            .choice
            .options
            .iter()
            .any(|o| o.id == found));
        choose(&mut g, vec![found]);
        assert_eq!(grave_count(&g), usize::from(lose_target));
        assert_eq!(
            g.sealed_cards
                .iter()
                .filter(|s| s.card.definition == "XG25")
                .count(),
            usize::from(!lose_target)
        );
        assert!(g.death_replacements.is_empty());
        restore(&mut g);
    }
}

/// MSBQ01 prerequisite: a real non-death response must declare its observers
/// before an older unresolved departure barrier, using the ordinary interpreter.
#[test]
fn next20_death_core_nondeath_response_observer_precedes_held_destination() {
    let mut g = game();
    let ghost = field(&mut g, "XG25", 0);
    let host = field(&mut g, "JC125", 3);
    field(&mut g, "CQ18", 0);
    let returned = field(&mut g, "JC029", 3);
    g.board_mut(&returned).unwrap().lock_markers = 1;
    g.players[0].society_zone.secrecy = 1;
    response_kills(&mut g, 1);
    fund(&mut g, 2, "JC006", 2);
    let bounce = held(&mut g, "JC006", 2);
    kill(&mut g, &ghost);
    death_choice(&mut g);
    choose(&mut g, vec![host.clone()]);
    act(&mut g, 2, Action {card_id:Some(bounce),target_id:Some(returned.clone()),..Action::new("play")});
    for _ in 0..20 {
        if g.board(&returned).is_none() { break; }
        assert!(g.pending.is_none());
        pass(&mut g);
    }
    assert!(g.board(&returned).is_none());
    assert!(matches!(&g.pending.as_ref().map(|p|&p.resolution),Some(ChoiceResolution::Declare{declaration:d,..})if d.ability.key==crate::import20_sentinel::LEAVE),"non-death observer must not wait behind the older destination: {:?}",g.effects);
    assert_eq!(g.death_replacements.len(),1);
    assert!(g.death_replacements[0].destination.is_none());
    assert_eq!(grave_count(&g),0);
    restore(&mut g);
    let mut room=crate::jc029_tests::envelope(&g);
    room=crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).unwrap();
    g=room.game;
    let hand=g.players[0].hand.len();
    choose(&mut g,vec!["accept".into()]);
    top(&mut g);
    assert_eq!(g.players[0].hand.len(),hand+1);
    assert!(g.death_replacements[0].destination.is_none());
    drain(&mut g);
    assert!(g.death_replacements.is_empty());
    assert_eq!(g.sealed_cards.iter().filter(|s|s.card.definition=="XG25"&&s.host_id==host).count(),1);
    restore(&mut g);
}
