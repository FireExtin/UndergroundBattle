# Current closed-candidate validation on isolated 9950 task

Cloned the authorized branch into a new task directory and verified exact baseline `960fc49b4900c16cb67e703e4afc7df621f9c9d2`. Development branch: `wip/9950-validation-960fc49b-20261030`. No existing project was overwritten. This is author technical evidence, not independent review or browser play.

## Actual completed checks

| Check | Result and boundary |
| --- | --- |
| Complete Native workspace | **3243 passed, 0 failed, 1 ignored**, exit 0, 1729.188 seconds including compilation. Library 3171/0/1 plus 72 integration passes. Ran once at exact baseline with `--locked --workspace --no-fail-fast -- --nocapture --test-threads=1`. |
| Witness bodies | Actual output paths supplied. Sentinel: 1 scenario/5 steps; platform: 2 scenarios/6 steps. Not skipped by absent environment variables. |
| Final complete UI | **209 files, 1929 assertions passed**, zero failures/skips, exit 0, 216.704 seconds, one worker. All previously missing fixture consumers loaded. |
| Builds | Default current WASM, Web TypeScript/Vite, and Sites local `--dry-run` all exit 0. No deployment. |

The one Native ignore remains `bq001_candidate_natural_supply_anchor_forecast_contest_and_dream_payment`, which requires an internal catalog variant. Mainline public admission remains closed. The old `4098526` 3242/1/1 result is not relabeled as current evidence.

Native ran before test-only metadata edits. All 460 Rust/source manifest inputs and the WASM bytes were rehashed afterwards with no changes. No second full Native run is claimed. Final UI tested the ten-line metadata patch whose SHA-256 is recorded in the receipt; this checkpoint also adds documentation only. Product rules, client behavior, admission flags and runtime identity were not changed.

## Fresh fixtures and preserved failures

All inputs were freshly exported by the current Rust tests/examples. The 76 generic UI fixture references were present; dedicated oracle packets also passed strict current-catalog, source, command/view and applicable restore checks. There are 102 generated packets in the isolated evidence root, including additional classic inputs. Export runs overlap the complete Native suite and are not added to its unique test count. Historical raw-format labels and filenames are schema labels; old states were not re-signed or adapted.

A fresh JC035 oracle input reproduced the stale `327 != 278` assertion. Nine oracle files now require the actual 327 definitions/23 societies, preserving whole-catalog equality, 236 supported cards, closed-card checks, source hashes and all semantic checks. The first full UI run loaded every file but finished 205/209 files, 1917 passed/12 failed assertions. Eleven failures came from this task's assembly ordering: BQ044, WM026 and MSBQ01 require the natural case first. Their fresh packets were rebuilt in the correct order without changing states or test assertions. The remaining failure was SevenCard's stale 278-definition assertion, now 327. Four affected files then passed 34/34; the separate final complete run passed 1929/1929. Original red logs and incorrectly ordered packets remain preserved.

CQ13's complete fresh 2284-command input exceeded Node's string-length limit when the raw JSON was encoded a second time into the fixture envelope. The full original remains preserved. Its UI packet is the unchanged contiguous suffix starting at command 2124: 160 commands/640 views/160 duplicate responses compared successfully against current WASM. LC04 uses its recorded first-mechanism boundary: 15 natural suffix commands plus 29 explicit-layout commands; the other 902 preparation commands remain in the original Native input. LC04 raw SHA-256 is `df117d7408fb9fcdb0a82bf5b6639852515b95bdcbc28cb0a33a7bca23b8c63d`, matching the prior cloud record. No opaque state was edited, and packet construction indices explicitly refer to the original trace.

## Toolchain and runtime identity

Official verified downloads installed only under this task: Node 24.22.1 (bundled npm 11.19.1), Rust 1.90.0 plus native/wasm32 standard libraries and rustfmt, Zig 0.15.2, and wasm-bindgen-cli 0.2.104 with `--locked`. npm dependencies used existing lockfiles. No system or shell configuration changed. The initial CLI compilation failed because Zig could not parse Rust's target spelling; only the local compiler wrapper was adjusted to the equivalent Zig target, and the retry succeeded. Both logs remain.

Current identity: `rust-v0.2.113-bq001-anchor-source-wip` / `limited-v2.104-bq001-anchor-source-wip`; 327 definitions, 236 supported, 23 societies. New WASM SHA-256: `6f3f190b47871ff6a1d4a1be6fadbd50fdf5b3884e0cf1747e18c437bedef452`. Source/WASM fixture manifest: `8712556ccd5052f2424ec694f249bab35c2872467acbb9042a6007dae0674411`. This build differs from the cloud bytes; every new packet binds this actual build.

## Evidence and remaining scope

[Machine-readable receipt](../evidence/current-9950-20261030/summary.json) records exact identities, UTC timing, commands, patch/hash boundaries and log hashes. Full local evidence is `/home/ddd/Documents/Codex/2026-10-30/task-3/validation`: `native-full-evidence.json`, `ui-full.json` (red), `ui-affected.json`, `ui-final.json`, both fixture inventories, original Native exports, packet receipts and all failure logs. Generated raw inputs, fixtures, dependencies, databases and build caches are not staged for Git.

To reuse this task's tests, source `../toolchains/env.sh` from the repository and set `HEGEMONY_CANDIDATE_NATIVE_FIXTURES` to the absolute `validation/fixtures` path. Full UI command: `npm --prefix web test -- --maxWorkers=1`. This assumes the retained local generated packets; a plain remote clone alone does not contain them.

No full Sites test suite or broad Native-to-WASM scenario sweep was run. These results do not replace independent review or the parent's personal browser match. The prior review denial was not retried, no reviewer was substituted, and no main merge, release, deployment or CI dispatch occurred. Closed implementation records do not become an approved card batch. Original partial task-region and city/support-mode boundaries remain unchanged.
