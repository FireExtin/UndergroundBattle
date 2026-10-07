//! Explicit offline layouts, actual paid reveals/responses and private search.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::jz50_search::jz50_search_match;
use crate::{
    catalog,
    model::*,
    room::{RoomCommand, RoomEnvelope, SessionAction},
    rules::*,
};
fn held(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn library(g: &mut Game, actor: usize, defs: &[&str]) -> Vec<String> {
    g.players[actor].deck.clear();
    let mut ids = Vec::new();
    for def in defs {
        let c = g.make_card(def, actor);
        ids.push(c.id.clone());
        g.players[actor].deck.push(c);
    }
    ids
}
fn ids(g: &Game, actor: usize) -> Vec<String> {
    g.players[actor].deck.iter().map(|c| c.id.clone()).collect()
}
fn trigger(g: &mut Game, def: &str, owner: usize, actor: usize) -> String {
    // Explicit already-entered/revealed source primitive. All later declarations
    // and choices use the real queue; paid entry/reveal also have separate tests.
    let id = board(g, def, owner, 2);
    g.board_mut(&id).unwrap().controller = actor;
    g.enter_triggers(actor, def, &id, def == "JZ50");
    g.drive().unwrap();
    checkpoint(g);
    id
}
fn accept(g: &mut Game) -> ResolutionFrame {
    choose(g, vec!["accept".into()]);
    pass_top(g);
    let ChoiceResolution::Frame { frame, .. } = &g.pending.as_ref().unwrap().resolution else {
        panic!()
    };
    (**frame).clone()
}
fn selection(g: &Game, selected: Vec<String>) -> Action {
    Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(selected),
        ..Action::new("choose")
    }
}
fn fixture(kind: &str, g: &Game) {
    if let Ok(dir) = std::env::var("SEARCH_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{kind}.json"),serde_json::to_vec(&serde_json::json!({
            "kind":kind,"catalog":catalog::catalog(),"state":serde_json::to_string(&r).unwrap(),"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
    checkpoint(g);
}

#[test]
fn jz50_original_remains_unchanged_after_entry_search_admission() {
    let j = catalog::card("JZ50");
    assert_eq!(
        (&*j.name, j.cost, &*j.color, j.defense),
        ("墓穴食尸鬼", 2, "黑", Some(1))
    );
    assert_eq!(j.loyalty, ["黑色"]);
    assert_eq!(j.magic_icon, MagicIcon::Death);
    assert_eq!(j.subtypes, ["不死生物", "食尸鬼"]);
    assert!(j.keywords.is_empty() && !j.unique);
    assert_eq!(
        j.permanent_icons,
        Icons {
            combat: 1,
            ..Icons::default()
        }
    );
    assert_eq!(j.temporary_icons, Icons::default());
    assert_eq!(catalog::card("JC125").name, "无知路人");
    for id in ["BQ104", "XQ48"] {
        assert!(catalog::catalog().cards.iter().any(|c| c.id == id));
    }
    let d = definition("JZ50");
    let a = &d.abilities[0];
    assert_eq!(a.event, Some(Event::Reveal));
    assert_eq!(a.key, "jz50-reveal-death-search");
    assert!(
        a.costs.is_empty()
            && a.targets.is_empty()
            && a.modes.is_empty()
            && !a.requires_ready_source
    );
    assert!(matches!(a.ops.as_slice(), [Op::JZ50SearchDeathToGraveyard]));
}

#[test]
fn search_filters_use_exact_printed_domain_kind_cost_subtype_and_name() {
    for id in ["JC032", "JC085", "BQ083", "JZ24", "JZ49"] {
        assert!(jz50_search_match(catalog::card(id)));
    }
    for id in ["JC092", "JC093", "JC018", "JC125", "LC01", "JC089"] {
        assert!(!jz50_search_match(catalog::card(id)));
    }
    let mut d = catalog::card("JC032").clone();
    d.magic_icon = MagicIcon::Other("死亡".into());
    assert!(!jz50_search_match(&d));
    d.magic_icon = MagicIcon::None;
    d.permanent_icons.combat = 50;
    d.temporary_icons.combat = 50;
    assert!(!jz50_search_match(&d));
}

#[test]
fn jz50_paid_faceup_deploy_does_not_reveal_trigger_and_conceal_then_paid_reveal_does() {
    let mut g = game(0);
    fund(&mut g, 0, "JC084", 2);
    let id = held(&mut g, "JZ50", 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert!(g.stack.is_empty());
    checkpoint(&g);
    let mut g = game(0);
    fund(&mut g, 0, "JC084", 5);
    let id = held(&mut g, "JZ50", 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
    assert!(g.pending.is_none());
    let hidden = g.regions[2].cards[0].id.clone();
    assert!(g.board(&hidden).unwrap().1.face_down);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(hidden.clone()),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut g);
    let source = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JZ50")
        .unwrap();
    assert_ne!(source.id, hidden);
    let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
    else {
        panic!()
    };
    assert_eq!(declaration.ability.event, Some(Event::Reveal));
    choose(&mut g, vec![]);
    let id = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JZ50")
        .unwrap()
        .id
        .clone();
    // Explicit hide input, then a second genuine paid Reveal with a new identity.
    g.board_mut(&id).unwrap().face_down = true;
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id.clone()),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut g);
    assert!(g.board(&id).is_none());
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
    checkpoint(&g);
}

#[test]
fn jz50_refused_trigger_does_not_search_or_shuffle() {
    let mut g = game(0);
    library(&mut g, 0, &["JC032", "JC125", "JC029", "JC085"]);
    trigger(&mut g, "JZ50", 0, 0);
    let before = ids(&g, 0);
    let rng = g.random;
    choose(&mut g, vec![]);
    assert_eq!(ids(&g, 0), before);
    assert_eq!(g.random, rng);
    assert!(g.pending.is_none() && g.stack.is_empty() && g.players[0].graveyard.is_empty());
    fixture("jz50-refused", &g);
}

#[test]
fn jz50_accepted_zero_and_one_shuffle_exact_remaining_deck_after_move() {
    for take in [false, true] {
        let mut g = game(0);
        let options = library(&mut g, 0, &["JC125", "JC032", "JC029", "JC085", "JC018"]);
        trigger(&mut g, "JZ50", 0, 0);
        accept(&mut g);
        assert_eq!(
            (
                g.pending.as_ref().unwrap().choice.min,
                g.pending.as_ref().unwrap().choice.max
            ),
            (Some(0), Some(1))
        );
        let mut expected = g.clone();
        if take {
            expected.players[0].deck.remove(1);
        }
        expected.shuffle_player(0);
        choose(
            &mut g,
            if take {
                vec![options[1].clone()]
            } else {
                vec![]
            },
        );
        assert_eq!(ids(&g, 0), ids(&expected, 0));
        assert_eq!(g.random, expected.random);
        assert_eq!(g.players[0].graveyard.len(), usize::from(take));
        if take {
            assert_eq!(g.players[0].graveyard[0].definition, "JC032");
            assert_ne!(g.players[0].graveyard[0].id, options[1]);
        }
        fixture(
            if take {
                "jz50-one"
            } else {
                "jz50-zero-with-candidates"
            },
            &g,
        );
    }
}

#[test]
fn jz50_empty_no_candidate_and_one_card_shuffle_follow_existing_rng_algorithm() {
    for defs in [
        vec![],
        vec!["JC125"],
        vec!["JC092", "JC093", "JC029", "JC125"],
    ] {
        let mut g = game(0);
        library(&mut g, 0, &defs);
        trigger(&mut g, "JZ50", 0, 0);
        accept(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "jz50_death_search");
        assert!(g.pending.as_ref().unwrap().choice.options.is_empty());
        assert_eq!(
            (
                g.pending.as_ref().unwrap().choice.min,
                g.pending.as_ref().unwrap().choice.max
            ),
            (Some(0), Some(1))
        );
        let mut expected = g.clone();
        expected.shuffle_player(0);
        choose(&mut g, vec![]);
        assert!(g.pending.is_none());
        assert_eq!(ids(&g, 0), ids(&expected, 0));
        assert_eq!(g.random, expected.random);
        assert!(g.players[0].graveyard.is_empty());
        checkpoint(&g);
    }
}

#[test]
fn jz50_private_candidates_and_choices_belong_to_controller_not_owner_or_team() {
    for actor in 0..4 {
        let owner = (actor + 2) % 4;
        let mut g = game(actor);
        let candidates = library(&mut g, actor, &["JC032", "JC085", "JC125"]);
        trigger(&mut g, "JZ50", owner, actor);
        accept(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().seat, actor);
        fixture(&format!("jz50-private-choice-{actor}"), &g);
        for viewer in 0..4 {
            let v = g.view(viewer);
            assert_eq!(v.pending_choice.is_some(), viewer == actor);
            if viewer != actor {
                let text = serde_json::to_string(&v).unwrap();
                assert!(!text.contains(&candidates[0]));
                assert!(!text.contains(&candidates[1]));
            }
        }
        let wrong = (actor + 1) % 4;
        let action = selection(&g, vec![candidates[0].clone()]);
        reject(&mut g, wrong, action);
        choose(&mut g, vec![candidates[0].clone()]);
        assert_eq!(g.players[actor].graveyard[0].definition, "JC032");
        assert!(g.players[owner].graveyard.is_empty());
        fixture("jz50-private-four-seat", &g);
    }
}

#[test]
fn jz50_frozen_actor_survives_source_control_change_departure_and_region_replacement() {
    let mut g = game(2);
    let picked = library(&mut g, 2, &["JC032", "JC125"])[0].clone();
    let source = trigger(&mut g, "JZ50", 0, 2);
    choose(&mut g, vec!["accept".into()]);
    g.board_mut(&source).unwrap().controller = 3;
    g.remove_dead(&source, RemovalCause::Destroy);
    g.regions[2].card = g.make_card("DQJC115", 0);
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    choose(&mut g, vec![picked]);
    assert_eq!(g.players[2].graveyard[0].definition, "JC032");
    assert_eq!(g.players[0].graveyard[0].definition, "JZ50");
    checkpoint(&g);
}

#[test]
fn jz50_deck_to_graveyard_resets_owner_state_and_never_emits_death() {
    let mut g = game(0);
    let id = library(&mut g, 0, &["BQ083", "JC125"])[0].clone();
    trigger(&mut g, "JZ50", 0, 0);
    let frame = accept(&mut g);
    // Explicit borrowed/dirty zone card input to exercise the existing mill reset.
    let c = &mut g.players[0].deck[0];
    c.owner = 2;
    c.controller = 3;
    c.face_down = true;
    c.exhausted = true;
    c.damage = 8;
    c.wounds = 4;
    c.shield = 3;
    g.jz50_search_complete(&frame, &[id]).unwrap();
    let c = &g.players[2].graveyard[0];
    assert_eq!((c.owner, c.controller), (2, 2));
    assert!(!c.face_down && !c.exhausted);
    assert_eq!((c.damage, c.wounds, c.shield), (0, 0, 0));
    assert!(!g.effects.iter().any(|e|matches!(e,Effect::Declare{declaration} if declaration.ability.event==Some(Event::Death))));
}

#[test]
fn jz50_stale_candidate_and_illegal_multi_choice_reject_atomically() {
    let mut g = game(0);
    let names = library(&mut g, 0, &["JC032", "JC032", "JC125"]);
    trigger(&mut g, "JZ50", 0, 0);
    let frame = accept(&mut g);
    let action = selection(&g, vec![names[0].clone(), names[1].clone()]);
    reject(&mut g, 0, action);
    let action = selection(&g, vec![names[2].clone()]);
    reject(&mut g, 0, action);
    let moved = g.players[0].deck.remove(0);
    g.players[0].hand.push(moved);
    let before = serde_json::to_string(&g).unwrap();
    assert!(g.jz50_search_complete(&frame, &[names[0].clone()]).is_err());
    assert_eq!(serde_json::to_string(&g).unwrap(), before);
    assert!(Game::from_persisted(&before).is_err());
    let moved = g.players[0].hand.pop().unwrap();
    g.players[0].deck.push(moved);
    choose(&mut g, vec![names[1].clone()]);
    assert_eq!(g.players[0].graveyard.len(), 1);
    assert!(g.players[0].deck.iter().any(|c| c.id == names[0]));
}

#[test]
fn search_family_printed_definition_and_dynamic_program_transplants_are_rejected() {
    for id in ["JZ50"] {
        for variant in 0..14 {
            let mut r = definitions().clone();
            let d = r.get_mut(id).unwrap();
            let a = &mut d.abilities[0];
            match variant {
                0 => a.event = Some(Event::Death),
                1 => a
                    .targets
                    .push(definition("JC003").abilities[0].targets[0].clone()),
                2 => a.costs.push(Cost::Assets(1)),
                3 => a.requires_ready_source = true,
                4 => a.once_per_game = true,
                5 => a.ops = vec![Op::ForEachLivingPlayer(a.ops.clone())],
                6 => a.key = "transplanted-search".into(),
                7 => a.per_turn_limit = Some(1),
                8 => a.play_only = true,
                9 => a.activation_only = true,
                10 => a.response_policy = ResponsePolicy::Immediate,
                11 => a.timing = Timing::Standard,
                12 => a.modes.push(Mode {
                    key: "nested-search".into(),
                    label: "nested".into(),
                    targets: vec![],
                    ops: a.ops.clone(),
                }),
                _ => {
                    a.ops = vec![Op::IfTargetExhausted {
                        slot: 0,
                        exhausted: Box::new(a.ops[0].clone()),
                        ready: Box::new(a.ops[0].clone()),
                    }]
                }
            }
            assert!(validate_definitions(&r).is_err(), "{id} variant{variant}");
        }
    }
    for id in ["JC125", "JC032", "JZ49"] {
        for source in ["JZ50"] {
            assert!(validate_ability(id, &definition(source).abilities[0]).is_err());
            let mut r = definitions().clone();
            r.get_mut(id)
                .unwrap()
                .abilities
                .push(definition(source).abilities[0].clone());
            assert!(validate_definitions(&r).is_err());
        }
    }
}

#[test]
fn search_family_saved_choices_and_stale_cas_reject_atomically() {
    for source in ["JZ50"] {
        let mut g = game(0);
        let selected = library(&mut g, 0, &["JC032", "JC125"]);
        trigger(&mut g, source, 0, 0);
        accept(&mut g);
        let mut room = envelope(&g);
        let action = selection(&g, vec![selected[0].clone()]);
        let command = RoomCommand {
            command_id: format!("{source}-search-choice"),
            expected_version: room.revision,
            action: SessionAction::Game { action },
        };
        let first = room.transition(0, Some(command.clone()), 0).unwrap();
        assert!(first.error_code.is_none());
        room = RoomEnvelope::from_persisted(&first.state).unwrap();
        let second = room.transition(0, Some(command.clone()), 0).unwrap();
        // Receipts belong to the native Store/Worker hosts, not this reducer.
        assert_eq!(second.error_code.as_deref(), Some("version_conflict"));
        assert_eq!(second.state, first.state);
        let mut changed = command;
        changed.expected_version = room.revision;
        let rejected = room.transition(0, Some(changed), 0).unwrap();
        assert!(rejected.error_code.is_some());
        assert_eq!(rejected.state, first.state);
        for seat in 0..4 {
            assert_eq!(
                serde_json::to_value(room.view(seat, 0)).unwrap(),
                serde_json::to_value(
                    RoomEnvelope::from_persisted(&first.state)
                        .unwrap()
                        .view(seat, 0)
                )
                .unwrap()
            );
        }
    }
}

#[test]
fn jz50_multiple_same_printed_sources_keep_fifo_actor_and_independent_decks() {
    let mut g = game(0);
    let p0 = library(&mut g, 0, &["JC032", "JC032", "JC125"]);
    let p2 = library(&mut g, 2, &["JC032", "JC085", "JC125"]);
    let a = board(&mut g, "JZ50", 0, 2);
    let b = board(&mut g, "JZ50", 2, 2);
    assert_ne!(a, b);
    g.enter_triggers(0, "JZ50", &a, true);
    g.enter_triggers(2, "JZ50", &b, true);
    g.drive().unwrap();
    // Existing declaration queue is FIFO, accepted stack entries resolve LIFO.
    for (actor, id) in [(0, a.clone()), (2, b.clone())] {
        let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
        else {
            panic!()
        };
        assert_eq!(declaration.actor, actor);
        assert_eq!(declaration.source.card.id, id);
        choose(&mut g, vec!["accept".into()]);
    }
    assert!(g.pending.is_none());
    assert_eq!(
        g.stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .source
            .card
            .id,
        b
    );
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    choose(&mut g, vec![p2[0].clone()]);
    assert_eq!(
        g.stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .source
            .card
            .id,
        a
    );
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    choose(&mut g, vec![p0[1].clone()]);
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.players[0].graveyard.len(), 1);
    assert_eq!(g.players[2].graveyard.len(), 1);
    assert!(g.players[0].deck.iter().any(|c| c.id == p0[0]));
    assert!(g.players[2].deck.iter().any(|c| c.id == p2[1]));
    fixture("jz50-two-fifo-sources", &g);
}

#[test]
fn jz50_frozen_trigger_survives_actual_paid_return_response() {
    let mut g = game(0);
    let picked = library(&mut g, 0, &["JC032", "JC125", "JC029"])[0].clone();
    fund(&mut g, 1, "JC002", 2);
    let spell = held(&mut g, "JC006", 1);
    let source = trigger(&mut g, "JZ50", 0, 0);
    choose(&mut g, vec!["accept".into()]);
    assert_eq!(g.stack.len(), 1);
    let frame_id = g.stack[0].frame.as_ref().unwrap().frame_id.clone();
    apply(
        &mut g,
        1,
        Action {
            card_id: Some(spell),
            target_id: Some(source.clone()),
            ..Action::new("play")
        },
    );
    assert_eq!(g.resources(1), 0);
    pass_top(&mut g);
    assert!(g.board(&source).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.definition == "JZ50"));
    assert_eq!(g.stack[0].frame.as_ref().unwrap().frame_id, frame_id);
    pass_top(&mut g);
    choose(&mut g, vec![picked]);
    assert_eq!(g.players[0].graveyard[0].definition, "JC032");
    assert_eq!(g.players[1].graveyard[0].definition, "JC006");
    fixture("jz50-paid-return-response", &g);
}

