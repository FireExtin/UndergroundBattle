# Attachments 设计文档

## 1. 高层摘要 (TL;DR)

*   **目标：** 为"附属"类型卡牌建立真实的生命周期模型
*   **核心概念：**
    - **Attachment**：附属卡与被附属角色的关系
    - **Permanent**：永久存在的效果（直到源卡离开游戏）
    - **Pruning**：在连续效果重新计算时清理过期的 Attachment
*   **关键卡牌：** BQ022（合金指虎）
*   **状态：** 已实施完成

---

## 2. 核心数据结构

### Attachment

```go
// Attachment represents an attachment relationship between two cards.
type Attachment struct {
	ID                string `json:"id"`                // Attachment ID
	SourceCardID      string `json:"sourceCardId"`      // 附属卡（如 BQ022）
	TargetCardID      string `json:"targetCardId"`      // 被附属角色
	CreatedAtRevision int    `json:"createdAtRevision"` // 创建时的版本号
}
```

### AttachmentRegistry

```go
// AttachmentRegistry tracks all active attachments.
type AttachmentRegistry struct {
	Active           []Attachment `json:"active"`
	NextAttachmentID int          `json:"nextAttachmentId"`
}
```

### BoardState 更新

```go
type BoardState struct {
	// ... 现有字段
	Attachments AttachmentRegistry `json:"attachments"`
}
```

### ContinuousEffect 更新（可选）

```go
type ContinuousEffect struct {
	// ... 现有字段
	SourceAttachmentID *string `json:"sourceAttachmentId,omitempty"`
}
```

---

## 3. Pruning 清理机制

我们采用了 pruning 方式，在 `RecalculateContinuousEffects` 函数中自动清理过期的 Attachment，而不是使用显式的 lifecycle hooks。

### pruneExpiredAttachments

```go
func pruneExpiredAttachments(state *GameState) {
	registry := &state.Board.Attachments
	if len(registry.Active) == 0 {
		return
	}

	kept := make([]Attachment, 0, len(registry.Active))
	removed := false
	for _, attachment := range registry.Active {
		if !attachmentIsStillActive(*state, attachment) {
			removed = true
			continue
		}

		kept = append(kept, attachment)
	}

	if removed {
		registry.Active = kept
	}
}
```

### attachmentIsStillActive

```go
func attachmentIsStillActive(state GameState, attachment Attachment) bool {
	sourceIndex := findCardIndex(state, attachment.SourceCardID)
	if sourceIndex == -1 {
		return false
	}
	source := state.Board.Cards[sourceIndex]
	if source.Zone != CardZoneTable || source.Destroyed {
		return false
	}

	targetIndex := findCardIndex(state, attachment.TargetCardID)
	if targetIndex == -1 {
		return false
	}
	target := state.Board.Cards[targetIndex]
	if target.Zone != CardZoneTable || target.Destroyed {
		return false
	}

	return true
}
```

### 集成到 RecalculateContinuousEffects

```go
func RecalculateContinuousEffects(state GameState) GameState {
	working := cloneGameState(state)
	// ... 初始化代码

	resetDerivedCardState(&working)
	pruneExpiredAttachments(&working)  // 新增：清理过期的 Attachment
	pruneExpiredContinuousEffects(&working)

	// ... 其余代码
}
```

---

## 4. 创建 Attachment

### 使用 AttachmentBuilder（推荐方式）

我们使用 Builder 模式来创建 Attachment，提供流畅的 API 和清晰的错误处理：

```go
// Create attachment if this is an attachment card (basicType = "附属")
// Note: If attachment creation fails (e.g., target destroyed or moved), the error is
// intentionally ignored to allow the continuous effect to still be created. This is
// a design choice - the attachment relationship is optional metadata, while the
// continuous effect is the primary game mechanic.
if operation.Source.BasicType == "附属" {
	newState, err := NewAttachment(working).
		From(operation.CardID).
		To(targetCardID).
		AtRevision(working.Revision.Number).
		WithBasicType(operation.Source.BasicType).
		Create()
	if err == nil {
		working = newState
	}
	// If err != nil, attachment creation failed silently (target invalid/destroyed).
	// This is acceptable - the continuous effect still applies, we just don't track
	// the attachment relationship for pruning purposes.
}
```

