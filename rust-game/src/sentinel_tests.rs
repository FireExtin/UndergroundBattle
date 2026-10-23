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
    assert!(g.can_fast(2));assert!(g.check_timing(2,&definition("MSJC14").abilities[0]).is_err());
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
