# BQ104 / XQ48 云端有限进场检索候选

BQ104 猎头顾问与 XQ48 街头演说家的完整印刷能力、原始卡面及有限检索程序已接入候选。候选源提交为 `637a159`，在父端独审有限通过后合并 Go 退役提交 `7dab3a138a7ac85cd93f57a7565958a022b3be1f`，合并提交为 `54e0c68959cceeca26e938c8973980cbfdad7a51`。父端已审固定产品 `d55a740` 与 QA追加 `28a72af`；用户于2026-10-08 01:23 UTC明确同意最后的BQ原地区消失行为。**本次按既有授权执行官方Sites发布，最终发布结果以原生版本/部署回读receipt为准。**

## 版本和恢复边界

工作环境为 UndergroundBattle 云端，普通共享 clone `/tmp/undergroundbattle-shared-cloud`、协作分支 `codex/jz48-combined-review-20261007`。没有新建工作树、切换 Air 或改动 GitHub/main。Go 清理按父端独审结果保留；App 顶层直接运行 GameApp，AppShell、lazy debugger 和 Go 服务均已退役。README 的现役 Go 说明已修正。

| 项目 | 候选值 |
|---|---|
| 规则 | `hegemony-pdf-v1` |
| 引擎 | `rust-v0.2.50-bounded-entry-search-candidate` |
| 卡池 | `limited-v2.45-bounded-entry-search-candidate` |
| 普通卡 | 109，其中10地区、99非地区 |
| 会社 | 8 |
| 原有自组预组 | 5套、内容不变 |
| 原图注册 | 120条，原118条不变 |
| 当前 WASM | 2,277,324字节，SHA256 `e3d26f7041a114a96c9933374a33e63b0c8e45b9a4506b7937fb70cbf99a9d81` |

合并前保存34个 WIP 文件的大小和 SHA256，并先提交完整候选；解决唯一 JC089 测试冲突后再次逐文件核验，**34/34字节一致**。两方保留已准入 JZ50 和未准入 JZ51 的正确测试断言，没有恢复旧调试器。Go 清理对原卡 archive、规则 PDF、手编 rules、Rust data 和已发布 card assets 没有修改。父端提供的独审结果和本机保存检查是不同证据，不能把父端提及的未执行 CI 数字当成本机通过数。

旧107张卡的数据结构、5套牌组、world、原118条扫描注册及24份历史夹具逐一核验保持。历史 engine49 WASM 2,248,074字节、SHA256 `9b8a7a69cd3c09781387cb4e66ad710d803783447fb6b0382fb2697eac50e885` 已冻结保留；未改写 engine45—48 的既有文件。新 v050 trace 是当前 Native 真正生成的状态，旧 v049 等 JSON 未通过改版本字符串冒充新引擎。

**旧桌兼容范围：** 本批沿用既有 current-only Worker 路由。50构建对49及其他历史元组返回 `unsupported_room_version` / HTTP410；保留旧数据和冻结49不代表50可以继续旧桌。发布审查须明确接受这个实际边界，不能宣称无缝恢复 Site42 的旧桌。

## 实际原图和规则依据

已实际查看两张完整原图，复制到公开 assets 后与原文件逐字节匹配：

| 卡 | 文件 / 收藏编号 | 完整原图 SHA256 | 印刷属性 |
|---|---|---|---|
| BQ104 猎头顾问 | `BQ104 猎头顾问.jpg` / **097/116**，文件ID与收藏编号不同 | `dbee5b3829cf0c46b2820beccefda28708c3c8e87c589dbefce1e975faa916eb` | 费用3，中立，无忠诚/领域，人类/雇员，永久势力1，临时图标全0，防御1，公开，非独特 |
| XQ48 街头演说家 | `XQ48 街头演说家.jpg` / 48/52 | `0cd31207bda681093839e86e5bb87cee624a3c0a657c70b60fa765f439089746` | 费用4，中立，无忠诚/领域，人类/政治家，永久势力1，临时图标全0，防御1，公开/声望，非独特 |

