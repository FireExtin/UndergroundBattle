//! Recovery parity is a domain contract; Room keeps its own pacing checks.
//! Fixtures are native layouts, not evidence of production state corruption.
use crate::{model::*, room::RoomEnvelope, unit_support::*};
use crate::jc029_tests::{envelope, pass_top};

fn same_class(original:&Game, altered:serde_json::Value, rejected:bool) {
    assert!(Game::from_persisted(&serde_json::to_string(original).unwrap()).is_ok());
    let valid_room=envelope(original);
    let mut room=serde_json::to_value(&valid_room).unwrap();
    assert!(RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_ok());
    room["game"]=altered.clone();
    let game=Game::from_persisted(&serde_json::to_string(&altered).unwrap());
    let wrapped=RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap());
    assert_eq!(game.is_err(),rejected,"domain fixture classification must be intentional");
    assert_eq!(wrapped.is_err(),rejected,"Game rejects={}, Room rejects={}; same domain snapshot must agree",game.is_err(),wrapped.is_err());
    if let Ok(dir)=std::env::var("HEGEMONY_RECOVERY_FIXTURE_DIR") {
        use std::sync::atomic::{AtomicUsize,Ordering};
        static SEQ:AtomicUsize=AtomicUsize::new(0);
        std::fs::create_dir_all(&dir).unwrap();
        let record=serde_json::json!({"name":std::thread::current().name(),"shouldReject":rejected,
            "validRoomState":serde_json::to_string(&valid_room).unwrap(),
            "alteredRoomState":serde_json::to_string(&room).unwrap(),
            "alteredGameState":serde_json::to_string(&altered).unwrap(),
            "nativeError":game.as_ref().err(),
            "validViews":(0..4).map(|s|serde_json::to_value(valid_room.view(s,0)).unwrap()).collect::<Vec<_>>()});
        std::fs::write(std::path::Path::new(&dir).join(format!("case-{}.json",SEQ.fetch_add(1,Ordering::Relaxed))),serde_json::to_vec(&record).unwrap()).unwrap();
    }
    if rejected {assert_eq!(game.unwrap_err(),wrapped.unwrap_err());}
}
#[test]
fn recovery_validation_batch3_paid_frame_cursor_rejected_by_both_entries() {
    let mut g=game();fund(&mut g,0,"WM030",4);let id=held(&mut g,"WM030",0);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});
    let mut altered=serde_json::to_value(&g).unwrap();altered["stack"][0]["frame"]["cursor"]=99.into();same_class(&g,altered,true);
}
#[test]
fn recovery_validation_impossible_destruction_score_rejected_by_both_entries() {
    let g=game();let mut altered=serde_json::to_value(&g).unwrap();altered["players"][0]["destruction_markers"]=1.into();same_class(&g,altered,true);
}
#[test]
fn recovery_validation_honest_paid_multi_target_frame_restores_then_fizzles() {
    let mut g=game();g.seed=u64::MAX-127;g.random=g.seed;fund(&mut g,0,"JC057",1);let spell=held(&mut g,"JC062",0);
    let first=field(&mut g,"JC125",0);let second=field(&mut g,"JC125",2);
    g.board_mut(&first).unwrap().face_down=true;g.board_mut(&second).unwrap().face_down=true;
    let deck=g.players[0].deck.len();
    act(&mut g,0,Action{card_id:Some(spell),selected:Some(vec![first.clone(),second.clone()]),..Action::new("play")});
    for _ in 0..4 {if g.priority_team==g.team(0){break;}pass(&mut g);}
    act(&mut g,0,Action{card_id:Some(first.clone()),..Action::new("reveal")});pass_top(&mut g);
    assert!(g.board(&first).is_none());assert_eq!(g.stack.len(),1);
    same_class(&g,serde_json::to_value(&g).unwrap(),false);let resources=g.resources(0);
    pass_top(&mut g);assert!(g.stack.is_empty()&&g.pending.is_none());assert_eq!(g.resources(0),resources);
    assert_eq!(g.players[0].deck.len(),deck);assert!(g.board(&second).unwrap().1.face_down);
    assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JC062").count(),1);
    same_class(&g,serde_json::to_value(&g).unwrap(),false);restore(&mut g);
}
