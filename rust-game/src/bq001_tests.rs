//! Explicit unit layouts; all revelation/sealing/draw operations use actual
//! engine actions. These cases do not claim a public construction trajectory.
use crate::{catalog,model::*,rules::*,unit_support::*};
const ID:&str="BQ001";
const CHOICE:&str="bq001-star-hand";
fn require_card(){assert!(catalog::catalog().cards.iter().any(|c|c.id==ID),"original BQ001 is not registered");}
fn yellow()->String {catalog::catalog().cards.iter().find(|c|c.supported&&c.color=="黄"&&matches!(c.kind.as_str(),"character"|"spell"|"attachment")).unwrap().id.clone()}
fn stop_at(g:&mut Game,kind:&str){for _ in 0..100{if let Some(p)=&g.pending{assert_eq!(p.choice.kind,kind,"unexpected choice {}",p.choice.title);return;}assert!(!g.stack.is_empty(),"missing choice {kind}");pass(g);}panic!("bounded choice {kind}");}
fn decline_pending(g:&mut Game){for _ in 0..100{if let Some(p)=&g.pending{assert!(matches!(p.resolution,ChoiceResolution::Declare{..}),"unexpected choice {}",p.choice.kind);assert_eq!(p.choice.min,Some(0));choose(g,vec![]);}else if !g.stack.is_empty(){pass(g);}else{return;}}panic!("bounded optional trigger cleanup");}
#[test]
fn bq001_printed_whole_card_is_registered_closed_and_image_specific(){
 require_card();let d=catalog::card(ID);assert_eq!((d.cost,d.defense,d.unique,d.supported),(1,Some(1),false,false));
 assert_eq!(d.color,"黄");assert_eq!(d.loyalty,["黄色"]);assert_eq!(d.magic,"心灵");assert_eq!(d.subtypes,["人类","法师","学者"]);assert_eq!(d.permanent_icons,Icons{investigation:0,combat:0,influence:1});assert_eq!(d.temporary_icons,Icons::default());
 let program=serde_json::to_value(definition(ID)).unwrap();assert_eq!(program["modifiers"],serde_json::json!(["BQ001MysticAnchorShown"]));
 assert_eq!(program["abilities"].as_array().unwrap().len(),1);assert_eq!(program["abilities"][0]["event"],"Reveal");
}
#[test]
fn bq001_paid_reveal_selects_only_actor_star_hand_then_seals_and_draws_once(){
 require_card();let mut g=game();fund(&mut g,0,&yellow(),3);let source=field(&mut g,ID,0);g.board_mut(&source).unwrap().face_down=true;
 let anchor=held(&mut g,"BQ009",0);let nonstar=held(&mut g,"JC125",0);let other=held(&mut g,"BQ009",2);let money=g.resources(0);let deck=g.players[0].deck.len();let hand=g.players[0].hand.len();
 act(&mut g,0,Action{card_id:Some(source),..Action::new("reveal")});assert_eq!(g.resources(0),money-1);
 stop_at(&mut g,"trigger");choose(&mut g,vec!["accept".into()]);stop_at(&mut g,CHOICE);
 let p=g.pending.clone().unwrap();assert_eq!(p.seat,0);assert_eq!(p.choice.options.iter().map(|o|o.id.as_str()).collect::<Vec<_>>(),[anchor.as_str()]);
 for (seat,id) in [(0,nonstar),(0,other),(2,anchor.clone())]{reject(&mut g,seat,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![id]),..Action::new("choose")});}
 choose(&mut g,vec![anchor.clone()]);assert_eq!(g.resources(0),money-1);assert_eq!(g.players[0].deck.len(),deck-1);assert_eq!(g.players[0].hand.len(),hand);
 assert!(!g.players[0].hand.iter().any(|c|c.id==anchor));let sealed=g.sealed_cards.iter().find(|s|s.card.definition=="BQ009").unwrap();assert_eq!(sealed.host_id,"society:p0");assert_eq!(sealed.card.owner,0);assert_ne!(sealed.card.id,anchor);
 reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec![anchor]),..Action::new("choose")});decline_pending(&mut g);
 assert_eq!(g.players[0].deck.len(),deck-1);assert_eq!(g.sealed_cards.len(),1);assert!(g.effects.is_empty());
}
#[test]
fn bq001_normal_face_up_deploy_does_not_seal_or_draw(){
 require_card();let mut g=game();fund(&mut g,0,&yellow(),1);let source=held(&mut g,ID,0);held(&mut g,"BQ009",0);let deck=g.players[0].deck.len();
 act(&mut g,0,Action{card_id:Some(source),region:Some(0),..Action::new("deploy")});drain(&mut g);assert!(g.sealed_cards.is_empty());assert_eq!(g.players[0].deck.len(),deck);assert!(g.pending.is_none());
}
