use crate::{catalog, model::*, rules::*, unit_support::*};
const KEY:&str=crate::batch4_conference::KEY;
fn layout(secret:bool)->(Game,String){let mut g=game();fund(&mut g,0,"WM047",12);if secret{g.add_society_markers(0,0,1);}let id=held(&mut g,"WM047",0);act(&mut g,0,Action{card_id:Some(id),target_id:Some("society:p0".into()),option:secret.then(||"secret-plan".into()),..Action::new("play")});top(&mut g);let plan=g.attachments[0].card.id.clone();(g,plan)}
fn pending(g:&mut Game){for _ in 0..96{if g.pending.is_some(){restore(g);return;}if !g.effects.is_empty(){g.drive().unwrap();restore(g);}else{pass(g);}}panic!("bounded conference trigger");}
fn deploy(g:&mut Game,id:&str,seat:usize,r:usize){let d=catalog::card(id);let cost=d.cost as usize;fund(g,seat,id,cost.max(d.loyalty.len()).max(1));let id=held(g,id,seat);act(g,seat,Action{card_id:Some(id),region:Some(r),..Action::new("deploy")});}
fn marks(g:&Game,id:&str)->u32{g.attachments.iter().find(|a|a.card.id==id).unwrap().card.advancement_markers}
fn accept(g:&mut Game,plan:&str){pending(g);assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id==plan));choose(g,vec![plan.into()]);top(g);}
fn drain(g:&mut Game,plan:&str)->u32{let mut maximum=0;for _ in 0..384{
    if let Some(c)=g.attachments.iter().find(|a|a.card.id==plan){maximum=maximum.max(c.card.advancement_markers);}
    if let Some(p)=g.pending.clone(){let selected=if p.choice.kind=="batch3-entry-order"{vec![p.choice.options[0].id.clone()]}else if p.choice.options.iter().any(|o|o.id==plan){vec![plan.into()]}else{vec![]};choose(g,selected);}
    else if !g.stack.is_empty(){pass(g);}else if !g.effects.is_empty(){g.drive().unwrap();restore(g);}else{return maximum;}
}panic!("bounded entry/order/response chain");}
#[test]
fn batch4_conference_exact_print_and_closed_registry() {
    let c = catalog::card("XG17"); assert_eq!((c.cost,c.defense,c.supported,c.unique),(4,Some(2),false,false));
    assert_eq!(c.loyalty,["灰色","灰色"]);assert!(definition("XG17").traits.public);assert!(!definition("XG17").traits.renown);
    let mut changed=crate::rules::definitions().clone();changed.get_mut("XG17").unwrap().abilities[0].targets[0].relation=Relation::FriendlyTeam;assert!(crate::rules::validate_definitions(&changed).is_err());
}

#[test]
fn batch4_conference_public_sun_renown_and_temporary_icons_use_personal_condition(){
    let mut g=game();let id=field(&mut g,"XG17",0);let c=g.board(&id).unwrap().1;assert!(g.current_public(c,0));assert!(!g.has_renown(c));assert_eq!(g.icons(c,0),Icons{investigation:1,influence:2,..Icons::default()});
    g.add_society_markers(1,1,0);assert!(!g.has_renown(g.board(&id).unwrap().1));g.add_society_markers(0,1,0);assert!(g.has_renown(g.board(&id).unwrap().1));
    for seat in 0..4{assert_eq!(g.view(seat).regions[0].characters.iter().find(|c|c.instance_id==id).unwrap().current_renown,Some(true));}
    g.first_team=1;assert_eq!(g.icons(g.board(&id).unwrap().1,0).influence,1);g.add_society_markers(0,0,1);assert!(!g.has_renown(g.board(&id).unwrap().1));restore(&mut g);
}

