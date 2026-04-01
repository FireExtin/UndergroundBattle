# 项目交接文档

> 《隐秘世界》数字化项目 - Codex 会话交接
> 基于 `docs/TEMPLATE_HANDOVER.md` 填写

---

## 1. 项目概览

- **项目名称**：隐秘世界数字化项目
- **项目目标**：先构建 Go 权威规则核 + TypeScript 调试/契约工具链，保证规则可回放、可测试、可投影，再逐步扩真实卡与规则
- **技术栈**：Go、TypeScript、Vitest、JSON Schema
- **项目阶段**：`Phase 3` 持续扩真实卡语义与 legality / fixture 契约
- **交接时间**：2026-04-01

### 1.1 Big Picture

这个项目当前不是“做 UI 产品”，也不是“快速把一堆卡录进去”。
真正的北极星是先做出一台可长期维护的规则机，满足下面四条：

1. Go 服务端是唯一语义权威
2. 所有重要状态变化都可 replay / 可审计 / 可回归
3. 隐藏信息通过 projection 隔离，而不是靠前端自觉
4. 新卡进入主卡池前，必须先经过 fixture gate 和 rules test

换句话说，眼前的 live sandbox、web debugger、fixture catalog，都不是终点；它们是为了把规则核变成一个“能安全持续扩卡”的工程系统。

### 1.2 当前阶段的真实定位

当前仓库已经越过“纯骨架”阶段，但还没有到“完整可玩 alpha”：

- 已经有：
  - priority / stack / projection / replay
  - continuous effects 最小层
  - 角色动作、地区控制、得分、胜利、reset
  - 第一批真实 fixture
- 还没有：
  - 通用 prohibition / targeting framework
  - attachments / host / recycle 等永久物生命周期
  - trigger / replacement / prevention 正式框架
  - 大规模真实卡池
  - 正式 match service / websocket / 持久化

所以接手时的正确心态应该是：

- 这是一个“规则引擎工程化阶段”的项目
- 不是“继续堆前端功能”的项目
- 也不是“为了追卡数而快速加特判”的项目

### 1.3 新 AI 建议阅读顺序

如果是第一次接手，建议按这个顺序看文档，不要乱跳：

1. `README.md`
   - 理解项目目标、非目标、权威边界、测试纪律
2. `docs/NEXT_GEN_RULE_PLAN.md`
   - 理解当前总路线与阶段状态
3. `docs/NEXT_STEP_EXECUTION_PLAN_2026-03-31.md`
   - 理解为什么当前优先级不是 UI / 联机 / 大规模扩卡
4. `docs/LIVE_SANDBOX_2026-03-31.md`
   - 理解当前 live sandbox 实际提供了什么，不提供什么
5. `docs/GO_CARD_FIXTURE_ENTRYPOINT_2026-03-31.md`
   - 理解 fixture → normalized artifact → queue_operation → Go rules 的入口链路
6. `docs/TEST_PLAN.md`
   - 理解本项目什么改动必须带测试，什么行为不允许
7. 再回来看本交接文档
   - 用来决定“下一步到底该做哪一刀”

---

## 2. 当前进度

### 2.1 已完成

- [x] 最小 Go 规则核、priority / stack / projection / replay 已建立
- [x] `declare_attack / declare_investigation` 已接入
- [x] 地区控制、得分、主动玩家轮转、终局 gate、sandbox reset 已接入
- [x] continuous source lifecycle 最小闭环已接入
- [x] `XQ31` 的 target legality 已收窄到真正的 card-effect targeting，不再误伤 `declare_attack / declare_investigation`
- [x] invariant check 已从 pre-commit working state 改为 committed state，并收紧了 `revision == len(history.actions)` 一致性
- [x] 第一批 10 张真实 fixture 已接入 shared catalog / normalized artifact
- [x] fixture `card.basicType` 已贯通 schema / TS / Go / normalized artifact
- [x] `XQ22` 第一条禁令已接入：ready 时禁止打出 `basicType == "事务"` 的卡
- [x] `XQ22` 禁令已从"按卡名匹配"修正为"按 `CardState.DefinitionID` 匹配"
- [x] attachment tracking V0 已接入：fixture-only 附属 source 也可被追踪，且 host 离场会同步清理对应 continuous effect
- [x] 对应高风险回归测试已补齐
- [x] **Attachment / Host Lifecycle V1 已完成**：
  - `Attachment` 结构体新增 `HostCardID` 字段
  - `AttachmentBuilder` 新增 `Host()` 方法
  - `PruneExpired()` 实现宿主离场联动和 continuous effect 清理
  - 新增 `attachment_lifecycle_test.go` 完整测试
