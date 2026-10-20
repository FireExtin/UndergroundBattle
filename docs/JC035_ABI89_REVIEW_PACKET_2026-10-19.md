# JC035传媒魔咒 / ABI89批次审查材料

**WIP，P2最小修复已取得修前接受/写入与修后Game、Room、WASM拒绝、HTTP20次400且D1零写入的精确对照。新141输入、当前89、三份见证、标准/时钟Worker和完整生产清单均已重新绑定；Native JC03515/15、前端指定16/16、十三核路由14/14、保留核HTTP1/1、MSJC03入桌1/1通过。当前完整74项JC035 HTTP/D1及新浏览器证据仍待完成，父独审未完成；原完整Worker717/722及后续4/4、45/45保留不改，socket根因未知。不发布、不合main。**

本批只新增蓝色JC035整张卡，有限池153张普通定义、10张秘社。新核为`rust-v0.2.89-jc035-preparation-candidate` / `limited-v2.81-jc035-preparation-candidate`；保留实际已发布ABI77–88十二个旧核，候选共十三核。没有新增中立或双色卡。共同先手特权改为按实际图标贡献者判断，因而也覆盖既有JZ46地区附属，此共享行为变化在下文单独列出。

## P2恢复完整性修复（2026-10-20，WIP）

修前精确反例、Game/Room实际loader探针、WASM/HTTP接受及D1写入证据见[本次P2材料](evidence/jc035-p2-2026-10-20/README.md)。最小修复保留原容器→观察绑定及拒重复检查，反向要求进行中的存活原观察者，其None/Accepted记录必须仍有原声明、待选或响应帧。真正Declined/Resolved、出局及终局历史可无活动容器；不泵状态、不重建触发，不要求来源仍在场或当前操控者不变。

141项编译输入仅`jc035.rs`与`jc035_tests.rs`变化，最小源码检查点为`d2c245606ec377136615c67edfc8434fe7ff8752`。[v2源码清单](evidence/jc035-p2-2026-10-20/source-141.fixed-02.json)SHA256为`4c153aae7ad91b280f38253a2badeec903d6f58860d434bd4a7e29fd5f3a6f79`。Native首轮13通过、2失败均为新增离场夹具触发旧完整待选选项校验；原失败/v1输入保留，仅修正为接受后来源离场，产品守卫不变。Native02自然退出0、15/15通过，含50坏档、11合法恢复及1条真实续行；[独立精确双loader对照](evidence/jc035-p2-2026-10-20/after/exact-native-before-after-verification.json)确认八份修前输入原字节未变、同一探针换新链接库后两坏档均拒绝，两个原始合法状态仍接受。

| P2修后候选/已自然结束范围 | 本次实际身份或结果 |
| --- | --- |
| 当前89 WASM | 3,471,247B；`df890ed0b596ca245a39bc8466270edc2e8f40f92ef8dd34359534342ccf24de` |
| 标准生产Worker | `695c17c25667d165df429351ab65414dfd7d749e4b5c9a7837a7cfc851e4fb45` |
| 本地时钟Worker（不得发布） | `eeb4af7441cb2c4546c934e01b748694c5e37f19a9657d764de2437a4e458221` |
| [完整新生产清单](evidence/jc035-p2-2026-10-20/after/production-p2-fixed.manifest.json) | 187文件/142,257,627B；`28b461d38f3d0140ef7413c62509bbd58be46f88dea5f4141bcab054f6bc4da8`；TS/Vite/Wrangler仅dry-run |
| 三份新Native/WASM见证 | 383轨迹命令、81坏档、11合法恢复及1续行全部实际比较；[绑定身份](evidence/jc035-p2-2026-10-20/after/current89-fixtures-installed.json) |
| 正常机制/前端适用性 | [三份普通轨迹与修前完全相同](evidence/jc035-p2-2026-10-20/after/normal-trajectories-vs-before.json)；全部客户端文件逐字节未变，JC035前端指定文件16/16通过 |
| 路由/保留核/入桌 | 路由14/14，含238条原88命令；同一新十三核Worker保留核HTTP1/1（13核/13房/13命令及原回执）；MSJC03入桌完整文件1/1 |
| 精确坏档WASM/HTTP/D1 | [丢pending](evidence/jc035-p2-2026-10-20/after/missing-pending.http-wasm-rejection.json)及[丢Accepted帧](evidence/jc035-p2-2026-10-20/after/accepted-missing-frame.http-wasm-rejection.json)各10次HTTP400，D1重开前后房间不变、commands/journal各0；Game/Room/WASM错误一致 |

