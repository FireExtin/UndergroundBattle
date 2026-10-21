//! Explicit native layouts; normal actions and both persistence loaders after setup.
use crate::{catalog, jc035, model::*, rules::*, unit_support as u};
use serde_json::Value;
thread_local! { static BAD_EXPORT: std::cell::RefCell<Option<Vec<Value>>> = const { std::cell::RefCell::new(None) }; }

fn reject_restore(original: &Game, changed: Value, name: &str) {
    reject_restore_with_window(original, changed, name, false);
}
fn reject_restore_with_window(original: &Game, changed: Value, name: &str, clear_window: bool) {
    let raw = serde_json::to_string(&changed).unwrap();
    let game_error = Game::from_persisted(&raw).err().unwrap_or_else(|| panic!("Game accepted {name}"));
    let mut room = serde_json::to_value(crate::jc029_tests::envelope(original)).unwrap();
    room["game"] = changed;
    if clear_window { room["pacing"]["window"] = Value::Null; }
    let state = serde_json::to_string(&room).unwrap();
    let room_error = crate::room::RoomEnvelope::from_persisted(&state).err().unwrap_or_else(|| panic!("Room accepted {name}"));
    BAD_EXPORT.with(|saved| if let Some(rows) = saved.borrow_mut().as_mut() {
        rows.push(serde_json::json!({"name":name,"state":state,"gameError":game_error,"roomError":room_error,
            "roomId":original.room_id,"inviteCode":original.invite_code,"revision":room["revision"]}));
    });
}

fn attachment(g: &mut Game, actor: usize, region: usize) -> String {
    let c = g.make_card("JC035", actor); let id = c.id.clone();
    let host_id = g.regions[region].card.id.clone();
    g.attachments.push(Attachment { card: c, host_id }); id
}
fn hands(g: &mut Game, counts: &[usize]) {
    for (seat, count) in counts.iter().enumerate() {
        g.players[seat].hand.clear();
        for _ in 0..*count { u::held(g, "JC125", seat); }
    }
}
fn preparation(g: &mut Game) {
    g.jc035_preparation = None;
    g.begin_window(Window::Prepare);
    g.drive().unwrap();
    u::restore(g);
}
fn choose(g: &mut Game, source: &str) { u::choose(g, vec![source.into()]); }

#[test]
fn jc035_whole_print_and_closed_definition() {
    let c = catalog::card("JC035");
    assert_eq!((c.cost, c.color.as_str(), c.kind.as_str()), (3, "蓝", "attachment"));
    assert_eq!(c.loyalty, vec!["蓝色"]); assert_eq!(c.magic_icon, MagicIcon::Mind);
    assert_eq!(c.subtypes, vec!["结界", "媒体"]); assert!(!c.unique);
    assert_eq!(c.temporary_icons.investigation, 1);
    let whole = jc035::definition();
    for variant in 0..12 {
        let mut d = whole.clone();
        match variant {
            0 => d.attachment = None,
            1 => d.abilities.pop().map(|_| ()).unwrap(),
            2 => d.abilities[1].timing = Timing::Standard,
            3 => d.abilities[1].event = Some(Event::Enter),
            4 => d.abilities[1].response_policy = ResponsePolicy::Immediate,
            5 => d.abilities[1].targets.push(target(Zone::Region, EntityKind::Any, Relation::Any, Range::Anywhere)),
            6 => d.abilities[1].costs.push(Cost::ExhaustSource),
            7 => d.abilities[1].ops.push(Op::PlaceOneInfluenceInSourceRegion),
            8 => d.abilities[1].ops = vec![Op::ForEachLivingPlayer(d.abilities[1].ops.clone())],
            9 => d.abilities[0].play_only = false,
            10 => d.attachment.as_mut().unwrap().host.zone = Zone::Board,
            _ => d.attachment.as_mut().unwrap().host_leaves = HostLeaveDestination::OwnerHand,
        }
        assert!(crate::rules::validate_definitions(&std::collections::BTreeMap::from([("JC035".into(), d)])).is_err(), "variant {variant}");
    }
    assert!(crate::rules::validate_ability("JC035", &whole.abilities[1]).is_ok());
    assert!(crate::rules::validate_ability("JC029", &whole.abilities[1]).is_err());
}

#[test]
fn jc035_actual_paid_attachment_all_four_seats_and_temporary_icon() {
    for actor in 0..4 {
        let mut g = u::game(); g.first_team = g.team(actor); g.begin_window(Window::Action(g.team(actor)));
        u::fund(&mut g, actor, "JC029", 3); let source = u::held(&mut g, "JC035", actor);
        u::act(&mut g, actor, Action { card_id: Some(source.clone()), region: Some(0), ..Action::new("play") });
        assert_eq!(u::resources(&g, actor), 0); assert!(g.attachments.is_empty());
        u::top(&mut g);
        let a = g.attachments.iter().find(|a| a.card.definition == "JC035").unwrap();
        assert_ne!(a.card.id, source); assert_eq!(a.host_id, g.regions[0].card.id);
        assert!(g.pending.is_none()); assert_eq!(g.current_icons(&a.card, 0).investigation, 1);
        assert!(g.legal_actions(actor).iter().all(|legal| legal.action.kind != "activate"
            || legal.action.card_id.as_deref() != Some(a.card.id.as_str())));
    }
}

