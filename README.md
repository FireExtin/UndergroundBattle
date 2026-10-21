# 隐秘世界 · 霸权数字版

《隐秘世界》桌游的有限卡池数字化实现：React 玩家界面、Rust 权威规则核，以及 Native/SQLite 与 Sites Worker/D1 两种服务宿主。当前规则包为《霸权》；不是全扩展、全卡牌或经典规则的完整实现。

## 当前状态（2026-10-21）

| 范围 | 已核实状态 |
| --- | --- |
| 产品源码候选 | `progress/batch20-society-zone-fix-20261021` 空秘社区P2修复候选，引擎 `rust-v0.2.92-society-zone-fix`；173张卡池不变，未混入下一批20卡 |
| GitHub保存状态 | 固定版本审查后的整改与前期README更新已推入main，已确认main为 `d1c068ea8c4162212cd19fa023841513c4c67e33`（含来源准备文档）；本批20卡仍在进度分支，Git保存不等于Sites发布 |
| 规则 / 卡池 | `hegemony-pdf-v1` / `limited-v2.82-batch20` |
| 最近已发布 Site | **v73 / ABI91**；2026-10-21官方部署成功；保留待验收状态，已发现空秘社区P2 |
| 审查及整改 | 固定产品源码 `100161b628e56b9153041d952ac0f5d83428ca88` 的审查后，完成8项明确问题的本地修复及相关清理；见[整改与验证记录](docs/LOCAL_CORRECTNESS_AUDIT_2026-10-21.md) |
| 修复发布与验证 | 必要构建/启动及最小P2源码独立补验已通过；已保存v74的官方部署被自动审批拒绝且未执行。首批固定7f的Sites全套仍在原进程运行；v73已有一局8回合自然完局，修复版浏览器补验待发布 |
| 首批20张与修复 | [首批清单](docs/factions/batch20-2026-10-21.md)已随v73上线；[空秘社区修复](docs/factions/batch20-society-zone-fix-2026-10-21.md)及两项UI反馈在本分支，下一批20张在另一独立进度分支开发 |

[已发布的游戏入口](https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site)当前为v73。官方部署 `appgdep_6ad8bf5fa23081918eca730d2b96e3bb` 绑定Site源码 `e1969c3ded2ffa44ce429be87c0005d928d5ef38`，与GitHub固定审查提交 `7f4e6d63b2498ea2f47a3a6e5b1bf29dfd3dc762` 属于不同源码仓库。见[发布回执与核对边界](docs/evidence/batch20-2026-10-21/publication-v73.json)；有缺陷的v72未部署。

Go服务、旧调试器和 `/legacy-debugger` 已退役。此前README中的Go正式架构、WebSocket路线和计划目录不是当前实现；历史内容仍可从Git历史查看。

## 卡牌与玩法范围

当前运行目录由 [cards.json](rust-game/data/cards.json)、[catalog.rs](rust-game/src/catalog.rs) 与 [society.rs](rust-game/src/society.rs) 共同提供；以实际 `/api/catalog` 为准，已有牌桌使用其自己的 `/api/rooms/{roomId}/catalog`。

| 目录类型 | 数量 |
| --- | ---: |
| 角色 | 105 |
| 事务（接口类型 `spell`） | 33 |
| 附属 | 25 |
| 地区 | 10 |
| 上述规则卡合计 | **173** |
| 另列的秘社 | **10** |

173张目录卡包含地区，玩家牌种合计163张；v73与本修复候选均为173张规则卡。世界牌为DQJC107–DQJC116各一张。支持红、黑、蓝、灰等已准入牌的相应机制，不代表这些阵营全部卡牌已实现。原始研究目录、卡图存在或历史候选文档均不等于运行时准入。

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
| Sites宿主 | Worker HTTP、D1、创建/加入回执、命令CAS事务、旧内核分派、静态资源 | [sites](sites/README.md) |
| 玩家客户端 | React/TypeScript大厅、牌组与牌桌；提交服务端提供的动作并显示本人视图 | [web/src/game](web/src/game/README.md) |
| 构建和研究工具 | 磁盘/内存预检、冻结输入校验、卡图与来源检查、历史证据工具 | [tools](tools/) |
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

