//! XQ23 full printed action, actual response windows and restore rejection.
use crate::{catalog, model::*, rules::*, unit_support as u, xq23};
use serde_json::Value;
thread_local! { static BAD_EXPORT: std::cell::RefCell<Option<Vec<Value>>> = const { std::cell::RefCell::new(None) }; }
fn active(actor: usize) -> (Game, String, String) {
    let mut g=u::game();g.first_team=g.team(actor);g.begin_window(Window::Action(g.team(actor)));
    let source=u::field(&mut g,"XQ23",actor);u::fund(&mut g,actor,"JC060",3);
    let target=g.players[actor].assets.last().unwrap().id.clone();(g,source,target)
}
fn action(source: &str, target: &str) -> Action {
    Action {card_id:Some(source.into()),target_id:Some(target.into()),ability_id:Some(xq23::KEY.into()),..Action::new("activate")}
}
fn bad(g: &Game, name: &str) {
    let mut g=g.clone();BAD_EXPORT.with(|saved|if let Some(rows)=saved.borrow().as_ref(){g.room_id=format!("{:024x}",889000+rows.len())});
    let game_error=Game::from_persisted(&serde_json::to_string(&g).unwrap()).err().unwrap_or_else(||panic!("Game accepted {name}"));
    let mut clean=g.clone();clean.stack.clear();clean.effects.clear();clean.pending=None;
    // Only the temporary valid envelope scaffold is sanitized. The complete
    // original corrupted Game is restored below before either loader sees it.
    let seats=clean.players.len();
    for (seat,p) in clean.players.iter_mut().enumerate() {
        for c in &mut p.assets {if c.owner>=seats {c.owner=seat;}}
    }
    let mut room=crate::jc029_tests::envelope(&clean);room.game=g.clone();
    let state=serde_json::to_string(&room).unwrap();
    let room_error=crate::room::RoomEnvelope::from_persisted(&state).err().unwrap_or_else(||panic!("Room accepted {name}"));
    BAD_EXPORT.with(|saved|if let Some(rows)=saved.borrow_mut().as_mut(){rows.push(serde_json::json!({"name":name,"state":state,"gameError":game_error,"roomError":room_error,"roomId":room.game.room_id,"inviteCode":room.game.invite_code,"revision":room.revision}))});
}
pub(crate) fn source_bindings() -> Value {
    let mut binding=crate::jc066_tests::source_bindings();
    use sha2::{Digest,Sha256};
    for(name,bytes)in[
        ("rust-game/src/jc041.rs",include_bytes!("jc041.rs").as_slice()),
        ("rust-game/src/jc041_tests.rs",include_bytes!("jc041_tests.rs").as_slice()),
        ("rust-game/src/xq23.rs",include_bytes!("xq23.rs").as_slice()),
        ("rust-game/src/xq23_tests.rs",include_bytes!("xq23_tests.rs").as_slice()),
    ] {binding["files"].as_object_mut().unwrap().insert(name.into(),serde_json::json!(format!("{:x}",Sha256::digest(bytes))));}
    assert_eq!(binding["files"].as_object().unwrap().len(),139);
    binding
}

#[test]
fn xq23_whole_print_and_definition_cannot_be_weakened_or_transplanted() {
    let c=catalog::card("XQ23");assert_eq!(c.cost,1);assert_eq!(c.loyalty,vec!["灰色"]);assert_eq!(c.color,"灰");
    assert_eq!(c.subtypes,vec!["人类","文员"]);assert_eq!(c.defense,Some(1));assert_eq!(c.permanent_icons.influence,1);
    assert_eq!(c.magic_icon,MagicIcon::None);assert!(!c.unique);
    let d=xq23::definition();assert!(d.traits.public&&d.traits.retreat);assert_eq!(d.abilities.len(),1);
    assert!(crate::rules::validate_ability("XQ23",&d.abilities[0]).is_ok());
    assert!(crate::rules::validate_ability("JC029",&d.abilities[0]).is_err());
    for variant in 0..11 {
        let mut d=d.clone();match variant {
            0=>d.abilities[0].costs=vec![Cost::Assets(1),Cost::ExhaustSource],
            1=>d.abilities[0].costs=vec![Cost::Assets(2)],2=>d.abilities[0].timing=Timing::Standard,
            3=>d.abilities[0].activation_only=false,4=>d.abilities[0].event=Some(Event::Enter),
            5=>d.abilities[0].per_turn_limit=Some(1),6=>d.abilities[0].targets[0].relation=Relation::Any,
            7=>d.abilities[0].targets[0].predicate=None,8=>d.traits.public=false,
            9=>d.traits.retreat=false,_=>d.abilities[0].ops=vec![Op::ForEachLivingPlayer(vec![Op::XQ23ReturnAssetThenBecomeAsset])],
        }
        assert!(crate::rules::validate_definitions(&std::collections::BTreeMap::from([("XQ23".into(),d)])).is_err(),"variant {variant}");
    }
}

