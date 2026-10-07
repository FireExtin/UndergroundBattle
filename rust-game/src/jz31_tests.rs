//! Explicit offline layouts; real paid commands, response windows and four-seat
//! persisted projections. Primitive mutations are identified in each test.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, deck, model::*, rules::*};
use std::collections::BTreeMap;

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
fn cast(g: &mut Game, def: &str, actor: usize, target: &str) {
    let source = held(g, def, actor);
    let a = g
        .legal_actions(actor)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&source)
                && a.action.target_id.as_deref() == Some(target)
        })
        .unwrap()
        .action;
    apply(g, actor, a);
    pass_top(g);
}
fn death(g: &mut Game, id: &str, cause: RemovalCause) {
    // Explicit primitive death input, followed by ordinary declaration/actions.
    g.remove_dead(id, cause);
    g.drive().unwrap();
    checkpoint(g);
}
fn pending_source(g: &Game) -> &Declaration {
    match &g.pending.as_ref().unwrap().resolution {
        ChoiceResolution::Declare { declaration, .. } => declaration,
        _ => panic!("expected death declaration"),
    }
}
fn accept(g: &mut Game) {
    assert_eq!(pending_source(g).ability.event, Some(Event::Death));
    choose(g, vec!["accept".into()]);
    assert_eq!(
        g.stack.last().unwrap().frame.as_ref().unwrap().ability_key,
        "death-source-influence"
    );
}
fn fixture(kind: &str, g: &Game) {
    if let Ok(dir) = std::env::var("JZ31_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
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

#[test]
fn jz31_whole_original_has_no_magic_and_exact_unparameterized_death_program() {
    let c = catalog::card("JZ31");
    assert_eq!(
        (&*c.name, &*c.color, c.cost, c.defense),
        ("破茧者秘教线人", "红", 3, Some(1))
    );
    assert_eq!(c.loyalty, ["红色"]);
    assert_eq!(c.subtypes, ["人类", "宿主"]);
    assert!(c.magic.is_empty());
    assert_eq!(c.magic_icon, MagicIcon::None);
    assert_eq!(
        c.permanent_icons,
        Icons {
            investigation: 1,
            influence: 1,
            ..Default::default()
        }
    );
    assert_eq!(c.temporary_icons, Icons::default());
    assert!(!c.unique && c.keywords.is_empty());
    assert_eq!(deck::copy_limit(c), Some(3));
    let a = &definition("JZ31").abilities[0];
    assert_eq!(a.event, Some(Event::Death));
    assert_eq!(a.timing, Timing::Fast);
    assert_eq!(a.response_policy, ResponsePolicy::Respondable);
    assert!(a.costs.is_empty() && a.targets.is_empty() && a.modes.is_empty());
    assert!(matches!(
        a.ops.as_slice(),
        [Op::PlaceOneInfluenceInSourceRegion]
    ));
    assert_eq!(
        serde_json::to_string(&a.ops).unwrap(),
        "[\"PlaceOneInfluenceInSourceRegion\"]"
    );
    assert_eq!(catalog::catalog().cards.len(), 103);
}

#[test]
fn jz31_real_deploy_needs_red_loyalty_but_no_magic_asset() {
    let mut g = initial(0);
    fund(&mut g, 0, "JC059", 3); // no red loyalty, no magic.
    let id = held(&mut g, "JZ31", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    g.players[0].assets.clear();
    fund(&mut g, 0, "JZ31", 3); // every asset has MagicIcon::None.
    assert!(g.players[0]
        .assets
        .iter()
        .all(|c| catalog::card(&c.definition).magic_icon == MagicIcon::None));
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert!(g.regions[2].cards.iter().any(|c| c.definition == "JZ31"));
    fixture("deployed-no-magic", &g);
}

#[test]
fn jz31_real_destroy_and_damage_offer_optional_respondable_one_influence() {
    for (spell, asset, cost) in [("JC091", "JC084", 3), ("JC102", "JC104", 2)] {
        for actor in 0..4 {
            let mut g = initial(actor);
            let id = board(&mut g, "JZ31", actor, 2);
            let original_region = g.regions[2].card.id.clone();
            fund(&mut g, actor, asset, cost);
            cast(&mut g, spell, actor, &id);
            assert!(g.board(&id).is_none());
            let d = pending_source(&g);
            assert_eq!((d.actor, d.source.card.controller), (actor, actor));
            assert_eq!(d.source.card.id, id);
            assert_eq!(d.source.region, Some(2));
            assert_eq!(
                d.source.source_region_instance.as_deref(),
                Some(original_region.as_str())
            );
            assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(0));
            fixture(&format!("death-choice-{spell}-{actor}"), &g);
            let mut declined = g.clone();
            choose(&mut declined, vec![]);
            assert_eq!(declined.regions[2].influence, [0; 2]);
            accept(&mut g);
            assert_eq!(g.regions[2].influence, [0; 2]); // response window before effect.
            fixture(&format!("death-stack-{spell}-{actor}"), &g);
            pass_top(&mut g);
            let mut expected = [0; 2];
            expected[g.team(actor)] = 1;
            assert_eq!(g.regions[2].influence, expected);
            assert!(g
                .regions
                .iter()
                .enumerate()
                .all(|(r, x)| r == 2 || x.influence == [0; 2]));
            fixture(&format!("death-result-{spell}-{actor}"), &g);
        }
    }
}

#[test]
fn jz31_real_sacrifice_last_controller_acts_and_owner_receives_grave() {
    for actor in 0..4 {
        let owner = (actor + 2) % 4;
        let mut g = initial(actor);
        let id = board(&mut g, "JZ31", owner, 2);
        let other = board(&mut g, "JZ31", owner, 1);
        fund(&mut g, actor, "JC104", 3);
        cast(&mut g, "JC129", actor, &id); // actual printed temporary control.
        assert_eq!(g.board(&id).unwrap().1.controller, actor);
        assert_eq!(g.board(&other).unwrap().1.controller, owner);
        fund(&mut g, actor, "JZ31", 2);
        let source = held(&mut g, "JC049", actor);
        reject(
            &mut g,
            actor,
            Action {
                card_id: Some(source.clone()),
                cost_selected: Some(vec![other.clone()]),
                ..Action::new("play")
            },
        );
        let mut a = g
            .legal_actions(actor)
            .into_iter()
            .find(|a| a.action.kind == "play" && a.action.card_id.as_deref() == Some(&source))
            .unwrap()
            .action;
        a.cost_selected = Some(vec![id.clone()]);
        apply(&mut g, actor, a);
        assert!(g.board(&id).is_none());
        assert_eq!(pending_source(&g).actor, actor);
        assert_eq!(pending_source(&g).source.card.controller, actor);
        assert!(g.players[owner]
            .graveyard
            .iter()
            .any(|c| c.definition == "JZ31" && c.id != id && c.controller == owner));
        assert!(g.players[actor]
            .graveyard
            .iter()
            .all(|c| c.definition != "JZ31"));
        fixture(&format!("borrowed-death-{actor}"), &g);
        accept(&mut g);
        pass_top(&mut g);
        let mut expected = [0; 2];
        expected[g.team(actor)] = 1;
        assert_eq!(g.regions[2].influence, expected);
        assert!(g.board(&other).is_some());
        pass_top(&mut g); // original JC049 frame still resolves once.
    }
}

#[test]
fn jz31_hidden_death_return_hand_and_bottom_never_trigger() {
    for mode in 0..4 {
        let mut g = initial(0);
        let id = board(&mut g, "JZ31", 0, 2);
        match mode {
            0 => {
                g.board_mut(&id).unwrap().face_down = true;
                g.remove_dead(&id, RemovalCause::Destroy);
            }
            1 => {
                g.board_mut(&id).unwrap().face_down = true;
                g.damage(BTreeMap::from([(id.clone(), 99)])).unwrap();
                assert!(g.board(&id).is_some());
            }
            2 => g.return_hand(&id),
            _ => g.to_bottom(&id),
        }
        g.drive().unwrap();
        assert!(g.pending.is_none() && g.stack.is_empty());
        assert!(g.effects.iter().all(|e| !matches!(e, Effect::Declare { declaration } if declaration.ability.event == Some(Event::Death))));
        assert_eq!(g.regions[2].influence, [0; 2]);
        checkpoint(&g);
        fixture(&format!("nondeath-{mode}"), &g);
    }
}

#[test]
fn jz31_same_card_color_independent_instances_and_multiple_simultaneous_deaths() {
    let mut g = initial(0);
    let a = board(&mut g, "JZ31", 0, 2);
    let b = board(&mut g, "JZ31", 2, 1);
    let live = board(&mut g, "JZ31", 0, 2);
    assert!(a != b && a != live && b != live);
    g.damage(BTreeMap::from([(a.clone(), 1), (b.clone(), 1)]))
        .unwrap();
    g.drive().unwrap();
    assert_eq!(pending_source(&g).source.card.id, a);
    accept(&mut g);
    // Next simultaneous declaration may be offered while the first is stacked.
    assert_eq!(pending_source(&g).source.card.id, b);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[1].influence, [0, 1]);
    assert_eq!(g.regions[2].influence, [0, 0]);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [1, 0]);
    assert!(g.board(&live).is_some());
    fixture("independent-simultaneous", &g);
}

