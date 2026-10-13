//! Offline layouts; paid Game/Room commands, real fast responses, no live injection.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, rules::*};

fn held(g: &mut Game, actor: usize) -> String {
    let c = g.make_card("JC087", actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn deploy(g: &mut Game, actor: usize) -> String {
    fund(g, actor, "JC093", 4);
    let id = held(g, actor);
    apply(
        g,
        actor,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(g);
    g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC087")
        .unwrap()
        .id
        .clone()
}
fn snapshot(name: &str, g: &Game, selected: Vec<String>) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("JC087_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{name}.json"),serde_json::to_vec(&serde_json::json!({
 "scope":"offline-jc087-current-native-layout","selected":selected,"state":serde_json::to_string(&r).unwrap(),"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()
 })).unwrap()).unwrap();
    }
}
fn invalid(g: &Game, bad: Game, name: &str) {
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut r = envelope(g);
    r.game = bad;
    let state = serde_json::to_string(&r).unwrap();
    assert!(crate::room::RoomEnvelope::from_persisted(&state).is_err());
    if let Ok(dir) = std::env::var("JC087_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/{name}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn jc087_complete_original_fields_and_closed_conditional_program() {
    let c = catalog::card("JC087");
    assert_eq!(
        (&*c.name, &*c.color, c.cost, c.defense),
        ("格鲁尼的走私客", "黑", 4, Some(2))
    );
    assert_eq!(c.loyalty, vec!["黑色", "黑色"]);
    assert_eq!(c.subtypes, vec!["人类", "罪犯"]);
    assert_eq!(
        c.permanent_icons,
        Icons {
            investigation: 0,
            combat: 1,
            influence: 1
        }
    );
    assert_eq!(
        c.temporary_icons,
        Icons {
            investigation: 1,
            combat: 1,
            influence: 0
        }
    );
    assert_eq!(c.keywords, vec!["撤回"]);
    assert!(!c.unique && c.magic.is_empty());
    let d = definition("JC087");
    assert!(d.traits.retreat && !d.traits.public);
    let a = &d.abilities[0];
    assert_eq!(a.event, Some(Event::EnterRegion));
    assert!(a.costs.is_empty());
    assert_eq!(a.targets[0].relation, Relation::ControlledByActor);
    assert!(!a.targets[0].exclude_source);
    for mutation in 0..6 {
        let mut changed = d.clone();
        match mutation {
            0 => changed.abilities[0].event = Some(Event::Enter),
            1 => changed.abilities[0].targets[0].relation = Relation::FriendlyTeam,
            2 => changed.abilities[0].targets[0].exclude_source = true,
            3 => changed.abilities[0].ops = vec![Op::Hide(EntityRef::Target(0))],
            4 => changed.traits.retreat = false,
            _ => changed.abilities[0].response_policy = ResponsePolicy::Immediate,
        }
        let mut defs = definitions().clone();
        defs.insert("JC087".into(), changed);
        assert!(crate::rules::validate_definitions(&defs).is_err());
    }
    crate::rules::validate_definitions(definitions()).unwrap();
}

#[test]
fn jc087_paid_entry_self_other_decline_and_exact_controller_all_seats() {
    for actor in 0..4 {
        for pick in 0..3 {
            let mut g = game(actor);
            g.regions[2].influence = [0, 0];
            let own = board(&mut g, "JC125", actor, 2);
            let ally = board(&mut g, "JC125", (actor + 2) % 4, 2);
            let enemy = board(&mut g, "JC125", (actor + 1) % 4, 2);
            let far = board(&mut g, "JC125", actor, 3);
            let hidden = board(&mut g, "JC125", actor, 2);
            g.board_mut(&hidden).unwrap().face_down = true;
            let source = deploy(&mut g, actor);
            assert_eq!(g.resources(actor), 0);
            let p = g.pending.clone().unwrap();
            let ids = p
                .choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<Vec<_>>();
            assert_eq!(ids.len(), 2);
            assert!(ids.contains(&source) && ids.contains(&own));
            for id in [&ally, &enemy, &far, &hidden] {
                assert!(!ids.contains(id));
            }
            reject(
                &mut g,
                (actor + 1) % 4,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec![own.clone()]),
                    ..Action::new("choose")
                },
            );
            if actor == 0 {
                snapshot(
                    ["decline", "self", "other"][pick],
                    &g,
                    match pick {
                        0 => vec![],
                        1 => vec![source.clone()],
                        _ => vec![own.clone()],
                    },
                );
            }
            choose(
                &mut g,
                match pick {
                    0 => vec![],
                    1 => vec![source.clone()],
                    _ => vec![own.clone()],
                },
            );
            if pick != 0 {
                pass_top(&mut g);
            }
            assert_eq!(g.regions[2].influence[g.team(actor)], u32::from(pick != 0));
            assert_eq!(g.board(&source).is_none(), pick == 1);
            assert_eq!(g.board(&own).is_none(), pick == 2);
            assert!(g.board(&ally).is_some() && g.board(&enemy).is_some());
            if pick != 0 {
                let c = g.regions[2]
                    .cards
                    .iter()
                    .find(|c| {
                        c.face_down
                            && c.definition == if pick == 1 { "JC087" } else { "JC125" }
                            && c.id != hidden
                    })
                    .unwrap();
                assert_eq!(c.owner, actor);
                assert!(!g.has_retreat(c));
            }
            checkpoint(&g);
        }
    }
}

