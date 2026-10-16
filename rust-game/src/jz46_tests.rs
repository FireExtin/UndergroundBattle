//! Explicit offline layouts, followed by real paid Game/Room actions; not online gameplay.
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
            (
                "rust-game/src/bq030_preparation.rs",
                include_bytes!("bq030_preparation.rs"),
            ),
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

fn base(actor: usize) -> Game {
    let mut g = crate::jc029_tests::game(actor);
    for r in &mut g.regions {
        r.influence = [0; 2];
    }
    g
}
fn prepared(actor: usize, criminal: bool, target_seat: usize) -> (Game, String, String) {
    let mut g = base(actor);
    u::fund(&mut g, actor, "JZ46", 4);
    let held = u::held(&mut g, "JZ46", actor);
    let c = g.make_card(if criminal { "JC087" } else { "JC125" }, target_seat);
    let target = c.id.clone();
    let mut c = c;
    c.face_down = true;
    g.regions[2].cards.push(c);
    (g, held, target)
}
fn attach(t: &mut Trace, actor: usize, held: &str) -> String {
    let a = t.legal(actor, "play", held, None, Some(2));
    t.action(actor, a);
    t.finish();
    t.room
        .game
        .attachments
        .iter()
        .find(|a| a.card.definition == "JZ46")
        .unwrap()
        .card
        .id
        .clone()
}
fn inspect(t: &mut Trace, actor: usize, source: &str, target: &str) {
    let actions = t.room.game.legal_actions(actor);
    let a = actions
        .into_iter()
        .find(|a| {
            a.action.kind == "activate"
                && a.action.card_id.as_deref() == Some(source)
                && a.action.target_id.as_deref() == Some(target)
        })
        .unwrap()
        .action;
    t.action(actor, a);
    t.finish();
    assert_eq!(
        t.room.game.pending.as_ref().unwrap().choice.kind,
        "jz46-hidden-inspect"
    );
}
fn confirm(t: &mut Trace, actor: usize) {
    let id = t.room.game.pending.as_ref().unwrap().choice.id.clone();
    t.action(
        actor,
        Action {
            choice_id: Some(id),
            selected: Some(vec![]),
            ..Action::new("choose")
        },
    );
}
fn pass_one(t: &mut Trace) {
    if let Some(p) = t.room.game.pending.clone() {
        let selected = p
            .choice
            .options
            .iter()
            .take(p.choice.min.unwrap_or(1))
            .map(|o| o.id.clone())
            .collect();
        t.action(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(selected),
                ..Action::new("choose")
            },
        );
    } else if let Some(w) = t.room.pacing.window.clone() {
        let seat = *w
            .members
            .iter()
            .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
            .unwrap()
            .0;
        t.command(seat, SessionAction::PassResponse { window_id: w.id }, false);
    } else {
        let seat = (0..4)
            .find(|s| {
                t.room
                    .game
                    .legal_actions(*s)
                    .iter()
                    .any(|a| a.action.kind == "pass")
            })
            .unwrap();
        t.command(
            seat,
            SessionAction::Game {
                action: Action::new("pass"),
            },
            false,
        );
    }
}
#[test]
fn jz46_print_and_definition_are_complete_and_cannot_be_transplanted() {
    let c = catalog::card("JZ46");
    assert_eq!(
        (c.cost, c.color.as_str(), c.magic.as_str()),
        (2, "灰", "神圣")
    );
    assert_eq!(c.subtypes, ["雕像"]);
    assert_eq!(c.permanent_icons.influence, 1);
    assert_eq!(c.temporary_icons, Icons::default());
    assert_eq!(c.loyalty, ["灰色"]);
    let expected = crate::jz46::definition();
    assert_eq!(expected.abilities[1].timing, Timing::Standard);
    for variant in 0..16 {
        let mut d = expected.clone();
        let a = &mut d.abilities[1];
        match variant {
            0 => {
                d.abilities.pop();
            }
            1 => a.costs.clear(),
            2 => a.timing = Timing::Fast,
            3 => a.targets[0].relation = Relation::EnemyTeam,
            4 => a.targets[0].subtype = Some("罪犯".into()),
            5 => a.targets[0].range = Range::Anywhere,
            6 => a.targets[0].kind = EntityKind::CharacterOrHidden,
            7 => a.ops.push(Op::Draw {
                player: PlayerRef::Actor,
                count: 1,
                end: DeckEnd::Top,
            }),
            8 => d.attachment.as_mut().unwrap().host.zone = Zone::Board,
            9 => a.per_turn_limit = Some(1),
            10 => a.response_policy = ResponsePolicy::Immediate,
            11 => a.event = Some(Event::Reveal),
            12 => a.play_only = true,
            13 => d.attachment.as_mut().unwrap().host_icons.influence = 1,
            14 => a.targets[0].max = 2,
            _ => d.modifiers.push(StaticModifier::PeekOwnDeckTop),
        }
        assert!(
            crate::jz46::validate_definition("JZ46", &d).is_err(),
            "variant {variant}"
        );
    }
    for id in ["JC125", "JC069", "XQ25"] {
        assert!(crate::jz46::validate_definition(id, &expected).is_err());
        assert!(crate::rules::validate_ability(id, &expected.abilities[1]).is_err());
    }
}
#[test]
fn jz46_paid_private_criminal_and_noncriminal_all_four_actors() {
    for (actor, name) in [
        (0, "criminal-seat0"),
        (1, "criminal-seat1"),
        (2, "criminal-seat2"),
        (3, "criminal-seat3"),
    ] {
        let (g, held, target) = prepared(actor, true, (actor + 2) % 4);
        let mut t = Trace::new(name, g);
        let source = attach(&mut t, actor, &held);
        assert_eq!(t.room.game.resources(actor), 2);
        assert_eq!(
            t.room.game.contest_counts(2, 2),
            [1, 1],
            "the region attachment contributes its own printed influence"
        );
        inspect(&mut t, actor, &source, &target);
        let (region, live) = t.room.game.board(&source).unwrap();
        assert_eq!(t.room.game.current_icons(live, region).influence, 1);
        assert_eq!(t.room.game.icons(live, region), Icons::default());
        assert_eq!(t.room.game.current_icons(live, 1), Icons::default());
        assert_eq!(t.room.game.contest_counts(2, 2)[t.room.game.team(actor)], 0);
        for seat in 0..4 {
            let v = t.room.game.view(seat);
            if seat == actor {
                let p = v.pending_choice.unwrap();
                assert_eq!(p.preview_cards.len(), 1);
                assert_eq!(p.preview_cards[0].card_id.as_deref(), Some("JC087"));
                assert_eq!(p.max, Some(0));
                assert!(p.options.is_empty());
            } else {
                assert!(v.pending_choice.is_none());
            }
            if seat != (actor + 2) % 4 {
                let c = v.regions[2]
                    .characters
                    .iter()
                    .find(|c| c.instance_id == target)
                    .unwrap();
                assert!(c.card_id.is_none());
                assert_eq!(c.name, "暗藏者");
            }
            assert!(!serde_json::to_string(&v.waiting_choice)
                .unwrap()
                .contains("走私客"));
        }
        let before = t.room.game.board(&target).unwrap().1;
        assert!(!before.exhausted);
        assert_eq!(before.lock_markers, 0);
        let choice = t.room.game.pending.as_ref().unwrap().choice.id.clone();
        t.command(
            (actor + 1) % 4,
            SessionAction::Game {
                action: Action {
                    choice_id: Some(choice.clone()),
                    selected: Some(vec![]),
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
                    region: Some(2),
                    selected: Some(vec![]),
                    ..Action::new("choose")
                },
            },
            true,
        );
        confirm(&mut t, actor);
        let c = t.room.game.board(&target).unwrap().1;
        assert!(c.face_down && c.exhausted);
        assert_eq!(c.lock_markers, 1);
        assert_eq!(t.room.game.contest_counts(2, 2), [0, 0]);
        assert!(t.room.game.pending.is_none());
        t.command(
            actor,
            SessionAction::Game {
                action: Action {
                    choice_id: Some(choice),
                    selected: Some(vec![]),
                    ..Action::new("choose")
                },
            },
            true,
        );
        assert_eq!(t.room.game.board(&target).unwrap().1.lock_markers, 1);
        assert!(!t
            .room
            .game
            .legal_actions(actor)
            .iter()
            .any(|a| a.action.card_id.as_deref() == Some(&source) && a.action.kind == "activate"));
        t.export();
    }
    let (g, held, target) = prepared(0, false, 1);
    let mut t = Trace::new("noncriminal-unchanged", g);
    let source = attach(&mut t, 0, &held);
    inspect(&mut t, 0, &source, &target);
    confirm(&mut t, 0);
    let c = t.room.game.board(&target).unwrap().1;
    assert!(c.face_down && !c.exhausted);
    assert_eq!(c.lock_markers, 0);
    t.export();
}
#[test]
fn jz46_attachment_alone_participates_and_its_controller_receives_influence_reward() {
    let mut g = base(1);
    u::fund(&mut g, 1, "JZ46", 4);
    let held = u::held(&mut g, "JZ46", 1);
    g.players[1]
        .hand
        .iter_mut()
        .find(|c| c.id == held)
        .unwrap()
        .owner = 3;
    let mut t = Trace::new("attachment-alone-influence-reward", g);
    let source = attach(&mut t, 1, &held);
    let (_, live) = t.room.game.board(&source).unwrap();
    assert_eq!((live.owner, live.controller), (3, 1));
    assert!(t.room.game.regions[2].cards.is_empty());
    assert_eq!(t.room.game.contest_counts(2, 0), [0, 0]);
    assert_eq!(t.room.game.contest_counts(2, 1), [0, 0]);
    assert_eq!(t.room.game.contest_counts(2, 2), [1, 0]);
    assert_eq!(t.room.game.contributors(0, 2, 2), vec![1]);
    assert!(t.room.game.contributors(1, 2, 2).is_empty());
    let mut stale = live.clone();
    stale.controller = 0;
    assert_eq!(t.room.game.current_icons(&stale, 2), Icons::default());
    for _ in 0..250 {
        if t.room.game.regions[2].influence[0] > 0 {
            break;
        }
        pass_one(&mut t);
    }
    assert_eq!(t.room.game.regions[2].influence, [1, 0]);
    assert_eq!(t.room.game.contributors(0, 2, 2), vec![1]);
    t.export();
}
#[test]
fn jz46_target_legality_has_no_criminal_or_enemy_filter_and_costs_are_atomic() {
    let (mut g, held, target) = prepared(0, true, 0);
    let other = crate::jc029_tests::board(&mut g, "JC125", 1, 2);
    g.board_mut(&other).unwrap().face_down = true;
    let distant = crate::jc029_tests::board(&mut g, "JC087", 2, 1);
    g.board_mut(&distant).unwrap().face_down = true;
    let faceup = crate::jc029_tests::board(&mut g, "JC087", 3, 2);
    let mut t = Trace::new("all-hidden-no-identity-leak", g);
    let source = attach(&mut t, 0, &held);
    let a = |id: &str| Action {
        card_id: Some(source.clone()),
        ability_id: Some("jz46-inspect-hidden".into()),
        target_id: Some(id.into()),
        ..Action::new("activate")
    };
    let offered = t
        .room
        .game
        .legal_actions(0)
        .iter()
        .filter(|x| x.action.kind == "activate" && x.action.card_id.as_deref() == Some(&source))
        .map(|x| x.action.target_id.clone().unwrap())
        .collect::<Vec<_>>();
    assert!(offered.contains(&target) && offered.contains(&other));
    assert!(!offered.contains(&distant) && !offered.contains(&faceup));
    let mut extra = a(&target);
    extra.option = Some("skip-inspection".into());
    t.command(0, SessionAction::Game { action: extra }, true);
    for id in [&distant, &faceup] {
        t.command(0, SessionAction::Game { action: a(id) }, true);
        assert!(
            !t.room
                .game
                .attachments
                .iter()
                .find(|x| x.card.id == source)
                .unwrap()
                .card
                .exhausted
        );
    }
    inspect(&mut t, 0, &source, &target);
    confirm(&mut t, 0);
    t.export();
}
#[test]
fn jz46_pending_and_response_restore_mutations_fail_in_game_and_room() {
    let (g, held, target) = prepared(0, true, 2);
    let mut t = Trace::new("restore-guards", g);
    let source = attach(&mut t, 0, &held);
    let a = t.legal(0, "activate", &source, Some(&target), None);
    t.action(0, a);
    let response = serde_json::to_value(&t.room.game).unwrap();
    t.finish();
    let pending = serde_json::to_value(&t.room.game).unwrap();
    let bad = |name: &str, value: Value| {
        let game_state = serde_json::to_string(&value).unwrap();
        let game_error = Game::from_persisted(&game_state).unwrap_err();
        let mut room = serde_json::to_value(&t.room).unwrap();
        room["game"] = value;
        let state = serde_json::to_string(&room).unwrap();
        let room_error = RoomEnvelope::from_persisted(&state).unwrap_err();
        if let Ok(dir) = std::env::var("JZ46_INVALID_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(Path::new(&dir).join(format!("{name}.json")),serde_json::to_vec(&json!({"state":state,"gameState":game_state,"gameError":game_error,"roomError":room_error,"sourceBindings":source_bindings()})).unwrap()).unwrap();
        }
    };
    for n in 0..22 {
        let mut v = pending.clone();
        let r = &mut v["pending"]["resolution"]["Frame"];
        match n {
            0 => r["frame"]["actor"] = json!(2),
            1 => r["frame"]["already_paid"] = json!([]),
            2 => r["frame"]["cursor"] = json!(0),
            3 => r["frame"]["guard"] = json!("Unchecked"),
            4 => r["frame"]["source"]["source_region_instance"] = json!("other"),
            5 => r["frame"]["source"]["card"]["definition"] = json!("XQ25"),
            6 => r["frame"]["steps"][0]["op"] = json!({"Exhaust":"Source"}),
            7 => r["choice"]["JZ46InspectHidden"]["inspected"]["definition"] = json!("JC125"),
            8 => r["choice"]["JZ46InspectHidden"]["inspected"]["face_down"] = json!(false),
            9 => v["pending"]["seat"] = json!(1),
            10 => v["pending"]["choice"]["kind"] = json!("other"),
            11 => v["pending"]["choice"]["min"] = json!(1),
            12 => v["pending"]["choice"]["options"] = json!([{"id":"leak","label":"秘密"}]),
            13 => {
                v["pending"]["choice"]["previewCards"] =
                    serde_json::to_value(t.room.game.view(0).pending_choice.unwrap().preview_cards)
                        .unwrap()
            }
            14 => v["pending"]["choice"]["allowDecline"] = json!(true),
            15 => r["frame"]["targets"][0]["spec"]["relation"] = json!("EnemyTeam"),
            16 => r["frame"]["targets"][0]["public"]["kind"] = json!("character"),
            17 => r["frame"]["source"]["attachment_host_instance"] = json!("other"),
            18 => r["frame"]["targets"][0]["public"]["label"] = json!("秘密罪犯"),
            19 => r["frame"]["targets"][0]["public"]["owner"] = json!("p99"),
            20 => r["frame"]["targets"][0]["public"]["controller"] = json!("p99"),
            _ => r["frame"]["targets"][0]["public"]["status"] = json!("cancelled"),
        };
        bad(&format!("pending-{n:02}"), v);
    }
    let mut effect = pending.clone();
    effect["effects"] =
        json!([{ "Frame": {"frame":effect["pending"]["resolution"]["Frame"]["frame"].clone()} }]);
    effect["pending"] = Value::Null;
    bad("effect-transplant", effect);
    let mut duplicate = pending.clone();
    duplicate["stack"] = response["stack"].clone();
    bad("duplicate-pending-stack", duplicate);
    for (name, definition) in [
        ("unknown-front", "NOT_ADMITTED"),
        ("attachment-front", "JZ46"),
    ] {
        let mut v = pending.clone();
        v["pending"]["resolution"]["Frame"]["choice"]["JZ46InspectHidden"]["inspected"]
            ["definition"] = json!(definition);
        for c in v["regions"][2]["cards"].as_array_mut().unwrap() {
            if c["id"] == target {
                c["definition"] = json!(definition);
            }
        }
        bad(name, v);
    }
    for n in 0..8 {
        let mut v = response.clone();
        let f = &mut v["stack"][0]["frame"];
        match n {
            0 => f["cursor"] = json!(1),
            1 => f["guard"] = json!("Accepted"),
            2 => f["already_paid"] = json!([]),
            3 => f["source"]["card"]["controller"] = json!(1),
            4 => f["targets"][0]["spec"]["subtype"] = json!("罪犯"),
            5 => f["source"]["attachment_host_instance"] = json!("wrong"),
            6 => v["stack"][0]["controller"] = json!(1),
            _ => f["steps"][0]["context"] = json!(1),
        };
        bad(&format!("response-{n:02}"), v);
    }
}

