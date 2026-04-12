# 下一阶段规则核计划 (Next Gen Rule Plan)

## Summary
- 当前已经有：最小规则核、priority/stack、projection、continuous effects、`inspect` 的 permission hook、`dealDamage` 对 `EffectiveStats.Defense` 的最小致命判定，以及第一批角色动作入口 `declare_attack / declare_investigation`。
- **已完成 (Consolidation Phase)**：全局领域语义收口的第一轮。显式会话生命周期、元数据驱动动作策略 (ActionPolicy)、正式战斗动作支付并轨、投影契约加固。
- **本轮新增基线**：recovery 会恢复 table 上的角色/资产 ready 状态；地区控制已能基于“本地区未横置角色的 influence + 持久地区势力”动态刷新；projection / battle UI 已能展示地区控制者与按玩家分布的当前势力。
- **2026-04-03 继续推进**：`main` 已拆成 `first_player_action / second_player_action`；`conflict` 已成为正式 phase；调查奖励 / 战斗奖励已有 prompt 状态；`reveal_face_down` / `activate_ability` 已进入权威动作管线；prompt 私有信息已按 viewer 做投影裁剪。
- 如果目标是“继续稳定接真实卡 DSL”，接下来的核心矛盾是：`Attachment` 完整生命周期、`World Deck` 地区规则、`End Step` 自动处理、`Continuous Effect` 叠加、`Trigger` 队列。

## 目标 (Goals)
- 完成 `Region Scoring` (占领/计分)。
- 完成 `Role Actions` (攻击/调查) 完整流程。
- 实现 `Continuous Effect` 叠加（不仅仅是覆盖）。
- 实现 `End of Turn` 自动维护。
- 在未来半年内，把当前原型推进到“可用规则书核心 + 可持续扩卡”的状态，而不是继续堆单点特判。

## 核心约束 (Constraints)
- 保持“Go 为唯一真相源”。
- 所有的“规则”必须能在 Go Test 中独立验证。
- 前端只负责“投影渲染”和“前置格式校验”。
- 不做向后兼容；过渡层、兼容层、双轨语义在影响理解时应直接删除。

## 未来半年执行假设 (2026-04 ~ 2026-09)
- 规划口径按 **1 名主力工程师 + AI 协作** 估算。
- 预计总工作量约 **24-30 人周**，其中：
  - 规则核与状态机：12-14 人周
  - 前端 battle / projection / 协议：4-5 人周
  - DSL / 内容接入 / 卡牌能力注册：3-4 人周
  - 回归测试 / e2e / 文档 / trace 调试：5-7 人周
- 预留 **15%-20% 缓冲** 处理规则书细节、trace 复盘和重构返工。
- 半年目标不是“支持全部卡池”，而是：
  - 把 battle 主流程补到规则书核心闭环
  - 建立可扩展的 trigger / replacement / attachment / payment 基础设施
  - 让新增卡牌能力更多通过显式注册或 DSL 接入，而不是再回到散落特判

## 未来半年阶段划分 (Recommended Sequence)

### Phase A: 对抗流程收口（2026-04 ~ 2026-05）
- 目标：把当前 `conflict` 主骨架补成规则书可玩的 battle 核心闭环。
- 退出标准：
  - tie + 先手标志特权并入新 conflict 状态机
  - 拦截 / 战斗伤害 / 致命判定闭环
  - `declare_attack / declare_investigation` 从正式 battle UI 移除，只保留 debug 通道
  - 前端改为以 `capabilities + pendingPrompt` 为主，不再靠动作类型下拉硬凑规则

### Phase B: 触发与持续系统（2026-06 ~ 2026-07）
- 目标：把“能玩几张卡”推进到“能稳定接更多卡”的机制层。
- 退出标准：
  - Trigger Stack 正式落地
  - Replacement / Prevention / Duration 基础骨架可用
  - Attachment 生命周期与宿主离场联动稳定
  - Layer System 从“覆盖”推进到“可叠加、可重算、可清理”

