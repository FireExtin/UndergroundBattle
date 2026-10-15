# PC串行整合 / ABI77 证据入口

本目录属于 `codex/hidden-world-serial-integration-review-20261015`，验证源码检查点 `b8ced08ec5a55be3b96bfd96cc528e5aef00d4e3`。整批仍待父会话独立复审，没有部署或线上试玩。

- `verification.json`：当前PC批次完整机器摘要，含Native1169、Web1145、Worker179与明确浏览器范围。
- `native-full.log` / `native-full-summary.json`：未过滤长对局的单次完整Native运行。
- `web-tests.log`：首轮15项失败；`web-tests-final.log`：修正后121文件/1145项完整通过。当前PC整个Web Git树和生产文件与该通过版本一致。
- `sites-tests.log`：179项普通Worker/D1整轮。`sites-build-pc-final.log`：最终PC源码TypeScript、Vite和Worker dry-run，没有实际部署。
- `*-wasm-parity.log`、`xq24-container-results.json`：原样Native状态、完整返回、各席视图与实际WASM拒绝。JC04655份非法状态原文件仍在 `../jc046-2026-10-15/invalid-room/`，新导出匹配清单见 `jc046-fresh-export-match.json`。
- `current-inputs.json` / `native77-generation-commands.json`：50份新77输入的精确hash、39条成功生成命令；禁止把生成命令里的重复测试相加为新的独立Native通过数。
- `artifact-binding-pc-final.json`：最终三处WASM相同、158份生产文件与通过构建相同、Worker包/核未变化、Native二进制指纹；本目录不含编译产物。
- `integration-input-file-matches.json` / `review-source-match-pc-final.json`：17份UI/恢复文件逐字节保留相应来源，JC04614份静态指纹保持；不是新增独立复审。
- `historical-inputs-*.json`：明确清单范围内698份原扫描/历史输入未改；不把此清单数字冒称所有历史资料总数。
- `browser-ui-pc/`：1366×900和1366×768真实浏览器的Native77投影展示/确认区截图及摘要，0 API请求，非游戏重放。
- `browser-recovery-pc/`：最终真实本地HTTP、独立SQLite服务、1366×900 PC的7项恢复检查；两次503为合成丢ACK，不是线上故障或完整对局。源码标记由Git读取并验证；早先手填标记错误的本地运行未作为本目录最终回执。

PC截图示例：[领域展示](browser-ui-pc/1366x900-domain-two.png)、[较矮桌面强制待选](browser-ui-pc/1366x768-mandatory-choice.png)、[组卡](browser-ui-pc/1366x900-deck.png)。手机适配未进入本批，已知离屏限制见主说明。

## 复现边界

使用Rust1.90.0、wasm32目标及wasm-bindgen0.2.104，Cargo统一经 `node tools/build-storage.mjs -- …`。完整Native命令为 `cargo test --locked --workspace -- --test-threads=1`；完整Web为 `npm --prefix web test -- --maxWorkers=1`；构建为 `npm --prefix sites run build`；Worker在sites目录执行 `node --test --test-concurrency=1 test/*.test.mjs`。

浏览器检查为 `node tools/frontend/serial-integration-qa.mjs <新的空输出>` 和 `python3 tools/cloud-playtest/recovery_review.py --binary <当前原生二进制> --cwd <仓库> --static-dir <当前web/dist> --output <新目录> --source-commit <git实际SHA>`。仅使用本地自有服务和库内离线投影，不访问生产数据库或写公开站。

重新生成77输入会拒绝覆盖已存在目标；需在新的普通克隆使用尚未含77输入的b6f19ce检查点及新的非重叠输出目录，按JC046恢复入口生成。不要删除已有材料、改旧opaque状态或改fixture的WASM摘要来掩盖不同产物。历史ABI74身份差异仍未解决，当前77结论仅绑定这里实际测试的字节。

原始日志保留CLI格式，可能有表格尾空格/末尾空行；不为diff空白门禁篡改原日志。无真实凭据、数据库、依赖/构建缓存或编译产物。
