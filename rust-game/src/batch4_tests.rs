use crate::{catalog, model::*, rules::*, unit_support::*};

fn deploy(g: &mut Game) -> String {
    let id = held(g, "XG14", 0);
    act(g, 0, Action { card_id: Some(id.clone()), ..Action::new("deploy") });
    id
}
fn pending(g: &mut Game) {
    for _ in 0..64 { if g.pending.is_some() { restore(g); return; } if !g.effects.is_empty() { g.drive().unwrap(); } else { pass(g); } }
    panic!("bounded disaster observation");
}
fn disaster(g: &mut Game) {
    fund(g, 0, "JC047", 3); let id = held(g, "JC047", 0);
    act(g, 0, Action { card_id: Some(id), region: Some(1), ..Action::new("play") });
}

#[test]
fn batch4_xg14_exact_print_and_closed_chaos_and_nested_program_admission() {
    let d = catalog::card("XG14");
    assert_eq!((d.cost, d.defense, d.unique, d.supported), (4, Some(1), false, false));
    assert_eq!(d.loyalty, ["红色", "红色", "红色"]);
    assert_eq!(d.subtypes, ["熵", "法师", "灾难"]);
    assert_eq!(d.permanent_icons, Icons { combat: 2, ..Icons::default() });
    assert_eq!(d.temporary_icons, Icons { investigation: 1, ..Icons::default() });
    assert!(definition("XG14").traits.chaos); assert!(!definition("XG14").traits.spirit);
    let mut changed = crate::rules::definitions().clone(); changed.get_mut("XG14").unwrap().traits.chaos = false;
    assert!(crate::rules::validate_definitions(&changed).is_err());
    let mut transplant = definition("JC125").clone(); transplant.traits.chaos = true;
    assert!(crate::batch4::validate_definition("JC125", &transplant).is_err());
    let mut a = definition("XG14").abilities[0].clone(); a.key = "foreign".into(); a.event = None;
    a.ops = vec![Op::ForEachLivingPlayer(vec![Op::XG14DisasterInfluence { observation: None }])];
    assert!(crate::rules::validate_ability("JC125", &a).is_err());
}

#[test]
fn batch4_xg14_real_deploy_has_no_preselected_region_and_randomizes_once_at_entry() {
    let mut g = game(); g.seed = u64::MAX - 221; g.random = g.seed; fund(&mut g, 0, "XG14", 4);
    let id = held(&mut g, "XG14", 0); let random = g.random;
    let actions = g.legal_actions(0).into_iter().filter(|a| a.action.kind == "deploy" && a.action.card_id.as_deref() == Some(&id)).collect::<Vec<_>>();
    assert_eq!(actions.len(), 1); assert_eq!(actions[0].action.region, None);
    reject(&mut g, 0, Action { card_id: Some(id.clone()), region: Some(0), ..Action::new("deploy") });
    assert_eq!(g.random, random); assert_eq!(g.resources(0), 4);
    act(&mut g, 0, Action { card_id: Some(id), ..Action::new("deploy") });
    assert_eq!(g.random, random); assert_eq!(g.stack.last().unwrap().deploy_region, None);
    let mut expected = g.clone(); let region = expected.chaos_region().unwrap();
    top(&mut g); assert_eq!(g.random, expected.random); assert_eq!(g.seed, u64::MAX - 221);
    let c = g.regions[region].cards.iter().find(|c| c.definition == "XG14").unwrap();
    assert_eq!(c.time_markers, 1); assert_eq!(g.resources(0), 0); restore(&mut g);
}

#[test]
fn batch4_xg14_destroyed_regions_reduce_faceup_play_to_zero_but_not_paid_reveal() {
    for destroyed in 0..=2 {
        let mut g = game(); for r in 0..destroyed { g.regions[r].destroyed = true; }
        let cost = 4usize.saturating_sub(2 * destroyed); fund(&mut g, 0, "XG14", cost.max(3));
        deploy(&mut g);
        assert!(matches!(g.stack.last().unwrap().frame.as_ref().unwrap().already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len()==cost));
        top(&mut g); restore(&mut g);
    }
    let mut g = game(); g.regions[0].destroyed = true; g.regions[1].destroyed = true; fund(&mut g, 0, "XG14", 5);
    let id = held(&mut g, "XG14", 0); let random = g.random;
    act(&mut g, 0, Action { card_id: Some(id), region: Some(0), ..Action::new("conceal") });
    assert_eq!(g.random, random); let hidden = g.regions[0].cards.iter().find(|c| c.definition == "XG14").unwrap().id.clone();
    assert_eq!(g.board(&hidden).unwrap().1.time_markers, 0);
    act(&mut g, 0, Action { card_id: Some(hidden), ..Action::new("reveal") });
    assert!(matches!(g.stack.last().unwrap().frame.as_ref().unwrap().already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len()==4));
    top(&mut g); assert_eq!(g.resources(0), 0); assert_ne!(g.random, random); restore(&mut g);
}

