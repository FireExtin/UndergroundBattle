//! Explicit offline initial layouts, followed by paid production commands.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{
    catalog, deck,
    model::*,
    room::{Decision, RoomCommand, RoomEnvelope, SessionAction},
    rules::*,
};

fn held(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn play(g: &Game, actor: usize, source: &str, target: &str, mode: Option<&str>) -> Action {
    g.legal_actions(actor)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(source)
                && a.action.target_id.as_deref() == Some(target)
                && (mode.is_none() || a.action.option.as_deref() == mode)
        })
        .unwrap()
        .action
}
fn save_fixture(kind: &str, g: &Game) {
    save_room_fixture(kind, &envelope(g));
}
fn save_room_fixture(kind: &str, r: &RoomEnvelope) {
    if let Ok(dir) = std::env::var("JZ55_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/{kind}.json"),
            serde_json::to_vec(&serde_json::json!({
                "kind":kind,"state":serde_json::to_string(&r).unwrap(),
                "views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()
            }))
            .unwrap(),
        )
        .unwrap();
    }
}
fn command(
    room: &mut RoomEnvelope,
    steps: &mut Vec<serde_json::Value>,
    seat: usize,
    action: SessionAction,
) {
    let state = serde_json::to_string(room).unwrap();
    let cmd = RoomCommand {
        command_id: format!("jz55-chain-{}", steps.len()),
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
        serde_json::to_string(&room.replay_events(&expected.journal).unwrap()).unwrap(),
        expected.state
    );
    *room = RoomEnvelope::from_persisted(&expected.state).unwrap();
    let views: Vec<_> = (0..4)
        .map(|s| serde_json::to_value(room.view(s, 0)).unwrap())
        .collect();
    steps.push(serde_json::json!({"state":state,"seat":seat,"command":cmd,"serverNowMs":"0","expected":expected,"views":views}));
}

#[test]
fn jz55_original_and_exact_finite_immediate_destroy_definition() {
    let c = catalog::card("JZ55");
    assert_eq!(
        (&*c.name, &*c.kind, &*c.color, c.cost),
        ("传奇落幕", "spell", "黑", 2)
    );
    assert_eq!(c.loyalty, ["黑色"]);
    assert_eq!(c.subtypes, ["命运"]);
    assert!(c.magic.is_empty());
    assert_eq!(c.magic_icon, MagicIcon::None);
    assert_eq!(c.permanent_icons, Icons::default());
    assert_eq!(c.temporary_icons, Icons::default());
    assert!(!c.unique);
    assert!(c.keywords.is_empty());
    assert_eq!(deck::copy_limit(c), Some(3));
    assert_eq!(c.text, "传奇落幕不能被响应。\n快速行动：消灭目标独有角色。");
    let d = definition("JZ55");
    assert_eq!(d.abilities.len(), 1);
    let a = &d.abilities[0];
    assert_eq!(a.timing, Timing::Fast);
    assert_eq!(a.response_policy, ResponsePolicy::Immediate);
    assert!(a.event.is_none() && a.costs.is_empty() && a.modes.is_empty());
    assert!(matches!(
        a.ops.as_slice(),
        [Op::Destroy(EntityRef::Target(0))]
    ));
    assert_eq!(a.targets.len(), 1);
    let t = &a.targets[0];
    assert_eq!(
        (t.zone, t.kind, t.relation, t.range, t.min, t.max),
        (
            Zone::Board,
            EntityKind::Character,
            Relation::Any,
            Range::Anywhere,
            1,
            1
        )
    );
    assert_eq!(
        serde_json::to_value(&t.predicate).unwrap(),
        serde_json::json!("JZ55UniqueCharacter")
    );
    assert_eq!(catalog::catalog().cards.len(), 102);
}

