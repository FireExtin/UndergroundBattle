//! Explicit board/asset setup then actual paid commands and normal continuations.
//! Author native regression, not browser or independent acceptance.
use crate::{
    catalog, import20_tax_ban as t, jc029_tests::pass_top, model::*, rules::*, unit_support::*,
};
fn finish(g: &mut Game, draw: bool) {
    for _ in 0..160 {
        if let Some(p) = g.pending.clone() {
            let selected = if p.choice.kind == "pete-extra-payment" {
                vec!["decline".into()]
            } else if p.choice.kind == "pete-optional-draw" {
                if draw {
                    vec!["draw".into()]
                } else {
                    vec![]
                }
            } else if p.choice.kind == "trigger" {
                vec!["accept".into()]
            } else {
                p.choice
                    .options
                    .iter()
                    .take(p.choice.min.unwrap_or(0))
                    .map(|o| o.id.clone())
                    .collect()
            };
            if p.choice.kind == t::NUMBER {
                act(
                    g,
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id),
                        selected: Some(vec![]),
                        amount: Some(0),
                        ..Action::new("choose")
                    },
                );
            } else {
                choose(g, selected);
            }
        } else if !g.stack.is_empty() {
            pass_top(g);
        } else if !g.effects.is_empty() {
            g.drive().unwrap();
        } else {
            return;
        }
    }
    panic!("bounded tax/ban flow")
}
fn at_number() -> Game {
    let mut g = game();
    fund(&mut g, 0, "JZ43", 3);
    let id = held(&mut g, "XG20", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            target_id: Some("society:p0".into()),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    choose(&mut g, vec!["accept".into()]);
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, t::NUMBER);
    g
}
fn number(g: &mut Game, n: u32) {
    let p = g.pending.clone().unwrap();
    act(
        g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![]),
            amount: Some(n),
            ..Action::new("choose")
        },
    );
}
fn ban(n: u32) -> (Game, String) {
    let mut g = at_number();
    number(&mut g, n);
    let id = g
        .attachments
        .iter()
        .find(|a| a.card.definition == "XG20")
        .unwrap()
        .card
        .id
        .clone();
    (g, id)
}
fn tax(payer: usize, available: u32) -> Game {
    let mut g = game();
    field(&mut g, "BQ057", 0);
    g.begin_window(Window::Action(g.team(payer)));
    fund(&mut g, payer, "JZ43", 4 + available as usize);
    let a = held(&mut g, "CQ19", payer);
    act(
        &mut g,
        payer,
        Action {
            card_id: Some(a),
            target_id: Some(format!("society:p{payer}")),
            ..Action::new("play")
        },
    );
    g
}
fn tax_choice(g: &mut Game) {
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger");
    choose(g, vec!["accept".into()]);
    pass_top(g);
}
#[test]
fn import20_tax_ban_exact_printed_gray_unique_nonasset_public_retreat() {
    assert_current_serial_catalog();
    let b = catalog::card("XG20");
    assert_eq!(
        (&*b.name, b.cost, b.loyalty.len(), b.unique),
        ("禁令", 3, 3, false)
    );
    assert!(b.keywords.contains(&"非资产".into()));
    let p = catalog::card("BQ057");
    assert_eq!(
        (&*p.name, p.cost, p.loyalty.len(), p.unique),
        ("皮特·冈萨雷斯", 3, 2, true)
    );
    assert_eq!(p.subtitle.as_deref(), Some("国税局副局长"));
    assert_eq!(
        p.permanent_icons,
        Icons {
            investigation: 1,
            influence: 1,
            ..Icons::default()
        }
    );
    assert_eq!(p.temporary_icons.influence, 1);
    assert_eq!(p.defense, Some(1));
    assert!(definition("BQ057").traits.public && definition("BQ057").traits.retreat);
    validate_definitions(definitions()).unwrap();
}
#[test]
fn import20_tax_ban_real_attach_number_zero_public_four_seats() {
    let (mut g, id) = ban(0);
    assert_eq!(g.resources(0), 0);
    assert_eq!(g.in_play(&id).unwrap().1.chosen_number, Some(0));
    for seat in 0..4 {
        assert_eq!(
            g.view(seat)
                .attachments
                .iter()
                .find(|a| a.card.instance_id == id)
                .unwrap()
                .card
                .chosen_number,
            Some(0)
        );
    }
    restore(&mut g);
}
#[test]
fn import20_tax_ban_number_is_not_limited_to_catalog_and_new_instance_clears_it() {
    let (mut g, id) = ban(u32::MAX);
    let c = g.in_play(&id).unwrap().1.clone();
    let fresh = g.fresh(c);
    assert_eq!(fresh.chosen_number, None);
    assert_ne!(fresh.id, id);
    restore(&mut g);
}
#[test]
fn import20_tax_ban_numeric_pending_is_private_and_rejects_missing_wrong_actor_and_payload() {
    let mut g = at_number();
    let p = g.pending.clone().unwrap();
    let l = g.legal_actions(0);
    assert_eq!(l.len(), 1);
    assert_eq!(l[0].amount_range.as_ref().unwrap().max, u32::MAX);
    assert_eq!(l[0].action.amount, Some(0));
    for seat in 1..4 {
        assert!(g.view(seat).pending_choice.is_none());
        assert!(g.legal_actions(seat).is_empty());
    }
    let a = Action {
        choice_id: Some(p.choice.id),
        amount: Some(3),
        selected: Some(vec![]),
        ..Action::new("choose")
    };
    reject(&mut g, 1, a.clone());
    let mut bad = a.clone();
    bad.amount = None;
    reject(&mut g, 0, bad);
    let mut bad = a.clone();
    bad.target_id = Some("p2".into());
    reject(&mut g, 0, bad);
    let mut bad = a.clone();
    bad.selected = Some(vec!["3".into()]);
    reject(&mut g, 0, bad);
    act(&mut g, 0, a.clone());
    reject(&mut g, 0, a);
}
#[test]
fn import20_tax_ban_real_enemy_character_and_paid_reveal_rejected_before_payment() {
    let (mut g, _) = ban(3);
    g.begin_window(Window::Action(g.team(2)));
    fund(&mut g, 2, "JC001", 5);
    let c = held(&mut g, "JC003", 2);
    reject(
        &mut g,
        2,
        Action {
            card_id: Some(c.clone()),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    assert_eq!(g.resources(2), 5);
    act(
        &mut g,
        2,
        Action {
            card_id: Some(c),
            region: Some(0),
            ..Action::new("conceal")
        },
    );
    assert_eq!(g.resources(2), 4);
    let c = g.regions[0].cards.last().unwrap().id.clone();
    reject(
        &mut g,
        2,
        Action {
            card_id: Some(c),
            ..Action::new("reveal")
        },
    );
}
#[test]
fn import20_tax_ban_enemy_spell_and_attachment_use_printed_cost() {
    let (mut g, _) = ban(3);
    g.begin_window(Window::Action(g.team(2)));
    fund(&mut g, 2, "JZ43", 8);
    let c = held(&mut g, "XG20", 2);
    reject(
        &mut g,
        2,
        Action {
            card_id: Some(c),
            target_id: Some("society:p2".into()),
            ..Action::new("play")
        },
    );
    let c = held(&mut g, "BQ064", 2);
    fund(&mut g, 2, "JC084", 3);
    reject(
        &mut g,
        2,
        Action {
            card_id: Some(c),
            ..Action::new("play")
        },
    );
}
#[test]
fn import20_tax_ban_friendly_side_and_other_printed_cost_can_play() {
    for actor in [0, 1, 2] {
        let (mut g, _) = ban(3);
        g.begin_window(Window::Action(g.team(actor)));
        fund(&mut g, actor, "JC001", 5);
        let c = held(&mut g, if actor == 2 { "JC002" } else { "JC003" }, actor);
        let r = (0..g.regions.len())
            .find(|r| g.accessible(actor, *r))
            .unwrap();
        act(
            &mut g,
            actor,
            Action {
                card_id: Some(c),
                region: Some(r),
                ..Action::new("deploy")
            },
        );
        pass_top(&mut g);
        finish(&mut g, false);
    }
}
#[test]
fn import20_tax_ban_number_not_yet_selected_allows_normal_response_and_decline_keeps_cost() {
    let mut g = game();
    fund(&mut g, 0, "JZ43", 3);
    let a = held(&mut g, "XG20", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(a),
            target_id: Some("society:p0".into()),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    assert!(g
        .attachments
        .iter()
        .any(|a| a.card.definition == "XG20" && a.card.chosen_number.is_none()));
    let c = g.make_card("JZ43", 2);
    assert!(g.xg20_check_play(2, &c).is_ok());
    choose(&mut g, vec![]);
    assert_eq!(g.resources(0), 0);
    assert!(g.attachments.iter().all(|a| a.card.chosen_number.is_none()));
}
#[test]
fn import20_tax_ban_nonasset_and_original_source_departure() {
    let mut g = game();
    let c = held(&mut g, "XG20", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(c),
            ..Action::new("asset")
        },
    );
    let (mut g, id) = ban(2);
    let c = g.make_card("JZ43", 2);
    assert!(g.xg20_check_play(2, &c).is_err());
    g.destroy_as(0, &id);
    g.drive().unwrap();
    assert!(g.xg20_check_play(2, &c).is_ok());
    restore(&mut g);
}
fn mystic(politician: Option<usize>) -> Game {
    let mut g = game();
    if let Some(seat) = politician {
        fund(&mut g, seat, "WM029", 1);
    }
    for s in 0..4 {
        g.players[s].hand.clear();
        held(&mut g, if s == 0 { "XG20" } else { "JC001" }, s);
    }
    fund(&mut g, 0, "JC084", 2);
    let a = held(&mut g, "BQ090", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(a),
            ability_id: Some(crate::import20_contest::HAND.into()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    for _ in 0..4 {
        let p = g.pending.clone().unwrap();
        let id = if p.seat == 0 {
            g.players[0]
                .hand
                .iter()
                .find(|c| c.definition == "XG20")
                .unwrap()
                .id
                .clone()
        } else {
            g.players[p.seat]
                .hand
                .iter()
                .find(|c| c.definition == "JC001")
                .unwrap()
                .id
                .clone()
        };
        choose(&mut g, vec![id]);
    }
    finish(&mut g, false);
    g
}
#[test]
fn import20_tax_ban_shown_own_politician_asset_frozen_plus_three_not_teammate() {
    for seat in [None, Some(0), Some(1)] {
        let mut g = mystic(seat);
        let s = g
            .mystic_history
            .last()
            .unwrap()
            .shown
            .iter()
            .find(|s| s.card.definition == "XG20")
            .unwrap();
        assert_eq!(s.value, if seat == Some(0) { 6 } else { 3 });
        assert_eq!(s.politician_asset.is_some(), seat == Some(0));
        g.players[0].assets.clear();
        restore(&mut g);
    }
}
#[test]
fn import20_tax_ban_pete_real_enemy_paid_attachment_taxpayer_can_pay_two() {
    let mut g = tax(2, 2);
    tax_choice(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    let n = g.players[0].hand.len();
    choose(&mut g, vec!["pay".into()]);
    assert_eq!(g.resources(2), 0);
    finish(&mut g, false);
    assert_eq!(g.players[0].hand.len(), n);
    assert!(g.attachments.iter().any(|a| a.card.definition == "CQ19"));
}
#[test]
fn import20_tax_ban_pete_refusal_original_observer_may_draw_once() {
    let mut g = tax(2, 2);
    tax_choice(&mut g);
    let n = g.players[0].hand.len();
    choose(&mut g, vec!["decline".into()]);
    assert_eq!(g.pending.as_ref().unwrap().seat, 0);
    let old = g.pending.clone().unwrap();
    choose(&mut g, vec!["draw".into()]);
    assert_eq!(g.players[0].hand.len(), n + 1);
    assert_eq!(g.resources(2), 2);
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(old.choice.id),
            selected: Some(vec!["draw".into()]),
            ..Action::new("choose")
        },
    );
    finish(&mut g, false);
}
#[test]
fn import20_tax_ban_pete_unaffordable_extra_still_optional_not_forced_draw() {
    for draw in [false, true] {
        let mut g = tax(2, 1);
        let n = g.players[0].hand.len();
        tax_choice(&mut g);
        assert_eq!(
            g.pending.as_ref().unwrap().choice.kind,
            "pete-optional-draw"
        );
        choose(&mut g, if draw { vec!["draw".into()] } else { vec![] });
        finish(&mut g, false);
        assert_eq!(g.players[0].hand.len(), n + usize::from(draw));
        assert_eq!(g.resources(2), 1);
    }
}
#[test]
fn import20_tax_ban_pete_teammate_attachment_is_not_an_enemy_play() {
    let mut g = tax(1, 2);
    assert!(g.pending.is_none());
    finish(&mut g, false);
    assert_eq!(g.resources(1), 2);
}
#[test]
fn import20_tax_ban_pete_not_triggered_by_direct_placement_or_movement() {
    let mut g = game();
    field(&mut g, "BQ057", 0);
    let c = g.make_card("CQ19", 2);
    g.attachments.push(Attachment {
        card: c.clone(),
        host_id: "society:p2".into(),
    });
    g.enter_triggers(2, "CQ19", &c.id, false);
    g.drive().unwrap();
    finish(&mut g, false);
    assert!(g.players[0].hand.is_empty());
}
#[test]
fn import20_tax_ban_pete_source_can_leave_during_response_without_cancelling_observation() {
    let mut g = tax(2, 2);
    choose(&mut g, vec!["accept".into()]);
    let pete = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "BQ057")
        .unwrap()
        .id
        .clone();
    g.destroy_as(2, &pete);
    g.drive().unwrap();
    pass_top(&mut g);
    choose(&mut g, vec!["decline".into()]);
    let n = g.players[0].hand.len();
    choose(&mut g, vec!["draw".into()]);
    finish(&mut g, false);
    assert_eq!(g.players[0].hand.len(), n + 1);
    restore(&mut g);
}
#[test]
fn import20_tax_ban_pete_wrong_payer_payload_and_payment_replay_are_atomic() {
    let mut g = tax(2, 2);
    tax_choice(&mut g);
    let p = g.pending.clone().unwrap();
    let a = Action {
        choice_id: Some(p.choice.id),
        selected: Some(vec!["pay".into()]),
        ..Action::new("choose")
    };
    reject(&mut g, 0, a.clone());
    let mut bad = a.clone();
    bad.amount = Some(2);
    reject(&mut g, 2, bad);
    act(&mut g, 2, a.clone());
    reject(&mut g, 2, a);
    assert_eq!(g.resources(2), 0);
    finish(&mut g, false);
}
fn asset(honor: u32) -> (Game, String, Action) {
    let mut g = game();
    let c = g.make_card("BQ057", 0);
    let id = c.id.clone();
    g.players[0].assets.push(c);
    g.players[0].honor = honor;
    let a = Action {
        card_id: Some(id.clone()),
        ability_id: Some(t::ASSET.into()),
        region: Some(1),
        ..Action::new("activate")
    };
    (g, id, a)
}
#[test]
fn import20_tax_ban_pete_asset_spends_three_honor_and_directly_places_fresh_uid() {
    let (mut g, id, a) = asset(4);
    let money = g.resources(0);
    assert!(g.legal_actions(0).iter().any(|l| l.action == a));
    act(&mut g, 0, a);
    assert_eq!(g.players[0].honor, 1);
    assert_eq!(g.resources(0), money);
    assert!(matches!(
        g.stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .already_paid
            .as_slice(),
        [PaidCost::Honor {
            before: 4,
            amount: 3
        }]
    ));
    pass_top(&mut g);
    finish(&mut g, false);
    let c = g.regions[1]
        .cards
        .iter()
        .find(|c| c.definition == "BQ057")
        .unwrap();
    assert_ne!(c.id, id);
    assert_eq!(c.controller, 0);
    assert!(!c.face_down && !c.exhausted);
    assert!(!g.players[0].assets.iter().any(|c| c.id == id));
    for s in 0..4 {
        assert_eq!(g.view(s).players[0].honor, 1);
    }
    restore(&mut g);
}
#[test]
fn import20_tax_ban_pete_honor_is_not_money_score_or_destruction_and_insufficient_rejects() {
    let (mut g, _, a) = asset(2);
    fund(&mut g, 0, "JZ43", 8);
    let score = g.make_card("DQJC115", 0);
    g.players[0].score_cards.push(score);
    let n = g.player_score(0);
    reject(&mut g, 0, a);
    assert_eq!(g.players[0].honor, 2);
    assert_eq!(g.player_score(0), n);
    assert_eq!(g.resources(0), 9);
}
#[test]
fn import20_tax_ban_pete_asset_action_rejects_wrong_zone_owner_target_and_extra_payload() {
    let (mut g, _, a) = asset(3);
    reject(&mut g, 2, a.clone());
    let mut bad = a.clone();
    bad.target_id = Some("p2".into());
    reject(&mut g, 0, bad);
    let mut bad = a.clone();
    bad.region = Some(usize::MAX);
    reject(&mut g, 0, bad);
    let mut bad = a.clone();
    bad.amount = Some(3);
    reject(&mut g, 0, bad);
    let card = g.players[0].assets.remove(0);
    g.players[0].hand.push(card);
    reject(&mut g, 0, a);
    assert_eq!(g.players[0].honor, 3);
}
#[test]
fn import20_tax_ban_pete_asset_departure_during_response_keeps_honor_paid_and_no_rebind() {
    let (mut g, id, a) = asset(3);
    act(&mut g, 0, a);
    let old = g.players[0].assets.remove(0);
    let fresh = g.fresh(old);
    g.players[0].assets.push(fresh);
    pass_top(&mut g);
    finish(&mut g, false);
    assert_eq!(g.players[0].honor, 0);
    assert!(!g
        .regions
        .iter()
        .any(|r| r.cards.iter().any(|c| c.definition == "BQ057")));
    assert_ne!(g.players[0].assets[0].id, id);
    restore(&mut g);
}
#[test]
fn import20_tax_ban_pete_public_paid_deploy_and_hidden_rejected() {
    let mut g = game();
    fund(&mut g, 0, "JZ43", 3);
    let c = held(&mut g, "BQ057", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(c.clone()),
            region: Some(0),
            ..Action::new("conceal")
        },
    );
    act(
        &mut g,
        0,
        Action {
            card_id: Some(c),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut g);
    finish(&mut g, false);
    assert!(g.regions[0].cards.iter().any(|c| c.definition == "BQ057"));
    assert_eq!(g.resources(0), 0);
}
#[test]
fn import20_tax_ban_original_definitions_reject_op_cost_event_and_wrapped_transplant() {
    for id in t::CARDS {
        let mut ds = definitions().clone();
        ds.get_mut(*id).unwrap().abilities.pop();
        assert!(validate_definitions(&ds).is_err());
    }
    let mut ds = definitions().clone();
    ds.get_mut("JC001").unwrap().abilities.push(ability(
        "foreign",
        "invalid",
        Timing::Fast,
        vec![],
        vec![],
        vec![Op::ForEachLivingPlayer(vec![Op::XG20ChooseNumber])],
        None,
    ));
    assert!(validate_definitions(&ds).is_err());
    let mut a = t::definition("BQ057").unwrap().abilities.remove(1);
    a.costs = vec![Cost::Honor(2)];
    assert!(t::validate_ability("BQ057", &a).is_err());
}
#[test]
fn import20_tax_ban_real_fast_response_to_entry_before_number_is_selected() {
    let mut g = game();
    fund(&mut g, 0, "JZ43", 3);
    let id = held(&mut g, "XG20", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            target_id: Some("society:p0".into()),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    choose(&mut g, vec!["accept".into()]);
    for _ in 0..4 {
        if g.can_fast(2) {
            break;
        }
        pass(&mut g);
    }
    fund(&mut g, 2, "JC084", 2);
    let target = field(&mut g, "JC001", 0);
    let spell = held(&mut g, "BQ090", 2);
    act(
        &mut g,
        2,
        Action {
            card_id: Some(spell),
            target_id: Some(target.clone()),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    pass_top(&mut g);
    number(&mut g, 2);
    g.begin_window(Window::Action(g.team(2)));
    fund(&mut g, 2, "JC084", 2);
    let target = field(&mut g, "JC001", 0);
    let spell = held(&mut g, "BQ090", 2);
    let a = Action {
        card_id: Some(spell),
        target_id: Some(target),
        ..Action::new("play")
    };
    let before = serde_json::to_string(&g).unwrap();
    assert!(g.apply(2, a).unwrap_err().contains("禁令"));
    assert_eq!(serde_json::to_string(&g).unwrap(), before);
    restore(&mut g);
}
#[test]
fn import20_tax_ban_pete_real_dual_type_bats_deploy_is_played_attachment() {
    let mut g = game();
    field(&mut g, "BQ057", 0);
    g.begin_window(Window::Action(g.team(2)));
    let price = catalog::card("BQ026").cost;
    fund(&mut g, 2, "JC029", price as usize + 2);
    let c = held(&mut g, "BQ026", 2);
    act(
        &mut g,
        2,
        Action {
            card_id: Some(c),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    tax_choice(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    choose(&mut g, vec!["pay".into()]);
    finish(&mut g, false);
    assert_eq!(g.resources(2), 0);
}
#[test]
fn import20_tax_ban_activated_card_ability_and_direct_placement_are_not_paid_play() {
    let (mut g, _) = ban(4);
    g.begin_window(Window::Action(g.team(2)));
    let c = g.make_card("CQ19", 2);
    let id = c.id.clone();
    g.attachments.push(Attachment {
        card: c,
        host_id: "society:p2".into(),
    });
    act(
        &mut g,
        2,
        Action {
            card_id: Some(id.clone()),
            ability_id: Some(crate::import20_sentinel::CHARGE.into()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    assert_eq!(g.in_play(&id).unwrap().1.charge_markers, 1);
    let c = g.make_card("CQ18", 2);
    g.regions[0].cards.push(c.clone());
    g.enter_triggers(2, "CQ18", &c.id, false);
    g.drive().unwrap();
    finish(&mut g, false);
    assert!(g.board(&c.id).is_some());
}
#[test]
fn import20_tax_ban_room_original_numeric_choice_once() {
    use crate::room::{RoomCommand, RoomEnvelope};
    let g = at_number();
    let r = crate::jc029_tests::envelope(&g);
    let action = Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        amount: Some(3),
        selected: Some(vec![]),
        ..Action::new("choose")
    };
    let cmd = RoomCommand {
        command_id: "ban-number-once".into(),
        expected_version: r.revision,
        action: action.into(),
    };
    let out = r.transition(0, Some(cmd.clone()), 9000).unwrap();
    assert!(out.error_code.is_none(), "{:?}", out.error_message);
    let n = RoomEnvelope::from_persisted(&out.state).unwrap();
    assert!(n
        .game
        .attachments
        .iter()
        .any(|a| a.card.definition == "XG20" && a.card.chosen_number == Some(3)));
    let twice = n.transition(0, Some(cmd), 9001).unwrap();
    assert_eq!(twice.state, out.state);
    assert_eq!(twice.error_code.as_deref(), Some("version_conflict"));
}
#[test]
fn import20_tax_ban_room_payer_receipt_once_and_no_observer_draw() {
    use crate::room::{RoomCommand, RoomEnvelope};
    let mut g = tax(2, 2);
    tax_choice(&mut g);
    let r = crate::jc029_tests::envelope(&g);
    let action = Action {
        choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
        selected: Some(vec!["pay".into()]),
        ..Action::new("choose")
    };
    let cmd = RoomCommand {
        command_id: "pete-pay-once".into(),
        expected_version: r.revision,
        action: action.into(),
    };
    let out = r.transition(2, Some(cmd.clone()), 9000).unwrap();
    assert!(out.error_code.is_none(), "{:?}", out.error_message);
    let n = RoomEnvelope::from_persisted(&out.state).unwrap();
    assert_eq!(n.game.resources(2), 0);
    assert!(n.game.players[0].hand.is_empty());
    let twice = n.transition(2, Some(cmd), 9001).unwrap();
    assert_eq!(twice.state, out.state);
    assert_eq!(twice.error_code.as_deref(), Some("version_conflict"));
}
#[test]
fn import20_tax_ban_secret_plan_bypasses_faceup_ban_but_remains_an_attachment_play_without_identity_leak(
) {
    let (mut g, _) = ban(1);
    field(&mut g, "BQ057", 0);
    g.begin_window(Window::Action(g.team(2)));
    g.add_society_markers(2, 0, 1);
    fund(&mut g, 2, "WM046", 3);
    let id = held(&mut g, "WM046", 2);
    act(
        &mut g,
        2,
        Action {
            card_id: Some(id),
            target_id: Some("society:p2".into()),
            option: Some("secret-plan".into()),
            ..Action::new("play")
        },
    );
    for s in [0, 1, 3] {
        let v = g.view(s);
        let stack = v.stack.last().unwrap();
        assert_eq!(stack.card_id, None);
        assert_eq!(stack.label, "秘密计划");
        assert!(!v
            .log
            .iter()
            .any(|l| l.text.contains(catalog::card("WM046").name.as_str())));
    }
    tax_choice(&mut g);
    choose(&mut g, vec!["pay".into()]);
    finish(&mut g, false);
    assert!(g
        .attachments
        .iter()
        .any(|a| a.card.definition == "WM046" && a.card.face_down));
}
#[test]
fn import20_tax_ban_two_personal_petes_observe_one_enemy_play_each_exactly_once() {
    let mut g = game();
    field(&mut g, "BQ057", 0);
    field(&mut g, "BQ057", 1);
    g.begin_window(Window::Action(g.team(2)));
    fund(&mut g, 2, "JZ43", 6);
    let id = held(&mut g, "CQ19", 2);
    act(
        &mut g,
        2,
        Action {
            card_id: Some(id),
            target_id: Some("society:p2".into()),
            ..Action::new("play")
        },
    );
    finish(&mut g, true);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[1].hand.len(), 1);
    assert_eq!(g.resources(2), 2);
}
#[test]
fn import20_tax_ban_pete_a_spell_play_and_dual_character_paid_reveal_do_not_trigger() {
    let mut g = game();
    field(&mut g, "BQ057", 0);
    g.begin_window(Window::Action(g.team(2)));
    fund(&mut g, 2, "JC084", 2);
    let target = field(&mut g, "JC001", 1);
    let id = held(&mut g, "BQ090", 2);
    act(
        &mut g,
        2,
        Action {
            card_id: Some(id),
            target_id: Some(target),
            ..Action::new("play")
        },
    );
    assert!(g.pending.is_none());
    finish(&mut g, true);
    assert!(g.players[0].hand.is_empty());
    let mut g = game();
    field(&mut g, "BQ057", 0);
    let id = field(&mut g, "BQ026", 2);
    g.board_mut(&id).unwrap().face_down = true;
    g.begin_window(Window::Action(g.team(2)));
    fund(&mut g, 2, "JC029", 3);
    act(
        &mut g,
        2,
        Action {
            card_id: Some(id),
            ..Action::new("reveal")
        },
    );
    assert!(g.pending.is_none());
    finish(&mut g, true);
    assert!(g.players[0].hand.is_empty());
}
#[test]
fn import20_tax_ban_pete_asset_entry_obeys_existing_unique_in_play_cleanup() {
    let (mut g, _, a) = asset(3);
    field(&mut g, "BQ057", 0);
    act(&mut g, 0, a);
    pass_top(&mut g);
    finish(&mut g, false);
    assert_eq!(
        g.regions
            .iter()
            .flat_map(|r| &r.cards)
            .filter(|c| c.definition == "BQ057" && c.controller == 0)
            .count(),
        1
    );
    assert_eq!(g.players[0].honor, 0);
}
