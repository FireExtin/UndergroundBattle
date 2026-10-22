//! Marker/control/attachment layouts are explicit Native fixtures. Paid entry,
//! Prepare ordering/target selection, responses and restore use real APIs.
use crate::{catalog, model::*, rules::*, unit_support::*};
use crate::batch4_timekeeper::{ENTRY, PREPARE};
use crate::jc029_tests::{envelope, pass_top};

fn enter(g: &mut Game, accept: bool) -> String {
    fund(g, 0, "XG16", 6);
    let hand = held(g, "XG16", 0);
    act(g, 0, Action { card_id: Some(hand), region: Some(0), ..Action::new("deploy") });
    pass_top(g);
    let id = g.regions[0].cards.iter().find(|c| c.definition == "XG16").unwrap().id.clone();
    let ChoiceResolution::Declare { declaration: d, .. } = &g.pending.as_ref().unwrap().resolution else { panic!() };
    assert_eq!(d.ability.key, ENTRY);
    choose(g, if accept { vec!["accept".into()] } else { vec![] });
    if accept { pass_top(g); }
    id
}
fn begin(g: &mut Game) {
    g.turn += 1; g.begin_window(Window::Prepare); g.drive().unwrap(); restore(g);
}
fn prepare(g: &mut Game, source: &str, target: &str) {
    begin(g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "jc035-preparation-order");
    choose(g, vec![source.into()]);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "jc055-preparation-target");
    choose(g, vec![target.into()]);
}
fn markers(g: &Game, id: &str) -> u32 { g.in_play(id).unwrap().1.time_markers }
fn rescue(g: &mut Game, target: &str) {
    let actor = g.in_play(target).unwrap().1.controller;
    // The rescuer's placement is a disclosed fixture; the paid/exhausting Fast
    // ability and its ownership check are actual commands, never a spell alias.
    let rescuer = field(g, "JC075", actor);
    for _ in 0..4 { if g.priority_team == g.team(actor) { break; } pass(g); }
    fund(g, actor, "JC075", 2);
    let a = g.legal_actions(actor).into_iter().find(|a| a.action.kind == "activate"
        && a.action.card_id.as_deref() == Some(&rescuer) && a.action.target_id.as_deref() == Some(target)).unwrap().action;
    act(g, actor, a); pass_top(g);
}
fn invalid(g: &Game, value: serde_json::Value) {
    let text = serde_json::to_string(&value).unwrap();
    let result = std::panic::catch_unwind(|| Game::from_persisted(&text));
    assert!(result.is_ok() && result.unwrap().is_err());
    let mut room = serde_json::to_value(envelope(g)).unwrap(); room["game"] = value;
    assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());
}

