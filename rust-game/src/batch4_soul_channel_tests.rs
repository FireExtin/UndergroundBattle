//! Disclosed Native layouts; actual play/payment/response commands, not helper-only acceptance.
use crate::{batch4_markers::MarkerKind, catalog, model::*, rules::*, unit_support::*};
const KEY: &str = crate::batch4_soul_channel::KEY;
fn command(card: &str, from: &str, to: &str) -> Action {
    Action { card_id: Some(card.into()), ability_id: Some(KEY.into()), selected: Some(vec![from.into(), to.into()]), ..Action::new("play") }
}
fn layout() -> (Game, String, String, String) {
    let mut g = game(); fund(&mut g, 0, "XQ20", 6);
    let from = field(&mut g, "BQ079", 0); let to = field(&mut g, "BQ079", 0); let card = held(&mut g, "XQ20", 0);
    (g, card, from, to)
}
fn resolve_spell(g: &mut Game, id: &str) {
    for _ in 0..64 {
        if !g.stack.iter().any(|i| i.frame.as_ref().is_some_and(|f| f.source.card.id == id)) { return; }
        assert!(g.pending.is_none(), "counter move itself cannot pause for a choice"); pass(g);
    }
    panic!("bounded XQ20 resolution");
}
fn plan(g: &mut Game, printed: &str, secret: bool, n: u32) -> String {
    let card = held(g, printed, 0);
    act(g, 0, Action { card_id: Some(card), target_id: Some("society:p0".into()), option: secret.then(|| "secret-plan".into()), ..Action::new("play") }); top(g);
    let id = g.attachments.last().unwrap().card.id.clone();
    for _ in 0..n { act(g, 0, Action { card_id: Some(id.clone()), ability_id: Some(crate::batch4_plans::ADVANCE.into()), ..Action::new("activate") }); top(g); }
    id
}

