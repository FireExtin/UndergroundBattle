//! Native layouts are explicit. Commands use real payment/response paths and
//! every step compares the complete state and all four player projections.
use crate::{batch4_vampires as v, catalog, model::*, rules::*, unit_support::*};
use crate::jc029_tests::pass_top;

fn grave(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat); let uid = c.id.clone(); g.players[seat].graveyard.push(c); uid
}
fn feed(source: &str, target: &str) -> Action {
    Action { card_id: Some(source.into()), target_id: Some(target.into()), ability_id: Some(v::FEED_TARGET.into()), ..Action::new("activate") }
}
fn actual_reveal(g: &mut Game, card: &str) -> String {
    fund(g, 0, card, catalog::card(card).cost as usize + 1);
    let id = held(g, card, 0);
    act(g, 0, Action { card_id: Some(id), region: Some(0), ..Action::new("conceal") });
    let hidden = g.regions[0].cards.iter().find(|c| c.definition == card && c.face_down).unwrap().id.clone();
    act(g, 0, Action { card_id: Some(hidden), ..Action::new("reveal") });
    pass_top(g);
    g.regions[0].cards.iter().find(|c| c.definition == card && !c.face_down).unwrap().id.clone()
}
fn accept(g: &mut Game, key: &str, selected: &str) {
    for _ in 0..20 {
        let p = g.pending.clone().expect("expected trigger");
        match &p.resolution {
            ChoiceResolution::Batch3EntryOrder { .. } => {
                let id = p.choice.options.iter().find(|o| o.id.ends_with(key)).unwrap().id.clone(); choose(g, vec![id]);
            }
            ChoiceResolution::Declare { declaration, .. } if declaration.ability.key == key => { choose(g, vec![selected.into()]); return; }
            ChoiceResolution::Declare { .. } => choose(g, vec![]),
            _ => panic!("unexpected {}", p.choice.kind),
        }
    }
    panic!("trigger not found");
}
fn skip_declarations(g: &mut Game) {
    for _ in 0..30 {
        let Some(p) = g.pending.clone() else { return; };
        match p.resolution {
            ChoiceResolution::Batch3EntryOrder { .. } => choose(g, vec![p.choice.options[0].id.clone()]),
            ChoiceResolution::Declare { .. } => choose(g, vec![]),
            _ => return,
        }
    }
    panic!("unbounded declarations");
}
fn pending_feed(g: &mut Game) { skip_declarations(g); pass_top(g); assert!(matches!(g.pending.as_ref().unwrap().resolution, ChoiceResolution::Frame { choice: FrameChoice::VampireBlood, .. })); }
fn rejected_both(g: &Game, bad: &Game) {
    let raw = serde_json::to_string(bad).unwrap();
    assert!(std::panic::catch_unwind(|| Game::from_persisted(&raw)).unwrap().is_err());
    let mut room = serde_json::to_value(crate::jc029_tests::envelope(g)).unwrap(); room["game"] = serde_json::to_value(bad).unwrap();
    assert!(std::panic::catch_unwind(|| crate::room::RoomEnvelope::from_persisted(&room.to_string())).unwrap().is_err());
}
fn attachment(g: &mut Game, card: &str, actor: usize, host: &str) -> String {
    fund(g, actor, card, catalog::card(card).cost as usize);
    let id = held(g, card, actor);
    act(g, actor, Action { card_id: Some(id), target_id: Some(host.into()), ..Action::new("play") });
    pass_top(g);
    g.attachments.iter().find(|a| a.card.definition == card && a.host_id == host).unwrap().card.id.clone()
}

#[test]
fn batch4_vampires_partial_sources_are_closed_and_not_deck_admitted() {
    for (id, cost, unique, abilities) in [("BQ027",1,false,1),("BQ031",3,true,2),("BQ032",3,false,2),("BQ036",5,true,1),("BQ038",1,false,2)] {
        let c = catalog::card(id); assert_eq!((c.cost,c.unique,c.supported),(cost,unique,false)); assert_eq!(c.color,"蓝");
        assert_eq!(definition(id).abilities.len(),abilities);
        let mut defs=definitions().clone();defs.get_mut(id).unwrap().abilities[0].timing=Timing::ActionFast;assert!(validate_definitions(&defs).is_err());
    }
    assert_eq!(catalog::card("BQ031").subtitle.as_deref(),Some("红色獠牙"));
    assert_eq!(catalog::card("BQ036").subtitle.as_deref(),Some("大判官"));
    assert_eq!(catalog::card("BQ036").permanent_icons.investigation,2);
    assert_eq!(catalog::card("BQ032").temporary_icons.influence,1);
    assert!(!catalog::card("BQ027").text.contains("地区势力"));
    let a=definition("BQ027").abilities[0].clone();assert!(validate_ability("JC125",&a).is_err());
}

