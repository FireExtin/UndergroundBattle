# 后台 AutoPass 3 秒轮询实现：待发布复核

输入：父线程已审阅 `AUTOPASS_BACKGROUND_PROPOSAL_2026-10-03.md` 并批准实现，要求先交完整生产 diff、回归与实际合成测量，暂不部署。生产基线为已发布 Git `0ec1d7643fff64b3f8c98cdac343b11fea49d10e`，Site16 源码 `f5764e8a99f74585eedd74a82786912f20e837fe`。唯一执行者，没有子代理，没有访问父线程或任何真实房间。

输出：`useGame.ts`、`AutoPass.tsx`、`Table.tsx`、`GameApp.tsx` 四个运行文件；默认轮询回归 `AutoPassPolling.test.jsx` 及其合成 fixture；本说明及 `docs/evidence/autopass-background-implementation-2026-10-03/` 的完整 diff、日志和实际请求轨迹。本地分支 `codex/autopass-background-implementation-20261003`，从已批准提案提交 `59f229cab6f2a5a0aceec358a4c0b3fd04e597a6` 继续，未推送或部署。

## 最终行为

- 后台：最后接受的状态为 playing 且内存 AutoPass 开启时等待 3 秒，不要求旧视图已轮到本人；其他后台状态仍等 12 秒。前台仍 1.5 秒。无活跃席位的公共大厅仍无周期状态 GET。
- 私有选择、他人选择及全部响应意图状态只继续同步，仍暂停自动决策。唯一合法 pass 且在线、无 busy/uncertain/选择/响应窗口时，沿用原 550 毫秒提交与按房间/版本/pass ID 去重。
- 偏好由 `useGame` 唯一拥有，AutoPass 为受控组件；沿用 `hegemony.autoPass.v1` 和现有玩家存储命名空间。存储写失败仍使用本玩家内存值。模式切换重新读取相应范围；其他标签 storage 事件不会覆盖本标签活跃内存值，刷新才读持久偏好。
- 开关或接受状态导致等待长度改变时，只重排健康空闲 timer，从变化时起按新长度等待；同长度或新版本本身不重排。正在 GET/确认处理中只更新策略，完成后排下一轮，不取消在途请求或创建重叠读取。轮询 effect 仍仅依赖 session 与稳定 accept。
- 204 保留最后接受的状态作为周期依据。单调接受版本阻止旧 playing 视图恢复已结束牌局的快轮询。创建/恢复/切玩家/回大厅/失效座位同步清理最新视图；原 controller abort 与 room/seat/token 校验阻止迟到旧席响应及同 token 返回的 ABA 响应污染新视图。
- 前台恢复仍立即同步；切后台可执行原先已排的最后一次前台读取，此后按后台周期。现有离线 2/4/8/15 秒退避、API 12 秒超时、命令锁、原命令 ID/version/action 恢复均保持原逻辑。`api.ts` 和 `playerStorage.ts` 未修改。

## 实测与成本

使用实际生产 `useGame` + 受控 AutoPass、jsdom 假时钟、合成 fetch。未运行公网四浏览器游戏；网络 RTT 为 0，除专门延迟的在途测试外无处理等待。

| 同一“旧视图未轮到本人”场景 | 发布版本观测 | 当前实现实测 |
| --- | ---: | ---: |
| 合成服务端变为新版本唯一 pass | 1ms | 1ms |
| GET 发现 | 12,000ms | 3,000ms |
| 自动 POST | 12,550ms | 3,550ms |
| 事件至 POST | 12,549ms | 3,549ms |

实际改进为 9,000ms。当前默认测试完整执行到 3,550ms 命令断言，已经是绿证据，不再仅为提案预测。比较的旧轨迹来自已保留、注明生产基线的上一轮观测。

60 秒每席实际周期 GET：后台开启 playing 20 次，后台关闭/房间 lobby/finished 各 5 次，前台 40 次，公共大厅 0 次。连续 204 和连续 20 个新版本都只保留一条轮询生命周期，未重新完整 GET。

由独立单席实测相加：1 前台 + 3 后台开启为 100 次 GET/分钟（旧 55）；4 后台开启为 80（旧 20）；4 前台仍 160；4 后台关闭仍 20。这些是组成量计算，不是四浏览器同时公网实测。初始 GET、catalog 和命令 POST 不计入周期数；204 仍消耗现有 Worker/D1 读取及期限检查。浏览器节流、网络 RTT、等待命令确认会延长实际等待。旧 lobby 视图仍可能等 12 秒才发现开始 playing；本实现保留该范围。