### Phase C: 规则书支付与地区生命周期（2026-08 ~ 2026-09）
- 目标：从 prototype 资源模型继续逼近规则书语义，并补齐地区/局终流程。
- 退出标准：
  - `PaymentModeRulebook` 初版可跑通
  - 资源生成/支付/清空不再耦合 turn-number 原型模型
  - 地区计分、胜利判定、地区补充/World Deck 生命周期闭环
  - 回归测试和 trace 调试足以支撑继续扩卡

---

## 计划路线图 (Roadmap)

### PN-BASE: 基础机制加固 (已完成)
- [x] **PN-BASE-001: 支付引擎/合法性并轨** (已完成)
  - 验收标准：`play_card` / `build_asset` 必须通过统一支付逻辑；`queue_operation` 继续保留调试通道兼容。
- [x] **PN-BASE-002: 正式开局构筑合法性 (派系)** (已完成)
  - 验收标准：开局阶段验证每位玩家仅选择 2 派系且牌库牌系合法。
- [x] **PN-BASE-003: 构筑限制 (张数/副本)** (已完成)
  - 验收标准：支持 40-60 张牌库和同名 3 副本限制。
- [x] **PN-BASE-004: 忠诚/颜色前提校验** (已完成)
  - 验收标准：`play_card` 必须检查场上角色提供的颜色；`queue_operation` 不作为正式对战忠诚入口。
- [x] **PN-BASE-006: 交互式开局 (先手选择)** (已完成)
  - 验收标准：支持上一局败者决定先手的交互分支。
- [x] **PN-BASE-007: 丰富视觉表现** (已完成)
  - 验收标准：牌桌支持状态（横置/暗置）和数值统计（战斗/防御/调查）的直观显示。

### PN-REG: 地区与计分 (Phase 3 Active)
- [x] **PN-REG-001: 地区势力值动态计算** (completed 2026-04-12)
  - 验收标准：地区卡 `EffectiveStats.Influence` 随驻场角色变化。 已把 ready 角色 + 地区基础势力汇总折叠进地区 `EffectiveStats.Influence`，并将地区得分阈值显式收口到 `RegionScore / PrintedStats.Influence`，避免与动态当前势力混用。
- [x] **PN-REG-002: 地区计分快照** (completed 2026-04-12)
  - 验收标准：每回合结束检测地区占领情况并增加玩家分数。 现有 end->main 流程已在 authoritative state 与 player/spectator projection 中同步刷新分数。
- [x] **PN-REG-003: 胜利判定规则** (completed 2026-04-12)
  - 验收标准：分数达到阈值（如 100）时自动触发 `MatchFinished`。 当前已支持阈值达成后自动写入 `MatchFinished` / winner / finished revision，并投影到前端。
- [ ] **PN-REG-004: 地区补充 / World Deck 生命周期**
  - 验收标准：地区赢取后，旧地区清场、进分区、补充新地区、相关触发与快速窗口顺序正确。
- [x] **PN-REG-005: 地区级投影契约** (completed 2026-04-12)
  - 验收标准：控制者、地区势力分布、地区替换过程在 player/spectator projection 中保持稳定，不再依赖 battle UI 猜测。 已补 region controller / influence / replacement 的 projection 回归覆盖。

### PN-ACT: 规则书行动权与对抗流程 (Phase 3 Active)
- [x] **PN-ACT-001: 主行动步骤拆分**
  - 验收标准：`main` 拆成 `first_player_action / second_player_action`，后手行动步骤由后手拿初始行动权。
- [x] **PN-ACT-002: 正式 conflict phase**
  - 验收标准：地区按顺序进入调查 / 战斗 / 势力对抗，不再依赖 battle UI 手动 `declare_investigation`。
- [x] **PN-ACT-003: 调查奖励 / 战斗奖励 prompt**
  - 验收标准：调查奖励按差额检视并抓 1；战斗奖励进入伤害分配 prompt；二者均不入栈。
- [x] **PN-ACT-004: 现身与行动能力正式动作**
  - 验收标准：`reveal_face_down` / `activate_ability` 进入统一 legality + stack 管线。
