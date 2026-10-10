//! User-ruling regressions. Accumulated marker/attachment layouts are explicit
//! Native fixtures; payments, declarations, response guards and Room pause use
//! production calls. A pending choice blocks other game actions, so stale-board
//! cases below deliberately exercise defensive invariants, not legal interleaving.
use crate::jc029_tests::{
    apply, board, checkpoint, choose, envelope, fund, game, pass_top, reject,
};
use crate::{catalog, model::*, room::*, rules::*};
use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicUsize, Ordering},
};
static SEQ: AtomicUsize = AtomicUsize::new(0);

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
fn attachment(g: &mut Game, def: &str, seat: usize, host: &str, marks: u32) -> String {
    let mut c = g.make_card(def, seat);
    c.time_markers = marks;
    let id = c.id.clone();
    g.attachments.push(Attachment {
        card: c,
        host_id: host.into(),
    });
    id
}
fn reveal(g: &mut Game, actor: usize) -> String {
    let old = board(g, "XQ18", actor, 2);
    g.board_mut(&old).unwrap().face_down = true;
    fund(g, actor, "XQ18", 2);
    apply(
        g,
        actor,
        Action {
            card_id: Some(old.clone()),
            ..Action::new("reveal")
        },
    );
    pass_top(g);
    let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
    else {
        panic!()
    };
    let id = declaration.source.card.id.clone();
    assert_ne!(id, old);
    id
}
fn declare(g: &mut Game, actor: usize, target: &str, add: bool) -> String {
    let source = reveal(g, actor);
    choose(
        g,
        vec![if add {
            "add-time".into()
        } else {
            "remove-time".into()
        }],
    );
    choose(g, vec![target.into()]);
    assert!(g.pending.is_none());
    source
}
fn carrier_ids(g: &Game) -> BTreeSet<String> {
    let p = g.pending.as_ref().unwrap();
    assert_eq!(p.choice.kind, "xq18-time-carrier");
    assert_eq!(
        (
            p.choice.min,
            p.choice.max,
            p.choice.amount,
            p.choice.allow_decline
        ),
        (Some(1), Some(1), Some(1), Some(false))
    );
    assert!(p.choice.options.iter().all(|o| o.card.is_none()));
    p.choice.options.iter().map(|o| o.id.clone()).collect()
}
fn priority(g: &mut Game, actor: usize) {
    for _ in 0..4 {
        if g.priority_team == g.team(actor) {
            return;
        }
        let s = (0..4)
            .find(|&s| g.legal_actions(s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        apply(g, s, Action::new("pass"));
    }
    panic!("response priority");
}
fn cast_response(g: &mut Game, def: &str, actor: usize, target: &str) {
    priority(g, actor);
    let h = held(g, def, actor);
    let a = g
        .legal_actions(actor)
        .into_iter()
        .find(|a| {
            a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(&h)
                && a.action.target_id.as_deref() == Some(target)
        })
        .unwrap()
        .action;
    apply(g, actor, a);
    pass_top(g);
}
fn bad(g: &Game, v: serde_json::Value) {
    let a = std::panic::catch_unwind(|| Game::from_persisted(&serde_json::to_string(&v).unwrap()));
    assert!(a.is_ok() && a.unwrap().is_err());
    let mut r = serde_json::to_value(envelope(g)).unwrap();
    r["game"] = v;
    let state = serde_json::to_string(&r).unwrap();
    let a = std::panic::catch_unwind(|| RoomEnvelope::from_persisted(&state));
    assert!(a.is_ok() && a.unwrap().is_err());
    if let Ok(dir) = std::env::var("XQ18_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        std::fs::write(
            format!("{dir}/invalid-{n:04}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}
fn pending() -> (Game, String, String) {
    let mut g = initial(0);
    let t = board(&mut g, "JC125", 2, 0);
    let a = attachment(&mut g, "XQ47", 1, &t, 2);
    declare(&mut g, 0, &t, false);
    pass_top(&mut g);
    (g, t, a)
}

#[test]
fn xq18_printed_fields_and_two_exact_modes() {
    let c = catalog::card("XQ18");
    assert_eq!(
        (&*c.name, c.cost, c.defense, &*c.magic),
        ("计时人", 2, Some(1), "死亡")
    );
    assert_eq!(c.loyalty, ["红色", "红色"]);
    assert_eq!(c.subtypes, ["人类", "法师", "邪教徒"]);
    assert_eq!(c.permanent_icons, Icons::default());
    assert_eq!(
        c.temporary_icons,
        Icons {
            investigation: 1,
            ..Icons::default()
        }
    );
    assert!(!c.unique && c.keywords.is_empty() && c.supported);
    let d = definition("XQ18");
    assert!(!d.traits.public);
    assert_eq!(d.abilities.len(), 1);
    let a = &d.abilities[0];
    assert_eq!(a.event, Some(Event::Reveal));
    assert_eq!(a.response_policy, ResponsePolicy::Respondable);
    assert!(a.costs.is_empty());
    assert!(matches!(
        a.modes[0].ops.as_slice(),
        [Op::XQ18AddOneTimeToTarget]
    ));
    assert!(matches!(
        a.modes[1].ops.as_slice(),
        [Op::XQ18ChooseTimeCarrier]
    ));
    crate::red_time::validate_definition("XQ18", d).unwrap();
}

#[test]
fn xq18_real_deploy_conceal_paid_reveal_and_optional_trigger() {
    let mut g = initial(0);
    fund(&mut g, 0, "XQ18", 2);
    let h = held(&mut g, "XQ18", 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(h),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    assert!(g.pending.is_none());
    let visible = g.regions[2].cards[0].id.clone();
    assert_eq!(
        g.current_icons(g.board(&visible).unwrap().1, 2)
            .investigation,
        1
    );
    for c in &mut g.players[0].assets {
        c.exhausted = false;
    }
    let h = held(&mut g, "XQ18", 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(h),
            region: Some(1),
            ..Action::new("conceal")
        },
    );
    let hidden = g.regions[1].cards[0].id.clone();
    assert!(g.pending.is_none());
    for c in &mut g.players[0].assets {
        c.exhausted = false;
    }
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(hidden.clone()),
            ..Action::new("reveal")
        },
    );
    assert_eq!(
        g.players[0].assets.iter().filter(|c| c.exhausted).count(),
        2
    );
    pass_top(&mut g);
    assert!(g.board(&hidden).is_none());
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 2);
    choose(&mut g, vec![]);
    assert!(g.pending.is_none());
    let visible = g.regions[1].cards[0].id.clone();
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(visible),
            ..Action::new("reveal")
        },
    );
}

