# JC046 独立静态复审：提前收尾 WIP

日期：2026-10-15。根据用户提前收尾指令保存当前结果，停止扩展与等待运行。**这不是完整独审验收，也不是准入、合并或发布批准。**实现者与本复核者为不同代理；本复核者只读源码、原图、规则页及已有日志，没有运行 Cargo、git、应用集成补丁或修改生产／测试源码。本任务仅新增本报告。

当前源码候选身份为 `rust-v0.2.77-jc046-mandatory-sacrifice-candidate`／`limited-v2.70-jc046-mandatory-sacrifice-candidate`，目录143张。来源规格中的 `gameAcceptance=notAccepted`、`selectionEnabled=false` 是来源记录，不把源码集成或来源完成数当成整牌验收。

## 当前结论与证据限度

已复核强制现身、正常响应、逐席牺牲、来源离场、控制与拥有者、杀伤，以及下述三项预审缺口的修复。冻结源码静读未发现新的确定机制阻断；**修后完整运行未完成，不能据此声称整卡通过。**

| 已读证据 | 当前可以声称的结果 |
| --- | --- |
| `/workspace/jc046-evidence-20261015/native-first.log` | 首轮29项专项：28通过、1失败；失败为光环夹具。原日志保留。 |
| `native-first/`、`invalid-first/` | 实际读取目录计数为36条正例轨迹、55个负例JSON；首轮输出不完整，原样保留。 |
| `native-final/`、`invalid-final/` | 中止前已导出37条正例轨迹、55个负例JSON；本复核者直接核对目录计数。尚无ABI77 WASM对照结果。 |
| `source-tests.log` | 来源测试40项通过、0失败。 |
| `generator-guards.log` | 输入生成器守卫22项通过。 |
| `native-regression.log` | 已核对604条 `... ok`、0条断言失败，其中29个JC046专项全部通过，包括修后的光环测试。全回归由SIGTERM中止，无完整成功汇总；不能记为全回归通过。 |
| ABI77 WASM／Worker／前端 | 本轮尚未完成构建或运行验收；本报告只静读新接线及Face测试。不能沿用ABI76结果。 |

测试日志均由父任务或其他代理产生，本复核者没有执行这些测试。未部署，未做线上自然浏览器试玩。

## 原图、规则与项目应用

已实际查看[JC046原图](<../resource/ymsj-fun.github.io/cards/JC046 无面屠戮者.jpg>)及原规则关键页。印刷字段为：白色名称、非独有、红色角色、恶魔／异界、5费、红忠诚1、鲜血领域1、防御3；永久调查0／战斗2／势力1，临时调查0／战斗0／势力1；完整正文为“杀伤2”“现身强制触发：每位玩家各牺牲一个角色”。编号046/135、酒杯系列符号，不推断发行年。

| 原始来源 | SHA-256 |
| --- | --- |
| JC046原图 | `5d22023e0848d5d2e6566b2fdfe94ef62a80c7b97812f3f81066392aa97cc1ab` |
| 《霸权说明书》PDF | `a1e5bca72b8dbb374357feace28b7fbfd31136349ee23455d60f5f62e0a6b886` |
| 《隐秘世界规则手册》PDF | `2a771874efbdb3ad8a339ea01ff62888766090d075fe2e013632c79de1aeba35` |
| 《隐秘世界勘误及释疑》PDF | `916aa6a9f50ac7e737b517358e0c4270f715d0ed7391d55c8fcb629392a71d38` |

- FAQ PDF3／P3的“无面屠戮者（基础042）”按名称与完整能力对应当前046/135原图，不能直接把旧编号042视为当前archive ID。它明确有角色必须牺牲1、没有其他角色时必须牺牲自己。
- 手册PDF4／P2明确强制触发必须发动、触发进入堆叠并可响应；PDF12／P10明确只能牺牲自己操控的牌、进入拥有者墓地；PDF10／P8支持杀伤参与获胜时增加战斗伤害及叠加。霸权PDF17／P16支持能力不因来源离场取消，PDF22／P21区分本人和友方。
- **项目应用而非官方专属裁决：**声明时按既有 `ForEachLivingPlayer` 冻结在局席，0起递增逐席处理；每席轮到时查询当前角色，真实牺牲、稳定化后再续行；整条续体先于其排队死亡声明继续。不声称官方规定JC046必须座位0先，也不将逐席处理扩成全游戏同时动作裁决。完整设计见[来源设计](JC046_NEXT_BATCH_DESIGN_2026-10-15.md)。

