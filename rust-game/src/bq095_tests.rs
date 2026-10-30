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
