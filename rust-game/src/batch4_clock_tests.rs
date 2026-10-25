//! Disclosed Native layouts, followed by normal paid commands and checkpoints.
use crate::{catalog, deck, model::*, rules::*, unit_support::*, batch4_clock as c};
use crate::jc029_tests::{pass_top, envelope};

fn layout() -> (Game,String) {
    let mut g=game();
    let s=g.make_card("MSJC13",0); let id=s.id.clone();g.players[0].society_zone.card=Some(s);
    for seat in 0..4 { g.players[seat].deck.clear();for _ in 0..12 {let c=g.make_card("JC125",seat);g.players[seat].deck.push(c);} }
    (g,id)
}
fn activate(s:&str,key:&str,t:Option<String>)->Action {
    Action{card_id:Some(s.into()),ability_id:Some(key.into()),target_id:t,..Action::new("activate")}
}
fn attach(g:&mut Game,id:&str,host:&str,seat:usize)->String {
    let card=g.make_card(id,seat);let id=card.id.clone();g.attachments.push(Attachment{card,host_id:host.into()});id
}
fn room_checkpoint(g:&Game) {
    let r=envelope(g);let s=serde_json::to_string(&r).unwrap();
    assert_eq!(serde_json::to_string(&crate::room::RoomEnvelope::from_persisted(&s).unwrap()).unwrap(),s);
}

