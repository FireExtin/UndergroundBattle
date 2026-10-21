use crate::{catalog, model::*, rules, unit_support::*};

fn grave(g: &mut Game, id: &str, owner: usize) -> String {
    let c = g.make_card(id, owner); let id = c.id.clone(); g.players[owner].graveyard.push(c); id
}
fn activate(source: &str, target: &str) -> Action {
    Action { card_id: Some(source.into()), target_id: Some(target.into()), ..Action::new("activate") }
}
fn pending(g: &mut Game) {
    for _ in 0..64 { if g.pending.is_some() { restore(g); return; } pass(g); }
    panic!("expected choice");
}
fn shelter(g: &mut Game, seat: usize) -> String {
    fund(g, seat, "JC084", 3); let held = held(g, "XG28", seat);
    act(g, seat, Action { card_id: Some(held), region: Some(0), ..Action::new("play") }); top(g);
    g.attachments.iter().find(|a| a.card.definition == "XG28").unwrap().card.id.clone()
}

#[test]
fn batch3_jz25_real_paid_entry_random_seal_is_public_blank_and_returns_to_original_owner() {
    for target in 0..4 {
        let mut g = game(); g.seed = u64::MAX - 7; g.random = g.seed;
        fund(&mut g, 0, "JC029", 3);
        let hunter = held(&mut g, "JZ25", 0);
        let a = held(&mut g, "JC125", target); let b = held(&mut g, "JC029", target);
        act(&mut g, 0, Action { card_id: Some(hunter), region: Some(0), ..Action::new("deploy") });
        pending(&mut g);
        let source = g.regions[0].cards.iter().find(|c| c.definition == "JZ25").unwrap().id.clone();
        let p = g.pending.clone().unwrap();
        assert_eq!(p.choice.options.len(), 4);
        for seat in 0..4 { let v = g.view(seat); if seat != target {
            let text = serde_json::to_string(&v).unwrap();
            assert!(!text.contains(&format!("\"instanceId\":\"{a}\"")) && !text.contains(&format!("\"instanceId\":\"{b}\"")));
        } }
        choose(&mut g, vec![format!("p{target}")]);
        let mut expected = g.clone(); let index = expected.random_below(expected.players[target].hand.len());
        let selected = expected.players[target].hand[index].clone();
        top(&mut g);
        assert_eq!(g.random, expected.random); assert_eq!(g.seed, u64::MAX - 7);
        assert_eq!(g.sealed_cards.len(), 1); let s = &g.sealed_cards[0];
        assert_ne!(s.card.id, selected.id); assert_eq!(s.card.definition, selected.definition);
        assert_eq!(s.card.owner, target); assert_eq!(s.host_id, source);
        for seat in 0..4 { let v = g.view(seat); let c = &v.sealed_cards[0].card;
            assert_eq!(c.kind, "sealed"); assert!(c.card_id.is_some()); assert!(c.icons.is_none() && c.magic.is_none()); }
        let seal_id = s.card.id.clone();
        g.remove_dead(&source, RemovalCause::Destroy); g.drive().unwrap();
        assert!(g.sealed_cards.is_empty());
        assert!(g.players[target].hand.iter().any(|c| c.definition == selected.definition && c.id != selected.id && c.id != seal_id));
        restore(&mut g);
    }
}

#[test]
fn batch3_jz25_empty_hand_and_missing_original_host_do_not_consume_rng_or_seal_on_replacement() {
    for missing in [false, true] {
        let mut g = game(); let source = field(&mut g, "JZ25", 0);
        if missing { held(&mut g, "JC125", 2); }
        g.enter_triggers(0, "JZ25", &source, false); g.drive().unwrap(); choose(&mut g, vec!["p2".into()]);
        if missing { g.remove_dead(&source, RemovalCause::Destroy); field(&mut g, "JZ25", 0); }
        let random = g.random; top(&mut g); assert_eq!(g.random, random); assert!(g.sealed_cards.is_empty()); restore(&mut g);
    }
}

#[test]
fn batch3_jz25_original_actor_and_pending_metadata_cannot_be_switched() {
    let mut g = game(); let source = field(&mut g, "JZ25", 0); held(&mut g, "JC125", 2);
    g.enter_triggers(0, "JZ25", &source, false); g.drive().unwrap();
    let p = g.pending.clone().unwrap();
    reject(&mut g, 1, Action { choice_id: Some(p.choice.id.clone()), selected: Some(vec!["p2".into()]), ..Action::new("choose") });
    reject(&mut g, 0, Action { choice_id: Some(p.choice.id), selected: Some(vec!["p2".into()]), target_id: Some("private-hand-guess".into()), ..Action::new("choose") });
    let mut bad = g.clone(); bad.pending.as_mut().unwrap().choice.options[0].id = "p99".into();
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    choose(&mut g, vec!["p2".into()]); g.board_mut(&source).unwrap().controller = 3; top(&mut g);
    assert_eq!(g.sealed_cards.len(), 1); assert_eq!(g.sealed_cards[0].card.owner, 2); restore(&mut g);
}