#[test]
fn batch4_vampires_bq027_actual_deploy_sacrifices_once_and_current_recipient_controls_blood() {
    let mut g=game();fund(&mut g,0,"BQ027",1);let source=held(&mut g,"BQ027",0);
    act(&mut g,0,Action{card_id:Some(source),region:Some(0),..Action::new("deploy")});pass_top(&mut g);
    let source=g.regions[0].cards[0].id.clone();let target=field(&mut g,"JC029",1);
    let own=grave(&mut g,"XQ14",0);let enemy=grave(&mut g,"BQ040",1);
    act(&mut g,0,feed(&source,&target));assert!(g.board(&source).is_none());assert_eq!(g.resources(0),0);
    assert!(matches!(g.stack.last().unwrap().frame.as_ref().unwrap().already_paid.as_slice(),[PaidCost::Sacrificed{old_instance,..}] if old_instance==&source));
    pending_feed(&mut g);assert_eq!(g.pending.as_ref().unwrap().seat,1);assert!(!g.pending.as_ref().unwrap().choice.options.iter().any(|o|o.id==own));
    choose(&mut g,vec![enemy.clone()]);assert!(g.attachments.iter().any(|a|a.host_id==target&&a.card.controller==1&&a.card.id!=enemy));
    reject(&mut g,0,feed(&source,&target));restore(&mut g);
}

#[test]
fn batch4_vampires_bq027_targeting_self_cancels_after_sacrifice_without_refund() {
    let mut g=game();let source=field(&mut g,"BQ027",0);grave(&mut g,"XQ14",0);
    act(&mut g,0,feed(&source,&source));pass_top(&mut g);assert!(g.pending.is_none());assert!(g.attachments.is_empty());
    assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="BQ027").count(),1);
}

#[test]
fn batch4_vampires_bq027_invalid_payload_or_nonvampire_never_pays() {
    let mut g=game();let source=field(&mut g,"BQ027",0);let invalid=field(&mut g,"JC125",0);let target=field(&mut g,"JC029",0);
    reject(&mut g,0,feed(&source,&invalid));let mut a=feed(&source,&target);a.cost_selected=Some(vec![target.clone()]);reject(&mut g,0,a);
    let mut a=feed(&source,&target);a.option=Some("extra".into());reject(&mut g,0,a);assert!(g.board(&source).is_some());
}

#[test]
fn batch4_vampires_bq027_departure_and_control_response_keep_original_target() {
    let mut g=game();let source=field(&mut g,"BQ027",0);let target=field(&mut g,"JC029",0);let blood=grave(&mut g,"XQ14",1);
    act(&mut g,0,feed(&source,&target));g.board_mut(&target).unwrap().controller=1;pending_feed(&mut g);assert_eq!(g.pending.as_ref().unwrap().seat,1);choose(&mut g,vec![blood]);
    let mut g=game();let source=field(&mut g,"BQ027",0);let target=field(&mut g,"JC029",0);grave(&mut g,"XQ14",0);act(&mut g,0,feed(&source,&target));
    g.remove_dead(&target,RemovalCause::Destroy);field(&mut g,"JC029",0);pass_top(&mut g);assert!(g.pending.is_none());assert!(g.attachments.is_empty());
}

#[test]
fn batch4_vampires_bq032_reveal_feeds_zero_one_or_two_mixed_sources_simultaneously() {
    for n in 0..=2 { let mut g=game();let first=grave(&mut g,"XQ14",0);let c=g.make_card("BQ040",1);g.seal_card_on_valid_host(c,0,"society:p0");let second=g.sealed_cards[0].card.id.clone();
        let source=actual_reveal(&mut g,"BQ032");assert_eq!(g.resources(0),0);accept(&mut g,v::REVEAL_FEED,"accept");pending_feed(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().choice.min,Some(0));assert_eq!(g.pending.as_ref().unwrap().choice.max,Some(2));
        choose(&mut g,vec![first,second].into_iter().take(n).collect());assert_eq!(g.attachments.len(),n);assert!(g.attachments.iter().all(|a|a.host_id==source&&a.card.controller==0));
        if n==2 {g.remove_dead(&source,RemovalCause::Destroy);assert!(g.players[1].graveyard.iter().any(|c|c.definition=="BQ040"));}restore(&mut g);
    }
}

