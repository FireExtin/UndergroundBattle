//! JC046 isolated candidate tests. Initial layouts are explicitly prepared
//! offline; exported traces then use continuous real Room commands, loaders,
//! journal replay and all-seat views. No browser or migration claim is made.
use crate::{
    catalog,
    jc029_tests::{board, envelope, fund, game},
    model::*,
    room::*,
    rules::*,
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::Path};
const KEY: &str = crate::jc046::KEY;
fn held(g: &mut Game, definition: &str, actor: usize) -> String {
    let c = g.make_card(definition, actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn hidden(g: &mut Game, actor: usize, region: usize) -> String {
    let id = board(g, "JC046", actor, region);
    g.board_mut(&id).unwrap().face_down = true;
    id
}
fn choose_action(g: &Game, selected: Vec<String>) -> Action {
    Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(selected),
        ..Action::new("choose")
    }
}
fn invalid(valid: &Game, bad: Game, name: &str) {
    assert!(
        Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),
        "Game accepted {name}"
    );
    let mut room = envelope(valid);
    room.game = bad;
    let state = serde_json::to_string(&room).unwrap();
    assert!(
        RoomEnvelope::from_persisted(&state).is_err(),
        "Room accepted {name}"
    );
    if let Ok(dir) = std::env::var("JC046_INVALID_DIR") {
        let safe_name = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();
        assert!(!safe_name.is_empty(), "empty JC046 invalid export name");
        std::fs::create_dir_all(&dir).unwrap();
        let path = Path::new(&dir).join(format!("{safe_name}.json"));
        assert!(
            !path.exists(),
            "duplicate JC046 invalid export name: {name}"
        );
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap()
            .write_all(&serde_json::to_vec(&json!({"state":state})).unwrap())
            .unwrap();
    }
}
/// Explicit offline stack preparation, only used where the test says so.
fn prepared_trigger(g: &mut Game, actor: usize, source: &str) {
    let (r, c) = g.board(source).unwrap();
    let snapshot = g.source_snapshot(c, Some(r));
    g.emit_event(actor, snapshot, Event::Reveal);
    g.drive().unwrap();
    assert!(g.pending.is_none());
    assert_eq!(
        g.stack.last().unwrap().frame.as_ref().unwrap().ability_key,
        KEY
    );
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
        g.room_id = "460000000000000000000001".into();
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
            command_id: format!("jc046-{}-{}", self.name, self.steps.len()),
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
            let intent_id = format!("jc046-intent-{}", self.steps.len());
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
        if let Ok(dir) = std::env::var("JC046_NATIVE_DIR") {
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
    fn pass_top(&mut self) {
        let original = self.room.game.stack.last().expect("stack").id.clone();
        for _ in 0..32 {
            if self.room.game.stack.last().is_none_or(|s| s.id != original) {
                return;
            }
            assert!(self.room.game.pending.is_none());
            let w = self.room.pacing.window.clone().expect("response window");
            let seat = *w
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            self.command(seat, SessionAction::PassResponse { window_id: w.id }, false);
        }
        panic!("top frame did not finish");
    }
    fn choose(&mut self, selected: Vec<String>) {
        let seat = self.room.game.pending.as_ref().unwrap().seat;
        let action = choose_action(&self.room.game, selected);
        self.action(seat, action);
    }
    fn reject(&mut self, seat: usize, action: Action) {
        self.command(seat, SessionAction::Game { action }, true);
    }
    fn reveal(&mut self, actor: usize, source: &str) -> String {
        self.action(actor, self.legal(actor, "reveal", source, None, None));
        self.pass_top();
        assert!(
            self.room.game.pending.is_none(),
            "mandatory reveal must not offer Accept"
        );
        let f = self.room.game.stack.last().unwrap().frame.as_ref().unwrap();
        assert_eq!(f.ability_key, KEY);
        assert_eq!(f.actor, actor);
        assert!(matches!(f.guard, GuardState::Unchecked));
        assert!(self.room.pacing.window.is_some());
        f.source.card.id.clone()
    }
    fn drain(&mut self) {
        for _ in 0..128 {
            if let Some(p) = self.room.game.pending.clone() {
                match p.resolution {
                    ChoiceResolution::Frame {
                        choice: FrameChoice::Sacrifice { .. },
                        ..
                    } => self.choose(vec![p.choice.options[0].id.clone()]),
                    ChoiceResolution::Declare { .. } => self.choose(vec![]),
                    _ => panic!("unexpected pending in sacrifice drain"),
                }
            } else if !self.room.game.stack.is_empty() {
                self.finish();
            } else {
                return;
            }
        }
        panic!("finite sacrifices/death declarations did not finish");
    }
}

#[test]
fn jc046_complete_original_fields_and_closed_definition() {
    let c = catalog::card("JC046");
    assert_eq!(
        (&*c.name, &*c.kind, &*c.color, c.cost),
        ("无面屠戮者", "character", "红", 5)
    );
    assert_eq!(c.subtypes, ["恶魔", "异界"]);
    assert_eq!(c.loyalty, ["红色"]);
    assert_eq!(c.magic_icon, MagicIcon::Blood);
    assert_eq!(c.defense, Some(3));
    assert!(!c.unique);
    assert!(!crate::jc052::printed_leader("JC046"));
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
            investigation: 0,
            combat: 0,
            influence: 1
        }
    );
    assert_eq!(definition("JC046").traits.kill, 2);
    let a = &definition("JC046").abilities[0];
    assert_eq!(a.event, Some(Event::Reveal));
    assert!(a.costs.is_empty() && a.targets.is_empty());
    assert!(matches!(a.response_policy, ResponsePolicy::Respondable));
    for n in 0..12 {
        let mut defs = definitions().clone();
        let d = defs.get_mut("JC046").unwrap();
        match n {
            0 => d.traits.kill = 0,
            1 => d.abilities[0].event = Some(Event::Enter),
            2 => d.abilities[0].response_policy = ResponsePolicy::Immediate,
            3 => d.abilities[0].ops = vec![Op::SacrificeChosen(PlayerRef::Actor)],
            4 => {
                d.abilities[0].ops =
                    vec![Op::ForEachLivingPlayerFromActor(vec![Op::SacrificeChosen(
                        PlayerRef::Context,
                    )])]
            }
            5 => d.abilities[0].costs.push(Cost::Assets(1)),
            6 => d.abilities[0].requires_ready_source = true,
            7 => d.abilities[0].once_per_game = true,
            8 => d.abilities[0].key = "fake-optional-sacrifice".into(),
            9 => d.abilities[0].ops.clear(),
            10 => d.abilities[0].label = "可以牺牲".into(),
            _ => d.abilities.clear(),
        }
        assert!(
            validate_definitions(&defs).is_err(),
            "definition variant {n}"
        );
    }
    let mut defs = definitions().clone();
    defs.insert("JC125".into(), crate::jc046::definition());
    assert!(validate_definitions(&defs).is_err());
    let mut forged = a.clone();
    forged.key = "copied-sacrifice".into();
    assert!(validate_ability("JC125", &forged).is_err());
}

