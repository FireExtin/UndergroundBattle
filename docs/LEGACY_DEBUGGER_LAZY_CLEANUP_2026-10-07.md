# 历史调试器按需加载候选与 Site41 元数据复核

输入：父会话要求发布事实正常提交并继续有独立价值的代码清理；用户已授权清理旧的、不需要的占用。本次基线为协作分支 `f061b227c254657e16976fe76983c2ce6ef1e9af`，其中实际 Site41 产品源码为已审 `2ad04f7b91ca3d6030ff27a5d45598703592fbde`。输出是独立前端候选和本目录证据；本次未重新保存或部署 Sites，未推 main。

`sites.get_site_version` 再次返回保存版本 **41**、provider 源 **42229677a9a69fc6b1d469af0f6539ac03bb429b** 与部署 **appgdep_6ac6a314c9808191a72a0b9ddbc78353**，与成功发布回执一致。原生成功部署和本次元数据核对不代表公网 HTTP/UI 验收；原公开 catalog 的 HTTP403 与组件下载403均未重试。完整官方 Sites0.1.75包及 `site-workflow.mjs` 已由此前167项完整性核验确认可用，并成功执行Site41发布；未声称原下载入口恢复或全局插件安装状态改变。

## 实际 Go 引用与保留条件

`AppShell.tsx` 的 `/legacy-debugger` 显式路由仍调用 `LiveDebuggerShell`；`web/src/debugger/live.ts` 调用 `/api/debugger/messages`、`actions`、`reset`。Vite 的具体 debugger 代理8080，普通游戏 API 代理8090；`server/internal/api/http.go` 实现这些端点，Go main 默认为8080并可托管 `web/dist`。当前 Rust Cargo workspace 与 Sites Worker 构建不运行 Go 后端。

`server/`、`shared/` 和 `go.mod` 共 **123个已跟踪文件、712,801字节，其中88个Go文件**。前两个目录在当前稀疏工作树未物化。文件及历史均在Git中可恢复，不能称为不可替代的唯一历史材料。当前保留是因为可选调试器仍有真实功能入口；明确退役或替代该功能时，应同步删入口、代理、后端、共享类型、模块与相应说明。README已纠正原本把默认游戏入口描述为Go sandbox的错误。

## 最小实现与可量化结果

`AppShell` 用 React `lazy`/`Suspense` 引入新的 `LegacyDebuggerEntry`，调试器和内置fallback数据在显式访问时才加载；原Go端点、调试动作和fallback逻辑未改。`go.mod` 只改用途注释。没有通用路由框架或规则身份变更。

以 **Site41正式归档中的实际入口JS** 比较：342,675 → **324,773字节**，首页初始JS少 **17,902字节（5.224%）**；同一Python gzip level9方法为103,513 →99,217字节，少4,296字节。独立调试器块19,273字节。所有JS合计与整个客户端总字节均增加 **1,371字节**，这是加载拆分的开销；本候选改善默认入口加载，不宣称减少总产物体积。既有CSS及原图等120文件与Site41归档逐字节相同。

## 验证与证据边界

- `npm run build` 成功，包含两个TypeScript配置检查和Vite生产构建。
- 现有 debugger 三个测试文件 **29/29通过**，覆盖现有界面、live动作客户端和fallback行为。
- 对实际构建的单浏览器本地检查 **2/2通过**：首页渲染GameApp并且零调试器chunk请求；显式调试器路由只加载一个独立chunk，原内置fallback正常展示，live提交禁用。无未捕获JS错误；两张实际截图已查看。
- 本地检查刻意没有后端，普通catalog返回501、首页显示既有连接错误；仅证明路由/加载和现有fallback，不证明Rust正常对局、实时Go后端或公网验收。未拦截HTTP响应或注入牌桌。
- `rust-game/`、`rust-game-wasm/`、`sites/` 的已跟踪diff为空，engine49/pool44及历史engine/fixture文件未改。此前Native626、Worker33和实际WASM6311命令/25404视图完整一致属于Site41已有证据，本次没有重复运行。两个未接入的有限搜寻草稿原SHA保持；BQ104/XQ48仍待用户裁定、未准入。

完整客户端123文件逐项SHA清单和前后指标在 [证据目录](evidence/legacy-debugger-cleanup-2026-10-07/)。仅三项生成文件变化（HTML、新入口JS、新调试器JS）；已归档为105,343字节增量包并重新打开逐字节验证，SHA256 **59c9d8f1edc21a0648b8163f6a791048cb1e662f48a1f4f73682af17fd522dc3**。120项未改资源可直接从已验证的Site41正式包恢复。清理此次可重建 `web/dist` 前先完成全部文件覆盖核验；源码、正式包、原图、WASM、Worker、Git历史和审查证据保留。

这是本地云端协作分支的待审清理候选，现网仍为上述Site41源码。Library容量没有可用配额读数，本次缓存删除释放的是执行器磁盘，不是Library或线上数据。新的规则范围没有扩展。