#[test]
fn batch4_clock_society_is_complete_red_dependency_but_public_construction_stays_closed() {
    let s=crate::society::definition("MSJC13").unwrap();
    assert_eq!(s.card.name,"计时者");assert_eq!(s.subtitle,"终末的阴谋家");assert_eq!(s.card.color,"红");
    assert_eq!(s.card.subtypes,["秘密教派","分支"]);assert_eq!(s.starting_hand,6);assert_eq!(s.printed_cost,None);
    assert!(s.card.unique && !s.card.supported);assert_eq!(definition("MSJC13").abilities.len(),2);
    assert!(definition("MSJC13").abilities.iter().all(|a|a.activation_only && a.timing==Timing::Standard));
    let mut d=deck::preset("watchers").unwrap();d.society_id=Some("MSJC13".into());
    assert!(deck::validate(d).unwrap_err().contains("尚未开放"));
    assert_eq!(catalog::catalog().societies.iter().filter(|s|s.card.supported).count(),11);
}
#[test]
fn batch4_clock_construction_counts_all_off_colors_together_at_twelve_boundary() {
    let s=crate::society::definition("MSJC13").unwrap();let cards=&catalog::catalog().cards;
    let ids=[("JC001",3),("JC029",3),("JC056",3),("JC073",3)];
    // Select one actual non-red/non-neutral definition per available color,
    // then count physical copies, not colors or entry rows.
    let mut entries=vec![catalog::DeckEntry{card_id:"JC125".into(),count:38}];
    for (id,n) in ids {assert!(!["红","中立"].contains(&catalog::card(id).color.as_str()));entries.push(catalog::DeckEntry{card_id:id.into(),count:n});}
    assert!(crate::society::validate_construction(s,&entries,cards).is_ok());
    entries[1].count+=1;assert!(crate::society::validate_construction(s,&entries,cards).unwrap_err().contains("13"));
    entries[1].card_id="JC042".into();assert_eq!(catalog::card("JC042").color,"红");
    assert!(crate::society::validate_construction(s,&entries,cards).is_ok());
}
#[test]
fn batch4_clock_add_time_pays_two_and_exhausts_once_then_resolves_on_enemy() {
    let(mut g,s)=layout();let t=field(&mut g,"JC029",2);fund(&mut g,0,"JC125",4);
    assert!(g.legal_actions(0).iter().any(|a|a.action==activate(&s,c::ADD,Some(t.clone()))));
    act(&mut g,0,activate(&s,c::ADD,Some(t.clone())));assert_eq!(g.resources(0),2);
    assert!(g.players[0].society_zone.card.as_ref().unwrap().exhausted);assert_eq!(g.board(&t).unwrap().1.time_markers,0);
    room_checkpoint(&g);pass_top(&mut g);assert_eq!(g.board(&t).unwrap().1.time_markers,1);
    reject(&mut g,0,activate(&s,c::ADD,Some(t)));assert_eq!(g.resources(0),2);
}
#[test]
fn batch4_clock_add_rejects_hidden_asset_society_and_extra_payload_before_payment() {
    let(mut g,s)=layout();let hidden=field(&mut g,"JC029",2);g.board_mut(&hidden).unwrap().face_down=true;
    fund(&mut g,0,"JC125",3);let asset=g.players[0].assets[0].id.clone();
    for t in [hidden,asset,s.clone(),"society:p0".into()] {reject(&mut g,0,activate(&s,c::ADD,Some(t)));}
    let t=field(&mut g,"JC029",2);let mut a=activate(&s,c::ADD,Some(t));a.selected=Some(vec![s.clone()]);reject(&mut g,0,a);
    assert_eq!(g.resources(0),3);assert!(!g.players[0].society_zone.card.as_ref().unwrap().exhausted);
}
#[test]
fn batch4_clock_paid_target_departure_cancels_without_rebinding_or_refund() {
    let(mut g,s)=layout();let t=field(&mut g,"JC029",2);fund(&mut g,0,"JC125",2);
    act(&mut g,0,activate(&s,c::ADD,Some(t.clone())));g.remove_dead(&t,RemovalCause::Destroy);
    let replacement=field(&mut g,"JC029",2);restore(&mut g);pass_top(&mut g);
    assert_eq!(g.board(&replacement).unwrap().1.time_markers,0);assert_eq!(g.resources(0),0);
    assert!(g.players[0].society_zone.card.as_ref().unwrap().exhausted);
}
#[test]
fn batch4_clock_target_shield_stops_time_but_sweep_is_not_targeted() {
    let(mut g,s)=layout();let t=field(&mut g,"JC029",2);g.board_mut(&t).unwrap().shield=1;fund(&mut g,0,"JC125",2);
    act(&mut g,0,activate(&s,c::ADD,Some(t.clone())));pass_top(&mut g);
    assert_eq!(g.board(&t).unwrap().1.time_markers,0);assert_eq!(g.board(&t).unwrap().1.shield,0);
    g.players[0].society_zone.card.as_mut().unwrap().exhausted=false;g.board_mut(&t).unwrap().time_markers=3;g.board_mut(&t).unwrap().shield=1;
    fund(&mut g,0,"JC125",3);act(&mut g,0,activate(&s,c::SWEEP,None));pass_top(&mut g);assert!(g.in_play(&t).is_none());assert_eq!(g.players[0].hand.len(),1);
}
#[test]
fn batch4_clock_sweep_counts_direct_qualifiers_not_unqualified_collateral() {
    for attachment_time in [0,3] {
        let(mut g,s)=layout();let host=field(&mut g,"JC029",2);let child=attach(&mut g,"BQ022",&host,2);
        g.board_mut(&host).unwrap().time_markers=if attachment_time==0{3}else{0};g.board_mut(&child).unwrap().time_markers=attachment_time;
        let hidden=field(&mut g,"JC029",3);g.board_mut(&hidden).unwrap().face_down=true;g.board_mut(&hidden).unwrap().time_markers=0;
        let low=field(&mut g,"JC029",1);g.board_mut(&low).unwrap().time_markers=2;
        fund(&mut g,0,"JC125",3);act(&mut g,0,activate(&s,c::SWEEP,None));room_checkpoint(&g);pass_top(&mut g);
        assert!(g.in_play(&host).is_none()&&g.in_play(&child).is_none());assert!(g.in_play(&hidden).is_some()&&g.in_play(&low).is_some());
        assert_eq!(g.players[0].hand.len(),if attachment_time==0{1}else{2});
    }
}
#[test]
fn batch4_clock_sweep_includes_attached_bats_as_their_own_qualifying_role() {
    for bat_time in [0,3] {
        let(mut g,s)=layout();let host=field(&mut g,"JC029",2);let bat=attach(&mut g,"BQ026",&host,2);
        g.board_mut(&host).unwrap().time_markers=if bat_time==0{3}else{0};g.board_mut(&bat).unwrap().time_markers=bat_time;fund(&mut g,0,"JC125",3);
        act(&mut g,0,activate(&s,c::SWEEP,None));pass_top(&mut g);assert!(g.in_play(&host).is_none()&&g.in_play(&bat).is_none());
        assert_eq!(g.players[0].hand.len(),if bat_time==0{1}else{2});assert_eq!(g.players[2].graveyard.len(),2);
    }
}
#[test]
fn batch4_clock_sweep_empty_set_draws_none_but_still_pays_and_exhausts() {
    let(mut g,s)=layout();fund(&mut g,0,"JC125",3);act(&mut g,0,activate(&s,c::SWEEP,None));pass_top(&mut g);
    assert!(g.players[0].hand.is_empty());assert_eq!(g.resources(0),0);assert!(g.players[0].society_zone.card.as_ref().unwrap().exhausted);
}
#[test]
fn batch4_clock_full_program_and_target_predicate_cannot_be_transplanted() {
    let mut defs=crate::rules::definitions().clone();let a=definition("MSJC13").abilities[0].clone();defs.get_mut("JC125").unwrap().abilities.push(a);
    assert!(crate::rules::validate_definitions(&defs).is_err());
    let mut defs=crate::rules::definitions().clone();defs.get_mut("MSJC13").unwrap().abilities.remove(1);
    assert!(crate::rules::validate_definitions(&defs).is_err());
}

