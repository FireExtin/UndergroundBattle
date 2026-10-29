//! Prepared layouts test the project ruling, not natural generation or admission.
use crate::{model::*, unit_support::*};

#[test]
fn tentacle_departure_never_enters_hand_or_deck() {
    for bottom in [false, true] {
        let mut g = game();
        let id = field(&mut g, "TK010", 0);
        if bottom { g.to_bottom(&id); } else { g.return_hand(&id); }
        assert!(g.board(&id).is_none());
        assert!(!g.batch3_physical_cards().any(|c| c.definition == "TK010"));
    }
}

#[test]
fn tentacle_death_has_no_grave_instance() {
    let mut g = game();
    let id = field(&mut g, "TK010", 0);
    g.remove_dead(&id, RemovalCause::Destroy);
    assert!(!g.batch3_physical_cards().any(|c| c.definition == "TK010"));
    g.remove_dead(&id, RemovalCause::Destroy);
    assert!(!g.batch3_physical_cards().any(|c| c.definition == "TK010"));
}

#[test]
fn tentacle_hide_retains_entity_then_return_removes_it() {
    let mut g = game();
    let id = field(&mut g, "TK010", 0);
    g.hide_in_place(&id);
    let hidden = g.regions[0].cards.iter().find(|c| c.definition == "TK010").unwrap();
    assert!(hidden.face_down);
    assert_ne!(hidden.id, id);
    let id = hidden.id.clone();
    restore(&mut g);
    g.return_hand(&id);
    assert!(!g.batch3_physical_cards().any(|c| c.definition == "TK010"));
}

#[test]
fn tentacle_sealed_entity_retained_but_release_cannot_create_hand_card() {
    let mut g = game();
    let id = field(&mut g, "TK010", 0);
    let host = field(&mut g, "JC125", 0);
    let (_, c) = g.leave_board(&id).unwrap();
    g.seal_card_on_valid_host(c, 0, &host);
    assert_eq!(g.sealed_cards.len(), 1);
    assert_eq!(g.sealed_cards[0].card.definition, "TK010");
    restore(&mut g);
    g.return_hand(&host);
    assert!(g.sealed_cards.is_empty());
    assert!(!g.batch3_physical_cards().any(|c| c.definition == "TK010"));
}

#[test]
fn tentacle_ruling_does_not_change_ordinary_cards() {
    let mut g = game();
    let id = field(&mut g, "JC125", 0);
    g.return_hand(&id);
    assert_eq!(g.players[0].hand[0].definition, "JC125");
    let id = field(&mut g, "JC125", 0);
    g.to_bottom(&id);
    assert_eq!(g.players[0].deck.last().unwrap().definition, "JC125");
    let id = field(&mut g, "JC125", 0);
    g.remove_dead(&id, RemovalCause::Destroy);
    assert_eq!(g.players[0].graveyard[0].definition, "JC125");
    restore(&mut g);
}

#[test]
fn tentacle_corrupt_physical_zones_rejected_by_game_and_room() {
    let g = game();
    for zone in 0..5 {
        let mut bad = g.clone();
        let c = bad.make_card("TK010", 0);
        match zone {
            0 => bad.players[0].hand.push(c),
            1 => bad.players[0].deck.push(c),
            2 => bad.players[0].graveyard.push(c),
            3 => bad.players[0].assets.push(c),
            _ => bad.players[0].score_cards.push(c),
        }
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
        let mut room = crate::jc029_tests::envelope(&g);
        room.game = bad;
        assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());
    }
}

#[test]
fn tentacle_original_icons_follow_live_core_controller_and_hide() {
    let mut g = game();
    g.players[0].society_zone.card = Some(g.make_card("MSBQ02", 0));
    let core = field(&mut g, "BQ046", 0);
    g.core_zones.push(crate::core_zone::CoreZone { owner:0, definition:"BQ046".into(), card:None, rest:0 });
    let first = field(&mut g, "TK010", 0);
    let second = field(&mut g, "TK010", 0);
    let base = crate::catalog::card("BQ046").defense.unwrap();
    assert_eq!(g.defense(g.board(&core).unwrap().1, 0), base + 2);
    assert_eq!(g.tentacle_temporary_combat(g.board(&first).unwrap().1, 0), 2);
    assert_eq!(g.current_icons(g.board(&first).unwrap().1, 0).combat, 2);
    g.hide_in_place(&second);
    assert_eq!(g.defense(g.board(&core).unwrap().1, 0), base + 1);
    g.board_mut(&first).unwrap().controller = 1;
    assert_eq!(g.tentacle_core_defense(g.board(&core).unwrap().1), 0);
    assert_eq!(g.tentacle_temporary_combat(g.board(&first).unwrap().1, 0), 0);
}