#[test]
fn xq18_cost_double_red_and_wrong_actor_are_atomic() {
    for defs in [vec!["JC125", "JC125"], vec!["JZ30", "JC125"], vec!["JZ30"]] {
        let mut g = initial(0);
        for def in defs {
            fund(&mut g, 0, def, 1);
        }
        let h = held(&mut g, "XQ18", 0);
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(h),
                region: Some(2),
                ..Action::new("deploy")
            },
        );
        let hidden = board(&mut g, "XQ18", 0, 2);
        g.board_mut(&hidden).unwrap().face_down = true;
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(hidden.clone()),
                ..Action::new("reveal")
            },
        );
        reject(
            &mut g,
            2,
            Action {
                card_id: Some(hidden),
                ..Action::new("reveal")
            },
        );
    }
}

#[test]
fn xq18_add_places_one_on_only_original_physical_target() {
    for attached in [false, true] {
        let mut g = initial(0);
        let t = board(&mut g, "JC125", 2, 0);
        let a = attachment(&mut g, "XQ47", 1, &t, 2);
        g.board_mut(&t).unwrap().time_markers = 3;
        declare(&mut g, 0, if attached { &a } else { &t }, true);
        pass_top(&mut g);
        assert!(g.pending.is_none());
        assert_eq!(
            g.board(&t).unwrap().1.time_markers,
            3 + u32::from(!attached)
        );
        assert_eq!(g.board(&a).unwrap().1.time_markers, 2 + u32::from(attached));
    }
}

