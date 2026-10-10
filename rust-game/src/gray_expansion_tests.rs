//! Offline layouts, followed by real paid Game/Room commands and restore checks.
//! Fixture movement is named explicitly; these are engine regressions, not UI QA.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, deck, model::*, room::*, rules::*};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
static INVALID_SEQ: AtomicUsize = AtomicUsize::new(0);

fn held(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn city(g: &mut Game, region: usize) {
    g.regions[region].card = g.make_card("DQJC112", 0);
}
fn move_fixture(g: &mut Game, id: &str, region: usize) {
    let (_, c) = g.remove_board(id).unwrap();
    g.regions[region].cards.push(c);
}
fn trigger_fixture(g: &mut Game, source: &str, mode: Option<&str>, target: Option<&str>) {
    let (_, c) = g.board(source).unwrap();
    let (actor, def) = (c.controller, c.definition.clone());
    g.enter_triggers(actor, &def, source, false);
    g.drive().unwrap();
    checkpoint(g);
    if let Some(mode) = mode {
        choose(g, vec![mode.into()]);
    }
    if let Some(target) = target {
        choose(g, vec![target.into()]);
    }
}
fn deploy(g: &mut Game, actor: usize, def: &str) -> String {
    city(g, 2);
    let id = held(g, def, actor);
    let cost = catalog::card(def).cost;
    fund(g, actor, "JC059", cost as usize);
    apply(
        g,
        actor,
        Action {
            card_id: Some(id.clone()),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    assert_eq!(g.resources(actor), 0);
    assert!(g.board(&id).is_none());
    pass_top(g);
    g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == def && c.controller == actor)
        .unwrap()
        .id
        .clone()
}
fn fixture(kind: &str, g: &Game) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("GRAY_EXPANSION_FIXTURE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let room = envelope(g);
        std::fs::write(format!("{dir}/{kind}.json"), serde_json::to_vec(&serde_json::json!({
            "scope":"offline-gray-expansion-native-fixture", "state":serde_json::to_string(&room).unwrap(),
            "views":(0..4).map(|seat| room.view(seat,0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
}
fn invalid_state(g: &Game, broken: Game) {
    assert!(Game::from_persisted(&serde_json::to_string(&broken).unwrap()).is_err());
    let mut room = envelope(g);
    room.game = broken;
    let state = serde_json::to_string(&room).unwrap();
    assert!(RoomEnvelope::from_persisted(&state).is_err());
    if let Ok(dir) = std::env::var("GRAY_EXPANSION_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let n = INVALID_SEQ.fetch_add(1, Ordering::Relaxed);
        std::fs::write(
            format!("{dir}/invalid-gray-{n:04}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn gray_expansion_complete_original_fields_and_exact_definitions() {
    for (id, name, cost, loyalty, defense, subtypes, permanent, temporary, keywords) in [
        (
            "JZ44",
            "洛杉矶巡警",
            3,
            1,
            1,
            vec!["人类", "警察"],
            Icons {
                combat: 1,
                influence: 1,
                ..Default::default()
            },
            Icons {
                combat: 1,
                ..Default::default()
            },
            vec!["公开", "护卫1"],
        ),
        (
            "JZ45",
            "警用直升机",
            5,
            3,
            3,
            vec!["人类", "警察", "载具"],
            Icons {
                investigation: 1,
                combat: 1,
                influence: 1,
            },
            Icons::default(),
            vec!["公开", "机动"],
        ),
    ] {
        let c = catalog::card(id);
        assert_eq!(
            (&*c.name, &*c.kind, &*c.color, c.cost, c.defense),
            (name, "character", "灰", cost, Some(defense))
        );
        assert_eq!(c.loyalty, vec!["灰色"; loyalty]);
        assert_eq!(c.subtypes, subtypes);
        assert_eq!(
            (c.permanent_icons, c.temporary_icons),
            (permanent, temporary)
        );
        assert_eq!(c.keywords, keywords);
        assert!(!c.unique && c.magic.is_empty());
        assert_eq!(c.magic_icon, MagicIcon::None);
        assert_eq!(deck::copy_limit(c), Some(3));
        let d = definition(id);
        assert!(d.traits.public && d.traits.city_play_only);
        assert_eq!(
            serde_json::to_value(d).unwrap(),
            serde_json::to_value(crate::gray_expansion::definition(id).unwrap()).unwrap()
        );
        for variant in 0..7 {
            let mut changed = d.clone();
            match variant {
                0 => changed.traits.public = false,
                1 => changed.traits.city_play_only = false,
                2 => changed.traits.guard += 1,
                3 => changed.abilities[0].timing = Timing::Standard,
                4 => changed.abilities[0].costs.push(Cost::Assets(1)),
                5 => changed.abilities[0].event = Some(Event::Reveal),
                _ => changed.abilities[0].response_policy = ResponsePolicy::Immediate,
            }
            let mut registry = definitions().clone();
            registry.insert(id.into(), changed);
            assert!(
                crate::rules::validate_definitions(&registry).is_err(),
                "{id} variant {variant}"
            );
        }
    }
    assert_eq!(definition("JZ44").traits.guard, 1);
    let criminal = &definition("JZ44").abilities[0].targets[0];
    assert_eq!(
        (
            criminal.kind,
            criminal.range,
            criminal.relation,
            criminal.subtype.as_deref()
        ),
        (
            EntityKind::Character,
            Range::SourceRegion,
            Relation::Any,
            Some("罪犯")
        )
    );
    let helicopter = definition("JZ45");
    assert_eq!(helicopter.abilities.len(), 2);
    assert_eq!(helicopter.abilities[0].modes.len(), 2);
    assert!(helicopter.abilities[0].targets.is_empty() && helicopter.abilities[0].ops.is_empty());
    assert_eq!(
        helicopter.abilities[1].event,
        Some(Event::ConfrontationStart)
    );
    assert!(helicopter.abilities[1].requires_ready_source);
    crate::rules::validate_definitions(definitions()).unwrap();
}

#[test]
fn gray_expansion_city_public_loyalty_cost_and_actor_rejections_are_atomic() {
    for actor in 0..4 {
        for (id, cost) in [("JZ44", 3), ("JZ45", 5)] {
            for reason in 0..4 {
                let mut g = game(actor);
                city(&mut g, 2);
                let source = held(&mut g, id, actor);
                if reason == 0 {
                    fund(&mut g, actor, "JC125", cost);
                } else {
                    fund(
                        &mut g,
                        actor,
                        "JC059",
                        if reason == 1 { cost - 1 } else { cost },
                    );
                }
                if reason == 2 {
                    g.regions[2].card = g.make_card("DQJC107", 0);
                }
                let a = Action {
                    card_id: Some(source.clone()),
                    region: Some(2),
                    ..Action::new("deploy")
                };
                reject(&mut g, if reason == 3 { (actor + 2) % 4 } else { actor }, a);
                reject(
                    &mut g,
                    actor,
                    Action {
                        card_id: Some(source.clone()),
                        region: Some(2),
                        ..Action::new("conceal")
                    },
                );
                assert!(g.players[actor].hand.iter().any(|c| c.id == source));
                assert!(g.players[actor].assets.iter().all(|c| !c.exhausted));
                if reason != 3 {
                    assert!(!g
                        .legal_actions(actor)
                        .iter()
                        .any(|a| a.action.kind == "deploy"
                            && a.action.card_id.as_deref() == Some(&source)
                            && a.action.region == Some(2)));
                }
            }
        }
    }
}

#[test]
fn gray_expansion_patrol_paid_entry_hides_any_controller_criminal_and_draws_actor_once() {
    for actor in 0..4 {
        for owner in 0..4 {
            let mut g = game(actor);
            let target = board(&mut g, "JZ48", owner, 2);
            g.board_mut(&target).unwrap().exhausted = true;
            g.board_mut(&target).unwrap().lock_markers = 2;
            let equipment = g.make_card("BQ022", owner);
            g.attachments.push(Attachment {
                card: equipment,
                host_id: target.clone(),
            });
            let expected_top = g.players[actor].deck[0].definition.clone();
            let patrol = deploy(&mut g, actor, "JZ44");
            assert!(g
                .pending
                .as_ref()
                .unwrap()
                .choice
                .options
                .iter()
                .any(|o| o.id == target));
            choose(&mut g, vec![target.clone()]);
            assert!(g.board(&target).is_some() && g.players[actor].hand.is_empty());
            pass_top(&mut g);
            assert!(g.board(&target).is_none());
            let hidden = g.regions[2]
                .cards
                .iter()
                .find(|c| c.definition == "JZ48" && c.owner == owner)
                .unwrap();
            assert!(hidden.face_down && hidden.exhausted);
            assert_ne!(hidden.id, target);
            assert_eq!(
                (
                    hidden.controller,
                    hidden.damage,
                    hidden.wounds,
                    hidden.lock_markers
                ),
                (owner, 0, 0, 0)
            );
            assert!(g.attachments.is_empty());
            assert!(g.players[owner]
                .hand
                .iter()
                .any(|c| c.definition == "BQ022"));
            // BQ022 prints host departure -> owner hand. Hide resets the host,
            // so its return is before the patrol's independent draw append.
            assert_eq!(g.players[actor].hand.len(), 1 + usize::from(owner == actor));
            assert_eq!(
                g.players[actor].hand.last().unwrap().definition,
                expected_top
            );
            assert!(g.board(&patrol).is_some() && g.stack.is_empty());
            fixture(&format!("patrol-seat{actor}-owner{owner}"), &g);
        }
    }
}

#[test]
fn gray_expansion_patrol_only_local_faceup_criminals_and_optional_decline_no_draw() {
    for actor in 0..4 {
        let mut g = game(actor);
        let patrol = board(&mut g, "JZ44", actor, 2);
        let legal = board(&mut g, "JZ48", (actor + 2) % 4, 2);
        let remote = board(&mut g, "JZ48", actor, 3);
        let hidden = board(&mut g, "JZ48", actor, 2);
        g.board_mut(&hidden).unwrap().face_down = true;
        let ordinary = board(&mut g, "JC125", actor, 2);
        let attached = g.make_card("BQ022", actor);
        let attachment = attached.id.clone();
        g.attachments.push(Attachment {
            card: attached,
            host_id: legal.clone(),
        });
        trigger_fixture(&mut g, &patrol, None, None);
        let p = g.pending.clone().unwrap();
        assert_eq!(
            p.choice
                .options
                .iter()
                .map(|o| o.id.as_str())
                .collect::<Vec<_>>(),
            [legal.as_str()]
        );
        for invalid in [
            remote,
            hidden,
            ordinary,
            attachment,
            patrol.clone(),
            "absent".into(),
        ] {
            reject(
                &mut g,
                actor,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec![invalid]),
                    ..Action::new("choose")
                },
            );
        }
        choose(&mut g, vec![]);
        assert!(g.stack.is_empty() && g.pending.is_none() && g.players[actor].hand.is_empty());
        assert!(!g.board(&legal).unwrap().1.face_down);
    }
}

#[test]
fn gray_expansion_patrol_response_target_failures_cancel_draw_and_source_departure_does_not() {
    for change in 0..6 {
        let mut g = game(0);
        let patrol = board(&mut g, "JZ44", 0, 2);
        let target = board(&mut g, "JZ48", 2, 2);
        trigger_fixture(&mut g, &patrol, None, Some(&target));
        match change {
            0 => move_fixture(&mut g, &target, 3),
            1 => g.return_hand(&target),
            2 => {
                let (r, c) = g.remove_board(&target).unwrap();
                let mut c = g.fresh(c);
                c.face_down = true;
                g.regions[r].cards.push(c);
            }
            3 => {
                g.regions[2].card = g.make_card("DQJC112", 0);
            }
            4 => g.return_hand(&patrol),
            _ => move_fixture(&mut g, &patrol, 3),
        }
        pass_top(&mut g);
        assert_eq!(
            g.players[0]
                .hand
                .iter()
                .filter(|c| c.definition != "JZ44")
                .count(),
            usize::from(change >= 4)
        );
        assert_eq!(
            g.regions
                .iter()
                .flat_map(|r| &r.cards)
                .any(|c| c.definition == "JZ48" && c.face_down),
            change == 2 || change >= 4
        );
        fixture(&format!("patrol-response-fixture-{change}"), &g);
    }
}

#[test]
fn gray_expansion_shield_cancels_whole_patrol_and_helicopter_effects() {
    for id in ["JZ44", "JZ45"] {
        for mode in ["lock", "destroy-locked"] {
            if id == "JZ44" && mode == "destroy-locked" {
                continue;
            }
            let mut g = game(0);
            let source = board(&mut g, id, 0, 2);
            let target = board(&mut g, "JZ48", 2, 2);
            let c = g.board_mut(&target).unwrap();
            c.shield = 1;
            c.lock_markers = 2;
            trigger_fixture(
                &mut g,
                &source,
                (id == "JZ45").then_some(mode),
                Some(&target),
            );
            pass_top(&mut g);
            let c = g.board(&target).unwrap().1;
            assert!(!c.face_down);
            assert_eq!((c.shield, c.lock_markers), (0, 2));
            assert!(g.players[0].hand.is_empty());
        }
    }
}

#[test]
fn gray_expansion_barrier_uses_current_controller_and_hidden_target_is_still_publicly_targetable() {
    for id in ["JZ44", "JZ45"] {
        for controller in [0, 1, 2] {
            for hidden in [false, true] {
                let mut g = game(0);
                let source = board(&mut g, id, 0, 2);
                let target = board(&mut g, "JZ48", 2, 2);
                let c = g.board_mut(&target).unwrap();
                c.controller = controller;
                c.face_down = hidden;
                c.lock_markers = 1;
                let barrier = g.make_card("JC073", controller);
                g.attachments.push(Attachment {
                    card: barrier,
                    host_id: target.clone(),
                });
                trigger_fixture(
                    &mut g,
                    &source,
                    (id == "JZ45").then_some("destroy-locked"),
                    None,
                );
                let allowed = if id == "JZ44" {
                    !hidden && controller < 2
                } else {
                    hidden || controller < 2
                };
                let offered = g
                    .pending
                    .as_ref()
                    .is_some_and(|p| p.choice.options.iter().any(|o| o.id == target));
                assert_eq!(
                    offered, allowed,
                    "{id} controller={controller} hidden={hidden}"
                );
                if allowed {
                    choose(&mut g, vec![target.clone()]);
                    pass_top(&mut g);
                } else if g.pending.is_some() {
                    choose(&mut g, vec![]);
                }
                if id == "JZ45" {
                    assert_eq!(g.board(&target).is_none(), allowed);
                } else {
                    assert_eq!(g.players[0].hand.len(), usize::from(allowed));
                }
            }
        }
    }
}

#[test]
fn gray_expansion_no_eligible_target_and_declined_mode_never_run_partial_program() {
    let mut g = game(0);
    let source = board(&mut g, "JZ44", 0, 2);
    g.enter_triggers(0, "JZ44", &source, false);
    g.drive().unwrap();
    assert!(g.pending.is_none() && g.stack.is_empty() && g.players[0].hand.is_empty());
    for decline in [false, true] {
        let mut g = game(0);
        let source = board(&mut g, "JZ45", 0, 2);
        let target = board(&mut g, "JC125", 2, 2);
        trigger_fixture(&mut g, &source, None, None);
        choose(
            &mut g,
            if decline {
                vec![]
            } else {
                vec!["destroy-locked".into()]
            },
        );
        assert!(g.pending.is_none() && g.stack.is_empty());
        assert_eq!(g.board(&target).unwrap().1.lock_markers, 0);
    }
}

#[test]
fn gray_expansion_patrol_guard_one_enforces_real_combat_allocation_when_exhausted() {
    let mut g = game(0);
    board(&mut g, "JC016", 0, 2);
    let patrol = board(&mut g, "JZ44", 2, 2);
    g.board_mut(&patrol).unwrap().exhausted = true;
    let other = board(&mut g, "LC24", 2, 2);
    g.begin_window(Window::Before(2, 1));
    g.close_window().unwrap();
    g.drive().unwrap();
    let p = g.pending.clone().unwrap();
    let amount = p.choice.amount.unwrap();
    assert!(amount >= 1);
    reject(
        &mut g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id.clone()),
            allocations: Some(BTreeMap::from([(other.clone(), amount)])),
            ..Action::new("choose")
        },
    );
    apply(
        &mut g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            allocations: Some(BTreeMap::from([
                (patrol.clone(), 1),
                (other.clone(), amount - 1),
            ])),
            ..Action::new("choose")
        },
    );
    assert!(g.board(&patrol).is_none());
    fixture("patrol-guard-combat", &g);
}

#[test]
fn gray_expansion_helicopter_paid_lock_modes_local_faceup_hidden_and_any_controller() {
    for actor in 0..4 {
        for owner in 0..4 {
            for hidden in [false, true] {
                let mut g = game(actor);
                let target = board(&mut g, "LC01", owner, 2);
                g.board_mut(&target).unwrap().face_down = hidden;
                g.board_mut(&target).unwrap().lock_markers = 2;
                let source = deploy(&mut g, actor, "JZ45");
                assert_eq!(
                    g.pending
                        .as_ref()
                        .unwrap()
                        .choice
                        .options
                        .iter()
                        .map(|o| o.id.as_str())
                        .collect::<Vec<_>>(),
                    ["lock", "destroy-locked"]
                );
                choose(&mut g, vec!["lock".into()]);
                choose(&mut g, vec![target.clone()]);
                pass_top(&mut g);
                let c = g.board(&target).unwrap().1;
                assert_eq!(c.lock_markers, 3);
                assert_eq!(c.face_down, hidden);
                assert!(g.board(&source).is_some());
                for viewer in 0..4 {
                    let v = g.card_view(c, viewer, Some(2), None);
                    assert_eq!(v.lock_markers, Some(3));
                    if hidden && viewer != owner {
                        assert!(v.card_id.is_none());
                    }
                }
            }
        }
    }
}

#[test]
fn gray_expansion_helicopter_mode_and_locked_target_legality_are_closed() {
    let mut g = game(0);
    let source = board(&mut g, "JZ45", 0, 2);
    let unlocked = board(&mut g, "JC125", 2, 2);
    let locked = board(&mut g, "LC01", 2, 2);
    g.board_mut(&locked).unwrap().lock_markers = 1;
    let hidden = board(&mut g, "JC125", 1, 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    g.board_mut(&hidden).unwrap().lock_markers = 2;
    let remote = board(&mut g, "JC125", 2, 3);
    g.board_mut(&remote).unwrap().lock_markers = 1;
    let c = g.make_card("BQ022", 2);
    let attachment = c.id.clone();
    g.attachments.push(Attachment {
        card: c,
        host_id: locked.clone(),
    });
    trigger_fixture(&mut g, &source, None, None);
    let p = g.pending.clone().unwrap();
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec!["damage".into()]),
            ..Action::new("choose")
        },
    );
    choose(&mut g, vec!["destroy-locked".into()]);
    let p = g.pending.clone().unwrap();
    let opts = p
        .choice
        .options
        .iter()
        .map(|o| o.id.clone())
        .collect::<Vec<_>>();
    assert_eq!(opts, vec![locked.clone(), hidden]);
    for invalid in [unlocked, remote, attachment, source, "absent".into()] {
        reject(
            &mut g,
            0,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec![invalid]),
                ..Action::new("choose")
            },
        );
    }
    choose(&mut g, vec![]);
    assert!(g.stack.is_empty());
    assert_eq!(g.board(&locked).unwrap().1.lock_markers, 1);
}

#[test]
fn gray_expansion_helicopter_destroy_clears_marker_owner_graveyard_and_frozen_death_controller() {
    for actor in 0..4 {
        for hidden in [false, true] {
            let owner = (actor + 1) % 4;
            let controller = (actor + 2) % 4;
            let mut g = game(actor);
            let source = board(&mut g, "JZ45", actor, 2);
            let target = board(&mut g, "JZ31", owner, 2);
            let c = g.board_mut(&target).unwrap();
            c.controller = controller;
            c.face_down = hidden;
            c.lock_markers = 2;
            trigger_fixture(&mut g, &source, Some("destroy-locked"), Some(&target));
            pass_top(&mut g);
            assert!(g.board(&target).is_none());
            let dead = g.players[owner]
                .graveyard
                .iter()
                .find(|c| c.definition == "JZ31")
                .unwrap();
            assert_eq!(
                (
                    dead.owner,
                    dead.controller,
                    dead.face_down,
                    dead.lock_markers
                ),
                (owner, owner, false, 0)
            );
            if hidden {
                assert!(g.pending.is_none());
            } else {
                assert_eq!(g.pending.as_ref().unwrap().seat, controller);
                choose(&mut g, vec!["accept".into()]);
                pass_top(&mut g);
                let mut marks = [0, 0];
                marks[g.team(controller)] = 1;
                assert_eq!(g.regions[2].influence, marks);
            }
            fixture(
                &format!("helicopter-destroy-seat{actor}-hidden{hidden}"),
                &g,
            );
        }
    }
}

#[test]
fn gray_expansion_helicopter_lock_self_and_destroy_locked_self_are_legal() {
    for mode in ["lock", "destroy-locked"] {
        let mut g = game(0);
        let source = board(&mut g, "JZ45", 0, 2);
        if mode == "destroy-locked" {
            g.board_mut(&source).unwrap().lock_markers = 1;
        }
        trigger_fixture(&mut g, &source, Some(mode), Some(&source));
        pass_top(&mut g);
        if mode == "lock" {
            assert_eq!(g.board(&source).unwrap().1.lock_markers, 1);
        } else {
            assert!(g.board(&source).is_none());
            assert!(g.players[0]
                .graveyard
                .iter()
                .any(|c| c.definition == "JZ45"));
        }
    }
}

#[test]
fn gray_expansion_helicopter_resolution_rechecks_lock_target_and_original_region() {
    for mode in ["lock", "destroy-locked"] {
        for change in 0..7 {
            let mut g = game(0);
            let source = board(&mut g, "JZ45", 0, 2);
            let target = board(&mut g, "LC01", 2, 2);
            g.board_mut(&target).unwrap().lock_markers = 2;
            trigger_fixture(&mut g, &source, Some(mode), Some(&target));
            match change {
                0 => g.board_mut(&target).unwrap().lock_markers = 0,
                1 => move_fixture(&mut g, &target, 3),
                2 => g.return_hand(&target),
                3 => {
                    g.regions[2].card = g.make_card("DQJC112", 0);
                }
                4 => g.return_hand(&source),
                5 => move_fixture(&mut g, &source, 3),
                _ => {
                    g.board_mut(&target).unwrap().face_down = true;
                } // same fixture instance remains eligible in both modes
            }
            pass_top(&mut g);
            if mode == "lock" {
                if change == 2 {
                    assert!(g.board(&target).is_none());
                } else {
                    assert_eq!(
                        g.board(&target).unwrap().1.lock_markers,
                        if change == 0 {
                            1
                        } else if [4, 5, 6].contains(&change) {
                            3
                        } else {
                            2
                        }
                    );
                }
            } else {
                assert_eq!(g.board(&target).is_none(), [2, 4, 5, 6].contains(&change));
            }
            fixture(&format!("helicopter-response-fixture-{mode}-{change}"), &g);
        }
    }
}

#[test]
fn gray_expansion_helicopter_natural_mobility_ready_adjacent_preserves_identity_and_no_entry_retrigger(
) {
    for actor in 0..4 {
        let mut g = game(actor);
        let source = board(&mut g, "JZ45", actor, 2);
        let target = board(&mut g, "JC125", (actor + 2) % 4, 3);
        let c = g.board_mut(&source).unwrap();
        c.damage = 1;
        c.lock_markers = 2;
        // Production end of the second team's action emits mobility naturally;
        // this also exercises the legacy snapshot with no region-instance field.
        g.begin_window(Window::Action(1 - g.first_team));
        g.close_window().unwrap();
        g.drive().unwrap();
        checkpoint(&g);
        let p = g.pending.clone().unwrap();
        assert_eq!(p.seat, actor);
        assert_eq!(
            p.choice
                .options
                .iter()
                .map(|o| o.id.as_str())
                .collect::<Vec<_>>(),
            ["region:1", "region:3"]
        );
        reject(
            &mut g,
            actor,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec!["region:4".into()]),
                ..Action::new("choose")
            },
        );
        choose(&mut g, vec!["region:3".into()]);
        pass_top(&mut g);
        let (r, c) = g.board(&source).unwrap();
        assert_eq!((r, c.damage, c.lock_markers, c.exhausted), (3, 1, 2, false));
        assert_eq!(g.board(&target).unwrap().1.lock_markers, 0);
        assert!(g.pending.is_none() && g.stack.is_empty());
    }
    let mut g = game(0);
    let source = board(&mut g, "JZ45", 0, 2);
    g.board_mut(&source).unwrap().exhausted = true;
    g.begin_window(Window::Action(1 - g.first_team));
    g.close_window().unwrap();
    g.drive().unwrap();
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.board(&source).unwrap().0, 2);
}

