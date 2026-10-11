//! Explicit layouts followed by continuous, paid Room commands. Every step
//! restores both loaders and replays the actual journal; no mid-game edits.
use crate::{catalog, jc029_tests, model::*, room::*, rules::Event};

struct Case {
    name: String,
    room: RoomEnvelope,
    step: usize,
    renown_choices: std::collections::BTreeSet<String>,
    return_orders: std::collections::BTreeMap<(usize, usize), Vec<(String, String)>>,
}
impl Case {
    fn new(name: &str, game: Game) -> Self {
        let case = Self {
            name: name.into(),
            room: jc029_tests::envelope(&game),
            step: 0,
            renown_choices: Default::default(),
            return_orders: Default::default(),
        };
        case.checkpoint();
        case
    }
    fn checkpoint(&self) {
        let state = serde_json::to_string(&self.room).unwrap();
        let restored = RoomEnvelope::from_persisted(&state).unwrap();
        assert_eq!(serde_json::to_string(&restored).unwrap(), state);
        let game = serde_json::to_string(&self.room.game).unwrap();
        assert_eq!(
            serde_json::to_string(&Game::from_persisted(&game).unwrap()).unwrap(),
            game
        );
        let views: Vec<_> = (0..4)
            .map(|seat| serde_json::to_value(self.room.view(seat, 0)).unwrap())
            .collect();
        for (seat, view) in views.iter().enumerate() {
            assert_eq!(*view, serde_json::to_value(restored.view(seat, 0)).unwrap());
        }
        self.export(
            "checkpoint",
            serde_json::json!({"state": state, "views": views}),
        );
    }
    fn export(&self, kind: &str, value: serde_json::Value) {
        if let Ok(root) = std::env::var("WIN_FLOW_EVIDENCE_DIR") {
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(
                format!("{root}/{}-{}-{kind}.json", self.name, self.step),
                serde_json::to_vec(&value).unwrap(),
            )
            .unwrap();
        }
    }
    fn command(&mut self, seat: usize, action: SessionAction) {
        let state = serde_json::to_string(&self.room).unwrap();
        let command = RoomCommand {
            command_id: format!("{}-{}", self.name, self.step),
            expected_version: self.room.revision,
            action,
        };
        let expected = self
            .room
            .transition(seat, Some(command.clone()), 0)
            .unwrap();
        self.export(
            "step",
            serde_json::json!({
                "state": state, "seat": seat, "command": command,
                "serverNowMs": "0", "expected": expected,
                "views": (0..4).map(|s|
                    RoomEnvelope::from_persisted(&expected.state).unwrap().view(s, 0)
                ).collect::<Vec<_>>()
            }),
        );
        assert!(
            expected.error_code.is_none(),
            "{:?}",
            expected.error_message
        );
        assert_eq!(
            serde_json::to_string(&self.room.replay_events(&expected.journal).unwrap()).unwrap(),
            expected.state
        );
        self.room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        if let Some(p) = &self.room.game.pending {
            if matches!(&p.resolution, ChoiceResolution::Declare { declaration, .. }
                if declaration.ability.key == "renown")
            {
                self.renown_choices.insert(p.choice.id.clone());
            }
        }
        self.step += 1;
        self.checkpoint();
    }
    fn game(&mut self, seat: usize, action: Action) {
        if let Some(pending) = &self.room.game.pending {
            if let ChoiceResolution::Bottom { region } = &pending.resolution {
                if action.kind == "choose" && action.choice_id.as_ref() == Some(&pending.choice.id)
                {
                    let ordered = action
                        .selected
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|id| {
                            let card = self.room.game.board(id).unwrap().1;
                            (card.id.clone(), card.definition.clone())
                        })
                        .collect();
                    self.return_orders.insert((seat, *region), ordered);
                }
            }
        }
        if let Some(window) = self.room.pacing.window.clone() {
            let intent_id = format!("{}-intent-{}", self.name, self.step);
            self.command(
                seat,
                SessionAction::BeginResponse {
                    window_id: window.id.clone(),
                    intent_id: intent_id.clone(),
                },
            );
            self.command(
                seat,
                SessionAction::SubmitResponse {
                    window_id: window.id,
                    intent_id,
                    action,
                },
            );
        } else {
            self.command(seat, SessionAction::Game { action });
        }
    }
    fn reject(&mut self, seat: usize, action: SessionAction, code: &str) {
        let state = serde_json::to_string(&self.room).unwrap();
        let command = RoomCommand {
            command_id: format!("{}-rejected-{}", self.name, self.step),
            expected_version: self.room.revision,
            action,
        };
        let expected = self
            .room
            .transition(seat, Some(command.clone()), 0)
            .unwrap();
        assert_eq!(expected.error_code.as_deref(), Some(code));
        assert_eq!(expected.state, state);
        assert!(!expected.changed && expected.journal.is_empty());
        self.export(
            "step",
            serde_json::json!({"state": state, "seat": seat,
            "command": command, "serverNowMs": "0", "expected": expected,
            "views": (0..4).map(|s| self.room.view(s, 0)).collect::<Vec<_>>() }),
        );
        self.step += 1;
        self.checkpoint();
    }
    fn advance(&mut self) {
        if let Some(pending) = self.room.game.pending.clone() {
            let selected = if matches!(&pending.resolution,
                ChoiceResolution::Declare { declaration, .. }
                    if declaration.ability.event == Some(Event::RegionWon))
            {
                vec![]
            } else {
                pending
                    .choice
                    .options
                    .iter()
                    .take(pending.choice.min.unwrap_or(0))
                    .map(|o| o.id.clone())
                    .collect()
            };
            self.game(
                pending.seat,
                Action {
                    choice_id: Some(pending.choice.id),
                    selected: Some(selected),
                    ..Action::new("choose")
                },
            );
        } else if let Some(window) = self.room.pacing.window.clone() {
            let seat = *window
                .members
                .iter()
                .find(|(_, decision)| matches!(decision, Decision::Undecided { .. }))
                .unwrap()
                .0;
            self.command(
                seat,
                SessionAction::PassResponse {
                    window_id: window.id,
                },
            );
        } else {
            let seat = (0..4)
                .find(|s| {
                    self.room
                        .game
                        .legal_actions(*s)
                        .iter()
                        .any(|a| a.action.kind == "pass")
                })
                .unwrap();
            self.game(seat, Action::new("pass"));
        }
    }
    fn until(&mut self, predicate: impl Fn(&Game) -> bool) {
        for _ in 0..160 {
            if predicate(&self.room.game) {
                return;
            }
            self.advance();
        }
        panic!("bounded win flow {}", self.name);
    }
    fn finish_win(&mut self, score_count: usize) {
        self.until(|g| g.players.iter().map(|p| p.score_cards.len()).sum::<usize>() == score_count);
        while self.room.game.pending.is_some() {
            self.advance();
        }
    }
}
fn region(g: &mut Game, index: usize, definition: &str) {
    g.regions[index].card = g.make_card(definition, 0);
    g.regions[index].influence = [0, 0];
}
fn held(g: &mut Game, definition: &str, seat: usize) -> String {
    let card = g.make_card(definition, seat);
    let id = card.id.clone();
    g.players[seat].hand.push(card);
    id
}
fn accept_entry(case: &mut Case, region: usize, actor: usize) {
    case.until(|g| g.pending.is_some());
    let p = case.room.game.pending.clone().unwrap();
    assert!(
        matches!(&p.resolution, ChoiceResolution::Declare { declaration, .. }
        if declaration.ability.event == Some(Event::Enter))
    );
    case.game(
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec!["accept".into()]),
            ..Action::new("choose")
        },
    );
    case.until(|g| g.window == Some(Window::Win(region, actor)));
}

