use crate::{bq052 as b, model::*, unit_support::*};
fn setup()->(Game,String) {
    let mut g=game();g.players[0].society_zone.card=Some(g.make_card("MSBQ02",0));
    let core=g.make_card("BQ046",0);g.core_zones.push(crate::core_zone::CoreZone{owner:0,definition:"BQ046".into(),card:Some(core),rest:2});
    let c=g.make_card(b::ID,0);let id=c.id.clone();g.attachments.push(Attachment{card:c,host_id:"society:p0".into()});(g,id)
}
fn prepare(g:&mut Game,source:&str) {
    g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();restore(g);
    choose(g,vec![source.into()]);crate::jc029_tests::pass_top(g);restore(g);
}
fn count(g:&Game)->usize {g.batch3_physical_cards().filter(|c|c.definition=="TK010").count()}
fn activate(source:&str,ids:Vec<String>)->Action {Action{card_id:Some(source.into()),ability_id:Some(b::PLACE.into()),cost_selected:Some(ids),..Action::new("activate")}}
fn bad(reference:&Game,g:&Game) {let game_error=Game::from_persisted(&serde_json::to_string(g).unwrap()).err().expect("corrupt Game accepted");let mut room=crate::jc029_tests::envelope(reference);room.game=g.clone();let room_error=crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).err().expect("corrupt Room accepted");assert_eq!(room_error,game_error);}

#[test]
fn bq052_preparation_generates_exactly_one_and_decline_generates_none() {
    let(mut g,id)=setup();prepare(&mut g,&id);assert_eq!(count(&g),1);
    prepare(&mut g,&id);assert_eq!(count(&g),2);
    g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();choose(&mut g,vec![]);assert_eq!(count(&g),2);restore(&mut g);
}
#[test]
fn bq052_three_actual_generated_tentacles_pay_before_response_then_core_enters() {
    let(mut g,id)=setup();for _ in 0..3{prepare(&mut g,&id);}assert_eq!(count(&g),3);
    g.begin_window(Window::Action(0));
    let ids=g.regions.iter().flat_map(|r|&r.cards).filter(|c|c.definition=="TK010").map(|c|c.id.clone()).collect::<Vec<_>>();
    act(&mut g,0,activate(&id,ids));assert_eq!(count(&g),0);assert!(g.core_zones[0].card.is_some());
    restore(&mut g);crate::jc029_tests::pass_top(&mut g);
    assert!(g.core_zones[0].card.is_none());assert_eq!(g.core_zones[0].rest,2);
    assert_eq!(g.regions.iter().flat_map(|r|&r.cards).filter(|c|c.definition=="BQ046").count(),1);restore(&mut g);
}
#[test]
fn bq052_rejects_repeat_hidden_enemy_and_short_sacrifice_without_partial_payment() {
    let(mut g,id)=setup();let ids=(0..3).map(|_|field(&mut g,"TK010",0)).collect::<Vec<_>>();
    for selected in [vec![ids[0].clone();3],ids[..2].to_vec(),vec![]] {reject(&mut g,0,activate(&id,selected));assert_eq!(count(&g),3);}
    g.hide_in_place(&ids[0]);reject(&mut g,0,activate(&id,ids.clone()));assert_eq!(count(&g),3);
    let enemy=field(&mut g,"TK010",1);reject(&mut g,0,activate(&id,vec![enemy,ids[1].clone(),ids[2].clone()]));assert_eq!(count(&g),4);
}
#[test]
fn bq052_aura_reads_current_secrecy_core_and_face_up_tentacles() {
    let(mut g,id)=setup();let token=field(&mut g,"TK010",0);
    assert_eq!(g.bq052_influence(g.board(&token).unwrap().1),0);
    g.players[0].society_zone.secrecy=1;assert_eq!(g.bq052_influence(g.board(&token).unwrap().1),1);
    g.players[0].society_zone.secrecy=0;let core=g.core_zones[0].card.take().unwrap();g.regions[1].cards.push(core);
    assert_eq!(g.bq052_influence(g.board(&token).unwrap().1),1);
    g.remove_dead(&id,RemovalCause::Destroy);assert_eq!(g.bq052_influence(g.board(&token).unwrap().1),0);restore(&mut g);
}
#[test]
fn bq052_corrupt_original_payment_and_program_rejected() {
    let(mut g,id)=setup();let ids=(0..3).map(|_|field(&mut g,"TK010",0)).collect();act(&mut g,0,activate(&id,ids));
    for mode in 0..4 {let mut broken=g.clone();let f=broken.stack.last_mut().unwrap().frame.as_mut().unwrap();match mode{0=>f.source.card.definition="JC001".into(),1=>f.already_paid.clear(),2=>f.steps.clear(),_=>if let PaidCost::BQ052Sacrificed(v)=&mut f.already_paid[0]{v[1]=v[0].clone();}}bad(&g,&broken);}
    restore(&mut g);
}