#[test]
fn jz46_real_paid_reveal_response_invalidates_the_original_hidden_target() {
    let (g, held, target) = prepared(0, false, 2);
    let mut t = Trace::new("target-paid-reveal-cancels", g);
    let source = attach(&mut t, 0, &held);
    let a = t.legal(0, "activate", &source, Some(&target), None);
    t.action(0, a);
    while t.room.pacing.window.as_ref().unwrap().holder_team != t.room.game.team(2) {
        let w = t.room.pacing.window.clone().unwrap();
        let seat = *w
            .members
            .iter()
            .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
            .unwrap()
            .0;
        t.command(seat, SessionAction::PassResponse { window_id: w.id }, false);
    }
    let reveal = t.legal(2, "reveal", &target, None, None);
    t.action(2, reveal);
    t.finish();
    assert!(t.room.game.pending.is_none());
    assert!(t.room.game.board(&target).is_none());
    let c = t.room.game.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC125")
        .unwrap();
    assert!(!c.face_down && c.lock_markers == 0);
    assert!(
        t.room
            .game
            .attachments
            .iter()
            .find(|a| a.card.id == source)
            .unwrap()
            .card
            .exhausted
    );
    assert!(!t
        .steps
        .iter()
        .flat_map(|r| r["views"].as_array().unwrap())
        .any(|v| v["pendingChoice"]["kind"] == "jz46-hidden-inspect"));
    t.export();
}
#[test]
fn jz46_pause_preserves_private_inspection_and_rejects_action_until_resume() {
    let (g, held, target) = prepared(0, true, 2);
    let mut t = Trace::new("pause-inspection-resume", g);
    let source = attach(&mut t, 0, &held);
    inspect(&mut t, 0, &source, &target);
    let choice = t.room.game.pending.as_ref().unwrap().choice.id.clone();
    t.command(1, SessionAction::PauseRoom, false);
    t.command(
        0,
        SessionAction::Game {
            action: Action {
                choice_id: Some(choice.clone()),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        },
        true,
    );
    assert_eq!(t.room.game.pending.as_ref().unwrap().choice.id, choice);
    t.command(2, SessionAction::ResumeRoom, false);
    confirm(&mut t, 0);
    assert_eq!(t.room.game.board(&target).unwrap().1.lock_markers, 1);
    t.export();
}
#[test]
fn jz46_lock_persists_and_can_stack_on_next_natural_turn() {
    let (g, held, target) = prepared(0, true, 2);
    let mut t = Trace::new("repeat-lock-next-turn", g);
    let source = attach(&mut t, 0, &held);
    inspect(&mut t, 0, &source, &target);
    confirm(&mut t, 0);
    let turn = t.room.game.turn;
    for _ in 0..300 {
        if t.room.game.turn > turn
            && t.room.game.pending.is_none()
            && t.room.game.stack.is_empty()
            && t.room.game.legal_actions(0).iter().any(|a| {
                a.action.kind == "activate" && a.action.card_id.as_deref() == Some(&source)
            })
        {
            break;
        }
        if let Some(p) = t.room.game.pending.clone() {
            let selected = p
                .choice
                .options
                .iter()
                .take(p.choice.min.unwrap_or(1))
                .map(|o| o.id.clone())
                .collect();
            t.action(
                p.seat,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(selected),
                    ..Action::new("choose")
                },
            );
        } else if let Some(w) = t.room.pacing.window.clone() {
            let seat = *w
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            t.command(seat, SessionAction::PassResponse { window_id: w.id }, false);
        } else {
            let seat = (0..4)
                .find(|s| {
                    t.room
                        .game
                        .legal_actions(*s)
                        .iter()
                        .any(|a| a.action.kind == "pass")
                })
                .unwrap();
            t.command(
                seat,
                SessionAction::Game {
                    action: Action::new("pass"),
                },
                false,
            );
        }
    }
    assert!(t.room.game.turn > turn);
    let c = t.room.game.board(&target).unwrap().1;
    assert!(!c.exhausted);
    assert_eq!(c.lock_markers, 1);
    inspect(&mut t, 0, &source, &target);
    confirm(&mut t, 0);
    let c = t.room.game.board(&target).unwrap().1;
    assert!(c.exhausted);
    assert_eq!(c.lock_markers, 2);
    t.export();
}
#[test]
fn jz46_original_source_and_region_guards_use_explicit_offline_response_layouts() {
    // These mutations are explicit Native boundary layouts, not claimed real
    // response commands. Subsequent settlement/inspection uses real actions.
    for mode in 0..3 {
        let (g, held, target) = prepared(0, true, 2);
        let mut t = Trace::new("offline-source-region-boundary", g);
        let source = attach(&mut t, 0, &held);
        let a = t.legal(0, "activate", &source, Some(&target), None);
        t.action(0, a);
        if mode < 2 {
            t.room.game.remove_dead(&source, RemovalCause::Destroy);
        }
        if mode == 1 {
            let r = t.room.game.make_card("DQJC115", 0);
            t.room.game.regions[2].card = r;
        }
        if mode == 2 {
            let c = t.room.game.regions[2].cards.pop().unwrap();
            t.room.game.regions[1].cards.push(c);
        }
        let mut t = Trace::new("prepared-after-offline-response-change", t.room.game);
        t.finish();
        if mode == 0 {
            assert!(t.room.game.pending.is_some());
            confirm(&mut t, 0);
            assert_eq!(t.room.game.board(&target).unwrap().1.lock_markers, 1);
        } else {
            assert!(t.room.game.pending.is_none());
            assert_eq!(t.room.game.board(&target).unwrap().1.lock_markers, 0);
        }
    }
}
impl Trace {
    fn new(name: &'static str, mut g: Game) -> Self {
        g.room_id = "460000000000000000000001".into();
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
            command_id: format!("jz46-{}-{}", self.name, self.steps.len()),
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
            let intent_id = format!("jz46-intent-{}", self.steps.len());
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
        if let Ok(dir) = std::env::var("JZ46_NATIVE_DIR") {
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
