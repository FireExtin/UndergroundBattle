//! Offline mechanism layouts and a separate fresh-lobby lifecycle match.
//! Both use native commands; neither is browser or online-room evidence.
use crate::jc029_tests::{apply, board, checkpoint, choose, fund, game, pass_top, reject};
use crate::{catalog, model::*, rules::*};

fn export_sealed_ui_case(name: &str, g: &Game) {
    if let Ok(dir) = std::env::var("SEALING_UI_FIXTURE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let room = crate::jc029_tests::envelope(g);
        let value = serde_json::json!({"scope":"explicit-offline-native-sealing-layout", "catalog":catalog::catalog(),
            "views":(0..4).map(|seat| room.view(seat,0)).collect::<Vec<_>>()});
        std::fs::write(
            format!("{dir}/{name}.json"),
            serde_json::to_vec(&value).unwrap(),
        )
        .unwrap();
    }
}

fn held(g: &mut Game, definition: &str, owner: usize, holder: usize) -> String {
    let c = g.make_card(definition, owner);
    let id = c.id.clone();
    g.players[holder].hand.push(c);
    id
}
fn layout(actor: usize, host_owner: usize) -> (Game, String, String) {
    let mut g = game(actor);
    fund(&mut g, actor, "XQ12", 3);
    let source = board(&mut g, "XQ40", actor, 0);
    let host = board(&mut g, "LC01", host_owner, 2);
    (g, source, host)
}
fn begin_seal(g: &mut Game, actor: usize, source: &str, host: &str) {
    apply(
        g,
        actor,
        Action {
            card_id: Some(source.into()),
            target_id: Some(host.into()),
            ability_id: Some("draw-hand-seal".into()),
            ..Action::new("activate")
        },
    );
    assert!(g.board(source).unwrap().1.exhausted);
    assert_eq!(g.resources(actor), 1);
    pass_top(g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "handSeal");
}
fn start_seal(actor: usize, host_owner: usize) -> (Game, String) {
    let (mut g, source, host) = layout(actor, host_owner);
    begin_seal(&mut g, actor, &source, &host);
    (g, host)
}

#[test]
fn sealing_exact_printed_three_definitions_and_xq45_no_domain() {
    for (id, cost, loyalty, kind) in [
        ("XQ40", 2, 1, "character"),
        ("XQ41", 3, 2, "character"),
        ("XQ45", 1, 3, "spell"),
    ] {
        let c = catalog::card(id);
        assert_eq!(
            (c.cost, c.loyalty.len(), c.kind.as_str()),
            (cost, loyalty, kind)
        );
        assert!(!c.unique && c.color == "紫");
    }
    assert_eq!(catalog::card("XQ45").magic_icon, MagicIcon::None);
    assert_eq!(
        catalog::card("XQ41").magic_icon,
        MagicIcon::Other("星辰".into())
    );
    assert_eq!(
        catalog::card("XQ41").permanent_icons,
        Icons {
            investigation: 1,
            ..Icons::default()
        }
    );
    assert_eq!(
        catalog::card("XQ41").temporary_icons,
        Icons {
            influence: 1,
            ..Icons::default()
        }
    );
    assert!(definition("XQ41").traits.spirit);
}

#[test]
fn sealing_drawn_card_is_a_required_private_choice_then_public_blank_zone() {
    let (mut g, host) = start_seal(0, 2);
    let p = g.pending.clone().unwrap();
    export_sealed_ui_case("private-hand-seal", &g);
    assert_eq!((p.choice.min, p.choice.max), (Some(1), Some(1)));
    assert_eq!(p.choice.options.len(), 1);
    for viewer in 1..4 {
        let v = g.view(viewer);
        assert!(v.pending_choice.is_none());
        assert_eq!(v.waiting_choice.unwrap().kind, "handSeal");
    }
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id.clone()),
            selected: Some(vec![]),
            ..Action::new("choose")
        },
    );
    let id = p.choice.options[0].id.clone();
    choose(&mut g, vec![id.clone()]);
    assert!(g.players[0].hand.is_empty());
    assert_eq!(g.sealed_cards.len(), 1);
    let sealed = &g.sealed_cards[0];
    assert_eq!(sealed.host_id, host);
    assert_ne!(sealed.card.id, id);
    assert!(g.board(&sealed.card.id).is_none());
    for viewer in 0..4 {
        let v = g.view(viewer);
        assert_eq!(v.sealed_cards.len(), 1);
        assert_eq!(v.sealed_cards[0].card.kind, "sealed");
        assert!(v.sealed_cards[0].card.card_id.is_some());
        assert!(v.sealed_cards[0].card.icons.is_none() && v.sealed_cards[0].card.magic.is_none());
        assert!(!v
            .legal_actions
            .iter()
            .any(|a| a.action.card_id.as_deref() == Some(&sealed.card.id)
                || a.action.target_id.as_deref() == Some(&sealed.card.id)));
    }
    checkpoint(&g);
    export_sealed_ui_case("public-sealed", &g);
}

#[test]
fn sealing_xq41_freezes_hand_holder_and_trigger_survives_host_destruction() {
    let (mut g, source, host) = layout(0, 2);
    // Explicit holder != owner fixture. Ordinary actor-hand moves preserve owner.
    let id = held(&mut g, "XQ41", 3, 0);
    begin_seal(&mut g, 0, &source, &host);
    choose(&mut g, vec![id.clone()]);
    export_sealed_ui_case("sealed-trigger", &g);
    let p = g.pending.as_ref().unwrap();
    let ChoiceResolution::Declare { declaration, .. } = &p.resolution else {
        panic!()
    };
    assert_eq!((declaration.actor, declaration.source.card.owner), (0, 3));
    assert_eq!(declaration.source.card.controller, 0);
    assert_eq!(declaration.source.card.id, id);
    assert_eq!(declaration.ability.event, Some(Event::Sealed));
    choose(&mut g, vec!["accept".into()]);
    let n = g.players[0].hand.len();
    g.remove_dead(&host, RemovalCause::Destroy);
    assert!(g.sealed_cards.is_empty());
    assert!(g.players[3].hand.iter().any(|c| c.definition == "XQ41"));
    export_sealed_ui_case("owner-return", &g);
    pass_top(&mut g);
    assert_eq!(g.players[0].hand.len(), n + 1);
    checkpoint(&g);
}

