# 隐秘世界 · 霸权数字版

《隐秘世界》桌游的有限卡池数字化实现：React 玩家界面、Rust 权威规则核，以及 Native/SQLite 与 Sites Worker/D1 两种服务宿主。当前规则包为《霸权》；不是全扩展、全卡牌或经典规则的完整实现。

## 当前状态（2026-10-22）

| 范围 | 已核实状态 |
| --- | --- |
| 已发布源码 | 第一批20卡修复快照 `7f4e6d63b2498ea2f47a3a6e5b1bf29dfd3dc762`，引擎 `rust-v0.2.91-batch20`；对应Site v73，待验收。含旧缺陷的v72未部署 |
| GitHub保存状态 | 固定版本审查后的整改与前期README更新已推入main，已确认main为 `d1c068ea8c4162212cd19fa023841513c4c67e33`（含来源准备文档）；本批20卡仍在进度分支，Git保存不等于Sites发布 |
| 已发布规则 / 卡池 | `hegemony-pdf-v1` / `limited-v2.82-batch20` |
| 最近确认成功的Site | **v73 / ABI91**；2026-10-21 13:34 UTC官方部署回执为 `succeeded`；v74随后获准重试，仍在pending，不能报告新的线上身份 |
| 审查及整改 | 固定产品源码 `100161b628e56b9153041d952ac0f5d83428ca88` 的审查后，完成8项明确问题的本地修复及相关清理；见[整改与验证记录](docs/LOCAL_CORRECTNESS_AUDIT_2026-10-21.md) |
| 第一批后续 | 空秘社区P2修复通过最小源码独立补验；v74在持续授权后已创建部署，当前pending。v73已有一局8回合自然完局及终局刷新，不是20卡全机制验收。固定7f的Sites最终完整运行736通过、0失败 |
| 第一批20张 | [发布与验证记录](docs/factions/batch20-2026-10-21.md)：20张实现和准入完成；Web1471、工具84、Rust workspace1395项及修复版Native/WASM完整440场景通过 |
| 第二批20张 | [组合候选](docs/factions/batch20-next-2026-10-21.md)：20张实现及准入完成；ABI93/pool83共193张。固定9aa4fda独立复审的已验范围无新P0/P1；两项构建/测试P2已在677a7b1独立补验关闭。旧多核v75退出发布队列，ABI93单核仍未上线 |
| 当前单核验证 | [清理记录](docs/LATEST_KERNEL_ONLY_2026-10-21.md)及[夹具迁移](docs/CURRENT_FIXTURES_ABI93_2026-10-21.md)保留各固定身份的历史结果；[9950X Ubuntu恢复](docs/UBUNTU_RECOVERY_2026-10-21.md)在精确ABI93字节上重新完成Web148文件1498项、产品内Sites宿主58文件637项、来源工具27文件及原5项目录失败补验，均通过。中断轮单独留档；本机未重跑Rust1482项或广域Native/WASM483场景，未完成独立复审/线上批次验收 |
| 第三批进度分支 | [20张本机候选](docs/factions/batch20-third-2026-10-21.md)及[ABI94必要检查](docs/factions/ABI94_INTEGRATION_PREPARATION_2026-10-21.md)：CQ07替补待裁定的CQ11，红4/黑5/蓝6/灰5；单ABI94、213条定义，本机新增20张整体准入。原生本批162、Web70、Sites12及构建/启动身份检查通过，均为作者必要验证。原v74在00:03:57UTC仍pending，未新部署；本批公开准入及验收仍0/20 |
| 第四批隔离研发 | [20张来源与进度](docs/factions/batch20-fourth-2026-10-22.md)：红黑蓝灰各5张，前213条逐字段保留。独立普通克隆/进度分支预留ABI95，已接入朽灭者、雾涌海怪、恶魔刺青、兽形变化、多情达莉娅、欢宴、仪式雕像、计划：天眼、计划：贝奥武夫、专家研讨会、阴魂术士、被胁迫的政要、“弹簧刀”钱斯及其共享机制；新增构筑准入、WASM、公开站与验收均为0/20。第三批b80f1220及其ABI94部署包冻结，不混入本批WIP |