- [x] **legality production rule catalog 已建立**（`server/pkg/rules/legality_catalog.go`）
- [x] **shared source-condition 和 actor-scope matching 已提取**（`server/pkg/rules/legality_shared.go`）
- [x] **`XQ31` 只在 `queue_operation` 上检查 target legality，`declare_attack/declare_investigation` 不再被误伤**
- [x] **`XQ31` 的数值光环已接入，并收紧到“本方声望角色”**
- [x] **`XQ01` 的错误全局沉默实现已回滚；该牌仍保持 deferred**

### 2.2 正在进行

- **当前任务**：把 `B` 组中的最窄 legality slice 稳定下来，并为 TRAE 留下可继续扩展的交接文档
- **相关文件**：
  - `server/pkg/rules/engine.go`
  - `server/pkg/rules/projection.go`
  - `server/pkg/rules/engine_test.go`
  - `shared/schemas/fixture.schema.json`
  - `shared/contracts/fixtures/*.fixture.json`
  - `shared/contracts/normalized/card-logic.contracts.normalized.json`
  - `tools/fixture-tools/src/contracts.ts`
  - `tools/fixture-tools/src/validation.test.ts`
  - `docs/GO_CARD_FIXTURE_ENTRYPOINT_2026-03-31.md`
  - `docs/NEXT_GEN_RULE_PLAN.md`
- **开始时间**：2026-04-01

### 2.2.1 这轮工作的真实意义

这轮不只是“修了 XQ22”。
更重要的推进有两条：

1. `basicType` 终于进入 fixture contract 主链
   - 以后 legality 不用只看 `cardId`，可以先按卡类型过滤
2. `CardState.DefinitionID` 正式出现
   - 这把“场上实例 ID”和“卡牌定义 ID”分开了
   - 是以后做 aura / prohibition / attachment source / 同名多张共存的必要前提

如果只把这轮看成“多了一张卡的禁令”，就会低估它对后续架构的价值。

### 2.2.2 最新纠偏同步（这部分优先于前文旧表述）

这份交接文档最初写完后，又发生了两轮重要的纠偏修复。TRAE 接手时必须把这两条视为当前事实，而不是沿用旧判断。

#### A. `XQ31 / invariant` 纠偏

动机：

- 之前的 `XQ31` target legality 是“只要 action 带 `targetCardId` 就拦”，这会误伤 `declare_attack / declare_investigation`
- 之前的 invariant check 挂在 commit 前，只能看到 working state，看不到最终 `revision/history/continuous recalc` 的 commit 结果
- 同时 `InvariantRevisionConsistent` 被放宽成了 `revision == len(actions)` 或 `+1` 都算合法，这会掩盖真正的 committed-state 不一致

现在已经修成：

- `XQ31` 只在 `queue_operation` 上检查 target legality，也就是只拦“卡牌/能力指定目标”这条主链
- `declare_attack / declare_investigation` 不再被 `XQ31` 误伤
- invariant 改为在 committed state 上检查
- `InvariantPriorityPlayerValid` 现在会按 engine 的真实 fallback 逻辑读 `PriorityPlayerID`
- `InvariantRevisionConsistent` 现在要求 committed state 严格满足 `revision == len(history.actions)`

这条修复的重点不是“过一个测试”，而是重新把 legality 和 health-check 放回正确层次：

- `XQ31` 是 targeting restriction，不是泛化的“所有 targetCardId 动作禁止”
- invariant 是 commit 后真相检查，不是 pre-commit 中间态检查

