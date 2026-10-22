//! Source/asset layouts are disclosed Native fixtures. Payment, play, response,
//! ordered triggers and persistence use production commands; not browser QA.
use crate::{catalog, model::*, rules::*, unit_support::*};
use crate::batch4_omega::KEY;
use crate::jc029_tests::pass_top;

fn asset(g:&mut Game, id:&str, seat:usize, exhausted:bool)->String {
    let mut c=g.make_card(id,seat);c.exhausted=exhausted;let id=c.id.clone();g.players[seat].assets.push(c);id
}
fn setup(exhausted:bool)->(Game,String,String) {
    let mut g=game();let source=field(&mut g,"XG18",0);let c=asset(&mut g,"JC067",0,exhausted);fund(&mut g,0,"XG18",6);restore(&mut g);(g,source,c)
}
fn deploy(g:&mut Game,id:&str){act(g,0,Action{card_id:Some(id.into()),region:Some(0),..Action::new("deploy")});}
fn played(g:&Game)->&ResolutionFrame {g.stack.iter().find_map(|i|i.frame.as_ref().filter(|f|f.ability_key=="deploy")).unwrap()}
fn finish(g:&mut Game){for _ in 0..64{if g.pending.is_some(){crate::unit_support::resolve_choice(g);}else if !g.stack.is_empty(){top(g);}else{return;}}panic!("bounded Omega resolution");}
fn invalid(g:&Game,v:serde_json::Value){let s=serde_json::to_string(&v).unwrap();let r=std::panic::catch_unwind(||Game::from_persisted(&s));assert!(r.is_ok()&&r.unwrap().is_err());}

