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

## WIP2: real attachment and triggers

The following supersedes this document's earlier no-catalog/no-dispatch statement. BQ095 now has a closed319th catalog record (`supported:false`, public count236 unchanged), exact original art/index mapping, a real paid society-attachment action, optional entry trigger creating/reusing headquarters, and preparation draw2→mandatory discard1 using the existing preparation queue. The card remains incomplete because global own-grave reference consumers and headquarters extraction are not all wired. This is not a whole-card completion or natural construction chain claim.

Run07 failed compilation due to a missing enum import (0 executed); run08 passed15. Run09 executed17:16pass/1failed because the test called the board-character-only `return_hand` helper on an attachment, so no departure occurred. Run10 uses the actual attachment destruction path and passed17/17,0fail/ignored. It also verifies corrupt actual attach frames, missing/foreign frame sources, wrong society target/host, and accepted entry resolving after its source leaves. The paid-play fixture explicitly prepares resources and society; it is not natural deck setup or browser play.

WIP1 WASM06 build passed but is superseded for the new entry code; WIP2 WASM/UI/Worker validation remains NOT_RUN at this checkpoint. No earlier14+39 count is advertised as the final WIP2 result. Complete-card increment remains0. The next required work is the owner-bound printed-reference/extraction layer and its consumers, followed by complete natural/cross-end evidence.

## WIP3: owner-bound extraction, public UI and prepared cross-end evidence

Source-aware headquarters extraction now records actual JZ53 departure events, including the headquarters identity, and BQ079 preparation reads/pays from its owner's rewritten reference. Controller changes do not change the owning headquarters; the controller still makes the choice. World region cards are excluded from owned-card text rewriting. Public headquarters projection preserves the printed card kind, and Table renders a separate public headquarters pile without creating unsupported actions.

Final run16: BQ095 Native23/23 and related Native62/62, no failures/ignored (85 distinct selected tests, not the full library). Default WASM build and web TypeScript/production build passed. Same-source Native→WASM comparison passed two explicitly prepared scenarios,16 commands/64 seat views/16 duplicate transitions. Local Worker passed2 scenarios with16 commands,16 cold reopens,64 views, per-command duplicate receipt and original receipt after completion. UI9 files/196 tests passed, including6 new headquarters tests. These are prepared component/protocol scenarios, not natural deck construction or browser play.

Retained failures: run12 exposed a controller-based filter incorrectly hiding owner-headquarters arrival (0pass/1fail); run13 had a SocietyDefinition field compile error (0executed); run14 exposed world-region cards being mistaken for owned player cards (21pass/1fail). Runs15/16 fixed these causes; none of the failed outputs were overwritten. Current source manifest SHA256 `9fef961e7a08768f46e15f48a9812c6d9a42cd4914b54822a9ac9348611560b7`; raw Native SHA256 `f897ffa5c438a9d30d578a9ef101689db8996a5a8e42e20bb165c2735d04f354`; WASM SHA256 `b6a35c9f506c5300c5d12130694fb2a3b0aaeb2736e9450a4b9be69bddd590df`.

Still incomplete engineering: source-bound own-grave target binding/extraction in resolution/world and remaining card consumers (including BQ061/MSJC16/JZ53 entry), self-grave play sources (CQ27/XQ35/JZ51), pending targets frozen before headquarters creation, and a complete natural construction chain. Generic/enemy/all-grave references must remain physical; a global graveyard substitution would be incorrect. Complete-card increment0, independent review0, supported:false. No neutral dependency blocks these remaining steps. Local WIP only; no upload/review/push/deploy retry.
