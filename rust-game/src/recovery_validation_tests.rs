//! Recovery parity is a domain contract; Room keeps its own pacing checks.
//! Fixtures are native layouts, not evidence of production state corruption.
use crate::{model::*, room::RoomEnvelope, unit_support::*};
use crate::jc029_tests::{envelope, pass_top};

fn same_class(original:&Game, altered:serde_json::Value, rejected:bool) {
    assert!(Game::from_persisted(&serde_json::to_string(original).unwrap()).is_ok());
    let mut room=serde_json::to_value(envelope(original)).unwrap();
    assert!(RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_ok());
    room["game"]=altered.clone();
    let game=Game::from_persisted(&serde_json::to_string(&altered).unwrap());
    let wrapped=RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap());
    assert_eq!(game.is_err(),rejected,"domain fixture classification must be intentional");
    assert_eq!(wrapped.is_err(),rejected,"Game rejects={}, Room rejects={}; same domain snapshot must agree",game.is_err(),wrapped.is_err());
    if rejected {assert_eq!(game.unwrap_err(),wrapped.unwrap_err());}
}
#[test]
fn recovery_validation_batch3_paid_frame_cursor_rejected_by_both_entries() {
    let mut g=game();fund(&mut g,0,"WM030",4);let id=held(&mut g,"WM030",0);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});
    let mut altered=serde_json::to_value(&g).unwrap();altered["stack"][0]["frame"]["cursor"]=99.into();same_class(&g,altered,true);
}
#[test]
fn recovery_validation_batch4_unpaid_grave_draft_rejected_by_both_entries() {
    let mut g=game();let c=g.make_card("JZ51",0);let source=c.id.clone();g.players[0].graveyard.push(c);
    for _ in 0..3 {let c=g.make_card("XQ46",0);g.players[0].graveyard.push(c);}
    act(&mut g,0,Action{card_id:Some(source),ability_id:Some(crate::batch4_wendigo::GRAVE.into()),..Action::new("activate")});
    let mut altered=serde_json::to_value(&g).unwrap();altered["pending"]["resolution"]["JZ51Payment"]["draft"]["turn"]=999.into();same_class(&g,altered,true);
}
#[test]
fn recovery_validation_impossible_destruction_score_rejected_by_both_entries() {
    let g=game();let mut altered=serde_json::to_value(&g).unwrap();altered["players"][0]["destruction_markers"]=1.into();same_class(&g,altered,true);
}
#[test]
fn recovery_validation_honest_paid_frame_with_departed_target_restores_and_fizzles() {
    let mut g=game();fund(&mut g,0,"JZ40",4);let id=held(&mut g,"JZ40",0);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("play")});top(&mut g);
    let source=g.attachments.iter().find(|a|a.card.definition=="JZ40").unwrap().card.id.clone();let target=field(&mut g,"JC125",0);
    act(&mut g,0,Action{card_id:Some(source),ability_id:Some(crate::batch4_skull::KEY.into()),target_id:Some(target.clone()),..Action::new("activate")});
    let rescuer=field(&mut g,"JC075",0);fund(&mut g,0,"JC075",2);
    for _ in 0..4 {if g.priority_team==g.team(0){break;}pass(&mut g);}
    let action=g.legal_actions(0).into_iter().find(|a|a.action.kind=="activate"&&a.action.card_id.as_deref()==Some(&rescuer)&&a.action.target_id.as_deref()==Some(&target)).unwrap().action;
    act(&mut g,0,action);pass_top(&mut g);assert!(g.board(&target).is_none());assert_eq!(g.stack.len(),1);
    same_class(&g,serde_json::to_value(&g).unwrap(),false);
    let before=g.resources(0);pass_top(&mut g);assert!(g.stack.is_empty());assert_eq!(g.resources(0),before);
    assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JZ40").count(),1);
    same_class(&g,serde_json::to_value(&g).unwrap(),false);restore(&mut g);
}
