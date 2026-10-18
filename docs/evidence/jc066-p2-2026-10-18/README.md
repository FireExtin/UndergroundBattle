# JC066 P2 精确恢复修复证据（WIP，待父会话复审）

候选源码 `fa94ec406a9e8163ed9b42756181bd629d3a129f`（实现检查点f2f68e5，随后只记录用户B裁定）；独立分支 `codex/jc066-p2-target-face-review-20261018`。基线 `8607a94298ed323b0375106fac7db01ffbc77ccd`。产品只增加jc066_target内一条live同UID face_down与public.kind一致性条件。

| 范围 | 本轮实际结果 |
| --- | --- |
| 精确静态反例修前 | Native1项复现、WASM/HTTP-D1 4项复现成功：两方向都接受，HTTP200，pause实际产生1命令/1日志，重开原收据仍返回200 |
| 同一字节修后 | Native两恢复入口、两方向各六个WASM入口拒绝；HTTP-D1 4/4，共20次HTTP400，重开前后room/commands/journal零额外写入 |
| JC066专项 | 16/16，含精确反例、合法返手/移动/控制/现身续行 |
| 完整Native workspace | 单次1314/1314（1247库+67集成），默认自包含P2用例也通过 |
| Native→实际WASM | 135输入，19场景252命令16预期拒绝1084视图87坏档；所有完整状态/日志/四席视图一致，原15轨迹完全不变 |
| 集中Worker | 单次130/130：新JC066106 HTTP/D1 +18准确内核路由 +6来源保护 |
| JC066 HTTP明细 | 19正常轨迹236接受/16拒绝/2104四席视图，87坏档870次HTTP400；重开、所有原收据、坏档零写入 |
| 相关前端 | 3文件51/51；生产双TS/Vite/十核dry-run通过，167客户端文件与修前逐字节一致 |

用户于2026-10-18 10:33 UTC明确选择自封印B：自身可选，结算后立即回owner手牌。`user-project-ruling.json`将其记为用户项目裁定，**不是官方FAQ明文**，不扩大其他授权，不豁免父会话复审。既有代码与测试已符合B，没有为此改变测试中的代码或中断进程。

## 文件

- `validation-summary.json`：上述实测计数、自然exit、精确before/after及10点小时网站检查。
- `artifacts.json`：源码Git blob与135输入、Native raw、fixture、新旧WASM、181生产文件及十核一致性、167客户端相同、535个修前生产文件原件保持。
- `sources/*-repro.json.gz`：Rust实际产生的精确原状态/只改kind与label的坏状态、两恢复入口结果、编译输入绑定；JS仅传输不透明Room字符串，没有JS解析/重写Room。
- `sources/before-regression-jc066_tests.rs.gz`、`baseline-source-proof.json`：修前135输入中134个与基线Git完全一致，只追加复现用例；该测试源已从原Git和追加用例重构并核对编译时hash，不能把EXPECT=accept误解为产品绕过。
- `sources/exact-original-inputs.json.gz`：来自原86 fixture的两个原始状态，给定entry-deploy-accept.steps[9].expected.state完全保留。
- `runs/`：13次已自然结束的原exit JSON与无损gzip日志，没有中断/伪造输出。修前“通过”是成功复现接受漏洞，不能当修后安全结果。
- `audit-artifacts.py`、`assemble-evidence.py`：在选定执行环境生成材料的原脚本；输出新文件，不覆盖原件。
- `official-v67-10.json`、`normal-browser10.json`：10:29官方仍v67/public revision2/原部署succeeded；10:29:27普通Chromium只导航一次，ERR_CERT_AUTHORITY_INVALID后停止，0写入0线上游戏，无绕过或重试。

## 身份与保留边界

新WASM `2c9b850eb5b9e5e0b2e5e137d5056154602e82ba05db7d33ad251209ba2c3352`，3291234B；修前 `a389ee48edb55ef5089a641c781352f8890ea1a6238efd1bde2664fb6f78d30e`，3291176B。只有当前未发布86内核被替换，原77–85保持原字节，不重建替代或重签旧夹具。Native raw `ce6f0329fd30daca4b2d5a960ad4310eec3e4b4a82394ca941b5f642531ffb50`；新86 fixture `d58b1309cd86fed440755dc883ffb55e3b52bc3dc308b56d925e84550c83d0d6`。完整Native与专项导出逐字节相同。

原完整前端1342未在本轮重跑，使用51项相关组件与实际新核视图、完整生产TS/Vite及167客户端字节相同作适用验证。旧Worker306/全55文件不重跑、不称全绿：既有306首轮294过/12个JZ46测试版本引用失败，修后JZ46文件15过；原覆盖16文件，另5文件6项，依父会话指定范围保留精确边界。不得跨轮相加。

原件、当前生产产物、合成D1和原始证据在 `/workspace/jc066-p2-evidence-20261018`。Git只保存源码/测试/必要材料，没有数据库、编译WASM、缓存、凭据、真实私人数据或聊天内部标识。没有新worktree/子代理、main合并、Site保存/部署或exchange操作；待父会话复审通过后才进入既有Sites发布授权。