// Lifecycle fixtures call the actual departure primitives. Paid declaration,
// response and choice cases below use the ordinary reducer commands separately.
fn seal_fixture(g: &mut Game, definition: &str, owner: usize, holder: usize, host: &str) -> String {
    let old = held(g, definition, owner, holder);
    g.seal_hand_card(holder, &old, host).unwrap();
    g.sealed_cards.last().unwrap().card.id.clone()
}
fn activate_seal(g: &mut Game, source: &str, host: &str) {
    apply(
        g,
        0,
        Action {
            card_id: Some(source.into()),
            target_id: Some(host.into()),
            ability_id: Some("draw-hand-seal".into()),
            ..Action::new("activate")
        },
    );
}
fn play_breakout(g: &mut Game, host: &str) {
    // Three purple, domainless assets prove the printed loyalty without giving
    // XQ45 the erroneous stars-domain requirement from old metadata.
    g.players[0].assets.clear();
    fund(g, 0, "XQ40", 3);
    let event = held(g, "XQ45", 0, 0);
    apply(
        g,
        0,
        Action {
            card_id: Some(event),
            target_id: Some(host.into()),
            ability_id: Some("destroy-sealed-host".into()),
            ..Action::new("play")
        },
    );
}

#[test]
fn sealing_sacrifice_destroy_and_actual_lethal_return_to_each_owner_without_payload_death() {
    for cause in [
        RemovalCause::Sacrifice,
        RemovalCause::Destroy,
        RemovalCause::Lethal,
    ] {
        let (mut g, _, host) = layout(0, 2);
        let sealed = seal_fixture(&mut g, "JZ31", 3, 0, &host);
        if matches!(cause, RemovalCause::Lethal) {
            g.board_mut(&host).unwrap().damage = 100;
            g.settle_deaths();
        } else {
            g.remove_dead(&host, cause);
        }
        assert!(g.board(&host).is_none() && g.sealed_cards.is_empty());
        let returned = g.players[3]
            .hand
            .iter()
            .find(|c| c.definition == "JZ31")
            .unwrap();
        assert_ne!(returned.id, sealed);
        assert_eq!((returned.owner, returned.controller), (3, 3));
        assert!(g.players[3].graveyard.is_empty());
        assert!(!g.effects.iter().any(|effect|matches!(effect,Effect::Declare{declaration} if declaration.ability.event==Some(Event::Death))));
        checkpoint(&g);
    }
}

#[test]
fn sealing_actual_paid_rescue_returns_host_and_payload_to_different_owners() {
    let (mut g, _, host) = layout(0, 0);
    seal_fixture(&mut g, "JZ31", 3, 0, &host);
    let rescuer = board(&mut g, "JC075", 0, 0);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(rescuer.clone()),
            target_id: Some(host.clone()),
            ability_id: Some("rescue".into()),
            ..Action::new("activate")
        },
    );
    assert!(g.board(&rescuer).unwrap().1.exhausted);
    assert_eq!(g.resources(0), 1);
    pass_top(&mut g);
    assert!(g.board(&host).is_none() && g.sealed_cards.is_empty());
    assert!(g.players[0].hand.iter().any(|c| c.definition == "LC01"));
    assert!(g.players[3].hand.iter().any(|c| c.definition == "JZ31"));
    assert!(g.players[0].graveyard.is_empty() && g.players[3].graveyard.is_empty());
    checkpoint(&g);
}

#[test]
fn sealing_world_hide_returns_payload_and_replaces_host_without_a_death_trigger() {
    let (mut g, _, host) = layout(0, 2);
    seal_fixture(&mut g, "JZ31", 3, 0, &host);
    g.world_hide(None, false);
    assert!(g.board(&host).is_none() && g.sealed_cards.is_empty());
    assert!(g.regions[2]
        .cards
        .iter()
        .any(|c| c.definition == "LC01" && c.face_down));
    assert!(g.players[3].hand.iter().any(|c| c.definition == "JZ31"));
    assert!(g.effects.iter().all(|e|!matches!(e,Effect::Declare{declaration} if declaration.ability.event==Some(Event::Death))));
    checkpoint(&g);
}

#[test]
fn sealing_region_win_bottom_batch_excludes_sealed_cards_and_returns_them_to_hand() {
    let (mut g, _, host) = layout(0, 2);
    let sealed = seal_fixture(&mut g, "JZ31", 3, 0, &host);
    g.prepare_region_return(2).unwrap();
    let batch = g.region_return.as_mut().unwrap();
    assert!(!batch.bottom.iter().flatten().any(|id| id == &sealed));
    batch.orders = batch.bottom.iter().cloned().map(Some).collect();
    g.commit_region_return(2).unwrap();
    assert!(g.board(&host).is_none() && g.sealed_cards.is_empty());
    assert!(g.players[2]
        .deck
        .last()
        .is_some_and(|c| c.definition == "LC01"));
    assert!(g.players[3].hand.iter().any(|c| c.definition == "JZ31"));
    assert!(g.players[3].graveyard.is_empty());
    checkpoint(&g);
}

#[test]
fn sealing_board_move_and_control_change_keep_exact_host_and_independent_payloads() {
    let (mut g, _, host) = layout(0, 2);
    let a = seal_fixture(&mut g, "LC01", 0, 0, &host);
    let b = seal_fixture(&mut g, "LC01", 3, 0, &host);
    assert_ne!(a, b);
    // The same remove_board/push path as MoveOnBoard, deliberately not leave_board.
    let (_, mut c) = g.remove_board(&host).unwrap();
    c.controller = 1;
    g.regions[1].cards.push(c);
    assert_eq!(g.sealed_cards.len(), 2);
    assert!(g.sealed_cards.iter().all(|s| s.host_id == host));
    export_sealed_ui_case("moved-carrier", &g);
    checkpoint(&g);
    g.board_mut(&host).unwrap().controller = 2;
    g.remove_dead(&host, RemovalCause::Destroy);
    assert_eq!(
        g.players[0]
            .hand
            .iter()
            .filter(|c| c.definition == "LC01")
            .count(),
        1
    );
    assert_eq!(
        g.players[3]
            .hand
            .iter()
            .filter(|c| c.definition == "LC01")
            .count(),
        1
    );
    assert!(g.sealed_cards.is_empty());
    checkpoint(&g);
}

#[test]
fn sealing_elimination_releases_foreign_payloads_and_removes_eliminated_owners() {
    let (mut g, _, host) = layout(0, 2);
    seal_fixture(&mut g, "LC01", 3, 0, &host);
    g.eliminate(2);
    assert!(g.sealed_cards.is_empty());
    assert!(g.players[3].hand.iter().any(|c| c.definition == "LC01"));
    checkpoint(&g);
    let (mut g, _, host) = layout(0, 2);
    seal_fixture(&mut g, "LC01", 1, 0, &host);
    g.eliminate(1);
    assert!(g.sealed_cards.is_empty() && g.board(&host).is_some());
    assert!(g.players[1].hand.is_empty());
    checkpoint(&g);
}

