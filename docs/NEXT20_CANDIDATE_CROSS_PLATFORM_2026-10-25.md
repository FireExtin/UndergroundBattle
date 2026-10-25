# NEXT20 independent candidate cross-platform evidence · 2026-10-25

Fixed tested source: `9afd8a573d8e7a3c6a416537578364846ce5534d`. Runtime implementation: `1bd07762824e138c0560863130ec7eacaddbf9c8`. This turn changes tests, exact closed-catalog expectations and explicit candidate-evidence paths only. The two runtime-looking Rust changes are `cfg(test)` declarations/test assertions; the catalog and production behavior are unchanged. See [source identity](evidence/next20-candidate-cross-platform-20261025/runtime-source-equality.json).

## Actual fixed-source results

| Scope | Result | Receipt |
| --- | --- | --- |
| Ordinary complete Native workspace baseline | **2633 passed, 0 failed**; 2561 library +72 integration, one whole run | [Native](evidence/next20-candidate-cross-platform-20261025/logs/full-native-baseline.json) |
| Fresh WASM release | PASS; 8,651,037 bytes | [Build](evidence/next20-candidate-cross-platform-20261025/logs/wasm-build.json) |
| UI TypeScript/Vite + Worker dry-run build | PASS; dry-run does not deploy | [Build](evidence/next20-candidate-cross-platform-20261025/logs/worker-build.json) |
| Fresh default Native→WASM replay | **483 cases**, 25,135 transitions, 99,903 views, 362 rejected commands, 42 rejected deck creations | [Replay](evidence/next20-candidate-cross-platform-20261025/logs/wasm-replay.json) |
| Complete UI with independent candidates | **160 files, 1584 passed, 0 failed** | [UI](evidence/next20-candidate-cross-platform-20261025/logs/ui-full.json) |
| Complete Worker with independent candidates | **66 files, 669 passed, 0 failed**, local workerd/D1 | [Worker](evidence/next20-candidate-cross-platform-20261025/logs/worker-full.json) |
| New closed-card Native→WASM witnesses | **4 continuous cases, 54 commands, 232 views, 54 real Native duplicate reducer probes** | [Comparison](evidence/next20-candidate-cross-platform-20261025/logs/pack-next20.json) |
| Candidate path contract | 1 passed; absent candidate never falls back to prior evidence | [Contract](evidence/next20-candidate-cross-platform-20261025/logs/tool-path-contract.json) |

The full Native export-enabled run separately retained **2632 passed /1 failed**: a runner variable incorrectly named a directory where BATCH4_ABI95_NATIVE_OUT requires a file. Its exact IsADirectory panic remains saved. The corrected actual Native export test passed and its six-case Native/WASM comparison passed. The ordinary 2633/0 baseline above is a fresh complete run, not a sum of targeted reruns. Earlier 1bd0776 results remain historical and were not relabeled.

## Candidate evidence contract and provenance

Existing hard-coded loaders could not select new evidence without overwriting old fixtures. `HEGEMONY_CANDIDATE_NATIVE_FIXTURES` now chooses a separate absolute candidate root for test evidence. It rejects source-tree overlap and traversal and never falls back. Existing format, source manifest, compiler bindings, catalog, WASM, raw byte/hash and view assertions remain. Original legacy fixtures and all 32 raw delivery inputs stay unchanged. The unchanged default paths still retain their original identity checks and can reject stale evidence.

The 35 prior UI collection failures and 40 distinct Worker dependency failures, exact files and fixture imports, are listed in [failure-dependencies.json](evidence/next20-candidate-cross-platform-20261025/failure-dependencies.json). The former whole-manifest mismatch was `b527bf37…` versus prior production `09ac47dd…`; three compiler-bound oracles additionally rejected the old cards.json binding. All 63 replacement candidate packets use actual newly generated Native outputs at this source. No old room state was relabeled or resigned. This snapshot's complete source manifest is `9cb74b60533a7a3e470219f75765bb7f64e7d6c6bd492127bacde4a7ff4c4a03`; WASM SHA256 is `c97f177335f0afb94e154886006c91b2a35d2d2e5113e76a4630d0b0ab8e6a02`.

