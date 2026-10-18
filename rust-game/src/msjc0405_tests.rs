//! Complete printed MSJC04/MSJC05 bodies, using only admitted real cards.
use crate::msjc06_tests::*;
use crate::{catalog, model::*, room::*};
use serde_json::{json, Value};
const RED: Spec = Spec {
    id: "MSJC04", search: "search-red-unique", color: "红",
    name: ("鸣钟教派", "末日钟的侍奉者"), subtypes: &["秘密教派"],
    cards: &["JC042", "JC049", "JC047", "XQ17", "JZ31", "JC045", "JC050", "XQ18",
        "JZ30", "WM038", "JZ35", "JC052", "JC046", "JC054", "JC044"],
    full: 45, color_count: 15, uniques: &["JC052"],
    hits: ["JC052", "JC052", "JC042", "LC23"],
};
const GRAY: Spec = Spec {
    id: "MSJC05", search: "search-gray-unique", color: "灰",
    name: ("国家机构", "政府权力"), subtypes: &["机构"],
    cards: &["JC056", "JC058", "JC059", "JC063", "JC069", "JZ43", "XQ27", "JZ44",
        "JZ45", "XQ24", "JC061", "XQ25", "JC060", "JZ46", "JC057"],
    full: 45, color_count: 15, uniques: &[],
    hits: ["JC060", "JZ46", "JC056", "LC23"],
};
#[test] fn msjc04_whole_original_and_real_red_unique_program() { original(&RED); }
#[test] fn msjc05_whole_original_and_gray_unique_program() { original(&GRAY); }
#[test] fn msjc04_construction_24_25_45_and_society_outside_50() { construction(&RED); }
#[test] fn msjc05_construction_24_25_45_and_society_outside_50() { construction(&GRAY); }
#[test] fn msjc04_four_personal_searches_private_choice_public_reveal_and_restore() { four_searches(&RED); }
#[test] fn msjc05_real_pool_no_gray_unique_still_pays_consumes_quota_and_shuffles_all_seats() { no_unique_in_admitted_pool(&GRAY); }
#[test] fn msjc04_atomic_rejections_wrong_source_cost_ability_and_exhaustion() { rejections(&RED); }
#[test] fn msjc05_atomic_rejections_wrong_source_cost_ability_and_exhaustion() { rejections(&GRAY); }
#[test] fn msjc04_initiative_draw_and_rear_paid_no_draw() { initiative_draw(&RED); }
#[test] fn msjc05_initiative_draw_and_rear_paid_no_draw() { initiative_draw(&GRAY); }
#[test] fn msjc04_empty_search_shuffles_and_empty_deck_is_not_draw_elimination() { empty_search(&RED); }
#[test] fn msjc05_empty_search_shuffles_and_empty_deck_is_not_draw_elimination() { empty_search(&GRAY); }
#[test] fn msjc04_cancelled_paid_frame_keeps_payment_exhaustion_and_quota() { cancelled_frame(&RED); }
#[test] fn msjc05_cancelled_paid_frame_keeps_payment_exhaustion_and_quota() { cancelled_frame(&GRAY); }
#[test] fn msjc04_ready_turn_new_instance_restore_and_restart_quota() { quota_lifecycle(&RED); }
#[test] fn msjc05_ready_turn_new_instance_restore_and_restart_quota() { quota_lifecycle(&GRAY); }

#[test]
fn msjc0405_reject_partial_or_relaxed_registry_programs() {
    for spec in [&RED, &GRAY] {
        let complete = crate::rules::definition(spec.id).clone();
        let valid = std::collections::BTreeMap::from([(spec.id.into(), complete.clone())]);
        crate::rules::validate_definitions(&valid).unwrap();
        for case in ["missing-draw", "free-search", "unlimited-search", "wrong-color"] {
            let mut partial = complete.clone();
            match case {
                "missing-draw" => { partial.abilities.remove(0); }
                "free-search" => partial.abilities[1].costs.clear(),
                "unlimited-search" => partial.abilities[1].once_per_game = false,
                _ => partial.abilities[1].ops[0] = crate::rules::Op::Search {
                    player: crate::rules::PlayerRef::Actor, filter: crate::rules::CardFilter::Any,
                    to_top: false, optional: false, visibility: crate::rules::SearchVisibility::Reveal,
                },
            }
            assert!(crate::rules::validate_definitions(&std::collections::BTreeMap::from([
                (spec.id.into(), partial),
            ])).is_err(), "{} {case}", spec.id);
        }
    }
}

