//! Shared lifecycle regressions derived from B's five interface probes at
//! 2dd1f6d1. Hook boundaries are explicit layouts from real paid WM022 frames;
//! these do not claim B hooks/cards are connected or browser acceptance.
use crate::{model::*, mystic_contest::{MysticPhase, COMMIT_KIND, REWARD_KIND},
    pack_a_contest, unit_support::*};
use crate::jc029_tests::checkpoint;

fn layout() -> Game {
    let mut g = game();
    for seat in 0..4 {
        g.players[seat].deck.clear();
        for _ in 0..8 { let c=g.make_card("JC125",seat); g.players[seat].deck.push(c); }
    }
    fund(&mut g,0,"WM022",12);
    g
}
fn reveal(g: &mut Game) {
    let id=field(g,"WM022",0); g.board_mut(&id).unwrap().face_down=true;
    act(g,0,Action { card_id:Some(id), ..Action::new("reveal") }); checkpoint(g);
}
fn start(g: &mut Game) {
    reveal(g);
    for _ in 0..24 { if g.pending.is_some() { break; } pass(g); checkpoint(g); }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,COMMIT_KIND);
}
fn before_selection() -> Game {
    let mut g=layout(); start(&mut g);
    pause_before_selection(&mut g); g
}
fn pause_before_selection(g: &mut Game) {
    // Reuse the original source/cost/frame, rewinding only the documented
    // top-to-hand boundary. A/B's production start hook remains unimplemented.
    for p in &mut g.players { let c=p.hand.pop().unwrap(); p.deck.insert(0,c); }
    let state=g.mystic_contest.as_mut().unwrap();
    state.phase=MysticPhase::BeforeSelection;
    let id=state.contest_id.clone();
    g.pending=None;
    g.effects.push_front(Effect::MysticContinue { contest_id:id });
    restore(g); checkpoint(g);
}
fn choose_zeroes(g: &mut Game) {
    for _ in 0..4 {
        let p=g.pending.as_ref().unwrap(); assert_eq!(p.choice.kind,COMMIT_KIND);
        let id=g.players[p.seat].hand.iter().find(|c|c.definition=="JC125").unwrap().id.clone();
        choose(g,vec![id]); checkpoint(g);
    }
}
fn start_child(g: &mut Game) {
    reveal(g);
    for _ in 0..24 { if g.pending.is_some() {break;} pass(g); checkpoint(g); }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,COMMIT_KIND);
    assert!(g.mystic_contest.as_ref().unwrap().parent.is_some());
}
fn after_boundary() -> Game {
    let mut g=layout(); held(&mut g,"WM021",0); start(&mut g);
    for seat in 0..4 {
        let id=g.players[seat].hand.iter().find(|c|c.definition==if seat==0 {"WM021"} else {"JC125"}).unwrap().id.clone();
        choose(&mut g,vec![id]);
    }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,REWARD_KIND);
    let state=g.mystic_contest.as_mut().unwrap();
    state.phase=MysticPhase::After {outcome:Box::new(g.mystic_history[0].clone()),next_shown:1,post_frame:None};
    let id=state.contest_id.clone(); g.pending=None; g.effects.clear();
    g.effects.push_back(Effect::MysticContinue {contest_id:id});
    restore(&mut g); checkpoint(&g); g
}

#[test]
fn mystic_lifecycle_response_generated_trigger_precedes_secret_selection() {
    let mut g=before_selection(); let parent=g.mystic_contest.as_ref().unwrap().contest_id.clone();
    reveal(&mut g);
    for _ in 0..16 {
        if g.stack.last().is_some_and(|s|s.frame.as_ref().is_some_and(|f|f.ability_key==pack_a_contest::SHADOW_REVEAL)) { break; }
        pass(&mut g); checkpoint(&g);
        assert!(g.pending.is_none(),"response-generated trigger must precede the parent's commitment");
    }
    assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().ability_key,pack_a_contest::SHADOW_REVEAL);
    assert_eq!(g.mystic_contest.as_ref().unwrap().contest_id,parent);
    assert!(g.players.iter().all(|p|p.hand.is_empty()));
}

