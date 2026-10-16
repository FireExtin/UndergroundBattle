# BQ035 v60 发布与 main 快进回执

输入：父会话完成独立复审的固定 `bd36b70b32890f539fde019613f5ae6d60c2903d` 及用户明确发布／main 授权。任务说明见 [BQ035_V60_PUBLICATION_AND_MAIN_2026-10-16.md](../../BQ035_V60_PUBLICATION_AND_MAIN_2026-10-16.md)。

- `official-receipts.json`：官方打开源码、推送打包、v60 保存、成功部署、发布后版本及公开范围白名单回执；不保存源仓库凭据或用户账号字段。
- `source-binding.json`：1145 映射文件、两个 package hook 例外、旧支持文件哈希及 provider 138 变更路径。
- `prepublish-verification.json`、`approved-doc-only-delta.json`：固定审批点、852 验证范围及完整运行包身份检查。
- `runtime-manifest.json`、`archive-verification.json`：162 运行文件 + 1 hosting，以及总共 166 上传文件的逐项字节／哈希。
- `archive-identities.json`：本地 gzip／tar 和平台保存 tar 各自口径；不声称整包字节相同，也未下载平台包重新比较内部字节。
- `main-promotion-preflight.json`、`main-promotion.json`、`main-push.log`：正常快进，main 固定 bd，Actions 仍 8、该提交 0。
- `normal_browser.py`、`normal-browser.json`、`normal-browser.png`：一次正常 TLS 访问被证书阻断；0 POST、0 线上对局。
- `summary.json`：上述结果白名单摘要，沿用既有验证结果，不产生新的完整回归声称。
- `evidence-file-hashes.json`：文件校验索引，另记录仅保留本地的准备脚本哈希。

原始发布包及旧 provider dist 保存在 `/workspace/bq035-publication-20261016/`，不加入 Git。全程未新增 worktree 或开发线，未重建已测产物，未重置数据库、触发 Actions 或访问真实用户桌。
