# Site42 历史调试器按需加载与失败恢复已发布

父端独审通过 **97de78d52141ae4ebe41da441cc6410691384509**，确认P2关闭、真实新document恢复而非已缓存lazy假重试，32项相关测试及类型检查/构建通过。用户明确授权更新现有Sites网站，并已有原用户AGENTS中经过检查的后续版本持续发布授权。按父端本轮发布指令，完整官方Sites0.1.75 helper完成现有站点源码打开、构建、推送和打包；原生保存后单次部署直接返回succeeded，没有审批拒绝或重复发布。

## 当前发布事实

**Site42**，现有网址 **https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site**。项目 **appgprj_6abf7bf54a7481918a50e1ef1509ca68**；保存版本ID **appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_20ab97e0166481919a3f2bb63d2f7ea6**；官方provider源码 **4595431ba223736fee200844c200ccda48cd078b**；部署 **appgdep_6ac6b12bb48c8191bfae2fd680647d50** 于 **2026-10-07T20:53:10.402017UTC** succeeded、failure_message为空、env revision0。发布后原生get_site/get_site_version再次确认相同源码/版本/部署，owner/public/active及原网址保留；没有另建Site或修改访问范围、运行数据、数据库迁移。

引擎 **rust-v0.2.49-sealed-restart-candidate**、卡池 **limited-v2.44-jz50-death-search-candidate** 保持；107普通卡（含10地区）、8会社。旧桌current-only提示及数据保留策略不变，没有跨版本迁移或删除旧桌。

## 精确构建与 ABI

从固定97de逐项映射 **459个源码/测试/原图/说明文件**，逐字节与Git源码比对。官方 `build-site.mjs` 完整运行原项目build脚本：两份TypeScript配置检查、Vite生产构建、Wrangler dry-run。**123个客户端文件**全部与审查时的实际构建SHA一致；最终首页JS **325,274字节**，相对Site41少17,401字节（**5.078%**，更新后不再引用初版5.224%）。整个客户端比Site41多1,872字节，收益为默认入口加载量。

正式归档 `/workspace/game-publication-evidence/site42-lazy-debugger-97de78d.tar.gz`，**49,398,199字节**，SHA256 **a6e8fa5da66e5a67814ca8a50999c9f3406f49d346fcbbfa3f9df13fcf82143a**；131文件全部与实际构建核验。后端Worker代码、唯一WASM、manifest和迁移与Site41字节一致；WASM **2,248,074字节**、SHA256 **9b8a7a69cd3c09781387cb4e66ad710d803783447fb6b0382fb2697eac50e885**，实际模块检查2导入/19导出。三项客户端生成文件变化，另有Wrangler README的生成时间变化。首个严格3changed断言因此失败，查看diff确认仅时间后修正为4项检查并通过，原始说明保留在核验记录。后端和历史engine均未改，已有Native/WASM机制证据没有重复冒称本次重跑。

## 验收范围与资源

已有同一审查构建的真实本地浏览器3项通过：首页不请求调试器chunk；chunk503由边界显示错误；点击真实链接打开新document，timeOrigin改变、同chunk URL返回200并恢复原调试器fallback。此证据证明加载恢复，不代表公网UI或实时Go/Rust机制验收。[本地修复证据](LEGACY_DEBUGGER_LOAD_RECOVERY_2026-10-07.md)完整保留。

**公开catalog的原HTTP403未解除，未用浏览器、另一route、bypass凭证或其他执行器重试。** 原生成功终态确认部署成功，发布后公网catalog/UI验收未完成。原官方组件下载403也未重试；既有完整167/167核验的官方包足以完成本次流程，未宣称全局插件安装状态修复。

构建前临时盘约180MB不足三份静态资源峰值，核验并保留三份精确当前WASM后，仅删除本任务自己已停止使用的WASM编译target缓存（逻辑95,526,938字节、314文件，无cargo/rustc/bindgen进程）；源码、绑定、Native测试、历史包不删。构建完成后两份中间静态镜像逐文件比对再删除，逻辑129,303,206字节，保留正式归档及单份实际client/server构建。凭证仅会话内存/隐藏stdin，不入文件、Git或日志。此为构建必要缓存管理，没有继续独立瘦身，也不代表Library空间或线上数据清理。

## 未做项与真实阻塞

- **BQ104**：接受后且有雇员时必须选1，还是允许选0仍洗牌。**XQ48**：原地区被替换后跳过检索/入场但洗牌，还是私检0..3留库再洗。父端已向用户提问，尚无裁定；两张仍未准入，两份未接入草稿原SHA保持。不能把JZ50已裁定的B选项移植为答案。
- **公网验收**：原403阻塞实际公网HTTP/catalog/UI检查；现有Native发布回执及本地证据仅支持各自范围。原官方下载入口的403尚未诊断恢复，但当前完整包已可用，所以不阻塞本次发布。
- **机制浏览器覆盖**：封印三卡/JZ50的完整自然对局浏览器流程尚未逐张补齐；既有实际界面检查覆盖基础牌桌/暂停续接，本次覆盖按需加载/失败恢复。规则机制由已审Native/WASM轨迹证明，不替换为“全部UI机制通过”。这属于后续验证工作，没有阻塞独审已批准的此次包装层发布。
- **Go整体退役**：仍有显式 `/legacy-debugger` 与三个实际API引用。源码在Git可恢复，非唯一历史保留；完整删除需同时退役/替代该功能与代理/共享类型。本次已完成有量化收益的按需加载及失败恢复，没有继续扩展清理范围。
- **Library配额与线上清理**：没有Library容量读数；本地cache删除不能证明Library配额变化。没有实现线上trash/TTL清理、删除旧桌或变更数据策略，这些不在此次发布范围。

凭证清除后的 [发布原生收据](evidence/site42-live-2026-10-07/publication-receipt.json)、459文件映射、ABI、131文件归档检查、官方构建日志及必要缓存收据在 [证据目录](evidence/site42-live-2026-10-07/)。审查源码仍固定97de；本次后续Git提交只记录发布事实和未做项，不推GitHub/main。
