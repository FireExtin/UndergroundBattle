# Site20 桌面卡牌详情对比度修复

输入是当前发布实现 `6346fc66df4c54472b42a2c1ab0201f7b3dd4d92` 和父线程已审前端增量 `539ffc33e80220f04f87dc24a650ff12552ac79f`。本地 cherry-pick 为 `0837dd481dccc22048d5bbf03e64634db5829380`。生产改动只有两处删除浅色文字覆盖，让详情规则、类别与忠诚恢复现有深色。

前端完整测试 28 文件 / 284 项通过；typecheck 与 production build 通过。仓库新增的 `tools/frontend/archive-contrast-qa.mjs` 在 Chromium 实测完整 Table/CardContent/ReadModal 与真实 CSS，采用明确标注的 LC19 UI fixture，不是对局验收。1440×1000 和 1366×768 的详情、hover、文字 reader、原图 caption 共八个面板均通过；无 API 请求或 pageerror。

同一脚本在原 6346fc 上复现六项失败：规则 1.01，类别/忠诚 1.36。修复后分别为 10.43、6.64、7.74。hover、文字 reader 与原图 caption 的六份计算样式逐项相同。原图未修改。

输出落点为原 Site `appgprj_6abf7bf54a7481918a50e1ef1509ca68`。按原 Sites 内部源码、构建、保存、部署流程操作；实际版本与部署回执记录在 `/workspace/.private-validation/archive-detail-contrast-hotfix-20261004/`。原 GitHub 不新增公开分支写入。

本地 JC004 独立候选继续保留，本次不包含它的卡池、版本、规则或 JPG。rust-v0.2.9、九个 WASM、Worker、D1 和公开权限均保持原版。公网新主题验证与本地 UI 颜色回归分开记录。