#[test]
fn mystic_lifecycle_nested_contest_finishes_then_resumes_parent_once() {
    let mut g=before_selection(); let parent=g.mystic_contest.as_ref().unwrap().contest_id.clone();
    start_child(&mut g);
    let child=g.mystic_contest.as_ref().unwrap().contest_id.clone(); assert_ne!(child,parent);
    choose_zeroes(&mut g);
    assert_eq!(g.mystic_history.len(),1); assert_eq!(g.mystic_history[0].contest_id,child);
    assert_eq!(g.mystic_contest.as_ref().unwrap().contest_id,parent);
    assert!(g.players.iter().all(|p|p.hand.len()==1&&p.deck.len()==7));
    choose_zeroes(&mut g);
    assert_eq!(g.mystic_history.iter().map(|o|o.contest_id.as_str()).collect::<Vec<_>>(),[child.as_str(),parent.as_str()]);
    assert!(g.mystic_contest.is_none()&&g.pending.is_none()&&g.stack.is_empty()&&g.effects.is_empty());
    assert!(g.players.iter().all(|p|p.hand.is_empty()&&p.deck.len()==8)); checkpoint(&g);
    assert_eq!(g.resources(0),6,"only the two actual printed reveal payments");
}

#[test]
fn mystic_lifecycle_post_elimination_commits_terminal_state_without_continuation() {
    let mut g=after_boundary();
    g.effects.push_front(Effect::Draw {seat:2,count:1});
    g.players[2].deck.clear(); g.eliminate(3); restore(&mut g); checkpoint(&g);
    pass(&mut g);
    assert_eq!(g.status,"finished"); assert_eq!(g.winner_team,Some(0));
    assert!(g.players[2].eliminated&&g.mystic_contest.is_none());
    assert!(g.effects.is_empty()&&g.stack.is_empty()&&g.pending.is_none());
    assert_eq!(g.mystic_history.len(),1); checkpoint(&g);
}

fn invalid(g: &Game, value: serde_json::Value) {
    assert!(std::panic::catch_unwind(||Game::from_persisted(&value.to_string())).unwrap().is_err());
    let mut room=serde_json::to_value(crate::jc029_tests::envelope(g)).unwrap(); room["game"]=value;
    assert!(std::panic::catch_unwind(||crate::room::RoomEnvelope::from_persisted(&room.to_string())).unwrap().is_err());
}

#[test]
fn mystic_lifecycle_nested_receipts_and_continuations_are_closed_in_both_loaders() {
    let mut g=before_selection(); start_child(&mut g);
    let base=serde_json::to_value(&g).unwrap();
    for case in 0..12 {
        let mut v=base.clone();
        match case {
            0=>v["mystic_contest"]["parent"]["actor_bonus"]=42.into(),
            1=>v["mystic_contest"]["parent"]["frame"]["cursor"]=0.into(),
            2=>v["mystic_contest"]["parent"]["participants"]=serde_json::json!([0,1,2]),
            3=>v["mystic_contest"]["parent"]["contest_id"]=v["mystic_contest"]["contest_id"].clone(),
            4=>v["mystic_contest"]["parent"]["frame"]["frame_id"]=v["mystic_contest"]["frame"]["frame_id"].clone(),
            5=>v["mystic_contest"]["parent"]["phase"]=serde_json::json!({"Selecting":{"committed":[]}}),
            6=>{v["mystic_contest"].as_object_mut().unwrap().remove("parent");},
            7=>v["effects"]=serde_json::json!([]),
            8=>{let e=v["effects"][0].clone();v["effects"].as_array_mut().unwrap().push(e);},
            9=>v["mystic_contest"]["parent"]["response_base"]=serde_json::json!([g.mystic_contest.as_ref().unwrap().frame.frame_id]),
            10=>v["effects"][0]=serde_json::json!({"MysticQueueFence":{"depth":0}}),
            _=>v["mystic_contest"]["parent"]["frame"]["source"]["card"]["definition"]="JC125".into(),
        }
        invalid(&g,v);
    }
    let p=g.pending.as_ref().unwrap().clone();
    let id=g.players[p.seat].hand[0].id.clone(); choose(&mut g,vec![id.clone()]);
    reject(&mut g,p.seat,Action{choice_id:Some(p.choice.id),selected:Some(vec![id]),..Action::new("choose")});
}

