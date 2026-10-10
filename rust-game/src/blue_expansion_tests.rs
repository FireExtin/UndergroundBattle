//! Offline native layouts; all plays, reveals, payments and choices are real
//! Game/Room transitions. These are candidates, not independent UI playtests.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{blue_expansion, catalog, deck, model::*, room::*, rules::*};
use std::sync::atomic::{AtomicUsize, Ordering};

static INVALID_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

fn initial(actor: usize) -> Game {
    let mut g = game(actor);
    for r in &mut g.regions {
        r.influence = [0; 2];
    }
    g
}
fn held(g: &mut Game, def: &str, seat: usize) -> String {
    let c = g.make_card(def, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn play(g: &mut Game, actor: usize, def: &str, target: &str) -> String {
    let id = held(g, def, actor);
    let a = g
        .legal_actions(actor)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&id)
                && a.action.target_id.as_deref() == Some(target)
        })
        .expect("real attachment action")
        .action;
    apply(g, actor, a);
    pass_top(g);
    g.attachments
        .iter()
        .find(|a| a.card.definition == def && a.host_id == target)
        .unwrap()
        .card
        .id
        .clone()
}
fn declare_reveal(g: &mut Game, actor: usize) -> String {
    let id = held(g, "BQ028", actor);
    fund(g, actor, "BQ028", 3);
    apply(
        g,
        actor,
        Action {
            card_id: Some(id),
            region: Some(2),
            ..Action::new("conceal")
        },
    );
    assert_eq!(g.resources(actor), 2);
    assert!(g.pending.is_none() && g.stack.is_empty());
    let hidden = g.regions[2].cards.last().unwrap().id.clone();
    apply(
        g,
        actor,
        Action {
            card_id: Some(hidden.clone()),
            ..Action::new("reveal")
        },
    );
    assert_eq!(g.resources(actor), 0);
    pass_top(g);
    let revealed = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "BQ028")
        .unwrap()
        .id
        .clone();
    assert_ne!(hidden, revealed);
    assert!(g.pending.is_some());
    let ChoiceResolution::Declare {
        declaration,
        stage: DeclareChoice::Target,
    } = &g.pending.as_ref().unwrap().resolution
    else {
        panic!("real Reveal must offer a target declaration")
    };
    assert_eq!(declaration.source.card.id, revealed);
    assert_eq!(declaration.ability.event, Some(Event::Reveal));
    revealed
}
fn inspect(g: &mut Game, target: usize) {
    choose(g, vec![format!("p{target}")]);
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(
        f.targets[0].public.controller.as_deref(),
        Some(format!("p{target}").as_str())
    );
    pass_top(g);
    assert_eq!(
        g.pending.as_ref().unwrap().choice.kind,
        "bq028-hand-inspect"
    );
}
fn bad_state(g: &Game, v: serde_json::Value) {
    let encoded = serde_json::to_string(&v).unwrap();
    assert!(Game::from_persisted(&encoded).is_err());
    let mut r = serde_json::to_value(envelope(g)).unwrap();
    r["game"] = v;
    let room_state = serde_json::to_string(&r).unwrap();
    assert!(RoomEnvelope::from_persisted(&room_state).is_err());
    if let Ok(dir) = std::env::var("BLUE_EXPANSION_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let n = INVALID_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        std::fs::write(
            format!("{dir}/blue-invalid-{n:03}.json"),
            serde_json::to_vec(&serde_json::json!({ "state": room_state })).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn blue_expansion_original_fields_and_whole_definition_reuse() {
    let c = catalog::card("BQ040");
    assert_eq!(
        (&*c.name, &*c.kind, c.cost, &*c.color, &*c.magic),
        ("鲜血", "attachment", 1, "蓝", "鲜血")
    );
    assert_eq!(c.loyalty, ["蓝色"]);
    assert_eq!(c.subtypes, ["状态"]);
    assert_eq!(c.magic_icon, MagicIcon::Blood);
    assert!(!c.unique && c.keywords.is_empty());
    assert_eq!(
        serde_json::to_value(definition("BQ040")).unwrap(),
        serde_json::to_value(definition("XQ14")).unwrap()
    );
    let c = catalog::card("BQ028");
    assert_eq!(
        (&*c.name, &*c.kind, c.cost, c.defense, &*c.magic),
        ("水沟鼠群密探", "character", 2, Some(1), "")
    );
    assert_eq!(c.loyalty, ["蓝色"]);
    assert_eq!(c.subtypes, ["鼠", "奴仆", "魔宠"]);
    assert_eq!(c.permanent_icons, Icons::default());
    assert_eq!(
        c.temporary_icons,
        Icons {
            investigation: 1,
            ..Default::default()
        }
    );
    assert!(definition("BQ028").traits.cannot_be_equipped);
    let a = &definition("BQ028").abilities[0];
    assert_eq!(a.event, Some(Event::Reveal));
    assert_eq!(a.timing, Timing::Fast);
    assert_eq!(a.response_policy, ResponsePolicy::Respondable);
    assert!(a.costs.is_empty() && a.modes.is_empty() && a.per_turn_limit.is_none());
    assert!(matches!(
        a.ops.as_slice(),
        [Op::BQ028InspectTargetHandAttachments]
    ));
    for id in ["BQ028", "BQ040"] {
        blue_expansion::validate_definition(id, definition(id)).unwrap();
        for a in &definition(id).abilities {
            validate_ability(id, a).unwrap();
        }
    }
}

#[test]
fn blue_expansion_blood_prints_share_constructed_name_limit_three() {
    for (xq, bq, valid) in [(1, 2, true), (2, 1, true), (2, 2, false)] {
        let d = deck::DeckDraft {
            id: "blue-expansion".into(),
            name: "蓝候选".into(),
            description: String::new(),
            society_id: None,
            cards: vec![
                catalog::DeckEntry {
                    card_id: "JC125".into(),
                    count: 50 - xq - bq,
                },
                catalog::DeckEntry {
                    card_id: "XQ14".into(),
                    count: xq,
                },
                catalog::DeckEntry {
                    card_id: "BQ040".into(),
                    count: bq,
                },
            ],
            rules_version: catalog::RULES_VERSION.into(),
            card_pool_version: catalog::POOL_VERSION.into(),
            engine_version: catalog::ENGINE_VERSION.into(),
            updated_at: String::new(),
        };
        let r = deck::validate(d);
        assert_eq!(r.is_ok(), valid);
        if !valid {
            assert!(r.unwrap_err().contains("鲜血合计4张，最多3张"));
        }
    }
}

#[test]
fn blue_expansion_blood_real_payment_loyalty_and_current_host_target() {
    let mut g = initial(0);
    let vampire = board(&mut g, "JZ27", 2, 2);
    let human = board(&mut g, "JC125", 0, 0);
    let id = held(&mut g, "BQ040", 0);
    let action = Action {
        card_id: Some(id.clone()),
        target_id: Some(vampire.clone()),
        ..Action::new("play")
    };
    reject(&mut g, 0, action.clone());
    fund(&mut g, 0, "JC125", 1);
    reject(&mut g, 0, action.clone());
    g.players[0].assets.clear();
    fund(&mut g, 0, "BQ040", 2);
    reject(
        &mut g,
        0,
        Action {
            target_id: Some(human),
            ..action.clone()
        },
    );
    reject(
        &mut g,
        0,
        Action {
            target_id: Some("stale".into()),
            ..action.clone()
        },
    );
    g.board_mut(&vampire).unwrap().face_down = true;
    reject(&mut g, 0, action.clone());
    g.board_mut(&vampire).unwrap().face_down = false;
    let before = g.defense(g.board(&vampire).unwrap().1, 2);
    apply(&mut g, 0, action);
    assert_eq!(g.resources(0), 1);
    pass_top(&mut g);
    let bq = g.attachments[0].card.id.clone();
    assert_ne!(id, bq);
    assert_eq!(g.defense(g.board(&vampire).unwrap().1, 2), before + 1);
    play(&mut g, 0, "XQ14", &vampire);
    assert_eq!(g.defense(g.board(&vampire).unwrap().1, 2), before + 2);
    for seat in 0..4 {
        let v = g.view(seat);
        let c = v.regions[2]
            .characters
            .iter()
            .find(|c| c.instance_id == vampire)
            .unwrap();
        assert_eq!(
            c.icons.unwrap().combat,
            catalog::card("JZ27").permanent_icons.combat + 2
        );
        assert_eq!(c.defense, Some(before + 2));
    }
}

#[test]
fn blue_expansion_blood_response_host_return_cancels_without_refund_or_new_instance() {
    let mut g = initial(0);
    let vampire = board(&mut g, "JZ27", 0, 2);
    let rescuer = board(&mut g, "JC075", 0, 0);
    fund(&mut g, 0, "BQ040", 3);
    let id = held(&mut g, "BQ040", 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            target_id: Some(vampire.clone()),
            ..Action::new("play")
        },
    );
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(rescuer),
            ability_id: Some("rescue".into()),
            target_id: Some(vampire.clone()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    assert!(g.board(&vampire).is_none());
    pass_top(&mut g);
    assert!(g.attachments.is_empty());
    assert_eq!(g.resources(0), 0);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ040"));
    assert!(g.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "JZ27" && c.id != vampire));
}

#[test]
fn blue_expansion_blood_move_preserves_and_host_departure_removes_to_owner() {
    let mut g = initial(0);
    let vampire = board(&mut g, "JZ27", 2, 2);
    fund(&mut g, 0, "BQ040", 1);
    let a = play(&mut g, 0, "BQ040", &vampire);
    // Explicit fixture: losing the current vampire character face is a
    // continuous host-condition failure, independent of new targeting.
    g.board_mut(&vampire).unwrap().face_down = true;
    g.settle_deaths();
    assert!(g.attachments.is_empty());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ040" && c.id != a));
    let mut g = initial(0);
    let vampire = board(&mut g, "JZ27", 2, 2);
    fund(&mut g, 0, "BQ040", 1);
    let a = play(&mut g, 0, "BQ040", &vampire);
    let c = g.regions[2].cards.remove(0);
    g.regions[1].cards.push(c); // Explicit location-only fixture; not departure.
    assert_eq!(g.attachment_region(&g.attachments[0]), Some(1));
    checkpoint(&g);
    g.remove_dead(&vampire, RemovalCause::Destroy);
    g.drive().unwrap();
    assert!(g.attachments.is_empty());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ040" && c.id != a));
    assert!(g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JZ27"));
    checkpoint(&g);
}

#[test]
fn blue_expansion_blood_gained_vampire_and_lost_type_use_real_attachment_pipeline() {
    let mut g = initial(0);
    let target = board(&mut g, "JC125", 2, 2);
    fund(&mut g, 0, "JC036", 6);
    let embrace = play(&mut g, 0, "JC036", &target);
    assert_eq!(g.board(&target).unwrap().1.controller, 0);
    assert!(g
        .current_subtypes(g.board(&target).unwrap().1)
        .contains(&"吸血鬼".into()));
    let blood = play(&mut g, 0, "BQ040", &target);
    assert_eq!(g.defense(g.board(&target).unwrap().1, 2), 2);
    fund(&mut g, 0, "JC005", 2);
    let collapse = held(&mut g, "JC005", 0);
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&collapse)
                && a.action.target_id.as_deref() == Some(&embrace)
        })
        .expect("real disintegrate response")
        .action;
    apply(&mut g, 0, a);
    pass_top(&mut g);
    assert!(g.attachments.is_empty());
    assert_eq!(g.board(&target).unwrap().1.controller, 2);
    assert_eq!(g.defense(g.board(&target).unwrap().1, 2), 1);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "BQ040" && c.id != blood));
    checkpoint(&g);
}

