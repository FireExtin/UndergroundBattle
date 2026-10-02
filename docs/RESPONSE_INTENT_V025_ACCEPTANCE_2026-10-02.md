# v0.2.5 response-intent session acceptance

This document covers the server-owned response-intent policy in `rust-game/src/room.rs`, its native SQLite host, and the server-only WASM ABI. Card-pool changes in the same release have a separate source review and acceptance scope. The deterministic card interpreter remains in Rust.

## State and timing contract

`RoomEnvelope` uses outer `state_schema: 3`, `revision`, fixed `versions`, a schema-2 `game`, and finite `pacing`. The browser sees only the seat-filtered view. Persisted state remains an opaque JSON string across the WASM/JavaScript boundary. A strict header check rejects schema-2 rooms and old engine versions before typed decoding; old rooms require their pinned kernel bundle. No migration or reinterpretation occurs.

The public `version` is the room revision. `Game.version` is a derived mirror of it, including log entries created during that room transition. Several timeout passes in one transition therefore produce one room CAS, one journal row, and one revision. Native joins advance the same revision with their existing `Join` or `JoinWithDeck` journal event.

A window exists only while the game is playing, the stack is nonempty, and no game choice is pending. Its identity contains a monotonically increasing sequence, the stack-top instance, and the holder team. Each living member of that team has an independent `undecided`, `composing`, or `passed` decision. An undecided member has a deadline of trusted server time plus 5,000 ms. The boundary is inclusive: at elapsed 4,999 ms Begin may succeed; at 5,000 ms expiry acts first.

Begin does not select a card, pay costs, allocate game identities, or consume randomness. Composing has no selection deadline. A player can explicitly cancel the unsubmitted intent and pass. Formal submission atomically declares and pays a complete ordinary rule Action; it cannot be cancelled afterward. Any formal action creates a new response epoch and invalidates other old drafts. Passing one teammate does not pass the other. Repeated Begin cannot reset a deadline or revive a passed member.

Empty-stack priority, trigger-declaration choices, and already-paid resolution-frame choices remain untimed. A rule action that does not permit responses, such as the audited cost-reduction ability, still follows its existing Rust response policy; the session does not introduce a window for that ability itself.

The host supplies trusted decimal `serverNow`. The reducer clamps backward observations to the last committed time. Native HTTP uses the host clock; it does not accept browser-supplied timing. Polling or a new command processes only the currently due window. A newly opened window gets a fresh five seconds, and an empty stack never causes automatic phase progression. Native SSE projects committed state; state polling and commands perform authoritative expiry. There is no alarm, client timer, or offline pass cascade.

## Commands and view

Every command is `{commandId, expectedVersion, action}`. Session Actions are:

- `{kind:"game", action}` for lobby operations, empty-stack rule actions, and game choices.
- `{kind:"beginResponse", windowId, intentId}`.
- `{kind:"passResponse", windowId}` for an undecided member.
- `{kind:"cancelAndPass", windowId, intentId}` for a composing member.
- `{kind:"submitResponse", windowId, intentId, action}` with the complete ordinary rule Action.

Formal `game` and `submitResponse` actions require the exact current version. Decision-only commands may carry an older version only while the named window and this seat's required decision still match. Native retries these decision-only CAS failures at most three times with the original command; it does not rebase a paid action. An unwrapped paid response through `game` is rejected while a window exists.

The view retains the existing flattened Game view and legal-action shape. It adds `serverNowMs` and nullable `responseWindow`: `id`, `stackTopId`, `holderTeam`, public member `playerId/status/deadlineMs`, viewer-only `canBegin`, and viewer-only `myIntentId` when composing. Only a composing member receives their ordinary response candidates. Other seats receive no candidate count or intent ID. Existing hidden-card, hand, deck snapshot, and pending-choice filtering remains in the Game view.

`quoteRoom` accepts `{windowId,intentId,draft?}`. It checks only the authenticated composing member's candidates and validates a complete draft on a clone. It returns `version/windowId/intentId/legalActions/ready/normalizedAction?/error?`. The original state, randomness, revision, costs, and triggers remain unchanged.

## ABI and persistence

