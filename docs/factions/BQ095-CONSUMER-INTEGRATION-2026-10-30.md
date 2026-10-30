# BQ095 consumer integration inventory (unfinished)

Base205bef6d. This is implementation tracking, not completion or admission.

- Already wired: grave destination commit, BQ079 arrival/upkeep, JZ53 departure, persistent zone/public UI.
- Direct choices: BQ061 single recovery; MSBQ03 corpse sealing; MSJC16 seven-card payment. Each needs options, extraction, pending validation and recovery, not just a getter.
- Bound own-grave targets: CQ09, BQ084, BQ086, JZ53 entry, BQ082/pack_a, batch3 XG28/BQ080/BQ081, MSJC16. Need target discovery, public binding, owner semantics, immutable old instance cancellation and actual effect extraction.
- Other source-specific consumers: import20_gate/broker, black_next20/next20_black_wraiths, JC097, BQ091/CQ26 counts. Inspect exact own/enemy/all reference before routing.
- Self-grave plays: play_sources, engine legal actions, CQ27/XQ35/JZ51 and frozen play-source validators. Granted permissions (JZ52) must bind the grant owner's reference, not assume the played card defines the source zone.
- Generic physical grave access and world region effects remain physical. Identity/provenance traversal in mobility/death/attachment modules must distinguish an actual HQ card from a physical grave card without broadening effect targets.
- Complete natural deck chain follows consumer integration; prepared layouts cannot substitute for it.

Current working slice starts with BQ061/MSBQ03 choices. Remaining rows are explicitly pending, no whole-card claim.

## WIP4 verified boundary

Run17 passed25; run19 passed28; final run21 passed30 BQ095 tests,0failed/ignored. Run20 failed compilation before tests (reference type in CQ27 lookup), corrected without changing the rule. A direct cargo-check invocation without the established toolchain wrapper failed ENOENT; wrapped check18 passed. Original logs remain in /workspace/bq084-validation.

Wired in this slice: BQ061/MSBQ03 options, actual extraction and pending restore; explicit own-grave target discovery/public metadata/guard and generic Move extraction; predicates and effect entry for CQ09/BQ084/BQ086/JZ53/BQ082/batch3 grave family; CQ27/XQ35 intrinsic actions, actual payments, source cancellation and restoration. Migration refreshes entity IDs; a target bound before migration cancels instead of selecting the new HQ instance. Frozen public owner validation uses zone creation versus frame allocation ordering, retaining pre-migration response restoration.

New tests cover two direct choices, own/borrowed/generic target separation, JZ53 actual deploy-trigger-return, bound target migration cancellation, CQ27 actual three-character payment, XQ35 actual seal payment and departed-source cancellation. These are prepared engine scenarios. Wider related regression and new same-source WASM/UI/Worker remain pending; the prior WIP3 cross-end evidence must not be attributed to WIP4. Whole-card increment remains0. JZ51/normal grave deployment, MSJC16 and other listed consumers remain to integrate.

## WIP5 verified boundary

Added JZ51 intrinsic headquarters action (unpaid three-corpse choice, actual sealed payment, response, chosen-region entry), MSJC16 seven-card payment/revival, JC085 intrinsic normal-cost headquarters play, and same-owner JZ52 grant lookup/extraction. The physical grave remains separate in public views. Run22 had32pass/1failure solely because the new JZ51 test expected `jz51-region` instead of the existing `jz51-entry-region`; its log remains. Run23 corrected that exact test expectation:33/33 BQ095 tests,0failed/ignored. Related existing13 test groups actually discovered168 and passed168,0failed/ignored (not full library). Native33+168 are distinct selected tests; historical repetitions are not added.

Same-source default WASM build passed; seven explicitly prepared Native Room scenarios compare46 commands/184 seat views/46 duplicate transitions. Worker7/7 passed those46 commands with46 cold reopens, four-seat views, duplicate DB invariance and original receipts after completion. UI9 files/196 tests passed. These are component/protocol tests, not natural deck setup or browser play. Native raw SHA256 `3b5f854a71c9aae715df10808a373428db62a5a2972e287037c9149c038b69f6`; source manifest `720fc51b5c061ffb37003edc00b8df29655046a4dfa12aeb47d0e48728d1a221`; WASM `bb19383d45df3cb588a857f3d862d8797476c3a9b7d20e32a67a657094847c74`.

Resource observation: concurrent related Native execution and WASM build reached cgroup17179869184bytes, both exited0. Subsequent heavy builds/regressions should run sequentially; jobs1/test-threads1 retained. This is cloud usage, not the user's local model host.

Remaining concrete engineering (no new permission/rule blocker): BQ078/BQ081/CQ25/CQ28/BQ085/XQ39 direct consumers; CQ26/BQ077/XG27 observations and thresholds; JC097 observation; whole-physical-state provenance scans; cross-owner JZ52-granted plays; complete natural construction chain. Original JZ53 scan was rechecked: “将你墓地中的目标[死亡]角色牌置于你的手中。” Existing OwnerHand operation must be corrected to ActorHand for the now reachable owner/controller-divergent HQ case, with a real command regression. This known pending issue means no whole-card completion is claimed. Complete-card increment0, review0, supported:false. No refused operation retried.

