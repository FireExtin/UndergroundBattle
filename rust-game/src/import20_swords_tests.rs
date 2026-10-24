//! Author Native proof through paid commands, response stack and private choice.
use crate::{catalog, import20_swords as s, jc029_tests::pass_top, model::*, unit_support::*};
fn attached(owner: usize, controller: usize) -> (Game, String, String) {
    let mut g = game();
    let host = field(&mut g, "JC125", controller);
    fund(&mut g, owner, "LC16", 6);
    let sword = held(&mut g, "CQ15", owner);
    g.begin_window(Window::Action(g.team(owner)));
    act(
        &mut g,
        owner,
        Action {
            card_id: Some(sword),
            target_id: Some(host.clone()),
            ..Action::new("play")
        },
    );
    pass_top(&mut g);
    let sword = g
        .attachments
        .iter()
        .find(|a| a.card.definition == "CQ15")
        .unwrap()
        .card
        .id
        .clone();
    g.begin_window(Window::Action(g.team(controller)));
    (g, host, sword)
}
fn activation(host: &str, sword: &str, target: &str) -> Action {
    Action {
        card_id: Some(host.into()),
        ability_id: Some(format!("{}{}", s::PREFIX, sword)),
        target_id: Some(target.into()),
        ..Action::new("activate")
    }
}
fn at_search(g: &mut Game) {
    pass_top(g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, s::SEARCH);
}
#[test]
fn cq15_printed_metadata_blood_loyalty_and_attachment() {
    let (g, h, _) = attached(0, 0);
    let d = catalog::card("CQ15");
    assert_eq!(
        (&*d.color, d.cost, d.loyalty.clone(), d.unique, d.supported),
        ("红", 6, vec!["鲜血".to_string(); 4], true, false)
    );
    assert_eq!(d.subtitle.as_deref(), Some("杀伐至宝"));
    let (_, c) = g.board(&h).unwrap();
    assert_eq!(g.effective_wound(c), 2);
    assert_eq!(g.icons(c, 0).combat, 2);
}
#[test]
fn cq15_real_destroy_then_three_zones_seal_and_four_seat_privacy() {
    let (mut g, h, sword) = attached(0, 0);
    let target = field(&mut g, "JC125", 1);
    g.players[1].deck.clear();
    let d = g.make_card("JC125", 1);
    let did = d.id.clone();
    g.players[1].deck.push(d);
    let hid = held(&mut g, "JC125", 1);
    let c = g.make_card("JC125", 1);
    let gid = c.id.clone();
    g.players[1].graveyard.push(c);
    let other = g.make_card("JC126", 1);
    g.players[1].deck.push(other);
    act(&mut g, 0, activation(&h, &sword, &target));
    assert!(g.board(&h).unwrap().1.exhausted);
    assert!(g.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "CQ15" && c.id != sword));
    assert!(g.attachments.is_empty());
    assert_eq!(g.effective_wound(g.board(&h).unwrap().1), 0);
    at_search(&mut g);
    assert!(g.board(&target).is_none());
    let p = g.pending.clone().unwrap();
    assert_eq!(p.choice.options.len(), 4);
    for seat in 1..4 {
        let view = serde_json::to_value(g.view(seat)).unwrap();
        assert!(
            !view
                .to_string()
                .contains(&p.choice.options[0].id.to_string())
                || seat == 1
        );
        assert!(view["choice"].is_null());
    }
    choose(&mut g, vec![did, hid, gid]);
    assert!(g.pending.is_none());
    assert_eq!(g.sealed_cards.len(), 3);
    assert!(g
        .sealed_cards
        .iter()
        .all(|s| s.host_id == "society:p0" && s.card.owner == 1));
    assert!(g.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125"));
    restore(&mut g);
}
#[test]
fn cq15_enemy_host_pays_and_sword_returns_to_original_owner() {
    let (mut g, h, sword) = attached(0, 2);
    let t = field(&mut g, "JC125", 0);
    assert!(!g.legal_actions(0).iter().any(|a| a
        .action
        .ability_id
        .as_deref()
        .is_some_and(|k| k.starts_with(s::PREFIX))));
    act(&mut g, 2, activation(&h, &sword, &t));
    assert!(g.players[0].hand.iter().any(|c| c.definition == "CQ15"));
    assert!(!g.players[2].hand.iter().any(|c| c.definition == "CQ15"));
    at_search(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().seat, 2);
    let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
    choose(&mut g, vec![id]);
    assert_eq!(g.sealed_cards[0].host_id, "society:p2");
}
#[test]
fn cq15_target_controller_is_frozen_before_owner_grave_reset() {
    let (mut g, h, sword) = attached(0, 0);
    let t = field(&mut g, "JC125", 1);
    g.board_mut(&t).unwrap().controller = 3;
    let mine = held(&mut g, "JC125", 3);
    act(&mut g, 0, activation(&h, &sword, &t));
    at_search(&mut g);
    let p = g.pending.as_ref().unwrap();
    assert!(p.choice.options.iter().any(|o| o.id == mine));
    assert!(!p
        .choice
        .options
        .iter()
        .any(|o| g.players[1].graveyard.iter().any(|c| c.id == o.id)));
    choose(&mut g, vec![mine]);
    assert_eq!(g.sealed_cards[0].card.owner, 3);
}
#[test]
fn cq15_return_is_cost_when_target_leaves_during_response() {
    let (mut g, h, sword) = attached(0, 0);
    let t = field(&mut g, "JC125", 1);
    act(&mut g, 0, activation(&h, &sword, &t));
    g.remove_dead(&t, RemovalCause::Destroy);
    pass_top(&mut g);
    assert!(g.pending.is_none());
    assert!(g.players[0].hand.iter().any(|c| c.definition == "CQ15"));
    assert!(g.board(&h).unwrap().1.exhausted);
    assert!(g.sealed_cards.is_empty());
}
#[test]
fn cq15_paid_effect_survives_host_departure() {
    let (mut g, h, sword) = attached(0, 0);
    let t = field(&mut g, "JC125", 1);
    act(&mut g, 0, activation(&h, &sword, &t));
    g.remove_dead(&h, RemovalCause::Destroy);
    at_search(&mut g);
    choose(&mut g, vec![]);
    assert!(g.board(&t).is_none());
}
#[test]
fn cq15_zero_selection_still_shuffles_original_controller() {
    let (mut g, h, sword) = attached(0, 0);
    let t = field(&mut g, "JC125", 1);
    act(&mut g, 0, activation(&h, &sword, &t));
    at_search(&mut g);
    let before = g.players[1]
        .deck
        .iter()
        .map(|c| c.id.clone())
        .collect::<Vec<_>>();
    let mut expected = g.clone();
    expected.shuffle_player(1);
    let order = expected.players[1]
        .deck
        .iter()
        .map(|c| c.id.clone())
        .collect::<Vec<_>>();
    assert_ne!(before, order);
    choose(&mut g, vec![]);
    assert_eq!(
        g.players[1]
            .deck
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>(),
        order
    );
    assert!(g.sealed_cards.is_empty());
}
#[test]
fn cq15_rejects_wrong_actor_wrong_sword_and_extra_payload_without_cost() {
    let (mut g, h, sword) = attached(0, 0);
    let t = field(&mut g, "JC125", 1);
    for (actor, mut a) in [
        (1, activation(&h, &sword, &t)),
        (0, activation(&t, &sword, &h)),
        (0, activation(&h, "missing", &t)),
        (0, activation(&h, &sword, &t)),
    ] {
        if actor == 0
            && a.card_id.as_deref() == Some(&h)
            && a.ability_id.as_deref() == Some(&format!("{}{}", s::PREFIX, sword))
        {
            a.option = Some("extra".into())
        }
        let state = serde_json::to_string(&g).unwrap();
        assert!(g.apply(actor, a).is_err());
        assert_eq!(state, serde_json::to_string(&g).unwrap());
    }
}
#[test]
fn cq15_search_rejects_foreign_card_and_duplicate_selection() {
    let (mut g, h, sword) = attached(0, 0);
    let t = field(&mut g, "JC125", 1);
    act(&mut g, 0, activation(&h, &sword, &t));
    at_search(&mut g);
    let p = g.pending.clone().unwrap();
    let id = p.choice.options[0].id.clone();
    for ids in [vec!["missing".into()], vec![id.clone(), id]] {
        let before = serde_json::to_string(&g).unwrap();
        assert!(g
            .apply(
                0,
                Action {
                    choice_id: Some(p.choice.id.clone()),
                    selected: Some(ids),
                    ..Action::new("choose")
                }
            )
            .is_err());
        assert_eq!(before, serde_json::to_string(&g).unwrap());
    }
}
#[test]
fn cq15_grant_program_cannot_be_transplanted_or_shortened() {
    let (mut g, _, _) = attached(0, 0);
    let grant = g.attachments[0].clone();
    let mut a = s::granted(&grant);
    assert!(crate::rules::validate_ability("JC125", &a).is_ok());
    a.costs.pop();
    assert!(crate::rules::validate_ability("JC125", &a).is_err());
    let mut defs = crate::rules::definitions().clone();
    defs.get_mut("JC125")
        .unwrap()
        .abilities
        .push(s::granted(&grant));
    assert!(crate::rules::validate_definitions(&defs).is_err());
    g.attachments.clear();
}

