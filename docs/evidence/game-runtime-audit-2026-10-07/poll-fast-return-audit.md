# 未变化轮询早返回：独立只读源码审计

审计时间：2026-10-07。唯一源码基线为 `7e5cc3c243aa6d52d2de9c3275ed6a0af28d9092`，普通 clone `/tmp/undergroundbattle-shared-cloud`；所有产品读取均使用 `git show 7e5cc3c:<path>`，排除了其他执行者未提交的核心 A 工作。该提交 Sites 的当前核为 engine45/pool42/schema3（`sites/src/kernel.mjs:5–7`）。树内没有 AGENTS.md 或 `.agents/skills/**/SKILL.md`。本轮只读源码、测试定义和既有脱敏测试日志，未实现、未改 index/产品/Git、未重跑全量测试、未开浏览器、未访问线上 D1/Library、未清数据。

## 裁定

可以做安全的未变化 204 早返回，但**不能仅比较 room.version**。当前同版本 GET 负责落库到期的响应决定，忽略这一步会使窗口卡住。

最小可行方案是：认证后从 D1 当前主库读取与该 room 的状态、revision 一起提交的少量可信轮询元数据，只有 `afterVersion` 精确匹配、元数据完整且来源匹配、当前核支持该 tuple、认证席位有效、当前服务端时间尚未到最早自动推进时刻（或已确认没有自动推进时刻）时返回现有 204。其他情况全部回到现有 `refreshedRoom → pollRoom → CAS/system journal` 路径。没有全局 scheduler，也不使用 isolate 内旧缓存作决定。

这是可审查的设计结论，尚无优化实现或新方案测试通过的声称。本轮未发现当前“先 tick 再 204”路径违背这一语义；风险在于未来缩短路径时丢失约束。

## 1. 当前同版本 GET 实际做了什么

固定源码定位（以下路径/行号均属于上述固定提交）：

| 环节 | 定位 | 事实 |
| --- | --- | --- |
| 认证 | `sites/src/service.mjs:154–159`、`store.mjs:8` | Bearer token 取 SHA256，按 `(room_id, token_hash)` 查认证 seat；不会根据 afterVersion 跳过认证。 |
| 全房间读取 | `store.mjs:5` | `SELECT * FROM rooms WHERE id=?`，不仅取 `state`，也取 `initial_state` 等整行。现有 `version(id)` 在第 6 行仅查版本，但此路径未用。 |
| 当前核选择 | `kernel-router.mjs:12–18,29–31` | `supportsPacing` 和 `pollRoom` 分别选择核，选择过程读 opaque state 的 Rust identity；当前仅支持 schema3 + 当前 rules/pool/engine tuple，其他 tuple 返回 410。 |
| WASM | `rust-game-wasm/src/lib.rs:180–188` | decode 完整 RoomEnvelope、valid_seat、解析受信 now，再调用 transition 并序列化完整结果。`from_persisted` 自己先反序列化 header，再完整反序列化并验证窗口。 |
| 规则及投影 | `rust-game/src/room.rs:564–622` | clone 房间、expire_due；无变化仍构建 seat view，序列化 next.state，再序列化外层 transition。view 内有游戏视图与 legal_actions 计算（328–340）。 |
| 到期提交 | `service.mjs:73–85`、`store.mjs:40–47` | 若 tick 改变状态，CAS 更新同一 room 的 state/version/nonce，并同 batch 条件写 journal；冲突重新读真实状态，最多五次。 |
| 最后判断 204 | `service.mjs:161–166`、`index.mjs:33–35` | 刷新完成后才与 `afterVersion` 比较；相同返回无 body 的 204 + `Cache-Control:no-store`。没有把刚算出的 view/serverNowMs 送给客户端。 |

所以当前 204 已省响应 body 的传输，但没有省 D1 整行取回和 Rust 的 decode/clone/view/state/外层序列化。`index.mjs:25–26` 使用普通 env.DB binding；源码明确未用 read-replica Sessions。这里记录配置和调用方式，不把源码注释替代线上容量或性能实测。

## 2. 当前需要自动推进的状态

