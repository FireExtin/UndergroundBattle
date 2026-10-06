# 游戏根 Sites 源码同步候选（2026-10-06）

此提交从父已审游戏核心 `db71d611cfa9f3615a61dfff528e6c3ca2b57a31` 建立独立工作树，将父已审独立 Sites 候选 `50103291eab8c68ad862ee1bcca5f3e55568cf1d` 的有限适配映射回游戏仓。它不合并两个仓库的根目录，不复制第二份 web/Rust 源，不改规则、原生服务入口、数据库结构、依赖锁或发布配置。本提交待父独立审查，未写 main、未推送、未发布。

## 最小映射及实跑发现

| 已审独立 Sites 源 | 游戏仓目的地 |
| --- | --- |
| src/kernel.mjs | sites/src/kernel.mjs，逐字节一致 |
| src/kernel-router.mjs | sites/src/kernel-router.mjs，逐字节一致 |
| src/lazy-kernel.mjs | sites/src/lazy-kernel.mjs，逐字节一致 |
| scripts/build.mjs | sites/scripts/build.mjs，逐字节一致 |

`sites/scripts/stage-source.mjs` 将固定1..10复制改为已审 build 使用的历史目录枚举；`build-backend-test.mjs` 同样枚举，并采用现有游戏根/独立根选择方式。没有新增绑定、身份、队列或构建框架。

此前只读目录审计漏报了干净 db71 中的四组材料。首次真实根构建在11、12、13及25-resource-policy的 WASM/JS 导入处报 ENOENT。因此除了原计划的34，本提交精确补入这四组已审501冻结文件，共五个目录、20个文件；全部取原字节，没有重建历史引擎。旧历史文件继续保持 db71 原字节。已有25的 README 与独立 Site 文档不同，保留游戏仓原文；对应可执行 ABI 一致。

## 已完成的本地验证

- 游戏根 `web/` 与 `sites/` 的锁文件安装成功。初次 offline 安装因缓存缺项 ENOTCACHED 失败；在可写任务缓存中按原锁安装成功，独立源码包随后可用该缓存离线安装。没有修改依赖或锁文件。
- 在游戏 `sites/` 运行 `npm run build` 成功：原 web 类型检查与 Vite 构建通过，Wrangler仅 dry-run，输出38个 WASM；原生 cargo/web 的所有既有源码、入口和配置字节未变。
- 根目录 `node sites/scripts/build-backend-test.mjs` 成功；由游戏根实际 `stage-source.mjs ABSOLUTE_DESTINATION` 生成的独立源码包，其 `npm run build` 和同一后端 helper 均成功。后端 helper 没有构建前端、没有部署。
- 四项定向 Node 验证全部通过：两个已审 lazy/router 测试、38套真实 ABI 路由/152个座位视图、五类 HTTP 新旧房间（36、35、34、25-resource-policy、25）的10次写入和5次真实 Worker 重开，包括精确重复回执及409无旁写。它们是从已审501测试中提取的定向检查；运行脚本与日志在独立审查材料中，没有扩大生产 diff。
- 根后端、独立源码包正常构建、独立源码包后端三套实际产物均启动本地 workerd，health/catalog共6次 GET 均200，确认 engine36/pool33、100张普通卡含JZ55；未接触线上房间。
- 三套产物的 `index.js` 及全部38个 WASM 均逐字节对应已审501产物。Worker运行代码 SHA256：`3d049cdd47e136fdd6b4e4d2bf14500a06601d2e0dc2a36c22f5822505f3d2a6`。源码地图、生成 README 受输出路径影响，不作为运行代码一致性的断言；前端112个产物文件与已审501全部一致。
- 两个源码打包保护测试通过：拒绝仓库内目的地；拒绝其他 project 的目的地且保持其文件不变。
- 原 db71 除五个授权修改的既有适配/脚本文件外，其余2784个已跟踪文件逐字节保全；新增 lazy 文件精确映射。最终源码包的来源提交在提交后重新生成并单独核对，不使用脏工作树的旧 provenance 作为候选提交证明。

当前 WASM SHA256仍为 `0f3a637d854074fe0689927c51e7969e8a32551ddcb48e1e1f3f1b6d022ea036`。当前身份仍为 `rust-v0.2.36-jz55-unique-destroy-candidate / limited-v2.33-jz55-unique-destroy-candidate`；没有增加卡牌或规则版本。

## 入口与验证限制

游戏根构建/打包命令：

```sh
cd web && npm ci && npm run build
cd ../sites && npm ci && npm run build
node scripts/stage-source.mjs /absolute/separate/site-checkout
HEGEMONY_BACKEND_TEST_ROOT=/absolute/private/check node scripts/build-backend-test.mjs
```

独立源码包可执行 `npm ci`、`npm run build`，以及设置私有输出目录后的 `node scripts/build-backend-test.mjs`。`stage-source.mjs` 的再次打包入口仍约定在游戏仓 sites 下运行；未增加独立源码包再次打包支持。它消费游戏仓已有 pkg，不提供该有限副本的完整原生工作区/WASM重建流程。

本次没有宣称全套 `npm test` 绿色：仓内旧 `test/kernel-routing.test.mjs` 实跑仍断言 engine11，与当前已审36不一致，这是原有测试版本未更新的失败。本次保留原测试，日志列出明确差异，使用上述定向实核验证同步。全套测试需要独立更新旧用例，未执行全套回归。

`build-backend-test.mjs --society-fixtures` 仍需预先生成的独立 `pkg-society-fixtures`；当前冻结材料没有此包，实跑在其 JS 路径报 ENOENT。因此没有声称该模式通过，也未为本次源码同步新增或重建测试专用规则核。

官方 Sites 包下载的 HTTP403仍按父指示暂停，没有重试或借其他执行器转包；所有 Wrangler调用均带 `--dry-run`。正常原生 `cargo run -p hegemony-server` 仍使用原配置与源码，本次没有重新编译或运行 Rust规则测试，字节保全作为未修改的证据。engine36是规则核身份，不能称作已发布的平台 Site版本。

完整差异、增量 Git bundle、五组新增冻结文件、源/产物哈希映射、打包清单、成功及失败日志均在本次独立审查包。