## 验证

默认 `npm test`：**22 个文件 / 225 项全部通过**；其中新增 **38 项**快轮询回归，前四项是批准提案的四个红标准，已纳入默认。原提案独立脚本的四项也按原步骤重放，**4/4 通过**。现有六项 AutoPass 单元测试改为受控输入测试；偏好读写和保存失败由真实 hook 集成测试覆盖。`npm run build` 的 TypeScript 两个配置与 Vite 构建通过。默认 Vite 测试配置及依赖未修改。

38 项实际轨迹覆盖：

- 未轮到本人也在 3 秒发现；途中开启重排；等待他人选择继续快同步；偏好写失败仍快同步。
- 关闭取消旧 550ms 定时器并降频；poll/command 接受 finished 均降频；后续 204 和旧 playing 响应不能恢复快同步；前台开关及可见性节奏保留。
- 版本更新不重建 effect；lobby→playing 的旧 12 秒发现边界；在途 GET 开/关及可见事件不重叠、不 abort，完成按最新内存策略排 3/12 秒；离线退避不受开关干扰。
- 普通/独立范围、刷新、切回普通偏好恢复；外部 storage 事件与内存唯一来源；回大厅和旧同房间其他 seat 的迟到 200/401；同 token 与玩家模式 ABA 的旧响应拒绝。
- 在途 POST 阻止新动作，较新的 finished poll 不被旧 ACK 回退；快 204 不重复同版本 pass，新版本允许新命令；手动请求不同动作和自动轮询恢复丢 ACK 都只重发原 actor/ID/version/paced action；刷新恢复原 persisted 命令且不回退新版视图。
- 正常 3 秒 poll 在 2.8 秒命令回包形成 pass 后，于 3 秒接受私有选择、公开选择或响应 undecided/composing/passed，取消原应在 3.35 秒提交的自动动作；这些期间继续 3 秒同步，未自动决策。无需可见事件、DOM 状态注入或真实 API。

所有合成请求的座位 token/房间/命令标识均为测试生成或明确 synthetic 字符串，不涉及真实凭据。

## 证据与复现

`production-vs-site16.diff` 是相对发布基线四个运行文件的**完整** Git diff。`frontend-and-tests-vs-site16.diff` 另包括受控单元测试、38 项默认回归及原红标准重放适配。`source-and-validation.json` 保存前后源码 SHA256、保护范围及测试结果。`actual-traces.json` 为全部 38 项请求轨迹，`actual-timing-summary.json` 为实测摘要。`default-frontend.log`、`original-four-now-green.log`、`frontend-build.log` 为原始输出。

源码、测试与说明通过 Git 空白检查；两个原始 `.diff` 保留 Git 空行上下文的单个空格，检查时仅排除这两个原始证据文件，未修改或裁剪补丁内容。

从 `web/` 运行默认回归与构建：

```sh
npm test -- --reporter=verbose
npm run build
```

显式生成实际轨迹（默认测试不写文件）：

```sh
HEGEMONY_AUTOPASS_TRACE_OUTPUT=/workspace/UndergroundBattle/docs/evidence/autopass-background-implementation-2026-10-03/actual-traces.json npm test -- --reporter=verbose
npx vitest --run --config proposals/autopass-background/vitest.config.ts proposals/autopass-background/acceptance-red.spec.jsx --reporter=verbose
```

上一轮 `observation.spec.jsx` 固定断言旧 12 秒行为，配合旧基线日志作为历史证据，不是当前候选的默认验收；当前实际观测在新增默认文件中，避免混淆“旧观测现在红”与产品回归。

## 未完成部分与发布边界

Rust、Rust-WASM、server、sites、shared、组织卡池数据、原图、依赖、默认测试配置、API 和玩家存储模块均与已发布基线无差异。没有重建 WASM/backend，没有更改 core、机制、卡牌、schema 或 engine 版本。Site16 工作区仍为 `f5764e8a99f74585eedd74a82786912f20e837fe` 且干净。

只完成了本地前端构建与验证，尚未部署，也未用合成测量替代父线程真实游戏体验。按本次明确指令，下一步等待父线程复核本候选；通过后可依既有同项目公开发布授权执行，无需再问用户。
