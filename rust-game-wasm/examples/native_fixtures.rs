//! Native oracle for the WASM ABI test; private state remains an opaque string in JS.
use hegemony_server::{
    catalog,
    model::{Action, ChoiceResolution, FrameChoice, Game, Window},
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn step(game: &Game, operation: &str, args: Value, seat: usize) -> Value {
    json!({"operation":operation,"args":args,"state":serde_json::to_string(game).unwrap(),"version":game.version,"seat":seat,"view":game.view(seat),"views":(0..game.players.len()).map(|s|game.view(s)).collect::<Vec<_>>()})
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
    let mut steps = vec![step(
        &game,
        "initialFixture",
        json!([serde_json::to_string(&game).unwrap()]),
        0,
    )];
    let mut apply = |game: &mut Game, seat: usize, action: Action| {
        game.apply(seat, action.clone()).unwrap();
        steps.push(step(game, "apply", json!([seat, action]), seat));
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
    let mut rejected = game.clone();
    assert!(rejected.apply(0, heal.clone()).is_err());
    json!({"name":"real-response-frame-continuation-and-heal-cost","seed":seed,"steps":steps,"rejectedActions":[[0,heal]]})
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
    let mut steps = vec![step(
        &game,
        "initialFixture",
        json!([serde_json::to_string(&game).unwrap()]),
        0,
    )];
    let mut apply = |game: &mut Game, seat: usize, action: Action| {
        game.apply(seat, action.clone()).unwrap();
        steps.push(step(game, "apply", json!([seat, action]), seat));
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
    game = Game::from_persisted(&serde_json::to_string(&game).unwrap()).unwrap();
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
        assert_eq!(game.view(0).stack[0].target_summaries[0].status, "missing");
    }
    for seat in [1, 0] {
        apply(&mut game, seat, Action::new("pass"));
    }
    assert_eq!(game.stack.len(), 1);
    game = Game::from_persisted(&serde_json::to_string(&game).unwrap()).unwrap();
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
    json!({"name":if kill_source {"detective-source-death-independent-hidden-destroy"} else {"detective-target-reveal-invalidates-original-instance"},"seed":seed,"steps":steps,"rejectedActions":[[0,choose],[0,Action{card_id:Some(detective_id),..Action::new("reveal")}]]})
}
fn fixture(mode: &str, seed: &str) -> Value {
    let mut game = Game::new(
        format!("wasm-{mode}"),
        "TEST-INVITE".into(),
        mode.into(),
        "玩家0".into(),
        "watchers".into(),
        seed.parse().unwrap(),
    )
    .unwrap();
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
        game.join(name.clone(), deck.into()).unwrap();
        steps.push(step(&game, "joinGame", json!([name, deck]), seat));
    }
    for seat in 0..game.players.len() {
        let action = Action::new("ready");
        game.apply(seat, action.clone()).unwrap();
        steps.push(step(&game, "apply", json!([seat, action]), seat));
    }
    let action = Action::new("start");
    game.apply(0, action.clone()).unwrap();
    steps.push(step(&game, "apply", json!([0, action]), 0));
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
        game.apply(seat, action.clone()).unwrap();
        steps.push(step(&game, "apply", json!([seat, action]), seat));
    }
    json!({"name":mode,"seed":seed,"steps":steps})
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
    assert!(Game::from_persisted(&previous_state).is_err());
    let mut rejected_states = vec![previous_state];
    for engine in ["rust-v0.2.1", "rust-v0.2.2"] {
        previous.versions.engine = engine.into();
        previous.versions.card_pool = "limited-v2.1".into();
        let state = serde_json::to_string(&previous).unwrap();
        assert!(Game::from_persisted(&state).is_err());
        rejected_states.push(state);
    }
    let value = json!({"catalog":catalog::catalog(),"cases":[fixture("duel","18446744073709551615"),fixture("teams","9007199254740993"),response_fixture(),detective_fixture(false),detective_fixture(true)],"rejectedStates":rejected_states});
    std::fs::write(output, serde_json::to_vec(&value).unwrap()).unwrap();
}
