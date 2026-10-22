use crate::{catalog,model::*,rules::*,unit_support::*};
const ADVANCE:&str=crate::batch4_plans::ADVANCE;
#[test]
fn batch4_new_order_shuffle_preserves_markers_on_cards_already_hidden(){
    let mut g=game();let a=field(&mut g,"JC125",0);let b=field(&mut g,"JC032",0);let ally=field(&mut g,"JC125",1);
    for id in [&a,&b,&ally]{g.board_mut(id).unwrap().face_down=true;}g.board_mut(&a).unwrap().lock_markers=1;g.board_mut(&ally).unwrap().lock_markers=2;restore(&mut g);
    g.world_hide(None,true);
    assert_eq!(g.regions[0].cards.iter().filter(|c|c.controller==0).map(|c|c.lock_markers).sum::<u32>(),1,"shuffling an already hidden card is not a new flip or zone departure");
    assert_eq!(g.board(&ally).unwrap().1.lock_markers,2);restore(&mut g);
}
fn layout(secret:bool)->Game { let mut g=game();fund(&mut g,0,"WM046",12);if secret{g.add_society_markers(0,0,1);}g }
fn start(g:&mut Game,id:&str,secret:bool){let id=held(g,id,0);act(g,0,Action{card_id:Some(id),target_id:Some("society:p0".into()),option:secret.then(||"secret-plan".into()),..Action::new("play")});}
fn enter(g:&mut Game,id:&str,secret:bool)->String{start(g,id,secret);top(g);g.attachments.iter().find(|a|a.card.definition==id).unwrap().card.id.clone()}
fn advance(g:&mut Game,id:&str){act(g,0,Action{card_id:Some(id.into()),ability_id:Some(ADVANCE.into()),..Action::new("activate")});}
fn complete_window(g:&mut Game){for _ in 0..64{if g.stack.last().and_then(|i|i.frame.as_ref()).is_some_and(|f|f.ability_key=="batch4-plan-complete"){restore(g);return;}pass(g);}panic!("bounded forced completion");}
fn view_secret(g:&Game,id:&str,n:u32){for seat in 0..4{let v=g.view(seat);let a=v.attachments.iter().find(|a|a.card.instance_id==id).unwrap();assert_eq!(a.card.name,"秘密计划");assert_eq!(a.card.kind,"attachment");assert_eq!(a.card.cost,Some(1));assert_eq!(a.card.advancement_markers,Some(n));assert!(a.card.face_down);if seat!=0{assert!(a.card.card_id.is_none()&&a.card.text.is_none()&&a.card.magic.is_none());}}}

#[test]
fn batch4_plans_fixed_thresholds_and_closed_pilot_admission() {
    for (id, threshold) in [("WM046",2),("WM047",5)] {
        assert_eq!(crate::batch4_plans::threshold(id),Some(threshold));
        assert!(!catalog::card(id).supported);
        let a=definition(id).abilities.iter().find(|a|a.key==crate::batch4_plans::ADVANCE).unwrap();
        assert!(a.activation_only);assert!(a.per_turn_limit.is_none());assert!(!a.once_per_game);
        let mut changed=crate::rules::definitions().clone();changed.get_mut(id).unwrap().abilities[1].costs.clear();assert!(crate::rules::validate_definitions(&changed).is_err());
    }
}

