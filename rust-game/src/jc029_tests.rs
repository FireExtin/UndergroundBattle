//! JC029 source-complete reuse tests. Layouts are explicit native fixtures,
//! never a public UI playtest or server-state injection.
use crate::{catalog, model::*, room::RoomEnvelope, rules::*};
use std::sync::atomic::{AtomicUsize, Ordering};
static SEQ: AtomicUsize = AtomicUsize::new(0);

pub(super) fn envelope(g: &Game) -> RoomEnvelope {
    // Explicit valid session layout for a native rule fixture; this is not a
    // natural room continuation. The production persisted-state guard remains.
    let mut r = RoomEnvelope::from_game(g.clone());
    if let Some(top) = g
        .stack
        .last()
        .filter(|_| g.status == "playing" && g.pending.is_none())
    {
        r.pacing.window_seq = 1;
        r.pacing.window = Some(crate::room::PriorityWindow {
            id: "response:1".into(),
            stack_top_id: top.id.clone(),
            holder_team: g.priority_team,
            members: g
                .players
                .iter()
                .filter(|p| !p.eliminated && g.team(p.seat) == g.priority_team)
                .map(|p| {
                    (
                        p.seat,
                        if g.passed.contains(&p.seat) {
                            crate::room::Decision::Passed
                        } else {
                            crate::room::Decision::Undecided {
                                deadline_ms: crate::room::INTENT_DURATION_MS,
                            }
                        },
                    )
                })
                .collect(),
        });
    }
    RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap()
}
pub(super) fn checkpoint(g: &Game) {
    let room = envelope(g);
    let state = serde_json::to_string(&room).unwrap();
    let restored = RoomEnvelope::from_persisted(&state).unwrap();
    let views = (0..4)
        .map(|s| serde_json::to_value(room.view(s, 0)).unwrap())
        .collect::<Vec<_>>();
    for s in 0..4 {
        assert_eq!(views[s], serde_json::to_value(restored.view(s, 0)).unwrap());
    }
    if let Ok(dir) =
        std::env::var("GREEN_EVIDENCE_DIR").or_else(|_| std::env::var("JC029_EVIDENCE_DIR"))
    {
        std::fs::create_dir_all(&dir).unwrap();
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        std::fs::write(
            format!("{dir}/checkpoint-{n:04}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state,"views":views})).unwrap(),
        )
        .unwrap();
    }
}
pub(super) fn apply(g: &mut Game, seat: usize, a: Action) {
    checkpoint(g);
    if let Ok(dir) =
        std::env::var("GREEN_EVIDENCE_DIR").or_else(|_| std::env::var("JC029_EVIDENCE_DIR"))
    {
        let mut room = envelope(g);
        let session = if a.kind == "pass" && room.pacing.window.is_some() {
            crate::room::SessionAction::PassResponse {
                window_id: room.pacing.window.as_ref().unwrap().id.clone(),
            }
        } else if let Some(window) = room.pacing.window.clone() {
            // A real responsive action must use the production intent protocol.
            // Record BeginResponse and SubmitResponse as separate whole native
            // transitions; never synthesize the composing decision in state.
            let intent_id = format!("native-intent-{}", SEQ.load(Ordering::Relaxed));
            let begin = crate::room::RoomCommand {
                command_id: format!("native-begin-{}", SEQ.load(Ordering::Relaxed)),
                expected_version: room.revision,
                action: crate::room::SessionAction::BeginResponse {
                    window_id: window.id.clone(),
                    intent_id: intent_id.clone(),
                },
            };
            let state = serde_json::to_string(&room).unwrap();
            let expected = room.transition(seat, Some(begin.clone()), 0).unwrap();
            assert!(
                expected.error_code.is_none(),
                "{:?}",
                expected.error_message
            );
            room = RoomEnvelope::from_persisted(&expected.state).unwrap();
            let views = (0..4)
                .map(|s| serde_json::to_value(room.view(s, 0)).unwrap())
                .collect::<Vec<_>>();
            let n = SEQ.fetch_add(1, Ordering::Relaxed);
            std::fs::write(format!("{dir}/step-{n:04}.json"),serde_json::to_vec(&serde_json::json!({"state":state,"seat":seat,"command":begin,"expected":expected,"views":views})).unwrap()).unwrap();
            crate::room::SessionAction::SubmitResponse {
                window_id: window.id,
                intent_id,
                action: a.clone(),
            }
        } else {
            crate::room::SessionAction::Game { action: a.clone() }
        };
        let command = crate::room::RoomCommand {
            command_id: format!("jc029-step-{}", SEQ.load(Ordering::Relaxed)),
            expected_version: room.revision,
            action: session,
        };
        let state = serde_json::to_string(&room).unwrap();
        let expected = room.transition(seat, Some(command.clone()), 0).unwrap();
        assert!(
            expected.error_code.is_none(),
            "{}",
            expected.error_message.clone().unwrap_or_default()
        );
        let after = RoomEnvelope::from_persisted(&expected.state).unwrap();
        let views = (0..4)
            .map(|s| serde_json::to_value(after.view(s, 0)).unwrap())
            .collect::<Vec<_>>();
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        std::fs::write(format!("{dir}/step-{n:04}.json"),serde_json::to_vec(&serde_json::json!({"state":state,"seat":seat,"command":command,"expected":expected,"views":views})).unwrap()).unwrap();
    }
    g.apply(seat, a).unwrap();
    checkpoint(g);
}
pub(super) fn game(actor: usize) -> Game {
    let mut g = Game::new(
        "jc029-native".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        9,
    )
    .unwrap();
    for s in 1..4 {
        g.join(format!("P{s}"), "watchers".into()).unwrap();
    }
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
    while let Some(p) = g.pending.clone() {
        g.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.cards.clear();
    }
    g.first_team = g.team(actor);
    g.begin_window(Window::Action(g.team(actor)));
    g
}
pub(super) fn board(g: &mut Game, def: &str, owner: usize, region: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.regions[region].cards.push(c);
    id
}
pub(super) fn fund(g: &mut Game, actor: usize, def: &str, count: usize) {
    for _ in 0..count {
        let c = g.make_card(def, actor);
        g.players[actor].assets.push(c);
    }
}
fn remove_board(g: &mut Game, id: &str) -> Card {
    let (r, i) = g
        .regions
        .iter()
        .enumerate()
        .find_map(|(r, x)| x.cards.iter().position(|c| c.id == id).map(|i| (r, i)))
        .unwrap();
    g.regions[r].cards.remove(i)
}
pub(super) fn pass_top(g: &mut Game) {
    let count = g.stack.len();
    assert!(count > 0);
    for _ in 0..32 {
        if g.stack.len() < count || g.pending.is_some() {
            return;
        }
        let s = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        apply(g, s, Action::new("pass"));
    }
    panic!("top did not resolve");
}
pub(super) fn choose(g: &mut Game, selected: Vec<String>) {
    let p = g.pending.clone().unwrap();
    apply(
        g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(selected),
            ..Action::new("choose")
        },
    );
}
fn reveal(g: &mut Game, actor: usize, exhausted: bool) -> String {
    let id = board(g, "JC029", actor, 0);
    let c = g.board_mut(&id).unwrap();
    c.face_down = true;
    c.exhausted = exhausted;
    fund(g, actor, "XQ12", 2);
    apply(
        g,
        actor,
        Action {
            card_id: Some(id.clone()),
            ..Action::new("reveal")
        },
    );
    assert!(g.pending.is_none());
    assert_eq!(g.resources(actor), 0);
    pass_top(g);
    let c = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC029" && c.owner == actor)
        .unwrap();
    assert_ne!(c.id, id);
    assert!(!c.face_down);
    assert_eq!(c.exhausted, exhausted);
    c.id.clone()
}
pub(super) fn reject(g: &mut Game, seat: usize, a: Action) {
    let before = serde_json::to_string(g).unwrap();
    let error = g.apply(seat, a.clone()).unwrap_err();
    assert_eq!(serde_json::to_string(g).unwrap(), before);
    if let Ok(dir) =
        std::env::var("GREEN_EVIDENCE_DIR").or_else(|_| std::env::var("JC029_EVIDENCE_DIR"))
    {
        std::fs::create_dir_all(&dir).unwrap();
        let room = envelope(g);
        let command = crate::room::RoomCommand {
            command_id: format!("focused-reject-{}", SEQ.load(Ordering::Relaxed)),
            expected_version: room.revision,
            action: crate::room::SessionAction::Game { action: a },
        };
        let state = serde_json::to_string(&room).unwrap();
        let expected = room.transition(seat, Some(command.clone()), 0).unwrap();
        assert!(expected.error_code.is_some());
        assert_eq!(state, expected.state);
        let views = (0..4)
            .map(|s| serde_json::to_value(room.view(s, 0)).unwrap())
            .collect::<Vec<_>>();
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        std::fs::write(format!("{dir}/reject-{n:04}.json"),serde_json::to_vec(&serde_json::json!({"state":state,"seat":seat,"command":command,"expected":expected,"views":views,"nativeRuleError":error})).unwrap()).unwrap();
    }
}

