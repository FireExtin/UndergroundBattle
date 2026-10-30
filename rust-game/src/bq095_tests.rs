//! Foundation tests, not Game/Room admission or end-to-end card evidence.
use crate::{bq095::{Destination as D, Headquarters}, model::Card, unit_support::game};

fn source(g: &mut crate::model::Game, owner: usize, controller: usize) -> Card {
    let mut c = g.make_card("BQ095", owner);
    c.controller = controller;
    c
}

#[test]
fn bq095_first_entry_moves_all_cards_with_existing_zone_reset_and_fresh_identity() {
    let mut g = game();
    let source = source(&mut g, 0, 0);
    g.players[0].graveyard = vec![g.make_card("JC125", 0), g.make_card("JC001", 0)];
    let before = g.players[0].graveyard.clone();
    let mut expected = g.clone();
    let expected_zone = expected.id();
    let expected_cards = before.iter().cloned().map(|c| expected.reset_zone_card(c)).collect::<Vec<_>>();
    let hq = Headquarters::first_entry(&mut g, &source, "society:p0").unwrap();
    assert!(g.players[0].graveyard.is_empty());
    assert_eq!(hq.id, expected_zone);
    assert_eq!(serde_json::to_value(&hq.cards).unwrap(), serde_json::to_value(expected_cards).unwrap());
    assert!(hq.cards.iter().all(|c| !before.iter().any(|old| old.id == c.id)));
    assert_eq!(hq.cards.iter().map(|c| c.definition.as_str()).collect::<Vec<_>>(), ["JC125", "JC001"]);
    hq.validate().unwrap();
}

#[test]
fn bq095_owned_text_replacement_does_not_follow_controller_or_enemy_text() {
    let mut g = game();
    let entry = source(&mut g, 0, 0);
    let hq = Headquarters::first_entry(&mut g, &entry, "society:p0").unwrap();
    let mut ours = g.make_card("CQ09", 0);
    ours.controller = 2;
    assert_eq!(hq.own_printed_reference(&ours), D::Headquarters(hq.id.clone()));
    let mut foreign = g.make_card("CQ09", 2);
    foreign.controller = 0;
    assert_eq!(hq.own_printed_reference(&foreign), D::Graveyard(0));
    assert_eq!(Headquarters::ordinary_grave_reference(0), D::Graveyard(0));
    assert_eq!(Headquarters::ordinary_grave_reference(2), D::Graveyard(2));
}

#[test]
fn bq095_destination_replacement_is_distinct_from_reference_replacement() {
    let mut g = game();
    let entry = source(&mut g, 2, 0);
    let hq = Headquarters::first_entry(&mut g, &entry, "society:p0").unwrap();
    // The entering attachment's owner does not change which society it serves.
    assert_eq!(hq.seat, 0);
    assert_eq!(hq.incoming_destination(0), D::Headquarters(hq.id.clone()));
    assert_eq!(hq.incoming_destination(2), D::Graveyard(2));
}

#[test]
fn bq095_rejects_wrong_source_host_and_duplicate_migration_without_mutation() {
    let mut g = game();
    let entry = source(&mut g, 0, 0);
    let card = g.make_card("JC125", 0);
    for case in 0..5 {
        let mut source = entry.clone();
        let mut grave = vec![card.clone()];
        let mut host = "society:p0";
        match case {
            0 => source.definition = "BQ091".into(),
            1 => source.face_down = true,
            2 => host = "society:p2",
            3 => grave.push(card.clone()),
            _ => grave[0].owner = 2,
        }
        g.players[0].graveyard = grave;
        let before = serde_json::to_value(&g).unwrap();
        assert!(Headquarters::first_entry(&mut g, &source, host).is_err());
        assert_eq!(serde_json::to_value(&g).unwrap(), before);
    }
}

#[test]
fn bq095_zone_round_trip_retains_modifier_after_original_attachment_leaves() {
    let mut g = game();
    let entry = source(&mut g, 0, 0);
    g.players[0].graveyard = vec![g.make_card("JC125", 0)];
    let hq = Headquarters::first_entry(&mut g, &entry, "society:p0").unwrap();
    let bytes = serde_json::to_string(&hq).unwrap();
    let restored: Headquarters = serde_json::from_str(&bytes).unwrap();
    restored.validate().unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), bytes);
    assert_eq!(restored.incoming_destination(0), D::Headquarters(hq.id));
    let mut bad = restored.clone();
    bad.cards.push(bad.cards[0].clone());
    assert!(bad.validate().is_err());
    let mut extra = serde_json::to_value(restored).unwrap();
    extra["graveyardAlias"] = true.into();
    assert!(serde_json::from_value::<Headquarters>(extra).is_err());
}

#[test]
fn bq095_real_container_receive_and_take_preserve_one_physical_instance() {
    let mut g = game();
    let entry = source(&mut g, 0, 0);
    let mut hq = Headquarters::first_entry(&mut g, &entry, "society:p0").unwrap();
    let card = g.make_card("JC125", 0);
    let before = serde_json::to_value(&card).unwrap();
    hq.receive(card.clone()).unwrap();
    assert!(hq.receive(card.clone()).is_err());
    assert!(hq.receive(g.make_card("JC125", 2)).is_err());
    let foreign_source = g.make_card("CQ09", 2);
    assert!(hq.take_for_own_printed_reference(&foreign_source, &card.id).is_err());
    assert_eq!(hq.cards.len(), 1);
    let own_source = g.make_card("CQ09", 0);
    let taken = hq.take_for_own_printed_reference(&own_source, &card.id).unwrap();
    assert_eq!(serde_json::to_value(taken).unwrap(), before);
    assert!(hq.cards.is_empty());
    assert!(hq.take_for_own_printed_reference(&own_source, &card.id).is_err());
}

#[test]
fn bq095_persistent_zone_reentry_reuses_identity_and_keeps_existing_cards() {
    let mut g=game();let s=source(&mut g,0,0);
    let c=g.make_card("JC125",0);g.players[0].graveyard.push(c);
    let id=g.bq095_create_or_reuse(&s,"society:p0").unwrap();
    let before=serde_json::to_value(&g.bq095_headquarters).unwrap();
    let s2=source(&mut g,0,0);
    assert_eq!(g.bq095_create_or_reuse(&s2,"society:p0").unwrap(),id);
    assert_eq!(serde_json::to_value(&g.bq095_headquarters).unwrap(),before);
    crate::unit_support::restore(&mut g);
    assert_eq!(g.bq095_headquarters.len(),1);
    assert!(g.players[0].graveyard.is_empty());
    for seat in 0..4 {let view=g.view(seat);assert_eq!(view.headquarters.len(),1);assert_eq!(view.headquarters[0].id,id);assert_eq!(view.headquarters[0].cards.len(),1);assert!(view.graveyard.is_empty());}
}

