# XQ01 地区作用域沉默前置设计文档

## 1. 高层摘要 (TL;DR)

**目标**：分析 XQ01（联会禁音使）的需求，设计并实现必要的前置框架，为后续完整实施 XQ01 铺路。

**当前状态**：
- ✅ `TargetCondition.RegionID` - 已预留扩展点
- ✅ `TargetCondition.AbilityKinds` - 已预留扩展点
- ⏳ "地区"概念 - 需要明确设计
- ⏳ "行动能力/触发能力"概念 - 需要明确设计
- ⏳ 能力禁止与 permission/prohibition 框架的集成 - 需要设计

---

## 2. XQ01 需求分析

### 卡牌信息

| 属性 | 值 |
|------|-----|
| **卡牌ID** | `XQ01` |
| **名称** | 联会禁音使 |
| **效果** | 持续：只要联会禁音使未横置，所有本地区的角色不能发动触发能力和行动能力。 |

### 功能分解

| 需求 | 现有支持 | 状态 |
|------|----------|------|
| 源卡条件（XQ01 在场且未横置） | ✅ 已有 CardCondition | ✅ 已支持 |
| 地区限制（本地区） | ⏳ `TargetCondition.RegionID` 已预留 | ⏳ 待设计 |
| 能力类型限制（触发能力、行动能力） | ⏳ `TargetCondition.AbilityKinds` 已预留 | ⏳ 待设计 |
| 行动能力禁止 | ⏳ 需要与 permission 框架集成 | ⏳ 待设计 |
| 触发能力禁止 | ⏳ 需要明确触发能力模型 | ⏳ 待设计 |

---

## 3. 需要明确的概念设计

### 3.1 "地区"概念设计

**问题**：什么是"本地区"？

**设计选项**：

| 选项 | 说明 | 优点 | 缺点 |
|------|------|------|------|
| **A. 区域牌（Region）** | 场上的区域牌作为地区边界 | 符合卡牌游戏直觉 | 需要完整的区域模型 |
| **B. 控制器分组** | 按控制器（ControllerID）分组 | 实现简单 | 不符合"地区"语义 |
| **C. 预留占位** | 当前先按全桌处理，预留 RegionID | 不阻塞 XQ01 实施 | 语义不完整 |

**推荐**：选项 C（预留占位），因为：
1. 不阻塞 XQ01 的核心功能实施
2. 可以后续完善地区模型
3. 当前可以先实现"全桌"效果

---

### 3.2 "能力类型"概念设计

**问题**：什么是"行动能力"和"触发能力"？

**当前 ActionKind**：
```go
const (
    ActionKindAdvancePhase         // 推进阶段
    ActionKindRevealCard           // 揭示卡牌
    ActionKindInspectCard          // 检视卡牌
    ActionKindPassPriority         // 传递优先权
    ActionKindQueueOperation       // 队列操作（打出卡牌）
    ActionKindDeclareAttack        // 宣告攻击
    ActionKindDeclareInvestigation // 宣告调查
    ActionKindResolveTopStack      // 解析堆叠
    ActionKindRollSeededRandom     // 掷随机数
)
```

**设计选项**：

| 能力类型 | 对应 ActionKind | 说明 |
|---------|----------------|------|
| **行动能力** | `DeclareAttack`、`DeclareInvestigation` | 角色主动发动的能力 |
| **触发能力** | （待定） | 自动触发的能力（当前模型中暂无） |

**推荐**：
1. 当前先将"行动能力"映射到 `DeclareAttack` 和 `DeclareInvestigation`
2. "触发能力"可以后续再设计完整模型

---

### 3.3 能力禁止与 permission 框架的集成

**问题**：如何用现有 permission/prohibition 框架禁止能力？

**当前权限系统**：
```go
// CardState 已有：
Permissions   []string // 授予的权限
Prohibitions  []string // 禁止的权限

// ContinuousEffect 已有：
EffectKind: "grantPermission"  // 授予权限
EffectKind: "prohibitPermission" // 禁止权限
Permission: "inspect"            // 权限名称
```

**设计选项**：

| 选项 | 说明 |
|------|------|
| **A. 新增 permission 名称** | 新增 "declare_attack"、"declare_investigation" 等权限 |
| **B. 新增 effect kind** | 新增 "prohibitAbility" 效果类型 |

**推荐**：选项 A（新增 permission 名称），因为：
1. 复用现有的 permission/prohibition 框架
2. 保持架构一致性
3. 实现简单直接

---

## 4. 前置实施步骤

### 阶段 1：明确概念并预留

| 步骤 | 任务 | 优先级 |
|------|------|--------|
| 1.1 | 明确"地区"当前为全桌，预留 RegionID | 🟡 中 |
| 1.2 | 明确"行动能力"为 `DeclareAttack`/`DeclareInvestigation` | 🟡 中 |
| 1.3 | 明确"触发能力"为后续扩展 | 🟢 低 |

### 阶段 2：扩展 permission 系统

| 步骤 | 任务 | 优先级 |
|------|------|--------|
| 2.1 | 在 legality check 中集成权限检查到角色动作 | 🔴 高 |
| 2.2 | 添加 "declare_attack" 权限支持 | 🔴 高 |
| 2.3 | 添加 "declare_investigation" 权限支持 | 🔴 高 |

### 阶段 3：XQ01 规则接入

| 步骤 | 任务 | 优先级 |
|------|------|--------|
| 3.1 | 在 `legality_catalog.go` 中添加 XQ01 规则 | 🔴 高 |
| 3.2 | 实现 XQ01 的 continuous effect 模板 | 🔴 高 |
| 3.3 | 添加 XQ01 的完整测试和 Golden Scenario | 🔴 高 |

---

## 5. 关键设计决策

| 决策 | 选项 | 推荐 | 理由 |
|------|------|--------|------|
| 地区实现 | RegionID 预留 / 全桌 | 全桌 | 不阻塞核心功能 |
| 能力类型 | 新增 permission / 新增 effect kind | 新增 permission | 复用现有框架 |
| 行动能力映射 | Attack/Investigation / 其他 | Attack/Investigation | 符合当前模型 |

---

## 6. 向后兼容

- ✅ 所有现有测试保持通过
- ✅ 不破坏现有 permission/prohibition 框架
- ✅ 不破坏现有 ContinuousEffect 系统
- ✅ RegionID 和 AbilityKinds 作为预留字段保持兼容

---

## 7. 相关文档

- [XQ31/XQ01 需求分析与预留设计](./xq31_xq01_requirement_analysis.md)
- [HANDOVER_TRAE_2026-04-01](../HANDOVER_TRAE_2026-04-01.md)
- [NEXT_GEN_RULE_PLAN](../NEXT_GEN_RULE_PLAN.md)
