# ABI94 recovery validation correction

This independent ordinary clone starts at `44cc35564eccc7f38bcfe5568556c3fa746d5d5f` and retains its 213 definitions, ABI94 kernel and card rules. It contains no fourth-batch or Bot changes.

`Game::from_persisted` and `RoomEnvelope::from_persisted` now use one domain validation entry with the original Game check order. The Room envelope keeps its version, seat, pacing and pause guards. Runtime command validation is unchanged.

The directed native fixture confirmed one missing coverage case: a malformed WM030 paid-frame cursor was rejected by Game recovery but accepted by Room recovery. The original explicit lists also differed in destruction validation, but both entrances already reached that check through batch20 validation. It was not another uncovered check.

The same regression group preserves honest history: after an actual reveal replaces one original JC062 target UID, both recovery entrances accept the paid frame and the entire effect subsequently fizzles, retaining payment without revealing the other target or drawing a card. This fixture uses `u64::MAX - 127` for its seed and random state. No production corruption is claimed.

Author execution: before the correction, 2/3 directed tests passed; afterwards, 3/3 passed. The 12 existing Room pacing integration tests also passed. The second successful directed run exports four synthetic opaque Room fixtures and their 16 native seat views for focused WASM comparison; it is not another three unique tests. Failed baseline logs remain in the evidence directory.

See [native evidence](evidence/abi94-recovery-validation-2026-10-22/native-recovery.json) and its source manifest and compressed logs. The source checkpoint was `d9e9fe114265eecd54c9a6c6e3c8d15a4eb4ad23`. The later [WASM evidence](evidence/abi94-recovery-validation-2026-10-22/wasm-recovery.json) confirms all 16 native seat views, both same-error rejections and both honest before/after target-loss snapshots in the final path-normalized ABI94 build. Its [five-output build recipe](evidence/abi94-recovery-validation-2026-10-22/wasm-build.json) retains the missing-cc failure and first unmapped output history. Web typecheck/build and official Worker dry-run passed. The isolated loopback Worker returned 200 for health, catalogue and static assets; its full catalogue equals the fresh WASM catalogue and its sole WASM module has the same SHA. The two initial raw-source-versus-runtime assertion errors are preserved separately. Independent review is pending. This is not a deployment or a complete-match acceptance result.

To repeat the focused packaged-fixture check after building this source:

```sh
node rust-game-wasm/tests/recovery-parity.mjs docs/evidence/abi94-recovery-validation-2026-10-22/native-fixtures
```
