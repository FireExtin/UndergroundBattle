//! Explicit native layouts with real payments, choices and response commands.
use crate::{batch4_night::{self as n, UndeadKind}, catalog, model::*, rules::*, unit_support::*};
use crate::jc029_tests::{checkpoint, pass_top};

fn play(g: &mut Game, seat: usize) -> String {
    g.begin_window(Window::Action(g.team(seat)));
    fund(g, seat, "JC084", 2);
    let hand = held(g, "JZ56", seat);
    act(g, seat, Action { card_id: Some(hand), target_id: Some(format!("society:p{seat}")), ..Action::new("play") });
    assert_eq!(g.resources(seat), 0);
    checkpoint(g);
    pass_top(g);
    let id = g.attachments.iter().find(|a| a.card.definition == "JZ56" && a.card.controller == seat && a.card.chosen_subtype.is_none()).unwrap().card.id.clone();
    assert!(matches!(&g.pending.as_ref().unwrap().resolution, ChoiceResolution::Declare { declaration, .. } if declaration.ability.key == n::KEY));
    choose(g, vec!["accept".into()]);
    checkpoint(g);
    pass_top(g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, n::KEY);
    checkpoint(g);
    id
}
fn night(g: &mut Game, seat: usize, kind: &str) -> String {
    let id = play(g, seat); choose(g, vec![kind.into()]); checkpoint(g); id
}
fn dead(g: &mut Game, seat: usize, count: usize) {
    for _ in 0..count { let c = g.make_card("JC125", seat); g.players[seat].graveyard.push(c); }
}
fn invalid(g: &Game, bad: &Game) {
    let raw = serde_json::to_string(bad).unwrap();
    assert!(std::panic::catch_unwind(|| Game::from_persisted(&raw)).unwrap().is_err());
    let mut r = serde_json::to_value(crate::jc029_tests::envelope(g)).unwrap();
    r["game"] = serde_json::to_value(bad).unwrap();
    assert!(std::panic::catch_unwind(|| crate::room::RoomEnvelope::from_persisted(&r.to_string())).unwrap().is_err());
}

#[test]
fn batch4_night_original_cost_loyalty_full_program_and_closed_deck_gate() {
    let c = catalog::card("JZ56");
    assert_eq!((c.cost, c.defense, c.unique, c.supported), (2, None, false, false));
    assert_eq!(c.loyalty, ["黑色"]); assert_eq!(c.subtypes, ["仪式"]); assert_eq!(c.magic, "死亡");
    assert!(catalog::runtime_implemented(c));
    let draft = crate::deck::DeckDraft { id: "night".into(), name: "native only".into(), description: String::new(), society_id: None, core_card_id:None,
        cards: vec![catalog::DeckEntry { card_id: "JC125".into(), count: 49 }, catalog::DeckEntry { card_id: "JZ56".into(), count: 1 }],
        rules_version: catalog::RULES_VERSION.into(), card_pool_version: catalog::POOL_VERSION.into(), engine_version: catalog::ENGINE_VERSION.into(), updated_at: "2026-10-23".into() };
    assert!(crate::deck::validate(draft).is_err());
    let d = definition("JZ56"); assert_eq!(d.abilities.len(), 2);
    assert_eq!(d.abilities[1].event, Some(Event::Enter));
    assert!(serde_json::to_value(&d.modifiers).unwrap().to_string().contains("JZ56ChosenUndeadAura"));
    let mut g = game(); let id = play(&mut g, 0);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.iter().map(|o|o.id.as_str()).collect::<Vec<_>>(), ["ghoul","zombie","skeleton"]);
    choose(&mut g, vec!["ghoul".into()]);
    assert_eq!(g.attachments.iter().find(|a|a.card.id==id).unwrap().card.chosen_subtype, Some(UndeadKind::Ghoul));
    checkpoint(&g);
}

#[test]
fn batch4_night_three_subtypes_global_personal_bonuses_and_live_grave_threshold() {
    for (choice, card) in [("ghoul","JZ50"),("zombie","JC085"),("skeleton","BQ083")] {
        let mut g = game();
        let personal = field(&mut g, card, 0); let ally = field(&mut g, card, 1); let enemy = field(&mut g, card, 2);
        let other = field(&mut g, "JC125", 0);
        let remote = field(&mut g, card, 0); let moved = g.regions[0].cards.pop().unwrap(); assert_eq!(moved.id, remote); g.regions[1].cards.push(moved);
        let original = g.board(&personal).unwrap().1.clone(); let base_icons = g.current_icons(&original, 0);
        let base_defense = g.defense(&original, 0); let base_kill = g.effective_kill(&original);
        dead(&mut g, 0, 6); let source = night(&mut g, 0, choice);
        for id in [&personal, &remote] { let (r,c)=g.board(id).unwrap(); assert_eq!(g.current_icons(c,r), base_icons.add(Icons{influence:1,..Icons::default()})); assert_eq!(g.defense(c,r),base_defense+1);assert_eq!(g.effective_kill(c),base_kill); }
        for id in [&ally,&enemy,&other] { assert_eq!(g.jz56_aura(g.board(id).unwrap().1),(0,0)); }
        dead(&mut g,0,1);
        let (r,c)=g.board(&personal).unwrap();assert_eq!(g.current_icons(c,r),base_icons.add(Icons{combat:1,influence:1,..Icons::default()}));assert_eq!(g.effective_kill(c),base_kill);assert_eq!(g.effective_wound(c),1);
        assert_eq!(g.current_permanent_combat(c,r),catalog::card(card).permanent_icons.combat+1);
        for viewer in 0..4 {
            let v=g.view(viewer);let a=v.attachments.iter().find(|a|a.card.instance_id==source).unwrap();
            assert_eq!(a.card.chosen_subtype.as_deref(),Some(match choice{"ghoul"=>"食尸鬼","zombie"=>"行尸",_=>"骷髅"}));
            let cv=&v.regions[0].characters.iter().find(|c|c.instance_id==personal).unwrap();
            assert_eq!(cv.current_kill,None);assert_eq!(cv.current_wound,Some(1));assert_eq!(cv.defense,Some(base_defense+1));
        }
        g.players[0].graveyard.pop();assert_eq!(g.jz56_aura(g.board(&personal).unwrap().1),(1,0));checkpoint(&g);
    }
}

#[test]
fn batch4_night_sources_stack_and_exhaustion_preserves_categories_not_participation() {
    let mut g=game();let target=field(&mut g,"JC085",0);dead(&mut g,0,7);
    let base_influence=g.current_icons(g.board(&target).unwrap().1,0).influence;
    let one=night(&mut g,0,"zombie");let two=night(&mut g,0,"zombie");
    assert_eq!(g.jz56_aura(g.board(&target).unwrap().1),(2,2));
    g.board_mut(&target).unwrap().exhausted=true;let (r,c)=g.board(&target).unwrap();
    assert_eq!(g.icons(c,r),Icons::default());assert_eq!(g.current_icons(c,r).influence,base_influence+2);
    g.remove_dead(&one,RemovalCause::Destroy);assert_eq!(g.jz56_aura(g.board(&target).unwrap().1),(1,1));
    g.remove_dead(&two,RemovalCause::Destroy);assert_eq!(g.jz56_aura(g.board(&target).unwrap().1),(0,0));
    assert!(g.players[0].graveyard.iter().filter(|c|c.definition=="JZ56").all(|c|c.chosen_subtype.is_none()));checkpoint(&g);
}

