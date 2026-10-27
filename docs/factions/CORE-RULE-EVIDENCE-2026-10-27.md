# Current core source findings and implementation boundary

Sources are the tracked original `resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf` and `cards/MSBQ02 修格斯.jpg`, `BQ046 探巡者莫佳娜.jpg`, `BQ048 位格压迫.jpg`, `BQ056 便衣警探.jpg`. No third-party engine is a rule authority.

## Construction: what is explicit and what is not

Printed p13 (PDF page14) says each player deck contains at least50 cards. The official Shoggoth preset separately lists society, auxiliary core-zone card and core BQ043×1, followed by50 ordinary cards. Transcribed counts:053×2,054×2,055×3,056×2,057×2,058×2,060×2,061×2,062×1,063×3,064×2,065×2,066×2,067×2,042×2,048×3,049×2,050×3,051×2,052×2,109×2,112×1,113×1,114×1,115×1,116×1. Sum50, separate core1.

Printed p20 (PDF page21) selects one Shoggoth core before the game and starts it in the core zone. The MSBQ02 original says choose a character with subtitle Shoggoth during construction, only gray/red/neutral cards and at least12 command cards.

Thus the explicit official preset demonstrates50 ordinary cards plus one separately set-up core. It does not literally state whether49 ordinary cards plus that core could also satisfy the minimum50. Precise unresolved question: does the selected core count toward the minimum deck size (49+1), or must it be additional to at least50 ordinary cards (50+1)? Do not turn the preset example into an unsupported universal49-card permission. Natural MSBQ02 setup remains unfinished; this question does not block the already implemented in-game operations.

## Other zones and costs

Printed p20 prohibits establishing an asset or secret deployment directly from the core zone, but explicitly permits ordinary hidden/asset use if another card effect moves the core into hand. Printed p11 defines those operations separately. The temporary global BQ046 hand/asset/conceal/reveal rejections have therefore been removed; normal cost, loyalty, timing, play-source and payload checks still apply. Rest surcharge applies when playing from the core zone, not from hand. Optional departure returns and rest accounting retain the prior checkpoint's rules.

BQ046's ordinary action costs1 honor, once per turn. Its core column is only Forecast3. BQ048's printed fast command mode “发动一次核心能力” now enters that core program after paying the spell's printed money/loyalty cost. It does not charge BQ046's ordinary honor cost or create a free core button. Only this BQ048 mode is implemented; its ready/marker and core/tentacle-region damage modes remain unavailable and the whole card is closed. No completed-card claim for BQ048.

## BQ056

The pinned original states: if a local enemy character or hidden card has a lock marker, gain temporary investigation1; when your own core enters, place one lock on a target character or hidden card, then repress1. Implementation uses real Enter events, a frozen core/observer event, ordinary target/response guard, enemy-player choice then that enemy's choice of one original influence marker. Same-team other-owner cores do not count as this player's core. EnterRegion movement alone does not generate the event. Temporary icon follows the existing first-player rule.

All new candidates remain closed. Independent reviews previously rejected by the platform were not retried. Self-tests and prepared-layout Room traces are not independent review, natural lobby start, browser play or release evidence. MSBQ02's complete construction/lobby and society mechanisms remain a separate unfinished prerequisite; no admission override or fixture-only kernel was introduced.

## Validation checkpoint

Rust1.90.0, wasm-bindgen0.2.104, Node24.19.0; default native/WASM features, locked dependencies, one Cargo job and one Rust test thread. Final Native targeted suite:33 passed/0 failed/0 ignored; adjacent lifecycle/source suite:89/0/0; attached-bats regression:19/0/0. These are filtered suites, not a full-library run or additive batch-completion count.

The final327-file Rust/data/manifest hash set was unchanged across Native export and WASM build. Fresh Native raw SHA256 `cb9bfc15d0e38d778d59f3353ee120ef72c5ad3a88994eea1d0bf148bf8b2399` contains9 prepared-layout scenarios,78 actual commands,312 four-seat projections and78 actual Native duplicate transitions. Native/WASM comparison passes. Two legacy prepared-core scenarios explicitly use the internal component hook; the two BQ048 scenarios use the paid real command producer. UI52/52 and local Sites build/typecheck pass. No natural lobby/browser/full-suite claim.

Earlier failures remain in external evidence:23 tests failed at catalog initialization after an insertion into the wrong JSON collection, then3 BQ056 failures exposed an overly strict stack-target restore guard. Both were corrected before the final runs; no negative tests, admission checks or historical hashes were removed.

Local Worker9/9 passed,0 failed/0 skipped; all78 commands were replayed through HTTP with78 cold D1 reopenings and duplicate receipts. This is local component integration, not browser play.