fn wm_hand(g:&mut Game,ids:&[&str])->Vec<String> {ids.iter().map(|id|held(g,id,0)).collect()}
fn wm_deck(g:&mut Game,ids:&[&str]) {g.players[0].deck.clear();for id in ids{let c=g.make_card(id,0);g.players[0].deck.push(c);}}
fn wm_play(g:&mut Game)->String {
    fund(g,0,"JC049",2);let card=held(g,"WM037",0);
    act(g,0,Action{card_id:Some(card.clone()),..Action::new("play")});pass_top(g);card
}
fn start_other_contest(g:&mut Game) {
    fund(g,0,"WM022",3);let source=field(g,"WM022",0);g.board_mut(&source).unwrap().face_down=true;
    act(g,0,Action{card_id:Some(source),..Action::new("reveal")});
    for _ in 0..24 {if g.pending.is_some(){break;}pass(g);}
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,crate::mystic_contest::COMMIT_KIND);
}
fn commits(g:&mut Game,ids:[&str;4]) {
    for(seat,id)in ids.into_iter().enumerate(){assert_eq!(g.pending.as_ref().unwrap().seat,seat);
        let c=g.players[seat].hand.iter().find(|c|c.definition==id).unwrap().id.clone();choose(g,vec![c]);}
}
fn rewards(g:&mut Game) {
    for _ in 0..4 {if g.pending.as_ref().is_some_and(|p|p.choice.kind==crate::mystic_contest::REWARD_KIND){choose(g,vec![]);}else{break;}}
}
fn decline_triggers(g:&mut Game) {
    for _ in 0..16 {
        if let Some(p)=g.pending.as_ref() {
            if p.choice.kind=="batch3-entry-order" {let id=p.choice.options[0].id.clone();choose(g,vec![id]);}
            else {assert_eq!(p.choice.min,Some(0),"unexpected {}",p.choice.kind);choose(g,vec![]);}
        }else if !g.stack.is_empty(){pass(g);}else{break;}
    }
}

