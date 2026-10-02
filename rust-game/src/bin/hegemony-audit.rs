use hegemony_server::service::Store;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("Usage: hegemony-audit <sqlite-database> <room-id> [room-id ...]");
        std::process::exit(2);
    }
    let store = Store::open_read_only(&args[0])?;
    let mut all_match = true;
    for room in &args[1..] {
        let report = store.audit_replay(room)?;
        all_match &= report.matches;
        println!("{}", serde_json::to_string(&report)?);
    }
    if !all_match {
        std::process::exit(1);
    }
    Ok(())
}
