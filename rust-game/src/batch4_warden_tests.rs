//! Marker-producing Shoggoth is absent. Set up its documented 0..3 marker
//! state explicitly, then use real payments/response commands for removal.
use crate::{catalog, model::*, rules::*, unit_support::*};
use crate::batch4_warden::{ENTER, LEAVE};
fn marked() -> Game {
    let mut g = game();
    for p in &mut g.players { p.society_zone.loss_control = 3; }
    restore(&mut g);
    g
}
fn wait(g: &mut Game, key: &str) {
    for _ in 0..96 {
        if g.pending.as_ref().is_some_and(|p| matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.ability.key == key)) {
            restore(g); return;
        }
        if g.pending.is_some() { resolve_choice(g); }
        else if !g.effects.is_empty() && g.stack.is_empty() { g.drive().unwrap(); restore(g); }
        else { pass(g); }
    }
    panic!("bounded warden trigger {key}");
}
fn deploy(g: &mut Game, seat: usize, region: usize) -> String {
    g.begin_window(Window::Action(g.team(seat)));
    fund(g, seat, "BQ053", 1);
    let id = held(g, "BQ053", seat);
    act(g, seat, Action { card_id: Some(id), region: Some(region), ..Action::new("deploy") });
    wait(g, ENTER);
    let ChoiceResolution::Declare { declaration: d, .. } = &g.pending.as_ref().unwrap().resolution else { panic!() };
    d.source.card.id.clone()
}
fn accept(g: &mut Game) { choose(g, vec!["accept".into()]); top(g); }
fn kill(g: &mut Game, seat: usize, target: &str) {
    g.begin_window(Window::Action(g.team(seat)));
    fund(g, seat, "JC104", 2);
    let spell = held(g, "JC102", seat);
    act(g, seat, Action { card_id: Some(spell), target_id: Some(target.into()), ..Action::new("play") });
    wait(g, LEAVE);
}
fn hide(g: &mut Game, target: &str) {
    g.begin_window(Window::Action(0));
    fund(g, 0, "JC063", 2);
    let spell = held(g, "JC063", 0);
    act(g, 0, Action { card_id: Some(spell), option: Some("hide".into()), target_id: Some(target.into()), ..Action::new("play") });
    wait(g, LEAVE);
}
fn bad(g: &Game) { assert!(Game::from_persisted(&serde_json::to_string(g).unwrap()).is_err()); }

#[test]
fn batch4_warden_original_print_public_guard_and_native_only_catalog() {
    let c = catalog::card("BQ053");
    assert_eq!((c.cost, c.defense, c.supported), (1, Some(1), false));
    assert_eq!(c.permanent_icons.influence, 1);
    let d = crate::rules::definition("BQ053");
    assert!(d.traits.public); assert_eq!(d.traits.guard, 1);
    assert_eq!(d.abilities.iter().map(|a| a.event.unwrap()).collect::<Vec<_>>(), vec![Event::Enter, Event::Leave]);
}

#[test]
fn batch4_warden_real_paid_entry_only_removes_personal_separate_marker() {
    let mut g = marked();
    // Two previously won real world cards make the separate destruction count legal.
    for _ in 0..2 { let c=g.world.pop().unwrap(); g.players[0].score_cards.push(c); }
    g.players[0].society_zone.exposure = 2; g.players[0].society_zone.secrecy = 0; g.players[0].destruction_markers = 2;
    let id = deploy(&mut g, 0, 0);
    assert_eq!(g.resources(0), 0); assert_eq!(g.players[0].society_zone.loss_control, 3);
    accept(&mut g);
    assert_eq!(g.players.iter().map(|p| p.society_zone.loss_control).collect::<Vec<_>>(), vec![2,3,3,3]);
    assert_eq!((g.players[0].society_zone.exposure, g.players[0].society_zone.secrecy, g.players[0].destruction_markers), (2,0,2));
    assert_eq!(g.current_guard(g.board(&id).unwrap().1), 1); restore(&mut g);
}

#[test]
fn batch4_warden_entry_can_decline_without_refunding_deployment() {
    let mut g = marked(); let id = deploy(&mut g, 0, 0);
    choose(&mut g, vec![]);
    assert!(g.board(&id).is_some()); assert_eq!(g.resources(0),0); assert_eq!(g.players[0].society_zone.loss_control,3); restore(&mut g);
}

#[test]
fn batch4_warden_actual_lethal_damage_retains_original_departure_and_no_second_death_trigger() {
    let mut g = marked(); let id = field(&mut g, "BQ053", 0);
    kill(&mut g, 2, &id);
    assert!(g.board(&id).is_none());
    let ChoiceResolution::Declare { declaration: d, .. } = &g.pending.as_ref().unwrap().resolution else { panic!() };
    assert_eq!(d.source.card.id,id); assert_eq!(d.source.card.controller,0); assert_eq!(d.ability.event,Some(Event::Leave));
    accept(&mut g); drain(&mut g);
    assert_eq!(g.players[0].society_zone.loss_control,2); assert!(g.pending.is_none()); restore(&mut g);
}

#[test]
fn batch4_warden_actual_hide_removes_once_and_four_seats_hide_front_but_show_society_marker() {
    let mut g = marked(); let id = field(&mut g,"BQ053",0);
    hide(&mut g,&id); accept(&mut g);
    let hidden = g.regions[0].cards.iter().find(|c|c.definition=="BQ053").unwrap();
    assert!(hidden.face_down); assert_ne!(hidden.id,id); assert_eq!(g.current_guard(hidden),0);
    for s in 0..4 {
        let view=serde_json::to_value(g.view(s)).unwrap();
        assert_eq!(view["societyZones"][0]["lossControl"],2);
        let card=&g.view(s).regions[0].characters.iter().find(|c|c.instance_id==hidden.id).unwrap().clone();
        if s!=0 { assert_ne!(card.card_id.as_deref(),Some("BQ053")); }
    }
    restore(&mut g);
}

