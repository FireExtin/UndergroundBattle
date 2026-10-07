# JC089／JC018 威名奖励边界修复（待独立闭环审查）

独立审查发现：JC018 接受 JC089 授予的战斗威名后，玩家真实付费 JC005，在地区全部对抗结束前移除最后诅咒，旧 JC018 `traits.renown` 会恢复并再奖励一次伪声望。原候选 `19e594f00004c16c95cc1846c4ad3d252c5b0021` 的限定抑制未消除这个合法交互。本修复永久纠正 JC018 的印刷威名绑定：赢得战斗后可放一个本方势力标志，地区结束不会因同一印刷能力再奖励。真正的独立声望仍按原规则生效。

修复从上述固定候选创建隔离 worktree `/tmp/jc089-glory-fix-game`，分支 `codex/jc089-glory-fix-19e594f-cloud`。原 worktree、原审查反例、原 oracle、原审计二进制和原图组合提交保持冻结。上一份 [JC089 初次候选说明](JC089_POISON_BLOOD_CLOUD_CANDIDATE_2026-10-07.md) 及其证据属于修复前历史；其中保留无诅咒 JC018 旧行为的文字不代表本修复的最终行为。没有 owner 要求保留错误绑定；那是父先前缩小范围的决定，父现已明确授权必要回修。

## 正式规则与范围

实际查看 JC018 原图，SHA256 `a764f6ad543c1aa25e287e42a508ecce2368277e64377c2969b2aff079b63af8`：印刷关键词是威名。JC089 原图 SHA256 `83853a1a4c3e6ecd9644f7bb5c843e70585d421f1792a0e4a141ebbe867d12f7`，授予附主威名，同时防御−1、永久战斗1和临时战斗1。正式《霸权说明书》印刷18／PDF实体19明确区分：威名在赢得战斗后可触发，同地区多个威名角色只放一个本方标志；声望在地区全部对抗结束后比较数量。FAQ整页2支持无击杀仍可赢得战斗；FAQ整页4明确防御1受诅咒减至0会死亡。原始材料与实际查看页码仍见 [原图及规则记录](evidence/jc089-2026-10-07/original-and-rule-review.json)。

当前候选仍为 `rust-v0.2.45-jc089-poison-blood-candidate / limited-v2.42-jc089-poison-blood-candidate / hegemony-pdf-v1`，尚未发布。本次仅纠正已准入 JC018 的有限绑定和 JC089 的直接奖励交互。103张准入牌中，新增卡仍只有 JC089；JZ48、JZ49保留。对44旧目录的比较明确允许 JC018 `ruleTraits.renown` 移除，其余旧102张牌的印刷字段、目录与构筑数据不变。没有新增历史45、历史奖励标志、通用关键词框架、队列或身份机制。

## 实现

`jc018_definition` 不再设置错误的 `traits.renown`。新增有限资格查询只认完整定义守卫下的真实 JC018，以及实际有效的 JC089 附主授予；它不按任意印刷关键词授予通用能力。两种来源共同复用候选已有固定形状 `CombatWon`、`Declare`、`PlaceInfluence` 和现有队列，仍每团队／地区合并一份可选声明。运行时内部能力 key 沿用 `jc089-combat-glory`，不新增能力形状。

伤害前冻结获胜参与角色、当时操控者与真实地区实例；来源离场或控制权改变不转移已声明奖励，同下标地区替换后不把奖励施加给新实例。印刷 JC018 与授予来源共同使用这些边界。隐藏、横置、战斗无贡献、失败、平手及其他对抗不触发。2v2 的最小合资格操控者排序沿用父认可的项目政策，不宣称正式手册规定这种席号选举。

`has_renown` 只查询真正的印刷声望或独立 `grants_renown`。JC074 真实付费授予声望后，威名与声望仍分别在战斗胜利、全部对抗结束两个时点生效；卸除最后 JC089 不改变 JC018 的正确印刷威名。当前卡片投影显示无诅咒 JC018 的 `currentCombatGlory`，不显示伪 `currentRenown`。旧 green 测试误断言 JC018 声望已按正式规则改正。

完整 Definition 守卫继续拒绝给其他牌移植 JC018 属性、JC089 属性或动态 CombatWon 能力；`validate_ability` 独立验证固定运行时形状，允许冻结附主已经离场的状态存读。