#[test]
fn batch4_timekeeper_exact_source_and_closed_definition() {
    let c = catalog::card("XG16");
    assert_eq!((c.name.as_str(),c.cost,c.defense,c.unique,c.supported), ("计时人戴森·帕佐",6,Some(2),true,false));
    assert_eq!(c.subtitle.as_deref(),Some("策划阴谋")); assert_eq!(c.loyalty,["红色","红色","红色"]);
    assert_eq!(c.subtypes,["人类","超能力者","邪教徒"]); assert_eq!(c.magic,"心灵");
    assert_eq!(c.permanent_icons,Icons { investigation:2,combat:1,influence:1 });
    let d = definition("XG16"); assert!(d.traits.public); assert_eq!(d.abilities.len(),2);
    assert_eq!(d.abilities[0].event,Some(Event::Enter)); assert_eq!(d.abilities[1].event,Some(Event::Prepare));
    assert!(!crate::jc035::preparation_forced("XG16"));
    assert!(matches!(d.modifiers.as_slice(),[StaticModifier::XG16TimeDefense]));
    for change in 0..4 {
        let mut defs=crate::rules::definitions().clone(); let d=defs.get_mut("XG16").unwrap();
        match change { 0=>d.traits.public=false,1=>d.abilities[1].event=Some(Event::Death),2=>d.abilities[1].targets[0].relation=Relation::FriendlyTeam,_=>d.modifiers.clear() }
        assert!(crate::rules::validate_definitions(&defs).is_err());
    }
}
#[test]
fn batch4_timekeeper_real_paid_entry_optional_time_and_four_seat_projection() {
    let mut g=game(); let id=enter(&mut g,true);
    assert_eq!(g.resources(0),0); assert_eq!(markers(&g,&id),1); assert_eq!(g.defense(g.board(&id).unwrap().1,0),3);
    for seat in 0..4 { let v=g.view(seat); let c=v.regions[0].characters.iter().find(|c|c.instance_id==id).unwrap();assert_eq!(c.time_markers,Some(1));assert_eq!(c.defense,Some(3)); }
    restore(&mut g);
}
#[test]
fn batch4_timekeeper_entry_decline_and_public_conceal_and_payload_rejection() {
    let mut g=game(); let id=enter(&mut g,false); assert_eq!(markers(&g,&id),0);
    g.begin_window(Window::Action(0));let h=held(&mut g,"XG16",0);fund(&mut g,0,"XG16",6);
    reject(&mut g,0,Action{card_id:Some(h.clone()),region:Some(1),..Action::new("conceal")});
    reject(&mut g,0,Action{card_id:Some(h),region:Some(1),selected:Some(vec!["forged".into()]),..Action::new("deploy")});
    assert_eq!(g.resources(0),6);
}
#[test]
fn batch4_timekeeper_real_entry_source_rescued_before_time_does_not_mark_replacement() {
    let mut g=game(); fund(&mut g,0,"XG16",6);let h=held(&mut g,"XG16",0);
    act(&mut g,0,Action{card_id:Some(h),region:Some(0),..Action::new("deploy")});pass_top(&mut g);
    let source=g.regions[0].cards[0].id.clone();choose(&mut g,vec!["accept".into()]);rescue(&mut g,&source);pass_top(&mut g);
    assert!(g.in_play(&source).is_none());let c=g.players[0].hand.iter().find(|c|c.definition=="XG16").unwrap();assert_ne!(c.id,source);assert_eq!(c.time_markers,0);restore(&mut g);
}
#[test]
fn batch4_timekeeper_leader_uses_personal_sun_not_ally_sun_and_follows_control() {
    let mut g=game();let id=field(&mut g,"XG16",0);assert!(!g.current_leader(g.board(&id).unwrap().1));
    g.players[1].society_zone.exposure=1;assert!(!g.current_leader(g.board(&id).unwrap().1));
    g.players[0].society_zone.exposure=1;assert!(g.current_leader(g.board(&id).unwrap().1));
    g.board_mut(&id).unwrap().controller=2;assert!(!g.current_leader(g.board(&id).unwrap().1));
    g.players[2].society_zone.exposure=1;assert!(g.current_leader(g.board(&id).unwrap().1));restore(&mut g);
}
#[test]
fn batch4_timekeeper_defense_personal_ally_and_both_enemy_seats_across_regions() {
    let mut g=game();let own=field(&mut g,"JC125",0);let ally=field(&mut g,"XG15",1);let a=field(&mut g,"XG15",2);let b=field(&mut g,"XG15",3);
    let base=g.defense(g.board(&own).unwrap().1,0);g.board_mut(&own).unwrap().time_markers=2;
    for id in [&ally,&a,&b]{g.board_mut(id).unwrap().time_markers=1;}let source=g.make_card("XG16",0);g.regions[4].cards.push(source);
    assert_eq!(g.defense(g.board(&own).unwrap().1,0),base+2);assert_eq!(g.defense(g.board(&ally).unwrap().1,0),4);
    assert_eq!(g.defense(g.board(&a).unwrap().1,0),3);assert_eq!(g.defense(g.board(&b).unwrap().1,0),3);restore(&mut g);
}
#[test]
fn batch4_timekeeper_two_opposing_sources_combine_positive_and_negative_before_wounds() {
    let mut g=game();field(&mut g,"XG16",0);field(&mut g,"XG16",2);
    let a=field(&mut g,"XG15",0);let ally=field(&mut g,"XG15",1);for id in [&a,&ally]{g.board_mut(id).unwrap().time_markers=2;}
    assert_eq!(g.defense(g.board(&a).unwrap().1,0),4);assert_eq!(g.defense(g.board(&ally).unwrap().1,0),2);restore(&mut g);
}
#[test]
fn batch4_timekeeper_inherited_attachment_time_follows_its_actual_host() {
    let mut g=game();field(&mut g,"XG16",0);let own=field(&mut g,"JC125",0);let ally=field(&mut g,"JC125",1);
    let base=catalog::card("JC125").defense.unwrap();let mut card=g.make_card("XQ47",0);card.time_markers=2;
    g.attachments.push(Attachment{card,host_id:own.clone()});assert_eq!(g.defense(g.board(&own).unwrap().1,0),base+3);
    g.attachments[0].host_id=ally;assert_eq!(g.defense(g.board(&own).unwrap().1,0),base);restore(&mut g);
}
#[test]
fn batch4_timekeeper_attached_bat_is_a_character_and_host_inherits_time_once() {
    let mut g=game();field(&mut g,"XG16",0);let host=field(&mut g,"XQ12",0);let mut card=g.make_card("BQ026",0);let id=card.id.clone();card.time_markers=1;
    g.attachments.push(Attachment{card,host_id:host.clone()});assert_eq!(g.defense(g.in_play(&id).unwrap().1,0),2);
    assert_eq!(g.defense(g.board(&host).unwrap().1,0),catalog::card("XQ12").defense.unwrap()+1);restore(&mut g);
}
#[test]
fn batch4_timekeeper_time_bonus_precedes_wound_clamp() {
    let mut g=game();field(&mut g,"XG16",0);let id=field(&mut g,"JC125",0);let base=catalog::card("JC125").defense.unwrap();
    let c=g.board_mut(&id).unwrap();c.time_markers=2;c.wounds=base+1;
    assert_eq!(g.defense(g.board(&id).unwrap().1,0),1);restore(&mut g);
}
#[test]
fn batch4_timekeeper_all_positive_defense_precedes_time_penalty() {
    let mut g=game();field(&mut g,"XG16",0);let id=field(&mut g,"BQ079",2);
    // Permanent counters are an accumulated Ghoul fixture, not a new TK001 producer.
    let c=g.board_mut(&id).unwrap();c.time_markers=3;c.permanent_defense_markers=3;c.permanent_combat_markers=3;
    assert_eq!(g.defense(g.board(&id).unwrap().1,0),1);restore(&mut g);
}
#[test]
fn batch4_timekeeper_prepare_target_current_destroyed_count_and_receipt_once() {
    let mut g=game();let source=field(&mut g,"XG16",0);let id=field(&mut g,"JC125",0);g.regions[3].destroyed=true;
    prepare(&mut g,&source,&id);let f=g.stack.last().unwrap().frame.clone().unwrap();assert_eq!(f.ability_key,PREPARE);
    // Explicit response-layout fixture; no unimplemented destruction action claimed.
    g.regions[4].destroyed=true;restore(&mut g);pass_top(&mut g);assert_eq!(markers(&g,&id),2);
    assert!(matches!(g.jc035_preparation.as_ref().unwrap().entries[0].decision,Some(crate::jc035::PreparationDecision::Resolved{..})));
    let mut repeated=f;repeated.cursor=1;repeated.guard=GuardState::Accepted;let before=serde_json::to_string(&g).unwrap();assert!(g.jc035_resolve(&repeated).is_err());assert_eq!(serde_json::to_string(&g).unwrap(),before);
}
#[test]
fn batch4_timekeeper_prepare_can_be_declined_without_markers_or_rng_consumption() {
    let mut g=game();field(&mut g,"XG16",0);let random=g.random;begin(&mut g);choose(&mut g,vec![]);
    assert!(g.stack.is_empty());assert_eq!(g.random,random);assert!(matches!(g.jc035_preparation.as_ref().unwrap().entries[0].decision,Some(crate::jc035::PreparationDecision::Declined)));restore(&mut g);
}
#[test]
fn batch4_timekeeper_secret_plan_target_keeps_time_and_advance_and_identity_separate() {
    for destroyed in [0,2] {let mut g=game();let source=field(&mut g,"XG16",0);let hidden=field(&mut g,"JC125",2);g.board_mut(&hidden).unwrap().face_down=true;
        let mut card=g.make_card("WM045",2);card.face_down=true;let id=card.id.clone();g.attachments.push(Attachment{card,host_id:"society:p2".into()});
        for r in 0..destroyed{g.regions[r].destroyed=true;}begin(&mut g);choose(&mut g,vec![source]);
        let options=&g.pending.as_ref().unwrap().choice.options;assert!(options.iter().all(|o|o.id!=hidden));let target=options.iter().find(|o|o.id==id).unwrap();assert!(target.label.contains("秘密计划"));
        choose(&mut g,vec![id.clone()]);pass_top(&mut g);assert_eq!(markers(&g,&id),destroyed as u32);
        for seat in 0..4 {let v=g.view(seat);let c=&v.attachments.iter().find(|a|a.card.instance_id==id).unwrap().card;assert_eq!(c.name,"秘密计划");assert_eq!(c.advancement_markers,Some(0));if seat!=2{assert!(c.card_id.is_none()&&c.text.is_none());}}restore(&mut g);
    }
}
#[test]
fn batch4_timekeeper_real_prepare_source_rescue_keeps_declared_effect() {
    let mut g=game();let source=field(&mut g,"XG16",0);let target=field(&mut g,"JC125",0);g.regions[4].destroyed=true;
    prepare(&mut g,&source,&target);rescue(&mut g,&source);pass_top(&mut g);
    assert!(g.in_play(&source).is_none());assert_eq!(markers(&g,&target),1);restore(&mut g);
}
#[test]
fn batch4_timekeeper_real_prepare_target_rescue_cancels_and_does_not_mark_new_uid() {
    let mut g=game();let source=field(&mut g,"XG16",0);let target=field(&mut g,"XG15",2);g.regions[4].destroyed=true;
    prepare(&mut g,&source,&target);rescue(&mut g,&target);pass_top(&mut g);
    assert!(g.in_play(&target).is_none());assert!(g.players[2].hand.iter().filter(|c|c.definition=="XG15").all(|c|c.id!=target&&c.time_markers==0));
    assert!(matches!(g.jc035_preparation.as_ref().unwrap().entries[0].decision,Some(crate::jc035::PreparationDecision::Cancelled{..})));restore(&mut g);
}
#[test]
fn batch4_timekeeper_actual_time_increase_settles_enemy_zero_defense_death() {
    let mut g=game();let source=field(&mut g,"XG16",0);let target=field(&mut g,"XG15",2);g.board_mut(&target).unwrap().time_markers=3;g.regions[4].destroyed=true;
    prepare(&mut g,&source,&target);pass_top(&mut g);assert!(g.in_play(&target).is_none());assert!(g.players[2].graveyard.iter().any(|c|c.definition=="XG15"&&c.time_markers==0));restore(&mut g);
}
#[test]
fn batch4_timekeeper_prepare_frame_and_observation_mutations_reject_restore() {
    let mut g=game();let source=field(&mut g,"XG16",0);let target=field(&mut g,"JC125",0);prepare(&mut g,&source,&target);
    let base=serde_json::to_value(&g).unwrap();
    for change in 0..6 {let mut v=base.clone();match change {
        0=>v["stack"][0]["frame"]["steps"][0]["op"]=serde_json::json!(Op::XG16EntryTime),
        1=>v["stack"][0]["frame"]["actor"]=serde_json::json!(2),
        2=>v["stack"][0]["frame"]["targets"][0]["spec"]["relation"]=serde_json::json!(Relation::FriendlyTeam),
        3=>v["jc035_preparation"]["event_id"]=serde_json::json!("other-event"),
        4=>{let i=v["stack"][0].clone();v["stack"].as_array_mut().unwrap().push(i);},
        _=>v["stack"][0]["frame"]["already_paid"]=serde_json::json!([PaidCost::Assets(vec!["forged".into()])]),
    }invalid(&g,v);}
}
#[test]
fn batch4_timekeeper_prepare_order_with_jc055_resolves_each_receipt_once() {
    let mut g=game();let source=field(&mut g,"XG16",0);let old=field(&mut g,"JC055",0);let target=field(&mut g,"JC125",0);g.regions[4].destroyed=true;
    begin(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.options.len(),2);
    choose(&mut g,vec![source]);choose(&mut g,vec![target.clone()]);choose(&mut g,vec![old]);choose(&mut g,vec![target.clone()]);
    pass_top(&mut g);pass_top(&mut g);assert_eq!(markers(&g,&target),2);
    assert!(g.jc035_preparation.as_ref().unwrap().entries.iter().all(|e|matches!(e.decision,Some(crate::jc035::PreparationDecision::Resolved{..}))));restore(&mut g);
}