#[test]
fn jz31_simultaneous_control_source_departure_preserves_prelethal_controller() {
    for source_first in [true, false] {
        let mut g = initial(0);
        let id = board(&mut g, "JZ31", 2, 2);
        let source = board(&mut g, "JZ27", 0, 2);
        g.board_mut(&source).unwrap().face_down = true;
        fund(&mut g, 0, "JZ24", 6);
        apply(
            &mut g,
            0,
            Action {
                card_id: Some(source),
                ..Action::new("reveal")
            },
        );
        pass_top(&mut g);
        choose(&mut g, vec![id.clone()]);
        pass_top(&mut g);
        let source = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "JZ27")
            .unwrap()
            .id
            .clone();
        assert_eq!(g.board(&id).unwrap().1.controller, 0);
        g.regions[2].cards.sort_by_key(|c| {
            if c.id == source {
                !source_first
            } else {
                source_first
            }
        });
        g.damage(BTreeMap::from([(source.clone(), 3), (id.clone(), 1)]))
            .unwrap();
        assert!(g.board(&source).is_none() && g.board(&id).is_none());
        g.drive().unwrap();
        assert_eq!(
            (
                pending_source(&g).actor,
                pending_source(&g).source.card.controller
            ),
            (0, 0)
        );
        accept(&mut g);
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [1, 0]);
    }
}

