# 官方 v59 发布回执

父独立复审通过后，按既有明确授权发布到原公开项目；完整说明见 `../../SITES_V59_PASS_HISTORY_PUBLICATION_2026-10-16.md`。

- `official-receipts.json`：source helper 的打开／推送打包结果，官方 v59、部署 `succeeded`，部署后的版本与公开范围回读。
- `publication-input-audit.json`、`provider-source-files.json`、`provider-copy-plan.json`：1122 个有限源码文件、29 个映射改动与完整 162 项产物的发布前检查。
- `provider-provenance.json`：正式来源 `a2a002fb7cafb9030405853a1c98bd91800b627c` 绑定的已审基线、精确补丁、覆盖树、复审证据与清单哈希；保留两份原正式站 fixture 还原钩子。
- `publication-archive-audit.json`：归档 160 个运行文件与复审清单逐项一致；额外 4 个文件只有原 hosting 和 drizzle 元数据。
- `verify_publish_input.py`：官方 helper 在 commit/push 前执行的审计脚本，无凭据输入。
- `normal-browser.json`、`normal_browser.py`、`normal-browser.png`：v59 身份成功核对后，唯一一次正常 TLS 根 URL 导航在 12:17:27.854156 UTC 被证书信任错误阻断；没有绕过，UI 未到达，POST／线上对局均 0。
- `summary.json` 与 `evidence-file-hashes.json`：最终结果及上述文件的字节数／SHA-256；哈希清单不递归计入自身。

无数据库、构建产物、缓存、来源凭据或真实用户桌数据。v59 succeeded 与浏览器未能开始试玩分开记录。