#[test]
fn jc029_whole_original_and_exact_reused_ability() {
    let d = catalog::card("JC029");
    assert_eq!(
        (&*d.name, &*d.kind, &*d.color, d.cost, &*d.magic),
        ("新生血族", "character", "蓝", 2, "鲜血")
    );
    assert_eq!(d.loyalty, ["蓝色", "蓝色"]);
    assert_eq!(d.subtypes, ["吸血鬼"]);
    assert_eq!(d.defense, Some(1));
    assert_eq!(
        d.permanent_icons,
        Icons {
            influence: 1,
            ..Icons::default()
        }
    );
    assert_eq!(
        d.temporary_icons,
        Icons {
            combat: 1,
            ..Icons::default()
        }
    );
    assert!(!d.unique);
    assert_eq!(d.keywords, ["袭击1"]);
    assert_eq!(d.deck_copy_limit, Some(3));
    let a = &definition("JC029").abilities[0];
    assert_eq!(a.key, "raid-1");
    assert_eq!(a.event, Some(Event::Reveal));
    assert_eq!(a.timing, Timing::Fast);
    assert_eq!(a.response_policy, ResponsePolicy::Respondable);
    assert!(a.costs.is_empty() && a.modes.is_empty() && !a.requires_ready_source);
    assert_eq!(a.targets.len(), 1);
    assert_eq!(a.targets[0].range, Range::SourceRegion);
    assert_eq!(a.targets[0].relation, Relation::Any);
    assert_eq!(a.targets[0].kind, EntityKind::Character);
    assert!(matches!(
        a.ops.as_slice(),
        [Op::DamageTarget { slot: 0, amount: 1 }]
    ));
    assert_eq!(catalog::catalog().cards.len(), 93);
    assert!(crate::society::definition("MSJC03").is_err());
}