#[test]
fn tentacle_name_not_a_fabricated_subtype_and_current_directory_remains_closed() {
    assert_current_serial_catalog();
    let mut g = game();
    let id = field(&mut g, "TK010", 0);
    let c = g.board(&id).unwrap().1;
    assert_eq!(crate::catalog::card("TK010").subtypes, ["突变体", "器官"]);
    assert!(crate::tentacle::character(c));
    assert!(!crate::catalog::card("TK010").supported);
    let source_card = g.make_card("BQ048", 0);
    let source = g.source_snapshot(&source_card, None);
    let victim = field(&mut g, "JC125", 1);
    assert!(g.bq048_region_target(0, &source, &victim));
    g.hide_in_place(&id);
    assert!(!g.bq048_region_target(0, &source, &victim));
}

#[test]
fn tentacle_prepared_room_hide_reveal_return_and_duplicate_export() {
    use crate::abi112_review_play_tests::Trace;
    fn settle(t: &mut Trace) {
        for _ in 0..80 {
            if let Some(p) = &t.room.game.pending {
                assert_eq!(p.choice.kind, "bq063-opponent");
                let id = p.choice.options[0].id.clone();
                t.choose(vec![id]);
            } else if t.room.game.stack.is_empty() && t.room.game.effects.is_empty() { return; }
            else { t.pass(); }
        }
        panic!("room did not settle");
    }
    let mut g = game();
    g.players[0].society_zone.card = Some(g.make_card("MSBQ02", 0));
    fund(&mut g, 0, "JC069", 6);
    field(&mut g, "TK010", 0);
    field(&mut g, "TK010", 0);
    let hide = held(&mut g, "BQ063", 0);
    let back = held(&mut g, "BQ063", 0);
    let mut t = Trace::new_preserving_rng(g, "tentacle-prepared-lifecycle".into(), 681);
    t.action(0, Action { card_id:Some(hide), ability_id:Some(crate::bq063::KEY.into()), option:Some("hide".into()), ..Action::new("play") });
    settle(&mut t);
    assert_eq!(t.room.game.regions[0].cards.iter().filter(|c| c.definition == "TK010" && c.face_down).count(), 2);
    let hidden = t.room.game.regions[0].cards.iter().find(|c| c.definition == "TK010").unwrap().id.clone();
    t.action(0, Action {card_id:Some(hidden), ..Action::new("reveal")});
    settle(&mut t);
    t.action(0, Action { card_id:Some(back), ability_id:Some(crate::bq063::KEY.into()), option:Some("return".into()), ..Action::new("play") });
    settle(&mut t);
    assert_eq!(t.room.game.batch3_physical_cards().filter(|c| c.definition == "TK010").count(), 1);
    assert!(!t.room.game.regions[0].cards.iter().find(|c| c.definition == "TK010").unwrap().face_down);
    let mut value = t.value();
    for step in value["steps"].as_array_mut().unwrap() {
        let room = crate::room::RoomEnvelope::from_persisted(step["expected"]["state"].as_str().unwrap()).unwrap();
        let duplicate = room.transition(step["seat"].as_u64().unwrap() as usize, Some(serde_json::from_value(step["command"].clone()).unwrap()), 0).unwrap();
        assert_eq!(duplicate.outcome, "rejected");
        assert_eq!(duplicate.state, step["expected"]["state"]);
        assert!(!duplicate.changed && duplicate.journal.is_empty());
        step["duplicate"] = serde_json::to_value(duplicate).unwrap();
    }
    let sealed = sealed_room_trace();
    if let Ok(path) = std::env::var("TENTACLE_NATIVE_OUT") {
        std::fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({"catalog":crate::catalog::catalog(),"sourceBindings":crate::bq052_tests::source_bindings(),"cases":[value,sealed]})).unwrap()).unwrap();
    }
}