#[test]
fn win_flow_paid_xq37_and_jz22_entries_resume_both_action_steps() {
    for definition in ["XQ37", "JZ22"] {
        for actor in [0, 2] {
            for first_team in [0, 1] {
                let mut g = jc029_tests::game(actor);
                let team = g.team(actor);
                g.first_team = first_team;
                region(&mut g, 2, "DQJC112");
                g.regions[2].influence[team] = 3;
                jc029_tests::fund(&mut g, actor, definition, 2);
                let source = held(&mut g, definition, actor);
                let asset = held(&mut g, "JC125", actor);
                // Actual owner ordering must survive persistence during the win.
                jc029_tests::board(&mut g, "JC125", actor, 2);
                jc029_tests::board(&mut g, "JC125", actor, 2);
                let mut case =
                    Case::new(&format!("entry-{definition}-{actor}-first-{first_team}"), g);
                case.game(
                    actor,
                    Action {
                        card_id: Some(source),
                        region: Some(2),
                        ..Action::new("deploy")
                    },
                );
                accept_entry(&mut case, 2, actor);
                case.finish_win(1);
                assert_eq!(case.room.game.window, Some(Window::Action(team)));
                assert_eq!(case.room.game.active_team, team);
                assert_eq!(case.room.game.priority_team, team);
                case.game(
                    actor,
                    Action {
                        card_id: Some(asset),
                        ..Action::new("asset")
                    },
                );
                assert!(case.room.game.players[actor].asset_used);
                let next = if first_team == team {
                    Window::Action(1 - team)
                } else {
                    Window::Mobility
                };
                case.until(|g| g.window != Some(Window::Action(team)));
                assert_eq!(case.room.game.window, Some(next));
            }
        }
    }
}

