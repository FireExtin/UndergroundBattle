# Prohibition Framework 多卡验证文档

## 1. 高层摘要 (TL;DR)

*   **目标：** 验证 prohibition 框架可以支持多张 prohibition 卡和不同的作用范围，确保框架的可扩展性和正确性
*   **方法：** 按照红-绿-红开发流程（Red-Green-Red）
*   **关键验证：**
    *   ✨ 新增 **TEST01** 测试卡（禁止角色卡），验证多卡同时生效
    *   ✨ 新增 **TEST02** 测试卡（仅禁止对手的事务卡），验证不同作用范围
    *   🧪 新增 **2 个单元测试**，覆盖多卡叠加和不同作用范围场景
    *   ✅ 所有现有 XQ22 测试保持通过，确保向后兼容

---

## 2. 红-绿-红开发流程 (Red-Green-Red)

### 第一步：红测 (Red) - 写失败的测试用例

**目标：** 在没有实现功能的情况下，先写测试用例，验证测试会失败。

**新增测试文件：** `server/pkg/rules/prohibition_multicard_test.go`

| 测试用例 | 验证内容 |
|----------|----------|
| `TestMultiCardProhibition` | XQ22（禁事务）和 TEST01（禁角色）可以同时生效 |
| `TestMultiCardProhibitionDifferentScopes` | TEST02 仅禁止对手，不禁止控制者本人 |

**红测结果：** ✅ 测试失败，符合预期（TEST01 和 TEST02 规则还未添加）

---

### 第二步：实现 (Green) - 添加功能代码

**目标：** 添加 TEST01 和 TEST02 的 ProhibitionRule 到框架中。

**修改文件：** `server/pkg/rules/prohibition.go`

#### TEST01ProhibitionRule

| 属性 | 值 |
|------|-----|
| 来源卡 | `TEST01`（测试禁角色卡） |
| 激活条件 | 在场 (`CardZoneTable`) + 就绪 (`Ready`) + 未被摧毁 (`NotDestroyed`) |
| 作用域 | `all_players`（所有玩家） |
| 禁止目标 | `BasicTypes: ["角色"]`（角色卡） |

#### TEST02ProhibitionRule

| 属性 | 值 |
|------|-----|
| 来源卡 | `TEST02`（测试仅对手禁用卡） |
| 激活条件 | 在场 (`CardZoneTable`) + 就绪 (`Ready`) + 未被摧毁 (`NotDestroyed`) |
| 作用域 | `opponents_only`（仅对手） |
| 禁止目标 | `BasicTypes: ["事务"]`（事务卡） |

#### BuildProhibitionChecker 更新

将 TEST01 和 TEST02 规则添加到规则列表中：

```go
func BuildProhibitionChecker(state GameState) *ScopedProhibitionChecker {
	rules := []ProhibitionRule{
		XQ22ProhibitionRule,
		TEST01ProhibitionRule,  // 新增
		TEST02ProhibitionRule,  // 新增
	}
	return NewScopedProhibitionChecker(rules)
}
```

---

### 第三步：绿测 (Red-Green) - 运行测试并验证通过

**目标：** 运行所有测试，确保新功能正常工作且不破坏现有功能。

**绿测结果：** ✅ 所有测试通过！

| 测试类别 | 结果 |
|----------|------|
| 新增多卡测试 | ✅ 2/2 通过 |
| 原有 prohibition 测试 | ✅ 7/7 通过 |
| 原有 XQ22 测试 | ✅ 7/7 通过 |
| 所有规则测试 | ✅ 全部通过 |
| 整个项目测试 | ✅ 全部通过 |

---

## 3. 验证结论

### ✅ Prohibition 框架有效！

框架已验证可以支持：

| 功能 | 验证状态 | 说明 |
|------|----------|------|
| **多卡同时生效** | ✅ 验证通过 | XQ22 和 TEST01 可以同时在场上生效，各自禁止不同类型的卡牌 |
| **不同作用范围** | ✅ 验证通过 | `AllPlayers`（影响所有人）和 `OpponentsOnly`（仅影响对手）都能正常工作 |
| **向后兼容** | ✅ 验证通过 | 所有原有的 XQ22 测试继续通过，行为无变化 |
| **可扩展性** | ✅ 验证通过 | 新增 prohibition 卡只需添加新的 `ProhibitionRule` 常量，无需修改引擎逻辑 |

