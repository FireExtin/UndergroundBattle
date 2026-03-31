# 阶段 1：项目骨架、共享协议、测试底座

```text
你正在为《隐秘世界》数字化项目初始化正式工程骨架。

背景：
- 项目 README.md 已经存在，并明确规定技术栈是 Go + TypeScript
- Go 是唯一规则语义权威
- TypeScript 负责 web 前端、调试器、DSL 作者工具
- 单元测试是第一公民
- 每次较重的逻辑改动都必须新增测试，并且所有旧测试必须继续通过
- Go 测试统一使用原生 testing，不引入第三方测试框架
- TS 测试统一使用 Vitest
- 不允许为了让测试通过而删除旧测试或弱化断言

当前已存在资源目录：
- organized_content/cards
- organized_content/rules
- organized_content/tokens
- resource/ymsj-fun.github.io/

请你完成以下任务：

1. 建立推荐的仓库目录结构：
   - /server
   - /web
   - /shared/contracts/fixtures
   - /shared/contracts/expectations
   - /shared/schemas
   - /shared/protocol
   - /shared/protocol/messages
   - /shared/protocol/events
   - /shared/protocol/views
   - /tools/card-importer
   - /tools/schema-validator
   - /tools/fixture-tools
   - /docs

2. 初始化 Go 工程：
   - go.mod
   - server/cmd/api
   - server/internal
   - server/pkg

3. 初始化 TypeScript web 工程：
   - Vite + React + TypeScript
   - Vitest
   - 基础 src 目录结构

4. 建立共享协议文件：
   - shared/protocol/messages.schema.json
   - shared/schemas/card.schema.json
   - shared/schemas/fixture.schema.json

5. 建立最小测试底座：
   - Go 侧：使用原生 testing，写一个最小示例测试
   - TS 侧：使用 Vitest，写一个最小示例测试

6. 写 docs/TEST_PLAN.md 初稿，明确以下规则：
   - 测试是第一公民
   - 较重逻辑改动必须新增测试
   - 旧测试必须继续通过
   - fixture 是主卡池准入门槛
   - Go 统一使用原生 testing
   - TS 统一使用 Vitest
   - 不允许删除旧测试或弱化断言来“修复”CI

7. 生成 docs/SCHEMA_VERSIONING.md 初稿，声明以下三条原则：
   - schema 使用 semver
   - minor 版本尽量向后兼容
   - major 版本需要迁移脚本

输出要求：
- 直接生成目录和文件内容
- 不要引入第三方 Go 测试框架
- 不要实现业务逻辑，只做项目骨架和测试底座
- 每个生成文件都要有简短注释说明用途
```

***

# 阶段 2：资源归一化

```text
你正在为《隐秘世界》数字化项目实现“资源归一化阶段”。

背景：
- 项目已有 organized_content/cards、organized_content/rules、organized_content/tokens
- 还有 resource/ymsj-fun.github.io/ 下的原始抓取内容
- 目标不是网页展示，而是把这些资源整理为后续规则引擎和前端共用的数据资产
- Go 是语义权威，但这一步主要在 TypeScript tools 侧完成
- 单元测试是第一公民
- 每次较重的逻辑改动都必须新增测试，并且所有旧测试必须继续通过
- Go 测试统一使用原生 testing，不引入第三方测试框架
- TS 测试统一使用 Vitest
- 不允许为了让测试通过而删除旧测试或弱化断言

请你实现：

1. tools/card-importer：
   - 扫描 organized_content/cards、organized_content/rules、organized_content/tokens
   - 生成中间数据
   - 输出到 shared/schemas 或 data 目录中的 normalized JSON

2. 设计最小统一数据格式：
   - CardPrint
   - RuleDocMeta
   - TokenMeta

3. CardPrint 必须包含以下字段：
   - id
   - schemaVersion: string
   - name
   - category
   - sourcePath
   - text/rawText（按你的设计保留）
   - 其他你认为后续规则引擎必要的基础字段

4. 生成以下文件：
   - cards.raw.index.json
   - cards.normalized.json
   - rules.index.json
   - tokens.index.json

5. 写 TypeScript 校验器：
   - 校验必须字段
   - 校验 ID 唯一性
   - 校验 schemaVersion 存在
   - 校验 category / keyword / cost 等基础字段格式

6. 缺少 schemaVersion 的记录必须报结构化错误：
   - code: SCHEMA_VERSION_MISSING
   - 不允许静默通过
   - 不允许自动填默认值掩盖问题

7. 写 Vitest 测试：
   - importer 最小样例测试
   - schema 校验测试
   - 重复 ID 失败测试
   - 缺字段失败测试
   - 缺少 schemaVersion 返回 SCHEMA_VERSION_MISSING 的测试

8. 生成 docs/DATA_PIPELINE.md：
   - 说明原始资源到 normalized JSON 的转换流程
   - 说明后续 DSL 和规则引擎只依赖 normalized 数据，不直接依赖原始 HTML 或杂散文件

约束：
- 先不要尝试自动理解全部中文卡面语义
- 先不要解析复杂卡牌效果
- 先不要接数据库
- 所有测试统一使用 Vitest

输出要求：
- 直接实现 importer 和测试
- 给出清晰的文件落点
- 给出后续阶段可复用的 normalized schema
```