#[test]
fn batch4_night_current_controller_not_owner_or_teammate_and_hidden_blank_boundaries() {
    let mut g=game();let target=field(&mut g,"JC085",2);g.board_mut(&target).unwrap().controller=0;
    let source=night(&mut g,0,"zombie");assert_eq!(g.jz56_aura(g.board(&target).unwrap().1),(1,0));
    g.board_mut(&target).unwrap().controller=1;assert_eq!(g.jz56_aura(g.board(&target).unwrap().1),(0,0));
    g.board_mut(&target).unwrap().controller=0;g.board_mut(&target).unwrap().face_down=true;
    assert_eq!(g.jz56_aura(g.board(&target).unwrap().1),(0,0));
    let mut snapshot=g.attachments.iter().find(|a|a.card.id==source).unwrap().card.clone();
    for viewer in 0..4 {
        assert!(g.card_view(&snapshot,viewer,None,Some("asset")).chosen_subtype.is_none());
        assert!(g.sealed_card_view(&snapshot,viewer).chosen_subtype.is_none());
    }
    snapshot.face_down=true;for viewer in 0..4 {assert!(g.card_view(&snapshot,viewer,None,Some("attachment")).chosen_subtype.is_none());}
    checkpoint(&g);
}

#[test]
fn batch4_night_source_loss_removes_defense_before_existing_death_cleanup() {
    let mut g=game();let target=field(&mut g,"JC085",0);let source=night(&mut g,0,"zombie");
    g.board_mut(&target).unwrap().wounds=1;g.settle_deaths();assert!(g.board(&target).is_some());
    g.remove_dead(&source,RemovalCause::Destroy);g.settle_deaths();assert!(g.board(&target).is_none());checkpoint(&g);
}

#[test]
fn batch4_night_decline_entry_and_source_departure_do_not_rebind_replacement() {
    for after_accept in [false,true] {
        let mut g=game();fund(&mut g,0,"JC084",2);let h=held(&mut g,"JZ56",0);
        act(&mut g,0,Action{card_id:Some(h),target_id:Some("society:p0".into()),..Action::new("play")});pass_top(&mut g);
        let source=g.attachments.iter().find(|a|a.card.definition=="JZ56").unwrap().card.id.clone();
        if !after_accept {choose(&mut g,vec![]);assert!(g.attachments[0].card.chosen_subtype.is_none());}
        else {choose(&mut g,vec!["accept".into()]);g.remove_dead(&source,RemovalCause::Destroy);
            let replacement=g.make_card("JZ56",0);let replacement_id=replacement.id.clone();g.attachments.push(Attachment{card:replacement,host_id:"society:p0".into()});
            pass_top(&mut g);assert!(g.pending.is_none());assert!(g.attachments.iter().find(|a|a.card.id==replacement_id).unwrap().card.chosen_subtype.is_none());}
        checkpoint(&g);
    }
}

#[test]
fn batch4_night_choice_rejects_wrong_actor_bad_payload_duplicates_and_replay() {
    let mut g=game();let source=play(&mut g,0);let id=g.pending.as_ref().unwrap().choice.id.clone();
    let valid=Action{choice_id:Some(id),selected:Some(vec!["zombie".into()]),..Action::new("choose")};
    reject(&mut g,1,valid.clone());
    for selected in [vec![],vec!["vampire".into()],vec!["ghoul".into(),"zombie".into()],vec!["zombie".into(),"zombie".into()]] {reject(&mut g,0,Action{selected:Some(selected),..valid.clone()});}
    reject(&mut g,0,Action{region:Some(0),..valid.clone()});
    act(&mut g,0,valid.clone());assert_eq!(g.attachments.iter().find(|a|a.card.id==source).unwrap().card.chosen_subtype,Some(UndeadKind::Zombie));
    reject(&mut g,0,valid);checkpoint(&g);
}

#[test]
fn batch4_night_both_restore_entries_reject_corrupt_pending_frames_and_moved_choices() {
    let mut g=game();play(&mut g,0);
    for mode in 0..12 {
        let mut bad=g.clone();let p=bad.pending.as_mut().unwrap();
        match mode {
            0=>p.seat=1,1=>p.choice.options.reverse(),2=>p.choice.options.pop().map(|_|()).unwrap(),
            3=>p.choice.min=Some(0),4=>p.choice.allow_decline=Some(true),5=>p.choice.title="forged".into(),
            _=>{let ChoiceResolution::Frame{frame,choice}=&mut p.resolution else{panic!("frame")};match mode {
                6=>frame.cursor=0,7=>frame.source.attachment_host_instance=Some("society:p1".into()),
                8=>frame.source.card.definition="XQ15".into(),9=>frame.steps.push(frame.steps[0].clone()),
                10=>*choice=FrameChoice::XQ26Name,_=>frame.source.card.chosen_subtype=Some(UndeadKind::Zombie),
            }}
        }invalid(&g,&bad);
    }
    choose(&mut g,vec!["zombie".into()]);
    let mut bad=g.clone();let c=bad.make_card("JC085",0);bad.players[0].hand.push(Card{chosen_subtype:Some(UndeadKind::Zombie),..c});invalid(&g,&bad);
    let mut bad=g.clone();let i=bad.attachments.iter().position(|a|a.card.definition=="JZ56").unwrap();let c=bad.attachments.remove(i).card;bad.players[0].graveyard.push(c);invalid(&g,&bad);
    let mut raw=serde_json::to_value(&g).unwrap();raw["attachments"][0]["card"]["chosen_subtype"]=serde_json::json!("vampire");assert!(Game::from_persisted(&raw.to_string()).is_err());
}

#[test]
fn batch4_night_illegal_host_and_loyalty_never_pay() {
    let mut g=game();fund(&mut g,0,"JC125",2);let h=held(&mut g,"JZ56",0);
    let a=Action{card_id:Some(h.clone()),target_id:Some("society:p0".into()),..Action::new("play")};
    reject(&mut g,0,a.clone());fund(&mut g,0,"JC084",1);
    for target in ["society:p1","region:0",h.as_str()] {reject(&mut g,0,Action{target_id:Some(target.into()),..a.clone()});}
    reject(&mut g,0,Action{selected:Some(vec!["zombie".into()]),..a});assert_eq!(g.resources(0),3);
}

#[test]
fn batch4_night_enhanced_wound_uses_real_combat_marker_pipeline() {
    let mut g=game();let ghoul=field(&mut g,"JZ50",0);let target=field(&mut g,"JC125",2);
    dead(&mut g,0,7);night(&mut g,0,"ghoul");
    let c=g.board(&ghoul).unwrap().1;let expected=g.icons(c,0).combat+g.effective_kill(c);
    assert_eq!(g.effective_kill(c),0);assert_eq!(g.effective_wound(c),1);
    g.begin_window(Window::Before(0,1));g.close_window().unwrap();g.drive().unwrap();
    let p=g.pending.as_ref().unwrap();assert_eq!(p.choice.amount,Some(expected));
    let seat=p.seat;let a=Action{choice_id:Some(p.choice.id.clone()),allocations:Some(std::collections::BTreeMap::from([(target.clone(),expected)])),..Action::new("choose")};
    act(&mut g,seat,a);assert!(g.board(&target).is_none());checkpoint(&g);
}