#[test]
fn xq18_body_zero_attachment_one_requires_actor_carrier_choice() {
    for actor in 0..4 {
        let mut g = initial(actor);
        let t = board(&mut g, "JC125", (actor + 2) % 4, 0);
        let a = attachment(&mut g, "XQ47", (actor + 1) % 4, &t, 1);
        let other = board(&mut g, "JC125", actor, 1);
        g.board_mut(&other).unwrap().time_markers = 4;
        declare(&mut g, actor, &t, false);
        pass_top(&mut g);
        assert_eq!(carrier_ids(&g), BTreeSet::from([a.clone()]));
        assert_eq!(g.pending.as_ref().unwrap().seat, actor);
        for s in 0..4 {
            assert_eq!(g.view(s).pending_choice.is_some(), s == actor);
        }
        assert_eq!(g.board(&a).unwrap().1.time_markers, 1);
        choose(&mut g, vec![a.clone()]);
        assert_eq!(g.board(&a).unwrap().1.time_markers, 0);
        assert_eq!(g.board(&t).unwrap().1.time_markers, 0);
        assert_eq!(g.board(&other).unwrap().1.time_markers, 4);
    }
}

#[test]
fn xq18_multiple_carriers_choose_each_without_body_priority() {
    for selected in 0..3 {
        let mut g = initial(0);
        let t = board(&mut g, "JC125", 2, 0);
        g.board_mut(&t).unwrap().time_markers = 2;
        let a = attachment(&mut g, "XQ47", 0, &t, 3);
        let b = attachment(&mut g, "XQ47", 3, &t, 4);
        declare(&mut g, 0, &t, false);
        pass_top(&mut g);
        let ids = [t, a, b];
        assert_eq!(carrier_ids(&g), ids.iter().cloned().collect());
        choose(&mut g, vec![ids[selected].clone()]);
        for (i, id) in ids.iter().enumerate() {
            assert_eq!(
                g.board(id).unwrap().1.time_markers,
                (i + 2) as u32 - u32::from(i == selected)
            );
        }
    }
}

#[test]
fn xq18_same_name_same_markers_carriers_have_public_stable_identity() {
    // Equal printed cards and marker counts, including equal owners, must be
    // distinguishable in the existing text-only ChoicePanel. Controllers are
    // explicit prepared state; the frozen actor still performs the real choice.
    for same_owner in [false, true] {
        for same_controller in [false, true] {
            for selected in 0..2 {
                let mut g = initial(0);
                let target = board(&mut g, "JC125", 2, 0);
                let a = attachment(&mut g, "XQ47", 0, &target, 3);
                let b = attachment(&mut g, "XQ47", if same_owner { 0 } else { 2 }, &target, 3);
                g.board_mut(&b).unwrap().controller = if same_controller { 0 } else { 3 };
                let hidden = board(&mut g, "XQ18", 2, 1);
                g.board_mut(&hidden).unwrap().face_down = true;
                declare(&mut g, 0, &target, false);
                pass_top(&mut g);
                let ids = [a, b];
                assert_eq!(carrier_ids(&g), ids.iter().cloned().collect());
                let choice = g.view(0).pending_choice.unwrap();
                assert_ne!(choice.options[0].label, choice.options[1].label);
                for (index, id) in ids.iter().enumerate() {
                    let option = choice.options.iter().find(|o| o.id == *id).unwrap();
                    assert!(option.label.contains("防弹战术背心"));
                    assert!(option.label.contains("3个时间标志"));
                    assert!(option.label.contains(&format!("#{id}")));
                    assert!(option.label.contains(if index == 0 || same_owner {
                        "1号席拥有"
                    } else {
                        "3号席拥有"
                    }));
                    assert!(option.card.is_none());
                    assert!(!option.label.contains(&hidden));
                }
                for seat in 1..4 {
                    assert!(g.view(seat).pending_choice.is_none());
                }
                let projected = g.card_view(g.board(&hidden).unwrap().1, 0, Some(1), None);
                assert_eq!(projected.name, "暗藏者");
                assert!(projected.card_id.is_none() && projected.text.is_none());
                let valid = g.clone();
                let mut tampered = serde_json::to_value(&g).unwrap();
                tampered["pending"]["choice"]["options"][1]["label"] =
                    serde_json::json!(choice.options[0].label);
                bad(&valid, tampered);
                checkpoint(&g);
                choose(&mut g, vec![ids[selected].clone()]);
                for (index, id) in ids.iter().enumerate() {
                    assert_eq!(
                        g.board(id).unwrap().1.time_markers,
                        3 - u32::from(index == selected)
                    );
                }
            }
        }
    }
}