#[test]
fn jc035_temporary_investigation_is_local_ready_and_supports_real_rewards_and_privilege() {
    let mut g = u::game(); let source = attachment(&mut g, 0, 0);
    let c = g.attachments[0].card.clone();
    assert_eq!(g.contest_counts(0, 0), [1, 0]); assert_eq!(g.contributors(0, 0, 0), vec![0]);
    assert_eq!(g.current_icons(&c, 1), Icons::default());
    g.first_team = 1; assert_eq!(g.contest_counts(0, 0), [0, 0]);
    g.first_team = 0; g.attachments[0].card.exhausted = true;
    assert_eq!(g.contest_counts(0, 0), [0, 0]); assert!(g.contributors(0, 0, 0).is_empty());
    g.attachments[0].card.exhausted = false;
    let hand = g.players[0].hand.len(); g.reward(0, 0, 0, 1).unwrap(); g.drive().unwrap();
    assert_eq!(g.pending.as_ref().unwrap().seat, 0); u::resolve_choice(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    u::field(&mut g, "JC003", 2); u::fund(&mut g, 0, "JC029", 1);
    g.begin_window(Window::Before(0, 0)); assert_eq!(g.contest_counts(0, 0), [1, 1]);
    assert!(g.legal_actions(0).iter().any(|a| a.action.kind == "privilege"));
    u::act(&mut g, 0, Action::new("privilege")); assert_eq!(g.resources(0), 0);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0); u::resolve_choice(&mut g);
    assert!(g.attachments.iter().any(|a| a.card.id == source));
}

#[test]
fn jc035_borrowed_attachment_uses_original_controller_for_condition_and_reward() {
    let mut g = u::game(); hands(&mut g, &[4, 4, 3, 4]);
    let source = attachment(&mut g, 2, 0);
    g.attachments[0].card.controller = 0; // Explicit initial shared-control layout.
    preparation(&mut g); assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    choose(&mut g, &source); u::top(&mut g);
    assert_eq!(g.regions[0].influence, [1, 0]);
    assert_eq!(g.attachments[0].card.owner, 2);
}

#[test]
fn jc035_rejects_underpayment_wrong_loyalty_and_unprinted_play_payload_atomically() {
    let mut g = u::game(); let source = u::held(&mut g, "JC035", 0);
    let action = Action { card_id: Some(source), region: Some(0), ..Action::new("play") };
    u::fund(&mut g, 0, "JC029", 2); u::reject(&mut g, 0, action.clone());
    let mut other = g.clone(); other.players[0].assets.clear(); u::fund(&mut other, 0, "JC046", 3);
    u::reject(&mut other, 0, action.clone());
    u::fund(&mut g, 0, "JC029", 1);
    for variant in 0..5 {
        let mut a = action.clone();
        match variant { 0 => a.target_id = Some("region:0".into()), 1 => a.option = Some("invented".into()),
            2 => a.cost_selected = Some(vec![g.players[0].assets[0].id.clone()]),
            3 => a.ability_id = Some(jc035::KEY.into()), _ => a.selected = Some(vec![]), }
        u::reject(&mut g.clone(), 0, a);
    }
}

#[test]
fn jc035_preparation_threshold_enemy_relation_and_elimination_matrix() {
    for actor in 0..4 {
        for enemy_count in [0, 3, 4] {
            let mut g = u::game(); hands(&mut g, &[4, 4, 4, 4]);
            let enemy = (0..4).find(|s| g.team(*s) != g.team(actor)).unwrap();
            g.players[enemy].hand.truncate(enemy_count);
            let source = attachment(&mut g, actor, 0);
            preparation(&mut g);
            if enemy_count <= 3 {
                assert_eq!(g.pending.as_ref().unwrap().seat, actor);
                choose(&mut g, &source); assert_eq!(g.regions[0].influence, [0, 0]);
                assert_eq!(g.stack.len(), 1); u::top(&mut g);
                assert_eq!(g.regions[0].influence[g.team(actor)], 1);
            } else { assert!(g.pending.is_none() && g.stack.is_empty()); }
        }
        let mut g = u::game(); hands(&mut g, &[4, 4, 4, 4]);
        let teammate = (0..4).find(|s| *s != actor && g.team(*s) == g.team(actor)).unwrap();
        g.players[teammate].hand.clear(); attachment(&mut g, actor, 0); preparation(&mut g);
        assert!(g.pending.is_none(), "A teammate is not an opponent");
        let enemy = (0..4).find(|s| g.team(*s) != g.team(actor)).unwrap();
        g.players[enemy].eliminated = true; g.players[enemy].hand.clear();
        preparation(&mut g); assert!(g.pending.is_none(), "An eliminated enemy does not qualify");
    }
}

