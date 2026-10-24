//! Explicit legal layouts, then real attach, battle, prepare and choice commands.
//! Normal continuation roundtrips are not damaged-save or browser acceptance.
use crate::{catalog, import20_observers as o, import20_contest as c, model::*, rules::*, unit_support::*};
use crate::jc029_tests::pass_top;

fn attach(g: &mut Game, id: &str, seat: usize, target: &str) -> String {
    fund(g, seat, "JC029", 2);
    let source = held(g, id, seat);
    let team = g.team(seat); g.begin_window(Window::Action(team));
    act(g, seat, Action { card_id: Some(source), target_id: Some(target.into()), ..Action::new("play") });
    pass_top(g);
    g.attachments.iter().find(|a| a.card.definition == id && a.card.controller == seat).unwrap().card.id.clone()
}
fn battle(g: &mut Game, region: usize) {
    g.begin_window(Window::Before(region, 1));
    for _ in 0..4 { pass(g); }
}
fn is_eye(g: &Game) -> bool {
    g.pending.as_ref().is_some_and(|p| matches!(&p.resolution,
        ChoiceResolution::Declare { declaration, .. } if declaration.ability.key == o::EYE_WIN))
}
fn to_eye(g: &mut Game, damage_target: Option<&str>) {
    for _ in 0..40 {
        if is_eye(g) { return; }
        if let Some(p) = g.pending.clone() {
            if matches!(p.resolution, ChoiceResolution::Damage { .. }) {
                let target = damage_target.filter(|id| p.choice.options.iter().any(|x| x.id == *id))
                    .unwrap_or(&p.choice.options[0].id).to_owned();
                act(g, p.seat, Action { choice_id: Some(p.choice.id),
                    allocations: Some([(target, p.choice.amount.unwrap())].into()), ..Action::new("choose") });
            } else { resolve_choice(g); }
        } else if !g.stack.is_empty() { pass(g); }
        else { break; }
    }
    assert!(is_eye(g), "expected one optional host-battle declaration");
}
fn next_prepare(g: &mut Game, first_team: usize) {
    // End a legal explicit end-window; cleanup performs the real turn/key flip.
    g.first_team = 1 - first_team; let turn = g.turn; g.begin_window(Window::End);
    for _ in 0..12 { if g.turn > turn { break; } if g.pending.is_some() { resolve_choice(g); } else { pass(g); } }
    assert_eq!(g.turn, turn + 1); assert_eq!(g.first_team, first_team);
    assert_eq!(g.window, Some(Window::Prepare));
}
fn finish_contest(g: &mut Game) {
    for _ in 0..100 {
        if let Some(p) = g.pending.clone() {
            if p.choice.kind == crate::mystic_contest::COMMIT_KIND {
                choose(g, vec![p.choice.options[0].id.clone()]);
            } else if p.choice.kind == crate::mystic_contest::REWARD_KIND { choose(g, vec![]); }
            else { resolve_choice(g); }
        } else if !g.stack.is_empty() { pass(g); }
        else { return; }
    }
    panic!("bounded contest continuation");
}

#[test]
fn import20_observer_data_programs_and_public_gate() {
    assert_eq!(catalog::ENGINE_VERSION, "rust-v0.2.107-import20-sentinel");
    for id in o::CARDS { let d = catalog::card(id); assert!(!d.supported); assert!(catalog::runtime_implemented(d)); }
    let eye = catalog::card("BQ039"); assert_eq!((eye.cost, eye.kind.as_str()), (1,"attachment"));
    assert_eq!(eye.loyalty, ["蓝色","蓝色"]); assert_eq!(eye.magic,"心灵"); assert!(!eye.unique);
    let hall = catalog::card("WM028"); assert_eq!(hall.cost,2); assert_eq!(hall.loyalty,["蓝色"]); assert!(hall.unique);
    validate_definitions(definitions()).unwrap();
}

#[test]
fn import20_eye_paid_attachment_accepts_any_character_controller() {
    for owner in 0..4 {
        let mut g = game(); let host = field(&mut g,"JC084",owner);
        let id = attach(&mut g,"BQ039",0,&host);
        let a = g.attachments.iter().find(|a|a.card.id==id).unwrap();
        assert_eq!(a.host_id,host); assert_eq!(a.card.controller,0); assert_eq!(g.resources(0),1);
    }
}