#[test]
fn xq18_attachment_target_excludes_host_and_siblings() {
    let mut g = initial(0);
    let t = board(&mut g, "JC125", 2, 0);
    g.board_mut(&t).unwrap().time_markers = 3;
    let a = attachment(&mut g, "XQ47", 0, &t, 2);
    let b = attachment(&mut g, "XQ47", 1, &t, 4);
    declare(&mut g, 0, &a, false);
    pass_top(&mut g);
    assert_eq!(carrier_ids(&g), BTreeSet::from([a.clone()]));
    choose(&mut g, vec![a.clone()]);
    assert_eq!(g.board(&a).unwrap().1.time_markers, 1);
    assert_eq!(g.board(&t).unwrap().1.time_markers, 3);
    assert_eq!(g.board(&b).unwrap().1.time_markers, 4);
}

#[test]
fn xq18_empty_marker_set_finishes_without_choice_or_other_removal() {
    for target_attachment in [false, true] {
        let mut g = initial(0);
        let t = board(&mut g, "JC125", 2, 0);
        let a = attachment(&mut g, "XQ47", 1, &t, 0);
        let other = board(&mut g, "JC125", 1, 1);
        g.board_mut(&other).unwrap().time_markers = 5;
        declare(&mut g, 0, if target_attachment { &a } else { &t }, false);
        pass_top(&mut g);
        assert!(g.pending.is_none() && g.stack.is_empty());
        assert_eq!(g.board(&other).unwrap().1.time_markers, 5);
    }
}

#[test]
fn xq18_all_seats_relations_and_regions_can_choose_target_body() {
    for actor in 0..4 {
        for controller in 0..4 {
            let mut g = initial(actor);
            let t = board(&mut g, "JC125", controller, 0);
            g.board_mut(&t).unwrap().time_markers = 2;
            declare(&mut g, actor, &t, false);
            pass_top(&mut g);
            assert_eq!(carrier_ids(&g), BTreeSet::from([t.clone()]));
            choose(&mut g, vec![t.clone()]);
            assert_eq!(g.board(&t).unwrap().1.time_markers, 1);
        }
    }
}

#[test]
fn xq18_wrong_seat_count_unrelated_carrier_and_old_choice_rejected() {
    let (mut g, _, a) = pending();
    let p = g.pending.clone().unwrap();
    let other = board(&mut g, "JC125", 0, 1);
    g.board_mut(&other).unwrap().time_markers = 4;
    for (seat, ids) in [
        (2, vec![a.clone()]),
        (0, vec![]),
        (0, vec![a.clone(), a.clone()]),
        (0, vec![other.clone()]),
    ] {
        reject(
            &mut g,
            seat,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(ids),
                ..Action::new("choose")
            },
        );
    }
    let action = Action {
        choice_id: Some(p.choice.id),
        selected: Some(vec![a.clone()]),
        ..Action::new("choose")
    };
    apply(&mut g, 0, action.clone());
    assert_eq!(g.board(&a).unwrap().1.time_markers, 1);
    reject(&mut g, 0, action);
}