#[test]
fn blue_expansion_rat_normal_deploy_and_secret_deploy_do_not_inspect() {
    for hidden in [false, true] {
        let mut g = initial(0);
        fund(&mut g, 0, "BQ028", 2);
        let id = held(&mut g, "BQ028", 0);
        apply(
            &mut g,
            0,
            Action {
                card_id: Some(id),
                region: Some(2),
                ..Action::new(if hidden { "conceal" } else { "deploy" })
            },
        );
        if !hidden {
            pass_top(&mut g);
        }
        assert!(g.pending.is_none() && g.stack.is_empty());
        assert_eq!(g.resources(0), usize::from(hidden) as u32);
    }
}

#[test]
fn blue_expansion_rat_equipment_forbidden_but_non_equipment_status_allowed() {
    let mut g = initial(0);
    let rat = board(&mut g, "BQ028", 0, 2);
    fund(&mut g, 0, "JC104", 4);
    let vest = held(&mut g, "XQ47", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(vest),
            target_id: Some(rat.clone()),
            ..Action::new("play")
        },
    );
    let protector = board(&mut g, "JC071", 0, 2);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(protector),
            target_id: Some(rat.clone()),
            ability_id: Some("protect-local-character".into()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    fund(&mut g, 0, "JC089", 2);
    play(&mut g, 0, "JC089", &rat);
    assert!(g
        .attachments
        .iter()
        .any(|a| a.card.definition == "JC089" && a.host_id == rat));
    assert_eq!(g.defense(g.board(&rat).unwrap().1, 2), 1);
}

