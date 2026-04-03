# 全局领域语义收口计划 (Domain Semantics Consolidation Plan)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**背景与根因：**
系统频繁出现“旧状态残留”、“该谁动判断漂移”、“隐藏牌区域丢失”、“费用扣除遗漏”等问题，其共同根因是**领域语义没有集中建模，导致在不同层（后端会话、后端引擎、前端组件、前端测试）重复表达并逐渐漂移**。

**目标：**
通过建立“单一真相源”，彻底根除多点维护导致的逻辑漂移。
实施顺序严格遵循：状态机收口 -> 动作策略收口 -> 前端元数据驱动 -> 支付引擎收口 -> 跨层一致性校验。

**架构原则与洁癖：**
1. **单一真相源 (SSOT)**：所有核心校验和状态流转只允许在一处定义。
2. **前端无状态校验**：前端只做基于 schema/metadata 的展示驱动，绝对不允许硬编码业务规则。
3. **红绿重构 (TDD)**：所有变更必须先写/修改测试（使其失败），再实现逻辑（使其通过）。

---

## 阶段 1: 核心机制单一真相化 (Core Mechanisms SSOT)

### Task 1.1: SessionLifecycle 显式状态机 (P1)
**目标：** 消除 `session.go` 中多个字段拼凑生命周期的坏味道。
- [x] **Step 1: 定义 `SessionLifecycle` 状态机**
  - 状态：`Setup(step)`, `MatchActive`, `MatchFinished(reportRef)`。
  - 规则：所有会话状态转移只允许通过统一的 `Transition()` 函数。
- [x] **Step 2: 移除二次推导**
  - 废弃 `buildSetupSteps` 中的隐式推导，改由状态机直接记录当前所处步骤。
  - `Reset` 逻辑统一为一次全量重建。
- [x] **Step 3: 编写生命周期状态机回归测试**
  - 测试：状态机非非法跃迁拒绝、重置后状态纯净度校验。

### Task 1.2: ActionPolicy 集中化 (P1)
**目标：** 消除“谁能动”、“能干嘛”在各个动作文件和前端的离散判断。
- [x] **Step 1: 建立 `ActionPolicy` 结构**
  - 为每个动作声明：`ActorConstraint` (优先权/主回合), `TargetConstraint`, `TimingConstraint`, `QuotaConstraint`。
- [x] **Step 2: 引擎侧并轨**
  - `action_preflight_flow.go` 和各动作的独立校验，全部改为读取 `ActionPolicy`。
- [x] **Step 3: 暴露 Metadata**
  - 将计算好的 `ActionPolicy` 附加到 `rulesMetadata` 中，随 `StatePatched` 下发给前端。

---

## 阶段 2: 前端元数据驱动重构 (Frontend Metadata-Driven)

### Task 2.1: 移除前端硬编码校验 (P1)
**目标：** 前端成为纯粹的渲染与提交层。
- [x] **Step 1: 重构 `ActionComposer.validateBeforeSubmit`**
  - 删掉前端自己维护的动作前提条件代码。
  - 改为读取 Go 下发的 `rulesMetadata.actionPolicies` 进行禁用/高亮判断。
- [x] **Step 2: 共享忠诚解析器 (P2)**
  - 抽取忠诚/颜色限制解析为通用层（前端直接引用或读取 Go 下发配置），移除 `battle.spec.ts` 和业务逻辑中的重复实现。
- [x] **Step 3: Playwright 验证**
  - 运行 e2e 测试，确保按钮的可用性状态在重构后表现一致。

---

## 阶段 3: PaymentEngine 收口与 Rulebook 骨架 (P2)

### Task 3.1: `queue_operation` 与 `build_asset` 支付并轨
**目标：** 消灭游离于支付系统之外的费用路径。
- [x] **Step 1: TDD 失败测试构建**
  - 编写 `queue_operation` 和 `build_asset` 资源不足时的测试用例。
- [x] **Step 2: 并轨实现**
  - `queue_operation`：预检查读取 `source.Cost`，执行时调用 `PayCost`。
  - `build_asset`：接入检查和扣费接口（即使默认是0费）。

### Task 3.2: 隔离 Prototype 与 Rulebook 模式
**目标：** 把当前的“可玩优先”资源模型封入 `PaymentModePrototype`，为正式规则书让路。
- [x] **Step 1: 升级接口**
  - `PaymentEngine` 增加 `OnStepEnd` 钩子。
- [x] **Step 2: Rulebook 骨架实现**
  - 实现 `PaymentModeRulebook`：`RefillForTurn` 为空，`OnStepEnd` 清空浮动资源。
- [x] **Step 3: 引擎模式解耦**
  - `resources.go` 不再承担判断，所有加减法依赖 `CurrentPaymentEngine()`。

---

## 阶段 4: 投影契约与跨层一致性校验 (Projection & Cross-layer)

### Task 4.1: 确立 ProjectionContract (P2)
**目标：** 解决隐藏信息投影导致的布局字段丢失。
- [x] **Step 1: 制定必保留清单**
  - 即使是隐藏牌（`FaceDown`或不可见），也必须保留 `zone`, `owner`, `regionCardId`, `faceDown` 等结构字段。
- [x] **Step 2: 修改 `hiddenCardView`**
  - 严格按照契约填充隐藏视图。

### Task 4.2: 跨层一致性测试防护网
**目标：** 保证未来的迭代不会打破今天的规矩。
- [x] **Step 1: 综合状态机测试** (`server/internal/api/session_test.go`)
- [x] **Step 2: 策略执行测试** (`server/pkg/rules/action_policy_test.go`)
- [x] **Step 3: 前端 Metadata 测试** (`web/src/battle/actionPolicy.test.ts`)
- [x] **Step 4: 投影防泄漏与完整性测试** (`server/pkg/rules/projection_test.go`)

---

## 决策与留档要求 (Design Decisions)
在每个 Task 执行完毕后，必须生成/更新 `docs/trae_review/` 下的中文架构决策文档，包含：
1. **决策点**：为什么这么改？（如：为何废弃前端验证）
2. **影响面**：哪些系统边界受到了保护。
3. **重点代码留档**：贴出核心的 Interface 或关键算法。
