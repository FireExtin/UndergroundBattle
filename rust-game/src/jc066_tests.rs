//! Explicit offline layouts with real commands and exact persistence; no online claims.
use crate::{catalog, model::*, rules::*, unit_support as u};
fn responses(g: &mut Game) {
    for _ in 0..64 {
        if g.stack.is_empty() || g.pending.is_some() { return; }
        u::pass(g);
    }
    panic!("bounded JC066 responses");
}
fn entry(g: &mut Game, reveal: bool) -> String {
    u::fund(g,0,"JC066",5);
    let id=if reveal { let id=u::field(g,"JC066",0);g.board_mut(&id).unwrap().face_down=true;id } else { u::held(g,"JC066",0) };
    u::act(g,0,Action { card_id:Some(id),region:(!reveal).then_some(0),..Action::new(if reveal {"reveal"} else {"deploy"}) });
    responses(g);
    g.regions[0].cards.iter().find(|c| c.definition=="JC066"&&!c.face_down).unwrap().id.clone()
}
fn seal(g: &mut Game, target: &str) { u::choose(g,vec![target.into()]);responses(g); }
#[test]
fn jc066_complete_print_program_and_modifier_cannot_be_split_or_transplanted() {
    let c=catalog::card("JC066");assert_eq!(c.cost,5);assert_eq!(c.loyalty,vec!["灰色";3]);assert_eq!(c.subtitle.as_deref(),Some("样本B7021"));
    assert!(c.unique);assert_eq!(c.defense,Some(2));assert_eq!(c.permanent_icons.influence,2);assert_eq!(c.temporary_icons.investigation,1);
    assert_eq!(c.magic_icon,MagicIcon::Mind);assert_eq!(c.subtypes,vec!["人类","超能力者"]);
    let d=crate::jc066::definition();crate::jc066::validate_definition("JC066",&d).unwrap();
    for n in 0..7 {let mut bad=d.clone();match n {0=>bad.modifiers.clear(),1=>bad.abilities.clear(),2=>bad.abilities[0].targets[0].range=Range::Anywhere,
        3=>bad.abilities[0].targets[0].relation=Relation::EnemyTeam,4=>bad.abilities[0].event=Some(Event::Reveal),
        5=>bad.abilities[0].costs.push(Cost::ExhaustSource),_=>bad.abilities[0].ops.push(Op::Draw{player:PlayerRef::Actor,count:1,end:DeckEnd::Top})}
        assert!(crate::jc066::validate_definition("JC066",&bad).is_err());}
    assert!(crate::jc066::validate_definition("JC044",&d).is_err());
    let mut nested=d.abilities[0].clone();nested.key="foreign".into();nested.ops=vec![Op::ForEachLivingPlayer(vec![Op::JC066SealLocalTargetOnSource])];
    assert!(crate::jc066::validate_ability("JC044",&nested).is_err());
}
#[test]
fn jc066_deploy_and_paid_reveal_optional_entry_seal_without_death() {
    for reveal in [false,true] {for skip in [false,true] {
        let mut g=u::game();let target=u::field(&mut g,"JC029",2);let source=entry(&mut g,reveal);
        assert_eq!(g.pending.as_ref().unwrap().seat,0);
        if skip { u::choose(&mut g,vec![]);assert!(g.board(&target).is_some());assert!(g.sealed_cards.is_empty()); }
        else { seal(&mut g,&target);assert!(g.board(&target).is_none());assert_eq!(g.sealed_cards.len(),1);assert_eq!(g.sealed_cards[0].host_id,source);assert!(g.players[2].graveyard.is_empty()); }
        u::restore(&mut g);
    }}
}
#[test]
fn jc066_secret_deploy_neither_seals_nor_gains_faceup_icons() {
    let mut g=u::game();let id=u::held(&mut g,"JC066",0);u::fund(&mut g,0,"JC066",2);
    u::act(&mut g,0,Action { card_id:Some(id),region:Some(0),..Action::new("conceal") });responses(&mut g);
    assert!(g.pending.is_none());assert!(g.sealed_cards.is_empty());
    let c=g.regions[0].cards.iter().find(|c|c.definition=="JC066").unwrap();assert!(c.face_down);assert_eq!(g.current_icons(c,0),Icons { influence:1,..Default::default() });
}
#[test]
fn jc066_payment_loyalty_and_extra_entry_payloads_reject_atomically() {
    let mut g=u::game();let id=u::held(&mut g,"JC066",0);u::fund(&mut g,0,"JC066",2);u::fund(&mut g,0,"JC029",3);
    let a=Action { card_id:Some(id),region:Some(0),..Action::new("deploy") };u::reject(&mut g,0,a.clone());
    u::fund(&mut g,0,"JC066",3);
    for n in 0..3 {let mut bad=a.clone();match n {0=>bad.target_id=Some("p2".into()),1=>bad.cost_selected=Some(vec![]),_=>bad.ability_id=Some(crate::jc066::KEY.into())};u::reject(&mut g,0,bad);}
}
#[test]
fn jc066_seals_hidden_publicly_and_returns_to_owner_when_host_leaves() {
    let mut g=u::game();let target=u::field(&mut g,"JC029",2);g.board_mut(&target).unwrap().face_down=true;g.board_mut(&target).unwrap().controller=1;
    let source=entry(&mut g,false);seal(&mut g,&target);let s=&g.sealed_cards[0];assert_eq!(s.card.owner,2);assert_eq!(s.card.controller,2);assert!(!s.card.face_down);assert_ne!(s.card.id,target);
    let spell=u::field(&mut g,"JC075",0);u::fund(&mut g,0,"JC075",2);
    u::act(&mut g,0,Action { card_id:Some(spell),target_id:Some(source.clone()),..Action::new("activate") });responses(&mut g);
    assert!(g.sealed_cards.is_empty());assert!(g.players[2].hand.iter().any(|c|c.definition=="JC029"));assert!(g.players[0].hand.iter().any(|c|c.definition=="JC066"));assert!(g.board(&source).is_none());u::restore(&mut g);
}
#[test]
fn jc066_self_seal_is_atomic_host_departure_and_returns_owner_hand() {
    let mut g=u::game();let source=entry(&mut g,false);seal(&mut g,&source);
    assert!(g.sealed_cards.is_empty());assert!(g.board(&source).is_none());assert!(g.players[0].hand.iter().any(|c|c.definition=="JC066"));assert!(g.players[0].graveyard.is_empty());u::restore(&mut g);
}
#[test]
fn jc066_uses_only_sealed_character_printed_icons_and_current_host_initiative() {
    let mut g=u::game();let source=u::field(&mut g,"JC066",0);let victim=g.make_card("JC095",2);let d=catalog::card("JC095");
    g.seal_card_on_valid_host(victim,2,&source);let attachment=g.make_card("JZ46",2);g.seal_card_on_valid_host(attachment,2,&source);
    let c=g.board(&source).unwrap().1.clone();let (p,t)=g.jc066_sealed_printed_icons(&c,0);assert_eq!(p,d.permanent_icons);assert_eq!(t,d.temporary_icons);
    g.first_team=0;assert_eq!(g.current_icons(&c,0),catalog::card("JC066").permanent_icons.add(d.permanent_icons).add(catalog::card("JC066").temporary_icons).add(d.temporary_icons));
    g.first_team=1;assert_eq!(g.current_icons(&c,0),catalog::card("JC066").permanent_icons.add(d.permanent_icons));
    g.board_mut(&source).unwrap().exhausted=true;assert_eq!(g.icons(g.board(&source).unwrap().1,0),Icons::default());assert_ne!(g.current_icons(g.board(&source).unwrap().1,0),Icons::default());u::restore(&mut g);
}
#[test]
fn jc066_printed_icons_ignore_victims_conditional_bonuses_and_keep_live_uid_only() {
    let mut g=u::game();let source=u::field(&mut g,"JC066",0);let victim=u::field(&mut g,"JC030",2);u::fund(&mut g,2,"XQ14",2);
    assert!(g.jc030_blood_assets_active(g.board(&victim).unwrap().1));let (_,c)=g.leave_board(&victim).unwrap();g.seal_card_on_valid_host(c,2,&source);
    let c=g.board(&source).unwrap().1.clone();let (p,t)=g.jc066_sealed_printed_icons(&c,0);assert_eq!(p,catalog::card("JC030").permanent_icons);assert_eq!(t,catalog::card("JC030").temporary_icons);
    let mut fake=c.clone();fake.id="departed".into();assert_eq!(g.jc066_sealed_printed_icons(&fake,0),(Icons::default(),Icons::default()));
    let mut hidden=c.clone();hidden.face_down=true;assert_eq!(g.jc066_sealed_printed_icons(&hidden,0),(Icons::default(),Icons::default()));u::restore(&mut g);
}
#[test]
fn jc066_entry_containers_reject_foreign_frozen_source_both_directions() {
    for reveal in [false,true] {
        let mut g=u::game();u::fund(&mut g,0,"JC066",5);let id=if reveal {let id=u::field(&mut g,"JC066",0);g.board_mut(&id).unwrap().face_down=true;id} else {u::held(&mut g,"JC066",0)};
        u::act(&mut g,0,Action { card_id:Some(id),region:(!reveal).then_some(0),..Action::new(if reveal {"reveal"} else {"deploy"}) });
        for other in [false,true] {let mut bad=g.clone();let item=&mut bad.stack[0];if other {item.card.as_mut().unwrap().definition="LC19".into();}else{item.frame.as_mut().unwrap().source.card.definition="LC19".into();}
            assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());}
    }
}
#[test]
fn jc066_pending_metadata_and_direct_frame_cannot_bypass_choice_or_response() {
    let mut g=u::game();let target=u::field(&mut g,"JC029",2);entry(&mut g,false);
    for n in 0..7 {let mut bad=g.clone();let p=bad.pending.as_mut().unwrap();match n {0=>p.seat=2,1=>p.choice.player_id="p2".into(),2=>p.choice.title.clear(),3=>p.choice.min=Some(1),4=>p.choice.allow_decline=Some(false),5=>p.choice.options.clear(),_=>p.choice.options[0].label="forged".into()};assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());}
    u::choose(&mut g,vec![target]);let mut bad=g.clone();let f=bad.stack.pop().unwrap().frame.unwrap();bad.effects.push_back(Effect::Frame { frame:Box::new(f) });assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
}
#[test]
fn jc066_sealing_performs_existing_attachment_and_nested_seal_departure_cleanup() {
    let mut g=u::game();let target=u::field(&mut g,"JC029",2);let attachment=g.make_card("XQ14",2);
    g.attachments.push(Attachment { card:attachment,host_id:target.clone() });
    let child=g.make_card("JC084",3);g.seal_card_on_valid_host(child,3,&target);
    let source=entry(&mut g,false);seal(&mut g,&target);
    assert_eq!(g.sealed_cards.len(),1);assert_eq!(g.sealed_cards[0].host_id,source);
    assert!(g.attachments.is_empty());assert!(g.players[2].graveyard.iter().any(|c|c.definition=="XQ14"));assert!(g.players[3].hand.iter().any(|c|c.definition=="JC084"));u::restore(&mut g);
}
#[test]
fn jc066_original_host_or_region_departure_cancels_without_sealing_another_instance() {
    for host_leaves in [false,true] {
        let mut g=u::game();let target=u::field(&mut g,"JC029",2);let source=entry(&mut g,false);u::choose(&mut g,vec![target.clone()]);
        if host_leaves {let (_,c)=g.leave_board(&source).unwrap();let c=g.reset_zone_card(c);g.players[0].hand.push(c);}
        else {g.regions[0].card=g.make_card("DQJC115",0);}
        responses(&mut g);assert!(g.board(&target).is_some());assert!(g.sealed_cards.is_empty());u::restore(&mut g);
    }
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
fn bad_room(room:&crate::room::RoomEnvelope,name:String)->serde_json::Value {
    let state=serde_json::to_string(room).unwrap();
    let game_error=Game::from_persisted(&serde_json::to_string(&room.game).unwrap()).err().unwrap_or_else(||panic!("Game accepted {name}"));
    let room_error=crate::room::RoomEnvelope::from_persisted(&state).err().unwrap_or_else(||panic!("Room accepted {name}"));
    serde_json::json!({"name":name,"state":state,"gameError":game_error,"roomError":room_error,"roomId":room.game.room_id,"inviteCode":room.game.invite_code,"revision":room.revision})
}
fn bad_paid_entries(room:&crate::room::RoomEnvelope,kind:&str)->Vec<serde_json::Value> {
    (0..16).map(|n| {
        let mut bad=room.clone();let item=&mut bad.game.stack[0];
        match n {
            0=>item.frame.as_mut().unwrap().source.card.definition="LC19".into(),
            1=>item.card.as_mut().unwrap().definition="LC19".into(),
            2=>item.frame.as_mut().unwrap().frame_id="forged".into(),
            3=>item.controller=(item.controller+1)%4,
            4=>item.card=None,
            5=>item.deploy_region=None,
            6=>item.reveal=!item.reveal,
            7=>item.target=Some("p2".into()),
            8=>item.frame.as_mut().unwrap().steps.push(Step{context:0,op:Op::JC066SealLocalTargetOnSource}),
            9=>item.frame.as_mut().unwrap().actor=2,
            10=>item.frame.as_mut().unwrap().source.source_region_instance=None,
            11=>item.frame.as_mut().unwrap().guard=GuardState::Accepted,
            12=>item.frame.as_mut().unwrap().cursor=1,
            13=>item.frame.as_mut().unwrap().already_paid.clear(),
            14=>item.frame.as_mut().unwrap().source.play_source=if kind=="deploy"{None}else{Some(PlaySource::Hand)},
            _=>item.frame=None,
        }
        bad_room(&bad,format!("stack-{kind}-{n:02}"))
    }).collect()
}
fn bad_declarations(room:&crate::room::RoomEnvelope)->Vec<serde_json::Value> {
    (0..25).map(|n| {
        let mut bad=room.clone();let p=bad.game.pending.as_mut().unwrap();
        match n {
            0=>p.seat=2,1=>p.choice.player_id="p2".into(),2=>p.choice.kind="forged".into(),3=>p.choice.title.clear(),4=>p.choice.description.clear(),
            5=>p.choice.min=None,6=>p.choice.max=Some(2),7=>p.choice.allow_decline=Some(false),8=>p.choice.amount=Some(1),
            9=>p.choice.preview_cards.push(p.choice.options[0].card.clone().unwrap()),10=>p.choice.id.clear(),11=>p.choice.options.clear(),
            12=>p.choice.options.push(p.choice.options[0].clone()),13=>p.choice.options.reverse(),14=>p.choice.options[0].id="forged".into(),
            15=>p.choice.options[0].label="forged".into(),16=>p.choice.options[0].card=None,
            17=>{let ChoiceResolution::Declare{stage,..}=&mut p.resolution else{panic!()};*stage=DeclareChoice::Accept;},
            18=>{let ChoiceResolution::Declare{declaration,..}=&mut p.resolution else{panic!()};declaration.actor=2;},
            19=>bad.game.players[0].eliminated=true,
            _=>{let ChoiceResolution::Declare{declaration:d,..}=&mut p.resolution else{panic!()};match n {
                20=>d.source.card.definition="LC19".into(),21=>d.ability.targets[0].range=Range::Anywhere,
                22=>d.ability.ops.push(Op::JC066SealLocalTargetOnSource),23=>d.source.source_region_instance=None,_=>d.source.region=None,
            }},
        }
        bad_room(&bad,format!("declare-{n:02}"))
    }).collect()
}
fn bad_target_frames(room:&crate::room::RoomEnvelope)->Vec<serde_json::Value> {
    (0..28).map(|n| {
        let mut bad=room.clone();let item=&mut bad.game.stack[0];let f=item.frame.as_mut().unwrap();
        match n {
            0=>f.source.card.definition="LC19".into(),1=>f.ability_key="other".into(),2=>f.steps[0].op=Op::Draw{player:PlayerRef::Actor,count:1,end:DeckEnd::Top},
            3=>f.steps.push(f.steps[0].clone()),4=>f.steps[0].context=2,5=>f.cursor=1,6=>f.guard=GuardState::Accepted,
            7=>f.targets.clear(),8=>f.targets[0].spec.range=Range::Anywhere,9=>f.targets[0].spec.relation=Relation::EnemyTeam,
            10=>f.targets[0].spec.kind=EntityKind::Attachment,11=>f.targets[0].id=f.source.source_region_instance.clone().unwrap(),
            12=>f.targets[0].public.label="forged".into(),13=>f.targets[0].public.owner=Some("p99".into()),
            14=>f.targets[0].public.controller=Some("p99".into()),15=>f.targets[0].public.region=None,16=>f.targets[0].public.kind="attachment".into(),
            17=>f.targets[0].public.valid=false,18=>f.targets[0].public.status="invalid".into(),19=>f.targets[0].region_instance=Some("forged".into()),
            20=>f.source.source_region_instance=None,21=>f.already_paid.push(PaidCost::Assets(vec![])),
            22=>item.card=Some(f.source.card.clone()),23=>{let copy=item.clone();bad.game.stack.push(copy);},
            24|25=>{let mut f=bad.game.stack.pop().unwrap().frame.unwrap();if n==25{f.cursor=1;f.guard=GuardState::Accepted;}bad.game.effects.push_back(Effect::Frame{frame:Box::new(f)});},
            26=>{let f=bad.game.stack.pop().unwrap().frame.unwrap();bad.game.pending=Some(Pending{seat:0,choice:Choice{id:"forged".into(),kind:"handSeal".into(),title:"forged".into(),description:"forged".into(),player_id:"p0".into(),options:vec![],preview_cards:vec![],min:Some(0),max:Some(0),amount:None,allow_decline:Some(false)},resolution:ChoiceResolution::Frame{frame:Box::new(f),choice:FrameChoice::HandSeal{seat:0,host_id:"forged".into()}}});},
            _=>item.target=Some("p2".into()),
        }
        bad_room(&bad,format!("frame-{n:02}"))
    }).collect()
}
#[test]
fn jc066_native86_oracles() {
    use crate::room::SessionAction;
    let mut scenarios=vec![];let mut invalid=vec![];
    for reveal in [false,true] {for skip in [false,true] {
        let mut g=u::game();g.room_id=format!("{:024x}",860000+usize::from(reveal)*2+usize::from(skip));
        let victim=u::field(&mut g,"JC029",2);u::fund(&mut g,0,"JC066",5);
        let id=if reveal{let id=u::field(&mut g,"JC066",0);g.board_mut(&id).unwrap().face_down=true;id}else{u::held(&mut g,"JC066",0)};
        let kind=if reveal{"reveal"}else{"deploy"};let mut t=Trace::new(&format!("entry-{kind}-{}",if skip{"skip"}else{"accept"}),&g);
        t.game(0,Action{card_id:Some(id),region:(!reveal).then_some(0),..Action::new(kind)},true);
        if !skip{invalid.extend(bad_paid_entries(&t.room,kind));}
        t.responses();let source=t.room.game.regions[0].cards.iter().find(|c|c.definition=="JC066").unwrap().id.clone();
        if !reveal&&!skip{invalid.extend(bad_declarations(&t.room));}
        let p=t.room.game.pending.as_ref().unwrap();let choice=Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![victim.clone()]),..Action::new("choose")};
        t.game(2,choice.clone(),false);t.command(0,SessionAction::PauseRoom,true);t.game(0,choice.clone(),false);t.command(0,SessionAction::ResumeRoom,true);
        t.choose(if skip{vec![]}else{vec![victim.clone()]});
        if !reveal&&!skip{invalid.extend(bad_target_frames(&t.room));}
        t.responses();t.game(0,choice,false);
        assert_eq!(t.room.game.sealed_cards.len(),usize::from(!skip));
        if !skip{assert_eq!(t.room.game.sealed_cards[0].host_id,source);}
        scenarios.push(t.finish());
    }}
    for actor in 0..4 {
        let mut g=u::game();g.room_id=format!("{:024x}",860100+actor);g.begin_window(Window::Action(g.team(actor)));u::fund(&mut g,actor,"JC066",5);let source=u::held(&mut g,"JC066",actor);
        let action=g.legal_actions(actor).into_iter().find(|a|a.action.kind=="deploy"&&a.action.card_id.as_deref()==Some(&source)).expect("actual legal deploy for this seat").action;
        let region=action.region.unwrap();let victim=u::field(&mut g,"JC095",(actor+2)%4);
        if actor%2==1{g.board_mut(&victim).unwrap().face_down=true;}
        if region!=0{let i=g.regions[0].cards.iter().position(|c|c.id==victim).unwrap();let c=g.regions[0].cards.remove(i);g.regions[region].cards.push(c);}
        let mut t=Trace::new(&format!("actor-{actor}"),&g);
        t.game(actor,action,true);t.responses();t.choose(vec![victim]);t.responses();
        assert_eq!(t.room.game.sealed_cards[0].card.owner,(actor+2)%4);scenarios.push(t.finish());
    }
    {
        let mut g=u::game();g.room_id=format!("{:024x}",860200);u::fund(&mut g,0,"JC066",5);let id=u::held(&mut g,"JC066",0);let mut t=Trace::new("self-seal",&g);
        t.game(0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")},true);t.responses();let source=t.room.game.regions[0].cards.iter().find(|c|c.definition=="JC066").unwrap().id.clone();
        t.choose(vec![source]);t.responses();assert!(t.room.game.sealed_cards.is_empty());assert!(t.room.game.players[0].hand.iter().any(|c|c.definition=="JC066"));scenarios.push(t.finish());
    }
    for (i,gift) in ["JC095","JZ46"].into_iter().enumerate() {
        let mut g=u::game();g.room_id=format!("{:024x}",860210+i);let host=u::field(&mut g,"JC066",0);let sealer=u::field(&mut g,"XQ40",0);let id=u::held(&mut g,gift,0);u::fund(&mut g,0,"JC066",2);
        let mut t=Trace::new(if i==0{"shared-hand-seal-character"}else{"shared-hand-seal-attachment"},&g);
        t.game(0,Action{card_id:Some(sealer),target_id:Some(host.clone()),..Action::new("activate")},true);t.responses();t.choose(vec![id]);t.responses();
        assert_eq!(t.room.game.sealed_cards.len(),1);let c=t.room.game.board(&host).unwrap().1;
        assert_eq!(t.room.game.jc066_sealed_printed_icons(c,0).0,if i==0{catalog::card(gift).permanent_icons}else{Icons::default()});scenarios.push(t.finish());
    }
    for (i,kind) in ["host-hide","target-hide","host-return"].into_iter().enumerate() {
        let mut g=u::game();g.room_id=format!("{:024x}",860300+i);let victim=u::field(&mut g,"JC029",2);u::fund(&mut g,0,"JC066",7);let source=u::held(&mut g,"JC066",0);
        let response=if kind=="host-return"{u::field(&mut g,"JC075",0)}else{u::fund(&mut g,2,"JC063",2);u::held(&mut g,"JC063",2)};
        let mut t=Trace::new(kind,&g);t.game(0,Action{card_id:Some(source),region:Some(0),..Action::new("deploy")},true);t.responses();let host=t.room.game.regions[0].cards.iter().find(|c|c.definition=="JC066").unwrap().id.clone();t.choose(vec![victim.clone()]);
        let a=if kind=="host-return"{Action{card_id:Some(response),target_id:Some(host),..Action::new("activate")}}else{Action{card_id:Some(response),target_id:Some(if kind=="host-hide"{host}else{victim}),option:Some("hide".into()),..Action::new("play")}};
        t.respond(if kind=="host-return"{0}else{2},a);t.responses();assert!(t.room.game.sealed_cards.is_empty());scenarios.push(t.finish());
    }
    assert_eq!(scenarios.len(),14);assert_eq!(invalid.len(),85);
    if let Ok(dir)=std::env::var("JC066_NATIVE_DIR") {
        use std::io::Write;std::fs::create_dir_all(&dir).unwrap();let result=serde_json::json!({"format":"jc066-native86-v1","scope":"Synthetic offline layouts; real Room commands, complete states/journals/private views and receipts; not online play","catalog":catalog::catalog(),"sourceBindings":source_bindings(),"scenarios":scenarios,"invalid":invalid});
        std::fs::OpenOptions::new().create_new(true).write(true).open(std::path::Path::new(&dir).join("native86.json")).unwrap().write_all(&serde_json::to_vec(&result).unwrap()).unwrap();
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
        ("rust-game/src/jc066.rs", include_bytes!("jc066.rs")),
        ("rust-game/src/jc066_tests.rs", include_bytes!("jc066_tests.rs")),
    ];
    serde_json::json!({"algorithm":"sha256","files":files.iter().map(|(path,bytes)| ((*path).to_owned(),format!("{:x}",Sha256::digest(bytes)))).collect::<std::collections::BTreeMap<_,_>>()})
}