- [x] **PN-ACT-005: tie + 先手标志特权并入新 conflict 流程** (completed 2026-04-11)
  - 验收标准：调查 / 战斗 / 势力对抗打平时，先手标志特权能在新 phase/stage 模型下正确接管。 已添加 first-player privilege 字段并在 conflict tie 解析中作为首要判定，若无该特权则回退到 priority leader 判定，若仍无则无胜者。
- [ ] **PN-ACT-006: 拦截 / 更完整战斗结算**
  - 验收标准：攻击者横置 -> 对方指定拦截 -> 计算伤害 -> 致命判定 -> 离场。
  - 当前状态：`battle_damage` prompt 现已在结算后触发致命伤害清理与地区控制重算；仍缺少“对方指定拦截”与完整攻防分配入口。
- [ ] **PN-ACT-007: 能力注册表扩面**
  - 验收标准：行动能力不再只有示例卡，battle 常见 quick/action abilities 都能正式发动。
- [ ] **PN-ACT-008: 前端能力驱动化**
  - 验收标准：battle UI 读取 `capabilities + pendingPrompt` 渲染可行动作；旧动作类型下拉不再承担规则入口。
- [x] **PN-ACT-009: 响应窗口完备化** (completed 2026-04-12)
  - 验收标准：行动阶段、对抗前后、堆叠结算后都能按规则书正确重开行动权。 现有回归已覆盖主行动步骤切换、prompt 结算后的 fast window、以及 reveal/ability 入栈结算后的行动权回开。

### PN-SYS: 系统扩展
- [ ] **PN-SYS-001: 效果叠加系统 (Layer System)**
  - 验收标准：支持多个数值光环叠加（如 +1/+1 和 +2/+2 得到 +3/+3）。
- [ ] **PN-SYS-002: 触发器队列 (Trigger Stack)**
  - 验收标准：支持“当某事发生时”入栈。
- [ ] **PN-SYS-003: Replacement / Prevention 基础设施**
  - 验收标准：替代与防止效果不再靠动作文件散写，能进入统一裁决管线。
- [ ] **PN-SYS-004: Attachment 完整生命周期**
  - 验收标准：附属进场、脱离、宿主离场、持续效果解绑、投影清理都走统一机制。
- [ ] **PN-SYS-005: Duration / Cleanup 统一收口**
  - 验收标准：直到回合结束 / 对抗结束 / 离场失效的效果，都能通过统一过期清理移除。
- [x] **PN-SYS-006: 状态机可观测性** (completed 2026-04-12)
  - 验收标准：match trace 能明确记录 phase/step/conflict/prompt/stack/priority 的转移，便于复盘 bug。 当前 trace snapshot 已补 conflict / pendingPrompt / priority window / stack depth 摘要。

### PN-PAY: 规则书支付模型
- [ ] **PN-PAY-001: queue_operation / activate_ability 全面并轨 PaymentEngine**
  - 验收标准：正式 battle 动作的费用检查与扣费不再散落在各 action 文件。
- [ ] **PN-PAY-002: PaymentModeRulebook 初版**
  - 验收标准：支持“横置资产得费、步骤结束清空”的基础规则书支付模型，并可与 prototype mode 切换。
- [ ] **PN-PAY-003: 成本语义模型化**
  - 验收标准：资源费、横置费、弃牌费、牺牲费、移除标记费使用统一 cost spec。

### PN-CARD: 卡牌接入与 DSL 扩展
- [x] **PN-CARD-001: 能力注册表与 DSL 分层** (completed 2026-04-12)
  - 验收标准：battle 核心常见能力可判断“走显式注册表”还是“走 DSL”，边界清晰。 当前已明确：`activate_ability` 走显式 ability registry；卡牌效果继续走 fixture source + `executionKind (dsl|script)`。
- [ ] **PN-CARD-002: 进场 / 离场 / 现身触发样板能力**
  - 验收标准：至少一批代表卡可通过统一机制实现，不再新增 ad-hoc 特判。
- [ ] **PN-CARD-003: 共享规则夹具**
  - 验收标准：Go authoritative parser / rule table 可供 web / e2e 直接消费，避免前后端重复解析。

