//! Explicit native layouts; all actual commands round-trip opaque and four views.
use crate::{batch4_wendigo as w, catalog, model::*, rules::*, unit_support::*};
use crate::jc029_tests::pass_top;
fn grave(g:&mut Game,id:&str,actor:usize)->String {let c=g.make_card(id,actor);let uid=c.id.clone();g.players[actor].graveyard.push(c);uid}
fn activate(g:&mut Game,id:&str){act(g,0,Action{card_id:Some(id.into()),ability_id:Some(w::GRAVE.into()),..Action::new("activate")});}
fn deck(g:&mut Game,ids:&[&str])->Vec<String>{g.players[0].deck.clear();ids.iter().map(|id|{let c=g.make_card(id,0);let uid=c.id.clone();g.players[0].deck.push(c);uid}).collect()}
fn enter(g:&mut Game)->String {
    fund(g,0,"JZ51",5);let id=held(g,"JZ51",0);act(g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});
    pass_top(g);let id=g.regions[0].cards.iter().find(|c|c.definition=="JZ51").unwrap().id.clone();
    assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key==w::ENTRY));
    choose(g,vec!["accept".into()]);pass_top(g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"jz51-top-eight");id
}
fn unpaid()->(Game,String,Vec<String>){let mut g=game();let source=grave(&mut g,"JZ51",0);let ids=(0..4).map(|_|grave(&mut g,"XQ46",0)).collect();(g,source,ids)}
fn paid()->(Game,String){let(mut g,source,ids)=unpaid();activate(&mut g,&source);choose(&mut g,ids[..3].to_vec());(g,source)}
fn invalid(v:serde_json::Value){let result=std::panic::catch_unwind(||Game::from_persisted(&serde_json::to_string(&v).unwrap()));assert!(result.is_ok()&&result.unwrap().is_err());}
#[test]
fn batch4_wendigo_source_and_complete_definition_are_closed(){
    let c=catalog::card("JZ51");assert_eq!((c.name.as_str(),c.cost,c.defense,c.unique,c.supported),("温迪戈",5,Some(3),false,false));
    assert_eq!(c.subtypes,["不死生物","食尸鬼"]);assert_eq!(c.loyalty,["黑色"]);assert_eq!(c.magic,"死亡");
    let d=definition("JZ51");assert_eq!(d.abilities.len(),2);assert!(!d.graveyard_face_up);
    for n in 0..4 {let mut defs=definitions().clone();let d=defs.get_mut("JZ51").unwrap();match n {0=>d.abilities[1].costs.clear(),1=>d.abilities[1].timing=Timing::Fast,2=>d.abilities[0].event=Some(Event::Reveal),_=>d.graveyard_face_up=true};assert!(validate_definitions(&defs).is_err());}
    let a=d.abilities[0].clone();assert!(validate_ability("JC125",&a).is_err());
}
#[test]
fn batch4_wendigo_actual_paid_entry_private_top_eight_and_zero_defense_corpses(){
    let mut g=game();let ids=deck(&mut g,&["XQ46","JC125","XQ46","JC125","JC125","JC125","JC125","JC125","XQ46"]);
    enter(&mut g);assert_eq!(g.resources(0),0);let p=g.pending.as_ref().unwrap();assert_eq!(p.choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>(),vec![ids[0].clone(),ids[2].clone()]);
    assert_eq!(g.view(0).pending_choice.unwrap().preview_cards.len(),8);
    for s in 1..4 {assert!(g.view(s).pending_choice.is_none());}
    choose(&mut g,vec![ids[0].clone(),ids[2].clone()]);
    assert_eq!(g.players[0].deck.len(),7);assert!(g.players[0].deck.iter().any(|c|c.id==ids[8]));
    assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="XQ46").count(),2);
    assert!(g.players[0].graveyard.iter().all(|c|!ids.contains(&c.id)));restore(&mut g);
}
#[test]
fn batch4_wendigo_grave_cost_uses_own_three_and_optional_society_zone(){
    let mut g=game();assert!(g.players[0].society_zone.card.is_none());let source=grave(&mut g,"JZ51",0);
    let own=(0..3).map(|_|grave(&mut g,"XQ46",0)).collect::<Vec<_>>();let ally=grave(&mut g,"XQ46",2);let resources=g.resources(0);
    let legal=g.view(0).legal_actions.iter().any(|a|a.action.card_id.as_deref()==Some(&source)&&a.action.ability_id.as_deref()==Some(w::GRAVE));assert!(legal);
    activate(&mut g,&source);assert_eq!(g.players[0].graveyard.len(),4);assert!(g.stack.is_empty());assert_eq!(g.pending.as_ref().unwrap().choice.options.len(),3);
    choose(&mut g,own.clone());assert_eq!(g.sealed_cards.len(),3);assert!(g.sealed_cards.iter().all(|s|s.host_id=="society:p0" && !own.contains(&s.card.id)));
    assert!(g.players[2].graveyard.iter().any(|c|c.id==ally));assert_eq!(g.resources(0),resources);
    let f=g.stack.last().unwrap().frame.as_ref().unwrap();assert_eq!(f.ability_key,w::GRAVE);assert!(matches!(f.already_paid.as_slice(),[PaidCost::JZ51Corpses(p)] if p.corpses.len()==3));
    pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"jz51-entry-region");choose(&mut g,vec!["region:0".into()]);
    assert!(!g.players[0].graveyard.iter().any(|c|c.id==source));assert!(g.regions[0].cards.iter().any(|c|c.definition=="JZ51"&&c.id!=source));
    assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key==w::ENTRY));
    choose(&mut g,vec![]);restore(&mut g);
}
#[test]
fn batch4_wendigo_top_eight_accepts_zero_and_a_proper_subset(){
    for n in [0,1] {let mut g=game();let ids=deck(&mut g,&["XQ46","JC125","XQ46","JC125"]);enter(&mut g);
        choose(&mut g,ids.iter().take(n).cloned().collect());assert_eq!(g.players[0].deck.len(),4-n);
        assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="XQ46").count(),n);restore(&mut g);
    }
}
#[test]
fn batch4_wendigo_empty_or_short_library_still_has_private_finite_choice(){
    for cards in [vec![],vec!["JC125"],vec!["XQ46"]] {let mut g=game();deck(&mut g,&cards);enter(&mut g);
        let n=cards.len();assert_eq!(g.view(0).pending_choice.unwrap().preview_cards.len(),n);for s in 1..4{assert!(g.view(s).pending_choice.is_none());}
        choose(&mut g,vec![]);assert_eq!(g.players[0].deck.len(),n);restore(&mut g);
    }
}
#[test]
fn batch4_wendigo_search_rejects_ninth_card_duplicates_and_extra_payload_atomically(){
    let mut g=game();let ids=deck(&mut g,&["XQ46","JC125","XQ46","JC125","JC125","JC125","JC125","JC125","XQ46"]);enter(&mut g);let p=g.pending.clone().unwrap();
    for selected in [vec![ids[8].clone()],vec![ids[0].clone(),ids[0].clone()],vec![ids[1].clone()]] {reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(selected),..Action::new("choose")});}
    reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec![]),region:Some(4),..Action::new("choose")});
}
#[test]
fn batch4_wendigo_grave_action_does_not_borrow_ally_corpses_or_other_zones(){
    let mut g=game();let source=grave(&mut g,"JZ51",0);for _ in 0..2{grave(&mut g,"XQ46",0);}for _ in 0..3{grave(&mut g,"XQ46",2);}
    let a=Action{card_id:Some(source.clone()),ability_id:Some(w::GRAVE.into()),..Action::new("activate")};reject(&mut g,0,a.clone());
    grave(&mut g,"JC125",0);reject(&mut g,0,a.clone());grave(&mut g,"XQ46",0);activate(&mut g,&source);
    let mut g=game();for _ in 0..3{grave(&mut g,"XQ46",0);}for zone in 0..3{
        let id=if zone==0{held(&mut g,"JZ51",0)}else if zone==1{field(&mut g,"JZ51",0)}else{grave(&mut g,"JZ51",2)};
        reject(&mut g,0,Action{card_id:Some(id),ability_id:Some(w::GRAVE.into()),..Action::new("activate")});
    }
}
#[test]
fn batch4_wendigo_grave_action_is_standard_only_and_choice_cannot_change_payment(){
    let(mut g,source,ids)=unpaid();let a=Action{card_id:Some(source.clone()),ability_id:Some(w::GRAVE.into()),..Action::new("activate")};
    for window in [Window::Draw,Window::Action(1),Window::End]{g.begin_window(window);reject(&mut g,0,a.clone());}
    g.begin_window(Window::Action(0));let mut wrong=a.clone();wrong.region=Some(0);reject(&mut g,0,wrong);activate(&mut g,&source);let p=g.pending.clone().unwrap();
    for selected in [vec![],ids[..2].to_vec(),vec![ids[0].clone();3]] {reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(selected),..Action::new("choose")});}
    reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(ids[..3].to_vec()),cost_selected:Some(vec![]),..Action::new("choose")});
    assert!(g.sealed_cards.is_empty());assert_eq!(g.players[0].graveyard.len(),5);
}
#[test]
fn batch4_wendigo_paid_grave_action_cannot_replay_or_charge_again(){
    let(mut g,source,ids)=unpaid();activate(&mut g,&source);let p=g.pending.clone().unwrap();choose(&mut g,ids[..3].to_vec());
    let count=g.sealed_cards.len();reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(ids[..3].to_vec()),..Action::new("choose")});
    reject(&mut g,0,Action{card_id:Some(source),ability_id:Some(w::GRAVE.into()),..Action::new("activate")});assert_eq!(g.sealed_cards.len(),count);restore(&mut g);
}
#[test]
fn batch4_wendigo_replaced_original_region_does_not_receive_search_results(){
    let mut g=game();let ids=deck(&mut g,&["XQ46","JC125"]);enter(&mut g);
    // Explicit region replacement fixture while the private choice is pending.
    // No public rule changes: this exercises the stored original-region UID.
    g.regions[0].card=g.make_card("DQJC115",0);choose(&mut g,vec![ids[0].clone()]);
    assert!(g.players[0].deck.iter().any(|c|c.id==ids[0]));assert!(g.players[0].graveyard.is_empty());restore(&mut g);
}
#[test]
fn batch4_wendigo_normal_grave_play_via_jz52_remains_printed_cost_play(){
    let mut g=game();field(&mut g,"JZ52",0);fund(&mut g,0,"JZ51",5);let source=grave(&mut g,"JZ51",0);
    act(&mut g,0,Action{card_id:Some(source),region:Some(0),..Action::new("deploy")});assert_eq!(g.resources(0),0);assert!(g.sealed_cards.is_empty());
    pass_top(&mut g);assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key==w::ENTRY));choose(&mut g,vec![]);restore(&mut g);
}
#[test]
fn batch4_wendigo_unpaid_draft_rejects_forged_actor_options_and_turn(){
    let(mut g,source,_)=unpaid();activate(&mut g,&source);let base=serde_json::to_value(&g).unwrap();
    for n in 0..8 {let mut v=base.clone();match n {0=>v["pending"]["seat"]=2.into(),1=>v["pending"]["choice"]["min"]=2.into(),2=>v["pending"]["choice"]["max"]=4.into(),3=>v["pending"]["choice"]["allowDecline"]=true.into(),4=>v["pending"]["resolution"]["JZ51Payment"]["draft"]["turn"]=999.into(),5=>v["pending"]["choice"]["options"][0]["id"]="unknown".into(),6=>v["pending"]["choice"]["title"]="foreign".into(),_=>v["pending"]["resolution"]["JZ51Payment"]["draft"]["source"]["card"]["owner"]=2.into()};invalid(v);}
}
#[test]
fn batch4_wendigo_paid_receipt_rejects_wrong_host_cost_source_and_operation(){
    let(g,_)=paid();let base=serde_json::to_value(&g).unwrap();
    for n in 0..8 {let mut v=base.clone();let f=&mut v["stack"][0]["frame"];match n {0=>f["already_paid"][0]["JZ51Corpses"]["host_id"]="society:p2".into(),1=>{f["already_paid"][0]["JZ51Corpses"]["corpses"].as_array_mut().unwrap().pop();},2=>f["already_paid"][0]["JZ51Corpses"]["corpses"][1]=f["already_paid"][0]["JZ51Corpses"]["corpses"][0].clone(),3=>f["source"]["card"]["definition"]="JC125".into(),4=>f["steps"][0]["op"]="JZ51TopEightCorpses".into(),5=>f["actor"]=2.into(),6=>f["already_paid"][0]["JZ51Corpses"]["corpses"][0]["owner"]=2.into(),_=>f["chosen_region"]=0.into()};invalid(v);}
}
#[test]
fn batch4_wendigo_private_search_rejects_inspected_snapshot_and_choice_transplants(){
    let mut g=game();deck(&mut g,&["XQ46","JC125"]);enter(&mut g);let base=serde_json::to_value(&g).unwrap();
    for n in 0..6 {let mut v=base.clone();match n {0=>v["pending"]["resolution"]["Frame"]["choice"]["JZ51TopEight"]["inspected_ids"][0]="other".into(),1=>v["pending"]["choice"]["options"][0]["id"]="other".into(),2=>v["pending"]["choice"]["max"]=8.into(),3=>v["pending"]["resolution"]["Frame"]["frame"]["cursor"]=0.into(),4=>v["pending"]["seat"]=2.into(),_=>v["pending"]["resolution"]["Frame"]["choice"]="JZ50DeathSearch".into()};invalid(v);}
}
#[test]
fn batch4_wendigo_entry_trigger_survives_actual_rescuer_to_original_region(){
    let mut g=game();let ids=deck(&mut g,&["XQ46","JC125"]);fund(&mut g,0,"JZ51",5);let hand=held(&mut g,"JZ51",0);
    act(&mut g,0,Action{card_id:Some(hand),region:Some(0),..Action::new("deploy")});pass_top(&mut g);
    let source=g.regions[0].cards.iter().find(|c|c.definition=="JZ51").unwrap().id.clone();choose(&mut g,vec!["accept".into()]);
    let rescuer=field(&mut g,"JC075",0);fund(&mut g,0,"JC075",2);
    for _ in 0..4 {if g.priority_team==g.team(0){break;}pass(&mut g);}
    let a=g.legal_actions(0).into_iter().find(|a|a.action.kind=="activate" && a.action.card_id.as_deref()==Some(&rescuer) && a.action.target_id.as_deref()==Some(&source)).unwrap().action;
    act(&mut g,0,a);pass_top(&mut g);assert!(g.board(&source).is_none());pass_top(&mut g);
    choose(&mut g,vec![ids[0].clone()]);assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="XQ46").count(),1);
    assert!(g.players[0].hand.iter().any(|c|c.definition=="JZ51" && c.id!=source));restore(&mut g);
}
fn room_step(r:&mut crate::room::RoomEnvelope,seat:usize,action:crate::room::SessionAction,now:u64)->crate::room::RoomCommand {
    let command=crate::room::RoomCommand{command_id:format!("wendigo-room-{}",r.revision),expected_version:r.revision,action};let result=r.transition(seat,Some(command.clone()),now).unwrap();
    assert!(result.error_code.is_none(),"{:?}",result.error_message);*r=crate::room::RoomEnvelope::from_persisted(&result.state).unwrap();command
}
#[test]
fn batch4_wendigo_room_pause_cost_response_replay_and_high_u64_restore(){
    use crate::room::{RoomEnvelope,SessionAction};let(mut g,source,ids)=unpaid();g.seed=u64::MAX-113;g.random=g.seed;activate(&mut g,&source);
    let choice=g.pending.as_ref().unwrap().choice.id.clone();let mut r=crate::jc029_tests::envelope(&g);
    room_step(&mut r,2,SessionAction::PauseRoom,1000);r=RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();assert_eq!(r.game.pending.as_ref().unwrap().choice.id,choice);
    room_step(&mut r,1,SessionAction::ResumeRoom,2000);let before=serde_json::to_string(&r).unwrap();
    let a=Action{choice_id:Some(choice),selected:Some(ids[..3].to_vec()),..Action::new("choose")};let command=room_step(&mut r,0,SessionAction::Game{action:a},3000);
    let original=RoomEnvelope::from_persisted(&before).unwrap();let replay=original.transition(0,Some(command.clone()),3000).unwrap();assert_eq!(replay.state,serde_json::to_string(&r).unwrap());
    let duplicate=r.transition(0,Some(command),3001).unwrap();assert_eq!(duplicate.error_code.as_deref(),Some("version_conflict"));assert_eq!(duplicate.state,serde_json::to_string(&r).unwrap());assert_eq!(r.game.sealed_cards.len(),3);
    for n in 0..16 {if r.game.stack.is_empty(){break;}let id=r.pacing.window.as_ref().unwrap().id.clone();let seat=(0..4).find(|s|r.game.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();room_step(&mut r,seat,SessionAction::PassResponse{window_id:id},4000+n);}
    assert_eq!(r.game.pending.as_ref().unwrap().choice.kind,"jz51-entry-region");let restored=RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();
    for seat in 0..4 {assert_eq!(serde_json::to_value(r.view(seat,5000)).unwrap(),serde_json::to_value(restored.view(seat,5000)).unwrap());}assert_eq!(restored.game.seed,u64::MAX-113);
}
#[test]
fn batch4_wendigo_any_number_all_eight_are_not_capped_at_three(){
    let mut g=game();let ids=deck(&mut g,&["XQ46";8]);enter(&mut g);choose(&mut g,ids.clone());
    assert!(g.players[0].deck.is_empty());assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="XQ46").count(),8);
    assert!(g.players[0].graveyard.iter().all(|c|!ids.contains(&c.id)));restore(&mut g);
}
#[test]
fn batch4_wendigo_paid_martial_law_allows_both_direct_placement_programs(){
    let(mut g,source,ids)=unpaid();let inspected=deck(&mut g,&["XQ46"]);fund(&mut g,0,"JC057",2);let law=held(&mut g,"JC064",0);
    act(&mut g,0,Action{card_id:Some(law),region:Some(0),..Action::new("play")});top(&mut g);
    assert!(g.attachments.iter().any(|a|a.card.definition=="JC064"&&a.host_id==g.regions[0].card.id));
    activate(&mut g,&source);choose(&mut g,ids[..3].to_vec());pass_top(&mut g);assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id=="region:0"));
    choose(&mut g,vec!["region:0".into()]);assert!(g.regions[0].cards.iter().any(|c|c.definition=="JZ51"));
    choose(&mut g,vec!["accept".into()]);pass_top(&mut g);choose(&mut g,inspected.clone());
    assert!(g.players[0].deck.is_empty());assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="XQ46").count(),2);restore(&mut g);
}
#[test]
fn batch4_wendigo_grave_entry_choice_keeps_exact_original_uid(){
    let(mut g,source)=paid();pass_top(&mut g);
    // Explicit replacement-in-grave fixture tests an old-UID reference. Normal
    // authenticated commands are blocked while choosing; no legal extra window
    // is invented to mutate a pending choice.
    g.players[0].graveyard.retain(|c|c.id!=source);grave(&mut g,"JZ51",0);
    assert!(Game::from_persisted(&serde_json::to_string(&g).unwrap()).is_err());
}
