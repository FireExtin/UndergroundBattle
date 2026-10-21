//! Prepared layouts followed by real paid Room actions, strict restore and journal replay.
use crate::{catalog, model::*, room::*, rules::*, unit_support as u};
use serde_json::{json, Value};
use std::path::Path;
fn source_bindings() -> &'static Value {
    static BINDINGS: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    BINDINGS.get_or_init(|| {
        use sha2::{Digest, Sha256};
        let files: &[(&str, &[u8])] = &[
            ("Cargo.lock", include_bytes!("../../Cargo.lock")),
            ("Cargo.toml", include_bytes!("../../Cargo.toml")),
            (
                "rust-game-wasm/Cargo.toml",
                include_bytes!("../../rust-game-wasm/Cargo.toml"),
            ),
            (
                "rust-game-wasm/build.sh",
                include_bytes!("../../rust-game-wasm/build.sh"),
            ),
            (
                "rust-game-wasm/src/lib.rs",
                include_bytes!("../../rust-game-wasm/src/lib.rs"),
            ),
            ("rust-game/Cargo.toml", include_bytes!("../Cargo.toml")),
            (
                "rust-game/data/cards.json",
                include_bytes!("../data/cards.json"),
            ),
            (
                "rust-game/src/attachment.rs",
                include_bytes!("attachment.rs"),
            ),
            (
                "rust-game/src/attributes.rs",
                include_bytes!("attributes.rs"),
            ),
            (
                "rust-game/src/bin/hegemony-audit.rs",
                include_bytes!("bin/hegemony-audit.rs"),
            ),
            (
                "rust-game/src/black_expansion.rs",
                include_bytes!("black_expansion.rs"),
            ),
            (
                "rust-game/src/black_expansion_tests.rs",
                include_bytes!("black_expansion_tests.rs"),
            ),
            (
                "rust-game/src/blue_expansion.rs",
                include_bytes!("blue_expansion.rs"),
            ),
            (
                "rust-game/src/blue_expansion_tests.rs",
                include_bytes!("blue_expansion_tests.rs"),
            ),
            (
                "rust-game/src/blue_minimum.rs",
                include_bytes!("blue_minimum.rs"),
            ),
            (
                "rust-game/src/blue_minimum_tests.rs",
                include_bytes!("blue_minimum_tests.rs"),
            ),
            (
                "rust-game/src/bounded_entry_fresh_tests.rs",
                include_bytes!("bounded_entry_fresh_tests.rs"),
            ),
            (
                "rust-game/src/bounded_search.rs",
                include_bytes!("bounded_search.rs"),
            ),
            (
                "rust-game/src/bounded_search_tests.rs",
                include_bytes!("bounded_search_tests.rs"),
            ),
            ("rust-game/src/bq030.rs", include_bytes!("bq030.rs")),
            ("rust-game/src/bq035.rs", include_bytes!("bq035.rs")),
            (
                "rust-game/src/bq035_tests.rs",
                include_bytes!("bq035_tests.rs"),
            ),
            ("rust-game/src/catalog.rs", include_bytes!("catalog.rs")),
            ("rust-game/src/jz46.rs", include_bytes!("jz46.rs")),
            (
                "rust-game/src/jz46_tests.rs",
                include_bytes!("jz46_tests.rs"),
            ),
            ("rust-game/src/control.rs", include_bytes!("control.rs")),
            (
                "rust-game/src/control_tests.rs",
                include_bytes!("control_tests.rs"),
            ),
            (
                "rust-game/src/death_observer_tests.rs",
                include_bytes!("death_observer_tests.rs"),
            ),
            (
                "rust-game/src/death_observers.rs",
                include_bytes!("death_observers.rs"),
            ),
            ("rust-game/src/deck.rs", include_bytes!("deck.rs")),
            (
                "rust-game/src/deck_seal_search.rs",
                include_bytes!("deck_seal_search.rs"),
            ),
            (
                "rust-game/src/deck_seal_search_draft.rs",
                include_bytes!("deck_seal_search_draft.rs"),
            ),
            (
                "rust-game/src/deck_seal_search_tests.rs",
                include_bytes!("deck_seal_search_tests.rs"),
            ),
            (
                "rust-game/src/defence_equipment_tests.rs",
                include_bytes!("defence_equipment_tests.rs"),
            ),
            (
                "rust-game/src/dream_reveal_tests.rs",
                include_bytes!("dream_reveal_tests.rs"),
            ),
            ("rust-game/src/engine.rs", include_bytes!("engine.rs")),
            (
                "rust-game/src/equipment_abilities_tests.rs",
                include_bytes!("equipment_abilities_tests.rs"),
            ),
            (
                "rust-game/src/gray_expansion.rs",
                include_bytes!("gray_expansion.rs"),
            ),
            (
                "rust-game/src/gray_expansion_tests.rs",
                include_bytes!("gray_expansion_tests.rs"),
            ),
            ("rust-game/src/gray_lock.rs", include_bytes!("gray_lock.rs")),
            (
                "rust-game/src/gray_lock_tests.rs",
                include_bytes!("gray_lock_tests.rs"),
            ),
            (
                "rust-game/src/green_minimum_tests.rs",
                include_bytes!("green_minimum_tests.rs"),
            ),
            (
                "rust-game/src/hand_deck_tests.rs",
                include_bytes!("hand_deck_tests.rs"),
            ),
            (
                "rust-game/src/hand_interactions_tests.rs",
                include_bytes!("hand_interactions_tests.rs"),
            ),
            (
                "rust-game/src/jc004_tests.rs",
                include_bytes!("jc004_tests.rs"),
            ),
            (
                "rust-game/src/jc005_tests.rs",
                include_bytes!("jc005_tests.rs"),
            ),
            (
                "rust-game/src/jc008_tests.rs",
                include_bytes!("jc008_tests.rs"),
            ),
            (
                "rust-game/src/jc029_tests.rs",
                include_bytes!("jc029_tests.rs"),
            ),
            (
                "rust-game/src/jc030_tests.rs",
                include_bytes!("jc030_tests.rs"),
            ),
            ("rust-game/src/jc037.rs", include_bytes!("jc037.rs")),
            (
                "rust-game/src/jc037_tests.rs",
                include_bytes!("jc037_tests.rs"),
            ),
            ("rust-game/src/jc046.rs", include_bytes!("jc046.rs")),
            (
                "rust-game/src/jc046_tests.rs",
                include_bytes!("jc046_tests.rs"),
            ),
            ("rust-game/src/jc050.rs", include_bytes!("jc050.rs")),
            (
                "rust-game/src/jc050_tests.rs",
                include_bytes!("jc050_tests.rs"),
            ),
            ("rust-game/src/jc052.rs", include_bytes!("jc052.rs")),
            (
                "rust-game/src/jc052_tests.rs",
                include_bytes!("jc052_tests.rs"),
            ),
            ("rust-game/src/jc060.rs", include_bytes!("jc060.rs")),
            (
                "rust-game/src/jc060_tests.rs",
                include_bytes!("jc060_tests.rs"),
            ),
            (
                "rust-game/src/jc061_tests.rs",
                include_bytes!("jc061_tests.rs"),
            ),
            (
                "rust-game/src/jc087_tests.rs",
                include_bytes!("jc087_tests.rs"),
            ),
            (
                "rust-game/src/jc089_tests.rs",
                include_bytes!("jc089_tests.rs"),
            ),
            (
                "rust-game/src/jc094_tests.rs",
                include_bytes!("jc094_tests.rs"),
            ),
            ("rust-game/src/jc095.rs", include_bytes!("jc095.rs")),
            (
                "rust-game/src/jc095_tests.rs",
                include_bytes!("jc095_tests.rs"),
            ),
            ("rust-game/src/jz22.rs", include_bytes!("jz22.rs")),
            (
                "rust-game/src/jz22_tests.rs",
                include_bytes!("jz22_tests.rs"),
            ),
            (
                "rust-game/src/jz31_tests.rs",
                include_bytes!("jz31_tests.rs"),
            ),
            ("rust-game/src/jz35.rs", include_bytes!("jz35.rs")),
            (
                "rust-game/src/jz35_tests.rs",
                include_bytes!("jz35_tests.rs"),
            ),
            (
                "rust-game/src/jz48_tests.rs",
                include_bytes!("jz48_tests.rs"),
            ),
            (
                "rust-game/src/jz49_tests.rs",
                include_bytes!("jz49_tests.rs"),
            ),
            (
                "rust-game/src/jz50_search.rs",
                include_bytes!("jz50_search.rs"),
            ),
            (
                "rust-game/src/jz50_search_tests.rs",
                include_bytes!("jz50_search_tests.rs"),
            ),
            (
                "rust-game/src/jz55_tests.rs",
                include_bytes!("jz55_tests.rs"),
            ),
            ("rust-game/src/lib.rs", include_bytes!("lib.rs")),
            ("rust-game/src/main.rs", include_bytes!("main.rs")),
            (
                "rust-game/src/mill_public_tests.rs",
                include_bytes!("mill_public_tests.rs"),
            ),
            ("rust-game/src/mobility.rs", include_bytes!("mobility.rs")),
            ("rust-game/src/model.rs", include_bytes!("model.rs")),
            (
                "rust-game/src/msjc01_tests.rs",
                include_bytes!("msjc01_tests.rs"),
            ),
            (
                "rust-game/src/msjc06_tests.rs",
                include_bytes!("msjc06_tests.rs"),
            ),
            (
                "rust-game/src/msjc07_tests.rs",
                include_bytes!("msjc07_tests.rs"),
            ),
            (
                "rust-game/src/msjc08_tests.rs",
                include_bytes!("msjc08_tests.rs"),
            ),
            (
                "rust-game/src/msjc11_tests.rs",
                include_bytes!("msjc11_tests.rs"),
            ),
            (
                "rust-game/src/play_sources.rs",
                include_bytes!("play_sources.rs"),
            ),
            (
                "rust-game/src/protection_tests.rs",
                include_bytes!("protection_tests.rs"),
            ),
            (
                "rust-game/src/purple_tools_tests.rs",
                include_bytes!("purple_tools_tests.rs"),
            ),
            ("rust-game/src/red_time.rs", include_bytes!("red_time.rs")),
            (
                "rust-game/src/red_time_tests.rs",
                include_bytes!("red_time_tests.rs"),
            ),
            (
                "rust-game/src/region_aura_tests.rs",
                include_bytes!("region_aura_tests.rs"),
            ),
            ("rust-game/src/renown.rs", include_bytes!("renown.rs")),
            (
                "rust-game/src/renown_tests.rs",
                include_bytes!("renown_tests.rs"),
            ),
            (
                "rust-game/src/repress_assets_tests.rs",
                include_bytes!("repress_assets_tests.rs"),
            ),
            (
                "rust-game/src/resolution.rs",
                include_bytes!("resolution.rs"),
            ),
            ("rust-game/src/room.rs", include_bytes!("room.rs")),
            ("rust-game/src/rules.rs", include_bytes!("rules.rs")),
            ("rust-game/src/sealing.rs", include_bytes!("sealing.rs")),
            (
                "rust-game/src/sealing_tests.rs",
                include_bytes!("sealing_tests.rs"),
            ),
            ("rust-game/src/service.rs", include_bytes!("service.rs")),
            ("rust-game/src/society.rs", include_bytes!("society.rs")),
            (
                "rust-game/src/society_msjc09_tests.rs",
                include_bytes!("society_msjc09_tests.rs"),
            ),
            (
                "rust-game/src/unit_support.rs",
                include_bytes!("unit_support.rs"),
            ),
            (
                "rust-game/src/white_wound_tests.rs",
                include_bytes!("white_wound_tests.rs"),
            ),
            (
                "rust-game/src/win_flow_tests.rs",
                include_bytes!("win_flow_tests.rs"),
            ),
            ("rust-game/src/wm023.rs", include_bytes!("wm023.rs")),
            (
                "rust-game/src/wm023_tests.rs",
                include_bytes!("wm023_tests.rs"),
            ),
            ("rust-game/src/wm038.rs", include_bytes!("wm038.rs")),
            (
                "rust-game/src/wm038_tests.rs",
                include_bytes!("wm038_tests.rs"),
            ),
            ("rust-game/src/wm056.rs", include_bytes!("wm056.rs")),
            (
                "rust-game/src/wm056_tests.rs",
                include_bytes!("wm056_tests.rs"),
            ),
            ("rust-game/src/world.rs", include_bytes!("world.rs")),
            (
                "rust-game/src/wound_defence_tests.rs",
                include_bytes!("wound_defence_tests.rs"),
            ),
            (
                "rust-game/src/xq18_tests.rs",
                include_bytes!("xq18_tests.rs"),
            ),
            (
                "rust-game/src/xq24_tests.rs",
                include_bytes!("xq24_tests.rs"),
            ),
            ("rust-game/src/xq25.rs", include_bytes!("xq25.rs")),
            (
                "rust-game/src/xq25_tests.rs",
                include_bytes!("xq25_tests.rs"),
            ),
            ("rust-game/src/xq27.rs", include_bytes!("xq27.rs")),
            (
                "rust-game/src/xq27_tests.rs",
                include_bytes!("xq27_tests.rs"),
            ),
            ("rust-game/src/xq37.rs", include_bytes!("xq37.rs")),
            (
                "rust-game/src/xq37_tests.rs",
                include_bytes!("xq37_tests.rs"),
            ),
            (
                "rust-game/src/yellow_search_batch_tests.rs",
                include_bytes!("yellow_search_batch_tests.rs"),
            ),
            ("rust-game/src/wm062.rs", include_bytes!("wm062.rs")),
            (
                "rust-game/src/wm062_tests.rs",
                include_bytes!("wm062_tests.rs"),
            ),
        ];
        let hashes = files
            .iter()
            .map(|(name, bytes)| (*name, format!("{:x}", Sha256::digest(bytes))))
            .collect::<std::collections::BTreeMap<_, _>>();
        json!({"algorithm":"sha256","files":hashes})
    })
}

