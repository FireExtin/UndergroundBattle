use crate::{msjc12 as m,catalog,model::*,rules::*,unit_support::*};
use crate::jc029_tests::pass_top;
fn setup()->Game {let mut g=game();g.players[0].society_zone.card=Some(g.make_card(m::ID,0));fund(&mut g,0,"JC001",4);deck_cards(&mut g,0,&["JC125";8]);g}
fn action(g:&Game,key:&str,ids:Vec<String>,target:Option<usize>)->Action {Action{card_id:Some(g.players[0].society_zone.card.as_ref().unwrap().id.clone()),ability_id:Some(key.into()),cost_selected:Some(ids),target_id:target.map(player_id),..Action::new("activate")}}
fn trio(g:&mut Game,def:&str)->Vec<String>{(0..3).map(|r|{let c=g.make_card(def,0);let id=c.id.clone();g.regions[r].cards.push(c);id}).collect()}
#[test]
fn msjc12_catalog_printed_construction_and_closed_admission(){
 let s=crate::society::definition(m::ID).unwrap();assert!(!s.card.supported);assert_eq!(s.card.color,"蓝");assert_eq!(s.starting_hand,6);assert_eq!(s.subtitle,"黑色家族");
 let cards=&catalog::catalog().cards;
 for c in cards {let expected=matches!(c.color.as_str(),"蓝"|"中立")||(c.kind=="character"&&c.subtypes.iter().any(|s|matches!(s.as_str(),"吸血鬼"|"奴仆")));assert_eq!(crate::society::validate_construction(s,&[catalog::DeckEntry{card_id:c.id.clone(),count:1}],cards).is_ok(),expected,"{}",c.id);}
 let mut d=crate::deck::preset("watchers").unwrap();d.society_id=Some(m::ID.into());assert!(crate::deck::validate(d).is_err());
}
#[test]
fn msjc12_live_printed_cost_grant_is_controller_and_zone_scoped(){
 let mut g=setup();let id=field(&mut g,"JC125",0);let c=g.board(&id).unwrap().1;assert!(g.current_subtypes(c).iter().any(|s|s=="奴仆"));
 let hand=held(&mut g,"JC125",0);assert!(!g.current_subtypes(g.players[0].hand.iter().find(|c|c.id==hand).unwrap()).iter().any(|s|s=="奴仆"));
 for seat in [1,2]{g.board_mut(&id).unwrap().controller=seat;assert!(!g.current_subtypes(g.board(&id).unwrap().1).iter().any(|s|s=="奴仆"));}
 g.board_mut(&id).unwrap().controller=0;g.board_mut(&id).unwrap().face_down=true;assert!(g.current_subtypes(g.board(&id).unwrap().1).is_empty());
 let expensive=catalog::catalog().cards.iter().find(|c|c.kind=="character"&&c.cost>2&&c.subtypes.iter().any(|s|s=="人类")&&!c.subtypes.iter().any(|s|s=="奴仆")).unwrap().id.clone();let id=field(&mut g,&expensive,0);assert!(!g.current_subtypes(g.board(&id).unwrap().1).iter().any(|s|s=="奴仆"));
}
#[test]
fn msjc12_three_distinct_across_regions_paid_before_response_and_draw(){let mut g=setup();let ids=trio(&mut g,"JC125");let before=g.players[0].hand.len();let a=action(&g,m::DRAW,ids.clone(),None);assert!(g.legal_actions(0).iter().any(|x|x.action==a));act(&mut g,0,a);assert_eq!(g.resources(0),3);assert!(g.players[0].society_zone.card.as_ref().unwrap().exhausted);assert!(ids.iter().all(|id|g.board(id).unwrap().1.exhausted));assert_eq!(g.players[0].hand.len(),before);pass_top(&mut g);assert_eq!(g.players[0].hand.len(),before+1);restore(&mut g);}
#[test]
fn msjc12_invalid_payment_is_atomic_and_enemy_target_required(){
 for case in 0..6 {let mut g=setup();let mut ids=trio(&mut g,"JC125");match case {0=>{ids.pop();},1=>ids[2]=ids[1].clone(),2=>g.board_mut(&ids[1]).unwrap().exhausted=true,3=>g.board_mut(&ids[1]).unwrap().face_down=true,4=>g.board_mut(&ids[1]).unwrap().controller=2,_=>g.players[0].assets.clear()};let a=action(&g,m::DRAW,ids,None);reject(&mut g,0,a);}
 for target in [0,1]{let mut g=setup();let ids=trio(&mut g,"JC042");let a=action(&g,m::DISCARD,ids,Some(target));reject(&mut g,0,a);}
}
#[test]
fn msjc12_vampires_discard_target_chooses_and_restore_rejects_corruption(){
 let mut g=setup();let vampire=catalog::catalog().cards.iter().find(|c|c.kind=="character"&&c.subtypes.iter().any(|s|s=="吸血鬼")&&c.supported).unwrap().id.clone();let ids=trio(&mut g,&vampire);let chosen=held(&mut g,"JC125",2);held(&mut g,"JC001",2);let a=action(&g,m::DISCARD,ids,Some(2));act(&mut g,0,a);
 for case in 0..5 {let mut bad=g.clone();let f=bad.stack.last_mut().unwrap().frame.as_mut().unwrap();match case{0=>f.steps[0].op=Op::Draw{player:PlayerRef::Actor,count:9,end:DeckEnd::Top},1=>{f.already_paid.pop();},2=>f.targets[0].id="p1".into(),3=>if let PaidCost::MSJC12Trio(p)=&mut f.already_paid[2]{p[2]=p[0].clone()},_=>f.source.card.definition="MSJC09".into()};assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());let mut room=crate::jc029_tests::envelope(&g);room.game=bad;assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());}
 pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().seat,2);restore(&mut g);
 let mut bad=g.clone();if let ChoiceResolution::Frame{choice:FrameChoice::Discard{redraw,..},..}=&mut bad.pending.as_mut().unwrap().resolution{*redraw=true;}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());choose(&mut g,vec![chosen]);assert_eq!(g.players[2].hand.len(),1);assert_eq!(g.players[2].graveyard.len(),1);assert_eq!(g.resources(0),2);restore(&mut g);
}
#[test]
fn msjc12_closed_definition_cannot_change_or_transplant(){let original=m::definition();for case in 0..5{let mut d=original.clone();match case{0=>d.abilities[0].costs.clear(),1=>d.abilities[0].timing=Timing::Fast,2=>d.abilities[1].targets[0].relation=Relation::Any,3=>{d.abilities.pop();},_=>d.abilities[0].activation_only=false};assert!(m::validate_definition(m::ID,&d).is_err());}assert!(m::validate_definition("MSJC09",&original).is_err());}
fn finish_trace(t:crate::abi112_review_play_tests::Trace)->serde_json::Value {
 let mut v=t.value();for step in v["steps"].as_array_mut().unwrap(){let room=crate::room::RoomEnvelope::from_persisted(step["expected"]["state"].as_str().unwrap()).unwrap();let cmd=serde_json::from_value(step["command"].clone()).unwrap();let dup=room.transition(step["seat"].as_u64().unwrap()as usize,Some(cmd),0).unwrap();assert_eq!(dup.outcome,"rejected");assert!(!dup.changed&&dup.journal.is_empty());assert_eq!(dup.state,step["expected"]["state"]);step["duplicate"]=serde_json::to_value(dup).unwrap();}v
}
#[test]
fn msjc12_natural_fifty_card_deck_to_three_humans_paid_draw_room_witness(){
 use crate::{abi112_review_play_tests::Trace,deck};
 let mut d=deck::preset("watchers").unwrap();d.society_id=Some(m::ID.into());d.cards=vec![catalog::DeckEntry{card_id:"JC125".into(),count:50}];
 let mut societies=catalog::catalog().societies.clone();societies.iter_mut().find(|s|s.card.id==m::ID).unwrap().card.supported=true;
 let d=deck::validate_with_registry(d,&catalog::catalog().cards,&societies).unwrap();
 let mut g=Game::new("natural-msjc12".into(),"LOCAL".into(),"teams".into(),"P0".into(),"watchers".into(),84).unwrap();for seat in 1..4{g.join(format!("P{seat}"),"watchers".into()).unwrap();}
 g.start_validated_decks(vec![d.clone(),deck::preset("watchers").unwrap(),deck::preset("watchers").unwrap(),deck::preset("watchers").unwrap()]).unwrap();g.drive().unwrap();assert_eq!(g.players[0].hand.len(),6);
 let mut t=Trace::new_preserving_rng(g,"msjc12-natural-fifty-card-draw".into(),630);let mut invoked=false;let mut before=0;let mut done=false;
 for _ in 0..700 {
  if let Some(p)=t.room.game.pending.clone(){let ids=p.choice.options.iter().take(p.choice.min.unwrap_or(0)).map(|o|o.id.clone()).collect();t.choose(ids);continue;}
  if invoked&&t.room.game.stack.is_empty(){assert_eq!(t.room.game.players[0].hand.len(),before+1);done=true;break;}
  let g=&t.room.game;let legal=g.legal_actions(0);let mut a=None;
  if g.players[0].assets.is_empty(){a=legal.iter().find(|a|a.action.kind=="asset").map(|a|a.action.clone());}
  if a.is_none()&&g.regions.iter().flat_map(|r|&r.cards).filter(|c|c.controller==0&&c.definition=="JC125").count()<3 {a=legal.iter().find(|a|a.action.kind=="deploy"&&g.players[0].hand.iter().any(|c|Some(&c.id)==a.action.card_id.as_ref()&&c.definition=="JC125")).map(|a|a.action.clone());}
  if a.is_none(){a=legal.iter().find(|a|a.action.ability_id.as_deref()==Some(m::DRAW)).map(|a|a.action.clone());if a.is_some(){invoked=true;before=g.players[0].hand.len();}}
  if let Some(a)=a{t.action(0,a);}else{t.pass();}
 }
 assert!(done,"natural chain failed at turn {}",t.room.game.turn);
 let mut natural=finish_trace(t);natural["construction"]=serde_json::json!({"deck":d,"seed":84,"admission":"private candidate registry only","noLayoutOrResourceMutation":true});
 let mut g=setup();let vampire=catalog::catalog().cards.iter().find(|c|c.kind=="character"&&c.subtypes.iter().any(|s|s=="吸血鬼")&&c.supported).unwrap().id.clone();let ids=trio(&mut g,&vampire);let victim=held(&mut g,"JC125",2);held(&mut g,"JC001",2);let a=action(&g,m::DISCARD,ids,Some(2));let mut t=Trace::new(g,"msjc12-prepared-vampire-discard".into(),631);t.action(0,a);t.until_pending();t.choose(vec![victim]);assert!(t.room.game.pending.is_none());
 if let Ok(path)=std::env::var("MSJC12_NATIVE_OUT"){std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"catalog":catalog::catalog(),"sourceBindings":source_bindings(),"cases":[natural,finish_trace(t)]})).unwrap()).unwrap();}
}
#[test]
fn msjc12_paid_trio_remains_paid_after_character_leaves_during_response(){let mut g=setup();let ids=trio(&mut g,"JC125");let a=action(&g,m::DRAW,ids.clone(),None);act(&mut g,0,a);g.remove_dead(&ids[0],RemovalCause::Destroy);restore(&mut g);pass_top(&mut g);assert_eq!(g.players[0].hand.len(),1);assert_eq!(g.resources(0),3);assert!(ids[1..].iter().all(|id|g.board(id).unwrap().1.exhausted));}