#[test]
fn win_flow_paid_reveal_restores_other_steps_and_skips_only_captured_contest() {
    for original in [
        Window::Prepare,
        Window::Draw,
        Window::Mobility,
        Window::End,
        Window::Before(0, 0),
        Window::After(0, 1),
        Window::Before(2, 0),
        Window::After(2, 1),
    ] {
        let mut g = jc029_tests::game(0);
        region(&mut g, 2, "DQJC112");
        g.regions[2].influence = [3, 0];
        jc029_tests::fund(&mut g, 0, "XQ37", 2);
        let source = jc029_tests::board(&mut g, "XQ37", 0, 2);
        g.board_mut(&source).unwrap().face_down = true;
        g.begin_window(original.clone());
        let expected = match original {
            Window::Before(2, _) | Window::After(2, _) => Window::After(2, 2),
            _ => original.clone(),
        };
        let mut case = Case::new(&format!("reveal-{original:?}"), g);
        case.game(
            0,
            Action {
                card_id: Some(source),
                ..Action::new("reveal")
            },
        );
        accept_entry(&mut case, 2, 0);
        case.finish_win(1);
        assert_eq!(case.room.game.window, Some(expected));
    }
}

#[test]
fn win_flow_nested_paid_reveals_resume_outer_win_then_original_action() {
    nested_reveals(Window::Action(0));
}

#[test]
fn win_flow_nested_capture_skips_replacement_in_saved_contest() {
    for original in [Window::Before(1, 0), Window::After(1, 1)] {
        nested_reveals(original);
    }
}