struct Trace { name: String, room: RoomEnvelope, initial: Value, steps: Vec<Value> }
impl Trace {
    fn new(spec: &Spec, suffix: &str, first: usize) -> Self {
        let mut g = initial(spec);
        g.room_id = if spec.id == "MSJC04" { "830000000000000000000004" } else { "830000000000000000000005" }.into();
        g.first_team = first;
        crate::msjc01_tests::fund(&mut g, 0, 8);
        if suffix == "search" {
            for id in spec.hits {
                let c = g.make_card(id, 0); g.players[0].deck.push(c);
            }
        }
        let room = crate::jc029_tests::envelope(&g);
        let initial = json!({"state":serde_json::to_string(&room).unwrap(),"views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>()});
        Self {name:format!("{}-{suffix}",spec.id.to_lowercase()),room,initial,steps:vec![]}
    }
    fn command(&mut self, seat: usize, action: SessionAction, accepted: bool) {
        let state = serde_json::to_string(&self.room).unwrap();
        let command = RoomCommand {command_id:format!("{}-{}",self.name,self.steps.len()),expected_version:self.room.revision,action};
        let expected = self.room.transition(seat,Some(command.clone()),0).unwrap();
        assert_eq!(expected.error_code.is_none(),accepted,"{}: {:?}",self.name,expected.error_message);
        if expected.changed {
            assert_eq!(serde_json::to_string(&self.room.replay_events(&expected.journal).unwrap()).unwrap(),expected.state);
        } else { assert_eq!(expected.state,state); assert!(expected.journal.is_empty()); }
        self.room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        let views = (0..4).map(|s|self.room.view(s,0)).collect::<Vec<_>>();
        self.steps.push(json!({"state":state,"seat":seat,"command":command,"serverNowMs":"0","expected":expected,"views":views}));
    }
    fn resolve_response(&mut self) {
        for _ in 0..16 {
            let Some(window) = self.room.pacing.window.clone() else { return; };
            let seat = *window.members.iter().find(|(_,d)|matches!(d,Decision::Undecided {..})).expect("undecided response").0;
            self.command(seat,SessionAction::PassResponse {window_id:window.id},true);
        }
        panic!("response bound");
    }
    fn finish(self) -> Value { json!({"name":self.name,"initial":self.initial,"steps":self.steps}) }
}

#[test]
fn msjc0405_native83_oracles() {
    let mut scenarios = vec![];
    for spec in [&RED, &GRAY] {
        let mut trace = Trace::new(spec,"search",0);
        let action = activate(&trace.room.game,0,spec.search);
        trace.command(1,SessionAction::Game {action:action.clone()},false);
        trace.command(0,SessionAction::Game {action:action.clone()},true);
        trace.resolve_response();
        if let Some(pending) = trace.room.game.pending.clone() {
            assert_eq!(spec.id,"MSJC04"); assert_eq!(pending.choice.options.len(),2);
            let choose = Action {choice_id:Some(pending.choice.id),selected:Some(vec![pending.choice.options[0].id.clone()]),..Action::new("choose")};
            trace.command(1,SessionAction::Game {action:choose.clone()},false);
            trace.command(0,SessionAction::PauseRoom,true);
            trace.command(0,SessionAction::Game {action:choose.clone()},false);
            trace.command(0,SessionAction::ResumeRoom,true);
            trace.command(0,SessionAction::Game {action:choose},true);
            assert_eq!(trace.room.game.players[0].hand[0].definition,"JC052");
        } else {
            assert_eq!(spec.id,"MSJC05"); assert!(trace.room.game.players[0].hand.is_empty());
        }
        assert!(trace.room.game.players[0].society_zone.used_once_per_game.contains(spec.search));
        assert_eq!(trace.room.game.players[0].assets.iter().filter(|c|c.exhausted).count(),4);
        trace.command(0,SessionAction::Game {action},false);
        scenarios.push(trace.finish());
        for first in [0,1] {
            let mut trace = Trace::new(spec,if first==0 {"initiative-draw"} else {"rear-paid-no-draw"},first);
            let action = activate(&trace.room.game,0,"drawWithInitiative");
            trace.command(0,SessionAction::Game {action},true); trace.resolve_response();
            assert_eq!(trace.room.game.players[0].hand.len(),usize::from(first==0));
            assert_eq!(trace.room.game.players[0].assets.iter().filter(|c|c.exhausted).count(),3);
            assert!(trace.room.game.players[0].society_zone.used_once_per_game.is_empty());
            scenarios.push(trace.finish());
        }
    }
    if let Ok(directory) = std::env::var("MSJC0405_NATIVE_DIR") {
        assert_eq!(catalog::ENGINE_VERSION, "rust-v0.2.83-msjc04-msjc05-societies-candidate", "A newer compiler input cannot export or re-sign the historical Native83 witness");
        std::fs::create_dir_all(&directory).unwrap();
        let output=json!({"format":"msjc0405-native83-v1","scope":"Explicit Native layouts and real Room commands, complete opaque states, journals and four views; no online gameplay claim","catalog":catalog::catalog(),"sourceBindings":source_bindings(),"scenarios":scenarios});
        use std::io::Write;
        std::fs::OpenOptions::new().create_new(true).write(true).open(std::path::Path::new(&directory).join("native83.json")).unwrap().write_all(&serde_json::to_vec(&output).unwrap()).unwrap();
    }
}

fn source_bindings() -> Value {
    use sha2::{Digest, Sha256};
    let files: &[(&str, &[u8])] = &[
        ("Cargo.lock", include_bytes!("../../Cargo.lock")),
        ("Cargo.toml", include_bytes!("../../Cargo.toml")),
        ("rust-game-wasm/Cargo.toml", include_bytes!("../../rust-game-wasm/Cargo.toml")),
        ("rust-game-wasm/build.sh", include_bytes!("../../rust-game-wasm/build.sh")),
        ("rust-game-wasm/src/lib.rs", include_bytes!("../../rust-game-wasm/src/lib.rs")),
        ("rust-game/Cargo.toml", include_bytes!("../Cargo.toml")),
        ("rust-game/data/cards.json", include_bytes!("../data/cards.json")),
        ("rust-game/src/attachment.rs", include_bytes!("attachment.rs")),
        ("rust-game/src/attributes.rs", include_bytes!("attributes.rs")),
        ("rust-game/src/bin/hegemony-audit.rs", include_bytes!("bin/hegemony-audit.rs")),
        ("rust-game/src/black_expansion.rs", include_bytes!("black_expansion.rs")),
        ("rust-game/src/black_expansion_tests.rs", include_bytes!("black_expansion_tests.rs")),
        ("rust-game/src/blue_expansion.rs", include_bytes!("blue_expansion.rs")),
        ("rust-game/src/blue_expansion_tests.rs", include_bytes!("blue_expansion_tests.rs")),
        ("rust-game/src/blue_minimum.rs", include_bytes!("blue_minimum.rs")),
        ("rust-game/src/blue_minimum_tests.rs", include_bytes!("blue_minimum_tests.rs")),
        ("rust-game/src/bounded_entry_fresh_tests.rs", include_bytes!("bounded_entry_fresh_tests.rs")),
        ("rust-game/src/bounded_search.rs", include_bytes!("bounded_search.rs")),
        ("rust-game/src/bounded_search_tests.rs", include_bytes!("bounded_search_tests.rs")),
        ("rust-game/src/bq030.rs", include_bytes!("bq030.rs")),
        ("rust-game/src/bq030_preparation.rs", include_bytes!("bq030_preparation.rs")),
        ("rust-game/src/bq035.rs", include_bytes!("bq035.rs")),
        ("rust-game/src/bq035_tests.rs", include_bytes!("bq035_tests.rs")),
        ("rust-game/src/catalog.rs", include_bytes!("catalog.rs")),
        ("rust-game/src/control.rs", include_bytes!("control.rs")),
        ("rust-game/src/control_tests.rs", include_bytes!("control_tests.rs")),
        ("rust-game/src/death_observer_tests.rs", include_bytes!("death_observer_tests.rs")),
        ("rust-game/src/death_observers.rs", include_bytes!("death_observers.rs")),
        ("rust-game/src/deck.rs", include_bytes!("deck.rs")),
        ("rust-game/src/deck_seal_search.rs", include_bytes!("deck_seal_search.rs")),
        ("rust-game/src/deck_seal_search_draft.rs", include_bytes!("deck_seal_search_draft.rs")),
        ("rust-game/src/deck_seal_search_tests.rs", include_bytes!("deck_seal_search_tests.rs")),
        ("rust-game/src/defence_equipment_tests.rs", include_bytes!("defence_equipment_tests.rs")),
        ("rust-game/src/dream_reveal_tests.rs", include_bytes!("dream_reveal_tests.rs")),
        ("rust-game/src/engine.rs", include_bytes!("engine.rs")),
        ("rust-game/src/equipment_abilities_tests.rs", include_bytes!("equipment_abilities_tests.rs")),
        ("rust-game/src/gray_expansion.rs", include_bytes!("gray_expansion.rs")),
        ("rust-game/src/gray_expansion_tests.rs", include_bytes!("gray_expansion_tests.rs")),
        ("rust-game/src/gray_lock.rs", include_bytes!("gray_lock.rs")),
        ("rust-game/src/gray_lock_tests.rs", include_bytes!("gray_lock_tests.rs")),
        ("rust-game/src/green_minimum_tests.rs", include_bytes!("green_minimum_tests.rs")),
        ("rust-game/src/hand_deck_tests.rs", include_bytes!("hand_deck_tests.rs")),
        ("rust-game/src/hand_interactions_tests.rs", include_bytes!("hand_interactions_tests.rs")),
        ("rust-game/src/jc004_tests.rs", include_bytes!("jc004_tests.rs")),
        ("rust-game/src/jc005_tests.rs", include_bytes!("jc005_tests.rs")),
        ("rust-game/src/jc008_tests.rs", include_bytes!("jc008_tests.rs")),
        ("rust-game/src/jc029_tests.rs", include_bytes!("jc029_tests.rs")),
        ("rust-game/src/jc030_tests.rs", include_bytes!("jc030_tests.rs")),
        ("rust-game/src/jc037.rs", include_bytes!("jc037.rs")),
        ("rust-game/src/jc037_tests.rs", include_bytes!("jc037_tests.rs")),
        ("rust-game/src/jc046.rs", include_bytes!("jc046.rs")),
        ("rust-game/src/jc046_tests.rs", include_bytes!("jc046_tests.rs")),
        ("rust-game/src/jc050.rs", include_bytes!("jc050.rs")),
        ("rust-game/src/jc050_tests.rs", include_bytes!("jc050_tests.rs")),
        ("rust-game/src/jc052.rs", include_bytes!("jc052.rs")),
        ("rust-game/src/jc052_tests.rs", include_bytes!("jc052_tests.rs")),
        ("rust-game/src/jc054.rs", include_bytes!("jc054.rs")),
        ("rust-game/src/jc054_tests.rs", include_bytes!("jc054_tests.rs")),
        ("rust-game/src/jc060.rs", include_bytes!("jc060.rs")),
        ("rust-game/src/jc060_tests.rs", include_bytes!("jc060_tests.rs")),
        ("rust-game/src/jc061_tests.rs", include_bytes!("jc061_tests.rs")),
        ("rust-game/src/jc087_tests.rs", include_bytes!("jc087_tests.rs")),
        ("rust-game/src/jc089_tests.rs", include_bytes!("jc089_tests.rs")),
        ("rust-game/src/jc094_tests.rs", include_bytes!("jc094_tests.rs")),
        ("rust-game/src/jc095.rs", include_bytes!("jc095.rs")),
        ("rust-game/src/jc095_tests.rs", include_bytes!("jc095_tests.rs")),
        ("rust-game/src/jz22.rs", include_bytes!("jz22.rs")),
        ("rust-game/src/jz22_tests.rs", include_bytes!("jz22_tests.rs")),
        ("rust-game/src/jz31_tests.rs", include_bytes!("jz31_tests.rs")),
        ("rust-game/src/jz35.rs", include_bytes!("jz35.rs")),
        ("rust-game/src/jz35_tests.rs", include_bytes!("jz35_tests.rs")),
        ("rust-game/src/jz46.rs", include_bytes!("jz46.rs")),
        ("rust-game/src/jz46_tests.rs", include_bytes!("jz46_tests.rs")),
        ("rust-game/src/jz48_tests.rs", include_bytes!("jz48_tests.rs")),
        ("rust-game/src/jz49_tests.rs", include_bytes!("jz49_tests.rs")),
        ("rust-game/src/jz50_search.rs", include_bytes!("jz50_search.rs")),
        ("rust-game/src/jz50_search_tests.rs", include_bytes!("jz50_search_tests.rs")),
        ("rust-game/src/jz55_tests.rs", include_bytes!("jz55_tests.rs")),
        ("rust-game/src/lib.rs", include_bytes!("lib.rs")),
        ("rust-game/src/main.rs", include_bytes!("main.rs")),
        ("rust-game/src/mill_public_tests.rs", include_bytes!("mill_public_tests.rs")),
        ("rust-game/src/mobility.rs", include_bytes!("mobility.rs")),
        ("rust-game/src/model.rs", include_bytes!("model.rs")),
        ("rust-game/src/msjc01_tests.rs", include_bytes!("msjc01_tests.rs")),
        ("rust-game/src/msjc06_tests.rs", include_bytes!("msjc06_tests.rs")),
        ("rust-game/src/msjc07_tests.rs", include_bytes!("msjc07_tests.rs")),
        ("rust-game/src/msjc08_tests.rs", include_bytes!("msjc08_tests.rs")),
        ("rust-game/src/msjc11_tests.rs", include_bytes!("msjc11_tests.rs")),
        ("rust-game/src/play_sources.rs", include_bytes!("play_sources.rs")),
        ("rust-game/src/protection_tests.rs", include_bytes!("protection_tests.rs")),
        ("rust-game/src/purple_tools_tests.rs", include_bytes!("purple_tools_tests.rs")),
        ("rust-game/src/red_time.rs", include_bytes!("red_time.rs")),
        ("rust-game/src/red_time_tests.rs", include_bytes!("red_time_tests.rs")),
        ("rust-game/src/region_aura_tests.rs", include_bytes!("region_aura_tests.rs")),
        ("rust-game/src/renown.rs", include_bytes!("renown.rs")),
        ("rust-game/src/renown_tests.rs", include_bytes!("renown_tests.rs")),
        ("rust-game/src/repress_assets_tests.rs", include_bytes!("repress_assets_tests.rs")),
        ("rust-game/src/resolution.rs", include_bytes!("resolution.rs")),
        ("rust-game/src/room.rs", include_bytes!("room.rs")),
        ("rust-game/src/response_preview_tests.rs", include_bytes!("response_preview_tests.rs")),
        ("rust-game/src/rules.rs", include_bytes!("rules.rs")),
        ("rust-game/src/sealing.rs", include_bytes!("sealing.rs")),
        ("rust-game/src/sealing_tests.rs", include_bytes!("sealing_tests.rs")),
        ("rust-game/src/service.rs", include_bytes!("service.rs")),
        ("rust-game/src/society.rs", include_bytes!("society.rs")),
        ("rust-game/src/society_msjc09_tests.rs", include_bytes!("society_msjc09_tests.rs")),
        ("rust-game/src/unit_support.rs", include_bytes!("unit_support.rs")),
        ("rust-game/src/white_wound_tests.rs", include_bytes!("white_wound_tests.rs")),
        ("rust-game/src/win_flow_tests.rs", include_bytes!("win_flow_tests.rs")),
        ("rust-game/src/wm023.rs", include_bytes!("wm023.rs")),
        ("rust-game/src/wm023_tests.rs", include_bytes!("wm023_tests.rs")),
        ("rust-game/src/wm038.rs", include_bytes!("wm038.rs")),
        ("rust-game/src/wm038_tests.rs", include_bytes!("wm038_tests.rs")),
        ("rust-game/src/wm056.rs", include_bytes!("wm056.rs")),
        ("rust-game/src/wm056_tests.rs", include_bytes!("wm056_tests.rs")),
        ("rust-game/src/wm062.rs", include_bytes!("wm062.rs")),
        ("rust-game/src/wm062_tests.rs", include_bytes!("wm062_tests.rs")),
        ("rust-game/src/world.rs", include_bytes!("world.rs")),
        ("rust-game/src/wound_defence_tests.rs", include_bytes!("wound_defence_tests.rs")),
        ("rust-game/src/xq18_tests.rs", include_bytes!("xq18_tests.rs")),
        ("rust-game/src/xq24_tests.rs", include_bytes!("xq24_tests.rs")),
        ("rust-game/src/xq25.rs", include_bytes!("xq25.rs")),
        ("rust-game/src/xq25_tests.rs", include_bytes!("xq25_tests.rs")),
        ("rust-game/src/xq27.rs", include_bytes!("xq27.rs")),
        ("rust-game/src/xq27_tests.rs", include_bytes!("xq27_tests.rs")),
        ("rust-game/src/xq37.rs", include_bytes!("xq37.rs")),
        ("rust-game/src/xq37_tests.rs", include_bytes!("xq37_tests.rs")),
        ("rust-game/src/yellow_search_batch_tests.rs", include_bytes!("yellow_search_batch_tests.rs")),
        ("rust-game/src/msjc0405_tests.rs", include_bytes!("msjc0405_tests.rs")),
    ];
    json!({"algorithm":"sha256","files":files.iter().map(|(path,bytes)| ((*path).to_owned(),format!("{:x}",Sha256::digest(bytes)))).collect::<std::collections::BTreeMap<_,_>>()})
}
