# 下一阶段规则核计划

## Summary
- 当前已经有：最小规则核、priority/stack、projection、continuous effects、`inspect` 的 permission hook、`dealDamage` 对 `EffectiveStats.Defense` 的最小致命判定，以及第一批角色动作入口 `declare_attack / declare_investigation`。
- 如果目标是“继续稳定接真实卡 DSL，并形成一个可扩展 alpha”，还差 4 个明确里程碑。做完这 4 个就够继续推进主卡池，不需要先上完整 dependency engine。

## PLAN_NEXT（本轮延后项）

### PN-BASE-001
- ID：`PN-BASE-001`
- 延后原因：本轮目标是先打通“基础包可玩闭环”，费用子系统会显著扩大建模和前后端联调范围。
- 触发条件：开始实现资源池/费用支付与费用不足拒绝路径时。
- 验收标准：`play_card/queue_operation` 必须在支付成功后才可生效；费用不足返回稳定 reasonCode；包含正反例单测与 e2e。
- 依赖：资源状态模型、费用 DSL 字段规范、动作 legality 扩展。
- 当前状态（2026-04-02）：`play_card` 已落地基础费用校验与扣费；`queue_operation` 仍在调试通道，未并轨费用体系。

### PN-BASE-002
- ID：`PN-BASE-002`
- 延后原因：当前原型优先验证战斗主回路，暂时放开构筑限制以提升可玩调试效率。
- 触发条件：启用“正式开局构筑合法性”时。
- 验收标准：开局阶段必须验证每位玩家仅选择 2 派系且牌库牌系合法；非法构筑拒绝进入步骤 5。
- 依赖：卡牌 `society` 元数据稳定、开局状态机校验入口。

### PN-BASE-003
- ID：`PN-BASE-003`
- 延后原因：当前先保证可打牌与可结算，不阻塞原型体验于牌库规模规则。
- 触发条件：进入“正式规则校验版”时。
- 验收标准：支持最小/最大牌库张数和同名副本数限制；错误信息可回显到开局向导。
- 依赖：构筑校验器、卡牌定义去重键、开局 UI 错误提示。

### PN-BASE-004
- ID：`PN-BASE-004`
- 延后原因：忠诚/颜色前置条件需要额外的状态字段与关键词解释器，本轮未纳入最小可玩范围。
- 触发条件：开始接入基础包中依赖忠诚/颜色限制的卡义时。
- 验收标准：不满足前置条件时 `play_card` 拒绝；满足时正常部署；测试覆盖角色/附属/事务三类。
- 依赖：玩家属性模型、卡牌前置条件解析、legality catalog 扩展。
- 当前状态（2026-04-02）：`play_card` 已接入“场上已揭示且未摧毁角色/附属颜色计数”忠诚校验；扩展到其他动作仍延后。

### PN-BASE-001A
- ID：`PN-BASE-001A`
- 延后原因：`queue_operation` 目前承担调试器直连定义卡入口，直接并轨费用/忠诚会影响现有回归与调试效率。
- 触发条件：battle 主流程稳定后，开始收敛调试动作到统一打牌语义时。
- 验收标准：`queue_operation` 进入与 `play_card` 一致的费用/忠诚 legality；保留兼容模式开关或完成迁移后删除兼容路径。
- 依赖：调试器动作迁移方案、fixture 更新、回归基线重录。

### PN-BASE-004.5
- ID：`PN-BASE-004.5`
- 延后原因：基础包卡义仍处于分批接入，本轮只覆盖对战主通路所需子集。
- 触发条件：进入“基础包完整卡义”阶段时。
- 验收标准：基础包未落地卡牌全部完成语义实现、回归场景与异常分支测试。
- 依赖：卡义拆解清单、DSL/脚本执行能力、规则回放基线。

### PN-BASE-005
- ID：`PN-BASE-005`
- 延后原因：已明确“基础包优先”，扩展系列暂不进入当前迭代。
- 触发条件：基础包核心卡义闭环与回归稳定后。
- 验收标准：可按 set 开关扩展卡池（霸权/星光/帷幕/序曲/间奏），且不破坏基础包回归。
- 依赖：卡池选择器、扩展规则清单、兼容性回归套件。