#[test]
fn xq23_current89_regression_oracles() {
    use crate::room::SessionAction;
    let mut scenarios=vec![];
    for actor in 0..4 {
        let (mut g,s,t)=active(actor);g.room_id=format!("{:024x}",880000+actor);
        {let c=g.board_mut(&s).unwrap();c.owner=(actor+1)%4;c.time_markers=2;c.lock_markers=1;}
        let mut trace=Trace::new(&format!("actor-{actor}"),&g);
        trace.game(actor,action(&s,&t),true);
        trace.command(0,SessionAction::PauseRoom,true);trace.game(actor,action(&s,&t),false);
        trace.command(3,SessionAction::ResumeRoom,true);trace.responses();
        let c=trace.room.game.players[actor].assets.iter().find(|c|c.definition=="XQ23").unwrap();
        assert!(c.exhausted);assert_eq!(c.controller,actor);assert_eq!(c.owner,(actor+1)%4);
        assert_eq!((c.time_markers,c.lock_markers),(0,0));assert!(trace.room.game.board(&s).is_none());
        scenarios.push(trace.finish());
    }
    {
        let mut g=u::game();g.room_id=format!("{:024x}",880010);u::fund(&mut g,0,"JC060",1);let s=u::held(&mut g,"XQ23",0);
        let mut trace=Trace::new("ordinary-deploy",&g);
        trace.game(0,Action{card_id:Some(s.clone()),region:Some(0),..Action::new("conceal")},false);
        trace.game(0,Action{card_id:Some(s),region:Some(0),..Action::new("deploy")},true);trace.responses();
        assert!(trace.room.game.regions[0].cards.iter().any(|c|c.definition=="XQ23"&&!c.face_down));scenarios.push(trace.finish());
    }
    {
        let (mut g,s,t)=active(0);g.room_id=format!("{:024x}",880011);u::fund(&mut g,0,"JC060",2);let second=u::field(&mut g,"XQ23",0);
        let mut trace=Trace::new("target-return-response",&g);trace.game(0,action(&s,&t),true);trace.respond(0,action(&second,&t));trace.responses();
        assert!(trace.room.game.board(&s).unwrap().1.exhausted);assert!(trace.room.game.board(&second).is_none());
        assert_eq!(trace.room.game.players[0].hand.len(),1);scenarios.push(trace.finish());
    }
    {
        let (mut g,s,t)=active(0);g.room_id=format!("{:024x}",880012);let other=u::field(&mut g,"XQ23",0);u::field(&mut g,"JC060",0);
        let spell=u::held(&mut g,"JC006",2);u::fund(&mut g,2,"JC006",3);
        let mut trace=Trace::new("source-return-response",&g);trace.game(0,action(&s,&t),true);
        trace.respond(2,Action{card_id:Some(spell),target_id:Some(s.clone()),..Action::new("play")});trace.responses();
        assert!(trace.room.game.board(&s).is_none());assert!(trace.room.game.board(&other).is_some());
        assert!(trace.room.game.players[0].assets.iter().all(|c|c.definition!="XQ23"));scenarios.push(trace.finish());
    }
    for paid in [false,true] {
        let (mut g,s,mut t)=active(0);g.room_id=format!("{:024x}",880020+usize::from(paid));
        if paid {t=g.players[0].assets[0].id.clone();} else {g.players[0].assets.last_mut().unwrap().exhausted=true;}
        let mut trace=Trace::new(if paid{"paid-target"}else{"exhausted-target"},&g);trace.game(0,action(&s,&t),true);trace.responses();
        assert!(trace.room.game.players[0].hand.iter().any(|c|c.definition=="JC060"));scenarios.push(trace.finish());
    }
    BAD_EXPORT.with(|saved|*saved.borrow_mut()=Some(vec![]));
    xq23_bad_paid_frames_targets_sources_and_response_shells_reject_in_game_and_room();
    let invalid=BAD_EXPORT.with(|saved|saved.borrow_mut().take().unwrap());assert_eq!(invalid.len(),31);
    let binding=crate::jc035_tests::source_bindings();
    let commands=scenarios.iter().map(|s|s["steps"].as_array().unwrap().len()).sum::<usize>();
    let rejected=scenarios.iter().flat_map(|s|s["steps"].as_array().unwrap()).filter(|r|!r["expected"]["errorCode"].is_null()).count();
    let counts=serde_json::json!({"scenarios":scenarios.len(),"commands":commands,"rejected":rejected,"projections":4*(commands+scenarios.len()),"invalidRestores":invalid.len()});
    let value=serde_json::json!({"format":"xq23-current-native89-v1","catalog":catalog::catalog(),"sourceBindings":binding,"scenarios":scenarios,"invalid":invalid,"counts":counts});
    if let Ok(dir)=std::env::var("XQ23_CURRENT_CORE_NATIVE_DIR") {
        use std::io::Write;std::fs::create_dir_all(&dir).unwrap();
        std::fs::OpenOptions::new().create_new(true).write(true).open(std::path::Path::new(&dir).join("NativeCurrent89.json")).unwrap().write_all(&serde_json::to_vec(&value).unwrap()).unwrap();
    }
    println!("XQ23_CURRENT89_COUNTS {}",counts);
}
#[test]
fn xq23_all_four_seats_pay_and_exhaust_before_response_then_return_asset_and_reset_original() {
    for actor in 0..4 {
        let (mut g,source,target)=active(actor);
        let ordinary_asset_already_used=actor%2==1;
        g.players[actor].asset_used=ordinary_asset_already_used;
        {let c=g.board_mut(&source).unwrap();c.time_markers=2;c.lock_markers=1;c.wounds=0;c.shield=2;}
        u::act(&mut g,actor,action(&source,&target));assert!(g.board(&source).unwrap().1.exhausted);
        assert_eq!(u::resources(&g,actor),1);assert!(g.players[actor].assets.iter().any(|c|c.id==target));
        let f=g.stack.last().unwrap().frame.as_ref().unwrap();assert_eq!(f.ability_key,xq23::KEY);assert_eq!(f.already_paid.len(),2);
        assert!(!f.source.card.exhausted);assert_eq!(f.source.card.id,source);assert!(g.pending.is_none());
        u::top(&mut g);assert!(g.board(&source).is_none());assert!(g.players[actor].assets.iter().all(|c|c.id!=target));
        let clerk=g.players[actor].assets.iter().find(|c|c.definition=="XQ23").unwrap();
        assert_ne!(clerk.id,source);assert!(clerk.exhausted);assert_eq!(clerk.controller,actor);assert_eq!(clerk.owner,actor);
        assert_eq!((clerk.time_markers,clerk.lock_markers,clerk.damage,clerk.wounds,clerk.shield),(0,0,0,0,0));
        assert!(g.players[actor].hand.iter().any(|c|c.definition=="JC060"&&c.id!=target&&!c.exhausted));
        assert_eq!(u::resources(&g,actor),0);assert!(g.pending.is_none());
        assert_eq!(g.players[actor].asset_used,ordinary_asset_already_used,"Card conversion preserves the ordinary establishment quota");
        let asset_id=clerk.id.clone();
        for viewer in 0..4 {
            let v=g.view(viewer);
            let shown=v.assets.iter().find(|c|c.instance_id==asset_id).unwrap();
            assert_eq!(shown.kind,"asset");assert!(shown.exhausted);assert!(shown.card_id.is_none());
            assert!(shown.text.is_none()&&shown.icons.is_none()&&shown.defense.is_none());
        }
    }
}
#[test]
fn xq23_can_return_an_exhausted_asset_including_one_just_used_for_payment() {
    let (mut g,s,_)=active(0);let target=g.players[0].assets[0].id.clone();
    u::act(&mut g,0,action(&s,&target));assert!(g.players[0].assets[0].exhausted);
    u::top(&mut g);assert!(g.players[0].hand.iter().any(|c|c.definition=="JC060"));
    assert_eq!(g.players[0].assets.iter().filter(|c|c.exhausted).count(),2);assert_eq!(u::resources(&g,0),1);
    let (mut g,s,t)=active(0);g.players[0].assets.last_mut().unwrap().exhausted=true;
    u::act(&mut g,0,action(&s,&t));u::top(&mut g);assert!(g.players[0].hand.iter().any(|c|c.definition=="JC060"));
}
#[test]
fn xq23_return_uses_asset_owner_and_source_becomes_frozen_actor_asset() {
    let (mut g,s,t)=active(0);g.players[0].assets.last_mut().unwrap().owner=3;g.board_mut(&s).unwrap().owner=2;
    u::act(&mut g,0,action(&s,&t));u::top(&mut g);
    assert!(g.players[3].hand.iter().any(|c|c.definition=="JC060"));assert!(g.players[0].hand.is_empty());
    let c=g.players[0].assets.iter().find(|c|c.definition=="XQ23").unwrap();assert_eq!((c.owner,c.controller,c.exhausted),(2,0,true));
}
#[test]
fn xq23_rejects_foreign_assets_nonassets_unpaid_or_exhausted_sources_atomically() {
    let (g,s,t)=active(0);
    for seat in 1..4 {
        let mut next=g.clone();u::fund(&mut next,seat,"JC125",1);let foreign=next.players[seat].assets[0].id.clone();
        u::reject(&mut next,0,action(&s,&foreign));
    }
    for id in [s.clone(),g.regions[0].card.id.clone(),"missing".into()] {u::reject(&mut g.clone(),0,action(&s,&id));}
    let mut next=g.clone();let hand=u::held(&mut next,"JC060",0);u::reject(&mut next,0,action(&s,&hand));
    let mut next=g.clone();next.board_mut(&s).unwrap().exhausted=true;u::reject(&mut next,0,action(&s,&t));
    let mut next=g.clone();next.players[0].assets.truncate(1);let own=next.players[0].assets[0].id.clone();u::reject(&mut next,0,action(&s,&own));
    let mut next=g.clone();let mut a=action(&s,&t);a.kind="play".into();u::reject(&mut next,0,a);
}
#[test]
fn xq23_target_returned_by_second_real_fast_action_fizzles_first_without_refund() {
    let (mut g,s,t)=active(0);u::fund(&mut g,0,"JC060",2);let response=u::field(&mut g,"XQ23",0);
    u::act(&mut g,0,action(&s,&t));u::pass(&mut g);u::act(&mut g,0,action(&response,&t));
    u::top(&mut g);assert!(g.players[0].assets.iter().all(|c|c.id!=t));u::top(&mut g);
    assert!(g.board(&s).unwrap().1.exhausted);assert_eq!(u::resources(&g,0),1);assert_eq!(g.players[0].hand.len(),1);
    assert!(g.board(&response).is_none());assert_eq!(g.players[0].assets.iter().filter(|c|c.definition=="XQ23").count(),1);
}
#[test]
fn xq23_source_returned_by_real_response_does_not_resurrect_or_move_same_name_replacement() {
    let (mut g,s,t)=active(0);let replacement=u::field(&mut g,"XQ23",0);
    u::field(&mut g,"JC060",0); // The real adviser grants Mind, making the real JC006 target legal.
    let response=u::held(&mut g,"JC006",1);u::fund(&mut g,1,"JC006",8);
    u::act(&mut g,0,action(&s,&t));u::act(&mut g,1,Action {card_id:Some(response),target_id:Some(s.clone()),..Action::new("play")});
    u::top(&mut g);assert!(g.board(&s).is_none());u::top(&mut g);
    assert!(g.board(&replacement).is_some());assert!(!g.board(&replacement).unwrap().1.exhausted);
    assert!(g.players[0].assets.iter().all(|c|c.definition!="XQ23"));
    assert!(g.players[0].hand.iter().any(|c|c.definition=="XQ23"));assert!(g.players[0].hand.iter().any(|c|c.definition=="JC060"));
}
#[test]
fn xq23_normal_paid_deploy_and_public_conceal_rejection_roundtrip() {
    let mut g=u::game();u::fund(&mut g,0,"JC060",1);let id=u::held(&mut g,"XQ23",0);
    u::reject(&mut g,0,Action {card_id:Some(id.clone()),region:Some(0),..Action::new("conceal")});
    u::act(&mut g,0,Action {card_id:Some(id),region:Some(0),..Action::new("deploy")});u::top(&mut g);
    let c=g.regions[0].cards.iter().find(|c|c.definition=="XQ23").unwrap();assert!(!c.exhausted&&!c.face_down);assert!(g.pending.is_none());
}
#[test]
fn xq23_printed_retreat_uses_shared_region_return_without_death() {
    let (mut g,s,_)=active(0);let grave=g.players[0].graveyard.len();
    g.effect(Effect::Bottom{seat:0,region:0}).unwrap();u::restore(&mut g);
    assert!(g.board(&s).is_none());assert!(g.players[0].hand.iter().any(|c|c.definition=="XQ23"&&c.id!=s));
    assert_eq!(g.players[0].graveyard.len(),grave);
}
#[test]
fn xq23_rejects_unprinted_action_payload_fields_before_payment() {
    let (g,s,t)=active(0);
    for n in 0..6 {let mut a=action(&s,&t);match n {
        0=>a.region=Some(0),1=>a.option=Some("extra".into()),2=>a.cost_selected=Some(vec![]),
        3=>a.selected=Some(vec![]),4=>a.choice_id=Some("extra".into()),_=>a.allocations=Some(std::collections::BTreeMap::new())
    };u::reject(&mut g.clone(),0,a);}
}
#[test]
fn xq23_exact_review_paid_state_rejects_mismatched_summary_uid_and_nonasset_kind() {
    use sha2::{Digest,Sha256};
    let raw=std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"),"/../docs/evidence/xq23-p2-2026-10-19/paid-room-before.json")).unwrap();
    assert_eq!(raw.len(),31_197);
    assert_eq!(format!("{:x}",Sha256::digest(raw.as_bytes())),"76a47a29fb9d3b1d772ffdb6087e9180cac93f91fdd0c3af0d989fdc36b48c65");
    let original:Value=serde_json::from_str(&raw).unwrap();
    // The unchanged published88 control belongs only to the exact frozen88
    // reducer. Never alter its version fields, UIDs or source bindings for89.
    assert!(crate::room::RoomEnvelope::from_persisted(&raw).unwrap_err().contains("版本"));
    assert!(Game::from_persisted(&serde_json::to_string(&original["game"]).unwrap()).unwrap_err().contains("版本"));
    let (mut g,source,target)=active(0);u::act(&mut g,0,action(&source,&target));
    let current=serde_json::to_value(crate::jc029_tests::envelope(&g)).unwrap();
    let fresh=serde_json::to_string(&current).unwrap();
    crate::room::RoomEnvelope::from_persisted(&fresh).unwrap();
    Game::from_persisted(&serde_json::to_string(&current["game"]).unwrap()).unwrap();
    let other=g.players[0].assets.iter().find(|c|c.id!=target).unwrap().id.as_str();
    for(field,value)in[("instanceId",other),("instanceId",""),("kind","hidden"),("kind","character")] {
        let mut changed=current.clone();changed["game"]["stack"][0]["frame"]["targets"][0]["public"][field]=value.into();
        assert_eq!(changed["game"]["stack"][0]["target"],target);
        assert_eq!(changed["game"]["stack"][0]["frame"]["targets"][0]["id"],target);
        let game_error=Game::from_persisted(&serde_json::to_string(&changed["game"]).unwrap()).unwrap_err();
        let room_error=crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&changed).unwrap()).unwrap_err();
        assert!(game_error.contains("XQ23"));assert_eq!(game_error,room_error);
    }
}
#[test]
fn xq23_restore_after_target_control_transfer_keeps_uid_and_fizzles_without_refund() {
    let(mut g,source,target)=active(0);u::act(&mut g,0,action(&source,&target));
    // Explicit response-time control/zone fixture; no claim that a standard
    // JC129 spell may be cast during a response window.
    let index=g.players[0].assets.iter().position(|c|c.id==target).unwrap();
    let mut asset=g.players[0].assets.remove(index);asset.controller=2;g.players[2].assets.push(asset);
    u::restore(&mut g);crate::jc029_tests::checkpoint(&g);
    let summary=&g.view(0).stack[0].target_summaries[0];
    assert_eq!(summary.instance_id,target);assert_eq!(summary.controller.as_deref(),Some("p2"));assert!(!summary.valid);
    assert_eq!(g.stack[0].frame.as_ref().unwrap().targets[0].public.controller.as_deref(),Some("p0"));
    u::top(&mut g);assert!(g.board(&source).unwrap().1.exhausted);
    assert!(g.players[2].assets.iter().any(|c|c.id==target&&c.controller==2));
    assert!(g.players[0].hand.is_empty());assert!(g.players[0].assets.iter().all(|c|c.exhausted));
}
#[test]
fn xq23_restore_after_source_control_change_retains_the_paid_actor_asset_destination() {
    let(mut g,source,target)=active(0);u::act(&mut g,0,action(&source,&target));
    g.add_control(&source,2,ControlLifetime::TurnEnd{turn:g.turn},SubtypeChange::None);
    assert_eq!(g.board(&source).unwrap().1.controller,2);
    u::restore(&mut g);crate::jc029_tests::checkpoint(&g);u::top(&mut g);
    assert!(g.board(&source).is_none());
    assert!(g.players[0].assets.iter().any(|c|c.definition=="XQ23"&&c.controller==0&&c.owner==0&&c.exhausted));
    assert!(g.players[0].hand.iter().any(|c|c.definition=="JC060"));
}
#[test]
fn xq23_real_borrowed_assets_follow_owner_elimination_across_seats_and_preserve_team_score() {
    // Start from the same explicit initial sequence as the independently run
    // pre-fix Native probe. All subsequent changes use ordinary Room commands.
    let(mut g,_,_)=active(0);g.room_id=format!("{:024x}",880031);
    for p in &mut g.players {p.hand.clear();p.assets.clear();p.graveyard.clear();p.score_cards.clear();}
    for r in &mut g.regions {r.cards.clear();r.influence=[0;2];}
    let mut sources=vec![];
    for owner in [2,3,0] {let c=g.make_card("XQ23",owner);sources.push(c.id.clone());g.regions[0].cards.push(c);}
    u::fund(&mut g,0,"JC104",6);u::fund(&mut g,0,"JC060",5);
    let spells=[u::held(&mut g,"JC129",0),u::held(&mut g,"JC129",0)];
    u::fund(&mut g,3,"JC060",1);
    let score=g.make_card("DQJC107",2);g.players[2].score_cards.push(score);
    let score=g.make_card("DQJC115",3);g.players[3].score_cards.push(score);g.players[2].deck.clear();
    let mut trace=Trace::new("owner-p2",&g);
    // Original exact88 state remains pinned by frozen-abi88.test.mjs; this is a fresh current-engine run.
    assert_eq!(trace.room.game.versions.engine,catalog::ENGINE_VERSION);
    let mut converted=vec![];
    for(index,owner)in[2,3].into_iter().enumerate() {
        trace.game(0,Action{card_id:Some(spells[index].clone()),target_id:Some(sources[index].clone()),..Action::new("play")},true);trace.responses();
        let source=trace.room.game.board(&sources[index]).unwrap().1;
        assert_eq!((source.owner,source.controller),(owner,0));
        let target=trace.room.game.players[0].assets.iter().find(|c|c.definition=="JC060"&&c.owner==0).unwrap().id.clone();
        trace.game(0,action(&sources[index],&target),true);trace.responses();
        let asset=trace.room.game.players[0].assets.iter().find(|c|c.definition=="XQ23"&&c.owner==owner).unwrap();
        assert_eq!(asset.controller,0);assert!(asset.exhausted);converted.push(asset.id.clone());
    }
    let before=serde_json::to_string(&trace.room).unwrap();
    assert_eq!(crate::room::RoomEnvelope::from_persisted(&before).unwrap().game.versions.engine,catalog::ENGINE_VERSION);
    assert_eq!(trace.room.game.players[0].assets.len(),11);
    let live_owned_before=trace.room.game.players.iter().flat_map(|p|&p.assets).filter(|c|c.owner!=2).map(|c|c.id.clone()).collect::<std::collections::BTreeSet<_>>();
    for _ in 0..600 {if trace.room.game.players[2].eliminated {break;}assert!(trace.room.game.pending.is_none());trace.pass_current();}
    assert!(trace.room.game.players[2].eliminated);assert!(!trace.room.game.players[3].eliminated);
    assert_eq!(trace.room.game.status,"playing");
    assert!(trace.room.game.players.iter().flat_map(|p|&p.assets).all(|c|c.owner!=2));
    assert!(trace.room.game.players[0].assets.iter().all(|c|c.id!=converted[0]));
    assert!(trace.room.game.players[0].assets.iter().any(|c|c.id==converted[1]&&c.owner==3&&c.controller==0));
    assert_eq!(trace.room.game.players.iter().flat_map(|p|&p.assets).map(|c|c.id.clone()).collect::<std::collections::BTreeSet<_>>(),live_owned_before);
    assert_eq!(trace.room.game.players[0].assets.len(),10);
    assert_eq!(trace.room.game.players[0].assets.iter().filter(|c|!c.exhausted).count(),10);
    assert_eq!((trace.room.view(0,0).players[2].score,trace.room.view(0,0).players[3].score),(4,2));
    let after=serde_json::to_string(&trace.room).unwrap();
    for _ in 0..16 {
        if trace.room.game.legal_actions(0).iter().any(|a|a.action.kind=="activate"&&a.action.card_id.as_deref()==Some(sources[2].as_str())&&a.action.target_id.as_deref()==Some(converted[1].as_str())) {break;}
        trace.pass_current();
    }
    assert!(trace.room.game.legal_actions(0).iter().all(|a|a.action.target_id.as_deref()!=Some(converted[0].as_str())));
    trace.game(0,action(&sources[2],&converted[0]),false);
    trace.game(0,action(&sources[2],&converted[1]),true);trace.responses();
    assert!(trace.room.game.players[2].hand.is_empty());
    assert!(trace.room.game.players[3].hand.iter().any(|c|c.definition=="XQ23"));
    assert_eq!((trace.room.view(0,0).players[2].score,trace.room.view(0,0).players[3].score),(4,2));
    let count=trace.steps.len();
    let value=serde_json::json!({"format":"xq23-current-owner-elimination-native-v1","catalog":catalog::catalog(),"sourceBindings":crate::jc035_tests::source_bindings(),"initial":trace.initial,"steps":trace.steps,"beforeElimination":before,"afterElimination":after,"summary":{"actualXQ23Conversions":2,"realJC129Declarations":2,"beforeControlledAssets":11,"afterControlledAssets":10,"readyResourcesAfterElimination":10,"eliminatedOwner2AssetSurvives":false,"livingOwner3AssetSurvives":true,"canLegallyReturnDeadOwnerAsset":false,"deadOwnerHandRepopulated":false,"livingOwnerHandReceivedOwnCard":true,"owner2EliminatedByNaturalNextTurnEmptyDraw":true,"teammate3Lives":true,"gameContinues":true,"commands":count}});
    if let Ok(dir)=std::env::var("XQ23_CURRENT_NATIVE_DIR") {
        use std::io::Write;std::fs::create_dir_all(&dir).unwrap();
        std::fs::OpenOptions::new().create_new(true).write(true).open(std::path::Path::new(&dir).join("NativeCurrentOwner89.json")).unwrap().write_all(&serde_json::to_vec(&value).unwrap()).unwrap();
    }
    println!("XQ23_P2_ELIMINATION {}",value["summary"]);
}
#[test]
fn xq23_bad_paid_frames_targets_sources_and_response_shells_reject_in_game_and_room() {
    let (mut g,s,t)=active(0);u::act(&mut g,0,action(&s,&t));
    for variant in 0..23 {
        let mut v=serde_json::to_value(&g).unwrap();let f=&mut v["stack"][0]["frame"];
        match variant {
            0=>f["source"]["card"]["definition"]="JC060".into(),1=>f["actor"]=7.into(),
            2=>f["source"]["card"]["face_down"]=true.into(),3=>f["source"]["card"]["exhausted"]=true.into(),
            4=>f["already_paid"]=serde_json::json!([]),5=>f["already_paid"][0]["Assets"]=serde_json::json!([]),
            6=>f["already_paid"][0]["Assets"]=serde_json::json!(["same","same"]),7=>f["already_paid"][1]["Exhausted"]="other".into(),
            8=>f["source"]["source_region_instance"]=s.clone().into(),9=>f["source"]["source_region_instance"]=t.clone().into(),
            10=>f["targets"][0]["spec"]["relation"]="Any".into(),11=>f["targets"][0]["spec"]["predicate"]=Value::Null,
            12=>f["targets"]=serde_json::json!([]),13=>f["steps"]=serde_json::json!([]),14=>f["steps"][0]["context"]=2.into(),
            15=>f["cursor"]=1.into(),16=>f["guard"]="Accepted".into(),17=>f["chosen_region"]=0.into(),
            18=>v["stack"][0]["frame"]=Value::Null,19=>v["stack"][0]["controller"]=2.into(),
            20=>v["stack"][0]["deploy_region"]=0.into(),21=>v["stack"][0]["reveal"]=true.into(),_=>v["stack"][0]["target"]="other".into(),
        }
        bad(&serde_json::from_value(v).unwrap(),&format!("shape-{variant}"));
    }
    let mut next=g.clone();let frame=next.stack.pop().unwrap().frame.unwrap();next.effects.push_back(Effect::Frame {frame:Box::new(frame)});bad(&next,"bypass-stack");
    let mut next=g.clone();next.stack.push(next.stack[0].clone());bad(&next,"duplicate-frame");
    let mut next=g.clone();next.players[0].assets.iter_mut().find(|c|c.id==t).unwrap().owner=7;
    bad(&next,"asset-owner-paid-frame");
    let other=g.players[0].assets.iter().find(|c|c.id!=t).unwrap().id.clone();
    for(name,field,value)in[
        ("target-summary-other-asset-uid","instanceId",other.as_str()),
        ("target-summary-empty-uid","instanceId",""),
        ("target-summary-hidden-kind","kind","hidden"),
        ("target-summary-character-kind","kind","character"),
    ] {
        let mut next=serde_json::to_value(&g).unwrap();
        next["stack"][0]["frame"]["targets"][0]["public"][field]=value.into();
        bad(&serde_json::from_value(next).unwrap(),name);
    }
    let (mut next,source,target)=active(0);
    next.players[0].assets.iter_mut().find(|c|c.id==target).unwrap().owner=7;
    let before=serde_json::to_string(&next).unwrap();
    assert!(next.apply(0,action(&source,&target)).unwrap_err().contains("资产区"));
    assert_eq!(serde_json::to_string(&next).unwrap(),before,"Invalid owners reject before labels, costs or mutation");
    bad(&next,"asset-owner-before-declaration");
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
    fn pass_current(&mut self) {
        use crate::room::*;
        if let Some(window)=self.room.pacing.window.clone() {
            let seat=*window.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
            self.command(seat,SessionAction::PassResponse{window_id:window.id},true);
        } else {
            let seat=(0..4).find(|s|self.room.game.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();
            self.game(seat,Action::new("pass"),true);
        }
    }
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
    fn finish(self)->serde_json::Value{serde_json::json!({"name":self.name,"initial":self.initial,"steps":self.steps})}
}