The independent fixture archive contains 63 packets (38,414,910 bytes before tar compression), plus manifests and audit. It verifies 234 original raw-record byte/hash pairs, all 8,146 tracked snapshot blobs and original32 hashes. [Archive record](evidence/next20-candidate-cross-platform-20261025/fixture-archive.json): `/home/ddd/Documents/Codex/2026-10-24/task-7/checkpoints/next20-candidate-fixtures-20261025.tar.gz`; SHA256 `e9719bddc031b059e44328dbe4c45d3d3e4971411efbcb3cbb9e90a685e49d87`. It contains synthetic test data only, no database/cache/build tree or credentials. Native raw exports remain separately available in the authorized local candidate directory. The source recovery bundle is recorded in the final handoff, outside this commit to avoid a self-referential hash.

The strict classic assembler needed a resumable destination for already-generated same-source candidates; it checks exact equality and never overwrites. Its deck-seal input directory correction is recorded. A derived smoke assembler updates only the verified closed counts (277/233/44) and destination existence check; rule assertions remain. Sentinel's missing duplicate field was supplemented by directly invoking the compiled production Native Room API, then compared to WASM; no WASM-generated expectation was substituted. Derivation, library hashes, helper source and failed/successful attempts are all retained in this evidence directory.

## Reachability, public rejection and review

The four new continuous scenarios exercise BQ033 real asset/anchor payment with the actual BQ009 mechanism, BQ034 real paid entry/blood attachment, CQ25 departed-source search/shuffle/fallback grave, and XG25 real response/failure grave/original caller continuation. Initial layouts explicitly prepare the required cards/seals; they do not prove a natural supply chain. Four corresponding UI and Worker cases compare full states/journals/seat views, private chooser visibility, pause/reopen and original command receipts. The separate public case actually rejects each of the five closed definitions at HTTP create/join with no partial room/seat/receipt mutation, then exercises a valid supported Room's create/join/ready/start/choice/pause/reopen/resume lifecycle.

Reducer duplicate probes preserve actual Native semantics (a repeated BeginResponse can be accepted). HTTP receipt idempotency is verified separately, including after elapsed time and D1 reopening. Public admission stays **277 catalog /233 supported /44 closed**; **BQ009, BQ033, BQ034, CQ25, XG25 all remain supported:false**. This evidence is not four-card public admission or player playability. The 41 isolated adapter cases are not used as card-completion evidence.

[Parent review excerpts](evidence/next20-candidate-cross-platform-20261025/parent-review-excerpts.md) provide the exact shared instance matcher, BQ034 cleanup/unique shared continuation, counterexample names/assertions and BQ079 capture analysis. XG25 owner activation and CQ25 source-departure rulings remain implemented; no rule question remains pending. Parent's independent review is still required.

Remaining/NOT_RUN: missing BQ009 natural hand/deck→own-society supply (out of this turn's scope); cross-port coverage for every one of the 52 Native core-directed tests; complete card admission; new real browser play; remote Cloudflare D1/online acceptance; alternate society-fixtures feature matrix. No new cards, supply mechanisms, workers or public tables were added. Original table `97EC1A99E58C`, original32, website and deployments remain untouched.

## Recovery and commands

[Command index](evidence/next20-candidate-cross-platform-20261025/command-log-index.json) records exact commands, source pins, exit codes, durations and raw hashes. Exact raw logs are retained as deterministic `.raw.log.gz`, with readable copies separately. Failed attempts are not deleted or counted as passes. The reproduction scripts preserve their original 9950 paths; use the same frozen checkout and authorized ordinary-disk build root. The fixed-clock test wrapper is local-only.

Core commands are `cargo test --offline --locked --workspace --no-fail-fast -- --test-threads=1`, `bash rust-game-wasm/build.sh`, `npm --prefix sites run build`, `npm --prefix web test -- --maxWorkers=2`, and all 66 `sites/test/*.test.mjs` through `node --test --test-concurrency=1`, with the explicit candidate root and CURRENT_CLOCK_WORKER for the latter two. They run through the existing build-storage guard. The Native export environment and corrected BATCH4 file destination are saved verbatim; outputs are always new candidate files.

Four remote refs were re-read and remain unchanged from the previous handoff. The previously rejected workflow endpoint was not retried or accessed another way; CI/publish trigger guarantees remain unavailable. No push, GitHub CI, main merge, settings change or deployment was attempted. Work continues from local history/checkpoint/bundle.