#[test]
fn batch4_clock_wm037_pinned_fields_and_canonical_standard_program() {
    crate::unit_support::assert_current_serial_catalog();assert_eq!(catalog::catalog().cards.iter().filter(|c|c.supported).count(),236);
    let d=catalog::card("WM037");assert_eq!((&*d.name,d.cost,&*d.color,&*d.kind),("戴森的计谋",2,"红","spell"));
    assert_eq!(d.loyalty,["红色"]);assert_eq!(d.magic_icon,MagicIcon::Mind);assert_eq!(d.subtypes,["阴谋"]);assert!(!d.supported&&!d.unique);
    let a=&definition("WM037").abilities[0];assert!(a.play_only&&!a.activation_only);assert_eq!(a.timing,Timing::Standard);assert!(a.costs.is_empty());
    let mut defs=crate::rules::definitions().clone();defs.get_mut("WM037").unwrap().abilities[0].ops.remove(0);assert!(crate::rules::validate_definitions(&defs).is_err());
}
#[test]
fn batch4_clock_wm037_pays_before_discard_then_takes_bottom_two_in_order() {
    let(mut g,_)=layout();wm_deck(&mut g,&["JC029","JC042","JC056"]);let discarded=wm_hand(&mut g,&["JC125"])[0].clone();
    wm_play(&mut g);assert_eq!(g.resources(0),0);assert_eq!(g.players[0].hand.len(),1);assert_eq!(g.players[0].deck.len(),3);
    assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Frame{choice:FrameChoice::Discard{..},..}));
    room_checkpoint(&g);choose(&mut g,vec![discarded]);
    assert_eq!(g.players[0].hand.iter().map(|c|c.definition.as_str()).collect::<Vec<_>>(),["JC056","JC042"]);
    assert_eq!(g.players[0].deck[0].definition,"JC029");assert!(g.pending.is_none());assert!(g.mystic_history.is_empty());
}
#[test]
fn batch4_clock_wm037_short_or_empty_bottom_is_not_a_draw_and_does_not_eliminate() {
    for n in [0,1] {let(mut g,_)=layout();wm_deck(&mut g,if n==0{&[]}else{&["JC125"]});wm_hand(&mut g,&["JC125"]);wm_play(&mut g);
        let discard=g.players[0].hand[0].id.clone();choose(&mut g,vec![discard]);
        assert_eq!(g.players[0].hand.len(),n);assert!(g.players[0].deck.is_empty());assert!(!g.players[0].eliminated);assert_eq!(g.status,"playing");}
}
#[test]
fn batch4_clock_wm037_no_hand_to_discard_still_continues_bottom_movement() {
    let(mut g,_)=layout();wm_deck(&mut g,&["JC042","JC056"]);wm_play(&mut g);
    assert!(g.pending.is_none());assert_eq!(g.players[0].hand.len(),2);assert!(!g.players[0].eliminated);
}
#[test]
fn batch4_clock_wm037_threshold_counts_physical_markers_once_and_decline_preserves_steps() {
    for n in [3,4] {let(mut g,_)=layout();let host=field(&mut g,"JC029",2);let child=attach(&mut g,"BQ022",&host,2);
        g.board_mut(&child).unwrap().time_markers=n;assert_eq!(g.effective_time_markers(g.board(&host).unwrap().1),n);
        let discard=wm_hand(&mut g,&["JC125"])[0].clone();wm_play(&mut g);choose(&mut g,vec![discard]);
        if n==3 {assert!(g.pending.is_none());}else{assert_eq!(g.pending.as_ref().unwrap().choice.kind,"wm037-optional-contest");room_checkpoint(&g);choose(&mut g,vec![]);}
        assert!(g.pending.is_none()&&g.mystic_contest.is_none()&&g.mystic_history.is_empty());assert_eq!(g.players[0].hand.len(),2);assert_eq!(g.resources(0),0);
    }
}
#[test]
fn batch4_clock_wm037_accept_reuses_private_shared_contest_and_all_seats_restore() {
    let(mut g,_)=layout();let t=field(&mut g,"JC029",2);g.board_mut(&t).unwrap().time_markers=4;
    let discard=wm_hand(&mut g,&["JC125"])[0].clone();wm_play(&mut g);choose(&mut g,vec![discard]);choose(&mut g,vec!["contest".into()]);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,crate::mystic_contest::COMMIT_KIND);
    commits(&mut g,["JC125","JC125","JC125","JC125"]);rewards(&mut g);
    assert!(g.pending.is_none()&&g.mystic_contest.is_none());assert_eq!(g.mystic_history.len(),1);assert_eq!(g.mystic_history[0].winner_team,None);room_checkpoint(&g);
}
#[test]
fn batch4_clock_wm037_illegal_play_or_optional_payload_is_atomic() {
    let(mut g,_)=layout();fund(&mut g,0,"JC125",2);let c=held(&mut g,"WM037",0);
    reject(&mut g,0,Action{card_id:Some(c.clone()),..Action::new("play")});
    fund(&mut g,0,"JC049",2);reject(&mut g,0,Action{card_id:Some(c.clone()),selected:Some(vec![]),..Action::new("play")});
    let t=field(&mut g,"JC029",2);g.board_mut(&t).unwrap().time_markers=4;let discard=held(&mut g,"JC125",0);
    act(&mut g,0,Action{card_id:Some(c),..Action::new("play")});pass_top(&mut g);choose(&mut g,vec![discard]);
    let p=g.pending.clone().unwrap();reject(&mut g,2,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec!["contest".into()]),..Action::new("choose")});
    reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec!["contest".into()]),region:Some(0),..Action::new("choose")});choose(&mut g,vec![]);
}
#[test]
fn batch4_clock_wm037_shown_bonus_is_unconditional_but_time_requires_personal_clockmakers() {
    for own in [false,true] {let(mut g,_)=layout();if !own{let card=g.players[0].society_zone.card.take();g.players[1].society_zone.card=card.map(|mut c|{c.owner=1;c.controller=1;c});}
        held(&mut g,"WM037",0);start_other_contest(&mut g);commits(&mut g,["WM037","JC125","JC125","JC125"]);
        assert_eq!(g.mystic_history[0].shown[0].value,3);rewards(&mut g);
        if own {assert_eq!(g.pending.as_ref().unwrap().choice.kind,"wm037-shown-time-target");let target=g.pending.as_ref().unwrap().choice.options[0].id.clone();choose(&mut g,vec![target]);}
        decline_triggers(&mut g);assert!(g.pending.is_none()&&g.mystic_contest.is_none());
    }
}
#[test]
fn batch4_clock_wm037_losing_shown_card_still_places_time_after_rewards() {
    let(mut g,_)=layout();let t=field(&mut g,"JC029",2);held(&mut g,"WM037",0);held(&mut g,"LC16",2);held(&mut g,"LC16",3);
    start_other_contest(&mut g);commits(&mut g,["WM037","JC125","LC16","LC16"]);assert_eq!(g.mystic_history[0].winner_team,Some(1));
    assert_eq!(g.board(&t).unwrap().1.time_markers,0);rewards(&mut g);choose(&mut g,vec![t.clone()]);assert_eq!(g.board(&t).unwrap().1.time_markers,1);
}
#[test]
fn batch4_clock_wm037_shown_target_is_mandatory_personal_private_and_shielded() {
    let(mut g,_)=layout();let t=field(&mut g,"JC029",2);g.board_mut(&t).unwrap().shield=1;let hidden=field(&mut g,"JC029",3);g.board_mut(&hidden).unwrap().face_down=true;
    held(&mut g,"WM037",0);start_other_contest(&mut g);commits(&mut g,["WM037","JC125","JC125","JC125"]);rewards(&mut g);
    let p=g.pending.clone().unwrap();assert_eq!((p.choice.min,p.choice.max,p.choice.allow_decline),(Some(1),Some(1),Some(false)));
    assert!(!p.choice.options.iter().any(|o|o.id==hidden));for seat in 1..4{assert!(g.view(seat).pending_choice.is_none());}room_checkpoint(&g);
    for selected in [vec![],vec![hidden],vec!["society:p0".into()]]{reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(selected),..Action::new("choose")});}
    choose(&mut g,vec![t.clone()]);assert_eq!(g.board(&t).unwrap().1.time_markers,0);assert_eq!(g.board(&t).unwrap().1.shield,0);assert!(g.mystic_contest.is_none());
}