#[test]
fn batch4_timekeeper_real_aura_entry_settles_preexisting_enemy_time_death() {
    let mut g=game();let target=field(&mut g,"XG15",2);g.board_mut(&target).unwrap().time_markers=4;
    assert_eq!(g.defense(g.board(&target).unwrap().1,0),4);enter(&mut g,false);
    assert!(g.in_play(&target).is_none());assert!(g.players[2].graveyard.iter().any(|c|c.definition=="XG15"&&c.time_markers==0));restore(&mut g);
}
#[test]
fn batch4_timekeeper_real_source_rescue_settles_recipient_losing_defense() {
    let mut g=game();let source=field(&mut g,"XG16",0);let target=field(&mut g,"JC125",0);
    let c=g.board_mut(&target).unwrap();c.time_markers=1;c.wounds=1;g.regions[4].destroyed=true;
    assert_eq!(g.defense(g.board(&target).unwrap().1,0),1);prepare(&mut g,&source,&target);rescue(&mut g,&source);
    assert!(g.in_play(&target).is_none());pass_top(&mut g);
    assert!(matches!(g.jc035_preparation.as_ref().unwrap().entries[0].decision,Some(crate::jc035::PreparationDecision::Cancelled{..})));restore(&mut g);
}
fn room_step(r: &mut crate::room::RoomEnvelope, seat: usize, action: crate::room::SessionAction, now: u64) -> crate::room::RoomCommand {
    let command=crate::room::RoomCommand {command_id:format!("timekeeper-room-{}",r.revision),expected_version:r.revision,action};
    let result=r.transition(seat,Some(command.clone()),now).unwrap();assert!(result.error_code.is_none(),"{:?}",result.error_message);
    *r=crate::room::RoomEnvelope::from_persisted(&result.state).unwrap();command
}
#[test]
fn batch4_timekeeper_room_pause_target_resume_replay_and_response_receipt_large_u64() {
    use crate::room::{RoomEnvelope,SessionAction};
    let mut g=game();g.seed=u64::MAX-53;g.random=g.seed;let source=field(&mut g,"XG16",0);let target=field(&mut g,"JC125",0);g.regions[4].destroyed=true;
    begin(&mut g);choose(&mut g,vec![source]);let pending=g.pending.as_ref().unwrap().clone();let mut r=envelope(&g);
    room_step(&mut r,2,SessionAction::PauseRoom,1000);r=RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();
    assert_eq!(r.game.pending.as_ref().unwrap().choice.id,pending.choice.id);
    let a=Action{choice_id:Some(pending.choice.id),selected:Some(vec![target.clone()]),..Action::new("choose")};
    let blocked=r.transition(0,Some(crate::room::RoomCommand{command_id:"blocked-target".into(),expected_version:r.revision,action:SessionAction::Game{action:a.clone()}}),2000).unwrap();
    assert_eq!(blocked.error_code.as_deref(),Some("room_paused"));assert_eq!(markers(&r.game,&target),0);
    room_step(&mut r,1,SessionAction::ResumeRoom,3000);let before=serde_json::to_string(&r).unwrap();
    let command=room_step(&mut r,0,SessionAction::Game{action:a},4000);
    let original=RoomEnvelope::from_persisted(&before).unwrap();let replay=original.transition(0,Some(command.clone()),4000).unwrap();
    assert_eq!(serde_json::to_string(&r).unwrap(),replay.state);let duplicate=r.transition(0,Some(command),4001).unwrap();assert_eq!(duplicate.error_code.as_deref(),Some("version_conflict"));assert_eq!(duplicate.state,serde_json::to_string(&r).unwrap());
    for n in 0..16 {if r.game.stack.is_empty(){break;}let id=r.pacing.window.as_ref().unwrap().id.clone();let seat=(0..4).find(|s|r.game.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();room_step(&mut r,seat,SessionAction::PassResponse{window_id:id},5000+n);}
    assert!(r.game.stack.is_empty());assert_eq!(markers(&r.game,&target),1);assert_eq!(r.game.seed,u64::MAX-53);
    let restored=RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();
    for seat in 0..4{assert_eq!(serde_json::to_value(r.view(seat,6000)).unwrap(),serde_json::to_value(restored.view(seat,6000)).unwrap());}
}
