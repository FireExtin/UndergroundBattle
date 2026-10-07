//! Source-bound JC030 only. Explicit native fixtures, never public UI play.
use crate::jc029_tests::{apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject};
use crate::{catalog, model::*, rules::*};

fn hand(g: &mut Game, id: &str, owner: usize) -> String {
    let c = g.make_card(id, owner);
    let id = c.id.clone(); g.players[owner].hand.push(c); id
}
fn play(g: &Game, actor: usize, id: &str, target: &str) -> Action {
    g.legal_actions(actor).into_iter().find(|a| a.action.kind == "play" && a.action.card_id.as_deref() == Some(id) && a.action.target_id.as_deref() == Some(target)).unwrap_or_else(|| panic!("real offered spell missing: definition={} resources={} legal={:?}", g.players[actor].hand.iter().find(|c|c.id==id).unwrap().definition,g.resources(actor),g.legal_actions(actor).iter().filter(|a|a.action.card_id.as_deref()==Some(id)).map(|a|&a.action).collect::<Vec<_>>())).action
}
fn investigation(g: &Game, id: &str) -> u32 {
    let (r,c) = g.board(id).unwrap(); g.icons(c,r).investigation
}
fn vampire(g: &Game, id: &str) -> bool {
    g.current_subtypes(g.board(id).unwrap().1).iter().any(|x| x == "吸血鬼")
}

#[test]
fn jc030_whole_original_fields_and_exact_reused_raid_equipment_rule() {
    let d = catalog::card("JC030");
    assert_eq!((&*d.name, &*d.kind, &*d.color, d.cost), ("巨型蝙蝠", "character", "蓝", 3));
    assert_eq!(d.loyalty, ["蓝色", "蓝色"]); assert!(d.magic.is_empty());
    assert_eq!(d.magic_icon, MagicIcon::None); assert_eq!(d.subtypes, ["蝙蝠"]);
    assert_eq!(d.defense, Some(1)); assert!(!d.unique && d.subtitle.is_none());
    assert_eq!(d.permanent_icons, Icons {combat:2,..Default::default()});
    assert_eq!(d.temporary_icons, Icons::default());
    assert_eq!(d.keywords, ["袭击1","不能被装备结附"]);
    assert!(d.rule_traits.cannot_be_equipped);
    let spec = definition("JC030");
    assert_eq!(serde_json::to_value(&spec.abilities).unwrap(), serde_json::to_value(&definition("JC029").abilities).unwrap());
    assert_eq!(spec.modifiers.len(),1);
    assert!(matches!(spec.modifiers[0],StaticModifier::JC030BloodAssetsVampireAndInvestigation));
    assert_eq!(catalog::catalog().cards.len(),103); assert_eq!(catalog::catalog().societies.len(),8);
    for absent in ["XQ11"] { assert!(!catalog::catalog().cards.iter().any(|c| c.id == absent)); }
    assert!(catalog::catalog().societies.iter().any(|c|c.card.id=="MSJC03"));
}

#[test]
fn jc030_zero_one_two_three_domains_are_permanent_and_exhausted_assets_count_all_seats() {
    for actor in 0..4 { for count in 0..=3 {
        let mut g = game(actor); let bat = board(&mut g,"JC030",actor,2);
        fund(&mut g,actor,"JC029",count);
        for c in &mut g.players[actor].assets {c.exhausted = true;}
        g.first_team = 1-g.team(actor);
        assert_eq!(investigation(&g,&bat),u32::from(count>=2));
        assert_eq!(vampire(&g,&bat),count>=2);
        assert_eq!(g.current_subtypes(g.board(&bat).unwrap().1),if count>=2 {vec!["蝙蝠","吸血鬼"]} else {vec!["蝙蝠"]});
        assert_eq!(catalog::card("JC030").subtypes,["蝙蝠"]);
        assert_eq!(catalog::card("JC030").permanent_icons.investigation,0);
        checkpoint(&g);
    }}
}

#[test]
fn jc030_counts_blood_icons_not_blue_color_teammate_or_owner_assets() {
    let mut g=game(0); let bat=board(&mut g,"JC030",2,2);
    g.add_control(&bat,0,ControlLifetime::TurnEnd{turn:g.turn},SubtypeChange::None);
    fund(&mut g,2,"JC029",3);fund(&mut g,1,"JC029",3);
    fund(&mut g,0,"XQ16",3);fund(&mut g,0,"JC003",2);
    assert_eq!(investigation(&g,&bat),0);assert!(!vampire(&g,&bat));checkpoint(&g);
    fund(&mut g,0,"JC029",1);assert!(!vampire(&g,&bat));checkpoint(&g);
    fund(&mut g,0,"XQ14",1);assert!(vampire(&g,&bat));assert_eq!(investigation(&g,&bat),1);checkpoint(&g);
}

