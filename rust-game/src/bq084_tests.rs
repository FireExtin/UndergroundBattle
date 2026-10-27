use crate::{bq084,catalog,model::*,rules::*,unit_support::*};
use crate::jc029_tests::pass_top;
fn grave(g:&mut Game,id:&str,seat:usize)->String{let c=g.make_card(id,seat);let id=c.id.clone();g.players[seat].graveyard.push(c);id}
fn setup(anchor_owner:Option<usize>)->(Game,String,String,Option<String>){
 let mut g=game();fund(&mut g,0,"JC096",5);let source=held(&mut g,"BQ084",0);let target=grave(&mut g,"JC001",0);
 let anchor=anchor_owner.map(|owner|{let c=g.make_card("BQ009",owner);let id=c.id.clone();g.sealed_cards.push(SealedCard{card:c,host_id:"society:p0".into()});id});(g,source,target,anchor)
}
fn declared(owner:Option<usize>)->(Game,String,Option<String>){let(mut g,source,target,anchor)=setup(owner);
 act(&mut g,0,Action{card_id:Some(source),region:Some(0),..Action::new("deploy")});pass_top(&mut g);
 choose(&mut g,vec![target.clone()]);assert!(g.stack.last().unwrap().frame.as_ref().unwrap().already_paid.is_empty());(g,target,anchor)
}
#[test]
fn bq084_optional_anchor_is_resolved_after_response_and_returns_to_actual_owner(){
 let(mut g,target,anchor)=declared(Some(2));let anchor=anchor.unwrap();
 assert!(g.sealed_cards.iter().any(|s|s.card.id==anchor));assert!(g.players[0].graveyard.iter().any(|c|c.id==target));
 pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"bq084-return-anchor");choose(&mut g,vec![anchor.clone()]);
 assert!(!g.sealed_cards.iter().any(|s|s.card.id==anchor));assert!(g.players[2].deck.iter().any(|c|c.definition=="BQ009"&&c.id!=anchor&&c.owner==2&&c.controller==2));
 assert!(!g.players[0].graveyard.iter().any(|c|c.id==target));assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC001"&&c.id!=target&&!c.exhausted));
}
#[test]
fn bq084_decline_or_absent_anchor_never_reanimates(){
 let(mut g,target,anchor)=declared(Some(0));pass_top(&mut g);choose(&mut g,vec![]);
 assert!(g.players[0].graveyard.iter().any(|c|c.id==target));assert!(g.sealed_cards.iter().any(|s|Some(&s.card.id)==anchor.as_ref()));
 let(mut g,target,_)=declared(None);pass_top(&mut g);assert!(g.pending.is_none());assert!(g.players[0].graveyard.iter().any(|c|c.id==target));
}
#[test]
fn bq084_foreign_host_and_non_anchor_are_not_offered(){
 let(mut g,source,target,_)=setup(None);for (id,host) in [("BQ009","society:p1"),("JC001","society:p0")]{let c=g.make_card(id,0);g.sealed_cards.push(SealedCard{card:c,host_id:host.into()});}
 act(&mut g,0,Action{card_id:Some(source),region:Some(0),..Action::new("deploy")});pass_top(&mut g);choose(&mut g,vec![target.clone()]);pass_top(&mut g);
 assert!(g.pending.is_none());assert_eq!(g.sealed_cards.len(),2);assert!(g.players[0].graveyard.iter().any(|c|c.id==target));
}
#[test]
fn bq084_invalidated_original_target_cannot_consume_an_anchor(){
 let(mut g,target,anchor)=declared(Some(0));let i=g.players[0].graveyard.iter().position(|c|c.id==target).unwrap();let c=g.players[0].graveyard.remove(i);let c=g.reset_zone_card(c);g.players[0].hand.push(c);
 grave(&mut g,"JC001",0);pass_top(&mut g);assert!(g.pending.is_none());assert!(g.sealed_cards.iter().any(|s|Some(&s.card.id)==anchor.as_ref()));assert!(!g.regions[0].cards.iter().any(|c|c.definition=="JC001"));
}
#[test]
fn bq084_departed_source_keeps_original_region(){
 let(mut g,_,anchor)=declared(Some(0));let id=g.regions[0].cards.iter().find(|c|c.definition=="BQ084").unwrap().id.clone();g.remove_dead(&id,RemovalCause::Destroy);restore(&mut g);pass_top(&mut g);choose(&mut g,vec![anchor.unwrap()]);
 assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC001"));assert!(g.players[0].graveyard.iter().any(|c|c.definition=="BQ084"));
}
#[test]
fn bq084_pending_restore_rejects_wrong_seat_options_program_and_payload(){
 let(mut g,_,anchor)=declared(Some(0));pass_top(&mut g);let p=g.pending.clone().unwrap();
 let mut bad_action=Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![anchor.unwrap()]),region:Some(1),..Action::new("choose")};reject(&mut g,0,bad_action.clone());bad_action.region=None;reject(&mut g,1,bad_action);
 for n in 0..4{let mut bad=g.clone();let p=bad.pending.as_mut().unwrap();match n{0=>p.seat=1,1=>p.choice.min=Some(1),2=>p.choice.options.clear(),_=>{if let ChoiceResolution::Frame{frame,..}=&mut p.resolution{frame.cursor=0;}}}
 assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"mutation {n}");}
}
#[test]
fn bq084_original_traits_and_definition_remain_closed(){
 let(mut g,source,_,_)=setup(None);let c=catalog::card(bq084::ID);assert!(!c.supported);assert_eq!((c.cost,c.defense),(5,Some(1)));assert_eq!(c.temporary_icons.combat,1);assert_eq!(c.magic,"星辰");assert!(definition(bq084::ID).traits.spirit);
 reject(&mut g,0,Action{card_id:Some(source.clone()),..Action::new("asset")});reject(&mut g,0,Action{card_id:Some(source),region:Some(0),cost_selected:Some(vec!["fake".into()]),..Action::new("deploy")});
 for n in 0..3{let mut defs=definitions().clone();let d=defs.get_mut(bq084::ID).unwrap();match n{0=>d.abilities.clear(),1=>d.traits.spirit=false,_=>d.abilities[0].costs.push(Cost::Assets(1))};assert!(validate_definitions(&defs).is_err());}
}
#[test]
fn bq084_room_native_witness(){
 use crate::{abi112_review_play_tests::Trace,room::{RoomCommand,RoomEnvelope}};use sha2::{Digest,Sha256};
 let mut cases=vec![];
 for accept in [true,false]{let(g,source,target,anchor)=setup(Some(2));let mut t=Trace::new(g,format!("bq084-anchor-{accept}"),884+usize::from(accept));
 t.action(0,Action{card_id:Some(source),region:Some(0),..Action::new("deploy")});t.until_pending();t.choose(vec![target]);t.until_pending();assert_eq!(t.room.game.pending.as_ref().unwrap().choice.kind,"bq084-return-anchor");t.choose(if accept{vec![anchor.unwrap()]}else{vec![]});
 assert!(t.room.game.pending.is_none());assert!(t.room.game.stack.is_empty());assert_eq!(t.room.game.regions[0].cards.iter().any(|c|c.definition=="JC001"),accept);
 let mut v=t.value();for step in v["steps"].as_array_mut().unwrap(){let room=RoomEnvelope::from_persisted(step["expected"]["state"].as_str().unwrap()).unwrap();let cmd:RoomCommand=serde_json::from_value(step["command"].clone()).unwrap();let dup=room.transition(step["seat"].as_u64().unwrap() as usize,Some(cmd),0).unwrap();assert_eq!(dup.outcome,"rejected");assert!(!dup.changed&&dup.journal.is_empty());assert_eq!(dup.state,step["expected"]["state"]);step["duplicate"]=serde_json::to_value(dup).unwrap();}cases.push(v);}
 let mut bindings=crate::jc066_tests::source_bindings();for(name,bytes)in[("rust-game/src/bq084.rs",include_bytes!("bq084.rs").as_slice()),("rust-game/src/bq084_tests.rs",include_bytes!("bq084_tests.rs").as_slice()),("rust-game/src/xq35.rs",include_bytes!("xq35.rs").as_slice()),("rust-game/src/xq35_tests.rs",include_bytes!("xq35_tests.rs").as_slice()),("rust-game/src/abi112_review_play_tests.rs",include_bytes!("abi112_review_play_tests.rs").as_slice()),("rust-game/src/unit_support.rs",include_bytes!("unit_support.rs").as_slice())]{bindings["files"].as_object_mut().unwrap().insert(name.into(),serde_json::json!(format!("{:x}",Sha256::digest(bytes))));}
 if let Ok(path)=std::env::var("BQ084_NATIVE_OUT"){std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"catalog":catalog::catalog(),"sourceBindings":bindings,"cases":cases})).unwrap()).unwrap();}
}
#[test]
fn bq084_trigger_restoration_rejects_foreign_grave_options_and_wrong_choice_stage(){
 let(mut g,source,_,_)=setup(Some(0));grave(&mut g,"JC001",1);
 act(&mut g,0,Action{card_id:Some(source),region:Some(0),..Action::new("deploy")});pass_top(&mut g);
 let p=g.pending.as_ref().unwrap();assert_eq!(p.choice.options.len(),1);
 for n in 0..3{let mut bad=g.clone();let p=bad.pending.as_mut().unwrap();match n{0=>p.choice.options[0].id="i1".into(),1=>p.choice.min=Some(1),_=>if let ChoiceResolution::Declare{stage,..}=&mut p.resolution{*stage=DeclareChoice::Accept;}}
 assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());}
}
#[test]
fn bq084_original_region_replacement_does_not_redirect_entry(){
 let(mut g,target,anchor)=declared(Some(0));
 // Explicit replacement layout: the same regional index now has a different
 // physical region UID, while the already declared source keeps the old UID.
 g.regions[0].card=g.make_card("DQJC115",0);restore(&mut g);pass_top(&mut g);choose(&mut g,vec![anchor.clone().unwrap()]);
 assert!(g.players[0].graveyard.iter().any(|c|c.id==target));assert!(!g.regions[0].cards.iter().any(|c|c.definition=="JC001"));
 assert!(!g.sealed_cards.iter().any(|s|Some(&s.card.id)==anchor.as_ref()));assert!(g.players[0].deck.iter().any(|c|c.definition=="BQ009"));
}
