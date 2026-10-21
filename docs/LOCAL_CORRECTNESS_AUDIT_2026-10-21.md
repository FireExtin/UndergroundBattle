# 固定 100161b 审查后的本地整改（待独立复审）

审查基线为 `100161b628e56b9153041d952ac0f5d83428ca88`，产品 v71/ABI89。固定审查报告、5689文件/304目录清单、805普通代码文件逐段覆盖账本与原始失败证据保存在本任务独立审查目录；未把第三方归档、历史补丁、全部资料或浏览器场景称为全量验证。本文件记录后续修改，不替代固定版本报告。

当前候选引擎标识是 `rust-v0.2.90-audit-correctness-candidate`；卡池仍为153普通卡/10秘社，牌面目录与预组没有新增。Sites沿既有版本分派保留ABI77–89，旧房间不改存档、不重新签名或转换到新内核。本次规则修复只适用于ABI90；保留的历史内核仍有其原版本缺陷。Native服务仍只有当前原生内核，不能把Sites历史内核路由视为原生数据库迁移支持。

修改内容：合法时间标志与BQ028奖励来源的恢复；跨角色声望/荣耀准入；多组独有冲突连续处理；香港空检索私密确认；三处ID生成复用；旧结束步骤夹具；独立Site源码复制闭包；XQ25附属图标。合并重复的附属图标路径及冻结字节装载；删除重复XQ23校验；纠正过时说明、恢复检查点字段和回放哈希报告；删除没有生产调用的BQ030 preparation原型及其仅自测模块，当前编译见证移除对应文件；冻结ABI77–89原始来源见证不变。删除已无法生成当前166项图集的旧generate-card-scans入口，保留现有卡图注册与validate-admitted-scans校验。三个来源测试改用批准历史对象的固定语义哈希，执行时不再要求Git历史。

## 本地构建与输入

需要Rust工具链 `rust-toolchain.toml`（1.90.0）、wasm32-unknown-unknown、wasm-bindgen 0.2.104，以及web/sites锁定的npm依赖。先在根目录设置自己的独立 `UNDERGROUNDBATTLE_BUILD_ROOT`，按现有 `tools/build-storage.mjs` 运行，避免把缓存放进源码。

冻结输入是必需的外部原始产物，不能用当前源码重编历史ABI。每个ABI在 `$UNDERGROUNDBATTLE_BUILD_ROOT/frozen-kernels/abiNN/` 提供 `hegemony_wasm.js` 与 `hegemony_wasm_bg.wasm.gz`，或设置已有 `HEGEMONY_FROZEN_ABINN_DIR`；`rust-game-wasm/frozen/abiNN/identity.json` 校验JS、压缩包、解压长度及WASM哈希，缺失/错误即停止。

ABI77–88来自已恢复的 `HiddenWorld_JC035_ABI77-89_5056e4e_migration.zip`，ZIP SHA256 `02a4d8fb4ababa881589b80fff7252ac0406063f99defcd0d75574b468b75c43`。该包的ABI89是P2修复前产物，不可用于v71。

v71 ABI89原件来自 `HiddenWorld_JC035_v71_ABI89_100161b_exact_original_inputs_2026-10-21.zip`，ZIP SHA256 `ed328fff20063cb097283040826d2434c00a457388ec85dc817c53084ceb9365`。JS SHA256为`fb262978c41802322caec9fa9dd38ff6e273ff019865fd401a82afd78c1cfebb`，原始WASM为3471247字节/SHA256 `df890ed0b596ca245a39bc8466270edc2e8f40f92ef8dd34359534342ccf24de`。本次仅对这些已验证原字节使用Python `gzip.compress(wasm, mtime=0)`生成运输压缩输入，gzip SHA256 `73fcc1c3804f9c75016bbe1c74664549f29e35f0148fc11a6b7cbec70a7b831b`；没有重编内核。整改交付包另附`frozen-kernels/abi89`下这两份精确运输文件，接续环境直接使用它们，不依赖跨Python/zlib版本重压缩的字节一致性。141个历史源码哈希保存在ABI89的native-source-binding.json，历史Native89测试比较该原始见证，不比较不断变化的当前源码。

```sh
node tools/build-storage.mjs -- cargo test --locked --workspace
bash rust-game-wasm/verify.sh
(cd web && npm ci && npm test -- --maxWorkers=2)
(cd sites && npm ci && npm run build)
node tools/build-jc035-clock-worker.mjs "$UNDERGROUNDBATTLE_BUILD_ROOT/audit90-worker/server"
(cd sites && HEGEMONY_CURRENT_WORKER_DIR="$UNDERGROUNDBATTLE_BUILD_ROOT/audit90-worker/server" npm test)
```

`build-jc035-clock-worker.mjs`沿用既有本地工具名，当前复制14个构建模块到新的本地目录，仅供测试。`sites`构建的wrangler命令固定为`deploy --dry-run`；上述命令均不是GitHub CI，不执行发布。独立Site输出用 `node sites/scripts/stage-source.mjs /ABSOLUTE/NEW/DESTINATION`，原始PDF、图像资料库、数据库和凭据不复制，外部冻结输入仍按上述方式供给。

`verify.sh`用现有两个XQ24 Rust回归测试导出当前Native输入，再检查当前WASM的恢复入口；不要求手工复制一套v090历史夹具。导出的opaque状态直接作为字符串交给WASM，保留u64精度。单独运行`tools/cards/check-xq24-containers.mjs`时，第三参数可指定该导出目录（含`invalid.json`和`room/*.json`）；历史版本化夹具入口继续保留。

## 本环境验证结果

- 修复阶段Rust workspace：1365通过；此全套编译早于最终法术奖励限制收窄及死原型删除。最终源码另有5项公开命令、4项奖励负例/来源绑定、2项XQ24回归通过，不能把它们写为最终提交全套重跑。
- 默认Native生成及最终WASM比较：433场景、24407迁移、96991视图相等，含两处原失败cleanup。Native生成器早于上述两项末尾变动，最终WASM为3471082字节/SHA256 `9e8aba115338ad8cc628ff631b4db9074a5b3a6d996097a8f8f36e966f61e009`。另用最终源码导出的XQ24输入验证4非法状态/12恢复入口和12正常场景117命令370视图通过。
- Web全套：1431通过、1陈旧文案断言失败；修正后的该文件4/4通过。Sites全套：736项中708通过、28旧当前身份断言失败；四组修正后的28项均定向通过。两者都没有为更新状态再跑整套。
- tools来源测试99/99、冻结输入专项6/6、三文件无Git历史来源测试9/9通过。最终WASM、Web、Sites14模块dry-run及独立stage的13冻结loader/backend dry-run通过；独立stage前端未再次构建。
- 原始v71 ABI89精确哈希门禁及oracle通过；本地同一HTTP/D1实例的ABI77–90共14桌状态、u64和重复回执验证通过。香港私密性与BQ028公开Room/WASM恢复探针通过。

详细原日志、阶段来源和覆盖边界在独立整改证据包。未进行浏览器试玩、线上回归或GitHub CI。新候选尚未独立复审、合并或部署；GitHub Actions、push、PR、tag/release、自动合main与部署均暂停。