#[test]
fn xq18_original_target_legality_hidden_assets_hand_grave_region_and_barrier() {
    let mut g = initial(0);
    let hidden = board(&mut g, "JC125", 2, 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    let public = board(&mut g, "JC125", 2, 1);
    let h = held(&mut g, "JC125", 2);
    fund(&mut g, 2, "JC125", 1);
    let grave = g.make_card("JC125", 2);
    let grave_id = grave.id.clone();
    g.players[2].graveyard.push(grave);
    // Barrier is a real valid prepared attachment, not a target-rule mock.
    attachment(&mut g, "JC073", 2, &public, 0);
    reveal(&mut g, 0);
    choose(&mut g, vec!["remove-time".into()]);
    let ids = g
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .map(|o| o.id.clone())
        .collect::<BTreeSet<_>>();
    for id in [
        hidden,
        h,
        grave_id,
        g.players[2].assets[0].id.clone(),
        g.regions[0].card.id.clone(),
    ] {
        assert!(!ids.contains(&id));
    }
    assert!(!ids.contains(&public));
    choose(&mut g, vec![]);
    assert!(g.pending.is_none());
}

#[test]
fn xq18_real_source_departure_and_prepared_control_preserve_frozen_actor() {
    for transfer in [false, true] {
        let mut g = initial(0);
        let t = board(&mut g, "JC125", 2, 0);
        g.board_mut(&t).unwrap().time_markers = 2;
        let source = declare(&mut g, 0, &t, false);
        // Existing control spells are Standard: this branch is an explicit control-state fixture, not a legal fast spell.
        if transfer {
            g.board_mut(&source).unwrap().controller = 2;
            checkpoint(&g);
        } else {
            fund(&mut g, 2, "JC084", 3);
            cast_response(&mut g, "JC091", 2, &source);
            assert!(g.board(&source).is_none());
        }
        pass_top(&mut g);
        assert_eq!(g.pending.as_ref().unwrap().seat, 0);
        choose(&mut g, vec![t.clone()]);
        assert_eq!(g.board(&t).unwrap().1.time_markers, 1);
    }
}

#[test]
fn xq18_real_rescue_response_cancels_original_target_without_repayment() {
    let mut g = initial(0);
    let t = board(&mut g, "JC125", 2, 0);
    g.board_mut(&t).unwrap().time_markers = 2;
    let rescue = board(&mut g, "JC075", 2, 1);
    fund(&mut g, 2, "JC075", 2);
    declare(&mut g, 0, &t, false);
    priority(&mut g, 2);
    apply(
        &mut g,
        2,
        Action {
            card_id: Some(rescue),
            target_id: Some(t.clone()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    pass_top(&mut g);
    assert!(g.pending.is_none() && g.board(&t).is_none());
    assert!(g.players[2].hand.iter().all(|c| c.time_markers == 0));
    assert_eq!(
        g.players[0].assets.iter().filter(|c| c.exhausted).count(),
        2
    );
}

#[test]
fn xq18_real_destroy_attachment_response_recomputes_current_carriers() {
    let mut g = initial(0);
    let t = board(&mut g, "JC125", 2, 0);
    let a = attachment(&mut g, "XQ47", 1, &t, 1);
    declare(&mut g, 0, &t, false);
    fund(&mut g, 2, "JC005", 2);
    cast_response(&mut g, "JC005", 2, &a);
    assert!(g.board(&a).is_none());
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert!(g.players[1].graveyard.iter().all(|c| c.time_markers == 0));
}

#[test]
fn xq18_real_standard_reattach_before_declaration_excludes_detached_carrier() {
    let mut g = initial(0);
    let t = board(&mut g, "JC125", 2, 0);
    let other = board(&mut g, "JC125", 1, 1);
    let a = attachment(&mut g, "XQ47", 0, &t, 2);
    fund(&mut g, 0, "XQ47", 2);
    let action = g
        .legal_actions(0)
        .into_iter()
        .find(|x| {
            x.action.kind == "activate"
                && x.action.card_id.as_deref() == Some(&a)
                && x.action.target_id.as_deref() == Some(&other)
                && x.action.ability_id.as_deref() == Some("reattach")
        })
        .unwrap()
        .action;
    apply(&mut g, 0, action);
    pass_top(&mut g);
    declare(&mut g, 0, &t, false);
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert_eq!(g.board(&a).unwrap().1.time_markers, 2);
}

#[test]
fn xq18_prepared_target_control_change_keeps_any_relation_and_operator() {
    let mut g = initial(0);
    let t = board(&mut g, "JC125", 1, 0);
    g.board_mut(&t).unwrap().time_markers = 2;
    // Explicit post-declaration control fixture; no Standard control card is claimed to respond.
    declare(&mut g, 0, &t, false);
    g.board_mut(&t).unwrap().controller = 2;
    checkpoint(&g);
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    choose(&mut g, vec![t.clone()]);
    assert_eq!(g.board(&t).unwrap().1.time_markers, 1);
}

#[test]
fn xq18_stale_pending_carrier_departure_detachment_hiding_empty_and_replacement_are_atomic() {
    for variant in 0..5 {
        let (mut g, t, a) = pending();
        let valid = g.clone();
        match variant {
            0 => g.return_hand(&a),
            1 => {
                let other = board(&mut g, "JC125", 1, 1);
                g.reattach_source(&a, &other);
            }
            2 => g.board_mut(&a).unwrap().face_down = true,
            3 => g.board_mut(&a).unwrap().time_markers = 0,
            _ => {
                g.return_hand(&a);
                attachment(&mut g, "XQ47", 1, &t, 2);
            }
        }
        let v = serde_json::to_value(&g).unwrap();
        bad(&valid, v);
        let p = g.pending.clone().unwrap();
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    choice_id: Some(p.choice.id),
                    selected: Some(vec![a]),
                    ..Action::new("choose")
                }
            )
            .is_err());
        assert_eq!(serde_json::to_string(&g).unwrap(), before);
    }
}

#[test]
fn xq18_pending_carrier_control_change_remains_legal_for_frozen_actor() {
    let (mut g, _, a) = pending();
    g.board_mut(&a).unwrap().controller = 3;
    checkpoint(&g);
    let p = g.pending.clone().unwrap();
    reject(
        &mut g,
        3,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![a.clone()]),
            ..Action::new("choose")
        },
    );
    choose(&mut g, vec![a.clone()]);
    assert_eq!(g.board(&a).unwrap().1.time_markers, 1);
}