#[test]
fn jc029_paid_faceup_has_no_raid_conceal_waits_and_loyalty_failure_is_atomic() {
    let mut g = game(0);
    let c = g.make_card("JC029", 0);
    let id = c.id.clone();
    g.players[0].hand.push(c);
    fund(&mut g, 0, "JC125", 2);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(0),
            ..Action::new("conceal")
        },
    );
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.resources(0), 1);
    let source = g.regions[0].cards[0].id.clone();
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    );
    let mut g = game(0);
    fund(&mut g, 0, "XQ12", 2);
    let c = g.make_card("JC029", 0);
    let id = c.id.clone();
    g.players[0].hand.push(c);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.resources(0), 0);
}

#[test]
fn jc029_optional_local_any_controller_for_all_four_actors_and_orientation() {
    for actor in 0..4 {
        for exhausted in [false, true] {
            let mut g = game(actor);
            let own = board(&mut g, "JC059", actor, 0);
            let other = board(&mut g, "JC059", (actor + 1) % 4, 0);
            let ally = board(&mut g, "JC059", (actor + 2) % 4, 0);
            let distant = board(&mut g, "JC059", (actor + 1) % 4, 1);
            let hidden = board(&mut g, "JC059", (actor + 1) % 4, 0);
            g.board_mut(&hidden).unwrap().face_down = true;
            let source = reveal(&mut g, actor, exhausted);
            let p = g.pending.as_ref().unwrap();
            assert_eq!(p.seat, actor);
            assert_eq!(p.choice.allow_decline, Some(true));
            let ids = p
                .choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<Vec<_>>();
            for id in [&own, &other, &ally, &source] {
                assert!(ids.contains(id));
            }
            for id in [&distant, &hidden] {
                assert!(!ids.contains(id));
            }
            for viewer in 0..4 {
                let v = g.view(viewer);
                assert_eq!(v.pending_choice.is_some(), viewer == actor);
            }
            choose(&mut g, vec![]);
            assert!(g.stack.is_empty() && g.pending.is_none());
            assert_eq!(g.board(&other).unwrap().1.damage, 0);
        }
    }
}

