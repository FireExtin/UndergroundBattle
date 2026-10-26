//! Explicit unit layout for the existing CQ15 host grant on closed BQ001.
//! All attach, activation, payment and choice transitions use engine actions
//! and unit_support persistence roundtrips. Not a public construction proof.
use crate::{catalog, import20_swords as swords, jc029_tests::pass_top, model::*, unit_support::*};

#[test]
fn bq001_shared_grant_sword_payment_private_search_and_continuation_use_original_host() {
    assert!(!catalog::card("BQ001").supported);
    let mut g = game();
    let host = field(&mut g, "BQ001", 0);
    fund(&mut g, 0, "LC16", 6);
    let sword = held(&mut g, "CQ15", 0);
    act(
        &mut g,
        0,
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
    let target = field(&mut g, "JC125", 2);
    let enemy_hand = held(&mut g, "JC125", 2);
    g.begin_window(Window::Action(g.team(0)));
    let activation = Action {
        card_id: Some(host.clone()),
        ability_id: Some(format!("{}{}", swords::PREFIX, sword)),
        target_id: Some(target.clone()),
        ..Action::new("activate")
    };
    assert!(g.legal_actions(0).iter().any(|a| a.action == activation));
    act(&mut g, 0, activation);
    assert!(g.board(&host).unwrap().1.exhausted);
    assert!(g.attachments.iter().all(|a| a.card.id != sword));
    assert_eq!(
        g.players[0]
            .hand
            .iter()
            .filter(|c| c.definition == "CQ15" && c.id != sword)
            .count(),
        1
    );
    pass_top(&mut g);
    assert!(g.board(&target).is_none());
    let pending = g.pending.clone().unwrap();
    assert_eq!(pending.choice.kind, swords::SEARCH);
    assert_eq!(pending.seat, 0);
    let ChoiceResolution::Frame { frame, .. } = &pending.resolution else {
        panic!("expected shared sword frame")
    };
    assert_eq!(frame.source.card.definition, "BQ001");
    assert!(g.validated_cross_card_frame(frame).unwrap());
    assert!(pending.choice.options.iter().any(|o| o.id == enemy_hand));
    for viewer in 1..4 {
        assert!(g.view(viewer).pending_choice.is_none());
    }
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(pending.choice.id.clone()),
            selected: Some(vec![enemy_hand.clone()]),
            amount: Some(1),
            ..Action::new("choose")
        },
    );
    choose(&mut g, vec![enemy_hand.clone()]);
    assert!(g.pending.is_none());
    assert!(g.stack.is_empty());
    assert!(g.effects.is_empty());
    assert_eq!(g.sealed_cards.len(), 1);
    assert_eq!(g.sealed_cards[0].host_id, "society:p0");
    assert_eq!(g.sealed_cards[0].card.owner, 2);
    assert!(!g.players[2].hand.iter().any(|c| c.id == enemy_hand));
    let state = serde_json::to_string(&g).unwrap();
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(pending.choice.id),
            selected: Some(vec![enemy_hand]),
            ..Action::new("choose")
        },
    );
    assert_eq!(serde_json::to_string(&g).unwrap(), state);
    assert!(!catalog::card("BQ001").supported);
}