#[test]
fn gray_expansion_helicopter_lock_feeds_real_paid_police_dog_chase() {
    let mut g = game(0);
    let source = board(&mut g, "JZ45", 0, 2);
    let target = board(&mut g, "JC125", 2, 2);
    g.board_mut(&target).unwrap().face_down = true;
    trigger_fixture(&mut g, &source, Some("lock"), Some(&target));
    pass_top(&mut g);
    let dog = board(&mut g, "JC069", 0, 0);
    fund(&mut g, 0, "JC125", 2);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(dog.clone()),
            target_id: Some(target.clone()),
            ability_id: Some("chase-locked".into()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    assert_eq!(g.board(&dog).unwrap().0, 2);
    assert_eq!(
        g.current_icons(g.board(&dog).unwrap().1, 2).investigation,
        1
    );
    assert_eq!(g.board(&target).unwrap().1.lock_markers, 1);
    assert_eq!(g.resources(0), 0);
    fixture("helicopter-dog-lock-loop", &g);
}

#[test]
fn gray_expansion_restore_rejects_modified_program_targets_identity_and_atomic_cursor() {
    for id in ["JZ44", "JZ45"] {
        for mode in ["lock", "destroy-locked"] {
            if id == "JZ44" && mode == "destroy-locked" {
                continue;
            }
            let mut g = game(0);
            let source = board(&mut g, id, 0, 2);
            let target = board(&mut g, "JZ48", 2, 2);
            g.board_mut(&target).unwrap().lock_markers = 1;
            trigger_fixture(
                &mut g,
                &source,
                (id == "JZ45").then_some(mode),
                Some(&target),
            );
            for change in 0..19 {
                let mut broken = g.clone();
                let item = broken.stack.last_mut().unwrap();
                let f = item.frame.as_mut().unwrap();
                match change {
                    0 => f.source.card.definition = "JC125".into(),
                    1 => f.actor = 2,
                    2 => f.source.card.face_down = true,
                    3 => f.source.source_region_instance = None,
                    4 => f.targets[0].spec.range = Range::Anywhere,
                    5 => f.targets[0].public.instance_id = "other".into(),
                    6 => f.steps[0].context = 2,
                    7 => f.steps.push(f.steps[0].clone()),
                    8 => f.guard = GuardState::Accepted,
                    9 => f.cursor = 1,
                    10 => f.already_paid.push(PaidCost::Assets(vec![])),
                    11 => f.steps[0].op = Op::ForEachLivingPlayer(vec![f.steps[0].op.clone()]),
                    12 => f.ability_key = "mobility".into(),
                    13 => f.chosen_region = Some(2),
                    14 => item.card = Some(g.board(&source).unwrap().1.clone()),
                    15 => item.deploy_region = Some(2),
                    16 => item.reveal = true,
                    17 => item.target = Some("unrelated-target".into()),
                    _ => item.controller = 2,
                }
                invalid_state(&g, broken);
            }
        }
    }
}

#[test]
fn gray_expansion_restore_rejects_changed_original_and_selected_mode_declarations() {
    for mode in [None, Some("lock"), Some("destroy-locked")] {
        let mut g = game(0);
        let source = board(&mut g, "JZ45", 0, 2);
        let target = board(&mut g, "JC125", 2, 2);
        g.board_mut(&target).unwrap().lock_markers = 1;
        trigger_fixture(&mut g, &source, mode, None);
        for change in 0..6 {
            let mut broken = g.clone();
            let ChoiceResolution::Declare { declaration: d, .. } =
                &mut broken.pending.as_mut().unwrap().resolution
            else {
                panic!()
            };
            match change {
                0 => d.source.card.definition = "JC125".into(),
                1 => d.actor = 2,
                2 => d.source.source_region_instance = None,
                3 => d.ability.costs.push(Cost::Assets(1)),
                4 => d.ability.ops.push(Op::Draw {
                    player: PlayerRef::Actor,
                    count: 1,
                    end: DeckEnd::Top,
                }),
                _ => d.source.play_source = Some(PlaySource::Hand),
            }
            invalid_state(&g, broken);
        }
    }
}

#[test]
fn gray_expansion_declaration_metadata_stage_and_enemy_choose_are_closed() {
    for (id, mode, mobility) in [
        ("JZ44", None, false),
        ("JZ45", None, false),
        ("JZ45", Some("lock"), false),
        ("JZ45", Some("destroy-locked"), false),
        ("JZ45", None, true),
    ] {
        let mut g = game(0);
        let source = board(&mut g, id, 0, 2);
        let target = board(&mut g, "JZ48", 2, 2);
        g.board_mut(&target).unwrap().lock_markers = 1;
        if mobility {
            g.begin_window(Window::Action(1 - g.first_team));
            g.close_window().unwrap();
            g.drive().unwrap();
            checkpoint(&g);
        } else {
            trigger_fixture(&mut g, &source, mode, None);
        }
        let valid = g.pending.clone().unwrap();
        let selected = vec![valid.choice.options[0].id.clone()];
        reject(
            &mut g,
            2,
            Action {
                choice_id: Some(valid.choice.id.clone()),
                selected: Some(selected.clone()),
                ..Action::new("choose")
            },
        );
        for change in 0..15 {
            let mut broken = g.clone();
            let p = broken.pending.as_mut().unwrap();
            match change {
                0 => p.seat = 2,
                1 => p.choice.player_id = "p2".into(),
                2 => {
                    p.seat = 2;
                    p.choice.player_id = "p2".into();
                }
                3 => p.choice.kind = "damage".into(),
                4 => p.choice.min = Some(1),
                5 => p.choice.max = Some(2),
                6 => p.choice.allow_decline = Some(false),
                7 => p.choice.amount = Some(1),
                8 => p.choice.options.clear(),
                9 => p.choice.options[0].id = "forged-option".into(),
                10 => p.choice.options[0].label = "forged-label".into(),
                11 => {
                    let ChoiceResolution::Declare { stage, .. } = &mut p.resolution else {
                        panic!()
                    };
                    *stage = DeclareChoice::Accept;
                }
                12 => p.choice.id.clear(),
                13 => p.choice.title = "forged-title".into(),
                _ => p.choice.description = "forged-description".into(),
            }
            invalid_state(&g, broken.clone());
            let before = serde_json::to_string(&broken).unwrap();
            // Real Game::choose must reject even if a corrupt in-memory pending
            // item nominates the enemy seat; no effect may run as frozen actor.
            assert!(broken
                .apply(
                    2,
                    Action {
                        choice_id: Some(valid.choice.id.clone()),
                        selected: Some(selected.clone()),
                        ..Action::new("choose")
                    }
                )
                .is_err());
            assert_eq!(serde_json::to_string(&broken).unwrap(), before);
            let mut room = envelope(&g);
            room.game = broken;
            let saved = serde_json::to_string(&room).unwrap();
            let cmd = RoomCommand {
                command_id: format!("gray-forged-enemy-{id}-{change}"),
                expected_version: room.revision,
                action: SessionAction::Game {
                    action: Action {
                        choice_id: Some(valid.choice.id.clone()),
                        selected: Some(selected.clone()),
                        ..Action::new("choose")
                    },
                },
            };
            let result = room.transition(2, Some(cmd), 0).unwrap();
            assert!(result.error_code.is_some());
            assert_eq!(result.state, saved);
        }
    }
}

#[test]
fn gray_expansion_existing_granted_rewards_keep_valid_accept_stage_and_guard_metadata() {
    for id in ["JZ44", "JZ45"] {
        for glory in [false, true] {
            let mut g = game(0);
            let source = board(&mut g, id, 0, 2);
            g.turn_attribute_modifiers.push(TurnAttributeModifier {
                target_instance: source.clone(),
                defense_bonus: 1,
                kill_bonus: 0,
                grants_retreat: false,
                printed_defense_override: None,
                ordinary_icons: Icons::default(),
                grants_renown: !glory,
                prevents_damage: false,
                expires_turn: g.turn,
            });
            if glory {
                let c = g.make_card("JC089", 0);
                g.attachments.push(Attachment {
                    card: c,
                    host_id: source,
                });
                let d = g.jc089_combat_glory(0, 2).unwrap();
                g.effects.push_back(Effect::Declare { declaration: d });
            } else {
                g.begin_window(Window::After(2, 2));
                let region_instance = g.regions[2].card.id.clone();
                g.declare_region_renown(2, &region_instance);
            }
            g.drive().unwrap();
            checkpoint(&g);
            assert_eq!(g.pending.as_ref().unwrap().choice.options[0].id, "accept");
            let mut broken = g.clone();
            let p = broken.pending.as_mut().unwrap();
            let ChoiceResolution::Declare { stage, .. } = &mut p.resolution else {
                panic!()
            };
            *stage = DeclareChoice::Mode;
            invalid_state(&g, broken);
            choose(&mut g, vec!["accept".into()]);
            pass_top(&mut g);
            assert_eq!(g.regions[2].influence, [1, 0]);
        }
    }
}

#[test]
fn gray_expansion_restore_rejects_bad_paid_deploy_and_fake_frame_choice() {
    for id in ["JZ44", "JZ45"] {
        let mut g = game(0);
        city(&mut g, 2);
        let source = held(&mut g, id, 0);
        fund(&mut g, 0, "JC059", catalog::card(id).cost as usize);
        apply(
            &mut g,
            0,
            Action {
                card_id: Some(source),
                region: Some(2),
                ..Action::new("deploy")
            },
        );
        for change in 0..14 {
            let mut broken = g.clone();
            let item = broken.stack.last_mut().unwrap();
            let f = item.frame.as_mut().unwrap();
            match change {
                0 => f.source.play_source = None,
                1 => f.source.card.face_down = true,
                2 => f.already_paid.clear(),
                3 => f.steps.push(Step {
                    context: 0,
                    op: Op::JZ45LockLocalTarget,
                }),
                4 => f.chosen_region = Some(3),
                5 => item.deploy_region = Some(3),
                6 => item.card.as_mut().unwrap().owner = 2,
                7 => item.id = "unrelated-stack-item".into(),
                8 => item.card.as_mut().unwrap().face_down = true,
                9 => item.card.as_mut().unwrap().id.clear(),
                10 => item.card.as_mut().unwrap().controller = 2,
                11 => item.target = Some("unrelated-target".into()),
                12 => item.card.as_mut().unwrap().id = f.source.card.id.clone(),
                _ => item.frame = None,
            }
            invalid_state(&g, broken);
        }
        pass_top(&mut g);
        if g.pending.is_some() {
            choose(&mut g, vec![]);
        }
        let source = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == id)
            .unwrap()
            .id
            .clone();
        let target = board(&mut g, "JZ48", 2, 2);
        g.board_mut(&target).unwrap().lock_markers = 1;
        trigger_fixture(
            &mut g,
            &source,
            (id == "JZ45").then_some("lock"),
            Some(&target),
        );
        let mut broken = g.clone();
        let f = broken.stack.pop().unwrap().frame.unwrap();
        let options = vec![ChoiceOption {
            id: "fake".into(),
            label: "fake".into(),
            card: None,
        }];
        broken.choice(
            0,
            "forecast",
            "fake frame suspension".into(),
            options,
            1,
            1,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::Forecast { seat: 0 },
            },
        );
        invalid_state(&g, broken);
    }
}

