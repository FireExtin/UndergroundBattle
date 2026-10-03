# Site13 / v027正式发布检查点

输入：已审候选`37a09a2cc0088845657eafd7088f54a2bbc19012`，生产与`be93de3805ee53676242043ccb4bd91137a6589a`一致；用户于2026-10-03 01:26明确允许先更新原公开站点，再进行公网四人验证。父线程转达该确认，解除待确认暂停。没有更改候选机制、追加v028、启用零中立研究预组或新裁定。

最小环境核对通过，远端主实施仍为fd2f1fd，现有Site12源仍为1bc83a0；复用已有环境及官方bundle，未重新搭建。主实施分支`codex/hegemony-playable`正常快进到已审37a09a2；生产源manifest、当前v027 WASM和六个冻结核与父审候选逐字一致。

官方`site-workflow.mjs`先打开原Site，随后执行官方build-site构建、精确源码推送、归档。TypeScript/Vite/Worker构建通过；97原生、160前端、16七核Worker及591步/1889投影一致性均复用输入未变的已审结果，不重复跑本机整局。

发布结果：

| 项目 | 精确值 |
| --- | --- |
| 实现源SHA | `37a09a2cc0088845657eafd7088f54a2bbc19012` |
| Site源SHA | `197fba9ca5dec95c09a68ddb4066c9f246a4a548` |
| 原projectId | `appgprj_6abf7bf54a7481918a50e1ef1509ca68` |
| Site版本 | 13 |
| 保存versionId | `appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_7a8842419c948191b2360f2ce75b2186` |
| deploymentId | `appgdep_6ac05ae6758c81919382cf3d1073f441` |
| 状态/时间 | succeeded，2026-10-03 01:31:28.075103 UTC |
| 当前WASM | `21169e9d579886a55474b21aa506a4a9adca779428d7d11f2e966ae077656198`，1,593,545字节 |
| 保存归档 | 57文件，服务端tar 24,381,440字节，content hash `00729ca7dcd66c6cdcc501e2fd453f727df7c9f706000ab42eb748263b882814` |

正式URL来自成功部署结果：<https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site>。public受众、访问revision2、环境revision0、原D1/DB及所有旧房保留；没有写旧房或更换其凭据/测试token。六旧核继续按各房完整版本元组路由，不迁移原房规则或数据。出现v027新房后，后续修复/回退也必须保留七核。

边界：此为发布检查点，**公网新v027四席墓地有限切片、刷新重连及旧房只读兼容验证尚未执行**。此前本机897次UI请求和902条journal通过不能代替公网通过。下一阶段沿用同一实施线程，在新建明确QA桌、四个独立正常UI身份下验证实际墓地付款、目标/席位归属与刷新恢复；旧房仅只读。发布成功即向父线程返回精确三项标识，再继续该阶段。

输出：原Site13成功部署、本说明和本机私有`v027-site13-release-20261003.json`。归档保留在`/workspace/.private-validation/hegemony-v026-fast-evidence/site-v027-official-20261003.tar.gz`；源repo写凭据只在会话内存及官方隐藏stdin中使用，没有保存或打印。