### PN-QA: 测试与交付护栏
- [ ] **PN-QA-001: battle trace 回归套件**
  - 验收标准：真实 `/runtime/match-traces/*.log` 能沉淀为可重放、可断言的回归样例。
- [x] **PN-QA-002: projection golden tests 扩面** (completed 2026-04-12)
  - 验收标准：暗藏者、prompt 私有信息、地区替换、附属解绑等高风险投影都有 golden coverage。 当前已补 prompt 私有信息、地区动态势力、地区替换稳定性的 projection 回归。
- [x] **PN-QA-003: 跨层一致性测试** (completed 2026-04-12)
  - 验收标准：server legality / projection / web composer / Playwright battle flow 对同一规则不再各说各话。 已以地区计分 / 胜利态 / finished match UI 为主线建立 server regression + web rendering + Playwright smoke 的同口径覆盖。
- [x] **PN-QA-004: 文档与里程碑同步机制** (completed 2026-04-12)
  - 验收标准：每轮机制级重构必须同步更新 README、NEXT_GEN_RULE_PLAN、专项设计文档。 本轮已把 README、NEXT_GEN_RULE_PLAN 与 battle/conflict 设计文档同步更新，并作为后续迭代固定动作。

## 建议优先级 (What To Do Next)
1. 先完成 `PN-ACT-005 / 006 / 008 / 009`，把 battle 核心从“能跑”推进到“规则书主流程基本正确”。
2. 紧接着做 `PN-SYS-002 / 003 / 004`，否则继续扩卡只会制造更多局部特判。
3. 再推进 `PN-PAY-001 / 002 / 003`，把当前原型资源模型真正隔离并替换。
4. 最后做 `PN-REG-004 / 005` 与 `PN-QA-*`，把地区生命周期和回归基础设施补完整。

## 未来 6-8 周可执行迭代 (For Agent Swarms)

### Iteration 1: battle 核心闭环，先把主流程做对（第 1-2 周）
- 目标：
  - 完成 `PN-ACT-005 / 006 / 009`
  - 让“调查 -> 战斗 -> 势力”在规则书口径下跑通
- 交付物：
  - tie + 先手标志特权接入新 conflict 流程
  - 拦截 / 战斗伤害 / 致命判定闭环
  - 对抗前后快速窗口、堆叠结算后行动权返回规则稳定
  - 新增 battle trace 回归样例 2-3 条
- 建议并行：
  - `Advisor` 盯状态机和边界
  - `Rules Worker A` 做 tie/privilege
  - `Rules Worker B` 做 intercept/combat resolution
  - `QA Worker` 负责 trace 回放和规则回归

### Iteration 2: battle UI 改为 capabilities/prompt 驱动（第 3-4 周）
- 目标：
  - 完成 `PN-ACT-008`
  - 减少前端动作面板对旧动作类型的依赖
- 交付物：
  - Go projection 下发正式 `capabilities`
  - web `ActionComposer` 改为读 `capabilities + pendingPrompt`
  - `declare_attack / declare_investigation` 从正式 UI 移除，仅保留 debug 通道
  - Playwright battle 流程覆盖 prompt/response/ability/reveal
- 建议并行：
  - `Advisor` 定义 capabilities 契约
  - `Backend Worker` 下发 projection + metadata
  - `Frontend Worker` 改 battle UI
  - `QA Worker` 补 e2e 和 golden

### Iteration 3: Trigger / Replacement / Attachment 机制层（第 5-6 周）
- 目标：
  - 完成 `PN-SYS-002 / 003 / 004`
  - 为后续扩卡建立统一机制层
- 交付物：
  - Trigger Stack 初版
  - Replacement / Prevention 基础裁决点
  - Attachment 生命周期机制收口
  - 至少一组“进场 / 离场 / 宿主离场”样板卡回归
- 建议并行：
  - `Advisor` 审查机制边界，防止又写回 ad-hoc
  - `Rules Worker A` 做 trigger stack
  - `Rules Worker B` 做 attachment lifecycle
  - `Rules Worker C` 做 replacement/prevention

