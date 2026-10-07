use hegemony_server::{
    model::{Action, Game, Window},
    room::{Decision, QuoteRequest, RoomCommand, RoomEnvelope, RoomTransition, SessionAction},
};

fn fixture(mode: &str) -> RoomEnvelope {
    let mut game = Game::new(
        "clock".into(),
        "CLOCK".into(),
        mode.into(),
        "P0".into(),
        "responders".into(),
        17,
    )
    .unwrap();
    for seat in 1..game.capacity() {
        game.join(format!("P{seat}"), "responders".into()).unwrap();
    }
    for p in &mut game.players {
        p.ready = true;
    }
    game.apply(0, Action::new("start")).unwrap();
    while let Some(p) = game.pending.clone() {
        game.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    // Explicit initial layout; every subsequent transition goes through the room protocol.
    game.window = Some(Window::Action(0));
    game.priority_team = 0;
    game.active_team = 0;
    game.first_team = 0;
    for p in &mut game.players {
        p.hand.clear();
        p.assets.clear();
    }
    for seat in 0..game.capacity() {
        let source = game.make_card("JC042", seat);
        game.regions[0].cards.push(source);
        for _ in 0..3 {
            let a = game.make_card("JC091", seat);
            game.players[seat].assets.push(a);
        }
    }
    let enemy = if mode == "teams" { 2 } else { 1 };
    let victim = game.make_card("LC20", enemy);
    game.regions[0].cards.push(victim);
    let murder = game.make_card("JC091", 0);
    game.players[0].hand.push(murder);
    if mode == "teams" {
        let ritual = game.make_card("JZ54", 2);
        game.players[2].hand.push(ritual);
    }
    RoomEnvelope::from_game(game)
}
fn apply(room: &mut RoomEnvelope, seat: usize, action: SessionAction, now: u64) -> RoomTransition {
    let command = RoomCommand {
        command_id: format!("command:{}", room.revision + 1),
        expected_version: room.revision,
        action,
    };
    let before = room.clone();
    let t = room.transition(seat, Some(command), now).unwrap();
    assert_eq!(t.outcome, "accepted", "{:?}", t.error_message);
    *room = RoomEnvelope::from_persisted(&t.state).unwrap();
    assert_eq!(
        serde_json::to_string(&before.replay_events(&t.journal).unwrap()).unwrap(),
        t.state
    );
    t
}
fn play_murder(room: &mut RoomEnvelope) -> RoomTransition {
    let enemy = if room.game.mode == "teams" { 2 } else { 1 };
    let victim = room.game.regions[0]
        .cards
        .iter()
        .find(|c| c.controller == enemy && c.definition == "LC20")
        .unwrap()
        .id
        .clone();
    let murder = room.game.players[0].hand[0].id.clone();
    apply(
        room,
        0,
        Action {
            card_id: Some(murder),
            target_id: Some(victim),
            ..Action::new("play")
        }
        .into(),
        1_000,
    )
}
fn window(room: &RoomEnvelope) -> String {
    room.pacing.window.as_ref().unwrap().id.clone()
}

#[test]
fn paused_room_freezes_paid_stack_and_remaining_clock_across_days_then_resumes() {
    for mode in ["duel", "teams"] {
        let mut room = fixture(mode);
        play_murder(&mut room);
        let old_window = window(&room);
        let paid = serde_json::to_value(&room.game).unwrap();
        apply(&mut room, 1, SessionAction::PauseRoom, 2_250);
        let paused = serde_json::to_string(&room).unwrap();
        assert_eq!(room.pacing.pause.as_ref().unwrap().paused_by, 1);
        for now in [2_250, 6_000, 86_402_250, 172_802_250] {
            let tick = room.transition(0, None, now).unwrap();
            assert!(!tick.changed && tick.journal.is_empty());
            assert_eq!(tick.state, paused);
            assert_eq!(tick.view.server_now_ms, 2_250);
            assert!(!tick.view.response_window.as_ref().unwrap().can_begin);
            assert!(tick.view.legal_actions.is_empty());
        }
        apply(&mut room, 0, SessionAction::ResumeRoom, 86_402_250);
        assert!(room.pacing.pause.is_none());
        assert_ne!(window(&room), old_window);
        let mut after = serde_json::to_value(&room.game).unwrap();
        after["version"] = paid["version"].clone();
        assert_eq!(
            after, paid,
            "Pause/resume cannot re-pay or advance game rules"
        );
        for d in room.pacing.window.as_ref().unwrap().members.values() {
            if let Decision::Undecided { deadline_ms } = d {
                assert_eq!(*deadline_ms, 86_406_000);
            }
        }
        assert!(!room.transition(0, None, 86_405_999).unwrap().changed);
        assert!(room.transition(0, None, 86_406_000).unwrap().changed);
        let late = room
            .transition(
                0,
                Some(RoomCommand {
                    command_id: "pre-pause-window".into(),
                    expected_version: room.revision,
                    action: SessionAction::BeginResponse {
                        window_id: old_window,
                        intent_id: "old".into(),
                    },
                }),
                86_402_251,
            )
            .unwrap();
        assert_eq!(late.error_code.as_deref(), Some("window_expired"));
    }
}

#[test]
fn pause_blocks_every_ordinary_protocol_command_and_quote_atomically() {
    let mut room = fixture("teams");
    play_murder(&mut room);
    let id = window(&room);
    apply(
        &mut room,
        0,
        SessionAction::BeginResponse {
            window_id: id.clone(),
            intent_id: "draft".into(),
        },
        1_500,
    );
    apply(&mut room, 3, SessionAction::PauseRoom, 2_000);
    let before = serde_json::to_string(&room).unwrap();
    let actions = [
        SessionAction::Game {
            action: Action::new("pass"),
        },
        SessionAction::BeginResponse {
            window_id: id.clone(),
            intent_id: "other".into(),
        },
        SessionAction::PassResponse {
            window_id: id.clone(),
        },
        SessionAction::CancelAndPass {
            window_id: id.clone(),
            intent_id: "draft".into(),
        },
        SessionAction::SubmitResponse {
            window_id: id.clone(),
            intent_id: "draft".into(),
            action: Action::new("activate"),
        },
        SessionAction::PauseRoom,
    ];
    for (i, action) in actions.into_iter().enumerate() {
        let t = room
            .transition(
                0,
                Some(RoomCommand {
                    command_id: format!("blocked-{i}"),
                    expected_version: room.revision,
                    action,
                }),
                86_400_000,
            )
            .unwrap();
        assert_eq!(t.error_code.as_deref(), Some("room_paused"));
        assert!(!t.changed && t.journal.is_empty());
        assert_eq!(t.state, before);
    }
    assert!(room
        .quote(
            0,
            QuoteRequest {
                window_id: id,
                intent_id: "draft".into(),
                draft: None
            }
        )
        .is_err());
    assert!(room
        .transition(
            4,
            Some(RoomCommand {
                command_id: "spectator".into(),
                expected_version: room.revision,
                action: SessionAction::ResumeRoom
            }),
            86_400_000
        )
        .is_err());
    apply(&mut room, 2, SessionAction::ResumeRoom, 86_400_000);
    let w = room.pacing.window.as_ref().unwrap();
    assert!(matches!(&w.members[&0], Decision::Composing { intent_id } if intent_id == "draft"));
    assert!(
        matches!(&w.members[&1], Decision::Undecided { deadline_ms } if *deadline_ms == 86_404_000)
    );
}

#[test]
fn pause_keeps_private_pending_choice_and_no_window_resume_has_no_automatic_gameplay() {
    // A real initial game choice, rather than an invented timer-only choice.
    let mut g = Game::new(
        "choice".into(),
        "CHOICE".into(),
        "duel".into(),
        "A".into(),
        "responders".into(),
        1,
    )
    .unwrap();
    g.join("B".into(), "responders".into()).unwrap();
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
    assert!(g.pending.is_some());
    let mut room = RoomEnvelope::from_game(g);
    let choice = serde_json::to_value(&room.game.pending).unwrap();
    apply(&mut room, 1, SessionAction::PauseRoom, 2_000);
    let saved = serde_json::to_string(&room).unwrap();
    room = RoomEnvelope::from_persisted(&saved).unwrap();
    assert_eq!(serde_json::to_value(&room.game.pending).unwrap(), choice);
    assert!(!room.transition(0, None, 86_400_000).unwrap().changed);
    apply(&mut room, 0, SessionAction::ResumeRoom, 86_400_000);
    assert!(room.pacing.window.is_none());
    assert_eq!(serde_json::to_value(&room.game.pending).unwrap(), choice);
}

#[test]
fn pause_after_deadline_commits_due_tick_before_rejecting_stale_pause() {
    let mut room = fixture("duel");
    play_murder(&mut room);
    let t = room
        .transition(
            1,
            Some(RoomCommand {
                command_id: "late-pause".into(),
                expected_version: room.revision,
                action: SessionAction::PauseRoom,
            }),
            6_000,
        )
        .unwrap();
    assert!(t.changed);
    assert_eq!(t.error_code.as_deref(), Some("version_conflict"));
    assert!(matches!(
        t.journal.as_slice(),
        [hegemony_server::room::SessionEvent::Tick { .. }]
    ));
    room = RoomEnvelope::from_persisted(&t.state).unwrap();
    assert!(room.pacing.pause.is_none());
    assert_eq!(room.game.priority_team, 1);
    apply(&mut room, 1, SessionAction::PauseRoom, 6_000);
}

#[test]
fn invalid_pause_records_resume_overflow_and_stale_management_reject_without_mutation() {
    let mut room = fixture("duel");
    play_murder(&mut room);
    apply(&mut room, 0, SessionAction::PauseRoom, 2_000);
    let before = serde_json::to_string(&room).unwrap();
    for now in [u64::MAX, 86_400_000] {
        let t = room
            .transition(
                1,
                Some(RoomCommand {
                    command_id: format!("resume-{now}"),
                    expected_version: if now == u64::MAX {
                        room.revision
                    } else {
                        room.revision - 1
                    },
                    action: SessionAction::ResumeRoom,
                }),
                now,
            )
            .unwrap();
        assert!(!t.changed);
        assert_eq!(t.state, before);
        assert!(t.error_code.is_some());
    }
    for variant in 0..3 {
        let mut bad = room.clone();
        match variant {
            0 => bad.pacing.pause.as_mut().unwrap().paused_at_ms += 1,
            1 => bad.pacing.pause.as_mut().unwrap().paused_by = 4,
            _ => bad.game.status = "finished".into(),
        }
        assert!(RoomEnvelope::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    }
    let lobby = RoomEnvelope::from_game(
        Game::new(
            "lobby".into(),
            "LOBBY".into(),
            "duel".into(),
            "A".into(),
            "responders".into(),
            1,
        )
        .unwrap(),
    );
    let t = lobby
        .transition(
            0,
            Some(RoomCommand {
                command_id: "pause-lobby".into(),
                expected_version: lobby.revision,
                action: SessionAction::PauseRoom,
            }),
            0,
        )
        .unwrap();
    assert!(!t.changed);
    assert_eq!(t.error_code.as_deref(), Some("invalid_action"));
}

#[test]
fn pause_native_full_transition_trace_for_actual_wasm_parity() {
    let mut room = fixture("teams");
    let initial = serde_json::to_string(&room).unwrap();
    let mut steps = vec![];
    let mut record = |room: &mut RoomEnvelope,
                      seat: usize,
                      action: Option<SessionAction>,
                      now: u64| {
        let cmd = action.map(|action| RoomCommand {
            command_id: format!("pause-parity-{}", steps.len()),
            expected_version: room.revision,
            action,
        });
        let t = room.transition(seat, cmd.clone(), now).unwrap();
        if t.changed {
            assert_eq!(
                serde_json::to_string(&room.replay_events(&t.journal).unwrap()).unwrap(),
                t.state
            );
        }
        *room = RoomEnvelope::from_persisted(&t.state).unwrap();
        steps.push(serde_json::json!({ "seat":seat,"command":cmd,"serverNowMs":now.to_string(),"transition":t,
            "views":(0..4).map(|s| room.view(s,room.pacing.last_server_now_ms)).collect::<Vec<_>>() }));
    };
    let victim = room.game.regions[0]
        .cards
        .iter()
        .find(|c| c.controller == 2 && c.definition == "LC20")
        .unwrap()
        .id
        .clone();
    let murder = room.game.players[0].hand[0].id.clone();
    record(
        &mut room,
        0,
        Some(SessionAction::Game {
            action: Action {
                card_id: Some(murder),
                target_id: Some(victim),
                ..Action::new("play")
            },
        }),
        1_000,
    );
    let old_id = window(&room);
    record(
        &mut room,
        0,
        Some(SessionAction::BeginResponse {
            window_id: old_id.clone(),
            intent_id: "saved-composition".into(),
        }),
        1_500,
    );
    record(&mut room, 3, Some(SessionAction::PauseRoom), 2_000);
    record(&mut room, 0, None, 86_400_000);
    record(
        &mut room,
        1,
        Some(SessionAction::PassResponse {
            window_id: old_id.clone(),
        }),
        86_400_000,
    );
    record(&mut room, 2, Some(SessionAction::ResumeRoom), 86_400_000);
    record(
        &mut room,
        1,
        Some(SessionAction::PassResponse { window_id: old_id }),
        86_400_001,
    );
    record(&mut room, 1, None, 86_403_999);
    record(&mut room, 1, None, 86_404_000);
    assert_eq!(room.game.stack.len(), 1);
    assert!(matches!(
        room.pacing.window.as_ref().unwrap().members[&0],
        Decision::Composing { .. }
    ));
    if let Ok(dir) = std::env::var("PAUSE_PARITY_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/native-trace.json"), serde_json::to_vec(&serde_json::json!({"scope":"explicit-offline-teams-layout-paid-room-protocol", "initialState":initial,"steps":steps})).unwrap()).unwrap();
    }
}

