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

### 在 registerContinuousEffect 中

当附属卡（basicType = "附属"）被使用时，在创建 ContinuousEffect 的同时创建 Attachment：

```go
func registerContinuousEffect(state GameState, operation Operation, effect EffectSpec) GameState {
	// ... 现有代码

	// Create attachment if this is an attachment card (basicType = "附属")
	if operation.Source.BasicType == "附属" {
		attachmentRegistry := &working.Board.Attachments
		attachmentRegistry.NextAttachmentID++
		attachment := Attachment{
			ID:                fmt.Sprintf("att:%d", attachmentRegistry.NextAttachmentID),
			SourceCardID:      operation.CardID,
			TargetCardID:      targetCardID,
			CreatedAtRevision: working.Revision.Number,
		}
		attachmentRegistry.Active = append(attachmentRegistry.Active, attachment)
	}

	requestContinuousRecalculation(&working)
	return working
}
```

---

## 5. 实施完成

已完成的工作：

1. ✅ 在 `types.go` 中添加数据结构（Attachment、AttachmentRegistry）
2. ✅ 在 `registerContinuousEffect()` 中集成 Attachment 创建
3. ✅ 实现 `pruneExpiredAttachments()` 清理逻辑，使用 pruning 方式
4. ✅ 验证 BQ022 完整流程（所有测试通过）

### 修改的文件：

- `server/pkg/rules/types.go` - 添加 Attachment 相关类型
- `server/pkg/rules/engine.go` - 初始化 Attachments
- `server/pkg/rules/continuous.go` - 创建和清理 Attachment

### 验证结果：

- 所有测试通过，包括 BQ022（合金指虎）的黄金场景测试
- 构建成功
