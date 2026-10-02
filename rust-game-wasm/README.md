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

The v2.1 fixtures cover duel and four-player teams plus an explicitly marked initial-layout scenario whose subsequent transitions use real JC042/JC091 response Actions, JZ54 sacrifice choices, LC24 frame forecast continuations and the LC19 heal exhaustion cost. The fixtures also require both native and WASM to reject previous-patch opaque states. Every opaque state string and every player view is compared against native output. Decimal seeds `18446744073709551615` and `9007199254740993` remain exact.

The completed v2.1 run on fixed commit `d94baaf3e7caabfdb81e1b3cc2a537fefe355fd0`, built in the fresh `/tmp/hegemony-d94baaf-final-target`, compared **397 transitions / 1,165 player projections**, all identical. Node actually instantiated the module and verified opaque-state equality, private projections, exact maximum-u64 seeds, rejected repeat actions and previous-patch state rejection. Its generated module is **1,040,341 bytes**, SHA-256:

```text
a6366ef6bc7df03ba666b33286d3a01c412ac6bf779d6396fcbf6affcdc43966
```

The historical `8a16cf3` v2.0 run compared 394 transitions / 1,159 player projections; its 1,039,765-byte module SHA-256 was `b00480fe9490117d327ead03acc94774ccfc0377f9ff1993a330e7be0c2522cb`.

The historical `84a2c59` v1 proof compared 374 transitions / 1,119 player projections; its 803,612-byte module SHA-256 was `fb75e4c68f604c46febba70211d673d4d492c5994bc17dead7c5de26d8c330be`. These are historical values, not the current v2.1 artifact.

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

V2.1 uses state schema 2, rules `hegemony-pdf-v1`, card pool `limited-v2.1`, engine `rust-v0.2.1`. The shared core now uses typed abilities, paid costs, bound target predicates and public snapshots, one whole-object guard, persisted effect cursors, source last-known information, and finite queries/modifiers. All 25 released player cards and four active world types use that kernel. The curated ten-card world contains three DQJC107, three DQJC112, two DQJC113 and two DQJC114. The original four decks remain intact; a fifth 50-card curated deck exposes the four newly verified cards. V2.1 adds registration guards, distinguishes entering play from entering a region, and corrects LC19’s heal cost to two assets plus exhausting its source. Existing API fields are unchanged.

V1 and prior `rust-v0.2.0` / `limited-v2` states are explicitly rejected before typed deserialization. Keep original binaries/databases for old rooms and replay; no silent migration is supplied. Historical native damage choices could also contain a 64-bit `usize::MAX` value that cannot be read by 32-bit WASM. Current choice metadata uses actual target counts. Existing native processes and their old databases remain untouched.