## WIP6 source integration, focused verification only

Run26 passed39 and run29 passed42 BQ095 tests,0fail/ignored. Checks24/25/27/28 passed. WIP5 WASM/UI/Worker evidence does not cover this new source yet.

Now wired: BQ078/BQ081/CQ25/CQ28 direct choices/extraction; BQ085/XQ39 recovery and upkeep; CQ26 thresholds and actual destination-sensitive mill observation; BQ077/XG27 self-zone triggers; JC097 frozen observation; BQ091 threshold; BQ087 preparation; WM033/JZ32 recovery; JZ56 thresholds computed per actual granting attachment rather than recipient owner; statue-copied BQ078 pending options. JZ53 uses ActorHand per original scan, with an actual owner0/controller2 trigger test. JZ52 has an explicit persisted headquarters-grant receipt, exact source-zone/allocation validation, cross-owner legal-action generation, normal payment and source-departure recovery tests.

Physical identity/retired-instance scans were individually classified separately from effect targets: bq030, death_observers, jc035, jc041, jc044, jc046, jc052, jc054, jc057, jc066, jc095, mobility, red_time, sealing, xq23, batch3_grave, batch3_politician, import20_swords, next20_blue_gray, batch3_host. Existing batch3_physical_cards already included HQ and retains exactly one enumeration. Actual transfer receipts in rumor and JC028 follow destination replacement; generic grave effects do not inherit that access.

Explicit exclusions from printed-own-grave rewrite: public physical grave projection; generic/any/enemy grave slots; world-region revival; BQ088 friendly-grave search; CQ15 target player's broad physical-zone inspection; JZ35's explicit generic “这些角色牌从墓地移回场上”; generic 饮血 keyword access. BQ095 says owned cards' printed own-grave references, not every engine operation using a grave. JC028 instead says return the sacrificed named card, so its exact death receipt follows the actual replaced destination. These scope choices are documented and require no change to paused card admission; independent review is still absent.

Remaining before whole-card completion: complete legal-deck natural command chain; cumulative library/affected integration regression; same-source WASM/UI/Worker evidence after this wide source slice; review any failures without weakening constraints. No whole-card increment yet.

## WIP7 natural construction and foreign-hand follow-through

Natural run30 failed before headquarters migration: the command driver declined BQ093's optional mill and did not reserve enough black loyalty assets. No production rule was relaxed. Run31 passed a real validated50-card opening with candidate-only registry admission (BQ095 and MSBQ03), unmodified opening RNG/state/resources,1678 accepted Room commands, four-seat views and duplicate receipts. Its full raw/log are retained. The later frozen driver uses additional already-implemented black asset cards in the legal deck to reduce setup commands; this does not implement or admit any neutral card.

Extending borrowed JZ53 recovery through the receiving player's subsequent normal deployment exposed a real controller bug. Runs32/33 first exposed test preparation omissions (unresolved departure choice; passing an empty stack), retained as failed runs. Run34 then reached deployment and failed the owner0/controller2 board assertion: generic ActorHand reset retained the old owner's controller. The minimal fix sets controller to the receiving actor in ActorHand while preserving owner. It does not redirect OwnerHand, grave ownership or generic targets. The regression follows actual recovery, optional departure choice, paid deployment and Game/Room restoration.

The natural chain covers actual industry mill, headquarters paid attachment/entry migration, preparation draw2/discard1, and JC085 redeployment from headquarters, with public admission still false. UI tests additionally consume the natural preparation/discard decisions after refresh. Cumulative and same-source cross-end validation are pending at this editing boundary; whole-card completion is not yet claimed. Prior WIP evidence remains separate and cannot be attributed to this source.

Run35 frozen focused result:43passed,0failed/ignored,3033filtered; natural seed507,938 real commands, separate from seven prepared scenarios46commands. The controller regression passes. Exact same-source raw exports are `native-35.json` and `natural-35.json` under `/workspace/bq095-live-validation`; full-library and WASM/UI/Worker are next, not yet passed at this checkpoint.

## Cumulative run36 failures and minimal follow-up

The final Native library result at823191d6 is3070passed/5failed/1ignored (3076discovered),2563.80seconds. Early progress parsing counted only same-line `... FAILED` and missed multiline failures; only the final summary is authoritative. Original log and metrics remain unchanged. Both explicitly requested witness bodies produced nonempty output (sentinel1scenario,WM0442scenarios); the existing BQ001 candidate-only natural test remains ignored.