#[test]
fn batch4_soul_channel_printed_fields_timing_and_closed_public_admission() {
    let d = catalog::card("XQ20"); assert_eq!(d.cost, 2); assert_eq!(d.color, "红"); assert_eq!(d.magic, "鲜血");
    assert_eq!(d.subtypes, vec!["法术", "生命"]); assert!(!d.supported);
    let d = definition("XQ20"); let a = &d.abilities[0];
    assert_eq!(a.timing, Timing::Standard); assert!(a.play_only); assert_eq!(a.targets.len(), 2);
    let mut changed = crate::rules::definitions().clone(); changed.get_mut("JC062").unwrap().abilities[0] = a.clone();
    assert!(crate::rules::validate_definitions(&changed).is_err());
}
#[test]
fn batch4_soul_channel_paid_move_preserves_seven_categories_and_both_physical_uids() {
    let (mut g, card, from, to) = layout();
    { let c = g.board_mut(&from).unwrap(); for k in MarkerKind::ALL { *k.field(c) = 1; } }
    let resources = g.resources(0); let a = command(&card, &from, &to);
    assert!(g.legal_actions(0).iter().any(|v| v.action == a));
    act(&mut g, 0, a.clone()); assert_eq!(g.resources(0), resources-2); reject(&mut g, 0, a);
    resolve_spell(&mut g, &card);
    for k in MarkerKind::ALL { assert_eq!(k.count(g.in_play(&from).unwrap().1), 0); assert_eq!(k.count(g.in_play(&to).unwrap().1), 1); }
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "XQ20").count(), 1);
    for seat in 0..4 { let v = g.view(seat); let c = v.regions[0].characters.iter().find(|c| c.instance_id == to).unwrap();
        assert_eq!(c.advancement_markers, Some(1)); assert_eq!(c.lock_markers, Some(1)); assert_eq!(c.time_markers, Some(1)); }
}
#[test]
fn batch4_soul_channel_pair_order_candidates_include_enemy_controller_but_not_teammate_relation() {
    let (mut g, card, from, to) = layout(); let ally = field(&mut g, "JC125", 1);
    let a = field(&mut g, "JC125", 2); let b = field(&mut g, "JC125", 2); g.board_mut(&a).unwrap().time_markers = 2;
    let c = g.players[0].hand.iter().find(|c| c.id == card).unwrap(); let options = g.xq20_action_candidates(0, c);
    for pair in [(&from,&to),(&to,&from),(&a,&b),(&b,&a)] { assert!(options.iter().any(|(v,_)| *v == command(&card,pair.0,pair.1))); }
    assert!(!options.iter().any(|(v,_)| *v == command(&card,&from,&ally)));
    act(&mut g, 0, command(&card,&a,&b)); resolve_spell(&mut g,&card);
    assert_eq!(g.board(&a).unwrap().1.time_markers,0); assert_eq!(g.board(&b).unwrap().1.time_markers,2);
}
#[test]
fn batch4_soul_channel_bad_target_counts_hidden_private_and_extra_payload_reject_before_payment() {
    let (mut g, card, from, to) = layout(); let ally = field(&mut g,"JC125",1); let hidden = field(&mut g,"JC125",0);
    g.board_mut(&hidden).unwrap().face_down=true; let private=held(&mut g,"JC125",0); let asset=g.players[0].assets[0].id.clone();
    for targets in [vec![],vec![from.clone()],vec![from.clone(),to.clone(),from.clone()],vec![from.clone(),from.clone()],vec![from.clone(),ally],vec![hidden,to.clone()],vec![private,to.clone()],vec![asset,to.clone()],vec!["region:0".into(),to.clone()],vec!["society:p0".into(),to.clone()]] {
        let resources=g.resources(0);reject(&mut g,0,Action{card_id:Some(card.clone()),selected:Some(targets),..Action::new("play")});assert_eq!(g.resources(0),resources);
    }
    for mode in 0..5 { let mut a=command(&card,&from,&to); match mode {0=>a.target_id=Some(from.clone()),1=>a.cost_selected=Some(vec![]),2=>a.region=Some(0),3=>a.option=Some("partial".into()),_=>a.ability_id=Some("other".into())};reject(&mut g,0,a); }
}
#[test]
fn batch4_soul_channel_real_loyalty_and_two_asset_cost_are_required() {
    for funding in ["JC029","JC057"] {let mut g=game();fund(&mut g,0,funding,2);let from=field(&mut g,"JC125",0);let to=field(&mut g,"JC125",0);let c=held(&mut g,"XQ20",0);
        reject(&mut g,0,command(&c,&from,&to));assert_eq!(g.resources(0),2);}
    // The printed Blood icon is a possessed domain, not another play cost.
    let mut g=game();fund(&mut g,0,"XQ21",2);let from=field(&mut g,"JC125",0);let to=field(&mut g,"JC125",0);let c=held(&mut g,"XQ20",0);
    act(&mut g,0,command(&c,&from,&to));resolve_spell(&mut g,&c);assert_eq!(g.resources(0),0);
    let (mut g,c,from,to)=layout();g.players[0].assets.truncate(1);reject(&mut g,0,command(&c,&from,&to));assert_eq!(g.resources(0),1);
}
#[test]
fn batch4_soul_channel_standard_action_is_not_a_response_or_other_players_turn() {
    let (mut g,c,from,to)=layout();let a=command(&c,&from,&to);
    g.begin_window(Window::Before(0,0));reject(&mut g,0,a.clone());g.begin_window(Window::Action(1));reject(&mut g,0,a.clone());
    g.begin_window(Window::Action(0));let second=held(&mut g,"XQ20",0);act(&mut g,0,a);reject(&mut g,0,command(&second,&from,&to));
}
#[test]
fn batch4_soul_channel_zero_counters_is_a_paid_noop_and_damage_is_not_a_marker() {
    let (mut g,c,from,to)=layout();let before=(&g.board(&from).unwrap().1.clone(),&g.board(&to).unwrap().1.clone());
    let original=(before.0.clone(),before.1.clone());act(&mut g,0,command(&c,&from,&to));resolve_spell(&mut g,&c);
    assert_eq!(*g.board(&from).unwrap().1,original.0);assert_eq!(*g.board(&to).unwrap().1,original.1);
    assert_eq!(g.resources(0),4);
}
#[test]
fn batch4_soul_channel_same_controller_rechecked_before_consuming_shield_or_any_counter() {
    let (mut g,c,from,to)=layout();g.board_mut(&from).unwrap().time_markers=2;
    act(&mut g,0,command(&c,&from,&to));
    // Disclosed response-layout control change; not attributed to a separate card.
    g.board_mut(&to).unwrap().controller=2;g.board_mut(&to).unwrap().shield=1;restore(&mut g);
    resolve_spell(&mut g,&c);assert_eq!(g.board(&from).unwrap().1.time_markers,2);assert_eq!(g.board(&to).unwrap().1.time_markers,0);
    assert_eq!(g.board(&to).unwrap().1.shield,1);assert_eq!(g.resources(0),4);
}
#[test]
fn batch4_soul_channel_real_rescue_invalidates_uid_and_keeps_paid_cost() {
    let (mut g,c,_,to)=layout();let from=field(&mut g,"JZ51",0);g.board_mut(&from).unwrap().time_markers=2;let rescuer=field(&mut g,"JC075",0);
    act(&mut g,0,command(&c,&from,&to));for _ in 0..4 {if g.can_fast(0){break;}pass(&mut g);}
    let a=g.legal_actions(0).into_iter().find(|v|v.action.card_id.as_deref()==Some(&rescuer)&&v.action.target_id.as_deref()==Some(&from)).unwrap().action;
    act(&mut g,0,a);top(&mut g);assert!(g.in_play(&from).is_none());resolve_spell(&mut g,&c);
    assert_eq!(g.board(&to).unwrap().1.time_markers,0);let returned=g.players[0].hand.iter().find(|c|c.definition=="JZ51").unwrap();assert_ne!(returned.id,from);assert_eq!(returned.time_markers,0);
}
#[test]
fn batch4_soul_channel_defense_and_wounds_move_atomically_without_intermediate_death() {
    let (mut g,c,from,to)=layout();g.board_mut(&from).unwrap().permanent_defense_markers=1;g.board_mut(&from).unwrap().wounds=1;
    act(&mut g,0,command(&c,&from,&to));resolve_spell(&mut g,&c);
    for id in [&from,&to] {assert!(g.in_play(id).is_some());assert_eq!(g.defense(g.in_play(id).unwrap().1,0),1);}
    assert_eq!(g.board(&from).unwrap().1.wounds,0);assert_eq!(g.board(&to).unwrap().1.wounds,1);
}
#[test]
fn batch4_soul_channel_inherited_nested_markers_move_once_without_moving_attachments() {
    let (mut g,c,_,to)=layout();let from=field(&mut g,"JC029",0);let a=g.make_card("BQ026",0);let child=a.id.clone();g.attachments.push(Attachment{card:a,host_id:from.clone()});
    let a=g.make_card("BQ026",0);let nested=a.id.clone();g.attachments.push(Attachment{card:a,host_id:child.clone()});
    for (id,n) in [(&from,1),(&child,2),(&nested,3)] {g.board_mut(id).unwrap().time_markers=n;}
    let hosts=g.attachments.iter().map(|a|a.host_id.clone()).collect::<Vec<_>>();act(&mut g,0,command(&c,&from,&to));resolve_spell(&mut g,&c);
    for id in [&from,&child,&nested] {assert_eq!(g.in_play(id).unwrap().1.time_markers,0);}assert_eq!(g.board(&to).unwrap().1.time_markers,6);
    assert_eq!(g.attachments.iter().map(|a|a.host_id.clone()).collect::<Vec<_>>(),hosts);
}
#[test]
fn batch4_soul_channel_user_ruling_tianyan_one_to_three_is_one_real_completion() {
    let mut g=game();fund(&mut g,0,"WM046",8);fund(&mut g,0,"XQ20",2);g.add_society_markers(0,0,1);
    let from=plan(&mut g,"WM047",false,2);let to=plan(&mut g,"WM046",true,1);let c=held(&mut g,"XQ20",0);
    act(&mut g,0,command(&c,&from,&to));resolve_spell(&mut g,&c);
    assert_eq!(g.in_play(&from).unwrap().1.advancement_markers,0);let target=g.in_play(&to).unwrap().1;assert_eq!(target.advancement_markers,3);assert!(!target.face_down);
    let frames=g.stack.iter().filter_map(|i|i.frame.as_ref()).filter(|f|f.ability_key=="batch4-plan-complete"&&f.source.card.id==to).collect::<Vec<_>>();
    assert_eq!(frames.len(),1);assert_eq!(frames[0].source.card.advancement_markers,3);assert_eq!(frames[0].actor,0);
    for seat in 0..4 {let v=g.view(seat);let c=&v.attachments.iter().find(|a|a.card.instance_id==to).unwrap().card;assert_eq!(c.card_id.as_deref(),Some("WM046"));assert_eq!(c.advancement_markers,Some(3));}
    top(&mut g);assert!(g.in_play(&to).is_none());assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="WM046").count(),1);
}
#[test]
fn batch4_soul_channel_advancement_can_move_to_character_then_xq21_copy_without_advancing_plan() {
    let mut g=game();fund(&mut g,0,"WM046",6);fund(&mut g,0,"XQ20",3);let from=plan(&mut g,"WM047",false,2);let to=field(&mut g,"JC125",0);let c=held(&mut g,"XQ20",0);
    act(&mut g,0,command(&c,&from,&to));resolve_spell(&mut g,&c);assert_eq!(g.board(&to).unwrap().1.advancement_markers,2);
    let fee=field(&mut g,"JC125",0);let copy=held(&mut g,"XQ21",0);act(&mut g,0,Action{card_id:Some(copy),target_id:Some(to.clone()),cost_selected:Some(vec![fee]),..Action::new("play")});
    until_choice(&mut g,"xq21-marker");choose(&mut g,vec!["advancement".into()]);assert_eq!(g.board(&to).unwrap().1.advancement_markers,3);assert_eq!(g.in_play(&from).unwrap().1.advancement_markers,0);
}

