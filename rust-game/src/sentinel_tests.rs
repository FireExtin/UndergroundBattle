//! Legal society selection/start; explicit board/hand layouts followed by real actions.
use crate::{catalog, deck, model::*, rules::*, unit_support::*};
pub(crate) fn sentinel_game() -> Game {
    let mut d=deck::preset("watchers").unwrap();d.society_id=Some("MSJC14".into());
    d.cards=vec![catalog::DeckEntry{card_id:"JC125".into(),count:50}];
    let mut g=Game::new_with_deck("sentinel".into(),"LOCAL".into(),"teams".into(),"P0".into(),d.clone(),8).unwrap();
    for s in 1..4 { g.join_with_deck(format!("P{s}"),d.clone()).unwrap(); }
    for s in 0..4 { g.apply(s,Action::new("ready")).unwrap(); }
    g.apply(0,Action::new("start")).unwrap();while g.pending.is_some(){choose(&mut g,vec![]);}
    for p in &mut g.players { p.hand.clear();p.assets.clear(); }
    for r in &mut g.regions { r.cards.clear();r.influence=[0;2]; }
    g.regions[0].card=g.make_card("DQJC115",0);g.first_team=0;g.begin_window(Window::Action(0));g
}
fn hidden(g:&mut Game,s:usize,r:usize)->String {
    let mut c=g.make_card("JC125",s);c.face_down=true;let id=c.id.clone();g.regions[r].cards.push(c);id
}
fn action(g:&Game,seat:usize,key:&str,target:&str,anchor:Option<&str>)->Action {
    g.legal_actions(seat).into_iter().find(|a|a.action.ability_id.as_deref()==Some(key)
        && a.action.target_id.as_deref()==Some(target)
        && a.action.selected.as_ref().map(|v|v[0].as_str())==anchor).expect("real legal society action").action
}
#[test]
fn sentinel_original_fields_and_full_registry() {
    let s=crate::society::definition("MSJC14").unwrap();assert_eq!((&*s.card.color,&*s.card.name,s.starting_hand,s.printed_cost),("灰","哨兵",6,None));
    assert_eq!(s.subtitle,"国际超自然监控中心");assert!(s.card.unique && s.card.supported && s.unresolved_abilities.is_empty());
    assert_eq!(definition("MSJC14").abilities.len(),2);validate_definitions(definitions()).unwrap();
}
#[test]
fn sentinel_construction_uses_magic_not_loyalty() {
    let society=crate::society::definition("MSJC14").unwrap();
    for color in ["灰","中立","蓝","红","黑","绿"] { for magic in [MagicIcon::None,MagicIcon::Blood,MagicIcon::Mind,MagicIcon::Other("火".into())] {
        let mut c=catalog::card("JC125").clone();c.color=color.into();c.magic_icon=magic.clone();c.loyalty=vec![color.into(),color.into()];
        let entries=vec![catalog::DeckEntry{card_id:c.id.clone(),count:50}];
        assert_eq!(crate::society::validate_construction(society,&entries,&[c]).is_ok(),["灰","中立"].contains(&color)||magic==MagicIcon::None);
    }}
    let g=sentinel_game();assert!(g.players.iter().all(|p|p.society_zone.card.as_ref().unwrap().definition=="MSJC14"));restore(&mut g.clone());
}
#[test]
fn sentinel_paid_lock_enemy_hidden_only_and_visible_marker() {
    let mut g=sentinel_game();fund(&mut g,0,"JC125",3);let enemy=hidden(&mut g,2,1);hidden(&mut g,1,1);field(&mut g,"JC125",2);
    let actions=g.legal_actions(0).into_iter().filter(|a|a.action.ability_id.as_deref()==Some("sentinel-lock")).collect::<Vec<_>>();
    assert_eq!(actions.len(),1);let a=action(&g,0,"sentinel-lock",&enemy,None);act(&mut g,0,a);
    assert_eq!(g.resources(0),0);assert!(g.players[0].society_zone.card.as_ref().unwrap().exhausted);top(&mut g);
    assert_eq!(g.board(&enemy).unwrap().1.lock_markers,1);
    for s in [0,1,3] {let v=g.card_view(g.board(&enemy).unwrap().1,s,Some(1),None);assert_eq!(v.lock_markers,Some(1));assert!(v.card_id.is_none());}
}
#[test]
fn sentinel_move_independent_targets_any_controller_anchor() {
    for owner in 0..4 { for concealed in [false,true] {
        let mut g=sentinel_game();fund(&mut g,0,"JC125",3);let mover=field(&mut g,"JC125",0);
        let mut c=g.make_card("JC125",owner);c.face_down=concealed;c.lock_markers=1;let anchor=c.id.clone();g.regions[1].cards.push(c);
        let a=action(&g,0,"sentinel-move",&mover,Some(&anchor));act(&mut g,0,a);assert_eq!(g.resources(0),0);top(&mut g);
        assert_eq!(g.board(&mover).unwrap().0,1);assert_eq!(g.board(&anchor).unwrap().1.lock_markers,1);restore(&mut g);
    }}
}
#[test]
fn sentinel_move_filters_same_region_unlocked_hidden_mover_and_teammate() {
    let mut g=sentinel_game();fund(&mut g,0,"JC125",3);let mover=field(&mut g,"JC125",0);let anchor=field(&mut g,"JC125",1);g.board_mut(&anchor).unwrap().lock_markers=1;
    assert!(!g.legal_actions(0).iter().any(|a|a.action.ability_id.as_deref()==Some("sentinel-move")));
    g.move_on_board_instance(&anchor,1);let a=action(&g,0,"sentinel-move",&mover,Some(&anchor));
    g.board_mut(&anchor).unwrap().lock_markers=0;reject(&mut g,0,a.clone());g.board_mut(&anchor).unwrap().lock_markers=1;
    g.board_mut(&mover).unwrap().face_down=true;reject(&mut g,0,a.clone());g.board_mut(&mover).unwrap().face_down=false;
    g.board_mut(&mover).unwrap().controller=1;reject(&mut g,0,a);
}
#[test]
fn sentinel_response_invalidates_either_target_without_refund_or_rebind() {
    for scenario in ["unlock","anchor-leaves","mover-leaves","control","hide"] {
        let mut g=sentinel_game();fund(&mut g,0,"JC125",3);let mover=field(&mut g,"JC125",0);let anchor=hidden(&mut g,1,1);g.board_mut(&anchor).unwrap().lock_markers=1;
        let a=action(&g,0,"sentinel-move",&mover,Some(&anchor));act(&mut g,0,a);
        match scenario {"unlock"=>g.board_mut(&anchor).unwrap().lock_markers=0,"anchor-leaves"=>{g.remove_dead(&anchor,RemovalCause::Destroy);},"mover-leaves"=>{g.remove_dead(&mover,RemovalCause::Destroy);},"control"=>g.board_mut(&mover).unwrap().controller=1,_=>g.board_mut(&mover).unwrap().face_down=true}
        top(&mut g);assert!(g.board(&mover).is_none_or(|(r,_)|r==0));assert_eq!(g.resources(0),0);restore(&mut g);
    }
}
#[test]
fn sentinel_response_anchor_movement_follows_current_region_without_false_same_region_move() {
    for destination in [0,2] {
        let mut g=sentinel_game();fund(&mut g,0,"JC125",3);let mover=field(&mut g,"JC125",0);let anchor=hidden(&mut g,1,1);g.board_mut(&anchor).unwrap().lock_markers=1;
        let a=action(&g,0,"sentinel-move",&mover,Some(&anchor));act(&mut g,0,a);g.move_on_board_instance(&anchor,destination);top(&mut g);
        assert_eq!(g.board(&mover).unwrap().0,destination);restore(&mut g);
    }
}
#[test]
fn sentinel_insufficient_assets_exhaustion_and_response_timing() {
    let mut g=sentinel_game();let target=hidden(&mut g,2,1);fund(&mut g,0,"JC125",2);
    assert!(!g.legal_actions(0).iter().any(|a|a.action.ability_id.as_deref()==Some("sentinel-lock")));
    fund(&mut g,0,"JC125",1);let a=action(&g,0,"sentinel-lock",&target,None);act(&mut g,0,a);
    let responder=g.priority_team*2;assert!(g.can_fast(responder));assert!(g.check_timing(responder,&definition("MSJC14").abilities[0]).is_err());
    top(&mut g);fund(&mut g,0,"JC125",3);
    assert!(!g.legal_actions(0).iter().any(|a|a.action.ability_id.as_deref()==Some("sentinel-lock")));
}

