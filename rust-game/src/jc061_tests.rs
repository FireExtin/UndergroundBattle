//! Explicit offline layouts; actual paid Game/Room commands and restore guards.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, rules::*};

fn hidden(g: &mut Game, seat: usize, region: usize) -> String {
    let id = board(g, "JC125", seat, region);
    g.board_mut(&id).unwrap().face_down = true;
    id
}
fn held(g: &mut Game, actor: usize) -> String {
    let c = g.make_card("JC061", actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn deploy(g: &mut Game, actor: usize, assets: usize) -> String {
    fund(g, actor, "JC059", assets);
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
        .find(|c| c.definition == "JC061")
        .unwrap()
        .id
        .clone()
}
fn snapshot(name: &str, g: &Game) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("JC061_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let room = envelope(g);
        std::fs::write(format!("{dir}/{name}.json"),serde_json::to_vec(&serde_json::json!({
            "scope":"offline-jc061-current-native-layout", "state":serde_json::to_string(&room).unwrap(),
            "views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
}
fn invalid(g: &Game, broken: Game, name: &str) {
    assert!(Game::from_persisted(&serde_json::to_string(&broken).unwrap()).is_err());
    let mut room = envelope(g);
    room.game = broken;
    let state = serde_json::to_string(&room).unwrap();
    assert!(crate::room::RoomEnvelope::from_persisted(&state).is_err());
    if let Ok(dir) = std::env::var("JC061_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/{name}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn jc061_original_fields_and_closed_programs() {
    let c = catalog::card("JC061");
    assert_eq!(
        (&*c.name, &*c.color, c.cost, c.defense),
        ("危机处理特派小组", "灰", 5, Some(2))
    );
    assert_eq!(c.loyalty, vec!["灰色", "灰色"]);
    assert_eq!(c.subtypes, vec!["人类", "特工"]);
    assert_eq!(
        c.permanent_icons,
        Icons {
            investigation: 1,
            combat: 2,
            influence: 1
        }
    );
    assert_eq!(c.temporary_icons, Icons::default());
    assert!(c.magic.is_empty() && !c.unique);
    assert_eq!(c.keywords, vec!["机动"]);
    let d = definition("JC061");
    assert!(!d.traits.public && !d.traits.city_play_only);
    let a = &d.abilities[0];
    assert_eq!(a.event, Some(Event::EnterRegion));
    assert_eq!(a.costs.len(), 1);
    assert!(matches!(a.costs[0], Cost::Assets(1)));
    assert_eq!(a.targets[0].kind, EntityKind::Hidden);
    assert_eq!(a.targets[0].relation, Relation::Any);
    assert_eq!(a.targets[0].range, Range::SourceRegion);
    assert!(d.abilities[1].requires_ready_source);
    assert_eq!(d.abilities[1].event, Some(Event::ConfrontationStart));
    for change in 0..7 {
        let mut changed = d.clone();
        match change {
            0 => changed.abilities[0].costs.clear(),
            1 => changed.abilities[0].costs = vec![Cost::Assets(2)],
            2 => changed.abilities[0].event = Some(Event::Enter),
            3 => changed.abilities[0].targets[0].relation = Relation::EnemyTeam,
            4 => changed.abilities[0].targets[0].kind = EntityKind::Character,
            5 => changed.traits.public = true,
            _ => changed.abilities[0].response_policy = ResponsePolicy::Immediate,
        }
        let mut registry = definitions().clone();
        registry.insert("JC061".into(), changed);
        assert!(crate::rules::validate_definitions(&registry).is_err());
    }
    crate::rules::validate_definitions(definitions()).unwrap();
}

#[test]
fn jc061_paid_entry_all_seats_hidden_only_and_optional_decline() {
    for actor in 0..4 {
        for accept in [false, true] {
            let mut g = game(actor);
            let target = hidden(&mut g, (actor + 2) % 4, 2);
            let own = hidden(&mut g, actor, 2);
            let far = hidden(&mut g, (actor + 1) % 4, 3);
            let visible = board(&mut g, "JC125", (actor + 1) % 4, 2);
            let source = deploy(&mut g, actor, 6);
            assert_eq!(g.resources(actor), 1);
            let p = g.pending.clone().unwrap();
            assert_eq!(p.seat, actor);
            let ids = p
                .choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<Vec<_>>();
            assert!(ids.contains(&target) && ids.contains(&own));
            assert!(!ids.contains(&far) && !ids.contains(&visible));
            reject(
                &mut g,
                (actor + 1) % 4,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec![target.clone()]),
                    ..Action::new("choose")
                },
            );
            reject(
                &mut g,
                actor,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec![visible]),
                    ..Action::new("choose")
                },
            );
            if actor == 0 {
                snapshot(
                    if accept {
                        "entry-accept"
                    } else {
                        "entry-decline"
                    },
                    &g,
                );
            }
            choose(&mut g, if accept { vec![target.clone()] } else { vec![] });
            assert_eq!(g.resources(actor), if accept { 0 } else { 1 });
            if accept {
                pass_top(&mut g);
            }
            assert_eq!(g.board(&target).is_none(), accept);
            assert!(g.board(&source).is_some());
            assert!(g.board(&own).is_some() && g.board(&far).is_some());
            checkpoint(&g);
        }
    }
}

#[test]
fn jc061_no_asset_accept_rejected_atomically_but_decline_is_free() {
    let mut g = game(0);
    let target = hidden(&mut g, 1, 2);
    deploy(&mut g, 0, 5);
    let p = g.pending.clone().unwrap();
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![target.clone()]),
            ..Action::new("choose")
        },
    );
    assert_eq!(g.resources(0), 0);
    choose(&mut g, vec![]);
    assert!(g.board(&target).is_some());
}