#[test]
fn jc035_optional_decline_once_per_preparation_and_no_draw_yet() {
    let mut g = u::game(); hands(&mut g, &[4, 4, 3, 4]); let source = attachment(&mut g, 0, 0);
    let sizes = g.players.iter().map(|p| p.hand.len()).collect::<Vec<_>>();
    preparation(&mut g);
    assert_eq!(g.players.iter().map(|p| p.hand.len()).collect::<Vec<_>>(), sizes);
    assert_eq!(g.window, Some(Window::Prepare));
    u::choose(&mut g, vec![]); assert!(g.stack.is_empty());
    assert!(matches!(g.jc035_preparation.as_ref().unwrap().entries[0].decision, Some(jc035::PreparationDecision::Declined)));
    g.begin_window(Window::Prepare); g.drive().unwrap(); u::restore(&mut g);
    assert!(g.pending.is_none()); assert_eq!(g.regions[0].influence, [0, 0]);
    assert!(g.attachments.iter().any(|a| a.card.id == source));
}

#[test]
fn jc035_multiple_optional_sources_are_ordered_by_controller_then_responded_to() {
    let mut g = u::game(); hands(&mut g, &[3, 3, 3, 3]);
    let a = attachment(&mut g, 0, 0); let b = attachment(&mut g, 0, 1);
    let c = attachment(&mut g, 1, 2); let d = attachment(&mut g, 2, 3);
    preparation(&mut g); assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    choose(&mut g, &b); assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    choose(&mut g, &a); assert_eq!(g.pending.as_ref().unwrap().seat, 1);
    choose(&mut g, &c); assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    u::choose(&mut g, vec![]);
    assert!(g.stack.iter().all(|s| s.frame.as_ref().unwrap().ability_key == jc035::KEY));
    assert_eq!(g.stack.iter().map(|s| s.frame.as_ref().unwrap().source.card.id.clone()).collect::<Vec<_>>(), vec![b, a, c]);
    u::top(&mut g); u::top(&mut g); u::top(&mut g);
    assert_eq!(g.regions[0].influence[0], 1); assert_eq!(g.regions[1].influence[0], 1);
    assert_eq!(g.regions[2].influence[0], 1); assert_eq!(g.regions[3].influence, [0, 0]);
    assert!(g.attachments.iter().any(|a| a.card.id == d));
}

#[test]
fn jc035_preparation_condition_is_event_time_snapshot_not_later_hand_changes() {
    let mut g = u::game(); hands(&mut g, &[4, 4, 3, 4]); let source = attachment(&mut g, 0, 0);
    preparation(&mut g); u::held(&mut g, "JC125", 2);
    u::restore(&mut g); choose(&mut g, &source); u::top(&mut g);
    assert_eq!(g.players[2].hand.len(), 4); assert_eq!(g.regions[0].influence[0], 1);
    let mut g = u::game(); hands(&mut g, &[4, 4, 4, 4]); attachment(&mut g, 0, 0);
    preparation(&mut g); g.players[2].hand.clear();
    g.begin_window(Window::Prepare); g.drive().unwrap(); u::restore(&mut g);
    assert!(g.pending.is_none() && g.stack.is_empty()); assert_eq!(g.regions[0].influence, [0, 0]);
}

#[test]
fn jc035_source_destroyed_by_real_fast_reply_still_rewards_original_region() {
    let mut g = u::game(); hands(&mut g, &[4, 4, 3, 4]); let source = attachment(&mut g, 0, 0);
    let reply = u::held(&mut g, "JC005", 2); u::fund(&mut g, 2, "JC005", 3);
    // The spell replaces one of the three observed hand cards.
    g.players[2].hand.remove(0);
    preparation(&mut g); choose(&mut g, &source);
    while g.priority_team != 1 { u::pass(&mut g); }
    u::act(&mut g, 2, Action { card_id: Some(reply), target_id: Some(source.clone()), ..Action::new("play") });
    u::top(&mut g); assert!(g.board(&source).is_none());
    u::top(&mut g); assert_eq!(g.regions[0].influence[0], 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition == "JC035" && c.id != source));
}

#[test]
fn jc035_replaced_region_does_not_receive_old_reward_or_inherit_host() {
    let mut g = u::game(); hands(&mut g, &[4, 4, 3, 4]); let source = attachment(&mut g, 0, 0);
    preparation(&mut g); choose(&mut g, &source);
    // Shared replacement primitive layout, separate from the normal-command oracle.
    let old = g.regions[0].card.id.clone(); g.host_leaves(&old);
    g.regions[0].card = g.make_card("DQJC115", 0);
    u::restore(&mut g); u::top(&mut g);
    assert_eq!(g.regions[0].influence, [0, 0]); assert!(g.board(&source).is_none());
}

#[test]
fn jc035_partial_decline_and_next_turn_can_offer_again_after_real_cleanup() {
    let mut g = u::game(); hands(&mut g, &[4, 4, 3, 4]);
    let a = attachment(&mut g, 0, 0); let b = attachment(&mut g, 0, 1);
    preparation(&mut g); choose(&mut g, &b); u::choose(&mut g, vec![]); u::top(&mut g);
    assert_eq!(g.regions[0].influence[0], 0); assert_eq!(g.regions[1].influence[0], 1);
    g.begin_window(Window::End);
    let turn = g.turn;
    for _ in 0..12 {
        if g.pending.is_some() { break; } u::pass(&mut g);
    }
    assert_eq!(g.turn, turn + 1); assert_eq!(g.window, Some(Window::Prepare));
    let ids = g.pending.as_ref().unwrap().choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>();
    assert!(ids.contains(&a) && ids.contains(&b));
}

