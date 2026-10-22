//! Explicit current-native fixtures. Each normal command restores exact opaque
//! state and all four player projections before/after through unit_support.
use crate::{batch4_skull as skull, catalog, model::*, rules::*, unit_support::*};

fn setup() -> (Game,String) {
    let mut g=game(); fund(&mut g,0,"JZ40",4);let id=held(&mut g,"JZ40",0);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("play")});top(&mut g);
    let source=g.attachments.iter().find(|a|a.card.definition=="JZ40").unwrap().card.id.clone();
    (g,source)
}
fn fire(g:&mut Game,seat:usize,source:&str,target:&str){
    act(g,seat,Action{card_id:Some(source.into()),ability_id:Some(skull::KEY.into()),target_id:Some(target.into()),..Action::new("activate")});
}
#[test]
fn batch4_skull_exact_source_and_closed_definition_reject_transplants() {
    let c=catalog::card("JZ40");assert_eq!((c.cost,c.kind.as_str(),c.magic.as_str(),c.supported),(4,"attachment","死亡",false));
    assert_eq!(c.subtypes,["法器"]);assert_eq!(c.permanent_icons,Icons{investigation:1,combat:0,influence:1});
    let d=skull::definition();assert_eq!(d.abilities.len(),2);
    for n in 0..4 {let mut wrong=d.clone();match n {0=>wrong.abilities[1].timing=Timing::Fast,1=>wrong.abilities[1].costs.clear(),2=>wrong.abilities[1].targets[0].predicate=None,_=>wrong.attachment.as_mut().unwrap().host.zone=Zone::Board};assert!(crate::batch4::validate_definition("JZ40",&wrong).is_err());}
    let mut wrong=d.abilities[1].clone();wrong.key="foreign".into();wrong.ops=vec![Op::ForEachLivingPlayer(vec![Op::JZ40TimeAndDestroyHost])];assert!(crate::rules::validate_ability("JC125",&wrong).is_err());
}
#[test]
fn batch4_skull_actual_attach_pays_four_and_projects_original_region() {
    let (mut g,id)=setup();assert_eq!(g.resources(0),0);
    assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().host_id,g.regions[0].card.id);
    for seat in 0..4 {let v=g.view(seat);let c=&v.attachments.iter().find(|a|a.card.instance_id==id).unwrap().card;assert_eq!(c.card_id.as_deref(),Some("JZ40"));}
    restore(&mut g);
}
#[test]
fn batch4_skull_real_sacrifice_is_paid_once_before_response_and_adds_one_below_four() {
    let (mut g,source)=setup();let target=field(&mut g,"JC125",2);let host=g.regions[0].card.id.clone();
    fire(&mut g,0,&source,&target);assert!(!g.attachments.iter().any(|a|a.card.id==source));
    assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JZ40").count(),1);
    let f=g.stack.last().unwrap().frame.as_ref().unwrap();assert_eq!(f.source.source_region_instance.as_deref(),Some(host.as_str()));assert_eq!(f.source.attachment_host_instance.as_deref(),Some(host.as_str()));
    top(&mut g);assert_eq!(g.board(&target).unwrap().1.time_markers,1);assert!(!g.regions[0].destroyed);restore(&mut g);
}
#[test]
fn batch4_skull_distinct_character_carriers_reach_four_without_removing_region() {
    let (mut g,source)=setup();let target=field(&mut g,"JC125",0);g.board_mut(&target).unwrap().time_markers=2;let other=field(&mut g,"JC125",2);g.board_mut(&other).unwrap().time_markers=1;
    let host=g.regions[0].card.id.clone();fire(&mut g,0,&source,&target);top(&mut g);
    assert!(g.regions[0].destroyed);assert_eq!(g.regions[0].card.id,host);assert!(g.board(&target).is_some());
    for seat in 0..4 {assert!(g.view(seat).regions[0].destroyed);}restore(&mut g);
}
#[test]
fn batch4_skull_sacrificed_carrier_markers_are_not_left_in_region_total() {
    let (mut g,source)=setup();let target=field(&mut g,"JC125",0);
    g.attachments.iter_mut().find(|a|a.card.id==source).unwrap().card.time_markers=4;
    fire(&mut g,0,&source,&target);top(&mut g);assert_eq!(g.physical_region_time_markers(0),1);assert!(!g.regions[0].destroyed);restore(&mut g);
}
#[test]
fn batch4_skull_remote_target_does_not_retarget_destruction_away_from_attached_region() {
    for original in [3,4] {let (mut g,source)=setup();let carrier=field(&mut g,"JC125",0);g.board_mut(&carrier).unwrap().time_markers=original;
      let mut c=g.make_card("JC125",2);c.time_markers=3;let target=c.id.clone();g.regions[1].cards.push(c);
      fire(&mut g,0,&source,&target);top(&mut g);assert_eq!(g.board(&target).unwrap().1.time_markers,4);assert_eq!(g.regions[0].destroyed,original==4);assert!(!g.regions[1].destroyed);restore(&mut g);
    }
}
#[test]
fn batch4_skull_self_target_lost_to_actual_payment_cancels_the_whole_effect_without_refund() {
    let (mut g,source)=setup();let carrier=field(&mut g,"JC125",0);g.board_mut(&carrier).unwrap().time_markers=4;
    fire(&mut g,0,&source,&source);top(&mut g);assert!(!g.regions[0].destroyed);
    assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JZ40").count(),1);restore(&mut g);
}
#[test]
fn batch4_skull_replaced_host_slot_cannot_destroy_the_new_region() {
    let (mut g,source)=setup();let mut c=g.make_card("JC125",2);let target=c.id.clone();c.time_markers=3;g.regions[1].cards.push(c);
    fire(&mut g,0,&source,&target);
    // Disclosed response-bound replacement fixture, not a claimed command test.
    g.regions[0].card=g.make_card("DQJC114",0);let new_carrier=field(&mut g,"JC125",0);g.board_mut(&new_carrier).unwrap().time_markers=9;
    restore(&mut g);top(&mut g);assert_eq!(g.board(&target).unwrap().1.time_markers,4);assert!(!g.regions[0].destroyed);restore(&mut g);
}
#[test]
fn batch4_skull_hidden_character_assets_and_forged_payloads_reject_before_payment() {
    let (mut g,source)=setup();let id=field(&mut g,"JC125",2);g.board_mut(&id).unwrap().face_down=true;
    let base=Action{card_id:Some(source.clone()),ability_id:Some(skull::KEY.into()),target_id:Some(id),..Action::new("activate")};reject(&mut g,0,base.clone());
    let asset=g.players[0].assets.first().map(|c|c.id.clone()).unwrap_or_else(||{let c=g.make_card("JC125",0);let id=c.id.clone();g.players[0].assets.push(c);id});
    reject(&mut g,0,Action{target_id:Some(asset),..base.clone()});
    let role=field(&mut g,"JC125",0);
    for a in [Action{target_id:Some(role.clone()),option:Some("4".into()),..base.clone()},Action{target_id:Some(role),cost_selected:Some(vec![source]),..base}] {reject(&mut g,0,a);}
}
#[test]
fn batch4_skull_secret_plan_is_an_attachment_and_time_does_not_reveal_its_identity() {
    let (mut g,source)=setup();g.players[0].society_zone.secrecy=1;fund(&mut g,0,"WM045",1);let id=held(&mut g,"WM045",0);
    act(&mut g,0,Action{card_id:Some(id),target_id:Some("society:p0".into()),option:Some("secret-plan".into()),..Action::new("play")});top(&mut g);
    let target=g.attachments.iter().find(|a|a.card.definition=="WM045").unwrap().card.id.clone();
    fire(&mut g,0,&source,&target);top(&mut g);assert!(g.attachments.iter().find(|a|a.card.id==target).unwrap().card.face_down);
    for seat in 0..4 {let v=g.view(seat);let c=&v.attachments.iter().find(|a|a.card.instance_id==target).unwrap().card;assert_eq!(c.time_markers,Some(1));if seat!=0 {assert!(c.card_id.is_none());assert_eq!(c.name,"秘密计划");}}
    restore(&mut g);
}
#[test]
fn batch4_skull_physical_query_counts_attached_bats_once_and_excludes_society_attachment() {
    let (mut g,source)=setup();let host=field(&mut g,"JC029",0);g.board_mut(&host).unwrap().time_markers=2;
    // Disclosed, validated carrier layout; Bat remains both character/attachment.
    let mut bat=g.make_card("BQ026",0);bat.time_markers=2;let target=bat.id.clone();g.attachments.push(Attachment{card:bat,host_id:host});
    let mut badge=g.make_card("JC033",0);badge.time_markers=50;g.attachments.push(Attachment{card:badge,host_id:"society:p0".into()});
    restore(&mut g);let before=serde_json::to_string(&g).unwrap();assert_eq!(g.physical_region_time_markers(0),4);assert_eq!(serde_json::to_string(&g).unwrap(),before);
    fire(&mut g,0,&source,&target);top(&mut g);assert_eq!(g.board(&target).unwrap().1.time_markers,3);assert_eq!(g.physical_region_time_markers(0),5);assert!(g.regions[0].destroyed);restore(&mut g);
}
#[test]
fn batch4_skull_payment_host_program_and_duplicate_frame_mutations_reject_restore() {
    let (mut g,source)=setup();let target=field(&mut g,"JC125",0);fire(&mut g,0,&source,&target);
    for n in 0..6 {let mut bad=g.clone();let f=bad.stack.last_mut().unwrap().frame.as_mut().unwrap();match n {
      0=>f.already_paid.clear(),1=>f.source.attachment_host_instance=Some("wrong-host".into()),2=>f.source.region=Some(1),3=>f.steps[0].op=Op::PlanAdvance,4=>f.cursor=1,_=>f.targets[0].spec.predicate=None,
    } assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"mutation {n}");}
    let mut bad=g.clone();bad.stack.push(bad.stack.last().unwrap().clone());assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    restore(&mut g);top(&mut g);
}
