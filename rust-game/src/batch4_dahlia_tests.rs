use crate::{batch4_dahlia as dahlia,catalog,model::*,rules::*,unit_support::*};
fn pending(g:&mut Game){for _ in 0..96{if g.pending.is_some(){restore(g);return;}if !g.effects.is_empty(){g.drive().unwrap();}else{pass(g);}}panic!("bounded Dahlia choice");}
fn crime(g:&mut Game,actor:usize){g.begin_window(Window::Action(g.team(actor)));let cost=catalog::card("JZ24").cost as usize;fund(g,actor,"JZ24",cost.max(1));let id=held(g,"JZ24",actor);let r=(0..g.regions.len()).find(|r|g.accessible(actor,*r)).unwrap();act(g,actor,Action{card_id:Some(id),region:Some(r),..Action::new("deploy")});}
fn trigger()->(Game,String){let mut g=game();let source=field(&mut g,"WM060",0);crime(&mut g,0);pending(&mut g);let ChoiceResolution::Declare{declaration:d,..}=&g.pending.as_ref().unwrap().resolution else{panic!()};assert_eq!(d.ability.key,dahlia::ENTRY);choose(&mut g,vec!["accept".into()]);(g,source)}
fn draw_discard(g:&mut Game){pending(g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"discard");let id=g.pending.as_ref().unwrap().choice.options[0].id.clone();choose(g,vec![id]);}

#[test]
fn batch4_dahlia_exact_gold_print_sun_public_and_finite_draw_discard_repress_program_are_closed(){
    let c=catalog::card("WM060");assert_eq!((c.cost,c.unique,c.supported,c.defense),(5,true,false,Some(2)));assert_eq!(c.subtitle.as_deref(),Some("夜之花"));assert_eq!(c.loyalty,["黑色"]);assert_eq!(c.permanent_icons.influence,2);assert_eq!(c.temporary_icons.investigation,2);
    let d=dahlia::definition();assert!(!d.traits.public);assert_eq!(d.abilities[1].ops.len(),4);
    for mode in 0..4{let mut bad=d.clone();match mode{0=>bad.traits.public=true,1=>{bad.abilities[1].ops.remove(2);},2=>bad.abilities[1].event=Some(Event::Enter),_=>bad.modifiers.clear()};assert!(crate::batch4::validate_definition("WM060",&bad).is_err());}
    let mut a=d.abilities[1].clone();a.key="foreign".into();assert!(crate::rules::validate_ability("JC125",&a).is_err());
}

#[test]
fn batch4_dahlia_conditional_public_uses_personal_sun_and_blocks_secret_deploy_without_spending(){
    let mut g=game();let id=held(&mut g,"WM060",0);fund(&mut g,0,"WM060",6);g.players[1].society_zone.exposure=1;
    assert!(!g.current_public(&g.players[0].hand[0],0));g.players[0].society_zone.exposure=1;
    reject(&mut g,0,Action{card_id:Some(id.clone()),region:Some(0),..Action::new("conceal")});assert_eq!(g.resources(0),6);
    g.add_society_markers(0,0,1);assert_eq!(g.players[0].society_zone.exposure,0);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("conceal")});assert_eq!(g.resources(0),5);restore(&mut g);
}

#[test]
fn batch4_dahlia_actual_normal_deploy_does_not_observe_itself(){
    let mut g=game();fund(&mut g,0,"WM060",5);let id=held(&mut g,"WM060",0);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});top(&mut g);assert_eq!(g.resources(0),0);assert!(g.pending.is_none());
    let id=g.regions[0].cards.iter().find(|c|c.definition=="WM060").unwrap().id.clone();
    for seat in 0..4{let v=g.view(seat);assert_eq!(v.regions[0].characters.iter().find(|c|c.instance_id==id).unwrap().current_public,Some(false));}
    g.players[0].society_zone.exposure=1;
    for seat in 0..4{let v=g.view(seat);assert_eq!(v.regions[0].characters.iter().find(|c|c.instance_id==id).unwrap().current_public,Some(true));}restore(&mut g);
}

