//! Public construction; every operation after lobby creation is a real
//! Room command. No runtime cards, assets, grave entries or markers are injected.
use crate::{catalog::{self,DeckEntry},deck::DeckDraft,model::*,room::{Decision,RoomCommand,RoomEnvelope,SessionAction}};
use serde_json::{json,Value};
use std::{collections::BTreeMap,fs::File,io::Write,path::PathBuf};

const ADMITTED:&[&str]=&["BQ034","CQ25","XG25"];

#[derive(Clone,Copy,Debug,PartialEq)]
enum Case {Entry,Shown,Black,Unique}
impl Case {fn name(self)->&'static str {match self {Self::Entry=>"bq034-natural-entry-counter",Self::Shown=>"bq034-natural-shown-feed",Self::Black=>"cq25-xg25-natural-seal-move-search",Self::Unique=>"bq034-natural-two-unique-sacrifice"}}}

fn draft(case:Case,seat:usize)->DeckDraft {
    let mut entries=match (case,seat) {
        (Case::Unique,0)=>vec![("BQ034",2)],
        (Case::Black,0)=>vec![("CQ25",3),("XG25",3),("CQ07",1),("BQ090",3)],
        (Case::Black,_)=>vec![("BQ090",3),("JC125",3),("JC084",3),("JZ48",3)],
        (_,0)=>vec![("BQ034",1),("JC029",3),("XQ14",3),("BQ090",3)],
        (_,_)=>vec![("JC005",3),("JC102",3)],
    }.into_iter().map(|(id,count)|DeckEntry{card_id:id.into(),count}).collect::<Vec<_>>();
    let mut names=entries.iter().map(|e|catalog::card(&e.card_id).name.clone()).collect::<Vec<_>>();
    let raw:Value=serde_json::from_str(include_str!("../data/cards.json")).unwrap();
    let printed_unique=raw["cards"].as_array().unwrap().iter().filter(|c|c["unique"]==true).map(|c|c["id"].as_str().unwrap()).collect::<Vec<_>>();
    let colors=if case==Case::Black {vec!["黑","蓝"]} else if seat==0 {vec!["蓝","黄","紫"]} else {vec!["黄","紫","蓝"]};
    let mut groups=colors.iter().map(|color|catalog::catalog().cards.iter().filter(|c|c.supported&&c.color==*color
        &&matches!(c.kind.as_str(),"character"|"spell"|"attachment")
        &&crate::deck::copy_limit(c)!=Some(1)&&!printed_unique.contains(&c.id.as_str())&&!c.text.contains("资产")
        &&!names.contains(&c.name)).map(|c|c.id.clone()).collect::<Vec<_>>()).collect::<Vec<_>>();
    let mut n:usize=entries.iter().map(|e|e.count).sum();let mut index=0;
    while n<50 {
        let group=index%groups.len();index+=1;
        let Some(id)=groups[group].first().cloned() else {assert!(!groups.iter().all(|g|g.is_empty()),"insufficient ordinary filler definitions");continue;};
        groups[group].remove(0);let c=catalog::card(&id);
        if names.contains(&c.name){continue;}
        names.push(c.name.clone());let count=(50-n).min(3);entries.push(DeckEntry{card_id:id,count});n+=count;
    }
    let d=DeckDraft{id:format!("natural-{}-{seat}",case.name()),name:format!("Natural candidate {seat}"),description:"Public legal construction; ordinary copies and all real commands".into(),society_id:None, core_card_id:None,cards:entries,
        rules_version:catalog::RULES_VERSION.into(),card_pool_version:catalog::POOL_VERSION.into(),engine_version:catalog::ENGINE_VERSION.into(),updated_at:"2026-10-25T00:00:00Z".into()};
    let d=crate::deck::validate(d).unwrap();let mut by_name=BTreeMap::<String,usize>::new();
    for e in &d.cards {assert!(e.count<=3);*by_name.entry(catalog::card(&e.card_id).name.clone()).or_default()+=e.count;}
    assert!(by_name.values().all(|n|*n<=3));assert_eq!(d.cards.iter().map(|e|e.count).sum::<usize>(),50);
    assert!(!d.cards.iter().any(|e|matches!(e.card_id.as_str(),"BQ009"|"BQ033")));d
}

struct Natural {case:Case,room:RoomEnvelope,sequence:usize,out:Option<File>,record:bool,region:usize,adjacent:usize,targets:BTreeMap<&'static str,String>,show_bq034:bool,unique_choices:usize}
impl Natural {
    fn lobby(case:Case,seed:u64,record:bool)->Self {
        let decks=[draft(case,0),draft(case,1)];
        let suffix=seed*10+match case {Case::Entry=>0,Case::Shown=>1,Case::Black=>2,Case::Unique=>3};
        let mut game=Game::new_with_deck(format!("a112{suffix:020x}"),"NAT20A".into(),"duel".into(),"P0".into(),decks[0].clone(),seed).unwrap();
        let created=serde_json::to_string(&game).unwrap();game.join_with_deck("P1".into(),decks[1].clone()).unwrap();
        let room=RoomEnvelope::from_game(game);
        let out=if record {std::env::var_os("NEXT20_NATURAL_OUT_DIR").map(|dir|{let p=PathBuf::from(dir);std::fs::create_dir_all(&p).unwrap();File::options().write(true).create_new(true).open(p.join(format!("{}.jsonl",case.name()))).unwrap()})}else{None};
        let mut t=Self{case,room,sequence:0,out,record,region:0,adjacent:1,targets:BTreeMap::new(),show_bq034:false,unique_choices:0};
        if record {t.write(&json!({"kind":"initial","format":"next20-natural-public-room-v1","case":case.name(),"seedDecimal":seed.to_string(),"candidateConstructionOnly":[],"admittedCards":ADMITTED,"publicAdmission":true,"decks":decks,"createGameState":created,"state":serde_json::to_string(&t.room).unwrap(),"catalog":catalog::catalog(),"views":t.views()}));}t
    }
    fn write(&mut self,v:&Value){if let Some(out)=&mut self.out {serde_json::to_writer(&mut *out,v).unwrap();out.write_all(b"\n").unwrap();out.flush().unwrap();}}
    fn views(&self)->Vec<Value>{(0..2).map(|s|serde_json::to_value(self.room.view(s,0)).unwrap()).collect()}
    fn step(&mut self,seat:usize,action:SessionAction){
        assert!(self.sequence<2500,"bounded natural command limit");
        let state=if self.record {serde_json::to_string(&self.room).unwrap()}else{String::new()};let command=RoomCommand{command_id:format!("natural-{}-{}",self.case.name(),self.sequence),expected_version:self.room.revision,action};
        if self.record {self.write(&json!({"kind":"attempt","seat":seat,"command":command,"serverNowMs":"0"}));}
        let unique_choice=self.room.game.pending.as_ref().is_some_and(|p|matches!(p.resolution,ChoiceResolution::Unique))
            &&matches!(&command.action,SessionAction::Game{action} if action.kind=="choose");
        let expected=self.room.transition(seat,Some(command.clone()),0).unwrap();
        if self.record {self.write(&json!({"kind":"outcome","outcome":expected.outcome,"error":expected.error_message}));}
        assert_eq!(expected.outcome,"accepted","{} step {} turn {} {:?}: {:?}",self.case.name(),self.sequence,self.room.game.turn,command,expected.error_message);
        if unique_choice {self.unique_choices+=1;}
        self.room=RoomEnvelope::from_persisted(&expected.state).unwrap();
        if self.record {let duplicate=self.room.transition(seat,Some(command.clone()),0).unwrap();RoomEnvelope::from_persisted(&duplicate.state).unwrap();
        let views=self.views();self.write(&json!({"kind":"step","state":state,"seat":seat,"command":command,"serverNowMs":"0","expected":expected,"duplicate":duplicate,"views":views}));}self.sequence+=1;
    }
    fn action(&mut self,seat:usize,a:Action){self.step(seat,SessionAction::Game{action:a});}
    fn boot(&mut self){for s in 0..2 {self.action(s,Action::new("ready"));}self.action(0,Action::new("start"));
        self.region=self.room.game.regions.iter().position(|r|r.card.definition=="DQJC115").unwrap_or(0);self.adjacent=if self.region==0{1}else{self.region-1};self.settle();}
    fn hand(&self,seat:usize,id:&str)->usize{self.room.game.players[seat].hand.iter().filter(|c|c.definition==id).count()}
    fn definition(&self,id:&str)->Option<&str>{let g=&self.room.game;
        g.players.iter().flat_map(|p|p.hand.iter().chain(&p.deck).chain(&p.assets).chain(&p.graveyard))
            .chain(g.regions.iter().flat_map(|r|&r.cards)).chain(g.attachments.iter().map(|a|&a.card)).chain(g.sealed_cards.iter().map(|s|&s.card)).find(|c|c.id==id).map(|c|c.definition.as_str())}
    fn keep(&self,seat:usize,id:&str)->usize {match (self.case,seat,id){
        (Case::Unique,0,"BQ034")=>2,(Case::Black,0,"XG25")=>2,(Case::Black,0,"CQ25"|"CQ07"|"BQ090")=>1,
        (Case::Black,1,"BQ090"|"JC125"|"JC084"|"JZ48")=>1,
        (_,0,"BQ034"|"JC029"|"XQ14"|"BQ090")=>1,
        (_,1,"JC005"|"JC102")=>1,_=>0}}
    fn selection(&self)->Vec<String>{
        let p=self.room.game.pending.as_ref().unwrap();let options=&p.choice.options;let kind=p.choice.kind.as_str();
        let contains=|id:&str|options.iter().any(|o|o.id==id);
        if self.case==Case::Unique&&matches!(p.resolution,ChoiceResolution::Unique){
            assert_eq!(p.seat,0);assert_eq!(p.choice.min,Some(1));assert_eq!(p.choice.max,Some(1));assert_eq!(options.len(),2);
            assert!(options.iter().all(|o|self.definition(&o.id)==Some("BQ034")));
            let selected=self.targets.get("unique-sacrifice").unwrap();assert!(contains(selected));return vec![selected.clone()];
        }
        if kind=="mulligan" {return vec![];}
        if kind==crate::mystic_contest::REWARD_KIND {return vec![];}
        if kind==crate::mystic_contest::COMMIT_KIND {
            if p.seat==0&&self.show_bq034 {return vec![options.iter().find(|o|self.definition(&o.id)==Some("BQ034")).unwrap().id.clone()];}
            if self.case==Case::Black&&p.seat==0 {if let Some(o)=options.iter().find(|o|self.definition(&o.id)==Some("XG25")){return vec![o.id.clone()];}}
            return vec![options.iter().min_by_key(|o|self.definition(&o.id).map_or(0,|d|if self.case==Case::Black&&p.seat==0&&d=="BQ090"{1000}else{self.keep(p.seat,d)})).unwrap().id.clone()];
        }
        if kind==crate::next20_black_wraiths::SEARCH {return vec![options.iter().find(|o|self.definition(&o.id)==Some("XG25")).unwrap().id.clone()];}
        for key in ["seal-host","vampire"] {if let Some(id)=self.targets.get(key){if contains(id){return vec![id.clone()];}}}
        if let Some(o)=options.iter().find(|o|self.definition(&o.id)==Some("XQ14")){if kind!= "discard" {return vec![o.id.clone()];}}
        if contains("accept"){return vec!["accept".into()];}
        let min=p.choice.min.unwrap_or(0) as usize;if min==0{return vec![];}
        let mut options=options.iter().collect::<Vec<_>>();options.sort_by_key(|o|self.definition(&o.id).map_or(0,|d|self.keep(p.seat,d)));
        assert!(options.len()>=min,"choice {kind}");options.into_iter().take(min).map(|o|o.id.clone()).collect()
    }
    fn idle(&self)->bool{self.room.game.pending.is_none()&&self.room.game.stack.is_empty()}
    fn pass(&mut self){if self.room.game.pending.is_some(){let p=self.room.game.pending.as_ref().unwrap();let seat=p.seat;let id=p.choice.id.clone();
        let a=if matches!(p.resolution,ChoiceResolution::Forecast{..}) {Action{choice_id:Some(id),top:Some(p.choice.options.iter().map(|o|o.id.clone()).collect()),bottom:Some(vec![]),..Action::new("choose")}}
            else if matches!(p.resolution,ChoiceResolution::Damage{..}) {Action{choice_id:Some(id),allocations:Some(BTreeMap::from([(p.choice.options[0].id.clone(),p.choice.amount.unwrap())])),..Action::new("choose")}}
            else{let selected=self.selection();Action{choice_id:Some(id),selected:Some(selected),..Action::new("choose")}};
        self.action(seat,a);return;}
        if let Some(w)=&self.room.pacing.window {let seat=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;let id=w.id.clone();self.step(seat,SessionAction::PassResponse{window_id:id});}
        else {let seat=(0..2).find(|s|self.room.game.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap_or_else(||panic!("no pass {:?}",self.room.game.window));self.action(seat,Action::new("pass"));}}
    fn settle(&mut self){for _ in 0..300 {if self.idle(){return;}self.pass();}panic!("settle exceeded bound");}
    fn asset(&mut self,seat:usize)->bool{
        let g=&self.room.game;if !g.can_standard(seat)||g.players[seat].asset_used{return false;}
        let assets=&g.players[seat].assets;let want=if self.case==Case::Black {if seat==0&&!assets.iter().any(|c|catalog::card(&c.definition).color=="蓝"){Some("蓝")}else if !assets.iter().any(|c|catalog::card(&c.definition).color=="黑"){Some("黑")}else{None}}
            else if seat==0 {if assets.iter().filter(|c|catalog::card(&c.definition).color=="蓝").count()<2{Some("蓝")}else{None}}
            else if !assets.iter().any(|c|catalog::card(&c.definition).color=="黄"){Some("黄")}else if !assets.iter().any(|c|catalog::card(&c.definition).color=="紫"){Some("紫")}else{None};
        let mut actions=g.legal_actions(seat).into_iter().filter(|a|a.action.kind=="asset").filter(|a|{let d=self.definition(a.action.card_id.as_ref().unwrap()).unwrap();self.hand(seat,d)>self.keep(seat,d)}).collect::<Vec<_>>();
        actions.sort_by_key(|a|{let d=self.definition(a.action.card_id.as_ref().unwrap()).unwrap();u8::from(want.is_some_and(|color|catalog::card(d).color!=color))});
        if let Some(a)=actions.into_iter().next(){self.action(seat,a.action);true}else{false}
    }
    fn play(&mut self,seat:usize,id:&str,target:Option<&str>,region:Option<usize>)->bool{
        let action=self.room.game.legal_actions(seat).into_iter().find(|a|matches!(a.action.kind.as_str(),"play"|"deploy")
            &&a.action.card_id.as_deref().is_some_and(|uid|self.definition(uid)==Some(id))
            &&a.action.target_id.as_deref()==target&&(a.action.kind!="deploy"||a.action.region==region));
        if let Some(a)=action {self.action(seat,a.action);self.settle();true}else{false}
    }
    fn board(&self,seat:usize,id:&str)->Option<String>{self.room.game.regions.iter().flat_map(|r|&r.cards).find(|c|c.controller==seat&&c.definition==id&&!c.face_down).map(|c|c.id.clone())}
    fn attachment(&self,host:&str)->Option<String>{self.room.game.attachments.iter().find(|a|a.host_id==host&&a.card.definition=="XQ14").map(|a|a.card.id.clone())}
    fn finish(&mut self,details:Value){assert!(self.idle());assert!(self.room.game.death_replacements.is_empty());
        self.step(0,SessionAction::PauseRoom);assert!(self.room.pacing.pause.is_some());
        self.step(0,SessionAction::ResumeRoom);assert!(self.room.pacing.pause.is_none());
        self.write(&json!({"kind":"complete","case":self.case.name(),"seedDecimal":self.room.game.seed.to_string(),"commands":self.sequence,"publicAdmission":true,"candidateConstructionOnly":[],"admittedCards":ADMITTED,"details":details,"finalState":serde_json::to_string(&self.room).unwrap(),"views":self.views()}));
        eprintln!("NATURAL COMPLETE {} seed={} commands={} turn={}",self.case.name(),self.room.game.seed,self.sequence,self.room.game.turn);}
}

fn selected_seed(case:Case)->u64{
    for seed in [if case==Case::Black{1723}else{1603}] {
        let mut t=Natural::lobby(case,seed,false);t.boot();
        if !t.room.game.regions.iter().any(|r|r.card.definition=="DQJC115"){continue;}
        let yes=if case==Case::Black {t.hand(0,"CQ07")>0&&t.hand(0,"CQ25")>0&&t.hand(0,"XG25")>0&&t.hand(0,"BQ090")>0
            &&t.hand(1,"BQ090")>0&&["JC125","JC084","JZ48"].iter().map(|d|t.hand(1,d)).sum::<usize>()>=2}
            else {t.hand(0,"BQ034")>0&&t.hand(0,"JC029")>0&&t.hand(0,"XQ14")>0&&t.hand(1,"JC005")>0
                &&(case!=Case::Entry||t.hand(1,"JC102")>0)&&(case!=Case::Shown||t.hand(0,"BQ090")>0)};
        if yes {eprintln!("NATURAL SELECTED {:?} seed={seed}",case);return seed;}
    }panic!("recorded seed no longer supplies the verified natural opening for {:?}",case)
}

fn blue(case:Case){
    let seed=selected_seed(case);let mut t=Natural::lobby(case,seed,true);t.boot();let mut stage=0;let mut blood=String::new();
    for _ in 0..1800 {
        assert_eq!(t.room.game.status,"playing");assert!(t.room.game.turn<=15,"bounded natural turns");
        if t.room.game.can_standard(0){t.asset(0);}if t.room.game.can_standard(1){t.asset(1);}
        if stage==0&&t.room.game.can_standard(0)&&t.room.game.resources(0)>=6&&t.play(0,"JC029",None,Some(t.region)) {
            let vampire=t.board(0,"JC029").unwrap();t.targets.insert("vampire",vampire.clone());
            assert!(t.play(0,"XQ14",Some(&vampire),None));blood=t.attachment(&vampire).unwrap();stage=1;
        }
        if stage==1&&t.room.game.can_standard(1)&&t.play(1,"JC005",Some(&blood),None){
            assert!(t.room.game.players[0].graveyard.iter().any(|c|c.definition=="XQ14"));stage=2;
        }
        if stage==2&&t.room.game.can_standard(0){
            if case==Case::Entry&&t.room.game.resources(0)>=4&&t.play(0,"BQ034",None,Some(t.region)){
                let source=t.board(0,"BQ034").unwrap();assert!(t.attachment(&source).is_some());t.targets.insert("counter-source",source);stage=3;
            }else if case==Case::Shown&&t.room.game.resources(0)>=2 {
                t.show_bq034=true;
                let a=t.room.game.legal_actions(0).into_iter().find(|a|a.action.ability_id.as_deref()==Some(crate::import20_contest::HAND)).unwrap().action;
                t.action(0,a);t.settle();let vampire=t.targets["vampire"].clone();assert!(t.attachment(&vampire).is_some());assert_eq!(t.room.game.mystic_history.len(),1);
                t.finish(json!({"realGraveSource":"JC005 destroyed attached XQ14","shownBQ034Target":vampire,"shownBloodAttached":true}));return;
            }
        }
        if stage==3&&t.room.game.can_standard(1){let source=t.targets["counter-source"].clone();if t.play(1,"JC102",Some(&source),None){
            assert!(t.room.game.board(&source).is_some());assert!(t.attachment(&source).is_none());assert_eq!(t.room.game.board(&source).unwrap().1.damage,0);
            assert!(t.room.game.players[0].graveyard.iter().any(|c|c.definition=="XQ14"));
            t.finish(json!({"realGraveSource":"JC005 destroyed attached XQ14","entryBloodAttached":true,"counterSacrificedBlood":true,"enemySpellTerminated":true,"source":source}));return;
        }}
        t.pass();t.settle();
    }panic!("natural blue state machine did not finish stage {stage}");
}

fn black(){
    let seed=selected_seed(Case::Black);let mut t=Natural::lobby(Case::Black,seed,true);t.boot();let mut stage=0;let mut ghost=String::new();let mut moved=String::new();
    for _ in 0..1800 {
        assert_eq!(t.room.game.status,"playing");assert!(t.room.game.turn<=15);
        if t.room.game.can_standard(0){t.asset(0);}if t.room.game.can_standard(1){t.asset(1);}
        if stage==0&&t.room.game.can_standard(0)&&t.room.game.resources(0)>=6 {
            assert!(t.play(0,"CQ07",None,Some(t.region)));let host=t.board(0,"CQ07").unwrap();t.targets.insert("seal-host",host);
            assert!(t.room.game.players[0].society_zone.secrecy>0,"actual CQ07 trigger must supply moon");
            assert!(t.play(0,"XG25",None,Some(t.region)));ghost=t.board(0,"XG25").unwrap();stage=1;
        }
        if stage==1&&t.room.game.can_standard(1)&&t.room.game.resources(1)>=4 {
            let mut bodies=vec![];for id in ["JC125","JC084","JZ48"] {for _ in 0..t.hand(1,id){bodies.push(id);}}assert!(bodies.len()>=2);
            assert!(t.play(1,bodies[0],None,Some(t.region)));assert!(t.play(1,bodies[1],None,Some(t.adjacent)));
            let enemy_host=t.room.game.regions[t.region].cards.iter().find(|c|c.controller==1).unwrap().id.clone();t.targets.insert("seal-host",enemy_host);
            moved=t.room.game.regions[t.adjacent].cards.iter().find(|c|c.controller==1).unwrap().id.clone();
            assert!(t.play(1,"BQ090",Some(&ghost),None));
            assert!(t.room.game.sealed_cards.iter().any(|s|s.card.definition=="XG25"&&s.host_id==t.targets["seal-host"]));stage=2;
        }
        if stage==2&&t.room.game.can_standard(0)&&t.room.game.resources(0)>=7 {
            assert!(t.play(0,"CQ25",None,Some(t.region)));let tiger=t.board(0,"CQ25").unwrap();
            let sealed=t.room.game.sealed_cards.iter().find(|s|s.card.definition=="XG25"&&s.host_id==t.targets["seal-host"]).unwrap().card.id.clone();
            assert_eq!(t.room.game.board(&t.targets["seal-host"]).unwrap().1.controller,1);
            assert_eq!(t.room.game.sealed_cards.iter().find(|s|s.card.id==sealed).unwrap().card.owner,0);
            let a=t.room.game.legal_actions(0).into_iter().find(|a|a.action.card_id.as_deref()==Some(sealed.as_str())&&a.action.ability_id.as_deref()==Some(crate::next20_black_wraiths::MOVE)&&a.action.target_id.as_deref()==Some(moved.as_str())).unwrap().action;
            let before=t.room.game.resources(0);t.action(0,a);t.settle();assert_eq!(t.room.game.resources(0),before-1);assert!(t.room.game.regions[t.region].cards.iter().any(|c|c.id==moved));
            assert!(t.play(0,"BQ090",Some(&moved),None));
            assert!(t.room.game.sealed_cards.iter().any(|s|s.card.definition=="XG25"&&s.host_id==tiger));
            t.finish(json!({"moonSource":"real CQ07 deploy/entry","deathSealHost":t.targets["seal-host"],"sealedMovePayer":0,"moved":moved,"nonemptyTigerSearchAndSealHost":tiger,"onlyDeckGeneratedCards":true}));return;
        }
        t.pass();t.settle();
    }panic!("natural black state machine did not finish stage {stage}");
}

#[test]
fn next20_natural_public_admission_keeps_anchor_closed_and_copy_limits(){
    for id in ADMITTED {assert!(catalog::card(id).supported);}
    let d=draft(Case::Black,0);assert!(crate::deck::validate(d.clone()).is_ok());
    for id in ["BQ009","BQ033"] {assert!(!catalog::card(id).supported);let mut bad=d.clone();bad.cards.push(DeckEntry{card_id:id.into(),count:1});assert!(crate::deck::validate(bad).unwrap_err().contains("不能加入"));}
    let mut bad=d.clone();bad.cards.iter_mut().find(|e|e.card_id=="XG25").unwrap().count=4;assert!(crate::deck::validate(bad).unwrap_err().contains("同名"));
    let source=draft(Case::Entry,0);let yayoi=catalog::card("BQ034");assert!(yayoi.unique);assert!(yayoi.keywords.is_empty());assert_eq!(yayoi.deck_copy_limit,Some(3));
    for copies in 1..=3 {let mut legal=source.clone();legal.cards.iter_mut().find(|e|e.card_id=="BQ034").unwrap().count=copies;assert!(crate::deck::validate(legal).is_ok());}
    let mut bad=source;bad.cards.iter_mut().find(|e|e.card_id=="BQ034").unwrap().count=4;assert!(crate::deck::validate(bad).unwrap_err().contains("同名"));
}
#[test]fn next20_natural_bq034_entry_and_counter_from_deck(){blue(Case::Entry);}
#[test]fn next20_natural_bq034_shown_target_and_blood_from_deck(){blue(Case::Shown);}
#[test]fn next20_natural_cq25_xg25_search_death_seal_and_sealed_action_from_deck(){black();}

#[test]
fn next20_natural_bq034_two_same_name_enter_choose_one_sacrifice(){
    let seed=171;let mut opening=Natural::lobby(Case::Unique,seed,false);opening.boot();
    assert_eq!(opening.hand(0,"BQ034"),2);assert!(opening.room.game.regions.iter().any(|r|r.card.definition=="DQJC115"));
    eprintln!("NATURAL SELECTED Unique seed={seed}");
    let mut t=Natural::lobby(Case::Unique,seed,true);t.boot();let mut first=None;
    for _ in 0..1800 {
        assert_eq!(t.room.game.status,"playing");assert!(t.room.game.turn<=10);
        if t.room.game.can_standard(0){t.asset(0);}if t.room.game.can_standard(1){t.asset(1);}
        if t.room.game.can_standard(0)&&t.room.game.resources(0)>=4&&t.hand(0,"BQ034")>0 {
            if first.is_none(){assert!(t.play(0,"BQ034",None,Some(t.region)));let id=t.board(0,"BQ034").unwrap();
                assert_eq!(t.unique_choices,0);t.targets.insert("unique-sacrifice",id.clone());first=Some(id);
            }else{
                let old=first.unwrap();assert!(t.play(0,"BQ034",None,Some(t.adjacent)));
                let live=t.room.game.regions.iter().flat_map(|r|&r.cards).filter(|c|c.definition=="BQ034"&&c.controller==0).collect::<Vec<_>>();
                assert_eq!(t.unique_choices,1);assert_eq!(live.len(),1);assert_ne!(live[0].id,old);assert!(t.room.game.board(&old).is_none());
                let kept=live[0].id.clone();let grave=t.room.game.players[0].graveyard.iter().filter(|c|c.definition=="BQ034").collect::<Vec<_>>();
                assert_eq!(grave.len(),1);assert_eq!(grave[0].owner,0);assert_ne!(grave[0].id,old);
                let dead=grave[0].id.clone();assert!(t.room.game.pending.is_none());
                t.finish(json!({"printedUniqueNotDeckLimit":true,"copiesInDeck":2,"uniqueChoices":1,"selectedOldInstance":old,"keptNewInstance":kept,"ownerGraveInstance":dead,"differentRegions":true}));return;
            }
        }
        t.pass();t.settle();
    }panic!("bounded real double-unique trajectory");
}