#[test]
fn jz55_only_face_up_unique_characters_are_paid_targets_and_illegal_commands_are_atomic() {
    for actor in 0..4 {
        let mut g = game(actor);
        fund(&mut g, actor, "JC084", 2);
        let spell = held(&mut g, "JZ55", actor);
        let mut valid = vec![];
        for owner in 0..4 {
            valid.push(board(&mut g, "LC23", owner, owner));
        }
        let ordinary = board(&mut g, "JC125", 2, 2);
        let hidden = board(&mut g, "LC01", 2, 2);
        g.board_mut(&hidden).unwrap().face_down = true;
        let c = g.make_card("BQ022", 2);
        let attachment = c.id.clone();
        g.attachments.push(Attachment {
            card: c,
            host_id: valid[2].clone(),
        });
        let actions: Vec<_> = g
            .legal_actions(actor)
            .into_iter()
            .filter(|a| a.action.kind == "play" && a.action.card_id.as_deref() == Some(&spell))
            .collect();
        assert_eq!(actions.len(), 4);
        assert!(valid.iter().all(|id| actions
            .iter()
            .any(|a| a.action.target_id.as_deref() == Some(id))));
        for invalid in [ordinary, hidden, attachment, "missing-instance".into()] {
            reject(
                &mut g,
                actor,
                Action {
                    card_id: Some(spell.clone()),
                    target_id: Some(invalid),
                    ..Action::new("play")
                },
            );
        }
        save_fixture(&format!("targets-seat{actor}"), &g);
        let target = valid[2].clone();
        let a = play(&g, actor, &spell, &target, None);
        apply(&mut g, actor, a);
        assert!(g.board(&target).is_none());
        assert!(valid
            .iter()
            .filter(|id| **id != target)
            .all(|id| g.board(id).is_some()));
        assert!(g.stack.is_empty() && g.pending.is_none());
        assert_eq!(g.resources(actor), 0);
        assert_eq!(
            g.players[actor]
                .graveyard
                .iter()
                .filter(|c| c.definition == "JZ55")
                .count(),
            1
        );
        assert_eq!(
            g.players[2]
                .graveyard
                .iter()
                .filter(|c| c.definition == "LC23")
                .count(),
            1
        );
        assert!(envelope(&g).pacing.window.is_none());
        save_fixture(&format!("ordinary-seat{actor}"), &g);
    }
}

