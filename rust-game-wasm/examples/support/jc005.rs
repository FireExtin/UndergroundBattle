//! Explicit local initial layouts; all recorded changes use the real Room ABI.
//! There is only one admitted attachment, BQ022. Non-character-host layouts
//! below are intentionally illegal BQ022-host predicate/settlement fixtures.
//! They never claim a naturally playable region/society attachment or bypass
//! continuous host cleanup in order to make the broad target effect succeed.
use super::*;

fn asset(game: &mut Game, actor: usize, definition: &str, count: usize) {
    for _ in 0..count {
        let c = game.make_card(definition, actor);
        game.players[actor].assets.push(c);
    }
}
fn case(kind: &str) -> Value {
    let non_character = kind.starts_with("synthetic-noncharacter-");
    let bound = kind.starts_with("bound-");
    let mind = matches!(
        kind,
        "mind-character"
            | "exhausted-mind"
            | "synthetic-noncharacter-mind-settlement"
            | "bound-mind-removed-character"
    );
    let mut game = attachment_initial("teams", "888888888888888888880005");
    let source = game.make_card("JC005", 0);
    let source_id = source.id.clone();
    game.players[0].hand.push(source);
    asset(&mut game, 0, "JC002", 2);
    if mind {
        asset(&mut game, 0, "JC004", 1);
        game.players[0].assets.last_mut().unwrap().exhausted = kind != "mind-character";
    }
    let host_definition = if non_character {
        "XQ03"
    } else if kind == "bound-host-moved-region" {
        "JC014"
    } else if matches!(kind, "response-host-return" | "response-host-hide") {
        "JC004"
    } else {
        "LC22"
    };
    let host = game.make_card(host_definition, 2);
    let host_id = host.id.clone();
    game.regions[0].cards.push(host);
    let target = game.make_card("BQ022", 3);
    let target_id = target.id.clone();
    game.attachments.push(Attachment {
        card: target,
        host_id: host_id.clone(),
    });
    let mut hidden = game.make_card("JC007", 1);
    hidden.face_down = true;
    let hidden_id = hidden.id.clone();
    game.regions[3].cards.push(hidden);
    let mut reply_id = None;
    if matches!(kind, "response-host-return" | "response-host-hide") {
        asset(&mut game, 2, "JC003", 2);
        asset(&mut game, 2, "JC063", 2);
        let reply = game.make_card(
            if kind == "response-host-return" {
                "JC006"
            } else {
                "JC063"
            },
            2,
        );
        reply_id = Some(reply.id.clone());
        game.players[2].hand.push(reply);
    }
    let play = Action {
        card_id: Some(source_id.clone()),
        target_id: Some(target_id.clone()),
        ..Action::new("play")
    };
    let mut current_target = target_id.clone();
    if bound {
        game.apply(0, play.clone()).unwrap();
        match kind {
            "bound-host-moved-region" => {
                let c = game.regions[0].cards.remove(0);
                game.regions[1].cards.push(c);
            }
            "bound-host-change-character" => {
                let host = game.make_card("LC22", 1);
                game.attachments[0].host_id = host.id.clone();
                game.regions[4].cards.push(host);
            }
            "bound-host-change-type" => game.regions[0].cards[0].definition = "XQ03".into(),
            "bound-target-reentered" => {
                game.attachments.clear();
                let target = game.make_card("BQ022", 3);
                current_target = target.id.clone();
                game.attachments.push(Attachment {
                    card: target,
                    host_id: host_id.clone(),
                });
            }
            "bound-target-hidden" => game.attachments[0].card.face_down = true,
            "bound-target-shield" => game.attachments[0].card.shield = 1,
            "bound-mind-removed-character" => {
                game.players[0].assets.pop().unwrap();
            }
            _ => unreachable!(),
        }
    }
    let mut room = RoomEnvelope::from_game(game);
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
    RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).unwrap();
    let initial_mind_asset_count = room.players[0]
        .assets
        .iter()
        .filter(|c| catalog::card(&c.definition).magic_icon == rules::MagicIcon::Mind)
        .count();
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        0,
    )];
    let mut denied = vec![];
    if kind == "synthetic-noncharacter-no-mind" {
        denied.push(rejected(&room, 0, play.clone()));
        assert!(room.players[0].assets.iter().all(|c| !c.exhausted));
        assert_eq!(room.attachments[0].card.id, target_id);
    } else {
        if !bound {
            apply_game(&mut room, &mut steps, 0, play.clone());
        }
        assert_eq!(
            room.players[0]
                .assets
                .iter()
                .filter(|c| c.exhausted && c.definition == "JC002")
                .count(),
            2
        );
        if let Some(reply) = reply_id {
            apply_game(&mut room, &mut steps, 0, Action::new("pass"));
            apply_game(&mut room, &mut steps, 1, Action::new("pass"));
            apply_game(
                &mut room,
                &mut steps,
                2,
                Action {
                    card_id: Some(reply),
                    target_id: Some(host_id.clone()),
                    option: (kind == "response-host-hide").then(|| "hide".into()),
                    ..Action::new("play")
                },
            );
            attachment_pass_top(&mut room, &mut steps);
            assert!(room.attachments.is_empty());
            assert!(room.players[3]
                .hand
                .iter()
                .any(|c| c.definition == "BQ022" && c.id != target_id));
        }
        attachment_pass_top(&mut room, &mut steps);
        let cancelled = matches!(
            kind,
            "response-host-return"
                | "response-host-hide"
                | "bound-host-change-type"
                | "bound-target-reentered"
                | "bound-target-hidden"
                | "bound-target-shield"
                | "synthetic-noncharacter-mind-settlement"
        );
        if cancelled {
            assert!(room
                .log
                .iter()
                .any(|line| line.text.contains("效果取消") || line.text.contains("护盾")));
        }
        if matches!(
            kind,
            "bound-target-reentered" | "bound-target-hidden" | "bound-target-shield"
        ) {
            assert!(room.attachments.iter().any(|a| a.card.id == current_target));
            if kind == "bound-target-shield" {
                assert_eq!(room.attachments[0].card.shield, 0);
            }
        } else if !matches!(kind, "response-host-return" | "response-host-hide") {
            assert!(room.attachments.is_empty());
            let dead = room.players[3]
                .graveyard
                .iter()
                .find(|c| c.definition == "BQ022")
                .unwrap();
            assert_ne!(dead.id, target_id);
            assert_eq!((dead.owner, dead.controller), (3, 3));
            assert!(
                !dead.face_down
                    && !dead.exhausted
                    && dead.damage == 0
                    && dead.wounds == 0
                    && dead.shield == 0
            );
            assert!(room.players[3].hand.iter().all(|c| c.definition != "BQ022"));
        }
        assert!(room.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC005"));
        if kind == "synthetic-noncharacter-mind-settlement" {
            assert!(room
                .log
                .iter()
                .any(|line| line.text.contains("结附条件不再满足")));
        }
        let last = steps.last().unwrap();
        let seat = last["args"][0].as_u64().unwrap() as usize;
        let command: RoomCommand = serde_json::from_value(last["args"][1].clone()).unwrap();
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
    for seat in 0..4 {
        let v = room.view(seat, 0);
        let h = v.regions[3]
            .characters
            .iter()
            .find(|c| c.instance_id == hidden_id)
            .unwrap();
        assert_eq!(
            h.card_id.as_deref(),
            if seat == 1 { Some("JC007") } else { None }
        );
    }
    denied.push(rejected(&room, 1, play));
    json!({"name":format!("JC005-{kind}"),"seed":"9007199254740993","steps":steps,"rejectedCommands":denied,
        "initialActorMindAssetCount":initial_mind_asset_count,"syntheticInitialLayout":true,"syntheticBoundFrame":bound,"syntheticNonCharacterHost":non_character,
        "naturallyAdmittedNonCharacterAttachment":false,"nonCharacterHostUsesNormalSettlement":non_character,
        "assetDomainTimingIsGeneralTargetRuleInference":true,"publicNaturalUiAcceptance":false})
}
pub(super) fn cases() -> impl Iterator<Item = Value> {
    [
        "normal-character",
        "mind-character",
        "exhausted-mind",
        "bound-host-moved-region",
        "response-host-return",
        "response-host-hide",
        "bound-host-change-character",
        "bound-host-change-type",
        "bound-target-reentered",
        "bound-target-hidden",
        "bound-target-shield",
        "bound-mind-removed-character",
        "synthetic-noncharacter-no-mind",
        "synthetic-noncharacter-mind-settlement",
    ]
    .into_iter()
    .map(case)
}
