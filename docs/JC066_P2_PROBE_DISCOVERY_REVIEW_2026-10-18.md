# JC066 专用恢复探针默认发现修正（待其余独审）

> 后续状态：父会话已确认完整批次独审通过；官方 v68 / ABI86 已于 11:19:44 UTC 发布成功。见[成功发布回执](SITES_V68_JC066_PUBLICATION_RETRY_2026-10-18.md)。下文保留当时验证与范围；普通浏览器仍受 TLS 证书阻断，0 场线上试玩。

输入为父会话增量独审指出的测试基础设施 P2：`sites/test/jc066-p2-restore.test.mjs` 在模块加载时要求 `JC066_P2_CASES_FILE` 和专用 Worker 路径，但默认 `npm test` 会展开 `test/*.test.mjs` 并加载它。产品恢复 P2 已由该轮独审确认闭合；用户自封印 B 仍为项目裁定，不是官方 FAQ。

从 `a8bf844c30cf988abb25af73e15951849b5fc93a` 作最小修正：将探针移到 [`sites/probes/jc066-p2-restore.mjs`](../sites/probes/jc066-p2-restore.mjs)，更新一处辅助模块相对导入并加显式入口注释。探针的断言、实物输入要求、WASM/HTTP/D1 验证行为均不变；不静默跳过、不替换反例。默认 package 脚本保持原样。当前修前/修后命令见 [`sites/probes/README.md`](../sites/probes/README.md)。

本轮三次命令均自然结束、exit 0：

| 验证 | 结果与范围 |
| --- | --- |
| 默认发现/模块加载 | 使用原 `test/*.test.mjs`，显式清空四项 P2/Worker 环境变量，55 个默认文件全部加载；通过名称过滤只执行 1 项既有 ABI86 身份断言。Node 报 55 个文件/断言结果通过，**不是完整 55 文件用例回归** |
| 新入口、旧实物 | 4/4，WASM `a389ee48…`；两方向坏档仍被接受，实际 HTTP/D1 写入各 1 命令/1 日志，重开后仍取原收据，成功复现旧漏洞 |
| 新入口、修复实物 | 4/4，WASM `2c9b850e…`；同一字节两方向坏档各 6 个 WASM 入口拒绝，合计 20 次 HTTP400，重开前后 room/commands/journal 无额外写入 |

只核对既有产物，没有重建：135 个编译输入仍匹配产品提交 `fa94ec406a9e8163ed9b42756181bd629d3a129f`；当前 WASM SHA256 为 `2c9b850eb5b9e5e0b2e5e137d5056154602e82ba05db7d33ad251209ba2c3352`，86 夹具为 `d58b1309cd86fed440755dc883ffb55e3b52bc3dc308b56d925e84550c83d0d6`。181 个生产文件（含 167 客户端文件）、20 个生成文件及 13 个测试 Worker 文件与原产物清单完全相同，77–85 未改。

证据落在 [`docs/evidence/jc066-probe-discovery-2026-10-18/`](evidence/jc066-probe-discovery-2026-10-18/README.md)：自然命令回执、无损日志、发现列表、产物核对和 SHA 清单。上轮证据的历史旧入口回执/内嵌命令保持原字节；只给旧 README 追加当前入口说明，并同步其 manifest 中该 README 的摘要。

没有扩大重跑 Native/完整前端/全 Worker，没有新 worktree/子代理、数据库或构建产物入 Git、main 合并、Sites 保存/发布或 exchange 操作。本项修正回交父会话，其余独审继续；本轮没有新的线上检查或线上试玩，不把本地实物探针当作线上试玩，站点沿用此前 v67 的已核查记录。
