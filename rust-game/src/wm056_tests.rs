//! Actual paid commands and immutable Native outputs from explicit offline layouts.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, rules::*};
const KEY: &str = "wm056-adjacent-influence";
fn activate(id: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(KEY.into()),
        ..Action::new("activate")
    }
}
fn setup(actor: usize) -> (Game, String) {
    let mut g = game(actor);
    fund(&mut g, actor, "JC093", 3);
    let c = g.make_card("WM056", actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    apply(
        &mut g,
        actor,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    let id = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "WM056")
        .unwrap()
        .id
        .clone();
    (g, id)
}
fn snap(name: &str, g: &Game, a: &Action) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("WM056_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{name}.json"),serde_json::to_vec(&serde_json::json!({"state":serde_json::to_string(&r).unwrap(),"action":a,"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>(),"scope":"offline-current-native-wm056"})).unwrap()).unwrap();
    }
}
fn pick(g: &Game, region: usize) -> Action {
    let p = g.pending.as_ref().unwrap();
    Action {
        choice_id: Some(p.choice.id.clone()),
        selected: Some(vec![format!("region:{region}")]),
        ..Action::new("choose")
    }
}
fn invalid(g: &Game, bad: Game, name: &str) {
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut r = envelope(g);
    r.game = bad;
    let state = serde_json::to_string(&r).unwrap();
    assert!(crate::room::RoomEnvelope::from_persisted(&state).is_err());
    if let Ok(dir) = std::env::var("WM056_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/{name}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}
#[test]
fn wm056_complete_original_and_closed_standard_non_targeted_once_action() {
    let c = catalog::card("WM056");
    assert_eq!(
        (&*c.name, &*c.color, c.cost, c.defense),
        ("赃物贩子", "黑", 1, Some(1))
    );
    assert_eq!(c.loyalty, vec!["黑色"]);
    assert_eq!(c.subtypes, vec!["人类", "罪犯", "商人"]);
    assert!(c.magic.is_empty() && !c.unique && c.keywords.is_empty());
    assert_eq!(
        c.permanent_icons,
        Icons {
            investigation: 0,
            combat: 0,
            influence: 1
        }
    );
    assert_eq!(c.temporary_icons, Icons::default());
    let d = definition("WM056");
    assert!(d.abilities[0].targets.is_empty());
    assert_eq!(d.abilities[0].per_turn_limit, Some(1));
    for n in 0..7 {
        let mut bad = d.clone();
        let a = &mut bad.abilities[0];
        match n {
            0 => a.costs = vec![Cost::Assets(1)],
            1 => a.timing = Timing::Fast,
            2 => a.per_turn_limit = None,
            3 => {
                a.targets = vec![crate::rules::target(
                    Zone::Region,
                    EntityKind::Any,
                    Relation::Any,
                    Range::Mobility,
                )]
            }
            4 => a.response_policy = ResponsePolicy::Immediate,
            5 => a.activation_only = false,
            _ => a.event = Some(Event::Enter),
        };
        let mut ds = definitions().clone();
        ds.insert("WM056".into(), bad);
        assert!(crate::rules::validate_definitions(&ds).is_err());
    }
    let mut ds = definitions().clone();
    ds.insert("JC125".into(), d.clone());
    assert!(crate::rules::validate_definitions(&ds).is_err());
    crate::rules::validate_definitions(definitions()).unwrap();
}
#[test]
fn wm056_real_paid_activation_all_seats_choice_projection_restart_and_exact_marker_move() {
    for actor in 0..4 {
        for origin in [1, 3] {
            let (mut g, id) = setup(actor);
            let team = g.team(actor);
            g.regions[1].influence[team] = 2;
            g.regions[3].influence[team] = 1;
            g.regions[0].influence[team] = 1;
            if actor == 0 && origin == 1 {
                snap("activation", &g, &activate(&id));
            }
            apply(&mut g, actor, activate(&id));
            assert_eq!(g.resources(actor), 0);
            assert!(g.pending.is_none());
            pass_top(&mut g);
            let p = g.pending.as_ref().unwrap();
            assert_eq!(p.seat, actor);
            assert_eq!(p.choice.min, Some(1));
            assert_eq!(
                p.choice
                    .options
                    .iter()
                    .map(|o| o.id.as_str())
                    .collect::<Vec<_>>(),
                vec!["region:1", "region:3"]
            );
            if actor == 0 {
                snap(
                    if origin == 1 {
                        "origin-left"
                    } else {
                        "origin-right"
                    },
                    &g,
                    &pick(&g, origin),
                );
            }
            choose(&mut g, vec![format!("region:{origin}")]);
            assert_eq!(g.regions[2].influence[team], 1);
            assert_eq!(
                g.regions[origin].influence[team],
                if origin == 1 { 1 } else { 0 }
            );
            checkpoint(&g);
        }
    }
}
#[test]
fn wm056_actual_duel_adjacency_all_other_live_regions_and_board_edge_no_wrap_in_teams() {
    let (mut g, id) = setup(0);
    g.mode = "duel".into();
    g.players.truncate(2);
    g.regions[0].influence[0] = 1;
    g.regions[4].influence[0] = 1;
    g.apply(0, activate(&id)).unwrap();
    while g.pending.is_none() && !g.stack.is_empty() {
        let seat = (0..2)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(seat, Action::new("pass")).unwrap();
    }
    assert_eq!(
        g.pending
            .as_ref()
            .unwrap()
            .choice
            .options
            .iter()
            .map(|o| o.id.clone())
            .collect::<Vec<_>>(),
        vec!["region:0", "region:4"]
    );
    let r = envelope(&g);
    for seat in 0..2 {
        assert_eq!(
            serde_json::to_value(r.view(seat, 0)).unwrap(),
            serde_json::to_value(
                crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap())
                    .unwrap()
                    .view(seat, 0)
            )
            .unwrap()
        );
    }
    // Room helper has four-seat projections and belongs to teams evidence; do not export this truncated fixture.
    let (mut g, id) = setup(0);
    let (_, c) = g.remove_board(&id).unwrap();
    g.regions[0].cards.push(c);
    g.regions[4].influence[0] = 1;
    g.regions[1].influence[0] = 1;
    apply(&mut g, 0, activate(&id));
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options[0].id, "region:1");
    choose(&mut g, vec!["region:1".into()]);
    assert_eq!(g.regions[4].influence[0], 1);
    checkpoint(&g);
}
#[test]
fn wm056_cost_loyalty_wrong_actor_timing_and_targets_are_atomic() {
    let (mut g, id) = setup(0);
    g.players[0].assets.pop();
    reject(&mut g, 0, activate(&id));
    fund(&mut g, 0, "JC093", 1);
    reject(&mut g, 1, activate(&id));
    g.begin_window(Window::Before(2, 0));
    reject(&mut g, 0, activate(&id));
    g.begin_window(Window::Action(0));
    let mut bad = activate(&id);
    bad.target_id = Some("region:1".into());
    reject(&mut g, 0, bad);
    assert_eq!(g.resources(0), 2);
    assert!(!g.turn_ability_usage.iter().any(|u| u.ability_key == KEY));
    checkpoint(&g);
}
#[test]
fn wm056_choice_actor_counts_regions_and_enemy_only_marker_rejected_without_change() {
    let (mut g, id) = setup(0);
    g.regions[1].influence[0] = 1;
    g.regions[3].influence[1] = 1;
    apply(&mut g, 0, activate(&id));
    pass_top(&mut g);
    let a = pick(&g, 1);
    reject(&mut g, 1, a);
    let a = pick(&g, 3);
    reject(&mut g, 0, a);
    let mut a = pick(&g, 1);
    a.selected = Some(vec![]);
    reject(&mut g, 0, a);
    let mut a = pick(&g, 1);
    a.selected = Some(vec!["region:1".into(), "region:1".into()]);
    reject(&mut g, 0, a);
    snap("origin-single", &g, &pick(&g, 1));
    choose(&mut g, vec!["region:1".into()]);
    assert_eq!(g.regions[3].influence[1], 1);
    checkpoint(&g);
}
#[test]
fn wm056_exhausted_source_no_influence_still_pays_and_consumes_once_quota() {
    let (mut g, id) = setup(0);
    g.board_mut(&id).unwrap().exhausted = true;
    apply(&mut g, 0, activate(&id));
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert_eq!(g.resources(0), 0);
    fund(&mut g, 0, "JC093", 2);
    g.regions[1].influence[0] = 1;
    reject(&mut g, 0, activate(&id));
    checkpoint(&g);
}
#[test]
fn wm056_once_quota_follows_instance_through_control_and_resets_next_turn() {
    let (mut g, id) = setup(0);
    apply(&mut g, 0, activate(&id));
    pass_top(&mut g);
    g.board_mut(&id).unwrap().controller = 1;
    fund(&mut g, 1, "JC093", 2);
    reject(&mut g, 1, activate(&id));
    g.board_mut(&id).unwrap().controller = 0;
    fund(&mut g, 0, "JC093", 2);
    g.turn += 1;
    g.turn_ability_usage.clear();
    g.regions[1].influence[0] = 1;
    apply(&mut g, 0, activate(&id));
    pass_top(&mut g);
    choose(&mut g, vec!["region:1".into()]);
    checkpoint(&g);
}
#[test]
fn wm056_source_departure_control_and_movement_keep_original_destination_and_team() {
    for n in 0..3 {
        let (mut g, id) = setup(0);
        g.regions[1].influence[0] = 1;
        apply(&mut g, 0, activate(&id));
        match n {
            0 => {
                let (_, c) = g.remove_board(&id).unwrap();
                g.players[0].hand.push(c);
            }
            1 => g.board_mut(&id).unwrap().controller = 2,
            _ => {
                let (_, c) = g.remove_board(&id).unwrap();
                g.regions[4].cards.push(c);
            }
        }
        pass_top(&mut g);
        choose(&mut g, vec!["region:1".into()]);
        assert_eq!(g.regions[2].influence[0], 1);
        assert_eq!(g.regions[4].influence[0], 0);
        checkpoint(&g);
    }
}
#[test]
fn wm056_real_fast_hide_response_preserves_original_paid_destination() {
    let (mut g, id) = setup(0);
    g.regions[1].influence[0] = 1;
    let responder = board(&mut g, "JC063", 0, 2);
    apply(&mut g, 0, activate(&id));
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(responder),
            ability_id: Some("chase".into()),
            option: Some("hide".into()),
            target_id: Some(id.clone()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    assert!(g.board(&id).is_none());
    pass_top(&mut g);
    choose(&mut g, vec!["region:1".into()]);
    assert_eq!(g.regions[2].influence[0], 1);
    assert_eq!(g.regions[1].influence[0], 0);
    checkpoint(&g);
}
#[test]
fn wm056_response_time_marker_changes_and_original_destination_replacement() {
    let (mut g, id) = setup(0);
    g.regions[1].influence[0] = 1;
    apply(&mut g, 0, activate(&id));
    g.regions[1].influence[0] = 0;
    g.regions[3].influence[0] = 1;
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options[0].id, "region:3");
    choose(&mut g, vec!["region:3".into()]);
    checkpoint(&g);
    let (mut g, id) = setup(0);
    g.regions[1].influence[0] = 1;
    apply(&mut g, 0, activate(&id));
    g.regions[2].card.id = "replacement-region".into();
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert_eq!(g.regions[1].influence[0], 1);
    assert_eq!(g.regions[2].influence[0], 0);
    checkpoint(&g);
}
#[test]
fn wm056_moved_marker_cancels_opposition_and_can_open_real_win_window() {
    let (mut g, id) = setup(0);
    g.regions[1].influence[0] = 1;
    g.regions[2].influence[1] = 1;
    apply(&mut g, 0, activate(&id));
    pass_top(&mut g);
    choose(&mut g, vec!["region:1".into()]);
    assert_eq!(g.regions[2].influence, [0, 0]);
    checkpoint(&g);
    let (mut g, id) = setup(0);
    let threshold = catalog::card(&g.regions[2].card.definition)
        .threshold
        .unwrap();
    g.regions[2].influence[0] = threshold - 1;
    g.regions[1].influence[0] = 1;
    apply(&mut g, 0, activate(&id));
    pass_top(&mut g);
    choose(&mut g, vec!["region:1".into()]);
    assert_eq!(g.window, Some(Window::Win(2, 0)));
    checkpoint(&g);
}
#[test]
fn wm056_invalid_frame_payment_cursor_quota_and_response_bypass_rejected() {
    let (mut g, id) = setup(0);
    apply(&mut g, 0, activate(&id));
    for n in 0..6 {
        let mut bad = g.clone();
        let f = bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
        match n {
            0 => f.already_paid.clear(),
            1 => f.actor = 1,
            2 => f.cursor = 1,
            3 => f.steps.push(f.steps[0].clone()),
            4 => bad.turn_ability_usage.clear(),
            _ => f.source.source_region_instance = None,
        }
        invalid(&g, bad, &format!("frame-{n}"));
    }
    let mut bad = g.clone();
    let f = bad.stack.pop().unwrap().frame.unwrap();
    bad.effects.push_back(Effect::Frame { frame: Box::new(f) });
    invalid(&g, bad, "response-bypass");
    let mut bad = g.clone();
    let source = g
        .stack
        .last()
        .unwrap()
        .frame
        .as_ref()
        .unwrap()
        .source
        .clone();
    let mut ability = definition("WM056").abilities[0].clone();
    ability.key = "fake-trigger".into();
    ability.event = Some(Event::Enter);
    ability.costs.clear();
    ability.per_turn_limit = None;
    ability.activation_only = false;
    ability.ops = vec![Op::Draw {
        player: PlayerRef::Actor,
        count: 1,
        end: DeckEnd::Top,
    }];
    bad.effects.push_back(Effect::Declare {
        declaration: Declaration {
            actor: 0,
            source,
            ability,
        },
    });
    invalid(&g, bad, "unknown-trigger-declaration");
    let mut g = game(0);
    fund(&mut g, 0, "JC093", 1);
    let c = g.make_card("WM056", 0);
    let id = c.id.clone();
    g.players[0].hand.push(c);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    for n in 0..5 {
        let mut bad = g.clone();
        let s = bad.stack.last_mut().unwrap();
        match n {
            0 => s.card.as_mut().unwrap().definition = "JC125".into(),
            1 => s.id = "fake-stack".into(),
            2 => s.deploy_region = Some(3),
            3 => s.card.as_mut().unwrap().controller = 1,
            _ => s.frame.as_mut().unwrap().source.card.definition = "JC125".into(),
        }
        invalid(&g, bad, &format!("deploy-shell-{n}"));
    }
}

#[test]
fn wm056_canonical_acquired_renown_continuation_remains_optional_and_restorable() {
    let (mut g, id) = setup(0);
    for accept in [false, true] {
        let source = g.source_snapshot(g.board(&id).unwrap().1, Some(2));
        let ability = crate::renown::renown_ability(&g.regions[2].card.id);
        g.effects.push_back(Effect::Declare {
            declaration: Declaration {
                actor: 0,
                source,
                ability,
            },
        });
        g.drive().unwrap();
        checkpoint(&g); // Prepared canonical acquired reward; not a claim that WM056 prints renown.
        choose(
            &mut g,
            if accept {
                vec!["accept".into()]
            } else {
                vec![]
            },
        );
        if accept {
            pass_top(&mut g);
        }
        assert_eq!(g.regions[2].influence[0], if accept { 1 } else { 0 });
        checkpoint(&g);
    }
}
#[test]
fn wm056_pending_regions_options_and_selection_metadata_are_closed() {
    let (mut g, id) = setup(0);
    g.regions[1].influence[0] = 1;
    apply(&mut g, 0, activate(&id));
    pass_top(&mut g);
    for n in 0..9 {
        let mut bad = g.clone();
        let p = bad.pending.as_mut().unwrap();
        match n {
            0 => p.seat = 1,
            1 => p.choice.min = Some(0),
            2 => p.choice.options[0].id = "region:3".into(),
            3 => p.choice.title = "fake".into(),
            4 => {
                let ChoiceResolution::Frame { frame, .. } = &mut p.resolution else {
                    panic!()
                };
                frame.cursor = 0;
            }
            5 => bad.regions[1].card.id = "replaced".into(),
            6 => p.choice.amount = Some(2),
            7 => p.choice.player_id = "p1".into(),
            _ => p.choice.allow_decline = Some(true),
        }
        invalid(&g, bad, &format!("pending-{n}"));
    }
    let mut bad = g.clone();
    let ChoiceResolution::Frame { frame, .. } = &mut bad.pending.as_mut().unwrap().resolution else { panic!() };
    frame.source.card.definition = "JC125".into();
    frame.ability_key = "foreign-program".into();
    frame.actor = usize::MAX;
    frame.steps.clear();
    invalid(&g, bad, "pending-foreign-source-actor");
}