#[test]
fn batch4_dahlia_actual_reveal_counts_only_current_other_personal_criminals_in_original_region(){
    let mut g=game();let own=field(&mut g,"JZ24",0);field(&mut g,"XG26",0);field(&mut g,"JZ24",1);field(&mut g,"JZ24",2);
    fund(&mut g,0,"WM060",6);let id=held(&mut g,"WM060",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("conceal")});
    let id=g.regions[0].cards.iter().find(|c|c.definition=="WM060").unwrap().id.clone();act(&mut g,0,Action{card_id:Some(id),..Action::new("reveal")});pending(&mut g);
    let ChoiceResolution::Declare{declaration:d,..}=&g.pending.as_ref().unwrap().resolution else{panic!()};assert_eq!(d.ability.key,dahlia::REVEAL);
    choose(&mut g,vec!["accept".into()]);g.remove_dead(&own,RemovalCause::Destroy);g.regions[0].influence[1]=1;restore(&mut g);top(&mut g);
    assert_eq!(g.regions[0].influence,[1,1]);assert_eq!(g.resources(0),0);restore(&mut g);
}

#[test]
fn batch4_dahlia_reveal_effect_survives_source_leave_but_not_region_replacement(){
    for replace in [false,true]{let mut g=game();let source=field(&mut g,"WM060",0);field(&mut g,"JZ24",0);
        let s=g.source_snapshot(g.board(&source).unwrap().1,Some(0));g.emit_event(0,s,Event::Reveal);g.drive().unwrap();choose(&mut g,vec!["accept".into()]);
        if replace{g.regions[0].card=g.make_card("DQJC115",0);}else{g.remove_dead(&source,RemovalCause::Destroy);}
        restore(&mut g);top(&mut g);assert_eq!(g.regions[0].influence[0],u32::from(!replace));restore(&mut g);
    }
}

#[test]
fn batch4_dahlia_actual_entry_chain_restores_draw_discard_and_two_separate_region_markers(){
    let (mut g,_)=trigger();g.regions[0].influence[0]=2;g.regions[1].influence[0]=1;g.regions[0].influence[1]=2;
    pending(&mut g);let p=g.pending.clone().unwrap();assert_eq!(p.choice.kind,"discard");assert_eq!(g.players[0].hand.len(),1);
    for seat in 1..4{assert!(g.view(seat).pending_choice.as_ref().is_none_or(|p|p.options.is_empty()));}
    let discarded=p.choice.options[0].id.clone();choose(&mut g,vec![discarded]);assert!(g.players[0].hand.is_empty());
    let p=g.pending.as_ref().unwrap();assert_eq!(p.choice.kind,"influence");assert_eq!(p.choice.amount,Some(2));restore(&mut g);
    choose(&mut g,vec!["region:0:marker:0".into(),"region:1:marker:0".into()]);assert_eq!(g.regions[0].influence,[1,2]);assert_eq!(g.regions[1].influence[0],0);restore(&mut g);
}

#[test]
fn batch4_dahlia_ally_enemy_non_criminal_and_hidden_source_do_not_trigger(){
    for actor in [1,2]{let mut g=game();field(&mut g,"WM060",0);crime(&mut g,actor);top(&mut g);assert!(g.pending.is_none());restore(&mut g);}
    let mut g=game();field(&mut g,"WM060",0);fund(&mut g,0,"JC029",3);let id=held(&mut g,"JC029",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});top(&mut g);assert!(g.pending.is_none());
    let mut g=game();let id=field(&mut g,"WM060",0);g.board_mut(&id).unwrap().face_down=true;crime(&mut g,0);top(&mut g);assert!(g.pending.is_none());restore(&mut g);
}

#[test]
fn batch4_dahlia_direct_grave_entry_is_observed_without_a_paid_play_receipt(){
    let mut g=game();field(&mut g,"WM060",0);let c=g.make_card("JZ24",0);let id=c.id.clone();g.players[0].graveyard.push(c);
    let region=g.regions.len()-1;g.world_graveyard_entry(0,region,&id).unwrap();g.drive().unwrap();let ChoiceResolution::Declare{declaration:d,..}=&g.pending.as_ref().unwrap().resolution else{panic!()};
    assert_eq!(d.ability.key,dahlia::ENTRY);let Op::WM060CriminalObservation{observation:Some(o)}=&d.ability.ops[0]else{panic!()};assert_ne!(o.entered.card.id,id);assert_eq!(o.entered.region,Some(region));assert_eq!(d.source.region,Some(0));assert!(o.entered.play_source.is_none());restore(&mut g);
}

