use crate::{catalog,model::*,rules::*,unit_support::*};
const KEY:&str=crate::batch4_politician::KEY;
fn pending(g:&mut Game){for _ in 0..96{if g.pending.is_some(){restore(g);return;}if !g.effects.is_empty(){g.drive().unwrap();restore(g);}else{pass(g);}}panic!("bounded politician trigger");}
fn drain(g:&mut Game){for _ in 0..192{if let Some(p)=g.pending.clone(){let ids=if p.choice.kind=="batch3-entry-order"{vec![p.choice.options[0].id.clone()]}else if p.choice.options.iter().any(|o|o.id=="accept"){vec!["accept".into()]}else if p.choice.min.unwrap_or(0)>0{vec![p.choice.options[0].id.clone()]}else{vec![]};choose(g,ids);}else if !g.stack.is_empty(){pass(g);}else if !g.effects.is_empty(){g.drive().unwrap();restore(g);}else{return;}}panic!("bounded politician chain");}
fn paid(g:&mut Game,id:&str,seat:usize,r:usize)->String{g.begin_window(Window::Action(g.team(seat)));let n=catalog::card(id).cost as usize;fund(g,seat,id,n.max(1));let h=held(g,id,seat);act(g,seat,Action{card_id:Some(h),region:Some(r),..Action::new("deploy")});top(g);g.regions[r].cards.iter().rev().find(|c|c.definition==id&&c.controller==seat).unwrap().id.clone()}
fn kill(g:&mut Game,id:&str){g.begin_window(Window::Action(g.team(2)));fund(g,2,"JC104",3);let spell=held(g,"JC102",2);act(g,2,Action{card_id:Some(spell),target_id:Some(id.into()),..Action::new("play")});pending(g);}
fn aura(g:&mut Game)->String{let id=field(g,"WM029",0);kill(g,&id);choose(g,vec!["accept".into()]);top(g);assert_eq!(g.politician_auras.len(),1);id}
fn seal(g:&mut Game,id:&str){let c=g.make_card("JC125",0);g.seal_card_on_valid_host(c,0,id);g.drive().unwrap();restore(g);}
fn traits(g:&Game,id:&str)->(bool,bool){let c=g.board(id).unwrap().1;(g.has_renown(c),g.has_barrier(c))}
fn bad(g:&Game){assert!(Game::from_persisted(&serde_json::to_string(g).unwrap()).is_err());}
#[test]
fn batch4_politician_exact_original_fields_and_closed_registry(){
 let c=catalog::card("WM029");assert_eq!((c.cost,c.defense,c.supported,c.unique),(1,Some(1),false,false));assert_eq!(c.loyalty,["红色"]);assert_eq!(c.subtypes,["人类","宿主","政治家","邪教徒"]);assert_eq!(c.permanent_icons.influence,1);assert!(definition("WM029").traits.public);
 let mut wrong=crate::rules::definitions().clone();wrong.get_mut("WM029").unwrap().abilities[0].ops.clear();assert!(crate::rules::validate_definitions(&wrong).is_err());
}

#[test]
fn batch4_politician_real_public_deploy_pays_one_and_rejects_extra_fields_or_conceal(){
 let mut g=game();fund(&mut g,0,"WM029",3);let h=held(&mut g,"WM029",0);reject(&mut g,0,Action{card_id:Some(h.clone()),region:Some(0),..Action::new("conceal")});reject(&mut g,0,Action{card_id:Some(h.clone()),region:Some(0),option:Some("extra".into()),..Action::new("deploy")});let before=g.resources(0);act(&mut g,0,Action{card_id:Some(h),region:Some(0),..Action::new("deploy")});top(&mut g);let c=&g.regions[0].cards[0];assert_eq!(c.definition,"WM029");assert_eq!(g.resources(0),before-1);assert_eq!(traits(&g,&c.id),(false,false));assert!(g.politician_auras.is_empty());restore(&mut g);
}

#[test]
fn batch4_politician_indicator_types_include_wounds_and_shields_but_never_damage(){
 let mut g=game();let id=field(&mut g,"WM029",0);let c=g.board(&id).unwrap().1.clone();for n in 0..7{let mut c=c.clone();match n{0=>c.advancement_markers=1,1=>c.permanent_combat_markers=1,2=>c.permanent_defense_markers=1,3=>c.time_markers=1,4=>c.wounds=1,5=>c.shield=1,_=>c.lock_markers=1};assert_eq!(crate::batch4_politician::marker_count(&c),1);assert!(g.wm029_renown(&c));}let mut damaged=c;damaged.damage=1;assert_eq!(crate::batch4_politician::marker_count(&damaged),0);assert!(!g.wm029_renown(&damaged));
 // The above is a pure classifier test, not a claim those seven marker kinds
 // can all legally be placed on this cost1/defense1 character today.
}

