# ABI94 recovery validation correction

This independent ordinary clone starts at `44cc35564eccc7f38bcfe5568556c3fa746d5d5f` and retains its 213 definitions, ABI94 kernel and card rules. It contains no fourth-batch or Bot changes.

`Game::from_persisted` and `RoomEnvelope::from_persisted` now use one domain validation entry with the original Game check order. The Room envelope keeps its version, seat, pacing and pause guards. Runtime command validation is unchanged.

The directed native fixture confirmed one missing coverage case: a malformed WM030 paid-frame cursor was rejected by Game recovery but accepted by Room recovery. The original explicit lists also differed in destruction validation, but both entrances already reached that check through batch20 validation. It was not another uncovered check.

The same regression group preserves honest history: after an actual reveal replaces one original JC062 target UID, both recovery entrances accept the paid frame and the entire effect subsequently fizzles, retaining payment without revealing the other target or drawing a card. This fixture uses `u64::MAX - 127` for its seed and random state. No production corruption is claimed.

Author execution: before the correction, 2/3 directed tests passed; afterwards, 3/3 passed. The 12 existing Room pacing integration tests also passed. The second successful directed run exports four synthetic opaque Room fixtures and their 16 native seat views for focused WASM comparison; it is not another three unique tests. Failed baseline logs remain in the evidence directory.

See [native evidence](evidence/abi94-recovery-validation-2026-10-22/native-recovery.json) and its source manifest and compressed logs. WASM verification and independent review are pending at this source checkpoint. This is not a deployment or a complete-match acceptance result.