#[test]
fn batch4_plans_secret_play_requires_personal_moon_and_real_loyalty_without_consuming_moon(){
    for with_moon in [false,true]{let mut g=layout(false);g.add_society_markers(1,0,1);if with_moon{g.add_society_markers(0,0,1);}let id=held(&mut g,"WM046",0);let a=Action{card_id:Some(id.clone()),target_id:Some("society:p0".into()),option:Some("secret-plan".into()),..Action::new("play")};
        if with_moon{act(&mut g,0,a);assert_eq!(g.players[0].society_zone.secrecy,1);assert_eq!(g.resources(0),11);let item=g.stack.last().unwrap();let view=g.view(2).stack.last().unwrap().clone();assert_eq!(item.frame.as_ref().unwrap().source.card.definition,"WM046");assert_eq!(view.card_id,None);assert_eq!(view.ability_id,None);assert_eq!(view.label,"秘密计划");top(&mut g);let id=g.attachments[0].card.id.clone();view_secret(&g,&id,0);}
        else{reject(&mut g,0,a);assert_eq!(g.resources(0),12);}
    }
    let mut g=game();g.add_society_markers(0,0,1);fund(&mut g,0,"JC029",2);let id=held(&mut g,"WM046",0);reject(&mut g,0,Action{card_id:Some(id),target_id:Some("society:p0".into()),option:Some("secret-plan".into()),..Action::new("play")});
}

#[test]
fn batch4_plans_faceup_play_uses_stable_empty_or_present_society_and_only_own_zone(){
    for present in [false,true]{let mut g=if present{crate::msjc01_tests::game()}else{game()};fund(&mut g,0,"WM046",3);let society=serde_json::to_value(&g.players[0].society_zone.card).unwrap();let hand=held(&mut g,"WM046",0);
        for host in ["society:p1","society:p2","society:p99","region:0"]{reject(&mut g,0,Action{card_id:Some(hand.clone()),target_id:Some(host.into()),..Action::new("play")});}
        act(&mut g,0,Action{card_id:Some(hand),target_id:Some("society:p0".into()),..Action::new("play")});top(&mut g);assert!(!g.attachments[0].card.face_down);assert_eq!(g.attachments[0].host_id,"society:p0");assert_eq!(serde_json::to_value(&g.players[0].society_zone.card).unwrap(),society);restore(&mut g);
    }
}

#[test]
fn batch4_plans_secret_advances_repeatedly_even_after_moon_loss_and_reveals_only_at_exact_threshold(){
    let mut g=layout(true);let id=enter(&mut g,"WM046",true);g.add_society_markers(0,1,0);assert_eq!(g.players[0].society_zone.secrecy,0);let resources=g.resources(0);
    advance(&mut g,&id);assert_eq!(g.resources(0),resources-1);top(&mut g);view_secret(&g,&id,1);assert_eq!(g.attachments[0].card.id,id);assert!(!g.attachments[0].card.exhausted);
    advance(&mut g,&id);assert_eq!(g.resources(0),resources-2);complete_window(&mut g);assert!(g.pending.is_none());let c=&g.attachments[0].card;assert_eq!(c.id,id);assert!(!c.face_down);assert_eq!(c.advancement_markers,2);
    for seat in 0..4{let c=&g.view(seat).attachments[0].card;assert_eq!(c.card_id.as_deref(),Some("WM046"));assert_eq!(c.advancement_markers,Some(2));}
    top(&mut g);assert!(g.attachments.is_empty());let dead=g.players[0].graveyard.iter().find(|c|c.definition=="WM046").unwrap();assert_ne!(dead.id,id);assert_eq!(dead.advancement_markers,0);assert!(!dead.face_down);restore(&mut g);
}

#[test]
fn batch4_plans_advance_rejects_other_actor_offturn_response_extra_payload_and_duplicate_payments(){
    let mut g=layout(false);let id=enter(&mut g,"WM047",false);let a=Action{card_id:Some(id.clone()),ability_id:Some(ADVANCE.into()),..Action::new("activate")};
    for actor in 1..4{reject(&mut g,actor,a.clone());}
    let mut bad=a.clone();bad.target_id=Some("society:p0".into());reject(&mut g,0,bad);let mut bad=a.clone();bad.option=Some("repeat".into());reject(&mut g,0,bad);let mut bad=a.clone();bad.cost_selected=Some(vec![]);reject(&mut g,0,bad);
    g.begin_window(Window::Action(1));reject(&mut g,0,a.clone());g.begin_window(Window::Action(0));let resources=g.resources(0);advance(&mut g,&id);reject(&mut g,0,a);assert_eq!(g.resources(0),resources-1);top(&mut g);assert_eq!(g.attachments[0].card.advancement_markers,1);
}