fn nested_reveals(original: Window) {
    let mut g = jc029_tests::game(0);
    let mut sources = vec![];
    for (seat, r) in [(0, 0), (1, 1)] {
        region(&mut g, r, if r == 0 { "DQJC115" } else { "DQJC109" });
        g.regions[r].influence = [2, 0];
        jc029_tests::fund(&mut g, seat, "JZ22", 2);
        let id = jc029_tests::board(&mut g, "JZ22", seat, r);
        g.board_mut(&id).unwrap().face_down = true;
        jc029_tests::board(&mut g, "JC125", seat, r);
        sources.push(id);
    }
    let old_instances: Vec<_> = g.regions[..2].iter().map(|r| r.card.id.clone()).collect();
    let deck_prefixes: Vec<_> = g.players[..2]
        .iter()
        .map(|p| serde_json::to_value(&p.deck).unwrap())
        .collect();
    g.begin_window(original.clone());
    let mut case = Case::new(&format!("nested-reveal-{original:?}"), g);
    case.game(
        0,
        Action {
            card_id: Some(sources[0].clone()),
            ..Action::new("reveal")
        },
    );
    accept_entry(&mut case, 0, 0);
    case.game(
        1,
        Action {
            card_id: Some(sources[1].clone()),
            ..Action::new("reveal")
        },
    );
    accept_entry(&mut case, 1, 1);
    assert_eq!(case.room.game.win_contexts.len(), 2);
    // Closing the inner win restores A before B's owners finish ordering.
    case.until(|g| {
        g.pending
            .as_ref()
            .is_some_and(|p| p.choice.kind == "region_return")
    });
    assert_eq!(case.room.game.window, Some(Window::Win(0, 0)));
    assert_eq!(case.room.game.region_return.as_ref().unwrap().region, 1);
    assert_eq!(case.room.game.win_contexts.len(), 1);
    let contexts = serde_json::to_value(&case.room.game.win_contexts).unwrap();
    case.command(0, SessionAction::PauseRoom);
    case.reject(
        1,
        SessionAction::Game {
            action: Action::new("pass"),
        },
        "room_paused",
    );
    case.command(1, SessionAction::ResumeRoom);
    assert_eq!(
        serde_json::to_value(&case.room.game.win_contexts).unwrap(),
        contexts
    );
    case.finish_win(1);
    assert_eq!(case.room.game.window, Some(Window::Win(0, 0)));
    assert!(case.room.game.players[0].score_cards.is_empty());
    assert_eq!(
        case.room.game.players[1].score_cards[0].definition,
        "DQJC109"
    );
    assert_ne!(case.room.game.regions[1].card.id, old_instances[1]);
    assert_eq!(case.room.game.regions[0].card.id, old_instances[0]);
    case.finish_win(2);
    let expected = match original {
        Window::Before(1, _) | Window::After(1, _) => Window::After(1, 2),
        _ => original,
    };
    assert_eq!(case.room.game.window, Some(expected));
    assert_eq!(
        case.room.game.players[0].score_cards[0].definition,
        "DQJC115"
    );
    assert_eq!(case.room.game.players[1].score_cards.len(), 1);
    for (r, old) in old_instances.iter().enumerate() {
        assert_ne!(&case.room.game.regions[r].card.id, old);
        assert!(case.room.game.regions[r].cards.is_empty());
        assert_eq!(case.room.game.regions[r].influence, [0, 0]);
        let deck = &case.room.game.players[r].deck;
        let prefix_len = deck_prefixes[r].as_array().unwrap().len();
        assert_eq!(deck.len(), prefix_len + 2);
        assert_eq!(
            serde_json::to_value(&deck[..prefix_len]).unwrap(),
            deck_prefixes[r]
        );
        let ordered = &case.return_orders[&(r, r)];
        assert_eq!(ordered.len(), 2);
        let definitions: std::collections::BTreeSet<_> =
            ordered.iter().map(|(_, d)| d.as_str()).collect();
        assert_eq!(definitions, ["JZ22", "JC125"].into_iter().collect());
        for (i, (old_id, definition)) in ordered.iter().enumerate() {
            let returned = &deck[prefix_len + i];
            assert_eq!(&returned.definition, definition);
            assert_eq!((returned.owner, returned.controller), (r, r));
            assert_ne!(&returned.id, old_id);
            assert!(case.room.game.board(old_id).is_none());
        }
    }
    assert!(case.room.game.win_contexts.is_empty());
}