fn source_bindings()->serde_json::Value {
 use sha2::{Digest,Sha256};let mut binding=crate::jc066_tests::source_bindings();
 for (name,bytes) in [("rust-game/src/msjc12.rs",include_bytes!("msjc12.rs").as_slice()),("rust-game/src/msjc12_tests.rs",include_bytes!("msjc12_tests.rs").as_slice()),("rust-game/src/society.rs",include_bytes!("society.rs").as_slice()),("rust-game/src/control.rs",include_bytes!("control.rs").as_slice()),("rust-game/src/abi112_review_play_tests.rs",include_bytes!("abi112_review_play_tests.rs").as_slice())] {binding["files"].as_object_mut().unwrap().insert(name.into(),serde_json::json!(format!("{:x}",Sha256::digest(bytes))));}binding
}
#[test]
fn msjc12_grant_tracks_control_conversion_hide_and_departure(){
 let mut g=setup();let id=field(&mut g,"JC125",2);
 let slave=|g:&Game,id:&str|g.current_subtypes(g.board(id).unwrap().1).iter().any(|s|s=="奴仆");
 assert!(!slave(&g,&id));g.add_control(&id,0,ControlLifetime::TurnEnd{turn:g.turn},SubtypeChange::None);assert!(slave(&g,&id));restore(&mut g);
 g.add_control(&id,0,ControlLifetime::TurnEnd{turn:g.turn},SubtypeChange::HumanToVampire);assert!(!slave(&g,&id));assert!(g.current_subtypes(g.board(&id).unwrap().1).iter().any(|s|s=="吸血鬼"));restore(&mut g);
 let own=field(&mut g,"JC125",0);assert!(slave(&g,&own));let hidden=g.hide_in_place_successor(&own).unwrap();assert!(g.current_subtypes(&hidden).is_empty());restore(&mut g);
 let own=field(&mut g,"JC125",0);g.remove_dead(&own,RemovalCause::Destroy);let grave=g.players[0].graveyard.last().unwrap();assert!(!g.current_subtypes(grave).iter().any(|s|s=="奴仆"));restore(&mut g);
}
#[test]
fn msjc12_draw_empty_deck_uses_real_elimination_and_one_card_draw_is_mandatory(){
 for n in [0,1]{let mut g=setup();deck_cards(&mut g,0,&vec!["JC125";n]);let ids=trio(&mut g,"JC125");let a=action(&g,m::DRAW,ids,None);act(&mut g,0,a);pass_top(&mut g);assert_eq!(g.players[0].eliminated,n==0);assert_eq!(g.players[0].hand.len(),n);assert!(g.pending.is_none());restore(&mut g);}
}
#[test]
fn msjc12_discard_empty_single_and_response_draw_keep_original_enemy(){
 for n in [0,1,2]{let mut g=setup();let vampire=catalog::catalog().cards.iter().find(|c|c.kind=="character"&&c.subtypes.iter().any(|s|s=="吸血鬼")&&c.supported).unwrap().id.clone();let ids=trio(&mut g,&vampire);if n==1{held(&mut g,"JC125",2);}let a=action(&g,m::DISCARD,ids.clone(),Some(2));act(&mut g,0,a);
  // Explicit response-boundary component exercise: the enemy gains a hand card
  // after declaration, and a paid character changes controller. Not natural play.
  if n==2{deck_cards(&mut g,2,&["JC125"]);g.draw(2,1).unwrap();g.add_control(&ids[0],2,ControlLifetime::TurnEnd{turn:g.turn},SubtypeChange::None);}
  restore(&mut g);pass_top(&mut g);
  if n==0{assert!(g.pending.is_none());}else{let p=g.pending.clone().unwrap();assert_eq!(p.seat,2);assert_eq!(p.choice.min,Some(1));reject(&mut g,2,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),..Action::new("choose")});choose(&mut g,vec![p.choice.options[0].id.clone()]);assert!(g.players[2].hand.is_empty());assert_eq!(g.players[2].graveyard.len(),1);}
  assert_eq!(g.resources(0),2);assert!(g.players[0].society_zone.card.as_ref().unwrap().exhausted);restore(&mut g);
 }
}
