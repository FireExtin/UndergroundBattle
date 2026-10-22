//! Prerequisite counter contract, not XQ20 command/admission acceptance.
use crate::{batch4_markers::MarkerKind, model::*, unit_support::*};

fn attached(g: &mut Game, host: &str, owner: usize) -> String {
    let c = g.make_card("BQ026", owner); let id = c.id.clone();
    g.attachments.push(Attachment { card: c, host_id: host.into() }); id
}
fn transfer(g: &mut Game, from: &str, to: &str) {
    let p = g.prepare_marker_move(from, to).unwrap();
    g.commit_marker_move_counters(p).unwrap();
}

#[test]
fn marker_transfer_zero_source_distinct_entities_and_exact_controller() {
    let mut g = game(); let a = field(&mut g, "JC125", 0); let b = field(&mut g, "JC125", 0);
    let teammate = field(&mut g, "JC125", 1); let enemy_a = field(&mut g, "JC125", 2); let enemy_b = field(&mut g, "JC125", 2);
    assert!(g.prepare_marker_move(&a, &a).is_err());
    assert!(g.prepare_marker_move(&a, &teammate).is_err());
    assert!(g.prepare_marker_move(&a, &enemy_a).is_err());
    assert!(g.prepare_marker_move(&enemy_a, &enemy_b).is_ok());
    let before = serde_json::to_string(&g).unwrap(); transfer(&mut g, &a, &b);
    assert_eq!(serde_json::to_string(&g).unwrap(), before); restore(&mut g);
}
#[test]
fn marker_transfer_excludes_hidden_characters_assets_regions_and_private_cards() {
    let mut g = game(); let a = field(&mut g, "JC125", 0); let hidden = field(&mut g, "JC125", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    let private = held(&mut g, "JC125", 0); fund(&mut g, 0, "JC125", 1);
    for id in [hidden, private, g.players[0].assets[0].id.clone(), "region:0".into(), "society:p0".into()] {
        assert!(g.prepare_marker_move(&a, &id).is_err()); assert!(g.prepare_marker_move(&id, &a).is_err());
    }
}
#[test]
fn marker_transfer_secret_plan_is_an_attachment_not_a_hidden_character() {
    let mut g = game(); let a = field(&mut g, "JC125", 0); let mut p = g.make_card("WM046", 0); p.face_down = true;
    let id = p.id.clone(); g.attachments.push(Attachment {card:p, host_id:"society:p0".into()});
    transfer(&mut g, &a, &id); restore(&mut g);
    for s in 1..4 { let v = g.view(s); let p = &v.attachments.iter().find(|a|a.card.instance_id == id).unwrap().card; assert!(p.card_id.is_none()); assert_eq!(p.name,"秘密计划"); }
}
#[test]
fn marker_transfer_seven_field_synthetic_counter_contract_excludes_damage_and_print() {
    // This matrix deliberately tests raw counters, including advancement on a
    // non-plan; it does not claim this synthetic fixture is a legal game save.
    let mut g = game(); let a = field(&mut g,"BQ079",0); let b = field(&mut g,"BQ079",0);
    { let c = g.board_mut(&a).unwrap(); for k in MarkerKind::ALL { *k.field(c) = 2; } c.damage = 3; }
    { let c = g.board_mut(&b).unwrap(); for k in MarkerKind::ALL { *k.field(c) = 1; } c.damage = 4; }
    transfer(&mut g,&a,&b);
    for k in MarkerKind::ALL { assert_eq!(k.count(g.in_play(&a).unwrap().1),0); assert_eq!(k.count(g.in_play(&b).unwrap().1),3); }
    assert_eq!(g.in_play(&a).unwrap().1.damage,3); assert_eq!(g.in_play(&b).unwrap().1.damage,4);
    assert_eq!(g.in_play(&a).unwrap().1.definition,"BQ079"); assert_eq!(g.in_play(&b).unwrap().1.definition,"BQ079");
}
#[test]
fn marker_transfer_inherited_nested_carriers_are_moved_once_without_rehosting() {
    let mut g = game(); let a = field(&mut g,"JC029",0); let b = field(&mut g,"JC029",0);
    let child = attached(&mut g,&a,0); let nested = attached(&mut g,&child,0);
    for (id,n) in [(&a,1),(&child,2),(&nested,3)] { g.board_mut(id).unwrap().time_markers=n; }
    g.board_mut(&b).unwrap().time_markers=4; let hosts=g.attachments.iter().map(|a|a.host_id.clone()).collect::<Vec<_>>();
    transfer(&mut g,&a,&b);
    for id in [&a,&child,&nested] {assert_eq!(g.in_play(id).unwrap().1.time_markers,0);}
    assert_eq!(g.in_play(&b).unwrap().1.time_markers,10);
    assert_eq!(g.attachments.iter().map(|a|a.host_id.clone()).collect::<Vec<_>>(),hosts); restore(&mut g);
}
#[test]
fn marker_transfer_destination_inside_source_subtree_retains_own_counters_once() {
    let mut g = game(); let a = field(&mut g,"JC029",0); let child=attached(&mut g,&a,0); let nested=attached(&mut g,&child,0);
    for (id,n) in [(&a,1),(&child,2),(&nested,3)] {g.board_mut(id).unwrap().time_markers=n;}
    transfer(&mut g,&a,&child);
    assert_eq!(g.in_play(&a).unwrap().1.time_markers,0); assert_eq!(g.in_play(&child).unwrap().1.time_markers,6); assert_eq!(g.in_play(&nested).unwrap().1.time_markers,0);
    assert_eq!(g.effective_time_markers(g.in_play(&a).unwrap().1),6); restore(&mut g);
}
#[test]
fn marker_transfer_destination_host_keeps_existing_other_attachment_markers() {
    let mut g=game(); let a=field(&mut g,"JC029",0); let b=field(&mut g,"JC029",0); let child=attached(&mut g,&b,0);
    g.board_mut(&a).unwrap().time_markers=2; g.board_mut(&b).unwrap().time_markers=1; g.board_mut(&child).unwrap().time_markers=4;
    transfer(&mut g,&a,&b); assert_eq!(g.in_play(&b).unwrap().1.time_markers,3); assert_eq!(g.in_play(&child).unwrap().1.time_markers,4);
    assert_eq!(g.effective_time_markers(g.in_play(&b).unwrap().1),7); restore(&mut g);
}
#[test]
fn marker_transfer_addition_overflow_is_rejected_before_any_write() {
    for nested in [false,true] {let mut g=game(); let a=field(&mut g,"JC029",0); let b=field(&mut g,"JC029",0);
        g.board_mut(&a).unwrap().time_markers=u32::MAX;
        if nested {let child=attached(&mut g,&a,0); g.board_mut(&child).unwrap().time_markers=1;} else {g.board_mut(&b).unwrap().time_markers=1;}
        let before=serde_json::to_string(&g).unwrap(); assert!(g.prepare_marker_move(&a,&b).is_err()); assert_eq!(serde_json::to_string(&g).unwrap(),before);
    }
}
#[test]
fn marker_transfer_stale_snapshot_is_rejected_for_counters_controllers_or_hosts() {
    for mutation in 0..3 {let mut g=game(); let a=field(&mut g,"JC029",0); let b=field(&mut g,"JC029",0); let child=attached(&mut g,&a,0);
        g.board_mut(&child).unwrap().time_markers=2; let p=g.prepare_marker_move(&a,&b).unwrap();
        match mutation {0=>g.board_mut(&a).unwrap().lock_markers=1,1=>g.board_mut(&b).unwrap().controller=1,_=>g.attachments.iter_mut().find(|v|v.card.id==child).unwrap().host_id=b.clone()}
        let before=serde_json::to_string(&g).unwrap(); assert!(g.commit_marker_move_counters(p).is_err()); assert_eq!(serde_json::to_string(&g).unwrap(),before);
    }
}
#[test]
fn marker_transfer_defense_and_wounds_commit_together_before_lethal_settlement() {
    let mut g=game(); let a=field(&mut g,"BQ079",0); let b=field(&mut g,"BQ079",0);
    {let c=g.board_mut(&a).unwrap();c.permanent_combat_markers=1;c.permanent_defense_markers=1;c.wounds=1;}
    restore(&mut g); assert_eq!(g.defense(g.in_play(&a).unwrap().1,0),1);
    transfer(&mut g,&a,&b); g.settle_deaths();
    for id in [&a,&b] {assert!(g.in_play(id).is_some()); assert_eq!(g.defense(g.in_play(id).unwrap().1,0),1);}
    assert_eq!(g.in_play(&a).unwrap().1.wounds,0); assert_eq!(g.in_play(&b).unwrap().1.wounds,1); restore(&mut g);
}
