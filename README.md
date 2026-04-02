# 隐秘世界 数字化项目

> 基于《隐秘世界》桌游的数字化实现项目。
> 技术栈固定为 **Go + TypeScript**。
> 本项目优先服务于 **vibe coding / AI 协作开发**，因此要求：架构边界清晰、测试优先、规则可回放、错误可解释、状态可投影。

---

# 1. 项目背景

我很喜欢《隐秘世界》这款桌游，但我没有卡牌类游戏开发经验。
这个项目的目标，不是一次性“把整个游戏做完”，而是先搭出一个**长期可维护、适合 AI 协作开发、适合逐步扩卡和补规则**的数字化基础工程。

这个 README 的角色不是宣传文案，而是：

* 项目背景说明
* 架构原则说明
* 技术选型依据
* 开发边界定义
* 测试与质量门槛
* Vibe coding 的工作准则

它会作为整个项目的背景和骨架，供人类开发者和 AI 编码工具共同参考。

## 最近里程碑（2026-03-31）

- 规则核已把“对局结束”提升为正式 `MatchState`，而不是仅依赖 `winner` 字段。
- Go sandbox 已支持 `POST /api/debugger/reset`，结束后可直接在同一会话重开一局。
- Web Live Debugger 已接入 `Reset Sandbox` 按钮，并在终局态禁用动作提交、显示胜者提示。
- 2026-04-03 起，sandbox session 额外引入显式 `SessionLifecycle`；前端动作表单改为读取 Go projection 下发的 `rulesMetadata.actionPolicies` 做 schema-driven 预校验，不再自己维护动作分支规则。
- 同日，当前资源池模型已被显式收口到 `PaymentEngine + PaymentModePrototype`；`TurnState.Resources` 仍是原型快照，不代表最终规则书支付模型。
- 相关细节与阶段状态请继续参考 `docs/NEXT_GEN_RULE_PLAN.md` 与 git 提交记录。

---

# 2. 现有公开资料与数据来源

当前已知的公开资料来源包括：

* 网页版查卡器主页：`https://ymsj-fun.github.io/cards/`
* 规则书与帮助文档入口：`https://ymsj-fun.github.io/帮助文档/2022/07/01/rulebooks.html`

这些资料将作为第一阶段的数据整理和规则整理基础来源。
项目早期会优先做：

* 卡牌数据归一化
* 规则术语表
* 最小可执行规则核
* 最小网页对局原型
* 调试与回放系统

---

# 3. 项目目标

本项目的第一目标不是“做一个看起来很像游戏的客户端”，而是做一台：

* **规则正确**
* **状态可回放**
* **错误可解释**
* **隐藏信息安全**
* **适合 AI 持续协作开发**

的《隐秘世界》规则机。

## 3.1 第一阶段目标

第一阶段只追求以下能力：

1. 导入并规范化现有卡牌与规则数据
2. 实现服务端权威裁决
3. 实现最小回合/阶段/步骤/堆叠/响应系统
4. 实现最小持续效果系统
5. 实现投影视图系统，保证隐藏信息安全
6. 实现最小可玩网页原型
7. 实现最小调试器和回放能力
8. 建立测试先行的开发机制

## 3.2 非目标

第一阶段不追求：

* 全部扩展一次性支持
* 所有卡牌一次性全实现
* 漂亮动画
* 移动端或桌面壳
* 自动平衡性分析
* AI 自动理解全部中文卡面并直接执行

---

# 4. 正式技术栈声明

本项目唯一正式技术栈为：

* **Go**：规则真相源、联机服务、裁决引擎、回放系统、最终合法性判断
* **TypeScript**：Web 前端、查卡器、构筑器、调试器、DSL 作者工具、契约测试辅助工具

此前讨论过的 “TypeScript-only” 路线，视为探索方案，不再作为执行依据。
从现在开始，项目中的所有正式设计、目录结构、提示词模板、测试规则，均以 **Go + TypeScript** 为准。

---

# 5. 核心设计原则

## 5.1 规则优先于界面