#[test]
fn bq052_real_paid_own_society_attachment_and_corrupt_actual_card_frame() {
    let(mut g,_)=setup();g.attachments.clear();fund(&mut g,0,"JC042",2);let id=held(&mut g,b::ID,0);
    let action=g.legal_actions(0).into_iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_deref()==Some(&id)).expect("legal own society attach").action;
    act(&mut g,0,action);
    eprintln!("paid attach stack identities: {:?}",g.stack.iter().map(|i|(i.card.as_ref().map(|c|c.definition.as_str()),i.frame.as_ref().map(|f|f.source.card.definition.as_str()))).collect::<Vec<_>>());
    for missing in [false,true] {let mut broken=g.clone();let item=broken.stack.iter_mut().find(|i|i.card.as_ref().is_some_and(|c|c.definition==b::ID)).expect("actual BQ052 attachment transaction");if missing{item.frame=None;}else{item.frame.as_mut().unwrap().source.card.definition="JC001".into();}bad(&g,&broken);}
    let mut missing_loss=g.clone();missing_loss.stack.iter_mut().find(|i|i.frame.as_ref().is_some_and(|f|f.ability_key==crate::msbq02::LOSS)).unwrap().frame=None;bad(&g,&missing_loss);
    drain(&mut g);assert_eq!(g.attachments.len(),1);assert_eq!(g.attachments[0].host_id,"society:p0");restore(&mut g);
}

#[test]
fn bq052_preparation_missing_or_duplicate_frame_cannot_generate_twice() {
    let(mut g,id)=setup();g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();choose(&mut g,vec![id]);
    let mut broken=g.clone();broken.stack.last_mut().unwrap().frame=None;bad(&g,&broken);
    let mut broken=g.clone();broken.stack.push(broken.stack[0].clone());bad(&g,&broken);
    crate::jc029_tests::pass_top(&mut g);assert_eq!(count(&g),1);restore(&mut g);
    // Re-entering the same preparation window does not create another observation.
    g.begin_window(Window::Prepare);g.drive().unwrap();assert_eq!(count(&g),1);assert!(g.pending.is_none());
}

