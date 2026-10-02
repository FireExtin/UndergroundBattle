# 云端 Chromium 试玩与验收

脚本位于 `tools/cloud-playtest/`，仅使用云端 Python Playwright 和 `/usr/bin/chromium`，无需 Mac 或隧道。规则边界遵循 [云端切片接口与已核对规则](HEGEMONY_CLOUD_SLICE_2026-10-02.md)。这些验收只检验公开接口和玩家操作流程，不替代逐卡规则测试。

最新验收对应固定生产源码 `d94baaf3e7caabfdb81e1b3cc2a537fefe355fd0`、`rust-v0.2.1` / `limited-v2.1`，开放 29 个定义（25 张玩家牌、4 张世界牌）。最终结果见文末；此前版本的成功与失败记录均保留为历史证据。

从仓库根目录运行，先构建 Rust 服务和 React 静态资源，并让服务在 8090 监听：

```bash
cargo build -p hegemony-server
npm --prefix web run build
PORT=8090 HEGEMONY_DB=/tmp/hegemony-playtest.sqlite3 WEB_DIST=web/dist target/debug/hegemony-server
```

另一个终端运行真实 UI 试玩：

```bash
python3 tools/cloud-playtest/browser_playtest.py \
  --base-url http://127.0.0.1:8090 \
  --output /tmp/hegemony-cloud-playtest-2026-10-02
```

`browser_playtest.py` 为双人局和四人 2V2 各建独立浏览器上下文，点击创建/加入、各自选不同预组、准备/开始、再调度、卡牌上下文行动及待选表单。脚本读取各端 HTTP/SSE 已接受视图来决定合法动作；不通过 API 提交游戏动作，不注入管理状态或特制卡。策略每回合建立资产、派遣/暗派、选择调查顶底排序、伤害与弃牌等，并在必要时让过，直到规则自然结束。默认最多 4500 个动作，失败会记录最后状态和截图。

独立 HTTP 验收和服务重启验收：

```bash
python3 tools/cloud-playtest/api_acceptance.py \
  --output /tmp/hegemony-cloud-acceptance-2026-10-02
python3 tools/cloud-playtest/browser_restart.py \
  --output /tmp/hegemony-cloud-browser-restart-2026-10-02
python3 tools/cloud-playtest/autopass_probe.py \
  --base-url http://127.0.0.1:8090 \
  --output /tmp/hegemony-cloud-autopass-2026-10-02
```

`api_acceptance.py` 使用自有 8091 子进程/SQLite 测试鉴权、跨房令牌、伪造 actor、409 过期版本、命令幂等、非法选择不改变持久化状态、SSE 本人手牌/待选及队友和对手暗牌投影，以及进程重启后原状态、实体身份、待选和去重响应恢复。每次拒绝比较持久化状态摘要、命令数量及日志数量。

`browser_restart.py` 使用自有 8092 子进程和明确标识的 API 房间夹具。在浏览器中观察原待选，停掉自己创建的进程、看到自动重连提示，再启动同一个数据库，验证浏览器恢复完全相同的已确认视图，最后点击保留手牌继续原选择。它属于恢复验收，不计为真实 UI 开局或完整对局。

两个恢复脚本只停止自己 `Popen` 创建的 PID，并拒绝占用已经监听的端口。不会重启 8090 共享服务。所有启动使用有限健康检查重试。

`autopass_probe.py` 通过两个独立浏览器真实创建/加入小房间，验证可选自动让过默认关闭，启用后遇到本人或他人待选时不推进，在唯一让过窗口推进，遇到其他合法行动时停止，而且可随时关闭。完整对局仍保留默认关闭，逐次点击让过。

新手交互另用只读可见 DOM 的短探针，避免用引擎合法动作列表来证明界面可理解：

```bash
python3 tools/cloud-playtest/newcomer_probe.py \
  --base-url http://127.0.0.1:8091 \
  --output /tmp/hegemony-cloud-newcomer-verified-2026-10-02
python3 tools/cloud-playtest/newcomer_probe.py --reader-only \
  --base-url http://127.0.0.1:8091 \
  --output /tmp/hegemony-cloud-phone-reader-2026-10-02
python3 tools/cloud-playtest/layout_probe.py \
  --output /tmp/hegemony-cloud-layout-final-2026-10-02
```