#[test]
fn cq15_granted_wound_replaces_equal_combat_damage_without_bonus_damage() {
    let (mut g, h, _) = attached(0, 0);
    let t = field(&mut g, "JC094", 2);
    g.begin_window(Window::Before(0, 1));
    g.close_window().unwrap();
    g.drive().unwrap();
    let p = g.pending.clone().unwrap();
    assert_eq!(p.choice.kind, "damage");
    assert_eq!(p.choice.amount, Some(2));
    act(
        &mut g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            allocations: Some(std::collections::BTreeMap::from([(t.clone(), 2)])),
            ..Action::new("choose")
        },
    );
    let c = g.board(&t).unwrap().1;
    assert_eq!((c.damage, c.wounds), (0, 2));
    assert_eq!(g.defense(c, 0), 1);
    assert!(g.board(&h).is_some());
}
#[test]
fn cq15_actual_attachment_owner_is_not_borrowed_into_host_authority() {
    let (mut g, h, sword) = attached(2, 0);
    let t = field(&mut g, "JC125", 2);
    let actions = g.legal_actions(0);
    assert!(actions
        .iter()
        .any(|a| a.action == activation(&h, &sword, &t)));
    assert!(!g.legal_actions(2).iter().any(|a| a
        .action
        .ability_id
        .as_deref()
        .is_some_and(|k| k.starts_with(s::PREFIX))));
    act(&mut g, 0, activation(&h, &sword, &t));
    assert!(g.players[2].hand.iter().any(|c| c.definition == "CQ15"));
    at_search(&mut g);
    choose(&mut g, vec![]);
}
#[test]
fn cq15_equipment_restriction_rejects_before_paying() {
    let mut g = game();
    fund(&mut g, 0, "LC16", 6);
    let sword = held(&mut g, "CQ15", 0);
    let h = field(&mut g, "JC001", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(sword),
            target_id: Some(h),
            ..Action::new("play")
        },
    );
    assert_eq!(g.resources(0), 6);
}
#[test]
fn cq15_exhausted_hidden_and_unattached_host_cannot_activate() {
    let (mut g, h, sword) = attached(0, 0);
    let t = field(&mut g, "JC125", 2);
    let a = activation(&h, &sword, &t);
    g.board_mut(&h).unwrap().exhausted = true;
    reject(&mut g, 0, a.clone());
    g.board_mut(&h).unwrap().exhausted = false;
    g.board_mut(&h).unwrap().face_down = true;
    reject(&mut g, 0, a.clone());
    g.board_mut(&h).unwrap().face_down = false;
    g.attachments[0].host_id = t;
    reject(&mut g, 0, a);
}
#[test]
fn cq15_room_paid_once_and_duplicate_command_keeps_return_receipt() {
    use crate::room::{RoomCommand, RoomEnvelope};
    let (g, h, sword) = attached(0, 0);
    let mut g = g;
    let t = field(&mut g, "JC125", 2);
    let r = crate::jc029_tests::envelope(&g);
    let cmd = RoomCommand {
        command_id: "cq15-cost-once".into(),
        expected_version: r.revision,
        action: activation(&h, &sword, &t).into(),
    };
    let out = r.transition(0, Some(cmd.clone()), 9000).unwrap();
    assert!(out.error_code.is_none(), "{:?}", out.error_message);
    let next = RoomEnvelope::from_persisted(&out.state).unwrap();
    assert!(next.game.attachments.is_empty());
    assert_eq!(
        next.game.players[0]
            .hand
            .iter()
            .filter(|c| c.definition == "CQ15")
            .count(),
        1
    );
    let duplicate = next.transition(0, Some(cmd), 9001).unwrap();
    assert_eq!(duplicate.state, out.state);
}
