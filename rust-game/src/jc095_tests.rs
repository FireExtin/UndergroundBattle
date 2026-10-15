//! JC095 complete-source tests. Prepared layouts are explicit; exported traces
//! use continuous real Room commands, strict loads and actual journal replay.
use crate::{
    catalog,
    jc029_tests::{board, envelope, fund, game},
    model::*,
    room::*,
    rules::*,
};
use serde_json::{json, Value};
use std::path::Path;

const KEY: &str = "jc095-convert-enemy-original-region-influence";
fn held(g: &mut Game, definition: &str, actor: usize) -> String {
    let card = g.make_card(definition, actor);
    let id = card.id.clone();
    g.players[actor].hand.push(card);
    id
}
fn restore(g: &Game) {
    let raw = serde_json::to_string(g).unwrap();
    let restored = Game::from_persisted(&raw).unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), raw);
    let room = envelope(g);
    let state = serde_json::to_string(&room).unwrap();
    let restored_room = RoomEnvelope::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(&restored_room).unwrap(), state);
    for seat in 0..g.players.len() {
        assert_eq!(
            serde_json::to_value(room.view(seat, 0)).unwrap(),
            serde_json::to_value(restored_room.view(seat, 0)).unwrap()
        );
    }
}
fn choose(g: &mut Game, selected: Vec<String>) {
    let p = g.pending.clone().unwrap();
    g.apply(
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(selected),
            ..Action::new("choose")
        },
    )
    .unwrap();
}
fn top(g: &mut Game) {
    let count = g.stack.len();
    assert!(count > 0);
    for _ in 0..32 {
        if g.stack.len() < count || g.pending.is_some() {
            return;
        }
        let seat = (0..g.players.len())
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(seat, Action::new("pass")).unwrap();
    }
    panic!("stack did not resolve");
}
fn enter(g: &mut Game, actor: usize) -> String {
    fund(g, actor, "JC084", 7);
    let source = held(g, "JC095", actor);
    g.apply(
        actor,
        Action {
            card_id: Some(source),
            region: Some(2),
            ..Action::new("deploy")
        },
    )
    .unwrap();
    top(g);
    g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC095" && c.controller == actor)
        .unwrap()
        .id
        .clone()
}
fn invalid(g: &Game, bad: Game, name: &str) {
    assert!(
        Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),
        "Game accepted {name}"
    );
    let mut room = envelope(g);
    room.game = bad;
    let state = serde_json::to_string(&room).unwrap();
    assert!(
        RoomEnvelope::from_persisted(&state).is_err(),
        "Room accepted {name}"
    );
    if let Ok(dir) = std::env::var("JC095_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            Path::new(&dir).join(format!("{name}.json")),
            serde_json::to_vec(&json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn jc095_original_fields_complete_definition_and_nested_transplants_are_closed() {
    let c = catalog::card("JC095");
    assert_eq!(
        (&*c.name, c.subtitle.as_deref(), &*c.color),
        ("唐·格鲁尼", Some("波士顿之主"), "黑")
    );
    assert_eq!(c.cost, 7);
    assert_eq!(c.loyalty, ["黑色", "黑色", "黑色"]);
    assert_eq!(c.subtypes, ["人类", "罪犯"]);
    assert_eq!(c.magic_icon, MagicIcon::None);
    assert_eq!(c.defense, Some(2));
    assert!(c.unique && crate::jc052::printed_leader("JC095"));
    assert_eq!(c.keywords, ["领袖", "声望"]);
    assert_eq!(
        c.permanent_icons,
        Icons {
            investigation: 0,
            combat: 2,
            influence: 1
        }
    );
    assert_eq!(
        c.temporary_icons,
        Icons {
            investigation: 3,
            combat: 1,
            influence: 0
        }
    );
    let d = crate::jc095::definition();
    assert!(d.traits.renown && !d.traits.public && !d.traits.retreat);
    assert_eq!(d.abilities.len(), 1);
    let a = &d.abilities[0];
    assert_eq!(a.key, KEY);
    assert_eq!(a.event, Some(Event::Enter));
    assert_eq!(a.response_policy, ResponsePolicy::Respondable);
    assert!(a.targets.is_empty() && a.costs.is_empty() && a.modes.is_empty());
    assert!(matches!(
        a.ops.as_slice(),
        [Op::JC095ConvertEnemyInfluenceInOriginalRegion]
    ));
    for variant in 0..9 {
        let mut bad = d.clone();
        match variant {
            0 => bad.traits.renown = false,
            1 => bad.abilities[0].event = Some(Event::EnterRegion),
            2 => bad.abilities[0].event = Some(Event::Reveal),
            3 => bad.abilities[0].response_policy = ResponsePolicy::Immediate,
            4 => bad.abilities[0].ops.clear(),
            5 => bad.abilities[0].costs.push(Cost::Assets(1)),
            6 => bad.abilities[0]
                .ops
                .push(Op::JC095ConvertEnemyInfluenceInOriginalRegion),
            7 => bad.abilities[0].requires_ready_source = true,
            _ => bad.abilities[0].targets = definition("JC006").abilities[0].targets.clone(),
        }
        let mut registry = definitions().clone();
        registry.insert("JC095".into(), bad);
        assert!(
            validate_definitions(&registry).is_err(),
            "definition {variant}"
        );
    }
    for variant in 0..5 {
        let mut a = definition("JC006").abilities[0].clone();
        let op = Op::JC095ConvertEnemyInfluenceInOriginalRegion;
        match variant {
            0 => a.ops = vec![op],
            1 => a.ops = vec![Op::ForEachLivingPlayer(vec![op])],
            2 => a.ops = vec![Op::ForEachLivingPlayerFromActor(vec![op])],
            3 => {
                a.ops = vec![Op::IfTargetExhausted {
                    slot: 0,
                    exhausted: Box::new(op.clone()),
                    ready: Box::new(op),
                }]
            }
            _ => a.modes.push(Mode {
                key: "forged".into(),
                label: "forged".into(),
                targets: vec![],
                ops: vec![op],
            }),
        }
        assert!(
            validate_ability("JC006", &a).is_err(),
            "nested transplant {variant}"
        );
    }
}

#[test]
fn jc095_all_four_actors_optional_entry_zero_one_two_and_own_markers() {
    for actor in 0..4 {
        for amount in 0..=2 {
            for accept in [false, true] {
                let mut g = game(actor);
                let team = g.team(actor);
                g.regions[2].influence[1 - team] = amount;
                g.regions[1].influence[1 - team] = 2;
                let source = enter(&mut g, actor);
                assert_eq!(g.resources(actor), 0);
                let pending = g.pending.as_ref().unwrap();
                assert_eq!(pending.seat, actor);
                assert_eq!(
                    (
                        pending.choice.min,
                        pending.choice.max,
                        pending.choice.allow_decline
                    ),
                    (Some(0), Some(1), Some(true))
                );
                assert_eq!(g.regions[2].influence[1 - team], amount);
                choose(
                    &mut g,
                    if accept {
                        vec!["accept".into()]
                    } else {
                        vec![]
                    },
                );
                if accept {
                    top(&mut g);
                }
                assert_eq!(
                    g.regions[2].influence[team],
                    if accept { amount } else { 0 }
                );
                assert_eq!(
                    g.regions[2].influence[1 - team],
                    if accept { 0 } else { amount }
                );
                assert_eq!(g.regions[1].influence[1 - team], 2);
                assert!(g.board(&source).is_some());
                assert!(g.win_contexts.is_empty());
                restore(&g);
            }
        }
        let mut g = game(actor);
        let team = g.team(actor);
        g.regions[2].influence[team] = 2;
        enter(&mut g, actor);
        choose(&mut g, vec!["accept".into()]);
        top(&mut g);
        assert_eq!(g.regions[2].influence[team], 2);
        assert_eq!(g.regions[2].influence[1 - team], 0);
        restore(&g);
    }
}

#[test]
fn jc095_non_targets_ignore_barrier_shield_and_keep_current_permanent_and_temporary_icons() {
    let mut g = game(0);
    let enemy = board(&mut g, "JZ08", 2, 2);
    g.board_mut(&enemy).unwrap().shield = 3;
    g.regions[2].influence[1] = 2;
    let source = enter(&mut g, 0);
    choose(&mut g, vec!["accept".into()]);
    assert!(g
        .stack
        .last()
        .unwrap()
        .frame
        .as_ref()
        .unwrap()
        .targets
        .is_empty());
    top(&mut g);
    assert_eq!(g.regions[2].influence, [2, 0]);
    assert_eq!(g.board(&enemy).unwrap().1.shield, 3);
    assert!(g.has_barrier(g.board(&enemy).unwrap().1));
    assert_eq!(
        g.current_icons(g.board(&source).unwrap().1, 2),
        Icons {
            investigation: 3,
            combat: 3,
            influence: 1
        }
    );
    g.first_team = 1;
    assert_eq!(
        g.current_icons(g.board(&source).unwrap().1, 2),
        Icons {
            investigation: 0,
            combat: 2,
            influence: 1
        }
    );
    restore(&g);
}

#[test]
fn jc095_frozen_actor_region_and_original_instance_survive_explicit_source_changes() {
    for change in 0..5 {
        let mut g = game(0);
        g.regions[2].influence[1] = 2;
        g.regions[3].influence[1] = 1;
        let source = enter(&mut g, 0);
        choose(&mut g, vec!["accept".into()]);
        // These are disclosed rule-primitive probes during a real response
        // interval, separate from the continuous Room-command traces below.
        match change {
            0 => {
                let (_, c) = g.remove_board(&source).unwrap();
                g.regions[3].cards.push(c);
            }
            1 => {
                g.return_hand(&source);
            }
            2 => {
                assert!(g.hide_in_place(&source));
            }
            3 => g.add_control(
                &source,
                2,
                ControlLifetime::TurnEnd { turn: g.turn },
                SubtypeChange::None,
            ),
            _ => {
                g.return_hand(&source);
                board(&mut g, "JC095", 2, 3);
            }
        }
        restore(&g);
        top(&mut g);
        assert_eq!(g.regions[2].influence, [2, 0]);
        assert_eq!(g.regions[3].influence, [0, 1]);
        restore(&g);
    }
}

#[test]
fn jc095_original_region_incarnation_guards_paid_entry_and_conversion() {
    for already_entered in [false, true] {
        let mut g = game(0);
        g.regions[2].influence[1] = 2;
        if already_entered {
            enter(&mut g, 0);
            choose(&mut g, vec!["accept".into()]);
        } else {
            fund(&mut g, 0, "JC084", 7);
            let source = held(&mut g, "JC095", 0);
            g.apply(
                0,
                Action {
                    card_id: Some(source),
                    region: Some(2),
                    ..Action::new("deploy")
                },
            )
            .unwrap();
        }
        // Explicit replacement-boundary probe, not a natural win fixture.
        g.regions[2].card = g.make_card("DQJC107", 0);
        restore(&g);
        top(&mut g);
        assert_eq!(g.regions[2].influence, [0, 2]);
        assert!(g.pending.is_none());
        if !already_entered {
            assert!(!g.regions[2].cards.iter().any(|c| c.definition == "JC095"));
            assert!(g.players[0]
                .graveyard
                .iter()
                .any(|c| c.definition == "JC095"));
        }
        restore(&g);
    }
}

#[test]
fn jc095_payment_loyalty_and_extra_payload_fail_atomically() {
    for variant in 0..9 {
        let mut g = game(0);
        fund(
            &mut g,
            0,
            if variant == 1 { "JC125" } else { "JC084" },
            if variant == 0 { 6 } else { 7 },
        );
        let source = held(&mut g, "JC095", 0);
        let mut action = Action {
            card_id: Some(source),
            region: Some(2),
            ..Action::new("deploy")
        };
        match variant {
            0 | 1 => {}
            2 => action.target_id = Some("region:2".into()),
            3 => action.ability_id = Some(KEY.into()),
            4 => action.option = Some("accept".into()),
            5 => action.choice_id = Some("forged".into()),
            6 => action.selected = Some(vec![]),
            7 => action.top = Some(vec![]),
            _ => action.cost_selected = Some(vec![]),
        }
        let before = serde_json::to_string(&g).unwrap();
        assert!(g.apply(0, action).is_err(), "payment/payload {variant}");
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        restore(&g);
    }
}

#[test]
fn jc095_pending_metadata_and_frozen_source_identity_are_strict() {
    let mut g = game(0);
    let unrelated = board(&mut g, "JC125", 0, 1);
    enter(&mut g, 0);
    for variant in 0..17 {
        let mut bad = g.clone();
        let p = bad.pending.as_mut().unwrap();
        match variant {
            0 => p.choice.min = Some(1),
            1 => p.choice.max = Some(2),
            2 => p.choice.allow_decline = Some(false),
            3 => p.choice.title.clear(),
            4 => p.choice.description.clear(),
            5 => p.choice.options[0].label.clear(),
            6 => p.choice.options.clear(),
            7 => p.choice.player_id = "p1".into(),
            8 => p.seat = 1,
            9 => p.choice.amount = Some(2),
            10 => p.choice.id.clear(),
            _ => {
                let ChoiceResolution::Declare { declaration, stage } = &mut p.resolution else {
                    panic!("optional entry");
                };
                match variant {
                    11 => *stage = DeclareChoice::Target,
                    12 => declaration.source.card.id = unrelated.clone(),
                    13 => declaration.source.card.owner = 1,
                    14 => declaration.source.source_region_instance = None,
                    15 => {
                        declaration.ability.ops =
                            vec![Op::ForEachLivingPlayer(declaration.ability.ops.clone())]
                    }
                    _ => declaration.ability.response_policy = ResponsePolicy::Immediate,
                }
            }
        }
        invalid(&g, bad, &format!("entry-metadata-{variant}"));
    }
}

#[test]
fn jc095_paid_trigger_reward_and_nested_frames_cannot_be_forged_or_replayed() {
    let mut g = game(0);
    fund(&mut g, 0, "JC084", 7);
    let source = held(&mut g, "JC095", 0);
    g.apply(
        0,
        Action {
            card_id: Some(source),
            region: Some(2),
            ..Action::new("deploy")
        },
    )
    .unwrap();
    for variant in 0..9 {
        let mut bad = g.clone();
        let f = bad.stack[0].frame.as_mut().unwrap();
        match variant {
            0 => f.source.source_region_instance = None,
            1 => f.already_paid.clear(),
            2 => f.source.play_source = None,
            3 => f.chosen_region = Some(3),
            4 => f.cursor = 1,
            5 => f.guard = GuardState::Accepted,
            6 => f.steps.push(Step {
                context: 0,
                op: Op::JC095ConvertEnemyInfluenceInOriginalRegion,
            }),
            7 => f.source.card.definition = "JC125".into(),
            _ => f.already_paid = vec![PaidCost::Assets(vec!["same".into(), "same".into()])],
        }
        invalid(&g, bad, &format!("deploy-frame-{variant}"));
    }
    top(&mut g);
    choose(&mut g, vec!["accept".into()]);
    for variant in 0..16 {
        let mut bad = g.clone();
        let f = bad.stack[0].frame.as_mut().unwrap();
        match variant {
            0 => f.source.source_region_instance = None,
            1 => f.actor = 2,
            2 => f.steps.clear(),
            3 => f.steps.push(f.steps[0].clone()),
            4 => f.guard = GuardState::Accepted,
            5 => f.cursor = 1,
            6 => f.source.card.face_down = true,
            7 => f.steps[0].context = 1,
            8 => f.chosen_region = Some(3),
            9 => f.source.card.definition = "JC125".into(),
            10 => {
                f.steps[0].op =
                    Op::ForEachLivingPlayer(vec![Op::JC095ConvertEnemyInfluenceInOriginalRegion])
            }
            11 => {
                f.steps[0].op = Op::ForEachLivingPlayerFromActor(vec![
                    Op::JC095ConvertEnemyInfluenceInOriginalRegion,
                ])
            }
            12 => f.already_paid = vec![PaidCost::Assets(vec![])],
            13 => {
                f.ability_key = "renown".into();
                f.steps[0].op = Op::PlaceInfluence {
                    region_instance: f.source.source_region_instance.clone().unwrap(),
                    amount: 2,
                };
            }
            14 => {
                f.steps[0].op = Op::IfTargetExhausted {
                    slot: 0,
                    exhausted: Box::new(Op::JC095ConvertEnemyInfluenceInOriginalRegion),
                    ready: Box::new(Op::JC095ConvertEnemyInfluenceInOriginalRegion),
                }
            }
            _ => {
                f.ability_key = "jc089-combat-glory".into();
                f.steps[0].op = Op::PlaceInfluence {
                    region_instance: "different-region".into(),
                    amount: 1,
                };
            }
        }
        invalid(&g, bad, &format!("trigger-frame-{variant}"));
    }
    let mut bad = g.clone();
    let frame = bad.stack.pop().unwrap().frame.unwrap();
    bad.effects.push_back(Effect::Frame {
        frame: Box::new(frame),
    });
    invalid(&g, bad, "direct-or-replayed-conversion");
    let mut bad = g.clone();
    bad.stack[0].controller = 2;
    invalid(&g, bad, "trigger-stack-controller");
    let mut bad = g.clone();
    bad.stack[0].target = Some("region:2".into());
    invalid(&g, bad, "trigger-stack-target");
}

#[test]
fn jc095_existing_granted_combat_glory_remains_source_bound() {
    let mut g = game(0);
    let source = board(&mut g, "JC095", 0, 2);
    let curse = g.make_card("JC089", 2);
    g.attachments.push(Attachment {
        card: curse,
        host_id: source,
    });
    let declaration = g.jc089_combat_glory(0, 2).unwrap();
    g.effects.push_back(Effect::Declare { declaration });
    g.drive().unwrap();
    restore(&g);
    choose(&mut g, vec!["accept".into()]);
    restore(&g);
    top(&mut g);
    assert_eq!(g.regions[2].influence, [1, 0]);
    restore(&g);
}

#[test]
fn jc095_assets_hidden_and_region_movement_do_not_retrigger_entry_or_apply_printed_traits() {
    let mut g = game(0);
    fund(&mut g, 0, "JC095", 1);
    let hidden = board(&mut g, "JC095", 0, 1);
    g.board_mut(&hidden).unwrap().face_down = true;
    assert!(!g.has_renown(g.board(&hidden).unwrap().1));
    assert!(g.leader_options(0).is_empty());
    let source = board(&mut g, "JC095", 0, 2);
    g.regions[3].influence[1] = 2;
    let (_, card) = g.remove_board(&source).unwrap();
    g.regions[3].cards.push(card);
    let snapshot = g.source_snapshot(g.board(&source).unwrap().1, Some(3));
    g.emit_event(0, snapshot, Event::EnterRegion);
    g.drive().unwrap();
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.regions[3].influence, [0, 2]);
    assert_eq!(g.leader_options(0).len(), 1);
    restore(&g);
}

/// One explicit offline layout followed by continuous actual Native commands.
/// Every checkpoint reloads both strict loaders and replays the real journal.
struct Trace {
    name: &'static str,
    room: RoomEnvelope,
    initial: Value,
    steps: Vec<Value>,
}
impl Trace {
    fn new(name: &'static str, mut g: Game) -> Self {
        g.room_id = "950000000000000000000001".into();
        let room = envelope(&g);
        let initial = json!({"state":serde_json::to_string(&room).unwrap(),"views":(0..room.game.players.len()).map(|s|room.view(s,0)).collect::<Vec<_>>()});
        Self {
            name,
            room,
            initial,
            steps: vec![],
        }
    }
    fn command(&mut self, seat: usize, action: SessionAction, error: bool) {
        let state = serde_json::to_string(&self.room).unwrap();
        let command = RoomCommand {
            command_id: format!("jc095-{}-{}", self.name, self.steps.len()),
            expected_version: self.room.revision,
            action,
        };
        let expected = self
            .room
            .transition(seat, Some(command.clone()), 0)
            .unwrap();
        assert_eq!(
            expected.error_code.is_some(),
            error,
            "{:?}",
            expected.error_message
        );
        if !error {
            assert_eq!(
                serde_json::to_string(&self.room.replay_events(&expected.journal).unwrap())
                    .unwrap(),
                expected.state
            );
        }
        if error {
            assert_eq!(expected.state, state);
        }
        self.room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        assert_eq!(serde_json::to_string(&self.room).unwrap(), expected.state);
        let game = serde_json::to_string(&self.room.game).unwrap();
        assert_eq!(
            serde_json::to_string(&Game::from_persisted(&game).unwrap()).unwrap(),
            game
        );
        self.steps.push(json!({"state":state,"seat":seat,"command":command,"serverNowMs":"0","expected":expected,"views":(0..self.room.game.players.len()).map(|s|self.room.view(s,0)).collect::<Vec<_>>()}));
    }
    fn action(&mut self, seat: usize, action: Action) {
        if let Some(w) = self.room.pacing.window.clone() {
            let intent_id = format!("jc095-intent-{}", self.steps.len());
            self.command(
                seat,
                SessionAction::BeginResponse {
                    window_id: w.id.clone(),
                    intent_id: intent_id.clone(),
                },
                false,
            );
            self.command(
                seat,
                SessionAction::SubmitResponse {
                    window_id: w.id,
                    intent_id,
                    action,
                },
                false,
            );
        } else {
            self.command(seat, SessionAction::Game { action }, false);
        }
    }
    fn finish(&mut self) {
        for _ in 0..64 {
            if self.room.game.pending.is_some() {
                return;
            }
            if self.room.game.stack.is_empty() {
                return;
            }
            let w = self
                .room
                .pacing
                .window
                .clone()
                .expect("responsive paid frame");
            let seat = *w
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            self.command(seat, SessionAction::PassResponse { window_id: w.id }, false);
        }
        panic!("finite stack did not finish");
    }
    fn legal(
        &self,
        actor: usize,
        kind: &str,
        source: &str,
        target: Option<&str>,
        region: Option<usize>,
    ) -> Action {
        self.room
            .game
            .legal_actions(actor)
            .into_iter()
            .find(|a| {
                a.action.kind == kind
                    && a.action.card_id.as_deref() == Some(source)
                    && target.is_none_or(|t| a.action.target_id.as_deref() == Some(t))
                    && region.is_none_or(|r| a.action.region == Some(r))
            })
            .unwrap_or_else(|| panic!("{} missing {kind} for {source}", self.name))
            .action
    }
    fn export(&self) {
        if let Ok(dir) = std::env::var("JC095_NATIVE_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            let bytes=serde_json::to_vec(&json!({"name":self.name,"scope":"Explicit prepared Native layout; continuous real Room commands, strict restore and journal replay; not browser play or migration","catalog":catalog::catalog(),"initial":self.initial,"steps":self.steps})).unwrap();
            let path = Path::new(&dir).join(format!("{}.json", self.name));
            use std::io::Write;
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .unwrap()
                .write_all(&bytes)
                .unwrap();
        }
    }
}

impl Trace {
    fn response_priority(&mut self, actor: usize) {
        for _ in 0..8 {
            let window = self
                .room
                .pacing
                .window
                .clone()
                .expect("live response window");
            if window.holder_team == self.room.game.team(actor) {
                return;
            }
            let seat = *window
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            self.command(
                seat,
                SessionAction::PassResponse {
                    window_id: window.id,
                },
                false,
            );
        }
        panic!("response priority did not reach actor");
    }
}

impl Trace {
    fn choose(&mut self, selected: Vec<String>) {
        let p = self.room.game.pending.clone().expect("pending choice");
        self.action(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(selected),
                ..Action::new("choose")
            },
        );
    }
    fn source(&self, actor: usize) -> String {
        self.room
            .game
            .regions
            .iter()
            .flat_map(|r| &r.cards)
            .find(|c| c.definition == "JC095" && !c.face_down && c.controller == actor)
            .unwrap()
            .id
            .clone()
    }
    fn advance(&mut self) {
        if let Some(p) = self.room.game.pending.clone() {
            self.choose(
                p.choice
                    .options
                    .iter()
                    .take(p.choice.min.unwrap_or(0))
                    .map(|o| o.id.clone())
                    .collect(),
            );
        } else if let Some(window) = self.room.pacing.window.clone() {
            let seat = *window
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            self.command(
                seat,
                SessionAction::PassResponse {
                    window_id: window.id,
                },
                false,
            );
        } else {
            let seat = (0..self.room.game.players.len())
                .find(|s| {
                    self.room
                        .game
                        .legal_actions(*s)
                        .iter()
                        .any(|a| a.action.kind == "pass")
                })
                .unwrap();
            self.action(seat, Action::new("pass"));
        }
    }
    fn until(&mut self, predicate: impl Fn(&Game) -> bool) {
        for _ in 0..192 {
            if predicate(&self.room.game) {
                return;
            }
            self.advance();
        }
        panic!("{} did not reach bounded state", self.name);
    }
}