## 原31命令反例：同输入 red→green

独立审查冻结的 `/tmp/review-jc089-mechanism-evidence/frozen31/command-stream.json`，SHA256 `df861682a177b6d7db036127c329c7eca58a122ade492c0fcb41156abaf2fbb8`，31条命令的 seat、payload、serverNow 原样回放。原 driver SHA256 `4d19e7b14d27d35186b98b45ed5952804cfdc099596c450d0c71772c47d5322f` 和初始 fixture 保持不改。

旧19候选接受31条，最终地区势力 `[2,0]`，并恢复伪声望。修复包接受26条，剩5条原子拒绝，最终 `[1,0]`，印刷威名仍有效、伪声望不存在。原第27条等待并接受不存在的伪声望选择，正确返回 `invalid_action`；后4条仍携带原版本，返回 `version_conflict`。不伪造选择来让31条全部接受。新回放仅把结果断言改为正确语义，证据见 [原反例结果](evidence/jc089-glory-fix-2026-10-07/exact-31-red-green.json)。

另有正常合法流程测试，无错误选择命令：真实付费 JC059 防御支持、JC089 结附、JC005 响应，在威名奖励结算前／结算后且地区结束前移除最后诅咒，都只获得一次威名。

## 最终验证

最终字节、driver/test SHA、测试总数与审计来源见 [验证清单](evidence/jc089-glory-fix-2026-10-07/validation-summary.json) 和 [当前构建清单](evidence/jc089-glory-fix-2026-10-07/current-artifacts.json)。完整逐命令证据和日志在 `/tmp/jc089-glory-fix-evidence`。

- 26项专项保留原JC089全部边界，并覆盖永久纠正 JC018、真实 JC005 最后诅咒移除的两个时点、独立真声望、印刷来源隐藏／横置／参与筛选。`jc089_glory_snapshot_survives_host_and_curse_departure_and_controller_change` 与 `jc089_glory_exact_region_instance_rejects_replacement_before_or_after_acceptance` 均对印刷和授予两种来源执行。
- `jc018_printed_glory_decline_and_next_round_win_use_normal_window_progression` 只在初始战斗使用离线布局，随后真实 pass 经过势力对抗、回合结束、清理、先手轮替、下一轮行动和战斗。第一次拒绝后只有普通势力1；真实付费添加／移除诅咒不补发；下一轮战斗接受威名后总数2，再正常推进至第三轮仍2。没有同轮手工重开战斗，也没有 once-per-game 标志。
- 全部默认原生539项库测试与32项集成测试，共571项通过；没有开启 `society-fixtures`。
- WASM比较938条独立命令（含5条原子拒绝）、2,033个检查点、13条连续真实付费响应链、11,936份四席投影，与原生完全一致。u64存档保持opaque字符串。
- Web567项／67文件通过，JC089专项10项，27个明确离线布局。当前展示包括印刷 JC018 真威名、移除诅咒后的真威名及独立声望。组件 fixture 不宣称自然浏览器对局。
- 两项TypeScript检查、Vite产品构建、Worker本地dry-run通过；Workerd／D1全部25项通过。所有40份历史元组和5种身份篡改在当前 reducer 前拒绝。
- 相对固定19的全部209个历史文件、40个目录，逐 Git blob 核对字节一致。生产 generated 只含当前JS／WASM两个文件，Worker只有一个当前WASM。旧历史引擎不受此规则修复影响。

原生使用已有 `/tmp/jz49-native-target`，Rust1.90，locked/offline、jobs1、无增量、移除debug。全套原生完成后串行构建并冻结当前 `hegemony-audit`，供父与独立审查进行只读审计。没有接触真实数据库。当前本地候选等待独立闭环审查及父的新组合自然UI；本任务没有 Site调用、发布、main合并或GitHub推送。

## JZ50 下一批

Owner 已在2026-10-07 12:23 UTC明确裁定 B：接受现身搜索后，即使存在合资格角色也可选择0或1张，0张仍洗牌；拒绝触发则不搜索、不洗牌。这是项目owner补充裁定，不冒充正式扫描FAQ原文。下一批计划独立保存在 `/tmp/jz50-next-plan-evidence/owner-B-plan.json`，等待本次闭环后另建隔离工作树；本修复没有实现或准入 JZ50。
