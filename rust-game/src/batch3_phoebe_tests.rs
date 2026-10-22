use crate::{batch3_phoebe::*, catalog, model::*, room::*, rules::*, unit_support::*};

fn fill(g: &mut Game) { g.players[0].deck = (0..6).map(|_| g.make_card("JC125", 0)).collect(); }
fn pending(g: &mut Game) {
    for _ in 0..64 {
        if g.pending.is_some() { restore(g); return; }
        if !g.effects.is_empty() { g.drive().unwrap(); } else { pass(g); }
    }
    panic!("bounded Phoebe trigger");
}
fn spell(g: &mut Game, actor: usize, target: &str) {
    g.begin_window(Window::Action(g.team(actor)));
    fund(g, actor, "JC078", 1);
    let card = held(g, "JC078", actor);
    act(g, actor, Action { card_id: Some(card), target_id: Some(target.into()), ..Action::new("play") });
}
fn settle(g: &mut Game) -> usize {
    let mut targeted = 0;
    for _ in 0..160 {
        if let Some(p) = &g.pending {
            if matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.ability.key == TARGETED) { targeted += 1; }
            resolve_choice(g); // The base helper declines optional triggers.
        } else if !g.effects.is_empty() { g.drive().unwrap(); }
        else if !g.stack.is_empty() { pass(g); }
        else { return targeted; }
    }
    panic!("bounded spell settlement");
}

#[test]
fn batch3_cq07_exact_unique_print_and_whole_program_cannot_be_rewritten_or_transplanted() {
    let c = catalog::card("CQ07");
    assert_eq!((c.name.as_str(), c.subtitle.as_deref(), c.cost, c.defense, c.unique), ("菲碧·π", Some("巡查使"), 3, Some(1), true));
    assert_eq!(c.subtypes, ["吸血鬼", "法师"]);
    assert_eq!(c.loyalty, ["蓝色"]);
    assert_eq!(c.magic_icon, MagicIcon::Blood);
    assert_eq!(c.permanent_icons, Icons::default());
    assert_eq!(c.temporary_icons, Icons { investigation: 1, influence: 1, ..Icons::default() });
    assert!(c.supported); // The complete twenty-card local candidate is admitted together.
    let d = crate::batch3_phoebe::definition();
    assert!(crate::batch3::validate_definition("CQ07", &d).is_ok());
    assert!(crate::batch3::validate_definition("JC125", &d).is_err());
    let mut changed = d.abilities[1].clone(); changed.costs.push(Cost::Assets(1));
    assert!(crate::batch3::validate_ability("CQ07", &changed).is_err());
    changed = d.abilities[1].clone(); changed.event = Some(Event::Enter);
    assert!(crate::batch3::validate_ability("CQ07", &changed).is_err());
}

#[test]
fn batch3_cq07_real_paid_entry_has_one_personal_moon_or_cancels_one_sun() {
    for (moon, sun, expected_moon, expected_sun) in [(0, 0, 1, 0), (2, 0, 2, 0), (0, 2, 0, 1)] {
        let mut g = game();
        g.players[0].society_zone.secrecy = moon;
        g.players[0].society_zone.exposure = sun;
        g.players[1].society_zone.secrecy = 2;
        fund(&mut g, 0, "JC029", 3);
        let c = held(&mut g, "CQ07", 0);
        act(&mut g, 0, Action { card_id: Some(c), region: Some(0), ..Action::new("deploy") });
        pending(&mut g); choose(&mut g, vec!["accept".into()]); top(&mut g);
        assert_eq!(g.resources(0), 0);
        assert_eq!((g.players[0].society_zone.secrecy, g.players[0].society_zone.exposure), (expected_moon, expected_sun));
        assert_eq!(g.players[1].society_zone.secrecy, 2);
        let c = g.regions[0].cards.iter().find(|c| c.definition == "CQ07").unwrap();
        assert_eq!(g.current_icons(c, 0), Icons { investigation: 1, influence: 1, ..Icons::default() });
        restore(&mut g);
    }
}

#[test]
fn batch3_cq07_entry_condition_rechecks_at_resolution_and_optional_decline_does_nothing() {
    for accept in [false, true] {
        let mut g = game(); let id = field(&mut g, "CQ07", 0);
        g.enter_triggers(0, "CQ07", &id, false); pending(&mut g);
        choose(&mut g, if accept { vec!["accept".into()] } else { vec![] });
        if accept { g.players[0].society_zone.secrecy = 2; top(&mut g); }
        assert_eq!(g.players[0].society_zone.secrecy, if accept { 2 } else { 0 });
        restore(&mut g);
    }
}