当前规则层没有名为 `tick` 或 `ensureWindow` 的公共函数；实际入口分别是 `transition → expire_due` 和 `synchronize_window`，不能按旧函数名猜机制。

* **唯一受墙钟驱动的状态变化**：当前 response window 中的 `Decision::Undecided` 成员，在 `now >= deadline_ms` 时真实让过。期限常量为 5,000ms（`room.rs:7`），比较包含恰好到期边界（377–415）。应取整个 window 中所有 Undecided 成员的最早 deadline，而非只取轮询者本人或某一个 seat。
* 一次 expire_due 取原窗口的到期成员，逐个应用规则 pass；每次调用 synchronize_window。若窗口 ID 已改变就停止旧窗口循环（392–409）。新窗口取得 `now + 5,000ms`，不能把旧窗口的下一批到期者作用于新窗口。next-wake 必须从**整个最终新状态**重新派生。
* 新窗口不是一个遗漏的后台任务：Game 动作、正式 SubmitResponse 强制同步窗口；PassResponse、CancelAndPass 和 expire_due 在优先权改变后同步窗口（449–561）。窗口适用条件是 `playing && pending.is_none() && stack.last().is_some()`；无适用堆顶则清窗口（272–319）。
* 从持久化状态读取时，适用堆顶必须与 window 的 stack_top_id、holder_team、序号和有效成员一致；每个 Undecided deadline 必须大于 last_server_now_ms（192–269）。因此合法已提交状态不会等下一次 poll 才补建一个无期限的新窗口。无窗口而有适用堆顶是非法状态，应走现有校验错误，不能被 fast 204 掩盖。
* **不自动推进**：Composing 没有期限；Pending 游戏选择暂停窗口；空堆、lobby/finished 不因 poll 自动换阶段、回合、行动或重启。不能对“正在编辑很久”“没有合法响应”新增自动让过。相关测试定义见 `room_pacing.rs:105–161,164–224,331–375,474–505`。
* 观察时间采用 `max(observed_now_ms, last_server_now_ms)`（573）；持久化合法 deadline 严格晚于 last_server_now_ms，因此 fast gate 比较受信 observed now 与可信 min-deadline 即可判断未到期。时钟来源、无符号整数及边界检查必须与核心一致；不接受客户端传入 now。未变化 poll 不持久化新的 last_server_now_ms。

这里的自动推进由 GET、命令、报价访问驱动（报价也先 refreshedRoom，`service.mjs:250–259`）。Sites 没有全局循环或常驻队列；所有客户端停止访问后不会在第 5 秒准点写 tick。后台 12 秒轮询可能更晚发现到期，早返回方案应保留这个边界，不能声称增加了准点 scheduler。上述“唯一时间推进”裁定限于固定核45；后续若增加其他无需玩家命令的自动推进，必须同时更新派生wake与相应验收，不能沿用本次null语义而漏掉新规则。

## 3. 最少元数据及安全条件

建议只给 `rooms` 加**一个 nullable、小型 poll_meta 列**（或等价的少量列），不改 Game/RoomEnvelope 的持久化内容、不加游戏实例 ID/队列。概念内容：

```text
format: 一个受支持的元数据格式版本
sourceRevision: 最终 RoomEnvelope.revision
sourceAttemptNonce: 对应现有 rooms.attempt_nonce（可复用，非新身份）
stateSchema + rules/pool/engine: 最终状态的完整当前支持身份
seatCount: 已验证的连续 seats 数量，用于保持 valid_seat 约束
nextWakeMs: 所有当前 window.Undecided.deadline_ms 的最小值；确定无值时为 null
```

`poll_meta IS NULL` 表示**未知**；`poll_meta` 内的 `nextWakeMs:null` 才表示**已验证无时间推进**，两者不能混为一谈。核心应从最终已验证 RoomEnvelope 输出这个有限派生值，或提供小型固定 ABI；不要在 JS 通用解析私有游戏 JSON 或在前端自行推导规则。metadata 位于 host 存储/外层结果，不需写入 Game/历史 room 字节。时间和 revision 若经过 JS，应按受校验的十进制整数表示/比较，避免 u64 转 Number 丢精度；当前 Date.now 也须是合法受信毫秒。

