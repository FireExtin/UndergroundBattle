# PaymentEngine + PaymentModePrototype（2026-04-03）

## 目标

把当前“按回合数补资源”的临时资源模型从具体规则逻辑里抽出来，形成可替换的支付边界，但**不**在这一轮改动现有客户端协议和 `TurnState.Resources` 视图。

本轮目标只有两件事：

1. 让资源生成 / 查询 / 扣费都经过统一 `PaymentEngine`
2. 把当前临时实现明确命名为 `PaymentModePrototype`

---

## 已落地

### 1. 新增统一支付接口

新增：

- `server/pkg/rules/payment.go`

当前接口：

- `PaymentEngine`
  - `Mode()`
  - `Initialize()`
  - `RefillForTurn()`
  - `ResourceView()`
  - `PayCost()`

以及全局当前模式入口：

- `CurrentPaymentEngine()`
- `CurrentPaymentMode()`

### 2. 当前实现被显式命名为 `PaymentModePrototype`

`resources.go` 不再只是“临时函数堆”，而是承载 `PrototypePaymentEngine`。

它仍然保留原有 prototype 语义：

- `turnNumber` 决定资源上限
- `end -> main` 时双方一起补满
- `play_card` 直接扣减当前资源池

但这些行为现在都经过 `PaymentEngine`，不再散落在：

- `NewGameState`
- `applyPhaseAdvance`
- `play_card` legality / execution

### 3. `play_card` / `build_asset` 改为通过 PaymentEngine 走统一费用入口

`play_card` 当前两处都改走统一引擎：

- legality 阶段：读取 `engine.ResourceView()`
- commit 阶段：调用 `engine.PayCost()`

这意味着后续切换到 rulebook 模式时，不需要再去每个 action 里翻旧资源函数。

`build_asset` 也已经接到同一条路径，但当前费用语义明确为：

- 建立资产是卡牌从手牌转换到资产区的标准动作
- 不支付该牌的印刷费用
- 在 prototype 模式下会经过 `PaymentEngine`，但请求费用恒为 `0`

### 4. rules metadata 现在显式暴露 payment mode

`RulesMetadata` 新增：

- `payment.mode`

当前值固定为：

- `prototype`

这样前端和调试器可以知道自己看到的是 prototype 支付模型，而不是把当前资源池误认成最终规则书语义。

---

## 刻意没做

### 1. 没有改 `TurnState.Resources` 的对外结构

这轮仍然保留：

- `TurnState.Resources[playerId] = { current, max }`

原因是当前目标是先抽边界，不是同时重写投影协议。

### 2. 没有实现 rulebook 支付流程

还没做：

- 横置资产得费
- 分步支付
- 步骤结束清空支付资源
- 费用选择 / 支付日志 / 中间态

也就是说，当前只有：

- `PaymentModePrototype`

还没有：

- `PaymentModeRulebook`

### 3. 没有把所有动作都切到 PaymentEngine

本轮只收口了当前真实用到的 battle 主路径：

- 初始化补费
- 回合切换补费
- `play_card`

如果后面 `queue_operation` / 其他费用动作进入正式语义，也要继续并轨到同一支付接口。

---

## 新增护栏

- `server/pkg/rules/resources_test.go`
  - 当前 payment engine 必须暴露 `prototype` mode
  - 初始化资源和回合切换补费必须继续工作
  - `PayCost()` 在资源不足时必须拒绝且不污染状态
- `server/pkg/rules/action_policy_test.go`
  - projected `rulesMetadata.payment.mode` 必须是 `prototype`

---

## 后续顺序

1. 新增 `PaymentModeRulebook` 骨架，但先不切默认模式
2. 让 `queue_operation` 的费用路径也走 `PaymentEngine`
3. 把“支付资源池快照”和“最终公共资源显示”拆开，避免 prototype 视图绑死未来规则书模型