#[test]
fn xq18_pending_original_target_departure_is_rejected() {
    let (mut g, t, a) = pending();
    let valid = g.clone();
    g.return_hand(&t);
    bad(&valid, serde_json::to_value(&g).unwrap());
    let p = g.pending.clone().unwrap();
    let before = serde_json::to_string(&g).unwrap();
    assert!(g
        .apply(
            0,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![a]),
                ..Action::new("choose")
            }
        )
        .is_err());
    assert_eq!(serde_json::to_string(&g).unwrap(), before);
}

fn room_step(r: &mut RoomEnvelope, seat: usize, action: SessionAction, now: u64) -> RoomCommand {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let cmd = RoomCommand {
        command_id: format!("xq18-room-{n}"),
        expected_version: r.revision,
        action,
    };
    let state = serde_json::to_string(r).unwrap();
    let out = r.transition(seat, Some(cmd.clone()), now).unwrap();
    assert!(out.error_code.is_none(), "{:?}", out.error_message);
    let next = RoomEnvelope::from_persisted(&out.state).unwrap();
    let replay = RoomEnvelope::from_persisted(&state)
        .unwrap()
        .transition(seat, Some(cmd.clone()), now)
        .unwrap();
    assert_eq!(
        serde_json::to_value(&out).unwrap(),
        serde_json::to_value(replay).unwrap()
    );
    *r = next;
    if let Ok(dir) = std::env::var("XQ18_ROOM_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/room-{n:04}.json"),
            serde_json::to_vec(
                &serde_json::json!({"state":state,"seat":seat,"command":cmd,"serverNow":now,
            "expected":out,"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    cmd
}

#[test]
fn xq18_real_room_pause_persist_resume_choose_and_replay() {
    let (g, _, a) = pending();
    let mut r = envelope(&g);
    let p = r.game.pending.clone().unwrap();
    room_step(&mut r, 2, SessionAction::PauseRoom, 1000);
    r = RoomEnvelope::from_persisted(&serde_json::to_string(&r).unwrap()).unwrap();
    assert_eq!(r.game.pending.as_ref().unwrap().choice.id, p.choice.id);
    assert_eq!(r.game.board(&a).unwrap().1.time_markers, 2);
    let blocked = r
        .transition(
            0,
            Some(RoomCommand {
                command_id: "paused-choice".into(),
                expected_version: r.revision,
                action: SessionAction::Game {
                    action: Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(vec![a.clone()]),
                        ..Action::new("choose")
                    },
                },
            }),
            2000,
        )
        .unwrap();
    assert_eq!(blocked.error_code.as_deref(), Some("room_paused"));
    room_step(&mut r, 1, SessionAction::ResumeRoom, 10000);
    let cmd = room_step(
        &mut r,
        0,
        SessionAction::Game {
            action: Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![a.clone()]),
                ..Action::new("choose")
            },
        },
        10000,
    );
    assert_eq!(r.game.board(&a).unwrap().1.time_markers, 1);
    let before = serde_json::to_string(&r).unwrap();
    let replay = r.transition(0, Some(cmd), 10000).unwrap();
    assert_eq!(replay.error_code.as_deref(), Some("version_conflict"));
    assert_eq!(replay.state, before);
}

