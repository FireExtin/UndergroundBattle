//! Finite, test-only Native replays of preserved engine57 prepared layouts.
//! Never linked into the production WASM/Worker; not a saved-room migration.
use hegemony_server::{
    catalog,
    deck::{self, DeckDraft},
    room::{QuoteRequest, RoomCommand, RoomEnvelope},
};
use serde_json::{json, Value};
use std::path::Path;

const FILES: &[&str] = &[
    "sites/test/fixtures/xq37-native-v057.json",
    "sites/test/fixtures/gray-lock-native-v057.json",
    "sites/test/fixtures/jc050-native-v057.json",
    "sites/test/fixtures/xq27-invalid-rootface-v057.json",
    "sites/test/fixtures/deck-seal-native-v057.json",
    "sites/test/fixtures/three-color-death-native-v057.json",
    "sites/test/fixtures/xq37-invalid-rootface-v057.json",
    "sites/test/fixtures/jz50-room-native-v057.json",
    "sites/test/fixtures/bounded-entry-native-v057.json",
    "sites/test/fixtures/xq27-native-v057.json",
    "sites/test/fixtures/pause-resume-native-v057.json",
    "sites/test/fixtures/jz22-native-v057.json",
    "sites/test/fixtures/sealing-room-native-v057.json",
    "sites/test/fixtures/sealed-restart-native-v057.json",
    "web/src/game/threeColorDeathNative57Test.fixture.json",
    "web/src/game/xq37Native57Test.fixture.json",
    "web/src/game/jz22Native57Test.fixture.json",
    "web/src/game/deckSealNative57Test.fixture.json",
    "web/src/game/xq27Native57Test.fixture.json",
    "web/src/game/boundedEntryNative57Test.fixture.json",
    "web/src/game/jc050Native57Test.fixture.json",
    "web/src/game/jc050Native57QuoteTest.fixture.json",
    "web/src/game/xq18Native60Test.fixture.json",
];

