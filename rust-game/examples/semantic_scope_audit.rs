//! Read-only audit export of the actual compiled finite bindings, not a rule registry.
fn main() {
    println!(
        "{}",
        serde_json::json!({
            "catalog": hegemony_server::catalog::catalog(),
            "definitions": hegemony_server::rules::definitions(),
        })
    );
}
