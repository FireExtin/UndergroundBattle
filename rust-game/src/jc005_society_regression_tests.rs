//! Current, playable Society attachments, including the stable empty zone.
//! The original FAQ explicitly includes Society attachments when strengthened.
use crate::{catalog,model::*,unit_support::*};
fn layout(with_society:bool)->Game{
    let mut g=if with_society{crate::msjc01_tests::game()}else{game()};
    for p in &mut g.players{p.hand.clear();p.assets.clear();p.graveyard.clear();}
    for r in &mut g.regions{r.cards.clear();r.influence=[0,0];}
    g.first_team=0;g.begin_window(Window::Action(0));g
}
fn society_attachment(g:&mut Game)->String{
    fund(g,0,"BQ040",3);let id=held(g,"JC034",0);
    act(g,0,Action{card_id:Some(id),target_id:Some("society:p0".into()),..Action::new("play")});top(g);
    g.attachments.iter().find(|a|a.card.definition=="JC034").unwrap().card.id.clone()
}
fn yellow_without_mind()->String{catalog::catalog().cards.iter().find(|d|d.color=="黄"&&d.magic.is_empty()&&d.supported).unwrap().id.clone()}

#[test]
fn jc005_society_real_paid_play_targets_live_society_attachment_with_or_without_society_card(){
    for with_society in [false,true]{
        let mut g=layout(with_society);let source=society_attachment(&mut g);let before_society=serde_json::to_value(&g.players[0].society_zone).unwrap();
        fund(&mut g,0,&yellow_without_mind(),2);let spell=held(&mut g,"JC005",0);
        let other_mind=g.make_card("JC005",1);g.players[1].assets.push(other_mind);
        let action=Action{card_id:Some(spell.clone()),target_id:Some(source.clone()),..Action::new("play")};
        reject(&mut g,0,action.clone());assert!(!g.legal_actions(0).iter().any(|a|a.action.card_id.as_ref()==Some(&spell)&&a.action.target_id.as_ref()==Some(&source)));
        let mut mind=g.make_card("JC005",0);mind.exhausted=true;g.players[0].assets.push(mind);restore(&mut g);
        assert!(g.legal_actions(0).iter().any(|a|a.action.kind=="play"&&a.action.card_id.as_ref()==Some(&spell)&&a.action.target_id.as_ref()==Some(&source)),"strengthened JC005 must enumerate an actual Society attachment");
        let resources=g.resources(0);act(&mut g,0,action);assert_eq!(g.resources(0),resources-2);top(&mut g);
        assert!(!g.attachments.iter().any(|a|a.card.id==source));assert!(g.players[0].graveyard.iter().any(|c|c.definition=="JC034"));
        assert_eq!(serde_json::to_value(&g.players[0].society_zone).unwrap(),before_society);assert_eq!(g.society_host_seat("society:p0"),Some(0));restore(&mut g);
    }
}

#[test]
fn jc005_society_strengthened_guard_accepts_other_players_live_zone_but_rejects_invalid_host_fixtures(){
    let mut g=layout(false);let source=society_attachment(&mut g);fund(&mut g,0,&yellow_without_mind(),2);let spell=held(&mut g,"JC005",0);
    let c=g.make_card("JC005",0);g.players[0].assets.push(c);
    let s=g.source_snapshot(g.players[0].hand.iter().find(|c|c.id==spell).unwrap(),None);let slot=crate::rules::definition("JC005").abilities[0].targets[0].clone();
    assert!(g.valid_binding(0,&s,&slot,&source));
    let i=g.attachments.iter().position(|a|a.card.id==source).unwrap();g.attachments[i].card.controller=1;g.attachments[i].host_id="society:p1".into();
    assert!(g.valid_binding(0,&s,&slot,&source));restore(&mut g);
    // Pure invalid-host predicate fixtures; never described as a saved legal game.
    for host in ["society:p0","society:p99","i999999999"]{g.attachments[i].host_id=host.into();assert!(!g.valid_binding(0,&s,&slot,&source));}
}