[游戏入口](https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site)最近确认成功的官方部署为v73，包含第一批20卡及此前整改。该成功部署ID为 `appgdep_6ad8bf5fa23081918eca730d2b96e3bb`；Site源码提交为 `e1969c3ded2ffa44ce429be87c0005d928d5ef38`，与GitHub产品仓库属于不同源码仓库，不应混用。精确版本、产物及访问核对边界见[发布证据](docs/evidence/batch20-2026-10-21/publication-v73.json)；v74仍pending，v75只保存未请求部署，现已退出发布队列。

Go服务、旧调试器和 `/legacy-debugger` 已退役。此前README中的Go正式架构、WebSocket路线和计划目录不是当前实现；历史内容仍可从Git历史查看。

## 已发布v73的卡牌与玩法范围

下表统计v73冻结版本；第二批冻结验收基线bb77的ABI93候选为193张（角色121、事务34、附属28、地区10），尚未发布。第三批冻结b80f1220使用单ABI94：213条定义，本机候选新增20张整体准入。本第四批分支预留单ABI95，目前226条目录记录，新增十三张先行卡仍禁止构筑；只有原生先行实现，尚无本批WASM。CQ11/TK008不在运行目录。这些状态不代表公开站已更新或整批验收通过。另列秘社仍10张。运行目录由 [cards.json](rust-game/data/cards.json)、[catalog.rs](rust-game/src/catalog.rs) 与 [society.rs](rust-game/src/society.rs) 共同提供；以实际 `/api/catalog` 为准，当前身份牌桌使用 `/api/rooms/{roomId}/catalog`；单核候选拒绝旧身份房间并提示新开。

| 目录类型 | 数量 |
| --- | ---: |
| 角色 | 105 |
| 事务（接口类型 `spell`） | 33 |
| 附属 | 25 |
| 地区 | 10 |
| 上述规则卡合计 | **173** |
| 另列的秘社 | **10** |

173张目录卡包含地区，玩家牌种合计163张；此数量已在v73对应的本地构建和新房间中核对，线上目录HTTP核对尚受403限制。世界牌为DQJC107–DQJC116各一张。支持红、黑、蓝、灰等已准入牌的相应机制，不代表这些阵营全部卡牌已实现。原始研究目录、卡图存在或历史候选文档均不等于运行时准入。

- 双人模式 `duel`：3个地区，8分胜利；四人团队模式 `teams`：5个地区，团队10分胜利。
- 五套自组预组各50张：帷幕调查、公路猎手、城市守卫、墓地回声、牺牲与响应；它们不是官方预组。
- 支持命名自组牌组与秘社选择。最低50张；同名数量按印刷牌名及目录中的 `deckCopyLimit` 检查，常规为3张、独有为1张，存在明确例外。2048张是服务容量上限，不是桌游规则。秘社构筑要求仍由服务端校验。
- 提供邀请入席、准备/开局、选牌与排序、伤害分配、响应窗口、暂停/恢复、对局结束及重开。隐藏信息由服务端按席位投影；客户端不能读取完整对局状态。

## 实际架构与目录

