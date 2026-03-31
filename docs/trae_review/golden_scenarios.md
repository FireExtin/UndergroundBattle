# Golden Scenarios 设计文档

## 1. 高层摘要 (TL;DR)

*   **目标：** 定义并实现 Golden Scenarios（黄金场景），作为回归测试的基准
*   **场景列表：**
    1.  XQ22 禁止事务卡
    2.  XQ31 保护声望盟友
    3.  完整游戏回合
*   **状态：** 设计完成，准备实现

---

## 2. 什么是 Golden Scenario？

Golden Scenario 是一系列预定义的游戏场景，代表"正确的游戏行为"。它们用于：

- **回归测试**：确保新功能不会破坏现有功能
- **文档**：帮助新开发者理解系统行为
- **调试**：快速定位问题

### 格式（Given-When-Then）

```
Scenario: <场景名称>
  Given: <初始状态>
  When: <玩家动作>
  Then: <预期结果>
```

---

## 3. 场景定义

### 场景 1：XQ22 禁止事务卡

**目的：** 验证 XQ22 的 prohibition 效果正常工作

**Given：**
- P1 场上有就绪的 XQ22（州议员贝伦·希恩斯）
- P2 手牌中有事务卡

**When：**
- P2 试图打出事务卡

**Then：**
- 动作被拒绝
- 返回 `LEGALITY_FAILED_ACTION_PROHIBITED`
- 错误信息包含 XQ22 的卡牌信息

---

### 场景 2：XQ31 保护声望盟友

**目的：** 验证 XQ31 的目标合法性效果正常工作

**Given：**
- P1 场上有就绪的 XQ31（莫兰大主教）
- P1 场上有声望盟友
- P2 手牌中有可以目标角色的卡牌

**When：**
- P2 试图目标 P1 的声望盟友

**Then：**
- 动作被拒绝
- 返回 `TARGET_FAILED_PROHIBITED`
- 错误信息包含 XQ31 的卡牌信息

---

### 场景 3：完整游戏回合

**目的：** 验证完整的游戏流程

**Given：**
- 初始游戏状态
- P1 有优先权
- P1 和 P2 手牌中都有卡牌

**When：**
1. P1 打出卡牌（进入堆栈）
2. P2 响应（打出快速卡牌）
3. P2 传递优先权
4. P1 传递优先权
5. 堆栈开始结算
6. 堆栈结算完成

**Then：**
- 所有动作都被接受
- 版本号正确递增
- 状态正确变更
- 生成正确的事件
- Invariants 全部通过

---

## 4. 实现计划

### 文件结构

```
server/pkg/rules/
├── golden_scenario_test.go    # Golden Scenarios 测试
└── ...
```

### 核心接口

```go
// GoldenScenario represents a test scenario.
type GoldenScenario struct {
    Name        string
    InitialState GameState
    Actions     []Action
    ExpectedResults []ExpectedResult
}

// ExpectedResult defines the expected outcome of an action.
type ExpectedResult struct {
    ActionIndex int
    ShouldSucceed bool
    ExpectedErrorCode string // empty if should succeed
    PostCondition func(GameState) bool
}

// RunGoldenScenario executes a golden scenario and validates results.
func RunGoldenScenario(t *testing.T, scenario GoldenScenario)
```

---

## 5. 测试策略

### 单元测试风格

每个场景是一个完整的测试函数：

```go
func TestGoldenScenario_XQ22BlocksEventCard(t *testing.T) {
    // Setup initial state
    // Execute actions
    // Validate results
}
```

### 验证点

- 动作是否被接受/拒绝
- 错误码是否正确
- 状态是否正确变更
- Invariants 是否通过
- 版本号是否正确

---

## 6. 下一步

1. 实现 `golden_scenario_test.go` 中的场景
2. 运行所有场景测试
3. 验证所有原有测试继续通过
