# Attachments / Permanents Lifecycle 实施计划

## 1. 高层摘要 (TL;DR)

*   **目标：** 为 BQ022（合金指虎）等"附属"类型卡牌建立真实的生命周期模型
*   **现状：** 已有 Continuous Effects 系统，BQ022 可以正常工作，但缺少：
    - 明确的 Attachment 关系跟踪
    - 附属卡离开时的清理机制
    - 正式的 permanents 生命周期管理
*   **关键概念：**
    - **Attachment**：附属卡与被附属角色的关系
    - **Permanent**：永久存在的效果（直到源卡离开游戏）
    - **Lifecycle Hooks**：源卡进入/离开游戏时的钩子
*   **状态：** 计划制定中

---

## 2. 现状分析

### BQ022 的当前实现

**Fixture**（`shared/contracts/fixtures/bq022-alloy-knuckles.fixture.json`）：
```json
{
  "cardId": "BQ022",
  "card": {
    "name": "合金指虎",
    "basicType": "附属"
  },
  "input": {
    "logic": {
      "speed": "slow",
      "targetKinds": ["character"],
      "requiresStack": false,
      "durationKind": "permanent",
      "effects": [
        {
          "kind": "addKeyword",
          "targetRef": "selected",
          "keyword": "blackBlade"
        }
      ]
    }
  }
}
```

**当前工作方式**：
1. BQ022 直接结算（`requiresStack: false`）
2. 通过 `ContinuousEffect` 系统添加 "blackBlade" 关键词
3. `DurationKind: "permanent"` 表示永久生效

**缺失的功能**：
- ❌ 没有跟踪 BQ022 附属到了哪个角色
- ❌ 当 BQ022 离开游戏时，没有清理其 continuous effects
- ❌ 没有明确的 Attachment 关系数据结构

---

## 3. 设计方案

### 核心数据结构

```go
// Attachment represents an attachment relationship between two cards.
type Attachment struct {
    ID                string `json:"id"`                // Attachment ID
    SourceCardID      string `json:"sourceCardId"`      // 附属卡（如 BQ022）
    TargetCardID      string `json:"targetCardId"`      // 被附属角色
    CreatedAtRevision int    `json:"createdAtRevision"` // 创建时的版本号
}

// AttachmentRegistry tracks all active attachments.
type AttachmentRegistry struct {
    Active []Attachment `json:"active"`
}

// 在 BoardState 中添加
type BoardState struct {
    // ... 现有字段
    Attachments AttachmentRegistry `json:"attachments"`
}

// 在 ContinuousEffect 中添加 SourceAttachmentID（可选）
type ContinuousEffect struct {
    // ... 现有字段
    SourceAttachmentID *string `json:"sourceAttachmentId,omitempty"`
}
```

### Lifecycle Hooks

```go
// onCardEntersGame: 当卡牌进入游戏时调用
func onCardEntersGame(state GameState, card CardState) GameState

// onCardLeavesGame: 当卡牌离开游戏时调用
func onCardLeavesGame(state GameState, card CardState) GameState
```

---

## 4. 实施步骤

### 阶段 1：数据结构定义

#### 任务 1.1：在 types.go 中添加 Attachment 相关类型
- **行动**：添加 `Attachment`、`AttachmentRegistry` 结构体
- **行动**：在 `BoardState` 中添加 `Attachments` 字段
- **文档**：创建 `docs/trae_review/attachments_design.md`

### 阶段 2：Attachment 创建

#### 任务 2.1：在角色动作中创建 Attachment
- **行动**：修改 `role_actions.go`，处理"附属"类型卡牌
- **行动**：当 BQ022 等附属卡结算时，创建 Attachment
- **测试**：验证 Attachment 正确创建

### 阶段 3：Lifecycle Hooks

#### 任务 3.1：实现 onCardEntersGame
- **行动**：在卡牌进入游戏时调用
- **测试**：验证钩子被正确调用

#### 任务 3.2：实现 onCardLeavesGame
- **行动**：在卡牌离开游戏时调用
- **行动**：清理该卡牌创建的所有 Attachment
- **行动**：清理相关的 Continuous Effects
- **测试**：验证清理逻辑正确

### 阶段 4：BQ022 验证

#### 任务 4.1：验证 BQ022 完整流程
- **测试**：BQ022 附属到角色
- **测试**：被附属角色获得 blackBlade 关键词
- **测试**：BQ022 离开游戏时，blackBlade 关键词被移除

### 阶段 5：文档与最终验证

#### 任务 5.1：更新文档
- **行动**：更新 `docs/trae_review/attachments_design.md`
- **行动**：创建 `docs/trae_review/attachments_summary.md`

#### 任务 5.2：最终验证
- **行动**：运行所有测试
- **行动**：确保现有测试通过

---

## 5. 验收标准

- [ ] Attachment 数据结构定义完成
- [ ] BQ022 可以创建 Attachment
- [ ] onCardLeavesGame 正确清理 Attachment 和 Continuous Effects
- [ ] 所有现有测试继续通过
- [ ] 文档同步完整

---

## 6. 相关文档

- [BQ022 Alloy Knuckles Fixture](../../shared/contracts/fixtures/bq022-alloy-knuckles.fixture.json)
- [BQ022 Test Scenario](../server/pkg/rules/testdata/m0/08-alloy-knuckles-applies-permanent-keyword.json)
- [HANDOVER_TRAE_2026-04-01.md](../HANDOVER_TRAE_2026-04-01.md)