#[test]
fn jc046_real_paid_reveal_is_mandatory_responsive_and_all_seats_start_at_zero() {
    for actor in 0..4 {
        let mut g = game(actor);
        fund(&mut g, actor, "JC042", 5);
        let source = hidden(&mut g, actor, 4);
        let victims = (0..4)
            .map(|s| board(&mut g, "JC125", s, s))
            .collect::<Vec<_>>();
        let mut t = Trace::new(
            ["paid-seat0", "paid-seat1", "paid-seat2", "paid-seat3"][actor],
            g,
        );
        let revealed = t.reveal(actor, &source);
        assert_ne!(source, revealed);
        assert_eq!(t.room.game.resources(actor), 0);
        let f = t.room.game.stack.last().unwrap().frame.as_ref().unwrap();
        assert_eq!(
            f.steps.iter().map(|s| s.context).collect::<Vec<_>>(),
            [0, 1, 2, 3]
        );
        t.finish();
        for seat in 0..4 {
            let p = t.room.game.pending.as_ref().unwrap();
            assert_eq!(p.seat, seat);
            assert_eq!(
                (p.choice.min, p.choice.max, p.choice.allow_decline),
                (Some(1), Some(1), Some(false))
            );
            for viewer in 0..4 {
                assert_eq!(
                    t.room.view(viewer, 0).pending_choice.is_some(),
                    viewer == seat
                );
            }
            t.choose(vec![victims[seat].clone()]);
            assert!(t.room.game.board(&victims[seat]).is_none());
            assert!(t.room.game.players[seat]
                .graveyard
                .iter()
                .any(|c| c.definition == "JC125" && c.id != victims[seat]));
        }
        assert!(t.room.game.pending.is_none() && t.room.game.stack.is_empty());
        assert!(t.room.game.board(&revealed).is_some());
        t.export();
    }
}

#[test]
fn jc046_faceup_deployment_and_concealment_do_not_trigger_reveal() {
    for conceal in [false, true] {
        let mut g = game(0);
        fund(&mut g, 0, "JC042", 5);
        let source = held(&mut g, "JC046", 0);
        let mut t = Trace::new(
            if conceal {
                "conceal-no-trigger"
            } else {
                "deploy-no-trigger"
            },
            g,
        );
        let kind = if conceal { "conceal" } else { "deploy" };
        t.action(0, t.legal(0, kind, &source, None, Some(2)));
        t.finish();
        assert!(t.room.game.pending.is_none() && t.room.game.stack.is_empty());
        let c = t.room.game.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "JC046")
            .unwrap();
        assert_eq!(c.face_down, conceal);
        assert_eq!(t.room.game.resources(0), if conceal { 4 } else { 0 });
        t.export();
    }
}

#[test]
fn jc046_reveal_payment_loyalty_and_extra_command_fields_reject_atomically() {
    for mode in 0..3 {
        let mut g = game(0);
        fund(
            &mut g,
            0,
            if mode == 1 { "JC125" } else { "JC042" },
            if mode == 0 { 4 } else { 5 },
        );
        let source = hidden(&mut g, 0, 0);
        let mut t = Trace::new(
            [
                "reveal-low-assets",
                "reveal-no-red-loyalty",
                "reveal-extra-target",
            ][mode],
            g,
        );
        let mut a = Action {
            card_id: Some(source),
            ..Action::new("reveal")
        };
        if mode == 2 {
            a.target_id = Some("p0".into());
        }
        t.reject(0, a);
        t.export();
    }
}

#[test]
fn jc046_self_sacrifice_is_required_when_alone_and_does_not_cancel_later_players() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 5);
    let old = hidden(&mut g, 0, 0);
    let ally = board(&mut g, "JC125", 1, 4);
    let enemy = board(&mut g, "JC125", 3, 2);
    let hidden_only = board(&mut g, "JC125", 2, 1);
    g.board_mut(&hidden_only).unwrap().face_down = true;
    let mut t = Trace::new("self-sacrifice-continues", g);
    let source = t.reveal(0, &old);
    t.finish();
    assert_eq!(
        t.room.game.pending.as_ref().unwrap().choice.options.len(),
        1
    );
    t.choose(vec![source.clone()]);
    assert!(t.room.game.board(&source).is_none());
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 1);
    t.choose(vec![ally]);
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 3);
    t.choose(vec![enemy]);
    assert!(t.room.game.pending.is_none());
    assert!(t.room.game.board(&hidden_only).unwrap().1.face_down);
    t.export();
}