struct Trace {
    name: &'static str,
    room: RoomEnvelope,
    initial: Value,
    steps: Vec<Value>,
}

fn prepared(actor: usize, definition: &str) -> (Game, String, String) {
    let mut g = crate::jc029_tests::game(actor);
    for r in &mut g.regions {
        r.influence = [0; 2];
    }
    u::fund(&mut g, actor, "WM062", 4);
    let held = u::held(&mut g, "WM062", actor);
    let mut c = g.make_card(definition, actor);
    c.exhausted = actor % 2 == 1;
    let target = c.id.clone();
    g.regions[2].cards.push(c);
    (g, held, target)
}
fn play(t: &mut Trace, actor: usize, source: &str, target: &str) {
    let a = t.legal(actor, "play", source, Some(target), None);
    t.action(actor, a);
}
fn select(t: &mut Trace, actor: usize, destination: Option<usize>) {
    let choice_id = t.room.game.pending.as_ref().unwrap().choice.id.clone();
    t.action(
        actor,
        Action {
            choice_id: Some(choice_id),
            selected: Some(
                destination
                    .map(|r| vec![format!("region:{r}")])
                    .unwrap_or_default(),
            ),
            ..Action::new("choose")
        },
    );
}
fn successor(t: &Trace) -> Card {
    match &t.room.game.pending.as_ref().unwrap().resolution {
        ChoiceResolution::Frame {
            choice: FrameChoice::WM062AdjacentMove { hidden, .. },
            ..
        } => hidden.clone(),
        _ => panic!("WM062 successor choice missing"),
    }
}
#[test]
fn wm062_whole_print_and_closed_fast_definition() {
    let c = catalog::card("WM062");
    assert_eq!(
        (c.kind.as_str(), c.cost, c.color.as_str()),
        ("spell", 2, "黑")
    );
    assert_eq!(c.loyalty, ["黑色"]);
    assert_eq!(c.subtypes, ["阴谋"]);
    assert!(!c.unique);
    assert!(c.magic.is_empty());
    assert_eq!(c.permanent_icons, Icons::default());
    assert_eq!(c.temporary_icons, Icons::default());
    let d = crate::wm062::definition();
    assert_eq!(d.abilities[0].timing, Timing::Fast);
    for variant in 0..13 {
        let mut bad = d.clone();
        let a = &mut bad.abilities[0];
        match variant {
            0 => a.timing = Timing::Standard,
            1 => a.targets[0].relation = Relation::FriendlyTeam,
            2 => a.targets[0].subtype = None,
            3 => a.targets[0].kind = EntityKind::CharacterOrHidden,
            4 => a.targets[0].range = Range::SourceRegion,
            5 => a.play_only = false,
            6 => a.response_policy = ResponsePolicy::Immediate,
            7 => a.costs.push(Cost::ExhaustSource),
            8 => a.ops.clear(),
            9 => a.ops.push(Op::Draw {
                player: PlayerRef::Actor,
                count: 1,
                end: DeckEnd::Top,
            }),
            10 => a.event = Some(Event::Reveal),
            11 => a.per_turn_limit = Some(1),
            _ => a.targets[0].max = 2,
        }
        assert!(
            crate::wm062::validate_definition("WM062", &bad).is_err(),
            "{variant}"
        );
        assert!(
            crate::rules::validate_ability("WM062", &bad.abilities[0]).is_err(),
            "{variant}"
        );
    }
    for id in ["JC063", "JC087", "WM056"] {
        assert!(crate::wm062::validate_definition(id, &d).is_err());
        assert!(crate::rules::validate_ability(id, &d.abilities[0]).is_err());
    }
}
#[test]
fn wm062_paid_move_preserves_actual_successor_and_does_not_trigger_hidden_front_all_seats() {
    for (actor, name) in [
        (0, "move-seat0"),
        (1, "move-seat1"),
        (2, "move-seat2"),
        (3, "move-seat3"),
    ] {
        let (mut g, held, target) = prepared(actor, "JC087");
        // A same-print decoy must never be substituted for the chosen identity.
        let decoy = g.make_card("JC087", actor);
        let decoy_id = decoy.id.clone();
        g.regions[2].cards.push(decoy);
        let mut t = Trace::new(name, g);
        let assets = t.room.game.players[actor]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count();
        play(&mut t, actor, &held, &target);
        assert_eq!(
            t.room.game.players[actor]
                .assets
                .iter()
                .filter(|c| c.exhausted)
                .count(),
            assets + 2
        );
        assert!(t.room.game.board(&target).is_some());
        t.finish();
        let h = successor(&t);
        assert_ne!(h.id, target);
        assert!(h.face_down);
        assert_eq!(h.exhausted, actor % 2 == 1);
        for seat in 0..4 {
            let view = t.room.game.view(seat);
            assert_eq!(view.pending_choice.is_some(), seat == actor);
            let hidden = view.regions[2]
                .characters
                .iter()
                .find(|c| c.instance_id == h.id)
                .unwrap();
            assert_eq!(hidden.card_id.is_some(), seat == actor);
        }
        // Nonadjacent slot and a foreign seat are strict no-op rejections.
        let choice = t.room.game.pending.as_ref().unwrap().choice.id.clone();
        t.command(
            (actor + 2) % 4,
            SessionAction::Game {
                action: Action {
                    choice_id: Some(choice.clone()),
                    selected: Some(vec!["region:1".into()]),
                    ..Action::new("choose")
                },
            },
            true,
        );
        t.command(
            actor,
            SessionAction::Game {
                action: Action {
                    choice_id: Some(choice.clone()),
                    selected: Some(vec!["region:4".into()]),
                    ..Action::new("choose")
                },
            },
            true,
        );
        select(&mut t, actor, Some(1));
        assert!(t.room.game.pending.is_none());
        assert!(t.room.game.stack.is_empty());
        assert_eq!(t.room.game.board(&h.id).unwrap().0, 1);
        assert_eq!(t.room.game.board(&decoy_id).unwrap().0, 2);
        assert!(t.room.game.board(&target).is_none());
        assert_eq!(
            t.room.game.players[actor]
                .graveyard
                .iter()
                .filter(|c| c.definition == "WM062")
                .count(),
            1
        );
        t.command(
            actor,
            SessionAction::Game {
                action: Action {
                    choice_id: Some(choice),
                    selected: Some(vec!["region:1".into()]),
                    ..Action::new("choose")
                },
            },
            true,
        );
        t.export();
    }
}
#[test]
fn wm062_decline_keeps_conceal_and_cleans_attachments_seals_markers_without_repaying() {
    let (mut g, held, target) = prepared(0, "JC094");
    let c = g.board_mut(&target).unwrap();
    c.damage = 1;
    c.wounds = 1;
    c.shield = 1;
    c.time_markers = 2;
    c.lock_markers = 3;
    c.exhausted = true;
    let attached = g.make_card("XQ47", 0);
    g.attachments.push(Attachment {
        card: attached,
        host_id: target.clone(),
    });
    let mut t = Trace::new("decline-cleanup", g);
    play(&mut t, 0, &held, &target);
    t.finish();
    let h = successor(&t);
    assert!(h.exhausted);
    assert_eq!(
        (h.damage, h.wounds, h.shield, h.time_markers, h.lock_markers),
        (0, 0, 0, 0, 0)
    );
    assert!(t.room.game.attachments.is_empty());
    let exhausted = t.room.game.players[0]
        .assets
        .iter()
        .filter(|c| c.exhausted)
        .count();
    select(&mut t, 0, None);
    assert_eq!(t.room.game.board(&h.id).unwrap().0, 2);
    assert_eq!(
        t.room.game.players[0]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        exhausted
    );
    assert!(t.room.game.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ47"));
    t.export();
}
#[test]
fn wm062_controlled_foreign_criminal_returns_controller_but_actor_keeps_optional_move() {
    let (mut g, held, original) = prepared(0, "JC087");
    g.board_mut(&original).unwrap().owner = 1;
    g.board_mut(&original).unwrap().controller = 1;
    let embrace = g.make_card("JC036", 0);
    let attached_id = embrace.id.clone();
    g.attachments.push(Attachment {
        card: embrace,
        host_id: original.clone(),
    });
    g.add_control(
        &original,
        0,
        ControlLifetime::Attached {
            source_instance: attached_id,
        },
        SubtypeChange::HumanToVampire,
    );
    g.settle_controls();
    let mut t = Trace::new("controlled-foreign-successor", g);
    play(&mut t, 0, &held, &original);
    t.finish();
    let h = successor(&t);
    assert_eq!((h.owner, h.controller), (1, 1));
    assert!(t.room.game.view(0).pending_choice.is_some());
    assert!(t.room.game.view(1).pending_choice.is_none());
    let hidden = t.room.game.view(0).regions[2]
        .characters
        .iter()
        .find(|c| c.instance_id == h.id)
        .unwrap()
        .clone();
    assert!(hidden.card_id.is_none());
    select(&mut t, 0, Some(3));
    assert_eq!(t.room.game.board(&h.id).unwrap().0, 3);
    assert!(t.room.game.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC036"));
    t.export();
}
#[test]
fn wm062_real_police_response_retires_target_and_cancels_whole_paid_transaction() {
    let (mut g, held, target) = prepared(0, "JC087");
    u::fund(&mut g, 1, "JC063", 2);
    let police = u::held(&mut g, "JC063", 1);
    let mut t = Trace::new("target-concealed-response", g);
    play(&mut t, 0, &held, &target);
    let a = t
        .room
        .game
        .legal_actions(1)
        .into_iter()
        .find(|a| {
            a.action.card_id.as_deref() == Some(&police)
                && a.action.option.as_deref() == Some("hide")
                && a.action.target_id.as_deref() == Some(&target)
        })
        .unwrap()
        .action;
    t.action(1, a);
    t.finish();
    assert!(t.room.game.pending.is_none());
    assert!(t.room.game.board(&target).is_none());
    assert_eq!(
        t.room.game.regions[2]
            .cards
            .iter()
            .filter(|c| c.face_down)
            .count(),
        1
    );
    assert!(t.room.game.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "WM062"));
    t.export();
}
#[test]
fn wm062_real_bounce_response_cancels_without_concealing_a_substitute() {
    let (mut g, held, target) = prepared(0, "JZ24");
    u::fund(&mut g, 1, "JC006", 2);
    let bounce = u::held(&mut g, "JC006", 1);
    let decoy = g.make_card("JZ24", 0);
    let decoy_id = decoy.id.clone();
    g.regions[1].cards.push(decoy);
    let mut t = Trace::new("target-bounced-response", g);
    play(&mut t, 0, &held, &target);
    let a = t.legal(1, "play", &bounce, Some(&target), None);
    t.action(1, a);
    t.finish();
    assert!(t.room.game.pending.is_none());
    assert!(t.room.game.board(&target).is_none());
    assert!(!t.room.game.board(&decoy_id).unwrap().1.face_down);
    assert!(t.room.game.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "JZ24"));
    t.export();
}
#[test]
fn wm062_pause_resume_preserves_paid_pending_successor_and_rejects_moves_while_paused() {
    let (g, held, target) = prepared(0, "JC087");
    let mut t = Trace::new("pause-successor-resume", g);
    play(&mut t, 0, &held, &target);
    t.finish();
    let h = successor(&t);
    let choice = t.room.game.pending.as_ref().unwrap().choice.id.clone();
    t.command(0, SessionAction::PauseRoom, false);
    t.command(
        0,
        SessionAction::Game {
            action: Action {
                choice_id: Some(choice.clone()),
                selected: Some(vec!["region:3".into()]),
                ..Action::new("choose")
            },
        },
        true,
    );
    t.command(0, SessionAction::ResumeRoom, false);
    assert_eq!(t.room.game.pending.as_ref().unwrap().choice.id, choice);
    assert_eq!(successor(&t).id, h.id);
    select(&mut t, 0, Some(3));
    assert_eq!(t.room.game.board(&h.id).unwrap().0, 3);
    t.export();
}
#[test]
fn wm062_fast_in_end_window_and_no_destination_still_completes_conceal() {
    let (mut g, held, target) = prepared(0, "JC087");
    g.begin_window(Window::End);
    let mut t = Trace::new("fast-end-window", g);
    play(&mut t, 0, &held, &target);
    t.finish();
    select(&mut t, 0, None);
    t.export();
    let (mut g, held, target) = prepared(0, "JC087");
    g.world.clear();
    for (r, v) in g.regions.iter_mut().enumerate() {
        if r != 2 {
            v.vacant = true;
            v.influence = [0; 2];
            v.skip = false;
        }
    }
    let mut t = Trace::new("no-adjacent-destination", g);
    play(&mut t, 0, &held, &target);
    t.finish();
    assert!(t.room.game.pending.is_none());
    assert!(t.room.game.board(&target).is_none());
    assert!(t.room.game.regions[2].cards[0].face_down);
    t.export();
}
#[test]
fn wm062_rejects_teammate_enemy_hidden_noncriminal_and_premature_destination() {
    let (mut g, held, target) = prepared(0, "JC087");
    let mut invalid = vec![];
    for (id, owner, hidden) in [
        ("JC087", 1, false),
        ("JC087", 2, false),
        ("JC087", 0, true),
        ("JC125", 0, false),
    ] {
        let mut c = g.make_card(id, owner);
        c.face_down = hidden;
        invalid.push(c.id.clone());
        g.regions[2].cards.push(c);
    }
    let mut t = Trace::new("invalid-target-and-payload", g);
    for id in invalid {
        t.command(
            0,
            SessionAction::Game {
                action: Action {
                    card_id: Some(held.clone()),
                    target_id: Some(id),
                    ..Action::new("play")
                },
            },
            true,
        );
    }
    t.command(
        0,
        SessionAction::Game {
            action: Action {
                card_id: Some(held.clone()),
                target_id: Some(target.clone()),
                region: Some(1),
                ..Action::new("play")
            },
        },
        true,
    );
    play(&mut t, 0, &held, &target);
    t.finish();
    select(&mut t, 0, None);
    t.export();
}
#[test]
fn wm062_duel_uses_existing_adjacency_and_preserves_exhaustion() {
    let mut g = Game::new(
        "duel".into(),
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
    }
    for r in &mut g.regions {
        r.cards.clear();
        r.influence = [0; 2];
    }
    g.begin_window(Window::Action(0));
    u::fund(&mut g, 0, "WM062", 2);
    let held = u::held(&mut g, "WM062", 0);
    let mut c = g.make_card("JC087", 0);
    c.exhausted = true;
    let target = c.id.clone();
    g.regions[0].cards.push(c);
    let mut t = Trace::new("duel-adjacent-move", g);
    play(&mut t, 0, &held, &target);
    t.finish();
    let h = successor(&t);
    select(&mut t, 0, Some(2));
    assert_eq!(t.room.game.board(&h.id).unwrap().0, 2);
    assert!(t.room.game.board(&h.id).unwrap().1.exhausted);
    t.export();
}