完整回执及原始压缩日志保留于[本次runs](evidence/jc035-p2-2026-10-20/runs/)。原输出5目录556文件已完整改名保留，旧时钟Worker硬链接字节未变；新187项中184路径相同，另为新当前89模块、标准Worker引用和README生成时间，所有客户端字节相同。新完整74项JC035 HTTP/D1（13正常、50坏档、11合法）及四客户端浏览器/新PNG正式传递仍待完成；不重跑旧全量722或前端1431。

ABI77–88仍使用迁入的原件，未重编译；新环境逐项哈希核验不替代父在任何未来发布前对旧环境实际载体的最终哈希检查。旧独审者未见四PNG像素及官方Library401仍如实保留，当前不发布、不合main。

## P2修复前固定源码与产物（历史记录）

| 项目 | 固定身份 |
| --- | --- |
| 审查分支 | `codex/jc035-blue-preparation-review-20261019` |
| 批次基线 | `cf0f1f461a7180ccc834a3289f3211583a417e11` |
| Native与WASM编译源码 | `fba026bd887458b65c46fc855b9d28484c8a1a57` |
| 原生产候选构建源码 | `31949d778d83ea2f88957b10f812df20bc7a9499` |
| 新环境恢复与验证代码 | `5056e4e65f0e1edacf617ce12175d67f86eee0fc`；后续小时保存仅新增证据/文档 |
| Native141输入清单SHA256 | `0e98a5cb1c714c94ed2bc9c38ace3b48089fcd4a6a466f6fac158de83267fe98` |
| 新89 WASM | 3,470,546B，`02e36dc9cd760213978dad67ebbcd95392793a64a5e9a5e108ec59eb4fcac640` |
| 标准生产Worker | `ba792492721a5bbf96a5af5bada80c9d3d3d72411b50d9aaaa92f7cd5184f15d` |
| 构建后、完整Worker前清单 | 187文件 /142,256,926B；`8695312fbc911f79e247f03816fc22932eb87911ef77fad1ac7418cb788f9148` |

构建后清单见[production89-before-worker.manifest.json](evidence/jc035-2026-10-19/production89-before-worker.manifest.json)。本节及以下恢复记录截至P2修复前的`ecf2236`：当时编译源码之后的提交仅为测试投影/入桌夹具修正、测试见证和证据文档，产品代码与该编译源码一致。上述身份用于历史对照；P2修后不能沿用为新候选身份。

## P2修复前新环境恢复与验证（历史适用补验）

用户授权经官方Library流程迁移精确原件。迁移包13,882,939B，SHA256 `02a4d8fb4ababa881589b80fff7252ac0406063f99defcd0d75574b468b75c43`；27个文件共13,841,070B逐项验证后按相对清单恢复。包路径、逐文件哈希、12个gzip解压WASM及13个实际ABI目录全部匹配，没有重编译或替代历史内核。公开回执见[恢复核验](evidence/jc035-recovery-2026-10-19/runtime-restoration-verification.json)及[实际ABI身份](evidence/jc035-recovery-2026-10-19/runtime-abi-identities.json)；私有传输标识、签名URL、数据库、缓存与二进制载体未纳入Git。

新生产构建通过两套TS检查、Vite和Wrangler dry-run；构建后普通磁盘可用27.317GiB。187项中186个执行/静态文件与原候选逐字节相同，标准Worker及十三个WASM均保持原哈希。唯一不同路径是Wrangler以当前UTC生成的`server/README.md`；旧README原件未迁入，没有重构或改写它来匹配旧哈希。[初次整清单对照](evidence/jc035-recovery-2026-10-19/production-rebuild-verification.json)与[生成器差异分析](evidence/jc035-recovery-2026-10-19/production-rebuild-delta-analysis.json)分别保留。本次187项/142,256,926B的[新环境候选清单](evidence/jc035-recovery-2026-10-19/production-recovered.manifest.json)SHA256为`0ba0f278b10e65f71908acdb27b2c5366c3387aaa3616bfe676931577ceaaa77`。本地测试Worker为`6dc7cf9e217f6be438b8b422e66a91ff725f837e45beacddab8a500a0108e567`，与原四客户端浏览器验证所用字节一致。