#[test]
fn bq095_persistent_zone_rejects_duplicate_physical_entity_and_invalid_zone() {
    let mut g=game();let s=source(&mut g,0,0);let c=g.make_card("JC125",0);g.players[0].graveyard.push(c);
    g.bq095_create_or_reuse(&s,"society:p0").unwrap();
    for case in 0..8 {let mut bad=g.clone();match case {
        0=>bad.bq095_headquarters.push(bad.bq095_headquarters[0].clone()),
        1=>bad.players[0].hand.push(bad.bq095_headquarters[0].cards[0].clone()),
        2=>bad.bq095_headquarters[0].cards[0].owner=1,
        3=>bad.bq095_headquarters[0].cards[0].controller=1,
        4=>bad.bq095_headquarters[0].cards[0].damage=1,
        5=>bad.bq095_headquarters[0].id=bad.bq095_headquarters[0].cards[0].id.clone(),
        6=>bad.bq095_headquarters[0].source_instance="i999999999".into(),
        _=>bad.bq095_headquarters[0].seat=99,
    }assert!(crate::model::Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"case {case}");
    let room=crate::room::RoomEnvelope::from_game(bad);
    assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err(),"room case {case}");}
}

#[test]
fn bq095_two_owned_headquarters_do_not_follow_changed_control() {
    let mut g=game();let a=source(&mut g,0,0);let b=source(&mut g,1,1);
    let za=g.bq095_create_or_reuse(&a,"society:p0").unwrap();let zb=g.bq095_create_or_reuse(&b,"society:p1").unwrap();assert_ne!(za,zb);
    let mut c=g.make_card("CQ09",0);c.controller=1;
    assert_eq!(g.bq095_own_reference(&c),D::Headquarters(za));
    assert_eq!(Headquarters::ordinary_grave_reference(1),D::Graveyard(1));
    crate::unit_support::restore(&mut g);
}

#[test]
fn bq095_actual_discard_routes_owner_destination_and_only_owned_text_observes() {
    use crate::{model::*,unit_support::*};
    let mut g=game();let s=source(&mut g,0,0);g.bq095_create_or_reuse(&s,"society:p0").unwrap();
    let own=field(&mut g,"BQ079",0);let foreign=field(&mut g,"BQ079",2);g.board_mut(&foreign).unwrap().controller=0;
    let mut corpse=g.make_card("XQ46",0);let old=corpse.id.clone();corpse.controller=2;g.players[2].hand.push(corpse);
    g.discard_hand_card(2,&old).unwrap();
    assert!(g.players.iter().all(|p|p.graveyard.is_empty()));assert_eq!(g.bq095_headquarters[0].cards.len(),1);
    assert_ne!(g.bq095_headquarters[0].cards[0].id,old);
    for _ in 0..32 {if g.pending.is_some(){choose(&mut g,vec!["accept".into()]);}else if !g.stack.is_empty(){pass(&mut g);}else if !g.effects.is_empty(){g.drive().unwrap();}else{break;}}
    assert_eq!(g.board(&own).unwrap().1.permanent_combat_markers,1);
    assert_eq!(g.board(&foreign).unwrap().1.permanent_combat_markers,0);
    restore(&mut g);let room=crate::room::RoomEnvelope::from_game(g);assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_ok());
}

#[test]
fn bq095_generated_departure_disappears_before_destination_replacement() {
    use crate::{model::*,unit_support::*};let mut g=game();let s=source(&mut g,0,0);g.bq095_create_or_reuse(&s,"society:p0").unwrap();
    let c=g.make_generated(crate::generated::GeneratedKind::Walker,0).unwrap();let id=c.id.clone();g.regions[0].cards.push(c);
    g.remove_dead(&id,RemovalCause::Destroy);g.drive().unwrap();
    assert!(g.board(&id).is_none());assert!(g.players[0].graveyard.is_empty());assert!(g.bq095_headquarters[0].cards.is_empty());restore(&mut g);
}

#[test]
fn bq095_reentry_does_not_hide_unexpected_physical_grave_and_elimination_clears_cards() {
    let mut g=game();let s=source(&mut g,0,0);g.bq095_create_or_reuse(&s,"society:p0").unwrap();
    let c=g.make_card("JC125",0);g.players[0].graveyard.push(c);let before=serde_json::to_value(&g).unwrap();
    assert!(g.bq095_create_or_reuse(&s,"society:p0").is_err());assert_eq!(serde_json::to_value(&g).unwrap(),before);
    let c=g.players[0].graveyard.pop().unwrap();g.commit_owner_grave(c,None);assert_eq!(g.bq095_headquarters[0].cards.len(),1);
    g.eliminate(0);assert!(g.bq095_headquarters[0].cards.is_empty());crate::unit_support::restore(&mut g);
}

#[test]
fn bq095_invalid_first_migration_is_atomic_across_other_physical_zones() {
    let mut g=game();let s=source(&mut g,0,0);let c=g.make_card("JC125",0);
    g.players[0].hand.push(c.clone());g.players[0].graveyard.push(c);
    let before=serde_json::to_value(&g).unwrap();
    assert!(g.bq095_create_or_reuse(&s,"society:p0").is_err());assert_eq!(serde_json::to_value(&g).unwrap(),before);
}

#[test]
fn bq095_prepared_room_discard_duplicate_and_cold_restore_keep_one_destination() {
    use crate::{model::*,room::*};let mut g=game();let s=source(&mut g,0,0);g.bq095_create_or_reuse(&s,"society:p0").unwrap();
    let c=g.make_card("JC125",0);let id=c.id.clone();g.players[0].hand.push(c);
    g.effects.push_back(Effect::Discard{seat:0,amount:1,redraw:false,optional:false});g.drive().unwrap();
    let p=g.pending.as_ref().unwrap();let room=RoomEnvelope::from_game(g.clone());
    let command=RoomCommand{command_id:"bq095-prepared-discard".into(),expected_version:room.revision,action:Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![id]),..Action::new("choose")}.into()};
    let result=room.transition(0,Some(command.clone()),0).unwrap();let restored=RoomEnvelope::from_persisted(&result.state).unwrap();
    assert!(restored.game.players[0].graveyard.is_empty());assert_eq!(restored.game.bq095_headquarters[0].cards.len(),1);
    let duplicate=restored.transition(0,Some(command),0).unwrap();assert_eq!(duplicate.state,result.state);assert!(!duplicate.changed&&duplicate.journal.is_empty());
}

#[test]
fn bq095_real_paid_attachment_entry_then_preparation_draw_two_discard_one() {
    use crate::{model::*,unit_support::*};let mut g=game();
    g.players[0].society_zone.card=Some(g.make_card("MSBQ03",0));fund(&mut g,0,"JC084",6);
    let corpse=g.make_card("JC125",0);g.players[0].graveyard.push(corpse);let id=held(&mut g,"BQ095",0);
    let action=g.legal_actions(0).into_iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_ref()==Some(&id)).expect("paid society attachment action").action;
    act(&mut g,0,action);
    for _ in 0..40 {if g.pending.is_some(){choose(&mut g,vec!["accept".into()]);}else if !g.stack.is_empty(){pass(&mut g);}else if !g.effects.is_empty(){g.drive().unwrap();}else{break;}}
    assert_eq!(g.bq095_headquarters.len(),1);assert_eq!(g.bq095_headquarters[0].cards.len(),1);assert!(g.players[0].graveyard.is_empty());
    let source=g.attachments.iter().find(|a|a.card.definition=="BQ095").unwrap().card.id.clone();
    let before=g.players[0].hand.len();g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();
    choose(&mut g,vec![source]);
    for _ in 0..32 {if g.pending.is_some(){break;}if !g.stack.is_empty(){pass(&mut g);}else{g.drive().unwrap();}}
    assert_eq!(g.players[0].hand.len(),before+2);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"discard");
    let discard=g.pending.as_ref().unwrap().choice.options[0].id.clone();choose(&mut g,vec![discard]);
    assert_eq!(g.players[0].hand.len(),before+1);assert_eq!(g.bq095_headquarters[0].cards.len(),2);assert!(g.players[0].graveyard.is_empty());restore(&mut g);
}