#[test]
fn batch4_vampires_bq032_normal_faceup_entry_does_not_feed_or_raid() {
    let mut g=game();fund(&mut g,0,"BQ032",3);grave(&mut g,"XQ14",0);let source=held(&mut g,"BQ032",0);
    act(&mut g,0,Action{card_id:Some(source),region:Some(0),..Action::new("deploy")});pass_top(&mut g);
    assert!(g.pending.is_none());assert!(g.stack.is_empty());assert!(g.attachments.is_empty());
}

#[test]
fn batch4_vampires_bq032_short_and_empty_blood_supply_and_hidden_recipient() {
    for n in 0..2 {let mut g=game();for _ in 0..n{grave(&mut g,"XQ14",0);}actual_reveal(&mut g,"BQ032");accept(&mut g,v::REVEAL_FEED,"accept");skip_declarations(&mut g);pass_top(&mut g);
        if n==0{assert!(g.pending.is_none());}else{assert_eq!(g.pending.as_ref().unwrap().choice.max,Some(1));choose(&mut g,vec![]);}}
    let mut g=game();grave(&mut g,"XQ14",0);let source=actual_reveal(&mut g,"BQ032");accept(&mut g,v::REVEAL_FEED,"accept");skip_declarations(&mut g);
    g.hide_in_place_successor(&source).unwrap();pass_top(&mut g);assert!(g.pending.is_none());assert!(g.attachments.is_empty());
}

#[test]
fn batch4_vampires_bq032_only_personal_grave_or_personal_society_blood_is_available() {
    let mut g=game();let legal=grave(&mut g,"XQ14",0);grave(&mut g,"BQ040",2);grave(&mut g,"JC125",0);
    let host=field(&mut g,"JC029",0);let c=g.make_card("XQ14",0);g.seal_card_on_valid_host(c,0,&host);let c=g.make_card("BQ040",0);g.seal_card_on_valid_host(c,2,"society:p2");
    actual_reveal(&mut g,"BQ032");accept(&mut g,v::REVEAL_FEED,"accept");pending_feed(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>(),vec![legal.clone()]);
    for s in 1..4{assert!(g.view(s).pending_choice.is_none());}choose(&mut g,vec![legal]);
}

#[test]
fn batch4_vampires_conditional_nonasset_requires_personal_sun() {
    for id in ["BQ031","BQ032"] {for sun in [false,true] {let mut g=game();let source=held(&mut g,id,0);if sun{g.players[0].society_zone.exposure=1;}else{g.players[2].society_zone.exposure=1;}
        let a=Action{card_id:Some(source),..Action::new("asset")};if sun{reject(&mut g,0,a);}else{act(&mut g,0,a);assert_eq!(g.players[0].assets.len(),1);}}}
}

#[test]
fn batch4_vampires_bq036_actual_search_top_six_private_reveals_only_selected_and_shuffles() {
    let mut g=game();let ids=deck_cards(&mut g,0,&["BQ022","JC125","XQ14","XQ15","XG12","JC020","BQ040"]);actual_reveal(&mut g,"BQ036");accept(&mut g,v::SEARCH,"accept");pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(),4);assert_eq!(g.view(0).pending_choice.unwrap().preview_cards.len(),6);for s in 1..4{assert!(g.view(s).pending_choice.is_none());}
    let old_random=g.random;choose(&mut g,vec![ids[0].clone(),ids[2].clone()]);assert_eq!(g.players[0].hand.len(),2);assert_eq!(g.players[0].deck.len(),5);assert_ne!(g.random,old_random);
    assert!(g.players[0].hand.iter().all(|c|!ids.contains(&c.id)));assert!(g.players[0].deck.iter().any(|c|c.id==ids[6]));
    let revealed=g.log.iter().filter(|l|l.text.contains("展示菲尼克斯搜寻")).map(|l|l.text.clone()).collect::<Vec<_>>();assert_eq!(revealed.len(),2);assert!(revealed[0].contains("合金指虎"));assert!(revealed[1].contains("鲜血"));
}

#[test]
fn batch4_vampires_bq036_zero_all_short_and_empty_choices() {
    for (cards,n) in [(vec![],0),(vec!["JC125"],0),(vec!["BQ022";6],0),(vec!["BQ022";6],6)] {
        let mut g=game();let ids=deck_cards(&mut g,0,&cards);actual_reveal(&mut g,"BQ036");accept(&mut g,v::SEARCH,"accept");pass_top(&mut g);
        assert_eq!(g.view(0).pending_choice.unwrap().preview_cards.len(),cards.len());choose(&mut g,ids.into_iter().take(n).collect());assert_eq!(g.players[0].hand.len(),n);assert_eq!(g.players[0].deck.len(),cards.len()-n);
    }
}