### PN-BASE-006
- ID：`PN-BASE-006`
- 延后原因：真实“互洗/先手决定历史规则”涉及多人交互，本轮先落地服务端一次性语义。
- 触发条件：需要更贴近线下流程或联机交互时。
- 验收标准：支持双方互洗确认、上一局败者决定先手的完整交互分支和审计日志。
- 依赖：setup API 扩展、前端多步骤交互、会话历史存储。

### PN-BASE-007
- ID：`PN-BASE-007`
- 延后原因：当前优先可玩性与规则闭环，暂以文本与折叠说明承载地区信息。
- 触发条件：进入原型美术迭代阶段时。
- 验收标准：地区/卡面支持图片资源与文本混排，移动端与桌面端都可读可操作。
- 依赖：美术资源清单、静态资源管线、前端布局升级。



## 2026-04-02 第二十次补记（可玩前端牌桌 + Playwright 试玩闭环）

- 本轮把 web 入口从“调试器主视图”切到“可玩牌桌主视图”，目标是让玩家可直接在牌桌区域进行对战动作，而不是只看调试面板：
  - `web/src/app/AppShell.tsx` 现默认挂载 `BattleShell`。
  - 新增 battle slice：
    - `web/src/battle/model.ts`：把 `StatePatched` 解析为牌桌区域（对方区域/争夺区/本方区域）与动作候选。
    - `web/src/battle/components/BattleTable.tsx`：按牌桌语义渲染双方区域、3 个地区槽、争夺区未归属区、手牌/牌库/墓地/计分/秘社。
    - `web/src/battle/components/ActionComposer.tsx`：可提交 `declare_attack / declare_investigation / move_card / queue_operation / reveal / inspect / marker / pass / advance` 等动作。
- 为支持牌桌地区分组，协议投影补了最小元数据：
  - `CardView.kind`、`CardView.regionCardId`、`CardView.regionOrder`（Go projection + TS protocol 同步）。
- 自动试玩闭环已接入：
  - 新增 `web/playwright.config.ts`，启动 `Go API (:8080)` + `Vite (:4173)` 双服务。
  - 新增 `web/tests/battle.spec.ts`，覆盖“打开牌桌 -> 重开 -> 攻击 -> 过牌”的 smoke 对战流。
  - 新增 npm scripts：`test:e2e`、`test:e2e:headed`。
- 回归结果：
  - `go test ./server/...` 通过
  - `cd web && npm test` 通过（含 battle 新增单测）
  - `cd web && npm run build` 通过
  - `cd web && npm run test:e2e -- tests/battle.spec.ts --project=chromium` 通过

## 2026-04-02 第十九次补记（Day1~Day5 串行落地）

- 按“基础包优先、扩展后置”口径，已串行落地 4 个收口项（对应本轮实现 commit：`d7ae754`）：
  1. 赢区去向修正：地区赢取后由 `discard` 改为进入 `score` 区；并补齐“先清该地区单位/附属，再地区入计分区，再补地区”的最小顺序闭环。
  2. 抓牌空库终局：补上 `deck_out`（单方空库失败）与 `deck_out_draw`（双方同时空库平局）两条终局分支。
  3. 先手特权显式动作化：新增 `use_first_player_privilege`，具备 legality、单回合一次限制与事件回放。
  4. 基础机制补齐：新增 `move_card`（相邻地区移动）与 `set_card_marker/remove_card_marker`（卡牌级标记注册表），并接入投影显示。
- 状态模型补充：
  - 新增区域/终局相关字段和枚举：`CardZoneScore`、`MatchEndReasonDeckOut`、`MatchEndReasonDeckOutDraw`。
  - 新增卡牌级字段：`CardState.RegionCardID`、`CardState.RegionOrder`、`BoardState.CardMarkers`。
- 前端同步：
  - Live Debugger 新增 `Use First-Player Privilege` 预置动作。
  - 卡牌详情面板新增 `markers` 展示（card markers）。
