use crate::{batch4_feast as feast, catalog, model::*, rules::*, unit_support::*};
fn pending(g: &mut Game) { for _ in 0..96 { if g.pending.is_some() { restore(g); return; } if !g.effects.is_empty() { g.drive().unwrap(); } else { pass(g); } } panic!("bounded Feast choice"); }
fn setup() -> (Game, String, String) {
    let mut g = game(); let host = field(&mut g, "JC032", 0);
    let source = play_feast(&mut g, 0); (g, source, host)
}
fn play_feast(g: &mut Game, r: usize) -> String {
    g.begin_window(Window::Action(0)); fund(g, 0, "BQ040", 2); let id = held(g, "BQ041", 0);
    act(g, 0, Action { card_id: Some(id), region: Some(r), ..Action::new("play") }); top(g);
    g.attachments.iter().rev().find(|a| a.card.definition == "BQ041").unwrap().card.id.clone()
}
fn blood(g: &mut Game, print: &str, payer: usize, host: &str) {
    g.begin_window(Window::Action(g.team(payer))); fund(g, payer, "BQ040", 1); let id = held(g, print, payer);
    act(g, payer, Action { card_id: Some(id), target_id: Some(host.into()), ..Action::new("play") }); pending(g);
}
fn target(g: &mut Game, seat: usize) { assert_eq!(g.pending.as_ref().unwrap().choice.kind, "trigger"); choose(g, vec![player_id(seat)]); pending(g); }
fn bad(g: &Game) { assert!(Game::from_persisted(&serde_json::to_string(g).unwrap()).is_err()); }

#[test]
fn batch4_feast_exact_region_attachment_retreat_and_closed_native_definition() {
    let c = catalog::card("BQ041"); assert_eq!((c.cost, c.defense, c.supported), (2, None, false)); assert_eq!(c.loyalty, ["蓝色"]);
    assert_eq!(c.permanent_icons.influence, 1); assert_eq!(c.magic, "鲜血"); let d = feast::definition(); assert!(d.traits.retreat);
    assert_eq!(d.attachment.as_ref().unwrap().host.zone, Zone::Region); assert_eq!(d.abilities[1].event, Some(Event::BQ041BloodAttached));
    for mode in 0..4 { let mut wrong = d.clone(); match mode { 0 => wrong.traits.retreat = false, 1 => wrong.abilities[1].targets[0].relation = Relation::Any, 2 => wrong.abilities[1].ops.clear(), _ => wrong.abilities[1].costs.push(Cost::Assets(2)) }; assert!(crate::batch4::validate_definition("BQ041", &wrong).is_err()); }
    assert!(crate::rules::validate_ability("JC125", &d.abilities[1]).is_err());
    let (mut g, source, _) = setup(); assert_eq!(g.resources(0), 0); assert!(g.has_retreat(&g.attachments.iter().find(|a| a.card.id == source).unwrap().card));
    assert_eq!(g.region_attachment_cards(0).filter(|c| c.id == source).count(), 1); assert_eq!(g.icons(&g.attachments.iter().find(|a| a.card.id == source).unwrap().card, 0).influence, 1); restore(&mut g);
}

#[test]
fn batch4_feast_both_blood_prints_and_enemy_controlled_blood_observe_personal_host() {
    for print in ["XQ14", "BQ040"] { for payer in [0, 2] {
        let (mut g, _, host) = setup(); let hand = held(&mut g, "JC029", 2); fund(&mut g, 2, "JC125", 2);
        blood(&mut g, print, payer, &host); let ChoiceResolution::Declare { declaration: d, .. } = &g.pending.as_ref().unwrap().resolution else { panic!() };
        assert_eq!(d.ability.key, feast::KEY); assert_eq!(d.actor, 0); target(&mut g, 2);
        assert_eq!(g.pending.as_ref().unwrap().choice.kind, "bq041-payment"); let before = g.resources(2); choose(&mut g, vec!["pay".into()]);
        assert_eq!(g.resources(2), before - 2); assert!(g.players[2].hand.iter().any(|c| c.id == hand)); assert!(g.pending.is_none()); restore(&mut g);
    } }
}

