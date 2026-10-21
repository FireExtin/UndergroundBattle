# ABI93 固定源码重建配方（作者执行，待独立审查）

输入是正常授权仓库中干净的 `bb77eb8ad69738ebecad860d60df05a9276efd00`。不要覆盖当前开发克隆，不使用worktree，不读取另一机器文件。脚本拒绝不同提交、脏源目录和已存在或包含源码的输出目录。输出目录位于正常磁盘，原有 `tools/build-storage.mjs` 保持1GiB预检及jobs1；不使用tmpfs、清理或回退。

工具约束：Rust `1.90.0 (1159e78c4 2025-09-14)`，`wasm32-unknown-unknown`；来自锁定官方crate的 `wasm-bindgen-cli 0.2.104`，版本文本必须恰为 `wasm-bindgen 0.2.104`。预编译CLI附加的Git版本字样会改变producers节，不能替代。此次CLI二进制SHA256为 `9e04f8fc32eafed0396f48d897d74ba8ae5081ff4d4f70550b885fb51ae7c47c`。构建CLI使用官方 `cargo install wasm-bindgen-cli --version 0.2.104 --locked --root <新目录>`；下载/构建日志保存在此前重建证据中，不修改全局工具。Node为本机24.22.1，原生链接使用本机Zig0.15.2包装器（仅将cc-rs目标拼写转换为Zig识别的Linux目标）。WASM代码由Rust目标编译。

此次完整实际命令（仓库目录 `UndergroundBattle-batch3`，各路径均为本任务已验证本地工具）：

```sh
ZIG_GLOBAL_CACHE_DIR=/home/ddd/Documents/Codex/2026-10-21/task-3/.tools/zig-cache \
ZIG_LOCAL_CACHE_DIR=/home/ddd/Documents/Codex/2026-10-21/task-3/.tools/zig-cache-local \
python3 tools/reproduce-abi93.py \
  --source ../UndergroundBattle \
  --output ../repro-abi93-executable-recipe \
  --node ../.tools/node-v24.22.1-linux-x64/bin/node \
  --cargo ../.tools/cargo/bin/cargo \
  --bindgen ../.tools/wasm-bindgen-0.2.104-from-crate/bin/wasm-bindgen \
  --cargo-home ../.tools/cargo \
  --rustup-home ../.tools/rustup \
  --cc ../.tools/batch3-zig-cc
```

Cargo的代理可执行路径保留`cargo`文件名，不解析到rustup本体。脚本离线、锁定Cargo.lock构建，注册表源码须已经由正常Cargo流程缓存；缺失时如实失败，不更改依赖。Cargo.lock SHA256为 `1745cc1af931f500e763222c39100da3ac0521f4a1f96fc54e82c8589e2edb54`。

显式编译参数是 `--remap-path-prefix=<本机CargoHome绝对路径>=/workspace/audit-underground-100161b/cargo`。右侧只是原ABI93产物包含的路径字符串，不访问此目录。原13处绝对路径经编译器统一，未直接改动WASM任何节。release保持原存储脚本的codegen-units256设置。原生raw WASM SHA256为 `f186f9ca6052392b455076e14d4e8b1840188f7db09108b7d3d0c542cb0f613d`。

| 输出 | 字节 | SHA256 |
|---|---:|---|
| hegemony_wasm_bg.wasm | 4315979 | `098986aff958d97b5fb5f49339daf07586a200997bf49c179f424ea51dd04e7e` |
| hegemony_wasm.js | 18203 | `fb262978c41802322caec9fa9dd38ff6e273ff019865fd401a82afd78c1cfebb` |
| hegemony_wasm.d.ts | 3921 | `c957ff06db12352cfc9ebb92a45f1707432e53972efcbee7df24c8b9c211a758` |
| hegemony_wasm_bg.wasm.d.ts | 1944 | `5ebdfb8ff652624f96dfba3d1dcecf1ec3db65c65846a3579ad1aa94372dbdf3` |
| package.json | 33 | `89c47db7851f89caec72da4f215ebc21cf1e294aeef3b45d2583562d846ab035` |

执行结果和完整构建日志位于[证据目录](../evidence/batch20-third-2026-10-21/wasm-abi93-executable-recipe-result.json)。五个结果与原ABI93逐字节一致；旧二进制不作为生成输入，原候选目录不变。该结论只覆盖固定ABI93配方；未修改未来ABI94默认build.sh，也未声称独立复现审查已通过。