#[test]
fn batch4_night_room_commands_choice_restore_four_views_and_duplicate_receipt() {
    use crate::room::{Decision,RoomEnvelope,RoomCommand,SessionAction};
    let mut g=game();fund(&mut g,0,"JC084",2);let hand=held(&mut g,"JZ56",0);
    let mut room=RoomEnvelope::from_game(g);
    let first=RoomCommand{command_id:"night-play".into(),expected_version:room.revision,action:SessionAction::Game{
        action:Action{card_id:Some(hand),target_id:Some("society:p0".into()),..Action::new("play")}}};
    let out=room.transition(0,Some(first.clone()),0).unwrap();assert_eq!(out.outcome,"accepted");room=RoomEnvelope::from_persisted(&out.state).unwrap();
    for i in 0..60 {
        if room.game.attachments.iter().any(|a|a.card.definition=="JZ56"&&a.card.chosen_subtype==Some(UndeadKind::Skeleton)){break;}
        let (seat,action)=if let Some(p)=&room.game.pending {
            let selected=match &p.resolution {
                ChoiceResolution::Declare{declaration,..}if declaration.ability.key==n::KEY=>vec!["accept".into()],
                ChoiceResolution::Frame{choice:FrameChoice::JZ56Undead,..}=>vec!["skeleton".into()],
                _=>panic!("unexpected night choice {}",p.choice.kind),
            };
            (p.seat,SessionAction::Game{action:Action{choice_id:Some(p.choice.id.clone()),selected:Some(selected),..Action::new("choose")}})
        }else if let Some(w)=&room.pacing.window {
            let seat=*w.members.iter().find(|(_,d)|matches!(d,Decision::Undecided{..})).unwrap().0;
            (seat,SessionAction::PassResponse{window_id:w.id.clone()})
        }else{let seat=(0..4).find(|s|room.game.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();(seat,SessionAction::Game{action:Action::new("pass")})};
        let out=room.transition(seat,Some(RoomCommand{command_id:format!("night-step-{i}"),expected_version:room.revision,action}),0).unwrap();
        assert_eq!(out.outcome,"accepted");let next=RoomEnvelope::from_persisted(&out.state).unwrap();
        assert_eq!(serde_json::to_value(room.replay_events(&out.journal).unwrap()).unwrap(),serde_json::to_value(&next).unwrap());
        for viewer in 0..4 {assert_eq!(serde_json::to_value(next.view(viewer,0)).unwrap(),serde_json::to_value(RoomEnvelope::from_persisted(&out.state).unwrap().view(viewer,0)).unwrap());}
        room=next;
    }
    assert_eq!(room.game.resources(0),0);
    let c=&room.game.attachments.iter().find(|a|a.card.definition=="JZ56").unwrap().card;
    assert_eq!(c.chosen_subtype,Some(UndeadKind::Skeleton));
    for viewer in 0..4 {assert_eq!(room.game.card_view(c,viewer,None,Some("attachment")).chosen_subtype.as_deref(),Some("骷髅"));}
    let saved=serde_json::to_value(&room).unwrap();let again=room.transition(0,Some(first),0).unwrap();
    assert_eq!(again.outcome,"rejected");assert!(!again.changed&&again.journal.is_empty());
    assert_eq!(serde_json::to_value(RoomEnvelope::from_persisted(&again.state).unwrap()).unwrap(),saved);
}

#[test]
fn batch4_night_finite_program_and_choice_cannot_be_transplanted() {
    let d=n::definition();assert!(crate::batch4::validate_definition("JC125",&d).is_err());
    let a=d.abilities[1].clone();assert!(crate::batch4::validate_ability("JC125",&a).is_err());
    let mut g=game();play(&mut g,0);
    let mut raw=serde_json::to_value(&g).unwrap();raw["attachments"][0]["card"]["chosen_subtype"]=serde_json::json!("vampire");
    let mut room=serde_json::to_value(crate::jc029_tests::envelope(&g)).unwrap();room["game"]=raw.clone();
    assert!(std::panic::catch_unwind(||Game::from_persisted(&raw.to_string())).unwrap().is_err());
    assert!(std::panic::catch_unwind(||crate::room::RoomEnvelope::from_persisted(&room.to_string())).unwrap().is_err());
}

// Independent review of 64420e55. Explicit layouts, production commands and
// both persisted entry points; these are not browser or full-game fixtures.
mod independent_review {
    use super::*;
    fn record(label: &str, g: &Game, extra: serde_json::Value) {
        if let Ok(dir) = std::env::var("INDEPENDENT_HISTORY_OUT") {
            std::fs::create_dir_all(&dir).unwrap();
            let room = crate::jc029_tests::envelope(g);
            let views = (0..4).map(|s| serde_json::to_value(g.view(s)).unwrap()).collect::<Vec<_>>();
            std::fs::write(format!("{dir}/{label}.json"), serde_json::to_vec_pretty(
                &serde_json::json!({"game":g,"room":room,"views":views,"observation":extra})).unwrap()).unwrap();
        }
    }
    fn cast(g: &mut Game, seat: usize, definition: &str, target: &str, option: Option<&str>) {
        let id = held(g, definition, seat);
        act(g, seat, Action { card_id: Some(id), target_id: Some(target.into()),
            option: option.map(str::to_string), ..Action::new("play") });
        pass_top(g);
        checkpoint(g);
    }
    fn enhanced_combat() -> (Game, String, u32) {
        let mut g = game();
        let ghoul = field(&mut g, "JZ50", 0);
        // LC01 has defense 4 and no combat icons: the wrong +1 damage does not
        // kill it, so damage and persistent wound are independently observable.
        let victim = field(&mut g, "LC01", 2);
        dead(&mut g, 0, 7);
        night(&mut g, 0, "ghoul");
        let score = g.icons(g.board(&ghoul).unwrap().1, 0).combat;
        assert_eq!(score, 2);
        g.begin_window(Window::Before(0, 1));
        g.close_window().unwrap(); g.drive().unwrap();
        checkpoint(&g);
        assert!(matches!(g.pending.as_ref().unwrap().resolution, ChoiceResolution::Damage { .. }));
        (g, victim, score)
    }

    #[test]
    fn independent_jz56_real_grave_play_and_spell_cross_threshold_at_the_correct_steps() {
        let mut g = game(); let target = field(&mut g, "JC085", 0);
        let enemy = field(&mut g, "LC01", 2);
        dead(&mut g, 0, 6); let c = g.make_card("JC085", 0); let grave_id = c.id.clone();
        g.players[0].graveyard.push(c);
        night(&mut g, 0, "zombie");
        assert_eq!(g.current_permanent_combat(g.board(&target).unwrap().1, 0), 2);
        fund(&mut g, 0, "JC084", 2);
        act(&mut g, 0, Action { card_id: Some(grave_id), region: Some(0), ..Action::new("deploy") });
        assert_eq!(g.players[0].graveyard.len(), 6);
        assert_eq!(g.current_permanent_combat(g.board(&target).unwrap().1, 0), 1);
        checkpoint(&g); record("grave-7-to-6-before-resolution", &g, serde_json::json!({"combat":1}));
        pass_top(&mut g); checkpoint(&g);
        fund(&mut g, 0, "JC102", 2);
        cast(&mut g, 0, "JC102", &enemy, None);
        assert_eq!(g.players[0].graveyard.len(), 7);
        assert_eq!(g.current_permanent_combat(g.board(&target).unwrap().1, 0), 2);
        assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 2);
        record("spell-6-to-7-after-resolution", &g, serde_json::json!({"combat":2}));
    }

    #[test]
    fn independent_jz56_live_added_skeleton_type_then_real_attachment_removal() {
        let mut g = game(); let human = field(&mut g, "JC125", 0);
        night(&mut g, 0, "skeleton");
        assert_eq!(g.defense(g.board(&human).unwrap().1, 0), 1);
        fund(&mut g, 0, "JZ57", 1);
        cast(&mut g, 0, "JZ57", &human, None);
        let tattoo = g.attachments.iter().find(|a| a.card.definition == "JZ57").unwrap().card.id.clone();
        assert_eq!(g.defense(g.board(&human).unwrap().1, 0), 3);
        assert_eq!(g.current_icons(g.board(&human).unwrap().1, 0).influence, 1);
        g.begin_window(Window::Action(1)); fund(&mut g, 2, "JC005", 2);
        cast(&mut g, 2, "JC005", &tattoo, None);
        assert_eq!(g.defense(g.board(&human).unwrap().1, 0), 1);
        assert_eq!(g.current_icons(g.board(&human).unwrap().1, 0).influence, 0);
        assert_eq!(g.current_subtypes(g.board(&human).unwrap().1), ["人类"]);
        record("dynamic-skeleton-grant-ended", &g, serde_json::json!({"defense":1}));
    }

    #[test]
    fn independent_jz56_real_hide_reveal_and_source_removal_keep_identity_and_recompute() {
        let mut g = game(); let old = field(&mut g, "JC085", 0);
        let source = night(&mut g, 0, "zombie"); dead(&mut g, 0, 7);
        fund(&mut g, 0, "JC063", 2);
        cast(&mut g, 0, "JC063", &old, Some("hide"));
        assert!(g.board(&old).is_none());
        let hidden = g.regions[0].cards.iter().find(|c| c.definition == "JC085").unwrap().clone();
        assert!(hidden.face_down); assert_eq!(g.jz56_aura(&hidden), (0,0));
        for s in 1..4 { let v = g.view(s); let c = v.regions[0].characters.iter().find(|c| c.instance_id == hidden.id).unwrap();
            assert!(c.current_subtypes.is_none() && c.current_kill.is_none() && c.chosen_subtype.is_none()); }
        assert_eq!(g.attachments.iter().find(|a| a.card.id == source).unwrap().card.chosen_subtype, Some(UndeadKind::Zombie));
        fund(&mut g, 0, "JC084", 2);
        act(&mut g, 0, Action { card_id: Some(hidden.id.clone()), ..Action::new("reveal") });
        pass_top(&mut g); checkpoint(&g);
        let shown = g.regions[0].cards.iter().find(|c| c.definition == "JC085").unwrap().clone();
        assert_ne!(shown.id, hidden.id); assert!(!shown.face_down);
        assert_eq!(g.current_permanent_combat(&shown, 0), 2); assert_eq!(g.defense(&shown,0),2);
        g.begin_window(Window::Action(1)); fund(&mut g, 2, "JC005", 2); fund(&mut g, 2, "JZ02", 1);
        cast(&mut g, 2, "JC005", &source, None);
        assert_eq!(g.current_permanent_combat(g.board(&shown.id).unwrap().1, 0), 1);
        assert_eq!(g.defense(g.board(&shown.id).unwrap().1, 0), 1);
        assert!(g.players[0].graveyard.iter().filter(|c| c.definition == "JZ56").all(|c| c.chosen_subtype.is_none()));
        record("hide-reveal-source-removal", &g, serde_json::json!({"formerTarget":old,"hiddenTarget":hidden.id,"currentTarget":shown.id}));
    }

    #[test]
    fn independent_jz56_actual_response_destroys_source_and_restore_keeps_historical_frame() {
        let mut g = game(); fund(&mut g,0,"JC084",2); let h = held(&mut g,"JZ56",0);
        act(&mut g,0,Action{card_id:Some(h),target_id:Some("society:p0".into()),..Action::new("play")});
        pass_top(&mut g); let source = g.attachments.iter().find(|a| a.card.definition == "JZ56").unwrap().card.id.clone();
        choose(&mut g,vec!["accept".into()]);
        // The trigger's controller initially retains priority. Use genuine
        // passes to hand the still-unresolved layer to the opposing team.
        for _ in 0..4 { if g.priority_team == g.team(2) { break; } crate::unit_support::pass(&mut g); }
        assert_eq!(g.priority_team,g.team(2)); assert_eq!(g.stack.len(),1);
        fund(&mut g,2,"JC005",2);fund(&mut g,2,"JZ02",1);
        cast(&mut g,2,"JC005",&source,None);
        assert!(g.in_play(&source).is_none()); assert_eq!(g.stack.len(),1);
        record("original-source-gone-frame-restorable",&g,serde_json::json!({"historicalSource":source}));
        pass_top(&mut g); checkpoint(&g);
        assert!(g.pending.is_none()); assert!(g.stack.is_empty());
    }

    #[test]
    fn independent_jz56_corrupt_live_chosen_value_cannot_coexist_with_pending_choice() {
        let mut g = game(); let source = play(&mut g,0);
        let mut bad = g.clone(); bad.attachments.iter_mut().find(|a|a.card.id==source).unwrap().card.chosen_subtype=Some(UndeadKind::Ghoul);
        let raw = serde_json::to_string(&bad).unwrap();
        let game_restored = std::panic::catch_unwind(||Game::from_persisted(&raw)).unwrap();
        let mut room = serde_json::to_value(crate::jc029_tests::envelope(&g)).unwrap(); room["game"]=serde_json::to_value(&bad).unwrap();
        let room_restored = std::panic::catch_unwind(||crate::room::RoomEnvelope::from_persisted(&room.to_string())).unwrap();
        let mut overwritten = None;
        if let Ok(mut accepted) = game_restored.clone() {
            choose(&mut accepted,vec!["skeleton".into()]);
            overwritten=accepted.attachments.iter().find(|a|a.card.id==source).unwrap().card.chosen_subtype;
        }
        let mut room_outcome = None;
        if let Ok(mut accepted) = room_restored.clone() {
            let choice_id=accepted.game.pending.as_ref().unwrap().choice.id.clone();
            let out=accepted.transition(0,Some(crate::room::RoomCommand{command_id:"independent-night-corrupt-choose".into(),expected_version:accepted.revision,
                action:crate::room::SessionAction::Game{action:Action{choice_id:Some(choice_id),selected:Some(vec!["skeleton".into()]),..Action::new("choose")}}}),0).unwrap();
            room_outcome=Some(out.outcome);
        }
        record("corrupt-choice-observation",&g,serde_json::json!({"badGame":bad,"gameAccepted":game_restored.is_ok(),"roomAccepted":room_restored.is_ok(),"overwrittenChoice":overwritten,"roomCommandOutcome":room_outcome}));
        assert!(game_restored.is_err() && room_restored.is_err(), "already-selected live instance restored with pending choice; overwritten={overwritten:?}");
    }

    #[test]
    fn independent_jz56_old_pool_and_unknown_enum_fail_closed_without_panics() {
        let mut g = game(); let source = night(&mut g,0,"skeleton");
        for mode in 0..2 {
            let mut raw=serde_json::to_value(&g).unwrap();
            if mode==0 {raw["versions"]["cardPool"]=serde_json::json!("limited-v2.85-batch20-fourth");}
            else {let i=g.attachments.iter().position(|a|a.card.id==source).unwrap(); raw["attachments"][i]["card"]["chosen_subtype"]=serde_json::json!("human");}
            let mut room=serde_json::to_value(crate::jc029_tests::envelope(&g)).unwrap();room["game"]=raw.clone();
            assert!(std::panic::catch_unwind(||Game::from_persisted(&raw.to_string())).unwrap().is_err(),"mode {mode}");
            assert!(std::panic::catch_unwind(||crate::room::RoomEnvelope::from_persisted(&room.to_string())).unwrap().is_err(),"mode {mode}");
        }
    }

    #[test]
    fn independent_jz56_printed_wound_must_not_add_kill_damage_to_combat_reward() {
        let (g, _, score) = enhanced_combat();
        let actual = g.pending.as_ref().unwrap().choice.amount;
        record("wound-not-kill-amount",&g,serde_json::json!({"combatScore":score,"assignedDamage":actual}));
        assert_eq!(actual,Some(score),"JZ56 prints 创伤, whereas extra combat damage is 杀伤 (official printed p17)");
    }

    #[test]
    fn independent_jz56_printed_wound_must_reach_the_wound_path_in_nonlethal_combat() {
        let (mut g,victim,score) = enhanced_combat();
        let p = g.pending.as_ref().unwrap(); let amount=p.choice.amount.unwrap(); let seat=p.seat;
        let a=Action{choice_id:Some(p.choice.id.clone()),allocations:Some(std::collections::BTreeMap::from([(victim.clone(),amount)])),..Action::new("choose")};
        act(&mut g,seat,a); checkpoint(&g);
        let c=g.board(&victim).unwrap().1;
        record("wound-not-ordinary-damage",&g,serde_json::json!({"combatScore":score,"assignedDamage":amount,"damage":c.damage,"wounds":c.wounds,"defense":g.defense(c,0)}));
        assert!(c.wounds>0,"JZ56 创伤 must use wound markers; actual damage={}, wounds={}",c.damage,c.wounds);
    }
}

