# 云端互联网部署：已确认能力与待决事项

截至 2026-10-02。本文件记录调查证据，不代表部署方案已经选定或发布。

## 当前可运行成果

- 原生 Rust 服务 + React 界面已在云端运行，SQLite 先持久化再确认及广播；双人和四人独立浏览器上下文均已完成对局。
- 8090/8091 是执行环境内部服务地址，不能作为用户互联网试玩入口。
- 同一 Rust 规则核心已编译为 WASM（803612 字节）。Node 成功实例化模块，对照 374 次状态转移、1119 次座位投影，opaque 状态字符串与公开视图均和原生完全一致；u64::MAX 与 2^53+1 种子保持精确。TypeScript 没有另写规则。
- `web/package-lock.json` 已补齐跨平台可选依赖，`npm ci --dry-run --ignore-scripts --no-audit --no-fund` 通过，便于独立环境重新构建。

## Cloudflare 平台与 Sites 产品分别确认

Cloudflare [Workers WASM 文档](https://developers.cloudflare.com/workers/runtime-apis/webassembly/) 明确支持 Rust 编译模块。它要求预编译模块，且不支持线程。纯规则核心不依赖网络、文件系统或 WASI；原生 Axum/SQLite 服务本身不会直接变成 Worker。

Cloudflare [Durable Objects 概览](https://developers.cloudflare.com/durable-objects/)和 [WebSocket 文档](https://developers.cloudflare.com/durable-objects/best-practices/websockets/)提供房间协调、持久状态和实时连接能力。这支持“Worker 鉴权入口 + 每房一个 DO + Rust WASM 核心”的讨论方向，但不证明 ChatGPT Sites 已开放对应绑定。

本环境读取的 Sites `starter-capabilities.md` 仅说明逻辑 D1/R2 绑定。`sites-hosting/SKILL.md` 要求 `.openai/hosting.json` 只存 `project_id`、可选静态配置、逻辑 D1/R2、已验证的插件/连接器及受支持的 capabilities。当前未发现 DO 声明、DO migration、WASM 文件打包的产品级支持说明；因此这些能力均标为“待确认”，不是“平台不支持”。

Sites `list_sites(limit=1, role=owner)` 只读调用成功，连接可用。本任务尚未创建 Site，没有项目 ID、部署版本或外网 URL。

## 当前执行环境的发布工具缺口

Sites 托管技能要求运行 `node <plugin-root>/scripts/site-workflow.mjs --project-id <project_id>`，由脚本完成有序检查、源码推送、打包和归档校验。

本环境只有云端技能资源，没有可执行的插件目录；在 `/workspace/.codex`、`/workspace/.agents`、`/opt`、`/usr/local/lib` 与 `/tmp` 未找到脚本。通过技能资源读取以下脚本失败：

- `scripts/site-workflow.mjs`
- `sites-hosting/scripts/site-workflow.mjs`
- `scripts/project-setup.mjs`
- `sites-building/scripts/project-setup.mjs`

可在具备同一 Sites 插件脚本的环境接管发布，或补齐当前云端的插件文件。接管前明确单一 Site 生命周期负责人，防止重复注册。不能伪造 hosting 配置或绕过上述工作流来声称发布成功。

## 房间层验收条件

部署架构仍由父线程与用户讨论。无论最后使用 DO、D1 或原生服务，都需要保留下列行为：

- 令牌确定座位；客户端不能指定 actor，邀请只能占空座。
- 同房命令串行生效；异步存储/网络 await 期间仍处理交错风险。
- 固定 expectedVersion、原始幂等 ACK 与拒绝无状态变化。
- 状态、去重记录与日志原子持久化成功后才能确认或广播。
- 对每个连接单独生成私有投影；队友也不能读取手牌及暗牌牌面。
- 重启、休眠、断线、刷新恢复原 pendingChoice、实例 ID、PRNG 与版本。
- WASM 的完整状态仅作为服务器端 opaque JSON 字符串传递；JS 不解析重序列化 u64 seed/PRNG，也不把完整状态发送到客户端。
- 上线后使用真实互联网入口、多浏览器上下文验证同步与恢复。云端回环地址测试不能替代此项。

尚待决事项：Sites 的 WASM/DO 支持、发布脚本可用性、私有 Site 的实际试玩者访问范围。付费服务、长期新凭据或扩大公开范围均未授权。
