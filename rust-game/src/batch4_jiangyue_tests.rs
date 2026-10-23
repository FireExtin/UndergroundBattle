//! Normal Native play/Prepare/search/combat and Room routes from disclosed layouts.
use crate::{catalog, model::*, rules::*, unit_support::*};
use crate::jc029_tests::{envelope,pass_top};
use std::collections::BTreeMap;
const KEY:&str=crate::batch4_jiangyue::KEY;
fn funds(g:&mut Game) {fund(g,0,"JC029",2);fund(g,0,"JC070",2);fund(g,0,"JZ50",2);}
fn begin(g:&mut Game) {g.turn+=1;g.begin_window(Window::Prepare);g.drive().unwrap();restore(g);}
fn prepare(g:&mut Game,source:&str,region:usize) {
    begin(g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,"jc035-preparation-order");choose(g,vec![source.into()]);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"lc16-preparation-target");choose(g,vec![format!("region:{region}")]);
}
fn search(g:&mut Game) {pass_top(g);assert_eq!(g.pending.as_ref().unwrap().choice.kind,crate::batch4_jiangyue::KIND);}
fn battle(g:&mut Game) {g.begin_window(Window::Before(0,1));g.close_window().unwrap();g.drive().unwrap();restore(g);}
fn allocate(g:&mut Game,id:&str,amount:u32) {let p=g.pending.as_ref().unwrap();act(g,p.seat,Action{choice_id:Some(p.choice.id.clone()),allocations:Some(BTreeMap::from([(id.into(),amount)])),..Action::new("choose")});}