/// P2 closing cases. Starting layouts/windows are explicit fixtures; payments,
/// declarations, allocations and cleanup passes use the normal command paths.
mod p2_close {
    use super::*;
    use std::collections::BTreeMap;
    fn battle(g: &mut Game) {
        g.begin_window(Window::Before(0, 1));
        g.close_window().unwrap(); g.drive().unwrap(); checkpoint(g);
    }
    fn allocate(g: &mut Game, targets: &[(String, u32)]) {
        let p = g.pending.as_ref().unwrap();
        act(g, p.seat, Action { choice_id: Some(p.choice.id.clone()),
            allocations: Some(targets.iter().cloned().collect()), ..Action::new("choose") });
        checkpoint(g);
    }
    fn layout() -> (Game, String, String) {
        let mut g = game(); let ghoul = field(&mut g, "JZ50", 0);
        let victim = field(&mut g, "LC01", 2);
        dead(&mut g, 0, 7); night(&mut g, 0, "ghoul");
        (g, ghoul, victim)
    }
    fn record(name: &str, value: serde_json::Value) {
        if let Ok(dir) = std::env::var("INDEPENDENT_HISTORY_OUT") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(format!("{dir}/{name}.json"), serde_json::to_vec_pretty(&value).unwrap()).unwrap();
        }
    }
    #[test]
    fn batch4_night_p2_exact_wounds_real_room_replay_duplicate_and_end_cleanup() {
        use crate::room::{RoomCommand, RoomEnvelope, SessionAction};
        let (mut g, ghoul, victim) = layout(); battle(&mut g);
        let p = g.pending.as_ref().unwrap(); assert_eq!(p.choice.amount, Some(2));
        assert!(p.choice.title.contains("等量创伤标志"));
        assert!(matches!(&p.resolution, ChoiceResolution::Damage { wound_sources, .. } if wound_sources == &[ghoul.clone()]));
        let room = crate::jc029_tests::envelope(&g);
        let cmd = RoomCommand { command_id: "night-wound-allocation".into(), expected_version: room.revision,
            action: SessionAction::Game { action: Action { choice_id: Some(p.choice.id.clone()),
                allocations: Some(BTreeMap::from([(victim.clone(), 2)])), ..Action::new("choose") } } };
        let out = room.transition(p.seat, Some(cmd.clone()), 0).unwrap();
        assert_eq!(out.outcome, "accepted");
        let next = RoomEnvelope::from_persisted(&out.state).unwrap();
        assert_eq!(serde_json::to_value(room.replay_events(&out.journal).unwrap()).unwrap(), serde_json::to_value(&next).unwrap());
        let duplicate = next.transition(p.seat, Some(cmd), 0).unwrap();
        assert_eq!(duplicate.outcome, "rejected"); assert!(!duplicate.changed && duplicate.journal.is_empty());
        assert_eq!(serde_json::to_value(RoomEnvelope::from_persisted(&duplicate.state).unwrap()).unwrap(), serde_json::to_value(&next).unwrap());
        for viewer in 0..4 {
            let v = next.view(viewer, 0); let c = v.game.regions[0].characters.iter().find(|c| c.instance_id == victim).unwrap();
            assert_eq!((c.damage, c.wounds, c.defense), (Some(0), Some(2), Some(2)));
        }
        record("p2-wound-room", serde_json::json!({"initial":room, "after":next, "journal":out.journal, "duplicate":duplicate}));
        g = next.game;
        // Give the nonlethal recipient an ordinary wound-free point through the
        // existing damage function as a disclosed adjacent engine-path probe.
        g.damage(BTreeMap::from([(victim.clone(), 1)])).unwrap(); checkpoint(&g);
        assert_eq!((g.board(&victim).unwrap().1.damage, g.board(&victim).unwrap().1.wounds), (1, 2));
        let turn = g.turn; g.begin_window(Window::End);
        for _ in 0..128 { if g.turn != turn { break; } if g.pending.is_some() { resolve_choice(&mut g); } else { pass(&mut g); } }
        assert_eq!(g.turn, turn + 1); checkpoint(&g);
        let c = g.board(&victim).unwrap().1; assert_eq!((c.damage, c.wounds, g.defense(c, 0)), (0, 2, 2));
        for viewer in 0..4 { let v = g.view(viewer); let c = v.regions[0].characters.iter().find(|c| c.instance_id == victim).unwrap();
            assert_eq!((c.damage, c.wounds, c.defense), (Some(0), Some(2), Some(2))); }
        record("p2-wounds-survive-cleanup", serde_json::json!({"game":g, "victim":victim}));
    }
    #[test]
    fn batch4_night_p2_seven_to_six_paid_grave_play_removes_future_wound() {
        let mut g = game(); let zombie = field(&mut g, "JC085", 0); let victim = field(&mut g, "LC01", 2);
        dead(&mut g, 0, 6); let card = g.make_card("JC085", 0); let id = card.id.clone(); g.players[0].graveyard.push(card);
        night(&mut g, 0, "zombie"); assert_eq!(g.effective_wound(g.board(&zombie).unwrap().1), 1);
        fund(&mut g, 0, "JC084", 2);
        act(&mut g, 0, Action { card_id: Some(id), region: Some(0), ..Action::new("deploy") });
        assert_eq!(g.players[0].graveyard.len(), 6); assert_eq!(g.effective_wound(g.board(&zombie).unwrap().1), 0);
        pass_top(&mut g); battle(&mut g);
        let p = g.pending.as_ref().unwrap(); let amount = p.choice.amount.unwrap();
        assert_eq!(amount, 2); assert!(matches!(&p.resolution, ChoiceResolution::Damage { wound_sources, .. } if wound_sources.is_empty()));
        allocate(&mut g, &[(victim.clone(), amount)]);
        let c = g.board(&victim).unwrap().1; assert_eq!((c.damage, c.wounds), (2, 0));
    }
    #[test]
    fn batch4_night_p2_real_kill_still_adds_damage_and_combines_with_wound_replacement() {
        for wound in [false, true] {
            let mut g = game(); let killer = field(&mut g, "JC016", 0); let victim = field(&mut g, "LC01", 2); let other = field(&mut g, "LC01", 3);
            assert_eq!(g.effective_kill(g.board(&killer).unwrap().1), 1);
            if wound { field(&mut g, "JZ50", 0); dead(&mut g, 0, 7); night(&mut g, 0, "ghoul"); }
            let combat = g.contest_counts(0, 1); let amount = combat[0] - combat[1] + 1;
            battle(&mut g); assert_eq!(g.pending.as_ref().unwrap().choice.amount, Some(amount));
            allocate(&mut g, &[(victim.clone(), 2), (other.clone(), amount - 2)]);
            let c = g.board(&victim).unwrap().1;
            assert_eq!((c.damage, c.wounds), if wound { (0, 2) } else { (2, 0) });
            let c = g.board(&other).unwrap().1;
            assert_eq!((c.damage, c.wounds), if wound { (0, amount - 2) } else { (amount - 2, 0) });
        }
    }
    #[test]
    fn batch4_night_p2_team_recipient_personal_participation_and_nonparticipating_grant() {
        for recipient in [0, 1] {
            let (mut g, ghoul, victim) = layout(); field(&mut g, "JC014", 1); let other = field(&mut g, "LC01", 3);
            battle(&mut g); let p = g.pending.as_ref().unwrap();
            assert!(matches!(&p.resolution, ChoiceResolution::Recipient { wound_sources, .. } if wound_sources == &[ghoul.clone()]));
            let amount = p.choice.amount.unwrap(); choose(&mut g, vec![player_id(recipient)]); checkpoint(&g);
            let p = g.pending.as_ref().unwrap(); assert_eq!(p.seat, recipient);
            assert!(matches!(&p.resolution, ChoiceResolution::Damage { wound_sources, .. } if wound_sources.is_empty() == (recipient == 1)));
            allocate(&mut g, &[(victim.clone(), 2), (other.clone(), amount - 2)]);
            let c = g.board(&victim).unwrap().1;
            assert_eq!((c.damage, c.wounds), if recipient == 0 { (0, 2) } else { (2, 0) });
            let c = g.board(&other).unwrap().1;
            assert_eq!((c.damage, c.wounds), if recipient == 0 { (0, amount - 2) } else { (amount - 2, 0) });
        }
        let (mut g, ghoul, victim) = layout(); let killer = field(&mut g, "JC016", 0);
        g.board_mut(&ghoul).unwrap().exhausted = true;
        assert_eq!(g.effective_wound(g.board(&ghoul).unwrap().1), 1); assert_eq!(g.icons(g.board(&ghoul).unwrap().1, 0).combat, 0);
        let amount = g.icons(g.board(&killer).unwrap().1, 0).combat + 1;
        battle(&mut g); allocate(&mut g, &[(victim.clone(), amount)]);
        assert_eq!(g.board(&victim).unwrap().1.wounds, 0);
    }
    #[test]
    fn batch4_night_p2_multiple_sources_convert_equal_points_once_and_emit_real_lethal_wound() {
        let (mut g, _, victim) = layout(); night(&mut g, 0, "ghoul");
        battle(&mut g); assert_eq!(g.pending.as_ref().unwrap().choice.amount, Some(3));
        allocate(&mut g, &[(victim.clone(), 3)]);
        assert_eq!((g.board(&victim).unwrap().1.damage, g.board(&victim).unwrap().1.wounds), (0, 3));
        let (mut g, _, victim) = layout(); field(&mut g, "JC016", 0); let monk = field(&mut g, "LC06", 2);
        let hand = g.players[2].hand.len(); battle(&mut g); assert_eq!(g.pending.as_ref().unwrap().choice.amount, Some(5));
        allocate(&mut g, &[(victim.clone(), 3), (monk.clone(), 2)]);
        assert_eq!(g.board(&victim).unwrap().1.wounds, 3); assert!(g.board(&monk).is_none());
        let p = g.pending.as_ref().unwrap();
        assert!(matches!(&p.resolution, ChoiceResolution::Declare { declaration, .. } if declaration.source.card.id == monk && declaration.ability.event == Some(Event::ReceiveWound)));
        assert_eq!(p.seat, 2);
        for viewer in 0..4 { assert_eq!(g.view(viewer).pending_choice.is_some(), viewer == 2); }
        choose(&mut g, vec!["accept".into()]); pass_top(&mut g);
        assert_eq!(g.players[2].hand.len(), hand + 2); checkpoint(&g);
    }
    #[test]
    fn batch4_night_p2_wound_receipts_fail_closed_for_missing_foreign_duplicate_or_rebound_participants() {
        let (mut g, ghoul, victim) = layout(); battle(&mut g);
        for sources in [vec![], vec![victim.clone()], vec![ghoul.clone(), ghoul.clone()], vec!["missing".into()]] {
            let mut bad = g.clone(); if let ChoiceResolution::Damage { wound_sources, .. } = &mut bad.pending.as_mut().unwrap().resolution { *wound_sources = sources; }
            invalid(&g, &bad);
        }
        let mut bad = g.clone(); bad.pending.as_mut().unwrap().choice.amount = Some(3); invalid(&g, &bad);
        let mut bad = g.clone(); bad.regions[0].cards.iter_mut().find(|c| c.id == ghoul).unwrap().id = bad.id(); invalid(&g, &bad);
        let mut raw = serde_json::to_value(&g).unwrap(); raw["pending"]["resolution"]["Damage"].as_object_mut().unwrap().remove("wound_sources");
        assert!(Game::from_persisted(&raw.to_string()).is_err());
        let (mut g, ghoul, victim) = layout(); field(&mut g, "JC014", 1); battle(&mut g);
        for sources in [vec![], vec![victim], vec![ghoul.clone(), ghoul]] {
            let mut bad = g.clone(); if let ChoiceResolution::Recipient { wound_sources, .. } = &mut bad.pending.as_mut().unwrap().resolution { *wound_sources = sources; }
            invalid(&g, &bad);
        }
    }
    #[test]
    fn batch4_night_p2_chosen_live_value_rejects_every_unfinished_entry_stage_and_finish_overwrite() {
        let mut g = game(); fund(&mut g, 0, "JC084", 2); let h = held(&mut g, "JZ56", 0);
        act(&mut g, 0, Action { card_id: Some(h), target_id: Some("society:p0".into()), ..Action::new("play") });
        pass_top(&mut g); let source = g.attachments.iter().find(|a| a.card.definition == "JZ56").unwrap().card.id.clone();
        let declaration = g.clone();
        let mut queued = g.clone(); let p = queued.pending.take().unwrap();
        if let ChoiceResolution::Declare { declaration, .. } = p.resolution { queued.effects.push_front(Effect::Declare { declaration }); } else { panic!("entry declaration"); }
        checkpoint(&queued); choose(&mut g, vec!["accept".into()]); let stack = g.clone(); pass_top(&mut g);
        let frame = match &g.pending.as_ref().unwrap().resolution { ChoiceResolution::Frame { frame, .. } => frame.as_ref().clone(), _ => panic!("entry choice") };
        for good in [queued, declaration, stack, g.clone()] {
            checkpoint(&good);
            for kind in [UndeadKind::Ghoul, UndeadKind::Zombie, UndeadKind::Skeleton] {
                let mut bad = good.clone(); bad.attachments.iter_mut().find(|a| a.card.id == source).unwrap().card.chosen_subtype = Some(kind);
                invalid(&good, &bad);
                let before = serde_json::to_value(&bad).unwrap();
                let action = Action { choice_id: g.pending.as_ref().map(|p| p.choice.id.clone()), selected: Some(vec!["skeleton".into()]), ..Action::new("choose") };
                assert!(bad.jz56_choose_finish(&frame, 0, &action, &["skeleton".into()]).is_err());
                assert_eq!(serde_json::to_value(&bad).unwrap(), before);
            }
        }
        choose(&mut g, vec!["ghoul".into()]); checkpoint(&g);
        assert_eq!(g.attachments.iter().find(|a| a.card.id == source).unwrap().card.chosen_subtype, Some(UndeadKind::Ghoul));
    }
}