#[test]
fn batch3_jz25_duplicate_entry_and_private_zone_source_are_rejected() {
    let mut g = game(); let id = field(&mut g, "JZ25", 0);
    g.enter_triggers(0, "JZ25", &id, false);
    let e = g.effects.iter().find(|e| matches!(e, Effect::Declare { declaration } if declaration.source.card.definition == "JZ25")).unwrap().clone();
    let mut bad = g.clone(); bad.effects.push_back(e);
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let (_, c) = g.remove_board(&id).unwrap(); g.players[0].hand.push(c);
    assert!(Game::from_persisted(&serde_json::to_string(&g).unwrap()).is_err());
}

#[test]
fn batch3_xg28_real_paid_recovery_targets_only_own_grave_characters_and_checks_personal_moon_at_resolution() {
    for moon in [false, true] {
        let mut g = game(); let source = shelter(&mut g, 0);
        let own = grave(&mut g, "JC125", 0); let ally = grave(&mut g, "JC125", 1); let spell = grave(&mut g, "JC047", 0);
        reject(&mut g, 0, activate(&source, &ally)); reject(&mut g, 0, activate(&source, &spell));
        act(&mut g, 0, activate(&source, &own));
        assert!(g.attachments.iter().find(|a| a.card.id == source).unwrap().card.exhausted);
        assert_eq!(g.players[0].assets.iter().filter(|a| a.exhausted).count(), 3);
        g.players[if moon { 0 } else { 1 }].society_zone.secrecy = 1;
        top(&mut g);
        assert!(g.players[0].graveyard.iter().all(|c| c.id != own));
        assert!(g.players[0].hand.iter().any(|c| c.definition == "JC125" && c.id != own));
        assert_eq!(g.attachments.iter().find(|a| a.card.id == source).unwrap().card.exhausted, !moon); restore(&mut g);
    }
}

#[test]
fn batch3_xg28_lost_grave_target_keeps_cost_and_does_not_reset_but_source_departure_keeps_return() {
    for target_leaves in [false, true] {
        let mut g = game(); let source = shelter(&mut g, 0); let target = grave(&mut g, "JC125", 0);
        act(&mut g, 0, activate(&source, &target)); g.players[0].society_zone.secrecy = 1;
        if target_leaves { g.players[0].graveyard.retain(|c| c.id != target); } else { g.remove_dead(&source, RemovalCause::Destroy); }
        top(&mut g);
        assert_eq!(g.players[0].assets.iter().filter(|a| a.exhausted).count(), 3);
        if target_leaves { assert!(g.attachments.iter().find(|a| a.card.id == source).unwrap().card.exhausted); assert!(g.players[0].hand.is_empty()); }
        else { assert!(g.players[0].hand.iter().any(|c| c.definition == "JC125")); }
        restore(&mut g);
    }
}

#[test]
fn batch3_xg28_real_prepare_queue_is_optional_respondable_frozen_and_mills_with_fresh_uid() {
    for accepted in [false, true] {
        let mut g = game(); let source = shelter(&mut g, 0);
        let top_card = g.make_card("JC125", 0); let old = top_card.id.clone(); g.players[0].deck.insert(0, top_card);
        let hand = g.players[0].hand.len(); let deck = g.players[0].deck.len();
        g.turn += 1; g.begin_window(Window::Prepare); g.drive().unwrap();
        let p = g.pending.clone().unwrap(); assert_eq!(p.choice.kind, "jc035-preparation-order"); assert_eq!(p.choice.min, Some(0));
        choose(&mut g, if accepted { vec![source.clone()] } else { vec![] });
        if accepted {
            assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().ability_key, "batch3-xg28-prepare");
            g.remove_dead(&source, RemovalCause::Destroy); top(&mut g);
            assert_eq!(g.players[0].deck.len(), deck - 1);
            assert!(g.players[0].graveyard.iter().any(|c| c.definition == "JC125" && c.id != old));
        } else { assert_eq!(g.players[0].deck.len(), deck); }
        assert_eq!(g.players[0].hand.len(), hand); restore(&mut g);
    }
}