#[test]
fn jz50_hidden_deployment_has_no_search_and_exhausted_reveal_does_not_require_ready_source() {
    let mut g = game(0);
    fund(&mut g, 0, "JC084", 3);
    let source = held(&mut g, "JZ50", 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
    assert!(g.pending.is_none() && g.stack.is_empty());
    let id = g.regions[2].cards[0].id.clone();
    g.board_mut(&id).unwrap().exhausted = true;
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut g);
    assert!(g.regions[2].cards[0].exhausted);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
    choose(&mut g, vec![]);
    checkpoint(&g);
}

#[test]
fn jz50_source_program_and_actor_transplants_reject_before_any_move_or_shuffle() {
    let mut g = game(0);
    let picked = library(&mut g, 0, &["JC032", "JC125"])[0].clone();
    trigger(&mut g, "JZ50", 0, 0);
    let frame = accept(&mut g);
    for variant in 0..3 {
        let mut bad = frame.clone();
        match variant {
            0 => bad.source.card.definition = "JC125".into(),
            1 => bad.ability_key = "forged".into(),
            _ => bad.actor = 2,
        }
        let before = serde_json::to_string(&g).unwrap();
        assert!(g.jz50_search_complete(&bad, &[picked.clone()]).is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
    }
}

#[cfg(feature = "native")]
#[tokio::test]
async fn jz50_native_store_selection_receipt_survives_reopen_and_never_repeats_shuffle() {
    use crate::service::{CreateRoom, JoinRoom, Store};
    for take in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jz50-private-local-fixture.sqlite");
        let store = Store::open(&path).unwrap();
        let host = store
            .create(CreateRoom {
                name: "P0".into(),
                mode: "teams".into(),
                deck_id: "watchers".into(),
                deck_draft: None,
            })
            .await
            .unwrap();
        let mut sessions = vec![host];
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
        let picked = library(&mut g, 0, &["JC032", "JC125", "JC085", "JC029"])[0].clone();
        trigger(&mut g, "JZ50", 0, 0);
        accept(&mut g);
        g.room_id = sessions[0].room_id.clone();
        g.invite_code = sessions[0].invite_code.clone();
        let mut room = envelope(&g);
        // Explicit disposable test DB; no actual user/game database is touched.
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
        let command = RoomCommand {
            command_id: "jz50-store-selection".into(),
            expected_version: room.revision,
            action: SessionAction::Game {
                action: selection(&g, if take { vec![picked] } else { vec![] }),
            },
        };
        let expected = room.transition(0, Some(command.clone()), 0).unwrap();
        assert!(expected.error_code.is_none());
        let store = Store::open(&path).unwrap();
        let first = store
            .command_at_now(&g.room_id, &sessions[0].token, command.clone(), 0)
            .await
            .unwrap();
        drop(store);
        room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(room.view(0, 0)).unwrap()
        );
        let store = Store::open(&path).unwrap();
        let duplicate = store
            .command_at_now(&g.room_id, &sessions[0].token, command.clone(), 90_000)
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&duplicate).unwrap(),
            serde_json::to_value(first).unwrap()
        );
        let mut conflicting = command;
        conflicting.expected_version += 1;
        assert_eq!(
            store
                .command_at_now(&g.room_id, &sessions[0].token, conflicting, 90_000)
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
        assert_eq!(stored, expected.state);
        let receipts: u64 = db
            .query_row(
                "SELECT COUNT(*) FROM commands WHERE room_id=?1",
                [&g.room_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(receipts, 1);
    }
}

#[test]
fn jz50_private_choice_hides_candidate_hits_from_all_other_seats_even_when_empty() {
    for actor in 0..4 {
        let mut hit = game(actor);
        library(&mut hit, actor, &["JC032", "JC125", "JC029"]);
        trigger(&mut hit, "JZ50", actor, actor);
        accept(&mut hit);
        let mut miss = game(actor);
        library(&mut miss, actor, &["JC029", "JC125", "JC029"]);
        trigger(&mut miss, "JZ50", actor, actor);
        accept(&mut miss);
        assert_eq!(hit.pending.as_ref().unwrap().choice.options.len(), 1);
        assert!(miss.pending.as_ref().unwrap().choice.options.is_empty());
        fixture(&format!("jz50-empty-choice-{actor}"), &miss);
        for viewer in 0..4 {
            let h = hit.view(viewer);
            let m = miss.view(viewer);
            if viewer == actor {
                assert!(h.pending_choice.is_some() && m.pending_choice.is_some());
            } else {
                assert!(h.pending_choice.is_none() && m.pending_choice.is_none());
                assert_eq!(
                    serde_json::to_value(h).unwrap(),
                    serde_json::to_value(m).unwrap()
                );
            }
        }
        choose(&mut hit, vec![]);
        choose(&mut miss, vec![]);
        assert!(hit.pending.is_none() && miss.pending.is_none());
    }
}

#[test]
fn jz50_saved_private_choice_rejects_changed_actor_program_bounds_and_library_instances() {
    for actor in 0..4 {
        let mut g = game(actor);
        library(&mut g, actor, &["JC032", "JC125", "JC085"]);
        let source = trigger(&mut g, "JZ50", (actor + 2) % 4, actor);
        accept(&mut g);
        g.remove_dead(&source, RemovalCause::Destroy);
        let valid = serde_json::to_string(&g).unwrap();
        Game::from_persisted(&valid).unwrap();
        envelope(&g);
        for variant in 0..21 {
            let mut bad = g.clone();
            let p = bad.pending.as_mut().unwrap();
            let ChoiceResolution::Frame { frame, .. } = &mut p.resolution else {
                panic!()
            };
            match variant {
                0 => p.seat = (actor + 1) % 4,
                1 => p.choice.player_id = format!("p{}", (actor + 2) % 4),
                2 => frame.actor = (actor + 2) % 4,
                3 => frame.source.card.controller = (actor + 1) % 4,
                4 => frame.source.card.definition = "JC125".into(),
                5 => frame.ability_key = "wrong".into(),
                6 => frame.cursor = 0,
                7 => frame.steps[0].context = (actor + 1) % 4,
                8 => frame.steps[0].op = Op::JC032TopSixVampireHidden,
                9 => frame.guard = GuardState::Unchecked,
                10 => frame.steps.push(frame.steps[0].clone()),
                11 => p.choice.min = Some(1),
                12 => p.choice.max = Some(2),
                13 => p.choice.allow_decline = Some(true),
                14 => p.choice.kind = "search".into(),
                15 => p.choice.options.push(p.choice.options[0].clone()),
                16 => {
                    p.choice.options.pop();
                }
                17 => p.choice.options[0].id = "missing-deck-instance".into(),
                18 => {
                    bad.players[actor].deck.remove(0);
                }
                19 => frame.source.card.owner = 99,
                _ => bad.players[actor].deck[0].definition = "unadmitted".into(),
            }
            assert!(
                Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),
                "actor{actor} variant{variant}"
            );
            let mut room = envelope(&g);
            room.game = bad;
            assert!(
                RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err(),
                "Room actor{actor} variant{variant}"
            );
        }
    }
    for wrapped in [
        Op::ForEachLivingPlayer(vec![Op::JZ50SearchDeathToGraveyard]),
        Op::ForEachLivingPlayerFromActor(vec![Op::JZ50SearchDeathToGraveyard]),
        Op::IfTargetExhausted {
            slot: 0,
            exhausted: Box::new(Op::JZ50SearchDeathToGraveyard),
            ready: Box::new(Op::JC032TopSixVampireHidden),
        },
        Op::IfTargetExhausted {
            slot: 0,
            exhausted: Box::new(Op::JC032TopSixVampireHidden),
            ready: Box::new(Op::JZ50SearchDeathToGraveyard),
        },
    ] {
        let mut defs = definitions().clone();
        let mut ability = definition("JZ50").abilities[0].clone();
        ability.ops = vec![wrapped];
        defs.get_mut("LC01").unwrap().abilities.push(ability);
        assert!(validate_definitions(&defs).is_err());
    }
    let mut defs = definitions().clone();
    defs.get_mut("JZ50").unwrap().traits.spirit = true;
    assert!(validate_definitions(&defs).is_err());
}