| 本次已自然结束阶段 | 实际结果 |
| --- | --- |
| MSJC03入桌完整文件 | 1/1通过；四席命名完整牌组、六张起手、D1重开及原回执；[退出/输入回执](evidence/jc035-recovery-2026-10-19/runs/jc035-entry-msjc03-recovered.exit.json) |
| 原样JC066 ABI86 HTTP target-reveal | 1/1通过；19条接受命令、160次投影；[退出/输入回执](evidence/jc035-recovery-2026-10-19/runs/jc066-target-reveal-recovered.exit.json) |
| 新完整Worker自然终态 | 69文件、722用例：717通过、5失败，0取消/跳过，退出1；[完整回执](evidence/jc035-recovery-2026-10-19/runs/worker-full-recovered.exit.json)与[完整原日志](evidence/jc035-recovery-2026-10-19/runs/worker-full-recovered.log.gz) |
| pass-history专用本地Worker补建 | 既有脚本dry-run退出0；执行文件与旧专用包装逐字节相同，WASM为原ABI77；[构建回执](evidence/jc035-recovery-2026-10-19/runs/pass-history-worker-recovered.exit.json)及[产物身份](evidence/jc035-recovery-2026-10-19/pass-history-wrapper-before-tests.json) |
| pass-history完整文件补验 | 4/4通过，退出0，0取消/跳过；含37原Native77轨迹、322接受命令和224条公开让过记录；[回执](evidence/jc035-recovery-2026-10-19/runs/pass-history-complete-recovered.exit.json)及[前后核验](evidence/jc035-recovery-2026-10-19/pass-history-complete-verification.json) |
| JC041完整文件补验 | 45/45通过，退出0，0取消/跳过；10正常场景、35坏档，含原discard-6；[回执](evidence/jc035-recovery-2026-10-19/runs/jc041-complete-recovered.exit.json)、[完整原日志](evidence/jc035-recovery-2026-10-19/runs/jc041-complete-recovered.log.gz)及[最终核验](evidence/jc035-recovery-2026-10-19/worker-followup-closure.json) |

入桌失败的已确认原因仍是原黄色MSJC01夹具只能凑39张；本次验证的是已有单行MSJC03修正，没有修改断言、业务或预算。JC066的原socket异常本次未复现，关闭根因仍未证实，没有添加重试或据通过结果归因。

新完整Worker于2026-10-19 18:37:29 UTC以原69文件、`--test-concurrency=1`启动，同一会话持续运行，未重启。19:36:18 UTC的[小时检查点](evidence/jc035-recovery-2026-10-19/hour-checkpoint.json)仅观察到60条通过、0条失败；JC035的13条正常HTTP轨迹通过，47种坏档矩阵仍在执行，没有自然退出回执。这些是日志前缀计数，不是完整通过汇总。141编译输入、Worker测试源码和27个载体的前后校验由原运行记录器保留，最终校验须等该轮自然结束。

20:17:20 UTC的[首条新失败检查点](evidence/jc035-recovery-2026-10-19/first-new-failure-checkpoint.json)保留同一运行的有界原日志前缀：160条通过、1条失败标记，未自然退出。名称为`JC041 corrupted discard-6 restores HTTP400 with no D1 writes`，耗时26514.679158ms；随后discard-7等用例继续通过。当前原日志没有错误类型、原因链、栈或断言差异，不能把这项标记归为旧socket异常、业务、核版本或资源问题。只读资源样本在20:15:56 UTC显示OOM及OOM kill均0、可用空间27.174GiB，不用于证明失败根因。继续原运行至自然终态后再定位；没有重试、修改用例/业务或预算。该保存点不覆盖原失败，也不把中途条目当最终总数。

