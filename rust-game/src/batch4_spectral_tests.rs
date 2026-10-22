use crate::{catalog,model::*,rules::*,unit_support::*};
const KEY:&str=crate::batch4_spectral::KEY;
fn layout(moon:u32,mentors:usize)->(Game,String){let mut g=game();fund(&mut g,0,"XG11",8);if moon>0{g.add_society_markers(0,0,moon);}let target=field(&mut g,"BQ079",2);g.board_mut(&target).unwrap().permanent_defense_markers=12;for n in 0..mentors{field(&mut g,"JC038",n%2);}restore(&mut g);(g,target)}
fn hidden(g:&mut Game)->String{let id=held(g,"XG11",0);act(g,0,Action{card_id:Some(id),region:Some(0),..Action::new("conceal")});let c=g.regions[0].cards.iter().find(|c|c.definition=="XG11").unwrap();assert!(c.face_down);c.id.clone()}
fn pending(g:&mut Game){for _ in 0..96{if g.pending.is_some(){restore(g);return;}if !g.effects.is_empty(){g.drive().unwrap();restore(g);}else{pass(g);}}panic!("bounded conditional Raid choice");}
fn reveal(g:&mut Game,id:&str){act(g,0,Action{card_id:Some(id.into()),..Action::new("reveal")});}
fn caster(g:&Game)->String{g.regions.iter().flat_map(|r|&r.cards).find(|c|c.definition=="XG11"&&!c.face_down).unwrap().id.clone()}
fn damage(g:&Game,id:&str)->u32{g.board(id).unwrap().1.damage}
#[test]
fn batch4_spectral_exact_original_metadata_and_closed_conditional_registry(){
    let d=catalog::card("XG11");assert_eq!((d.cost,d.defense,d.supported,d.unique),(4,Some(1),false,false));assert_eq!(d.loyalty,["蓝色"]);assert_eq!(d.magic,"死亡");assert_eq!(d.subtypes,["阴魂","法师","诅咒"]);assert_eq!(d.permanent_icons,Icons{influence:1,..Icons::default()});assert_eq!(d.temporary_icons,Icons{investigation:1,combat:1,..Icons::default()});assert!(!definition("XG11").traits.spirit);
    let mut changed=crate::rules::definitions().clone();changed.get_mut("XG11").unwrap().abilities[0].targets[0].range=Range::Anywhere;assert!(crate::rules::validate_definitions(&changed).is_err());
}

#[test]
fn batch4_spectral_moon_spirit_is_personal_dynamic_and_never_active_while_hidden(){
    let mut g=game();let id=field(&mut g,"XG11",0);assert!(!g.current_spirit(g.board(&id).unwrap().1));g.add_society_markers(1,0,3);assert!(!g.current_spirit(g.board(&id).unwrap().1));g.add_society_markers(0,0,1);assert!(g.current_spirit(g.board(&id).unwrap().1));
    for seat in 0..4{assert_eq!(g.view(seat).regions[0].characters.iter().find(|c|c.instance_id==id).unwrap().current_spirit_protection,Some(true));}
    let h=g.hide_in_place_successor(&id).unwrap().id;assert!(!g.current_spirit(g.board(&h).unwrap().1));for seat in 0..4{assert_eq!(g.view(seat).regions[0].characters.iter().find(|c|c.instance_id==h).unwrap().current_spirit_protection,None);}restore(&mut g);
}

#[test]
fn batch4_spectral_paid_faceup_entry_and_movement_never_emit_reveal_raid(){
    let(mut g,target)=layout(3,2);let id=held(&mut g,"XG11",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});top(&mut g);assert_eq!(damage(&g,&target),0);assert!(g.pending.is_none());let id=caster(&g);g.move_on_board_instance(&id,1);g.drive().unwrap();assert!(g.pending.is_none()&&g.stack.is_empty());assert_eq!(damage(&g,&target),0);restore(&mut g);
}