#[test]
fn batch4_vampires_bq036_wrong_seventh_duplicates_and_extra_payload_reject_atomically() {
    let mut g=game();let ids=deck_cards(&mut g,0,&["XQ14","JC125","JC125","JC125","JC125","JC125","BQ040"]);actual_reveal(&mut g,"BQ036");accept(&mut g,v::SEARCH,"accept");pass_top(&mut g);let p=g.pending.clone().unwrap();
    for selected in [vec![ids[6].clone()],vec![ids[0].clone();2],vec![ids[1].clone()]] {reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(selected),..Action::new("choose")});}
    reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),region:Some(2),..Action::new("choose")});
    reject(&mut g,1,Action{choice_id:Some(p.choice.id),selected:Some(vec![]),..Action::new("choose")});
}

#[test]
fn batch4_vampires_bq036_source_departure_does_not_cancel_or_rebind_private_search() {
    let mut g=game();let ids=deck_cards(&mut g,0,&["XQ14"]);let source=actual_reveal(&mut g,"BQ036");accept(&mut g,v::SEARCH,"accept");g.remove_dead(&source,RemovalCause::Destroy);field(&mut g,"BQ036",1);
    pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().seat,0);choose(&mut g,ids);assert_eq!(g.players[0].hand.len(),1);assert!(g.players[1].hand.is_empty());
}

#[test]
fn batch4_vampires_bq031_attachment_trigger_enemy_discards_privately_once_per_turn() {
    let mut g=game();let source=field(&mut g,"BQ031",0);let host=field(&mut g,"JC029",0);held(&mut g,"JC125",2);let chosen=held(&mut g,"JC091",2);
    attachment(&mut g,"XQ14",0,&host);accept(&mut g,v::ATTACH_DISCARD,"p2");pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat,2);for s in [0,1,3]{assert!(g.view(s).pending_choice.is_none());}choose(&mut g,vec![chosen]);
    assert_eq!(g.players[2].hand.len(),1);assert!(g.turn_ability_usage.iter().any(|u|u.source_instance==source&&u.uses==1));
    attachment(&mut g,"BQ040",0,&host);assert!(g.pending.is_none());restore(&mut g);
}

#[test]
fn batch4_vampires_bq031_decline_does_not_consume_and_empty_enemy_hand_still_consumes() {
    let mut g=game();field(&mut g,"BQ031",0);let host=field(&mut g,"JC029",0);attachment(&mut g,"XQ14",0,&host);choose(&mut g,vec![]);assert!(g.turn_ability_usage.is_empty());
    attachment(&mut g,"BQ040",0,&host);accept(&mut g,v::ATTACH_DISCARD,"p3");pass_top(&mut g);assert!(g.pending.is_none());assert_eq!(g.turn_ability_usage.len(),1);
}

#[test]
fn batch4_vampires_bq031_cross_region_foreign_attachment_personal_host_and_original_observer() {
    let mut g=game();let source=field(&mut g,"BQ031",0);let host=field(&mut g,"JC029",0);g.move_on_board_instance(&host,2);held(&mut g,"JC125",2);
    g.begin_window(Window::Action(1));attachment(&mut g,"XQ14",2,&host);accept(&mut g,v::ATTACH_DISCARD,"p2");g.remove_dead(&source,RemovalCause::Destroy);g.remove_dead(&host,RemovalCause::Destroy);
    pass_top(&mut g);assert_eq!(g.pending.as_ref().unwrap().seat,2);let id=g.players[2].hand[0].id.clone();choose(&mut g,vec![id]);
    let mut g=game();field(&mut g,"BQ031",0);let ally=field(&mut g,"JC029",1);attachment(&mut g,"XQ14",0,&ally);assert!(g.pending.is_none());
}

#[test]
fn batch4_vampires_bq031_current_subtype_from_manor_qualifies_but_plain_human_does_not() {
    let mut g=game();field(&mut g,"BQ031",0);let human=field(&mut g,"JC125",0);attachment(&mut g,"BQ022",0,&human);assert!(g.pending.is_none());
    attachment(&mut g,"XQ15",0,"society:p0");attachment(&mut g,"JC116",0,&human);assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key==v::ATTACH_DISCARD));choose(&mut g,vec![]);
}

#[test]
fn batch4_vampires_bq031_and_multiple_arbiters_share_original_ordering_without_lost_declines() {
    let mut g=game();field(&mut g,"BQ031",0);field(&mut g,"BQ030",0);field(&mut g,"BQ030",0);let host=field(&mut g,"JC029",0);held(&mut g,"JC125",2);
    attachment(&mut g,"XQ14",0,&host);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"batch3-entry-order");accept(&mut g,v::ATTACH_DISCARD,"p2");
    skip_declarations(&mut g);pass_top(&mut g);let id=g.players[2].hand[0].id.clone();choose(&mut g,vec![id]);assert!(g.attachment_declines.is_empty());restore(&mut g);
}