#[test]
fn batch4_clock_time_add_and_sweep_include_secret_attachment_without_revealing_identity() {
    let(mut g,s)=layout();let id=attach(&mut g,"WM046","society:p2",2);g.board_mut(&id).unwrap().face_down=true;
    fund(&mut g,0,"JC125",2);act(&mut g,0,activate(&s,c::ADD,Some(id.clone())));pass_top(&mut g);
    assert_eq!(g.in_play(&id).unwrap().1.time_markers,1);
    for seat in [0,1,3]{let v=g.view(seat);let a=v.attachments.iter().find(|a|a.card.instance_id==id).unwrap();assert!(a.card.card_id.is_none());}
    g.board_mut(&id).unwrap().time_markers=3;g.players[0].society_zone.card.as_mut().unwrap().exhausted=false;
    fund(&mut g,0,"JC125",3);act(&mut g,0,activate(&s,c::SWEEP,None));pass_top(&mut g);
    assert!(g.in_play(&id).is_none());assert_eq!(g.players[0].hand.len(),1);assert_eq!(g.players[2].graveyard[0].definition,"WM046");
}

#[test]
fn batch4_clock_wm037_room_command_private_choice_retry_and_u64_use_normal_envelope() {
    let(mut g,_)=layout();g.random=(1u64<<63)+77;let t=field(&mut g,"JC029",2);g.board_mut(&t).unwrap().time_markers=4;
    let discard=wm_hand(&mut g,&["JC125"])[0].clone();wm_play(&mut g);choose(&mut g,vec![discard]);
    let room=envelope(&g);let command=crate::room::RoomCommand{command_id:"wm037-accept-once".into(),expected_version:room.revision,
        action:Action{choice_id:Some(g.pending.as_ref().unwrap().choice.id.clone()),selected:Some(vec!["contest".into()]),..Action::new("choose")}.into()};
    let out=room.transition(0,Some(command.clone()),1).unwrap();assert!(out.changed&&out.error_code.is_none(),"{:?}",out.error_message);
    assert_eq!(serde_json::to_string(&room.replay_events(&out.journal).unwrap()).unwrap(),out.state);
    let restored=crate::room::RoomEnvelope::from_persisted(&out.state).unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(),out.state);assert_eq!(restored.game.resources(0),0);
    assert_eq!(restored.game.pending.as_ref().unwrap().choice.kind,crate::mystic_contest::COMMIT_KIND);
    for seat in 1..4{assert!(restored.game.view(seat).pending_choice.is_none());}
    let duplicate=restored.transition(0,Some(command),2).unwrap();assert!(!duplicate.changed);assert_eq!(duplicate.error_code.as_deref(),Some("version_conflict"));assert_eq!(duplicate.state,out.state);
}

