//! Explicit starting layouts; declarations, costs, responses and restoration use real actions.
use crate::{catalog, model::*, rules::*};

fn game() -> Game {
    let mut g = Game::new(
        "defence-unit".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        8,
    )
    .unwrap();
    for s in 1..4 {
        g.join(format!("P{s}"), "watchers".into()).unwrap();
    }
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
    while g.pending.is_some() {
        choose(&mut g, vec![]);
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
        p.graveyard.clear();
    }
    for r in &mut g.regions {
        r.cards.clear();
        r.influence = [0; 2];
    }
    g.regions[0].card = g.make_card("DQJC115", 0);
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    g
}
fn field(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.regions[0].cards.push(c);
    id
}
fn held(g: &mut Game, id: &str, seat: usize) -> String {
    let c = g.make_card(id, seat);
    let id = c.id.clone();
    g.players[seat].hand.push(c);
    id
}
fn fund(g: &mut Game, seat: usize, id: &str, n: usize) {
    for _ in 0..n {
        let c = g.make_card(id, seat);
        g.players[seat].assets.push(c);
    }
}
fn restore(g: &mut Game) {
    let state = serde_json::to_string(g).unwrap();
    let views = (0..4)
        .map(|s| serde_json::to_value(g.view(s)).unwrap())
        .collect::<Vec<_>>();
    *g = Game::from_persisted(&state).unwrap();
    assert_eq!(serde_json::to_string(g).unwrap(), state);
    for s in 0..4 {
        assert_eq!(serde_json::to_value(g.view(s)).unwrap(), views[s]);
    }
}
fn act(g: &mut Game, s: usize, a: Action) {
    restore(g);
    g.apply(s, a).unwrap();
    restore(g);
}
fn reject(g: &mut Game, s: usize, a: Action) {
    let before = serde_json::to_string(g).unwrap();
    assert!(g.apply(s, a).is_err());
    assert_eq!(serde_json::to_string(g).unwrap(), before);
    restore(g);
}
fn choose(g: &mut Game, ids: Vec<String>) {
    let p = g.pending.clone().unwrap();
    act(
        g,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(ids),
            ..Action::new("choose")
        },
    );
}
fn pass(g: &mut Game) {
    let s = (0..4)
        .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
        .unwrap();
    act(g, s, Action::new("pass"));
}
fn top(g: &mut Game) {
    let n = g.stack.len();
    assert!(n > 0);
    for _ in 0..64 {
        if g.pending.is_some() {
            resolve_choice(g);
        } else if g.stack.len() < n {
            return;
        } else {
            pass(g);
        }
    }
    panic!("bounded stack");
}
fn resolve_choice(g: &mut Game) {
    let p = g.pending.clone().unwrap();
    if matches!(
        p.resolution,
        ChoiceResolution::Forecast { .. }
            | ChoiceResolution::Frame {
                choice: FrameChoice::Forecast { .. },
                ..
            }
    ) {
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                top: Some(ids),
                bottom: Some(vec![]),
                ..Action::new("choose")
            },
        );
    } else if matches!(p.resolution, ChoiceResolution::Bottom { .. }) {
        let ids = p.choice.options.iter().map(|o| o.id.clone()).collect();
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                bottom: Some(ids),
                ..Action::new("choose")
            },
        );
    } else if matches!(p.resolution, ChoiceResolution::Damage { .. }) {
        let mut allocations = std::collections::BTreeMap::new();
        allocations.insert(p.choice.options[0].id.clone(), p.choice.amount.unwrap());
        act(
            g,
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                allocations: Some(allocations),
                ..Action::new("choose")
            },
        );
    } else {
        let ids = p
            .choice
            .options
            .iter()
            .take(p.choice.min.unwrap_or(0))
            .map(|o| o.id.clone())
            .collect();
        choose(g, ids);
    }
}
fn play(g: &mut Game, source: &str, s: usize, target: &str) {
    act(
        g,
        s,
        Action {
            card_id: Some(source.into()),
            target_id: Some(target.into()),
            ..Action::new("play")
        },
    );
    top(g);
}
fn cast(g: &mut Game, id: &str, s: usize, target: &str) {
    let source = held(g, id, s);
    play(g, &source, s, target);
}
fn attach(g: &mut Game, id: &str, target: &str) -> String {
    cast(g, id, 0, target);
    g.attachments.last().unwrap().card.id.clone()
}
fn activate(g: &mut Game, source: &str, key: &str, target: Option<&str>, sacrifice: Option<&str>) {
    act(g, 0, action(source, key, target, sacrifice));
}
fn action(source: &str, key: &str, target: Option<&str>, sacrifice: Option<&str>) -> Action {
    Action {
        card_id: Some(source.into()),
        ability_id: Some(key.into()),
        target_id: target.map(str::to_string),
        cost_selected: sacrifice.map(|id| vec![id.into()]),
        ..Action::new("activate")
    }
}
fn funded() -> Game {
    let mut g = game();
    fund(&mut g, 0, "JC014", 8);
    fund(&mut g, 0, "JC084", 4);
    g
}
fn moved(g: &mut Game, id: &str, region: usize) {
    let (_, c) = g.remove_board(id).unwrap();
    g.regions[region].cards.push(c);
}
fn icons(g: &Game, id: &str) -> Icons {
    let (r, c) = g.board(id).unwrap();
    g.current_icons(c, r)
}
fn override_count(g: &Game) -> usize {
    g.turn_attribute_modifiers
        .iter()
        .filter(|m| m.printed_defense_override == Some(1))
        .count()
}

