//! Legal explicit layouts and real paid commands. Roundtrips test normal
//! continuations; these are author native tests, not browser acceptance.
use crate::{catalog, import20_vampires as v, model::*, rules::*, unit_support::*};
use crate::jc029_tests::pass_top;

fn conceal(g: &mut Game, id: &str) -> String {
    let source=held(g,id,0);
    act(g,0,Action{card_id:Some(source),region:Some(0),..Action::new("conceal")});
    g.regions[0].cards.iter().find(|c|c.definition==id&&c.face_down).unwrap().id.clone()
}
fn raid_pending(g: &mut Game) {
    for _ in 0..80 {
        if g.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Declare{declaration:d,..}if d.ability.key==v::RAID)){return;}
        if g.pending.is_some(){resolve_choice(g);}else{pass(g);}
    }
    panic!("bounded real reveal raid choice");
}
fn assassin(exposure: [u32;2], target: &str) -> (Game,String) {
    let mut g=game();fund(&mut g,0,"JC029",10);
    g.add_society_markers(2,exposure[0],0);g.add_society_markers(3,exposure[1],0);
    let target=field(&mut g,target,2);let hidden=conceal(&mut g,"WM024");
    act(&mut g,0,Action{card_id:Some(hidden),..Action::new("reveal")});raid_pending(&mut g);
    (g,target)
}
fn begin_win(g: &mut Game, winning: bool) {
    let cards=if winning{["WM025","WM025","JC125","JC125"]}else{["JC125","JC125","WM025","WM025"]};
    let shown=cards.iter().enumerate().map(|(seat,id)|held(g,id,seat)).collect::<Vec<_>>();
    fund(g,0,"JC125",2);let source=held(g,"BQ090",0);
    act(g,0,Action{card_id:Some(source),ability_id:Some(crate::import20_contest::HAND.into()),..Action::new("activate")});
    pass_top(g);
    for _ in 0..96 {
        if let Some(p)=g.pending.clone(){
            if matches!(&p.resolution,ChoiceResolution::Declare{declaration:d,..}if d.ability.key==v::WIN){return;}
            if p.choice.kind==crate::mystic_contest::COMMIT_KIND{choose(g,vec![shown[p.seat].clone()]);}
            else if p.choice.kind==crate::mystic_contest::REWARD_KIND{choose(g,vec![]);}
            else{resolve_choice(g);}
        }else if !g.stack.is_empty(){pass(g);}else{return;}
    }
    panic!("bounded real mystic win");
}
fn accept_win(g: &mut Game) {
    assert!(g.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Declare{declaration:d,..}if d.ability.key==v::WIN)));
    choose(g,vec!["accept".into()]);pass_top(g);
}
fn field_at(g: &mut Game,id:&str,seat:usize,region:usize)->String {
    let c=g.make_card(id,seat);let uid=c.id.clone();g.regions[region].cards.push(c);uid
}

#[test]
fn import20_vampire_prints_exact_closed_native_definitions_and_latest_identity(){
    assert_eq!(catalog::ENGINE_VERSION,"rust-v0.2.109-wm057-personal-borrow");
    for id in v::CARDS{let d=catalog::card(id);assert!(!d.supported);assert!(catalog::runtime_implemented(d));assert_eq!(d.color,"蓝");}
    let d=catalog::card("WM025");assert_eq!((d.cost,d.defense,d.unique),(8,Some(4),true));assert_eq!(d.loyalty,["蓝色","蓝色","蓝色"]);
    assert_eq!(d.permanent_icons,Icons{investigation:1,combat:2,influence:2});assert_eq!(d.temporary_icons,Icons{investigation:1,..Icons::default()});
    assert_eq!(d.keywords,["领袖","隐秘条件：屏障"]);assert!(!definition("WM025").traits.barrier);
    let d=catalog::card("WM024");assert_eq!((d.cost,d.defense,d.unique),(6,Some(2),false));assert_eq!(d.loyalty,["蓝色"]);assert_eq!(d.magic,"血");
    assert_eq!(d.subtypes,["吸血鬼","罪犯"]);assert_eq!(d.permanent_icons.combat,2);assert_eq!(d.temporary_icons.influence,1);
    validate_definitions(definitions()).unwrap();
}

#[test]
fn import20_wm025_real_deploy_pays_eight_and_requires_three_blue_loyalty(){
    let mut g=game();let c=held(&mut g,"WM025",0);fund(&mut g,0,"JC029",2);fund(&mut g,0,"JC125",6);
    let a=Action{card_id:Some(c),region:Some(0),..Action::new("deploy")};reject(&mut g,0,a.clone());
    fund(&mut g,0,"JC029",1);act(&mut g,0,a);assert_eq!(g.resources(0),1);pass_top(&mut g);
    assert_eq!(g.regions[0].cards.iter().filter(|c|c.definition=="WM025").count(),1);assert!(g.pending.is_none());restore(&mut g);
}