#[test]
fn win_flow_corrupted_contexts_reject_in_game_and_room_loaders() {
    let mut g = jc029_tests::game(0);
    for (seat, r) in [(0, 0), (1, 1)] {
        region(&mut g, r, "DQJC115");
        g.regions[r].influence = [2, 0];
        jc029_tests::fund(&mut g, seat, "JZ22", 2);
        let id = jc029_tests::board(&mut g, "JZ22", seat, r);
        g.board_mut(&id).unwrap().face_down = true;
    }
    let sources: Vec<_> = (0..2).map(|r| g.regions[r].cards[0].id.clone()).collect();
    let mut case = Case::new("context-guards", g);
    for (seat, source) in sources.into_iter().enumerate() {
        case.game(
            seat,
            Action {
                card_id: Some(source),
                ..Action::new("reveal")
            },
        );
        accept_entry(&mut case, seat, seat);
    }
    let good = serde_json::to_value(&case.room).unwrap();
    for mutation in 0..15 {
        let mut bad = good.clone();
        match mutation {
            0 => bad["game"]["win_contexts"] = serde_json::json!([]),
            1 => bad["game"]["win_contexts"][1]["region"] = serde_json::json!(99),
            2 => bad["game"]["win_contexts"][1]["seat"] = serde_json::json!(99),
            3 => bad["game"]["win_contexts"][1]["region_instance"] = serde_json::json!("stale"),
            4 => bad["game"]["win_contexts"][1] = bad["game"]["win_contexts"][0].clone(),
            5 => bad["game"]["win_contexts"][0]["resume_window"] = serde_json::json!({"Win":[0,0]}),
            6 => bad["game"]["win_contexts"][1]["resume_window"] = serde_json::json!({"Win":[1,1]}),
            7 => bad["game"]["win_contexts"][1]["resume_window"] = serde_json::json!({"Action":0}),
            8 => bad["game"]["win_contexts"][0]["resume_window"] = serde_json::json!({"Action":2}),
            9 => {
                bad["game"]["win_contexts"][0]["resume_window"] =
                    serde_json::json!({"Before":[99,0]})
            }
            10 => {
                bad["game"]["win_contexts"][0]["resume_window"] = serde_json::json!({"After":[0,3]})
            }
            11 => bad["game"]["window"] = serde_json::json!({"Action":0}),
            12 => bad["game"]["status"] = serde_json::json!("finished"),
            13 => {
                bad["game"]["win_contexts"][0]["resume_window"] =
                    serde_json::json!({"Before":[1,0]})
            }
            _ => {
                bad["game"]["win_contexts"][0]["resume_window"] = serde_json::json!({"After":[1,1]})
            }
        }
        assert!(
            Game::from_persisted(&bad["game"].to_string()).is_err(),
            "mutation {mutation}"
        );
        assert!(
            RoomEnvelope::from_persisted(&bad.to_string()).is_err(),
            "mutation {mutation}"
        );
        if let Ok(root) = std::env::var("WIN_FLOW_EVIDENCE_DIR") {
            std::fs::create_dir_all(format!("{root}/invalid")).unwrap();
            std::fs::write(
                format!("{root}/invalid/context-{mutation}.json"),
                serde_json::to_vec(&serde_json::json!({"state":bad.to_string()})).unwrap(),
            )
            .unwrap();
        }
    }
}

#[test]
fn win_flow_duplicate_and_stale_awards_cannot_capture_a_replacement() {
    let mut g = jc029_tests::game(0);
    region(&mut g, 2, "DQJC112");
    g.place_influence(0, 2, 4);
    g.place_influence(0, 2, 1);
    let mut case = Case::new("duplicate-awards", g);
    case.advance();
    assert_eq!(case.room.game.window, Some(Window::Win(2, 0)));
    assert_eq!(case.room.game.win_contexts.len(), 1);
    case.finish_win(1);
    assert!(case.room.game.win_contexts.is_empty());
    assert_eq!(case.room.game.regions[2].influence, [0, 0]);

    let mut g = jc029_tests::game(0);
    let old_instance = g.regions[2].card.id.clone();
    region(&mut g, 2, "DQJC112");
    g.effects.push_back(Effect::Award {
        seat: 0,
        region: 2,
        region_instance: old_instance,
    });
    let mut case = Case::new("stale-award", g);
    case.advance();
    assert_eq!(case.room.game.window, Some(Window::Action(0)));
    assert!(case.room.game.win_contexts.is_empty());
    assert!(case.room.game.players[0].score_cards.is_empty());
}