#### B. `附属追踪系统` 纠偏

动机：

- attachment tracking 初版要求 source 必须是场上实体 `cardId`
- 但真实 `queue_operation` 打出的 `BQ022` 这类附属，目前 source 仍是 fixture / definition identity，不是已 materialize 的 table permanent
- 同时 attachment 被 prune 时，对应的 continuous effect 没有一起失效；再加上 `cloneBoardState()` 漏了 `Attachments` 深拷贝，这三处会让“看起来有追踪、实际生命周期仍错位”

现在已经修成：

- attachment 允许用 `sourceDefinitionId + sourceOperationId` 追踪 fixture-only source
- 因此 `BQ022` 这种真实 `queue_operation` 路径现在也会建立 attachment 记录，而不是只在手工测试 state 里成功
- `ContinuousEffect` 新增 `attachmentId`，host 离场后 attachment prune 会同步清掉对应 continuous effect
- `cloneBoardState()` 已补上 `Attachments` 深拷贝，避免 snapshot aliasing

这条修复的重点也要理解清楚：

- 这不是“完整附属系统完成了”
- 这是一个 **attachment tracking V0**
- 它解决的是 review 里指出的三个硬伤：
  - 生产路径不可达
  - attachment 和 continuous 生命周期脱钩
  - state clone 漏接

但它**没有**解决：

- 附属作为真实 table permanent 的正式上场表示
- `attachedTo` 驱动的完整 permanents lifecycle
- `回收` / 回手 / 附属自身离场处理

如果 TRAE 后续继续做附属，不要误以为这块已经“系统化完成”；它只是从“错误的半成品”推进到了“可以安全继续演进的 V0”。

### 2.3 待完成

- [x] 把 `XQ22` 从单卡特例推进成更一般的 scoped prohibition / targeting framework（legality_catalog.go + legality_shared.go）
- [x] 完成 `XQ31` 数值光环（+1 防御力）实现，但当前只覆盖“本方声望角色”这一正确最小语义
- [ ] 设计 `XQ01` 地区作用域沉默的 prerequisite
- [x] 把 `CardState.DefinitionID` 贯穿到未来真正的 permanents / attachments 上场建模（Attachment / Host Lifecycle V1 已完成）
- [x] **Secret Society Marker V1 已完成**：
  - `MarkerRegistry` 类型和 `BoardState.Markers` 字段
  - `GetMarker()` / `SetMarker()` 方法
  - Projection 支持 `PlayerViewState.Markers` / `SpectatorViewState.Markers`
  - 新增 `marker_test.go` 完整测试
- [x] **Hidden Deployment & Reveal V1 已完成**：
  - `CardState.FaceDown` 字段
  - 投影可见性控制（owner 可见，对手/观众隐藏）
  - 新增 `hidden_deployment_test.go` 完整测试
- [x] **Timing Window V2 已完成**：
  - Fast action 允许条件检查
  - Reaction 需要 stack 非空
  - 新增 `timing_window_test.go` 完整测试
- [x] **Conflict Loop V2 已完成**：
  - 战斗目标合法性检查
  - 调查与地区控制衔接
  - 游戏结束检查
  - 新增 `conflict_loop_test.go` 完整测试
- [ ] 把当前"rules core + fixture gate + sandbox"整理成更清晰的 AI 接手路径，避免后续会话重新摸索上下文

### 2.3.1 最新待做补充（2026-04-01，覆盖优先）

> 以下待做项基于“降复杂度重构”最新落地状态，应优先于旧的“继续扩卡”叙事。

- [ ] **统一状态迁移入口继续收口（Transition API）**
  - 已收口：`inspect`、`drawCards`、`RandomResults`、`Board.Resolved`
  - 待收口：其余跨模块 append/字段写入点（尤其是未来 permanents/attachments 扩展路径）
- [ ] **`engine.go` 继续去重型**
  - 已拆出：`marker_actions.go`、`card_effect_resolution.go`
  - 待拆：legality/card-source catalog 相关段落，目标是 engine 只保留 orchestration