#[test]
fn bq095_actual_attachment_and_entry_frames_reject_wrong_or_missing_source() {
    use crate::{model::*,unit_support::*};let mut g=game();g.players[0].society_zone.card=Some(g.make_card("MSBQ03",0));fund(&mut g,0,"JC084",6);let id=held(&mut g,"BQ095",0);
    let a=g.legal_actions(0).into_iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_ref()==Some(&id)).unwrap().action;act(&mut g,0,a);
    let rejects=|bad:&Game|{assert!(Game::from_persisted(&serde_json::to_string(bad).unwrap()).is_err());let room=crate::room::RoomEnvelope::from_game(bad.clone());assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());};
    for case in 0..3 {let mut bad=g.clone();let item=bad.stack.last_mut().unwrap();match case {0=>item.frame=None,1=>item.frame.as_mut().unwrap().source.card.definition="JC001".into(),_=>item.frame.as_mut().unwrap().targets[0].id="society:p2".into()};rejects(&bad);}
    for _ in 0..32 {if g.pending.is_some(){break;}pass(&mut g);}choose(&mut g,vec!["accept".into()]);
    assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().ability_key,"bq095-entry");
    for case in 0..3 {let mut bad=g.clone();let f=bad.stack.last_mut().unwrap().frame.as_mut().unwrap();match case {0=>f.source.attachment_host_instance=Some("society:p2".into()),1=>f.source.card.definition="JC001".into(),_=>f.steps.clear()};rejects(&bad);}
    restore(&mut g);
}

#[test]
fn bq095_accepted_entry_survives_source_departure_and_retains_original_society() {
    use crate::{model::*,unit_support::*};let mut g=game();g.players[0].society_zone.card=Some(g.make_card("MSBQ03",0));fund(&mut g,0,"JC084",6);let id=held(&mut g,"BQ095",0);
    let a=g.legal_actions(0).into_iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_ref()==Some(&id)).unwrap().action;act(&mut g,0,a);
    for _ in 0..32 {if g.pending.is_some(){break;}pass(&mut g);}choose(&mut g,vec!["accept".into()]);
    let live=g.attachments.iter().find(|a|a.card.definition=="BQ095").unwrap().card.id.clone();g.remove_dead(&live,RemovalCause::Destroy);restore(&mut g);
    while !g.stack.is_empty(){pass(&mut g);}assert_eq!(g.bq095_headquarters.len(),1);assert_eq!(g.bq095_headquarters[0].seat,0);assert!(g.attachments.iter().all(|a|a.card.definition!="BQ095"));restore(&mut g);
}

#[test]
fn bq095_owned_reference_take_emits_only_matching_jz53_departure_and_restores() {
    use crate::{model::*,unit_support::*};let mut g=game();let s=source(&mut g,0,0);let corpse=g.make_card("JC125",0);g.players[0].graveyard.push(corpse);g.bq095_create_or_reuse(&s,"society:p0").unwrap();
    let owned=field(&mut g,"JZ53",0);let borrowed=field(&mut g,"JZ53",2);g.board_mut(&borrowed).unwrap().controller=0;
    let source=g.board(&owned).unwrap().1.clone();let id=g.bq095_headquarters[0].cards[0].id.clone();let c=g.bq095_take_reference(&source,&id).unwrap();let c=g.reset_zone_card(c);g.players[0].hand.push(c);
    let events=g.effects.iter().filter_map(|e|if let Effect::Declare{declaration:d}=e {if d.ability.event==Some(crate::rules::Event::JZ53GraveDeparture){Some(d)}else{None}}else{None}).collect::<Vec<_>>();
    assert_eq!(events.len(),1);assert_eq!(events[0].source.card.id,owned);
    let crate::rules::Op::JZ53Generate{departure:Some(d)}=&events[0].ability.ops[0]else{panic!()};assert_eq!(d.headquarters.as_deref(),Some(g.bq095_headquarters[0].id.as_str()));
    restore(&mut g);let room=crate::room::RoomEnvelope::from_game(g);assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_ok());
}

#[test]
fn bq095_bq079_prepare_selects_actual_headquarters_card_then_bottoms_it() {
    use crate::{model::*,unit_support::*};let mut g=game();let s=source(&mut g,0,0);let c=g.make_card("JC125",0);g.players[0].graveyard.push(c);g.bq095_create_or_reuse(&s,"society:p0").unwrap();let id=g.bq095_headquarters[0].cards[0].id.clone();let ghoul=field(&mut g,"BQ079",0);
    g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();choose(&mut g,vec![ghoul.clone()]);for _ in 0..32{if g.pending.is_some(){break;}pass(&mut g);}
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"batch3-bq079-bottom");assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id==id));choose(&mut g,vec![id.clone()]);
    assert!(g.bq095_headquarters[0].cards.is_empty());assert!(g.board(&ghoul).is_some());assert_eq!(g.players[0].deck.last().unwrap().definition,"JC125");assert_ne!(g.players[0].deck.last().unwrap().id,id);restore(&mut g);
}

#[test]
fn bq095_owned_ghoul_under_enemy_control_still_observes_owners_headquarters() {
    use crate::{model::*,unit_support::*};let mut g=game();let s=source(&mut g,0,0);g.bq095_create_or_reuse(&s,"society:p0").unwrap();
    let ghoul=field(&mut g,"BQ079",0);g.board_mut(&ghoul).unwrap().controller=2;
    let corpse=held(&mut g,"XQ46",0);g.discard_hand_card(0,&corpse).unwrap();
    let events=g.effects.iter().filter_map(|e|if let Effect::Declare{declaration:d}=e{Some(d)}else{None}).collect::<Vec<_>>();assert_eq!(events.len(),1);assert_eq!(events[0].actor,2);
    for _ in 0..32{if g.pending.is_some(){choose(&mut g,vec!["accept".into()]);}else if !g.stack.is_empty(){pass(&mut g);}else if !g.effects.is_empty(){g.drive().unwrap();}else{break;}}
    assert_eq!(g.board(&ghoul).unwrap().1.permanent_combat_markers,1);restore(&mut g);
}

#[test]
fn bq095_world_regions_are_not_player_owned_cards_for_printed_reference_rewrite() {
    let mut g=game();let s=source(&mut g,0,0);g.bq095_create_or_reuse(&s,"society:p0").unwrap();
    let region=&g.regions[0].card;assert_eq!(g.bq095_own_reference(region),D::Graveyard(region.controller));
}