The schema-3 WASM exports are `newGame`, `newGameWithDeck`, `joinGame`, `joinGameWithDeck`, `view`, `roomCatalog`, `stateIdentity`, `applyRoom`, `pollRoom`, and `quoteRoom`. The legacy bare `apply` export rejects schema-3 use so it cannot bypass pacing. Creation seeds remain decimal u64 strings, including the maximum u64 and values above JavaScript's exact-integer range.

`applyRoom(state, seat, commandJSON, serverNowDecimal)` and `pollRoom(state, seat, serverNowDecimal)` return:

```json
{"state":"opaque private string","view":{},"version":1,"seat":0,"changed":true,"journal":[],"outcome":"accepted"}
```

A rejection adds `errorCode` and `errorMessage`; codes include `version_conflict`, `window_expired`, and `invalid_action`. Receipt conflicts are detected by the host as `command_id_conflict`. A no-expiry poll is accepted with `changed:false`, no journal, and the same state/revision; its returned view may have a later `serverNowMs`.

Native and Sites journal wrappers use the same externally tagged shape:

```json
{"SessionEvents":{"events":[{"Tick":{"server_now_ms":6000}},{"Command":{"seat":0,"command":{"commandId":"example","expectedVersion":8,"action":{"kind":"beginResponse","windowId":"response:2","intentId":"draft"}},"server_now_ms":6000}}]}}
```

One committed transition contains either a Tick, a Command, or a Tick followed by a Command. Empty or reordered event batches are rejected by replay. The observed trusted time and original command are retained; native audit calls the same room reducer.

Receipts are looked up before expiry. The complete original seat, expected version, and typed Session Action must match; the same accepted intent returns its original ACK after restart or a later clock observation. A changed window, intent, action, seat, or expected version with the same command ID conflicts without changing state or journal. If expiry occurs before a rejected command, the host first commits the system Tick using CAS and journal, then returns the rejection with the new view. It does not create a successful user receipt.

Native SQLite commits state/revision, journal, and any successful receipt in one transaction before ACK. A failed transaction changes none of them. Concurrent joins use CAS before adding credentials and journal. `HEGEMONY_DB` defaults to the isolated `rust-game-v2.5.sqlite3`; old running services and databases are left intact.

## Tests and retained v0.2.4 cases

The six `room_pacing` tests cover the boundary, repeat Begin, independent teammates, re-quote after formal action, mixed timeout/composing decisions, private projection, naked-response rejection, invalid atomic submission, paid choices without clocks, and no empty-stack cascade.

The six `session_service` tests cover paid JC042 response and missing Murder target; reopen/quote/original ACK; window/intent receipt conflicts; expiry-only journal on rejection; Begin versus timeout CAS; two parallel teammate Begins with strict stale payment rejection; concurrent joins; and an injected SQLite failure without ACK or timer mutation. Fixtures explicitly declare an initial layout before any recorded command. All later transitions are authenticated Room commands; these fixtures do not claim natural dealing.

The existing ten native service tests retain their original purpose under the new protocol:

| Existing test | Schema-3 correspondence |
| --- | --- |
| `command_id_reuse_requires_same_complete_typed_intent_before_and_after_reopen` | Typed Session Action and original expected version are checked before expiry. |
| `paid_sacrifice_death_trigger_and_accepted_frame_restore_without_repayment` | Initial declaration uses `game`; stack passes use response decisions; pending trigger/frame choices use untimed `game/choose`. |
| `detective_reveal_choice_and_bound_frame_restore_without_repayment_or_private_leaks` | Reveal, private trigger choice, explicit response passes, bound-frame restore, and fee assertions remain. |
| `legacy_state_is_rejected_explicitly_without_silent_migration` | Schema-2 and old fixed versions are rejected before typed room loading. |
| `dedupe_conflicts_rejections_auth_and_replay_are_strict` | Original receipt, authorization, version conflict, rejected action, and replay assertions remain. |
| `sqlite_reopen_restores_identity_pending_choice_and_dedupe` | Seat credentials, untimed pending choice, and original receipt survive reopen. |
| `failed_transaction_does_not_ack_or_mutate_state` | A real SQLite transaction failure still rolls back state, journal, and receipt. |
| `room_catalog_requires_a_token_for_that_room_and_returns_current_pool` | Room-scoped catalog authentication is unchanged. |
| `authenticated_sse_projects_actor_view_and_updates_after_commit` | SSE emits the seat-filtered Room view only after a committed transition. |
| `read_only_audit_cli_matches_seed_journal_and_detects_corruption` | Audit replays schema-3 SessionEvents and compares complete canonical room bytes. |

