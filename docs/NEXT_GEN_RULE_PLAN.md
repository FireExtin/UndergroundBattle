# 下一阶段规则核计划

## Summary
- 当前已经有：最小规则核、priority/stack、projection、continuous effects、`inspect` 的 permission hook、`dealDamage` 对 `EffectiveStats.Defense` 的最小致命判定。
- 如果目标是“继续稳定接真实卡 DSL，并形成一个可扩展 alpha”，还差 4 个明确里程碑。做完这 4 个就够继续推进主卡池，不需要先上完整 dependency engine。

## 2026-03-31 执行顺序说明

- 当前仓库状态已经从“纯规则核骨架”推进到了 `LIVE_SANDBOX` 阶段，因此下一步不建议立刻继续堆按钮、堆卡或上真实联机。
- 当前最优先级应该调整为：
  1. 冻结 `M0 sandbox` 基线
  2. 建立 golden scenarios / replay / invariants / projection leak 回归护城河
  3. 再进入第一套完整可玩规则闭环
  4. 最后才扩首发卡池与真实联机
- 换句话说，这份文档里的 4 个里程碑仍然成立，但当前执行顺序应以前两项“稳固基线”和“最小完整闭环”为先。
- 具体落地顺序见 [NEXT_STEP_EXECUTION_PLAN_2026-03-31.md](/Users/ddd/Downloads/UndergroundBattle/docs/NEXT_STEP_EXECUTION_PLAN_2026-03-31.md)。

## 里程碑 1：Permission / Legality V2
- 把 permission hook 从 `inspect_card` 扩到现有需要约束的动作：`reveal_card`、`queue_operation`，以及下一步新增的角色动作入口。
- 在 `CardState` 固化 `RequiredPermissions`、`Prohibitions`、`ActionQuota`；`CostAdjustment` 继续只保留字段，不先做完整费用系统。
- `CheckLegality` 统一走一个 capability 检查入口；拒绝码固定为：
  - `LEGALITY_FAILED_PERMISSION_REQUIRED`
  - `LEGALITY_FAILED_ACTION_PROHIBITED`
  - `LEGALITY_FAILED_ACTION_QUOTA_EXCEEDED`
- 规则固定：`prohibit` 覆盖 `grant`；无 `RequiredPermissions` 的动作不要求显式 grant；quota 必须可 replay。

## 里程碑 2：Character Stats Semantics V1
- 新增两个最小角色动作：`declare_attack`、`declare_investigation`。
- `combat` 用于攻击伤害值，`defense` 用于致命阈值，`investigation` 用于放置影响力；`influence` 先继续只作为有效面板值保留。
- 角色动作统一要求：目标在 `table`、未 `Destroyed`、未 `Exhausted`、当前有行动权、stack 为空。
- 结算后生成可 replay 的结构化事件：`attack_declared`、`damage_applied`、`card_destroyed`、`investigation_applied`。
- `modifyStat` 必须真正影响这些动作读取的数值，而不只是影响投影。

## 里程碑 3：Continuous Effect Lifecycle V1.1
- 增加“来源失效即移除”规则：source card 离场/弃置/被销毁后，其 continuous effects 在下一次 commit 重算中移除。
- 保持当前约束：每次 commit 最多一次完整重算；重算中再次请求重算只记录一次 pending，不允许递归失控。
- 继续不做完整 dependency graph；`DependencyKey` 和 `ResolveConflict` 只保留接口。
- `grantPermission / prohibitPermission / modifyStat / addKeyword` 允许来自真实 fixture；`scriptId != null` 的卡仍只走 script 入口。

## 里程碑 4：DSL Fixture Gate 扩展到真实可玩样例
- 新增第一批命中新语义的 fixture：授予检视、禁止发动、本回合 `+defense`、攻击造成伤害、调查放置影响力。
- TS 侧继续只做 schema/fixture gate；Go 侧对同一 fixture 做 contract test + rules test。
- `queue_operation` 对 pure DSL 卡必须能真正落到：permission change / numeric change / damage / influence / destroy，而不是只写 event。
- 每批新增真实卡后都补一篇 docs，记录规则假设和当前未实现边界。

## Test Plan
- 合法性：required permission 缺失会拒绝；grant 后可通过；prohibit 存在时即使 grant 也拒绝；quota 用尽后拒绝。
- 数值：`modifyStat(defense)` 改变致命阈值；`modifyStat(combat)` 改变攻击伤害；持续效果到期后恢复并影响后续结算。
- 回放：permission、damage、destroy、discard、continuous cleanup 下，action log replay 到同一状态。
- 投影：destroy/discard 后仍遵守现有 hidden-info 隔离，不泄露手牌/牌库信息。
- 双端契约：新增 fixture 在 TS Vitest 和 Go `testing` 下都通过。

## Assumptions / Defaults
- 先以“可扩展 alpha”为终点，不追求完整 dependency engine、replacement effect、trigger stack、完整费用系统。
- Go 继续是唯一语义权威；TS 只负责 schema、fixture、作者工具和前置校验。
- 销毁规则固定为：`damage >= EffectiveStats.Defense` 即离场进 `discard`；本阶段不实现护盾、再生、濒死队列。
- 新动作只先做 `declare_attack`、`declare_investigation`，不同时展开完整战斗阶段改造。