#[test]
fn batch4_vampires_bq031_real_reattachment_orders_with_arbiters_and_keeps_uid() {
    let mut g=game();field(&mut g,"BQ031",0);field(&mut g,"BQ030",0);field(&mut g,"BQ030",0);
    let first=field(&mut g,"JC029",0);let second=field(&mut g,"JC029",0);let shape=attachment(&mut g,"XG12",0,&first);skip_declarations(&mut g);
    fund(&mut g,0,"XG12",2);let key=definition("XG12").abilities[1].key.clone();
    act(&mut g,0,Action{card_id:Some(shape.clone()),ability_id:Some(key),target_id:Some(second.clone()),..Action::new("activate")});pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"batch3-entry-order");assert!(g.attachments.iter().any(|a|a.card.id==shape&&a.host_id==second));
    accept(&mut g,v::ATTACH_DISCARD,"p2");skip_declarations(&mut g);pass_top(&mut g);assert!(g.pending.is_none());
}

#[test]
fn batch4_vampires_bq032_two_blood_entries_cannot_activate_bq031_twice() {
    let mut g=game();field(&mut g,"BQ031",0);let ids=vec![grave(&mut g,"XQ14",0),grave(&mut g,"BQ040",0)];held(&mut g,"JC125",2);held(&mut g,"JC125",2);
    actual_reveal(&mut g,"BQ032");accept(&mut g,v::REVEAL_FEED,"accept");pending_feed(&mut g);choose(&mut g,ids);
    accept(&mut g,v::ATTACH_DISCARD,"p2");assert!(g.pending.is_none());pass_top(&mut g);let selected=g.players[2].hand[0].id.clone();choose(&mut g,vec![selected]);
    assert_eq!(g.players[2].hand.len(),1);assert!(g.turn_ability_usage.iter().filter(|u|u.ability_key==v::ATTACH_DISCARD).all(|u|u.uses==1));
}

#[test]
fn batch4_vampires_bq031_host_type_and_bq038_raid_receipts_cannot_be_forged() {
    let mut g=game();field(&mut g,"BQ031",0);let host=field(&mut g,"JC029",0);attachment(&mut g,"XQ14",0,&host);
    for n in 0..4 {let mut bad=g.clone();let ChoiceResolution::Declare{declaration,..}=&mut bad.pending.as_mut().unwrap().resolution else{panic!()};let Op::BQ031AttachmentDiscard{observation:Some(o)}=&mut declaration.ability.ops[0] else{panic!()};
        match n{0=>o.subtypes.push("invented".into()),1=>o.host.card.controller=1,2=>o.turn+=1,_=>o.attachment.host_id=declaration.source.card.id.clone()};rejected_both(&g,&bad);}
    let mut g=game();attachment(&mut g,"BQ038",0,"society:p0");grave(&mut g,"XQ14",0);let target=field(&mut g,"JC125",2);actual_reveal(&mut g,"BQ032");accept(&mut g,"raid-1",&target);
    // BQ038 is pending immediately after the accepted raid; the BQ032 entry
    // trigger is still a separate declaration and is not a second raid.
    while !matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key==v::RAID_FEED){
        if matches!(g.pending.as_ref().unwrap().resolution,ChoiceResolution::Batch3EntryOrder{..}){let id=g.pending.as_ref().unwrap().choice.options[0].id.clone();choose(&mut g,vec![id]);}else{choose(&mut g,vec![]);}
    }
    for n in 0..5 {let mut bad=g.clone();let ChoiceResolution::Declare{declaration,..}=&mut bad.pending.as_mut().unwrap().resolution else{panic!()};let Op::BQ038RaidFeed{observation:Some(o)}=&mut declaration.ability.ops[0] else{panic!()};
        match n{0=>o.raid.actor=1,1=>o.raid.ability_key=v::REVEAL_FEED.into(),2=>o.raid.frame_id=o.raid.source.card.id.clone(),3=>o.turn+=1,_=>o.raid.targets[0].spec.relation=Relation::ControlledByActor};rejected_both(&g,&bad);}
}