#[test]
fn sealing_simultaneous_lethal_keeps_same_printed_payload_instances_and_owners_separate() {
    let (mut g, _, a) = layout(0, 1);
    let b = board(&mut g, "LC01", 2, 1);
    let sa = seal_fixture(&mut g, "LC01", 0, 0, &a);
    let sb = seal_fixture(&mut g, "LC01", 3, 0, &b);
    assert_ne!(sa, sb);
    for host in [&a, &b] {
        g.board_mut(host).unwrap().damage = 100;
    }
    g.settle_deaths();
    assert!(g.sealed_cards.is_empty() && g.board(&a).is_none() && g.board(&b).is_none());
    for seat in [0, 3] {
        assert_eq!(
            g.players[seat]
                .hand
                .iter()
                .filter(|c| c.definition == "LC01")
                .count(),
            1
        );
    }
    checkpoint(&g);
}

#[test]
fn sealing_paid_source_departure_does_not_cancel_draw_or_original_host_binding() {
    let (mut g, source, host) = layout(0, 2);
    activate_seal(&mut g, &source, &host);
    g.remove_dead(&source, RemovalCause::Destroy);
    pass_top(&mut g);
    assert_eq!(g.pending.as_ref().unwrap().choice.kind, "handSeal");
    let id = g.pending.as_ref().unwrap().choice.options[0].id.clone();
    choose(&mut g, vec![id]);
    assert_eq!(g.sealed_cards.len(), 1);
    assert_eq!(g.sealed_cards[0].host_id, host);
    checkpoint(&g);
}

#[test]
fn sealing_missing_or_replaced_paid_target_cancels_entire_draw_without_refund() {
    for replacement in [false, true] {
        let (mut g, source, host) = layout(0, 2);
        let deck_len = g.players[0].deck.len();
        activate_seal(&mut g, &source, &host);
        g.remove_dead(&host, RemovalCause::Destroy);
        let new = if replacement {
            Some(board(&mut g, "LC01", 2, 2))
        } else {
            None
        };
        pass_top(&mut g);
        assert!(g.pending.is_none() && g.sealed_cards.is_empty());
        assert_eq!(g.players[0].deck.len(), deck_len);
        assert_eq!(g.resources(0), 1);
        assert!(g.board(&source).unwrap().1.exhausted);
        if let Some(id) = new {
            assert_ne!(id, host);
            assert!(g.board(&id).is_some());
        }
        checkpoint(&g);
    }
}

#[test]
fn sealing_choice_rejects_wrong_seat_unknown_duplicate_and_stale_ids_atomically() {
    let (mut g, host) = start_seal(0, 2);
    let p = g.pending.clone().unwrap();
    let id = p.choice.options[0].id.clone();
    for (seat, ids) in [
        (1, vec![id.clone()]),
        (0, vec!["unknown".into()]),
        (0, vec![id.clone(), id.clone()]),
    ] {
        reject(
            &mut g,
            seat,
            Action {
                choice_id: Some(p.choice.id.clone()),
                selected: Some(ids),
                ..Action::new("choose")
            },
        );
    }
    choose(&mut g, vec![id]);
    reject(
        &mut g,
        0,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![]),
            ..Action::new("choose")
        },
    );
    assert_eq!(g.sealed_cards[0].host_id, host);
}

#[test]
fn sealing_xq41_can_decline_once_and_does_not_trigger_again_when_returning_to_hand() {
    let (mut g, _, host) = layout(0, 2);
    seal_fixture(&mut g, "XQ41", 0, 0, &host);
    g.drive().unwrap();
    let before = g.players[0].deck.len();
    choose(&mut g, vec![]);
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert_eq!(g.players[0].deck.len(), before);
    g.remove_dead(&host, RemovalCause::Destroy);
    g.drive().unwrap();
    assert!(g.pending.is_none() && g.stack.is_empty());
    assert!(g.players[0].hand.iter().any(|c| c.definition == "XQ41"));
    assert_eq!(g.players[0].deck.len(), before);
    checkpoint(&g);
}

#[test]
fn sealing_xq45_may_target_an_unsealed_character_but_does_not_destroy_it() {
    let (mut g, _, host) = layout(0, 2);
    play_breakout(&mut g, &host);
    assert_eq!(g.resources(0), 2);
    pass_top(&mut g);
    assert!(g.board(&host).is_some());
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ45"));
    checkpoint(&g);
}

#[test]
fn sealing_xq45_destroys_a_sealed_host_once_and_returns_payloads_before_death() {
    let (mut g, _, host) = layout(0, 2);
    seal_fixture(&mut g, "LC01", 3, 0, &host);
    play_breakout(&mut g, &host);
    pass_top(&mut g);
    assert!(g.board(&host).is_none() && g.sealed_cards.is_empty());
    assert_eq!(
        g.players[2]
            .graveyard
            .iter()
            .filter(|c| c.definition == "LC01")
            .count(),
        1
    );
    assert_eq!(
        g.players[3]
            .hand
            .iter()
            .filter(|c| c.definition == "LC01")
            .count(),
        1
    );
    checkpoint(&g);
}

#[test]
fn sealing_xq45_checks_actual_relationship_at_resolution_and_never_retargets_a_new_instance() {
    let (mut g, _, host) = layout(0, 2);
    seal_fixture(&mut g, "LC01", 3, 0, &host);
    play_breakout(&mut g, &host);
    g.return_sealed_cards(&host);
    pass_top(&mut g);
    assert!(g.board(&host).is_some());
    let (mut g, _, host) = layout(0, 2);
    play_breakout(&mut g, &host);
    g.remove_dead(&host, RemovalCause::Destroy);
    let new = board(&mut g, "LC01", 2, 2);
    seal_fixture(&mut g, "LC01", 3, 0, &new);
    pass_top(&mut g);
    assert!(g.board(&new).is_some());
    assert_eq!(g.sealed_cards[0].host_id, new);
    checkpoint(&g);
}

#[test]
fn sealing_new_zone_rejects_invalid_saved_hosts_owners_duplicates_and_mutated_state() {
    let (mut good, _, host) = layout(0, 2);
    seal_fixture(&mut good, "LC01", 3, 0, &host);
    let json = serde_json::to_string(&good).unwrap();
    Game::from_persisted(&json).unwrap();
    for variant in 0..10 {
        let mut bad = good.clone();
        match variant {
            0 => bad.sealed_cards[0].host_id = "unknown".into(),
            1 => bad.sealed_cards[0].card.owner = 99,
            2 => bad.sealed_cards[0].card.controller = 0,
            3 => bad.sealed_cards[0].card.face_down = true,
            4 => bad.sealed_cards[0].card.exhausted = true,
            5 => bad.sealed_cards[0].card.damage = 1,
            6 => bad.sealed_cards[0].card.definition = "unknown".into(),
            7 => bad.players[3].hand.push(bad.sealed_cards[0].card.clone()),
            8 => bad.sealed_cards.push(bad.sealed_cards[0].clone()),
            _ => bad.board_mut(&host).unwrap().face_down = true,
        }
        assert!(
            Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),
            "variant {variant}"
        );
        let room = crate::room::RoomEnvelope::from_game(bad);
        assert!(
            crate::room::RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap())
                .is_err(),
            "room variant {variant}"
        );
    }
}