#[test]
fn mystic_lifecycle_after_parent_keeps_its_own_outcome_when_child_finishes() {
    let mut g=after_boundary(); let parent=g.mystic_contest.as_ref().unwrap().contest_id.clone();
    start_child(&mut g);
    let original=serde_json::to_value(&g.mystic_history[0]).unwrap();
    choose_zeroes(&mut g);
    assert_eq!(g.mystic_history.len(),2); assert_eq!(g.mystic_history[0].contest_id,parent);
    assert_eq!(serde_json::to_value(&g.mystic_history[0]).unwrap(),original);
    assert!(g.mystic_contest.is_none()&&g.effects.is_empty()&&g.pending.is_none()); checkpoint(&g);
}

#[test]
fn mystic_lifecycle_response_search_private_choice_finishes_before_selection() {
    let mut g=before_selection(); let parent=g.mystic_contest.as_ref().unwrap().contest_id.clone();
    // This was already below the parent boundary, so it must stay below it.
    g.effects.push_back(Effect::Draw{seat:2,count:1});
    fund(&mut g,0,"BQ036",4); let id=field(&mut g,"BQ036",0);g.board_mut(&id).unwrap().face_down=true;
    let secret=g.make_card("XQ14",0);g.players[0].deck.insert(0,secret);
    act(&mut g,0,Action{card_id:Some(id),..Action::new("reveal")});
    for _ in 0..16 {if g.pending.is_some(){break;}pass(&mut g);}
    let ChoiceResolution::Declare {declaration,..}=&g.pending.as_ref().unwrap().resolution else {panic!("search declaration before contest");};
    assert_eq!(declaration.ability.key,crate::batch4_vampires::SEARCH);
    choose(&mut g,vec!["accept".into()]);
    for _ in 0..16 {if g.pending.is_some(){break;}pass(&mut g);}
    assert!(matches!(&g.pending.as_ref().unwrap().resolution,ChoiceResolution::Frame {choice:FrameChoice::BQ036TopSix {..},..}));
    assert!(matches!(g.mystic_contest.as_ref().unwrap().phase,MysticPhase::BeforeSelection));
    assert!(g.players.iter().all(|p|p.hand.is_empty())); checkpoint(&g);
    let selected=g.pending.as_ref().unwrap().choice.options[0].id.clone();
    for viewer in 1..4 {assert!(!serde_json::to_string(&g.view(viewer)).unwrap().contains(&selected));}
    choose(&mut g,vec![selected]);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,COMMIT_KIND);
    assert_eq!(g.mystic_contest.as_ref().unwrap().contest_id,parent);
    assert_eq!(g.players.iter().map(|p|p.hand.len()).collect::<Vec<_>>(),[2,1,1,1]); checkpoint(&g);
    choose_zeroes(&mut g);assert!(g.mystic_contest.is_none());
    assert_eq!(g.players[2].hand.len(),1,"the older tail runs only after the parent completes");
}