#[test]
fn jc046_source_can_be_selected_even_with_other_characters() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 5);
    let old = hidden(&mut g, 0, 2);
    let other = board(&mut g, "JC125", 0, 4);
    let mut t = Trace::new("choose-self-among-others", g);
    let source = t.reveal(0, &old);
    t.finish();
    assert_eq!(
        t.room.game.pending.as_ref().unwrap().choice.options.len(),
        2
    );
    t.choose(vec![source]);
    assert!(t.room.game.board(&other).is_some());
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_choices_use_current_control_all_regions_and_exclude_hidden_assets_and_attachments() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 5);
    let old = hidden(&mut g, 0, 2);
    let stolen = board(&mut g, "JC125", 2, 4);
    g.add_control(
        &stolen,
        0,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    g.board_mut(&stolen).unwrap().exhausted = true;
    let given = board(&mut g, "JC125", 0, 0);
    g.add_control(
        &given,
        1,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    let hidden_own = board(&mut g, "JC125", 0, 3);
    g.board_mut(&hidden_own).unwrap().face_down = true;
    let ink = g.make_card("WM038", 0);
    let ink_id = ink.id.clone();
    g.attachments.push(Attachment {
        card: ink,
        host_id: stolen.clone(),
    });
    let mut t = Trace::new("current-control-owner-graves", g);
    let source = t.reveal(0, &old);
    t.finish();
    let ids = t
        .room
        .game
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .map(|o| o.id.clone())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&source) && ids.contains(&stolen));
    assert!(!ids.contains(&given) && !ids.contains(&hidden_own) && !ids.contains(&ink_id));
    t.reject(2, choose_action(&t.room.game, vec![stolen.clone()]));
    t.reject(0, choose_action(&t.room.game, vec![given.clone()]));
    t.choose(vec![stolen.clone()]);
    assert!(t.room.game.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125"));
    assert!(t.room.game.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "WM038"));
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 1);
    t.choose(vec![given]);
    assert!(t.room.game.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125"));
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_illegal_sacrifice_choices_are_strict_noops_and_stale_choice_cannot_repeat() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 5);
    let old = hidden(&mut g, 0, 2);
    let other = board(&mut g, "JC125", 0, 4);
    let ally = board(&mut g, "JC125", 1, 0);
    let mut t = Trace::new("illegal-and-stale-choices", g);
    let source = t.reveal(0, &old);
    t.finish();
    for selected in [
        vec![],
        vec![source.clone(), other.clone()],
        vec![other.clone(), other.clone()],
        vec![ally.clone()],
    ] {
        t.reject(0, choose_action(&t.room.game, selected));
    }
    let mut extra = choose_action(&t.room.game, vec![other.clone()]);
    extra.option = Some("accept".into());
    t.reject(0, extra);
    let stale = choose_action(&t.room.game, vec![other.clone()]);
    t.choose(vec![other]);
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 1);
    t.reject(1, stale);
    t.choose(vec![ally]);
    t.export();
}

#[test]
fn jc046_real_response_bounce_keeps_the_frozen_trigger_and_remaining_sacrifices() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 7);
    let old = hidden(&mut g, 0, 2);
    let rescuer = board(&mut g, "JC075", 0, 0);
    let ally = board(&mut g, "JC125", 1, 4);
    let mut t = Trace::new("response-source-bounce", g);
    let source = t.reveal(0, &old);
    t.response_priority(0);
    t.action(0, t.legal(0, "activate", &rescuer, Some(&source), None));
    t.pass_top();
    assert!(t.room.game.board(&source).is_none());
    assert!(t.room.game.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "JC046"));
    assert_eq!(
        t.room
            .game
            .stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .ability_key,
        KEY
    );
    t.finish();
    t.choose(vec![rescuer]);
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 1);
    t.choose(vec![ally]);
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_real_response_hide_excludes_the_new_hidden_instance_but_keeps_trigger() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 5);
    fund(&mut g, 2, "JC063", 2);
    let old = hidden(&mut g, 0, 2);
    let ally = board(&mut g, "JC125", 1, 4);
    let spell = held(&mut g, "JC063", 2);
    let mut t = Trace::new("response-source-hide", g);
    let source = t.reveal(0, &old);
    t.response_priority(2);
    t.action(2, t.legal(2, "play", &spell, Some(&source), None));
    t.pass_top();
    assert!(t.room.game.board(&source).is_none());
    let hidden = t.room.game.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC046")
        .unwrap();
    assert!(hidden.face_down);
    t.finish();
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 1);
    t.choose(vec![ally]);
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_prepared_source_control_change_preserves_frozen_actor_but_moves_choice_ownership() {
    let mut g = game(0);
    let source = board(&mut g, "JC046", 2, 2);
    g.add_control(
        &source,
        0,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    prepared_trigger(&mut g, 0, &source);
    // Explicit offline layout: a second existing control layer was applied after
    // declaration. This is not a claim that JC129 can be cast in a response window.
    g.add_control(
        &source,
        1,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    let mut t = Trace::new("prepared-source-control-change", g);
    let f = t.room.game.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(
        (f.actor, f.source.card.owner, f.source.card.controller),
        (0, 2, 0)
    );
    assert_eq!(t.room.game.board(&source).unwrap().1.controller, 1);
    t.finish();
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 1);
    t.reject(2, choose_action(&t.room.game, vec![source.clone()]));
    t.choose(vec![source]);
    assert!(t.room.game.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC046"));
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_previous_sacrifice_recomputes_next_seat_after_aura_death_cleanup() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 5);
    let old = hidden(&mut g, 0, 2);
    let aura = board(&mut g, "JC059", 0, 0);
    // JC059 protects only other friendly characters in its own region.
    let doomed = board(&mut g, "JC125", 1, 0);
    g.board_mut(&doomed).unwrap().damage = 1;
    assert_eq!(g.defense(g.board(&doomed).unwrap().1, 0), 2);
    let survivor = board(&mut g, "JC125", 3, 3);
    let mut t = Trace::new("aura-loss-before-next-seat", g);
    t.reveal(0, &old);
    t.finish();
    assert!(t.room.game.board(&doomed).is_some());
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 0);
    let (region, card) = t.room.game.board(&doomed).unwrap();
    assert_eq!((t.room.game.defense(card, region), card.damage), (2, 1));
    t.choose(vec![aura]);
    assert!(t.room.game.board(&doomed).is_none());
    assert!(t.room.game.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125"));
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 3);
    t.choose(vec![survivor]);
    t.export();
}

