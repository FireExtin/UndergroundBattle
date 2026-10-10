# 暂停与续局：Site40 已发布

输入是父端对完整候选945e8b2c8b96e2d75fcb709db72c7a1e9e4ad635的独立源码验收，以及既有 Sites 更新授权。父确认无发布前必修 P0–P3；真实并发竞速可另补测试。本批精确发布暂停 engine46 / pool42，不包含封印47或搜索48工作进度。

输出：Site40 在 https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site 上线。官方 source helper 正常推送57039066c304db8ccd51155ecf2941b91df3abbd；保存版本appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_1c6a8132f7f8819184966311d01d6633；部署appgdep_6ac67ef0ad888191b4da536a8e61dfad于2026-10-07T17:18:51.687668UTC直接返回succeeded，无失败，env revision0。项目、公开访问范围和网址保持原值，未清空任何线上牌桌或升级数据。

发布前复核371个产品源码/测试/原图文件与审查提交逐字节一致；真实 TypeScript、Vite 和 Wrangler dry-run 构建成功。归档46,032,169字节，SHA256 a444b3b1351e724fac9680228d3d77862e9f9038a2f1ade76682bcf9b6b1721a，127个文件，唯一WASM2,209,157字节且SHA2569e6362c967046ff245b94ab650bc9ed0f3af32e45cc4ca979305b3921543a2a9与独立审查产物一致。Sites保存服务的unpacked representation为62,648,320字节、SHA25600b20b9238da7feb96bdd942f95821216b4b73d1e865f7501fab09a61c943292；不是本地gzip文件哈希。归档保存在/workspace/game-publication-evidence/site40-pause-945e8b2.tar.gz，没有数据库、Git、依赖或座位凭证。

实际公开浏览器检查通过四项：两位玩家自然UI创建/加入/准备/开始；私有选择者在选择弹窗内暂停，暂停期间3.5秒无自动state请求；返回大厅、刷新并从“我的牌桌”恢复原席和原选择；同伴恢复后手动同步，选择ID和手牌实例一致，随后实际提交“保留全部手牌”成功。真实服务器revision5暂停→6恢复→7继续选择→8再次暂停。测试桌ee7077836a199dcdb3719d55最后留在暂停状态。使用一个普通Chromium中的两个隔离上下文，无owner/bypass凭证；数据操作仅为新QA桌正常游戏请求。四张实际截图保留，桌面暂停和恢复截图已实际查看。公网catalog与view均确认rust-v0.2.46-pause-resume-candidate / limited-v2.42-jc089-poison-blood-candidate。

失败轮不计入通过：早期两个独立浏览器启动发生进程退出；第一桌仅创建到等待加入的Lobby，邀请码015CEABA0239，尚未开始对局，没有可恢复席位凭证。改用一个正常多进程浏览器内两个隔离上下文后全流程成功。首次save参数误带checkout_path被argument_binding拒绝，去掉该不支持字段后正常保存；没有权限拒绝、绕过或重复发布。

旧桌tuple不兼容时现有UI显示“旧牌桌使用的规则已停止支持。请返回大厅新建牌桌；旧牌桌数据仍保留。”保存旧数据并不表示跨版本自动恢复。任一原席（包括仍有凭证的淘汰席位）可恢复；同伴需要手动同步，未声称全员ready或账号跨设备找回。暂停候选完整离线验收579 Native、620 Web、27本地Worker/WASM/D1、9 Native/WASM转换36视图是原独立审查证据，本次未重复全量执行。

官方 Sites 工具和恢复的0.1.75完整包已实际完成源码推送、打包、保存、公开发布；未重试原官方组件下载403，不能据此宣称该下载端点已修复。验证归档全部127文件与源构建/迁移文件相符后移除三个可重建构建镜像180,600,360字节，保留源码、Worker、固定归档和证据。这是云端文件系统空间，不是已验证的Library配额释放。封印/搜索14文件从a28163acccb42f1c8f4108a0f55023e5dd66d61e stash完整恢复并逐字节复核，stash仍保留；恢复后的封印47专项已实际19/19通过，余下完整生命周期、WASM和UI仍在研发。

证据在[发布收据](evidence/site40-live-2026-10-07/publication-receipt.json)、[真实公网结果](evidence/site40-live-2026-10-07/browser-results.json)、[归档验证](evidence/site40-live-2026-10-07/archive-verification.json)、[构建日志](evidence/site40-live-2026-10-07/production-build.log.gz)、[恢复WIP](evidence/site40-live-2026-10-07/restored-wip.json)。公网脚本同目录public-natural-ui.py会新建真实QA桌，应只在相应测试授权下运行；座位凭证仅留云端私有QA文件，不入Git/静态包。Github/main未推送。