- 回归与覆盖：
  - `go test ./server/...`、`cd web && npm test` 均通过；
  - `server/pkg/rules` 总覆盖率约 `86%`，新增规则文件边角/异常分支覆盖均达到当前阶段门槛。

## 2026-04-02 第十八次补记（机制→引擎差距矩阵 + Shield V1）

- 已新增“机制 -> 引擎差距矩阵”文档：
  - `docs/MECHANISM_ENGINE_GAP_MATRIX_2026-04-02.md`
- 本轮规则落地：
  - Shield V1 已接线：当带护盾的目标被敌方 `queue_operation` 或 `declare_attack` 指定时，会自动移除 1 个护盾并终止该次效果。
  - 当前为最小闭环实现，不包含“可选择是否消耗护盾”的交互决策模型。
- 执行优先级（记忆约束）同步：
  - 先做基础卡牌所需机制。
  - 扩展系列（星光、帷幕之后、序曲、间奏）降级为低优先级，当前阶段不纳入实现范围。

## 2026-04-02 第十七次补记（Engine 去重型收口阶段完成）

- 第十六次补记中的 4 个“下一步动作”已完成：
  1. `engine.go` 通用 preflight（priority / empty-stack / target player/card 存在性）已抽离到 `action_preflight_flow.go`。
  2. `engine` 结构守卫已扩展：
     - `TestEngineOrchestrationGuard_NoGenericPreflightChecks`
     - 防止 preflight 细节回流到 `engine.go`。
  3. `XQ01` prerequisite（框架层）已接线：
     - 在不引入全局沉默的前提下，将 `TargetCondition.AbilityKinds` 映射到现有动作权限检查入口；
     - 新增 `action_permission_flow_test.go` 覆盖命中与不命中场景；
     - 仍不引入 `XQ01` 生产规则，保持 deferred 结论不变。
  4. 回归基线已通过：
     - `go test ./server/...`
     - `cd tools/fixture-tools && npm test`
     - `cd web && npm test`
- 本轮详细记录见：
  - `docs/trae_review/engine_preflight_xq01_prereq_followup_2026-04-02.md`

## 2026-04-01 第十六次补记（Engine 去重型收口下一步）

- 已完成的结构收口（本轮）：
  - `queue_operation` 的 legality/build/window/play/target-precheck 已从 `engine.go` 拆到独立 flow 文件。
  - action-permission legality 已拆到 `action_permission_flow.go`。
  - 状态写入 guard 已覆盖 `FaceDown/Revealed/Markers/Resolved/RandomResults/Exhausted/Destroyed/Zone`。
- **下一步动作（按顺序执行）**：
  1. 抽离 `engine.go` 的通用 preflight（`target/player/card` 存在性与 empty-stack/priority 前置检查）到独立模块，`engine.go` 保留编排 switch。
  2. 新增 `engine` 结构守卫测试，禁止回流 card-rule 细节和 permission/targeting helper 到 `engine.go`。
  3. 在不引入全局沉默的前提下完成 `XQ01` 最小 prerequisite 对接：把 `TargetCondition.AbilityKinds` 映射到现有动作权限校验入口（仅框架层，不落错误牌义）。
  4. 完成一轮回归基线：`go test ./server/...`、`(cd tools/fixture-tools && npm test)`、`(cd web && npm test)`，并同步 `docs/trae_review` 任务记录。

## 2026-04-01 进度补记

- `Phase 3` 已经开始，但只推进了第一刀：
  - 新增 `declare_attack`
  - 新增 `declare_investigation`
  - 角色动作会读取 `EffectiveStats`
  - 角色动作会使执行者 `Exhausted`
  - 攻击会走最小伤害/销毁语义
  - 调查会向地区放置 `influence` counter
- 这还不等于“完整可玩对局闭环”。
- 下一步仍然是：
  - 地区争夺
  - 得分
  - 胜利条件
  - 持续效果来源离场清理（已在后续补上）

## 2026-04-01 第二次补记

