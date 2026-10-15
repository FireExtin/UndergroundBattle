# Site54：ABI77串行整合批次的官方发布与PC线上验收

官方发布及完整PC线上1V1验收已完成。同一自有牌桌经历四次明确保留的脚本中止后续跑至自然终局：第6回合，5:8，team1胜，最终版本344。本文接续既有[串行整合材料](SERIAL_INTEGRATION_REVIEW_2026-10-15.md)和[JC046两个P2修正](JC046_P2_VALIDATION_FIX_2026-10-15.md)，不改写其历史测试结果，也不把中止尝试标为通过。

输入为父会话明确通知：`4918cfb6372fb9ec900f037e8ed518385d6183e6` 的独立补审通过，两个P2关闭；沿用户既有授权发布到原公开Site，再做实际线上PC验收。本线程负责实施与验收，不冒称独立复审者。全程单线程，没有子代理、worktree、main合并、强推或交易所部署。

## 实际发布身份

| 项目 | 实际值 |
| --- | --- |
| 公开地址 | https://hidden-world-hegemony-20261002.chengliang1984286.chatgpt.site |
| 游戏源码候选 | `4918cfb6372fb9ec900f037e8ed518385d6183e6` |
| GitHub审查分支 | `codex/hidden-world-serial-integration-review-20261015` |
| Sites源码提交 | `cec2bd64b847d47ac971a459dc8ef2d61aec9e89` |
| project_id | `appgprj_6abf7bf54a7481918a50e1ef1509ca68` |
| 版本 | **54** |
| version_id | `appgprj_6abf7bf54a7481918a50e1ef1509ca68~appgver_42a67f56836c8191a1a053c3dee65514` |
| deployment_id | `appgdep_6ad09dcf575c819182ed0f3340eccfb1` |
| 原生部署结果 | `succeeded`，2026-10-15 09:33:29 UTC |
| 在线引擎 / 卡池 | `rust-v0.2.77-jc046-mandatory-sacrifice-candidate` / `limited-v2.70-jc046-mandatory-sacrifice-candidate`，目录143卡 |

实际使用已核验的官方 `site-workflow.mjs` 打开原Site源码、普通push、构建并打包；之后调用原生 `sites_save_site_version` 与 `sites_deploy_site_version`，各一次，均获得明确成功结果。保持原公开受众、项目身份、DB绑定及环境revision 0，没有使用私有发布或owner bypass。环境断连提示期间仍可执行真实shell及连接器请求，未因提示重复发布。

## 产物与归档

三份WASM分别位于原仓pkg、Sites generated及Worker输出，均为 **2,909,006 bytes**，SHA256为 `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c`。Worker入口SHA256为 `e9a2282ea515402467c0f6dacdd695f9494e4141332e9764c950a1aa589a64e6`。

前端完整清单为[158个实际构建文件](evidence/sites-abi77-2026-10-15/client-files-158.json)，合计93,201,591 bytes。逐一核对路径、长度与SHA，覆盖原 `web/dist`、`sites/public`、`sites/dist/client`、独立Sites构建以及最终官方归档，没有以源码Git树代替产物清单。归档中的hosting与3份迁移文件保持原Site53身份；无数据库迁移或旧房间数据删除。另从两个精确Git提交读取并逐字节对照[820份源码/输入/锁文件](evidence/sites-abi77-2026-10-15/source-provider-binding.json)，覆盖Rust、WASM、Web、Worker、卡定义、构建与JC046测试；大fixture先从provider gzip恢复，原始字节相等。两份package JSON仅新增已列明的恢复hook，其余JSON完全相同。

本地tar.gz为68,894,798 bytes，SHA256 `8ef5cd675ffdb01cec3ec7230d370c88bc98d761ad0438796eea0c6ad7992cea`；本地解压tar为96,378,880 bytes，SHA256 `931304cf93780261fffe5a64dd0deeb9f600f5fd48c2bc6c824430616f603eff`。原生save返回96,378,880 bytes / 166文件及 `content_hash=sha256:8266fedc8816af615f1d882aeb0f2a583c46521cc56a2c6be348f088dccb0881`。平台未说明该字段的计算语义；它与本地压缩/解压tar整体SHA分开记录，不主张二者整体字节相等。部署前逐文件的Worker/WASM/158前端归档门已通过，线上全部158前端响应字节亦一致。服务端核通过provider源码提交、已保存版本及归档核SHA与线上ABI/catalog绑定。见[编码层记录](evidence/sites-abi77-2026-10-15/archive-storage-binding.json)、[归档核验](evidence/sites-abi77-2026-10-15/archive-verification.json)及[原生发布结果](evidence/sites-abi77-2026-10-15/publication-result.json)。

## 源码存储上限的实际修正

首次官方helper源码push被 `artifacts_git_receive_pack_object_too_large` 拒绝；当时没有调用save/deploy。原因是新增JC046两份同内容测试fixture形成41,911,207-byte单blob，并非权限拒绝。失败提交 `b456b4cdb290c3aa74bd09084377a11391f2c90e` 保留在独立Sites普通checkout的本地保存分支，未强推或覆盖远端历史。