该探针建立四个独立上下文，交替使用 1366×900 桌面与 390×844 触屏视口。它只读取网页上可见的状态、下一步提示、牌面图标、上下文按钮和待选说明；不读取 API JSON、请求体或 `legalActions`。HTTP 状态仅用于确认所点击的命令被接受。短流程检查谁行动/谁等待、唯一固定下一步提示在视口内、无横向页面溢出、我方视角、选择可达、暗牌归属和非本人身份隐藏，实际点击每次 1 点伤害按钮并确认可见伤害计数，还打开卡牌放大阅读。它不替代真人新手观察，也不计为额外完整对局。

`layout_probe.py` 专用于微调后的局部布局复看，使用自有 8095 服务；用正常 API 游戏命令生成实际伤害待选，再让浏览器检查 390/1366 视口中的公开拥有者名字、稳定目标序号和手机操作面板背景，截图并关闭自有进程。它明确标识为 API 布局夹具，不计入新手交互或完整 UI 对局，不注入状态或特制卡，也不保存座位凭证。

结果 JSON 保存动作类型、匿名实例引用、公开桌面、数量、版本、分数、覆盖、浏览器错误和截图路径。JSON 不保存座位令牌、私有手牌定义或私有选项文字。浏览器截图可能含自己的手牌/待选，SQLite 含完整权威状态，仅保存在所选本机输出目录，禁止上传私有卡图到第三方。请把输出放 `/tmp` 或已被忽略的 scratch 目录，不提交运行产物。

## 历史 v1 UI 与局部交互结果

双人 UI 对局已通过：两个独立 Chromium 上下文，387 个游戏循环动作，390 个已接受命令，零拒绝命令；第 7 回合以 9–6 达到 8 分目标结束。实际选择覆盖再调度、伤害分配、调查、弃牌、地区回牌自定顺序、触发、检索和目标。无非预期浏览器错误；断线探针的 `ERR_INTERNET_DISCONNECTED` 单列为预期事件。

四人 UI 对局已通过：四个独立上下文、四套不同预组，2521 个游戏循环动作，2526 个已接受命令，零拒绝；第 16 回合最终团队分数 8–11，胜利目标 10。实际执行伤害、调查、地区回牌排序、检索、触发、先手特权与事务，无非预期浏览器错误。

修正后 Rust 构建已重验双人 UI 局：第 6 回合 8–3 得分胜利，313 个游戏循环动作，316 个已接受命令，零拒绝和非预期浏览器错误。修正后四人局也完成得分胜利：第 18 回合团队 0 为 7 分、团队 1 为 10 分，2913 个游戏循环动作，2918 个已接受命令，零拒绝和非预期浏览器错误；四席独立上下文。实际选择包含 11 次伤害、4 次调查、4 次弃牌、1 次奖励执行者、14 次自定回牌顺序、19 次目标及 9 次触发。修正后独立 HTTP 八项验收、浏览器进程重启恢复和可选自动让过真实 UI 探针均通过。8091 当时为后端代理持有的最终服务，隔离恢复探针改用自有 8093/8094 端口。

新手可见 DOM 短流程通过：99 步、104 个已接受 UI 命令、四个独立视角，以上各项交互检查通过，浏览器错误为空。最初的字符串断言把“请完成下方选择”误判为不明确，首次失败输出另行保留；复核改为检查实际本人选择面板和可达操作，界面同时增强为“请你完成下方选择”。手机阅读的独立最新构建检查也通过：两台 390px 视口的窗口宽 362px、正文 16px，Tab 焦点留在窗口内，Escape 可关闭，浏览器错误为空。这里的浏览器全部运行在云端，所用入口仍为本机 HTTP；尚未获得已发布互联网 URL，不能据此宣称互联网部署或真人新手验收完成。

最终两处微调已局部复核：独立夹具接受 561 个正常 API 命令后进入实际伤害待选，桌面与手机都显示公开拥有者和目标 1；手机操作面板计算背景为不透明的 `rgb(36, 44, 50)`，没有透出底下文字，两个视口均无横向页面溢出，浏览器错误为空。源脚本后续还增强暗牌观察计数，必须实际观察本人、队友和对手关系才能标记隐私检查通过；原 99 步证据保持原样，未为这个统计增强再次跑整轮。