#[test]
fn batch4_spectral_actual_paid_reveal_uses_one_merged_raid_for_each_personal_moon_tier(){
    for moon in 0..=5{let(mut g,target)=layout(moon,0);let h=hidden(&mut g);let before=g.resources(0);reveal(&mut g,&h);assert_eq!(g.resources(0),before-4);
        if moon==0{top(&mut g);assert!(g.pending.is_none());}else{pending(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.title,format!("阴魂术士：袭击{}",moon.min(3)));choose(&mut g,vec![target.clone()]);assert_eq!(g.stack.iter().filter(|i|i.frame.as_ref().is_some_and(|f|f.ability_key==KEY)).count(),1);top(&mut g);}
        assert_eq!(damage(&g,&target),moon.min(3));assert_eq!(g.resources(0),3);restore(&mut g);
    }
}

#[test]
fn batch4_spectral_jc038_grants_merge_once_with_conditional_printed_raid_including_zero_moon(){
    for moon in [0,1,3,7]{let(mut g,target)=layout(moon,2);field(&mut g,"JC038",2);let mut facedown=g.make_card("JC038",0);facedown.face_down=true;g.regions[0].cards.push(facedown);let h=hidden(&mut g);reveal(&mut g,&h);pending(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().choice.title,format!("阴魂术士：袭击{}",moon.min(3)+2));assert!(!g.effects.iter().any(|e|matches!(e,Effect::Declare{declaration:d}if d.source.card.definition=="XG11"&&d.ability.key=="jc038-granted-raid")));
        choose(&mut g,vec![target.clone()]);top(&mut g);assert_eq!(damage(&g,&target),moon.min(3)+2);restore(&mut g);
    }
}

#[test]
fn batch4_spectral_teammate_moon_does_not_satisfy_personal_tiers_but_team_mentor_still_grants(){
    let(mut g,target)=layout(0,0);g.add_society_markers(1,0,3);field(&mut g,"JC038",1);let h=hidden(&mut g);reveal(&mut g,&h);pending(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.title,"阴魂术士：袭击1");choose(&mut g,vec![target.clone()]);top(&mut g);assert_eq!(damage(&g,&target),1);assert!(!g.current_spirit(g.board(&caster(&g)).unwrap().1));restore(&mut g);
}

#[test]
fn batch4_spectral_reveal_receipt_survives_moon_and_mentor_loss_and_original_source_departure(){
    let(mut g,target)=layout(1,2);let h=hidden(&mut g);reveal(&mut g,&h);pending(&mut g);let source=caster(&g);choose(&mut g,vec![target.clone()]);let mentors=g.regions[0].cards.iter().filter(|c|c.definition=="JC038").map(|c|c.id.clone()).collect::<Vec<_>>();g.add_society_markers(0,1,0);for id in mentors{g.remove_dead(&id,RemovalCause::Destroy);}g.remove_dead(&source,RemovalCause::Destroy);restore(&mut g);top(&mut g);assert_eq!(damage(&g,&target),3);restore(&mut g);
}

#[test]
fn batch4_spectral_condition_gained_after_reveal_does_not_retroactively_create_a_raid(){
    let(mut g,target)=layout(0,0);let h=hidden(&mut g);reveal(&mut g,&h);top(&mut g);g.add_society_markers(0,0,3);field(&mut g,"JC038",0);g.drive().unwrap();assert!(g.pending.is_none()&&g.stack.is_empty());assert_eq!(damage(&g,&target),0);assert!(g.current_spirit(g.board(&caster(&g)).unwrap().1));restore(&mut g);
}

#[test]
fn batch4_spectral_decline_has_no_damage_or_repeated_payment(){
    let(mut g,target)=layout(3,1);let h=hidden(&mut g);reveal(&mut g,&h);pending(&mut g);let resources=g.resources(0);choose(&mut g,vec![]);assert!(g.stack.is_empty()&&g.pending.is_none());assert_eq!(damage(&g,&target),0);assert_eq!(g.resources(0),resources);restore(&mut g);
}