#[test]
fn sealing_whole_definitions_and_nested_program_transplants_are_rejected() {
    use std::collections::BTreeMap;
    for id in ["XQ40", "XQ41", "XQ45"] {
        let original = definition(id).clone();
        let mut one = BTreeMap::from([(id.to_string(), original.clone())]);
        validate_definitions(&one).unwrap();
        for variant in 0..5 {
            let mut d = original.clone();
            match variant {
                0 => d.traits.public = !original.traits.public,
                1 => d.abilities[0].key = "transplanted".into(),
                2 => d.abilities[0].costs.push(Cost::Assets(1)),
                3 => d.abilities[0].event = Some(Event::Death),
                _ => d.abilities[0].ops = vec![Op::ForEachLivingPlayer(d.abilities[0].ops.clone())],
            }
            one.insert(id.into(), d);
            assert!(validate_definitions(&one).is_err(), "{id} variant{variant}");
        }
        assert!(validate_ability("LC01", &original.abilities[0]).is_err());
        one = BTreeMap::from([("LC01".into(), original)]);
        assert!(validate_definitions(&one).is_err());
    }
    let admitted = definition("XQ40").abilities[0].clone();
    for nested in [
        Op::ForEachLivingPlayer(vec![Op::SealOneActorHandCardOnTarget]),
        Op::ForEachLivingPlayerFromActor(vec![Op::SealOneActorHandCardOnTarget]),
        Op::IfTargetExhausted {
            slot: 0,
            exhausted: Box::new(Op::SealOneActorHandCardOnTarget),
            ready: Box::new(Op::DestroyTargetIfSealed),
        },
    ] {
        let mut transplanted = admitted.clone();
        transplanted.key = "ported-sealing".into();
        transplanted.ops = vec![nested];
        assert!(validate_ability("LC01", &transplanted).is_err());
    }
    let mut modal = admitted;
    modal.ops.clear();
    modal.modes.push(Mode {
        key: "ported-mode".into(),
        label: "非法封印分支".into(),
        targets: modal.targets.clone(),
        ops: vec![Op::SealOneActorHandCardOnTarget],
    });
    assert!(validate_ability("LC01", &modal).is_err());
}

#[test]
fn sealing_private_choice_roundtrips_for_every_actor_and_rejects_invalid_bound_frames() {
    for actor in 0..4 {
        let (good, _) = start_seal(actor, (actor + 1) % 4);
        let restored = Game::from_persisted(&serde_json::to_string(&good).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(good.view(actor)).unwrap(),
            serde_json::to_value(restored.view(actor)).unwrap()
        );
        for variant in 0..9 {
            let mut bad = good.clone();
            let p = bad.pending.as_mut().unwrap();
            let ChoiceResolution::Frame {
                frame,
                choice: FrameChoice::HandSeal { seat, host_id },
            } = &mut p.resolution
            else {
                panic!()
            };
            match variant {
                0 => *seat = 99,
                1 => *host_id = "wrong-bound-host".into(),
                2 => p.choice.options[0].id = "other-hand-instance".into(),
                3 => frame.steps[0].op = Op::DestroyTargetIfSealed,
                4 => p.choice.min = Some(0),
                5 => frame.cursor = usize::MAX,
                6 => frame.actor = (actor + 1) % 4,
                7 => p.choice.player_id = format!("p{}", (actor + 1) % 4),
                _ => p.choice.options.push(p.choice.options[0].clone()),
            }
            let json = serde_json::to_string(&bad).unwrap();
            assert!(
                Game::from_persisted(&json).is_err(),
                "actor{actor} variant{variant}"
            );
            let room = crate::room::RoomEnvelope::from_game(bad);
            assert!(crate::room::RoomEnvelope::from_persisted(
                &serde_json::to_string(&room).unwrap()
            )
            .is_err());
        }
    }
}

#[test]
fn sealing_empty_draw_eliminates_actor_before_any_seal_or_private_selection() {
    let (mut g, source, host) = layout(0, 2);
    held(&mut g, "XQ41", 3, 0);
    g.players[0].deck.clear();
    activate_seal(&mut g, &source, &host);
    pass_top(&mut g);
    assert!(g.players[0].eliminated && g.players[0].hand.is_empty());
    assert!(g.sealed_cards.is_empty() && g.pending.is_none());
    assert!(g.board(&host).is_some());
    checkpoint(&g);
}

#[test]
fn sealing_two_same_printed_xq41_events_keep_fifo_original_holders_and_exact_one_draw() {
    let (mut g, _, host) = layout(0, 2);
    let first = held(&mut g, "XQ41", 3, 0);
    let second = held(&mut g, "XQ41", 1, 2);
    g.seal_hand_card(0, &first, &host).unwrap();
    g.seal_hand_card(2, &second, &host).unwrap();
    assert_ne!(g.sealed_cards[0].card.id, g.sealed_cards[1].card.id);
    // Both explicit triggers must survive the same carrier departure. Returning
    // payloads go to owners 3/1; drawing belongs to original hand holders 0/2.
    g.remove_dead(&host, RemovalCause::Destroy);
    assert!(g.sealed_cards.is_empty());
    let before = [g.players[0].deck.len(), g.players[2].deck.len()];
    g.drive().unwrap();
    for (actor, id) in [(0, first), (2, second)] {
        let ChoiceResolution::Declare { declaration, .. } = &g.pending.as_ref().unwrap().resolution
        else {
            panic!()
        };
        assert_eq!(declaration.actor, actor);
        assert_eq!(declaration.source.card.id, id);
        choose(&mut g, vec!["accept".into()]);
    }
    // Declarations are offered FIFO. Accepted effects use the existing stack,
    // so resolve both entries only after both optional declarations complete.
    while !g.stack.is_empty() {
        pass_top(&mut g);
    }
    assert_eq!(
        [g.players[0].deck.len(), g.players[2].deck.len()],
        [before[0] - 1, before[1] - 1]
    );
    assert_eq!((g.players[0].hand.len(), g.players[2].hand.len()), (1, 1));
    assert!(g.players[3].hand.iter().any(|c| c.definition == "XQ41"));
    assert!(g.players[1].hand.iter().any(|c| c.definition == "XQ41"));
    assert!(g.pending.is_none() && g.stack.is_empty());
    checkpoint(&g);
}