UI 不是规则来源。
规则真相源只存在于 Go 服务端。

客户端只能：

* 发送动作请求
* 展示服务端投影状态
* 展示日志与错误
* 读取 Go 下发的 `rulesMetadata` 做本地预校验与提示

客户端不能：

* 自行裁决
* 自行发明动作合法性规则
* 自行更新最终游戏状态
* 自行生成隐藏信息真相

---

## 5.2 所有状态变化都必须可回放

每次合法状态提交都必须留下：

* action log
* revision
* checkpoint（按策略）
* 投影视图
* 事件日志

我们默认未来一定会需要：

* 复现 bug
* 对局回放
* 断线恢复
* 审计
* 重演测试
* 回归测试

因此回放不是后期锦上添花，而是第一层架构约束。

---

## 5.3 所有复杂规则都必须进入同一裁决管线

不允许遇到复杂效果就临时写特判，把状态直接改掉。
所有规则都应尽量进入统一执行管线：

1. 玩家提交动作
2. legality 检查
3. 目标/模式/费用确认
4. 入堆叠或直接结算
5. 生成事件
6. 替代 / 防止处理
7. 提交状态
8. 触发收集
9. 清理与状态检查
10. 生成 revision 与投影

---

## 5.4 单元测试是第一公民

这是本项目的硬规则，不是建议。

### 强制要求

* 每次**较重的逻辑改动**，都必须伴随**充分的新单元测试**
* 所有新逻辑必须满足：

  * 新增测试通过
  * 以前的测试全部继续通过
* 没有测试的复杂逻辑改动，视为**未完成**
* 破坏已有测试而没有正当迁移说明的改动，不允许合并

### “较重的逻辑改动”包括但不限于

* 新增或修改 legality 逻辑
* 新增或修改持续效果层
* 新增或修改 trigger / replacement / prevention 行为
* 修改 projection 生成规则
* 修改 replay / revision 机制
* 修改 DSL 语义解释
* 修改错误协议
* 修改牌组合法性判定

### 这条规则的工程含义

我们默认：

* 规则代码会不断重构
* AI 会反复生成、替换、补丁式修改代码
* 如果没有测试作为第一公民，系统会很快腐化

因此，**测试不是补充材料，而是规则系统的一部分。**

---

# 6. 权威边界

## 6.1 Go 是唯一语义权威

Go 服务端负责：

* 动作合法性
* 目标合法性
* 费用合法性
* 堆叠与响应
* 持续效果重算
* 替代 / 防止 / 触发
* 状态提交
* revision 生成
* 视图投影生成
* 错误码生成
* 回放与恢复

## 6.2 TypeScript 不是最终裁判

TypeScript 负责：

* schema 定义与静态校验
* DSL 作者工具
* 构筑器前端
* 查卡器前端
* 对局界面
* 调试界面
* 回放界面
* 基于 Go 下发 metadata 的本地预校验
* fixture 管理辅助

### 一句话定义

**TS 决定“怎么写”，Go 决定“是什么意思”。**

---

# 7. 总体架构

```text
[ Web Client / TypeScript ]
        |
        | REST / WebSocket
        v
[ Go Game Server ]
  |- Match Service
  |- Rules Engine
  |- Stack Engine
  |- Priority Engine
  |- Projection Engine
  |- Continuous Effect Registry
  |- Replay / History Engine
  |- Deck Validation Service
        |
        v
[ PostgreSQL / Redis / Files ]
```

## 7.1 当前最小可运行形态

当前仓库已经具备一个**最小可运行 sandbox**：

* Go 服务端提供内存内单局 session
* Web 前端可以通过 HTTP 拉取协议 envelope，并提交预置动作
* 如果 Go sandbox 不可用，Web 会自动退回 mock protocol 调试模式

### 本地开发运行

1. 启动 Go sandbox：

   ```bash
   go run ./server/cmd/api
   ```

2. 启动 Web 开发服务器：

   ```bash
   cd web
   npm install
   npm run dev
   ```

3. 打开浏览器：

   ```text
   http://localhost:5173
   ```

