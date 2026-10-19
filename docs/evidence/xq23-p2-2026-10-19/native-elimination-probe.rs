use hegemony_server::{model::{Action,Game},room::{Decision,RoomCommand,RoomEnvelope,SessionAction}};
use serde_json::{json,Value};
struct Trace{room:RoomEnvelope,initial:Value,steps:Vec<Value>}
impl Trace{
 fn new(g:Game)->Self{let room=RoomEnvelope::from_game(g);let raw=serde_json::to_string(&room).unwrap();RoomEnvelope::from_persisted(&raw).unwrap();let initial=json!({"state":raw,"views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>()});Self{room,initial,steps:vec![]}}
 fn send(&mut self,seat:usize,action:SessionAction,accepted:bool){
  let state=serde_json::to_string(&self.room).unwrap();Game::from_persisted(&serde_json::to_string(&self.room.game).unwrap()).unwrap();
  let command=RoomCommand{command_id:format!("owner-p2-{}",self.steps.len()),expected_version:self.room.revision,action};
  let t=self.room.transition(seat,Some(command.clone()),0).unwrap();assert_eq!(t.error_code.is_none(),accepted,"{:?}",t.error_message);
  self.room=RoomEnvelope::from_persisted(&t.state).unwrap();Game::from_persisted(&serde_json::to_string(&self.room.game).unwrap()).unwrap();
  self.steps.push(json!({"state":state,"seat":seat,"command":command,"serverNowMs":"0","expected":t,"views":(0..4).map(|s|self.room.view(s,0)).collect::<Vec<_>>() }));
 }
 fn game(&mut self,seat:usize,a:Action,accepted:bool){self.send(seat,SessionAction::Game{action:a},accepted);}
 fn pass(&mut self){
  if let Some(w)=self.room.pacing.window.clone(){let seat=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;self.send(seat,SessionAction::PassResponse{window_id:w.id},true);}
  else{let seat=(0..4).find(|s|self.room.game.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();self.game(seat,Action::new("pass"),true);}
 }
 fn responses(&mut self){for _ in 0..64{if self.room.pacing.window.is_none(){return;}self.pass();}panic!("response limit");}
}
fn main(){
 let args=std::env::args().collect::<Vec<_>>();let f:Value=serde_json::from_str(&std::fs::read_to_string(&args[1]).unwrap()).unwrap();
 let mut g=RoomEnvelope::from_persisted(f["scenarios"][0]["initial"]["state"].as_str().unwrap()).unwrap().game;
 g.room_id=format!("{:024x}",880031);for p in &mut g.players{p.hand.clear();p.assets.clear();p.graveyard.clear();p.score_cards.clear();}for r in &mut g.regions{r.cards.clear();r.influence=[0;2];}
 let mut sources=vec![];for owner in [2,3,0]{let c=g.make_card("XQ23",owner);sources.push(c.id.clone());g.regions[0].cards.push(c);}
 for (definition,n)in[("JC104",6),("JC060",5)]{for _ in 0..n{let c=g.make_card(definition,0);g.players[0].assets.push(c);}}
 let mut spells=vec![];for _ in 0..2{let c=g.make_card("JC129",0);spells.push(c.id.clone());g.players[0].hand.push(c);}
 let own_other=g.make_card("JC060",3);g.players[3].assets.push(own_other);
 let score=g.make_card("DQJC107",2);g.players[2].score_cards.push(score);let score=g.make_card("DQJC115",3);g.players[3].score_cards.push(score);g.players[2].deck.clear();
 let mut t=Trace::new(g);let mut converted=vec![];
 for i in 0..2{
  t.game(0,Action{card_id:Some(spells[i].clone()),target_id:Some(sources[i].clone()),..Action::new("play")},true);t.responses();
  let source=t.room.game.regions[0].cards.iter().find(|c|c.id==sources[i]).unwrap();assert_eq!((source.owner,source.controller),(i+2,0));
  let target=t.room.game.players[0].assets.iter().find(|c|c.definition=="JC060"&&c.owner==0).unwrap().id.clone();
  t.game(0,Action{card_id:Some(sources[i].clone()),target_id:Some(target),ability_id:Some("xq23-return-asset-become-asset".into()),..Action::new("activate")},true);t.responses();
  let c=t.room.game.players[0].assets.iter().find(|c|c.definition=="XQ23"&&c.owner==i+2).unwrap();assert_eq!(c.controller,0);assert!(c.exhausted);converted.push(c.id.clone());
 }
 let before_elimination=serde_json::to_string(&t.room).unwrap();let before_assets=t.room.game.players[0].assets.len();
 for _ in 0..600{if t.room.game.players[2].eliminated{break;}assert!(t.room.game.pending.is_none(),"unexpected choice");t.pass();}
 assert!(t.room.game.players[2].eliminated);assert!(!t.room.game.players[3].eliminated);assert_eq!(t.room.game.status,"playing");
 let after_elimination=serde_json::to_string(&t.room).unwrap();let owned_asset_survives=t.room.game.players[0].assets.iter().any(|c|c.id==converted[0]);
 let live_owner_asset_survives=t.room.game.players[0].assets.iter().any(|c|c.id==converted[1]);assert!(live_owner_asset_survives);
 let ready_assets=t.room.game.players[0].assets.iter().filter(|c|!c.exhausted).count();let after_assets=t.room.game.players[0].assets.len();
 let scores=t.room.view(0,0).players.iter().map(|p|json!({"seat":p.seat,"score":p.score,"eliminated":p.eliminated})).collect::<Vec<_>>();
 for _ in 0..16{if t.room.game.legal_actions(0).iter().any(|a|a.action.kind=="activate"&&a.action.card_id.as_deref()==Some(&sources[2])&&a.action.target_id.as_deref()==Some(&converted[1])){break;}t.pass();}
 let can_return_dead_owner=t.room.game.legal_actions(0).iter().any(|a|a.action.kind=="activate"&&a.action.card_id.as_deref()==Some(&sources[2])&&a.action.target_id.as_deref()==Some(&converted[0]));
 let return_action=Action{card_id:Some(sources[2].clone()),target_id:Some(converted[0].clone()),ability_id:Some("xq23-return-asset-become-asset".into()),..Action::new("activate")};
 t.game(0,return_action,owned_asset_survives);
 if owned_asset_survives{t.responses();}else{t.game(0,Action{card_id:Some(sources[2].clone()),target_id:Some(converted[1].clone()),ability_id:Some("xq23-return-asset-become-asset".into()),..Action::new("activate")},true);t.responses();}
 let dead_hand=t.room.game.players[2].hand.iter().any(|c|c.definition=="XQ23");let live_hand=t.room.game.players[3].hand.iter().any(|c|c.definition=="XQ23");
 let summary=json!({"explicitInitialLayout":true,"realJC129Declarations":2,"actualXQ23Conversions":2,"owner2EliminatedByNaturalNextTurnEmptyDraw":true,"teammate3Lives":true,"gameContinues":t.room.game.status=="playing","beforeControlledAssets":before_assets,"afterControlledAssets":after_assets,"readyResourcesAfterElimination":ready_assets,"eliminatedOwner2AssetSurvives":owned_asset_survives,"livingOwner3AssetSurvives":live_owner_asset_survives,"canLegallyReturnDeadOwnerAsset":can_return_dead_owner,"deadOwnerHandRepopulated":dead_hand,"livingOwnerHandReceivedOwnCard":live_hand,"scores":scores,"commands":t.steps.len(),"borrowedOwner2AssetId":converted[0],"borrowedOwner3AssetId":converted[1]});
 let value=json!({"format":"xq23-owner-elimination-native-probe-v1","runtimeBefore":"dd974287393b1dc39ac5728766f0a6a2da4b3b04","initial":t.initial,"steps":t.steps,"beforeElimination":before_elimination,"afterElimination":after_elimination,"summary":summary});
 use std::io::Write;std::fs::OpenOptions::new().create_new(true).write(true).open(&args[2]).unwrap().write_all(&serde_json::to_vec(&value).unwrap()).unwrap();println!("{}",summary);
}