fn command(
    room: &mut RoomEnvelope,
    steps: &mut Vec<serde_json::Value>,
    seat: usize,
    action: SessionAction,
) {
    let before = room.clone();
    let state = serde_json::to_string(&before).unwrap();
    let cmd = RoomCommand {
        command_id: format!("gray-expansion-chain-{}", steps.len()),
        expected_version: room.revision,
        action,
    };
    let expected = room.transition(seat, Some(cmd.clone()), 0).unwrap();
    assert!(
        expected.error_code.is_none(),
        "{:?}",
        expected.error_message
    );
    assert_eq!(
        serde_json::to_string(&before.replay_events(&expected.journal).unwrap()).unwrap(),
        expected.state
    );
    *room = RoomEnvelope::from_persisted(&expected.state).unwrap();
    // BeginResponse is deliberately idempotent for an already composing intent.
    // Game / SubmitResponse have strict expected_version checks, so re-applying
    // their accepted journal to the result must reject without paying twice.
    if matches!(
        &cmd.action,
        SessionAction::Game { .. } | SessionAction::SubmitResponse { .. }
    ) {
        assert!(room.replay_events(&expected.journal).is_err());
    }
    let views = (0..4)
        .map(|s| serde_json::to_value(room.view(s, 0)).unwrap())
        .collect::<Vec<_>>();
    steps.push(serde_json::json!({"state":state,"seat":seat,"command":cmd,"serverNowMs":"0","expected":expected,"views":views}));
}
fn room_pass(room: &mut RoomEnvelope, steps: &mut Vec<serde_json::Value>) {
    let w = room.pacing.window.clone().unwrap();
    let seat = *w
        .members
        .iter()
        .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
        .unwrap()
        .0;
    command(
        room,
        steps,
        seat,
        SessionAction::PassResponse { window_id: w.id },
    );
}