#[test]
fn jc046_non_target_sacrifice_ignores_barrier_spirit_shield_and_damage_prevention() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 5);
    let old = hidden(&mut g, 0, 2);
    let barrier = board(&mut g, "JZ08", 1, 4);
    let spirit = board(&mut g, "JZ58", 2, 3);
    g.board_mut(&barrier).unwrap().shield = 4;
    g.turn_attribute_modifiers.push(TurnAttributeModifier {
        target_instance: barrier.clone(),
        defense_bonus: 0,
        kill_bonus: 0,
        grants_retreat: false,
        printed_defense_override: None,
        prevents_damage: true,
        ordinary_icons: Icons::default(),
        grants_renown: false,
        expires_turn: g.turn,
    });
    let mut t = Trace::new("sacrifice-bypasses-protection", g);
    let source = t.reveal(0, &old);
    t.finish();
    t.choose(vec![source]);
    t.choose(vec![barrier.clone()]);
    t.choose(vec![spirit.clone()]);
    assert!(t.room.game.board(&barrier).is_none() && t.room.game.board(&spirit).is_none());
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_death_callbacks_wait_until_all_players_and_own_observers_exclude_teammate_death() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 5);
    let old = hidden(&mut g, 0, 2);
    let own = board(&mut g, "JZ31", 0, 4);
    let teammate = board(&mut g, "JC125", 1, 1);
    let enemy = board(&mut g, "JC125", 2, 3);
    let observer = board(&mut g, "JC031", 0, 0);
    held(&mut g, "JC125", 3);
    let contract = g.make_card("JC090", 0);
    g.attachments.push(Attachment {
        card: contract,
        host_id: g.regions[0].card.id.clone(),
    });
    let mut t = Trace::new("death-callbacks-after-all-seats", g);
    t.reveal(0, &old);
    t.finish();
    t.choose(vec![own.clone()]);
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 1);
    assert!(matches!(
        t.room.game.pending.as_ref().unwrap().resolution,
        ChoiceResolution::Frame { .. }
    ));
    t.choose(vec![teammate.clone()]);
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 2);
    t.choose(vec![enemy.clone()]);
    let mut declarations = vec![];
    for _ in 0..16 {
        let Some(p) = t.room.game.pending.clone() else {
            break;
        };
        let ChoiceResolution::Declare { declaration, .. } = p.resolution else {
            panic!("death callback expected");
        };
        declarations.push((
            declaration.source.card.definition.clone(),
            declaration
                .source
                .observed_death
                .as_ref()
                .map(|d| d.card.id.clone()),
        ));
        assert_eq!(declaration.actor, 0);
        t.choose(vec![]);
    }
    assert!(declarations.iter().any(|(d, _)| d == "JZ31"));
    let observed = declarations
        .iter()
        .filter(|(d, _)| d == "JC031" || d == "JC090")
        .collect::<Vec<_>>();
    assert_eq!(observed.len(), 2);
    assert!(observed
        .iter()
        .all(|(_, death)| death.as_deref() == Some(own.as_str())));
    assert!(t.room.game.board(&observer).is_some());
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_prepared_response_destroyed_source_returns_sealed_card_and_preserves_trigger() {
    let mut g = game(0);
    let source = board(&mut g, "JC046", 0, 2);
    let ally = board(&mut g, "JC125", 1, 4);
    let sealed = g.make_card("JC125", 3);
    g.sealed_cards.push(SealedCard {
        card: sealed,
        host_id: source.clone(),
    });
    fund(&mut g, 2, "XQ45", 3);
    let spell = held(&mut g, "XQ45", 2);
    prepared_trigger(&mut g, 0, &source);
    let mut t = Trace::new("prepared-response-source-destroyed", g);
    t.response_priority(2);
    t.action(2, t.legal(2, "play", &spell, Some(&source), None));
    t.pass_top();
    assert!(t.room.game.board(&source).is_none());
    assert!(t.room.game.sealed_cards.is_empty());
    assert!(t.room.game.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC046"));
    assert!(t.room.game.players[3]
        .hand
        .iter()
        .any(|c| c.definition == "JC125"));
    t.finish();
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 1);
    t.choose(vec![ally]);
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_real_florence_free_reveal_is_mandatory_without_assets_and_keeps_exhaustion() {
    let mut g = game(0);
    let old = hidden(&mut g, 0, 2);
    g.board_mut(&old).unwrap().exhausted = true;
    let ally = board(&mut g, "JC125", 1, 4);
    // Explicit prepared region-win event; subsequent accept, free reveal,
    // responsive mandatory frame and sacrifices all use real Room commands.
    let world = g.make_card("DQJC108", 0);
    let source = g.source_snapshot(&world, Some(0));
    g.emit_event(0, source, Event::RegionWon);
    g.drive().unwrap();
    let mut t = Trace::new("prepared-florence-free-reveal", g);
    t.choose(vec!["accept".into()]);
    t.finish();
    assert!(matches!(
        t.room.game.pending.as_ref().unwrap().resolution,
        ChoiceResolution::Frame {
            choice: FrameChoice::FreeReveal { .. },
            ..
        }
    ));
    t.choose(vec![old.clone()]);
    assert!(t.room.game.pending.is_none());
    let f = t.room.game.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(f.ability_key, KEY);
    let source = f.source.card.id.clone();
    assert_ne!(old, source);
    assert!(t.room.game.board(&source).unwrap().1.exhausted);
    assert!(t.room.game.players.iter().all(|p| p.assets.is_empty()));
    t.finish();
    t.choose(vec![source]);
    t.choose(vec![ally]);
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_kill_two_uses_actual_ready_winning_participation_and_stacks() {
    for mode in 0..5 {
        let mut g = game(0);
        let source = board(&mut g, "JC046", 0, if mode == 4 { 0 } else { 2 });
        let (target, damage) = match mode {
            0 => (board(&mut g, "XQ37", 2, 2), 3), // combat 2:1 plus kill 2
            3 => {
                board(&mut g, "JC046", 0, 2);
                board(&mut g, "JC016", 2, 2);
                (board(&mut g, "XQ37", 2, 2), 5)
            } // 4:3 plus 4
            _ => {
                if mode == 1 {
                    g.board_mut(&source).unwrap().exhausted = true;
                }
                if mode == 2 {
                    g.board_mut(&source).unwrap().face_down = true;
                }
                board(&mut g, "XQ37", 0, 2);
                (board(&mut g, "XQ36", 2, 2), 1)
            }
        };
        g.begin_window(Window::Before(2, 1));
        let mut t = Trace::new(
            [
                "combat-kill-two",
                "combat-exhausted",
                "combat-hidden",
                "combat-two-sources",
                "combat-other-region",
            ][mode],
            g,
        );
        for _ in 0..8 {
            if t.room.game.pending.is_some() {
                break;
            }
            let seat = (0..4)
                .find(|s| {
                    t.room
                        .game
                        .legal_actions(*s)
                        .iter()
                        .any(|a| a.action.kind == "pass")
                })
                .unwrap();
            t.action(seat, Action::new("pass"));
        }
        let p = t.room.game.pending.as_ref().unwrap();
        assert_eq!(p.choice.amount, Some(damage));
        assert_eq!(p.seat, 0);
        let a = Action {
            choice_id: Some(p.choice.id.clone()),
            allocations: Some(BTreeMap::from([(target.clone(), damage)])),
            ..Action::new("choose")
        };
        t.action(0, a);
        assert!(t.room.game.board(&target).is_none());
        t.export();
    }
}

#[test]
fn jc046_printed_blood_permanent_combat_and_temporary_influence_stay_distinct() {
    let mut g = game(0);
    let source = board(&mut g, "JC046", 0, 4);
    board(&mut g, "JC060", 1, 0);
    let c = g.board(&source).unwrap().1;
    assert_eq!(g.current_magic_count(c), 1);
    assert!(g.jc060_current_domains(c).is_none());
    assert_eq!(
        g.current_icons(c, 4),
        Icons {
            investigation: 0,
            combat: 2,
            influence: 2
        }
    );
    assert_eq!(g.effective_kill(c), 2);
    g.first_team = 1;
    let c = g.board(&source).unwrap().1;
    assert_eq!(
        g.current_icons(c, 4),
        Icons {
            investigation: 0,
            combat: 2,
            influence: 1
        }
    );
    let raw = serde_json::to_string(&g).unwrap();
    assert_eq!(
        serde_json::to_string(&Game::from_persisted(&raw).unwrap()).unwrap(),
        raw
    );
}

#[test]
fn jc046_initial_eliminated_seats_are_excluded_from_frozen_program() {
    let mut g = game(2);
    g.players[1].eliminated = true;
    fund(&mut g, 2, "JC042", 5);
    let old = hidden(&mut g, 2, 3);
    let other = board(&mut g, "JC125", 0, 4);
    let mut t = Trace::new("pre-eliminated-seat-skip", g);
    let source = t.reveal(2, &old);
    let f = t.room.game.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(
        f.steps.iter().map(|s| s.context).collect::<Vec<_>>(),
        [0, 2, 3]
    );
    t.finish();
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 0);
    t.choose(vec![other]);
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 2);
    t.choose(vec![source]);
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_real_duel_reveal_preserves_two_player_independent_choices() {
    let mut g = Game::new(
        "jc046-duel".into(),
        "LOCAL".into(),
        "duel".into(),
        "P0".into(),
        "watchers".into(),
        9,
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
    for r in &mut g.regions {
        r.cards.clear();
    }
    g.first_team = g.team(1);
    g.begin_window(Window::Action(g.team(1)));
    fund(&mut g, 1, "JC042", 5);
    let old = hidden(&mut g, 1, 2);
    let other = board(&mut g, "JC125", 0, 0);
    let mut t = Trace::new("duel-seat-zero-first", g);
    let source = t.reveal(1, &old);
    assert_eq!(
        t.room
            .game
            .stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .steps
            .len(),
        2
    );
    t.finish();
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 0);
    t.choose(vec![other]);
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 1);
    t.choose(vec![source]);
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_existing_granted_renown_and_combat_glory_remain_optional() {
    for glory in [false, true] {
        let mut g = game(0);
        let source = board(&mut g, "JC046", 0, 2);
        g.turn_attribute_modifiers.push(TurnAttributeModifier {
            target_instance: source.clone(),
            defense_bonus: 0,
            kill_bonus: 0,
            grants_retreat: false,
            printed_defense_override: None,
            prevents_damage: false,
            ordinary_icons: Icons::default(),
            grants_renown: !glory,
            expires_turn: g.turn,
        });
        if glory {
            let curse = g.make_card("JC089", 2);
            g.attachments.push(Attachment {
                card: curse,
                host_id: source,
            });
            let d = g.jc089_combat_glory(0, 2).unwrap();
            g.effects.push_back(Effect::Declare { declaration: d });
        } else {
            g.window = Some(Window::After(2, 2));
            let region = g.regions[2].card.id.clone();
            g.declare_region_renown(2, &region);
        }
        g.drive().unwrap();
        let mut t = Trace::new(
            if glory {
                "prepared-granted-glory"
            } else {
                "prepared-granted-renown"
            },
            g,
        );
        assert_eq!(t.room.game.pending.as_ref().unwrap().choice.min, Some(0));
        let before = t.room.game.regions[2].influence[0];
        t.choose(vec!["accept".into()]);
        t.finish();
        assert_eq!(t.room.game.regions[2].influence[0], before + 1);
        t.export();
    }
}

#[test]
fn jc046_saved_stack_rejects_program_order_actor_cost_and_shell_forgeries() {
    let mut g = game(0);
    let source = board(&mut g, "JC046", 0, 2);
    let alias = board(&mut g, "JC125", 1, 4);
    prepared_trigger(&mut g, 0, &source);
    for n in 0..17 {
        let mut bad = g.clone();
        let item = bad.stack.last_mut().unwrap();
        let f = item.frame.as_mut().unwrap();
        match n {
            0 => f.ability_key = "optional".into(),
            1 => {
                f.steps.remove(1);
            }
            2 => {
                f.steps.swap(0, 1);
            }
            3 => f.steps[1].context = 0,
            4 => f.steps[0].op = Op::SacrificeChosen(PlayerRef::Actor),
            5 => f.steps[0].context = 99,
            6 => f.cursor = 1,
            7 => f.guard = GuardState::Accepted,
            8 => f.actor = 1,
            9 => f.source.card.owner = 1,
            10 => f.source.card.id = alias.clone(),
            11 => f.already_paid.push(PaidCost::Assets(vec![])),
            12 => f.chosen_region = Some(2),
            13 => f.source.card.face_down = true,
            14 => item.controller = 1,
            15 => item.target = Some(alias.clone()),
            _ => item.reveal = true,
        };
        invalid(&g, bad, &format!("stack-{n}"));
    }
    let mut bad = g.clone();
    let f = bad.stack.pop().unwrap().frame.unwrap();
    bad.effects.push_front(Effect::Frame { frame: Box::new(f) });
    invalid(&g, bad, "immediate-bypass");
}

#[test]
fn jc046_saved_sacrifice_pending_rejects_optional_metadata_and_wrong_continuations() {
    let mut g = game(0);
    let source = board(&mut g, "JC046", 0, 2);
    board(&mut g, "JC125", 1, 4);
    prepared_trigger(&mut g, 0, &source);
    let mut t = Trace::new("pending-guard-reference", g);
    t.finish();
    let valid = t.room.game.clone();
    for n in 0..19 {
        let mut bad = valid.clone();
        let p = bad.pending.as_mut().unwrap();
        match n {
            0 => p.choice.min = Some(0),
            1 => p.choice.max = Some(2),
            2 => p.choice.allow_decline = Some(true),
            3 => p.seat = 1,
            4 => p.choice.player_id = player_id(1),
            5 => p.choice.kind = "trigger".into(),
            6 => p.choice.title = "可以牺牲".into(),
            7 => p.choice.options.clear(),
            8 => p.choice.options[0].id = "not-in-play".into(),
            9 => p.choice.description.clear(),
            10 => p.choice.amount = Some(1),
            _ => {
                let ChoiceResolution::Frame { frame, choice } = &mut p.resolution else {
                    panic!()
                };
                match n {
                    11 => frame.cursor = 0,
                    12 => frame.cursor = frame.steps.len() + 1,
                    13 => frame.steps[0].context = 1,
                    14 => frame.guard = GuardState::Unchecked,
                    15 => *choice = FrameChoice::Sacrifice { seat: 1 },
                    16 => frame.source.card.definition = "JC125".into(),
                    17 => frame.steps[0].op = Op::SacrificeChosen(PlayerRef::Target(0)),
                    _ => frame.source.card.controller = 1,
                }
            }
        }
        invalid(&valid, bad, &format!("pending-{n}"));
    }
    let mut bad = valid.clone();
    let p = bad.pending.as_mut().unwrap();
    let ChoiceResolution::Frame { frame, .. } = &p.resolution else {
        panic!()
    };
    let d = Declaration {
        actor: frame.actor,
        source: frame.source.clone(),
        ability: definition("JC046").abilities[0].clone(),
    };
    p.resolution = ChoiceResolution::Declare {
        declaration: d,
        stage: DeclareChoice::Accept,
    };
    p.choice.kind = "trigger".into();
    p.choice.min = Some(0);
    p.choice.allow_decline = Some(true);
    p.choice.options = vec![ChoiceOption {
        id: "accept".into(),
        label: "发动触发能力".into(),
        card: None,
    }];
    invalid(&valid, bad, "forged-optional-declaration");
    t.drain();
    t.export();
}

#[test]
fn jc046_saved_paid_entry_rejects_malformed_payment_and_source_identity() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 5);
    let source = held(&mut g, "JC046", 0);
    let mut t = Trace::new("paid-entry-guard-reference", g);
    t.action(0, t.legal(0, "deploy", &source, None, Some(2)));
    let valid = t.room.game.clone();
    for n in 0..8 {
        let mut bad = valid.clone();
        let item = bad.stack.last_mut().unwrap();
        let f = item.frame.as_mut().unwrap();
        match n {
            0 => f.already_paid.clear(),
            1 => f.already_paid = vec![PaidCost::Assets(vec!["same".into(); 5])],
            2 => {
                f.already_paid = vec![PaidCost::Assets(
                    (0..6).map(|s| format!("asset-{s}")).collect(),
                )]
            }
            3 => f.source.play_source = None,
            4 => f.source.card.face_down = true,
            5 => item.card.as_mut().unwrap().owner = 1,
            6 => item.deploy_region = Some(1),
            _ => item.frame = None,
        }
        invalid(&valid, bad, &format!("entry-{n}"));
    }
    t.finish();
    assert!(t.room.game.pending.is_none());
    t.export();
}