#[test]
fn batch4_conference_defense_is_global_other_personal_renown_and_stacks_per_source(){
    let mut g=game();let first=field(&mut g,"XG17",0);let own=field(&mut g,"JC070",0);let ally=field(&mut g,"JC070",1);let plain=field(&mut g,"JC125",0);
    let remote=g.make_card("JC070",0);let far=remote.id.clone();g.regions[2].cards.push(remote);
    let base=catalog::card("JC070").defense.unwrap();assert_eq!(g.defense(g.board(&own).unwrap().1,0),base+1);assert_eq!(g.defense(g.board(&far).unwrap().1,2),base+1);assert_eq!(g.defense(g.board(&ally).unwrap().1,0),base);assert_eq!(g.defense(g.board(&plain).unwrap().1,0),catalog::card("JC125").defense.unwrap());assert_eq!(g.defense(g.board(&first).unwrap().1,0),2);
    let second=field(&mut g,"XG17",0);g.add_society_markers(0,1,0);assert_eq!(g.defense(g.board(&first).unwrap().1,0),3);assert_eq!(g.defense(g.board(&second).unwrap().1,0),3);assert_eq!(g.defense(g.board(&own).unwrap().1,0),base+2);
    g.hide_in_place(&second);assert_eq!(g.defense(g.board(&own).unwrap().1,0),base+1);g.remove_dead(&first,RemovalCause::Destroy);assert_eq!(g.defense(g.board(&own).unwrap().1,0),base);restore(&mut g);
}

#[test]
fn batch4_conference_real_self_entry_requires_renown_at_entry_and_advance_has_no_extra_payment(){
    for exposed in [false,true]{let(mut g,plan)=layout(false);if exposed{g.add_society_markers(0,1,0);}let before=g.resources(0);deploy(&mut g,"XG17",0,0);let after=g.resources(0);assert_eq!(after,before);
        // deploy helper adds four real assets, then the actual play pays four.
        if exposed{accept(&mut g,&plan);assert_eq!(marks(&g,&plan),1);}else{top(&mut g);assert_eq!(marks(&g,&plan),0);assert!(g.pending.is_none());}
        assert_eq!(g.resources(0),after);restore(&mut g);
    }
}

#[test]
fn batch4_conference_real_printed_renown_entry_can_advance_secret_plan_and_decline(){
    for decline in [false,true]{let(mut g,plan)=layout(true);let observer=field(&mut g,"XG17",0);let (_,c)=g.remove_board(&observer).unwrap();g.regions[2].cards.push(c);deploy(&mut g,"JC070",0,0);pending(&mut g);let p=g.pending.clone().unwrap();assert_eq!(p.seat,0);assert_eq!(p.choice.min,Some(0));assert!(p.choice.options.iter().any(|o|o.id==plan));
        for seat in 1..4{assert!(g.view(seat).pending_choice.is_none());}
        choose(&mut g,if decline{vec![]}else{vec![plan.clone()]});if !decline{for seat in 1..4{let raw=serde_json::to_string(&g.view(seat)).unwrap();assert!(!raw.contains("贝奥武夫")&&!raw.contains("WM047"));}top(&mut g);}
        assert_eq!(marks(&g,&plan),u32::from(!decline));assert!(g.attachments.iter().find(|a|a.card.id==plan).unwrap().card.face_down);restore(&mut g);
    }
}

#[test]
fn batch4_conference_teammate_or_enemy_renown_entry_never_uses_another_controllers_plan(){
    for actor in [1,2]{let(mut g,plan)=layout(false);field(&mut g,"XG17",0);if actor==2{g.begin_window(Window::Action(1));}let home=if actor%2==0{0}else{g.regions.len()-1};deploy(&mut g,"JC070",actor,home);top(&mut g);assert_eq!(marks(&g,&plan),0);assert!(g.pending.is_none());restore(&mut g);}
}

