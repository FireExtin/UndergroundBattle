//! Native oracle for the WASM ABI test; private state remains an opaque string in JS.
use hegemony_server::{
    catalog,
    model::{Action, ChoiceResolution, Game},
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
        ChoiceResolution::Forecast { .. } => {
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
    let value = json!({"catalog":catalog::catalog(),"cases":[fixture("duel","18446744073709551615"),fixture("teams","9007199254740993")]});
    std::fs::write(output, serde_json::to_vec(&value).unwrap()).unwrap();
}
