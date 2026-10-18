# JC041 / ABI87 v69 发布证据

结果：原公开站 v69 已由官方 Sites 成功部署，源提交 `0bf3a2793b0e4d28d4324b8251cebde685c8c085`，已审游戏 HEAD `4ffa7155537187d057b52612d2da076da6a2a47d`，Rust 产品源 `6b5b824b1e3fd08edfb1b47f7d979ef1ffe69317`。详见 [发布回执](../../JC041_ABI87_PUBLIC_RELEASE_2026-10-18.md)。

| 文件 | 用途 |
| --- | --- |
| `production-preflight.json` | 冻结清单自身哈希、183 个生产文件、新旧 11 内核、真实时钟与发布前空间 |
| `provider-input-audit.json` | 137 编译输入、配置/恢复钩子/历史缓存保留、有限源改动审计 |
| `provider-source-files.json`、`provider-copy-plan.json` | 已审游戏输入到 Sites 推送源的逐文件映射、字节数和哈希 |
| `official-source-workflow.json` | 官方 helper 打开结果与推送/打包结果；不含短期凭据 |
| `publication-archive-audit.json` | 183 生产文件加 3 个既有 Drizzle 文件；归档、原始与规范化 tar 哈希；推送源 blob 核验 |
| `official-publication-receipts.json` | 保存版本、部署成功和前后回读的官方结构化回执；已排除 bearer/凭据 |
| `v69-browser-preflight-blocked.json` | 普通浏览器启动前空间守卫退出 78；浏览器未启动、0 GET/POST、0 线上试玩 |
| `prepare-reviewed-source.py`、`verify-publication-archive.py` | 本次精确源准备与归档验证脚本；不含凭据，未替代官方 helper 推送/打包 |
| `manifest.json` | 上述十个文件的字节数和 SHA256 |

发布前后的完整测试边界保留在 JC041 全批次及 P2 材料中，本目录没有新增整套回归通过的主张。v69 没有普通浏览器 TLS 观察结果；不得把空间守卫阻挡写成 TLS 通过或线上试玩通过。

原始 gzip 归档 `/workspace/jc041-publication-20261018/abi87-reviewed.tar.gz`、旧 v68 产物 `/workspace/jc041-publication-20261018/prepublish-v68-dist` 与历史其他证据均保留。未清理旧证据、缓存、数据库或保护锁，未推进 main。