// Independent follow-up: the new combat continuation's recipient envelope and
// the distinction between wound replacement and ordinary damage prevention.
mod independent_closure_review {
    use super::*;
    use std::collections::BTreeMap;
    fn layout(team: bool) -> (Game, String, String) {
        let mut g=game(); let ghoul=field(&mut g,"JZ50",0); let victim=field(&mut g,"LC01",2);
        if team { field(&mut g,"JC014",1); }
        dead(&mut g,0,7);night(&mut g,0,"ghoul");
        (g,ghoul,victim)
    }
    fn battle(g: &mut Game) {
        g.begin_window(Window::Before(0,1));g.close_window().unwrap();g.drive().unwrap();checkpoint(g);
    }
    fn record(name: &str, data: serde_json::Value) {
        if let Ok(dir)=std::env::var("INDEPENDENT_HISTORY_OUT") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(format!("{dir}/{name}.json"),serde_json::to_vec_pretty(&data).unwrap()).unwrap();
        }
    }
    fn load_both(good: &Game, bad: &Game) -> (Result<Game,String>,Result<crate::room::RoomEnvelope,String>) {
        let raw=serde_json::to_string(bad).unwrap();
        let game=std::panic::catch_unwind(||Game::from_persisted(&raw)).expect("Game restore must not panic");
        let mut raw_room=serde_json::to_value(crate::jc029_tests::envelope(good)).unwrap();
        raw_room["game"]=serde_json::to_value(bad).unwrap();
        let room=std::panic::catch_unwind(||crate::room::RoomEnvelope::from_persisted(&raw_room.to_string())).expect("Room restore must not panic");
        (game,room)
    }
    #[test]
    fn independent_close_recipient_may_not_add_enemy_to_frozen_wound_reward() {
        let (mut g,_,_)=layout(true);battle(&mut g);
        assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Recipient{wound_sources,..} if !wound_sources.is_empty()));
        let mut bad=g.clone();
        bad.pending.as_mut().unwrap().choice.options.push(ChoiceOption{id:"p2".into(),label:bad.players[2].name.clone(),card:None});
        let (game,room)=load_both(&g,&bad);
        let mut next=None; let mut outcome=None;
        if let Ok(r)=&room {
            let p=r.game.pending.as_ref().unwrap();
            let out=r.transition(p.seat,Some(crate::room::RoomCommand{command_id:"independent-recipient-enemy".into(),expected_version:r.revision,
                action:crate::room::SessionAction::Game{action:Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec!["p2".into()]),..Action::new("choose")}}}),0).unwrap();
            outcome=Some(out.outcome.clone());
            if out.outcome=="accepted" { next=Some(crate::room::RoomEnvelope::from_persisted(&out.state).unwrap()); }
        }
        record("recipient-enemy-option",serde_json::json!({"good":g,"bad":bad,"gameAccepted":game.is_ok(),"roomAccepted":room.is_ok(),"roomOutcome":outcome,"after":next}));
        assert!(game.is_err()&&room.is_err(),"wound recipient corruption accepted: roomOutcome={outcome:?}");
    }
    #[test]
    fn independent_close_recipient_may_not_restore_empty_selection_that_panics() {
        let (mut g,_,_)=layout(true);battle(&mut g);
        let mut bad=g.clone();bad.pending.as_mut().unwrap().choice.min=Some(0);
        let (game,room)=load_both(&g,&bad);
        let mut command_panics=false; let mut command_error=None;
        if let Ok(mut accepted)=game.clone() {
            let p=accepted.pending.as_ref().unwrap();let seat=p.seat;
            let a=Action{choice_id:Some(p.choice.id.clone()),selected:Some(vec![]),..Action::new("choose")};
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(||accepted.apply(seat,a))) {
                Err(_)=>command_panics=true,Ok(result)=>command_error=result.err(),
            }
        }
        record("recipient-empty-choice",serde_json::json!({"good":g,"bad":bad,"gameAccepted":game.is_ok(),"roomAccepted":room.is_ok(),"commandPanics":command_panics,"commandError":command_error}));
        assert!(game.is_err()&&room.is_err(),"wound recipient cardinality corruption accepted; commandPanics={command_panics}");
    }
    #[test]
    fn independent_close_real_damage_prevention_does_not_prevent_combat_wounds() {
        let (mut g,_,victim)=layout(false);
        g.begin_window(Window::Action(1));fund(&mut g,2,"JC078",1);
        let spell=held(&mut g,"JC078",2);
        act(&mut g,2,Action{card_id:Some(spell),target_id:Some(victim.clone()),..Action::new("play")});
        pass_top(&mut g);checkpoint(&g);assert!(g.damage_prevented(g.board(&victim).unwrap().1));
        battle(&mut g);let p=g.pending.as_ref().unwrap();assert_eq!(p.choice.amount,Some(2));let seat=p.seat;
        let a=Action{choice_id:Some(p.choice.id.clone()),allocations:Some(BTreeMap::from([(victim.clone(),2)])),..Action::new("choose")};
        act(&mut g,seat,a);checkpoint(&g);
        let c=g.board(&victim).unwrap().1;assert_eq!((c.damage,c.wounds,g.defense(c,0)),(0,2,2));
        assert!(g.damage_prevented(c));
        record("actual-prevention-versus-wound",serde_json::json!({"game":g,"victim":victim,"damage":0,"wounds":2,"defense":2}));
    }
}

