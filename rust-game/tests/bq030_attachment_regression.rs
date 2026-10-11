//! Real BQ030 admission and attachment-event Room regressions.
//! Initial layouts are explicit; afterward only real Room transitions are used.
use hegemony_server::{
    catalog,
    model::{Action, ChoiceResolution, Game, Window},
    room::{Decision, RoomCommand, RoomEnvelope, SessionAction},
};
use std::sync::atomic::{AtomicUsize, Ordering};
static SEQ: AtomicUsize = AtomicUsize::new(0);

fn require_candidate() {
    assert!(
        catalog::catalog()
            .cards
            .iter()
            .any(|c| c.id == "BQ030" && c.supported),
        "BQ030 must be admitted before these actual-card Room regressions run"
    );
}

fn initial(actor: usize, host_controller: usize, observer: bool) -> (RoomEnvelope, String) {
    if observer {
        require_candidate();
    }
    let mut g = Game::new(
        "bq030-room-regression".into(),
        "LOCAL".into(),
        "teams".into(),
        "P0".into(),
        "watchers".into(),
        9,
    )
    .unwrap();
    for s in 1..4 {
        g.join(format!("P{s}"), "watchers".into()).unwrap();
    }
    for p in &mut g.players {
        p.ready = true;
    }
    g.apply(0, Action::new("start")).unwrap();
    while let Some(p) = g.pending.clone() {
        g.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    for r in &mut g.regions {
        r.cards.clear();
    }
    for p in &mut g.players {
        p.hand.clear();
        p.assets.clear();
        p.graveyard.clear();
    }
    for seat in 0..4 {
        for _ in 0..6 {
            let c = g.make_card("JZ22", seat);
            g.players[seat].assets.push(c);
        }
    }
    for seat in 0..4 {
        g.players[seat].deck.clear();
        for _ in 0..4 {
            let c = g.make_card("JC125", seat);
            g.players[seat].deck.push(c);
        }
    }
    let host = g.make_card("XQ12", host_controller);
    let host_id = host.id.clone();
    g.regions[0].cards.push(host);
    if observer {
        let c = g.make_card("BQ030", 0);
        g.regions[2].cards.push(c);
    }
    g.first_team = actor / 2;
    g.active_team = actor / 2;
    g.priority_team = actor / 2;
    g.passed.clear();
    g.team_passed = [false; 2];
    g.window = Some(Window::Action(actor / 2));
    let room = RoomEnvelope::from_game(g);
    (
        RoomEnvelope::from_persisted(&serde_json::to_string(&room).unwrap()).unwrap(),
        host_id,
    )
}

fn held(room: &mut RoomEnvelope, card_id: &str, actor: usize) -> String {
    // Only used while preparing the explicit initial layout.
    let c = room.game.make_card(card_id, actor);
    let id = c.id.clone();
    room.game.players[actor].hand.push(c);
    id
}

fn command(room: &mut RoomEnvelope, seat: usize, action: SessionAction) {
    let sequence = SEQ.fetch_add(1, Ordering::Relaxed);
    let request = RoomCommand {
        command_id: format!("bq030-{sequence}"),
        expected_version: room.revision,
        action,
    };
    let before = serde_json::to_string(room).unwrap();
    let result = room.transition(seat, Some(request.clone()), 0).unwrap();
    assert!(result.error_code.is_none(), "{:?}", result.error_message);
    let replay = room.replay_events(&result.journal).unwrap();
    assert_eq!(serde_json::to_string(&replay).unwrap(), result.state);
    let restored = RoomEnvelope::from_persisted(&result.state).unwrap();
    assert_eq!(serde_json::to_string(&restored).unwrap(), result.state);
    let views: Vec<_> = (0..4).map(|viewer| restored.view(viewer, 0)).collect();
    for viewer in 0..4 {
        assert_eq!(
            serde_json::to_value(replay.view(viewer, 0)).unwrap(),
            serde_json::to_value(&views[viewer]).unwrap()
        );
    }
    if let Ok(dir) = std::env::var("BQ030_TRACE_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let catalog_path = std::path::Path::new(&dir)
            .parent()
            .unwrap()
            .join("native-catalog.json");
        if !catalog_path.exists() {
            std::fs::write(
                catalog_path,
                serde_json::to_vec(catalog::catalog()).unwrap(),
            )
            .unwrap();
        }
        std::fs::write(
            format!("{dir}/transition-{sequence:04}.json"),
            serde_json::to_vec(
                &serde_json::json!({ "scope":"actual-bq030-native-room-regression",
                "state":before,"seat":seat,"command":request,"expected":result,"views":views }),
            )
            .unwrap(),
        )
        .unwrap();
    }
    *room = restored;
}

fn act(room: &mut RoomEnvelope, seat: usize, action: Action) {
    assert!(
        room.pacing.window.is_none(),
        "Responses must use Begin/Submit protocol"
    );
    command(room, seat, SessionAction::Game { action });
}

fn pass_top(room: &mut RoomEnvelope) {
    let count = room.game.stack.len();
    assert!(count > 0);
    for _ in 0..32 {
        if room.game.stack.len() < count || room.game.pending.is_some() {
            return;
        }
        let window = room.pacing.window.clone().expect("real response window");
        let seat = *window
            .members
            .iter()
            .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
            .expect("undecided response seat")
            .0;
        command(
            room,
            seat,
            SessionAction::PassResponse {
                window_id: window.id,
            },
        );
    }
    panic!("top did not resolve within the bounded response loop");
}

fn choose(room: &mut RoomEnvelope, accept: bool) {
    let pending = room
        .game
        .pending
        .clone()
        .expect("BQ030 attachment declaration missing");
    assert_eq!(pending.seat, 0);
    assert_eq!(
        (
            pending.choice.min,
            pending.choice.max,
            pending.choice.allow_decline
        ),
        (Some(0), Some(1), Some(true))
    );
    let ChoiceResolution::Declare { declaration, .. } = &pending.resolution else {
        panic!("attachment observation must use the existing optional declaration");
    };
    assert_eq!(declaration.source.card.definition, "BQ030");
    assert_eq!(
        serde_json::to_value(declaration.ability.event).unwrap(),
        serde_json::json!("AttachmentCommittedObserved")
    );
    act(
        room,
        0,
        Action {
            choice_id: Some(pending.choice.id),
            selected: Some(if accept {
                vec!["accept".into()]
            } else {
                vec![]
            }),
            ..Action::new("choose")
        },
    );
}

fn select(room: &mut RoomEnvelope, selected: Vec<String>) {
    let pending = room.game.pending.clone().expect("real pending choice");
    act(
        room,
        pending.seat,
        Action {
            choice_id: Some(pending.choice.id),
            selected: Some(selected),
            ..Action::new("choose")
        },
    );
}

fn observer(room: &mut RoomEnvelope, owner: usize, controller: usize, region: usize) -> String {
    // Only before the first Room command in each explicitly prepared layout.
    let mut c = room.game.make_card("BQ030", owner);
    c.controller = controller;
    let id = c.id.clone();
    room.game.regions[region].cards.push(c);
    id
}

fn fund(room: &mut RoomEnvelope, card: &str, seat: usize, count: usize) {
    for _ in 0..count {
        let c = room.game.make_card(card, seat);
        room.game.players[seat].assets.push(c);
    }
}

fn response(room: &mut RoomEnvelope, seat: usize, action: Action) {
    // Let the engine's actual team priority advance to this responding seat.
    for _ in 0..8 {
        let w = room
            .pacing
            .window
            .as_ref()
            .expect("response priority window");
        if matches!(w.members.get(&seat), Some(Decision::Undecided { .. })) {
            break;
        }
        let other = *w
            .members
            .iter()
            .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
            .expect("undecided prior response seat")
            .0;
        let id = w.id.clone();
        command(room, other, SessionAction::PassResponse { window_id: id });
    }
    let window = room
        .pacing
        .window
        .as_ref()
        .expect("response window")
        .id
        .clone();
    let intent = format!("bq030-response-{}", SEQ.fetch_add(1, Ordering::Relaxed));
    command(
        room,
        seat,
        SessionAction::BeginResponse {
            window_id: window.clone(),
            intent_id: intent.clone(),
        },
    );
    command(
        room,
        seat,
        SessionAction::SubmitResponse {
            window_id: window,
            intent_id: intent,
            action,
        },
    );
}

fn rejected(room: &RoomEnvelope, seat: usize, action: Action) {
    let state = serde_json::to_string(room).unwrap();
    let request = RoomCommand {
        command_id: format!("bq030-reject-{}", SEQ.fetch_add(1, Ordering::Relaxed)),
        expected_version: room.revision,
        action: SessionAction::Game { action },
    };
    let result = room.transition(seat, Some(request.clone()), 0).unwrap();
    assert!(result.error_code.is_some());
    assert!(!result.changed && result.journal.is_empty());
    assert_eq!(result.state, state, "invalid command must be atomic");
    if let Ok(dir) = std::env::var("BQ030_TRACE_DIR") {
        let sequence = SEQ.fetch_add(1, Ordering::Relaxed);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            format!("{dir}/transition-{sequence:04}.json"),
            serde_json::to_vec(
                &serde_json::json!({"scope":"actual-bq030-rejected-room-command",
                "state":state,"seat":seat,"command":request,"expected":result,
                "views":(0..4).map(|s| room.view(s,0)).collect::<Vec<_>>() }),
            )
            .unwrap(),
        )
        .unwrap();
    }
}