#[test]
fn batch4_warden_public_role_can_reveal_after_effect_hide_and_entry_is_new_instance() {
    let mut g=marked(); let id=field(&mut g,"BQ053",0); hide(&mut g,&id); accept(&mut g);
    let hidden=g.regions[0].cards.iter().find(|c|c.definition=="BQ053").unwrap().id.clone();
    g.begin_window(Window::Action(0)); fund(&mut g,0,"BQ053",1);
    act(&mut g,0,Action{card_id:Some(hidden.clone()),..Action::new("reveal")}); wait(&mut g,ENTER); accept(&mut g);
    let live=g.regions[0].cards.iter().find(|c|c.definition=="BQ053").unwrap();
    assert!(!live.face_down); assert_ne!(live.id,hidden); assert_eq!(g.players[0].society_zone.loss_control,1); restore(&mut g);
}

#[test]
fn batch4_warden_hidden_return_is_not_second_face_up_departure() {
    let mut g=marked(); let id=field(&mut g,"BQ053",0); hide(&mut g,&id); accept(&mut g);
    let hidden=g.regions[0].cards.iter().find(|c|c.definition=="BQ053").unwrap().id.clone();
    g.begin_window(Window::Action(0)); fund(&mut g,0,"JC063",2); let spell=held(&mut g,"JC063",0);
    act(&mut g,0,Action{card_id:Some(spell),option:Some("return".into()),target_id:Some(hidden),..Action::new("play")}); top(&mut g);
    assert_eq!(g.players[0].society_zone.loss_control,2); assert!(g.pending.is_none()); restore(&mut g);
}

#[test]
fn batch4_warden_movement_primitive_preserves_identity_and_does_not_emit_entry_or_departure() {
    let mut g=marked(); let id=field(&mut g,"BQ053",0);
    g.move_on_board_instance(&id,1); g.drive().unwrap();
    assert_eq!(g.board(&id).unwrap().0,1); assert!(g.pending.is_none()); assert!(g.effects.is_empty());
    assert_eq!(g.players[0].society_zone.loss_control,3); restore(&mut g);
}

#[test]
fn batch4_warden_original_controller_not_owner_or_teammate_receives_departure_effect() {
    let mut g=marked(); let id=field(&mut g,"BQ053",0); g.board_mut(&id).unwrap().controller=2;
    kill(&mut g,0,&id);
    assert_eq!(g.pending.as_ref().unwrap().seat,2);
    accept(&mut g);
    assert_eq!(g.players.iter().map(|p|p.society_zone.loss_control).collect::<Vec<_>>(),vec![3,3,2,3]); restore(&mut g);
}

#[test]
fn batch4_warden_zero_is_noop_and_multiple_real_entries_each_consume_one() {
    let mut g=marked(); g.players[0].society_zone.loss_control=1;
    deploy(&mut g,0,0); accept(&mut g); deploy(&mut g,0,0); accept(&mut g);
    assert_eq!(g.players[0].society_zone.loss_control,0); restore(&mut g);
}

#[test]
fn batch4_warden_marker_is_read_at_resolution_and_turn_cleanup_does_not_erase_it() {
    let mut g=marked(); g.players[0].society_zone.loss_control=0; deploy(&mut g,0,0);
    choose(&mut g,vec!["accept".into()]); g.players[0].society_zone.loss_control=2; restore(&mut g); top(&mut g);
    assert_eq!(g.players[0].society_zone.loss_control,1);
    g.effects.push_back(Effect::FinishCleanup); g.drive().unwrap(); assert_eq!(g.players[0].society_zone.loss_control,1); restore(&mut g);
}

#[test]
fn batch4_warden_choices_frames_and_duplicate_active_receipts_are_closed() {
    let mut g=marked(); deploy(&mut g,0,0);
    let p=g.pending.clone().unwrap(); let a=Action{choice_id:Some(p.choice.id),selected:Some(vec!["accept".into()]),..Action::new("choose")};
    reject(&mut g,1,a.clone()); let mut extra=a.clone(); extra.target_id=Some("society:p1".into()); reject(&mut g,0,extra);
    let mut altered=g.clone(); if let ChoiceResolution::Declare{declaration:d,..}=&mut altered.pending.as_mut().unwrap().resolution {d.ability.ops=vec![];} bad(&altered);
    let mut duplicate=g.clone(); if let ChoiceResolution::Declare{declaration:d,..}=&g.pending.as_ref().unwrap().resolution {duplicate.effects.push_back(Effect::Declare{declaration:d.clone()});} bad(&duplicate);
    act(&mut g,0,a.clone()); reject(&mut g,0,a);
    let mut altered=g.clone(); altered.stack.last_mut().unwrap().frame.as_mut().unwrap().actor=1; bad(&altered);
    let mut duplicate=g.clone(); duplicate.stack.push(duplicate.stack.last().unwrap().clone()); bad(&duplicate);
    top(&mut g); assert_eq!(g.players[0].society_zone.loss_control,2); restore(&mut g);
}

#[test]
fn batch4_warden_marker_upper_bound_and_large_u64_roundtrip_are_exact() {
    let mut g=marked(); g.seed=9_007_199_254_740_993; g.random=g.seed; restore(&mut g);
    let mut invalid=g.clone(); invalid.players[0].society_zone.loss_control=4; bad(&invalid);
    deploy(&mut g,0,0); accept(&mut g);
    assert_eq!(g.seed,9_007_199_254_740_993); assert_eq!(g.random,9_007_199_254_740_993); restore(&mut g);
}
