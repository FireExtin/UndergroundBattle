//! Explicit offline layouts; actual declarations, payment, response and restore.
use crate::{catalog, model::*, rules::*, unit_support as u};
fn layout(actor: usize) -> (Game, String) {
    let mut g = u::game(); let team = g.team(actor); g.begin_window(Window::Action(team));
    let id = u::field(&mut g, "JC044", actor); g.board_mut(&id).unwrap().time_markers = 2;
    (g, id)
}
fn activate(source: &str, carrier: &str, target: usize) -> Action {
    Action { card_id: Some(source.into()), target_id: Some(player_id(target)), cost_selected: Some(vec![carrier.into()]), ..Action::new("activate") }
}
fn response_until_pending(g: &mut Game) {
    for _ in 0..32 { if g.pending.is_some() || g.stack.is_empty() { return; } u::pass(g); }
    panic!("bounded responses");
}
#[test]
fn jc044_complete_print_and_program_cannot_be_split_or_transplanted() {
    let c = catalog::card("JC044");
    assert_eq!((c.name.as_str(), c.cost, c.color.as_str(), c.magic.as_str(), c.defense), ("灾难使者",3,"红","死亡",Some(1)));
    assert_eq!(c.subtypes,["人类","法师","邪教徒"]); assert_eq!(c.loyalty,["红色"]); assert!(!c.unique);
    assert_eq!((c.permanent_icons.investigation,c.permanent_icons.combat,c.temporary_icons.influence),(1,1,1));
    assert_eq!(c.keywords,["时间标志1"]);
    let d = crate::jc044::definition(); assert_eq!(d.abilities.len(),2);
    crate::jc044::validate_definition("JC044", &d).unwrap();
    assert!(crate::jc044::validate_definition("JC125", &d).is_err());
    for change in 0..5 {
        let mut bad = d.clone();
        match change { 0 => bad.abilities.remove(0), 1 => { bad.abilities[1].timing=Timing::Fast;bad.abilities[0].clone() },
            2 => {bad.abilities[1].costs.clear();bad.abilities[0].clone()},
            3 => {bad.abilities[1].targets[0].relation=Relation::EnemyTeam;bad.abilities[0].clone()},
            _ => {bad.abilities[1].costs.push(Cost::ExhaustSource);bad.abilities[0].clone()} };
        assert!(crate::jc044::validate_definition("JC044",&bad).is_err());
    }
}
#[test]
fn jc044_each_actor_can_pay_self_time_and_each_target_player_chooses() {
    for actor in 0..4 { for target in 0..4 {
        let (mut g, source) = layout(actor); let team = g.team(target);
        g.regions[1].influence[team]=2;g.regions[3].influence[team]=1;
        u::act(&mut g, actor, activate(&source,&source,target));
        assert_eq!(g.board(&source).unwrap().1.time_markers,1);assert!(!g.board(&source).unwrap().1.exhausted);
        response_until_pending(&mut g);let p=g.pending.as_ref().unwrap();assert_eq!(p.seat,target);
        assert_eq!(p.choice.min,Some(1));assert_eq!(p.choice.allow_decline,Some(false));
        for seat in 0..4 {assert_eq!(g.view(seat).pending_choice.is_some(),seat==target);}
        u::restore(&mut g);u::choose(&mut g,vec!["region:3".into()]);
        assert_eq!(g.regions[1].influence[team],2);assert_eq!(g.regions[3].influence[team],0);
        assert_eq!(g.board(&source).unwrap().1.time_markers,1);
    }}
}
#[test]
fn jc044_time_is_paid_before_response_and_empty_influence_does_not_refund() {
    let (mut g,source)=layout(0);u::act(&mut g,0,activate(&source,&source,2));
    assert_eq!(g.board(&source).unwrap().1.time_markers,1);response_until_pending(&mut g);
    assert!(g.pending.is_none());assert_eq!(g.board(&source).unwrap().1.time_markers,1);
    u::act(&mut g,0,activate(&source,&source,0));response_until_pending(&mut g);
    assert_eq!(g.board(&source).unwrap().1.time_markers,0);u::reject(&mut g,0,activate(&source,&source,1));
}
#[test]
fn jc044_already_exhausted_source_can_pay_without_an_unprinted_exhaust_cost() {
    let (mut g,source)=layout(0);g.board_mut(&source).unwrap().exhausted=true;
    u::act(&mut g,0,activate(&source,&source,2));
    assert!(g.board(&source).unwrap().1.exhausted);
    assert_eq!(g.board(&source).unwrap().1.time_markers,1);
    response_until_pending(&mut g);u::restore(&mut g);
}
#[test]
fn jc044_fee_carriers_are_physical_and_personally_controlled_not_team_totals() {
    let (mut g,source)=layout(0);g.board_mut(&source).unwrap().time_markers=0;
    let host=u::field(&mut g,"JC029",0);let ally=u::field(&mut g,"JZ30",1);let enemy=u::field(&mut g,"JZ30",2);
    g.board_mut(&ally).unwrap().time_markers=2;g.board_mut(&enemy).unwrap().time_markers=2;
    let mut own=g.make_card("XQ14",0);own.time_markers=2;let own_id=own.id.clone();g.attachments.push(Attachment{card:own,host_id:host.clone()});
    let mut other=g.make_card("XQ14",2);other.time_markers=3;let other_id=other.id.clone();g.attachments.push(Attachment{card:other,host_id:host.clone()});
    u::restore(&mut g);assert_eq!(g.effective_time_markers(g.board(&host).unwrap().1),5);
    for id in [&source,&host,&ally,&enemy,&other_id] {u::reject(&mut g,0,activate(&source,id,2));}
    u::act(&mut g,0,activate(&source,&own_id,2));assert_eq!(g.board(&own_id).unwrap().1.time_markers,1);
    assert_eq!(g.board(&other_id).unwrap().1.time_markers,3);response_until_pending(&mut g);
}
#[test]
fn jc044_bad_fee_shape_or_nonstandard_timing_rejects_atomically() {
    let (mut g,source)=layout(0);
    for n in 0..6 {
        let mut a=activate(&source,&source,2);
        match n {0=>a.cost_selected=None,1=>a.cost_selected=Some(vec![]),2=>a.cost_selected=Some(vec![source.clone(),source.clone()]),
            3=>a.target_id=None,4=>a.region=Some(0),_=>a.option=Some("forged".into())}
        u::reject(&mut g,0,a);
    }
    g.begin_window(Window::After(0,0));u::reject(&mut g,0,activate(&source,&source,2));
}
#[test]
fn jc044_paid_frame_and_repression_pending_reject_forged_metadata() {
    let (mut g,source)=layout(0);g.regions[2].influence[1]=1;u::act(&mut g,0,activate(&source,&source,2));
    for n in 0..6 {
        let mut bad=g.clone();let f=bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
        match n {0=>f.already_paid.clear(),1=>f.already_paid.push(f.already_paid[0].clone()),2=>f.targets[0].id="p3".into(),
            3=>f.steps[0].context=2,4=>{let PaidCost::RemovedTime(p)=&mut f.already_paid[0]else{panic!()};p.remaining+=1;},_=>f.actor=1}
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"frame{n}");
    }
    response_until_pending(&mut g);u::restore(&mut g);
    for n in 0..8 {
        let mut bad=g.clone();let p=bad.pending.as_mut().unwrap();
        match n {0=>p.seat=0,1=>p.choice.player_id="p0".into(),2=>p.choice.min=Some(0),3=>p.choice.options.clear(),
            4=>p.choice.title.clear(),5=>p.choice.description.clear(),6=>p.choice.options[0].id="region:0".into(),_=>p.choice.allow_decline=Some(true)}
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"pending{n}");
    }
    u::choose(&mut g,vec!["region:2".into()]);
}
#[test]
fn jc044_pending_requires_living_actor_and_target_but_response_target_elimination_fizzles_paid_ability() {
    let (mut g,source)=layout(0);g.regions[2].influence[1]=1;
    u::act(&mut g,0,activate(&source,&source,2));response_until_pending(&mut g);
    let room=crate::jc029_tests::envelope(&g);
    for seat in [0,2] {
        let mut bad=room.clone();bad.game.eliminate(seat);
        assert_eq!(bad.game.status,"playing");assert!(bad.game.pending.is_some());
        let game_error=Game::from_persisted(&serde_json::to_string(&bad.game).unwrap()).err().expect("dead pending player");
        let room_error=crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&bad).unwrap()).err().expect("dead pending Room player");
        assert!(game_error.contains("JC044遏制待选"));assert!(room_error.contains("JC044遏制待选"));
    }
    let (mut g,source)=layout(0);g.regions[2].influence[1]=1;
    u::act(&mut g,0,activate(&source,&source,2));g.eliminate(2);
    // A formerly legal frozen target may leave during response. Only the
    // impossible private-choice checkpoint is rejected; payment stays paid.
    u::restore(&mut g);response_until_pending(&mut g);
    assert!(g.pending.is_none());assert!(g.stack.is_empty());
    assert_eq!(g.board(&source).unwrap().1.time_markers,1);assert_eq!(g.regions[2].influence[1],1);
}
#[test]
fn jc044_normal_deploy_and_paid_reveal_have_optional_responsive_entry_time() {
    for reveal in [false,true] {for accept in [false,true] {
        let mut g=u::game();u::fund(&mut g,0,"JC044",3);
        let id=if reveal {let id=u::field(&mut g,"JC044",0);g.board_mut(&id).unwrap().face_down=true;id}else{u::held(&mut g,"JC044",0)};
        u::act(&mut g,0,Action{card_id:Some(id),region:(!reveal).then_some(0),..Action::new(if reveal{"reveal"}else{"deploy"})});
        response_until_pending(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"trigger");
        let source=g.regions[0].cards.iter().find(|c|c.definition=="JC044"&&!c.face_down).unwrap().id.clone();
        assert_eq!(g.board(&source).unwrap().1.time_markers,0);u::choose(&mut g,if accept{vec!["accept".into()]}else{vec![]});
        response_until_pending(&mut g);assert_eq!(g.board(&source).unwrap().1.time_markers,u32::from(accept));
    }}
}

