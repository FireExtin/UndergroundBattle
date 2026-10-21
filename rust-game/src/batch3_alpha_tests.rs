use crate::{model::*,rules::*,unit_support::*};
fn fill(g:&mut Game){g.players[0].deck=(0..8).map(|_|g.make_card("JC125",0)).collect();}
fn ready_reward(g:&mut Game){for _ in 0..32{if g.stack.last().and_then(|s|s.frame.as_ref()).is_some_and(|f|f.ability_key=="batch3-wm041-public-play"){restore(g);return;}if !g.effects.is_empty(){g.drive().unwrap();}else{pass(g);}}panic!("forced reward missing");}
#[test]fn batch3_wm041_renown_is_personal_for_public_characters_and_enemy_denial_overrides_printed_and_temporary_grants(){
    let mut g=game();let source=field(&mut g,"WM041",0);let own=field(&mut g,"XQ46",0);let ally=field(&mut g,"XQ46",1);let enemy=field(&mut g,"JC070",2);
    let temporary=field(&mut g,"JC084",2);g.turn_attribute_modifiers.push(TurnAttributeModifier{target_instance:temporary.clone(),defense_bonus:0,kill_bonus:0,grants_retreat:false,printed_defense_override:None,ordinary_icons:Icons::default(),grants_renown:true,grants_barrier:false,prevents_damage:false,expires_turn:g.turn});assert!(!g.has_renown(g.board(&temporary).unwrap().1));
    assert!(g.has_renown(g.board(&own).unwrap().1));assert!(!g.has_renown(g.board(&ally).unwrap().1));assert!(!g.has_renown(g.board(&enemy).unwrap().1));assert!(!g.has_renown(g.board(&source).unwrap().1));
    g.players[0].society_zone.exposure=1;assert!(g.has_renown(g.board(&source).unwrap().1));g.board_mut(&source).unwrap().exhausted=true;assert!(g.has_renown(g.board(&own).unwrap().1));
    let other=field(&mut g,"WM041",2);assert!(!g.has_renown(g.board(&own).unwrap().1));assert!(!g.has_renown(g.board(&source).unwrap().1));g.board_mut(&other).unwrap().face_down=true;assert!(g.has_renown(g.board(&own).unwrap().1));
    g.board_mut(&source).unwrap().face_down=true;assert!(!g.has_renown(g.board(&own).unwrap().1));assert!(g.has_renown(g.board(&enemy).unwrap().1));assert!(g.has_renown(g.board(&temporary).unwrap().1));
}
#[test]fn batch3_wm041_real_public_play_forced_reward_goes_above_paid_card_and_resolves_after_source_leaves(){
    let mut g=game();fill(&mut g);let source=field(&mut g,"WM041",0);fund(&mut g,0,"JC056",3);let id=held(&mut g,"JC056",0);let hand=g.players[0].hand.len();
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});ready_reward(&mut g);assert!(g.pending.is_none());assert_eq!(g.stack.len(),2);
    g.board_mut(&source).unwrap().controller=2;g.remove_dead(&source,RemovalCause::Destroy);top(&mut g);assert_eq!(g.players[0].society_zone.exposure,1);assert_eq!(g.players[2].society_zone.exposure,0);assert_eq!(g.players[0].hand.len(),hand);assert_eq!(g.stack.len(),1);top(&mut g);assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC056"));restore(&mut g);
}
#[test]fn batch3_wm041_direct_entry_and_paid_reveal_do_not_trigger_public_play_reward(){
    let mut g=game();fill(&mut g);field(&mut g,"WM041",0);let c=g.make_card("JC056",0);let id=c.id.clone();g.players[0].graveyard.push(c);g.world_graveyard_entry(0,1,&id).unwrap();assert!(g.effects.iter().all(|e|!matches!(e,Effect::Declare{declaration} if declaration.ability.event==Some(Event::WM041PublicCharacterPlayed))));
    let hidden=field(&mut g,"JC056",0);g.board_mut(&hidden).unwrap().face_down=true;fund(&mut g,0,"JC056",2);act(&mut g,0,Action{card_id:Some(hidden),..Action::new("reveal")});assert!(g.stack.iter().all(|s|s.frame.as_ref().is_none_or(|f|f.ability_key!="batch3-wm041-public-play")));top(&mut g);assert_eq!(g.players[0].society_zone.exposure,0);restore(&mut g);
}
#[test]fn batch3_wm041_nonpublic_play_and_allied_public_play_do_not_create_personal_reward(){
    let mut g=game();fill(&mut g);field(&mut g,"WM041",0);fund(&mut g,0,"JC084",1);let id=held(&mut g,"JC084",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});assert_eq!(g.stack.len(),1);top(&mut g);
    fund(&mut g,1,"JC056",3);let id=held(&mut g,"JC056",1);let region=(0..g.regions.len()).find(|r|g.accessible(1,*r)).unwrap();act(&mut g,1,Action{card_id:Some(id),region:Some(region),..Action::new("deploy")});assert_eq!(g.stack.len(),1);top(&mut g);assert_eq!(g.players[0].society_zone.exposure,0);restore(&mut g);
}
#[test]fn batch3_wm041_personal_sun_public_play_snapshot_survives_loss_of_condition(){
    let mut g=game();fill(&mut g);field(&mut g,"WM041",0);g.players[0].society_zone.exposure=1;fund(&mut g,0,"JC056",4);let id=held(&mut g,"WM040",0);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});ready_reward(&mut g);g.players[0].society_zone.exposure=0;top(&mut g);assert_eq!(g.players[0].society_zone.exposure,1);restore(&mut g);
}
#[test]fn batch3_wm041_duplicate_or_forged_public_observation_and_optional_pending_are_rejected(){
    let mut g=game();fill(&mut g);field(&mut g,"WM041",0);let c=held(&mut g,"XQ46",0);act(&mut g,0,Action{card_id:Some(c),region:Some(0),..Action::new("deploy")});ready_reward(&mut g);
    let item=g.stack.last().unwrap().clone();let mut bad=g.clone();bad.stack.push(item.clone());assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let f=item.frame.unwrap();let mut bad=g.clone();if let Op::WM041PublicPlay{observation:Some(o)}=&mut bad.stack.last_mut().unwrap().frame.as_mut().unwrap().steps[0].op{o.played.card.definition="JC084".into();}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut d=crate::batch3::definition("WM041").unwrap().abilities.remove(0);d.ops=f.steps.iter().map(|s|s.op.clone()).collect();let d=Declaration{actor:f.actor,source:f.source.clone(),ability:d};assert!(g.choose_declaration(d,DeclareChoice::Accept,vec![]).is_err());restore(&mut g);
}
#[test]fn batch3_wm041_four_seat_renown_projection_tracks_personal_sun_and_enemy_sources(){
    let mut g=game();let source=field(&mut g,"WM041",0);g.players[1].society_zone.exposure=1;
    for active in [false,true,false]{g.players[0].society_zone.exposure=u32::from(active);for seat in 0..4{let v=g.view(seat);let c=v.regions[0].characters.iter().find(|c|c.instance_id==source).unwrap();assert_eq!(c.current_renown,active.then_some(true));}restore(&mut g);}
    let own=field(&mut g,"JC056",0);let enemy=field(&mut g,"WM041",2);for seat in 0..4{let v=g.view(seat);assert_eq!(v.regions[0].characters.iter().find(|c|c.instance_id==own).unwrap().current_renown,None);}g.hide_in_place_successor(&enemy);assert!(g.has_renown(g.board(&own).unwrap().1));restore(&mut g);
}
#[test]fn batch3_wm041_real_paid_six_asset_deploy_preserves_print_and_does_not_trigger_its_own_play(){
    let mut g=game();fill(&mut g);fund(&mut g,0,"JC056",6);let id=held(&mut g,"WM041",0);let hand=g.players[0].hand.len();act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});assert_eq!(g.stack.len(),1);top(&mut g);
    let c=g.regions[0].cards.iter().find(|c|c.definition=="WM041").unwrap();assert_eq!(g.current_icons(c,0),Icons{investigation:3,combat:1,influence:2});assert_eq!(g.defense(c,0),2);assert!(g.current_leader(c));assert!(!g.has_renown(c));assert_eq!(g.players[0].society_zone.exposure,0);assert_eq!(g.players[0].hand.len(),hand-1);assert!(g.players[0].assets.iter().all(|c|c.exhausted));restore(&mut g);
}
#[test]fn batch3_wm041_observer_source_cannot_be_a_private_copy_with_a_forged_board_region(){
    let mut g=game();fill(&mut g);field(&mut g,"WM041",0);fund(&mut g,0,"JC056",2);let id=held(&mut g,"JC056",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});ready_reward(&mut g);let c=g.make_card("WM041",0);g.players[0].hand.push(c.clone());g.stack.last_mut().unwrap().frame.as_mut().unwrap().source.card=c;assert!(Game::from_persisted(&serde_json::to_string(&g).unwrap()).is_err());
}