#[test]
fn batch4_dahlia_captured_chain_survives_source_leave_and_repress_clamps_to_available_influence(){
    for count in 0..=2{let (mut g,source)=trigger();g.regions[0].influence[0]=count;g.remove_dead(&source,RemovalCause::Destroy);restore(&mut g);draw_discard(&mut g);
        if count>0{let p=g.pending.as_ref().unwrap();assert_eq!(p.choice.min,Some(count as usize));let ids=p.choice.options.iter().take(count as usize).map(|o|o.id.clone()).collect();choose(&mut g,ids);}else{assert!(g.pending.is_none());}
        assert_eq!(g.regions[0].influence[0],0);restore(&mut g);
    }
}

#[test]
fn batch4_dahlia_multiple_actual_entries_before_queue_drain_keep_distinct_order_group_references(){
    let mut g=game();field(&mut g,"WM060",0);field(&mut g,"BQ037",0);
    for _ in 0..2{let c=g.make_card("JZ24",0);let id=c.id.clone();g.players[0].graveyard.push(c);g.world_graveyard_entry(0,0,&id).unwrap();}
    assert_eq!(g.entry_order_groups.len(),2);restore(&mut g);g.drive().unwrap();assert_eq!(g.pending.as_ref().unwrap().choice.kind,"batch3-entry-order");restore(&mut g);
}

#[test]
fn batch4_dahlia_recovery_rejects_duplicate_or_foreign_entry_and_changed_draw_discard_program(){
    let mut g=game();field(&mut g,"WM060",0);crime(&mut g,0);pending(&mut g);
    for mode in 0..4{let mut bad=g.clone();let ChoiceResolution::Declare{declaration:d,..}=&mut bad.pending.as_mut().unwrap().resolution else{panic!()};match mode{
        0=>if let Op::WM060CriminalObservation{observation:Some(o)}=&mut d.ability.ops[0]{o.entered.card.controller=1;},
        1=>{d.ability.ops.remove(2);},2=>d.ability.ops[1]=Op::Draw{player:PlayerRef::Actor,count:2,end:DeckEnd::Top},
        _=>{let copy=d.clone();bad.effects.push_back(Effect::Declare{declaration:copy});},
    }assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());}
}

#[test]
fn batch4_dahlia_discard_and_repress_metadata_and_polluted_commands_are_closed(){
    let (mut g,_)=trigger();g.regions[0].influence[0]=2;pending(&mut g);let p=g.pending.clone().unwrap();let id=p.choice.options[0].id.clone();
    reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec![id.clone()]),target_id:Some("p2".into()),..Action::new("choose")});
    let mut bad=g.clone();if let ChoiceResolution::Frame{frame,..}=&mut bad.pending.as_mut().unwrap().resolution{frame.cursor=2;}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    choose(&mut g,vec![id]);let mut bad=g.clone();bad.pending.as_mut().unwrap().choice.amount=Some(1);assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let p=g.pending.clone().unwrap();reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec!["region:0:marker:0".into(),"region:0:marker:0".into()]),..Action::new("choose")});
}

#[test]
fn batch4_dahlia_existing_granted_combat_glory_remains_resumable(){
    let mut g=game();let source=field(&mut g,"WM060",0);let c=g.make_card("JC089",0);g.attachments.push(Attachment{card:c,host_id:source});
    let d=g.jc089_combat_glory(0,0).unwrap();g.effects.push_back(Effect::Declare{declaration:d});g.drive().unwrap();restore(&mut g);choose(&mut g,vec!["accept".into()]);top(&mut g);assert_eq!(g.regions[0].influence[0],1);restore(&mut g);
}

#[test]
fn batch4_dahlia_other_criminal_reveal_and_its_own_trigger_can_be_ordered_without_old_batch_observers(){
    let mut g=game();field(&mut g,"WM060",0);let n=catalog::card("JZ24").cost as usize;fund(&mut g,0,"JZ24",n+1);let id=held(&mut g,"JZ24",0);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("conceal")});let id=g.regions[0].cards.iter().find(|c|c.definition=="JZ24").unwrap().id.clone();
    act(&mut g,0,Action{card_id:Some(id),..Action::new("reveal")});pending(&mut g);
    let p=g.pending.as_ref().unwrap();assert_eq!(p.choice.kind,"batch3-entry-order");assert_eq!(p.choice.options.len(),2);
    let id=p.choice.options.iter().find(|o|o.id.ends_with(dahlia::ENTRY)).unwrap().id.clone();choose(&mut g,vec![id]);
    let ChoiceResolution::Declare{declaration:d,..}=&g.pending.as_ref().unwrap().resolution else{panic!()};assert_eq!(d.ability.key,dahlia::ENTRY);restore(&mut g);
}
