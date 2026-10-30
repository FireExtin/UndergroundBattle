# BQ095 persistent zone WIP — not a completed card

Base `ab2287a448364927b26433dff409a3ead0ee18a7`. Original scan was directly inspected again: `resource/ymsj-fun.github.io/cards/BQ095 圣甲虫药业集团总部.jpg`, SHA256 `42f82a8ceb0d3513c0861995026f2891ae64d20699c3c49d44aa5a4120a749cc`. Black attachment, facility/research, unique, cost6/black loyalty3, attached to own society. It creates a named **zone**, not a contestable world region. No paused neutral-card input is required for this card's entry rule.

Printed clauses: create headquarters for own society; move all own grave cards there; for the rest of the game replace owned cards' printed own-grave references and destinations into own grave; prepare draw2 then discard1. These clauses are not yet all wired. Project choice under the continuing ordinary-game-rule delegation: re-entry reuses the society's one lasting headquarters and its existing cards, rather than creating duplicates or silently destroying/merging old zones. This is not an official ruling. First source identity remains the creation witness; later source departure does not expire the zone.

## This slice

- Headquarters now has a distinct persisted Game container and public four-seat projection, separate from the physical graveyard. Physical entity traversal includes its cards.
- Game/Room restore and action boundaries check unique zone/seat/physical entity identities, allocation order, ownership, cleared cross-zone markers and absence of generated cards. Restart clears the lasting zone; elimination clears its card contents along with the ordinary grave.
- Internal resolved-entry primitive creates or reuses the zone atomically. No public action or BQ095 dispatch invokes it yet. Tests explicitly set up this internal primitive; they do not claim a natural BQ095 play.
- Existing unified grave commit routes into an existing headquarters. Actual generated objects still disappear before that routing. Owned BQ079 printed-own-grave observation sees headquarters arrival; a borrowed BQ079 with another owner does not falsely observe the same arrival. No global `.graveyard` replacement was performed.
- A prepared actual Room discard checks duplicate transition and cold restore. Direct discard and generated departure tests exercise the real engine paths, but zone creation remains prepared setup.

## Failure history and completion boundary

Runs01 and04 failed test compilation (Room constructor/borrow and missing explicit Discard fields); zero executed tests. Run02 passed9. Run03 passed12. Run05 executed14:13pass/1fail, exposing that migration could refresh a physically duplicated card into a new ID and hide the original malformed state. The fix checks whole-physical-state uniqueness before migration; run06 is the fixed-source verification. Earlier logs are retained, never counted toward the final total.

Still engineering work, not a rule/permission blocker: actual attachment/entry/prepare dispatch; source-aware own-grave target binding and extraction versus generic/enemy/all-grave references; leave observers and self-grave play sources; every applicable direct consumer; client zone rendering and real natural construction/play chain; then same-source WASM/UI/Worker verification. Consumers include resolution/world/play_sources plus CQ27/XQ35/MSJC16/BQ079 and other direct source references. The persisted container cannot be treated as completion of these consumers.

Complete cards added in this slice0. BQ095 is still absent from the playable catalog; no admission is opened. No independent-review retry, Library401 retry, push, main merge or deploy. Native components are not browser play. Local evidence and recovery bundle only.

Fixed-source run06 passed14/14,0fail/ignored. Same-source related run06 passed39/39 across batch3_ghoul_tests, jz53_tests, generated_tests and wm026_tests;53 distinct selected Native tests total, not full library. Default WASM build06 is running at this checkpoint; compilation alone will not establish a BQ095 cross-end natural chain.
