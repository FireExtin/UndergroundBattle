"""Archive only reproducible source/text/test evidence after all fixed runs finish."""
from pathlib import Path
import datetime, gzip, hashlib, json, re, shutil, subprocess

root = Path(__file__).resolve().parent
repo = Path('/home/ddd/Documents/Codex/2026-10-21/task-3/UndergroundBattle-game-serial')
source = root / 'source'
pin = '9afd8a573d8e7a3c6a416537578364846ce5534d'
runtime = '1bd07762824e138c0560863130ec7eacaddbf9c8'
destination = root / 'deliverable'
assert not destination.exists()
evidence = destination / 'docs/evidence/next20-candidate-cross-platform-20261025'
evidence.mkdir(parents=True)
hash_bytes = lambda b: hashlib.sha256(b).hexdigest()
read = lambda name: json.loads((root / name).read_text())
required = ['full-native-baseline', 'wasm-build', 'worker-build', 'worker-clock', 'wasm-replay', 'ui-full', 'worker-full', 'pack-next20', 'tool-path-contract']
for name in required:
    d = read('logs/' + name + '.json')
    assert d['source_commit'] == pin and d['state'] == 'complete' and d['exit_code'] == 0, name

native_text = (root / 'logs/full-native-baseline.log').read_text()
native_rows = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed;', native_text)
assert sum(int(p) for p, _ in native_rows) == 2633
assert not any(int(f) for _, f in native_rows)
worker_text = (root / 'logs/worker-full.log').read_text()
def node_count(name):
    m = re.search(r'^ℹ ' + name + r' (\d+)\s*$', worker_text, re.M)
    assert m, name
    return int(m[1])
worker_counts = {k: node_count(k) for k in ['tests', 'suites', 'pass', 'fail', 'cancelled', 'skipped', 'todo']}
assert worker_counts['tests'] == worker_counts['pass'] and not any(worker_counts[k] for k in ['fail', 'cancelled', 'skipped', 'todo'])
worker_header = json.loads(worker_text.splitlines()[0])
assert worker_header['sourceFiles'] == 66
new_worker_names = [line for line in worker_text.splitlines() if line.startswith('✔ Internal closed next20 ') or line.startswith('✔ public closed next20:')]
assert len(new_worker_names) == 5
ui_text = (root / 'logs/ui-full.log').read_text()
assert re.search(r'Test Files\s+160 passed \(160\)', ui_text)
assert re.search(r'Tests\s+1584 passed \(1584\)', ui_text)
replay = json.loads((root / 'logs/wasm-replay.log').read_text().splitlines()[-1])
next20 = json.loads((root / 'logs/pack-next20.log').read_text().splitlines()[-1])
archive = read('fixture-archive.json')

metadata_names = [
    'frozen-source.json', 'candidate-fixture-manifest.json', 'candidate-audit.json',
    'fixture-archive.json', 'original32-archive.json', 'preserved-original32.json', 'failure-dependencies.json', 'export-path-diagnostic.json',
    'interface-change-list.json', 'native-env.json', 'native-env-corrected.json',
    'native-build-cache-reuse.json', 'native-probe-libraries.json',
    'native-probe-libraries-attempt1.json', 'native-probe-relative-depfile-provenance.json',
    'native-probe-compile-args.json', 'classic-resume-derivation.json',
    'smoke-packer-derivation.json', 'selector-input-manifest.json',
    'xq18-selected-manifest.json', 'runtime-source-equality.json',
    'remote-ref-check-1007.json', 'history-provenance.json', 'batch20-review-compare.json', 'batch4-compare.json',
    'rewards-compare.json', 'sentinel-compare.json',
]
for name in metadata_names:
    shutil.copy2(root / name, evidence / name)
repro = evidence / 'reproduction'
repro.mkdir()
for name in ['run_stage.py', 'run_all_worker.mjs', 'pack_direct.py', 'assemble_selectors.mjs',
             'select_classic_traces.py', 'pack_next20.mjs', 'pack_remaining.py',
             'finalize_small_packets.mjs', 'current-bq030-new-output.mjs',
             'current-classic-resume.mjs', 'pack-smoke-current-counts.mjs',
             'supplement_native_duplicates.rs', 'run_compile_probe.py',
             'audit_candidate.py', 'assemble_evidence.py']:
    shutil.copy2(root / name, repro / name)

