# 开发决策与架构留档 (Development Decisions & Architecture Review)

## 1. 规则引擎与支付系统优化
- **决策点**：保留 `build_asset` 的正式支付并轨，但不把 `queue_operation` 强行升级为 battle 费用/忠诚入口。
- **影响面**：避免调试/fixture 通道和正式对战语义混淆，现有 M0 场景、golden replay 与调试器行为继续稳定。
- **重点代码**：
  ```go
  func checkQueueOperationActionLegality(...) LegalityResult {
      // queue_operation 保持 window / target / prohibition 校验
      // 不在这里强制接入 battle payment / loyalty
  }
  ```

## 2. 会话生命周期状态机增强
- **决策点**：`Transition` 只再写 `session.lifecycle`，不再回写 `session.setup`。`SetupState.Lifecycle` 仅在 API 返回时按 `session.lifecycle` 投影。
- **影响面**：内部生命周期只剩一个真相源，避免 setup 副本残留旧生命周期导致对外状态漂移。
- **重点代码**：
  ```go
  func (session *SandboxSession) Transition(next SessionLifecycle) error {
      session.lifecycle = next
  }

  func projectSetupState(state SetupState, lifecycle SessionLifecycle) SetupState {
      cloned := cloneSetupState(state)
      cloned.Lifecycle = lifecycle
      return cloned
  }
  ```

## 3. Setup 输入前置校验
- **决策点**：在 setup 第 1 步就拒绝“不是 2 个派系”或“重复选择同一派系”的输入，而不是拖到第 4 步构筑牌库时报硬错误。
- **影响面**：前端和后端的用户感知一致，非法开局输入会在最靠前的边界被拦下。
- **重点代码**：
  ```go
  func validateSocietyChoices(playerID string, societies []string) error {
      if len(societies) != societyLimit {
          return fmt.Errorf("society_count_invalid: ...")
      }
      // duplicate rejection
  }
  ```

## 4. 动作时序修正
- **决策点**：移除 `advance_phase` 对 action window 的错误依赖，允许在步骤关闭后由当前优先权玩家推进阶段。
- **影响面**：恢复 `pass -> pass -> advance_phase` 的正常主流程，避免卡死在 `step_ended`。

## 5. 质量保证 (QA)
- **新增测试**：补了真实入口回归测试，覆盖：
  - `advance_phase` 在 closed window 下仍可推进
  - `queue_operation` 在真实 fixture 路径下继续保持 debug 兼容
  - setup 第 1 步重复派系 / 单派系提前拒绝
- **验证结果**：上述回归测试和 `go test ./server/...` 全部通过。
