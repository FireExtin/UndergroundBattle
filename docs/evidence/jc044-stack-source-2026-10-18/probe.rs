use hegemony_server::{model::Game,room::{RoomEnvelope,RoomCommand,SessionAction,Decision}};
use serde_json::{Value,json};
fn main(){
 let a:Vec<String>=std::env::args().collect();let root=std::path::Path::new(&a[1]);let phase=&a[2];
 let raw:Value=serde_json::from_slice(&std::fs::read(root.join("original-native85.json")).unwrap()).unwrap();
 let mut rows=vec![];
 for name in ["entry-deploy-skip","entry-reveal-skip"]{
  let s=raw["scenarios"].as_array().unwrap().iter().find(|s|s["name"]==name).unwrap();
  let original=s["steps"][0]["expected"]["state"].as_str().unwrap();
  let original_room=RoomEnvelope::from_persisted(original).unwrap();assert_eq!(original_room.revision,9);assert_eq!(original_room.game.stack.len(),1);
  let item=&original_room.game.stack[0];assert_eq!(item.card.as_ref().unwrap().definition,"JC044");assert_eq!(item.frame.as_ref().unwrap().source.card.definition,"JC044");
  if name=="entry-deploy-skip"{assert_eq!(item.id,"i245");assert_eq!(item.card.as_ref().unwrap().id,"i244");assert_eq!(item.frame.as_ref().unwrap().source.card.id,"i243");}
  let ids=json!({"item":item.id,"card":item.card.as_ref().unwrap().id,"source":item.frame.as_ref().unwrap().source.card.id});
  let entering_id=item.card.as_ref().unwrap().id.clone();
  let mut bad=original_room.clone();bad.game.stack[0].frame.as_mut().unwrap().source.card.definition="LC19".into();
  let mut expected=serde_json::to_value(&original_room).unwrap();*expected.pointer_mut("/game/stack/0/frame/source/card/definition").unwrap()=json!("LC19");assert_eq!(serde_json::to_value(&bad).unwrap(),expected);
  let state=serde_json::to_string(&bad).unwrap();
  if phase=="before"{std::fs::write(root.join(format!("{name}-bad.room")),&state).unwrap();std::fs::write(root.join(format!("{name}-good.room")),original).unwrap();}
  let game_result=Game::from_persisted(&serde_json::to_string(&bad.game).unwrap());let room_result=RoomEnvelope::from_persisted(&state);
  let game_error=game_result.as_ref().err().cloned();let room_error=room_result.as_ref().err().cloned();
  let mut bad_commands=0;let mut bad_placed=false;let mut placed_id=None;
  if phase=="before"{
   assert!(game_result.is_ok());let mut room=room_result.unwrap();
   for n in 0..32{let Some(w)=room.pacing.window.clone()else{break};let seat=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
    let cmd=RoomCommand{command_id:format!("p2-before-{name}-{n}"),expected_version:room.revision,action:SessionAction::PassResponse{window_id:w.id}};
    let tr=room.transition(seat,Some(cmd),0).unwrap();assert!(tr.error_code.is_none());bad_commands+=1;room=RoomEnvelope::from_persisted(&tr.state).unwrap();
   }
   placed_id=room.game.regions.iter().flat_map(|r|&r.cards).find(|c|c.definition=="JC044").map(|c|c.id.clone());bad_placed=placed_id.is_some();assert!(bad_placed);assert_ne!(placed_id.as_ref().unwrap(),&entering_id);
   std::fs::write(root.join(format!("{name}-bad-continued.room")),serde_json::to_string(&room).unwrap()).unwrap();
  }else{assert!(game_result.is_err());assert!(room_result.is_err());}
  let mut legal=original_room;let mut legal_commands=0;
  for row in s["steps"].as_array().unwrap().iter().skip(1){let cmd:RoomCommand=serde_json::from_value(row["command"].clone()).unwrap();let tr=legal.transition(row["seat"].as_u64().unwrap()as usize,Some(cmd),0).unwrap();assert_eq!(serde_json::to_value(&tr).unwrap(),row["expected"]);legal=RoomEnvelope::from_persisted(&tr.state).unwrap();legal_commands+=1;}
  assert!(legal.game.regions.iter().flat_map(|r|&r.cards).any(|c|c.definition=="JC044"&&c.time_markers==0));
  rows.push(json!({"scenario":name,"revision":9,"ids":ids,"onlyMutation":"/game/stack/0/frame/source/card/definition: JC044 -> LC19","gameAccepted":game_error.is_none(),"roomAccepted":room_error.is_none(),"gameError":game_error,"roomError":room_error,"badResponseCommands":bad_commands,"badJC044Placed":bad_placed,"badPlacedFreshId":placed_id,"legalContinuationCommands":legal_commands,"legalCompleteTransitionsEqualOriginal":true}));
 }
 println!("{}",serde_json::to_string_pretty(&json!({"phase":phase,"rows":rows})).unwrap());
}
