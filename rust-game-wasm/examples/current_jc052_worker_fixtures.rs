//! Fresh Native HTTP-compatible prepared layouts; no save migration or live data.
use hegemony_server::room::{RoomCommand, RoomEnvelope};
use serde_json::{json, Value};
use std::{fs, io::Write, path::Path};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 3, "current_jc052_worker_fixtures input output");
    let output = Path::new(&args[2]);
    assert!(!output.exists(), "Preserve prior inputs; select a new output path");
    let input: Value = serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let mut cases = vec![];
    for (index, row) in input["cases"].as_array().unwrap().iter().enumerate() {
        // Both the seed and prepared output use the actual current identity.
        // Change only the layout's room identifier before executing Native.
        let mut room = RoomEnvelope::from_persisted(row["before"].as_str().unwrap()).unwrap();
        room.game.room_id = format!("5200000000000000000000{:02}", index + 1);
        let seat = row["seat"].as_u64().unwrap() as usize;
        let before = serde_json::to_string(&room).unwrap();
        let before_views = (0..room.game.players.len()).map(|s| room.view(s, 0)).collect::<Vec<_>>();
        let command: RoomCommand = serde_json::from_value(row["command"].clone()).unwrap();
        let transition = room.transition(seat, Some(command), 0).unwrap();
        assert_eq!(transition.outcome, "accepted");
        let next = RoomEnvelope::from_persisted(&transition.state).unwrap();
        let after_views = (0..next.game.players.len()).map(|s| next.view(s, 0)).collect::<Vec<_>>();
        cases.push(json!({"name":row["name"],"seat":seat,"before":before,
            "beforeViews":before_views,"command":row["command"],"expected":transition,"afterViews":after_views}));
    }
    assert_eq!(cases.len(), 4);
    let result = json!({"scope":"Current Native prepared layouts with HTTP-compatible IDs and actual transitions; not browser play or migration.",
        "catalog":hegemony_server::catalog::catalog(),"cases":cases});
    fs::OpenOptions::new().write(true).create_new(true).open(output).unwrap()
        .write_all(&serde_json::to_vec(&result).unwrap()).unwrap();
    println!("4 fresh Native JC052 Worker inputs: {}", output.display());
}
