# ABI94 workspace regression repair — 2026-10-22

The fixed recovery/test baseline is `59ac43d0d6fa142577fd59c892e49302208f018b`. Repairs are isolated on `progress/fix-abi94-workspace-regressions-20261022` in the ordinary recovery clone. Runtime/test commits are `66bc75f09b1cb403bebfff4654b134858a7eb376` and `932de1da80809e5535688a7e0c7d7910b5de8862`. The prior recovery branches remain fixed at 1e9e679 and 59ac43d. No batch-four, A/B contest, Bot, deployment, main merge or permission change is included.

## Confirmed failures and repairs

The original default-feature workspace attempt at 59ac43d ran the full 1,590-test lib: 1,584 passed and six failed. Cargo stopped at that failed target. The complete original log is preserved as [baseline-six-failures.log.gz](evidence/abi94-workspace-regressions-2026-10-22/baseline-six-failures.log.gz), with its original invocation receipt. Its decompressed SHA256 remains `f7eebd13a347242d896bc4f3d1d613a2140f4e9c1d430fe4982a7ab7a4da4a63`.

Five failures were stale catalogue assertions: the engine complete-pool test, the second-batch cross-registry test, and the red/gray/black society originals. A separate read-only source audit confirmed that all 193 preceding records, including order, exactly match bb77; their canonical SHA256 remains `0e85e0784acef129cea818f4c2fb694912070fbb17bde53b61d52d2fb2def01e`. The added suffix is exactly 20 distinct third-batch IDs. The current pool is 213 definitions, 203 non-regions and the same ten regions. Complete red/gray/black non-region pools are 29/32/41; their complete unique subsets are 6/6/11. All 20 original-image hashes still match the reviewed source plan. Details and exact sets are in [catalog-audit.json](evidence/abi94-workspace-regressions-2026-10-22/catalog-audit.json).

The repaired tests retain the second batch's own twenty IDs and ten unique cards, and retain the society construction fixtures and deck constraints. They now additionally lock the preceding-record hash, exact added-ID set, full region set, and exact society colour/unique sets. No catalogue records or rules were changed to make the assertions pass.

The sixth failure was a real BQ035 grant regression. A BQ037 recipient's authenticated `mobility` target declaration was incorrectly routed to its printed feeding `Accept` validator. After fixing that route, the broad recipient test exposed the same error for CQ07. Both have dedicated pre-fix failing reproductions. Routing is now by source plus exact ability key, leaving the mobility witness and the printed feeding/entry validators intact. New coverage exercises real phase/pass/choice commands, wrong seats, stale/repeated choices, invalid destinations, forged witnesses/programs, optional feeding, and Game/Room recovery with four views.

## Additional workspace failures discovered after lib passed

At 66bc75f the lib passed 1,593/1,593. The native bins had zero tests and `attachment_service` passed 3/3. The newly reached `audit_regressions` target then passed 3/5: third-batch source validators had omitted the two existing finite runtime rewards, `renown` and `jc089-combat-glory`. The complete failed workspace log is retained; it is not relabelled as a successful workspace run.

The third-batch validator now recognizes only those two complete canonical reward programs, tied to the frozen original region, face-up character source, actor, exact zero-cost/zero-target program, response shell and optional `Accept` choice. It retains rejection of altered amount, region, context, payment, cursor, source and choice shape. Reward continuations still survive original-source departure. This does not add a new grant, event, schema or ABI.

The renown audit also encountered XQ22's real spell ban. Its fixture now first proves the prohibited JC074 command is rejected atomically, pays 2 to exhaust the senator, pays 2 for the real grant, and pays 2 to ready the same senator before confrontation. It asserts the original window and 2/4/6 spent assets. The audit still requires exactly the original XQ46 skip: its zero printed defense makes that synthetic target die before the grant can remain live. No new skipped recipient was accepted.

The public-command integration audit now passes 5/5: renown covers 135 live recipients plus that one disclosed XQ46 fixture skip; glory covers all 136 recipients with no skip. Two additional unit tests cover the canonical pending/frame shapes for all 15 new characters and both rewards, both storage entries, and source loss without binding a replacement. These unit tests use disclosed grant/completed-confrontation layouts and the normal reward producers; the separate integration audit supplies actual paid grants and public confrontation commands. Intermediate failed unit attempts exposed fixture assumptions about remaining influence and intermediate confrontation choices; those logs are retained too.

## Current verification status

At the exact source commit `932de1da80809e5535688a7e0c7d7910b5de8862`, the final command was:

```sh
node tools/build-storage.mjs -- cargo test --offline --locked --workspace --all-targets --no-fail-fast -- --test-threads=2
```

It exited **0** after **400.763 s**: **1,595 lib tests + 72 integration tests = 1,667 passed**, **0 failed, 0 ignored, 0 filtered**, across 17 target invocations. Both native bins, the native audit example, the host WASM lib and its four fixture examples also compiled and ran their zero-test harnesses. This host run does not execute the examples' application `main` functions.

| Integration target | Passed |
|---|---:|
| attachment_service | 3 |
| audit_regressions | 5 |
| bq030_attachment_regression | 27 |
| deck_construction | 5 |
| jc059_friendly | 2 |
| room_pacing | 12 |
| service | 10 |
| session_service | 8 |

A separate `cargo test --offline --locked --workspace --doc -- --test-threads=2` also exited 0: both crates have zero doctests. There were no skipped Rust tests. The one disclosed XQ46 matrix recipient described above is not an ignored Rust test. Only documentation/evidence is added after the tested source commit; all bound source inputs remain identical.

Directed author checks before the final workspace run:

- BQ035: 30/30, including the broad admitted-recipient test and three new focused regressions.
- Reward storage/program checks: 2/2, with the matrices described above.
- Public-command integration audit: 5/5 with the disclosed one-recipient renown fixture skip.
- Third-batch original-image/source checks: 2/2 Node tests.

The default native workspace run is the scope of this repair. The optional `society-fixtures` feature and a separate no-default-features matrix are not covered by this run. A host test of the `hegemony-wasm` crate is not a wasm32 guest rebuild or Native/WASM parity test. No new guest package, Web/Worker suite or browser acceptance is claimed. The ignored package previously built from 1e9e679 must not be described as containing these repairs.

All command logs/receipts and their hashes are indexed in `evidence/abi94-workspace-regressions-2026-10-22/results.json`. The catalogue audit subagent only read sources; it did not execute tests. Runtime changes still require the parent's independent review of the new fixed commits. Main stays unchanged until that review and its remaining applicable gates are satisfied. No GitHub CI was dispatched or rerun.
