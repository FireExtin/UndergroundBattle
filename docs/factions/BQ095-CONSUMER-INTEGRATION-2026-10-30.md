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