#[test]
fn batch4_plans_tianyan_counts_locked_entities_not_markers_and_includes_hidden_and_attached_characters(){
    let mut g=layout(false);let host=field(&mut g,"JC029",0);g.board_mut(&host).unwrap().lock_markers=3;
    let other=field(&mut g,"JC125",2);let hidden=g.hide_in_place_successor(&other).unwrap().id;g.board_mut(&hidden).unwrap().lock_markers=2;
    let mut bat=g.make_card("BQ026",0);bat.lock_markers=1;g.attachments.push(Attachment{card:bat,host_id:host});field(&mut g,"JC125",3);
    let id=enter(&mut g,"WM046",false);let cards=g.players[0].hand.len();advance(&mut g,&id);top(&mut g);assert_eq!(g.players[0].hand.len(),cards);advance(&mut g,&id);complete_window(&mut g);assert_eq!(g.players[0].hand.len(),cards);top(&mut g);assert_eq!(g.players[0].hand.len(),cards+3);restore(&mut g);
}

#[test]
fn batch4_plans_tianyan_zero_draw_does_not_eliminate_and_completion_survives_original_source_loss(){
    let mut g=layout(false);let id=enter(&mut g,"WM046",false);let locked=field(&mut g,"JC125",2);g.board_mut(&locked).unwrap().lock_markers=1;advance(&mut g,&id);top(&mut g);advance(&mut g,&id);complete_window(&mut g);
    let cards=g.players[0].hand.len();g.remove_dead(&id,RemovalCause::Destroy);let replacement=enter_fixture(&mut g,"WM046");top(&mut g);assert_eq!(g.players[0].hand.len(),cards+1);assert!(g.attachments.iter().any(|a|a.card.id==replacement&&a.card.advancement_markers==0));restore(&mut g);
    let mut g=layout(false);let id=enter(&mut g,"WM046",false);g.players[0].deck.clear();advance(&mut g,&id);top(&mut g);advance(&mut g,&id);complete_window(&mut g);top(&mut g);assert!(!g.players[0].eliminated);restore(&mut g);
}
fn enter_fixture(g:&mut Game,id:&str)->String{let c=g.make_card(id,0);let id=c.id.clone();g.attachments.push(Attachment{card:c,host_id:"society:p0".into()});id}

#[test]
fn batch4_plans_secret_target_remains_attachment_for_strengthened_jc005_not_hidden_character(){
    let mut g=layout(true);let id=enter(&mut g,"WM047",true);fund(&mut g,0,"JC005",2);let spell=held(&mut g,"JC005",0);
    assert!(g.legal_actions(0).iter().any(|a|a.action.card_id.as_ref()==Some(&spell)&&a.action.target_id.as_ref()==Some(&id)));
    act(&mut g,0,Action{card_id:Some(spell),target_id:Some(id.clone()),..Action::new("play")});top(&mut g);assert!(!g.attachments.iter().any(|a|a.card.id==id));
}

#[test]
fn batch4_plans_forged_secret_receipts_programs_markers_and_stack_duplicates_are_rejected(){
    let mut g=layout(true);start(&mut g,"WM046",true);
    for mutation in 0..5{let mut bad=g.clone();let f=bad.stack.last_mut().unwrap().frame.as_mut().unwrap();match mutation{0=>if let Op::PlanPlayed{secrecy:Some(p)}=&mut f.steps[0].op{p.concealment=0},1=>f.source.card.face_down=false,2=>f.steps[0].op=Op::PlanComplete,3=>f.already_paid.clear(),_=>f.targets[0].id="society:p1".into()};assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());}
    top(&mut g);let id=g.attachments[0].card.id.clone();let mut bad=g.clone();bad.attachments[0].card.advancement_markers=2;assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    advance(&mut g,&id);let mut bad=g.clone();bad.stack.push(bad.stack.last().unwrap().clone());assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());top(&mut g);
    let card=held(&mut g,"JC125",0);g.players[0].hand.iter_mut().find(|c|c.id==card).unwrap().advancement_markers=1;assert!(Game::from_persisted(&serde_json::to_string(&g).unwrap()).is_err());
}