mod recipient_p2_close {
    use super::*;
    use crate::room::{RoomCommand, RoomEnvelope, SessionAction};
    fn reward() -> (Game, String, [String; 2]) {
        let mut g = game(); let ghoul = field(&mut g, "JZ50", 0); field(&mut g, "JC014", 1);
        let targets = [field(&mut g, "LC01", 2), field(&mut g, "LC01", 3)];
        dead(&mut g, 0, 7); night(&mut g, 0, "ghoul");
        g.begin_window(Window::Before(0, 1)); g.close_window().unwrap(); g.drive().unwrap(); checkpoint(&g);
        assert_eq!(g.pending.as_ref().unwrap().choice.amount, Some(4));
        (g, ghoul, targets)
    }
    fn record(name: &str, data: serde_json::Value) {
        if let Ok(dir) = std::env::var("INDEPENDENT_HISTORY_OUT") {
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(format!("{dir}/{name}.json"), serde_json::to_vec_pretty(&data).unwrap()).unwrap();
        }
    }
    fn deny(good: &Game, bad: &Game, label: &str) {
        let raw = serde_json::to_string(bad).unwrap();
        assert!(std::panic::catch_unwind(|| Game::from_persisted(&raw)).expect(label).is_err(), "Game accepted {label}");
        let mut room = serde_json::to_value(crate::jc029_tests::envelope(good)).unwrap(); room["game"] = serde_json::to_value(bad).unwrap();
        assert!(std::panic::catch_unwind(|| RoomEnvelope::from_persisted(&room.to_string())).expect(label).is_err(), "Room accepted {label}");
    }
    #[test]
    fn batch4_night_recipient_p2_both_restore_entries_certify_complete_choice_envelope() {
        let (g, _, targets) = reward();
        let mutations = ["enemy", "missing", "duplicate", "swapped", "unknown", "label", "card", "min-zero", "min-two", "min-none",
            "max-zero", "max-two", "max-none", "decline", "decline-none", "amount", "chooser-ally", "chooser-enemy", "player", "kind", "empty-id", "preview"];
        for mutation in mutations {
            let mut bad = g.clone(); let p = bad.pending.as_mut().unwrap();
            match mutation {
                "enemy" => p.choice.options.push(ChoiceOption { id: "p2".into(), label: "P2".into(), card: None }),
                "missing" => { p.choice.options.pop(); }
                "duplicate" => p.choice.options[1] = p.choice.options[0].clone(),
                "swapped" => p.choice.options.swap(0, 1),
                "unknown" => p.choice.options[1].id = "p99".into(),
                "label" => p.choice.options[0].label = "伪造姓名".into(),
                "card" => p.choice.options[0].card = Some(g.card_view(g.board(&targets[0]).unwrap().1, 0, Some(0), None)),
                "min-zero" => p.choice.min = Some(0), "min-two" => p.choice.min = Some(2), "min-none" => p.choice.min = None,
                "max-zero" => p.choice.max = Some(0), "max-two" => p.choice.max = Some(2), "max-none" => p.choice.max = None,
                "decline" => p.choice.allow_decline = Some(true), "decline-none" => p.choice.allow_decline = None,
                "amount" => p.choice.amount = Some(5), "chooser-ally" => p.seat = 1, "chooser-enemy" => p.seat = 2,
                "player" => p.choice.player_id = "p1".into(), "kind" => p.choice.kind = "target".into(), "empty-id" => p.choice.id.clear(),
                "preview" => p.choice.preview_cards.push(g.card_view(g.board(&targets[0]).unwrap().1, 0, Some(0), None)), _ => unreachable!(),
            }
            deny(&g, &bad, mutation);
        }
        record("recipient-envelope-rejections", serde_json::json!({"mutations":mutations, "bothEntriesRejected":true, "good":g}));
    }
    #[test]
    fn batch4_night_recipient_p2_queued_effect_certifies_exact_ordered_contributors() {
        let (mut g, _, _) = reward(); let p = g.pending.take().unwrap();
        let ChoiceResolution::Recipient { region, contest, amount, wound_sources } = p.resolution else { panic!("recipient"); };
        let effect = Effect::Recipient { team: 0, region, contest, amount, seats: vec![0, 1], wound_sources };
        // Explicit finite queue fixture; the normal command drive usually turns
        // this effect into a pending choice before a snapshot is returned.
        g.effects.push_front(effect.clone()); checkpoint(&g);
        let candidates = [vec![], vec![0], vec![1], vec![1, 0], vec![0, 0], vec![0, 2], vec![0, 4]];
        for seats in &candidates {
            let mut bad = g.clone(); if let Effect::Recipient { seats: actual, .. } = bad.effects.front_mut().unwrap() { *actual = seats.clone(); }
            deny(&g, &bad, &format!("queued seats {seats:?}"));
            let mut direct = g.clone(); direct.effects.clear(); let before = serde_json::to_value(&direct).unwrap();
            let invalid = bad.effects.front().unwrap().clone();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| direct.effect(invalid))).unwrap();
            assert!(result.is_err()); assert_eq!(serde_json::to_value(&direct).unwrap(), before);
        }
        g.drive().unwrap(); checkpoint(&g);
        assert_eq!(g.pending.as_ref().unwrap().choice.options.iter().map(|o| o.id.as_str()).collect::<Vec<_>>(), ["p0", "p1"]);
        record("recipient-queue-rejections", serde_json::json!({"rejectedSeats":candidates, "bothEntriesRejected":true, "validAfter":g}));
    }
    #[test]
    fn batch4_night_recipient_p2_consumer_requires_one_canonical_actual_recipient_before_mutation() {
        let (mut g, _, _) = reward(); let before = serde_json::to_value(&g).unwrap();
        let bad = [vec![], vec!["p0", "p1"], vec!["p0", "p0"], vec!["p2"], vec!["p99"], vec!["p00"], vec!["p+0"], vec!["p-1"], vec!["0"]];
        for selected in &bad {
            let ids: Vec<String> = selected.iter().map(|s| (*s).into()).collect();
            assert!(g.selected_reward_recipient(0, 0, 1, &ids).is_err());
            let a = Action { choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()), selected: Some(ids), ..Action::new("choose") };
            assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| g.apply(0, a))).unwrap().is_err());
            assert_eq!(serde_json::to_value(&g).unwrap(), before);
        }
        for seat in [1, 2, 99] { assert!(g.selected_reward_recipient(seat, 0, 1, &["p0".into()]).is_err()); }
        for recipient in [0, 1] { assert_eq!(g.selected_reward_recipient(0, 0, 1, &[player_id(recipient)]).unwrap(), recipient); }
        checkpoint(&g);
    }
    #[test]
    fn batch4_night_recipient_p2_legal_game_room_replay_four_views_and_exact_duplicates() {
        for recipient in [0, 1] {
            let (g, ghoul, targets) = reward(); let p = g.pending.as_ref().unwrap();
            let action = Action { choice_id: Some(p.choice.id.clone()), selected: Some(vec![player_id(recipient)]), ..Action::new("choose") };
            let mut plain = g.clone(); act(&mut plain, 0, action.clone()); checkpoint(&plain);
            let room = crate::jc029_tests::envelope(&g);
            let cmd = RoomCommand { command_id: format!("recipient-{recipient}"), expected_version: room.revision, action: SessionAction::Game { action } };
            let out = room.transition(0, Some(cmd.clone()), 0).unwrap(); assert_eq!(out.outcome, "accepted");
            let next = RoomEnvelope::from_persisted(&out.state).unwrap();
            assert_eq!(serde_json::to_value(room.replay_events(&out.journal).unwrap()).unwrap(), serde_json::to_value(&next).unwrap());
            assert_eq!(serde_json::to_value(&plain).unwrap(), serde_json::to_value(&next.game).unwrap());
            let p = next.game.pending.as_ref().unwrap(); assert_eq!(p.seat, recipient); assert_eq!(p.choice.amount, Some(4));
            assert!(matches!(&p.resolution, ChoiceResolution::Damage { wound_sources, .. } if *wound_sources == if recipient == 0 { vec![ghoul] } else { vec![] }));
            for viewer in 0..4 {
                assert_eq!(next.view(viewer, 0).pending_choice.is_some(), viewer == recipient);
                assert_eq!(serde_json::to_value(next.view(viewer, 0)).unwrap(), serde_json::to_value(RoomEnvelope::from_persisted(&out.state).unwrap().view(viewer, 0)).unwrap());
            }
            let duplicate = next.transition(0, Some(cmd), 0).unwrap();
            assert_eq!(duplicate.outcome, "rejected"); assert!(!duplicate.changed && duplicate.journal.is_empty());
            assert_eq!(serde_json::to_value(RoomEnvelope::from_persisted(&duplicate.state).unwrap()).unwrap(), serde_json::to_value(&next).unwrap());
            let allocation = RoomCommand { command_id: format!("recipient-damage-{recipient}"), expected_version: next.revision,
                action: SessionAction::Game { action: Action { choice_id: Some(p.choice.id.clone()), allocations: Some(targets.iter().map(|id| (id.clone(), 2)).collect()), ..Action::new("choose") } } };
            let allocated = next.transition(recipient, Some(allocation), 0).unwrap(); assert_eq!(allocated.outcome, "accepted");
            let final_room = RoomEnvelope::from_persisted(&allocated.state).unwrap();
            assert_eq!(serde_json::to_value(next.replay_events(&allocated.journal).unwrap()).unwrap(), serde_json::to_value(&final_room).unwrap());
            for target in targets {
                let c = final_room.game.board(&target).unwrap().1; assert_eq!((c.damage, c.wounds), if recipient == 0 { (0, 2) } else { (2, 0) });
            }
            record(&format!("legal-recipient-{recipient}"), serde_json::json!({"initial":room, "chosen":next, "choiceJournal":out.journal,
                "allocated":final_room, "allocationJournal":allocated.journal, "duplicate":duplicate}));
        }
    }
}