- `Phase 3` 现在已经补上第二刀：
  - 地区牌会记录 `InfluenceByPlayer`
  - 地区控制权以最高 influence 决定；平局则无控制者
  - `declare_investigation` 会把 influence 记到行动者名下，而不只是加总 counter
  - `placeInfluence` DSL 作用到地区时，也会同步 region control
  - `end -> main` 的回合切换会按当前已控制地区结算得分
  - 分数达到 `VictoryThreshold` 时会写入 `WinnerPlayerID`
- 这仍然不是完整闭环，当前明确未完成的是：
  - 分数 / 胜利状态尚未投影到 Web 调试器
  - 回合切换尚未轮转 `ActivePlayerID`
  - 地区争夺还没有更完整的占领/结算规则
  - 持续效果来源离场清理当时仍未接入，但已在后续补上

## 2026-04-01 第三次补记

- `Phase 3` 的这一步又补了两个调试和回合层缺口：
  - `end -> main` 现在会轮转 `ActivePlayerID`
  - projection 现在会携带 `Score`
  - Web 调试器现在会显示：
    - `Active Player`
    - `Score`
    - `Winner`
- 因此当前 sandbox 已经能观察到：
  - 地区控制
  - 回合结束得分
  - 胜利者出现
  - 新回合主动玩家轮换
- 但它仍然没有到“完整可玩”：
  - 还没有基于 `WinnerPlayerID` 停止后续动作
  - 还没有更完整的地区与得分节奏
  - 还没有把这些状态接成更正式的玩家操作流

## 2026-04-01 第四次补记

- `Phase 3` 这一步把“胜利出现后对局应停止”也接上了：
  - Go legality 会在 `WinnerPlayerID != ""` 后拒绝后续动作
  - 拒绝码为 `RULES_FAILED_GAME_ALREADY_OVER`
  - `ActionRejected.context.winnerPlayerId` 会明确指出当前胜者
  - live sandbox 的动作面板会在 winner 存在时禁用，并显示 winner 提示
- 因此当前最小 sandbox 已经具备：
  - 地区争夺
  - 得分
  - 胜利产生
  - 新回合主动玩家轮转
  - 胜利后停止继续提交动作
- 当前仍未完成的，是把这些规则进一步拓展成更完整的一局流程，而不是继续在“对局已结束”之后推进状态。

## 2026-04-01 后续补记（continuous source lifecycle）

- `Phase 3` 这一步把 continuous source lifecycle 的最小语义也接上了：
  - `RecalculateContinuousEffects()` 在 prune 阶段会移除来源牌已离场、弃置或销毁的 continuous effects
  - 当前只追踪能在 `board.cards` 中定位到的 source card
  - 像 `BQ022` 这类 fixture-only source 不会被误判为“来源失效”
- 新增回归测试：
  - 来源牌仍在 `table` 时，target 继续获得 buff
  - 来源牌进入 `discard` 后，下一次重算会移除 effect，并让 target 恢复 printed stats
- 这意味着当前最小闭环已经补上：
  - 地区争夺
  - 得分
  - 胜利产生
  - 新回合主动玩家轮转
  - 胜利后停止继续提交动作
  - continuous effects 来源离场清理
- 现在仍值得继续推进的，主要只剩：
  - 更完整的地区与得分节奏
  - 更正式的玩家操作流
  - 更多真实可玩的 fixture / card semantics

## 2026-04-01 补记（XQ22 legality identity fix）

- `queue_operation` 的第一条"按真实卡牌定义识别场上特定牌"的 prohibition slice 已接入：
  - 当前 `XQ22` 会在 ready 且位于 `table` 时禁止打出 `basicType == "事务"` 的卡
  - 匹配依据不再是显示名，而是 `CardState.DefinitionID == "XQ22"`
- 这次补丁明确了两个身份层：
  - `CardState.CardID`：场上实例 ID，同一玩家同名两张牌也必须不同
  - `CardState.DefinitionID`：卡牌定义 ID，用于规则识别"这是不是 XQ22"
- 这一步是后续做：
  - 多张同定义牌共存
  - 按定义识别光环 / 禁令 / 结附来源
  - 避免文案改名导致规则失效
  的前置条件。