#[test]
fn mystic_lifecycle_nested_win_triggers_finish_before_parent_commit() {
    let mut g=before_selection(); held(&mut g,"WM021",0); start_child(&mut g);
    let child=g.mystic_contest.as_ref().unwrap().contest_id.clone();
    let parent=g.mystic_contest.as_ref().unwrap().parent.as_ref().unwrap().contest_id.clone();
    for seat in 0..4 {
        let id=g.players[seat].hand.iter().find(|c|c.definition==if seat==0 {"WM021"} else {"JC125"}).unwrap().id.clone();
        choose(&mut g,vec![id]);
    }
    while g.pending.as_ref().is_some_and(|p|p.choice.kind==REWARD_KIND) {choose(&mut g,vec![]);}
    assert!(matches!(g.mystic_contest.as_ref().unwrap().phase,MysticPhase::Finishing { .. }));
    assert_eq!(g.mystic_contest.as_ref().unwrap().contest_id,child);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"batch3-entry-order"); checkpoint(&g);
    let mut accepted=false;let mut discarded=false;
    for _ in 0..32 {
        if let Some(p)=g.pending.as_ref() {
            if p.choice.kind==COMMIT_KIND {break;}
            let ids=match p.choice.kind.as_str() {
                "batch3-entry-order"=>vec![p.choice.options[0].id.clone()],
                "trigger" if !accepted=>{accepted=true;vec![player_id(0)]},
                "trigger"=>vec![],
                "discard"=>{discarded=true;vec![p.choice.options[0].id.clone()]},
                other=>panic!("unexpected {other}"),
            };
            choose(&mut g,ids);
        } else {pass(&mut g);}
        checkpoint(&g);
    }
    assert!(accepted&&discarded);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,COMMIT_KIND);
    assert_eq!(g.mystic_contest.as_ref().unwrap().contest_id,parent);
    assert!(g.players.iter().all(|p|p.hand.len()==1));
    for _ in 0..4 {let id=g.pending.as_ref().unwrap().choice.options[0].id.clone();choose(&mut g,vec![id]);}
    assert_eq!(g.mystic_history[0].first_winners,[0,1]);
    assert!(g.mystic_history[1].first_winners.is_empty(),"parent cannot reset the child's first win");
    while g.pending.as_ref().is_some_and(|p|p.choice.kind==REWARD_KIND) {choose(&mut g,vec![]);}
    assert!(g.mystic_contest.is_none()&&g.stack.is_empty()&&g.pending.is_none());checkpoint(&g);
}

#[test]
fn mystic_lifecycle_post_nonterminal_elimination_preserves_history_and_finishes_remaining_seats() {
    let mut g=after_boundary(); g.players[2].deck.clear();
    g.effects.push_front(Effect::Draw{seat:2,count:1}); pass(&mut g);
    assert_eq!(g.status,"playing"); assert!(g.players[2].eliminated);
    assert!(g.mystic_contest.is_none()); assert_eq!(g.mystic_history[0].participants,[0,1,2,3]); checkpoint(&g);
}

#[test]
fn mystic_lifecycle_ruling_empty_after_responses_keeps_top_cards_without_zero_or_fence() {
    let mut g=before_selection(); g.players[2].deck.clear();
    let random=g.random; act(&mut g,0,Action::new("pass"));
    assert!(g.mystic_contest.is_none()&&g.mystic_history.is_empty()&&g.pending.is_none());
    assert!(!g.players[2].eliminated);assert_eq!(g.random,random);
    for seat in [0,1,3] {assert_eq!(g.players[seat].hand.len(),1);assert_eq!(g.players[seat].deck.len(),7);}
    assert!(g.effects.is_empty());checkpoint(&g);
}

#[test]
fn mystic_lifecycle_ruling_empty_child_resumes_existing_after_stage_without_outcome() {
    let mut g=after_boundary(); g.players[2].deck.clear();g.players[2].hand.clear();
    let history=serde_json::to_value(&g.mystic_history).unwrap();let resources=g.resources(0);
    reveal(&mut g);
    for _ in 0..48 {
        if g.mystic_contest.is_none()&&g.stack.is_empty()&&g.effects.is_empty(){break;}
        assert!(g.pending.is_none(),"no selection or reward may be fabricated for the child");
        pass(&mut g);checkpoint(&g);
    }
    assert!(g.mystic_contest.is_none()&&g.stack.is_empty()&&g.effects.is_empty());
    assert_eq!(serde_json::to_value(&g.mystic_history).unwrap(),history);
    assert!(!g.players[2].eliminated);assert_eq!(g.resources(0),resources-3);
    checkpoint(&g);
}