// Fresh explicit layouts followed only by actual Room commands. Opaque state
// remains a string throughout export and the WASM/Worker comparisons.
#[test]
fn jc005_society_room_paid_response_restore_and_stale_retry(){
    use crate::room::*;
    fn command(room:&mut RoomEnvelope,seat:usize,c:RoomCommand,error:Option<&str>,steps:&mut Vec<serde_json::Value>){
        let before=serde_json::to_string(room).unwrap();
        let result=room.transition(seat,Some(c.clone()),0).unwrap();
        assert_eq!(result.error_code.as_deref(),error,"{:?}",result.error_message);
        if result.changed{assert_eq!(serde_json::to_string(&room.replay_events(&result.journal).unwrap()).unwrap(),result.state);}
        else{assert_eq!(result.state,before);assert!(result.journal.is_empty());}
        *room=RoomEnvelope::from_persisted(&result.state).unwrap();
        assert_eq!(serde_json::to_string(room).unwrap(),result.state);
        let views=(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>();
        steps.push(serde_json::json!({"state":before,"seat":seat,"command":c,"serverNowMs":"0","expected":result,"views":views}));
    }
    fn action(room:&mut RoomEnvelope,seat:usize,a:SessionAction,steps:&mut Vec<serde_json::Value>)->RoomCommand{
        let c=RoomCommand{command_id:format!("jc005-society-{}",steps.len()),expected_version:room.revision,action:a};
        command(room,seat,c.clone(),None,steps);c
    }
    fn pass_one(room:&mut RoomEnvelope,steps:&mut Vec<serde_json::Value>){
        let w=room.pacing.window.clone().expect("real response window");
        let seat=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
        action(room,seat,SessionAction::PassResponse{window_id:w.id},steps);
    }
    fn finish(room:&mut RoomEnvelope,steps:&mut Vec<serde_json::Value>){
        for _ in 0..64{if room.game.stack.is_empty()&&room.pacing.window.is_none(){return;}assert!(room.game.pending.is_none());pass_one(room,steps);}
        panic!("bounded actual response chain");
    }
    let mut cases=vec![];
    for (with_society,own_mind) in [(false,false),(false,true),(true,true)]{
        let mut g=layout(with_society);g.seed=u64::MAX-701-cases.len() as u64;g.random=g.seed;
        fund(&mut g,0,"BQ040",3);fund(&mut g,0,&yellow_without_mind(),4);
        // A teammate's Mind never strengthens this caster. Exhausted own Mind
        // does strengthen it and cannot be consumed as resource payment.
        let teammate_mind=g.make_card("JC005",1);g.players[1].assets.push(teammate_mind);
        if own_mind{let mut c=g.make_card("JC005",0);c.exhausted=true;g.players[0].assets.push(c);}
        let attachment_hand=held(&mut g,"JC034",0);let spell=held(&mut g,"JC005",0);let reply=held(&mut g,"JC005",0);
        let society=serde_json::to_value(&g.players[0].society_zone).unwrap();let seed=g.seed.to_string();
        let mut room=crate::jc029_tests::envelope(&g);let initial=serde_json::json!({"state":serde_json::to_string(&room).unwrap(),"views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>()});let mut steps=vec![];
        action(&mut room,0,Action{card_id:Some(attachment_hand),target_id:Some("society:p0".into()),..Action::new("play")}.into(),&mut steps);finish(&mut room,&mut steps);
        let attachment=room.game.attachments.iter().find(|a|a.card.definition=="JC034").expect("real Society attachment resolved").card.id.clone();
        let resources=room.game.resources(0);
        let cast=Action{card_id:Some(spell.clone()),target_id:Some(attachment.clone()),..Action::new("play")};
        if !own_mind{
            let c=RoomCommand{command_id:"jc005-unstrengthened".into(),expected_version:room.revision,action:cast.into()};
            command(&mut room,0,c,Some("invalid_action"),&mut steps);
            assert_eq!(room.game.resources(0),resources);assert!(room.game.attachments.iter().any(|a|a.card.id==attachment));
        }else{
            action(&mut room,0,cast.into(),&mut steps);assert_eq!(room.game.resources(0),resources-2);
            for _ in 0..16{if room.pacing.window.as_ref().is_some_and(|w|w.members.contains_key(&0)){break;}pass_one(&mut room,&mut steps);}
            let window=room.pacing.window.as_ref().unwrap().id.clone();let intent="jc005-response".to_string();
            action(&mut room,0,SessionAction::BeginResponse{window_id:window.clone(),intent_id:intent.clone()},&mut steps);
            let submit=action(&mut room,0,SessionAction::SubmitResponse{window_id:window,intent_id:intent,action:Action{card_id:Some(reply.clone()),target_id:Some(attachment.clone()),..Action::new("play")}},&mut steps);
            assert_eq!(room.game.resources(0),resources-4);assert_eq!(room.game.stack.len(),2);
            command(&mut room,0,submit,Some("version_conflict"),&mut steps);
            assert_eq!(room.game.resources(0),resources-4);finish(&mut room,&mut steps);
            assert!(!room.game.attachments.iter().any(|a|a.card.id==attachment));
            assert_eq!(room.game.players[0].graveyard.iter().filter(|c|c.definition=="JC034").count(),1);
            assert_eq!(room.game.players[0].graveyard.iter().filter(|c|c.definition=="JC005").count(),2);
        }
        assert_eq!(serde_json::to_value(&room.game.players[0].society_zone).unwrap(),society);
        assert_eq!(room.game.society_host_seat("society:p0"),Some(0));
        cases.push(serde_json::json!({"withSocietyCard":with_society,"ownExhaustedMind":own_mind,"teammateMind":true,"seedDecimal":seed,"attachmentInstance":attachment,"initial":initial,"steps":steps}));
    }
    if let Ok(path)=std::env::var("JC005_SOCIETY_NATIVE_OUT"){
        std::fs::write(path,serde_json::to_vec(&serde_json::json!({"format":"jc005-society-native94-v1","method":"Fresh explicit offline layouts; current Room commands, real paid response chains, rejected/stale commands, journal replay and four private views; no browser acceptance","catalog":catalog::catalog(),"cases":cases})).unwrap()).unwrap();
    }
}
