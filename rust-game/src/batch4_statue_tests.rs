use crate::{batch4_statue as statue,catalog,model::*,rules::*,unit_support::*};
fn pending(g:&mut Game){for _ in 0..128{if g.pending.is_some(){restore(g);return;}if !g.effects.is_empty(){g.drive().unwrap();}else{pass(g);}}panic!("bounded Statue choice");}
fn ritual(g:&mut Game)->String{g.begin_window(Window::Action(0));fund(g,0,"JZ38",2);let id=held(g,"JZ38",0);act(g,0,Action{card_id:Some(id),target_id:Some("society:p0".into()),..Action::new("play")});top(g);g.attachments.iter().rev().find(|a|a.card.definition=="JZ38").unwrap().card.id.clone()}
fn setup(id:&str)->(Game,String,String){let mut g=game();g.players[0].deck=(0..8).map(|_|g.make_card("JC125",0)).collect();let host=field(&mut g,id,0);let r=ritual(&mut g);(g,host,r)}
fn kill(g:&mut Game,host:&str){g.begin_window(Window::Action(g.team(2)));fund(g,2,"JC104",3);let id=held(g,"JC102",2);act(g,2,Action{card_id:Some(id),target_id:Some(host.into()),..Action::new("play")});pending(g);}
fn select_source(g:&mut Game,id:&str,selection:Vec<String>){if g.pending.as_ref().is_some_and(|p|p.choice.kind=="batch3-entry-order"){let key=g.pending.as_ref().unwrap().choice.options.iter().find(|o|o.id.starts_with(&format!("{id}:"))).unwrap().id.clone();choose(g,vec![key]);}choose(g,selection);}
fn accept_all(g:&mut Game){for _ in 0..64{let Some(p)=g.pending.clone()else{return;};if p.choice.kind=="batch3-entry-order"{choose(g,vec![p.choice.options[0].id.clone()]);}else if matches!(p.resolution,ChoiceResolution::Declare{..}){choose(g,vec!["accept".into()]);}else{return;}}panic!("bounded death declarations");}
fn drain(g:&mut Game){for _ in 0..128{if g.pending.is_some(){resolve_choice(g);}else if !g.effects.is_empty(){g.drive().unwrap();restore(g);}else if !g.stack.is_empty(){pass(g);}else{return;}}panic!("bounded death resolution");}
fn bad(g:&Game){assert!(Game::from_persisted(&serde_json::to_string(g).unwrap()).is_err());}

#[test]
fn batch4_statue_exact_personal_society_attachment_and_current_death_registry_are_closed(){
    let c=catalog::card("JZ38");assert_eq!((c.cost,c.magic.as_str(),c.supported),(2,"心灵",false));assert_eq!(c.loyalty,["红色"]);
    let d=statue::definition();assert_eq!(d.attachment.as_ref().unwrap().host.zone,Zone::Society);assert_eq!(d.abilities[1].event,Some(Event::JZ38HostDied));
    let mut wrong=d.clone();wrong.abilities[1].event=Some(Event::CharacterDeathObserved);assert!(crate::batch4::validate_definition("JZ38",&wrong).is_err());assert!(crate::rules::validate_ability("JC125",&d.abilities[1]).is_err());
    let mut count=0;for(id,d)in crate::rules::definitions(){let deaths=d.abilities.iter().filter(|a|a.event==Some(Event::Death)).collect::<Vec<_>>();if deaths.is_empty(){continue;}count+=1;println!("current-death {id} keys={:?} costs={} modes={} targets={} ops={:?}",deaths.iter().map(|a|&a.key).collect::<Vec<_>>(),deaths[0].costs.len(),deaths[0].modes.len(),deaths[0].targets.len(),deaths[0].ops);assert_eq!(deaths.len(),1);assert!(deaths[0].costs.is_empty()&&deaths[0].modes.is_empty()&&deaths[0].targets.len()<=1);}assert!(count>=7);
}

