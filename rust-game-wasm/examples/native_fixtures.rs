//! Native oracle for the WASM ABI test; private state remains an opaque string in JS.
use hegemony_server::{
    catalog, deck,
    model::{
        Action, ChoiceResolution, Declaration, Effect, FrameChoice, Game, SourceSnapshot, Window,
    },
    room::{Decision, QuoteRequest, RoomCommand, RoomEnvelope, RoomTransition, SessionAction},
    rules::{self, CardFilter, Op},
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn step(room: &RoomEnvelope, operation: &str, args: Value, seat: usize) -> Value {
    json!({"operation":operation,"args":args,"state":serde_json::to_string(room).unwrap(),"version":room.revision,"seat":seat,"view":room.view(seat,room.pacing.last_server_now_ms),"views":(0..room.players.len()).map(|s|room.view(s,room.pacing.last_server_now_ms)).collect::<Vec<_>>()})
}
fn record_transition(
    room: &mut RoomEnvelope,
    steps: &mut Vec<Value>,
    operation: &str,
    args: Value,
    transition: RoomTransition,
) {
    let next = RoomEnvelope::from_persisted(&transition.state).unwrap();
    if transition.changed {
        assert_eq!(
            serde_json::to_string(&room.replay_events(&transition.journal).unwrap()).unwrap(),
            transition.state
        );
    } else {
        assert_eq!(serde_json::to_string(room).unwrap(), transition.state);
    }
    *room = next;
    let mut entry = step(room, operation, args, transition.seat);
    entry["view"] = serde_json::to_value(&transition.view).unwrap();
    entry["transition"] = serde_json::to_value(transition).unwrap();
    steps.push(entry);
}
fn apply_session(
    room: &mut RoomEnvelope,
    steps: &mut Vec<Value>,
    seat: usize,
    action: SessionAction,
) {
    let now = 1_000 + steps.len() as u64 * 10;
    let command = RoomCommand {
        command_id: format!("oracle:{}", steps.len()),
        expected_version: room.revision,
        action,
    };
    let transition = room.transition(seat, Some(command.clone()), now).unwrap();
    assert_eq!(
        transition.outcome,
        "accepted",
        "{}",
        transition.error_message.clone().unwrap_or_default()
    );
    record_transition(
        room,
        steps,
        "applyRoom",
        json!([seat, command, now.to_string()]),
        transition,
    );
}
fn apply_game(room: &mut RoomEnvelope, steps: &mut Vec<Value>, seat: usize, action: Action) {
    let session = if let Some(window) = room.pacing.window.clone() {
        if action.kind == "pass" {
            match window.members.get(&seat).unwrap() {
                Decision::Composing { intent_id } => SessionAction::CancelAndPass {
                    window_id: window.id,
                    intent_id: intent_id.clone(),
                },
                Decision::Undecided { .. } => SessionAction::PassResponse {
                    window_id: window.id,
                },
                Decision::Passed => panic!("fixture attempts a duplicate pass"),
            }
        } else {
            let intent = match window.members.get(&seat).unwrap() {
                Decision::Undecided { .. } => {
                    let intent = format!("draft:{}", steps.len());
                    apply_session(
                        room,
                        steps,
                        seat,
                        SessionAction::BeginResponse {
                            window_id: window.id.clone(),
                            intent_id: intent.clone(),
                        },
                    );
                    intent
                }
                Decision::Composing { intent_id } => intent_id.clone(),
                Decision::Passed => panic!("fixture attempts a paid response after passing"),
            };
            SessionAction::SubmitResponse {
                window_id: window.id,
                intent_id: intent,
                action,
            }
        }
    } else {
        action.into()
    };
    apply_session(room, steps, seat, session);
}
fn rejected(room: &RoomEnvelope, seat: usize, action: Action) -> Value {
    let command = RoomCommand {
        command_id: "rejected:fixture".into(),
        expected_version: room.revision,
        action: action.into(),
    };
    let now = room.pacing.last_server_now_ms;
    let transition = room.transition(seat, Some(command.clone()), now).unwrap();
    assert_eq!(transition.outcome, "rejected");
    assert!(!transition.changed);
    json!({"seat":seat,"command":command,"serverNow":now.to_string(),"expected":transition})
}
fn pick_choice(game: &Game) -> (usize, Action) {
    let p = game.pending.as_ref().unwrap();
    let mut action = Action {
        choice_id: Some(p.choice.id.clone()),
        ..Action::new("choose")
    };
    match &p.resolution {
        ChoiceResolution::Forecast { .. }
        | ChoiceResolution::Frame {
            choice: FrameChoice::Forecast { .. },
            ..
        } => {
            let ids = p
                .choice
                .options
                .iter()
                .map(|o| o.id.clone())
                .collect::<Vec<_>>();
            let middle = ids.len() / 2;
            action.top = Some(ids[..middle].iter().rev().cloned().collect());
            action.bottom = Some(ids[middle..].iter().rev().cloned().collect());
        }
        ChoiceResolution::Bottom { .. } => {
            action.bottom = Some(
                p.choice
                    .options
                    .iter()
                    .rev()
                    .map(|o| o.id.clone())
                    .collect(),
            )
        }
        ChoiceResolution::Damage { .. } => {
            let mut remaining = p.choice.amount.unwrap();
            let mut allocations = BTreeMap::new();
            for o in &p.choice.options {
                if o.card
                    .as_ref()
                    .and_then(|c| c.card_id.as_deref())
                    .is_some_and(|id| id == "LC21" || id == "LC22")
                    && remaining > 0
                {
                    allocations.insert(o.id.clone(), 1);
                    remaining -= 1;
                }
            }
            if remaining > 0 {
                *allocations
                    .entry(p.choice.options.last().unwrap().id.clone())
                    .or_default() += remaining;
            }
            action.allocations = Some(allocations);
        }
        _ => {
            let desired = match p.choice.kind.as_str() {
                "mulligan" => 2,
                "trigger" | "target" | "search" | "recipient" => 1,
                "discard" => p.choice.min.unwrap_or(0).max(1),
                _ => p.choice.min.unwrap_or(0),
            };
            action.selected = Some(
                p.choice
                    .options
                    .iter()
                    .rev()
                    .take(desired.min(p.choice.max.unwrap_or(usize::MAX)))
                    .map(|o| o.id.clone())
                    .collect(),
            );
        }
    }
    (p.seat, action)
}
fn response_fixture() -> Value {
    let seed = "9007199254740993";
    let mut game = Game::new(
        "wasm-response".into(),
        "RESPONSE".into(),
        "duel".into(),
        "玩家0".into(),
        "responders".into(),
        seed.parse().unwrap(),
    )
    .unwrap();
    game.join("玩家1".into(), "responders".into()).unwrap();
    for p in &mut game.players {
        p.ready = true;
    }
    game.apply(0, Action::new("start")).unwrap();
    while let Some(p) = game.pending.clone() {
        game.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    // Explicit initial layout shared byte-for-byte with WASM; all subsequent transitions are Actions.
    game.window = Some(Window::Action(0));
    game.first_team = 0;
    game.active_team = 0;
    game.priority_team = 0;
    let disciple = game.make_card("JC042", 0);
    let disciple_id = disciple.id.clone();
    game.regions[0].cards.push(disciple);
    let victim = game.make_card("LC20", 1);
    game.regions[0].cards.push(victim);
    let doctor = game.make_card("LC19", 0);
    let doctor_id = doctor.id.clone();
    game.regions[1].cards.push(doctor);
    let mut wounded = game.make_card("JC059", 0);
    wounded.wounds = 1;
    let wounded_id = wounded.id.clone();
    game.regions[1].cards.push(wounded);
    let murder = game.make_card("JC091", 1);
    let murder_id = murder.id.clone();
    game.players[1].hand = vec![murder];
    let ritual = game.make_card("JZ54", 0);
    let ritual_id = ritual.id.clone();
    let witch = game.make_card("LC24", 0);
    let witch_id = witch.id.clone();
    game.players[0].hand = vec![ritual, witch];
    for _ in 0..2 {
        let asset = game.make_card("JC042", 0);
        game.players[0].assets.push(asset);
        let asset = game.make_card("JC091", 0);
        game.players[0].assets.push(asset);
    }
    for _ in 0..2 {
        let asset = game.make_card("JC125", 0);
        game.players[0].assets.push(asset);
    }
    for _ in 0..3 {
        let asset = game.make_card("JC091", 1);
        game.players[1].assets.push(asset);
    }
    let mut game = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &game,
        "initialFixture",
        json!([serde_json::to_string(&game).unwrap()]),
        0,
    )];
    let mut apply = |game: &mut RoomEnvelope, seat: usize, action: Action| {
        apply_game(game, &mut steps, seat, action);
    };
    apply(&mut game, 0, Action::new("pass"));
    apply(
        &mut game,
        1,
        Action {
            card_id: Some(murder_id),
            target_id: Some(disciple_id.clone()),
            ..Action::new("play")
        },
    );
    apply(&mut game, 1, Action::new("pass"));
    apply(
        &mut game,
        0,
        Action {
            card_id: Some(disciple_id),
            ..Action::new("activate")
        },
    );
    assert_eq!(game.modifiers.len(), 1);
    assert_eq!(game.stack.len(), 1);
    for seat in [0, 1] {
        apply(&mut game, seat, Action::new("pass"));
    }
    apply(
        &mut game,
        0,
        Action {
            card_id: Some(ritual_id),
            target_id: Some("p1".into()),
            ..Action::new("play")
        },
    );
    for seat in [0, 1] {
        apply(&mut game, seat, Action::new("pass"));
    }
    let (seat, choice) = pick_choice(&game);
    apply(&mut game, seat, choice);
    apply(
        &mut game,
        0,
        Action {
            card_id: Some(witch_id),
            region: Some(0),
            ..Action::new("deploy")
        },
    );
    for seat in [0, 1] {
        apply(&mut game, seat, Action::new("pass"));
    }
    let (seat, choice) = pick_choice(&game);
    apply(&mut game, seat, choice);
    for seat in [0, 1] {
        apply(&mut game, seat, Action::new("pass"));
    }
    let (seat, choice) = pick_choice(&game);
    apply(&mut game, seat, choice);
    let heal = Action {
        card_id: Some(doctor_id.clone()),
        target_id: Some(wounded_id.clone()),
        ability_id: Some("heal".into()),
        ..Action::new("activate")
    };
    apply(&mut game, 0, heal.clone());
    assert!(
        game.regions[1]
            .cards
            .iter()
            .find(|c| c.id == doctor_id)
            .unwrap()
            .exhausted
    );
    for seat in [0, 1] {
        apply(&mut game, seat, Action::new("pass"));
    }
    assert_eq!(
        game.regions[1]
            .cards
            .iter()
            .find(|c| c.id == wounded_id)
            .unwrap()
            .wounds,
        0
    );
    let rejected_heal = rejected(&game, 0, heal);
    json!({"name":"real-response-frame-continuation-and-heal-cost","seed":seed,"steps":steps,"rejectedCommands":[rejected_heal]})
}
fn detective_fixture(kill_source: bool) -> Value {
    let seed = "9007199254740993";
    let mut game = Game::new(
        format!("wasm-detective-{kill_source}"),
        "DETECTIVE".into(),
        "duel".into(),
        "刑警操控者".into(),
        "keepers".into(),
        seed.parse().unwrap(),
    )
    .unwrap();
    game.join("暗藏者操控者".into(), "watchers".into()).unwrap();
    for player in &mut game.players {
        player.ready = true;
    }
    game.apply(0, Action::new("start")).unwrap();
    while let Some(pending) = game.pending.clone() {
        game.apply(
            pending.seat,
            Action {
                choice_id: Some(pending.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    // Explicit initial layout, identical opaque bytes in both kernels. Every later
    // state is reached by an Action; this is not evidence of a naturally dealt game.
    game.window = Some(Window::Action(0));
    game.first_team = 0;
    game.active_team = 0;
    game.priority_team = 0;
    for player in &mut game.players {
        player.hand.clear();
    }
    let mut detective = game.make_card("JC058", 0);
    let detective_id = detective.id.clone();
    detective.face_down = true;
    detective.exhausted = true;
    game.regions[0].cards.push(detective);
    let mut target = game.make_card(if kill_source { "XQ12" } else { "LC21" }, 1);
    let target_id = target.id.clone();
    target.face_down = true;
    game.regions[0].cards.push(target);
    for _ in 0..3 {
        let asset = game.make_card("JC058", 0);
        game.players[0].assets.push(asset);
    }
    let response = if kill_source {
        let murder = game.make_card("JC091", 1);
        let murder_id = murder.id.clone();
        game.players[1].hand.push(murder);
        for _ in 0..3 {
            let asset = game.make_card("JC091", 1);
            game.players[1].assets.push(asset);
        }
        Action {
            card_id: Some(murder_id),
            ..Action::new("play")
        }
    } else {
        for _ in 0..catalog::card("LC21").cost {
            let asset = game.make_card("JC125", 1);
            game.players[1].assets.push(asset);
        }
        Action {
            card_id: Some(target_id.clone()),
            ..Action::new("reveal")
        }
    };
    let mut game = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &game,
        "initialFixture",
        json!([serde_json::to_string(&game).unwrap()]),
        0,
    )];
    let mut apply = |game: &mut RoomEnvelope, seat: usize, action: Action| {
        apply_game(game, &mut steps, seat, action);
    };
    apply(
        &mut game,
        0,
        Action {
            card_id: Some(detective_id.clone()),
            ..Action::new("reveal")
        },
    );
    for seat in [0, 1] {
        apply(&mut game, seat, Action::new("pass"));
    }
    let pending = game.pending.as_ref().unwrap();
    assert_eq!(pending.choice.options.len(), 1);
    assert_eq!(
        pending.choice.options[0].card.as_ref().unwrap().card_id,
        None
    );
    let choose = Action {
        choice_id: Some(pending.choice.id.clone()),
        selected: Some(vec![target_id.clone()]),
        ..Action::new("choose")
    };
    game = RoomEnvelope::from_persisted(&serde_json::to_string(&game).unwrap()).unwrap();
    apply(&mut game, 0, choose.clone());
    apply(&mut game, 0, Action::new("pass"));
    let mut response = response;
    if kill_source {
        response.target_id = Some(
            game.regions[0]
                .cards
                .iter()
                .find(|c| c.definition == "JC058")
                .unwrap()
                .id
                .clone(),
        );
    }
    apply(&mut game, 1, response);
    if !kill_source {
        assert_eq!(
            game.view(0, game.pacing.last_server_now_ms).stack[0].target_summaries[0].status,
            "missing"
        );
    }
    for seat in [1, 0] {
        apply(&mut game, seat, Action::new("pass"));
    }
    assert_eq!(game.stack.len(), 1);
    game = RoomEnvelope::from_persisted(&serde_json::to_string(&game).unwrap()).unwrap();
    for seat in [0, 1] {
        apply(&mut game, seat, Action::new("pass"));
    }
    assert!(game.pending.is_none() && game.stack.is_empty());
    assert!(game
        .players
        .iter()
        .flat_map(|p| &p.assets)
        .all(|a| a.exhausted));
    if kill_source {
        assert_eq!(game.players[0].graveyard[0].definition, "JC058");
        assert_eq!(game.players[1].graveyard.len(), 2);
    } else {
        assert!(game.players[1].graveyard.is_empty());
        let revealed = game.regions[0]
            .cards
            .iter()
            .find(|c| c.definition == "LC21")
            .unwrap();
        assert_ne!(revealed.id, target_id);
        assert!(!revealed.face_down);
    }
    json!({"name":if kill_source {"detective-source-death-independent-hidden-destroy"} else {"detective-target-reveal-invalidates-original-instance"},"seed":seed,"steps":steps,"rejectedCommands":[rejected(&game,0,choose),rejected(&game,0,Action{card_id:Some(detective_id),..Action::new("reveal")})]})
}
fn fixture(mode: &str, seed: &str) -> Value {
    let game = Game::new(
        format!("wasm-{mode}"),
        "TEST-INVITE".into(),
        mode.into(),
        "玩家0".into(),
        "watchers".into(),
        seed.parse().unwrap(),
    )
    .unwrap();
    let mut game = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &game,
        "newGame",
        json!([
            game.room_id,
            game.invite_code,
            mode,
            "玩家0",
            "watchers",
            seed
        ]),
        0,
    )];
    for seat in 1..game.capacity() {
        let name = format!("玩家{seat}");
        let deck = ["watchers", "hunters", "keepers", "reclaimers"][seat];
        game.game.join(name.clone(), deck.into()).unwrap();
        game.revision = game.game.version;
        steps.push(step(&game, "joinGame", json!([name, deck]), seat));
    }
    for seat in 0..game.players.len() {
        let action = Action::new("ready");
        apply_game(&mut game, &mut steps, seat, action);
    }
    let action = Action::new("start");
    apply_game(&mut game, &mut steps, 0, action);
    for _ in 0..180 {
        if game.status == "finished" {
            break;
        }
        let (seat, action) = if game.pending.is_some() {
            pick_choice(&game)
        } else {
            let seat = game
                .players
                .iter()
                .find(|p| {
                    !p.eliminated
                        && game.team(p.seat) == game.priority_team
                        && !game.passed.contains(&p.seat)
                })
                .unwrap()
                .seat;
            let legal = game.legal_actions(seat);
            let selected = legal
                .iter()
                .find(|a| {
                    a.action.kind == "asset"
                        && a.action.card_id.as_ref().is_some_and(|id| {
                            game.players[seat].hand.iter().any(|c| {
                                c.id == *id
                                    && (c.definition == "JC125"
                                        || catalog::card(&c.definition).kind == "spell")
                            })
                        })
                })
                .or_else(|| legal.iter().find(|a| a.action.kind == "asset"))
                .or_else(|| legal.iter().find(|a| a.action.kind == "reveal"))
                .or_else(|| {
                    legal.iter().find(|a| {
                        a.action.kind == "deploy"
                            && a.action.card_id.as_ref().is_some_and(|id| {
                                game.players[seat]
                                    .hand
                                    .iter()
                                    .any(|c| c.id == *id && c.definition != "JC125")
                            })
                    })
                })
                .or_else(|| legal.iter().find(|a| a.action.kind == "conceal"))
                .or_else(|| legal.iter().find(|a| a.action.kind == "deploy"))
                .or_else(|| legal.iter().find(|a| a.action.kind == "pass"))
                .unwrap();
            (seat, selected.action.clone())
        };
        apply_game(&mut game, &mut steps, seat, action);
    }
    json!({"name":mode,"seed":seed,"steps":steps})
}
fn custom_deck_fixture() -> Value {
    let seed = "18446744073709551615";
    let mut alpha = deck::preset("responders").unwrap();
    alpha.id = "private-alpha-spec".into();
    alpha.name = "响应自组".into();
    let mut beta = deck::preset("keepers").unwrap();
    beta.id = "private-beta-spec".into();
    beta.name = "守护自组".into();
    let game = Game::new_with_deck(
        "wasm-custom-decks".into(),
        "CUSTOM".into(),
        "teams".into(),
        "玩家0".into(),
        alpha.clone(),
        seed.parse().unwrap(),
    )
    .unwrap();
    let mut game = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &game,
        "newGameWithDeck",
        json!([
            game.room_id,
            game.invite_code,
            "teams",
            "玩家0",
            serde_json::to_string(&alpha).unwrap(),
            seed,
        ]),
        0,
    )];
    alpha.cards.clear(); // Editing the saved draft cannot alter the accepted snapshot.
    assert_eq!(
        game.players[0]
            .deck_snapshot
            .as_ref()
            .unwrap()
            .cards
            .iter()
            .map(|c| c.count)
            .sum::<usize>(),
        50
    );
    game.game
        .join_with_deck("玩家1".into(), beta.clone())
        .unwrap();
    game.revision = game.game.version;
    steps.push(step(
        &game,
        "joinGameWithDeck",
        json!(["玩家1", serde_json::to_string(&beta).unwrap()]),
        1,
    ));
    assert!(
        !serde_json::to_string(&game.view(1, game.pacing.last_server_now_ms))
            .unwrap()
            .contains("private-alpha-spec")
    );
    assert!(
        !serde_json::to_string(&game.view(1, game.pacing.last_server_now_ms))
            .unwrap()
            .contains("JC042")
    );
    for seat in 2..4 {
        game.game
            .join(format!("玩家{seat}"), "watchers".into())
            .unwrap();
        game.revision = game.game.version;
        steps.push(step(
            &game,
            "joinGame",
            json!([format!("玩家{seat}"), "watchers"]),
            seat,
        ));
    }
    let mut gamma = deck::preset("hunters").unwrap();
    gamma.id = "private-gamma-spec".into();
    gamma.name = "追猎自组".into();
    let change = Action {
        deck_draft: Some(gamma.clone()),
        ..Action::new("deck")
    };
    apply_game(&mut game, &mut steps, 2, change);
    for seat in 0..4 {
        let action = Action::new("ready");
        apply_game(&mut game, &mut steps, seat, action);
    }
    let start = Action::new("start");
    apply_game(&mut game, &mut steps, 0, start);
    while let Some(pending) = game.pending.clone() {
        let action = Action {
            choice_id: Some(pending.choice.id),
            selected: Some(vec![]),
            ..Action::new("choose")
        };
        apply_game(&mut game, &mut steps, pending.seat, action);
    }
    assert_eq!(game.players[2].deck_snapshot.as_ref(), Some(&gamma));
    for player in &game.players {
        assert_eq!(player.hand.len() + player.deck.len(), 50);
    }
    json!({"name":"custom-deck-create-join-lobby-freeze-and-start", "seed":seed, "steps":steps,
        "rejectedCommands":[rejected(&game,0,Action { deck_draft: Some(deck::preset("watchers").unwrap()), ..Action::new("deck") })]})
}
fn friendly_icons_fixture() -> Value {
    let seed = "9007199254740993";
    let mut game = Game::new(
        "wasm-friendly-icons".into(),
        "FRIENDLY".into(),
        "teams".into(),
        "玩家0".into(),
        "keepers".into(),
        seed.parse().unwrap(),
    )
    .unwrap();
    for seat in 1..4 {
        game.join(format!("玩家{seat}"), "watchers".into()).unwrap();
    }
    for player in &mut game.players {
        player.ready = true;
    }
    game.apply(0, Action::new("start")).unwrap();
    while let Some(pending) = game.pending.clone() {
        game.apply(
            pending.seat,
            Action {
                choice_id: Some(pending.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    // Explicit initial layout verifies both kernels' public team calculations, not natural dealing.
    game.first_team = 0;
    let source = game.make_card("JC059", 0);
    game.regions[0].cards.push(source);
    let friend = game.make_card("LC20", 1);
    let friend_id = friend.id.clone();
    game.regions[0].cards.push(friend);
    let enemy = game.make_card("LC20", 2);
    game.regions[0].cards.push(enemy);
    let mut hidden = game.make_card("LC24", 3);
    hidden.face_down = true;
    game.regions[0].cards.push(hidden);
    let mut exhausted = game.make_card("LC21", 0);
    exhausted.exhausted = true;
    game.regions[0].cards.push(exhausted);
    for seat in 0..4 {
        assert_eq!(
            game.view(seat).regions[0]
                .characters
                .iter()
                .find(|c| c.instance_id == friend_id)
                .unwrap()
                .defense,
            Some(catalog::card("LC20").defense.unwrap() + 1)
        );
    }
    let game = RoomEnvelope::from_game(game);
    json!({"name":"friendly-teammate-defense-and-public-team-icons", "seed":seed,
        "steps":[step(&game,"initialFixture",json!([serde_json::to_string(&game).unwrap()]),0)]})
}
fn pacing_fixture(expire: bool) -> Value {
    let seed = "9007199254740993";
    let mut game = Game::new(
        if expire {
            "eeeeeeeeeeeeeeeeeeeeeeee".into()
        } else {
            "ffffffffffffffffffffffff".into()
        },
        "CLOCK".into(),
        "teams".into(),
        "玩家0".into(),
        "responders".into(),
        seed.parse().unwrap(),
    )
    .unwrap();
    for seat in 1..4 {
        game.join(format!("玩家{seat}"), "responders".into())
            .unwrap();
    }
    for p in &mut game.players {
        p.ready = true;
    }
    game.apply(0, Action::new("start")).unwrap();
    while let Some(p) = game.pending.clone() {
        game.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    game.window = Some(Window::Action(0));
    game.first_team = 0;
    game.active_team = 0;
    game.priority_team = 0;
    for p in &mut game.players {
        p.hand.clear();
        p.assets.clear();
    }
    let mut sources = vec![];
    for seat in 0..4 {
        let c = game.make_card("JC042", seat);
        sources.push(c.id.clone());
        game.regions[0].cards.push(c);
    }
    let murder = game.make_card("JC091", 0);
    let murder_id = murder.id.clone();
    game.players[0].hand.push(murder);
    for _ in 0..3 {
        let c = game.make_card("JC091", 0);
        game.players[0].assets.push(c);
    }
    let mut room = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        0,
    )];
    let mut quotes = vec![];
    fn run(
        room: &mut RoomEnvelope,
        steps: &mut Vec<Value>,
        seat: usize,
        action: SessionAction,
        now: u64,
        expected: Option<u64>,
    ) {
        let command = RoomCommand {
            command_id: format!("clock:{}", steps.len()),
            expected_version: expected.unwrap_or(room.revision),
            action,
        };
        let result = room.transition(seat, Some(command.clone()), now).unwrap();
        record_transition(
            room,
            steps,
            "applyRoom",
            json!([seat, command, now.to_string()]),
            result,
        );
    }
    fn poll(room: &mut RoomEnvelope, steps: &mut Vec<Value>, now: u64) {
        let result = room.transition(0, None, now).unwrap();
        record_transition(room, steps, "pollRoom", json!([0, now.to_string()]), result);
    }
    run(
        &mut room,
        &mut steps,
        0,
        Action {
            card_id: Some(murder_id),
            target_id: Some(sources[2].clone()),
            ..Action::new("play")
        }
        .into(),
        1_000,
        None,
    );
    let window = room.pacing.window.as_ref().unwrap().id.clone();
    let version = room.revision;
    if expire {
        run(
            &mut room,
            &mut steps,
            0,
            SessionAction::BeginResponse {
                window_id: window.clone(),
                intent_id: "late".into(),
            },
            6_000,
            None,
        );
        assert_eq!(steps.last().unwrap()["transition"]["outcome"], "rejected");
        assert!(steps.last().unwrap()["transition"]["changed"]
            .as_bool()
            .unwrap());
        assert_eq!(room.game.priority_team, 1);
        poll(&mut room, &mut steps, 11_000);
        assert!(room.stack.is_empty());
        run(
            &mut room,
            &mut steps,
            0,
            SessionAction::BeginResponse {
                window_id: window,
                intent_id: "obsolete".into(),
            },
            11_001,
            Some(version),
        );
        assert_eq!(
            steps.last().unwrap()["transition"]["errorCode"],
            "window_expired"
        );
    } else {
        poll(&mut room, &mut steps, 5_999);
        run(
            &mut room,
            &mut steps,
            0,
            SessionAction::BeginResponse {
                window_id: window.clone(),
                intent_id: "first".into(),
            },
            5_999,
            Some(version),
        );
        run(
            &mut room,
            &mut steps,
            1,
            SessionAction::BeginResponse {
                window_id: window.clone(),
                intent_id: "second".into(),
            },
            5_999,
            Some(version),
        );
        let request = QuoteRequest {
            window_id: window.clone(),
            intent_id: "first".into(),
            draft: Some(Action {
                card_id: Some(sources[0].clone()),
                ..Action::new("activate")
            }),
        };
        let state = serde_json::to_string(&room).unwrap();
        let expected = room.quote(0, request.clone()).unwrap();
        assert!(expected.ready);
        quotes.push(json!({"state":state,"seat":0,"request":request,"expected":expected}));
        run(
            &mut room,
            &mut steps,
            0,
            SessionAction::BeginResponse {
                window_id: window.clone(),
                intent_id: "first".into(),
            },
            9_999_999,
            None,
        );
        assert_eq!(room.pacing.window.as_ref().unwrap().id, window);
        run(
            &mut room,
            &mut steps,
            0,
            SessionAction::SubmitResponse {
                window_id: window.clone(),
                intent_id: "first".into(),
                action: Action::new("activate"),
            },
            9_999_999,
            None,
        );
        assert_eq!(
            steps.last().unwrap()["transition"]["errorCode"],
            "invalid_action"
        );
        run(
            &mut room,
            &mut steps,
            0,
            SessionAction::CancelAndPass {
                window_id: window.clone(),
                intent_id: "first".into(),
            },
            10_000_000,
            None,
        );
        run(
            &mut room,
            &mut steps,
            1,
            SessionAction::CancelAndPass {
                window_id: window,
                intent_id: "second".into(),
            },
            10_000_001,
            None,
        );
        poll(&mut room, &mut steps, 10_005_000);
        assert!(!room.stack.is_empty());
        poll(&mut room, &mut steps, 10_005_001);
        assert!(room.stack.is_empty());
    }
    let revision = room.revision;
    let phase = room.window.clone();
    poll(&mut room, &mut steps, 90_000_000);
    assert_eq!(room.revision, revision);
    assert_eq!(room.window, phase);
    json!({"name":if expire {"5000-expiry-rejected-command-commits-only-system-tick"}else{"4999-parallel-intents-quote-repeat-cancel-and-untimed-composition"},"seed":seed,"steps":steps,"quotes":quotes})
}
fn world_fixture(id: &str, generic_search: bool) -> Value {
    let seed = "9007199254740993";
    let actor = if matches!(id, "DQJC108" | "DQJC111") {
        2
    } else if id == "DQJC115" {
        1
    } else {
        0
    };
    let mut game = Game::new(
        format!(
            "{:024x}",
            id.strip_prefix("DQJC").unwrap().parse::<u64>().unwrap() * 2
                + u64::from(generic_search)
        ),
        "WORLD".into(),
        "teams".into(),
        "玩家0".into(),
        "responders".into(),
        seed.parse().unwrap(),
    )
    .unwrap();
    for seat in 1..4 {
        game.join(format!("玩家{seat}"), "responders".into())
            .unwrap();
    }
    for p in &mut game.players {
        p.ready = true;
    }
    game.apply(0, Action::new("start")).unwrap();
    while let Some(p) = game.pending.clone() {
        game.apply(
            p.seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(vec![]),
                ..Action::new("choose")
            },
        )
        .unwrap();
    }
    for p in &mut game.players {
        p.hand.clear();
        p.assets.clear();
        p.graveyard.clear();
    }
    for r in &mut game.regions {
        r.cards.clear();
    }
    // Move the physical world card rather than duplicate a definition. The fixture
    // preserves all ten original regions exactly once across field/world/score.
    if let Some(at) = game.regions.iter().position(|r| r.card.definition == id) {
        game.regions.swap(0, at);
    } else {
        let at = game.world.iter().position(|c| c.definition == id).unwrap();
        std::mem::swap(&mut game.regions[0].card, &mut game.world[at]);
    }
    game.window = Some(Window::Win(0, actor));
    game.first_team = 0;
    game.priority_team = 0;
    game.active_team = 0;
    game.passed.clear();
    game.team_passed = [false; 2];
    fn board(
        g: &mut Game,
        definition: &str,
        owner: usize,
        controller: usize,
        hidden: bool,
    ) -> String {
        let mut c = g.make_card(definition, owner);
        c.controller = controller;
        c.face_down = hidden;
        let id = c.id.clone();
        g.regions[2].cards.push(c);
        id
    }
    let mut old_ids = vec![];
    match id {
        "DQJC108" => {
            for seat in 0..4 {
                let id = board(&mut game, "JC003", seat, seat, true);
                old_ids.push(id);
                if seat == 0 {
                    game.regions[2].cards.last_mut().unwrap().exhausted = true;
                }
            }
        }
        "DQJC109" => {
            old_ids.push(board(&mut game, "JC125", 0, 0, false));
            old_ids.push(board(&mut game, "XQ12", 3, 2, false));
            let c = game.regions[2].cards.last_mut().unwrap();
            c.exhausted = true;
            c.damage = 1;
            c.wounds = 1;
            c.shield = 1;
            // XQ12 has printed defense 1. Two actual friendly auras keep this
            // wounded/damaged role alive during the passes before Kyoto resolves.
            board(&mut game, "JC059", 2, 2, false);
            board(&mut game, "JC059", 3, 3, false);
            old_ids.push(board(&mut game, "JC003", 1, 1, true));
        }
        "DQJC110" => {
            for seat in 0..4 {
                old_ids.push(board(&mut game, "JC125", seat, seat, false));
            }
            old_ids.push(board(&mut game, "JC003", 0, 0, true));
        }
        "DQJC111" => {
            for seat in 0..4 {
                board(&mut game, "JC125", seat, seat, false);
            }
            board(&mut game, "JC059", 2, 2, false);
        }
        "DQJC115" => {
            for seat in 0..4 {
                let mut c = game.make_card("JC125", seat);
                c.exhausted = true;
                c.damage = 7;
                old_ids.push(c.id.clone());
                game.players[seat].graveyard.push(c);
            }
        }
        "DQJC116" => {}
        _ => unreachable!(),
    }
    if generic_search {
        for seat in 0..4 {
            game.players[seat].deck.clear();
            for definition in ["JC003", "JC125"] {
                let c = game.make_card(definition, seat);
                game.players[seat].deck.push(c);
            }
        }
        // Deliberately synthetic declaration of the shared operation using real
        // registered characters. This is not the Hong Kong attachment binding.
        let source_card = game.regions[0].card.clone();
        let replacement = game.world.remove(0);
        game.regions[0].card = replacement;
        let mut scored = source_card;
        scored.owner = actor;
        scored.controller = actor;
        game.players[actor].score_cards.push(scored.clone());
        let mut ability = rules::definition(id).abilities[0].clone();
        ability.ops = vec![Op::SimultaneousSearch {
            filter: CardFilter::Kind("character".into()),
        }];
        game.effects.push_back(Effect::Declare {
            declaration: Declaration {
                actor,
                source: SourceSnapshot {
                    card: scored,
                    region: Some(0),
                    play_source: None,
                },
                ability,
            },
        });
        game.window = Some(Window::After(0, 2));
    }
    fn distinct_world(g: &Game) {
        let world = g
            .regions
            .iter()
            .map(|r| &r.card)
            .chain(g.world.iter())
            .chain(g.players.iter().flat_map(|p| p.score_cards.iter()))
            .filter(|c| catalog::card(&c.definition).kind == "region")
            .collect::<Vec<_>>();
        assert_eq!(world.len(), 10);
        assert_eq!(
            world
                .iter()
                .map(|c| &c.definition)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            10
        );
    }
    distinct_world(&game);
    let initial_rng = game.random;
    let initial_hands = game
        .players
        .iter()
        .map(|p| p.hand.len())
        .collect::<Vec<_>>();
    let initial_decks = game
        .players
        .iter()
        .map(|p| p.deck.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let mut room = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        0,
    )];
    let mut won = false;
    let mut started = false;
    let mut commitments: Vec<String> = vec![];
    let mut search_rng = 0;
    let mut search_logs = 0;
    for _ in 0..28 {
        if won && started && room.pending.is_none() && room.stack.is_empty() {
            break;
        }
        if let Some(pending) = room.pending.clone() {
            let (seat, mut action) = pick_choice(&room);
            match &pending.resolution {
                ChoiceResolution::Declare { declaration, .. } => {
                    assert_eq!(declaration.source.card.definition, id);
                    assert_eq!(seat, actor);
                    action.selected = Some(vec!["accept".into()]);
                    won = true;
                    started = true;
                }
                ChoiceResolution::Frame {
                    choice: FrameChoice::SacrificeDraw { .. },
                    ..
                } => {
                    action.selected = Some(vec![pending
                        .choice
                        .options
                        .iter()
                        .find(|o| o.card.as_ref().unwrap().card_id.as_deref() == Some("JC125"))
                        .unwrap()
                        .id
                        .clone()]);
                }
                ChoiceResolution::Frame {
                    choice: FrameChoice::Region,
                    ..
                } => {
                    action.selected = Some(vec!["region:4".into()]);
                }
                ChoiceResolution::Frame {
                    choice: FrameChoice::SimultaneousSearch { .. },
                    ..
                } => {
                    assert!(generic_search);
                    assert_eq!(
                        room.game
                            .players
                            .iter()
                            .map(|p| p.hand.len())
                            .collect::<Vec<_>>(),
                        initial_hands
                    );
                    assert_eq!(
                        room.game
                            .players
                            .iter()
                            .map(|p| p.deck.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
                            .collect::<Vec<_>>(),
                        initial_decks
                    );
                    if commitments.is_empty() {
                        search_rng = room.random;
                        search_logs = room.log.len();
                    } else {
                        assert_eq!(room.random, search_rng);
                        assert_eq!(room.log.len(), search_logs);
                    }
                    for prior in &commitments {
                        for viewer in 0..4 {
                            assert!(!serde_json::to_string(
                                &room.view(viewer, room.pacing.last_server_now_ms)
                            )
                            .unwrap()
                            .contains(prior.as_str()));
                        }
                    }
                    action.selected = if seat == 1 {
                        Some(vec![])
                    } else {
                        let id = pending.choice.options[0].id.clone();
                        commitments.push(id.clone());
                        Some(vec![id])
                    };
                }
                _ => {}
            }
            for other in 0..4 {
                if other != seat {
                    assert!(room
                        .view(other, room.pacing.last_server_now_ms)
                        .pending_choice
                        .is_none());
                }
            }
            assert!(room.pacing.window.is_none());
            apply_game(&mut room, &mut steps, seat, action);
        } else {
            let seat = room
                .players
                .iter()
                .find(|p| {
                    !p.eliminated
                        && room.team(p.seat) == room.priority_team
                        && !room.passed.contains(&p.seat)
                })
                .unwrap()
                .seat;
            apply_game(&mut room, &mut steps, seat, Action::new("pass"));
        }
    }
    assert!(won && started && room.pending.is_none() && room.stack.is_empty());
    distinct_world(&room);
    match id {
        "DQJC108" => {
            assert_eq!(
                room.regions[2]
                    .cards
                    .iter()
                    .filter(|c| !c.face_down)
                    .count(),
                4
            );
            assert!(room.players.iter().all(|p| p.assets.is_empty()));
            assert!(
                room.regions[2]
                    .cards
                    .iter()
                    .find(|c| c.controller == 0)
                    .unwrap()
                    .exhausted
            );
            assert!(room.regions[2]
                .cards
                .iter()
                .all(|c| !old_ids.contains(&c.id)));
        }
        "DQJC109" => {
            assert!(room.regions[2]
                .cards
                .iter()
                .find(|c| c.id == old_ids[0])
                .is_some_and(|c| !c.face_down));
            let c = room.regions[2]
                .cards
                .iter()
                .find(|c| c.definition == "XQ12")
                .unwrap();
            assert!(c.face_down && c.exhausted);
            assert_eq!((c.damage, c.wounds, c.shield), (0, 0, 0));
            assert_ne!(c.id, old_ids[1]);
            assert!(room.regions[2]
                .cards
                .iter()
                .any(|c| c.id == old_ids[2] && c.face_down));
        }
        "DQJC110" => {
            assert_eq!(room.regions[2].cards.len(), 5);
            assert!(room.regions[2]
                .cards
                .iter()
                .all(|c| c.face_down && !old_ids.contains(&c.id)));
            assert_ne!(room.random, initial_rng);
        }
        "DQJC111" => {
            assert_eq!(
                room.players
                    .iter()
                    .map(|p| p.hand.len())
                    .collect::<Vec<_>>(),
                vec![1, 1, 2, 2]
            );
            assert!(room
                .players
                .iter()
                .all(|p| p.graveyard.iter().any(|c| c.definition == "JC125")));
        }
        "DQJC115" => {
            assert_eq!(room.regions[4].cards.len(), 4);
            assert!(room.players.iter().all(|p| p.graveyard.is_empty()));
            assert!(room.regions[4].cards.iter().all(|c| !c.face_down
                && !c.exhausted
                && c.damage == 0
                && !old_ids.contains(&c.id)));
        }
        "DQJC116" if generic_search => {
            assert_eq!(
                room.players
                    .iter()
                    .map(|p| p.hand.len())
                    .collect::<Vec<_>>(),
                vec![1, 0, 1, 1]
            );
            assert_eq!(
                room.log
                    .iter()
                    .filter(|l| l.text.contains("同时展示检索"))
                    .count(),
                3
            );
        }
        "DQJC116" => {
            assert_eq!(
                room.players
                    .iter()
                    .map(|p| p.hand.len())
                    .collect::<Vec<_>>(),
                initial_hands
            );
            assert_eq!(
                room.players
                    .iter()
                    .map(|p| p.deck.len())
                    .collect::<Vec<_>>(),
                initial_decks.iter().map(|d| d.len()).collect::<Vec<_>>()
            );
        }
        _ => unreachable!(),
    }
    assert!(steps.len() <= 30);
    json!({"name":if generic_search {"generic-simultaneous-character-search-not-hong-kong-binding".into()}else{format!("real-world-win-trigger-{id}")},"seed":seed,"steps":steps,"syntheticInitialLayout":true,"genericOperationOnly":generic_search})
}

fn attachment_initial(mode: &str, room_id: &str) -> Game {
    let mut game = Game::new(
        room_id.into(),
        "ATTACH".into(),
        mode.into(),
        "玩家0".into(),
        "hunters".into(),
        9007199254740993,
    )
    .unwrap();
    for seat in 1..game.capacity() {
        game.join(format!("玩家{seat}"), "hunters".into()).unwrap();
    }
    for p in &mut game.players {
        p.ready = true;
    }
    game.apply(0, Action::new("start")).unwrap();
    while game.pending.is_some() {
        let (seat, a) = pick_choice(&game);
        game.apply(seat, a).unwrap();
    }
    for p in &mut game.players {
        p.hand.clear();
        p.assets.clear();
    }
    for r in &mut game.regions {
        r.cards.clear();
    }
    game.first_team = 0;
    game.active_team = 0;
    game.priority_team = 0;
    game.window = Some(Window::Action(0));
    game.passed.clear();
    game.team_passed = [false; 2];
    game
}
fn attachment_pass_top(room: &mut RoomEnvelope, steps: &mut Vec<Value>) {
    let n = room.game.stack.len();
    assert!(n > 0);
    while room.game.stack.len() >= n && room.pending.is_none() {
        let seat = (0..room.players.len())
            .find(|s| room.team(*s) == room.priority_team && !room.passed.contains(s))
            .unwrap();
        apply_game(room, steps, seat, Action::new("pass"));
    }
}
fn attachment_fixture(cancel: bool) -> Value {
    // Only the starting layout is synthetic; the complete play/response/cleanup
    // sequence is expressed as public, legal session commands.
    let mut game = attachment_initial(
        "teams",
        if cancel {
            "666666666666666666666661"
        } else {
            "666666666666666666666660"
        },
    );
    let host = game.make_card("LC22", 2);
    let host_id = host.id.clone();
    game.regions[4].cards.push(host);
    let mut hidden = game.make_card("LC22", 3);
    hidden.face_down = true;
    game.regions[3].cards.push(hidden);
    let equipment = game.make_card("BQ022", 0);
    let equipment_id = equipment.id.clone();
    game.players[0].hand.push(equipment);
    let chase = game.make_card("JC063", 0);
    let chase_id = chase.id.clone();
    game.players[0].hand.push(chase);
    for _ in 0..3 {
        let c = game.make_card("JC063", 0);
        game.players[0].assets.push(c);
    }
    let murder = game.make_card("JC091", 2);
    let murder_id = murder.id.clone();
    game.players[2].hand.push(murder);
    for _ in 0..3 {
        let c = game.make_card("JC091", 2);
        game.players[2].assets.push(c);
    }
    let mut room = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        0,
    )];
    apply_game(
        &mut room,
        &mut steps,
        0,
        Action {
            card_id: Some(equipment_id),
            target_id: Some(host_id.clone()),
            ..Action::new("play")
        },
    );
    assert!(room.attachments.is_empty());
    if cancel {
        for seat in 0..2 {
            apply_game(&mut room, &mut steps, seat, Action::new("pass"));
        }
        apply_game(
            &mut room,
            &mut steps,
            2,
            Action {
                card_id: Some(murder_id),
                target_id: Some(host_id.clone()),
                ..Action::new("play")
            },
        );
        attachment_pass_top(&mut room, &mut steps);
        assert!(room.regions[4].cards.is_empty());
        assert_eq!(room.game.stack.len(), 1);
        attachment_pass_top(&mut room, &mut steps);
        assert!(room.attachments.is_empty());
        assert_eq!(
            room.players[0]
                .graveyard
                .iter()
                .filter(|c| c.definition == "BQ022")
                .count(),
            1
        );
        assert_eq!(
            room.players[0]
                .assets
                .iter()
                .filter(|c| c.exhausted)
                .count(),
            1
        );
    } else {
        attachment_pass_top(&mut room, &mut steps);
        assert_eq!(room.attachments.len(), 1);
        assert_eq!(
            room.view(1, room.pacing.last_server_now_ms).regions[4].icons_by_team[1].combat,
            2
        );
        for viewer in 0..4 {
            let v = room.view(viewer, room.pacing.last_server_now_ms);
            assert_eq!(v.attachments[0].host_id, host_id);
            assert_eq!(v.attachments[0].card.region, Some(4));
        }
        let old = room.attachments[0].card.id.clone();
        apply_game(
            &mut room,
            &mut steps,
            0,
            Action {
                card_id: Some(chase_id),
                target_id: Some(host_id.clone()),
                option: Some("hide".into()),
                ..Action::new("play")
            },
        );
        attachment_pass_top(&mut room, &mut steps);
        assert!(room.attachments.is_empty());
        assert!(room.game.stack.is_empty());
        assert!(room.pending.is_none());
        let returned = room.players[0]
            .hand
            .iter()
            .find(|c| c.definition == "BQ022")
            .unwrap();
        assert_ne!(returned.id, old);
        assert!(room.players[2].hand.iter().all(|c| c.definition != "BQ022"));
    }
    assert!(steps.len() < 30);
    json!({"name":if cancel{"attachment-target-murder-response-cancelled"}else{"attachment-enemy-host-and-hide-owner-recycle"},"seed":"9007199254740993","steps":steps,"syntheticInitialLayout":true})
}
fn attachment_hk_fixture() -> Value {
    let mut game = attachment_initial("teams", "666666666666666666666662");
    if let Some(at) = game
        .regions
        .iter()
        .position(|r| r.card.definition == "DQJC116")
    {
        game.regions.swap(0, at);
    } else {
        let at = game
            .world
            .iter()
            .position(|c| c.definition == "DQJC116")
            .unwrap();
        std::mem::swap(&mut game.regions[0].card, &mut game.world[at]);
    }
    game.window = Some(Window::Win(0, 0));
    for seat in 0..4 {
        game.players[seat].deck.clear();
        for _ in 0..2 {
            let c = game.make_card("BQ022", seat);
            game.players[seat].deck.push(c);
        }
        for _ in 0..4 {
            let c = game.make_card("JC125", seat);
            game.players[seat].deck.push(c);
        }
    }
    let original = game
        .players
        .iter()
        .map(|p| p.deck.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let mut room = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        0,
    )];
    for seat in 0..4 {
        apply_game(&mut room, &mut steps, seat, Action::new("pass"));
    }
    let p = room.pending.clone().unwrap();
    apply_game(
        &mut room,
        &mut steps,
        p.seat,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec!["accept".into()]),
            ..Action::new("choose")
        },
    );
    attachment_pass_top(&mut room, &mut steps);
    for seat in 0..4 {
        let p = room.pending.clone().unwrap();
        assert_eq!(p.seat, seat);
        assert_eq!(p.choice.options.len(), 2);
        assert!(p
            .choice
            .options
            .iter()
            .all(|o| o.card.as_ref().unwrap().card_id.as_deref() == Some("BQ022")));
        for other in 0..4 {
            if other != seat {
                assert!(room
                    .view(other, room.pacing.last_server_now_ms)
                    .pending_choice
                    .is_none());
            }
        }
        assert!(room.pacing.window.is_none());
        let selected = if seat == 1 {
            vec![]
        } else {
            vec![p.choice.options[0].id.clone()]
        };
        apply_game(
            &mut room,
            &mut steps,
            seat,
            Action {
                choice_id: Some(p.choice.id),
                selected: Some(selected),
                ..Action::new("choose")
            },
        );
        if seat < 3 {
            assert!(room.players.iter().all(|p| p.hand.is_empty()));
            assert_eq!(
                room.players
                    .iter()
                    .map(|p| p.deck.iter().map(|c| c.id.clone()).collect::<Vec<_>>())
                    .collect::<Vec<_>>(),
                original
            );
            assert!(room.log.iter().all(|l| !l.text.contains("同时展示检索")));
        }
    }
    assert_eq!(
        room.players
            .iter()
            .map(|p| p.hand.len())
            .collect::<Vec<_>>(),
        vec![1, 0, 1, 1]
    );
    assert_eq!(
        room.log
            .iter()
            .filter(|l| l.text.contains("同时展示检索") && l.text.contains("合金指虎"))
            .count(),
        3
    );
    assert!(steps.len() < 30);
    json!({"name":"actual-hong-kong-nonempty-attachment-search","seed":"9007199254740993","steps":steps,"syntheticInitialLayout":true,"genericOperationOnly":false})
}
fn attachment_region_return_fixture() -> Value {
    let mut game = attachment_initial("teams", "666666666666666666666663");
    for seat in 0..2 {
        let host = game.make_card("LC22", seat);
        let host_id = host.id.clone();
        game.regions[0].cards.push(host);
        let equipment = game.make_card("BQ022", 1 - seat);
        game.attachments.push(hegemony_server::model::Attachment {
            card: equipment,
            host_id,
        });
    }
    let aura = game.make_card("JC059", 0);
    let aura_id = aura.id.clone();
    game.regions[0].cards.push(aura);
    let mut patient = game.make_card("JC125", 1);
    patient.damage = 1;
    let patient_id = patient.id.clone();
    game.regions[0].cards.push(patient);
    for (seat, definitions) in [(2, ["LC21", "LC20"]), (3, ["JC125", "LC23"])] {
        for (index, definition) in definitions.into_iter().enumerate() {
            let mut c = game.make_card(definition, seat);
            c.face_down = index == 0;
            if seat == 2 && index == 0 {
                c.damage = 7;
            }
            game.regions[0].cards.push(c);
        }
    }
    let old_definitions = game.regions[0]
        .cards
        .iter()
        .chain(game.attachments.iter().map(|a| &a.card))
        .map(|c| (c.id.clone(), c.definition.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let old_ids = old_definitions
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let old_prefixes = game
        .players
        .iter()
        .map(|p| serde_json::to_string(&p.deck).unwrap())
        .collect::<Vec<_>>();
    let mut expected_bottoms = vec![Vec::<String>::new(); 4];
    let old_decks = game
        .players
        .iter()
        .map(|p| p.deck.len())
        .collect::<Vec<_>>();
    game.window = Some(Window::Win(0, 0));
    let mut room = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        0,
    )];
    for seat in 0..4 {
        apply_game(&mut room, &mut steps, seat, Action::new("pass"));
    }
    for seat in 0..4 {
        let p = room.pending.clone().unwrap();
        assert_eq!(p.seat, seat);
        assert_eq!(p.choice.options.len(), if seat < 2 { 3 } else { 2 });
        expected_bottoms[seat] = p
            .choice
            .options
            .iter()
            .rev()
            .map(|o| old_definitions[&o.id].clone())
            .collect();
        apply_game(
            &mut room,
            &mut steps,
            seat,
            Action {
                choice_id: Some(p.choice.id),
                bottom: Some(
                    p.choice
                        .options
                        .iter()
                        .rev()
                        .map(|o| o.id.clone())
                        .collect(),
                ),
                ..Action::new("choose")
            },
        );
        if seat < 3 {
            assert!(room.regions[0].cards.iter().any(|c| c.id == aura_id));
            assert!(room.regions[0].cards.iter().any(|c| c.id == patient_id));
            assert_eq!(room.attachments.len(), 2);
            assert_eq!(
                room.players
                    .iter()
                    .map(|p| p.deck.len())
                    .collect::<Vec<_>>(),
                old_decks
            );
        }
    }
    assert!(room.region_return.is_none() && room.attachments.is_empty());
    let mut new_ids = std::collections::BTreeSet::new();
    for seat in 0..4 {
        assert_eq!(
            room.players[seat].deck.len(),
            old_decks[seat] + expected_bottoms[seat].len()
        );
        assert_eq!(
            serde_json::to_string(&room.players[seat].deck[..old_decks[seat]]).unwrap(),
            old_prefixes[seat]
        );
        let returned = &room.players[seat].deck[old_decks[seat]..];
        assert_eq!(
            returned
                .iter()
                .map(|c| c.definition.clone())
                .collect::<Vec<_>>(),
            expected_bottoms[seat]
        );
        for c in returned {
            assert!(!old_ids.contains(&c.id) && new_ids.insert(c.id.clone()));
            assert_eq!((c.owner, c.controller), (seat, seat));
            assert!(!c.exhausted && !c.face_down);
            assert_eq!((c.damage, c.wounds, c.shield), (0, 0, 0));
        }
        assert!(room.players[seat].hand.is_empty() && room.players[seat].graveyard.is_empty());
    }
    assert!(room.log.iter().all(|l| !l.text.contains("死亡")));
    json!({"name":"won-region-batch-cross-owner-equipment-and-aura","seed":"9007199254740993","steps":steps,"syntheticInitialLayout":true})
}
fn grave_play_fixture() -> Value {
    let mut game = attachment_initial("teams", "777777777777777777777770");
    let corpse = game.make_card("JC085", 0);
    let old = corpse.id.clone();
    game.players[0].graveyard.push(corpse);
    for id in ["JC085", "JC084"] {
        let c = game.make_card(id, 0);
        game.players[0].assets.push(c);
    }
    let street = game.make_card("JC084", 1);
    game.regions[2].cards.push(street);
    game.regions[2].influence = [1, 1];
    let mut room = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        0,
    )];
    apply_game(
        &mut room,
        &mut steps,
        0,
        Action {
            card_id: Some(old.clone()),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    assert!(room.players[0].hand.is_empty() && room.players[0].graveyard.is_empty());
    assert_eq!(
        room.stack[0].frame.as_ref().unwrap().source.play_source,
        Some(hegemony_server::model::PlaySource::Graveyard)
    );
    assert_eq!(
        room.players[0]
            .assets
            .iter()
            .filter(|c| c.exhausted)
            .count(),
        2
    );
    attachment_pass_top(&mut room, &mut steps);
    let returned = room.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC085")
        .unwrap();
    assert_ne!(returned.id, old);
    assert!(!returned.face_down);
    assert_eq!(
        room.view(0, room.pacing.last_server_now_ms).regions[2]
            .characters
            .iter()
            .find(|c| c.card_id.as_deref() == Some("JC085"))
            .unwrap()
            .icons
            .unwrap()
            .influence,
        1
    );
    // The teammate's street thug uses team markers, while asset domains remain personal.
    assert_eq!(
        room.view(1, room.pacing.last_server_now_ms).regions[2]
            .characters
            .iter()
            .find(|c| c.card_id.as_deref() == Some("JC084"))
            .unwrap()
            .icons
            .unwrap()
            .investigation,
        1
    );
    json!({"name":"direct-grave-face-up-play-normal-cost-and-conditional-icons","seed":"9007199254740993","steps":steps,"syntheticInitialLayout":true})
}
fn assassin_fixture() -> Value {
    let mut game = attachment_initial("teams", "888888888888888888888880");
    let mut source = game.make_card("JC088", 0);
    source.face_down = true;
    let old_source = source.id.clone();
    game.regions[2].cards.push(source);
    let victim = game.make_card("JC084", 2);
    let target = victim.id.clone();
    game.regions[2].cards.push(victim);
    let expensive = game.make_card("JC086", 3);
    let high = expensive.id.clone();
    game.regions[2].cards.push(expensive);
    let mut hidden = game.make_card("JC085", 1);
    hidden.face_down = true;
    game.regions[2].cards.push(hidden);
    for _ in 0..5 {
        let c = game.make_card("JC088", 0);
        game.players[0].assets.push(c);
    }
    let mut room = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        0,
    )];
    apply_game(
        &mut room,
        &mut steps,
        0,
        Action {
            card_id: Some(old_source.clone()),
            ..Action::new("reveal")
        },
    );
    attachment_pass_top(&mut room, &mut steps);
    let pending = room.pending.clone().unwrap();
    assert_eq!(
        pending
            .choice
            .options
            .iter()
            .map(|o| o.id.as_str())
            .collect::<Vec<_>>(),
        vec![target.as_str()]
    );
    apply_game(
        &mut room,
        &mut steps,
        0,
        Action {
            choice_id: Some(pending.choice.id),
            selected: Some(vec![target.clone()]),
            ..Action::new("choose")
        },
    );
    assert_eq!(
        room.stack.last().unwrap().frame.as_ref().unwrap().targets[0]
            .spec
            .printed_cost_max,
        Some(2)
    );
    attachment_pass_top(&mut room, &mut steps);
    assert!(!room.regions[2].cards.iter().any(|c| c.id == target));
    assert!(room.regions[2].cards.iter().any(|c| c.id == high));
    let face_up = room.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC088")
        .unwrap()
        .id
        .clone();
    assert_ne!(face_up, old_source);
    apply_game(
        &mut room,
        &mut steps,
        0,
        Action {
            card_id: Some(face_up.clone()),
            ability_id: Some("hide-self".into()),
            ..Action::new("activate")
        },
    );
    assert!(room.players[0].assets.iter().all(|c| c.exhausted));
    attachment_pass_top(&mut room, &mut steps);
    assert!(!room.regions[2].cards.iter().any(|c| c.id == face_up));
    assert!(room.regions[2]
        .cards
        .iter()
        .any(|c| c.definition == "JC088" && c.face_down));
    json!({"name":"printed-cost-reveal-guard-and-paid-self-hide","seed":"9007199254740993","steps":steps,"syntheticInitialLayout":true})
}
fn two_card_fixture() -> Value {
    // Explicit finite initial fixture: controller differs from owner. All later
    // transitions are ordinary authenticated-equivalent public session commands.
    let mut game = attachment_initial("teams", "888888888888888888888882");
    let mut owl = game.make_card("JC001", 2);
    owl.controller = 1;
    owl.face_down = true;
    let old_owl = owl.id.clone();
    game.regions[2].cards.push(owl);
    for n in 0..2 {
        let mut asset = game.make_card("JC003", 1);
        asset.exhausted = n == 0;
        game.players[1].assets.push(asset);
    }
    let skeleton = game.make_card("BQ083", 0);
    let skeleton_id = skeleton.id.clone();
    game.players[0].hand.push(skeleton);
    for _ in 0..4 {
        let asset = game.make_card("JC085", 0);
        game.players[0].assets.push(asset);
    }
    let outside = game.make_card("JC125", 3);
    let outside_id = outside.id.clone();
    game.regions[3].cards.push(outside);
    let mut room = RoomEnvelope::from_game(game);
    let mut steps = vec![step(
        &room,
        "initialFixture",
        json!([serde_json::to_string(&room).unwrap()]),
        1,
    )];
    for viewer in [0, 2, 3] {
        assert!(
            room.view(viewer, room.pacing.last_server_now_ms).regions[2].characters[0]
                .card_id
                .is_none()
        );
    }
    assert_eq!(
        room.view(1, room.pacing.last_server_now_ms).regions[2].characters[0]
            .card_id
            .as_deref(),
        Some("JC001")
    );
    apply_game(
        &mut room,
        &mut steps,
        1,
        Action {
            card_id: Some(old_owl.clone()),
            ..Action::new("reveal")
        },
    );
    attachment_pass_top(&mut room, &mut steps);
    let revealed = room.regions[2]
        .cards
        .iter()
        .find(|c| c.definition == "JC001")
        .unwrap()
        .clone();
    assert_ne!(revealed.id, old_owl);
    assert!(room.players[1].assets.iter().all(|c| c.exhausted));
    assert_eq!(room.icons(&revealed, 2).investigation, 1);
    assert_eq!(room.icons(&revealed, 2).influence, 1);
    apply_game(
        &mut room,
        &mut steps,
        0,
        Action {
            card_id: Some(skeleton_id),
            region: Some(2),
            ..Action::new("deploy")
        },
    );
    attachment_pass_top(&mut room, &mut steps);
    let p = room.pending.clone().unwrap();
    assert!(p.choice.options.iter().any(|o| o.id == revealed.id));
    assert!(!p.choice.options.iter().any(|o| o.id == outside_id));
    assert!(room.players[0].assets.iter().all(|c| c.exhausted));
    apply_game(
        &mut room,
        &mut steps,
        0,
        Action {
            choice_id: Some(p.choice.id),
            selected: Some(vec![revealed.id.clone()]),
            ..Action::new("choose")
        },
    );
    attachment_pass_top(&mut room, &mut steps);
    assert!(!room.regions[2].cards.iter().any(|c| c.id == revealed.id));
    assert!(room.players[2]
        .graveyard
        .iter()
        .any(|c| c.definition == "JC001"));
    assert!(room.players[1]
        .graveyard
        .iter()
        .all(|c| c.definition != "JC001"));
    assert!(room.regions[3].cards.iter().any(|c| c.id == outside_id));
    json!({"name":"owl-controller-mind-two-and-skeleton-paid-entry","seed":"9007199254740993","steps":steps,"syntheticInitialLayout":true})
}
fn main() {
    let output = std::env::args()
        .nth(1)
        .expect("Usage: native_fixtures <output.json>");
    let mut previous = Game::new(
        "previous-patch".into(),
        "OLD".into(),
        "duel".into(),
        "P0".into(),
        "watchers".into(),
        1,
    )
    .unwrap();
    previous.versions.engine = "rust-v0.2.0".into();
    previous.versions.card_pool = "limited-v2".into();
    let previous_state = serde_json::to_string(&previous).unwrap();
    assert!(RoomEnvelope::from_persisted(&previous_state).is_err());
    let mut rejected_states = vec![previous_state];
    for engine in ["rust-v0.2.1", "rust-v0.2.2"] {
        previous.versions.engine = engine.into();
        previous.versions.card_pool = "limited-v2.1".into();
        let state = serde_json::to_string(&previous).unwrap();
        assert!(RoomEnvelope::from_persisted(&state).is_err());
        rejected_states.push(state);
    }
    previous.versions.engine = "rust-v0.2.3".into();
    previous.versions.card_pool = "limited-v2.2".into();
    let state = serde_json::to_string(&previous).unwrap();
    assert!(RoomEnvelope::from_persisted(&state).is_err());
    rejected_states.push(state);
    previous.versions.engine = "rust-v0.2.4".into();
    let state = serde_json::to_string(&previous).unwrap();
    assert!(RoomEnvelope::from_persisted(&state).is_err());
    rejected_states.push(state);
    let old_room = {
        let mut old = RoomEnvelope::from_game(previous.clone());
        old.versions.engine = "rust-v0.2.5".into();
        old.versions.card_pool = "limited-v2.3".into();
        old.game.versions = old.versions.clone();
        serde_json::to_string(&old).unwrap()
    };
    assert!(RoomEnvelope::from_persisted(&old_room).is_err());
    rejected_states.push(old_room);
    let previous_six = {
        let mut old = RoomEnvelope::from_game(previous.clone());
        old.versions.engine = "rust-v0.2.6".into();
        old.versions.card_pool = "limited-v2.4".into();
        old.game.versions = old.versions.clone();
        serde_json::to_string(&old).unwrap()
    };
    assert!(RoomEnvelope::from_persisted(&previous_six).is_err());
    rejected_states.push(previous_six);
    let previous_seven = {
        let mut old = RoomEnvelope::from_game(previous.clone());
        old.versions.engine = "rust-v0.2.7".into();
        old.versions.card_pool = "limited-v2.5".into();
        old.game.versions = old.versions.clone();
        serde_json::to_string(&old).unwrap()
    };
    assert!(RoomEnvelope::from_persisted(&previous_seven).is_err());
    rejected_states.push(previous_seven);
    let clock = pacing_fixture(false);
    let prepared = json!({"state":clock["steps"][0]["state"],"version":clock["steps"][0]["version"],"roomId":"ffffffffffffffffffffffff","firstAction":clock["steps"][1]["args"][1]["action"],"firstCommand":clock["steps"][1]["args"][1],"serverNowMs":clock["steps"][1]["args"][2],"seat":0});
    if std::env::args().any(|a| a == "--slice-v027") {
        let value = json!({"catalog":catalog::catalog(),"cases":[grave_play_fixture(),attachment_region_return_fixture(),clock],"preparedResponse":prepared,"rejectedStates":rejected_states});
        std::fs::write(output, serde_json::to_vec(&value).unwrap()).unwrap();
        return;
    }
    if std::env::args().any(|a| a == "--slice-v026") {
        let value = json!({"catalog":catalog::catalog(),"cases":[attachment_fixture(false),attachment_fixture(true),attachment_hk_fixture(),attachment_region_return_fixture(),clock],"preparedResponse":prepared,"rejectedStates":rejected_states});
        std::fs::write(output, serde_json::to_vec(&value).unwrap()).unwrap();
        return;
    }
    let two_cards = two_card_fixture();
    let two_initial: RoomEnvelope =
        serde_json::from_str(two_cards["steps"][0]["state"].as_str().unwrap()).unwrap();
    let prepared_two_cards = json!({
        "syntheticInitialLayout":true,"roomId":two_initial.room_id,"version":two_initial.revision,
        "state":two_cards["steps"][0]["state"],"firstAction":two_cards["steps"][1]["args"][1]["action"],
        "oldOwlId":two_initial.regions[2].cards[0].id,
        "skeletonId":two_initial.players[0].hand[0].id,
        "outsideId":two_initial.regions[3].cards[0].id,
    });
    let assassin = assassin_fixture();
    let initial: RoomEnvelope =
        serde_json::from_str(assassin["steps"][0]["state"].as_str().unwrap()).unwrap();
    let targets = &initial.regions[2].cards;
    let prepared_assassin = json!({
        "syntheticInitialLayout":true,"roomId":initial.room_id,"version":initial.revision,
        "state":assassin["steps"][0]["state"],"firstAction":assassin["steps"][1]["args"][1]["action"],
        "oldSourceId":targets.iter().find(|c| c.definition == "JC088").unwrap().id,
        "targetId":targets.iter().find(|c| c.definition == "JC084").unwrap().id,
        "expensiveTargetId":targets.iter().find(|c| c.definition == "JC086").unwrap().id
    });
    let value = json!({"preparedTwoCards":prepared_two_cards,"preparedAssassin":prepared_assassin,"preparedResponse":prepared,"catalog":catalog::catalog(),"cases":[two_cards,assassin,attachment_fixture(false),attachment_fixture(true),attachment_hk_fixture(),attachment_region_return_fixture(),grave_play_fixture(),fixture("duel","18446744073709551615"),fixture("teams","9007199254740993"),response_fixture(),detective_fixture(false),detective_fixture(true),custom_deck_fixture(),friendly_icons_fixture(),pacing_fixture(false),pacing_fixture(true),world_fixture("DQJC108",false),world_fixture("DQJC109",false),world_fixture("DQJC110",false),world_fixture("DQJC111",false),world_fixture("DQJC115",false),world_fixture("DQJC116",false),world_fixture("DQJC116",true)],"rejectedStates":rejected_states});
    std::fs::write(output, serde_json::to_vec(&value).unwrap()).unwrap();
}