#[test]
fn bq095_prepared_real_room_entry_and_prepare_export() {
    use crate::{model::*,unit_support::*,abi112_review_play_tests::Trace};
    let mut g=game();g.players[0].society_zone.card=Some(g.make_card("MSBQ03",0));fund(&mut g,0,"JC084",6);let c=g.make_card("JC125",0);g.players[0].graveyard.push(c);let id=held(&mut g,"BQ095",0);
    let a=g.legal_actions(0).into_iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_ref()==Some(&id)).unwrap().action;
    let mut t=Trace::new_preserving_rng(g,"bq095-prepared-paid-entry".into(),971);t.action(0,a);t.until_pending();t.choose(vec!["accept".into()]);
    for _ in 0..40 {if t.room.game.stack.is_empty()&&t.room.game.pending.is_none()&&t.room.pacing.window.is_none(){break;}t.pass();}
    assert_eq!(t.room.game.bq095_headquarters.len(),1);assert_eq!(t.room.game.bq095_headquarters[0].cards.len(),1);
    let mut g=t.room.game.clone();let source=g.attachments.iter().find(|a|a.card.definition=="BQ095").unwrap().card.id.clone();let hand=g.players[0].hand.len();g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();
    let mut p=Trace::new_preserving_rng(g,"bq095-prepared-prepare-draw-discard".into(),972);p.choose(vec![source]);p.until_pending();assert_eq!(p.room.game.players[0].hand.len(),hand+2);assert_eq!(p.room.game.pending.as_ref().unwrap().choice.kind,"discard");let id=p.room.game.pending.as_ref().unwrap().choice.options[0].id.clone();p.choose(vec![id]);assert_eq!(p.room.game.players[0].hand.len(),hand+1);assert_eq!(p.room.game.bq095_headquarters[0].cards.len(),2);
    let mut cases=vec![t.value(),p.value()];cases.extend(bq095_consumer_room_cases());for c in &mut cases {c["preparedNativeLayout"]=true.into();for s in c["steps"].as_array_mut().unwrap(){let r=crate::room::RoomEnvelope::from_persisted(s["expected"]["state"].as_str().unwrap()).unwrap();let dup=r.transition(s["seat"].as_u64().unwrap()as usize,Some(serde_json::from_value(s["command"].clone()).unwrap()),0).unwrap();assert_eq!(dup.state,s["expected"]["state"]);assert!(!dup.changed&&dup.journal.is_empty());s["duplicate"]=serde_json::to_value(dup).unwrap();}}
    use sha2::{Digest,Sha256};let mut bindings=crate::bq052_tests::source_bindings();for(n,b)in[("rust-game/src/bq095.rs",include_bytes!("bq095.rs").as_slice()),("rust-game/src/bq095_tests.rs",include_bytes!("bq095_tests.rs").as_slice()),("rust-game/src/jc035.rs",include_bytes!("jc035.rs").as_slice()),("rust-game/src/jz53.rs",include_bytes!("jz53.rs").as_slice()),("rust-game/src/batch3_ghoul.rs",include_bytes!("batch3_ghoul.rs").as_slice())]{bindings["files"][n]=format!("{:x}",Sha256::digest(b)).into();}
    let out=serde_json::json!({"catalog":crate::catalog::catalog(),"sourceBindings":bindings,"cases":cases});if let Ok(path)=std::env::var("BQ095_NATIVE_OUT"){std::fs::write(path,serde_json::to_vec(&out).unwrap()).unwrap();}
}

#[test]
fn bq095_changed_control_prepare_keeps_owner_zone_and_controller_choice() {
    use crate::{model::*,unit_support::*};let mut g=game();let s=source(&mut g,0,0);let c=g.make_card("JC125",0);g.players[0].graveyard.push(c);g.bq095_create_or_reuse(&s,"society:p0").unwrap();let id=g.bq095_headquarters[0].cards[0].id.clone();
    let ghoul=field(&mut g,"BQ079",0);g.board_mut(&ghoul).unwrap().controller=2;let dancer=field(&mut g,"JZ53",0);g.board_mut(&dancer).unwrap().controller=2;
    g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();assert_eq!(g.pending.as_ref().unwrap().seat,2);choose(&mut g,vec![ghoul]);for _ in 0..32{if g.pending.is_some(){break;}pass(&mut g);}
    assert_eq!(g.pending.as_ref().unwrap().seat,2);choose(&mut g,vec![id]);assert!(g.bq095_headquarters[0].cards.is_empty());let bottom=g.players[2].deck.last().unwrap();assert_eq!(bottom.definition,"JC125");assert_eq!(bottom.owner,0);
    restore(&mut g);assert_eq!(g.pending.as_ref().unwrap().seat,2);let room=crate::room::RoomEnvelope::from_game(g);assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_ok());
}

#[test]
fn bq095_bq061_current_headquarters_choice_recovers_one_and_rejects_old_id() {
 use crate::{model::*,unit_support::*};
 let mut g=game();fund(&mut g,0,"JC069",1);let play=held(&mut g,"BQ061",0);
 let c=g.make_card("BQ064",0);let old=c.id.clone();g.players[0].graveyard.push(c);
 let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();
 act(&mut g,0,Action{card_id:Some(play),ability_id:Some(crate::bq061::KEY.into()),..Action::new("play")});
 crate::jc029_tests::pass_top(&mut g);restore(&mut g);
 let p=g.pending.clone().unwrap();assert_eq!(p.choice.options.len(),1);let id=p.choice.options[0].id.clone();assert_ne!(id,old);
 reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![old]),..Action::new("choose")});
 choose(&mut g,vec![id.clone()]);assert!(g.players[0].hand.iter().any(|c|c.definition=="BQ064"&&c.id!=id));
 assert!(g.players[0].graveyard.is_empty());assert!(!g.bq095_headquarters[0].cards.iter().any(|c|c.id==id));restore(&mut g);
}

#[test]
fn bq095_msbq03_seals_actual_headquarters_corpse_after_restore() {
 use crate::{model::*,unit_support::*};
 let mut g=game();g.players[0].society_zone.card=Some(g.make_card("MSBQ03",0));fund(&mut g,0,"JC096",1);
 let c=g.make_card("BQ091",0);let host=c.id.clone();g.attachments.push(Attachment{card:c,host_id:g.regions[0].card.id.clone()});
 let c=g.make_card("XQ46",0);g.players[0].graveyard.push(c);let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();
 let source=g.players[0].society_zone.card.as_ref().unwrap().id.clone();
 act(&mut g,0,Action{card_id:Some(source),ability_id:Some(crate::msbq03::SEAL.into()),target_id:Some(host.clone()),..Action::new("activate")});
 crate::jc029_tests::pass_top(&mut g);restore(&mut g);let p=g.pending.clone().unwrap();assert_eq!(p.choice.options.len(),1);let id=p.choice.options[0].id.clone();choose(&mut g,vec![id]);
 assert!(g.bq095_headquarters[0].cards.is_empty());assert!(g.sealed_cards.iter().any(|c|c.host_id==host&&c.card.definition=="XQ46"));restore(&mut g);
}

