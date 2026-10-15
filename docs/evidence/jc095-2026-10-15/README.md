# JC095 / ABI76 可复核材料

基线 `e53e8b017f0ec974c6cf3d8f446b6541771cf4a5`，当前身份与完整范围见
[批次报告](../../JC095_BLACK_INFLUENCE_CONVERSION_REVIEW_2026-10-15.md)，独审见
[封存报告](../../JC095_INDEPENDENT_REVIEW_2026-10-15.md)。未部署，未线上试玩。

`verification.json` 汇总结果，`native-summary.json` 明示跨轮计数与唯一过滤的长自然对局。
Native 首轮旧黑牌数量断言、Web 首轮旧黑色独有名单均保留原失败日志，修正后定向通过。
不把跨轮结果声称为76单次全量通过。`web-build.log` 为TypeScript与Vite结果；
`sites-tests.log` 为142项本地Worker/D1结果。

`current-inputs.json` 固定48份新76输入字节；历史输入未修改、未重新标记。
`jc095-wasm-parity.log` 包含11连续Room场景146命令614席位投影、45伪造状态双入口拒绝，
以及原75存档拒绝。起点是明示离线布局，后续是真实Room命令，不等同线上自然试玩。
`jc060-current-wasm-parity.log` 是当前76下的灰牌回归；未重做已通过的v53线上验收。

复现使用锁定工具链、`node tools/build-storage.mjs -- …` 守卫串行Cargo。
生成器仅允许不存在的新目标，禁止覆盖已提交输入或修改opaque state中的版本/哈希。
具体生成入口为 `tools/fixture-tools/regenerate_native72_inputs.py` 与 `jc095-current.py`；
生产WASM一致性使用 `rust-game-wasm/tests/jc095_compare.mjs`，可传45负例目录为第三参数。
本地打包通过 `sites/scripts/build-backend-test.mjs` 的dry-run，不代表发布。
数据库、凭据、依赖缓存、编译产物未进入候选提交。