#[cfg(feature = "native")]
#[tokio::test]
async fn sealing_store_choice_reopen_receipt_preserves_owner_holder_and_never_reseals() {
    use crate::room::{RoomCommand, RoomEnvelope, SessionAction};
    use crate::service::{CreateRoom, JoinRoom, Store};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sealing-explicit-local-fixture.sqlite");
    let store = Store::open(&path).unwrap();
    let mut sessions = vec![store
        .create(CreateRoom {
            name: "P0".into(),
            mode: "teams".into(),
            deck_id: "watchers".into(),
            deck_draft: None,
        })
        .await
        .unwrap()];
    for seat in 1..4 {
        sessions.push(
            store
                .join(JoinRoom {
                    invite_code: sessions[0].invite_code.clone(),
                    name: format!("P{seat}"),
                    deck_id: "watchers".into(),
                    deck_draft: None,
                })
                .await
                .unwrap(),
        );
    }
    drop(store);
    let (mut g, source, host) = layout(0, 2);
    let id = held(&mut g, "XQ41", 3, 0);
    begin_seal(&mut g, 0, &source, &host);
    g.room_id = sessions[0].room_id.clone();
    g.invite_code = sessions[0].invite_code.clone();
    let mut room = crate::jc029_tests::envelope(&g);
    let db = rusqlite::Connection::open(&path).unwrap();
    // This is a disposable fixture DB, never a user/runtime database.
    db.execute(
        "UPDATE rooms SET state=?1,revision=?2 WHERE id=?3",
        rusqlite::params![
            serde_json::to_string(&room).unwrap(),
            room.revision,
            g.room_id
        ],
    )
    .unwrap();
    drop(db);
    let command = RoomCommand {
        command_id: "seal-once-after-reopen".into(),
        expected_version: room.revision,
        action: SessionAction::Game {
            action: Action {
                choice_id: Some(g.pending.as_ref().unwrap().choice.id.clone()),
                selected: Some(vec![id.clone()]),
                ..Action::new("choose")
            },
        },
    };
    let expected = room.transition(0, Some(command.clone()), 0).unwrap();
    assert!(expected.error_code.is_none());
    let store = Store::open(&path).unwrap();
    let first = store
        .command_at_now(&g.room_id, &sessions[0].token, command.clone(), 0)
        .await
        .unwrap();
    drop(store);
    room = RoomEnvelope::from_persisted(&expected.state).unwrap();
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(room.view(0, 0)).unwrap()
    );
    assert_eq!(room.game.sealed_cards.len(), 1);
    assert_eq!(room.game.sealed_cards[0].card.owner, 3);
    let ChoiceResolution::Declare { declaration, .. } =
        &room.game.pending.as_ref().unwrap().resolution
    else {
        panic!()
    };
    assert_eq!(
        (
            declaration.actor,
            declaration.source.card.owner,
            declaration.source.card.id.as_str()
        ),
        (0, 3, id.as_str())
    );
    let store = Store::open(&path).unwrap();
    let duplicate = store
        .command_at_now(&g.room_id, &sessions[0].token, command.clone(), 90_000)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&duplicate).unwrap(),
        serde_json::to_value(&first).unwrap()
    );
    let mut conflicting = command;
    conflicting.expected_version += 1;
    assert_eq!(
        store
            .command_at_now(&g.room_id, &sessions[0].token, conflicting, 90_000)
            .await
            .unwrap_err()
            .error,
        "command_id_conflict"
    );
    drop(store);
    let db = rusqlite::Connection::open(&path).unwrap();
    let stored: String = db
        .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(stored, expected.state);
    let receipts: u64 = db
        .query_row(
            "SELECT COUNT(*) FROM commands WHERE room_id=?1",
            [&g.room_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(receipts, 1);
}

#[test]
fn sealing_private_selection_trigger_and_paid_stack_pause_resume_actual_room_trace() {
    use crate::room::{RoomCommand, RoomEnvelope, SessionAction};
    let (mut g, source, host) = layout(0, 2);
    let payload = held(&mut g, "XQ41", 3, 0);
    begin_seal(&mut g, 0, &source, &host);
    let original_choice = g.pending.as_ref().unwrap().choice.id.clone();
    let mut room = crate::jc029_tests::envelope(&g);
    let initial = serde_json::to_string(&room).unwrap();
    let mut steps = vec![];
    let mut record = |room: &mut RoomEnvelope,
                      seat: usize,
                      action: Option<SessionAction>,
                      now: u64| {
        let command = action.map(|action| RoomCommand {
            command_id: format!("seal-pause-{}", steps.len()),
            expected_version: room.revision,
            action,
        });
        let transition = room.transition(seat, command.clone(), now).unwrap();
        if transition.changed {
            assert_eq!(
                serde_json::to_string(&room.replay_events(&transition.journal).unwrap()).unwrap(),
                transition.state
            );
        }
        *room = RoomEnvelope::from_persisted(&transition.state).unwrap();
        steps.push(serde_json::json!({"seat":seat,"command":command,"serverNowMs":now.to_string(),"transition":transition,
            "views":(0..4).map(|s| room.view(s,room.pacing.last_server_now_ms)).collect::<Vec<_>>() }));
    };
    let select = SessionAction::Game {
        action: Action {
            choice_id: Some(original_choice.clone()),
            selected: Some(vec![payload]),
            ..Action::new("choose")
        },
    };
    record(&mut room, 2, Some(SessionAction::PauseRoom), 1_000);
    record(&mut room, 3, None, 86_400_000);
    record(&mut room, 0, Some(select.clone()), 86_400_000);
    assert!(room.game.sealed_cards.is_empty());
    record(&mut room, 3, Some(SessionAction::ResumeRoom), 86_400_000);
    assert_eq!(
        room.game.pending.as_ref().unwrap().choice.id,
        original_choice
    );
    record(&mut room, 0, Some(select), 86_400_001);
    assert_eq!(room.game.sealed_cards.len(), 1);
    record(&mut room, 1, Some(SessionAction::PauseRoom), 86_400_002);
    record(&mut room, 2, Some(SessionAction::ResumeRoom), 172_800_000);
    let trigger = room.game.pending.as_ref().unwrap().choice.id.clone();
    record(
        &mut room,
        0,
        Some(SessionAction::Game {
            action: Action {
                choice_id: Some(trigger),
                selected: Some(vec!["accept".into()]),
                ..Action::new("choose")
            },
        }),
        172_800_001,
    );
    let hand_before = room.game.players[0].hand.len();
    let stack_id = room.game.stack[0].id.clone();
    record(&mut room, 3, Some(SessionAction::PauseRoom), 172_801_000);
    record(&mut room, 1, None, 345_600_000);
    assert_eq!(room.game.stack[0].id, stack_id);
    record(&mut room, 1, Some(SessionAction::ResumeRoom), 345_600_000);
    let mut now = 345_600_000;
    for _ in 0..8 {
        if room.game.stack.is_empty() {
            break;
        }
        now += 5_000;
        record(&mut room, 0, None, now);
    }
    assert!(room.game.stack.is_empty());
    assert_eq!(room.game.players[0].hand.len(), hand_before + 1);
    assert_eq!(room.game.sealed_cards[0].card.owner, 3);
    if let Ok(dir) = std::env::var("SEALING_ROOM_TRACE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/native-trace.json"),serde_json::to_vec(&serde_json::json!({"scope":"explicit-offline-native-sealing-private-choice-trigger-paid-stack-pause","initialState":initial,"steps":steps})).unwrap()).unwrap();
    }
}