#[test]
fn bq095_bound_own_grave_targets_follow_owner_not_control_and_generic_stays_physical() {
 use crate::{rules::*,unit_support::*};
 let mut g=game();let c=g.make_card("CQ27",0);g.players[0].graveyard.push(c);let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let id=g.bq095_headquarters[0].cards[0].id.clone();
 let sid=field(&mut g,"JZ53",0);g.board_mut(&sid).unwrap().controller=2;let (r,c)=g.in_play(&sid).unwrap();let s=g.source_snapshot(c,r);let spec=crate::jz53::definition().abilities[0].targets[0].clone();
 assert!(g.valid_binding(2,&s,&spec,&id));assert_eq!(g.binding_options(2,&s,&spec).iter().map(|o|o.id.as_str()).collect::<Vec<_>>(),[id.as_str()]);
 let public=g.public_target(2,&s,&spec,&id);assert_eq!(public.owner.as_deref(),Some("p0"));assert_eq!(public.controller.as_deref(),Some("p0"));
 let generic=target(Zone::Graveyard,EntityKind::Any,Relation::Any,Range::Anywhere);assert!(!g.valid_binding(2,&s,&generic,&id));assert!(g.binding_options(2,&s,&generic).is_empty());
 let mut borrowed=s.clone();borrowed.card.owner=2;borrowed.card.controller=0;assert!(!g.valid_binding(0,&borrowed,&spec,&id));
}

#[test]
fn bq095_jz53_real_entry_returns_headquarters_card_after_cold_restore() {
 use crate::{model::*,unit_support::*};
 let mut g=game();let c=g.make_card("CQ27",0);g.players[0].graveyard.push(c);let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let target=g.bq095_headquarters[0].cards[0].id.clone();
 fund(&mut g,0,"JC096",4);let id=held(&mut g,"JZ53",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});crate::jc029_tests::pass_top(&mut g);
 assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id==target));choose(&mut g,vec![target]);restore(&mut g);crate::jc029_tests::pass_top(&mut g);
 assert!(g.players[0].hand.iter().any(|c|c.definition=="CQ27"));assert!(g.bq095_headquarters[0].cards.is_empty());restore(&mut g);
}

#[test]
fn bq095_migration_invalidates_bound_grave_instance_without_retargeting() {
 use crate::{model::*,unit_support::*};
 let mut g=game();let c=g.make_card("CQ27",0);let target=c.id.clone();g.players[0].graveyard.push(c);
 fund(&mut g,0,"JC096",4);let id=held(&mut g,"JZ53",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});crate::jc029_tests::pass_top(&mut g);choose(&mut g,vec![target.clone()]);
 let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let new=g.bq095_headquarters[0].cards[0].id.clone();assert_ne!(target,new);restore(&mut g);
 let f=g.stack.last().unwrap().frame.as_ref().unwrap();assert!(!g.valid_bound_target(f.actor,&f.source,&f.targets[0]));crate::jc029_tests::pass_top(&mut g);
 assert!(!g.players[0].hand.iter().any(|c|c.definition=="CQ27"));assert!(g.bq095_headquarters[0].cards.iter().any(|c|c.id==new));restore(&mut g);
}

#[test]
fn bq095_cq27_intrinsic_headquarters_action_pays_and_restores_without_duplicate() {
 use crate::{model::*,unit_support::*};
 let mut g=game();let c=g.make_card("CQ27",0);g.players[0].graveyard.push(c);let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let id=g.bq095_headquarters[0].cards[0].id.clone();
 let ids=(0..3).map(|_|field(&mut g,"JC125",0)).collect::<Vec<_>>();let a=Action{card_id:Some(id.clone()),ability_id:Some(crate::cq27::GRAVE.into()),cost_selected:Some(ids.clone()),..Action::new("activate")};
 assert!(g.legal_actions(0).iter().any(|v|v.action==a));act(&mut g,0,a.clone());restore(&mut g);reject(&mut g,0,a);crate::jc029_tests::pass_top(&mut g);
 assert!(g.regions.iter().flat_map(|r|&r.cards).any(|c|c.definition=="CQ27"&&c.id!=id));assert_eq!(g.bq095_headquarters[0].cards.len(),3);assert!(g.players[0].graveyard.is_empty());restore(&mut g);
}

#[test]
fn bq095_xq35_intrinsic_headquarters_payment_and_departed_source_cancellation() {
 use crate::{model::*,unit_support::*};
 for remove in [false,true] {let mut g=game();fund(&mut g,0,"JC096",1);for def in ["XQ35","JC125"] {let c=g.make_card(def,0);g.players[0].graveyard.push(c);}let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();
 let id=g.bq095_headquarters[0].cards[0].id.clone();let cost=g.bq095_headquarters[0].cards[1].id.clone();let a=Action{card_id:Some(id.clone()),ability_id:Some(crate::xq35::KEY.into()),cost_selected:Some(vec![cost]),..Action::new("activate")};
 assert!(g.legal_actions(0).iter().any(|v|v.action==a));act(&mut g,0,a.clone());restore(&mut g);reject(&mut g,0,a);assert_eq!(g.sealed_cards.len(),1);
 if remove {let source=g.bq095_headquarters[0].cards[0].clone();let c=g.bq095_take_reference(&source,&id).unwrap();let c=g.reset_zone_card(c);g.players[0].hand.push(c);restore(&mut g);}
 crate::jc029_tests::pass_top(&mut g);assert_eq!(g.regions[0].cards.iter().any(|c|c.definition=="XQ35"&&c.exhausted),!remove);assert!(g.players[0].graveyard.is_empty());restore(&mut g);}
}

#[test]
fn bq095_jz51_seals_three_headquarters_corpses_and_enters_selected_region() {
 use crate::{model::*,unit_support::*};let mut g=game();for def in ["JZ51","XQ46","XQ46","XQ46"]{let c=g.make_card(def,0);g.players[0].graveyard.push(c);}let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let id=g.bq095_headquarters[0].cards[0].id.clone();
 let a=Action{card_id:Some(id.clone()),ability_id:Some(crate::batch4_wendigo::GRAVE.into()),..Action::new("activate")};assert!(g.legal_actions(0).iter().any(|v|v.action==a));act(&mut g,0,a);restore(&mut g);
 let ids=g.pending.as_ref().unwrap().choice.options.iter().map(|o|o.id.clone()).collect();choose(&mut g,ids);assert_eq!(g.sealed_cards.len(),3);restore(&mut g);crate::jc029_tests::pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"jz51-entry-region");choose(&mut g,vec!["region:0".into()]);assert!(g.regions[0].cards.iter().any(|c|c.definition=="JZ51"&&c.id!=id));assert!(g.bq095_headquarters[0].cards.is_empty());restore(&mut g);
}

#[test]
fn bq095_msjc16_seven_payment_and_actual_headquarters_revival() {
 use crate::{model::*,unit_support::*};let mut g=game();g.players[0].society_zone.card=Some(g.make_card("MSJC16",0));fund(&mut g,0,"JC096",6);for def in ["JC044","JC125","JC125","JC125","JC125","JC125","JC125","JC125"]{let c=g.make_card(def,0);g.players[0].graveyard.push(c);}let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let target=g.bq095_headquarters[0].cards[0].id.clone();let ids=g.bq095_headquarters[0].cards[1..].iter().map(|c|c.id.clone()).collect();let sid=g.players[0].society_zone.card.as_ref().unwrap().id.clone();
 act(&mut g,0,Action{card_id:Some(sid),ability_id:Some(crate::msjc16::REVIVE.into()),target_id:Some(target),region:Some(0),..Action::new("activate")});restore(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.options.len(),8);choose(&mut g,ids);assert_eq!(g.sealed_cards.len(),7);restore(&mut g);crate::jc029_tests::pass_top(&mut g);assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC044"));assert!(g.bq095_headquarters[0].cards.is_empty());restore(&mut g);
}