| 部分 | 职责 | 入口 |
| --- | --- | --- |
| Rust规则核 | 合法动作、费用、结算、触发、选择续体、胜负、隐私投影、严格恢复 | [engine.rs](rust-game/src/engine.rs)、[rules.rs](rust-game/src/rules.rs)、[resolution.rs](rust-game/src/resolution.rs)及各规则模块 |
| 房间语义 | 带版本命令、可信服务端时钟、响应期限、暂停与恢复 | [room.rs](rust-game/src/room.rs) |
| Native宿主 | Axum HTTP、席位令牌、SQLite状态/日志/命令回执；另有SSE接口 | [rust-game](rust-game/README.md) |
| WASM边界 | 编译同一Rust规则核，供服务端JS宿主调用；不复制一套JS规则 | [rust-game-wasm](rust-game-wasm/README.md) |
| Sites宿主 | Worker HTTP、D1、创建/加入回执、命令CAS事务、当前版本校验、静态资源 | [sites](sites/README.md) |
| 玩家客户端 | React/TypeScript大厅、牌组与牌桌；提交服务端提供的动作并显示本人视图 | [web/src/game](web/src/game/README.md) |
| 构建和研究工具 | 磁盘/内存预检、当前Native输入绑定、卡图与来源检查、历史证据工具 | [tools](tools/) |
| 原始资料 / 文档 | 规则与卡图依据、历史设计和验收证据；不是自动生效的规则实现 | [resource](resource/)、[docs](docs/README.md) |

当前React主路径通过HTTP轮询更新：前台正常间隔约1.5秒，后台降频，恢复可见时同步；已确认暂停后停止轮询并可手动刷新。它没有使用WebSocket，也没有通过 `streamEvents` 建立SSE主循环。

客户端只保存席位凭据、牌组草稿、选择草稿和未确认命令，刷新后重新获取本人视图。未确认请求保留原commandId与原意图；Worker以鉴权、版本CAS及原始回执处理重复命令。Native与Sites的入桌能力不同：Sites提供创建/加入requestId回执，Native没有同一入桌幂等契约。

## 本地运行

以下命令从仓库根目录执行，使用你自己的普通磁盘输出目录。不要复用仍有任务运行的构建目录或旧版本数据库。本次验证环境为Linux、Node **24.19.0**、npm **11.9.0**、Rust **1.90.0**；Rust版本由 [rust-toolchain.toml](rust-toolchain.toml) 固定，npm依赖使用各自锁文件。

```sh
export UNDERGROUNDBATTLE_BUILD_ROOT="$(pwd)/../undergroundbattle-local-build"
node tools/build-storage.mjs
npm --prefix web ci
```

`tools/build-storage.mjs`将Cargo target和临时目录放到普通磁盘，串行编译并关闭增量/debug缓存；启动前检查至少1GiB磁盘余量和Linux cgroup内存余量。这只是启动门禁，不是完整编译/大规模回放的峰值资源保证。不要跳过预检或把输出转到tmpfs。

### Native + Vite：无需冻结旧WASM即可开发

终端A：

```sh
HEGEMONY_DB="$UNDERGROUNDBATTLE_BUILD_ROOT/native-dev.sqlite3" \
  node tools/build-storage.mjs -- cargo run --locked -p hegemony-server
```

终端B：

```sh
npm --prefix web run dev -- --host 127.0.0.1
```

按Vite输出的本地地址打开客户端；其 `/api` 默认代理到 `127.0.0.1:8090`。Native默认监听 `0.0.0.0:8090`；可设置 `PORT`，更换端口时也需调整Vite代理。若未设置 `HEGEMONY_DB`，数据库文件名仍是历史命名 `rust-game-v2.8.sqlite3`，**该文件名不表示引擎版本**。

若要由Native托管构建后的前端：

```sh
node tools/build-storage.mjs --output-dir "$PWD/web/dist" -- npm --prefix web run build
```

然后用上面的Native命令启动；`WEB_DIST`默认 `web/dist`，访问本地8090即可。不要同时在同一端口重复启动服务。

### Sites Worker/D1：本地仿真

先完成下节当前WASM构建，再执行：

```sh
npm --prefix sites ci
npm --prefix sites run build
npm --prefix sites run db:local
npm --prefix sites run dev -- --ip 127.0.0.1
```

本地入口为 `http://127.0.0.1:8102`。`db:local`明确使用本地D1；`build`调用 `wrangler deploy --dry-run`，输出 `sites/dist/server`、`sites/dist/client`，不发布到线上。Worker返回的状态仍由Rust WASM裁决。