fn attach(room: &mut RoomEnvelope, actor: usize, held_id: String, host: &str) {
    act(
        room,
        actor,
        Action {
            card_id: Some(held_id),
            target_id: Some(host.into()),
            ..Action::new("play")
        },
    );
    pass_top(room);
}

#[test]
fn preparation_room_harness_uses_existing_bq040_payment_replay_and_four_views() {
    let (mut room, host) = initial(0, 0, false);
    let blood = held(&mut room, "BQ040", 0);
    attach(&mut room, 0, blood, &host);
    assert_eq!(room.game.attachments.len(), 1);
    assert_eq!(room.game.attachments[0].host_id, host);
    assert_eq!(
        room.game.players[0]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        1
    );
    assert!(room.game.pending.is_none());
}

#[test]
fn bq030_source_fields_must_be_admitted_before_runtime_acceptance() {
    require_candidate();
    let c = catalog::card("BQ030");
    assert_eq!(
        (
            c.name.as_str(),
            c.cost,
            c.color.as_str(),
            c.magic.as_str(),
            c.defense
        ),
        ("裁定官", 2, "蓝", "鲜血", Some(1))
    );
    assert_eq!(c.loyalty, ["蓝色"]);
    assert_eq!(c.subtypes, ["吸血鬼", "政治家"]);
    assert_eq!(c.kind, "character");
    assert_eq!(c.magic_icon, hegemony_server::rules::MagicIcon::Blood);
    assert_eq!(c.permanent_icons, hegemony_server::model::Icons::default());
    assert_eq!(
        c.temporary_icons,
        hegemony_server::model::Icons {
            influence: 1,
            ..Default::default()
        }
    );
    assert_eq!(c.text, "触发 当一个附属结附于本方角色上时，抓一张牌。");
    assert!(!c.unique && c.keywords.is_empty());
    assert_eq!(c.temporary_icons.influence, 1);
}