#[test]
fn bq095_intrinsic_jc085_normal_deploy_from_headquarters_pays_once() {
 use crate::{model::*,unit_support::*};let mut g=game();fund(&mut g,0,"JC085",2);let c=g.make_card("JC085",0);g.players[0].graveyard.push(c);let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let id=g.bq095_headquarters[0].cards[0].id.clone();let a=Action{card_id:Some(id.clone()),region:Some(0),..Action::new("deploy")};assert!(g.legal_actions(0).iter().any(|v|v.action==a));act(&mut g,0,a.clone());assert_eq!(g.resources(0),0);restore(&mut g);reject(&mut g,0,a);crate::jc029_tests::pass_top(&mut g);assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC085"&&c.id!=id));assert!(g.bq095_headquarters[0].cards.is_empty());restore(&mut g);
}

fn bq095_consumer_room_cases()->Vec<serde_json::Value> {
 use crate::{model::*,unit_support::*,abi112_review_play_tests::Trace};let mut cases=vec![];
 for (n,def) in ["CQ27","XQ35","JZ51","MSJC16","JC085"].iter().enumerate() {
  let mut g=game();let defs=match *def {"CQ27"=>vec!["CQ27"],"XQ35"=>vec!["XQ35","JC125"],"JZ51"=>vec!["JZ51","XQ46","XQ46","XQ46"],"MSJC16"=>vec!["JC044","JC125","JC125","JC125","JC125","JC125","JC125","JC125"],_=>vec!["JC085"]};
  if *def=="MSJC16" {g.players[0].society_zone.card=Some(g.make_card(def,0));}
  fund(&mut g,0,if *def=="JC085"{"JC085"}else{"JC096"},if *def=="MSJC16"{6}else if *def=="JC085"{2}else{1});
  for d in defs {let c=g.make_card(d,0);g.players[0].graveyard.push(c);}let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let id=g.bq095_headquarters[0].cards[0].id.clone();
  let action=match *def {
   "CQ27"=>Action{card_id:Some(id),ability_id:Some(crate::cq27::GRAVE.into()),cost_selected:Some((0..3).map(|_|field(&mut g,"JC125",0)).collect()),..Action::new("activate")},
   "XQ35"=>Action{card_id:Some(id),ability_id:Some(crate::xq35::KEY.into()),cost_selected:Some(vec![g.bq095_headquarters[0].cards[1].id.clone()]),..Action::new("activate")},
   "JZ51"=>Action{card_id:Some(id),ability_id:Some(crate::batch4_wendigo::GRAVE.into()),..Action::new("activate")},
   "MSJC16"=>Action{card_id:Some(g.players[0].society_zone.card.as_ref().unwrap().id.clone()),ability_id:Some(crate::msjc16::REVIVE.into()),target_id:Some(id),region:Some(0),..Action::new("activate")},
   _=>Action{card_id:Some(id),region:Some(0),..Action::new("deploy")},
  };
  let mut t=Trace::new_preserving_rng(g,format!("bq095-prepared-{def}-headquarters"),980+n);t.action(0,action);
  for _ in 0..100 {
   if let Some(p)=t.room.game.pending.clone(){let ids=match p.choice.kind.as_str(){"jz51-seal-payment"=>p.choice.options.iter().take(3).map(|o|o.id.clone()).collect(),"jz51-entry-region"=>vec!["region:0".into()],"msjc16-seven"=>p.choice.options.iter().filter(|o|t.room.game.bq095_headquarters[0].cards.iter().any(|c|c.id==o.id&&c.definition=="JC125")).map(|o|o.id.clone()).collect(),_=>vec![]};t.choose(ids);}
   else if t.room.game.stack.is_empty()&&t.room.game.effects.is_empty()&&t.room.pacing.window.is_none(){break;}
   else{t.pass();}
  }
  let entered=if *def=="MSJC16"{"JC044"}else{def};assert!(t.room.game.regions.iter().flat_map(|r|&r.cards).any(|c|c.definition==entered),"{def} actual Room chain did not enter");assert!(t.room.game.players[0].graveyard.is_empty());assert!(t.room.game.pending.is_none());cases.push(t.value());
 }
 cases
}

#[test]
fn bq095_borrowed_jz53_returns_owner_headquarters_card_to_actors_hand() {
 use crate::{unit_support::*,model::*};let mut g=game();let c=g.make_card("JC085",0);g.players[0].graveyard.push(c);let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let target=g.bq095_headquarters[0].cards[0].id.clone();let id=field(&mut g,"JZ53",0);g.board_mut(&id).unwrap().controller=2;g.enter_triggers(2,"JZ53",&id,false);g.drive().unwrap();choose(&mut g,vec![target]);restore(&mut g);crate::jc029_tests::pass_top(&mut g);assert!(g.players[2].hand.iter().any(|c|c.definition=="JC085"&&c.owner==0));assert!(!g.players[0].hand.iter().any(|c|c.definition=="JC085"));restore(&mut g);
 while let Some(p)=g.pending.clone(){let ids=p.choice.options.iter().take(p.choice.min.unwrap_or(0)).map(|o|o.id.clone()).collect();choose(&mut g,ids);if !g.stack.is_empty(){crate::jc029_tests::pass_top(&mut g);}}
 let id=g.players[2].hand.iter().find(|c|c.definition=="JC085").unwrap().id.clone();fund(&mut g,2,"JC085",2);g.begin_window(Window::Action(1));act(&mut g,2,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});restore(&mut g);crate::jc029_tests::pass_top(&mut g);assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC085"&&c.owner==0&&c.controller==2));restore(&mut g);
}

#[test]
fn bq095_bq078_death_returns_headquarters_corpse_via_actual_choice() {
 use crate::{unit_support::*,model::*};let mut g=game();let c=g.make_card("XQ46",0);g.players[0].graveyard.push(c);let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let target=g.bq095_headquarters[0].cards[0].id.clone();let id=field(&mut g,"BQ078",0);g.remove_dead(&id,RemovalCause::Destroy);g.drive().unwrap();choose(&mut g,vec!["accept".into()]);crate::jc029_tests::pass_top(&mut g);restore(&mut g);choose(&mut g,vec![target]);assert!(g.players[0].hand.iter().any(|c|c.definition=="XQ46"));restore(&mut g);
}

#[test]
fn bq095_bq081_actual_headquarters_choice_bottoms_before_attachment_destruction() {
 use crate::{unit_support::*,model::*};let mut g=game();let c=g.make_card("JC125",0);g.players[0].graveyard.push(c);let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let target=g.bq095_headquarters[0].cards[0].id.clone();let c=g.make_card("JC033",2);let host=c.id.clone();g.attachments.push(Attachment{card:c,host_id:"society:p2".into()});let id=field(&mut g,"BQ081",0);g.enter_triggers(0,"BQ081",&id,false);g.drive().unwrap();choose(&mut g,vec![host.clone()]);crate::jc029_tests::pass_top(&mut g);restore(&mut g);choose(&mut g,vec![target]);assert_eq!(g.players[0].deck.last().unwrap().definition,"JC125");assert!(!g.attachments.iter().any(|a|a.card.id==host));restore(&mut g);
}