## WASM、版本与旧牌桌

构建当前WASM前，需要 `wasm32-unknown-unknown` 和与crate匹配的 `wasm-bindgen-cli 0.2.104`；仅在缺失时安装：

```sh
rustup target add wasm32-unknown-unknown --toolchain 1.90.0
node tools/build-storage.mjs -- cargo install wasm-bindgen-cli --version 0.2.104 --locked
bash rust-game-wasm/build.sh
```

当前输出为被Git忽略的 `rust-game-wasm/pkg/`。**交付只包含当时最新的一套内核**。第三批ABI94已在独立克隆冻结，本第四批ABI95尚无生成的WASM；不可把旧产物充作本批产物。构建清空旧的generated与dist产物，Worker只导入当前JS/WASM；Sites源码同步也删除先前副本中的冻结内核目录。

用户于2026-10-21 16:30明确取消旧牌桌兼容。单核候选只接受当前 `hegemony-pdf-v1` / `limited-v2.83-batch20-next` / `rust-v0.2.93-batch20-next` 完整身份；旧房间与旧创建、加入、命令回执均返回410 `unsupported_room_version`，提示新建牌桌。旧数据保留，不迁移、不把旧opaque状态送入新核。Native服务同样只接受当前身份。

**持续开发约定（用户于2026-10-21 17:53再次确认）：隐秘世界以后也不提供旧牌桌兼容。** 每次交付只含该版本最新内核，优先减少包体、构建空间和运行开销；不再增加旧内核路由、保留包或状态迁移层。历史源码与审查证据用于追溯，不是运行依赖。此约定仅适用于本项目；当前规则正确性、u64精度、schema/状态完整性、权限、付费与重复请求保护及旧身份明确拒绝仍必须保留。

房间外层schema为3，内部Game schema为2；现有状态完整性验证仍保留。JS只读Rust导出的短身份，完整状态保持opaque字符串，避免损坏u64种子。移除的是旧版本执行分支，不是精度、付费回执、CAS或恢复校验。

历史源码、身份清单和审查输入保留在Git及原证据目录中。当前机制回归已移至实际ABI93 Native输入；无人引用的旧内核loader与仅比较ABI26/27的过期探针已删除。原始旧状态只用于拒绝边界，不能通过改版本字段或重编替代原件。迁移与验证状态见[当前夹具记录](docs/CURRENT_FIXTURES_ABI93_2026-10-21.md)。

## 本地测试与已知验证边界

按改动范围选择运行，以下全部是本地命令，**不是GitHub CI**。先完成依赖和当前WASM构建。当前夹具迁移的最终全套仍在进行，不能把下面的定向测试称作全套。共享夹具绑定导出时的Rust输入和WASM哈希；修改这些输入后须重新真实导出，不能重签旧状态。

```sh
# Rust默认workspace
node tools/build-storage.mjs -- cargo test --locked --workspace

# 实际Native生成、WASM逐步对照，以及当前XQ24恢复入口检查
bash rust-game-wasm/verify.sh

# React组件与客户端测试
npm --prefix web test -- --maxWorkers=2

# 来源与工具检查（完整checkout需包含测试引用的原始卡图与规则PDF）
node --test tools/*.test.mjs tools/cards/*.test.mjs
```

当前单核的HTTP测试使用独立固定时钟wrapper，目标目录必须不存在，并与 `sites/dist/server` 位于可硬链接的同一文件系统：

```sh
node tools/build-current-clock-worker.mjs "$UNDERGROUNDBATTLE_BUILD_ROOT/current-worker/server"
cd sites
CURRENT_CLOCK_WORKER="$UNDERGROUNDBATTLE_BUILD_ROOT/current-worker/server/worker.js" \
  node --test --test-concurrency=1 test/kernel-binding.test.mjs test/kernel-routing.test.mjs \
  test/integration.test.mjs test/current-kernel-http.test.mjs test/storage-retry.test.mjs test/session-host.test.mjs
```

