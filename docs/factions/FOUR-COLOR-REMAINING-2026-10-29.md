# CQ27恢复核对及当前四色剩余范围

核对基线 `9de3cde8663200452c5a41d3342572879a41e6b5`；工作区干净、无活动Native/Worker测试，原exec断连已恢复。此记录不重跑测试，不开放卡牌，不构成独立审查。

## CQ27准确范围与恢复身份

实现提交 `367dd32725aecb80a0833ca76bc400028395e4ec`，依赖 `dacb2ed0964e9372f7c60856219a703f53665ff5`。正式黑卡CQ27的普通派遣/潜伏/现身、混乱进场、明置禁止移动、公开条件非资产、墓地牺牲三个自己的明置角色回场、准备置底本人墓地角色并生成迟缓行尸均已封闭实现。TK001为既有支撑，不计另一张新增正式卡。目录307/公开236为该提交身份，不代表后续JZ53提交的目录308。

该提交Native本组11＋相关37＝48个不同测试通过；同源默认WASM186命令/744视图/186实际重复；UI8文件151/151；Worker3场景158命令/632视图/158重复与冷重开。自然准备前28命令仅Native/WASM。未重跑全库，未真实浏览器试玩，未独审，准入关闭。原两轮失败保留于CQ27证据包。

- `/workspace/cq27-validation/delivery/cq27-recovery.bundle`：436100字节，SHA256 `8ba57625e3a63bbcedfcc108f3b2f2ab14fad71ee01c9093a7581a6468532ab8`。
- `/workspace/cq27-validation/cq27-367dd327-evidence.tar.gz`：2549785字节，SHA256 `3cdac6d0394cea3742aab101d3b99b6a23161cf06868737546efa77c645e412e`。

本轮重新计算以上哈希；原bundle验证记录保留，不重建原包或覆盖旧失败。证据均仅云端本地，不宣称Library已交付。

## 重新计算的未登记范围

从当前源码WASM目录的cards＋societies对照683条原图规格，其中红/黑/蓝/灰且非TEST的来源记录269条，尚未登记26条。26包括TK009指示物及ZHWM026背面，不等于26张可独立完成正式卡。原图文件均存在，26条SHA256逐项符合固定规格。完整逐卡原文/来源/哈希保存在 `/workspace/four-color-remaining-validation/after-9de3cde8.json`。以下是待解决依赖，不能把“未登记”本身当成规则阻塞。

| 范围 | 实际待解决点 |
|---|---|
| BQ043、BQ058、BQ066 | 完整任务地区入口涉及当前暂停的中立牌范围；并非缺失原图。已登记BQ049奖励亦未因此补造。 |
| BQ047、CQ13、JZ42、MSJZ03、TK009 | 末日钟同组；MSJZ03位于秘社区时地区目标适用性、JZ42红色费用轮廓待明文。TK009费用与生成生命周期已解决，不再列旧费用阻塞，亦不单做指示物凑正式卡。 |
| CQ12、CQ14 | CQ12从手牌先兆的“本地区”没有场上来源锚点；CQ14完整墓地重打依赖“名为盖罗涅”角色入口，不仅解除准入凑半张。 |
| JC028、MSBQ01 | 本轮重读原图；转换后未明示属性／原能力如何保留，以及鲜血离场可选触发在默认入墓清理中的时序。不得套用另一张同名鲜血的数据或改成无条件强制封印。 |
| WM063、MSWM03 | 黑市没有结附行，缺默认宿主条款；秘社组依赖真实黑市入口。 |
| MSLC03 | 警察费用/生命周期已解决，但其支援牌库、跨局放弃/名誉与城市模式需要完整洛城规则；仓库现有说明未给完整模式。 |
| BQ044 | “敌方玩家牺牲”在当前四人模式中是一名还是双方两名对手。 |
| BQ095 | 重复总部及生成角色生命周期先前裁定边界，基础层不计完整。 |
| JC053、JC097 | 分别为本回合已发生移除检查/未来等待，以及墓地数量决定的袭击X取值时点。 |
| WM026、ZHWM026 | 死亡“转变，而非入墓”涉及同实体/新实体以及离场/进场观察。 |
| WM027、XQ13 | 第二个道途进入或控制变更时，禁止动作还是选择保留及另一张去向。 |
| LC04、WM031、WM036 | LC04颜色徽记冲突；另两张红色费用轮廓是否有独立时序含义。不能仅凭轮廓猜新增效果。 |