#[test]
fn jc046_sacrificing_control_source_restores_teammate_control_before_their_choice() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 5);
    let old = hidden(&mut g, 0, 2);
    let controller = board(&mut g, "JZ27", 0, 0);
    let victim = board(&mut g, "JC125", 1, 0);
    // Explicit existing, source-bound JZ27 control layout. Its removal and the
    // teammate's subsequent sacrifice are actual consecutive Room commands.
    g.add_control(
        &victim,
        0,
        ControlLifetime::SourceLeaves {
            source_instance: controller.clone(),
        },
        SubtypeChange::AddSlave,
    );
    let mut t = Trace::new("control-source-sacrifice-restores-next-seat", g);
    t.reveal(0, &old);
    t.finish();
    assert_eq!(t.room.game.board(&victim).unwrap().1.controller, 0);
    assert!(t
        .room
        .game
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .any(|o| o.id == victim));
    t.choose(vec![controller]);
    assert_eq!(t.room.game.board(&victim).unwrap().1.controller, 1);
    assert!(!t
        .room
        .game
        .current_subtypes(t.room.game.board(&victim).unwrap().1)
        .contains(&"奴仆".into()));
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 1);
    t.choose(vec![victim]);
    assert!(t.room.game.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125"));
    assert!(t.room.game.pending.is_none());
    t.export();
}

