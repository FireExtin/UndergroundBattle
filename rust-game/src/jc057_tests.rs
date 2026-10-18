//! Explicit offline layouts; real view/restore paths. Never online play evidence.
use crate::{catalog, model::*, rules::*, unit_support as u};

fn setup() -> (Game, String, String) {
    let mut g = u::game();
    let source = u::field(&mut g, "JC057", 0);
    let hidden = u::field(&mut g, "JC052", 1);
    g.board_mut(&hidden).unwrap().face_down = true;
    (g, source, hidden)
}
#[test]
fn jc057_complete_print_and_closed_definition() {
    let d = catalog::card("JC057");
    assert_eq!((d.name.as_str(), d.cost, d.color.as_str(), d.magic.as_str(), d.defense), ("监控员",2,"灰","",Some(1)));
    assert_eq!(d.subtypes,["人类","工程师"]); assert_eq!(d.loyalty,["灰色"]);
    assert!(!d.unique); assert_eq!(d.permanent_icons,Icons::default());
    assert_eq!((d.temporary_icons.investigation,d.temporary_icons.combat,d.temporary_icons.influence),(1,0,1));
    let original=crate::jc057::definition();
    assert_eq!(original.abilities[0].event,Some(Event::Reveal));
    assert!(crate::jc057::validate_definition("JC057",&original).is_ok());
    assert!(crate::jc057::validate_definition("LC19",&original).is_err());
    for mutation in 0..6 {
        let mut altered=original.clone();
        match mutation {
            0=>altered.modifiers.clear(), 1=>altered.abilities.clear(),
            2=>altered.abilities[0].event=Some(Event::Enter),
            3=>altered.abilities[0].targets[0].relation=Relation::EnemyTeam,
            4=>altered.abilities[0].costs.push(Cost::Assets(1)),
            _=>altered.abilities[0].ops.push(Op::Draw{player:PlayerRef::Actor,count:1,end:DeckEnd::Top}),
        }
        assert!(crate::jc057::validate_definition("JC057",&altered).is_err());
    }
}
#[test]
fn jc057_continuous_inspection_is_controller_private_and_keeps_public_hidden_identity() {
    let (mut g,source,hidden)=setup();
    let owner_card=g.board(&hidden).unwrap().1.clone();
    let before=serde_json::to_string(&g).unwrap();
    let v=g.view(0); assert_eq!(v.private_inspections.len(),1);
    let p=&v.private_inspections[0];assert_eq!(p.source_instance_id,source);assert_eq!(p.region,0);
    assert_eq!(p.cards[0].instance_id,hidden);assert_eq!(p.cards[0].card_id.as_deref(),Some("JC052"));
    assert_eq!(p.cards[0].region,None);assert!(!p.cards[0].face_down);
    let board=v.regions[0].characters.iter().find(|c| c.instance_id==hidden).unwrap();
    assert!(board.face_down);assert!(board.card_id.is_none());assert_eq!(board.name,"暗藏者");
    for seat in 1..4 {assert!(g.view(seat).private_inspections.is_empty());}
    assert!(g.log.iter().all(|l| !l.text.contains("终焉的诗寇蒂")));
    assert_eq!(serde_json::to_string(&g).unwrap(),before);
    assert_eq!(serde_json::to_value(g.board(&hidden).unwrap().1).unwrap(),serde_json::to_value(owner_card).unwrap());
    u::restore(&mut g);
}
#[test]
fn jc057_read_permission_tracks_ready_face_region_and_current_control() {
    let (g,source,hidden)=setup();
    for mutation in 0..5 {
        let mut branch=g.clone();
        match mutation {
            0=>branch.board_mut(&source).unwrap().exhausted=true,
            1=>branch.board_mut(&source).unwrap().face_down=true,
            2=>{let c=branch.regions[0].cards.remove(0);branch.regions[1].cards.push(c);},
            3=>{branch.board_mut(&source).unwrap().controller=2;assert_eq!(branch.view(2).private_inspections.len(),1);},
            _=>{let c=branch.regions[0].cards.remove(0);branch.players[0].assets.push(c);},
        }
        assert!(branch.view(0).private_inspections.is_empty(),"mutation{mutation}");
        u::restore(&mut branch);
    }
    let mut branch=g.clone();branch.board_mut(&hidden).unwrap().face_down=false;
    assert!(branch.view(0).private_inspections.is_empty());
    let mut branch=g.clone();branch.players[0].eliminated=true;
    assert!(branch.view(0).private_inspections.is_empty());
}
#[test]
fn jc057_repeated_views_and_restore_are_pure_with_multiple_sources_and_hidden_cards() {
    let (mut g,_,_)=setup();u::field(&mut g,"JC057",0);
    let second=u::field(&mut g,"JC125",2);g.board_mut(&second).unwrap().face_down=true;
    assert_eq!(g.view(0).private_inspections.len(),2);
    assert!(g.view(0).private_inspections.iter().all(|p|p.cards.len()==2));
    let before=serde_json::to_string(&g).unwrap();
    for _ in 0..4 {for seat in 0..4 {g.view(seat);}}
    assert_eq!(serde_json::to_string(&g).unwrap(),before);u::restore(&mut g);
}
#[test]
fn jc057_normal_faceup_deploy_does_not_create_a_reveal_inspection() {
    let mut g=u::game();let held=u::held(&mut g,"JC057",0);u::fund(&mut g,0,"JC057",2);
    u::act(&mut g,0,Action{card_id:Some(held),region:Some(0),..Action::new("deploy")});
    u::top(&mut g);
    assert!(g.pending.is_none());assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC057"));
}
fn reveal_trigger(g: &mut Game) -> String {
    let hidden=u::field(g,"JC057",0);g.board_mut(&hidden).unwrap().face_down=true;u::fund(g,0,"JC057",2);
    u::act(g,0,Action{card_id:Some(hidden),..Action::new("reveal")});
    for _ in 0..32 {if g.pending.is_some(){break;}u::pass(g);}
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"trigger");
    g.regions[0].cards.iter().find(|c|c.definition=="JC057"&&!c.face_down).unwrap().id.clone()
}
fn resolve_inspection(g: &mut Game, target: usize) {
    u::choose(g,vec![player_id(target)]);
    for _ in 0..32 {if g.pending.is_some(){break;}u::pass(g);}
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"jc057-hand-inspect");
}
#[test]
fn jc057_paid_reveal_can_inspect_each_player_and_empty_hands_privately() {
    for target in 0..4 {for empty in [false,true] {
        let mut g=u::game();if !empty{u::held(&mut g,"JC094",target);u::held(&mut g,"JC125",target);}
        reveal_trigger(&mut g);resolve_inspection(&mut g,target);
        let preview=&g.view(0).pending_choice.unwrap().preview_cards;
        assert_eq!(preview.len(),if empty{0}else{2});
        for seat in 1..4{assert!(g.view(seat).pending_choice.is_none());}
        for seat in 0..4{assert!(g.view(seat).revealed_hands.is_empty());}
        for seat in 2..4 {if seat!=target{assert!(!serde_json::to_string(&g.view(seat)).unwrap().contains("JC094"));}}
        assert!(!g.log.iter().any(|l|l.text.contains("革命家阿黛拉")));
        let hands=serde_json::to_value(g.players.iter().map(|p|&p.hand).collect::<Vec<_>>()).unwrap();
        u::choose(&mut g,vec![]);
        assert_eq!(serde_json::to_value(g.players.iter().map(|p|&p.hand).collect::<Vec<_>>()).unwrap(),hands);
        assert!(g.pending.is_none());
    }}
}
#[test]
fn jc057_optional_reveal_may_skip_without_refunding_paid_assets() {
    let mut g=u::game();reveal_trigger(&mut g);u::choose(&mut g,vec![]);
    assert!(g.pending.is_none()&&g.stack.is_empty());assert_eq!(g.players[0].assets.iter().filter(|c|c.exhausted).count(),2);
}
#[test]
fn jc057_reveal_inspects_resolution_time_hand_after_its_source_leaves() {
    let mut g=u::game();let source=reveal_trigger(&mut g);u::choose(&mut g,vec!["p1".into()]);
    u::held(&mut g,"JC094",1);g.remove_dead(&source,RemovalCause::Destroy);
    for _ in 0..32{if g.pending.is_some(){break;}u::pass(&mut g);}
    assert_eq!(g.view(0).pending_choice.as_ref().unwrap().preview_cards.len(),1);
    assert!(g.view(0).private_inspections.is_empty());u::choose(&mut g,vec![]);
}
#[test]
fn jc057_private_confirmation_rejects_other_actor_stale_id_and_extra_payload_atomically() {
    let mut g=u::game();u::held(&mut g,"JC094",1);reveal_trigger(&mut g);resolve_inspection(&mut g,1);
    let id=g.pending.as_ref().unwrap().choice.id.clone();
    let action=Action{choice_id:Some(id.clone()),selected:Some(vec![]),..Action::new("choose")};
    u::reject(&mut g,1,action.clone());
    let mut bad=action.clone();bad.choice_id=Some("stale".into());u::reject(&mut g,0,bad);
    let mut bad=action.clone();bad.selected=Some(vec![g.players[1].hand[0].id.clone()]);u::reject(&mut g,0,bad);
    let mut bad=action;bad.region=Some(0);u::reject(&mut g,0,bad);
    u::choose(&mut g,vec![]);
    u::reject(&mut g,0,Action{choice_id:Some(id),selected:Some(vec![]),..Action::new("choose")});
}
#[test]
fn jc057_restore_rejects_changed_private_hand_and_program_metadata() {
    let mut g=u::game();u::held(&mut g,"JC094",1);reveal_trigger(&mut g);resolve_inspection(&mut g,1);
    for mutation in 0..10 {
        let mut bad=g.clone();let p=bad.pending.as_mut().unwrap();
        if let ChoiceResolution::Frame{frame,choice:FrameChoice::JC057InspectHand{seat,inspected}}=&mut p.resolution {
            match mutation {
                0=>*seat=2,1=>inspected.clear(),2=>frame.cursor=0,3=>frame.guard=GuardState::Unchecked,
                4=>frame.source.region=None,5=>frame.source.card.definition="JC060".into(),
                6=>p.choice.preview_cards.push(g.view(0).pending_choice.unwrap().preview_cards[0].clone()),
                7=>p.choice.allow_decline=Some(true),8=>p.seat=2,_=>frame.targets[0].id="p2".into(),
            }
        } else{panic!("expected inspection");}
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),"mutation{mutation}");
    }
    u::restore(&mut g);
}