历史证据（本机私有，不入 Git）：

- `/tmp/hegemony-cloud-playtest-verified-2026-10-02/duel-summary.json`
- `/tmp/hegemony-cloud-playtest-verified-2026-10-02/duel-actions.json` 和逐行 `duel-actions.jsonl`
- 同目录 `duel-initial-lobby.png`、`duel-spatial-board.png`、`duel-private-choice-*.png`、`duel-finished.png`、`duel-mobile-board.png`、`duel-mobile-finished.png`
- `/tmp/hegemony-cloud-acceptance-2026-10-02/api-summary.json`
- `/tmp/hegemony-cloud-browser-restart-2026-10-02/browser-restart-summary.json`
- `/tmp/hegemony-cloud-playtest-final-engine-2026-10-02/duel-summary.json`
- `/tmp/hegemony-cloud-playtest-final-engine-2026-10-02/teams-summary.json`
- `/tmp/hegemony-cloud-playtest-final-engine-2026-10-02/duel-spatial-board.png`：最终构建正常牌桌
- `/tmp/hegemony-cloud-playtest-final-engine-2026-10-02/teams-private-choice-damage.png`：最终构建实际伤害选择
- `/tmp/hegemony-cloud-acceptance-final-2026-10-02/api-summary.json`
- `/tmp/hegemony-cloud-browser-restart-final-2026-10-02/browser-restart-summary.json`
- `/tmp/hegemony-cloud-autopass-verified-2026-10-02/autopass-summary.json`
- `/tmp/hegemony-cloud-newcomer-2026-10-02/newcomer-summary.json`：保留的首次断言失败
- `/tmp/hegemony-cloud-newcomer-verified-2026-10-02/newcomer-summary.json`
- 同目录 `newcomer-perspective-seat1.png` 至 `seat4.png`：桌面与手机四方视角
- 同目录 `newcomer-actual-damage-choice-seat3.png` 与 `newcomer-readable-card-seat3.png`
- `/tmp/hegemony-cloud-phone-reader-2026-10-02/newcomer-reader-summary.json`
- 同目录 `newcomer-phone-reader-seat2.png` 与 `newcomer-phone-reader-seat4.png`：手机放大阅读
- `/tmp/hegemony-cloud-layout-final-2026-10-02/layout-summary.json`：最终微调的局部布局夹具结果
- 同目录 `layout-mobile-inspector-390.png`、`layout-mobile-reader-390.png` 与 `layout-actual-damage-1366.png` / `layout-actual-damage-390.png`

## 保留最初的合法差策略

最初的策略优先把任何合法手牌转成资产，甚至每轮唯一新抓到的角色。其完整 UI 证据保留在 `/tmp/hegemony-cloud-playtest-2026-10-02/duel-actions.json` 与 `duel-summary.json`。它并非规则死局：2356 个命令全部被接受，86 次把最后一张手牌作为资产；所有回合仍推进，第 47 回合对手下一次必须抓牌时因空牌库出局，最终得分仍为 3–0。最后一张牌被抓走导致牌库为 0 时，玩家尚未出局；随后强制抓牌才失败。

原策略实现保存在 `tools/cloud-playtest/initial_policy.py`，可再次运行：

```bash
python3 tools/cloud-playtest/browser_playtest.py \
  --modes duel --policy initial --max-steps 4500 \
  --output /tmp/hegemony-initial-policy-reproduction
```

随机发牌会导致不同路线；原局的精确 seed、初始状态、完整已接受命令 journal 另保存在 `/tmp/hegemony-original-ui-policy-2026-10-02/private-room-replay.json`，仅用于本地离线审计，含私有牌序但不含 seats 表、令牌或令牌哈希。配套 `original-policy-summary.json` 只记录公开计数。`preserve_trace.py` 提供只读提取与规则转移检查，原输出文件未被新策略覆盖。

