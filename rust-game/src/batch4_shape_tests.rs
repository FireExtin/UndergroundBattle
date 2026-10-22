use crate::{batch4_shape as shape, catalog, model::*, rules::*, unit_support::*};

fn setup(seat: usize) -> (Game, String, String) {
    let mut g=game(); let host=field(&mut g,"JC029",seat); fund(&mut g,0,"XG12",6);
    let card=held(&mut g,"XG12",0);play(&mut g,&card,0,&host);
    let source=g.attachments.iter().find(|a|a.card.definition=="XG12").unwrap().card.id.clone(); (g,host,source)
}
fn transfer(g:&mut Game,source:&str,host:&str) {
    act(g,0,Action {card_id:Some(source.into()),ability_id:Some(shape::KEY.into()),target_id:Some(host.into()),..Action::new("activate")});
}
fn mobility(g:&mut Game,host:&str) {
    g.begin_window(Window::Mobility); let (r,c)=g.board(host).unwrap();let actor=c.controller;let s=g.mobility_source(c,r);
    g.emit_event(actor,s,Event::ConfrontationStart);g.drive().unwrap();restore(g);
}
fn destination(g:&Game)->(String,usize) {
    let id=g.pending.as_ref().unwrap().choice.options[0].id.clone();let r=id.strip_prefix("region:").unwrap().parse().unwrap();(id,r)
}

#[test]
fn batch4_shape_exact_print_complete_program_and_granted_mobility_cannot_be_static_transplants() {
    let c=catalog::card("XG12");assert_eq!((c.cost,c.kind.as_str(),c.magic.as_str(),c.supported),(2,"attachment","鲜血",false));
    assert_eq!(c.loyalty,["蓝色","蓝色"]);assert_eq!(c.subtypes,["状态","吸血鬼"]);
    let d=shape::definition();let a=d.attachment.as_ref().unwrap();assert_eq!(a.host.subtype.as_deref(),Some("吸血鬼"));
    assert_eq!(a.host_icons.combat,1);assert_eq!(a.host_temporary_icons.unwrap().combat,1);assert!(!a.host.equipment_host);
    for mode in 0..4 {let mut bad=d.clone();match mode {0=>bad.abilities[1].costs=vec![Cost::Assets(1)],1=>bad.abilities[1].targets[0].exclude_attachment_host=false,2=>bad.attachment.as_mut().unwrap().host_icons.combat=2,_=>bad.modifiers.clear()};assert!(crate::batch4::validate_definition("XG12",&bad).is_err());}
    let (mut g,host,_)=setup(0);g.players[0].society_zone.secrecy=1;let proof=g.xg12_mobility_grant(g.board(&host).unwrap().1).unwrap();
    let a=shape::mobility_ability(proof);assert!(crate::rules::validate_ability("JC029",&a).is_ok());
    let mut fake=definition("JC014").clone();fake.abilities.retain(|a|a.event!=Some(Event::ConfrontationStart));fake.abilities.push(a);
    assert!(crate::mobility::validate_definition("JC014",&fake).is_err());
}

#[test]
fn batch4_shape_actual_attach_preserves_permanent_temporary_and_bat_without_requiring_moon() {
    let (mut g,host,_)=setup(0);let c=g.board(&host).unwrap().1;
    assert_eq!(g.current_permanent_combat(c,0),1);assert_eq!(g.current_icons(c,0).combat,3);assert!(!g.has_mobility(c));
    assert!(g.current_subtypes(c).iter().any(|s|s=="蝙蝠"));g.first_team=1;
    assert_eq!(g.current_icons(g.board(&host).unwrap().1,0).combat,1);restore(&mut g);
}

