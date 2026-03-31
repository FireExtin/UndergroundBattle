# 整理 Golden Scenario / Replay / Invariants 计划

## 目标
建立系统的健康检查机制，包括不变量（Invariants）、黄金场景（Golden Scenarios）和回放（Replay）验证，为后续功能开发提供坚实基础。

## 约束条件
1. 每次涉及核心业务逻辑、架构调整或功能模块的重要变更时，必须同步在 docs 目录下创建或更新相应文档
2. 严格依照 superpower 编码规范与最佳实践进行开发
3. 每次较重逻辑改动都必须先补测试，再做实现，再跑验证

## 实施步骤

### 阶段 1：定义核心 Invariants（预计 2-3 小时）

#### 任务 1.1：分析现有代码，识别关键不变量
- **行动**：审查 GameState、CardState、Board 等核心数据结构
- **产出**：列出 5-8 个候选不变量
- **文档**：创建 `docs/trae_review/invariants_design.md`

#### 任务 1.2：实现核心 Invariants 检查函数
- **行动**：在 `server/pkg/rules/invariants.go` 中实现以下不变量：
  - `InvariantCardIDUnique`：所有卡牌 ID 必须唯一
  - `InvariantCardZoneValid`：卡牌区域必须合法
  - `InvariantPriorityPlayerValid`：优先权玩家必须存在
  - `InvariantStackDepthNonNegative`：堆栈深度不能为负
  - `InvariantRevisionConsistent`：版本号必须递增
- **测试**：为每个 invariant 编写单元测试
- **文档**：更新 `docs/trae_review/invariants_design.md`

#### 任务 1.3：在关键位置插入 Invariant 检查
- **行动**：在 `SubmitAction` 和状态变更后插入 invariant 检查
- **配置**：添加开关（仅在测试/调试模式启用，生产环境可选）
- **测试**：验证 invariant 检查能捕获非法状态

### 阶段 2：创建 Golden Scenarios（预计 3-4 小时）

#### 任务 2.1：设计 Golden Scenario 格式
- **行动**：定义场景描述格式（Given-When-Then）
- **产出**：创建场景定义模板
- **文档**：创建 `docs/trae_review/golden_scenarios.md`

#### 任务 2.2：实现场景 1：XQ22 禁止事务卡
- **Given**：P1 场上有就绪的 XQ22
- **When**：P2 试图打出事务卡
- **Then**：动作被拒绝，返回 LEGALITY_FAILED_ACTION_PROHIBITED
- **实现**：在 `server/pkg/rules/golden_scenario_test.go` 中实现
- **文档**：更新 `docs/trae_review/golden_scenarios.md`

#### 任务 2.3：实现场景 2：XQ31 保护声望盟友
- **Given**：P1 场上有就绪的 XQ31，P1 有声望盟友
- **When**：P2 试图目标该声望盟友
- **Then**：动作被拒绝，返回 TARGET_FAILED_PROHIBITED
- **实现**：在 `server/pkg/rules/golden_scenario_test.go` 中实现
- **文档**：更新 `docs/trae_review/golden_scenarios.md`

#### 任务 2.4：实现场景 3：完整游戏回合
- **Given**：初始游戏状态，P1 有优先权
- **When**：P1 打出卡牌 -> P2 响应 -> 堆栈结算
- **Then**：状态正确变更，版本号递增，事件正确生成
- **实现**：在 `server/pkg/rules/golden_scenario_test.go` 中实现
- **文档**：更新 `docs/trae_review/golden_scenarios.md`

### 阶段 3：验证 Replay 系统（预计 2-3 小时）

#### 任务 3.1：分析现有 Replay 能力
- **行动**：审查 Revision、Action、Event 的关联
- **产出**：评估当前 replay 系统的完整性
- **文档**：创建 `docs/trae_review/replay_system_analysis.md`

#### 任务 3.2：实现 Replay 验证函数
- **行动**：创建 `ReplayValidator` 结构体
- **实现**：
  - `ValidateReplay(actions []Action, finalState GameState) error`
  - `ReplayActions(initialState GameState, actions []Action) (GameState, error)`
- **测试**：使用 golden scenarios 验证 replay 正确性
- **文档**：更新 `docs/trae_review/replay_system_analysis.md`

#### 任务 3.3：添加 Replay 回归测试
- **行动**：创建 `server/pkg/rules/replay_test.go`
- **测试**：
  - 测试 replay 能重现相同状态
  - 测试 replay 能检测不一致
- **文档**：更新 `docs/trae_review/replay_system_analysis.md`

### 阶段 4：整合与验证（预计 1-2 小时）

#### 任务 4.1：创建综合测试套件
- **行动**：创建 `server/pkg/rules/sanity_check_test.go`
- **实现**：运行所有 invariant、golden scenario、replay 测试
- **验证**：确保所有测试通过

#### 任务 4.2：更新项目文档
- **行动**：更新 `docs/trae_review/README.md` 或创建索引文档
- **内容**：
  - Invariants 列表和使用说明
  - Golden Scenarios 目录
  - Replay 系统说明
- **验证**：文档完整、准确

#### 任务 4.3：最终验证
- **行动**：运行完整测试套件
- **验证**：
  - 所有原有测试通过
  - 所有新测试通过
  - 代码构建成功
  - 文档同步完成

## 交付物清单

### 代码文件
1. `server/pkg/rules/invariants.go` - Invariants 检查实现
2. `server/pkg/rules/invariants_test.go` - Invariants 测试
3. `server/pkg/rules/golden_scenario_test.go` - Golden Scenarios 测试
4. `server/pkg/rules/replay_validator.go` - Replay 验证实现
5. `server/pkg/rules/replay_test.go` - Replay 测试
6. `server/pkg/rules/sanity_check_test.go` - 综合测试

### 文档文件
1. `docs/trae_review/invariants_design.md` - Invariants 设计文档
2. `docs/trae_review/golden_scenarios.md` - Golden Scenarios 文档
3. `docs/trae_review/replay_system_analysis.md` - Replay 系统分析
4. `docs/trae_review/sanity_check_summary.md` - 综合总结（可选）

## 验收标准

- [ ] 所有 5 个核心 Invariants 实现并通过测试
- [ ] 3 个 Golden Scenarios 实现并通过测试
- [ ] Replay 验证系统实现并通过测试
- [ ] 所有原有测试继续通过（回归测试）
- [ ] 所有文档同步创建并完整
- [ ] 代码符合 superpower 编码规范

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| Invariants 检查影响性能 | 中 | 仅在测试/调试模式启用 |
| Golden Scenarios 维护成本高 | 低 | 场景设计简洁，聚焦核心流程 |
| Replay 系统复杂度高 | 中 | 先实现基础验证，逐步完善 |

## 时间估算

- 阶段 1：2-3 小时
- 阶段 2：3-4 小时
- 阶段 3：2-3 小时
- 阶段 4：1-2 小时
- **总计：8-12 小时**