logs = evidence / 'logs'
logs.mkdir()
log_index = []
ansi = re.compile(r'\x1b\[[0-9;?]*[ -/]*[@-~]')
for p in sorted((root / 'logs').glob('*.log')):
    raw = p.read_bytes()
    # Exact bytes survive in .raw.log.gz, including failures and ANSI output.
    zipped = logs / (p.stem + '.raw.log.gz')
    zipped.write_bytes(gzip.compress(raw, compresslevel=9, mtime=0))
    clean = '\n'.join(line.rstrip() for line in ansi.sub('', raw.decode(errors='replace')).splitlines()).rstrip() + '\n'
    (logs / (p.stem + '.log')).write_text(clean)
    receipt = p.with_suffix('.json')
    entry = {'stage': p.stem, 'raw_log_sha256': hash_bytes(raw), 'raw_log_bytes': len(raw),
             'raw_log_gzip': str(zipped.relative_to(evidence)), 'readable_log': 'logs/' + p.stem + '.log'}
    if receipt.exists():
        shutil.copy2(receipt, logs / receipt.name)
        d = json.loads(receipt.read_text())
        assert d['state'] == 'complete', p.stem
        assert d.get('log_sha256') == hash_bytes(raw), p.stem
        entry.update(source_commit=d.get('source_commit'), precommit_test=d.get('precommit_test', False),
                     command=d.get('command'), exit_code=d.get('exit_code'), seconds=d.get('seconds'))
    else:
        entry['scope'] = 'retained supplemental compiler diagnostic; not a successful test receipt'
    log_index.append(entry)
(evidence / 'command-log-index.json').write_text(json.dumps(log_index, ensure_ascii=False, indent=2) + '\n')

original_four = (root / 'native/single/next20_candidate_native_out.json').read_bytes()
assert hash_bytes(original_four) == next20['rawNativeSha256']
(evidence / 'new-four-original-native.raw.json.gz').write_bytes(gzip.compress(original_four, compresslevel=9, mtime=0))
replay_index = (root / 'replay/native.json').read_bytes()
assert len(replay_index) < 2_000_000
(evidence / 'default-replay-native-index.raw.json.gz').write_bytes(gzip.compress(replay_index, compresslevel=9, mtime=0))
(evidence / 'original-native-inputs.json').write_text(json.dumps({
    'new_four': {'path': 'native/single/next20_candidate_native_out.json', 'bytes': len(original_four), 'sha256': hash_bytes(original_four), 'archive': 'new-four-original-native.raw.json.gz'},
    'default_replay_index': {'path': 'replay/native.json', 'bytes': len(replay_index), 'sha256': hash_bytes(replay_index), 'archive': 'default-replay-native-index.raw.json.gz', 'payloads': '483 newly exported payload files retained in authorized local replay directory; export command is recorded'},
}, indent=2) + '\n')

diff = subprocess.check_output(['git', 'diff', 'e26640b4a8a82fad0fad89ba7552a2933c110bba', pin], cwd=repo)
(evidence / 'source-diff-e266-to-9afd.patch.gz').write_bytes(gzip.compress(diff, compresslevel=9, mtime=0))
review = [
    '# Parent review excerpts at ' + pin,
    '',
    'Production methods below are unchanged from runtime 1bd0776. The two blue review fixes were already made in d0b3e325; their real-command counterexamples pass again in the complete fixed-source Native baseline.',
    '',
    '1. Stack transaction identity: the public/general validator previously did not establish the required blue shell instance relation. The card is deliberately freshened, so literal source-card ID equality is incorrect. Both blue and black call the shared matcher. It requires the full fresh-card snapshot, source UID < transaction UID and transaction UID + 1 == frame UID. The deploy/reveal counterexamples replace only the actual transaction UID with the retired source UID and require restoration rejection; they also reject damage-only snapshot tampering.',
    '',
]
def excerpt(path, first, last):
    lines = (source / path).read_text().splitlines()
    return '\n'.join([f'`{path}:{first}`', '```rust'] + [f'{i+1}: {lines[i]}' for i in range(first - 1, last)] + ['```', ''])