fn sealed_game_finished_by_actual_pass_commands() -> (Game, String, String) {
    let mut g = game(0);
    fund(&mut g, 0, "XQ12", 3);
    let source = board(&mut g, "XQ40", 0, 0);
    let host = board(&mut g, "JC125", 0, 2);
    let payload = held(&mut g, "JZ31", 1, 0);
    begin_seal(&mut g, 0, &source, &host);
    choose(&mut g, vec![payload]);
    let sealed = g.sealed_cards[0].card.id.clone();
    // Explicit endgame layout: opposing decks are exhausted. Normal pass
    // commands advance to the next preparation draw, which really eliminates
    // the opposing team and calls finish. Neither status nor winner is assigned.
    g.players[2].deck.clear();
    g.players[3].deck.clear();
    g.add_control(
        &host,
        1,
        ControlLifetime::SourceLeaves {
            source_instance: source,
        },
        SubtypeChange::None,
    );
    for _ in 0..200 {
        if g.status == "finished" {
            break;
        }
        assert!(
            g.pending.is_none(),
            "unexpected endgame choice {:?}",
            g.pending.as_ref().map(|p| &p.choice.kind)
        );
        let seat = g
            .living(g.priority_team)
            .into_iter()
            .find(|s| !g.passed.contains(s))
            .unwrap();
        apply(&mut g, seat, Action::new("pass"));
    }
    assert_eq!(g.status, "finished");
    assert_eq!(g.winner_team, Some(0));
    assert!(g.players[2].eliminated && g.players[3].eliminated);
    assert_eq!(g.sealed_cards.len(), 1);
    assert_eq!(g.sealed_cards[0].host_id, host);
    assert_eq!(g.sealed_cards[0].card.id, sealed);
    assert_eq!(g.sealed_cards[0].card.owner, 1);
    Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    checkpoint(&g);
    (g, host, sealed)
}

#[test]
fn sealing_finished_restart_clears_old_seals_roundtrips_and_accepts_next_command() {
    let (mut g, host, sealed) = sealed_game_finished_by_actual_pass_commands();
    assert!(!g.control_effects.is_empty() && !g.control_baselines.is_empty());
    apply(&mut g, 0, Action::new("restart"));
    assert!(
        g.sealed_cards.is_empty(),
        "restart retained a payload bound to old host {host}"
    );
    assert!(g.board(&host).is_none());
    assert!(g.view(0).sealed_cards.is_empty());
    assert!(g.control_effects.is_empty() && g.control_baselines.is_empty());
    assert!(g.attachments.is_empty() && g.region_return.is_none());
    assert!(
        g.modifiers.is_empty()
            && g.turn_attribute_modifiers.is_empty()
            && g.turn_ability_usage.is_empty()
    );
    assert!(g.stack.is_empty() && g.winner_team.is_none());
    assert!(g
        .players
        .iter()
        .all(|p| !p.eliminated && p.score_cards.is_empty() && p.hand.len() + p.deck.len() == 50));
    assert!(!serde_json::to_string(&g)
        .unwrap()
        .contains(&format!("\"id\":\"{sealed}\"")));
    let saved = serde_json::to_string(&g).unwrap();
    let mut restored = Game::from_persisted(&saved).unwrap();
    let pending = restored.pending.clone().unwrap();
    assert_eq!(pending.choice.kind, "mulligan");
    apply(
        &mut restored,
        pending.seat,
        Action {
            choice_id: Some(pending.choice.id),
            selected: Some(vec![]),
            ..Action::new("choose")
        },
    );
    assert_eq!(restored.status, "playing");
    assert!(restored.sealed_cards.is_empty());
    checkpoint(&restored);
}

#[cfg(feature = "native")]
#[tokio::test]
async fn sealing_finished_restart_native_store_reopen_next_choice_and_original_receipt() {
    use crate::room::{RoomCommand, RoomEnvelope, SessionAction};
    use crate::service::{CreateRoom, JoinRoom, Store};
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sealed-restart-local-fixture.sqlite3");
    let store = Store::open(&path).unwrap();
    let host = store
        .create(CreateRoom {
            name: "P0".into(),
            mode: "teams".into(),
            deck_id: "watchers".into(),
            deck_draft: None,
        })
        .await
        .unwrap();
    let mut sessions = vec![host];
    for seat in 1..4 {
        sessions.push(
            store
                .join(JoinRoom {
                    invite_code: sessions[0].invite_code.clone(),
                    name: format!("P{seat}"),
                    deck_id: "watchers".into(),
                    deck_draft: None,
                })
                .await
                .unwrap(),
        );
    }
    drop(store);
    let (mut g, host_id, _) = sealed_game_finished_by_actual_pass_commands();
    g.room_id = sessions[0].room_id.clone();
    g.invite_code = sessions[0].invite_code.clone();
    let mut room = crate::jc029_tests::envelope(&g);
    let initial = serde_json::to_string(&room).unwrap();
    let db = rusqlite::Connection::open(&path).unwrap();
    // Disposable authenticated Store fixture, with an actual terminal Game;
    // no user DB mutation and no manually assigned finished status.
    db.execute(
        "UPDATE rooms SET state=?1,revision=?2 WHERE id=?3",
        rusqlite::params![initial, room.revision, g.room_id],
    )
    .unwrap();
    drop(db);
    let restart = RoomCommand {
        command_id: "sealed-new-game".into(),
        expected_version: room.revision,
        action: SessionAction::Game {
            action: Action::new("restart"),
        },
    };
    let first = room.transition(0, Some(restart.clone()), 0).unwrap();
    assert!(first.error_code.is_none());
    room = RoomEnvelope::from_persisted(&first.state).unwrap();
    assert!(room.game.sealed_cards.is_empty() && room.game.board(&host_id).is_none());
    let store = Store::open(&path).unwrap();
    let reply = store
        .command_at_now(&g.room_id, &sessions[0].token, restart.clone(), 0)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        serde_json::to_value(&first.view).unwrap()
    );
    drop(store);
    let store = Store::open(&path).unwrap();
    assert_eq!(
        serde_json::to_value(
            store
                .state_at_now(&g.room_id, &sessions[0].token, 90000)
                .await
                .unwrap()
        )
        .unwrap(),
        serde_json::to_value(room.view(0, 90000)).unwrap()
    );
    let p = room.game.pending.clone().unwrap();
    let next = RoomCommand {
        command_id: "sealed-next-mulligan".into(),
        expected_version: room.revision,
        action: SessionAction::Game {
            action: Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        },
    };
    let second = room.transition(p.seat, Some(next.clone()), 90000).unwrap();
    assert!(second.error_code.is_none());
    let reply = store
        .command_at_now(&g.room_id, &sessions[p.seat].token, next.clone(), 90000)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        serde_json::to_value(&second.view).unwrap()
    );
    let saved = Store::open(&path).unwrap();
    let repeat = saved
        .command_at_now(&g.room_id, &sessions[0].token, restart.clone(), 86400000)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(&repeat).unwrap(),
        serde_json::to_value(&first.view).unwrap()
    );
    drop(saved);
    drop(store);
    let db = rusqlite::Connection::open(&path).unwrap();
    let actual: String = db
        .query_row("SELECT state FROM rooms WHERE id=?1", [&g.room_id], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(actual, second.state);
    RoomEnvelope::from_persisted(&actual).unwrap();
    let count: u64 = db
        .query_row(
            "SELECT COUNT(*) FROM commands WHERE room_id=?1",
            [&g.room_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);
    if let Ok(dir) = std::env::var("SEALING_RESTART_TRACE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let steps = vec![
            serde_json::json!({"seat":0,"command":restart,"serverNowMs":"0","transition":first,
            "views":(0..4).map(|seat|RoomEnvelope::from_persisted(&first.state).unwrap().view(seat,0)).collect::<Vec<_>>()}),
            serde_json::json!({"seat":p.seat,"command":next,"serverNowMs":"90000","transition":second,
            "views":(0..4).map(|seat|RoomEnvelope::from_persisted(&second.state).unwrap().view(seat,90000)).collect::<Vec<_>>()}),
        ];
        std::fs::write(format!("{dir}/native-trace.json"),serde_json::to_vec(&serde_json::json!({"scope":"actual-pass-terminal-then-authenticated-local-Store-restart-reopen-next-command-original-receipt","initialState":initial,"steps":steps})).unwrap()).unwrap();
    }
}