#[test]
fn jc035_bad_observations_pending_choices_and_frames_reject_both_loaders() {
    let mut g = u::game(); g.room_id = "0000000000000000008900aa".into();
    hands(&mut g, &[4, 4, 3, 4]); let source = attachment(&mut g, 0, 0); preparation(&mut g);
    let initial = serde_json::to_value(&g).unwrap();
    let mut orphan = initial.clone(); orphan["pending"] = Value::Null;
    reject_restore(&g, orphan, "orphan-undecided-pending");
    let mut multiple = u::game(); multiple.room_id = "0000000000000000008900ac".into();
    hands(&mut multiple, &[4, 4, 3, 4]); attachment(&mut multiple, 0, 0); attachment(&mut multiple, 0, 1);
    preparation(&mut multiple);
    let mut partial = serde_json::to_value(&multiple).unwrap();
    partial["pending"]["resolution"]["JC035PreparationOrder"]["declarations"].as_array_mut().unwrap().pop();
    partial["pending"]["choice"]["options"].as_array_mut().unwrap().pop();
    reject_restore(&multiple, partial, "orphan-undecided-partial-choice");
    for variant in 0..24 {
        let mut v = initial.clone();
        match variant {
            0 => v["jc035_preparation"]["event_id"] = "".into(),
            1 => v["jc035_preparation"]["turn"] = (g.turn + 1).into(),
            2 => v["jc035_preparation"]["hands"][2]["count"] = 4.into(),
            3 => v["jc035_preparation"]["hands"][2]["eliminated"] = true.into(),
            4 => v["jc035_preparation"]["hands"][2]["seat"] = 0.into(),
            5 => v["jc035_preparation"]["entries"][0]["source"]["source_region_instance"] = source.clone().into(),
            6 => v["jc035_preparation"]["entries"][0]["source"]["attachment_host_instance"] = "invented".into(),
            7 => v["jc035_preparation"]["entries"][0]["source"]["card"]["owner"] = 9.into(),
            8 => v["jc035_preparation"]["entries"][0]["source"]["card"]["definition"] = "JC029".into(),
            9 => v["jc035_preparation"]["entries"][0]["source"]["card"]["face_down"] = true.into(),
            10 => v["pending"]["choice"]["min"] = 1.into(),
            11 => v["pending"]["choice"]["max"] = 2.into(),
            12 => v["pending"]["choice"]["allowDecline"] = false.into(),
            13 => v["pending"]["choice"]["title"] = "invented".into(),
            14 => v["pending"]["choice"]["options"][0]["id"] = "missing".into(),
            15 => v["pending"]["choice"]["options"][0]["card"]["instanceId"] = "wrong-public-uid".into(),
            16 => v["pending"]["seat"] = 1.into(),
            17 => v["pending"]["resolution"]["JC035PreparationOrder"]["declarations"][0]["ability"]["ops"][0]["JC035PreparedInfluence"]["event_id"] = "wrong-event".into(),
            18 => {
                let d = v["pending"]["resolution"]["JC035PreparationOrder"]["declarations"][0].clone();
                v["pending"]["resolution"]["JC035PreparationOrder"]["declarations"].as_array_mut().unwrap().push(d);
            },
            19 => v["jc035_preparation"] = Value::Null,
            20 => v["pending"]["choice"]["id"] = "".into(),
            21 => v["pending"]["resolution"]["JC035PreparationOrder"]["declarations"][0]["actor"] = 2.into(),
            22 => v["players"][0]["eliminated"] = true.into(),
            _ => v["jc035_preparation"]["hands"][1]["eliminated"] = true.into(),
        }
        reject_restore(&g, v, &format!("prepare-shape-{variant}"));
    }
    for variant in 0..7 {
        let mut v = initial.clone();
        match variant {
            0 => { let card = v["attachments"][0]["card"].clone(); v["attachments"] = serde_json::json!([]);
                v["regions"][0]["cards"].as_array_mut().unwrap().push(card); },
            1 => v["attachments"][0]["card"]["face_down"] = true.into(),
            2 => v["attachments"][0]["hostId"] = g.players[0].hand[0].id.clone().into(),
            3 => { let card = v["attachments"][0]["card"].clone(); v["players"][0]["assets"].as_array_mut().unwrap().push(card); },
            4 => { v["jc035_preparation"] = Value::Null; v["pending"] = Value::Null; v["effects"] = serde_json::json!([]); },
            5 => v["attachments"][0]["card"]["owner"] = 9.into(),
            _ => v["attachments"][0]["card"]["controller"] = 9.into(),
        }
        reject_restore(&g, v, &format!("entity-shape-{variant}"));
    }
    choose(&mut g, &source); let accepted = serde_json::to_value(&g).unwrap();
    let mut orphan = accepted.clone(); orphan["stack"] = serde_json::json!([]);
    reject_restore_with_window(&g, orphan, "orphan-accepted-frame", true);
    for variant in 0..13 {
        let mut v = accepted.clone();
        match variant {
            0 => v["stack"][0]["frame"]["actor"] = 1.into(),
            1 => v["stack"][0]["frame"]["cursor"] = 1.into(),
            2 => v["stack"][0]["frame"]["guard"] = "Accepted".into(),
            3 => v["stack"][0]["frame"]["steps"][0]["context"] = 1.into(),
            4 => v["stack"][0]["frame"]["steps"][0]["op"]["JC035PreparedInfluence"]["event_id"] = Value::Null,
            5 => v["stack"][0]["frame"]["already_paid"] = serde_json::json!([{"Assets":[]}]),
            6 => v["stack"][0]["frame"]["chosen_region"] = 0.into(),
            7 => v["stack"][0]["label"] = "invented".into(),
            8 => v["stack"][0]["target"] = "region:0".into(),
            9 => v["jc035_preparation"]["entries"][0]["decision"] = "Declined".into(),
            10 => {
                let s = v["stack"][0].clone(); v["stack"].as_array_mut().unwrap().push(s);
            },
            11 => {
                let f = v["stack"][0]["frame"].clone(); v["stack"] = serde_json::json!([]);
                v["effects"] = serde_json::json!([{"Frame":{"frame":f}}]);
            },
            _ => v["jc035_preparation"]["entries"][0]["decision"]["Accepted"]["frame_id"] = "different-frame".into(),
        }
        reject_restore(&g, v, &format!("frame-shape-{variant}"));
    }
    let old = g.stack[0].clone(); u::top(&mut g);
    let mut replay = serde_json::to_value(&g).unwrap(); replay["stack"] = serde_json::json!([old]);
    reject_restore(&g, replay, "resolved-event-replay");
    let mut paid = u::game(); paid.room_id = "0000000000000000008900ab".into();
    u::fund(&mut paid, 0, "JC029", 3); let held = u::held(&mut paid, "JC035", 0);
    u::act(&mut paid, 0, Action { card_id: Some(held), region: Some(0), ..Action::new("play") });
    for variant in 0..2 {
        let mut v = serde_json::to_value(&paid).unwrap();
        if variant == 0 {
            let source = v["stack"][0]["frame"]["source"]["card"].clone();
            v["players"][0]["assets"].as_array_mut().unwrap().push(source);
        } else { v["stack"][0]["card"]["exhausted"] = true.into(); }
        reject_restore(&paid, v, &format!("attach-shape-{variant}"));
    }
}

