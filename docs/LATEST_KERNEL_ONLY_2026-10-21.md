# 当前ABI93单核清理（待独立检查与发布）

2026-10-21 16:30用户明确取消旧牌桌兼容，以空间和性能优先。本改动从固定 `9aa4fda499c25268a632fa578c31a04c70eac6e8` 分出 `progress/latest-kernel-only-20261021`；第一批、第二批原审查快照均未改动。实现及验证固定在 `c1e5e8dd79b4899b1cf9eb0f871bb861e4bdb10b`；后续证据提交只补充此精确绑定，不改变已测代码。

运行内核仍是ABI93 / `limited-v2.83-batch20-next` / `hegemony-pdf-v1`，WASM 4,315,979字节，SHA256 `098986aff958d97b5fb5f49339daf07586a200997bf49c179f424ea51dd04e7e`。Rust规则和卡牌数据未改，不新增兼容内核或状态迁移。

## 实际删除与保留

- Worker从16个内核减为1个，只导入当前JS/WASM。删除多版本选择、懒加载及重复的JS版本声明；当前身份取自Rust目录，打包时与源码cards.json的三项版本字段核对。
- 正式构建、后端测试构建均清空旧generated/dist，只准备当前模块。Sites源码同步删除旧副本中的frozen目录和旧loader，不再复制冻结输入。
- 服务端删除旧的非paced命令、GET与报价分支；当前客户端必须提交原requestId，服务端不再随机补旧客户端请求ID。CAS、原命令回执、可信时间、全状态opaque字符串和Rust恢复验证保留。
- 删除纯旧兼容测试、5个旧版Worker构建脚本及冻结模块staging函数；本地当前固定时钟wrapper只链接已构建的单核原字节。
- 旧桌、旧创建/加入/命令回执都返回410 `unsupported_room_version`并提示新建。测试检查五张D1表原值不变；没有删除用户数据库或将旧状态送入新核。
- 历史源码、身份与原件保留作审查证据；未改写历史opaque夹具。历史机制测试仍有只用于测试的原核loader，见下方未覆盖清单。它们不再进入运行包，也不是当前构建的前置输入。

## 体积、启动与内存实测

同一环境，旧候选先测3次，单核候选再测3次。每次创建新本地Miniflare/workerd进程与合成D1数据库，依次健康检查、执行数据库迁移、创建当前房间并读取状态；6次均成功。测量期间没有其他本任务测试或构建进程。

| 指标 | 9aa4fda多核 | 当前单核 |
| --- | ---: | ---: |
| WASM模块 | 16 | 1 |
| WASM合计字节 | 53,262,284 | 4,315,979 |
| generated输出字节 | 53,553,532 | 4,334,182 |
| dist输出字节（含静态客户端） | 184,528,955 | 134,836,916 |
| 进程启动到健康响应，中位数 | 1,120.84 ms | 278.96 ms |
| 启动、迁移、新房可读，中位数 | 1,374.47 ms | 559.14 ms |
| 创建请求耗时，中位数 | 31.00 ms | 24.87 ms |
| Node进程最大RSS，中位数 | 424,892 KiB | 239,260 KiB |
| 进程树采样峰值RSS，中位数 | 614,473,728 B | 360,513,536 B |

WASM减少48,946,305字节（91.9%），generated与dist合计减少98,911,389字节。静态客户端仍130,386,175字节，未改变。这里的构建空间是输出文件体积，不是构建过程瞬时磁盘峰值；新官方压缩归档尚未生成。三次采样不是线上性能承诺，RSS进程树求和可能重复计算共享页；原始范围、数值、测量脚本和日志见[verification.json](evidence/latest-kernel-only-2026-10-21/verification.json)。另清理的可重建旧Cargo缓存未算作单核收益。

## 验证与独审两项P2

| 检查 | 实际结果 |
| --- | --- |
| Web/Worker正式构建及当前身份核对 | 通过；只输出1个WASM，哈希仍为上述ABI93 |
| 版本门禁、产物绑定、HTTP/D1 | 4文件6项通过：55历史身份在解释状态前拒绝；15退休身份×4 HTTP入口共60次410；当前u64、并发、回执、重开、旧数据不变通过 |
| 服务层重试和响应时钟 | 2文件12项通过，保留原actor、commandId、expectedVersion与完整action |
| 当前目录/新房Web组件 | 36文件210个唯一用例：209/210通过后，修正MSJC07旧名单，该文件4/4补验通过；其余35文件未重复运行 |
| 旧桌失效与客户端恢复 | api/useGame/RecoveryReviewRegression/QuotaRetirementReload共4文件52项通过 |
| 来源/类型 | 当前秘社来源1项通过；Web类型检查通过 |
| 干净后端构建 | 全新目的地，generated、dist和frozen均不存在，仅输入当前源码/JS/WASM和已安装依赖；dry-run通过，1模块；实际本地健康检查、开房、读房通过 |

B2-IR-01（旧身份断言/遗漏93 pacing）：集成测试改为ABI93/pool83，测试助手只用当前响应协议，删除版本白名单。保留并发、回执、回滚、重开和u64检查。完成协议简化后另运行该文件，结果在[integration-single-protocol-final.log](evidence/latest-kernel-only-2026-10-21/integration-single-protocol-final.log)。

B2-IR-02（清空generated后缺92导入）：删除全部旧导入和旧模块准备，不补回92。正式及后端构建仅当前核，干净目录构建与实际启动日志分别为[backend-clean-build.log](evidence/latest-kernel-only-2026-10-21/backend-clean-build.log)、[backend-clean-start.log](evidence/latest-kernel-only-2026-10-21/backend-clean-start.log)。这是作者修复与复现结果，仍待协调任务独立补验。

历史失败记录未抹除：前端首轮因Vitest重写动态import而未执行用例，改为Node加载同一当前绑定；来源首轮缺本地秘社原图，核验现有原件哈希后恢复输入；第二轮的MSJC07断言失败及修正保留在[完整日志](evidence/latest-kernel-only-2026-10-21/web-current-catalog.log)。未通过放宽名单、禁用测试或重贴版本字段消除失败。

## 覆盖边界与剩余事项

本次不是全仓再审或当前版本全套通过，没有重跑Rust全套或35+8机制回放；Rust/WASM字节未改，原结果只按各自固定身份引用。没有新增真实浏览器对战，也没有由作者自称完成独立复审。

旧机制证据不能直接换成当前核运行：剩余24个Web ABI77 opaque轨迹、10个Web ABI78–89专用oracle、20个Site ABI77机制集成加pass-history，以及14个Site专用oracle仍需从当前Rust测试重新导出并对照。它们包含有效的选择、隐藏信息、恢复和回执断言，未当作“无用测试”删掉；默认全套仍含这些历史输入依赖。此次已切换的目录/新房测试中，保留的历史视图仅用于UI渲染断言，不是当前引擎回放。Sites独立源码再次同步/构建和单核正式归档还未执行；本次验证了产品仓库构建及全新后端测试目录。

v74部署 `appgdep_6ad8dd9acc1c8191912336351e022f25` 仍在官方pending，updated_at一直为2026-10-21T15:43:22.880209+00:00，尚无URL/provider ID/failure_message。当前工具仅能读状态与生产Worker日志，没有部署队列修复接口；继续记录原请求，不重发、不取消、不更改安全设置。

旧多核v75已经退出发布队列。新单核候选未请求发布，等待单核独立补验及既有部署终态后再沿官方Sites流程准备新版本；不修改v74/v75已保存快照。未运行GitHub CI，未合main、创建PR/tag/release或部署其他项目。