#[test]
fn jc095_room_paid_deploy_accept_and_decline_are_optional_untargeted_and_public() {
    for accept in [true, false] {
        let mut g = game(0);
        g.regions[2].influence[1] = 2;
        fund(&mut g, 0, "JC084", 7);
        let source = held(&mut g, "JC095", 0);
        let mut t = Trace::new(
            if accept {
                "paid-deploy-accept"
            } else {
                "paid-deploy-decline"
            },
            g,
        );
        t.action(0, t.legal(0, "deploy", &source, None, Some(2)));
        t.finish();
        let source = t.source(0);
        assert_eq!(t.room.game.regions[2].influence, [0, 2]);
        assert_eq!(t.room.game.resources(0), 0);
        let p = t.room.game.pending.clone().unwrap();
        assert_eq!(p.seat, 0);
        assert_eq!(p.choice.options.len(), 1);
        assert_eq!(p.choice.options[0].id, "accept");
        assert!(
            matches!(&p.resolution, ChoiceResolution::Declare { declaration, stage: DeclareChoice::Accept } if declaration.ability.key == KEY && declaration.ability.targets.is_empty())
        );
        for viewer in 0..4 {
            let view = t.room.view(viewer, 0);
            let c = view.regions[2]
                .characters
                .iter()
                .find(|c| c.instance_id == source)
                .unwrap();
            assert_eq!(c.card_id.as_deref(), Some("JC095"));
            assert_eq!(c.current_renown, Some(true));
            assert_eq!(view.pending_choice.is_some(), viewer == 0);
        }
        t.command(
            1,
            SessionAction::Game {
                action: Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec!["accept".into()]),
                    ..Action::new("choose")
                },
            },
            true,
        );
        t.choose(if accept {
            vec!["accept".into()]
        } else {
            vec![]
        });
        t.finish();
        assert_eq!(
            t.room.game.regions[2].influence,
            if accept { [2, 0] } else { [0, 2] }
        );
        t.export();
    }
}