fn layout(raw: &str) -> String {
    // serde_json preserves the original u64 seed/random values. This is an
    // offline prepared layout, including current validated construction snapshots.
    // The original input and production strict loader are never changed.
    let mut value: Value = serde_json::from_str(raw).unwrap();
    let versions = json!({"rules":catalog::RULES_VERSION,"cardPool":catalog::POOL_VERSION,"engine":catalog::ENGINE_VERSION});
    value["versions"] = versions.clone();
    value["game"]["versions"] = versions;
    for player in value["game"]["players"].as_array_mut().unwrap() {
        if let Some(snapshot) = player.get_mut("deck_snapshot").filter(|v| !v.is_null()) {
            snapshot["rulesVersion"] = json!(catalog::RULES_VERSION);
            snapshot["cardPoolVersion"] = json!(catalog::POOL_VERSION);
            snapshot["engineVersion"] = json!(catalog::ENGINE_VERSION);
            let draft: DeckDraft = serde_json::from_value(snapshot.clone()).unwrap();
            *snapshot = serde_json::to_value(
                deck::validate(draft).expect("prepared deck must pass current construction rules"),
            )
            .unwrap();
        }
    }
    serde_json::to_string(&value).unwrap()
}
fn room(raw: &str) -> RoomEnvelope {
    RoomEnvelope::from_persisted(&layout(raw)).unwrap()
}
fn views(room: &RoomEnvelope) -> Value {
    json!((0..room.players.len())
        .map(|seat| room.view(seat, room.pacing.last_server_now_ms))
        .collect::<Vec<_>>())
}
fn normalized(value: &Value) -> Value {
    match value {
        Value::String(s) if s.starts_with('{') && s.contains("\"versions\"") => {
            normalized(&serde_json::from_str::<Value>(s).unwrap())
        }
        Value::String(s)
            if s == "rust-v0.2.57-black-entry-influence-candidate"
                || s == "rust-v0.2.60-xq18-carrier-candidate"
                || s == "rust-v0.2.62-fixed-empty-slots-candidate"
                || s == catalog::ENGINE_VERSION =>
        {
            json!("CURRENT_ENGINE")
        }
        Value::String(s)
            if s == "limited-v2.52-black-entry-influence-candidate"
                || s == "limited-v2.55-xq18-carrier-candidate"
                || s == catalog::POOL_VERSION =>
        {
            json!("CURRENT_POOL")
        }
        Value::Array(a) => Value::Array(a.iter().map(normalized).collect()),
        Value::Object(o) => Value::Object(
            o.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        if k == "catalog" {
                            json!("CURRENT_CATALOG_CHECKED_SEPARATELY")
                        } else {
                            normalized(v)
                        },
                    )
                })
                .collect(),
        ),
        _ => value.clone(),
    }
}
fn same_semantics(old: &Value, new: &Value, label: &str) {
    if normalized(old) != normalized(new) {
        eprintln!(
            "{label}: old outcome={:?}, new outcome={:?}, new error={:?}",
            old.get("outcome"),
            new.get("outcome"),
            new.get("errorMessage")
        );
    }
    assert!(
        normalized(old) == normalized(new),
        "semantic conflict in {label}; do not silently replace expectations"
    );
}
fn regenerate(value: &mut Value) {
    if let Some(items) = value.as_array_mut() {
        for item in items {
            regenerate(item);
        }
        return;
    }
    if !value.is_object() {
        return;
    }
    if value.get("expectedErrorContains").is_some() {
        let state = layout(value["state"].as_str().unwrap());
        let error = RoomEnvelope::from_persisted(&state)
            .err()
            .expect("corrupt layout must reject");
        assert!(error.contains(value["expectedErrorContains"].as_str().unwrap()));
        value["state"] = json!(state);
    } else if value.get("initialState").is_some() {
        let mut current = room(value["initialState"].as_str().unwrap());
        value["initialState"] = json!(serde_json::to_string(&current).unwrap());
        for step in value["steps"].as_array_mut().unwrap() {
            let command: Option<RoomCommand> =
                serde_json::from_value(step["command"].clone()).unwrap();
            let now = step["serverNowMs"].as_str().unwrap().parse().unwrap();
            let transition = current
                .transition(step["seat"].as_u64().unwrap() as usize, command, now)
                .unwrap();
            let expected = serde_json::to_value(&transition).unwrap();
            same_semantics(&step["transition"], &expected, "trace transition");
            step["transition"] = expected;
            current = RoomEnvelope::from_persisted(&transition.state).unwrap();
            let next_views = views(&current);
            same_semantics(&step["views"], &next_views, "trace views");
            step["views"] = next_views;
        }
        if value.get("terminalState").is_some() {
            value["terminalState"] = json!(serde_json::to_string(&current).unwrap());
        }
        if let Some(receipts) = value.get("receipts").and_then(Value::as_array).cloned() {
            value["receipts"] = json!(receipts
                .iter()
                .map(|receipt| {
                    let step = value["steps"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|s| s["command"]["commandId"] == receipt["commandId"])
                        .unwrap();
                    json!({"commandId":receipt["commandId"],"response":step["transition"]["view"]})
                })
                .collect::<Vec<_>>());
        }
    } else if let Some(raw) = value
        .get("state")
        .and_then(Value::as_str)
        .map(str::to_owned)
    {
        let current = room(&raw);
        value["state"] = json!(serde_json::to_string(&current).unwrap());
        if value.get("beforeViews").is_some() {
            let current_views = views(&current);
            same_semantics(&value["beforeViews"], &current_views, "prepared before views");
            value["beforeViews"] = current_views;
        }
        if value.get("command").is_some() {
            let command = serde_json::from_value(value["command"].clone()).unwrap();
            let now = value["expected"]["view"]["serverNowMs"].as_u64().unwrap();
            let transition = current
                .transition(value["seat"].as_u64().unwrap() as usize, Some(command), now)
                .unwrap();
            let expected = serde_json::to_value(&transition).unwrap();
            same_semantics(&value["expected"], &expected, "single transition");
            value["expected"] = expected;
            let next = RoomEnvelope::from_persisted(&transition.state).unwrap();
            let next_views = views(&next);
            same_semantics(&value["views"], &next_views, "single views");
            value["views"] = next_views;
        } else if value.get("request").is_some() {
            let request: QuoteRequest = serde_json::from_value(value["request"].clone()).unwrap();
            let expected = serde_json::to_value(
                current
                    .quote(value["seat"].as_u64().unwrap() as usize, request)
                    .unwrap(),
            )
            .unwrap();
            same_semantics(&value["expected"], &expected, "quote");
            value["expected"] = expected;
        } else if value.get("views").is_some() {
            let current_views = views(&current);
            same_semantics(&value["views"], &current_views, "prepared views");
            value["views"] = current_views;
        }
        if value.get("catalog").is_some() {
            value["catalog"] = serde_json::to_value(catalog::catalog()).unwrap();
        }
    } else if let Some(states) = value.get_mut("states").and_then(Value::as_object_mut) {
        for state in states.values_mut() {
            if let Some(raw) = state.as_str() {
                *state = json!(serde_json::to_string(&room(raw)).unwrap());
            } else {
                regenerate(state);
            }
        }
    } else {
        for item in value.as_object_mut().unwrap().values_mut() {
            regenerate(item);
        }
    }
}
fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".".into());
    let fresh_death = std::env::args().nth(2);
    for source in FILES {
        let input = Path::new(&root).join(source);
        let mut value: Value = serde_json::from_slice(&std::fs::read(&input).unwrap()).unwrap();
        if *source == "web/src/game/threeColorDeathNative57Test.fixture.json" {
            // The old Win layout has no recoverable interrupted step. Re-run
            // death_observer_tests with DEATH_OBSERVER_FRONTEND_DIR and use
            // those actual current Native outputs; never invent a save context.
            let dir = fresh_death.as_ref().expect("provide fresh DEATH_OBSERVER_FRONTEND_DIR as the second argument");
            for (name, old) in value.as_object_mut().unwrap() {
                let fresh: Value = serde_json::from_slice(
                    &std::fs::read(Path::new(dir).join(format!("{name}.json"))).unwrap(),
                ).unwrap();
                RoomEnvelope::from_persisted(fresh["state"].as_str().unwrap()).unwrap();
                let mut compared = normalized(&fresh);
                if name == "contractWinWindow" {
                    assert_eq!(compared["state"]["game"]["win_contexts"].as_array().unwrap().len(), 1);
                    compared["state"]["game"].as_object_mut().unwrap().remove("win_contexts");
                }
                assert_eq!(normalized(old), compared, "fresh Native death layout changed beyond engine identity and the new Win context: {name}");
                *old = fresh;
            }
        }
        regenerate(&mut value);
        let destination = source
            .replace("v057", "v063")
            .replace("Native57", "Native63")
            .replace("Native60", "Native63");
        std::fs::write(
            Path::new(&root).join(&destination),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
        println!("Native current fixture: {source} -> {destination}");
    }
}