### Iteration 4: PaymentModeRulebook 骨架 + 成本模型（第 7-8 周）
- 目标：
  - 启动 `PN-PAY-001 / 002 / 003`
  - 把当前 prototype 资源模型继续隔离
- 交付物：
  - `queue_operation / activate_ability` 全面通过 `PaymentEngine`
  - 成本语义抽成统一 cost spec
  - `PaymentModeRulebook` 初版可切换，先覆盖 battle 核心支付路径
  - 规则书支付与 prototype mode 的对照测试
- 建议并行：
  - `Advisor` 审核支付抽象是否足够窄、是否可替换
  - `Rules Worker A` 做 cost spec
  - `Rules Worker B` 做 payment engine integration
  - `QA Worker` 做双模式回归

## Agent 群运行建议

### 推荐拓扑
- `1 个主执行 Agent（GPT-5.4）`
  - 职责：主导一整天的推进、拆子任务、集成结果、维持节奏
- `2-4 个实现 Worker（GPT-5.4）`
  - 职责：按明确边界实现具体任务
- `1 个 Advisor（gpt-5.4）`
  - 职责：仅在遇到架构分歧、规则书歧义、状态机卡住、抽象升级风险高时被咨询
- `1 个 QA / Regression Worker（GPT-5.4）`
  - 职责：整理 trace、写回归、跑验证、报告残余风险

### 运行规则
- 默认模式是 **5.4 先做**，不是先把所有问题都提交给 Advisor。
- 主执行 Agent（GPT-5.4）负责：
  - 根据当前 iteration 拆分任务
  - 为 worker 指定互不冲突的 write scope
  - 持续集成、跑测试、更新文档
  - 只有在高风险问题上才升级咨询 Advisor
- Advisor 不直接吞掉全部实现，只负责：
  - 回答高难架构问题
  - 评估状态机/支付/trigger 等抽象是否走偏
  - 在 worker 卡住时给出更窄、更稳的修正方向
- Worker 必须按文件 ownership 工作：
  - 一个 worker 只负责一组不重叠文件
  - 不允许不同 worker 同时改同一个核心文件，除非 Advisor 明确安排串行
- QA Worker 不做主逻辑设计：
  - 只做回归、trace、golden、e2e、风险报告
- 每一轮都必须满足：
  - server tests 绿
  - web unit tests 绿
  - 至少一条 battle e2e 绿
  - 文档同步

### 何时才该升级问 Advisor
- 出现以下情况之一，再升级给 `gpt-5.4 Advisor`：
  - 规则书条文之间有冲突，5.4 无法自信判断
  - 一个改动会同时影响 `engine / projection / web protocol`
  - 需要重新定义状态机、prompt 契约、payment 抽象、trigger/replacement 边界
  - 同一个 bug 背后有两种以上合理修法，且代价差异明显
  - 相关测试已经补了，但实现仍在局部兜圈子
- 不该升级的情况：
  - 单文件 bugfix
  - 已有设计下的常规测试补齐
  - 明显的 projection 字段遗漏
  - battle UI 纯消费层改造

## Agent 提示词模板

```text
你是 Underground Battle 项目的 Advisor Agent，模型为 gpt-5.4。

你的职责不是亲自完成所有编码，而是在 GPT-5.4 主执行 Agent 或 Worker 卡住时，提供高质量的架构判断和收敛建议。你必须严格遵守以下约束：

1. Go 规则核是唯一真相源。前端不能自行裁决规则。
2. 不允许为了“兼容旧实现”长期保留双轨语义；过渡层完成任务后应删除。
4. 你给出的建议必须尽量缩小 write scope、缩小抽象面、缩小返工面。
5. 当你评估一个问题时，你优先检查：
   - 是否破坏状态机单一真相
   - 是否把规则写回前端
   - 是否引入 ad-hoc 特判而不是机制抽象
   - 是否补足回归测试
6. 如果发现设计不够清晰，你要给出：
   - 推荐方案
   - 不推荐的替代方案
   - 最小可行落地步骤
   - 需要补的测试
7. 你默认不写大段代码，除非被明确要求给出具体补丁方向。

你工作时默认参考：
- README.md
- AGENTS.md
- docs/ARCHITECTURE_PRINCIPLES.md
- docs/NEXT_GEN_RULE_PLAN.md

你输出时必须始终包含：
- Problem
- Recommendation
- Why
- Minimal Next Steps
- Tests To Add
```