#[test]
fn batch4_omega_exact_source_and_fixed_full_definition(){
    let c=catalog::card("XG18");assert_eq!((c.name.as_str(),c.cost,c.defense,c.unique,c.supported),("欧米伽小姐",6,Some(2),true,false));
    assert_eq!(c.subtitle.as_deref(),Some("P.S.C.管理员"));assert_eq!(c.loyalty,["灰色","灰色","灰色"]);assert_eq!(c.subtypes,["人类","工程师","互联网"]);
    assert_eq!(c.permanent_icons,Icons{investigation:2,combat:0,influence:2});let d=definition("XG18");assert!(d.traits.public);assert_eq!(d.abilities.len(),2);
    for change in 0..4{let mut defs=crate::rules::definitions().clone();let d=defs.get_mut("XG18").unwrap();match change{0=>d.traits.public=false,1=>d.abilities[0].costs.push(Cost::Assets(1)),2=>d.abilities[0].event=Some(Event::Enter),_=>d.modifiers.clear()};assert!(crate::rules::validate_definitions(&defs).is_err());}
}
#[test]
fn batch4_omega_asset_can_pay_for_its_own_normal_play_then_reward_is_forced(){
    let(mut g,_,id)=setup(false);assert!(g.has_renown(g.players[0].assets.iter().find(|c|c.id==id).unwrap()));let before=g.resources(0);deploy(&mut g,&id);
    let f=played(&g);let Some(PlaySource::AssetByXG18(grant))=&f.source.play_source else{panic!()};assert_eq!(grant.original_asset.id,id);assert!(!grant.original_asset.exhausted);
    assert!(matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.contains(&id)&&ids.len()==6));assert!(!g.players[0].assets.iter().any(|c|c.id==id));assert_eq!(g.resources(0),before-6);
    assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().ability_key,KEY);assert!(g.pending.is_none());finish(&mut g);
    assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC067"&&c.id!=id));assert!(g.players.iter().all(|p|p.society_zone.exposure==1));restore(&mut g);
}
#[test]
fn batch4_omega_exhausted_asset_uses_other_assets_and_enters_ready(){
    let(mut g,_,id)=setup(true);deploy(&mut g,&id);let f=played(&g);
    assert!(matches!(f.source.play_source.as_ref(),Some(PlaySource::AssetByXG18(p)) if p.original_asset.exhausted));
    assert!(!f.source.card.exhausted);assert!(matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if !ids.contains(&id)&&ids.len()==6));finish(&mut g);
    assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC067"&&!c.exhausted));restore(&mut g);
}
#[test]
fn batch4_omega_permission_requires_personal_live_source_and_real_asset_zone(){
    for holder in [1,2]{let mut g=game();field(&mut g,"XG18",holder);let id=asset(&mut g,"JC067",0,false);fund(&mut g,0,"XG18",6);reject(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});}
    let(mut g,source,id)=setup(false);g.board_mut(&source).unwrap().face_down=true;reject(&mut g,0,Action{card_id:Some(id.clone()),region:Some(0),..Action::new("deploy")});
    g.board_mut(&source).unwrap().face_down=false;let n=g.players[0].assets.iter().position(|c|c.id==id).unwrap();let c=g.players[0].assets.remove(n);g.players[0].graveyard.push(c);reject(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});
}
#[test]
fn batch4_omega_no_conceal_asset_reuse_or_wrong_owner_permission(){
    let(mut g,_,id)=setup(false);for kind in ["conceal","asset"]{reject(&mut g,0,Action{card_id:Some(id.clone()),region:(kind=="conceal").then_some(0),..Action::new(kind)});}
    reject(&mut g,2,Action{card_id:Some(id),region:Some(2),..Action::new("deploy")});
    let not_renown=asset(&mut g,"JC125",0,false);reject(&mut g,0,Action{card_id:Some(not_renown),region:Some(0),..Action::new("deploy")});
}
#[test]
fn batch4_omega_normal_cost_loyalty_and_hand_only_jz52_restriction_remain(){
    let(mut g,_,id)=setup(true);g.players[0].assets.retain(|c|c.id==id);reject(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});
    let(mut g,_,id)=setup(false);g.players[0].assets.retain(|c|c.id==id);fund(&mut g,0,"JC125",6);reject(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});
    let(mut g,_,id)=setup(false);field(&mut g,"JZ52",0);let hand=held(&mut g,"JC067",0);reject(&mut g,0,Action{card_id:Some(hand),region:Some(0),..Action::new("deploy")});deploy(&mut g,&id);finish(&mut g);
}
#[test]
fn batch4_omega_hand_play_rewards_but_direct_entry_and_paid_reveal_do_not(){
    let mut g=game();field(&mut g,"XG18",0);fund(&mut g,0,"XG18",6);let id=held(&mut g,"JC067",0);deploy(&mut g,&id);assert!(matches!(played(&g).source.play_source,Some(PlaySource::Hand)));finish(&mut g);assert!(g.players.iter().all(|p|p.society_zone.exposure==1));
    let mut g=game();field(&mut g,"XG18",0);let id=field(&mut g,"JC067",0);g.enter_triggers(0,"JC067",&id,false);g.drive().unwrap();finish(&mut g);assert!(g.players.iter().all(|p|p.society_zone.exposure==0));
    let mut g=game();field(&mut g,"XG18",0);fund(&mut g,0,"XG18",6);let id=field(&mut g,"JC067",0);g.board_mut(&id).unwrap().face_down=true;act(&mut g,0,Action{card_id:Some(id),..Action::new("reveal")});finish(&mut g);assert!(g.players.iter().all(|p|p.society_zone.exposure==0));
}
#[test]
fn batch4_omega_conditional_renown_is_frozen_from_personal_sun_at_actual_play(){
    let mut g=game();field(&mut g,"XG18",0);let id=asset(&mut g,"XG17",0,false);fund(&mut g,0,"XG18",6);g.players[1].society_zone.exposure=1;
    reject(&mut g,0,Action{card_id:Some(id.clone()),region:Some(0),..Action::new("deploy")});g.players[0].society_zone.exposure=1;deploy(&mut g,&id);
    assert!(matches!(&played(&g).source.play_source,Some(PlaySource::AssetByXG18(p)) if matches!(p.qualification,crate::batch4_omega::PlayedRenown::Exposure{count:1})));finish(&mut g);assert_eq!(g.players[0].society_zone.exposure,2);restore(&mut g);
}
#[test]
fn batch4_omega_reward_uses_normal_sun_moon_cancellation_and_living_societies(){
    let(mut g,_,id)=setup(false);g.players[2].society_zone.secrecy=2;deploy(&mut g,&id);pass_top(&mut g);
    assert_eq!((g.players[2].society_zone.exposure,g.players[2].society_zone.secrecy),(0,1));assert_eq!(g.players[0].society_zone.exposure,1);finish(&mut g);restore(&mut g);
}
#[test]
fn batch4_omega_source_departure_after_commit_keeps_permission_and_already_triggered_reward(){
    let(mut g,source,id)=setup(false);deploy(&mut g,&id);g.remove_dead(&source,RemovalCause::Destroy);restore(&mut g);finish(&mut g);
    assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC067"));assert!(g.players.iter().all(|p|p.society_zone.exposure==1));restore(&mut g);
}
#[test]
fn batch4_omega_conditional_mobility_requires_personal_sun_and_frozen_observation(){
    let(mut g,source,_)=setup(false);assert!(!g.has_mobility(g.board(&source).unwrap().1));g.players[1].society_zone.exposure=1;assert!(!g.has_mobility(g.board(&source).unwrap().1));
    g.players[0].society_zone.exposure=1;let c=g.board(&source).unwrap().1;assert!(g.has_mobility(c));let snapshot=g.mobility_source(c,0);g.begin_window(Window::Mobility);assert_eq!(snapshot.observed_exposure,Some(1));g.validate_mobility_source(0,&snapshot).unwrap();
    g.players[0].society_zone.exposure=0;g.validate_mobility_source(0,&snapshot).unwrap();assert!(!g.has_mobility(g.board(&source).unwrap().1));restore(&mut g);
}
#[test]
fn batch4_omega_grant_and_observer_mutations_reject_without_panics(){
    let(mut g,_,id)=setup(false);deploy(&mut g,&id);let base=serde_json::to_value(&g).unwrap();
    for change in 0..6{let mut v=base.clone();let grant=&mut v["stack"][0]["frame"]["source"]["play_source"]["AssetByXG18"];match change{
        0=>grant["source"]["definition"]=serde_json::json!("XG16"),1=>grant["source"]["owner"]=serde_json::json!(99),2=>grant["turn"]=serde_json::json!(0),
        3=>grant["qualification"]=serde_json::json!({"Exposure":{"count":0}}),4=>grant["original_asset"]["definition"]=serde_json::json!("JC125"),_=>grant["original_asset"]["exhausted"]=serde_json::json!(true),
    }invalid(&g,v);}
}