#[test]
fn bq095_cq25_search_seals_headquarters_ghost_without_searching_deck() {
 use crate::{unit_support::*,model::*};let mut g=game();let c=g.make_card("XG25",0);g.players[0].graveyard.push(c);let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let target=g.bq095_headquarters[0].cards[0].id.clone();let tiger=field(&mut g,"CQ25",0);let victim=field(&mut g,"JC125",2);g.remove_dead(&victim,RemovalCause::Destroy);g.drive().unwrap();choose(&mut g,vec!["accept".into()]);crate::jc029_tests::pass_top(&mut g);restore(&mut g);let rng=g.random;choose(&mut g,vec![target]);assert_eq!(rng,g.random);assert!(g.sealed_cards.iter().any(|s|s.host_id==tiger&&s.card.definition=="XG25"));restore(&mut g);
}

#[test]
fn bq095_bq085_prepare_orders_two_actual_headquarters_cards() {
 use crate::{unit_support::*,model::*};let mut g=game();for def in ["JC125","JZ50"]{let c=g.make_card(def,0);g.players[0].graveyard.push(c);}let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let ids=g.bq095_headquarters[0].cards.iter().map(|c|c.id.clone()).collect();let id=field(&mut g,"BQ085",0);g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();choose(&mut g,vec![id]);crate::jc029_tests::pass_top(&mut g);restore(&mut g);choose(&mut g,ids);assert_eq!(g.players[0].deck[g.players[0].deck.len()-2..].iter().map(|c|c.definition.as_str()).collect::<Vec<_>>(),["JC125","JZ50"]);assert!(g.bq095_headquarters[0].cards.is_empty());restore(&mut g);
}

#[test]
fn bq095_cq26_threshold_and_real_mill_observer_distinguish_owned_and_borrowed_text() {
 use crate::{unit_support::*,model::*};for owned in [true,false] {let mut g=game();for _ in 0..6{let c=g.make_card("JC125",0);g.players[0].graveyard.push(c);}let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let id=field(&mut g,"CQ26",if owned{0}else{2});g.board_mut(&id).unwrap().controller=0;deck_cards(&mut g,0,&["JC125"]);g.mill_cards_with_cause(0,1,None).unwrap();g.drive().unwrap();assert_eq!(g.bq095_headquarters[0].cards.len(),7);assert_eq!(g.black_next20_icon_bonus(g.board(&id).unwrap().1,0).0.combat,u32::from(owned));assert_eq!(g.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Declare{declaration:d,..}if d.source.card.definition=="CQ26")),owned);restore(&mut g);}
}

#[test]
fn bq095_jz52_owned_headquarters_grant_under_foreign_control_freezes_actual_zone() {
 use crate::{unit_support::*,model::*};let mut g=game();let c=g.make_card("JC085",0);g.players[0].graveyard.push(c);let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let target=g.bq095_headquarters[0].cards[0].id.clone();let grant=field(&mut g,"JZ52",0);g.board_mut(&grant).unwrap().controller=2;fund(&mut g,2,"JC085",2);g.begin_window(Window::Action(1));let a=Action{card_id:Some(target),region:Some(0),..Action::new("deploy")};assert!(g.legal_actions(2).iter().any(|v|v.action==a));act(&mut g,2,a);restore(&mut g);
 for case in 0..3 {let mut bad=g.clone();let f=bad.stack.last_mut().unwrap().frame.as_mut().unwrap();let Some(PlaySource::HeadquartersByJZ52{zone,grant})=&mut f.source.play_source else{panic!()};match case{0=>*zone="i999999".into(),1=>grant.owner=1,_=>f.source.card.owner=1}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());let mut r=crate::jc029_tests::envelope(&g);r.game=bad;assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).is_err());}
 g.remove_dead(&grant,RemovalCause::Destroy);restore(&mut g);crate::jc029_tests::pass_top(&mut g);assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC085"&&c.owner==0&&c.controller==2));restore(&mut g);
}

#[test]
fn bq095_jc028_sacrifice_receipt_follows_actual_headquarters_destination() {
 use crate::{unit_support::*,model::*};let mut g=game();let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();let host=field(&mut g,"JC029",0);let id=field(&mut g,"JC028",0);act(&mut g,0,Action{card_id:Some(id),ability_id:Some(crate::jc028::KEY.into()),target_id:Some(host.clone()),..Action::new("activate")});assert!(g.bq095_headquarters[0].cards.iter().any(|c|c.definition=="JC028"));restore(&mut g);crate::jc029_tests::pass_top(&mut g);assert!(g.attachments.iter().any(|a|a.host_id==host&&a.card.definition==crate::jc028::FORM));assert!(g.bq095_headquarters[0].cards.is_empty());restore(&mut g);
}

#[test]
fn bq095_partial_mill_receipt_survives_real_rumor_pause_and_duplicate_rejection() {
 use crate::{unit_support::*,model::*};let mut g=game();let h=source(&mut g,0,0);g.bq095_create_or_reuse(&h,"society:p0").unwrap();field(&mut g,"JC125",0);fund(&mut g,0,"WM057",4);let id=held(&mut g,"CQ28",0);deck_cards(&mut g,0,&["JC125","WM042","JC125","JC125","JC125","JC125","JC125","JC125"]);
 act(&mut g,0,Action{card_id:Some(id),..Action::new("play")});choose(&mut g,vec!["item-0".into()]);choose(&mut g,vec!["item-3".into()]);crate::jc029_tests::pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"rumor");assert_eq!(g.bq095_headquarters[0].cards.len(),1);restore(&mut g);let a=Action{choice_id:Some(g.pending.as_ref().unwrap().choice.id.clone()),selected:Some(vec![]),..Action::new("choose")};act(&mut g,0,a.clone());reject(&mut g,0,a);assert!(g.bq095_headquarters[0].cards.len()>=6);assert!(g.players[0].graveyard.is_empty());restore(&mut g);
}