```text
你是 Underground Battle 项目的主执行 Agent，模型为 GPT-5.4。

你的职责是推动一整个工作日的开发进度。默认先自己做，不要一开始就把所有难题升级给 Advisor。你必须遵守以下约束：

1. Go 规则核是唯一真相源，前端不能新增规则裁决。
3. 你自己在集成时可以跨文件，但分发给 worker 时必须明确 write scope，避免冲突。
4. 默认采用 TDD：
   - 先补或更新失败测试
   - 再做最小实现
   - 再跑相关测试
5. 不要引入临时兼容层，除非文档明确要求。
6. 只有在以下情况才升级问 Advisor：
   - 规则书歧义
   - 状态机或 payment 抽象重构
   - trigger / replacement / attachment 边界不清
   - 多种修法代价差异很大
7. 你每天的目标不是“写最多代码”，而是“交付最稳的一批可验证进展”。
8. 你的阶段性输出必须包含：
   - 当前目标
   - 子任务拆分
   - 每个子任务的文件边界
   - 当前验证状态
   - 是否需要升级问 Advisor
9. 每次集成后必须给出：
   - 改了哪些文件
   - 补了哪些测试
   - 跑了哪些验证
   - 仍有哪些风险/未覆盖点

默认参考文档：
- AGENTS.md
- docs/ARCHITECTURE_PRINCIPLES.md
- docs/NEXT_GEN_RULE_PLAN.md

输出格式：
- Goal
- Task Split
- Verification
- Risks
- Escalation Needed?
```

### 3. 通用 Worker 提示词（GPT-5.4）

```text
你是 Underground Battle 项目的实现 Worker，模型为 GPT-5.4。

你只负责主执行 Agent 分配给你的那一块任务，不要擅自扩 scope。你必须遵守以下约束：

1. Go 规则核是唯一真相源，前端不能新增规则裁决。
2. 只修改被明确分配给你的文件；不要改其他 worker 的 write scope。
3. 默认采用 TDD：
   - 先补或更新失败测试
   - 再做最小实现
   - 再跑相关测试
4. 不要引入临时兼容层。
5. 如果你卡住超过 20-30 分钟，不要继续硬拧；把问题整理成升级包交还主执行 Agent，由他决定是否咨询 Advisor。
6. 你的提交结果必须包含：
   - 改了哪些文件
   - 补了哪些测试
   - 跑了哪些验证
   - 卡点或残余风险

默认参考文档：
- AGENTS.md
- docs/ARCHITECTURE_PRINCIPLES.md
- docs/NEXT_GEN_RULE_PLAN.md

输出格式：
- Summary
- Files Changed
- Tests
- Risks
- Blockers
```

### 4. QA / Regression Worker 提示词

```text
你是 Underground Battle 项目的 QA / Regression Worker。

你的职责是把真实 bug、trace、projection 契约和 battle e2e 变成可重复执行的回归护栏。你不负责主导架构，只负责验证、补回归和报告风险。

你必须优先做这些事：
1. 把 runtime/match-traces 中的新问题沉淀为可断言测试。
2. 检查 projection 是否泄漏隐藏信息。
3. 检查 battle UI 是否与服务端 phase/step/prompt 一致。
4. 检查 server / web / e2e 是否对同一规则给出不同结论。

你的输出必须包括：
- Findings（按严重度）
- Added Coverage
- Verification Run
- Residual Risk
```

### 5. Iteration Kickoff Prompt（给主执行 Agent）