已检查原 PDF 印刷 P8（PDF 第 9 页）开始阶段：每位玩家从自己的牌库顶抓 1；建立资产的约束为每回合一次，没有禁止把最后角色作资产。原 trace 覆盖 47 次准备→抓牌转移，检查有牌可抓的玩家手牌 +1、牌库 −1，并保留最后一次空牌库导致出局的真实转移。印刷 P3（PDF 第 4 页）空牌库败北条件由父代理查看原页确认，核对页码记录在 [source-verification.json](../rust-game/data/source-verification.json)。新策略调整的是玩家决策，游戏仍允许这些合法建资产操作。2V2 队友继续行动与出局区域/栈清理由后端明确测试覆盖，未用管理注入伪装为完整 UI 对局。

## 历史 v2 响应与完整 UI 验收

该轮固定服务版本为 `rust-v0.2.0`。二进制固定副本在 `/tmp/hegemony-response-v2-2026-10-02/bin/final/hegemony-server`，SHA256 为 `56a98031ef54c49f5b8634310a5e8728b4e38f516387e92e1f0c55fd39afc6f0`；较早 v2 副本 SHA256 为 `5ee67315779c8ee8792c702e2f6a3bb5d08f1df4c2d0f9512bd6c27271f5850a`。只测试各自拥有的 8096/8097 子进程与独立 SQLite，没有停止或更换共享 8090/8091。

```bash
python3 tools/cloud-playtest/response_v2.py \
  --binary /tmp/hegemony-response-v2-2026-10-02/bin/final/hegemony-server \
  --output /tmp/hegemony-response-v2-2026-10-02/final
python3 tools/cloud-playtest/full_ui_v2.py \
  --binary /tmp/hegemony-response-v2-2026-10-02/bin/final/hegemony-server \
  --output /tmp/hegemony-cloud-playtest-final-v2-2026-10-02
```

`response_v2.py` 先用正常 API 创建、加入、再调度、逐轮抓牌、按忠诚建立资产和派遣，所有参与者使用第五套 50 张 `responders` 预组。创建接口没有传入 seed，不写数据库、不注入状态。准备阶段按强制手牌上限保留主链，最多到第 15 回合便停下检查阻碍。准备达到真实合法“谋杀→末日信徒”局面后，声明谋杀、让过、点击快速牺牲响应以及全部队友让过都通过实际浏览器按钮。`common.action_payload` 完整转发服务的 `abilityId` 与 `costSelected`。

最终版本的双人响应夹具用了 364 个 API 游戏准备命令、7 个实际 UI 游戏命令，另有 1 个去重 API 探针；四人夹具为 1504、12 和 1。这里的准备计数包含 `/commands` 的准备/开始，不包含创建/加入或只读请求。两个模式均检查：对手在让过前看到待结算谋杀；真实 `reduce-next` 快速行动存在时，启用自动让过仍保留响应；牺牲费用立即支付，旧目标 ID 显示 `missing/原目标已离场`；两人或四人都让过一次只结算独立的顶层效果；底层仅能让过的窗口可自动推进；谋杀因原目标失效整体取消，三资产费用不退，法术进入墓地。链前以及支付后响应中分别刷新、重启同一数据库，所有独立端接受视图完全一致，原命令去重重试不改变数据库或重放效果。

响应窗口与手机快速行动按钮位于可见视口内，两种宽度 390/1366 没有页面横向溢出，非预期浏览器错误为空。主动停止自有 SSE 服务所产生的 `ERR_INCOMPLETE_CHUNKED_ENCODING` 单列为预期恢复事件（双人 4 次、四人 8 次），不隐藏原始记录。JZ54 效果内牺牲待选没有在这两个浏览器响应夹具中覆盖：其 JSON 恢复由内核测试 `effect_sacrifice_private_choice_restores_accepted_guard_and_current_control` 检查；SQLite 中已接受执行帧、费用不重复支付、待选恢复和去重由 `paid_sacrifice_death_trigger_and_accepted_frame_restore_without_repayment` 检查，该 SQLite 测试使用 JC049/XQ12 的明确初始布局夹具，不能称作浏览器或直接 JZ54 SQLite 试玩。

