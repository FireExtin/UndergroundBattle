# 5个分计划串行执行总计划

## 记忆约束（严格执行）

* ✅ **TDD**：先写测试（红）-> 最小实现（绿）-> 回归验证

* ✅ **形式化思维**：准确定义问题，严格建模

* ✅ **数字卡牌游戏严格要求**：容不得一丝宽松懈怠

* ✅ **不允许通过放宽 invariant、删断言、降级测试来"修绿"**

## 执行顺序（必须按顺序）

***

## Plan 1: Attachment / Host Lifecycle V1

### 目标

把附属从 tracking V0 推进到真实生命周期：附属在场、attachedTo、宿主离场联动、附属离场联动。

### 范围

* ✅ 给附属建立正式 host 关系字段（attachedToCardID）

* ✅ 宿主离场时：附属按规则离场（最小默认：进 discard）

* ✅ 附属离场时：由附属提供的 continuous effect 同步失效

* ✅ replay / projection / invariant 保持一致

### 范围外

* ❌ 不做"回手/回收"复杂牌面语义

* ❌ 不做完整 attachment stack 互动

### Task 1.0: 先写测试 - Attachment 基础测试（TDD，红测优先）

* **Priority**: P0

* 在 `attachment_lifecycle_test.go` 中先写红测

* 测试场景：

  1. 宿主离场时：附属按规则离场
  2. 附属离场时：由附属提供的 continuous effect 同步失效
  3. 现有 Attachment tracking V0 不退化

* **Success Criteria**: 测试运行结果为红（先红后绿）

### Task 1.1: 给 Attachment 添加 HostCardID 字段

* **Priority**: P0

* 在 `Attachment` 结构体中添加 `HostCardID` 字段

* 在 `AttachmentBuilder` 中支持设置 HostCardID

* **Success Criteria**: 所有现有 Go 测试通过

### Task 1.2: 宿主离场联动

* **Priority**: P0

* 在 `pruneExpiredAttachments` 中增加检查：宿主不在场时附属自动移除

* **Success Criteria**: 宿主离场时，所有附属自动从 Active 中移除

### Task 1.3: 附属离场联动 - effect 失效

* **Priority**: P0

* 确保当附属离场时，由该附属提供的 continuous effect 同步失效

* 利用现有 continuous source validity 机制

* **Success Criteria**: 附属离场时，相关 continuous effect 自动失效

### Task 1.4: 回归验证

* **Priority**: P0

* 运行所有测试：

  * `go test ./server/...`

  * `(cd tools/fixture-tools && npm test)`

  * `(cd web && npm test)`

* **Success Criteria**: 所有测试通过

### Task 1.5: 文档同步

* **Priority**: P1

* 更新 `docs/NEXT_GEN_RULE_PLAN.md` 和 `docs/HANDOVER_TRAE_2026-04-01.md`

* 明确标注这是 "Attachment / Host Lifecycle V1"

***

## Plan 2: Secret Society Marker V1

### 目标

把"秘社标记物"从 schema 预留变成 rules authoritative state。

### 范围

* ✅ 定义 marker state（建议按 player 维度）

* ✅ 新增最小动作：增加/移除 marker

* ✅ projection 输出公开可见部分（若有私有信息，遵守 hidden-info 边界）

* ✅ legality 支持 marker 作为条件（先做最小 hook）

### 范围外

* ❌ 不做大量真实卡接入

* ❌ 不做 UI 大改，只保证协议里有数据

### Task 2.0: 先写测试 - Marker 基础测试（TDD，红测优先）

* **Priority**: P0

* 在 `marker_test.go` 中先写红测

* 测试场景：

  1. marker 增减
  2. 回放一致
  3. 投影一致
  4. legality 条件支持

* **Success Criteria**: 测试运行结果为红（先红后绿）

### Task 2.1: 定义 Marker State 模型

* **Priority**: P0

* 在 `GameState` 或 `PlayerState` 中添加 marker 存储

* 定义 marker 的数据结构（类型、数量、可见性等）

* **Success Criteria**: Marker state 模型已定义

### Task 2.2: 实现 Marker 增减动作

* **Priority**: P0

* 新增 DSL effect：`addMarker`、`removeMarker`