Vite 会把 `/api` 代理到本地 Go 服务，默认目标是 `http://127.0.0.1:8080`。

### 最小部署运行

1. 构建前端：

   ```bash
   cd web
   npm install
   npm run build
   ```

2. 启动 Go 服务：

   ```bash
   go run ./server/cmd/api
   ```

3. 打开浏览器：

   ```text
   http://localhost:8080
   ```

当 `web/dist` 存在时，Go 服务会直接托管构建后的前端静态资源。

---

# 8. 状态模型与投影系统

---

## 8.1 三层状态

### FullState

* 完整真相状态
* 仅存在于 Go 服务端
* 包含所有隐藏信息
* 禁止直接发给客户端

### PlayerViewState

* 针对单个玩家生成
* 只包含该玩家当前可见信息
* 同一 revision 下，不同玩家拿到的视图可以不同

### SpectatorViewState

* 观战视图
* 默认不暴露隐藏信息
* 不是第一阶段重点，但接口要预留

---

## 8.2 投影生成时机

服务端只在**原子状态提交点**生成新 revision 和投影，不在中间临时态广播。

### 原子状态提交点

以下时刻允许提交并广播：

1. 一个动作通过合法性检查并正式入堆叠
2. 一个 stack item 完整结算结束
3. 一组同时触发能力按顺序入堆叠完成
4. 一个阶段或步骤切换完成
5. 一个地区争夺完成
6. 一次胜负检查完成
7. 一次隐藏信息的可见性发生合法变化

### 明确不广播的过程

以下过程只存在于服务端内部，不广播半成品状态：

* legality 中间过程
* cost 试算中间过程
* trigger 搜索中间态
* replacement 链上的临时状态
* 持续效果重算过程中的中间态

---

## 8.3 隐藏信息规则

所有隐藏信息只允许从 `FullState` 派生。
不允许客户端通过拼接日志、时间差、错误消息等旁路推理拿到不该知道的信息。

### 私密信息变化场景

以下场景发生时，必须重新按玩家分别投影：

* 暗藏者翻开
* 卡牌从隐藏区进入公开区
* 卡牌从公开区进入私有区
* 某玩家仅自己可见地检视一张牌
* 某效果让特定玩家获得临时私密信息

---

# 9. 规则执行模型

## 9.1 统一执行管线

```text
SubmitAction
-> Legality Check
-> Choose Target / Mode / Cost
-> Build Operation
-> Put On Stack / Resolve Directly
-> Build Event
-> Replacement / Prevention
-> Commit State
-> Collect Triggers
-> Cleanup / State Check
-> Recalculate Continuous Effects
-> Generate Revision
-> Generate Per-Player Views
-> Broadcast
```

---

## 9.2 Hook / Modifier 分层

为了避免规则腐化，所有复杂规则尽量进入这五层：

### 1) Legality Hooks

判断能不能做：
核心冲突原则：在任何合法性检查中，“禁止/不能（Prohibition）”效果永远绝对优先于“允许/许可（Permission）”效果。

* CanPlayCard
* CanActivateAbility
* CanTarget
* CanPayCost

### 2) Modifier Hooks

改数值：

* ModifyCost
* ModifyPower
* ModifyInvestigation
* ModifyInfluence

### 3) Replacement / Prevention Hooks

替代与防止：

* ReplaceDestroy
* PreventDamage
* ReplaceDiscard

### 4) Trigger Hooks

收集触发：

* OnEnter
* OnReveal
* OnRegionWon
* OnDestroyed

### 5) Cleanup Hooks

系统后处理：

* 步骤结束
* 地区重整
* 状态检查
* 胜负检查
* 回合切换

---

# 10. 持续效果系统（最小版本）

这是第一阶段必须设计但不追求过度复杂化的系统。

## 10.1 最小结构

```go
type ContinuousEffect struct {
    ID            string
    SourceID      string
    Layer         ContinuousLayer
    Timestamp     int64
    Duration      DurationSpec
    DependencyKey []string
    Applies       func(state *GameState, obj ObjectRef) bool
    Modify        func(state *GameState, obj ObjectRef, acc ModifierAccumulator) ModifierAccumulator
}
```