#[test]
fn mystic_lifecycle_ruling_unshown_finish_requires_original_empty_participant_and_boundary() {
    let mut g=before_selection();start_child(&mut g);pause_before_selection(&mut g);
    g.players[2].deck.clear();g.players[2].hand.clear();
    let child=g.mystic_contest.as_ref().unwrap().contest_id.clone();
    assert!(matches!(g.effects.pop_front(),Some(Effect::MysticContinue{contest_id}) if contest_id==child));
    g.mystic_contest_resume(&child).unwrap();checkpoint(&g);
    assert!(matches!(&g.mystic_contest.as_ref().unwrap().phase,MysticPhase::FinishingUnshown{unable} if unable==&vec![2]));
    for seats in [vec![],vec![2,2],vec![0],vec![99]] {
        let mut bad=g.clone();bad.mystic_contest.as_mut().unwrap().phase=MysticPhase::FinishingUnshown{unable:seats};
        invalid(&g,serde_json::to_value(bad).unwrap());
    }
    let mut bad=g.clone();bad.effects.pop_front();invalid(&g,serde_json::to_value(bad).unwrap());
    g.drive().unwrap();checkpoint(&g);
    assert!(g.mystic_contest.is_none()&&g.mystic_history.is_empty()&&g.effects.is_empty());
}

#[test]
fn mystic_lifecycle_ruling_start_response_can_leave_a_hand_with_an_empty_deck() {
    let mut g=before_selection();g.players[2].deck.clear();held(&mut g,"JC125",2);
    // Explicit restored end-of-start-response layout, not a synthetic Draw.
    checkpoint(&g);act(&mut g,0,Action::new("pass"));
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,COMMIT_KIND);
    choose_zeroes(&mut g);assert_eq!(g.mystic_history.len(),1);
    assert!(!g.players[2].eliminated);checkpoint(&g);
}

#[test]
fn mystic_lifecycle_three_levels_resume_in_lifo_order_and_reject_reordered_barriers() {
    let mut g=before_selection(); let outer=g.mystic_contest.as_ref().unwrap().contest_id.clone();
    start_child(&mut g); let middle=g.mystic_contest.as_ref().unwrap().contest_id.clone();
    pause_before_selection(&mut g);
    let mut bad=serde_json::to_value(&g).unwrap();bad["effects"].as_array_mut().unwrap().swap(0,1);invalid(&g,bad);
    start_child(&mut g); let inner=g.mystic_contest.as_ref().unwrap().contest_id.clone();
    assert_eq!(g.mystic_states().count(),3);
    for id in [&inner,&middle,&outer] {
        assert_eq!(&g.mystic_contest.as_ref().unwrap().contest_id,id);
        choose_zeroes(&mut g);
    }
    assert_eq!(g.mystic_history.iter().map(|o|&o.contest_id).collect::<Vec<_>>(),[&inner,&middle,&outer]);
    assert!(g.mystic_contest.is_none()); assert_eq!(g.resources(0),3);
    assert!(g.players.iter().all(|p|p.hand.is_empty()&&p.deck.len()==8));checkpoint(&g);
}

#[test]
fn mystic_lifecycle_terminal_clears_every_suspended_level_and_restart_does_not_replay() {
    let mut g=after_boundary(); start_child(&mut g); pause_before_selection(&mut g);
    let stale=serde_json::to_value(&g.mystic_contest).unwrap();
    g.players[2].deck.clear();g.eliminate(3);g.effects.push_front(Effect::Draw{seat:2,count:1});
    pass(&mut g);assert_eq!(g.status,"finished");assert!(g.mystic_contest.is_none());checkpoint(&g);
    let mut bad=serde_json::to_value(&g).unwrap();bad["mystic_contest"]=stale;invalid(&g,bad);
    act(&mut g,0,Action::new("restart"));
    while g.pending.is_some() {choose(&mut g,vec![]);}
    assert_eq!(g.status,"playing");assert!(g.mystic_history.is_empty()&&g.mystic_contest.is_none());checkpoint(&g);
}

