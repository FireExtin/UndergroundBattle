# Current core source findings and implementation boundary

Sources are the tracked original `resource/ymsj-fun.github.io/public/docs/霸权说明书.pdf` and `cards/MSBQ02 修格斯.jpg`, `BQ046 探巡者莫佳娜.jpg`, `BQ048 位格压迫.jpg`, `BQ056 便衣警探.jpg`. No third-party engine is a rule authority.

## Construction: what is explicit and what is not

Printed p13 (PDF page14) says each player deck contains at least50 cards. The official Shoggoth preset separately lists society, auxiliary core-zone card and core BQ043×1, followed by50 ordinary cards. Transcribed counts:053×2,054×2,055×3,056×2,057×2,058×2,060×2,061×2,062×1,063×3,064×2,065×2,066×2,067×2,042×2,048×3,049×2,050×3,051×2,052×2,109×2,112×1,113×1,114×1,115×1,116×1. Sum50, separate core1.

Printed p20 (PDF page21) selects one Shoggoth core before the game and starts it in the core zone. The MSBQ02 original says choose a character with subtitle Shoggoth during construction, only gray/red/neutral cards and at least12 command cards.

Thus the explicit official preset demonstrates50 ordinary cards plus one separately set-up core. It does not literally state whether49 ordinary cards plus that core could also satisfy the minimum50. Precise unresolved question: does the selected core count toward the minimum deck size (49+1), or must it be additional to at least50 ordinary cards (50+1)? Do not turn the preset example into an unsupported universal49-card permission. Natural MSBQ02 setup remains unfinished; this question does not block the already implemented in-game operations.

## Other zones and costs

Printed p20 prohibits establishing an asset or secret deployment directly from the core zone, but explicitly permits ordinary hidden/asset use if another card effect moves the core into hand. Printed p11 defines those operations separately. The temporary global BQ046 hand/asset/conceal/reveal rejections have therefore been removed; normal cost, loyalty, timing, play-source and payload checks still apply. Rest surcharge applies when playing from the core zone, not from hand. Optional departure returns and rest accounting retain the prior checkpoint's rules.

BQ046's ordinary action costs1 honor, once per turn. Its core column is only Forecast3. BQ048's printed fast command mode “发动一次核心能力” now enters that core program after paying the spell's printed money/loyalty cost. It does not charge BQ046's ordinary honor cost or create a free core button. At checkpoint1c31c7b4 only this BQ048 mode was implemented; the later damage-mode increment is recorded below. The whole card remains closed. No completed-card claim for BQ048.

## BQ056

The pinned original states: if a local enemy character or hidden card has a lock marker, gain temporary investigation1; when your own core enters, place one lock on a target character or hidden card, then repress1. Implementation uses real Enter events, a frozen core/observer event, ordinary target/response guard, enemy-player choice then that enemy's choice of one original influence marker. Same-team other-owner cores do not count as this player's core. EnterRegion movement alone does not generate the event. Temporary icon follows the existing first-player rule.

All new candidates remain closed. Independent reviews previously rejected by the platform were not retried. Self-tests and prepared-layout Room traces are not independent review, natural lobby start, browser play or release evidence. MSBQ02's complete construction/lobby and society mechanisms remain a separate unfinished prerequisite; no admission override or fixture-only kernel was introduced.

## Validation checkpoint1c31c7b4

Rust1.90.0, wasm-bindgen0.2.104, Node24.19.0; default native/WASM features, locked dependencies, one Cargo job and one Rust test thread. Final Native targeted suite:33 passed/0 failed/0 ignored; adjacent lifecycle/source suite:89/0/0; attached-bats regression:19/0/0. These are filtered suites, not a full-library run or additive batch-completion count.

The final327-file Rust/data/manifest hash set was unchanged across Native export and WASM build. Fresh Native raw SHA256 `cb9bfc15d0e38d778d59f3353ee120ef72c5ad3a88994eea1d0bf148bf8b2399` contains9 prepared-layout scenarios,78 actual commands,312 four-seat projections and78 actual Native duplicate transitions. Native/WASM comparison passes. Two legacy prepared-core scenarios explicitly use the internal component hook; the two BQ048 scenarios use the paid real command producer. UI52/52 and local Sites build/typecheck pass. No natural lobby/browser/full-suite claim.

Earlier failures remain in external evidence:23 tests failed at catalog initialization after an insertion into the wrong JSON collection, then3 BQ056 failures exposed an overly strict stack-target restore guard. Both were corrected before the final runs; no negative tests, admission checks or historical hashes were removed.

Local Worker9/9 passed,0 failed/0 skipped; all78 commands were replayed through HTTP with78 cold D1 reopenings and duplicate receipts. This is local component integration, not browser play.

## Follow-up: explicitly legal50+1 path and damage mode

The selected development path is50 ordinary cards plus a separately selected core.49+1 remains unsupported/pending, not declared illegal. The present registry cannot yet build a genuine MSBQ02 deck: it has only two command names,BQ064 and closed BQ048, hence at most6 command cards under the printed3-copy rule, below MSBQ02's minimum12. MSBQ02 itself has no executable society definition or lobby/core construction field. The official preset's BQ043 core and several ordinary cards are also not implemented. These are engineering prerequisites; they must not be hidden with excessive copies, unsupported-card placeholders, prepared layouts or a public admission override.

The smallest candidate route toward12 command cards needs at least two further distinct command names. BQ050 and BQ051 have pinned complete text and no token generation requirement; their society restriction/loss-control interactions still require MSBQ02 integration. BQ049/BQ052 involve the unresolved full tentacle token identity and are not used as shortcuts. This is a dependency inventory, not a claim those cards are implemented.

BQ048's damage mode now uses the actual paid spell frame and standard character target guard: the target must be in a region containing the acting player's visible selected core or controlled tentacle character. Eligibility is rechecked after responses. The core's movement removes eligibility; hidden cores and other regions do not qualify. Target predicate, amount, slot, source and transaction are checked on restore. The ready/permanent-combat-marker mode is still closed: the inspected manual explicitly extends core abilities to the core zone, but a direct instruction to reset/add a permanent marker on an off-board core has not yet been resolved against the current zone-state lifecycle. No invented charge or free action was added.

Printed p20 also explicitly names honor as an alternative for command fees and core rest fees. The current paid-command/core-deployment tests cover assets only; alternative honor payment is unfinished engineering; mixed payment has not been specified here, not evidence of an official prohibition. No claim of full payment closure is made.

The damage increment's frozen327-file set passed35 core tests and187 generic-target regressions (filtered, potentially overlapping suites; not summed). Native/WASM comparison passed10 prepared-layout scenarios/83 commands/332 projections/83 actual Native duplicates. UI53/53 passed including the actual damage-mode/target/confirmation buttons. WASM and local Sites builds passed. The initial new test compilation failed on tuple field access (`board` returns `(region,card)`); this harness error and all intermediate logs are retained. The first export ran beyond damage resolution to a later pending choice; the final trace explicitly stops when the original stack empties, with no pending choice.

The positive damage witness currently covers a selected core in play; no positive tentacle-token runtime case is claimed while the token's full original identity remains unresolved. Closed admission is unchanged. Natural lobby, full-library/full-UI suites, independent review and browser play are NOT_RUN.

Local Worker10/10 passed,0 failed/0 skipped, with83 HTTP commands and83 cold D1 reopenings. No public service was started or updated.
