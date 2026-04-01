# Attachment / Host Lifecycle V1 - 实施计划

## 概述

**目标**：把附属从 tracking V0 推进到真实生命周期：附属在场、attachedTo、宿主离场联动、附属离场联动。

**记忆约束严格执行**：
- ✅ TDD：先写测试，再实现
- ✅ 形式化思维：准确定义问题，严格建模
- ✅ 数字卡牌游戏严格要求：容不得一丝宽松懈怠

**范围外（这轮不要做）**：
- ❌ 不做“回手/回收”复杂牌面语义
- ❌ 不做完整 attachment stack 互动

---

## 形式化问题定义

### Attachment 宿主关系公理

**宿主-附属关系公理**：
- Forall 附属 a ∈ Attachments.Active,
  a.TargetCardID 定义附属的宿主卡片
- Forall 宿主卡片 h ∈ Board.Cards,
  if h.Zone != CardZoneTable || h.Destroyed:
    所有指向 h 的附属 a 必须按规则离场（最小默认：附属进 discard）

**附属离场公理**：
- Forall 附属 a ∈ Attachments.Active,
  when a 离场：
    1. a 从 Attachments.Active 中移除
    2. 由 a 提供的 continuous effect 同步失效

---

## [ ] Task 0: 先写测试 - Attachment 基础测试（TDD，红测优先）
- **Priority**: P0
- **Depends On**: None
- **Description**:
  - 在 `attachment_test.go` 中先写红测
  - 测试场景：
    1. 宿主离场时：附属按规则离场（进 discard）
    2. 附属离场时：由附属提供的 continuous effect 同步失效
    3. 现有 Attachment tracking V0 不退化
- **Success Criteria**:
  - 测试文件已添加，测试用例完整覆盖形式化问题
  - 测试运行结果为红（先红后绿，TDD 流程）
- **Test Requirements**:
  - `programmatic` TR-0.1: 测试文件包含完整测试场景
  - `programmatic` TR-0.2: 测试断言严格按照公理编写
  - `programmatic` TR-0.3: 测试运行失败（红测状态）

---

## [ ] Task 1: 给 Attachment 添加 attachedToCardID
- **Priority**: P0
- **Depends On**: Task 0
- **Description**:
  - 在 `Attachment` 结构体中添加 `HostCardID` 字段（attachedTo）
  - 在 `AttachmentBuilder` 中支持设置 HostCardID
  - 保持向后兼容
- **Success Criteria**:
  - Attachment 有 HostCardID 字段
  - AttachmentBuilder 支持设置 HostCardID
  - 现有测试通过
- **Test Requirements**:
  - `programmatic` TR-1.1: HostCardID 已添加
  - `programmatic` TR-1.2: 所有现有 Go 测试通过

---

## [ ] Task 2: 宿主离场联动 - 宿主离场时附属离场
- **Priority**: P0
- **Depends On**: Task 1
- **Description**:
  - 在 `pruneExpiredAttachments` 中增加检查：宿主不在场时附属自动移除
  - 当宿主卡片离场时，所有指向它的附属按规则离场
  - 最小默认规则：附属进 discard（如果附属本身也是卡片的话，先只处理关系清理）
- **Success Criteria**:
  - 宿主离场时，所有附属自动从 Active 中移除
  - 现有测试通过
- **Test Requirements**:
  - `programmatic` TR-2.1: 宿主离场联动已实现
  - `programmatic` TR-2.2: 所有现有 Go 测试通过

---

## [ ] Task 3: 附属离场联动 - 附属离场时 effect 失效
- **Priority**: P0
- **Depends On**: Task 2
- **Description**:
  - 确保当附属离场时，由该附属提供的 continuous effect 同步失效
  - 利用现有 continuous source validity 机制
  - 不新开大重构
- **Success Criteria**:
  - 附属离场时，相关 continuous effect 自动失效
  - 现有测试通过
- **Test Requirements**:
  - `programmatic` TR-3.1: 附属离场联动已实现
  - `programmatic` TR-3.2: 所有现有 Go 测试通过

---

## [ ] Task 4: 回归与护栏
- **Priority**: P0
- **Depends On**: Task 3
- **Description**:
  - 运行所有测试，确保不回归
  - 确保 replay / projection / invariant 保持一致
- **Success Criteria**:
  - 所有测试通过
  - 无旧语义回退
- **Test Requirements**:
  - `programmatic` TR-4.1: 所有 Go 测试通过
  - `programmatic` TR-4.2: 所有 fixture-tools 测试通过
  - `programmatic` TR-4.3: 所有 web 测试通过

---

## [ ] Task 5: 文档同步
- **Priority**: P1
- **Depends On**: Task 4
- **Description**:
  - 更新：
    - `/Users/ddd/Downloads/UndergroundBattle/docs/NEXT_GEN_RULE_PLAN.md`
    - `/Users/ddd/Downloads/UndergroundBattle/docs/HANDOVER_TRAE_2026-04-01.md`
  - 明确标注这是 “Attachment / Host Lifecycle V1”，不是完整 attachment system
- **Success Criteria**:
  - 文档已同步更新
- **Test Requirements**:
  - `human-judgement` TR-5.1: 文档更新内容准确
