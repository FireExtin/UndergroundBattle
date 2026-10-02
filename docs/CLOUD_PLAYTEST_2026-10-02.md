# 云端 Chromium 试玩与验收

脚本位于 `tools/cloud-playtest/`，仅使用云端 Python Playwright 和 `/usr/bin/chromium`，无需 Mac 或隧道。规则边界遵循 [云端切片接口与已核对规则](HEGEMONY_CLOUD_SLICE_2026-10-02.md)。这些验收只检验公开接口和玩家操作流程，不替代逐卡规则测试。

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

## 本次结果

双人 UI 对局已通过：两个独立 Chromium 上下文，387 个游戏循环动作，390 个已接受命令，零拒绝命令；第 7 回合以 9–6 达到 8 分目标结束。实际选择覆盖再调度、伤害分配、调查、弃牌、地区回牌自定顺序、触发、检索和目标。无非预期浏览器错误；断线探针的 `ERR_INTERNET_DISCONNECTED` 单列为预期事件。

四人 UI 对局已通过：四个独立上下文、四套不同预组，2521 个游戏循环动作，2526 个已接受命令，零拒绝；第 16 回合最终团队分数 8–11，胜利目标 10。实际执行伤害、调查、地区回牌排序、检索、触发、先手特权与事务，无非预期浏览器错误。

修正后 Rust 构建已重验双人 UI 局：第 6 回合 8–3 得分胜利，313 个游戏循环动作，316 个已接受命令，零拒绝和非预期浏览器错误。修正后四人局也完成得分胜利：第 18 回合团队 0 为 7 分、团队 1 为 10 分，2913 个游戏循环动作，2918 个已接受命令，零拒绝和非预期浏览器错误；四席独立上下文。实际选择包含 11 次伤害、4 次调查、4 次弃牌、1 次奖励执行者、14 次自定回牌顺序、19 次目标及 9 次触发。修正后独立 HTTP 八项验收、浏览器进程重启恢复和可选自动让过真实 UI 探针均通过。8091 当时为后端代理持有的最终服务，隔离恢复探针改用自有 8093/8094 端口。

新手可见 DOM 短流程通过：99 步、104 个已接受 UI 命令、四个独立视角，以上各项交互检查通过，浏览器错误为空。最初的字符串断言把“请完成下方选择”误判为不明确，首次失败输出另行保留；复核改为检查实际本人选择面板和可达操作，界面同时增强为“请你完成下方选择”。手机阅读的独立最新构建检查也通过：两台 390px 视口的窗口宽 362px、正文 16px，Tab 焦点留在窗口内，Escape 可关闭，浏览器错误为空。这里的浏览器全部运行在云端，所用入口仍为本机 HTTP；尚未获得已发布互联网 URL，不能据此宣称互联网部署或真人新手验收完成。

最终两处微调已局部复核：独立夹具接受 561 个正常 API 命令后进入实际伤害待选，桌面与手机都显示公开拥有者和目标 1；手机操作面板计算背景为不透明的 `rgb(36, 44, 50)`，没有透出底下文字，两个视口均无横向页面溢出，浏览器错误为空。源脚本后续还增强暗牌观察计数，必须实际观察本人、队友和对手关系才能标记隐私检查通过；原 99 步证据保持原样，未为这个统计增强再次跑整轮。

当前证据（本机私有，不入 Git）：

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