#[test]
fn jc095_room_secret_deployment_and_paid_reveal_only_offer_one_entry_trigger() {
    let mut g = game(0);
    g.regions[2].influence[1] = 2;
    fund(&mut g, 0, "JC084", 8);
    let source = held(&mut g, "JC095", 0);
    let mut t = Trace::new("secret-reveal", g);
    t.action(0, t.legal(0, "conceal", &source, None, Some(2)));
    assert!(t.room.game.pending.is_none() && t.room.game.stack.is_empty());
    assert_eq!(t.room.game.regions[2].influence, [0, 2]);
    let hidden = t.room.game.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC095")
        .unwrap()
        .id
        .clone();
    assert!(t.room.game.leader_options(0).is_empty());
    for seat in 1..4 {
        let view = t.room.view(seat, 0);
        let card = view.regions[2]
            .characters
            .iter()
            .find(|c| c.instance_id == hidden)
            .unwrap();
        assert!(card.card_id.is_none() && card.current_renown.is_none());
    }
    t.action(0, t.legal(0, "reveal", &hidden, None, None));
    t.finish();
    assert_ne!(t.source(0), hidden);
    t.choose(vec!["accept".into()]);
    t.finish();
    assert_eq!(t.room.game.resources(0), 0);
    assert_eq!(t.room.game.regions[2].influence, [2, 0]);
    assert!(t.room.game.pending.is_none() && t.room.game.stack.is_empty());
    t.export();
}

