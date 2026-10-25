//! Internal candidate construction only. The ordinary mainline catalog keeps
//! BQ001/BQ009/BQ033 closed. The ignored trajectory runs on a separately hashed
//! catalog variant; it never changes validators, constructs runtime cards, or
//! writes assets, graves, sealed cards, markers or game state after creation.
use crate::{
    catalog::{self, DeckEntry},
    deck::DeckDraft,
    model::*,
    room::{Decision, RoomCommand, RoomEnvelope, SessionAction},
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs::File, io::Write, path::PathBuf};
const CANDIDATES: &[&str] = &["BQ001", "BQ009", "BQ033"];
fn draft(seat: usize) -> DeckDraft {
    let mut entries = if seat == 0 {
        vec![("BQ001", 3), ("BQ009", 3), ("BQ033", 3), ("BQ090", 3)]
    } else {
        vec![("JC125", 3)]
    }
    .into_iter()
    .map(|(id, count)| DeckEntry {
        card_id: id.into(),
        count,
    })
    .collect::<Vec<_>>();
    let mut names = entries
        .iter()
        .map(|e| catalog::card(&e.card_id).name.clone())
        .collect::<Vec<_>>();
    let raw: Value = serde_json::from_str(include_str!("../data/cards.json")).unwrap();
    let unique = raw["cards"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["unique"] == true)
        .map(|c| c["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    let mut groups = ["黄", "蓝", "黑"]
        .iter()
        .map(|color| {
            catalog::catalog()
                .cards
                .iter()
                .filter(|c| {
                    c.supported
                        && c.color == *color
                        && matches!(c.kind.as_str(), "character" | "spell" | "attachment")
                        && crate::deck::copy_limit(c) != Some(1)
                        && !unique.contains(&c.id.as_str())
                        && !c.text.contains("资产")
                        && !names.contains(&c.name)
                })
                .map(|c| c.id.clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut count: usize = entries.iter().map(|e| e.count).sum();
    let mut i = 0;
    while count < 50 {
        let n = i % groups.len();
        i += 1;
        let Some(id) = groups[n].first().cloned() else {
            assert!(!groups.iter().all(Vec::is_empty));
            continue;
        };
        groups[n].remove(0);
        let c = catalog::card(&id);
        if names.contains(&c.name) {
            continue;
        }
        names.push(c.name.clone());
        let n = (50 - count).min(3);
        entries.push(DeckEntry {
            card_id: id,
            count: n,
        });
        count += n;
    }
    let d = DeckDraft {
        id: format!("bq001-candidate-{seat}"),
        name: format!("BQ001 candidate seat {seat}"),
        description:
            "Internal catalog candidate; normal fifty-card construction and real Room commands"
                .into(),
        society_id: None,
        cards: entries,
        rules_version: catalog::RULES_VERSION.into(),
        card_pool_version: catalog::POOL_VERSION.into(),
        engine_version: catalog::ENGINE_VERSION.into(),
        updated_at: "2026-10-25T00:00:00Z".into(),
    };
    let mut names = BTreeMap::<String, usize>::new();
    for e in &d.cards {
        *names
            .entry(catalog::card(&e.card_id).name.clone())
            .or_default() += e.count;
    }
    assert!(names.values().all(|n| *n <= 3));
    assert_eq!(d.cards.iter().map(|e| e.count).sum::<usize>(), 50);
    d
}
#[test]
fn bq001_supply_public_gate_rejects_all_three_closed_cards_at_create_and_join() {
    let ordinary = draft(1);
    crate::deck::validate(ordinary.clone()).unwrap();
    for id in CANDIDATES {
        assert!(!catalog::card(id).supported);
        let mut bad = ordinary.clone();
        bad.cards[0].count -= 1;
        bad.cards.push(DeckEntry {
            card_id: (*id).into(),
            count: 1,
        });
        assert!(Game::new_with_deck(
            "bq001-closed".into(),
            "CLOSED".into(),
            "duel".into(),
            "P0".into(),
            bad.clone(),
            37
        )
        .is_err());
        let mut g = Game::new_with_deck(
            "bq001-closed".into(),
            "CLOSED".into(),
            "duel".into(),
            "P0".into(),
            ordinary.clone(),
            37,
        )
        .unwrap();
        let before = serde_json::to_string(&g).unwrap();
        assert!(g.join_with_deck("P1".into(), bad).is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
    }
}
struct Natural {
    room: RoomEnvelope,
    sequence: usize,
    out: Option<File>,
    seed: u64,
    record: bool,
    region: usize,
    forecast: usize,
    stage: usize,
}
impl Natural {
    fn lobby(seed: u64, record: bool) -> Self {
        let decks = [draft(0), draft(1)];
        let mut g = Game::new_with_deck(
            format!("b001{seed:024x}"),
            "ANCHOR".into(),
            "duel".into(),
            "P0".into(),
            decks[0].clone(),
            seed,
        )
        .unwrap();
        let create = serde_json::to_string(&g).unwrap();
        g.join_with_deck("P1".into(), decks[1].clone()).unwrap();
        let out = if record {
            std::env::var_os("BQ001_NATURAL_OUT_DIR").map(|dir| {
                let p = PathBuf::from(dir);
                std::fs::create_dir_all(&p).unwrap();
                File::options()
                    .write(true)
                    .create_new(true)
                    .open(p.join("bq001-anchor-dream-natural.jsonl"))
                    .unwrap()
            })
        } else {
            None
        };
        let mut t = Self {
            room: RoomEnvelope::from_game(g),
            sequence: 0,
            out,
            seed,
            record,
            region: 0,
            forecast: 0,
            stage: 0,
        };
        if record {
            t.write(&json!({"kind":"initial","format":"bq001-natural-candidate-room-v1","candidateConstructionOnly":CANDIDATES,"publicAdmission":false,"seedDecimal":seed.to_string(),"decks":decks,"createGameState":create,"state":serde_json::to_string(&t.room).unwrap(),"catalog":catalog::catalog(),"views":t.views()}));
        }
        t
    }
    fn write(&mut self, v: &Value) {
        if let Some(f) = &mut self.out {
            serde_json::to_writer(&mut *f, v).unwrap();
            f.write_all(b"\n").unwrap();
            f.flush().unwrap();
        }
    }
    fn views(&self) -> Vec<Value> {
        (0..2)
            .map(|s| serde_json::to_value(self.room.view(s, 0)).unwrap())
            .collect()
    }
    fn step(&mut self, seat: usize, action: SessionAction) {
        assert!(self.sequence < 1800, "bounded command count");
        let before = serde_json::to_string(&self.room).unwrap();
        let command = RoomCommand {
            command_id: format!("bq001-natural-{}", self.sequence),
            expected_version: self.room.revision,
            action,
        };
        if self.record {
            self.write(&json!({"kind":"attempt","seat":seat,"command":command,"serverNowMs":"0"}));
        }
        let expected = self
            .room
            .transition(seat, Some(command.clone()), 0)
            .unwrap();
        assert_eq!(
            expected.outcome, "accepted",
            "step {} stage {} turn {} {:?}: {:?}",
            self.sequence, self.stage, self.room.game.turn, command, expected.error_message
        );
        self.room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        assert_eq!(serde_json::to_string(&self.room).unwrap(), expected.state);
        let duplicate = self
            .room
            .transition(seat, Some(command.clone()), 0)
            .unwrap();
        assert_eq!(duplicate.outcome, "rejected");
        assert!(!duplicate.changed);
        assert_eq!(
            duplicate.state, expected.state,
            "stale duplicate cannot move state"
        );
        let views = self.views();
        for s in 0..2 {
            assert_eq!(
                serde_json::to_value(
                    RoomEnvelope::from_persisted(&expected.state)
                        .unwrap()
                        .view(s, 0)
                )
                .unwrap(),
                views[s]
            );
        }
        if self.record {
            self.write(&json!({"kind":"step","state":before,"seat":seat,"command":command,"serverNowMs":"0","expected":expected,"duplicate":duplicate,"views":views}));
        }
        self.sequence += 1;
    }
    fn action(&mut self, seat: usize, a: Action) {
        self.step(seat, SessionAction::Game { action: a });
    }
    fn rejected(&mut self, seat: usize, a: Action) {
        let before = serde_json::to_string(&self.room).unwrap();
        let command = RoomCommand {
            command_id: format!("bq001-rejected-{}", self.sequence),
            expected_version: self.room.revision,
            action: SessionAction::Game { action: a },
        };
        let rejected = self
            .room
            .transition(seat, Some(command.clone()), 0)
            .unwrap();
        assert_eq!(rejected.outcome, "rejected");
        assert!(!rejected.changed);
        assert_eq!(rejected.state, before);
        self.write(&json!({"kind":"deliberateRejection","seat":seat,"command":command,"expected":rejected,"views":self.views()}));
    }
    fn boot(&mut self) {
        for s in 0..2 {
            self.action(s, Action::new("ready"));
        }
        self.action(0, Action::new("start"));
        self.region = self
            .room
            .game
            .regions
            .iter()
            .position(|r| r.card.definition == "DQJC115")
            .unwrap_or(0);
        self.settle();
    }
    fn definition(&self, id: &str) -> Option<&str> {
        let g = &self.room.game;
        g.players
            .iter()
            .flat_map(|p| {
                p.hand
                    .iter()
                    .chain(&p.deck)
                    .chain(&p.assets)
                    .chain(&p.graveyard)
            })
            .chain(g.regions.iter().flat_map(|r| &r.cards))
            .chain(g.sealed_cards.iter().map(|s| &s.card))
            .find(|c| c.id == id)
            .map(|c| c.definition.as_str())
    }
    fn hand_id(&self, seat: usize, id: &str) -> Option<String> {
        self.room.game.players[seat]
            .hand
            .iter()
            .find(|c| c.definition == id)
            .map(|c| c.id.clone())
    }
    fn keep(&self, seat: usize, id: &str) -> usize {
        if seat == 0 && CANDIDATES.contains(&id) || seat == 0 && id == "BQ090" && self.stage < 3 {
            1
        } else {
            0
        }
    }
    fn selection(&self) -> Vec<String> {
        let p = self.room.game.pending.as_ref().unwrap();
        let options = &p.choice.options;
        let contains = |id: &str| options.iter().any(|o| o.id == id);
        if p.choice.kind == "mulligan" || p.choice.kind == crate::mystic_contest::REWARD_KIND {
            return vec![];
        }
        if p.choice.kind == crate::bq001::CHOICE {
            return vec![options
                .iter()
                .find(|o| self.definition(&o.id) == Some("BQ009"))
                .unwrap()
                .id
                .clone()];
        }
        if contains("forecast") {
            return vec!["forecast".into()];
        }
        if contains("accept") {
            return vec!["accept".into()];
        }
        if p.choice.kind == crate::mystic_contest::COMMIT_KIND {
            let protected = [
                "XG20", "WM021", "WM032", "WM061", "WM043", "BQ064", "CQ18", "WM037", "BQ081",
            ];
            let o = options
                .iter()
                .filter(|o| {
                    self.definition(&o.id)
                        .is_some_and(|d| !protected.contains(&d))
                })
                .min_by_key(|o| {
                    self.definition(&o.id).map_or(1000, |d| {
                        100 * self.keep(p.seat, d) + catalog::card(d).cost as usize
                    })
                })
                .unwrap();
            return vec![o.id.clone()];
        }
        if contains("p1") {
            return vec!["p1".into()];
        }
        let min = p.choice.min.unwrap_or(0) as usize;
        if min == 0 {
            return vec![];
        }
        let mut options = options.iter().collect::<Vec<_>>();
        options.sort_by_key(|o| self.definition(&o.id).map_or(0, |d| self.keep(p.seat, d)));
        assert!(options.len() >= min, "unknown choice {}", p.choice.kind);
        options
            .into_iter()
            .take(min)
            .map(|o| o.id.clone())
            .collect()
    }
    fn pass(&mut self) {
        if let Some(p) = self.room.game.pending.clone() {
            let a = if matches!(
                p.resolution,
                ChoiceResolution::Forecast { .. }
                    | ChoiceResolution::Frame {
                        choice: FrameChoice::Forecast { .. },
                        ..
                    }
            ) {
                self.forecast += 1;
                Action {
                    choice_id: Some(p.choice.id),
                    top: Some(p.choice.options.iter().map(|o| o.id.clone()).collect()),
                    bottom: Some(vec![]),
                    ..Action::new("choose")
                }
            } else {
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(self.selection()),
                    ..Action::new("choose")
                }
            };
            self.action(p.seat, a);
            return;
        }
        if let Some(w) = &self.room.pacing.window {
            let seat = *w
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            let id = w.id.clone();
            self.step(seat, SessionAction::PassResponse { window_id: id });
        } else {
            let seat = (0..2)
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
    fn idle(&self) -> bool {
        self.room.game.pending.is_none() && self.room.game.stack.is_empty()
    }
    fn settle(&mut self) {
        for _ in 0..300 {
            if self.idle() {
                return;
            }
            self.pass();
        }
        panic!("bounded settlement");
    }
    fn asset(&mut self, seat: usize) {
        let g = &self.room.game;
        if !g.can_standard(seat) || g.players[seat].asset_used {
            return;
        }
        let wants_yellow = seat == 0
            && !g.players[seat]
                .assets
                .iter()
                .any(|c| catalog::card(&c.definition).color == "黄");
        let mut actions = g
            .legal_actions(seat)
            .into_iter()
            .filter(|a| a.action.kind == "asset")
            .filter(|a| {
                let d = self.definition(a.action.card_id.as_ref().unwrap()).unwrap();
                g.players[seat]
                    .hand
                    .iter()
                    .filter(|c| c.definition == d)
                    .count()
                    > self.keep(seat, d)
            })
            .collect::<Vec<_>>();
        actions.sort_by_key(|a| {
            usize::from(
                wants_yellow
                    && catalog::card(self.definition(a.action.card_id.as_ref().unwrap()).unwrap())
                        .color
                        != "黄",
            )
        });
        if let Some(a) = actions.into_iter().next() {
            self.action(seat, a.action);
        }
    }
}
#[test]
#[ignore = "Internal candidate catalog variant only; mainline admission remains closed"]
fn bq001_candidate_natural_supply_anchor_forecast_contest_and_dream_payment() {
    assert_eq!(
        std::env::var("BQ001_CATALOG_VARIANT").as_deref(),
        Ok("internal-three-only")
    );
    for id in CANDIDATES {
        assert!(catalog::card(id).supported);
    }
    // First naturally matching opening found by the retained bounded search.
    let seed = 345;
    eprintln!("BQ001 CANDIDATE SELECTED seed={seed}");
    let mut t = Natural::lobby(seed, true);
    t.boot();
    let mut source = String::new();
    let mut sealed_id = String::new();
    let mut contest = Value::Null;
    for _ in 0..1400 {
        assert_eq!(t.room.game.status, "playing");
        assert!(t.room.game.turn <= 16, "bounded natural turn");
        for s in 0..2 {
            t.asset(s);
        }
        if t.stage == 0
            && t.room.game.can_standard(0)
            && t.room.game.resources(0) >= 3
            && t.room.game.players[0]
                .assets
                .iter()
                .any(|c| catalog::card(&c.definition).color == "黄")
        {
            let heat = t.hand_id(0, "BQ033").unwrap();
            let anchor = t.hand_id(0, "BQ009").unwrap();
            assert!(!t
                .room
                .game
                .legal_actions(0)
                .iter()
                .any(|a| a.action.kind == "deploy" && a.action.card_id.as_ref() == Some(&heat)));
            t.rejected(
                0,
                Action {
                    card_id: Some(heat),
                    region: Some(t.region),
                    cost_selected: Some(vec![anchor]),
                    ..Action::new("deploy")
                },
            );
            let hand = t.hand_id(0, "BQ001").unwrap();
            let a = t
                .room
                .game
                .legal_actions(0)
                .into_iter()
                .find(|a| {
                    a.action.kind == "conceal"
                        && a.action.card_id.as_ref() == Some(&hand)
                        && a.action.region == Some(t.region)
                })
                .unwrap()
                .action;
            t.action(0, a);
            t.settle();
            source = t
                .room
                .game
                .regions
                .iter()
                .flat_map(|r| &r.cards)
                .find(|c| c.definition == "BQ001" && c.controller == 0 && c.face_down)
                .unwrap()
                .id
                .clone();
            t.stage = 1;
        }
        if t.stage == 1 && t.room.game.can_fast(0) {
            if let Some(a) =
                t.room.game.legal_actions(0).into_iter().find(|a| {
                    a.action.kind == "reveal" && a.action.card_id.as_ref() == Some(&source)
                })
            {
                let money = t.room.game.resources(0);
                let deck = t.room.game.players[0].deck.len();
                let hand = t.room.game.players[0].hand.len();
                t.action(0, a.action);
                t.settle();
                assert_eq!(t.room.game.resources(0), money - 1);
                assert_eq!(t.room.game.players[0].deck.len(), deck - 1);
                assert_eq!(t.room.game.players[0].hand.len(), hand);
                assert_eq!(t.forecast, 1);
                let sealed = t
                    .room
                    .game
                    .sealed_cards
                    .iter()
                    .find(|s| s.card.definition == "BQ009" && s.host_id == "society:p0")
                    .unwrap();
                sealed_id = sealed.card.id.clone();
                t.stage = 2;
            }
        }
        // Keep the contest and anchor consumption in the same real turn: 2 + 3.
        // The public engine normally clears per-turn contest history next turn.
        if t.stage == 2 && t.room.game.can_standard(0) && t.room.game.resources(0) >= 5 {
            if let Some(a) = t
                .room
                .game
                .legal_actions(0)
                .into_iter()
                .find(|a| a.action.ability_id.as_deref() == Some(crate::import20_contest::HAND))
            {
                t.action(0, a.action);
                t.settle();
                assert_eq!(t.room.game.mystic_history.len(), 1);
                let o = &t.room.game.mystic_history[0];
                let shown = o.shown.iter().find(|s| s.seat == 0).unwrap();
                let w = shown.anchor_witness.as_ref().unwrap();
                assert_eq!((w.sources.len(), w.anchors.len()), (1, 1));
                assert_eq!(t.room.game.bq001_shown_bonus(shown).unwrap(), 1);
                assert_eq!(shown.value, catalog::card(&shown.card.definition).cost + 1);
                assert!(o
                    .shown
                    .iter()
                    .find(|s| s.seat == 1)
                    .unwrap()
                    .anchor_witness
                    .is_none());
                contest = serde_json::to_value(o).unwrap();
                t.stage = 3;
            }
        }
        if t.stage == 3 && t.room.game.can_standard(0) && t.room.game.resources(0) >= 3 {
            let heat = t.hand_id(0, "BQ033").unwrap();
            if let Some(a) = t.room.game.legal_actions(0).into_iter().find(|a| {
                a.action.kind == "deploy"
                    && a.action.card_id.as_ref() == Some(&heat)
                    && a.action.region == Some(t.region)
                    && a.action.cost_selected.as_deref() == Some(&[sealed_id.clone()])
            }) {
                let money = t.room.game.resources(0);
                let enemy = t.room.game.players[1].hand.len();
                assert!(enemy > 0);
                t.action(0, a.action);
                assert_eq!(t.room.game.resources(0), money - 3);
                assert!(t
                    .room
                    .game
                    .sealed_cards
                    .iter()
                    .all(|s| s.card.id != sealed_id));
                let paid = serde_json::to_value(
                    &t.room
                        .game
                        .stack
                        .last()
                        .unwrap()
                        .frame
                        .as_ref()
                        .unwrap()
                        .already_paid,
                )
                .unwrap();
                t.settle();
                assert!(t
                    .room
                    .game
                    .regions
                    .iter()
                    .flat_map(|r| &r.cards)
                    .any(|c| c.definition == "BQ033" && c.controller == 0 && !c.face_down));
                assert_eq!(t.room.game.players[1].hand.len(), enemy - 1);
                assert!(t.room.game.players[0]
                    .deck
                    .iter()
                    .any(|c| c.definition == "BQ009" && c.id != sealed_id));
                assert_eq!(
                    serde_json::to_value(&t.room.game.mystic_history[0]).unwrap(),
                    contest
                );
                t.step(0, SessionAction::PauseRoom);
                t.step(0, SessionAction::ResumeRoom);
                t.write(&json!({"kind":"complete","candidateConstructionOnly":CANDIDATES,"publicAdmission":false,"seedDecimal":t.seed.to_string(),"commands":t.sequence,"turn":t.room.game.turn,"details":{"normalFiftyCardDecks":true,"allPostStartActions":"RoomCommand","realConcealAndPaidReveal":true,"anchorSealedFromHand":true,"forecastChoices":t.forecast,"frozenContest":contest,"paymentReceipt":paid,"sealedAnchorInstance":sealed_id,"dreamEntered":true,"enemyRandomDiscard":true},"finalState":serde_json::to_string(&t.room).unwrap(),"views":t.views()}));
                eprintln!(
                    "BQ001 CANDIDATE COMPLETE seed={} commands={} turn={}",
                    t.seed, t.sequence, t.room.game.turn
                );
                return;
            }
        }
        t.pass();
        t.settle();
    }
    panic!("bounded complete supply path at stage {}", t.stage);
}