#[test]
fn jc030_same_blue_identity_does_not_share_bonus_and_exhausting_bat_keeps_subtype() {
    let mut g=game(0);let bat=board(&mut g,"JC030",0,2);let fledgling=board(&mut g,"JC029",0,2);let lawyers=board(&mut g,"XQ16",0,2);
    fund(&mut g,0,"JC029",2);
    assert!(vampire(&g,&bat));assert_eq!(investigation(&g,&bat),1);
    assert_eq!(investigation(&g,&fledgling),0);assert_eq!(g.current_subtypes(g.board(&fledgling).unwrap().1),["吸血鬼"]);
    assert_eq!(g.current_subtypes(g.board(&lawyers).unwrap().1),["人类","律师"]);
    g.board_mut(&bat).unwrap().exhausted=true;
    assert!(vampire(&g,&bat));assert_eq!(investigation(&g,&bat),0);checkpoint(&g);
}

#[test]
fn jc030_real_control_spell_uses_new_controller_personal_assets_and_restores() {
    let mut g=game(0);let bat=board(&mut g,"JC030",2,2);fund(&mut g,2,"JC029",2);
    fund(&mut g,0,"JC104",5);let control=hand(&mut g,"JC129",0);
    assert!(vampire(&g,&bat));let action=play(&g,0,&control,&bat);apply(&mut g,0,action);pass_top(&mut g);
    assert_eq!(g.board(&bat).unwrap().1.controller,0);assert!(!vampire(&g,&bat));checkpoint(&g);
    fund(&mut g,0,"JC029",2);assert!(vampire(&g,&bat));checkpoint(&g);
    g.players[0].assets.retain(|c|c.definition!="JC029");assert!(!vampire(&g,&bat));
    // Explicit turn-control expiry boundary, not a natural turn simulation.
    g.control_effects.clear();g.settle_controls();
    assert_eq!(g.board(&bat).unwrap().1.controller,2);assert!(vampire(&g,&bat));checkpoint(&g);
}

#[test]
fn jc030_real_asset_action_crosses_threshold_and_destroyed_asset_removes_both_grants() {
    let mut g=game(0);let bat=board(&mut g,"JC030",0,2);fund(&mut g,0,"JC029",1);fund(&mut g,0,"JC104",2);
    let second=hand(&mut g,"JC029",0);assert!(!vampire(&g,&bat));
    apply(&mut g,0,Action{card_id:Some(second),..Action::new("asset")});assert!(vampire(&g,&bat));assert_eq!(investigation(&g,&bat),1);
    let blood=g.players[0].assets.iter().find(|c|c.definition=="JC029").unwrap().id.clone();let destroy=hand(&mut g,"JC107",0);
    let a=play(&g,0,&destroy,&blood);apply(&mut g,0,a);pass_top(&mut g);
    assert!(!vampire(&g,&bat));assert_eq!(investigation(&g,&bat),0);checkpoint(&g);
}

#[test]
fn jc030_equipment_rejected_strictly_but_current_vampire_status_is_allowed() {
    let mut g=game(0);let bat=board(&mut g,"JC030",0,2);fund(&mut g,0,"JC029",4);
    for def in ["BQ022","JC116","XQ47"] {
        let equipment=hand(&mut g,def,0);
        assert!(!g.legal_actions(0).iter().any(|a|a.action.card_id.as_deref()==Some(&equipment)&&a.action.target_id.as_deref()==Some(&bat)));
        reject(&mut g,0,Action{card_id:Some(equipment),target_id:Some(bat.clone()),..Action::new("play")});
    }
    let status=hand(&mut g,"XQ14",0);let a=play(&g,0,&status,&bat);apply(&mut g,0,a);pass_top(&mut g);
    assert!(g.attachments.iter().any(|a|a.host_id==bat && a.card.definition=="XQ14"));assert!(vampire(&g,&bat));assert_eq!(investigation(&g,&bat),1);checkpoint(&g);
}

