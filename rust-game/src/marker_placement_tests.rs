use crate::{batch4_markers::MarkerKind as K, model::*, unit_support::*};
fn roundtrip(g: &mut Game) {
    restore(g);
    let room=crate::room::RoomEnvelope::from_game(g.clone());
    let r=crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).unwrap();
    assert_eq!(serde_json::to_value(&g.first_marker_placements).unwrap(),serde_json::to_value(&r.game.first_marker_placements).unwrap());
}
#[test]
fn marker_placement_add_remove_add_retains_first_event_across_game_and_room() {
    let mut g=game();let id=field(&mut g,"JC125",0);
    let (first,is_first)=g.place_board_markers(&id,K::Time,2).unwrap();assert!(is_first);
    g.board_mut(&id).unwrap().time_markers=0;
    roundtrip(&mut g);
    let (second,is_first)=g.place_board_markers(&id,K::Shield,1).unwrap();assert!(!is_first);
    assert_eq!((first.before,first.placed,second.before,second.placed),(0,2,0,1));
    assert_eq!(g.first_marker_placements,[first]);roundtrip(&mut g);
}
#[test]
fn marker_placement_each_physical_kind_zero_missing_saturation_and_new_turn() {
    let mut g=game();let id=field(&mut g,"JC125",0);
    assert!(g.place_board_markers(&id,K::Time,0).is_none());
    assert!(g.place_board_markers("missing",K::Time,1).is_none());
    assert!(g.first_marker_placements.is_empty());
    for (i,k) in K::ALL.into_iter().enumerate(){let(e,first)=g.place_board_markers(&id,k,1).unwrap();assert_eq!(first,i==0);assert_eq!(e.kind.count(&e.carrier),1);}
    g.board_mut(&id).unwrap().time_markers=u32::MAX;
    assert!(g.place_board_markers(&id,K::Time,1).is_none());
    g.effect(Effect::NextTurn).unwrap();
    assert!(g.first_marker_placements.is_empty());
    let(_,first)=g.place_board_markers(&id,K::Shield,2).unwrap();assert!(first);
    assert_eq!(g.first_marker_placements.len(),1);roundtrip(&mut g);
}
#[test]
fn marker_placement_departure_does_not_rewrite_original_receipt() {
    let mut g=game();let id=field(&mut g,"JC125",0);
    g.place_board_markers(&id,K::Time,1).unwrap();
    let receipt=g.first_marker_placements[0].clone();
    g.remove_dead(&id,RemovalCause::Destroy);
    assert_eq!(g.first_marker_placements,[receipt]);roundtrip(&mut g);
    let next=field(&mut g,"JC125",0);assert_ne!(id,next);
    assert!(g.place_board_markers(&next,K::Time,1).unwrap().1);roundtrip(&mut g);
}
#[test]
fn marker_placement_corrupt_receipts_reject_both_restore_entries() {
    let mut g=game();let id=field(&mut g,"JC125",0);g.place_board_markers(&id,K::Time,1).unwrap();
    for n in 0..5 {let mut bad=g.clone();match n{0=>bad.first_marker_placements[0].placed=0,1=>bad.first_marker_placements[0].before=9,2=>bad.first_marker_placements[0].turn+=1,3=>bad.first_marker_placements.push(bad.first_marker_placements[0].clone()),_=>bad.first_marker_placements[0].carrier.controller=99}
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
        let r=crate::room::RoomEnvelope::from_game(bad);assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).is_err());
    }
}

#[test]
fn marker_placement_real_wound_and_atomic_transfer_record_before_removal() {
    let mut g=game();let from=field(&mut g,"JC125",0);let to=field(&mut g,"JC125",0);
    g.place_wounds(std::collections::BTreeMap::from([(from.clone(),1)])).unwrap();
    assert_eq!(g.first_marker_placements[0].kind,K::Wound);
    let prepared=g.prepare_marker_move(&from,&to).unwrap();
    g.commit_marker_move_counters(prepared).unwrap();
    assert_eq!(g.board(&from).unwrap().1.wounds,0);
    assert_eq!(g.board(&to).unwrap().1.wounds,1);
    assert_eq!(g.first_marker_placements.len(),2);
    assert_eq!(g.first_marker_placements[0].carrier.wounds,1);
    roundtrip(&mut g);
}
