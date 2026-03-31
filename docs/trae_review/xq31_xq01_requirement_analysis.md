# XQ31/XQ01 需求分析与预留设计文档

## 1. 高层摘要 (TL;DR)

*   **目标：** 分析 XQ31（莫兰大主教）和 XQ01（联会禁音使）的需求，在现有 prohibition 框架基础上预留扩展点，为后续支持这些卡牌铺路
*   **已完成：**
    *   ✨ 新增 **`TargetCondition`** 类型，支持关键词、区域、能力类型、本方/敌方区分
    *   ✨ 新增 **`SideKind`** 枚举（`SideAlly`/`SideEnemy`），支持本方/敌方区分
    *   ✨ 扩展 **`TargetCategory`**，添加 `Condition` 字段（预留扩展）
    *   ✅ 所有现有测试保持通过，确保向后兼容
*   **状态：** 框架已预留扩展点，等待后续完整实现

---

## 2. XQ31 需求分析

### 卡牌信息

| 属性 | 值 |
|------|-----|
| **卡牌ID** | `XQ31` |
| **名称** | 莫兰大主教 |
| **关键词** | 领袖、公开、声望 |
| **效果** | 持续：所有本方声望角色获得+1防御力，且不能成为敌方卡牌或能力的目标。 |

### 功能分解

#### 功能 1：所有本方声望角色获得+1防御力

| 需求 | 现有支持 | 状态 |
|------|----------|------|
| 连续效果系统 | ✅ 已有 `ContinuousEffectRegistry` | ⏳ 可接入 |
| 关键词匹配 | ✅ `CardState` 已有 `PrintedKeywords`/`EffectiveKeywords` | ✅ 预留扩展 |
| 本方/敌方区分 | ✅ 新增 `SideKind` 枚举 | ✅ 预留扩展 |
| 防御力修改 | ⏳ 需要具体机制 | ⏳ 待设计 |

#### 功能 2：不能成为敌方卡牌或能力的目标

| 需求 | 现有支持 | 状态 |
|------|----------|------|
| 目标合法性框架 | ❌ 暂无 | ⏳ 待设计 |
| 本方/敌方区分 | ✅ 新增 `SideKind` 枚举 | ✅ 预留扩展 |
| 关键词匹配（声望） | ✅ `TargetCondition.Keywords` | ✅ 预留扩展 |

---

## 3. XQ01 需求分析

### 卡牌信息

| 属性 | 值 |
|------|-----|
| **卡牌ID** | `XQ01` |
| **名称** | 联会禁音使 |
| **效果** | 持续：只要联会禁音使未横置，所有本地区的角色不能发动触发能力和行动能力。 |

### 功能分解

| 需求 | 现有支持 | 状态 |
|------|----------|------|
| 区域限制（本地区） | ⏳ `TargetCondition.RegionID` | ✅ 预留扩展 |
| 能力类型限制 | ⏳ `TargetCondition.AbilityKinds` | ✅ 预留扩展 |
| 行动能力禁止 | ⏳ 待设计 | ⏳ 待设计 |
| 触发能力禁止 | ⏳ 待设计 | ⏳ 待设计 |

---

## 4. 已完成的预留设计

### 新增类型定义（`types.go`）

#### SideKind 枚举

```go
// SideKind defines whether a target is considered ally or enemy.
type SideKind string

const (
	// SideAlly means the target is an ally (same controller as source).
	SideAlly SideKind = "ally"
	// SideEnemy means the target is an enemy (different controller from source).
	SideEnemy SideKind = "enemy"
)
```

#### TargetCondition 类型

```go
// TargetCondition defines additional conditions on the target being acted upon.
// This is a reserved extension point for future use (e.g., XQ31/XQ01).
type TargetCondition struct {
	// Keywords defines required keywords on the target (reserved for XQ31: "声望")
	Keywords []string `json:"keywords,omitempty"`

	// RegionID defines the region scope (reserved for XQ01: "本地区")
	RegionID string `json:"regionId,omitempty"`

	// AbilityKinds defines which ability kinds are affected (reserved for XQ01: "触发能力", "行动能力")
	AbilityKinds []string `json:"abilityKinds,omitempty"`

	// Side defines whether the target must be ally or enemy (reserved for XQ31: "本方", "敌方")
	Side SideKind `json:"side,omitempty"`
}
```

#### TargetCategory 扩展

```go
// TargetCategory defines what kinds of targets are prohibited.
type TargetCategory struct {
	BasicTypes  []string         `json:"basicTypes,omitempty"`  // Prohibited card basic types (e.g., "事务")
	ActionKinds []ActionKind     `json:"actionKinds,omitempty"` // Prohibited action kinds
	Condition   *TargetCondition `json:"condition,omitempty"`   // Additional target conditions (reserved extension)
}
```

---

## 5. 未来实现建议

### 实现路线图

| 阶段 | 任务 | 优先级 | 说明 |
|------|------|--------|------|
| **阶段 1** | 设计目标合法性框架 | 🔴 高 | 为 XQ31 的"不能成为目标"需求设计 |
| **阶段 2** | 实现 TargetCondition 匹配 | 🟡 中 | 关键词、区域、能力类型、本方/敌方 |
| **阶段 3** | 接入 XQ31 的连续效果 | 🟡 中 | 声望角色 +1 防御 |
| **阶段 4** | 接入 XQ01 的能力禁止 | 🟡 中 | 行动能力/触发能力禁止 |

### 关键设计决策

| 决策 | 选项 | 推荐 | 理由 |
|------|------|--------|------|
| 目标合法性位置 | prohibition 框架 / 独立框架 | 独立框架 | "不能成为目标"与"不能打出卡牌"是不同概念 |
| 能力类型 | 字符串枚举 / 强类型 | 字符串枚举 | 灵活性高，便于快速迭代 |
| 区域模型 | 字符串 ID / 结构化 | 字符串 ID | 预留扩展，不引入大系统 |

---

## 6. 验证结果

### ✅ 向后兼容

| 测试类别 | 结果 |
|----------|------|
| 原有 XQ22 测试 | ✅ 7/7 通过 |
| 原有 prohibition 测试 | ✅ 7/7 通过 |
| 多卡验证测试 | ✅ 2/2 通过 |
| 所有规则测试 | ✅ 全部通过 |

### ✅ 框架可扩展

预留的扩展点：
- `TargetCondition.Keywords` - 为 XQ31 的"声望"关键词准备
- `TargetCondition.RegionID` - 为 XQ01 的"本地区"准备
- `TargetCondition.AbilityKinds` - 为 XQ01 的"触发能力/行动能力"准备
- `TargetCondition.Side` - 为 XQ31 的"本方/敌方"准备

---

## 7. 相关文档

- [重构禁止逻辑为规则引擎.md](./重构禁止逻辑为规则引擎.md) - 框架基础架构
- [prohibition_framework_multi_card_validation.md](./prohibition_framework_multi_card_validation.md) - 多卡验证文档
- [HANDOVER_TRAE_2026-04-01.md](../HANDOVER_TRAE_2026-04-01.md) - 项目交接文档
