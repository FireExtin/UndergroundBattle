# JC066 证据索引（WIP，待父会话独审）

准确候选源码：`804f00fce94c2cc7e182119c9bc2f1049cb27105`；分支 `codex/jc044-time-repress-review-20261018`。此目录仅记录无损日志、合成测试状态摘要、源码/产物哈希、公开牌面与合成浏览器截图，没有数据库、编译产物、缓存、真实私人数据或凭据。原始材料位于 `/workspace/jc066-evidence-20261018`。

- `validation-summary.json`：单次完整Native1313、单次完整前端1342、新JC066100HTTP/D1，以及Worker首轮294/12、修后JZ46全文件15/15的精确边界；整306未重跑，不跨轮累加。
- `source-binding-git.json`：135编译输入逐项与候选Git blob/工作树相等，完整Native与专项导出逐字节相同。
- `runtime-manifest.json`：181生产文件、20generated文件，当前86及原77–85共十份准确WASM；生产真实Date.now、隔离测试固定时钟；534个原v67保存文件不变。
- `sources.json`、`catalog-delta.json`、`baseline85.json`：原图/两份官方PDF输入、仅追加JC066与旧149定义/预组/世界保护、原v67身份。
- `runs/*.exit.json`、`runs/*.log.gz`：30个已自然结束的运行原回执与无损日志，失败轮次也保留。`gzip -cd`可直接阅读，无虚构日志或手填退出状态。
- 三个`browser-*`目录：真实生产GameApp/HTTP/workerd/WASM/合成D1的四独立浏览器上下文，原收据与状态/日志/四席视图；只选择必要原始截图。actor-1使用26972b2时的驱动，后两例的附加只读检查使用a1b05315…驱动；实际生产字节三例一致，摘要给出准确驱动源码提交及SHA。
- `official-v67-09.json`与`v67-browser09/normal-browser.json`：09:22官方仍v67/public revision2、原部署succeeded；09:22:29Z一次普通Chromium仍证书阻断，1次导航0写入0线上游戏，不绕过。本材料没有10点小时新访问结果。
- `scripts/`：按原执行环境记录的组装与审计脚本。输出使用新文件，不覆盖原件；审计前曾适配旧清单list格式和旧核大小元数据字段，生产字节未修改。

## 真实验证命令

所有构建/测试都经过 `node tools/build-storage.mjs -- ...`，不绕过磁盘/内存检查，不清理或使用tmpfs回退。工具链Rust1.90/wasm-bindgen0.2.104；环境为：

```sh
export PATH=/workspace/tooling/cargo/bin:$PATH
export CARGO_HOME=/workspace/tooling/cargo
export RUSTUP_HOME=/workspace/tooling/rustup
export WASM_BINDGEN=/workspace/tooling/wasm-bindgen-0.2.104-x86_64-unknown-linux-musl/wasm-bindgen
```

最终Native、当前WASM及新见证的执行记录见对应exit文件。独立复验必须使用新的输出目录/文件；原77–85载体位于 `/workspace/undergroundbattle-build/frozen-kernels/abiNN`，缺失或hash不符时必须停止，不以重建替代：

```sh
node tools/build-storage.mjs -- env JC066_NATIVE_DIR=/workspace/jc066-new-native cargo test --locked --workspace -- --nocapture
node tools/build-storage.mjs --output-dir rust-game-wasm/pkg -- bash rust-game-wasm/build.sh
node tools/build-storage.mjs -- node tools/jc066-oracle.mjs /workspace/jc066-new-native/native86.json rust-game-wasm/pkg/hegemony_wasm_bg.wasm /workspace/jc066-new.fixture.json.gz
node tools/build-storage.mjs -- npm --prefix web test -- --maxWorkers=1
node tools/build-storage.mjs -- node sites/scripts/build.mjs
node tools/build-storage.mjs -- env HEGEMONY_FIXED_CLOCK_TEST_ROOT=/workspace/jc066-new-worker node sites/scripts/build-fixed-clock-test.mjs
```

`worker-batch-first.exit.json`保留306项原始完整文件列表。只修全局helper引用后实际复验：

```sh
node tools/build-storage.mjs -- env HEGEMONY_CURRENT_WORKER_DIR=/workspace/jc066-evidence-20261018/worker86/server bash -c 'cd sites && exec node --test --test-concurrency=1 test/jz46-integration.test.mjs'
```

三项本地四浏览器场景使用 `JC066_BROWSER_SCENARIO=actor-1`、`entry-reveal-accept`、`shared-hand-seal-character`，驱动 `tools/frontend/jc066-http-four-clients-qa.mjs`要求新证据目录。初始布局及可信本地时钟是明确的合成输入；之后均实际UI/HTTP/内核/D1，没有API模拟或不透明房间状态JS重写。这些是本地验证，不是线上试玩或远端Cloudflare D1验收。

## 发布与审查边界

完整两项卡文一起实现，新增单灰角色；不升级档案selectionEnabled/gameAcceptance/strategyUiPlaytest。自封印允许并立即返owner手牌是一般规则应用推导，须独审确认。父会话按精确源码/产物与分轮测试边界串行安排独审，本线没有启动复审子代理、合main、触发发布或部署exchange。旧核历史源重建身份问题不在本批宣称解决。
