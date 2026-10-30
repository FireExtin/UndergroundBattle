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
