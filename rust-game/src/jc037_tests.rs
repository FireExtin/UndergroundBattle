//! JC037 source admission must fail until the complete card is registered.
use crate::catalog;
#[test]
fn jc037_complete_source_is_admitted() {
    let c = catalog::catalog()
        .cards
        .iter()
        .find(|c| c.id == "JC037")
        .expect("JC037 complete original must be admitted");
    assert_eq!(c.cost, 6);
    assert_eq!(c.loyalty, ["蓝色", "蓝色", "蓝色"]);
    assert_eq!(c.defense, Some(3));
    assert!(c.unique && crate::jc052::printed_leader("JC037"));
    assert_eq!(
        c.permanent_icons,
        crate::model::Icons {
            investigation: 0,
            combat: 2,
            influence: 2
        }
    );
    assert_eq!(
        c.temporary_icons,
        crate::model::Icons {
            investigation: 0,
            combat: 0,
            influence: 1
        }
    );
}

#[test]
fn jc037_counts_current_controlled_hidden_across_regions_not_team_or_owner() {
    let mut g = crate::jc029_tests::game(0);
    let source = crate::jc029_tests::board(&mut g, "JC037", 0, 2);
    for (seat, region) in [(0, 0), (0, 4), (1, 2), (2, 1), (3, 3)] {
        let id = crate::jc029_tests::board(&mut g, "JC125", seat, region);
        g.board_mut(&id).unwrap().face_down = true;
    }
    let (r, c) = g.board(&source).unwrap();
    assert_eq!(g.current_icons(c, r).investigation, 2);
}

use crate::{
    jc029_tests::{board, envelope, fund, game},
    model::*,
    room::*,
    rules::*,
};
use serde_json::{json, Value};
use std::path::Path;

fn hidden(g: &mut Game, owner: usize, region: usize) -> String {
    let id = board(g, "JC125", owner, region);
    g.board_mut(&id).unwrap().face_down = true;
    id
}
fn held(g: &mut Game, def: &str, actor: usize) -> String {
    let c = g.make_card(def, actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn investigation(g: &Game, id: &str) -> u32 {
    let (r, c) = g.board(id).unwrap();
    g.current_icons(c, r).investigation
}
fn restore(g: &Game) {
    let raw = serde_json::to_string(g).unwrap();
    assert_eq!(
        serde_json::to_string(&Game::from_persisted(&raw).unwrap()).unwrap(),
        raw
    );
    let room = envelope(g);
    let state = serde_json::to_string(&room).unwrap();
    let restored = RoomEnvelope::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), state);
    for s in 0..4 {
        assert_eq!(
            serde_json::to_value(room.view(s, 0)).unwrap(),
            serde_json::to_value(restored.view(s, 0)).unwrap()
        );
    }
}