#[test]
fn jz55_real_begin_submit_destroys_immediately_without_a_new_responsive_stack_object() {
    let mut g = game(0);
    fund(&mut g, 0, "JC063", 2);
    fund(&mut g, 2, "JC084", 2);
    let target = board(&mut g, "LC23", 2, 2);
    let hide = held(&mut g, "JC063", 0);
    let spell = held(&mut g, "JZ55", 2);
    let mut room = envelope(&g);
    let initial = serde_json::to_string(&room).unwrap();
    let mut steps = vec![];
    let a = play(&room.game, 0, &hide, &target, Some("hide"));
    command(&mut room, &mut steps, 0, SessionAction::Game { action: a });
    assert_eq!(room.stack.len(), 1);
    let frozen = serde_json::to_value(room.stack[0].frame.as_ref().unwrap()).unwrap();
    let underlying = room.stack[0].id.clone();
    for seat in [0, 1] {
        let window_id = room.pacing.window.as_ref().unwrap().id.clone();
        command(
            &mut room,
            &mut steps,
            seat,
            SessionAction::PassResponse { window_id },
        );
    }
    let window = room.pacing.window.clone().unwrap();
    assert_eq!(window.holder_team, 1);
    command(
        &mut room,
        &mut steps,
        2,
        SessionAction::BeginResponse {
            window_id: window.id.clone(),
            intent_id: "jz55-real-response".into(),
        },
    );
    assert!(matches!(
        room.pacing.window.as_ref().unwrap().members[&2],
        Decision::Composing { .. }
    ));
    save_room_fixture("real-composing", &room);
    let a = play(&room.game, 2, &spell, &target, None);
    command(
        &mut room,
        &mut steps,
        2,
        SessionAction::SubmitResponse {
            window_id: window.id.clone(),
            intent_id: "jz55-real-response".into(),
            action: a,
        },
    );
    assert!(room.board(&target).is_none());
    assert_eq!(room.resources(2), 0);
    assert_eq!(room.stack.len(), 1);
    assert_eq!(room.stack[0].id, underlying);
    assert_eq!(
        serde_json::to_value(room.stack[0].frame.as_ref().unwrap()).unwrap(),
        frozen
    );
    assert_eq!(
        room.pacing.window.as_ref().unwrap().stack_top_id,
        underlying
    );
    assert_ne!(room.pacing.window.as_ref().unwrap().id, window.id);
    assert_eq!(
        room.players[2]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JZ55")
            .count(),
        1
    );
    let before = serde_json::to_string(&room).unwrap();
    let stale = RoomCommand {
        command_id: "stale-jz55-response".into(),
        expected_version: room.revision,
        action: SessionAction::SubmitResponse {
            window_id: window.id,
            intent_id: "jz55-real-response".into(),
            action: Action::new("play"),
        },
    };
    let rejected = room.transition(2, Some(stale), 0).unwrap();
    assert_eq!(rejected.error_code.as_deref(), Some("window_expired"));
    assert_eq!(rejected.state, before);
    save_room_fixture("immediate-response-complete", &room);
    for _ in 0..16 {
        if room.stack.is_empty() {
            break;
        }
        let w = room.pacing.window.clone().unwrap();
        let seat = *w
            .members
            .iter()
            .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
            .unwrap()
            .0;
        command(
            &mut room,
            &mut steps,
            seat,
            SessionAction::PassResponse { window_id: w.id },
        );
    }
    assert!(room.stack.is_empty());
    assert!(room
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .all(|c| c.definition != "LC23"));
    if let Ok(dir) = std::env::var("JZ55_RESPONSE_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/response-chain.json"),serde_json::to_vec(&serde_json::json!({"initialState":initial,"steps":steps,"finalState":serde_json::to_string(&room).unwrap(),"underlyingStackInstance":underlying,"targetInstance":target})).unwrap()).unwrap();
    }
    checkpoint(&room.game);
}

#[test]
fn jz55_black_loyalty_and_failed_cost_target_timing_checks_never_pay() {
    for variant in 0..4 {
        let mut g = game(0);
        let target = board(&mut g, "LC23", 2, 2);
        let spell = held(&mut g, "JZ55", 0);
        match variant {
            0 => fund(&mut g, 0, "JC125", 2), // sufficient assets, no black loyalty.
            1 => fund(&mut g, 0, "JC084", 1), // black loyalty, insufficient assets.
            2 => fund(&mut g, 0, "JC084", 2),
            _ => fund(&mut g, 0, "JC084", 2),
        }
        let a = Action {
            card_id: Some(spell.clone()),
            target_id: (variant != 2).then_some(target),
            ..Action::new("play")
        };
        reject(&mut g, if variant == 3 { 2 } else { 0 }, a);
        assert!(g.players[0].hand.iter().any(|c| c.id == spell));
        assert!(g.players[0].assets.iter().all(|c| !c.exhausted));
    }
}

