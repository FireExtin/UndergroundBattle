//! Fresh normal-command Room witnesses for the two batch20 review findings.
//! Explicit layouts are test inputs, not a browser match or a save migration.
use crate::{catalog, model::*, room::{Decision, RoomCommand, RoomEnvelope, SessionAction}, unit_support::*};
use serde_json::{json, Value};

pub(super) struct Trace { name: String, pub(super) room: RoomEnvelope, initial: Value, steps: Vec<Value> }
impl Trace {
    pub(super) fn new(mut game: Game, name: String, index: usize) -> Self {
        game.room_id = format!("112{:021x}", index + 100);
        game.invite_code = "112ABC".into();
        game.seed = u64::MAX - 112;
        game.random = game.seed;
        Self::new_preserving_rng(game,name,index)
    }
    pub(super) fn new_preserving_rng(mut game:Game,name:String,index:usize)->Self {
        game.room_id = format!("112{:021x}", index + 100);
        game.invite_code = "112ABC".into();
        let room = crate::jc029_tests::envelope(&game);
        let initial = json!({"state":serde_json::to_string(&room).unwrap(),
            "views":(0..4).map(|seat|room.view(seat,0)).collect::<Vec<_>>()});
        Self { name, room, initial, steps: vec![] }
    }
    pub(super) fn step(&mut self, seat: usize, action: SessionAction) {
        let state = serde_json::to_string(&self.room).unwrap();
        let command = RoomCommand { command_id: format!("review-{}-{}",self.name,self.steps.len()),
            expected_version: self.room.revision, action };
        let expected = self.room.transition(seat,Some(command.clone()),0).unwrap();
        if expected.outcome!="accepted" {if let Ok(path)=std::env::var("ROOM_TRACE_FAILURE_OUT"){std::fs::write(path,serde_json::to_vec(&json!({"state":state,"seat":seat,"command":command,"response":expected})).unwrap()).unwrap();}}
        assert_eq!(expected.outcome,"accepted","{}: {:?}",self.name,(&command.action,expected.error_message));
        self.room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        self.steps.push(json!({"state":state,"seat":seat,"command":command,"serverNowMs":"0",
            "expected":expected,"views":(0..4).map(|seat|self.room.view(seat,0)).collect::<Vec<_>>()}));
    }
    pub(super) fn action(&mut self, seat: usize, action: Action) { self.step(seat,SessionAction::Game{action}); }
    pub(super) fn choose(&mut self, ids: Vec<String>) {
        let p=self.room.game.pending.as_ref().unwrap();
        self.action(p.seat,Action{choice_id:Some(p.choice.id.clone()),selected:Some(ids),..Action::new("choose")});
    }
    pub(super) fn pass(&mut self) {
        if let Some(w)=&self.room.pacing.window {
            let seat=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
            self.step(seat,SessionAction::PassResponse{window_id:w.id.clone()});
        } else {
            let seat=(0..4).find(|seat|self.room.game.legal_actions(*seat).iter().any(|a|a.action.kind=="pass")).unwrap();
            self.action(seat,Action::new("pass"));
        }
    }
    pub(super) fn until_pending(&mut self) {
        for _ in 0..128 { if self.room.game.pending.is_some(){return;} self.pass(); }
        panic!("{}: normal pending not reached",self.name);
    }
    pub(super) fn value(self) -> Value { json!({"name":self.name,"seedDecimal":self.room.game.seed.to_string(),
        "initial":self.initial,"steps":self.steps}) }
}

fn gate(second: u8, index: usize) -> Value {
    let mut g=game();fund(&mut g,0,"WM057",4);let host=field(&mut g,"JC125",0);
    let source=held(&mut g,"CQ28",0);g.players[0].deck.clear();
    for i in 0..13 {let c=g.make_card(if i==0{"WM042"}else{"JC125"},0);g.players[0].deck.push(c);}
    let mut t=Trace::new(g,format!("cq28-mill-{second}"),index);
    t.action(0,Action{card_id:Some(source),..Action::new("play")});
    t.choose(vec!["item-0".into()]);t.choose(vec![format!("item-{second}")]);t.until_pending();
    assert!(matches!(t.room.game.pending.as_ref().unwrap().resolution,ChoiceResolution::Rumor{..}));
    t.choose(vec![host.clone()]);
    assert!(t.room.game.pending.is_none()&&t.room.game.stack.is_empty()&&t.room.game.effects.is_empty());
    assert_eq!(t.room.game.players[0].deck.len(),if second==0{1}else{6});
    assert_eq!(t.room.game.players[0].hand.len(),usize::from(second==3));
    assert_eq!(t.room.game.sealed_cards.iter().filter(|s|s.host_id==host).count(),1);
    assert_eq!(t.room.game.resources(0),0);t.value()
}