---

## 10.2 最小 layer

```go
type ContinuousLayer int

const (
    LayerProhibition ContinuousLayer = iota
    LayerPermission
    LayerCost
    LayerNumeric
    LayerActionQuota
)
```

### 规则

* 禁止优先于许可
* 同层按 timestamp
* dependency 先只预留 key，不在第一阶段做完整复杂系统

---

## 10.3 重算入口

当发生以下事件时，需要触发持续效果重算：

* 物件进出场
* 卡牌翻面
* 控制权变化
* 阶段切换
* 回合切换
* 持续效果开始
* 持续效果结束
* 某对象特征变化

---

## 10.4 防循环与幂等约束

这是本项目的强约束，必须写进实现要求。

### 规则

1. **持续效果重算必须是幂等的**
2. **每次 state commit 内，最多执行一次完整持续效果重算**
3. 重算过程必须从同一个明确的 base state 出发
4. 重算结果必须在有限步内收敛
5. 如果本次提交中持续效果重算已经执行过，不得因其自身副作用再次递归触发第二轮完整重算
6. 如确需延迟到下一轮处理，必须通过显式队列而不是递归重入

### 一句话

**持续效果重算是“提交后派生计算”，不是“可以无限重入的事件链”。**

---

# 11. DSL、Schema 与契约测试

---

## 11.1 DSL 边界

我们不做“直接从中文卡面自动解释执行”。
正确做法是：

* 用 TypeScript 定义 schema 和作者工具
* 导出规范化 JSON
* 由 Go 服务端作为最终语义解释者执行

---

## 11.2 Schema 版本策略

这是必须写明的长期约束。

### 规则

1. schema 使用语义化版本号（semver）
2. minor 版本必须尽量向后兼容
3. major 版本允许 breaking change，但必须配套迁移脚本
4. fixture、contract tests、回放文件都必须携带 schema version
5. 回放系统在读取历史文件时，必须先检查版本并决定：

   * 直接读取
   * 自动迁移
   * 显式拒绝并提示需要迁移

### 为什么需要这个

因为未来扩展很可能引入：

* 新关键词
* 新 card field
* 新 duration 种类
* 新 target 模式
* 新 ability 类型

如果没有版本策略，第一次 schema 破坏性升级就会把 fixture、回放和旧数据全打碎。

---

## 11.3 契约测试目录

```text
/shared/contracts
  /fixtures
  /expectations
```

每个 fixture 至少包含：

```json
{
  "cardId": "TEST001",
  "schemaVersion": "0.1.0",
  "input": {},
  "expectations": {
    "parseOk": true,
    "targetKinds": ["region"],
    "requiresStack": true,
    "speed": "fast",
    "durationKind": "turn",
    "scriptId": null
  }
}
```

---

## 11.4 契约测试权责

### TypeScript 侧

负责：

* schema 校验
* normalized JSON 输出
* fixture 维护工具
* Vitest 测试

### Go 侧

负责：

* 读取同一 fixture
* 解析语义
* 执行前置检查
* Go 原生 `testing` 测试

---

## 11.5 Fixture 维护规则

这是强约束。

### 规则

1. 每张新卡要进入主卡池之前，必须先有对应 contract fixture
2. fixture 必须通过：

   * TypeScript / Vitest
   * Go / 原生 `testing`
3. fixture 不通过，不允许合并主卡池变更
4. DSL 语义修改时，必须同步更新对应 fixture 与 expectation
5. breaking change 必须显式更新 schemaVersion 和迁移说明

### 一句话

**fixture 不是文档附件，而是主卡池准入门槛。**

---

# 12. 错误处理协议

## 12.1 LegalityResult

合法性检查绝不能只返回 bool。

```go
type LegalityResult struct {
    OK         bool
    ReasonCode string
    MessageKey string
    Hook       string
    Context    map[string]any
}
```

---

## 12.2 错误码规则

统一使用机器可读错误码：

