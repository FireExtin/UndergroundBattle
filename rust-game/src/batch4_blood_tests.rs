use crate::{batch4_blood as blood,catalog,model::*,rules::*,unit_support::*};
fn setup(n:usize)->(Game,String,Vec<String>,String){
 let mut g=game();fund(&mut g,0,"JZ36",1);let source=held(&mut g,"JZ36",0);
 let victims=(0..n).map(|_|field(&mut g,"JC125",0)).collect();let target=field(&mut g,"JC125",2);(g,source,victims,target)
}
fn carrier(g:&mut Game,source:&str,target:&str,cost:Option<Vec<String>>){act(g,0,Action{card_id:Some(source.into()),option:Some("carrier".into()),target_id:Some(target.into()),cost_selected:cost,..Action::new("play")});}
fn region(g:&mut Game,source:&str,r:usize,cost:Option<Vec<String>>){act(g,0,Action{card_id:Some(source.into()),option:Some("region".into()),region:Some(r),cost_selected:cost,..Action::new("play")});}
#[test]
fn batch4_blood_exact_print_and_closed_modes_and_cost_program(){
 let c=catalog::card("JZ36");assert_eq!((c.kind.as_str(),c.cost,c.magic.as_str(),c.supported),("spell",1,"鲜血",false));assert_eq!(c.subtypes,["仪式"]);
 let d=blood::definition();assert_eq!(d.abilities[0].modes.len(),2);assert_eq!(d.abilities[0].timing,Timing::Standard);
 let mut wrong=d.clone();wrong.abilities[0].costs.clear();assert!(crate::batch4::validate_definition("JZ36",&wrong).is_err());
 let mut a=d.abilities[0].clone();a.key="foreign".into();a.ops=vec![Op::ForEachLivingPlayer(vec![Op::JZ36TimeAndDestroyTargetRegion])];assert!(crate::rules::validate_ability("JC125",&a).is_err());
}
#[test]
fn batch4_blood_unpaid_draft_preserves_hand_assets_targets_and_four_seat_privacy(){
 let (mut g,source,victims,target)=setup(3);let version=g.version;carrier(&mut g,&source,&target,None);
 assert_eq!(g.version,version+1);assert_eq!(g.resources(0),1);assert!(g.stack.is_empty());assert!(g.players[0].hand.iter().any(|c|c.id==source));assert!(victims.iter().all(|id|g.board(id).is_some()));
 let p=g.pending.as_ref().unwrap();assert_eq!((p.choice.min,p.choice.max),(Some(0),Some(3)));assert_eq!(p.choice.kind,"extra-sacrifice");
 for seat in 1..4{let v=g.view(seat);assert!(v.pending_choice.is_none());let waiting=serde_json::to_string(&v.waiting_choice).unwrap();assert!(!waiting.contains("JZ36")&&!waiting.contains(&source));}
 restore(&mut g);
}
#[test]
fn batch4_blood_confirmed_x_three_pays_once_and_replays_exact_choice_to_stack_to_effect(){
 let (mut g,source,victims,target)=setup(3);carrier(&mut g,&source,&target,None);let version=g.version;let original_choice=g.pending.as_ref().unwrap().choice.id.clone();choose(&mut g,victims.clone());
 assert_eq!(g.version,version+1);assert_eq!(g.resources(0),0);assert!(victims.iter().all(|id|g.board(id).is_none()));assert_eq!(g.players[0].graveyard.len(),3);
 let f=g.stack.last().unwrap().frame.as_ref().unwrap();assert!(matches!(f.already_paid.as_slice(),[PaidCost::Assets(a),PaidCost::JZ36Sacrificed(v)] if a.len()==1&&v.len()==3));
 reject(&mut g,0,Action{choice_id:Some(original_choice),selected:Some(victims),..Action::new("choose")});
 top(&mut g);assert_eq!(g.board(&target).unwrap().1.time_markers,3);assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JZ36").count(),1);restore(&mut g);
}
#[test]
fn batch4_blood_explicit_zero_is_a_payment_not_a_repeated_draft(){
 let (mut g,source,victims,target)=setup(1);carrier(&mut g,&source,&target,Some(vec![]));assert!(g.pending.is_none());assert_eq!(g.resources(0),0);top(&mut g);
 assert!(g.board(&victims[0]).is_some());assert_eq!(g.board(&target).unwrap().1.time_markers,0);assert!(!g.regions[0].destroyed);restore(&mut g);
}
#[test]
fn batch4_blood_region_time_four_is_public_to_every_seat_and_destroys_in_place(){
 let (mut g,source,victims,_)=setup(4);let uid=g.regions[1].card.id.clone();region(&mut g,&source,1,None);choose(&mut g,victims);top(&mut g);
 assert_eq!(g.regions[1].card.id,uid);assert_eq!(g.regions[1].card.time_markers,4);assert!(g.regions[1].destroyed);
 for seat in 0..4{let v=g.view(seat);assert_eq!(v.regions[1].time_markers,4);assert!(v.regions[1].destroyed);}restore(&mut g);
}
#[test]
fn batch4_blood_invalid_or_duplicate_cost_and_extra_choice_payload_do_not_pay(){
 let (mut g,source,victims,target)=setup(2);carrier(&mut g,&source,&target,None);let p=g.pending.clone().unwrap();
 for a in [Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![victims[0].clone(),victims[0].clone()]),..Action::new("choose")},Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![target]),..Action::new("choose")},Action{choice_id:Some(p.choice.id),selected:Some(vec![victims[0].clone()]),option:Some("region".into()),..Action::new("choose")}] {reject(&mut g,0,a);}
 assert_eq!(g.resources(0),1);assert!(g.players[0].hand.iter().any(|c|c.id==source));choose(&mut g,vec![victims[0].clone()]);top(&mut g);restore(&mut g);
}
#[test]
fn batch4_blood_host_and_attached_bat_are_one_frozen_two_role_payment(){
 let (mut g,source,_,target)=setup(0);let host=field(&mut g,"JC029",0);let bat=g.make_card("BQ026",0);let bat_id=bat.id.clone();g.attachments.push(Attachment{card:bat,host_id:host.clone()});restore(&mut g);
 carrier(&mut g,&source,&target,None);choose(&mut g,vec![host.clone(),bat_id.clone()]);
 assert!(g.board(&host).is_none()&&g.board(&bat_id).is_none());assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JC029").count(),1);assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="BQ026").count(),1);
 top(&mut g);assert_eq!(g.board(&target).unwrap().1.time_markers,2);restore(&mut g);
}
#[test]
fn batch4_blood_a_paid_victim_target_invalidates_the_whole_effect_without_returning_cost(){
 let (mut g,source,victims,_)=setup(2);g.board_mut(&victims[1]).unwrap().time_markers=4;
 carrier(&mut g,&source,&victims[0],None);choose(&mut g,vec![victims[0].clone()]);top(&mut g);
 assert_eq!(g.resources(0),0);assert!(g.board(&victims[0]).is_none());assert!(!g.regions[0].destroyed);assert_eq!(g.board(&victims[1]).unwrap().1.time_markers,4);restore(&mut g);
}
#[test]
fn batch4_blood_captured_region_clears_time_and_replacement_or_empty_slot_never_inherits_it(){
 for empty in [false,true] {let (mut g,source,victims,_)=setup(2);region(&mut g,&source,1,Some(victims));top(&mut g);let old=g.regions[1].card.id.clone();if empty{g.world.clear();}
   g.effect(Effect::Score{seat:0,region:1}).unwrap();assert!(g.players[0].score_cards.iter().all(|c|c.time_markers==0));assert_eq!(g.regions[1].card.time_markers,0);assert_eq!(g.regions[1].vacant,empty);if !empty{assert_ne!(g.regions[1].card.id,old);}restore(&mut g);
 }
}
#[test]
fn batch4_blood_original_region_target_uid_cannot_rebind_after_response_replacement(){
 let (mut g,source,victims,_)=setup(1);region(&mut g,&source,1,Some(victims));
 // Explicit response-layout fixture, not a claimed opponent command chain.
 g.regions[1].card=g.make_card("DQJC114",0);restore(&mut g);top(&mut g);assert_eq!(g.regions[1].card.time_markers,0);assert!(!g.regions[1].destroyed);restore(&mut g);
}
#[test]
fn batch4_blood_carrier_follows_current_region_while_x_remains_the_paid_set_size(){
 let (mut g,source,victims,target)=setup(2);carrier(&mut g,&source,&target,Some(victims));
 let p=g.regions[0].cards.iter().position(|c|c.id==target).unwrap();let mut c=g.regions[0].cards.remove(p);c.time_markers=2;g.regions[1].cards.push(c);
 restore(&mut g);top(&mut g);assert_eq!(g.board(&target).unwrap().1.time_markers,4);assert!(g.regions[1].destroyed);assert!(!g.regions[0].destroyed);restore(&mut g);
}
#[test]
fn batch4_blood_pending_and_paid_receipt_mutations_reject_restore(){
 let (mut g,source,victims,target)=setup(2);carrier(&mut g,&source,&target,None);
 for n in 0..5 {let mut bad=g.clone();let p=bad.pending.as_mut().unwrap();if let ChoiceResolution::JZ36Payment{draft}=&mut p.resolution {match n {0=>draft.turn+=1,1=>draft.action.option=Some("region".into()),2=>draft.action.cost_selected=Some(vec![]),3=>p.choice.max=Some(99),_=>p.seat=1};}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"pending {n}");}
 choose(&mut g,victims);
 for n in 0..5 {let mut bad=g.clone();let f=bad.stack.last_mut().unwrap().frame.as_mut().unwrap();match n {0=>f.already_paid.pop().map(|_|()).unwrap(),1=>{if let PaidCost::JZ36Sacrificed(v)=&mut f.already_paid[1]{v.push(v[0].clone());}},2=>f.steps[0].op=Op::JZ40TimeAndDestroyHost,3=>f.cursor=1,_=>f.targets[0].spec.predicate=None};assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"paid {n}");}
 top(&mut g);restore(&mut g);
}
#[test]
fn batch4_blood_original_two_command_replay_preserves_large_u64_and_every_view(){
 let (mut g,source,victims,target)=setup(2);g.seed=u64::MAX;g.random=9007199254740993;let initial=serde_json::to_string(&g).unwrap();
 let play=Action{card_id:Some(source),option:Some("carrier".into()),target_id:Some(target),..Action::new("play")};act(&mut g,0,play.clone());
 let choose_action=Action{choice_id:Some(g.pending.as_ref().unwrap().choice.id.clone()),selected:Some(victims),..Action::new("choose")};act(&mut g,0,choose_action.clone());
 let mut replay=Game::from_persisted(&initial).unwrap();for a in [play,choose_action]{replay.apply(0,a).unwrap();}assert_eq!(serde_json::to_string(&replay).unwrap(),serde_json::to_string(&g).unwrap());assert_eq!((replay.seed,replay.random),(u64::MAX,9007199254740993));for s in 0..4{assert_eq!(serde_json::to_value(replay.view(s)).unwrap(),serde_json::to_value(g.view(s)).unwrap());}
 top(&mut g);restore(&mut g);
}
#[test]
fn batch4_blood_zero_victim_draft_still_confirms_and_unaffordable_draft_cannot_start(){
 let (mut g,source,_,target)=setup(0);carrier(&mut g,&source,&target,None);assert_eq!(g.pending.as_ref().unwrap().choice.max,Some(0));choose(&mut g,vec![]);top(&mut g);assert_eq!(g.resources(0),0);assert_eq!(g.board(&target).unwrap().1.time_markers,0);restore(&mut g);
 let (mut g,source,_,target)=setup(1);g.players[0].assets[0].exhausted=true;
 reject(&mut g,0,Action{card_id:Some(source),option:Some("carrier".into()),target_id:Some(target),..Action::new("play")});assert!(g.pending.is_none());
}
#[test]
fn batch4_blood_actual_paid_death_forecast_resumes_without_reusing_the_cost(){
 let (mut g,source,_,target)=setup(0);let victim=field(&mut g,"JZ30",0);g.board_mut(&victim).unwrap().time_markers=2;
 carrier(&mut g,&source,&target,None);choose(&mut g,vec![victim.clone()]);let mut saw_forecast=false;
 for _ in 0..128 {if let Some(p)=g.pending.clone(){
   if matches!(&p.resolution,ChoiceResolution::Frame{choice:FrameChoice::Forecast{..},..}){
     saw_forecast=true;assert_eq!(p.choice.amount,Some(2));assert_eq!(g.board(&target).unwrap().1.time_markers,0);
     assert!(g.stack.iter().any(|s|s.frame.as_ref().is_some_and(|f|f.source.card.definition=="JZ36")));restore(&mut g);resolve_choice(&mut g);
   }
   else if p.choice.kind=="trigger"{choose(&mut g,vec![p.choice.options[0].id.clone()]);}else{resolve_choice(&mut g);}
 }else if !g.effects.is_empty(){g.drive().unwrap();restore(&mut g);}else if !g.stack.is_empty(){pass(&mut g);}else{break;}}
 assert!(saw_forecast,"must execute the actual JZ30 death forecast, not decline its trigger");
 assert_eq!(g.resources(0),0);assert_eq!(g.board(&target).unwrap().1.time_markers,1);assert!(g.board(&victim).is_none());
 assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JZ30").count(),1);assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JZ36").count(),1);restore(&mut g);
}
#[test]
fn batch4_blood_second_team_actor_pays_only_personal_characters_and_keeps_their_private_choice(){
 let mut g=game();fund(&mut g,2,"JZ36",1);let source=held(&mut g,"JZ36",2);let own=field(&mut g,"JC125",2);let ally=field(&mut g,"JC125",3);let target=field(&mut g,"JC125",0);g.begin_window(Window::Action(1));
 act(&mut g,2,Action{card_id:Some(source),option:Some("carrier".into()),target_id:Some(target.clone()),..Action::new("play")});let p=g.pending.as_ref().unwrap();assert_eq!(p.seat,2);assert!(p.choice.options.iter().any(|o|o.id==own));assert!(!p.choice.options.iter().any(|o|o.id==ally));
 let wrong=Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![own.clone()]),..Action::new("choose")};reject(&mut g,3,wrong);choose(&mut g,vec![own]);top(&mut g);assert_eq!(g.resources(2),0);assert!(g.board(&ally).is_some());assert_eq!(g.board(&target).unwrap().1.time_markers,1);restore(&mut g);
}