21:23:00 UTC的[计数与资源检查点](evidence/jc035-recovery-2026-10-19/progress-counting-and-resource-checkpoint.json)仅观察377个通过用例、1条失败标记，仍无自然退出。进度标记表示已完成的顶层测试用例，不是文件、断言、HTTP请求或Native命令数。旧轮首722条用例标记为720通过/2失败，与其最终Node汇总相同；本轮同一69文件、已完成378个用例的名称及顺序与旧轮前缀逐项一致，仅归一化已有MSJC01至MSJC03改名，没有增减用例。因此预期全量仍为722，实际总数、取消/跳过及退出码须等末尾Node汇总。

同一检查点记录普通磁盘余27.015GiB、OOM计数0、CPU配额`400000 100000`。本轮期间约4%的cgroup调度周期有节流记录；这些是整个环境的计数，不是独立Worker CPU，不能用于判定全部变慢原因或JC041失败根因。以已完成前缀耗时比外推整轮约5小时，仅是随用例组合/负载变化的粗估，不是超时或终止期限。没有改资源、并发或任何预算；继续现有运行，不增加重试。

21:42:20 UTC的[小时检查点](evidence/jc035-recovery-2026-10-19/jc066-normal-hour-checkpoint.json)保留同一运行的新前缀：434个通过用例、1条JC041 discard-6失败标记，最新日志21:42:16 UTC，详细异常仍未输出，尚无自然退出。JC066的[全部19条正常HTTP/D1场景](evidence/jc035-recovery-2026-10-19/jc066-normal-scenarios-in-full-run.json)已通过，其中原失败`target-reveal`在本轮也通过，耗时92484.052136ms；87种非法状态矩阵仍在执行。因此原socket异常在定向及本轮该场景均未复现，但关闭根因仍未证实，也不能把本轮或JC066整个文件称为通过。没有为小时保存重启、重复测试或修改源码、并发、资源与预算。

22:05:34 UTC的[实质变化检查点](evidence/jc035-recovery-2026-10-19/pass-history-failures-checkpoint.json)保留547个通过用例、5条失败标记：原JC041 discard-6，加上`pass-history-integration.test.mjs`的全部4项（37条Native77轨迹、超时/取消/暂停、真实HTTP双端完整回合、CAS与并发回执），仍未自然退出或输出具体异常。[22:06只读检查](evidence/jc035-recovery-2026-10-19/pass-history-build-prerequisite-readonly.json)确认，这4项硬编码使用的`/workspace/undergroundbattle-build/pass-history-worker/server`不存在，不读取当前Worker覆盖变量；仓库已有`sites/scripts/build-pass-history-test.mjs`，会从精确原ABI77载体生成独立本地时钟包装并仅执行Wrangler dry-run。此处记录的是确定的构建前置缺口，具体异常与关联仍待完整报告核对；不改写为生产断言失败或已证明同因。保留现场，待本轮结束后再补齐并完整复验受影响文件；当前不构建、不重试，也不改活动输入。

23:01:59 UTC，同一原会话82121自然退出1，Node最终汇总为722项、717通过、5失败、0取消/跳过，耗时15870.093秒。[完整失败栈](evidence/jc035-recovery-2026-10-19/worker-full-recovered.failure-details.txt)与[终态完整性核验](evidence/jc035-recovery-2026-10-19/full-worker-natural-exit-integrity.json)独立保存。141编译输入、74个顶层Worker测试源码、27个原始载体前后及终态均相同；187项新候选产物、十三核测试WASM、标准/时钟Worker哈希及旧失败日志也保持原样。全部722用例的名称和顺序与旧轮相同，仅归一化已有JC035秘社夹具改名。末尾重复打印失败名称不另计失败，最终5项以Node汇总为准。

4项pass-history均在`harness`第21行`readdirSync`抛出`ENOENT scandir /workspace/undergroundbattle-build/pass-history-worker/server`，未到产品断言；这是本次恢复启动预检漏查独立本地包装的准备遗漏。23:05已通过仓库既有脚本补建，执行文件SHA256为`de4caa99beae0ddc3b150ea38fcea2b4641e93a8008f401785e45ddfe49a42ce`，与旧专用包装原记录一致；唯一WASM为精确ABI77原件`b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c`，没有重编译历史核。完整4项原文件已自然退出0，4/4通过、0取消/跳过，耗时198.223秒；37条原Native77轨迹、322接受命令、224公开让过记录、真实双端整回合48次让过及CAS/并发回执断言均通过。全部输入、独立包装8个文件及Miniflare依赖前后不变。完整套件的准备门槛需同时核对十三核当前时钟Worker和这一独立ABI77 pass-history Worker。