The new native WASM oracle retains the seven prior rule/deck/privacy scenarios and adds two clock scenarios. Each changed transition is also replayed from its journal and compared byte-for-byte. The JavaScript comparator must parse only outer ABI results and seat views, never the private `.state` string.

## Verification record

Before world-card integration, the native oracle ran successfully with 9 scenarios, 453 steps, 1,337 seat views, and 1 pure quote. This is a native oracle result, not a WASM-equivalence or production acceptance claim. The isolated target is `/workspace/.private-validation/hegemony-v025-target`; private logs and fixtures are under `/workspace/.private-validation/hegemony-v025-evidence`. The final combined native build, WASM comparison, and production acceptance are recorded after the shared world-card source is frozen.

The combined `limited-v2.3` / `rust-v0.2.5` native verification is complete:

- 81 distinct native tests are green: 52 library tests (including 12 world tests) and 29 integration tests. The first library run had 51 passes and one old registry-size fixture assertion (`30`, now `36`); after root corrected that assertion, its targeted rerun passed. The 51 successful library cases were not mechanically repeated.
- The five integration suites passed together: deck construction 5, friendly defense/public totals 2, room pacing 6, retained native service 10, session SQLite/concurrency 6.
- No-default-features library check, both native binaries, WASM crate native check, and the repository Rust formatting check passed.
- The final native oracle has 16 cases, 537 steps, 1,673 seat views, and 1 pure quote. Six added released-world scenarios use an explicitly marked initial `Window::Win` layout, preserve ten distinct physical world cards, and then use ordinary passes, the real `RegionWon` trigger, acceptance, response passes, and private choices. They cover free reveal without loyalty, nonhuman hiding, per-controller London mixing/PRNG/instance replacement, current-defense sacrifice draw, independently chosen graveyard entries in a retained region, and the exact empty Hong Kong attachment search. Every persisted step is decoded before continuing; every changed event batch replays to identical room bytes.
- The seventh added world-related scenario deliberately changes only a synthetic initial declaration to the shared `SimultaneousSearch(Kind("character"))` operation. It checks four independent private commitments, a decline, unchanged hands/decks/PRNG/log until all commit, and then simultaneous reveal/hand transfer before shuffle. It is a generic-operation test, **not evidence of a nonempty actual Hong Kong attachment search**. The released pool currently has no attachments.

The synthetic clock and world fixture room IDs use fixed 24-character hexadecimal strings for the host's route validation. They contain no real room credentials or injected real-game state. World cases take 10–15 steps each; the generic search case takes 11. These are deterministic mechanism proofs, not naturally dealt browser-game evidence.

Logs are `final-native-tests.log`, `final-registration-regression.log`, `final-integration-tests.log`, `final-pure-lib.log`, `final-native-build.log`, `final-wasm-native-check.log`, `final-world-oracle-native.log`, and `final-fmt.log` under the evidence directory above. The first native library failure and its targeted green correction are retained.

Final oracle file: `/workspace/.private-validation/hegemony-v025-evidence/final-native-fixtures.json`, 50,719,015 bytes, SHA-256 `147fed359e0c5607bb222f39c154963cc320cb17ca20f7e7701b97d778a50057`.

Final native artifact paths and SHA-256:

- `/workspace/.private-validation/hegemony-v025-target/debug/hegemony-server`: `7231d759a9620634b1601a2f695764a2d2ed05d8552364cb7ae398877bb9fa54`.
- `/workspace/.private-validation/hegemony-v025-target/debug/hegemony-audit`: `52e549cd0494941b17d0567407c1758c3db07090540114c2c995a1edc2b7a93e`.

Actual wasm32 compilation/Node instantiation, Sites CAS/adapter acceptance, browser acceptance, commit and publishing are root-owned follow-up gates. This record does not substitute the native oracle for those checks. No running server, legacy bundle, existing QA room, or production database was changed by this clock implementation.