已登记但未完整的BQ048核心区重置/三失控响应补选、BQ049任务奖励继续保留既有边界。独审拒绝与暂停颜色是范围/权限限制，不写成官方游戏规则缺失。

## 本轮额外原件复核

实际查看了JC028、MSBQ01原始卡图以及《帷幕之后说明书.jpg》《星光俱乐部说明书.jpg》，并查规则手册X/袭击、发动、封印、自指名称段。扩展说明解释计划、隐秘对抗和传闻，并未提供红费用轮廓或上述鲜血转换/替代时序的额外裁定。手册X条只明确费用中的X在支付时决定，不能将此直接推广为JC097非费用的动态X一律冻结。

优先可解除蓝卡组的最小决定：

1. JC028是否仅更改卡文明示的名称/类型/图标/防御并增加宿主加成，其余原印刷颜色/领域/费用与原能力保留；或转为独立鲜血牌面（后者需要明确完整字段，不能凭同名牌替代）。
2. MSBQ01是否沿可选、可响应的入墓前挂起流程：接受并成功封印才替代入墓，放弃/未成功则正常入墓；不能先埋入墓地再凭同名检索，也不能无条件强制改封印。

本轮未替用户裁定以上内容，也未新增半成品或无关框架。后续若从原件得到明确依据，按该依据继续，不把这张依赖表作为永久不能实现的结论。无新机制测试；Library401/独审拒绝/push限制不重试，不合main、不发布。

## 后续增量去重（2026-10-29）

上表是当时范围快照，不是当前未实现清单。后续已封闭实现 JC097、JC028、JC053、MSBQ01、XQ13，分别见其机制文档与本地检查点；不可再次计为新增。WM027 本轮已完成封闭完整机制及同源 Native/WASM/UI/Worker 验证，见 `WM027-CLOSED-MECHANISM-2026-10-29.md`。红费用圈采用不额外赋予快速时机的封闭项目选择，未宣称官方解释。所有这些候选仍未独审、关闭准入；旧阻塞记录保留为历史，不再用已解决条目阻挡其他实现。

WM036 本轮已实现原图普通事务、非资产及可选计时者展示后置，Native 定向与相关回归、同源 WASM/UI/Worker 验证及前端构建已通过；见 `WM036-CLOSED-MECHANISM-2026-10-29.md`。独审/准入仍关闭，不能将红圈项目选择标成官方规则。

WM031 本轮已封闭完成：逐次放置及附属宿主首次观察、可响应强制选择、集中创伤、暴露条件混乱和同源跨端验证，见 `WM031-CLOSED-MECHANISM-2026-10-29.md`。太阳符号已按手册纠正为暴露条件，不是临时/先手图标。新增完整正式红卡1，独审0、准入关闭；不能再把它列为未实现，也不能用本次自测代替完整批次独审。

## BQ044 ordinary-trigger WIP (after WM031 witness closure)

The former single/all-enemies ambiguity is now a documented project choice: the resolving controller chooses one enemy player, without adding a printed-target contract. See `BQ044-WIP-2026-10-29.md`. The ordinary paid sacrifice trigger has a source implementation and targeted tests; core invocation and the natural entry chain remain engineering work. **Do not count BQ044 as complete or open admission.** This does not resolve WM026/BQ095 marker/lifecycle questions.

## BQ044 closure superseding the WIP status above

BQ044 ordinary trigger and core sacrifice→draw→repress continuation, private 50+1 natural entry, artwork/client mapping and applicable Native/WASM/UI/Worker recovery checks are closed. See `BQ044-CORE-CLOSURE-2026-10-29.md`. Count one complete closed implementation, zero independent approvals; public admission remains false. Earlier WIP and failed evidence remain historical records, not current missing work.

## WM026 closure 2026-10-30

WM026 is now a complete closed blue-card mechanism, including actual natural sacrifice→mist→preparation return. ZHWM026 is its reverse and does not increment the formal-card count. See `WM026-CLOSED-MECHANISM-2026-10-30.md`: Native68 distinct, UI165, WASM/Worker898 commands; independent review0, admission closed. Earlier WM026 engineering-pending entries are historical, not current blockers. BQ049 remains blocked on the paused neutral task-region scope. No other inventory entry is changed.
