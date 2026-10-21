use crate::{model::*,rules::*,unit_support::*};
fn pending(g:&mut Game){for _ in 0..48{if g.pending.is_some(){restore(g);return;}pass(g);}panic!("missing warrant pending");}
fn warrant(g:&mut Game,name:&str)->String{
    g.players[0].society_zone.card=None;fund(g,0,"JC056",2);let card=held(g,"XQ26",0);act(g,0,Action{card_id:Some(card),target_id:Some("society:p0".into()),..Action::new("play")});pending(g);
    assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key=="batch3-xq26-name"));choose(g,vec!["accept".into()]);pending(g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"batch3-xq26-name");choose(g,vec![name.into()]);g.attachments.iter().find(|a|a.card.definition=="XQ26").unwrap().card.id.clone()
}
fn wanted(g:&mut Game){for _ in 0..48{if g.pending.is_some(){assert!(!matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.event==Some(Event::XQ26NamedCharacterEntered)));choose(g,vec![]);continue;}if g.stack.last().and_then(|s|s.frame.as_ref()).is_some_and(|f|f.ability_key=="batch3-xq26-wanted"){restore(g);return;}if !g.effects.is_empty(){g.drive().unwrap();}else{pass(g);}}panic!("missing wanted response");}
fn enemy_play(g:&mut Game){g.begin_window(Window::Action(1));fund(g,2,"JC029",2);let id=held(g,"JC029",2);let region=(0..g.regions.len()).find(|r|g.accessible(2,*r)).unwrap();act(g,2,Action{card_id:Some(id),region:Some(region),..Action::new("deploy")});wanted(g);}
#[test]fn batch3_xq26_paid_society_attachment_without_society_card_names_publicly_and_rejects_polluted_confirmation(){
    let mut g=game();let source=warrant(&mut g,"新生血族");assert!(g.players[0].society_zone.card.is_none());assert_eq!(g.players[0].assets.iter().filter(|c|c.exhausted).count(),1);
    for seat in 0..4{let v=g.view(seat);let a=v.attachments.iter().find(|a|a.card.instance_id==source).unwrap();assert_eq!(a.card.named_character.as_deref(),Some("新生血族"));assert_eq!(a.host_id,"society:p0");}restore(&mut g);
    g.remove_dead(&source,RemovalCause::Destroy);assert!(g.players[0].graveyard.iter().filter(|c|c.definition=="XQ26").all(|c|c.named_character.is_none()));restore(&mut g);
}
#[test]fn batch3_xq26_actual_enemy_entry_has_no_targets_and_destroys_even_with_shield(){
    let mut g=game();let source=warrant(&mut g,"新生血族");enemy_play(&mut g);let f=g.stack.last().unwrap().frame.as_ref().unwrap();assert!(f.targets.is_empty());let id=if let Op::XQ26Wanted{observation:Some(o)}=&f.steps[0].op{o.entered.card.id.clone()}else{panic!()};g.board_mut(&id).unwrap().shield=1;top(&mut g);
    assert!(g.board(&id).is_none());assert!(!g.attachments.iter().any(|a|a.card.id==source));assert!(g.players[2].graveyard.iter().any(|c|c.definition=="JC029"));assert!(g.pending.is_none());restore(&mut g);
}
#[test]fn batch3_xq26_original_warrant_departure_does_not_cancel_frozen_kill_or_sacrifice_an_identical_replacement(){
    let mut g=game();let source=warrant(&mut g,"新生血族");enemy_play(&mut g);let id=g.regions.iter().flat_map(|r|&r.cards).find(|c|c.definition=="JC029").unwrap().id.clone();g.remove_dead(&source,RemovalCause::Destroy);
    let mut c=g.make_card("XQ26",0);c.named_character=Some("新生血族".into());let replacement=c.id.clone();g.attachments.push(Attachment{card:c,host_id:"society:p0".into()});top(&mut g);assert!(g.board(&id).is_none());assert!(g.attachments.iter().any(|a|a.card.id==replacement));restore(&mut g);
}
#[test]fn batch3_xq26_entry_instance_hidden_in_response_is_not_replaced_by_its_new_hidden_identity(){
    let mut g=game();let source=warrant(&mut g,"新生血族");enemy_play(&mut g);let id=g.regions.iter().flat_map(|r|&r.cards).find(|c|c.definition=="JC029").unwrap().id.clone();let hidden=g.hide_in_place_successor(&id).unwrap();top(&mut g);assert!(!g.attachments.iter().any(|a|a.card.id==source));assert!(g.board(&hidden.id).unwrap().1.face_down);restore(&mut g);
}
#[test]fn batch3_xq26_same_name_friendly_direct_entry_also_triggers(){
    let mut g=game();warrant(&mut g,"新生血族");let c=g.make_card("JC029",1);let id=c.id.clone();g.players[1].graveyard.push(c);g.world_graveyard_entry(1,0,&id).unwrap();wanted(&mut g);top(&mut g);assert!(g.players[1].graveyard.iter().any(|c|c.definition=="JC029"));restore(&mut g);
}
#[test]fn batch3_xq26_hidden_deploy_has_no_named_trigger_until_real_paid_reveal(){
    let mut g=game();warrant(&mut g,"新生血族");g.begin_window(Window::Action(1));fund(&mut g,2,"JC029",3);let id=held(&mut g,"JC029",2);let region=(0..g.regions.len()).find(|r|g.accessible(2,*r)).unwrap();act(&mut g,2,Action{card_id:Some(id),region:Some(region),..Action::new("conceal")});assert!(g.stack.is_empty());let hidden=g.regions[region].cards[0].id.clone();act(&mut g,2,Action{card_id:Some(hidden),..Action::new("reveal")});wanted(&mut g);top(&mut g);assert!(g.players[2].graveyard.iter().any(|c|c.definition=="JC029"));restore(&mut g);
}
#[test]fn batch3_xq26_named_instance_private_zone_and_duplicate_or_altered_observation_are_rejected(){
    let mut g=game();warrant(&mut g,"新生血族");enemy_play(&mut g);let mut bad=g.clone();bad.stack.push(bad.stack.last().unwrap().clone());assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut bad=g.clone();bad.stack.last_mut().unwrap().frame.as_mut().unwrap().source.card.named_character=Some("无知路人".into());assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut bad=g.clone();let mut c=bad.make_card("XQ26",0);c.named_character=Some("新生血族".into());bad.players[0].hand.push(c);assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());restore(&mut g);
}
#[test]fn batch3_xq26_naming_pending_is_exact_and_optional_entry_can_be_declined_without_a_named_effect(){
    let mut g=game();fund(&mut g,0,"JC056",2);let id=held(&mut g,"XQ26",0);act(&mut g,0,Action{card_id:Some(id),target_id:Some("society:p0".into()),..Action::new("play")});pending(&mut g);choose(&mut g,vec!["accept".into()]);pending(&mut g);
    let p=g.pending.clone().unwrap();reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec!["新生血族".into()]),target_id:Some("p2".into()),..Action::new("choose")});let mut bad=g.clone();bad.pending.as_mut().unwrap().choice.options.pop();assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());choose(&mut g,vec!["新生血族".into()]);
    let mut g=game();fund(&mut g,0,"JC056",2);let id=held(&mut g,"XQ26",0);act(&mut g,0,Action{card_id:Some(id),target_id:Some("society:p0".into()),..Action::new("play")});pending(&mut g);choose(&mut g,vec![]);assert!(g.attachments[0].card.named_character.is_none());enemy_play_without_wanted(&mut g);restore(&mut g);
}
fn enemy_play_without_wanted(g:&mut Game){g.begin_window(Window::Action(1));fund(g,2,"JC029",2);let id=held(g,"JC029",2);let region=(0..g.regions.len()).find(|r|g.accessible(2,*r)).unwrap();act(g,2,Action{card_id:Some(id),region:Some(region),..Action::new("deploy")});top(g);assert!(g.stack.is_empty());assert!(g.attachments.iter().any(|a|a.card.definition=="XQ26"));}
#[test]fn batch3_xq26_non_targeted_forced_destruction_ignores_barrier_and_cannot_be_declined(){
    let mut g=game();warrant(&mut g,"幸运新人");let c=g.make_card("JZ08",2);let id=c.id.clone();g.players[2].graveyard.push(c);g.world_graveyard_entry(2,1,&id).unwrap();wanted(&mut g);let id=g.regions[1].cards[0].id.clone();assert!(g.has_barrier(g.board(&id).unwrap().1));
    let f=g.stack.last().unwrap().frame.as_ref().unwrap();let mut a=crate::batch3::definition("XQ26").unwrap().abilities.remove(2);a.ops=f.steps.iter().map(|s|s.op.clone()).collect();let d=Declaration{actor:f.actor,source:f.source.clone(),ability:a};assert!(g.choose_declaration(d,DeclareChoice::Accept,vec![]).is_err());top(&mut g);assert!(g.board(&id).is_none());restore(&mut g);
}
