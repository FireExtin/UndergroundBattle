# JC095 / ABI76 最终复审附录

日期：2026-10-15。补充 [独立源码复审主报告](JC095_INDEPENDENT_REVIEW_2026-10-15.md)，不修改其内容。基线仍为 `e53e8b017f0ec974c6cf3d8f446b6541771cf4a5`；最终提交身份由父任务保存后记录。

**结论：`web/src/game/MSJC07Scan.test.jsx:25` 的名单及标题修正适当，没有新增阻断问题。** 此次只读复核当前文件、目录数据、相关源码和父任务日志，没有执行 Git、Cargo、前端测试或构建；仅新增本附录。

首轮 Web 日志显示，旧精确名单为 `JC094 / JC096 / WM059 / XQ38`，实际多出 `JC095`。当前 ABI76 目录逐项过滤 `color == 黑 && unique` 后恰为 `JC094 / JC095 / JC096 / WM059 / XQ38`，且 JC095 的黑色、金名独有已经原图及主报告确认。因此加入 JC095 是更新新增卡后的完整预期，没有把相等断言改成包含或放宽过滤条件。

测试标题原先写“three”，与修正前已有四项名单也不一致；改为不写固定数量合理。`MSJC07Scan.test.jsx:26` 仍要求构筑 `minimumColor=黑、count=25`；`:29` 起仍检查 24 张失败、25/33 张通过，以及 49 张牌组、超三张副本、秘社混入主牌组均失败。`society.rs:234` 的原卡正文也要求 25 张或更多黑色派系牌。这次没有改变构筑门槛、能力或生产过滤逻辑。

主报告文件 SHA256 仍为 `47e888fccc98b2efdc2741e1c71fad37f84f73d65323fcf642ac37a29228c098`；复审已重新读取其 31 个源码/来源文件和 8 个证据，全部 39 项与封存指纹相同。

本次实际读取的日志均位于 `/workspace/jc095-evidence-20261015/`，运行归父任务所有：

| 日志 | 已核实结果及边界 |
| --- | --- |
| `web-tests.log` | 首轮 107 个测试文件，106 通过、1 失败；971 项中 970 通过，唯一失败是上述旧名单。保留首次失败。 |
| `web-count-fix.log` | 修正后仅执行 MSJC07Scan，4/4 通过；不能表述为修正后再次完整运行 971 项。 |
| `web-build.log` | 两个 `tsc --noEmit` 项目检查及 Vite 生产构建命令完成。 |
| `sites-tests.log` | 142/142 通过，0 failed / skipped / cancelled；包含 JC095 全部 11 条连续场景、完整状态和日志、D1 重开及原命令回执。属于本地 Worker/Miniflare 验证，没有生产发布含义。 |
| `jc095-wasm-parity.log` | 11 场景、146 命令、614 席位视图、3 个预期拒绝、45 个非法存档；完整 Native 状态相等，未改写的 ABI75 状态被拒绝。该日志补充主报告当时待完成的验证。 |

这些结果没有新增四端浏览器自然整局或 JC095 专属秘社检索整链的验收声明；主报告所列机制探针与自然试玩边界继续适用。

以下指纹绑定本次读取的测试修正、相关证据及当前 WASM 字节；不改写主报告的冻结记录。

读取时间：2026-10-15 06:10:03 UTC。适用对象为上述 ABI76 文件及证据，不自动扩展到后续 ABI77。

| 文件 | SHA256 |
| --- | --- |
| `web/src/game/MSJC07Scan.test.jsx` | `607cbbcbbde81a176b9bc6192ed1d040a088d5e38b9bbf8db3885614d2b41161` |
| `docs/JC095_INDEPENDENT_REVIEW_2026-10-15.md` | `47e888fccc98b2efdc2741e1c71fad37f84f73d65323fcf642ac37a29228c098` |
| `rust-game-wasm/pkg/hegemony_wasm_bg.wasm` | `5c00126ab73033631150d78bdfc21076bc2b30b0bf3cf41dba20cb1bae95ddf5` |
| `/workspace/jc095-evidence-20261015/web-tests.log` | `bdc5b9f20b525481a1b5204ae8421180640b0887b24b9b1967e719f0f25f3954` |
| `/workspace/jc095-evidence-20261015/web-count-fix.log` | `49955d083ca146be6a2c8c37558e0ed0199dbd81789c48a3ee871c0f87b27fda` |
| `/workspace/jc095-evidence-20261015/web-build.log` | `89dcb13787f6c8ddf4b5835a5a3add6680fa6e670689d337be54270606ee51ad` |
| `/workspace/jc095-evidence-20261015/sites-tests.log` | `e97b3391e0677974154f6f445f151d748eda4cd1a1e4222d3fd77aeec62ad403` |
| `/workspace/jc095-evidence-20261015/jc095-wasm-parity.log` | `29e2460a7a8c78ea3a42f024fdea5e6f42341f536336cd3c352432402acbff4a` |
