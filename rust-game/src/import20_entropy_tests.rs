//! XQ19 exact user ruling, real hand command and existing combat/contest paths.
use crate::{catalog, import20_entropy as e, jc029_tests::pass_top, model::*, unit_support::*};
fn action(id: &str) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: Some(e::HAND.into()),
        ..Action::new("activate")
    }
}
fn setup(n: u32) -> (Game, String, String) {
    let mut g = game();
    let carrier = field(&mut g, "JC125", 2);
    g.board_mut(&carrier).unwrap().time_markers = n;
    let h = held(&mut g, "XQ19", 0);
    (g, h, carrier)
}
fn enter() -> (Game, String) {
    let (mut g, h, _) = setup(4);
    act(&mut g, 0, action(&h));
    pass_top(&mut g);
    let id = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "XQ19")
        .unwrap()
        .id
        .clone();
    (g, id)
}
#[test]
fn xq19_printed_dash_is_typed_numeric_compare0_with_exact_red_star_stats() {
    let d = catalog::card("XQ19");
    assert_eq!(
        (&*d.name, &*d.color, d.cost, d.defense, d.unique),
        ("熵力化身", "红", 0, Some(1), false)
    );
    assert_eq!(d.cost_appearance, Some(catalog::CostAppearance::Dash));
    assert_eq!(d.magic, "星辰");
    assert_eq!(d.temporary_icons.combat, 1);
    assert!(d.rule_traits.spirit);
    assert_eq!(d.rule_traits.kill, 3);
    let (mut g, h, _) = setup(4);
    let c = g.players[0].hand.iter().find(|c| c.id == h).unwrap();
    let v = g.card_view(c, 0, None, None);
    assert_eq!(v.cost_appearance, Some(catalog::CostAppearance::Dash));
    assert!(v.cost.is_none() && v.effective_cost.is_none());
    restore(&mut g);
}
#[test]
fn xq19_hand_action_uses_four_global_time_markers_and_places_in_region_one_without_payment() {
    let (mut g, h, _) = setup(3);
    reject(&mut g, 0, action(&h));
    let other = field(&mut g, "JC125", 1);
    g.board_mut(&other).unwrap().time_markers = 1;
    assert!(g.legal_actions(0).iter().any(|a| a.action == action(&h)));
    act(&mut g, 0, action(&h));
    assert!(g.players[0].hand.iter().any(|c| c.id == h));
    assert_eq!(g.resources(0), 0);
    let f = g.stack.last().unwrap().frame.as_ref().unwrap();
    assert!(matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if ids.is_empty()));
    pass_top(&mut g);
    assert!(g.players[0].hand.iter().all(|c| c.id != h));
    assert!(g.regions[0]
        .cards
        .iter()
        .any(|c| c.definition == "XQ19" && c.id != h));
    assert!(!g.regions[1].cards.iter().any(|c| c.definition == "XQ19"));
}
#[test]
fn xq19_time_qualification_is_activation_only_after_response_removes_markers() {
    let (mut g, h, carrier) = setup(4);
    act(&mut g, 0, action(&h));
    g.board_mut(&carrier).unwrap().time_markers = 0;
    pass_top(&mut g);
    assert!(g.regions[0].cards.iter().any(|c| c.definition == "XQ19"));
}
#[test]
fn xq19_inherited_time_is_not_double_counted() {
    let (mut g, h, carrier) = setup(1);
    let mut a = g.make_card("CQ15", 0);
    a.time_markers = 2;
    g.attachments.push(Attachment {
        card: a,
        host_id: carrier.clone(),
    });
    assert_eq!(g.effective_time_markers(g.board(&carrier).unwrap().1), 3);
    reject(&mut g, 0, action(&h));
}
#[test]
fn xq19_source_hand_loss_during_response_never_rebinds_same_print() {
    let (mut g, h, _) = setup(4);
    act(&mut g, 0, action(&h));
    g.discard_hand_cards(0, std::slice::from_ref(&h)).unwrap();
    let replacement = held(&mut g, "XQ19", 0);
    pass_top(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == replacement));
    assert!(!g
        .regions
        .iter()
        .flat_map(|r| &r.cards)
        .any(|c| c.definition == "XQ19"));
}
#[test]
fn xq19_forbids_normal_faceup_play_and_paid_reveal_but_allows_secret_deploy() {
    let (mut g, h, _) = setup(4);
    fund(&mut g, 0, "JC125", 1);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(h.clone()),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    act(
        &mut g,
        0,
        Action {
            card_id: Some(h),
            region: Some(0),
            ..Action::new("conceal")
        },
    );
    let id = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "XQ19")
        .unwrap()
        .id
        .clone();
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(id),
            ..Action::new("reveal")
        },
    );
}
#[test]
fn xq19_no_region_override_extra_payload_or_enemy_hand_permission() {
    let (mut g, h, _) = setup(4);
    reject(
        &mut g,
        0,
        Action {
            region: Some(1),
            ..action(&h)
        },
    );
    reject(
        &mut g,
        0,
        Action {
            target_id: Some("p1".into()),
            ..action(&h)
        },
    );
    reject(&mut g, 1, action(&h));
}
#[test]
fn xq19_can_be_low_cost_destroy_target_after_legal_entry() {
    let (mut g, id) = enter();
    fund(&mut g, 2, "WM057", 2);
    let spell = held(&mut g, "BQ090", 2);
    g.begin_window(Window::Action(1));
    let play = Action {
        card_id: Some(spell),
        target_id: Some(id.clone()),
        ..Action::new("play")
    };
    assert!(g.legal_actions(2).iter().any(|a| a.action.kind == "play"
        && a.action.card_id == play.card_id
        && a.action.target_id == play.target_id));
    act(&mut g, 2, play);
    pass_top(&mut g);
    assert!(g.board(&id).is_none());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ19"));
}
#[test]
fn xq19_can_be_cq28_low_cost_grave_placement_without_own_time_condition() {
    let mut g = game();
    fund(&mut g, 0, "WM057", 4);
    let c = g.make_card("XQ19", 0);
    let id = c.id.clone();
    g.players[0].graveyard.push(c);
    let gate = held(&mut g, "CQ28", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(gate),
            ..Action::new("play")
        },
    );
    choose(&mut g, vec!["item-1".into()]);
    choose(&mut g, vec!["item-3".into()]);
    choose(&mut g, vec![id]);
    choose(&mut g, vec!["region:1".into()]);
    pass_top(&mut g);
    assert!(g.regions[1].cards.iter().any(|c| c.definition == "XQ19"));
}
#[test]
fn xq19_spirit_uses_existing_domain_protection_and_temporary_combat_is_initiative_only() {
    let (mut g, id) = enter();
    let c = g.board(&id).unwrap().1;
    assert!(g.spirit_protected(c));
    assert_eq!(g.icons(c, 0).combat, 1);
    g.damage(std::collections::BTreeMap::from([(id.clone(), 1)]))
        .unwrap();
    assert_eq!(g.board(&id).unwrap().1.damage, 0);
    g.first_team = 1;
    assert_eq!(g.icons(g.board(&id).unwrap().1, 0).combat, 0);
}
#[test]
fn xq19_kill3_adds_exactly_three_to_real_combat_allocation() {
    let (mut g, id) = enter();
    let target = field(&mut g, "JC094", 2);
    g.begin_window(Window::Before(0, 1));
    g.close_window().unwrap();
    g.drive().unwrap();
    let p = g.pending.clone().unwrap();
    assert_eq!(p.choice.kind, "damage");
    assert_eq!(p.choice.amount, Some(4));
    act(
        &mut g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            allocations: Some(std::collections::BTreeMap::from([(target.clone(), 4)])),
            ..Action::new("choose")
        },
    );
    assert!(g.board(&target).is_none());
    assert!(g.board(&id).is_some());
}
#[test]
fn xq19_mystic_shown_contributes_numeric_zero_to_team_total() {
    let mut g = game();
    fund(&mut g, 0, "WM057", 2);
    let source = field(&mut g, "WM057", 0);
    let shown = [
        held(&mut g, "XQ19", 0),
        held(&mut g, "JC125", 1),
        held(&mut g, "JC084", 2),
        held(&mut g, "JC125", 3),
    ];
    act(
        &mut g,
        0,
        Action {
            card_id: Some(source),
            ability_id: Some(crate::import20_broker::ACT.into()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    for _ in 0..24 {
        if let Some(p) = g.pending.clone() {
            let selected = if p.choice.kind == "mystic-commit" {
                vec![shown[p.seat].clone()]
            } else {
                p.choice
                    .options
                    .iter()
                    .take(p.choice.min.unwrap_or(0))
                    .map(|o| o.id.clone())
                    .collect()
            };
            choose(&mut g, selected)
        } else if !g.stack.is_empty() {
            pass_top(&mut g)
        } else if !g.effects.is_empty() {
            g.drive().unwrap()
        } else {
            break;
        }
    }
    let o = g.mystic_history.last().unwrap();
    assert_eq!(o.totals, [0, 1]);
    assert_eq!(o.winner_team, Some(1));
    assert_eq!(
        o.shown
            .iter()
            .find(|s| s.card.definition == "XQ19")
            .unwrap()
            .value,
        0
    );
}
#[test]
fn xq19_room_zero_cost_action_is_once_only_and_resumes_with_original_condition() {
    use crate::room::{RoomCommand, RoomEnvelope};
    let (g, h, _) = setup(4);
    let r = crate::jc029_tests::envelope(&g);
    let c = RoomCommand {
        command_id: "xq19-hand-once".into(),
        expected_version: r.revision,
        action: action(&h).into(),
    };
    let out = r.transition(0, Some(c.clone()), 9000).unwrap();
    assert!(out.error_code.is_none(), "{:?}", out.error_message);
    let next = RoomEnvelope::from_persisted(&out.state).unwrap();
    let duplicate = next.transition(0, Some(c), 9001).unwrap();
    assert_eq!(out.state, duplicate.state);
    assert_eq!(next.game.stack.len(), 1);
}

#[test]
fn xq19_batch20_admission_opens_exact_cards_without_opening_other_native_wip() {
    assert_current_serial_catalog();
    for id in e::BATCH20 {
        let mut deck = crate::deck::preset("watchers").unwrap();
        deck.cards.push(crate::catalog::DeckEntry {
            card_id: (*id).into(),
            count: 1,
        });
        let valid = crate::deck::validate(deck).unwrap();
        assert!(valid.cards.iter().any(|c| c.card_id == *id));
    }
    let mut deck = crate::deck::preset("watchers").unwrap();
    deck.cards.push(crate::catalog::DeckEntry {
        card_id: "WM043".into(),
        count: 1,
    });
    assert!(crate::deck::validate(deck).is_err());
}
