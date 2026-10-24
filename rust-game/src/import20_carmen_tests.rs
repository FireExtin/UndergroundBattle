//! Normal paid CQ24 commands on legal explicit layouts, not browser or an
//! independent review. All shared helpers preserve normal Game/four-seat views.
use crate::{catalog, import20_carmen as c, model::*, rules::*, unit_support::*};
use crate::jc029_tests::pass_top;
fn att(g:&mut Game,id:&str,actor:usize,host:&str)->String{let card=g.make_card(id,actor);let uid=card.id.clone();g.attachments.push(Attachment{card,host_id:host.into()});uid}
fn ordinary(g:&mut Game)->(String,String){let host=field(g,"JC125",2);let a=att(g,"WM038",2,&host);(a,host)}
fn enter(g:&mut Game,actor:usize,extra:bool)->String{
    g.begin_window(Window::Action(g.team(actor)));fund(g,actor,"JC084",if extra{4}else{2});let hand=held(g,c::ID,actor);
    let region=(0..g.regions.len()).find(|r|g.accessible(actor,*r)).unwrap();
    act(g,actor,Action{card_id:Some(hand),region:Some(region),option:extra.then(||"pay-extra-3".into()),..Action::new("deploy")});pass_top(g);
    g.regions[region].cards.iter().rev().find(|c|c.definition==c::ID&&c.controller==actor).unwrap().id.clone()
}
fn bind(g:&mut Game,target:&str,host:Option<&str>){
    assert!(g.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Declare{declaration:d,..}if d.ability.key==c::ENTRY)));
    choose(g,vec![target.into()]);
    if g.pending.as_ref().is_some_and(|p|matches!(p.resolution,ChoiceResolution::CQ24Reattach{..})){choose(g,host.map(|s|vec![s.into()]).unwrap_or_default());}else{assert!(host.is_none());}
    assert!(g.stack.last().is_some_and(|s|s.frame.as_ref().is_some_and(|f|f.ability_key==c::ENTRY)));
}
fn priority(g:&mut Game,s:usize){for _ in 0..8{if g.can_fast(s){return;}pass(g);}panic!("missing priority");}
fn destroy_character(g:&mut Game,id:&str){fund(g,2,"JC084",2);let h=held(g,"BQ090",2);priority(g,2);act(g,2,Action{card_id:Some(h),target_id:Some(id.into()),..Action::new("play")});pass_top(g);assert!(g.board(id).is_none());}
fn destroy_attachment(g:&mut Game,id:&str){fund(g,2,"JC005",2);let h=held(g,"JC005",2);priority(g,2);act(g,2,Action{card_id:Some(h),target_id:Some(id.into()),..Action::new("play")});pass_top(g);assert!(!g.attachments.iter().any(|a|a.card.id==id));}
#[test]
fn import20_cq24_original_cost3_limit_conditional_retreat_and_nonasset(){
    let d=catalog::card(c::ID);assert_eq!((d.cost,d.color.as_str(),d.subtitle.as_deref(),d.defense),(1,"黑",Some("神偷"),Some(1)));assert_eq!(d.loyalty,["黑色","黑色"]);assert!(d.unique&&!d.supported);
    assert_eq!(d.temporary_icons.influence,2);assert!(d.text.contains("等于③"));assert!(!d.text.contains("等于⑤"));assert!(!definition(c::ID).traits.retreat);
    let mut g=game();let id=field(&mut g,c::ID,0);assert!(!g.has_retreat(g.board(&id).unwrap().1));g.players[1].society_zone.secrecy=1;assert!(!g.has_retreat(g.board(&id).unwrap().1));g.players[0].society_zone.secrecy=1;assert!(g.has_retreat(g.board(&id).unwrap().1));
    for s in 0..4{assert_eq!(g.card_view(g.board(&id).unwrap().1,s,Some(0),None).current_retreat,Some(true));}
    let h=held(&mut g,c::ID,0);reject(&mut g,0,Action{card_id:Some(h),..Action::new("asset")});
}
#[test]
fn import20_cq24_actual_extra3_pays4_before_entry_and_takes_original_attachment_keep_host(){
    let mut g=game();let (id,host)=ordinary(&mut g);g.board_mut(&id).unwrap().exhausted=true;g.board_mut(&id).unwrap().time_markers=2;
    let owner=g.attachments[0].card.owner;enter(&mut g,0,true);assert_eq!(g.resources(0),0);bind(&mut g,&id,None);pass_top(&mut g);
    let a=g.attachments.iter().find(|a|a.card.id==id).unwrap();assert_eq!(a.card.controller,0);assert_eq!(a.card.owner,owner);assert_eq!(a.host_id,host);assert!(a.card.exhausted);assert_eq!(a.card.time_markers,2);assert_eq!(g.control_effects.len(),1);restore(&mut g);
}
#[test]
fn import20_cq24_extra_cost_insufficient_loyalty_unknown_option_and_payload_rollback(){
    let mut g=game();ordinary(&mut g);fund(&mut g,0,"JC084",3);let hand=held(&mut g,c::ID,0);let a=Action{card_id:Some(hand.clone()),region:Some(0),option:Some("pay-extra-3".into()),..Action::new("deploy")};reject(&mut g,0,a.clone());assert_eq!(g.resources(0),3);
    fund(&mut g,0,"JC125",1);reject(&mut g,0,Action{option:Some("pay-extra-1".into()),..a.clone()});reject(&mut g,0,Action{target_id:Some("extra".into()),..a});
    let mut g=game();fund(&mut g,0,"JC084",1);fund(&mut g,0,"JC125",3);let h=held(&mut g,c::ID,0);reject(&mut g,0,Action{card_id:Some(h),region:Some(0),option:Some("pay-extra-3".into()),..Action::new("deploy")});assert_eq!(g.resources(0),4);
}
#[test]
fn import20_cq24_unpaid_personal_secrecy_must_exceed_each_enemy_not_team_sum_or_ally(){
    for (own,ally,enemies,yes) in [(2,0,[1,1],true),(1,3,[0,1],false),(2,0,[1,2],false),(0,3,[0,0],false),(1,0,[0,0],true)]{
        let mut g=game();let (id,_)=ordinary(&mut g);g.players[0].society_zone.secrecy=own;g.players[1].society_zone.secrecy=ally;g.players[2].society_zone.secrecy=enemies[0];g.players[3].society_zone.secrecy=enemies[1];
        enter(&mut g,0,false);assert_eq!(g.pending.is_some(),yes);assert_eq!(g.resources(0),1);
        if yes{bind(&mut g,&id,None);pass_top(&mut g);assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().card.controller,0);}
    }
}
#[test]
fn import20_cq24_extra_payment_qualifies_with_lower_secrecy_and_may_decline_no_refund(){
    let mut g=game();let (id,_)=ordinary(&mut g);g.players[2].society_zone.secrecy=3;enter(&mut g,0,true);choose(&mut g,vec![]);assert_eq!(g.resources(0),0);assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().card.controller,2);assert!(g.control_effects.is_empty());
}
#[test]
fn import20_cq24_both_targets_bound_before_response_and_control_has_no_early_mutation(){
    let mut g=game();let (id,old)=ordinary(&mut g);let new=field(&mut g,"JC125",0);enter(&mut g,0,true);choose(&mut g,vec![id.clone()]);
    assert!(g.stack.is_empty());assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().card.controller,2);assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().host_id,old);
    choose(&mut g,vec![new.clone()]);let frame=g.stack.last().unwrap().frame.as_ref().unwrap();assert_eq!(frame.targets.len(),2);assert_eq!(frame.targets[0].id,id);assert_eq!(frame.targets[1].id,new);assert!(g.pending.is_none());pass_top(&mut g);
    let a=g.attachments.iter().find(|a|a.card.id==id).unwrap();assert_eq!(a.card.controller,0);assert_eq!(a.host_id,new);restore(&mut g);
}
#[test]
fn import20_cq24_source_leaves_response_but_original_attachment_transfer_still_resolves(){
    let mut g=game();let (id,_)=ordinary(&mut g);let source=enter(&mut g,0,true);bind(&mut g,&id,None);destroy_character(&mut g,&source);pass_top(&mut g);assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().card.controller,0);
}
#[test]
fn import20_cq24_original_attachment_or_new_host_leaves_whole_effect_cancels(){
    let mut g=game();let (id,old)=ordinary(&mut g);let new=field(&mut g,"JC125",0);enter(&mut g,0,true);bind(&mut g,&id,Some(&new));destroy_attachment(&mut g,&id);
    let replacement=att(&mut g,"WM038",2,&old);pass_top(&mut g);assert_eq!(g.attachments.iter().find(|a|a.card.id==replacement).unwrap().card.controller,2);assert!(g.control_effects.is_empty());
    let mut g=game();let (id,old)=ordinary(&mut g);let new=field(&mut g,"JC125",0);enter(&mut g,0,true);bind(&mut g,&id,Some(&new));destroy_character(&mut g,&new);pass_top(&mut g);
    let a=g.attachments.iter().find(|a|a.card.id==id).unwrap();assert_eq!(a.card.controller,2);assert_eq!(a.host_id,old);assert_eq!(g.resources(0),0);assert!(g.control_effects.is_empty());
}
#[test]
fn import20_cq24_original_attachment_shield_cancels_before_control_or_new_host(){
    let mut g=game();let (id,host)=ordinary(&mut g);let new=field(&mut g,"JC125",0);enter(&mut g,0,true);bind(&mut g,&id,Some(&new));g.board_mut(&id).unwrap().shield=1;restore(&mut g);pass_top(&mut g);
    let a=g.attachments.iter().find(|a|a.card.id==id).unwrap();assert_eq!((a.card.controller,a.card.shield),(2,0));assert_eq!(a.host_id,host);assert!(g.control_effects.is_empty());
}
#[test]
fn import20_cq24_cost_limit_is_printed3_excludes5_even_discounted_and_own_or_ally(){
    let mut g=game();let host=field(&mut g,"JC125",2);let five=att(&mut g,"JC036",2,&host);g.add_control(&host,2,ControlLifetime::Attached{source_instance:five.clone()},SubtypeChange::HumanToVampire);
    let own_host=field(&mut g,"JC125",0);let own=att(&mut g,"WM038",0,&own_host);let ally_host=field(&mut g,"JC125",1);let ally=att(&mut g,"WM038",1,&ally_host);let valid=att(&mut g,"BQ022",2,&host);
    enter(&mut g,0,true);let options=&g.pending.as_ref().unwrap().choice.options;assert!(options.iter().any(|o|o.id==valid));for id in [&five,&own,&ally]{assert!(!options.iter().any(|o|&o.id==id));}
    let choice_id=g.pending.as_ref().unwrap().choice.id.clone();reject(&mut g,0,Action{choice_id:Some(choice_id),selected:Some(vec![five]),..Action::new("choose")});
}
#[test]
fn import20_cq24_personal_society_attachment_transfers_and_reattaches_atomically(){
    let mut g=game();let id=att(&mut g,"JC033",2,"society:p2");enter(&mut g,0,true);bind(&mut g,&id,Some("society:p0"));pass_top(&mut g);
    let a=g.attachments.iter().find(|a|a.card.id==id).unwrap();assert_eq!(a.card.controller,0);assert_eq!(a.card.owner,2);assert_eq!(a.host_id,"society:p0");assert!(g.attachment_host_valid(a));restore(&mut g);
}
#[test]
fn import20_cq24_region_attachment_uses_exact_bound_new_region_instance(){
    let mut g=game();let old=g.regions[0].card.id.clone();let id=att(&mut g,"JC048",2,&old);let new=g.regions[1].card.id.clone();enter(&mut g,0,true);bind(&mut g,&id,Some("region:1"));
    assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().targets[1].region_instance.as_deref(),Some(new.as_str()));pass_top(&mut g);
    let a=g.attachments.iter().find(|a|a.card.id==id).unwrap();assert_eq!(a.host_id,new);assert_eq!(a.card.controller,0);restore(&mut g);
}
#[test]
fn import20_cq24_standalone_and_attached_bats_keep_dual_role_original_uid(){
    for attached in [false,true]{let mut g=game();let enemy=field(&mut g,"JC029",2);let own=field(&mut g,"JC029",0);let id=if attached{att(&mut g,"BQ026",2,&enemy)}else{field(&mut g,"BQ026",2)};
        g.board_mut(&id).unwrap().time_markers=2;enter(&mut g,0,true);bind(&mut g,&id,Some(&own));pass_top(&mut g);
        let a=g.attachments.iter().find(|a|a.card.id==id).unwrap();assert_eq!((a.card.controller,a.card.owner,a.card.time_markers),(0,2,2));assert_eq!(a.host_id,own);assert!(g.attached_character(&id)&&g.live_attachment_role(&a.card));restore(&mut g);
    }
}
#[test]
fn import20_cq24_secret_plan_not_disclosed_before_transfer_and_new_controller_sees_own_plan(){
    let mut g=game();g.begin_window(Window::Action(1));fund(&mut g,2,"WM046",3);g.players[2].society_zone.secrecy=1;let h=held(&mut g,"WM046",2);
    act(&mut g,2,Action{card_id:Some(h),target_id:Some("society:p2".into()),option:Some("secret-plan".into()),..Action::new("play")});pass_top(&mut g);let id=g.attachments[0].card.id.clone();
    enter(&mut g,0,true);let option=g.pending.as_ref().unwrap().choice.options.iter().find(|o|o.id==id).unwrap();assert!(option.card.as_ref().is_none_or(|c|c.card_id.is_none()));
    choose(&mut g,vec![id.clone()]);let p=g.pending.as_ref().unwrap();assert_eq!(p.choice.options.len(),1);assert_eq!(p.choice.options[0].id,"society:p0");
    for seat in [0,1,3]{let v=g.view(seat);let a=v.attachments.iter().find(|a|a.card.instance_id==id).unwrap();assert!(a.card.card_id.is_none());}
    choose(&mut g,vec!["society:p0".into()]);pass_top(&mut g);
    let a=g.attachments.iter().find(|a|a.card.id==id).unwrap();assert_eq!((a.card.controller,a.card.owner),(0,2));assert!(a.card.face_down);assert_eq!(a.host_id,"society:p0");
    assert_eq!(g.view(0).attachments.iter().find(|a|a.card.instance_id==id).unwrap().card.card_id.as_deref(),Some("WM046"));
    assert!(g.view(2).attachments.iter().find(|a|a.card.instance_id==id).unwrap().card.card_id.is_none());restore(&mut g);
}
#[test]
fn import20_cq24_permanent_control_does_not_expire_with_source_or_turn_and_last_take_wins(){
    let mut g=game();let (id,_)=ordinary(&mut g);let source=enter(&mut g,0,true);bind(&mut g,&id,None);pass_top(&mut g);
    g.remove_dead(&source,RemovalCause::Destroy);g.turn+=1;g.settle_deaths();assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().card.controller,0);restore(&mut g);
    enter(&mut g,2,true);bind(&mut g,&id,None);pass_top(&mut g);assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().card.controller,2);assert_eq!(g.control_effects.len(),2);restore(&mut g);
}
#[test]
fn import20_cq24_first_extra_source_and_entry_condition_survive_normal_response_changes(){
    let mut g=game();let (id,_)=ordinary(&mut g);g.players[0].society_zone.secrecy=1;enter(&mut g,0,false);bind(&mut g,&id,None);g.add_society_markers(0,1,0);g.add_society_markers(2,0,3);restore(&mut g);pass_top(&mut g);
    assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().card.controller,0);
}
#[test]
fn import20_cq24_room_host_choice_is_before_response_and_stale_repeat_cannot_take_twice(){
    use crate::room::{RoomCommand,RoomEnvelope};
    let mut g=game();let (id,_)=ordinary(&mut g);let host=field(&mut g,"JC125",0);enter(&mut g,0,true);choose(&mut g,vec![id.clone()]);let p=g.pending.as_ref().unwrap();
    let r=crate::jc029_tests::envelope(&g);let command=RoomCommand{command_id:"cq24-original-host-choice".into(),expected_version:r.revision,action:Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![host]),..Action::new("choose")}.into()};
    let out=r.transition(0,Some(command.clone()),2000).unwrap();assert!(out.error_code.is_none(),"{:?}",out.error_message);let next=RoomEnvelope::from_persisted(&out.state).unwrap();assert!(next.game.pending.is_none());assert_eq!(next.game.stack.last().unwrap().frame.as_ref().unwrap().targets.len(),2);
    assert_eq!(serde_json::to_value(r.replay_events(&out.journal).unwrap()).unwrap(),serde_json::to_value(&next).unwrap());let repeat=next.transition(0,Some(command),2001).unwrap();assert_eq!(repeat.error_code.as_deref(),Some("version_conflict"));assert_eq!(repeat.state,out.state);assert_eq!(next.game.attachments.iter().find(|a|a.card.id==id).unwrap().card.controller,2);
}
#[test]
fn import20_cq24_closed_programs_reject_transplants_cost5_limit_and_unpaid_extra_ops(){
    let mut all=definitions().clone();all.get_mut(c::ID).unwrap().abilities[0].targets[0].printed_cost_max=Some(5);assert!(validate_definitions(&all).is_err());
    let mut all=definitions().clone();all.get_mut("JC125").unwrap().abilities.push(c::definition().abilities[0].clone());assert!(validate_definitions(&all).is_err());
    let mut all=definitions().clone();all.get_mut(c::ID).unwrap().abilities[0].ops.clear();assert!(validate_definitions(&all).is_err());
    validate_definitions(definitions()).unwrap();
}