#[test]
fn batch4_omega_all_current_printed_and_sun_renown_origins_roundtrip_paid_assets(){
    for id in ["JC070","JC076","XQ48","JC095","JC067","WM041","XG17"]{
        let mut g=game();field(&mut g,"XG18",0);let card=catalog::card(id);let source=asset(&mut g,id,0,false);
        let funding=catalog::catalog().cards.iter().find(|d|d.kind=="character"&&d.color==card.color&&!definition(&d.id).traits.renown&&!matches!(d.id.as_str(),"WM041"|"XG17")).unwrap().id.clone();
        fund(&mut g,0,&funding,card.cost.max(3) as usize);if matches!(id,"WM041"|"XG17"){g.players[0].society_zone.exposure=1;}
        deploy(&mut g,&source);assert!(played(&g).source.valid_character_play(),"{id}");finish(&mut g);
        assert!(g.regions[0].cards.iter().any(|c|c.definition==id&&!c.exhausted),"{id}");restore(&mut g);
    }
}
#[test]
fn batch4_omega_alpha_and_omega_observers_offer_real_controller_order(){
    let mut g=game();field(&mut g,"XG18",0);field(&mut g,"WM041",0);g.players[0].society_zone.exposure=1;fund(&mut g,0,"XG18",6);let id=held(&mut g,"XG17",0);deploy(&mut g,&id);
    let p=g.pending.as_ref().unwrap();assert_eq!(p.choice.kind,"batch3-entry-order");assert_eq!(p.choice.min,Some(1));assert_eq!(p.choice.options.len(),2);
    let option=p.choice.options.iter().find(|o|o.label.contains("欧米伽小姐")).unwrap().id.clone();choose(&mut g,vec![option]);finish(&mut g);
    assert_eq!(g.players[0].society_zone.exposure,3);for s in 1..4{assert_eq!(g.players[s].society_zone.exposure,1);}assert!(g.entry_order_groups.is_empty());restore(&mut g);
}
#[test]
fn batch4_omega_real_conditional_mobility_moves_once_with_original_sun_receipt(){
    let(mut g,source,_)=setup(false);g.players[0].society_zone.exposure=1;g.begin_window(Window::Mobility);
    let snapshot=g.mobility_source(g.board(&source).unwrap().1,0);g.emit_event(0,snapshot,Event::ConfrontationStart);g.drive().unwrap();
    let p=g.pending.as_ref().unwrap();assert_eq!(p.seat,0);let ChoiceResolution::Declare{declaration:d,..}=&p.resolution else{panic!()};assert_eq!(d.ability.key,"mobility");
    choose(&mut g,vec!["region:1".into()]);g.players[0].society_zone.exposure=0;restore(&mut g);pass_top(&mut g);
    assert_eq!(g.board(&source).unwrap().0,1);assert!(!g.has_mobility(g.board(&source).unwrap().1));restore(&mut g);
}
#[test]
fn batch4_omega_original_large_u64_asset_command_replays_all_four_views(){
    let(mut g,_,id)=setup(true);g.seed=u64::MAX-29;g.random=g.seed;let before=serde_json::to_string(&g).unwrap();
    let command=Action{card_id:Some(id),region:Some(0),..Action::new("deploy")};act(&mut g,0,command.clone());let expected=serde_json::to_string(&g).unwrap();
    let mut replay=Game::from_persisted(&before).unwrap();replay.apply(0,command).unwrap();assert_eq!(serde_json::to_string(&replay).unwrap(),expected);assert_eq!(replay.seed,u64::MAX-29);
    for seat in 0..4{assert_eq!(serde_json::to_value(replay.view(seat)).unwrap(),serde_json::to_value(g.view(seat)).unwrap());}
}
#[test]
fn batch4_omega_actual_rescuer_response_preserves_asset_permission_and_forced_reward(){
    let(mut g,source,id)=setup(false);deploy(&mut g,&id);let rescuer=field(&mut g,"JC075",0);fund(&mut g,0,"JC075",2);
    for _ in 0..4{if g.priority_team==g.team(0){break;}pass(&mut g);}
    let a=g.legal_actions(0).into_iter().find(|a|a.action.kind=="activate"&&a.action.card_id.as_deref()==Some(&rescuer)&&a.action.target_id.as_deref()==Some(&source)).unwrap().action;
    act(&mut g,0,a);pass_top(&mut g);assert!(g.in_play(&source).is_none());finish(&mut g);
    assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC067"));assert!(g.players.iter().all(|p|p.society_zone.exposure==1));restore(&mut g);
}
#[test]
fn batch4_omega_asset_actions_are_private_and_follow_current_renown_denial(){
    let(mut g,_,id)=setup(false);assert!(g.legal_actions(0).iter().any(|a|a.action.kind=="deploy"&&a.action.card_id.as_deref()==Some(&id)));
    for seat in 1..4{assert!(g.view(seat).legal_actions.iter().all(|a|a.action.card_id.as_deref()!=Some(&id)));}
    field(&mut g,"WM041",2);assert!(!g.legal_actions(0).iter().any(|a|a.action.kind=="deploy"&&a.action.card_id.as_deref()==Some(&id)));restore(&mut g);
}
#[test]
fn batch4_omega_duplicate_queued_reward_cannot_consume_one_play_twice(){
    let(mut g,_,id)=setup(false);deploy(&mut g,&id);let mut copy=g.stack.last().unwrap().clone();let id=g.id();copy.id=id.clone();copy.frame.as_mut().unwrap().frame_id=id;g.stack.push(copy);
    let value=serde_json::to_value(&g).unwrap();invalid(&g,value);
}
