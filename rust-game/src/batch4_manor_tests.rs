use crate::{catalog, model::*, rules::*, unit_support::*};
fn play(g: &mut Game, seat: usize) -> String {
    g.begin_window(Window::Action(g.team(seat)));
    fund(g,seat,"XQ15",2); let hand=held(g,"XQ15",seat);
    act(g,seat,Action{card_id:Some(hand),target_id:Some(format!("society:p{seat}")),..Action::new("play")}); top(g);
    g.attachments.iter().find(|a|a.card.definition=="XQ15"&&a.card.controller==seat).unwrap().card.id.clone()
}
fn has(g:&Game,id:&str,t:&str)->bool{g.current_subtypes(g.board(id).unwrap().1).iter().any(|s|s==t)}
fn bad(g:&Game){assert!(Game::from_persisted(&serde_json::to_string(g).unwrap()).is_err());}

#[test]
fn batch4_manor_original_unique_subtitle_and_personal_society_registry_are_closed(){
 let c=catalog::card("XQ15");assert_eq!((c.cost,c.unique,c.supported),(2,true,false));assert_eq!(c.subtitle.as_deref(),Some("阴谋会聚之所"));assert_eq!(c.loyalty,["蓝色"]);
 let d=definition("XQ15");assert_eq!(d.abilities.len(),1);assert_eq!(d.attachment.as_ref().unwrap().host.zone,Zone::Society);
 let mut wrong=d.clone();wrong.modifiers.clear();assert!(crate::batch4::validate_definition("XQ15",&wrong).is_err());
 let mut copy=crate::rules::definitions().clone();copy.get_mut("JC125").unwrap().modifiers.push(StaticModifier::XQ15OwnerVampireSlave);assert!(crate::rules::validate_definitions(&copy).is_err());
}

#[test]
fn batch4_manor_actual_paid_attachment_changes_owned_human_not_printed_definition(){
 let mut g=game();let role=field(&mut g,"JC125",0);let before=catalog::card("JC125").subtypes.clone();let source=play(&mut g,0);
 assert_eq!(g.resources(0),0);assert!(has(&g,&role,"吸血鬼"));assert!(!has(&g,&role,"人类"));assert!(!has(&g,&role,"奴仆"));
 assert_eq!(catalog::card("JC125").subtypes,before);assert_eq!(g.attachments.iter().find(|a|a.card.id==source).unwrap().host_id,"society:p0");
 for s in 0..4{let v=g.view(s);let c=v.regions[0].characters.iter().find(|c|c.instance_id==role).unwrap();assert_eq!(c.current_subtypes,Some(vec!["吸血鬼".into()]));}restore(&mut g);
}

#[test]
fn batch4_manor_ownership_differs_from_current_controller_and_teammate(){
 let mut g=game();let own=field(&mut g,"JC125",0);g.board_mut(&own).unwrap().controller=2;
 let stolen=field(&mut g,"JC125",2);g.board_mut(&stolen).unwrap().controller=0;
 let ally=field(&mut g,"JC125",1);let enemy=field(&mut g,"JC125",3);play(&mut g,0);
 assert!(has(&g,&own,"吸血鬼"));for id in [stolen,ally,enemy]{assert!(has(&g,&id,"人类"));assert!(!has(&g,&id,"吸血鬼"));}restore(&mut g);
}

#[test]
fn batch4_manor_nonhuman_becomes_slave_but_vampire_is_unchanged_without_duplicate_types(){
 let mut g=game();let ghoul=field(&mut g,"BQ079",0);let vamp=field(&mut g,"JC029",0);let human=field(&mut g,"WM029",0);play(&mut g,0);
 assert!(has(&g,&ghoul,"奴仆"));assert!(has(&g,&ghoul,"食尸鬼"));assert!(has(&g,&vamp,"吸血鬼"));assert!(!has(&g,&vamp,"奴仆"));
 assert_eq!(g.current_subtypes(g.board(&human).unwrap().1).iter().filter(|s|s.as_str()=="吸血鬼").count(),1);restore(&mut g);
}

#[test]
fn batch4_manor_works_in_hand_deck_and_grave_while_assets_and_seals_stay_blank(){
 let mut g=game();let hand=held(&mut g,"JC125",0);let grave=g.make_card("JC125",0);g.players[0].graveyard.push(grave.clone());
 let deck=g.make_card("JC125",0);g.players[0].deck.push(deck.clone());let asset=g.make_card("JC125",0);g.players[0].assets.push(asset.clone());
 let sealed=g.make_card("JC125",0);g.sealed_cards.push(SealedCard{card:sealed.clone(),host_id:"society:p0".into()});play(&mut g,0);
 for c in [g.players[0].hand.iter().find(|c|c.id==hand).unwrap(),&grave,&deck]{assert_eq!(g.current_subtypes(c),vec!["吸血鬼"]);}
 assert!(!g.xq15_owner_aura(&asset));assert!(!g.xq15_owner_aura(&sealed));
 assert_eq!(g.view(0).hand.iter().find(|c|c.instance_id==hand).unwrap().current_subtypes,Some(vec!["吸血鬼".into()]));
 assert_eq!(g.view(0).graveyard.iter().find(|c|c.instance_id==grave.id).unwrap().current_subtypes,Some(vec!["吸血鬼".into()]));
 for s in 0..4{let v=g.sealed_card_view(&sealed,s);assert!(v.current_subtypes.is_none());assert!(v.icons.is_none());assert!(v.defense.is_none());}
 restore(&mut g);
}