## 三项预审缺口及修复复核

1. **P1：跨持久化容器复制续体。**初版允许 `Accepted`、正cursor的 `Effect::Frame`，但没有与pending／stack／其他effects核对同一 `frame_id`。将真实牺牲pending中的帧复制进effects，可使初版形状守卫接受重复续体，造成后续席重复牺牲。此结论来自静态路径分析，不声称本复核者运行过旧版漏洞。修后 `jc046_stored_frames` 覆盖三种容器，`validate_jc046_frame` 只允许同一存储引用本身、拒绝另一个同ID帧；执行时当前帧已取出，残留副本也被拒绝。正cursor effects还须挂起在完整真实Unique／领袖选择之后，仅允许冲突检查前缀，领袖要求队首重检。新负例覆盖pending+effect、两effects、stack+effect、不同ID伪造第二续体、孤立／无关容器及缺少重检。
2. **缺帧的强制触发壳漏检。**初版只用 `stack.card=Some(JC046)` 识别无帧壳，而真实触发 `card=None`。修后额外识别JC046的canonical强制触发label，并拒绝缺帧。负例 `mandatory-trigger-shell-without-frame` 已加入。证明范围是该实际壳形状；不声称覆盖任意同时伪造label与所有历史字段的通用存档认证。
3. **获授声望／威名漏接地区快照。**初版外置集成补丁未将JC046加入 `source_snapshot` 的 `source_region_instance` 范围，但模块要求获授 `renown`／`jc089-combat-glory` 有冻结地区身份，导致合法获授奖励被拒绝。实际77源码已加入JC046，原有可选奖励不被强制现身拦截；首轮相应专项通过。KEY自身是全玩家效果，不把地区限制加到牺牲范围。

上述修复已静读，不抹去初始缺口。没有按source UID、卡定义或拥有者去重；只禁止同 `frame_id` 的复制。真实JC063潜伏后再次付费现身测试使用两个不同来源UID和不同frame ID，保留两次独立强制触发；首轮该专项通过。

## 首轮失败、夹具修复及合法边界

`jc046_previous_sacrifice_recomputes_next_seat_after_aura_death_cleanup` 首轮在 `jc046_tests.rs:750`、选择前断言doomed仍在场时失败。独立核对原图JC059《安全保卫部门》与 `engine::defense`：其光环只给**本地区其他友方角色**加1防御。初始夹具JC059在region0，JC125在region4且damage=1；JC125基础防御1，因此第一次正常死亡稳定化已将其移除。

冻结修复仅把JC125放到region0，保留其controller=1，并增加初始防御2、选择前防御2／damage1及之后进入owner墓地的断言。生产光环和死亡规则没有因这个失败而修改。随后中止的回归日志已记录该测试及全部29个JC046专项通过，修正得到实际运行验证；但整个回归未完成，不能扩大为全回归通过。

领袖挂起正例也有明确边界：预审曾推测牺牲JZ27、归还领袖控制自然导致领袖选择先于JC046续体；与实现者核对后已撤回该推测。当前JC037／JC052／JC095均独有，控制恢复会先append Unique，随后牺牲选择把JC046续体push_front；已有Unique使领袖检查不再前插。因此本轮未证明自然Room命令能到达那个暂停布局。

`prepared_leader_suspension` 明确离线调用生产牺牲／控制helper，再主动调用 `offer_leader_choice` 构造暂停。JZ27借控对象已改为印刷人类JC052，归还后与JC095形成领袖冲突；正例证明此准备容器可严格恢复、逐席续行一次，不证明自然命令到达性。普通控制归还另有离线初始控制布局后的连续真实Room命令测试。该边界不隐藏，也没有为造正例改生产队列。

## 机制与接线静读范围

`jc046_declare` 在普通可选声明前核验完整能力，直接进入Respondable堆叠，没有可拒绝的Accept，也没有改Immediate。付费与免费Reveal共用事件；正面打出、秘密派遣不触发。`SacrificeChosen(Context)` 查询所有地区当前controller的明置角色，空候选跳过，否则强制1；来源自身、横置角色可选，暗藏者／资产／附属排除。牺牲复用 `remove_dead(...Sacrifice)`，不经过目标／伤害过滤，墓地归owner，来源离场不取消已冻结续体。

杀伤2与永久战斗2分开，复用既有实际参战及获胜计算。逐席死亡通过既有leave/control/death路径；队友不自动成为“本方”死亡观察对象。死亡专项检查整帧完成前不开放死亡声明及自己的观察范围；没有单独逐项断言所有多席死亡回调排列，不能称为全局FIFO穷尽证明。

