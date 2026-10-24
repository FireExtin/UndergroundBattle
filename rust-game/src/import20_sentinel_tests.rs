//! Explicit native layouts followed by real commands; not browser acceptance.
use crate::jc029_tests::pass_top;
use crate::{catalog, import20_sentinel as s, model::*, rules::*, unit_support::*};
fn sentinel(g: &mut Game, actor: usize) {
    let c = g.make_card("MSJC14", actor);
    g.players[actor].society_zone.card = Some(c);
}
fn finish(g: &mut Game, yes: bool) {
    for _ in 0..128 {
        if let Some(p) = &g.pending {
            let selected = if yes {
                p.choice
                    .options
                    .first()
                    .map(|o| vec![o.id.clone()])
                    .unwrap_or_default()
            } else {
                vec![]
            };
            choose(g, selected);
        } else if !g.stack.is_empty() {
            pass_top(g);
        } else if !g.effects.is_empty() {
            g.drive().unwrap();
        } else {
            return;
        }
    }
    panic!("bounded sentinel flow");
}
fn weapon(g: &mut Game, is_sentinel: bool) -> String {
    if is_sentinel {
        sentinel(g, 0);
    }
    g.begin_window(Window::Action(0));
    fund(g, 0, "JZ43", 4);
    let h = held(g, "CQ19", 0);
    let a = g
        .legal_actions(0)
        .into_iter()
        .find(|a| a.action.card_id.as_deref() == Some(&h) && a.action.kind == "play")
        .expect("weapon play")
        .action;
    act(g, 0, a);
    pass_top(g);
    finish(g, true);
    g.attachments
        .iter()
        .find(|a| a.card.definition == "CQ19")
        .unwrap()
        .card
        .id
        .clone()
}
fn activate(id: &str, key: &str, n: Option<u32>, target: Option<&str>) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(key.into()),
        amount: n,
        target_id: target.map(str::to_string),
        ..Action::new("activate")
    }
}
#[test]
fn import20_sentinel_cq19_original_unique_gray_and_exact_standard_costs() {
    let d = catalog::card("CQ19");
    assert_eq!(
        (&*d.name, d.cost, &*d.color, &*d.kind),
        ("达摩克里斯之剑", 4, "灰", "attachment")
    );
    assert!(d.unique && !d.supported);
    assert_eq!(d.loyalty, ["灰色"]);
    assert_eq!(d.subtitle.as_deref(), Some("天基动能武器系统"));
    let a = s::definition("CQ19").unwrap();
    assert_eq!(a.abilities[2].timing, Timing::Standard);
    assert!(matches!(
        a.abilities[3].costs.as_slice(),
        [Cost::ExhaustSource, Cost::RemoveChargeX]
    ));
    validate_definitions(definitions()).unwrap();
}
#[test]
fn import20_sentinel_cq19_real_paid_entry_sentinel_charge_and_non_sentinel_no_trigger() {
    for yes in [false, true] {
        let mut g = game();
        let id = weapon(&mut g, yes);
        assert_eq!(g.resources(0), 0);
        assert_eq!(g.in_play(&id).unwrap().1.charge_markers, u32::from(yes));
        restore(&mut g);
    }
}
#[test]
fn import20_sentinel_cq19_charge_action_exhausts_once() {
    let mut g = game();
    let id = weapon(&mut g, false);
    act(&mut g, 0, activate(&id, s::CHARGE, None, None));
    assert!(g.in_play(&id).unwrap().1.exhausted);
    pass_top(&mut g);
    assert_eq!(g.in_play(&id).unwrap().1.charge_markers, 1);
    reject(&mut g, 0, activate(&id, s::CHARGE, None, None));
}
#[test]
fn import20_sentinel_cq19_x_payment_and_damage_use_original_typed_receipt() {
    let mut g = game();
    let id = weapon(&mut g, true);
    g.board_mut(&id).unwrap().charge_markers = 3;
    let t = field(&mut g, "LC01", 2);
    act(&mut g, 0, activate(&id, s::FIRE, Some(2), Some(&t)));
    assert_eq!(g.in_play(&id).unwrap().1.charge_markers, 1);
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert!(matches!(&f.already_paid[1],PaidCost::CQ19ChargeRemoved(p)if p.amount==2));
    pass_top(&mut g);
    assert_eq!(g.board(&t).unwrap().1.damage, 2);
    assert!(g.in_play(&id).unwrap().1.exhausted);
}
#[test]
fn import20_sentinel_cq19_wrong_x_actor_target_and_payload_leave_costs_unchanged() {
    let mut g = game();
    let id = weapon(&mut g, true);
    let t = field(&mut g, "LC01", 2);
    let hidden = field(&mut g, "JC125", 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    for a in [
        activate(&id, s::FIRE, None, Some(&t)),
        activate(&id, s::FIRE, Some(2), Some(&t)),
        activate(&id, s::FIRE, Some(1), Some(&hidden)),
        activate(&id, s::CHARGE, Some(0), None),
    ] {
        reject(&mut g, 0, a);
    }
    reject(&mut g, 2, activate(&id, s::FIRE, Some(1), Some(&t)));
    assert_eq!(g.in_play(&id).unwrap().1.charge_markers, 1);
    assert!(!g.in_play(&id).unwrap().1.exhausted);
}
#[test]
fn import20_sentinel_cq19_zero_x_is_legal_but_still_exhausts() {
    let mut g = game();
    let id = weapon(&mut g, false);
    let t = field(&mut g, "LC01", 2);
    act(&mut g, 0, activate(&id, s::FIRE, Some(0), Some(&t)));
    pass_top(&mut g);
    assert_eq!(g.board(&t).unwrap().1.damage, 0);
    assert!(g.in_play(&id).unwrap().1.exhausted);
}
#[test]
fn import20_sentinel_cq19_shield_keeps_spent_charge_and_cancels_damage() {
    let mut g = game();
    let id = weapon(&mut g, true);
    let t = field(&mut g, "LC01", 2);
    g.board_mut(&t).unwrap().shield = 1;
    act(&mut g, 0, activate(&id, s::FIRE, Some(1), Some(&t)));
    pass_top(&mut g);
    assert_eq!(g.board(&t).unwrap().1.damage, 0);
    assert_eq!(g.board(&t).unwrap().1.shield, 0);
    assert_eq!(g.in_play(&id).unwrap().1.charge_markers, 0);
}
#[test]
fn import20_sentinel_cq19_source_leaves_response_original_damage_still_resolves() {
    let mut g = game();
    let id = weapon(&mut g, true);
    let t = field(&mut g, "LC01", 2);
    act(&mut g, 0, activate(&id, s::FIRE, Some(1), Some(&t)));
    g.remove_dead(&id, RemovalCause::Destroy);
    restore(&mut g);
    pass_top(&mut g);
    assert_eq!(g.board(&t).unwrap().1.damage, 1);
    assert!(g.players[0]
        .graveyard
        .iter()
        .filter(|c| c.definition == "CQ19")
        .all(|c| c.charge_markers == 0));
}
#[test]
fn import20_sentinel_cq19_charge_projection_all_seats_and_asset_seal_mask() {
    let mut g = game();
    let id = weapon(&mut g, true);
    let c = g.in_play(&id).unwrap().1;
    for v in 0..4 {
        assert_eq!(
            g.card_view(c, v, None, Some("attachment")).charge_markers,
            Some(1)
        );
        assert!(g
            .card_view(c, v, None, Some("asset"))
            .charge_markers
            .is_none());
        assert!(g.sealed_card_view(c, v).charge_markers.is_none());
    }
}
#[test]
fn import20_sentinel_cq19_legal_action_has_bounded_x_instead_of_enumerating_all_charges() {
    let mut g = game();
    let id = weapon(&mut g, true);
    g.board_mut(&id).unwrap().charge_markers = u32::MAX;
    let t = field(&mut g, "LC01", 2);
    let actions = g.legal_actions(0);
    let a = actions
        .iter()
        .find(|a| {
            a.action.card_id.as_deref() == Some(&id)
                && a.action.ability_id.as_deref() == Some(s::FIRE)
                && a.action.target_id.as_deref() == Some(&t)
        })
        .unwrap();
    assert_eq!(a.action.amount, Some(u32::MAX));
    assert_eq!(a.amount_range.as_ref().unwrap().max, u32::MAX);
    assert!(actions.len() < 1000);
}
#[test]
fn import20_sentinel_cq19_new_charge_category_uses_existing_copy_operation() {
    let mut g = game();
    let id = weapon(&mut g, true);
    let fee = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "XQ21", 1);
    let h = held(&mut g, "XQ21", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(h),
            target_id: Some(id.clone()),
            cost_selected: Some(vec![fee]),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    let p = g.pending.as_ref().unwrap();
    assert!(p.choice.options.iter().any(|o| o.id == "charge"));
    choose(&mut g, vec!["charge".into()]);
    assert_eq!(g.in_play(&id).unwrap().1.charge_markers, 2);
}
#[test]
fn import20_sentinel_cq19_normal_room_charge_payment_and_duplicate_once() {
    use crate::room::{RoomCommand, RoomEnvelope};
    let mut g = game();
    let id = weapon(&mut g, true);
    let t = field(&mut g, "LC01", 2);
    let r = crate::jc029_tests::envelope(&g);
    let cmd = RoomCommand {
        command_id: "cq19-fire-once".into(),
        expected_version: r.revision,
        action: activate(&id, s::FIRE, Some(1), Some(&t)).into(),
    };
    let out = r.transition(0, Some(cmd.clone()), 9000).unwrap();
    assert!(out.error_code.is_none(), "{:?}", out.error_message);
    let next = RoomEnvelope::from_persisted(&out.state).unwrap();
    assert_eq!(next.game.in_play(&id).unwrap().1.charge_markers, 0);
    let twice = next.transition(0, Some(cmd), 9001).unwrap();
    assert_eq!(twice.state, out.state);
    assert_eq!(twice.error_code.as_deref(), Some("version_conflict"));
}

fn ren(g: &mut Game, actor: usize, region: usize) -> String {
    g.begin_window(Window::Action(g.team(actor)));
    fund(g, actor, "JZ43", 4);
    let h = held(g, "CQ18", actor);
    act(
        g,
        actor,
        Action {
            card_id: Some(h),
            region: Some(region),
            ..Action::new("deploy")
        },
    );
    pass_top(g);
    g.regions[region]
        .cards
        .iter()
        .find(|c| c.definition == "CQ18" && c.controller == actor)
        .unwrap()
        .id
        .clone()
}
fn lock(g: &mut Game, id: &str) {
    g.board_mut(id).unwrap().lock_markers = 1;
}
fn enemy(g: &Game, actor: usize) -> usize {
    (0..g.players.len())
        .find(|s| g.team(*s) != g.team(actor))
        .unwrap()
}
#[test]
fn import20_sentinel_cq18_original_gray_unique_exact_body_and_closed_pool() {
    let d = catalog::card("CQ18");
    assert_eq!(
        (&*d.name, d.cost, &*d.color, d.defense),
        ("任秋雨", 4, "灰", Some(2))
    );
    assert_eq!(d.loyalty, ["灰色", "灰色", "灰色"]);
    assert!(d.unique && !d.supported);
    assert_eq!(
        d.permanent_icons,
        Icons {
            investigation: 0,
            combat: 2,
            influence: 1
        }
    );
    assert_eq!(d.magic_icon, MagicIcon::Mind);
    assert_eq!(d.subtypes, ["人类", "超能力者", "特工"]);
    assert_eq!(catalog::catalog().cards.len(), 267);
    assert_eq!(
        catalog::catalog()
            .cards
            .iter()
            .filter(|d| d.supported)
            .count(),
        213
    );
}
#[test]
fn import20_sentinel_cq18_actual_paid_entry_locks_enemy_current_region_and_hidden() {
    let mut g = game();
    let e = enemy(&g, 0);
    let a = field(&mut g, "LC01", e);
    let b = field(&mut g, "JC125", e);
    g.board_mut(&b).unwrap().face_down = true;
    let own = field(&mut g, "JC125", 0);
    let other = field(&mut g, "JC125", e);
    g.move_on_board_instance(&other, 1);
    let source = ren(&mut g, 0, 0);
    assert!(g.pending.is_some());
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    assert_eq!(g.board(&a).unwrap().1.lock_markers, 1);
    assert_eq!(g.board(&b).unwrap().1.lock_markers, 1);
    assert_eq!(g.board(&own).unwrap().1.lock_markers, 0);
    assert_eq!(g.board(&other).unwrap().1.lock_markers, 0);
    assert_eq!(g.board(&source).unwrap().1.lock_markers, 0);
    for v in 0..4 {
        let view = g.view(v);
        let c = view.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == b)
            .unwrap();
        assert_eq!(c.lock_markers, Some(1));
        if v != e {
            assert!(c.card_id.is_none());
        }
    }
}
#[test]
fn import20_sentinel_cq18_entry_decline_and_source_departure_response_keep_rules() {
    for accept in [false, true] {
        let mut g = game();
        let e = enemy(&g, 0);
        let t = field(&mut g, "LC01", e);
        let source = ren(&mut g, 0, 0);
        choose(
            &mut g,
            if accept {
                vec!["accept".into()]
            } else {
                vec![]
            },
        );
        if accept {
            g.return_hand(&source);
            restore(&mut g);
            pass_top(&mut g);
        }
        assert_eq!(g.board(&t).unwrap().1.lock_markers, u32::from(accept));
    }
}
#[test]
fn import20_sentinel_cq18_each_legitimate_departure_cause_draws_once() {
    for cause in ["destroy", "sacrifice", "return", "bottom", "hide"] {
        let mut g = game();
        let source = field(&mut g, "CQ18", 0);
        let e = enemy(&g, 0);
        let t = field(&mut g, "LC01", e);
        lock(&mut g, &t);
        deck_cards(&mut g, 0, &["JC125", "JC125"]);
        let before = g.players[0].hand.len();
        match cause {
            "destroy" => g.remove_dead(&t, RemovalCause::Destroy),
            "sacrifice" => g.remove_dead(&t, RemovalCause::Sacrifice),
            "return" => g.return_hand(&t),
            "bottom" => g.to_bottom(&t),
            _ => {
                assert!(g.hide_in_place(&t));
            }
        }
        g.drive().unwrap();
        restore(&mut g);
        finish(&mut g, true);
        assert_eq!(g.players[0].hand.len(), before + 1, "{cause}");
        assert!(g.board(&source).is_some());
    }
}
#[test]
fn import20_sentinel_cq18_unlocked_friendly_movement_and_detached_lock_do_not_draw() {
    let mut g = game();
    field(&mut g, "CQ18", 0);
    let e = enemy(&g, 0);
    let unlocked = field(&mut g, "LC01", e);
    g.return_hand(&unlocked);
    let friendly = field(&mut g, "LC01", 0);
    lock(&mut g, &friendly);
    g.return_hand(&friendly);
    let moving = field(&mut g, "LC01", e);
    lock(&mut g, &moving);
    g.move_on_board_instance(&moving, 1);
    g.drive().unwrap();
    assert!(g.pending.is_none() && g.stack.is_empty());
}
#[test]
fn import20_sentinel_cq18_inherited_descendant_lock_is_captured_before_host_cleanup() {
    let mut g = game();
    field(&mut g, "CQ18", 0);
    let e = enemy(&g, 0);
    let host = field(&mut g, "LC01", e);
    let mut att = g.make_card("BQ022", e);
    att.lock_markers = 1;
    g.attachments.push(Attachment {
        card: att,
        host_id: host.clone(),
    });
    deck_cards(&mut g, 0, &["JC125", "JC125"]);
    let before = g.players[0].hand.len();
    g.return_hand(&host);
    g.drive().unwrap();
    restore(&mut g);
    finish(&mut g, true);
    assert_eq!(g.players[0].hand.len(), before + 1);
}
#[test]
fn import20_sentinel_cq18_simultaneous_source_and_two_locked_enemies_capture_two_distinct_events() {
    let mut g = game();
    let source = field(&mut g, "CQ18", 0);
    let e = enemy(&g, 0);
    let a = field(&mut g, "LC01", e);
    let b = field(&mut g, "LC01", e);
    lock(&mut g, &a);
    lock(&mut g, &b);
    deck_cards(&mut g, 0, &["JC125", "JC125", "JC125"]);
    let before = g.players[0].hand.len();
    let deaths = [source, a, b]
        .into_iter()
        .map(|id| {
            let (r, c) = g.board(&id).unwrap();
            (
                id.clone(),
                RemovalCause::Destroy,
                g.source_snapshot(c, Some(r)),
            )
        })
        .collect();
    g.remove_death_batch(deaths);
    g.drive().unwrap();
    restore(&mut g);
    finish(&mut g, true);
    assert_eq!(g.players[0].hand.len(), before + 2);
    assert!(g.entry_order_groups.is_empty());
}
fn contest_cq18(sentinel_mode: bool) -> (Game, String) {
    let mut g = game();
    if sentinel_mode {
        sentinel(&mut g, 0);
    }
    let e = enemy(&g, 0);
    let t = field(&mut g, "LC01", e);
    for seat in 0..4 {
        deck_cards(&mut g, seat, &["JC125", "JC125"]);
        let id = if seat == 0 {
            "CQ18"
        } else if g.team(seat) != g.team(0) {
            "WM021"
        } else {
            "JC125"
        };
        held(&mut g, id, seat);
    }
    fund(&mut g, 0, "JC125", 2);
    let source = held(&mut g, "BQ032", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            ability_id: Some(crate::batch4_vampire_contest::HAND.into()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    for seat in 0..4 {
        let desired = if seat == 0 {
            "CQ18"
        } else if g.team(seat) != g.team(0) {
            "WM021"
        } else {
            "JC125"
        };
        let id = g.players[seat]
            .hand
            .iter()
            .find(|c| c.definition == desired)
            .unwrap()
            .id
            .clone();
        choose(&mut g, vec![id]);
    }
    while g
        .pending
        .as_ref()
        .is_some_and(|p| p.choice.kind == crate::mystic_contest::REWARD_KIND)
    {
        choose(&mut g, vec![]);
    }
    (g, t)
}
#[test]
fn import20_sentinel_cq18_real_losing_contest_has_plus_one_and_sentinel_optional_post_lock() {
    let (mut g, t) = contest_cq18(true);
    let o = g.mystic_history.last().unwrap();
    assert_eq!(
        o.shown
            .iter()
            .find(|s| s.card.definition == "CQ18")
            .unwrap()
            .value,
        5
    );
    assert_ne!(o.winner_team, Some(g.team(0)));
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    restore(&mut g);
    choose(&mut g, vec![t.clone()]);
    finish(&mut g, false);
    assert_eq!(g.board(&t).unwrap().1.lock_markers, 1);
}
#[test]
fn import20_sentinel_cq18_non_sentinel_still_gets_bonus_without_post_choice() {
    let (mut g, t) = contest_cq18(false);
    assert!(g.pending.is_none());
    assert_eq!(g.board(&t).unwrap().1.lock_markers, 0);
    assert_eq!(
        g.mystic_history
            .last()
            .unwrap()
            .shown
            .iter()
            .find(|s| s.card.definition == "CQ18")
            .unwrap()
            .value,
        5
    );
    finish(&mut g, false);
}
#[test]
fn import20_sentinel_cq18_post_lock_respects_shield_and_decline() {
    for shield in [false, true] {
        let (mut g, t) = contest_cq18(true);
        if shield {
            g.board_mut(&t).unwrap().shield = 1;
            choose(&mut g, vec![t.clone()]);
        } else {
            choose(&mut g, vec![]);
        }
        finish(&mut g, false);
        assert_eq!(g.board(&t).unwrap().1.lock_markers, 0);
        if shield {
            assert_eq!(g.board(&t).unwrap().1.shield, 0);
        }
    }
}
