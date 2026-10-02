# Frozen rust-v0.2.5 kernel

Preserve `hegemony-pdf-v1 / limited-v2.3 / rust-v0.2.5`, schema3 RoomEnvelope, for rooms created with the ten-world and server response-intent release. Executable source: main commit `6f5108ea8156df6959e3b630cf7a209f70d767d5`; deployed Site7 source `f2be5a565282ef89df1ac30f32de3d8cadde0267`, also unchanged in Site8 source `2757673f19805e00b76cb2a01a891a9ebca76565`.

WASM SHA256: `22299f6db9923a93ea7da7bd742afbf0a0b1e4de4ff3c41bd99f16860df3a971` (1,487,505 bytes). The JS glue and WASM are copied byte-for-byte before subsequent core changes. This kernel retains 26 player definitions, ten distinct single-copy base worlds, frozen deck snapshots, persistent choice frames and per-player server response clocks. All applyRoom/pollRoom/quoteRoom operations and original receipts must continue routing to this bundle for its exact version tuple and schema.

Existing evidence: 81 distinct native tests; final real native/wasm32 oracle of 16 scenarios, 537 durable states, 1,673 seat views and one quote; fourteen Worker/D1 host/integration tests. These are bounded regression results, not complete original-card equivalence or a new natural four-player game acceptance. See `docs/RELEASE_V025_ACCEPTANCE_2026-10-02.md` in the main repository.