#[test]
fn batch4_jiangyue_exact_black_printed_fields_unique_and_domain_loyalty() {
    let c=catalog::card("LC16");assert_eq!((&*c.name,c.cost,&*c.color,c.defense),("江月",6,"黑",Some(2)));assert_eq!(c.subtitle.as_deref(),Some("狐之影"));
    assert_eq!(c.loyalty,["鲜血","神圣","死亡"]);assert_eq!(c.subtypes,["妖","阴"]);assert!(c.unique&&!c.supported);assert_eq!(c.permanent_icons.combat,3);
    let d=definition("LC16");assert_eq!(d.traits.wound,2);assert_eq!(d.traits.kill,0);assert_eq!(d.abilities[0].event,Some(Event::Prepare));
    let mut defs=crate::rules::definitions().clone();defs.get_mut("JC125").unwrap().traits.wound=2;assert!(crate::rules::validate_definitions(&defs).is_err());
}
#[test]
fn batch4_jiangyue_paid_deploy_requires_all_three_domains_and_six_assets() {
    for omit in 0..3 {let mut g=game();for (i,id) in ["JC029","JC070","JZ50"].into_iter().enumerate(){fund(&mut g,0,if i==omit{"JC125"}else{id},2);}let c=held(&mut g,"LC16",0);
        reject(&mut g,0,Action{card_id:Some(c),region:Some(0),..Action::new("deploy")});assert_eq!(g.resources(0),6);}
    let mut g=game();funds(&mut g);let c=held(&mut g,"LC16",0);act(&mut g,0,Action{card_id:Some(c),region:Some(0),..Action::new("deploy")});assert_eq!(g.resources(0),0);pass_top(&mut g);
    let c=g.regions[0].cards.iter().find(|c|c.definition=="LC16").unwrap();assert_eq!(g.effective_wound(c),2);assert_eq!(g.effective_kill(c),0);
    for seat in 0..4 {let v=g.view(seat);let c=v.regions[0].characters.iter().find(|c|c.card_id.as_deref()==Some("LC16")).unwrap();assert_eq!(c.current_wound,Some(2));assert!(c.current_kill.is_none());}
}
#[test]
fn batch4_jiangyue_hidden_does_not_reveal_wound_or_prepare_until_real_paid_reveal() {
    let mut g=game();funds(&mut g);fund(&mut g,0,"JC125",2);let hand=held(&mut g,"LC16",0);act(&mut g,0,Action{card_id:Some(hand),region:Some(0),..Action::new("conceal")});
    let hidden=g.regions[0].cards.iter().find(|c|c.definition=="LC16").unwrap().id.clone();assert_eq!(g.effective_wound(g.board(&hidden).unwrap().1),0);
    for seat in 0..4 {let v=g.view(seat);assert!(v.regions[0].characters.iter().find(|c|c.instance_id==hidden).unwrap().current_wound.is_none());}
    act(&mut g,0,Action{card_id:Some(hidden),..Action::new("reveal")});pass_top(&mut g);let c=g.regions[0].cards.iter().find(|c|c.definition=="LC16").unwrap();assert!(!c.face_down);assert_eq!(g.effective_wound(c),2);
}
#[test]
fn batch4_jiangyue_prepare_private_spell_or_yin_search_shuffles_then_influences_chosen_region() {
    let mut g=game();let source=field(&mut g,"LC16",0);let ids=deck_cards(&mut g,0,&["JC102","LC16","JC125","XQ21","JC091"]);prepare(&mut g,&source,1);search(&mut g);
    let p=g.pending.as_ref().unwrap();let opts=p.choice.options.iter().map(|o|o.id.clone()).collect::<Vec<_>>();assert_eq!(opts,vec![ids[0].clone(),ids[1].clone()]);
    assert_eq!(g.regions[1].influence[0],0);assert_eq!(p.choice.min,Some(0));assert_eq!(p.choice.allow_decline,Some(false));
    for seat in 1..4 {assert!(g.view(seat).pending_choice.is_none());}assert_eq!(g.view(0).pending_choice.unwrap().options.len(),2);
    let random=g.random;choose(&mut g,vec![ids[1].clone()]);assert_eq!(g.players[0].deck.len(),4);assert_ne!(g.random,random);assert!(!g.players[0].deck.iter().any(|c|c.id==ids[1]));
    assert!(g.players[0].graveyard.iter().any(|c|c.definition=="LC16"&&c.id!=ids[1]));assert_eq!(g.regions[1].influence[0],1);assert_eq!(g.regions[0].influence[0],0);
}
#[test]
fn batch4_jiangyue_empty_search_and_zero_selection_still_shuffle_and_place_one() {
    for hit in [false,true] {let mut g=game();let source=field(&mut g,"LC16",0);deck_cards(&mut g,0,if hit{&["JC102","JC125","XQ21"]}else{&["JC125","XQ21","JC125"]});let ids=g.players[0].deck.iter().map(|c|c.id.clone()).collect::<std::collections::BTreeSet<_>>();
        prepare(&mut g,&source,2);search(&mut g);assert_eq!(g.pending.as_ref().unwrap().choice.options.len(),usize::from(hit));let random=g.random;choose(&mut g,vec![]);
        assert_eq!(g.players[0].deck.iter().map(|c|c.id.clone()).collect::<std::collections::BTreeSet<_>>(),ids);assert!(g.players[0].graveyard.is_empty());assert_ne!(g.random,random);assert_eq!(g.regions[2].influence[0],1);}
}
#[test]
fn batch4_jiangyue_declining_prepare_does_not_search_shuffle_or_place_influence() {
    let mut g=game();let source=field(&mut g,"LC16",0);let ids=deck_cards(&mut g,0,&["JC102","LC16","JC125"]);begin(&mut g);let random=g.random;choose(&mut g,vec![]);
    assert_eq!(g.random,random);assert_eq!(g.players[0].deck.iter().map(|c|c.id.clone()).collect::<Vec<_>>(),ids);assert!(g.regions.iter().all(|r|r.influence==[0,0]));
    reject(&mut g,0,Action{card_id:Some(source),ability_id:Some(KEY.into()),..Action::new("activate")});
}
#[test]
fn batch4_jiangyue_wrong_search_card_player_extra_payload_and_repeated_choice_are_atomic() {
    let mut g=game();let source=field(&mut g,"LC16",0);let ids=deck_cards(&mut g,0,&["JC102","JC125","XQ21"]);let foreign=deck_cards(&mut g,2,&["JC102"])[0].clone();prepare(&mut g,&source,1);search(&mut g);let id=g.pending.as_ref().unwrap().choice.id.clone();
    for selected in [vec![ids[1].clone()],vec![ids[2].clone()],vec![foreign],vec![ids[0].clone(),ids[0].clone()]]{reject(&mut g,0,Action{choice_id:Some(id.clone()),selected:Some(selected),..Action::new("choose")});}
    reject(&mut g,2,Action{choice_id:Some(id.clone()),selected:Some(vec![ids[0].clone()]),..Action::new("choose")});
    reject(&mut g,0,Action{choice_id:Some(id.clone()),selected:Some(vec![ids[0].clone()]),region:Some(2),..Action::new("choose")});
    choose(&mut g,vec![ids[0].clone()]);reject(&mut g,0,Action{choice_id:Some(id),selected:Some(vec![ids[0].clone()]),..Action::new("choose")});assert_eq!(g.regions[1].influence[0],1);assert_eq!(g.players[0].graveyard.len(),1);
}
#[test]
fn batch4_jiangyue_real_source_rescue_preserves_original_actor_and_targeted_preparation() {
    let mut g=game();let source=field(&mut g,"LC16",0);let rescuer=field(&mut g,"JC075",0);fund(&mut g,0,"JC075",2);let ids=deck_cards(&mut g,0,&["JC102","JC125"]);prepare(&mut g,&source,1);
    for _ in 0..4 {if g.can_fast(0){break;}pass(&mut g);}let a=g.legal_actions(0).into_iter().find(|v|v.action.card_id.as_deref()==Some(&rescuer)&&v.action.target_id.as_deref()==Some(&source)).unwrap().action;
    act(&mut g,0,a);pass_top(&mut g);assert!(g.in_play(&source).is_none());search(&mut g);choose(&mut g,vec![ids[0].clone()]);assert_eq!(g.regions[1].influence[0],1);
}
#[test]
fn batch4_jiangyue_replaced_target_region_cancels_whole_search_before_private_choice() {
    let mut g=game();let source=field(&mut g,"LC16",0);let ids=deck_cards(&mut g,0,&["JC102","JC125"]);prepare(&mut g,&source,1);
    // Disclosed adjacent identity replacement probe, not a fabricated card action.
    g.regions[1].card=g.make_card("DQJC115",0);restore(&mut g);pass_top(&mut g);
    assert!(g.pending.is_none());assert_eq!(g.players[0].deck.iter().map(|c|c.id.clone()).collect::<Vec<_>>(),ids);assert_eq!(g.regions[1].influence[0],0);
}
#[test]
fn batch4_jiangyue_wound_two_uses_equal_replacement_without_extra_damage_or_multiplier() {
    let mut g=game();let source=field(&mut g,"LC16",0);let victim=field(&mut g,"LC01",2);battle(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.amount,Some(3));assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Damage{wound_sources,..}if wound_sources==&[source]));
    allocate(&mut g,&victim,3);let c=g.board(&victim).unwrap().1;assert_eq!((c.damage,c.wounds,g.defense(c,0)),(0,3,1));
}
#[test]
fn batch4_jiangyue_teammate_recipient_does_not_inherit_another_seats_wound() {
    let mut g=game();field(&mut g,"LC16",0);field(&mut g,"JC014",1);let a=field(&mut g,"LC01",2);let b=field(&mut g,"LC01",3);battle(&mut g);let amount=g.pending.as_ref().unwrap().choice.amount.unwrap();assert_eq!(amount,5);
    choose(&mut g,vec!["p1".into()]);assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Damage{wound_sources,..}if wound_sources.is_empty()));
    let id=g.pending.as_ref().unwrap().choice.id.clone();act(&mut g,1,Action{choice_id:Some(id),allocations:Some(BTreeMap::from([(a.clone(),2),(b.clone(),3)])),..Action::new("choose")});assert_eq!((g.board(&a).unwrap().1.damage,g.board(&a).unwrap().1.wounds),(2,0));
}
#[test]
fn batch4_jiangyue_nonparticipating_wound_does_not_replace_another_characters_reward() {
    let mut g=game();let source=field(&mut g,"LC16",0);g.board_mut(&source).unwrap().exhausted=true;field(&mut g,"JC016",0);let victim=field(&mut g,"LC01",2);battle(&mut g);let amount=g.pending.as_ref().unwrap().choice.amount.unwrap();
    assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Damage{wound_sources,..}if wound_sources.is_empty()));allocate(&mut g,&victim,amount);assert_eq!(g.board(&victim).unwrap().1.wounds,0);
}
#[test]
fn batch4_jiangyue_room_private_search_pause_resume_and_duplicate_keep_one_result() {
    use crate::room::{RoomCommand,RoomEnvelope,SessionAction};
    let mut g=game();let source=field(&mut g,"LC16",0);let ids=deck_cards(&mut g,0,&["JC102","JC125","XQ21"]);g.seed=u64::MAX-97;g.random=g.seed;prepare(&mut g,&source,1);search(&mut g);let mut r=envelope(&g);
    for (n,a) in [SessionAction::PauseRoom,SessionAction::ResumeRoom].into_iter().enumerate(){let c=RoomCommand{command_id:format!("lc16-session-{n}"),expected_version:r.revision,action:a};let out=r.transition(n,Some(c),1000+n as u64).unwrap();assert!(out.error_code.is_none(),"{:?}",out.error_message);r=RoomEnvelope::from_persisted(&out.state).unwrap();}
    let p=r.game.pending.as_ref().unwrap();let cmd=RoomCommand{command_id:"lc16-search-normal".into(),expected_version:r.revision,action:SessionAction::Game{action:Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![ids[0].clone()]),..Action::new("choose")}}};
    let out=r.transition(0,Some(cmd.clone()),2000).unwrap();assert!(out.error_code.is_none(),"{:?}",out.error_message);let next=RoomEnvelope::from_persisted(&out.state).unwrap();assert_eq!(serde_json::to_value(r.replay_events(&out.journal).unwrap()).unwrap(),serde_json::to_value(&next).unwrap());
    let repeat=next.transition(0,Some(cmd),2001).unwrap();assert_eq!(repeat.error_code.as_deref(),Some("version_conflict"));assert_eq!(repeat.state,out.state);assert_eq!(next.game.seed,u64::MAX-97);assert_eq!(next.game.regions[1].influence[0],1);assert_eq!(next.game.players[0].graveyard.len(),1);
}
#[test]
fn batch4_jiangyue_receives_and_resolves_normal_granted_renown_without_losing_its_wound() {
    let mut g=game();let source=field(&mut g,"LC16",0);fund(&mut g,0,"JC074",catalog::card("JC074").cost.max(1) as usize);let spell=held(&mut g,"JC074",0);
    act(&mut g,0,Action{card_id:Some(spell),target_id:Some(source.clone()),..Action::new("play")});pass_top(&mut g);assert!(g.has_renown(g.board(&source).unwrap().1));assert_eq!(g.effective_wound(g.board(&source).unwrap().1),2);
    g.begin_window(Window::Before(0,2));g.close_window().unwrap();g.drive().unwrap();restore(&mut g);
    for _ in 0..32 {if g.pending.is_some(){break;}pass(&mut g);}
    let p=g.pending.as_ref().unwrap();assert!(matches!(&p.resolution,ChoiceResolution::Declare{declaration,..} if declaration.ability.key=="renown"));
    choose(&mut g,vec!["accept".into()]);pass_top(&mut g);assert_eq!(g.regions[0].influence[0],2);assert_eq!(g.effective_wound(g.board(&source).unwrap().1),2);
}

