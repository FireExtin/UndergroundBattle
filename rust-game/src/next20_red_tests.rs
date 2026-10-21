//! Native public-command coverage; unit_support round-trips all four views after every command.
use crate::{catalog, model::*, rules::{self, *}, unit_support::*};
use std::collections::BTreeMap;

fn command(kind: &str, id: &str) -> Action { Action { card_id: Some(id.into()), ..Action::new(kind) } }
fn rejected_state(g: &Game) { assert!(Game::from_persisted(&serde_json::to_string(g).unwrap()).is_err()); }
fn until_choice(g: &mut Game) {
    for _ in 0..40 { if g.pending.is_some() { restore(g); return; } pass(g); }
    panic!("expected bounded pending choice");
}
fn emit(g: &mut Game, id: &str, event: Event) {
    let (r,c) = g.in_play(id).unwrap();
    let s = g.source_snapshot(c,r); g.emit_event(s.card.controller,s,event); g.drive().unwrap(); restore(g);
}
fn choose_request(g: &Game, ids: Vec<String>) -> Action { Action { choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()), selected: Some(ids), ..Action::new("choose") } }

#[test]
fn next20_red_prints_and_exact_program_admission() {
    for id in crate::next20_red::CARDS {
        let d = catalog::card(id); assert!(d.supported); assert_eq!(d.color,"红");
        let mut defs = rules::definitions().clone(); defs.get_mut(*id).unwrap().traits.retreat = true;
        assert!(rules::validate_definitions(&defs).is_err());
        for a in &rules::definition(id).abilities {
            let mut a = a.clone(); a.timing = if a.timing == Timing::Fast { Timing::Standard } else { Timing::Fast };
            assert!(rules::validate_ability(id,&a).is_err(),"{id} {}", a.key);
        }
    }
    assert!(catalog::card("JC051").unique); assert_eq!(catalog::card("JC051").subtitle.as_deref(),Some("鸣钟人"));
    assert!(catalog::card("JZ34").unique); assert_eq!(catalog::card("JZ34").defense,Some(2));
    assert_eq!(catalog::card("JC055").loyalty.len(),2);
    let mut a = crate::next20_red::definition("JZ32").unwrap().abilities[0].clone();
    a.costs.clear(); assert!(rules::validate_ability("JZ32",&a).is_err());
    assert!(rules::validate_ability("JC125",&crate::next20_red::definition("JC051").unwrap().abilities[0]).is_err());
    let mut g=game(); let id=field(&mut g,"JZ32",0); g.regions[0].destroyed=true;
    let (r,c)=g.in_play(&id).unwrap(); let source=g.source_snapshot(c,r);
    let reward=crate::renown::renown_ability(&g.regions[0].card.id);
    let mut bad=g.clone(); bad.effects.push_back(Effect::Declare { declaration:Declaration { actor:0,source:source.clone(),
        ability:rules::ability("forged-red-draw","伪造",Timing::Fast,vec![],vec![],vec![Op::Draw { player:PlayerRef::Actor,count:1,end:DeckEnd::Top }],Some(Event::Enter)) } });
    rejected_state(&bad);
    g.effects.push_back(Effect::Declare { declaration:Declaration { actor:0,source,ability:reward } });
    g.drive().unwrap(); restore(&mut g); choose(&mut g,vec!["accept".into()]); top(&mut g); assert_eq!(g.regions[0].influence,[1,0]);
    // Exact region identity belongs to JZ34's printed death target, not to a
    // canonical generic mobility slot whose own receipt is validated separately.
    let id=field(&mut g,"JZ34",0); let (r,c)=g.in_play(&id).unwrap(); let source=g.source_snapshot(c,r);
    let slot=crate::mobility::ability().targets[0].clone();
    let target=BoundTarget { id:"region:1".into(),public:g.public_target(0,&source,&slot,"region:1"),spec:slot,region_instance:None };
    assert!(g.valid_bound_target(0,&source,&target));
}

#[test]
fn next20_red_jc051_public_reveal_only_and_empty_search_shuffles() {
    let mut g = game(); fund(&mut g,0,"JC047",8);
    let hand = held(&mut g,"JC051",0);
    act(&mut g,0,Action { region: Some(0), ..command("deploy",&hand) });
    top(&mut g); assert!(g.pending.is_none(),"ordinary entry is not Reveal");
    let id = g.regions[0].cards.iter().find(|c| c.definition == "JC051").unwrap().id.clone();
    g.board_mut(&id).unwrap().face_down = true;
    for a in &mut g.players[0].assets { a.exhausted = false; }
    act(&mut g,0,command("reveal",&id)); until_choice(&mut g);
    assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare { declaration,.. } if declaration.ability.event==Some(Event::Reveal)));
    choose(&mut g,vec!["accept".into()]);
    let random = g.random; let deck = g.players[0].deck.iter().map(|c| c.id.clone()).collect::<std::collections::BTreeSet<_>>();
    top(&mut g); assert_ne!(random,g.random); assert!(g.pending.is_none());
    assert_eq!(deck,g.players[0].deck.iter().map(|c| c.id.clone()).collect());
    assert!(g.view(2).pending_choice.is_none());
}