#[test]
fn batch4_vampires_bq038_actual_personal_society_raid_accept_feeds_before_damage() {
    let mut g=game();let source=attachment(&mut g,"BQ038",0,"society:p0");let blood=grave(&mut g,"XQ14",0);let target=field(&mut g,"JC125",1);let raider=actual_reveal(&mut g,"BQ032");
    accept(&mut g,"raid-1",&target);accept(&mut g,v::RAID_FEED,"accept");pending_feed(&mut g);assert!(g.board(&target).is_some());choose(&mut g,vec![blood]);
    assert!(g.attachments.iter().any(|a|a.host_id==raider&&a.card.definition=="XQ14"));pass_top(&mut g);assert!(g.board(&target).is_none());assert!(g.attachments.iter().any(|a|a.card.id==source));
}

#[test]
fn batch4_vampires_bq038_revealing_or_declining_raid_does_not_feed() {
    let mut g=game();attachment(&mut g,"BQ038",0,"society:p0");grave(&mut g,"XQ14",0);actual_reveal(&mut g,"BQ032");skip_declarations(&mut g);
    assert!(g.pending.is_none());assert!(g.stack.is_empty());assert_eq!(g.attachments.len(),1);
}

#[test]
fn batch4_vampires_bq038_personal_only_and_source_departure_keep_accepted_raid_receipt() {
    let mut g=game();let source=attachment(&mut g,"BQ038",0,"society:p0");let blood=grave(&mut g,"XQ14",0);let target=field(&mut g,"JC125",1);actual_reveal(&mut g,"BQ032");accept(&mut g,"raid-1",&target);accept(&mut g,v::RAID_FEED,"accept");skip_declarations(&mut g);
    g.remove_dead(&source,RemovalCause::Destroy);pending_feed(&mut g);choose(&mut g,vec![blood]);pass_top(&mut g);restore(&mut g);
    let mut g=game();attachment(&mut g,"BQ038",0,"society:p0");let raider=field(&mut g,"BQ029",2);let target=field(&mut g,"JC125",1);let source=g.source_snapshot(g.board(&raider).unwrap().1,Some(0));
    g.emit_event(2,source,Event::Reveal);g.drive().unwrap();accept(&mut g,"raid-1",&target);assert!(!g.effects.iter().any(|e|matches!(e,Effect::Declare{declaration} if declaration.ability.key==v::RAID_FEED)));assert!(g.pending.is_none());
}

#[test]
fn batch4_vampires_bq038_granted_raid_combines_once_not_each_point() {
    let mut g=game();attachment(&mut g,"BQ038",0,"society:p0");field(&mut g,"JC038",0);grave(&mut g,"XQ14",0);let target=field(&mut g,"JC029",1);actual_reveal(&mut g,"BQ032");
    accept(&mut g,"jc038-granted-raid",&target);accept(&mut g,v::RAID_FEED,"accept");pending_feed(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.max,Some(1));let blood=g.pending.as_ref().unwrap().choice.options[0].id.clone();choose(&mut g,vec![blood]);
    assert_eq!(g.attachments.iter().filter(|a|a.card.definition=="XQ14").count(),1);pass_top(&mut g);
}

#[test]
fn batch4_vampires_frames_receipts_pending_and_duplicate_events_reject_both_recovery_entries() {
    let mut g=game();let source=field(&mut g,"BQ027",0);let target=field(&mut g,"JC029",0);grave(&mut g,"XQ14",0);act(&mut g,0,feed(&source,&target));
    for n in 0..6 {let mut bad=g.clone();let f=bad.stack.last_mut().unwrap().frame.as_mut().unwrap();match n{0=>f.already_paid.clear(),1=>f.source.card.controller=1,2=>f.steps[0].op=Op::BQ036TopSix,3=>f.targets[0].spec.subtype=None,4=>f.cursor=1,_=>f.frame_id=f.source.card.id.clone()};rejected_both(&g,&bad);}
    pending_feed(&mut g);for n in 0..4{let mut bad=g.clone();let p=bad.pending.as_mut().unwrap();match n{0=>p.seat=1,1=>p.choice.min=Some(0),2=>p.choice.options.clear(),_=>if let ChoiceResolution::Frame{choice,..}=&mut p.resolution{*choice=FrameChoice::BQ036TopSix{inspected_ids:vec![]};}}rejected_both(&g,&bad);}
    let mut g=game();field(&mut g,"BQ031",0);let host=field(&mut g,"JC029",0);attachment(&mut g,"XQ14",0,&host);let mut bad=g.clone();let ChoiceResolution::Declare{declaration,..}=&bad.pending.as_ref().unwrap().resolution else{panic!()};bad.effects.push_back(Effect::Declare{declaration:declaration.clone()});rejected_both(&g,&bad);
    accept(&mut g,v::ATTACH_DISCARD,"p2");let mut bad=g.clone();let mut extra=bad.stack.last().unwrap().clone();
    let event=bad.id();let id=bad.id();let f=extra.frame.as_mut().unwrap();let Op::BQ031AttachmentDiscard{observation:Some(o)}=&mut f.steps[0].op else{panic!()};o.event_id=event;f.frame_id=id.clone();extra.id=id;bad.stack.push(extra);rejected_both(&g,&bad);
}