#[test]
fn win_flow_inner_score_terminal_clears_outer_context_and_restart_is_fresh() {
    let mut g = jc029_tests::game(0);
    for _ in 0..2 {
        let score = g.make_card("DQJC107", 0);
        g.players[0].score_cards.push(score);
    }
    let mut sources = vec![];
    for (seat, r) in [(0, 0), (1, 1)] {
        region(&mut g, r, "DQJC115");
        g.regions[r].influence = [2, 0];
        jc029_tests::fund(&mut g, seat, "JZ22", 2);
        let id = jc029_tests::board(&mut g, "JZ22", seat, r);
        g.board_mut(&id).unwrap().face_down = true;
        sources.push(id);
    }
    let outer_instance = g.regions[0].card.id.clone();
    let inner_instance = g.regions[1].card.id.clone();
    let mut case = Case::new("inner-win-terminal", g);
    for (seat, source) in sources.into_iter().enumerate() {
        case.game(
            seat,
            Action {
                card_id: Some(source),
                ..Action::new("reveal")
            },
        );
        accept_entry(&mut case, seat, seat);
    }
    case.until(|g| g.status == "finished");
    assert_eq!(case.room.game.winner_team, Some(0));
    let points: u32 = case.room.game.players[..2]
        .iter()
        .flat_map(|p| &p.score_cards)
        .map(|c| catalog::card(&c.definition).points.unwrap_or(0))
        .sum();
    assert_eq!(points, 10);
    assert_eq!(case.room.game.players[0].score_cards.len(), 2);
    assert_eq!(case.room.game.players[1].score_cards.len(), 1);
    assert_eq!(
        case.room.game.players[1].score_cards[0].definition,
        "DQJC115"
    );
    assert_eq!(case.room.game.regions[0].card.id, outer_instance);
    assert_ne!(case.room.game.regions[1].card.id, inner_instance);
    assert!(case.room.game.win_contexts.is_empty());
    assert!(case.room.game.stack.is_empty() && case.room.game.effects.is_empty());
    assert!(case.room.pacing.window.is_none());
    case.game(0, Action::new("restart"));
    assert_eq!(case.room.game.status, "playing");
    assert!(case.room.game.win_contexts.is_empty());
    assert!(case
        .room
        .game
        .players
        .iter()
        .all(|p| p.score_cards.is_empty()));
}

fn privilege_case(name: &str, influence: u32) -> Case {
    let mut g = jc029_tests::game(0);
    region(&mut g, 2, "DQJC112");
    g.regions[2].influence = [influence, 0];
    jc029_tests::board(&mut g, "JC075", 0, 2);
    jc029_tests::board(&mut g, "JC075", 2, 2);
    jc029_tests::board(&mut g, "JC070", 0, 2);
    jc029_tests::fund(&mut g, 0, "JC075", 1);
    g.begin_window(Window::Before(2, 2));
    Case::new(name, g)
}

#[test]
fn win_flow_privilege_offers_renown_once_and_acceptance_can_win() {
    for accept in [false, true] {
        let mut case = privilege_case(&format!("privilege-renown-{accept}"), 2);
        case.game(0, Action::new("privilege"));
        assert_eq!(case.room.game.regions[2].influence, [3, 0]);
        assert!(case.room.game.players[0].assets[0].exhausted);
        let p = case
            .room
            .game
            .pending
            .clone()
            .expect("third contest ends after privilege too");
        assert!(
            matches!(&p.resolution, ChoiceResolution::Declare { declaration, .. }
            if declaration.ability.key == "renown")
        );
        case.game(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(if accept {
                    vec!["accept".into()]
                } else {
                    vec![]
                }),
                ..Action::new("choose")
            },
        );
        if accept {
            case.until(|g| g.window == Some(Window::Win(2, 0)));
            case.finish_win(1);
            assert_eq!(case.room.game.regions[2].influence, [0, 0]);
        } else {
            case.until(|g| g.window == Some(Window::Before(3, 0)));
            assert!(case.room.game.pending.is_none());
            assert_eq!(case.room.game.regions[2].influence, [3, 0]);
        }
        assert_eq!(case.renown_choices.len(), 1, "one distinct renown offer");
    }
}

#[test]
fn win_flow_privilege_reaching_threshold_skips_old_region_renown() {
    let mut case = privilege_case("privilege-already-wins", 3);
    case.game(0, Action::new("privilege"));
    assert_eq!(case.room.game.window, Some(Window::Win(2, 0)));
    assert!(case.room.game.pending.is_none());
    assert!(case.renown_choices.is_empty());
    case.finish_win(1);
    assert_eq!(case.room.game.regions[2].influence, [0, 0]);
    assert_eq!(case.room.game.window, Some(Window::After(2, 2)));
    assert!(case.room.game.pending.is_none());
    assert_eq!(case.room.game.versions.engine, catalog::ENGINE_VERSION);
    assert!(
        case.renown_choices.is_empty(),
        "no renown silently declined during return"
    );
}