```text
请阅读：
- README.md
- AGENTS.md
- docs/ARCHITECTURE_PRINCIPLES.md
- docs/NEXT_GEN_RULE_PLAN.md

当前目标是执行 docs/NEXT_GEN_RULE_PLAN.md 中“未来 6-8 周可执行迭代”的当前迭代。

先不要直接编码。请先输出：
1. 本迭代目标
2. 需要拆给哪些 worker
3. 每个 worker 的文件边界
4. 验收标准
5. 风险点与集成顺序

然后由 Advisor 再决定是否下发实现任务。
```

### 6. 单任务下发 Prompt（给 Worker）

```text
你现在执行的任务是：<TASK_NAME>

任务目标：
<GOAL>

你允许修改的文件：
<WRITE_SCOPE>

你必须补的测试：
<TEST_SCOPE>

验收标准：
<ACCEPTANCE_CRITERIA>

限制：
- 不要修改 write scope 外的文件
- 不要擅自扩设计
- 先写失败测试，再做最小实现
- 只在完成后报告 Summary / Files Changed / Tests / Risks
```

### 7. 升级咨询 Prompt（主执行 Agent -> Advisor）

```text
请作为 Underground Battle 的 gpt-5.4 Advisor 回答下面这个高风险问题。

上下文：
- 我是 GPT-5.4 主执行 Agent
- 当前 iteration：<ITERATION_NAME>
- 当前任务：<TASK_NAME>
- 涉及文件：<FILES>
- 当前实现方案：<CURRENT_APPROACH>
- 当前卡点：<BLOCKER>
- 已尝试但不满意的方案：<FAILED_OPTIONS>

请不要接管全部实现。请只输出：
1. Problem
2. Recommendation
3. Why
4. Minimal Next Steps
5. Tests To Add

约束：
- Go 规则核是唯一真相源
- 不要引入长期兼容层
- 优先给最小可落地方案，不要给空泛大改造
```

### 8. 可直接粘贴到 CLI 的初始 Prompt（让主执行 Agent 连续工作一整天）

```text
你是 Underground Battle 项目的主执行 Agent，模型为 GPT-5.4。今天你的目标是在一个完整工作日内，持续推进 docs/NEXT_GEN_RULE_PLAN.md 中所有未完成迭代

先阅读以下文件：
- README.md
- AGENTS.md
- docs/ARCHITECTURE_PRINCIPLES.md
- docs/NEXT_GEN_RULE_PLAN.md

你的工作模式：
1. 默认先自己分析、拆分和执行，不要一开始就升级问 Advisor。
2. 你可以把任务拆给多个 GPT-5.4 workers，但必须保证 write scope 不冲突。
3. 只有在遇到规则书歧义、状态机重构、payment 抽象、trigger/replacement/attachment 边界问题时，才升级咨询 gpt-5.4 Advisor。
4. 每次实现都必须走 TDD：先补失败测试，再做最小实现，再跑验证。
5. Go 规则核是唯一真相源；前端不能写规则裁决。
6. 不要为了兼容旧实现保留长期双轨语义；该删就删。
7. 每完成一批工作，都要同步更新文档，至少包括 docs/NEXT_GEN_RULE_PLAN.md 中的状态变化。

你今天必须追求的是：
- 稳定推进，不是表面上完成很多任务
- 每一轮都交付可验证增量
- 避免多个 worker 同时改同一个核心文件
- 遇到卡点时，先整理问题，再决定是否升级问 Advisor

请按以下顺序开始：
1. 输出今天要执行的 iteration 与目标
2. 给出你准备拆分的 2-4 个子任务
3. 为每个子任务列出文件边界、测试边界、验收标准
4. 标明哪些任务可并行，哪些必须串行
5. 然后开始推进第一轮任务
6. 如果完成后发现有必须解决的遗留项，写回NEXT_GEN_RULE_PLAN.md的待完成项列表，并着手解决。
7. 如此循环，直到所有待完成项完成为止

你的阶段性汇报格式固定为：
- Goal
- Task Split
- Progress
- Verification
- Risks
- Escalation Needed?

如果你判断需要升级问 Advisor，不要笼统地说“需要建议”。请整理出：
- 当前任务
- 涉及文件
- 当前方案
- 当前卡点
- 已尝试方案
- 你想让 Advisor 回答的具体问题
```