#[test]
fn import20_eye_illegal_host_loyalty_payload_and_manual_trigger_roll_back() {
    let mut g = game(); let host=field(&mut g,"JC084",0); let source=held(&mut g,"BQ039",0);
    fund(&mut g,0,"JC029",1);
    let a=Action {card_id:Some(source.clone()),target_id:Some(host.clone()),..Action::new("play")};
    reject(&mut g,0,a.clone());fund(&mut g,0,"JC029",1);
    for target in ["society:p0".to_string(),g.regions[0].card.id.clone()] {
        let mut bad=a.clone();bad.target_id=Some(target);reject(&mut g,0,bad);
    }
    let hidden=field(&mut g,"JC125",2);g.board_mut(&hidden).unwrap().face_down=true;
    let mut bad=a.clone();bad.target_id=Some(hidden);reject(&mut g,0,bad);
    let mut bad=a.clone();bad.selected=Some(vec![host]);reject(&mut g,0,bad);
    reject(&mut g,0,Action{card_id:Some(source),ability_id:Some(o::EYE_WIN.into()),..Action::new("activate")});
}

#[test]
fn import20_eye_actual_battle_any_host_side_and_private_enemy_discard() {
    for host_seat in 0..4 {
        let mut g=game();let host=field(&mut g,"JC084",host_seat);
        field(&mut g,"JC084",0);field(&mut g,"JC084",0);
        let enemy_card=held(&mut g,"JC125",2);let other=held(&mut g,"JC125",3);
        let id=attach(&mut g,"BQ039",0,&host);
        battle(&mut g,0);to_eye(&mut g,Some(&host));
        if host_seat>=2 {assert!(g.board(&host).is_none());assert!(!g.attachments.iter().any(|a|a.card.id==id));}
        let p=g.pending.as_ref().unwrap();assert!(p.choice.options.iter().any(|x|x.id=="p2"));
        assert!(!p.choice.options.iter().any(|x|x.id=="p0"||x.id=="p1"));
        choose(&mut g,vec!["p2".into()]);pass_top(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().seat,2);
        for seat in 0..4 {assert_eq!(g.view(seat).pending_choice.is_some(),seat==2);}
        choose(&mut g,vec![enemy_card]);assert!(g.players[2].hand.is_empty());
        assert!(g.players[3].hand.iter().any(|c|c.id==other));
        assert!(g.pending.is_none()&&g.stack.is_empty());
    }
}

#[test]
fn import20_eye_no_trigger_from_exhausted_zero_icon_off_region_host() {
    for mode in ["exhausted","zero","other-region"] {
        let mut g=game();let host=field(&mut g,if mode=="zero"{"JC125"}else{"JC084"},0);
        if mode=="exhausted" {g.board_mut(&host).unwrap().exhausted=true;}
        if mode=="other-region" {let c=g.regions[0].cards.pop().unwrap();g.regions[1].cards.push(c);}
        field(&mut g,"JC084",0);held(&mut g,"JC125",2);attach(&mut g,"BQ039",0,&host);
        battle(&mut g,0);drain(&mut g);
        assert!(!is_eye(&g));assert_eq!(g.players[2].hand.len(),1);
    }
}

#[test]
fn import20_eye_tie_loss_decline_and_wrong_enemy_are_distinct() {
    for outcome in ["tie","loss","decline","target"] {
        let mut g=game();let host=field(&mut g,"JC084",0);field(&mut g,"JC084",2);
        if outcome=="loss" {field(&mut g,"JC084",2);}else if outcome!="tie" {field(&mut g,"JC084",0);}
        let h=held(&mut g,"JC125",2);attach(&mut g,"BQ039",0,&host);battle(&mut g,0);
        if matches!(outcome,"tie"|"loss") {drain(&mut g);assert!(!is_eye(&g));}
        else {
            to_eye(&mut g,None);
            let p=g.pending.as_ref().unwrap().clone();
            reject(&mut g,0,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec!["p1".into()]),..Action::new("choose")});
            reject(&mut g,2,Action{choice_id:Some(p.choice.id),selected:Some(vec!["p2".into()]),..Action::new("choose")});
            if outcome=="decline" {choose(&mut g,vec![]);}
            else {choose(&mut g,vec!["p2".into()]);pass_top(&mut g);choose(&mut g,vec![h.clone()]);}
        }
        assert_eq!(g.players[2].hand.iter().any(|c|c.id==h),outcome!="target");
    }
}

