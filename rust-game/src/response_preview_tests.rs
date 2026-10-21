//! Read-only response source projection. All layouts are explicit native
//! fixtures followed by real Room commands; no natural online play is claimed.
use crate::{catalog,model::*,room::*,unit_support as u};
use serde_json::{json,Value};
use std::collections::BTreeSet;

fn layout() -> (RoomEnvelope,String,String,String) {
    let mut g=u::game();g.room_id="820000000000000000000000".into();
    let worm=u::field(&mut g,"JC054",0);
    for seat in [2,3] {u::field(&mut g,"JC125",seat);}
    let enemy=u::field(&mut g,"JC125",2);g.board_mut(&enemy).unwrap().face_down=true;
    let hidden=g.hide_in_place_successor(&worm).unwrap().id;g.drive().unwrap();
    for seat in [0,1,2] {u::fund(&mut g,seat,"JC091",4);u::held(&mut g,"JC091",seat);}
    let spell=g.players[0].hand[0].id.clone();
    (crate::jc029_tests::envelope(&g),hidden,spell,enemy)
}
fn encode(room:&RoomEnvelope)->String {serde_json::to_string(room).unwrap()}
fn restore(room:&RoomEnvelope) {let raw=encode(room);assert_eq!(encode(&RoomEnvelope::from_persisted(&raw).unwrap()),raw);}
fn sources(room:&RoomEnvelope,seat:usize,now:u64)->Vec<ResponseSourceHint> {room.view(seat,now).response_window.unwrap().preview_sources}
fn step(room:&mut RoomEnvelope,seat:usize,action:SessionAction)->RoomTransition {
    let result=room.transition(seat,Some(RoomCommand {command_id:format!("preview-test-{}",room.revision),expected_version:room.revision,action}),0).unwrap();
    assert!(result.error_code.is_none(),"{:?}",result.error_message);
    *room=RoomEnvelope::from_persisted(&result.state).unwrap();result
}

#[test]
fn response_preview_is_pure_deduplicated_engine_sources_without_private_targets() {
    let (room,hidden,spell,enemy)=layout();let before=encode(&room);
    for _ in 0..3 {for seat in 0..4 {
        let view=room.view(seat,0);let response=view.response_window.as_ref().unwrap();
        assert!(view.legal_actions.iter().all(|a|a.action.kind=="pass"));assert!(response.my_intent_id.is_none());
        let hints=&response.preview_sources;
        assert_eq!(hints.len(),hints.iter().map(|h|(h.kind.clone(),h.card_id.clone(),h.ability_id.clone())).collect::<BTreeSet<_>>().len());
        for hint in hints {
            let value=serde_json::to_value(hint).unwrap();
            assert!(value.as_object().unwrap().keys().all(|k|matches!(k.as_str(),"kind"|"cardId"|"abilityId")));
            assert_ne!(hint.card_id.as_deref(),Some(enemy.as_str()));
            assert!(room.game.legal_actions(seat).iter().any(|a|a.action.kind==hint.kind&&a.action.card_id==hint.card_id&&a.action.ability_id==hint.ability_id));
        }
        if seat==0 {
            assert!(hints.iter().any(|h|h.kind=="reveal"&&h.card_id.as_ref()==Some(&hidden)));
            assert!(hints.iter().any(|h|h.kind=="play"&&h.card_id.as_ref()==Some(&spell)));
            assert!(room.game.legal_actions(0).iter().filter(|a|a.action.card_id.as_ref()==Some(&spell)).count()>1);
        } else {assert!(!hints.iter().any(|h|h.card_id.as_ref()==Some(&hidden)||h.card_id.as_ref()==Some(&spell)));}
        if seat>=2 {assert!(hints.is_empty());}
    }}
    assert_eq!(encode(&room),before,"view must not allocate IDs, change deadlines, exhaust assets or create intents");restore(&room);
}

#[test]
fn response_preview_respects_pause_composing_pass_and_expiry_without_driving_time() {
    let (mut room,_,_,_)=layout();let original=encode(&room);
    assert!(sources(&room,0,INTENT_DURATION_MS).is_empty());assert_eq!(encode(&room),original);
    let expired=room.transition(0,None,INTENT_DURATION_MS).unwrap();assert!(expired.changed);assert!(matches!(expired.journal.as_slice(),[SessionEvent::Tick {..}]));
    step(&mut room,0,SessionAction::PauseRoom);assert!(sources(&room,0,0).is_empty());
    step(&mut room,0,SessionAction::ResumeRoom);assert!(!sources(&room,0,0).is_empty());
    let window=room.pacing.window.as_ref().unwrap().id.clone();
    step(&mut room,0,SessionAction::BeginResponse {window_id:window.clone(),intent_id:"real-intent".into()});
    assert!(sources(&room,0,0).is_empty());assert!(room.view(0,0).legal_actions.iter().any(|a|a.action.kind=="play"));
    step(&mut room,1,SessionAction::PassResponse {window_id:window.clone()});assert!(sources(&room,1,0).is_empty());
    step(&mut room,0,SessionAction::CancelAndPass {window_id:window,intent_id:"real-intent".into()});assert!(sources(&room,0,0).is_empty());restore(&room);
}