- [ ] **建立“禁止散写”的长期护栏**
  - 增加面向 transition helper 的约束测试/检查（例如针对关键字段写入路径的预算或守卫）
  - 防止后续扩卡时回到“就地字段突变 + 局部特判”模式
- [ ] **effect 绑定粒度继续坚持实体 ID 语义**
  - 现有 `attachmentId` 绑定能力必须在后续新牌实现中持续沿用
  - 禁止回退到 source 粗粒度解绑，避免同源误删

---

## 3. 核心文件结构

```text
/UndergroundBattle
├── server/
│   ├── cmd/api/main.go
│   ├── internal/api/
│   └── pkg/rules/
│       ├── engine.go
│       ├── m0.go
│       ├── role_actions.go
│       ├── stack.go
│       ├── priority.go
│       ├── projection.go
│       ├── continuous.go
│       ├── dsl.go
│       ├── region.go
│       ├── regression.go
│       ├── scenario.go
│       ├── prohibition.go          # XQ22 prohibition checker
│       ├── target_legality.go      # XQ31 target legality checker
│       ├── legality_catalog.go     # production rule catalog (NEW)
│       ├── legality_shared.go      # shared source/scope matchers (NEW)
│       ├── types.go               # includes ProhibitionRule, TargetLegalityRule
├── web/
│   └── src/debugger/
├── shared/
│   ├── schemas/
│   ├── contracts/fixtures/
│   ├── contracts/normalized/
│   └── protocol/
└── docs/
    ├── NEXT_GEN_RULE_PLAN.md
    ├── GO_CARD_FIXTURE_ENTRYPOINT_2026-03-31.md
    ├── TEMPLATE_HANDOVER.md
    └── HANDOVER_TRAE_2026-04-01.md
```

---

## 4. 关键设计决策

### 4.1 架构原则

| 决策项 | 选择 | 原因 |
|-------|------|------|
| 规则权威 | Go 是唯一语义权威 | legality / replay / hidden info 必须统一由服务端裁决 |
| 状态可回放 | action log + revision + projection | 所有较重规则改动都要求可 replay / 可回归 |
| 隐藏信息 | FullState + per-player projection | 客户端不直接读取真相状态 |
| 持续效果 | commit 后统一重算、最小 layer 模型 | 先保住可解释与可回归，不先上完整 dependency graph |
| 测试策略 | Go `testing` + TS Vitest 双侧契约 | fixture / legality / projection / regression 需要双端锁定 |

### 4.1.1 明确不要做的事

接手后最容易犯的错不是“写错代码”，而是“走错方向”。下面几类事当前明确不该优先做：

- 不要把主要精力放在继续堆 web 调试器按钮
- 不要为了“看起来更完整”而抢先做 websocket / room service / DB
- 不要为了追卡数去写一堆单卡 name-based 特判
- 不要在 attachment / trigger / replacement 还没设计好时，拿 `BQ022` 这类牌硬冲
- 不要削弱测试断言来换取“表面上的通过”

如果不先守住这些边界，项目很快会从“可持续扩展的规则机”退化成“能跑但不可维护的 demo”。

### 4.2 Hook/Modifier 分层

```text
1. Legality Hooks     → inspect_card permission、winner gate、XQ22 queue_operation prohibition
2. Modifier Hooks     → modifyStat、addKeyword、grant/prohibit permission、costAdjustment 预留
3. Replacement/Prevention → 暂未正式展开
4. Trigger Hooks     → 暂未正式展开
5. Cleanup Hooks     → continuous prune、source 离场清理、turn duration 失效
```

### 4.2.1 当前最重要的架构张力

当前 rules core 正处在一个典型过渡期：

- 一方面，已经开始接真实卡，必须让 legality 真正读到 card semantics
- 另一方面，完整的 prohibition / targeting / attachment framework 还没到位

因此最近几刀不可避免地带有“窄切片、局部 helper”的过渡味道。
这没有问题，但必须满足两个条件：

