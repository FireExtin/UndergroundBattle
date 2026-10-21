# 固定 100161b 审查后的本地整改（WIP）

审查基线为 `100161b628e56b9153041d952ac0f5d83428ca88`，产品 v71/ABI89。固定审查报告、5689文件/304目录清单、805普通代码文件逐段覆盖账本与原始失败证据保存在本任务独立审查目录；未把第三方归档、历史补丁、全部资料或浏览器场景称为全量验证。本文件记录后续修改，不替代固定版本报告。

当前候选引擎标识是 `rust-v0.2.90-audit-correctness-candidate`；卡池仍为153普通卡/10秘社，牌面目录与预组没有新增。Sites沿既有版本分派保留ABI77–89，旧房间不改存档、不重新签名或转换到新内核。Native服务仍只有当前原生内核，不能把Sites历史内核路由视为原生数据库迁移支持。

修改内容：合法时间标志与BQ028奖励来源的恢复；跨角色声望/荣耀准入；多组独有冲突连续处理；香港空检索私密确认；三处ID生成复用；旧结束步骤夹具；独立Site源码复制闭包；XQ25附属图标。合并重复的附属图标路径及冻结字节装载；删除重复XQ23校验；纠正过时说明、恢复检查点字段和回放哈希报告；删除没有生产调用的BQ030 preparation原型及其仅自测模块，当前编译见证移除对应文件；冻结ABI77–89原始来源见证不变。删除已无法生成当前166项图集的旧generate-card-scans入口，保留现有卡图注册与validate-admitted-scans校验。三个来源测试改用批准历史对象的固定语义哈希，执行时不再要求Git历史。

## 本地构建与输入

需要Rust工具链 `rust-toolchain.toml`（1.90.0）、wasm32-unknown-unknown、wasm-bindgen 0.2.104，以及web/sites锁定的npm依赖。先在根目录设置自己的独立 `UNDERGROUNDBATTLE_BUILD_ROOT`，按现有 `tools/build-storage.mjs` 运行，避免把缓存放进源码。

冻结输入是必需的外部原始产物，不能用当前源码重编历史ABI。每个ABI在 `$UNDERGROUNDBATTLE_BUILD_ROOT/frozen-kernels/abiNN/` 提供 `hegemony_wasm.js` 与 `hegemony_wasm_bg.wasm.gz`，或设置已有 `HEGEMONY_FROZEN_ABINN_DIR`；`rust-game-wasm/frozen/abiNN/identity.json` 校验JS、压缩包、解压长度及WASM哈希，缺失/错误即停止。

ABI77–88来自已恢复的 `HiddenWorld_JC035_ABI77-89_5056e4e_migration.zip`，ZIP SHA256 `02a4d8fb4ababa881589b80fff7252ac0406063f99defcd0d75574b468b75c43`。该包的ABI89是P2修复前产物，不可用于v71。

v71 ABI89原件来自 `HiddenWorld_JC035_v71_ABI89_100161b_exact_original_inputs_2026-10-21.zip`，ZIP SHA256 `ed328fff20063cb097283040826d2434c00a457388ec85dc817c53084ceb9365`。JS SHA256为`fb262978c41802322caec9fa9dd38ff6e273ff019865fd401a82afd78c1cfebb`，原始WASM为3471247字节/SHA256 `df890ed0b596ca245a39bc8466270edc2e8f40f92ef8dd34359534342ccf24de`。本次仅对这些已验证原字节使用Python `gzip.compress(wasm, mtime=0)`生成运输压缩输入，gzip SHA256 `73fcc1c3804f9c75016bbe1c74664549f29e35f0148fc11a6b7cbec70a7b831b`；没有重编内核。141个历史源码哈希保存在ABI89的native-source-binding.json，历史Native89测试比较该原始见证，不比较不断变化的当前源码。

```sh
node tools/build-storage.mjs -- cargo test --locked --workspace
bash rust-game-wasm/verify.sh
(cd web && npm ci && npm test -- --maxWorkers=2)
(cd sites && npm ci && npm run build)
node tools/build-jc035-clock-worker.mjs "$UNDERGROUNDBATTLE_BUILD_ROOT/audit90-worker/server"
(cd sites && HEGEMONY_CURRENT_WORKER_DIR="$UNDERGROUNDBATTLE_BUILD_ROOT/audit90-worker/server" npm test)
```

`build-jc035-clock-worker.mjs`沿用既有本地工具名，当前复制14个构建模块到新的本地目录，仅供测试。`sites`构建的wrangler命令固定为`deploy --dry-run`；上述命令均不是GitHub CI，不执行发布。独立Site输出用 `node sites/scripts/stage-source.mjs /ABSOLUTE/NEW/DESTINATION`，原始PDF、图像资料库、数据库和凭据不复制，外部冻结输入仍按上述方式供给。

验证结果另存整改记录，未运行的检查不得写为通过。新候选尚未独立复审、合并或部署。GitHub CI/Actions、push、PR、tag/release、自动合main与部署均暂停。