#[test]
fn deadline_4999_accepts_begin_5000_expires_and_new_window_gets_fresh_time() {
    let mut room = fixture("duel");
    play_murder(&mut room);
    let original = room.clone();
    let id = window(&room);
    let before = room.game.clone();
    apply(
        &mut room,
        0,
        SessionAction::BeginResponse {
            window_id: id.clone(),
            intent_id: "intent".into(),
        },
        5_999,
    );
    let mut comparable = room.game.clone();
    comparable.version = before.version;
    assert_eq!(
        serde_json::to_string(&before).unwrap(),
        serde_json::to_string(&comparable).unwrap()
    );
    let stalled = room.transition(0, None, 999_999).unwrap();
    assert!(!stalled.changed);
    assert_eq!(stalled.version, room.revision);
    let expired = original
        .transition(
            0,
            Some(RoomCommand {
                command_id: "late".into(),
                expected_version: original.revision,
                action: SessionAction::BeginResponse {
                    window_id: id,
                    intent_id: "late".into(),
                },
            }),
            6_000,
        )
        .unwrap();
    assert_eq!(expired.outcome, "rejected");
    assert_eq!(expired.error_code.as_deref(), Some("window_expired"));
    assert!(expired.changed);
    let next = RoomEnvelope::from_persisted(&expired.state).unwrap();
    assert_eq!(next.game.priority_team, 1);
    assert_eq!(next.game.stack.len(), 1);
    assert!(matches!(
        next.pacing.window.as_ref().unwrap().members[&1],
        Decision::Undecided {
            deadline_ms: 11_000
        }
    ));
    let no_cascade = next.transition(1, None, 6_000).unwrap();
    assert!(!no_cascade.changed);
    assert_eq!(
        serde_json::to_string(&original.replay_events(&expired.journal).unwrap()).unwrap(),
        expired.state
    );
}