#[test]
fn batch4_manor_hidden_identity_and_private_projection_stay_hidden_until_real_paid_reveal(){
 let mut g=game();play(&mut g,0);fund(&mut g,0,"JC125",1);let hand=held(&mut g,"JC125",0);
 act(&mut g,0,Action{card_id:Some(hand),region:Some(0),..Action::new("conceal")});let hidden=g.regions[0].cards.iter().find(|c|c.definition=="JC125").unwrap().id.clone();
 assert!(g.current_subtypes(g.board(&hidden).unwrap().1).is_empty());for s in 0..4{let v=g.view(s);let c=v.regions[0].characters.iter().find(|c|c.instance_id==hidden).unwrap();assert!(c.current_subtypes.is_none());if s!=0{assert!(c.card_id.is_none());}}
 act(&mut g,0,Action{card_id:Some(hidden),..Action::new("reveal")});top(&mut g);let live=g.regions[0].cards.iter().find(|c|c.definition=="JC125"&&!c.face_down).unwrap();assert_eq!(g.current_subtypes(live),vec!["吸血鬼"]);restore(&mut g);
}

#[test]
fn batch4_manor_exhaustion_keeps_aura_and_source_departure_reverts_in_all_owned_zones(){
 let mut g=game();let role=field(&mut g,"JC125",0);let h=held(&mut g,"JC125",0);let source=play(&mut g,0);
 g.board_mut(&source).unwrap().exhausted=true;assert!(has(&g,&role,"吸血鬼"));restore(&mut g);
 g.remove_dead(&source,RemovalCause::Destroy);g.drive().unwrap();assert!(has(&g,&role,"人类"));assert_eq!(g.current_subtypes(g.players[0].hand.iter().find(|c|c.id==h).unwrap()),vec!["人类"]);restore(&mut g);
}

#[test]
fn batch4_manor_real_jc005_society_attachment_removal_reverts_without_changing_owners(){
 let mut g=game();let role=field(&mut g,"JC125",0);let source=play(&mut g,0);
 g.begin_window(Window::Action(1));fund(&mut g,2,"JC005",2);fund(&mut g,2,"JZ02",1);let spell=held(&mut g,"JC005",2);
 act(&mut g,2,Action{card_id:Some(spell),target_id:Some(source.clone()),..Action::new("play")});top(&mut g);
 assert!(g.in_play(&source).is_none());assert!(has(&g,&role,"人类"));assert_eq!(g.board(&role).unwrap().1.owner,0);restore(&mut g);
}

#[test]
fn batch4_manor_losing_human_invalidates_real_human_host_attachment_without_recursion(){
 let mut g=game();let human=field(&mut g,"JC125",0);let tattoo=g.make_card("JZ39",0);let tid=tattoo.id.clone();g.attachments.push(Attachment{card:tattoo,host_id:human.clone()});restore(&mut g);
 assert!(has(&g,&human,"恶魔"));play(&mut g,0);assert!(has(&g,&human,"吸血鬼"));assert!(!has(&g,&human,"恶魔"));assert!(g.in_play(&tid).is_none());restore(&mut g);
}

#[test]
fn batch4_manor_live_bat_blood_domain_conversion_precedes_nonhuman_slave_test(){
 let mut g=game();let bat=field(&mut g,"JC030",0);fund(&mut g,0,"XQ14",2);let source=play(&mut g,0);
 assert!(has(&g,&bat,"吸血鬼"));assert!(!has(&g,&bat,"奴仆"));g.players[0].assets.clear();assert!(!has(&g,&bat,"吸血鬼"));assert!(has(&g,&bat,"奴仆"));
 g.remove_dead(&source,RemovalCause::Destroy);assert!(!has(&g,&bat,"奴仆"));restore(&mut g);
}

