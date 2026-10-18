# Explicit recovery probes

`jc066-p2-restore.mjs` is an opt-in JC066 recovery probe. It is deliberately outside the default `npm test` discovery (`test/*.test.mjs`) and has no `.test.mjs` suffix. Normal default test discovery does not require its dedicated environment variables.

Use the repository-root commands below after selecting the exact Native reproduction data, WASM package and matching ten-core fixed-clock test Worker. Each command changes to `sites/` for the probe. The probe intentionally requires explicit inputs; it does not substitute a fixture or silently skip when they are missing.

For the preserved vulnerable baseline (expected acceptance, synthetic local D1 only):

```sh
cd /workspace/UndergroundBattle
node tools/build-storage.mjs -- env JC066_P2_EXPECT=accept JC066_P2_CASES_FILE=/workspace/jc066-p2-evidence-20261018/native-before/repro.json JC066_P2_PKG=/workspace/jc066-p2-evidence-20261018/baseline-pkg HEGEMONY_CURRENT_WORKER_DIR=/workspace/jc066-p2-evidence-20261018/baseline-worker bash -c 'cd sites && exec node --test --test-concurrency=1 probes/jc066-p2-restore.mjs'
```

For the corrected candidate (expected rejection):

```sh
cd /workspace/UndergroundBattle
node tools/build-storage.mjs -- env JC066_P2_EXPECT=reject JC066_P2_CASES_FILE=/workspace/jc066-p2-evidence-20261018/native-after-final/repro.json HEGEMONY_CURRENT_WORKER_DIR=/workspace/jc066-p2-evidence-20261018/worker-after/server bash -c 'cd sites && exec node --test --test-concurrency=1 probes/jc066-p2-restore.mjs'
```

`JC066_P2_PKG` defaults to `../rust-game-wasm/pkg` relative to `sites/`. `JC066_P2_CASES_FILE` and `HEGEMONY_CURRENT_WORKER_DIR` are mandatory only for this explicit probe. These are local workerd/SQLite D1 checks, not online gameplay or a deployment.

Earlier natural run receipts in `docs/evidence/jc066-p2-2026-10-18/runs/` retain the historical `test/jc066-p2-restore.test.mjs` path used at that time. Those immutable receipts are not current invocation instructions and are not rewritten. Use the commands above for new runs.
