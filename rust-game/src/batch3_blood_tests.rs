use crate::{model::*,unit_support::*};
fn grave(g:&mut Game,print:&str,owner:usize)->String{let c=g.make_card(print,owner);let id=c.id.clone();g.players[owner].graveyard.push(c);id}
fn pending(g:&mut Game){for _ in 0..48{if g.pending.is_some(){restore(g);return;}if !g.effects.is_empty(){g.drive().unwrap();}else{pass(g);}}panic!("missing blood pending");}
fn enter(g:&mut Game,seat:usize)->String{let id=field(g,"JC029",seat);g.enter_triggers(seat,"JC029",&id,false);id}
fn accept_pair(g:&mut Game){pending(g);assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key=="batch3-bq037-feed"));choose(g,vec!["accept".into()]);}
fn clear_optional(g:&mut Game){for _ in 0..24{if let Some(p)=&g.pending{let ids=if matches!(&p.resolution,ChoiceResolution::Batch3EntryOrder{..}){vec![p.choice.options[0].id.clone()]}else{vec![]};choose(g,ids);}else if !g.stack.is_empty(){pass(g);}else if !g.effects.is_empty(){g.drive().unwrap();}else{return;}}panic!("optional events must finish");}
fn reveal_action(source:&str,target:&str)->Action{Action{card_id:Some(source.into()),ability_id:Some("batch3-bq037-reveal".into()),target_id:Some(target.into()),..Action::new("activate")}}
#[test]fn batch3_bq037_real_paid_deploy_is_not_its_own_vampire_entry_trigger(){
    let mut g=game();fund(&mut g,0,"JC029",7);let id=held(&mut g,"BQ037",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});top(&mut g);assert!(g.pending.is_none());let c=&g.regions[0].cards[0];assert_eq!(g.current_icons(c,0),Icons{investigation:1,combat:3,influence:3});assert_eq!(g.defense(c,0),3);assert!(g.players[0].assets.iter().all(|c|c.exhausted));restore(&mut g);
}
#[test]fn batch3_bq037_actual_paid_second_vampire_uses_two_distinct_blood_cards_and_real_attachment_bonuses(){
    let mut g=game();let source=field(&mut g,"BQ037",0);let a=grave(&mut g,"XQ14",0);let b=grave(&mut g,"BQ040",0);fund(&mut g,0,"JC029",2);let id=held(&mut g,"JC029",0);act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});accept_pair(&mut g);pending(&mut g);let incoming=g.regions[0].cards.iter().find(|c|c.definition=="JC029").unwrap().id.clone();choose(&mut g,vec![a.clone()]);restore(&mut g);choose(&mut g,vec![b.clone()]);assert_eq!(g.attachments.len(),2);assert!(g.attachments.iter().all(|a|a.card.id!=b && a.card.id!=a.host_id));assert_eq!(g.defense(g.board(&incoming).unwrap().1,0),2);assert_eq!(g.defense(g.board(&source).unwrap().1,0),4);assert_eq!(g.current_icons(g.board(&source).unwrap().1,0).combat,4);restore(&mut g);
}
#[test]fn batch3_bq037_each_drinker_current_controller_selects_own_blood_after_response_control_change(){
    let mut g=game();let source=field(&mut g,"BQ037",0);let own=grave(&mut g,"XQ14",0);let enemy=grave(&mut g,"BQ040",2);let incoming=enter(&mut g,0);accept_pair(&mut g);g.board_mut(&incoming).unwrap().controller=2;pending(&mut g);assert_eq!(g.pending.as_ref().unwrap().seat,2);assert!(g.pending.as_ref().unwrap().choice.options.iter().all(|o|o.id!=own));
    let p=g.pending.clone().unwrap();reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec![enemy.clone()]),..Action::new("choose")});choose(&mut g,vec![enemy]);assert_eq!(g.pending.as_ref().unwrap().seat,0);choose(&mut g,vec![own]);assert!(g.attachments.iter().any(|a|a.host_id==incoming && a.card.controller==2));assert!(g.attachments.iter().any(|a|a.host_id==source && a.card.controller==0));restore(&mut g);
}
#[test]fn batch3_bq037_society_sealed_blood_is_blank_public_and_preserves_foreign_owner_on_entry(){
    let mut g=game();field(&mut g,"BQ037",0);let c=g.make_card("XQ14",2);g.seal_card_on_valid_host(c,0,"society:p0");let old=g.sealed_cards[0].card.id.clone();for seat in 0..4{let v=g.view(seat);assert_eq!(v.sealed_cards[0].card.cost,None);assert_eq!(v.sealed_cards[0].card.icons,None);}let incoming=enter(&mut g,0);accept_pair(&mut g);pending(&mut g);assert_eq!(g.pending.as_ref().unwrap().seat,0);choose(&mut g,vec![old.clone()]);assert!(g.pending.is_none());let a=&g.attachments[0];assert_eq!((a.card.owner,a.card.controller),(2,0));assert_ne!(a.card.id,old);assert_eq!(a.host_id,incoming);g.remove_dead(&incoming,RemovalCause::Destroy);assert!(g.players[2].graveyard.iter().any(|c|c.definition=="XQ14"));restore(&mut g);
}
#[test]fn batch3_bq037_blood_on_another_character_or_teammate_society_is_not_an_eligible_source(){
    let mut g=game();field(&mut g,"BQ037",0);let host=field(&mut g,"JC029",0);for place in [host.as_str(),"society:p1"]{let c=g.make_card("XQ14",0);g.seal_card_on_valid_host(c,0,place);}enter(&mut g,0);accept_pair(&mut g);top(&mut g);assert!(g.pending.is_none());assert!(g.attachments.is_empty());assert_eq!(g.sealed_cards.len(),2);restore(&mut g);
}
#[test]fn batch3_bq037_departed_source_and_hidden_incoming_never_redirect_blood_to_replacements(){
    let mut g=game();let source=field(&mut g,"BQ037",0);let a=grave(&mut g,"XQ14",0);let incoming=enter(&mut g,0);accept_pair(&mut g);g.remove_dead(&source,RemovalCause::Destroy);let replacement=field(&mut g,"BQ037",0);pending(&mut g);choose(&mut g,vec![a]);assert!(g.attachments.iter().all(|a|a.host_id==incoming && a.host_id!=replacement));restore(&mut g);
    let mut g=game();let source=field(&mut g,"BQ037",0);let a=grave(&mut g,"XQ14",0);let incoming=enter(&mut g,0);accept_pair(&mut g);let hidden=g.hide_in_place_successor(&incoming).unwrap();pending(&mut g);assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Frame{choice:FrameChoice::BloodFeed{state},..} if state.cursor==1));choose(&mut g,vec![a]);assert_eq!(g.attachments[0].host_id,source);assert!(g.board(&hidden.id).unwrap().1.face_down);restore(&mut g);
}
#[test]fn batch3_bq037_does_not_observe_allied_enemy_or_movement_as_another_personal_vampire_entry(){
    let mut g=game();field(&mut g,"BQ037",0);for seat in [1,2]{enter(&mut g,seat);}let id=field(&mut g,"JC029",0);g.move_on_board_instance(&id,1);assert!(g.effects.iter().all(|e|!matches!(e,Effect::Declare{declaration} if declaration.ability.key=="batch3-bq037-feed")));restore(&mut g);
}
#[test]fn batch3_bq037_free_reveal_costs_one_ignores_loyalty_preserves_optional_decision_and_consumes_once(){
    let mut g=game();let source=field(&mut g,"BQ037",0);let target=field(&mut g,"JC029",0);let hidden=g.hide_in_place_successor(&target).unwrap();fund(&mut g,0,"JC125",2);assert!(!g.loyalty(0,"JC029"));act(&mut g,0,reveal_action(&source,&hidden.id));pending(&mut g);assert_eq!(g.resources(0),1);choose(&mut g,vec![hidden.id.clone()]);clear_optional(&mut g);assert!(g.board(&hidden.id).is_none());assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC029" && !c.face_down));
    let second=field(&mut g,"JC029",0);let hidden2=g.hide_in_place_successor(&second).unwrap();reject(&mut g,0,reveal_action(&source,&hidden2.id));restore(&mut g);
}
#[test]fn batch3_bq037_nonvampire_hidden_target_or_declined_resolution_keeps_payment_and_limit(){
    for vampire in [false,true]{let mut g=game();let source=field(&mut g,"BQ037",0);let target=field(&mut g,if vampire{"JC029"}else{"JC125"},0);let hidden=g.hide_in_place_successor(&target).unwrap();fund(&mut g,0,"JC125",2);act(&mut g,0,reveal_action(&source,&hidden.id));if vampire{pending(&mut g);choose(&mut g,vec![]);}else{top(&mut g);}assert_eq!(g.resources(0),1);assert!(g.board(&hidden.id).unwrap().1.face_down);reject(&mut g,0,reveal_action(&source,&hidden.id));}
}
#[test]fn batch3_bq037_blood_cursor_private_zone_and_duplicate_observation_tampering_are_rejected(){
    let mut g=game();field(&mut g,"BQ037",0);let blood=grave(&mut g,"XQ14",0);enter(&mut g,0);let mut bad=g.clone();let d=bad.effects.iter().find_map(|e|if let Effect::Declare{declaration}=e{Some(declaration.clone())}else{None}).unwrap();bad.effects.push_back(Effect::Declare{declaration:d});assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());accept_pair(&mut g);pending(&mut g);
    let mut bad=g.clone();if let ChoiceResolution::Frame{choice:FrameChoice::BloodFeed{state},..}=&mut bad.pending.as_mut().unwrap().resolution{state.cursor=1;state.recipients[1]="replaced".into();}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());let p=g.pending.clone().unwrap();reject(&mut g,p.seat,Action{choice_id:Some(p.choice.id),selected:Some(vec![blood.clone()]),target_id:Some("other".into()),..Action::new("choose")});choose(&mut g,vec![blood]);restore(&mut g);
}
#[test]fn batch3_society_seal_foreign_blood_returns_to_owner_when_society_player_is_eliminated(){
    let mut g=game();let c=g.make_card("XQ14",2);g.seal_card_on_valid_host(c,0,"society:p0");let id=g.sealed_cards[0].card.id.clone();g.eliminate(0);assert!(g.sealed_cards.is_empty());assert!(g.players[2].hand.iter().any(|c|c.definition=="XQ14" && c.id!=id));restore(&mut g);
}
#[test]fn batch3_bq037_continuation_cannot_skip_a_valid_first_recipient_or_reuse_an_old_blood_attachment(){
    let mut g=game();let source=field(&mut g,"BQ037",0);let first=grave(&mut g,"XQ14",0);grave(&mut g,"BQ040",0);enter(&mut g,0);accept_pair(&mut g);pending(&mut g);
    let mut bad=g.clone();if let ChoiceResolution::Frame{choice:FrameChoice::BloodFeed{state},..}=&mut bad.pending.as_mut().unwrap().resolution{state.cursor=1;}bad.pending.as_mut().unwrap().choice.title="穿刺公弗拉德三世：饮血1".into();assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    choose(&mut g,vec![first]);restore(&mut g);let mut bad=g.clone();if let ChoiceResolution::Frame{choice:FrameChoice::BloodFeed{state},..}=&mut bad.pending.as_mut().unwrap().resolution{state.completed[0].attached=Some(source);}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
}
#[test]fn batch3_bq037_observer_cannot_originate_from_a_private_zone_with_a_forged_region(){
    let mut g=game();field(&mut g,"BQ037",0);enter(&mut g,0);let c=g.make_card("BQ037",0);g.players[0].hand.push(c.clone());if let Effect::Declare{declaration}=g.effects.front_mut().unwrap(){declaration.source.card=c;}assert!(Game::from_persisted(&serde_json::to_string(&g).unwrap()).is_err());
}
