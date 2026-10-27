use crate::{model::*,unit_support::*,bq048 as b};
use crate::jc029_tests::{pass_top,envelope};
fn setup(parked:bool)->(Game,String){let mut g=game();let core=if parked{Some(g.make_card("BQ046",0))}else{field(&mut g,"BQ046",0);None};g.core_zones.push(crate::core_zone::CoreZone{owner:0,definition:"BQ046".into(),rest:0,card:core});fund(&mut g,0,"JC042",2);let id=held(&mut g,b::ID,0);(g,id)}
fn play(id:&str)->Action{Action{card_id:Some(id.into()),ability_id:Some(b::KEY.into()),option:Some("core".into()),..Action::new("play")}}
#[test]
fn bq048_paid_command_invokes_printed_core_from_field_and_core_zone_without_honor(){
 for parked in [false,true]{let(mut g,id)=setup(parked);assert_eq!(g.players[0].honor,0);act(&mut g,0,play(&id));assert_eq!(g.resources(0),1);pass_top(&mut g);let p=g.pending.clone().unwrap();assert_eq!(p.choice.options.len(),3);let ids=p.choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>();act(&mut g,0,Action{choice_id:Some(p.choice.id),top:Some(ids),bottom:Some(vec![]),..Action::new("choose")});assert_eq!(g.players[0].honor,0);assert!(g.players[0].graveyard.iter().any(|c|c.definition==b::ID));restore(&mut g);}
}
#[test]
fn bq048_unsupported_modes_payload_and_unpaid_core_button_are_not_admitted(){let(mut g,id)=setup(true);for mode in ["ready",""]{let mut a=play(&id);a.option=Some(mode.into());reject(&mut g,0,a);}let mut a=play(&id);a.target_id=Some("p0".into());reject(&mut g,0,a);reject(&mut g,0,Action{card_id:Some(id),ability_id:Some(b::KEY.into()),..Action::new("activate")});assert_eq!(g.resources(0),2);assert!(!crate::catalog::card(b::ID).supported);}
#[test]
fn bq048_absent_or_hidden_core_is_not_replaced_by_another_players_core(){let(mut g,id)=setup(false);g.regions[0].cards.iter_mut().find(|c|c.definition=="BQ046").unwrap().face_down=true;act(&mut g,0,play(&id));pass_top(&mut g);assert!(g.pending.is_none());assert_eq!(g.resources(0),1);}
#[test]
fn bq048_wrong_source_missing_frame_and_modified_program_rejected(){let(mut g,id)=setup(true);act(&mut g,0,play(&id));for case in 0..4{let mut bad=g.clone();let item=bad.stack.last_mut().unwrap();if case==0{item.frame=None;}else{let f=item.frame.as_mut().unwrap();match case{1=>f.source.card.definition="JC001".into(),2=>f.steps.push(f.steps[0].clone()),_=>f.already_paid.clear()}}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());let mut room=envelope(&g);room.game=bad;assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());}}
pub(crate) fn native_cases()->Vec<serde_json::Value>{use crate::abi112_review_play_tests::Trace;let mut cases=vec![];for parked in [false,true]{let(g,id)=setup(parked);let mut t=Trace::new(g,format!("bq048-command-core-{parked}"),480+usize::from(parked));t.action(0,play(&id));t.until_pending();let p=t.room.game.pending.clone().unwrap();let ids=p.choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>();t.action(0,Action{choice_id:Some(p.choice.id),top:Some(vec![ids[2].clone(),ids[0].clone()]),bottom:Some(vec![ids[1].clone()]),..Action::new("choose")});assert!(t.room.game.pending.is_none()&&t.room.game.stack.is_empty());cases.push(t.value());}let(mut g,id)=setup(false);let victim=field(&mut g,"JC032",1);let mut t=Trace::new(g,"bq048-command-damage".into(),489);t.action(0,damage_action(&id,&victim));for _ in 0..8{if t.room.game.stack.is_empty(){break;}t.pass();}assert!(t.room.game.stack.is_empty()&&t.room.game.pending.is_none());assert_eq!(t.room.game.board(&victim).unwrap().1.damage,1);cases.push(t.value());cases}
fn damage_action(id:&str,target:&str)->Action{Action{card_id:Some(id.into()),ability_id:Some(b::KEY.into()),option:Some("damage".into()),target_id:Some(target.into()),..Action::new("play")}}
#[test]
fn bq048_damage_uses_real_paid_stack_and_rechecks_core_region(){
 for leaves in [false,true]{let(mut g,id)=setup(false);let victim=field(&mut g,"JC032",1);act(&mut g,0,damage_action(&id,&victim));assert_eq!(g.resources(0),1);
 if leaves{let core=g.regions[0].cards.iter().position(|c|c.definition=="BQ046").unwrap();let c=g.regions[0].cards.remove(core);g.regions[1].cards.push(c);}
 pass_top(&mut g);assert_eq!(g.board(&victim).unwrap().1.damage,if leaves{0}else{1});restore(&mut g);}
}
#[test]
fn bq048_damage_rejects_other_regions_hidden_core_and_wrong_frame_contract(){
 let(mut g,id)=setup(false);let victim=field(&mut g,"JC032",1);let c=g.regions[0].cards.pop().unwrap();g.regions[1].cards.push(c);reject(&mut g,0,damage_action(&id,&victim));let c=g.regions[1].cards.pop().unwrap();g.regions[0].cards.push(c);
 g.regions[0].cards.iter_mut().find(|c|c.definition=="BQ046").unwrap().face_down=true;reject(&mut g,0,damage_action(&id,&victim));g.regions[0].cards.iter_mut().find(|c|c.definition=="BQ046").unwrap().face_down=false;
 act(&mut g,0,damage_action(&id,&victim));for case in 0..4{let mut bad=g.clone();let f=bad.stack.last_mut().unwrap().frame.as_mut().unwrap();match case{0=>f.targets[0].spec.predicate=None,1=>f.steps[0].op=crate::rules::Op::DamageTarget{slot:0,amount:2},2=>f.targets.clear(),_=>{f.source.card.definition="JC001".into();f.ability_key="foreign".into();}};assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());let mut room=envelope(&g);room.game=bad;assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());}
}