JC041 discard-6的实际错误为`TypeError: fetch failed`，原因链`SocketError: other side closed`，耗时26514.679158ms。[本地锁定依赖源码映射](evidence/jc035-recovery-2026-10-19/jc041-transport-stack-mapping.json)把`[worker eval]:32:22`定位到Miniflare `core/proxy/fetch-sync.ts`的子线程fetch；该机制转发异常时未保留具体夹具await位置。可观察异常位于同步绑定代理传输，没有业务值断言差异；socket关闭根因仍未证明，也未证明与旧JC066异常同因，不据CPU/OOM样本归因。23:09:38 UTC，前项完整文件结束后，按[启动核验](evidence/jc035-recovery-2026-10-19/jc041-complete-followup-gate.json)以相同十三核、依赖、断言、并发和预算串行启动完整JC041文件，23:32:36 UTC自然退出0：45/45通过，0取消/跳过，1378.383秒。10条正常场景完成146条接受命令、9条拒绝命令及1284次席位投影；35坏档共350个HTTP400，D1重开前后无写入。原discard-6在本次通过，11081.010652ms。该异常未在整文件补验中复现，不宣称已证明或修复底层socket关闭原因。

[最终补验核验](evidence/jc035-recovery-2026-10-19/worker-followup-closure.json)逐项对应本轮5个失败名称，均已在各自完整源文件的补验中通过；没有把原完整轮改写成一轮722全绿。完整轮、补建和两项完整文件补验的141编译输入、74测试源码、27载体快照完全相同；终态再核187项候选产物、13个当前测试WASM、标准/时钟Worker、独立包装8文件、锁定Miniflare依赖、JC035原图及两轮失败证据均未变。新完整轮原日志SHA256仍为`b6728a4c16b5fc195078241618c5f1ff5925eb871917e4a675cad7b9da5cdd6f`，JC041补验日志为`23a308b33cd88a72382799b25db6d6f0882493bb9377219bc6535c6a9dc0d893`。

新环境仓库为`/workspace/UndergroundBattle`，固定工具链入口`/workspace/jc035-toolchain/env.sh`，运行载体与两个本地测试Worker位于`/workspace/undergroundbattle-build`。首次生产构建后实际余量27.317234GiB，最终26.771122GiB，满足至少6GiB的保守余量目标；原1GiB启动守卫不变，没有清理旧环境、原数据库、缓存或证据。验证源码固定`5056e4e65f0e1edacf617ce12175d67f86eee0fc`，其后Git变化仅为本次相关WIP文档和证据，固定独审提交由交付回执给出。

完整Native release1358、前端1431、Native/WASM383命令/78坏档及四客户端15命令的旧通过范围未重复运行：141输入、实际WASM、Worker和前端资产字节未变化，原生产构建源码至验证代码的唯一业务/测试路径差异是已列出的入桌夹具一行。原失败回执、外置PNG和封存候选仍按旧环境边界保留；本次不是线上实局，没有发布或合main。适用补验与终态身份复核已完成，剩余父串行独审；旧/新完整失败轮和各项补验保持独立。

## 整张卡与运行行为

原图为`resource/ymsj-fun.github.io/cards/JC035 传媒魔咒.jpg`，341,309B，SHA256 `e7f2cff71f3a511d5065ece513db9013d02e5685a7d7ee899de4afbc109fe319`。完整核图字段：3费、蓝忠诚1、非独有、附属/结界/媒体、心灵领域1、临时调查1；结附目标地区，准备步骤满足敌方手牌条件时可在本地区放置1本方势力标志。规格档案的`selectionEnabled`、`gameAcceptance`、`strategyUiPlaytest`仍保留原档案值，没有用组件或本地测试改写档案验收。

