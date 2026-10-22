//! Explicit native layouts; not WASM, HTTP scheduling or browser acceptance.
use crate::{catalog, model::*, rules::*, unit_support::*};
const KEY: &str = crate::batch4_phantom::KEY;
const ADVANCE: &str = crate::batch4_plans::ADVANCE;
fn checkpoint(g: &mut Game) { restore(g); crate::jc029_tests::checkpoint(g); }
fn plan(secret: bool, n: u32) -> (Game, String) {
    let mut g = game(); fund(&mut g, 0, "WM046", 12);
    if secret { g.add_society_markers(0, 0, 1); }
    let hand = held(&mut g, "XG21", 0);
    act(&mut g, 0, Action { card_id: Some(hand), target_id: Some("society:p0".into()),
        option: secret.then(|| "secret-plan".into()), ..Action::new("play") });
    checkpoint(&mut g); top(&mut g);
    let id = g.attachments.iter().find(|a| a.card.definition == "XG21").unwrap().card.id.clone();
    for _ in 0..n {
        act(&mut g, 0, Action { card_id: Some(id.clone()), ability_id: Some(ADVANCE.into()), ..Action::new("activate") });
        top(&mut g);
    }
    checkpoint(&mut g); (g, id)
}
fn library(g: &mut Game, defs: &[&str]) -> Vec<String> {
    g.players[0].deck.clear(); let mut ids = vec![];
    for def in defs { let c = g.make_card(def, 0); ids.push(c.id.clone()); g.players[0].deck.push(c); }
    ids
}
fn activate(g: &mut Game, id: &str) {
    act(g, 0, Action { card_id: Some(id.into()), ability_id: Some(KEY.into()), ..Action::new("activate") });
    checkpoint(g);
}
fn search(g: &mut Game) {
    for _ in 0..64 {
        if let Some(p) = &g.pending { assert_eq!(p.choice.kind, "xg21-renown-search"); checkpoint(g); return; }
        pass(g);
    }
    panic!("bounded search");
}
// The envelope helper validates before returning, so corrupt both envelopes
// from the last valid shell rather than trying to construct an invalid shell.
fn invalid_both(valid: &Game, bad: &Game) {
    assert!(Game::from_persisted(&serde_json::to_string(bad).unwrap()).is_err());
    let mut room = crate::jc029_tests::envelope(valid); room.game = bad.clone();
    assert!(crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).is_err());
}

#[test]
fn batch4_phantom_no_completion_threshold_secret_and_faceup_advance_through_two_three_five() {
    for secret in [false, true] {
        let (mut g, id) = plan(secret, 6);
        let c = &g.attachments.iter().find(|a| a.card.id == id).unwrap().card;
        assert_eq!(c.advancement_markers, 6); assert_eq!(c.face_down, secret);
        assert_eq!(crate::batch4_plans::threshold("XG21"), None);
        assert!(g.effects.is_empty() && g.stack.is_empty() && g.pending.is_none());
        for s in 0..4 {
            let v = g.view(s); let c = &v.attachments.iter().find(|a| a.card.instance_id == id).unwrap().card;
            assert_eq!(c.advancement_markers, Some(6));
            if secret && s != 0 { assert_eq!(c.name, "秘密计划"); assert!(c.card_id.is_none() && c.text.is_none()); }
        }
        assert_eq!(g.resources(0), 5); checkpoint(&mut g);
    }
}

#[test]
fn batch4_phantom_printed_strict_cost_boundaries_zero_one_two_three_four_and_real_sacrifice() {
    for n in [0, 1, 2, 3, 4] {
        let (mut g, id) = plan(false, n);
        let ids = library(&mut g, &["JC070", "JC076", "JC125", "WM046", "JC005"]);
        let resources = g.resources(0); activate(&mut g, &id);
        assert_eq!(g.resources(0), resources); assert!(g.in_play(&id).is_none());
        let f = g.stack.last().unwrap().frame.as_ref().unwrap();
        assert_eq!(f.source.card.advancement_markers, n);
        assert!(matches!(f.already_paid.as_slice(), [PaidCost::Sacrificed { old_instance, controller: 0, owner: 0 }] if *old_instance == id));
        assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition == "XG21").count(), 1);
        assert!(g.players[0].graveyard.iter().all(|c| c.id != id && c.advancement_markers == 0));
        search(&mut g);
        let options = g.pending.as_ref().unwrap().choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>();
        let expected = ids.iter().take(2).enumerate().filter(|(i, _)| [1, 3][*i] < n).map(|(_, id)| id.clone()).collect::<Vec<_>>();
        assert_eq!(options, expected);
        for s in 1..4 { assert!(g.view(s).pending_choice.is_none()); }
        let before = g.random;
        if expected.is_empty() { choose(&mut g, vec![]); } else {
            let choice_id = g.pending.as_ref().unwrap().choice.id.clone();
            reject(&mut g, 0, Action { choice_id: Some(choice_id), selected: Some(vec![]), ..Action::new("choose") });
            choose(&mut g, vec![expected[0].clone()]);
            let c = g.regions[0].cards.iter().find(|c| c.definition == "JC070").unwrap();
            assert_ne!(c.id, ids[0]); assert_eq!((c.controller, c.face_down, c.advancement_markers), (0, false, 0));
            assert!(g.regions.iter().skip(1).all(|r| r.cards.is_empty()));
        }
        assert_ne!(g.random, before); checkpoint(&mut g);
    }
}