#[test]
fn jz50_optional_trigger_stack_and_private_search_pause_resume_real_room_chain() {
    let mut g = game(0);
    let pick = library(&mut g, 0, &["JC032", "JC125", "JC085", "JC029"])[0].clone();
    trigger(&mut g, "JZ50", 2, 0);
    let mut room = envelope(&g);
    let initial = serde_json::to_string(&room).unwrap();
    let mut steps = vec![];
    let mut record = |room: &mut RoomEnvelope,
                      seat: usize,
                      action: Option<SessionAction>,
                      now: u64| {
        let command = action.map(|action| RoomCommand {
            command_id: format!("jz50-chain-{}", steps.len()),
            expected_version: room.revision,
            action,
        });
        let transition = room.transition(seat, command.clone(), now).unwrap();
        if transition.changed {
            assert_eq!(
                serde_json::to_string(&room.replay_events(&transition.journal).unwrap()).unwrap(),
                transition.state
            );
        }
        *room = RoomEnvelope::from_persisted(&transition.state).unwrap();
        steps.push(serde_json::json!({"seat":seat,"command":command,"serverNowMs":now.to_string(),"transition":transition,
            "views":(0..4).map(|s| room.view(s,room.pacing.last_server_now_ms)).collect::<Vec<_>>() }));
    };
    record(&mut room, 2, Some(SessionAction::PauseRoom), 1000);
    record(&mut room, 1, None, 86400000);
    record(&mut room, 3, Some(SessionAction::ResumeRoom), 86400000);
    let id = room.game.pending.as_ref().unwrap().choice.id.clone();
    record(
        &mut room,
        0,
        Some(SessionAction::Game {
            action: Action {
                choice_id: Some(id),
                selected: Some(vec!["accept".into()]),
                ..Action::new("choose")
            },
        }),
        86400001,
    );
    assert_eq!(room.game.stack.len(), 1);
    record(&mut room, 1, Some(SessionAction::PauseRoom), 86401000);
    record(&mut room, 3, None, 172800000);
    record(&mut room, 2, Some(SessionAction::ResumeRoom), 172800000);
    let mut now = 172800000;
    for _ in 0..8 {
        if room.game.pending.is_some() {
            break;
        }
        now += 5000;
        record(&mut room, 0, None, now);
    }
    assert_eq!(
        room.game.pending.as_ref().unwrap().choice.kind,
        "jz50_death_search"
    );
    let original = room.game.pending.as_ref().unwrap().choice.id.clone();
    let select = SessionAction::Game {
        action: Action {
            choice_id: Some(original.clone()),
            selected: Some(vec![pick]),
            ..Action::new("choose")
        },
    };
    let rng = room.game.random;
    record(&mut room, 3, Some(SessionAction::PauseRoom), now + 1);
    record(&mut room, 2, None, 345600000);
    record(&mut room, 0, Some(select.clone()), 345600000);
    assert_eq!(room.game.random, rng);
    record(&mut room, 1, Some(SessionAction::ResumeRoom), 345600000);
    assert_eq!(room.game.pending.as_ref().unwrap().choice.id, original);
    record(&mut room, 0, Some(select), 345600001);
    assert_eq!(room.game.players[0].graveyard.len(), 1);
    assert_eq!(room.game.players[0].graveyard[0].definition, "JC032");
    assert!(room.game.pending.is_none() && room.game.stack.is_empty());
    if let Ok(dir) = std::env::var("JZ50_ROOM_TRACE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/native-trace.json"),serde_json::to_vec(&serde_json::json!({"scope":"explicit-offline-native-JZ50-trigger-stack-private-search-pause","initialState":initial,"steps":steps})).unwrap()).unwrap();
    }
}
