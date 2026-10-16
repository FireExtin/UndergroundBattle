# WM062 完整本地批次证据（待串行独审，未发布）

代码来源固定为 `codex/stage-pass-progress-20261016@a9161c958dedceb95b70310d897dd5eeb8550e52`，起点为 `27026fd864b96da5950c07ca1f58a8315c27a70e`。当前候选80／pool73／hegemony-pdf-v1，WASM `0229f75b7af79035f2a4025fc11c6de6f3f0b895127cc8b537f829268d030472`，3047680字节。完整行为、审查入口及每轮失败边界见 [串行独审交接](../../WM062_SERIAL_REVIEW_HANDOFF_2026-10-16.md)。

| 文件 | 精确范围 |
| --- | --- |
| [verification-summary.json](verification-summary.json) | 各层实际通过、失败、定向补跑及未执行范围；不将重叠或分次结果相加为“全绿” |
| [source-closure.json](source-closure.json) | 125项Native编译输入、66项变动非文档文件与代码提交逐字节绑定；原145定义、牌组／世界／规则／PDF哈希及原图映射核对 |
| [runtime-manifest.json](runtime-manifest.json) | production dry-run的161客户端与5服务器运行文件；恰好当前80和原79／78／77的WASM，无二进制入Git |
| [native-source-binding.json](native-source-binding.json) | 最终Native第五轮编译内嵌125项来源；原始14正例及45坏档在 `rust-game-wasm/tests/wm062Native80.fixture.json.gz`，保留字节与哈希 |
| [wasm-parity-final.json](wasm-parity-final.json) | 14场景、111命令、490席投影、18次预期拒绝、45坏档540恢复调用；完整opaque状态／席视图／日志／回执／错误对照 |
| [log-index.json](log-index.json)、`logs/` | 全部50份完成的顶层原文日志，含失败、历史版本和最终轮次；gzip mtime0且解压哈希相同 |
| [browser-final-runs.json](browser-final-runs.json)、`browser/` | 最终0229下移动／暂停恢复、放弃、JZ48防御三条四上下文生产浏览器流程；报告及7张选定截图 |
| [published-v61-tls-212835.json](published-v61-tls-212835.json) | 21:28:35 UTC单次普通浏览器访问，证书错误、0POST、0建／入桌、未达UI及版本身份，未线上试玩 |

Native专项第五轮12/12。完整Native首轮1145/1152、次轮1152/1153，均为数量期望过期；最后一处cfg(test)135→136后，定向catalog1/1、integration七组67/67通过，WASM重建仍逐字节0229；没有完整1153一次全绿的记录。最终前端132文件1256/1256通过；此前默认5秒预算1255/1256的唯一超时保留，15秒预算重跑没有改动产品和断言。

既有Worker首轮213/219，6项旧数量／版本期望失败；三个文件定向1/1、1/1、4/4通过，没有完整219一次全绿的记录。WM062独立HTTP／D1最终20/20；这20项不属于前述219的同一次运行。所有D1验证是本地实际workerd和SQLite；未做远程Cloudflare D1验收。

`defense-before-after*` 保留同一合法初始输入在修前ef68 WASM与修后Native／0229 WASM中的实际差异：5条真实Room命令后，旧核在移动待选时留下伤害1防御1的JZ48，新核已清理，所得暗藏者UID相同。没有改写版本头；`fixedWasmOpaqueStateByteIdenticalToNative`指opaque状态字符串，不是二进制等于Native程序。证明中的Native第四轮绑定按原样保留；第五轮仅测试期望变动，重建身份检查和最终fixture另存，不给旧证明换标签。

旧ef68、第三轮Native13场景及与精确95b32536匹配的125输入绑定在Git外 `before-defense-fix80/` 保留。原77／78／79运行依赖在 `/workspace/undergroundbattle-build/frozen-kernels/`，缺失或篡改即失败，不重建或替代。原始大输入、全部四席截图、旧浏览器失败轮次及构建在 `/workspace/wm062-evidence-20261016/` 与 `/workspace/undergroundbattle-build/`。

复核使用现有Rust1.90／wasm-bindgen0.2.104、Node24.19和Chromium151.0.7922.173。按现有 `tools/build-storage.mjs` 磁盘及内存预检运行；不清缓存或绕过门槛。`WM062_NATIVE_DIR`、`WM062_NEGATIVE_DIR`和浏览器证据目录必须使用新路径，导出工具拒绝覆盖。核心对照入口为 `rust-game-wasm/tests/wm062-compare.mjs`；本地四客户端入口为 `tools/frontend/wm062-http-four-clients-qa.mjs`，三条流程用 `WM062_BROWSER_SCENARIO` 指定，依赖已构建的production客户端及独立fixed-clock Worker。

源码、测试与证据只有本地合成场景；没有真实桌、数据库、凭据、缓存或编译运行文件入Git。本批无新开发分支、worktree、开发子任务、Actions、main修改或发布；待父会话串行独立复审。