#[test]
fn batch4_clock_sweep_includes_own_ally_and_enemy_and_draws_after_the_whole_wave() {
    let(mut g,s)=layout();let mut ids=vec![];
    for seat in [0,1,2]{let id=field(&mut g,"JC029",seat);g.board_mut(&id).unwrap().time_markers=3;ids.push(id);}
    fund(&mut g,0,"JC125",3);act(&mut g,0,activate(&s,c::SWEEP,None));
    assert!(g.players[0].hand.is_empty());pass_top(&mut g);
    assert!(ids.iter().all(|id|g.in_play(id).is_none()));assert_eq!(g.players[0].hand.len(),3);
    for seat in [0,1,2]{assert_eq!(g.players[seat].graveyard.len(),1);}
}

#[test]
fn batch4_clock_wm037_and_bq027_shown_continuations_preserve_both_adapters_in_order() {
    let(mut g,_)=layout();let vampire=field(&mut g,"JC029",1);held(&mut g,"WM037",0);held(&mut g,"BQ027",1);
    start_other_contest(&mut g);commits(&mut g,["WM037","BQ027","JC125","JC125"]);rewards(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"wm037-shown-time-target");choose(&mut g,vec![vampire.clone()]);
    assert_eq!(g.board(&vampire).unwrap().1.time_markers,1);
    let p=g.pending.as_ref().unwrap();assert_eq!(p.seat,1);assert_eq!(p.choice.kind,crate::batch4_vampire_contest::TARGET);room_checkpoint(&g);
    choose(&mut g,vec![vampire.clone()]);decline_triggers(&mut g);
    assert_eq!(g.board(&vampire).unwrap().1.time_markers,1);assert!(g.mystic_contest.is_none()&&g.pending.is_none());
    assert_eq!(g.mystic_history[0].shown[0].value,3);assert_eq!(g.mystic_history[0].shown[1].value,3);
}

// Integrator regression: a single contest traverses all three received adapters.
// The source layouts are explicit; every contest/target step uses normal commands.
#[test]
fn serial_clock_sentinel_and_blood_shown_choices_preserve_all_three_adapters() {
    let (mut g, _) = layout();
    let sentinel = g.make_card("MSJC14", 1);
    g.players[1].society_zone.card = Some(sentinel);
    let target = field(&mut g, "JC029", 2);
    held(&mut g, "WM037", 0);
    held(&mut g, "WM043", 1);
    held(&mut g, "BQ027", 2);
    start_other_contest(&mut g);
    commits(&mut g, ["WM037", "WM043", "BQ027", "JC125"]);
    rewards(&mut g);

    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "wm037-shown-time-target");
    choose(&mut g, vec![target.clone()]);
    assert_eq!(g.board(&target).unwrap().1.time_markers, 1);

    assert_eq!(g.pending.as_ref().unwrap().seat, 1);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, crate::wm043::CHOICE);
    choose(&mut g, vec![target.clone()]);
    assert_eq!(g.board(&target).unwrap().1.lock_markers, 1);

    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, crate::batch4_vampire_contest::TARGET);
    choose(&mut g, vec![target.clone()]);
    decline_triggers(&mut g);
    assert_eq!(g.board(&target).unwrap().1.time_markers, 1);
    assert_eq!(g.board(&target).unwrap().1.lock_markers, 1);
    assert!(g.mystic_contest.is_none() && g.pending.is_none());
}

