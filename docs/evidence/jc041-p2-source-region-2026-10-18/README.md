# JC041 原地区UID恢复P2证据（待父复审）

基线f3dece927c733a761cf78d62514c6299fab280fa；Rust修复6b5b824b1e3fd08edfb1b47f7d979ef1ffe69317；独立分支codex/jc041-p2-source-region-review-20261018。[复审说明](../../JC041_P2_SOURCE_REGION_REVIEW_2026-10-18.md)、[验证摘要](validation-summary.json)、[137编译输入/准确实物及预期Git blob](artifacts-after.json)为最终范围。wip-status保留第一次检查点，不代表最终总结果。

修前Native1项、准确980e WASM/HTTP8项明确四坏状态被接受并实际写入；修后Native14、相关前端2文件77、集中P2/兼容29均通过；常规10/155/9/660/35完整Native/WASM对照通过。四坏字符串、四组56条合法续行完整轨迹逐字节相同，修后40 HTTP400、实际D1重开无额外room/commands/journal写入。

before/after.native-p2.json.gz均为Rust类型化输出，JS不改不透明Room；原始输入固定31,476B/SHA ad127d41b99f0bcae5cb8fe8c1488775fad1ef1c0407140fc30d6ccd2f83850c。运行时测试输入独立绑定，不称额外编译输入。原始大件、原pkg/生产server/Worker、所有合成DB在/workspace/jc041-p2-region-evidence-20261018保留。小时普通TLS记录只Git外，不造重复提交。

## 当前修复复现

先使用现有toolchain与77–86原冻结输入，选择全新输出目录；原普通磁盘target/tmp、jobs1和1GiB启动守卫保留。缺旧核应停止，不能替代构建或重签。

```bash
node tools/build-storage.mjs -- env JC041_P2_DIR=/workspace/jc041-p2-replay/p2 JC041_NATIVE_DIR=/workspace/jc041-p2-replay/native cargo test -p hegemony-server --lib jc041 --locked --offline -- --test-threads=1
node tools/build-storage.mjs -- env CARGO_NET_OFFLINE=true bash rust-game-wasm/build.sh
node tools/build-storage.mjs -- node tools/jc041-oracle.mjs /workspace/jc041-p2-replay/native/Native87.json rust-game-wasm/pkg/hegemony_wasm_bg.wasm /workspace/jc041-p2-replay/fresh.fixture.json.gz
node tools/build-storage.mjs -- node sites/scripts/build.mjs
node tools/build-storage.mjs --output-dir /workspace/undergroundbattle-build/jc041-p2-replay/server -- node tools/build-jc041-p2-clock-worker.mjs /workspace/undergroundbattle-build/jc041-p2-replay/server
node tools/build-storage.mjs -- env JC041_P2_CASES_FILE=/workspace/jc041-p2-replay/p2/p2.json JC041_P2_WASM_DIR=/workspace/UndergroundBattle/rust-game-wasm/pkg HEGEMONY_CURRENT_WORKER_DIR=/workspace/undergroundbattle-build/jc041-p2-replay/server bash -c 'cd sites && exec node --test --test-concurrency=1 probes/jc041-p2-source-region.mjs test/jc041-kernel-routing.test.mjs test/jc041-retained-http.test.mjs test/kernel-binding.test.mjs ../tools/cards/jc041-source.test.mjs ../tools/cards/jc066-source.test.mjs ../tools/frozen-abi86.test.mjs'
node tools/build-storage.mjs -- npm --prefix web test -- --maxWorkers=1 src/game/JC041Native87.test.jsx src/game/JC066Native86.test.jsx
```

本地时钟包装只编译相同kernel/http源与可信时钟入口，11个WASM均为普通磁盘生产原件的新硬链接；没有重复存35MB、删除旧文件或回退tmpfs，未修改生产时钟。全部14文件及同inode/大小/SHA在实物审计绑定，生产183文件与168客户端完整核对。该包装仅本地测试，不发布。

## 修前原件重放

原8项自然运行已保留。当前分支也能用保存的旧pkg/Worker重放：把before.native-p2.json.gz无损解压到一个新文件（保留gzip原件），令JC041_P2_CASES_FILE指向该文件、JC041_P2_WASM_DIR指向/workspace/jc041-p2-region-evidence-20261018/before-pkg、HEGEMONY_CURRENT_WORKER_DIR指向/workspace/jc041-p2-region-evidence-20261018/before-worker/server，再从sites执行显式probes/jc041-p2-source-region.mjs。探针核对f3产品和6b测试专用源码的不可变Git blob，不拿修后工作树给旧见证重签；旧核必须准确980e。新入口仅复跑source-self烟测1项通过，未把1叠加原8或冒称整8重跑。

专用探针位于probes且非*.test.mjs，默认npm test的test/*.test.mjs不加载它；固定输入只在显式探针使用。Native新测试默认检查拒绝，原31KB运行时输入随测试代码提交，不依赖额外环境配置。所有56合法续行之后的操作都为真实Room命令；初始移动/原地区消失/vacant是明确Native布局边界，不冒充可玩的换地区动作，也不是线上游戏。

本轮未扩大重跑完整workspace1328、全量前端1391、原173或全部59默认Worker。原f3的完整测试仍为修前历史。旧980e、3325夹具、所有77–86原件及d58b Native86继续保留；当前f2c1c2f3和23b52be4夹具只属于未发布87候选。剩余为父复审，未发布/合本批main。
