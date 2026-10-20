use hegemony_server::{model::Game, room::RoomEnvelope};
use std::{env, fs};

fn inspect(label: &str, game_path: &str, room_path: &str) {
    let game_raw = fs::read_to_string(game_path).unwrap();
    let room_raw = fs::read_to_string(room_path).unwrap();
    let game = Game::from_persisted(&game_raw);
    let room = RoomEnvelope::from_persisted(&room_raw);
    println!("CASE {label} GAME_ACCEPTED={} ROOM_ACCEPTED={}", game.is_ok(), room.is_ok());
    match game {
        Ok(game) => println!("GAME {label} pending={} stack={} effects={} window={:?} undecided={}",
            game.pending.is_some(), game.stack.len(), game.effects.len(), game.window,
            game.jc035_preparation.as_ref().map_or(0, |o| o.entries.iter().filter(|e| e.decision.is_none()).count())),
        Err(error) => println!("GAME_ERROR {label} {error}"),
    }
    match room {
        Ok(room) => println!("ROOM {label} revision={} pending={} stack={} effects={}",
            room.revision, room.game.pending.is_some(), room.game.stack.len(), room.game.effects.len()),
        Err(error) => println!("ROOM_ERROR {label} {error}"),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() > 1 && (args.len() - 1) % 3 == 0, "label, Game path, Room path triples");
    for case in args[1..].chunks_exact(3) {
        inspect(&case[0], &case[1], &case[2]);
    }
}
