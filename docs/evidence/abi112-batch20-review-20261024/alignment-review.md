# ABI112 活卡表版本对齐：独立静态补充

固定核查 `6e5567a4c3a2e4374fcf33d830cc95b434ed9945` 相对已审 `33bb5824f7c756a87f68c0b19ad1612a0b59e233`；祖先关系已验证。本次未发现此窄范围的确认问题，新增 P0/P1/P2 均为0。仅做 Git 固定对象、源码与哈希静态核查，未启动 Cargo、Node 测试、WASM、Worker或浏览器。

`rust-game/data/cards.json` 的逐字差异恰好是一处顶层 `engineVersion`，从 `rust-v0.2.111-import20-rulings-complete` 改为 `rust-v0.2.112-shared-keyword-rewards`，与原本已为112且本次未改的运行时常量一致。全部272张卡记录及233张准入相同；其余顶层字段、完整卡图目录树和 `docs/factions/card-specifications.json` 均未变。原 `docs/factions/import20-rulings-2026-10-24.json` 字节未变，仍保留111、`independentReview:notRun`、`published:false` 的历史记录；新测试分别断言活表112与历史111。

三个已审产品运行时文件 `rumor.rs`、`batch3_entry_order.rs`、`engine.rs` 与33提交逐字相同，sha256分别为 `e54988b6f57d0ca2b5814d9227b1674c968ad7d1de583646d2ebb5e512c6b688`、`881663e07bb22a376dd1718f7929acba933f23dd440d86803c936d29659298b2`、`09e55ae23b54d4b41d2a511c907cedac29e157060a5e7c27dab42e6842ab9e42`。其余已有 Rust 源文件也未修改；`lib.rs` 仅增加 `cfg(all(test, not(feature = "society-fixtures")))` 的新测试模块引用。没有新增生产兼容内核或状态迁移。

严格校验没有被删除或绕过，以下三文件与33逐字相同：`sites/scripts/build.mjs:12–14` 检查规则、卡池、引擎三项一致；`tools/fixture-tools/current-bq030.mjs:15` 检查 runtime/data 的引擎一致；`tools/fixture-tools/jc052-current.py:20,26–33` 保留引擎/卡池一致及原状态、实际接受命令、视图版本的精确关联。`tools/current-native-source.mjs` 的完整 catalog 深比较、输入/fixture 哈希约束也未变。

本地 clock wrapper 只把既有 `X-Local-Clock` 优先、其后 `LOCAL_TEST_CLOCK_MS`、最后0的取值方式接齐，仍要求非负安全整数；原先实际112 WASM字节绑定、全新输出目录约束和仅本地输出范围均保留，生产 `sites/src/kernel.mjs`、`sites/src/http.mjs` 未变。新6个 Room 见证从当前 Native 的合法布局和实际命令生成 opaque 状态，使用 `u64::MAX-112` 的 seed 字符串见证；消费者仍做精确112 catalog、完整状态/四座视图、重复命令及日志比较。代码中未见将旧局版本字符串替换为112或将 opaque state 经 JS 数值化重写。本提交没有修改任何既有 `rust-game-wasm/tests/` fixture；这只证明本固定差异未虚假重标旧状态，不替代新输入的实际生成和跨端验证。

原91初审与33闭合的16项、11项锁定证据哈希全部未变，原报告不覆盖。此前独立4红→4绿仅适用于已记录的33候选窄复验；本次不增加动态通过数、不声明完整套件或平台独立通过，也不确认正在主线重建的WASM/Native输入。无当前审查阻塞，未占用 Cargo/CPU构建窗口。