#[test]
fn equipment_original_fields_and_bindings() {
    assert_eq!(catalog::catalog().cards.len(), 100);
    for (id, cost, color, loyalty, magic) in [
        ("JC116", 1, "中立", 0, ""),
        ("JC020", 2, "绿", 1, ""),
        ("JC093", 1, "黑", 2, "死亡"),
        ("XQ07", 1, "绿", 1, "神圣"),
    ] {
        let d = catalog::card(id);
        assert_eq!(
            (d.cost, d.color.as_str(), d.loyalty.len(), d.magic.as_str()),
            (cost, color, loyalty, magic)
        );
        assert_eq!(d.kind, "attachment");
        assert_eq!(d.subtypes[0], "装备");
        let spec = definition(id);
        assert!(spec.abilities[0].play_only);
        for a in &spec.abilities {
            validate_ability(id, a).unwrap();
        }
    }
    assert!(definition("JC116").traits.retreat);
    assert_eq!(definition("JC093").abilities[1].per_turn_limit, Some(2));
    assert!(catalog::card("JC093")
        .text
        .contains("每回合至多只能发动两次"));
    assert!(catalog::card("XQ07").text.contains("印刷防御力成为1"));
    assert_eq!(
        definition("JC020")
            .attachment
            .as_ref()
            .unwrap()
            .host
            .relation,
        Relation::ControlledByActor
    );
    assert_eq!(
        definition("JC093")
            .attachment
            .as_ref()
            .unwrap()
            .host
            .relation,
        Relation::Any
    );
}
#[test]
fn equipment_four_paid_play_and_role_guards() {
    let mut g = funded();
    let friend = field(&mut g, "JC125", 1);
    let own_host = field(&mut g, "JC125", 0);
    let enemy = field(&mut g, "JC125", 2);
    let initial = g.resources(0);
    let gun = attach(&mut g, "JC116", &enemy);
    assert_eq!(icons(&g, &enemy).combat, 1);
    for id in ["JC020", "JC093", "XQ07"] {
        attach(&mut g, id, if id == "JC093" { &friend } else { &own_host });
    }
    assert_eq!(g.resources(0), initial - 5);
    assert_eq!(g.attachments.len(), 4);
    reject(&mut g, 0, action(&gun, "attach", Some(&friend), None));
    let knife = held(&mut g, "JC020", 0);
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(knife.clone()),
            target_id: Some(enemy),
            ..Action::new("play")
        },
    );
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(knife),
            ability_id: Some("exhaust-local-hidden".into()),
            target_id: Some(friend),
            ..Action::new("play")
        },
    );
}
#[test]
fn equipment_controller_only_declarations_reject_teammate_enemy_atomically() {
    for (id, cost) in [("JC020", 2), ("XQ07", 1)] {
        let mut g = funded();
        let own_host = field(&mut g, "JC125", 0);
        let teammate = field(&mut g, "JC125", 1);
        let enemy = field(&mut g, "JC125", 2);
        let source = held(&mut g, id, 0);
        let play_at = |target: &str| Action {
            card_id: Some(source.clone()),
            target_id: Some(target.into()),
            ..Action::new("play")
        };
        assert!(g.legal_actions(0).iter().any(|a| a.action.kind == "play"
            && a.action.card_id.as_deref() == Some(source.as_str())
            && a.action.target_id.as_deref() == Some(own_host.as_str())));
        let before = g.resources(0);
        for forbidden in [&teammate, &enemy] {
            assert!(!g.legal_actions(0).iter().any(|a| a.action.kind == "play"
                && a.action.card_id.as_deref() == Some(source.as_str())
                && a.action.target_id.as_deref() == Some(forbidden.as_str())));
            reject(&mut g, 0, play_at(forbidden));
            assert_eq!(g.resources(0), before);
            assert!(g.players[0].hand.iter().any(|c| c.id == source));
            assert!(g.attachments.is_empty() && g.players[0].graveyard.is_empty());
        }
        play(&mut g, &source, 0, &own_host);
        assert_eq!(g.resources(0), before - cost);
        assert_eq!(g.attachments[0].host_id, own_host);
        assert_ne!(g.attachments[0].card.id, source);
    }
}
#[test]
fn equipment_controller_only_host_taken_by_teammate_uses_existing_cleanup() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let gun = attach(&mut g, "JC116", &host);
    let knife = attach(&mut g, "JC020", &host);
    let blade = attach(&mut g, "JC093", &host);
    let water = attach(&mut g, "XQ07", &host);
    fund(&mut g, 1, "XQ16", 6);
    let source = field(&mut g, "JZ27", 1);
    g.board_mut(&source).unwrap().face_down = true;
    act(
        &mut g,
        1,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    );
    for _ in 0..80 {
        if g.pending.is_some() {
            break;
        }
        pass(&mut g);
    }
    assert!(g
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .any(|o| o.id == host));
    choose(&mut g, vec![host.clone()]);
    top(&mut g);
    assert_eq!(g.board(&host).unwrap().1.controller, 1);
    for (id, old) in [("JC020", knife), ("XQ07", water)] {
        assert!(g.board(&old).is_none());
        let grave = g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == id)
            .collect::<Vec<_>>();
        assert_eq!(grave.len(), 1);
        assert_ne!(grave[0].id, old);
        assert_eq!((grave[0].owner, grave[0].controller), (0, 0));
    }
    assert!(g.board(&gun).is_some() && g.board(&blade).is_some());
    restore(&mut g);
}
#[test]
fn equipment_knife_exhausts_attachment_not_host_and_uses_frozen_region() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let hidden = field(&mut g, "JC125", 1);
    g.board_mut(&hidden).unwrap().face_down = true;
    let knife = attach(&mut g, "JC020", &host);
    assert!(g
        .legal_actions(0)
        .iter()
        .any(|a| a.action == action(&knife, "exhaust-local-hidden", Some(&hidden), None)));
    activate(&mut g, &knife, "exhaust-local-hidden", Some(&hidden), None);
    assert!(g.board(&knife).unwrap().1.exhausted);
    assert!(!g.board(&host).unwrap().1.exhausted);
    moved(&mut g, &host, 1);
    restore(&mut g);
    top(&mut g);
    assert!(g.board(&hidden).unwrap().1.exhausted);
    reject(
        &mut g,
        0,
        action(&knife, "exhaust-local-hidden", Some(&hidden), None),
    );
}
#[test]
fn equipment_knife_target_outside_snapshot_cancels_without_refund() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let hidden = field(&mut g, "JC125", 1);
    g.board_mut(&hidden).unwrap().face_down = true;
    let knife = attach(&mut g, "JC020", &host);
    activate(&mut g, &knife, "exhaust-local-hidden", Some(&hidden), None);
    moved(&mut g, &hidden, 1);
    top(&mut g);
    assert!(!g.board(&hidden).unwrap().1.exhausted);
    assert!(g.board(&knife).unwrap().1.exhausted);
}
#[test]
fn equipment_knife_refreshes_with_other_in_play_cards_at_next_turn() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let hidden = field(&mut g, "JC125", 2);
    g.board_mut(&hidden).unwrap().face_down = true;
    let knife = attach(&mut g, "JC020", &host);
    activate(&mut g, &knife, "exhaust-local-hidden", Some(&hidden), None);
    top(&mut g);
    assert!(g.board(&knife).unwrap().1.exhausted);
    assert!(g.board(&hidden).unwrap().1.exhausted);
    g.effect(Effect::NextTurn).unwrap();
    assert!(!g.board(&knife).unwrap().1.exhausted);
    assert!(!g.board(&hidden).unwrap().1.exhausted);
    g.begin_window(Window::Action(0));
    activate(&mut g, &knife, "exhaust-local-hidden", Some(&hidden), None);
    top(&mut g);
    assert!(g.board(&hidden).unwrap().1.exhausted);
    restore(&mut g);
}
#[test]
fn equipment_attachment_sacrifice_is_paid_once_to_owner_new_instance() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let target = field(&mut g, "JC125", 1);
    let knife = attach(&mut g, "JC020", &host);
    g.board_mut(&knife).unwrap().owner = 3;
    activate(
        &mut g,
        &knife,
        "sacrifice-local-damage",
        Some(&target),
        None,
    );
    assert!(g.board(&knife).is_none());
    assert_eq!(g.board(&target).unwrap().1.damage, 0);
    let grave = &g.players[3].graveyard;
    assert_eq!(grave.len(), 1);
    assert_ne!(grave[0].id, knife);
    assert_eq!(grave[0].controller, 3);
    top(&mut g);
    assert!(g.board(&target).is_none());
    assert!(g.board(&host).is_some());
    reject(
        &mut g,
        0,
        action(&knife, "sacrifice-local-damage", Some(&host), None),
    );
    assert_eq!(g.players[3].graveyard.len(), 1);
}
#[test]
fn equipment_sacrificed_knife_invalid_target_and_shield_keep_payment() {
    for change in ["move", "shield", "reenter"] {
        let mut g = funded();
        let host = field(&mut g, "JC125", 0);
        let target = field(&mut g, "JC003", 2);
        let knife = attach(&mut g, "JC020", &host);
        activate(
            &mut g,
            &knife,
            "sacrifice-local-damage",
            Some(&target),
            None,
        );
        let actual = match change {
            "move" => {
                moved(&mut g, &target, 1);
                target.clone()
            }
            "shield" => {
                g.board_mut(&target).unwrap().shield = 1;
                target.clone()
            }
            _ => {
                let (_, c) = g.leave_board(&target).unwrap();
                let c = g.reset_zone_card(c);
                let id = c.id.clone();
                g.regions[0].cards.push(c);
                id
            }
        };
        top(&mut g);
        assert_eq!(g.board(&actual).unwrap().1.damage, 0);
        assert!(g.board(&knife).is_none());
        assert_eq!(
            g.players[0]
                .graveyard
                .iter()
                .filter(|c| c.definition == "JC020")
                .count(),
            1
        );
    }
}
#[test]
fn equipment_blade_twice_limit_illegal_cost_atomicity_and_cleanup() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let victim1 = field(&mut g, "JC125", 0);
    let victim2 = field(&mut g, "JC125", 0);
    let victim3 = field(&mut g, "JC125", 0);
    let enemy = field(&mut g, "JC125", 2);
    let blade = attach(&mut g, "JC093", &host);
    reject(
        &mut g,
        0,
        action(&blade, "sacrifice-character-host-icons", None, Some(&enemy)),
    );
    reject(
        &mut g,
        0,
        action(&blade, "sacrifice-character-host-icons", None, None),
    );
    assert!(g.turn_ability_usage.is_empty());
    for victim in [&victim1, &victim2] {
        activate(
            &mut g,
            &blade,
            "sacrifice-character-host-icons",
            None,
            Some(victim),
        );
        top(&mut g);
    }
    assert_eq!(
        icons(&g, &host),
        Icons {
            combat: 2,
            influence: 2,
            investigation: 0
        }
    );
    assert_eq!(g.turn_ability_usage[0].uses, 2);
    reject(
        &mut g,
        0,
        action(
            &blade,
            "sacrifice-character-host-icons",
            None,
            Some(&victim3),
        ),
    );
    assert!(g.board(&victim3).is_some());
    g.effect(Effect::FinishCleanup).unwrap();
    assert_eq!(g.turn_ability_usage[0].uses, 2);
    reject(
        &mut g,
        0,
        action(
            &blade,
            "sacrifice-character-host-icons",
            None,
            Some(&victim3),
        ),
    );
    g.effect(Effect::NextTurn).unwrap();
    assert!(g.turn_ability_usage.is_empty());
    assert_eq!(icons(&g, &host), Icons::default());
}
#[test]
fn equipment_blade_paid_declaration_persists_through_attachment_destruction() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let victim = field(&mut g, "JC125", 0);
    let other = field(&mut g, "JC125", 0);
    let blade = attach(&mut g, "JC093", &host);
    activate(
        &mut g,
        &blade,
        "sacrifice-character-host-icons",
        None,
        Some(&victim),
    );
    assert_eq!(g.turn_ability_usage[0].uses, 1);
    g.remove_dead(&blade, RemovalCause::Destroy);
    restore(&mut g);
    top(&mut g);
    assert_eq!(icons(&g, &host).combat, 1);
    assert_eq!(icons(&g, &other), Icons::default());
    assert_eq!(g.turn_ability_usage[0].uses, 1);
}
#[test]
fn equipment_blade_host_change_does_not_redirect_saved_effect() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let next = field(&mut g, "JC125", 0);
    let victim = field(&mut g, "JC125", 0);
    let blade = attach(&mut g, "JC093", &host);
    activate(
        &mut g,
        &blade,
        "sacrifice-character-host-icons",
        None,
        Some(&victim),
    );
    // Synthetic relationship mutation isolates the snapshot; this card has no transfer ability.
    g.attachments
        .iter_mut()
        .find(|a| a.card.id == blade)
        .unwrap()
        .host_id = next.clone();
    moved(&mut g, &host, 1);
    top(&mut g);
    assert_eq!(icons(&g, &host).combat, 1);
    assert_eq!(icons(&g, &next), Icons::default());
}
#[test]
fn equipment_blade_left_reentered_host_gets_no_grant_and_keeps_use() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let victim = field(&mut g, "JC125", 0);
    let blade = attach(&mut g, "JC093", &host);
    activate(
        &mut g,
        &blade,
        "sacrifice-character-host-icons",
        None,
        Some(&victim),
    );
    let (_, c) = g.leave_board(&host).unwrap();
    let c = g.reset_zone_card(c);
    let fresh = c.id.clone();
    g.regions[0].cards.push(c);
    top(&mut g);
    assert_ne!(fresh, host);
    assert_eq!(icons(&g, &fresh), Icons::default());
    assert_eq!(g.turn_ability_usage[0].uses, 1);
}
#[test]
fn equipment_blade_usage_survives_control_change_and_reentry_resets_instance_only() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let victim = field(&mut g, "JC125", 0);
    let second = field(&mut g, "JC125", 1);
    let blade = attach(&mut g, "JC093", &host);
    activate(
        &mut g,
        &blade,
        "sacrifice-character-host-icons",
        None,
        Some(&victim),
    );
    top(&mut g);
    g.board_mut(&blade).unwrap().controller = 1;
    restore(&mut g);
    act(
        &mut g,
        1,
        action(
            &blade,
            "sacrifice-character-host-icons",
            None,
            Some(&second),
        ),
    );
    top(&mut g);
    assert_eq!(g.turn_ability_usage[0].uses, 2);
    let third = field(&mut g, "JC125", 1);
    reject(
        &mut g,
        1,
        action(&blade, "sacrifice-character-host-icons", None, Some(&third)),
    );
    let (_, c) = g.leave_board(&blade).unwrap();
    let c = g.reset_zone_card(c);
    let new = c.id.clone();
    g.attachments.push(Attachment {
        card: c,
        host_id: host.clone(),
    });
    let last = field(&mut g, "JC125", 0);
    activate(
        &mut g,
        &new,
        "sacrifice-character-host-icons",
        None,
        Some(&last),
    );
    top(&mut g);
    assert_eq!(
        g.turn_ability_usage
            .iter()
            .find(|u| u.source_instance == new)
            .unwrap()
            .uses,
        1
    );
}
#[test]
fn equipment_water_changes_printed_base_keeps_bonuses_wounds_and_damage() {
    let mut g = funded();
    let host = field(&mut g, "LC01", 0);
    let no_magic = field(&mut g, "JC125", 1);
    let helper = field(&mut g, "JC071", 0);
    let vest = attach(&mut g, "XQ47", &host);
    let water = attach(&mut g, "XQ07", &host);
    activate(
        &mut g,
        &helper,
        "protect-local-character",
        Some(&host),
        None,
    );
    top(&mut g);
    g.board_mut(&host).unwrap().wounds = 1;
    g.board_mut(&host).unwrap().damage = 1;
    let before = g.defense(g.board(&host).unwrap().1, 0);
    assert_eq!(before, 5);
    activate(
        &mut g,
        &water,
        "sacrifice-local-printed-defense",
        None,
        None,
    );
    top(&mut g);
    let c = g.board(&host).unwrap().1;
    assert_eq!((g.defense(c, 0), c.wounds, c.damage), (2, 1, 1));
    assert!(g.board(&vest).is_some());
    assert_eq!(g.printed_defense_override(c), Some(1));
    assert_eq!(
        g.printed_defense_override(g.board(&no_magic).unwrap().1),
        None
    );
    for s in 0..4 {
        assert_eq!(
            g.view(s).regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == host)
                .unwrap()
                .current_printed_defense,
            Some(1)
        );
    }
}
#[test]
fn equipment_water_resolves_current_public_magic_set_at_original_region_only() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let current = field(&mut g, "LC01", 1);
    let hidden = field(&mut g, "JC003", 1);
    g.board_mut(&hidden).unwrap().face_down = true;
    let water = attach(&mut g, "XQ07", &host);
    activate(
        &mut g,
        &water,
        "sacrifice-local-printed-defense",
        None,
        None,
    );
    assert!(g.board(&water).is_none());
    moved(&mut g, &host, 1);
    let entered = field(&mut g, "JZ27", 2);
    let outside = field(&mut g, "JC003", 0);
    moved(&mut g, &outside, 1);
    top(&mut g);
    assert_eq!(override_count(&g), 2);
    for id in [&current, &entered] {
        assert_eq!(g.printed_defense_override(g.board(id).unwrap().1), Some(1));
    }
    for id in [&hidden, &outside, &host] {
        assert_eq!(g.printed_defense_override(g.board(id).unwrap().1), None);
    }
    let late = field(&mut g, "JC003", 0);
    assert_eq!(g.printed_defense_override(g.board(&late).unwrap().1), None);
    for s in 0..4 {
        assert!(g.view(s).regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == hidden)
            .unwrap()
            .current_printed_defense
            .is_none());
    }
}
#[test]
fn equipment_water_nontargeted_effect_ignores_barrier_shield_and_checks_lethal() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let magic = field(&mut g, "LC01", 1);
    let c = g.make_card("BQ022", 1);
    g.attachments.push(Attachment {
        card: c,
        host_id: magic.clone(),
    });
    g.board_mut(&magic).unwrap().shield = 2;
    let water = attach(&mut g, "XQ07", &host);
    activate(
        &mut g,
        &water,
        "sacrifice-local-printed-defense",
        None,
        None,
    );
    top(&mut g);
    assert_eq!(
        g.printed_defense_override(g.board(&magic).unwrap().1),
        Some(1)
    );
    assert_eq!(g.board(&magic).unwrap().1.shield, 2);
    let mut g = funded();
    let host = field(&mut g, "JC125", 0);
    let magic = field(&mut g, "LC01", 1);
    g.board_mut(&magic).unwrap().damage = 1;
    let water = attach(&mut g, "XQ07", &host);
    activate(
        &mut g,
        &water,
        "sacrifice-local-printed-defense",
        None,
        None,
    );
    top(&mut g);
    assert!(g.board(&magic).is_none());
    assert_eq!(override_count(&g), 0);
}
#[test]
fn equipment_water_exact_instance_move_hide_reentry_and_turn_cleanup() {
    for change in ["move", "hide", "reenter", "cleanup"] {
        let mut g = funded();
        let host = field(&mut g, "JC125", 0);
        let magic = field(&mut g, "LC01", 1);
        let water = attach(&mut g, "XQ07", &host);
        activate(
            &mut g,
            &water,
            "sacrifice-local-printed-defense",
            None,
            None,
        );
        top(&mut g);
        match change {
            "move" => {
                moved(&mut g, &magic, 1);
                g.settle_deaths();
                assert_eq!(g.defense(g.board(&magic).unwrap().1, 1), 1);
            }
            "hide" => {
                g.board_mut(&magic).unwrap().face_down = true;
                g.settle_deaths();
                assert_eq!(override_count(&g), 0);
            }
            "reenter" => {
                let (_, c) = g.leave_board(&magic).unwrap();
                let c = g.reset_zone_card(c);
                let id = c.id.clone();
                g.regions[0].cards.push(c);
                g.settle_deaths();
                assert_eq!(g.defense(g.board(&id).unwrap().1, 0), 4);
            }
            _ => {
                g.effect(Effect::FinishCleanup).unwrap();
                assert_eq!(g.defense(g.board(&magic).unwrap().1, 0), 4);
            }
        }
        restore(&mut g);
    }
}
#[test]
fn equipment_gun_retreat_is_own_destination_not_host_and_owner_is_respected() {
    let mut g = funded();
    let host = field(&mut g, "JC125", 1);
    let gun = attach(&mut g, "JC116", &host);
    g.board_mut(&gun).unwrap().owner = 3;
    g.prepare_region_return(0).unwrap();
    let b = g.region_return.as_mut().unwrap();
    assert!(b.hand.contains(&gun));
    assert!(!b.hand.contains(&host));
    for s in 0..4 {
        b.orders[s] = Some(b.bottom[s].clone());
    }
    g.commit_region_return(0).unwrap();
    assert!(g.board(&host).is_none());
    assert!(g.players[1].deck.iter().any(|c| c.definition == "JC125"));
    let c = g.players[3]
        .hand
        .iter()
        .find(|c| c.definition == "JC116")
        .unwrap();
    assert_ne!(c.id, gun);
    assert_eq!(c.controller, 3);
    restore(&mut g);
}
#[test]
fn equipment_gun_host_departure_and_invalid_host_do_not_trigger_retreat() {
    for hide in [false, true] {
        let mut g = funded();
        let host = field(&mut g, "JC125", 1);
        let gun = attach(&mut g, "JC116", &host);
        if hide {
            g.board_mut(&host).unwrap().face_down = true;
            g.settle_deaths();
        } else {
            g.remove_dead(&host, RemovalCause::Destroy);
        }
        assert!(g.board(&gun).is_none());
        assert!(g.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC116"));
        assert!(!g.players[0].hand.iter().any(|c| c.definition == "JC116"));
    }
}
#[test]
fn equipment_finite_declarations_reject_new_programs_and_limit_variants() {
    let original = definition("JC093").abilities[1].clone();
    for variant in [0, 1, 3] {
        let mut a = original.clone();
        a.per_turn_limit = Some(variant);
        assert!(validate_ability("finite", &a).is_err());
    }
    let mut a = original.clone();
    a.activation_only = false;
    assert!(validate_ability("finite", &a).is_err());
    let mut a = original.clone();
    a.targets = definition("JC020").abilities[2].targets.clone();
    assert!(validate_ability("finite", &a).is_err());
    let mut a = definition("XQ07").abilities[1].clone();
    a.ops = vec![Op::ForEachLivingPlayer(a.ops)];
    assert!(validate_ability("finite", &a).is_err());
}