1. 每一刀都在为未来抽象铺路，而不是把临时逻辑焊死
2. 每一刀都要带回归测试，保证以后抽象时能安全回收

### 4.3 持续效果 Layer

```text
LayerProhibition
LayerPermission
LayerCost
LayerNumeric
LayerActionQuota
```

---

## 5. 当前工作细节

### 5.1 最新实现的功能

**文件**：`server/pkg/rules/engine.go`

**功能描述**：

- `queue_operation` 现在在 window legality 之后再走一层 play legality
- 当前最小落地的是 `XQ22`：
  - 若场上存在任意 ready 的 `DefinitionID == "XQ22"` 实体
  - 则禁止打出所有 `basicType == "事务"` 的 fixture 卡
- 这条规则不再依赖显示名称，因此：
  - 改名不会失效
  - 同名碰撞不会误伤
  - 双方或单方存在多张同定义牌时也能正确工作

**关键代码片段**：

```go
func checkQueuedCardPlayLegality(state GameState, source CardOperationSource) LegalityResult {
	if source.BasicType != "事务" {
		return okLegalityResult()
	}

	for _, card := range state.Board.Cards {
		if !isReadyDefinitionCard(card, "XQ22") {
			continue
		}

		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.legality.action_prohibited",
			"board.cards",
			map[string]string{
				"cardId":              source.CardID,
				"basicType":           source.BasicType,
				"prohibitingCardId":   card.CardID,
				"prohibitingCardName": card.Name,
			},
		)
	}

	return okLegalityResult()
}
```

**测试覆盖**：

- `TestXQ22PreventsQueueOperationForEventCardsWhileReady`
- `TestXQ22StillPreventsEventCardsWhenDisplayNameChanges`
- `TestXQ22DoesNotPreventEventCardsForNameCollisionOnly`
- `TestXQ22PreventsBothPlayersFromQueueingEventCardsWhileReady`
- `TestXQ22AllowsNonEventCardsWhileReady`
- `TestXQ22AllowsEventCardsWhenInactive`
- `TestSubmitActionReturnsLegalityErrorWhenXQ22BlocksEventCards`
- `validation.test.ts` 额外覆盖 `basicType` 为空 / source basic-type mismatch

### 5.1.1 为什么这仍然是临时措施

虽然本轮已经把“按卡名特判”修正成了“按定义 ID 特判”，但它仍然不是最终架构。

当前状态可以理解为：

- 错误版本：`engine` 直接认显示名
- 现在版本：`engine` 直接认定义 ID
- 目标版本：`engine` 不直接认识 `XQ22`，而是解释一套通用 prohibition 描述

所以这轮的价值在于先把“身份模型”做对，而不是宣称 prohibition framework 已经完成。

### 5.1.2 这轮之后应如何看待 `DefinitionID`

`DefinitionID` 不是为了前端展示加的字段，而是为了规则识别：

- 同一玩家两张 `XQ22`：实例 ID 不同，`DefinitionID` 相同
- 双方都带 `XQ22`：四张场上实体都可以共享同一 `DefinitionID`
- 改名 / 别名 / 文案重写：不影响规则识别
- 同名碰撞：不会误触发某张特定卡的规则

后续只要碰到“某张真实定义牌在场 / 离场 / 结附 / 提供 aura”这种需求，都应该优先问：

- 这是实例问题，还是定义身份问题？

### 5.2 数据结构

**关键类型定义**：

```go
type CardState struct {
	CardID       string `json:"cardId"`
	DefinitionID string `json:"definitionId,omitempty"`
	Name         string `json:"name"`
	// ...
}

type CardOperationSource struct {
	CardID    string `json:"cardId"`
	BasicType string `json:"basicType,omitempty"`
	// ...
}
```

---

## 6. 已知问题与风险

### 6.1 待解决