/// Deliberately prepared paused layout: use production sacrifice/control and
/// forced-leader helpers, then snapshot before draining the JC046 continuation.
/// The current scheduler normally continues JC046 before its queued Unique
/// check; this fixture does NOT claim natural Room commands reach this pause.
fn prepared_leader_suspension() -> Game {
    let mut g = game(0);
    let source = board(&mut g, "JC046", 0, 2);
    let controller = board(&mut g, "JZ27", 0, 0);
    let returning = board(&mut g, "JC052", 1, 0);
    board(&mut g, "JC095", 1, 4);
    board(&mut g, "JC125", 2, 3);
    g.add_control(
        &returning,
        0,
        ControlLifetime::SourceLeaves {
            source_instance: controller.clone(),
        },
        SubtypeChange::AddSlave,
    );
    prepared_trigger(&mut g, 0, &source);
    for _ in 0..8 {
        if g.pending.is_some() {
            break;
        }
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(seat, Action::new("pass")).unwrap();
    }
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    let action = choose_action(&g, vec![controller]);
    g.apply_inner(0, action).unwrap();
    g.settle_deaths();
    assert_eq!(g.board(&returning).unwrap().1.controller, 1);
    assert!(g.offer_leader_choice(1));
    assert!(g.validate_jc046_state().is_ok());
    g
}