```text
LEGALITY_FAILED_*
COST_FAILED_*
TARGET_FAILED_*
STACK_FAILED_*
RULES_FAILED_*
PROTOCOL_FAILED_*
```

前端逻辑只认 `code`，不能依赖自然语言文本。

---

## 12.3 WebSocket 错误响应

```json
{
  "type": "ActionRejected",
  "matchId": "m_123",
  "clientActionId": "a_456",
  "revision": 42,
  "error": {
    "code": "LEGALITY_FAILED_CANNOT_TARGET_HIDDEN_ENEMY",
    "messageKey": "cannot_target_hidden_enemy",
    "hook": "CanTarget",
    "context": {
      "cardId": "JC001",
      "targetId": "obj_42"
    }
  }
}
```

### 用途

* 前端按钮禁用提示
* 调试器展示失败原因
* 自动测试断言 reason code
* 回放器记录非法动作尝试

---

# 13. 回放、Revision 与历史系统

所有状态提交都必须产生新的 revision。

## 13.1 必备内容

* `revision`
* `actionLog`
* `checkpoint`
* `publicEvents`
* `privateEvents`
* `viewState`
* `rngSeed / rngState`（保证随机事件的可重演性）

## 13.2 目标

* 能重演
* 能断线恢复
* 能审计
* 能定位 bug
* 能做 golden tests

---

# 14. 最小调试器前置

完整调试器可以后做，但最小调试器必须跟 Phase 1 一起交付。

## 14.1 Phase 1 必须有的调试信息

* stack dump
* action log
* current revision
* current phase / step
* priority 状态
* pending triggers
* legality failure log
* 当前 per-player view 摘要

## 14.2 初版形式

* 简单 React 页面
* JSON dump
* 文本日志面板

不要求好看，但必须可用。

---

# 15. 测试原则（必须严格执行）

这部分是整个项目最重要的工程纪律之一。

## 15.1 测试是第一公民

测试优先于功能堆积。
没有测试的规则代码，默认不可信。

## 15.2 变更规则

### 每次较重逻辑改动，必须满足：

1. 新增测试覆盖新逻辑
2. 旧测试全部继续通过
3. 如旧测试不再成立，必须：

   * 给出原因
   * 更新设计说明
   * 更新迁移说明
   * 修改对应 golden/fixture

### 禁止行为

* 为了让 CI 通过而删除旧测试
* 把失败测试简单改成弱断言
* 先改逻辑、后想测试
* 把复杂规则只放在手测里，不写自动化测试

---

## 15.3 测试层级

### Go 侧

使用 **Go 原生 `testing` 包**，不引入第三方测试框架。
对于持久化或网络层的依赖，统一使用 Go 原生 interface 进行依赖注入和手工 Mock，禁止引入额外的 Mock 生成库。
包含：

* 单元测试
* 集成测试
* 回放测试
* golden tests
* 不变量测试
* 协议测试
* projection 测试
* 持续效果幂等测试

### TS 侧

统一使用 **Vitest**。

包含：

* schema 测试
* fixture 测试
* view-model 测试
* 组件测试
* 协议解码测试

---

## 15.4 必测主题

以下内容必须有明确测试：

* legality 判定
* target 判定
* cost 修正
* stack 结算顺序
* priority / pass 逻辑
* trigger 收集顺序
* replacement / prevention 冲突
* projection 隐藏信息隔离
* revision 单调递增
* replay 一致性
* continuous effect 重算幂等
* DSL contract fixtures 双端一致性
* deck legality

---

## 15.5 PR / 合并门槛

以下情况不得合并：

* 新增复杂逻辑但没有新增测试
* fixture 不通过
* Go 测试失败
* Vitest 失败
* 旧 golden tests 破坏且无说明
* 修改 schema 但没有版本/迁移说明
* 修改持续效果系统但没有幂等和防循环测试

---

# 16. 里程碑

## Phase 0：资料、Schema、Fixture、契约测试

交付：