四种现有状态写入路径都要覆盖：create、join、command、system tick（`store.mjs:13–47`）。新的 metadata 与 state、version、已有 nonce 必须在**同一条 UPDATE/INSERT、同一 batch/CAS 成功**后生效，而非事后单独补写。接受动作和“用户动作失败但 tick 成功”的 system 写入均要派生最终值；CAS 失败者不能留下 metadata。未来 restore/reset/import 也必须原子更新或显式置未知，不能只恢复 state。

sourceRevision 不能省掉：旧 writer 可递增 rooms.version 却不更新新列，旧 poll_meta 会被保留。来源 revision/nonce 不匹配时 fallback，即可避免把旧无 wake 元数据用于新窗口。若不支持原地同版本状态替换，仍建议绑定现有 nonce，便于恢复/延迟补元数据时复用现有 CAS 保护。

推荐 state?afterVersion 的流程：

1. 保持现有 token/room/seat 认证。只有请求带 afterVersion 才尝试小型 header 查询，初次完整 GET、quote、command 保持现有入口。
2. 从同一主库 room 行读取 `version,attempt_nonce,poll_meta`；不读 `initial_state/state`。不存在、未知、无法校验的 header 保持现有错误/完整读取路径。任何缺字段、未知格式、不合法整数、来源不匹配均 fallback，不默认为无 wake。
3. 身份完整且可信时检查当前支持 tuple；已知旧 tuple 仍按当前 410 停止支持，不能因为版本相同一直给 204。认证 seat 范围也须保持 valid_seat 检查，否则应走完整校验/拒绝。
4. 在读 header 后取得受信服务端 now；仅当 afterVersion 精确匹配 header.version，且 `nextWakeMs == null || now < nextWakeMs` 时返回原来的 204/no-store。恰好 `now == nextWakeMs` 必须走完整推进。
5. 其余情况重新读真实完整 room，由现有 pollRoom 处理到期、窗口变更、private view 与 CAS。不要用先前 header 对应的旧缓存 state 继续 tick。

不强制给旧行做在线回填：未知行正确地走完整路径，下一次真实成功写入即可建立 metadata。若后续确需延迟回填，应只在完整 decode/校验后，以原 revision+nonce 作条件更新派生列；不加游戏 revision/journal/command receipt，条件失效就丢弃结果。无此回填实现也能正确运行，只是这些行暂时不获快路径收益。

同一次主库读取建立观察点；header 读取以后并发另一席提交新版本，允许本次 204 表示该观察点尚未变化，下一次轮询看到新版本。当前完整读取后也可能发生并发写，不能要求整个请求期间游戏停止变化。到期慢路径的竞争继续由同 room CAS 决定，不能给所有 room 加全局锁。若将来使用副本/缓存，必须重新证明读取的一致性；单纯 TTL 缓存不满足此条件。

## 4. 前端 since、通知和 serverTime