fn constructed(seed:u64)->(Game,crate::deck::DeckDraft) {
    use crate::{catalog,deck};
    let mut d=deck::preset("watchers").unwrap();d.id="closed-bq052-50-plus-core".into();d.name="辞典触手私有50+1".into();d.description="50普通牌及独立核心；非官方预组".into();d.society_id=Some("MSBQ02".into());d.core_card_id=Some("BQ046".into());
    d.cards=[("BQ064",3),("BQ048",3),("BQ050",3),("BQ063",3),("BQ052",3),("JC125",35)].into_iter().map(|(id,count)|catalog::DeckEntry{card_id:id.into(),count}).collect();assert!(deck::validate(d.clone()).is_err());
    let mut cards=catalog::catalog().cards.clone();for c in &mut cards{if ["BQ046","BQ048","BQ050","BQ063","BQ052"].contains(&c.id.as_str()){c.supported=true;}}
    let mut societies=catalog::catalog().societies.clone();societies.iter_mut().find(|s|s.card.id=="MSBQ02").unwrap().card.supported=true;
    let d=deck::validate_with_registry(d,&cards,&societies).unwrap();
    let mut other=deck::preset("watchers").unwrap();other.society_id=None;other.core_card_id=None;other.cards=vec![catalog::DeckEntry{card_id:"JC125".into(),count:50}];let other=deck::validate(other).unwrap();
    let mut g=Game::new("bq052-natural".into(),"LOCAL".into(),"teams".into(),"P0".into(),"watchers".into(),seed).unwrap();for seat in 1..4{g.join(format!("P{seat}"),"watchers".into()).unwrap();}
    g.start_validated_decks(vec![d.clone(),other.clone(),other.clone(),other]).unwrap();g.drive().unwrap();(g,d)
}
fn natural()->serde_json::Value {
    use crate::abi112_review_play_tests::Trace;
    let(seed,g,deck)=(1..513).find_map(|seed|{let(g,d)=constructed(seed);let h=&g.players[0].hand;(h.iter().filter(|c|c.definition==b::ID).count()>=2&&h.iter().any(|c|c.definition=="JC125")).then_some((seed,g,d))}).expect("bounded actual shuffled opening");
    let mut t=Trace::new_preserving_rng(g,"bq052-natural-generation-and-core-placement".into(),682);
    let mut paid=false;let mut done=false;let mut generated=0usize;
    for _ in 0..1800 {
        generated=generated.max(count(&t.room.game));
        if paid&&t.room.game.stack.is_empty()&&t.room.game.effects.is_empty()&&t.room.game.pending.is_none(){done=true;break;}
        if let Some(p)=t.room.game.pending.clone(){
            if let ChoiceResolution::JC035PreparationOrder{declarations}=&p.resolution {
                let source=declarations.iter().find(|d|d.source.card.definition==b::ID).expect("dictionary preparation").source.card.id.clone();t.choose(vec![source]);
            }else if p.choice.kind=="investigation" {
                let ids=p.choice.options.iter().map(|o|o.id.clone()).collect();t.action(p.seat,Action{choice_id:Some(p.choice.id),top:Some(ids),bottom:Some(vec![]),..Action::new("choose")});
            }else{t.choose(p.choice.options.iter().take(p.choice.min.unwrap_or(0)).map(|o|o.id.clone()).collect());}
            continue;
        }
        let g=&t.room.game;let legal=g.legal_actions(0);
        let a=if g.players[0].assets.len()<2 {
            let desired=if g.players[0].assets.is_empty(){b::ID}else{"JC125"};
            legal.iter().find(|a|a.action.kind=="asset"&&a.action.card_id.as_ref().is_some_and(|id|g.players[0].hand.iter().any(|c|&c.id==id&&c.definition==desired)))
        }else if !g.attachments.iter().any(|a|a.card.definition==b::ID){
            legal.iter().find(|a|a.action.kind=="play"&&a.action.card_id.as_ref().is_some_and(|id|g.players[0].hand.iter().any(|c|&c.id==id&&c.definition==b::ID)))
        }else{legal.iter().find(|a|a.action.ability_id.as_deref()==Some(b::PLACE))};
        if let Some(a)=a {if a.action.ability_id.as_deref()==Some(b::PLACE){assert_eq!(a.action.cost_selected.as_ref().unwrap().len(),3);paid=true;}t.action(0,a.action.clone());}else{t.pass();}
    }
    assert!(done,"bounded natural chain unfinished at turn {}",t.room.game.turn);assert_eq!(generated,3);assert_eq!(count(&t.room.game),0);
    assert!(t.room.game.core_zones[0].card.is_none());assert_eq!(t.room.game.regions.iter().flat_map(|r|&r.cards).filter(|c|c.definition=="BQ046").count(),1);
    let mut v=t.value();v["construction"]=serde_json::json!({"deck":deck,"seed":seed,"ordinaryCards":50,"separateCore":1,"minimumCommandCards":12,"privateRegistryOnly":true,"noLayoutOrResourceMutation":true,"generatedTentacles":generated});
    for step in v["steps"].as_array_mut().unwrap(){let room=crate::room::RoomEnvelope::from_persisted(step["expected"]["state"].as_str().unwrap()).unwrap();let d=room.transition(step["seat"].as_u64().unwrap()as usize,Some(serde_json::from_value(step["command"].clone()).unwrap()),0).unwrap();assert_eq!(d.outcome,"rejected");assert_eq!(d.state,step["expected"]["state"]);assert!(!d.changed&&d.journal.is_empty());step["duplicate"]=serde_json::to_value(d).unwrap();}
    v
}
pub(crate) fn source_bindings()->serde_json::Value {
    use sha2::{Digest,Sha256};let mut bindings=crate::shoggoth_tests::shoggoth_source_bindings();
    for(name,bytes)in[("rust-game/src/bq052.rs",include_bytes!("bq052.rs").as_slice()),("rust-game/src/bq052_tests.rs",include_bytes!("bq052_tests.rs").as_slice()),("rust-game/src/tentacle.rs",include_bytes!("tentacle.rs").as_slice()),("rust-game/src/tentacle_tests.rs",include_bytes!("tentacle_tests.rs").as_slice()),("rust-game/src/jc035.rs",include_bytes!("jc035.rs").as_slice())]{bindings["files"].as_object_mut().unwrap().insert(name.into(),serde_json::json!(format!("{:x}",Sha256::digest(bytes))));}bindings
}
#[test]
fn bq052_natural_fifty_plus_core_three_preparations_and_atomic_sacrifice_export() {
    let value=natural();if let Ok(path)=std::env::var("BQ052_NATIVE_OUT"){std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"catalog":crate::catalog::catalog(),"sourceBindings":source_bindings(),"cases":[value]})).unwrap()).unwrap();}
}