fn room_step(room: &mut crate::room::RoomEnvelope, seat: usize, action: crate::room::SessionAction) {
    use crate::room::{RoomCommand,RoomEnvelope};
    let cmd=RoomCommand {command_id:format!("mystic-life-{}",room.revision),expected_version:room.revision,action};
    let result=room.transition(seat,Some(cmd),0).unwrap();
    assert!(result.error_code.is_none(),"{:?}",result.error_message);
    assert_eq!(serde_json::to_string(&room.replay_events(&result.journal).unwrap()).unwrap(),result.state);
    *room=RoomEnvelope::from_persisted(&result.state).unwrap();
    let mut copy=room.game.clone();restore(&mut copy);checkpoint(&room.game);
    for viewer in 0..4 {assert_eq!(serde_json::to_value(room.view(viewer,0)).unwrap(),serde_json::to_value(RoomEnvelope::from_persisted(&result.state).unwrap().view(viewer,0)).unwrap());}
}

#[test]
fn mystic_lifecycle_room_commands_and_journal_replay_complete_nested_response() {
    use crate::room::SessionAction;
    let mut g=before_selection();let outer=g.mystic_contest.as_ref().unwrap().contest_id.clone();
    let id=field(&mut g,"WM022",0);g.board_mut(&id).unwrap().face_down=true;
    let mut room=crate::jc029_tests::envelope(&g);
    room_step(&mut room,0,SessionAction::Game{action:Action{card_id:Some(id),..Action::new("reveal")}});
    for _ in 0..40 {
        if let Some(p)=room.game.pending.clone() {
            assert_eq!(p.choice.kind,COMMIT_KIND);
            let id=room.game.players[p.seat].hand.iter().find(|c|c.definition=="JC125").unwrap().id.clone();
            room_step(&mut room,p.seat,SessionAction::Game{action:Action{choice_id:Some(p.choice.id),selected:Some(vec![id]),..Action::new("choose")}});
        } else if let Some(window)=room.pacing.window.clone() {
            let seat=(0..4).find(|s|room.game.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();
            room_step(&mut room,seat,SessionAction::PassResponse{window_id:window.id});
        } else {break;}
    }
    assert!(room.game.mystic_contest.is_none()&&room.game.pending.is_none()&&room.game.stack.is_empty());
    assert_eq!(room.game.mystic_history.len(),2);assert_eq!(room.game.mystic_history[1].contest_id,outer);
    assert_eq!(room.game.resources(0),6);
}

#[test]
fn mystic_lifecycle_room_post_terminal_commits_and_replays_instead_of_rolling_back() {
    let mut g=after_boundary();g.players[2].deck.clear();g.eliminate(3);
    g.effects.push_front(Effect::Draw{seat:2,count:1});
    let mut room=crate::jc029_tests::envelope(&g);
    let seat=(0..4).find(|s|g.legal_actions(*s).iter().any(|a|a.action.kind=="pass")).unwrap();
    room_step(&mut room,seat,crate::room::SessionAction::Game{action:Action::new("pass")});
    assert_eq!(room.game.status,"finished");assert_eq!(room.game.winner_team,Some(0));
    assert!(room.game.mystic_contest.is_none()&&room.game.effects.is_empty()&&room.pacing.window.is_none());
}

#[test]
fn mystic_lifecycle_child_win_chain_precedes_older_parent_effects() {
    let mut g=before_selection();held(&mut g,"WM021",0);reveal(&mut g);
    for _ in 0..16 {
        if g.stack.last().is_some_and(|s|s.frame.as_ref().is_some_and(|f|f.ability_key==pack_a_contest::SHADOW_REVEAL)){break;}
        pass(&mut g);
    }
    // Explicit parent-work boundary, like the original B probes: resolve the
    // genuine accepted child frame while an older parent Draw is still queued.
    // The Draw is a layout, not an assertion that a B hook emitted it today.
    g.effects.push_front(Effect::Draw{seat:2,count:1});
    let item=g.stack.pop().unwrap();g.resolve_stack(item).unwrap();
    let Effect::Frame{frame}=g.effects.pop_front().unwrap() else {panic!("accepted response frame");};
    g.resolve_frame(*frame).unwrap();g.drive().unwrap();restore(&mut g);checkpoint(&g);
    for seat in 0..4 {
        let id=g.players[seat].hand.iter().find(|c|c.definition==if seat==0 {"WM021"} else {"JC125"}).unwrap().id.clone();
        choose(&mut g,vec![id]);
    }
    while g.pending.as_ref().is_some_and(|p|p.choice.kind==REWARD_KIND){choose(&mut g,vec![]);}
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,"batch3-entry-order");
    assert!(g.players[2].hand.is_empty(),"the child's winning-trigger chain must finish before the older parent Draw");
    for _ in 0..20 {
        let p=g.pending.as_ref().unwrap();if p.choice.kind==COMMIT_KIND {break;}
        let ids=if p.choice.kind=="batch3-entry-order"{vec![p.choice.options[0].id.clone()]}else{vec![]};
        choose(&mut g,ids);
    }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,COMMIT_KIND);
    assert_eq!(g.players[2].hand.len(),2);checkpoint(&g);
}