***

# 阶段 3：DSL fixture 和双端契约测试

```text
你正在为《隐秘世界》数字化项目实现 DSL contract fixtures 和双端契约测试。

背景：
- README 已明确：TS 负责 schema 和作者工具，Go 负责最终语义解释
- fixture 是主卡池准入门槛
- 每张新卡进入主卡池前，必须先有 contract fixture
- 单元测试是第一公民
- 每次较重的逻辑改动都必须新增测试，并且所有旧测试必须继续通过
- Go 测试统一使用原生 testing，不引入第三方测试框架
- TS 测试统一使用 Vitest
- 不允许为了让测试通过而删除旧测试或弱化断言

请你完成以下任务：

1. 设计最小 CardLogic DSL schema，至少支持：
   - id
   - schemaVersion
   - speed
   - targetKinds
   - requiresStack
   - durationKind
   - scriptId
   - basic effects

2. 在 shared/contracts/fixtures 下创建至少 5 个 fixture，分别覆盖：
   - 单目标地区牌
   - 无目标即时效果
   - 需要入堆叠的快速效果
   - 带持续时间的本回合修正
   - 使用 scriptId 的复杂特殊牌

3. 每个 fixture 必须包含：
   - cardId
   - schemaVersion
   - input
   - expectations

4. schemaVersion 必须与 shared/schemas/card.schema.json 中声明的当前版本一致

5. TypeScript 侧：
   - 写 schema 校验
   - 写 fixture 读取器
   - 写 Vitest 契约测试
   - 输出 normalized JSON

6. Go 侧：
   - 读取相同 fixture
   - 解析出对应结构
   - 写原生 testing 契约测试
   - 断言 speed / targetKinds / requiresStack / durationKind / scriptId 与 expectation 一致

7. 当 fixture 的 scriptId 非 null 时，Go 解析结果必须：
   - 标记 requiresScript: true
   - 不得尝试把该卡当作纯 DSL 可执行卡处理

8. 新增 docs/CARD_DSL.md：
   - 明确 TS 不是语义权威
   - 明确 Go 是最终解释者
   - 说明 fixture 的准入门槛角色

强制要求：
- Go 测试统一使用原生 testing
- TS 测试统一使用 Vitest
- fixture 不通过则视为该 DSL 样例不可用
- 不要引入业务 UI
- 不要开始写完整规则引擎

输出要求：
- 直接创建 fixture、测试和文档
- 所有测试可本地直接运行
```

***

# 阶段 4：最小 Go rules core

```text
你正在为《隐秘世界》数字化项目实现 Go 最小规则内核。

背景：
- Go 是唯一规则语义权威
- 规则机优先于 UI
- 所有状态变化都必须可回放
- 单元测试是第一公民
- 每次较重的逻辑改动都必须新增测试，并且所有旧测试必须继续通过
- Go 测试统一使用原生 testing，不引入第三方测试框架
- TS 测试统一使用 Vitest
- 不允许为了让测试通过而删除旧测试或弱化断言

请实现以下模块：

1. GameState
2. Action
3. Operation
4. Event
5. Revision
6. HistoryState
7. 最小 TurnState / PhaseState
8. 最小 BoardState
9. RNGState（种子化随机状态，保证 replay 可重演）

同时实现一条最小执行管线：
SubmitAction
-> Legality Check
-> Build Operation
-> Put On Stack / Resolve Directly
-> Build Event
-> Commit State
-> Generate Revision

请先不要实现完整卡牌效果，只做最小骨架。

要求：
- 所有核心逻辑为纯函数或接近纯函数
- 不依赖数据库
- 不依赖 websocket
- 不依赖 UI
- 所有状态可序列化
- 所有 commit 都必须生成 revision

测试要求：
- 使用 Go 原生 testing
- 至少实现以下测试：
  1. 合法动作可以进入执行管线
  2. 非法动作会被拒绝
  3. revision 单调递增
  4. state commit 后会记录 action log
  5. 同一最小 action log 可以重放到同一状态
  6. 使用相同 RNG seed 的两次执行，产生相同的随机结果

输出要求：
- 直接给出 Go 代码
- 不要跳过测试
- 不要实现复杂效果
- 不要写假测试
```

