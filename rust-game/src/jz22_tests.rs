//! Explicit prepared native layouts. All choices/payments/responses use real
//! commands; fixture exports are for WASM/UI regression, not natural gameplay.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, room::RoomEnvelope, rules::*};
fn initial(actor: usize) -> Game {
    let mut g = game(actor);
    for r in &mut g.regions {
        r.influence = [0; 2];
    }
    g
}
fn held(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn hands(g: &mut Game, counts: [usize; 4]) {
    for (s, n) in counts.into_iter().enumerate() {
        g.players[s].hand.clear();
        for _ in 0..n {
            held(g, "JC059", s);
        }
    }
}
fn enter(g: &mut Game, owner: usize, controller: usize) -> String {
    // Explicit entry event preparation; ordinary paid deployment tested below.
    let id = board(g, "JZ22", owner, 2);
    g.board_mut(&id).unwrap().controller = controller;
    g.enter_triggers(controller, "JZ22", &id, false);
    g.drive().unwrap();
    checkpoint(g);
    id
}
fn accept(g: &mut Game) {
    choose(g, vec!["accept".into()]);
}
fn fixture(kind: &str, g: &Game) {
    if let Ok(dir) = std::env::var("JZ22_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{kind}.json"),serde_json::to_vec(&serde_json::json!({"kind":kind,"state":serde_json::to_string(&r).unwrap(),"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()})).unwrap()).unwrap();
    }
}
#[test]
fn jz22_original_fields_and_closed_program() {
    let c = catalog::card("JZ22");
    assert_eq!(
        (&*c.name, c.cost, &*c.magic, c.defense),
        ("暗夜游掠者", 2, "鲜血", Some(1))
    );
    assert_eq!(c.loyalty, ["蓝色"]);
    assert_eq!(c.subtypes, ["吸血鬼"]);
    assert!(!c.unique && c.keywords.is_empty());
    assert_eq!(
        c.permanent_icons,
        Icons {
            combat: 1,
            ..Default::default()
        }
    );
    assert_eq!(
        c.temporary_icons,
        Icons {
            combat: 1,
            ..Default::default()
        }
    );
    let a = &definition("JZ22").abilities[0];
    assert_eq!(a.event, Some(Event::Enter));
    assert_eq!(a.timing, Timing::Fast);
    assert_eq!(a.response_policy, ResponsePolicy::Respondable);
    assert!(
        a.costs.is_empty()
            && a.targets.is_empty()
            && a.modes.is_empty()
            && a.per_turn_limit.is_none()
    );
    assert!(matches!(
        a.ops.as_slice(),
        [Op::JZ22LowHandInfluenceInSourceRegion]
    ));
}
#[test]
fn jz22_paid_deploy_and_paid_reveal_emit_once_secret_entry_and_move_do_not() {
    for hidden in [false, true] {
        let mut g = initial(0);
        fund(&mut g, 0, "JZ22", 2);
        let id = held(&mut g, "JZ22", 0);
        apply(
            &mut g,
            0,
            Action {
                card_id: Some(id),
                region: Some(2),
                ..Action::new(if hidden { "conceal" } else { "deploy" })
            },
        );
        if !hidden {
            pass_top(&mut g);
        }
        if hidden {
            assert!(g.pending.is_none());
            let id = g.regions[2].cards[0].id.clone();
            for c in &mut g.players[0].assets {
                c.exhausted = false;
            }
            apply(
                &mut g,
                0,
                Action {
                    card_id: Some(id),
                    ..Action::new("reveal")
                },
            );
            pass_top(&mut g);
        }
        assert!(g.pending.is_some());
        fixture(
            if hidden {
                "reveal-choice"
            } else {
                "deploy-choice"
            },
            &g,
        );
        accept(&mut g);
        pass_top(&mut g);
        assert!(g.pending.is_none() && g.stack.is_empty());
        assert_eq!(g.regions[2].influence, [1, 0]);
        let id = g.regions[2].cards[0].id.clone();
        g.regions[2].cards[0].exhausted = false;
        let (_, c) = g.remove_board(&id).unwrap();
        g.regions[1].cards.push(c);
        g.drive().unwrap();
        checkpoint(&g);
        assert!(g.pending.is_none());
        assert_eq!(g.regions[2].influence, [1, 0]);
    }
}
#[test]
fn jz22_cost_magic_and_loyalty_fail_atomically() {
    for assets in [vec!["JC059", "JC059"], vec!["JZ31", "JZ31"], vec!["JZ22"]] {
        let mut g = initial(0);
        for a in assets {
            fund(&mut g, 0, a, 1);
        }
        let id = held(&mut g, "JZ22", 0);
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(id),
                region: Some(2),
                ..Action::new("deploy")
            },
        );
        assert!(g.regions[2].cards.is_empty());
    }
}
#[test]
fn jz22_team_enemy_thresholds_and_only_one_point_for_multiple_enemies() {
    for (counts, want) in [
        ([0, 0, 4, 4], 0),
        ([4, 4, 3, 4], 1),
        ([4, 4, 4, 3], 1),
        ([4, 4, 0, 0], 1),
    ] {
        let mut g = initial(0);
        hands(&mut g, counts);
        enter(&mut g, 0, 0);
        fixture("low-hand-choice", &g);
        for s in 0..4 {
            assert_eq!(g.view(s).pending_choice.is_some(), s == 0);
        }
        accept(&mut g);
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [want, 0]);
    }
    let mut g = initial(1);
    hands(&mut g, [4, 4, 3, 4]);
    enter(&mut g, 1, 1);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [1, 0]); // teammate actor shares influence
    let mut g = initial(0);
    hands(&mut g, [0, 0, 0, 4]);
    g.players[2].eliminated = true;
    enter(&mut g, 0, 0);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [0, 0]);
}
#[test]
fn jz22_condition_is_live_after_responses_both_threshold_directions() {
    for (before, after, want) in [(3, 4, 0), (4, 3, 1)] {
        let mut g = initial(0);
        hands(&mut g, [4, 4, before, 4]);
        enter(&mut g, 0, 0);
        accept(&mut g);
        // Explicit changed public hand count during a response window.
        if after > before {
            held(&mut g, "JC059", 2);
        } else {
            g.players[2].hand.pop();
        }
        checkpoint(&g);
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [want, 0]);
    }
}
#[test]
fn jz22_borrowed_source_freezes_controller_and_region_before_declaring_or_responses() {
    for after_accept in [false, true] {
        for mutation in 0..4 {
            let mut g = initial(3);
            hands(&mut g, [2, 4, 4, 4]);
            let id = enter(&mut g, 0, 3);
            if after_accept {
                accept(&mut g);
            }
            match mutation {
                0 => {
                    let c = g.regions[2].cards.remove(0);
                    g.regions[1].cards.push(c);
                }
                1 => g.return_hand(&id),
                2 => g.board_mut(&id).unwrap().controller = 1,
                _ => {
                    g.remove_dead(&id, RemovalCause::Destroy);
                }
            }
            checkpoint(&g);
            if !after_accept {
                assert_eq!(g.pending.as_ref().unwrap().seat, 3);
                accept(&mut g);
            }
            pass_top(&mut g);
            assert_eq!(g.regions[2].influence, [0, 1]);
            assert_eq!(g.regions[1].influence, [0, 0]);
            if mutation == 1 {
                assert!(g.players[0].hand.iter().any(|c| c.definition == "JZ22"));
            }
            if mutation == 3 {
                assert!(g.players[0]
                    .graveyard
                    .iter()
                    .any(|c| c.definition == "JZ22"));
            }
        }
    }
}
#[test]
fn jz22_region_replacement_before_and_after_accept_invalidates_only_original_slot() {
    for after_accept in [false, true] {
        let mut g = initial(0);
        enter(&mut g, 0, 0);
        if after_accept {
            accept(&mut g);
        }
        g.regions[2].card = g.make_card("DQJC107", 0);
        checkpoint(&g);
        if !after_accept {
            accept(&mut g);
        }
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [0, 0]);
    }
}
#[test]
fn jz22_decline_restore_wrong_actor_and_duplicate_choice() {
    let mut g = initial(0);
    enter(&mut g, 0, 0);
    let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    let a = Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(vec!["accept".into()]),
        ..Action::new("choose")
    };
    reject(&mut g, 2, a.clone());
    apply(&mut g, 0, a.clone());
    apply(&mut restored, 0, a.clone());
    reject(&mut g, 0, a);
    assert_eq!(
        serde_json::to_value(&g).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    fixture("accepted-stack", &g);
    let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    pass_top(&mut g);
    pass_top(&mut restored);
    assert_eq!(
        serde_json::to_value(&g).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    assert_eq!(g.regions[2].influence, [1, 0]);
    fixture("resolved", &g);
    let mut g = initial(0);
    enter(&mut g, 0, 0);
    choose(&mut g, vec![]);
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.regions[2].influence, [0, 0]);
}
#[test]
fn jz22_two_independent_same_color_instances_and_one_point_threshold() {
    let mut g = initial(0);
    let first = enter(&mut g, 0, 0);
    accept(&mut g);
    pass_top(&mut g);
    let second = enter(&mut g, 0, 0);
    assert_ne!(first, second);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [2, 0]);
    let mut g = initial(0);
    g.regions[2].card = g.make_card("DQJC115", 0);
    g.regions[2].influence = [2, 0];
    enter(&mut g, 0, 0);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [3, 0]);
    assert_eq!(g.window, Some(Window::Win(2, 0)));
    fixture("win-threshold", &g);
    let mut g = initial(0);
    g.regions[2].influence = [0, 1];
    enter(&mut g, 0, 0);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [0, 0]);
}
#[test]
fn jz22_definition_transplant_and_nested_program_rejected() {
    let a = definition("JZ22").abilities[0].clone();
    assert!(crate::rules::validate_ability("JC059", &a).is_err());
    for n in 0..7 {
        let mut a = a.clone();
        match n {
            0 => a.event = Some(Event::Death),
            1 => a.ops.push(Op::Draw {
                player: PlayerRef::Actor,
                count: 1,
                end: DeckEnd::Top,
            }),
            2 => a.ops = vec![Op::ForEachLivingPlayer(a.ops.clone())],
            3 => a.costs = vec![Cost::Assets(1)],
            4 => a.response_policy = ResponsePolicy::Immediate,
            5 => a.key = "wrong".into(),
            _ => {
                a.modes = vec![Mode {
                    key: "wrong".into(),
                    label: "wrong".into(),
                    targets: vec![],
                    ops: a.ops.clone(),
                }]
            }
        };
        assert!(crate::rules::validate_ability("JZ22", &a).is_err());
        assert!(crate::rules::validate_ability("JC059", &a).is_err());
    }
    let mut definitions = crate::rules::definitions().clone();
    definitions.get_mut("JZ22").unwrap().abilities.clear();
    assert!(validate_definitions(&definitions).is_err());
}
#[test]
fn jz22_persisted_declaration_and_frame_tampering_rejected() {
    for stage in [false, true] {
        let mut g = initial(0);
        enter(&mut g, 0, 0);
        if stage {
            accept(&mut g);
        }
        let original = serde_json::to_value(&g).unwrap();
        let room = serde_json::to_value(envelope(&g)).unwrap();
        for n in 0..9 {
            let mut v = original.clone();
            let f = if stage {
                &mut v["stack"][0]["frame"]
            } else {
                &mut v["pending"]["resolution"]["Declare"]["declaration"]
            };
            match n {
                0 => f["actor"] = serde_json::json!(2),
                1 => f["source"]["card"]["owner"] = serde_json::json!(9),
                2 => f["source"]["card"]["face_down"] = serde_json::json!(true),
                3 => {
                    f["source"]
                        .as_object_mut()
                        .unwrap()
                        .remove("source_region_instance");
                }
                4 => f["source"]["card"]["definition"] = serde_json::json!("JC059"),
                5 => {
                    if stage {
                        f["cursor"] = serde_json::json!(1);
                    } else {
                        f["ability"]["event"] = serde_json::json!("Death");
                    }
                }
                6 => {
                    if stage {
                        f["guard"] = serde_json::json!("Accepted");
                    } else {
                        f["ability"]["ops"] =
                            serde_json::json!(["PlaceOneInfluenceInSourceRegion"]);
                    }
                }
                7 => f["source"]["region"] = serde_json::json!(99),
                _ => f["source"]["card"]["controller"] = serde_json::json!(3),
            }
            assert!(
                Game::from_persisted(&v.to_string()).is_err(),
                "stage={stage} mutation={n}"
            );
            let mut r = room.clone();
            r["game"] = v;
            assert!(RoomEnvelope::from_persisted(&r.to_string()).is_err());
        }
    }
}