#[test]
fn jc095_room_response_reads_current_enemy_count_including_zero_after_cancellation() {
    for increase in [true, false] {
        let mut g = game(0);
        g.regions[2].influence[1] = 1;
        fund(&mut g, 0, "JC084", 7);
        let responder = if increase { 2 } else { 0 };
        fund(&mut g, responder, "JC029", 2);
        let hidden = board(&mut g, "JZ22", responder, 2);
        g.board_mut(&hidden).unwrap().face_down = true;
        let source = held(&mut g, "JC095", 0);
        let mut t = Trace::new(
            if increase {
                "response-increase"
            } else {
                "response-zero"
            },
            g,
        );
        t.action(0, t.legal(0, "deploy", &source, None, Some(2)));
        t.finish();
        t.choose(vec!["accept".into()]);
        t.response_priority(responder);
        t.action(responder, t.legal(responder, "reveal", &hidden, None, None));
        t.finish();
        assert!(
            matches!(&t.room.game.pending.as_ref().unwrap().resolution, ChoiceResolution::Declare { declaration, .. } if declaration.source.card.definition == "JZ22")
        );
        t.choose(vec!["accept".into()]);
        t.finish();
        assert_eq!(
            t.room.game.regions[2].influence,
            if increase { [2, 0] } else { [0, 0] }
        );
        assert!(t.room.game.win_contexts.is_empty());
        t.export();
    }
}