#[test]
fn batch3_xg28_prepare_mill_preserves_real_rumor_replacement_and_persisted_preparation_receipt() {
    let mut g = game(); let source = shelter(&mut g, 0); let rumor_host = field(&mut g, "JC125", 0);
    let top_card = g.make_card("WM033", 0); let old = top_card.id.clone(); g.players[0].deck.insert(0, top_card);
    g.players[0].society_zone.secrecy = 1;
    g.turn += 1; g.begin_window(Window::Prepare); g.drive().unwrap(); choose(&mut g, vec![source.clone()]);
    pending(&mut g);
    assert!(matches!(g.pending.as_ref().unwrap().resolution, ChoiceResolution::Rumor { .. }));
    choose(&mut g, vec![rumor_host.clone()]);
    assert!(g.players[0].graveyard.iter().all(|c| c.definition != "WM033"));
    assert!(g.sealed_cards.iter().any(|s| s.card.definition == "WM033" && s.card.id != old && s.host_id == rumor_host));
    let observation = g.jc035_preparation.as_ref().unwrap();
    assert!(observation.entries.iter().any(|e| e.source.card.id == source && matches!(e.decision, Some(crate::jc035::PreparationDecision::Resolved { .. }))));
    restore(&mut g);
}

#[test]
fn batch3_bq089_real_two_target_play_uses_printed_cost_and_current_personal_secrecy() {
    let mut g = game(); fund(&mut g, 0, "JC084", 2); let spell = held(&mut g, "BQ089", 0);
    let target = grave(&mut g, "WM035", 0); assert_eq!(catalog::card("WM035").cost, 5);
    let action = Action { card_id: Some(spell.clone()), target_id: Some(target.clone()), region: Some(1), ..Action::new("play") };
    reject(&mut g, 0, Action { card_id: Some(spell.clone()), ..Action::new("asset") });
    g.players[1].society_zone.secrecy = 1; reject(&mut g, 0, action.clone());
    g.players[0].society_zone.secrecy = 1;
    assert!(g.legal_actions(0).iter().any(|a| a.action.card_id.as_deref() == Some(&spell) && a.action.target_id.as_deref() == Some(&target) && a.action.region == Some(1)));
    act(&mut g, 0, action); let uid = g.regions[1].card.id.clone();
    assert_eq!(g.stack.last().unwrap().frame.as_ref().unwrap().targets[1].region_instance.as_deref(), Some(uid.as_str()));
    top(&mut g); let c = g.regions[1].cards.iter().find(|c| c.definition == "WM035").unwrap();
    assert_ne!(c.id, target); assert_eq!((c.owner, c.controller, c.face_down, c.exhausted), (0,0,false,false));
    assert!(g.players[0].graveyard.iter().any(|c| c.definition == "BQ089")); restore(&mut g);
}

#[test]
fn batch3_bq089_target_loss_secrecy_loss_or_region_replacement_cancels_without_refund() {
    for change in 0..3 {
        let mut g = game(); fund(&mut g, 0, "JC084", 2); let spell = held(&mut g, "BQ089", 0); let target = grave(&mut g, "WM035", 0);
        g.players[0].society_zone.secrecy = 1;
        act(&mut g, 0, Action { card_id: Some(spell), target_id: Some(target.clone()), region: Some(1), ..Action::new("play") });
        match change { 0 => g.players[0].graveyard.retain(|c| c.id != target), 1 => g.players[0].society_zone.secrecy = 0,
            _ => { g.regions[1].card = g.make_card("DQJC115", 0); } }
        top(&mut g); assert!(g.regions.iter().all(|r| r.cards.is_empty()));
        assert_eq!(g.players[0].assets.iter().filter(|a| a.exhausted).count(), 2);
        assert!(g.players[0].graveyard.iter().any(|c| c.definition == "BQ089")); restore(&mut g);
    }
}

#[test]
fn batch3_grave_programs_reject_transplants_changed_payments_and_premature_region_bindings() {
    for id in ["JZ25", "XG28", "BQ089"] {
        let d = crate::batch3::definition(id).unwrap();
        assert!(crate::batch3::validate_definition("JC125", &d).is_err());
        assert!(crate::batch3::validate_definition(id, &rules::Definition::default()).is_err());
    }
    let mut g = game(); let source = shelter(&mut g, 0); let target = grave(&mut g, "JC125", 0);
    act(&mut g, 0, activate(&source, &target));
    let mut bad = g.clone(); bad.stack.last_mut().unwrap().frame.as_mut().unwrap().already_paid.remove(0);
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    let mut bad = g.clone(); bad.stack.last_mut().unwrap().frame.as_mut().unwrap().source.attachment_host_instance = Some("society:p0".into());
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
}
