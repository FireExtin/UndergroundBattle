//! Internal candidate Room witnesses. Initial layouts are explicit; all later
//! transitions use the shared production Room reducer. BQ033 supply stays closed; natural admission is separately tested.
use crate::{abi112_review_play_tests::Trace, catalog, model::*, room::{RoomCommand,RoomEnvelope,SessionAction}, unit_support::*};
use serde_json::{json,Value};

fn until_empty(t:&mut Trace) {
    for _ in 0..128 {
        if t.room.game.pending.is_some() { panic!("unexpected unresolved candidate choice"); }
        if t.room.game.stack.is_empty() { return; }
        t.pass();
    }
    panic!("candidate continuation did not finish");
}
fn responsive(t:&mut Trace,seat:usize,action:Action) {
    let w=t.room.pacing.window.as_ref().unwrap().id.clone();
    let intent="next20-native-response".to_string();
    t.step(seat,SessionAction::BeginResponse{window_id:w.clone(),intent_id:intent.clone()});
    t.step(seat,SessionAction::SubmitResponse{window_id:w,intent_id:intent,action});
}
fn destroy(t:&mut Trace,target:&str,response:bool) {
    let action=t.room.game.legal_actions(2).into_iter().find(|a|a.action.kind=="play"
        &&a.action.target_id.as_deref()==Some(target)
        &&t.room.game.players[2].hand.iter().any(|c|Some(&c.id)==a.action.card_id.as_ref()&&c.definition=="BQ090"))
        .unwrap().action;
    if response {responsive(t,2,action);} else {t.action(2,action);}
}
fn bq033()->Value {
    let mut g=game();fund(&mut g,0,"JC125",3);
    let anchor=g.make_card("BQ009",2);let anchor_id=anchor.id.clone();
    g.sealed_cards.push(SealedCard{card:anchor,host_id:"society:p0".into()});
    held(&mut g,"JC125",2);held(&mut g,"JC125",2);
    let hand=held(&mut g,"BQ033",0);let mut t=Trace::new(g,"next20-bq033-real-anchor-payment".into(),700);
    t.action(0,Action{card_id:Some(hand),region:Some(0),cost_selected:Some(vec![anchor_id.clone()]),..Action::new("deploy")});
    assert_eq!(t.room.game.resources(0),0);
    assert!(!t.room.game.sealed_cards.iter().any(|s|s.card.id==anchor_id));
    let f=t.room.game.stack.last().unwrap().frame.as_ref().unwrap();
    assert!(matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids),PaidCost::BQ033Anchor(r)] if ids.len()==3&&r.payer==0&&r.anchor.owner==2&&r.anchor.id==anchor_id));
    t.until_pending();t.choose(vec!["p2".into()]);until_empty(&mut t);
    assert_eq!(t.room.game.players[2].hand.len(),1);
    assert!(t.room.game.players[2].deck.iter().any(|c|c.definition=="BQ009"));
    assert!(t.room.game.regions[0].cards.iter().any(|c|c.definition=="BQ033"));t.value()
}
fn bq034()->Value {
    let mut g=game();fund(&mut g,0,"JC029",2);fund(&mut g,0,"JC125",2);
    let c=g.make_card("XQ14",0);let blood=c.id.clone();g.players[0].graveyard.push(c);
    let hand=held(&mut g,"BQ034",0);let mut t=Trace::new(g,"next20-bq034-real-blood-entry".into(),701);
    t.action(0,Action{card_id:Some(hand),region:Some(0),..Action::new("deploy")});
    t.until_pending();t.choose(vec!["accept".into()]);t.until_pending();t.choose(vec![blood]);until_empty(&mut t);
    assert_eq!(t.room.game.resources(0),0);
    assert!(t.room.game.attachments.iter().any(|a|a.card.definition=="XQ14"&&t.room.game.board(&a.host_id).is_some_and(|(_,c)|c.definition=="BQ034")));t.value()
}
fn cq25()->Value {
    let mut g=game();let tiger=field(&mut g,"CQ25",0);let victim=field(&mut g,"JC125",2);
    g.players[0].deck.clear();let c=g.make_card("XG25",0);let ghost=c.id.clone();g.players[0].deck.push(c);
    let c=g.make_card("JC125",0);g.players[0].deck.push(c);
    fund(&mut g,2,"JC102",4);held(&mut g,"JC102",2);held(&mut g,"JC102",2);g.begin_window(Window::Action(g.team(2)));
    let mut t=Trace::new(g,"next20-cq25-departed-source-search".into(),702);
    let fire=|t:&mut Trace,target:&str,response:bool|{
        let a=t.room.game.legal_actions(2).into_iter().find(|l|l.action.kind=="play"&&l.action.target_id.as_deref()==Some(target)
            &&t.room.game.players[2].hand.iter().any(|c|Some(&c.id)==l.action.card_id.as_ref()&&c.definition=="JC102")).unwrap().action;
        if response{responsive(t,2,a);}else{t.action(2,a);}
    };
    fire(&mut t,&victim,false);t.until_pending();t.choose(vec!["accept".into()]);fire(&mut t,&tiger,true);
    t.until_pending();assert!(t.room.game.board(&tiger).is_none());t.choose(vec![ghost.clone()]);until_empty(&mut t);
    assert!(t.room.game.players[0].graveyard.iter().any(|c|c.definition=="XG25"&&c.id!=ghost));
    assert!(t.room.game.sealed_cards.is_empty());t.value()
}
fn xg25()->Value {
    let mut g=game();let ghost=field(&mut g,"XG25",0);let host=field(&mut g,"JC125",3);
    g.players[0].society_zone.secrecy=1;fund(&mut g,2,"XG25",4);held(&mut g,"BQ090",2);held(&mut g,"BQ090",2);
    g.begin_window(Window::Action(g.team(2)));let mut t=Trace::new(g,"next20-xg25-real-response-fallback-caller".into(),703);
    destroy(&mut t,&ghost,false);t.until_pending();
    assert_eq!(t.room.game.players[0].graveyard.iter().filter(|c|c.definition=="XG25").count(),0);
    t.choose(vec![host.clone()]);destroy(&mut t,&host,true);until_empty(&mut t);
    assert!(t.room.game.board(&host).is_none());
    assert_eq!(t.room.game.players[0].graveyard.iter().filter(|c|c.definition=="XG25").count(),1);
    assert_eq!(t.room.game.players[2].graveyard.iter().filter(|c|c.definition=="BQ090").count(),2);
    assert!(t.room.game.death_replacements.is_empty()&&t.room.game.effects.is_empty());t.value()
}
#[test]
fn next20_candidate_actual_room_four_closed_card_witnesses() {
    let mut cases=vec![bq033(),bq034(),cq25(),xg25()];
    for case in &mut cases {
        for step in case["steps"].as_array_mut().unwrap() {
            let room=RoomEnvelope::from_persisted(step["expected"]["state"].as_str().unwrap()).unwrap();
            let command:RoomCommand=serde_json::from_value(step["command"].clone()).unwrap();
            let duplicate=room.transition(step["seat"].as_u64().unwrap() as usize,Some(command),0).unwrap();
            // This reducer does not own HTTP command receipts. Record its real
            // duplicate probe, including idempotent BeginResponse acceptance;
            // Worker tests separately assert original-receipt replay without mutation.
            RoomEnvelope::from_persisted(&duplicate.state).unwrap();
            step["duplicate"]=serde_json::to_value(duplicate).unwrap();
        }
    }
    for id in ["BQ033","BQ009"] {assert!(!catalog::card(id).supported);}
    for id in ["BQ034","CQ25","XG25"] {assert!(catalog::card(id).supported);}
    if let Ok(file)=std::env::var("NEXT20_CANDIDATE_NATIVE_OUT") {
        use std::io::Write;
        let mut f=std::fs::OpenOptions::new().write(true).create_new(true).open(file).unwrap();
        f.write_all(&serde_json::to_vec(&json!({"format":"next20-candidate-room-native-v1","publicAdmission":false,
            "scope":"Explicit internal layouts; BQ009 seal is pre-arranged, no supply producer. Every later operation is an actual Room command.","catalog":catalog::catalog(),"cases":cases})).unwrap()).unwrap();
    }
}
#[test]
fn next20_candidate_public_new_and_join_reject_all_five_without_mutation() {
    for id in ["BQ033","BQ034","CQ25","XG25","BQ009"] {
        let mut draft=crate::deck::preset("watchers").unwrap();
        draft.cards.push(crate::catalog::DeckEntry{card_id:id.into(),count:1});
        assert!(Game::new_with_deck("candidate-public".into(),"PUBLIC".into(),"teams".into(),"P0".into(),draft.clone(),7).is_err());
        let mut game=Game::new("candidate-public".into(),"PUBLIC".into(),"teams".into(),"P0".into(),"watchers".into(),7).unwrap();
        let before=serde_json::to_string(&game).unwrap();assert!(game.join_with_deck("P1".into(),draft).is_err());
        assert_eq!(serde_json::to_string(&game).unwrap(),before);
    }
}