#[test]
fn batch4_shape_moon_and_four_seat_projection_follow_recipient_controller_not_attachment_or_teammate() {
    let (mut g,host,source)=setup(2);g.players[0].society_zone.secrecy=1;g.players[3].society_zone.secrecy=1;
    assert!(!g.has_mobility(g.board(&host).unwrap().1));g.players[2].society_zone.secrecy=1;
    for seat in 0..4 {let v=g.view(seat);let c=v.regions[0].characters.iter().find(|c|c.instance_id==host).unwrap();assert_eq!(c.current_mobility,Some(true));assert_eq!(c.current_mobility_source.as_deref(),Some(source.as_str()));}
    g.players[2].society_zone.secrecy=0;
    for seat in 0..4 {let v=g.view(seat);let c=v.regions[0].characters.iter().find(|c|c.instance_id==host).unwrap();assert!(c.current_mobility.is_none());assert!(c.current_mobility_source.is_none());}restore(&mut g);
    g.players[2].society_zone.secrecy=1;mobility(&mut g,&host);assert_eq!(g.pending.as_ref().unwrap().seat,2);
    let (id,r)=destination(&g);choose(&mut g,vec![id]);top(&mut g);assert_eq!(g.board(&host).unwrap().0,r);restore(&mut g);
}

#[test]
fn batch4_shape_real_transfer_pays_two_without_tapping_and_keeps_the_attachment_uid() {
    let (mut g,old,source)=setup(0);let next=field(&mut g,"JC029",2);g.players[2].society_zone.secrecy=1;
    transfer(&mut g,&source,&next);assert_eq!(g.resources(0),2);top(&mut g);
    let a=g.attachments.iter().find(|a|a.card.id==source).unwrap();assert_eq!(a.host_id,next);assert!(!a.card.exhausted);
    assert!(!g.current_subtypes(g.board(&old).unwrap().1).iter().any(|s|s=="蝙蝠"));assert!(g.has_mobility(g.board(&next).unwrap().1));
    transfer(&mut g,&source,&old);top(&mut g);assert_eq!(g.resources(0),0);assert_eq!(g.attachments.iter().find(|a|a.card.id==source).unwrap().host_id,old);restore(&mut g);
}

#[test]
fn batch4_shape_transfer_rejects_same_host_non_vampire_and_polluted_action_atomically() {
    let (mut g,host,source)=setup(0);let human=field(&mut g,"JC125",0);
    for target in [host,human] {reject(&mut g,0,Action {card_id:Some(source.clone()),ability_id:Some(shape::KEY.into()),target_id:Some(target),..Action::new("activate")});assert_eq!(g.resources(0),4);}
    let next=field(&mut g,"JC029",0);
    reject(&mut g,0,Action {card_id:Some(source),ability_id:Some(shape::KEY.into()),target_id:Some(next),cost_selected:Some(vec!["foreign".into()]),..Action::new("activate")});
}

#[test]
fn batch4_shape_transfer_target_hide_or_original_source_leave_never_retargets_or_refunds() {
    for source_left in [false,true] {
        let (mut g,old,source)=setup(0);let next=field(&mut g,"JC029",0);transfer(&mut g,&source,&next);
        let replacement=if source_left {g.remove_dead(&source,RemovalCause::Destroy);let c=g.make_card("XG12",0);let id=c.id.clone();g.attachments.push(Attachment {card:c,host_id:old.clone()});Some(id)} else {g.hide_in_place(&next);None};
        g.drive().unwrap();restore(&mut g);top(&mut g);assert_eq!(g.resources(0),2);
        let id=replacement.as_deref().unwrap_or(&source);assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().host_id,old);restore(&mut g);
    }
}

#[test]
fn batch4_shape_mobility_frozen_grant_survives_secrecy_and_attachment_loss_and_moves_once() {
    let (mut g,host,source)=setup(0);g.players[0].society_zone.secrecy=1;g.seed=u64::MAX-937;g.random=g.seed;
    mobility(&mut g,&host);let ChoiceResolution::Declare {declaration:d,..}=&g.pending.as_ref().unwrap().resolution else {panic!()};
    assert!(shape::mobility_proof(&d.ability).is_some());let (id,r)=destination(&g);
    g.players[0].society_zone.secrecy=0;g.remove_dead(&source,RemovalCause::Destroy);g.drive().unwrap();restore(&mut g);
    choose(&mut g,vec![id]);top(&mut g);assert_eq!(g.board(&host).unwrap().0,r);assert_eq!(g.seed,u64::MAX-937);
    let s=g.mobility_source(g.board(&host).unwrap().1,r);g.emit_event(0,s,Event::ConfrontationStart);g.drive().unwrap();assert!(g.pending.is_none());restore(&mut g);
}

