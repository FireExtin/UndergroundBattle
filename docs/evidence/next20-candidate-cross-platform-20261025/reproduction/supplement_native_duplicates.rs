//! Supplemental probe only: call the already compiled production Room reducer.
//! Original opaque states/commands/views remain unchanged in a new output file.
use hegemony_server::room::{RoomCommand,RoomEnvelope};
use serde_json::Value;
use std::{env,fs,io::Write};
fn main(){
    let args=env::args().collect::<Vec<_>>();assert_eq!(args.len(),3);
    let mut scenarios:Value=serde_json::from_slice(&fs::read(&args[1]).unwrap()).unwrap();
    let mut count=0;
    for scenario in scenarios.as_array_mut().unwrap(){
        for row in scenario["steps"].as_array_mut().unwrap(){
            assert!(row.get("duplicate").is_none());
            let room=RoomEnvelope::from_persisted(row["expected"]["state"].as_str().unwrap()).unwrap();
            let cmd:RoomCommand=serde_json::from_value(row["command"].clone()).unwrap();
            let now=row["now"].as_str().unwrap().parse::<u64>().unwrap();
            let probe=room.transition(row["seat"].as_u64().unwrap() as usize,Some(cmd),now).unwrap();
            row["duplicate"]=serde_json::to_value(probe).unwrap();count+=1;
        }
    }
    fs::OpenOptions::new().write(true).create_new(true).open(&args[2]).unwrap()
        .write_all(&serde_json::to_vec(&scenarios).unwrap()).unwrap();
    println!("actual Native duplicate probes: {count}");
}
