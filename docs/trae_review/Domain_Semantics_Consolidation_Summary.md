# 全局领域语义收口总结 (Domain Semantics Consolidation Summary)

## 决策背景 (Decision Background)
为了解决系统频繁出现的“状态残留”、“校验逻辑漂移”以及“隐秘信息泄露”等深层架构坏味道，我们实施了全局领域语义收口。核心目标是将分散在后端 API、规则引擎、前端组件和测试用例中的重复规则收拢为“单一真相源 (SSOT)”。

## 核心架构变更 (Core Architectural Changes)

### 1. 会话生命周期显式化 (Session Lifecycle)
- **变更**：引入了显式的 `SessionLifecycle` 状态机，并将 setup/match/reset 的切换收束到 `Transition()`。
- **当前边界**：`session.lifecycle` 已是唯一内部状态来源；`SetupState.Lifecycle` 只在返回 API 响应时按需投影，不再参与内部存储或转移。
- **保护**：非法跃迁会被拒绝，setup 第 7 步不会在执行前被提前标记为已完成。
- **文件**：`server/internal/api/lifecycle.go`, `server/internal/api/session.go`

### 2. 规则元数据驱动 (Rules Metadata Driven)
- **变更**：在 Go 后端定义了全量 `ActionPolicy`，通过 `RulesMetadata` 随投影下发给前端。
- **前端去状态化**：前端 `ActionComposer` 和 `actionPolicy.ts` 不再硬编码校验逻辑，而是基于元数据动态执行字段要求、行动者约束和忠诚解析。
- **文件**：`server/pkg/rules/action_policy.go`, `web/src/battle/actionPolicy.ts`

### 3. 支付引擎并轨 (Payment Engine Consolidation)
- **变更**：确立了 `PaymentEngine` 接口，并将正式战斗动作优先并入统一支付入口。
- **当前边界**：`build_asset` / `play_card` 走正式支付语义；`queue_operation` 继续保留为调试/fixture 通道，不强行套用 battle 费用与忠诚规则。
- **模式隔离**：隔离了 `Prototype` (当前原型) 与 `Rulebook` (未来正式规则) 模式。在 `Rulebook` 模式下，步结束会通过 `OnStepEnd` 钩子自动清空浮动资源。
- **文件**：`server/pkg/rules/payment.go`, `server/pkg/rules/resources.go`

### 4. 投影契约加固 (Projection Contract)
- **变更**：明确了隐藏牌投影必须保留的结构化字段（`ownerId`, `zone`, `regionCardId`, `regionOrder`, `faceDown`, `destroyed`），确保前端布局一致性。
- **文件**：`server/pkg/rules/projection.go`

## 重点代码留档 (Key Code Reference)

### 状态机转移逻辑 (Transition Logic)
```go
func (session *SandboxSession) Transition(next SessionLifecycle) error {
    // 强制执行生命周期拓扑结构
    // 1. Reset -> Setup 1
    // 2. Setup N -> Setup N+1
    // 3. Setup Final -> MatchActive
    // 4. MatchActive -> MatchFinished
}
```

### 动作策略定义 (Action Policy)
```go
{
    ActionKind: ActionKindPlayCard,
    ActorConstraint: ActionActorConstraintPriorityPlayer,
    RequiresPriority: true,
    FieldRules: []ActionFieldRule{
        {Field: ActionFieldNameCardID, Requirement: ActionFieldRequirementRequired},
        {Field: ActionFieldNameTargetRegionCardID, Requirement: ActionFieldRequirementRequired, SourceKinds: []CardKind{CardKindCharacter}},
    },
    CardKindConstraints: []ActionCardKindConstraint{
        {Kind: CardKindCharacter, RequiresEmptyStack: true, RequiresActionWindow: true},
    },
}
```

## 下一步建议 (Next Steps)
- 随着 `Rulebook` 模式的推进，逐步迁移 `Prototype` 中的资源刷新逻辑到正式资产横置模型。
- 扩展 `ActionPolicy` 以支持更复杂的动态 Target Schema。
- 若后续协议允许，可进一步把 `SetupState.Lifecycle` 从 JSON 负载里也收掉，只保留独立生命周期字段。