#[test]
fn batch4_xg14_direct_grave_entry_overrides_destination_once_and_movement_never_rerolls() {
    let mut g = game(); let c = g.make_card("XG14", 0); let old = c.id.clone(); g.players[0].graveyard.push(c);
    let mut expected = g.clone(); let region = expected.chaos_region().unwrap();
    g.world_graveyard_entry(0, 0, &old).unwrap();
    let c = g.regions[region].cards.iter().find(|c| c.definition == "XG14").unwrap(); let id = c.id.clone();
    assert_ne!(id, old); assert_eq!(c.time_markers, 1); assert_eq!(g.random, expected.random);
    assert!(g.effects.iter().all(|e| !matches!(e, Effect::Declare { declaration: d } if d.ability.event == Some(Event::DisasterPlayed))));
    let next = (region + 1) % g.regions.len(); let random = g.random;
    g.move_on_board_instance(&id, next); assert_eq!(g.board(&id).unwrap().0, next); assert_eq!(g.random, random);
    assert_eq!(g.board(&id).unwrap().1.time_markers, 1); restore(&mut g);
}

#[test]
fn batch4_xg14_personal_moon_spirit_projection_and_real_damage_track_current_condition() {
    let mut g = game(); let id = field(&mut g, "XG14", 0);
    g.players[1].society_zone.secrecy = 1; assert!(!g.current_spirit(g.board(&id).unwrap().1));
    g.players[0].society_zone.secrecy = 1; g.board_mut(&id).unwrap().exhausted = true;
    assert!(g.current_spirit(g.board(&id).unwrap().1));
    for seat in 0..4 { let v = g.view(seat); assert_eq!(v.regions[0].characters.iter().find(|c| c.instance_id == id).unwrap().current_spirit_protection, Some(true)); }
    g.damage([(id.clone(), 1)].into_iter().collect()).unwrap(); assert_eq!(g.board(&id).unwrap().1.damage, 0);
    restore(&mut g); g.players[0].society_zone.secrecy = 0;
    for seat in 0..4 { let v = g.view(seat); assert_eq!(v.regions[0].characters.iter().find(|c| c.instance_id == id).unwrap().current_spirit_protection, None); }
    g.damage([(id.clone(), 1)].into_iter().collect()).unwrap(); assert!(g.board(&id).is_none()); restore(&mut g);
}

#[test]
fn batch4_xg14_real_disaster_spell_yields_optional_frozen_region_reward_above_the_spell() {
    let mut g = game(); let source = field(&mut g, "XG14", 0); g.regions[1].destroyed = true;
    disaster(&mut g); pending(&mut g); assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 1);
    choose(&mut g, vec!["region:1".into()]); assert_eq!(g.stack.len(), 2);
    g.remove_dead(&source, RemovalCause::Destroy); let replacement = field(&mut g, "XG14", 2);
    restore(&mut g); top(&mut g); assert_eq!(g.regions[1].influence, [1, 0]);
    assert_eq!(g.board(&replacement).unwrap().1.controller, 2); assert_eq!(g.stack.len(), 1);
    top(&mut g); restore(&mut g);
}

#[test]
fn batch4_xg14_changed_region_identity_or_destroyed_condition_cancels_without_retargeting() {
    for replacement in [false, true] {
        let mut g = game(); field(&mut g, "XG14", 0); g.regions[1].destroyed = true;
        disaster(&mut g); pending(&mut g); choose(&mut g, vec!["region:1".into()]);
        if replacement { g.regions[1].card = g.make_card("DQJC115", 0); } else { g.regions[1].destroyed = false; }
        restore(&mut g); top(&mut g); assert_eq!(g.regions[1].influence, [0, 0]); restore(&mut g);
    }
}