* 参数当前名是 `afterVersion`，不是 since 时间戳。首次 getState 不带它；后续取 `acceptedVersion.current`，严格接受同 room 的安全整数版本（`api.ts:112–115,153–162`、`useGame.ts:40,100–112`）。不是“客户端最后一次 poll 的时间”。
* 收到 204 时 pollState 返回 null；useGame 不改 view/acceptedVersion，但照常设 online、清离线退避，并在无 commandLock 时 resolvePending 原命令（`useGame.ts:105–112`）。fast gate 不得把这个响应换成失败或停轮询，也不能跳过恢复未知 ACK。
* view 接受按 room+you 防混席，activeSession 另检查 room+seat+token；旧 version 不回退，同 version 的实际 200 view 可以更新（7–8、42–58）。不应为时间样本给游戏加 revision。父设计 reset 时不要复用原 roomId 并把 version 归零：现有客户端单调 acceptedVersion/newerView 会忽略较低版本；新局沿用现有新 roomId/新 session 即可，无需新游戏身份框架。
* ResponseWindow 用 `(deadlineMs - serverNowMs)` 和收到样本时的 `performance.now()` 计算本地显示，100ms UI timer 到零后显示“等待服务器确认”，不会自行 pass（`ResponseWindow.tsx:23–37,45–67`）。serverNowMs 是 view 的时间样本，不是持久化版本；现有同版本 204 本来就丢弃新样本。快返回保持此语义，不需 204 body，也不应拿客户端 Date.now 替造新 serverNowMs。需要重新校时的完整 200/命令 view 仍保留。
* AutoPass 在 responseWindow/选择/busy/uncertain/offline 时暂停；只对空窗口的唯一 pass 经过 550ms 和正常 command 管道，同 room/version/pass ID 去重（`AutoPass.tsx:14–28`）。不得把它误当服务器到期逻辑。
* 当前 Sites events 接口直接 410，前端 useGame 使用轮询；API 中残留 streamEvents 函数未被 useGame 使用（`index.mjs:39`、`api.ts:191–215`）。固定 web runtime 搜索未发现浏览器 Notification/音频通知，故此方案没有要维持的 push 通知。新版本被 200 接受后触发 UI 是实际通知方式。Native service 的 room-local watch/SSE 是另一后端路径（`rust-game/src/service.rs:737–750`），不能当成 Sites 的全局推送/自动 tick。
* 现有 cadence：前台 1,500ms；后台 playing+AutoPass 开启 3,000ms；其他后台 12,000ms；每次请求完成后排下一次，恢复可见立即同步，仍只有一条不重叠轮询生命周期（`useGame.ts:77–143`）。只改服务端 early return 不更改 cadence。

## 5. 反例和必须验证的边界

| 反例 | 必须保持的结果/测试 |
| --- | --- |
| v4 相同，now 恰好等于 deadline | 不返回 fast 204；一次真实 tick 落 v5+journal 后返回 v5 view。deadline−1 可 fast 204，deadline、deadline+1 必须慢路径。 |
| 轮询者已 passed/不是优先权队成员，另一席 Undecided 到期 | metadata 包含全窗口 min-wake；不能依据“本人无待办”204。覆盖 duel、2v2 一人 composing 一人到期及两人同时到期。 |
| tick 交出优先权，创建另一团队新窗口 | 最终 nextWake 为新窗口 now+5秒；旧窗口 ID/due 列表不串到新窗口；同 now 再 poll 不连环超时。 |
| 已 BeginResponse 的 composing 或 pending choice 等待很久 | 无对应自动期限；不发人工 pass、不自动提交/支付，原 intent/choice/private data 保留。 |
| 旧 metadata 无 wake，另一次 Game/Submit/Cancel/choose 写出有 wake 新版本 | 所有成功 CAS 同步修改 metadata；来源 revision/nonce 不同即 fallback；重启 worker 也从 D1 读，不能用 isolate 缓存。 |
| metadata缺失/坏值/未知格式/旧 writer 遗留；合法 state 的 window 不匹配或非法seat | 未知不等于无 wake；完整 from_persisted/valid_seat 检查继续报错。覆盖 schema2/旧 tuple同版本，仍410；不能fast掩盖非法状态或旧核。 |
| 两个隔离 worker 同时 poll 到期，或 poll 与 Begin/Submit 竞争 | 只一个 CAS 成功；每个成功 revision 只一条合法 journal；loser 重新读完整状态/重算 metadata，paid submit 不擅自 rebasing。事务故障不能留下“state旧、wake新”或相反。 |
| 一个 room fast 204，另一个 room 有到期/玩家写入 | 无 room 间 wake/metadata/seat/receipt 污染；独立完成，不为测试引入全局 scheduler/锁。测试并发正确性不等于测试容量。 |
| 丢 ACK 的原 paid 命令，之后 v相同fast204或tick推进 | 前端仍 resolvePending；后端原receipt仍先于时间计算；同 command ID/actor/原 expectedVersion/action 原样恢复，不重复支付。 |
| 204反复、后台→可见、网络慢/abort/切room/切seat及same-tokenABA | 保持online/退避/单poll/最新version/pending恢复/迟到响应丢弃；倒计时继续耗时，到零只等待server；完整200仍正确重采样。 |
| server clock 回退/向前跳/整数越界，读取时刻跨到期 | 与核心 max(lastServer,observedNow) 等价；不得用客户端时间，边界不会提前或漏过。fast决策的服务端观察点与并发结果需明确。 |
| restore/reset/import 或修改/清空 metadata | state/version/metadata原子一致或置未知；不复用低version同room的客户端since；删除/重置身份与认证先于任何204，不能由旧缓存隐藏。 |

