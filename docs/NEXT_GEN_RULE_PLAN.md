# 下一阶段规则核计划 (Next Gen Rule Plan)

## Summary
- 当前已经有：最小规则核、priority/stack、projection、continuous effects、`inspect` 的 permission hook、`dealDamage` 对 `EffectiveStats.Defense` 的最小致命判定，以及第一批角色动作入口 `declare_attack / declare_investigation`。
- **已完成 (Consolidation Phase)**：全局领域语义收口的第一轮。显式会话生命周期、元数据驱动动作策略 (ActionPolicy)、正式战斗动作支付并轨、投影契约加固。
- **本轮新增基线**：recovery 会恢复 table 上的角色/资产 ready 状态；地区控制已能基于“本地区未横置角色的 influence + 持久地区势力”动态刷新；projection / battle UI 已能展示地区控制者与按玩家分布的当前势力。
- **2026-04-03 继续推进**：`main` 已拆成 `first_player_action / second_player_action`；`conflict` 已成为正式 phase；调查奖励 / 战斗奖励已有 prompt 状态；`reveal_face_down` / `activate_ability` 已进入权威动作管线；prompt 私有信息已按 viewer 做投影裁剪。
- 如果目标是“继续稳定接真实卡 DSL”，接下来的核心矛盾是：`Attachment` 完整生命周期、`World Deck` 地区规则、`End Step` 自动处理、`Continuous Effect` 叠加、`Trigger` 队列。

## 目标 (Goals)
- 完成 `Region Scoring` (占领/计分)。
- 完成 `Role Actions` (攻击/调查) 完整流程。
- 实现 `Continuous Effect` 叠加（不仅仅是覆盖）。
- 实现 `End of Turn` 自动维护。

## 核心约束 (Constraints)
- 保持“Go 为唯一真相源”。
- 所有的“规则”必须能在 Go Test 中独立验证。
- 前端只负责“投影渲染”和“前置格式校验”。
- 不做向后兼容；过渡层、兼容层、双轨语义在影响理解时应直接删除。

---

## 计划路线图 (Roadmap)

### PN-BASE: 基础机制加固 (已完成)
- [x] **PN-BASE-001: 支付引擎/合法性并轨** (已完成)
  - 验收标准：`play_card` / `build_asset` 必须通过统一支付逻辑；`queue_operation` 继续保留调试通道兼容。
- [x] **PN-BASE-002: 正式开局构筑合法性 (派系)** (已完成)
  - 验收标准：开局阶段验证每位玩家仅选择 2 派系且牌库牌系合法。
- [x] **PN-BASE-003: 构筑限制 (张数/副本)** (已完成)
  - 验收标准：支持 40-60 张牌库和同名 3 副本限制。
- [x] **PN-BASE-004: 忠诚/颜色前提校验** (已完成)
  - 验收标准：`play_card` 必须检查场上角色提供的颜色；`queue_operation` 不作为正式对战忠诚入口。
- [x] **PN-BASE-006: 交互式开局 (先手选择)** (已完成)
  - 验收标准：支持上一局败者决定先手的交互分支。
- [x] **PN-BASE-007: 丰富视觉表现** (已完成)
  - 验收标准：牌桌支持状态（横置/暗置）和数值统计（战斗/防御/调查）的直观显示。

### PN-REG: 地区与计分 (Phase 3 Active)
- [ ] **PN-REG-001: 地区势力值动态计算**
  - 验收标准：地区卡 `EffectiveStats.Influence` 随驻场角色变化。
  - 当前状态：`ControllerID` 与 `InfluenceByPlayer` 已可根据本地区 ready 角色动态刷新，但还未把该结果正式折叠进 `EffectiveStats.Influence` 字段。
- [ ] **PN-REG-002: 地区计分快照**
  - 验收标准：每回合结束检测地区占领情况并增加玩家分数。
- [ ] **PN-REG-003: 胜利判定规则**
  - 验收标准：分数达到阈值（如 100）时自动触发 `MatchFinished`。

### PN-ACT: 规则书行动权与对抗流程 (Phase 3 Active)
- [x] **PN-ACT-001: 主行动步骤拆分**
  - 验收标准：`main` 拆成 `first_player_action / second_player_action`，后手行动步骤由后手拿初始行动权。
- [x] **PN-ACT-002: 正式 conflict phase**
  - 验收标准：地区按顺序进入调查 / 战斗 / 势力对抗，不再依赖 battle UI 手动 `declare_investigation`。
- [x] **PN-ACT-003: 调查奖励 / 战斗奖励 prompt**
  - 验收标准：调查奖励按差额检视并抓 1；战斗奖励进入伤害分配 prompt；二者均不入栈。
- [x] **PN-ACT-004: 现身与行动能力正式动作**
  - 验收标准：`reveal_face_down` / `activate_ability` 进入统一 legality + stack 管线。
- [ ] **PN-ACT-005: tie + 先手标志特权并入新 conflict 流程**
  - 验收标准：调查 / 战斗 / 势力对抗打平时，先手标志特权能在新 phase/stage 模型下正确接管。
- [ ] **PN-ACT-006: 拦截 / 更完整战斗结算**
  - 验收标准：攻击者横置 -> 对方指定拦截 -> 计算伤害 -> 致命判定 -> 离场。
- [ ] **PN-ACT-007: 能力注册表扩面**
  - 验收标准：行动能力不再只有示例卡，battle 常见 quick/action abilities 都能正式发动。

### PN-SYS: 系统扩展
- [ ] **PN-SYS-001: 效果叠加系统 (Layer System)**
  - 验收标准：支持多个数值光环叠加（如 +1/+1 和 +2/+2 得到 +3/+3）。
- [ ] **PN-SYS-002: 触发器队列 (Trigger Stack)**
  - 验收标准：支持“当某事发生时”入栈。