wrapper只供本地测试，不能发布。当前单核的改动、实测体积/启动/内存、已通过项及未迁移历史测试边界见[清理记录](docs/LATEST_KERNEL_ONLY_2026-10-21.md)。此前旧版专用多核wrapper已删除；不要拿它的历史结果证明当前构建通过。

下表是此前d98117d阶段的历史验证记录，当前ABI93结果另见上方迁移记录：

| 检查 | 已记录结果与边界 |
| --- | --- |
| Rust workspace | 较早修复阶段1365通过；最终法术奖励限制收窄/死原型删除后另有定向回归，不称最终HEAD全套重跑 |
| Native→最终WASM | 433场景、24407迁移、96991视图相等；Native生成器阶段差异详见整改记录 |
| XQ24当前恢复 | 4非法状态/12入口拒绝；12正常场景117命令370视图相等 |
| Web | 原全套1431通过/1陈旧文案断言失败；修正后该文件4/4通过 |
| Sites | 原全套736项：708通过/28旧当前身份断言失败；四个文件31项定向补测覆盖全部28失败并通过 |
| 构建 | 当前WASM、Web、Sites14模块dry-run通过；独立stage的冻结loader/backend dry-run通过，未重做该副本前端构建 |
| 未运行/未通过的门槛 | 固定d98117d有限独立复审已完成且已验证范围无新增P0–P2；没有最终全套、该候选浏览器完整对局、旧桌兼容独立确认或全卡组合穷举 |

组件、Native回放、HTTP/workerd/D1仿真均不是浏览器试玩。原失败日志与补测结果分别保留；不要合写为“最终全套一次通过”。

前期README更新核对了65个本地链接和13段shell示例语法；在上述候选源码上完成Native构建，并用新建本地数据库验证启动命令、`/api/health`、`/api/catalog`和既有前端构建的静态首页。没有因此重跑既有全套或浏览器对局。

本批必要构建和新房间检查通过；集中检查中Web最终145文件1471项、来源/冻结工具84项全过；修复后的Rust workspace最终整轮1395/1395、Sites受影响15文件81/81通过。修复版WASM完整440场景全部通过，核对24582次迁移及97691个视图，末尾XQ24恢复检查也通过；对应源码、WASM及日志哈希见[本轮回放证据](docs/evidence/batch20-2026-10-21/fixed-wasm-replay.json)。修复版官方本地Site构建和20卡新房检查通过。历史失败、原v72的433场景和修复版13场景定向结果分别保留于[批次记录](docs/factions/batch20-2026-10-21.md)。上述结果针对首批ABI91。首批Sites最终轮首次644通过/12失败，原因为本地运行目录缺少原ABI78–89输入；原件恢复核验后，同一固定7f快照的最终全套736通过、0失败、0跳过，exit0；见[完整运行证据](docs/evidence/batch20-2026-10-21/final-sites-suite.json)。此结果不覆盖ABI92或ABI93。独立复审已发现B20-IR-01；最小P2源码修复a0b1b3d补验通过，不能扩张为后续UX、WASM或第二批独立通过。协调任务反馈v73一局8回合A8:B6自然完局及终局刷新通过，不等于所有20张机制验收。

## Sites官方发布方式与当前限制

已有Site身份在 [sites/.openai/hosting.json](sites/.openai/hosting.json)，更新时复用项目和现有访问范围，不重新注册Site。详细流程见 [Sites README](sites/README.md)：使用当前官方Sites插件的源码工作流同步同一版本源码、执行必要构建并打包，再按已有受众保存版本和部署。GitHub仓库的main与Sites源码仓库不是同一个远端；单独推GitHub main不构成一次已验证发布。