#[test]
fn repeated_begin_keeps_epoch_and_composing_without_timeout_cancel_is_a_real_pass() {
    let mut room = fixture("duel");
    play_murder(&mut room);
    let id = window(&room);
    apply(
        &mut room,
        0,
        SessionAction::BeginResponse {
            window_id: id.clone(),
            intent_id: "draft".into(),
        },
        2_000,
    );
    apply(
        &mut room,
        0,
        SessionAction::BeginResponse {
            window_id: id.clone(),
            intent_id: "draft".into(),
        },
        500_000,
    );
    assert_eq!(window(&room), id);
    assert!(
        matches!(&room.pacing.window.as_ref().unwrap().members[&0],Decision::Composing{intent_id}if intent_id=="draft")
    );
    apply(
        &mut room,
        0,
        SessionAction::CancelAndPass {
            window_id: id.clone(),
            intent_id: "draft".into(),
        },
        500_001,
    );
    assert_eq!(room.game.priority_team, 1);
    assert_eq!(
        room.game.regions[0]
            .cards
            .iter()
            .filter(|c| c.definition == "JC042")
            .count(),
        2
    );
    let old = room
        .transition(
            0,
            Some(RoomCommand {
                command_id: "old".into(),
                expected_version: room.revision,
                action: SessionAction::BeginResponse {
                    window_id: id,
                    intent_id: "new".into(),
                },
            }),
            500_002,
        )
        .unwrap();
    assert_eq!(old.error_code.as_deref(), Some("window_expired"));
    assert!(!old.changed);
}

