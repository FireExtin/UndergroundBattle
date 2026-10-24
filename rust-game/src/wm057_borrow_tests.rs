//! Personal-marker ruling: explicit native layouts followed by actual commands.
//! No browser, independent-review or corrupted-save fuzzing claim.
use crate::{
    catalog, import20_broker as b, jc029_tests::pass_top, model::*, rules::*, unit_support::*,
};
fn action(id: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(b::ACT.into()),
        ..Action::new("activate")
    }
}
fn layout(actor: usize) -> (Game, String) {
    let mut g = game();
    let id = field(&mut g, "WM057", 0);
    fund(&mut g, 0, "JC125", 2);
    for a in [1, 2, 3] {
        fund(&mut g, a, "JC125", 2);
    }
    g.begin_window(Window::Action(g.team(actor)));
    (g, id)
}
fn has(g: &Game, actor: usize, id: &str) -> bool {
    g.legal_actions(actor).iter().any(|l| {
        l.action.kind == "activate"
            && l.action.card_id.as_deref() == Some(id)
            && l.action.ability_id.as_deref() == Some(b::ACT)
    })
}
fn seed(g: &mut Game, winning_team: usize) -> Vec<String> {
    (0..4)
        .map(|s| {
            held(
                g,
                if g.team(s) == winning_team {
                    "LC01"
                } else {
                    "JC001"
                },
                s,
            )
        })
        .collect()
}
fn rewards(g: &mut Game, shown: &[String]) {
    for _ in 0..100 {
        if let Some(p) = g.pending.clone() {
            if p.choice.kind == crate::mystic_contest::REWARD_KIND {
                return;
            }
            if p.choice.kind == crate::mystic_contest::COMMIT_KIND {
                choose(g, vec![shown[p.seat].clone()]);
            } else {
                resolve_choice(g);
            }
        } else if !g.stack.is_empty() {
            pass_top(g);
        } else if !g.effects.is_empty() {
            g.drive().unwrap();
        } else {
            panic!("missing contest rewards");
        }
    }
    panic!("bounded borrowed contest")
}
fn finish(g: &mut Game) {
    for _ in 0..150 {
        if let Some(p) = g.pending.clone() {
            if p.choice.kind == crate::mystic_contest::REWARD_KIND {
                choose(g, vec![]);
            } else {
                resolve_choice(g);
            }
        } else if !g.stack.is_empty() {
            pass_top(g);
        } else if !g.effects.is_empty() {
            g.drive().unwrap();
        } else {
            return;
        }
    }
    panic!("bounded post contest")
}
fn borrow_to_rewards(actor: usize, winning_team: usize) -> (Game, String) {
    let (mut g, id) = layout(actor);
    g.players[actor].society_zone.secrecy = 1;
    let shown = seed(&mut g, winning_team);
    act(&mut g, actor, action(&id));
    pass_top(&mut g);
    rewards(&mut g, &shown);
    (g, id)
}
fn to_other_step(g: &mut Game) {
    let w = serde_json::to_value(&g.window).unwrap();
    for _ in 0..16 {
        if serde_json::to_value(&g.window).unwrap() != w {
            restore(g);
            return;
        }
        assert!(g.pending.is_none());
        pass(g);
    }
    panic!("step did not advance");
}
#[test]
fn wm057_borrow_ruling_two_enemies_one_marked_only_that_person_is_authorized() {
    let (mut g, id) = layout(2);
    g.players[2].society_zone.secrecy = 1;
    assert!(has(&g, 2, &id));
    assert!(!has(&g, 3, &id));
    reject(&mut g, 3, action(&id));
    assert!(!g.board(&id).unwrap().1.exhausted);
    assert_eq!(g.resources(3), 2);
    g.players[2].society_zone.secrecy = 0;
    g.players[3].society_zone.secrecy = 1;
    assert!(!has(&g, 2, &id));
    assert!(has(&g, 3, &id));
    reject(&mut g, 2, action(&id));
    act(&mut g, 3, action(&id));
    assert_eq!(g.resources(3), 0);
    assert_eq!(g.resources(2), 2);
    assert_eq!(g.board(&id).unwrap().1.controller, 0);
}
#[test]
fn wm057_borrow_both_marked_enemies_each_have_a_candidate_but_share_original_exhaustion() {
    let (mut g, id) = layout(2);
    g.players[2].society_zone.secrecy = 1;
    g.players[3].society_zone.secrecy = 2;
    assert!(has(&g, 2, &id) && has(&g, 3, &id));
    act(&mut g, 2, action(&id));
    reject(&mut g, 3, action(&id));
    assert_eq!(g.resources(3), 2);
    assert_eq!(g.resources(2), 0);
    assert!(g.board(&id).unwrap().1.exhausted);
}
#[test]
fn wm057_borrow_teammate_markers_and_exposure_do_not_grant_permission() {
    let (mut g, id) = layout(1);
    g.players[1].society_zone.secrecy = 2;
    assert!(!has(&g, 1, &id));
    reject(&mut g, 1, action(&id));
    g.begin_window(Window::Action(1));
    g.players[2].society_zone.exposure = 2;
    g.players[3].society_zone.secrecy = 2;
    assert!(!has(&g, 2, &id));
    reject(&mut g, 2, action(&id));
    assert_eq!(g.resources(2), 2);
}
#[test]
fn wm057_borrow_owner_still_needs_no_marker_and_does_not_get_a_borrow_receipt() {
    let (mut g, id) = layout(0);
    let shown = seed(&mut g, 0);
    assert!(has(&g, 0, &id));
    act(&mut g, 0, action(&id));
    assert!(!g
        .stack
        .last()
        .unwrap()
        .frame
        .as_ref()
        .unwrap()
        .already_paid
        .iter()
        .any(|p| matches!(p, PaidCost::WM057Borrowed(_))));
    pass_top(&mut g);
    rewards(&mut g, &shown);
    assert!(g.mystic_history[0].borrow_permission.is_none());
    finish(&mut g);
}
#[test]
fn wm057_borrow_only_actual_actor_pays_and_untouched_control_and_personal_marker_remain() {
    let (mut g, id) = layout(2);
    g.players[2].society_zone.secrecy = 3;
    let source = g.board(&id).unwrap().1.clone();
    act(&mut g, 2, action(&id));
    assert_eq!(g.resources(2), 0);
    assert_eq!(g.resources(0), 2);
    assert_eq!(g.players[2].society_zone.secrecy, 3);
    let c = g.board(&id).unwrap().1;
    assert_eq!(
        (c.id.as_str(), c.owner, c.controller),
        (id.as_str(), source.owner, 0)
    );
    assert!(c.exhausted);
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(f.actor, 2);
    assert_eq!(f.source.card.controller, 0);
    assert!(
        matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids),PaidCost::Exhausted(source),PaidCost::WM057Borrowed(p)]if ids.len()==2&&source==&id&&p.actor==2&&p.source_controller==0&&p.personal_secrecy==3&&p.source_instance==id)
    );
    restore(&mut g);
}
#[test]
fn wm057_borrow_own_two_assets_cannot_pay_for_an_unfunded_borrower() {
    let (mut g, id) = layout(2);
    g.players[2].society_zone.secrecy = 1;
    g.players[2].assets.pop();
    assert_eq!(g.resources(0), 2);
    assert!(!has(&g, 2, &id));
    reject(&mut g, 2, action(&id));
    assert_eq!(g.resources(2), 1);
    assert_eq!(g.resources(0), 2);
    assert!(!g.board(&id).unwrap().1.exhausted);
}
#[test]
fn wm057_borrow_hidden_exhausted_or_offboard_source_has_no_permission_or_spend() {
    for kind in 0..5 {
        let (mut g, id) = layout(2);
        g.players[2].society_zone.secrecy = 1;
        match kind {
            0 => g.board_mut(&id).unwrap().face_down = true,
            1 => g.board_mut(&id).unwrap().exhausted = true,
            2 => {
                let c = g.regions[0].cards.remove(0);
                g.players[0].assets.push(c);
            }
            3 => {
                let c = g.regions[0].cards.remove(0);
                g.players[0].hand.push(c);
            }
            _ => {
                let c = g.regions[0].cards.remove(0);
                g.players[0].graveyard.push(c);
            }
        }
        assert!(!has(&g, 2, &id));
        reject(&mut g, 2, action(&id));
        assert_eq!(g.resources(2), 2);
    }
}
#[test]
fn wm057_borrow_only_printed_action_no_manual_win_mode_target_amount_or_other_card() {
    let (mut g, id) = layout(2);
    g.players[2].society_zone.secrecy = 1;
    let base = action(&id);
    for kind in 0..4 {
        let mut a = base.clone();
        match kind {
            0 => a.ability_id = Some(b::WIN.into()),
            1 => a.target_id = Some("p0".into()),
            2 => a.amount = Some(2),
            _ => a.option = Some("inspect".into()),
        }
        reject(&mut g, 2, a);
    }
    let other = field(&mut g, "WM044", 0);
    reject(
        &mut g,
        2,
        Action {
            card_id: Some(other),
            ability_id: Some(b::ACT.into()),
            ..Action::new("activate")
        },
    );
    assert_eq!(g.resources(2), 2);
    assert!(!g.board(&id).unwrap().1.exhausted);
}
#[test]
fn wm057_borrow_default_printed_ability_command_uses_the_same_authenticated_path() {
    let (mut g, id) = layout(2);
    g.players[2].society_zone.secrecy = 1;
    act(
        &mut g,
        2,
        Action {
            card_id: Some(id.clone()),
            ..Action::new("activate")
        },
    );
    assert!(matches!(
        g.stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .already_paid
            .last(),
        Some(PaidCost::WM057Borrowed(_))
    ));
    assert!(g.board(&id).unwrap().1.exhausted);
}
#[test]
fn wm057_borrow_does_not_allow_standard_action_during_a_response_or_private_selection() {
    let (mut g, id) = layout(2);
    g.players[2].society_zone.secrecy = 1;
    g.players[2].assets.clear();
    fund(&mut g, 2, "JC084", 2);
    let target = field(&mut g, "JC001", 0);
    let spell = held(&mut g, "BQ090", 2);
    act(
        &mut g,
        2,
        Action {
            card_id: Some(spell),
            target_id: Some(target),
            ..Action::new("play")
        },
    );
    assert!(!g.board(&id).unwrap().1.exhausted);
    reject(&mut g, 2, action(&id));
    assert_eq!(g.resources(2), 0); // the spell paid its own2
}
#[test]
fn wm057_borrow_receipt_survives_marker_loss_source_departure_and_no_rebinding() {
    let (mut g, id) = layout(2);
    g.players[2].society_zone.secrecy = 1;
    let shown = seed(&mut g, 1);
    act(&mut g, 2, action(&id));
    g.players[2].society_zone.secrecy = 0;
    g.remove_dead(&id, RemovalCause::Destroy);
    let replacement = field(&mut g, "WM057", 0);
    restore(&mut g);
    pass_top(&mut g);
    rewards(&mut g, &shown);
    finish(&mut g);
    let o = &g.mystic_history[0];
    assert_eq!(o.actor, 2);
    assert_eq!(o.source.card.id, id);
    assert_eq!(o.borrow_permission.as_ref().unwrap().personal_secrecy, 1);
    assert_eq!(g.resources(2), 0);
    assert!(!g.board(&replacement).unwrap().1.exhausted);
    restore(&mut g);
}
#[test]
fn wm057_borrow_actual_four_private_commits_follow_participants_not_source_controller() {
    let (mut g, id) = layout(3);
    g.players[3].society_zone.secrecy = 1;
    let shown = seed(&mut g, 1);
    act(&mut g, 3, action(&id));
    pass_top(&mut g);
    for expected in 0..4 {
        let p = g.pending.as_ref().unwrap();
        assert_eq!(p.seat, expected);
        for viewer in 0..4 {
            let v = g.view(viewer);
            assert_eq!(v.pending_choice.is_some(), viewer == expected);
            if viewer != expected {
                assert!(!v.hand.iter().any(|c| c.instance_id == shown[expected]));
            }
        }
        choose(&mut g, vec![shown[expected].clone()]);
    }
    assert_eq!(g.mystic_history[0].actor, 3);
    finish(&mut g);
    assert_eq!(g.mystic_history[0].source.card.controller, 0);
    assert_eq!(g.mystic_history[0].winner_team, Some(1));
}
#[test]
fn wm057_borrow_winning_borrower_does_not_acquire_original_owners_win_inspection() {
    let (mut g, id) = borrow_to_rewards(2, 1);
    finish(&mut g);
    assert_eq!(g.mystic_history[0].winner_team, Some(1));
    assert!(g.pending.is_none() && g.effects.is_empty());
    assert!(g.board(&id).unwrap().1.exhausted);
    assert_eq!(g.regions.iter().map(|r| r.influence[0]).sum::<u32>(), 0);
}
#[test]
fn wm057_borrow_original_owner_win_keeps_repression_and_private_inspection_with_owner() {
    let (mut g, id) = layout(2);
    g.players[2].society_zone.secrecy = 1;
    let hidden = field(&mut g, "JC056", 3);
    g.board_mut(&hidden).unwrap().face_down = true;
    g.regions[0].influence[1] = 1;
    let shown = seed(&mut g, 0);
    act(&mut g, 2, action(&id));
    pass_top(&mut g);
    rewards(&mut g, &shown);
    while g
        .pending
        .as_ref()
        .is_some_and(|p| p.choice.kind == crate::mystic_contest::REWARD_KIND)
    {
        choose(&mut g, vec![]);
    }
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    choose(&mut g, vec!["inspect".into()]);
    choose(&mut g, vec![hidden.clone()]);
    pass_top(&mut g);
    assert!(matches!(
        g.pending.as_ref().unwrap().resolution,
        ChoiceResolution::Frame {
            choice: FrameChoice::WM057Opponent,
            ..
        }
    ));
    choose(&mut g, vec!["p2".into()]);
    choose(&mut g, vec!["region:0".into()]);
    choose(&mut g, vec!["inspect".into()]);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    for viewer in 0..4 {
        let v = g.view(viewer);
        if viewer == 0 {
            let p = v.pending_choice.unwrap();
            assert_eq!(p.preview_cards.len(), 1);
            assert_eq!(p.preview_cards[0].instance_id, hidden);
            assert_eq!(p.preview_cards[0].card_id.as_deref(), Some("JC056"));
        } else {
            assert!(v.pending_choice.is_none());
            assert!(v.private_inspections.is_empty());
        }
    }
    assert_eq!(g.regions[0].influence[1], 0);
    assert!(g.board(&hidden).unwrap().1.face_down);
    choose(&mut g, vec![]);
    finish(&mut g);
    restore(&mut g);
}
fn credit(g: &mut Game, actor: usize) {
    let host = g.regions[0].card.id.clone();
    let a = g.make_card("BQ092", actor);
    let id = a.id.clone();
    g.attachments.push(Attachment {
        card: a,
        host_id: host.clone(),
    });
    let b = g.make_card("BQ091", actor);
    let other = b.id.clone();
    g.attachments.push(Attachment {
        card: b,
        host_id: host,
    });
    act(
        g,
        actor,
        Action {
            card_id: Some(id),
            ability_id: Some(crate::import20_industry::FUND.into()),
            cost_selected: Some(vec![other]),
            ..Action::new("activate")
        },
    );
    pass_top(g);
    assert_eq!(g.step_resource_count(actor), 1);
}
#[test]
fn wm057_borrow_and_owner_both_use_existing_credit_plus_asset_payment_and_normal_next_step() {
    for actor in [0, 2] {
        let (mut g, id) = layout(actor);
        g.players[actor].society_zone.secrecy = 1;
        g.players[actor].assets.pop();
        credit(&mut g, actor);
        let shown = seed(&mut g, 1);
        act(&mut g, actor, action(&id));
        let f = g.stack.last().unwrap().frame.as_ref().unwrap();
        assert_eq!(f.money_units(0), 2);
        assert_eq!(f.resource_payments[0].paid_index, 0);
        assert!(matches!(&f.already_paid[0],PaidCost::Assets(ids)if ids.len()==1));
        assert_eq!(g.resources(actor), 0);
        pass_top(&mut g);
        rewards(&mut g, &shown);
        finish(&mut g);
        to_other_step(&mut g);
        assert_eq!(g.mystic_history.len(), 1);
        assert_eq!(g.mystic_history[0].borrow_permission.is_some(), actor == 2);
    }
}
#[test]
fn wm057_borrow_repeated_legal_contests_have_separate_personal_receipts_and_once_payments() {
    let (mut g, first) = layout(2);
    let second = field(&mut g, "WM057", 1);
    g.players[2].society_zone.secrecy = 1;
    fund(&mut g, 2, "JC125", 2);
    for id in [&first, &second] {
        let shown = seed(&mut g, 1);
        act(&mut g, 2, action(id));
        pass_top(&mut g);
        rewards(&mut g, &shown);
        finish(&mut g);
    }
    assert_eq!(g.resources(2), 0);
    assert_eq!(g.mystic_history.len(), 2);
    assert_ne!(g.mystic_history[0].frame_id, g.mystic_history[1].frame_id);
    assert_eq!(
        g.mystic_history[0]
            .borrow_permission
            .as_ref()
            .unwrap()
            .source_instance,
        first
    );
    assert_eq!(
        g.mystic_history[1]
            .borrow_permission
            .as_ref()
            .unwrap()
            .source_instance,
        second
    );
    restore(&mut g);
}
#[test]
fn wm057_borrow_room_normal_duplicate_and_other_eligible_enemy_stale_command_do_not_pay_twice() {
    use crate::room::{RoomCommand, RoomEnvelope};
    let (mut g, id) = layout(2);
    g.players[2].society_zone.secrecy = 1;
    g.players[3].society_zone.secrecy = 1;
    let r = crate::jc029_tests::envelope(&g);
    let c = RoomCommand {
        command_id: "wm057-borrow-once".into(),
        expected_version: r.revision,
        action: action(&id).into(),
    };
    let out = r.transition(2, Some(c.clone()), 9000).unwrap();
    assert!(out.error_code.is_none(), "{:?}", out.error_message);
    let next = RoomEnvelope::from_persisted(&out.state).unwrap();
    assert_eq!(next.game.resources(2), 0);
    assert_eq!(next.game.resources(3), 2);
    let dup = next.transition(2, Some(c.clone()), 9001).unwrap();
    assert_eq!(dup.state, out.state);
    let mut enemy = c;
    enemy.command_id = "other-enemy-stale".into();
    let stale = next.transition(3, Some(enemy), 9002).unwrap();
    assert_eq!(stale.state, out.state);
    assert_eq!(stale.error_code.as_deref(), Some("version_conflict"));
}
#[test]
fn wm057_borrow_keeps_printed_program_and_closed_public_gate_no_cost_assumption_for_xq19() {
    assert_eq!(catalog::card("WM057").color, "黑");
    assert!(!catalog::card("WM057").supported);
    assert_eq!(
        b::definition("WM057").unwrap().abilities[0].timing,
        Timing::Standard
    );
    assert!(matches!(
        b::definition("WM057").unwrap().abilities[0]
            .costs
            .as_slice(),
        [Cost::Assets(2), Cost::ExhaustSource]
    ));
    assert!(!catalog::catalog().cards.iter().any(|c| c.id == "XQ19"));
    assert_current_serial_catalog();
}