2026-10-21用户更新发布顺序为：**每批完整整理并上线20张卡，再集中全套重跑、独立复审和dot真实浏览器对战**。发布前保留必要构建、启动与版本身份检查，确认无已知破坏性故障；上线批次标记待验收。旧文档中的全套/独立复审先行门槛不再拦本批发布，但仍不能把未实现的能力开放为可玩。

**禁止运行GitHub CI的限制仍有效。** 用户已明确授权正常保存和推送相关源码；`d98117d`及其前三个相关提交已快进推入main，Git与GitHub只读接口核对一致。推送前后的实际树没有 `.github/workflows`，已知配置无push自动部署证据，推送后即时查询未新增Actions运行。连接器的workflow清单HTTP400表示该URL不受支持，不表示Git保存权限被拒绝；不能据此断言存在未知外部CI，也不承诺所有外部服务均已穷举。后续仍检查实际触发器，不强推、不改权限、不创建PR/tag/release或触发CI。

Sites的显式发布操作及凭据明确接受的限时 `publish_on_push` 均可能产生生产部署。本批通过原始官方Sites 0.1.75 helper完成源码同步和打包，未开启自动私有发布。v72保存后发现奖励快照缺陷，修复版另存为v73；旧版未部署。先前部署拒绝均保留记录，收到用户2026-10-21 13:32 UTC针对本次操作的“批准发布”后，仅重试同一官方部署一次并成功。没有更改站点受众或使用替代发布接口。本环境随后读取公开健康接口返回403，已停止该访问核对；部署回执不等同于浏览器验收。随后P2修复独立保存为v74（Site源码fe182e656f35153be9225c0b17fbce708831b1ba），官方部署被自动审批以“未明确授权v74”拒绝；官方只读查询确认deployment_id=null，未执行。当时停止发布，不经下一批或其他接口绕过；后续持续授权与已创建部署见下方记录；精确目标与独立补验范围见[修复记录](docs/factions/batch20-society-zone-fix-2026-10-21.md)。

## 资料与维护

规则与卡牌原文以 [原始资料](resource/) 中的卡图和PDF为依据，OCR/整理内容只用于定位。[研究与准入资料](docs/factions/README.md)和[历史设计](docs/architecture/full-mechanics-design/README.md)保留各自范围，不作为已实现能力承诺。代码许可见 [LICENSE](LICENSE)。

维护时优先规则正确性、隐藏信息、重复操作与恢复边界；复用已有语义和校验，删除可证明无调用的冗余。行为变更以可复现命令链及适用测试说明，记录精确源码/运行字节身份；不为让测试变绿而重标旧状态、弱化哈希或跳过失败证据。

2026-10-21 15:41 UTC收到对现有公开站所有修复的明确持续授权后，同一v74官方部署重试一次。调用返回Internal error，但只读核对确认已创建部署 `appgdep_6ad8dd9acc1c8191912336351e022f25`；后端状态自15:43:22 UTC起仍为pending，未提供URL或失败原因。没有重复请求。见[发布记录](docs/evidence/batch20-2026-10-21/publication-v74.json)。

第二批通过官方helper完成精确源码推送、Web/Worker构建及打包，已保存为v75，Site源码 `cd32602ce1acd6d65a0bbed8d86f4b7d01885d82`，对应产品候选 `34e51ce1aa3da5676569875156199b1dcebfa599`。已接入保存版v74的精确ABI92原件，16模块构建、冻结工具12项、路由5项及16内核HTTP/D1检查通过。v75未请求部署，避免与pending的v74并发覆盖。保存回执、本地与服务端归档哈希的比对边界见[版本记录](docs/evidence/batch20-next-2026-10-21/publication-v75.json)。全套和真实浏览器对战仍未完成。2026-10-21 16:30用户明确取消旧桌兼容后，v75已退出发布队列，不再等待v74结束后自动发布；下一候选只含当前ABI93。固定9aa4fda第二批独立复审由协调任务负责，不覆盖后续单核清理。