#[test]
fn jc087_cost_and_black_loyalty_rejections_atomic() {
    for reason in 0..3 {
        let mut g = game(0);
        let source = held(&mut g, 0);
        fund(
            &mut g,
            0,
            if reason == 0 { "JC125" } else { "JC093" },
            if reason == 1 { 3 } else { 4 },
        );
        reject(
            &mut g,
            if reason == 2 { 1 } else { 0 },
            Action {
                card_id: Some(source),
                region: Some(2),
                ..Action::new("deploy")
            },
        );
    }
}

#[test]
fn jc087_secret_deploy_no_trigger_then_paid_reveal_self_hides() {
    for actor in 0..4 {
        let mut g = game(actor);
        g.regions[2].influence = [0, 0];
        fund(&mut g, actor, "JC093", 5);
        let source = held(&mut g, actor);
        apply(
            &mut g,
            actor,
            Action {
                card_id: Some(source),
                region: Some(2),
                ..Action::new("conceal")
            },
        );
        assert!(g.pending.is_none());
        let old = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "JC087")
            .unwrap()
            .id
            .clone();
        apply(
            &mut g,
            actor,
            Action {
                card_id: Some(old),
                ..Action::new("reveal")
            },
        );
        pass_top(&mut g);
        assert_eq!(g.resources(actor), 0);
        let source = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "JC087")
            .unwrap()
            .id
            .clone();
        choose(&mut g, vec![source]);
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence[g.team(actor)], 1);
        assert!(g.regions[2]
            .cards
            .iter()
            .any(|c| c.definition == "JC087" && c.face_down));
    }
}

#[test]
fn jc087_explicit_move_emits_real_enter_region_and_declines_without_retrigger() {
    let mut g = game(0);
    g.regions[3].influence = [0, 0];
    let source = board(&mut g, "JC087", 0, 2);
    let target = board(&mut g, "JC125", 0, 3);
    let (_, c) = g.remove_board(&source).unwrap();
    let snap = g.source_snapshot(&c, Some(3));
    g.regions[3].cards.push(c);
    g.emit_event(0, snap, Event::EnterRegion);
    g.drive().unwrap();
    checkpoint(&g);
    assert!(g
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .any(|o| o.id == target));
    snapshot("move-entry", &g, vec![target.clone()]);
    choose(&mut g, vec![target.clone()]);
    pass_top(&mut g);
    assert_eq!(g.regions[3].influence[0], 1);
    assert!(g.board(&target).is_none());
    assert!(g.pending.is_none() && g.stack.is_empty());
}

fn chase(g: &mut Game, source: &str, target: &str) {
    apply(
        g,
        0,
        Action {
            card_id: Some(source.into()),
            ability_id: Some("chase".into()),
            option: Some("hide".into()),
            target_id: Some(target.into()),
            ..Action::new("activate")
        },
    );
    pass_top(g);
}
#[test]
fn jc087_real_fast_hide_response_invalidates_target_but_not_departed_source() {
    for hide_source in [false, true] {
        let mut g = game(0);
        g.regions[2].influence = [0, 0];
        let target = board(&mut g, "JC125", 0, 2);
        let responder = board(&mut g, "JC063", 0, 2);
        let source = deploy(&mut g, 0);
        choose(&mut g, vec![target.clone()]);
        chase(
            &mut g,
            &responder,
            if hide_source { &source } else { &target },
        );
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence[0], u32::from(hide_source));
        assert!(g.board(&target).is_none());
        checkpoint(&g);
    }
}

#[test]
fn jc087_resolution_controller_region_identity_and_face_guards() {
    for change in 0..5 {
        let mut g = game(0);
        g.regions[2].influence = [0, 0];
        let target = board(&mut g, "JC125", 0, 2);
        deploy(&mut g, 0);
        choose(&mut g, vec![target.clone()]);
        match change {
            0 => g.board_mut(&target).unwrap().controller = 2,
            1 => {
                let (_, c) = g.remove_board(&target).unwrap();
                g.regions[3].cards.push(c);
            }
            2 => {
                g.hide_in_place(&target);
            }
            3 => {
                g.regions[2].card = g.make_card("DQJC112", 0);
            }
            _ => {
                let (_, c) = g.remove_board(&target).unwrap();
                let c = g.fresh(c);
                g.regions[2].cards.push(c);
            }
        };
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence[0], 0);
        assert_eq!(g.board(&target).is_none(), change == 2 || change == 4);
        checkpoint(&g);
    }
}