#[test]
fn batch4_phantom_empty_library_private_choice_is_completable_without_draw_or_elimination() {
    let (mut g, id) = plan(false, 4); library(&mut g, &[]); activate(&mut g, &id); search(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.min, Some(0));
    assert_eq!(g.pending.as_ref().unwrap().choice.max, Some(0));
    choose(&mut g, vec![]); assert!(!g.players[0].eliminated); assert!(g.pending.is_none()); checkpoint(&mut g);
}

#[test]
fn batch4_phantom_battlefield_renown_denial_does_not_remove_printed_library_candidates() {
    let (mut g, id) = plan(false, 4); field(&mut g, "WM041", 2);
    let ids = library(&mut g, &["JC070", "JC076", "JC125"]);
    // Library orientation is private storage, not an on-board concealment.
    g.players[0].deck[0].face_down = true;
    activate(&mut g, &id); search(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.iter().map(|o| o.id.clone()).collect::<Vec<_>>(), ids[..2]);
    choose(&mut g, vec![ids[0].clone()]); assert!(g.regions[0].cards.iter().any(|c| c.definition == "JC070" && !c.face_down));
    checkpoint(&mut g);
}

#[test]
fn batch4_phantom_response_replacement_plan_does_not_rebind_x_and_private_queries_are_read_only() {
    let (mut g, id) = plan(false, 2); let ids = library(&mut g, &["JC070", "JC076", "JC125"]);
    activate(&mut g, &id);
    let mut replacement = g.make_card("XG21", 0); replacement.advancement_markers = 9;
    let replacement_id = replacement.id.clone(); g.attachments.push(Attachment { card: replacement, host_id: "society:p0".into() });
    checkpoint(&mut g); search(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.options.len(), 1);
    let persisted = serde_json::to_string(&g).unwrap();
    for s in 0..4 { let _ = g.view(s); let _ = g.legal_actions(s); }
    assert_eq!(serde_json::to_string(&g).unwrap(), persisted);
    let old_choice = g.pending.as_ref().unwrap().choice.id.clone();
    choose(&mut g, vec![ids[0].clone()]);
    reject(&mut g, 0, Action { choice_id: Some(old_choice), selected: Some(vec![ids[0].clone()]), ..Action::new("choose") });
    assert_eq!(g.in_play(&replacement_id).unwrap().1.advancement_markers, 9); checkpoint(&mut g);
}

#[test]
fn batch4_phantom_pending_corruption_rejected_by_game_and_room() {
    let (mut g, id) = plan(false, 2); let ids = library(&mut g, &["JC070", "JC076"]); activate(&mut g, &id); search(&mut g);
    for mutation in 0..13 {
        let mut bad = g.clone(); let p = bad.pending.as_mut().unwrap();
        let ChoiceResolution::Frame { frame: f, choice } = &mut p.resolution else { panic!() };
        match mutation {
            0 => f.already_paid.clear(),
            1 => if let PaidCost::Sacrificed { old_instance, .. } = &mut f.already_paid[0] { *old_instance = "i0".into(); },
            2 => f.source.card.face_down = true,
            3 => f.steps[0].context = 1,
            4 => f.steps[0].op = Op::PlanAdvance,
            5 => f.cursor = 0,
            6 => f.chosen_region = Some(1),
            7 => *choice = FrameChoice::JZ50DeathSearch,
            8 => p.choice.options.clear(),
            9 => p.choice.min = Some(0),
            10 => p.seat = 1,
            11 => f.source.card.advancement_markers = 0,
            _ => p.choice.options[0].id = ids[1].clone(),
        }
        invalid_both(&g, &bad);
    }
    let p = g.pending.as_ref().unwrap().clone();
    reject(&mut g, 0, Action { choice_id: Some(p.choice.id), selected: Some(vec![ids[0].clone()]), region: Some(4), ..Action::new("choose") });
}

