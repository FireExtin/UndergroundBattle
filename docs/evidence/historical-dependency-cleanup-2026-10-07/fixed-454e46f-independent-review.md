# Fixed B-only independent closure — 2026-10-07

Conclusion: **no open P0, P1, P2 or P3 finding** in fixed B-only commit `454e46f67f4a55814321b66817b704d74fbad2bc`, parent `27b2d573073ff7211facc9ebe8feb2826cecbe40`. It is suitable for the authorized normal push of the shared branch. This conclusion does not authorize main changes, publication or inclusion of the uncommitted sealing/search work.

This is a fresh fixed-commit review in the existing ordinary clone `/tmp/undergroundbattle-shared-cloud`. Product/core reads used Git objects of the fixed commits; the concurrent Rust/data WIP was excluded. I created no clone/worktree, performed no Git mutation and did not compile Cargo or run a browser.

## Fixed scope and receipts

- The fixed diff contains 243 paths: the 237 B-manifest paths plus six new documentation/evidence files. The 28 nonhistorical B paths are **27 retained/new files with exact size/SHA-256 checks, and deletion of `sites/test/fixtures/frozen-kernel.mjs`**. They are not 28 live hashed files. The other 209 paths are historical kernel files.
- All 237 manifest entries correspond to the actual fixed Git diff. Manifest SHA-256 is `f4a3a0a9e1dbad4a3063d216a5066926685699eb5bd2a4d5ba9d1d041b7643e5`, matching `final-summary.json`.
- All 209 deletion-receipt sizes/SHA-256 values were recomputed from the exact parent Git blobs. Their total is 73,678,775 bytes in exactly the 40 listed `rust-game-wasm/legacy-*` directories. Those files/directories are absent from the fixed tree and local clone. No other historical directory is silently included in the receipt. The separate regenerable stage totals also add up to the stated 147,774,411 bytes.
- The six new docs were read from fixed Git objects. Their summaries preserve the actual execution boundary, identify existing accepted a96 input separately from root's fresh 5eb native input, contain no raw room IDs, credentials or private runtime snapshots, and describe only an exact-directory recovery from Git history. The restore recipe avoids the current pkg, core and WIP; it was not executed.
- The JZ55 title now says it checks the complete printed definition. The three real own-hand WASM tests retain truthful current-WASM labels. The pre-review P3 label finding is closed. Recorded historic views are explicitly presentation/privacy evidence, not current reducer/projector regression.

## Product invariance and preserved assertions

The complete runtime trees are identical between parent and fixed commit:

| Tree | Git tree ID in both commits |
| --- | --- |
| `rust-game` | `bcef26a909adb20e2fe737f15082cbe7000bc531` |
| `rust-game-wasm/src` | `188680abc5cd379e03ee2bd8ba85c22662b49335` |
| `sites/src` | `441c403bb669e2530da4b7aa0a8539013aff227a` |

The actual ignored source pkg and Sites generated WASM remain 2,198,305 bytes with SHA-256 `afc1b42523111dc242fd21850da9af243112a619c9f759bc60cbabac045883f7`; their JS modules are also byte-equal. Both dependency locks are unchanged. The fixed diff passes `git diff --check`.

The current transition/checkpoint/whole-seat-view/continuous-chain assertion bodies in all six WASM comparison scripts are byte-identical to their corresponding parent bodies, after excluding the removed legacy setup/assertions. They still execute real commands, compare native expected results, preserve atomic rejection and exact final chain state; no historic state identity relabelling was added. The finite current-kernel helper loads an actual ABI. The MSJC09 substantive body, from deck construction through both paid reopen/retry/reset scenarios, is byte-identical to the parent; only legacy selection/catalog assumptions and its execution label changed.

The exact static fixture blobs match those I actually checked against all 40 old ABIs before deletion: `historical-rooms.json` 41,820 bytes/SHA-256 `daa452f4b7fa5bd695f29c02a2291cbddd61b0eaa87d04fde28c2afd6976a124`; `reviewed-catalog-v044.json` 95,650 bytes/SHA-256 `49386a177863cf15ba82a38811ba938e17e9c7c8600e5d568524ca2670b72901`. That earlier check covered all 40 identities and Git WASM hashes, exact ABI-created 1–10 lobby states/views, and the v044 catalog values. I did not restore or re-execute deleted ABIs during this closure. The preserved pre-review report remains 5,070 bytes/SHA-256 `a2691ef368254d0bd92b6b4d2dfc5025a27b41e80e10db22996db2e3d0305b41`.

## Actual independent check and root evidence

I independently reran the **three kernel-routing tests** only, after confirming their local inputs equal fixed Git blobs. All passed: current pkg/generated/Worker byte equality and room flow; all 40 historical identity tuples plus five altered identities reject before any current state operation; the production generated modules and Worker carry only the current WASM. Log: `/tmp/b-fixed-454e46f-kernel-routing.log`, 534 bytes/SHA-256 `557cb690b230d4ed888bcf63eb3ae99c25b8de40f77db01b8bb6ec140cc4931a`.

I read and hashed root/worker's actual postdelete logs, rather than claiming to rerun their full suites: Sites 25/0 (including current engine45 MSJC09, 461 accepted commands, 2,346 HTTP calls, four seats and two paid frame reopens), staged routing 3/0, Web 605/0 across 68 files, and JZ55 focused 8/0. Root Web log SHA-256 `2868426e56258cb141234a64d3ede10c256782b377a753218cdc672dcd61a7ef` matches fixed `root-validation.json`. Fixed-5eb native logs total 571 default and 356 society-fixtures passes, with no failures. I did not rerun those builds.

Root's fresh 5eb JC089 evidence archive is actually 15,481,632 bytes/SHA-256 `4874f92f55625f20bea625ab9ae6b64660c1a5b042f6698448e2a28cf1ba549a`, matching the fixed root receipt. I independently streamed and checked **all 2,999 archive files** against the local source-bound evidence manifest (SHA-256 `d1108e5ec22c0ea35b12613f7167f897084d9b36e5b0f2feb36a16f0bd5820af`, source `5eb5127008188bf445399b74f8909632ccbfce54`). The archive contains 2,033 checkpoint rows, 938 command rows (933 normal, five native-rule rejection rows), 27 frontend fixtures and the 13-step paid response chain. All inspected native state identities are schema3/rules `hegemony-pdf-v1`/engine45/pool42. The root fresh native log records 26 focused passes and the root parity summary records 11,936 views. This fresh run used the predelete comparison, including the old44 check; it is distinct from the worker's postdelete comparison using accepted a96 inputs. I verified the archive/logs/binding, not a new native/WASM execution of that whole suite.

## Limits

B removes executable historical dependencies; its recorded historical identities/lobbies/catalog are immutable evidence and rejection inputs. It does not provide current gameplay regression merely by displaying old static views. There were no fresh exports of the other comparator families in this batch and no society-fixtures WASM Worker run. No UI, terminal strategy-game, sealing/search rules, main or public deployment acceptance is inferred. The staged cleanup only concerns regenerable copies. No P0–P3 blocker remains within this fixed B-only scope.