#[test]
fn batch4_soul_channel_move_to_attachment_changes_host_queries_and_keeps_physical_counters_visible() {
    let (mut g,c,from,_)=layout();let host=field(&mut g,"JC029",0);let a=g.make_card("BQ026",0);let to=a.id.clone();g.attachments.push(Attachment{card:a,host_id:host.clone()});
    let combat=g.current_icons(g.board(&host).unwrap().1,0).combat;let defense=g.defense(g.board(&host).unwrap().1,0);
    {let a=g.board_mut(&from).unwrap();a.permanent_combat_markers=1;a.permanent_defense_markers=1;a.wounds=1;a.time_markers=2;a.lock_markers=1;}
    act(&mut g,0,command(&c,&from,&to));resolve_spell(&mut g,&c);
    assert_eq!(g.current_icons(g.board(&host).unwrap().1,0).combat,combat+1);assert_eq!(g.defense(g.board(&host).unwrap().1,0),defense);
    assert_eq!(g.effective_time_markers(g.board(&host).unwrap().1),2);assert_eq!(g.effective_marker_count(g.board(&host).unwrap().1,MarkerKind::Lock),1);
    for seat in 0..4 {let v=g.view(seat);let a=&v.attachments.iter().find(|a|a.card.instance_id==to).unwrap().card;
        assert_eq!(a.permanent_combat_markers,Some(1));assert_eq!(a.permanent_defense_markers,Some(1));assert_eq!(a.lock_markers,Some(1));}
    let checker=g.make_card("JC069",0);let s=g.source_snapshot(&checker,None);let mut t=target(Zone::Board,EntityKind::Character,Relation::Any,Range::Anywhere);t.predicate=Some(TargetPredicate::JC069LockedAnchor);
    assert!(g.valid_binding(0,&s,&t,&host),"normal locked-target query sees the inherited physical lock");
}
#[test]
fn batch4_soul_channel_inherited_shield_is_consumed_once_by_real_enemy_targeted_play() {
    // Use a legal equipment host, rather than BQ026 which separately protects
    // its host from all enemy spells before any shield can be consumed.
    let (mut g,c,from,other)=layout();let host=field(&mut g,"JC029",0);let a=g.make_card("JC020",0);let to=a.id.clone();g.attachments.push(Attachment{card:a,host_id:host.clone()});
    g.board_mut(&from).unwrap().shield=1;act(&mut g,0,command(&c,&from,&to));resolve_spell(&mut g,&c);assert_eq!(g.in_play(&to).unwrap().1.shield,1);
    fund(&mut g,2,"XQ20",2);g.board_mut(&other).unwrap().time_markers=2;let enemy=held(&mut g,"XQ20",2);g.begin_window(Window::Action(1));
    act(&mut g,2,command(&enemy,&other,&host));resolve_spell(&mut g,&enemy);
    assert_eq!(g.in_play(&to).unwrap().1.shield,0);assert_eq!(g.board(&other).unwrap().1.time_markers,2);assert_eq!(g.board(&host).unwrap().1.time_markers,0);assert_eq!(g.resources(2),0);
}
#[test]
fn batch4_soul_channel_secret_plan_exposes_public_moved_counters_without_exposing_print() {
    let (mut g,c,from,_)=layout();fund(&mut g,0,"WM046",2);g.add_society_markers(0,0,1);let to=plan(&mut g,"WM047",true,0);
    {let c=g.board_mut(&from).unwrap();for k in MarkerKind::ALL {*k.field(c)=1;}}
    act(&mut g,0,command(&c,&from,&to));resolve_spell(&mut g,&c);
    for seat in 0..4 {let v=g.view(seat);let a=&v.attachments.iter().find(|a|a.card.instance_id==to).unwrap().card;
        assert_eq!(a.advancement_markers,Some(1));assert_eq!(a.time_markers,Some(1));assert_eq!(a.lock_markers,Some(1));assert_eq!(a.wounds,Some(1));assert_eq!(a.shield,Some(1));
        if seat!=0 {assert!(a.card_id.is_none()&&a.text.is_none());assert_eq!(a.name,"秘密计划");}}
}