#[test]
fn teammates_begin_independently_but_formal_submit_changes_epoch_and_requires_requote() {
    let mut room = fixture("teams");
    play_murder(&mut room);
    let id = window(&room);
    let revision = room.revision;
    apply(
        &mut room,
        0,
        SessionAction::BeginResponse {
            window_id: id.clone(),
            intent_id: "a".into(),
        },
        2_000,
    );
    let before = room.clone();
    let t = room
        .transition(
            1,
            Some(RoomCommand {
                command_id: "parallel-b".into(),
                expected_version: revision,
                action: SessionAction::BeginResponse {
                    window_id: id.clone(),
                    intent_id: "b".into(),
                },
            }),
            2_001,
        )
        .unwrap();
    assert_eq!(t.outcome, "accepted");
    room = RoomEnvelope::from_persisted(&t.state).unwrap();
    assert_eq!(
        serde_json::to_string(&before.replay_events(&t.journal).unwrap()).unwrap(),
        t.state
    );
    let source = room.game.regions[0]
        .cards
        .iter()
        .find(|c| c.controller == 0 && c.definition == "JC042")
        .unwrap()
        .id
        .clone();
    let action = Action {
        card_id: Some(source),
        ..Action::new("activate")
    };
    let exact = serde_json::to_string(&room).unwrap();
    let quote = room
        .quote(
            0,
            QuoteRequest {
                window_id: id.clone(),
                intent_id: "a".into(),
                draft: Some(action.clone()),
            },
        )
        .unwrap();
    assert!(quote.ready);
    assert_eq!(serde_json::to_string(&room).unwrap(), exact);
    let formal_revision = room.revision;
    apply(
        &mut room,
        0,
        SessionAction::SubmitResponse {
            window_id: id.clone(),
            intent_id: "a".into(),
            action,
        },
        2_002,
    );
    assert_eq!(room.game.modifiers.len(), 1);
    assert_eq!(room.game.stack.len(), 1);
    assert_eq!(room.game.priority_team, 0);
    assert_ne!(window(&room), id);
    let stale = room
        .transition(
            1,
            Some(RoomCommand {
                command_id: "stale-b".into(),
                expected_version: formal_revision,
                action: SessionAction::SubmitResponse {
                    window_id: id.clone(),
                    intent_id: "b".into(),
                    action: Action::new("activate"),
                },
            }),
            2_003,
        )
        .unwrap();
    assert_eq!(stale.error_code.as_deref(), Some("version_conflict"));
    assert!(!stale.changed);
    assert!(room
        .quote(
            1,
            QuoteRequest {
                window_id: id,
                intent_id: "b".into(),
                draft: None
            }
        )
        .is_err());
}

