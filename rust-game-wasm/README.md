# Shared Rust kernel WASM proof

This crate exposes the existing deterministic `hegemony-server` kernel to a server-side JavaScript host. It contains no separate rules implementation. The native service remains the default build; its HTTP, SQLite, authentication and SSE code is behind the `native` feature and is excluded from this WASM build.

This is a local compilation and compatibility proof. No Workers room service, hosting configuration or Internet deployment is included or validated.

## Build and verify

Use Rust 1.90.0, Node.js and `wasm-bindgen-cli` **0.2.104** (matching the exact crate version):

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.104 --locked
bash rust-game-wasm/verify.sh
```

`WASM_BINDGEN=/path/to/wasm-bindgen` selects an alternate CLI location. `build.sh` creates the ignored `rust-game-wasm/pkg/hegemony_wasm.js` ESM binding and `hegemony_wasm_bg.wasm`. `verify.sh` compiles the module, generates native fixture results, actually instantiates the WASM in Node, then compares every transition and each player's view.

The completed local run compared **374 transitions and 1,119 player projections** across duel and four-player teams. Every opaque state string matched the native output byte for byte; every projected view matched. The run also exercised private choices and exact decimal seeds `18446744073709551615` and `9007199254740993`.

The generated WASM artifact was **803,612 bytes**, SHA-256:

```text
fb75e4c68f604c46febba70211d673d4d492c5994bc17dead7c5de26d8c330be
```

## Server-only interface

| Export | Arguments | JSON string result |
| --- | --- | --- |
| `catalog` | none | Public card/deck catalog |
| `newGame` | room ID, invite code, mode, player name, deck ID, decimal seed string | `{ state, view, version, seat }` |
| `joinGame` | opaque state string, player name, deck ID | `{ state, view, version, seat }` |
| `apply` | opaque state string, authenticated seat, action JSON string | `{ state, view, version, seat }` |
| `view` | opaque state string, authenticated seat | Only that player's projected view |

Parse the outer result to access its fields, but preserve `state` as an **opaque string** in server storage. Never parse and reserialize that string through JavaScript: native `u64` values exceed JavaScript's exact integer range. Pass the seed as decimal text for the same reason. Full `state` contains private information and must never be returned to a browser. The HTTP response may contain only the authenticated player's `view` and relevant public metadata.

The caller must authenticate and map the seat, enforce expected versions and command deduplication, serialize room mutations, and persist before acknowledging. This crate supplies none of those room-service responsibilities. Imported states are checked against the pinned rules, card-pool and engine versions.

Node verification initializes the `--target web` output using:

```js
import { readFileSync } from 'node:fs';
import { initSync, newGame } from './pkg/hegemony_wasm.js';
initSync({ module: readFileSync('./pkg/hegemony_wasm_bg.wasm') });
const result = JSON.parse(newGame('room', 'invite', 'duel', '玩家', 'watchers', '9007199254740993'));
// Save result.state unchanged on the server; send only result.view to this seat.
```

A runtime that imports a compiled `WebAssembly.Module` can pass that module to `initSync`; provider-specific import, limits and execution behavior still require validation before deployment.

## Compatibility boundary

The only shared-core behavior change made for this proof is the damage choice's UI metadata: `max` now reports the number of available targets instead of `usize::MAX`. Damage allocation validation is unchanged. This removes the architecture-dependent 64-bit/32-bit sentinel and makes fresh native and WASM states identical.

An old native room paused at a damage choice may retain `18446744073709551615` in its saved `max`. That historical state cannot be deserialized into 32-bit WASM without an explicit migration. No migration is included, and running native services or their databases were not changed by this proof.