#[test]
fn import20_eye_real_privileged_tie_is_a_battle_win() {
    let mut g=game();let host=field(&mut g,"JC084",0);let target=field(&mut g,"JC084",2);
    let hand=held(&mut g,"JC125",2);attach(&mut g,"BQ039",0,&host);
    g.begin_window(Window::Before(0,1));
    assert_eq!(g.contest_counts(0,1),[1,1]);
    act(&mut g,0,Action::new("privilege"));to_eye(&mut g,Some(&target));
    choose(&mut g,vec!["p2".into()]);pass_top(&mut g);choose(&mut g,vec![hand]);
    assert!(g.players[2].hand.is_empty());
}

#[test]
fn import20_eye_hand_uses_original_source_not_blue_printed_loyalty() {
    let mut g=game();fund(&mut g,0,"JC125",2);let source=held(&mut g,"BQ039",0);let other=held(&mut g,"JC125",0);
    let a=Action{card_id:Some(source.clone()),ability_id:Some(c::EYE_HAND.into()),..Action::new("activate")};
    let mut bad=a.clone();bad.cost_selected=Some(vec![other.clone()]);reject(&mut g,0,bad);reject(&mut g,1,a.clone());
    assert!(g.legal_actions(0).iter().any(|x|x.action==a));act(&mut g,0,a.clone());
    assert_eq!(g.resources(0),0);assert!(g.players[0].hand.iter().any(|x|x.id==other));
    assert_eq!(g.players[0].graveyard.iter().filter(|x|x.definition=="BQ039").count(),1);reject(&mut g,0,a);
    pass_top(&mut g);finish_contest(&mut g);assert_eq!(g.mystic_history.len(),1);
}

#[test]
fn import20_eye_mystic_win_does_not_count_as_host_battle_win() {
    let mut g=game();let host=field(&mut g,"JC084",0);attach(&mut g,"BQ039",0,&host);
    for seat in 0..4 {deck_cards(&mut g,seat,&["JC125","JC125"]);}
    held(&mut g,"BQ064",0);held(&mut g,"JC125",2);
    fund(&mut g,0,"JC125",2);let source=held(&mut g,"BQ039",0);
    act(&mut g,0,Action{card_id:Some(source),ability_id:Some(c::EYE_HAND.into()),..Action::new("activate")});pass_top(&mut g);
    let p=g.pending.as_ref().unwrap();let shown=g.players[p.seat].hand.iter().find(|x|x.definition=="BQ064").unwrap().id.clone();choose(&mut g,vec![shown]);
    finish_contest(&mut g);assert_eq!(g.mystic_history[0].winner_team,Some(0));
    assert_eq!(g.players[2].hand.len(),1);assert!(!is_eye(&g));
}

#[test]
fn import20_hall_paid_own_society_only_and_no_manual_prepare() {
    let mut g=game();fund(&mut g,0,"JC029",2);let source=held(&mut g,"WM028",0);
    for target in ["society:p1".to_string(),field(&mut g,"JC125",0)] {
        reject(&mut g,0,Action{card_id:Some(source.clone()),target_id:Some(target),..Action::new("play")});
    }
    reject(&mut g,0,Action{card_id:Some(source.clone()),ability_id:Some(o::HALL_PREP.into()),..Action::new("activate")});
    act(&mut g,0,Action{card_id:Some(source),target_id:Some("society:p0".into()),..Action::new("play")});pass_top(&mut g);
    assert_eq!(g.resources(0),0);assert!(g.attachments.iter().any(|a|a.card.definition=="WM028"&&a.host_id=="society:p0"));
}

#[test]
fn import20_hall_real_next_prepare_forces_one_contest_only_with_key() {
    for first_team in [0,1] {
        let mut g=game();attach(&mut g,"WM028",0,"society:p0");
        for seat in 0..4 {deck_cards(&mut g,seat,&["JC125","JC125"]);}
        next_prepare(&mut g,first_team);
        if first_team==0 {
            let p=g.pending.as_ref().unwrap().clone();
            assert_eq!(p.choice.kind,"jc035-preparation-order");assert_eq!(p.choice.min,Some(1));
            reject(&mut g,p.seat,Action{choice_id:Some(p.choice.id),selected:Some(vec![]),..Action::new("choose")});
            choose(&mut g,vec![p.choice.options[0].id.clone()]);
            assert_eq!(g.stack.iter().filter(|x|x.frame.as_ref().is_some_and(|f|f.ability_key==o::HALL_PREP)).count(),1);
            finish_contest(&mut g);assert_eq!(g.mystic_history.len(),1);
        } else {assert!(g.stack.is_empty());assert!(g.mystic_history.is_empty()&&g.pending.is_none());}
        assert!(g.stack.is_empty()&&g.mystic_contest.is_none());
    }
}

