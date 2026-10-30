use crate::{model::*,unit_support::*,bq048 as b};
use crate::jc029_tests::{pass_top,envelope};
fn setup(parked:bool)->(Game,String){let mut g=game();let core=if parked{Some(g.make_card("BQ046",0))}else{field(&mut g,"BQ046",0);None};g.core_zones.push(crate::core_zone::CoreZone{owner:0,definition:"BQ046".into(),rest:0,card:core});fund(&mut g,0,"JC042",2);let id=held(&mut g,b::ID,0);(g,id)}
fn play(id:&str)->Action{Action{card_id:Some(id.into()),ability_id:Some(b::KEY.into()),option:Some("core".into()),..Action::new("play")}}
#[test]
fn bq048_paid_command_invokes_printed_core_from_field_and_core_zone_without_honor(){
 for parked in [false,true]{let(mut g,id)=setup(parked);assert_eq!(g.players[0].honor,0);act(&mut g,0,play(&id));assert_eq!(g.resources(0),1);pass_top(&mut g);let p=g.pending.clone().unwrap();assert_eq!(p.choice.options.len(),3);let ids=p.choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>();act(&mut g,0,Action{choice_id:Some(p.choice.id),top:Some(ids),bottom:Some(vec![]),..Action::new("choose")});assert_eq!(g.players[0].honor,0);assert!(g.players[0].graveyard.iter().any(|c|c.definition==b::ID));restore(&mut g);}
}
#[test]
fn bq048_unsupported_modes_payload_and_unpaid_core_button_are_not_admitted(){let(mut g,id)=setup(true);for mode in ["unknown",""]{let mut a=play(&id);a.option=Some(mode.into());reject(&mut g,0,a);}let mut a=play(&id);a.target_id=Some("p0".into());reject(&mut g,0,a);reject(&mut g,0,Action{card_id:Some(id),ability_id:Some(b::KEY.into()),..Action::new("activate")});assert_eq!(g.resources(0),2);assert!(!crate::catalog::card(b::ID).supported);}
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

fn ready_action(id:&str)->Action{Action{option:Some("ready".into()),..play(id)}}
#[test]
fn bq048_ready_paid_field_and_core_zone_restore_and_public_marker(){
 for parked in [false,true]{let(mut g,id)=setup(parked);let core=g.selected_core_source(0).unwrap().card.id;
 if !parked{g.board_mut(&core).unwrap().exhausted=true;}
 act(&mut g,0,ready_action(&id));restore(&mut g);pass_top(&mut g);
 let c=g.selected_core_source(0).unwrap().card;assert!(!c.exhausted);assert_eq!(c.permanent_combat_markers,1);assert_eq!(g.resources(0),1);restore(&mut g);
 if parked{assert_eq!(g.core_zone_views(1)[0].card.as_ref().unwrap().permanent_combat_markers,Some(1));}
 }
}

