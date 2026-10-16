# 让过到指定阶段：精确审查材料

- 源码：`7ce4cb4d3b1af7bcd5502dc4f96e522d70cabc08`。
- Web tree：`84d9e77f3f592e015dd0bfec807d82f4917e36f4`。
- 分支：`codex/stage-pass-progress-20261016`；旧共享分支 `ac831db` 仅为早期检查点。
- 状态：开发验收完成，待父会话串行独立复审；新功能未发布。

## 文件与对应范围

| 文件 | 用途 |
| --- | --- |
| `validation-summary.json` | 机器可读源码、结果及限制；各轮结果分别列出 |
| `final-full-vitest.json.gz`、`.log` | 原始全套 1195/1195，126 个测试文件；JS/TS 为 `19b07116f1f080a9ebd18469538809d1a27defe8` |
| `post-full-web-delta.patch.gz` | 全套之后到最终源码的全部 Web 改动：点选处理、4 项新增测试和手机 CSS |
| `manual-p2-vitest.json.gz`、`.log` | 最终源码的 95/95 定向；包含手牌/场上牌/地区在 300 毫秒点选取消和悬停保持模式 |
| `manual-p2-build.log.gz` | 最终源码 TypeScript、Vite、Sites Worker dry-run 构建 |
| `final-source-binding.json.gz` | 551 个相关源码/配置文件哈希，与最终提交逐字节匹配 |
| `final-artifact-manifest.json` | 实际浏览器使用的 client、Worker、WASM 文件哈希，无产物本体 |
| `browser-result.json` | 最终源码真实 Worker/D1 双端 16 项检查、103 个命令及 103 个 200 回执；不含鉴权头、座位凭据或入桌码 |
| `desktop-active.png` | 桌面实际开启状态及取消入口 |
| `phone-mandatory-stop.png` | 手机第二回合手牌上限强制选择；不会自动选牌 |
| `prior-debug-only/` | 已解决的中间失败，不能算作通过结果 |
| `evidence-file-hashes.json` | 证据文件校验；不包含自身 |

JSON gzip 保留原始字节，解压即可检查。Vitest JSON 中 `numTotalTestSuites` 统计含描述分组；本说明的“文件”数使用 `testResults.length`（126 或 5）。1195 与 95 不相加，也不声称最终源码完整重跑 1199 项。

## 已解决失败的记录

`busy-uncertain-failed-vitest.json.gz` 是中间 70 通过/1 失败的集成运行：上层把未确认同时标记 busy，曾导致连续模式没有停止。最终改为 uncertain 无条件停止，3 项真实 GameApp/api/useGame 管线恢复回归覆盖同一命令 ID 重试、刷新恢复旧命令及模式不重启；它们包含在最终 95 项中。

早期浏览器运行按发生顺序保留：

1. `browser-candidate`：测试驱动同时准备引发版本竞争；改为看到双方同步版本后顺序准备，没有修改产品准备逻辑。
2. `browser-candidate2`：测试驱动假设房主先手；改为读取实际行动队伍。
3. `browser-candidate3`、`browser-layout`：新控件使手机固定手牌区增高，旧详情面板实际被遮挡；加入实际高度测量和正确优先级的手机定位。
4. `browser-final`：手机选地区仍被详情遮挡；压缩目标面板、保留来源与确认/取消，并实际检查地区中心命中。
5. `browser-targeting`：真实 `passResponse` 已成功且效果立即结算，测试仍等待短暂的 `passed` 展示；最终检查对应命令的 200 回执，不再依赖瞬间状态停留。

这些原始中间结果没有完整通过，只有本目录 `browser-result.json` 为最终 16/16。最终测试驱动位于 `tools/cloud-playtest/stage_pass.py`，硬性限定 localhost；所有测试牌桌、姓名与命令来自自建隔离环境。

## 复现及边界

用 `node tools/build-storage.mjs --` 运行测试和 Sites 构建，然后以 `sites/scripts/qa-preview.mjs` 启动自有 localhost 持久 D1 环境，再运行 `python tools/cloud-playtest/stage_pass.py --source <SHA> --output <新的目录>`。构建前应停止该测试服务，防止混用旧资源。不要将脚本指向正式站，也不需要真实用户桌或凭据。

引擎、Worker、数据库结构、WASM 均未修改；未为未变化部分重复运行 Native/WASM/Worker 全套。不将两回合局部验收称作完整对局，不将本地浏览器称作线上试玩。正式 v57 身份和证书阻断单独记录，新功能必须另经独立复审及发布授权。
