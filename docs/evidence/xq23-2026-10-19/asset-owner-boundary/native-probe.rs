use hegemony_server::room::{RoomEnvelope,RoomCommand};
fn main() {
 let raw=std::fs::read_to_string("/workspace/xq23-evidence-20261019/native-fifth/Native88.json").unwrap();
 let f:serde_json::Value=serde_json::from_str(&raw).unwrap();let s=&f["scenarios"][0];
 let mut room:RoomEnvelope=serde_json::from_str(s["initial"]["state"].as_str().unwrap()).unwrap();
 let target=s["steps"][0]["command"]["action"]["action"]["targetId"].as_str().unwrap();
 room.game.players[0].assets.iter_mut().find(|c|c.id==target).unwrap().owner=7;
 let state=serde_json::to_string(&room).unwrap();
 match RoomEnvelope::from_persisted(&state) {
  Err(error)=>{println!("{}",serde_json::json!({"case":"asset-owner-out-of-bounds","restored":false,"error":error}));return;},
  Ok(restored)=>room=restored,
 }
 for (index,row) in s["steps"].as_array().unwrap().iter().enumerate() {
  let command:RoomCommand=serde_json::from_value(row["command"].clone()).unwrap();let seat=row["seat"].as_u64().unwrap()as usize;
  match std::panic::catch_unwind(||room.transition(seat,Some(command),0)) {
   Err(_)=>{println!("{}",serde_json::json!({"case":"asset-owner-out-of-bounds","restored":true,"panickedAtStep":index}));return;},
   Ok(Err(error))=>{println!("{}",serde_json::json!({"case":"asset-owner-out-of-bounds","restored":true,"errorAtStep":index,"error":error}));return;},
   Ok(Ok(t))=>room=RoomEnvelope::from_persisted(&t.state).unwrap(),
  }
 }
 println!("{}",serde_json::json!({"case":"asset-owner-out-of-bounds","restored":true,"completed":true}));
}
