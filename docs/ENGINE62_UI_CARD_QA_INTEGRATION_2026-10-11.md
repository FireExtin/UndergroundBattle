# Engine62, reviewed UI and card provenance integration

The candidate combines the reviewed UI at `c285239be866523044734f7b0bf5198664e6396b`,
Engine61 `fc4c61dc29f6b9816951ab5cf68cb1cc53aa797d`, Engine62
`57733aba7db44e246bf4690e3155b4a8d5b9ff5e`, and the admitted card-scan provenance
repair at `e067b3b167367fc98562095abbd02db337fd9c5a`. Their common base is
`14b7b5e06a0b1a3649f2a3f947ad515236ebdfd9`. The source patches apply without
conflicts. BQ030 is outside this candidate.

The Worker now declares `rust-v0.2.62-fixed-empty-slots-candidate`. Rules remain
`hegemony-pdf-v1`, the pool remains `limited-v2.55-xq18-carrier-candidate`, and the
catalog has 128 ordinary cards. Current UI/Worker tests consume new Native62
fixtures. Preserved older fixtures remain historical evidence; actual Engine60
and Engine61 rooms reject before interpretation. There is no room migration.

One old prepared UI layout, `contractWinWindow`, predates saved Win contexts.
The strict current loader correctly rejects it. Its replacement comes from
`death_observer_tests::death_observer_contract_threshold_runs_existing_win_window_and_team_victory`,
which enters Win through actual current commands and exports the resulting
context. The generator checks that the eight fresh death layouts retain the
old semantics after identity normalization, allowing only the added context in
this one Win layout. It does not guess an interrupted step.

To refresh the current fixtures using the locked Native source:

```sh
export CARGO_TARGET_DIR=/absolute/path/to/reusable-disk-cache
export CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0
export CARGO_PROFILE_TEST_DEBUG=0 CARGO_PROFILE_DEV_DEBUG=0
export DEATH_OBSERVER_FRONTEND_DIR=/absolute/path/to/fresh-death-layouts
cargo test --locked -p hegemony-server death_observer_tests:: --lib
cargo run --locked -p hegemony-wasm --example current_ui_worker_fixtures -- . "$DEATH_OBSERVER_FRONTEND_DIR"
cargo run --locked -p hegemony-wasm --example seven_card_ui_fixtures -- web/src/game/sevenCardUIV062.fixture.json
```

The P1 test exporter also covers a new duel case. It retains all three original
positions after an actual capture, restores both loaders, pauses/resumes, and
performs actual mobility from position 0 to position 2 while rejecting the
vacant position 1. Duel mobility permits any other live region; the existing
five-position team case continues to prohibit jumping across a vacancy as
though it were adjacent.

`NativeFixedSlots.test.jsx` executes the original commands through the actual
WASM and renders the unmodified Native projections. It checks three/five slot
positions, retirement of the original inspector, readability of the fresh
scored instance, and JC050's sparse original indices. The scored card has a
fresh instance ID; the retired battlefield ID is not revived.

`win-flow-integration.test.mjs` executes eight continuous Native traces through
the production RoomService and real local D1, checking opaque state strings,
complete stored journals, every actual seat's private view, reopening and
original duplicate receipts. These prepared Native IDs use RoomService rather
than HTTP; the ordinary Worker HTTP integration suite verifies HTTP routing.

Independent QA evidence is separate from author logs. The exact Engine62
source passed all 898 existing workspace tests. Its 1,826 regenerated P1
records and all 433 regenerated full-corpus case hashes match the received
package. The additional duel test raises the focused suite from 19 to 20 cases.
The full current Native/WASM corpus and combined UI/Worker checks are recorded
in the independent QA report accompanying the final candidate.

These are source, automated UI, ABI and local D1 checks. The parent session
owns browser gameplay acceptance, including natural match completion.