- 普通结附复用标准时点、个人忠诚/资产付费、实际地区UID绑定与目标失效费用不退；没有额外手动发动准备能力的入口。
- 重置后、共同抓牌前捕获本次准备观察；可发动一项或跳过，同一操控者可选择自己的多来源入栈顺序。已接受帧独立于来源，结算绑定原地区实体。
- 来源被真实JC005快速回应销毁仍可结算；原地区被替换或成为空位时不向替代地区放置标志。赢区回底使用共同批次流程，两张JC035回拥有者牌库且不进墓地。
- 附属临时调查仅在其操控者团队持先手时计入对应地区；横置抑制实际对抗贡献。共同先手特权入口改用实际贡献者列表，包括已存在的JZ46和本次JC035地区附属。霸权说明书印刷P9以对抗图标说明特权，双方0图标不发生对抗。
- Game与Room恢复均检查准备事件/来源/帧、完整待选、物理形态与UID、原手牌/事务实体、接受/放弃/已结算等边界；固定准备效果不允许绕开响应堆栈或恢复已结算帧。
- 前端显示触发编号、原地区和实例标记；选择下一项后才启用确认，放弃使用明确的跳过按钮。沿用草稿恢复、换选和忙碌状态，避免下一待选到达时确认连点误放弃。

## 代码与证据入口

| 入口 | 审核重点 |
| --- | --- |
| [jc035.rs](../rust-game/src/jc035.rs) | 整张定义、准备观察、可选排序、独立帧、地区UID与恢复守卫 |
| [engine.rs](../rust-game/src/engine.rs) / [resolution.rs](../rust-game/src/resolution.rs) | 重置后准备时点、实际图标贡献、共同先手特权、响应与结算接入 |
| [jc035_tests.rs](../rust-game/src/jc035_tests.rs) | 原图字段、阈值与敌我/出局矩阵、普通付费、图标奖励、原地区、47坏档、13条原生轨迹 |
| [ChoicePanel.tsx](../web/src/game/ChoicePanel.tsx) / [JC035Native89.test.jsx](../web/src/game/JC035Native89.test.jsx) | 来源分辨、换选、下一待选清空选中、确认与跳过边界 |
| [jc035-integration.test.mjs](../sites/test/jc035-integration.test.mjs) | 原生轨迹逐命令经HTTP/D1、重开与原回执；坏档零写入 |
| [jc035-entry.test.mjs](../sites/test/jc035-entry.test.mjs) | 普通API创建/加入/准备/开始、已命名完整牌组和六张起手；MSJC03修正后的完整文件已1/1通过，新完整轮亦通过该项 |
| [jc035-retained-http.test.mjs](../sites/test/jc035-retained-http.test.mjs) / [jc035-kernel-routing.test.mjs](../sites/test/jc035-kernel-routing.test.mjs) | 原77–88载体、同一十三核Worker路由与当前89严格拒绝旧身份 |
| [xq23-current-owner-integration.test.mjs](../sites/test/xq23-current-owner-integration.test.mjs) | 新89真实借控出局/返还轨迹，独立保留原88/P2见证 |

## 需要父串行独审核对的规则应用

**JC035手牌条件取样采用本批明确的项目规则。**准备开始时只检查存活敌方各席位是否有手牌不超过3张；队友和当时已出局席不计，后续手牌变化不追溯创建或撤销资格。XQ37除事件时资格检查外，结算仍复检地区势力（`resolution.rs`1107–1116、`xq37.rs`35–46），不能作为“完全相同事件条件快照契约”的依据。上述JC035准备取样应用不冒称官方专属FAQ，也不据此改动本批机制范围。

共同先手特权的入口从仅扫描地区角色改为既有`contributors`列表，本批JC035以真实付费、平局和奖励用例验证；同一路径也允许既有JZ46作为地区附属贡献者。此处需连同印刷P9的图标条件一起审核，不能把批次说成只改目录。

四席组队为0/1对2/3，双席为0对1。同团队不同操控者仍依既有项目席位顺序处理，每个操控者选择自己来源的顺序；没有将此席位顺序宣称为官方多人规则裁定。

## 已完成验证