#[test]
fn jc029_declared_raid_responds_one_damage_or_self_death_and_no_wound() {
    let mut g = game(0);
    let target = board(&mut g, "JC059", 1, 0);
    reveal(&mut g, 0, false);
    choose(&mut g, vec![target.clone()]);
    assert_eq!(g.board(&target).unwrap().1.damage, 0);
    assert_eq!(g.stack.len(), 1);
    pass_top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.damage, 1);
    assert_eq!(g.board(&target).unwrap().1.wounds, 0);
    let mut g = game(0);
    let source = reveal(&mut g, 0, true);
    choose(&mut g, vec![source.clone()]);
    pass_top(&mut g);
    assert!(g.board(&source).is_none());
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JC029")
            .count(),
        1
    );
    assert_ne!(g.players[0].graveyard[0].id, source);
}

#[test]
fn jc029_response_guard_rechecks_hidden_moved_and_replaced_target_without_new_payment() {
    for variant in 0..3 {
        let mut g = game(0);
        let target = board(&mut g, "JC059", 1, 0);
        reveal(&mut g, 0, false);
        choose(&mut g, vec![target.clone()]);
        match variant {
            0 => g.board_mut(&target).unwrap().face_down = true,
            1 => {
                let c = remove_board(&mut g, &target);
                g.regions[1].cards.push(c);
            }
            _ => {
                remove_board(&mut g, &target);
                board(&mut g, "JC059", 1, 0);
            }
        }
        checkpoint(&g);
        pass_top(&mut g);
        assert!(g
            .regions
            .iter()
            .flat_map(|r| &r.cards)
            .all(|c| c.damage == 0));
        assert_eq!(g.resources(0), 0);
    }
}

#[test]
fn jc029_source_departure_does_not_cancel_independent_raid_and_prevention_is_reused() {
    let mut g = game(0);
    let target = board(&mut g, "JC059", 1, 0);
    let source = reveal(&mut g, 0, false);
    choose(&mut g, vec![target.clone()]);
    remove_board(&mut g, &source);
    pass_top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.damage, 1);
    let mut g = game(0);
    let target = board(&mut g, "JC059", 1, 0);
    reveal(&mut g, 0, false);
    choose(&mut g, vec![target.clone()]);
    g.turn_attribute_modifiers.push(TurnAttributeModifier {
        target_instance: target.clone(),
        defense_bonus: 0,
        kill_bonus: 0,
        grants_retreat: false,
        printed_defense_override: None,
        ordinary_icons: Icons::default(),
        grants_renown: false,
        prevents_damage: true,
        expires_turn: g.turn,
    });
    pass_top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.damage, 0);
    assert_eq!(g.board(&target).unwrap().1.wounds, 0);
}

#[test]
fn jc029_declared_frame_restores_all_four_views_and_same_next_command() {
    let mut g = game(0);
    let target = board(&mut g, "JC059", 1, 0);
    let source = reveal(&mut g, 0, false);
    let p = g.pending.clone().unwrap();
    let mut pending_restored = envelope(&g).game;
    let choice = Action {
        choice_id: Some(p.choice.id),
        selected: Some(vec![target.clone()]),
        ..Action::new("choose")
    };
    apply(&mut g, 0, choice.clone());
    apply(&mut pending_restored, 0, choice);
    assert_eq!(
        serde_json::to_value(&g).unwrap(),
        serde_json::to_value(&pending_restored).unwrap()
    );
    let mut restored = envelope(&g).game;
    let frame = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(frame.source.card.id, source);
    assert_eq!(frame.targets[0].id, target);
    assert!(frame.already_paid.is_empty());
    pass_top(&mut g);
    pass_top(&mut restored);
    assert_eq!(
        serde_json::to_value(g).unwrap(),
        serde_json::to_value(restored).unwrap()
    );
}