#[test]
fn batch4_spectral_original_region_guard_survives_source_movement_and_rejects_region_replacement(){
    for replace in [false,true]{let(mut g,target)=layout(2,0);let h=hidden(&mut g);reveal(&mut g,&h);pending(&mut g);choose(&mut g,vec![target.clone()]);let source=caster(&g);
        if replace{g.regions[0].card=g.make_card("DQJC114",0);}else{g.move_on_board_instance(&source,1);}top(&mut g);assert_eq!(damage(&g,&target),if replace{0}else{2});restore(&mut g);
    }
}

#[test]
fn batch4_spectral_actual_fast_hide_response_invalidates_original_target_without_retargeting(){
    let(mut g,target)=layout(3,0);fund(&mut g,2,"JC063",2);let chase=held(&mut g,"JC063",2);let h=hidden(&mut g);reveal(&mut g,&h);pending(&mut g);choose(&mut g,vec![target.clone()]);
    for _ in 0..8{if g.priority_team==g.team(2){break;}pass(&mut g);}assert_eq!(g.priority_team,g.team(2));act(&mut g,2,Action{card_id:Some(chase),target_id:Some(target.clone()),option:Some("hide".into()),..Action::new("play")});top(&mut g);assert!(g.board(&target).is_none());top(&mut g);assert!(g.regions[0].cards.iter().any(|c|c.definition=="BQ079"&&c.face_down&&c.damage==0));restore(&mut g);
}

#[test]
fn batch4_spectral_choice_actor_extra_payload_and_receipt_corruption_are_rejected(){
    let(mut g,target)=layout(2,1);let h=hidden(&mut g);reveal(&mut g,&h);pending(&mut g);let a=Action{choice_id:Some(g.pending.as_ref().unwrap().choice.id.clone()),selected:Some(vec![target.clone()]),..Action::new("choose")};reject(&mut g,1,a.clone());let mut extra=a.clone();extra.cost_selected=Some(vec![]);reject(&mut g,0,extra);
    for change in 0..7{let mut bad=g.clone();let ChoiceResolution::Declare{declaration:d,..}=&mut bad.pending.as_mut().unwrap().resolution else{panic!("Raid declaration")};let Op::XG11RevealRaid{observation:Some(o)}=&mut d.ability.ops[0]else{panic!("Raid receipt")};match change{0=>o.turn+=1,1=>o.event_id=d.source.card.id.clone(),2=>o.grants.push(o.grants[0].clone()),3=>o.grants[0].controller=2,4=>o.grants[0].definition="JC095".into(),5=>o.secrecy=0,_=>d.ability.targets[0].range=Range::Anywhere};assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());}
    act(&mut g,0,a.clone());reject(&mut g,0,a);let mut bad=g.clone();bad.stack.push(bad.stack.last().unwrap().clone());assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());top(&mut g);assert_eq!(damage(&g,&target),3);
}

#[test]
fn batch4_spectral_conditional_program_cannot_be_replaced_with_the_old_generic_grant_path(){
    let(mut g,_)=layout(1,1);let h=hidden(&mut g);reveal(&mut g,&h);pending(&mut g);let grant=g.regions[0].cards.iter().find(|c|c.definition=="JC038").unwrap().clone();
    let a=ability("jc038-granted-raid","袭击1",Timing::Fast,vec![],vec![target(Zone::Board,EntityKind::Character,Relation::Any,Range::SourceRegion)],vec![Op::JC038GrantedRaid{grants:vec![grant]}],Some(Event::Reveal));
    assert!(crate::raid::validate_ability("JC038",&a).is_ok());assert!(crate::raid::validate_ability("XG11",&a).is_err());
    let mut bad=g.clone();let ChoiceResolution::Declare{declaration:d,..}=&mut bad.pending.as_mut().unwrap().resolution else{panic!("Raid declaration")};d.ability=a;assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
}