#[test]
fn batch3_cq07_enemy_spell_targeting_self_with_both_types_draws_once_above_the_paid_spell() {
    let mut g = game(); fill(&mut g); let source = field(&mut g, "CQ07", 0);
    g.board_mut(&source).unwrap().exhausted = true;
    spell(&mut g, 2, &source); pending(&mut g);
    assert_eq!(g.players[0].hand.len(), 0); assert_eq!(g.resources(2), 0);
    assert!(matches!(&g.pending.as_ref().unwrap().resolution, ChoiceResolution::Declare { declaration: d, .. } if d.ability.key == TARGETED));
    choose(&mut g, vec!["accept".into()]); assert_eq!(g.stack.len(), 2);
    top(&mut g); assert_eq!(g.players[0].hand.len(), 1); assert_eq!(g.stack.len(), 1);
    top(&mut g); assert!(g.damage_prevented(g.board(&source).unwrap().1));
    assert_eq!(g.players[0].hand.len(), 1); restore(&mut g);
}

#[test]
fn batch3_cq07_personal_target_enemy_spell_and_visible_subtype_are_each_required() {
    for (actor, target_seat, print, hide_source, expected) in [
        (2, 0, "JC003", false, 1), (2, 1, "JC003", false, 0),
        (1, 0, "JC003", false, 0), (0, 0, "JC003", false, 0),
        (2, 0, "JC125", false, 0), (2, 0, "JC003", true, 0),
    ] {
        let mut g = game(); fill(&mut g); let source = field(&mut g, "CQ07", 0);
        if hide_source { g.hide_in_place_successor(&source); }
        let target = field(&mut g, print, target_seat);
        spell(&mut g, actor, &target);
        assert_eq!(settle(&mut g), expected);
        assert_eq!(g.players[0].hand.len(), 0); // Optional rewards explicitly declined.
        restore(&mut g);
    }
}

#[test]
fn batch3_cq07_character_ability_and_region_damage_do_not_count_as_targeted_enemy_spells() {
    let mut g = game(); fill(&mut g); field(&mut g, "CQ07", 0); let target = field(&mut g, "JC003", 0);
    let hypnotist = field(&mut g, "JC003", 2);
    g.begin_window(Window::Action(1)); fund(&mut g, 2, "JC125", 2);
    act(&mut g, 2, Action { card_id: Some(hypnotist), target_id: Some(target.clone()), ..Action::new("activate") });
    assert_eq!(settle(&mut g), 0); assert!(g.board(&target).unwrap().1.exhausted);
    fund(&mut g, 2, "JC047", 3);
    let card = held(&mut g, "JC047", 2);
    act(&mut g, 2, Action { card_id: Some(card), region: Some(0), ..Action::new("play") });
    assert_eq!(settle(&mut g), 0); restore(&mut g);
}

#[test]
fn batch3_cq07_frozen_trigger_draws_for_original_actor_after_source_and_target_leave() {
    let mut g = game(); fill(&mut g); let source = field(&mut g, "CQ07", 0); let target = field(&mut g, "JC003", 0);
    spell(&mut g, 2, &target); pending(&mut g); choose(&mut g, vec!["accept".into()]);
    g.remove_dead(&source, RemovalCause::Destroy); g.return_hand(&target);
    let before = g.players[0].hand.len(); restore(&mut g); top(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1); assert_eq!(g.players[2].hand.len(), 0);
    top(&mut g); assert_eq!(g.players[0].hand.len(), before + 1); restore(&mut g);
}

#[test]
fn batch3_cq07_duplicate_forged_observations_and_pending_payloads_are_rejected() {
    let mut g = game(); fill(&mut g); field(&mut g, "CQ07", 0); let target = field(&mut g, "JC003", 0);
    spell(&mut g, 2, &target); pending(&mut g);
    let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution else { panic!() };
    let mut bad = g.clone(); bad.effects.push_back(Effect::Declare { declaration: declaration.clone() });
    assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    for change in 0..4 {
        let mut bad = g.clone();
        if let ChoiceResolution::Declare { declaration: d, stage } = &mut bad.pending.as_mut().unwrap().resolution {
            if change == 3 { *stage = DeclareChoice::Target; }
            if let Op::CQ07SpellTargeted { observation: Some(o) } = &mut d.ability.ops[0] {
                match change { 0 => o.spell.card.controller = 0, 1 => o.target.card.controller = 1,
                    2 => o.event_id = o.target.card.id.clone(), _ => {} }
            }
        }
        assert!(Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err());
    }
    let p = g.pending.clone().unwrap();
    reject(&mut g, p.seat, Action { choice_id: Some(p.choice.id), selected: Some(vec!["accept".into()]), target_id: Some("other".into()), ..Action::new("choose") });
    restore(&mut g);
}

#[test]
fn batch3_cq07_actual_control_conversion_uses_current_vampire_type_and_not_printed_human() {
    let mut g = game(); fill(&mut g); field(&mut g, "CQ07", 0);
    let target = field(&mut g, "JC125", 2); fund(&mut g, 0, "JC029", 5);
    cast(&mut g, "JC036", 0, &target);
    assert_eq!(g.players[0].hand.len(), 0);
    let c = g.board(&target).unwrap().1;
    assert_eq!(c.controller, 0); assert!(g.current_subtypes(c).iter().any(|t| t == "吸血鬼"));
    spell(&mut g, 2, &target); pending(&mut g); choose(&mut g, vec!["accept".into()]);
    top(&mut g); assert_eq!(g.players[0].hand.len(), 1); top(&mut g); restore(&mut g);
}

