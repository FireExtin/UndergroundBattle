//! Observation probe for the reviewed baseline, not the desired future contract.
//! Run only in a temporary copy; see run_checks.py. No production DB is opened.
use hegemony_server::{
    model::Action,
    service::{Command, CreateRoom, JoinRoom, Store},
};

#[tokio::main]
async fn main() {
    let store = Store::open(":memory:").unwrap();
    let a = store.create(CreateRoom {
        name: "probe-a".into(), mode: "duel".into(), deck_id: "watchers".into(),
    }).await.unwrap();
    store.join(JoinRoom {
        invite_code: a.invite_code.clone(), name: "probe-b".into(), deck_id: "hunters".into(),
    }).await.unwrap();
    let original = Command {
        command_id: "receipt-probe".into(), expected_version: 1, action: Action::new("ready"),
    };
    let first = store.command(&a.room_id, &a.token, original).await.unwrap();
    let changed = Command {
        command_id: "receipt-probe".into(), expected_version: 2, action: Action::new("start"),
    };
    let response = store.command(&a.room_id, &a.token, changed).await.unwrap();
    assert_eq!(serde_json::to_value(&response).unwrap(), serde_json::to_value(&first).unwrap());
    assert_eq!(store.state(&a.room_id, &a.token).await.unwrap().version, 2);
    println!("OBSERVED: same seat + commandId with changed action and expectedVersion returns original receipt; state remains version 2");
    println!("DESIRED: reject changed intent; preserve exact original-intent retries");
}