#[test]
fn jc046_frame_identity_is_unique_across_pending_effects_and_stack() {
    let mut g = game(0);
    let source = board(&mut g, "JC046", 0, 2);
    board(&mut g, "JC125", 1, 4);
    prepared_trigger(&mut g, 0, &source);
    let mut t = Trace::new("duplicate-frame-reference", g);
    t.finish();
    let valid = t.room.game.clone();
    let ChoiceResolution::Frame { frame, .. } = &valid.pending.as_ref().unwrap().resolution else {
        panic!()
    };
    let mut bad = valid.clone();
    bad.effects.push_back(Effect::Frame {
        frame: frame.clone(),
    });
    assert!(bad.validate_jc046_state().unwrap_err().contains("重复"));
    invalid(&valid, bad, "same-frame-in-pending-and-effect");

    let suspended = prepared_leader_suspension();
    let frame = suspended
        .effects
        .iter()
        .find_map(|e| match e {
            Effect::Frame { frame } if frame.ability_key == KEY => Some(frame.clone()),
            _ => None,
        })
        .unwrap();
    let mut bad = suspended.clone();
    bad.effects.push_back(Effect::Frame {
        frame: frame.clone(),
    });
    assert!(bad.validate_jc046_state().unwrap_err().contains("重复"));
    invalid(&suspended, bad, "same-frame-in-two-effects");

    let mut bad = suspended.clone();
    let mut stack_frame = (*frame).clone();
    stack_frame.cursor = 0;
    stack_frame.guard = GuardState::Unchecked;
    bad.stack.push(StackItem {
        id: stack_frame.frame_id.clone(),
        label: format!(
            "{}：{}",
            catalog::card("JC046").name,
            definition("JC046").abilities[0].label
        ),
        controller: stack_frame.actor,
        card: None,
        deploy_region: None,
        reveal: false,
        target: None,
        frame: Some(stack_frame),
    });
    assert!(bad.validate_jc046_state().unwrap_err().contains("重复"));
    invalid(&suspended, bad, "same-frame-in-stack-and-effect");

    // Even a newly invented frame ID cannot turn a second effects copy into a
    // valid paused continuation: a different frame precedes it in the queue.
    let mut bad = suspended.clone();
    let mut different = frame.clone();
    different.frame_id = "invented-second-continuation".into();
    bad.effects.push_back(Effect::Frame { frame: different });
    invalid(&suspended, bad, "two-paused-frames-with-different-ids");
    t.drain();
    t.export();
}