#[test]
fn import20_wm025_moon_barrier_is_personal_dynamic_and_four_seat_public_only(){
    let mut g=game();let source=field(&mut g,"WM025",0);g.add_society_markers(1,0,4);
    assert!(g.targetable(2,g.board(&source).unwrap().1));g.add_society_markers(0,0,1);
    assert!(!g.targetable(2,g.board(&source).unwrap().1));assert!(g.targetable(1,g.board(&source).unwrap().1));
    for seat in 0..4{assert_eq!(g.view(seat).regions[0].characters.iter().find(|c|c.instance_id==source).unwrap().current_barrier,Some(true));}
    g.add_society_markers(0,1,0);assert!(g.targetable(2,g.board(&source).unwrap().1));restore(&mut g);
    g.add_society_markers(0,0,1);let hidden=g.hide_in_place_successor(&source).unwrap().id;
    for seat in 0..4{let view=g.view(seat);let c=view.regions[0].characters.iter().find(|c|c.instance_id==hidden).unwrap();assert_eq!(c.current_barrier,None);if seat!=0{assert!(c.card_id.is_none());}}
    assert!(g.targetable(2,g.board(&hidden).unwrap().1));restore(&mut g);
}

#[test]
fn import20_wm025_reveal_aura_is_personal_current_and_does_not_reduce_faceup_play(){
    let mut g=game();let source=field(&mut g,"WM025",0);g.add_society_markers(0,0,2);g.add_society_markers(1,0,5);
    let mut own=g.make_card("JC029",0);own.face_down=true;let mut ally=g.make_card("JC029",1);ally.face_down=true;
    assert_eq!(g.paid_reveal_cost(0,&own),0);assert_eq!(g.effective_cost(0,&own),2);assert_eq!(g.paid_reveal_cost(1,&ally),2);
    let mut human=g.make_card("JC056",0);human.face_down=true;assert_eq!(g.paid_reveal_cost(0,&human),2);
    g.remove_dead(&source,RemovalCause::Destroy);assert_eq!(g.paid_reveal_cost(0,&own),2);restore(&mut g);
}

#[test]
fn import20_wm025_real_paid_reveal_aura_floors_cost_but_never_waives_loyalty(){
    let mut g=game();field(&mut g,"WM025",0);g.add_society_markers(0,0,7);fund(&mut g,0,"JC125",1);
    let hidden=field(&mut g,"JC029",0);g.board_mut(&hidden).unwrap().face_down=true;reject(&mut g,0,Action{card_id:Some(hidden.clone()),..Action::new("reveal")});
    fund(&mut g,0,"JC029",1);reject(&mut g,0,Action{card_id:Some(hidden.clone()),..Action::new("reveal")});
    fund(&mut g,0,"JC029",1);let before=g.resources(0);act(&mut g,0,Action{card_id:Some(hidden),..Action::new("reveal")});
    assert_eq!(g.resources(0),before);pass_top(&mut g);assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC029"&&!c.face_down));restore(&mut g);
}

#[test]
fn import20_wm025_owned_manor_conversion_also_discounts_hidden_humans(){
    let mut g=game();field(&mut g,"WM025",0);g.add_society_markers(0,0,1);fund(&mut g,0,"JC029",5);
    let manor=held(&mut g,"XQ15",0);act(&mut g,0,Action{card_id:Some(manor),target_id:Some("society:p0".into()),..Action::new("play")});pass_top(&mut g);
    let mut human=g.make_card("JC056",0);human.face_down=true;assert_eq!(g.paid_reveal_cost(0,&human),1);
    human.owner=1;assert_eq!(g.paid_reveal_cost(0,&human),2);restore(&mut g);
}

#[test]
fn import20_wm025_actual_personal_mystic_win_places_once_per_current_own_vampire_region(){
    let mut g=game();field(&mut g,"WM025",0);field(&mut g,"JC029",0);field_at(&mut g,"JC029",0,1);
    field_at(&mut g,"JC029",1,2);field_at(&mut g,"JC029",2,3);
    let hidden=field_at(&mut g,"JC029",0,4);g.board_mut(&hidden).unwrap().face_down=true;
    begin_win(&mut g,true);assert_eq!(g.mystic_history.len(),1);accept_win(&mut g);
    assert_eq!(g.regions.iter().map(|r|r.influence[0]).collect::<Vec<_>>(),[1,1,0,0,0]);restore(&mut g);
}