#[test]
fn batch4_manor_current_subtype_targeting_and_explicit_printed_subtype_targeting_stay_distinct(){
 let mut g=game();let grave=g.make_card("JC125",0);let id=grave.id.clone();g.players[0].graveyard.push(grave);let source=play(&mut g,0);
 let s=g.source_snapshot(g.in_play(&source).unwrap().1,None);let mut t=target(Zone::Graveyard,EntityKind::Character,Relation::OwnedByActor,Range::Anywhere);t.subtype=Some("吸血鬼".into());
 assert!(g.binding_options(0,&s,&t).iter().any(|o|o.id==id));t.printed_subtype=true;assert!(!g.binding_options(0,&s,&t).iter().any(|o|o.id==id));t.subtype=Some("人类".into());assert!(g.binding_options(0,&s,&t).iter().any(|o|o.id==id));restore(&mut g);
}

#[test]
fn batch4_manor_personal_binding_and_extra_action_payloads_reject_atomically(){
 let mut g=game();fund(&mut g,0,"XQ15",2);let h=held(&mut g,"XQ15",0);
 for host in ["society:p1","region:0"]{reject(&mut g,0,Action{card_id:Some(h.clone()),target_id:Some(host.into()),..Action::new("play")});}
 reject(&mut g,0,Action{card_id:Some(h.clone()),target_id:Some("society:p0".into()),option:Some("forged".into()),..Action::new("play")});assert_eq!(g.resources(0),2);
 act(&mut g,0,Action{card_id:Some(h),target_id:Some("society:p0".into()),..Action::new("play")});
 for n in 0..5{let mut w=g.clone();let f=w.stack.last_mut().unwrap().frame.as_mut().unwrap();match n{0=>f.actor=1,1=>f.targets[0].id="society:p1".into(),2=>f.targets[0].spec.zone=Zone::Board,3=>f.steps.push(Step{context:0,op:Op::WM029TurnAura}),_=>f.cursor=1}bad(&w);}
 let mut duplicate=g.clone();duplicate.stack.push(duplicate.stack.last().unwrap().clone());bad(&duplicate);top(&mut g);restore(&mut g);
}

#[test]
fn batch4_manor_each_seat_uses_its_own_society_and_preserves_u64_seed(){
 for seat in 0..4{let mut g=game();g.seed=9_007_199_254_740_993;g.random=g.seed;let own=field(&mut g,"JC125",seat);let other=field(&mut g,"JC125",(seat+1)%4);play(&mut g,seat);assert!(has(&g,&own,"吸血鬼"));assert!(has(&g,&other,"人类"));assert_eq!(g.seed,9_007_199_254_740_993);assert_eq!(g.random,g.seed);restore(&mut g);}
}

#[test]
fn batch4_manor_same_gold_name_uses_existing_uniqueness_and_never_stacks_conversion_into_slave(){
 let mut g=game();let human=field(&mut g,"JC125",0);play(&mut g,0);play(&mut g,0);drain(&mut g);
 assert_eq!(g.attachments.iter().filter(|a|a.card.definition=="XQ15").count(),1);assert!(has(&g,&human,"吸血鬼"));assert!(!has(&g,&human,"奴仆"));restore(&mut g);
}

#[test]
fn batch4_manor_actual_dual_type_bat_attachment_keeps_owned_slave_type_visible_to_four_seats(){
 let mut g=game();let host=field(&mut g,"JC029",0);fund(&mut g,0,"JC029",1);let h=held(&mut g,"BQ026",0);
 act(&mut g,0,Action{card_id:Some(h),region:Some(0),..Action::new("deploy")});
 for _ in 0..64{if g.pending.is_some(){break;}pass(&mut g);}choose(&mut g,vec![host]);top(&mut g);
 let bat=g.attachments.iter().find(|a|a.card.definition=="BQ026").unwrap().card.id.clone();play(&mut g,0);assert!(has(&g,&bat,"奴仆"));
 for seat in 0..4{let v=g.view(seat);let card=&v.attachments.iter().find(|a|a.card.instance_id==bat).unwrap().card;assert!(card.current_subtypes.as_ref().unwrap().iter().any(|s|s=="奴仆"));}restore(&mut g);
}

#[test]
fn batch4_manor_owned_warden_still_emits_original_leave_after_becoming_vampire(){
 let mut g=game();g.players[0].society_zone.loss_control=3;let w=field(&mut g,"BQ053",0);play(&mut g,0);assert!(has(&g,&w,"吸血鬼"));
 g.begin_window(Window::Action(1));fund(&mut g,2,"JC104",2);let spell=held(&mut g,"JC102",2);act(&mut g,2,Action{card_id:Some(spell),target_id:Some(w.clone()),..Action::new("play")});
 for _ in 0..64{if g.pending.is_some(){break;}pass(&mut g);}assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration:d,..}if d.ability.key==crate::batch4_warden::LEAVE));
 choose(&mut g,vec!["accept".into()]);top(&mut g);assert_eq!(g.players[0].society_zone.loss_control,2);assert!(g.board(&w).is_none());restore(&mut g);
}