#[test]
fn jc046_prepared_leader_pause_restores_and_resumes_each_remaining_player_once() {
    let g = prepared_leader_suspension();
    let mut t = Trace::new("prepared-leader-paused-continuation", g);
    let frame_id = t
        .room
        .game
        .effects
        .iter()
        .find_map(|e| match e {
            Effect::Frame { frame } if frame.ability_key == KEY => Some(frame.frame_id.clone()),
            _ => None,
        })
        .unwrap();
    let existing = t
        .room
        .game
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .find(|o| t.room.game.board(&o.id).unwrap().1.definition == "JC095")
        .unwrap()
        .id
        .clone();
    t.choose(vec![existing]);
    let p = t.room.game.pending.as_ref().unwrap();
    assert_eq!(p.seat, 1);
    let ChoiceResolution::Frame { frame, .. } = &p.resolution else {
        panic!()
    };
    assert_eq!(frame.frame_id, frame_id);
    assert_eq!(frame.cursor, 2);
    let returning = p.choice.options[0].id.clone();
    assert_eq!(t.room.game.board(&returning).unwrap().1.definition, "JC052");
    t.choose(vec![returning]);
    assert_eq!(t.room.game.pending.as_ref().unwrap().seat, 2);
    let victim = t.room.game.pending.as_ref().unwrap().choice.options[0]
        .id
        .clone();
    t.choose(vec![victim]);
    t.drain();
    assert_eq!(
        t.room.game.players[1]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JC052")
            .count(),
        1
    );
    assert_eq!(
        t.room.game.players[2]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JC125")
            .count(),
        1
    );
    assert!(t.room.game.pending.is_none());
    assert!(!t
        .room
        .game
        .effects
        .iter()
        .any(|e| matches!(e, Effect::Frame { frame } if frame.frame_id == frame_id)));
    t.export();
}

#[test]
fn jc046_effect_containers_and_mandatory_trigger_shell_are_bound() {
    let mut g = game(0);
    let source = board(&mut g, "JC046", 0, 2);
    prepared_trigger(&mut g, 0, &source);
    let mut bad = g.clone();
    assert!(bad.stack.last().unwrap().card.is_none());
    bad.stack.last_mut().unwrap().frame = None;
    invalid(&g, bad, "mandatory-trigger-shell-without-frame");
    let mut t = Trace::new("continuation-container-reference", g);
    t.finish();
    let valid = t.room.game.clone();
    let ChoiceResolution::Frame { frame, .. } = &valid.pending.as_ref().unwrap().resolution else {
        panic!()
    };
    let mut bad = valid.clone();
    let mut forged = frame.clone();
    forged.frame_id = "unrelated-effect-frame".into();
    bad.effects.push_back(Effect::Frame { frame: forged });
    invalid(&valid, bad, "effect-copy-with-unrelated-sacrifice-pending");
    let mut bad = valid.clone();
    bad.pending = None;
    bad.effects.push_back(Effect::Frame {
        frame: frame.clone(),
    });
    invalid(&valid, bad, "orphan-accepted-effect-without-forced-pause");

    let suspended = prepared_leader_suspension();
    let mut bad = suspended.clone();
    let index = bad
        .effects
        .iter()
        .position(|e| matches!(e, Effect::Frame { frame } if frame.ability_key == KEY))
        .unwrap();
    bad.effects
        .insert(index, Effect::Draw { seat: 0, count: 0 });
    invalid(
        &suspended,
        bad,
        "unrelated-effect-before-paused-continuation",
    );
    let mut bad = suspended.clone();
    bad.effects.pop_front();
    invalid(&suspended, bad, "leader-pause-without-required-recheck");
    t.drain();
    t.export();
}

#[test]
fn jc046_real_hide_and_second_reveal_keep_two_independent_mandatory_frames() {
    let mut g = game(0);
    fund(&mut g, 0, "JC042", 10);
    fund(&mut g, 2, "JC063", 2);
    let old = hidden(&mut g, 0, 2);
    let own = board(&mut g, "JC125", 0, 0);
    let ally = board(&mut g, "JC125", 1, 4);
    let spell = held(&mut g, "JC063", 2);
    let mut t = Trace::new("two-real-reveals-independent-frames", g);
    let first_source = t.reveal(0, &old);
    let first_frame = t.room.game.stack.last().unwrap().id.clone();
    t.response_priority(2);
    t.action(2, t.legal(2, "play", &spell, Some(&first_source), None));
    t.pass_top();
    let hidden = t.room.game.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC046")
        .unwrap()
        .id
        .clone();
    assert_ne!(hidden, first_source);
    t.response_priority(0);
    let second_source = t.reveal(0, &hidden);
    assert_ne!(second_source, first_source);
    let second_frame = t.room.game.stack.last().unwrap().id.clone();
    assert_ne!(first_frame, second_frame);
    assert_eq!(
        t.room
            .game
            .stack
            .iter()
            .filter(|s| s.frame.as_ref().is_some_and(|f| f.ability_key == KEY))
            .count(),
        2
    );
    t.finish();
    let p = t.room.game.pending.as_ref().unwrap();
    let ChoiceResolution::Frame { frame, .. } = &p.resolution else {
        panic!()
    };
    assert_eq!(frame.frame_id, second_frame);
    t.choose(vec![own]);
    t.choose(vec![ally]);
    assert!(t.room.game.pending.is_none());
    assert_eq!(t.room.game.stack.last().unwrap().id, first_frame);
    t.finish();
    let p = t.room.game.pending.as_ref().unwrap();
    assert_eq!(p.seat, 0);
    let ChoiceResolution::Frame { frame, .. } = &p.resolution else {
        panic!()
    };
    assert_eq!(frame.frame_id, first_frame);
    t.choose(vec![second_source]);
    assert!(t.room.game.pending.is_none() && t.room.game.stack.is_empty());
    assert_eq!(
        t.room.game.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JC046")
            .count(),
        1
    );
    t.export();
}