- 当前仍未完成的是把 `DefinitionID` 贯穿到更正式的 permanents / attachments 上场生命周期里；目前它主要先服务于规则判定和测试建模。

## 2026-04-01 第六次补记（Legality Framework Hardening V1）

- Phase 3 legality hardening 现在有了专用的 production rule catalog：
  - `server/pkg/rules/legality_catalog.go` - 集中管理 XQ22/XQ31 等 production rules
  - `server/pkg/rules/legality_shared.go` - 共享 source-condition 和 actor-scope 匹配逻辑
- duplicated legality matcher logic 已合并到 shared helpers：
  - `cardMatchesDefinitionAndCondition()` - 检查卡牌是否满足定义 ID 和条件
  - `scopeAppliesToActor()` - 检查作用域是否适用于指定行动者
- `XQ31` 现在只在 `queue_operation` 上检查 target legality：
  - `declare_attack` 和 `declare_investigation` 不再被 XQ31 误伤
  - 边界已锁定并通过回归测试验证
- 下一阶段跟进方向：
  - 完成 XQ31 数值光环（+1 防御力）实现
  - 设计 XQ01 地区作用域沉默的 prerequisite

## 2026-04-01 第七次补记（XQ31 数值光环）

- Phase 3 continuous effects 现在有了专用的 production template catalog：
  - `server/pkg/rules/types.go` - 新增 `ContinuousEffectTemplate` 类型
  - `server/pkg/rules/legality_catalog.go` - 新增 `XQ31ContinuousEffectTemplate` 和 `BuildProductionContinuousEffectTemplates()`
  - `server/pkg/rules/legality_shared.go` - 新增 `cardMatchesTargetCondition()` 共享函数
  - `server/pkg/rules/continuous.go` - 新增 `BuildContinuousEffectsFromTemplates()` 函数，并集成到 `RecalculateContinuousEffects`
- XQ31（莫兰大主教）的数值光环已完整实现：
  - 当 XQ31 在场且就绪时，所有本方声望角色获得 +1 防御力
  - 效果不影响敌方、非声望、非角色、已摧毁的卡牌
  - 新增完整的单元测试和 Golden Scenario 验证
- 修复了 `resetDerivedCardState` 中意外重置 `Destroyed` 标志的问题
- 下一阶段跟进方向：
  - 设计 XQ01 地区作用域沉默的 prerequisite

## 2026-04-01 第八次补记（XQ01 错误全局沉默已回滚）

- `XQ01` 曾被误实现成“全桌所有角色都不能攻击/调查”的 production continuous template。
- 该实现现已回滚，原因不是代码质量问题，而是**牌义范围错误**：
  - 真实需求仍然是“本地区角色不能发动触发能力和行动能力”
  - 当前 rules core 还没有 region-scoped silence / ability-kind restriction 的正式模型
- 当前结论恢复为：
  - `XQ01` 仍保持 deferred
  - 不应再以全局 `prohibitPermission(attack/investigate)` 形式进入 production catalog
  - 等 prerequisite 到位后再按正确语义实现

## 2026-04-01 第九次补记（Asset / Permanent Model V1）

- Phase 3 现在正式把“资产牌作为真实在场永久物”这层状态模型做出来了：
  - `projection.go` 中新增 `CardKindAsset` 常量
  - `invariants.go` 中新增 `InvariantCardDestroyedStateValid` 检查，确保卡片的 Destroyed 状态与 Zone 一致（在 Table 时不应该被 Destroyed，在 Discard 时应该被 Destroyed）
  - `clone.go` 已完整覆盖所有 CardState 字段克隆，包括 Kind
  - 现有代码框架（进场、离场、continuous source validity、投影、回放）已经支持任何 CardKind，包括 Asset
- 新增完整的单元测试：
  - `asset_test.go` - 覆盖 Asset Card 进场、离场、不影响现有 Character 等场景
- 这是 Asset / Permanent V1，不是完整 attachment system：
  - 不做 BQ022 这类附属/结附牌的完整语义
  - 不做回收/回手
  - 不做暗藏部署
  - 不做资产主动能力
  - 不做更多 UI