#[test]
fn bq030_paid_deploy_then_cross_region_attachment_accepts_one_draw_or_declines() {
    require_candidate();
    let (mut room, host) = initial(0, 0, false);
    let officer = held(&mut room, "BQ030", 0);
    let blood = held(&mut room, "BQ040", 0);
    act(
        &mut room,
        0,
        Action {
            card_id: Some(officer),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    pass_top(&mut room);
    assert!(
        room.game.pending.is_none(),
        "deployment alone is not attachment"
    );
    attach(&mut room, 0, blood, &host);
    assert!(
        room.game.pending.is_some(),
        "successful attachment did not emit the BQ030 observation"
    );
    let mut declined = room.clone();
    let hand = room.game.players[0].hand.len();
    let deck = room.game.players[0].deck.len();
    choose(&mut declined, false);
    assert!(declined.game.pending.is_none());
    assert!(declined.game.stack.is_empty() && declined.game.effects.is_empty());
    assert_eq!(
        (
            declined.game.players[0].hand.len(),
            declined.game.players[0].deck.len()
        ),
        (hand, deck)
    );
    choose(&mut room, true);
    assert!(
        room.pacing.window.is_some(),
        "accepted trigger must be respondable"
    );
    pass_top(&mut room);
    assert_eq!(
        (
            room.game.players[0].hand.len(),
            room.game.players[0].deck.len()
        ),
        (hand + 1, deck - 1)
    );
    let drawn = room.game.players[0].hand.last().unwrap().id.clone();
    for viewer in 1..4 {
        assert!(!serde_json::to_string(&room.view(viewer, 0))
            .unwrap()
            .contains(&drawn));
    }
    assert!(room.game.pending.is_none() && room.game.stack.is_empty());
}

#[test]
fn bq030_foreign_attachment_qualifies_but_teammate_host_does_not() {
    for (attachment_actor, host_controller, qualifies) in
        [(1, 0, true), (2, 0, true), (0, 1, false), (0, 2, false)]
    {
        let (mut room, host) = initial(attachment_actor, host_controller, true);
        let blood = held(&mut room, "BQ040", attachment_actor);
        attach(&mut room, attachment_actor, blood, &host);
        assert_eq!(room.game.pending.is_some(), qualifies);
        if qualifies {
            choose(&mut room, false);
        }
    }
}

#[test]
fn bq030_real_xq47_paid_transfer_to_own_host_emits_one_observation() {
    let (mut room, original_host) = initial(0, 1, true);
    let target = room.game.make_card("XQ12", 0);
    let target_id = target.id.clone();
    room.game.regions[1].cards.push(target);
    let vest = held(&mut room, "XQ47", 0);
    attach(&mut room, 0, vest, &original_host);
    assert!(room.game.pending.is_none());
    let source = room.game.attachments[0].card.id.clone();
    let spent = room.game.players[0]
        .assets
        .iter()
        .filter(|c| c.exhausted)
        .count();
    act(
        &mut room,
        0,
        Action {
            card_id: Some(source.clone()),
            target_id: Some(target_id.clone()),
            ability_id: Some("reattach".into()),
            ..Action::new("activate")
        },
    );
    pass_top(&mut room);
    assert_eq!(room.game.attachments[0].card.id, source);
    assert_eq!(room.game.attachments[0].host_id, target_id);
    assert_eq!(
        room.game.players[0]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        spent + 2
    );
    choose(&mut room, false);
    assert!(room.game.pending.is_none());
    assert!(room.game.stack.is_empty() && room.game.effects.is_empty());
}

#[test]
fn bq030_multiple_sources_choose_stack_order_and_can_decline_remaining() {
    let (mut room, host) = initial(0, 0, true);
    let first = room.game.regions[2].cards[0].id.clone();
    let second = observer(&mut room, 1, 0, 4); // Controlled, despite another owner.
    room.game.regions[4].cards[0].exhausted = true; // No ready-source cost.
    let blood = held(&mut room, "BQ040", 0);
    attach(&mut room, 0, blood, &host);
    let pending = room.game.pending.as_ref().expect("simultaneous observers");
    assert_eq!(pending.choice.options.len(), 2);
    assert!(pending.choice.description.contains("入栈"));
    assert!(room.pacing.window.is_none());
    let hand = room.game.players[0].hand.len();
    select(&mut room, vec![second.clone()]);
    assert_eq!(room.game.stack.len(), 1);
    assert_eq!(
        room.game.stack[0].frame.as_ref().unwrap().source.card.id,
        second
    );
    assert!(room.game.pending.is_some() && room.pacing.window.is_none());
    let mut partial = room.clone();
    select(&mut partial, vec![]);
    assert_eq!(partial.game.stack.len(), 1);
    pass_top(&mut partial);
    assert_eq!(partial.game.players[0].hand.len(), hand + 1);
    choose(&mut room, true); // The final remaining observer uses Accept.
    assert!(room.game.pending.is_none() && room.pacing.window.is_some());
    assert_eq!(room.game.stack.len(), 2);
    assert_eq!(
        room.game
            .stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .source
            .card
            .id,
        first
    );
    pass_top(&mut room); // Last declared resolves first.
    assert_eq!(room.game.players[0].hand.len(), hand + 1);
    assert_eq!(
        room.game
            .stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .source
            .card
            .id,
        second
    );
    pass_top(&mut room);
    assert_eq!(room.game.players[0].hand.len(), hand + 2);
    assert!(room.game.pending.is_none() && room.game.stack.is_empty());
}

#[test]
fn bq030_hidden_source_and_barred_attachment_do_not_trigger_or_pay() {
    let (mut hidden, host) = initial(0, 0, true);
    hidden.game.regions[2].cards[0].face_down = true;
    let blood = held(&mut hidden, "BQ040", 0);
    attach(&mut hidden, 0, blood, &host);
    assert!(hidden.game.pending.is_none());

    let (mut barred, _) = initial(0, 0, true);
    // Printed barrier prevents enemy targeting; friendly equipment remains legal.
    let shield = barred.game.make_card("JZ08", 2);
    let shield_id = shield.id.clone();
    barred.game.regions[0].cards.clear();
    barred.game.regions[0].cards.push(shield);
    let vest = held(&mut barred, "XQ47", 0);
    rejected(
        &barred,
        0,
        Action {
            card_id: Some(vest),
            target_id: Some(shield_id),
            ..Action::new("play")
        },
    );
    assert!(barred.game.pending.is_none() && barred.game.attachments.is_empty());
    assert!(barred.game.players[0].assets.iter().all(|c| !c.exhausted));
}

#[test]
fn bq030_region_attachment_is_not_attachment_to_a_character() {
    let (mut room, _) = initial(0, 0, true);
    let manifestation = held(&mut room, "XQ43", 0);
    fund(&mut room, "XQ43", 0, 2);
    let host = room.game.regions[0].card.id.clone();
    act(
        &mut room,
        0,
        Action {
            card_id: Some(manifestation),
            region: Some(0),
            ..Action::new("play")
        },
    );
    pass_top(&mut room);
    assert_eq!(room.game.attachments.len(), 1);
    assert_eq!(room.game.attachments[0].host_id, host);
    assert!(room.game.pending.is_none());
    assert!(room.game.stack.is_empty() && room.game.effects.is_empty());
}

#[test]
fn bq030_embrace_observation_uses_new_host_controller() {
    let (mut room, _) = initial(2, 0, true);
    let human = room.game.make_card("JC125", 0);
    let host = human.id.clone();
    room.game.regions[0].cards.clear();
    room.game.regions[0].cards.push(human);
    let new_observer = observer(&mut room, 2, 2, 4);
    let embrace = held(&mut room, "JC036", 2);
    attach(&mut room, 2, embrace, &host);
    assert_eq!(room.game.regions[0].cards[0].controller, 2);
    let pending = room
        .game
        .pending
        .as_ref()
        .expect("new controller's trigger");
    assert_eq!(pending.seat, 2);
    let ChoiceResolution::Declare { declaration, .. } = &pending.resolution else {
        panic!()
    };
    assert_eq!(declaration.source.card.id, new_observer);
    let hand = room.game.players[2].hand.len();
    select(&mut room, vec!["accept".into()]);
    pass_top(&mut room);
    assert_eq!(room.game.players[2].hand.len(), hand + 1);
    assert!(room.game.pending.is_none());
}

#[test]
fn bq030_accepted_source_can_die_to_real_response_and_still_draws() {
    let (mut room, host) = initial(0, 0, true);
    let officer = room.game.regions[2].cards[0].id.clone();
    let blood = held(&mut room, "BQ040", 0);
    let fire = held(&mut room, "JC102", 2);
    fund(&mut room, "JC102", 2, 2);
    attach(&mut room, 0, blood, &host);
    choose(&mut room, true);
    let hand = room.game.players[0].hand.len();
    response(
        &mut room,
        2,
        Action {
            card_id: Some(fire),
            target_id: Some(officer.clone()),
            ..Action::new("play")
        },
    );
    pass_top(&mut room);
    assert!(!room
        .game
        .regions
        .iter()
        .any(|r| r.cards.iter().any(|c| c.id == officer)));
    assert_eq!(room.game.stack.len(), 1);
    pass_top(&mut room);
    assert_eq!(room.game.players[0].hand.len(), hand + 1);
}

#[test]
fn bq030_attachment_fizzling_after_real_host_death_emits_no_observation() {
    let (mut room, _) = initial(0, 0, true);
    let human = room.game.make_card("JC125", 0);
    let host = human.id.clone();
    room.game.regions[0].cards.clear();
    room.game.regions[0].cards.push(human);
    let vest = held(&mut room, "XQ47", 0);
    let murder = held(&mut room, "JC091", 2);
    fund(&mut room, "JC091", 2, 3);
    act(
        &mut room,
        0,
        Action {
            card_id: Some(vest),
            target_id: Some(host.clone()),
            ..Action::new("play")
        },
    );
    response(
        &mut room,
        2,
        Action {
            card_id: Some(murder),
            target_id: Some(host),
            ..Action::new("play")
        },
    );
    pass_top(&mut room);
    pass_top(&mut room);
    assert!(room.game.attachments.is_empty());
    assert!(room.game.pending.is_none());
    assert!(room.game.stack.is_empty() && room.game.effects.is_empty());
}

#[test]
fn bq030_same_host_transfer_rejected_without_new_event_or_payment() {
    let (mut room, host) = initial(0, 0, true);
    let vest = held(&mut room, "XQ47", 0);
    attach(&mut room, 0, vest, &host);
    choose(&mut room, false);
    let source = room.game.attachments[0].card.id.clone();
    let paid = room.game.players[0]
        .assets
        .iter()
        .filter(|c| c.exhausted)
        .count();
    rejected(
        &room,
        0,
        Action {
            card_id: Some(source),
            target_id: Some(host),
            ability_id: Some("reattach".into()),
            ..Action::new("activate")
        },
    );
    assert_eq!(
        room.game.players[0]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        paid
    );
    assert!(room.game.pending.is_none());
}

#[test]
fn bq030_deploy_transaction_cannot_erase_or_disguise_its_frame_on_restore() {
    let (mut room, _) = initial(0, 0, false);
    let other_officer = room.game.make_card("BQ030", 0);
    let other_officer_id = other_officer.id.clone();
    room.game.regions[4].cards.push(other_officer);
    let officer = held(&mut room, "BQ030", 0);
    let same_card_asset = room.game.make_card("BQ030", 0);
    let ordered_asset_id = same_card_asset.id.clone();
    room.game.players[0].assets.push(same_card_asset);
    act(
        &mut room,
        0,
        Action {
            card_id: Some(officer),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    let item = room.game.stack.last().unwrap();
    assert_eq!(item.card.as_ref().unwrap().definition, "BQ030");
    assert_eq!(item.frame.as_ref().unwrap().source.card.definition, "BQ030");
    let good = serde_json::to_value(&room).unwrap();
    for (label, path, value) in [
        (
            "deploy-source-disguised",
            "/game/stack/0/frame/source/card/definition",
            serde_json::json!("JC125"),
        ),
        (
            "deploy-frame-erased",
            "/game/stack/0/frame",
            serde_json::Value::Null,
        ),
        (
            "deploy-fresh-id-borrows-asset",
            "/game/stack/0/card/id",
            serde_json::json!(room.game.players[0].assets[0].id),
        ),
        (
            "deploy-source-borrows-board-officer",
            "/game/stack/0/frame/source/card/id",
            serde_json::json!(other_officer_id),
        ),
        (
            "deploy-fresh-id-borrows-ordered-officer-asset",
            "/game/stack/0/card/id",
            serde_json::json!(ordered_asset_id),
        ),
    ] {
        let mut corrupt = good.clone();
        *corrupt.pointer_mut(path).unwrap() = value;
        let state = serde_json::to_string(&corrupt).unwrap();
        if let Ok(dir) = std::env::var("BQ030_TRACE_DIR") {
            let invalid = std::path::Path::new(&dir).parent().unwrap().join("invalid");
            std::fs::create_dir_all(&invalid).unwrap();
            std::fs::write(
                invalid.join(format!("{label}.json")),
                serde_json::to_vec(&serde_json::json!({"state":state,"case":label})).unwrap(),
            )
            .unwrap();
        }
        assert!(
            RoomEnvelope::from_persisted(&state).is_err(),
            "accepted corrupt BQ030 transaction: {label}"
        );
        assert!(
            Game::from_persisted(&serde_json::to_string(&corrupt["game"]).unwrap()).is_err(),
            "accepted corrupt BQ030 Game: {label}"
        );
    }
}

#[test]
fn bq030_frozen_event_cannot_borrow_a_live_card_identity() {
    let (mut room, host) = initial(0, 0, true);
    let officer_asset = room.game.make_card("BQ030", 0);
    let officer_asset_id = officer_asset.id.clone();
    room.game.players[0].assets.push(officer_asset);
    let attachment_asset = room.game.make_card("BQ040", 0);
    let attachment_asset_id = attachment_asset.id.clone();
    room.game.players[0].assets.push(attachment_asset);
    let blood = held(&mut room, "BQ040", 0);
    attach(&mut room, 0, blood, &host);
    let good = serde_json::to_value(&room).unwrap();
    let declaration = "/game/pending/resolution/Declare/declaration";
    let event = format!("{declaration}/ability/ops/0/BQ030AttachmentDraw/observation");
    let asset_id = room.game.players[0].assets[0].id.clone();
    for case in 0..9 {
        let mut corrupt = good.clone();
        let label = match case {
            0 => {
                for path in [
                    format!("{declaration}/source/card/id"),
                    format!("{event}/observers/0/card/id"),
                    format!("{event}/remaining/0"),
                ] {
                    *corrupt.pointer_mut(&path).unwrap() = asset_id.clone().into();
                }
                "observer-borrows-live-asset-id"
            }
            1 => {
                *corrupt
                    .pointer_mut(&format!("{event}/host/card/id"))
                    .unwrap() = asset_id.clone().into();
                "host-borrows-live-asset-id"
            }
            2 => {
                *corrupt
                    .pointer_mut(&format!("{event}/attachment/id"))
                    .unwrap() = asset_id.clone().into();
                "attachment-borrows-live-asset-id"
            }
            3 => {
                *corrupt
                    .pointer_mut(&format!("{declaration}/source/card/owner"))
                    .unwrap() = 2.into();
                *corrupt
                    .pointer_mut(&format!("{event}/observers/0/card/owner"))
                    .unwrap() = 2.into();
                "observer-disguises-live-owner"
            }
            4 => {
                *corrupt
                    .pointer_mut(&format!("{event}/host/card/definition"))
                    .unwrap() = "JC125".into();
                "host-disguises-live-definition"
            }
            5 => {
                *corrupt
                    .pointer_mut(&format!("{event}/attachment/definition"))
                    .unwrap() = "JC089".into();
                "attachment-disguises-live-definition"
            }
            6 => {
                let source = corrupt
                    .pointer(&format!("{declaration}/source/card"))
                    .unwrap()
                    .clone();
                *corrupt.pointer_mut("/game/players/0/assets/0").unwrap() = source;
                "observer-duplicate-live-identity"
            }
            7 => {
                for path in [
                    format!("{declaration}/source/card/id"),
                    format!("{event}/observers/0/card/id"),
                    format!("{event}/remaining/0"),
                ] {
                    *corrupt.pointer_mut(&path).unwrap() = officer_asset_id.clone().into();
                }
                "observer-borrows-same-card-asset-id"
            }
            _ => {
                *corrupt
                    .pointer_mut(&format!("{event}/attachment/id"))
                    .unwrap() = attachment_asset_id.clone().into();
                "attachment-borrows-same-card-asset-id"
            }
        };
        let state = serde_json::to_string(&corrupt).unwrap();
        if let Ok(dir) = std::env::var("BQ030_TRACE_DIR") {
            let invalid = std::path::Path::new(&dir).parent().unwrap().join("invalid");
            std::fs::create_dir_all(&invalid).unwrap();
            std::fs::write(
                invalid.join(format!("{label}.json")),
                serde_json::to_vec(&serde_json::json!({"state":state,"case":label})).unwrap(),
            )
            .unwrap();
        }
        assert!(
            RoomEnvelope::from_persisted(&state).is_err(),
            "accepted stolen live identity: {label}"
        );
        assert!(
            Game::from_persisted(&serde_json::to_string(&corrupt["game"]).unwrap()).is_err(),
            "accepted stolen Game identity: {label}"
        );
    }
}

#[test]
fn bq030_paid_deploy_frame_cannot_be_restored_as_an_effect() {
    let (mut room, _) = initial(0, 0, false);
    let officer = held(&mut room, "BQ030", 0);
    act(
        &mut room,
        0,
        Action {
            card_id: Some(officer),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    let mut corrupt = serde_json::to_value(&room).unwrap();
    let frame = corrupt["game"]["stack"].as_array_mut().unwrap().remove(0)["frame"].clone();
    corrupt["game"]["effects"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"Frame":{"frame":frame}}));
    corrupt["pacing"]["window"] = serde_json::Value::Null;
    let state = serde_json::to_string(&corrupt).unwrap();
    if let Ok(dir) = std::env::var("BQ030_TRACE_DIR") {
        let invalid = std::path::Path::new(&dir).parent().unwrap().join("invalid");
        std::fs::create_dir_all(&invalid).unwrap();
        std::fs::write(
            invalid.join("deploy-frame-moved-to-effects.json"),
            serde_json::to_vec(
                &serde_json::json!({"state":state,"case":"deploy-frame-moved-to-effects"}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    assert!(
        RoomEnvelope::from_persisted(&state).is_err(),
        "BQ030 atomic deploy frame cannot bypass the transaction stack"
    );
    assert!(Game::from_persisted(&serde_json::to_string(&corrupt["game"]).unwrap()).is_err());
    // The actual paid transaction still resolves normally.
    pass_top(&mut room);
    assert_eq!(room.game.regions[2].cards[0].definition, "BQ030");
}

#[test]
fn bq030_paid_reveal_preserves_snapshot_and_then_observes_attachment() {
    let (mut room, host) = initial(0, 0, true);
    let hidden = &mut room.game.regions[2].cards[0];
    hidden.face_down = true;
    let source = hidden.id.clone();
    let blood = held(&mut room, "BQ040", 0);
    act(
        &mut room,
        0,
        Action {
            card_id: Some(source),
            ..Action::new("reveal")
        },
    );
    pass_top(&mut room);
    assert!(
        room.game.pending.is_none(),
        "reveal alone is not attachment"
    );
    assert!(!room.game.regions[2].cards[0].face_down);
    // A character reveal never produces a spell/attachment transaction burial.
    let mut corrupt = serde_json::to_value(&room).unwrap();
    let card = corrupt
        .pointer_mut("/game/regions/2/cards")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .remove(0);
    corrupt
        .pointer_mut("/game/effects")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({"Bury":{"card":card}}));
    let state = serde_json::to_string(&corrupt).unwrap();
    if let Ok(dir) = std::env::var("BQ030_TRACE_DIR") {
        let invalid = std::path::Path::new(&dir).parent().unwrap().join("invalid");
        std::fs::create_dir_all(&invalid).unwrap();
        std::fs::write(
            invalid.join("revealed-character-burial-container.json"),
            serde_json::to_vec(
                &serde_json::json!({"state":state,"case":"revealed-character-burial-container"}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    assert!(
        RoomEnvelope::from_persisted(&state).is_err(),
        "character cannot be restored as a transaction burial"
    );
    assert!(Game::from_persisted(&serde_json::to_string(&corrupt["game"]).unwrap()).is_err());
    attach(&mut room, 0, blood, &host);
    let hand = room.game.players[0].hand.len();
    choose(&mut room, true);
    pass_top(&mut room);
    assert_eq!(room.game.players[0].hand.len(), hand + 1);
}

#[test]
fn bq030_runtime_payload_and_order_cannot_be_transplanted_or_corrupted_on_restore() {
    fn rejects(value: serde_json::Value, label: &str) {
        let state = serde_json::to_string(&value).unwrap();
        assert!(
            RoomEnvelope::from_persisted(&state).is_err(),
            "accepted corrupt state: {label}"
        );
        let game = serde_json::to_string(&value["game"]).unwrap();
        assert!(
            Game::from_persisted(&game).is_err(),
            "accepted corrupt Game: {label}"
        );
        if let Ok(dir) = std::env::var("BQ030_TRACE_DIR") {
            let invalid = std::path::Path::new(&dir).parent().unwrap().join("invalid");
            std::fs::create_dir_all(&invalid).unwrap();
            std::fs::write(
                invalid.join(format!("{label}.json")),
                serde_json::to_vec(&serde_json::json!({"state":state,"case":label})).unwrap(),
            )
            .unwrap();
        }
    }
    let (mut room, host) = initial(0, 0, true);
    observer(&mut room, 0, 0, 4);
    let blood = held(&mut room, "BQ040", 0);
    attach(&mut room, 0, blood, &host);
    let pending = serde_json::to_value(&room).unwrap();
    let declaration = "/game/pending/resolution/Declare/declaration";
    let event = format!("{declaration}/ability/ops/0/BQ030AttachmentDraw/observation");
    for case in 0..12 {
        let mut corrupt = pending.clone();
        match case {
            0 => {
                *corrupt
                    .pointer_mut(&format!("{declaration}/actor"))
                    .unwrap() = 2.into()
            }
            1 => {
                *corrupt
                    .pointer_mut(&format!("{declaration}/source/card/definition"))
                    .unwrap() = "JZ22".into()
            }
            2 => *corrupt.pointer_mut(&event).unwrap() = serde_json::Value::Null,
            3 => {
                *corrupt
                    .pointer_mut(&format!("{event}/host/card/controller"))
                    .unwrap() = 2.into()
            }
            4 => {
                *corrupt
                    .pointer_mut(&format!("{event}/observers/0/card/face_down"))
                    .unwrap() = true.into()
            }
            5 => {
                *corrupt
                    .pointer_mut(&format!("{event}/observers/0/region"))
                    .unwrap() = 99.into()
            }
            6 => {
                let remaining = corrupt
                    .pointer_mut(&format!("{event}/remaining"))
                    .unwrap()
                    .as_array_mut()
                    .unwrap();
                remaining.push(remaining[0].clone());
            }
            7 => {
                let host = corrupt
                    .pointer(&format!("{event}/host/card/id"))
                    .unwrap()
                    .clone();
                *corrupt.pointer_mut(&format!("{event}/event_id")).unwrap() = host;
            }
            8 => {
                *corrupt
                    .pointer_mut(&format!("{declaration}/ability/costs"))
                    .unwrap() = serde_json::json!([{"Assets":1}])
            }
            9 => {
                *corrupt
                    .pointer_mut(&format!("{declaration}/ability/event"))
                    .unwrap() = "Enter".into()
            }
            10 => corrupt["game"]["pending"]["choice"]["max"] = 2.into(),
            11 => {
                corrupt["game"]["pending"]["choice"]["options"][0]["id"] =
                    "invented-observer".into()
            }
            _ => unreachable!(),
        }
        rejects(corrupt, &format!("bq030-pending-{case:02}"));
    }
    let chosen = room.game.pending.as_ref().unwrap().choice.options[1]
        .id
        .clone();
    select(&mut room, vec![chosen]);
    choose(&mut room, true);
    let stack = serde_json::to_value(&room).unwrap();
    for case in 0..8 {
        let mut corrupt = stack.clone();
        let frame = &mut corrupt["game"]["stack"][0]["frame"];
        match case {
            0 => frame["actor"] = 2.into(),
            1 => frame["steps"][0]["context"] = 2.into(),
            2 => frame["cursor"] = 1.into(),
            3 => frame["already_paid"] = serde_json::json!([{"Assets":[]}]),
            4 => {
                frame["steps"][0]["op"]["BQ030AttachmentDraw"]["observation"] =
                    serde_json::Value::Null
            }
            5 => corrupt["game"]["stack"][0]["controller"] = 2.into(),
            6 => {
                let copied = corrupt["game"]["stack"][0].clone();
                corrupt["game"]["stack"]
                    .as_array_mut()
                    .unwrap()
                    .push(copied);
            }
            7 => {
                let moved =
                    corrupt["game"]["stack"].as_array_mut().unwrap().remove(0)["frame"].clone();
                corrupt["game"]["effects"]
                    .as_array_mut()
                    .unwrap()
                    .push(serde_json::json!({"Frame":{"frame":moved}}));
            }
            _ => unreachable!(),
        }
        rejects(corrupt, &format!("bq030-stack-{case:02}"));
    }
}

#[test]
fn room_response_control_uses_real_begin_submit_and_fizzles_xq47_after_host_death() {
    // Control for the new response harness: no unregistered fixture definition.
    let (mut room, _) = initial(0, 0, false);
    let human = room.game.make_card("JC125", 0);
    let host = human.id.clone();
    room.game.regions[0].cards.clear();
    room.game.regions[0].cards.push(human);
    let vest = held(&mut room, "XQ47", 0);
    let murder = held(&mut room, "JC091", 2);
    fund(&mut room, "JC091", 2, 3);
    act(
        &mut room,
        0,
        Action {
            card_id: Some(vest),
            target_id: Some(host.clone()),
            ..Action::new("play")
        },
    );
    response(
        &mut room,
        2,
        Action {
            card_id: Some(murder),
            target_id: Some(host),
            ..Action::new("play")
        },
    );
    pass_top(&mut room);
    pass_top(&mut room);
    assert!(room.game.attachments.is_empty());
    assert!(room.game.pending.is_none());
    assert!(room.game.stack.is_empty() && room.game.effects.is_empty());
    assert_eq!(
        room.game.players[0]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        1
    );
    assert_eq!(
        room.game.players[2]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        3
    );
}

#[cfg(feature = "native")]
#[tokio::test]
async fn bq030_sqlite_reopen_pending_stack_and_duplicate_receipt_draw_exactly_once() {
    use hegemony_server::service::{CreateRoom, JoinRoom, Session, Store};
    use rusqlite::params;

    async fn stored_command(
        store: &Store,
        seats: &[Session],
        seat: usize,
        action: SessionAction,
    ) -> RoomCommand {
        let before = store.replay(&seats[0].room_id).unwrap();
        let request = RoomCommand {
            command_id: format!("bq030-sqlite-{}", SEQ.fetch_add(1, Ordering::Relaxed)),
            expected_version: before.revision,
            action,
        };
        let expected = before.transition(seat, Some(request.clone()), 0).unwrap();
        assert!(
            expected.error_code.is_none(),
            "{:?}",
            expected.error_message
        );
        store
            .command_at_now(&seats[0].room_id, &seats[seat].token, request.clone(), 0)
            .await
            .unwrap();
        let replay = store.replay(&seats[0].room_id).unwrap();
        assert_eq!(serde_json::to_string(&replay).unwrap(), expected.state);
        let mut views = Vec::new();
        for (s, session) in seats.iter().enumerate() {
            let view = store
                .state_at_now(&seats[0].room_id, &session.token, 0)
                .await
                .unwrap();
            assert_eq!(
                serde_json::to_value(&view).unwrap(),
                serde_json::to_value(replay.view(s, 0)).unwrap()
            );
            views.push(view);
        }
        if let Ok(dir) = std::env::var("BQ030_TRACE_DIR") {
            std::fs::create_dir_all(&dir).unwrap();
            let seq = SEQ.fetch_add(1, Ordering::Relaxed);
            std::fs::write(
                format!("{dir}/transition-{seq:04}.json"),
                serde_json::to_vec(
                    &serde_json::json!({"scope":"actual-bq030-authenticated-sqlite",
                    "state":serde_json::to_string(&before).unwrap(),"seat":seat,
                    "command":request,"expected":expected,"views":views}),
                )
                .unwrap(),
            )
            .unwrap();
        }
        request
    }

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bq030.sqlite3");
    let store = Store::open(&path).unwrap();
    let mut seats = vec![store
        .create(CreateRoom {
            name: "P0".into(),
            mode: "teams".into(),
            deck_id: "watchers".into(),
            deck_draft: None,
        })
        .await
        .unwrap()];
    for seat in 1..4 {
        seats.push(
            store
                .join(JoinRoom {
                    invite_code: seats[0].invite_code.clone(),
                    name: format!("P{seat}"),
                    deck_id: "watchers".into(),
                    deck_draft: None,
                })
                .await
                .unwrap(),
        );
    }
    drop(store);
    let (mut layout, host) = initial(0, 0, true);
    let blood = held(&mut layout, "BQ040", 0);
    layout.game.room_id = seats[0].room_id.clone();
    layout.game.invite_code = seats[0].invite_code.clone();
    layout.game.version = 3;
    let layout = RoomEnvelope::from_game(layout.game);
    let initial_state = serde_json::to_string(&layout).unwrap();
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute(
        "UPDATE rooms SET state=?2,initial_state=?2,revision=?3 WHERE id=?1",
        params![seats[0].room_id, initial_state, layout.revision],
    )
    .unwrap();
    db.execute("DELETE FROM journal WHERE room_id=?1", [&seats[0].room_id])
        .unwrap();
    drop(db);
    let mut store = Store::open(&path).unwrap();
    stored_command(
        &store,
        &seats,
        0,
        SessionAction::Game {
            action: Action {
                card_id: Some(blood),
                target_id: Some(host),
                ..Action::new("play")
            },
        },
    )
    .await;
    for _ in 0..32 {
        let room = store.replay(&seats[0].room_id).unwrap();
        if room.game.pending.is_some() {
            break;
        }
        let window = room.pacing.window.unwrap();
        let seat = *window
            .members
            .iter()
            .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
            .unwrap()
            .0;
        stored_command(
            &store,
            &seats,
            seat,
            SessionAction::PassResponse {
                window_id: window.id,
            },
        )
        .await;
    }
    let waiting = store.replay(&seats[0].room_id).unwrap();
    assert!(
        waiting.game.pending.is_some(),
        "successful attachment must reach actual BQ030 choice"
    );
    drop(store);
    store = Store::open(&path).unwrap();
    assert_eq!(
        serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap(),
        serde_json::to_string(&waiting).unwrap()
    );
    let pending = waiting.game.pending.unwrap();
    let receipt = stored_command(
        &store,
        &seats,
        0,
        SessionAction::Game {
            action: Action {
                choice_id: Some(pending.choice.id),
                selected: Some(vec!["accept".into()]),
                ..Action::new("choose")
            },
        },
    )
    .await;
    let stack = store.replay(&seats[0].room_id).unwrap();
    assert_eq!(stack.game.stack.len(), 1);
    let hand = stack.game.players[0].hand.len();
    drop(store);
    store = Store::open(&path).unwrap();
    assert_eq!(
        serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap(),
        serde_json::to_string(&stack).unwrap()
    );
    let original_receipt = store
        .command_at_now(&seats[0].room_id, &seats[0].token, receipt.clone(), 0)
        .await
        .unwrap();
    assert_eq!(
        store.replay(&seats[0].room_id).unwrap().revision,
        stack.revision
    );
    for _ in 0..32 {
        let room = store.replay(&seats[0].room_id).unwrap();
        if room.game.stack.is_empty() {
            break;
        }
        let window = room.pacing.window.unwrap();
        let seat = *window
            .members
            .iter()
            .find(|(_, d)| matches!(d, Decision::Undecided { .. }))
            .unwrap()
            .0;
        stored_command(
            &store,
            &seats,
            seat,
            SessionAction::PassResponse {
                window_id: window.id,
            },
        )
        .await;
    }
    let done = store.replay(&seats[0].room_id).unwrap();
    assert_eq!(done.game.players[0].hand.len(), hand + 1);
    let duplicate = store
        .command_at_now(&seats[0].room_id, &seats[0].token, receipt, 0)
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(duplicate).unwrap(),
        serde_json::to_value(original_receipt).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&store.replay(&seats[0].room_id).unwrap()).unwrap(),
        serde_json::to_string(&done).unwrap()
    );
    assert!(store.audit_replay(&seats[0].room_id).unwrap().matches);
    drop(store);
    if let Ok(trace) = std::env::var("BQ030_TRACE_DIR") {
        let evidence = std::path::Path::new(&trace)
            .parent()
            .unwrap()
            .join("bq030.sqlite3");
        std::fs::copy(&path, &evidence).unwrap();
        assert_eq!(
            Store::open(&evidence)
                .unwrap()
                .replay(&seats[0].room_id)
                .unwrap()
                .revision,
            done.revision
        );
    }
}

// Parent relayed user ruling Sentinel_e43732e12de88191a10198be4173a2c8: retain
// the qualified optional draw even when this attachment immediately kills BQ030.
#[test]
fn bq030_lethal_poison_retains_frozen_optional_source_for_accept_or_decline() {
    for accept in [false, true] {
        let (mut room, _) = initial(2, 0, true);
        let officer = room.game.regions[2].cards[0].id.clone();
        let poison = held(&mut room, "JC089", 2);
        fund(&mut room, "JC089", 2, 2);
        attach(&mut room, 2, poison, &officer);
        assert!(!room
            .game
            .regions
            .iter()
            .any(|r| r.cards.iter().any(|c| c.id == officer)));
        assert!(room.game.attachments.is_empty());
        assert!(room.game.players[0]
            .graveyard
            .iter()
            .any(|c| c.definition == "BQ030"));
        assert!(room.game.players[2]
            .graveyard
            .iter()
            .any(|c| c.definition == "JC089"));
        let pending = room
            .game
            .pending
            .as_ref()
            .expect("lethal attachment must retain optional trigger");
        assert_eq!(pending.seat, 0);
        let ChoiceResolution::Declare { declaration, .. } = &pending.resolution else {
            panic!()
        };
        assert_eq!(declaration.source.card.id, officer);
        assert_eq!(declaration.source.card.controller, 0);
        let hand = room.game.players[0].hand.len();
        let deck = room.game.players[0].deck.len();
        choose(&mut room, accept);
        if accept {
            assert!(
                room.pacing.window.is_some(),
                "retained ability remains respondable"
            );
            pass_top(&mut room);
        }
        let drawn = usize::from(accept);
        assert_eq!(room.game.players[0].hand.len(), hand + drawn);
        assert_eq!(room.game.players[0].deck.len(), deck - drawn);
        assert!(
            room.game.pending.is_none()
                && room.game.stack.is_empty()
                && room.game.effects.is_empty()
        );
    }
}

#[test]
fn bq030_lethal_source_can_be_ordered_with_a_surviving_observer() {
    let (mut room, _) = initial(2, 0, true);
    let dying = room.game.regions[2].cards[0].id.clone();
    let surviving = observer(&mut room, 1, 0, 4);
    let poison = held(&mut room, "JC089", 2);
    fund(&mut room, "JC089", 2, 2);
    attach(&mut room, 2, poison, &dying);
    let pending = room
        .game
        .pending
        .as_ref()
        .expect("both qualified sources must survive in the event");
    assert_eq!(pending.choice.options.len(), 2);
    assert!(pending.choice.options.iter().any(|c| c.id == dying));
    assert!(pending.choice.options.iter().any(|c| c.id == surviving));
    let hand = room.game.players[0].hand.len();
    select(&mut room, vec![dying.clone()]);
    assert_eq!(
        room.game
            .stack
            .last()
            .unwrap()
            .frame
            .as_ref()
            .unwrap()
            .source
            .card
            .id,
        dying
    );
    assert!(room.game.pending.is_some() && room.pacing.window.is_none());
    let mut corrupt = serde_json::to_value(&room).unwrap();
    corrupt["game"]["stack"].as_array_mut().unwrap().clear();
    let event = &mut corrupt["game"]["pending"]["resolution"]["Declare"]["declaration"]["ability"]
        ["ops"][0]["BQ030AttachmentDraw"]["observation"];
    event["observers"]
        .as_array_mut()
        .unwrap()
        .retain(|source| source["card"]["id"] != dying);
    let state = serde_json::to_string(&corrupt).unwrap();
    if let Ok(dir) = std::env::var("BQ030_TRACE_DIR") {
        let invalid = std::path::Path::new(&dir).parent().unwrap().join("invalid");
        std::fs::create_dir_all(&invalid).unwrap();
        std::fs::write(
            invalid.join("lethal-bq-host-missing-participant.json"),
            serde_json::to_vec(
                &serde_json::json!({"state":state,"case":"lethal-bq-host-missing-participant"}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    assert!(
        RoomEnvelope::from_persisted(&state).is_err(),
        "frozen BQ030 host must remain a member of the full participant snapshot"
    );
    assert!(Game::from_persisted(&serde_json::to_string(&corrupt["game"]).unwrap()).is_err());
    choose(&mut room, false);
    pass_top(&mut room);
    assert_eq!(room.game.players[0].hand.len(), hand + 1);
    assert!(room.game.pending.is_none() && room.game.stack.is_empty());
}