最初 v2 API 准备误保留所有四色源的首张，同时弃牌未保住主链，资产只有中立来源，最终第 45 回合自然空牌库结束。该失败数据库与完整命令 trace 保留在 `/tmp/hegemony-response-v2-2026-10-02/`；修正为保留 JC042/JC091 的首张、允许 JC049/JZ54 作忠诚资产后即形成局面。`retry2/` 保留第一个实际跑通链却被预期 SSE 截断错误分类拦住的报告；`verified/` 为较早二进制通过的双/四响应，`final/` 为最终固定二进制的通过结果。没有覆盖失败输出，也没有通过改 seed 或注入补牌使夹具成立。

`full_ui_v2.py` 只启动/停止自有服务并记录 SHA256，游戏动作仍全部由 `browser_playtest.py` 在真实 UI 中完成。新增 `--response-deck` 把最后席换为 `responders`，各席牌组仍不同；原默认策略和原差策略可复现方式保留。完整局保持自动让过默认关闭，逐次点击，不把 API 响应夹具计入完整 UI 游戏。两局均通过，本次每个模式只完整运行一次：

| 最终完整 UI 局 | 各席预组 | 终局 | 游戏循环 | 接受命令 |
| --- | --- | --- | --- | --- |
| 双人 | watchers / responders | 第 11 回合，11–7，目标 8 | 587 | 590 |
| 四人 2V2 | watchers / hunters / keepers / responders | 第 14 回合，团队 0 为 10、团队 1 为 7，目标 10；个人分数 0/10/7/0 | 2279 | 2284 |

两局全部游戏命令均由浏览器 UI 产生，API 游戏命令为 0；零拒绝，非预期浏览器错误为空，断网恢复探针的 `ERR_INTERNET_DISCONNECTED` 各 1 次单列为预期。双人实际进行了建立资产、暗派、派遣、调查、弃牌、回牌排序、触发、事务、现身和发动；四人还完成 8 次伤害选择、12 次调查、11 次自定回牌顺序、4 次检索、2 次目标、8 次弃牌、4 次事务和 6 次先手特权。自有 8097 PID 38917 在运行退出码 0 后已停止，服务身份与停止记录见 `binary-and-service.json`。

父代理使用最终只读 audit 独立检查两个完整 UI 房间，均 `matches=true`，seed 与已接受 journal 重放完全等于持久化终局；本代理未重复运行审计。双人房间 `ac12372251bbf58200a9b8cc` 为版本 591 / 591 条 journal，摘要 `e881339ca8f984221eccb1a636da3b81cd3872d31a9080c13bcc5da759e8e5d1`；四人房间 `2522fdb7e9bac5a16ba38cb4` 为版本 2287 / 2287 条 journal，摘要 `5c09ebc7fa065eb310b5c80428c9ece12de685e081a3d2dd5b4ceb9ffeee3749`。journal 条数包含房间创建/加入，不能混作浏览器游戏命令数。父代理也已独立核对最终响应两房重放一致。审计不输出任何座位令牌或私有牌序。

最终证据目录：

- `/tmp/hegemony-response-v2-2026-10-02/final/response-summary.json`
- 同目录 `duel-response-summary.json` / `teams-response-summary.json` 与分别计数的 `*-fixture-commands.jsonl` / `*-ui-commands.jsonl`
- 同目录 `duel-fast-response-drawer-phone-seat2.png`：手机实际快速响应及固定待结算提示
- 同目录 `duel-original-target-missing-seat2.png` 与 `teams-original-target-missing-seat1.png` / `seat3.png`
- `/tmp/hegemony-cloud-playtest-final-v2-2026-10-02/binary-and-service.json`
- 同目录 `duel-summary.json` / `teams-summary.json`、全部 UI 动作 trace、正常牌桌、实际伤害待选及桌面/手机结束截图
- 同目录 `root-readonly-audit.jsonl`：父代理对两个完整 UI 房间的独立最终审计

## 固定 d94baaf / v2.1 最终验收

30 张原图集中核对后，LC19 外科医生的能力费用补齐横置自身；其公开限制在旧 v2 已存在，不能把它描述为新补的限制。香港 DQJC116 的同时展示、入手后洗牌流程尚未实现，已隔离出开放池。本轮为 25 张玩家牌加 4 张世界牌，共 29 个开放定义；五套预组各 50 张保持不变。世界牌库仍为 10 张：DQJC107×3、DQJC112×3、DQJC113×2、DQJC114×2，不包含香港。

