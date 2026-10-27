use crate::{bq046 as b,model::*,rules::*,unit_support::*};
use crate::jc029_tests::{pass_top,envelope};
fn setup()->(Game,String){let mut g=game();let id=field(&mut g,b::ID,0);g.players[0].honor=2;(g,id)}
fn action(id:&str)->Action{Action{card_id:Some(id.into()),ability_id:Some(b::ACTION.into()),..Action::new("activate")}}
#[test]
fn bq046_paid_bottom_move_is_not_a_draw_and_once_per_turn(){
 let(mut g,id)=setup();let bottom=g.players[0].deck.last().unwrap().clone();let top=g.players[0].deck[0].id.clone();let n=g.players[0].hand.len();
 act(&mut g,0,action(&id));assert_eq!(g.players[0].honor,1);assert_eq!(g.players[0].hand.len(),n);pass_top(&mut g);
 assert_eq!(g.players[0].hand.last().unwrap().definition,bottom.definition);assert_ne!(g.players[0].hand.last().unwrap().id,bottom.id);assert_eq!(g.players[0].deck[0].id,top);assert_eq!(g.players[0].hand.len(),n+1);
 reject(&mut g,0,action(&id));assert_eq!(g.players[0].honor,1);assert!(g.legal_actions(0).iter().all(|a|a.action!=action(&id)));
}
#[test]
fn bq046_no_honor_cannot_pay_assets_and_empty_deck_still_consumes_action(){
 let(mut g,id)=setup();g.players[0].honor=0;fund(&mut g,0,"JC096",5);reject(&mut g,0,action(&id));
 g.players[0].honor=1;g.players[0].deck.clear();act(&mut g,0,action(&id));pass_top(&mut g);assert_eq!(g.players[0].honor,0);assert_eq!(g.turn_ability_usage.iter().filter(|u|u.ability_key==b::ACTION).count(),1);assert!(g.winner_team.is_none());
}
#[test]
fn bq046_core_forecast_is_separate_and_not_an_unpaid_player_action(){
 let(mut g,id)=setup();assert!(matches!(b::core_program().as_slice(),[Op::Forecast{player:PlayerRef::Actor,count:3}]));
 reject(&mut g,0,Action{card_id:Some(id),ability_id:Some(b::CORE.into()),..Action::new("activate")});
 assert_eq!(g.players[0].honor,2);
}
#[test]
fn bq046_corrupt_paid_frame_is_rejected_by_game_and_room(){
 let(mut g,id)=setup();act(&mut g,0,action(&id));
 for n in 0..4 {let mut bad=g.clone();let f=bad.stack.last_mut().unwrap().frame.as_mut().unwrap();match n{0=>f.already_paid.clear(),1=>f.steps[0].op=Op::MoveBottomToHand{player:PlayerRef::Actor,count:2},2=>f.ability_key=b::CORE.into(),_=>bad.turn_ability_usage.clear()};
 assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());let mut room=envelope(&g);room.game=bad;assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());}
}
#[test]
fn bq046_rejects_extra_payload_and_duplicate_usage_then_resets_next_turn(){
 let(mut g,id)=setup();let mut a=action(&id);a.region=Some(0);reject(&mut g,0,a);
 act(&mut g,0,action(&id));let mut bad=g.clone();bad.turn_ability_usage.push(bad.turn_ability_usage[0].clone());assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());pass_top(&mut g);
 g.turn+=1;g.turn_ability_usage.clear();restore(&mut g);act(&mut g,0,action(&id));pass_top(&mut g);assert_eq!(g.players[0].honor,0);
}
#[test]
fn bq046_layout_only_admission_does_not_invent_core_asset_or_conceal_permissions(){
 let mut g=game();fund(&mut g,0,"JC096",8);let id=held(&mut g,b::ID,0);
 reject(&mut g,0,Action{card_id:Some(id.clone()),..Action::new("asset")});
 for kind in ["deploy","conceal"] {reject(&mut g,0,Action{card_id:Some(id.clone()),region:Some(0),..Action::new(kind)});}
 assert!(!crate::catalog::card(b::ID).supported);
}
fn core_layout(parked:bool)->Game {
 let mut g=game();let c=if parked {Some(g.make_card(b::ID,0))} else {field(&mut g,b::ID,0);None};
 g.core_zones.push(crate::core_zone::CoreZone{owner:0,definition:b::ID.into(),card:c});restore(&mut g);g
}
#[test]
fn bq046_core_component_forecasts_three_from_board_and_core_zone_without_honor(){
 for parked in [false,true] {let mut g=core_layout(parked);assert_eq!(g.players[0].honor,0);let before=g.players[0].deck.iter().map(|c|c.id.clone()).collect::<Vec<_>>();
 g.resolve_selected_bq046_core(0).unwrap();g.drive().unwrap();restore(&mut g);let p=g.pending.clone().unwrap();assert_eq!(p.choice.options.len(),3);
 let ids=p.choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>();assert_eq!(ids,before[..3]);
 reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),top:Some(vec![ids[0].clone()]),bottom:Some(vec![]),..Action::new("choose")});
 act(&mut g,0,Action{choice_id:Some(p.choice.id),top:Some(vec![ids[2].clone(),ids[0].clone()]),bottom:Some(vec![ids[1].clone()]),..Action::new("choose")});
 assert_eq!(g.players[0].deck[0].id,ids[2]);assert_eq!(g.players[0].deck[1].id,ids[0]);assert_eq!(g.players[0].deck.last().unwrap().id,ids[1]);assert_eq!(g.players[0].honor,0);assert!(g.players[0].hand.is_empty());assert!(g.turn_ability_usage.is_empty());}
}
#[test]
fn bq046_core_component_does_not_activate_hidden_foreign_or_absent_core(){
 let mut g=core_layout(false);let id=g.regions[0].cards[0].id.clone();g.board_mut(&id).unwrap().face_down=true;assert!(g.resolve_selected_bq046_core(0).is_err());g.board_mut(&id).unwrap().face_down=false;g.board_mut(&id).unwrap().controller=2;assert!(g.resolve_selected_bq046_core(0).is_err());assert!(g.resolve_selected_bq046_core(1).is_err());
}
#[test]
fn bq046_core_zone_corruption_and_public_view_are_explicit(){
 let g=core_layout(true);let id=g.core_zones[0].card.as_ref().unwrap().id.clone();
 for viewer in 0..4 {let v=g.view(viewer);assert_eq!(v.core_zones.len(),1);assert_eq!(v.core_zones[0].card.as_ref().unwrap().instance_id,id);assert!(!v.regions.iter().any(|r|r.characters.iter().any(|c|c.instance_id==id)));}
 for case in 0..4 {let mut bad=g.clone();match case {0=>bad.core_zones.push(bad.core_zones[0].clone()),1=>bad.core_zones[0].card.as_mut().unwrap().owner=2,2=>bad.players[0].hand.push(bad.core_zones[0].card.as_ref().unwrap().clone()),_=>bad.core_zones[0].card.as_mut().unwrap().time_markers=1};assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());let mut room=envelope(&g);room.game=bad;assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());}
}
#[test]
fn bq046_bq055_native_component_witness(){
 use crate::{abi112_review_play_tests::Trace,room::{RoomEnvelope,RoomCommand}};use sha2::{Digest,Sha256};
 let(mut g,id)=setup();g.players[0].honor=1;let mut t=Trace::new(g,"bq046-paid-bottom".into(),460);t.action(0,action(&id));while !t.room.game.stack.is_empty(){t.pass();}let mut cases=vec![t.value()];
 for parked in [false,true]{let mut g=core_layout(parked);g.resolve_selected_bq046_core(0).unwrap();g.drive().unwrap();let mut t=Trace::new(g,format!("bq046-prepared-core-{parked}"),461+usize::from(parked));let p=t.room.game.pending.clone().unwrap();let ids=p.choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>();t.action(0,Action{choice_id:Some(p.choice.id),top:Some(vec![ids[2].clone(),ids[0].clone()]),bottom:Some(vec![ids[1].clone()]),..Action::new("choose")});assert!(t.room.game.pending.is_none());let mut v=t.value();v["preparedComponentInvocation"]=true.into();cases.push(v);}
 cases.push(crate::bq055_tests::native_case());
 for v in &mut cases {for step in v["steps"].as_array_mut().unwrap(){let room=RoomEnvelope::from_persisted(step["expected"]["state"].as_str().unwrap()).unwrap();let cmd:RoomCommand=serde_json::from_value(step["command"].clone()).unwrap();let duplicate=room.transition(step["seat"].as_u64().unwrap() as usize,Some(cmd),0).unwrap();assert_eq!(duplicate.outcome,"rejected");assert!(!duplicate.changed&&duplicate.journal.is_empty());assert_eq!(duplicate.state,step["expected"]["state"]);step["duplicate"]=serde_json::to_value(duplicate).unwrap();}}
 let mut bindings=crate::jc066_tests::source_bindings();for(name,bytes)in[("rust-game/src/bq046.rs",include_bytes!("bq046.rs").as_slice()),("rust-game/src/bq046_tests.rs",include_bytes!("bq046_tests.rs").as_slice()),("rust-game/src/bq055.rs",include_bytes!("bq055.rs").as_slice()),("rust-game/src/bq055_tests.rs",include_bytes!("bq055_tests.rs").as_slice()),("rust-game/src/core_zone.rs",include_bytes!("core_zone.rs").as_slice())]{bindings["files"].as_object_mut().unwrap().insert(name.into(),serde_json::json!(format!("{:x}",Sha256::digest(bytes))));}
 if let Ok(path)=std::env::var("BQ046_BQ055_NATIVE_OUT"){std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"catalog":crate::catalog::catalog(),"sourceBindings":bindings,"cases":cases})).unwrap()).unwrap();}
}
