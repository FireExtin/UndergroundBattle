//! Explicit offline layouts, production payments/actions and current Room evidence.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, rules::*};
const KEY: &str = "jc094-local-sacrifice-destroy";
fn held(g: &mut Game, actor: usize, def: &str) -> String {
    let c = g.make_card(def, actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn deploy(g: &mut Game, actor: usize) -> String {
    fund(g, actor, "JC093", 6);
    let source = held(g, actor, "JC094");
    apply(
        g,
        actor,
        Action {
            card_id: Some(source),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(g);
    g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC094")
        .unwrap()
        .id
        .clone()
}
fn action(source: &str, cost: &str, target: &str) -> Action {
    Action {
        card_id: Some(source.into()),
        ability_id: Some(KEY.into()),
        cost_selected: Some(vec![cost.into()]),
        target_id: Some(target.into()),
        ..Action::new("activate")
    }
}
fn snapshot(name: &str, g: &Game, a: &Action) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("JC094_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{name}.json"),serde_json::to_vec(&serde_json::json!({"scope":"offline-jc094-current-native-layout","state":serde_json::to_string(&r).unwrap(),"action":a,"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()})).unwrap()).unwrap();
    }
}
fn invalid(g: &Game, bad: Game, name: &str) {
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut r = envelope(g);
    r.game = bad;
    let state = serde_json::to_string(&r).unwrap();
    assert!(crate::room::RoomEnvelope::from_persisted(&state).is_err());
    if let Ok(dir) = std::env::var("JC094_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/{name}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn jc094_complete_original_and_closed_local_other_sacrifice_cost() {
    let c = catalog::card("JC094");
    assert_eq!(
        (&*c.name, c.subtitle.as_deref(), c.cost, c.defense),
        ("威达的汀巴巴迪", Some("巫毒大师"), 6, Some(3))
    );
    assert!(c.unique);
    assert_eq!(c.loyalty, vec!["黑色"; 3]);
    assert_eq!(c.magic, "死亡");
    assert_eq!(c.subtypes, vec!["人类", "法师", "罪犯"]);
    assert_eq!(
        c.permanent_icons,
        Icons {
            investigation: 0,
            combat: 0,
            influence: 2
        }
    );
    assert_eq!(
        c.temporary_icons,
        Icons {
            investigation: 3,
            combat: 0,
            influence: 0
        }
    );
    let d = definition("JC094");
    let a = &d.abilities[0];
    assert_eq!(a.timing, Timing::Standard);
    assert!(a.activation_only);
    assert!(matches!(
        a.costs.as_slice(),
        [Cost::SacrificeSelectedOtherLocalControlledCharacter]
    ));
    assert_eq!(a.targets[0].kind, EntityKind::CharacterOrHidden);
    assert_eq!(a.targets[0].range, Range::Anywhere);
    for change in 0..6 {
        let mut bad = d.clone();
        match change {
            0 => bad.abilities[0].timing = Timing::Fast,
            1 => bad.abilities[0].costs = vec![Cost::SacrificeSelectedControlledCharacter],
            2 => bad.abilities[0].costs.clear(),
            3 => bad.abilities[0].targets[0].kind = EntityKind::Character,
            4 => bad.abilities[0].activation_only = false,
            _ => bad.abilities[0].event = Some(Event::Enter),
        }
        let mut defs = definitions().clone();
        defs.insert("JC094".into(), bad);
        assert!(crate::rules::validate_definitions(&defs).is_err());
    }
    let mut transplanted = definitions().clone();
    transplanted.insert("JC125".into(), d.clone());
    assert!(crate::rules::validate_definitions(&transplanted).is_err());
    crate::rules::validate_definitions(definitions()).unwrap();
}

#[test]
fn jc094_paid_deploy_and_standard_activation_destroy_public_or_hidden_any_region_all_seats() {
    for actor in 0..4 {
        for hidden in [false, true] {
            let mut g = game(actor);
            let source = deploy(&mut g, actor);
            let cost = board(&mut g, "JC125", actor, 2);
            let target = board(&mut g, "JC125", (actor + 1) % 4, 4);
            g.board_mut(&target).unwrap().face_down = hidden;
            let a = action(&source, &cost, &target);
            if actor == 0 {
                snapshot(
                    if hidden {
                        "hidden-target"
                    } else {
                        "public-target"
                    },
                    &g,
                    &a,
                );
            }
            apply(&mut g, actor, a);
            assert!(g.board(&cost).is_none());
            assert!(g.players[actor]
                .graveyard
                .iter()
                .any(|c| c.definition == "JC125"));
            assert!(g.board(&target).is_some());
            pass_top(&mut g);
            assert!(g.board(&target).is_none());
            checkpoint(&g);
        }
    }
}

#[test]
fn jc094_action_projection_only_has_non_source_local_controlled_public_costs() {
    let mut g = game(0);
    let source = deploy(&mut g, 0);
    let cost = board(&mut g, "JC125", 0, 2);
    let own_far = board(&mut g, "JC125", 0, 3);
    let ally = board(&mut g, "JC125", 2, 2);
    let hidden = board(&mut g, "JC125", 0, 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    let target = board(&mut g, "JC125", 1, 4);
    let offered = g
        .legal_actions(0)
        .into_iter()
        .filter(|a| {
            a.action.card_id.as_deref() == Some(&source)
                && a.action.ability_id.as_deref() == Some(KEY)
        })
        .collect::<Vec<_>>();
    assert!(!offered.is_empty());
    assert!(offered
        .iter()
        .all(|a| a.action.cost_selected.as_ref() == Some(&vec![cost.clone()])));
    for illegal in [&source, &own_far, &ally, &hidden] {
        reject(&mut g, 0, action(&source, illegal, &target));
    }
    reject(&mut g, 1, action(&source, &cost, &target));
    let mut a = action(&source, &cost, &target);
    a.cost_selected = None;
    reject(&mut g, 0, a);
    let mut a = action(&source, &cost, &target);
    a.cost_selected = Some(vec![cost.clone(), cost.clone()]);
    reject(&mut g, 0, a);
    snapshot(
        "local-cost-projection",
        &g,
        &action(&source, &cost, &target),
    );
    checkpoint(&g);
}

#[test]
fn jc094_not_fast_no_cost_and_no_target_invalidations_are_atomic() {
    let mut g = game(0);
    let source = deploy(&mut g, 0);
    let target = board(&mut g, "JC125", 1, 4);
    assert!(!g
        .legal_actions(0)
        .iter()
        .any(|a| a.action.ability_id.as_deref() == Some(KEY)));
    reject(&mut g, 0, action(&source, "missing", &target));
    let cost = board(&mut g, "JC125", 0, 2);
    g.begin_window(Window::Before(2, 0));
    reject(&mut g, 0, action(&source, &cost, &target));
    g.begin_window(Window::Action(0));
    reject(&mut g, 0, action(&source, &cost, "missing-target"));
    assert!(g.board(&cost).is_some());
    checkpoint(&g);
}

#[test]
fn jc094_can_target_self_or_the_sacrificed_character_without_refund() {
    for same_cost in [false, true] {
        let mut g = game(0);
        let source = deploy(&mut g, 0);
        let cost = board(&mut g, "JC125", 0, 2);
        let target = if same_cost {
            cost.clone()
        } else {
            source.clone()
        };
        snapshot(
            if same_cost {
                "target-is-cost"
            } else {
                "self-target"
            },
            &g,
            &action(&source, &cost, &target),
        );
        apply(&mut g, 0, action(&source, &cost, &target));
        pass_top(&mut g);
        assert!(g.board(&cost).is_none());
        assert_eq!(g.board(&source).is_none(), !same_cost);
        checkpoint(&g);
    }
}

#[test]
fn jc094_source_exhaustion_does_not_add_unprinted_cost() {
    let mut g = game(0);
    let source = deploy(&mut g, 0);
    g.board_mut(&source).unwrap().exhausted = true;
    let cost = board(&mut g, "JC125", 0, 2);
    let target = board(&mut g, "JC125", 1, 4);
    apply(&mut g, 0, action(&source, &cost, &target));
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    checkpoint(&g);
}

#[test]
fn jc094_cost_owner_retained_and_sacrifice_cannot_be_stopped_by_shield() {
    let mut g = game(0);
    let source = deploy(&mut g, 0);
    let cost = board(&mut g, "JC125", 1, 2);
    let c = g.board_mut(&cost).unwrap();
    c.controller = 0;
    c.shield = 2;
    let target = board(&mut g, "JC125", 1, 4);
    apply(&mut g, 0, action(&source, &cost, &target));
    assert!(g.board(&cost).is_none());
    assert!(g.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125" && c.owner == 1));
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    checkpoint(&g);
}

#[test]
fn jc094_target_shield_preserves_target_but_cost_remains_paid() {
    let mut g = game(0);
    let source = deploy(&mut g, 0);
    let cost = board(&mut g, "JC125", 0, 2);
    let target = board(&mut g, "JC125", 2, 4);
    g.board_mut(&target).unwrap().shield = 1;
    apply(&mut g, 0, action(&source, &cost, &target));
    pass_top(&mut g);
    assert!(g.board(&cost).is_none());
    assert_eq!(g.board(&target).unwrap().1.shield, 0);
    checkpoint(&g);
}

#[test]
fn jc094_real_fast_hide_response_fizzles_original_target_after_sacrifice() {
    let mut g = game(0);
    let source = deploy(&mut g, 0);
    let cost = board(&mut g, "JC125", 0, 2);
    let target = board(&mut g, "JC125", 1, 4);
    let responder = board(&mut g, "JC063", 0, 2);
    apply(&mut g, 0, action(&source, &cost, &target));
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(responder),
            ability_id: Some("chase".into()),
            option: Some("hide".into()),
            target_id: Some(target.clone()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    pass_top(&mut g);
    assert!(g.board(&cost).is_none() && g.board(&target).is_none());
    assert!(g.regions[4]
        .cards
        .iter()
        .any(|c| c.definition == "JC125" && c.face_down));
    checkpoint(&g);
}

#[test]
fn jc094_frozen_paid_actor_and_target_survive_source_departure_and_movement() {
    for change in 0..3 {
        let mut g = game(0);
        let source = deploy(&mut g, 0);
        let cost = board(&mut g, "JC125", 0, 2);
        let target = board(&mut g, "JC125", 1, 4);
        apply(&mut g, 0, action(&source, &cost, &target));
        match change {
            0 => {
                let (_, c) = g.remove_board(&source).unwrap();
                g.players[0].hand.push(c);
            }
            1 => g.board_mut(&source).unwrap().controller = 1,
            _ => {
                let (_, c) = g.remove_board(&source).unwrap();
                g.regions[3].cards.push(c);
            }
        }
        pass_top(&mut g);
        assert!(g.board(&target).is_none());
        assert!(g.board(&cost).is_none());
        checkpoint(&g);
    }
}

#[test]
fn jc094_payment_frame_program_and_response_boundary_tampering_rejected() {
    let mut g = game(0);
    let source = deploy(&mut g, 0);
    let cost = board(&mut g, "JC125", 0, 2);
    let target = board(&mut g, "JC125", 1, 4);
    apply(&mut g, 0, action(&source, &cost, &target));
    for change in 0..6 {
        let mut bad = g.clone();
        let f = bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
        match change {
            0 => f.already_paid.clear(),
            1 => f.already_paid = vec![PaidCost::Assets(vec!["fake".into()])],
            2 => f.actor = 1,
            3 => f.cursor = 1,
            4 => f.targets[0].spec.kind = EntityKind::Character,
            _ => {
                f.already_paid = vec![PaidCost::Sacrificed {
                    old_instance: source.clone(),
                    controller: 0,
                    owner: 0,
                }]
            }
        }
        invalid(&g, bad, &format!("frame-{change}"));
    }
    let mut bad = g.clone();
    let f = bad.stack.pop().unwrap().frame.unwrap();
    bad.effects.push_back(Effect::Frame { frame: Box::new(f) });
    invalid(&g, bad, "frame-response-bypass");
}

#[test]
fn jc094_sacrificing_time_character_emits_real_frozen_death_trigger_before_completion() {
    let mut g = game(0);
    let source = deploy(&mut g, 0);
    let cost = board(&mut g, "JZ30", 0, 2);
    g.board_mut(&cost).unwrap().time_markers = 3;
    let target = board(&mut g, "JC125", 2, 4);
    apply(&mut g, 0, action(&source, &cost, &target));
    let p = g.pending.as_ref().unwrap();
    let ChoiceResolution::Declare { declaration, .. } = &p.resolution else {
        panic!("death declaration missing")
    };
    assert_eq!(declaration.source.card.definition, "JZ30");
    assert_eq!(declaration.source.card.time_markers, 3);
    assert_eq!(declaration.actor, 0);
    assert!(g.board(&cost).is_none());
    choose(&mut g, vec![]);
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    checkpoint(&g);
}
