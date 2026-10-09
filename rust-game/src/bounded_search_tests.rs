//! Explicit offline layouts followed by real declarations, paid plays,
//! response commands, Room restoration and SQLite receipt handling.
use crate::bounded_search::{bq104_search_match, xq48_search_match};
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{
    catalog,
    model::*,
    room::{RoomCommand, RoomEnvelope, SessionAction},
    rules::*,
};
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
fn held(g: &mut Game, def: &str, owner: usize) -> String {
    let c = g.make_card(def, owner);
    let id = c.id.clone();
    g.players[owner].hand.push(c);
    id
}
fn trigger(g: &mut Game, def: &str, owner: usize, actor: usize) -> String {
    let id = board(g, def, owner, 2);
    g.board_mut(&id).unwrap().controller = actor;
    g.enter_triggers(actor, def, &id, false);
    g.drive().unwrap();
    checkpoint(g);
    id
}
fn accept(g: &mut Game) {
    choose(g, vec!["accept".into()]);
    pass_top(g);
}
fn selection(g: &Game, selected: Vec<String>) -> Action {
    Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(selected),
        ..Action::new("choose")
    }
}
fn fixture(kind: &str, g: &Game) {
    if let Ok(dir) = std::env::var("ENTRY_SEARCH_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{kind}.json"), serde_json::to_vec(&serde_json::json!({"kind":kind,
            "method":"Explicit offline Native layout; real declarations/commands; not a natural playtest",
            "catalog":catalog::catalog(),"state":serde_json::to_string(&r).unwrap(),
            "views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>() })).unwrap()).unwrap();
    }
    checkpoint(g);
}
#[test]
fn entry_search_originals_and_closed_definitions_are_complete() {
    assert_eq!(catalog::catalog().cards.len(), 116);
    for (id, name, cost, subtype, renown) in [
        ("BQ104", "猎头顾问", 3, "雇员", false),
        ("XQ48", "街头演说家", 4, "政治家", true),
    ] {
        let c = catalog::card(id);
        assert_eq!(
            (&*c.name, c.cost, &*c.color, c.defense),
            (name, cost, "中立", Some(1))
        );
        assert!(c.loyalty.is_empty() && c.magic_icon == MagicIcon::None && !c.unique);
        assert_eq!(c.subtypes, ["人类", subtype]);
        assert_eq!(
            c.permanent_icons,
            Icons {
                influence: 1,
                ..Default::default()
            }
        );
        assert_eq!(c.temporary_icons, Icons::default());
        let d = definition(id);
        assert!(d.traits.public);
        assert_eq!(d.traits.renown, renown);
        assert_eq!(d.abilities.len(), 1);
        assert_eq!(d.abilities[0].event, Some(Event::Enter));
        assert!(
            d.abilities[0].costs.is_empty()
                && d.abilities[0].targets.is_empty()
                && d.abilities[0].modes.is_empty()
        );
    }
    for id in ["BQ104", "XQ36"] {
        assert!(bq104_search_match(catalog::card(id)));
    }
    for id in ["JC125", "XQ48", "JC089", "LC01"] {
        assert!(!bq104_search_match(catalog::card(id)));
    }
    let mut c = catalog::card("BQ104").clone();
    c.cost = 4;
    assert!(!bq104_search_match(&c));
    c.cost = 0;
    c.kind = "spell".into();
    assert!(!bq104_search_match(&c));
    assert!(xq48_search_match(catalog::card("JC125")));
    c = catalog::card("JC125").clone();
    c.id = "fake-alias".into();
    assert!(!xq48_search_match(&c));
}
#[test]
fn entry_search_paid_public_deploy_optional_decline_and_accepted_search() {
    for def in ["BQ104", "XQ48"] {
        for take in [false, true] {
            let mut g = game(0);
            let picked = library(&mut g, 0, &["BQ104", "JC125", "XQ36", "JC125"]);
            fund(&mut g, 0, "JC084", catalog::card(def).cost as usize);
            let id = held(&mut g, def, 0);
            reject(
                &mut g,
                0,
                Action {
                    card_id: Some(id.clone()),
                    region: Some(2),
                    ..Action::new("conceal")
                },
            );
            apply(
                &mut g,
                0,
                Action {
                    card_id: Some(id),
                    region: Some(2),
                    ..Action::new("deploy")
                },
            );
            assert_eq!(g.resources(0), 0);
            assert!(g.pending.is_none());
            pass_top(&mut g);
            assert!(g.pending.as_ref().unwrap().choice.allow_decline.unwrap());
            let before_rng = g.random;
            let before_deck = g.players[0].deck.clone();
            if take {
                accept(&mut g);
                choose(
                    &mut g,
                    vec![picked[if def == "BQ104" { 0 } else { 1 }].clone()],
                );
                assert_eq!(g.players[0].deck.len(), 3);
                assert_ne!(g.random, before_rng);
            } else {
                choose(&mut g, vec![]);
                assert_eq!(g.random, before_rng);
                assert_eq!(
                    serde_json::to_value(&g.players[0].deck).unwrap(),
                    serde_json::to_value(before_deck).unwrap()
                );
            }
            checkpoint(&g);
        }
    }
}
#[test]
fn bq104_accept_with_matches_forces_one_and_empty_search_completes_zero_shuffle() {
    for (defs, take) in [
        (&["XQ36", "JC125", "BQ104"][..], true),
        (&["JC125", "XQ48", "JC125"][..], false),
        (&[][..], false),
    ] {
        let mut g = game(0);
        let ids = library(&mut g, 0, defs);
        trigger(&mut g, "BQ104", 0, 0);
        accept(&mut g);
        let p = g.pending.as_ref().unwrap();
        assert_eq!(
            (p.choice.min, p.choice.max, p.choice.allow_decline),
            (
                Some(usize::from(take)),
                Some(usize::from(take)),
                Some(false)
            )
        );
        fixture(if take { "bq104-choice" } else { "bq104-empty" }, &g);
        if take {
            let a = selection(&g, vec![]);
            reject(&mut g, 0, a);
            let a = selection(&g, vec![ids[0].clone(), ids[2].clone()]);
            reject(&mut g, 0, a);
        }
        let before = g.random;
        choose(&mut g, if take { vec![ids[0].clone()] } else { vec![] });
        assert!(g.pending.is_none());
        if defs.len() > 1 {
            assert_ne!(g.random, before);
        }
        fixture(
            if take {
                "bq104-hidden-result"
            } else {
                "bq104-empty-result"
            },
            &g,
        );
    }
}
#[test]
fn bq104_borrowed_employee_reveals_then_enters_hidden_for_frozen_controller() {
    for actor in 0..4 {
        let owner = (actor + 2) % 4;
        let mut g = game(actor);
        let ids = library(&mut g, actor, &["XQ36", "JC125", "JC125"]);
        g.players[actor].deck[0].owner = owner;
        g.players[actor].deck[0].controller = owner;
        let source = trigger(&mut g, "BQ104", owner, actor);
        accept(&mut g);
        let before_other = (0..4).map(|s| g.players[s].deck.len()).collect::<Vec<_>>();
        g.board_mut(&source).unwrap().controller = (actor + 1) % 4;
        choose(&mut g, vec![ids[0].clone()]);
        let c = g.regions[2]
            .cards
            .iter()
            .find(|c| c.definition == "XQ36")
            .unwrap();
        assert_eq!((c.owner, c.controller, c.face_down), (owner, actor, true));
        assert_ne!(c.id, ids[0]);
        assert!(g.pending.is_none() && g.stack.is_empty());
        assert!(g
            .log
            .iter()
            .any(|l| l.text.contains("展示检索的 圣甲虫的清理员")));
        for s in 0..4 {
            assert_eq!(
                g.players[s].deck.len(),
                before_other[s] - usize::from(s == actor)
            );
        }
        let instance = c.id.clone();
        for viewer in 0..4 {
            let v = g.view(viewer);
            let card = v.regions[2]
                .characters
                .iter()
                .find(|c| c.instance_id == instance)
                .unwrap();
            if viewer == actor {
                assert_eq!(card.card_id.as_deref(), Some("XQ36"));
            } else {
                assert!(card.card_id.is_none());
            }
        }
        fixture(&format!("bq104-borrowed-result-{actor}"), &g);
    }
}
#[test]
fn entry_search_private_choices_hide_candidates_from_opponents_and_teammates() {
    for actor in 0..4 {
        for def in ["BQ104", "XQ48"] {
            let mut hit = game(actor);
            library(&mut hit, actor, &["BQ104", "JC125", "XQ36", "JC125"]);
            let mut miss = hit.clone();
            for c in &mut miss.players[actor].deck {
                c.definition = "XQ48".into();
            }
            trigger(&mut hit, def, (actor + 2) % 4, actor);
            trigger(&mut miss, def, (actor + 2) % 4, actor);
            accept(&mut hit);
            accept(&mut miss);
            for viewer in 0..4 {
                if viewer == actor {
                    assert!(
                        hit.view(viewer).pending_choice.is_some()
                            && miss.view(viewer).pending_choice.is_some()
                    );
                } else {
                    assert!(hit.view(viewer).pending_choice.is_none());
                    assert_eq!(
                        serde_json::to_value(hit.view(viewer)).unwrap(),
                        serde_json::to_value(miss.view(viewer)).unwrap()
                    );
                }
            }
        }
    }
}
#[test]
fn xq48_zero_one_three_real_instances_shuffle_once_and_have_original_owners() {
    for count in [0, 1, 3] {
        for actor in 0..4 {
            let mut g = game(actor);
            let ids = library(
                &mut g,
                actor,
                &["JC125", "JC125", "JC125", "JC125", "BQ104", "JC125"],
            );
            g.players[actor].deck[0].owner = (actor + 2) % 4;
            trigger(&mut g, "XQ48", (actor + 1) % 4, actor);
            accept(&mut g);
            fixture(&format!("xq48-choice-{count}-{actor}"), &g);
            let mut expected = g.clone();
            expected.players[actor].deck.drain(0..count);
            expected.shuffle_player(actor);
            choose(&mut g, ids[..count].to_vec());
            assert_eq!(g.random, expected.random);
            assert_eq!(
                g.players[actor]
                    .deck
                    .iter()
                    .map(|c| &c.id)
                    .collect::<Vec<_>>(),
                expected.players[actor]
                    .deck
                    .iter()
                    .map(|c| &c.id)
                    .collect::<Vec<_>>()
            );
            let entered = g.regions[2]
                .cards
                .iter()
                .filter(|c| c.definition == "JC125")
                .collect::<Vec<_>>();
            assert_eq!(entered.len(), count);
            for (n, c) in entered.iter().enumerate() {
                assert!(!c.face_down);
                assert_eq!(c.controller, actor);
                assert_eq!(c.owner, if n == 0 { (actor + 2) % 4 } else { actor });
                assert!(!ids.contains(&c.id));
            }
            assert!(g.pending.is_none() && g.stack.is_empty());
            fixture(&format!("xq48-result-{count}-{actor}"), &g);
        }
    }
}
#[test]
fn xq48_empty_search_still_has_explicit_zero_completion() {
    let mut g = game(0);
    library(&mut g, 0, &["BQ104", "XQ36", "XQ48"]);
    trigger(&mut g, "XQ48", 0, 0);
    accept(&mut g);
    let p = g.pending.as_ref().unwrap();
    assert_eq!((p.choice.min, p.choice.max), (Some(0), Some(0)));
    assert!(p.choice.options.is_empty());
    fixture("xq48-empty", &g);
    let mut expected = g.clone();
    expected.shuffle_player(0);
    choose(&mut g, vec![]);
    assert_eq!(g.random, expected.random);
    assert!(g.pending.is_none());
}
#[test]
fn entry_search_invalid_duplicate_foreign_stale_and_excess_selection_is_atomic() {
    for def in ["BQ104", "XQ48"] {
        let mut g = game(0);
        let ids = library(
            &mut g,
            0,
            &["BQ104", "JC125", "JC125", "JC125", "JC125", "XQ36"],
        );
        let foreign = library(&mut g, 2, &["BQ104", "JC125"]);
        trigger(&mut g, def, 0, 0);
        accept(&mut g);
        let valid = ids[if def == "BQ104" { 0 } else { 1 }].clone();
        for selected in [
            vec![valid.clone(), valid.clone()],
            vec![foreign[0].clone()],
            vec!["missing".into()],
            vec![
                ids[0].clone(),
                ids[1].clone(),
                ids[2].clone(),
                ids[3].clone(),
            ],
        ] {
            let a = selection(&g, selected);
            reject(&mut g, 0, a);
        }
        let a = selection(&g, vec![valid.clone()]);
        reject(&mut g, 1, a);
        let old = selection(&g, vec![valid.clone()]);
        choose(&mut g, vec![valid]);
        reject(&mut g, 0, old);
    }
}
#[test]
fn entry_search_source_leaves_moves_hides_or_changes_control_without_retargeting() {
    for def in ["BQ104", "XQ48"] {
        for variant in 0..5 {
            let mut g = game(0);
            let ids = library(&mut g, 0, &["BQ104", "JC125", "JC125"]);
            let source = trigger(&mut g, def, 2, 0);
            choose(&mut g, vec!["accept".into()]);
            match variant {
                0 => g.remove_dead(&source, RemovalCause::Destroy),
                1 => g.remove_dead(&source, RemovalCause::Sacrifice),
                2 => g.board_mut(&source).unwrap().face_down = true,
                3 => g.board_mut(&source).unwrap().controller = 2,
                _ => {
                    let i = g.regions[2]
                        .cards
                        .iter()
                        .position(|c| c.id == source)
                        .unwrap();
                    let c = g.regions[2].cards.remove(i);
                    g.regions[1].cards.push(c);
                }
            }
            checkpoint(&g);
            pass_top(&mut g);
            choose(
                &mut g,
                vec![ids[if def == "BQ104" { 0 } else { 1 }].clone()],
            );
            assert!(g.regions[2].cards.iter().any(|c| c.definition
                == if def == "BQ104" { "BQ104" } else { "JC125" }
                && c.controller == 0
                && c.id != source));
            assert!(!g.regions[1].cards.iter().any(|c| c.id != source));
            checkpoint(&g);
        }
    }
}
#[test]
fn xq48_region_replaced_before_declaration_or_during_response_skips_search_and_shuffles() {
    for before_accept in [false, true] {
        let mut g = game(0);
        library(&mut g, 0, &["JC125", "JC125", "JC125", "BQ104"]);
        trigger(&mut g, "XQ48", 0, 0);
        if !before_accept {
            choose(&mut g, vec!["accept".into()]);
        }
        let old = g.regions[2].card.id.clone();
        g.regions[2].card = g.make_card("DQJC107", 0);
        assert_ne!(g.regions[2].card.id, old);
        if before_accept {
            choose(&mut g, vec!["accept".into()]);
        }
        let mut expected = g.clone();
        expected.shuffle_player(0);
        pass_top(&mut g);
        assert!(g.pending.is_none());
        assert_eq!(g.random, expected.random);
        assert_eq!(g.regions[2].cards.len(), 1);
        assert_eq!(g.players[0].deck.len(), 4);
        fixture("xq48-replaced-region", &g);
    }
}
#[test]
fn bq104_lost_original_region_preserves_search_reveal_shuffle_without_placing_in_replacement() {
    let mut g = game(0);
    let ids = library(&mut g, 0, &["XQ36", "JC125", "JC125"]);
    trigger(&mut g, "BQ104", 0, 0);
    choose(&mut g, vec!["accept".into()]);
    g.regions[2].card = g.make_card("DQJC107", 0);
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(1));
    choose(&mut g, vec![ids[0].clone()]);
    assert_eq!(g.players[0].deck.len(), 3);
    assert!(g.players[0].deck.iter().any(|c| c.id == ids[0]));
    assert_eq!(g.regions[2].cards.len(), 1);
    assert!(g.log.iter().any(|l| l.text.contains("展示检索的")));
    checkpoint(&g);
}
#[test]
fn entry_search_two_same_sources_and_same_name_instances_resolve_independently() {
    let mut g = game(0);
    let ids = library(&mut g, 0, &["BQ104", "BQ104", "JC125"]);
    let first = board(&mut g, "BQ104", 0, 2);
    let second = board(&mut g, "BQ104", 0, 2);
    g.enter_triggers(0, "BQ104", &first, false);
    g.enter_triggers(0, "BQ104", &second, false);
    g.drive().unwrap();
    // Both simultaneous declarations enter the stack before LIFO resolution.
    choose(&mut g, vec!["accept".into()]);
    choose(&mut g, vec!["accept".into()]);
    for id in &ids[..2] {
        pass_top(&mut g);
        choose(&mut g, vec![id.clone()]);
    }
    assert_eq!(
        g.regions[2]
            .cards
            .iter()
            .filter(|c| c.definition == "BQ104" && c.face_down)
            .count(),
        2
    );
    assert!(g.pending.is_none() && g.stack.is_empty());
    checkpoint(&g);
}
#[test]
fn bq104_hidden_public_employee_has_no_entry_trigger_until_real_paid_reveal() {
    let mut g = game(0);
    let ids = library(&mut g, 0, &["BQ104", "JC125", "JC125"]);
    trigger(&mut g, "BQ104", 0, 0);
    accept(&mut g);
    choose(&mut g, vec![ids[0].clone()]);
    assert!(g.pending.is_none());
    let hidden = g.regions[2]
        .cards
        .iter()
        .find(|c| c.face_down)
        .unwrap()
        .id
        .clone();
    fund(&mut g, 0, "JC084", 3);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(hidden.clone()),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut g);
    assert!(g.pending.is_some());
    assert!(!g.regions[2].cards.iter().any(|c| c.id == hidden));
    choose(&mut g, vec![]);
    checkpoint(&g);
}
#[test]
fn entry_search_saved_choice_rejects_changed_program_bounds_payloads_and_deck() {
    for def in ["BQ104", "XQ48"] {
        for actor in 0..4 {
            let mut g = game(actor);
            library(&mut g, actor, &["BQ104", "JC125", "XQ36", "JC125"]);
            trigger(&mut g, def, (actor + 2) % 4, actor);
            accept(&mut g);
            for variant in 0..24 {
                let mut bad = g.clone();
                let p = bad.pending.as_mut().unwrap();
                let ChoiceResolution::Frame { frame, .. } = &mut p.resolution else {
                    panic!()
                };
                match variant {
                    0 => p.seat = (actor + 1) % 4,
                    1 => p.choice.player_id = "p99".into(),
                    2 => frame.actor = (actor + 1) % 4,
                    3 => frame.source.card.controller = (actor + 1) % 4,
                    4 => frame.source.card.definition = "JC125".into(),
                    5 => frame.ability_key = "wrong".into(),
                    6 => frame.cursor = 0,
                    7 => frame.steps[0].context = (actor + 1) % 4,
                    8 => frame.steps[0].op = Op::JZ50SearchDeathToGraveyard,
                    9 => frame.guard = GuardState::Unchecked,
                    10 => frame.steps.push(frame.steps[0].clone()),
                    11 => p.choice.min = Some(9),
                    12 => p.choice.max = Some(9),
                    13 => p.choice.allow_decline = Some(true),
                    14 => p.choice.kind = "search".into(),
                    15 => p.choice.options.push(p.choice.options[0].clone()),
                    16 => {
                        p.choice.options.pop();
                    }
                    17 => p.choice.options[0].id = "missing".into(),
                    18 => {
                        let id = p.choice.options[0].id.clone();
                        let i = bad.players[actor]
                            .deck
                            .iter()
                            .position(|c| c.id == id)
                            .unwrap();
                        bad.players[actor].deck.remove(i);
                    }
                    19 => frame.source.card.owner = 99,
                    20 => bad.players[actor].deck[0].definition = "unadmitted".into(),
                    21 => frame.source.source_region_instance = None,
                    22 => frame.source.region = None,
                    _ => p.choice.options[0].label = "forged public face".into(),
                }
                assert!(
                    Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),
                    "{def} actor{actor} variant{variant}"
                );
                let mut room = envelope(&g);
                room.game = bad;
                if let Ok(dir) = std::env::var("ENTRY_SEARCH_INVALID_DIR") {
                    std::fs::create_dir_all(&dir).unwrap();
                    std::fs::write(
                        format!("{dir}/{def}-{actor}-{variant}.json"),
                        serde_json::to_string(&room).unwrap(),
                    )
                    .unwrap();
                }
                assert!(
                    RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err()
                );
            }
            checkpoint(&g);
        }
    }
}
#[test]
fn entry_search_definitions_reject_direct_nested_modes_and_partial_transplants() {
    for id in ["BQ104", "XQ48"] {
        let d = definition(id).clone();
        let a = d.abilities[0].clone();
        for foreign in ["JC125", "JZ50", "XQ36"] {
            assert!(validate_ability(foreign, &a).is_err());
        }
        for wrap in [
            Op::ForEachLivingPlayer(a.ops.clone()),
            Op::ForEachLivingPlayerFromActor(a.ops.clone()),
            Op::IfTargetExhausted {
                slot: 0,
                exhausted: Box::new(a.ops[0].clone()),
                ready: Box::new(Op::JZ50SearchDeathToGraveyard),
            },
        ] {
            let mut bad = definition("JC125").clone();
            bad.abilities = vec![a.clone()];
            bad.abilities[0].ops = vec![wrap];
            let mut defs = definitions().clone();
            defs.insert("JC125".into(), bad);
            assert!(validate_definitions(&defs).is_err());
        }
        for variant in 0..5 {
            let mut bad = d.clone();
            match variant {
                0 => bad.traits.public = false,
                1 => bad.abilities.clear(),
                2 => bad.abilities[0].event = Some(Event::Reveal),
                3 => bad.abilities[0].ops.push(a.ops[0].clone()),
                _ => bad.traits.renown = !bad.traits.renown,
            }
            let mut defs = definitions().clone();
            defs.insert(id.into(), bad);
            assert!(validate_definitions(&defs).is_err());
        }
    }
}
#[cfg(feature = "native")]
#[tokio::test]
async fn entry_search_sqlite_receipt_reopen_duplicate_and_conflict_never_repeat_shuffle() {
    use crate::service::{CreateRoom, JoinRoom, Store};
    for (def, take) in [("BQ104", 1), ("XQ48", 0), ("XQ48", 3)] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("entry-search-fixture.sqlite");
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
        for seat in 1..4 {
            store
                .join(JoinRoom {
                    invite_code: host.invite_code.clone(),
                    name: format!("P{seat}"),
                    deck_id: "watchers".into(),
                    deck_draft: None,
                })
                .await
                .unwrap();
        }
        drop(store);
        let mut g = game(0);
        let ids = library(
            &mut g,
            0,
            if def == "BQ104" {
                &["BQ104", "JC125", "JC125", "JC125"]
            } else {
                &["JC125", "JC125", "JC125", "BQ104"]
            },
        );
        trigger(&mut g, def, 2, 0);
        accept(&mut g);
        g.room_id = host.room_id.clone();
        g.invite_code = host.invite_code.clone();
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
        let command = RoomCommand {
            command_id: "entry-search-receipt".into(),
            expected_version: room.revision,
            action: SessionAction::Game {
                action: selection(&g, ids[..take].to_vec()),
            },
        };
        let expected = room.transition(0, Some(command.clone()), 0).unwrap();
        assert!(expected.error_code.is_none());
        let store = Store::open(&path).unwrap();
        let first = store
            .command_at_now(&g.room_id, &host.token, command.clone(), 0)
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
            .command_at_now(&g.room_id, &host.token, command.clone(), 90000)
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(&duplicate).unwrap(),
            serde_json::to_value(&first).unwrap()
        );
        let mut conflicting = command;
        conflicting.expected_version += 1;
        assert_eq!(
            store
                .command_at_now(&g.room_id, &host.token, conflicting, 90000)
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
fn entry_search_real_damage_response_returns_sealed_payload_to_owner_and_source_trigger_survives() {
    for def in ["BQ104", "XQ48"] {
        let mut g = game(0);
        let ids = library(&mut g, 0, &["BQ104", "JC125", "XQ36", "JC125"]);
        let source = trigger(&mut g, def, 2, 0);
        // Explicit already-sealed carrier layout, then an actual paid responsive
        // damage spell and normal carrier-leave/owner-return handling.
        let payload = held(&mut g, "JC125", 1);
        g.seal_hand_card(1, &payload, &source).unwrap();
        assert_eq!(g.sealed_cards.len(), 1);
        choose(&mut g, vec!["accept".into()]);
        fund(&mut g, 2, "JC104", 2);
        let spell = held(&mut g, "JC102", 2);
        // The actor's team holds priority first; pass both real members before
        // the enemy submits its responsive damage intent.
        for seat in [0, 1] {
            apply(&mut g, seat, Action::new("pass"));
        }
        apply(
            &mut g,
            2,
            Action {
                card_id: Some(spell),
                target_id: Some(source.clone()),
                ..Action::new("play")
            },
        );
        pass_top(&mut g);
        assert!(g.board(&source).is_none() && g.sealed_cards.is_empty());
        assert!(g.players[2].graveyard.iter().any(|c| c.definition == def));
        assert!(g.players[1]
            .hand
            .iter()
            .any(|c| c.definition == "JC125" && c.id != payload));
        pass_top(&mut g);
        choose(
            &mut g,
            vec![ids[if def == "BQ104" { 0 } else { 1 }].clone()],
        );
        assert_eq!(g.players[0].deck.len(), 3);
        fixture(&format!("{def}-carrier-left-result"), &g);
    }
}

#[test]
fn xq48_renown_uses_existing_team_comparison_one_influence_and_excludes_hidden_exhausted() {
    for (hidden, exhausted, enemy, reward) in [
        (false, false, false, true),
        (true, false, false, false),
        (false, true, false, false),
        (false, false, true, false),
    ] {
        let mut g = game(0);
        g.regions[2].card = g.make_card("DQJC115", 0);
        g.regions[2].influence = [0, 0];
        let id = board(&mut g, "XQ48", 0, 2);
        g.board_mut(&id).unwrap().face_down = hidden;
        g.board_mut(&id).unwrap().exhausted = exhausted;
        if enemy {
            board(&mut g, "XQ48", 2, 2);
        }
        let region_id = g.regions[2].card.id.clone();
        g.begin_window(Window::After(2, 2));
        g.declare_region_renown(2, &region_id);
        g.drive().unwrap();
        assert_eq!(g.pending.is_some(), reward);
        if reward {
            choose(&mut g, vec!["accept".into()]);
            pass_top(&mut g);
            assert_eq!(g.regions[2].influence, [1, 0]);
        }
        checkpoint(&g);
    }
}

#[test]
fn entry_search_original_carrier_return_to_hand_and_library_bottom_still_preserves_accepted_frame()
{
    for def in ["BQ104", "XQ48"] {
        for bottom in [false, true] {
            let mut g = game(0);
            let ids = library(&mut g, 0, &["BQ104", "JC125", "XQ36"]);
            let source = trigger(&mut g, def, 2, 0);
            choose(&mut g, vec!["accept".into()]);
            if bottom {
                let (_, c) = g.leave_board(&source).unwrap();
                let c = g.reset_zone_card(c);
                g.players[c.owner].deck.push(c);
            } else {
                g.return_hand(&source);
            }
            checkpoint(&g);
            pass_top(&mut g);
            choose(
                &mut g,
                vec![ids[if def == "BQ104" { 0 } else { 1 }].clone()],
            );
            assert_eq!(g.players[0].deck.len(), 2);
            assert!(g.pending.is_none());
            checkpoint(&g);
        }
    }
}