#[test]
fn jc061_deploy_cost_and_loyalty_are_separate_from_optional_payment() {
    for gray in [false, true] {
        let mut g = game(0);
        let source = held(&mut g, 0);
        fund(
            &mut g,
            0,
            if gray { "JC059" } else { "JC125" },
            if gray { 4 } else { 5 },
        );
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(source),
                region: Some(2),
                ..Action::new("deploy")
            },
        );
    }
    let mut g = game(0);
    deploy(&mut g, 0, 5);
    assert_eq!(g.resources(0), 0);
    assert!(g.pending.is_none());
}

#[test]
fn jc061_secret_deploy_does_not_trigger_but_paid_reveal_does() {
    for actor in 0..4 {
        let mut g = game(actor);
        let target = hidden(&mut g, (actor + 1) % 4, 2);
        fund(&mut g, actor, "JC059", 7);
        let id = held(&mut g, actor);
        apply(
            &mut g,
            actor,
            Action {
                card_id: Some(id),
                region: Some(2),
                ..Action::new("conceal")
            },
        );
        assert!(g.pending.is_none());
        let source = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "JC061")
            .unwrap()
            .id
            .clone();
        apply(
            &mut g,
            actor,
            Action {
                card_id: Some(source),
                ..Action::new("reveal")
            },
        );
        pass_top(&mut g);
        assert_eq!(g.resources(actor), 1);
        assert_eq!(g.pending.as_ref().unwrap().seat, actor);
        choose(&mut g, vec![target.clone()]);
        pass_top(&mut g);
        assert!(g.board(&target).is_none());
    }
}

#[test]
fn jc061_natural_mobility_retriggers_in_destination_and_ready_required() {
    for actor in 0..4 {
        let mut g = game(actor);
        let source = board(&mut g, "JC061", actor, 2);
        let target = hidden(&mut g, (actor + 1) % 4, 3);
        fund(&mut g, actor, "JC059", 1);
        g.board_mut(&source).unwrap().lock_markers = 2;
        g.begin_window(Window::Action(1 - g.first_team));
        g.close_window().unwrap();
        g.drive().unwrap();
        checkpoint(&g);
        assert_eq!(g.pending.as_ref().unwrap().seat, actor);
        if actor == 0 {
            snapshot("mobility-choice", &g);
        }
        choose(&mut g, vec!["region:3".into()]);
        pass_top(&mut g);
        assert_eq!(g.board(&source).unwrap().0, 3);
        assert_eq!(g.board(&source).unwrap().1.lock_markers, 2);
        assert_eq!(g.pending.as_ref().unwrap().choice.options[0].id, target);
        if actor == 0 {
            snapshot("mobility-entry-choice", &g);
        }
        choose(&mut g, vec![target.clone()]);
        pass_top(&mut g);
        assert!(g.board(&target).is_none());
    }
    let mut g = game(0);
    let source = board(&mut g, "JC061", 0, 2);
    g.board_mut(&source).unwrap().exhausted = true;
    g.begin_window(Window::Action(1 - g.first_team));
    g.close_window().unwrap();
    g.drive().unwrap();
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.board(&source).unwrap().0, 2);
}