#[test]
fn bq095_natural_fifty_card_mill_headquarters_prepare_and_redeploy() {
 use crate::{catalog,deck,model::*,abi112_review_play_tests::Trace};
 let mut d=deck::preset("watchers").unwrap();d.id="closed-headquarters".into();d.name="总部自然闭合验证".into();d.description="非官方预组；私有候选注册表".into();d.society_id=Some("MSBQ03".into());d.cards=vec![catalog::DeckEntry{card_id:"XQ46".into(),count:20},catalog::DeckEntry{card_id:"JC096".into(),count:3},catalog::DeckEntry{card_id:"JC085".into(),count:3},catalog::DeckEntry{card_id:"BQ093".into(),count:3},catalog::DeckEntry{card_id:"BQ095".into(),count:3}];for id in ["JC086","JC084","JC088","JC087","JC094","JC095"]{d.cards.push(catalog::DeckEntry{card_id:id.into(),count:3});}
 assert!(deck::validate(d.clone()).is_err());let mut cards=catalog::catalog().cards.clone();cards.iter_mut().find(|c|c.id=="BQ095").unwrap().supported=true;let mut societies=catalog::catalog().societies.clone();societies.iter_mut().find(|s|s.card.id=="MSBQ03").unwrap().card.supported=true;let d=deck::validate_with_registry(d,&cards,&societies).unwrap();let mut other=d.clone();other.society_id=None;other.cards=vec![catalog::DeckEntry{card_id:"JC125".into(),count:50}];let other=deck::validate(other).unwrap();
 let(seed,g)=(1..4097).find_map(|seed|{let mut g=Game::new("headquarters-natural".into(),"LOCAL".into(),"teams".into(),"P0".into(),"watchers".into(),seed).unwrap();for seat in 1..4{g.join(format!("P{seat}"),"watchers".into()).unwrap();}g.start_validated_decks(vec![d.clone(),other.clone(),other.clone(),other.clone()]).unwrap();g.drive().unwrap();["BQ095","BQ093","JC085","JC096"].iter().all(|id|g.players[0].hand.iter().any(|c|c.definition==*id)).then_some((seed,g))}).expect("bounded unmodified legal opening seed");
 let mut t=Trace::new_preserving_rng(g,"bq095-natural-fifty-mill-entry-prepare-redeploy".into(),999);let(mut migrated,mut prepared,mut discarded,mut deployed)=(false,false,false,false);let mut preparation_hand=None;
 for _ in 0..2000 {
  if let Some(p)=t.room.game.pending.clone() {
   if matches!(&p.resolution,ChoiceResolution::Forecast{..}|ChoiceResolution::Frame{choice:FrameChoice::Forecast{..},..}) {let top=p.choice.options.iter().map(|o|o.id.clone()).collect();t.action(p.seat,Action{choice_id:Some(p.choice.id),top:Some(top),bottom:Some(vec![]),..Action::new("choose")});continue;}
   let ids=if matches!(&p.resolution,ChoiceResolution::Declare{declaration:d,..}if matches!(d.source.card.definition.as_str(),"BQ095"|"BQ093")) {vec!["accept".into()]}
    else if p.choice.kind=="jc035-preparation-order"&&p.seat==0 {let h=p.choice.options.iter().find(|o|t.room.game.attachments.iter().any(|a|a.card.id==o.id&&a.card.definition=="BQ095"));if let Some(h)=h{preparation_hand=Some(t.room.game.players[0].hand.len());prepared=true;vec![h.id.clone()]}else{p.choice.options.iter().take(p.choice.min.unwrap_or(0)).map(|o|o.id.clone()).collect()}}
    else if p.choice.kind=="discard"&&p.seat==0&&preparation_hand.is_some()&&!discarded {let id=p.choice.options.iter().find(|o|t.room.game.players[0].hand.iter().any(|c|c.id==o.id&&c.definition=="JC085")).expect("reserved JC085 for actual headquarters discard").id.clone();discarded=true;vec![id]}
    else {let mut options=p.choice.options.iter().collect::<Vec<_>>();options.sort_by_key(|o|t.room.game.players[p.seat].hand.iter().any(|c|c.id==o.id&&matches!(c.definition.as_str(),"BQ095"|"JC085"|"BQ093"|"JC096")));options.into_iter().take(p.choice.min.unwrap_or(0)).map(|o|o.id.clone()).collect()};
   let check=discarded&&preparation_hand.is_some()&&p.choice.kind=="discard"&&p.seat==0;t.choose(ids);if check{assert_eq!(t.room.game.players[0].hand.len(),preparation_hand.take().unwrap()+1);}continue;
  }
  if deployed&&t.room.game.stack.is_empty()&&t.room.game.effects.is_empty()&&t.room.pacing.window.is_none(){break;}
  if !t.room.game.stack.is_empty()||t.room.pacing.window.is_some(){t.pass();continue;}
  let g=&t.room.game;let legal=g.legal_actions(0);let mut a=None;
  if !g.attachments.iter().any(|a|a.card.definition=="BQ093"&&a.card.controller==0) {a=legal.iter().find(|a|a.action.kind=="play"&&g.players[0].hand.iter().any(|c|Some(&c.id)==a.action.card_id.as_ref()&&c.definition=="BQ093")).map(|a|a.action.clone());}
  if a.is_none()&&g.bq095_headquarters.is_empty()&&!g.players[0].graveyard.is_empty() {a=legal.iter().find(|a|a.action.kind=="play"&&g.players[0].hand.iter().any(|c|Some(&c.id)==a.action.card_id.as_ref()&&c.definition=="BQ095")).map(|a|a.action.clone());if a.is_some(){migrated=true;}}
  if a.is_none()&&discarded&&!deployed {a=legal.iter().find(|a|a.action.kind=="deploy"&&g.bq095_headquarters.iter().flat_map(|z|&z.cards).any(|c|Some(&c.id)==a.action.card_id.as_ref()&&c.definition=="JC085")).map(|a|a.action.clone());if a.is_some(){deployed=true;}}
  if a.is_none()&&(g.players[0].assets.len()<6||!g.loyalty(0,"BQ095")) {a=legal.iter().filter(|a|a.action.kind=="asset"&&g.players[0].hand.iter().any(|c|Some(&c.id)==a.action.card_id.as_ref()&&(matches!(c.definition.as_str(),"XQ46"|"JC096"|"JC086"|"JC084"|"JC088"|"JC087"|"JC094"|"JC095")||(matches!(c.definition.as_str(),"BQ093"|"BQ095")&&g.players[0].hand.iter().filter(|v|v.definition==c.definition).count()>1)))).min_by_key(|a|g.players[0].hand.iter().find(|c|Some(&c.id)==a.action.card_id.as_ref()).is_none_or(|c|catalog::card(&c.definition).color!="黑")).map(|a|a.action.clone());}
  if let Some(a)=a{t.action(0,a);}else{t.pass();}
 }
 assert!(migrated&&prepared&&discarded&&deployed,"natural stages: migration={migrated} prepare={prepared} discard={discarded} deploy={deployed}, turn={}",t.room.game.turn);assert!(t.room.game.regions.iter().flat_map(|r|&r.cards).any(|c|c.definition=="JC085"&&c.controller==0));assert!(t.room.game.players[0].graveyard.is_empty());assert!(!catalog::card("BQ095").supported);
 let mut v=t.value();v["construction"]=serde_json::json!({"seed":seed,"deck":d,"ordinaryCards":50,"privateCandidateRegistryOnly":true,"noLayoutOrResourceMutation":true});for s in v["steps"].as_array_mut().unwrap(){let r=crate::room::RoomEnvelope::from_persisted(s["expected"]["state"].as_str().unwrap()).unwrap();let dup=r.transition(s["seat"].as_u64().unwrap()as usize,Some(serde_json::from_value(s["command"].clone()).unwrap()),0).unwrap();assert!(!dup.changed&&dup.journal.is_empty());assert_eq!(dup.state,s["expected"]["state"]);s["duplicate"]=serde_json::to_value(dup).unwrap();}
 println!("BQ095_NATURAL seed={seed} commands={}",v["steps"].as_array().unwrap().len());
 if let Ok(path)=std::env::var("BQ095_NATURAL_OUT"){use sha2::{Digest,Sha256};let mut b=crate::bq052_tests::source_bindings();for(n,bytes)in[("rust-game/src/bq095.rs",include_bytes!("bq095.rs").as_slice()),("rust-game/src/bq095_tests.rs",include_bytes!("bq095_tests.rs").as_slice())]{b["files"][n]=serde_json::json!(format!("{:x}",Sha256::digest(bytes)));}std::fs::write(path,serde_json::to_vec(&serde_json::json!({"catalog":catalog::catalog(),"sourceBindings":b,"cases":[v]})).unwrap()).unwrap();}
}
