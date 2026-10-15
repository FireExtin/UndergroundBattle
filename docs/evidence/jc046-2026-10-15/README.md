# JC046 / ABI77 WIP 恢复入口

用户在2026-10-15约06:18 UTC明确提前收尾，原07:00开发计划取消。
立即停止新功能并SIGTERM中止已运行的Native回归，不启动额外门禁，不部署或合并main。
此目录保存实际源码候选的原始证据；状态见 `verification-wip.json`。

## 已完成与未完成

- 来源检查40项、生成器守卫22项通过。
- 首轮专项28/29；仅修正光环测试地区布局。修后中止回归中29项JC046全部通过。
- 中止回归共604条pass、无断言fail、没有整轮汇总；退出101来自SIGTERM，不代表全量通过。
- `native-room/` 37轨迹与 `invalid-room/` 55原样拒绝状态按SHA256清单保留，尚未与WASM比对。
- 77 WASM/当前输入/前端/Worker/构建均未完成。WIP引用尚不存在的77输入，不能直接构建发布。
- 三项静态预审缺口已修；最终运行联动复审未完成。新负例覆盖本地Game/Room，不冒称WASM已拒绝。
- 本轮没有整合并行UI或恢复分支，没有接入currentDomains显示。

## 恢复次序（本次未继续执行）

在此分支新取精确远端SHA后先核对工作区，保留任何已有工作；不要reset或覆写已存在输入。
安装锁定工具链，Cargo统一经 `node tools/build-storage.mjs -- …` 守卫串行执行。
本机工具链入口 `/workspace/tooling/env.sh`，本机原始材料 `/workspace/jc046-evidence-20261015`。

1. 在新的证据目录运行JC046专项与完整适用Native回归，确认整轮与集成目标结果。
   若设置 `JC046_NATIVE_DIR`/`JC046_INVALID_DIR`，必须指向新的空目标，导出器拒绝覆盖。
2. 经守卫运行 `bash rust-game-wasm/build.sh` 构建77核；不能继续使用本机残留76产物。
3. `python3 tools/fixture-tools/regenerate_native72_inputs.py <仓库> <新的Native77输入目录>`
   生成当前77其余输入；脚本已含JC095。所有历史74/75/76输入保留不变。
4. `python3 tools/fixture-tools/jc046-current.py <37条新Native轨迹目录>` 生成JC046当前输入。
   此目录保留的 `native-room/` 也可用于检查，但源码若变化必须重导出，禁止给旧状态改标签。
5. `node rust-game-wasm/tests/jc046_compare.mjs web/src/game/jc046Native77.fixture.json <新55负例目录>`，
   以及当前JC060/JC095回归；核对真实结果与所有席位投影和前76拒绝。
6. 本地Sites dry-run打包，核对pkg/generated/Worker三处WASM字节，然后运行前端/构建与Worker/D1。
7. 补齐独立复审与确切SHA后，才评估后续整合/部署；线上试玩必须成功部署且SHA一致。

本目录无真实凭据、数据库、依赖或构建缓存；Room状态来自明确离线布局与本地测试席。
