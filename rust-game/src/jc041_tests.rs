//! JC041 complete-card behavior and exact persisted-state validation.
use crate::{catalog, jc041, model::*, rules::*, unit_support as u};
fn water(g: &mut Game, r: usize) {
    let id = catalog::catalog().cards.iter().find(|c| c.kind=="region" && c.subtypes.iter().any(|s| s=="水域")).unwrap().id.clone();
    g.regions[r].card = g.make_card(&id, 0);
}
fn hidden(g: &mut Game, seat: usize) -> String {
    let id = u::field(g,"JC041",seat);g.board_mut(&id).unwrap().face_down=true;id
}
fn action(id: &str) -> Action {
    Action { card_id:Some(id.into()),ability_id:Some(jc041::HIDDEN.into()),..Action::new("activate") }
}
fn resolve_top(g: &mut Game) {
    let id=g.stack.last().unwrap().id.clone();
    for _ in 0..32 {
        if !g.stack.iter().any(|s|s.id==id) {return;}
        assert!(g.pending.is_none());u::pass(g);
    }
    panic!("JC041 bounded response stack");
}
fn hands(g: &mut Game, n: usize) {
    for seat in 0..4 {for _ in 0..n {u::held(g,"JC084",seat);}}
}
fn started() -> (Game,String) {
    let mut g=u::game();water(&mut g,0);hands(&mut g,2);u::fund(&mut g,0,"JC125",2);
    let id=hidden(&mut g,0);u::act(&mut g,0,action(&id));(g,id)
}
fn bad(g: &Game, label: &str) {
    assert!(Game::from_persisted(&serde_json::to_string(g).unwrap()).is_err(),"Game accepted {label}");
    let mut clean=g.clone();clean.stack.clear();clean.effects.clear();clean.pending=None;
    let mut r=crate::jc029_tests::envelope(&clean);r.game=g.clone();
    assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).is_err(),"Room accepted {label}");
}
#[test]
fn jc041_complete_print_is_closed_and_existing_cards_are_unchanged() {
    let c=catalog::card("JC041");assert_eq!(c.cost,4);assert!(c.loyalty.is_empty());assert_eq!(c.color,"蓝");
    assert_eq!(c.magic,"死亡");assert_eq!(c.defense,Some(1));assert_eq!(c.permanent_icons.combat,1);
    assert_eq!((c.temporary_icons.combat,c.temporary_icons.influence),(1,1));assert!(!c.unique);
    let d=jc041::definition();assert_eq!(d.abilities.len(),2);
    for a in &d.abilities {assert!(crate::rules::validate_ability("JC041",a).is_ok());assert!(crate::rules::validate_ability("JC029",a).is_err());}
    for n in 0..8 {
        let mut d=jc041::definition();match n {
            0=>d.abilities[0].costs=vec![Cost::Assets(1)],1=>d.abilities[0].timing=Timing::Standard,
            2=>d.abilities[0].per_turn_limit=Some(1),3=>d.abilities[0].activation_only=false,
            4=>d.abilities[1].event=Some(Event::Enter),5=>d.abilities[1].ops=vec![Op::Discard{player:PlayerRef::Actor,count:Some(1),redraw:false,optional:false}],
            6=>d.traits.public=true,_=>{d.abilities.remove(0);},
        };assert!(crate::rules::validate_definitions(&std::collections::BTreeMap::from([("JC041".into(),d)])).is_err());
    }
}
#[test]
fn jc041_hidden_action_pays_two_stays_in_play_then_mandatorily_discards_for_all_seats() {
    let (mut g,id)=started();assert!(g.board(&id).unwrap().1.face_down);assert_eq!(u::resources(&g,0),0);
    assert_eq!(g.stack.len(),1);assert!(g.stack[0].card.is_none());assert!(g.pending.is_none());
    resolve_top(&mut g);assert!(g.board(&id).is_none());
    assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().ability_key,jc041::REVEAL);
    assert!(g.pending.is_none(),"compulsory trigger has no optional Accept choice");
    resolve_top(&mut g);
    let mut seen=vec![];
    while let Some(p)=g.pending.clone() {
        assert_eq!(p.choice.kind,"discard");assert_eq!(p.choice.min,Some(1));assert_eq!(p.choice.max,Some(1));assert_eq!(p.choice.allow_decline,Some(false));
        seen.push(p.seat);
        for seat in 0..4 {if seat!=p.seat {
            let v=serde_json::to_value(g.view(seat)).unwrap();
            assert!(v["pendingChoice"]["options"].as_array().is_none_or(|a|a.is_empty()));
        }}
        u::reject(&mut g,p.seat,Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),..Action::new("choose")});
        let other=g.players[(p.seat+1)%4].hand[0].id.clone();
        u::reject(&mut g,p.seat,Action{choice_id:Some(p.choice.id),selected:Some(vec![other]),..Action::new("choose")});
        u::choose(&mut g,vec![p.choice.options[0].id.clone()]);
    }
    assert_eq!(seen,vec![0,1,2,3]);assert!(g.players.iter().all(|p|p.hand.len()==1 && p.graveyard.len()==1));
    assert!(g.stack.is_empty());
}
#[test]
fn jc041_water_and_hidden_and_control_are_activation_conditions_with_atomic_cost_failure() {
    let mut g=u::game();let id=hidden(&mut g,0);u::fund(&mut g,0,"JC125",2);
    u::reject(&mut g,0,action(&id));water(&mut g,0);
    let mut one=g.clone();one.players[0].assets.pop();u::reject(&mut one,0,action(&id));
    u::reject(&mut g,1,action(&id));
    let mut face_up=g.clone();face_up.board_mut(&id).unwrap().face_down=false;u::reject(&mut face_up,0,action(&id));
    for n in 0..5 {let mut a=action(&id);match n {0=>a.region=Some(0),1=>a.target_id=Some(id.clone()),2=>a.option=Some("reveal".into()),3=>a.cost_selected=Some(vec![]),_=>a.ability_id=None};u::reject(&mut g,0,a);}
    assert!(g.legal_actions(0).iter().any(|a|a.action==action(&id)));
    u::act(&mut g,0,action(&id));assert_eq!(u::resources(&g,0),0);
}
#[test]
fn jc041_normal_deploy_has_no_reveal_trigger_and_normal_reveal_still_costs_four_on_land() {
    let mut g=u::game();u::fund(&mut g,0,"JC125",4);let id=u::held(&mut g,"JC041",0);
    u::act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});resolve_top(&mut g);
    assert!(g.stack.is_empty() && g.pending.is_none());assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC041"&&!c.face_down));
    let mut g=u::game();hands(&mut g,1);let id=hidden(&mut g,0);u::fund(&mut g,0,"JC125",3);
    u::reject(&mut g,0,Action{card_id:Some(id.clone()),..Action::new("reveal")});
    u::fund(&mut g,0,"JC125",1);u::act(&mut g,0,Action{card_id:Some(id),..Action::new("reveal")});resolve_top(&mut g);
    assert_eq!(u::resources(&g,0),0);assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().ability_key,jc041::REVEAL);
    u::drain(&mut g);assert!(g.players.iter().all(|p|p.hand.is_empty() && p.graveyard.len()==1));
}
#[test]
fn jc041_real_police_pursuit_response_returns_original_hidden_without_refunding_or_revealing_replacement() {
    let (mut g,id)=started();u::fund(&mut g,1,"JC063",3);let spell=u::held(&mut g,"JC063",1);
    let a=g.legal_actions(1).into_iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_deref()==Some(spell.as_str())&&a.action.target_id.as_deref()==Some(id.as_str())).unwrap().action;
    u::act(&mut g,1,a);resolve_top(&mut g);assert!(g.board(&id).is_none());
    assert!(g.players[0].hand.iter().any(|c|c.definition=="JC041"&&c.id!=id));
    resolve_top(&mut g);assert!(g.stack.is_empty()&&g.pending.is_none());assert_eq!(u::resources(&g,0),0);
    assert!(g.players.iter().all(|p|!p.graveyard.iter().any(|c|c.definition=="JC084")));
}
#[test]
fn jc041_paid_hidden_action_uses_exact_instance_and_only_checks_water_when_activated() {
    let (mut g,id)=started();g.regions[1].card=g.make_card("DQJC115",0);let (_,c)=g.remove_board(&id).unwrap();g.regions[1].cards.push(c);
    assert!(!catalog::card(&g.regions[1].card.definition).subtypes.iter().any(|s|s=="水域"));
    u::restore(&mut g);resolve_top(&mut g);assert!(g.regions[1].cards.iter().any(|c|c.definition=="JC041"&&!c.face_down&&c.id!=id));
    u::drain(&mut g);assert!(g.players.iter().all(|p|p.hand.len()==1));
}
#[test]
fn jc041_repeated_fast_activations_pay_each_time_but_old_identity_cannot_reveal_twice() {
    let (mut g,id)=started();u::fund(&mut g,0,"JC125",2);
    u::pass(&mut g); // enemy passes priority back while the first action waits
    u::act(&mut g,0,action(&id));assert_eq!(u::resources(&g,0),0);assert_eq!(g.stack.len(),2);
    u::drain(&mut g);assert!(g.players.iter().all(|p|p.hand.len()==1&&p.graveyard.len()==1));
    assert_eq!(g.regions.iter().flat_map(|r|&r.cards).filter(|c|c.definition=="JC041").count(),1);
}
#[test]
fn jc041_owner_controller_readiness_and_lock_reset_are_preserved_by_effect_reveal() {
    for actor in 0..4 {
        let mut g=u::game();water(&mut g,0);g.first_team=g.team(actor);g.begin_window(Window::Action(g.team(actor)));
        u::fund(&mut g,actor,"JC125",2);let id=hidden(&mut g,actor);
        let owner=(actor+1)%4;let c=g.board_mut(&id).unwrap();c.owner=owner;c.exhausted=true;c.lock_markers=2;
        u::act(&mut g,actor,action(&id));resolve_top(&mut g);
        let c=g.regions[0].cards.iter().find(|c|c.definition=="JC041").unwrap();
        assert_eq!(c.owner,owner);assert_eq!(c.controller,actor);assert!(c.exhausted);assert_eq!(c.lock_markers,0);assert_ne!(c.id,id);
        u::drain(&mut g);assert!(g.pending.is_none());
    }
}
#[test]
fn jc041_forged_paid_stack_effect_bypass_and_duplicate_ids_fail_both_restore_entries() {
    let (g,_)=started();
    for n in 0..14 {
        let mut b=g.clone();let s=&mut b.stack[0];let f=s.frame.as_mut().unwrap();
        match n {
            0=>f.actor=1,1=>f.source.card.owner=1,2=>f.source.card.definition="JC029".into(),
            3=>f.source.source_region_instance=Some(String::new()),4=>f.steps[0].context=1,
            5=>f.steps[0].op=Op::Draw{player:PlayerRef::Actor,count:5,end:DeckEnd::Top},6=>f.already_paid.clear(),
            7=>f.already_paid=vec![PaidCost::Assets(vec!["same".into(),"same".into()])],8=>f.guard=GuardState::Accepted,
            9=>s.card=Some(f.source.card.clone()),10=>s.deploy_region=Some(0),11=>s.reveal=true,
            12=>s.id="forged".into(),_=>f.source.card.face_down=false,
        };bad(&b,&format!("paid-{n}"));
    }
    let mut b=g.clone();let f=b.stack.pop().unwrap().frame.unwrap();b.effects.push_back(Effect::Frame{frame:Box::new(f)});bad(&b,"direct-effect");
    let mut b=g.clone();b.stack.push(b.stack[0].clone());bad(&b,"duplicate-frame");
}
#[test]
fn jc041_forged_compulsory_discard_continuations_cannot_change_player_count_or_privacy() {
    let (mut g,_)=started();resolve_top(&mut g);resolve_top(&mut g);assert!(g.pending.is_some());
    for n in 0..14 {
        let mut b=g.clone();let p=b.pending.as_mut().unwrap();let ChoiceResolution::Frame{frame:f,choice}=&mut p.resolution else{panic!()};
        match n {
            0=>p.seat=1,1=>p.choice.player_id="p2".into(),2=>p.choice.title.clear(),3=>p.choice.min=Some(0),4=>p.choice.allow_decline=Some(true),
            5=>p.choice.options.clear(),6=>p.choice.options[0].label="forged".into(),7=>f.cursor=0,
            8=>f.steps.pop().map(|_|()).unwrap(),9=>f.steps[0].op=Op::Discard{player:PlayerRef::Context,count:Some(1),redraw:true,optional:false},
            10=>*choice=FrameChoice::Discard{seat:0,redraw:true},11=>f.already_paid=vec![PaidCost::Assets(vec![])],
            12=>f.chosen_region=Some(0),_=>f.source.card.definition="JC046".into(),
        };bad(&b,&format!("discard-{n}"));
    }
}