fn legal_restore_cases() -> Vec<Value> {
    let mut pending = u::game(); hands(&mut pending, &[4, 4, 3, 4]);
    let source = attachment(&mut pending, 0, 0); preparation(&mut pending);
    let mut queued = pending.clone();
    let ChoiceResolution::JC035PreparationOrder { declarations } = queued.pending.take().unwrap().resolution else { unreachable!() };
    queued.effects.extend(declarations.into_iter().map(|declaration| Effect::Declare { declaration }));
    let mut accepted = pending.clone(); choose(&mut accepted, &source);
    let mut declined = pending.clone(); u::choose(&mut declined, vec![]);
    let mut resolved = accepted.clone(); u::top(&mut resolved);
    assert!(matches!(declined.jc035_preparation.as_ref().unwrap().entries[0].decision, Some(jc035::PreparationDecision::Declined)));
    assert!(matches!(resolved.jc035_preparation.as_ref().unwrap().entries[0].decision, Some(jc035::PreparationDecision::Resolved { .. })));
    let mut eliminated_undecided = queued.clone(); eliminated_undecided.eliminate(0);
    let mut eliminated_accepted = accepted.clone(); eliminated_accepted.eliminate(0);
    assert_eq!(eliminated_undecided.status, "playing"); assert_eq!(eliminated_accepted.status, "playing");
    assert!(eliminated_undecided.effects.is_empty() && eliminated_accepted.stack.is_empty());
    let mut finished_undecided = pending.clone(); finished_undecided.finish(0);
    let mut finished_accepted = accepted.clone(); finished_accepted.finish(0);
    let mut departed = accepted.clone(); let host = departed.regions[0].card.id.clone(); departed.host_leaves(&host);
    assert!(departed.attachments.is_empty());
    // Explicit shared control layout, not a claim that JC129 targets attachments.
    let mut controlled = accepted.clone(); controlled.attachments[0].card.controller = 2;
    let layouts = vec![
        ("live-pending", pending), ("live-queued-undecided", queued), ("live-accepted", accepted),
        ("declined-without-container", declined), ("resolved-without-container", resolved),
        ("eliminated-actor-undecided", eliminated_undecided), ("eliminated-actor-accepted", eliminated_accepted),
        ("finished-undecided", finished_undecided), ("finished-accepted", finished_accepted),
        ("departed-source-accepted", departed), ("changed-controller-accepted", controlled),
    ];
    layouts.into_iter().enumerate().map(|(index, (name, mut game))| {
        game.room_id = format!("{:024x}", 890100 + index);
        let game_state = serde_json::to_string(&game).unwrap();
        let restored = Game::from_persisted(&game_state).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(serde_json::to_string(&restored).unwrap(), game_state, "{name}: loading must not pump or rebuild");
        let room = crate::jc029_tests::envelope(&game);
        let state = serde_json::to_string(&room).unwrap();
        let restored = crate::room::RoomEnvelope::from_persisted(&state).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(serde_json::to_string(&restored).unwrap(), state, "{name}: Room loading must not pump or rebuild");
        let views = (0..room.players.len()).map(|seat| room.view(seat, 0)).collect::<Vec<_>>();
        let continuation = (name == "live-pending").then(|| {
            let choice = game.pending.as_ref().unwrap();
            let command = crate::room::RoomCommand { command_id: "p2-restored-choice".into(), expected_version: room.revision,
                action: crate::room::SessionAction::Game { action: Action { choice_id: Some(choice.choice.id.clone()),
                    selected: Some(vec![source.clone()]), ..Action::new("choose") } } };
            let expected = room.transition(choice.seat, Some(command.clone()), 0).unwrap();
            assert!(expected.changed && expected.error_code.is_none());
            let after = crate::room::RoomEnvelope::from_persisted(&expected.state).unwrap();
            serde_json::json!({"seat":choice.seat,"command":command,"serverNowMs":"0","expected":expected,
                "views":(0..after.players.len()).map(|seat|after.view(seat,0)).collect::<Vec<_>>()})
        });
        serde_json::json!({"name":name,"gameState":game_state,"state":state,"views":views,"continuation":continuation})
    }).collect()
}