#[test]
fn batch4_conference_moon_loss_source_leave_and_renown_loss_after_entry_do_not_requalify(){
    let(mut g,plan)=layout(true);g.add_society_markers(0,2,0);deploy(&mut g,"XG17",0,0);pending(&mut g);let source=g.regions[0].cards.iter().find(|c|c.definition=="XG17").unwrap().id.clone();
    g.add_society_markers(0,0,2);g.remove_dead(&source,RemovalCause::Destroy);let replacement=field(&mut g,"XG17",0);restore(&mut g);choose(&mut g,vec![plan.clone()]);top(&mut g);assert_eq!(marks(&g,&plan),1);assert!(!g.has_renown(g.board(&replacement).unwrap().1));restore(&mut g);
}

#[test]
fn batch4_conference_renown_gained_only_after_entry_does_not_retroactively_trigger(){
    let(mut g,plan)=layout(false);deploy(&mut g,"XG17",0,0);top(&mut g);let source=g.regions[0].cards.iter().find(|c|c.definition=="XG17").unwrap().id.clone();g.add_society_markers(0,1,0);g.drive().unwrap();assert!(g.has_renown(g.board(&source).unwrap().1));assert_eq!(marks(&g,&plan),0);assert!(g.pending.is_none()&&g.stack.is_empty());restore(&mut g);
}

#[test]
fn batch4_conference_enemy_alpha_denies_even_printed_renown_at_actual_entry(){
    let(mut g,plan)=layout(false);field(&mut g,"XG17",0);let alpha=field(&mut g,"WM041",2);deploy(&mut g,"JC070",0,0);top(&mut g);assert_eq!(marks(&g,&plan),0);g.remove_dead(&alpha,RemovalCause::Destroy);g.drive().unwrap();assert_eq!(marks(&g,&plan),0);assert!(g.pending.is_none());restore(&mut g);
}

#[test]
fn batch4_conference_target_loss_cancels_original_frame_without_advancing_replacement(){
    let(mut g,plan)=layout(false);field(&mut g,"XG17",0);deploy(&mut g,"JC070",0,0);pending(&mut g);choose(&mut g,vec![plan.clone()]);let resources=g.resources(0);g.remove_dead(&plan,RemovalCause::Destroy);let c=g.make_card("WM047",0);let replacement=c.id.clone();g.attachments.push(Attachment{card:c,host_id:"society:p0".into()});top(&mut g);assert_eq!(marks(&g,&replacement),0);assert_eq!(g.resources(0),resources);restore(&mut g);
}

#[test]
fn batch4_conference_multiple_sources_order_once_and_same_controller_entries_keep_disjoint_keys(){
    let(mut g,plan)=layout(false);field(&mut g,"XG17",0);field(&mut g,"XG17",0);
    // Explicit native entry primitives exercise a queue containing two entries
    // before it drains; they are not described as a browser command chain.
    for _ in 0..2{let id=field(&mut g,"JC070",0);g.enter_triggers(0,"JC070",&id,false);}
    assert_eq!(g.entry_order_groups.len(),2);let all=g.entry_order_groups.iter().flat_map(|g|g.remaining.iter()).collect::<std::collections::BTreeSet<_>>();assert_eq!(all.len(),4);restore(&mut g);drain(&mut g,&plan);assert_eq!(marks(&g,&plan),4);assert!(g.entry_order_groups.is_empty());restore(&mut g);
}

#[test]
fn batch4_conference_frozen_alpha_and_destroyed_region_qualifications_survive_condition_loss(){
    for alpha_case in [false,true]{let(mut g,plan)=layout(false);field(&mut g,"XG17",0);let alpha=if alpha_case{Some(field(&mut g,"WM041",0))}else{g.regions[0].destroyed=true;None};let id=field(&mut g,if alpha_case{"XQ46"}else{"JZ32"},0);g.enter_triggers(0,if alpha_case{"XQ46"}else{"JZ32"},&id,false);pending(&mut g);
        if let Some(alpha)=alpha{g.remove_dead(&alpha,RemovalCause::Destroy);}else{g.regions[0].destroyed=false;}assert!(!g.has_renown(g.board(&id).unwrap().1));restore(&mut g);choose(&mut g,vec![plan.clone()]);top(&mut g);assert_eq!(marks(&g,&plan),1);restore(&mut g);
    }
}

