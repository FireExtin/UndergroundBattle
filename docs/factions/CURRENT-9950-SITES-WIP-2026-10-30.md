# 9950 Sites validation WIP checkpoint

Recorded 2026-10-30T17:37:07.793234+00:00. Base commit `f3fe505f9fb8e4d38e7623d4fddca03287307209`. This checkpoint is **not complete acceptance** and is not a release.

The existing serial run began 2026-10-30T15:40:13.630950+00:00, execution session 84724. At this snapshot, 92/114 planned files have completed: 86 passed and 6 failed. Active file: `test034-integration.test.mjs`. Remaining files, including active: 22. Original run continues without restart.

Failures pending targeted rerun: `cq-pair-integration.test.mjs`, `integration.test.mjs`, `jc035-entry.test.mjs`, `kernel-binding.test.mjs`, `kernel-routing.test.mjs`, `next20-natural.test.mjs`.

## Repairs saved before verification

* Catalog metadata assertions updated from 278 to the actual 327 definitions, and 12 to 23 societies. Supported cards remain 236; unsupported definitions are 91. CQ11/TK008 now explicitly must exist with supported=false, matching Rust catalog closure and batch3 Native assertions. No admission flag changes.
* Historical routing oracle still compares every prior card and society definition. Explicitly enumerate the 11 later closed societies and assert each stays unsupported; do not change the historical fixture.
* CQ-pair local packet initially contained the full 1666-command trace, while its Worker consumer requires the mechanism suffix. Preserve the red packet and derive exactly the unchanged 32-command suffix from original index 1634; 32 commands/128 views/32 duplicates match current WASM. No opaque state edits. This local generated packet is not committed.
* next20 local public-wrapper startup failed because the isolated output is outside the default Miniflare module root. Four startup failures leaked handles before cleanup registration; the fifth unwrapped production create/join/start test passed. After preserving errors and confirming all five tests ended, only the hung child process was terminated, letting the original queue continue. The harness now declares its module root and registers cleanup before awaiting startup. Its successful replay is still pending.

The 460 Rust inputs and runtime WASM remain unchanged; earlier Native 3243/0/1 and complete UI 209 files/1929 assertions retain their stated scope. This run uses exact WASM SHA256 `6f3f190b47871ff6a1d4a1be6fadbd50fdf5b3884e0cf1747e18c437bedef452`.

The applicable Sites scope is 114/115 repository files. Excluded `batch4-abi95-normal-game.test.mjs` is an unfocused automatic full-duel chooser; personal natural matches remain with the parent browser. No independent-review retry, merge, deployment or Actions dispatch. The catalog remains a closed implementation candidate, not an approved card batch.

[Snapshot receipt](../evidence/current-9950-sites-20261030/wip-checkpoint.json). Full original TAP logs, fixtures, exported Native inputs, hang diagnosis and process recovery are retained outside Git under `../validation-sites`.