#[test]
fn xq18_carrier_pending_metadata_program_and_transplant_tampering_rejected() {
    let (g, _, _) = pending();
    let base = serde_json::to_value(&g).unwrap();
    for (key, value) in [
        ("kind", serde_json::json!("trigger")),
        ("title", serde_json::json!("fake")),
        ("description", serde_json::json!("fake")),
        ("id", serde_json::json!("")),
        ("playerId", serde_json::json!("p2")),
        ("min", serde_json::json!(0)),
        ("max", serde_json::json!(2)),
        ("amount", serde_json::json!(2)),
        ("allowDecline", serde_json::json!(true)),
    ] {
        let mut v = base.clone();
        v["pending"]["choice"][key] = value;
        bad(&g, v);
    }
    for (key, value) in [
        ("actor", serde_json::json!(2)),
        ("cursor", serde_json::json!(0)),
        ("guard", serde_json::json!("Unchecked")),
        ("ability_key", serde_json::json!("fake")),
    ] {
        let mut v = base.clone();
        v["pending"]["resolution"]["Frame"]["frame"][key] = value;
        bad(&g, v);
    }
    let mut v = base.clone();
    v["pending"]["seat"] = serde_json::json!(2);
    bad(&g, v);
    let mut v = base.clone();
    v["pending"]["choice"]["options"][0]["id"] = serde_json::json!("fake");
    bad(&g, v);
    let mut v = base.clone();
    v["pending"]["resolution"]["Frame"]["choice"]["XQ18TimeCarrier"]["target_instance"] =
        serde_json::json!("fake");
    bad(&g, v);
    let mut v = base.clone();
    v["pending"]["resolution"]["Frame"]["frame"]["source"]["card"]["definition"] =
        serde_json::json!("JC125");
    bad(&g, v);
    let mut v = base;
    v["pending"]["resolution"]["Frame"]["frame"]["steps"][0]["op"] =
        serde_json::json!("XQ18AddOneTimeToTarget");
    bad(&g, v);
}

#[test]
fn xq18_unknown_definitions_fail_before_carrier_projection_without_panic() {
    let (g, t, _) = pending();
    let base = serde_json::to_value(&g).unwrap();
    let mut v = base.clone();
    v["attachments"][0]["card"]["definition"] = serde_json::json!("unknown");
    bad(&g, v);
    let mut v = base;
    let card = v["regions"][0]["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["id"] == t)
        .unwrap();
    card["definition"] = serde_json::json!("unknown");
    bad(&g, v);
}

#[test]
fn xq18_exact_definition_does_not_admit_alias_cost_event_or_nested_carrier_op() {
    let original = definition("XQ18");
    for variant in 0..5 {
        let mut d = original.clone();
        match variant {
            0 => d.abilities[0].key = "alias".into(),
            1 => d.abilities[0].event = Some(Event::Enter),
            2 => d.traits.public = true,
            3 => d.abilities[0].modes[1].ops.push(Op::XQ18ChooseTimeCarrier),
            _ => {
                d.abilities[0].modes[1].ops =
                    vec![Op::ForEachLivingPlayer(vec![Op::XQ18ChooseTimeCarrier])]
            }
        }
        assert!(crate::red_time::validate_definition("XQ18", &d).is_err());
    }
    assert!(crate::red_time::validate_definition("JC125", original).is_err());
}

#[test]
fn xq18_original_target_shield_cancels_whole_effect() {
    let mut g = initial(0);
    let t = board(&mut g, "JC125", 2, 0);
    g.board_mut(&t).unwrap().time_markers = 2;
    g.board_mut(&t).unwrap().shield = 1;
    declare(&mut g, 0, &t, false);
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert_eq!(g.board(&t).unwrap().1.time_markers, 2);
    assert_eq!(g.board(&t).unwrap().1.shield, 0);
}

#[test]
fn xq18_region_attachment_is_a_legal_direct_target() {
    let mut g = initial(0);
    let region = g.regions[0].card.id.clone();
    let a = attachment(&mut g, "JC090", 2, &region, 2);
    declare(&mut g, 0, &a, false);
    pass_top(&mut g);
    assert_eq!(carrier_ids(&g), BTreeSet::from([a.clone()]));
    choose(&mut g, vec![a.clone()]);
    assert_eq!(g.board(&a).unwrap().1.time_markers, 1);
}
