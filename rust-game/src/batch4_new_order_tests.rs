use crate::{catalog,model::*,unit_support::*};
const ADVANCE:&str=crate::batch4_plans::ADVANCE;
fn plan(g:&mut Game,secret:bool)->String{g.begin_window(Window::Action(0));fund(g,0,"WM045",12);if secret{g.add_society_markers(0,0,1);}let h=held(g,"WM045",0);act(g,0,Action{card_id:Some(h),target_id:Some("society:p0".into()),option:secret.then(||"secret-plan".into()),..Action::new("play")});top(g);g.attachments.iter().find(|a|a.card.definition=="WM045").unwrap().card.id.clone()}
fn threshold(g:&mut Game,id:&str){for n in 1..=3{act(g,0,Action{card_id:Some(id.into()),ability_id:Some(ADVANCE.into()),..Action::new("activate")});if n<3{top(g);}else{for _ in 0..64{if g.stack.last().and_then(|s|s.frame.as_ref()).is_some_and(|f|f.ability_key=="batch4-plan-complete"){restore(g);return;}pass(g);}}}panic!("bounded new-order threshold");}
fn hidden(g:&mut Game,id:&str,seat:usize,r:usize,locks:u32)->String{let mut c=g.make_card(id,seat);c.face_down=true;c.lock_markers=locks;let id=c.id.clone();g.regions[r].cards.push(c);id}
fn drain(g:&mut Game){for _ in 0..192{if g.pending.is_some(){resolve_choice(g);}else if !g.effects.is_empty(){g.drive().unwrap();restore(g);}else if !g.stack.is_empty(){pass(g);}else{return;}}panic!("bounded new-order aftermath");}

#[test]
fn batch4_new_order_exact_original_plan_and_closed_native_registry(){
 let c=catalog::card("WM045");assert_eq!((c.cost,c.kind.as_str(),c.defense,c.supported,c.unique),(1,"attachment",None,false,false));assert_eq!(c.loyalty,["灰色"]);assert_eq!(c.subtypes,["策略","计划"]);assert_eq!(crate::batch4_plans::threshold("WM045"),Some(3));assert!(crate::batch4_plans::CARDS.contains(&"WM045"));let mut wrong=crate::rules::definitions().clone();wrong.get_mut("WM045").unwrap().abilities[2].ops.clear();assert!(crate::rules::validate_definitions(&wrong).is_err());
}

#[test]
fn batch4_new_order_secret_real_pay_three_forces_public_completion_without_optional_decline(){
 let mut g=game();let role=field(&mut g,"JC125",2);let id=plan(&mut g,true);assert_eq!(g.resources(0),11);for s in [1,2,3]{let v=g.view(s);let c=&v.attachments.iter().find(|a|a.card.instance_id==id).unwrap().card;assert_eq!(c.name,"秘密计划");assert!(c.card_id.is_none()&&c.text.is_none());}threshold(&mut g,&id);assert_eq!(g.resources(0),8);assert!(g.pending.is_none());assert!(!g.attachments.iter().find(|a|a.card.id==id).unwrap().card.face_down);for s in 0..4{assert_eq!(g.view(s).attachments.iter().find(|a|a.card.instance_id==id).unwrap().card.card_id.as_deref(),Some("WM045"));}top(&mut g);assert!(g.board(&role).is_none());assert!(g.regions[0].cards.iter().any(|c|c.definition=="JC125"&&c.face_down));assert!(!g.attachments.iter().any(|a|a.card.id==id));assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="WM045").count(),1);restore(&mut g);
}

#[test]
fn batch4_new_order_flip_clears_faceup_markers_but_shuffle_preserves_preexisting_hidden_markers(){
 let mut g=game();let face=field(&mut g,"JC032",0);g.board_mut(&face).unwrap().lock_markers=4;let old=hidden(&mut g,"JC125",0,0,2);let id=plan(&mut g,false);threshold(&mut g,&id);top(&mut g);assert!(g.board(&face).is_none()&&g.board(&old).is_none());let face=g.regions[0].cards.iter().find(|c|c.definition=="JC032").unwrap();let old=g.regions[0].cards.iter().find(|c|c.definition=="JC125").unwrap();assert_eq!((face.lock_markers,old.lock_markers),(0,2));for s in [1,2,3]{let v=g.view(s);for c in &v.regions[0].characters{assert!(c.face_down&&c.card_id.is_none()&&c.text.is_none());}}restore(&mut g);
}

#[test]
fn batch4_new_order_groups_are_per_controller_and_region_and_singletons_keep_identity(){
 let mut g=game();let a=hidden(&mut g,"JC125",0,0,1);let b=hidden(&mut g,"JC032",0,0,2);let ally=hidden(&mut g,"JC125",1,0,3);let elsewhere=hidden(&mut g,"JC125",0,1,4);let enemy=hidden(&mut g,"JC125",2,0,5);let id=plan(&mut g,false);let rng=g.random;threshold(&mut g,&id);top(&mut g);assert!(g.board(&a).is_none()&&g.board(&b).is_none());assert_ne!(g.random,rng);for (id,marks,r) in [(ally,3,0),(elsewhere,4,1),(enemy,5,0)]{let (actual,c)=g.board(&id).unwrap();assert_eq!((actual,c.lock_markers),(r,marks));}assert_eq!(g.regions[0].cards.iter().filter(|c|c.controller==0).count(),2);assert_eq!(g.regions[1].cards.len(),1);restore(&mut g);
}