#[test]
fn one_teammate_times_out_while_the_other_composes_without_global_pass_or_private_leak() {
    let mut room = fixture("teams");
    play_murder(&mut room);
    let id = window(&room);
    apply(
        &mut room,
        1,
        SessionAction::BeginResponse {
            window_id: id.clone(),
            intent_id: "private-draft".into(),
        },
        2_000,
    );
    let tick = room.transition(0, None, 6_000).unwrap();
    room = RoomEnvelope::from_persisted(&tick.state).unwrap();
    assert_eq!(room.game.priority_team, 0);
    assert!(room.game.passed.contains(&0));
    assert!(!room.game.passed.contains(&1));
    assert_eq!(window(&room), id);
    assert!(matches!(
        room.pacing.window.as_ref().unwrap().members[&1],
        Decision::Composing { .. }
    ));
    for seat in [0, 2, 3] {
        let v = room.view(seat, 6_000);
        assert!(v.response_window.as_ref().unwrap().my_intent_id.is_none());
        assert!(v.game.legal_actions.is_empty());
        assert!(!serde_json::to_string(&v).unwrap().contains("private-draft"));
    }
    let passed = room
        .transition(
            0,
            Some(RoomCommand {
                command_id: "repeat-pass".into(),
                expected_version: room.revision,
                action: SessionAction::BeginResponse {
                    window_id: id,
                    intent_id: "again".into(),
                },
            }),
            6_001,
        )
        .unwrap();
    assert_eq!(passed.error_code.as_deref(), Some("window_expired"));
}