#[test]
fn import20_hall_key_is_shared_with_teammate_not_opponent() {
    for controller in 0..4 {
        let mut g=game();attach(&mut g,"WM028",controller,&format!("society:p{controller}"));
        next_prepare(&mut g,0);
        if controller<2 {
            let p=g.pending.as_ref().unwrap().clone();assert_eq!(p.seat,controller);
            assert_eq!(p.choice.kind,"jc035-preparation-order");assert_eq!(p.choice.min,Some(1));
            choose(&mut g,vec![p.choice.options[0].id.clone()]);
        }
        assert_eq!(g.stack.iter().any(|x|x.frame.as_ref().is_some_and(|f|f.ability_key==o::HALL_PREP)),controller<2);
        finish_contest(&mut g);
        assert_eq!(g.mystic_history.len(),usize::from(controller<2));
    }
}

#[test]
fn import20_hall_forced_prepare_order_with_existing_society_upkeep() {
    let mut g=game();attach(&mut g,"WM028",0,"society:p0");fund(&mut g,0,"JC063",3);attach(&mut g,"JC068",0,"society:p0");
    fund(&mut g,0,"JC125",1);next_prepare(&mut g,0);
    let p=g.pending.as_ref().unwrap().clone();assert_eq!(p.choice.kind,"jc035-preparation-order");assert_eq!(p.choice.min,Some(1));
    reject(&mut g,p.seat,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),..Action::new("choose")});
    let hall=p.choice.options.iter().find(|x|x.card.as_ref().is_some_and(|c|c.card_id.as_deref()==Some("WM028"))).unwrap().id.clone();
    choose(&mut g,vec![hall]);finish_contest(&mut g);
    assert_eq!(g.mystic_history.len(),1);
}

#[test]
fn import20_hall_empty_hand_deck_ruling_does_not_eliminate_or_replay() {
    let mut g=game();attach(&mut g,"WM028",0,"society:p0");g.players[2].hand.clear();g.players[2].deck.clear();
    next_prepare(&mut g,0);finish_contest(&mut g);
    assert!(g.mystic_history.is_empty()&&g.mystic_contest.is_none());assert!(!g.players[2].eliminated);
    let entry=g.jc035_preparation.as_ref().unwrap().entries.iter().find(|e|e.source.card.definition=="WM028").unwrap();
    assert!(matches!(entry.decision,Some(crate::jc035::PreparationDecision::Resolved{..})));
    restore(&mut g);assert!(g.stack.is_empty()&&g.pending.is_none());
}

#[test]
fn import20_observer_registry_rejects_transplanted_or_retimed_programs() {
    let mut a=definition("BQ039").abilities[1].clone();assert!(validate_ability("JC125",&a).is_err());
    a.event=None;assert!(validate_ability("BQ039",&a).is_err());
    let mut a=definition("WM028").abilities[1].clone();a.timing=Timing::Standard;assert!(validate_ability("WM028",&a).is_err());
    let mut wrong=o::definition("WM028").unwrap();wrong.attachment.as_mut().unwrap().host.relation=Relation::EnemyTeam;
    assert!(o::validate_definition("WM028",&wrong).is_err());
}

#[test]
fn import20_observer_room_paid_attachment_and_hand_receipt_are_once_only() {
    use crate::room::{RoomEnvelope,RoomCommand,SessionAction};
    for id in ["BQ039","WM028"] {
        let mut g=game();fund(&mut g,0,"JC029",2);let source=held(&mut g,id,0);
        let a=if id=="BQ039" {Action{card_id:Some(source),ability_id:Some(c::EYE_HAND.into()),..Action::new("activate")}}
            else {Action{card_id:Some(source),target_id:Some("society:p0".into()),..Action::new("play")}};
        let room=RoomEnvelope::from_game(g);let command=RoomCommand{command_id:format!("import20-next-{id}"),expected_version:room.revision,action:SessionAction::Game{action:a}};
        let result=room.transition(0,Some(command.clone()),0).unwrap();assert_eq!(result.outcome,"accepted");
        let room=RoomEnvelope::from_persisted(&result.state).unwrap();let before=serde_json::to_value(&room).unwrap();
        let duplicate=room.transition(0,Some(command),0).unwrap();assert_eq!(duplicate.outcome,"rejected");assert!(!duplicate.changed&&duplicate.journal.is_empty());
        assert_eq!(serde_json::to_value(RoomEnvelope::from_persisted(&duplicate.state).unwrap()).unwrap(),before);
    }
}