* 或新增 action kind：`addMarker`、`removeMarker`

* **Success Criteria**: marker 可以增减

### Task 2.3: Projection 支持

* **Priority**: P0

* 在 `projection.go` 中处理 marker 的可见性

* 公开 marker 对所有玩家可见

* 私有 marker 只对 owner 可见

* **Success Criteria**: projection 正确输出 marker

### Task 2.4: Legality 最小 Hook

* **Priority**: P1

* 在 legality 检查中支持 marker 作为条件

* 例如：检查玩家是否有足够的 marker

* **Success Criteria**: legality 可以检查 marker 条件

### Task 2.5: 回归验证

* **Priority**: P0

* 运行所有测试

* **Success Criteria**: 所有测试通过

### Task 2.6: 文档同步

* **Priority**: P1

* 更新文档，标注 "Secret Society Marker V1"

***

## Plan 3: Hidden Deployment & Reveal V1

### 目标

实现"暗藏部署 -> 现身"最小闭环，而不是只有 reveal 动作壳子。

### 范围

* ✅ 新增/规范 face-down deployed permanent 状态

* ✅ 只有 owner 可见真实信息，其他视角按 hidden card 投影

* ✅ reveal 后状态转换正确，并触发必要的连续效果重算

* ✅ legality 校验：非法 reveal/非法目标会拒绝

### 范围外

* ❌ 不做完整伏击时机学

* ❌ 不做复杂触发链

### Task 3.0: 先写测试 - Hidden Deployment 基础测试（TDD，红测优先）

* **Priority**: P0

* 在 `hidden_deployment_test.go` 中先写红测

* 测试场景：

  1. face-down 部署
  2. owner 可见真实信息
  3. 其他视角 hidden card 投影
  4. reveal 后状态转换
  5. reveal 触发 continuous recalculation

* **Success Criteria**: 测试运行结果为红（先红后绿）

### Task 3.1: 定义 Face-Down 状态模型

* **Priority**: P0

* 在 `CardState` 中添加 `FaceDown` 或 `Hidden` 字段

* 定义 face-down 卡片的可见性规则

* **Success Criteria**: Face-down 状态模型已定义

### Task 3.2: 实现部署动作

* **Priority**: P0

* 新增 DSL effect 或 action kind 支持 face-down 部署

* 部署时设置 face-down 状态

* **Success Criteria**: 可以 face-down 部署卡片

### Task 3.3: Projection 可见性控制

* **Priority**: P0

* 在 `projection.go` 中处理 face-down 卡片的可见性

* owner 看到完整信息

* 其他玩家看到隐藏信息（可能只有 cardID、zone 等）

* **Success Criteria**: projection 正确控制可见性

### Task 3.4: Reveal 动作实现

* **Priority**: P0

* 新增 `reveal` action kind 或 DSL effect

* reveal 后：

  * 卡片状态变为 face-up

  * 触发 continuous recalculation

* **Success Criteria**: reveal 动作正常工作

### Task 3.5: Legality 校验

* **Priority**: P1

* 校验 reveal 的合法性

* 非法 reveal/非法目标会拒绝

* **Success Criteria**: legality 正确校验 reveal

### Task 3.6: 回归验证

* **Priority**: P0

* 运行所有测试

* **Success Criteria**: 所有测试通过

### Task 3.7: 文档同步

* **Priority**: P1

* 更新文档，标注 "Hidden Deployment & Reveal V1"

***

## Plan 4: Timing Window V2 (Fast/Reaction)

### 目标

把当前最小时机模型扩到可承载更多 fast/reaction 牌，而不破坏已有优先权闭环。

### 范围

* ✅ 明确 action/response 窗口准入矩阵

* ✅ reaction 必须依附 stack 非空与合法触发上下文

* ✅ 统一错误码与拒绝上下文（便于前端展示）

### 范围外

* ❌ 不做完整 MTG 级别时机系统

* ❌ 不做 replacement effects

### Task 4.0: 先写测试 - Timing Window 基础测试（TDD，红测优先）

* **Priority**: P0

* 在 `timing_window_test.go` 中先写红测