***

# 阶段 5：Priority、Stack、错误协议

```text
你正在为《隐秘世界》数字化项目扩展 Go 规则内核，加入 Priority、Stack 和标准错误协议。

背景：
- 已有最小 GameState、Action、Operation、Event、Revision 骨架
- 当前目标是把卡牌游戏最核心的“响应窗口”和“错误可解释性”建立起来
- 单元测试是第一公民
- 每次较重的逻辑改动都必须新增测试，并且所有旧测试必须继续通过
- Go 测试统一使用原生 testing，不引入第三方测试框架
- TS 测试统一使用 Vitest
- 不允许为了让测试通过而删除旧测试或弱化断言

请实现以下内容：

1. PriorityState：
   - 当前行动权归属
   - pass 计数
   - 双方连续 pass 的处理

2. StackEngine：
   - stack item 入栈
   - 后进先出结算
   - stack 顶结算

3. LegalityResult：
   - OK
   - ReasonCode
   - MessageKey
   - Hook
   - Context

4. 标准错误码体系：
   - LEGALITY_FAILED_*
   - TARGET_FAILED_*
   - COST_FAILED_*
   - STACK_FAILED_*
   - RULES_FAILED_*

5. 最小协议结构：
   - ActionAccepted
   - ActionRejected
   - StatePatched

测试要求（Go 原生 testing）：
至少覆盖以下场景：
1. 一方 pass 后优先权转移
2. 双方连续 pass 且 stack 非空时，结算 stack 顶
3. 双方连续 pass 且 stack 为空时，进入步骤结束
4. 非法动作返回 machine-readable reason code
5. stack 结算顺序严格后进先出
6. 新增逻辑不破坏既有 revision / action log 测试
7. 标准行动在对方拥有行动权时，legality 检查必须返回 LEGALITY_FAILED_NOT_YOUR_PRIORITY
8. 标准行动在 stack 非空时，legality 检查必须返回 LEGALITY_FAILED_STACK_NOT_EMPTY

输出要求：
- 直接实现 Go 代码和测试
- 不要引入 websocket server
- 不要引入前端
- 错误必须是结构化的，而不是纯文本
```

***

# 阶段 6：Projection 和隐藏信息隔离

```text
你正在为《隐秘世界》数字化项目实现 ProjectionEngine 和隐藏信息隔离。

背景：
- Go 是唯一 FullState 真相源
- 客户端永远不能直接拿到 FullState
- 同一 revision 下，不同玩家可以拿到不同的 PlayerViewState
- 投影只允许在原子 state commit 完成后生成，不能在 legality 检查或其他中间态生成
- 单元测试是第一公民
- 每次较重的逻辑改动都必须新增测试，并且所有旧测试必须继续通过
- Go 测试统一使用原生 testing，不引入第三方测试框架
- TS 测试统一使用 Vitest
- 不允许为了让测试通过而删除旧测试或弱化断言

请实现：

1. FullState
2. PlayerViewState
3. SpectatorViewState
4. ProjectionEngine

要求：
- FullState 不得直接序列化给客户端
- PlayerViewState 必须按玩家单独生成
- 同一 revision 下允许 view[playerA] != view[playerB]
- 已公开信息对双方一致可见
- 隐藏信息不得通过 view 泄露

至少覆盖以下场景测试（Go 原生 testing）：
1. 自己的隐藏牌可见，对手不可见
2. 某张牌翻开后，下一次 commit 后双方都能看到公开信息
3. 某玩家仅自己检视一张牌后，对手视图仍不可见
4. SpectatorViewState 默认不暴露隐藏信息
5. projection 生成不破坏 revision 和 replay 逻辑
6. 在 legality 检查过程中，不得触发 projection 生成；projection 只在 state commit 完成后生成

输出要求：
- 直接实现 ProjectionEngine 代码和测试
- 不要实现复杂 UI
- 不要引入数据库
```