#[test]
fn jz31_aura_departure_cascade_and_multiple_copies_each_place_one() {
    let mut g = initial(0);
    let aura = board(&mut g, "JC059", 0, 2);
    let a = board(&mut g, "JZ31", 0, 2);
    let b = board(&mut g, "JZ31", 1, 2);
    g.board_mut(&a).unwrap().damage = 1;
    g.board_mut(&b).unwrap().damage = 1;
    assert_eq!(g.defense(g.board(&a).unwrap().1, 2), 2);
    g.damage(BTreeMap::from([(aura.clone(), 2)])).unwrap();
    assert!(g.board(&aura).is_none() && g.board(&a).is_none() && g.board(&b).is_none());
    g.drive().unwrap();
    accept(&mut g);
    accept(&mut g);
    pass_top(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [2, 0]);
    fixture("cascade-two-triggers", &g);
}

#[test]
fn jz31_region_replacement_before_declaration_or_during_response_never_redirects() {
    for stage in 0..3 {
        let mut g = initial(0);
        let id = board(&mut g, "JZ31", 0, 2);
        let original = g.regions[2].card.id.clone();
        g.remove_dead(&id, RemovalCause::Destroy);
        if stage > 0 {
            g.drive().unwrap();
        }
        if stage > 1 {
            accept(&mut g);
        }
        // Explicit primitive replacement; a different instance of the same printed region.
        let replacement = g.make_card(&g.regions[2].card.definition.clone(), 0);
        assert_ne!(replacement.id, original);
        g.regions[2].card = replacement;
        checkpoint(&g);
        if stage == 0 {
            g.drive().unwrap();
        }
        if stage < 2 {
            accept(&mut g);
        }
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [0; 2]);
        fixture(&format!("replaced-{stage}"), &g);
    }
}

#[test]
fn jz31_declared_snapshot_does_not_follow_returned_card_new_instance_or_controller() {
    let mut g = initial(0);
    let old = board(&mut g, "JZ31", 0, 2);
    death(&mut g, &old, RemovalCause::Sacrifice);
    accept(&mut g);
    // A same-card return is a fresh instance in a different region/controller.
    let c = g.players[0].graveyard.pop().unwrap();
    let mut c = g.reset_zone_card(c);
    let fresh = c.id.clone();
    c.controller = 2;
    g.regions[1].cards.push(c);
    assert_ne!(fresh, old);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [1, 0]);
    assert_eq!(g.regions[1].influence, [0, 0]);
    assert_eq!(g.board(&fresh).unwrap().1.controller, 2);
}