先完成下节WASM与冻结输入准备，再执行：

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

当前输出为被Git忽略的 `rust-game-wasm/pkg/`。Sites及不少Web/工具测试还需要精确的历史运行输入：

```text
$UNDERGROUNDBATTLE_BUILD_ROOT/frozen-kernels/abi77/…
…
$UNDERGROUNDBATTLE_BUILD_ROOT/frozen-kernels/abi89/…
$UNDERGROUNDBATTLE_BUILD_ROOT/frozen-kernels/abi91/…
每个目录：hegemony_wasm.js、hegemony_wasm_bg.wasm.gz
```

也可用已有 `HEGEMONY_FROZEN_ABI77_DIR` 至 `HEGEMONY_FROZEN_ABI89_DIR`，以及 `HEGEMONY_FROZEN_ABI91_DIR` 分别指定目录。Git中保存[身份与来源绑定](rust-game-wasm/frozen/)，运行字节是维护者提供的外部固定输入。**干净克隆并不自带这些旧二进制**；缺失时校验会停止，不能通过当前源码重编旧ABI来补齐。

ABI77–88取自已核验迁移包；ABI89必须使用v71的P2修复后原件。旧5056迁移包中的ABI89不能替代它。v71 WASM为3471247字节，SHA256 `df890ed0b596ca245a39bc8466270edc2e8f40f92ef8dd34359534342ccf24de`；输入包、运输压缩哈希及准备说明见[本地整改记录](docs/LOCAL_CORRECTNESS_AUDIT_2026-10-21.md)。

候选Worker沿现有路由包含 **当前ABI92 + 原始ABI77–89、91，共15个内核**。按完整规则/卡池/引擎身份匹配；未保留的版本明确拒绝，不将旧opaque状态改写后送入新核。房间外层schema为3，内部Game schema为2。JS宿主将完整状态视为opaque字符串，避免破坏u64种子。

本修复仅作用于新建ABI92房间；保留历史内核也保留其原版缺陷，因此已有v73/ABI91牌桌仍无法将四张秘社附属结附无秘社牌的区。未自动迁移、清空或重签旧桌。Native二进制仍只接受当前内核身份的数据库，不具有Sites的多内核兼容能力；不存在通用旧SQLite存档迁移。

## 本地测试与已知验证边界

按改动范围选择运行，以下全部是本地命令，**不是GitHub CI**。Web、Sites与冻结工具测试先完成上节依赖和原始输入准备。

```sh
# Rust默认workspace
node tools/build-storage.mjs -- cargo test --locked --workspace

# 实际Native生成、WASM逐步对照，以及当前XQ24恢复入口检查
bash rust-game-wasm/verify.sh

# React组件与客户端测试
npm --prefix web test -- --maxWorkers=2

# 来源与冻结输入工具检查
node --test tools/*.test.mjs tools/cards/*.test.mjs
```

Sites全套测试还需要已构建的15模块Worker和独立本地固定时钟包装目录；目标目录必须不存在，并与 `sites/dist/server` 位于可硬链接的同一文件系统：

```sh
node tools/build-jc035-clock-worker.mjs "$UNDERGROUNDBATTLE_BUILD_ROOT/society92-worker/server"
HEGEMONY_CURRENT_WORKER_DIR="$UNDERGROUNDBATTLE_BUILD_ROOT/society92-worker/server" \
  npm --prefix sites test
```

该包装器只供测试，不能发布。不要用旧版专用wrapper的模块数或陈旧脚本输出证明当前构建通过。

以下是固定审查整改阶段的历史结果，不代表本ABI92候选已通过全套：

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

