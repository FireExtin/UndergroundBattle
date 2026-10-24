//! Commands use real apply/payment/response paths on explicit legal layouts.
//! Sealing fixtures call the existing validated sealing primitive: this slice
//! does not invent another card's attachment-target sealing ability.
use crate::jc029_tests::pass_top;
use crate::{catalog, import20_industry as i, model::*, rules::*, unit_support::*};
fn fixture(g: &mut Game, id: &str, actor: usize) -> String {
    let c = g.make_card(id, actor);
    let uid = c.id.clone();
    let host = g.regions[0].card.id.clone();
    g.attachments.push(Attachment {
        card: c,
        host_id: host,
    });
    uid
}
fn activation(id: &str, other: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(i::FUND.into()),
        cost_selected: Some(vec![other.into()]),
        ..Action::new("activate")
    }
}
fn credit(g: &mut Game, actor: usize) -> (String, String) {
    let source = fixture(g, "BQ092", actor);
    let other = fixture(g, "BQ091", actor);
    priority(g, actor);
    act(g, actor, activation(&source, &other));
    pass_top(g);
    (source, other)
}
fn priority(g: &mut Game, actor: usize) {
    for _ in 0..8 {
        if g.can_fast(actor) {
            return;
        }
        pass(g);
    }
    panic!("missing priority");
}
fn drain(g: &mut Game, accept: bool) {
    for _ in 0..128 {
        if let Some(p) = &g.pending {
            let ids = if accept {
                p.choice
                    .options
                    .first()
                    .map(|o| vec![o.id.clone()])
                    .unwrap_or_default()
            } else {
                vec![]
            };
            choose(g, ids);
        } else if !g.stack.is_empty() {
            pass_top(g);
        } else if !g.effects.is_empty() {
            g.drive().unwrap();
        } else {
            return;
        }
    }
    panic!("bounded industry continuation");
}
fn play_attachment(g: &mut Game, id: &str, actor: usize) -> String {
    g.begin_window(Window::Action(g.team(actor)));
    fund(g, actor, "JC084", catalog::card(id).cost as usize);
    let h = held(g, id, actor);
    act(
        g,
        actor,
        Action {
            card_id: Some(h),
            target_id: Some("region:0".into()),
            ..Action::new("play")
        },
    );
    pass_top(g);
    g.attachments
        .iter()
        .rev()
        .find(|a| a.card.definition == id && a.card.controller == actor)
        .unwrap()
        .card
        .id
        .clone()
}
fn destroy_attachment(g: &mut Game, id: &str) {
    fund(g, 2, "JC005", 2);
    let h = held(g, "JC005", 2);
    priority(g, 2);
    act(
        g,
        2,
        Action {
            card_id: Some(h),
            target_id: Some(id.into()),
            ..Action::new("play")
        },
    );
    pass_top(g);
    assert!(g.in_play(id).is_none());
}
fn start_mill(g: &mut Game) -> String {
    let source = play_attachment(g, "BQ093", 0);
    assert!(g.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Declare{declaration,..}if declaration.ability.key==i::MILL)));
    source
}
#[test]
fn import20_industry_original_fields_exact_both_exhaustions_and_closed_gate() {
    for (id, cost) in [("BQ092", 1), ("BQ093", 2)] {
        let d = catalog::card(id);
        assert_eq!(
            (
                d.cost,
                d.color.as_str(),
                d.kind.as_str(),
                d.subtitle.as_deref()
            ),
            (cost, "黑", "attachment", Some("圣甲虫产业"))
        );
        assert_eq!(d.loyalty, ["黑色"]);
        assert!(!d.supported && !d.unique);
        assert!(catalog::runtime_implemented(d));
    }
    assert!(catalog::card("BQ092").text.contains("横置本牌"));
    let d = definition("BQ092");
    let a = d.abilities.iter().find(|a| a.key == i::FUND).unwrap();
    assert!(matches!(
        a.costs.as_slice(),
        [Cost::ExhaustSource, Cost::IndustryExhaustOther]
    ));
    assert_eq!(a.timing, Timing::Fast);
    validate_definitions(definitions()).unwrap();
}
#[test]
fn import20_industry_real_paid_region_attachment_and_actual_two_exhausted_costs() {
    let mut g = game();
    let source = play_attachment(&mut g, "BQ092", 0);
    let other = play_attachment(&mut g, "BQ091", 0);
    let assets = g.players[0].assets.clone();
    let before = g.resources(0);
    let action = activation(&source, &other);
    assert!(g.legal_actions(0).iter().any(|a| a.action == action));
    act(&mut g, 0, action);
    assert!(g.board(&source).unwrap().1.exhausted && g.board(&other).unwrap().1.exhausted);
    assert!(g.step_resources.is_empty());
    pass_top(&mut g);
    assert_eq!(g.resources(0), before + 1);
    assert_eq!(g.step_resource_count(0), 1);
    for seat in 0..4 {
        assert_eq!(g.view(seat).players[0].step_resources, 1);
        assert_eq!(g.view(seat).players[1].step_resources, 0);
    }
    assert_eq!(
        serde_json::to_value(&assets).unwrap(),
        serde_json::to_value(&g.players[0].assets).unwrap()
    );
    assert_eq!(g.players[0].assets.len(), 2);
    restore(&mut g);
}
#[test]
fn import20_industry_invalid_self_ally_enemy_wrong_subtitle_asset_hidden_and_count_rollback() {
    let mut g = game();
    let source = fixture(&mut g, "BQ092", 0);
    let other = fixture(&mut g, "BQ091", 0);
    let ally = fixture(&mut g, "BQ091", 1);
    let enemy = fixture(&mut g, "BQ094", 2);
    let host = field(&mut g, "JC125", 0);
    let wrong = g.make_card("WM038", 0);
    let wrong_id = wrong.id.clone();
    g.attachments.push(Attachment {
        card: wrong,
        host_id: host,
    });
    let hidden = field(&mut g, "JC125", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    fund(&mut g, 0, "BQ091", 1);
    let asset = g.players[0].assets[0].id.clone();
    for id in [&source, &ally, &enemy, &wrong_id, &hidden, &asset] {
        reject(&mut g, 0, activation(&source, id));
        assert!(!g.board(&source).unwrap().1.exhausted);
    }
    reject(
        &mut g,
        0,
        Action {
            cost_selected: Some(vec![other.clone(), other.clone()]),
            ..activation(&source, &other)
        },
    );
    reject(
        &mut g,
        0,
        Action {
            target_id: Some(other.clone()),
            ..activation(&source, &other)
        },
    );
    g.board_mut(&other).unwrap().exhausted = true;
    reject(&mut g, 0, activation(&source, &other));
    assert!(g.step_resources.is_empty());
}
#[test]
fn import20_industry_other_step_credit_is_personal_and_does_not_supply_loyalty() {
    let mut g = game();
    credit(&mut g, 0);
    assert_eq!(g.resources(0), 1);
    assert_eq!(g.resources(1), 0);
    assert_eq!(g.resources(2), 0);
    assert!(g.players[0].assets.is_empty());
    assert!(!g.loyalty(0, "BQ092"));
    let h = held(&mut g, "BQ092", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(h),
            target_id: Some("region:0".into()),
            ..Action::new("play")
        },
    );
    assert_eq!(g.step_resource_count(0), 1);
}
#[test]
fn import20_industry_credit_and_asset_jointly_pay_real_fast_spell_without_fabricating_asset_ids() {
    let mut g = game();
    credit(&mut g, 0);
    fund(&mut g, 0, "JC084", 1);
    let target = field(&mut g, "JC125", 2);
    let h = held(&mut g, "BQ090", 0);
    let a = Action {
        card_id: Some(h),
        target_id: Some(target.clone()),
        ..Action::new("play")
    };
    assert!(g.legal_actions(0).iter().any(|x| x.action == a));
    act(&mut g, 0, a);
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert!(matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if ids.len()==1));
    assert_eq!(f.money_units(0), 2);
    assert_eq!(f.resource_payments[0].credits.len(), 1);
    assert_eq!(g.resources(0), 0);
    assert_eq!(g.players[0].assets.len(), 1);
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
}
#[test]
fn import20_industry_credit_pays_tattoo_fast_cost_with_no_physical_assets() {
    let mut g = game();
    credit(&mut g, 0);
    let host = field(&mut g, "JC125", 0);
    let source = g.make_card("WM064", 0);
    let uid = source.id.clone();
    g.attachments.push(Attachment {
        card: source,
        host_id: host,
    });
    let cost = field(&mut g, "JC084", 0);
    let target = field(&mut g, "JC084", 0);
    g.board_mut(&target).unwrap().face_down = true;
    act(
        &mut g,
        0,
        Action {
            card_id: Some(uid),
            ability_id: Some(crate::import20_tattoo::FAST.into()),
            cost_selected: Some(vec![cost]),
            target_id: Some(target),
            ..Action::new("activate")
        },
    );
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(f.money_units(0), 1);
    assert!(matches!(&f.already_paid[0],PaidCost::Assets(ids)if ids.is_empty()));
    pass_top(&mut g);
    assert!(g.players[0].assets.is_empty());
    assert_eq!(g.resources(0), 0);
}
#[test]
fn import20_industry_two_credits_pay_normal_plan_advancement_and_unused_credit_remains() {
    let mut g = game();
    credit(&mut g, 0);
    credit(&mut g, 0);
    let plan = g.make_card("WM045", 0);
    let uid = plan.id.clone();
    g.attachments.push(Attachment {
        card: plan,
        host_id: "society:p0".into(),
    });
    act(
        &mut g,
        0,
        Action {
            card_id: Some(uid.clone()),
            ability_id: Some(crate::batch4_plans::ADVANCE.into()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    assert_eq!(
        g.attachments
            .iter()
            .find(|a| a.card.id == uid)
            .unwrap()
            .card
            .advancement_markers,
        1
    );
    assert_eq!(g.step_resource_count(0), 1);
    assert!(g.players[0].assets.is_empty());
}
#[test]
fn import20_industry_two_credits_pay_hand_discard_and_enter_real_contest() {
    let mut g = game();
    credit(&mut g, 0);
    credit(&mut g, 0);
    let h = held(&mut g, "BQ092", 0);
    let a = Action {
        card_id: Some(h.clone()),
        ability_id: Some(i::HAND.into()),
        ..Action::new("activate")
    };
    assert!(g.legal_actions(0).iter().any(|x| x.action == a));
    act(&mut g, 0, a);
    assert_eq!(g.step_resource_count(0), 0);
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(f.money_units(0), 2);
    assert!(matches!(&f.already_paid[0],PaidCost::Assets(ids)if ids.is_empty()));
    assert!(!g.players[0].hand.iter().any(|c| c.id == h));
    pass_top(&mut g);
    assert!(g.mystic_contest.is_some());
    assert!(g.pending.is_some());
}
#[test]
fn import20_industry_credit_payment_failure_does_not_consume_credit_or_tap_asset() {
    let mut g = game();
    credit(&mut g, 0);
    fund(&mut g, 0, "JC125", 1);
    let t = field(&mut g, "JC125", 2);
    let h = held(&mut g, "BQ090", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(h),
            target_id: Some(t),
            ..Action::new("play")
        },
    );
    assert_eq!(g.step_resource_count(0), 1);
    assert!(!g.players[0].assets[0].exhausted);
}
#[test]
fn import20_industry_source_destroyed_in_response_still_produces_paid_credit_once() {
    let mut g = game();
    let source = fixture(&mut g, "BQ092", 0);
    let other = fixture(&mut g, "BQ091", 0);
    act(&mut g, 0, activation(&source, &other));
    destroy_attachment(&mut g, &source);
    pass_top(&mut g);
    assert_eq!(g.step_resource_count(0), 1);
    assert!(g.board(&other).unwrap().1.exhausted);
    restore(&mut g);
}
#[test]
fn import20_industry_other_cost_destroyed_in_response_does_not_refund_or_cancel_credit() {
    let mut g = game();
    let source = fixture(&mut g, "BQ092", 0);
    let other = fixture(&mut g, "BQ091", 0);
    act(&mut g, 0, activation(&source, &other));
    destroy_attachment(&mut g, &other);
    pass_top(&mut g);
    assert_eq!(g.step_resource_count(0), 1);
    assert!(g.board(&source).unwrap().1.exhausted);
}
#[test]
fn import20_industry_step_end_expires_credit_by_actual_action_step_passes() {
    let mut g = game();
    credit(&mut g, 0);
    for _ in 0..16 {
        if g.window == Some(Window::Action(1)) {
            break;
        }
        pass(&mut g);
    }
    assert_eq!(g.window, Some(Window::Action(1)));
    assert!(g.step_resources.is_empty());
    assert_eq!(g.resources(0), 0);
    restore(&mut g);
}
#[test]
fn import20_industry_before_and_after_same_confrontation_preserve_then_next_type_expires() {
    let mut g = game();
    g.begin_window(Window::Before(0, 0));
    credit(&mut g, 0);
    for _ in 0..32 {
        if g.window == Some(Window::After(0, 0)) {
            break;
        }
        pass(&mut g);
    }
    assert_eq!(g.window, Some(Window::After(0, 0)));
    assert_eq!(g.step_resource_count(0), 1);
    for _ in 0..32 {
        if g.window == Some(Window::Before(0, 1)) {
            break;
        }
        pass(&mut g);
    }
    assert_eq!(g.window, Some(Window::Before(0, 1)));
    assert!(g.step_resources.is_empty());
}
#[test]
fn import20_industry_actual_entry_observes_itself_optional_mill_is_not_draw() {
    let mut g = game();
    g.players[0].deck.clear();
    let card = g.make_card("JC125", 0);
    g.players[0].deck.push(card);
    start_mill(&mut g);
    assert_eq!(g.players[0].deck.len(), 1);
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    assert!(g.players[0].deck.is_empty());
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JC125")
            .count(),
        1
    );
    assert!(!g.players[0].eliminated);
    assert!(g.players[0].hand.is_empty());
}
#[test]
fn import20_industry_empty_deck_mill_does_not_eliminate_and_decline_does_not_mill() {
    let mut g = game();
    g.players[0].deck.clear();
    start_mill(&mut g);
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    assert!(!g.players[0].eliminated);
    let mut g = game();
    let before = g.players[0].deck.len();
    start_mill(&mut g);
    choose(&mut g, vec![]);
    assert_eq!(g.players[0].deck.len(), before);
}
#[test]
fn import20_industry_only_same_controller_subtitle_actual_entry_qualifies_not_reattachment() {
    let mut g = game();
    let observer = fixture(&mut g, "BQ093", 0);
    let other = play_attachment(&mut g, "BQ091", 1);
    assert!(g.pending.is_none());
    g.begin_window(Window::Action(0));
    let h = held(&mut g, "JC048", 0);
    fund(&mut g, 0, "JC125", 2);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(h),
            target_id: Some("region:0".into()),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    assert!(g.pending.is_none());
    let new = g.regions[1].card.id.clone();
    g.reattach_source(&other, &new);
    g.drive().unwrap();
    assert!(g.pending.is_none());
    assert!(g.board(&observer).is_some());
}
#[test]
fn import20_industry_multiple_sources_get_one_event_each_and_can_choose_order() {
    let mut g = game();
    let first = fixture(&mut g, "BQ093", 0);
    let second = play_attachment(&mut g, "BQ093", 0);
    let before = g.players[0].deck.len();
    assert!(g
        .pending
        .as_ref()
        .is_some_and(|p| p.choice.options.len() == 2));
    drain(&mut g, true);
    assert_eq!(g.players[0].deck.len(), before - 2);
    assert!(g.board(&first).is_some() && g.board(&second).is_some());
}
#[test]
fn import20_industry_character_sealed_by_other_holder_rewards_original_host_controller() {
    let mut g = game();
    fixture(&mut g, "BQ093", 0);
    let host = fixture(&mut g, "BQ094", 0);
    let h = held(&mut g, "JC125", 2);
    let before = g.current_icons(g.board(&host).unwrap().1, 0).investigation;
    g.seal_hand_card(2, &h, &host).unwrap();
    g.drive().unwrap();
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    assert_eq!(
        g.current_icons(g.board(&host).unwrap().1, 0).investigation,
        before + 1
    );
    g.first_team = 1;
    assert_eq!(
        g.current_icons(g.board(&host).unwrap().1, 0).investigation,
        1
    );
    assert!(g.sealed_cards.iter().any(|s| s.host_id == host));
    restore(&mut g);
}
#[test]
fn import20_industry_sealed_noncharacter_other_subtitle_or_ally_industry_does_not_trigger() {
    for case in 0..3 {
        let mut g = game();
        fixture(&mut g, "BQ093", 0);
        let host = if case == 2 {
            fixture(&mut g, "BQ091", 1)
        } else if case == 1 {
            field(&mut g, "JC125", 0)
        } else {
            fixture(&mut g, "BQ091", 0)
        };
        let h = held(&mut g, if case == 0 { "BQ090" } else { "JC125" }, 0);
        g.seal_hand_card(0, &h, &host).unwrap();
        g.drive().unwrap();
        assert!(g.pending.is_none());
        assert!(g.industry_bonuses.is_empty());
        restore(&mut g);
    }
}
#[test]
fn import20_industry_sealed_reward_survives_source_response_departure() {
    let mut g = game();
    let source = fixture(&mut g, "BQ093", 0);
    let host = fixture(&mut g, "BQ091", 0);
    let h = held(&mut g, "JC125", 0);
    g.seal_hand_card(0, &h, &host).unwrap();
    g.drive().unwrap();
    choose(&mut g, vec!["accept".into()]);
    destroy_attachment(&mut g, &source);
    pass_top(&mut g);
    assert_eq!(
        g.current_icons(g.board(&host).unwrap().1, 0).investigation,
        1
    );
    restore(&mut g);
}
#[test]
fn import20_industry_sealed_host_departure_does_not_rebind_replacement_or_keep_seals() {
    let mut g = game();
    fixture(&mut g, "BQ093", 0);
    let host = fixture(&mut g, "BQ091", 0);
    let h = held(&mut g, "JC125", 0);
    g.seal_hand_card(0, &h, &host).unwrap();
    g.drive().unwrap();
    choose(&mut g, vec!["accept".into()]);
    destroy_attachment(&mut g, &host);
    let replacement = fixture(&mut g, "BQ091", 0);
    pass_top(&mut g);
    assert_eq!(
        g.current_icons(g.board(&replacement).unwrap().1, 0)
            .investigation,
        0
    );
    assert!(g.sealed_cards.is_empty());
    assert!(g.players[0].hand.iter().any(|c| c.definition == "JC125"));
    restore(&mut g);
}
#[test]
fn import20_industry_two_seals_stack_independent_of_initiative_and_expire_at_turn_end() {
    let mut g = game();
    fixture(&mut g, "BQ093", 0);
    let host = fixture(&mut g, "BQ091", 0);
    for _ in 0..2 {
        let h = held(&mut g, "JC125", 0);
        g.seal_hand_card(0, &h, &host).unwrap();
        g.drive().unwrap();
        drain(&mut g, true);
    }
    assert_eq!(
        g.current_icons(g.board(&host).unwrap().1, 0).investigation,
        2
    );
    g.first_team = 1;
    assert_eq!(
        g.current_icons(g.board(&host).unwrap().1, 0).investigation,
        2
    );
    g.begin_window(Window::End);
    let turn = g.turn;
    for _ in 0..96 {
        if g.turn != turn {
            break;
        }
        if g.pending.is_some() {
            let p = g.pending.as_ref().unwrap();
            let n = p.choice.min.unwrap_or(0);
            let selected = p
                .choice
                .options
                .iter()
                .take(n)
                .map(|o| o.id.clone())
                .collect();
            choose(&mut g, selected);
        } else {
            pass(&mut g);
        }
    }
    assert_eq!(g.turn, turn + 1);
    assert!(g.industry_bonuses.is_empty());
    assert_eq!(
        g.current_icons(g.board(&host).unwrap().1, 0).investigation,
        0
    );
}
#[test]
fn import20_industry_room_activation_journal_and_duplicate_cas_spend_once() {
    use crate::room::{RoomCommand, RoomEnvelope};
    let mut g = game();
    let source = fixture(&mut g, "BQ092", 0);
    let other = fixture(&mut g, "BQ091", 0);
    let r = crate::jc029_tests::envelope(&g);
    let command = RoomCommand {
        command_id: "industry-normal-cost-once".into(),
        expected_version: r.revision,
        action: activation(&source, &other).into(),
    };
    let out = r.transition(0, Some(command.clone()), 5000).unwrap();
    assert!(out.error_code.is_none(), "{:?}", out.error_message);
    let next = RoomEnvelope::from_persisted(&out.state).unwrap();
    assert!(next.game.board(&source).unwrap().1.exhausted);
    assert_eq!(
        serde_json::to_value(r.replay_events(&out.journal).unwrap()).unwrap(),
        serde_json::to_value(&next).unwrap()
    );
    let repeat = next.transition(0, Some(command), 5001).unwrap();
    assert_eq!(repeat.error_code.as_deref(), Some("version_conflict"));
    assert_eq!(repeat.state, out.state);
}

#[test]
fn import20_industry_credits_pay_bq029_printed_and_extra_costs_with_separate_receipts() {
    let mut g = game();
    fund(&mut g, 0, "JC029", 2);
    for c in &mut g.players[0].assets {
        c.exhausted = true;
    }
    for _ in 0..3 {
        credit(&mut g, 0);
    }
    let h = held(&mut g, "BQ029", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(h),
            region: Some(0),
            option: Some("pay-extra-1".into()),
            ..Action::new("deploy")
        },
    );
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(f.money_units(0), 2);
    assert_eq!(f.money_units(1), 1);
    assert!(
        matches!(f.already_paid.as_slice(),[PaidCost::Assets(a),PaidCost::BQ029ExtraAssets(b)]if a.is_empty()&&b.is_empty())
    );
    pass_top(&mut g);
    assert!(g.pending.is_some());
    assert_eq!(g.step_resource_count(0), 0);
    restore(&mut g);
    choose(&mut g, vec![]);
}

#[test]
fn import20_industry_credits_pay_carmen_printed_and_extra_three_and_keep_qualification() {
    let mut g = game();
    fund(&mut g, 0, "JC084", 2);
    for c in &mut g.players[0].assets {
        c.exhausted = true;
    }
    for _ in 0..4 {
        credit(&mut g, 0);
    }
    let target = fixture(&mut g, "BQ091", 2);
    let h = held(&mut g, "CQ24", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(h),
            region: Some(0),
            option: Some("pay-extra-3".into()),
            ..Action::new("deploy")
        },
    );
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert_eq!(f.money_units(0), 1);
    assert_eq!(f.money_units(1), 3);
    assert!(
        matches!(f.already_paid.as_slice(),[PaidCost::Assets(a),PaidCost::CQ24ExtraAssets(b)]if a.is_empty()&&b.is_empty())
    );
    pass_top(&mut g);
    assert!(g.pending.is_some());
    choose(&mut g, vec![target.clone()]);
    choose(&mut g, vec![]);
    pass_top(&mut g);
    assert_eq!(g.board(&target).unwrap().1.controller, 0);
    assert_eq!(g.step_resource_count(0), 0);
    restore(&mut g);
}