review += [excerpt('rust-game/src/pack_a_contest.rs', 295, 310),
           'Call sites: next20_blue_vampires.rs:988, next20_black_wraiths.rs:770. Counterexamples: next20_blue_core_tests.rs:394–427.\n',
           excerpt('rust-game/src/next20_blue_core_tests.rs', 404, 419),
           '2. BQ034 shown completion only clears its own post-frame. It queues no continuation. The shared mystic_after_next advances next_shown and queues one MysticContinue before dispatch; the later shown WM061 mills once. Both no-target and no-blood tests require exactly one history, only seat 2 grave +1, no pending/stack/continuation left. The target branch rejects duplicated/missing continuation and replayed choice.\n',
           excerpt('rust-game/src/next20_blue_vampires.rs', 1230, 1245),
           excerpt('rust-game/src/mystic_contest.rs', 371, 378),
           excerpt('rust-game/src/mystic_contest.rs', 396, 403),
           'Counterexamples: next20_blue_core_review_no_target_continues_later_shown_exactly_once and next20_blue_core_review_no_blood_continues_later_shown_exactly_once; assertions at next20_blue_core_tests.rs:430–492.\n',
           excerpt('rust-game/src/next20_blue_core_tests.rs', 476, 484),
           '3. BQ079 None capture: commit_owner_grave only captures observers when the arriving printed name is 无名尸体 and its owner is alive (batch3_ghoul.rs:26–39). None means recapture at the physical arrival only for that printed name. The delayed pregrave branch accepts immutable XG25, whose name is not 无名尸体, so the questioned path has no applicable BQ079 observation. Ordinary simultaneous deaths still pass frozen observers. No rule or production change was made for this hypothesis. Existing simultaneous-death and non-corpse tests pass in the complete Native baseline.\n',
           'The XG25 and CQ25 complete method excerpts remain in the preceding fixed-validation evidence archive; their production bytes are unchanged. See runtime-source-equality.json.\n']
(evidence / 'parent-review-excerpts.md').write_text('\n'.join(review))

