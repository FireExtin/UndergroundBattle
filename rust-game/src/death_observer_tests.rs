//! Source-pinned finite death observer regression cases.
//! Layout primitives are explicit offline fixtures, never natural-room evidence.
use crate::jc029_tests::{apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject};
use crate::{catalog, model::*, room::*, rules::*};
use std::{collections::BTreeMap, sync::atomic::{AtomicUsize, Ordering}};
static BAD: AtomicUsize = AtomicUsize::new(0);
fn initial(actor: usize) -> Game {
    let mut g = game(actor);
    for r in &mut g.regions { r.influence = [0;2]; }
    g
}
fn held(g: &mut Game, def: &str, seat: usize) -> String {
    let c=g.make_card(def,seat); let id=c.id.clone();g.players[seat].hand.push(c);id
}
fn contract(g: &mut Game, seat: usize, region: usize) -> String {
    let c=g.make_card("JC090",seat);let id=c.id.clone();
    g.attachments.push(Attachment{card:c,host_id:g.regions[region].card.id.clone()});id
}
fn observers(g: &mut Game, seat: usize) -> (String,String,String) {
    (board(g,"JC045",seat,2),board(g,"JC031",seat,2),contract(g,seat,1))
}
fn declarations(g: &Game) -> Vec<Declaration> {
    g.effects.iter().filter_map(|e| match e {Effect::Declare{declaration}
        if declaration.ability.event==Some(Event::CharacterDeathObserved)=>Some(declaration.clone()),_=>None}).collect()
}
fn die(g: &mut Game,id:&str,cause:RemovalCause) {
    g.remove_dead(id,cause);g.drive().unwrap();checkpoint(g);
}
fn pending(g:&Game)->&Declaration {
    let ChoiceResolution::Declare{declaration,..}=&g.pending.as_ref().unwrap().resolution else {panic!("declaration expected")};declaration
}
fn finish(g:&mut Game,target:usize) {
    for _ in 0..100 {
        g.drive().unwrap();
        if let Some(p)=&g.pending {
            let ids=match &p.resolution {
                ChoiceResolution::Declare{declaration,stage}=>{
                    if declaration.ability.event==Some(Event::CharacterDeathObserved) {
                        vec![if matches!(stage,DeclareChoice::Target){format!("p{target}")}else{"accept".into()}]
                    }else if declaration.ability.event==Some(Event::Death)&&declaration.source.card.definition=="JZ31"{vec!["accept".into()]}else{vec![]}
                },
                ChoiceResolution::Frame{choice:FrameChoice::Discard{..},..}=>vec![p.choice.options[0].id.clone()],
                _=>panic!("unexpected pending: {:?}",p),
            };choose(g,ids);
        }else if !g.stack.is_empty(){pass_top(g);}else{checkpoint(g);return;}
    }panic!("finite observer fixture did not finish");
}
fn cast(g:&mut Game,def:&str,actor:usize,target:&str) {
    let id=held(g,def,actor);let a=g.legal_actions(actor).into_iter().find(|a|
        a.action.kind=="play"&&a.action.card_id.as_deref()==Some(&id)&&a.action.target_id.as_deref()==Some(target)).unwrap().action;
    apply(g,actor,a);pass_top(g);
}
fn fixture(name:&str,g:&Game) {
    checkpoint(g);
    if let Ok(dir)=std::env::var("DEATH_OBSERVER_FRONTEND_DIR") {
        std::fs::create_dir_all(&dir).unwrap();let r=envelope(g);
        std::fs::write(format!("{dir}/{name}.json"),serde_json::to_vec(&serde_json::json!({
            "state":serde_json::to_string(&r).unwrap(),"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
}
fn bad_state(g:&Game,v:serde_json::Value) {
    let state=serde_json::to_string(&v).unwrap();assert!(Game::from_persisted(&state).is_err());
    if let Ok(dir)=std::env::var("DEATH_OBSERVER_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();let mut r=serde_json::to_value(envelope(g)).unwrap();r["game"]=v;
        let n=BAD.fetch_add(1,Ordering::Relaxed);std::fs::write(format!("{dir}/invalid-{n:03}.json"),
            serde_json::to_vec(&serde_json::json!({"state":serde_json::to_string(&r).unwrap()})).unwrap()).unwrap();
    }
}

#[test]
fn death_observer_original_fields_three_programs_and_unchanged_initiative_icons() {
    let p=catalog::card("JC045");assert_eq!((p.cost,p.defense),(4,Some(2)));
    assert_eq!(p.loyalty,["红色","红色"]);assert_eq!(p.subtypes,["人类","法师","邪教徒"]);
    assert_eq!(p.magic_icon,MagicIcon::Blood);assert_eq!(p.permanent_icons,Icons{investigation:1,combat:2,influence:0});
    let c=catalog::card("JC031");assert_eq!((c.cost,c.defense),(3,Some(1)));
    assert_eq!(c.loyalty,["蓝色","蓝色"]);assert_eq!(c.subtypes,["吸血鬼","法师"]);assert_eq!(c.magic_icon,MagicIcon::Death);
    assert_eq!(c.permanent_icons,Icons{influence:1,..Icons::default()});assert_eq!(c.temporary_icons,Icons{investigation:1,..Icons::default()});
    let a=catalog::card("JC090");assert_eq!((&*a.kind,a.cost,a.defense),("attachment",2,None));
    assert_eq!(a.loyalty,["黑色"]);assert_eq!(a.subtypes,["仪式"]);assert_eq!(a.magic_icon,MagicIcon::Mind);
    for id in ["JC045","JC031","JC090"] {
        let d=definition(id);validate_ability(id,d.abilities.iter().find(|a|a.event==Some(Event::CharacterDeathObserved)).unwrap()).unwrap();
        assert!(catalog::card(id).keywords.is_empty()&&!catalog::card(id).unique);
    }
    for actor in 0..4 {let mut g=initial(actor);let id=board(&mut g,"JC031",actor,2);
        for first in 0..2 {g.first_team=first;let c=g.board(&id).unwrap().1;
            assert_eq!(g.current_icons(c,2),Icons{influence:1,investigation:u32::from(g.team(actor)==first),combat:0});}}
}

#[test]
fn death_observer_real_paid_deployment_and_attachment_require_printed_domains() {
    for (id,cost) in [("JC045",4),("JC031",3),("JC090",2)] {
        let mut g=initial(0);let h=held(&mut g,id,0);fund(&mut g,0,"JC125",cost);
        let action=Action{card_id:Some(h.clone()),region:Some(2),target_id:(id=="JC090").then(||"region:2".into()),
            ..Action::new(if id=="JC090"{"play"}else{"deploy"})};
        reject(&mut g,0,action.clone());g.players[0].assets.clear();fund(&mut g,0,id,cost);apply(&mut g,0,action);pass_top(&mut g);
        if id=="JC090" {assert!(g.attachments.iter().any(|a|a.card.definition==id&&a.host_id==g.regions[2].card.id));}
        else{assert!(g.regions[2].cards.iter().any(|c|c.definition==id));}checkpoint(&g);
    }
}

#[test]
fn death_observer_real_destroy_lethal_and_paid_sacrifice_use_same_entry() {
    for cause in 0..3 {let mut g=initial(0);let (priest,_,_)=observers(&mut g,0);let dead=board(&mut g,"JC125",0,2);held(&mut g,"LC19",1);
        if cause==0 {fund(&mut g,0,"JC084",3);cast(&mut g,"JC091",0,&dead);}
        else if cause==1 {fund(&mut g,0,"JC104",2);cast(&mut g,"JC102",0,&dead);}
        else {fund(&mut g,0,"JC045",2);let spell=held(&mut g,"JC049",0);
            let mut a=g.legal_actions(0).into_iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_deref()==Some(&spell)).unwrap().action;
            a.cost_selected=Some(vec![dead.clone()]);apply(&mut g,0,a);}
        assert!(g.board(&dead).is_none());finish(&mut g,1);
        assert_eq!(g.board(&priest).unwrap().1.time_markers,1);assert_eq!(g.regions[1].influence,[1,0]);assert!(g.players[1].hand.is_empty());
    }
}

#[test]
fn death_observer_all_removal_causes_and_optional_decline() {
    for cause in [RemovalCause::Sacrifice,RemovalCause::Destroy,RemovalCause::Lethal] {let mut g=initial(0);
        let (p,_,_)=observers(&mut g,0);let d=board(&mut g,"JC125",0,0);held(&mut g,"LC19",1);die(&mut g,&d,cause);
        let mut declined=g.clone();for _ in 0..10{if declined.pending.is_some(){choose(&mut declined,vec![]);}else{declined.drive().unwrap();if declined.pending.is_none(){break;}}}
        assert_eq!(declined.board(&p).unwrap().1.time_markers,0);assert_eq!(declined.regions[1].influence,[0,0]);assert!(declined.turn_ability_usage.is_empty());
        finish(&mut g,1);assert_eq!(g.board(&p).unwrap().1.time_markers,1);assert_eq!(g.players[1].graveyard.len(),1);
    }
}

#[test]
fn death_observer_hidden_return_bottom_and_region_removal_do_not_trigger() {
    for mode in 0..4 {let mut g=initial(0);observers(&mut g,0);let d=board(&mut g,"JC125",0,0);
        match mode {0=>{g.board_mut(&d).unwrap().face_down=true;g.remove_dead(&d,RemovalCause::Destroy);},1=>g.return_hand(&d),2=>g.to_bottom(&d),_=>{let def=g.regions[0].card.definition.clone();g.regions[0].card=g.make_card(&def,0);}}
        assert!(declarations(&g).is_empty());g.drive().unwrap();assert!(g.pending.is_none());checkpoint(&g);
    }
}

#[test]
fn death_observer_simultaneous_dying_observers_still_observe_whole_wave() {
    let mut g=initial(0);let (p,c,_)=observers(&mut g,0);held(&mut g,"LC19",1);
    g.damage(BTreeMap::from([(p.clone(),2),(c.clone(),1)])).unwrap();let ds=declarations(&g);
    assert_eq!(ds.iter().filter(|d|d.source.card.id==p).count(),2);
    assert_eq!(ds.iter().filter(|d|d.source.card.id==c).count(),1);
    assert_eq!(ds.iter().filter(|d|d.source.card.definition=="JC090").count(),2);
    assert!(!ds.iter().find(|d|d.source.card.id==c).unwrap().source.observed_death.as_ref().unwrap().vampire);
    checkpoint(&g);finish(&mut g,1);assert_eq!(g.regions[1].influence,[2,0]);assert!(g.players[1].hand.is_empty());
    assert!(g.players[0].graveyard.iter().all(|c|c.time_markers==0));
}

#[test]
fn death_observer_cascade_captures_a_new_wave_and_excludes_prior_departures() {
    let mut g=initial(0);let (p,c,_)=observers(&mut g,0);let aura=board(&mut g,"JC059",0,2);held(&mut g,"LC19",1);
    g.board_mut(&c).unwrap().damage=1;let pd=g.defense(g.board(&p).unwrap().1,2);let ad=g.defense(g.board(&aura).unwrap().1,2);
    g.damage(BTreeMap::from([(p.clone(),pd),(aura.clone(),ad)])).unwrap();assert!(g.board(&c).is_none());let ds=declarations(&g);
    assert_eq!(ds.iter().filter(|d|d.source.card.id==p).count(),2);
    assert!(ds.iter().filter(|d|d.source.card.id==p).all(|d|d.source.observed_death.as_ref().unwrap().card.id!=c));
    assert_eq!(ds.iter().filter(|d|d.source.card.id==c).count(),2);
    assert_eq!(ds.iter().filter(|d|d.source.card.definition=="JC090").count(),3);finish(&mut g,1);assert_eq!(g.regions[1].influence,[3,0]);
}

#[test]
fn death_observer_late_arrival_self_exclusion_and_exhaustion() {
    let mut g=initial(0);let dead=board(&mut g,"JC125",0,2);g.remove_dead(&dead,RemovalCause::Destroy);observers(&mut g,0);assert!(declarations(&g).is_empty());
    let c=g.regions[2].cards.iter().find(|c|c.definition=="JC031").unwrap().id.clone();g.remove_dead(&c,RemovalCause::Destroy);
    assert!(declarations(&g).iter().all(|d|d.source.card.id!=c));finish(&mut g,1);
    let mut g=initial(0);let (p,c,_)=observers(&mut g,0);g.board_mut(&p).unwrap().exhausted=true;g.board_mut(&c).unwrap().exhausted=true;
    let d=board(&mut g,"JC125",0,0);held(&mut g,"LC19",1);die(&mut g,&d,RemovalCause::Destroy);finish(&mut g,1);
    assert_eq!(g.board(&p).unwrap().1.time_markers,1);assert!(g.players[1].hand.is_empty());
}

#[test]
fn death_observer_controller_owner_teams_and_identical_instances_are_independent() {
    for actor in 0..4 {let mut g=initial(actor);let teammate=(0..4).find(|&s|s!=actor&&g.team(s)==g.team(actor)).unwrap();let owner=(actor+1)%4;
        let (p,c,a)=observers(&mut g,actor);let (_,tc,ta)=observers(&mut g,teammate);
        let victim=board(&mut g,"JC125",owner,0);g.board_mut(&victim).unwrap().controller=actor;
        g.board_mut(&p).unwrap().owner=owner;g.attachments.iter_mut().find(|x|x.card.id==a).unwrap().card.owner=owner;
        g.remove_dead(&victim,RemovalCause::Destroy);let ds=declarations(&g);
        assert!(ds.iter().any(|d|d.source.card.id==c&&d.actor==actor));
        assert!(!ds.iter().any(|d|d.source.card.id==tc||d.source.card.id==ta));
        assert!(ds.iter().any(|d|d.source.card.id==a&&d.actor==actor&&d.source.card.owner==owner));
        assert_eq!(g.players[owner].graveyard.len(),1);assert_eq!(g.players[owner].graveyard[0].controller,owner);
        finish(&mut g,owner);assert_eq!(g.regions[1].influence[g.team(actor)],1);
        assert_eq!(g.board(&p).unwrap().1.time_markers,1);
    }
}

#[test]
fn death_observer_actual_vampire_conversion_is_frozen_before_control_source_leaves() {
    let mut g=initial(0);let c=board(&mut g,"JC031",0,2);let provider=board(&mut g,"LC19",0,0);let dead=board(&mut g,"JC125",1,2);
    g.control_baselines.push(ControlBaseline{target_instance:dead.clone(),controller:1});
    g.control_effects.push(ControlEffect{target_instance:dead.clone(),recipient:0,
        lifetime:ControlLifetime::SourceLeaves{source_instance:provider.clone()},subtype_change:SubtypeChange::HumanToVampire});g.settle_controls();
    let mut h=g.make_card("LC19",2);h.controller=1;g.players[1].hand.push(h);held(&mut g,"JC125",1);
    let damage=g.defense(g.board(&provider).unwrap().1,0);g.damage(BTreeMap::from([(provider,damage),(dead.clone(),1)])).unwrap();
    let ds=declarations(&g);let d=ds.iter().find(|d|d.source.card.id==c&&d.source.observed_death.as_ref().unwrap().card.id==dead).unwrap();
    assert!(d.source.observed_death.as_ref().unwrap().vampire);assert_eq!(d.source.observed_death.as_ref().unwrap().card.controller,0);
    // Two deaths qualify. Decline the first, accepting the converted victim's trigger.
    g.drive().unwrap();choose(&mut g,vec![]);assert!(pending(&g).source.observed_death.as_ref().unwrap().vampire);
    let rng=g.random;choose(&mut g,vec!["p1".into()]);fixture("casterRandomStack",&g);pass_top(&mut g);
    assert_ne!(g.random,rng);assert!(g.pending.is_none());assert_eq!(g.players[1].hand.len(),1);
    assert_eq!(g.players[1].graveyard.len()+g.players[2].graveyard.len(),2);checkpoint(&g);
}

#[test]
fn death_observer_normal_discard_private_choice_random_and_empty_hand() {
    for vampire in [false,true] {let mut g=initial(0);board(&mut g,"JC031",0,2);let dead=board(&mut g,if vampire{"JC031"}else{"JC125"},0,0);
        let h=held(&mut g,"LC19",1);held(&mut g,"JC125",1);die(&mut g,&dead,RemovalCause::Destroy);fixture(if vampire{"casterRandomTrigger"}else{"casterTrigger"},&g);
        choose(&mut g,vec!["p1".into()]);let rng=g.random;pass_top(&mut g);
        if vampire {assert_ne!(g.random,rng);assert!(g.pending.is_none());}
        else {assert_eq!(g.random,rng);assert_eq!(g.pending.as_ref().unwrap().seat,1);fixture("casterDiscard",&g);
            for s in [0,2,3] {let v=g.view(s);assert!(v.pending_choice.is_none());assert!(v.hand.iter().all(|c|c.instance_id!=h));}
            let choice_id=g.pending.as_ref().unwrap().choice.id.clone();reject(&mut g,0,Action{choice_id:Some(choice_id),selected:Some(vec![h.clone()]),..Action::new("choose")});choose(&mut g,vec![h]);}
        assert_eq!(g.players[1].hand.len(),1);checkpoint(&g);
        let mut empty=initial(0);board(&mut empty,"JC031",0,2);let d=board(&mut empty,if vampire{"JC031"}else{"JC125"},0,0);die(&mut empty,&d,RemovalCause::Destroy);
        let rng=empty.random;choose(&mut empty,vec!["p1".into()]);pass_top(&mut empty);assert_eq!(empty.random,rng);assert!(empty.pending.is_none());
    }
}

#[test]
fn death_observer_caster_limit_is_acceptance_per_original_instance_and_per_turn() {
    let mut g=initial(0);let c=board(&mut g,"JC031",0,2);let d=board(&mut g,"JC125",0,0);die(&mut g,&d,RemovalCause::Destroy);choose(&mut g,vec![]);assert!(g.turn_ability_usage.is_empty());
    let d=board(&mut g,"JC125",0,0);die(&mut g,&d,RemovalCause::Destroy);choose(&mut g,vec!["p1".into()]);assert_eq!(g.turn_ability_usage[0].uses,1);
    g.players[1].eliminated=true;pass_top(&mut g);assert_eq!(g.turn_ability_usage[0].uses,1);
    let d=board(&mut g,"JC125",0,0);die(&mut g,&d,RemovalCause::Destroy);assert!(g.pending.is_none());
    let fresh=board(&mut g,"JC031",0,2);assert_ne!(fresh,c);let d=board(&mut g,"JC125",0,0);die(&mut g,&d,RemovalCause::Destroy);assert_eq!(pending(&g).source.card.id,fresh);choose(&mut g,vec![]);
    g.turn+=1;g.turn_ability_usage.clear();let d=board(&mut g,"JC125",0,0);die(&mut g,&d,RemovalCause::Destroy);assert_eq!(pending(&g).source.card.id,c);finish(&mut g,3);
}

#[test]
fn death_observer_priest_exact_markers_move_control_exhaustion_and_leave_reset() {
    let mut g=initial(0);let p=board(&mut g,"JC045",0,2);
    for n in 1..=5 {let d=board(&mut g,"JC125",1,0);die(&mut g,&d,RemovalCause::Destroy);finish(&mut g,1);
        assert_eq!(g.board(&p).unwrap().1.time_markers,n);let (r,c)=g.board(&p).unwrap();assert_eq!(g.current_icons(c,r).influence,n);}
    g.first_team=1;g.board_mut(&p).unwrap().exhausted=true;assert_eq!(g.current_icons(g.board(&p).unwrap().1,2).influence,5);fixture("priestMarks",&g);
    let (_,mut c)=g.remove_board(&p).unwrap();c.controller=1;g.regions[3].cards.push(c);assert_eq!(g.current_icons(g.board(&p).unwrap().1,3).influence,5);
    g.return_hand(&p);assert_eq!(g.players[0].hand[0].time_markers,0);assert_ne!(g.players[0].hand[0].id,p);checkpoint(&g);
    let mut clean=initial(0);let c=board(&mut clean,"JC125",0,0);let v=serde_json::to_value(clean.board(&c).unwrap().1).unwrap();assert!(v.get("time_markers").is_none());
    assert!(serde_json::to_value(clean.source_snapshot(clean.board(&c).unwrap().1,Some(0))).unwrap().get("observed_death").is_none());
}

#[test]
fn death_observer_priest_original_instance_and_caster_effect_survive_source_departure() {
    let mut g=initial(0);let p=board(&mut g,"JC045",0,2);let d=board(&mut g,"JC125",1,0);die(&mut g,&d,RemovalCause::Destroy);choose(&mut g,vec!["accept".into()]);
    g.return_hand(&p);let fresh=board(&mut g,"JC045",0,2);pass_top(&mut g);assert_eq!(g.board(&fresh).unwrap().1.time_markers,0);checkpoint(&g);
    let mut g=initial(0);let c=board(&mut g,"JC031",0,2);let d=board(&mut g,"JC125",0,0);held(&mut g,"LC19",1);die(&mut g,&d,RemovalCause::Destroy);choose(&mut g,vec!["p1".into()]);
    g.return_hand(&c);pass_top(&mut g);assert!(g.pending.is_some());finish(&mut g,1);assert!(g.players[1].hand.is_empty());
}

#[test]
fn death_observer_contract_freezes_original_region_before_and_after_response() {
    for before_accept in [false,true] {let mut g=initial(0);contract(&mut g,0,1);let d=board(&mut g,"JC125",0,4);die(&mut g,&d,RemovalCause::Destroy);fixture("contractTrigger",&g);
        if !before_accept {choose(&mut g,vec!["accept".into()]);}
        let def=g.regions[1].card.definition.clone();g.regions[1].card=g.make_card(&def,0);g.settle_attachments();
        if before_accept {choose(&mut g,vec!["accept".into()]);}pass_top(&mut g);assert_eq!(g.regions[1].influence,[0,0]);checkpoint(&g);
    }
    let mut g=initial(0);let a=contract(&mut g,0,1);let d=board(&mut g,"JC125",0,4);die(&mut g,&d,RemovalCause::Destroy);choose(&mut g,vec!["accept".into()]);
    let host=g.regions[3].card.id.clone();g.attachments.iter_mut().find(|a1|a1.card.id==a).unwrap().host_id=host;pass_top(&mut g);
    assert_eq!(g.regions[1].influence,[1,0]);assert_eq!(g.regions[3].influence,[0,0]);checkpoint(&g);
}

#[test]
fn death_observer_contract_one_point_enemy_repression_threshold_and_self_death_preserved() {
    let mut g=initial(2);contract(&mut g,2,1);let d=board(&mut g,"JZ31",2,2);let own=g.team(2);g.regions[1].influence[1-own]=2;die(&mut g,&d,RemovalCause::Destroy);
    finish(&mut g,1);assert_eq!(g.regions[1].influence[1-own],1);assert_eq!(g.regions[1].influence[own],0);assert_eq!(g.regions[2].influence[own],1); // original JZ31 self-death reward also resolved.
    let mut g=initial(2);contract(&mut g,2,1);let threshold=catalog::card(&g.regions[1].card.definition).threshold.unwrap();let own=g.team(2);g.regions[1].influence[own]=threshold-1;
    let d=board(&mut g,"JC125",2,4);die(&mut g,&d,RemovalCause::Destroy);choose(&mut g,vec!["accept".into()]);pass_top(&mut g);
    assert_eq!(g.regions[1].influence[own],threshold);assert!(matches!(g.window,Some(Window::Win(1,2))));
    checkpoint(&g);
}

#[test]
fn death_observer_same_card_same_color_instances_limit_independently_and_first_team_declares_first() {
    let mut g=initial(0);let a=board(&mut g,"JC031",0,2);let b=board(&mut g,"JC031",0,2);let enemy=board(&mut g,"JC031",2,3);
    for _ in 0..4 {held(&mut g,"LC19",1);}let d=board(&mut g,"JC125",0,0);die(&mut g,&d,RemovalCause::Destroy);finish(&mut g,1);
    assert_eq!(g.players[1].hand.len(),2);assert_eq!(g.turn_ability_usage.iter().filter(|u|u.source_instance==a||u.source_instance==b).count(),2);
    assert!(g.turn_ability_usage.iter().all(|u|u.source_instance!=enemy));
    let d=board(&mut g,"JC125",2,4);die(&mut g,&d,RemovalCause::Destroy);finish(&mut g,1);assert_eq!(g.players[1].hand.len(),1);
    let mut g=initial(0);g.first_team=1;for s in 0..4 {board(&mut g,"JC045",s,2);}let d=board(&mut g,"JC125",0,0);g.remove_dead(&d,RemovalCause::Destroy);
    assert_eq!(declarations(&g).iter().map(|d|d.actor).collect::<Vec<_>>(),[2,3,0,1]);checkpoint(&g);
}

#[test]
fn death_observer_both_discard_branches_preserve_holder_event_and_owner_grave() {
    for vampire in [false,true] {let mut g=initial(0);board(&mut g,"JC031",0,2);let dead=board(&mut g,if vampire{"JC031"}else{"JC125"},0,0);
        let mut hand=g.make_card("XQ34",2);let old=hand.id.clone();hand.controller=1;g.players[1].hand.push(hand);fund(&mut g,1,"JC125",1);
        die(&mut g,&dead,RemovalCause::Destroy);choose(&mut g,vec!["p1".into()]);pass_top(&mut g);if !vampire {choose(&mut g,vec![old.clone()]);}
        let declaration=pending(&g);assert_eq!(declaration.ability.event,Some(Event::HandDiscard));assert_eq!(declaration.actor,1);
        assert_eq!((declaration.source.card.owner,declaration.source.card.controller),(2,1));assert_eq!(declaration.source.card.id,old);
        assert!(g.players[2].graveyard.iter().any(|c|c.definition=="XQ34"&&c.id!=old));choose(&mut g,vec![]);checkpoint(&g);
    }
}

#[test]
fn death_observer_contract_threshold_runs_existing_win_window_and_team_victory() {
    let mut g=initial(2);contract(&mut g,2,1);let region_def=g.regions[1].card.definition.clone();let points=catalog::card(&region_def).points.unwrap();let own=g.team(2);
    for _ in 0..(g.win_score()-1)/points {let c=g.make_card(&region_def,2);g.players[2].score_cards.push(c);}
    g.regions[1].influence[own]=catalog::card(&region_def).threshold.unwrap()-1;
    let d=board(&mut g,"JC125",2,4);die(&mut g,&d,RemovalCause::Destroy);choose(&mut g,vec!["accept".into()]);pass_top(&mut g);
    assert!(matches!(g.window,Some(Window::Win(1,2))));fixture("contractWinWindow",&g);
    for _ in 0..12 {if g.status=="finished"{break;}let seat=(0..4).find(|&s|g.legal_actions(s).iter().any(|a|a.action.kind=="pass")).unwrap();apply(&mut g,seat,Action::new("pass"));}
    assert_eq!(g.status,"finished");assert_eq!(g.winner_team,Some(own));checkpoint(&g);
}

#[test]
fn death_observer_definition_transplants_nested_modes_and_program_mutations_rejected() {
    validate_definitions(definitions()).unwrap();
    for id in ["JC045","JC031","JC090"] {let trigger=definition(id).abilities.iter().find(|a|a.event==Some(Event::CharacterDeathObserved)).unwrap().clone();
        for dest in ["JC125","JC045","JC031","JC090"] {if dest==id{continue;}
            let mut r=definitions().clone();r.entry(dest.into()).or_default().abilities.push(trigger.clone());assert!(validate_definitions(&r).is_err());
            let mut r=definitions().clone();let mut a=trigger.clone();a.event=Some(Event::Death);a.ops=vec![Op::ForEachLivingPlayer(a.ops)];r.entry(dest.into()).or_default().abilities.push(a);assert!(validate_definitions(&r).is_err());
        }
        for mode in 0..8 {let mut r=definitions().clone();let d=r.get_mut(id).unwrap();let a=d.abilities.iter_mut().find(|a|a.event==Some(Event::CharacterDeathObserved)).unwrap();
            match mode {0=>a.event=Some(Event::Death),1=>a.per_turn_limit=None,2=>a.requires_ready_source=true,3=>a.costs.push(Cost::Assets(1)),4=>a.targets.push(a.targets.first().cloned().unwrap_or(definition("JC031").abilities[0].targets[0].clone())),5=>a.ops.push(Op::Draw{player:PlayerRef::Actor,count:1,end:DeckEnd::Top}),6=>a.modes.push(Mode{key:"alias".into(),label:"alias".into(),targets:vec![],ops:a.ops.clone()}),_=>d.modifiers.push(StaticModifier::PeekOwnDeckTop)}
            // Removing an absent limit changes nothing on JC045/JC090.
            if mode==1&&id!="JC031"{continue;}assert!(validate_definitions(&r).is_err(),"{id} mutation {mode}");
        }
    }
}

#[test]
fn death_observer_persisted_declarations_atomic_guards_and_discard_cursor_are_closed() {
    for id in ["JC045","JC031","JC090"] {let mut g=initial(0);if id=="JC090"{contract(&mut g,0,1);}else{board(&mut g,id,0,2);}
        let d=board(&mut g,"JC125",0,0);held(&mut g,"LC19",1);die(&mut g,&d,RemovalCause::Destroy);let v=serde_json::to_value(&g).unwrap();
        for mode in 0..6 {let mut bad=v.clone();let d=&mut bad["pending"]["resolution"]["Declare"]["declaration"];
            match mode{0=>d["actor"]=3.into(),1=>d["source"]["observed_death"]=serde_json::Value::Null,2=>d["source"]["observed_death"]["card"]["face_down"]=true.into(),3=>d["source"]["card"]["definition"]="JC125".into(),4=>d["ability"]["event"]="Death".into(),_=>d["source"]["play_source"]="Hand".into()};bad_state(&g,bad);}
        choose(&mut g,vec![if id=="JC031"{"p1"}else{"accept"}.into()]);checkpoint(&g);let v=serde_json::to_value(&g).unwrap();
        for mode in 0..7 {let mut bad=v.clone();let f=&mut bad["stack"][0]["frame"];
            match mode{0=>f["guard"]="Accepted".into(),1=>f["cursor"]=1.into(),2=>{f["guard"]="Accepted".into();f["cursor"]=1.into();},3=>f["steps"][0]["context"]=3.into(),4=>f["steps"][0]["op"]="JC045AddOneTimeToOriginalSource".into(),5=>f["ability_key"]="attach".into(),_=>f["chosen_region"]=2.into()};
            if mode==4&&id=="JC045"{continue;}bad_state(&g,bad);}
        if id=="JC031" {pass_top(&mut g);fixture("casterDiscardGuard",&g);let v=serde_json::to_value(&g).unwrap();
            for mode in 0..7 {let mut bad=v.clone();let p=&mut bad["pending"];
                match mode{0=>p["resolution"]["Frame"]["frame"]["cursor"]=0.into(),1=>p["resolution"]["Frame"]["frame"]["guard"]="Unchecked".into(),2=>p["resolution"]["Frame"]["frame"]["source"]["observed_death"]["vampire"]=true.into(),3=>p["resolution"]["Frame"]["choice"]["Discard"]["redraw"]=true.into(),4=>p["choice"]["min"]=0.into(),5=>p["seat"]=2.into(),_=>p["choice"]["options"][0]["id"]="fake".into()};bad_state(&g,bad);}
            let selected=g.pending.as_ref().unwrap().choice.options[0].id.clone();choose(&mut g,vec![selected]);assert!(g.players[1].hand.is_empty());
        }
    }
}

#[test]
fn death_observer_duplicate_command_is_exact_and_illegal_markers_rejected() {
    let mut g=initial(0);board(&mut g,"JC031",0,2);let dead=board(&mut g,"JC125",0,0);held(&mut g,"LC19",1);die(&mut g,&dead,RemovalCause::Destroy);
    let r=envelope(&g);let command=RoomCommand{command_id:"death-observer-duplicate".into(),expected_version:r.revision,
        action:SessionAction::Game{action:Action{choice_id:Some(g.pending.as_ref().unwrap().choice.id.clone()),selected:Some(vec!["p1".into()]),..Action::new("choose")}}};
    let result=r.transition(0,Some(command.clone()),0).unwrap();assert!(result.error_code.is_none());let next=RoomEnvelope::from_persisted(&result.state).unwrap();
    let duplicate=next.transition(0,Some(command),0).unwrap();assert_eq!(duplicate.state,result.state);assert_eq!(duplicate.outcome,"rejected");assert_eq!(duplicate.error_code.as_deref(),Some("version_conflict"));
    let mut v=serde_json::to_value(&g).unwrap();v["regions"][2]["cards"][0]["time_markers"]=1.into();bad_state(&g,v);
    let h=held(&mut g,"JC045",0);let mut v=serde_json::to_value(&g).unwrap();let i=g.players[0].hand.iter().position(|c|c.id==h).unwrap();v["players"][0]["hand"][i]["time_markers"]=1.into();bad_state(&g,v);
    let mut v=serde_json::to_value(&g).unwrap();v["world"][0]["time_markers"]=1.into();bad_state(&g,v);
    let mut v=serde_json::to_value(&g).unwrap();v["regions"][0]["card"]["time_markers"]=1.into();bad_state(&g,v);
}
