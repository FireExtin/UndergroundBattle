# Replay 系统分析文档

## 1. 高层摘要 (TL;DR)

*   **目标：** 分析现有 Replay 能力，验证系统可以正确回放游戏
*   **现状：** 已有基础 Replay 功能（`ReplayActions` 函数），需要验证其正确性
*   **关键组件：**
    - `GameState.Revision` - 版本控制
    - `HistoryState.Actions` - 动作日志
    - `ReplayActions()` - 回放函数
*   **状态：** 分析完成，基础功能已存在

---

## 2. 现有 Replay 能力分析

### 核心数据结构

```go
// GameState 包含版本信息
type GameState struct {
    Revision Revision     // 当前版本
    History  HistoryState // 历史记录
    // ... 其他字段
}

// Revision 追踪状态版本
type Revision struct {
    Number int    // 版本号
    Hash   string // 状态哈希
}

// HistoryState 记录所有动作
type HistoryState struct {
    Actions []Action // 动作序列
}

// Action 记录玩家意图
type Action struct {
    ID             string     // 动作唯一ID
    ActorID        string     // 执行者
    Kind           ActionKind // 动作类型
    CardID         string     // 相关卡牌（可选）
    TargetPlayerID string     // 目标玩家（可选）
    TargetCardID   string     // 目标卡牌（可选）
    // ... 其他字段
}
```

### 现有回放函数

```go
// ReplayActions replays an action log against an initial snapshot.
func ReplayActions(initial GameState, actions []Action) (GameState, error) {
    replayed := cloneGameState(initial)
    for _, action := range actions {
        result, err := submitActionWithoutProjection(replayed, action)
        if err != nil {
            return GameState{}, err
        }
        replayed = result.State
    }
    return replayed, nil
}
```

---

## 3. Replay 验证需求

### 需要验证的点

1. **确定性：** 同样的初始状态 + 同样的动作序列 = 同样的最终状态
2. **版本一致性：** 回放后的版本号应与原始一致
3. **不变量保持：** 回放过程中所有不变量都应通过
4. **错误检测：** 能够检测不一致的状态

### 验证场景

| 场景 | 描述 |
|------|------|
| 简单回放 | 几个 PassPriority 动作 |
| 复杂回放 | 包含堆栈操作的动作序列 |
| 错误检测 | 检测动作序列与最终状态不匹配 |

---

## 4. 实现计划

### 文件结构

```
server/pkg/rules/
├── replay_test.go    # Replay 验证测试
└── ...
```

### 核心测试

```go
// TestReplaySimpleSequence verifies replay of simple action sequence.
func TestReplaySimpleSequence(t *testing.T)

// TestReplayDeterminism verifies that replay produces deterministic results.
func TestReplayDeterminism(t *testing.T)

// TestReplayWithInvariants verifies invariants hold during replay.
func TestReplayWithInvariants(t *testing.T)
```

---

## 5. 测试策略

### 测试 1：简单回放

1. 创建初始状态
2. 执行 3-5 个动作
3. 记录最终状态
4. 使用 ReplayActions 回放相同动作
5. 验证回放后的状态与原始最终状态一致

### 测试 2：确定性验证

1. 多次回放相同的动作序列
2. 验证每次结果都相同

### 测试 3：不变量验证

1. 在回放过程中检查不变量
2. 确保所有不变量都通过

---

## 6. 下一步

1. 实现 `replay_test.go` 中的测试
2. 运行所有 Replay 测试
3. 验证所有原有测试继续通过