***

# 阶段 7：持续效果最小系统

```text
你正在为《隐秘世界》数字化项目实现最小 Continuous Effect 系统。

背景：
- 当前项目已经有最小规则核、stack、priority、projection
- 现在要实现最小持续效果系统，但不追求一步到位的完整 dependency engine
- 项目要求：持续效果重算必须幂等；每次 state commit 内最多执行一次完整重算
- 单元测试是第一公民
- 每次较重的逻辑改动都必须新增测试，并且所有旧测试必须继续通过
- Go 测试统一使用原生 testing，不引入第三方测试框架
- TS 测试统一使用 Vitest
- 不允许为了让测试通过而删除旧测试或弱化断言

请实现：

1. ContinuousEffect
2. ContinuousLayer
3. ContinuousEffectRegistry
4. RecalculateContinuousEffects(state) 机制

最小 layer：
- LayerProhibition
- LayerPermission
- LayerCost
- LayerNumeric
- LayerActionQuota

规则要求：
- 禁止优先于许可
- 同层按 timestamp
- 每次 state commit 内最多完整重算一次
- 重算必须幂等
- 重算不得无限递归
- 必须显式防循环

保留以下后续扩展接口（现在不实现完整逻辑，但接口必须存在）：
- DependencyKey []string（用于未来 dependency tie-break）
- Timestamp int64（用于同层排序）
- ResolveConflict(a, b ContinuousEffect) *ContinuousEffect（现在可以返回 nil，表示暂不实现）

至少实现以下测试（Go 原生 testing）：
1. 数值修正生效
2. 禁止优先于许可
3. 本回合持续效果到期后失效
4. 同一 commit 内重复触发重算请求时，只执行一次完整重算
5. 重算幂等：相同输入 state，重复运行结果一致
6. 防循环：持续效果开始/结束不会导致无限递归重算

输出要求：
- 直接实现 Go 代码和测试
- 不要引入完整复杂 dependency graph
- 保留后续扩展接口
```

***

# 阶段 8：最小 Web 调试器和对局骨架

```text
你正在为《隐秘世界》数字化项目实现最小 Web 调试器和对局骨架。

背景：
- Go 服务端已经具备最小 rules core、stack、priority、projection、continuous effect 基础能力
- TypeScript 前端不是裁判，只是展示、调试和提交动作
- 单元测试是第一公民
- 每次较重的逻辑改动都必须新增测试，并且所有旧测试必须继续通过
- Go 测试统一使用原生 testing，不引入第三方测试框架
- TS 测试统一使用 Vitest
- 不允许为了让测试通过而删除旧测试或弱化断言

请实现一个最小 React + TypeScript 前端，包括：

1. 对局基础页面
2. stack 面板
3. action log 面板
4. current revision 显示
5. current phase / step 显示
6. priority 状态显示
7. legality failure 显示
8. 一个 mock 的 per-player view 切换器

要求：
- 使用 Vite + React + TypeScript
- 使用 Vitest
- 先用 mock protocol data，不必真实接 websocket
- mock data 的结构必须严格遵守 shared/protocol/messages.schema.json 中定义的协议格式，不得自行创造新的字段结构
- 状态管理只使用 React 原生 useState / useReducer
- 不引入 Redux、Zustand 或其他外部状态管理库
- UI 可以简陋，但结构清晰
- 不要写花哨样式

测试要求（Vitest）：
1. stack 面板渲染顺序正确
2. action log 正确展示
3. revision 显示正确
4. legality failure 可展示结构化错误
5. 玩家视图切换时隐藏信息展示不同

输出要求：
- 直接给出 React 组件和测试
- 不要实现美术资源
- 不要引入复杂状态管理库
```

<br />

<br />

- 地区争夺 现在已经能跑“调查放影响力 -> 决定地区控制者 -> 回合结束得分 -> 达到阈值胜利”，核心在 region.go 和 role\_actions.go。\
  如果你说的“世界争夺”是比地区更高一层的全局控制系统，那现在没有，当前只有 region 这一层，见 projection.go。