#[test]
fn blue_expansion_rat_all_player_targets_private_inspection_optional_attachment_only() {
    for target in 0..4 {
        for discard in [false, true] {
            let mut g = initial(0);
            let other = held(&mut g, "JC059", target);
            let attachment = held(&mut g, "XQ14", target);
            let rat = declare_reveal(&mut g, 0);
            inspect(&mut g, target);
            let p = g.pending.as_ref().unwrap();
            assert_eq!(p.seat, 0);
            assert_eq!(
                p.choice
                    .options
                    .iter()
                    .map(|o| o.id.as_str())
                    .collect::<Vec<_>>(),
                [attachment.as_str()]
            );
            for viewer in 0..4 {
                let v = g.view(viewer);
                assert!(v.revealed_hands.is_empty());
                assert_eq!(v.pending_choice.is_some(), viewer == 0);
                if viewer == 0 {
                    let ids = v
                        .pending_choice
                        .unwrap()
                        .preview_cards
                        .into_iter()
                        .map(|c| c.instance_id)
                        .collect::<Vec<_>>();
                    assert_eq!(ids, [other.clone(), attachment.clone()]);
                } else if viewer != target {
                    assert!(!serde_json::to_string(&v).unwrap().contains(&other));
                    assert!(!serde_json::to_string(&v).unwrap().contains(&attachment));
                }
            }
            assert!(g
                .log
                .iter()
                .all(|entry| !entry.text.contains("月之女祭司") && !entry.text.contains("鲜血")));
            let cid = g.pending.as_ref().unwrap().choice.id.clone();
            reject(
                &mut g,
                1,
                Action {
                    choice_id: Some(cid.clone()),
                    selected: Some(vec![attachment.clone()]),
                    ..Action::new("choose")
                },
            );
            reject(
                &mut g,
                0,
                Action {
                    choice_id: Some(cid),
                    selected: Some(vec![other]),
                    ..Action::new("choose")
                },
            );
            choose(
                &mut g,
                if discard {
                    vec![attachment.clone()]
                } else {
                    vec![]
                },
            );
            assert!(g.pending.is_none() && g.stack.is_empty());
            assert!(g.board(&rat).is_some()); // No hacker sacrifice is imported.
            assert_eq!(
                g.players[target].hand.iter().any(|c| c.id == attachment),
                !discard
            );
            assert_eq!(
                g.players[target]
                    .graveyard
                    .iter()
                    .filter(|c| c.definition == "XQ14")
                    .count(),
                usize::from(discard)
            );
            assert!(g.players[target]
                .graveyard
                .iter()
                .all(|c| c.id != attachment));
            for viewer in 0..4 {
                assert!(g.view(viewer).pending_choice.is_none());
            }
        }
    }
}

