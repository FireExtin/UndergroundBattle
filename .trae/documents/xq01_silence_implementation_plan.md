# XQ01 地区作用域沉默 - 实施计划（严格 TDD + 形式化）

## 概述

**目标**：完整实现 XQ01（联会禁音使）的持续效果：只要 XQ01 在场且未横置，所有角色不能发动攻击和调查。

**记忆约束严格执行**：

* ✅ TDD：先写测试，再实现

* ✅ 形式化思维：准确定义问题，严格建模

* ✅ 数字卡牌游戏严格要求：容不得一丝宽松懈怠

**前置框架状态**：✅ 完整，无需额外修改

* `permissionForActionKind` 已将 `DeclareAttack` 映射到 "attack"

* `permissionForActionKind` 已将 `DeclareInvestigation` 映射到 "investigate"

* `checkCardActionPermissionLegality` 已实现并集成到角色动作检查

***

## 形式化问题定义

**公理**：

* Forall 卡牌 c ∈ GameState.Board.Cards,
  if (c.DefinitionID == "XQ01" ∧ c.Zone == CardZoneTable ∧ ¬c.Exhausted ∧ ¬c.Destroyed)
  then forall 角色 d ∈ GameState.Board.Cards,
  (d.Kind == CardKindCharacter ∧ d.Zone == CardZoneTable ∧ ¬d.Destroyed)
  ⇒ ("attack" ∈ d.Prohibitions ∧ "investigate" ∈ d.Prohibitions)

**测试断言必须严格符合此公理**

***

## \[x] Task 0: 先写测试 - 添加 XQ01 单元测试（TDD，红测优先）

* **Priority**: P0

* **Depends On**: None

* **Description**:

  * 在 `continuous_test.go` 中先添加完整的 XQ01 测试用例（预期红测）

  * 严格按照形式化问题定义编写测试断言

  * 测试场景：

    1. **P1**: XQ01 在场且就绪时，所有角色不能攻击（形式化验证）
    2. **P2**: XQ01 在场且就绪时，所有角色不能调查（形式化验证）
    3. **P3**: XQ01 横置时，所有角色可以正常攻击
    4. **P4**: XQ01 离场时，所有角色可以正常攻击
    5. **P5**: XQ01 被摧毁时，所有角色可以正常攻击

* **Success Criteria**:

  * 测试文件已添加，测试用例完整覆盖形式化问题

  * 测试运行结果为红（先红后绿，TDD 流程）

* **Test Requirements**:

  * `programmatic` TR-0.1: 测试文件包含 5 个完整测试场景

  * `programmatic` TR-0.2: 测试断言严格按照公理编写

  * `programmatic` TR-0.3: 测试运行失败（红测状态）

***

## \[x] Task 1: 添加 XQ01 Continuous Effect 模板（禁止攻击）

* **Priority**: P0

* **Depends On**: Task 0（红测已存在）

* **Description**:

  * 在 `legality_catalog.go` 中添加 `XQ01SilenceAttackTemplate`

  * 严格按照形式化问题定义实现

  * 源卡条件：XQ01 在场、就绪、未被摧毁（¬c.Exhausted ∧ ¬c.Destroyed）

  * 目标条件：所有角色（无 Side 限制，全桌）

  * 效果：禁止 "attack" 权限

* **Success Criteria**:

  * XQ01 攻击禁止模板正确定义在 `legality_catalog.go` 中

  * 模板使用 `LayerProhibition` 和 `EffectKind: "prohibitPermission"`

  * 模板源条件严格匹配公理要求

* **Test Requirements**:

  * `programmatic` TR-1.1: 模板能够正确编译

  * `programmatic` TR-1.2: 模板源条件为 XQ01 在场且就绪且未被摧毁

***

## \[x] Task 2: 添加 XQ01 Continuous Effect 模板（禁止调查）

* **Priority**: P0

* **Depends On**: Task 1

* **Description**:

  * 在 `legality_catalog.go` 中添加 `XQ01SilenceInvestigateTemplate`

  * 严格按照形式化问题定义实现

  * 源卡条件：XQ01 在场、就绪、未被摧毁

  * 目标条件：所有角色（无 Side 限制，全桌）

  * 效果：禁止 "investigate" 权限

* **Success Criteria**:

  * XQ01 调查禁止模板正确定义在 `legality_catalog.go` 中

  * 模板源条件严格匹配公理要求

* **Test Requirements**:

  * `programmatic` TR-2.1: 模板能够正确编译

  * `programmatic` TR-2.2: 模板禁止 "investigate" 权限

***

## \[x] Task 3: 在 BuildProductionContinuousEffectTemplates 中注册 XQ01 模板

* **Priority**: P0

* **Depends On**: Task 2

* **Description**:

  * 将 `XQ01SilenceAttackTemplate` 和 `XQ01SilenceInvestigateTemplate` 添加到 `BuildProductionContinuousEffectTemplates` 返回的切片中

* **Success Criteria**:

  * XQ01 两个模板已正确注册到生产构建器

  * Task 0 中的测试现在应该开始变绿

* **Test Requirements**:

  * `programmatic` TR-3.1: 构建函数包含 XQ01 两个模板

  * `programmatic` TR-3.2: XQ01 单元测试 P1-P5 全部通过（绿测）

  * `programmatic` TR-3.3: 所有现有 Go 测试通过

***

## \[x] Task 4: 添加 XQ01 的 Golden Scenario 完整验证

* **Priority**: P0

* **Depends On**: Task 3

* **Description**:

  * 在 `golden_scenario_test.go` 中添加 `TestGoldenScenario_XQ01SilencesAttackAndInvestigation`

  * 覆盖完整游戏流程验证

  * 严格按照形式化问题定义验证

* **Success Criteria**:

  * Golden Scenario 测试完整覆盖游戏场景

  * Golden Scenario 测试通过

* **Test Requirements**:

  * `programmatic` TR-4.1: Golden Scenario 测试完整添加

  * `programmatic` TR-4.2: Golden Scenario 测试通过

***

## [x] Task 5: 同步文档

* **Priority**: P1

* **Depends On**: Task 4

* **Description**:

  * 在 `docs/NEXT_GEN_RULE_PLAN.md` 中添加新的补记

  * 补记内容必须包含形式化问题定义

* **Success Criteria**:

  * 文档已同步更新

* **Test Requirements**:

  * `human-judgement` TR-5.1: 文档更新内容准确描述了 XQ01 实现

  * `human-judgement` TR-5.2: 文档包含形式化问题定义