## 2026-04-01 第十次补记（Discard / Graveyard Semantics V1.5）

- Phase 3 现在把“进入 discard/坟墓”从零散赋值，收敛成统一的规则语义与实现入口：
  - `continuous.go` 中新增 `moveCardToDiscard()` 统一 helper 函数，专门处理“卡进入 discard”
  - `applyDerivedBoardSemantics()` 现在使用统一 helper，不再散写字段
  - `dsl.go` 中新增 `applyDiscardCardEffect()` DSL effect 处理
  - 在 `applyDSLEffect()` 中新增对 "discardCard" 的支持
- 新增完整的单元测试：
  - `discard_test.go` - 覆盖致命伤害路径和 discardCard DSL effect
- 这是 Discard / Graveyard V1.5，不是完整坟墓交互系统：
  - 已支持统一离场 + `discardCard` 最小 DSL
  - 仍未做：坟墓检索、复活、回手、坟墓作为资源区

## 2026-04-01 第十一次补记（Attachment / Host Lifecycle V1）

- Phase 3 现在把附属从 tracking V0 推进到真实生命周期：
  - `types.go` 中 `Attachment` 结构体新增 `HostCardID` 字段，用于宿主离场联动
  - `attachment.go` 中 `AttachmentBuilder` 新增 `Host()` 方法支持设置宿主
  - `attachment.go` 中 `PruneExpired()` 现在会检查宿主有效性，宿主离场时附属自动移除
  - `attachment.go` 中新增 `filterContinuousEffectsBySource()` 函数，清理已移除附属源的 continuous effects
  - `isAttachmentStillActive()` 现在检查宿主（HostCardID）、目标（TargetCardID）和源（SourceCardID）的有效性
- 新增完整的单元测试：
  - `attachment_lifecycle_test.go` - 覆盖宿主离场联动、附属离场 effect 失效、V0 不退化
- 这是 Attachment / Host Lifecycle V1，不是完整 attachment system：
  - 不做"回手/回收"复杂牌面语义
  - 不做完整 attachment stack 互动

## 2026-04-01 第十二次补记（Secret Society Marker V1）

- Phase 3 现在把"秘社标记物"从 schema 预留变成 rules authoritative state：
  - `types.go` 中新增 `MarkerRegistry` 类型，用于存储玩家标记物
  - `BoardState` 新增 `Markers` 字段
  - `MarkerRegistry` 提供 `GetMarker()` 和 `SetMarker()` 方法
  - `NewGameState()` 初始化 `Markers` 注册表
- `projection.go` 中新增 marker 投影支持：
  - `PlayerViewState` 新增 `Markers` 字段
  - `SpectatorViewState` 新增 `Markers` 字段
  - 新增 `projectMarkersForPlayer()` 和 `projectMarkersForSpectator()` 函数
- 新增完整的单元测试：
  - `marker_test.go` - 覆盖 marker 增减、回放一致、投影一致、legality 条件
- 这是 Secret Society Marker V1：
  - 已支持 marker 状态模型、增减动作、投影支持、legality 条件检查
  - 不做大量真实卡接入
  - 不做 UI 大改，只保证协议里有数据

## 2026-04-01 第十三次补记（Hidden Deployment & Reveal V1）

- Phase 3 现在实现"暗藏部署 -> 现身"最小闭环：
  - `projection.go` 中 `CardState` 新增 `FaceDown` 字段
  - `cardVisibleToPlayer()` 现在检查 `FaceDown`，face-down 卡牌只对 owner 可见
  - `projectCardForSpectator()` 对 face-down 卡牌返回 hidden view
- 新增完整的单元测试：
  - `hidden_deployment_test.go` - 覆盖 face-down 部署、owner 可见、对手隐藏、reveal 状态转换
- 这是 Hidden Deployment & Reveal V1：
  - 已支持 face-down 部署、投影可见性控制、reveal 状态转换
  - 不做完整伏击时机学
  - 不做复杂触发链

## 2026-04-01 第十四次补记（Timing Window V2 & Conflict Loop V2）