Game／Room加载、resolve_stack／resolve_frame、choose及角色动作入口均接入新守卫。定义锁定杀伤及完整canonical程序；持久化校验actor／source／程序／cursor／候选和不可拒绝元数据。已读负例针对具体变异，不能泛化成任意恶意存档都不可构造或历史迁移已支持。

- `tools/fixture-tools/jc046-current.py` 静读：精确ABI77／143张目录、37个场景名、状态连续性与版本核验；只增加provenance，不改opaque state，拒绝覆盖不同已有输入。首轮缺1条正例，最终目录已导出37条；不据此声称汇总或WASM对照已运行。
- `rust-game-wasm/tests/jc046_compare.mjs` 静读：完整返回值和各席view比较、WASM摘要绑定、旧76拒绝；若传负例目录，对每个原样state检查view与apply均拒绝。目录只断言非空，55数量必须以实际最终输出确认；目前未运行。
- `sites/test/jc046-integration.test.mjs` 静读：37个准备布局后连续真实RoomService命令、D1 opaque state／journal／各席view、关键阶段重开、拒绝不写receipt、重复原命令返回原receipt。它自身对catalog，不单独对WASM字节摘要；最终pkg／generated／Worker同字节证据本轮未完成。HTTP覆盖主要为重开后的重复receipt，不声称37条全程经浏览器HTTP自然操作。
- `tools/cards/jc046-source.test.mjs` 锁旧142张完整定义摘要、旧metadata、原图规格与143张原图映射；来源40项已通过。`web/src/game/JC046Face.test.jsx` 核完整印刷文字、图标与原图入口，当前仅静读，未记前端通过。不涉及并行组卡UI或单桌保存任务。

## 已读取的冻结源码指纹

以下为本次直接读取文件字节的SHA-256；不是构建产物摘要，也不是提交身份。若这些文件再变更，需要另行复核。

| 文件 | SHA-256 |
| --- | --- |
| `rust-game/src/jc046.rs` | `b767146f5d3942305caa1a34caf40461729bbdd05e84208f3e7af97a578021b2` |
| `rust-game/src/jc046_tests.rs` | `4a08fb9e94b501f929b4d672c311e7bf8bb924e5274dcf78f909aa2ca5729c99` |
| `rust-game/src/lib.rs` | `52e08066aca6a65d57eaae62d2defecc51c374a09ce48c8d2053053c7f520957` |
| `rust-game/src/rules.rs` | `ba5f56925e439af69ab9c262d36f21f4a64e65d04d1b06efa8484e44376b6a5b` |
| `rust-game/src/engine.rs` | `11b628c731524a2382068bc21c4cbc57ab0c5b48f84cf9f1b583946bac6e7e75` |
| `rust-game/src/model.rs` | `bc9b1c8e1dd110ed211d05e32b2120d2ef55b8d77c449671cae9de629f5c3ea4` |
| `rust-game/src/room.rs` | `2423e7e3483e2d00cded662050110bef7e2a433972495407bb31eb4435879079` |
| `rust-game/src/resolution.rs` | `f8c88bf7e73cca311fca83284c7a10473b7666feb84a3d1d23862f727d0bcc2a` |
| `rust-game/data/cards.json` | `694751f8429c433115047dddd5ac01c3b04ef5899ab6317271c73db8bf044ea6` |
| `tools/fixture-tools/jc046-current.py` | `4c068e48c2443536295dc3fa7fb16a19eab08bf51bbd66a17ffa3fc339917cea` |
| `rust-game-wasm/tests/jc046_compare.mjs` | `6b62237e83f2bc7581901eb9cd6f02502c1089bff73cbbb15d6952285378d3c3` |
| `sites/test/jc046-integration.test.mjs` | `7f89cc9dd097eba833bed6a2614f77ef138cbf8639cf9d845127f813db0e3a37` |
| `tools/cards/jc046-source.test.mjs` | `3dc5a41a06897de0c7fcf95bbdcfbe6fff892224a19e74e7b1e6b74358bb205a` |
| `web/src/game/JC046Face.test.jsx` | `3b98b4e99d18a365c053c4357fea525573698d4f8b56c793011f029742b42799` |

提前收尾后保持WIP。剩余是修后完整Native、ABI77实际构建与对应WASM／Worker／前端证据及后续独立验收；本报告不替这些未完成事项作结论，也不新增执行工作。