#[test]
fn batch4_statue_actual_host_death_wounds_two_independently_chosen_targets_with_distinct_frames(){
    let(mut g,host,r)=setup("JZ59");let a=field(&mut g,"JC032",2);let b=field(&mut g,"JC032",2);kill(&mut g,&host);
    select_source(&mut g,&host,vec![a.clone()]);select_source(&mut g,&r,vec!["accept".into()]);pending(&mut g);
    let p=g.pending.as_ref().unwrap();assert!(matches!(p.resolution,ChoiceResolution::JZ38EchoTarget{..}));assert_eq!(p.choice.min,Some(1));choose(&mut g,vec![b.clone()]);
    assert_eq!(g.stack.len(),2);let frames=g.stack.iter().map(|s|s.frame.as_ref().unwrap()).collect::<Vec<_>>();assert_eq!(frames[0].source.card.id,host);assert_eq!(frames[1].source.card.id,host);assert!(frames[0].source.death_echo.is_none()&&frames[1].source.death_echo.is_some());assert_ne!(frames[0].frame_id,frames[1].frame_id);
    drain(&mut g);assert_eq!(g.board(&a).unwrap().1.wounds,1);assert_eq!(g.board(&b).unwrap().1.wounds,1);restore(&mut g);
}

#[test]
fn batch4_statue_host_is_a_subtype_and_ally_nonhost_or_hidden_deaths_do_not_observe(){
    for(mode,id)in [(0,"JZ31"),(1,"JC125"),(2,"JZ31")]{let(mut g,host,_)=setup(id);if mode==0{g.board_mut(&host).unwrap().controller=1;}if mode==2{g.board_mut(&host).unwrap().face_down=true;}
        g.remove_dead(&host,RemovalCause::Destroy);g.drive().unwrap();
        assert!(!g.effects.iter().any(|e|matches!(e,Effect::Declare{declaration:d}if d.source.card.definition=="JZ38")));
        assert!(!g.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Declare{declaration:d,..}if d.source.card.definition=="JZ38")));restore(&mut g);
    }
}

#[test]
fn batch4_statue_two_rituals_each_add_once_and_never_repeat_each_others_echo(){
    let(mut g,host,_)=setup("JZ31");ritual(&mut g);kill(&mut g,&host);accept_all(&mut g);assert_eq!(g.stack.len(),3);drain(&mut g);assert_eq!(g.regions[0].influence[0],3);restore(&mut g);
}

#[test]
fn batch4_statue_wm035_grants_host_category_but_its_separate_observer_draws_only_once(){
    let(mut g,host,_)=setup("XQ17");field(&mut g,"WM035",0);assert!(g.current_subtypes(g.board(&host).unwrap().1).iter().any(|s|s=="宿主"));let deck=g.players[0].deck.len();
    kill(&mut g,&host);accept_all(&mut g);drain(&mut g);assert_eq!(g.players[0].deck.len(),deck-3);assert_eq!(g.players[0].hand.len(),1);assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JC125").count(),2);restore(&mut g);
}

#[test]
fn batch4_statue_real_death_search_and_extra_search_each_resume_privately_once(){
    let(mut g,host,_)=setup("JZ61");let nightmare=catalog::catalog().cards.iter().find(|c|c.name=="噩梦残像").unwrap().id.clone();g.players[0].deck=(0..2).map(|_|g.make_card(&nightmare,0)).collect();
    kill(&mut g,&host);accept_all(&mut g);let mut visits=0;
    for _ in 0..128{if let Some(p)=g.pending.clone(){assert_eq!(p.choice.kind,"search");visits+=1;let ids=p.choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>();for seat in [1,2,3]{let view=serde_json::to_string(&g.view(seat)).unwrap();for id in &ids{assert!(!view.contains(id));}}choose(&mut g,vec![ids[0].clone()]);}else if !g.stack.is_empty(){pass(&mut g);}else{break;}}
    assert_eq!(visits,2);assert_eq!(g.players[0].hand.iter().filter(|c|c.definition==nightmare).count(),2);assert!(g.players[0].deck.is_empty());restore(&mut g);
}

#[test]
fn batch4_statue_frozen_time_forecast_keeps_echo_context_across_top_bottom_continuation(){
    let(mut g,host,_)=setup("JZ30");field(&mut g,"WM035",0);g.board_mut(&host).unwrap().time_markers=2;
    kill(&mut g,&host);accept_all(&mut g);let mut visits=0;
    for _ in 0..128{if let Some(p)=g.pending.clone(){assert_eq!(p.choice.kind,"investigation");assert_eq!(p.choice.options.len(),2);visits+=1;let ids=p.choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>();act(&mut g,p.seat,Action{choice_id:Some(p.choice.id),top:Some(vec![ids[0].clone()]),bottom:Some(vec![ids[1].clone()]),..Action::new("choose")});}else if !g.stack.is_empty(){pass(&mut g);}else{break;}}
    assert_eq!(visits,2);assert_eq!(g.players[0].deck.len(),7);assert_eq!(g.players[0].hand.len(),1);restore(&mut g); // WM035's single observer draw.
}