#[test]
fn batch4_shape_mobility_uses_existing_adjacency_and_martial_law_targets() {
    let (mut g,host,_)=setup(0);g.players[0].society_zone.secrecy=1;
    g.move_on_board_instance(&host,2);
    let blocked=(0..g.regions.len()).find(|r|g.regions_adjacent(2,*r)).unwrap();let card=g.make_card("JC064",2);
    g.attachments.push(Attachment {card,host_id:g.regions[blocked].card.id.clone()});mobility(&mut g,&host);
    let options=&g.pending.as_ref().unwrap().choice.options;assert!(!options.iter().any(|o|o.id==format!("region:{blocked}")));
    assert!(options.iter().all(|o|g.regions_adjacent(2,o.id.strip_prefix("region:").unwrap().parse().unwrap())));restore(&mut g);
}

#[test]
fn batch4_shape_multiple_grants_create_one_mobility_capture_and_departed_recipient_cannot_move_a_replacement() {
    let (mut g,host,_)=setup(0);let c=g.make_card("XG12",0);g.attachments.push(Attachment {card:c,host_id:host.clone()});g.players[0].society_zone.secrecy=1;
    mobility(&mut g,&host);let s=g.mobility_source(g.board(&host).unwrap().1,0);g.emit_event(0,s,Event::ConfrontationStart);
    assert_eq!(g.turn_ability_usage.iter().filter(|u|u.ability_key=="mobility"&&u.source_instance==host).count(),1);
    let (id,_)=destination(&g);choose(&mut g,vec![id]);g.remove_dead(&host,RemovalCause::Destroy);let replacement=field(&mut g,"JC029",0);
    g.drive().unwrap();restore(&mut g);top(&mut g);assert_eq!(g.board(&replacement).unwrap().0,0);restore(&mut g);
}

#[test]
fn batch4_shape_grant_recovery_rejects_false_secrecy_host_identity_or_turn_and_program_stripping() {
    let (mut g,host,_)=setup(0);g.players[0].society_zone.secrecy=1;mobility(&mut g,&host);
    for mode in 0..5 {let mut bad=g.clone();let ChoiceResolution::Declare {declaration:d,..}=&mut bad.pending.as_mut().unwrap().resolution else {panic!()};
        if mode==4 {d.ability=crate::mobility::ability();} else if let Op::XG12GrantedMobility {proof}=&mut d.ability.ops[0] {match mode {0=>proof.secrecy=0,1=>proof.attachment.host_id="foreign".into(),2=>proof.attachment.card.definition="JZ39".into(),_=>proof.turn+=1}}
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    }
    let (id,_)=destination(&g);choose(&mut g,vec![id]);let mut bad=g.clone();bad.stack[0].frame.as_mut().unwrap().steps[0].context=2;
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());restore(&mut g);
}

#[test]
fn batch4_shape_transfer_recovery_rejects_wrong_host_receipt_program_or_source_zone() {
    let (mut g,_,source)=setup(0);let next=field(&mut g,"JC029",0);transfer(&mut g,&source,&next);
    for mode in 0..4 {let mut bad=g.clone();let f=bad.stack[0].frame.as_mut().unwrap();match mode {
        0=>f.source.attachment_host_instance=Some(next.clone()),1=>f.already_paid=vec![PaidCost::Assets(vec!["foreign".into()])],
        2=>f.steps[0].op=Op::JZ39EnemySacrifice,_=>f.source.attachment_host_instance=None,
    };assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());}
}

#[test]
fn batch4_shape_mobility_still_available_after_moving_cannot_capture_twice_in_the_same_turn() {
    let (mut g,host,_)=setup(0);g.players[0].society_zone.secrecy=1;mobility(&mut g,&host);
    let (id,r)=destination(&g);choose(&mut g,vec![id]);top(&mut g);assert!(g.has_mobility(g.board(&host).unwrap().1));
    let s=g.mobility_source(g.board(&host).unwrap().1,r);g.emit_event(0,s,Event::ConfrontationStart);g.drive().unwrap();
    assert!(g.pending.is_none());assert!(g.stack.is_empty());assert_eq!(g.turn_ability_usage.iter().filter(|u|u.ability_key=="mobility"&&u.source_instance==host).count(),1);restore(&mut g);
}