#[test]
fn jc095_room_source_return_response_does_not_cancel_original_conversion() {
    let mut g = game(0);
    g.regions[2].influence[1] = 2;
    fund(&mut g, 0, "JC084", 9);
    let rescuer = board(&mut g, "JC075", 0, 0);
    let source = held(&mut g, "JC095", 0);
    let mut t = Trace::new("source-return", g);
    t.action(0, t.legal(0, "deploy", &source, None, Some(2)));
    t.finish();
    let source = t.source(0);
    t.choose(vec!["accept".into()]);
    t.response_priority(0);
    t.action(0, t.legal(0, "activate", &rescuer, Some(&source), None));
    t.finish();
    assert!(t.room.game.board(&source).is_none());
    assert!(t.room.game.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "JC095"));
    assert_eq!(t.room.game.regions[2].influence, [2, 0]);
    t.export();
}

#[test]
fn jc095_room_source_hide_response_preserves_declared_actor_and_conversion() {
    let mut g = game(0);
    g.regions[2].influence[1] = 2;
    fund(&mut g, 0, "JC084", 7);
    fund(&mut g, 0, "JC061", 2);
    let chase = held(&mut g, "JC063", 0);
    let source = held(&mut g, "JC095", 0);
    let mut t = Trace::new("source-hide", g);
    t.action(0, t.legal(0, "deploy", &source, None, Some(2)));
    t.finish();
    let source = t.source(0);
    t.choose(vec!["accept".into()]);
    t.response_priority(0);
    t.action(0, t.legal(0, "play", &chase, Some(&source), None));
    t.finish();
    assert!(t.room.game.board(&source).is_none());
    let hidden = t.room.game.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC095")
        .unwrap();
    assert!(hidden.face_down);
    assert!(!t.room.game.has_renown(hidden));
    assert!(t.room.game.leader_options(0).is_empty());
    assert_eq!(t.room.game.regions[2].influence, [2, 0]);
    t.export();
}

