use crate::{cq10,catalog,model::*,rules::*,unit_support::*};
use crate::jc029_tests::{pass_top,envelope};
#[test]
fn cq10_real_rumor_replacement_pauses_before_next_draw_and_restores(){
 let mut g=declared(1,1);g.players[1].hand.clear();let rumor=held(&mut g,"WM042",1);let ordinary=held(&mut g,"JC001",1);let host=field(&mut g,"JC125",1);
 pass_top(&mut g);assert_eq!(g.players[0].hand.len(),1);choose(&mut g,vec![rumor]);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"rumor");assert_eq!(g.players[0].hand.len(),1);restore(&mut g);
 let mut duplicate=g.clone();let next=duplicate.effects.iter().find(|e|matches!(e,Effect::Frame{frame} if frame.ability_key==cq10::ENTRY)).unwrap().clone();duplicate.effects.push_back(next);assert!(Game::from_persisted(&serde_json::to_string(&duplicate).unwrap()).is_err());
 choose(&mut g,vec![host]);assert_eq!(g.players[0].hand.len(),2);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"cq10-discard");choose(&mut g,vec![ordinary]);assert_eq!(g.players[1].graveyard.len(),1);assert!(g.cq10_continuations.is_empty());restore(&mut g);
}
#[test]
fn cq10_paid_reveal_and_foreign_transaction_restore_boundary(){
 let(mut g,id)=setup(0,1);fund(&mut g,0,"JC029",2);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("conceal")});let id=g.regions[0].cards.iter().find(|c|c.definition==cq10::ID).unwrap().id.clone();act(&mut g,0,Action{card_id:Some(id),..Action::new("reveal")});pass_top(&mut g);choose(&mut g,vec![]);assert!(g.pending.is_none());
 let mut g=game();fund(&mut g,0,"JC001",1);let id=held(&mut g,"JC001",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});let original=envelope(&g);
 for missing in [false,true]{let mut bad=g.clone();let item=bad.stack.last_mut().unwrap();item.card.as_mut().unwrap().definition=cq10::ID.into();if missing{item.frame=None;}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());let mut room=original.clone();room.game=bad.clone();assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());let item=bad.stack.pop().unwrap();let before=serde_json::to_value(&bad).unwrap();assert!(bad.resolve_stack(item).is_err());assert_eq!(serde_json::to_value(&bad).unwrap(),before);}
}
#[test]
fn cq10_room_native_witness(){
 use crate::{abi112_review_play_tests::Trace,room::{RoomEnvelope,RoomCommand}};use sha2::{Digest,Sha256};
 let mut cases=vec![];
 for target in [0,1]{let(g,id)=setup(1,target);let mut t=Trace::new(g,format!("cq10-entry-target-{target}"),910+target);
  t.action(0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});t.until_pending();t.choose(vec![player_id(target)]);
  for _ in 0..40{if let Some(p)=t.room.game.pending.as_ref(){assert_eq!(p.choice.kind,"cq10-discard");let id=p.choice.options[0].id.clone();t.choose(vec![id]);}else if !t.room.game.stack.is_empty(){t.pass();}else{break;}}
  assert!(t.room.game.pending.is_none()&&t.room.game.stack.is_empty()&&t.room.game.cq10_continuations.is_empty());assert_eq!(t.room.game.players[target].graveyard.len(),2);cases.push(t.value());
 }
 let mut g=game();let source=field(&mut g,cq10::ID,0);for _ in 0..3{held(&mut g,"JC001",0);}held(&mut g,"JC001",1);g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();restore(&mut g);
 let mut t=Trace::new(g,"cq10-prepare-most-hand".into(),912);t.choose(vec![source]);t.choose(vec!["region:1".into()]);for _ in 0..32{if t.room.game.stack.is_empty(){break;}t.pass();}assert_eq!(t.room.game.regions[1].influence[0],1);cases.push(t.value());
 for v in &mut cases{for step in v["steps"].as_array_mut().unwrap(){let room=RoomEnvelope::from_persisted(step["expected"]["state"].as_str().unwrap()).unwrap();let cmd:RoomCommand=serde_json::from_value(step["command"].clone()).unwrap();let dup=room.transition(step["seat"].as_u64().unwrap() as usize,Some(cmd),0).unwrap();assert_eq!(dup.outcome,"rejected");assert!(!dup.changed&&dup.journal.is_empty());assert_eq!(dup.state,step["expected"]["state"]);step["duplicate"]=serde_json::to_value(dup).unwrap();}}
 let mut bindings=crate::jc066_tests::source_bindings();for(name,bytes)in[("rust-game/src/cq10.rs",include_bytes!("cq10.rs").as_slice()),("rust-game/src/cq10_tests.rs",include_bytes!("cq10_tests.rs").as_slice()),("rust-game/src/bq084.rs",include_bytes!("bq084.rs").as_slice()),("rust-game/src/xq35.rs",include_bytes!("xq35.rs").as_slice()),("rust-game/src/abi112_review_play_tests.rs",include_bytes!("abi112_review_play_tests.rs").as_slice()),("rust-game/src/unit_support.rs",include_bytes!("unit_support.rs").as_slice())]{bindings["files"].as_object_mut().unwrap().insert(name.into(),serde_json::json!(format!("{:x}",Sha256::digest(bytes))));}
 if let Ok(path)=std::env::var("CQ10_NATIVE_OUT"){std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"catalog":catalog::catalog(),"sourceBindings":bindings,"cases":cases})).unwrap()).unwrap();}
}
fn setup(moon:u32,target:usize)->(Game,String){let mut g=game();fund(&mut g,0,"JC029",8);g.players[0].society_zone.secrecy=moon;deck_cards(&mut g,0,&["JC125";8]);for _ in 0..5{held(&mut g,"JC001",target);}let id=held(&mut g,"CQ10",0);(g,id)}
fn declared(moon:u32,target:usize)->Game{let(mut g,id)=setup(moon,target);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});pass_top(&mut g);choose(&mut g,vec![player_id(target)]);g}
fn discard_all(g:&mut Game)->usize{let mut count=0;for _ in 0..24{let Some(p)=g.pending.as_ref()else{break;};assert_eq!(p.choice.kind,"cq10-discard");let id=p.choice.options[0].id.clone();choose(g,vec![id]);count+=1;}assert!(g.pending.is_none()&&g.stack.is_empty()&&g.cq10_continuations.is_empty());count}
#[test]
fn cq10_draw_discard_repeats_in_order_for_self_teammate_and_enemy(){for target in [0,1,2]{for moon in [0,1,3]{let mut g=declared(moon,target);let before=g.players[0].hand.len();pass_top(&mut g);assert_eq!(g.players[0].hand.len(),before+1);assert_eq!(discard_all(&mut g),moon as usize+1);assert_eq!(g.players[0].deck.len(),8-(moon as usize+1));assert_eq!(g.players[target].graveyard.len(),moon as usize+1);restore(&mut g);}}}
#[test]
fn cq10_uses_resolution_moon_count_and_original_target_after_source_departure(){let mut g=declared(0,1);g.players[0].society_zone.secrecy=2;let id=g.regions[0].cards.iter().find(|c|c.definition==cq10::ID).unwrap().id.clone();g.remove_dead(&id,RemovalCause::Destroy);restore(&mut g);pass_top(&mut g);assert_eq!(discard_all(&mut g),3);assert!(g.players[0].graveyard.iter().any(|c|c.definition==cq10::ID));}
#[test]
fn cq10_empty_target_hand_still_draws_every_pair(){let mut g=declared(3,1);g.players[1].hand.clear();pass_top(&mut g);assert!(g.pending.is_none());assert_eq!(g.players[0].hand.len(),4);assert_eq!(g.players[0].deck.len(),4);assert!(g.cq10_continuations.is_empty());restore(&mut g);}
#[test]
fn cq10_exhausted_deck_finishes_remaining_discard_pairs_without_unbounded_noops(){let mut g=declared(u32::MAX,1);g.players[0].deck.clear();pass_top(&mut g);assert!(g.players[0].eliminated);assert_eq!(discard_all(&mut g),5);assert!(g.players[1].hand.is_empty());restore(&mut g);}
#[test]
fn cq10_private_discard_rejects_wrong_seat_payload_repeat_and_corrupt_continuation(){
 let mut g=declared(1,1);pass_top(&mut g);let p=g.pending.as_ref().unwrap().clone();let id=p.choice.options[0].id.clone();
 assert!(g.view(0).pending_choice.is_none());assert!(g.view(2).pending_choice.is_none());assert!(g.view(3).pending_choice.is_none());assert_eq!(g.view(1).pending_choice.unwrap().options.len(),5);
 let a=Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![id]),..Action::new("choose")};reject(&mut g,0,a.clone());let mut extra=a.clone();extra.amount=Some(8);reject(&mut g,1,extra);
 let valid_room=envelope(&g);
 for n in 0..9{let mut bad=g.clone();match n{
  0=>bad.cq10_continuations.clear(),1=>bad.cq10_continuations.push(bad.cq10_continuations[0].clone()),2=>bad.cq10_continuations[0].waiting=false,
  3=>bad.pending.as_mut().unwrap().seat=0,4=>bad.pending.as_mut().unwrap().choice.options.clear(),
  _=>{let ChoiceResolution::Frame{frame,..}=&mut bad.pending.as_mut().unwrap().resolution else{panic!()};match n{5=>frame.targets[0].id="p2".into(),6=>frame.cursor=0,7=>frame.steps[0].op=Op::CQ10Entry{secrecy:Some(99),completed:0},_=>frame.steps[0].op=Op::CQ10Entry{secrecy:Some(1),completed:1}}}}
  assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"game {n}");let mut room=valid_room.clone();room.game=bad;assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err(),"room {n}");
 }
 act(&mut g,1,a.clone());reject(&mut g,1,a);assert_eq!(discard_all(&mut g),1);
}
#[test]
fn cq10_closed_printed_traits_loyalty_and_dynamic_public_leader(){let(mut g,_)=setup(0,1);let c=catalog::card(cq10::ID);assert!(!c.supported&&c.unique);assert_eq!(c.loyalty,vec!["蓝色";4]);assert_eq!(c.subtitle.as_deref(),Some("永劫的智者"));assert_eq!((c.cost,c.defense),(8,Some(4)));let id=field(&mut g,cq10::ID,0);g.players[0].society_zone.exposure=0;assert!(!g.current_leader(g.board(&id).unwrap().1));g.players[0].society_zone.exposure=1;assert!(g.current_leader(g.board(&id).unwrap().1));g.regions[0].cards.iter_mut().find(|c|c.id==id).unwrap().face_down=true;assert!(!g.current_leader(g.board(&id).unwrap().1));
 for n in 0..3{let mut ds=definitions().clone();let d=ds.get_mut(cq10::ID).unwrap();match n{0=>d.abilities.pop(),1=>{d.abilities[0].targets.clear();None},_=>{d.abilities[0].ops=vec![Op::CQ10Entry{secrecy:Some(2),completed:0}];None}};assert!(validate_definitions(&ds).is_err());}
}
fn prepare(g:&mut Game,source:&str){g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();restore(g);choose(g,vec![source.into()]);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"cq10-preparation-target");choose(g,vec!["region:1".into()]);}
#[test]
fn cq10_prepare_strict_hand_comparison_at_resolution_and_real_region(){for own in [1,2,3]{let mut g=game();let source=field(&mut g,cq10::ID,0);for _ in 0..own{held(&mut g,"JC001",0);}for _ in 0..2{held(&mut g,"JC001",1);}prepare(&mut g,&source);pass_top(&mut g);assert_eq!(g.regions[1].influence[0],u32::from(own>2));restore(&mut g);}}
#[test]
fn cq10_prepare_response_hand_changes_and_original_region_replacement(){for replacement in [false,true]{let mut g=game();let source=field(&mut g,cq10::ID,0);held(&mut g,"JC001",1);prepare(&mut g,&source);held(&mut g,"JC001",0);held(&mut g,"JC001",0);if replacement{g.regions[1].card=g.make_card("DQJC115",0);}restore(&mut g);pass_top(&mut g);assert_eq!(g.regions[1].influence[0],u32::from(!replacement));restore(&mut g);}}
