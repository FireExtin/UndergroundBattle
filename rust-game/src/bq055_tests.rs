use crate::{bq055 as b,core_zone::CoreZone,model::*,unit_support::*};
use crate::jc029_tests::{pass_top,envelope};
fn setup(parked:bool)->(Game,String){let mut g=game();let core=if parked{Some(g.make_card("BQ046",0))}else{field(&mut g,"BQ046",0);None};g.core_zones.push(CoreZone{rest:0,owner:0,definition:"BQ046".into(),card:core});let observer=field(&mut g,b::ID,0);restore(&mut g);(g,observer)}
#[test]
fn bq055_discount_only_own_selected_core_and_only_live_personal_observer(){
 let(mut g,observer)=setup(true);let core=g.core_zones[0].card.as_ref().unwrap().clone();assert_eq!(g.effective_cost(0,&core),2);assert_eq!(g.paid_reveal_cost(0,&core),2);
 let other=g.make_card("JC029",0);assert_eq!(g.bq055_reduction(0,&other),0);assert_eq!(g.bq055_reduction(1,&core),0);
 g.board_mut(&observer).unwrap().controller=1;assert_eq!(g.effective_cost(0,&core),3);g.board_mut(&observer).unwrap().controller=0;g.board_mut(&observer).unwrap().face_down=true;assert_eq!(g.paid_reveal_cost(0,&core),3);
}
#[test]
fn bq055_public_paid_deploy_and_no_conceal(){let mut g=game();fund(&mut g,0,"JC056",1);let id=held(&mut g,b::ID,0);reject(&mut g,0,Action{card_id:Some(id.clone()),region:Some(0),..Action::new("conceal")});act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});pass_top(&mut g);assert_eq!(g.resources(0),0);assert!(g.regions[0].cards.iter().any(|c|c.definition==b::ID));}
#[test]
fn bq055_core_zone_is_not_in_play_for_hidden_contest_bonus(){let(g,_)=setup(true);assert!(g.bq055_shown_witness(0).is_none());let(mut g,_)=setup(false);assert!(g.bq055_shown_witness(0).is_some());g.regions[0].cards.iter_mut().find(|c|c.definition=="BQ046").unwrap().face_down=true;assert!(g.bq055_shown_witness(0).is_none());}
fn contest()->(Game,String){
 let(mut g,observer)=setup(false);for seat in 0..4{g.players[seat].deck.clear();for _ in 0..5{let c=g.make_card("JC125",seat);g.players[seat].deck.push(c);}}
 fund(&mut g,0,"WM022",12);let id=field(&mut g,"WM022",0);g.board_mut(&id).unwrap().face_down=true;act(&mut g,0,Action{card_id:Some(id),..Action::new("reveal")});
 for _ in 0..32 {if g.pending.is_some(){break;}pass(&mut g);}assert_eq!(g.pending.as_ref().unwrap().choice.kind,crate::mystic_contest::COMMIT_KIND);
 for _ in 0..4 {let seat=g.pending.as_ref().unwrap().seat;let id=g.players[seat].hand.iter().find(|c|c.definition=="JC125").unwrap().id.clone();choose(&mut g,vec![id]);}
 assert_eq!(g.mystic_history.last().unwrap().shown.iter().find(|s|s.seat==0).unwrap().value,3);(g,observer)
}
#[test]
fn bq055_actual_contest_freezes_original_core_and_observer_after_they_leave(){let(mut g,observer)=contest();g.remove_dead(&observer,RemovalCause::Destroy);restore(&mut g);let o=g.mystic_history.last().unwrap();assert_eq!(g.bq055_shown_bonus(o.shown.iter().find(|s|s.seat==0).unwrap()).unwrap(),3);}
#[test]
fn bq055_corrupt_contest_witness_rejected_in_game_and_room(){let(g,_)=contest();for case in 0..3 {let mut bad=g.clone();let shown=bad.mystic_history.last_mut().unwrap().shown.iter_mut().find(|s|s.seat==0).unwrap();let w=shown.core_support.as_mut().unwrap();match case{0=>w.core.card.face_down=true,1=>w.observers.push(w.observers[0].clone()),_=>w.core.card.owner=2};assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());let mut room=envelope(&g);room.game=bad;assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());}}
pub(crate) fn native_case()->serde_json::Value{
 use crate::abi112_review_play_tests::Trace;
 let(mut g,_)=setup(false);for seat in 0..4 {g.players[seat].deck.clear();for _ in 0..5 {let c=g.make_card("JC125",seat);g.players[seat].deck.push(c);}}
 fund(&mut g,0,"WM022",12);let id=field(&mut g,"WM022",0);g.board_mut(&id).unwrap().face_down=true;let mut t=Trace::new(g,"bq055-real-contest".into(),550);t.action(0,Action{card_id:Some(id),..Action::new("reveal")});t.until_pending();
 for _ in 0..4 {let seat=t.room.game.pending.as_ref().unwrap().seat;let id=t.room.game.players[seat].hand.iter().find(|c|c.definition=="JC125").unwrap().id.clone();t.choose(vec![id]);}
 for _ in 0..32 {if let Some(p)=t.room.game.pending.clone(){let pick=p.choice.options.iter().find(|o|o.id=="decline").or_else(||p.choice.options.first()).map(|o|vec![o.id.clone()]).unwrap_or_default();t.choose(pick);}else if !t.room.game.stack.is_empty(){t.pass();}else{break;}}
 assert!(t.room.game.pending.is_none()&&t.room.game.stack.is_empty());assert_eq!(t.room.game.mystic_history.last().unwrap().shown.iter().find(|s|s.seat==0).unwrap().value,3);t.value()
}