- Phase 3 现在扩展时机模型和对抗节奏：
  - Timing Window V2：
    - 新增 `timing_window_test.go` - 覆盖 fast action 允许条件、reaction 需要 stack 非空
  - Conflict Loop V2：
    - 新增 `conflict_loop_test.go` - 覆盖战斗目标合法性、调查与地区控制衔接、游戏结束检查
- 这是最小实现：
  - Timing Window V2：不做完整 MTG 级别时机系统，不做 replacement effects
  - Conflict Loop V2：不做完整战斗子阶段系统，不做大规模扩卡

## 2026-04-01 第十五次补记（综合测试覆盖提升）

- 新增 `comprehensive_test.go`，包含10个高质量测试用例：
  1. `TestAttachment_MultipleAttachmentsCleanupOnHostDeparture` - 多附属同时清理
  2. `TestMarker_Boundary_ZeroAndNegativeValues` - Marker 边界条件（零值/负值）
  3. `TestHiddenDeployment_NonOwnerCannotSeeDetails` - 非 Owner 无法查看 Face-Down 详情
  4. `TestTimingWindow_ReactionNotAllowedAfterStackClear` - Stack 清空后 Reaction 不允许
  5. `TestConflictLoop_AttackWithExhaustedAttackerShouldFail` - Exhausted 攻击者攻击失败
  6. `TestAttachment_SourceDepartureWithHostRemaining` - 源离场但宿主保留
  7. `TestMarker_MultiplePlayersWithDifferentMarkers` - 多玩家不同类型 Marker
  8. `TestHiddenDeployment_RevealTriggersContinuousEffect` - Reveal 触发 Continuous Effect
  9. `TestConflictLoop_GameOverBoundaryConditions` - 游戏结束边界条件（9/10/11分）
  10. `TestComprehensive_CombinedSystems` - 综合场景（Attachment + Marker + Hidden Deployment）
- `projection.go` 中 `CardView` 新增 `FaceDown` 字段，支持投影中显示卡牌朝向状态
- 所有测试用例覆盖核心功能、边界条件和异常场景，确保系统在不同使用场景下的正确性、稳定性和可靠性

## 2026-03-31 执行顺序说明

- 当前仓库状态已经从“纯规则核骨架”推进到了 `LIVE_SANDBOX` 阶段，因此下一步不建议立刻继续堆按钮、堆卡或上真实联机。
- 当前最优先级应该调整为：
  1. 冻结 `M0 sandbox` 基线
  2. 建立 golden scenarios / replay / invariants / projection leak 回归护城河
  3. 再进入第一套完整可玩规则闭环
  4. 最后才扩首发卡池与真实联机
- 换句话说，这份文档里的 4 个里程碑仍然成立，但当前执行顺序应以前两项“稳固基线”和“最小完整闭环”为先。
- 具体落地顺序见 [NEXT_STEP_EXECUTION_PLAN_2026-03-31.md](./NEXT_STEP_EXECUTION_PLAN_2026-03-31.md)。

## 2026-03-31 第五次补记（sandbox 终局态与重开一局闭环）

- 已完成“winner gate -> 正式 MatchState”的升级：
  - `GameState.Match` 作为正式状态源，包含 `status / endReason / winnerPlayerId / finishedAtRevision`
  - 终局合法性 gate 改为读取 `Match.Status == finished`，拒绝码维持 `RULES_FAILED_GAME_ALREADY_OVER`
  - 拒绝上下文会携带 `winnerPlayerId`，便于前端直出终局原因
- 已完成 sandbox reset 通路：
  - `SandboxSession.Reset()` 会重建 canonical `NewM0SandboxState()`，重置投影器并重新生成 bootstrap `StatePatched` 批次
  - HTTP 新增 `POST /api/debugger/reset`，直接返回 reset 后的新 bootstrap 消息流
- 已完成 Web 调试器 reset 交互：
  - Action 面板新增 `Reset Sandbox` 按钮，调用 `/api/debugger/reset`
  - 若当前 patch 处于终局（`winnerPlayerId` 非空），动作按钮禁用并展示 `Game over. Winner: ...`
  - reset 成功后会替换消息流并恢复可提交动作状态
