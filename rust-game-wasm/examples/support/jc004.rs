//! Synthetic local starting layouts; subsequent commands use the real room ABI.
//! No public natural-play or newly admitted response mechanism is claimed.
use super::*;

fn choose(room: &mut RoomEnvelope, steps: &mut Vec<Value>, selected: Vec<String>) {
    let p = room.pending.clone().unwrap();
    apply_game(
        room,
        steps,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(selected),
            ..Action::new("choose")
        },
    );
}
fn game_pass_top(game: &mut Game) {
    let count = game.stack.len();
    for _ in 0..32 {
        if game.stack.len() < count || game.pending.is_some() {
            return;
        }
        let seat = (0..game.players.len())
            .find(|s| {
                game.legal_actions(*s)
                    .iter()
                    .any(|a| a.action.kind == "pass")
            })
            .unwrap();
        game.apply(seat, Action::new("pass")).unwrap();
    }
    panic!("bounded stack did not progress");
}
fn case(kind: &str) -> Value {
    let mut game = attachment_initial("teams", "888888888888888888880004");
    let mut target = game.make_card("JC004", 2);
    target.exhausted = matches!(
        kind,
        "exhausted"
            | "borrowed-return"
            | "continuation-return"
            | "declared-exhausted-resolved-ready"
    );
    if kind == "borrowed-return" {
        target.controller = 1;
        target.wounds = 1;
        target.shield = 2;
    }
    let target_id = target.id.clone();
    game.regions[0].cards.push(target);
    let mut hidden = game.make_card("JC007", 1);
    hidden.face_down = true;
    hidden.controller = 3;
    let hidden_id = hidden.id.clone();
    game.regions[0].cards.push(hidden);
    let barrier = game.make_card("JZ08", 2);
    let barrier_id = barrier.id.clone();
    game.regions[0].cards.push(barrier);
    let mut equipment_id = None;
    if kind == "borrowed-return" {
        let equipment = game.make_card("BQ022", 3);
        equipment_id = Some(equipment.id.clone());
        game.attachments.push(Attachment {
            card: equipment,
            host_id: target_id.clone(),
        });
    }
    let guard = game.make_card("JC003", 2);
    let guard_id = guard.id.clone();
    game.regions[1].cards.push(guard);
    for seat in [0, 2] {
        for _ in 0..4 {
            let c = game.make_card("JC003", seat);
            game.players[seat].assets.push(c);
        }
    }
    let mut hide_id = None;
    if kind == "response-hide" {
        let c = game.make_card("JC063", 2);
        hide_id = Some(c.id.clone());
        game.players[2].hand.push(c);
        for _ in 0..2 {
            let c = game.make_card("JC063", 2);
            game.players[2].assets.push(c);
        }
    }
    let source = game.make_card("JC004", 0);
    let hand_id = source.id.clone();
    game.players[0].hand.push(source);
    // Start before deployment for ordinary branch/payment/response cases.
    // Identity/movement/continuation cases intentionally start at a bound frame;
    // their isolated mutations occur before the first recorded ABI state.
    let bound_layout = matches!(
        kind,
        "old-instance-reentered"
            | "moved-away"
            | "moved-back"
            | "hidden-before-guard"
            | "shield-before-guard"
            | "continuation-return"
            | "continuation-exhaust"
            | "declared-ready-resolved-exhausted"
            | "declared-exhausted-resolved-ready"
    );
    let mut resolved_source = None;
    let mut current_target = target_id.clone();
    if bound_layout {
        game.apply(
            0,
            Action {
                card_id: Some(hand_id.clone()),
                region: Some(0),
                ..Action::new("deploy")
            },
        )
        .unwrap();
        game_pass_top(&mut game);
        resolved_source = Some(
            game.regions[0]
                .cards
                .iter()
                .find(|c| c.owner == 0 && c.definition == "JC004")
                .unwrap()
                .id
                .clone(),
        );
        let p = game.pending.clone().unwrap();
        game.apply(
            0,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![target_id.clone()]),
                ..Action::new("choose")
            },
        )
        .unwrap();
        match kind {
            "declared-ready-resolved-exhausted" => {
                game.regions[0]
                    .cards
                    .iter_mut()
                    .find(|c| c.id == target_id)
                    .unwrap()
                    .exhausted = true;
            }
            "declared-exhausted-resolved-ready" => {
                game.regions[0]
                    .cards
                    .iter_mut()
                    .find(|c| c.id == target_id)
                    .unwrap()
                    .exhausted = false;
            }
            "old-instance-reentered" => {
                let index = game.regions[0]
                    .cards
                    .iter()
                    .position(|c| c.id == target_id)
                    .unwrap();
                let previous = game.regions[0].cards.remove(index);
                let fresh = game.make_card(&previous.definition, previous.owner);
                current_target = fresh.id.clone();
                game.regions[0].cards.push(fresh);
            }
            "moved-away" | "moved-back" => {
                let index = game.regions[0]
                    .cards
                    .iter()
                    .position(|c| c.id == target_id)
                    .unwrap();
                let c = game.regions[0].cards.remove(index);
                game.regions[1].cards.push(c);
                if kind == "moved-back" {
                    let c = game.regions[1].cards.pop().unwrap();
                    game.regions[0].cards.push(c);
                }
            }
            "hidden-before-guard" => {
                game.regions[0]
                    .cards
                    .iter_mut()
                    .find(|c| c.id == target_id)
                    .unwrap()
                    .face_down = true
            }
            "shield-before-guard" => {
                game.regions[0]
                    .cards
                    .iter_mut()
                    .find(|c| c.id == target_id)
                    .unwrap()
                    .shield = 1
            }
            "continuation-return" | "continuation-exhaust" => {
                game.stack
                    .last_mut()
                    .unwrap()
                    .frame
                    .as_mut()
                    .unwrap()
                    .steps
                    .push(hegemony_server::model::Step {
                        context: 0,
                        op: Op::Forecast {
                            player: rules::PlayerRef::Actor,
                            count: 2,
                        },
                    });
            }
            _ => unreachable!(),
        }
    }
    let mut room = RoomEnvelope::from_game(game);
    if bound_layout {
        // This synthetic checkpoint starts with a bound stack frame, so provide
        // its matching formal room window before the ABI attempts restoration.
        // Production declarations create this via ordinary room transitions.
        room.pacing.window_seq = 1;
        room.pacing.window = Some(hegemony_server::room::PriorityWindow {
            id: "response:1".into(),
            stack_top_id: room.stack.last().unwrap().id.clone(),
            holder_team: room.priority_team,
            members: (0..4)
                .filter(|seat| room.team(*seat) == room.priority_team)
                .map(|seat| {
                    (
                        seat,
                        Decision::Undecided {
                            deadline_ms: 30_000,
                        },
                    )
                })
                .collect(),
        });
    }
    RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).unwrap();
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        0,
    )];
    for seat in 0..4 {
        let view = room.view(seat, 0);
        let hidden = view.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == hidden_id)
            .unwrap();
        assert_eq!(
            hidden.card_id.as_deref(),
            if seat == 3 { Some("JC007") } else { None }
        );
    }
    let mut source_id = resolved_source;
    let mut denied = vec![];
    if !bound_layout {
        apply_game(
            &mut room,
            &mut steps,
            0,
            Action {
                card_id: Some(hand_id.clone()),
                region: Some(0),
                ..Action::new("deploy")
            },
        );
        assert_eq!(
            room.players[0]
                .assets
                .iter()
                .filter(|c| c.exhausted)
                .count(),
            4
        );
        attachment_pass_top(&mut room, &mut steps);
        source_id = Some(
            room.regions[0]
                .cards
                .iter()
                .find(|c| c.owner == 0 && c.definition == "JC004")
                .unwrap()
                .id
                .clone(),
        );
        let p = room.pending.clone().unwrap();
        assert!(p.choice.options.iter().any(|o| o.id == target_id));
        assert!(p
            .choice
            .options
            .iter()
            .any(|o| Some(&o.id) == source_id.as_ref()));
        assert!(p
            .choice
            .options
            .iter()
            .all(|o| o.id != hidden_id && o.id != barrier_id));
        for id in [&hidden_id, &barrier_id] {
            let command = RoomCommand {
                command_id: format!("invalid-{id}"),
                expected_version: room.revision,
                action: Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(vec![id.clone()]),
                    ..Action::new("choose")
                }
                .into(),
            };
            let now = room.pacing.last_server_now_ms;
            let rejected = room.transition(0, Some(command.clone()), now).unwrap();
            assert_eq!(rejected.outcome, "rejected");
            assert!(!rejected.changed);
            let mut entry = step(&room, "applyRoom", json!([0, command, now.to_string()]), 0);
            entry["view"] = serde_json::to_value(&rejected.view).unwrap();
            entry["transition"] = serde_json::to_value(rejected).unwrap();
            steps.push(entry);
        }
        let selected = if kind == "decline" {
            vec![]
        } else if kind == "self-target" {
            vec![source_id.clone().unwrap()]
        } else {
            vec![target_id.clone()]
        };
        choose(&mut room, &mut steps, selected);
        if kind == "decline" {
            assert!(room.stack.is_empty());
            assert!(
                !room.regions[0]
                    .cards
                    .iter()
                    .find(|c| c.id == target_id)
                    .unwrap()
                    .exhausted
            );
            return json!({"name":"JC004-optional-decline", "seed":"9007199254740993", "steps":steps,
                "syntheticInitialLayout":true,"publicNaturalUiAcceptance":false});
        }
        assert!(room
            .stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .already_paid
            .is_empty());
    }
    if kind == "response-exhaust" || kind == "response-hide" {
        apply_game(&mut room, &mut steps, 0, Action::new("pass"));
        apply_game(&mut room, &mut steps, 1, Action::new("pass"));
        let action = if kind == "response-exhaust" {
            Action {
                card_id: Some(guard_id),
                target_id: Some(target_id.clone()),
                ability_id: Some("exhaust".into()),
                ..Action::new("activate")
            }
        } else {
            Action {
                card_id: hide_id,
                target_id: Some(target_id.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            }
        };
        apply_game(&mut room, &mut steps, 2, action);
        attachment_pass_top(&mut room, &mut steps);
        if kind == "response-exhaust" {
            assert!(
                room.regions[0]
                    .cards
                    .iter()
                    .find(|c| c.id == target_id)
                    .unwrap()
                    .exhausted
            );
        } else {
            let c = room.regions[0]
                .cards
                .iter()
                .find(|c| c.owner == 2 && c.definition == "JC004")
                .unwrap();
            assert!(c.face_down);
            assert_ne!(c.id, target_id);
            current_target = c.id.clone();
        }
    }
    attachment_pass_top(&mut room, &mut steps);
    if kind.starts_with("continuation-") {
        let p = room.pending.clone().unwrap();
        if let ChoiceResolution::Frame { frame, .. } = &p.resolution {
            assert_eq!(frame.cursor, 3);
        } else {
            panic!("expected frame continuation");
        }
        apply_game(
            &mut room,
            &mut steps,
            0,
            Action {
                choice_id: Some(p.choice.id),
                top: Some(p.choice.options.iter().map(|o| o.id.clone()).collect()),
                bottom: Some(vec![]),
                ..Action::new("choose")
            },
        );
    }
    let returned = matches!(
        kind,
        "exhausted"
            | "response-exhaust"
            | "borrowed-return"
            | "continuation-return"
            | "declared-ready-resolved-exhausted"
    );
    if returned {
        assert!(room
            .regions
            .iter()
            .all(|r| r.cards.iter().all(|c| c.id != target_id)));
        let c = room.players[2]
            .hand
            .iter()
            .find(|c| c.definition == "JC004")
            .unwrap();
        assert_ne!(c.id, target_id);
        assert_eq!((c.owner, c.controller), (2, 2));
        assert!(!c.face_down && !c.exhausted);
        assert_eq!((c.damage, c.wounds, c.shield), (0, 0, 0));
        if let Some(equipment_id) = equipment_id {
            assert!(room.attachments.is_empty());
            assert!(room.players[3]
                .hand
                .iter()
                .any(|c| c.definition == "BQ022" && c.id != equipment_id));
        }
    } else {
        let c = room
            .regions
            .iter()
            .flat_map(|r| &r.cards)
            .find(|c| c.id == current_target)
            .unwrap();
        let cancelled = matches!(
            kind,
            "old-instance-reentered"
                | "moved-away"
                | "hidden-before-guard"
                | "shield-before-guard"
                | "response-hide"
        );
        if kind == "self-target" {
            assert!(
                room.regions[0]
                    .cards
                    .iter()
                    .find(|c| Some(&c.id) == source_id.as_ref())
                    .unwrap()
                    .exhausted
            );
            assert!(!c.exhausted);
        } else {
            assert_eq!(c.exhausted, !cancelled);
        }
        if kind == "shield-before-guard" {
            assert_eq!(c.shield, 0);
        }
        assert!(room.players[2].hand.is_empty());
    }
    assert_eq!(
        room.players[0]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        4
    );
    // The last effect-committing command survives persistence and retry without
    // producing another zone transition, payment, branch query or choice.
    let last = steps.last().unwrap();
    let seat = last["args"][0].as_u64().unwrap() as usize;
    let command: RoomCommand = serde_json::from_value(last["args"][1].clone()).unwrap();
    let now = last["args"][2].as_str().unwrap().parse::<u64>().unwrap();
    let retry = room.transition(seat, Some(command.clone()), now).unwrap();
    assert!(!retry.changed);
    assert_eq!(retry.state, serde_json::to_string(&room).unwrap());
    record_transition(
        &mut room,
        &mut steps,
        "applyRoom",
        json!([seat, command, now.to_string()]),
        retry,
    );
    denied.push(rejected(
        &room,
        0,
        Action {
            card_id: Some(hand_id),
            region: Some(0),
            ..Action::new("deploy")
        },
    ));
    json!({"name":format!("JC004-{kind}"),"seed":"9007199254740993","steps":steps,
        "rejectedCommands":denied,"syntheticInitialLayout":true,"syntheticBoundFrame":bound_layout,
        "publicNaturalUiAcceptance":false,"restoreAndRetryVerified":true,
        "syntheticStateChangeBeforeResolution":kind.starts_with("declared-")})
}

pub(super) fn cases() -> Vec<Value> {
    [
        "ready",
        "exhausted",
        "response-exhaust",
        "response-hide",
        "old-instance-reentered",
        "moved-away",
        "moved-back",
        "hidden-before-guard",
        "shield-before-guard",
        "borrowed-return",
        "self-target",
        "continuation-return",
        "continuation-exhaust",
        "decline",
        "declared-ready-resolved-exhausted",
        "declared-exhausted-resolved-ready",
    ]
    .into_iter()
    .map(case)
    .collect()
}