#[test]
fn batch4_politician_real_paid_wm038_time_marker_updates_renown_for_every_seat(){
 let mut g=game();let id=paid(&mut g,"WM029",0,0);fund(&mut g,0,"WM038",1);let h=held(&mut g,"WM038",0);act(&mut g,0,Action{card_id:Some(h),target_id:Some(id.clone()),..Action::new("play")});drain(&mut g);assert_eq!(g.board(&id).unwrap().1.time_markers,1);assert_eq!(traits(&g,&id),(true,false));for s in 0..4{assert_eq!(g.view(s).regions[0].characters.iter().find(|c|c.instance_id==id).unwrap().current_renown,Some(true));}restore(&mut g);
}

#[test]
fn batch4_politician_attached_markers_count_on_host_and_fade_with_original_attachment(){
 let mut g=game();let id=field(&mut g,"WM029",0);let a=g.make_card("WM038",2);let aid=a.id.clone();g.attachments.push(Attachment{card:a,host_id:id.clone()});g.attachments.last_mut().unwrap().card.time_markers=1;assert_eq!(traits(&g,&id),(true,false));restore(&mut g);g.remove_dead(&aid,RemovalCause::Destroy);g.drive().unwrap();assert_eq!(traits(&g,&id),(false,false));restore(&mut g);
}

#[test]
fn batch4_politician_sealed_card_grants_barrier_and_real_enemy_spell_is_rejected(){
 let mut g=game();let id=field(&mut g,"WM029",0);seal(&mut g,&id);assert_eq!(traits(&g,&id),(false,true));for s in 0..4{let v=g.view(s);assert_eq!(v.regions[0].characters.iter().find(|c|c.instance_id==id).unwrap().current_barrier,Some(true));}
 g.begin_window(Window::Action(1));fund(&mut g,2,"JC104",3);let h=held(&mut g,"JC102",2);reject(&mut g,2,Action{card_id:Some(h),target_id:Some(id.clone()),..Action::new("play")});g.return_sealed_cards(&id);assert_eq!(traits(&g,&id),(false,false));restore(&mut g);
}

#[test]
fn batch4_politician_death_is_optional_respondable_and_applies_only_when_frame_resolves(){
 for decline in [true,false]{let mut g=game();let id=field(&mut g,"WM029",0);let other=field(&mut g,"JC032",2);g.board_mut(&other).unwrap().lock_markers=1;seal(&mut g,&other);kill(&mut g,&id);assert!(g.board(&id).is_none());assert_eq!(traits(&g,&other),(false,false));let p=g.pending.clone().unwrap();assert!(matches!(p.resolution,ChoiceResolution::Declare{..}));choose(&mut g,if decline{vec![]}else{vec!["accept".into()]});if !decline{assert!(g.stack.last().unwrap().frame.as_ref().unwrap().ability_key==KEY);assert!(g.politician_auras.is_empty());top(&mut g);}assert_eq!(traits(&g,&other),(!decline,!decline));assert_eq!(g.politician_auras.len(),usize::from(!decline));restore(&mut g);}
}

#[test]
fn batch4_politician_resolved_aura_is_global_dynamic_and_includes_future_characters(){
 let mut g=game();aura(&mut g);for s in 0..4{let id=field(&mut g,"JC032",s);assert_eq!(traits(&g,&id),(false,false));g.board_mut(&id).unwrap().lock_markers=1;assert_eq!(traits(&g,&id),(true,false));seal(&mut g,&id);assert_eq!(traits(&g,&id),(true,true));g.board_mut(&id).unwrap().lock_markers=0;g.return_sealed_cards(&id);assert_eq!(traits(&g,&id),(false,false));}let late=paid(&mut g,"JC032",2,0);assert_eq!(traits(&g,&late),(false,false));g.board_mut(&late).unwrap().time_markers=1;assert!(traits(&g,&late).0);restore(&mut g);
}

#[test]
fn batch4_politician_attached_character_receives_both_traits_and_views_without_exposing_hidden_front(){
 let mut g=game();aura(&mut g);let host=field(&mut g,"JC029",0);let c=g.make_card("BQ026",0);let bat=c.id.clone();g.attachments.push(Attachment{card:c,host_id:host.clone()});g.board_mut(&bat).unwrap().lock_markers=1;seal(&mut g,&bat);assert_eq!(traits(&g,&bat),(true,true));assert!(traits(&g,&host).0);
 for s in 0..4{let v=g.view(s);let a=&v.attachments.iter().find(|a|a.card.instance_id==bat).unwrap().card;assert_eq!((a.current_renown,a.current_barrier),(Some(true),Some(true)));}
 let hidden=g.hide_in_place_successor(&bat).unwrap().id;for s in 0..4{let v=g.view(s);let c=v.regions[0].characters.iter().find(|c|c.instance_id==hidden).unwrap();assert_eq!((c.current_renown,c.current_barrier),(None,None));}assert_eq!(traits(&g,&host),(false,false));restore(&mut g);
}