#[test]
fn jz55_barrier_uses_current_controller_and_shield_remains_a_mandatory_replacement() {
    for (controller, shield, allowed, destroyed) in [
        (2, 0, false, false),
        (1, 0, true, true),
        (0, 0, true, true),
        (2, 1, true, false),
    ] {
        let mut g = game(0);
        fund(&mut g, 0, "JC084", 2);
        let spell = held(&mut g, "JZ55", 0);
        let target = board(&mut g, "LC23", 2, 2);
        g.board_mut(&target).unwrap().controller = controller;
        if shield == 0 {
            let c = g.make_card("JC073", controller);
            g.attachments.push(Attachment {
                card: c,
                host_id: target.clone(),
            });
            assert!(g.has_barrier(g.board(&target).unwrap().1));
        } else {
            g.board_mut(&target).unwrap().shield = shield;
        }
        let advertised = g.legal_actions(0).iter().any(|a| {
            a.action.card_id.as_deref() == Some(&spell)
                && a.action.target_id.as_deref() == Some(&target)
        });
        assert_eq!(advertised, allowed);
        let a = Action {
            card_id: Some(spell.clone()),
            target_id: Some(target.clone()),
            ..Action::new("play")
        };
        if !allowed {
            reject(&mut g, 0, a);
            assert_eq!(g.resources(0), 2);
            save_fixture("enemy-barrier-rejected", &g);
            continue;
        }
        apply(&mut g, 0, a);
        assert_eq!(g.board(&target).is_none(), destroyed);
        assert_eq!(g.resources(0), 0);
        assert!(g.stack.is_empty());
        assert_eq!(
            g.players[0]
                .graveyard
                .iter()
                .filter(|c| c.definition == "JZ55")
                .count(),
            1
        );
        if !destroyed {
            assert_eq!(g.board(&target).unwrap().1.shield, 0);
            save_fixture("shield-stopped-immediate", &g);
        } else {
            assert_eq!(
                g.players[2]
                    .graveyard
                    .iter()
                    .filter(|c| c.definition == "LC23")
                    .count(),
                1
            );
        }
    }
}

#[test]
fn jz55_borrowed_source_and_target_bury_to_owners_and_keep_other_instances() {
    let mut g = game(0);
    fund(&mut g, 0, "JC084", 2);
    let spell = held(&mut g, "JZ55", 2);
    let mut c = g.players[2].hand.pop().unwrap();
    c.controller = 0;
    g.players[0].hand.push(c);
    let target = board(&mut g, "LC23", 2, 2);
    g.board_mut(&target).unwrap().controller = 1;
    let same = board(&mut g, "LC23", 0, 0);
    let a = play(&g, 0, &spell, &target, None);
    apply(&mut g, 0, a);
    assert!(g.board(&target).is_none() && g.board(&same).is_some());
    assert_eq!(g.resources(0), 0);
    assert_eq!(
        g.players[2]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JZ55" || c.definition == "LC23")
            .count(),
        2
    );
    let dead = g.players[2]
        .graveyard
        .iter()
        .find(|c| c.definition == "LC23")
        .unwrap();
    assert_eq!((dead.owner, dead.controller), (2, 2));
    assert_ne!(dead.id, target);
    let buried = g.players[2]
        .graveyard
        .iter()
        .find(|c| c.definition == "JZ55")
        .unwrap();
    // Existing transaction Bury preserves its paid source's controller record;
    // placement still uses owner. Do not change unrelated graveyard semantics.
    assert_eq!((buried.owner, buried.controller), (2, 0));
    assert_ne!(buried.id, spell);
    assert!(g.players[0].graveyard.is_empty());
    checkpoint(&g);
    save_fixture("borrowed-owners", &g);
}

#[test]
fn jz55_printed_uniqueness_survives_current_control_and_subtype_changes() {
    for (id, unique) in [("LC23", true), ("JC084", false)] {
        let mut g = game(0);
        fund(&mut g, 0, "JC084", 2);
        let spell = held(&mut g, "JZ55", 0);
        let target = board(&mut g, id, 2, 2);
        // Explicit initial control/subtype modifier fixture; printed definition
        // is unchanged. The separate cascade test uses paid control commands.
        g.add_control(
            &target,
            0,
            ControlLifetime::TurnEnd { turn: g.turn },
            SubtypeChange::HumanToVampire,
        );
        let c = g.board(&target).unwrap().1;
        assert_eq!((c.owner, c.controller), (2, 0));
        assert!(g.current_subtypes(c).iter().any(|s| s == "吸血鬼"));
        assert!(!g.current_subtypes(c).iter().any(|s| s == "人类"));
        assert_eq!(catalog::card(id).unique, unique);
        let advertised = g.legal_actions(0).iter().any(|a| {
            a.action.card_id.as_deref() == Some(&spell)
                && a.action.target_id.as_deref() == Some(&target)
        });
        assert_eq!(advertised, unique);
        let a = Action {
            card_id: Some(spell),
            target_id: Some(target.clone()),
            ..Action::new("play")
        };
        save_fixture(
            if unique {
                "printed-unique-current-vampire"
            } else {
                "printed-nonunique-current-vampire"
            },
            &g,
        );
        if unique {
            apply(&mut g, 0, a);
            assert!(g.board(&target).is_none());
        } else {
            reject(&mut g, 0, a);
            assert_eq!(g.resources(0), 2);
        }
    }
}