Failures classified from source and existing checks:
- CQ26: real BQ095 integration regression. Original text distinguishes “你的墓地” for the seven-card threshold from “从本方牌库进入墓地” for the trigger. The trigger observes any actual physical grave destination of that player's deck cards, including foreign-owned cards; headquarters is not a physical grave. Restore validates the same destination at observation time. Threshold remains source-specific. The new HQ test now also checks a foreign-owned deck card reaching its owner's physical grave.
- BQ035/TK008: existing dedicated CQ11/TK008 validation rejected independently validated cross-card grants. Reuse the existing strict cross-card declaration/frame validators, and apply CQ11's own accept-choice shape only to its own entry ability. No arbitrary ability admission or skip is introduced.
- Control catalog expectation omitted the already-implemented closed WM026 death ability; society expectation omitted closed MSBQ01. Append the exact current identities, retaining existing semantics and negative assertions.
- Rules registry contains the explicit internal `JC028:blood-form`, intentionally absent from the printed catalog. Compare the complete key set against catalog IDs plus that exact internal identity, retaining malformed multi-target rejection; do not merely increase a count.

Run37 failed compilation before executing tests because the new test used an unqualified BTreeSet name. The correction qualifies std::collections::BTreeSet; run38 is the selected verification. Cross-end evidence must be freshly exported and built after these source changes. Whole-card increment remains0 until this follow-up closes.

Run38:17passed/1failed; CQ26 and all three inventory assertions passed, but static ability dispatch still intercepted TK008's canonical mobility. Dispatch now delegates only recognized shared mobility/raid/sword programs to their existing validators; per-card definition registration stays exact. Added CQ11/TK008 to the existing rejected-static-registration/manual-activation negative test.

Run39:43passed/2failed. With dispatch fixed, the old BQ035 field helper exposed missing generated provenance for token recipients. It now calls the existing typed make_generated constructor for the four token definitions instead of fabricating plain token cards; no production identity check was relaxed.

Run40:44passed/1failed. TK008's native region-destruction pinning was being applied to a canonical granted-mobility target, which must retain the shared mobility destination contract. requires_region_instance now excludes the fully canonical mobility target independent of recipient, while all native region slots retain their prior pinning. Existing WM023 native-action/transplant negatives and CQ11 destruction/changed-region negatives remain. Run41 expands the selected regression to mobility-related tests. At this checkpoint verification is in progress; original failures remain preserved, complete-card increment0 and admission closed.


## Final closed implementation and same-source validation

Tested source: `9e48517844968cbc69f4d38d2fb740b92b77a0d3`. All concrete consumer rows above are integrated; historical WIP statements describe their checkpoints, not remaining work. The final commit only appends validation documentation. BQ095 closed implementation increment is1; independent review0; supported:false/public admission unchanged. This is not a released20-card batch or browser play.

Run41 passed61 affected tests and run42 passed43 BQ095 tests. These overlap the final run43 library:3076discovered,3075passed,0failed,1originalignored,2598.00seconds. Both sentinel and WM044 witness bodies actually executed and emitted1/2scenarios, already included in the pass count. The ignored BQ001 candidate natural test remains closed by design. Separate integration run44 passed72 tests in8targets. The original run36 five failures and runs37–40 follow-up failures remain preserved; final results do not overwrite them.

Default WASM build45 passed. Parity46 passed8scenarios/984commands/3936seatviews/984duplicates. UI47 passed199tests across9 affected files; Worker build48 and web build50 passed. Original Worker49 ran once to exit0 in1570.20seconds:8passed,0failed/skipped/cancelled,984commands/984cold reopens/3936seatviews, duplicate DB invariance and original receipt after completion in every scenario. Seven explicit prepared layouts account for46commands; the validated50-card natural opening accounts for938commands (760before headquarters play,178headquarters play and following commands). Natural construction uses the internal validated candidate registry, not UI deck-save or real browser input. No claim covers every unrelated UI suite or independent review.

Fresh raw Native hashes: prepared `2bce8f77cb9426ec1ba39f8969603078fb89c55b31ec90aebcf58210f072f766`; natural `11c6b684d716c7d2a34ba2ec55f89b7dc2818c6e3089d25fcbe317f91ea239ff`. WASM SHA256 `03907a74d0816c94a3b86f465eb584e43819b59a71282296c6443d2eda79b56b`; source manifest `c595248fddf71c54ab33ed3eec90695546980110c07eb5544708745dafb5849d` (441source inputs rechecked unchanged). Cargo.lock unchanged. Rust1.90.0/Node24.19.0/wasm-bindgen0.2.104; jobs1/testthreads1. Final library cgroup peak17112510464bytes; Worker peak15635496960bytes. Cloud only.

Evidence remains locally under `/workspace/bq095-live-validation/wip8-delivery` with original logs under `/workspace/bq084-validation`; final incremental recovery bundle starts at WM026 base `ab2287a448364927b26433dff409a3ead0ee18a7`. Library HTTP401, prior independent-review refusal and push restriction were not retried or bypassed. No push, merge, deployment, public admission or browser play occurred.