#[test]
fn batch4_statue_declined_original_does_not_disable_accepted_extra_and_departed_ritual_does_not_rebind(){
    let(mut g,host,r)=setup("JZ31");kill(&mut g,&host);select_source(&mut g,&host,vec![]);select_source(&mut g,&r,vec!["accept".into()]);g.remove_dead(&r,RemovalCause::Destroy);restore(&mut g);drain(&mut g);assert_eq!(g.regions[0].influence[0],1);restore(&mut g);
}

#[test]
fn batch4_statue_forged_observation_current_sources_unknown_card_and_nested_echo_are_rejected(){
    let(mut g,host,r)=setup("JZ31");kill(&mut g,&host);if g.pending.as_ref().unwrap().choice.kind=="batch3-entry-order"{let id=g.pending.as_ref().unwrap().choice.options.iter().find(|o|o.id.starts_with(&format!("{r}:"))).unwrap().id.clone();choose(&mut g,vec![id]);}
    for mode in 0..6{let mut wrong=g.clone();let ChoiceResolution::Declare{declaration:d,..}=&mut wrong.pending.as_mut().unwrap().resolution else{panic!()};let Op::JZ38RepeatDeath{observation:Some(o)}=&mut d.ability.ops[0]else{panic!()};
        match mode{0=>o.proof.subtypes.clear(),1=>o.dead.card.controller=1,2=>o.dead.card.definition="unknown".into(),3=>o.dead.death_echo=Some(Box::new(o.proof.clone())),4=>o.proof.ritual.host_id="society:p1".into(),_=>o.proof.turn+=1}bad(&wrong);
    }
    let mut wrong=g.clone();let i=wrong.attachments.iter().position(|a|a.card.id==r).unwrap();let c=wrong.attachments.remove(i).card;wrong.players[0].hand.push(c);bad(&wrong);
}

#[test]
fn batch4_statue_mandatory_target_and_echo_frame_cannot_duplicate_skip_or_pollute_the_program(){
    let(mut g,host,r)=setup("JZ59");let victim=field(&mut g,"JC032",2);kill(&mut g,&host);select_source(&mut g,&host,vec![]);select_source(&mut g,&r,vec!["accept".into()]);pending(&mut g);
    let p=g.pending.clone().unwrap();reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),..Action::new("choose")});reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec![victim.clone()]),option:Some("skip".into()),..Action::new("choose")});
    choose(&mut g,vec![victim]);let mut wrong=g.clone();wrong.stack.push(wrong.stack.last().unwrap().clone());bad(&wrong);
    let mut wrong=g.clone();let f=wrong.stack.last_mut().unwrap().frame.as_mut().unwrap();f.steps.clear();bad(&wrong);let mut wrong=g.clone();wrong.stack.last_mut().unwrap().frame.as_mut().unwrap().source.death_echo.as_mut().unwrap().ritual.card.controller=1;bad(&wrong);
}

#[test]
fn batch4_statue_actual_corpse_return_continuations_keep_original_actor_and_consume_each_card_once(){
    let(mut g,host,_)=setup("BQ078");field(&mut g,"WM035",0);for _ in 0..2{let c=g.make_card("XQ46",0);g.players[0].graveyard.push(c);}
    kill(&mut g,&host);accept_all(&mut g);let mut visits=0;
    for _ in 0..128{if let Some(p)=g.pending.clone(){assert_eq!(p.choice.kind,"bq078_corpse_return");visits+=1;let id=p.choice.options[0].id.clone();choose(&mut g,vec![id]);}else if !g.stack.is_empty(){pass(&mut g);}else{break;}}
    assert_eq!(visits,2);assert_eq!(g.players[0].hand.iter().filter(|c|c.definition=="XQ46").count(),2);assert!(!g.players[0].graveyard.iter().any(|c|c.definition=="XQ46"));restore(&mut g);
}

