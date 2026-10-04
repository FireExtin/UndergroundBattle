# Portable Linux loopback preview package — 2026-10-04

Prepared a small transferable preview of the independently reviewed `c4d20b599be77a3e09a4bd8dc3e5e0bd272e0620` production candidate, with built `web/dist`, the previously verified candidate WASM, a Linux x86_64 native server, minimum native rebuild inputs and the 3,816-byte `5bd2178` tests/docs-only patch. The package lives under `/workspace/.private-validation/yellow-search-portable-preview-20261004/`.

The repository's existing native `rust-game/src/main.rs` fixes its host to `0.0.0.0`; `PORT` only changes the port. The package copies this existing entrypoint and changes only `let address = format!("0.0.0.0:{port}");` to `let address = format!("127.0.0.1:{port}");`. No repository production source, game logic, API, database schema, or service framework was changed. The single-line package difference is included for review.

`./start-preview.sh 8090` starts the packaged native service on `http://127.0.0.1:8090/`, serves the existing build and creates a fresh private temporary data directory. The executable requires Linux x86_64 with glibc >= 2.34; SQLite is bundled. Binary SHA-256: `09b113f159493c22a7cefeccc578c2574ffe413d0e5a60f79c137d2335dbcf23`. WASM SHA-256 remains `36d28923bf077edb23a339ad1c906b4171e09710aa4389309baf75b1c08fd631`.

One bounded readiness check passed: process-owned listening socket only `127.0.0.1`, health and version correct, native catalog exactly equal to the instantiated WASM catalog, and served index/assets/scan manifest/MSJC01/WM003/LC01 images byte-equal to the packaged files. No room, seat, or game command was issued. All temporary database tables were empty; the check's service was stopped and its database removed. No default full regression was repeated.

The existing “开始新的独立玩家会话” entry supports the parent's two independent seat contexts. No browser, seat selection, or gameplay was performed here. Cloud-browser reachability of the parent's loopback origin remains for the parent to check; the package adds no forwarding/public-port path. No GitHub push, Sites save/deploy or additional network permission was attempted.
