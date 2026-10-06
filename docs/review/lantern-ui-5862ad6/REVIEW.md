# Lantern Table 5862ad6 cloud self-test evidence

This directory is evidence only. The tested production source HEAD is **5862ad6740dc095194655201745173d48a5989f1**, whose sole parent is **baed2594400dc38834943ab70ee2c42afcda7b72**. The child commit adding this directory did not change the four UI files or production runtime. Parent independent review is pending; these results do not constitute publication approval.

The actual source uses engine `rust-v0.2.36-jz55-unique-destroy-candidate`, pool `limited-v2.33-jz55-unique-destroy-candidate`, 100 ordinary cards and eight societies. It does not include the separate engine42/WM059 candidates. The four UI changes preserve attachment owner/controller/host context and accessible action names; instance tags are display-only.

## Build and test results

| Scope | Result |
|---|---|
| Locked WASM, rustc 1.90.0 / wasm-bindgen 0.2.104 | passed; equals frozen36 bytes |
| Current WASM | 2,181,427 bytes; SHA256 `0f3a637d854074fe0689927c51e7969e8a32551ddcb48e1e1f3f1b6d022ea036` |
| Sites TypeScript/Vite/Worker dry-run | passed; no deployment |
| Native full suite | 509 passed, 0 failed/ignored |
| Sites full suite | 24 passed, 0 failed/skipped |
| Adjacent UI suites | 10 files, 116 passed, 0 failed |
| Candidate full Web | 61 files, 476 tests: 456 passed, 20 failed, 0 skipped |
| Original baed full Web, same current36 package | 60 files, 473 tests: 453 passed, 20 failed, 0 skipped |

All 20 failed test identities match. The 15 affected test files are byte-identical to baed. After normalizing only the candidate/base worktree paths, all 20 complete failure messages match. Sixteen failures occur at the same assertions; four are old-fixture version rejection exceptions before the later UI assertions run. `baseline-assertions.json` records names, locations, source hashes and error headlines. No assertion or historical golden was changed to obtain these results.

## Actual four-seat UI run

Run7 used seed **9**, four independent Chromium contexts, the normal built Worker/WASM, workerd and persistent D1 SQLite. Only `RoomService.create` seed entropy was fixed; IDs, seat tokens and nonces retained native entropy. The harness applied schema migrations and ordinary create/join APIs; it did not inject or rewrite game state.

Actual UI actions created/joined four seats, readied/started the room, kept opening hands, created assets, and deployed independently identified same-name pieces. Two targeting cancellations left room state, command receipts and journal unchanged. Seat2 sacrificed its JC042. Seat1 then deployed a card onto the stack; Seat3 clicked BeginResponse and submitted XQ49 targeting Seat2's graveyard card. The real stack contained two effects; the target returned to its owner's deck bottom, the actor paid one asset and drew one card.

Four committed-command acknowledgements were deliberately lost, including automatic retries. A page reload preserved the original pending identity, and the real retry button recovered the stored response without a second execution. Normal UI passes completed all five regions and the first round, reaching turn2/version194. Two explicitly separate API probes checked a duplicate old command (200, no writes) and a new command at expectedVersion0 (409, no writes). Disposal/recreation of the actual Miniflare instance preserved all persisted rows byte-for-byte. All four pages reloaded into their original seats; Seat1 returned to the lobby and restored its original seat through the UI.

There were 191 unique command receipts, 194 consecutive journal entries, 196 successful command HTTP responses including five duplicate responses, four seats and four entry receipts. Every successful command response equals its stored receipt. Native `Store::open_read_only` replay matched at version194; persisted/replayed SHA256:

`c2cd2a3a21164e8dfed76bb64827c570361b9ee2d7a562019603d0f031decab4`

The persistent SQLite and WAL bytes were unchanged by this read audit. SQLite SHM read-lock bytes are excluded. The 390x844 restoration screenshot has a crowded status area and is submitted for visual review; this was not a full mobile interaction acceptance run.

Runs1-6 are retained locally: lost-ACK simulation/reload races, incorrect harness planning of loyalty/region/stack, a mistaken draw-two assertion (XQ49 draws one), and a reversed CSS selector. These were harness mistakes, not production fixes. Run6 already completed the round/persistence checks and matched native replay; run7 also passed the complete UI restoration flow.

## Execution record and reproduction

Cloud source directory: `/workspace/lantern-ui-integration-game`. Cargo environment used `CARGO_HOME=/workspace/jz31-tools/cargo`, `RUSTUP_HOME=/workspace/jz31-tools/rustup`, cargo bin on PATH, `CARGO_INCREMENTAL=0`. Native builds used `CARGO_PROFILE_DEV_DEBUG=0`.

```bash
# Source root; clean release build was followed by this locked/offline check.
CARGO_TARGET_DIR=/tmp/lantern-ui-wasm-target cargo build --locked --offline -p hegemony-wasm --release --target wasm32-unknown-unknown
/workspace/jz31-tools/bindgen/bin/wasm-bindgen --target web --out-dir /tmp/lantern-ui-locked-bindgen-check --out-name hegemony_wasm /tmp/lantern-ui-wasm-target/wasm32-unknown-unknown/release/hegemony_wasm.wasm
CARGO_TARGET_DIR=/tmp/lantern-ui-native-target CARGO_PROFILE_DEV_DEBUG=0 cargo test --locked --offline -p hegemony-server
# sites working directory
npm run build
npm test
node /workspace/lantern-ui-e2e-evidence/four-seat-e2e.mjs four-seat-run-7
# web working directory; baed used the same command with base-web-results.json.
npm test -- --reporter=default --reporter=json --outputFile=/workspace/lantern-ui-e2e-evidence/web-results.json
python /workspace/lantern-ui-e2e-evidence/audit-final-evidence.py four-seat-run-7
```

The scripts here are the historical verification sources. The E2E copy changes only its relocation-sensitive import to the equivalent absolute cloud path; original script hashes are in `test-summary.json`. They contain synthetic seat labels and runtime token variables, no literal credentials or signed URLs. They create raw local test artifacts outside the repository; those artifacts are not committed here. For a rerun, restore the locked dependencies and current36 package, use a new run name, and preserve the original run7 evidence. To review source behavior, check out the fixed tested source HEAD; do not equate an evidence child HEAD with a newly tested runtime change.

`screenshots-local-manifest.json` lists all 45 local PNGs with sizes and SHA256. The seven run7 PNGs are the completed acceptance sequence; earlier files are debugging evidence. No PNG, database, browser storage, raw private session or complete HTTP log is included in this commit. Screenshots still require a supported transfer/review route.

No main push, Site publication, Library upload retry or alteration of engine42/WM059 occurred. Original local review ZIP remains separate and unchanged. This directory supports parent independent review; it does not replace that review.