#[test]
fn batch4_conference_declaration_receipts_private_actor_and_duplicate_frames_are_closed(){
    let(mut g,plan)=layout(false);field(&mut g,"XG17",0);deploy(&mut g,"JC070",0,0);pending(&mut g);
    let a=Action{choice_id:Some(g.pending.as_ref().unwrap().choice.id.clone()),selected:Some(vec![plan.clone()]),..Action::new("choose")};reject(&mut g,1,a.clone());let mut extra=a.clone();extra.cost_selected=Some(vec![]);reject(&mut g,0,extra);
    for change in 0..6{let mut bad=g.clone();let p=bad.pending.as_mut().unwrap();let ChoiceResolution::Declare{declaration:d,..}=&mut p.resolution else{panic!("expected target declaration")};let Op::XG17AdvancePlan{observation:Some(o)}=&mut d.ability.ops[0]else{panic!("expected real receipt")};match change{0=>o.turn+=1,1=>o.entered.card.controller=1,2=>o.event_id=o.entered.card.id.clone(),3=>o.basis=crate::batch4_conference::RenownBasis::Exposure{count:1},4=>o.entered.card.face_down=true,_=>d.ability.targets[0].relation=Relation::FriendlyTeam};assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());}
    act(&mut g,0,a.clone());reject(&mut g,0,a);let mut bad=g.clone();bad.stack.push(bad.stack.last().unwrap().clone());assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut bad=g.clone();bad.stack.last_mut().unwrap().frame.as_mut().unwrap().already_paid.push(PaidCost::Assets(vec!["i1".into()]));assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());top(&mut g);assert_eq!(marks(&g,&plan),1);
}

#[test]
fn batch4_conference_multiple_observers_can_add_markers_after_threshold_without_duplicate_completion(){
    let mut g=game();fund(&mut g,0,"WM046",12);let hand=held(&mut g,"WM046",0);act(&mut g,0,Action{card_id:Some(hand),target_id:Some("society:p0".into()),..Action::new("play")});top(&mut g);let plan=g.attachments[0].card.id.clone();
    for _ in 0..4{field(&mut g,"XG17",0);}let locked=field(&mut g,"JC125",2);g.board_mut(&locked).unwrap().lock_markers=1;let hand=g.players[0].hand.len();deploy(&mut g,"JC070",0,0);
    let maximum=drain(&mut g,&plan);assert_eq!(maximum,2,"Once completion resolves, remaining observers cannot retarget the departed plan");assert!(g.attachments.iter().all(|a|a.card.id!=plan));assert_eq!(g.players[0].hand.len(),hand+1);assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="WM046").count(),1);restore(&mut g);
}

#[test]
fn batch4_conference_only_personal_live_plans_are_targets_and_no_plan_does_not_pause_entry(){
    let(mut g,own)=layout(false);field(&mut g,"XG17",0);
    for seat in [1,2]{let c=g.make_card("WM047",seat);g.attachments.push(Attachment{card:c,host_id:format!("society:p{seat}")});}
    deploy(&mut g,"JC070",0,0);pending(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.options.iter().map(|o|o.id.as_str()).collect::<Vec<_>>(),vec![own.as_str()]);choose(&mut g,vec![]);
    g.remove_dead(&own,RemovalCause::Destroy);deploy(&mut g,"JC070",0,0);top(&mut g);assert!(g.pending.is_none());restore(&mut g);
}

