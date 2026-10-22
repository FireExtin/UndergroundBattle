use crate::{catalog,model::*,rules::*,unit_support::*};
use crate::batch4_chance::{TAX,REVEAL,TaxOutcome};
fn layout()->(Game,String,String){let mut g=game();let chance=field(&mut g,"WM058",0);let target=field(&mut g,"JC084",0);(g,chance,target)}
fn pending(g:&mut Game){for _ in 0..128{if g.pending.is_some(){restore(g);return;}if !g.effects.is_empty(){g.drive().unwrap();restore(g);}else{pass(g);}}panic!("bounded Chance trigger");}
fn cast(g:&mut Game,id:&str,actor:usize,target:&str,funding:usize){g.begin_window(Window::Action(g.team(actor)));fund(g,actor,id,catalog::card(id).cost as usize+funding);let h=held(g,id,actor);act(g,actor,Action{card_id:Some(h),target_id:Some(target.into()),..Action::new("play")});}
fn accept(g:&mut Game){pending(g);assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration:d,..}if d.ability.key==TAX));choose(g,vec!["accept".into()]);}
fn payment(g:&mut Game){for _ in 0..96{if let Some(p)=&g.pending{assert_eq!(p.choice.kind,"wm058-payment");restore(g);return;}pass(g);}panic!("bounded Chance payment");}
fn drain(g:&mut Game){for _ in 0..192{if g.pending.is_some(){resolve_choice(g);}else if !g.effects.is_empty(){g.drive().unwrap();restore(g);}else if !g.stack.is_empty(){pass(g);}else{return;}}panic!("bounded Chance settlement");}
fn bad(g:&Game){assert!(Game::from_persisted(&serde_json::to_string(g).unwrap()).is_err());}
#[test]
fn batch4_chance_exact_original_fields_unique_and_closed_registry(){
 let c=catalog::card("WM058");assert_eq!((c.cost,c.defense,c.unique,c.supported),(3,Some(1),true,false));assert_eq!(c.loyalty,["黑色","黑色","黑色"]);assert_eq!(c.subtypes,["人类","罪犯"]);assert_eq!(c.subtitle.as_deref(),Some("“清洁工”"));assert_eq!(c.permanent_icons,Icons{combat:1,influence:1,..Icons::default()});assert_eq!(c.temporary_icons,Icons{combat:1,..Icons::default()});
 let mut defs=crate::rules::definitions().clone();defs.get_mut("WM058").unwrap().abilities[0].ops.clear();assert!(crate::rules::validate_definitions(&defs).is_err());
 // JZ55 is the existing nonrespondable card-body exception. The actual
 // command regression below checks its existing immediate ordering.
 let mut immediate=vec![];for d in &catalog::catalog().cards{if !matches!(d.kind.as_str(),"spell"|"attachment")||!catalog::runtime_implemented(d){continue;}for a in &definition(&d.id).abilities{if a.event.is_none()&&!a.activation_only&&a.targets.iter().any(|t|t.zone==Zone::Board)&&a.response_policy==ResponsePolicy::Immediate{immediate.push(format!("{}:{}",d.id,a.key));}}}assert_eq!(immediate,["JZ55:destroy-unique"]);
}

#[test]
fn batch4_chance_paid_normal_deploy_requires_three_black_and_does_not_remove_influence(){
 let mut g=game();fund(&mut g,0,"JC125",3);let h=held(&mut g,"WM058",0);reject(&mut g,0,Action{card_id:Some(h.clone()),region:Some(0),..Action::new("deploy")});fund(&mut g,0,"WM058",3);g.regions[0].influence=[1,2];let before=g.resources(0);act(&mut g,0,Action{card_id:Some(h),region:Some(0),..Action::new("deploy")});top(&mut g);assert_eq!(g.resources(0),before-3);assert_eq!(g.regions[0].influence,[1,2]);assert!(g.wm058_receipts.is_empty());restore(&mut g);
}

#[test]
fn batch4_chance_real_enemy_card_without_remaining_resource_is_countered_once_without_refund(){
 let(mut g,_,target)=layout();cast(&mut g,"JC102",2,&target,0);assert_eq!(g.resources(2),0);accept(&mut g);assert!(g.board(&target).is_some());top(&mut g);assert!(g.board(&target).is_some());assert!(g.stack.is_empty());assert_eq!(g.players[2].graveyard.iter().filter(|c|c.definition=="JC102").count(),1);assert_eq!(g.resources(2),0);assert!(matches!(g.wm058_receipts[0].outcome,TaxOutcome::Countered{..}));restore(&mut g);
}