#[test]
fn bq048_ready_rechecks_personal_visible_core_after_responses(){
 for change in 0..3 {let(mut g,id)=setup(false);let core=g.selected_core_source(0).unwrap().card.id;act(&mut g,0,ready_action(&id));
 match change {0=>g.board_mut(&core).unwrap().face_down=true,1=>g.board_mut(&core).unwrap().controller=2,_=>{let c=g.remove_board(&core).unwrap().1;let c=g.reset_zone_card(c);g.core_zones[0].card=Some(c);}}
 pass_top(&mut g);if change==2 {assert_eq!(g.core_zones[0].card.as_ref().unwrap().permanent_combat_markers,1);}else{assert_eq!(g.board(&core).unwrap().1.permanent_combat_markers,0);}restore(&mut g);
 }
}
#[test]
fn bq048_ready_marker_clears_on_paid_core_deployment_and_departure(){
 let(mut g,id)=setup(true);act(&mut g,0,ready_action(&id));pass_top(&mut g);let core=g.core_zones[0].card.as_ref().unwrap().id.clone();fund(&mut g,0,"JC042",3);fund(&mut g,0,"JC056",3);
 act(&mut g,0,Action{card_id:Some(core),region:Some(0),..Action::new("deploy")});pass_top(&mut g);let core=g.selected_core_source(0).unwrap().card.id;assert_eq!(g.board(&core).unwrap().1.permanent_combat_markers,0);restore(&mut g);
 let(mut g,id)=setup(false);let core=g.selected_core_source(0).unwrap().card.id;act(&mut g,0,ready_action(&id));pass_top(&mut g);g.return_hand(&core);g.drive().unwrap();let p=g.pending.clone().unwrap();assert_eq!(p.choice.kind,"core-return");act(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec!["core".into()]),..Action::new("choose")});assert_eq!(g.core_zones[0].card.as_ref().unwrap().permanent_combat_markers,0);restore(&mut g);
}
#[test]
fn bq048_ready_corrupt_frame_and_offboard_other_markers_rejected(){
 let(mut g,id)=setup(true);act(&mut g,0,ready_action(&id));
 for n in 0..4{let mut bad=g.clone();let item=bad.stack.last_mut().unwrap();if n==0{item.frame=None;}else{let f=item.frame.as_mut().unwrap();match n{1=>f.source.card.definition="JC001".into(),2=>f.steps.push(f.steps[0].clone()),_=>f.already_paid.clear()}}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());let mut room=envelope(&g);room.game=bad;assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());}
 pass_top(&mut g);for n in 0..3{let mut bad=g.clone();let c=bad.core_zones[0].card.as_mut().unwrap();match n{0=>c.permanent_investigation_markers=1,1=>c.permanent_defense_markers=1,_=>c.exhausted=true}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());}restore(&mut g);
}
fn with_duplicates(mut v:serde_json::Value)->serde_json::Value{for s in v["steps"].as_array_mut().unwrap(){let room=crate::room::RoomEnvelope::from_persisted(s["expected"]["state"].as_str().unwrap()).unwrap();let cmd=serde_json::from_value(s["command"].clone()).unwrap();let d=room.transition(s["seat"].as_u64().unwrap()as usize,Some(cmd),0).unwrap();assert_eq!(d.state,s["expected"]["state"]);assert!(!d.changed&&d.journal.is_empty());s["duplicate"]=serde_json::to_value(d).unwrap();}v}
fn constructed(seed:u64)->(Game,crate::deck::DeckDraft){
 use crate::{deck,catalog};let mut d=deck::preset("watchers").unwrap();d.id="closed-bq048-50-plus-core".into();d.society_id=Some("MSBQ02".into());d.core_card_id=Some("BQ046".into());d.cards=[("BQ064",3),("BQ048",3),("BQ050",3),("BQ063",3),("JC125",38)].into_iter().map(|(id,count)|catalog::DeckEntry{card_id:id.into(),count}).collect();assert!(deck::validate(d.clone()).is_err());let mut cards=catalog::catalog().cards.clone();for c in &mut cards{if ["BQ046","BQ048","BQ050","BQ063"].contains(&c.id.as_str()){c.supported=true;}}let mut societies=catalog::catalog().societies.clone();societies.iter_mut().find(|s|s.card.id=="MSBQ02").unwrap().card.supported=true;let d=deck::validate_with_registry(d,&cards,&societies).unwrap();
 let mut other=deck::preset("watchers").unwrap();other.society_id=None;other.core_card_id=None;other.cards=vec![catalog::DeckEntry{card_id:"JC125".into(),count:50}];let other=deck::validate(other).unwrap();let mut g=Game::new("bq048-natural".into(),"LOCAL".into(),"teams".into(),"P0".into(),"watchers".into(),seed).unwrap();for seat in 1..4{g.join(format!("P{seat}"),"watchers".into()).unwrap();}g.start_validated_decks(vec![d.clone(),other.clone(),other.clone(),other]).unwrap();g.drive().unwrap();(g,d)
}
fn ready_natural()->serde_json::Value{
 use crate::abi112_review_play_tests::Trace;
 let(seed,g,d)=(1..1025).find_map(|seed|{let(g,d)=constructed(seed);(g.players[0].hand.iter().filter(|c|c.definition==b::ID).count()>=2).then_some((seed,g,d))}).expect("bounded legal initial hand");
 let mut t=Trace::new_preserving_rng(g,"bq048-natural-fifty-plus-core-ready".into(),4801);let mut played=false;
 for _ in 0..400 {if let Some(p)=t.room.game.pending.clone(){t.choose(p.choice.options.iter().take(p.choice.min.unwrap_or(0)).map(|o|o.id.clone()).collect());continue;}if played&&t.room.game.stack.is_empty()&&t.room.game.effects.is_empty(){break;}
 let g=&t.room.game;let legal=g.legal_actions(0);let a=if g.players[0].assets.is_empty(){legal.iter().find(|a|a.action.kind=="asset"&&a.action.card_id.as_ref().is_some_and(|id|g.players[0].hand.iter().any(|c|&c.id==id&&c.definition==b::ID)))}else{legal.iter().find(|a|a.action.kind=="play"&&a.action.option.as_deref()==Some("ready")&&a.action.ability_id.as_deref()==Some(b::KEY))};if let Some(a)=a{if a.action.kind=="play"{played=true;}t.action(0,a.action.clone());}else{t.pass();}}
 assert!(played&&t.room.game.stack.is_empty()&&t.room.game.pending.is_none());assert_eq!(t.room.game.core_zones[0].card.as_ref().unwrap().permanent_combat_markers,1);let mut v=t.value();v["construction"]=serde_json::json!({"deck":d,"seed":seed,"ordinaryCards":50,"separateCore":1,"minimumCommandCards":12,"privateRegistryOnly":true,"noLayoutOrResourceMutation":true});with_duplicates(v)
}
#[test]
fn bq048_ready_natural_and_existing_modes_export(){
 use crate::abi112_review_play_tests::Trace;use sha2::{Digest,Sha256};
 let mut cases=vec![ready_natural()];for parked in [false,true]{let(mut g,id)=setup(parked);if !parked{let id=g.selected_core_source(0).unwrap().card.id;g.board_mut(&id).unwrap().exhausted=true;}let mut t=Trace::new_preserving_rng(g,format!("bq048-ready-prepared-{parked}"),4802+usize::from(parked));t.action(0,ready_action(&id));for _ in 0..20{if t.room.game.stack.is_empty(){break;}t.pass();}assert_eq!(t.room.game.selected_core_source(0).unwrap().card.permanent_combat_markers,1);cases.push(with_duplicates(t.value()));}cases.extend(native_cases().into_iter().map(with_duplicates));
 let mut bindings=crate::shoggoth_tests::shoggoth_source_bindings();for(n,bytes)in[("rust-game/src/bq048.rs",include_bytes!("bq048.rs").as_slice()),("rust-game/src/bq048_tests.rs",include_bytes!("bq048_tests.rs").as_slice())]{bindings["files"].as_object_mut().unwrap().insert(n.into(),serde_json::json!(format!("{:x}",Sha256::digest(bytes))));}
 if let Ok(path)=std::env::var("BQ048_CLOSURE_NATIVE_OUT"){std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"catalog":crate::catalog::catalog(),"sourceBindings":bindings,"cases":cases})).unwrap()).unwrap();}
}
