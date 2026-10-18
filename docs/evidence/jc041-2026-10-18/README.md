# JC041 完整批次证据（WIP，待父会话独审）

源码检查点 `a84cc55a8c40359a297ca2c4e47fa33d0e957385`，分支 `codex/jc041-blue-hidden-water-review-20261018`，起点 `2f141abaf7b2b728a4d5398f17a06acb7ffb021a`。最终完整Native1327、前端第二轮140文件1391、集中9文件Worker173均在各自单一自然命令中通过；137输入Native/WASM10场景155命令9拒绝660视图35坏档及三组四浏览器通过。不是全部59默认Worker文件回归。未独审、未发布JC041，当前公开站最后核对仍v68/ABI86。

[完整说明](../../JC041_COMPLETE_BATCH_REVIEW_2026-10-18.md)、[最终验证摘要](validation-summary.json)、[137输入与Git blob](source-binding-git.json)、[183生产文件/11核](runtime-manifest-paid-final.json)、[费用修正及浏览器准确产物](closed-payment-evidence.json)组成当前交接。`wip-status.json`与三个checkpoint保留当时范围；WIP时间线逐轮保留早期失败、修前Native1327、前端1389/2超时与Worker首轮64/107，不跨轮合并结果。`runs/`为原自然exit JSON及无损gzip日志；最初冻结harness失败和main首次postcheck各自明确只有工具输出，不伪造自然日志。

原图、完整raw/旧pkg/前后生产及测试包装、所有合成SQLite D1在 `/workspace/jc041-evidence-20261018` 及共同普通磁盘build root保留，Git只保存必要源码、合成夹具、日志和哈希。三张最终截图已实际查看。12/13点普通线上证书记录仅Git外，不为相同阻断另造提交；本地四浏览器不是线上试玩。

## 本地复现顺序

须使用已审查分支的准确源码和现有toolchain，并提供77–86的原冻结输入。ABI86目录默认 `/workspace/undergroundbattle-build/frozen-kernels/abi86`，亦可用`HEGEMONY_FROZEN_ABI86_DIR`指定；缺失/篡改会停止，不能替代构建或重签旧见证。所有编译/测试沿共同普通磁盘target/tmp、jobs1与1GiB启动守卫，不清理或回退tmpfs。下面导出与证据目录必须新建，原件不覆盖。

```bash
export PATH=/workspace/tooling/cargo/bin:$PATH
export CARGO_HOME=/workspace/tooling/cargo
export RUSTUP_HOME=/workspace/tooling/rustup
export WASM_BINDGEN=/workspace/tooling/wasm-bindgen-0.2.104-x86_64-unknown-linux-musl/wasm-bindgen
node tools/build-storage.mjs -- env JC041_NATIVE_DIR=/workspace/jc041-review-replay cargo test --workspace --locked --offline -- --test-threads=1
node tools/build-storage.mjs -- env CARGO_NET_OFFLINE=true bash rust-game-wasm/build.sh
node tools/build-storage.mjs -- node tools/jc041-oracle.mjs /workspace/jc041-review-replay/Native87.json rust-game-wasm/pkg/hegemony_wasm_bg.wasm /workspace/jc041-review-replay/fresh.fixture.json.gz
node tools/build-storage.mjs -- node sites/scripts/build.mjs
```

`build-inputs/clock-worker/`仅保存本轮三个原始测试包装源码/配置输入，没有编译产物或DB。复制到共同build root下的新目录，保留它与`/workspace/UndergroundBattle`的相对位置，才执行dry-run；生产入口始终使用Date.now。以下均为本地复现，发布由父会话独审后另行处理。

```bash
test ! -e /workspace/undergroundbattle-build/jc041-clock-replay
mkdir /workspace/undergroundbattle-build/jc041-clock-replay
cp docs/evidence/jc041-2026-10-18/build-inputs/clock-worker/* /workspace/undergroundbattle-build/jc041-clock-replay/
node tools/build-storage.mjs --output-dir /workspace/undergroundbattle-build/jc041-clock-replay/server -- env WRANGLER_SEND_METRICS=false XDG_CONFIG_HOME=/workspace/undergroundbattle-build/jc041-clock-replay/config bash -c 'cd /workspace/undergroundbattle-build/jc041-clock-replay && exec /workspace/UndergroundBattle/sites/node_modules/.bin/wrangler deploy --dry-run --outdir server --config wrangler.json'
node tools/build-storage.mjs -- env HEGEMONY_CURRENT_WORKER_DIR=/workspace/undergroundbattle-build/jc041-clock-replay/server bash -c 'cd sites && exec node --test --test-concurrency=1 test/jc041-*.test.mjs test/jc066-integration.test.mjs test/kernel-binding.test.mjs ../tools/cards/jc041-source.test.mjs ../tools/cards/jc066-source.test.mjs ../tools/frozen-abi86.test.mjs'
node tools/build-storage.mjs -- npm --prefix web test -- --maxWorkers=1
node tools/build-storage.mjs -- env HEGEMONY_CURRENT_WORKER_DIR=/workspace/undergroundbattle-build/jc041-clock-replay/server JC041_BROWSER_SCENARIO=water-actor-0 node tools/frontend/jc041-http-four-clients-qa.mjs /workspace/jc041-review-replay/browser-water-actor-0
```

另外两组将场景与新输出目录分别设为`water-actor-2`和`ordinary-reveal-land`。每组四独立浏览器、真实监听HTTP/workerd/11-WASM/D1，只有初始Native布局与可信本地时钟是合成输入；预期拒绝的fetch明确单列。不要与在线试玩混同。

最终费用守卫前后10个正常场景的完整不透明状态/日志/视图相同，仅新增paid-14/paid-15两坏档；最终35坏档全部重新核验。当前137-source见证由Rust真实编译导出，JS仅传递不透明字符串；原86 oracle严格固定原135绑定、d58b夹具、ce6f raw与2c9内核。证据manifest绑定本目录全部其他文件。

已审ABI86固定224另依本轮新增用户授权正常快进游戏main，参见[独立记录](../../REVIEWED_ABI86_MAIN_PROMOTION_2026-10-18.md)；该操作未把本批JC041或后续v68回执带入main。