#[cfg(feature = "native")]
#[test]
fn sealing_fresh_lobby_legal_match_terminal_restart_and_persisted_next_choice() {
    use crate::deck::DeckDraft;
    use crate::room::{Decision, RoomCommand, RoomEnvelope, SessionAction};
    use sha2::{Digest, Sha256};
    fn hash(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }
    fn draft(seat: usize) -> DeckDraft {
        let entries: Vec<(&str, usize)> = match seat {
            0 => vec![("XQ40", 3), ("XQ45", 3), ("JC125", 44)],
            1 => vec![
                ("JZ49", 3),
                ("JC084", 3),
                ("JC032", 3),
                ("JZ24", 3),
                ("JC125", 38),
            ],
            _ => vec![("JC125", 50)],
        };
        DeckDraft {
            id: format!("fresh-sealed-{seat}"),
            name: format!("Restart lifecycle P{seat}"),
            description: "Legal custom deck, deterministic fresh-lobby test only".into(),
            society_id: None,
            cards: entries
                .into_iter()
                .map(|(id, count)| catalog::DeckEntry {
                    card_id: id.into(),
                    count,
                })
                .collect(),
            rules_version: catalog::RULES_VERSION.into(),
            card_pool_version: catalog::POOL_VERSION.into(),
            engine_version: catalog::ENGINE_VERSION.into(),
            updated_at: "2026-10-07".into(),
        }
    }
    // Production factory and validated custom decks. After joining, every
    // state change below is a real Room command: no hand/deck/board/status edit.
    let mut g = Game::new_with_deck(
        "fresh-sealed-restart".into(),
        "FRESH49".into(),
        "teams".into(),
        "P0".into(),
        draft(0),
        9,
    )
    .unwrap();
    for seat in 1..4 {
        g.join_with_deck(format!("P{seat}"), draft(seat)).unwrap();
    }
    let mut room = RoomEnvelope::from_game(g);
    let initial = serde_json::to_string(&room).unwrap();
    let mut steps = Vec::new();
    let mut now = 0;
    let mut record = |room: &mut RoomEnvelope, seat: usize, action: SessionAction| {
        now += 1;
        let command = RoomCommand {
            command_id: format!("fresh-sealed-{}", steps.len()),
            expected_version: room.revision,
            action,
        };
        let transition = room.transition(seat, Some(command.clone()), now).unwrap();
        assert!(
            transition.error_code.is_none(),
            "step {} seat {seat}: {:?}",
            steps.len(),
            transition.error_message
        );
        assert_eq!(
            serde_json::to_string(&room.replay_events(&transition.journal).unwrap()).unwrap(),
            transition.state
        );
        *room = RoomEnvelope::from_persisted(&transition.state).unwrap();
        let views = (0..4)
            .map(|s| {
                hash(
                    &serde_json::to_vec(&serde_json::to_value(room.view(s, now)).unwrap()).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        // Compact hashes are of canonical full transition JSON and exact opaque
        // state bytes, not just selected fields. This avoids huge trace copies.
        steps.push(serde_json::json!({"seat":seat,"command":command,"serverNowMs":now.to_string(),
            "transitionSha256":hash(&serde_json::to_vec(&serde_json::to_value(&transition).unwrap()).unwrap()),
            "stateSha256":hash(transition.state.as_bytes()),"viewSha256":views}));
        transition
    };
    for seat in 0..4 {
        record(
            &mut room,
            seat,
            SessionAction::Game {
                action: Action::new("ready"),
            },
        );
    }
    record(
        &mut room,
        0,
        SessionAction::Game {
            action: Action::new("start"),
        },
    );
    let mut sealed_host = None;
    let mut milled = [0usize; 4];
    for move_no in 0..10000 {
        if room.game.status == "finished" {
            break;
        }
        let g = &room.game;
        let (seat, session) = if let Some(p) = &g.pending {
            let selected = match &p.resolution {
                ChoiceResolution::Declare { declaration, .. }
                    if declaration.ability.key == "mill-two-entry" =>
                {
                    let target = if milled[2] <= milled[3] { 2 } else { 3 };
                    milled[target] += 2;
                    vec![player_id(target)]
                }
                _ if p.choice.kind == "handSeal" => vec![p
                    .choice
                    .options
                    .iter()
                    .find(|o| {
                        g.players[p.seat]
                            .hand
                            .iter()
                            .any(|c| c.id == o.id && c.definition == "JC125")
                    })
                    .unwrap()
                    .id
                    .clone()],
                _ => p
                    .choice
                    .options
                    .iter()
                    .take(p.choice.min.unwrap_or(0))
                    .map(|o| o.id.clone())
                    .collect(),
            };
            (
                p.seat,
                SessionAction::Game {
                    action: Action {
                        choice_id: Some(p.choice.id.clone()),
                        selected: Some(selected),
                        ..Action::new("choose")
                    },
                },
            )
        } else if let Some(window) = &room.pacing.window {
            let seat = *window
                .members
                .iter()
                .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
                .unwrap()
                .0;
            (
                seat,
                SessionAction::PassResponse {
                    window_id: window.id.clone(),
                },
            )
        } else {
            let seat = g
                .living(g.priority_team)
                .into_iter()
                .find(|s| !g.passed.contains(s))
                .unwrap();
            let legal = g.legal_actions(seat);
            let card_def = |a: &LegalAction| {
                a.action
                    .card_id
                    .as_ref()
                    .and_then(|id| g.players[seat].hand.iter().find(|c| &c.id == id))
                    .map(|c| c.definition.as_str())
            };
            let source = g
                .regions
                .iter()
                .flat_map(|r| &r.cards)
                .find(|c| c.controller == 0 && c.definition == "XQ40");
            let host = g.regions.get(2).and_then(|r| {
                r.cards
                    .iter()
                    .find(|c| c.controller == 0 && c.definition == "JC125")
            });
            let preferred = if seat == 0 && g.sealed_cards.is_empty() && sealed_host.is_none() {
                legal
                    .iter()
                    .find(|a| {
                        a.action.kind == "activate"
                            && a.action.ability_id.as_deref() == Some("draw-hand-seal")
                            && a.action.target_id.as_deref() == host.map(|c| c.id.as_str())
                    })
                    .or_else(|| {
                        legal.iter().find(|a| {
                            a.action.kind == "deploy"
                                && card_def(a) == Some("XQ40")
                                && source.is_none()
                                && a.action.region == Some(0)
                        })
                    })
                    .or_else(|| {
                        legal.iter().find(|a| {
                            a.action.kind == "deploy"
                                && card_def(a) == Some("JC125")
                                && host.is_none()
                                && a.action.region == Some(2)
                        })
                    })
                    .or_else(|| {
                        legal.iter().find(|a| {
                            a.action.kind == "asset"
                                && (card_def(a) == Some("XQ45")
                                    || (card_def(a) == Some("XQ40")
                                        && g.players[0]
                                            .hand
                                            .iter()
                                            .filter(|c| c.definition == "XQ40")
                                            .count()
                                            > 1))
                        })
                    })
                    .or_else(|| {
                        legal.iter().find(|a| {
                            a.action.kind == "asset"
                                && card_def(a) == Some("JC125")
                                && (host.is_some()
                                    || g.players[0]
                                        .hand
                                        .iter()
                                        .filter(|c| c.definition == "JC125")
                                        .count()
                                        > 1)
                        })
                    })
            } else if seat == 1 {
                legal
                    .iter()
                    .find(|a| {
                        a.action.kind == "deploy"
                            && card_def(a) == Some("JZ49")
                            && sealed_host.is_some()
                            && (milled[2] < 2 || milled[3] < 2)
                            && a.action.region == Some(4)
                    })
                    .or_else(|| {
                        legal.iter().find(|a| {
                            a.action.kind == "asset"
                                && matches!(card_def(a), Some("JC084" | "JC032" | "JZ24"))
                        })
                    })
                    .or_else(|| {
                        legal
                            .iter()
                            .find(|a| a.action.kind == "asset" && card_def(a) == Some("JC125"))
                    })
            } else {
                None
            };
            let action = preferred
                .or_else(|| legal.iter().find(|a| a.action.kind == "pass"))
                .unwrap_or_else(|| panic!("no action at move {move_no}, seat {seat}"))
                .action
                .clone();
            (seat, SessionAction::Game { action })
        };
        record(&mut room, seat, session);
        if let Some(seal) = room.game.sealed_cards.first() {
            sealed_host = Some(seal.host_id.clone());
        }
    }
    assert_eq!(
        room.game.status,
        "finished",
        "turn {}, decks {:?}, mills {milled:?}, seals {}",
        room.game.turn,
        room.game
            .players
            .iter()
            .map(|p| p.deck.len())
            .collect::<Vec<_>>(),
        room.game.sealed_cards.len()
    );
    assert_eq!(room.game.winner_team, Some(0));
    assert!(room.game.players[2].eliminated && room.game.players[3].eliminated);
    assert_eq!(
        room.game.sealed_cards.len(),
        1,
        "natural terminal must retain the reviewable seal"
    );
    let host = sealed_host.unwrap();
    assert_eq!(room.game.sealed_cards[0].host_id, host);
    let terminal = serde_json::to_string(&room).unwrap();
    room = RoomEnvelope::from_persisted(&terminal).unwrap();
    let first = record(
        &mut room,
        0,
        SessionAction::Game {
            action: Action::new("restart"),
        },
    );
    assert!(room.game.sealed_cards.is_empty() && room.game.board(&host).is_none());
    // Reopen the real restart result before sending the next mandatory choice.
    room = RoomEnvelope::from_persisted(&first.state).unwrap();
    let p = room.game.pending.clone().unwrap();
    let second = record(
        &mut room,
        p.seat,
        SessionAction::Game {
            action: Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        },
    );
    assert_eq!(room.game.status, "playing");
    assert!(room.game.sealed_cards.is_empty());
    drop(record);
    if let Ok(dir) = std::env::var("SEALING_FRESH_RESTART_TRACE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(format!("{dir}/fresh-match.json"), serde_json::to_vec(&serde_json::json!({"scope":"actual-fresh-lobby-validated-decks-seed9-legal-Room-commands-no-board-or-terminal-injection","initialState":initial,"steps":steps,"terminalState":terminal,"restartState":first.state,"nextChoiceState":second.state})).unwrap()).unwrap();
        let restart_steps = vec![
            serde_json::json!({"seat":0,"command":steps[steps.len()-2]["command"],"serverNowMs":steps[steps.len()-2]["serverNowMs"],"transition":first,"views":(0..4).map(|s|RoomEnvelope::from_persisted(&first.state).unwrap().view(s,now-1)).collect::<Vec<_>>()}),
            serde_json::json!({"seat":p.seat,"command":steps[steps.len()-1]["command"],"serverNowMs":steps[steps.len()-1]["serverNowMs"],"transition":second,"views":(0..4).map(|s|RoomEnvelope::from_persisted(&second.state).unwrap().view(s,now)).collect::<Vec<_>>()}),
        ];
        std::fs::write(format!("{dir}/native-trace.json"),serde_json::to_vec(&serde_json::json!({"scope":"natural-fresh-match-terminal-state-then-real-restart-persist-next-choice","initialState":terminal,"steps":restart_steps})).unwrap()).unwrap();
    }
}