#[test]
fn response_preview_does_not_relax_quote_or_submit_guards_or_pay_costs() {
    let (mut room,hidden,_,_)=layout();let window=room.pacing.window.as_ref().unwrap().id.clone();let before=encode(&room);
    assert!(room.quote(0,QuoteRequest {window_id:window.clone(),intent_id:"not-started".into(),draft:None}).is_err());
    let action=Action {card_id:Some(hidden),..Action::new("reveal")};
    let rejected=room.transition(0,Some(RoomCommand {command_id:"cannot-submit-preview".into(),expected_version:room.revision,
        action:SessionAction::SubmitResponse {window_id:window.clone(),intent_id:"not-started".into(),action:action.clone()}}),0).unwrap();
    assert_eq!(rejected.error_code.as_deref(),Some("window_expired"));assert!(!rejected.changed&&rejected.journal.is_empty());assert_eq!(rejected.state,before);
    assert!(room.game.players[0].assets.iter().all(|c|!c.exhausted));
    step(&mut room,0,SessionAction::BeginResponse {window_id:window.clone(),intent_id:"real-intent".into()});
    let quoted=encode(&room);assert!(room.quote(0,QuoteRequest {window_id:window.clone(),intent_id:"real-intent".into(),draft:Some(action.clone())}).unwrap().ready);
    assert_eq!(encode(&room),quoted);
    step(&mut room,0,SessionAction::SubmitResponse {window_id:window,intent_id:"real-intent".into(),action});
    assert_eq!(room.game.players[0].assets.iter().filter(|c|c.exhausted).count(),4);restore(&room);
}

struct Trace {name:&'static str,room:RoomEnvelope,initial:Value,steps:Vec<Value>}
impl Trace {
    fn new(name:&'static str)->Self {let (room,_,_,_)=layout();let initial=json!({"state":encode(&room),"views":(0..4).map(|s|room.view(s,0)).collect::<Vec<_>>()});Self {name,room,initial,steps:vec![]}}
    fn views(&self)->Vec<RoomView> {(0..4).map(|s|self.room.view(s,self.room.pacing.last_server_now_ms)).collect()}
    fn query(&mut self) {let state=encode(&self.room);let views=self.views();assert_eq!(state,encode(&self.room));self.steps.push(json!({"kind":"view","state":state,"views":views}));}
    fn transition(&mut self,seat:usize,action:Option<SessionAction>,now:u64) {
        let state=encode(&self.room);let command=action.map(|action|RoomCommand {command_id:format!("preview-{}-{}",self.name,self.steps.len()),expected_version:self.room.revision,action});
        let expected=self.room.transition(seat,command.clone(),now).unwrap();
        if expected.changed {assert_eq!(encode(&self.room.replay_events(&expected.journal).unwrap()),expected.state);} else {assert_eq!(expected.state,state);}
        self.room=RoomEnvelope::from_persisted(&expected.state).unwrap();let views=self.views();
        self.steps.push(json!({"kind":if command.is_some(){"command"}else{"poll"},"state":state,"seat":seat,"command":command,"serverNowMs":now.to_string(),"expected":expected,"views":views}));
    }
    fn quote(&mut self,intent:&str) {let state=encode(&self.room);let request=QuoteRequest {window_id:self.room.pacing.window.as_ref().unwrap().id.clone(),intent_id:intent.into(),draft:None};
        let result=self.room.quote(0,request.clone());assert_eq!(encode(&self.room),state);
        self.steps.push(json!({"kind":"quote","state":state,"seat":0,"request":request,"expected":result.as_ref().ok(),"error":result.err()}));}
    fn finish(self)->Value {json!({"name":self.name,"initial":self.initial,"steps":self.steps})}
}

#[test]
fn response_preview_native82_oracles() {
    let mut a=Trace::new("preview-then-real-reveal");a.query();a.query();a.quote("not-started");
    let hidden=a.room.game.regions[0].cards.iter().find(|c|c.definition=="JC054").unwrap().id.clone();let w=a.room.pacing.window.as_ref().unwrap().id.clone();
    a.transition(0,Some(SessionAction::SubmitResponse {window_id:w.clone(),intent_id:"not-started".into(),action:Action {card_id:Some(hidden.clone()),..Action::new("reveal")}}),0);
    a.transition(0,Some(SessionAction::BeginResponse {window_id:w.clone(),intent_id:"intent-a".into()}),0);a.query();a.quote("intent-a");
    a.transition(0,Some(SessionAction::SubmitResponse {window_id:w,intent_id:"intent-a".into(),action:Action {card_id:Some(hidden),..Action::new("reveal")}}),0);a.query();
    let mut b=Trace::new("preview-pause-expiry");b.query();b.transition(0,Some(SessionAction::PauseRoom),0);b.query();b.transition(0,Some(SessionAction::ResumeRoom),0);
    b.transition(0,None,4999);b.query();b.transition(0,None,5000);b.query();
    let mut c=Trace::new("preview-peer-pass-cancel");c.query();let w=c.room.pacing.window.as_ref().unwrap().id.clone();
    c.transition(1,Some(SessionAction::PassResponse {window_id:w.clone()}),0);c.query();
    c.transition(0,Some(SessionAction::BeginResponse {window_id:w.clone(),intent_id:"intent-c".into()}),0);c.query();
    c.transition(0,Some(SessionAction::CancelAndPass {window_id:w,intent_id:"intent-c".into()}),0);c.query();
    if let Ok(directory)=std::env::var("RESPONSE_PREVIEW_NATIVE_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        let output=json!({"format":"response-preview-native82-v1","scope":"Explicit native layouts, pure view/quote observations and real Room commands; not online gameplay","catalog":catalog::catalog(),"sourceBindings":source_bindings(),"scenarios":[a.finish(),b.finish(),c.finish()]});
        use std::io::Write;std::fs::OpenOptions::new().create_new(true).write(true).open(std::path::Path::new(&directory).join("native82.json")).unwrap().write_all(&serde_json::to_vec(&output).unwrap()).unwrap();
    }
}

fn source_bindings() -> Value {
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
    ];
    json!({"algorithm":"sha256","files":files.iter().map(|(path,bytes)| ((*path).to_owned(),format!("{:x}",Sha256::digest(bytes)))).collect::<std::collections::BTreeMap<_,_>>()})
}