本批必要构建和新房间检查通过；集中检查中Web最终145文件1471项、来源/冻结工具84项全过；修复后的Rust workspace最终整轮1395/1395、Sites受影响15文件81/81通过。修复版WASM完整440场景全部通过，核对24582次迁移及97691个视图，末尾XQ24恢复检查也通过；对应源码、WASM及日志哈希见[本轮回放证据](docs/evidence/batch20-2026-10-21/fixed-wasm-replay.json)。修复版官方本地Site构建和20卡新房检查通过。历史失败、原v72的433场景和修复版13场景定向结果分别保留于[批次记录](docs/factions/batch20-2026-10-21.md)。上述结果针对首批ABI91。独立复审已确认B20-IR-01，真实浏览器亦复现；本ABI92修复新增Native9、批次30、秘社6、空区/秘社Web19通过，整合UX的7文件114通过，触发正文最终5通过。首批Sites最终轮首次644通过/12失败，原因是独立运行目录缺原ABI78–89输入；原件现已恢复核验，修正后的全套已于14:56 UTC启动，仍在原进程运行。本ABI92的最终构建、新房启动、8场景Native/WASM对照、15内核HTTP/D1保留检查均通过。协调任务反馈最小a0b1b3d源码独立补验通过（不含WASM、后续UX或第二批）。v73一局真实8回合A8:B6自然完局及终局刷新通过；不代表所有20张机制通过，修复版浏览器仍待发布。

## Sites官方发布方式与当前限制

已有Site身份在 [sites/.openai/hosting.json](sites/.openai/hosting.json)，更新时复用项目和现有访问范围，不重新注册Site。详细流程见 [Sites README](sites/README.md)：使用当前官方Sites插件的源码工作流同步同一版本源码、执行必要构建并打包，再按已有受众保存版本和部署。GitHub仓库的main与Sites源码仓库不是同一个远端；单独推GitHub main不构成一次已验证发布。

2026-10-21用户更新发布顺序为：**每批完整整理并上线20张卡，再集中全套重跑、独立复审和dot真实浏览器对战**。发布前保留必要构建、启动与版本身份检查，确认无已知破坏性故障；上线批次标记待验收。旧文档中的全套/独立复审先行门槛不再拦本批发布，但仍不能把未实现的能力开放为可玩。

**禁止运行GitHub CI的限制仍有效。** 用户已明确授权正常保存和推送相关源码；`d98117d`及其前三个相关提交已快进推入main，Git与GitHub只读接口核对一致。推送前后的实际树没有 `.github/workflows`，已知配置无push自动部署证据，推送后即时查询未新增Actions运行。连接器的workflow清单HTTP400表示该URL不受支持，不表示Git保存权限被拒绝；不能据此断言存在未知外部CI，也不承诺所有外部服务均已穷举。后续仍检查实际触发器，不强推、不改权限、不创建PR/tag/release或触发CI。

Sites的显式发布操作及凭据明确接受的限时 `publish_on_push` 均可能产生生产部署。当前官方Sites 0.1.75 helper已经恢复，167个文件清单核验通过。v73在用户针对先前拒绝明确批准发布后，经同一官方动作成功上线；先前拒绝记录保留，未换路径绕过。ABI92修复候选已保存为v74，Site源码fe182e656f35153be9225c0b17fbce708831b1ba；官方部署被自动审批拒绝，理由是此前明确批准针对v73，当前证据未明确授权v74。随后官方get_site_version仍返回deployment_id=null，确认未执行；已停止该动作并交协调任务处理，不换接口或用下一批间接发布。没有重新发布v72，也没有部署下一批实现。精确目标、拒绝原文和验证范围见[修复记录](docs/factions/batch20-society-zone-fix-2026-10-21.md)。

## 资料与维护

规则与卡牌原文以 [原始资料](resource/) 中的卡图和PDF为依据，OCR/整理内容只用于定位。[研究与准入资料](docs/factions/README.md)和[历史设计](docs/architecture/full-mechanics-design/README.md)保留各自范围，不作为已实现能力承诺。代码许可见 [LICENSE](LICENSE)。

维护时优先规则正确性、隐藏信息、重复操作与恢复边界；复用已有语义和校验，删除可证明无调用的冗余。行为变更以可复现命令链及适用测试说明，记录精确源码/运行字节身份；不为让测试变绿而重标旧状态、弱化哈希或跳过失败证据。
