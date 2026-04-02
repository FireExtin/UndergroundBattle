# SessionLifecycle + ActionPolicy Refactor（2026-04-03）

## 目标

本轮不继续堆叠单点 legality patch，而是先把几个反复漂移的领域语义收拢成“单一真相”：

1. 会话生命周期
2. 动作策略目录
3. 忠诚颜色词表
4. 隐藏牌投影契约

本轮**没有**推进 PaymentEngine 并轨；资源支付仍保持原型模式，只把边界写清楚并为后续切换留接口位置。

---

## 已落地

### 1. SessionLifecycle 成为 sandbox session 的唯一生命周期真相

新增 `server/internal/api/lifecycle.go`，当前生命周期只允许以下 4 种状态：

- `reset`
- `setup`
- `match_active`
- `match_finished`

`SandboxSession.SubmitAction()` 不再通过 `setup.Active && !setup.Completed` 这样的组合字段判断能否提交动作，而是直接读取 `session.lifecycle.Kind`。

`SetupState` 现在额外暴露 `lifecycle`，方便前端和测试看到当前 authoritative 生命周期，而不是再去猜 `active/completed/currentStep` 的组合语义。

### 2. setup steps 不再二次推导，而是由真实转移直接写入

旧实现里 `buildSetupSteps()` 会根据 `currentStep/completed` 再推导一遍 step completion。  
本轮改为：

- `StartSetup()` 创建全量 step 列表
- `AdvanceSetup()` 在每次真实 step 完成时直接标记该 step
- 第 7 步完成后切到 `match_active`

这样 setup 的 step 历史不再是“从别的字段推出来”，而是由真实转移写入。

### 3. ActionPolicy 进入 Go authoritative metadata，并下发到前端/测试

新增 Go 侧规则元数据目录：

- `server/pkg/rules/action_policy.go`
- `RulesMetadata`
- `ActionPolicy`
- `ActionFieldRule`

当前 ActionPolicy 承载两类约束：

- actor/timing 约束：Go legality preflight 直接消费
- field rule 约束：通过 projection 下发给前端，由前端 schema-driven 校验消费

本轮先统一了最容易漂移的 actor/timing 语义：

- priority player 约束
- active player 约束
- empty stack 约束

`use_first_player_privilege` 的“必须是主动玩家”不再由动作文件单独定义，而是转入 ActionPolicy 的 `active_player` actor constraint。

### 4. 前端不再手写 ActionComposer 分支校验

旧的 `ActionComposer.validateBeforeSubmit()` 里直接硬编码了：

- `play_card` 角色要地区
- `play_card` 附属要宿主
- 攻击/调查/移动要 source+target
- marker 要 type+amount

现在改为：

- Go projection 给每个 `PlayerViewState` / `SpectatorViewState` 下发 `rulesMetadata`
- `web/src/battle/actionPolicy.ts` 统一解释 metadata
- `ActionComposer` 只调用 `validateActionInput()`

前端的“可提交”判断现在是消费服务端策略，而不是自己维护动作分支表。

### 5. 忠诚解析共享同一套颜色词表

Go 侧忠诚颜色词表已经从 `play_card_action.go` 抽到 `server/pkg/rules/loyalty.go`。

同时 projection metadata 下发同一份 loyalty color alias vocabulary。  
前端和 Playwright e2e 不再各自维护一份中文颜色 hardcode，而是统一走：

- `rulesMetadata.loyalty.colorAliases`
- `web/src/battle/actionPolicy.ts`

这一步解决的是“跨层重复解析器”问题，不是把 Go 代码直接共享到 TS。共享的是**词表契约**。

### 6. ProjectionContract 明确 hidden card 必保留字段

`RulesMetadata.projection.hiddenCardPreserves` 当前声明 hidden card 至少保留：

- `ownerId`
- `zone`
- `regionCardId`
- `regionOrder`
- `faceDown`
- `destroyed`

并且 Go hidden projection 已明确保留 `regionOrder` / `destroyed`，避免再次出现“布局定位或状态字段在 hidden view 中消失”的周期性回归。

---

## 本轮刻意没做

### PaymentEngine 仍然延后

当前资源系统依然是 prototype 模式：

- 回合数补充资源
- `play_card` 直接读取 `turn.resources`

这轮没有把支付流程抽成完整 `PaymentEngine`，原因是：

1. SessionLifecycle + ActionPolicy 先解决“状态/权限语义漂移”这个更高频根因
2. PaymentEngine 会同时牵动 legality、结算、UI 提示和未来规则书资源模型，不适合在同一轮一起翻

下一轮建议把当前实现命名成 `PaymentModePrototype`，再补一个 `PaymentModeRulebook` 接口骨架。

---

## 新增护栏

### Go

- `server/internal/api/session_test.go`
  - 新增生命周期转移测试，覆盖 `reset -> setup -> match_active -> reset -> match_finished -> reset`
- `server/pkg/rules/action_policy_test.go`
  - 验证 projected metadata 含 action policy / loyalty aliases / projection contract
  - 验证 `use_first_player_privilege` 的 active-player 约束来自 ActionPolicy

### Web / e2e

- `web/src/battle/actionPolicy.test.ts`
  - 验证前端 field validation 读取 metadata
  - 验证 loyalty alias 解析读取 metadata
- `web/tests/battle.spec.ts`
  - Playwright 不再维护独立 loyalty parser，直接复用 `web/src/battle/actionPolicy.ts`

---

## 后续顺序

1. 把 prototype 资源模型显式抽成 `PaymentEngine + PaymentModePrototype`
2. 把 `queue_operation` / 更动态 target schema 逐步纳入 ActionPolicy
3. 给 hidden projection 增加更明确的 golden/schema 守护
4. 如果 setup 继续扩张，再把 `SetupState` 本身也收敛成更窄的 transition log / state snapshot 模型
