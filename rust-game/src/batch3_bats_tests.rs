use crate::{catalog, model::*, rules::*, unit_support::*};
fn attached(g: &mut Game, host: &str) -> String {
    let c=g.make_card("BQ026",0); let id=c.id.clone();g.attachments.push(Attachment{card:c,host_id:host.into()});id
}
fn pending(g:&mut Game) { for _ in 0..64 { if g.pending.is_some(){restore(g);return;} if !g.effects.is_empty(){g.drive().unwrap();}else{pass(g);} } panic!("bounded bats choice"); }
fn source(g:&Game,id:&str)->SourceSnapshot { let(r,c)=g.board(id).unwrap();g.source_snapshot(c,Some(r)) }
#[test] fn batch3_bq026_paid_entry_attaches_to_personal_vampire_without_second_character_entry() {
    let mut g=game();let host=field(&mut g,"JC029",0);let ally=field(&mut g,"JC029",1);fund(&mut g,0,"JC029",1);
    let id=held(&mut g,"BQ026",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});pending(&mut g);
    let p=g.pending.clone().unwrap();assert_eq!(p.choice.options.iter().map(|o|o.id.as_str()).collect::<Vec<_>>(),vec![host.as_str()]);assert!(!p.choice.options.iter().any(|o|o.id==ally));
    choose(&mut g,vec![host.clone()]);top(&mut g);assert_eq!(g.attachments.len(),1);assert_eq!(g.attachments[0].host_id,host);assert_eq!(g.resources(0),0);
    assert_eq!(g.effects.iter().filter(|e|matches!(e,Effect::Declare{declaration:d} if d.ability.key=="batch3-bq026-attach")).count(),0);restore(&mut g);
}
#[test] fn batch3_bq026_attached_icons_damage_and_four_seat_projection_are_character_attributes() {
    let mut g=game();let host=field(&mut g,"JC029",0);let bat=attached(&mut g,&host);let c=g.board(&bat).unwrap().1;
    assert_eq!(g.current_icons(c,0).combat,1);assert_eq!(g.defense(c,0),1);assert_eq!(g.region_characters(0).filter(|c|c.id==bat).count(),1);
    for seat in 0..4 {let v=g.view(seat);let a=v.attachments.iter().find(|a|a.card.instance_id==bat).unwrap();assert_eq!(a.card.defense,Some(1));assert_eq!(a.card.icons.unwrap().combat,1);assert_eq!(a.card.current_mobility,None);let h=v.regions[0].characters.iter().find(|c|c.instance_id==host).unwrap();assert_eq!(h.current_enemy_spell_protection,Some(true));}
    g.damage([(bat.clone(),1)].into_iter().collect()).unwrap();assert!(g.board(&bat).is_none());assert!(g.board(&host).is_some());assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="BQ026").count(),1);restore(&mut g);
}
#[test] fn batch3_bq026_attached_cannot_move_independently_but_follows_actual_host_move_and_emits_entry_once() {
    let mut g=game();let host=field(&mut g,"JC029",0);let bat=attached(&mut g,&host);assert!(!g.has_mobility(g.board(&bat).unwrap().1));g.move_on_board_instance(&bat,1);assert_eq!(g.board(&bat).unwrap().0,0);
    g.move_on_board_instance(&host,1);assert_eq!(g.board(&bat).unwrap().0,1);assert_eq!(g.attachments[0].host_id,host);
    assert_eq!(g.effects.iter().filter(|e|matches!(e,Effect::Declare{declaration:d} if d.source.card.id==bat && d.ability.key=="batch3-bq026-attach")).count(),1);restore(&mut g);
}
#[test] fn batch3_bq026_both_target_roles_work_in_board_and_grave_without_asset_or_hidden_front_leaks() {
    let mut g=game();let bat=field(&mut g,"BQ026",0);let s=source(&g,&bat);
    for kind in [EntityKind::Character,EntityKind::Attachment] {let t=target(Zone::Board,kind,Relation::Any,Range::Anywhere);assert!(g.valid_binding(0,&s,&t,&bat));assert_eq!(g.public_target(0,&s,&t,&bat).kind,if kind==EntityKind::Attachment{"attachment"}else{"character"});}
    let t=target(Zone::AttachmentOrAsset,EntityKind::Any,Relation::Any,Range::Anywhere);assert!(g.valid_binding(0,&s,&t,&bat));
    let hidden=g.hide_in_place_successor(&bat).unwrap();assert!(!g.valid_binding(0,&s,&target(Zone::Board,EntityKind::Attachment,Relation::Any,Range::Anywhere),&hidden.id));restore(&mut g);
}
#[test] fn batch3_bq026_host_protection_only_blocks_enemy_spell_sources_and_rechecks_after_response() {
    let mut g=game();let host=field(&mut g,"JC029",0);let spell=g.make_card("JC062",2);let ss=g.source_snapshot(&spell,Some(0));let skill=field(&mut g,"BQ042",2);let skill=source(&g,&skill);let t=target(Zone::Board,EntityKind::Character,Relation::Any,Range::Anywhere);
    assert!(g.valid_binding(2,&ss,&t,&host));let bat=attached(&mut g,&host);assert!(!g.valid_binding(2,&ss,&t,&host));assert!(g.valid_binding(2,&skill,&t,&host));assert!(g.valid_binding(1,&ss,&t,&host));assert!(!g.has_barrier(g.board(&host).unwrap().1));
    g.remove_dead(&bat,RemovalCause::Destroy);assert!(g.valid_binding(2,&ss,&t,&host));restore(&mut g);
}
#[test] fn batch3_bq026_non_death_host_departure_destroys_bat_as_character_and_cleans_its_attachments() {
    let mut g=game();field(&mut g,"WM035",0);let host=field(&mut g,"JC029",0);let bat=attached(&mut g,&host);let c=g.make_card("JZ41",0);g.attachments.push(Attachment{card:c,host_id:bat.clone()});assert_eq!(g.region_attachment_cards(0).count(),2);
    assert!(g.hide_in_place_successor(&host).is_some());assert!(g.board(&bat).is_none());assert!(g.attachments.is_empty());assert!(g.players[0].graveyard.iter().any(|c|c.definition=="BQ026"));assert!(g.players[0].graveyard.iter().any(|c|c.definition=="JZ41"));restore(&mut g);
}
#[test] fn batch3_bq026_simultaneous_host_and_bat_death_has_two_actual_receipts_in_either_input_order() {
    for reverse in [false,true] {let mut g=game();let host=field(&mut g,"JC029",0);let bat=attached(&mut g,&host);let mut deaths=vec![(host.clone(),RemovalCause::Lethal,source(&g,&host)),(bat.clone(),RemovalCause::Lethal,source(&g,&bat))];if reverse{deaths.reverse();}let receipts=g.remove_death_batch_with_receipts(deaths);assert_eq!(receipts.len(),2);assert_eq!(g.players[0].graveyard.len(),2);assert!(g.attachments.is_empty());restore(&mut g);}
}
#[test] fn batch3_bq026_host_only_death_expands_wave_before_observers_and_uses_distinct_fresh_entities() {
    let mut g=game();field(&mut g,"JC045",0);let host=field(&mut g,"JC029",0);let bat=attached(&mut g,&host);let receipts=g.remove_death_batch_with_receipts(vec![(host.clone(),RemovalCause::Destroy,source(&g,&host))]);assert_eq!(receipts.len(),2);assert_eq!(g.effects.iter().filter(|e|matches!(e,Effect::Declare{declaration:d} if d.source.card.definition=="JC045")).count(),2);assert!(g.players[0].graveyard.iter().all(|c|c.id!=host && c.id!=bat));restore(&mut g);
}
#[test] fn batch3_bq026_winning_region_preserves_assigned_destinations_for_attached_character_and_nested_attachment() {
    let mut g=game();let host=field(&mut g,"JC029",0);let bat=attached(&mut g,&host);let c=g.make_card("JZ41",0);let child=c.id.clone();g.attachments.push(Attachment{card:c,host_id:bat.clone()});g.prepare_region_return(0).unwrap();let b=g.region_return.as_mut().unwrap();for i in 0..4{b.orders[i]=Some(b.bottom[i].clone());}g.commit_region_return(0).unwrap();assert!(g.attachments.is_empty());assert!(g.players[0].graveyard.is_empty());assert!(g.board(&host).is_none());assert!(g.board(&bat).is_none());assert!(g.board(&child).is_none());restore(&mut g);
}
#[test] fn batch3_bq026_forged_cycles_duplicate_events_and_nonreviewed_dual_definitions_are_rejected() {
    let mut g=game();let a=field(&mut g,"BQ026",0);g.emit_event(0,source(&g,&a),Event::EnterRegion);let e=g.effects.front().unwrap().clone();g.effects.push_back(e);assert!(Game::from_persisted(&serde_json::to_string(&g).unwrap()).is_err());
    let mut g=game();let c=g.make_card("BQ026",0);let id=c.id.clone();g.attachments.push(Attachment{card:c,host_id:id});assert_eq!(g.attachment_region(&g.attachments[0]),None);assert!(Game::from_persisted(&serde_json::to_string(&g).unwrap()).is_err());assert_eq!(catalog::card("BQ026").additional_kinds,vec!["attachment"]);assert!(!catalog::card("BQ026").supported);
}
#[test] fn batch3_bq026_printed_public_mobility_and_equipment_restriction_are_live_and_not_just_catalog_words() {
    let mut g=game();let bat=field(&mut g,"BQ026",0);assert!(g.current_public(g.board(&bat).unwrap().1,0));assert!(g.has_mobility(g.board(&bat).unwrap().1));
    let c=g.make_card("JC036",2);let s=g.source_snapshot(&c,Some(0));let mut spec=target(Zone::Board,EntityKind::Character,Relation::Any,Range::Anywhere);spec.equipment_host=true;
    assert!(!g.valid_binding(2,&s,&spec,&bat));assert!(CardFilter::Kind("attachment".into()).matches(catalog::card("BQ026")));restore(&mut g);
}
#[test] fn batch3_bq026_mass_hide_assigns_both_character_destinations_before_host_cleanup() {
    let mut g=game();let host=field(&mut g,"JC029",0);let bat=attached(&mut g,&host);g.world_hide(None,false);
    assert!(g.attachments.is_empty());assert!(g.players[0].graveyard.is_empty());assert_eq!(g.regions[0].cards.iter().filter(|c|c.face_down).count(),2);assert!(g.board(&bat).is_none());assert!(g.board(&host).is_none());restore(&mut g);
}
#[test] fn batch3_bq026_turn_bonuses_and_permanent_markers_survive_attached_character_roundtrip_then_reset_on_hide() {
    let mut g=game();let host=field(&mut g,"JC029",0);let bat=attached(&mut g,&host);g.board_mut(&bat).unwrap().permanent_combat_markers=2;g.grant_execution_traits(bat.clone(),1,true);
    g.turn_attribute_modifiers.push(TurnAttributeModifier{target_instance:bat.clone(),defense_bonus:1,kill_bonus:0,grants_retreat:false,printed_defense_override:None,ordinary_icons:Icons{combat:1,..Icons::default()},grants_renown:true,grants_barrier:false,prevents_damage:false,expires_turn:g.turn});g.settle_deaths();
    let c=g.board(&bat).unwrap().1;assert_eq!(g.current_icons(c,0).combat,4);assert_eq!(g.defense(c,0),2);assert_eq!(g.effective_kill(c),1);assert!(g.has_renown(c));restore(&mut g);
    let h=g.hide_in_place_successor(&bat).unwrap();assert_eq!(h.permanent_combat_markers,0);g.settle_deaths();assert!(g.turn_attribute_modifiers.iter().all(|m|m.target_instance!=bat));restore(&mut g);
}
#[test] fn batch3_bq026_region_combat_damage_choice_includes_attached_character_and_counts_icons_once() {
    let mut g=game();let host=field(&mut g,"JC029",0);let bat=attached(&mut g,&host);let before=g.contest_counts(0,1)[0];assert_eq!(before,g.icons(g.board(&host).unwrap().1,0).combat+1);
    g.effects.push_back(Effect::Damage{seat:2,region:0,amount:1});g.drive().unwrap();let p=g.pending.clone().unwrap();assert!(p.choice.options.iter().any(|o|o.id==bat));
    act(&mut g,p.seat,Action{choice_id:Some(p.choice.id),allocations:Some([(bat.clone(),1)].into_iter().collect()),..Action::new("choose")});assert!(g.board(&bat).is_none());restore(&mut g);
}
#[test] fn batch3_bq026_entered_region_trigger_cannot_follow_moved_or_hidden_source_or_replacement_region() {
    for change in 0..3 {let mut g=game();let host=field(&mut g,"JC029",0);let bat=field(&mut g,"BQ026",0);g.emit_event(0,source(&g,&bat),Event::EnterRegion);g.drive().unwrap();choose(&mut g,vec![host]);
        match change {0=>{g.move_on_board_instance(&bat,1);g.effects.clear();},1=>{g.hide_in_place_successor(&bat);},_=>{let new=g.make_card("DQJC115",0);g.regions[0].card=new;}}
        top(&mut g);assert!(g.attachments.iter().all(|a|a.card.definition!="BQ026"));restore(&mut g);
    }
}
#[test] fn batch3_bq026_actual_attach_notifies_existing_attachment_observer_exactly_once() {
    let mut g=game();let host=field(&mut g,"JC029",0);field(&mut g,"BQ030",0);g.players[0].deck=(0..3).map(|_|g.make_card("JC125",0)).collect();let bat=field(&mut g,"BQ026",0);g.emit_event(0,source(&g,&bat),Event::EnterRegion);g.drive().unwrap();choose(&mut g,vec![host]);
    for _ in 0..64 {if let Some(p)=g.pending.clone(){assert!(matches!(&p.resolution,ChoiceResolution::Declare{declaration:d,..} if d.source.card.definition=="BQ030"));choose(&mut g,vec![p.choice.options[0].id.clone()]);}else if !g.stack.is_empty(){pass(&mut g);}else if !g.effects.is_empty(){g.drive().unwrap();}else{break;}}
    assert_eq!(g.players[0].hand.len(),1);assert_eq!(g.attachments.len(),1);restore(&mut g);
}
#[test] fn batch3_bq026_xg26_real_barrier_targets_both_dual_containers_and_never_exposes_a_hidden_successor() {
    for on_host in [false,true] {let mut g=game();let host=field(&mut g,"JC029",0);let bat=if on_host{attached(&mut g,&host)}else{field(&mut g,"BQ026",0)};let source=field(&mut g,"XG26",0);fund(&mut g,0,"JC125",1);
        act(&mut g,0,Action{card_id:Some(source),ability_id:Some("batch3-xg26-barrier".into()),target_id:Some(bat.clone()),..Action::new("activate")});top(&mut g);assert!(g.has_barrier(g.board(&bat).unwrap().1));
        for seat in 0..4{let v=g.view(seat);let c=v.attachments.iter().find(|a|a.card.instance_id==bat).map(|a|&a.card).or_else(||v.regions[0].characters.iter().find(|c|c.instance_id==bat)).unwrap();assert_eq!(c.current_barrier,Some(true));}
        let h=g.hide_in_place_successor(&bat).unwrap();g.settle_deaths();for seat in 0..4{let v=g.view(seat);let c=v.regions[0].characters.iter().find(|c|c.instance_id==h.id).unwrap();assert_eq!(c.current_barrier,None);assert_eq!(c.current_enemy_spell_protection,None);}restore(&mut g);
    }
}
#[test] fn batch3_bq026_jc044_real_time_payment_accepts_attached_dual_character_receipt_once() {
    let mut g=game();let host=field(&mut g,"JC029",0);let bat=attached(&mut g,&host);g.board_mut(&bat).unwrap().time_markers=1;let source=field(&mut g,"JC044",0);g.regions[1].influence[1]=1;
    act(&mut g,0,Action{card_id:Some(source),target_id:Some("p2".into()),cost_selected:Some(vec![bat.clone()]),..Action::new("activate")});pending(&mut g);choose(&mut g,vec!["region:1".into()]);assert_eq!(g.board(&bat).unwrap().1.time_markers,0);assert_eq!(g.regions[1].influence[1],0);restore(&mut g);
}