#[test]
fn batch4_feast_unfunded_enemy_privately_discards_one_and_other_seats_see_no_hand_ids() {
    let (mut g, _, host) = setup(); let a = held(&mut g, "JC029", 2); let b = held(&mut g, "JC125", 2);
    blood(&mut g, "BQ040", 0, &host); target(&mut g, 2); let p = g.pending.clone().unwrap(); assert_eq!((p.seat, p.choice.kind.as_str()), (2, "discard"));
    for seat in [0,1,3] { let v = serde_json::to_string(&g.view(seat)).unwrap(); assert!(!v.contains(&a) && !v.contains(&b)); }
    assert_eq!(g.view(2).pending_choice.unwrap().options.len(), 2); choose(&mut g, vec![b]); assert_eq!(g.players[2].hand.len(), 1); assert_eq!(g.players[2].hand[0].id, a); assert_eq!(g.players[2].graveyard.len(), 1); restore(&mut g);
}

#[test]
fn batch4_feast_funded_enemy_can_decline_and_empty_hand_has_no_invented_discard() {
    for has_hand in [true, false] { let (mut g, _, host) = setup(); fund(&mut g, 2, "JC125", 2); if has_hand { held(&mut g, "JC029", 2); }
        blood(&mut g, "XQ14", 0, &host); target(&mut g, 2); choose(&mut g, vec!["decline".into()]); assert_eq!(g.resources(2), 2);
        if has_hand { let id = g.pending.as_ref().unwrap().choice.options[0].id.clone(); choose(&mut g, vec![id]); assert_eq!(g.players[2].graveyard.len(), 1); }
        assert!(g.players[2].hand.is_empty() && g.pending.is_none()); restore(&mut g);
    }
}

#[test]
fn batch4_feast_ally_enemy_and_other_region_hosts_do_not_qualify() {
    for seat in [1,2] { let (mut g, _, _) = setup(); let host = field(&mut g, "JC032", seat); g.begin_window(Window::Action(0)); fund(&mut g, 0, "BQ040", 1); let id = held(&mut g, "XQ14", 0);
        act(&mut g, 0, Action { card_id: Some(id), target_id: Some(host), ..Action::new("play") }); top(&mut g); assert!(g.pending.is_none()); restore(&mut g);
    }
    let (mut g, _, host) = setup(); let c = g.regions[0].cards.remove(0); g.regions[1].cards.push(c); g.begin_window(Window::Action(0)); fund(&mut g, 0, "BQ040", 1); let id = held(&mut g, "XQ14", 0);
    act(&mut g, 0, Action { card_id: Some(id), target_id: Some(host), ..Action::new("play") }); top(&mut g); assert!(g.pending.is_none()); restore(&mut g);
}

#[test]
fn batch4_feast_non_blood_attachment_and_preexisting_blood_do_not_retroactively_trigger() {
    let (mut g, _, host) = setup(); fund(&mut g, 0, "JZ39", 2); let id = held(&mut g, "JZ39", 0); let human = field(&mut g, "JC125", 0);
    act(&mut g, 0, Action { card_id: Some(id), target_id: Some(human), ..Action::new("play") }); top(&mut g); assert!(g.pending.is_none());
    g.attachments.retain(|a| a.card.definition != "BQ041"); let mut c = g.make_card("BQ040", 0); c.face_down = false; g.attachments.push(Attachment { card: c, host_id: host });
    play_feast(&mut g, 0); assert!(g.pending.is_none()); restore(&mut g);
}

#[test]
fn batch4_feast_source_host_and_region_departure_after_event_do_not_erase_frozen_demand() {
    for mode in 0..4 { let (mut g, source, host) = setup(); held(&mut g, "JC029", 2); blood(&mut g, "BQ040", 0, &host);
        choose(&mut g, vec!["p2".into()]); match mode { 0 => g.remove_dead(&source, RemovalCause::Destroy), 1 => g.remove_dead(&host, RemovalCause::Destroy), 2 => { g.regions[0].card = g.make_card("DQJC115", 0); }, _ => { let c = g.regions[0].cards.remove(0); g.regions[1].cards.push(c); } }
        restore(&mut g); pending(&mut g); assert_eq!(g.pending.as_ref().unwrap().choice.kind, "discard"); let id = g.pending.as_ref().unwrap().choice.options[0].id.clone(); choose(&mut g, vec![id]); assert!(g.players[2].hand.is_empty()); restore(&mut g);
    }
}