BQ104 印刷进场触发从自己的牌库检索一张印刷费用不超过3的雇员角色，展示后作为暗藏者放入本地区，然后洗牌。XQ48 印刷进场触发检索至多3张名为无知路人的牌，直接置入街头演说家所在地区，然后洗牌。完整中文印刷文字保存在 catalog，阅读器使用真实原始卡面。

本轮实际查看了基础规则实体页3/4、霸权说明书实体页16/17、FAQ实体页3/5，涉及暗藏者无印刷角色能力、可选触发、直接进场、拥有/操控、来源离场后仍结算、原地区和公开牌翻暗；完整图片放在 trace 归档的 `rules/`。没有以未完整的 `manual-rules-16.png` 或 OCR 片段替代原页。`docs/factions/card-specifications.json` 保持历史审计记录，不能把该历史记录当作本批已通过审查/已上线证明。

## 三项已确认裁定及其原页证据

父端转交用户于 2026-10-07 22:52 UTC 回答“按你的来”，明确针对这两项：

1. **BQ104：** 可选进场触发可以拒绝；接受且牌库存在合法雇员时必须取1张，不能取0。没有合法雇员时提供显式0张完成并洗牌，UI不能死锁。
2. **XQ48：** 原地区实例在声明/响应后被替换时跳过检索与进场，仍洗牌。不能放入相同下标的新地区。

3. **BQ104原地区消失：** 父端于2026-10-08 00:08 UTC明确询问“仍选出并展示一张合格雇员；无法放置就留在牌库，然后洗牌”与“如XQ跳过检索，只洗牌”两种处理，用户于01:23 UTC回复“同意”，确认保留前一种候选行为。当前实现仍检索、强制合法1张并公开展示；原地区无法放置时，所选牌留在其牌库并按印刷顺序洗牌，不进入替换地区。**这是用户对该特定边界的明确裁定，产品无需修改。** 原文及范围见 [BQ104裁定记录](rules/BQ104_ORIGINAL_REGION_RULING_2026-10-08.md)。没有把XQ的跳过检索行为泛化到BQ，也没有把本裁定扩大为通用部分结算规则。

2026-10-08 按父端要求再次实际查看完整 BQ104 原图及相关规则原页。BQ104 的检索、展示、放置、洗牌顺序，以及霸权印刷第16页“来源离场仍结算”已核实；父端提到的“无效目标只让相关部分不结算”原句尚未在本机现有原页核实。霸权印刷第14页实际给出基础测试失去原目标后“未能生效就被置入了墓地”的示例。准确原句、PDF版次哈希和完整原页见 [规则来源复核](evidence/bounded-entry-search-2026-10-07/rule-source-recheck-20261008/README.md)。这些文字与 BQ104 的隐含放置地区是否属于目标是不同问题，本次不新增规则裁定。

## 最小实现和保护边界

新增参数无值的两个固定 Op 与对应 FrameChoice，只允许完整 BQ104 / XQ48 定义使用。复用现有 SourceSnapshot、ResolutionFrame、choice、zone reset、shuffle、Enter 触发和 room receipt，未添加通用绑定、检索框架或新的身份/队列机制；已准入的 JZ50 独立程序保持字节不变。

来源实例、当时操控者、owner、原地区下标和真实地区实例在触发前冻结。source_snapshot 的地区实例白名单只加入这两张。来源移动、离场、翻暗或改变操控后，接受的效果仍按冻结行动者检索；拥有者不决定效果 actor。若原地区实例不匹配，不会向同下标替换地区施加效果。

BQ104 的候选严格来自冻结行动者牌库内已准入、印刷费用≤3、雇员角色实例；选择合法1张后公开记录牌名，直接作为暗藏者进场，重新产生区间实例并保持 owner、以冻结 actor 操控。暗藏进场不发动印刷 Enter 能力；真实付费现身后才能触发其正常能力。XQ48 仅接受真实已准入的 `JC125` 无知路人牌库实例，0..3张作为一批全部放入原地区后再排入普通公开 Enter 触发，不制造 token。

候选仅对当前行动者可见，另外三席包含队友均不收到 options。空检索仍是 actor 私有的显式可完成选择。UI只给这两个固定 choice 的合法0张界限开放“不取牌并洗牌”，无关的强制空选择仍不能确认。