#[test]
fn batch4_politician_hidden_source_and_non_death_departure_never_install_an_aura(){
 for hidden in [false,true]{let mut g=game();let id=field(&mut g,"WM029",0);if hidden{g.hide_in_place(&id);let h=g.regions[0].cards[0].id.clone();g.remove_dead(&h,RemovalCause::Destroy);}else{g.return_hand(&id);}g.drive().unwrap();assert!(g.politician_auras.is_empty()&&g.pending.is_none());restore(&mut g);}
}

#[test]
fn batch4_politician_global_effect_expires_in_real_end_cleanup_without_erasing_markers_or_seals(){
 let mut g=game();aura(&mut g);let id=field(&mut g,"JC032",2);g.board_mut(&id).unwrap().lock_markers=1;seal(&mut g,&id);assert_eq!(traits(&g,&id),(true,true));let turn=g.turn;g.begin_window(Window::End);for _ in 0..64{if g.turn!=turn{break;}pass(&mut g);}assert_eq!(g.turn,turn+1);assert!(g.politician_auras.is_empty());assert_eq!(g.board(&id).unwrap().1.lock_markers,1);assert!(g.sealed_cards.iter().any(|s|s.host_id==id));assert_eq!(traits(&g,&id),(false,false));restore(&mut g);
}

#[test]
fn batch4_politician_two_deaths_and_ritual_echo_have_distinct_once_only_receipts(){
 let mut g=game();fund(&mut g,0,"JZ38",2);let r=held(&mut g,"JZ38",0);act(&mut g,0,Action{card_id:Some(r),target_id:Some("society:p0".into()),..Action::new("play")});top(&mut g);let id=field(&mut g,"WM029",0);kill(&mut g,&id);drain(&mut g);assert_eq!(g.politician_auras.len(),2);assert!(g.politician_auras.iter().any(|a|a.source.death_echo.is_some()));let id2=field(&mut g,"WM029",0);kill(&mut g,&id2);drain(&mut g);assert_eq!(g.politician_auras.len(),4);let ids=g.politician_auras.iter().map(|a|&a.frame_id).collect::<std::collections::BTreeSet<_>>();assert_eq!(ids.len(),4);restore(&mut g);
}

#[test]
fn batch4_politician_renown_denial_keeps_existing_enemy_alpha_precedence(){
 let mut g=game();let id=field(&mut g,"WM029",0);g.board_mut(&id).unwrap().lock_markers=1;assert!(traits(&g,&id).0);let a=field(&mut g,"WM041",2);assert!(!traits(&g,&id).0);g.remove_dead(&a,RemovalCause::Destroy);g.drive().unwrap();assert!(traits(&g,&id).0);restore(&mut g);
}

#[test]
fn batch4_politician_rejects_wrong_actor_extra_payload_duplicate_consumption_and_forged_auras(){
 let mut g=game();let id=field(&mut g,"WM029",0);kill(&mut g,&id);let p=g.pending.clone().unwrap();let a=Action{choice_id:Some(p.choice.id),selected:Some(vec!["accept".into()]),..Action::new("choose")};reject(&mut g,1,a.clone());let mut extra=a.clone();extra.cost_selected=Some(vec![]);reject(&mut g,0,extra);let mut forged=g.clone();let ChoiceResolution::Declare{declaration:d,..}=&mut forged.pending.as_mut().unwrap().resolution else{panic!()};d.ability.ops=vec![];bad(&forged);act(&mut g,0,a.clone());reject(&mut g,0,a);let original=g.stack.last().unwrap().clone();let mut wrong=g.clone();wrong.stack.push(original.clone());bad(&wrong);top(&mut g);assert_eq!(g.politician_auras.len(),1);
 for n in 0..6{let mut wrong=g.clone();let a=&mut wrong.politician_auras[0];match n{0=>a.expires_turn+=1,1=>a.frame_id=a.source.card.id.clone(),2=>a.source.card.definition="JC032".into(),3=>a.source.card.controller=1,4=>a.source.card.face_down=true,_=>a.source.region=Some(99)}bad(&wrong);}let mut wrong=g.clone();wrong.politician_auras.push(wrong.politician_auras[0].clone());bad(&wrong);let mut wrong=g.clone();wrong.stack.push(original);bad(&wrong);restore(&mut g);
}