#[test]
fn tentacle_hide_performs_normal_host_cleanup_without_destroying_token() {
    let mut g = game();
    let id = field(&mut g, "TK010", 0);
    let attachment = g.make_card("JC073", 0);
    g.attachments.push(Attachment { card:attachment, host_id:id.clone() });
    let sealed = g.make_card("JC125", 0);
    g.seal_card_on_valid_host(sealed, 0, &id);
    g.board_mut(&id).unwrap().shield = 1;
    g.board_mut(&id).unwrap().lock_markers = 2;
    g.hide_in_place(&id);
    assert!(g.attachments.is_empty() && g.sealed_cards.is_empty());
    assert!(g.players[0].graveyard.iter().any(|c| c.definition == "JC073"));
    assert!(g.players[0].hand.iter().any(|c| c.definition == "JC125"));
    let token = g.regions[0].cards.iter().find(|c| c.definition == "TK010").unwrap();
    assert!(token.face_down);
    assert_eq!((token.shield, token.lock_markers), (0,0));
    restore(&mut g);
}

#[test]
fn tentacle_sealing_suspends_core_aura_and_death_receipt_has_no_phantom_grave() {
    let mut g = game();
    g.players[0].society_zone.card = Some(g.make_card("MSBQ02", 0));
    let core = field(&mut g, "BQ046", 0);
    g.core_zones.push(crate::core_zone::CoreZone { owner:0, definition:"BQ046".into(), card:None, rest:0 });
    let id = field(&mut g, "TK010", 0);
    let second = field(&mut g, "TK010", 0);
    assert_eq!(g.tentacle_core_defense(g.board(&core).unwrap().1), 2);
    let (_, c) = g.leave_board(&id).unwrap();
    g.seal_card_on_valid_host(c, 0, &core);
    assert_eq!(g.tentacle_core_defense(g.board(&core).unwrap().1), 1);
    let receipt = g.remove_dead_with_snapshot(&second, RemovalCause::Sacrifice, None, None, None).unwrap();
    assert!(receipt.grave_instance.is_empty());
    assert_eq!(receipt.old_instance, second);
    assert_eq!(g.tentacle_core_defense(g.board(&core).unwrap().1), 0);
    assert_eq!(g.sealed_cards.len(), 1);
    restore(&mut g);
}

fn sealed_room_trace()->serde_json::Value {
    use crate::abi112_review_play_tests::Trace;
    let mut g=game();g.players[0].society_zone.card=Some(g.make_card("MSBQ02",0));
    fund(&mut g,0,"JC066",5);fund(&mut g,0,"JC069",3);
    let token=field(&mut g,"TK010",0);field(&mut g,"TK010",0);
    let host=held(&mut g,"JC066",0);let hide=held(&mut g,"BQ063",0);
    let mut t=Trace::new_preserving_rng(g,"tentacle-prepared-seal-and-host-hide".into(),683);
    t.action(0,Action{card_id:Some(host),region:Some(0),..Action::new("deploy")});
    t.until_pending();t.choose(vec![token]);
    for _ in 0..64 {if t.room.game.pending.is_none()&&t.room.game.stack.is_empty()&&t.room.game.effects.is_empty(){break;}t.pass();}
    assert_eq!(t.room.game.sealed_cards.len(),1);assert_eq!(t.room.game.sealed_cards[0].card.definition,"TK010");
    t.action(0,Action{card_id:Some(hide),ability_id:Some(crate::bq063::KEY.into()),option:Some("hide".into()),..Action::new("play")});
    for _ in 0..80 {if let Some(p)=&t.room.game.pending{assert_eq!(p.choice.kind,"bq063-opponent");let id=p.choice.options[0].id.clone();t.choose(vec![id]);}else if t.room.game.stack.is_empty()&&t.room.game.effects.is_empty(){break;}else{t.pass();}}
    assert!(t.room.game.pending.is_none()&&t.room.game.stack.is_empty());assert!(t.room.game.sealed_cards.is_empty());
    let tokens=t.room.game.batch3_physical_cards().filter(|c|c.definition=="TK010").collect::<Vec<_>>();assert_eq!(tokens.len(),1);assert!(tokens[0].face_down);
    let mut value=t.value();for step in value["steps"].as_array_mut().unwrap(){let room=crate::room::RoomEnvelope::from_persisted(step["expected"]["state"].as_str().unwrap()).unwrap();let d=room.transition(step["seat"].as_u64().unwrap()as usize,Some(serde_json::from_value(step["command"].clone()).unwrap()),0).unwrap();assert_eq!(d.outcome,"rejected");assert_eq!(d.state,step["expected"]["state"]);assert!(!d.changed&&d.journal.is_empty());step["duplicate"]=serde_json::to_value(d).unwrap();}value
}
