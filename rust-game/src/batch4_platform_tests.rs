//! Narrow ABI95 platform witnesses. Initial offline layouts are explicit;
//! all payments, responses, choices, combat allocations and cleanup use Room commands.
use crate::{catalog, model::*, room::*, unit_support::*};
use serde_json::{json, Value};
struct Witness { room: RoomEnvelope, initial: Value, steps: Vec<Value>, invalid: Vec<Value> }
impl Witness {
    fn new(game: Game) -> Self {
        let room = crate::jc029_tests::envelope(&game);
        let initial = json!({"state":serde_json::to_string(&room).unwrap(),"views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>()});
        Self { room, initial, steps: vec![], invalid: vec![] }
    }
    fn step(&mut self, seat: usize, action: SessionAction) {
        let state = serde_json::to_string(&self.room).unwrap();
        let command = RoomCommand { command_id: format!("abi95-jz56-{}", self.steps.len()), expected_version: self.room.revision, action };
        let expected = self.room.transition(seat, Some(command.clone()), 0).unwrap();
        assert_eq!(expected.outcome, "accepted", "error={:?} command={command:?} pending={:?}", expected.error_message, self.room.game.pending.as_ref().map(|p| &p.choice));
        assert_eq!(serde_json::to_string(&self.room.replay_events(&expected.journal).unwrap()).unwrap(), expected.state);
        self.room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        self.steps.push(json!({"state":state,"seat":seat,"command":command,"serverNowMs":"0","expected":expected,
            "views":(0..4).map(|s|self.room.view(s,0)).collect::<Vec<_>>()}));
    }
    fn game(&mut self, seat: usize, action: Action) { self.step(seat, SessionAction::Game { action }); }
    fn choose(&mut self, selected: Vec<String>) {
        let p = self.room.game.pending.as_ref().unwrap();
        self.game(p.seat, Action { choice_id: Some(p.choice.id.clone()), selected: Some(selected), ..Action::new("choose") });
    }
    fn reject_snapshot(&mut self, label: &str, bad: RoomEnvelope) {
        let state = serde_json::to_string(&bad).unwrap();
        let error = RoomEnvelope::from_persisted(&state).err().expect("corruption must fail native restore");
        assert!(Game::from_persisted(&serde_json::to_string(&bad.game).unwrap()).is_err());
        self.invalid.push(json!({"label":label,"state":state,"nativeError":error,
            "roomId":bad.game.room_id,"inviteCode":bad.game.invite_code,"revision":bad.revision}));
    }
    fn advance(&mut self, subtype: &str, recipient: usize) {
        if let Some(p) = self.room.game.pending.clone() {
            match p.resolution {
                ChoiceResolution::Declare { .. } => {
                    let selected = if p.choice.options.iter().any(|o| o.id == "accept") { vec!["accept".into()] }
                        else { p.choice.options.iter().take(p.choice.min.unwrap_or(0)).map(|o| o.id.clone()).collect() };
                    self.choose(selected);
                },
                ChoiceResolution::Frame { choice: FrameChoice::JZ56Undead, .. } => self.choose(vec![subtype.into()]),
                ChoiceResolution::Recipient { contest, .. } => self.choose(vec![if contest == 1 { player_id(recipient) } else { p.choice.options[0].id.clone() }]),
                ChoiceResolution::Forecast { .. } => self.game(p.seat, Action { choice_id: Some(p.choice.id),
                    top: Some(p.choice.options.iter().rev().map(|o|o.id.clone()).collect()), bottom: Some(vec![]), ..Action::new("choose") }),
                ChoiceResolution::Discard { .. } => self.choose(p.choice.options.iter().take(p.choice.min.unwrap()).map(|o|o.id.clone()).collect()),
                _ => panic!("unexpected platform choice {}", p.choice.kind),
            }
        } else if let Some(window) = self.room.pacing.window.clone() {
            let seat = *window.members.iter().find(|(_, d)| matches!(d, Decision::Undecided { .. })).unwrap().0;
            self.step(seat, SessionAction::PassResponse { window_id: window.id });
        } else {
            let seat = (0..4).find(|s| self.room.game.legal_actions(*s).iter().any(|a| a.action.kind == "pass")).expect("legal pass");
            self.game(seat, Action::new("pass"));
        }
    }
    fn finish(self, name: &str, observations: Value) -> Value {
        let seed = self.room.game.seed.to_string();
        assert!(self.initial["state"].as_str().unwrap().contains(&format!("\"seed\":{seed}")));
        json!({"name":name,"initial":self.initial,"steps":self.steps,"invalid":self.invalid,"seedDecimal":seed,"observations":observations})
    }
}
#[test]
fn batch4_abi95_jz56_native_platform_witnesses() {
    let mut cases = vec![];
    for (name, subtype, character, team, recipient, drop_grave) in [
        ("ghoul-owner", "ghoul", "JZ50", true, 0, false),
        ("zombie-owner", "zombie", "JC085", false, 0, false),
        ("skeleton-owner", "skeleton", "BQ083", false, 0, false),
        ("ghoul-teammate", "ghoul", "JZ50", true, 1, false),
        ("zombie-paid-grave-drop", "zombie", "JC085", false, 0, true),
    ] {
        let mut g = game(); g.room_id = format!("95{:022x}", cases.len() + 1); g.invite_code = "95ABCD".into(); g.seed = u64::MAX - 95; g.random = g.seed;
        let actor = field(&mut g, character, 0); if team { field(&mut g, "JC014", 1); }
        let mut targets = vec![field(&mut g, "LC01", 2)]; if team { targets.push(field(&mut g, "LC01", 3)); }
        let mut grave_card = None;
        for n in 0..7 { let card = g.make_card(if drop_grave && n == 0 { "JC085" } else { "JC125" }, 0);
            if n == 0 && drop_grave { grave_card = Some(card.id.clone()); } g.players[0].graveyard.push(card); }
        fund(&mut g, 0, "JC084", if drop_grave { 4 } else { 2 }); let hand = held(&mut g, "JZ56", 0);
        let mut w = Witness::new(g);
        w.game(0, Action { card_id: Some(hand), target_id: Some("society:p0".into()), ..Action::new("play") });
        for _ in 0..48 {
            if w.room.game.pending.as_ref().is_some_and(|p| matches!(p.resolution, ChoiceResolution::Frame { choice: FrameChoice::JZ56Undead, .. })) {
                let mut bad = w.room.clone(); bad.game.attachments.iter_mut().find(|a| a.card.definition == "JZ56").unwrap().card.chosen_subtype = Some(crate::batch4_night::UndeadKind::Ghoul);
                w.reject_snapshot("already-chosen-with-pending", bad);
            }
            if w.room.game.attachments.iter().any(|a| a.card.definition == "JZ56" && a.card.chosen_subtype.is_some()) { break; }
            w.advance(subtype, recipient);
        }
        assert!(w.room.game.attachments.iter().any(|a| a.card.definition == "JZ56" && a.card.chosen_subtype.is_some()));
        if let Some(id) = grave_card {
            w.game(0, Action { card_id: Some(id), region: Some(0), ..Action::new("deploy") });
            assert_eq!(w.room.game.players[0].graveyard.len(), 6);
            assert_eq!(w.room.game.effective_wound(w.room.game.board(&actor).unwrap().1), 0);
        }
        for _ in 0..160 {
            if let Some(p) = &w.room.game.pending {
                if matches!(p.resolution, ChoiceResolution::Damage { .. }) { break; }
                if matches!(p.resolution, ChoiceResolution::Recipient { contest: 1, .. }) {
                    let mut bad = w.room.clone(); bad.game.pending.as_mut().unwrap().choice.min = Some(0); w.reject_snapshot("recipient-empty-selection", bad);
                    let mut bad = w.room.clone(); bad.game.pending.as_mut().unwrap().choice.options.push(ChoiceOption { id: "p2".into(), label: "P2".into(), card: None }); w.reject_snapshot("recipient-enemy-option", bad);
                }
            }
            w.advance(subtype, recipient);
        }
        let p = w.room.game.pending.clone().expect("combat allocation"); assert!(matches!(p.resolution, ChoiceResolution::Damage { .. }));
        assert_eq!(p.seat, recipient); let amount = p.choice.amount.unwrap();
        if !drop_grave && recipient == 0 {
            let mut bad = w.room.clone(); if let ChoiceResolution::Damage { wound_sources, .. } = &mut bad.game.pending.as_mut().unwrap().resolution { wound_sources.clear(); }
            w.reject_snapshot("missing-wound-participant", bad);
        }
        let mut allocations = std::collections::BTreeMap::new();
        if team { assert_eq!(amount, 4); allocations.insert(targets[0].clone(), 2); allocations.insert(targets[1].clone(), 2); }
        else { assert!(amount < 4); allocations.insert(targets[0].clone(), amount); }
        w.game(recipient, Action { choice_id: Some(p.choice.id), allocations: Some(allocations), ..Action::new("choose") });
        let wound = !drop_grave && recipient == 0;
        for id in &targets { let c = w.room.game.board(id).expect("nonlethal target").1; let assigned = if team { 2 } else { amount };
            assert_eq!((c.damage, c.wounds), if wound { (0, assigned) } else { (assigned, 0) }); }
        if name == "ghoul-owner" {
            // Separate explicit End-window layout derived from the actual wound
            // result; no claim that skipped region windows were browser-played.
            let mut end_game = w.room.game.clone(); end_game.begin_window(Window::End); let mut cleanup = Witness::new(end_game); let turn = cleanup.room.game.turn;
            for _ in 0..64 { if cleanup.room.game.turn != turn { break; } cleanup.advance(subtype, recipient); }
            assert_eq!(cleanup.room.game.turn, turn + 1);
            for id in &targets { let c = cleanup.room.game.board(id).unwrap().1; assert_eq!((c.damage,c.wounds),(0,2)); }
            cases.push(cleanup.finish("wounds-survive-cleanup", json!({"woundsRetained":2,"ordinaryDamage":0})));
        }
        cases.push(w.finish(name, json!({"subtype":subtype,"recipient":recipient,"graveDropped":drop_grave,"wound":wound,"amount":amount,"targetInstances":targets})));
    }
    assert_eq!(cases.len(), 6);
    if let Ok(file) = std::env::var("BATCH4_ABI95_NATIVE_OUT") {
        std::fs::write(file, serde_json::to_vec(&json!({"format":"batch4-abi95-native-witness-v1","method":"Explicit offline initial layouts; real Room commands through normal phases, except separately disclosed End-window layout. No browser/public acceptance.","catalog":catalog::catalog(),"cases":cases})).unwrap()).unwrap();
    }
}
