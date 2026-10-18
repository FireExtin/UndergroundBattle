//! JC041 complete-card behavior and exact persisted-state validation.
use crate::{catalog, jc041, model::*, rules::*, unit_support as u};
use serde_json::{json, Value};
thread_local! { static BAD_EXPORT: std::cell::RefCell<Option<Vec<Value>>> = const { std::cell::RefCell::new(None) }; }
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
    let mut input=g.clone();
    BAD_EXPORT.with(|saved| {if let Some(rows)=saved.borrow().as_ref(){input.room_id=format!("{:024x}",879000+rows.len());}});
    let g=&input;
    let game_error=Game::from_persisted(&serde_json::to_string(g).unwrap()).err().unwrap_or_else(||panic!("Game accepted {label}"));
    let mut clean=g.clone();clean.stack.clear();clean.effects.clear();clean.pending=None;
    let mut r=crate::jc029_tests::envelope(&clean);r.game=g.clone();
    let state=serde_json::to_string(&r).unwrap();
    let room_error=crate::room::RoomEnvelope::from_persisted(&state).err().unwrap_or_else(||panic!("Room accepted {label}"));
    BAD_EXPORT.with(|saved| { if let Some(rows)=saved.borrow_mut().as_mut() {
        rows.push(json!({"name":label,"state":state,"gameError":game_error,"roomError":room_error,"roomId":r.game.room_id,"inviteCode":r.game.invite_code,"revision":r.revision}));
    }});
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
    for n in 0..16 {
        let mut b=g.clone();let s=&mut b.stack[0];let f=s.frame.as_mut().unwrap();
        match n {
            0=>f.actor=1,1=>f.source.card.owner=1,2=>f.source.card.definition="JC029".into(),
            3=>f.source.source_region_instance=Some(String::new()),4=>f.steps[0].context=1,
            5=>f.steps[0].op=Op::Draw{player:PlayerRef::Actor,count:5,end:DeckEnd::Top},6=>f.already_paid.clear(),
            7=>f.already_paid=vec![PaidCost::Assets(vec!["same".into(),"same".into()])],8=>f.guard=GuardState::Accepted,
            9=>s.card=Some(f.source.card.clone()),10=>s.deploy_region=Some(0),11=>s.reveal=true,
            12=>s.id="forged".into(),13=>f.source.card.face_down=false,
            14=>f.already_paid=vec![PaidCost::Assets(vec![])],
            _=>{let PaidCost::Assets(ids)=&mut f.already_paid[0] else{panic!()};ids.truncate(1);},
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
#[test]
fn jc041_missing_program_cannot_turn_its_stack_or_private_continuation_into_legacy_work() {
    let (mut g,_)=started();let mut b=g.clone();b.stack[0].frame=None;bad(&b,"missing-hidden-frame");
    resolve_top(&mut g);let mut b=g.clone();b.stack[0].frame=None;bad(&b,"missing-forced-frame");
    resolve_top(&mut g);let mut b=g.clone();b.pending.as_mut().unwrap().resolution=ChoiceResolution::Discard{redraw:false};bad(&b,"missing-discard-frame");
}
#[test]
fn jc041_forced_discard_survives_real_response_returning_its_revealed_source() {
    let (mut g,_)=started();resolve_top(&mut g);
    let id=g.regions[0].cards.iter().find(|c|c.definition=="JC041").unwrap().id.clone();
    u::fund(&mut g,1,"JC006",3);let spell=u::held(&mut g,"JC006",1);
    let a=g.legal_actions(1).into_iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_deref()==Some(spell.as_str())&&a.action.target_id.as_deref()==Some(id.as_str())).unwrap().action;
    u::act(&mut g,1,a);resolve_top(&mut g);assert!(g.board(&id).is_none());
    u::drain(&mut g);assert_eq!(g.players[0].hand.len(),2);assert!(g.players[1..].iter().all(|p|p.hand.len()==1));
    assert!(g.players.iter().all(|p|p.graveyard.iter().filter(|c|c.definition=="JC084"||c.definition=="JC041").count()==1));
}

struct Trace { name:String, room:crate::room::RoomEnvelope, initial:serde_json::Value, steps:Vec<serde_json::Value> }
impl Trace {
    fn top(&mut self) {
        let id=self.room.game.stack.last().unwrap().id.clone();
        for _ in 0..32 {
            if !self.room.game.stack.iter().any(|s|s.id==id) {return;}
            let w=self.room.pacing.window.clone().expect("response window");
            let seat=*w.members.iter().find(|(_,d)|matches!(d,crate::room::Decision::Undecided{..})).unwrap().0;
            self.command(seat,crate::room::SessionAction::PassResponse{window_id:w.id},true);
        }
        panic!("bounded original response frame");
    }
    fn drain(&mut self) {
        for _ in 0..80 {
            if let Some(p)=&self.room.game.pending {
                assert_eq!(p.choice.kind,"discard");
                let ids=p.choice.options.iter().take(p.choice.min.unwrap_or(0)).map(|o|o.id.clone()).collect();
                self.choose(ids);
            } else if self.room.game.stack.is_empty() {return;} else {self.responses();}
        }
        panic!("bounded JC041 Room continuation");
    }
    fn new(name:&str,g:&Game)->Self {
        let room=crate::jc029_tests::envelope(g);
        let initial=serde_json::json!({"state":serde_json::to_string(&room).unwrap(),"views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>()});
        Self{name:name.into(),room,initial,steps:vec![]}
    }
    fn command(&mut self,seat:usize,action:crate::room::SessionAction,accepted:bool) {
        use crate::room::*;
        let state=serde_json::to_string(&self.room).unwrap();
        let command=RoomCommand{command_id:format!("{}-{}",self.name,self.steps.len()),expected_version:self.room.revision,action};
        let expected=self.room.transition(seat,Some(command.clone()),0).unwrap();
        assert_eq!(expected.error_code.is_none(),accepted,"{} {:?}",self.name,expected.error_message);
        if expected.changed {assert_eq!(serde_json::to_string(&self.room.replay_events(&expected.journal).unwrap()).unwrap(),expected.state);}
        else {assert_eq!(expected.state,state);assert!(expected.journal.is_empty());}
        self.room=RoomEnvelope::from_persisted(&expected.state).unwrap();
        let views=(0..4).map(|s|self.room.view(s,0)).collect::<Vec<_>>();
        self.steps.push(serde_json::json!({"state":state,"seat":seat,"command":command,"serverNowMs":"0","expected":expected,"views":views}));
    }
    fn game(&mut self,seat:usize,a:Action,accepted:bool){self.command(seat,crate::room::SessionAction::Game{action:a},accepted);}
    fn respond(&mut self,seat:usize,action:Action){
        use crate::room::*;
        let window=loop {
            let w=self.room.pacing.window.clone().expect("response window");
            if matches!(w.members.get(&seat),Some(Decision::Undecided{..})){break w;}
            let s=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
            self.command(s,SessionAction::PassResponse{window_id:w.id},true);
        };
        let intent=format!("{}-response",self.name);
        self.command(seat,SessionAction::BeginResponse{window_id:window.id.clone(),intent_id:intent.clone()},true);
        self.command(seat,SessionAction::SubmitResponse{window_id:window.id,intent_id:intent,action},true);
    }
    fn responses(&mut self) {
        use crate::room::*;
        for _ in 0..32 {let Some(w)=self.room.pacing.window.clone()else{return};
            let seat=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
            self.command(seat,SessionAction::PassResponse{window_id:w.id},true);
        }
        panic!("bounded Room responses");
    }
    fn choose(&mut self,selected:Vec<String>){let p=self.room.game.pending.as_ref().unwrap();let seat=p.seat;
        let a=Action{choice_id:Some(p.choice.id.clone()),selected:Some(selected),..Action::new("choose")};self.game(seat,a,true);}
    fn finish(self)->serde_json::Value{serde_json::json!({"name":self.name,"initial":self.initial,"steps":self.steps})}
}

fn native_game(index:usize, actor:usize, hand_count:usize) -> Game {
    let mut g=u::game();g.room_id=format!("{:024x}",870000+index);water(&mut g,0);hands(&mut g,hand_count);
    g.first_team=g.team(actor);g.begin_window(Window::Action(g.team(actor)));g
}
#[test]
fn jc041_native87_oracles() {
    use crate::room::SessionAction;
    let mut scenarios=vec![];
    for actor in 0..4 {
        let mut g=native_game(actor,actor,2);u::fund(&mut g,actor,"JC125",2);let id=hidden(&mut g,actor);
        let c=g.board_mut(&id).unwrap();c.owner=(actor+1)%4;c.exhausted=actor%2==1;c.lock_markers=2;
        let mut t=Trace::new(&format!("water-actor-{actor}"),&g);
        t.game(actor,action(&id),true);assert!(t.room.game.board(&id).unwrap().1.face_down);
        t.command(0,SessionAction::PauseRoom,true);t.game(actor,action(&id),false);t.command(0,SessionAction::ResumeRoom,true);
        t.responses();assert_eq!(t.room.game.pending.as_ref().unwrap().choice.kind,"discard");
        let p=t.room.game.pending.as_ref().unwrap();let stale=Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),..Action::new("choose")};
        let seat=p.seat;t.game(seat,stale,false);
        t.command(0,SessionAction::PauseRoom,true);t.command(0,SessionAction::ResumeRoom,true);t.drain();
        assert!(t.room.game.players.iter().all(|p|p.hand.len()==1&&p.graveyard.len()==1));scenarios.push(t.finish());
    }
    {
        let mut g=native_game(10,0,2);u::fund(&mut g,0,"JC125",4);let id=u::held(&mut g,"JC041",0);
        let mut t=Trace::new("ordinary-deploy",&g);t.game(0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")},true);t.responses();
        assert!(t.room.game.stack.is_empty()&&t.room.game.pending.is_none());assert!(t.room.game.players.iter().all(|p|p.hand.len()==2));scenarios.push(t.finish());
    }
    {
        let mut g=native_game(11,0,2);g.regions[0].card=g.make_card("DQJC115",0);u::fund(&mut g,0,"JC125",4);let id=hidden(&mut g,0);
        let mut t=Trace::new("ordinary-reveal-land",&g);t.game(0,action(&id),false);t.game(0,Action{card_id:Some(id),..Action::new("reveal")},true);t.responses();t.drain();
        assert!(t.room.game.players.iter().all(|p|p.hand.len()==1));scenarios.push(t.finish());
    }
    {
        let mut g=native_game(12,0,0);u::held(&mut g,"JC084",1);u::held(&mut g,"JC084",3);u::fund(&mut g,0,"JC125",2);let id=hidden(&mut g,0);
        let mut t=Trace::new("empty-hands",&g);t.game(0,action(&id),true);t.responses();assert_eq!(t.room.game.pending.as_ref().unwrap().seat,1);t.drain();
        assert!(t.room.game.players.iter().all(|p|p.hand.is_empty()));assert_eq!(t.room.game.players[1].graveyard.len(),1);assert_eq!(t.room.game.players[3].graveyard.len(),1);scenarios.push(t.finish());
    }
    {
        let mut g=native_game(13,0,2);u::fund(&mut g,0,"JC125",2);let id=hidden(&mut g,0);u::fund(&mut g,1,"JC063",3);let spell=u::held(&mut g,"JC063",1);
        let mut t=Trace::new("hidden-return-response",&g);t.game(0,action(&id),true);
        let a=t.room.game.legal_actions(1).into_iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_deref()==Some(&spell)&&a.action.target_id.as_deref()==Some(&id)).unwrap().action;
        t.respond(1,a);t.responses();assert!(t.room.game.stack.is_empty()&&t.room.game.pending.is_none());
        assert!(t.room.game.players[0].hand.iter().any(|c|c.definition=="JC041"&&c.id!=id));assert!(t.room.game.players.iter().all(|p|!p.graveyard.iter().any(|c|c.definition=="JC084")));scenarios.push(t.finish());
    }
    {
        let mut g=native_game(14,0,2);u::fund(&mut g,0,"JC125",2);let id=hidden(&mut g,0);u::fund(&mut g,1,"JC006",3);let spell=u::held(&mut g,"JC006",1);
        let mut t=Trace::new("revealed-source-return-response",&g);t.game(0,action(&id),true);t.top();
        let source=t.room.game.regions[0].cards.iter().find(|c|c.definition=="JC041"&&!c.face_down).unwrap().id.clone();
        let a=t.room.game.legal_actions(1).into_iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_deref()==Some(&spell)&&a.action.target_id.as_deref()==Some(&source)).unwrap().action;
        t.respond(1,a);t.responses();assert!(t.room.game.board(&source).is_none());t.drain();
        assert_eq!(t.room.game.players[0].hand.len(),2);assert!(t.room.game.players[1..].iter().all(|p|p.hand.len()==1));scenarios.push(t.finish());
    }
    {
        let mut g=native_game(15,0,2);u::fund(&mut g,0,"JC125",4);let id=hidden(&mut g,0);let mut t=Trace::new("repeated-hidden-actions",&g);
        t.game(0,action(&id),true);t.respond(0,action(&id));t.responses();t.drain();
        assert!(t.room.game.players.iter().all(|p|p.hand.len()==1));assert_eq!(u::resources(&t.room.game,0),0);scenarios.push(t.finish());
    }
    BAD_EXPORT.with(|saved| *saved.borrow_mut()=Some(vec![]));
    jc041_forged_paid_stack_effect_bypass_and_duplicate_ids_fail_both_restore_entries();
    jc041_forged_compulsory_discard_continuations_cannot_change_player_count_or_privacy();
    jc041_missing_program_cannot_turn_its_stack_or_private_continuation_into_legacy_work();
    let invalid=BAD_EXPORT.with(|saved|saved.borrow_mut().take().unwrap());assert_eq!(invalid.len(),35);
    let mut source=crate::jc066_tests::source_bindings();
    use sha2::{Digest,Sha256};
    for (name,bytes) in [("rust-game/src/jc041.rs",include_bytes!("jc041.rs").as_slice()),("rust-game/src/jc041_tests.rs",include_bytes!("jc041_tests.rs").as_slice())] {
        source["files"].as_object_mut().unwrap().insert(name.into(),json!(format!("{:x}",Sha256::digest(bytes))));
    }
    assert_eq!(source["files"].as_object().unwrap().len(),137);
    let commands=scenarios.iter().map(|s|s["steps"].as_array().unwrap().len()).sum::<usize>();
    let rejected=scenarios.iter().flat_map(|s|s["steps"].as_array().unwrap()).filter(|r|!r["expected"]["errorCode"].is_null()).count();
    let counts=json!({"scenarios":scenarios.len(),"commands":commands,"rejected":rejected,"projections":4*(commands+scenarios.len()),"invalidRestores":invalid.len()});
    let result=json!({"format":"jc041-native87-v1","sourceBindings":source,"catalog":catalog::catalog(),"scenarios":scenarios,"invalid":invalid,"counts":counts});
    if let Ok(dir)=std::env::var("JC041_NATIVE_DIR") {
        use std::io::Write;std::fs::create_dir_all(&dir).unwrap();
        std::fs::OpenOptions::new().create_new(true).write(true).open(std::path::Path::new(&dir).join("Native87.json")).unwrap().write_all(&serde_json::to_vec(&result).unwrap()).unwrap();
    }
    println!("JC041_NATIVE87_COUNTS {}",counts);
}

