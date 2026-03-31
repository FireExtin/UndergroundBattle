# 下一阶段规则核计划

## Summary
- 当前已经有：最小规则核、priority/stack、projection、continuous effects、`inspect` 的 permission hook、`dealDamage` 对 `EffectiveStats.Defense` 的最小致命判定，以及第一批角色动作入口 `declare_attack / declare_investigation`。
- 如果目标是“继续稳定接真实卡 DSL，并形成一个可扩展 alpha”，还差 4 个明确里程碑。做完这 4 个就够继续推进主卡池，不需要先上完整 dependency engine。

## 2026-04-01 进度补记

- `Phase 3` 已经开始，但只推进了第一刀：
  - 新增 `declare_attack`
  - 新增 `declare_investigation`
  - 角色动作会读取 `EffectiveStats`
  - 角色动作会使执行者 `Exhausted`
  - 攻击会走最小伤害/销毁语义
  - 调查会向地区放置 `influence` counter
- 这还不等于“完整可玩对局闭环”。
- 下一步仍然是：
  - 地区争夺
  - 得分
  - 胜利条件
  - 持续效果来源离场清理

## 2026-04-01 第二次补记

- `Phase 3` 现在已经补上第二刀：
  - 地区牌会记录 `InfluenceByPlayer`
  - 地区控制权以最高 influence 决定；平局则无控制者
  - `declare_investigation` 会把 influence 记到行动者名下，而不只是加总 counter
  - `placeInfluence` DSL 作用到地区时，也会同步 region control
  - `end -> main` 的回合切换会按当前已控制地区结算得分
  - 分数达到 `VictoryThreshold` 时会写入 `WinnerPlayerID`
- 这仍然不是完整闭环，当前明确未完成的是：
  - 分数 / 胜利状态尚未投影到 Web 调试器
  - 回合切换尚未轮转 `ActivePlayerID`
  - 地区争夺还没有更完整的占领/结算规则
  - 持续效果来源离场清理仍未接入

## 2026-04-01 第三次补记

- `Phase 3` 的这一步又补了两个调试和回合层缺口：
  - `end -> main` 现在会轮转 `ActivePlayerID`
  - projection 现在会携带 `Score`
  - Web 调试器现在会显示：
    - `Active Player`
    - `Score`
    - `Winner`
- 因此当前 sandbox 已经能观察到：
  - 地区控制
  - 回合结束得分
  - 胜利者出现
  - 新回合主动玩家轮换
- 但它仍然没有到“完整可玩”：
  - 还没有基于 `WinnerPlayerID` 停止后续动作
  - 还没有更完整的地区与得分节奏
  - 还没有把这些状态接成更正式的玩家操作流

## 2026-04-01 第四次补记

- `Phase 3` 这一步把“胜利出现后对局应停止”也接上了：
  - Go legality 会在 `WinnerPlayerID != ""` 后拒绝后续动作
  - 拒绝码为 `RULES_FAILED_GAME_ALREADY_OVER`
  - `ActionRejected.context.winnerPlayerId` 会明确指出当前胜者
  - live sandbox 的动作面板会在 winner 存在时禁用，并显示 winner 提示
- 因此当前最小 sandbox 已经具备：
  - 地区争夺
  - 得分
  - 胜利产生
  - 新回合主动玩家轮转
  - 胜利后停止继续提交动作
- 当前仍未完成的，是把这些规则进一步拓展成更完整的一局流程，而不是继续在“对局已结束”之后推进状态。

## 2026-03-31 执行顺序说明

- 当前仓库状态已经从“纯规则核骨架”推进到了 `LIVE_SANDBOX` 阶段，因此下一步不建议立刻继续堆按钮、堆卡或上真实联机。
- 当前最优先级应该调整为：
  1. 冻结 `M0 sandbox` 基线
  2. 建立 golden scenarios / replay / invariants / projection leak 回归护城河
  3. 再进入第一套完整可玩规则闭环
  4. 最后才扩首发卡池与真实联机
- 换句话说，这份文档里的 4 个里程碑仍然成立，但当前执行顺序应以前两项“稳固基线”和“最小完整闭环”为先。
- 具体落地顺序见 [NEXT_STEP_EXECUTION_PLAN_2026-03-31.md](./NEXT_STEP_EXECUTION_PLAN_2026-03-31.md)。

## 2026-03-31 第五次补记（sandbox 终局态与重开一局闭环）

- 已完成“winner gate -> 正式 MatchState”的升级：
  - `GameState.Match` 作为正式状态源，包含 `status / endReason / winnerPlayerId / finishedAtRevision`
  - 终局合法性 gate 改为读取 `Match.Status == finished`，拒绝码维持 `RULES_FAILED_GAME_ALREADY_OVER`
  - 拒绝上下文会携带 `winnerPlayerId`，便于前端直出终局原因
- 已完成 sandbox reset 通路：
  - `SandboxSession.Reset()` 会重建 canonical `NewM0SandboxState()`，重置投影器并重新生成 bootstrap `StatePatched` 批次
  - HTTP 新增 `POST /api/debugger/reset`，直接返回 reset 后的新 bootstrap 消息流
- 已完成 Web 调试器 reset 交互：
  - Action 面板新增 `Reset Sandbox` 按钮，调用 `/api/debugger/reset`
  - 若当前 patch 处于终局（`winnerPlayerId` 非空），动作按钮禁用并展示 `Game over. Winner: ...`
  - reset 成功后会替换消息流并恢复可提交动作状态
- 这意味着浏览器里的最小闭环已经可直接体验：
  1. 打到终局
  2. 看到终局态（非仅 score winner 字段）
  3. 一键重开同一 sandbox 会话的新对局

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