#[test]
fn import20_wm025_win_can_be_declined_and_loss_never_creates_personal_reward(){
    let mut g=game();field(&mut g,"WM025",0);begin_win(&mut g,true);choose(&mut g,vec![]);
    assert!(g.regions.iter().all(|r|r.influence==[0,0]));assert!(g.stack.is_empty());
    let mut g=game();field(&mut g,"WM025",0);begin_win(&mut g,false);
    assert_eq!(g.mystic_history.last().unwrap().winner_team,Some(1));assert!(g.regions.iter().all(|r|r.influence==[0,0]));assert!(g.pending.is_none());restore(&mut g);
}

#[test]
fn import20_wm025_accepted_win_survives_source_departure_and_queries_current_vampires(){
    let mut g=game();let source=field(&mut g,"WM025",0);let moved=field_at(&mut g,"JC029",0,1);
    begin_win(&mut g,true);choose(&mut g,vec!["accept".into()]);g.remove_dead(&source,RemovalCause::Destroy);
    g.move_on_board_instance(&moved,2);restore(&mut g);pass_top(&mut g);
    assert_eq!(g.regions.iter().map(|r|r.influence[0]).collect::<Vec<_>>(),[0,0,1,0,0]);restore(&mut g);
}

#[test]
fn import20_wm025_real_attached_bat_remains_a_character_but_does_not_inherit_host_vampire_type(){
    let mut g=game();field(&mut g,"WM025",0);let host=field_at(&mut g,"JC056",0,1);fund(&mut g,0,"JC029",8);
    let manor=held(&mut g,"XQ15",0);act(&mut g,0,Action{card_id:Some(manor),target_id:Some("society:p0".into()),..Action::new("play")});pass_top(&mut g);
    let manor=g.attachments.iter().find(|a|a.card.definition=="XQ15").unwrap().card.id.clone();
    let hand=held(&mut g,"BQ026",0);act(&mut g,0,Action{card_id:Some(hand),region:Some(1),..Action::new("deploy")});
    for _ in 0..40{if g.pending.is_some(){break;}pass(&mut g);}
    assert!(g.pending.as_ref().is_some_and(|p|p.choice.options.iter().any(|o|o.id==host)));
    choose(&mut g,vec![host]);pass_top(&mut g);
    assert!(g.attachments.iter().any(|a|a.card.definition=="BQ026"));
    g.remove_dead(&manor,RemovalCause::Destroy);assert!(!g.current_subtypes(g.board(&g.regions[1].cards[0].id).unwrap().1).iter().any(|s|s=="吸血鬼"));
    let bat=&g.attachments.iter().find(|a|a.card.definition=="BQ026").unwrap().card;
    assert!(catalog::has_kind(catalog::card("BQ026"),"character"));assert!(!g.current_subtypes(bat).iter().any(|s|s=="吸血鬼"));restore(&mut g);
    begin_win(&mut g,true);accept_win(&mut g);assert_eq!(g.regions[1].influence[0],0);restore(&mut g);
}

#[test]
fn import20_wm025_hidden_source_and_nonmystic_events_do_not_generate_win_trigger(){
    let mut g=game();let source=field(&mut g,"WM025",0);let s=g.source_snapshot(g.board(&source).unwrap().1,Some(0));
    g.emit_event(0,s,Event::RegionConfrontationsEnded);assert!(g.effects.is_empty());
    g.hide_in_place_successor(&source).unwrap();begin_win(&mut g,true);assert!(g.pending.is_none());assert!(g.regions.iter().all(|r|r.influence==[0,0]));restore(&mut g);
}

#[test]
fn import20_wm024_both_enemy_exposures_reduce_play_and_reveal_not_friendly_or_printed_cost(){
    let mut g=game();let c=g.make_card("WM024",0);g.add_society_markers(0,9,0);g.add_society_markers(1,7,0);
    assert_eq!(g.effective_cost(0,&c),6);assert_eq!(g.paid_reveal_cost(0,&c),6);
    g.add_society_markers(2,1,0);g.add_society_markers(3,1,0);
    assert_eq!(g.effective_cost(0,&c),2);assert_eq!(g.paid_reveal_cost(0,&c),2);assert_eq!(catalog::card("WM024").cost,6);
    g.add_society_markers(3,20,0);assert_eq!(g.effective_cost(0,&c),0);assert_eq!(g.paid_reveal_cost(0,&c),0);restore(&mut g);
}