#[test]
fn mystic_lifecycle_finishing_chain_authenticates_boundary_and_accepts_another_nested_response() {
    let mut g=before_selection();let outer=g.mystic_contest.as_ref().unwrap().contest_id.clone();
    held(&mut g,"WM021",0);start_child(&mut g);
    for seat in 0..4 {
        let id=g.players[seat].hand.iter().find(|c|c.definition==if seat==0 {"WM021"}else{"JC125"}).unwrap().id.clone();choose(&mut g,vec![id]);
    }
    while g.pending.as_ref().is_some_and(|p|p.choice.kind==REWARD_KIND){choose(&mut g,vec![]);}
    assert!(matches!(g.mystic_contest.as_ref().unwrap().phase,MysticPhase::Finishing {..}));
    let base=serde_json::to_value(&g).unwrap();
    for case in 0..5 {
        let mut v=base.clone();let end=v["effects"].as_array().unwrap().iter().position(|e|e.get("MysticFinish").is_some()).unwrap();
        match case {
            0=>{v["effects"].as_array_mut().unwrap().remove(end);},
            1=>{let e=v["effects"][end].clone();v["effects"].as_array_mut().unwrap().insert(end,e);},
            2=>v["effects"].as_array_mut().unwrap().swap(end,end+1),
            3=>v["mystic_contest"]["phase"]["Finishing"]["outcome"]["totals"][0]=999.into(),
            _=>v["effects"][end]["MysticFinish"]["contest_id"]=outer.clone().into(),
        }
        invalid(&g,v);
    }
    let mut accepted=false;
    while let Some(p)=g.pending.as_ref() {
        let ids=if p.choice.kind=="batch3-entry-order" {vec![p.choice.options[0].id.clone()]}
            else if !accepted {accepted=true;vec![player_id(0)]}else{vec![]};
        choose(&mut g,ids);
    }
    assert!(accepted&&!g.stack.is_empty());
    while g.priority_team!=0 {pass(&mut g);}
    start_child(&mut g);assert_eq!(g.mystic_states().count(),3);
    assert!(matches!(g.mystic_contest.as_ref().unwrap().parent.as_ref().unwrap().phase,MysticPhase::Finishing {..}));
    choose_zeroes(&mut g);
    for _ in 0..20 {
        if let Some(p)=g.pending.as_ref() {
            if p.choice.kind==COMMIT_KIND {break;}
            assert_eq!(p.choice.kind,"discard");let id=p.choice.options[0].id.clone();choose(&mut g,vec![id]);
        }else{pass(&mut g);}
    }
    assert_eq!(g.pending.as_ref().unwrap().choice.kind,COMMIT_KIND);
    assert_eq!(g.mystic_contest.as_ref().unwrap().contest_id,outer);
    assert_eq!(g.mystic_history.len(),2);checkpoint(&g);
}