- 这意味着浏览器里的最小闭环已经可直接体验：
  1. 打到终局
  2. 看到终局态（非仅 score winner 字段）
  3. 一键重开同一 sandbox 会话的新对局

## 里程碑 1：Permission / Legality V2
- 把 permission hook 从 `inspect_card` 扩到现有需要约束的动作：`reveal_card`、`queue_operation`，以及下一步新增的角色动作入口。
- 在 `CardState` 固化 `RequiredPermissions`、`Prohibitions`、`ActionQuota`；`CostAdjustment` 继续只保留字段，不先做完整费用系统。
- `CheckLegality` 统一走一个 capability 检查入口；拒绝码固定为：
  - `LEGALITY_FAILED_PERMISSION_REQUIRED`
  - `LEGALITY_FAILED_ACTION_PROHIBITED`
  - `LEGALITY_FAILED_ACTION_QUOTA_EXCEEDED`
- 规则固定：`prohibit` 覆盖 `grant`；无 `RequiredPermissions` 的动作不要求显式 grant；quota 必须可 replay。

## 里程碑 2：Character Stats Semantics V1
- 新增两个最小角色动作：`declare_attack`、`declare_investigation`。
- `combat` 用于攻击伤害值，`defense` 用于致命阈值，`investigation` 用于放置影响力；`influence` 先继续只作为有效面板值保留。
- 角色动作统一要求：目标在 `table`、未 `Destroyed`、未 `Exhausted`、当前有行动权、stack 为空。
- 结算后生成可 replay 的结构化事件：`attack_declared`、`damage_applied`、`card_destroyed`、`investigation_applied`。
- `modifyStat` 必须真正影响这些动作读取的数值，而不只是影响投影。

## 里程碑 3：Continuous Effect Lifecycle V1.1
- 增加“来源失效即移除”规则：source card 离场/弃置/被销毁后，其 continuous effects 在下一次 commit 重算中移除。
- 保持当前约束：每次 commit 最多一次完整重算；重算中再次请求重算只记录一次 pending，不允许递归失控。
- 继续不做完整 dependency graph；`DependencyKey` 和 `ResolveConflict` 只保留接口。
- `grantPermission / prohibitPermission / modifyStat / addKeyword` 允许来自真实 fixture；`scriptId != null` 的卡仍只走 script 入口。

## 里程碑 4：DSL Fixture Gate 扩展到真实可玩样例
- 新增第一批命中新语义的 fixture：授予检视、禁止发动、本回合 `+defense`、攻击造成伤害、调查放置影响力。
- TS 侧继续只做 schema/fixture gate；Go 侧对同一 fixture 做 contract test + rules test。
- `queue_operation` 对 pure DSL 卡必须能真正落到：permission change / numeric change / damage / influence / destroy，而不是只写 event。
- 每批新增真实卡后都补一篇 docs，记录规则假设和当前未实现边界。

## Test Plan
- 合法性：required permission 缺失会拒绝；grant 后可通过；prohibit 存在时即使 grant 也拒绝；quota 用尽后拒绝。
- 数值：`modifyStat(defense)` 改变致命阈值；`modifyStat(combat)` 改变攻击伤害；持续效果到期后恢复并影响后续结算。
- 回放：permission、damage、destroy、discard、continuous cleanup 下，action log replay 到同一状态。
- 投影：destroy/discard 后仍遵守现有 hidden-info 隔离，不泄露手牌/牌库信息。
- 双端契约：新增 fixture 在 TS Vitest 和 Go `testing` 下都通过。

## Assumptions / Defaults
- 先以“可扩展 alpha”为终点，不追求完整 dependency engine、replacement effect、trigger stack、完整费用系统。
- Go 继续是唯一语义权威；TS 只负责 schema、fixture、作者工具和前置校验。
- 销毁规则固定为：`damage >= EffectiveStats.Defense` 即离场进 `discard`；当前只实现 Shield V1（敌方指定拦截），仍不实现再生、濒死队列。
- 新动作只先做 `declare_attack`、`declare_investigation`，不同时展开完整战斗阶段改造。