#[test]
fn gray_expansion_actual_room_paid_deploy_response_rescue_restore_replay_and_stale_rejection() {
    let mut g = game(0);
    city(&mut g, 2);
    let target = board(&mut g, "JZ48", 2, 2);
    let rescue = board(&mut g, "JC075", 2, 4);
    fund(&mut g, 2, "JC125", 2);
    fund(&mut g, 0, "JC059", 3);
    let patrol = held(&mut g, "JZ44", 0);
    let mut room = envelope(&g);
    let initial = serde_json::to_string(&room).unwrap();
    let mut steps = vec![];
    command(
        &mut room,
        &mut steps,
        0,
        SessionAction::Game {
            action: Action {
                card_id: Some(patrol),
                region: Some(2),
                ..Action::new("deploy")
            },
        },
    );
    for _ in 0..16 {
        if room.pending.is_some() {
            break;
        }
        room_pass(&mut room, &mut steps);
    }
    let p = room.pending.clone().unwrap();
    command(
        &mut room,
        &mut steps,
        0,
        SessionAction::Game {
            action: Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![target.clone()]),
                ..Action::new("choose")
            },
        },
    );
    for _ in 0..8 {
        if room.pacing.window.as_ref().unwrap().holder_team == room.team(2) {
            break;
        }
        room_pass(&mut room, &mut steps);
    }
    let w = room.pacing.window.clone().unwrap();
    let intent = "gray-expansion-rescue".to_string();
    command(
        &mut room,
        &mut steps,
        2,
        SessionAction::BeginResponse {
            window_id: w.id.clone(),
            intent_id: intent.clone(),
        },
    );
    command(
        &mut room,
        &mut steps,
        2,
        SessionAction::SubmitResponse {
            window_id: w.id.clone(),
            intent_id: intent.clone(),
            action: Action {
                card_id: Some(rescue.clone()),
                ability_id: Some("rescue".into()),
                target_id: Some(target.clone()),
                ..Action::new("activate")
            },
        },
    );
    for _ in 0..32 {
        if room.stack.is_empty() {
            break;
        }
        room_pass(&mut room, &mut steps);
    }
    assert!(room.stack.is_empty() && room.pending.is_none());
    assert!(room.board(&target).is_none());
    assert!(room.players[2]
        .hand
        .iter()
        .any(|c| c.definition == "JZ48" && c.id != target));
    assert!(room.players[0].hand.is_empty());
    assert_eq!((room.resources(0), room.resources(2)), (0, 0));
    assert!(room.board(&rescue).unwrap().1.exhausted);
    let saved = serde_json::to_string(&room).unwrap();
    let stale = RoomCommand {
        command_id: "gray-expansion-expired-response".into(),
        expected_version: room.revision,
        action: SessionAction::SubmitResponse {
            window_id: w.id,
            intent_id: intent,
            action: Action::new("activate"),
        },
    };
    let rejected = room.transition(2, Some(stale), 0).unwrap();
    assert!(rejected.error_code.is_some());
    assert_eq!(rejected.state, saved);
    if let Ok(dir) = std::env::var("GRAY_EXPANSION_CHAIN_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/paid-patrol-rescue-chain.json"),serde_json::to_vec(&serde_json::json!({
            "scope":"offline-continuous-real-room-commands", "initialState":initial,"steps":steps,
            "finalState":saved,"targetOldInstance":target,"staleRejected":rejected,
        })).unwrap()).unwrap();
    }
    fixture("paid-patrol-rescue-final", &room.game);
}