生产源码固定为 `d94baaf3e7caabfdb81e1b3cc2a537fefe355fd0`。独立 fresh target 生成的原生二进制被复制到 `/tmp/hegemony-cloud-playtest-v2.1-2026-10-02/bin/hegemony-server`，SHA256 为 `148d703b10fb20978b5e4f05345825c5e71a1d22c9c6b3b64080bc3e09d0f2cc`。`source-build.json`、`binary-and-service.json` 和响应摘要记录源码 commit、二进制摘要、版本及各自数据库路径。父代理完成该版本 42 项原生检查，以及 WASM/native 的 397 次状态转移、1165 个投影比较；本代理未重复运行这些检查。

完整 UI 双人和四人局各只运行一次，使用自有 8099 服务与独立数据库。每席创建、加入、准备、开始、再调度和所有游戏命令均通过真实浏览器 UI；API 游戏命令为 0。自动让过保持默认关闭，各席牌组不同，最后席使用第五套 `responders`：

| 完整 UI 局 | 各席预组 | 实际终局 | 游戏循环 | 接受命令 | 拒绝 / 非预期浏览器错误 |
| --- | --- | --- | --- | --- | --- |
| 双人 | watchers / responders | 第 10 回合，11–7，目标 8 | 542 | 545 | 0 / 0 |
| 四人 2V2 | watchers / hunters / keepers / responders | 第 12 回合，团队 0 为 3、团队 1 为 11，目标 10；个人 3/0/3/8 | 1868 | 1873 | 0 / 0 |

两局各有 1 次故意断网产生的 `ERR_INTERNET_DISCONNECTED`，单列为预期恢复事件。四人局实际完成 7 次伤害选择、6 次调查、6 次回牌排序、5 次触发、7 次弃牌、4 次检索、4 次事务、3 次现身和 10 次先手特权。双人没有主动能力发动，四人唯一一次发动为 JC003；不能称本轮完整 UI 局直接覆盖了 LC19 治疗费用。自有 8099 PID 50772 已随成功退出停止；本代理没有操作共享 8090/8091/8098 或父代理的 8101 预览。

响应专项与完整局分别计数：

| 专项 | 初局来源 | API 准备命令 | 实际 UI 命令 | 去重 API 请求 | 结果 |
| --- | --- | --- | --- | --- | --- |
| 双人首次有界准备 | 正常服务随机创建，无 seed 覆盖 | 753 | 0 | 0 | 完成第 15 回合仍未形成主链，第 16 回合起始停止 |
| 四人响应 | 正常服务随机创建，无 seed 覆盖 | 1652 | 12 | 1 | 第 12 回合形成主链并通过 |
| 双人定向补验 | 测试专用固定 seed 空大厅，重放历史成功准备序列 | 364 | 7 | 1 | 全部准备引用与版本匹配，真实响应 UI 通过 |

首次双人准备失败证据保留：双方均保有主链牌，但准备策略为缺少的红色忠诚预留第三个资产槽；一方只有两个资产，另一方等待对手有三项资源才派遣末日信徒，造成策略阻塞。游戏仍可通过合法建立第三项中立资产等行动推进；这里没有把失败称为引擎死局，也没有随机长跑或覆盖失败数据。原始 `response-summary.json` 保持失败；`response-v2.1-summary.json` 只汇总这次随机双人准备失败和随机四人通过，不含后来的固定 seed 补验。

定向补验由父端明确授权测试固定 seed。新增 `response_fixed_seed.py` / `wasm_new_lobby.mjs` 仅使用生产 WASM `Game::new` 生成一个 host 的空大厅。WASM SHA256 为 `a6366ef6bc7df03ba666b33286d3a01c412ac6bf779d6396fcbf6affcdc43966`。脚本先正常 HTTP 创建新室，停止自有进程，断言版本 0、仅 host、空手牌/牌库/资产/场上、零 commands/journal，再于同一 SQLite 事务替换该新室的初始和当前空大厅，仅 seed/random 改变，真实 host 凭证保留。权威 `.state` 在 JavaScript 中保持不透明字符串，不经解析重串行；随后重启自有服务，通过合法 HTTP 加入及执行历史成功双人局的 364 条准备命令，每条检查当前合法动作或选择 ID、前后版本及 HTTP 200。没有替换中间局面、补牌或改生产洗牌规则。摘要明确 `testOnlyFixedSeedLobbyBootstrap=true`、`seedOverride=true`、`intermediateStateInjection=false`、`countsAsCompleteUiGame=false`；固定 seed 私有文件及历史 trace 来源、摘要、逐步校验记录均保留本机。

