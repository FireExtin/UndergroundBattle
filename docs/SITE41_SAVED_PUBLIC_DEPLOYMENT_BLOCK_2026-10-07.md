# Site41 已保存；公开部署被自动审批阻止

后续状态：父端补充用户AGENTS.md持续发布授权的原文证据后，同一deploy_site_version工具仅重试一次，版本41于2026-10-07T19:53:08.809280UTC成功发布。以下保留首次阻断和保存阶段的历史记录；当前状态见[Site41发布说明](SITE41_SEALING_SEARCH_PUBLICATION_2026-10-07.md)。公网catalog403仍未重试。

父原审查者通过2ad04f7b91ca3d6030ff27a5d45598703592fbde的P1修复复核，并在后续调度消息中要求发布组合候选。本任务按官方Sites0.1.75流程重新get_site和取得临时源码凭证，复用原checkout，没有新clone/worktree。原项目appgprj_6abf7bf54a7481918a50e1ef1509ca68、owner和public范围均确认，开源助手前线上版本为40。Github/main未推送，线上牌桌数据未清理。

官方source helper打开57039066c304db8ccd51155ecf2941b91df3abbd，逐字节同步**455**个已审产品/测试/原图文件，恢复Git文件模式，真实TypeScript/Vite/Wrangler构建通过，正常推送官方源**42229677a9a69fc6b1d469af0f6539ac03bb429b**。真实WASM2,248,074字节/SHA2569b8a7a69cd3c09781387cb4e66ad710d803783447fb6b0382fb2697eac50e885，与原审结果一致，engine49/pool44，107普通卡/8会社。

官方打包归档/workspace/game-publication-evidence/site41-sealing-search-2ad04f7.tar.gz为**49,397,362字节**，SHA256**5d3747db730821554d3438726d5631127e28c861de823d68bb1be29f75db9e5a**。全部130个文件逐字节对应构建和迁移源码，只有一个WASM。首次检查错用迁移文件原地路径；官方packager将root/drizzle复制到dist/.openai/drizzle，修正该映射后全部通过，部署前已完成核验。保存服务返回的解包格式为tar、67,133,440字节、hash sha256:8c7a85f36d3eb711f3cd9ddff762d26694a601eec562ec692b03c0191853d868，与本地gzip hash不是同一表示。

save_site_version成功保存**版本41**，ID **appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_28b65a6c2bd08191bc711cf61449a03e**，source42229677a9a69fc6b1d469af0f6539ac03bb429b，deployment_id为空。随后sites.deploy_site_version被自动审批拒绝：该动作将版本41部署到公开生产Site，但最初用户“本阶段不发布”限制仍有效，父审查/assistant消息不被视为可信用户发布授权。**没有公开部署成功**，没有改用其他部署工具或间接推送绕过。已向用户明确请求本阶段发布版本41的授权，等待回复；官方skill不要求额外确认，本次阻塞来自自动审批。

另一次只读HTTP GET https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site/api/catalog返回**HTTP Error 403: Forbidden**，已停止该请求，没有通过浏览器或不同网络工具重试。不能据此确认当前公网catalog或声称上线检查完成；原组件下载403也未重试。

可独立完成的本地真实UI检查通过四项。运行相同生产Worker、WASM和dist/client，独立本地D1，无fixture响应拦截或棋盘注入。一席通过实际UI创建、准备、开始、私有mulligan暂停；另一席仅通过正常HTTP加入/准备/恢复。暂停3.5秒无自动state请求，返回大厅/刷新/我的牌桌恢复原席和原选择，peer恢复后UI手动同步保持选择ID及手牌实例，真实提交“保留全部手牌”并再暂停，revision5→6→7→8。本地QA桌a235ecfb78e75a0c9527a1e7，未写生产DB。实际catalog确认XQ40催眠术表演者、XQ41启迪之梦、XQ45破颅而出、JZ50墓穴食尸鬼收录，三张实际桌面截图均已查看。这不是公网验收，也不是三封印卡/JZ50自然机制全流程。

本地失败轮保留：Wrangler默认配置目录/home/agent/.config/.wrangler不可用退出，随后采用checkout可写XDG配置成功；普通双context Chromium在首次goto退出，单进程双context在new_page退出，均不计pass。最终使用一个context/真实UI席位加另席正常HTTP命令通过，没有重新安装浏览器或使用公开站点bypass凭证。测试座位令牌只在测试过程/浏览器内存及本地应用DB中，不写证据文件。测试后仅停止owned8113服务器，其他5189服务器保留。

为生产构建所需空间，把683个旧Native展开轨迹完整压缩归档并逐文件验证后删副本，另删除未运行旧可重建审计/fixture emitter二进制（源码、依赖和当前Native测试保留）。两份前端镜像122文件全部核验与dist/client相同后释放129,299,462字节；保留完整dist/client、Worker、正式归档。此为本云端缓存，不是Library配额释放，不涉及线上数据或独立历史源码清理。

可直接续接原保存版本41的deploy，不必重新build/save；需先取得可信用户对本阶段公开发布的明确回复。部署后仍需实际线上检查；原审626 Native、33 Worker、22相关Web/类型检查、6311命令及25404视图证明保留在P1候选材料。历史依赖清理继续暂停至发布完成，BQ104/XQ48仍等原用户裁定。证据在[evidence/site41-saved](evidence/site41-saved-2026-10-07/site41-save-and-deployment-block.json)。