* 测试场景：

  1. fast action 允许/拒绝
  2. reaction 允许/拒绝（stack 非空）
  3. 错误码统一
  4. 现有 priority/stack golden scenario 不回归

* **Success Criteria**: 测试运行结果为红（先红后绿）

### Task 4.1: 定义 Timing Window 模型

* **Priority**: P0

* 定义 action/response 窗口准入矩阵

* 明确哪些时机可以打 fast/reaction

* **Success Criteria**: Timing window 模型已定义

### Task 4.2: Fast Action 支持

* **Priority**: P0

* 在 legality 中支持 fast action 检查

* fast action 可以在特定时机发动

* **Success Criteria**: fast action 可以正确发动

### Task 4.3: Reaction 支持

* **Priority**: P0

* reaction 必须依附 stack 非空

* reaction 必须有合法触发上下文

* **Success Criteria**: reaction 可以正确发动

### Task 4.4: 统一错误码

* **Priority**: P1

* 统一 timing window 相关的错误码

* 提供清晰的拒绝上下文

* **Success Criteria**: 错误码统一且清晰

### Task 4.5: 回归验证

* **Priority**: P0

* 运行所有测试

* 确保现有 priority/stack golden scenario 不回归

* **Success Criteria**: 所有测试通过

### Task 4.6: 文档同步

* **Priority**: P1

* 更新文档，标注 "Timing Window V2"

***

## Plan 5: Conflict Loop V2 (战斗/调查/地区节奏)

### 目标

在现有 role action V1 基础上补"可玩但可控"的对抗节奏。

### 范围

* ✅ 战斗：最小对抗扩展（目标合法性、结算顺序一致化）

* ✅ 调查：与地区控制/得分节奏衔接更清晰

* ✅ end->main 结算链路与 game-over gate 做一次一致性清理

### 范围外

* ❌ 不做完整战斗子阶段系统

* ❌ 不做大规模扩卡

### Task 5.0: 先写测试 - Conflict Loop 基础测试（TDD，红测优先）

* **Priority**: P0

* 在 `conflict_loop_test.go` 中先写红测

* 测试场景：

  1. 战斗目标合法性
  2. 结算顺序一致化
  3. 调查与地区控制衔接
  4. end->main 结算链路
  5. game-over gate

* **Success Criteria**: 测试运行结果为红（先红后绿）

### Task 5.1: 战斗对抗扩展

* **Priority**: P0

* 完善战斗目标合法性检查

* 确保结算顺序一致化

* **Success Criteria**: 战斗对抗扩展完成

### Task 5.2: 调查与地区控制衔接

* **Priority**: P0

* 调查动作与地区控制/得分节奏衔接更清晰

* 确保 influence 放置和 region control 更新一致

* **Success Criteria**: 调查与地区控制衔接完成

### Task 5.3: End->Main 结算链路清理

* **Priority**: P0

* 清理 end->main 的结算链路

* 确保回合切换、得分、胜利条件检查顺序正确

* **Success Criteria**: 结算链路清理完成

### Task 5.4: Game-Over Gate 一致性

* **Priority**: P0

* 确保 game-over gate 逻辑一致

* 胜利条件检查正确

* **Success Criteria**: game-over gate 一致

### Task 5.5: Golden Scenarios

* **Priority**: P0

* 新增至少 3 条 conflict golden scenarios

* 覆盖战斗、调查、地区节奏

* **Success Criteria**: 3 条 golden scenarios 通过

### Task 5.6: 回归验证

* **Priority**: P0

* 运行所有测试

* replay + invariants + projection regression 全绿

* **Success Criteria**: 所有测试通过

### Task 5.7: 文档同步

* **Priority**: P1

* 更新文档，标注 "Conflict Loop V2"

***

## 全局验收标准（每个计划都要满足）

1. **go test ./server/... 通过**
2. **(cd tools/fixture-tools && npm test) 通过**
3. **(cd web && npm test) 通过**
4. **更新文档**：

   * docs/NEXT\_GEN\_RULE\_PLAN.md

   * docs/HANDOVER\_TRAE\_2026-04-01.md

## 每完成一个计划的汇报内容

* 改动文件列表

* 新增/修改的关键测试名

* 运行的命令与结果

* 风险与 defer 项（明确写"没做什么"）