#[test]
fn next20_red_jc051_aura_tracks_controller_visibility_and_initiative() {
    let mut g = game(); let leader = field(&mut g,"JC051",0); let red = field(&mut g,"JZ32",0);
    let teammate = field(&mut g,"JZ32",1); let other = field(&mut g,"JC018",0);
    let icons = |g: &Game,id: &str| { let (r,c)=g.board(id).unwrap(); g.current_icons(c,r).influence };
    assert_eq!(icons(&g,&leader),2); assert_eq!(icons(&g,&red),1); assert_eq!(icons(&g,&teammate),0); assert_eq!(icons(&g,&other),1);
    g.first_team=1; assert_eq!(icons(&g,&red),0); g.first_team=0;
    g.board_mut(&leader).unwrap().face_down=true; assert_eq!(icons(&g,&red),0);
    g.board_mut(&leader).unwrap().face_down=false; g.board_mut(&leader).unwrap().controller=1;
    assert_eq!(icons(&g,&red),0); assert_eq!(icons(&g,&teammate),1);
    g.remove_dead(&leader,RemovalCause::Destroy); assert_eq!(icons(&g,&teammate),0); restore(&mut g);
}

#[test]
fn next20_red_jc055_preparation_uses_shared_observation_and_target_restore() {
    let mut g=game(); let source=field(&mut g,"JC055",0); let enemy=field(&mut g,"JC125",2);
    let hidden=field(&mut g,"JC125",3); g.board_mut(&hidden).unwrap().face_down=true;
    let a=g.make_card("BQ022",2); let attachment=a.id.clone(); g.attachments.push(Attachment { card:a,host_id:enemy.clone() });
    g.jc035_preparation=None; g.begin_window(Window::Prepare); g.drive().unwrap(); restore(&mut g);
    choose(&mut g,vec![source.clone()]);
    let p=g.pending.clone().unwrap(); assert!(matches!(p.resolution,ChoiceResolution::JC055PreparationTarget {..}));
    for id in [&source,&enemy,&attachment] { assert!(p.choice.options.iter().any(|o| o.id==*id)); }
    assert!(!p.choice.options.iter().any(|o| o.id==hidden));
    assert!(g.view(2).pending_choice.is_none());
    let mut bad=g.clone(); bad.pending.as_mut().unwrap().choice.options.pop(); rejected_state(&bad);
    let mut bad=g.clone(); if let ChoiceResolution::JC055PreparationTarget { declaration,.. }=&mut bad.pending.as_mut().unwrap().resolution { declaration.source.source_region_instance=Some("replacement".into()); } rejected_state(&bad);
    let request=choose_request(&g,vec![attachment.clone()]); reject(&mut g,1,request.clone()); act(&mut g,0,request.clone()); reject(&mut g,0,request);
    g.board_mut(&source).unwrap().face_down=true; // Event-time preparation source remains valid after hiding.
    top(&mut g); assert_eq!(g.board(&attachment).unwrap().1.time_markers,1);
    assert_eq!(g.board(&enemy).unwrap().1.time_markers,0);
    g.begin_window(Window::Prepare); g.drive().unwrap(); assert!(g.pending.is_none(),"one preparation observation per turn");
}

