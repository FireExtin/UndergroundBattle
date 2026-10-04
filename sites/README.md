# 隐秘世界 Sites 适配

沿已验证的 Rust WASM 规则核，为现有 React 客户端提供 Worker/D1 房间服务及版本轮询。已部署至用户授权的公开 Site；当前版本、远端 D1 与浏览器验收见 [云端验收记录](../docs/SITES_CLOUD_ACCEPTANCE_2026-10-02.md)。原生服务及数据库保留。

当前正式身份为 `limited-v2.8 / rust-v0.2.11`，含新JC005和十旧核，共十一核。JC005当前合法卡池的角色附属路径已完成；其他宿主强化目标尚待后续宿主机制。XQ03谓词fixture不证明真正地区实体或秘社区宿主。完整范围见 [正式有限切片说明](../docs/JC005_FORMAL_V011_RELEASE_2026-10-04.md)。

Site 身份保存在 `.openai/hosting.json`，仅注册一次；访问范围遵从用户授权，当前公开入口供没有 ChatGPT 账号的朋友入席。

在本目录运行 `npm ci`、`npm run build`、`npm test`。构建使用既有 WASM 文件，输出 `dist/server/index.js`、相邻 WASM 模块及 `dist/client`；正式发布使用官方 Sites workflow，不运行 Wrangler 远端部署。

D1 保存不经 JavaScript 解析的完整状态、逐版本日志和请求回执。命令在鉴权后比对完整意图，使用房间版本 CAS；同一事务只有成功 CAS 的尝试才能写入日志与回执。创建／加入请求也可用稳定随机请求标识恢复丢失的响应。数据库故障不会返回成功确认。

前端在前台约每1.5秒轮询版本，后台降频，恢复可见时立即同步。保存的是座位凭据与未确认命令，不将浏览器状态作为对局权威。

`scripts/stage-source.mjs ABSOLUTE_DESTINATION` 生成同一 Site 的独立、有限源码副本，带原仓库提交和 WASM SHA-256。它不复制原始规则 PDF、完整素材库、运行数据库或凭据。不要将这个副本当作第二个 Site 注册。

`test/integration.test.mjs` 使用实际 workerd、同一 WASM 和本地 D1 SQLite 仿真，覆盖并发、幂等、CAS 无旁写、事务失败回滚、末席重复加入、最大 u64 精度和持久化重开。这只构成本地证据；真实 D1 与浏览器验收另行记录。
