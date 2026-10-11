# Engine63：BQ030 与固定空槽组合候选

本批独立 QA 将已经交付的 `591b48f0c05e64aa59d3520361a8d53c3fa09d33`
与 GitHub 上的 BQ030 固定源码 `23569135031d2e8e2fd18caf66ca6d2ef35a64b7`
整合。实际冲突为 `resolution.rs`、`catalog.rs`、`card-scans.json` 三处。
来源记录同时保留 BQ030 和 `!vacant` 过滤；卡图共有记录逐项相同，仅加入 BQ030。
没有合入 `16efe7b`、没有改 main、没有新建 worktree、没有发布网站。

统一身份：规则 `hegemony-pdf-v1`，卡池
`limited-v2.56-bq030-attachment-candidate`，生产引擎
`rust-v0.2.63-bq030-fixed-slots-candidate`，fixture 引擎为对应 `-fixture`。
129 张普通牌（119 张非地区牌及 10 张地区），140 条卡图记录。
旧版本房间直接拒绝，不执行状态迁移。BQ030 运行模块与上述固定源提交字节相同；
组合调整为引擎身份、Worker 绑定、测试计数和当前 Native fixture。

以下是本工作区执行所得的独立证据，不表示核过此前未收到的 ZIP，也不代替浏览器试玩。
未推送的 `b3749f087386a1446c7a0cb69298ef887242ba04` 未在本工作区读取；
本批另增六项组合回归，覆盖空库淘汰（含固定空槽与多个已入栈触发）、
实际支付授予的声望接受/跳过、响应使来源离场后奖励继续、声望赢区留下原位空槽、
真实战斗授予的威名接受/跳过。原有致死结附保留可选抓牌、多个来源排序和非法存档用例均保留。

全部 Native fixture 由当前 Rust 重新执行命令产生。历史 v057/v060/v062 文件保留。
`current_ui_worker_fixtures` 在离线初始布局上执行当前严格加载、牌组校验及真实 Room 命令，
并逐项比对原有语义；旧 Win 布局的恢复上下文来自重新执行的 `death_observer_tests`，不猜测。
P1/BQ030 选择脚本保存原始 opaque state 字符串和来源文件 SHA，不重写运行存档。
React 用例通过真实选择控件调用生产 WASM；D1 用例通过生产 RoomService 与实际 SQLite，
逐项核完整存档、journal、座位视图、重启和重复命令回执。普通 HTTP 回归独立覆盖 Worker 路由。

## 验证

最终执行结果记录于同目录的 `ENGINE63_BQ030_COMBINED_QA_2026-10-11.json`。
本工作区最终通过：完整 Native 930/930（库 863、集成 67，含 BQ030 27）、
UI 89 文件 833/833、Worker 91/91、卡图/来源 6/6，以及类型检查和构建。
新生成的 433-case Native/WASM 语料核 24403 次转移、96975 次投影、319 次命令拒绝和 42 次建组拒绝；
BQ030 专项另核 379 个完整命令、1516 个座位视图及 37 个非法存档；P1 专项通过 1865 个原始输入。
生产 WASM 为 2640686 字节，SHA256 `23f0d415e584afcfa69c87d29df59e095e67dbe76a362265f19674431c068e1d`。
真实 HTTP 核 144 个客户端资产、建桌/加入/准备、重复回执及 D1 重启通过。
工具链为 Rust/Cargo 1.90.0、wasm-bindgen 0.2.104、Node 24.19.0。
首轮 full Native 的唯一失败为旧 118 张非地区牌断言；修正为 119 后执行完整重跑。
新增交叉测试首轮 5/6，标准行动牌错误地从对抗前窗口开始；改为行动窗口后连续真实命令推进。
没有放宽 UI 超时，也没有用 mocked API 代替 Worker。

复现主要检查（仓库根目录）：

```bash
cargo test --locked --workspace
bash rust-game-wasm/build.sh
cargo run --locked -p hegemony-wasm --example native_fixtures -- /tmp/engine63-native.json
node rust-game-wasm/tests/compare.mjs /tmp/engine63-native.json
node --test tools/cards/*.test.mjs
npm --prefix web test -- --maxWorkers=1
npm --prefix sites run build
cd sites
node --test --test-concurrency=2 test/*.test.mjs
```

重新生成当前 UI/Worker fixture 的命令（先设置相应证据目录）：

```bash
DEATH_OBSERVER_FRONTEND_DIR=/tmp/engine63-death cargo test --locked -p hegemony-server death_observer_tests:: --lib
WIN_FLOW_EVIDENCE_DIR=/tmp/engine63-p1 cargo test --locked -p hegemony-server win_flow_tests:: --lib
BQ030_TRACE_DIR=/tmp/engine63-bq030/room cargo test --locked -p hegemony-server --test bq030_attachment_regression -- --test-threads=1
cargo run --locked -p hegemony-wasm --example current_ui_worker_fixtures -- . /tmp/engine63-death
cargo run --locked -p hegemony-wasm --example seven_card_ui_fixtures -- web/src/game/sevenCardUIV063.fixture.json
python3 tools/fixture-tools/current-win-flow.py /tmp/engine63-p1
node tools/fixture-tools/current-bq030.mjs /tmp/engine63-bq030 rust-game-wasm/pkg/hegemony_wasm.js
node tools/cards/compare-bq030-room.mjs rust-game-wasm/pkg/hegemony_wasm.js /tmp/engine63-bq030
```

## 父端启动与待验收

交付使用新的 GitHub 审查分支，精确 SHA 以父会话收到的普通 push 读回结果为准。
父端在自己的执行环境读取该提交，使用上述工具链和锁文件，先运行
`npm ci --prefix web`、`npm ci --prefix sites`，再执行 WASM 和 Sites 构建。
可复用匹配锁文件的依赖与 Rust 缓存。此 Git 交付不含依赖目录、缓存、运行数据库、
用户状态或凭据；WASM pkg 为生成目录，须从当前源码构建。

本地真实 Worker/D1 启动方式：

```bash
cd sites
HOST=0.0.0.0 PORT=8113 HEGEMONY_QA_STATE_DIR=/tmp/engine63-qa-d1 node scripts/qa-preview.mjs
```

启动脚本直接加载构建产物，在指定目录自动初始化并保留本地 D1 SQLite，无需账户认证。
默认端口为 8113；父端云浏览器应访问其自身执行环境暴露的本地服务地址。
也可由父会话按已授权的官方私有 Sites 流程从同一精确 SHA 构建访问；本 QA 没有部署。
QA 工作区的 `127.0.0.1:8113` 只用于本工作区运行核验，父端不能直接连接该 localhost。
原 `8112` 仍是上一批 Engine62 预览，不能用于本批 BQ030 验收。

父端仍需实际浏览器检查：自然对局中的赢区续接与声望时机、空世界牌堆保留原位、
BQ030 致死结附后接受/跳过及多来源排序、真实刷新/重连、卡图可读性和移动端操作。
本文件的 prepared-layout 自动回归不宣称自然对局或浏览器验收通过。
