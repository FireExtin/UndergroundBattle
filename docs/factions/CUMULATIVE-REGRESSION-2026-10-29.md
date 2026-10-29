# 累积回归与 MSJC16 目录期待修正

基线 `cdcef548048ad238d248c215f8985e27c7ff45ed` 全库实际发现 2792 项：2790 通过、1 失败、1 ignored。失败为 `jc008_tests::jc008_printed_admission_and_finite_modifier_program`，精确秘社列表遗漏 `MSJC16`。目录来源 `society.rs` 显式追加该秘社，`msjc16.rs` 仍为 `supported:false`。

本修订只在该测试期待追加字面量 `MSJC16`，保留规则、负例、准入和生产代码。定向函数 1/1 通过；同一修订源码最终 Native 全库 2791 通过、0 失败、1 ignored，耗时 1871.47 秒。使用 Rust 1.90.0、默认 native 特性、锁定离线依赖、单构建任务与单测试线程。

命令均经 `node tools/build-storage.mjs --`：

- `cargo test --offline --locked -p hegemony-server --lib jc008_printed_admission_and_finite_modifier_program -- --nocapture --test-threads=1`
- `cargo test --offline --locked -p hegemony-server --lib -- --nocapture --test-threads=1`

最终全库显式设置 `SENTINEL_WORKER_WITNESS` 与 `SENTINEL_PLATFORM_WITNESS` 到新的本地证据目录，真实执行见证主体。BQ001 内部候选自然链仍 ignored，未开启准入。

此前 8 个 Native 集成目标 72/72、相关 UI 23 文件 277/277，以及 Native/WASM 3329 命令／13316 视图／3329 重复命令一致，均属于原 `cdcef548` 源码，不能改签为本测试修订提交。本轮不重复无关 Worker 对局；不是浏览器试玩，也没有独审、推送或发布。

原失败日志保留于云端 `/workspace/cumulative-cdcef548/evidence/`；修复后原始日志与恢复包保存于 `/workspace/cumulative-msjc16-assertion-02/`。Library HTTP 401 仍阻塞，未重试或换入口。新候选保持关闭。