| 验证 | 同轮结果与范围 |
| --- | --- |
| 完整Native workspace，release profile | 1358通过、0失败、0忽略；包含1291 lib与67后续集成用例；141输入前后相同 |
| 原图、历史投影、冻结核与原见证检查 | 99通过；原卡数据、旧运行载体及旧见证独立核对 |
| 当前Native/WASM比较 | 23场景、383命令（10拒绝）、1612实际席位投影、78种坏档；完整状态/日志/视图与拒绝结果匹配 |
| 完整前端第三轮 | 143文件、1431用例通过；单工作进程、显式无文件并行、原5000ms预算 |
| 生产候选 | 两套TS检查、Vite、Wrangler dry-run通过；13个精确WASM |
| 本地四客户端浏览器 | 15个真实UI命令、3次D1重开、15份原回执重试、页面异常0；1366×768 |
| 完整Worker原轮 | **69文件自然完成，722项中720通过、2失败；退出1，0取消/跳过；141输入前后相同** |
| 新环境定向复验 | MSJC03入桌完整文件1/1、原样JC066 target-reveal单例1/1通过；旧空间预检退出78的原回执仍保留 |
| 新环境完整Worker | **69文件自然完成，722项中717通过、5失败；退出1，0取消/跳过；输入与187项产物不变**；4项为独立本地包装漏建，1项为JC041代理fetch/SocketError，原失败保留 |
| 新环境完整文件补验 | pass-history 4/4、JC041 45/45均通过，各自自然退出0，0取消/跳过。原完整失败轮保持717/722，补验单独计数，待父独审 |

JC035专属Native为13场景/145命令/4拒绝/620投影/47坏档；当前89的XQ23九场景为72命令/5拒绝/324投影/31坏档，借控出局为166命令/1拒绝/668投影。三份新的压缩见证都在实际WASM比较后才写入，141源码绑定直接核对当前文件，没有重签原88状态。

JC035的真实Room轨迹包括四席与双席、放弃、阈值4、JC005来源销毁、JC049观察3张后增至4张、观察4张后声明减至3张再结算5张不追溯触发、多来源顺序、原地区赢取和普通付费结附。所有轨迹明确采用合成初始布局，之后走普通Room命令。借控JC035的单元用例是显式初始共享操控布局，不冒称实际JC129能控制附属。

原地区赢取轨迹中，下层准备帧在原地区仍有效时结算，然后完成真实赢区窗口/回底/替换；没有声称该下层帧在替换之后失效。替代地区不继承旧UID效果由明确的共享替换原语单元布局覆盖。

原实现者的本地浏览器记录从当时Native精确待选状态开始，覆盖普通换选、刷新草稿、双击确认、下一来源清空旧选中、决定者权限、原图、HTTP/D1及回执；原实现者记录已查看待选和结算截图。原PNG在旧环境`/workspace/jc035-evidence-20261019/browser-jc035-first/`，Git仅保留[报告](evidence/jc035-2026-10-19/browser-local-report.json)和[图片身份](evidence/jc035-2026-10-19/browser-local-images.json)。**当前独审者尚未看到旧四PNG像素**；父报告官方Library上传返回401，未绕过认证，不能把报告/哈希核对写成像素独审通过。P2修后将另取当前候选截图并分别记录身份与正式传递结果。这些均不是线上实局或全卡组合覆盖。

## 失败边界与旧版本

以下记录原失败轮次及当时的待验证/空间阻塞状态；新环境恢复后的进展单独见上节，不覆盖或改写原失败。

完整失败轮次、修正与命令均原样记录在[WIP交接](JC035_BLUE_PREPARATION_WIP_2026-10-19.md)及[runs](evidence/jc035-2026-10-19/runs/)。默认profile全量两轮分别1279通过/10失败、1290通过/1失败，后者暴露同一目录测试后续的非地区143和末牌JC035固定值；没有把修正后的局部结果合并称为全绿。最终完整release轮独立为1358通过。

前端第一轮1430通过/1历史扫描投影失败，修正只补JC035历史排除项，保留154历史数量和原图固定哈希。第二轮1429通过/2失败，详细错误均为原5000ms超时，栈只指测试声明；未有值差异。原文件/断言/夹具/setup和业务未改，隔离29/29及完整第三轮1431/1431在相同预算通过。未据此编造原失败根因，原失败回执保留。

完整Worker于12:48:40 UTC自然退出1，耗时7071.559秒；同轮722项中720通过、2失败，0取消/跳过，141输入前后不变。原完整日志SHA256为`2b93a35abf6d5ea3dc2b5e53992a9a5f1925bd0626e81e7761e7004c977bb217`。原件、退出与输入回执、最终失败详单已完整保留，见[汇总](evidence/jc035-2026-10-19/worker-full-first.summary.json)与[原失败详单](evidence/jc035-2026-10-19/worker-full-first.failure-details.txt)。