* `cards.raw.json`
* `cards.normalized.json`
* `keywords.json`
* `card-schema.ts`
* `schema version policy`
* `contract fixtures`
* `TS / Go 双端 contract tests`

验收：

* TS 与 Go 对同一 fixture 解释一致
* fixture 成为主卡池准入门槛
* schema 有版本号

---

## Phase 1：最小规则核 + 最小调试器

交付：

* `GameState`
* `Action / Operation / Event`
* `StackEngine`
* `PriorityEngine`
* `ProjectionEngine`
* `ContinuousEffectRegistry`
* `HistoryState`
* `LegalityResult`
* `error protocol`
* `debug panel v0`

验收：

* 每次 commit 产生 revision
* 每次 commit 生成 per-player projection
* 非法动作返回 reason code
* stack / trigger / priority 可查看
* 持续效果重算满足幂等约束
* 重逻辑改动均伴随单元测试

---

## Phase 2：网页对局骨架

交付：

* React 对局页
* WebSocket 同步
* 手牌区 / 场地区 / 地区区 / 堆叠区
* 日志页
* 错误提示
* 调试入口

验收：

* 两个浏览器可连进同一局
* 刷新后可恢复最新 revision

---

## Phase 3：地区争夺、胜负与第一批卡池

交付：

* 地区争夺流程
* 得分系统
* 胜利判断
* 第一批持续效果卡
* 第一批 trigger / replacement 卡
* replay 可重演

验收：

* 能完成最小完整对局
* 旧测试全部保持通过
* 新规则均有对应测试

---

## Phase 4：构筑器 + 增强调试器

交付：

* deck builder
* legality 可视化
* projection diff
* trigger / replacement trace
* fixture 扩充
* golden tests 扩充

---

## Phase 5：联机稳定化

交付：

* 房间系统
* 断线重连
* 心跳与超时
* 回放存档
* 审计日志
* 对局恢复

---

# 17. 仓库结构建议

```text
/ymsj-digital
  /server
    /cmd
    /internal
    /pkg
    go.mod

  /web
    package.json
    /src

  /shared
    /contracts
      /fixtures
      /expectations
    /schemas
    /protocol

  /tools
    /card-importer
    /schema-validator
    /fixture-tools
    /replay-inspector

  /docs
    ARCHITECTURE.md
    GAME_FLOW.md
    WS_PROTOCOL.md
    CARD_DSL.md
    TEST_PLAN.md
    SCHEMA_VERSIONING.md
```

---

# 18. Vibe Coding 约束

这个项目是为 AI 协作开发准备的，所以必须约束任务颗粒度。

## 18.1 好任务

* 实现 PriorityEngine，并写 8 个 Go `testing` 单元测试，并确保所有非法状态转移都返回标准化的 ReasonCode​。
* 实现 ProjectionEngine，并写隐藏信息隔离测试
* 建立 5 个 DSL fixture，并分别补齐 Vitest 与 Go `testing`
* 实现 ContinuousEffectRegistry，并写幂等与防循环测试

## 18.2 坏任务

* 把整个隐秘世界做完
* 把所有卡牌都实现
* 随便先做个能玩的版本再说
* 先别管测试，后面补

---

# 19. 对 AI 代码生成的统一要求

每次让 AI 生成代码时，默认都要包含：

1. 类型定义
2. 最小实现
3. 失败路径
4. 单元测试
5. 非目标说明
6. 边界条件说明

### Go 侧统一要求

* 使用 Go 原生 `testing`
* 不引入第三方测试框架
* 不允许依赖数据库或 websocket 才能跑核心规则测试

### TS 侧统一要求

* 使用 Vitest
* 组件和 schema 测试都保持一致
* 不为小模块引入多套测试工具

---

# 20. 最终工程立场

这个项目的第一优先级不是“快做一个像游戏的东西”，而是：

**先做一台可裁决、可回放、可解释、可测试、可隐藏信息隔离的规则机。**

只有这台规则机立住了：

* 网页前端才不会变成临时演示壳
* 扩卡才不会变成 if/else 灾难
* AI 协作才不会越做越乱
* 联机与回放才有真正的地基