通过的双/四响应均真实点击谋杀、末日信徒牺牲响应与全部玩家让过，验证旧目标离场可见、每轮全席让过仅结算一个栈顶、谋杀取消且三资产费用不退、事务入墓。真实快速行动可用时启用自动让过仍不吞响应，下层只有让过时可自动推进。声明前与已支付响应中分别刷新、重启同一数据库，独立浏览器接受视图一致，重复命令不重付费用。390/1366 视口的响应提示和快速按钮可达，无横向页面溢出；非预期浏览器错误为 0，主动停止自有 SSE 服务造成的预期截断错误分别为双人 4 次、四人 8 次。8100 的每个自有子进程均已停止。

父代理用最终原生 audit 对以下房间分别只读审计一次，全部 `matches=true`；本代理没有代做或重复审计。journal 条数包含加入，不能作为 UI 命令数。随机双人准备房间的重放一致只证明保存/重放，不能证明响应 UI 通过：

| 房间用途 | 房间 ID | 版本 / journal | 持久化与重放共同 SHA256 |
| --- | --- | --- | --- |
| 完整 UI 双人 | 73c0fa78cc67e4c827633047 | 546 / 546 | e5bd8cec4323a57e310fbb33212a84c4c90a2122f637a698acf3b03ff11b0fe4 |
| 完整 UI 四人 | da677bd1327c6e8e844bd761 | 1876 / 1876 | b9a7c19d29588b287546170fedaffd24d18ef456461294f1cfb15b66f5628b40 |
| 随机双人准备停止 | e1b481876b9b5d15f1020876 | 754 / 754 | f8341e4ca186d285c7c213ed7a630bdceeea74c22cf47f60a5a602aaaa743e2c |
| 随机四人响应 | 3c8b2d5970ccd1ff08f2e174 | 1667 / 1667 | 430033db63181d8b73d7fa5658fd1eeb22d24fcbd615df61f4ac1179a19ebeff |
| 固定 seed 双人响应 | 244e2793513b55bd4b515f3e | 372 / 372 | 9e74f35c600c5627696e533cc3fe26298f47b907dec0248e872539a4d6950119 |

旧 `8a16cf3` 完整 UI 两局的 LC19 秘密派遣和发动次数均为 0；唯一主动能力为双人 JC003。秘密派遣的零计数由旧源码 `public=true`、拒绝公开牌秘密派遣的校验、全部命令接受和既有精确重放共同证实；脱敏 trace 本身不保留暗牌身份，不能声称从暗牌实例直接读出了定义。说明与计数见新完整 UI 目录的 `historical-lc19-summary.json`。旧 v1/v2 输出均未删除或覆盖。

最终本机证据：

- `/tmp/hegemony-cloud-playtest-v2.1-2026-10-02/`：二进制/服务身份、双/四摘要、动作 trace、桌面/手机截图、数据库及 `root-readonly-audit.jsonl`。
- `/tmp/hegemony-response-v2.1-2026-10-02/`：原始双人有界失败、诊断和 journal；`teams/` 保存随机四人响应摘要、API/UI 分别记录、手机快速响应及原目标失效截图、数据库和独立 audit。
- `/tmp/hegemony-response-v2.1-fixedseed-2026-10-02/`：定向双人摘要、`prefix-validation.jsonl`、API/UI trace、私有固定 seed 来源、固定 WASM 副本、手机快速响应及原目标失效截图、数据库和独立 audit。

本轮没有重新打包或更新 Library。浏览器使用云端本机 HTTP；没有已发布互联网 URL，因此这些结果仍不证明互联网部署或真人新手验收完成。