#[test]
fn sentinel_lock_response_reveal_invalidates_and_marker_stacks() {
    for reveal in [false,true] {
        let mut g=sentinel_game();fund(&mut g,0,"JC125",3);let target=hidden(&mut g,2,1);g.board_mut(&target).unwrap().lock_markers=1;
        let a=action(&g,0,"sentinel-lock",&target,None);act(&mut g,0,a);
        if reveal { g.board_mut(&target).unwrap().face_down=false; }
        top(&mut g);assert_eq!(g.board(&target).unwrap().1.lock_markers,if reveal {1}else{2});assert_eq!(g.resources(0),0);
    }
}

// Independent review: explicit valid layouts, then ordinary paid commands.
// No product code or corrupted-save boundary is changed by this patch.
#[test]
fn independent_sentinel_attachment_lock_is_inherited_by_anchor_host() {
    // Seat 0 really selects Sentinel. Other seats use a regular legal preset,
    // so the opposing vampire need not satisfy Sentinel's construction rule.
    let mut d=deck::preset("watchers").unwrap(); d.society_id=Some("MSJC14".into());
    d.cards=vec![catalog::DeckEntry{card_id:"JC125".into(),count:50}];
    let mut g=Game::new_with_deck("sentinel-inherited".into(),"LOCAL".into(),"teams".into(),"P0".into(),d,8).unwrap();
    for s in 1..4 { g.join(format!("P{s}"),"watchers".into()).unwrap(); }
    for s in 0..4 { g.apply(s,Action::new("ready")).unwrap(); }
    g.apply(0,Action::new("start")).unwrap(); while g.pending.is_some(){choose(&mut g,vec![]);}
    for p in &mut g.players {p.hand.clear();p.assets.clear();}
    for r in &mut g.regions {r.cards.clear();r.influence=[0;2];}
    g.regions[0].card=g.make_card("DQJC115",0);
    let host=field(&mut g,"JC029",2);
    g.begin_window(Window::Action(1)); fund(&mut g,2,"JC029",1);
    let bat_hand=held(&mut g,"BQ026",2);
    act(&mut g,2,Action{card_id:Some(bat_hand),region:Some(0),..Action::new("deploy")});
    crate::jc029_tests::pass_top(&mut g);
    choose(&mut g,vec![host.clone()]); top(&mut g);
    let bat=g.attachments.iter().find(|a|a.host_id==host).unwrap().card.id.clone();
    assert_eq!(g.board(&bat).unwrap().1.lock_markers,0);
    // A real paid reveal and its target choice place the only lock on the bat.
    g.begin_window(Window::Action(0)); fund(&mut g,0,"JC125",5);
    let dog=field(&mut g,"JC069",0); g.board_mut(&dog).unwrap().face_down=true;
    let mover=g.make_card("JC125",0); let mover_id=mover.id.clone(); g.regions[1].cards.push(mover);
    act(&mut g,0,Action{card_id:Some(dog),..Action::new("reveal")});
    crate::jc029_tests::pass_top(&mut g); choose(&mut g,vec![bat.clone()]); top(&mut g);
    assert_eq!(g.resources(0),3); assert_eq!(g.board(&host).unwrap().1.lock_markers,0);
    assert_eq!(g.board(&bat).unwrap().1.lock_markers,1);
    let a=action(&g,0,"sentinel-move",&mover_id,Some(&bat));
    let mut direct=g.clone(); act(&mut direct,0,a.clone()); top(&mut direct);
    assert_eq!(direct.board(&mover_id).unwrap().0,0); // positive physical-lock control
    let host_action=Action{selected:Some(vec![host.clone()]),..a};
    let listed=g.legal_actions(0).iter().any(|a|a.action==host_action);
    let before=serde_json::to_value(&g).unwrap();
    let result=g.apply(0,host_action.clone());
    if let Ok(dir)=std::env::var("INDEPENDENT_HISTORY_OUT") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/sentinel-inherited-lock.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "before":before,"command":host_action,"listed":listed,"result":format!("{result:?}"),
            "host":host,"attachment":bat,"direct_attachment_control":"passed",
            "after":serde_json::to_value(&g).unwrap()
        })).unwrap()).unwrap();
    }
    assert!(listed && result.is_ok(),"Original manual p11: an attachment's marker is also on its host; listed={listed}, command={result:?}");
    top(&mut g); assert_eq!(g.board(&mover_id).unwrap().0,0);
}

