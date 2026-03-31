# XQ31 数值光环（+1 防御力）实施计划

## 严格遵循的原则
- ✅ **TDD**：先写测试，再实现代码
- ✅ **准确定义问题**：使用形式化思维分析问题
- ✅ **严格建模**：数字卡牌游戏容不得一丝宽松懈怠
- ✅ **目标**：完成 XQ31 的第一个效果：所有本方声望角色获得 +1 防御力

## 问题的形式化定义

### 前置条件 (Preconditions)
给定 GameState S，场上有 Card C 满足：
- C.DefinitionID = "XQ31"
- C.Zone = CardZoneTable
- C.Ready = true
- C.Destroyed = false

### 效果谓词 (Effect Predicate)
对于场上每个 Card T：
如果：
  - T.Zone = CardZoneTable
  - T.Destroyed = false
  - T.ControllerID = C.ControllerID
  - "声望" ∈ T.PrintedKeywords

则：
  T.EffectiveStats.Defense = T.PrintedStats.Defense + 1

否则：
  T.EffectiveStats.Defense = T.PrintedStats.Defense

### 不变量 (Invariants)
1. 只有满足 SourceCondition 的 XQ31 实例才会产生效果
2. 效果只应用于本方声望角色
3. XQ31 离场/横置/摧毁后，效果立即停止

## 任务清单

### [ ] Task 1: 添加 XQ31 Continuous Effect 规则定义
- **Priority**: P0
- **Depends On**: None
- **Description**: 
  - 在 `legality_catalog.go` 中添加 XQ31 的 continuous effect 规则定义
  - 严格按照形式化定义
- **Success Criteria**:
  - 代码编译成功
  - 规则定义与形式化问题一致
- **Test Requirements (TDD - 先写测试)**:
  - `programmatic` TR-1.1: `TestBuildProductionContinuousEffectsReturnsXQ31Rule` - 验证规则被正确构建
  - `programmatic` TR-1.2: 验证规则的 SourceCondition 为 XQ31 在场且就绪
  - `programmatic` TR-1.3: 验证效果为 +1 defense，目标为声望盟友

### [ ] Task 2: 实现 Continuous Effects 的动态构建
- **Priority**: P0
- **Depends On**: Task 1
- **Description**: 
  - 在 `continuous.go` 中实现从场上卡牌构建 continuous effects 的逻辑
  - 类似于 prohibition 和 target legality 的构建方式
  - 严格按照形式化前置条件筛选源卡
- **Success Criteria**:
  - 能正确从场上 XQ31 构建 continuous effect
  - 不满足前置条件的 XQ31 不会产生效果
- **Test Requirements (TDD - 先写测试)**:
  - `programmatic` TR-2.1: `TestContinuousEffectsBuiltFromXQ31` - 验证当 XQ31 在场且就绪时，continuous effect 被正确创建
  - `programmatic` TR-2.2: `TestContinuousEffectsNotBuiltFromExhaustedXQ31` - 验证当 XQ31 横置时，continuous effect 不被创建
  - `programmatic` TR-2.3: `TestContinuousEffectsNotBuiltFromDestroyedXQ31` - 验证当 XQ31 摧毁时，continuous effect 不被创建

### [ ] Task 3: 实现 +1 防御力效果应用逻辑
- **Priority**: P0
- **Depends On**: Task 2
- **Description**: 
  - 实现 continuous effect 对目标卡牌的筛选逻辑
  - 严格按照形式化效果谓词应用 +1 defense
- **Success Criteria**:
  - 只有本方声望角色获得 +1 defense
  - 敌方和非声望角色不受影响
- **Test Requirements (TDD - 先写测试)**:
  - `programmatic` TR-3.1: `TestXQ31GrantsDefenseToPrestigeAlly` - 验证声望盟友防御+1
  - `programmatic` TR-3.2: `TestXQ31DoesNotAffectNonPrestigeAlly` - 验证非声望盟友防御不变
  - `programmatic` TR-3.3: `TestXQ31DoesNotAffectEnemy` - 验证敌方角色防御不变
  - `programmatic` TR-3.4: `TestXQ31DoesNotAffectDestroyedCard` - 验证已摧毁的角色不受影响

### [ ] Task 4: 集成到 RecalculateContinuousEffects
- **Priority**: P0
- **Depends On**: Task 3
- **Description**: 
  - 将 XQ31 的 continuous effect 构建集成到 `RecalculateContinuousEffects` 流程中
  - 确保每次重算时，XQ31 的效果被正确应用和清理
  - 严格遵循不变量
- **Success Criteria**:
  - XQ31 入场时添加效果
  - XQ31 离场/横置/摧毁时，效果被移除
- **Test Requirements (TDD - 先写测试)**:
  - `programmatic` TR-4.1: `TestXQ31EntersGameAddsDefenseBuff` - 验证 XQ31 入场时添加效果
  - `programmatic` TR-4.2: `TestXQ31LeavesGameRemovesDefenseBuff` - 验证 XQ31 离场时移除效果
  - `programmatic` TR-4.3: `TestXQ31ExhaustedRemovesDefenseBuff` - 验证 XQ31 横置时移除效果
  - `programmatic` TR-4.4: `TestXQ31DestroyedRemovesDefenseBuff` - 验证 XQ31 摧毁时移除效果

### [ ] Task 5: Golden Scenario 完整验证
- **Priority**: P0
- **Depends On**: Task 4
- **Description**: 
  - 添加完整的 Golden Scenario 测试
  - 覆盖完整游戏流程
- **Success Criteria**:
  - 所有测试通过
- **Test Requirements**:
  - `programmatic` TR-5.1: `TestGoldenScenario_XQ31GrantsDefenseToPrestigeAllies` - 完整场景验证

### [ ] Task 6: 同步文档
- **Priority**: P1
- **Depends On**: Task 5
- **Description**: 
  - 更新 `docs/NEXT_GEN_RULE_PLAN.md` 记录 XQ31 数值光环完成
  - 添加相关设计说明文档
- **Success Criteria**:
  - 文档已同步
- **Test Requirements**:
  - `human-judgement` TR-6.1: 文档更新完整准确，包含形式化问题定义