#[test]
fn next20_red_jc055_distributed_payment_frozen_x_and_only_next_disaster_play() {
    let mut g=game(); let source=field(&mut g,"JC055",0); g.board_mut(&source).unwrap().time_markers=3;
    let c=g.make_card("BQ022",0); let attachment=c.id.clone(); g.attachments.push(Attachment { card:c,host_id:source.clone() });
    g.board_mut(&attachment).unwrap().time_markers=2;
    let enemy=field(&mut g,"JC125",2); g.board_mut(&enemy).unwrap().time_markers=5;
    let action=command("activate",&source); assert!(g.legal_actions(0).iter().any(|a| a.action.kind=="activate" && a.action.card_id.as_deref()==Some(&source)));
    act(&mut g,0,action);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(),5);
    assert!(g.view(2).pending_choice.is_none());
    let selected=vec![format!("{source}/time/0"),format!("{source}/time/1"),format!("{attachment}/time/0")];
    let request=choose_request(&g,selected);
    reject(&mut g,1,request.clone()); act(&mut g,0,request.clone()); reject(&mut g,0,request);
    assert_eq!(g.board(&source).unwrap().1.time_markers,1); assert_eq!(g.board(&attachment).unwrap().1.time_markers,1);
    let mut bad=g.clone(); if let PaidCost::JC055Time(ps)=&mut bad.stack.last_mut().unwrap().frame.as_mut().unwrap().already_paid[0] { ps[0].remaining=100; } rejected_state(&bad);
    g.board_mut(&source).unwrap().time_markers=0; top(&mut g);
    let disaster=g.make_card("JZ37",0); let ordinary=g.make_card("JC049",0);
    assert_eq!(g.effective_cost(0,&disaster),6); assert_eq!(g.paid_reveal_cost(0,&disaster),9);
    assert_eq!(g.effective_cost(0,&ordinary),catalog::card("JC049").cost);
    fund(&mut g,0,"JC047",9); let id=held(&mut g,"JZ37",0); act(&mut g,0,command("play",&id));
    assert_eq!(g.players[0].assets.iter().filter(|c| !c.exhausted).count(),3); assert_eq!(g.effective_cost(0,&disaster),9);
    let mut bad=g.clone(); bad.modifiers[0].paid_reveal=true; rejected_state(&bad);
}

#[test]
fn next20_red_jc055_zero_payment_and_expiry_and_payload_rejection() {
    let mut g=game(); let id=field(&mut g,"JC055",0);
    reject(&mut g,0,Action { cost_selected:Some(vec![]), ..command("activate",&id) });
    act(&mut g,0,command("activate",&id)); assert!(g.pending.as_ref().unwrap().choice.options.is_empty());
    choose(&mut g,vec![]); top(&mut g); assert!(g.modifiers.is_empty());
    g.board_mut(&id).unwrap().time_markers=1; act(&mut g,0,command("activate",&id)); choose(&mut g,vec![format!("{id}/time/0")]); top(&mut g);
    assert_eq!(g.modifiers.len(),1); let previous_turn=g.turn; g.begin_window(Window::End);
    for _ in 0..16 {
        if g.turn != previous_turn { break; }
        if g.pending.is_some() { resolve_choice(&mut g); } else { pass(&mut g); }
    }
    assert!(g.turn > previous_turn); assert!(g.modifiers.is_empty());
}

#[test]
fn next20_red_jz32_sacrifice_cost_grave_choice_and_original_region() {
    let mut g=game(); let source=field(&mut g,"JZ32",0); fund(&mut g,0,"JC047",1);
    let hourglass=g.make_card("JC048",0); let graveid=hourglass.id.clone(); g.players[0].graveyard.push(hourglass);
    act(&mut g,0,command("activate",&source)); assert!(g.board(&source).is_none());
    assert_eq!(g.players[0].assets.iter().filter(|c| !c.exhausted).count(),0);
    let mut bad=g.clone(); bad.stack.last_mut().unwrap().frame.as_mut().unwrap().source.source_region_instance=None; rejected_state(&bad);
    until_choice(&mut g); assert!(matches!(g.pending.as_ref().unwrap().resolution,ChoiceResolution::Frame { choice:FrameChoice::JZ32Hourglass,.. }));
    let request=choose_request(&g,vec![graveid.clone()]); act(&mut g,0,request.clone()); reject(&mut g,0,request);
    let attached=g.attachments.iter().find(|a| a.card.definition=="JC048").unwrap(); assert_eq!(attached.host_id,g.regions[0].card.id); assert_ne!(attached.card.id,graveid);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition=="JZ32"));
}

#[test]
fn next20_red_jz32_replacement_region_does_not_receive_hourglass_and_icons_are_live() {
    let mut g=game(); let id=field(&mut g,"JZ32",0);
    assert!(!g.has_renown(g.board(&id).unwrap().1));
    g.regions[0].destroyed=true; assert!(g.has_renown(g.board(&id).unwrap().1)); assert_eq!(g.current_icons(g.board(&id).unwrap().1,0).influence,2);
    g.first_team=1; assert_eq!(g.current_icons(g.board(&id).unwrap().1,0).influence,2); g.first_team=0;
    g.regions[0].destroyed=false; assert!(!g.has_renown(g.board(&id).unwrap().1));
    fund(&mut g,0,"JC047",1); let hourglass=g.make_card("JC048",0); g.players[0].graveyard.push(hourglass);
    act(&mut g,0,command("activate",&id)); g.regions[0].card=g.make_card("DQJC115",0); top(&mut g);
    assert!(g.attachments.is_empty()); assert!(g.players[0].graveyard.iter().any(|c| c.definition=="JC048"));
}