#[test]
fn jc041_p2_source_region_uid_probe() {
    use crate::room::{RoomCommand,RoomEnvelope,SessionAction};
    use sha2::{Digest,Sha256};
    let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../rust-game-wasm/tests/jc041P2.original-room.json");
    let original_state=std::fs::read_to_string(path).unwrap();
    assert_eq!(format!("{:x}",Sha256::digest(original_state.as_bytes())),"ad127d41b99f0bcae5cb8fe8c1488775fad1ef1c0407140fc30d6ccd2f83850c");
    let original=RoomEnvelope::from_persisted(&original_state).unwrap();
    assert_eq!((original.revision,original.game.version),(9,9));
    let source=&original.game.stack[0].frame.as_ref().unwrap().source;
    assert_eq!((original.game.stack[0].id.as_str(),original.game.stack[0].frame.as_ref().unwrap().frame_id.as_str()),("i252","i252"));
    assert_eq!(source.card.id,"i251");assert_eq!(source.card.definition,"JC041");
    assert_eq!((source.card.owner,source.card.controller,source.card.face_down),(1,0,true));
    assert_eq!(source.region,Some(0));assert_eq!(source.source_region_instance.as_deref(),Some("i240"));
    assert_eq!(original.game.regions[0].card.id,"i240");assert_eq!(original.game.regions[0].card.definition,"DQJC107");
    let hand=original.game.players.iter().flat_map(|p|&p.hand).find(|c|catalog::card(&c.definition).kind!="region").unwrap().id.clone();
    let asset=original.game.players[0].assets[0].id.clone();
    let expected_accepted=std::env::var("JC041_P2_EXPECT_ACCEPTED").is_ok_and(|v|v=="1");
    let mut cases=vec![];
    let other_region=original.game.regions[1].card.id.clone();
    for (name,uid) in [("source-self","i251".into()),("known-hand-nonregion",hand),("known-paid-asset-nonregion",asset),("known-region-wrong-slot",other_region)] {
        let mut changed=original.clone();changed.game.stack[0].frame.as_mut().unwrap().source.source_region_instance=Some(uid.clone());
        let state=serde_json::to_string(&changed).unwrap();
        let mut undone=changed.clone();undone.game.stack[0].frame.as_mut().unwrap().source.source_region_instance=Some("i240".into());
        assert_eq!(serde_json::to_string(&undone).unwrap(),original_state,"exactly one source_region_instance field changed");
        let game_error=Game::from_persisted(&serde_json::to_string(&changed.game).unwrap()).err();
        let restored=RoomEnvelope::from_persisted(&state);let room_error=restored.as_ref().err().cloned();
        let command=RoomCommand{command_id:format!("jc041-p2-{name}"),expected_version:9,action:SessionAction::PauseRoom};
        let (views,pause,poll)=if let Ok(r)=&restored {(
            Some((0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()),
            Some(r.transition(0,Some(command.clone()),0).unwrap()),Some(r.transition(0,None,0).unwrap())
        )}else{(None,None,None)};
        cases.push(json!({"name":name,"state":state,"regionUid":uid,"gameError":game_error,"roomError":room_error,"views":views,"command":command,"expectedPause":pause,"expectedPoll":poll}));
        assert_eq!(game_error.is_none(),expected_accepted,"Game {name}: {game_error:?}");
        assert_eq!(room_error.is_none(),expected_accepted,"Room {name}: {room_error:?}");
    }
    let mut legal=vec![];
    for kind in ["original","source-moved","original-region-retired","vacant-original-slot"] {
        let mut r=original.clone();
        if kind!="original" {
            // Explicit Native layout boundary, followed only by real Room commands.
            // No claim that the layout edit is a playable response action.
            let (_,card)=r.game.remove_board("i251").unwrap();r.game.regions[1].cards.push(card);
            if kind=="original-region-retired" {
                let index=r.game.world.iter().position(|c|!catalog::card(&c.definition).subtypes.iter().any(|s|s=="水域")).unwrap();
                let replacement=r.game.world.remove(index);let old=std::mem::replace(&mut r.game.regions[0].card,replacement);
                let retired=r.game.fresh(old);r.game.world.push(retired);
                assert!(!r.game.regions.iter().any(|p|p.card.id=="i240")&&!r.game.world.iter().any(|c|c.id=="i240"));
            }
            if kind=="vacant-original-slot" {r.game.world.clear();r.game.regions[0].vacant=true;}
        }
        let state=serde_json::to_string(&r).unwrap();let r=RoomEnvelope::from_persisted(&state).unwrap();
        let initial=json!({"state":state,"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()});
        let mut t=Trace{name:format!("p2-{kind}"),room:r,initial,steps:vec![]};
        t.responses();assert!(t.room.game.pending.is_some());
        t.command(0,SessionAction::PauseRoom,true);t.command(2,SessionAction::ResumeRoom,true);t.drain();
        assert!(t.room.game.players.iter().all(|p|p.hand.len()==1));
        assert_eq!(t.room.game.regions.iter().flat_map(|r|&r.cards).filter(|c|c.definition=="JC041"&&!c.face_down).count(),1);
        legal.push(t.finish());
    }
    let mut bindings=crate::jc066_tests::source_bindings();
    for (name,bytes) in [("rust-game/src/jc041.rs",include_bytes!("jc041.rs").as_slice()),("rust-game/src/jc041_tests.rs",include_bytes!("jc041_tests.rs").as_slice())] {
        bindings["files"].as_object_mut().unwrap().insert(name.into(),json!(format!("{:x}",Sha256::digest(bytes))));
    }
    let data=json!({"format":"jc041-p2-source-region-v1","sourceBindings":bindings,"expectedAccepted":expected_accepted,"originalState":original_state,"originalViews":(0..4).map(|s|original.view(s,0)).collect::<Vec<_>>(),"cases":cases,"legal":legal});
    if let Ok(dir)=std::env::var("JC041_P2_DIR") {
        use std::io::Write;std::fs::create_dir_all(&dir).unwrap();
        std::fs::OpenOptions::new().create_new(true).write(true).open(std::path::Path::new(&dir).join("p2.json")).unwrap().write_all(&serde_json::to_vec(&data).unwrap()).unwrap();
    }
    println!("JC041_P2_REGION {}",json!({"accepted":expected_accepted,"cases":4,"legalScenarios":4}));
}
