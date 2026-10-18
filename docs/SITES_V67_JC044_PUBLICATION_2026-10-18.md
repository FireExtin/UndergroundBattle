# Sites v67：已审 JC044 / ABI85 完整批次发布

原站 https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site 已通过官方 Sites 发布 **v67 / ABI85**。原生部署返回 succeeded，版本、部署、provider源码、归档及公开权限回读一致。普通浏览器仍受证书阻断，**0 场线上试玩**。

父会话明确确认完整批次串行独审通过、原 P2 关闭、无剩余 P0/P1/P2 源码阻断，绑定源码 `f34424b8a699f74f742748a5cdbd4c73529bbe94`、证据 `f9b88a7a3d67e420edfc628cb6b11a03c49ea618`、运行清单 SHA256 `6123629e7508d08b059c8acdc93df6a03c0a68ff8f0bc2715e0f6c4f14c0b401`，并授权沿原官方站点发布。本次没有再修改规则、重建或重跑已审测试。

| 身份 | 精确值 |
| --- | --- |
| 项目 | `appgprj_6abf7bf54a7481918a50e1ef1509ca68` |
| 保存版本 | `appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_50183acdb5ec819187512206a7b0de9c`（v67） |
| 部署 | `appgdep_6ad46eaf94348191ba64eaa9364028cf` |
| 官方源码 | `db204017bf378fe068447a069acfb928854527bf` |
| 成功部署 updated_at | `2026-10-18T07:01:20.865335+00:00` |
| 引擎 / 卡池 | `rust-v0.2.85-jc044-time-repress-candidate` / `limited-v2.77-jc044-time-repress-candidate` |
| ABI85 WASM | 3263215B / `c02da9cbdfd99fcbfe3b4e0b82cc206ef8572edf2f58017ffd7a58bee2d0f9c1` |
| 生产 Worker | 193294B / `77139a8daecdfa5f733a7cdd8715e1892e2d779eef4d59d799df99d50d2972b1` |
| 上传 gzip | 78633235B / `c8df97296184398f82abf437c5630436d5c374149fb0c52f0b334d1c5eea4d2d` |
| 官方 PAX 归档 | 126423040B / 182文件 / `sha256:506e73a9fe7035f810f95226ecd8f69fd8dc9c416d18c6d17a02dc5196625d52` |
| 权限 / 环境 | public，访问 revision2；环境 revision0，均保持 |

发布前执行连接可用、无活动编辑或测试，游戏独立分支本地/远端均 f9b88a7，main仍398e220。960个源码输入与f344提交Git字节一致，179个生产输出逐项核大小/SHA，九个WASM包含准确新85及77–84旧八核。生产入口`createWorker(kernel)`使用默认`now = Date.now`真实时钟；固定时钟包装只核对、未入包。官方 source helper 打开原provider检出，确认原v66/source7ab9768未有未知推进，再按有限计划投影1262来源；133编译绑定全部匹配。

有限来源中1260项直接与新provider提交blob一致，另两份既有41MB夹具按原gzip/恢复钩子约定无损还原，原始SHA a6e3bc6c不变。两个package保留既有prebuild/pretest恢复钩子；hosting、Wrangler、D1配置及原迁移未改。历史647项provider缓存原样保留，本次未新增或删除缓存/数据库。官方helper正常commit/push并核远端HEAD，commands为空数组，复用已审产物；旧v66 dist整体保留`/workspace/jc044-publication-20261018/prepublish-v66-dist`。

归档182文件逐项核验：179项生产输出加3份原Drizzle文件，没有测试Worker、凭据、数据库或缓存。排序普通文件、默认零时间/身份的PAX规范化摘要/大小/文件数与官方保存回执完全一致；只部署这一个已保存v67。核验脚本两处比较形式过窄曾停止：先把真实时钟函数引用误要求成`Date.now()`字面量，再把官方带`sha256:`前缀的摘要与裸摘要直接比较。两次均只修核验形式，生产字节未改；第二次复用已保存v67，没有再次保存或重复部署。原始记录均保留。

部署与身份成功回读后，已跨入07UTC新时段，06:26的v66检查不算本时段检查。唯一普通Chromium151导航在 **07:03:12.045739Z** 开始，**07:03:12.295048Z** 遇到`ERR_CERT_AUTHORITY_INVALID`，随即停止；1GET、0POST、0入桌/建桌/游戏写入、0完整对局，未到界面、资源或目录身份检查。截图已查看，没有点警告、忽略TLS、修改信任、使用旁路令牌或重试。红黑蓝灰线上机制、选牌、连点与刷新恢复未执行；中立双色暂停，本地验证不算线上试玩。本07UTC时段不再重复访问。

完整[脱敏官方回执](evidence/jc044-v67-publication-2026-10-18/official-receipts.json)、[179实物和182归档核验](evidence/jc044-v67-publication-2026-10-18/publication-archive-audit.json)、[普通浏览器实际结果](evidence/jc044-v67-publication-2026-10-18/browser/normal-browser.json)以及[全部证据哈希](evidence/jc044-v67-publication-2026-10-18/manifest.json)已保存。原归档、旧产物和脚本在`/workspace/jc044-publication-20261018`；Git仅保存必要文档、无损压缩日志、合成来源/产物哈希和脱敏回执。没有手工推送provider、权限绕过、游戏main合并、Actions触发、并行子任务、额外清库或exchange部署。当前剩余阻断为正常浏览器证书错误，没有v67线上试玩通过结论。
