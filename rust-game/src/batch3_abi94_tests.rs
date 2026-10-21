//! A small, fresh Native/WASM integration witness. This is not a full-suite
//! export or public-batch acceptance. Every step uses the current Room protocol.
use crate::{catalog,model::*,room::*,unit_support::*};
fn step(room:&mut RoomEnvelope,seat:usize,action:SessionAction,steps:&mut Vec<serde_json::Value>){
    let before=serde_json::to_string(room).unwrap();
    let command=RoomCommand{command_id:format!("batch3-abi94-{}",steps.len()),expected_version:room.revision,action};
    let expected=room.transition(seat,Some(command.clone()),0).unwrap();assert_eq!(expected.error_code,None,"{:?}",expected.error_message);
    assert_eq!(serde_json::to_string(&room.replay_events(&expected.journal).unwrap()).unwrap(),expected.state);
    *room=RoomEnvelope::from_persisted(&expected.state).unwrap();
    let views=(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>();
    steps.push(serde_json::json!({"state":before,"seat":seat,"command":command,"serverNowMs":"0","expected":expected,"views":views}));
}
#[test]fn batch3_abi94_room_native_smoke(){
    let mut g=game();g.seed=u64::MAX-132;g.random=g.seed;
    let old=field(&mut g,"JZ26",0);let hidden=g.hide_in_place_successor(&old).unwrap().id;let monitor=field(&mut g,"JC057",2);fund(&mut g,0,"JC029",2);field(&mut g,"JC125",2);field(&mut g,"JC125",3);
    let mut room=crate::jc029_tests::envelope(&g);let initial=serde_json::json!({"state":serde_json::to_string(&room).unwrap(),"views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>()});let mut steps=vec![];
    for _ in 0..2{step(&mut room,2,SessionAction::Game{action:Action{card_id:Some(monitor.clone()),target_id:Some(hidden.clone()),..Action::new("inspect-hidden")}},&mut steps);}
    for _ in 0..48{
        if let Some(p)=room.game.pending.clone(){step(&mut room,p.seat,SessionAction::Game{action:Action{choice_id:Some(p.choice.id),selected:Some(vec!["accept".into()]),..Action::new("choose")}},&mut steps);}
        else if let Some(w)=room.pacing.window.clone(){let seat=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;step(&mut room,seat,SessionAction::PassResponse{window_id:w.id},&mut steps);}
        else{break;}
    }
    assert!(room.game.pending.is_none() && room.game.stack.is_empty() && room.game.effects.is_empty());
    assert_eq!(room.game.players[2].graveyard.len()+room.game.players[3].graveyard.len(),2);assert!(room.game.board(&hidden).is_some());
    if let Ok(path)=std::env::var("BATCH3_ABI94_SMOKE_OUT"){
        std::fs::write(path,serde_json::to_vec(&serde_json::json!({"format":"batch3-native94-integration-smoke-v1","method":"Explicit offline layout; real current Room commands, responses, restoration and four private views; not natural browser acceptance","catalog":catalog::catalog(),"initial":initial,"steps":steps,"seedDecimal":g.seed.to_string()})).unwrap()).unwrap();
    }
}