#[test]
fn jz31_one_point_cancels_enemy_or_opens_existing_win_threshold_window() {
    let mut g = initial(0);
    let id = board(&mut g, "JZ31", 0, 2);
    g.regions[2].influence = [0, 2];
    death(&mut g, &id, RemovalCause::Destroy);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [0, 1]);
    let mut g = initial(0);
    g.regions[2].card = g.make_card("DQJC115", 0); // threshold3, points2.
    g.regions[2].influence = [2, 0];
    for def in ["DQJC110", "DQJC112"] {
        let c = g.make_card(def, 0);
        g.players[0].score_cards.push(c);
    }
    let id = board(&mut g, "JZ31", 0, 2);
    death(&mut g, &id, RemovalCause::Lethal);
    accept(&mut g);
    pass_top(&mut g);
    assert_eq!(g.regions[2].influence, [3, 0]);
    assert_eq!(g.window, Some(Window::Win(2, 0)));
    fixture("win-threshold", &g);
    for _ in 0..12 {
        if g.status != "playing" {
            break;
        }
        if g.pending.is_some() {
            choose(&mut g, vec![]);
        } else {
            let seat = (0..4)
                .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
                .unwrap();
            apply(&mut g, seat, Action::new("pass"));
        }
    }
    assert_eq!(g.status, "finished");
    assert_eq!(g.winner_team, Some(0));
    fixture("victory", &g);
}

