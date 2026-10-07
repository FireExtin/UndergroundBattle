//! Fresh four-seat factory, legal decks, genuine Room commands; no layout edits.
use crate::{
    catalog,
    deck::DeckDraft,
    model::*,
    room::{Decision, RoomCommand, RoomEnvelope, SessionAction},
};
use sha2::{Digest, Sha256};
#[test]
fn fresh_entry_legal_decks_normal_commands_reach_both_printed_searches() {
    fn hash(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }
    let draft = |seat| DeckDraft {
        id: format!("fresh-entry-{seat}"),
        name: "有限进场检索真实合法牌组".into(),
        description: "Fresh factory; no hand/board edits".into(),
        society_id: None,
        cards: [("BQ104", 3), ("XQ48", 3), ("JC125", 44)]
            .into_iter()
            .map(|(id, count)| catalog::DeckEntry {
                card_id: id.into(),
                count,
            })
            .collect(),
        rules_version: catalog::RULES_VERSION.into(),
        card_pool_version: catalog::POOL_VERSION.into(),
        engine_version: catalog::ENGINE_VERSION.into(),
        updated_at: "2026-10-07".into(),
    };
    let mut g = Game::new_with_deck(
        "fresh-entry-50".into(),
        "FRESH50".into(),
        "teams".into(),
        "P0".into(),
        draft(0),
        9,
    )
    .unwrap();
    for seat in 1..4 {
        g.join_with_deck(format!("P{seat}"), draft(seat)).unwrap();
    }
    let mut room = RoomEnvelope::from_game(g);
    let initial = serde_json::to_string(&room).unwrap();
    let mut steps = Vec::new();
    let mut record = |room: &mut RoomEnvelope, seat, action| {
        let now = steps.len() as u64 + 1;
        let command = RoomCommand {
            command_id: format!("fresh-entry-{}", steps.len()),
            expected_version: room.revision,
            action,
        };
        let transition = room.transition(seat, Some(command.clone()), now).unwrap();
        assert!(
            transition.error_code.is_none(),
            "{:?}",
            transition.error_message
        );
        assert_eq!(
            serde_json::to_string(&room.replay_events(&transition.journal).unwrap()).unwrap(),
            transition.state
        );
        *room = RoomEnvelope::from_persisted(&transition.state).unwrap();
        steps.push(serde_json::json!({"seat":seat,"command":command,"serverNowMs":now.to_string(),"transitionSha256":hash(&serde_json::to_vec(&serde_json::to_value(&transition).unwrap()).unwrap()),"stateSha256":hash(transition.state.as_bytes()),
            "viewSha256":(0..4).map(|s|hash(&serde_json::to_vec(&serde_json::to_value(room.view(s,now)).unwrap()).unwrap())).collect::<Vec<_>>() }));
    };
    for seat in 0..4 {
        record(
            &mut room,
            seat,
            SessionAction::Game {
                action: Action::new("ready"),
            },
        );
    }
    record(
        &mut room,
        0,
        SessionAction::Game {
            action: Action::new("start"),
        },
    );
    let mut completed = [false, false];
    for _ in 0..800 {
        if completed.iter().all(|v| *v) {
            break;
        }
        let g = &room.game;
        let (seat, session) = if let Some(p) = &g.pending {
            let selected = match p.choice.kind.as_str() {
                "bq104_employee_search" => {
                    completed[0] = true;
                    p.choice
                        .options
                        .iter()
                        .take(p.choice.min.unwrap())
                        .map(|o| o.id.clone())
                        .collect()
                }
                "xq48_passer_search" => {
                    completed[1] = true;
                    p.choice
                        .options
                        .iter()
                        .take(p.choice.max.unwrap())
                        .map(|o| o.id.clone())
                        .collect()
                }
                "trigger" => vec!["accept".into()],
                _ => p
                    .choice
                    .options
                    .iter()
                    .take(p.choice.min.unwrap_or(0))
                    .map(|o| o.id.clone())
                    .collect(),
            };
            (
                p.seat,
                SessionAction::Game {
                    action: Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(selected),
                        ..Action::new("choose")
                    },
                },
            )
        } else if let Some(w) = &room.pacing.window {
            let seat = *w
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            (
                seat,
                SessionAction::PassResponse {
                    window_id: w.id.clone(),
                },
            )
        } else {
            let seat = g
                .living(g.priority_team)
                .into_iter()
                .find(|s| !g.passed.contains(s))
                .unwrap();
            let legal = g.legal_actions(seat);
            let def = |a: &LegalAction| {
                a.action
                    .card_id
                    .as_ref()
                    .and_then(|id| g.players[seat].hand.iter().find(|c| &c.id == id))
                    .map(|c| c.definition.as_str())
            };
            let pick = legal
                .iter()
                .find(|a| a.action.kind == "asset" && def(a) == Some("JC125"))
                .or_else(|| {
                    legal.iter().find(|a| {
                        a.action.kind == "deploy"
                            && a.action.region == Some(2)
                            && (def(a) == Some("BQ104") && !completed[0]
                                || def(a) == Some("XQ48") && !completed[1])
                    })
                })
                .or_else(|| legal.iter().find(|a| a.action.kind == "pass"))
                .unwrap();
            (
                seat,
                SessionAction::Game {
                    action: pick.action.clone(),
                },
            )
        };
        record(&mut room, seat, session);
    }
    assert!(completed.iter().all(|v| *v));
    assert!(room.game.regions.iter().any(|r| r
        .cards
        .iter()
        .any(|c| c.definition == "BQ104" && c.face_down)));
    assert!(room.game.regions.iter().any(|r| r
        .cards
        .iter()
        .any(|c| c.definition == "JC125" && !c.face_down)));
    let terminal = serde_json::to_string(&room).unwrap();
    drop(record);
    if let Ok(dir) = std::env::var("ENTRY_SEARCH_FRESH_TRACE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/fresh-match.json"),serde_json::to_vec(&serde_json::json!({"scope":"fresh-factory-four-legal-decks-seed9-normal-Room-commands-no-state-injection","initialState":initial,"steps":steps,"terminalState":terminal,"searchesCompleted":completed})).unwrap()).unwrap();
    }
}