#[test]
fn jc095_room_new_leader_sacrifice_keeps_the_already_observed_entry_trigger() {
    let mut g = game(0);
    let prior = board(&mut g, "JC037", 0, 0);
    g.regions[2].influence[1] = 2;
    fund(&mut g, 0, "JC084", 7);
    let source = held(&mut g, "JC095", 0);
    let mut t = Trace::new("leader-sacrifice-new", g);
    t.action(0, t.legal(0, "deploy", &source, None, Some(2)));
    t.finish();
    let source = t.source(0);
    let p = t.room.game.pending.clone().unwrap();
    assert!(matches!(p.resolution, ChoiceResolution::Unique));
    assert_eq!(
        (p.choice.min, p.choice.max, p.choice.allow_decline),
        (Some(1), Some(1), Some(false))
    );
    t.command(
        0,
        SessionAction::Game {
            action: Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        },
        true,
    );
    t.choose(vec![source.clone()]);
    t.finish();
    assert!(t.room.game.board(&source).is_none());
    assert!(t.room.game.board(&prior).is_some());
    assert!(
        matches!(&t.room.game.pending.as_ref().unwrap().resolution, ChoiceResolution::Declare { declaration, .. } if declaration.ability.key == KEY && declaration.source.card.id == source)
    );
    t.choose(vec!["accept".into()]);
    t.finish();
    assert_eq!(t.room.game.regions[2].influence, [2, 0]);
    assert_eq!(t.room.game.leader_options(0).len(), 1);
    t.export();
}