#[test]
fn jz55_destroyed_control_source_causes_real_cascade_death_with_normal_respondable_trigger() {
    let mut g = game(0);
    let source = board(&mut g, "JZ27", 0, 2);
    g.board_mut(&source).unwrap().face_down = true;
    let line = board(&mut g, "JZ31", 2, 2);
    board(&mut g, "JC059", 0, 2);
    fund(&mut g, 0, "JZ27", 6);
    fund(&mut g, 0, "JC102", 2);
    fund(&mut g, 0, "JC084", 2);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(source.clone()),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut g);
    choose(&mut g, vec![line.clone()]);
    pass_top(&mut g);
    let real_source = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JZ27")
        .unwrap()
        .id
        .clone();
    assert_eq!(g.board(&line).unwrap().1.controller, 0);
    assert_eq!(g.defense(g.board(&line).unwrap().1, 2), 2);
    let damage = held(&mut g, "JC102", 0);
    let a = play(&g, 0, &damage, &line, None);
    apply(&mut g, 0, a);
    pass_top(&mut g);
    assert_eq!(g.board(&line).unwrap().1.damage, 1);
    assert!(g.pending.is_none());
    // A real funded response remains available after the immediate destruction.
    fund(&mut g, 0, "JC063", 2);
    let hide_target = board(&mut g, "LC23", 0, 0);
    let hide = held(&mut g, "JC063", 0);
    let spell = held(&mut g, "JZ55", 0);
    let mut room = envelope(&g);
    let initial = serde_json::to_string(&room).unwrap();
    let mut steps = vec![];
    save_room_fixture("cascade-before-immediate", &room);
    let a = play(&room.game, 0, &spell, &real_source, None);
    command(&mut room, &mut steps, 0, SessionAction::Game { action: a });
    assert!(room.board(&real_source).is_none() && room.board(&line).is_none());
    assert!(room.stack.is_empty());
    let p = room.pending.as_ref().unwrap();
    assert_eq!(p.seat, 2);
    let ChoiceResolution::Declare { declaration, .. } = &p.resolution else {
        panic!("expected ordinary death declaration")
    };
    assert_eq!(declaration.ability.event, Some(Event::Death));
    assert_eq!(declaration.source.card.id, line);
    assert_eq!(
        (
            declaration.actor,
            declaration.source.card.owner,
            declaration.source.card.controller
        ),
        (2, 2, 2)
    );
    assert_eq!(declaration.source.region, Some(2));
    assert_eq!(
        declaration.source.source_region_instance.as_deref(),
        Some(room.regions[2].card.id.as_str())
    );
    assert_eq!(
        room.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JZ55")
            .count(),
        1
    );
    save_room_fixture("cascade-death-choice", &room);
    let a = Action {
        choice_id: Some(room.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(vec!["accept".into()]),
        ..Action::new("choose")
    };
    command(&mut room, &mut steps, 2, SessionAction::Game { action: a });
    assert_eq!(room.stack.len(), 1);
    let death = room.stack[0].frame.as_ref().unwrap();
    assert_eq!(death.source.card.definition, "JZ31");
    assert_eq!(death.actor, 2);
    let frozen = serde_json::to_value(death).unwrap();
    let underlying = room.stack[0].id.clone();
    save_room_fixture("cascade-death-stack", &room);
    assert!(room.view(0, 0).response_window.unwrap().can_begin);
    let window = room.pacing.window.clone().unwrap();
    command(
        &mut room,
        &mut steps,
        0,
        SessionAction::BeginResponse {
            window_id: window.id.clone(),
            intent_id: "jz55-cascade-response".into(),
        },
    );
    save_room_fixture("cascade-response-composing", &room);
    let a = play(&room.game, 0, &hide, &hide_target, Some("hide"));
    command(
        &mut room,
        &mut steps,
        0,
        SessionAction::SubmitResponse {
            window_id: window.id,
            intent_id: "jz55-cascade-response".into(),
            action: a,
        },
    );
    assert_eq!(room.stack.len(), 2);
    assert_eq!(
        serde_json::to_value(room.stack[0].frame.as_ref().unwrap()).unwrap(),
        frozen
    );
    save_room_fixture("cascade-response-top", &room);
    for depth in [2, 1] {
        for _ in 0..16 {
            if room.stack.len() < depth {
                break;
            }
            let w = room.pacing.window.clone().unwrap();
            let seat = *w
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            command(
                &mut room,
                &mut steps,
                seat,
                SessionAction::PassResponse { window_id: w.id },
            );
        }
        assert_eq!(room.stack.len(), depth - 1);
        if depth == 2 {
            assert_eq!(room.stack[0].id, underlying);
            assert_eq!(
                serde_json::to_value(room.stack[0].frame.as_ref().unwrap()).unwrap(),
                frozen
            );
            assert_eq!(room.regions[2].influence, [0, 0]);
        }
    }
    assert_eq!(room.regions[2].influence, [0, 1]);
    assert!(room.stack.is_empty() && room.pending.is_none());
    assert!(room.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "LC23" && c.face_down));
    save_room_fixture("cascade-final", &room);
    if let Ok(dir) = std::env::var("JZ55_RESPONSE_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/cascade-response-chain.json"),
            serde_json::to_vec(&serde_json::json!({"initialState":initial,"steps":steps,
                "finalState":serde_json::to_string(&room).unwrap(),
                "underlyingStackInstance":underlying,"targetInstance":hide_target,
                "deathActor":2,"originalRegion":2}))
            .unwrap(),
        )
        .unwrap();
    }
    checkpoint(&room.game);
}

