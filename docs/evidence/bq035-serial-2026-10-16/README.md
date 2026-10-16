# BQ035 串行收口证据

本目录是离线候选复审材料，源码说明见 `../../BQ035_SERIAL_COMPLETION_2026-10-16.md`，复审入口见 `../../BQ035_SERIAL_REVIEW_HANDOFF_2026-10-16.md`。最新结果在 `verification.json`；旧 `checkpoint-inputs.json`、`wasm-http-checkpoint.json` 是当时的 WIP 回执，不能代替最终结果。

- Native完整workspace：lib1128、integration67，均通过；来源44、输入／保留内核10通过。
- BQ035原文：仓库 `rust-game-wasm/tests/bq035Native78.fixture.json.gz` 内含11正例和54负例原始JSON，源码121文件绑定。326命令、1336投影和432恢复入口完整比较通过。
- 旧机制当前78：433情景、24403转换、96975投影，完整Native/WASM比较通过。`current78-old-mechanisms-release.json` 是含逐case字节长度和哈希的原生清单；完整case原文在 `/workspace/bq035-evidence-20261016/current78-old-mechanisms-release.json.cases/`，不把数GB生成数据或缓存提交Git。清单单独缺少case时比较器必须失败。
- 前端最终：`web78-final2.log.gz`，130文件1227通过；对应584输入在 `web78-final2-inputs-before.json`／`after.json` 验证未变。`web78-full`、`web78-final` 仍分别是1225/2、1226/1失败的原文，不覆盖。
- Worker：`sites78-full.log.gz` 首轮198通过／4失败；`sites78-seven-card-rerun.log.gz` 修正全局78／牌桌77断言后完整4链路通过。其余198未重复，整组202未重跑，不能改称单次202/202。
- 浏览器：`browser-final/` 两种视口四张原图截图和报告，均已实际查看；前两轮脚本失败保存在 `browser-first/`、`browser-second/`。所有这些都是本地投影检查，不是线上试玩。
- 身份：ABI78 `94e5b59743c5836a1b422c38cde553bacda21d1b57c45bbdb83762dc5daea745`，保留ABI77 `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c`，Worker `a85f09fd9f94cd311bfa5ba5f61d76a39b76327f7744ed6d751877eb3e1a80f5`。编译输出不提交Git；原77输入在Git外精确保留，缺失不得重建替代。

`.log.gz` 保留原始日志字节，raw SHA在 `verification.json`／各历史检查点中；`gzip -cd 路径.log.gz` 可读取。最终目录内容哈希见 `evidence-file-hashes.json`（不自包含该清单）。源码提交、有界普通目录副本和最终字节核对见 `source-closure.json`。有界副本不是Git worktree，不包含源档案PDF、数据库、凭据或缓存；它核对发布所需源输入和ABI，不声称在省略原始档案的副本上跑过完整档案／前端来源测试。

复现主命令均先 `. /workspace/tooling/env.sh`（Rust命令）并经 `node tools/build-storage.mjs -- ...` 使用磁盘、单Cargo编译作业：

1. `cargo test --locked --workspace -- --test-threads=1`，分别设置新空的 `BQ035_NATIVE_DIR`／`BQ035_INVALID_DIR`。
2. `bash rust-game-wasm/build.sh`；`node tools/bq035-oracle.mjs pack 正例目录 负例目录 新输出.gz`。
3. `cargo run --locked --release -p hegemony-wasm --example native_fixtures -- 新清单.json`，随后 `node --expose-gc rust-game-wasm/tests/compare.mjs 新清单.json`，明确选择当前pkg。
4. `npm --prefix sites run build`（wrangler dry-run）；`node sites/scripts/build-bq035-test.mjs` 创建仅时钟固定的本地测试入口。
5. `npm --prefix web test -- --maxWorkers=2 --reporter=verbose`；在sites目录执行 `node --test --test-concurrency=1 test/*.test.mjs`。这条Worker首轮的四项失败及单文件复跑如实保留。
6. `node tools/frontend/bq035-native78-qa.mjs 新证据目录`；不连接公开站、真实房间或邀请码。

本批没有生产发布、线上数据库操作、main合并或并行开发。v59的成功发布及证书检查另有独立材料，本批没有重复这些操作。独立复审及适用发布验收由父会话串行安排。
