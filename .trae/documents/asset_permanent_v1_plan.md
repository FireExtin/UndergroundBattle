# Asset / Permanent Model V1 - 实施计划

## 概述

**目标**：把"资产牌作为真实在场永久物"这层状态模型做出来，让它能正式进场、在场、离场、被投影、被回放，而不是只停留在 fixture target kind 或附属追踪元数据层。

**记忆约束严格执行**：

* ✅ TDD：先写测试，再实现

* ✅ 形式化思维：准确定义问题，严格建模

* ✅ 数字卡牌游戏严格要求：容不得一丝宽松懈怠

**明确不做**：

1. ❌ 不做 BQ022 这类附属/结附牌的完整语义
2. ❌ 不做回收/回手
3. ❌ 不做暗藏部署
4. ❌ 不做资产主动能力
5. ❌ 不做更多 UI

***

## 形式化问题定义

### Asset Card 语义

**Asset Card 进场公理**：

* Forall 资产牌 a ∈ Hand,
  when a is played via queue\_operation,
  after resolution:

  1. a.Zone = CardZoneTable
  2. a.Destroyed = false
  3. a.Exhausted = false (初始化时)

**Asset Card 离场公理**：

* Forall 资产牌 a ∈ Table,
  when a is destroyed or removed from table:

  1. a.Zone = CardZoneDiscard
  2. a.Destroyed = true

**Asset Card 投影公理**：

* Forall 资产牌 a,
  projection respects same visibility rules as character/region cards
  (no information leak)

***

## \[x] Task 0: 先写测试 - Asset Card 基础测试（TDD，红测优先）

* **Priority**: P0

* **Depends On**: None

* **Description**:

  * 在 `types_test.go` 或新的 `asset_test.go` 中先写红测

  * 测试场景：

    1. Asset Card 可以设置为 CardKind
    2. Asset Card 可以进场到 Table
    3. Asset Card 可以离场进入 Discard
    4. Asset Card 的投影不泄露不该泄露的信息
    5. Asset Card 不影响现有 Character/Region 语义

* **Success Criteria**:

  * 测试文件已添加，测试用例完整覆盖形式化问题

  * 测试运行结果为红（先红后绿，TDD 流程）

* **Test Requirements**:

  * `programmatic` TR-0.1: 测试文件包含完整测试场景

  * `programmatic` TR-0.2: 测试断言严格按照公理编写

  * `programmatic` TR-0.3: 测试运行失败（红测状态）

***

## \[x] Task 1: 补状态模型 - 添加 CardKindAsset

* **Priority**: P0

* **Depends On**: Task 0

* **Description**:

  * 在 `projection.go` 的 CardKind 枚举中添加 `CardKindAsset`

  * 验证：不会破坏现有 character/region 语义

* **Success Criteria**:

  * CardKindAsset 正式存在于 rules core

  * 所有现有测试保持通过

* **Test Requirements**:

  * `programmatic` TR-1.1: CardKindAsset 常量已定义

  * `programmatic` TR-1.2: 所有现有 Go 测试通过

***

## \[x] Task 2: 补 clone helpers - 确保 asset 被正确克隆

* **Priority**: P0

* **Depends On**: Task 1

* **Description**:

  * 检查 `clone.go` 文件

  * 确保 CardState 克隆包含所有字段，包括 Kind

  * 验证：克隆后 asset 的 Kind 保持不变

* **Success Criteria**:

  * Asset Card 可以被正确克隆

  * 所有现有测试保持通过

* **Test Requirements**:

  * `programmatic` TR-2.1: Asset 克隆后 Kind 不变

  * `programmatic` TR-2.2: 所有现有 Go 测试通过

***

## \[x] Task 3: 补 invariants - 添加 asset 相关不变量检查

* **Priority**: P0

* **Depends On**: Task 2

* **Description**:

  * 在 `invariants.go` 中检查 asset 的状态一致性

  * 检查：

    1. Asset 在 Table 时不应该被 Destroyed（除非刚被销毁）
    2. Asset 在 Discard 时应该被 Destroyed

* **Success Criteria**:

  * Invariant 检查覆盖 Asset Card 状态

  * 所有现有测试保持通过

* **Test Requirements**:

  * `programmatic` TR-3.1: Invariants 包含 Asset 检查

  * `programmatic` TR-3.2: 所有现有 Go 测试通过

***

## \[x] Task 4: 补最小上场通路 - Asset 能进场到 Table

* **Priority**: P0

* **Depends On**: Task 3

* **Description**:

  * 让一张最简单的资产牌在 queue\_operation/resolve 后真正 materialize 到 table

  * 先只做"进场成为 permanent"

  * 不要顺手加复杂效果

  * 可以先用一张 test-only fixture 验证机制

* **Success Criteria**:

  * Asset Card 可以进场到 Table

  * 进场后状态正确（Zone=Table, Destroyed=false）

* **Test Requirements**:

  * `programmatic` TR-4.1: Asset 进场测试通过

  * `programmatic` TR-4.2: 所有现有 Go 测试通过

***

## \[x] Task 5: 补最小离场通路 - Asset 能离场进入 Discard

* **Priority**: P0

* **Depends On**: Task 4

* **Description**:

  * 定义资产 permanent 的最小离场语义：

    * 被销毁或移出 table 后，进入 discard

  * 确保 replay/revision/projection 一致

* **Success Criteria**:

  * Asset Card 可以离场进入 Discard

  * 离场后状态正确（Zone=Discard, Destroyed=true）

* **Test Requirements**:

  * `programmatic` TR-5.1: Asset 离场测试通过

  * `programmatic` TR-5.2: 所有现有 Go 测试通过

***

## \[x] Task 6: 补 continuous source validity - Asset 可以作为 continuous effect 源

* **Priority**: P1

* **Depends On**: Task 5

* **Description**:

  * 确保 continuous source validity 能正确识别这类真实 permanent source

  * Asset 在场时可以作为 continuous effect 源

  * Asset 离场时 continuous effect 自动失效

* **Success Criteria**:

  * Asset 可以作为 continuous effect 源

  * 离场后效果自动失效

* **Test Requirements**:

  * `programmatic` TR-6.1: Asset 作为 source 测试通过

  * `programmatic` TR-6.2: 所有现有 Go 测试通过

***

## \[x] Task 7: 补测试护栏 - 完整测试覆盖

* **Priority**: P0

* **Depends On**: Task 6

* **Description**:

  * 必须先写测试再实现（已在 Task 0 完成红测）

  * 覆盖：

    * asset 进入 table

    * asset 离场进入 discard

    * projection 不泄露不该泄露的信息

    * replay 后状态一致

    * 不影响现有角色/地区/XQ22/XQ31/attachment tracking V0

* **Success Criteria**:

  * 所有测试通过

  * 不破坏现有功能

* **Test Requirements**:

  * `programmatic` TR-7.1: 所有 Asset 测试通过

  * `programmatic` TR-7.2: 所有现有 Go 测试通过

***

## \[x] Task 8: 同步文档

* **Priority**: P1

* **Depends On**: Task 7

* **Description**:

  * 在 `docs/NEXT_GEN_RULE_PLAN.md` 中添加新的补记

  * 说明这是 Asset / Permanent V1，不是完整 attachment system

* **Success Criteria**:

  * 文档已同步更新

* **Test Requirements**:

  * `human-judgement` TR-8.1: 文档更新内容准确

