use crate::{model::*,unit_support::*};
fn grave(g:&mut Game,print:&str,owner:usize)->String{let c=g.make_card(print,owner);let id=c.id.clone();g.players[owner].graveyard.push(c);id}
fn begin(g:&mut Game){g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();}
fn pending(g:&mut Game){for _ in 0..32{if g.pending.is_some(){restore(g);return;}pass(g);}panic!("pending missing");}
#[test]fn batch3_bq087_forced_prepare_binds_own_target_before_stack_and_resolves_rng_once_after_source_departure(){
    let mut g=game();g.seed=u64::MAX-17;g.random=g.seed;let source=field(&mut g,"BQ087",0);let id=grave(&mut g,"WM035",0);let enemy=grave(&mut g,"JC125",2);begin(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.min,Some(1));let p=g.pending.clone().unwrap();reject(&mut g,0,Action{choice_id:Some(p.choice.id),selected:Some(vec![]),..Action::new("choose")});
    choose(&mut g,vec![source.clone()]);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"batch3-bq087-target");
    assert!(g.pending.as_ref().unwrap().choice.options.iter().all(|o|o.id!=enemy));let p=g.pending.clone().unwrap();reject(&mut g,2,Action{choice_id:Some(p.choice.id),selected:Some(vec![id.clone()]),..Action::new("choose")});
    choose(&mut g,vec![id.clone()]);let mut expected=g.clone();let region=expected.random_below(g.regions.len());
    g.board_mut(&source).unwrap().controller=2;g.remove_dead(&source,RemovalCause::Destroy);top(&mut g);
    assert_eq!(g.random,expected.random);assert_eq!(g.seed,u64::MAX-17);let c=g.regions[region].cards.iter().find(|c|c.definition=="WM035").unwrap();
    assert_ne!(c.id,id);assert_eq!((c.owner,c.controller,c.face_down,c.exhausted),(0,0,false,false));restore(&mut g);
}
#[test]fn batch3_bq087_empty_grave_or_only_spells_produces_no_target_prompt_and_no_rng(){
    for have_spell in [false,true]{let mut g=game();field(&mut g,"BQ087",0);if have_spell{grave(&mut g,"JC047",0);}let random=g.random;begin(&mut g);
        assert!(g.pending.is_none());assert!(g.stack.is_empty());assert_eq!(g.random,random);assert!(g.jc035_preparation.as_ref().unwrap().entries.is_empty());restore(&mut g);}
}
#[test]fn batch3_bq087_lost_grave_target_cancels_and_keeps_a_completed_preparation_receipt(){
    let mut g=game();let source=field(&mut g,"BQ087",0);let target=grave(&mut g,"JC125",0);begin(&mut g);choose(&mut g,vec![source]);choose(&mut g,vec![target.clone()]);
    let random=g.random;g.players[0].graveyard.retain(|c|c.id!=target);top(&mut g);assert_eq!(g.random,random);
    assert!(matches!(g.jc035_preparation.as_ref().unwrap().entries[0].decision,Some(crate::jc035::PreparationDecision::Cancelled{..})));restore(&mut g);
}
#[test]fn batch3_preparation_existing_jc055_lost_target_also_closes_its_original_receipt(){
    let mut g=game();let source=field(&mut g,"JC055",0);let target=field(&mut g,"JC125",0);begin(&mut g);choose(&mut g,vec![source]);choose(&mut g,vec![target.clone()]);
    g.remove_dead(&target,RemovalCause::Destroy);top(&mut g);assert!(matches!(g.jc035_preparation.as_ref().unwrap().entries[0].decision,Some(crate::jc035::PreparationDecision::Cancelled{..})));restore(&mut g);
}
#[test]fn batch3_bq087_leader_gate_uses_personal_sun_and_keeps_gold_uniqueness_separate(){
    let mut g=game();let source=field(&mut g,"BQ087",0);let leader=field(&mut g,"WM035",0);
    assert_eq!(g.leader_options(0).iter().map(|o|o.id.clone()).collect::<Vec<_>>(),vec![leader.clone()]);g.players[1].society_zone.exposure=1;assert!(!g.current_leader(g.board(&source).unwrap().1));
    g.players[0].society_zone.exposure=1;assert_eq!(g.leader_options(0).len(),2);g.queue_leader_checks();g.drive().unwrap();assert_eq!(g.pending.as_ref().unwrap().choice.min,Some(1));choose(&mut g,vec![source.clone()]);assert!(g.board(&source).is_none());assert!(g.board(&leader).is_some());restore(&mut g);
}
#[test]fn batch3_bq087_moon_rumor_uses_real_mill_replacement_without_sun_leader(){
    let mut g=game();let mill=field(&mut g,"XG28",0);g.remove_board(&mill);let c=g.make_card("XG28",0);let source=c.id.clone();g.attachments.push(Attachment{card:c,host_id:g.regions[0].card.id.clone()});
    let host=field(&mut g,"JC125",0);let serpent=g.make_card("BQ087",0);g.players[0].deck.insert(0,serpent);g.players[0].society_zone.secrecy=1;begin(&mut g);choose(&mut g,vec![source]);pending(&mut g);assert!(matches!(g.pending.as_ref().unwrap().resolution,ChoiceResolution::Rumor{..}));choose(&mut g,vec![host]);
    assert!(g.sealed_cards.iter().any(|s|s.card.definition=="BQ087"));assert!(g.players[0].graveyard.iter().all(|c|c.definition!="BQ087"));restore(&mut g);
}
#[test]fn batch3_bq087_conditional_leader_does_not_corrupt_same_name_gold_uniqueness(){
    let mut g=game();let a=field(&mut g,"BQ087",0);let b=field(&mut g,"BQ087",0);g.effects.push_back(Effect::Unique{seat:0});g.drive().unwrap();
    assert!(g.pending.as_ref().unwrap().choice.title.starts_with("独有"));choose(&mut g,vec![a.clone()]);assert!(g.board(&a).is_none());assert!(g.board(&b).is_some());assert!(g.leader_options(0).is_empty());restore(&mut g);
}
#[test]fn batch3_bq087_sealed_outcome_survives_a_second_rumor_checkpoint_in_real_two_card_mill(){
    let mut g=game();let host=field(&mut g,"JC125",0);let c=g.make_card("JZ57",0);let source=c.id.clone();g.attachments.push(Attachment{card:c,host_id:host.clone()});
    let seal_host=field(&mut g,"JC125",1);g.players[0].deck=vec![g.make_card("BQ087",0),g.make_card("WM042",0)];g.players[0].society_zone.secrecy=1;
    act(&mut g,0,Action{card_id:Some(source),ability_id:Some("batch20-jz57-mill".into()),target_id:Some("p0".into()),cost_selected:Some(vec![host]),..Action::new("activate")});
    pending(&mut g);choose(&mut g,vec![seal_host.clone()]);assert!(matches!(g.pending.as_ref().unwrap().resolution,ChoiceResolution::Rumor{..}));restore(&mut g);
    choose(&mut g,vec![seal_host]);assert_eq!(g.sealed_cards.len(),2);if g.pending.is_some(){choose(&mut g,vec![]);}restore(&mut g);
}
#[test]fn batch3_bq087_real_paid_deploy_preserves_printed_icons_and_does_not_gain_leader_without_sun(){
    let mut g=game();field(&mut g,"WM035",0);fund(&mut g,0,"JC084",7);let id=held(&mut g,"BQ087",0);
    act(&mut g,0,Action{card_id:Some(id),region:Some(0),..Action::new("deploy")});top(&mut g);assert!(g.pending.is_none());
    let c=g.regions[0].cards.iter().find(|c|c.definition=="BQ087").unwrap();assert!(!g.current_leader(c));assert_eq!(g.defense(c,0),5);
    assert_eq!(g.current_icons(c,0),Icons{investigation:3,combat:1,influence:1});assert_eq!(g.players[0].assets.iter().filter(|a|a.exhausted).count(),7);restore(&mut g);
}