#[cfg(feature = "native")]
#[tokio::test]
async fn jz55_sqlite_game_and_real_response_receipts_restart_and_reject_without_double_payment() {
    use crate::service::{CreateRoom, JoinRoom, Store};
    for response in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jz55-atomic.sqlite");
        let store = Store::open(&path).unwrap();
        let host = store
            .create(CreateRoom {
                name: "P0".into(),
                mode: "teams".into(),
                deck_id: "watchers".into(),
                deck_draft: None,
            })
            .await
            .unwrap();
        let mut sessions = vec![host];
        for seat in 1..4 {
            sessions.push(
                store
                    .join(JoinRoom {
                        invite_code: sessions[0].invite_code.clone(),
                        name: format!("P{seat}"),
                        deck_id: "watchers".into(),
                        deck_draft: None,
                    })
                    .await
                    .unwrap(),
            );
        }
        drop(store);
        let mut g = game(if response { 0 } else { 2 });
        g.room_id = sessions[0].room_id.clone();
        g.invite_code = sessions[0].invite_code.clone();
        fund(&mut g, 2, "JC084", 2);
        let spell = held(&mut g, "JZ55", 2);
        let target = board(&mut g, "LC23", 0, 2);
        if response {
            fund(&mut g, 0, "JC063", 2);
            let hide = held(&mut g, "JC063", 0);
            let a = play(&g, 0, &hide, &target, Some("hide"));
            apply(&mut g, 0, a);
            apply(&mut g, 0, Action::new("pass"));
            apply(&mut g, 1, Action::new("pass"));
        }
        let mut room = envelope(&g);
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute(
            "UPDATE rooms SET state=?1, revision=?2 WHERE id=?3",
            rusqlite::params![
                serde_json::to_string(&room).unwrap(),
                room.revision,
                g.room_id
            ],
        )
        .unwrap();
        drop(db);
        if response {
            let window = room.pacing.window.clone().unwrap();
            assert_eq!(window.holder_team, 1);
            let begin = RoomCommand {
                command_id: "jz55-store-begin".into(),
                expected_version: room.revision,
                action: SessionAction::BeginResponse {
                    window_id: window.id,
                    intent_id: "jz55-store-intent".into(),
                },
            };
            let store = Store::open(&path).unwrap();
            store
                .command_at_now(&g.room_id, &sessions[2].token, begin, 0)
                .await
                .unwrap();
            drop(store);
            let db = rusqlite::Connection::open(&path).unwrap();
            let state: String = db
                .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| {
                    r.get(0)
                })
                .unwrap();
            drop(db);
            room = RoomEnvelope::from_persisted(&state).unwrap();
            assert!(matches!(
                room.pacing.window.as_ref().unwrap().members[&2],
                Decision::Composing { .. }
            ));
        }
        let a = play(&room.game, 2, &spell, &target, None);
        let session = if response {
            SessionAction::SubmitResponse {
                window_id: room.pacing.window.as_ref().unwrap().id.clone(),
                intent_id: "jz55-store-intent".into(),
                action: a,
            }
        } else {
            SessionAction::Game { action: a }
        };
        let cmd = RoomCommand {
            command_id: "jz55-store-once".into(),
            expected_version: room.revision,
            action: session,
        };
        let expected = room.transition(2, Some(cmd.clone()), 0).unwrap();
        assert!(expected.error_code.is_none());
        let store = Store::open(&path).unwrap();
        let accepted = store
            .command_at_now(&g.room_id, &sessions[2].token, cmd.clone(), 0)
            .await
            .unwrap();
        drop(store);
        let after = RoomEnvelope::from_persisted(&expected.state).unwrap();
        assert_eq!(
            serde_json::to_value(&accepted).unwrap(),
            serde_json::to_value(after.view(2, 0)).unwrap()
        );
        assert_eq!(after.resources(2), 0);
        assert!(after.board(&target).is_none());
        assert_eq!(
            after.players[2]
                .graveyard
                .iter()
                .filter(|c| c.definition == "JZ55")
                .count(),
            1
        );
        let db = rusqlite::Connection::open(&path).unwrap();
        let before: String = db
            .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| {
                r.get(0)
            })
            .unwrap();
        drop(db);
        assert_eq!(before, expected.state);
        let store = Store::open(&path).unwrap();
        let duplicate = store
            .command_at_now(&g.room_id, &sessions[2].token, cmd.clone(), 90_000)
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&accepted).unwrap(),
            serde_json::to_value(duplicate).unwrap()
        );
        let mut conflict = cmd.clone();
        conflict.expected_version += 1;
        assert_eq!(
            store
                .command_at_now(&g.room_id, &sessions[2].token, conflict, 90_000)
                .await
                .unwrap_err()
                .error,
            "command_id_conflict"
        );
        let invalid = RoomCommand {
            command_id: "jz55-stale-instance".into(),
            expected_version: after.revision,
            action: SessionAction::Game {
                action: Action {
                    card_id: Some(spell.clone()),
                    target_id: Some(target.clone()),
                    ..Action::new("play")
                },
            },
        };
        assert!(store
            .command_at_now(&g.room_id, &sessions[2].token, invalid, 0)
            .await
            .is_err());
        drop(store);
        let db = rusqlite::Connection::open(&path).unwrap();
        let saved: String = db
            .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(saved, before);
        let count: i64 = db
            .query_row(
                "SELECT count(*) FROM commands WHERE room_id=?1",
                [&g.room_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, if response { 2 } else { 1 });
        if let Ok(dir) = std::env::var("JZ55_RECEIPT_EVIDENCE_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(format!("{dir}/{}-receipt.json",if response{"submit"}else{"game"}),serde_json::to_vec(&serde_json::json!({"response":response,"initialState":serde_json::to_string(&room).unwrap(),"command":cmd,"expected":expected,"persistedState":saved,"commandsStored":count,"duplicateExact":true,"conflictRejected":true,"staleInstanceRejected":true,"storeReopens":if response{3}else{2}})).unwrap()).unwrap();
        }
    }
}