#[test]
fn blue_expansion_rat_empty_and_no_attachment_hands_still_confirm_private_inspection() {
    for n in [0, 2] {
        let mut g = initial(0);
        for _ in 0..n {
            held(&mut g, "JC125", 2);
        }
        declare_reveal(&mut g, 0);
        inspect(&mut g, 2);
        assert!(g.pending.as_ref().unwrap().choice.options.is_empty());
        assert_eq!(g.pending.as_ref().unwrap().choice.max, Some(0));
        assert_eq!(g.view(0).pending_choice.unwrap().preview_cards.len(), n);
        choose(&mut g, vec![]);
        assert_eq!(g.players[2].hand.len(), n);
        assert!(g.pending.is_none());
    }
}

#[test]
fn blue_expansion_rat_optional_trigger_decline_and_eliminated_target_rejected() {
    let mut g = initial(0);
    held(&mut g, "XQ14", 2);
    declare_reveal(&mut g, 0);
    choose(&mut g, vec![]);
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.players[2].hand.len(), 1);
    let mut g = initial(0);
    g.players[2].eliminated = true;
    declare_reveal(&mut g, 0);
    assert!(!g
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .any(|o| o.id == "p2"));
    let cid = g.pending.as_ref().unwrap().choice.id.clone();
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(cid),
            selected: Some(vec!["p2".into()]),
            ..Action::new("choose")
        },
    );
}