#[test]
fn jc061_response_guards_target_identity_region_and_hidden_status_without_refund() {
    for change in 0..5 {
        let mut g = game(0);
        let target = hidden(&mut g, 1, 2);
        deploy(&mut g, 0, 6);
        choose(&mut g, vec![target.clone()]);
        assert_eq!(g.resources(0), 0);
        match change {
            0 => g.board_mut(&target).unwrap().face_down = false,
            1 => {
                let (_, c) = g.remove_board(&target).unwrap();
                g.regions[3].cards.push(c);
            }
            2 => {
                g.regions[2].card = g.make_card("DQJC112", 0);
            }
            3 => {
                let (_, c) = g.remove_board(&target).unwrap();
                let c = g.fresh(c);
                g.regions[2].cards.push(c);
            }
            _ => g.board_mut(&target).unwrap().controller = 2,
        }
        pass_top(&mut g);
        assert_eq!(g.resources(0), 0);
        assert_eq!(
            g.board(&target).is_none(),
            change == 3 || change == 4,
            "response change {change}"
        );
        checkpoint(&g);
    }
}

#[test]
fn jc061_declared_source_departure_control_or_movement_preserves_frozen_actor() {
    for change in 0..3 {
        let mut g = game(0);
        let target = hidden(&mut g, 1, 2);
        let source = deploy(&mut g, 0, 6);
        choose(&mut g, vec![target.clone()]);
        match change {
            0 => {
                let (_, c) = g.remove_board(&source).unwrap();
                g.players[0].graveyard.push(c);
            }
            1 => g.board_mut(&source).unwrap().controller = 2,
            _ => {
                let (_, c) = g.remove_board(&source).unwrap();
                g.regions[3].cards.push(c);
            }
        }
        pass_top(&mut g);
        assert!(g.board(&target).is_none());
        assert_eq!(g.resources(0), 0);
        checkpoint(&g);
    }
}

#[test]
fn jc061_paid_frame_and_pending_metadata_tampering_rejected() {
    let mut g = game(0);
    let target = hidden(&mut g, 1, 2);
    deploy(&mut g, 0, 6);
    for change in 0..4 {
        let mut bad = g.clone();
        let p = bad.pending.as_mut().unwrap();
        match change {
            0 => p.seat = 1,
            1 => p.choice.options.clear(),
            2 => p.choice.allow_decline = Some(false),
            _ => {
                if let ChoiceResolution::Declare { declaration, .. } = &mut p.resolution {
                    declaration.ability.costs.clear();
                }
            }
        }
        invalid(&g, bad, &format!("pending-{change}"));
    }
    choose(&mut g, vec![target]);
    let mut bypass = g.clone();
    let queued = bypass.stack.pop().unwrap().frame.unwrap();
    bypass.effects.push_back(Effect::Frame {
        frame: Box::new(queued),
    });
    invalid(&g, bypass, "frame-response-bypass");
    for change in 0..6 {
        let mut bad = g.clone();
        let f = bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
        match change {
            0 => f.already_paid.clear(),
            1 => f.already_paid = vec![PaidCost::Assets(vec!["one".into(), "two".into()])],
            2 => f.actor = 1,
            3 => f.source.source_region_instance = None,
            4 => f.cursor = 1,
            _ => f.targets[0].spec.relation = Relation::EnemyTeam,
        }
        invalid(&g, bad, &format!("frame-{change}"));
    }
}
