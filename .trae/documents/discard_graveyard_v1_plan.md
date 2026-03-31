# Discard / Graveyard Semantics V1 - 实施计划

## 概述

**目标**：把“进入 discard/坟墓”从零散赋值，收敛成统一的规则语义与实现入口，保证 replay / projection / invariant 一致。

**记忆约束严格执行**：
- ✅ TDD：先写测试，再实现
- ✅ 形式化思维：准确定义问题，严格建模
- ✅ 数字卡牌游戏严格要求：容不得一丝宽松懈怠

**范围外（这轮不要做）**：
- ❌ 不做从坟墓回手/复活/检索
- ❌ 不做 asset/permanent V1 之外的新子系统
- ❌ 不做 XQ01、本地区沉默、UI/transport 改造
- ❌ 不顺手扩更多卡语义

---

## 形式化问题定义

### Discard 进场公理

**Discard 统一 transition 公理**：
- Forall 卡片 c ∈ Board.Cards,
  when moveCardToDiscard(c) is called:
    1. c.Zone = CardZoneDiscard
    2. c.Destroyed = true
    3. c.Revealed = true
    4. 所有 table-only 状态被清理（如需要）

---

## [/] Task 0: 先写测试 - Discard 基础测试（TDD，红测优先）
- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 在 `discard_test.go` 中先写红测
  - 测试场景：
    1. 致命伤害导致离场的字段一致性
    2. discard 后 projection 可见性
    3. discard 后 continuous effect 不应错误保留
- **Success Criteria**:
  - 测试文件已添加，测试用例完整覆盖形式化问题
  - 测试运行结果为红（先红后绿，TDD 流程）
- **Test Requirements**:
  - `programmatic` TR-0.1: 测试文件包含完整测试场景
  - `programmatic` TR-0.2: 测试断言严格按照公理编写
  - `programmatic` TR-0.3: 测试运行失败（红测状态）

---

## [ ] Task 1: 统一 discard transition helper
- **Priority**: P0
- **Depends On**: Task 0
- **Description**:
  - 在 rules core 增加统一 helper（例如 moveCardToDiscard）
  - 统一处理进入 discard 时的关键字段（Zone/Destroyed/Revealed 以及必须清理的 table-only 状态）
  - 当前“致命伤害离场”必须改为走这个 helper，不得再散写字段
- **Success Criteria**:
  - moveCardToDiscard 函数已实现
  - applyDerivedBoardSemantics 使用统一 helper
  - 致命伤害路径测试通过
- **Test Requirements**:
  - `programmatic` TR-1.1: moveCardToDiscard 已实现
  - `programmatic` TR-1.2: 旧回归不退化（attack/investigate/region scoring）

---

## [ ] Task 2: 固化 discard 基础语义
- **Priority**: P0
- **Depends On**: Task 1
- **Description**:
  - 进入 discard 后必须满足一致状态
  - 把语义写成可复用函数，不要在多处复制判断
- **Success Criteria**:
  - 有可复用函数判断 discard 状态
  - 状态一致性保证
- **Test Requirements**:
  - `programmatic` TR-2.1: 可复用函数已实现
  - `programmatic` TR-2.2: 状态一致性断言

---

## [ ] Task 3: 回归与护栏
- **Priority**: P0
- **Depends On**: Task 2
- **Description**:
  - 增加/更新测试覆盖：
    - 致命伤害 -> 进入 discard 的字段一致性
    - projection 对 discard 的可见性一致性
    - replay 后 discard 状态一致
    - invariant 对 discard 合法状态的约束
- **Success Criteria**:
  - 所有测试通过
  - 无旧语义回退
- **Test Requirements**:
  - `programmatic` TR-3.1: 致命伤害路径字段一致性
  - `programmatic` TR-3.2: projection discard 可见性
  - `programmatic` TR-3.3: replay 一致性
  - `programmatic` TR-3.4: invariant 约束

---

## [ ] Task 4: 文档同步
- **Priority**: P1
- **Depends On**: Task 3
- **Description**:
  - 更新：
    - `/Users/ddd/Downloads/UndergroundBattle/docs/NEXT_GEN_RULE_PLAN.md`
    - `/Users/ddd/Downloads/UndergroundBattle/docs/HANDOVER_TRAE_2026-04-01.md`
  - 明确标注这是 “Discard/Graveyard V1”，不是完整坟墓交互系统
- **Success Criteria**:
  - 文档已同步更新
- **Test Requirements**:
  - `human-judgement` TR-4.1: 文档更新内容准确