// Independent normal-play boundaries. The existing WIP layout supplies the
// unadmitted society and accumulated counters; interleavings use paid commands.
fn independent_clock_record(name: &str, value: serde_json::Value) {
    if let Ok(dir) = std::env::var("INDEPENDENT_HISTORY_OUT") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(std::path::Path::new(&dir).join(format!("{name}.json")),
            serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    }
}
fn independent_clock_hidden_timer(g: &mut Game) -> String {
    let id = field(g, "XQ18", 2);
    g.board_mut(&id).unwrap().face_down = true;
    fund(g, 2, "XQ18", 2);
    id
}
fn independent_clock_time_response(g: &mut Game, hidden: &str, target: &str, add: bool) {
    for _ in 0..4 { if g.can_fast(2) { break; } pass(g); }
    assert!(g.can_fast(2));
    act(g, 2, Action { card_id: Some(hidden.into()), ..Action::new("reveal") });
    pass_top(g);
    choose(g, vec![if add { "add-time".into() } else { "remove-time".into() }]);
    choose(g, vec![target.into()]);
    pass_top(g);
    if !add {
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "xq18-time-carrier");
        choose(g, vec![target.into()]);
    }
    assert!(g.pending.is_none());
}

#[test]
fn independent_clock_wm037_response_crosses_threshold_in_both_directions() {
    for add in [false, true] {
        let (mut g, _) = layout();
        let target = field(&mut g, "JC125", 2);
        g.board_mut(&target).unwrap().time_markers = if add { 3 } else { 4 };
        let timer = independent_clock_hidden_timer(&mut g);
        let discard = held(&mut g, "JC125", 0);
        fund(&mut g, 0, "JC049", 2);
        let source = held(&mut g, "WM037", 0);
        act(&mut g, 0, Action { card_id: Some(source), ..Action::new("play") });
        independent_clock_time_response(&mut g, &timer, &target, add);
        assert_eq!(g.board(&target).unwrap().1.time_markers, if add { 4 } else { 3 });
        pass_top(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "discard");
        choose(&mut g, vec![discard]);
        if add {
            assert_eq!(g.pending.as_ref().unwrap().choice.kind, "wm037-optional-contest");
            choose(&mut g, vec![]);
        } else { assert!(g.pending.is_none()); }
        assert!(g.mystic_contest.is_none() && g.mystic_history.is_empty());
        assert_eq!(g.players[0].hand.len(), 2);
        assert_eq!(g.resources(0), 0);
        assert_eq!(g.resources(2), 0);
        independent_clock_record(if add { "threshold-up" } else { "threshold-down" },
            serde_json::json!({"after": &g, "target": target, "optionalContestOffered": add}));
    }
}

#[test]
fn independent_clock_sweep_uses_real_transfer_then_current_response_markers() {
    let (mut g, society) = layout();
    let from = field(&mut g, "JC125", 2);
    let to = field(&mut g, "JC125", 2);
    let victim = field(&mut g, "JC125", 3);
    g.board_mut(&from).unwrap().time_markers = 3;
    g.board_mut(&victim).unwrap().time_markers = 3;
    let timer = independent_clock_hidden_timer(&mut g);
    fund(&mut g, 0, "XQ20", 2);
    fund(&mut g, 0, "JC125", 3);
    let transfer = held(&mut g, "XQ20", 0);
    // XQ20 is a standard action. It resolves BEFORE the separate society action.
    act(&mut g, 0, Action { card_id: Some(transfer),
        ability_id: Some(crate::batch4_soul_channel::KEY.into()),
        selected: Some(vec![from.clone(), to.clone()]), ..Action::new("play") });
    pass_top(&mut g);
    assert_eq!(g.board(&from).unwrap().1.time_markers, 0);
    assert_eq!(g.board(&to).unwrap().1.time_markers, 3);
    act(&mut g, 0, activate(&society, c::SWEEP, None));
    independent_clock_time_response(&mut g, &timer, &to, false);
    assert_eq!(g.board(&to).unwrap().1.time_markers, 2);
    pass_top(&mut g);
    assert!(g.board(&from).is_some() && g.board(&to).is_some());
    assert!(g.board(&victim).is_none());
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.resources(0), 0);
    assert!(g.players[0].society_zone.card.as_ref().unwrap().exhausted);
    independent_clock_record("transfer-response-sweep", serde_json::json!({
        "after": &g, "from": from, "to": to, "destroyed": victim, "cardsDrawn": 1
    }));
}

