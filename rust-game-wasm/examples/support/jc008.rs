//! Explicit initial/checkpoint fixtures; every recorded change thereafter uses
//! the real Room ABI, persisted decode, accepted journal replay and all views.
use super::*;
use hegemony_server::model::Card;

fn board<'a>(g: &'a Game, id: &str) -> Option<(usize, &'a Card)> {
    g.regions
        .iter()
        .enumerate()
        .find_map(|(r, region)| region.cards.iter().find(|c| c.id == id).map(|c| (r, c)))
}

fn fund(g: &mut Game, seat: usize, def: &str, count: usize) {
    for _ in 0..count {
        let c = g.make_card(def, seat);
        g.players[seat].assets.push(c);
    }
}
fn held(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn action(source: &str, target: &str) -> Action {
    Action {
        card_id: Some(source.into()),
        target_id: Some(target.into()),
        ..Action::new("play")
    }
}
fn pre_resolve(g: &mut Game) {
    while !g.stack.is_empty() {
        let s = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(s, Action::new("pass")).unwrap();
    }
}
fn advance(room: &mut RoomEnvelope, steps: &mut Vec<Value>, done: impl Fn(&RoomEnvelope) -> bool) {
    for _ in 0..40 {
        if done(room) {
            return;
        }
        let s = (0..4)
            .find(|s| {
                room.legal_actions(*s)
                    .iter()
                    .any(|a| a.action.kind == "pass")
            })
            .unwrap();
        apply_game(room, steps, s, Action::new("pass"));
    }
    panic!("bounded JC008 window did not advance");
}
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
fn case(kind: &str) -> Value {
    let checkpoint = kind.starts_with("checkpoint-") || kind.starts_with("cleanup-");
    let bound = kind == "bound-target-reentered";
    let invalid = kind.starts_with("reject-");
    let owner = if matches!(kind, "first-team" | "shield-friend") {
        0
    } else {
        2
    };
    let def = if kind == "reject-barrier" {
        "JZ08"
    } else if matches!(
        kind,
        "first-team"
            | "rear-team"
            | "exhausted-character"
            | "checkpoint-controller"
            | "cleanup-wounds"
    ) {
        "JC004"
    } else if kind == "cleanup-death-trigger-reentry" {
        "XQ12"
    } else if kind.contains("response") || kind.contains("reveal") || kind.contains("reenter") {
        "JC003"
    } else {
        "JC125"
    };
    let mut g = attachment_initial("teams", "888888888888888888880008");
    if kind == "cleanup-discard-pause-rear-initiative" {
        g.first_team = 1;
    }
    let target = g.make_card(def, owner);
    let target_id = target.id.clone();
    g.regions[0].cards.push(target);
    if kind == "exhausted-character" {
        g.regions[0].cards[0].exhausted = true;
    }
    if kind == "reject-hidden" {
        g.regions[0].cards[0].face_down = true;
    }
    if kind.starts_with("shield-") {
        g.regions[0].cards[0].shield = 1;
    }
    fund(
        &mut g,
        0,
        if kind == "reject-loyalty" {
            "JC063"
        } else {
            "JC002"
        },
        if matches!(kind, "stack-two" | "cleanup-death-trigger-reentry") {
            4
        } else if kind == "reject-cost" {
            1
        } else {
            2
        },
    );
    let source = held(&mut g, "JC008", 0);
    let second = if matches!(kind, "stack-two" | "cleanup-death-trigger-reentry") {
        Some(held(&mut g, "JC008", 0))
    } else {
        None
    };
    let mut reply = None;
    if matches!(
        kind,
        "response-hide" | "response-return" | "hide-reveal" | "return-reenter"
    ) {
        fund(&mut g, 2, "JC003", 8);
        fund(&mut g, 2, "JC063", 1);
        reply = Some(held(
            &mut g,
            if matches!(kind, "response-hide" | "hide-reveal") {
                "JC063"
            } else {
                "JC006"
            },
            2,
        ));
    }
    let mut survivor = None;
    let mut current_target = target_id.clone();
    if checkpoint || bound {
        g.apply(0, action(&source, &target_id)).unwrap();
        if checkpoint {
            pre_resolve(&mut g);
            assert_eq!(g.turn_attribute_modifiers.len(), 1);
        }
        if bound {
            g.regions[0].cards.clear();
            let c = g.make_card(def, owner);
            current_target = c.id.clone();
            g.regions[0].cards.push(c);
        }
        match kind {
            "checkpoint-controller" => g.regions[0].cards[0].controller = 1,
            "checkpoint-movement" => {
                let c = g.regions[0].cards.remove(0);
                g.regions[1].cards.push(c);
            }
            "cleanup-simultaneous-survival"
            | "cleanup-discard-pause"
            | "cleanup-discard-pause-rear-initiative" => g.regions[0].cards[0].damage = 1,
            "cleanup-wounds" => {
                let c = &mut g.regions[0].cards[0];
                c.wounds = 1;
                c.damage = 1;
                c.shield = 2;
            }
            "cleanup-zero-defense" | "cleanup-death-trigger-reentry" => {
                g.regions[0].cards[0].wounds = catalog::card(def).defense.unwrap()
            }
            _ => {}
        }
        if kind.starts_with("cleanup-discard-pause") {
            for s in [0, 2] {
                for _ in 0..8 {
                    held(&mut g, "JC125", s);
                }
            }
        }
        if kind == "cleanup-death-trigger-reentry" {
            held(&mut g, "JC125", 0);
            let c = g.make_card("JC125", 2);
            survivor = Some(c.id.clone());
            g.regions[1].cards.push(c);
        }
        if checkpoint {
            g.window = Some(Window::End);
            g.priority_team = g.first_team;
            g.passed.clear();
            g.team_passed = [false; 2];
        }
    }
    let initial_turn = g.turn;
    let mut room = RoomEnvelope::from_game(g);
    if bound {
        room.pacing.window_seq = 1;
        room.pacing.window = Some(hegemony_server::room::PriorityWindow {
            id: "response:1".into(),
            stack_top_id: room.stack.last().unwrap().id.clone(),
            holder_team: room.priority_team,
            members: (0..4)
                .filter(|s| room.team(*s) == room.priority_team)
                .map(|s| {
                    (
                        s,
                        Decision::Undecided {
                            deadline_ms: 30_000,
                        },
                    )
                })
                .collect(),
        });
    }
    room = RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).unwrap();
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        0,
    )];
    let mut denied = vec![];
    if invalid {
        denied.push(rejected(&room, 0, action(&source, &target_id)));
        assert!(room.turn_attribute_modifiers.is_empty());
        assert!(room.players[0].assets.iter().all(|c| !c.exhausted));
    } else if checkpoint {
        advance(&mut room, &mut steps, |r| {
            r.turn > initial_turn || r.pending.is_some()
        });
        if kind.starts_with("cleanup-discard-pause") {
            for seat in if kind == "cleanup-discard-pause-rear-initiative" {
                [2, 0]
            } else {
                [0, 2]
            } {
                let p = room.pending.clone().unwrap();
                assert_eq!(p.seat, seat);
                assert_eq!(board(&room, &target_id).unwrap().1.damage, 1);
                assert_eq!(room.turn_attribute_modifiers.len(), 1);
                assert!(room
                    .effects
                    .iter()
                    .any(|e| matches!(e, Effect::FinishCleanup)));
                choose(&mut room, &mut steps, vec![p.choice.options[0].id.clone()]);
            }
            assert_eq!(
                (room.players[0].hand.len(), room.players[2].hand.len()),
                (7, 7)
            );
        } else if kind == "cleanup-death-trigger-reentry" {
            assert_eq!(room.turn, initial_turn);
            assert_eq!(room.first_team, 0);
            let p = room.pending.clone().unwrap();
            assert_eq!(p.choice.kind, "trigger");
            choose(&mut room, &mut steps, vec!["p0".into()]);
            attachment_pass_top(&mut room, &mut steps);
            let p = room.pending.clone().unwrap();
            assert_eq!(p.choice.kind, "discard");
            let ordinary = p
                .choice
                .options
                .iter()
                .find(|o| {
                    o.card
                        .as_ref()
                        .is_some_and(|c| c.card_id.as_deref() == Some("JC125"))
                })
                .unwrap();
            choose(&mut room, &mut steps, vec![ordinary.id.clone()]);
            assert_eq!(room.turn, initial_turn);
            let alive = survivor.as_ref().unwrap();
            apply_game(
                &mut room,
                &mut steps,
                0,
                action(second.as_ref().unwrap(), alive),
            );
            attachment_pass_top(&mut room, &mut steps);
            assert_eq!(room.turn_attribute_modifiers.len(), 1);
            advance(&mut room, &mut steps, |r| r.turn > initial_turn);
            assert_eq!(room.defense(board(&room, alive).unwrap().1, 1), 1);
        }
        assert_eq!(room.turn, initial_turn + 1);
        assert!(room.turn_attribute_modifiers.is_empty());
        if matches!(
            kind,
            "cleanup-zero-defense" | "cleanup-death-trigger-reentry"
        ) {
            assert!(board(&room, &target_id).is_none());
            assert!(room.players[owner]
                .graveyard
                .iter()
                .any(|c| c.definition == def));
        } else {
            let (r, c) =
                board(&room, &target_id).expect("survivor must retain its original instance");
            assert_eq!(c.damage, 0);
            assert_eq!(
                room.defense(c, r),
                catalog::card(def).defense.unwrap().saturating_sub(c.wounds)
            );
            if kind == "cleanup-wounds" {
                assert_eq!((c.wounds, c.shield), (1, 2));
            }
            if kind == "checkpoint-controller" {
                assert_eq!((c.owner, c.controller), (2, 1));
            }
            if kind == "checkpoint-movement" {
                assert_eq!(r, 1);
            }
        }
    } else {
        if !bound {
            apply_game(&mut room, &mut steps, 0, action(&source, &target_id));
        }
        if kind == "stack-two" {
            apply_game(
                &mut room,
                &mut steps,
                0,
                action(second.as_ref().unwrap(), &target_id),
            );
        }
        if matches!(kind, "response-hide" | "response-return") {
            apply_game(&mut room, &mut steps, 0, Action::new("pass"));
            apply_game(&mut room, &mut steps, 1, Action::new("pass"));
            let mut a = action(reply.as_ref().unwrap(), &target_id);
            if kind == "response-hide" {
                a.option = Some("hide".into());
            }
            apply_game(&mut room, &mut steps, 2, a);
            attachment_pass_top(&mut room, &mut steps);
        }
        attachment_pass_top(&mut room, &mut steps);
        if kind == "stack-two" {
            assert_eq!(room.turn_attribute_modifiers.len(), 1);
            attachment_pass_top(&mut room, &mut steps);
        }
        let cancelled =
            bound || matches!(kind, "shield-enemy" | "response-hide" | "response-return");
        if cancelled {
            assert!(room.turn_attribute_modifiers.is_empty());
        } else {
            let (r, c) = board(&room, &target_id).unwrap();
            let count = if kind == "stack-two" { 2 } else { 1 };
            assert_eq!(room.turn_attribute_modifiers.len(), count);
            assert_eq!(
                room.defense(c, r),
                catalog::card(def).defense.unwrap() + count as u32
            );
            let printed = catalog::card(def);
            let combat = printed.permanent_icons.combat
                + if room.team(c.controller) == room.first_team {
                    printed.temporary_icons.combat
                } else {
                    0
                }
                + count as u32;
            assert_eq!(room.current_icons(c, r).combat, combat);
            assert_eq!(
                room.icons(c, r).combat,
                if c.exhausted { 0 } else { combat }
            );
            assert!(room
                .turn_attribute_modifiers
                .iter()
                .all(|m| m.target_instance == target_id && m.expires_turn == initial_turn));
        }
        if kind.starts_with("shield-") {
            assert_eq!(
                board(&room, &target_id).unwrap().1.shield,
                u32::from(kind == "shield-friend")
            );
        }
        if matches!(kind, "hide-reveal" | "return-reenter") {
            apply_game(&mut room, &mut steps, 0, Action::new("pass"));
            apply_game(&mut room, &mut steps, 1, Action::new("pass"));
            let mut a = action(reply.as_ref().unwrap(), &target_id);
            if kind == "hide-reveal" {
                a.option = Some("hide".into());
            }
            apply_game(&mut room, &mut steps, 2, a);
            attachment_pass_top(&mut room, &mut steps);
            assert!(room.turn_attribute_modifiers.is_empty());
            assert!(board(&room, &target_id).is_none());
            if kind == "hide-reveal" {
                current_target = room.regions[0]
                    .cards
                    .iter()
                    .find(|c| c.face_down && c.definition == def)
                    .unwrap()
                    .id
                    .clone();
                assert_ne!(current_target, target_id);
                for seat in 0..4 {
                    let v = room.view(seat, 0);
                    let c = v.regions[0]
                        .characters
                        .iter()
                        .find(|c| c.instance_id == current_target)
                        .unwrap();
                    assert_eq!(
                        c.card_id.as_deref(),
                        if seat == 2 { Some(def) } else { None }
                    );
                }
                apply_game(&mut room, &mut steps, 0, Action::new("pass"));
                apply_game(&mut room, &mut steps, 1, Action::new("pass"));
                apply_game(
                    &mut room,
                    &mut steps,
                    2,
                    Action {
                        card_id: Some(current_target.clone()),
                        ..Action::new("reveal")
                    },
                );
            } else {
                advance(&mut room, &mut steps, |r| {
                    matches!(r.window, Some(Window::Action(1)))
                });
                let held = room.players[2]
                    .hand
                    .iter()
                    .find(|c| c.definition == def)
                    .unwrap()
                    .id
                    .clone();
                apply_game(
                    &mut room,
                    &mut steps,
                    2,
                    Action {
                        card_id: Some(held),
                        region: Some(0),
                        ..Action::new("deploy")
                    },
                );
            }
            attachment_pass_top(&mut room, &mut steps);
            let c = room.regions[0]
                .cards
                .iter()
                .find(|c| !c.face_down && c.definition == def)
                .unwrap();
            current_target = c.id.clone();
            assert_ne!(current_target, target_id);
            assert_eq!(room.defense(c, 0), catalog::card(def).defense.unwrap());
            assert_eq!(room.current_icons(c, 0).combat, 0);
        }
        assert_eq!(
            room.players[0]
                .assets
                .iter()
                .filter(|c| c.exhausted)
                .count(),
            if kind == "stack-two" { 4 } else { 2 }
        );
        assert!(room.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC008"));
    }
    if steps.len() > 1 {
        let last = steps.last().unwrap();
        let command: RoomCommand = serde_json::from_value(last["args"][1].clone()).unwrap();
        let seat = last["args"][0].as_u64().unwrap() as usize;
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
    }
    denied.push(rejected(&room, 1, action(&source, &current_target)));
    json!({"name":format!("JC008-{kind}"),"seed":"9007199254740993","steps":steps,"rejectedCommands":denied,
        "syntheticInitialLayout":true,"syntheticInitialResolvedModifier":checkpoint,"syntheticBoundFrame":bound,
        "syntheticControllerChange":kind=="checkpoint-controller","syntheticMovementCheckpoint":kind=="checkpoint-movement",
        "syntheticDamageOrWounds":kind.starts_with("cleanup-"),"normalRoomCommandsAfterInitial":true,"publicNaturalUiAcceptance":false})
}
pub(super) fn cases() -> Vec<Value> {
    [
        "first-team",
        "rear-team",
        "exhausted-character",
        "stack-two",
        "reject-hidden",
        "reject-barrier",
        "reject-cost",
        "reject-loyalty",
        "shield-enemy",
        "shield-friend",
        "response-hide",
        "response-return",
        "hide-reveal",
        "return-reenter",
        "bound-target-reentered",
        "checkpoint-controller",
        "checkpoint-movement",
        "cleanup-simultaneous-survival",
        "cleanup-wounds",
        "cleanup-zero-defense",
        "cleanup-discard-pause",
        "cleanup-discard-pause-rear-initiative",
        "cleanup-death-trigger-reentry",
    ]
    .into_iter()
    .map(case)
    .collect()
}