#[test]
fn jc037_bonus_is_permanent_counts_exhausted_hidden_and_uses_personal_controller() {
    for actor in 0..4 {
        for n in 0..=3 {
            let mut g = game(actor);
            g.first_team = 1 - g.team(actor);
            let source = board(&mut g, "JC037", actor, 2);
            for index in 0..n {
                let id = hidden(&mut g, actor, index);
                g.board_mut(&id).unwrap().exhausted = true;
            }
            for other in 0..4 {
                if other != actor {
                    hidden(&mut g, other, 4);
                }
            }
            assert_eq!(investigation(&g, &source), n as u32);
            assert_eq!(g.icons(g.board(&source).unwrap().1, 2).influence, 2);
            g.board_mut(&source).unwrap().exhausted = true;
            assert_eq!(investigation(&g, &source), n as u32);
            assert_eq!(g.icons(g.board(&source).unwrap().1, 2), Icons::default());
            assert_eq!(catalog::card("JC037").permanent_icons.investigation, 0);
            restore(&g);
        }
    }
}
#[test]
fn jc037_source_control_uses_current_controller_not_owner() {
    let mut g = game(0);
    let source = board(&mut g, "JC037", 2, 2);
    hidden(&mut g, 2, 0);
    hidden(&mut g, 3, 1);
    hidden(&mut g, 0, 3);
    assert_eq!(investigation(&g, &source), 1);
    g.add_control(
        &source,
        0,
        ControlLifetime::TurnEnd { turn: g.turn },
        SubtypeChange::None,
    );
    assert_eq!(investigation(&g, &source), 1);
    restore(&g);
    g.control_effects.clear();
    g.settle_controls();
    assert_eq!(g.board(&source).unwrap().1.controller, 2);
    assert_eq!(investigation(&g, &source), 1);
    restore(&g);
}
#[test]
fn jc037_hidden_source_has_no_printed_bonus_and_departed_snapshot_cannot_count() {
    let mut g = game(0);
    let source = board(&mut g, "JC037", 0, 2);
    hidden(&mut g, 0, 4);
    assert_eq!(investigation(&g, &source), 1);
    g.board_mut(&source).unwrap().face_down = true;
    assert_eq!(
        g.current_icons(g.board(&source).unwrap().1, 2),
        Icons {
            influence: 1,
            ..Default::default()
        }
    );
    restore(&g);
    g.board_mut(&source).unwrap().face_down = false;
    let c = g.board(&source).unwrap().1.clone();
    g.return_hand(&source);
    assert_eq!(g.jc037_controlled_hidden(&c), 0);
    restore(&g);
}
#[test]
fn jc037_full_definition_cannot_be_changed_or_transplanted() {
    let mut registry = definitions().clone();
    registry.get_mut("JC037").unwrap().modifiers.clear();
    assert!(validate_definitions(&registry).is_err());
    let mut registry = definitions().clone();
    registry
        .get_mut("JC125")
        .unwrap()
        .modifiers
        .push(StaticModifier::JC037ControlledHiddenInvestigation);
    assert!(validate_definitions(&registry).is_err());
}
#[test]
fn jc037_legal_custom_deck_preserves_existing_copy_and_initial_state_rules() {
    let mut draft = crate::deck::preset("watchers").unwrap();
    draft
        .cards
        .iter_mut()
        .find(|c| c.card_id == "JC125")
        .unwrap()
        .count -= 3;
    draft.cards.push(catalog::DeckEntry {
        card_id: "JC037".into(),
        count: 3,
    });
    assert!(crate::deck::validate(draft.clone()).is_ok());
    let mut g = Game::new_with_deck(
        "jc037-custom".into(),
        "LOCAL".into(),
        "duel".into(),
        "P0".into(),
        draft.clone(),
        37,
    )
    .unwrap();
    g.join_with_deck("P1".into(), draft.clone()).unwrap();
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
    assert_eq!(
        g.players[0]
            .hand
            .iter()
            .chain(&g.players[0].deck)
            .filter(|c| c.definition == "JC037")
            .count(),
        3
    );
    draft.cards.last_mut().unwrap().count = 4;
    draft
        .cards
        .iter_mut()
        .find(|c| c.card_id == "JC125")
        .unwrap()
        .count -= 1;
    assert!(crate::deck::validate(draft).is_err());
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
        g.room_id = "370000000000000000000001".into();
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
            command_id: format!("jc037-{}-{}", self.name, self.steps.len()),
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
            let intent_id = format!("jc037-intent-{}", self.steps.len());
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
        if let Ok(dir) = std::env::var("JC037_NATIVE_DIR") {
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
#[test]
fn jc037_real_paid_deploy_pause_resume_and_four_seat_privacy() {
    let mut g = game(0);
    fund(&mut g, 0, "JC029", 6);
    hidden(&mut g, 0, 0);
    hidden(&mut g, 0, 4);
    let source = held(&mut g, "JC037", 0);
    let mut t = Trace::new("paid-deploy", g);
    t.action(0, t.legal(0, "deploy", &source, None, Some(2)));
    t.finish();
    let source = t
        .room
        .game
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .find(|c| c.definition == "JC037" && !c.face_down)
        .unwrap()
        .id
        .clone();
    assert_eq!(t.room.game.resources(0), 0);
    assert_eq!(investigation(&t.room.game, &source), 2);
    for s in 0..4 {
        let v = serde_json::to_value(t.room.view(s, 0)).unwrap();
        let c = v["regions"][2]["characters"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["cardId"] == "JC037")
            .unwrap();
        assert_eq!(c["icons"]["investigation"], 2);
        if s != 0 {
            for r in v["regions"].as_array().unwrap() {
                for c in r["characters"].as_array().unwrap() {
                    if c["faceDown"] == true {
                        assert!(c.get("cardId").is_none());
                        assert!(c.get("text").is_none());
                    }
                }
            }
        }
    }
    t.command(0, SessionAction::PauseRoom, false);
    t.command(1, SessionAction::ResumeRoom, false);
    t.export();
}
#[test]
fn jc037_real_paid_reveal_counts_other_hidden_only() {
    let mut g = game(0);
    g.first_team = 1;
    fund(&mut g, 0, "JC029", 6);
    let source = board(&mut g, "JC037", 0, 2);
    g.board_mut(&source).unwrap().face_down = true;
    hidden(&mut g, 0, 0);
    hidden(&mut g, 0, 4);
    let mut t = Trace::new("paid-reveal", g);
    t.action(0, t.legal(0, "reveal", &source, None, None));
    t.finish();
    let source = t
        .room
        .game
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .find(|c| c.definition == "JC037" && !c.face_down)
        .unwrap()
        .id
        .clone();
    assert_eq!(t.room.game.resources(0), 0);
    assert_eq!(investigation(&t.room.game, &source), 2);
    assert_eq!(
        t.room
            .game
            .icons(t.room.game.board(&source).unwrap().1, 2)
            .influence,
        2
    );
    t.export();
}
#[test]
fn jc037_real_secret_deployment_updates_bonus_immediately() {
    let mut g = game(0);
    let source = board(&mut g, "JC037", 0, 2);
    fund(&mut g, 0, "JC029", 3);
    let card = held(&mut g, "JC029", 0);
    let mut t = Trace::new("secret-deployment", g);
    assert_eq!(investigation(&t.room.game, &source), 0);
    t.action(0, t.legal(0, "conceal", &card, None, Some(0)));
    t.finish();
    assert_eq!(investigation(&t.room.game, &source), 1);
    t.export();
}
#[test]
fn jc037_real_control_spell_changes_personal_hidden_count_and_preserves_owner() {
    let mut g = game(0);
    let source = board(&mut g, "JC037", 2, 2);
    hidden(&mut g, 2, 0);
    hidden(&mut g, 2, 4);
    hidden(&mut g, 0, 3);
    fund(&mut g, 0, "JC104", 3);
    let spell = held(&mut g, "JC129", 0);
    let mut t = Trace::new("source-control", g);
    assert_eq!(investigation(&t.room.game, &source), 2);
    t.action(0, t.legal(0, "play", &spell, Some(&source), None));
    t.finish();
    assert_eq!(t.room.game.board(&source).unwrap().1.owner, 2);
    assert_eq!(t.room.game.board(&source).unwrap().1.controller, 0);
    assert_eq!(investigation(&t.room.game, &source), 1);
    t.export();
}
#[test]
fn jc037_real_return_resets_source_and_removes_continuous_bonus() {
    let mut g = game(0);
    let source = board(&mut g, "JC037", 0, 2);
    hidden(&mut g, 0, 0);
    let rescuer = board(&mut g, "JC075", 0, 2);
    fund(&mut g, 0, "JC125", 2);
    let old = g.board(&source).unwrap().1.clone();
    let mut t = Trace::new("source-return", g);
    t.action(0, t.legal(0, "activate", &rescuer, Some(&source), None));
    t.finish();
    assert!(t.room.game.board(&source).is_none());
    assert_eq!(t.room.game.jc037_controlled_hidden(&old), 0);
    assert!(t.room.game.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "JC037"));
    t.export();
}
#[test]
fn jc037_different_printed_leader_entry_requires_one_real_sacrifice_and_rejects_empty() {
    let mut g = game(0);
    let prior = board(&mut g, "JC052", 0, 0);
    hidden(&mut g, 0, 4);
    fund(&mut g, 0, "JC029", 6);
    let source = held(&mut g, "JC037", 0);
    let mut t = Trace::new("leader-replace", g);
    t.action(0, t.legal(0, "deploy", &source, None, Some(2)));
    t.finish();
    let source = t
        .room
        .game
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .find(|c| c.definition == "JC037")
        .unwrap()
        .id
        .clone();
    let p = t.room.game.pending.clone().unwrap();
    assert!(matches!(p.resolution, ChoiceResolution::Unique));
    assert_eq!(
        (p.choice.min, p.choice.max, p.choice.allow_decline),
        (Some(1), Some(1), Some(false))
    );
    let choose = |ids| Action {
        choice_id: Some(p.choice.id.clone()),
        selected: Some(ids),
        ..Action::new("choose")
    };
    t.command(
        0,
        SessionAction::Game {
            action: choose(vec![]),
        },
        true,
    );
    t.action(0, choose(vec![prior.clone()]));
    t.finish();
    assert!(t.room.game.board(&prior).is_none());
    assert!(t.room.game.board(&source).is_some());
    assert_eq!(investigation(&t.room.game, &source), 1);
    t.export();
}

#[test]
fn jc037_real_hide_removes_print_and_fresh_hidden_identity_has_no_bonus() {
    let mut g = game(0);
    let source = board(&mut g, "JC037", 0, 0);
    hidden(&mut g, 0, 2);
    fund(&mut g, 0, "XQ16", 2);
    let lawyer = held(&mut g, "XQ16", 0);
    let mut t = Trace::new("source-hide", g);
    t.action(0, t.legal(0, "deploy", &lawyer, None, Some(0)));
    t.finish();
    for _ in 0..4 {
        let Some(p) = t.room.game.pending.clone() else {
            break;
        };
        let selected = if p.choice.options.iter().any(|o| o.id == "accept") {
            vec!["accept".into()]
        } else {
            assert!(p.choice.options.iter().any(|o| o.id == source));
            vec![source.clone()]
        };
        t.action(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(selected),
                ..Action::new("choose")
            },
        );
        t.finish();
    }
    assert!(t.room.game.board(&source).is_none());
    let hidden = t.room.game.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC037")
        .unwrap();
    assert!(hidden.face_down);
    assert_ne!(hidden.id, source);
    assert_eq!(t.room.game.current_icons(hidden, 0).investigation, 0);
    t.export();
}
#[test]
fn jc037_cost_and_loyalty_rejections_preserve_every_state_field() {
    for mode in 0..2 {
        let mut g = game(0);
        fund(
            &mut g,
            0,
            if mode == 0 { "JC029" } else { "JC125" },
            if mode == 0 { 5 } else { 6 },
        );
        let source = held(&mut g, "JC037", 0);
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    card_id: Some(source),
                    region: Some(2),
                    ..Action::new("deploy")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
        restore(&g);
    }
}