#[test]
fn jc035_legal_restore_keeps_live_containers_and_cancelled_or_completed_history() {
    assert_eq!(legal_restore_cases().len(), 11);
}

pub(crate) fn source_bindings() -> Value {
    use sha2::{Digest, Sha256};
    let mut binding = crate::xq23_tests::source_bindings();
    for (name, bytes) in [
        ("rust-game/src/jc035.rs", include_bytes!("jc035.rs").as_slice()),
        ("rust-game/src/jc035_tests.rs", include_bytes!("jc035_tests.rs").as_slice()),
    ] {
        binding["files"].as_object_mut().unwrap().insert(name.into(), serde_json::json!(format!("{:x}", Sha256::digest(bytes))));
    }
    assert_eq!(binding["files"].as_object().unwrap().len(), 140); binding
}

struct Trace { name: String, room: crate::room::RoomEnvelope, initial: Value, steps: Vec<Value> }
impl Trace {
    fn new(name: &str, g: &Game) -> Self {
        let room = crate::jc029_tests::envelope(g);
        let initial = serde_json::json!({"state":serde_json::to_string(&room).unwrap(),
            "views":(0..room.players.len()).map(|s|room.view(s,0)).collect::<Vec<_>>()});
        Self { name: name.into(), room, initial, steps: vec![] }
    }
    fn command(&mut self, seat: usize, action: crate::room::SessionAction, accepted: bool) {
        use crate::room::*;
        let state = serde_json::to_string(&self.room).unwrap();
        let command = RoomCommand { command_id: format!("{}-{}", self.name, self.steps.len()), expected_version: self.room.revision, action };
        let expected = self.room.transition(seat, Some(command.clone()), 0).unwrap();
        assert_eq!(expected.error_code.is_none(), accepted, "{} {:?}", self.name, expected.error_message);
        if expected.changed {
            assert_eq!(serde_json::to_string(&self.room.replay_events(&expected.journal).unwrap()).unwrap(), expected.state);
        } else { assert_eq!(expected.state, state); assert!(expected.journal.is_empty()); }
        self.room = RoomEnvelope::from_persisted(&expected.state).unwrap();
        let views = (0..self.room.players.len()).map(|s| self.room.view(s, 0)).collect::<Vec<_>>();
        self.steps.push(serde_json::json!({"state":state,"seat":seat,"command":command,"serverNowMs":"0","expected":expected,"views":views}));
    }
    fn game(&mut self, seat: usize, action: Action, accepted: bool) {
        self.command(seat, crate::room::SessionAction::Game { action }, accepted);
    }
    fn choose(&mut self, selected: Vec<String>) {
        let p = self.room.game.pending.as_ref().unwrap();
        let (seat, id) = (p.seat, p.choice.id.clone());
        self.game(seat, Action { choice_id: Some(id), selected: Some(selected), ..Action::new("choose") }, true);
    }
    fn pass_current(&mut self) {
        use crate::room::*;
        if let Some(w) = self.room.pacing.window.clone() {
            let seat = *w.members.iter().find(|(_, d)| matches!(d, Decision::Undecided { .. })).unwrap().0;
            self.command(seat, SessionAction::PassResponse { window_id: w.id }, true);
        } else {
            let seat = (0..self.room.players.len()).find(|s| self.room.game.legal_actions(*s).iter().any(|a| a.action.kind == "pass")).unwrap();
            self.game(seat, Action::new("pass"), true);
        }
    }
    fn until_prepare(&mut self) {
        let turn = self.room.game.turn;
        for _ in 0..30 {
            if self.room.game.turn > turn && self.room.game.window == Some(Window::Prepare) { return; }
            self.pass_current();
        }
        panic!("preparation not reached in bounded ordinary Room commands");
    }
    fn respond(&mut self, seat: usize, action: Action) {
        use crate::room::*;
        let w = loop {
            let w = self.room.pacing.window.clone().expect("response window");
            if matches!(w.members.get(&seat), Some(Decision::Undecided { .. })) { break w; }
            self.pass_current();
        };
        let intent = format!("{}-intent-{}", self.name, self.steps.len());
        self.command(seat, SessionAction::BeginResponse { window_id: w.id.clone(), intent_id: intent.clone() }, true);
        self.command(seat, SessionAction::SubmitResponse { window_id: w.id, intent_id: intent, action }, true);
    }
    fn settle(&mut self) {
        for _ in 0..150 {
            if let Some(p) = &self.room.game.pending {
                let mut action = Action { choice_id: Some(p.choice.id.clone()), ..Action::new("choose") };
                if matches!(p.resolution, ChoiceResolution::Bottom { .. }) {
                    action.bottom = Some(p.choice.options.iter().map(|o| o.id.clone()).collect());
                } else { action.selected = Some(vec![]); }
                let seat = p.seat; self.game(seat, action, true);
            } else if self.room.game.stack.is_empty() && self.room.game.effects.is_empty()
                && self.room.game.win_contexts.is_empty() { return; }
            else { self.pass_current(); }
        }
        panic!("bounded responsive resolution");
    }
    fn finish(self) -> Value { serde_json::json!({"name":self.name,"initial":self.initial,"steps":self.steps}) }
}