实际发布的Sites源码将该测试数据以3,067,696-byte gzip存储，通过 `scripts/restore-test-fixtures.mjs` 在build/test前按原始长度与SHA恢复两条JSON路径。原始JSON SHA256为 `a6e3bc6c1ca236d908912320471b51c231a6bc8067fbf9d05f87ea70ee963601`。GitHub候选仍保留原JSON；游戏、Web与Worker源码逐文件相同，构建输出再次全部匹配。这里改变的是Sites源码内的测试数据存储和恢复hook，没有改变测试输入/期望、规则实现或生产输出。

缺文件恢复、重复执行和错误身份拒绝3项检查通过；Sites副本上JC046 **38/38 Worker**、**6/6 Web**通过。前端首次命令重复传入已有的`--run`参数而在CLI解析阶段失败，去掉重复参数后上述6项实际执行通过。上述数字是本次定向执行，不与早前1169/1145/179整轮或P2的45项相加，也不宣称重跑了当前全部180项Worker。

后续从GitHub重新运行旧 `stage-source.mjs` 会原样复制大fixture；发布时须保留或重做这层无损存储，不能通过省略输入或修改SHA接受不同数据。实际Sites源码中的恢复脚本、清单、gzip及npm hooks已随 `cec2bd64…` 推送。见[恢复器校验](evidence/sites-abi77-2026-10-15/fixture-storage-verification.json)。

## 线上PC实际结果与边界

普通公开Chromium，1180×760，独立持久浏览器席位；无owner bypass、固定种子、中局注入、直接行动API或其他玩家凭据。线上health/catalog确认为77 / 2.70 / 143卡，158个公开前端文件的实际响应字节SHA全部匹配。新建的自有1V1桌 `fd08636bd17bf0e956e194d1` 绑定同一ABI77身份。

已完成真实点击的选择/取消/换选、刷新保留席位与待选草稿、暂停后刷新与恢复保留选择，以及快速双击确认只发一个命令、服务版本只增加1并清理完成草稿。实际验证暂停版本7→恢复版本8→确认版本9。对手暂停页在另一席恢复后通过正常刷新同步，未宣称暂停时自动轮询。

完整对局已在同一桌自然结束，最终 `status=finished / version=344 / turn=6 / winnerTeam=1`，比分5:8。总共341个实际UI POST：创建/加入2个，行动命令339个；行动中337个200，另2个为响应期限已过的 `409 window_expired`，都附带更高版本的权威视图。两次对应HTTP409控制台记录明确保留并归类，0页面脚本异常、0其他浏览器错误；没有把整局描述成所有请求均为200。

真实点击包括16次“动作组→目标/地区→确认”，14次选择涵盖再调度、地区回收/排序、触发、目标、搜索和调查，另有资产、派遣、秘密派遣、出牌与实际响应按钮。最终成功进程退出码0，`passed=true / completeGame=true`，终局截图已实际打开检查。截图和浏览器profiles保留在本地，Git只保存公开投影与必要摘要。

完整[线上机器摘要](evidence/sites-abi77-2026-10-15/online-complete-summary.json)、[原始逐条行动](evidence/sites-abi77-2026-10-15/duel-actions.jsonl)、[最终进程回执](evidence/sites-abi77-2026-10-15/full-game-summary.json)及[PC检查](evidence/sites-abi77-2026-10-15/online-pc-checks.json)可复核。341个POST为同一桌完整网络事件流，已与五段进程各自新增请求及339条命令逐一计数对齐，不是把不同测试轮的通过数相加。

四次验收脚本中止均保留[原始摘要](evidence/sites-abi77-2026-10-15/interrupted-attempts)并在同一自有桌续跑：第一次脚本等待已暂停的同伴页自动轮询恢复，现有产品会在暂停后停止轮询，改为同伴正常刷新；第二次旧脚本还寻找独立目的地按钮，已改为当前真实UI的“动作组→目标/地区→确认”点击链；第三次旧续跑逻辑只等catalog就读取同伴视图，现明确等待其原房间视图就绪，无新POST；第四次是上面第一条真实 `window_expired` 到期拒绝，后续只对这类明确错误及已前进视图按正常UI继续，其他拒绝仍失败。后一条到期拒绝发生在最终进程，已实际同步后继续。源码和候选产物没有为适应脚本改变，中止摘要仍为 `passed=false`。

本轮只完成PC下的一局自然1V1完整验收，未扩展成线上四人或全卡池机制验收，也未定向保证JC046/JC060/JC095在本局均自然触发；机制专项仍依据各自已审Native/WASM/Worker/Web证据。手机适配仍未完成。ABI74历史重建hash疑点未解决，旧房间unsupported/410仍为兼容边界，未迁移、删除或借用旧房间凭据；本轮不冒称已逐个在线验证旧房间。中立双色继续暂停，BQ035仍不在本批实现与准入范围。

Git证据仅含必要文档、校验清单、脱敏日志与摘要。构建包、WASM、浏览器持久profile、数据库、缓存及短期源码写凭据均不提交。