#[test]
fn batch4_vampires_atomic_programs_cannot_recover_as_effect_queue_frames() {
    let mut g=game();let source=field(&mut g,"BQ027",0);let target=field(&mut g,"JC029",0);grave(&mut g,"XQ14",0);act(&mut g,0,feed(&source,&target));
    let mut bad=g.clone();let frame=bad.stack.pop().unwrap().frame.unwrap();bad.effects.push_front(Effect::Frame{frame:Box::new(frame)});rejected_both(&g,&bad);
    pending_feed(&mut g);let mut bad=g.clone();let ChoiceResolution::Frame{frame,..}=bad.pending.take().unwrap().resolution else{panic!()};bad.effects.push_front(Effect::Frame{frame});rejected_both(&g,&bad);
    let mut g=game();actual_reveal(&mut g,"BQ036");accept(&mut g,v::SEARCH,"accept");let mut bad=g.clone();let frame=bad.stack.pop().unwrap().frame.unwrap();bad.effects.push_front(Effect::Frame{frame:Box::new(frame)});rejected_both(&g,&bad);
}

#[test]
fn batch4_vampires_unknown_or_out_of_range_attachment_history_rejects_without_panics() {
    let mut g=game();field(&mut g,"BQ031",0);let host=field(&mut g,"JC029",0);attachment(&mut g,"XQ14",0,&host);
    for n in 0..5 {let mut bad=g.clone();let ChoiceResolution::Declare{declaration,..}=&mut bad.pending.as_mut().unwrap().resolution else{panic!()};let Op::BQ031AttachmentDiscard{observation:Some(o)}=&mut declaration.ability.ops[0] else{panic!()};
        match n{0=>o.attachment.card.definition="unknown".into(),1=>o.attachment.card.owner=99,2=>o.attachment.card.controller=99,3=>o.host.card.owner=99,_=>o.event_id=bad.regions[0].card.id.clone()};rejected_both(&g,&bad);}
}

#[test]
fn batch4_vampires_response_shell_cannot_inject_markers_or_alias_source_uid() {
    for id in ["BQ027","BQ032","BQ036","BQ038"] {
        let mut g=game();fund(&mut g,0,id,catalog::card(id).cost as usize);let held=held(&mut g,id,0);
        let a=if id=="BQ038"{Action{card_id:Some(held),target_id:Some("society:p0".into()),..Action::new("play")}}else{Action{card_id:Some(held),region:Some(0),..Action::new("deploy")}};act(&mut g,0,a);
        for n in 0..3 {let mut bad=g.clone();let item=bad.stack.last_mut().unwrap();let c=item.card.as_mut().unwrap();match n{0=>c.permanent_combat_markers+=1,1=>c.id=item.frame.as_ref().unwrap().source.card.id.clone(),_=>c.exhausted=true};rejected_both(&g,&bad);}
    }
}

fn room_step(r:&mut crate::room::RoomEnvelope,seat:usize,action:crate::room::SessionAction,now:u64)->crate::room::RoomCommand {
    let cmd=crate::room::RoomCommand{command_id:format!("vampire-{}",r.revision),expected_version:r.revision,action};
    let result=r.transition(seat,Some(cmd.clone()),now).unwrap();assert!(result.error_code.is_none(),"{:?}",result.error_message);*r=crate::room::RoomEnvelope::from_persisted(&result.state).unwrap();cmd
}
#[test]
fn batch4_vampires_search_room_pause_resume_replay_large_seed_and_four_private_views() {
    use crate::room::{RoomEnvelope,SessionAction};let mut g=game();g.seed=u64::MAX-93;g.random=g.seed;let ids=deck_cards(&mut g,0,&["BQ022","JC125","XQ14"]);actual_reveal(&mut g,"BQ036");accept(&mut g,v::SEARCH,"accept");
    let mut r=crate::jc029_tests::envelope(&g);room_step(&mut r,1,SessionAction::PauseRoom,1000);room_step(&mut r,2,SessionAction::ResumeRoom,2000);
    for n in 0..16{if r.game.pending.is_some(){break;}let id=r.pacing.window.as_ref().unwrap().id.clone();let seat=(0..4).find(|s|r.game.legal_actions(*s).iter().any(|v|v.action.kind=="pass")).unwrap();room_step(&mut r,seat,SessionAction::PassResponse{window_id:id},3000+n);}
    let choice=r.game.pending.as_ref().unwrap().choice.id.clone();room_step(&mut r,3,SessionAction::PauseRoom,4000);room_step(&mut r,0,SessionAction::ResumeRoom,5000);
    let before=serde_json::to_string(&r).unwrap();let cmd=room_step(&mut r,0,SessionAction::Game{action:Action{choice_id:Some(choice),selected:Some(vec![ids[0].clone()]),..Action::new("choose")}},6000);
    let replay=RoomEnvelope::from_persisted(&before).unwrap().transition(0,Some(cmd.clone()),6000).unwrap();assert_eq!(replay.state,serde_json::to_string(&r).unwrap());
    let duplicate=r.transition(0,Some(cmd),6001).unwrap();assert_eq!(duplicate.error_code.as_deref(),Some("version_conflict"));assert_eq!(duplicate.state,serde_json::to_string(&r).unwrap());
    assert_eq!(r.game.seed,u64::MAX-93);let restored=RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();for seat in 0..4{assert_eq!(serde_json::to_value(r.view(seat,7000)).unwrap(),serde_json::to_value(restored.view(seat,7000)).unwrap());}
}