#[test]
fn jc087_source_control_owner_and_movement_do_not_transfer_original_team_reward() {
    for change in 0..3 {
        let mut g = game(0);
        g.regions[2].influence = [0, 0];
        let target = board(&mut g, "JC125", 0, 2);
        let source = deploy(&mut g, 0);
        choose(&mut g, vec![target.clone()]);
        match change {
            0 => {
                g.board_mut(&source).unwrap().controller = 1;
            }
            1 => {
                let (_, c) = g.remove_board(&source).unwrap();
                g.regions[3].cards.push(c);
            }
            _ => {
                let (_, c) = g.remove_board(&source).unwrap();
                g.players[0].hand.push(c);
            }
        }
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [1, 0]);
        assert!(g.board(&target).is_none());
        checkpoint(&g);
    }
}

#[test]
fn jc087_success_uses_existing_opposing_mark_cancellation_and_attachment_owner_return() {
    let mut g = game(0);
    g.regions[2].influence = [0, 1];
    let target = board(&mut g, "JC125", 0, 2);
    let equipment = g.make_card("BQ022", 1);
    g.attachments.push(Attachment {
        card: equipment,
        host_id: target.clone(),
    });
    deploy(&mut g, 0);
    choose(&mut g, vec![target.clone()]);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [0, 0]);
    assert!(g.attachments.is_empty());
    assert!(g.players[1].hand.iter().any(|c| c.definition == "BQ022"));
    assert!(g.board(&target).is_none());
    checkpoint(&g);
}

#[test]
fn jc087_retreat_on_winning_region_returns_to_owner_hand() {
    let mut g = game(0);
    let source = board(&mut g, "JC087", 0, 2);
    assert!(g.has_retreat(g.board(&source).unwrap().1));
    g.regions[2].card = g.make_card("DQJC109", 0);
    let threshold = catalog::card("DQJC109").threshold.unwrap();
    g.regions[2].influence = [0, 0];
    g.place_influence(0, 2, threshold);
    g.drive().unwrap();
    for _ in 0..64 {
        if g.players[0].hand.iter().any(|c| c.definition == "JC087") {
            break;
        }
        if g.pending.is_some() {
            let p = g.pending.as_ref().unwrap();
            let selected = if p.choice.min == Some(0) {
                vec![]
            } else {
                p.choice.options.iter().map(|o| o.id.clone()).collect()
            };
            choose(&mut g, selected);
        } else if !g.stack.is_empty() {
            pass_top(&mut g);
        } else if let Some(seat) = (0..4).find(|seat| {
            g.legal_actions(*seat)
                .iter()
                .any(|a| a.action.kind == "pass")
        }) {
            apply(&mut g, seat, Action::new("pass"));
        } else {
            break;
        }
    }
    assert!(g.players[0].hand.iter().any(|c| c.definition == "JC087"));
    assert!(!g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC087"));
    checkpoint(&g);
}

#[test]
fn jc087_definition_frame_metadata_and_response_bypass_rejected() {
    let mut g = game(0);
    let target = board(&mut g, "JC125", 0, 2);
    deploy(&mut g, 0);
    for change in 0..3 {
        let mut bad = g.clone();
        let p = bad.pending.as_mut().unwrap();
        match change {
            0 => p.seat = 1,
            1 => p.choice.options.clear(),
            _ => p.choice.max = Some(2),
        }
        invalid(&g, bad, &format!("pending-{change}"));
    }
    choose(&mut g, vec![target]);
    for change in 0..6 {
        let mut bad = g.clone();
        let f = bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
        match change {
            0 => f.source.source_region_instance = None,
            1 => f.actor = 1,
            2 => f.cursor = 1,
            3 => f.already_paid = vec![PaidCost::Assets(vec!["fake".into()])],
            4 => f.targets[0].spec.relation = Relation::FriendlyTeam,
            _ => f.steps[0].op = Op::Hide(EntityRef::Target(0)),
        }
        invalid(&g, bad, &format!("frame-{change}"));
    }
    let mut bad = g.clone();
    let f = bad.stack.pop().unwrap().frame.unwrap();
    bad.effects.push_back(Effect::Frame { frame: Box::new(f) });
    invalid(&g, bad, "frame-response-bypass");
}