#[test]
fn batch4_feast_target_pay_and_discard_payloads_reject_extra_fields_and_wrong_seat() {
    let (mut g, _, host) = setup(); fund(&mut g, 2, "JC125", 2); held(&mut g, "JC029", 2); blood(&mut g, "BQ040", 0, &host);
    let p = g.pending.clone().unwrap(); reject(&mut g, 0, Action { choice_id: Some(p.choice.id), selected: Some(vec!["p2".into()]), target_id: Some("p3".into()), ..Action::new("choose") });
    target(&mut g, 2); let p = g.pending.clone().unwrap(); reject(&mut g, 0, Action { choice_id: Some(p.choice.id.clone()), selected: Some(vec!["pay".into()]), ..Action::new("choose") });
    reject(&mut g, 2, Action { choice_id: Some(p.choice.id), selected: Some(vec!["pay".into()]), option: Some("decline".into()), ..Action::new("choose") });
    choose(&mut g, vec!["decline".into()]); let p = g.pending.clone().unwrap(); let id = p.choice.options[0].id.clone(); reject(&mut g, 2, Action { choice_id: Some(p.choice.id), selected: Some(vec![id.clone()]), cost_selected: Some(vec![id.clone()]), ..Action::new("choose") }); choose(&mut g, vec![id]); assert_eq!(g.resources(2), 2);
}

#[test]
fn batch4_feast_saved_event_program_target_private_zone_and_duplicate_frames_are_rejected() {
    let (mut g, source, host) = setup(); held(&mut g, "JC029", 2); blood(&mut g, "BQ040", 0, &host);
    for mode in 0..6 { let mut wrong = g.clone(); let ChoiceResolution::Declare { declaration: d, .. } = &mut wrong.pending.as_mut().unwrap().resolution else { panic!() };
        match mode { 0 => d.ability.ops.clear(), 1 => { let Op::BQ041BloodDemand { observation: Some(o) } = &mut d.ability.ops[0] else { panic!() }; o.host.card.controller = 1; }, 2 => { let Op::BQ041BloodDemand { observation: Some(o) } = &mut d.ability.ops[0] else { panic!() }; o.attachment.card.definition = "JZ39".into(); }, 3 => d.source.attachment_host_instance = Some("i1".into()), 4 => { let i = wrong.attachments.iter().position(|a| a.card.id == source).unwrap(); let c = wrong.attachments.remove(i).card; wrong.players[0].hand.push(c); }, _ => { wrong.pending.as_mut().unwrap().choice.options.pop(); } } bad(&wrong);
    }
    choose(&mut g, vec!["p2".into()]); let mut wrong = g.clone(); wrong.stack.push(wrong.stack.last().unwrap().clone()); bad(&wrong);
    let mut wrong = g.clone(); let f = wrong.stack.last_mut().unwrap().frame.as_mut().unwrap(); f.targets[0].id = "p1".into(); f.targets[0].public.instance_id = "p1".into(); bad(&wrong);
    pending(&mut g); let mut wrong = g.clone(); wrong.pending.as_mut().unwrap().seat = 0; bad(&wrong); let mut wrong = g.clone(); wrong.pending.as_mut().unwrap().choice.options.clear(); bad(&wrong);
}

#[test]
fn batch4_feast_mixed_two_feasts_and_two_justiciars_interleave_decline_restore_and_draw_once() {
    let (mut g, first, host) = setup(); let second = play_feast(&mut g, 0); let a = field(&mut g, "BQ030", 0); let b = field(&mut g, "BQ030", 0);
    held(&mut g, "JC029", 2); held(&mut g, "JC125", 3); blood(&mut g, "BQ040", 0, &host);
    let original = g.pending.as_ref().unwrap().choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>(); assert_eq!(original.len(), 4); assert_eq!(original.iter().collect::<std::collections::BTreeSet<_>>().len(), 4);
    for (id, accept) in [(&first, true), (&a, false), (&second, true), (&b, true)] {
        if g.pending.as_ref().is_some_and(|p| p.choice.kind == "batch3-entry-order") { let choice = g.pending.as_ref().unwrap().choice.options.iter().find(|o| o.id.starts_with(&format!("{id}:"))).unwrap().id.clone(); choose(&mut g, vec![choice]); }
        let p = g.pending.as_ref().unwrap(); if p.choice.kind == "trigger" { if id == &first { choose(&mut g, vec!["p2".into()]); } else if id == &second { choose(&mut g, vec!["p3".into()]); } else { choose(&mut g, if accept { vec!["accept".into()] } else { vec![] }); } } else { panic!("unexpected {}", p.choice.kind); }
        restore(&mut g);
    }
    assert_eq!(g.stack.len(), 3); assert_eq!(g.attachment_declines.len(), 1); let hand = g.players[0].hand.len(); top(&mut g); assert_eq!(g.players[0].hand.len(), hand + 1); assert!(g.attachment_declines.is_empty());
    for _ in 0..2 { pending(&mut g); assert_eq!(g.pending.as_ref().unwrap().choice.kind, "discard"); let id = g.pending.as_ref().unwrap().choice.options[0].id.clone(); choose(&mut g, vec![id]); }
    assert!(g.players[2].hand.is_empty() && g.players[3].hand.is_empty() && g.stack.is_empty()); restore(&mut g);
}