#[test]
fn batch4_conference_defense_loss_rechecks_actual_damage_on_renown_recipient(){
    let mut g=game();let source=field(&mut g,"XG17",0);let monk=field(&mut g,"JC070",0);let base=catalog::card("JC070").defense.unwrap();
    g.damage([(monk.clone(),base)].into_iter().collect()).unwrap();assert!(g.board(&monk).is_some());g.remove_dead(&source,RemovalCause::Destroy);g.settle_deaths();assert!(g.board(&monk).is_none());restore(&mut g);
}

#[test]
fn batch4_conference_entry_while_completion_waits_adds_one_marker_but_never_redeclares_completion(){
    let mut g=game();fund(&mut g,0,"WM046",12);let hand=held(&mut g,"WM046",0);act(&mut g,0,Action{card_id:Some(hand),target_id:Some("society:p0".into()),..Action::new("play")});top(&mut g);let plan=g.attachments[0].card.id.clone();field(&mut g,"XG17",0);
    deploy(&mut g,"JC070",0,0);accept(&mut g,&plan);assert_eq!(marks(&g,&plan),1);deploy(&mut g,"JC070",0,0);pending(&mut g);choose(&mut g,vec![plan.clone()]);
    for _ in 0..64{if g.stack.last().and_then(|i|i.frame.as_ref()).is_some_and(|f|f.ability_key=="batch4-plan-complete"){break;}pass(&mut g);}
    assert_eq!(marks(&g,&plan),2);assert_eq!(g.stack.len(),1);restore(&mut g);
    // Disclosed native entry primitive represents another entry during the
    // open completion response. It is not a fabricated public command.
    let incoming=field(&mut g,"JC070",0);g.enter_triggers(0,"JC070",&incoming,false);g.drive().unwrap();restore(&mut g);choose(&mut g,vec![plan.clone()]);top(&mut g);
    assert_eq!(marks(&g,&plan),3);assert_eq!(g.stack.iter().filter(|i|i.frame.as_ref().is_some_and(|f|f.ability_key=="batch4-plan-complete")).count(),1);restore(&mut g);top(&mut g);assert!(g.attachments.iter().all(|a|a.card.id!=plan));assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="WM046").count(),1);
}

#[test]
fn batch4_conference_real_attached_character_keeps_renown_aura_and_is_not_a_second_entry(){
    let(mut g,plan)=layout(false);field(&mut g,"XG17",0);let alpha=field(&mut g,"WM041",0);let host=field(&mut g,"JC029",0);deploy(&mut g,"BQ026",0,0);
    for _ in 0..256{
        if let Some(p)=g.pending.clone(){let selected=match &p.resolution{
            ChoiceResolution::Batch3EntryOrder{..}=>vec![p.choice.options[0].id.clone()],
            ChoiceResolution::Declare{declaration:d,..}if d.source.card.definition=="BQ026"=>vec![host.clone()],
            ChoiceResolution::Declare{declaration:d,..}if d.source.card.definition=="XG17"=>vec![plan.clone()],
            _=>vec![],
        };choose(&mut g,selected);}else if !g.stack.is_empty(){pass(&mut g);}else if !g.effects.is_empty(){g.drive().unwrap();restore(&mut g);}else{break;}
    }
    assert!(g.pending.is_none()&&g.stack.is_empty()&&g.effects.is_empty());let bat=g.attachments.iter().find(|a|a.card.definition=="BQ026").unwrap().card.id.clone();
    assert_eq!(marks(&g,&plan),1);assert!(g.has_renown(g.board(&bat).unwrap().1));assert_eq!(g.defense(g.board(&bat).unwrap().1,0),2);
    for seat in 0..4{let v=g.view(seat);let c=&v.attachments.iter().find(|a|a.card.instance_id==bat).unwrap().card;assert_eq!(c.current_renown,Some(true));assert_eq!(c.defense,Some(2));}
    g.remove_dead(&alpha,RemovalCause::Destroy);assert!(!g.has_renown(g.board(&bat).unwrap().1));assert_eq!(g.defense(g.board(&bat).unwrap().1,0),1);restore(&mut g);
}
