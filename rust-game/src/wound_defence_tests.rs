//! Disclosed boundary layouts; new declarations and responses use real actions.
use crate::unit_support::*;
use crate::{catalog, model::*, rules::*};

fn grant(g: &mut Game, source: &str, target: &str) {
    act(
        g,
        0,
        Action {
            card_id: Some(source.into()),
            ability_id: Some("protect-local-character".into()),
            target_id: Some(target.into()),
            ..Action::new("activate")
        },
    );
}
#[test]
fn wound_defence_original_fields_and_narrow_wound_program() {
    for (id, color, loyalty, magic, icons) in [
        (
            "JZ59",
            "紫",
            vec!["紫色"],
            "",
            Icons {
                combat: 1,
                influence: 1,
                ..Default::default()
            },
        ),
        (
            "LC12",
            "白",
            vec!["白色"],
            "星辰",
            Icons {
                influence: 1,
                ..Default::default()
            },
        ),
    ] {
        let d = catalog::card(id);
        assert_eq!(d.cost, 2);
        assert_eq!(d.color, color);
        assert_eq!(d.loyalty, loyalty);
        assert_eq!(d.magic, magic);
        assert_eq!(d.permanent_icons, icons);
        assert_eq!(d.temporary_icons, Icons::default());
        assert_eq!(d.defense, Some(1));
        assert!(!d.unique);
        assert!(!d.subtypes.iter().any(|s| s == "梦魔"));
        for a in &definition(id).abilities {
            validate_ability(id, a).unwrap();
        }
    }
    let mut bad = definition("JZ59").abilities[0].clone();
    bad.targets[0].range = Range::Anywhere;
    assert!(validate_ability("JZ59", &bad).is_err());
    bad = definition("JZ59").abilities[0].clone();
    bad.ops = vec![Op::WoundTarget { slot: 0, amount: 2 }];
    assert!(validate_ability("JZ59", &bad).is_err());
    assert!(validate_ability("LC12", &definition("JZ59").abilities[0]).is_err());
}
#[test]
fn wound_defence_hermit_exhaust_only_no_assets_and_exact_defense_only() {
    let mut g = game();
    let h = field(&mut g, "LC12", 0);
    let target = field(&mut g, "JC125", 0);
    let before = g.icons(g.board(&target).unwrap().1, 0);
    grant(&mut g, &h, &target);
    assert!(g.board(&h).unwrap().1.exhausted);
    assert_eq!(resources(&g, 0), 0);
    assert!(g.turn_attribute_modifiers.is_empty());
    reject(
        &mut g,
        0,
        Action {
            card_id: Some(h.clone()),
            target_id: Some(target.clone()),
            ..Action::new("activate")
        },
    );
    drain(&mut g);
    assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 2);
    assert_eq!(g.icons(g.board(&target).unwrap().1, 0), before);
    assert!(!g.has_renown(g.board(&target).unwrap().1));
    for s in 0..4 {
        let v = g.view(s);
        assert_eq!(
            v.regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == target)
                .unwrap()
                .defense,
            Some(2)
        );
    }
}
#[test]
fn wound_defence_hermit_rejects_self_teammate_enemy_other_region_and_noncharacters() {
    let mut g = game();
    let h = field(&mut g, "LC12", 0);
    let teammate = field(&mut g, "JC125", 1);
    let enemy = field(&mut g, "JC125", 2);
    let other = field(&mut g, "JC125", 0);
    let c = g.regions[0].cards.pop().unwrap();
    g.regions[1].cards.push(c);
    let hidden = field(&mut g, "JC125", 0);
    g.board_mut(&hidden).unwrap().face_down = true;
    let asset = g.make_card("JC125", 0);
    let aid = asset.id.clone();
    g.players[0].assets.push(asset);
    let card = held(&mut g, "JC125", 0);
    for id in [
        h.clone(),
        teammate,
        enemy,
        other,
        hidden,
        aid,
        card,
        "missing".into(),
    ] {
        reject(
            &mut g,
            0,
            Action {
                card_id: Some(h.clone()),
                target_id: Some(id),
                ..Action::new("activate")
            },
        );
        assert!(!g.board(&h).unwrap().1.exhausted);
    }
    assert!(g
        .legal_actions(0)
        .iter()
        .all(|a| a.action.card_id.as_deref() != Some(&h)));
}
#[test]
fn wound_defence_hermit_current_controller_not_owner() {
    let mut g = game();
    let h = field(&mut g, "LC12", 3);
    g.board_mut(&h).unwrap().controller = 0;
    let target = field(&mut g, "JC125", 2);
    g.board_mut(&target).unwrap().controller = 0;
    grant(&mut g, &h, &target);
    drain(&mut g);
    assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 2);
    assert_eq!(g.board(&target).unwrap().1.owner, 2);
}
#[test]
fn wound_defence_hermit_two_sources_stack_and_preserve_icons() {
    let mut g = game();
    let a = field(&mut g, "LC12", 0);
    let b = field(&mut g, "LC12", 0);
    let target = field(&mut g, "JC125", 0);
    for h in [a, b] {
        grant(&mut g, &h, &target);
        drain(&mut g);
    }
    assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 3);
    assert_eq!(g.turn_attribute_modifiers.len(), 2);
    assert!(g
        .turn_attribute_modifiers
        .iter()
        .all(|m| m.ordinary_icons == Icons::default() && !m.grants_renown));
}
#[test]
fn wound_defence_source_real_rescue_does_not_cancel_declared_grant() {
    let mut g = game();
    let h = field(&mut g, "LC12", 0);
    let target = field(&mut g, "JC125", 0);
    let rescue = field(&mut g, "JC075", 0);
    fund(&mut g, 0, "JC125", 2);
    grant(&mut g, &h, &target);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(rescue),
            target_id: Some(h.clone()),
            ..Action::new("activate")
        },
    );
    drain(&mut g);
    assert!(g.board(&h).is_none());
    assert!(g.players[0]
        .hand
        .iter()
        .any(|c| c.definition == "LC12" && c.id != h));
    assert_eq!(g.defense(g.board(&target).unwrap().1, 0), 2);
}
#[test]
fn wound_defence_target_real_rescue_stales_frame_and_fresh_deploy_gets_no_bonus() {
    let mut g = game();
    let h = field(&mut g, "LC12", 0);
    let target = field(&mut g, "JC125", 0);
    let rescue = field(&mut g, "JC075", 0);
    fund(&mut g, 0, "JC125", 2);
    grant(&mut g, &h, &target);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(rescue),
            target_id: Some(target.clone()),
            ..Action::new("activate")
        },
    );
    drain(&mut g);
    assert!(g.turn_attribute_modifiers.is_empty());
    assert!(g.board(&h).unwrap().1.exhausted);
    let id = g.players[0]
        .hand
        .iter()
        .find(|c| c.definition == "JC125")
        .unwrap()
        .id
        .clone();
    assert_ne!(id, target);
    play_card(&mut g, 0, &id);
    drain(&mut g);
    let c = g.regions[0]
        .cards
        .iter()
        .find(|c| c.definition == "JC125")
        .unwrap();
    assert_ne!(c.id, target);
    assert_eq!(g.defense(c, 0), 1);
}
#[test]
fn wound_defence_target_move_or_control_response_checkpoint_cancels_but_resolved_bonus_follows_instance(
) {
    for moved in [false, true] {
        let mut g = game();
        let h = field(&mut g, "LC12", 0);
        let target = field(&mut g, "JC125", 0);
        grant(&mut g, &h, &target);
        if moved {
            let c = g.regions[0].cards.pop().unwrap();
            g.regions[1].cards.push(c);
        } else {
            g.board_mut(&target).unwrap().controller = 2;
        }
        restore(&mut g);
        drain(&mut g);
        assert!(g.turn_attribute_modifiers.is_empty());
        assert!(g.board(&h).unwrap().1.exhausted);
    }
    let mut g = game();
    let h = field(&mut g, "LC12", 0);
    let target = field(&mut g, "JC125", 0);
    grant(&mut g, &h, &target);
    drain(&mut g);
    let mut c = g.regions[0].cards.pop().unwrap();
    c.controller = 2;
    g.regions[1].cards.push(c);
    restore(&mut g);
    assert_eq!(g.defense(g.board(&target).unwrap().1, 1), 2);
}
#[test]
fn wound_defence_cleanup_damage_clears_atomically_but_wound_causes_real_death_at_expiry() {
    for wound in [false, true] {
        let mut g = game();
        let h = field(&mut g, "LC12", 0);
        let target = field(&mut g, if wound { "JZ59" } else { "JC125" }, 0);
        let buddy = field(&mut g, "LC01", 0);
        grant(&mut g, &h, &target);
        drain(&mut g);
        if wound {
            g.board_mut(&target).unwrap().wounds = 1;
        } else {
            g.board_mut(&target).unwrap().damage = 1;
        }
        g.effect(Effect::FinishCleanup).unwrap();
        g.drive().unwrap();
        restore(&mut g);
        if wound {
            assert!(g.board(&target).is_none());
            let p = g.pending.as_ref().unwrap();
            let ChoiceResolution::Declare { declaration, .. } = &p.resolution else {
                panic!()
            };
            assert_eq!(declaration.actor, 0);
            assert_eq!(declaration.source.card.id, target);
            assert_eq!(declaration.source.card.wounds, 1);
            choose(&mut g, vec![buddy.clone()]);
            drain(&mut g);
            assert_eq!(g.board(&buddy).unwrap().1.wounds, 1);
        } else {
            let c = g.board(&target).unwrap().1;
            assert_eq!(c.damage, 0);
            assert_eq!(g.defense(c, 0), 1);
        }
        assert!(g.turn_attribute_modifiers.is_empty());
    }
}
#[test]
fn wound_defence_real_murder_death_targets_old_region_wound_persists_without_damage() {
    let mut g = game();
    let source = field(&mut g, "JZ59", 0);
    let target = field(&mut g, "LC01", 2);
    let other = field(&mut g, "LC01", 2);
    let c = g.regions[0].cards.pop().unwrap();
    g.regions[1].cards.push(c);
    kill_source(&mut g, &source);
    let p = g.pending.as_ref().unwrap();
    assert_eq!(p.seat, 0);
    assert!(p.choice.options.iter().any(|c| c.id == target));
    assert!(!p
        .choice
        .options
        .iter()
        .any(|c| c.id == other || c.id == source));
    choose(&mut g, vec![target.clone()]);
    drain(&mut g);
    let c = g.board(&target).unwrap().1;
    assert_eq!(c.wounds, 1);
    assert_eq!(c.damage, 0);
    assert_eq!(g.defense(c, 0), 3);
    for s in 0..4 {
        let v = g.view(s);
        let c = v.regions[0]
            .characters
            .iter()
            .find(|c| c.instance_id == target)
            .unwrap();
        assert_eq!(c.wounds, Some(1));
        assert_eq!(c.damage, Some(0));
    }
    let turn = g.turn;
    for _ in 0..650 {
        if g.turn > turn {
            break;
        }
        if g.pending.is_some() {
            resolve_choice(&mut g);
        } else {
            pass(&mut g);
        }
    }
    assert!(g.turn > turn);
    assert_eq!(g.board(&target).unwrap().1.wounds, 1);
}
#[test]
fn wound_defence_trigger_decline_and_no_target_are_free() {
    let mut g = game();
    let source = field(&mut g, "JZ59", 0);
    field(&mut g, "LC01", 2);
    kill_source(&mut g, &source);
    let before = resources(&g, 0);
    choose(&mut g, vec![]);
    drain(&mut g);
    assert_eq!(resources(&g, 0), before);
    assert!(g.regions[0].cards.iter().all(|c| c.wounds == 0));
    let mut g = game();
    let source = field(&mut g, "JZ59", 0);
    fund(&mut g, 0, "JC091", 3);
    let c = held(&mut g, "JC091", 0);
    play_target(&mut g, &c, &source);
    drain(&mut g);
    assert!(g.pending.is_none());
    assert!(g.regions[0].cards.is_empty());
}
#[test]
fn wound_defence_wound_bypasses_real_damage_prevention_and_defense_zero_kills() {
    let mut g = game();
    let source = field(&mut g, "JZ59", 0);
    let target = field(&mut g, "JC125", 0);
    fund(&mut g, 0, "JC078", 1);
    let shield = held(&mut g, "JC078", 0);
    play_target(&mut g, &shield, &target);
    drain(&mut g);
    assert!(g.damage_prevented(g.board(&target).unwrap().1));
    kill_source(&mut g, &source);
    choose(&mut g, vec![target.clone()]);
    drain(&mut g);
    assert!(g.board(&target).is_none());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC125" && c.id != target));
}
#[test]
fn wound_defence_hermit_response_to_real_wound_saves_until_expiry() {
    let mut g = game();
    let h = field(&mut g, "LC12", 0);
    let source = field(&mut g, "JZ59", 0);
    let target = field(&mut g, "JC125", 0);
    kill_source(&mut g, &source);
    choose(&mut g, vec![target.clone()]);
    grant(&mut g, &h, &target);
    drain(&mut g);
    let c = g.board(&target).unwrap().1;
    assert_eq!(c.wounds, 1);
    assert_eq!(g.defense(c, 0), 1);
    assert_eq!(c.damage, 0);
    g.effect(Effect::FinishCleanup).unwrap();
    g.drive().unwrap();
    restore(&mut g);
    assert!(g.board(&target).is_none());
}
#[test]
fn wound_defence_death_last_controller_not_owner_and_target_shield_still_applies() {
    let mut g = game();
    let source = field(&mut g, "JZ59", 3);
    g.board_mut(&source).unwrap().controller = 0;
    let target = field(&mut g, "LC01", 2);
    g.board_mut(&target).unwrap().shield = 1;
    kill_source(&mut g, &source);
    let p = g.pending.as_ref().unwrap();
    let ChoiceResolution::Declare { declaration, .. } = &p.resolution else {
        panic!()
    };
    assert_eq!(declaration.actor, 0);
    assert_eq!(declaration.source.card.owner, 3);
    assert_eq!(declaration.source.card.controller, 0);
    choose(&mut g, vec![target.clone()]);
    drain(&mut g);
    assert_eq!(g.board(&target).unwrap().1.shield, 0);
    assert_eq!(g.board(&target).unwrap().1.wounds, 0);
    assert!(g.players[3]
        .graveyard
        .iter()
        .any(|c| c.definition == "JZ59" && c.id != source));
}
#[test]
fn wound_defence_real_sacrificed_cost_declares_independent_death_wound_without_second_fee() {
    let mut g = game();
    let source = field(&mut g, "JZ59", 0);
    let target = field(&mut g, "LC01", 2);
    fund(&mut g, 0, "JC049", 2);
    let spell = held(&mut g, "JC049", 0);
    act(
        &mut g,
        0,
        Action {
            card_id: Some(spell),
            cost_selected: Some(vec![source.clone()]),
            ..Action::new("play")
        },
    );
    until_choice(&mut g, "trigger");
    assert_eq!(resources(&g, 0), 0);
    let p = g.pending.as_ref().unwrap();
    let ChoiceResolution::Declare { declaration, .. } = &p.resolution else {
        panic!()
    };
    assert_eq!(declaration.source.card.id, source);
    assert_eq!(declaration.actor, 0);
    choose(&mut g, vec![target.clone()]);
    drain(&mut g);
    assert_eq!(resources(&g, 0), 0);
    assert_eq!(g.board(&target).unwrap().1.wounds, 1);
    assert_eq!(
        g.players[0]
            .graveyard
            .iter()
            .filter(|c| c.definition == "JZ59")
            .count(),
        1
    );
}
