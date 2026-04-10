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
- [ ] **PN-REG-001: 地区势力值动态计算**
  - 验收标准：地区卡 `EffectiveStats.Influence` 随驻场角色变化。
  - 当前状态：`ControllerID` 与 `InfluenceByPlayer` 已可根据本地区 ready 角色动态刷新，但还未把该结果正式折叠进 `EffectiveStats.Influence` 字段。
- [ ] **PN-REG-002: 地区计分快照**
  - 验收标准：每回合结束检测地区占领情况并增加玩家分数。
- [ ] **PN-REG-003: 胜利判定规则**
  - 验收标准：分数达到阈值（如 100）时自动触发 `MatchFinished`。
- [ ] **PN-REG-004: 地区补充 / World Deck 生命周期**
  - 验收标准：地区赢取后，旧地区清场、进分区、补充新地区、相关触发与快速窗口顺序正确。
- [ ] **PN-REG-005: 地区级投影契约**
  - 验收标准：控制者、地区势力分布、地区替换过程在 player/spectator projection 中保持稳定，不再依赖 battle UI 猜测。

### PN-ACT: 规则书行动权与对抗流程 (Phase 3 Active)
- [x] **PN-ACT-001: 主行动步骤拆分**
  - 验收标准：`main` 拆成 `first_player_action / second_player_action`，后手行动步骤由后手拿初始行动权。
- [x] **PN-ACT-002: 正式 conflict phase**
  - 验收标准：地区按顺序进入调查 / 战斗 / 势力对抗，不再依赖 battle UI 手动 `declare_investigation`。
- [x] **PN-ACT-003: 调查奖励 / 战斗奖励 prompt**
  - 验收标准：调查奖励按差额检视并抓 1；战斗奖励进入伤害分配 prompt；二者均不入栈。
- [x] **PN-ACT-004: 现身与行动能力正式动作**
  - 验收标准：`reveal_face_down` / `activate_ability` 进入统一 legality + stack 管线。
- [ ] **PN-ACT-005: tie + 先手标志特权并入新 conflict 流程**
  - 验收标准：调查 / 战斗 / 势力对抗打平时，先手标志特权能在新 phase/stage 模型下正确接管。
- [ ] **PN-ACT-006: 拦截 / 更完整战斗结算**
  - 验收标准：攻击者横置 -> 对方指定拦截 -> 计算伤害 -> 致命判定 -> 离场。
- [ ] **PN-ACT-007: 能力注册表扩面**
  - 验收标准：行动能力不再只有示例卡，battle 常见 quick/action abilities 都能正式发动。
- [ ] **PN-ACT-008: 前端能力驱动化**
  - 验收标准：battle UI 读取 `capabilities + pendingPrompt` 渲染可行动作；旧动作类型下拉不再承担规则入口。
- [ ] **PN-ACT-009: 响应窗口完备化**
  - 验收标准：行动阶段、对抗前后、堆叠结算后都能按规则书正确重开行动权。

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
- [ ] **PN-SYS-006: 状态机可观测性**
  - 验收标准：match trace 能明确记录 phase/step/conflict/prompt/stack/priority 的转移，便于复盘 bug。

### PN-PAY: 规则书支付模型
- [ ] **PN-PAY-001: queue_operation / activate_ability 全面并轨 PaymentEngine**
  - 验收标准：正式 battle 动作的费用检查与扣费不再散落在各 action 文件。
- [ ] **PN-PAY-002: PaymentModeRulebook 初版**
  - 验收标准：支持“横置资产得费、步骤结束清空”的基础规则书支付模型，并可与 prototype mode 切换。
- [ ] **PN-PAY-003: 成本语义模型化**
  - 验收标准：资源费、横置费、弃牌费、牺牲费、移除标记费使用统一 cost spec。

### PN-CARD: 卡牌接入与 DSL 扩展
- [ ] **PN-CARD-001: 能力注册表与 DSL 分层**
  - 验收标准：battle 核心常见能力可判断“走显式注册表”还是“走 DSL”，边界清晰。
- [ ] **PN-CARD-002: 进场 / 离场 / 现身触发样板能力**
  - 验收标准：至少一批代表卡可通过统一机制实现，不再新增 ad-hoc 特判。
- [ ] **PN-CARD-003: 共享规则夹具**
  - 验收标准：Go authoritative parser / rule table 可供 web / e2e 直接消费，避免前后端重复解析。