// Independent normal-operation probes on fixed a0b7e49d. The starting board
// layouts are explicit; no corrupted persisted-state input is introduced.
mod independent_normal_review {
    use super::*;
    fn record(label: &str, g: &Game) {
        if let Ok(dir) = std::env::var("INDEPENDENT_HISTORY_OUT") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(format!("{dir}/{label}.json"), serde_json::to_vec_pretty(
                &serde_json::json!({"game":g,"views":(0..4).map(|s|g.view(s)).collect::<Vec<_>>()})
            ).unwrap()).unwrap();
        }
    }
    fn advance_until(g: &mut Game, label: &str, done: impl Fn(&Game) -> bool) {
        for _ in 0..256 {
            if done(g) { record(label, g); return; }
            assert_eq!(g.status, "playing", "{label}");
            if g.pending.is_some() { resolve_choice(g); } else { pass(g); }
        }
        panic!("normal continuation did not reach {label}");
    }
    #[test]
    fn independent_lc16_search_supplies_later_forced_upkeep_in_the_same_prepare_stack() {
        let mut g = game();
        let jiang = field(&mut g, "LC16", 0);
        let ghoul = field(&mut g, "BQ079", 0);
        let deck = deck_cards(&mut g, 0, &["JC102", "JC125", "XQ21"]);
        begin(&mut g);
        let p = g.pending.as_ref().unwrap();
        assert_eq!(p.choice.min, Some(1));
        let invalid_decline = Action { choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),..Action::new("choose") };
        reject(&mut g, 0, invalid_decline);
        choose(&mut g, vec![ghoul.clone()]);
        choose(&mut g, vec![jiang.clone()]);
        choose(&mut g, vec!["region:1".into()]);
        assert_eq!(g.stack.len(), 2);
        assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().source.card.id, jiang);
        search(&mut g);
        assert!(g.players[0].graveyard.is_empty());
        choose(&mut g, vec![deck[0].clone()]);
        assert_eq!(g.regions[1].influence[0], 1);
        let fresh = g.players[0].graveyard.iter().find(|c|c.definition=="JC102").unwrap().id.clone();
        record("search-before-forced-upkeep", &g);
        pass_top(&mut g);
        assert!(matches!(&g.pending.as_ref().unwrap().resolution,
            ChoiceResolution::Frame { choice:FrameChoice::BQ079Upkeep,.. }));
        choose(&mut g, vec![fresh]);
        assert!(g.in_play(&ghoul).is_some());
        assert!(g.players[0].graveyard.is_empty());
        assert_eq!(g.players[0].deck.last().unwrap().definition, "JC102");
        assert_eq!(g.regions[1].influence[0], 1);
        assert_eq!(g.jc035_preparation.as_ref().unwrap().entries.iter().filter(|e|
            matches!(e.decision,Some(crate::jc035::PreparationDecision::Resolved {..}))).count(), 2);
        record("search-and-forced-upkeep-complete", &g);
    }
    #[test]
    fn independent_lc16_forced_prepare_damage_kills_source_but_its_queued_search_resolves() {
        let mut g = game();
        let jiang = field(&mut g, "LC16", 0);
        fund(&mut g, 0, "JC048", 2);
        let hand_clock = held(&mut g, "JC048", 0);
        let a = g.legal_actions(0).into_iter().find(|v|
            v.action.card_id.as_deref()==Some(hand_clock.as_str()) && v.action.region==Some(0)
            && v.action.kind=="play").unwrap().action;
        act(&mut g, 0, a); pass_top(&mut g);
        let clock = g.attachments.iter().find(|a|a.card.definition=="JC048").unwrap().card.id.clone();
        // One marker is an explicit reachable prior-turn layout.
        g.board_mut(&clock).unwrap().time_markers = 1;
        let deck = deck_cards(&mut g, 0, &["JC102", "JC125", "XQ21"]);
        begin(&mut g);
        choose(&mut g, vec![jiang.clone()]);
        choose(&mut g, vec!["region:1".into()]);
        assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(1));
        choose(&mut g, vec![clock.clone()]);
        assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().source.card.id, clock);
        pass_top(&mut g);
        assert!(g.in_play(&jiang).is_none());
        assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="LC16").count(), 1);
        record("forced-prepare-source-dead", &g);
        search(&mut g);
        choose(&mut g, vec![deck[0].clone()]);
        assert_eq!(g.regions[1].influence[0], 1);
        assert_eq!(g.players[0].graveyard.iter().filter(|c|c.definition=="JC102").count(), 1);
        record("queued-search-after-real-source-death", &g);
    }
    #[test]
    fn independent_lc16_reveal_during_prepare_waits_for_next_actual_prepare_event() {
        let mut g = game(); funds(&mut g);
        let jiang = field(&mut g, "LC16", 0);
        g.board_mut(&jiang).unwrap().face_down = true;
        let annie = field(&mut g, "JC055", 0);
        begin(&mut g);
        let turn = g.turn;
        assert_eq!(g.jc035_preparation.as_ref().unwrap().entries.len(), 1);
        choose(&mut g, vec![annie.clone()]);
        choose(&mut g, vec![annie.clone()]);
        for _ in 0..8 { if g.can_fast(0) { break; } pass(&mut g); }
        assert!(g.can_fast(0));
        act(&mut g, 0, Action {card_id:Some(jiang),..Action::new("reveal")});
        pass_top(&mut g);
        assert_eq!(g.resources(0), 0);
        assert!(g.jc035_preparation.as_ref().unwrap().entries.iter().all(|e|e.source.card.definition!="LC16"));
        pass_top(&mut g);
        assert!(g.pending.is_none());
        record("late-reveal-no-retroactive-prepare", &g);
        advance_until(&mut g, "late-reveal-next-natural-prepare", |g|
            g.turn==turn+1 && g.window==Some(Window::Prepare) && g.pending.is_some());
        assert!(g.jc035_preparation.as_ref().unwrap().entries.iter().any(|e|e.source.card.definition=="LC16"));
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "jc035-preparation-order");
        assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(0));
    }
    #[test]
    fn independent_lc16_actual_combat_wounds_survive_cleanup_then_move_by_paid_xq20() {
        let mut g = game();
        let jiang = field(&mut g, "LC16", 0);
        let carrier = field(&mut g, "LC01", 2);
        let fragile = field(&mut g, "JC125", 2);
        g.board_mut(&fragile).unwrap().exhausted = true;
        fund(&mut g, 0, "XQ20", 2);
        let spell = held(&mut g, "XQ20", 0);
        battle(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().choice.amount, Some(3));
        allocate(&mut g, &carrier, 3);
        assert_eq!((g.board(&carrier).unwrap().1.damage,g.board(&carrier).unwrap().1.wounds),(0,3));
        let turn = g.turn;
        record("combat-created-three-wounds", &g);
        advance_until(&mut g, "next-natural-action-before-xq20", |g|
            g.turn==turn+1 && g.window==Some(Window::Action(0)) && g.pending.is_none() && g.stack.is_empty());
        assert_eq!(g.board(&carrier).unwrap().1.wounds, 3);
        let a = Action {card_id:Some(spell),ability_id:Some(crate::batch4_soul_channel::KEY.into()),
            selected:Some(vec![carrier.clone(),fragile.clone()]),..Action::new("play")};
        assert!(g.legal_actions(0).iter().any(|v|v.action==a));
        act(&mut g, 0, a); pass_top(&mut g);
        assert_eq!((g.board(&carrier).unwrap().1.damage,g.board(&carrier).unwrap().1.wounds),(0,0));
        assert!(g.in_play(&fragile).is_none());
        assert_eq!(g.players[2].graveyard.iter().filter(|c|c.definition=="JC125").count(), 1);
        assert_eq!(g.effective_wound(g.board(&jiang).unwrap().1), 2);
        assert_eq!(g.resources(0), 0);
        record("combat-wounds-moved-by-paid-xq20", &g);
    }
}