summary = {
    'at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'fixed_tested_source_commit': pin, 'runtime_implementation_commit': runtime,
    'scope': 'fresh independent candidate evidence; all five definitions remain publicly closed',
    'native_complete_baseline': {'pass': 2633, 'fail': 0, 'library': 2561, 'integration': 72, 'one_complete_run': True, 'receipt': 'logs/full-native-baseline.json'},
    'native_export_run': {'pass': 2632, 'fail': 1, 'reason': 'BATCH4_ABI95_NATIVE_OUT was a directory, exporter expected a file', 'retained_failure': True, 'corrected_native_export_retest': 'logs/native-export-batch4.json'},
    'WASM_build': {'exit_code': 0, 'bytes': 8651037, 'sha256': replay['wasmSha256']},
    'UI_and_Worker_build': {'exit_code': 0, 'UI_typescript_and_vite': True, 'Worker_wrangler_dry_run_only': True},
    'WASM_complete_default_replay': replay,
    'UI_complete': {'files': 160, 'pass': 1584, 'fail': 0, 'candidate_path_explicit': True},
    'Worker_complete': {'files': 66, **worker_counts, 'candidate_path_explicit': True, 'local_workerd_D1_only': True},
    'new_four_internal_native_WASM': next20, 'new_four_worker_test_names': new_worker_names,
    'test_path_contract': {'pass': 1, 'fail': 0, 'no_fallback': True},
    'fixture_archive': archive, 'preserved_original32_archive': read('original32-archive.json'), 'candidate_source_manifest_sha256': read('candidate-fixture-manifest.json')['currentSourceManifestSha256'],
    'public_admission': {'catalog_total': 277, 'supported': 233, 'closed': 44, 'five_supported_false': ['BQ009', 'BQ033', 'BQ034', 'CQ25', 'XG25']},
    'NOT_RUN_or_remaining': [
        'Natural public reachability of the four cards; BQ009 hand/deck to own-society supply is absent and out of current scope',
        'Cross-port replay of every one of the 52 Native core-directed tests; only four new continuous internal witnesses are added here',
        'Complete public card admission/release and independent parent review',
        'New real-browser play, external Cloudflare D1/online acceptance, alternate society-fixtures feature matrix',
    ],
    'preserved': ['original 32 raw inputs unchanged', 'finished table 97EC1A99E58C unchanged', 'public website and deployments unchanged', 'all original failure and diagnostic logs retained'],
    'push': 'NOT_ATTEMPTED', 'CI': 'NOT_TRIGGERED', 'deploy': 'NOT_ATTEMPTED',
    'workflow_read': 'Prior endpoint allowlist rejection not retried; trigger guarantee unavailable; local checkpoint/bundle retained',
}
(evidence / 'summary.json').write_text(json.dumps(summary, ensure_ascii=False, indent=2) + '\n')
report = f'''# NEXT20 independent candidate cross-platform evidence · 2026-10-25

Fixed tested source: `{pin}`. Runtime implementation: `{runtime}`. This turn changes tests, exact closed-catalog expectations and explicit candidate-evidence paths only. The two runtime-looking Rust changes are `cfg(test)` declarations/test assertions; the catalog and production behavior are unchanged. See [source identity](evidence/next20-candidate-cross-platform-20261025/runtime-source-equality.json).

## Actual fixed-source results

| Scope | Result | Receipt |
| --- | --- | --- |
| Ordinary complete Native workspace baseline | **2633 passed, 0 failed**; 2561 library +72 integration, one whole run | [Native](evidence/next20-candidate-cross-platform-20261025/logs/full-native-baseline.json) |
| Fresh WASM release | PASS; 8,651,037 bytes | [Build](evidence/next20-candidate-cross-platform-20261025/logs/wasm-build.json) |
| UI TypeScript/Vite + Worker dry-run build | PASS; dry-run does not deploy | [Build](evidence/next20-candidate-cross-platform-20261025/logs/worker-build.json) |
| Fresh default Native→WASM replay | **483 cases**, 25,135 transitions, 99,903 views, 362 rejected commands, 42 rejected deck creations | [Replay](evidence/next20-candidate-cross-platform-20261025/logs/wasm-replay.json) |
| Complete UI with independent candidates | **160 files, 1584 passed, 0 failed** | [UI](evidence/next20-candidate-cross-platform-20261025/logs/ui-full.json) |
| Complete Worker with independent candidates | **66 files, {worker_counts['pass']} passed, 0 failed**, local workerd/D1 | [Worker](evidence/next20-candidate-cross-platform-20261025/logs/worker-full.json) |
| New closed-card Native→WASM witnesses | **4 continuous cases, 54 commands, 232 views, 54 real Native duplicate reducer probes** | [Comparison](evidence/next20-candidate-cross-platform-20261025/logs/pack-next20.json) |
| Candidate path contract | 1 passed; absent candidate never falls back to prior evidence | [Contract](evidence/next20-candidate-cross-platform-20261025/logs/tool-path-contract.json) |

The full Native export-enabled run separately retained **2632 passed /1 failed**: a runner variable incorrectly named a directory where BATCH4_ABI95_NATIVE_OUT requires a file. Its exact IsADirectory panic remains saved. The corrected actual Native export test passed and its six-case Native/WASM comparison passed. The ordinary 2633/0 baseline above is a fresh complete run, not a sum of targeted reruns. Earlier 1bd0776 results remain historical and were not relabeled.

## Candidate evidence contract and provenance

Existing hard-coded loaders could not select new evidence without overwriting old fixtures. `HEGEMONY_CANDIDATE_NATIVE_FIXTURES` now chooses a separate absolute candidate root for test evidence. It rejects source-tree overlap and traversal and never falls back. Existing format, source manifest, compiler bindings, catalog, WASM, raw byte/hash and view assertions remain. Original legacy fixtures and all 32 raw delivery inputs stay unchanged. The unchanged default paths still retain their original identity checks and can reject stale evidence.

The 35 prior UI collection failures and 40 distinct Worker dependency failures, exact files and fixture imports, are listed in [failure-dependencies.json](evidence/next20-candidate-cross-platform-20261025/failure-dependencies.json). The former whole-manifest mismatch was `b527bf37…` versus prior production `09ac47dd…`; three compiler-bound oracles additionally rejected the old cards.json binding. All 63 replacement candidate packets use actual newly generated Native outputs at this source. No old room state was relabeled or resigned. This snapshot's complete source manifest is `9cb74b60533a7a3e470219f75765bb7f64e7d6c6bd492127bacde4a7ff4c4a03`; WASM SHA256 is `c97f177335f0afb94e154886006c91b2a35d2d2e5113e76a4630d0b0ab8e6a02`.

The independent fixture archive contains 63 packets (38,414,910 bytes before tar compression), plus manifests and audit. It verifies 234 original raw-record byte/hash pairs, all 8,146 tracked snapshot blobs and original32 hashes. [Archive record](evidence/next20-candidate-cross-platform-20261025/fixture-archive.json): `{archive['path']}`; SHA256 `{archive['sha256']}`. It contains synthetic test data only, no database/cache/build tree or credentials. Native raw exports remain separately available in the authorized local candidate directory. The source recovery bundle is recorded in the final handoff, outside this commit to avoid a self-referential hash.

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
'''
(destination / 'docs/NEXT20_CANDIDATE_CROSS_PLATFORM_2026-10-25.md').write_text(report)
print(json.dumps({'deliverable': str(destination), 'summary': summary, 'evidence_files': sum(p.is_file() for p in evidence.rglob('*'))}, ensure_ascii=False, indent=2))
