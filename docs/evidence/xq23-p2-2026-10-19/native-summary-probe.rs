use hegemony_server::{model::Game,room::{RoomCommand,RoomEnvelope}};
use serde_json::{json,Value};
fn main(){
 let path=std::env::args().nth(1).unwrap();let input:Value=serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();let mut results=vec![];
 for case in input["cases"].as_array().unwrap(){
  let raw_game=std::fs::read_to_string(case["gameFile"].as_str().unwrap()).unwrap();let raw_room=std::fs::read_to_string(case["roomFile"].as_str().unwrap()).unwrap();
  let mut result=json!({"name":case["name"],"actualTarget":case["actualTarget"],"mutatedPublicTarget":case["frozenPublicTarget"]});
  match Game::from_persisted(&raw_game){Err(e)=>result["game"]=json!({"accepted":false,"error":e}),Ok(g)=>{let views=(0..4).map(|seat|serde_json::to_value(g.view(seat)).unwrap()["stack"][0]["targetSummaries"].clone()).collect::<Vec<_>>();result["game"]=json!({"accepted":true,"projectedTargetsFourSeats":views});}}
  match RoomEnvelope::from_persisted(&raw_room){Err(e)=>result["room"]=json!({"accepted":false,"error":e}),Ok(mut room)=>{
   let views=(0..4).map(|seat|serde_json::to_value(room.view(seat,0)).unwrap()["stack"][0]["targetSummaries"].clone()).collect::<Vec<_>>();let mut transitions=vec![];
   for row in input["remainingSteps"].as_array().unwrap(){let command:RoomCommand=serde_json::from_value(row["command"].clone()).unwrap();let seat=row["seat"].as_u64().unwrap()as usize;let t=room.transition(seat,Some(command),0).unwrap();transitions.push(json!({"changed":t.changed,"errorCode":t.error_code,"version":t.version}));room=RoomEnvelope::from_persisted(&t.state).unwrap();}
   result["room"]=json!({"accepted":true,"projectedTargetsFourSeats":views,"remainingTransitions":transitions,"finalStateMatchesOriginalNative":serde_json::to_string(&room).unwrap()==input["originalFinalState"].as_str().unwrap(),"originalActualTargetStillInAssets":room.game.players.iter().any(|p|p.assets.iter().any(|c|c.id=="i243")),"forgedSummaryTargetStillInAssets":room.game.players.iter().any(|p|p.assets.iter().any(|c|c.id=="i241")),"handInstances":room.game.players[0].hand.iter().map(|c|json!({"id":c.id,"definition":c.definition})).collect::<Vec<_>>()});
  }}
  results.push(result);
 }
 println!("{}",json!({"reference":"Real Native88 actor-0 first paid declaration, unchanged versions/fees/actual target/source; each invalid case changes exactly one summary field","results":results}));
}