#[test]
fn batch4_phantom_program_cannot_be_transplanted_or_given_a_completion_threshold() {
    let mut defs = definitions().clone();
    defs.get_mut("JC125").unwrap().abilities.push(definition("XG21").abilities[2].clone());
    assert!(validate_definitions(&defs).is_err());
    let (mut g, id) = plan(false, 2); library(&mut g, &["JC070"]); activate(&mut g, &id);
    for mutation in 0..5 {
        let mut bad = g.clone(); let f = bad.stack.last_mut().unwrap().frame.as_mut().unwrap();
        match mutation { 0 => f.source.card.definition = "WM046".into(), 1 => f.ability_key = "batch4-plan-complete".into(),
            2 => f.steps[0].op = Op::PlanComplete, 3 => f.already_paid.clear(), _ => f.source.attachment_host_instance = Some("society:p1".into()) }
        invalid_both(&g, &bad);
    }
    assert!(!catalog::card("XG21").supported);
}

#[test]
fn batch4_phantom_secret_sacrifice_stays_closed_pending_rule_and_rejects_other_actor_payload() {
    let (mut g, id) = plan(true, 2);
    let a = Action { card_id: Some(id.clone()), ability_id: Some(KEY.into()), ..Action::new("activate") };
    reject(&mut g, 0, a.clone()); reject(&mut g, 1, a.clone());
    assert!(!g.legal_actions(0).iter().any(|l| l.action == a));
    let (mut g, id) = plan(false, 2);
    let a = Action { card_id: Some(id), ability_id: Some(KEY.into()), ..Action::new("activate") };
    reject(&mut g, 1, a.clone());
    let mut b = a.clone(); b.region = Some(0); reject(&mut g, 0, b);
    let mut b = a.clone(); b.cost_selected = Some(vec![]); reject(&mut g, 0, b);
    let mut b = a; b.option = Some("X=999".into()); reject(&mut g, 0, b);
}

#[test]
fn batch4_phantom_duplicate_sacrificed_source_with_distinct_frame_uid_is_rejected() {
    let (mut g, id) = plan(false, 2); library(&mut g, &["JC070", "JC076"]); activate(&mut g, &id);
    let mut bad = g.clone(); let mut duplicate = bad.stack.last().unwrap().clone();
    let grave_id = bad.players[0].graveyard.iter().find(|c| c.definition == "XG21").unwrap().id.clone();
    duplicate.id = grave_id.clone(); duplicate.frame.as_mut().unwrap().frame_id = grave_id;
    bad.stack.insert(0, duplicate); // Keep the true top and Room response window unchanged.
    invalid_both(&g, &bad);
}

#[test]
fn batch4_phantom_known_unsupported_conditional_library_path_rejected_before_sacrifice() {
    for def in ["XG17", "WM041"] {
        let (mut g, id) = plan(false, 7); library(&mut g, &[def, "JC070"]);
        let a = Action { card_id: Some(id.clone()), ability_id: Some(KEY.into()), ..Action::new("activate") };
        reject(&mut g, 0, a); assert!(g.in_play(&id).is_some());
        assert!(!g.players[0].graveyard.iter().any(|c| c.definition == "XG21"));
        checkpoint(&mut g);
    }
}

#[test]
fn batch4_phantom_real_search_entry_queues_original_printed_enter_trigger_without_character_payment() {
    let (mut g, id) = plan(false, 5); let ids = library(&mut g, &["XQ48", "JC125", "JC125"]);
    let resources = g.resources(0); activate(&mut g, &id); search(&mut g); choose(&mut g, vec![ids[0].clone()]);
    assert_eq!(g.resources(0), resources);
    let entrant = g.regions[0].cards.iter().find(|c| c.definition == "XQ48").unwrap().clone();
    assert_ne!(entrant.id, ids[0]); assert!(!entrant.face_down); assert_eq!(entrant.controller, 0);
    let p = g.pending.as_ref().unwrap();
    let ChoiceResolution::Declare { declaration, .. } = &p.resolution else { panic!("normal entry observation"); };
    assert_eq!(declaration.ability.key, "xq48-entry-passer-search");
    assert_eq!(declaration.source.card.id, entrant.id);
    checkpoint(&mut g); choose(&mut g, vec![]); checkpoint(&mut g);
}