Game / Room 的存读和 choose 入口校验完整固定程序、source、actor、guard、cursor、steps/context、无target/cost、choice seat/player/kind/min/max/no-decline，以及整个真实候选 DTO 和当前牌库。重复、外来、消失和超额实例拒绝，失败命令保持状态。定义验证递归检查 modes / player loops / conditional branches，拒绝直接、嵌套和部分移植；XQ48 的正常共享声望能力按原声望验证，不能被检索定义的固定 key 检查误拦截。

每次接受的完成效果只调用一次原 shuffle；拒绝可选触发不调用。重复 commandId 返回原 receipt，冲突返回409，不重复取牌、进场或洗牌。零/一张剩余牌库的 Fisher-Yates 不消耗 RNG，不能用“RNG没变”误判未调用洗牌。

## 本机实际验证

| 范围 | 已执行结果 | 实际边界 |
|---|---|---|
| 新有限 Native 机制 | 19项全通过 | 含四actor循环；显式 Native 布局，不能称自然浏览器对局 |
| Native 全库 / 集成 | 606 lib + 40 integration，合计646个独立用例已验证 | 先全库604通过/1旧计数失败，再修正计数1通过、新鲜正常对局1通过；6个集成文件40通过。是合并覆盖，**不是一条646全通过的新全量命令** |
| 新鲜合法4席工厂 | 1项通过，497条正常 Room 命令，第4回合实际完成 BQ1与XQ3 | 从 newGameWithDeck + 3次合法join开始；没有写手牌、牌库、牌桌或待选布局 |
| Native / WASM 显式 parity | **776条**，其中743接受、33拒绝；完整transition、opaque state、4席view一致 | 真正 Native 导出的接受和失败命令，不只比较摘要或终态 |
| Native / WASM 新鲜工厂 parity | 工厂+join字节一致，497条每步transition / state / 4view hash相同，终态字节一致 | 正常命令轨迹，仍属于离线确定性验证 |
| 非法存读 | 192份，Native拒绝；WASM view和applyRoom均拒绝 | 2程序×4actor×24种程序/choice/source/牌库变体 |
| 当前 WASM release | 构建成功 | 版本50唯一当前 WASM；测试cfg更改不改产品WASM |
| 合并后 TypeScript | 两个配置都通过 | `npm run typecheck` 本机退出0 |
| 合并后 Web | **69文件、614项全通过** | 一次实际全量 Vitest，含13项新选择/原图/阅读器/真实WASM UI回归；不引用退役前647作为最终数 |
| 合并后 Worker | **39项全通过** | 一次本机全量 Node test，实际 Miniflare/workerd+D1。新检索5项是 RoomService+WASM+D1，不能称全HTTP或自然浏览器机制完成 |
| 合并后完整 Sites 构建 | 通过 | Web生产构建 + Wrangler dry-run；Worker47,314字节、WASM2,277,324字节。dry-run没有发布 |
| 首次合并后短视觉检查 | 实际大厅、两席入座/准备/开局/保留手牌、3地区桌面、手机渲染、两原图HTTP哈希；5条UI游戏命令 | **额外暂停步骤按钮名称错误超时，首次短脚本整体failed**；原失败结果保留 |
| 父端要求修正后的有限短烟测 | **完整脚本通过：1张新桌、8条真实UI命令、8项检查**；暂停→重载→恢复→再次暂停/重载，终态版本9 | 当前生产GameApp + 编译Worker50 + 本地D1，图片解码开启；手牌、地区、堆栈、选择等投影逐项保持，D1 receipt ID/席位/版本8条完全匹配。手机仅验证390×844实际渲染；**不证明新卡自然机制或完整手机可用性** |

新 Native 机制覆盖：接受/拒绝和0/1/3；BQ无合法项显式0、强制1；owner≠controller；两边同名同色独立实例；私有选择含队友；同时两个来源；来源回手/回底/控制改变；**XQ在接受可选声明前及接受后响应窗口的原地区替换，BQ在接受后、检索选择打开前的原地区替换**；真实付费伤害响应、来源死亡、封印 payload 返还 owner；暗藏公开雇员无 Enter 直到真实现身；XQ声望团队比较、仅1点势力、暗藏/横置/平手不获益；存读与SQLite reopen/重复/冲突；非法定义递归移植。已有 JZ31 / JZ24 / JC032、封印与 JZ50、宿主、伤害、团队等回归由全库覆盖。