| 问题 | 位置 | 状态 | 备注 |
|-----|------|-----|------|
| `XQ22` 仍属于“单卡规则实例”而非完整规则家族 | `server/pkg/rules/legality_catalog.go` | 已控范围 | 已从 engine 硬编码迁到 catalog，但仍需继续抽象更多可复用 prohibition 模式 |
| `DefinitionID` 还没贯穿真实 permanents lifecycle | `server/pkg/rules/projection.go` 及未来上场建模 | 待继续 | 当前主要在测试/规则识别里使用 |
| 状态迁移入口仍未完全覆盖 | `server/pkg/rules/*` | 进行中 | 已完成 inspect/draw/random/resolved 收口，仍需持续清理残余散写点 |
| `engine.go` 体量虽下降但仍偏大 | `server/pkg/rules/engine.go` | 进行中 | 已抽离 marker/card-effect 模块，仍需继续抽 legality/source-catalog 段 |

### 6.2 已知限制

- `XQ31` 现在已经有“本方声望角色 +1 防御力”这半边牌义，但过滤条件必须保持为 character-only，不能扩到任意声望 permanent
- `XQ01` 仍未做，因为还缺 region scoped silence / ability-kind restriction 正式框架
- 附属 / 结附 / 回收仍未完整建模；当前只有 attachment tracking V0，`BQ022` 不能视作完整正确实现
- `CardState.DefinitionID` 目前没有暴露到 projection，这符合 hidden-info 边界，但也意味着客户端不能直接拿它做展示或预校验

### 6.2.1 需要特别提醒新 AI 的认知陷阱

下面这些误解非常常见，接手时要主动避免：

- 误解 1：`CardState.CardID` 就是牌库里的卡牌定义 ID
  - 错。它是场上实例 ID
- 误解 2：web debugger 现在展示得差不多，就应该继续优先补 UI
  - 错。当前主要瓶颈仍然在 rules abstraction
- 误解 3：能靠一条特判做出来，就先做再说
  - 错。只有“明确是过渡切片，并且有回收路径”时才允许
- 误解 4：扩 20 张真实卡优先级高于补抽象
  - 错。没有抽象，卡越多，债越快爆炸

### 6.3 风险提示

- 如果未来有更多“按特定定义牌识别”的规则，而不尽快抽象成通用框架，`engine.go` 会开始堆很多局部 helper
- 如果 permanents 上场流程以后忘记填 `DefinitionID`，类似 `XQ22` 这类基于定义识别的规则会静默失效

---

## 7. 下一步任务

### 7.1 第一优先级

**任务**：把 `XQ22` 所在的 legality slice 抽象成更可复用的 prohibition helper
**目标**：避免继续堆 name/ID 特判，给 `XQ31 / XQ01` 铺路
**步骤**：
1. 抽出“按 `DefinitionID` 识别场上特定卡定义”的通用 helper
2. 明确 prohibition 的 source / scope / target category 数据结构
3. 保持现有 `XQ22` 测试全绿，再扩第二张卡

### 7.2 第二优先级

**任务**：推进 `B` 组剩余两张卡的最小可行规则
**目标**：评估 `XQ31 / XQ01` 哪部分能在不引入大系统的前提下先落一刀

### 7.3 第三优先级

**任务**：把 `DefinitionID` 接到未来 permanents / attachments 上场语义
**目标**：让卡牌定义身份不只存在于测试手工 state，而是进入正式生命周期

### 7.4 更远一段路的推荐路线

如果 TRAE 需要的不只是“下一刀”，而是后面几步的连续路线，我建议按下面这条线走，不要跳着做：

#### 路线 A：把 legality 身份模型做扎实

目标：
- 让 `DefinitionID`
- `basicType`
- prohibition / permission / target category

这几个概念形成最小稳定面。

产出：
- 一个最小通用 prohibition helper
- 1 到 2 张新增真实卡验证抽象没跑偏

#### 路线 B：补 rules abstraction，而不是先补卡数

目标：
- 处理 `XQ31 / XQ01` 所需的 target / scoped silence 能力
- 明确哪些属于通用层，哪些仍应留在单卡入口

产出：
- legality / targeting 的更清晰分层
- 更多“规则读状态，不读显示名”的测试

#### 路线 C：再回到 attachments / permanents lifecycle

