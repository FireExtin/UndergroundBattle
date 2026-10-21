use crate::{catalog, model::*, rules::*, unit_support::*};
fn hidden(g:&mut Game,seat:usize)->String {let old=field(g,"JZ26",seat);g.hide_in_place_successor(&old).unwrap().id}
fn pending(g:&mut Game){for _ in 0..64{if g.pending.is_some(){restore(g);return;}if !g.effects.is_empty(){g.drive().unwrap();}else if !g.stack.is_empty(){pass(g);}else{panic!("no trap pending");}}panic!("bounded trap");}
fn inspect(g:&mut Game,actor:usize,source:&str,id:&str){act(g,actor,Action{card_id:Some(source.into()),target_id:Some(id.into()),..Action::new("inspect-hidden")});}
fn accept(g:&mut Game){pending(g);assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration:d,..} if d.source.card.definition=="JZ26"));choose(g,vec!["accept".into()]);}
#[test]fn batch3_jz26_stealth_blocks_faceup_play_but_paid_conceal_and_paid_reveal_are_current_kernel_commands(){
    let mut g=game();fund(&mut g,0,"JC029",2);let id=held(&mut g,"JZ26",0);reject(&mut g,0,Action{card_id:Some(id.clone()),region:Some(0),..Action::new("deploy")});act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("conceal")});let id=g.regions[0].cards[0].id.clone();act(&mut g,0,Action{card_id:Some(id),..Action::new("reveal")});top(&mut g);let c=&g.regions[0].cards[0];assert!(!c.face_down);assert_eq!(g.current_icons(c,0).combat,1);assert!(definition("JZ26").traits.cannot_be_equipped);restore(&mut g);
}
#[test]fn batch3_jz26_monitor_actual_free_inspection_triggers_optional_private_source_without_changing_action_right(){
    let mut g=game();let trap=hidden(&mut g,0);fund(&mut g,0,"JC029",2);let monitor=field(&mut g,"JC057",2);let victim=field(&mut g,"JC125",2);g.board_mut(&victim).unwrap().shield=2;
    assert!(g.view(2).private_inspections.is_empty());let window=g.window.clone();let passed=g.passed.clone();assert!(g.legal_actions(2).iter().any(|a|a.action.kind=="inspect-hidden"));inspect(&mut g,2,&monitor,&trap);assert_eq!(g.window,window);assert_eq!(g.passed,passed);assert_eq!(g.view(2).private_inspections[0].cards[0].card_id.as_deref(),Some("JZ26"));
    for seat in [1,3]{let v=serde_json::to_string(&g.view(seat)).unwrap();assert!(!v.contains("JZ26"));assert!(!v.contains("袖中毒物"));}
    accept(&mut g);assert!(g.stack.last().unwrap().label.starts_with("暗藏者："));
    for seat in [1,3]{let v=g.view(seat);let text=serde_json::to_string(&v).unwrap().to_lowercase();assert!(!text.contains("jz26"));assert!(!text.contains("袖中毒物"));assert_eq!(v.stack.last().unwrap().card_id,None);assert_eq!(v.stack.last().unwrap().ability_id,None);}
    top(&mut g);assert!(g.board(&victim).is_none());assert!(g.board(&trap).is_some());restore(&mut g);
}
#[test]fn batch3_jz26_repeated_views_and_rejected_inspections_never_advance_rng_or_create_events(){
    let mut g=game();let trap=hidden(&mut g,0);fund(&mut g,0,"JC029",2);let monitor=field(&mut g,"JC057",2);let before=serde_json::to_string(&g).unwrap();for _ in 0..4{for seat in 0..4{g.view(seat);}}assert_eq!(serde_json::to_string(&g).unwrap(),before);
    reject(&mut g,3,Action{card_id:Some(monitor.clone()),target_id:Some(trap.clone()),..Action::new("inspect-hidden")});reject(&mut g,2,Action{card_id:Some(monitor.clone()),target_id:Some(trap.clone()),selected:Some(vec![]),..Action::new("inspect-hidden")});g.board_mut(&monitor).unwrap().exhausted=true;reject(&mut g,2,Action{card_id:Some(monitor),target_id:Some(trap),..Action::new("inspect-hidden")});
}
#[test]fn batch3_jz26_personal_blue_icons_count_exhausted_assets_and_never_allied_or_enemy_assets(){
    for mode in 0..4{let mut g=game();let trap=hidden(&mut g,0);let monitor=field(&mut g,"JC057",2);fund(&mut g,if mode==0{1}else{0},"JC029",if mode==1{1}else{2});if mode==2{for a in &mut g.players[0].assets{a.exhausted=true;}}if mode==3{g.board_mut(&trap).unwrap().controller=2;}
        inspect(&mut g,2,&monitor,&trap);assert_eq!(g.pending.is_some(),mode==2);restore(&mut g);}
}
#[test]fn batch3_jz26_actual_jc062_inspection_queues_once_before_private_confirmation_and_keeps_other_seats_private(){
    let mut g=game();let trap=hidden(&mut g,0);fund(&mut g,0,"JC029",2);fund(&mut g,2,"JC057",1);field(&mut g,"JC125",2);g.begin_window(Window::Action(1));let spell=held(&mut g,"JC062",2);
    act(&mut g,2,Action{card_id:Some(spell),ability_id:Some("batch3-jc062-investigate".into()),selected:Some(vec![trap]),..Action::new("play")});pending(&mut g);assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Frame{choice:FrameChoice::JC062Inspection{..},..}));assert_eq!(g.view(2).pending_choice.unwrap().preview_cards[0].card_id.as_deref(),Some("JZ26"));for seat in [1,3]{assert!(!serde_json::to_string(&g.view(seat)).unwrap().contains("袖中毒物"));}
    choose(&mut g,vec![]);accept(&mut g);top(&mut g);assert_eq!(g.players[2].graveyard.iter().filter(|c|c.definition=="JC125").count(),1);restore(&mut g);
}
#[test]fn batch3_jz26_enemy_destruction_captures_hidden_source_before_real_grave_entry_and_old_uid_does_not_return(){
    let mut g=game();let trap=hidden(&mut g,0);fund(&mut g,0,"JC029",2);let victim=field(&mut g,"JC125",2);g.destroy_as(2,&trap);assert!(g.board(&trap).is_none());assert!(g.players[0].graveyard.iter().any(|c|c.definition=="JZ26" && c.id!=trap));accept(&mut g);top(&mut g);assert!(g.board(&victim).is_none());restore(&mut g);
}
#[test]fn batch3_jz26_sacrifice_friendly_destruction_and_world_primitive_do_not_claim_an_enemy_player(){
    for mode in 0..3{let mut g=game();let trap=hidden(&mut g,0);fund(&mut g,0,"JC029",2);match mode{0=>g.remove_dead(&trap,RemovalCause::Sacrifice),1=>g.destroy_as(1,&trap),_=>g.remove_dead(&trap,RemovalCause::Destroy)}assert!(g.effects.iter().all(|e|!matches!(e,Effect::Declare{declaration:d} if d.source.card.definition=="JZ26")));restore(&mut g);}
}
#[test]fn batch3_jz26_no_enemy_or_replaced_original_region_uses_no_randomness_and_response_asset_loss_cancels(){
    for mode in 0..3{let mut g=game();let trap=hidden(&mut g,0);fund(&mut g,0,"JC029",2);if mode!=0{field(&mut g,"JC125",2);}g.observe_hidden_inspection(2,&trap);accept(&mut g);if mode==1{g.regions[0].card=g.make_card("DQJC115",0);}if mode==2{let a=g.players[0].assets.pop().unwrap();let a=g.reset_zone_card(a);g.players[0].graveyard.push(a);}let rng=g.random;top(&mut g);assert_eq!(g.random,rng);assert!(g.players[2].graveyard.is_empty());restore(&mut g);}
}
#[test]fn batch3_jz26_random_minimum_cost_is_untargeted_and_u64_seed_survives_each_response_restore(){
    let mut g=game();g.random=u64::MAX-132;let trap=hidden(&mut g,0);fund(&mut g,0,"JC029",2);let a=field(&mut g,"JC125",2);let b=field(&mut g,"JC125",3);let higher=field(&mut g,"JC029",2);g.observe_hidden_inspection(2,&trap);accept(&mut g);let mut same=g.clone();top(&mut g);top(&mut same);assert_eq!(serde_json::to_string(&g).unwrap(),serde_json::to_string(&same).unwrap());assert_ne!(g.board(&a).is_some(),g.board(&b).is_some());assert!(g.board(&higher).is_some());restore(&mut g);
}
#[test]fn batch3_jz26_duplicate_observation_forged_blue_receipt_and_optional_choice_payload_are_rejected(){
    let mut g=game();let trap=hidden(&mut g,0);fund(&mut g,0,"JC029",2);g.observe_hidden_inspection(2,&trap);let mut bad=g.clone();bad.effects.push_back(bad.effects.front().unwrap().clone());assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut bad=g.clone();if let Effect::Declare{declaration:d}=bad.effects.front_mut().unwrap(){if let Op::JZ26Trap{observation:Some(o)}=&mut d.ability.ops[0]{o.by=1;}}assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());g.drive().unwrap();let p=g.pending.clone().unwrap();reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec!["accept".into()]),target_id:Some("replacement".into()),..Action::new("choose")});assert!(!catalog::card("JZ26").supported);
}
#[test]fn batch3_jz26_actual_jz46_private_inspection_is_one_event_and_never_locks_a_snake_as_criminal(){
    let mut g=game();let trap=hidden(&mut g,0);fund(&mut g,0,"JC029",2);field(&mut g,"JC125",2);let c=g.make_card("JZ46",2);let source=c.id.clone();g.attachments.push(Attachment{card:c,host_id:g.regions[0].card.id.clone()});g.begin_window(Window::Action(1));
    act(&mut g,2,Action{card_id:Some(source.clone()),ability_id:Some("jz46-inspect-hidden".into()),target_id:Some(trap.clone()),..Action::new("activate")});pending(&mut g);assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Frame{choice:FrameChoice::JZ46InspectHidden{..},..}));assert_eq!(g.view(2).pending_choice.unwrap().preview_cards[0].card_id.as_deref(),Some("JZ26"));choose(&mut g,vec![]);accept(&mut g);top(&mut g);assert_eq!(g.board(&trap).unwrap().1.lock_markers,0);assert!(g.in_play(&source).unwrap().1.exhausted);restore(&mut g);
}
#[test]fn batch3_jz26_actual_xq27_paid_sweep_records_enemy_player_once_for_each_destroyed_hidden_trap(){
    let mut g=game();let trap=hidden(&mut g,0);fund(&mut g,0,"JC029",2);let victim=field(&mut g,"JC125",2);let own=hidden(&mut g,2);fund(&mut g,2,"XQ27",4);let spell=held(&mut g,"XQ27",2);g.begin_window(Window::Action(1));
    act(&mut g,2,Action{card_id:Some(spell),ability_id:Some("hidden-sweep".into()),..Action::new("play")});accept(&mut g);assert!(g.board(&trap).is_none());assert!(g.board(&own).is_none());top(&mut g);assert!(g.board(&victim).is_none());assert!(g.effects.iter().all(|e|!matches!(e,Effect::Declare{declaration:d} if d.source.card.definition=="JZ26")));restore(&mut g);
}
#[test]fn batch3_jz26_distinct_explicit_inspections_may_repeat_but_views_and_stale_revision_retries_cannot(){
    use crate::room::*;
    let mut g=game();let trap=hidden(&mut g,0);fund(&mut g,0,"JC029",2);let monitor=field(&mut g,"JC057",2);let room=crate::jc029_tests::envelope(&g);
    let cmd=RoomCommand{command_id:"trap-inspection-once".into(),expected_version:room.revision,action:SessionAction::Game{action:Action{card_id:Some(monitor.clone()),target_id:Some(trap.clone()),..Action::new("inspect-hidden")}}};
    let accepted=room.transition(2,Some(cmd.clone()),0).unwrap();assert_eq!(accepted.error_code,None);let mut next=RoomEnvelope::from_persisted(&accepted.state).unwrap();assert_eq!(serde_json::to_string(&room.replay_events(&accepted.journal).unwrap()).unwrap(),accepted.state);
    let retry=next.transition(2,Some(cmd),0).unwrap();assert_eq!(retry.error_code.as_deref(),Some("version_conflict"));assert!(!retry.changed);assert_eq!(retry.state,accepted.state);
    let before=serde_json::to_string(&next).unwrap();for seat in 0..4{next.view(seat,0);}assert_eq!(serde_json::to_string(&next).unwrap(),before);
    let again=RoomCommand{command_id:"trap-inspection-twice".into(),expected_version:next.revision,action:SessionAction::Game{action:Action{card_id:Some(monitor),target_id:Some(trap),..Action::new("inspect-hidden")}}};let accepted=next.transition(2,Some(again),0).unwrap();assert_eq!(accepted.error_code,None);next=RoomEnvelope::from_persisted(&accepted.state).unwrap();assert_eq!(next.game.effects.iter().filter(|e|matches!(e,Effect::Declare{declaration:d} if d.source.card.definition=="JZ26")).count(),1);assert!(matches!(&next.game.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration:d,..} if d.source.card.definition=="JZ26"));
}
#[test]fn batch3_jz26_monitor_read_during_response_preserves_original_clock_members_and_composing_intent(){
    use crate::room::*;
    let mut g=game();let target=field(&mut g,"JC125",0);let h=field(&mut g,"JC052",0);let hidden=g.hide_in_place_successor(&h).unwrap().id;let monitor=field(&mut g,"JC057",2);
    fund(&mut g,0,"XQ03",2);fund(&mut g,0,"JC057",1);fund(&mut g,2,"JC057",1);held(&mut g,"JC062",0);held(&mut g,"JC062",2);let spell=held(&mut g,"XQ03",0);
    act(&mut g,0,Action{card_id:Some(spell),target_id:Some(target),..Action::new("play")});let room=crate::jc029_tests::envelope(&g);let w=room.pacing.window.clone().unwrap();let composer=*w.members.keys().next().unwrap();
    let start=RoomCommand{command_id:"compose-before-inspect".into(),expected_version:room.revision,action:SessionAction::BeginResponse{window_id:w.id,intent_id:"retained-intent".into()}};
    let composing=room.transition(composer,Some(start),0).unwrap();assert_eq!(composing.error_code,None,"{:?}",composing.error_message);let room=RoomEnvelope::from_persisted(&composing.state).unwrap();assert!(matches!(room.pacing.window.as_ref().unwrap().members.get(&composer),Some(Decision::Composing{intent_id}) if intent_id=="retained-intent"));
    let before=serde_json::to_value(&room.pacing.window).unwrap();let cmd=RoomCommand{command_id:"free-inspect-response".into(),expected_version:room.revision,action:SessionAction::Game{action:Action{card_id:Some(monitor),target_id:Some(hidden),..Action::new("inspect-hidden")}}};assert!(room.view(2,0).game.legal_actions.iter().any(|a|a.action.kind=="inspect-hidden"));let accepted=room.transition(2,Some(cmd),0).unwrap();assert_eq!(accepted.error_code,None);let next=RoomEnvelope::from_persisted(&accepted.state).unwrap();assert_eq!(serde_json::to_value(&next.pacing.window).unwrap(),before);assert_eq!(next.game.stack.len(),1);
}