#[test]
fn independent_clock_wm037_shown_time_consumes_real_enemy_entry_shield() {
    let (mut g, _) = layout();
    g.begin_window(Window::Action(1));
    fund(&mut g, 2, "CQ17", 7);
    let id = held(&mut g, "BQ059", 2);
    play_card(&mut g, 2, &id);
    drain(&mut g);
    let target = g.regions[0].cards.iter().find(|c| c.definition == "BQ059").unwrap().id.clone();
    assert_eq!(g.board(&target).unwrap().1.shield, 2);
    g.begin_window(Window::Action(0));
    held(&mut g, "WM037", 0);
    start_other_contest(&mut g);
    commits(&mut g, ["WM037", "JC125", "JC125", "JC125"]);
    rewards(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "wm037-shown-time-target");
    choose(&mut g, vec![target.clone()]);
    decline_triggers(&mut g);
    assert_eq!((g.board(&target).unwrap().1.shield, g.board(&target).unwrap().1.time_markers), (1, 0));
    assert!(g.pending.is_none() && g.mystic_contest.is_none());
    independent_clock_record("shown-real-shield", serde_json::json!({"after": &g, "target": target}));
}

#[test]
fn independent_clock_room_sweep_short_draw_commits_elimination_and_terminal() {
    use crate::room::{Decision, RoomCommand, RoomEnvelope, SessionAction};
    for terminal in [false, true] {
        let (mut g, society) = layout();
        if terminal {
            g.players[1].deck.clear();
            g.draw(1, 1).unwrap();
            assert!(g.players[1].eliminated);
        }
        wm_deck(&mut g, &["JC125"]);
        let a = field(&mut g, "JC125", 2);
        let b = field(&mut g, "JC125", 3);
        for id in [&a, &b] { g.board_mut(id).unwrap().time_markers = 3; }
        fund(&mut g, 0, "JC125", 3);
        act(&mut g, 0, activate(&society, c::SWEEP, None));
        let mut room = envelope(&g);
        let mut transitions = vec![];
        for n in 0..16 {
            if room.game.stack.is_empty() || room.game.status == "finished" { break; }
            let w = room.pacing.window.clone().expect("normal response window");
            let seat = *w.members.iter().find(|(_, d)| !matches!(d, Decision::Passed)).unwrap().0;
            let cmd = RoomCommand { command_id: format!("independent-clock-sweep-{terminal}-{n}"),
                expected_version: room.revision,
                action: SessionAction::PassResponse { window_id: w.id },
            };
            let out = room.transition(seat, Some(cmd.clone()), 0).unwrap();
            assert_eq!(out.outcome, "accepted", "{:?}", out.error_message);
            transitions.push(serde_json::json!({"seat": seat, "command": cmd, "result": &out}));
            room = RoomEnvelope::from_persisted(&out.state).unwrap();
        }
        assert!(room.game.players[0].eliminated);
        assert!(room.game.board(&a).is_none() && room.game.board(&b).is_none());
        assert!(room.game.pending.is_none() && room.game.stack.is_empty());
        assert_eq!(room.game.status, if terminal { "finished" } else { "playing" });
        if terminal { assert_eq!(room.game.winner_team, Some(1)); }
        for viewer in 0..4 { let _ = room.view(viewer, 0); }
        independent_clock_record(if terminal { "sweep-terminal" } else { "sweep-elimination" },
            serde_json::json!({"after": &room, "transitions": transitions}));
    }
}
