# Current 9950 Sites and cross-platform validation

**Applicable Sites coverage is complete: 114 files, 820 Node test cases passed, zero failed/cancelled/skipped/todo in the consolidated final results.** These are successful per-file results assembled after repairs and recovery, not one clean monolithic run. Node does not count individual internal `assert` calls; 820 is a test-case count.

Initial tested source was `f3fe505f9fb8e4d38e7623d4fddca03287307209`; WIP repairs were saved at `3f81c0d4c19fd3aa3c57863de155d67b7d8a3b98`. The final tested change after that checkpoint sets the next20 harness module root to its directory. This delivery adds evidence only beyond that tested change. [Machine-readable receipt](../evidence/current-9950-sites-20261030/summary.json) contains every final file result, command, test-source hash, log hash, attempt identity and local artifact inventory hash.

## Scope and runtime identity

The candidate's gameplay/runtime inputs did not change. All **460** Rust/source-manifest inputs were rehashed unchanged, and all retained build artifact hashes stayed identical. Therefore the earlier complete Native **3243 passed / 0 failed / 1 ignored**, complete UI **209 files / 1929 assertions passed**, default WASM, Web and Sites dry-run builds retain their previous scope; they were not repeated to inflate counts.

* Engine: `rust-v0.2.113-bq001-anchor-source-wip`; pool: `limited-v2.104-bq001-anchor-source-wip`.
* Catalog: 327 card definitions, 236 supported, 91 unsupported; 23 society definitions. These remain closed implementation records, not a newly approved card batch. The inherited inventory is 92 closed implementation records; no new admission, card batch, neutral/dual/MSLC03 expansion, or completion of previously partial task-region/city-support boundaries is claimed.
* Runtime WASM SHA-256: `6f3f190b47871ff6a1d4a1be6fadbd50fdf5b3884e0cf1747e18c437bedef452`.
* Source/WASM manifest SHA-256: `8712556ccd5052f2424ec694f249bab35c2872467acbb9042a6007dae0674411`.
* Production Sites `dist/server/index.js` SHA-256: `a9c3cde3dd66d302ecf738871520b988403af688df0054c9514f998f57ef1b94`.

114 of the repository's 115 Sites test files were selected. `batch4-abi95-normal-game.test.mjs`, an unfocused automated full-duel chooser, was excluded. Existing deterministic mechanism/public-entry replays were included; none are represented as the parent's personal browser match. The receipt enumerates the complete tested file set.

## Actual execution and preserved failures

The original serial batch began **15:40:13 UTC**. It completed 96 files (90 pass, six failure files) before its tool session and task processes disappeared during WM026; its last recorded progress was 800/888 commands, without an exit status. The cause remains unknown. The unchanged original record still says `running`; the separate interruption receipt explicitly supersedes that stale status. No successful result is inferred from it.

Only interrupted WM026 plus the 17 not-yet-started files were continued in a new session. All 18 passed. WM026 had to restart from its file boundary because process state was lost; its partial prior attempt is not counted as a pass or additional coverage. The six failed files were then rerun: five passed, next20 still failed startup and exited normally. After correcting its module-root directory, only next20 was rerun again and passed 5/5. Final test completion was **18:26:05 UTC**, about 166 minutes after the initial start. Long per-command Worker/D1 cold-reopen traces, one startup handle leak and one interrupted-file repeat account for the elapsed time; no broader test set was added.

Repairs and their evidence:

1. **CQ-pair fixture assembly:** the Worker consumer expects a mechanism suffix, while the initial packet contained all 1666 commands without suffix metadata. Preserve the red packet. Derive exactly 32 unchanged commands from original index 1634, set the corresponding initial state/views and relative mechanism index, and retain the Native preparation count. All 32 commands, 128 views and 32 duplicate responses matched WASM, then the Worker file passed. No opaque state or assertion was weakened.
2. **Stale catalog tests:** initial generic integration and JC035 entry asserted 278 definitions instead of 327. Twelve related test files were updated for actual catalog cardinality (327/23), keeping 236 supported cards and semantic/admission checks. Kernel binding additionally needed 91 unsupported cards instead of 42 and explicit CQ11/TK008 presence with `supported=false`, matching the Rust catalog and Native batch3 negative tests.
3. **Historical routing oracle:** prior cards and societies still compare field-for-field with the preserved old catalog. Explicitly enumerate all 11 later closed societies and assert they remain unsupported, while retaining the earlier four society identities and new-room routing checks. No historical fixture was rewritten.
4. **next20 harness:** fixed-seed wrapper startup outside the default module root failed in workerd with `can't use ".." to break out of starting directory`. Four startup failures leaked handles because cleanup was registered after awaiting startup; the unwrapped production entry case originally passed. After all five tests ended, only the hung child was terminated to unblock the original queue. Register cleanup before startup and use `modulesRoot: directory`. An intermediate incorrect filename root also failed, but cleanly exited; both red logs remain. The final five cases passed in 52.253 seconds. No production runtime code changed.

