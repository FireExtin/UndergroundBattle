//! Explicit offline layouts; real paid commands and both persistence loaders.
//! Tapping/control/host-departure probes are marked primitive layouts, not card actions.
use crate::jc029_tests::{apply, board, checkpoint, envelope, fund, game, pass_top, reject};
use crate::{catalog, model::*, rules::*};
const KEY: &str = "xq25-attach-region";
fn held(g: &mut Game, actor: usize, def: &str) -> String {
    let c = g.make_card(def, actor);
    let id = c.id.clone();
    g.players[actor].hand.push(c);
    id
}
fn action(kind: &str, id: &str, r: usize) -> Action {
    Action {
        card_id: Some(id.into()),
        ability_id: (kind == "play").then(|| KEY.into()),
        region: Some(r),
        ..Action::new(kind)
    }
}
fn snap(name: &str, g: &Game, a: &Action) {
    checkpoint(g);
    if let Ok(dir) = std::env::var("XQ25_EVIDENCE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let r = envelope(g);
        std::fs::write(format!("{dir}/{name}.json"),serde_json::to_vec(&serde_json::json!({
            "scope":"offline-xq25-native; explicit prepared layout, actual accepted command",
            "state":serde_json::to_string(&r).unwrap(),"action":a,"views":(0..4).map(|s|r.view(s,0)).collect::<Vec<_>>()
        })).unwrap()).unwrap();
    }
}
fn invalid(g: &Game, bad: Game, name: &str) {
    assert!(
        Game::from_persisted(&serde_json::to_string(&bad).unwrap()).is_err(),
        "Game accepted {name}"
    );
    let mut r = envelope(g);
    r.game = bad;
    let state = serde_json::to_string(&r).unwrap();
    assert!(
        crate::room::RoomEnvelope::from_persisted(&state).is_err(),
        "Room accepted {name}"
    );
    if let Ok(dir) = std::env::var("XQ25_INVALID_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/{name}.json"),
            serde_json::to_vec(&serde_json::json!({"state":state})).unwrap(),
        )
        .unwrap();
    }
}
fn attach(g: &mut Game, actor: usize, r: usize) -> String {
    // Prepared priority only; play, payment, response passes and attachment commit are real.
    g.first_team = g.team(actor);
    g.begin_window(Window::Action(g.team(actor)));
    fund(g, actor, "JC059", 2);
    let id = held(g, actor, "XQ25");
    apply(g, actor, action("play", &id, r));
    pass_top(g);
    g.attachments.last().unwrap().card.id.clone()
}
fn conceal_layout(actor: usize, probe_owner: usize) -> (Game, String, String) {
    let mut g = game(probe_owner);
    let probe = attach(&mut g, probe_owner, 2);
    g.first_team = g.team(actor);
    g.begin_window(Window::Action(g.team(actor)));
    let id = held(&mut g, actor, "JC125");
    (g, probe, id)
}
#[test]
fn xq25_whole_original_and_canonical_definition_reject_rewrites_and_transplants() {
    let c = catalog::card("XQ25");
    assert_eq!(
        (&*c.name, &*c.kind, &*c.color, c.cost),
        ("监控探头", "attachment", "灰", 2)
    );
    assert_eq!(c.loyalty, ["灰色", "灰色"]);
    assert_eq!(c.subtypes, ["设施"]);
    assert!(c.magic.is_empty() && !c.unique && c.defense.is_none() && c.keywords.is_empty());
    assert_eq!(c.permanent_icons, Icons::default());
    assert_eq!(
        c.temporary_icons,
        Icons {
            investigation: 1,
            ..Default::default()
        }
    );
    assert_eq!(c.text,"结附于目标地区\n【持续】只要监控探头未横置，玩家向其所结附的地区进行秘密派遣所需的费用额外增加1。");
    let d = definition("XQ25");
    assert_eq!(d.abilities[0].key, KEY);
    assert_eq!(d.abilities.len(), 1);
    assert!(d.abilities[0].play_only);
    assert_eq!(d.abilities[0].timing, Timing::Standard);
    assert_eq!(d.abilities[0].targets[0].zone, Zone::Region);
    assert_eq!(d.abilities[0].targets[0].relation, Relation::Any);
    assert_eq!(d.abilities[0].targets[0].range, Range::Anywhere);
    assert!(d.abilities[0].costs.is_empty() && d.abilities[0].ops.is_empty());
    for n in 0..10 {
        let mut bad = d.clone();
        match n {
            0 => bad.abilities[0].timing = Timing::Fast,
            1 => bad.abilities[0].costs = vec![Cost::Assets(1)],
            2 => bad.abilities[0].play_only = false,
            3 => bad.abilities[0].targets[0].zone = Zone::Board,
            4 => bad.abilities[0].targets[0].relation = Relation::ControlledByActor,
            5 => bad.modifiers.clear(),
            6 => bad.attachment.as_mut().unwrap().controls_host = true,
            7 => bad.attachment.as_mut().unwrap().host_icons.investigation = 1,
            8 => bad.attachment.as_mut().unwrap().host_leaves = HostLeaveDestination::OwnerHand,
            _ => bad.abilities[0].response_policy = ResponsePolicy::Immediate,
        }
        let mut ds = definitions().clone();
        ds.insert("XQ25".into(), bad);
        assert!(crate::rules::validate_definitions(&ds).is_err());
    }
    let mut ds = definitions().clone();
    ds.insert("JC125".into(), d.clone());
    assert!(crate::rules::validate_definitions(&ds).is_err());
    crate::rules::validate_definitions(definitions()).unwrap();
}
#[test]
fn xq25_actual_paid_region_attachment_all_seats_all_live_regions_without_character_host_or_distance_gate(
) {
    for actor in 0..4 {
        for r in 0..5 {
            let mut g = game(actor);
            fund(&mut g, actor, "JC059", 2);
            let id = held(&mut g, actor, "XQ25");
            if actor == 0 && r == 4 {
                snap("attach", &g, &action("play", &id, r));
            }
            let host = g.regions[r].card.id.clone();
            apply(&mut g, actor, action("play", &id, r));
            assert_eq!(g.resources(actor), 0);
            assert_eq!(
                g.stack.last().unwrap().frame.as_ref().unwrap().targets[0].region_instance,
                Some(host.clone())
            );
            pass_top(&mut g);
            let a = g.attachments.last().unwrap();
            assert_eq!(a.host_id, host);
            assert_ne!(a.card.id, id);
            assert_eq!((a.card.owner, a.card.controller), (actor, actor));
            assert!(!a.card.face_down && !a.card.exhausted);
            assert_eq!(g.secret_deploy_cost(r), 2);
            assert!(g.pending.is_none());
            let first = g.team(actor) == g.first_team;
            assert_eq!(g.icons(&a.card, r).investigation, u32::from(first));
            let mut expected = [0, 0];
            expected[g.team(actor)] = u32::from(first);
            assert_eq!(g.contest_counts(r, 0), expected);
            checkpoint(&g);
        }
    }
}
#[test]
fn xq25_cost_loyalty_nonregion_vacant_and_extra_payload_reject_atomically() {
    for n in 0..6 {
        let mut g = game(0);
        let id = held(&mut g, 0, "XQ25");
        fund(
            &mut g,
            0,
            if n == 0 { "JC029" } else { "JC059" },
            if n == 1 { 1 } else { 2 },
        );
        let mut a = action("play", &id, 2);
        match n {
            2 => {
                a.region = None;
                a.target_id = Some(board(&mut g, "JC125", 0, 2));
            }
            3 => {
                g.regions[2].vacant = true;
                g.regions[2].cards.clear();
            }
            4 => a.selected = Some(vec!["bogus".into()]),
            5 => a.ability_id = Some("bogus".into()),
            _ => {}
        }
        // Vacant-region primitive is tested without exporting an invalid whole layout.
        if n == 3 {
            let before = serde_json::to_string(&g).unwrap();
            assert!(g.apply(0, a).is_err());
            assert_eq!(serde_json::to_string(&g).unwrap(), before);
        } else {
            reject(&mut g, 0, a);
        }
    }
}
#[test]
fn xq25_all_players_pay_surcharge_and_hidden_identity_stays_private() {
    for actor in 0..4 {
        let (mut g, _, id) = conceal_layout(actor, (actor + 1) % 4);
        fund(&mut g, actor, "JC059", 2);
        let a = action("conceal", &id, 2);
        if actor == 0 {
            snap("conceal-surcharge", &g, &a);
        }
        apply(&mut g, actor, a);
        assert_eq!(g.resources(actor), 0);
        assert!(g.players[actor].conceal_used);
        assert!(g.stack.is_empty());
        let c = g.regions[2].cards.last().unwrap();
        assert_ne!(c.id, id);
        assert_eq!(c.controller, actor);
        assert!(c.face_down);
        let room = envelope(&g);
        for seat in 0..4 {
            let v = room.view(seat, 0);
            let hidden = v.regions[2]
                .characters
                .iter()
                .find(|x| x.instance_id == c.id)
                .unwrap();
            assert_eq!(hidden.card_id.is_some(), seat == actor);
        }
        checkpoint(&g);
    }
}
#[test]
fn xq25_insufficient_extra_cost_preserves_assets_hand_quota_rng_and_version_all_seats() {
    for actor in 0..4 {
        let (mut g, _, id) = conceal_layout(actor, (actor + 1) % 4);
        fund(&mut g, actor, "JC059", 1);
        reject(&mut g, actor, action("conceal", &id, 2));
        assert_eq!(g.resources(actor), 1);
        assert!(!g.players[actor].conceal_used);
        assert!(!g
            .legal_actions(actor)
            .iter()
            .any(|a| a.action.kind == "conceal" && a.action.region == Some(2)));
    }
}
#[test]
fn xq25_multiple_active_probes_stack_independent_of_owner_controller_and_team() {
    let mut g = game(0);
    let first = attach(&mut g, 0, 2);
    attach(&mut g, 1, 2);
    attach(&mut g, 2, 3);
    // Explicit control-change primitive: the continuous clause is actor-independent.
    g.board_mut(&first).unwrap().controller = 3;
    g.first_team = 1;
    g.begin_window(Window::Action(1));
    assert_eq!(g.secret_deploy_cost(2), 3);
    assert_eq!(g.secret_deploy_cost(3), 2);
    fund(&mut g, 3, "JC059", 3);
    let id = held(&mut g, 3, "JC125");
    let a = action("conceal", &id, 2);
    snap("conceal-stacked", &g, &a);
    apply(&mut g, 3, a);
    assert_eq!(g.resources(3), 0);
    checkpoint(&g);
}
#[test]
fn xq25_tapped_probe_stops_surcharge_and_actual_next_turn_readies_it() {
    let (mut g, probe, id) = conceal_layout(0, 1);
    // External exhaustion primitive; XQ25 has no invented self-tapping action.
    g.board_mut(&probe).unwrap().exhausted = true;
    fund(&mut g, 0, "JC059", 1);
    assert_eq!(g.secret_deploy_cost(2), 1);
    let a = action("conceal", &id, 2);
    snap("conceal-tapped", &g, &a);
    apply(&mut g, 0, a);
    assert_eq!(g.resources(0), 0);
    g.effects.push_back(Effect::NextTurn);
    g.drive().unwrap();
    assert!(!g.board(&probe).unwrap().1.exhausted);
    assert_eq!(g.secret_deploy_cost(2), 2);
    checkpoint(&g);
}
#[test]
fn xq25_other_region_base_cost_and_host_departure_cleans_source_without_index_reuse() {
    let (mut g, probe, id) = conceal_layout(0, 1);
    assert_eq!(g.secret_deploy_cost(1), 1);
    fund(&mut g, 0, "JC059", 1);
    apply(&mut g, 0, action("conceal", &id, 1));
    // Host-departure primitive uses the shared lifecycle destination and reset.
    let host = g.regions[2].card.id.clone();
    g.host_leaves(&host);
    assert!(g.board(&probe).is_none());
    assert_eq!(g.secret_deploy_cost(2), 1);
    let grave = g.players[1]
        .graveyard
        .iter()
        .find(|c| c.definition == "XQ25")
        .unwrap();
    assert_ne!(grave.id, probe);
    assert_eq!(grave.controller, 1);
    assert!(!grave.face_down && !grave.exhausted);
    checkpoint(&g);
}
#[test]
fn xq25_face_up_deploy_and_paid_reveal_keep_printed_prices() {
    let (mut g, _, id) = conceal_layout(0, 1);
    fund(&mut g, 0, "JC059", 2);
    apply(&mut g, 0, action("deploy", &id, 2));
    pass_top(&mut g);
    assert_eq!(g.resources(0), 2);
    assert!(!g.players[0].conceal_used);
    let id = held(&mut g, 0, "JC029");
    apply(&mut g, 0, action("conceal", &id, 2));
    assert_eq!(g.resources(0), 0);
    let hidden = g.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC029")
        .unwrap()
        .id
        .clone();
    fund(&mut g, 0, "JC029", 2);
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(hidden),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut g);
    assert_eq!(g.resources(0), 0);
    checkpoint(&g);
}
#[test]
fn xq25_legal_labels_prices_do_not_bypass_distance_public_or_turn_quota() {
    let (mut g, probe, id) = conceal_layout(0, 1);
    fund(&mut g, 0, "JC059", 3);
    let labels = g.legal_actions(0);
    let affected = labels
        .iter()
        .find(|a| {
            a.action.kind == "conceal"
                && a.action.card_id.as_deref() == Some(&id)
                && a.action.region == Some(2)
        })
        .unwrap();
    assert!(affected.label.ends_with("（费用：支付2）"));
    assert!(!labels
        .iter()
        .find(|a| a.action.kind == "conceal" && a.action.region == Some(1))
        .unwrap()
        .label
        .contains("费用"));
    assert!(!labels
        .iter()
        .any(|a| a.action.kind == "deploy" && a.label.contains("费用")));
    reject(&mut g, 0, action("conceal", &id, 4));
    let public = held(&mut g, 0, "WM023");
    reject(&mut g, 0, action("conceal", &public, 2));
    apply(&mut g, 0, action("conceal", &id, 2));
    let second = held(&mut g, 0, "JC125");
    reject(&mut g, 0, action("conceal", &second, 2));
    assert!(!g
        .legal_actions(0)
        .iter()
        .any(|a| a.action.kind == "activate" && a.action.card_id.as_deref() == Some(&probe)));
}
#[test]
fn xq25_actual_paid_jc107_destroys_probe_and_immediately_removes_extra_cost() {
    let (mut g, probe, _) = conceal_layout(0, 1);
    fund(&mut g, 0, "JC103", 3);
    let destroy = held(&mut g, 0, "JC107");
    apply(
        &mut g,
        0,
        Action {
            card_id: Some(destroy),
            target_id: Some(probe.clone()),
            ..Action::new("play")
        },
    );
    assert_eq!(g.secret_deploy_cost(2), 2);
    pass_top(&mut g);
    assert!(g.board(&probe).is_none());
    assert_eq!(g.secret_deploy_cost(2), 1);
    assert!(g.players[1]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ25"));
    checkpoint(&g);
}
#[test]
fn xq25_original_region_replaced_during_response_cancels_attachment_without_refund() {
    let mut g = game(0);
    fund(&mut g, 0, "JC059", 2);
    let id = held(&mut g, 0, "XQ25");
    apply(&mut g, 0, action("play", &id, 2));
    // Prepared physical-region replacement during response, preserving the region card set.
    let replacement = g.regions[3].card.clone();
    let old = std::mem::replace(&mut g.regions[2].card, replacement);
    g.regions[3].card = old;
    checkpoint(&g);
    pass_top(&mut g);
    assert!(g.attachments.is_empty());
    assert_eq!(g.resources(0), 0);
    assert!(g.players[0]
        .graveyard
        .iter()
        .any(|c| c.definition == "XQ25"));
    assert_eq!(g.secret_deploy_cost(2), 1);
    checkpoint(&g);
}
#[test]
fn xq25_actual_win_flow_returns_probe_to_owner_deck_and_new_region_has_no_surcharge() {
    let (mut g, probe, _) = conceal_layout(0, 1);
    g.players[0].hand.clear();
    let region = g.regions[2].card.id.clone();
    let count = g.players[1]
        .deck
        .iter()
        .filter(|c| c.definition == "XQ25")
        .count();
    g.place_influence(0, 2, 100);
    g.drive().unwrap();
    assert!(matches!(g.window, Some(Window::Win(2, 0))));
    for _ in 0..4 {
        let seat = (0..4)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        apply(&mut g, seat, Action::new("pass"));
    }
    assert!(g.board(&probe).is_none());
    assert_ne!(g.regions[2].card.id, region);
    assert_eq!(g.secret_deploy_cost(2), 1);
    assert_eq!(
        g.players[1]
            .deck
            .iter()
            .filter(|c| c.definition == "XQ25")
            .count(),
        count + 1
    );
    checkpoint(&g);
}
#[test]
fn xq25_region_attachment_does_not_trigger_bq030_character_host_observer() {
    let mut g = game(0);
    board(&mut g, "BQ030", 0, 2);
    let hand = g.players[0].hand.len();
    let deck = g.players[0].deck.len();
    attach(&mut g, 0, 2);
    assert!(g.pending.is_none());
    assert_eq!(g.players[0].hand.len(), hand);
    assert_eq!(g.players[0].deck.len(), deck);
    checkpoint(&g);
}
#[test]
fn xq25_malformed_play_frames_shells_direct_effects_and_declarations_reject_both_loaders() {
    let mut g = game(0);
    fund(&mut g, 0, "JC059", 2);
    let id = held(&mut g, 0, "XQ25");
    apply(&mut g, 0, action("play", &id, 2));
    for n in 0..17 {
        let mut bad = g.clone();
        let s = bad.stack.last_mut().unwrap();
        let f = s.frame.as_mut().unwrap();
        match n {
            0 => f.actor = 99,
            1 => f.source.card.definition = "JC125".into(),
            2 => f.source.card.controller = 1,
            3 => f.source.region = Some(2),
            4 => f.source.attachment_host_instance = Some("wrong".into()),
            5 => f.cursor = 1,
            6 => f.already_paid = vec![],
            7 => f.already_paid = vec![PaidCost::Assets(vec!["same".into(), "same".into()])],
            8 => f.targets[0].region_instance = None,
            9 => f.targets[0].spec.zone = Zone::Board,
            10 => f.chosen_region = Some(3),
            11 => f.targets[0].public.kind = "character".into(),
            12 => s.deploy_region = Some(2),
            13 => s.reveal = true,
            14 => s.card = None,
            15 => f.guard = GuardState::Accepted,
            _ => s.frame = None,
        }
        invalid(&g, bad, &format!("play-frame-{n}"));
    }
    let mut bad = g.clone();
    let s = bad.stack.pop().unwrap();
    bad.effects.push_back(Effect::Frame {
        frame: Box::new(s.frame.unwrap()),
    });
    invalid(&g, bad, "direct-effect-queue");
    let frame = g.stack.last().unwrap().frame.as_ref().unwrap();
    let declaration = Declaration {
        actor: 0,
        source: frame.source.clone(),
        ability: definition("XQ25").abilities[0].clone(),
    };
    let mut bad = g.clone();
    bad.effects.push_back(Effect::Declare {
        declaration: declaration.clone(),
    });
    invalid(&g, bad, "invented-trigger-declaration");
    let mut bad = g.clone();
    bad.choice(
        0,
        "optional",
        "forged XQ25 trigger".into(),
        vec![],
        0,
        0,
        None,
        ChoiceResolution::Declare {
            declaration,
            stage: DeclareChoice::Accept,
        },
    );
    invalid(&g, bad, "invented-trigger-choice");
    let mut bad = g.clone();
    bad.choice(
        0,
        "hand",
        "forged XQ25 continuation".into(),
        vec![],
        0,
        0,
        None,
        ChoiceResolution::Frame {
            frame: Box::new(frame.clone()),
            choice: FrameChoice::WM023HandEntry,
        },
    );
    invalid(&g, bad, "invented-frame-choice");
}
#[test]
fn xq25_invalid_in_play_host_visibility_controller_and_duplicate_identity_reject_both_loaders() {
    let (g, _, _) = conceal_layout(0, 1);
    for n in 0..6 {
        let mut bad = g.clone();
        match n {
            0 => bad.attachments[0].host_id = "missing".into(),
            1 => bad.attachments[0].card.face_down = true,
            2 => bad.attachments[0].card.controller = 99,
            3 => bad.attachments[0].card.owner = 99,
            4 => bad.attachments[0].card.id.clear(),
            _ => {
                let a = bad.attachments[0].clone();
                bad.attachments.push(a);
            }
        }
        invalid(&g, bad, &format!("in-play-{n}"));
    }
}
#[test]
fn xq25_duel_actual_payment_commit_conceal_and_room_restore() {
    // Two-seat explicit prepared layout; do not reuse four-seat checkpoint exporters.
    let mut g = game(0);
    g.mode = "duel".into();
    g.players.truncate(2);
    g.jc035_preparation = None; // Explicit two-seat fixture discards the four-seat observation.
    g.first_team = 0;
    g.begin_window(Window::Action(0));
    fund(&mut g, 0, "JC059", 4);
    let probe = held(&mut g, 0, "XQ25");
    g.apply(0, action("play", &probe, 4)).unwrap();
    while !g.stack.is_empty() {
        let seat = (0..2)
            .find(|s| g.legal_actions(*s).iter().any(|a| a.action.kind == "pass"))
            .unwrap();
        g.apply(seat, Action::new("pass")).unwrap();
        g = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
    }
    let id = held(&mut g, 0, "JC125");
    g.apply(0, action("conceal", &id, 4)).unwrap();
    assert_eq!(g.resources(0), 0);
    let room = crate::room::RoomEnvelope::from_game(g);
    let state = serde_json::to_string(&room).unwrap();
    let restored = crate::room::RoomEnvelope::from_persisted(&state).unwrap();
    for seat in 0..2 {
        assert_eq!(
            serde_json::to_value(room.view(seat, 0)).unwrap(),
            serde_json::to_value(restored.view(seat, 0)).unwrap()
        );
    }
}
