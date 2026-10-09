//! Explicit offline layouts followed by real paid plays, optional declarations,
//! private choices and original receipt restoration. Not natural browser evidence.
use crate::deck_seal_search::{dream_card, space_spell_card};
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, rules::*};
use std::collections::BTreeMap;

fn library(g: &mut Game, actor: usize, defs: &[&str]) -> Vec<String> {
    g.players[actor].deck.clear();
    defs.iter()
        .map(|d| {
            let c = g.make_card(d, actor);
            let id = c.id.clone();
            g.players[actor].deck.push(c);
            id
        })
        .collect()
}
fn held(g: &mut Game, actor: usize, def: &str) -> String {
    let c = g.make_card(def, actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn search(g: &mut Game, actor: usize, dream: bool) -> String {
    if dream {
        fund(g, actor, "XQ41", 2);
        let host = board(g, "LC01", (actor + 2) % 4, 1);
        let source = held(g, actor, "XQ44");
        apply(
            g,
            actor,
            Action {
                card_id: Some(source),
                target_id: Some(host.clone()),
                ..Action::new("play")
            },
        );
        pass_top(g);
        host
    } else {
        let host = board(g, "JZ02", actor, 1);
        g.enter_triggers(actor, "JZ02", &host, false);
        g.drive().unwrap();
        choose(g, vec!["accept".into()]);
        pass_top(g);
        host
    }
}
fn selected(g: &Game, ids: Vec<String>) -> Action {
    Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(ids),
        ..Action::new("choose")
    }
}
fn fixture(name: &str, g: &Game) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("DECK_SEAL_UI_FIXTURE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{name}.json"),serde_json::to_vec(&serde_json::json!({
            "scope":"explicit-offline-native-actual-card-deck-sealing-layout", "catalog":catalog::catalog(),
            "state":serde_json::to_string(&r).unwrap(),"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>() })).unwrap()).unwrap();
    }
}
#[test]
fn deck_seal_printed_registry_and_finite_programs() {
    assert_eq!(catalog::catalog().cards.len(), 117);
    for (id, cost, color, kind, subtypes, unique) in [
        ("XQ44", 2, "紫", "spell", vec!["法术", "心灵"], false),
        ("JZ02", 3, "黄", "character", vec!["猫", "法师"], true),
    ] {
        let c = catalog::card(id);
        assert_eq!(
            (c.cost, c.color.as_str(), c.kind.as_str(), c.unique),
            (cost, color, kind, unique)
        );
        assert_eq!(c.subtypes, subtypes);
        assert_eq!(c.loyalty.len(), 2);
        assert_eq!(c.magic_icon, MagicIcon::Mind);
        let d = definition(id);
        assert_eq!(d.abilities.len(), 1);
        assert!(d.abilities[0].costs.is_empty());
        assert!(d.modifiers.is_empty() && d.attachment.is_none());
    }
    assert_eq!(catalog::card("JZ02").subtitle.as_deref(), Some("相位专家"));
    assert_eq!(
        catalog::card("JZ02").permanent_icons,
        Icons {
            investigation: 1,
            influence: 1,
            ..Default::default()
        }
    );
    assert_eq!(catalog::card("JZ02").defense, Some(1));
    assert!(definition("XQ44").abilities[0].play_only);
    assert_eq!(definition("JZ02").abilities[0].event, Some(Event::Enter));
    assert!(definition("JZ02").abilities[0].targets.is_empty());
    assert!(dream_card(catalog::card("XQ41")) && dream_card(catalog::card("XQ45")));
    for id in ["JC005", "JC132", "XQ03"] {
        assert!(space_spell_card(catalog::card(id)));
    }
    for id in ["JZ02", "XQ44", "XQ41", "JC125"] {
        assert!(!space_spell_card(catalog::card(id)));
    }
}
#[test]
fn deck_seal_one_is_required_with_hits_and_zero_is_explicit_without_hits() {
    for dream in [true, false] {
        for hits in [true, false] {
            for actor in 0..4 {
                let mut g = game(actor);
                let card = if dream { "XQ45" } else { "XQ03" };
                let defs = if hits {
                    vec![card, "JC125", card, "JC125"]
                } else {
                    vec!["JC125", "JC125", "JC125", "JC125"]
                };
                let ids = library(&mut g, actor, &defs);
                let host = search(&mut g, actor, dream);
                let p = g.pending.clone().unwrap();
                assert_eq!(
                    (p.seat, p.choice.min, p.choice.max, p.choice.allow_decline),
                    (
                        actor,
                        Some(usize::from(hits)),
                        Some(usize::from(hits)),
                        Some(false)
                    )
                );
                fixture(
                    &format!(
                        "{}-{}-{actor}",
                        if dream { "xq44" } else { "jz02" },
                        if hits { "choice" } else { "empty" }
                    ),
                    &g,
                );
                for viewer in 0..4 {
                    if viewer != actor {
                        let view = g.view(viewer);
                        assert!(view.pending_choice.is_none());
                        assert!(view.legal_actions.is_empty());
                        assert!(!serde_json::to_string(&view).unwrap().contains(&ids[0]));
                    }
                }
                if hits {
                    let a = selected(&g, vec![]);
                    reject(&mut g, actor, a);
                    let a = selected(&g, vec![ids[0].clone(), ids[2].clone()]);
                    reject(&mut g, actor, a);
                }
                let mut expected = g.clone();
                if hits {
                    expected.players[actor].deck.remove(0);
                }
                expected.shuffle_player(actor);
                choose(&mut g, if hits { vec![ids[0].clone()] } else { vec![] });
                assert_eq!(g.random, expected.random);
                assert_eq!(
                    serde_json::to_value(&g.players[actor].deck).unwrap(),
                    serde_json::to_value(&expected.players[actor].deck).unwrap()
                );
                assert_eq!(g.sealed_cards.len(), usize::from(hits));
                assert!(g.players[actor].hand.is_empty());
                if hits {
                    let sealed = &g.sealed_cards[0];
                    assert_eq!(sealed.host_id, host);
                    assert_ne!(sealed.card.id, ids[0]);
                    for viewer in 0..4 {
                        let s = &g.view(viewer).sealed_cards[0];
                        assert!(s.card.card_id.is_some());
                        assert!(
                            s.card.icons.is_none()
                                && s.card.magic.is_none()
                                && s.card.cost.is_none()
                        );
                    }
                }
                assert!(g.pending.is_none());
                fixture(
                    &format!(
                        "{}-{}-result-{actor}",
                        if dream { "xq44" } else { "jz02" },
                        if hits { "sealed" } else { "empty" }
                    ),
                    &g,
                );
            }
        }
    }
}
#[test]
fn deck_seal_foreign_duplicate_noneligible_wrong_seat_and_stale_choices_are_atomic() {
    for dream in [true, false] {
        let mut g = game(0);
        let card = if dream { "XQ45" } else { "XQ03" };
        let ids = library(&mut g, 0, &[card, "JC125", card]);
        let foreign = library(&mut g, 2, &[card]);
        search(&mut g, 0, dream);
        for ids in [
            vec![ids[0].clone(), ids[0].clone()],
            vec![ids[1].clone()],
            vec![foreign[0].clone()],
            vec!["stale".into()],
        ] {
            let a = selected(&g, ids);
            reject(&mut g, 0, a);
        }
        let a = selected(&g, vec![ids[0].clone()]);
        reject(&mut g, 1, a.clone());
        let old = a;
        choose(&mut g, vec![ids[0].clone()]);
        reject(&mut g, 0, old);
    }
}

#[test]
fn deck_seal_nonactors_cannot_distinguish_hits_from_equal_length_misses() {
    for dream in [true, false] {
        for actor in 0..4 {
            let mut hit = game(actor);
            library(
                &mut hit,
                actor,
                &[if dream { "XQ45" } else { "JC005" }, "JC125", "JC125"],
            );
            let mut miss = hit.clone();
            for c in &mut miss.players[actor].deck {
                c.definition = "JC125".into();
            }
            search(&mut hit, actor, dream);
            search(&mut miss, actor, dream);
            assert_eq!(hit.pending.as_ref().unwrap().choice.options.len(), 1);
            assert!(miss.pending.as_ref().unwrap().choice.options.is_empty());
            for viewer in 0..4 {
                if viewer != actor {
                    assert_eq!(
                        serde_json::to_value(hit.view(viewer)).unwrap(),
                        serde_json::to_value(miss.view(viewer)).unwrap()
                    );
                }
            }
            checkpoint(&hit);
            checkpoint(&miss);
        }
    }
}

#[cfg(feature = "native")]
#[test]
fn deck_seal_fresh_legal_four_seat_factory_reaches_both_actual_cards_without_layout_edits() {
    use crate::deck::DeckDraft;
    use crate::room::{Decision, RoomCommand, RoomEnvelope, SessionAction};
    use sha2::{Digest, Sha256};
    let hash = |s: &[u8]| format!("{:x}", Sha256::digest(s));
    let draft = |seat| DeckDraft {
        id: format!("fresh-deck-seal-{seat}"),
        name: "检索封印合法50张".into(),
        description: "Fresh factory; no layout edits".into(),
        society_id: None,
        cards: [
            ("JZ02", 1),
            ("XQ44", 3),
            ("XQ41", 3),
            ("XQ45", 3),
            ("JC005", 3),
            ("XQ03", 3),
            ("JC132", 3),
            ("JC001", 3),
            ("LC22", 3),
            ("XQ40", 3),
            ("XQ49", 3),
            ("JC125", 19),
        ]
        .into_iter()
        .map(|(id, count)| catalog::DeckEntry {
            card_id: id.into(),
            count,
        })
        .collect(),
        rules_version: catalog::RULES_VERSION.into(),
        card_pool_version: catalog::POOL_VERSION.into(),
        engine_version: catalog::ENGINE_VERSION.into(),
        updated_at: "2026-10-09".into(),
    };
    let mut g = Game::new_with_deck(
        "fresh-deck-seal-51".into(),
        "FRESH51".into(),
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
    let mut steps = vec![];
    let mut record = |room: &mut RoomEnvelope, seat, action| {
        let now = steps.len() as u64 + 1;
        let search_kind = room
            .game
            .pending
            .as_ref()
            .filter(|p| {
                matches!(
                    p.choice.kind.as_str(),
                    "xq44_dream_seal_search" | "jz02_space_seal_search"
                )
            })
            .map(|p| (p.choice.kind.clone(), p.choice.min.unwrap_or(0)));
        let seals_before = room.game.sealed_cards.len();
        let command = RoomCommand {
            command_id: format!("fresh-deck-seal-{}", steps.len()),
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
        if let Some((_, count)) = &search_kind {
            assert_eq!(room.game.sealed_cards.len(), seals_before + count);
        }
        steps.push(serde_json::json!({"seat":seat,"command":command,"serverNowMs":now.to_string(),"completedSearch":search_kind,"transitionSha256":hash(&serde_json::to_vec(&serde_json::to_value(&transition).unwrap()).unwrap()),
            "stateSha256":hash(transition.state.as_bytes()),"viewSha256":(0..4).map(|s|hash(&serde_json::to_vec(&serde_json::to_value(room.view(s,now)).unwrap()).unwrap())).collect::<Vec<_>>() }));
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
    for _ in 0..1400 {
        if completed.iter().all(|v| *v) {
            break;
        }
        let g = &room.game;
        let (seat, session) = if let Some(p) = &g.pending {
            let ids = match p.choice.kind.as_str() {
                "xq44_dream_seal_search" => {
                    completed[0] = true;
                    p.choice
                        .options
                        .iter()
                        .take(p.choice.min.unwrap())
                        .map(|o| o.id.clone())
                        .collect()
                }
                "jz02_space_seal_search" => {
                    completed[1] = true;
                    p.choice
                        .options
                        .iter()
                        .take(p.choice.min.unwrap())
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
                        selected: Some(ids),
                        top: if matches!(p.choice.kind.as_str(), "order" | "investigation") {
                            Some(p.choice.options.iter().map(|o| o.id.clone()).collect())
                        } else {
                            None
                        },
                        bottom: if matches!(p.choice.kind.as_str(), "order" | "investigation") {
                            Some(vec![])
                        } else {
                            None
                        },
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
            let colors = |color: &str| {
                g.players[seat]
                    .assets
                    .iter()
                    .filter(|c| catalog::card(&c.definition).color == color)
                    .count()
            };
            let score = |a: &LegalAction| -> u32 {
                let d = def(a);
                if a.action.kind == "deploy"
                    && a.action.region == Some(2)
                    && d == Some("JZ02")
                    && !completed[1]
                {
                    return 500;
                }
                if a.action.kind == "play" && d == Some("XQ44") && !completed[0] {
                    return 450;
                }
                if a.action.kind == "asset" {
                    let c = catalog::card(d.unwrap());
                    if colors(&c.color) < 2 && (c.color == "黄" || c.color == "紫") {
                        return 400;
                    }
                    if g.players[seat].assets.len() < 6 {
                        return 100;
                    }
                }
                if a.action.kind == "deploy"
                    && a.action.region == Some(2)
                    && matches!(d, Some("JC001" | "LC22"))
                    && g.regions[2].cards.is_empty()
                {
                    return 300;
                }
                if a.action.kind == "pass" {
                    return 1;
                }
                0
            };
            let a = legal
                .iter()
                .max_by_key(|a| score(a))
                .filter(|a| score(a) > 0)
                .unwrap();
            (
                seat,
                SessionAction::Game {
                    action: a.action.clone(),
                },
            )
        };
        record(&mut room, seat, session);
    }
    let terminal = serde_json::to_string(&room).unwrap();
    drop(record);
    if let Ok(dir) = std::env::var("DECK_SEAL_FRESH_TRACE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/fresh-match.json"),serde_json::to_vec(&serde_json::json!({"scope":"fresh factory seed9; four legal decks and actual Room commands; no hand/board/deck state edits","initialState":initial,"steps":steps,"terminalState":terminal,"completed":completed})).unwrap()).unwrap();
    }
    assert!(
        completed.iter().all(|v| *v),
        "Fresh legal match did not reach both searches: {completed:?}"
    );
    // The earlier searched payload can normally return to hand when its region
    // is won before the second source is drawn. Do not require it to stay sealed.
    assert!(!room.game.sealed_cards.is_empty());
}
#[test]
fn deck_seal_jz02_paid_deploy_and_reveal_offer_optional_entry_but_decline_never_shuffles() {
    for reveal in [false, true] {
        for take in [false, true] {
            let mut g = game(0);
            library(&mut g, 0, &["JC005", "XQ03", "JC125"]);
            fund(&mut g, 0, "JC001", 3);
            let source = if reveal {
                let id = board(&mut g, "JZ02", 0, 1);
                g.board_mut(&id).unwrap().face_down = true;
                id
            } else {
                held(&mut g, 0, "JZ02")
            };
            apply(
                &mut g,
                0,
                Action {
                    card_id: Some(source),
                    region: Some(1),
                    ..Action::new(if reveal { "reveal" } else { "deploy" })
                },
            );
            assert_eq!(g.resources(0), 0);
            pass_top(&mut g);
            assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
            let rng = g.random;
            let before = serde_json::to_value(&g.players[0].deck).unwrap();
            if take {
                choose(&mut g, vec!["accept".into()]);
                pass_top(&mut g);
                let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
                choose(&mut g, vec![id]);
                assert_eq!(g.sealed_cards.len(), 1);
            } else {
                choose(&mut g, vec![]);
                assert_eq!(g.random, rng);
                assert_eq!(serde_json::to_value(&g.players[0].deck).unwrap(), before);
                assert!(g.sealed_cards.is_empty());
            }
            checkpoint(&g);
        }
    }
}
#[test]
fn deck_seal_jz02_original_self_gone_or_hidden_skips_search_shuffles_and_never_invents_a() {
    for before_accept in [false, true] {
        for variant in 0..5 {
            let mut g = game(0);
            library(&mut g, 0, &["JC005", "XQ03", "JC125", "JC132"]);
            let source = board(&mut g, "JZ02", 0, 1);
            g.enter_triggers(0, "JZ02", &source, false);
            g.drive().unwrap();
            if !before_accept {
                choose(&mut g, vec!["accept".into()]);
            }
            match variant {
                0 => g.remove_dead(&source, RemovalCause::Destroy),
                1 => g.remove_dead(&source, RemovalCause::Sacrifice),
                2 => g.return_hand(&source),
                3 => g.to_bottom(&source),
                _ => g.board_mut(&source).unwrap().face_down = true,
            }
            let replacement = board(&mut g, "JZ02", 2, 1);
            assert_ne!(source, replacement);
            if before_accept {
                choose(&mut g, vec!["accept".into()]);
            }
            let mut expected = g.clone();
            expected.shuffle_player(0);
            pass_top(&mut g);
            assert!(g.pending.is_none() && g.sealed_cards.is_empty());
            assert_eq!(g.random, expected.random);
            assert_eq!(
                serde_json::to_value(&g.players[0].deck).unwrap(),
                serde_json::to_value(&expected.players[0].deck).unwrap()
            );
            assert!(!g
                .players
                .iter()
                .flat_map(|p| &p.graveyard)
                .any(|c| ["JC005", "XQ03", "JC132"].contains(&c.definition.as_str())));
            fixture(&format!("jz02-skipped-{before_accept}-{variant}"), &g);
        }
    }
}
#[test]
fn deck_seal_xq44_invalid_only_target_cancels_everything_and_keeps_cost_paid() {
    for variant in 0..5 {
        let mut g = game(0);
        library(&mut g, 0, &["XQ45", "XQ41", "JC125"]);
        fund(&mut g, 0, "XQ41", 2);
        let host = board(&mut g, "LC01", 2, 1);
        let source = held(&mut g, 0, "XQ44");
        apply(
            &mut g,
            0,
            Action {
                card_id: Some(source),
                target_id: Some(host.clone()),
                ..Action::new("play")
            },
        );
        let rng = g.random;
        let before = serde_json::to_value(&g.players[0].deck).unwrap();
        match variant {
            0 => g.remove_dead(&host, RemovalCause::Destroy),
            1 => g.remove_dead(&host, RemovalCause::Sacrifice),
            2 => g.return_hand(&host),
            3 => g.to_bottom(&host),
            _ => g.board_mut(&host).unwrap().face_down = true,
        }
        let replacement = board(&mut g, "LC01", 2, 1);
        assert_ne!(host, replacement);
        pass_top(&mut g);
        assert!(g.pending.is_none() && g.sealed_cards.is_empty());
        assert_eq!(g.resources(0), 0);
        assert_eq!(g.random, rng);
        assert_eq!(serde_json::to_value(&g.players[0].deck).unwrap(), before);
        assert!(g.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "XQ44"));
        checkpoint(&g);
    }
}
#[test]
fn deck_seal_original_instance_tracks_movement_and_freezes_actor_under_control_change() {
    for dream in [true, false] {
        for actor in 0..4 {
            let mut g = game(actor);
            let ids = library(
                &mut g,
                actor,
                &[if dream { "XQ45" } else { "XQ03" }, "JC125", "JC125"],
            );
            let owner = (actor + 2) % 4;
            g.players[actor].deck[0].owner = owner;
            g.players[actor].deck[0].controller = owner;
            let host = search(&mut g, actor, dream);
            let (_, mut c) = g.remove_board(&host).unwrap();
            c.controller = (actor + 1) % 4;
            g.regions[3].cards.push(c);
            choose(&mut g, vec![ids[0].clone()]);
            assert_eq!(g.sealed_cards[0].host_id, host);
            assert_eq!(g.sealed_cards[0].card.owner, owner);
            g.return_hand(&host);
            assert!(g.sealed_cards.is_empty());
            assert!(g.players[owner]
                .hand
                .iter()
                .any(|c| c.definition == if dream { "XQ45" } else { "XQ03" }));
            checkpoint(&g);
        }
    }
}
#[test]
fn deck_seal_xq41_event_freezes_actor_and_owner_and_survives_actual_responsive_destruction() {
    for actor in 0..4 {
        let owner = (actor + 2) % 4;
        let mut g = game(actor);
        let ids = library(&mut g, actor, &["XQ41", "JC125", "JC125"]);
        g.players[actor].deck[0].owner = owner;
        g.players[actor].deck[0].controller = owner;
        let host = search(&mut g, actor, true);
        choose(&mut g, vec![ids[0].clone()]);
        let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
        else {
            panic!()
        };
        assert_eq!(
            (
                declaration.actor,
                declaration.source.card.owner,
                declaration.source.card.controller
            ),
            (actor, owner, actor)
        );
        assert_eq!(declaration.source.card.id, ids[0]);
        fixture(&format!("xq41-event-{actor}"), &g);
        choose(&mut g, vec!["accept".into()]);
        fund(&mut g, actor, "XQ41", 3);
        let kill = held(&mut g, actor, "XQ45");
        apply(
            &mut g,
            actor,
            Action {
                card_id: Some(kill),
                target_id: Some(host.clone()),
                ..Action::new("play")
            },
        );
        pass_top(&mut g);
        assert!(g.sealed_cards.is_empty() && g.board(&host).is_none());
        assert!(g.players[owner].hand.iter().any(|c| c.definition == "XQ41"));
        let n = g.players[actor].deck.len();
        pass_top(&mut g);
        assert_eq!(g.players[actor].deck.len(), n - 1);
        assert!(g.pending.is_none());
        checkpoint(&g);
    }
}
#[test]
fn deck_seal_same_name_instances_both_teams_and_cascading_events_stay_separate() {
    let mut g = game(0);
    for actor in [0, 2] {
        g.begin_window(Window::Action(g.team(actor)));
        library(&mut g, actor, &["XQ45", "JC125", "JC125"]);
        search(&mut g, actor, true);
        let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
        choose(&mut g, vec![id]);
    }
    assert_eq!(g.sealed_cards.len(), 2);
    assert_ne!(g.sealed_cards[0].card.id, g.sealed_cards[1].card.id);
    assert_ne!(g.sealed_cards[0].host_id, g.sealed_cards[1].host_id);
    let hosts = g
        .sealed_cards
        .iter()
        .map(|s| s.host_id.clone())
        .collect::<Vec<_>>();
    for h in &hosts {
        g.board_mut(h).unwrap().damage = 100;
    }
    g.settle_deaths();
    assert!(g.sealed_cards.is_empty());
    assert!(g.players[0].hand.iter().any(|c| c.definition == "XQ45"));
    assert!(g.players[2].hand.iter().any(|c| c.definition == "XQ45"));
    checkpoint(&g);
}
#[test]
fn deck_seal_pending_mutations_are_rejected_at_read_and_before_any_mutation() {
    for dream in [true, false] {
        let mut g = game(0);
        library(
            &mut g,
            0,
            &[if dream { "XQ45" } else { "XQ03" }, "JC125", "JC125"],
        );
        search(&mut g, 0, dream);
        for variant in 0..14 {
            let mut bad = g.clone();
            let p = bad.pending.as_mut().unwrap();
            match variant {
                0 => p.seat = 1,
                1 => p.choice.player_id = "p1".into(),
                2 => p.choice.min = Some(0),
                3 => p.choice.max = Some(0),
                4 => p.choice.allow_decline = Some(true),
                5 => p.choice.options[0].label = "forged".into(),
                6 => p.choice.options.clear(),
                7 => p.choice.kind = "handSeal".into(),
                _ => {
                    let ChoiceResolution::Frame { frame, choice } = &mut p.resolution else {
                        panic!()
                    };
                    match variant {
                        8 => frame.actor = 1,
                        9 => frame.source.card.definition = "XQ40".into(),
                        10 => frame.ability_key = "fake".into(),
                        11 => frame.cursor = 0,
                        12 => frame.steps[0].op = Op::SealOneActorHandCardOnTarget,
                        _ => {
                            *choice = FrameChoice::HandSeal {
                                seat: 0,
                                host_id: "fake".into(),
                            }
                        }
                    }
                }
            }
            assert!(
                Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),
                "{dream}/{variant}"
            );
            let a = selected(&bad, vec![]);
            let before = serde_json::to_string(&bad).unwrap();
            assert!(bad.apply(0, a).is_err());
            assert_eq!(serde_json::to_string(&bad).unwrap(), before);
            let raw = crate::room::RoomEnvelope::from_game(bad.clone());
            assert!(crate::room::RoomEnvelope::from_persisted(
                &serde_json::to_string(&raw).unwrap()
            ).is_err(), "Room restore must reject {dream}/{variant}");
            if let Ok(dir) = std::env::var("DECK_SEAL_INVALID_DIR") {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(format!("{dir}/choice-{dream}-{variant}.json"),serde_json::to_vec(&serde_json::json!({"scope":"malformed explicit offline state must reject","state":serde_json::to_string(&raw).unwrap()})).unwrap()).unwrap();
            }
        }
        let mut bad = g.clone();
        let duplicate = bad.players[0].deck[0].clone();
        bad.players[0].deck.push(duplicate);
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    }
}
#[test]
fn deck_seal_full_definition_and_ops_cannot_be_transplanted_nested_or_rewritten() {
    for id in ["XQ44", "JZ02"] {
        let d = definition(id).clone();
        validate_definitions(&BTreeMap::from([(id.into(), d.clone())])).unwrap();
        let mut alias = BTreeMap::from([("JC125".into(), d.clone())]);
        assert!(validate_definitions(&alias).is_err());
        for variant in 0..7 {
            let mut bad = d.clone();
            match variant {
                0 => bad.abilities[0].key = "fake".into(),
                1 => bad.abilities[0].costs.push(Cost::Assets(1)),
                2 => bad.abilities[0].timing = Timing::Standard,
                3 => {
                    bad.abilities[0].ops =
                        vec![Op::ForEachLivingPlayer(bad.abilities[0].ops.clone())]
                }
                4 => bad.modifiers.push(StaticModifier::PeekOwnDeckTop),
                5 => bad.abilities.push(d.abilities[0].clone()),
                _ => bad.abilities[0].targets = vec![target_for_test()],
            }
            // XQ44 is already Standard; use Fast so this case always mutates.
            if variant == 2 && id == "XQ44" {
                bad.abilities[0].timing = Timing::Fast;
            }
            assert!(
                validate_definitions(&BTreeMap::from([(id.into(), bad)])).is_err(),
                "{id}/{variant}"
            );
        }
        alias.get_mut("JC125").unwrap().abilities[0].ops = vec![Op::IfTargetExhausted {
            slot: 0,
            exhausted: Box::new(d.abilities[0].ops[0].clone()),
            ready: Box::new(d.abilities[0].ops[0].clone()),
        }];
        assert!(validate_definitions(&alias).is_err());
    }
}
fn target_for_test() -> TargetSlotSpec {
    let mut t = definition("XQ44").abilities[0].targets[0].clone();
    t.relation = Relation::ControlledByActor;
    t
}

#[test]
fn deck_seal_real_response_hiding_or_lethal_cancels_xq44_but_jz02_only_shuffles() {
    for dream in [true, false] {
        for lethal in [true, false] {
            let mut g = game(0);
            library(
                &mut g,
                0,
                &[if dream { "XQ45" } else { "JC005" }, "JC125", "JC125"],
            );
            let host = board(&mut g, "JZ02", 0, 1);
            if dream {
                fund(&mut g, 0, "XQ41", 2);
                let source = held(&mut g, 0, "XQ44");
                apply(
                    &mut g,
                    0,
                    Action {
                        card_id: Some(source),
                        target_id: Some(host.clone()),
                        ..Action::new("play")
                    },
                );
            } else {
                g.enter_triggers(0, "JZ02", &host, false);
                g.drive().unwrap();
                choose(&mut g, vec!["accept".into()]);
            }
            for seat in [0, 1] {
                apply(&mut g, seat, Action::new("pass"));
            }
            fund(&mut g, 2, if lethal { "JC104" } else { "JC063" }, 2);
            let spell = held(&mut g, 2, if lethal { "JC102" } else { "JC063" });
            apply(
                &mut g,
                2,
                Action {
                    card_id: Some(spell),
                    target_id: Some(host.clone()),
                    option: if lethal { None } else { Some("hide".into()) },
                    ..Action::new("play")
                },
            );
            pass_top(&mut g);
            assert!(g.board(&host).is_none());
            if !lethal {
                assert!(g.regions[1]
                    .cards
                    .iter()
                    .any(|c| c.definition == "JZ02" && c.face_down && c.id != host));
            }
            let mut expected = g.clone();
            if !dream {
                expected.shuffle_player(0);
            }
            pass_top(&mut g);
            assert!(g.pending.is_none() && g.sealed_cards.is_empty());
            assert_eq!(g.random, expected.random);
            assert_eq!(
                serde_json::to_value(&g.players[0].deck).unwrap(),
                serde_json::to_value(&expected.players[0].deck).unwrap()
            );
            fixture(
                &format!(
                    "{}-real-response-{}",
                    if dream { "xq44" } else { "jz02" },
                    if lethal { "lethal" } else { "hide" }
                ),
                &g,
            );
        }
    }
}
#[test]
fn deck_seal_empty_or_singleton_libraries_finish_once_and_do_not_draw() {
    for dream in [true, false] {
        for defs in [
            vec![],
            vec!["JC125"],
            vec![if dream { "XQ45" } else { "JC005" }],
        ] {
            let mut g = game(0);
            library(&mut g, 0, &defs);
            search(&mut g, 0, dream);
            let ids = g
                .pending
                .as_ref()
                .unwrap()
                .choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<Vec<_>>();
            let mut expected = g.clone();
            if !ids.is_empty() {
                expected.players[0].deck.remove(0);
            }
            expected.shuffle_player(0);
            choose(&mut g, ids);
            assert_eq!(g.random, expected.random);
            assert!(g.pending.is_none());
            assert!(g.players[0].hand.is_empty());
            assert_eq!(g.players[0].deck.len(), expected.players[0].deck.len());
            checkpoint(&g);
        }
    }
}
#[test]
fn deck_seal_defensive_already_selected_failure_is_owner_grave_not_pre_search_invention() {
    // Unreachable by an ordinary interleaved action: exercise only the primitive
    // P4 failure branch. Persisted pending host tampering is rejected separately.
    for dream in [true, false] {
        let mut g = game(0);
        let ids = library(
            &mut g,
            0,
            &[if dream { "XQ41" } else { "JC005" }, "JC125", "JC125"],
        );
        g.players[0].deck[0].owner = 3;
        g.players[0].deck[0].controller = 3;
        let host = search(&mut g, 0, dream);
        let ChoiceResolution::Frame { frame, .. } = g.pending.take().unwrap().resolution else {
            panic!()
        };
        g.return_hand(&host);
        let replacement = board(&mut g, if dream { "LC01" } else { "JZ02" }, 2, 1);
        g.deck_seal_search_complete(&frame, &ids[..1], dream)
            .unwrap();
        assert!(g.sealed_cards.is_empty());
        assert!(g.players[3]
            .graveyard
            .iter()
            .any(|c| c.definition == if dream { "XQ41" } else { "JC005" }));
        assert!(g.pending.is_none());
        assert_ne!(host, replacement);
        checkpoint(&g);
    }
}

#[cfg(feature = "native")]
#[tokio::test]
async fn deck_seal_actual_search_pause_sqlite_reopen_and_original_receipts_are_idempotent() {
    use crate::room::{RoomCommand, RoomEnvelope, SessionAction};
    use crate::service::{CreateRoom, JoinRoom, Store};
    for (dream, hits, event_take) in [
        (true, true, Some(true)),
        (true, true, Some(false)),
        (true, false, None),
        (false, true, None),
        (false, false, None),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("actual-deck-seal-offline-fixture.sqlite");
        let store = Store::open(&path).unwrap();
        let mut sessions = vec![store
            .create(CreateRoom {
                name: "P0".into(),
                mode: "teams".into(),
                deck_id: "watchers".into(),
                deck_draft: None,
            })
            .await
            .unwrap()];
        for seat in 1..4 {
            sessions.push(
                store
                    .join(JoinRoom {
                        invite_code: sessions[0].invite_code.clone(),
                        name: format!("P{seat}"),
                        deck_id: "watchers".into(),
                        deck_draft: None,
                    })
                    .await
                    .unwrap(),
            );
        }
        drop(store);
        let mut g = game(0);
        let defs = if hits {
            vec![if dream { "XQ41" } else { "JC005" }, "JC125", "JC125"]
        } else {
            vec!["JC125", "JC125", "JC125"]
        };
        let ids = library(&mut g, 0, &defs);
        search(&mut g, 0, dream);
        g.room_id = sessions[0].room_id.clone();
        g.invite_code = sessions[0].invite_code.clone();
        let mut room = envelope(&g);
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute(
            "UPDATE rooms SET state=?1,revision=?2 WHERE id=?3",
            rusqlite::params![
                serde_json::to_string(&room).unwrap(),
                room.revision,
                g.room_id
            ],
        )
        .unwrap();
        drop(db);
        let initial = serde_json::to_string(&room).unwrap();
        let rng = g.random;
        let deck = serde_json::to_value(&g.players[0].deck).unwrap();
        let pick = selected(&g, if hits { vec![ids[0].clone()] } else { vec![] });
        let mut actions = vec![
            SessionAction::PauseRoom,
            SessionAction::ResumeRoom,
            SessionAction::Game { action: pick },
        ];
        let mut trace = vec![];
        let mut last = None;
        let mut responses = vec![];
        let mut index = 0;
        while index < actions.len() {
            let now = if index == 0 {
                100
            } else {
                180_000 + index as u64
            };
            let command = RoomCommand {
                command_id: format!("actual-deck-seal-{dream}-{hits}-{event_take:?}-{index}"),
                expected_version: room.revision,
                action: actions[index].clone(),
            };
            let transition = room.transition(0, Some(command.clone()), now).unwrap();
            assert!(
                transition.error_code.is_none(),
                "{:?}",
                transition.error_message
            );
            room = RoomEnvelope::from_persisted(&transition.state).unwrap();
            let store = Store::open(&path).unwrap();
            let response = store
                .command_at_now(&g.room_id, &sessions[0].token, command.clone(), now)
                .await
                .unwrap();
            drop(store);
            assert_eq!(
                serde_json::to_value(&response).unwrap(),
                serde_json::to_value(room.view(0, now)).unwrap()
            );
            responses.push(serde_json::json!({"commandId":command.command_id,"response":response}));
            trace.push(serde_json::json!({"seat":0,"command":command,"serverNowMs":now.to_string(),"transition":transition,"views":(0..4).map(|s|room.view(s,now)).collect::<Vec<_>>() }));
            if index < 2 {
                assert_eq!(room.game.random, rng);
                assert_eq!(
                    serde_json::to_value(&room.game.players[0].deck).unwrap(),
                    deck
                );
                assert!(room.game.sealed_cards.is_empty());
            }
            if index == 2 {
                assert_eq!(room.game.sealed_cards.len(), usize::from(hits));
                if let Some(take) = event_take {
                    let p = room.game.pending.as_ref().unwrap();
                    assert_eq!(p.choice.kind, "trigger");
                    actions.push(SessionAction::Game {
                        action: Action {
                            choice_id: Some(p.choice.id.clone()),
                            selected: Some(if take { vec!["accept".into()] } else { vec![] }),
                            ..Action::new("choose")
                        },
                    });
                }
            }
            for viewer in 1..4 {
                assert!(room.view(viewer, now).pending_choice.is_none());
            }
            last = Some((command, response));
            index += 1;
        }
        let final_state = serde_json::to_string(&room).unwrap();
        let (command, response) = last.unwrap();
        let store = Store::open(&path).unwrap();
        let duplicate = store
            .command_at_now(&g.room_id, &sessions[0].token, command.clone(), 600_000)
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&duplicate).unwrap(),
            serde_json::to_value(&response).unwrap()
        );
        let mut conflicting = command;
        conflicting.expected_version += 1;
        assert_eq!(
            store
                .command_at_now(&g.room_id, &sessions[0].token, conflicting, 600_000)
                .await
                .unwrap_err()
                .error,
            "command_id_conflict"
        );
        drop(store);
        let db = rusqlite::Connection::open(&path).unwrap();
        let stored: String = db
            .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(stored, final_state);
        let receipts: u64 = db
            .query_row(
                "SELECT COUNT(*) FROM commands WHERE room_id=?1",
                [&g.room_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(receipts, index as u64);
        if let Ok(dir) = std::env::var("DECK_SEAL_STORE_TRACE_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(format!("{dir}/search-{dream}-{hits}-{event_take:?}.json"),serde_json::to_vec(&serde_json::json!({"scope":"explicit disposable offline SQLite layout; actual new-card search and event Room commands","initialState":initial,"steps":trace,"receipts":responses,"duplicateOriginalResponseEqual":true,"conflictRejected":true,"storedStateByteEqual":true})).unwrap()).unwrap();
        }
    }
}