新增方案还需有真实当前 WASM + local D1/重开的端到端边界测试，证明metadata与最终 opaque state一致；只用 fake kernel 测 host分支不足。验证收益可用本地 query/ABI 调用记录：fast请求不选择 state/initial_state、不调用 identity/poll/view；slow请求仍调用真实当前核心、正确落库/replay。必须记录请求、WASM调用、事务与journal事实，不从测试数外推容量。

## 6. 哪部分成本能省，哪部分仍在

| 方案 | D1整行/全state读取 | Rust成本 | 代价/结论 |
| --- | --- | --- | --- |
| 现有204 | 保留：SELECT * 包含initial_state和state | identity扫描、完整decode/校验、clone、view/legal-actions、state与transition序列化均保留 | 已省HTTP body，语义正确。 |
| 只改Rust为合法未到期且同version的轻量返回 | 保留整行；仍要把opaque state送WASM | 可以省clone/view/两个序列化；保留identity和完整decode/校验 | 可分阶段做，但需要明确轻量ABI/host处理；不是D1减载方案。 |
| 权威D1轻量元数据 fast gate | 合格同版本请求不SELECT state/initial_state，仅header和认证 | 整个state identity/pollRoom/decode/clone/view/serialize都可跳过 | 推荐；需要每个写路径的原子元数据和unknown fallback。 |
| 版本或旧缓存直接204 | 可以少读 | 可以少算 | 不安全，漏到期/旧核拒绝/恢复/并发真实变更。 |

现有认证和room读取本来是两次查询；复用认证+独立header的最小fast路径也仍是两次查询。发生变化/到期的请求可能多一次小header查询再读整行；初次无afterVersion的GET可直接走现有完整路径。把认证与header做同一个索引JOIN是后续可选优化，不是安全early return的前提，也不必本批加入。

“不选择大TEXT”证明的是取回/反序列化/Worker内存成本减少，不等于证明 SQLite物理页面I/O、D1扫描计费、CPU时间或并发容量按某比例减少；行的物理存储仍存在。此审计没有RPS、延迟、峰值并发、quota或容量实测数字。

## 7. 已有证据边界

* 核心定义的现有 native测试：`rust-game/tests/room_pacing.rs:105–161` 到期前后、fresh window/no-cascade；164–224 composing untimed/cancel真实pass；331–375 teammate timeout+private projection；474–505 pending choice/empty stack无automaticadvance。它们为语义依据，本轮未重新编译/运行。
* host测试：`sites/test/session-host.test.mjs:44–55` 明确断言“matching-version GET persists expired window before deciding204”；kernel是mock，不能替代真实WASM metadata一致性测试。30–42原receipt先于clock、57–64reject仅systemtick、66–100CAS原命令/paid不rebasing、114–127quote刷新不付费都要保持。
* 固定提交已有 `docs/evidence/jc089-combined-2026-10-07/combined-worker-tests.log`，原始末尾为25 tests /25 pass/0 fail，包含上述host断言和localD1/历史拒绝项。是既有本地验证日志，本轮只读取证，没有声称新early-return方案通过25项。
* 前端测试定义：`ResponseWindow.test.tsx:45–57,79–101,129–135` 本地倒计时/服务端样本/untimed选择；`AutoPassPolling.test.jsx:64–76,268–293` 合成假时钟cadence、连续204不重复pass和原命令ACK恢复；`useGame.test.tsx:284–307` 不重叠/可见恢复/旧view不回退；`api.test.ts:109–122` auth/no-store/204/null及错误room。均非线上负载测试。

建议父在接口小批接线时以本节和反例表作验收，保持“未到期fast/到期交现有核心/CAS/未知fallback”的单一范围。永久删除、线上清空与Library整理不属于本次只读审计，未执行；此前保留/隔离/可恢复reset建议在 `/tmp/game-runtime-retention-isolation-audit.md`。