#[test]
fn blue_expansion_rat_real_response_inspects_live_hand_after_paid_fast_draw() {
    let mut g = initial(0);
    let attachment = held(&mut g, "XQ14", 2);
    let inspiration = held(&mut g, "XQ34", 2);
    let drawn = g.make_card("JC125", 2);
    g.players[2].deck.insert(0, drawn);
    fund(&mut g, 2, "JC075", 1);
    declare_reveal(&mut g, 0);
    choose(&mut g, vec!["p2".into()]);
    // Declaration keeps the current holder's window first. Both members of
    // that team pass through actual commands before the other team composes.
    for seat in [0, 1] {
        if g.priority_team == g.team(2) {
            break;
        }
        apply(&mut g, seat, Action::new("pass"));
    }
    assert_eq!(g.priority_team, g.team(2));
    assert_eq!(g.stack.len(), 1);
    apply(
        &mut g,
        2,
        Action {
            card_id: Some(inspiration.clone()),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    pass_top(&mut g);
    assert_eq!(
        g.pending.as_ref().unwrap().choice.kind,
        "bq028-hand-inspect"
    );
    let p = g.view(0).pending_choice.unwrap();
    assert_eq!(g.resources(2), 0);
    assert_eq!(
        p.options.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        [attachment.as_str()]
    );
    assert_eq!(p.preview_cards.len(), 2);
    assert!(p
        .preview_cards
        .iter()
        .any(|c| c.card_id.as_deref() == Some("JC125")));
    assert!(p.preview_cards.iter().all(|c| c.instance_id != inspiration));
    choose(&mut g, vec![attachment]);
    assert_eq!(g.players[2].hand.len(), 1);
    assert_eq!(g.players[2].hand[0].definition, "JC125");
}

#[test]
fn blue_expansion_rat_source_departure_and_control_changes_do_not_rebind_actor() {
    for after_accept in [false, true] {
        for mutation in 0..3 {
            let mut g = initial(0);
            let attachment = held(&mut g, "XQ14", 2);
            let rat = declare_reveal(&mut g, 0);
            if after_accept {
                choose(&mut g, vec!["p2".into()]);
            }
            match mutation {
                0 => g.return_hand(&rat),
                1 => g.board_mut(&rat).unwrap().controller = 3,
                _ => {
                    g.remove_dead(&rat, RemovalCause::Destroy);
                }
            }
            checkpoint(&g);
            if !after_accept {
                choose(&mut g, vec!["p2".into()]);
            }
            pass_top(&mut g);
            assert_eq!(g.pending.as_ref().unwrap().seat, 0);
            choose(&mut g, vec![attachment]);
            assert!(g.players[2].hand.is_empty());
            assert!(g.players[2]
                .graveyard
                .iter()
                .any(|c| c.definition == "XQ14"));
        }
    }
}

#[test]
fn blue_expansion_rat_discard_uses_holder_hand_and_owner_graveyard() {
    let mut g = initial(0);
    let attachment = held(&mut g, "XQ14", 2);
    g.players[2].hand[0].owner = 3;
    declare_reveal(&mut g, 0);
    inspect(&mut g, 2);
    choose(&mut g, vec![attachment.clone()]);
    assert!(g.players[2].hand.is_empty() && g.players[2].graveyard.is_empty());
    let c = &g.players[3].graveyard[0];
    assert_eq!(c.definition, "XQ14");
    assert_ne!(c.id, attachment);
    assert_eq!(c.controller, 3);
}

#[test]
fn blue_expansion_rat_stale_same_print_hand_instance_is_rejected_atomically() {
    let mut g = initial(0);
    let attachment = held(&mut g, "XQ14", 2);
    declare_reveal(&mut g, 0);
    inspect(&mut g, 2);
    let mut c = g.players[2].hand.remove(0);
    c = g.reset_zone_card(c);
    assert_ne!(c.id, attachment);
    g.players[2].hand.push(c);
    let before = serde_json::to_string(&g).unwrap();
    let p = g.pending.as_ref().unwrap();
    assert!(g
        .apply(
            0,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(vec![attachment]),
                ..Action::new("choose")
            }
        )
        .is_err());
    assert_eq!(serde_json::to_string(&g).unwrap(), before);
    assert!(Game::from_persisted(&before).is_err());
}

#[test]
fn blue_expansion_rat_closed_definitions_programs_and_persisted_frame_guards() {
    for id in ["BQ028", "BQ040"] {
        let mut d = definition(id).clone();
        d.traits.barrier = true;
        assert!(blue_expansion::validate_definition(id, &d).is_err());
    }
    for mutation in 0..6 {
        let mut a = definition("BQ028").abilities[0].clone();
        match mutation {
            0 => a.event = Some(Event::Enter),
            1 => a.costs.push(Cost::Assets(1)),
            2 => a.targets[0].relation = Relation::EnemyTeam,
            3 => a.ops.push(a.ops[0].clone()),
            4 => a.response_policy = ResponsePolicy::Immediate,
            _ => a.key = "alias".into(),
        }
        assert!(validate_ability("BQ028", &a).is_err());
        assert!(validate_ability("JC125", &a).is_err());
    }
    let mut g = initial(0);
    held(&mut g, "XQ14", 2);
    declare_reveal(&mut g, 0);
    let v = serde_json::to_value(&g).unwrap();
    for mutation in 0..6 {
        let mut bad = v.clone();
        let p = &mut bad["pending"];
        match mutation {
            0 => {
                p["resolution"]["Declare"]["declaration"]["source"]["card"]["controller"] = 3.into()
            }
            1 => p["resolution"]["Declare"]["declaration"]["ability"]["event"] = "Enter".into(),
            2 => p["seat"] = 3.into(),
            3 => p["choice"]["max"] = 2.into(),
            4 => p["choice"]["options"][0]["id"] = "p9".into(),
            _ => {
                p["resolution"]["Declare"]["declaration"]["source"]["source_region_instance"] =
                    "".into()
            }
        }
        bad_state(&g, bad);
    }
    choose(&mut g, vec!["p2".into()]);
    let v = serde_json::to_value(&g).unwrap();
    for mutation in 0..9 {
        let mut bad = v.clone();
        let f = &mut bad["stack"][0]["frame"];
        match mutation {
            0 => f["source"]["card"]["definition"] = "JC125".into(),
            1 => f["steps"][0]["context"] = 3.into(),
            2 => f["cursor"] = 1.into(),
            3 => f["guard"] = "Accepted".into(),
            4 => f["targets"][0]["id"] = "p02".into(),
            5 => f["targets"][0]["spec"]["relation"] = "EnemyTeam".into(),
            6 => f["targets"][0]["public"]["controller"] = "p3".into(),
            7 => f["targets"][0]["public"]["valid"] = false.into(),
            _ => f["targets"][0]["public"]["label"] = "other-player".into(),
        }
        bad_state(&g, bad);
    }
    pass_top(&mut g);
    let v = serde_json::to_value(&g).unwrap();
    for mutation in 0..11 {
        let mut bad = v.clone();
        let p = &mut bad["pending"];
        match mutation {
            0 => p["resolution"]["Frame"]["frame"]["cursor"] = 0.into(),
            1 => p["resolution"]["Frame"]["frame"]["guard"] = "Unchecked".into(),
            2 => p["resolution"]["Frame"]["choice"]["BQ028InspectAttachments"]["seat"] = 3.into(),
            3 => {
                p["resolution"]["Frame"]["choice"]["BQ028InspectAttachments"]["inspected"][0]
                    ["id"] = "new-instance".into()
            }
            4 => p["choice"]["options"][0]["id"] = "fake".into(),
            5 => p["choice"]["max"] = 2.into(),
            6 => p["seat"] = 2.into(),
            7 => {
                p["choice"]["previewCards"] =
                    serde_json::to_value(g.view(0).pending_choice.unwrap().preview_cards).unwrap()
            }
            8 => p["choice"]["id"] = "".into(),
            9 => p["choice"]["title"] = "unrelated".into(),
            _ => p["choice"]["description"] = "public-inspect".into(),
        }
        bad_state(&g, bad);
    }
}

#[test]
fn blue_expansion_rat_room_roundtrip_repeat_and_wrong_version_choice_are_atomic() {
    let mut g = initial(0);
    let attachment = held(&mut g, "XQ14", 2);
    declare_reveal(&mut g, 0);
    inspect(&mut g, 2);
    let r = envelope(&g);
    let command = RoomCommand {
        command_id: "blue-inspection-choice".into(),
        expected_version: r.revision,
        action: SessionAction::Game {
            action: Action {
                choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
                selected: Some(vec![attachment]),
                ..Action::new("choose")
            },
        },
    };
    let outcome = r.transition(0, Some(command.clone()), 0).unwrap();
    assert!(outcome.error_code.is_none(), "{:?}", outcome.error_message);
    let after = RoomEnvelope::from_persisted(&outcome.state).unwrap();
    assert_eq!(after.game.players[2].graveyard.len(), 1);
    let duplicate = after.transition(0, Some(command.clone()), 0).unwrap();
    assert_eq!(duplicate.state, outcome.state);
    assert_eq!(duplicate.error_code.as_deref(), Some("version_conflict"));
    let wrong = RoomCommand {
        command_id: "blue-invalid-new-version".into(),
        expected_version: after.revision,
        ..command
    };
    let rejected = after.transition(0, Some(wrong), 0).unwrap();
    assert!(rejected.error_code.is_some());
    assert_eq!(rejected.state, outcome.state);
    assert_eq!(after.game.players[2].graveyard.len(), 1);
}

#[test]
fn blue_expansion_rat_actual_deploy_and_reveal_stack_identity_rejects_single_field_changes() {
    for reveal in [false, true] {
        let mut g = initial(0);
        fund(&mut g, 0, "BQ028", 2);
        let id = if reveal {
            let id = board(&mut g, "BQ028", 0, 2);
            g.board_mut(&id).unwrap().face_down = true;
            id
        } else {
            held(&mut g, "BQ028", 0)
        };
        apply(
            &mut g,
            0,
            Action {
                card_id: Some(id),
                region: (!reveal).then_some(2),
                ..Action::new(if reveal { "reveal" } else { "deploy" })
            },
        );
        checkpoint(&g);
        let v = serde_json::to_value(&g).unwrap();
        for mutation in 0..10 {
            let mut bad = v.clone();
            let source_id = bad["stack"][0]["frame"]["source"]["card"]["id"].clone();
            let item = &mut bad["stack"][0];
            match mutation {
                0 => item["card"]["controller"] = 3.into(),
                1 => item["card"]["face_down"] = true.into(),
                2 => item["card"]["id"] = "".into(),
                3 => item["card"]["id"] = source_id,
                4 => item["card"]["definition"] = "JC125".into(),
                5 => item["controller"] = 3.into(),
                6 => item["deploy_region"] = 1.into(),
                7 => item["reveal"] = (!reveal).into(),
                8 => item["frame"]["ability_key"] = "reveal-alias".into(),
                _ => item["frame"]["source"]["source_region_instance"] = serde_json::Value::Null,
            }
            bad_state(&g, bad);
        }
    }
}

#[test]
fn blue_expansion_rat_actual_granted_rewards_bind_frozen_source_and_reject_bad_pending() {
    for glory in [false, true] {
        let mut g = initial(0);
        let rat = board(&mut g, "BQ028", 0, 2);
        if glory {
            let protector = board(&mut g, "JC071", 0, 2);
            apply(
                &mut g,
                0,
                Action {
                    card_id: Some(protector),
                    target_id: Some(rat.clone()),
                    ability_id: Some("protect-local-character".into()),
                    ..Action::new("activate")
                },
            );
            pass_top(&mut g);
            fund(&mut g, 0, "JC089", 2);
            play(&mut g, 0, "JC089", &rat);
            g.begin_window(Window::Before(2, 1));
            g.close_window().unwrap(); // Real combat result dispatches the granted glory.
        } else {
            fund(&mut g, 0, "JC074", 2);
            let grant = held(&mut g, "JC074", 0);
            let a = g
                .legal_actions(0)
                .into_iter()
                .find(|a| {
                    a.action.kind == "play"
                        && a.action.card_id.as_deref() == Some(&grant)
                        && a.action.target_id.as_deref() == Some(&rat)
                })
                .unwrap()
                .action;
            apply(&mut g, 0, a);
            pass_top(&mut g);
            g.begin_window(Window::After(2, 2));
            let original = g.regions[2].card.id.clone();
            g.declare_region_renown(2, &original);
        }
        g.drive().unwrap();
        let p = g
            .pending
            .as_ref()
            .expect("actual granted reward declaration");
        let ChoiceResolution::Declare {
            declaration,
            stage: DeclareChoice::Accept,
        } = &p.resolution
        else {
            panic!()
        };
        assert_eq!(
            declaration.ability.key,
            if glory {
                "jc089-combat-glory"
            } else {
                "renown"
            }
        );
        assert_eq!(declaration.source.card.id, rat);
        assert_eq!(
            declaration.source.source_region_instance.as_ref(),
            Some(&g.regions[2].card.id)
        );
        checkpoint(&g);
        let other_region = g.regions[1].card.id.clone();
        let v = serde_json::to_value(&g).unwrap();
        for mutation in 0..13 {
            let mut bad = v.clone();
            let p = &mut bad["pending"];
            match mutation {
                0 => p["seat"] = 3.into(),
                1 => p["choice"]["playerId"] = "p3".into(),
                2 => p["choice"]["kind"] = "target".into(),
                3 => p["choice"]["min"] = 1.into(),
                4 => p["choice"]["max"] = 2.into(),
                5 => p["choice"]["allowDecline"] = false.into(),
                6 => p["choice"]["options"][0]["id"] = "fake".into(),
                7 => p["resolution"]["Declare"]["stage"] = "Target".into(),
                8 => {
                    p["resolution"]["Declare"]["declaration"]["source"]["source_region_instance"] =
                        other_region.clone().into()
                }
                9 => {
                    p["resolution"]["Declare"]["declaration"]["ability"]["ops"][0]
                        ["PlaceInfluence"]["region_instance"] = other_region.clone().into()
                }
                10 => p["choice"]["id"] = "".into(),
                11 => p["choice"]["title"] = "bad-reward".into(),
                _ => p["choice"]["description"] = "bad-reward".into(),
            }
            bad_state(&g, bad);
        }
        choose(&mut g, vec!["accept".into()]);
        checkpoint(&g);
        let mut bad = serde_json::to_value(&g).unwrap();
        bad["stack"][0]["frame"]["steps"][0]["op"]["PlaceInfluence"]["region_instance"] =
            other_region.into();
        bad_state(&g, bad);
        pass_top(&mut g);
        assert_eq!(g.regions[2].influence, [1, 0]);
    }
}

#[test]
fn blue_expansion_rat_unknown_inspected_definition_returns_error_in_game_and_room_restore() {
    let mut g = initial(0);
    held(&mut g, "XQ14", 2);
    declare_reveal(&mut g, 0);
    inspect(&mut g, 2);
    let mut bad = serde_json::to_value(&g).unwrap();
    bad["players"][2]["hand"][0]["definition"] = "UNKNOWN-BLUE-CARD".into();
    bad["pending"]["resolution"]["Frame"]["choice"]["BQ028InspectAttachments"]["inspected"][0]
        ["definition"] = "UNKNOWN-BLUE-CARD".into();
    bad_state(&g, bad);
}