- `JC035 in MSJC01 enters with a named complete deck, exposes six-card start and preserves it after D1 reopen`：1561.527847ms；`AssertionError [ERR_ASSERTION]`，实际11、预期0；栈为`jc035-entry.test.mjs:31:12 draft`、`:36:49`。黄色MSJC01同色副本只能凑39张，缺11，在有效牌组创建请求发出前失败。现仅把该测试的一处MSJC01改为蓝色MSJC03；原生目录只读复算可凑含3张JC035的50张。原断言、业务和预算未改，修正后测试尚未启动，不能宣称通过。
- `JC066 ABI86 HTTP target-reveal`：37202.791364ms；`TypeError: fetch failed`，原因链`SocketError: other side closed`；栈落在Undici fetch `index.js:237:16`、`:1081:19`、Undici `index.js:157:7`、`[worker eval]:32:22`及`client-h1.js:959:22`。未出现业务值断言差异，也未证明socket关闭根因；不归因为目录、核版本、业务、资源或测试预算。该测试与session断言自批次基线未改，保留原样待验证，不添加重试掩盖异常。

JC035本身13条HTTP/D1轨迹及47坏档、同一实际十三核HTTP/D1、原88见证重放均在此完整轮通过。当前89的166命令借控拥有者自然出局、清除、计分和回执也通过；原88/P2独立166命令轨迹及原12核检查分别通过。这些局部通过没有被合称为完整Worker全绿。

12:50只运行未改的`node tools/build-storage.mjs`启动预检，无测试命令；自然退出78，target与TMPDIR同一普通磁盘可用1,031,143,424B，低于1,073,741,824B。预检明确“没有开始构建，没有存储回退或清理”；见[原预检输出](evidence/jc035-2026-10-19/runs/validation-preflight-after-worker.log)。执行连接正常，原Worker已自然结束；当前阻塞是新验证的磁盘门槛，不是执行连接中断。按父协调保留数据，不删除缓存、数据库或证据，不降低守卫或改用tmpfs。

后续先恢复至少1GiB普通磁盘可用空间，再串行验证修正后的完整`jc035-entry.test.mjs`和原样`JC066 ABI86 HTTP target-reveal`；根据实际结果处理必要修正。完成适用验证后才交父串行独审，不能现在发布。

原v70的ABI88仍为3,353,559B、SHA256 `5834f38dcdb9952a2430e7eb08c44ee9c4a1c98bc4a8ab138c4aee3c343ccde3`，保留原139输入绑定、原九场景72命令/31坏档及P2 166命令见证。原31,197B付费状态SHA256仍为`76a47a29fb9d3b1d772ffdb6087e9180cac93f91fdd0c3af0d989fdc36b48c65`；当前89原生载入会拒绝其旧身份，旧兼容核执行原字节。新旧JS包装哈希相同不代表WASM身份相同。

完整Worker后重新核对候选全部187路径、字节和哈希，原构建清单保持`8695312fbc911f79e247f03816fc22932eb87911ef77fad1ac7418cb788f9148`。标准Worker、当前89 WASM和clock目录13个WASM均未变；[运行后核对记录](evidence/jc035-2026-10-19/production89-after-worker.verification.json)明确标为资格未通过，没有新构建或部署。

11:09静态封存去重只对三个生成client目录的510个路径进行同字节、同权限/uid/gid硬链接归并，保留全部路径并再核候选187项清单，净回收300,277,760字节；活动Worker模块、WASM、源码、数据库与缓存均未改。明细见[封存静态去重记录](evidence/jc035-2026-10-19/sealed-candidate-static-dedup-summary.json)。任何重建前必须先将整个候选输出目录移出封存，不能原位覆盖共享inode。

普通磁盘target/TMPDIR、jobs1、incremental0及启动前1GiB磁盘/内存守卫未改。没有新建worktree、子代理、强推、合main、删除未知数据库/缓存或建立可编辑源码硬链接。公开v70及父保存的纯灰线上实局证据保留；本批尚未上线。完整Worker与父串行独审通过后，才可沿既有授权走官方Sites发布流程。
