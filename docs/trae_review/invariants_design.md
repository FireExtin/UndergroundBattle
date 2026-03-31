# Invariants 设计文档

## 1. 高层摘要 (TL;DR)

*   **目标：** 定义并实现游戏状态的不变量（Invariants），确保系统健康
*   **不变量列表：**
    1.  卡牌 ID 唯一性
    2.  卡牌区域合法性
    3.  优先权玩家存在性
    4.  堆栈深度非负
    5.  版本号递增一致性
*   **状态：** 设计完成，准备实现

---

## 2. 核心数据结构分析

### GameState 结构

```go
type GameState struct {
    GameID   string       // 游戏唯一标识
    Players  []string     // 玩家列表
    Revision Revision     // 版本信息
    Match    MatchState   // 比赛状态
    Turn     TurnState    // 回合状态
    Board    BoardState   // 场上状态
    Score    ScoreState   // 分数状态
    History  HistoryState // 历史记录
    RNG      RNGState     // 随机数状态
}
```

### CardState 结构

```go
type CardState struct {
    CardID              string           // 卡牌实例 ID（唯一）
    DefinitionID        string           // 卡牌定义 ID
    Name                string           // 卡牌名称
    Kind                CardKind         // 卡牌类型（角色/地区/其他）
    OwnerID             string           // 拥有者
    Zone                CardZone         // 所在区域
    Revealed            bool             // 是否揭示
    Exhausted           bool             // 是否横置
    Destroyed           bool             // 是否销毁
    ControllerID        string           // 控制者
    PrintedKeywords     []string         // 印刷关键词
    EffectiveKeywords   []string         // 有效关键词
    // ... 其他字段
}
```

### BoardState 结构

```go
type BoardState struct {
    Stack         []Operation              // 堆栈
    Resolved      []Operation              // 已结算
    Cards         []CardState              // 场上卡牌
    Continuous    ContinuousEffectRegistry // 连续效果
}
```

---

## 3. 不变量定义

### 不变量 1：卡牌 ID 唯一性 (InvariantCardIDUnique)

**描述：** 场上所有卡牌的 CardID 必须唯一

**检查逻辑：**
```go
func InvariantCardIDUnique(state GameState) bool {
    seen := make(map[string]bool)
    for _, card := range state.Board.Cards {
        if seen[card.CardID] {
            return false
        }
        seen[card.CardID] = true
    }
    return true
}
```

**违反后果：** 无法唯一标识卡牌，导致目标选择错误

**触发时机：** 每次状态变更后

---

### 不变量 2：卡牌区域合法性 (InvariantCardZoneValid)

**描述：** 所有卡牌的 Zone 必须是预定义的合法值

**合法区域：**
- `hand` - 手牌
- `table` - 场上
- `discard` - 弃牌堆
- `deck` - 牌库
- `removed` - 移除区

**检查逻辑：**
```go
func InvariantCardZoneValid(state GameState) bool {
    validZones := map[CardZone]bool{
        CardZoneHand:    true,
        CardZoneTable:   true,
        CardZoneDiscard: true,
        CardZoneDeck:    true,
        CardZoneRemoved: true,
    }
    for _, card := range state.Board.Cards {
        if !validZones[card.Zone] {
            return false
        }
    }
    return true
}
```

**违反后果：** 卡牌位置不明确，可能导致游戏逻辑错误

**触发时机：** 每次卡牌区域变更后

---

### 不变量 3：优先权玩家存在性 (InvariantPriorityPlayerValid)

**描述：** 当前优先权玩家必须在 Players 列表中

**检查逻辑：**
```go
func InvariantPriorityPlayerValid(state GameState) bool {
    priorityPlayer := state.Turn.Priority.CurrentPlayerID
    if priorityPlayer == "" {
        return true // 空值表示无优先权（可能游戏未开始）
    }
    for _, player := range state.Players {
        if player == priorityPlayer {
            return true
        }
    }
    return false
}
```

**违反后果：** 无法确定谁有优先权，导致游戏卡住

**触发时机：** 每次优先权变更后

---

### 不变量 4：堆栈深度非负 (InvariantStackDepthNonNegative)

**描述：** 堆栈深度必须大于等于 0

**检查逻辑：**
```go
func InvariantStackDepthNonNegative(state GameState) bool {
    return len(state.Board.Stack) >= 0
}
```

**违反后果：** 堆栈深度为负是逻辑错误，可能导致数组越界

**触发时机：** 每次堆栈操作后

---

### 不变量 5：版本号递增一致性 (InvariantRevisionConsistent)

**描述：** 版本号必须递增，且与历史记录一致

**检查逻辑：**
```go
func InvariantRevisionConsistent(state GameState) bool {
    // 版本号必须非负
    if state.Revision.Number < 0 {
        return false
    }
    // 历史记录数量应与版本号一致（或版本号 = 历史数 + 1）
    expectedRevision := len(state.History.Actions)
    return state.Revision.Number == expectedRevision || 
           state.Revision.Number == expectedRevision + 1
}
```

**违反后果：** 无法正确回放游戏，状态不一致

**触发时机：** 每次提交 Action 后

---

## 4. 实现计划

### 文件结构

```
server/pkg/rules/
├── invariants.go          # 不变量检查实现
├── invariants_test.go     # 不变量测试
└── ...
```

### 核心接口

```go
// InvariantFunc is a function that checks a specific invariant.
type InvariantFunc func(GameState) bool

// InvariantCheckResult contains the result of an invariant check.
type InvariantCheckResult struct {
    Name    string // 不变量名称
    Passed  bool   // 是否通过
    Message string // 失败信息（如果失败）
}

// CheckAllInvariants runs all invariant checks and returns results.
func CheckAllInvariants(state GameState) []InvariantCheckResult

// AssertInvariants panics if any invariant fails (for testing/debugging).
func AssertInvariants(state GameState)
```

### 配置选项

```go
// InvariantConfig controls invariant checking behavior.
type InvariantConfig struct {
    Enabled   bool // 是否启用
    PanicOnFail bool // 失败时是否 panic（仅测试）
}

var DefaultInvariantConfig = InvariantConfig{
    Enabled:     true,
    PanicOnFail: false,
}
```

---

## 5. 集成点

### 在 SubmitAction 中检查

```go
func SubmitAction(state GameState, action Action) (GameState, error) {
    // ... 提交动作 ...
    
    // 检查不变量
    if config.InvariantCheck.Enabled {
        results := CheckAllInvariants(newState)
        for _, result := range results {
            if !result.Passed {
                return state, fmt.Errorf("invariant violated: %s - %s", 
                    result.Name, result.Message)
            }
        }
    }
    
    return newState, nil
}
```

### 在测试中启用 Panic

```go
func init() {
    // 测试时启用 panic
    if testing.Testing() {
        DefaultInvariantConfig.PanicOnFail = true
    }
}
```

---

## 6. 测试策略

### 单元测试

为每个不变量编写测试：
- 测试通过情况（正常状态）
- 测试失败情况（构造违反条件）

### 集成测试

在关键流程后检查不变量：
- 打出卡牌后
- 堆栈结算后
- 优先权传递后

---

## 7. 下一步

1. 实现 `invariants.go` 中的核心函数
2. 实现 `invariants_test.go` 中的单元测试
3. 在 `SubmitAction` 中集成不变量检查
4. 验证所有现有测试通过
