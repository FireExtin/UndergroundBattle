# v0.2.9 桌面档案主题发布

输入是已审本地实现 `6346fc66df4c54472b42a2c1ab0201f7b3dd4d92`，位于 `codex/hegemony-archive-desktop-integration-20261004`。它未推送到原 GitHub 仓库；本次只更新原 Site 的内部源码仓库。前端主题来自已审 `a0ede4fd0b5a5d2f3e2f6b783a96a5525dae1646`，另有已审手牌高度修正。

输出仍为项目 `appgprj_6abf7bf54a7481918a50e1ef1509ca68`、原公开地址 `https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site`。使用现有 `stage-source.mjs` 和 `build.mjs`，发布包保留 `dist/` 前缀；由原生 Sites 工具保存和部署。实际源码 SHA、保存版本、部署回执以 `/workspace/.private-validation/archive-desktop-site-publish-20261004/` 内的执行证据为准。

构建输入保持 `hegemony-pdf-v1 / limited-v2.6 / rust-v0.2.9`。九个 WASM 与原 Site19 哈希一致，Worker SHA256 仍为 `5b1c160ab072b8ca60a01eeb193364ccb961341b51fdf71ad5e63f1a7f285b49`；D1 逻辑绑定仍为 `DB`。未修改权限、数据库迁移或房间数据。

验证复用同一产品代码已通过的 284 项前端测试、桌面及 Air/BQ022 局部浏览器证据，并重新运行暂存项目生产构建和 Worker/D1 测试。部署成功仅由原生部署回执确认；这里不声称完成新主题的公网自然游戏验收。
