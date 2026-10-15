# JC060 / ABI75 可复核材料

源码基线为 `3d642f0c1e6e5ac021cd40560086df35638bfe4c`；实际候选身份、原图与行为范围见
[`JC060_GRAY_OCCULT_ADVISER_REVIEW_2026-10-15.md`](../../JC060_GRAY_OCCULT_ADVISER_REVIEW_2026-10-15.md)。
独立审查见同目录上级的 JC060_SOURCE_REVIEW 与 JC060_INDEPENDENT_REVIEW。
本文不构成部署或线上浏览器验收。

## 复现顺序

使用 Rust 1.90.0、wasm-bindgen 0.2.104、Node 24.19.0 与仓库锁文件。
Cargo 经 `node tools/build-storage.mjs -- …` 启动并串行运行；磁盘 target 与临时目录由守卫指定。
初次取得本候选源码后，可在独立的新证据目录执行：

1. `JC060_NATIVE_DIR=<新目录> node tools/build-storage.mjs --output-dir <新目录> -- cargo test --locked -p hegemony-server -- --test-threads=1`。
2. `node tools/build-storage.mjs --output-dir rust-game-wasm/pkg -- bash rust-game-wasm/build.sh`。
3. 使用本次已提交的当前75输入运行 `node rust-game-wasm/tests/jc060_compare.mjs`。脚本严格比较当前核身份和WASM字节；重建哈希若不同，应重新导出到新的证据/输入目标，不能改写历史输入中的状态或哈希来冒充同一产物。
4. `HEGEMONY_BACKEND_TEST_ROOT=<新目录> node tools/build-storage.mjs --output-dir <新目录> -- node sites/scripts/build-backend-test.mjs`；这是本地 `--dry-run` 打包。
5. `npm --prefix web test -- --maxWorkers=1`，`npm --prefix web run build`，以及在 `sites/` 下执行 `node --test --test-concurrency=1 test/*.test.mjs`。

本轮35步统一当前输入生成使用 `tools/fixture-tools/regenerate_native72_inputs.py`，在目标尚不存在的工作区一次生成。
已提交输入受防覆盖检查保护，不能在原位置再次生成；需要重导出时先为新证据选择新的明确目标。
JC060专属11条轨迹由 `tools/fixture-tools/jc060-current.py` 汇总，始于明示的离线布局，之后均为真实 Room 命令。

## 记录边界

- `native-full.log` 是最终单次完整通过轮；首轮定向测试的黑猫断言错误与修正原因已在批次报告披露，原始日志保留在本机证据目录。
- `jc060-wasm-parity.log` 比较完整结果、opaque state 和全部席位投影；`wasm-binding.json` 记录本次三个本地产物的相同字节。
- `current-inputs.json` 记录新75输入大小及SHA256；历史原始资料和输入1131份逐字节未变。
- `review-fingerprints.json` 核对独立审查所列33份源文件；审查者没有参与实现，也没有把源码审查当运行测试。
- 所有D1/席位凭据均为本地测试数据；数据库、凭据、WASM构建文件、依赖缓存未进入候选提交。
- 并行组卡/选牌和单桌恢复分支尚未整合，`currentDomains` 界面展示与成功部署后的真实试玩仍由后续整合处理。
