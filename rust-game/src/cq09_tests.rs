use crate::{cq09,catalog,model::*,rules::*,unit_support::*,mystic_contest as m};
use crate::jc029_tests::{pass_top,envelope};
#[test]
fn cq09_room_native_witness(){
 use crate::{abi112_review_play_tests::Trace,room::{RoomEnvelope,RoomCommand}};use sha2::{Digest,Sha256};
 let mut cases=vec![];
 for accept in [true,false]{let shown=["JC001","JC125","JC125","JC125"];let(mut g,id)=setup(shown);let target=grave(&mut g,"JC001",0);let turn=g.turn;
  let mut t=Trace::new(g,format!("cq09-reveal-end-{accept}"),909+usize::from(accept));
  t.action(0,Action{card_id:Some(id),region:Some(0),..Action::new("conceal")});let hidden=t.room.game.regions[0].cards.iter().find(|c|c.definition==cq09::ID).unwrap().id.clone();t.action(0,Action{card_id:Some(hidden),..Action::new("reveal")});t.until_pending();t.choose(vec!["accept".into()]);t.until_pending();
  for(seat,id)in shown.into_iter().enumerate(){assert_eq!(t.room.game.pending.as_ref().unwrap().seat,seat);let id=t.room.game.players[seat].hand.iter().find(|c|c.definition==id).unwrap().id.clone();t.choose(vec![id]);}
  // Five live regions each have six four-seat response windows, plus action/rewards.
  let end_budget=64+t.room.game.regions.len()*32;
  for _ in 0..end_budget{
   if t.room.game.jc054_end_step_turn==Some(turn)&&t.room.game.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key==cq09::END)){break;}
   assert_eq!(t.room.game.turn,turn);
   if let Some(p)=t.room.game.pending.clone(){
    if matches!(&p.resolution,ChoiceResolution::Forecast{..}|ChoiceResolution::Frame{choice:FrameChoice::Forecast{..},..}){
     // Forecast partitions every offered card; preserve the returned order on top.
     let top=p.choice.options.iter().map(|o|o.id.clone()).collect();
     t.action(p.seat,Action{choice_id:Some(p.choice.id),top:Some(top),bottom:Some(vec![]),..Action::new("choose")});
    }else{let selected=p.choice.options.iter().take(p.choice.min.unwrap_or(0)).map(|o|o.id.clone()).collect();t.choose(selected);}
   }else{t.pass();}
  }
  assert!(t.room.game.pending.as_ref().is_some_and(|p|p.choice.options.iter().any(|o|o.id==target)), "CQ09 end target not reached: window={:?}, pending={:?}, end={:?}, history={:?}",t.room.game.window,t.room.game.pending,t.room.game.jc054_end_step_turn,t.room.game.mystic_history);
  t.choose(if accept{vec![target.clone()]}else{vec![]});for _ in 0..32{if t.room.game.stack.is_empty(){break;}t.pass();}
  assert!(t.room.game.pending.is_none()&&t.room.game.stack.is_empty());assert_eq!(t.room.game.players[0].graveyard.iter().any(|c|c.id==target),!accept);
  cases.push(t.value());
 }
 for v in &mut cases{for step in v["steps"].as_array_mut().unwrap(){let room=RoomEnvelope::from_persisted(step["expected"]["state"].as_str().unwrap()).unwrap();let cmd:RoomCommand=serde_json::from_value(step["command"].clone()).unwrap();let dup=room.transition(step["seat"].as_u64().unwrap() as usize,Some(cmd),0).unwrap();assert_eq!(dup.outcome,"rejected");assert!(!dup.changed&&dup.journal.is_empty());assert_eq!(dup.state,step["expected"]["state"]);step["duplicate"]=serde_json::to_value(dup).unwrap();}}
 let mut bindings=crate::jc066_tests::source_bindings();for(name,bytes)in[("rust-game/src/cq09.rs",include_bytes!("cq09.rs").as_slice()),("rust-game/src/cq09_tests.rs",include_bytes!("cq09_tests.rs").as_slice()),("rust-game/src/cq10.rs",include_bytes!("cq10.rs").as_slice()),("rust-game/src/cq10_tests.rs",include_bytes!("cq10_tests.rs").as_slice()),("rust-game/src/bq084.rs",include_bytes!("bq084.rs").as_slice()),("rust-game/src/xq35.rs",include_bytes!("xq35.rs").as_slice()),("rust-game/src/abi112_review_play_tests.rs",include_bytes!("abi112_review_play_tests.rs").as_slice()),("rust-game/src/unit_support.rs",include_bytes!("unit_support.rs").as_slice())]{bindings["files"].as_object_mut().unwrap().insert(name.into(),serde_json::json!(format!("{:x}",Sha256::digest(bytes))));}
 if let Ok(path)=std::env::var("CQ09_NATIVE_OUT"){std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"catalog":catalog::catalog(),"sourceBindings":bindings,"cases":cases})).unwrap()).unwrap();}
}
#[test]
fn cq09_previous_turn_win_does_not_qualify_next_end(){let mut g=won();grave(&mut g,"JC001",0);end(&mut g);choose(&mut g,vec![]);let turn=g.turn;for _ in 0..128{if g.turn!=turn{break;}if let Some(p)=g.pending.as_ref(){assert_eq!(p.choice.min,Some(0));choose(&mut g,vec![]);}else{pass(&mut g);}}assert_eq!(g.turn,turn+1);assert!(g.mystic_history.is_empty());end(&mut g);assert!(g.pending.is_none());}
#[test]
fn cq09_does_not_bonus_other_sources_contests(){let(mut g,_)=setup(["JC001","JC125","JC125","JC125"]);field(&mut g,cq09::ID,0);let hidden=field(&mut g,"WM022",0);g.board_mut(&hidden).unwrap().face_down=true;act(&mut g,0,Action{card_id:Some(hidden),..Action::new("reveal")});for _ in 0..24{if g.pending.as_ref().is_some_and(|p|p.choice.kind==m::COMMIT_KIND){break;}pass(&mut g);}finish(&mut g,["JC001","JC125","JC125","JC125"]);assert_eq!(g.mystic_history[0].shown[0].value,1);}
fn grave(g:&mut Game,id:&str,seat:usize)->String{let c=g.make_card(id,seat);let id=c.id.clone();g.players[seat].graveyard.push(c);id}
fn setup(shown:[&str;4])->(Game,String){let mut g=game();fund(&mut g,0,"JC029",8);for(seat,id)in shown.into_iter().enumerate(){held(&mut g,id,seat);deck_cards(&mut g,seat,&["JC125";5]);}let id=held(&mut g,cq09::ID,0);(g,id)}
fn reveal(shown:[&str;4])->Game{let(mut g,id)=setup(shown);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("conceal")});let id=g.regions[0].cards.iter().find(|c|c.definition==cq09::ID).unwrap().id.clone();act(&mut g,0,Action{card_id:Some(id),..Action::new("reveal")});pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"trigger");choose(&mut g,vec!["accept".into()]);pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,m::COMMIT_KIND);g}
fn finish(g:&mut Game,shown:[&str;4]){for(seat,id)in shown.into_iter().enumerate(){let id=g.players[seat].hand.iter().find(|c|c.definition==id).unwrap().id.clone();assert_eq!(g.pending.as_ref().unwrap().seat,seat);choose(g,vec![id]);}for _ in 0..16{if let Some(p)=g.pending.as_ref(){assert_eq!(p.choice.min,Some(0));choose(g,vec![]);}else if !g.stack.is_empty(){pass(g);}else{break;}}assert!(g.pending.is_none()&&g.stack.is_empty()&&g.mystic_contest.is_none());restore(g);}
fn won()->Game{let ids=["JC001","JC125","JC125","JC125"];let mut g=reveal(ids);finish(&mut g,ids);assert_eq!(g.mystic_history[0].winner_team,Some(g.team(0)));g}
fn end(g:&mut Game){g.begin_window(Window::End);g.drive().unwrap();restore(g);}
#[test]
fn cq09_reveal_bonus_is_personal_and_covers_vampire_mage_and_familiar(){for shown in ["JC029","JC003","JC001","JC125"]{let ids=[shown,"JC003","JC029","JC125"];let mut g=reveal(ids);finish(&mut g,ids);let o=&g.mystic_history[0];assert_eq!(o.shown[0].value,catalog::card(shown).cost+if shown=="JC125"{0}else{3});assert_eq!(o.shown[1].value,catalog::card("JC003").cost);assert_eq!(o.shown[2].value,catalog::card("JC029").cost);}}
#[test]
fn cq09_face_up_deploy_does_not_start_reveal_contest(){let(mut g,id)=setup(["JC125";4]);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});pass_top(&mut g);assert!(g.pending.is_none()&&g.mystic_contest.is_none()&&g.mystic_history.is_empty());end(&mut g);assert!(g.pending.is_none());}
#[test]
fn cq09_end_returns_each_original_printed_category_and_requires_actual_win(){for id in ["JC029","JC003","JC001"]{let mut g=won();let target=grave(&mut g,id,0);end(&mut g);assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id==target));choose(&mut g,vec![target.clone()]);pass_top(&mut g);assert!(!g.players[0].graveyard.iter().any(|c|c.id==target));assert!(g.regions[0].cards.iter().any(|c|c.definition==id&&c.id!=target));restore(&mut g);}}
#[test]
fn cq09_end_filters_personal_grave_printed_cost_and_character_types(){let mut g=won();let good=grave(&mut g,"JC001",0);grave(&mut g,"JC004",0);grave(&mut g,"JC125",0);grave(&mut g,"JC001",1);grave(&mut g,"JC036",0);end(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>(),vec![good]);choose(&mut g,vec![]);assert!(g.pending.is_none());}
#[test]
fn cq09_tie_and_loss_never_create_end_return(){for ids in [["JC125";4],["JC125","JC125","JZ52","JC125"]]{let mut g=reveal(ids);finish(&mut g,ids);assert_ne!(g.mystic_history[0].winner_team,Some(g.team(0)));grave(&mut g,"JC001",0);end(&mut g);assert!(g.pending.is_none());}}
#[test]
fn cq09_end_keeps_original_region_and_does_not_need_source_to_survive(){for replace in [false,true]{let mut g=won();let target=grave(&mut g,"JC001",0);end(&mut g);choose(&mut g,vec![target.clone()]);let source=g.regions[0].cards.iter().find(|c|c.definition==cq09::ID).unwrap().id.clone();g.remove_dead(&source,RemovalCause::Destroy);if replace{g.regions[0].card=g.make_card("DQJC115",0);}restore(&mut g);pass_top(&mut g);assert_eq!(g.players[0].graveyard.iter().any(|c|c.id==target),replace);assert_eq!(g.regions[0].cards.iter().any(|c|c.definition=="JC001"),!replace);}}
#[test]
fn cq09_end_observation_and_pending_tampering_reject_game_and_room(){let mut g=won();grave(&mut g,"JC001",0);end(&mut g);let original=envelope(&g);for n in 0..6{let mut bad=g.clone();match n{0=>bad.pending.as_mut().unwrap().seat=1,1=>bad.pending.as_mut().unwrap().choice.options.clear(),_=>{let ChoiceResolution::Declare{declaration:d,..}=&mut bad.pending.as_mut().unwrap().resolution else{panic!()};let Op::CQ09EndReturn{observation:Some(o)}=&mut d.ability.ops[0]else{panic!()};match n{2=>o.turn+=1,3=>o.winning_contest="i1".into(),4=>o.event_id=d.source.card.id.clone(),_=>d.actor=1}}}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"game {n}");let mut room=original.clone();room.game=bad;assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err(),"room {n}");}
 let mut bad=g.clone();let ChoiceResolution::Declare{declaration,..}=&g.pending.as_ref().unwrap().resolution else{panic!()};bad.effects.push_back(Effect::Declare{declaration:declaration.clone()});assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
}
#[test]
fn cq09_closed_whole_definition_and_original_familiar_rule_are_preserved(){let c=catalog::card(cq09::ID);assert!(!c.supported&&c.unique);assert_eq!((c.cost,c.defense),(6,Some(3)));assert_eq!(c.subtitle.as_deref(),Some("午夜魅影"));assert_eq!(c.text.matches("或魔宠").count(),2);for n in 0..3{let mut ds=definitions().clone();let d=ds.get_mut(cq09::ID).unwrap();match n{0=>d.abilities.clear(),1=>d.abilities[1].targets.clear(),_=>d.abilities[0].ops=vec![Op::MysticContest{actor_bonus:3}]};assert!(validate_definitions(&ds).is_err());}}