#[test]
fn batch3_cq07_actual_hidden_inspection_never_uses_the_hidden_front_to_award_a_draw() {
    let mut g = game(); fill(&mut g); field(&mut g, "CQ07", 0);
    let target = field(&mut g, "JC003", 0); let hidden = g.hide_in_place_successor(&target).unwrap().id;
    g.begin_window(Window::Action(1)); fund(&mut g, 2, "JC057", 1); let card = held(&mut g, "JC062", 2);
    act(&mut g, 2, Action { card_id: Some(card), selected: Some(vec![hidden.clone()]), ..Action::new("play") });
    assert_eq!(settle(&mut g), 0); assert_eq!(g.players[0].hand.len(), 0);
    assert!(g.board(&hidden).unwrap().1.face_down);
    for seat in [1, 3] {
        let v = g.view(seat);
        let c = v.regions[0].characters.iter().find(|c| c.instance_id == hidden).unwrap();
        assert!(c.card_id.is_none());
    }
    restore(&mut g);
}

#[test]
fn batch3_twenty_candidate_admits_all_twenty_and_real_deck_save_excludes_deferred_cards() {
    assert_eq!(crate::batch3::IMPLEMENTING.len(), 20);
    for id in crate::batch3::IMPLEMENTING { assert!(catalog::card(id).supported, "{id}"); }
    assert!(!catalog::catalog().cards.iter().any(|c| matches!(c.id.as_str(), "CQ11" | "TK008")));
    let mut cards = catalog::catalog().decks[0].cards.clone();
    let position = cards.iter().position(|entry| entry.card_id != "CQ07").unwrap();
    cards[position].count -= 1;
    cards.retain(|entry| entry.count > 0);
    cards.push(catalog::DeckEntry { card_id: "CQ07".into(), count: 1 });
    let draft = crate::deck::DeckDraft {
        id: "batch3-twenty".into(), name: "Current twenty candidate".into(), description: "Real deck validation".into(),
        society_id: None, cards, rules_version: catalog::RULES_VERSION.into(), card_pool_version: catalog::POOL_VERSION.into(),
        engine_version: catalog::ENGINE_VERSION.into(), updated_at: "0".into(),
    };
    let saved = crate::deck::validate(draft).unwrap();
    assert!(saved.cards.iter().any(|c| c.card_id == "CQ07" && c.count == 1));
    let mut deferred = saved.clone(); deferred.cards.push(catalog::DeckEntry { card_id: "CQ11".into(), count: 1 });
    assert!(crate::deck::validate(deferred).is_err());
}

#[test]
fn batch3_cq07_native_room_witness() {
    let mut g = game(); g.seed = u64::MAX - 190; g.random = g.seed; fill(&mut g);
    field(&mut g, "CQ07", 0); let target = field(&mut g, "JC003", 0);
    g.begin_window(Window::Action(1)); fund(&mut g, 2, "JC078", 1); let card = held(&mut g, "JC078", 2);
    let mut room = crate::jc029_tests::envelope(&g);
    let initial = serde_json::json!({ "state": serde_json::to_string(&room).unwrap(), "views": (0..4).map(|s| room.view(s, 0)).collect::<Vec<_>>() });
    let mut steps = vec![];
    crate::batch3_abi94_tests::step(&mut room, 2, SessionAction::Game { action: Action { card_id: Some(card), target_id: Some(target.clone()), ..Action::new("play") } }, &mut steps);
    for _ in 0..64 {
        if let Some(p) = room.game.pending.clone() {
            crate::batch3_abi94_tests::step(&mut room, p.seat, SessionAction::Game { action: Action { choice_id: Some(p.choice.id), selected: Some(vec!["accept".into()]), ..Action::new("choose") } }, &mut steps);
        } else if let Some(w) = room.pacing.window.clone() {
            let seat = *w.members.iter().find(|(_, d)| matches!(d, Decision::Undecided { .. })).unwrap().0;
            crate::batch3_abi94_tests::step(&mut room, seat, SessionAction::PassResponse { window_id: w.id }, &mut steps);
        } else { break; }
    }
    assert!(room.game.pending.is_none() && room.game.effects.is_empty() && room.game.stack.is_empty());
    assert_eq!(room.game.players[0].hand.len(), 1); assert!(room.game.damage_prevented(room.game.board(&target).unwrap().1));
    if let Ok(path) = std::env::var("BATCH3_CQ07_SMOKE_OUT") {
        std::fs::write(path, serde_json::to_vec(&serde_json::json!({
            "format": "batch3-native94-integration-smoke-v1", "method": "Explicit layout; actual paid enemy spell, optional CQ07 reward, Room responses, replay and four views; not natural browser acceptance",
            "catalog": catalog::catalog(), "initial": initial, "steps": steps, "seedDecimal": g.seed.to_string(),
        })).unwrap()).unwrap();
    }
}