上述地区替换均是显式 Native 布局测试，具体见 `rust-game/src/bounded_search_tests.rs` 的 `xq48_region_replaced_before_declaration_or_during_response_skips_search_and_shuffles`（第421行）与 `bq104_lost_original_region_preserves_search_reveal_shuffle_without_placing_in_replacement`（第446行）。原文“待选后的地区替换”覆盖声明不准确，已撤回：没有测试检索 pending 打开后替换地区。`rust-game/src/engine.rs:566` 的 `apply_inner` 在 pending 存在时只接受 choose，普通游戏命令不能制造这种地区替换；Room暂停/恢复属于另一入口。运行时完成选择仍检查地区实例，以及192份非法快照检查，均不能当作该时序已测试的证明。

## 浏览器机制缺项和失败记录

BQ/XQ 的自然浏览器机制尝试有限停止：最多3次尝试，实际只创建2桌，1条实际UI命令，**0项新机制完成**。第一次独立玩家切换脚本未先返回大厅而超时；第二次在 single-process 模式创建第二 browser context 时 TargetClosed；第三次按故障说明移除该模式后，在建桌前 Target crashed。没有把 Chromium 失败直接认定为产品错误，没有无限新桌或写入夹具。

独立的合并后短视觉检查只新建1张两人桌，5条UI命令；原先两个QA桌和本张短桌合计3张、6条实际D1 command receipts。桌面与大厅截图正常，手机截图只证明当前CSS实际渲染，未完成全部手机交互验证。额外暂停使用了 choice 内按钮名字“暂停并保存此桌”，此时全局入口实际为“暂停并保存”，超时后浏览器关闭。失败 result/log 保留，未增加新桌冒充通过。所有自有8120服务器、浏览器均停止；本地SQLite保留，归档不包含数据库、token、entry ACK或真实座位凭证。

父端随后明确要求修正定位并再完成一次有上限的短烟测。QA脚本只定位“保存与继续牌桌”区域内的真实全局暂停按钮，并允许独立输出目录以保留原失败证据；本次1桌、8条UI命令，小于10条上限，全部通过。`smoke-complete-20261007/result.json`、`commands.json`、5张截图与只读D1审计单独保存。该自有本地QA namespace至此共4张桌、14条命令记录，其中本次桌 `673ba1befc08b0113d8e7374` 恰好8条、版本2—9。浏览器已关闭；按验证过的本次PID停止6个自有Wrangler子进程，随后确认8120端口关闭。本次使用已成功启动的 post-Go Sites 构建服务器，无工厂种子header，Worker/WASM哈希与已提交候选相同。产品候选 `d55a740` 保持不变，本轮单独提交QA脚本、证据和文档修正；新卡自然浏览器机制完成数仍为0。

收尾第一次只停止了 Wrangler shell，最初 receipt 误报子进程已停止；后查发现本轮原8120子进程仍在，已按明确PID停止6个自有进程并确认端口关闭。另起 post-Go 服务器因端口占用未成功，因此短视觉检查实际使用原本地候选50 Worker及新构建的 post-Go `web/dist`。两份编译 Worker 逐字节相同，SHA256均为 `005c4b99e2fa91b15bb77aa67f0fd6a9a568849f6371412f8602dd6a7d2982e4`；未供应工厂种子header。相关事实和最初错误 receipt 保留，不能将失败的另起服务当作成功启动。

早期失败同样保留：Native测试最初字段名/借用/优先权假设和旧统计断言错误、编译缓存导致磁盘不足；真实产品缺陷为固定检索定义验证误拦 XQ48 生成的 renown，已修正并重测。UI首次错误导入/props/路径/owner字段及旧JZ50门禁和scan数量断言已修正。Worker首次缺50 pause fixture路径，以及错误假定单卡洗牌必变 RNG 已修正。完整失败日志和最终成功日志分开保留。

## Sites 插件和存储检查