fn mystic(vampire_first: bool, index: usize) -> Value {
    let mut g=game();let source=field(&mut g,"WM057",0);field(&mut g,"WM025",0);
    fund(&mut g,0,"JC125",2);g.regions[0].influence[1]=1;
    let shown=[held(&mut g,"WM025",0),held(&mut g,"WM025",1),held(&mut g,"JC125",2),held(&mut g,"JC125",3)];
    let mut t=Trace::new(g,format!("mystic-order-{vampire_first}"),index);
    t.action(0,Action{card_id:Some(source.clone()),ability_id:Some(crate::import20_broker::ACT.into()),..Action::new("activate")});
    t.until_pending();
    for seat in 0..4 {
        let p=t.room.game.pending.as_ref().unwrap();assert_eq!(p.choice.kind,crate::mystic_contest::COMMIT_KIND);assert_eq!(p.seat,seat);
        t.choose(vec![shown[seat].clone()]);
    }
    while t.room.game.pending.as_ref().is_some_and(|p|p.choice.kind==crate::mystic_contest::REWARD_KIND){t.choose(vec![]);}
    let first=if vampire_first{crate::import20_vampires::WIN}else{crate::import20_broker::WIN};
    let p=t.room.game.pending.as_ref().unwrap();assert_eq!(p.choice.kind,"batch3-entry-order");
    let id=p.choice.options.iter().find(|o|o.id.ends_with(first)).unwrap().id.clone();t.choose(vec![id]);
    assert!(matches!(&t.room.game.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration:d,..}if d.ability.key==first));
    for _ in 0..128 {
        if let Some(p)=t.room.game.pending.clone(){
            let selected=match &p.resolution{
                ChoiceResolution::Declare{declaration:d,..}if d.ability.key==crate::import20_broker::WIN=>vec!["repress".into()],
                ChoiceResolution::Declare{declaration:d,..}if d.ability.key==crate::import20_vampires::WIN=>vec!["accept".into()],
                ChoiceResolution::Frame{choice:FrameChoice::WM057Opponent,..}=>vec!["p2".into()],
                ChoiceResolution::Frame{choice:FrameChoice::RepressOne{..},..}=>vec!["region:0".into()],
                _=>panic!("unexpected normal mystic pending {}",p.choice.kind),
            };t.choose(selected);
        }else if !t.room.game.stack.is_empty(){t.pass();}else{break;}
    }
    // Entry order is stack order. If WM025 resolves first its influence
    // removes the existing enemy marker; repression first leaves room for it.
    assert_eq!(t.room.game.regions[0].influence,if vampire_first{[1,0]}else{[0,0]});
    assert_eq!(t.room.game.mystic_history.len(),1);
    assert!(t.room.game.pending.is_none()&&t.room.game.stack.is_empty()&&t.room.game.entry_order_groups.is_empty());
    assert_eq!(t.room.game.resources(0),0);assert!(t.room.game.board(&source).unwrap().1.exhausted);t.value()
}

fn eyes(reverse: bool, index: usize) -> Value {
    let mut g=game();let host=field(&mut g,"JC084",0);field(&mut g,"JC084",0);let enemy=field(&mut g,"JC084",2);
    fund(&mut g,0,"JC029",4);held(&mut g,"JC125",2);held(&mut g,"JC125",2);
    let hands=[held(&mut g,"BQ039",0),held(&mut g,"BQ039",0)];
    let mut t=Trace::new(g,format!("eye-order-{reverse}"),index);
    for hand in hands {
        t.action(0,Action{card_id:Some(hand),target_id:Some(host.clone()),..Action::new("play")});
        for _ in 0..32 {if t.room.game.stack.is_empty(){break;}t.pass();}
    }
    let eye_ids=t.room.game.attachments.iter().filter(|a|a.card.definition=="BQ039").map(|a|a.card.id.clone()).collect::<Vec<_>>();
    assert_eq!(eye_ids.len(),2);assert_eq!(t.room.game.resources(0),2);
    t.until_pending();let p=t.room.game.pending.clone().unwrap();
    assert!(matches!(p.resolution,ChoiceResolution::Damage{..}));
    t.action(p.seat,Action{choice_id:Some(p.choice.id),allocations:Some([(enemy,p.choice.amount.unwrap())].into()),..Action::new("choose")});
    let p=t.room.game.pending.as_ref().unwrap();assert_eq!(p.choice.kind,"batch3-entry-order");
    let first=&eye_ids[usize::from(reverse)];let id=p.choice.options.iter().find(|o|o.id.starts_with(&format!("{first}:"))).unwrap().id.clone();t.choose(vec![id]);
    assert!(matches!(&t.room.game.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare{declaration:d,..}if d.source.card.id==*first));
    for _ in 0..128 {
        if let Some(p)=t.room.game.pending.clone(){match p.resolution {
            ChoiceResolution::Declare{declaration:d,..}if d.ability.key==crate::import20_observers::EYE_WIN=>t.choose(vec!["p2".into()]),
            ChoiceResolution::Frame{choice:FrameChoice::Discard{seat:2,..},..}=>t.choose(vec![p.choice.options[0].id.clone()]),
            _=>panic!("unexpected normal eye pending {}",p.choice.kind),
        }}else if !t.room.game.stack.is_empty(){t.pass();}else{break;}
    }
    assert!(t.room.game.players[2].hand.is_empty());
    assert_eq!(t.room.game.players[2].graveyard.iter().filter(|c|c.definition=="JC125").count(),2);
    assert!(t.room.game.entry_order_groups.is_empty()&&t.room.game.pending.is_none());t.value()
}

#[test]
fn abi112_review_fixes_actual_room_normal_witnesses() {
    let cases=vec![gate(0,0),gate(3,1),mystic(false,2),mystic(true,3),eyes(false,4),eyes(true,5)];
    if let Ok(file)=std::env::var("ABI112_REVIEW_NATIVE_OUT") {
        use std::io::Write;
        let mut output=std::fs::OpenOptions::new().write(true).create_new(true).open(file).unwrap();
        output.write_all(&serde_json::to_vec(&json!({"format":"abi112-review-normal-native-v1",
            "method":"Explicit current Native layouts, actual Room commands, all four views. Not a natural browser match or a save migration.",
            "catalog":catalog::catalog(),"cases":cases})).unwrap()).unwrap();
    }
}
