use hegemony_server::{model::Game,room::RoomEnvelope};
fn main(){
 let root=std::path::Path::new("/workspace/jc044-evidence-20261018");
 let raw:serde_json::Value=serde_json::from_slice(&std::fs::read(root.join("native-assertion-final/native85.json")).unwrap()).unwrap();
 let s=raw["scenarios"].as_array().unwrap().iter().find(|s|s["name"]=="actor-0").unwrap();
 let row=s["steps"].as_array().unwrap().iter().find(|r|r["command"]["action"]["action"]["kind"]=="choose"&&r["expected"]["errorCode"].is_null()).unwrap();
 let mut room=RoomEnvelope::from_persisted(row["state"].as_str().unwrap()).unwrap();
 let target=room.game.pending.as_ref().unwrap().seat;assert!(!room.game.players[target].eliminated);
 let engine=room.game.versions.engine.clone();let actor=match &room.game.pending.as_ref().unwrap().resolution{hegemony_server::model::ChoiceResolution::Frame{frame,..}=>frame.actor,_=>panic!()};assert_ne!(actor,target);room.game.players[actor].eliminated=true;
 let game_json=serde_json::to_string(&room.game).unwrap();let room_json=serde_json::to_string(&room).unwrap();
 let game=Game::from_persisted(&game_json);let envelope=RoomEnvelope::from_persisted(&room_json);
 let private_choice=game.as_ref().ok().map(|g|g.view(target).pending_choice.is_some());
 std::fs::write(root.join("pending-actor-repro/eliminated-actor-room.json"),&room_json).unwrap();
 let result=serde_json::json!({"scope":"Native typed Room clone; only pending source actor eliminated flag changed, no JavaScript opaque-state parsing","engine":engine,"target":target,"gameRestoreAccepted":game.is_ok(),"roomRestoreAccepted":envelope.is_ok(),"privateChoiceForLivingTargetAfterActorEliminated":private_choice,"actor":actor,"actorEliminated":room.game.players[actor].eliminated,"targetEliminated":room.game.players[target].eliminated,"gameError":game.err(),"roomError":envelope.err()});
 std::fs::write(root.join("pending-actor-repro/result-second.json"),serde_json::to_vec_pretty(&result).unwrap()).unwrap();println!("{}",result);
}