#[test]
fn jc095_room_two_teammate_leaders_produce_one_current_renown_reward() {
    let mut g = game(0);
    g.regions[2].card = g.make_card("DQJC107", 0);
    board(&mut g, "JC095", 0, 2);
    board(&mut g, "JC095", 1, 2);
    g.begin_window(Window::Before(2, 2));
    let mut t = Trace::new("renown-team", g);
    t.until(|g| g.pending.as_ref().is_some_and(|p| matches!(&p.resolution, ChoiceResolution::Declare { declaration, .. } if declaration.ability.key == "renown")));
    assert_eq!(t.room.game.regions[2].influence, [2, 0]);
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 0);
    t.choose(vec!["accept".into()]);
    t.finish();
    assert_eq!(t.room.game.regions[2].influence, [3, 0]);
    assert!(t.room.game.pending.is_none() && t.room.game.stack.is_empty());
    assert_eq!(t.room.game.leader_options(0).len(), 1);
    assert_eq!(t.room.game.leader_options(1).len(), 1);
    t.export();
}

#[test]
fn jc095_room_reveal_in_already_won_window_cannot_change_winner_or_award_twice() {
    let mut g = game(0);
    let region_id = g.regions[2].card.id.clone();
    let region_definition = g.regions[2].card.definition.clone();
    let points = catalog::card(&g.regions[2].card.definition).points.unwrap();
    let threshold = catalog::card(&g.regions[2].card.definition)
        .threshold
        .unwrap();
    g.regions[2].influence[1] = threshold;
    let source = board(&mut g, "JC095", 0, 2);
    g.board_mut(&source).unwrap().face_down = true;
    fund(&mut g, 0, "JC084", 7);
    // A disclosed valid already-won fixture: its winner is frozen before any
    // actual reveal command. This is not an unattached over-threshold layout.
    g.begin_window(Window::Win(2, 2));
    let mut t = Trace::new("win-window-fixed-winner", g);
    t.action(0, t.legal(0, "reveal", &source, None, None));
    t.finish();
    t.choose(vec!["accept".into()]);
    t.finish();
    assert_eq!(t.room.game.regions[2].influence, [threshold, 0]);
    assert_eq!(t.room.game.window, Some(Window::Win(2, 2)));
    assert_eq!(t.room.game.win_contexts.len(), 1);
    assert_eq!(t.room.game.win_contexts[0].seat, 2);
    assert!(!t.room.game.effects.iter().any(
        |e| matches!(e, Effect::Award { region_instance, .. } if *region_instance == region_id)
    ));
    // Scoring moves the region to another zone and gives it a fresh instance
    // identity. Track its printed definition while retaining the frozen winner.
    t.until(|g| {
        g.players[2]
            .score_cards
            .iter()
            .any(|c| c.definition == region_definition)
    });
    assert!(t.room.game.win_contexts.is_empty());
    assert_eq!(t.room.game.view(0).players[2].score, points);
    assert_eq!(t.room.game.view(0).players[0].score, 0);
    let scored = &t.room.game.players[2].score_cards[0];
    assert_eq!(scored.definition, region_definition);
    assert_ne!(scored.id, region_id);
    assert_eq!((scored.owner, scored.controller), (2, 2));
    assert_eq!(
        t.room
            .game
            .players
            .iter()
            .flat_map(|p| &p.score_cards)
            .count(),
        1
    );
    assert!(t.room.game.players[0]
        .deck
        .iter()
        .any(|c| c.definition == "JC095"));
    t.export();
}

#[test]
fn jc095_room_duel_paid_entry_converts_only_the_other_player_pool() {
    let mut g = Game::new(
        "jc095-duel".into(),
        "LOCAL".into(),
        "duel".into(),
        "P0".into(),
        "watchers".into(),
        95,
    )
    .unwrap();
    g.join("P1".into(), "watchers".into()).unwrap();
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
    for region in &mut g.regions {
        region.cards.clear();
        region.influence = [0, 0];
    }
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    g.regions[1].influence[1] = 2;
    fund(&mut g, 0, "JC084", 7);
    let source = held(&mut g, "JC095", 0);
    let mut t = Trace::new("duel-paid-deploy", g);
    t.action(0, t.legal(0, "deploy", &source, None, Some(1)));
    t.finish();
    t.choose(vec!["accept".into()]);
    t.finish();
    assert_eq!(t.room.game.regions[1].influence, [2, 0]);
    assert_eq!(t.room.game.resources(0), 0);
    for viewer in 0..2 {
        let view = t.room.view(viewer, 0);
        assert_eq!((view.players.len(), view.regions.len()), (2, 3));
        assert_eq!(view.regions[1].influence, [2, 0]);
    }
    t.export();
}
