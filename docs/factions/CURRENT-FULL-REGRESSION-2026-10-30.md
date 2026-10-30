# Current closed-candidate regression checkpoint

Baseline: `4098526cd01c91f7a429f4d512dc1c644361f969`. No new card, admission, gameplay or production-rule changes in this checkpoint. All candidates remain closed; no independent review, push, upload, merge, deployment or browser play.

## Actual full Native result

Ran `node tools/build-storage.mjs -- cargo test --offline --locked --workspace --no-fail-fast -- --nocapture --test-threads=1` with Rust1.90.0, one build job and one test thread. Real output paths were provided for `SENTINEL_WORKER_WITNESS` and `SENTINEL_PLATFORM_WITNESS`; no candidate admission variant was enabled.

3172 library tests discovered:3170 passed,1 failed,1 ignored. Eight integration targets discovered72 and passed72 (attachment_service3, audit_regressions5, bq030_attachment_regression27, deck_construction5, jc059_friendly2, room_pacing12, service10, session_service8). Other binary/WASM native/doc targets discovered0. Workspace total3244:3242 passed,1 failed,1 ignored; exit101,3422.104 seconds including compilation. Peak child RSS4512616KiB; sampled cgroup12,615,254,016–17,179,869,184 bytes includes cache.

The sole failure was `bq035_every_admitted_human_or_vampire_uses_its_existing_validator_with_real_grant`: the local `field` helper constructed the newly registered TK009 with `make_card`, omitting actual generated identity. Existing restore validation correctly rejected it. The fix adds only `GeneratedKind::PendulumPriest` through the existing `make_generated` path; no production guard or assertion is weakened. The misleading existing error text says tentacle but the failing input is TK009. This fix is NOT YET COMPILED OR VERIFIED: the normal retry exited78 before build, with455,983,104 bytes available against the1GiB storage gate. No fallback, deletion or gate bypass occurred.

The ignored test is `bq001_candidate_natural_supply_anchor_forecast_contest_and_dream_payment`, requiring an internal candidate catalog variant; mainline admission remains closed. Both explicit witnesses ran their bodies: sentinel1 scenario/5 steps; WM0442 scenarios/6 steps. These are component evidence, not browser matches.

## UI red/green and remaining input block

Initial full UI run:209 files,123 passing/86 failing;1218 discovered assertions,1214 passed/4 failed.83 failing files could not load missing current-source fixture packets under `/workspace/lc04-validation/fixtures-final`; they did not execute assertions and are not83 gameplay defects. Exact filenames and missing paths are preserved in `ui-load-failures.json`.

Two test files had stale catalog expectations:327 current definitions,91 closed,236 public; CQ11/TK008 now exist but remain closed. The original TK010 image is PNG, with original/public SHA256 `cd0e9486e0ce96c5fb01bfa90290518b1fe6a63bf5cf614c2e6c0155a95bb3ef`; preserve exact PNG expectation rather than requiring JPG. Existing27 unavailable-face negatives and public admission checks remain. `UwDeckImportPanel` exceeded its existing5s timeout under concurrent load; no timeout or production code changed.

Affected three files passed18/18. Then all126 loadable files passed1218/1218 with one UI worker, zero skipped/failed. The83 missing-fixture files were explicitly excluded from that latter scope. This is NOT full209-file approval. UI correction commit: `e7beb5e81824879b1aa410788c00d3c99767255d`.

## Source/build scope and blockers

The full Native run tested baseline4098526 production and test inputs unchanged through terminal. The two UI test edits do not alter those inputs. The prior LC04 WASM/Web builds match baseline Rust/data source186b9f34;460 manifest files were rehashed with no mismatch before the TK009 test-only correction. WASM SHA256 `c0e9e183cb74acb7b651a2c5879028ac202eccacb1f46a90bdee340d903d6cb8`. Existing targeted Worker/WASM traces retain their original scope and identities, not full coverage. After the new Rust test correction, the exact all-Rust manifest changes: do not claim old WASM fixtures match the final manifest.

Disk remains below the project build gate. Current-source fixture regeneration for83 UI modules, fixed-test recompilation/full green Native confirmation, and Sites packaging are pending disk capacity. Old fixture outputs must not be re-signed. Evidence, unmerged work, dependencies and recovery bundles were preserved; released bytes0. This is a resource block, not an approval denial. Prior independent-review rejection, Library401 and push restrictions were not retried.

Reviewable implementation scope is the explicit92 closed source records in `FOUR-COLOR-INVENTORY-2026-10-30.json`, not92 new cards or an approved20-card batch. Remaining partial BQ049/MSBQ02 task-region reward/victory chains and unregistered BQ043/BQ058/BQ066 depend on paused neutral task inputs; MSLC03 city/support/multi-game scope remains unimplemented and excluded. No new independent approval exists.

Local evidence root: `/workspace/current-full-4098526-validation`. Full red Native/UI logs, green UI logs, exact commands/resource results, manifests, missing-input list, witness outputs, patch and recovery bundle are saved there. Local paths are not a Library delivery claim. Resume only after adequate disk capacity: run the fixed BQ035 test first, then final-source Native/fixture/UI/build confirmation without repeating unaffected old Worker matches.