#[test]
fn wm062_last_other_criminal_defense_loss_settles_before_optional_move() {
    let (mut g, held, target) = prepared(0, "JC087");
    let mut robber = g.make_card("JZ48", 0);
    robber.damage = 1;
    let robber_id = robber.id.clone();
    g.regions[2].cards.push(robber);
    assert_eq!(g.defense(g.board(&robber_id).unwrap().1, 2), 2);
    let mut t = Trace::new("last-criminal-defense-loss", g);
    play(&mut t, 0, &held, &target);
    t.finish();
    let hidden = successor(&t);
    assert!(
        t.room.game.board(&robber_id).is_none(),
        "潜伏已完成，失去防御加成的致命角色不能留到可选移动后才清理"
    );
    assert!(t.room.game.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JZ48"));
    select(&mut t, 0, Some(1));
    assert_eq!(t.room.game.board(&hidden.id).unwrap().0, 1);
    t.export();
}

fn rejected_input(name: String, value: Value) {
    let state = serde_json::to_string(&value).unwrap();
    let game_state = serde_json::to_string(&value["game"]).unwrap();
    let game_error = Game::from_persisted(&game_state)
        .err()
        .expect("bad WM062 Game accepted");
    let room_error = RoomEnvelope::from_persisted(&state)
        .err()
        .expect("bad WM062 Room accepted");
    if let Ok(directory) = std::env::var("WM062_NEGATIVE_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        let bytes = serde_json::to_vec(&json!({"state":state,"gameState":game_state,
            "gameError":game_error,"roomError":room_error,"sourceBindings":source_bindings()}))
        .unwrap();
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(Path::new(&directory).join(format!("{name}.json")))
            .unwrap()
            .write_all(&bytes)
            .unwrap();
    }
}
#[test]
fn wm062_bounded_restore_rejects_wrong_successor_region_program_and_paid_container() {
    let (g, held, target) = prepared(0, "JC087");
    let mut t = Trace::new("restore-seed-not-exported", g);
    play(&mut t, 0, &held, &target);
    let response = serde_json::to_value(&t.room).unwrap();
    for variant in 0..9 {
        let mut bad = response.clone();
        let s = &mut bad["game"]["stack"][0];
        match variant {
            0 => s["card"]["exhausted"] = json!(true),
            1 => s["card"]["owner"] = json!(1),
            2 => s["frame"]["actor"] = json!(1),
            3 => s["frame"]["cursor"] = json!(1),
            4 => s["frame"]["targets"][0]["spec"]["relation"] = json!("FriendlyTeam"),
            5 => s["frame"]["steps"][0]["context"] = json!(1),
            6 => s["frame"] = Value::Null,
            7 => s["target"] = json!("substitute"),
            _ => s["frame"]["source"]["card"]["definition"] = json!("JC063"),
        }
        rejected_input(format!("response-{variant:02}"), bad);
    }
    t.finish();
    let pending = serde_json::to_value(&t.room).unwrap();
    for variant in 0..36 {
        let mut bad = pending.clone();
        let f = &mut bad["game"]["pending"]["resolution"]["Frame"]["frame"];
        match variant {
            0 => f["actor"] = json!(1),
            1 => f["steps"] = json!([]),
            2 => f["cursor"] = json!(0),
            3 => f["guard"] = json!("Unchecked"),
            4 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["hidden"]["controller"] = json!(1)
            }
            5 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["hidden"]["exhausted"] = json!(true)
            }
            6 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["hidden"]["owner"] = json!(1)
            }
            7 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["hidden"]["id"] = json!(target)
            }
            8 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["before"]["id"] = json!("other-old-instance")
            }
            9 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["before"]["definition"] = json!("JC084")
            }
            10 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["before"]["face_down"] = json!(true)
            }
            11 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["baseline"] = json!({"target_instance":target,"controller":4})
            }
            12 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["region"] = json!(1)
            }
            13 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["region_instance"] = json!("replacement-region")
            }
            14 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["destinations"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            15 => {
                bad["game"]["pending"]["resolution"]["Frame"]["choice"]["WM062AdjacentMove"]
                    ["destinations"][0][1] = json!("replacement-destination")
            }
            16 => bad["game"]["pending"]["choice"]["options"][0]["card"] = json!({}),
            17 => bad["game"]["pending"]["choice"]["min"] = json!(1),
            18 => bad["game"]["pending"]["choice"]["max"] = json!(2),
            19 => bad["game"]["pending"]["seat"] = json!(1),
            20 => bad["game"]["pending"]["choice"]["playerId"] = json!("p1"),
            21 => bad["game"]["pending"]["choice"]["kind"] = json!("other-choice"),
            22 => bad["game"]["pending"]["choice"]["amount"] = json!(1),
            23 => bad["game"]["pending"]["choice"]["allowDecline"] = json!(false),
            24 => bad["game"]["pending"]["choice"]["previewCards"] = json!([{}]),
            25 => bad["game"]["effects"][0]["Bury"]["card"]["exhausted"] = json!(true),
            26 => bad["game"]["effects"][0]["Bury"]["card"]["owner"] = json!(1),
            27 => {
                let f = f.clone();
                bad["game"]["effects"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"Frame":{"frame":f}}));
            }
            28 => {
                let mut s = response["game"]["stack"][0].clone();
                s["frame"] = f.clone();
                bad["game"]["stack"].as_array_mut().unwrap().push(s);
            }
            29 => f["source"]["card"]["definition"] = json!("JC063"),
            30 => f["steps"]
                .as_array_mut()
                .unwrap()
                .push(json!({"context":0,"op":{"Draw":{"player":"Actor","count":1,"end":"Top"}}})),
            31 => bad["game"]["regions"][2]["card"]["id"] = json!("replaced-origin"),
            32 => {
                let before = bad["game"]["pending"]["resolution"]["Frame"]["choice"]
                    ["WM062AdjacentMove"]["before"]
                    .clone();
                bad["game"]["regions"][2]["cards"]
                    .as_array_mut()
                    .unwrap()
                    .push(before);
            }
            33 => {
                let c = bad["game"]["regions"][2]["cards"]
                    .as_array_mut()
                    .unwrap()
                    .pop()
                    .unwrap();
                bad["game"]["regions"][1]["cards"]
                    .as_array_mut()
                    .unwrap()
                    .push(c);
            }
            34 => bad["game"]["pending"]["choice"]["description"] = json!("old-description"),
            _ => bad["game"]["effects"] = json!([]),
        }
        rejected_input(format!("pending-{variant:02}"), bad);
    }
}

impl Trace {
    fn new(name: &'static str, mut g: Game) -> Self {
        g.room_id = "620000000000000000000001".into();
        let room = crate::jc029_tests::envelope(&g);
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
            command_id: format!("wm062-{}-{}", self.name, self.steps.len()),
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
            "{} action {:?}: {:?}",
            self.name,
            command.action,
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
            let intent_id = format!("wm062-intent-{}", self.steps.len());
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
        if let Ok(dir) = std::env::var("WM062_NATIVE_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            let bytes=serde_json::to_vec(&json!({"name":self.name,"scope":"Explicit prepared Native layout; continuous real Room commands, strict restore and journal replay; not browser play or migration","catalog":catalog::catalog(),"sourceBindings":source_bindings(),"initial":self.initial,"steps":self.steps})).unwrap();
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