#[test]
fn jz22_real_paid_enemy_responses_cross_three_card_threshold() {
    for draw in [false, true] {
        let mut g = initial(0);
        hands(&mut g, [4, 4, 3, 4]);
        let source = enter(&mut g, 0, 0);
        accept(&mut g);
        let (id, kind) = if draw {
            let id = board(&mut g, "JC076", 2, 1);
            g.board_mut(&id).unwrap().face_down = true;
            fund(&mut g, 2, "JC076", 3);
            (id, "reveal")
        } else {
            fund(&mut g, 2, "JC008", 2);
            (held(&mut g, "JC008", 2), "play")
        };
        while g.priority_team != g.team(2) {
            let s = (0..4)
                .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
                .unwrap();
            apply(&mut g, s, Action::new("pass"));
        }
        apply(
            &mut g,
            2,
            Action {
                card_id: Some(id),
                target_id: if draw { None } else { Some(source) },
                ..Action::new(kind)
            },
        );
        pass_top(&mut g);
        if draw {
            assert_eq!(g.pending.as_ref().unwrap().seat, 2);
            accept(&mut g);
            pass_top(&mut g);
            assert_eq!(g.players[2].hand.len(), 4);
        } else {
            assert_eq!(g.players[2].hand.len(), 3);
        }
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [u32::from(!draw), 0]);
    }
}
#[test]
fn jz22_preserves_existing_granted_renown_program() {
    let mut g = initial(0);
    let source = board(&mut g, "JZ22", 0, 2);
    fund(&mut g, 0, "JC074", 2);
    let spell = held(&mut g, "JC074", 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(spell),
            target_id: Some(source),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    g.begin_window(Window::After(2, 2));
    let region = g.regions[2].card.id.clone();
    g.declare_region_renown(2, &region);
    g.drive().unwrap();
    checkpoint(&g);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [1, 0]);
}