fn shared_death_layout() -> (Game, String, String, String, String) {
    let mut g = game();
    let host = field(&mut g, "BQ001", 0);
    fund(&mut g, 0, "LC16", 6);
    let sword = held(&mut g, "CQ15", 0);
    act(
        &mut g,
        0,
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
    let ghost = field(&mut g, "XG25", 2);
    let ghost_host = field(&mut g, "JC125", 3);
    let other_ghost = held(&mut g, "XG25", 2);
    // Explicit unit prerequisite only; this is not a natural moon-supply trace.
    g.players[2].society_zone.secrecy = 1;
    g.begin_window(Window::Action(g.team(0)));
    act(
        &mut g,
        0,
        Action {
            card_id: Some(host.clone()),
            ability_id: Some(format!("{}{}", swords::PREFIX, sword)),
            target_id: Some(ghost.clone()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut g);
    (g, host, ghost, ghost_host, other_ghost)
}
fn shared_death_pending(g: &Game) {
    assert!(
        matches!(&g.pending.as_ref().unwrap().resolution,
        ChoiceResolution::Declare { declaration, .. } if declaration.ability.key == crate::next20_black_wraiths::SEAL),
        "the XG25 destination must be offered before CQ15 search"
    );
    assert_eq!(g.death_replacements.len(), 1);
    assert_eq!(
        g.effects
            .iter()
            .filter(|e| matches!(e, Effect::DeathCaller { .. }))
            .count(),
        1
    );
    assert!(!g.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "XG25"));
}
fn shared_room_restore(g: &mut Game) {
    restore(g);
    let room = crate::jc029_tests::envelope(g);
    let raw = serde_json::to_string(&room).unwrap();
    let loaded = crate::room::RoomEnvelope::from_persisted(&raw).unwrap();
    assert_eq!(serde_json::to_string(&loaded).unwrap(), raw);
    for seat in 0..4 {
        assert_eq!(
            serde_json::to_value(loaded.view(seat, 0)).unwrap(),
            serde_json::to_value(room.view(seat, 0)).unwrap()
        );
    }
}
#[test]
fn bq001_shared_grant_sword_waits_for_xg25_grave_before_three_zone_search() {
    let (mut g, host, old_ghost, _, _) = shared_death_layout();
    shared_death_pending(&g);
    shared_room_restore(&mut g);
    choose(&mut g, vec![]);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, swords::SEARCH);
    assert!(g.death_replacements.is_empty());
    let grave = g.players[2]
        .graveyard
        .iter()
        .find(|c| c.definition == "XG25")
        .unwrap()
        .id
        .clone();
    assert_ne!(grave, old_ghost);
    assert!(g
        .pending
        .as_ref()
        .unwrap()
        .choice
        .options
        .iter()
        .any(|o| o.id == grave));
    assert!(g.board(&host).unwrap().1.exhausted);
    assert_eq!(
        g.players[0]
            .hand
            .iter()
            .filter(|c| c.definition == "CQ15")
            .count(),
        1
    );
    shared_room_restore(&mut g);
    let pending = g.pending.clone().unwrap();
    choose(&mut g, vec![grave.clone()]);
    assert!(
        g.death_replacements.is_empty()
            && g.effects.is_empty()
            && g.pending.is_none()
            && g.stack.is_empty()
    );
    assert!(g.players[2].graveyard.iter().all(|c| c.id != grave));
    assert_eq!(
        g.sealed_cards
            .iter()
            .filter(|s| s.card.definition == "XG25" && s.host_id == "society:p0")
            .count(),
        1
    );
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(pending.choice.id),
            selected: Some(vec![grave]),
            ..Action::new("choose")
        },
    );
    shared_room_restore(&mut g);
}
#[test]
fn bq001_shared_grant_sword_sealed_xg25_is_excluded_then_other_copy_can_be_sealed() {
    let (mut g, _, _, ghost_host, other_ghost) = shared_death_layout();
    shared_death_pending(&g);
    choose(&mut g, vec![ghost_host.clone()]);
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, swords::SEARCH);
    assert!(g.death_replacements.is_empty());
    let sealed = g
        .sealed_cards
        .iter()
        .find(|s| s.card.definition == "XG25" && s.host_id == ghost_host)
        .unwrap()
        .card
        .id
        .clone();
    let options = &g.pending.as_ref().unwrap().choice.options;
    assert!(options.iter().all(|o| o.id != sealed));
    assert!(options.iter().any(|o| o.id == other_ghost));
    shared_room_restore(&mut g);
    choose(&mut g, vec![other_ghost]);
    assert_eq!(
        g.sealed_cards
            .iter()
            .filter(|s| s.card.definition == "XG25")
            .count(),
        2
    );
    assert!(g.effects.is_empty() && g.pending.is_none() && g.stack.is_empty());
    shared_room_restore(&mut g);
}
#[test]
fn bq001_shared_grant_sword_death_wait_rejects_search_rebinding_and_duplicate_callers() {
    let (mut g, _, _, _, other_ghost) = shared_death_layout();
    shared_death_pending(&g);
    shared_room_restore(&mut g);
    let base = serde_json::to_value(&g).unwrap();
    let at = base["effects"]
        .as_array()
        .unwrap()
        .iter()
        .position(|e| e.get("DeathCaller").is_some())
        .unwrap();
    for case in 0..5 {
        let mut value = base.clone();
        let caller = &mut value["effects"][at]["DeathCaller"]["caller"];
        match case {
            0 => caller["kind"]["CQ15Search"]["search"]["controller"] = 3.into(),
            1 => caller["kind"]["CQ15Search"]["search"]["name"] = "other name".into(),
            2 => {
                caller["kind"]["CQ15Search"]["search"]["offered"] = serde_json::json!([other_ghost])
            }
            3 => caller["kind"] = "Frame".into(),
            _ => {
                let duplicate = value["effects"][at].clone();
                value["effects"].as_array_mut().unwrap().push(duplicate);
            }
        }
        assert!(Game::from_persisted(&value.to_string()).is_err());
        let mut room = serde_json::to_value(crate::jc029_tests::envelope(&g)).unwrap();
        room["game"] = value;
        assert!(crate::room::RoomEnvelope::from_persisted(&room.to_string()).is_err());
    }
}