### PN-QA: 测试与交付护栏
- [ ] **PN-QA-001: battle trace 回归套件**
  - 验收标准：真实 `/runtime/match-traces/*.log` 能沉淀为可重放、可断言的回归样例。
- [ ] **PN-QA-002: projection golden tests 扩面**
  - 验收标准：暗藏者、prompt 私有信息、地区替换、附属解绑等高风险投影都有 golden coverage。
- [ ] **PN-QA-003: 跨层一致性测试**
  - 验收标准：server legality / projection / web composer / Playwright battle flow 对同一规则不再各说各话。
- [ ] **PN-QA-004: 文档与里程碑同步机制**
  - 验收标准：每轮机制级重构必须同步更新 README、NEXT_GEN_RULE_PLAN、专项设计文档。

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
- `1 个 Advisor（gpt-5.4）`
  - 职责：架构把关、任务拆分、review worker 产出、拒绝错误抽象
- `2-4 个 Worker（GPT-4.1）`
  - 职责：按明确边界实现具体任务
- `1 个 QA / Regression Worker（GPT-4.1 或 gpt-5.4-mini）`
  - 职责：整理 trace、写回归、跑验证、报告残余风险

### 运行规则
- Advisor 不直接吞掉全部实现，只负责：
  - 把任务切成互不冲突的 write scope
  - 明确文件边界和验收标准
  - 审阅 worker diff，拒绝破坏“Go 唯一真相源”的实现
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

## Agent 提示词模板

### 1. Advisor 系统提示词

```text
你是 Underground Battle 项目的 Advisor Agent，模型为 gpt-5.4。

你的职责不是亲自完成所有编码，而是做架构把关、任务拆分、代码审查和集成决策。你必须严格遵守以下约束：

1. Go 规则核是唯一真相源。前端不能自行裁决规则。
2. 不允许为了“兼容旧实现”长期保留双轨语义；过渡层完成任务后应删除。
3. 任务必须按文件 ownership 切分，避免多个 worker 修改同一 write scope。
4. 每个任务都必须有明确验收标准、验证命令、以及需要补的测试。
5. 当 worker 给出实现结果时，你优先检查：
   - 是否破坏状态机单一真相
   - 是否把规则写回前端
   - 是否引入 ad-hoc 特判而不是机制抽象
   - 是否补足回归测试
6. 如果发现设计不够清晰，你先重写任务定义，而不是让 worker 硬上。

你工作时默认参考：
- README.md
- AGENTS.md
- docs/ARCHITECTURE_PRINCIPLES.md
- docs/NEXT_GEN_RULE_PLAN.md

你输出时必须始终包含：
- 本轮目标
- 拆分后的 worker 任务
- 每个任务的文件边界
- 验证要求
- 集成顺序
```

### 2. 通用 Worker 提示词

```text
你是 Underground Battle 项目的实现 Worker，模型为 GPT-4.1。

你只负责当前 Advisor 分配给你的那一块任务，不要擅自扩 scope。你必须遵守以下约束：

1. Go 规则核是唯一真相源，前端不能新增规则裁决。
2. 只修改被明确分配给你的文件；不要改其他 worker 的 write scope。
3. 默认采用 TDD：
   - 先补或更新失败测试
   - 再做最小实现
   - 再跑相关测试
4. 不要引入临时兼容层，除非 Advisor 明确要求。
5. 遇到需要跨模块抽象升级的情况，先停下来汇报给 Advisor，而不是自己拍脑袋扩设计。
6. 你的提交结果必须包含：
   - 改了哪些文件
   - 补了哪些测试
   - 跑了哪些验证
   - 仍有哪些风险/未覆盖点

默认参考文档：
- AGENTS.md
- docs/ARCHITECTURE_PRINCIPLES.md
- docs/NEXT_GEN_RULE_PLAN.md

输出格式：
- Summary
- Files Changed
- Tests
- Risks
```

### 3. QA / Regression Worker 提示词

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

### 4. Iteration Kickoff Prompt

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

### 5. 单任务下发 Prompt

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

## 实操建议

- 如果要让 Agent 群稳定推进半年，不要一次性把“半年计划”全扔给所有 worker。
- 正确方式是：
  1. Advisor 每次只拉起一个 iteration
  2. 每个 iteration 最多并行 3-4 个 worker
  3. 每轮集成后再开启下一轮
- 否则最容易出现：
  - 多个 worker 同时重写同一个状态机
  - 前端和后端对规则做出不同假设
  - 文档计划先于代码漂移
