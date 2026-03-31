# Discard / Graveyard Semantics V1.5 - 实施计划

## 概述

**目标**：

* V1：把“进入 discard/坟墓”从零散赋值，收敛成统一的规则语义与实现入口，保证 replay / projection / invariant 一致

* V1.5：在 V1“统一进 discard 语义”基础上，再增加一个最小可扩展能力：DSL 可显式把目标送入 discard（discardCard）

* 这一步为后续真实卡（弃置/牺牲/处决）打基础，但不进入复活/坟墓资源系统

**记忆约束严格执行**：

* ✅ TDD：先写测试，再实现

* ✅ 形式化思维：准确定义问题，严格建模

* ✅ 数字卡牌游戏严格要求：容不得一丝宽松懈怠

**范围外（这轮不要做）**：

* ❌ 不做从坟墓回手/复活/检索

* ❌ 不做 asset/permanent V1 之外的新子系统

* ❌ 不做 XQ01、本地区沉默、UI/transport 改造

* ❌ 不顺手扩更多卡语义

***

## 形式化问题定义

### Discard 进场公理

**Discard 统一 transition 公理**：

* Forall 卡片 c ∈ Board.Cards,
  when moveCardToDiscard(c) is called:

  1. c.Zone = CardZoneDiscard
  2. c.Destroyed = true
  3. c.Revealed = true
  4. 所有 table-only 状态被清理（如需要）

**DSL discardCard 公理**：

* Forall DSL effect with kind = "discardCard",
  when targetRef = "selected" 且 目标卡片 t ∈ Board.Cards,
  after resolution:

  1. t 经过统一 discard transition
  2. 目标不存在或不合法时，保持现有错误处理风格

***

## \[x] Task 0: 先写测试 - Discard 基础测试（TDD，红测优先）

* **Priority**: P0

* **Depends On**: None

* **Description**:

  * 在 `discard_test.go` 中先写红测

  * 测试场景：

    1. 致命伤害导致离场的字段一致性
    2. discardCard DSL effect 能正确把目标送入 discard
    3. discard 后 projection 可见性
    4. discard 后 continuous effect 不应错误保留

* **Success Criteria**:

  * 测试文件已添加，测试用例完整覆盖形式化问题

  * 测试运行结果为红（先红后绿，TDD 流程）

* **Test Requirements**:

  * `programmatic` TR-0.1: 测试文件包含完整测试场景

  * `programmatic` TR-0.2: 测试断言严格按照公理编写

  * `programmatic` TR-0.3: 测试运行失败（红测状态）

***

## \[x] Task 1: 统一 discard transition helper

* **Priority**: P0

* **Depends On**: Task 0

* **Description**:

  * 在 rules core 增加统一 helper（例如 moveCardToDiscard）

  * 统一处理进入 discard 时的关键字段（Zone/Destroyed/Revealed 以及必须清理的 table-only 状态）

  * 当前“致命伤害离场”必须改为走这个 helper，不得再散写字段

* **Success Criteria**:

  * moveCardToDiscard 函数已实现

  * applyDerivedBoardSemantics 使用统一 helper

  * 致命伤害路径测试通过

* **Test Requirements**:

  * `programmatic` TR-1.1: moveCardToDiscard 已实现

  * `programmatic` TR-1.2: 旧回归不退化（attack/investigate/region scoring）

***

## \[x] Task 2: 增加 DSL `discardCard` 最小 effect

* **Priority**: P0

* **Depends On**: Task 1

* **Description**:

  * 在 DSL effect 里新增 `discardCard`（最小形态：targetRef=selected/controller 里至少支持 selected）

  * 执行时调用统一 discard helper，而不是直接改字段

  * 若目标不存在/不合法，保持现有错误处理风格

* **Success Criteria**:

  * discardCard DSL effect 已实现

  * 调用 moveCardToDiscard

  * 相关测试通过

* **Test Requirements**:

  * `programmatic` TR-2.1: discardCard DSL effect 已实现

  * `programmatic` TR-2.2: queue/resolve 后目标进入 discard

  * `programmatic` TR-2.3: 目标不存在/不合法时正确处理

***

## [x] Task 3: 生命周期一致性

* **Priority**: P1

* **Depends On**: Task 2

* **Description**:

  * continuous / attachment 对“目标进入 discard”后的行为保持一致，不出现脏引用或幽灵效果

  * 不新开 attachment/permanent 大重构，只修 V1.5 触达路径

* **Success Criteria**:

  * target 进入 discard 后相关 continuous effect 不应错误保留

  * 现有 attachment tracking V0 相关用例保持通过

* **Test Requirements**:

  * `programmatic` TR-3.1: continuous effect 生命周期一致性

  * `programmatic` TR-3.2: attachment tracking V0 测试保持通过

***

## [x] Task 4: 投影/回放/不变量

* **Priority**: P0

* **Depends On**: Task 3

* **Description**:

  * projection 对 discard 卡展示保持既有 hidden-info 边界

  * replay 同 action log 到同状态

  * 如新增 invariant，保持严格，不放宽标准

* **Success Criteria**:

  * projection discard 可见性一致

  * replay 一致性通过

  * invariant 约束保持严格

* **Test Requirements**:

  * `programmatic` TR-4.1: projection discard 可见性断言

  * `programmatic` TR-4.2: replay 一致性断言

  * `programmatic` TR-4.3: invariant 断言通过

***

## [x] Task 5: 文档同步

* **Priority**: P1

* **Depends On**: Task 4

* **Description**:

  * 更新：

    * `/Users/ddd/Downloads/UndergroundBattle/docs/NEXT_GEN_RULE_PLAN.md`

    * `/Users/ddd/Downloads/UndergroundBattle/docs/HANDOVER_TRAE_2026-04-01.md`

  * 文档必须写清：

    * 完成的是 “Discard/Graveyard Semantics V1.5”

    * 已支持统一离场 + `discardCard` 最小 DSL

    * 仍未做：坟墓检索、复活、回手、坟墓作为资源区

* **Success Criteria**:

  * 文档已同步更新

* **Test Requirements**:

  * `human-judgement` TR-5.1: 文档更新内容准确