#[test]
fn direct_paid_response_is_blocked_invalid_submit_does_not_pay_or_reset_and_pending_choice_has_no_clock(
) {
    let mut room = fixture("teams");
    play_murder(&mut room);
    let id = window(&room);
    let source = room.game.regions[0]
        .cards
        .iter()
        .find(|c| c.controller == 0 && c.definition == "JC042")
        .unwrap()
        .id
        .clone();
    let bypass = room
        .transition(
            0,
            Some(RoomCommand {
                command_id: "bypass".into(),
                expected_version: room.revision,
                action: Action {
                    card_id: Some(source),
                    ..Action::new("activate")
                }
                .into(),
            }),
            2_000,
        )
        .unwrap();
    assert_eq!(bypass.error_code.as_deref(), Some("invalid_action"));
    assert!(!bypass.changed);
    apply(
        &mut room,
        0,
        SessionAction::PassResponse {
            window_id: id.clone(),
        },
        2_000,
    );
    apply(
        &mut room,
        1,
        SessionAction::PassResponse { window_id: id },
        2_001,
    );
    let id = window(&room);
    apply(
        &mut room,
        2,
        SessionAction::BeginResponse {
            window_id: id.clone(),
            intent_id: "ritual".into(),
        },
        2_002,
    );
    let before = serde_json::to_string(&room).unwrap();
    let invalid = room
        .transition(
            2,
            Some(RoomCommand {
                command_id: "invalid".into(),
                expected_version: room.revision,
                action: SessionAction::SubmitResponse {
                    window_id: id.clone(),
                    intent_id: "ritual".into(),
                    action: Action::new("play"),
                },
            }),
            2_003,
        )
        .unwrap();
    assert_eq!(invalid.error_code.as_deref(), Some("invalid_action"));
    assert!(!invalid.changed);
    assert_eq!(invalid.state, before);
    let ritual = room.game.players[2].hand[0].id.clone();
    apply(
        &mut room,
        2,
        SessionAction::SubmitResponse {
            window_id: id,
            intent_id: "ritual".into(),
            action: Action {
                card_id: Some(ritual),
                target_id: Some("p0".into()),
                ..Action::new("play")
            },
        },
        2_004,
    );
    for seat in [2, 3, 0, 1] {
        let id = window(&room);
        apply(
            &mut room,
            seat,
            SessionAction::PassResponse { window_id: id },
            2_005,
        );
    }
    assert!(room.game.pending.is_some());
    assert!(room.pacing.window.is_none());
    assert_eq!(room.game.stack.len(), 1);
    let t = room.transition(0, None, 9_999_999).unwrap();
    assert!(!t.changed);
    let p = room.game.pending.clone().unwrap();
    let choose = Action {
        choice_id: Some(p.choice.id),
        selected: Some(vec![p.choice.options[0].id.clone()]),
        ..Action::new("choose")
    };
    apply(&mut room, p.seat, choose.into(), 9_999_999);
    assert!(room.game.pending.is_none());
    assert!(room.pacing.window.is_some());
    assert!(matches!(
        room.pacing.window.as_ref().unwrap().members[&0],
        Decision::Undecided {
            deadline_ms: 10_004_999
        }
    ));
}

#[test]
fn empty_stack_poll_has_no_revision_or_automatic_phase_advance_and_schema2_is_rejected() {
    let room = fixture("duel");
    let bytes = serde_json::to_string(&room).unwrap();
    let t = room.transition(0, None, 10_000_000).unwrap();
    assert!(!t.changed);
    assert_eq!(t.state, bytes);
    assert!(t.journal.is_empty());
    assert_eq!(t.view.server_now_ms, 10_000_000);
    assert!(RoomEnvelope::from_persisted(&serde_json::to_string(&room.game).unwrap()).is_err());
    assert!(room.replay_events(&[]).is_err());
}