#[test]
fn batch4_plans_beowulf_exact_five_destroys_current_domains_atomically_including_attached_characters(){
    let mut g=layout(false);let id=enter(&mut g,"WM047",false);
    let advisor=field(&mut g,"JC060",0);let own=field(&mut g,"JC125",0);let ally=field(&mut g,"JC125",1);let enemy_magic=field(&mut g,"JC029",2);let enemy_plain=field(&mut g,"JC125",2);
    let old=field(&mut g,"JC029",3);let hidden=g.hide_in_place_successor(&old).unwrap().id;
    let vampire=field(&mut g,"JC029",0);let bat=g.make_card("BQ026",0);let bat_id=bat.id.clone();g.attachments.push(Attachment{card:bat,host_id:vampire.clone()});
    assert!(g.current_magic_count(g.board(&own).unwrap().1)>0);assert!(g.current_magic_count(g.board(&ally).unwrap().1)>0);
    assert!(g.current_magic_count(g.board(&bat_id).unwrap().1)>0);
    for n in 1..=5{advance(&mut g,&id);if n<5{top(&mut g);assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().card.advancement_markers,n);assert!(g.board(&own).is_some());}else{complete_window(&mut g);}}
    top(&mut g);for gone in [&advisor,&own,&ally,&enemy_magic,&bat_id,&vampire]{assert!(g.in_play(gone).is_none(),"Original magic-bearing instance {gone} survived");}
    assert!(g.board(&enemy_plain).is_some());assert!(g.board(&hidden).unwrap().1.face_down);assert!(g.attachments.iter().all(|a|a.card.id!=id));restore(&mut g);
}

#[test]
fn batch4_plans_source_loss_while_advancing_never_advances_a_replacement_and_does_not_refund(){
    let mut g=layout(false);let id=enter(&mut g,"WM047",false);let resources=g.resources(0);advance(&mut g,&id);g.remove_dead(&id,RemovalCause::Destroy);let replacement=enter_fixture(&mut g,"WM047");top(&mut g);
    assert_eq!(g.resources(0),resources-1);assert_eq!(g.attachments.iter().find(|a|a.card.id==replacement).unwrap().card.advancement_markers,0);restore(&mut g);
}

#[test]
fn batch4_plans_secret_play_freezes_moon_condition_before_response_without_revealing_print(){
    let mut g=layout(true);start(&mut g,"WM047",true);g.add_society_markers(0,1,0);restore(&mut g);top(&mut g);let id=g.attachments[0].card.id.clone();view_secret(&g,&id,0);
    for seat in 1..4{let public=serde_json::to_string(&g.view(seat)).unwrap();assert!(!public.contains("WM047"));assert!(!public.contains("贝奥武夫"));}
}

#[test]
fn batch4_plans_legal_secret_advancement_remains_visible_only_to_its_controller(){
    let mut g=layout(true);let id=enter(&mut g,"WM047",true);view_secret(&g,&id,0);
    assert!(g.legal_actions(0).iter().any(|a|a.action.card_id.as_ref()==Some(&id)&&a.action.ability_id.as_deref()==Some(ADVANCE)));
    for seat in 1..4{assert!(!g.legal_actions(seat).iter().any(|a|a.action.card_id.as_ref()==Some(&id)));}
    for n in 1..=3{advance(&mut g,&id);top(&mut g);view_secret(&g,&id,n);}
    assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().card.advancement_markers,3);restore(&mut g);
}