#[test]
fn jc044_paid_ability_survives_source_and_carrier_departure_without_refund_or_new_uid_borrow() {
    let (mut g,source)=layout(0);let carrier=u::field(&mut g,"JC057",0);g.board_mut(&carrier).unwrap().time_markers=2;
    g.regions[2].influence[1]=1;u::act(&mut g,0,activate(&source,&carrier,2));
    assert_eq!(g.board(&carrier).unwrap().1.time_markers,1);
    g.remove_dead(&source,RemovalCause::Destroy);g.remove_dead(&carrier,RemovalCause::Destroy);
    let fresh=u::field(&mut g,"JC044",0);u::restore(&mut g);response_until_pending(&mut g);
    u::choose(&mut g,vec!["region:2".into()]);assert_eq!(g.regions[2].influence[1],0);
    assert_eq!(g.board(&fresh).unwrap().1.time_markers,0);
}
#[test]
fn jc044_paid_time_follows_controller_and_does_not_require_owned_card() {
    let (mut g,source)=layout(0);g.board_mut(&source).unwrap().time_markers=0;
    let borrowed=u::field(&mut g,"JC057",2);g.board_mut(&borrowed).unwrap().controller=0;g.board_mut(&borrowed).unwrap().time_markers=1;
    u::act(&mut g,0,activate(&source,&borrowed,1));assert_eq!(g.board(&borrowed).unwrap().1.time_markers,0);
    response_until_pending(&mut g);g.board_mut(&borrowed).unwrap().time_markers=1;g.board_mut(&borrowed).unwrap().controller=1;
    u::reject(&mut g,0,activate(&source,&borrowed,1));
}
#[test]
fn jc044_declare_container_binds_actor_stage_and_full_accept_metadata() {
    let mut g=u::game();let id=u::held(&mut g,"JC044",0);u::fund(&mut g,0,"JC044",3);
    u::act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});response_until_pending(&mut g);
    for n in 0..10 {
        let mut bad=g.clone();let p=bad.pending.as_mut().unwrap();
        let ChoiceResolution::Declare{stage,..}=&mut p.resolution else{panic!("entry declaration")};
        match n {0=>p.seat=1,1=>{p.seat=1;p.choice.player_id="p1".into()},2=>*stage=DeclareChoice::Target,
            3=>p.choice.description.clear(),4=>p.choice.title.clear(),5=>p.choice.min=None,6=>p.choice.options[0].id="forged".into(),
            7=>p.choice.options[0].label.clear(),8=>p.choice.allow_decline=Some(false),_=>p.choice.id.clear()}
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"declare{n}");
    }
    u::choose(&mut g,vec!["accept".into()]);response_until_pending(&mut g);
}
#[test]
fn jc044_entry_op_cannot_be_hidden_inside_another_program() {
    let mut a=crate::jc044::definition().abilities[0].clone();a.key="fake".into();a.ops=vec![Op::ForEachLivingPlayer(vec![Op::JC044AddOneTimeToOriginalSource])];
    assert!(crate::jc044::validate_ability("JC125",&a).is_err());
    let mut a=crate::jc044::definition().abilities[1].clone();a.key="fake".into();a.costs.clear();
    assert!(crate::jc044::validate_ability("JC044",&a).is_err());
}
#[test]
fn jc044_paid_program_cannot_bypass_response_stack_as_a_direct_effect() {
    let(mut g,source)=layout(0);g.regions[2].influence[1]=1;u::act(&mut g,0,activate(&source,&source,2));
    let f=g.stack.pop().unwrap().frame.unwrap();g.effects.push_back(Effect::Frame{frame:Box::new(f)});
    assert!(Game::from_persisted(&serde_json::to_string(&g).unwrap()).is_err());
}

