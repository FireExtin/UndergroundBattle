//! Server-only ABI over the same deterministic native kernel.
//! Entire game state crosses JavaScript only as an opaque JSON string.
use hegemony_server::{
    catalog as definitions,
    model::{Action, Game, Versions, View},
};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct Transition {
    /// Private server storage. NEVER return this string to a browser.
    state: String,
    view: View,
    version: u64,
    seat: usize,
}
fn error(message: impl AsRef<str>) -> JsValue {
    JsValue::from_str(message.as_ref())
}
fn decode(state: &str) -> Result<Game, JsValue> {
    let game = Game::from_persisted(state).map_err(error)?;
    if game.versions.rules != definitions::RULES_VERSION
        || game.versions.card_pool != definitions::POOL_VERSION
        || game.versions.engine != definitions::ENGINE_VERSION
    {
        return Err(error(
            "Persisted rules/card-pool/engine versions do not match this kernel",
        ));
    }
    if (game.mode != "duel" && game.mode != "teams")
        || game.players.is_empty()
        || game.players.len() > game.capacity()
        || game.players.iter().enumerate().any(|(i, p)| p.seat != i)
    {
        return Err(error("Invalid persisted room seats or mode"));
    }
    Ok(game)
}
fn valid_seat(game: &Game, seat: usize) -> Result<(), JsValue> {
    if seat >= game.players.len() {
        Err(error("Invalid authenticated seat"))
    } else {
        Ok(())
    }
}

/// Only version strings leave this reader; private state and u64 values stay in Rust.
#[derive(Deserialize, Serialize)]
struct StateIdentity {
    state_schema: u32,
    versions: Versions,
}
#[wasm_bindgen(js_name = stateIdentity)]
pub fn state_identity(state: &str) -> Result<String, JsValue> {
    let identity: StateIdentity =
        serde_json::from_str(state).map_err(|_| error("Invalid persisted state identity"))?;
    if identity.state_schema != 2 {
        return Err(error("Unsupported persisted state schema"));
    }
    serde_json::to_string(&identity).map_err(|_| error("Identity serialization failed"))
}
fn encode(game: Game, seat: usize) -> Result<String, JsValue> {
    valid_seat(&game, seat)?;
    let state = serde_json::to_string(&game).map_err(|_| error("State serialization failed"))?;
    let view = game.view(seat);
    serde_json::to_string(&Transition {
        state,
        view,
        version: game.version,
        seat,
    })
    .map_err(|_| error("Response serialization failed"))
}

#[wasm_bindgen]
pub fn catalog() -> Result<String, JsValue> {
    serde_json::to_string(definitions::catalog()).map_err(|_| error("Catalog serialization failed"))
}

#[wasm_bindgen(js_name = newGame)]
pub fn new_game(
    room_id: &str,
    invite_code: &str,
    mode: &str,
    name: &str,
    deck_id: &str,
    seed_decimal: &str,
) -> Result<String, JsValue> {
    let seed = seed_decimal
        .parse::<u64>()
        .map_err(|_| error("Seed must be a decimal u64 string"))?;
    let game = Game::new(
        room_id.into(),
        invite_code.into(),
        mode.into(),
        name.into(),
        deck_id.into(),
        seed,
    )
    .map_err(error)?;
    encode(game, 0)
}

#[wasm_bindgen(js_name = joinGame)]
pub fn join_game(state: &str, name: &str, deck_id: &str) -> Result<String, JsValue> {
    let mut game = decode(state)?;
    let seat = game.join(name.into(), deck_id.into()).map_err(error)?;
    encode(game, seat)
}

#[wasm_bindgen]
pub fn apply(state: &str, seat: usize, action_json: &str) -> Result<String, JsValue> {
    let mut game = decode(state)?;
    valid_seat(&game, seat)?;
    let action: Action =
        serde_json::from_str(action_json).map_err(|_| error("Invalid action JSON"))?;
    game.apply(seat, action).map_err(error)?;
    encode(game, seat)
}

#[wasm_bindgen]
pub fn view(state: &str, seat: usize) -> Result<String, JsValue> {
    let game = decode(state)?;
    valid_seat(&game, seat)?;
    serde_json::to_string(&game.view(seat)).map_err(|_| error("View serialization failed"))
}