#[test]
fn import20_wm024_real_faceup_play_uses_discount_without_emitting_reveal_raid(){
    let mut g=game();g.add_society_markers(2,2,0);fund(&mut g,0,"JC029",3);let source=held(&mut g,"WM024",0);
    act(&mut g,0,Action{card_id:Some(source),region:Some(0),..Action::new("deploy")});assert_eq!(g.resources(0),1);pass_top(&mut g);
    assert!(g.pending.is_none()&&g.stack.is_empty());assert!(g.regions[0].cards.iter().any(|c|c.definition=="WM024"));restore(&mut g);
}

#[test]
fn import20_wm024_zero_cost_reveal_still_requires_blue_and_rejects_client_chosen_raid(){
    let mut g=game();fund(&mut g,0,"JC125",1);g.add_society_markers(2,3,0);let hidden=conceal(&mut g,"WM024");
    let a=Action{card_id:Some(hidden),..Action::new("reveal")};reject(&mut g,0,a.clone());fund(&mut g,0,"JC029",1);
    let mut bad=a.clone();bad.target_id=Some("unbound".into());reject(&mut g,0,bad);
    act(&mut g,0,a);assert_eq!(g.resources(0),1);raid_pending(&mut g);choose(&mut g,vec![]);restore(&mut g);
}

#[test]
fn import20_wm024_real_reveal_combines_both_enemy_counts_and_draws_only_for_renown_death(){
    for target in ["JC056","JC070"]{
        let(mut g,target_id)=assassin([1,1],target);let before=g.players[0].hand.len();choose(&mut g,vec![target_id.clone()]);pass_top(&mut g);
        assert!(g.board(&target_id).is_none());assert_eq!(g.players[0].hand.len(),before+usize::from(target=="JC070"));
        let after=g.players[0].hand.len();g.drive().unwrap();assert_eq!(g.players[0].hand.len(),after);restore(&mut g);
    }
}

#[test]
fn import20_wm024_dynamic_x_reads_exposure_after_response_and_paid_costs_stay_fixed(){
    let(mut g,target)=assassin([1,0],"JC095");choose(&mut g,vec![target.clone()]);let resources=g.resources(0);
    g.add_society_markers(3,1,0);restore(&mut g);pass_top(&mut g);assert!(g.board(&target).is_none());assert_eq!(g.players[0].hand.len(),1);assert_eq!(g.resources(0),resources);
    let(mut g,target)=assassin([1,1],"JC095");choose(&mut g,vec![target.clone()]);g.add_society_markers(2,0,1);restore(&mut g);pass_top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.damage,1);assert!(g.players[0].hand.is_empty());restore(&mut g);
}

#[test]
fn import20_wm024_actual_dynamic_renown_grant_and_denial_determine_the_death_reward(){
    let(mut g,target)=assassin([0,1],"JC056");choose(&mut g,vec![target.clone()]);field(&mut g,"WM041",2);assert!(g.has_renown(g.board(&target).unwrap().1));
    restore(&mut g);pass_top(&mut g);assert!(g.board(&target).is_none());assert_eq!(g.players[0].hand.len(),1);restore(&mut g);
    let(mut g,target)=assassin([1,0],"JC070");choose(&mut g,vec![target.clone()]);field(&mut g,"WM041",0);assert!(!g.has_renown(g.board(&target).unwrap().1));
    restore(&mut g);pass_top(&mut g);assert!(g.board(&target).is_none());assert!(g.players[0].hand.is_empty());restore(&mut g);
}

#[test]
fn import20_vampire_damage_receipts_freeze_simultaneous_renown_before_aura_departure(){
    let mut g=game();let aura=field(&mut g,"WM041",2);let target=field(&mut g,"JC056",2);g.add_society_markers(2,1,0);
    assert!(g.has_renown(g.board(&aura).unwrap().1)&&g.has_renown(g.board(&target).unwrap().1));
    let receipts=g.damage_with_receipts([(aura.clone(),2),(target.clone(),1)].into()).unwrap();
    assert_eq!(receipts.len(),2);assert!(receipts.iter().all(|r|r.was_renown));
    assert!(receipts.iter().any(|r|r.old_instance==aura)&&receipts.iter().any(|r|r.old_instance==target));restore(&mut g);
}

#[test]
fn import20_wm024_zero_damage_or_enemy_shield_does_not_fake_a_kill_reward(){
    let(mut g,target)=assassin([0,0],"JC070");choose(&mut g,vec![target.clone()]);pass_top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.damage,0);assert!(g.players[0].hand.is_empty());restore(&mut g);
    let(mut g,target)=assassin([2,0],"JC070");choose(&mut g,vec![target.clone()]);g.board_mut(&target).unwrap().shield=1;restore(&mut g);pass_top(&mut g);
    assert_eq!((g.board(&target).unwrap().1.damage,g.board(&target).unwrap().1.shield),(0,0));assert!(g.players[0].hand.is_empty());restore(&mut g);
}