任务 `01a10fcf-9a3a-755a-83bf-e8fe922ca0f4` 当前已有完整官方本地包 `/tmp/sites-official-restored-0.1.75`。`integrity.json` **167/167 SHA256一致**；`scripts/site-workflow.mjs` 可读，SHA256 `34c3408f93a12684c993adf0256d7fe138f2baa43ead46a63921c87e36935acd`，workflow/build/package helper语法检查通过。原生 Sites get_site 成功，确认当前用户owner、public、active、当前thread绑定及公开Site42。

Plugin Management 对名称 `Sites` 的全局目录依赖查询返回 `plugin_not_found`，原文为 `plugin_reference did not identify a public global listed plugin with a current release`。这不能证明插件未安装；已提供的云端 Sites skill、原生工具与完整官方包实际可用。没有可据此执行的安装操作，也没有猜URL、换权限/下载工具来绕过403。**原官方包下载403未重试，其原因和访问是否恢复仍未确认。**

Library配额没有可用证明；本地磁盘不足不能等同Library满。按用户既有清理授权，仅清理已停止且可重建的本机Native/bindgen编译缓存，保留源码、原图、rules、toolchain、Cargo缓存、冻结WASM、测试证据、provider和数据库。没有删除Library条目；此前Library403未通过其他route绕过。本轮最终另外清理1,658,776,005字节的自有已完成Native target，使tmpfs有空间完成开启图片的短视觉检查，具体receipt已保留。

现有发布项目为 `appgprj_6abf7bf54a7481918a50e1ef1509ca68`，公开Site42部署 `appgdep_6ac6b12bb48c8191bfae2fd680647d50`，provider源 `4595431ba223736fee200844c200ccda48cd078b`、engine49/pool44。provider tracked保持干净，没有新Sites版本或部署。

## 独立审查资料和复核

完整固定程序、卡数据、原始卡面、Native/Web/Worker测试以及真正v050夹具都在Git diff。`evidence/bounded-entry-search-2026-10-07/` 保存成功/失败日志、前后WIP ledger、历史保留receipt、Sites官方包检查、浏览器脚本/result/截图和收尾记录。`native-wasm-traces.zip` 保存所有显式 Native checkpoint/接受/拒绝、192非法状态、新鲜497步轨迹、当前50及冻结49 WASM、已查看的6张完整规则页，以及每个文件的SHA256 ledger。运行时凭证和数据库未打包。

复核离线 parity 不启动或修改服务：

```bash
unzip docs/evidence/bounded-entry-search-2026-10-07/native-wasm-traces.zip -d /tmp/entry-search50-review
WASM_PKG_DIR=/tmp/entry-search50-review/current-wasm50 ENTRY_SEARCH_EVIDENCE_DIR=/tmp/entry-search50-review node docs/evidence/bounded-entry-search-2026-10-07/parity.mjs
WASM_PKG_DIR=/tmp/entry-search50-review/current-wasm50 ENTRY_SEARCH_EVIDENCE_DIR=/tmp/entry-search50-review node docs/evidence/bounded-entry-search-2026-10-07/invalid.mjs
```

本机验证命令如下；Native汇总的具体分次边界以上表及原始日志为准，不应从这些可复跑命令推断曾执行单次全绿646：

```bash
CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_BUILD_JOBS=2 CARGO_NET_OFFLINE=true CARGO_HOME=/workspace/jz31-tools/cargo RUSTUP_HOME=/workspace/jz31-tools/rustup CARGO_TARGET_DIR=/tmp/entry-search50-review-target /workspace/jz31-tools/cargo/bin/cargo test --locked -p hegemony-server
cd web
npm run typecheck
npm test -- --maxWorkers=2
cd ../sites
npm run build
npm test -- --test-concurrency=1
```

父端已审固定树，并在最后BQ边界获得用户同意后指示官方发布。发布保留engine50/pool45及current-only边界，不永久删除49旧桌数据；不能把保留旧数据说成兼容恢复，也不能把已有8命令短烟测说成BQ/XQ自然机制已完成。官方workflow会在现有provider checkout构建、正常推送精确源码并打包；版本/source/deployment/archive hash以发布后的原生回读为准，不重发 unchanged Site42。