---

## 4. 架构设计验证

### 数据结构设计

```
ProhibitionRule
├── SourceDefinitionID:  "XQ22" | "TEST01" | "TEST02"
├── SourceCondition
│   ├── Zone:           CardZoneTable
│   ├── Ready:          true
│   └── NotDestroyed:   true
├── Scope
│   └── Kind:           AllPlayers | OpponentsOnly | ControllerOnly
└── TargetCategory
    └── BasicTypes:     ["事务"] | ["角色"]
```

### 匹配流水线

```
Check(state, actorID, targetCategory)
    ↓
遍历所有 ProhibitionRule
    ↓
matchesSourceCondition(card, rule)
    ├─ card.DefinitionID == rule.SourceDefinitionID
    ├─ card.Zone == rule.SourceCondition.Zone
    ├─ card.Exhausted != rule.SourceCondition.Ready
    └─ card.Destroyed != rule.SourceCondition.NotDestroyed
    ↓
matchesScope(state, card, actorID, rule.Scope)
    ├─ AllPlayers: return true
    ├─ OpponentsOnly: return actorID != card.ControllerID
    └─ ControllerOnly: return actorID == card.ControllerID
    ↓
matchesTargetCategory(actual, rule.TargetCategory)
    └─ BasicTypes 交叉匹配
    ↓
返回 ProhibitionMatchResult
```

---

## 5. 使用示例

### 示例 1：添加新的 Prohibition 卡

假设要添加一张新卡 "TEST03"，禁止"附属"卡：

```go
// 在 prohibition.go 中添加
var TEST03ProhibitionRule = ProhibitionRule{
	SourceDefinitionID: "TEST03",
	SourceCondition: CardCondition{
		Zone:         CardZoneTable,
		Ready:        true,
		NotDestroyed: true,
	},
	Scope: ProhibitionScope{
		Kind: ProhibitionScopeAllPlayers,
	},
	TargetCategory: TargetCategory{
		BasicTypes: []string{"附属"},
	},
	Description: "TEST03: Test card that prohibits attachment cards",
}

// 在 BuildProhibitionChecker 中添加
func BuildProhibitionChecker(state GameState) *ScopedProhibitionChecker {
	rules := []ProhibitionRule{
		XQ22ProhibitionRule,
		TEST01ProhibitionRule,
		TEST02ProhibitionRule,
		TEST03ProhibitionRule,  // 新增
	}
	return NewScopedProhibitionChecker(rules)
}
```

### 示例 2：使用 OpponentsOnly 作用域

```go
var MYCARDProhibitionRule = ProhibitionRule{
	SourceDefinitionID: "MYCARD",
	SourceCondition: CardCondition{
		Zone:         CardZoneTable,
		Ready:        true,
		NotDestroyed: true,
	},
	Scope: ProhibitionScope{
		Kind: ProhibitionScopeOpponentsOnly,  // 仅禁止对手
	},
	TargetCategory: TargetCategory{
		BasicTypes: []string{"事务"},
	},
	Description: "MYCARD: Only opponents can't play event cards",
}
```

---

## 6. 下一步建议

根据 HANDOVER_TRAE_2026-04-01.md，框架已经为以下卡牌做好了准备：

| 卡牌 | 所需功能 | 当前状态 |
|------|----------|----------|
| XQ31（莫兰大主教） | 目标合法性、声望角色、本方/敌方区分 | ✅ 框架已支持 ControllerOnly/OpponentsOnly |
| XQ01（联会禁音使） | 区域限制、行动能力/触发能力 | ⏳ 需要更多系统支持（区域、能力类型） |

**建议优先方向：**
1. 当需要支持 XQ31 时，可以直接在现有框架基础上扩展
2. 当需要支持 XQ01 时，需要先实现区域（Region）和能力类型（ActionKind/TriggerKind）的匹配逻辑

---

## 7. 相关文档

- [重构禁止逻辑为规则引擎.md](./重构禁止逻辑为规则引擎.md) - 框架基础架构文档
- [HANDOVER_TRAE_2026-04-01.md](../HANDOVER_TRAE_2026-04-01.md) - 项目交接文档