### AttachmentBuilder API

```go
// AttachmentBuilder provides a fluent interface for creating attachments.
type AttachmentBuilder struct {
	state      GameState
	sourceID   string
	targetID   string
	revision   int
	basicType  string
}

// 流畅的链式调用 API
newState, err := NewAttachment(state).
    From(sourceID).
    To(targetID).
    AtRevision(revision).
    WithBasicType("附属").
    Create()
```

### 错误处理设计

**重要设计决策**：Attachment 创建失败时**静默忽略**错误。

**原因**：
1. **Attachment 是可选元数据** - 它只用于追踪附属关系以便清理
2. **ContinuousEffect 是主要游戏机制** - 即使 Attachment 创建失败，效果仍然应该应用
3. **失败场景是合法的** - 目标卡牌可能在效果结算过程中被摧毁或移出桌面

**测试验证**：
```go
func TestAttachmentCreationFailureStillCreatesContinuousEffect(t *testing.T) {
    // 即使目标被摧毁，continuous effect 仍然应该被创建
    // 只是 attachment 不会被创建
}
```

---

## 5. 架构改进

### 5.1 文件组织

我们将 Attachment 相关代码从 `continuous.go` 中分离出来，创建了独立的 `attachment.go` 文件：

```
server/pkg/rules/
├── attachment.go       # 新增：Attachment 核心逻辑
├── attachment_test.go  # 新增：Attachment 单元测试
├── continuous.go       # 重构：使用 AttachmentBuilder
└── types.go            # Attachment 数据结构定义
```

### 5.2 AttachmentManager

提供高级管理操作：

```go
type AttachmentManager struct {
    state GameState
}

func (am *AttachmentManager) PruneExpired() GameState
func (am *AttachmentManager) GetAttachmentsForTarget(targetID string) []Attachment
func (am *AttachmentManager) GetAttachmentsFromSource(sourceID string) []Attachment
```

---

## 6. 实施完成

已完成的工作：

1. ✅ 在 `types.go` 中添加数据结构（Attachment、AttachmentRegistry）
2. ✅ 创建 `attachment.go` 实现 AttachmentBuilder 和 AttachmentManager
3. ✅ 重构 `continuous.go` 使用新的 AttachmentBuilder
4. ✅ 实现 `pruneExpiredAttachments()` 清理逻辑
5. ✅ 添加完整的单元测试（包括错误处理场景）
6. ✅ 验证 BQ022 完整流程（所有测试通过）

### 修改的文件：

- `server/pkg/rules/types.go` - 添加 Attachment 相关类型
- `server/pkg/rules/engine.go` - 初始化 Attachments
- `server/pkg/rules/continuous.go` - 使用 AttachmentBuilder 创建 Attachment
- `server/pkg/rules/attachment.go` - **新增**：AttachmentBuilder 和 AttachmentManager
- `server/pkg/rules/attachment_test.go` - **新增**：单元测试

### 测试覆盖：

- `TestAttachmentBuilder_Create_Success` - 正常创建
- `TestAttachmentBuilder_Create_InvalidBasicType` - 无效类型
- `TestAttachmentBuilder_Create_SourceNotOnTable` - 源卡不在桌面
- `TestAttachmentBuilder_Create_TargetDestroyed` - 目标被摧毁
- `TestAttachmentBuilder_CanCreate` - 预检查
- `TestAttachmentManager_PruneExpired` - 清理过期
- `TestAttachmentManager_GetAttachmentsForTarget` - 查询目标
- `TestAttachmentManager_GetAttachmentsFromSource` - 查询源
- `TestAttachmentCreationFailureStillCreatesContinuousEffect` - **错误处理验证**

### 验证结果：

- 所有测试通过，包括 BQ022（合金指虎）的黄金场景测试
- 构建成功
- 代码复杂度降低，可读性提升