#[test]
fn batch4_statue_two_actual_simultaneous_deaths_keep_both_ritual_event_references_distinct(){
    let(mut g,host,r)=setup("JZ31");let second=field(&mut g,"JZ31",0);
    g.damage([(host,99),(second,99)].into_iter().collect()).unwrap();g.drive().unwrap();restore(&mut g);
    let options=&g.pending.as_ref().unwrap().choice.options;assert_eq!(options.len(),4);assert_eq!(options.iter().map(|o|&o.id).collect::<std::collections::BTreeSet<_>>().len(),4);assert_eq!(options.iter().filter(|o|o.id.starts_with(&format!("{r}:"))).count(),2);
    accept_all(&mut g);drain(&mut g);assert_eq!(g.regions[0].influence[0],4);restore(&mut g);
}

#[test]
fn batch4_statue_extra_private_search_rejects_wrong_seat_options_stage_and_repeated_choice(){
    let(mut g,host,r)=setup("JZ61");let nightmare=catalog::catalog().cards.iter().find(|c|c.name=="噩梦残像").unwrap().id.clone();g.players[0].deck=(0..2).map(|_|g.make_card(&nightmare,0)).collect();
    kill(&mut g,&host);select_source(&mut g,&host,vec!["accept".into()]);select_source(&mut g,&r,vec!["accept".into()]);pending(&mut g);
    let p=g.pending.clone().unwrap();let ChoiceResolution::Frame{frame:f,..}=&p.resolution else{panic!()};assert!(f.source.death_echo.is_some());
    for mode in 0..6{let mut wrong=g.clone();let p=wrong.pending.as_mut().unwrap();match mode{0=>p.seat=1,1=>{p.choice.options.pop();},2=>p.choice.min=Some(1),3=>p.choice.kind="discard".into(),4=>{let ChoiceResolution::Frame{frame:f,..}=&mut p.resolution else{panic!()};f.source.death_echo.as_mut().unwrap().ritual.card.controller=1;},_=>{let ChoiceResolution::Frame{frame:f,..}=&mut p.resolution else{panic!()};f.guard=GuardState::Unchecked;}}bad(&wrong);}
    let action=Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![p.choice.options[0].id.clone()]),..Action::new("choose")};
    reject(&mut g,0,Action{option:Some("skip".into()),..action.clone()});act(&mut g,0,action.clone());let random=g.random;reject(&mut g,0,action);assert_eq!(g.random,random);assert_eq!(g.players[0].hand.len(),1);
}

#[test]
fn batch4_statue_real_paid_attachment_destruction_response_does_not_cancel_captured_death(){
    let(mut g,host,r)=setup("JZ31");fund(&mut g,0,"JC005",2);let response=held(&mut g,"JC005",0);
    kill(&mut g,&host);select_source(&mut g,&host,vec![]);select_source(&mut g,&r,vec!["accept".into()]);
    let mut activation=None;for _ in 0..4{activation=g.legal_actions(0).into_iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_ref()==Some(&response)&&a.action.target_id.as_ref()==Some(&r)).map(|a|a.action);if activation.is_some(){break;}pass(&mut g);}
    assert!(g.stack.iter().any(|s|s.frame.as_ref().is_some_and(|f|f.source.card.id==r)));act(&mut g,0,activation.expect("actual paid fast JC005 response must be available"));top(&mut g);assert!(!g.attachments.iter().any(|a|a.card.id==r));drain(&mut g);assert_eq!(g.regions[0].influence[0],1);restore(&mut g);
}

#[test]
fn batch4_statue_simultaneous_wm035_departure_preserves_human_host_fact_and_observer_event_keys(){
    let(mut g,host,_)=setup("XQ17");let grant=field(&mut g,"WM035",0);
    g.damage([(host,99),(grant.clone(),99)].into_iter().collect()).unwrap();g.drive().unwrap();restore(&mut g);
    let options=&g.pending.as_ref().unwrap().choice.options;assert_eq!(options.len(),5);assert_eq!(options.iter().map(|o|&o.id).collect::<std::collections::BTreeSet<_>>().len(),5);assert_eq!(options.iter().filter(|o|o.id.starts_with(&format!("{grant}:"))).count(),2);
    accept_all(&mut g);drain(&mut g);assert_eq!(g.players[0].deck.len(),4);assert_eq!(g.players[0].hand.len(),2);restore(&mut g);
}