#[test]
fn import20_wm024_one_additive_mentor_raid_keeps_grant_after_mentor_departure(){
    let mut g=game();let target=field(&mut g,"JC095",2);let mentor=field(&mut g,"JC038",1);field(&mut g,"JC038",3);
    g.add_society_markers(2,1,0);fund(&mut g,0,"JC029",7);let hidden=conceal(&mut g,"WM024");
    act(&mut g,0,Action{card_id:Some(hidden),..Action::new("reveal")});raid_pending(&mut g);
    assert!(!g.effects.iter().any(|e|matches!(e,Effect::Declare{declaration:d}if d.ability.key=="jc038-granted-raid")));
    choose(&mut g,vec![target.clone()]);assert_eq!(g.stack.len(),1);g.remove_dead(&mentor,RemovalCause::Destroy);restore(&mut g);pass_top(&mut g);
    assert!(g.board(&target).is_none());assert_eq!(g.players[0].hand.len(),1);restore(&mut g);
}

#[test]
fn import20_wm024_invalidated_original_target_cancels_without_new_target_or_refund(){
    let(mut g,target)=assassin([2,0],"JC070");choose(&mut g,vec![target.clone()]);let resources=g.resources(0);
    g.hide_in_place_successor(&target).unwrap();restore(&mut g);pass_top(&mut g);
    assert!(g.players[0].hand.is_empty());assert_eq!(g.resources(0),resources);assert!(g.regions[0].cards.iter().any(|c|c.face_down&&c.definition=="JC070"));restore(&mut g);
}

#[test]
fn import20_wm024_real_fast_defense_response_prevents_lethal_without_a_draw(){
    let(mut g,target)=assassin([1,0],"JC070");choose(&mut g,vec![target.clone()]);
    let hermit=field(&mut g,"LC12",2);for _ in 0..4{if g.can_fast(2){break;}pass(&mut g);}assert!(g.can_fast(2));
    act(&mut g,2,Action{card_id:Some(hermit.clone()),ability_id:Some("protect-local-character".into()),target_id:Some(target.clone()),..Action::new("activate")});
    pass_top(&mut g);assert!(g.board(&hermit).unwrap().1.exhausted);assert_eq!(g.defense(g.board(&target).unwrap().1,0),2);
    pass_top(&mut g);assert_eq!(g.board(&target).unwrap().1.damage,1);assert!(g.players[0].hand.is_empty());restore(&mut g);
}

#[test]
fn import20_wm024_room_choice_is_at_most_once_and_resumes_original_reveal(){
    use crate::room::{RoomEnvelope,RoomCommand,SessionAction};
    let(g,target)=assassin([1,0],"JC070");let p=g.pending.clone().unwrap();let room=RoomEnvelope::from_game(g);
    let command=RoomCommand{command_id:"import20-assassin-original-choice".into(),expected_version:room.revision,
        action:SessionAction::Game{action:Action{choice_id:Some(p.choice.id),selected:Some(vec![target.clone()]),..Action::new("choose")}}};
    let result=room.transition(0,Some(command.clone()),0).unwrap();assert_eq!(result.outcome,"accepted");
    let room=RoomEnvelope::from_persisted(&result.state).unwrap();let before=serde_json::to_value(&room).unwrap();
    let duplicate=room.transition(0,Some(command),0).unwrap();assert_eq!(duplicate.outcome,"rejected");assert!(!duplicate.changed&&duplicate.journal.is_empty());
    let room=RoomEnvelope::from_persisted(&duplicate.state).unwrap();assert_eq!(serde_json::to_value(&room).unwrap(),before);
    let mut g=room.game;pass_top(&mut g);assert!(g.board(&target).is_none());assert_eq!(g.players[0].hand.len(),1);restore(&mut g);
}

#[test]
fn import20_vampire_closed_registry_does_not_allow_borrowed_or_retimed_programs(){
    let a=definition("WM025").abilities[0].clone();assert!(validate_ability("JC125",&a).is_err());
    let mut changed=definitions().clone();changed.get_mut("WM025").unwrap().traits.barrier=true;assert!(validate_definitions(&changed).is_err());
    let mut a=definition("WM024").abilities[0].clone();a.targets[0].range=Range::Anywhere;assert!(validate_ability("WM024",&a).is_err());
    let mut a=definition("WM024").abilities[0].clone();a.event=Some(Event::Enter);assert!(validate_ability("WM024",&a).is_err());
}