#[test]
fn batch4_chance_original_card_controller_pays_one_and_card_resolves_only_afterwards(){
 let(mut g,_,target)=layout();cast(&mut g,"JC102",2,&target,1);accept(&mut g);payment(&mut g);assert_eq!(g.resources(2),1);assert!(g.board(&target).is_some());let p=g.pending.clone().unwrap();assert_eq!(p.seat,2);for s in [0,1,3]{assert!(g.view(s).pending_choice.is_none());}let a=Action{choice_id:Some(p.choice.id),selected:Some(vec!["pay".into()]),..Action::new("choose")};reject(&mut g,0,a.clone());let mut extra=a.clone();extra.cost_selected=Some(vec![]);reject(&mut g,2,extra);act(&mut g,2,a.clone());reject(&mut g,2,a);assert_eq!(g.resources(2),0);assert!(g.board(&target).is_some());assert_eq!(g.stack.len(),1);top(&mut g);assert!(g.board(&target).is_none());assert_eq!(g.wm058_receipts.len(),1);assert!(matches!(g.wm058_receipts[0].outcome,TaxOutcome::Paid{..}));restore(&mut g);
}

#[test]
fn batch4_chance_caster_can_refuse_despite_funding_and_observer_can_decline_entire_trigger(){
 for observer_declines in [false,true]{let(mut g,_,target)=layout();cast(&mut g,"JC102",2,&target,1);pending(&mut g);choose(&mut g,if observer_declines{vec![]}else{vec!["accept".into()]});if observer_declines{assert!(matches!(g.wm058_receipts[0].outcome,TaxOutcome::Declined));top(&mut g);assert!(g.board(&target).is_none());}else{payment(&mut g);choose(&mut g,vec!["decline".into()]);assert!(matches!(g.wm058_receipts[0].outcome,TaxOutcome::Countered{..}));assert!(g.board(&target).is_some());assert!(g.stack.is_empty());}assert_eq!(g.resources(2),1);assert_eq!(g.wm058_receipts.len(),1);restore(&mut g);}
}

#[test]
fn batch4_chance_enemy_attachment_is_a_card_transaction_and_can_be_paid_or_countered(){
 for pay in [false,true]{let(mut g,_,target)=layout();cast(&mut g,"JZ41",2,&target,1);accept(&mut g);payment(&mut g);choose(&mut g,vec![if pay{"pay"}else{"decline"}.into()]);if pay{top(&mut g);}assert_eq!(g.attachments.iter().filter(|a|a.card.definition=="JZ41"&&a.host_id==target).count(),usize::from(pay));assert_eq!(g.resources(2),u32::from(!pay));drain(&mut g);assert_eq!(g.wm058_receipts.len(),1);restore(&mut g);}
}

#[test]
fn batch4_chance_personal_not_teammate_target_and_enemy_not_friendly_card_gate(){
 for (target_seat,caster,id) in [(1,2,"WM060"),(0,1,"WM060"),(0,2,"JC032")]{let(mut g,_,old)=layout();g.return_hand(&old);let target=field(&mut g,id,target_seat);cast(&mut g,"JC078",caster,&target,0);assert!(g.pending.is_none());assert!(g.wm058_receipts.is_empty());top(&mut g);assert!(g.wm058_receipts.is_empty());restore(&mut g);}
}

#[test]
fn batch4_chance_hostile_activated_character_ability_never_counts_as_a_card(){
 let(mut g,_,target)=layout();field(&mut g,"JC003",2);g.begin_window(Window::Action(1));fund(&mut g,2,"JC003",4);let a=g.legal_actions(2).into_iter().find(|a|a.action.kind=="activate"&&a.action.target_id.as_deref()==Some(&target)).expect("real Veil Guardian exhaust target ability").action;act(&mut g,2,a);assert!(g.pending.is_none());assert!(g.wm058_receipts.is_empty());top(&mut g);assert!(g.board(&target).unwrap().1.exhausted);assert!(g.wm058_receipts.is_empty());restore(&mut g);
}

#[test]
fn batch4_chance_region_targeting_mass_destruction_does_not_target_each_criminal(){
 let(mut g,chance,target)=layout();g.begin_window(Window::Action(1));fund(&mut g,2,"JC050",5);let h=held(&mut g,"JC050",2);act(&mut g,2,Action{card_id:Some(h),..Action::new("play")});assert!(g.pending.is_none());pending(&mut g);assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id=="region:0"));choose(&mut g,vec!["region:0".into()]);assert!(g.board(&chance).is_none()&&g.board(&target).is_none());assert!(g.wm058_receipts.is_empty());restore(&mut g);
}