目标：
- 给 `BQ022` 一类卡建立真实可持续的载体模型

最小需要：
1. permanents 进入 table 的正式表示
2. `attachedTo` / host 关系
3. host 离场后的 attachment 处理
4. attachment 离场后的 continuous cleanup

#### 路线 D：最后才扩大真实卡批次

目标：
- 在抽象够稳之后，再去做 15 到 20 张“语义优先”的真实卡

原则：
- 按语义覆盖面扩，而不是按卡名数量扩
- 每扩一批卡，都要求 fixture gate + rules test + 文档同步

### 7.5 一个更现实的 5-session 展望

如果交给新的 AI 连续做 5 个 session，我建议大致这样安排：

1. Session 1
   - 抽象 `XQ22` prohibition helper
   - 不再新增更多 name/ID 硬编码
2. Session 2
   - 探 `XQ31 / XQ01` 所需最小 targeting / scope 模型
   - 只做最便宜的一刀
3. Session 3
   - 把 `DefinitionID` 接进更正式的 permanents lifecycle 设计
   - 不急着全实现 attachment
4. Session 4
   - 选 5 到 8 张新的“语义优先”真实卡落地
5. Session 5
   - 回头整理 golden scenario / replay / invariants
   - 防止规则面扩完后没有护城河

这 5 步里，最重要的不是快，而是不要把系统带歪。

---

## 8. 测试命令

### 8.1 Go 测试

```bash
go test ./server/...
go test ./server/pkg/rules -run 'TestXQ22' -count=1
```

### 8.2 TypeScript 测试

```bash
cd tools/fixture-tools && npm test
cd web && npm test
```

### 8.3 启动服务

```bash
go run ./server/cmd/api
cd web && npm run dev
```

---

## 9. 重要文档索引

| 文档 | 路径 | 用途 |
|------|------|------|
| 项目总背景 | `README.md` | 项目目标、非目标、权威边界、测试纪律 |
| 文档目录说明 | `docs/README.md` | docs 目录职责 |
| 下一阶段规则核计划 | `docs/NEXT_GEN_RULE_PLAN.md` | 当前阶段状态与总路线图 |
| 下一步执行顺序 | `docs/NEXT_STEP_EXECUTION_PLAN_2026-03-31.md` | 为什么当前优先级不是 UI / 联机 / 大规模扩卡 |
| Live sandbox 说明 | `docs/LIVE_SANDBOX_2026-03-31.md` | 当前 sandbox 有什么、没有什么 |
| Go fixture 入口说明 | `docs/GO_CARD_FIXTURE_ENTRYPOINT_2026-03-31.md` | fixture catalog / normalized / queue_operation 入口 |
| 测试纪律 | `docs/TEST_PLAN.md` | 什么改动必须带测试 |
| 本次交接文档 | `docs/HANDOVER_TRAE_2026-04-01.md` | 给 TRAE 直接续做 |

### 9.1 推荐阅读后的行动顺序

读完上面的文档后，建议新 AI 立即做的不是“开始写代码”，而是先回答这 3 个问题：

1. 当前最关键的技术债是“禁止规则的抽象缺失”，还是“卡牌数量不够”？
2. 哪个下一步能最大化复用 `DefinitionID` / `basicType` 这轮新增的基础设施？
3. 哪些诱人的工作其实应该继续 defer？

如果这 3 个问题答不出来，说明 big picture 还没真正吃透，不该急着动手。

---

## 10. 交接人信息

- **交接人**：Codex
- **交接时间**：2026-04-01
- **联系方式**：当前仓库会话上下文

---

## 11. 附：最近提交记录

```text
21f4f23 feat(卡牌逻辑): 新增五张真实卡牌的测试用例和文档
7227dcd feat(rules): 实现持续效果来源离场自动清理功能
72c3ac2 Merge pull request #1 from FireExtin/codex/implement-formal-matchstate-and-session-reset
fc90784 test(web): 修复vitest绑定并补reset后可继续提交用例
47915c3 docs: 补充终局状态与sandbox重置进度说明
```

---