#[test]
fn batch4_xg14_duplicate_or_forged_observation_and_polluted_choice_are_rejected() {
    let mut g = game(); field(&mut g, "XG14", 0); g.regions[1].destroyed = true;
    disaster(&mut g); pending(&mut g);
    let p = g.pending.clone().unwrap(); let ChoiceResolution::Declare { declaration: d, .. } = &p.resolution else { panic!() };
    let mut bad = g.clone(); bad.effects.push_back(Effect::Declare { declaration: d.clone() });
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut bad = g.clone(); if let ChoiceResolution::Declare { declaration: d, .. } = &mut bad.pending.as_mut().unwrap().resolution {
        if let Op::XG14DisasterInfluence { observation: Some(o) } = &mut d.ability.ops[0] { o.played.card.definition = "JC125".into(); }
    }
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    reject(&mut g, p.seat, Action { choice_id: Some(p.choice.id), selected: Some(vec!["region:1".into()]), target_id: Some("region:0".into()), ..Action::new("choose") });
}

#[test]
fn batch4_xg14_paid_deploy_cannot_forge_a_region_or_repeat_frame_and_existing_law_response_still_applies() {
    let mut g = game(); fund(&mut g, 0, "XG14", 4); deploy(&mut g);
    let mut bad = g.clone(); bad.stack.last_mut().unwrap().frame.as_mut().unwrap().chosen_region = Some(0);
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut bad = g.clone(); bad.stack.push(bad.stack[0].clone());
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut expected = g.clone(); let r = expected.chaos_region().unwrap(); let law = g.make_card("JC064", 2);
    g.attachments.push(Attachment { card: law, host_id: g.regions[r].card.id.clone() });
    top(&mut g); assert_eq!(g.random, expected.random); assert_eq!(g.resources(0), 0);
    assert!(g.regions.iter().all(|r| r.cards.iter().all(|c| c.definition != "XG14")));
    assert!(g.players[0].graveyard.iter().any(|c| c.definition == "XG14")); restore(&mut g);
}

#[test]
fn batch4_xg14_actual_disaster_character_play_observes_only_the_personal_existing_source() {
    for actor in [0, 1] {
        let mut g = game(); let sources = [field(&mut g, "XG14", 0), field(&mut g, "XG14", 1)];
        g.regions[1].destroyed = true; g.begin_window(Window::Action(g.team(actor)));
        fund(&mut g, actor, "XG14", 3); let played = held(&mut g, "XG14", actor);
        act(&mut g, actor, Action { card_id: Some(played), ..Action::new("deploy") });
        pending(&mut g);
        let ChoiceResolution::Declare { declaration: d, .. } = &g.pending.as_ref().unwrap().resolution else { panic!() };
        assert_eq!(d.actor, actor); assert_eq!(d.source.card.id, sources[actor]);
        assert!(g.effects.iter().all(|e| !matches!(e, Effect::Declare { declaration: d } if d.ability.event == Some(Event::DisasterPlayed))));
        choose(&mut g, vec!["region:1".into()]); top(&mut g);
        assert_eq!(g.regions[1].influence[g.team(actor)], 1); top(&mut g); restore(&mut g);
        assert!(g.pending.is_none());
    }
}

#[test]
fn batch4_xg14_first_self_play_and_non_disaster_play_do_not_trigger_a_disaster_reward() {
    let mut g = game(); g.regions[1].destroyed = true; fund(&mut g, 0, "XG14", 3);
    deploy(&mut g); top(&mut g); restore(&mut g);
    assert!(g.effects.iter().all(|e| !matches!(e, Effect::Declare { declaration: d } if d.ability.event == Some(Event::DisasterPlayed))));
    let id = held(&mut g, "JC125", 0); fund(&mut g, 0, "JC125", 4);
    g.begin_window(Window::Action(0));
    act(&mut g, 0, Action { card_id: Some(id), region: Some(0), ..Action::new("deploy") });
    top(&mut g); assert!(g.pending.is_none()); assert_eq!(g.regions[1].influence, [0, 0]); restore(&mut g);
}

#[test]
fn batch4_xg14_authenticated_cross_card_glory_remains_resumable() {
    let mut g = game(); let id = field(&mut g, "XG14", 0);
    // A disclosed bonus keeps the one-defense host alive under the curse.
    g.turn_attribute_modifiers.push(TurnAttributeModifier {
        target_instance: id.clone(), defense_bonus: 2, kill_bonus: 0, grants_retreat: false,
        printed_defense_override: None, grants_barrier: false, prevents_damage: false,
        ordinary_icons: Icons::default(), grants_renown: false, expires_turn: g.turn,
    });
    let curse = g.make_card("JC089", 2); g.attachments.push(Attachment { card: curse, host_id: id });
    let declaration = g.jc089_combat_glory(0, 0).unwrap();
    g.effects.push_back(Effect::Declare { declaration }); g.drive().unwrap(); restore(&mut g);
    choose(&mut g, vec!["accept".into()]); top(&mut g);
    assert_eq!(g.regions[0].influence[0], 1); restore(&mut g);
}
