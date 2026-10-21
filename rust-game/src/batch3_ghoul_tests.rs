use crate::{model::*,rules::*,unit_support::*};
fn rewards(g:&mut Game)->usize{
    let mut accepted=0;
    for _ in 0..128 {
        if let Some(p)=&g.pending {
            assert!(matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.event==Some(Event::BQ079CorpseGrave)));
            choose(g,vec!["accept".into()]);accepted+=1;
        } else if !g.stack.is_empty(){pass(g);}else if !g.effects.is_empty(){g.drive().unwrap();}else{restore(g);return accepted;}
    }panic!("grave rewards must terminate");
}
fn grave(g:&mut Game,print:&str,owner:usize)->String{let c=g.make_card(print,owner);let id=c.id.clone();g.players[owner].graveyard.push(c);id}
fn begin(g:&mut Game,source:&str){g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();choose(g,vec![source.into()]);}
fn pending(g:&mut Game){for _ in 0..32{if g.pending.is_some(){restore(g);return;}pass(g);}panic!("missing pending");}
#[test]fn batch3_bq079_paid_deploy_then_actual_discard_and_mill_grant_independent_permanent_icons(){
    let mut g=game();fund(&mut g,0,"JC084",2);let id=held(&mut g,"BQ079",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});top(&mut g);
    let id=g.regions[0].cards[0].id.clone();let corpse=held(&mut g,"XQ46",0);g.discard_hand_cards(0,&[corpse]).unwrap();assert_eq!(rewards(&mut g),1);
    let c=g.make_card("XQ46",0);g.players[0].deck.insert(0,c);g.mill_cards_with_cause(0,1,None).unwrap();assert_eq!(rewards(&mut g),1);
    let c=g.board(&id).unwrap().1;assert_eq!((c.permanent_combat_markers,c.permanent_defense_markers),(2,2));assert_eq!(g.current_icons(c,0).combat,3);assert_eq!(g.defense(c,0),3);
    g.board_mut(&id).unwrap().permanent_combat_markers+=1;assert_eq!(g.defense(g.board(&id).unwrap().1,0),3);restore(&mut g);
}
#[test]fn batch3_bq079_grave_owner_selects_observers_not_hand_holder_or_previous_controller(){
    let mut g=game();let own=field(&mut g,"BQ079",0);let ally=field(&mut g,"BQ079",1);let enemy=field(&mut g,"BQ079",2);
    let mut corpse=g.make_card("XQ46",0);let id=corpse.id.clone();corpse.controller=2;g.players[2].hand.push(corpse);g.discard_hand_cards(2,&[id]).unwrap();assert_eq!(rewards(&mut g),1);
    assert_eq!(g.board(&own).unwrap().1.permanent_defense_markers,1);assert_eq!(g.board(&ally).unwrap().1.permanent_defense_markers,0);assert_eq!(g.board(&enemy).unwrap().1.permanent_defense_markers,0);
    g.board_mut(&enemy).unwrap().controller=0;let corpse=held(&mut g,"XQ46",0);g.discard_hand_card(0,&corpse).unwrap();assert_eq!(rewards(&mut g),2);restore(&mut g);
}
#[test]fn batch3_bq079_real_public_and_hidden_death_observe_grave_arrival_without_invented_death(){
    for hidden in [false,true]{let mut g=game();let source=field(&mut g,"BQ079",0);let corpse=field(&mut g,"XQ46",0);g.board_mut(&corpse).unwrap().face_down=hidden;g.remove_dead(&corpse,RemovalCause::Destroy);
        assert_eq!(rewards(&mut g),1);assert_eq!(g.board(&source).unwrap().1.permanent_defense_markers,1);assert!(g.players[0].graveyard.iter().any(|c|c.definition=="XQ46" && c.id!=corpse));}
}
#[test]fn batch3_bq079_simultaneous_sources_and_corpse_capture_before_either_removal_order(){
    for reverse in [false,true]{let mut g=game();let source=field(&mut g,"BQ079",0);let survivor=field(&mut g,"BQ079",0);let corpse=field(&mut g,"XQ46",0);
        let mut ids=vec![source.clone(),corpse];if reverse{ids.reverse();}let deaths=ids.iter().map(|id|{let(r,c)=g.board(id).unwrap();(id.clone(),RemovalCause::Lethal,g.source_snapshot(c,Some(r)))}).collect();g.remove_death_batch(deaths);
        assert_eq!(rewards(&mut g),2);assert_eq!(g.board(&survivor).unwrap().1.permanent_defense_markers,1);assert!(g.players[0].graveyard.iter().filter(|c|c.definition=="BQ079").all(|c|c.permanent_defense_markers==0));}
}
#[test]fn batch3_bq079_snapshot_survives_corpse_exit_but_never_retargets_a_replaced_source(){
    let mut g=game();let source=field(&mut g,"BQ079",0);let corpse=held(&mut g,"XQ46",0);g.discard_hand_card(0,&corpse).unwrap();
    let c=g.players[0].graveyard.remove(0);let c=g.reset_zone_card(c);g.players[0].hand.push(c);
    let hidden=g.hide_in_place_successor(&source).unwrap();let replacement=field(&mut g,"BQ079",0);assert_eq!(rewards(&mut g),1);assert_eq!(g.board(&replacement).unwrap().1.permanent_defense_markers,0);assert_eq!(g.board(&hidden.id).unwrap().1.permanent_defense_markers,0);
}
#[test]fn batch3_bq079_duplicate_observation_owner_tampering_and_pending_metadata_are_rejected(){
    let mut g=game();field(&mut g,"BQ079",0);let corpse=held(&mut g,"XQ46",0);g.discard_hand_card(0,&corpse).unwrap();
    let d=g.effects.iter().find_map(|e|if let Effect::Declare{declaration}=e{Some(declaration.clone())}else{None}).unwrap();let mut bad=g.clone();bad.effects.push_back(Effect::Declare{declaration:d});assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut bad=g.clone();if let Effect::Declare{declaration}=bad.effects.front_mut().unwrap(){if let Op::BQ079CorpseIcons{observation:Some(o)}=&mut declaration.ability.ops[0]{o.arrived.owner=1;}}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    g.drive().unwrap();restore(&mut g);let mut bad=g.clone();bad.pending.as_mut().unwrap().choice.min=Some(1);assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    choose(&mut g,vec!["accept".into()]);restore(&mut g);top(&mut g);
}
#[test]fn batch3_bq079_four_seat_markers_are_visible_only_on_original_face_up_character(){
    let mut g=game();let source=field(&mut g,"BQ079",0);let corpse=held(&mut g,"XQ46",0);g.discard_hand_card(0,&corpse).unwrap();rewards(&mut g);
    for seat in 0..4{let v=g.view(seat);let c=v.regions[0].characters.iter().find(|c|c.instance_id==source).unwrap();assert_eq!((c.permanent_combat_markers,c.permanent_defense_markers),(Some(1),Some(1)));assert_eq!(c.defense,Some(2));}
    let hidden=g.hide_in_place_successor(&source).unwrap();for seat in 0..4{let v=g.view(seat);let c=v.regions[0].characters.iter().find(|c|c.instance_id==hidden.id).unwrap();assert_eq!((c.permanent_combat_markers,c.permanent_defense_markers),(None,None));if seat!=0{assert_eq!(c.card_id,None);}}
    g.remove_dead(&hidden.id,RemovalCause::Destroy);let c=g.players[0].graveyard.iter().find(|c|c.definition=="BQ079").unwrap();assert_eq!((c.permanent_combat_markers,c.permanent_defense_markers),(0,0));restore(&mut g);
}
#[test]fn batch3_bq079_forced_prepare_uses_original_controller_and_fresh_bottom_identity(){
    let mut g=game();let source=field(&mut g,"BQ079",0);let old=grave(&mut g,"JC047",0);let old_len=g.players[0].deck.len();begin(&mut g,&source);g.board_mut(&source).unwrap().controller=2;pending(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"batch3-bq079-bottom");let p=g.pending.clone().unwrap();reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),..Action::new("choose")});reject(&mut g,2,Action{choice_id:Some(p.choice.id),selected:Some(vec![old.clone()]),..Action::new("choose")});
    choose(&mut g,vec![old.clone()]);assert!(g.players[0].graveyard.is_empty());assert_eq!(g.players[0].deck.len(),old_len+1);let c=g.players[0].deck.last().unwrap();assert_eq!(c.definition,"JC047");assert_ne!(c.id,old);assert_eq!(c.controller,0);restore(&mut g);
}
#[test]fn batch3_bq079_empty_prepare_sacrifices_only_original_instance(){
    let mut g=game();let source=field(&mut g,"BQ079",0);begin(&mut g,&source);top(&mut g);assert!(g.board(&source).is_none());assert!(g.pending.is_none());assert!(g.players[0].graveyard.iter().any(|c|c.definition=="BQ079"));restore(&mut g);
}
#[test]fn batch3_bq079_normal_noncorpse_arrival_does_not_allocate_observation_or_duplicate_identity(){
    let mut g=game();field(&mut g,"BQ079",0);let id=held(&mut g,"JC047",0);let seq=g.sequence;g.discard_hand_card(0,&id).unwrap();assert_eq!(g.sequence,seq+1);assert!(g.effects.is_empty());restore(&mut g);
    let mut bad=g.clone();bad.players[0].graveyard[0].permanent_defense_markers=1;assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
}
#[test]fn batch3_bq079_real_paid_discard_cost_queues_one_arrival_and_preserves_original_payment(){
    let mut g=game();let ghoul=field(&mut g,"BQ079",0);let source=field(&mut g,"XQ38",0);let target=grave(&mut g,"JC125",0);let corpse=held(&mut g,"XQ46",0);fund(&mut g,0,"JC084",2);
    act(&mut g,0,Action{card_id:Some(source.clone()),ability_id:Some("recover-grave-discard".into()),target_id:Some(target.clone()),cost_selected:Some(vec![corpse.clone()]),..Action::new("activate")});
    assert!(g.players[0].assets.iter().all(|c|c.exhausted));assert!(g.board(&source).unwrap().1.exhausted);assert!(g.players[0].graveyard.iter().any(|c|c.definition=="XQ46" && c.id!=corpse));
    assert_eq!(rewards(&mut g),1);assert_eq!(g.board(&ghoul).unwrap().1.permanent_defense_markers,1);assert!(g.players[0].hand.iter().any(|c|c.definition=="JC125" && c.id!=target));
}
#[test]fn batch3_bq079_canceled_in_transit_corpse_arrives_in_grave_without_a_character_death(){
    let mut g=game();let ghoul=field(&mut g,"BQ079",0);let corpse=held(&mut g,"XQ46",0);act(&mut g,0,Action{card_id:Some(corpse),region:Some(1),..Action::new("deploy")});
    let c=g.make_card("JC064",2);g.attachments.push(Attachment{card:c,host_id:g.regions[1].card.id.clone()});pending(&mut g);
    assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.event==Some(Event::BQ079CorpseGrave)));
    assert_eq!(rewards(&mut g),1);assert_eq!(g.board(&ghoul).unwrap().1.permanent_defense_markers,1);assert!(g.regions[1].cards.is_empty());assert!(g.players[0].graveyard.iter().any(|c|c.definition=="XQ46"));
}