fn at_end(actor: usize, count: usize) -> (Game, String) {
    let mut g = u::game(); hands(&mut g, &[4, 4, 4, 4]);
    let enemy = (0..g.players.len()).find(|s| g.team(*s) != g.team(actor)).unwrap();
    g.players[enemy].hand.truncate(count);
    let source = attachment(&mut g, actor, 0);
    g.begin_window(Window::End); (g, source)
}

#[test]
fn jc035_native89_actual_room_trajectories_and_rejections() {
    use crate::room::SessionAction;
    let mut scenarios = vec![];
    for actor in 0..4 {
        let (mut g, source) = at_end(actor, 3); g.room_id = format!("{:024x}", 890000 + actor);
        let mut t = Trace::new(&format!("actor-{actor}"), &g); t.until_prepare();
        assert_eq!(t.room.game.players.iter().map(|p| p.hand.len()).sum::<usize>(), 15);
        t.choose(vec![source]);
        t.command(0, SessionAction::PauseRoom, true);
        t.game(actor, Action::new("pass"), false);
        t.command(3, SessionAction::ResumeRoom, true); t.settle();
        assert_eq!(t.room.game.regions[0].influence[t.room.game.team(actor)], 1);
        scenarios.push(t.finish());
    }
    for (name, count, accept) in [("optional-decline", 3, false), ("threshold-four", 4, false)] {
        let (mut g, _) = at_end(0, count); g.room_id = format!("{:024x}", 890010 + scenarios.len());
        let mut t = Trace::new(name, &g); t.until_prepare();
        if count == 3 { t.choose(vec![]); }
        assert!(!accept); assert!(t.room.game.stack.is_empty()); assert_eq!(t.room.game.regions[0].influence, [0, 0]);
        scenarios.push(t.finish());
    }
    {
        let (mut g, source) = at_end(0, 3); g.room_id = format!("{:024x}", 890020);
        let reply = u::held(&mut g, "JC005", 2); g.players[2].hand.remove(0); u::fund(&mut g, 2, "JC005", 2);
        let mut t = Trace::new("source-destroyed-response", &g); t.until_prepare(); t.choose(vec![source.clone()]);
        t.respond(2, Action { card_id: Some(reply), target_id: Some(source.clone()), ..Action::new("play") }); t.settle();
        assert!(t.room.game.board(&source).is_none()); assert_eq!(t.room.game.regions[0].influence[0], 1);
        scenarios.push(t.finish());
    }
    {
        let (mut g, source) = at_end(0, 3); g.room_id = format!("{:024x}", 890021);
        let reply = u::held(&mut g, "JC049", 2); g.players[2].hand.remove(0); u::fund(&mut g, 2, "JC049", 2);
        let mut victim = g.make_card("JC125", 2); victim.exhausted = false;
        let sacrifice = victim.id.clone(); g.regions[1].cards.push(victim);
        let mut t = Trace::new("hand-growth-response", &g); t.until_prepare(); t.choose(vec![source]);
        t.respond(2, Action { card_id: Some(reply), cost_selected: Some(vec![sacrifice]), ..Action::new("play") }); t.settle();
        assert_eq!(t.room.game.players[2].hand.len(), 4); assert_eq!(t.room.game.regions[0].influence[0], 1);
        scenarios.push(t.finish());
    }
    {
        let (mut g, _) = at_end(0, 4); g.room_id = format!("{:024x}", 890026);
        let reply = u::held(&mut g, "JC049", 2); g.players[2].hand.remove(0); u::fund(&mut g, 2, "JC049", 2);
        let victim = g.make_card("JC125", 2); let sacrifice = victim.id.clone(); g.regions[1].cards.push(victim);
        let mut t = Trace::new("late-shrink-no-retroactive-trigger", &g); t.until_prepare();
        assert!(t.room.game.jc035_preparation.as_ref().unwrap().entries.is_empty());
        // No trigger qualified, so no response stack exists yet. This is the
        // ordinary fast preparation action; its paid spell then opens responses.
        t.game(2, Action { card_id: Some(reply), cost_selected: Some(vec![sacrifice]), ..Action::new("play") }, true);
        assert_eq!(t.room.game.players[2].hand.len(), 3); assert!(t.room.game.pending.is_none());
        t.settle(); assert_eq!(t.room.game.players[2].hand.len(), 5);
        assert_eq!(t.room.game.regions[0].influence, [0, 0]); scenarios.push(t.finish());
    }
    {
        let (mut g, a) = at_end(0, 3); g.room_id = format!("{:024x}", 890022);
        let b = attachment(&mut g, 0, 1); let teammate = attachment(&mut g, 1, 2);
        let mut t = Trace::new("same-controller-order", &g); t.until_prepare();
        t.choose(vec![b]); t.choose(vec![a]); t.choose(vec![teammate]); t.settle();
        assert_eq!(t.room.game.regions[0].influence[0], 1); assert_eq!(t.room.game.regions[1].influence[0], 1);
        assert_eq!(t.room.game.regions[2].influence[0], 1); scenarios.push(t.finish());
    }
    {
        let (mut g, a) = at_end(0, 3); g.room_id = format!("{:024x}", 890023);
        let b = attachment(&mut g, 0, 0); let original = g.regions[0].card.id.clone();
        g.regions[0].influence[0] = catalog::card(&g.regions[0].card.definition).threshold.unwrap() - 1;
        let mut t = Trace::new("won-original-region", &g); t.until_prepare(); t.choose(vec![a]); t.choose(vec![b]); t.settle();
        assert_ne!(t.room.game.regions[0].card.id, original);
        assert_eq!(t.room.game.regions[0].influence, [0, 0]);
        // Printed win batch returns both attachments to their owner's deck;
        // normal host-departure burial does not replace that return batch.
        assert_eq!(t.room.game.players[0].deck.iter().filter(|c| c.definition == "JC035").count(), 2);
        assert!(t.room.game.players[0].graveyard.iter().all(|c| c.definition != "JC035"));
        assert!(t.room.game.attachments.iter().all(|a| a.card.definition != "JC035"));
        scenarios.push(t.finish());
    }
    {
        let mut g = u::game(); g.room_id = format!("{:024x}", 890024); u::fund(&mut g, 0, "JC029", 3);
        let source = u::held(&mut g, "JC035", 0); let mut t = Trace::new("ordinary-attach", &g);
        t.game(0, Action { card_id: Some(source), region: Some(0), ..Action::new("play") }, true); t.settle();
        assert_eq!(t.room.game.attachments.iter().filter(|a| a.card.definition == "JC035").count(), 1);
        scenarios.push(t.finish());
    }
    {
        let mut g = Game::new(format!("{:024x}", 890025), "LOCAL".into(), "duel".into(), "P0".into(), "watchers".into(), 8).unwrap();
        g.join("P1".into(), "watchers".into()).unwrap();
        for p in &mut g.players { p.ready = true; }
        g.apply(0, Action::new("start")).unwrap();
        while let Some(p) = g.pending.clone() {
            g.apply(p.seat, Action { choice_id: Some(p.choice.id), selected: Some(vec![]), ..Action::new("choose") }).unwrap();
        }
        for p in &mut g.players { p.hand.clear(); p.assets.clear(); p.graveyard.clear(); }
        for r in &mut g.regions { r.cards.clear(); r.influence = [0, 0]; }
        hands(&mut g, &[4, 3]); let source = attachment(&mut g, 0, 0);
        g.first_team = 0; g.begin_window(Window::End);
        let mut t = Trace::new("duel-preparation", &g); t.until_prepare(); t.choose(vec![source]); t.settle();
        assert_eq!(t.room.game.players.len(), 2); assert_eq!(t.room.game.regions[0].influence[0], 1);
        scenarios.push(t.finish());
    }
    BAD_EXPORT.with(|saved| *saved.borrow_mut() = Some(vec![]));
    jc035_bad_observations_pending_choices_and_frames_reject_both_loaders();
    let invalid = BAD_EXPORT.with(|saved| saved.borrow_mut().take().unwrap()); assert_eq!(invalid.len(), 50);
    let legal_restores = legal_restore_cases();
    let commands = scenarios.iter().map(|s| s["steps"].as_array().unwrap().len()).sum::<usize>();
    let rejected = scenarios.iter().flat_map(|s| s["steps"].as_array().unwrap()).filter(|r| !r["expected"]["errorCode"].is_null()).count();
    let projections = scenarios.iter().map(|s| s["initial"]["views"].as_array().unwrap().len()
        + s["steps"].as_array().unwrap().iter().map(|r| r["views"].as_array().unwrap().len()).sum::<usize>()).sum::<usize>();
    let counts = serde_json::json!({"scenarios":scenarios.len(),"commands":commands,"rejected":rejected,"projections":projections,
        "invalidRestores":invalid.len(),"legalRestores":legal_restores.len(),"restoreContinuationCommands":1});
    let value = serde_json::json!({"format":"jc035-native89-v1","catalog":catalog::catalog(),"sourceBindings":source_bindings(),
        "scenarios":scenarios,"invalid":invalid,"legalRestores":legal_restores,"counts":counts});
    if let Ok(dir) = std::env::var("JC035_NATIVE_DIR") {
        use std::io::Write; std::fs::create_dir_all(&dir).unwrap();
        std::fs::OpenOptions::new().create_new(true).write(true).open(std::path::Path::new(&dir).join("Native89.json"))
            .unwrap().write_all(&serde_json::to_vec(&value).unwrap()).unwrap();
    }
    println!("JC035_NATIVE89_COUNTS {counts}");
}