#[test]
fn batch4_chance_existing_nonrespondable_jz55_finishes_before_observer_and_never_charges_a_missing_card(){
 let(mut g,chance,_)=layout();cast(&mut g,"JZ55",2,&chance,1);assert!(g.board(&chance).is_none());assert_eq!(g.players[2].graveyard.iter().filter(|c|c.definition=="JZ55").count(),1);assert!(g.stack.is_empty());assert_eq!(g.resources(2),1);accept(&mut g);top(&mut g);assert!(g.stack.is_empty()&&g.pending.is_none());assert_eq!(g.resources(2),1);assert!(matches!(g.wm058_receipts[0].outcome,TaxOutcome::NoCard{..}));restore(&mut g);
}

#[test]
fn batch4_chance_frozen_source_and_original_target_can_leave_before_tax_resolves(){
 let(mut g,chance,target)=layout();cast(&mut g,"JC102",2,&target,1);accept(&mut g);g.return_hand(&chance);g.return_hand(&target);restore(&mut g);payment(&mut g);choose(&mut g,vec!["pay".into()]);assert_eq!(g.resources(2),0);top(&mut g);assert_eq!(g.players[0].hand.iter().filter(|c|matches!(c.definition.as_str(),"WM058"|"JC084")).count(),2);assert_eq!(g.wm058_receipts.len(),1);restore(&mut g);
}

#[test]
fn batch4_chance_already_removed_card_never_takes_an_extra_resource(){
 let(mut g,_,target)=layout();cast(&mut g,"JC102",2,&target,1);accept(&mut g);let item=g.stack.remove(0);g.effects.push_back(Effect::Bury{card:item.card.unwrap()});g.drive().unwrap();restore(&mut g);top(&mut g);assert_eq!(g.resources(2),1);assert!(g.board(&target).is_some());assert!(matches!(g.wm058_receipts[0].outcome,TaxOutcome::NoCard{..}));restore(&mut g);
}

fn reveal_layout()->(Game,String){let mut g=game();fund(&mut g,0,"WM058",7);let h=held(&mut g,"WM058",0);act(&mut g,0,Action{card_id:Some(h),region:Some(0),..Action::new("conceal")});let h=g.regions[0].cards[0].id.clone();g.regions[0].influence=[2,3];g.regions[1].influence=[1,2];act(&mut g,0,Action{card_id:Some(h),..Action::new("reveal")});pending(&mut g);let id=g.regions[0].cards[0].id.clone();(g,id)}

#[test]
fn batch4_chance_paid_reveal_removes_only_original_region_enemy_influence_or_can_decline(){
 for decline in [false,true]{let(mut g,_)=reveal_layout();assert_eq!(g.resources(0),3);assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration:d,..}if d.ability.key==REVEAL));choose(&mut g,if decline{vec![]}else{vec!["accept".into()]});if !decline{top(&mut g);}assert_eq!(g.regions[0].influence,[2,if decline{3}else{0}]);assert_eq!(g.regions[1].influence,[1,2]);restore(&mut g);}
}

#[test]
fn batch4_chance_reveal_keeps_original_region_when_source_moves_and_rejects_replacement(){
 for replace in [false,true]{let(mut g,id)=reveal_layout();choose(&mut g,vec!["accept".into()]);if replace{g.regions[0].card=g.make_card("DQJC115",0);}else{g.move_on_board_instance(&id,1);}restore(&mut g);top(&mut g);assert_eq!(g.regions[0].influence,[2,if replace{3}else{0}]);assert_eq!(g.regions[1].influence,[1,2]);restore(&mut g);}
}

#[test]
fn batch4_chance_target_event_order_with_phoebe_can_counter_before_or_after_her_draw(){
 for tax_first in [false,true]{let(mut g,_,old)=layout();g.return_hand(&old);let target=field(&mut g,"JZ24",0);field(&mut g,"CQ07",0);g.players[0].deck=(0..6).map(|_|g.make_card("JC125",0)).collect();let before=g.players[0].hand.len();cast(&mut g,"JC078",2,&target,0);pending(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"batch3-entry-order");let desired=if tax_first{TAX}else{crate::batch3_phoebe::TARGETED};let option=g.pending.as_ref().unwrap().choice.options.iter().find(|o|o.id.ends_with(desired)).unwrap().id.clone();choose(&mut g,vec![option]);
 for _ in 0..16{let Some(p)=g.pending.clone()else{break;};assert!(matches!(p.resolution,ChoiceResolution::Declare{..}));choose(&mut g,vec!["accept".into()]);}assert_eq!(g.stack.len(),3);while !g.stack.is_empty(){top(&mut g);}assert_eq!(g.players[0].hand.len(),before+1);assert!(g.board(&target).is_some());assert_eq!(g.wm058_receipts.len(),1);assert!(matches!(g.wm058_receipts[0].outcome,TaxOutcome::Countered{..}));restore(&mut g);}
}

