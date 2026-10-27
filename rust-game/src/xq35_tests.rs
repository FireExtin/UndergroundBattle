use crate::{catalog, model::*, unit_support::*, xq35};
use crate::jc029_tests::pass_top;
fn grave(g:&mut Game,id:&str,seat:usize)->String {let c=g.make_card(id,seat);let id=c.id.clone();g.players[seat].graveyard.push(c);id}
fn setup()->(Game,String,String){let mut g=game();fund(&mut g,0,"JC001",1);let source=grave(&mut g,"XQ35",0);let paid=grave(&mut g,"JC001",0);(g,source,paid)}
fn action(source:&str,paid:&str)->Action{Action{card_id:Some(source.into()),ability_id:Some(xq35::KEY.into()),cost_selected:Some(vec![paid.into()]),..Action::new("activate")}}
#[test]
fn xq35_paid_return_is_exhausted_and_restores_response(){
 let(mut g,source,paid)=setup();let a=action(&source,&paid);
 assert!(g.view(0).legal_actions.iter().any(|x|x.action==a));
 act(&mut g,0,a);assert!(g.players[0].assets[0].exhausted);assert_eq!(g.sealed_cards.len(),1);
 assert_eq!(g.sealed_cards[0].host_id,"society:p0");assert!(!g.players[0].graveyard.iter().any(|c|c.id==paid));
 assert_eq!(g.players[0].graveyard.len(),1);pass_top(&mut g);restore(&mut g);
 let c=g.regions[0].cards.iter().find(|c|c.definition=="XQ35").unwrap();assert!(c.exhausted);assert!(!c.face_down);assert_ne!(c.id,source);
 assert!(g.players[0].graveyard.is_empty());
}
#[test]
fn xq35_invalid_cost_and_payload_are_atomic(){
 let(mut g,source,paid)=setup();reject(&mut g,0,action(&source,&source));
 let foreign=grave(&mut g,"JC001",1);reject(&mut g,0,action(&source,&foreign));
 let spell=grave(&mut g,"BQ090",0);reject(&mut g,0,action(&source,&spell));
 let mut a=action(&source,&paid);a.region=Some(2);reject(&mut g,0,a);
 g.players[0].assets[0].exhausted=true;reject(&mut g,0,action(&source,&paid));
}
#[test]
fn xq35_source_leaving_after_payment_does_not_refund_or_reanimate_other_copy(){
 let(mut g,source,paid)=setup();act(&mut g,0,action(&source,&paid));
 let i=g.players[0].graveyard.iter().position(|c|c.id==source).unwrap();let c=g.players[0].graveyard.remove(i);let c=g.reset_zone_card(c);g.players[0].hand.push(c);
 grave(&mut g,"XQ35",0);restore(&mut g);pass_top(&mut g);
 assert!(!g.regions.iter().any(|r|r.cards.iter().any(|c|c.definition=="XQ35")));assert!(g.players[0].assets[0].exhausted);assert_eq!(g.sealed_cards.len(),1);
}
#[test]
fn xq35_printed_restrictions_and_closed_catalog(){
 let(mut g,_,_)=setup();let c=catalog::card("XQ35");assert!(!c.supported);assert_eq!(c.cost_appearance,Some(catalog::CostAppearance::Dash));assert_eq!(c.permanent_icons.combat,1);
 let id=held(&mut g,"XQ35",0);reject(&mut g,0,Action{card_id:Some(id.clone()),region:Some(0),..Action::new("deploy")});
 act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("conceal")});
 let hidden=g.regions[0].cards.iter().find(|c|c.definition=="XQ35").unwrap().id.clone();reject(&mut g,0,Action{card_id:Some(hidden),..Action::new("reveal")});
}
#[test]
fn xq35_restore_rejects_changed_payment_program_and_duplicate_frame(){
 let(mut g,source,paid)=setup();act(&mut g,0,action(&source,&paid));
 for n in 0..5 {let mut bad=g.clone();let f=bad.stack.last_mut().unwrap().frame.as_mut().unwrap();match n{
 0=>f.actor=1,1=>f.steps.clear(),2=>f.already_paid.clear(),3=>f.cursor=1,_=>{if let PaidCost::XQ35Sealed(p)=&mut f.already_paid[1]{p.host="society:p1".into();}}}
 assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"mutation {n}");}
 let mut bad=g.clone();bad.effects.push_back(Effect::Frame{frame:Box::new(bad.stack.last().unwrap().frame.clone().unwrap())});assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
}
#[test]
fn xq35_limit_is_per_original_instance_and_paid_before_response(){
 let(mut g,source,paid)=setup();act(&mut g,0,action(&source,&paid));
 assert_eq!(g.turn_ability_usage.iter().filter(|u|u.source_instance==source && u.ability_key==xq35::KEY).count(),1);
 // Explicit cancellation layout isolates the persisted usage check while the
 // original instance still resides in the graveyard (no zone reset).
 g.stack.clear();g.effects.clear();g.begin_window(Window::Action(0));g.players[0].assets[0].exhausted=false;
 let other=grave(&mut g,"JC001",0);restore(&mut g);reject(&mut g,0,action(&source,&other));
 g.turn+=1;g.turn_ability_usage.clear();act(&mut g,0,action(&source,&other));pass_top(&mut g);
 assert!(g.regions[0].cards.iter().any(|c|c.definition=="XQ35"&&c.exhausted));
}
#[test]
fn xq35_room_native_witness(){
 use crate::{abi112_review_play_tests::Trace,room::{RoomEnvelope,RoomCommand}};
 let(g,source,paid)=setup();let mut t=Trace::new(g,"xq35-grave-return".into(),835);
 t.action(0,action(&source,&paid));
 for _ in 0..32 {if t.room.game.stack.is_empty(){break;}t.pass();}
 assert!(t.room.game.stack.is_empty());assert!(t.room.game.regions[0].cards.iter().any(|c|c.definition=="XQ35"&&c.exhausted));
 let mut value=t.value();
 value["catalog"]=serde_json::to_value(catalog::catalog()).unwrap();
 let mut bindings=crate::jc066_tests::source_bindings();
 use sha2::{Digest,Sha256};
 for (name,bytes) in [("rust-game/src/xq35.rs",include_bytes!("xq35.rs").as_slice()),("rust-game/src/xq35_tests.rs",include_bytes!("xq35_tests.rs").as_slice()),("rust-game/src/abi112_review_play_tests.rs",include_bytes!("abi112_review_play_tests.rs").as_slice()),("rust-game/src/unit_support.rs",include_bytes!("unit_support.rs").as_slice())] {bindings["files"].as_object_mut().unwrap().insert(name.into(),serde_json::json!(format!("{:x}",Sha256::digest(bytes))));}
 value["sourceBindings"]=bindings;
 for step in value["steps"].as_array_mut().unwrap(){
   let mut room=RoomEnvelope::from_persisted(step["expected"]["state"].as_str().unwrap()).unwrap();
   let cmd:RoomCommand=serde_json::from_value(step["command"].clone()).unwrap();
   let duplicate=room.transition(step["seat"].as_u64().unwrap() as usize,Some(cmd),0).unwrap();
   assert_eq!(duplicate.outcome,"rejected");assert!(!duplicate.changed);assert!(duplicate.journal.is_empty());assert_eq!(duplicate.state,step["expected"]["state"]);
   step["duplicate"]=serde_json::to_value(duplicate).unwrap();
 }
 if let Ok(path)=std::env::var("XQ35_NATIVE_OUT") {std::fs::write(path,serde_json::to_vec_pretty(&value).unwrap()).unwrap();}
}
#[test]
fn xq35_complete_definition_cannot_be_shortened_or_transplanted(){
 for n in 0..4 {let mut defs=crate::rules::definitions().clone();let d=defs.get_mut("XQ35").unwrap();match n {
 0=>d.abilities[0].costs.clear(),1=>d.abilities[0].per_turn_limit=None,2=>d.abilities[0].ops.clear(),_=>d.abilities.clear()}
 assert!(crate::rules::validate_definitions(&defs).is_err());}
 let mut defs=crate::rules::definitions().clone();defs.insert("JC001".into(),xq35::definition());assert!(crate::rules::validate_definitions(&defs).is_err());
}
