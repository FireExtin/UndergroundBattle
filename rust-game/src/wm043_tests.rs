//! Explicit legal layouts, then paid commands with normal persistence/view roundtrips.
use crate::{catalog, model::*, rules::*, unit_support::*, wm043 as w};
use crate::jc029_tests::pass_top;

fn spell(g: &mut Game, seat: usize) -> String { fund(g,seat,"JC063",2); held(g,"WM043",seat) }
fn action(id: &str, mode: &str, target: &str) -> Action {
    Action { card_id:Some(id.into()),ability_id:Some(w::PLAY.into()),option:Some(mode.into()),
        target_id:Some(target.into()),..Action::new("play") }
}
fn response_priority(g: &mut Game, seat: usize) {
    for _ in 0..4 { if g.can_fast(seat) { return; } pass(g); }
    assert!(g.can_fast(seat));
}
fn hidden(g: &mut Game, seat: usize) -> String {
    let id=field(g,"JC125",seat);g.board_mut(&id).unwrap().face_down=true;id
}
fn shown_layout(sentinel: bool, shown: [&str;4]) -> Game {
    let mut g=if sentinel {crate::sentinel_tests::sentinel_game()} else {game()};
    for (s,id) in shown.into_iter().enumerate() {
        deck_cards(&mut g,s,&["JC125","JC125"]); held(&mut g,id,s);
    }
    g
}
fn show(g: &mut Game, ids: [&str;4]) {
    fund(g,0,"JC125",2);let source=held(g,"BQ032",0);
    act(g,0,Action{card_id:Some(source),ability_id:Some(crate::batch4_vampire_contest::HAND.into()),..Action::new("activate")});
    pass_top(g);
    for (s,id) in ids.into_iter().enumerate() {
        assert_eq!(g.pending.as_ref().unwrap().seat,s);
        let uid=g.players[s].hand.iter().find(|c|c.definition==id).unwrap().id.clone();choose(g,vec![uid]);
    }
    while g.pending.as_ref().is_some_and(|p|p.choice.kind==crate::mystic_contest::REWARD_KIND) {choose(g,vec![]);}
}
#[test]
fn wm043_source_fields_registry_and_wip_admission() {
    let c=catalog::card("WM043");assert_eq!((&*c.color,&*c.kind,c.cost),("灰","spell",1));
    assert_eq!(c.loyalty,["灰色"]);assert!(!c.supported);assert!(catalog::runtime_implemented(c));
    let d=definition("WM043");assert_eq!(d.abilities.len(),1);assert_eq!(d.abilities[0].modes.len(),2);
    validate_definitions(definitions()).unwrap();
    assert!(d.abilities[0].play_only);assert_eq!(d.abilities[0].timing,Timing::Fast);
}
#[test]
fn wm043_lock_any_controller_character_or_hidden_and_public_marker() {
    for seat in 0..4 { for concealed in [false,true] {
        let mut g=game();let t=if concealed {hidden(&mut g,seat)} else {field(&mut g,"JC125",seat)};
        let s=spell(&mut g,0);let a=action(&s,"lock",&t);
        assert!(g.legal_actions(0).iter().any(|x|x.action==a));let before=g.resources(0);
        act(&mut g,0,a);assert_eq!(g.resources(0),before-1);pass_top(&mut g);
        let c=g.board(&t).unwrap().1;assert_eq!(c.lock_markers,1);
        for viewer in 0..4 {let v=g.card_view(c,viewer,Some(0),None);assert_eq!(v.lock_markers,Some(1));if concealed&&viewer!=seat {assert!(v.card_id.is_none());}}
    }}
}
#[test]
fn wm043_lock_stacks_and_destroy_works_without_sentinel() {
    for concealed in [false,true] {
        let mut g=game();let t=if concealed{hidden(&mut g,2)}else{field(&mut g,"JC125",2)};
        g.board_mut(&t).unwrap().lock_markers=1;let s=spell(&mut g,0);
        act(&mut g,0,action(&s,"lock",&t));pass_top(&mut g);assert_eq!(g.board(&t).unwrap().1.lock_markers,2);
        g.begin_window(Window::Action(0));let s=spell(&mut g,0);act(&mut g,0,action(&s,"destroy",&t));pass_top(&mut g);
        assert!(g.board(&t).is_none());assert!(g.players[2].graveyard.iter().any(|c|c.definition=="JC125"));
    }
}
#[test]
fn wm043_modes_multiple_targets_and_reject_unlocked_or_extra_input_before_payment() {
    let mut g=game();let t=field(&mut g,"JC125",2);let h=hidden(&mut g,1);g.board_mut(&h).unwrap().lock_markers=1;
    let s=spell(&mut g,0);let options=g.legal_actions(0).into_iter().filter(|a|a.action.card_id.as_deref()==Some(&s)&&a.action.kind=="play").collect::<Vec<_>>();
    assert_eq!(options.len(),3);reject(&mut g,0,action(&s,"destroy",&t));
    for mode in ["", "both", "other"] {reject(&mut g,0,action(&s,mode,&h));}
    let mut a=action(&s,"lock",&t);a.selected=Some(vec![t.clone(),h.clone()]);reject(&mut g,0,a);
    for t in [g.regions[0].card.id.clone(),"society:p0".into(),g.players[0].assets[0].id.clone()] {reject(&mut g,0,action(&s,"lock",&t));}
}
#[test]
fn wm043_requires_loyalty_and_cost_before_play() {
    let mut g=game();let t=field(&mut g,"JC125",2);let s=held(&mut g,"WM043",0);
    fund(&mut g,0,"JC125",2);reject(&mut g,0,action(&s,"lock",&t));
    fund(&mut g,0,"JC063",1);for c in &mut g.players[0].assets{c.exhausted=true;}
    reject(&mut g,0,action(&s,"lock",&t));
}
#[test]
fn wm043_fast_response_resolves_before_lower_spell() {
    let mut g=game();let t=field(&mut g,"JC125",2);let lower=spell(&mut g,0);let upper=spell(&mut g,2);
    act(&mut g,0,action(&lower,"lock",&t));response_priority(&mut g,2);
    act(&mut g,2,action(&upper,"lock",&t));pass_top(&mut g);assert_eq!(g.board(&t).unwrap().1.lock_markers,1);
    pass_top(&mut g);assert_eq!(g.board(&t).unwrap().1.lock_markers,2);
}
#[test]
fn wm043_response_reveal_loses_original_target_and_does_not_refund() {
    for mode in ["lock","destroy"] {
        let mut g=game();let t=hidden(&mut g,2);g.board_mut(&t).unwrap().lock_markers=1;
        let s=spell(&mut g,0);act(&mut g,0,action(&s,mode,&t));
        response_priority(&mut g,2);act(&mut g,2,Action{card_id:Some(t.clone()),..Action::new("reveal")});pass_top(&mut g);pass_top(&mut g);
        assert!(g.board(&t).is_none());assert_eq!(g.regions[0].cards.len(),1);assert_eq!(g.regions[0].cards[0].lock_markers,0);
        assert_eq!(g.resources(0),1);assert!(g.players[0].graveyard.iter().any(|c|c.definition=="WM043"));
    }
}
#[test]
fn wm043_inherited_lock_is_legal_and_response_removal_cancels_destroy() {
    for remove in [false,true] {
        let mut g=game();let host=field(&mut g,"JC029",0);let mut bat=g.make_card("BQ026",0);bat.lock_markers=1;
        let bat_id=bat.id.clone();g.attachments.push(Attachment{card:bat,host_id:host.clone()});
        let s=spell(&mut g,0);let response=spell(&mut g,2);let a=action(&s,"destroy",&host);
        assert!(g.legal_actions(0).iter().any(|x|x.action==a));act(&mut g,0,a);
        if remove {response_priority(&mut g,2);act(&mut g,2,action(&response,"destroy",&bat_id));pass_top(&mut g);}
        pass_top(&mut g);assert_eq!(g.board(&host).is_some(),remove);assert_eq!(g.resources(0),1);
    }
}
#[test]
fn wm043_lock_does_not_inherit_from_host_or_sibling() {
    let mut g=game();let host=field(&mut g,"JC029",2);g.board_mut(&host).unwrap().lock_markers=1;
    let bat=g.make_card("BQ026",2);let id=bat.id.clone();g.attachments.push(Attachment{card:bat,host_id:host});
    let s=spell(&mut g,0);reject(&mut g,0,action(&s,"destroy",&id));
    let a=action(&s,"lock",&id);act(&mut g,0,a);pass_top(&mut g);assert_eq!(g.board(&id).unwrap().1.lock_markers,1);
}
#[test]
fn wm043_shown_bonus_is_three_without_sentinel_and_no_post_choice() {
    let ids=["WM043","JC125","JC125","JC125"];let mut g=shown_layout(false,ids);field(&mut g,"JC125",2);
    show(&mut g,ids);assert_eq!(g.mystic_history[0].shown[0].value,3);assert!(g.pending.is_none());assert!(g.mystic_contest.is_none());
}
#[test]
fn wm043_post_lock_on_win_loss_or_tie_targets_any_player() {
    for outcome in ["win","loss","tie"] {for target_owner in 0..4 {
        let ids=match outcome {"loss"=>["WM043","JC125","WM021","WM021"],"tie"=>["WM043","JC125","WM043","JC125"],_=>["WM043","JC125","JC125","JC125"]};
        let mut g=shown_layout(true,ids);let t=hidden(&mut g,target_owner);show(&mut g,ids);
        assert_eq!(g.mystic_history[0].winner_team,match outcome{"win"=>Some(0),"loss"=>Some(1),_=>None});
        let p=g.pending.as_ref().unwrap();assert_eq!((p.seat,p.choice.kind.as_str()),(0,w::CHOICE));
        assert_eq!((p.choice.min,p.choice.max),(Some(0),Some(1)));choose(&mut g,vec![t.clone()]);
        assert_eq!(g.board(&t).unwrap().1.lock_markers,1);
        if outcome=="tie" {assert_eq!(g.pending.as_ref().unwrap().seat,2);choose(&mut g,vec![]);}
        assert!(g.mystic_contest.is_none());
    }}
}
#[test]
fn wm043_post_decline_no_targets_and_duplicate_or_wrong_player_inputs() {
    let ids=["WM043","JC125","JC125","JC125"];
    let mut empty=shown_layout(true,ids);show(&mut empty,ids);assert!(empty.pending.is_none()&&empty.mystic_contest.is_none());
    let mut g=shown_layout(true,ids);let t=field(&mut g,"JC125",2);show(&mut g,ids);
    let id=g.pending.as_ref().unwrap().choice.id.clone();let a=Action{choice_id:Some(id),selected:Some(vec![t.clone()]),..Action::new("choose")};
    reject(&mut g,1,a.clone());let mut twice=a.clone();twice.selected=Some(vec![t.clone(),t.clone()]);reject(&mut g,0,twice);
    choose(&mut g,vec![]);assert_eq!(g.board(&t).unwrap().1.lock_markers,0);reject(&mut g,0,a);
}
#[test]
fn wm043_room_duplicate_paid_command_is_not_reapplied() {
    use crate::room::{RoomCommand,RoomEnvelope,SessionAction};
    let mut g=game();let t=field(&mut g,"JC125",2);let s=spell(&mut g,0);let room=RoomEnvelope::from_game(g);
    let c=RoomCommand{command_id:"wm043-paid-1".into(),expected_version:room.revision,action:SessionAction::Game{action:action(&s,"lock",&t)}};
    let first=room.transition(0,Some(c.clone()),0).unwrap();assert_eq!(first.outcome,"accepted");
    let room=RoomEnvelope::from_persisted(&first.state).unwrap();let saved=serde_json::to_value(&room).unwrap();
    let duplicate=room.transition(0,Some(c),0).unwrap();assert_eq!(duplicate.outcome,"rejected");assert!(!duplicate.changed&&duplicate.journal.is_empty());
    assert_eq!(serde_json::to_value(RoomEnvelope::from_persisted(&duplicate.state).unwrap()).unwrap(),saved);
}