#[test]
fn batch4_plans_secret_is_not_a_hidden_character_for_spells_or_character_abilities(){
    let mut g=layout(true);let id=enter(&mut g,"WM047",true);let witness=field(&mut g,"JC003",0);fund(&mut g,0,"JC003",2);
    let source=g.source_snapshot(g.board(&witness).unwrap().1,Some(0));
    let slot=target(Zone::Board,EntityKind::Hidden,Relation::Any,Range::Anywhere);assert!(!g.valid_binding(0,&source,&slot,&id));
    let slot=target(Zone::Board,EntityKind::CharacterOrHidden,Relation::Any,Range::Anywhere);assert!(!g.valid_binding(0,&source,&slot,&id));
    let slot=target(Zone::Board,EntityKind::Attachment,Relation::Any,Range::Anywhere);assert!(g.valid_binding(0,&source,&slot,&id));restore(&mut g);
}

#[test]
fn batch4_plans_advance_requires_available_resource_and_never_taps_the_plan(){
    let mut g=layout(false);let id=enter(&mut g,"WM047",false);for c in &mut g.players[0].assets{c.exhausted=true;}let a=Action{card_id:Some(id.clone()),ability_id:Some(ADVANCE.into()),..Action::new("activate")};reject(&mut g,0,a.clone());
    g.players[0].assets[0].exhausted=false;act(&mut g,0,a);top(&mut g);let c=&g.attachments.iter().find(|a|a.card.id==id).unwrap().card;assert!(!c.exhausted);assert_eq!(c.advancement_markers,1);assert_eq!(g.resources(0),0);restore(&mut g);
}

#[test]
fn batch4_plans_normal_and_secret_plays_have_executable_published_legal_actions(){
    let mut g=layout(true);let id=held(&mut g,"WM046",0);let actions=g.legal_actions(0).into_iter().filter(|a|a.action.kind=="play"&&a.action.card_id.as_ref()==Some(&id)).collect::<Vec<_>>();
    assert_eq!(actions.len(),2);assert!(actions.iter().any(|a|a.action.option.as_deref()==Some("secret-plan")));
    let faceup=actions.iter().find(|a|a.action.option.is_none()).unwrap().action.clone();assert_eq!(faceup.ability_id.as_deref(),Some("attach"));act(&mut g,0,faceup);top(&mut g);assert!(!g.attachments[0].card.face_down);restore(&mut g);
}

#[test]
fn jc060_attached_bat_real_paid_entry_keeps_friendly_granted_domain_and_combat_in_four_views(){
    let mut g=game();let host=field(&mut g,"JC029",0);let advisor=field(&mut g,"JC060",1);fund(&mut g,0,"JC029",1);let id=held(&mut g,"BQ026",0);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});
    for _ in 0..64{if g.pending.is_some(){break;}if !g.effects.is_empty(){g.drive().unwrap();}else{pass(&mut g);}}
    assert!(g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id==host));choose(&mut g,vec![host]);top(&mut g);let bat=g.attachments.iter().find(|a|a.card.definition=="BQ026").unwrap().card.id.clone();
    let c=g.board(&bat).unwrap().1;assert_eq!(g.current_magic_count(c),1,"Attaching does not remove character type or friendly continuous grants");assert_eq!(g.current_icons(c,0).combat,2);
    for seat in 0..4{let a=g.view(seat).attachments.into_iter().find(|a|a.card.instance_id==bat).unwrap();assert_eq!(a.card.current_domains,Some(vec![MagicDomainCount{magic:"心灵".into(),count:1}]));assert_eq!(a.card.icons.unwrap().combat,2);}restore(&mut g);
    g.remove_dead(&advisor,RemovalCause::Destroy);assert_eq!(g.current_magic_count(g.board(&bat).unwrap().1),0);assert_eq!(g.current_icons(g.board(&bat).unwrap().1,0).combat,1);
    for seat in 0..4{assert_eq!(g.view(seat).attachments.iter().find(|a|a.card.instance_id==bat).unwrap().card.current_domains,None);}restore(&mut g);
}