#[test]
fn jc037_duel_paid_entry_counts_own_hidden_and_not_opponent_with_two_real_views() {
    let mut g = Game::new(
        "jc037-duel".into(),
        "LOCAL".into(),
        "duel".into(),
        "P0".into(),
        "watchers".into(),
        37,
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
    // Explicit offline layout; only preparation changes state outside commands.
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.cards.clear();
        r.influence = [0, 0];
    }
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    hidden(&mut g, 0, 0);
    hidden(&mut g, 1, 2);
    fund(&mut g, 0, "JC029", 6);
    let source = held(&mut g, "JC037", 0);
    let mut t = Trace::new("duel-paid-deploy", g);
    t.action(0, t.legal(0, "deploy", &source, None, Some(1)));
    t.finish();
    let source = t.room.game.regions[1]
        .cards
        .iter()
        .find(|c| c.definition == "JC037")
        .unwrap()
        .id
        .clone();
    assert_eq!(investigation(&t.room.game, &source), 1);
    assert_eq!(t.room.game.resources(0), 0);
    for seat in 0..2 {
        let view = t.room.view(seat, 0);
        assert_eq!(view.regions.len(), 3);
        assert_eq!(view.players.len(), 2);
        assert_eq!(
            view.regions[1]
                .characters
                .iter()
                .find(|c| c.card_id.as_deref() == Some("JC037"))
                .unwrap()
                .icons
                .unwrap()
                .investigation,
            1
        );
    }
    t.export();
}