#[test]
fn jz31_declaration_and_stack_restore_old_choice_and_actor_rejections_are_atomic() {
    let mut g = initial(0);
    let id = board(&mut g, "JZ31", 0, 2);
    death(&mut g, &id, RemovalCause::Destroy);
    let pending = g.pending.clone().unwrap();
    let action = Action {
        choice_id: Some(pending.choice.id),
        selected: Some(vec!["accept".into()]),
        ..Action::new("choose")
    };
    reject(&mut g, 2, action.clone());
    let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    apply(&mut g, 0, action.clone());
    apply(&mut restored, 0, action.clone());
    assert_eq!(
        serde_json::to_value(&g).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    reject(&mut g, 0, action);
    let mut restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    pass_top(&mut g);
    pass_top(&mut restored);
    assert_eq!(
        serde_json::to_value(&g).unwrap(),
        serde_json::to_value(&restored).unwrap()
    );
    assert_eq!(g.regions[2].influence, [1, 0]);
}

#[cfg(feature = "native")]
#[tokio::test]
async fn jz31_sqlite_receipt_restart_duplicate_and_conflict_do_not_redeclare_or_replace_result() {
    use crate::room::{RoomCommand, SessionAction};
    use crate::service::{CreateRoom, JoinRoom, Store};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("jz31-receipts.sqlite");
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
    let mut g = initial(0);
    g.room_id = sessions[0].room_id.clone();
    g.invite_code = sessions[0].invite_code.clone();
    let id = board(&mut g, "JZ31", 0, 2);
    death(&mut g, &id, RemovalCause::Sacrifice);
    let room = envelope(&g);
    let state = serde_json::to_string(&room).unwrap();
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute(
        "UPDATE rooms SET state=?2,initial_state=?2,revision=?3 WHERE id=?1",
        rusqlite::params![g.room_id, state, room.revision],
    )
    .unwrap();
    db.execute("DELETE FROM journal WHERE room_id=?1", [&g.room_id])
        .unwrap();
    drop(db);
    let command = RoomCommand {
        command_id: "jz31-once".into(),
        expected_version: room.revision,
        action: SessionAction::Game {
            action: Action {
                choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
                selected: Some(vec!["accept".into()]),
                ..Action::new("choose")
            },
        },
    };
    let expected = room.transition(0, Some(command.clone()), 0).unwrap();
    assert!(expected.error_code.is_none());
    let store = Store::open(&path).unwrap();
    let accepted = store
        .command_at_now(&g.room_id, &sessions[0].token, command.clone(), 0)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&accepted).unwrap(),
        serde_json::to_value(
            crate::room::RoomEnvelope::from_persisted(&expected.state)
                .unwrap()
                .view(0, 0)
        )
        .unwrap()
    );
    drop(store);
    let db = rusqlite::Connection::open(&path).unwrap();
    let before: String = db
        .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| {
            r.get(0)
        })
        .unwrap();
    drop(db);
    let store = Store::open(&path).unwrap();
    let duplicate = store
        .command_at_now(&g.room_id, &sessions[0].token, command.clone(), 90_000)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&accepted).unwrap(),
        serde_json::to_value(&duplicate).unwrap()
    );
    let mut conflict = command.clone();
    conflict.expected_version += 1;
    assert_eq!(
        store
            .command_at_now(&g.room_id, &sessions[0].token, conflict, 90_000)
            .await
            .unwrap_err()
            .error,
        "command_id_conflict"
    );
    let db = rusqlite::Connection::open(&path).unwrap();
    let after: String = db
        .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(before, after);
    let count: i64 = db
        .query_row(
            "SELECT count(*) FROM commands WHERE room_id=?1",
            [&g.room_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
    let saved = crate::room::RoomEnvelope::from_persisted(&after).unwrap();
    assert_eq!(saved.stack.len(), 1);
    let mut resumed = saved.game.clone();
    pass_top(&mut resumed);
    assert_eq!(resumed.regions[2].influence, [1, 0]);
}

#[test]
fn jz31_actual_composed_response_hides_another_instance_before_frozen_death_effect() {
    use crate::room::{Decision, RoomCommand, RoomEnvelope, SessionAction};
    // One initial fixture, then one continuous persisted RoomEnvelope. No
    // reconstructed priority windows or board mutations between commands.
    fn command(
        room: &mut RoomEnvelope,
        steps: &mut Vec<serde_json::Value>,
        seat: usize,
        action: SessionAction,
    ) {
        let state = serde_json::to_string(room).unwrap();
        let cmd = RoomCommand {
            command_id: format!("jz31-composed-{}", steps.len()),
            expected_version: room.revision,
            action,
        };
        let expected = room.transition(seat, Some(cmd.clone()), 0).unwrap();
        assert!(
            expected.error_code.is_none(),
            "{:?}",
            expected.error_message
        );
        *room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        assert_eq!(serde_json::to_string(room).unwrap(), expected.state);
        let views: Vec<_> = (0..4)
            .map(|s| serde_json::to_value(room.view(s, 0)).unwrap())
            .collect();
        steps.push(serde_json::json!({"state": state, "seat": seat, "command": cmd, "serverNowMs": "0", "expected": expected, "views": views}));
    }
    fn resolve_top(room: &mut RoomEnvelope, steps: &mut Vec<serde_json::Value>) {
        let depth = room.stack.len();
        assert!(depth > 0);
        for _ in 0..32 {
            if room.stack.len() < depth || room.pending.is_some() {
                return;
            }
            let window = room.pacing.window.clone().unwrap();
            let seat = *window
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            command(
                room,
                steps,
                seat,
                SessionAction::PassResponse {
                    window_id: window.id,
                },
            );
        }
        panic!("bounded composed response did not resolve");
    }
    fn play(
        room: &RoomEnvelope,
        seat: usize,
        source: &str,
        target: &str,
        mode: Option<&str>,
    ) -> Action {
        room.game
            .legal_actions(seat)
            .into_iter()
            .find(|a| {
                a.action.kind == "play"
                    && a.action.card_id.as_deref() == Some(source)
                    && a.action.target_id.as_deref() == Some(target)
                    && a.action.option.as_deref() == mode
            })
            .unwrap()
            .action
    }
    let mut g = initial(0);
    let dead = board(&mut g, "JZ31", 2, 2);
    let other = board(&mut g, "JZ31", 2, 2);
    let region_instance = g.regions[2].card.id.clone();
    fund(&mut g, 0, "JC104", 3);
    fund(&mut g, 0, "JC084", 3);
    fund(&mut g, 2, "JC056", 2);
    let control = held(&mut g, "JC129", 0);
    let destroy = held(&mut g, "JC091", 0);
    let hide = held(&mut g, "JC063", 2);
    let mut room = envelope(&g);
    let initial_state = serde_json::to_string(&room).unwrap();
    let mut steps = vec![];
    let a = play(&room, 0, &control, &dead, None);
    command(&mut room, &mut steps, 0, SessionAction::Game { action: a });
    resolve_top(&mut room, &mut steps);
    assert_eq!(room.board(&dead).unwrap().1.controller, 0); // real JC129 control.
    assert_eq!(room.board(&other).unwrap().1.controller, 2);
    let a = play(&room, 0, &destroy, &dead, None);
    command(&mut room, &mut steps, 0, SessionAction::Game { action: a });
    resolve_top(&mut room, &mut steps);
    let pending = room.pending.clone().unwrap();
    let declaration = pending_source(&room.game);
    assert_eq!(
        (
            declaration.actor,
            declaration.source.card.owner,
            declaration.source.card.controller
        ),
        (0, 2, 0)
    );
    assert_eq!(declaration.source.card.id, dead);
    assert_eq!(declaration.source.region, Some(2));
    assert_eq!(
        declaration.source.source_region_instance.as_deref(),
        Some(region_instance.as_str())
    );
    command(
        &mut room,
        &mut steps,
        0,
        SessionAction::Game {
            action: Action {
                choice_id: Some(pending.choice.id),
                selected: Some(vec!["accept".into()]),
                ..Action::new("choose")
            },
        },
    );
    assert_eq!(room.stack.len(), 1);
    let frozen = serde_json::to_value(room.stack[0].frame.as_ref().unwrap()).unwrap();
    assert_eq!(room.regions[2].influence, [0, 0]);
    assert_eq!(room.pacing.window.as_ref().unwrap().holder_team, 0);
    for seat in [0, 1] {
        let window_id = room.pacing.window.as_ref().unwrap().id.clone();
        command(
            &mut room,
            &mut steps,
            seat,
            SessionAction::PassResponse { window_id },
        );
        assert_eq!(room.stack.len(), 1);
        assert_eq!(
            serde_json::to_value(room.stack[0].frame.as_ref().unwrap()).unwrap(),
            frozen
        );
    }
    let window = room.pacing.window.clone().unwrap();
    assert_eq!(window.holder_team, 1);
    assert!(room.view(2, 0).response_window.unwrap().can_begin);
    let intent = "jz31-real-hide-response".to_string();
    command(
        &mut room,
        &mut steps,
        2,
        SessionAction::BeginResponse {
            window_id: window.id.clone(),
            intent_id: intent.clone(),
        },
    );
    assert!(matches!(
        room.pacing.window.as_ref().unwrap().members[&2],
        Decision::Composing { .. }
    ));
    assert_eq!(
        room.view(2, 0)
            .response_window
            .unwrap()
            .my_intent_id
            .as_deref(),
        Some(intent.as_str())
    );
    assert_eq!(
        serde_json::to_value(room.stack[0].frame.as_ref().unwrap()).unwrap(),
        frozen
    );
    let a = play(&room, 2, &hide, &other, Some("hide"));
    command(
        &mut room,
        &mut steps,
        2,
        SessionAction::SubmitResponse {
            window_id: window.id,
            intent_id: intent,
            action: a,
        },
    );
    assert_eq!(room.stack.len(), 2);
    assert_eq!(
        room.stack[1].frame.as_ref().unwrap().source.card.definition,
        "JC063"
    );
    assert_eq!(
        serde_json::to_value(room.stack[0].frame.as_ref().unwrap()).unwrap(),
        frozen
    );
    assert_eq!(room.regions[2].influence, [0, 0]);
    resolve_top(&mut room, &mut steps);
    assert_eq!(room.stack.len(), 1);
    assert!(room.board(&other).is_none());
    let hidden = room.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JZ31")
        .unwrap();
    assert!(hidden.face_down && hidden.id != other && hidden.id != dead);
    assert_eq!((hidden.owner, hidden.controller), (2, 2));
    assert_eq!(
        serde_json::to_value(room.stack[0].frame.as_ref().unwrap()).unwrap(),
        frozen
    );
    assert_eq!(room.regions[2].influence, [0, 0]); // response resolves first.
    resolve_top(&mut room, &mut steps);
    assert!(room.stack.is_empty() && room.pending.is_none());
    assert_eq!(room.regions[2].influence, [1, 0]); // frozen death actor, exactly one.
    assert!(room.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JZ31" && c.id != dead && c.controller == 2));
    assert!(room.players[0]
        .graveyard
        .iter()
        .all(|c| c.definition != "JZ31"));
    if let Ok(dir) = std::env::var("JZ31_RESPONSE_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/response-chain.json"), serde_json::to_vec(&serde_json::json!({
            "initialState": initial_state, "steps": steps, "finalState": serde_json::to_string(&room).unwrap(),
            "deathInstance": dead, "sourceRegionInstance": region_instance, "deathActor": 0, "deathOwner": 2,
            "responseCard": "JC063", "responseActor": 2, "responseTargetOldInstance": other,
            "frozenDeathFrame": frozen, "finalInfluence": [1, 0]
        })).unwrap()).unwrap();
    }
}