// Compiler-bound offline Room84 witness. Explicit layouts, real commands,
// strict restoration, complete journals and four private views per checkpoint.
struct Trace { name: String, room: crate::room::RoomEnvelope, initial: serde_json::Value, steps: Vec<serde_json::Value> }
impl Trace {
    fn new(name: &str, g: &Game) -> Self {
        let room=crate::jc029_tests::envelope(g);
        let initial=serde_json::json!({"state":serde_json::to_string(&room).unwrap(),"views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>()});
        Self{name:name.into(),room,initial,steps:vec![]}
    }
    fn command(&mut self, seat: usize, action: crate::room::SessionAction, accepted: bool) {
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
    fn game(&mut self, seat: usize, action: Action, accepted: bool) {self.command(seat,crate::room::SessionAction::Game{action},accepted);}
    fn responses(&mut self) {
        use crate::room::*;
        for _ in 0..32 {
            let Some(window)=self.room.pacing.window.clone() else{return;};
            let seat=*window.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).expect("response member").0;
            self.command(seat,SessionAction::PassResponse{window_id:window.id},true);
        }
        panic!("bounded responses");
    }
    fn choose(&mut self, selected: Vec<String>) {
        let p=self.room.game.pending.as_ref().unwrap();
        self.game(p.seat,Action{choice_id:Some(p.choice.id.clone()),selected:Some(selected),..Action::new("choose")},true);
    }
    fn finish(self)->serde_json::Value {serde_json::json!({"name":self.name,"initial":self.initial,"steps":self.steps})}
}
#[test]
fn jc057_native84_oracles() {
    use crate::room::SessionAction;
    let mut scenarios=vec![];
    for target in 0..4 {
        let mut g=u::game();g.room_id=format!("84000000000000000000000{target}");
        let source=u::field(&mut g,"JC057",0);g.board_mut(&source).unwrap().face_down=true;
        u::fund(&mut g,0,"JC057",2);u::held(&mut g,"JC094",target);u::held(&mut g,"JC125",target);
        let hidden=u::field(&mut g,"JC052",1);g.board_mut(&hidden).unwrap().face_down=true;
        let mut t=Trace::new(&format!("reveal-p{target}"),&g);
        let reveal=Action{card_id:Some(source),..Action::new("reveal")};
        t.game(1,reveal.clone(),false);t.game(0,reveal,true);t.responses();
        assert_eq!(t.room.game.pending.as_ref().unwrap().choice.kind,"trigger");
        assert_eq!(t.room.game.view(0).private_inspections.len(),1);
        t.choose(vec![player_id(target)]);t.responses();
        assert_eq!(t.room.game.pending.as_ref().unwrap().choice.kind,"jc057-hand-inspect");
        assert_eq!(t.room.game.view(0).pending_choice.unwrap().preview_cards.len(),2);
        let id=t.room.game.pending.as_ref().unwrap().choice.id.clone();
        let confirm=Action{choice_id:Some(id.clone()),selected:Some(vec![]),..Action::new("choose")};
        t.game(1,confirm.clone(),false);
        t.command(0,SessionAction::PauseRoom,true);t.game(0,confirm.clone(),false);
        t.command(0,SessionAction::ResumeRoom,true);
        let mut malformed=confirm.clone();malformed.region=Some(0);t.game(0,malformed,false);
        t.game(0,confirm.clone(),true);t.game(0,confirm,false);
        assert!(t.room.game.pending.is_none());scenarios.push(t.finish());
    }
    for (mode_index,mode) in ["empty","skip","deploy"].into_iter().enumerate() {
        let mut g=u::game();g.room_id=format!("{:024x}",840100+mode_index);u::fund(&mut g,0,"JC057",2);
        let mut action=Action::new(if mode=="deploy"{"deploy"}else{"reveal"});
        if mode=="deploy"{action.card_id=Some(u::held(&mut g,"JC057",0));action.region=Some(0);}
        else {let id=u::field(&mut g,"JC057",0);g.board_mut(&id).unwrap().face_down=true;action.card_id=Some(id);}
        let mut t=Trace::new(mode,&g);t.game(0,action,true);t.responses();
        if mode!="deploy"{t.choose(if mode=="skip"{vec![]}else{vec!["p1".into()]});t.responses();}
        if mode=="empty"{assert!(t.room.game.view(0).pending_choice.unwrap().preview_cards.is_empty());t.choose(vec![]);}
        assert!(t.room.game.pending.is_none());assert_eq!(t.room.game.players[0].assets.iter().filter(|c|c.exhausted).count(),2);
        scenarios.push(t.finish());
    }
    for (mode_index,(name,spell,fund,source_seat,option)) in [
        ("live-exhaust","XQ03",2,0,None),
        ("live-hide","JC063",2,0,Some("hide")),
        ("live-control","JC129",3,1,None),
    ].into_iter().enumerate() {
        let mut g=u::game();g.room_id=format!("{:024x}",840200+mode_index);
        let source=u::field(&mut g,"JC057",source_seat);let hidden=u::field(&mut g,"JC052",2);g.board_mut(&hidden).unwrap().face_down=true;
        let held=u::held(&mut g,spell,0);u::fund(&mut g,0,if name=="live-control"{"JC104"}else{spell},fund);
        assert_eq!(g.view(source_seat).private_inspections.len(),1);
        let mut t=Trace::new(name,&g);
        t.game(0,Action{card_id:Some(held),target_id:Some(source),option:option.map(Into::into),..Action::new("play")},true);t.responses();
        assert!(t.room.game.view(source_seat).private_inspections.is_empty());
        assert_eq!(t.room.game.view(0).private_inspections.len(),usize::from(name=="live-control"));
        scenarios.push(t.finish());
    }
    if let Ok(directory)=std::env::var("JC057_NATIVE_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        let output=serde_json::json!({"format":"jc057-native84-v1","scope":"Explicit offline Native layouts and real Room commands; complete opaque states, journals and four views; no online gameplay claim","catalog":catalog::catalog(),"sourceBindings":source_bindings(),"scenarios":scenarios});
        use std::io::Write;
        std::fs::OpenOptions::new().create_new(true).write(true).open(std::path::Path::new(&directory).join("native84.json")).unwrap().write_all(&serde_json::to_vec(&output).unwrap()).unwrap();
    }
}
#[test]
fn jc057_eliminated_target_before_resolution_does_not_create_private_hand_choice() {
    let mut g=u::game();u::held(&mut g,"JC094",1);reveal_trigger(&mut g);u::choose(&mut g,vec!["p1".into()]);
    g.players[1].eliminated=true;
    for _ in 0..32 {if g.stack.is_empty(){break;}u::pass(&mut g);}
    assert!(g.pending.is_none());
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
