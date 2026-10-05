//! Finite MSJC11 tests; prepared board layouts below are disclosed synthetic.
use crate::msjc01_tests::{fund, pass_top, rejected};
use crate::{catalog, deck, model::*, rules::*};

fn game() -> Game {
    let mut g = Game::new_with_deck(
        "msjc11-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        draft(false),
        9,
    )
    .unwrap();
    for s in 1..4 {
        g.join_with_deck(format!("P{s}"), draft(s % 2 == 1))
            .unwrap();
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
        g.players[s].assets.clear();
        g.players[s].deck.clear();
        for _ in 0..20 {
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
fn board(g: &mut Game, definition: &str, owner: usize, controller: usize, region: usize) -> String {
    let mut c = g.make_card(definition, owner);
    c.controller = controller;
    let id = c.id.clone();
    g.regions[region].cards.push(c);
    id
}
fn activate(
    g: &Game,
    seat: usize,
    kill: bool,
    target: Option<String>,
    region: Option<usize>,
) -> Action {
    Action {
        card_id: Some(
            g.players[seat]
                .society_zone
                .card
                .as_ref()
                .unwrap()
                .id
                .clone(),
        ),
        ability_id: Some(
            if kill {
                "grant-kill"
            } else {
                "grant-region-retreat"
            }
            .into(),
        ),
        target_id: target,
        region,
        ..Action::new("activate")
    }
}
fn grant(g: &mut Game, seat: usize, kill: bool, target: Option<String>, region: Option<usize>) {
    fund(g, seat, 3);
    g.apply(seat, activate(g, seat, kill, target, region))
        .unwrap();
    pass_top(g);
}
fn restore(g: &mut Game) {
    let before = serde_json::to_string(g).unwrap();
    let views = (0..4)
        .map(|s| serde_json::to_value(g.view(s)).unwrap())
        .collect::<Vec<_>>();
    *g = Game::from_persisted(&before).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), before);
    assert_eq!(
        (0..4)
            .map(|s| serde_json::to_value(g.view(s)).unwrap())
            .collect::<Vec<_>>(),
        views
    );
}
fn ordinary(g: &mut Game, id: &str) {
    // Disclosed resolved prior ordinary-icon record, not a printed temporary icon.
    g.turn_attribute_modifiers.push(TurnAttributeModifier {
        target_instance: id.into(),
        defense_bonus: 0,
        kill_bonus: 0,
        grants_retreat: false,
        printed_defense_override: None,
        ordinary_icons: Icons {
            combat: 1,
            ..Default::default()
        },
        grants_renown: false,
        prevents_damage: false,
        expires_turn: g.turn,
    });
}

pub(crate) fn draft(red: bool) -> deck::DeckDraft {
    let mut d = deck::preset("responders").unwrap();
    d.society_id = Some("MSJC11".into());
    d.cards = [
        "JC014", "JC016", "JZ08", "BQ022", "JC020", "XQ07", "LC22", "LC23", "JC116", "JC129",
        "JC130", "JC131", "JC126",
    ]
    .into_iter()
    .chain(if red {
        vec!["JC042", "LC19", "XQ49"]
    } else {
        vec!["JC084", "JC086", "JC088"]
    })
    .map(|id| catalog::DeckEntry {
        card_id: id.into(),
        count: 3,
    })
    .collect();
    d.cards.push(catalog::DeckEntry {
        card_id: "JC125".into(),
        count: 2,
    });
    d
}

#[test]
fn msjc11_real_mixed_fifty_admission_and_original_fields() {
    let s = crate::society::definition("MSJC11").unwrap();
    assert_eq!(
        (&*s.card.name, &*s.subtitle, &*s.card.color),
        ("S.P.T.执行部", "直属特遣队", "绿")
    );
    assert!(s.card.unique && s.card.keywords.is_empty() && s.card.loyalty.is_empty());
    assert!(s.printed_cost.is_none() && s.card.defense.is_none());
    assert_eq!(s.starting_hand, 6);
    for red in [false, true] {
        assert_eq!(
            deck::validate(draft(red))
                .unwrap()
                .cards
                .iter()
                .map(|e| e.count)
                .sum::<usize>(),
            50
        );
    }
    let abilities = &definition("MSJC11").abilities;
    assert_eq!(abilities.len(), 2);
    assert!(abilities
        .iter()
        .all(|a| a.timing == Timing::Standard && a.costs.len() == 2));
}

#[test]
fn msjc11_construction_uses_printed_human_and_black_combat_and_old_limits() {
    for id in [
        "JC056", "JC058", "JC042", "JC084", "JC086", "JC088", "JZ59", "JZ61", "BQ022", "LC20",
    ] {
        let mut d = draft(false);
        d.cards = vec![
            catalog::DeckEntry {
                card_id: id.into(),
                count: 3,
            },
            catalog::DeckEntry {
                card_id: "JC125".into(),
                count: 47,
            },
        ];
        assert!(deck::validate(d).is_ok(), "{id}");
    }
    for id in ["JZ27", "JC104", "LC12", "JC093", "XQ43", "JC004"] {
        let mut d = draft(false);
        d.cards = vec![
            catalog::DeckEntry {
                card_id: id.into(),
                count: 3,
            },
            catalog::DeckEntry {
                card_id: "JC125".into(),
                count: 47,
            },
        ];
        assert!(
            deck::validate(d).unwrap_err().contains("构筑不允许"),
            "{id}"
        );
    }
    let mut d = draft(false);
    d.cards.last_mut().unwrap().count = 1;
    assert!(deck::validate(d).unwrap_err().contains("50"));
    let mut d = draft(false);
    d.cards[1].count = 4;
    d.cards.last_mut().unwrap().count = 1;
    assert!(deck::validate(d).unwrap_err().contains("最多3"));
}

#[test]
fn msjc11_programs_reject_other_cards_modes_nested_ops_costs_and_target_relations() {
    for a in &definition("MSJC11").abilities {
        assert!(validate_ability("MSJC11", a).is_ok());
        assert!(validate_ability("JC016", a).is_err());
        for flag in ["activation", "ready"] {
            let mut bad = a.clone();
            if flag == "activation" {
                bad.activation_only = true;
            } else {
                bad.requires_ready_source = true;
            }
            assert!(validate_ability("MSJC11", &bad).is_err());
        }
        let mut bad = a.clone();
        bad.costs = vec![];
        assert!(validate_ability("MSJC11", &bad).is_err());
        let mut bad = a.clone();
        bad.targets[0].relation = Relation::FriendlyTeam;
        assert!(validate_ability("MSJC11", &bad).is_err());
        let mut bad = a.clone();
        bad.ops = vec![Op::ForEachLivingPlayer(a.ops.clone())];
        assert!(validate_ability("MSJC11", &bad).is_err());
        let mut bad = a.clone();
        bad.ops.push(a.ops[0].clone());
        assert!(validate_ability("MSJC11", &bad).is_err());
    }
}

#[test]
fn msjc11_first_grant_any_controller_any_region_and_four_independent_sources() {
    for actor in 0..4 {
        let mut g = game();
        let team = g.team(actor);
        g.begin_window(Window::Action(team));
        let target = board(&mut g, "JC016", (actor + 2) % 4, (actor + 2) % 4, 4);
        grant(&mut g, actor, true, Some(target.clone()), None);
        restore(&mut g);
        assert_eq!(g.effective_kill(g.board(&target).unwrap().1), 2);
        for s in 0..4 {
            assert_eq!(
                g.players[s].society_zone.card.as_ref().unwrap().exhausted,
                s == actor
            );
            assert_eq!(
                g.players[s].assets.iter().filter(|c| c.exhausted).count(),
                if s == actor { 3 } else { 0 }
            );
        }
    }
}

#[test]
fn msjc11_rejected_declarations_do_not_pay_and_cancelled_bound_target_does_not_refund() {
    for bad in [
        "fee",
        "other-source",
        "barrier",
        "hidden",
        "attachment",
        "exhausted",
    ] {
        let mut g = game();
        fund(&mut g, 0, if bad == "fee" { 2 } else { 3 });
        fund(&mut g, 1, 3);
        let target = board(
            &mut g,
            if bad == "barrier" {
                "JZ08"
            } else if bad == "attachment" {
                "BQ022"
            } else {
                "JC016"
            },
            2,
            2,
            3,
        );
        if bad == "hidden" {
            g.board_mut(&target).unwrap().face_down = true;
        }
        if bad == "exhausted" {
            g.players[0].society_zone.card.as_mut().unwrap().exhausted = true;
        }
        let mut a = activate(&g, 0, true, Some(target), None);
        if bad == "other-source" {
            a.card_id = Some(g.players[1].society_zone.card.as_ref().unwrap().id.clone());
        }
        rejected(&mut g, 0, a);
    }
    let mut g = game();
    let target = board(&mut g, "JC016", 2, 2, 4);
    fund(&mut g, 0, 3);
    g.apply(0, activate(&g, 0, true, Some(target.clone()), None))
        .unwrap();
    g.return_hand(&target);
    pass_top(&mut g);
    assert!(g.turn_attribute_modifiers.is_empty());
    assert!(g.players[0].assets.iter().all(|c| c.exhausted));
    assert!(g.players[0].society_zone.card.as_ref().unwrap().exhausted);
}

#[test]
fn msjc11_kill_stacks_and_only_actual_ready_combat_participants_contribute() {
    let mut g = game();
    let target = board(&mut g, "JC016", 2, 2, 2);
    grant(&mut g, 0, true, Some(target.clone()), None);
    grant(&mut g, 1, true, Some(target.clone()), None);
    let zero = board(&mut g, "JC125", 2, 2, 2);
    g.grant_execution_traits(zero, 2, false);
    assert_eq!(g.effective_kill(g.board(&target).unwrap().1), 3);
    g.effects.clear();
    g.reward(1, 2, 1, 1).unwrap();
    assert!(matches!(
        g.effects.back(),
        Some(Effect::Damage { amount: 4, .. })
    ));
    g.effects.clear();
    g.board_mut(&target).unwrap().exhausted = true;
    board(&mut g, "LC22", 2, 2, 2);
    g.reward(1, 2, 1, 1).unwrap();
    assert!(matches!(
        g.effects.back(),
        Some(Effect::Damage { amount: 1, .. })
    ));
}

#[test]
fn msjc11_retreat_captures_controller_instances_once_not_team_owner_name_or_later_entrants() {
    let mut g = game();
    let own = board(&mut g, "LC22", 0, 0, 1);
    let borrowed = board(&mut g, "LC22", 2, 0, 1);
    let teammate = board(&mut g, "LC22", 1, 1, 1);
    let enemy = board(&mut g, "LC22", 0, 2, 1);
    let zero = board(&mut g, "LC20", 0, 0, 1);
    let hidden = board(&mut g, "JC016", 0, 0, 1);
    g.board_mut(&hidden).unwrap().face_down = true;
    grant(&mut g, 0, false, None, Some(1));
    restore(&mut g);
    for id in [&own, &borrowed] {
        assert!(g.has_retreat(g.board(id).unwrap().1));
    }
    for id in [&teammate, &enemy, &zero, &hidden] {
        assert!(!g.execution_turn_grants(g.board(id).unwrap().1).1);
    }
    let later = board(&mut g, "LC22", 0, 0, 1);
    ordinary(&mut g, &zero);
    assert!(!g.has_retreat(g.board(&later).unwrap().1));
    assert!(!g.has_retreat(g.board(&zero).unwrap().1));
    g.board_mut(&borrowed).unwrap().controller = 3;
    assert!(g.has_retreat(g.board(&borrowed).unwrap().1));
}

#[test]
fn msjc11_black_bottom_query_equipment_ordinary_initiative_and_converted_vector() {
    let mut g = game();
    let equipment_host = board(&mut g, "LC20", 0, 0, 0);
    let a = g.make_card("JC116", 0);
    g.attachments.push(Attachment {
        card: a,
        host_id: equipment_host.clone(),
    });
    let ordinary_host = board(&mut g, "LC20", 0, 0, 0);
    ordinary(&mut g, &ordinary_host);
    let white = board(&mut g, "JC004", 0, 0, 0);
    let spirit = board(&mut g, "JZ58", 0, 0, 0);
    let a = g.make_card("XQ43", 2);
    let region_id = g.regions[0].card.id.clone();
    g.attachments.push(Attachment {
        card: a,
        host_id: region_id,
    });
    for first in [0, 1] {
        g.first_team = first;
        assert_eq!(g.current_permanent_combat(g.board(&white).unwrap().1, 0), 0);
        assert_eq!(
            g.current_permanent_combat(g.board(&spirit).unwrap().1, 0),
            0
        );
        for id in [&equipment_host, &ordinary_host] {
            assert_eq!(g.current_permanent_combat(g.board(id).unwrap().1, 0), 1);
        }
    }
    // Current admitted pool has no confirmed spirit temporary-combat path.
    // This synthetic component vector proves the converted-T combat branch only.
    let p = Icons {
        combat: 2,
        ..Default::default()
    };
    let t = Icons {
        combat: 3,
        ..Default::default()
    };
    let o = Icons {
        combat: 1,
        ..Default::default()
    };
    assert_eq!(Game::permanent_combat_from_parts(p, t, o, false), 3);
    assert_eq!(Game::permanent_combat_from_parts(p, t, o, true), 6);
    grant(&mut g, 0, false, None, Some(0));
    for id in [&equipment_host, &ordinary_host] {
        assert!(g.has_retreat(g.board(id).unwrap().1));
    }
    assert!(!g.has_retreat(g.board(&white).unwrap().1));
}

#[test]
fn msjc11_region_instance_captured_required_and_replacement_cancelled_only_for_this_grant() {
    let mut g = game();
    let target = board(&mut g, "LC22", 0, 0, 0);
    fund(&mut g, 0, 3);
    g.apply(0, activate(&g, 0, false, None, Some(0))).unwrap();
    let frame = g.stack.last().unwrap().frame.as_ref().unwrap().clone();
    assert_eq!(
        frame.targets[0].region_instance.as_ref(),
        Some(&g.regions[0].card.id)
    );
    let mut missing = frame.targets[0].clone();
    missing.region_instance = None;
    assert!(!g.valid_bound_target(0, &frame.source, &missing));
    let old_source = SourceSnapshot {
        card: g.make_card("JC118", 0),
        region: None,
        attachment_host_instance: None,
        play_source: Some(PlaySource::Hand),
    };
    let old_bound = g
        .bind_action(
            0,
            &old_source,
            &definition("JC118").abilities[0],
            &Action {
                region: Some(0),
                ..Action::new("play")
            },
        )
        .unwrap();
    assert!(old_bound[0].region_instance.is_none());
    g.regions[0].card = g.make_card("DQJC115", 0);
    assert!(g.valid_bound_target(0, &old_source, &old_bound[0]));
    pass_top(&mut g);
    assert!(!g.has_retreat(g.board(&target).unwrap().1));
    assert!(g.players[0].assets.iter().all(|c| c.exhausted));
}

#[test]
fn msjc11_grant_survives_same_instance_move_and_lost_icons_but_not_hidden_or_reentry() {
    let mut g = game();
    let target = board(&mut g, "LC20", 0, 0, 0);
    ordinary(&mut g, &target);
    grant(&mut g, 0, false, None, Some(0));
    g.turn_attribute_modifiers
        .retain(|m| m.ordinary_icons == Icons::default());
    assert_eq!(
        g.current_permanent_combat(g.board(&target).unwrap().1, 0),
        0
    );
    assert!(g.has_retreat(g.board(&target).unwrap().1));
    let c = g.regions[0].cards.remove(0);
    g.regions[1].cards.push(c);
    g.prune_turn_attribute_modifiers();
    assert!(g.has_retreat(g.board(&target).unwrap().1));
    restore(&mut g);
    g.board_mut(&target).unwrap().face_down = true;
    g.prune_turn_attribute_modifiers();
    assert!(!g.has_retreat(g.board(&target).unwrap().1));
    g.board_mut(&target).unwrap().face_down = false;
    assert!(!g.has_retreat(g.board(&target).unwrap().1));
    g.return_hand(&target);
    let fresh = board(&mut g, "LC20", 0, 0, 1);
    assert_ne!(fresh, target);
    assert!(!g.has_retreat(g.board(&fresh).unwrap().1));
}

#[test]
fn msjc11_dynamic_projection_clears_for_all_hidden_views_and_end_turn() {
    let mut g = game();
    let target = board(&mut g, "LC22", 0, 0, 1);
    grant(&mut g, 0, true, Some(target.clone()), None);
    grant(&mut g, 1, false, None, Some(1));
    // Seat1's grant does not affect a seat0 card; add the resolved same-seat record
    // to isolate the shared cleanup/projection contract.
    g.grant_execution_traits(target.clone(), 0, true);
    restore(&mut g);
    for s in 0..4 {
        let v = g.view(s);
        let c = v.regions[1]
            .characters
            .iter()
            .find(|c| c.instance_id == target)
            .unwrap();
        assert_eq!(c.current_kill, Some(1));
        assert_eq!(c.current_retreat, Some(true));
    }
    let old_turn = g.turn;
    for _ in 0..300 {
        if g.turn > old_turn {
            break;
        }
        let s = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(s, Action::new("pass")).unwrap();
    }
    assert!(g.turn > old_turn);
    assert_eq!(
        g.execution_turn_grants(g.board(&target).unwrap().1),
        (0, false)
    );
    assert!(!g.players[0].society_zone.card.as_ref().unwrap().exhausted);
    g.grant_execution_traits(target.clone(), 1, true);
    g.board_mut(&target).unwrap().face_down = true;
    for s in 0..4 {
        let c = g.card_view(g.board(&target).unwrap().1, s, Some(1), None);
        let value = serde_json::to_value(c).unwrap();
        assert!(value.get("currentKill").is_none() && value.get("currentRetreat").is_none());
    }
}

#[test]
fn msjc11_real_win_window_returns_borrowed_instance_to_owner_and_preserves_attachment_batch() {
    let mut g = game();
    let target = board(&mut g, "LC22", 2, 0, 1);
    let other = board(&mut g, "LC22", 1, 1, 1);
    let a = g.make_card("BQ022", 3);
    let attachment = a.id.clone();
    g.attachments.push(Attachment {
        card: a,
        host_id: target.clone(),
    });
    grant(&mut g, 0, false, None, Some(1));
    let old_region = g.regions[1].card.id.clone();
    g.begin_window(Window::Win(1, 0));
    for _ in 0..100 {
        if g.regions[1].card.id != old_region {
            break;
        }
        if let Some(p) = g.pending.clone() {
            g.apply(
                p.seat,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(
                        p.choice
                            .options
                            .iter()
                            .rev()
                            .take(p.choice.min.unwrap_or(0))
                            .map(|o| o.id.clone())
                            .collect(),
                    ),
                    ..Action::new("choose")
                },
            )
            .unwrap();
        } else {
            let s = (0..4)
                .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
                .unwrap();
            g.apply(s, Action::new("pass")).unwrap();
        }
    }
    assert_ne!(g.regions[1].card.id, old_region);
    assert!(g.players[2]
        .hand
        .iter()
        .any(|c| c.definition == "LC22" && c.id != target));
    assert!(!g.players[0].hand.iter().any(|c| c.definition == "LC22"));
    assert!(g.players[1]
        .deck
        .iter()
        .any(|c| c.definition == "LC22" && c.id != other));
    assert!(g.players[3]
        .deck
        .iter()
        .any(|c| c.definition == "BQ022" && c.id != attachment));
    assert!(g.attachments.is_empty());
    assert!(g.turn_attribute_modifiers.is_empty());
}
