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
fn import20_sentinel_cq19_entry_can_decline_and_charge_action_exhausts_once() {
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
    fund(&mut g, 0, "JZ43", 1);
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