- 秘社标记物 现在没有 authoritative state、没有 action、没有 projection。只有 schema 里预留了 secretSociety 目标类型，以及导卡数据里有 society 元数据，见 card-logic.schema.json 和 fixture.schema.json。这还不算机制落地。
- 弃牌堆/坟墓 现在有最小语义：discard 是正式 zone，角色受到致命伤害会从 table 进 discard，见 projection.go 和 continuous.go (line 312)。\
  但它还不是完整“坟墓系统”：没有从弃牌堆检索、回收、复活、计数、以坟墓为 cost/target 的正式机制。

我会这样排顺序：

1. Asset / Permanent Model V1
2. Discard / Graveyard Semantics V1
3. Attachment / Host Lifecycle V1
4. Secret Society Marker V1
5. 暗藏部署 / 现身 V1
6. 快 / 闪电 / 反应时机 V2
7. 战斗对抗 V2
8. 调查 / 地区争夺 V2

理由很简单：

- 现在最缺的是“真实 permanent 骨架”。资产、附属、暗藏部署都建立在这上面。
- discard 应该比附属更早补强，因为所有 permanent 离场最终都要落到统一 zone 语义上；否则后面很多牌会反复返工。
- 附属要等 permanent + discard 稳了再做，不然还是特判。
- 秘社标记物本质是一个新状态子系统，最好在 zone / permanent 生命周期稳定后再接，不然容易和后续资产、地区、隐藏信息模型打架。
- 战斗、调查、地区争夺现在已经有 V1，先不急着做复杂版；否则是在不稳的底座上堆复杂度。
- 第一优先该做的是 `资产 / permanent`，因为它是缺的底座。现在规则核里真正稳定的在场实体主要还是角色和地区，见 [projection.go](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/projection.go) 和 [role\_actions.go](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/role_actions.go)。而你提到的资产、附属、隐藏部署，本质都要求“卡牌作为真实在场永久物存在，并有明确的进场/在场/离场语义”。这层不先补，后面全会变成特判。
- 第二做 `附属 / host lifecycle`，因为它直接建立在 permanent 模型上，而且你们已经被 `BQ022` 这类牌逼到边缘了。当前只有 attachment tracking V0，不是完整附属系统，见 [attachment.go](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/attachment.go) 和 [continuous.go](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/continuous.go)。这一步做完，装备、结附、宿主离场、附属离场这些才会进入正式生命周期。
- 第三再做 `暗藏部署 / 现身`。原因不是它不重要，而是它风险最高：它同时碰 permanent state、hidden-info projection、legality、replay。现在你们有 hidden-info 和 `reveal_card`，但还没有“隐藏永久物部署”系统，见 [engine.go](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/engine.go) 和 [m0-sandbox.md](/Users/ddd/Downloads/UndergroundBattle/docs/milestones/m0-sandbox.md)。太早做，很容易把 projection 边界搞脏。
- 第四才是把 `快 / 闪电 / 反应时机` 从最小版拉到 V2。当前 `speed + window + stack` 已经有最小可用形态，见 [engine.go](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/engine.go#L981) 和 [GO\_PHASE\_PRIORITY\_WINDOWS\_2026-03-31.md](/Users/ddd/Downloads/UndergroundBattle/docs/GO_PHASE_PRIORITY_WINDOWS_2026-03-31.md)。所以它不是零，但也还不值得现在深挖到完整时机学。等 permanent / attachment / hidden deploy 有了，再补 timing 更稳。
- 第五、第六再补 `战斗 / 调查 / 势力对抗` 的完整版本。不是因为它们不重要，而是因为现在已经有最小闭环：
  - 攻击：见 [role\_actions.go](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/role_actions.go#L69)
  - 调查：见 [role\_actions.go](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/role_actions.go#L108)
  - 地区控制与得分：见 [region.go](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/region.go)

所以这三块当前是“能跑但不完整”，不是“完全没有”。在 permanent/timing 没稳前，过早把它们做复杂，后面大概率还要重改。

**一句话判断**

现在最该做的不是继续补“对抗花样”，也不是急着做“暗藏部署”；而是先把 `资产 / permanent / attachment` 这条状态模型骨架做出来。\
如果只能选下一刀，我会选：`Asset / Permanent Model V1`。
