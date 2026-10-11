//! Explicit offline layouts; real paid commands, strict restores and Room replay.
//! This is engine evidence, never a browser acceptance claim.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, deck, model::*, room::*, rules::*};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
static INVALID_SEQ: AtomicUsize = AtomicUsize::new(0);
static ROOM_SEQ: AtomicUsize = AtomicUsize::new(0);

fn held(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn deploy(g: &mut Game, actor: usize) -> String {
    let old = held(g, "XQ24", actor);
    fund(g, actor, "JC059", 4);
    apply(
        g,
        actor,
        Action {
            card_id: Some(old.clone()),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    assert_eq!(g.resources(actor), 0);
    assert!(g.board(&old).is_none());
    pass_top(g);
    g.regions[2]
        .cards
        .iter()
        .rev()
        .find(|c| c.definition == "XQ24" && c.controller == actor)
        .unwrap()
        .id
        .clone()
}
fn accept(g: &mut Game) {
    choose(g, vec!["accept".into()]);
}
fn hidden<'a>(g: &'a Game, def: &str, owner: usize, region: usize) -> &'a Card {
    g.regions[region]
        .cards
        .iter()
        .find(|c| c.definition == def && c.owner == owner && c.face_down)
        .unwrap()
}
fn invalid(g: &Game, broken: Game) {
    assert!(Game::from_persisted(&serde_json::to_string(&broken).unwrap()).is_err());
    let mut room = envelope(g);
    room.game = broken;
    let state = serde_json::to_string(&room).unwrap();
    assert!(RoomEnvelope::from_persisted(&state).is_err());
    if let Ok(dir) = std::env::var("XQ24_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let n = INVALID_SEQ.fetch_add(1, Ordering::Relaxed);
        std::fs::write(
            format!("{dir}/invalid-{n:04}.json"),
            serde_json::to_vec(&json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn xq24_complete_printed_fields_and_closed_program() {
    let c = catalog::card("XQ24");
    assert_eq!(
        (&*c.name, &*c.color, c.cost, c.defense),
        ("防暴警察", "灰", 4, Some(2))
    );
    assert_eq!(c.loyalty, ["灰色", "灰色"]);
    assert_eq!(c.subtypes, ["人类", "警察"]);
    assert_eq!(
        c.permanent_icons,
        Icons {
            investigation: 1,
            combat: 1,
            influence: 1
        }
    );
    assert_eq!(c.temporary_icons, Icons::default());
    assert!(c.magic.is_empty() && !c.unique);
    assert_eq!(deck::copy_limit(c), Some(3));
    let d = definition("XQ24");
    assert!(d.traits.public && !d.traits.city_play_only);
    assert_eq!(d.abilities.len(), 1);
    let a = &d.abilities[0];
    assert_eq!(a.event, Some(Event::Enter));
    assert_eq!(a.response_policy, ResponsePolicy::Respondable);
    assert!(a.targets.is_empty() && a.costs.is_empty() && a.modes.is_empty());
    assert!(matches!(
        a.ops.as_slice(),
        [Op::XQ24HideCheapCharactersInSourceRegion]
    ));
    crate::rules::validate_definitions(definitions()).unwrap();
    for variant in 0..5 {
        let mut registry = definitions().clone();
        let d = registry.get_mut("XQ24").unwrap();
        match variant {
            0 => d.traits.public = false,
            1 => d.abilities[0].response_policy = ResponsePolicy::Immediate,
            2 => d.abilities[0].ops.push(Op::Draw {
                player: PlayerRef::Actor,
                count: 1,
                end: DeckEnd::Top,
            }),
            3 => d.abilities[0].event = Some(Event::Reveal),
            _ => d.abilities[0].targets.push(target(
                Zone::Board,
                EntityKind::Character,
                Relation::Any,
                Range::Anywhere,
            )),
        }
        assert!(crate::rules::validate_definitions(&registry).is_err());
    }
    let mut registry = definitions().clone();
    registry.get_mut("JC056").unwrap().abilities[0].ops = a.ops.clone();
    assert!(crate::rules::validate_definitions(&registry).is_err());
}

#[test]
fn xq24_paid_entry_is_optional_without_targets_and_has_no_city_gate() {
    for actor in 0..4 {
        let mut g = game(actor);
        g.regions[2].card = g.make_card("DQJC107", 0); // Non-city is legal.
        let other = board(&mut g, "JC125", (actor + 1) % 4, 2);
        let source = deploy(&mut g, actor);
        let p = g.pending.as_ref().unwrap();
        assert_eq!(p.seat, actor);
        assert_eq!(p.choice.options.len(), 1);
        assert_eq!(p.choice.options[0].id, "accept");
        assert_eq!(
            (p.choice.min, p.choice.max, p.choice.allow_decline),
            (Some(0), Some(1), Some(true))
        );
        assert!(g.board(&other).is_some());
        choose(&mut g, vec![]);
        assert!(g.stack.is_empty() && g.pending.is_none());
        assert!(g.board(&source).is_some() && g.board(&other).is_some());
    }
}

#[test]
fn xq24_public_loyalty_cost_wrong_actor_and_choice_rejections_are_atomic() {
    for actor in 0..4 {
        for reason in 0..4 {
            let mut g = game(actor);
            let id = held(&mut g, "XQ24", actor);
            match reason {
                0 => fund(&mut g, actor, "JC125", 4),
                1 => fund(&mut g, actor, "JC059", 3),
                _ => fund(&mut g, actor, "JC059", 4),
            }
            reject(
                &mut g,
                if reason == 3 { (actor + 2) % 4 } else { actor },
                Action {
                    card_id: Some(id),
                    region: Some(2),
                    ..Action::new(if reason == 2 { "conceal" } else { "deploy" })
                },
            );
        }
    }
    let mut g = game(0);
    deploy(&mut g, 0);
    let p = g.pending.clone().unwrap();
    for (seat, selected) in [
        (2, vec!["accept".into()]),
        (0, vec!["bogus".into()]),
        (0, vec!["accept".into(), "accept".into()]),
    ] {
        reject(
            &mut g,
            seat,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(selected),
                ..Action::new("choose")
            },
        );
    }
}

#[test]
fn xq24_local_both_teams_zero_three_and_printed_cost_not_effective_cost() {
    for actor in 0..4 {
        for owner in 0..4 {
            let mut g = game(actor);
            let zero = board(&mut g, "JC125", owner, 2);
            let three = board(&mut g, "JZ44", owner, 2); // Public can be hidden by an effect.
            let four = board(&mut g, "XQ24", owner, 2);
            let foreign = board(&mut g, "JC125", owner, 1);
            let discounted_five = board(&mut g, "JC032", owner, 2);
            let already_hidden = board(&mut g, "JC056", owner, 2);
            g.board_mut(&already_hidden).unwrap().face_down = true;
            g.modifiers.push(CostModifier {
                actor: owner,
                filter: CardFilter::NamedCharacter("血族长老".into()),
                amount: 5,
                expires_turn: g.turn,
                uses: 1,
                paid_reveal: false,
            });
            let source = deploy(&mut g, actor);
            accept(&mut g);
            assert!(g
                .stack
                .last()
                .unwrap()
                .frame
                .as_ref()
                .unwrap()
                .targets
                .is_empty());
            pass_top(&mut g);
            assert!(g.board(&zero).is_none() && g.board(&three).is_none());
            assert!(
                g.board(&four).is_some()
                    && g.board(&foreign).is_some()
                    && g.board(&discounted_five).is_some()
            );
            assert!(g.board(&source).is_some() && !g.board(&source).unwrap().1.face_down);
            assert!(g.board(&already_hidden).unwrap().1.face_down);
            assert_ne!(hidden(&g, "JC125", owner, 2).id, zero);
            assert_ne!(hidden(&g, "JZ44", owner, 2).id, three);
            for viewer in 0..4 {
                let c = hidden(&g, "JZ44", owner, 2);
                let v = g.card_view(c, viewer, Some(2), None);
                if viewer != owner {
                    assert!(v.card_id.is_none() && v.text.is_none() && v.cost.is_none());
                }
            }
            checkpoint(&g);
        }
    }
}

#[test]
fn xq24_effect_hidden_public_card_can_paid_reveal_and_trigger_entry_again() {
    let mut g = game(0);
    let old = board(&mut g, "XQ24", 0, 2);
    g.board_mut(&old).unwrap().face_down = true;
    let target = board(&mut g, "JC125", 2, 2);
    fund(&mut g, 0, "JC059", 4);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(old.clone()),
            ..Action::new("reveal")
        },
    );
    assert_eq!(g.resources(0), 0);
    pass_top(&mut g);
    assert!(g.board(&old).is_none());
    accept(&mut g);
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    assert!(g.regions[2]
        .cards
        .iter()
        .any(|c| c.definition == "XQ24" && !c.face_down));
}

#[test]
fn xq24_empty_matching_set_is_legal_and_does_not_search_private_faces() {
    let mut g = game(0);
    let hidden_id = board(&mut g, "JC125", 2, 2);
    g.board_mut(&hidden_id).unwrap().face_down = true;
    deploy(&mut g, 0);
    accept(&mut g);
    pass_top(&mut g);
    assert!(g.board(&hidden_id).unwrap().1.face_down);
    assert!(g.pending.is_none() && g.stack.is_empty());
}

#[test]
fn xq24_hide_clears_markers_returns_seals_and_uses_existing_attachment_cleanup() {
    let mut g = game(0);
    let target = board(&mut g, "JC059", 2, 2);
    let c = g.board_mut(&target).unwrap();
    c.exhausted = true;
    c.damage = 0;
    c.wounds = 0;
    c.shield = 3;
    c.lock_markers = 2;
    c.time_markers = 2;
    let equipment = g.make_card("BQ022", 1);
    let curse = g.make_card("JC089", 3);
    let (eq_id, curse_id) = (equipment.id.clone(), curse.id.clone());
    g.attachments.push(Attachment {
        card: equipment,
        host_id: target.clone(),
    });
    g.attachments.push(Attachment {
        card: curse,
        host_id: target.clone(),
    });
    let seal = held(&mut g, "LC20", 1);
    g.seal_hand_card(1, &seal, &target).unwrap();
    deploy(&mut g, 0);
    accept(&mut g);
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    let c = hidden(&g, "JC059", 2, 2);
    assert!(c.exhausted);
    assert_eq!(
        (c.damage, c.wounds, c.shield, c.lock_markers, c.time_markers),
        (0, 0, 0, 0, 0)
    );
    assert!(g.attachments.is_empty() && g.sealed_cards.is_empty());
    assert!(g.players[1]
        .hand
        .iter()
        .any(|c| c.definition == "BQ022" && c.id != eq_id));
    assert!(g.players[1]
        .hand
        .iter()
        .any(|c| c.definition == "LC20" && c.id != seal));
    assert!(g.players[3]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC089" && c.id != curse_id));
    assert!(g
        .players
        .iter()
        .all(|p| !p.graveyard.iter().any(|c| c.definition == "JC059")));
    checkpoint(&g);
}

#[test]
fn xq24_pending_metadata_and_atomic_frame_tampering_rejected() {
    let mut g = game(0);
    deploy(&mut g, 0);
    for variant in 0..10 {
        let mut broken = g.clone();
        let p = broken.pending.as_mut().unwrap();
        match variant {
            0 => p.seat = 2,
            1 => p.choice.player_id = "p2".into(),
            2 => p.choice.title.push('!'),
            3 => p.choice.options[0].id = "forged".into(),
            4 => p.choice.min = Some(1),
            5 => p.choice.allow_decline = Some(false),
            6 => p.choice.amount = Some(1),
            7 => {
                if let ChoiceResolution::Declare { stage, .. } = &mut p.resolution {
                    *stage = DeclareChoice::Target;
                }
            }
            8 => {
                if let ChoiceResolution::Declare { declaration: d, .. } = &mut p.resolution {
                    d.actor = 2;
                }
            }
            _ => {
                if let ChoiceResolution::Declare { declaration: d, .. } = &mut p.resolution {
                    d.source.source_region_instance = None;
                }
            }
        }
        invalid(&g, broken);
    }
    accept(&mut g);
    for variant in 0..7 {
        let mut broken = g.clone();
        let f = broken.stack.last_mut().unwrap().frame.as_mut().unwrap();
        match variant {
            0 => f.cursor = 1,
            1 => f.guard = GuardState::Accepted,
            2 => f.actor = 2,
            3 => f.steps[0].context = 2,
            4 => f.steps[0].op = Op::Hide(EntityRef::Source),
            5 => f.source.source_region_instance = None,
            _ => f.source.card.definition = "JC056".into(),
        }
        invalid(&g, broken);
    }
    pass_top(&mut g);
}

#[test]
fn xq24_paid_stack_source_identity_payment_and_metadata_cannot_be_forged() {
    let mut g = game(0);
    let source = held(&mut g, "XQ24", 0);
    fund(&mut g, 0, "JC059", 4);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    for variant in 0..12 {
        let mut broken = g.clone();
        let s = broken.stack.last_mut().unwrap();
        match variant {
            0 => s.frame.as_mut().unwrap().source.card.definition = "JC056".into(),
            1 => s.card.as_mut().unwrap().definition = "JC056".into(),
            2 => s.frame.as_mut().unwrap().source.play_source = None,
            3 => s.reveal = true,
            4 => s.frame.as_mut().unwrap().chosen_region = Some(1),
            5 => s.frame.as_mut().unwrap().source.source_region_instance = None,
            6 => s.frame.as_mut().unwrap().cursor = 1,
            7 => {
                s.frame.as_mut().unwrap().already_paid =
                    vec![PaidCost::Assets(vec!["same".into(), "same".into()])]
            }
            8 => s.controller = 2,
            9 => s.frame = None,
            10 => s.frame.as_mut().unwrap().source.card.owner = 99,
            _ => s.frame.as_mut().unwrap().source.card.face_down = true,
        }
        invalid(&g, broken);
    }
    pass_top(&mut g);
    choose(&mut g, vec![]);
}

#[test]
fn xq24_response_source_moved_fixture_still_uses_the_original_real_region() {
    let mut g = game(0);
    let original = board(&mut g, "JC125", 2, 2);
    let elsewhere = board(&mut g, "JC125", 2, 1);
    let source = deploy(&mut g, 0);
    accept(&mut g);
    // Explicit response layout: movement has no dependency on its producer.
    let (_, c) = g.remove_board(&source).unwrap();
    g.regions[1].cards.push(c);
    checkpoint(&g);
    pass_top(&mut g);
    assert!(g.board(&original).is_none());
    assert!(g.board(&elsewhere).is_some());
    assert_eq!(g.board(&source).unwrap().0, 1);
}

#[test]
fn xq24_actual_paid_murder_response_does_not_cancel_the_declared_trigger() {
    let mut g = game(0);
    let target = board(&mut g, "JC125", 2, 2);
    let murder = held(&mut g, "JC091", 2);
    fund(&mut g, 2, "JC084", 3);
    let source = deploy(&mut g, 0);
    accept(&mut g);
    while g.priority_team != g.team(2) {
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        apply(&mut g, seat, Action::new("pass"));
    }
    apply(
        &mut g,
        2,
        Action {
            card_id: Some(murder),
            target_id: Some(source.clone()),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    assert!(g.board(&source).is_none());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ24"));
    assert_eq!(g.stack.len(), 1);
    checkpoint(&g);
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    assert!(hidden(&g, "JC125", 2, 2).face_down);
}

#[test]
fn xq24_response_source_hidden_fixture_keeps_the_declared_region() {
    let mut g = game(0);
    let target = board(&mut g, "JC125", 2, 2);
    let source = deploy(&mut g, 0);
    accept(&mut g);
    // Same existing Hide operation as a paid JC063 response, isolated here.
    g.hide_in_place(&source);
    checkpoint(&g);
    pass_top(&mut g);
    assert!(g.board(&source).is_none() && g.board(&target).is_none());
    assert!(hidden(&g, "XQ24", 0, 2).face_down);
}

#[test]
fn xq24_original_region_replaced_or_vacant_never_reaches_a_replacement() {
    for vacant in [false, true] {
        let mut g = game(0);
        let original = board(&mut g, "JC125", 2, 2);
        let source = deploy(&mut g, 0);
        accept(&mut g);
        let saved_region_id = g.regions[2].card.id.clone();
        // Explicit completed replacement/empty-slot layout, not a room mutation API.
        let moved = std::mem::take(&mut g.regions[2].cards);
        g.regions[3].cards.extend(moved);
        g.regions[2].card = g.make_card("DQJC107", 0);
        if vacant {
            g.world.clear();
            g.regions[2].vacant = true;
            g.regions[2].influence = [0, 0];
            g.regions[2].skip = false;
        }
        let replacement = (!vacant).then(|| board(&mut g, "JC125", 2, 2));
        assert_ne!(g.regions[2].card.id, saved_region_id);
        checkpoint(&g);
        pass_top(&mut g);
        assert!(g.board(&original).is_some() && g.board(&source).is_some());
        if let Some(id) = replacement {
            assert!(!g.board(&id).unwrap().1.face_down);
        }
        assert_eq!(
            g.regions.iter().filter(|r| r.vacant).count(),
            usize::from(vacant)
        );
    }
}

#[test]
fn xq24_response_character_movement_is_evaluated_at_resolution_without_targets() {
    let mut g = game(0);
    let leaving = board(&mut g, "JC125", 2, 2);
    let arriving = board(&mut g, "JC125", 2, 1);
    deploy(&mut g, 0);
    accept(&mut g);
    let (_, c) = g.remove_board(&leaving).unwrap();
    g.regions[1].cards.push(c);
    let (_, c) = g.remove_board(&arriving).unwrap();
    g.regions[2].cards.push(c);
    checkpoint(&g);
    pass_top(&mut g);
    assert!(g.board(&leaving).is_some() && g.board(&arriving).is_none());
}

fn room_command(
    room: &mut RoomEnvelope,
    seat: usize,
    action: SessionAction,
    steps: &mut Vec<serde_json::Value>,
) {
    let request = RoomCommand {
        command_id: format!("xq24-{}", ROOM_SEQ.fetch_add(1, Ordering::Relaxed)),
        expected_version: room.revision,
        action,
    };
    let before = serde_json::to_string(room).unwrap();
    let result = room.transition(seat, Some(request.clone()), 0).unwrap();
    assert!(result.error_code.is_none(), "{:?}", result.error_message);
    let replay = room.replay_events(&result.journal).unwrap();
    assert_eq!(serde_json::to_string(&replay).unwrap(), result.state);
    let restored = RoomEnvelope::from_persisted(&result.state).unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), result.state);
    let views = (0..room.game.players.len())
        .map(|s| restored.view(s, 0))
        .collect::<Vec<_>>();
    for (s, v) in views.iter().enumerate() {
        assert_eq!(
            serde_json::to_value(replay.view(s, 0)).unwrap(),
            serde_json::to_value(v).unwrap()
        );
    }
    steps.push(json!({"state":before,"seat":seat,"command":request,"expected":result,"views":views,"serverNowMs":"0"}));
    *room = restored;
}
fn room_action(
    room: &mut RoomEnvelope,
    seat: usize,
    a: Action,
    steps: &mut Vec<serde_json::Value>,
) {
    if a.kind == "pass" {
        let id = room.pacing.window.as_ref().unwrap().id.clone();
        room_command(
            room,
            seat,
            SessionAction::PassResponse { window_id: id },
            steps,
        );
    } else if let Some(window) = room.pacing.window.clone() {
        let intent = format!("xq24-intent-{}", ROOM_SEQ.load(Ordering::Relaxed));
        room_command(
            room,
            seat,
            SessionAction::BeginResponse {
                window_id: window.id.clone(),
                intent_id: intent.clone(),
            },
            steps,
        );
        room_command(
            room,
            seat,
            SessionAction::SubmitResponse {
                window_id: window.id,
                intent_id: intent,
                action: a,
            },
            steps,
        );
    } else {
        room_command(room, seat, SessionAction::Game { action: a }, steps);
    }
}
fn room_pass_top(room: &mut RoomEnvelope, steps: &mut Vec<serde_json::Value>) {
    let count = room.game.stack.len();
    assert!(count > 0);
    for _ in 0..32 {
        if room.game.stack.len() < count || room.game.pending.is_some() {
            return;
        }
        let s = (0..room.game.players.len())
            .find(|s| {
                room.game
                    .legal_actions(*s)
                    .iter()
                    .any(|a| a.action.kind == "pass")
            })
            .unwrap();
        room_action(room, s, Action::new("pass"), steps);
    }
    panic!("XQ24 room top did not resolve");
}

#[test]
fn xq24_entry_frame_container_restore_rejects_response_bypass() {
    let mut forged = Vec::new();
    for mode in ["duel", "teams"] {
        let mut g = Game::new(
            if mode == "duel" {
                "240000000000000000000001"
            } else {
                "240000000000000000000002"
            }
            .into(),
            "LOCAL".into(),
            mode.into(),
            "P0".into(),
            "watchers".into(),
            u64::MAX,
        )
        .unwrap();
        for s in 1..g.capacity() {
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
        g.first_team = 0;
        g.begin_window(Window::Action(0));
        let enemy = if mode == "duel" { 1 } else { 2 };
        board(&mut g, "JZ44", enemy, 2);
        let source = held(&mut g, "XQ24", 0);
        fund(&mut g, 0, "JC059", 4);
        let mut room = envelope(&g);
        let mut steps = Vec::new();
        room_action(
            &mut room,
            0,
            Action {
                card_id: Some(source),
                region: Some(2),
                ..Action::new("deploy")
            },
            &mut steps,
        );
        room_pass_top(&mut room, &mut steps);
        let choice = room.game.pending.clone().unwrap();
        room_action(
            &mut room,
            0,
            Action {
                choice_id: Some(choice.choice.id),
                selected: Some(vec!["accept".into()]),
                ..Action::new("choose")
            },
            &mut steps,
        );
        let item = room.game.stack.last().unwrap().clone();
        assert_eq!(
            item.frame.as_ref().unwrap().ability_key,
            "entry-hide-cheap-characters"
        );
        assert!(room.game.effects.is_empty());
        let original = serde_json::to_string(&room).unwrap();
        // Normal Game pass cannot bypass the still-open responsive frame.
        let pass = RoomCommand {
            command_id: "ordinary-pass".into(),
            expected_version: room.revision,
            action: SessionAction::Game {
                action: Action::new("pass"),
            },
        };
        let denied = room.transition(0, Some(pass), 0).unwrap();
        assert_eq!(denied.error_code.as_deref(), Some("invalid_action"));
        assert_eq!(denied.state, original);
        for keep_stack in [false, true] {
            let mut bad = room.clone();
            if !keep_stack {
                bad.game.stack.clear();
                bad.pacing.window = None;
            }
            bad.game.effects.push_back(Effect::Frame {
                frame: Box::new(item.frame.clone().unwrap()),
            });
            forged.push(json!({"mode":mode,"keepStack":keep_stack,
                "original":original,"state":serde_json::to_string(&bad).unwrap()}));
        }
        // The same real Room commands cross both teams and restore each legal
        // paid/declaration/response boundary, then restore the finished effect.
        room_pass_top(&mut room, &mut steps);
        assert!(room.game.stack.is_empty());
        assert!(hidden(&room.game, "JZ44", enemy, 2).face_down);
        for s in &steps {
            let state = s["expected"]["state"].as_str().unwrap();
            RoomEnvelope::from_persisted(state).unwrap();
        }
    }
    if let Ok(path) = std::env::var("XQ24_CONTAINER_EVIDENCE_FILE") {
        std::fs::write(
            path,
            serde_json::to_vec(&json!({
                "producer":"xq24_entry_frame_container_restore_rejects_response_bypass",
                "syntheticNativeInitialLayouts":true,"cases":forged
            }))
            .unwrap(),
        )
        .unwrap();
    }
    for record in forged {
        let state = record["state"].as_str().unwrap();
        let raw: RoomEnvelope = serde_json::from_str(state).unwrap();
        assert!(
            Game::from_persisted(&serde_json::to_string(&raw.game).unwrap()).is_err(),
            "XQ24 entry frame in effects must fail Game restore"
        );
        assert!(
            RoomEnvelope::from_persisted(state).is_err(),
            "XQ24 entry frame in effects must fail Room restore"
        );
    }
}

#[test]
fn xq24_continuous_duel_teams_room_pause_restore_journal_and_receipt_inputs() {
    for mode in ["duel", "teams"] {
        for name in [
            "accept",
            "decline",
            "murder-response",
            "source-moved-fixture",
            "region-replaced-fixture",
            "region-vacant-fixture",
        ] {
            let mut g = Game::new(
                format!("xq24-{mode}-{name}"),
                "LOCAL".into(),
                mode.into(),
                "P0".into(),
                "watchers".into(),
                9,
            )
            .unwrap();
            for s in 1..g.capacity() {
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
            g.first_team = 0;
            g.begin_window(Window::Action(0));
            let enemy = if mode == "duel" { 1 } else { 2 };
            let target = board(&mut g, "JZ44", enemy, 2);
            let elsewhere = board(&mut g, "JC125", enemy, 1);
            let source_hand = held(&mut g, "XQ24", 0);
            fund(&mut g, 0, "JC059", 4);
            let murder = (name == "murder-response").then(|| {
                fund(&mut g, enemy, "JC084", 3);
                held(&mut g, "JC091", enemy)
            });
            let prepared = name.ends_with("fixture");
            if prepared {
                g.apply(
                    0,
                    Action {
                        card_id: Some(source_hand.clone()),
                        region: Some(2),
                        ..Action::new("deploy")
                    },
                )
                .unwrap();
                // Real paid entry/acceptance followed by a named offline response layout.
                while g.pending.is_none() {
                    let s = (0..g.players.len())
                        .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
                        .unwrap();
                    g.apply(s, Action::new("pass")).unwrap();
                }
                let p = g.pending.clone().unwrap();
                g.apply(
                    0,
                    Action {
                        choice_id: Some(p.choice.id),
                        selected: Some(vec!["accept".into()]),
                        ..Action::new("choose")
                    },
                )
                .unwrap();
                let source = g.regions[2]
                    .cards
                    .iter()
                    .find(|c| c.definition == "XQ24")
                    .unwrap()
                    .id
                    .clone();
                if name == "source-moved-fixture" {
                    let (_, c) = g.remove_board(&source).unwrap();
                    g.regions[1].cards.push(c);
                } else {
                    let all = std::mem::take(&mut g.regions[2].cards);
                    g.regions[1].cards.extend(all);
                    g.regions[2].card = g.make_card("DQJC107", 0);
                    if name == "region-vacant-fixture" {
                        g.world.clear();
                        g.regions[2].vacant = true;
                        g.regions[2].skip = false;
                        g.regions[2].influence = [0, 0];
                    } else {
                        board(&mut g, "JC125", enemy, 2);
                    }
                }
            }
            let mut room = envelope(&g);
            let mut steps = vec![];
            if !prepared {
                room_action(
                    &mut room,
                    0,
                    Action {
                        card_id: Some(source_hand),
                        region: Some(2),
                        ..Action::new("deploy")
                    },
                    &mut steps,
                );
                room_pass_top(&mut room, &mut steps);
            }
            // Pause at a real declaration or response frame, restore and resume.
            room_command(&mut room, 0, SessionAction::PauseRoom, &mut steps);
            room_command(&mut room, 0, SessionAction::ResumeRoom, &mut steps);
            if !prepared {
                let p = room.game.pending.clone().unwrap();
                room_action(
                    &mut room,
                    0,
                    Action {
                        choice_id: Some(p.choice.id),
                        selected: Some(if name == "decline" {
                            vec![]
                        } else {
                            vec!["accept".into()]
                        }),
                        ..Action::new("choose")
                    },
                    &mut steps,
                );
            }
            if let Some(murder) = murder {
                while room.game.priority_team != room.game.team(enemy) {
                    let s = (0..room.game.players.len())
                        .find(|s| {
                            room.game
                                .legal_actions(*s)
                                .iter()
                                .any(|a| a.action.kind == "pass")
                        })
                        .unwrap();
                    room_action(&mut room, s, Action::new("pass"), &mut steps);
                }
                let source = room.game.regions[2]
                    .cards
                    .iter()
                    .find(|c| c.definition == "XQ24")
                    .unwrap()
                    .id
                    .clone();
                room_action(
                    &mut room,
                    enemy,
                    Action {
                        card_id: Some(murder),
                        target_id: Some(source),
                        ..Action::new("play")
                    },
                    &mut steps,
                );
                room_pass_top(&mut room, &mut steps);
                assert!(room.game.players[0]
                    .graveyard
                    .iter()
                    .any(|c| c.definition == "XQ24"));
            }
            if name != "decline" {
                room_command(&mut room, 0, SessionAction::PauseRoom, &mut steps);
                room_command(&mut room, 0, SessionAction::ResumeRoom, &mut steps);
                room_pass_top(&mut room, &mut steps);
            }
            assert!(room.game.board(&elsewhere).is_some());
            let replaced = name.starts_with("region-");
            assert_eq!(
                room.game.board(&target).is_some(),
                name == "decline" || replaced
            );
            if let Ok(dir) = std::env::var("XQ24_TRACE_DIR") {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(format!("{dir}/{mode}-{name}.json"), serde_json::to_vec(&json!({"name":format!("{mode}-{name}"),
                "scope":"explicit-offline-xq24-layout-with-continuous-native-room-commands", "catalog":catalog::catalog(), "steps":steps})).unwrap()).unwrap();
            }
        }
    }
}