#[test]
fn batch4_feast_all_optional_observers_may_decline_without_lingering_receipts() {
    let (mut g, _, host) = setup(); field(&mut g, "BQ030", 0); field(&mut g, "BQ030", 0); blood(&mut g, "XQ14", 0, &host);
    for _ in 0..3 { if g.pending.as_ref().is_some_and(|p| p.choice.kind == "batch3-entry-order") { let id = g.pending.as_ref().unwrap().choice.options[0].id.clone(); choose(&mut g, vec![id]); } choose(&mut g, vec![]); }
    assert!(g.pending.is_none() && g.stack.is_empty() && g.effects.is_empty() && g.attachment_declines.is_empty()); restore(&mut g);
}

#[test]
fn batch4_feast_replaced_region_cancels_only_attachment_and_keeps_actual_printed_payment() {
    let mut g = game(); fund(&mut g, 0, "BQ040", 2); let id = held(&mut g, "BQ041", 0);
    act(&mut g, 0, Action { card_id: Some(id), region: Some(0), ..Action::new("play") });
    g.regions[0].card = g.make_card("DQJC115", 0); restore(&mut g); top(&mut g);
    assert!(g.attachments.is_empty()); assert_eq!(g.resources(0), 0); assert!(g.players[0].graveyard.iter().any(|c| c.definition == "BQ041")); restore(&mut g);
}

#[test]
fn batch4_feast_retreat_returns_card_to_owner_hand_in_actual_atomic_region_return() {
    let (mut g, source, _) = setup(); let original = g.attachments.iter().find(|a| a.card.id == source).unwrap().card.clone();
    g.prepare_region_return(0).unwrap(); assert!(g.region_return.as_ref().unwrap().hand.contains(&source));
    let batch = g.region_return.as_mut().unwrap(); for seat in 0..4 { batch.orders[seat] = Some(batch.bottom[seat].clone()); }
    g.commit_region_return(0).unwrap(); assert!(g.attachments.is_empty()); assert!(g.players[0].graveyard.is_empty());
    let returned = g.players[original.owner].hand.iter().find(|c| c.definition == "BQ041").unwrap(); assert_ne!(returned.id, source); restore(&mut g);
}

#[test]
fn batch4_feast_mixed_declined_justiciar_receipts_cannot_be_omitted_duplicated_or_reused() {
    let (mut g, _, host) = setup(); let first = field(&mut g, "BQ030", 0); field(&mut g, "BQ030", 0); blood(&mut g, "XQ14", 0, &host);
    let option = g.pending.as_ref().unwrap().choice.options.iter().find(|o| o.id.starts_with(&format!("{first}:"))).unwrap().id.clone(); choose(&mut g, vec![option]); choose(&mut g, vec![]);
    assert_eq!(g.attachment_declines.len(), 1); restore(&mut g);
    for mode in 0..5 { let mut wrong = g.clone(); match mode {
        0 => wrong.attachment_declines.clear(),
        1 => wrong.attachment_declines.push(wrong.attachment_declines[0].clone()),
        2 => wrong.attachment_declines[0].turn += 1,
        3 => wrong.attachment_declines[0].event.host.card.controller = 1,
        _ => { wrong.pending = None; wrong.effects.clear(); wrong.entry_order_groups.clear(); },
    } bad(&wrong); }
    let mut wrong = g.clone(); let d = wrong.effects.iter().find(|e| matches!(e, Effect::Declare { declaration: d } if d.source.card.definition == "BQ030")).unwrap().clone(); wrong.effects.push_back(d); bad(&wrong);
}

#[test]
fn batch4_feast_unknown_observed_card_and_stolen_physical_event_id_are_rejected_without_panic() {
    let (mut g, _, host) = setup(); blood(&mut g, "BQ040", 0, &host);
    for mode in 0..3 { let mut wrong = g.clone(); let physical = wrong.regions[1].card.id.clone(); let ChoiceResolution::Declare { declaration: d, .. } = &mut wrong.pending.as_mut().unwrap().resolution else { panic!() };
        let Op::BQ041BloodDemand { observation: Some(o) } = &mut d.ability.ops[0] else { panic!() };
        match mode { 0 => o.attachment.card.definition = "unknown".into(), 1 => o.host.card.definition = "unknown".into(), _ => o.event_id = physical } bad(&wrong);
    }
}