#[test]
fn batch4_chance_duplicate_and_corrupt_target_observations_payment_frames_and_spent_receipts_are_rejected(){
 let(mut g,_,target)=layout();cast(&mut g,"JC102",2,&target,1);pending(&mut g);let ChoiceResolution::Declare{declaration:d,..}=&g.pending.as_ref().unwrap().resolution else{panic!()};let original=d.clone();
 for n in 0..7{let mut wrong=g.clone();let ChoiceResolution::Declare{declaration:d,..}=&mut wrong.pending.as_mut().unwrap().resolution else{panic!()};let Op::WM058Tax{observation:Some(o)}=&mut d.ability.ops[0]else{panic!()};match n{0=>o.turn+=1,1=>o.event_id=o.target.card.id.clone(),2=>o.target.card.controller=1,3=>o.subtypes.clear(),4=>o.card.card.controller=0,5=>o.ability_key="not-the-card-body".into(),_=>d.ability.ops.push(Op::WM058RemoveEnemyInfluence)}bad(&wrong);}
 let mut wrong=g.clone();wrong.effects.push_back(Effect::Declare{declaration:original.clone()});bad(&wrong);choose(&mut g,vec!["accept".into()]);payment(&mut g);let mut wrong=g.clone();let ChoiceResolution::Frame{frame:f,..}=&mut wrong.pending.as_mut().unwrap().resolution else{panic!()};f.cursor=0;bad(&wrong);choose(&mut g,vec!["pay".into()]);let mut wrong=g.clone();wrong.effects.push_back(Effect::Declare{declaration:original.clone()});bad(&wrong);assert!(g.choose_declaration(original,DeclareChoice::Accept,vec!["accept".into()]).is_err());let mut wrong=g.clone();wrong.wm058_receipts.push(wrong.wm058_receipts[0].clone());bad(&wrong);top(&mut g);restore(&mut g);
}

#[test]
fn batch4_chance_hidden_criminal_front_does_not_satisfy_character_target_trigger(){
 let(mut g,_,target)=layout();let hidden=g.hide_in_place_successor(&target).unwrap().id;g.begin_window(Window::Action(1));fund(&mut g,2,"JC063",2);let h=held(&mut g,"JC063",2);act(&mut g,2,Action{card_id:Some(h),target_id:Some(hidden.clone()),option:Some("return".into()),..Action::new("play")});assert!(g.pending.is_none());top(&mut g);assert!(g.board(&hidden).is_none());assert!(g.wm058_receipts.is_empty());restore(&mut g);
}

#[test]
fn batch4_chance_real_nested_enemy_card_chain_keeps_independent_events_and_consumes_each_once(){
 let(mut g,_,target)=layout();let second=held(&mut g,"JC102",2);cast(&mut g,"JC102",2,&target,3);accept(&mut g);assert_eq!(g.resources(2),3);assert!(g.legal_actions(2).iter().any(|a|a.action.kind=="play"&&a.action.card_id.as_deref()==Some(&second)&&a.action.target_id.as_deref()==Some(&target)));
 act(&mut g,2,Action{card_id:Some(second),target_id:Some(target.clone()),..Action::new("play")});accept(&mut g);payment(&mut g);choose(&mut g,vec!["pay".into()]);assert_eq!(g.resources(2),0);top(&mut g);assert!(g.board(&target).is_none());top(&mut g);assert!(g.stack.is_empty());assert_eq!(g.wm058_receipts.len(),2);assert_ne!(g.wm058_receipts[0].observation.event_id,g.wm058_receipts[1].observation.event_id);assert!(matches!(g.wm058_receipts[0].outcome,TaxOutcome::Paid{..}));assert!(matches!(g.wm058_receipts[1].outcome,TaxOutcome::Countered{..}));assert_eq!(g.players[2].graveyard.iter().filter(|c|c.definition=="JC102").count(),2);restore(&mut g);
}

#[test]
fn batch4_chance_consumption_receipts_expire_in_real_end_cleanup(){
 let(mut g,_,target)=layout();cast(&mut g,"JC102",2,&target,0);accept(&mut g);top(&mut g);assert_eq!(g.wm058_receipts.len(),1);let turn=g.turn;g.begin_window(Window::End);for _ in 0..64{if g.turn!=turn{break;}pass(&mut g);}assert_eq!(g.turn,turn+1);assert!(g.wm058_receipts.is_empty());restore(&mut g);
}