#[test]
fn batch4_new_order_attached_bat_and_host_both_hide_before_host_cleanup_and_seals_return(){
 let mut g=game();let host=field(&mut g,"JC029",0);fund(&mut g,0,"BQ026",1);let h=held(&mut g,"BQ026",0);act(&mut g,0,Action{card_id:Some(h),region:Some(0),..Action::new("deploy")});for _ in 0..64{if g.pending.is_some(){break;}pass(&mut g);}choose(&mut g,vec![host.clone()]);top(&mut g);let bat=g.attachments.iter().find(|a|a.card.definition=="BQ026").unwrap().card.id.clone();for h in [&host,&bat]{let c=g.make_card("JC125",0);g.seal_card_on_valid_host(c,0,h);}g.drive().unwrap();let id=plan(&mut g,false);threshold(&mut g,&id);top(&mut g);assert_eq!(g.regions[0].cards.iter().filter(|c|c.face_down).count(),2);assert!(g.attachments.is_empty()&&g.sealed_cards.is_empty());assert_eq!(g.players[0].hand.iter().filter(|c|c.definition=="JC125").count(),2);assert!(!g.players[0].graveyard.iter().any(|c|matches!(c.definition.as_str(),"BQ026"|"JC029")));assert!(g.board(&host).is_none()&&g.board(&bat).is_none());restore(&mut g);
}

#[test]
fn batch4_new_order_two_worm_hide_events_keep_original_snapshots_after_mix_and_restore(){
 let mut g=game();field(&mut g,"JC054",0);field(&mut g,"JC054",0);let id=plan(&mut g,false);threshold(&mut g,&id);top(&mut g);drain(&mut g);assert_eq!(g.regions[0].cards.iter().filter(|c|c.definition=="JC054"&&c.face_down).count(),2);assert!(g.regions[0].cards.iter().all(|c|c.damage==0));assert!(g.players[0].graveyard.iter().all(|c|c.definition!="JC054"));restore(&mut g);
}

#[test]
fn batch4_new_order_high_u64_seed_is_deterministic_through_real_paid_completion_and_restore(){
 let mut g=game();g.seed=9_007_199_254_740_993;g.random=g.seed;for (n,id) in ["JC125","JC032","JC003","JC070"].iter().enumerate(){hidden(&mut g,id,0,0,n as u32);}let id=plan(&mut g,true);threshold(&mut g,&id);let mut clone=g.clone();top(&mut g);top(&mut clone);assert_eq!(g.seed,9_007_199_254_740_993);assert_eq!(serde_json::to_string(&g).unwrap(),serde_json::to_string(&clone).unwrap());for s in 0..4{assert_eq!(serde_json::to_value(g.view(s)).unwrap(),serde_json::to_value(clone.view(s)).unwrap());}restore(&mut g);
}

#[test]
fn batch4_new_order_shuffle_retires_old_target_identity_without_clearing_its_lock_marker(){
 // Explicit native response-boundary fixture: the already-paid real spell is
 // below a direct mass-shuffle primitive, not a claimed public fast advance.
 let mut g=game();let old=hidden(&mut g,"JC125",0,0,1);hidden(&mut g,"JC032",0,0,0);g.begin_window(Window::Action(1));fund(&mut g,2,"JC063",2);let h=held(&mut g,"JC063",2);act(&mut g,2,Action{card_id:Some(h),target_id:Some(old.clone()),option:Some("return".into()),..Action::new("play")});g.world_hide(None,true);restore(&mut g);assert!(g.board(&old).is_none());top(&mut g);assert_eq!(g.regions[0].cards.len(),2);assert_eq!(g.regions[0].cards.iter().map(|c|c.lock_markers).sum::<u32>(),1);assert_eq!(g.resources(2),0);assert!(g.players[0].hand.is_empty());restore(&mut g);
}

#[test]
fn batch4_new_order_accepted_completion_survives_real_disintegration_of_its_original_plan(){
 let mut g=game();field(&mut g,"JC125",0);let id=plan(&mut g,true);threshold(&mut g,&id);fund(&mut g,2,"JC005",catalog::card("JC005").cost as usize);let h=held(&mut g,"JC005",2);for _ in 0..8{if g.priority_team==g.team(2){break;}pass(&mut g);}assert!(g.legal_actions(2).iter().any(|a|a.action.kind=="play"&&a.action.card_id.as_deref()==Some(&h)&&a.action.target_id.as_deref()==Some(&id)));act(&mut g,2,Action{card_id:Some(h),target_id:Some(id.clone()),..Action::new("play")});top(&mut g);assert!(!g.attachments.iter().any(|a|a.card.id==id));top(&mut g);assert!(g.regions[0].cards.iter().all(|c|c.face_down));assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="WM045").count(),1);restore(&mut g);
}
