//! Explicit native layouts; normal actions and both persistence loaders after setup.
use crate::{catalog, jc035, model::*, rules::*, unit_support as u};
use serde_json::Value;
thread_local! { static BAD_EXPORT: std::cell::RefCell<Option<Vec<Value>>> = const { std::cell::RefCell::new(None) }; }

fn reject_restore(original: &Game, changed: Value, name: &str) {
    let raw = serde_json::to_string(&changed).unwrap();
    let game_error = Game::from_persisted(&raw).err().unwrap_or_else(|| panic!("Game accepted {name}"));
    let mut room = serde_json::to_value(crate::jc029_tests::envelope(original)).unwrap();
    room["game"] = changed;
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
fn jc035_departed_source_retains_original_region_reward_and_real_response() {
    let mut g = u::game(); hands(&mut g, &[4, 4, 3, 4]); let source = attachment(&mut g, 0, 0);
    let reply = u::held(&mut g, "JC006", 2); u::fund(&mut g, 2, "JC006", 3);
    // The spell replaces one of the three observed hand cards.
    g.players[2].hand.remove(0);
    preparation(&mut g); choose(&mut g, &source);
    while g.priority_team != 1 { u::pass(&mut g); }
    u::act(&mut g, 2, Action { card_id: Some(reply), target_id: Some(source.clone()), ..Action::new("play") });
    u::top(&mut g); assert!(g.board(&source).is_none());
    u::top(&mut g); assert_eq!(g.regions[0].influence[0], 1);
    assert!(g.players[0].hand.iter().any(|c| c.definition == "JC035" && c.id != source));
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
    for variant in 0..22 {
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
            _ => v["pending"]["resolution"]["JC035PreparationOrder"]["declarations"][0]["actor"] = 2.into(),
        }
        reject_restore(&g, v, &format!("prepare-shape-{variant}"));
    }
    choose(&mut g, &source); let accepted = serde_json::to_value(&g).unwrap();
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
}