struct Trace { name:String, room:crate::room::RoomEnvelope, initial:serde_json::Value, steps:Vec<serde_json::Value> }
impl Trace {
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
    fn game(&mut self,seat:usize,mut a:Action,accepted:bool){
        // Legal UI actions carry the explicit key; unit helpers also cover the default key.
        if a.kind=="activate" && a.ability_id.is_none()
            && a.card_id.as_ref().and_then(|id|self.room.game.board(id)).is_some_and(|(_,c)|c.definition=="JC044") {
            a.ability_id=Some(crate::jc044::ACTIVATE.into());
        }
        self.command(seat,crate::room::SessionAction::Game{action:a},accepted);
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
fn invalid_pending(room:&crate::room::RoomEnvelope,phase:&str)->Vec<serde_json::Value>{
    let mut rows=vec![];
    for n in 0..if phase=="declare" {17} else {18} {
        let mut bad=room.clone();let p=bad.game.pending.as_mut().unwrap();
        match n {0=>p.seat=(p.seat+1)%4,1=>p.choice.player_id="p99".into(),2=>p.choice.kind="forged".into(),
            3=>p.choice.title.clear(),4=>p.choice.description.clear(),5=>p.choice.min=None,6=>p.choice.max=Some(2),
            7=>p.choice.allow_decline=Some(!p.choice.allow_decline.unwrap()),8=>p.choice.amount=Some(1),9=>p.choice.id.clear(),
            10=>p.choice.options[0].id="forged".into(),11=>p.choice.options[0].label.clear(),12=>p.choice.options.clear(),
            13=>p.choice.options.push(p.choice.options[0].clone()),
            14=>match &mut p.resolution{ChoiceResolution::Declare{stage,..}=>*stage=DeclareChoice::Target,
                ChoiceResolution::Frame{frame,..}=>frame.guard=GuardState::Unchecked,_=>panic!()},
            15=>match &mut p.resolution{ChoiceResolution::Declare{declaration,..}=>declaration.actor=(declaration.actor+1)%4,
                ChoiceResolution::Frame{frame,..}=>frame.already_paid.clear(),_=>panic!()},
            16=>bad.game.players[p.seat].eliminated=true,
            _=>{let ChoiceResolution::Frame{frame,..}=&p.resolution else {panic!()};bad.game.players[frame.actor].eliminated=true;}}
        let state=serde_json::to_string(&bad).unwrap();
        let game_error=Game::from_persisted(&serde_json::to_string(&bad.game).unwrap()).err().expect("invalid Game restore");
        let room_error=crate::room::RoomEnvelope::from_persisted(&state).err().expect("invalid Room restore");
        rows.push(serde_json::json!({"name":format!("{phase}-{n:02}"),"state":state,"gameError":game_error,"roomError":room_error,
            "roomId":room.game.room_id,"inviteCode":room.game.invite_code,"revision":room.revision}));
    }
    rows
}
fn invalid_entry_stack(room:&crate::room::RoomEnvelope,kind:&str)->serde_json::Value{
    let mut bad=room.clone();
    let item=bad.game.stack.first_mut().expect("entry on stack");
    assert_eq!(item.card.as_ref().unwrap().definition,"JC044");
    let frame=item.frame.as_mut().unwrap();assert_eq!(frame.source.card.definition,"JC044");
    assert_eq!(frame.ability_key,kind);frame.source.card.definition="LC19".into();
    let mut expected=serde_json::to_value(room).unwrap();
    *expected.pointer_mut("/game/stack/0/frame/source/card/definition").unwrap()=serde_json::json!("LC19");
    assert_eq!(serde_json::to_value(&bad).unwrap(),expected,"only frozen source definition changes");
    let state=serde_json::to_string(&bad).unwrap();
    let game_error=Game::from_persisted(&serde_json::to_string(&bad.game).unwrap()).err().expect("foreign frame must reject JC044 Game restore");
    let room_error=crate::room::RoomEnvelope::from_persisted(&state).err().expect("foreign frame must reject JC044 Room restore");
    assert_eq!(game_error,"JC044不能挂接其他牌的堆栈程序");assert_eq!(room_error,game_error);
    serde_json::json!({"name":format!("stack-{kind}-source-identity"),"state":state,"gameError":game_error,"roomError":room_error,
        "roomId":room.game.room_id,"inviteCode":room.game.invite_code,"revision":room.revision})
}
#[test]
fn jc044_entry_stack_identity_is_symmetric_and_legal_entries_resume(){
    for reveal in [false,true]{for accept in [false,true]{
        let mut g=u::game();u::fund(&mut g,0,"JC044",3);
        let id=if reveal{let id=u::field(&mut g,"JC044",0);g.board_mut(&id).unwrap().face_down=true;id}else{u::held(&mut g,"JC044",0)};
        let kind=if reveal{"reveal"}else{"deploy"};let mut t=Trace::new(&format!("stack-{kind}-{accept}"),&g);
        t.game(0,Action{card_id:Some(id),region:(!reveal).then_some(0),..Action::new(kind)},true);
        invalid_entry_stack(&t.room,kind);
        t.responses();t.choose(if accept{vec!["accept".into()]}else{vec![]});t.responses();
        assert_eq!(t.room.game.regions[0].cards.iter().find(|c|c.definition=="JC044").unwrap().time_markers,u32::from(accept));
    }}
}
#[test]
fn jc044_native85_oracles(){
    use crate::room::SessionAction;
    let mut scenarios=vec![];let mut invalid=vec![];
    for reveal in [false,true]{for accept in [false,true]{
        let mut g=u::game();g.room_id=format!("{:024x}",850000+usize::from(reveal)*2+usize::from(accept));u::fund(&mut g,0,"JC044",3);
        let id=if reveal{let id=u::field(&mut g,"JC044",0);g.board_mut(&id).unwrap().face_down=true;id}else{u::held(&mut g,"JC044",0)};
        let name=format!("entry-{}-{}",if reveal{"reveal"}else{"deploy"},if accept{"accept"}else{"skip"});let mut t=Trace::new(&name,&g);
        t.game(0,Action{card_id:Some(id),region:(!reveal).then_some(0),..Action::new(if reveal{"reveal"}else{"deploy"})},true);
        if !accept{invalid.push(invalid_entry_stack(&t.room,if reveal{"reveal"}else{"deploy"}));}
        t.responses();
        if !reveal&&accept{invalid.extend(invalid_pending(&t.room,"declare"));}
        t.choose(if accept{vec!["accept".into()]}else{vec![]});t.responses();
        assert_eq!(t.room.game.regions[0].cards.iter().find(|c|c.definition=="JC044").unwrap().time_markers,u32::from(accept));
        scenarios.push(t.finish());
    }}
    for actor in 0..4{
        let (mut g,source)=layout(actor);g.room_id=format!("{:024x}",850100+actor);let target=(actor+2)%4;let team=g.team(target);
        g.regions[1].influence[team]=2;g.regions[3].influence[team]=1;
        let mut t=Trace::new(&format!("actor-{actor}"),&g);let action=activate(&source,&source,target);
        t.game((actor+1)%4,action.clone(),false);t.game(actor,action,true);t.responses();
        if actor==0{invalid.extend(invalid_pending(&t.room,"repression"));}
        let id=t.room.game.pending.as_ref().unwrap().choice.id.clone();
        let wrong=Action{choice_id:Some(id.clone()),selected:Some(vec!["region:3".into()]),..Action::new("choose")};
        t.game(actor,wrong.clone(),false);t.command(actor,SessionAction::PauseRoom,true);t.game(target,wrong.clone(),false);
        t.command(actor,SessionAction::ResumeRoom,true);t.choose(vec!["region:3".into()]);t.game(target,wrong,false);
        assert_eq!(t.room.game.regions[3].influence[team],0);assert_eq!(t.room.game.board(&source).unwrap().1.time_markers,1);
        scenarios.push(t.finish());
    }
    for (i,(name,target,has_influence))in[("target-self",0,true),("target-partner",1,true),("target-empty",2,false)].into_iter().enumerate(){
        let (mut g,source)=layout(0);g.room_id=format!("{:024x}",850200+i);let team=g.team(target);if has_influence{g.regions[2].influence[team]=1;}
        let mut t=Trace::new(name,&g);t.game(0,activate(&source,&source,target),true);t.responses();
        if has_influence{t.choose(vec!["region:2".into()]);}else{assert!(t.room.game.pending.is_none());}
        assert_eq!(t.room.game.board(&source).unwrap().1.time_markers,1);scenarios.push(t.finish());
    }
    {
        let(mut g,source)=layout(0);g.room_id=format!("{:024x}",850300);g.board_mut(&source).unwrap().time_markers=0;
        let host=u::field(&mut g,"JC029",0);let mut card=g.make_card("XQ14",0);card.time_markers=2;let fee=card.id.clone();g.attachments.push(Attachment{card,host_id:host.clone()});
        let mut card=g.make_card("XQ14",2);card.time_markers=3;let foreign=card.id.clone();g.attachments.push(Attachment{card,host_id:host});
        g.regions[2].influence[1]=1;let mut t=Trace::new("physical-attachment-cost",&g);
        t.game(0,activate(&source,&foreign,2),false);t.game(0,activate(&source,&fee,2),true);t.responses();t.choose(vec!["region:2".into()]);
        assert_eq!(t.room.game.board(&fee).unwrap().1.time_markers,1);assert_eq!(t.room.game.board(&foreign).unwrap().1.time_markers,3);scenarios.push(t.finish());
    }
    {
        let(mut g,source)=layout(0);g.room_id=format!("{:024x}",850301);let spell=u::held(&mut g,"XQ03",2);u::fund(&mut g,2,"XQ03",2);
        g.regions[2].influence[1]=1;let mut t=Trace::new("responsive-exhaust",&g);t.game(0,activate(&source,&source,2),true);
        // Real response composition while the paid ability remains on stack.
        let w=loop {
            let w=t.room.pacing.window.clone().expect("paid action response window");
            if w.members.contains_key(&2){break w;}
            let seat=*w.members.iter().find(|(_,d)|matches!(d,crate::room::Decision::Undecided{..})).unwrap().0;
            t.command(seat,SessionAction::PassResponse{window_id:w.id},true);
        };
        t.command(2,SessionAction::BeginResponse{window_id:w.id.clone(),intent_id:"jc044-response".into()},true);
        t.command(2,SessionAction::SubmitResponse{window_id:w.id,intent_id:"jc044-response".into(),action:Action{card_id:Some(spell),target_id:Some(source.clone()),..Action::new("play")}},true);
        t.responses();assert_eq!(t.room.game.pending.as_ref().unwrap().seat,2);t.choose(vec!["region:2".into()]);
        assert!(t.room.game.board(&source).unwrap().1.exhausted);assert_eq!(t.room.game.board(&source).unwrap().1.time_markers,1);scenarios.push(t.finish());
    }
    assert_eq!(scenarios.len(),13);assert_eq!(invalid.len(),37);
    if let Ok(dir)=std::env::var("JC044_NATIVE_DIR"){
        std::fs::create_dir_all(&dir).unwrap();let output=serde_json::json!({"format":"jc044-native85-v1","scope":"Explicit offline layouts with real Room commands, payments, responses, pause/reopen and complete private views; no online gameplay claim","catalog":catalog::catalog(),"sourceBindings":source_bindings(),"scenarios":scenarios,"invalid":invalid});
        use std::io::Write;std::fs::OpenOptions::new().create_new(true).write(true).open(std::path::Path::new(&dir).join("native85.json")).unwrap().write_all(&serde_json::to_vec(&output).unwrap()).unwrap();
    }
}

fn source_bindings() -> serde_json::Value {
    use sha2::{Digest, Sha256};
    let files: &[(&str, &[u8])] = &[
        ("Cargo.lock", include_bytes!("../../Cargo.lock")),
        ("Cargo.toml", include_bytes!("../../Cargo.toml")),
        ("rust-game-wasm/Cargo.toml", include_bytes!("../../rust-game-wasm/Cargo.toml")),
        ("rust-game-wasm/build.sh", include_bytes!("../../rust-game-wasm/build.sh")),
        ("rust-game-wasm/src/lib.rs", include_bytes!("../../rust-game-wasm/src/lib.rs")),
        ("rust-game/Cargo.toml", include_bytes!("../Cargo.toml")),
        ("rust-game/data/cards.json", include_bytes!("../data/cards.json")),
        ("rust-game/src/attachment.rs", include_bytes!("attachment.rs")),
        ("rust-game/src/attributes.rs", include_bytes!("attributes.rs")),
        ("rust-game/src/bin/hegemony-audit.rs", include_bytes!("bin/hegemony-audit.rs")),
        ("rust-game/src/black_expansion.rs", include_bytes!("black_expansion.rs")),
        ("rust-game/src/black_expansion_tests.rs", include_bytes!("black_expansion_tests.rs")),
        ("rust-game/src/blue_expansion.rs", include_bytes!("blue_expansion.rs")),
        ("rust-game/src/blue_expansion_tests.rs", include_bytes!("blue_expansion_tests.rs")),
        ("rust-game/src/blue_minimum.rs", include_bytes!("blue_minimum.rs")),
        ("rust-game/src/blue_minimum_tests.rs", include_bytes!("blue_minimum_tests.rs")),
        ("rust-game/src/bounded_entry_fresh_tests.rs", include_bytes!("bounded_entry_fresh_tests.rs")),
        ("rust-game/src/bounded_search.rs", include_bytes!("bounded_search.rs")),
        ("rust-game/src/bounded_search_tests.rs", include_bytes!("bounded_search_tests.rs")),
        ("rust-game/src/bq030.rs", include_bytes!("bq030.rs")),
        ("rust-game/src/bq030_preparation.rs", include_bytes!("bq030_preparation.rs")),
        ("rust-game/src/bq035.rs", include_bytes!("bq035.rs")),
        ("rust-game/src/bq035_tests.rs", include_bytes!("bq035_tests.rs")),
        ("rust-game/src/catalog.rs", include_bytes!("catalog.rs")),
        ("rust-game/src/control.rs", include_bytes!("control.rs")),
        ("rust-game/src/control_tests.rs", include_bytes!("control_tests.rs")),
        ("rust-game/src/death_observer_tests.rs", include_bytes!("death_observer_tests.rs")),
        ("rust-game/src/death_observers.rs", include_bytes!("death_observers.rs")),
        ("rust-game/src/deck.rs", include_bytes!("deck.rs")),
        ("rust-game/src/deck_seal_search.rs", include_bytes!("deck_seal_search.rs")),
        ("rust-game/src/deck_seal_search_draft.rs", include_bytes!("deck_seal_search_draft.rs")),
        ("rust-game/src/deck_seal_search_tests.rs", include_bytes!("deck_seal_search_tests.rs")),
        ("rust-game/src/defence_equipment_tests.rs", include_bytes!("defence_equipment_tests.rs")),
        ("rust-game/src/dream_reveal_tests.rs", include_bytes!("dream_reveal_tests.rs")),
        ("rust-game/src/engine.rs", include_bytes!("engine.rs")),
        ("rust-game/src/equipment_abilities_tests.rs", include_bytes!("equipment_abilities_tests.rs")),
        ("rust-game/src/gray_expansion.rs", include_bytes!("gray_expansion.rs")),
        ("rust-game/src/gray_expansion_tests.rs", include_bytes!("gray_expansion_tests.rs")),
        ("rust-game/src/gray_lock.rs", include_bytes!("gray_lock.rs")),
        ("rust-game/src/gray_lock_tests.rs", include_bytes!("gray_lock_tests.rs")),
        ("rust-game/src/green_minimum_tests.rs", include_bytes!("green_minimum_tests.rs")),
        ("rust-game/src/hand_deck_tests.rs", include_bytes!("hand_deck_tests.rs")),
        ("rust-game/src/hand_interactions_tests.rs", include_bytes!("hand_interactions_tests.rs")),
        ("rust-game/src/jc004_tests.rs", include_bytes!("jc004_tests.rs")),
        ("rust-game/src/jc005_tests.rs", include_bytes!("jc005_tests.rs")),
        ("rust-game/src/jc008_tests.rs", include_bytes!("jc008_tests.rs")),
        ("rust-game/src/jc029_tests.rs", include_bytes!("jc029_tests.rs")),
        ("rust-game/src/jc030_tests.rs", include_bytes!("jc030_tests.rs")),
        ("rust-game/src/jc037.rs", include_bytes!("jc037.rs")),
        ("rust-game/src/jc037_tests.rs", include_bytes!("jc037_tests.rs")),
        ("rust-game/src/jc044.rs", include_bytes!("jc044.rs")),
        ("rust-game/src/jc044_tests.rs", include_bytes!("jc044_tests.rs")),
        ("rust-game/src/jc046.rs", include_bytes!("jc046.rs")),
        ("rust-game/src/jc046_tests.rs", include_bytes!("jc046_tests.rs")),
        ("rust-game/src/jc050.rs", include_bytes!("jc050.rs")),
        ("rust-game/src/jc050_tests.rs", include_bytes!("jc050_tests.rs")),
        ("rust-game/src/jc052.rs", include_bytes!("jc052.rs")),
        ("rust-game/src/jc052_tests.rs", include_bytes!("jc052_tests.rs")),
        ("rust-game/src/jc054.rs", include_bytes!("jc054.rs")),
        ("rust-game/src/jc054_tests.rs", include_bytes!("jc054_tests.rs")),
        ("rust-game/src/jc057.rs", include_bytes!("jc057.rs")),
        ("rust-game/src/jc057_tests.rs", include_bytes!("jc057_tests.rs")),
        ("rust-game/src/jc060.rs", include_bytes!("jc060.rs")),
        ("rust-game/src/jc060_tests.rs", include_bytes!("jc060_tests.rs")),
        ("rust-game/src/jc061_tests.rs", include_bytes!("jc061_tests.rs")),
        ("rust-game/src/jc087_tests.rs", include_bytes!("jc087_tests.rs")),
        ("rust-game/src/jc089_tests.rs", include_bytes!("jc089_tests.rs")),
        ("rust-game/src/jc094_tests.rs", include_bytes!("jc094_tests.rs")),
        ("rust-game/src/jc095.rs", include_bytes!("jc095.rs")),
        ("rust-game/src/jc095_tests.rs", include_bytes!("jc095_tests.rs")),
        ("rust-game/src/jz22.rs", include_bytes!("jz22.rs")),
        ("rust-game/src/jz22_tests.rs", include_bytes!("jz22_tests.rs")),
        ("rust-game/src/jz31_tests.rs", include_bytes!("jz31_tests.rs")),
        ("rust-game/src/jz35.rs", include_bytes!("jz35.rs")),
        ("rust-game/src/jz35_tests.rs", include_bytes!("jz35_tests.rs")),
        ("rust-game/src/jz46.rs", include_bytes!("jz46.rs")),
        ("rust-game/src/jz46_tests.rs", include_bytes!("jz46_tests.rs")),
        ("rust-game/src/jz48_tests.rs", include_bytes!("jz48_tests.rs")),
        ("rust-game/src/jz49_tests.rs", include_bytes!("jz49_tests.rs")),
        ("rust-game/src/jz50_search.rs", include_bytes!("jz50_search.rs")),
        ("rust-game/src/jz50_search_tests.rs", include_bytes!("jz50_search_tests.rs")),
        ("rust-game/src/jz55_tests.rs", include_bytes!("jz55_tests.rs")),
        ("rust-game/src/lib.rs", include_bytes!("lib.rs")),
        ("rust-game/src/main.rs", include_bytes!("main.rs")),
        ("rust-game/src/mill_public_tests.rs", include_bytes!("mill_public_tests.rs")),
        ("rust-game/src/mobility.rs", include_bytes!("mobility.rs")),
        ("rust-game/src/model.rs", include_bytes!("model.rs")),
        ("rust-game/src/msjc01_tests.rs", include_bytes!("msjc01_tests.rs")),
        ("rust-game/src/msjc06_tests.rs", include_bytes!("msjc06_tests.rs")),
        ("rust-game/src/msjc07_tests.rs", include_bytes!("msjc07_tests.rs")),
        ("rust-game/src/msjc08_tests.rs", include_bytes!("msjc08_tests.rs")),
        ("rust-game/src/msjc11_tests.rs", include_bytes!("msjc11_tests.rs")),
        ("rust-game/src/play_sources.rs", include_bytes!("play_sources.rs")),
        ("rust-game/src/protection_tests.rs", include_bytes!("protection_tests.rs")),
        ("rust-game/src/purple_tools_tests.rs", include_bytes!("purple_tools_tests.rs")),
        ("rust-game/src/red_time.rs", include_bytes!("red_time.rs")),
        ("rust-game/src/red_time_tests.rs", include_bytes!("red_time_tests.rs")),
        ("rust-game/src/region_aura_tests.rs", include_bytes!("region_aura_tests.rs")),
        ("rust-game/src/renown.rs", include_bytes!("renown.rs")),
        ("rust-game/src/renown_tests.rs", include_bytes!("renown_tests.rs")),
        ("rust-game/src/repress_assets_tests.rs", include_bytes!("repress_assets_tests.rs")),
        ("rust-game/src/resolution.rs", include_bytes!("resolution.rs")),
        ("rust-game/src/room.rs", include_bytes!("room.rs")),
        ("rust-game/src/response_preview_tests.rs", include_bytes!("response_preview_tests.rs")),
        ("rust-game/src/rules.rs", include_bytes!("rules.rs")),
        ("rust-game/src/sealing.rs", include_bytes!("sealing.rs")),
        ("rust-game/src/sealing_tests.rs", include_bytes!("sealing_tests.rs")),
        ("rust-game/src/service.rs", include_bytes!("service.rs")),
        ("rust-game/src/society.rs", include_bytes!("society.rs")),
        ("rust-game/src/society_msjc09_tests.rs", include_bytes!("society_msjc09_tests.rs")),
        ("rust-game/src/unit_support.rs", include_bytes!("unit_support.rs")),
        ("rust-game/src/white_wound_tests.rs", include_bytes!("white_wound_tests.rs")),
        ("rust-game/src/win_flow_tests.rs", include_bytes!("win_flow_tests.rs")),
        ("rust-game/src/wm023.rs", include_bytes!("wm023.rs")),
        ("rust-game/src/wm023_tests.rs", include_bytes!("wm023_tests.rs")),
        ("rust-game/src/wm038.rs", include_bytes!("wm038.rs")),
        ("rust-game/src/wm038_tests.rs", include_bytes!("wm038_tests.rs")),
        ("rust-game/src/wm056.rs", include_bytes!("wm056.rs")),
        ("rust-game/src/wm056_tests.rs", include_bytes!("wm056_tests.rs")),
        ("rust-game/src/wm062.rs", include_bytes!("wm062.rs")),
        ("rust-game/src/wm062_tests.rs", include_bytes!("wm062_tests.rs")),
        ("rust-game/src/world.rs", include_bytes!("world.rs")),
        ("rust-game/src/wound_defence_tests.rs", include_bytes!("wound_defence_tests.rs")),
        ("rust-game/src/xq18_tests.rs", include_bytes!("xq18_tests.rs")),
        ("rust-game/src/xq24_tests.rs", include_bytes!("xq24_tests.rs")),
        ("rust-game/src/xq25.rs", include_bytes!("xq25.rs")),
        ("rust-game/src/xq25_tests.rs", include_bytes!("xq25_tests.rs")),
        ("rust-game/src/xq27.rs", include_bytes!("xq27.rs")),
        ("rust-game/src/xq27_tests.rs", include_bytes!("xq27_tests.rs")),
        ("rust-game/src/xq37.rs", include_bytes!("xq37.rs")),
        ("rust-game/src/xq37_tests.rs", include_bytes!("xq37_tests.rs")),
        ("rust-game/src/yellow_search_batch_tests.rs", include_bytes!("yellow_search_batch_tests.rs")),
        ("rust-game/src/msjc0405_tests.rs", include_bytes!("msjc0405_tests.rs")),
    ];
    serde_json::json!({"algorithm":"sha256","files":files.iter().map(|(path,bytes)| ((*path).to_owned(),format!("{:x}",Sha256::digest(bytes)))).collect::<std::collections::BTreeMap<_,_>>()})
}