#[test]
fn independent_sentinel_room_lock_is_paid_once_and_keeps_front_private() {
    use crate::room::{RoomEnvelope,RoomCommand,SessionAction};
    let mut g=sentinel_game(); fund(&mut g,0,"JC125",3); let target=hidden(&mut g,2,1);
    let a=action(&g,0,"sentinel-lock",&target,None); let mut r=RoomEnvelope::from_game(g);
    let command=RoomCommand{command_id:"review-sentinel-lock".into(),expected_version:r.revision,action:SessionAction::Game{action:a}};
    let result=r.transition(0,Some(command.clone()),0).unwrap(); assert!(result.error_code.is_none(),"{:?}",result.error_message);
    r=RoomEnvelope::from_persisted(&result.state).unwrap(); assert_eq!(r.game.resources(0),0);
    let duplicate=r.transition(0,Some(command),1).unwrap(); assert_eq!(duplicate.error_code.as_deref(),Some("version_conflict"));
    assert_eq!(duplicate.state,serde_json::to_string(&r).unwrap());
    for n in 0..16 {
        if r.game.stack.is_empty(){break;}
        let w=r.pacing.window.as_ref().unwrap().id.clone();
        let seat=(0..4).find(|s|r.game.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();
        let result=r.transition(seat,Some(RoomCommand{command_id:format!("review-pass-{n}"),expected_version:r.revision,
            action:SessionAction::PassResponse{window_id:w}}),2+n).unwrap();
        assert!(result.error_code.is_none(),"{:?}",result.error_message); r=RoomEnvelope::from_persisted(&result.state).unwrap();
    }
    assert!(r.game.stack.is_empty()); assert_eq!(r.game.board(&target).unwrap().1.lock_markers,1);
    assert_eq!(r.game.resources(0),0);
    for viewer in [0,1,3] {let c=r.game.card_view(r.game.board(&target).unwrap().1,viewer,Some(1),None); assert_eq!(c.lock_markers,Some(1));assert!(c.card_id.is_none());}
}

// Author follow-up to the independent P2: valid explicit attachment layouts.
fn sentinel_attached_bat(g:&mut Game,host:&str)->String {
    let c=g.make_card("BQ026",2);let id=c.id.clone();
    let a=Attachment{card:c,host_id:host.into()};assert!(g.attachment_host_valid(&a));g.attachments.push(a);id
}
fn sentinel_chain()->(Game,String,String,String,String) {
    let mut g=sentinel_game();fund(&mut g,0,"JC125",3);let mover=field(&mut g,"JC125",0);
    let c=g.make_card("JC029",2);let host=c.id.clone();g.regions[1].cards.push(c);
    let child=sentinel_attached_bat(&mut g,&host);let leaf=sentinel_attached_bat(&mut g,&child);
    (g,mover,host,child,leaf)
}
#[test]
fn sentinel_inherited_nested_lock_is_read_only_and_does_not_flow_backwards() {
    let (mut g,mover,host,child,leaf)=sentinel_chain();let sibling=sentinel_attached_bat(&mut g,&host);
    g.board_mut(&leaf).unwrap().lock_markers=1;
    let before=serde_json::to_string(&g).unwrap();
    for id in [&host,&child,&leaf] {assert!(g.sentinel_has_effective_lock(g.board(id).unwrap().1));action(&g,0,"sentinel-move",&mover,Some(id));}
    assert!(!g.sentinel_has_effective_lock(g.board(&sibling).unwrap().1));
    assert_eq!(serde_json::to_string(&g).unwrap(),before);
    let a=action(&g,0,"sentinel-move",&mover,Some(&host));act(&mut g,0,a);top(&mut g);
    assert_eq!(g.board(&mover).unwrap().0,1);
    for id in [&host,&child,&sibling] {assert_eq!(g.board(id).unwrap().1.lock_markers,0);}
    assert_eq!(g.board(&leaf).unwrap().1.lock_markers,1);
}
#[test]
fn sentinel_inherited_lock_rejects_other_markers_reverse_inheritance_and_noncharacters() {
    let (mut g,mover,host,child,leaf)=sentinel_chain();
    g.board_mut(&leaf).unwrap().time_markers=1;
    assert!(!g.sentinel_has_effective_lock(g.board(&host).unwrap().1));
    assert!(!g.legal_actions(0).iter().any(|a|a.action.ability_id.as_deref()==Some("sentinel-move")));
    g.board_mut(&host).unwrap().lock_markers=1;
    assert!(!g.sentinel_has_effective_lock(g.board(&child).unwrap().1));
    assert!(!g.sentinel_has_effective_lock(g.board(&leaf).unwrap().1));
    let valid=action(&g,0,"sentinel-move",&mover,Some(&host));
    for id in [child,leaf,"region:1".into(),"society:p2".into()] {reject(&mut g,0,Action{selected:Some(vec![id]),..valid.clone()});}
    let mut plan=g.make_card("WM046",2);plan.face_down=true;plan.lock_markers=1;let id=plan.id.clone();
    g.attachments.push(Attachment{card:plan,host_id:"society:p2".into()});
    reject(&mut g,0,Action{selected:Some(vec![id]),..valid});
}
#[test]
fn sentinel_inherited_last_lock_destroyed_by_paid_response_cancels_without_refund() {
    let (mut g,mover,host,_child,leaf)=sentinel_chain();g.board_mut(&leaf).unwrap().lock_markers=1;
    fund(&mut g,2,"JC005",2);let destroy=held(&mut g,"JC005",2);
    let a=action(&g,0,"sentinel-move",&mover,Some(&host));act(&mut g,0,a);
    for _ in 0..4 {if g.can_fast(2){break;}pass(&mut g);}
    let response=g.legal_actions(2).into_iter().find(|a|a.action.card_id.as_deref()==Some(&destroy)
        && a.action.target_id.as_deref()==Some(&leaf)).unwrap().action;
    act(&mut g,2,response);top(&mut g);assert!(g.in_play(&leaf).is_none());
    top(&mut g);assert_eq!(g.board(&mover).unwrap().0,0);assert_eq!(g.resources(0),0);
    assert!(g.players[0].society_zone.card.as_ref().unwrap().exhausted);
}