#[test]
fn batch4_soul_channel_does_not_move_damage_or_change_orientation() {
    let (mut g,c,_,_)=layout();let from=field(&mut g,"JZ51",0);let to=field(&mut g,"JZ51",0);
    {let c=g.board_mut(&from).unwrap();c.damage=1;c.exhausted=true;c.time_markers=2;}g.board_mut(&to).unwrap().damage=1;
    act(&mut g,0,command(&c,&from,&to));resolve_spell(&mut g,&c);
    assert_eq!(g.board(&from).unwrap().1.damage,1);assert!(g.board(&from).unwrap().1.exhausted);assert_eq!(g.board(&from).unwrap().1.time_markers,0);
    assert_eq!(g.board(&to).unwrap().1.damage,1);assert!(!g.board(&to).unwrap().1.exhausted);assert_eq!(g.board(&to).unwrap().1.time_markers,2);
}
#[test]
fn batch4_soul_channel_room_legal_commands_preserve_pair_and_commit_only_once() {
    use crate::room::{RoomCommand,RoomEnvelope,SessionAction};
    let (mut g,c,from,to)=layout();g.board_mut(&from).unwrap().time_markers=2;g.seed=u64::MAX-95;g.random=g.seed;
    let mut r=crate::jc029_tests::envelope(&g);
    let cmd=RoomCommand{command_id:"xq20-normal-play".into(),expected_version:r.revision,action:SessionAction::Game{action:command(&c,&from,&to)}};
    let step=r.transition(0,Some(cmd.clone()),1000).unwrap();assert!(step.error_code.is_none(),"{:?}",step.error_message);r=RoomEnvelope::from_persisted(&step.state).unwrap();
    let duplicate=r.transition(0,Some(cmd),1001).unwrap();assert_eq!(duplicate.error_code.as_deref(),Some("version_conflict"));assert_eq!(duplicate.state,step.state);
    for n in 0..32 {if !r.game.stack.iter().any(|i|i.frame.as_ref().is_some_and(|f|f.source.card.id==c)){break;}
        let id=r.pacing.window.as_ref().unwrap().id.clone();let seat=(0..4).find(|s|r.game.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();
        let cmd=RoomCommand{command_id:format!("xq20-pass-{n}"),expected_version:r.revision,action:SessionAction::PassResponse{window_id:id}};
        let step=r.transition(seat,Some(cmd),1100+n).unwrap();assert!(step.error_code.is_none(),"{:?}",step.error_message);r=RoomEnvelope::from_persisted(&step.state).unwrap();
    }
    assert!(r.game.stack.is_empty());assert_eq!(r.game.resources(0),4);assert_eq!(r.game.board(&from).unwrap().1.time_markers,0);assert_eq!(r.game.board(&to).unwrap().1.time_markers,2);assert_eq!(r.game.seed,u64::MAX-95);
    for seat in 0..4 {let v=r.view(seat,2000);assert_eq!(v.game.regions[0].characters.iter().find(|c|c.instance_id==to).unwrap().time_markers,Some(2));}
}
// Independent normal-play follow-up. Layouts are disclosed; no corrupted
// persisted-state input or recovery-boundary matrix is introduced here.
mod independent_normal_review {
    use super::*;
    fn record(name: &str, g: &Game) {
        if let Ok(dir) = std::env::var("INDEPENDENT_HISTORY_OUT") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(format!("{dir}/{name}.json"), serde_json::to_vec_pretty(&serde_json::json!({
                "game":g, "views":(0..4).map(|s|g.view(s)).collect::<Vec<_>>()
            })).unwrap()).unwrap();
        }
    }
    #[test]
    fn independent_xq20_actual_host_child_roundtrip_preserves_all_physical_markers_once() {
        let mut g = game(); fund(&mut g, 0, "XQ20", 4);
        let host = field(&mut g, "JC029", 0);
        let c = g.make_card("BQ026", 0); let child = c.id.clone();
        assert!(g.valid_binding(0, &g.source_snapshot(&c, None),
            &definition("BQ026").attachment.as_ref().unwrap().host, &host));
        g.attachments.push(Attachment { card: c, host_id: host.clone() });
        // BQ026 needs a Vampire host; another Bat is not a legal nested host.
        // JZ41 is a non-equipment attachment allowed on this character.
        let c = g.make_card("JZ41", 0); let nested = c.id.clone();
        assert!(g.valid_binding(0, &g.source_snapshot(&c, None),
            &definition("JZ41").attachment.as_ref().unwrap().host, &child));
        g.attachments.push(Attachment { card: c, host_id: child.clone() });
        for (id, amount) in [(&host, 1), (&child, 2), (&nested, 3)] {
            for kind in MarkerKind::ALL { *kind.field(g.board_mut(id).unwrap()) = amount; }
        }
        let hosts = g.attachments.iter().map(|a| (a.card.id.clone(), a.host_id.clone())).collect::<Vec<_>>();
        for (label, from, to) in [("into-child", &host, &child), ("back-to-host", &child, &host)] {
            let card = held(&mut g, "XQ20", 0);
            act(&mut g, 0, command(&card, from, to)); resolve_spell(&mut g, &card);
            for kind in MarkerKind::ALL {
                for id in [&host, &child, &nested] {
                    assert_eq!(kind.count(g.in_play(id).unwrap().1), if id == to { 6 } else { 0 });
                }
                assert_eq!(g.effective_marker_count(g.in_play(&host).unwrap().1, kind), 6);
            }
            assert_eq!(g.attachments.iter().map(|a| (a.card.id.clone(), a.host_id.clone())).collect::<Vec<_>>(), hosts);
            record(label, &g);
        }
        assert_eq!(g.resources(0), 0);
    }
    #[test]
    fn independent_xq20_real_reveal_response_changes_control_and_cancels_whole_move() {
        let mut g = game(); fund(&mut g, 0, "XQ20", 4); fund(&mut g, 2, "XQ16", 6);
        let from = field(&mut g, "JC125", 0); let to = field(&mut g, "JC125", 0);
        g.board_mut(&from).unwrap().time_markers = 2;
        let vampire = field(&mut g, "JZ27", 2); g.board_mut(&vampire).unwrap().face_down = true;
        let card = held(&mut g, "XQ20", 0); act(&mut g, 0, command(&card, &from, &to));
        for _ in 0..4 { if g.can_fast(2) { break; } pass(&mut g); }
        assert!(g.can_fast(2));
        act(&mut g, 2, Action { card_id: Some(vampire), ..Action::new("reveal") });
        crate::jc029_tests::pass_top(&mut g);
        choose(&mut g, vec![to.clone()]);
        crate::jc029_tests::pass_top(&mut g);
        assert_eq!(g.board(&to).unwrap().1.controller, 2);
        assert_eq!(g.board(&from).unwrap().1.controller, 0);
        assert!(g.stack.iter().any(|i| i.frame.as_ref().is_some_and(|f| f.source.card.id == card)));
        record("actual-control-response-before-original-resolution", &g);
        resolve_spell(&mut g, &card);
        assert_eq!(g.board(&from).unwrap().1.time_markers, 2);
        assert_eq!(g.board(&to).unwrap().1.time_markers, 0);
        assert_eq!(g.resources(0), 2);
        record("actual-control-response-cancelled", &g);
    }
    #[test]
    fn independent_xq20_sender_dies_after_losing_defense_only_after_destination_receives_batch() {
        let mut g = game(); fund(&mut g, 0, "XQ20", 2);
        let from = field(&mut g, "JC125", 0); let to = field(&mut g, "LC01", 0);
        { let c = g.board_mut(&from).unwrap(); c.permanent_defense_markers = 1; c.damage = 1; c.time_markers = 2; }
        g.board_mut(&to).unwrap().damage = 1;
        assert!(g.defense(g.board(&from).unwrap().1, 0) > 1);
        assert!(g.defense(g.board(&to).unwrap().1, 0) > 1);
        record("source-defense-before", &g);
        let card = held(&mut g, "XQ20", 0); act(&mut g, 0, command(&card, &from, &to)); resolve_spell(&mut g, &card);
        assert!(g.in_play(&from).is_none());
        assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "JC125").count(), 1);
        let c = g.board(&to).unwrap().1;
        assert_eq!((c.permanent_defense_markers, c.time_markers, c.damage), (1, 2, 1));
        assert_eq!(g.resources(0), 0);
        record("source-defense-after", &g);
    }
}
