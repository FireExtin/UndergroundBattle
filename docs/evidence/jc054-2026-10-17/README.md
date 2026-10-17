# JC054 完整批次审查证据

源码点 `29efe87644120505dee15f1287db206e8307962c`，Rust编译输入点 `bc9d5bd652a592036002ca5caf74031942b668b0`，分支 `codex/stage-pass-progress-20261016`。完整范围、逐轮失败和线上边界见 [串行独审交接](../../JC054_SERIAL_REVIEW_HANDOFF_2026-10-17.md)。实现者的本地验证不替代父会话安排的独立复审。

- [validation.json](validation.json)：最终适用范围及各轮独立计数；专项／整轮／功能开关／历史检查不相加。
- [source-closure.json](source-closure.json)：127项实际编译输入与当前源码、核心Git提交逐项一致；64个改动的非文档文件与最终代码提交相同；原始Native JSON、压缩测试输入及原图／PDF哈希。
- [runtime-manifest.json](runtime-manifest.json)：最终162个客户端文件、index.js和5个WASM，共168个文件；生产真实时钟后端与本地固定时钟测试后端分别标明。只保存清单，不把编译运行文件加入Git。
- [log-index.json](log-index.json)：外部根目录全部既有日志及前端JSON报告的原文SHA；15份必要最终或失败日志在 `logs/*.gz` 无损压缩保存，解压内容与原文逐字节一致。未归档的大日志仍留在外部，不覆盖早期失败。
- [暂停排序报告](browser-pause-final.json) 与 [双席排序报告](browser-order-final.json)：最终Chrome151四生产GameApp上下文，分别16／23条连续命令、各两次Worker重开；真实本地HTTP、WASM及持久D1。起始布局来自明确的Native合成输入，时钟0；不是线上或自然完整对局。
- [原图阅读截图](jc054-original-scan.png)、[同名强制排序截图](mandatory-order-final.png)、[暂停流程排序截图](pause-order-final.png)：原始PNG未编辑，均已实际查看。完整四席截图保留在外部。

原始材料：`/workspace/jc054-evidence-20261017/`。最终运行文件：其下 `runtime-final/`。保留原核：`/workspace/undergroundbattle-build/frozen-kernels/abi77..80/`。不要将早期 `runtime-candidate/` 或含历史多余资产的 `sites/dist/client/` 当作最终客户端。

合成Native测试输入在 `rust-game-wasm/tests/jc054Native81.fixture.json.gz`，SHA256 `101280b4140675e3557fab3ac829621c959e2eb5255326f9f43cc1ec465f51f7`；其内部原始JSON为22878098字节、SHA256 `2f0b17b48a553fd210ad3361ef2ab3280e86ee989bf8c46b164ad60d544ba28c`，未改写opaque状态或换标版本。历史Native80输入保持原样，不按127项新源码重新签名。

复现入口依次为：现有构建余量守卫 `tools/build-storage.mjs`、Cargo默认和 `society-fixtures` 配置、`rust-game-wasm/build.sh`、`rust-game-wasm/tests/jc054-compare.mjs`、前端test及build、`sites/scripts/build-backend-test.mjs`、`sites/scripts/build-fixed-clock-test.mjs`、串行 `sites/test/*.test.mjs`、`tools/frontend/jc054-http-four-clients-qa.mjs`。浏览器第二场使用 `JC054_BROWSER_SCENARIO=mandatory-order`，最终客户端用 `JC054_CLIENT_DIRECTORY=/workspace/jc054-evidence-20261017/runtime-final/client`。复审不应重启仍在运行的命令、重建旧核、清理保护缓存或省略构建余量检查。

所有编译物、本地D1、缓存和大输入留在Git外；没有真实用户桌、凭据、main合并或候选发布。公开站仍v62，正常TLS访问被证书阻断，故本批没有线上试玩验收。