// These fixtures cover only B-owned read-only inputs to the pending public
// adapters. They do not claim that a contest action or hook has been enabled.
fn contest_attachment(g: &mut Game, id: &str, controller: usize, host: &str) -> String {
    let card=g.make_card(id,controller);let uid=card.id.clone();g.attachments.push(Attachment{card,host_id:host.into()});uid
}
#[test]
fn batch4_vampires_contract_bq036_cost_requires_personal_attachment_and_personal_character() {
    let mut g=game();let own=field(&mut g,"JC029",0);let ally=field(&mut g,"JC029",1);
    let valid=contest_attachment(&mut g,"XQ14",0,&own);contest_attachment(&mut g,"BQ040",1,&own);contest_attachment(&mut g,"BQ040",0,&ally);
    contest_attachment(&mut g,"BQ038",0,"society:p0");
    assert_eq!(g.bq036_contest_cost_candidates(0),[valid.clone()]);
    g.attachments.iter_mut().find(|a|a.card.id==valid).unwrap().card.owner=2;
    assert_eq!(g.bq036_contest_cost_candidates(0),[valid]);
    g.board_mut(&own).unwrap().face_down=true;assert!(g.bq036_contest_cost_candidates(0).is_empty());
}
#[test]
fn batch4_vampires_contract_bq038_counts_characters_once_and_accepts_foreign_attachment_controller() {
    let mut g=game();let human=field(&mut g,"JC125",0);let vampire=field(&mut g,"JC029",0);let ally=field(&mut g,"JC029",1);let bare=field(&mut g,"JC029",0);
    let foreign=contest_attachment(&mut g,"BQ022",2,&human);contest_attachment(&mut g,"XQ47",0,&human);
    contest_attachment(&mut g,"XQ14",0,&vampire);contest_attachment(&mut g,"XQ14",0,&ally);contest_attachment(&mut g,"BQ026",0,&bare);
    let witnesses=g.bq038_contest_count_witnesses(0);assert_eq!(witnesses.len(),2);
    assert_eq!(witnesses[0].character.card.id,human);assert_eq!(witnesses[0].attachment.card.id,foreign);
    assert_eq!(witnesses[1].character.card.id,vampire);assert_eq!(witnesses[0].character.source_region_instance.as_ref(),Some(&g.regions[0].card.id));
    g.board_mut(&human).unwrap().face_down=true;assert_eq!(g.bq038_contest_count_witnesses(0).len(),1);
    g.players[0].eliminated=true;assert!(g.bq038_contest_count_witnesses(0).is_empty());
}
#[test]
fn batch4_vampires_contract_bq032_authenticates_original_hand_without_paying_or_publishing_action() {
    let mut g=game();let id=held(&mut g,"BQ032",0);let wrong=held(&mut g,"BQ027",0);let other=held(&mut g,"BQ032",1);let board=field(&mut g,"BQ032",0);
    let before=serde_json::to_string(&g).unwrap();let source=g.bq032_hand_contest_source(0,&id).unwrap();
    assert_eq!(source.card.id,id);assert!(source.region.is_none()&&source.play_source.is_none());
    assert!(g.bq032_hand_contest_source(0,&wrong).is_err());assert!(g.bq032_hand_contest_source(0,&other).is_err());assert!(g.bq032_hand_contest_source(0,&board).is_err());
    assert!(g.bq032_hand_contest_source(99,&id).is_err());assert_eq!(serde_json::to_string(&g).unwrap(),before);
    assert!(!definition("BQ032").abilities.iter().any(|a|a.event.is_none()));
}