All 14 changed source files are Sites tests. Generated inputs, databases, dependencies, caches and builds remain outside Git. The earlier [WIP note](CURRENT-9950-SITES-WIP-2026-10-30.md) is a historical snapshot, not the final status.

## Cross-platform evidence boundaries

Fresh Native exports used the current unchanged source. Three batch3 exporters and five next20 Native tests were rerun only to obtain missing artifact bodies; these eight executions overlap the prior complete Native suite and are not added to its unique count. Existing current-source raw bodies were reused for other missing Worker packets. Historical filename/ABI labels are retained schema labels, not evidence of an old runtime.

17 new packet receipts include 16 packets with command comparisons: **2178 command comparisons, 6078 view comparisons, 1720 duplicate comparisons** against the actual current WASM. The XQ24 invalid-snapshot packet has zero command comparisons in the packer and is exercised by its separate Sites restore tests. These comparisons can overlap prior traces and are not a deduplicated full-library scenario sweep. The 116 final local fixture packets and 750 final diagnostic records have hashed inventories.

| Focused Worker scope | Actual coverage |
| --- | --- |
| CQ13 conversion/clock/entry/blink | 1 scenario, 329 commands, 1316 views, 329 cold reopens. Original Native preparation: 1955 commands, not claimed as Worker replay. |
| LC04 | 2 scenarios, 44 commands (15 natural suffix + 29 prepared), 176 views, 44 cold reopens. Original natural preparation: 902 commands. |
| CQ12/CQ14 pair | 1 scenario, 32 commands, 128 views, 32 cold reopens, 1634 Native preparation commands. |
| BQ035 | 13 existing positive/negative scenarios, 378 command attempts, 1510 projections; exact opaque D1 states, Native journals and restored original receipts checked. |
| Sentinel/platform witnesses | 3 scenarios, 11 commands, 56 views, 11 reloads, 22 duplicate HTTP receipts; separate normal-entry check also passed. |
| next20 public admission | 4 original Native trajectories, 1304 commands, both seat refreshes and duplicate receipts on every step, 16 cold reopens. Only random room ID/invite labels normalized. No admission override or D1 state injection. |
| Unwrapped production public flow | Separate create/join/ready/start/pause/resume case, real random seed, 5 command receipts, one cold reopen. |

The CQ13 trace above exercises its own mechanics. No distinct CQ13-with-BQ035 granted-mobility cross-platform trace was added; the complete Native all-recipient traversal and existing BQ035/CQ13 Worker scenarios remain separate evidence. The one ignored BQ001 internal-catalog Native variant remains ignored; public admission is still closed.

## Reproduction and remaining release gates

Source `../toolchains/env.sh` in the repository; use `HEGEMONY_CANDIDATE_NATIVE_FIXTURES=../validation-sites/fixtures` as an **absolute** path. Supply `CURRENT_CLOCK_WORKER` and `NEXT20_PUBLIC_WORKER` pointing to the isolated `../build/sites-verification-20261030/current-clock/worker.js` and `public-seed/worker.js`. Each file command is `node --test --test-concurrency=1 --test-reporter=tap test/<file>` in `sites`. Full raw evidence is in the isolated task's `validation-sites` directory; a fresh clone alone does not contain the generated packets. Local fixed-clock/seed wrappers are test-only and must never be deployed.

Independent review is still absent; the earlier denial was not retried or routed around. Personal browser play remains the parent's responsibility. No merge, deployment, release or CI dispatch was performed. The user's publication intent does not turn these technical results into independent review; the parent must assess the remaining release gates using the exact candidate and production artifact identities.