#[test]
fn jc030_conceal_four_seat_privacy_reveal_new_identity_and_optional_local_raid_restore() {
    for actor in 0..4 {
        let mut g=game(actor);fund(&mut g,actor,"JC029",4);let old=hand(&mut g,"JC030",actor);let target=board(&mut g,"JC059",(actor+2)%4,2);
        apply(&mut g,actor,Action{card_id:Some(old.clone()),region:Some(2),..Action::new("conceal")});
        let hidden=g.regions[2].cards.iter().find(|c|c.definition=="JC030").unwrap().id.clone();assert_ne!(old,hidden);assert!(!vampire(&g,&hidden));
        for viewer in 0..4 {let view=g.view(viewer);let c=view.regions[2].characters.iter().find(|c|c.instance_id==hidden).unwrap();assert_eq!(c.card_id.is_some(),viewer==actor);assert!(c.current_subtypes.is_none());}
        checkpoint(&g);apply(&mut g,actor,Action{card_id:Some(hidden.clone()),..Action::new("reveal")});pass_top(&mut g);
        let live=g.regions[2].cards.iter().find(|c|c.definition=="JC030").unwrap().id.clone();assert_ne!(hidden,live);assert!(vampire(&g,&live));assert_eq!(investigation(&g,&live),1);
        assert_eq!(g.pending.as_ref().unwrap().choice.min,Some(0));
        let state=serde_json::to_string(&envelope(&g)).unwrap();let mut restored=crate::room::RoomEnvelope::from_persisted(&state).unwrap().game;
        if actor%2==0 {choose(&mut g,vec![target.clone()]);choose(&mut restored,vec![target.clone()]);pass_top(&mut g);pass_top(&mut restored);assert_eq!(g.board(&target).unwrap().1.damage,1);}
        else {choose(&mut g,vec![]);choose(&mut restored,vec![]);assert_eq!(g.board(&target).unwrap().1.damage,0);}
        assert_eq!(serde_json::to_value(&g).unwrap(),serde_json::to_value(restored).unwrap());checkpoint(&g);
    }
}

#[test]
fn jc030_normal_deploy_has_no_raid_and_zone_reset_cannot_carry_conditional_state() {
    let mut g=game(0);fund(&mut g,0,"JC029",3);let old=hand(&mut g,"JC030",0);
    apply(&mut g,0,Action{card_id:Some(old.clone()),region:Some(2),..Action::new("deploy")});pass_top(&mut g);assert!(g.pending.is_none());
    let id=g.regions[2].cards.iter().find(|c|c.definition=="JC030").unwrap().id.clone();assert!(vampire(&g,&id));
    let (_, departed)=g.leave_board(&id).unwrap();assert!(!g.jc030_blood_assets_active(&departed));let c=g.reset_zone_card(departed);assert_ne!(c.id,id);assert!(!g.jc030_blood_assets_active(&c));assert_eq!(g.current_subtypes(&c),["蝙蝠"]);g.players[0].hand.push(c);checkpoint(&g);
}

#[test]
fn jc030_export_complete_four_seat_frontend_restoration_fixtures() {
    let mut fixtures=vec![];
    for controller in 0..4 {
        let mut g=game(controller);let owner=(controller+1)%4;let id=board(&mut g,"JC030",owner,2);
        g.add_control(&id,controller,ControlLifetime::TurnEnd{turn:g.turn},SubtypeChange::None);fund(&mut g,controller,"JC029",2);g.board_mut(&id).unwrap().face_down=true;
        let room=envelope(&g);let state=serde_json::to_string(&room).unwrap();let views=(0..4).map(|s|serde_json::to_value(room.view(s,0)).unwrap()).collect::<Vec<_>>();
        fixtures.push(serde_json::json!({"kind":"hidden","controller":controller,"owner":owner,"state":state,"views":views}));checkpoint(&g);
    }
    let mut g=game(0);let id=board(&mut g,"JC030",0,2);fund(&mut g,0,"JC029",2);assert!(vampire(&g,&id));let room=envelope(&g);
    fixtures.push(serde_json::json!({"kind":"positive","state":serde_json::to_string(&room).unwrap(),"views":(0..4).map(|s|serde_json::to_value(room.view(s,0)).unwrap()).collect::<Vec<_>>()}));
    if let Ok(path)=std::env::var("JC030_FRONTEND_FIXTURE_PATH") {std::fs::write(path,serde_json::to_vec(&serde_json::json!({"scope":"explicit native layouts; whole states and four RoomViews; no public gameplay","fixtures":fixtures})).unwrap()).unwrap();}
}