#[test]
fn next20_red_jz34_public_entry_sacrifices_only_controlled_other_then_destroys() {
    let mut g=game(); let ally=field(&mut g,"JC125",0); let teammate=field(&mut g,"JC125",1); let enemy=field(&mut g,"JC125",2);
    fund(&mut g,0,"JC047",6); let source=held(&mut g,"JZ34",0);
    act(&mut g,0,Action { region:Some(0),..command("deploy",&source) }); until_choice(&mut g); choose(&mut g,vec!["accept".into()]);
    until_choice(&mut g); assert_eq!(g.pending.as_ref().unwrap().choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>(),vec![ally.clone()]);
    assert!(!g.regions[0].destroyed); choose(&mut g,vec![ally.clone()]); assert!(g.regions[0].destroyed);
    assert!(g.board(&ally).is_none()); assert!(g.board(&teammate).is_some()); assert!(g.board(&enemy).is_some());
    assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Declare { declaration,.. } if declaration.ability.event==Some(Event::JZ34OtherDeath)));
    choose(&mut g,vec!["region:0".into()]);
    assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().targets[0].region_instance,Some(g.regions[0].card.id.clone()));
    top(&mut g); assert_eq!(g.regions[0].influence,[1,0]);
}

#[test]
fn next20_red_jz34_no_sacrifice_still_destroys_and_death_target_freezes_region() {
    let mut g=game(); let source=field(&mut g,"JZ34",0);
    emit(&mut g,&source,Event::Enter); choose(&mut g,vec!["accept".into()]); top(&mut g); assert!(g.regions[0].destroyed);
    let enemy=field(&mut g,"JC125",2); g.remove_dead(&enemy,RemovalCause::Destroy); g.drive().unwrap(); restore(&mut g);
    choose(&mut g,vec!["region:0".into()]);
    let mut bad=g.clone(); bad.stack.last_mut().unwrap().frame.as_mut().unwrap().targets[0].region_instance=None; rejected_state(&bad);
    g.regions[0].card=g.make_card("DQJC115",0); g.regions[0].destroyed=true; top(&mut g); assert_eq!(g.regions[0].influence,[0,0]);
}

#[test]
fn next20_red_jz34_same_wave_death_snapshot_excludes_self_hidden_and_attachment() {
    let mut g=game(); let source=field(&mut g,"JZ34",0); let enemy=field(&mut g,"JC125",2); let hidden=field(&mut g,"JC125",3); g.board_mut(&hidden).unwrap().face_down=true;
    let a=g.make_card("BQ022",0); g.attachments.push(Attachment { card:a,host_id:source.clone() });
    fund(&mut g,0,"JC047",9); let spell=held(&mut g,"JZ37",0); act(&mut g,0,command("play",&spell)); until_choice(&mut g);
    assert!(g.board(&source).is_none()); assert!(g.board(&enemy).is_none()); assert!(g.board(&hidden).is_none());
    let p=g.pending.clone().unwrap();
    assert!(matches!(&p.resolution,ChoiceResolution::Declare { declaration,.. } if matches!(&declaration.ability.ops[0],Op::JZ34DeathInfluence { observation:Some(o) } if o.card.id==enemy)));
    let mut bad=g.clone(); if let ChoiceResolution::Declare { declaration,.. }=&p.resolution { bad.effects.push_back(Effect::Declare { declaration:declaration.clone() }); } rejected_state(&bad);
    choose(&mut g,vec!["region:0".into()]); top(&mut g); assert_eq!(g.regions[0].influence,[1,0]); assert!(g.pending.is_none());
}

#[test]
fn next20_red_jz41_public_attachment_forecast_private_restore_and_replay() {
    let mut g=game(); let host=field(&mut g,"JC051",0); fund(&mut g,0,"JC047",1); let id=held(&mut g,"JZ41",0);
    act(&mut g,0,Action { target_id:Some(host),..command("play",&id) }); until_choice(&mut g); choose(&mut g,vec!["accept".into()]); until_choice(&mut g);
    let p=g.pending.clone().unwrap(); assert!(matches!(p.resolution,ChoiceResolution::Frame { choice:FrameChoice::Forecast {..},.. }));
    assert_eq!(p.choice.options.len(),2); assert!(g.view(2).pending_choice.is_none());
    let mut bad=g.clone(); bad.pending.as_mut().unwrap().choice.options.reverse(); rejected_state(&bad);
    let ids=p.choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>();
    let a=Action { choice_id:Some(p.choice.id),top:Some(vec![ids[1].clone()]),bottom:Some(vec![ids[0].clone()]),..Action::new("choose") };
    act(&mut g,0,a.clone()); reject(&mut g,0,a); assert_eq!(g.players[0].deck[0].id,ids[1]); assert_eq!(g.players[0].deck.last().unwrap().id,ids[0]);
}