fn conference(g:&mut Game)->String{g.begin_window(Window::Action(0));fund(g,0,"WM047",3);let p=held(g,"WM047",0);act(g,0,Action{card_id:Some(p),target_id:Some("society:p0".into()),..Action::new("play")});top(g);field(g,"XG17",0);g.attachments.iter().find(|a|a.card.definition=="WM047").unwrap().card.id.clone()}

#[test]
fn batch4_politician_real_later_chaos_entry_freezes_global_marker_renown_for_conference(){
 let mut g=game();aura(&mut g);let plan=conference(&mut g);fund(&mut g,0,"XG14",8);let h=held(&mut g,"XG14",0);act(&mut g,0,Action{card_id:Some(h),..Action::new("deploy")});pending(&mut g);let p=g.pending.clone().unwrap();assert!(p.choice.options.iter().any(|o|o.id==plan));let ChoiceResolution::Declare{declaration:d,..}=&p.resolution else{panic!()};let Op::XG17AdvancePlan{observation:Some(o)}=&d.ability.ops[0]else{panic!()};assert!(matches!(o.basis,crate::batch4_conference::RenownBasis::Politician{..}));assert_eq!(o.entered.card.time_markers,1);
 choose(&mut g,vec![plan.clone()]);g.politician_auras.clear();let id=g.regions.iter().flat_map(|r|&r.cards).find(|c|c.definition=="XG14").unwrap().id.clone();g.board_mut(&id).unwrap().time_markers=0;restore(&mut g);top(&mut g);assert_eq!(g.attachments.iter().find(|a|a.card.id==plan).unwrap().card.advancement_markers,1);restore(&mut g);
}

#[test]
fn batch4_politician_conference_freezes_self_attachment_markers_but_does_not_retroactively_observe(){
 let mut g=game();let plan=conference(&mut g);let id=field(&mut g,"WM029",0);g.enter_triggers(0,"WM029",&id,false);g.drive().unwrap();assert!(g.pending.is_none());g.board_mut(&id).unwrap().lock_markers=1;assert!(traits(&g,&id).0);assert!(g.effects.is_empty());g.board_mut(&id).unwrap().lock_markers=0;
 let mut c=g.make_card("WM038",0);c.time_markers=1;let aid=c.id.clone();g.attachments.push(Attachment{card:c,host_id:id.clone()});g.enter_triggers(0,"WM029",&id,false);g.drive().unwrap();let p=g.pending.clone().unwrap();let ChoiceResolution::Declare{declaration:d,..}=&p.resolution else{panic!()};let Op::XG17AdvancePlan{observation:Some(o)}=&d.ability.ops[0]else{panic!()};let crate::batch4_conference::RenownBasis::Politician{proof}=&o.basis else{panic!()};assert_eq!(proof.attachments.len(),1);assert!(proof.aura.is_none());choose(&mut g,vec![plan.clone()]);g.remove_dead(&aid,RemovalCause::Destroy);restore(&mut g);top(&mut g);assert_eq!(g.attachments.iter().find(|a|a.card.id==plan).unwrap().card.advancement_markers,1);restore(&mut g);
}

#[test]
fn batch4_politician_nested_attached_character_marker_inheritance_keeps_full_host_path(){
 let mut g=game();aura(&mut g);let host=field(&mut g,"JC029",0);let c=g.make_card("BQ026",0);let bat=c.id.clone();g.attachments.push(Attachment{card:c,host_id:host.clone()});g.begin_window(Window::Action(0));fund(&mut g,0,"JZ41",2);let h=held(&mut g,"JZ41",0);act(&mut g,0,Action{card_id:Some(h),target_id:Some(bat.clone()),..Action::new("play")});top(&mut g);let curse=g.attachments.iter().find(|a|a.card.definition=="JZ41").unwrap().card.id.clone();g.board_mut(&curse).unwrap().time_markers=1;
 assert!(traits(&g,&bat).0&&traits(&g,&host).0);let proof=g.wm029_renown_basis(g.board(&host).unwrap().1).unwrap();assert_eq!(proof.attachments.len(),2);g.validate_marker_renown(g.board(&host).unwrap().1,&proof).unwrap();restore(&mut g);
 let mut broken=proof.clone();broken.attachments.retain(|a|a.card.id!=bat);assert!(g.validate_marker_renown(g.board(&host).unwrap().1,&broken).is_err());let mut cycle=proof.clone();cycle.attachments.iter_mut().find(|a|a.card.id==bat).unwrap().host_id=curse.clone();assert!(g.validate_marker_renown(g.board(&host).unwrap().1,&cycle).is_err());
 g.remove_dead(&curse,RemovalCause::Destroy);g.drive().unwrap();assert!(!traits(&g,&bat).0&&!traits(&g,&host).0);g.validate_marker_renown(g.board(&host).unwrap().1,&proof).unwrap();restore(&mut g);
}