#[test]
fn next20_red_jz41_actual_damage_wounds_once_prevention_zero_and_lethal_identity() {
    let mut g=game(); let host=field(&mut g,"JC051",0); let a=g.make_card("JZ41",2); let ring=a.id.clone(); g.attachments.push(Attachment {card:a,host_id:host.clone()});
    g.damage(BTreeMap::from([(host.clone(),0)])).unwrap(); assert!(g.effects.is_empty());
    g.turn_attribute_modifiers.push(TurnAttributeModifier { target_instance:host.clone(),defense_bonus:0,kill_bonus:0,grants_retreat:false,printed_defense_override:None,ordinary_icons:Icons::default(),grants_renown:false,grants_barrier: false, prevents_damage:true,expires_turn:g.turn });
    g.damage(BTreeMap::from([(host.clone(),1)])).unwrap(); assert!(g.effects.is_empty()); g.turn_attribute_modifiers.clear();
    g.damage(BTreeMap::from([(host.clone(),1)])).unwrap(); g.drive().unwrap(); restore(&mut g);
    let mut bad=g.clone(); if let ChoiceResolution::Declare { declaration,.. }=&mut bad.pending.as_mut().unwrap().resolution { declaration.source.attachment_host_instance=Some("different".into()); } rejected_state(&bad);
    let barrier=g.make_card("JC073",0); g.attachments.push(Attachment { card:barrier,host_id:host.clone() });
    g.board_mut(&host).unwrap().shield=1;
    choose(&mut g,vec!["accept".into()]); top(&mut g); assert_eq!(g.board(&host).unwrap().1.wounds,1);
    assert_eq!(g.board(&host).unwrap().1.shield,1,"该角色 is not a new target and does not consume a shield");
    assert!(g.pending.is_none()); assert!(g.effects.is_empty());
    g.damage(BTreeMap::from([(host.clone(),10)])).unwrap(); assert!(g.board(&host).is_none()); assert!(g.board(&ring).is_none()); g.drive().unwrap(); restore(&mut g);
    let replacement=field(&mut g,"JC051",0); choose(&mut g,vec!["accept".into()]); top(&mut g); assert_eq!(g.board(&replacement).unwrap().1.wounds,0);
}

// Cross-package validation uses both real badge definitions and public commands.
#[test]
fn next20_red_jc051_cross_package_badge_public_search_host_restore() {
    for badge_id in ["JC039","XQ39"] { for with_society in [false,true] {
        assert!(catalog::card(badge_id).subtypes.iter().any(|s| s=="徽章"));
        let mut g=game(); g.players[0].society_zone.card=with_society.then(||g.make_card("MSJC04",0)); let source=field(&mut g,"JC051",0); g.board_mut(&source).unwrap().face_down=true;
        fund(&mut g,0,"JC047",6); let badge=g.make_card(badge_id,0); let badge_instance=badge.id.clone(); g.players[0].deck.push(badge);
        act(&mut g,0,command("reveal",&source)); until_choice(&mut g); choose(&mut g,vec!["accept".into()]); until_choice(&mut g);
        assert!(g.view(2).pending_choice.is_none());
        let mut expected=g.clone(); expected.players[0].deck.retain(|c| c.id!=badge_instance); expected.shuffle_player(0);
        let request=choose_request(&g,vec![badge_instance.clone()]); act(&mut g,0,request.clone()); reject(&mut g,0,request);
        if matches!(g.pending.as_ref().map(|p| &p.resolution),Some(ChoiceResolution::Frame { choice:FrameChoice::JC051Host {..},.. })) {
            let host=g.pending.as_ref().unwrap().choice.options[0].id.clone(); choose(&mut g,vec![host]);
        }
        assert!(g.attachments.iter().any(|a| a.card.definition==badge_id && a.card.id!=badge_instance && a.host_id=="society:p0"));
        assert_eq!(g.players[0].deck,expected.players[0].deck); assert_eq!(g.random,expected.random,"badge search shuffles exactly once");
        assert!(!g.players[0].deck.iter().any(|c| c.id==badge_instance)); restore(&mut g);
    }}
}
