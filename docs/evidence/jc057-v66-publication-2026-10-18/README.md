# JC057 / ABI84已审v66发布证据

固定输入为代码5a1a40d、证据c674946；父完整串行独审已通过并关闭P2。官方source7ab9768、v66、deployment6ad43cd发布成功，身份/原生回读见official-receipts.json和publication-summary.json。

production-preflight.json与publication-archive-audit.json核对177项产物/174运行文件，归档只额外带三份原Drizzle文件，总计180。上传gzip和规范化PAX哈希、大小、计数与官方保存回执一致；八个WASM包含原77–83，生产Worker是真实时钟，测试Worker未入包。provider-copy-plan.json.gz和provider-source-files.json.gz是有限来源映射和内容哈希，不是数据库或缓存。

provider审计1244项Git blob及两份按原gzip恢复的忽略夹具。archive-first-audit.json保留额外源码审计首次错误假定解压目标应被Git跟踪的失败；按既有恢复约定核对原字节后通过。配置/迁移/恢复钩子原样保留，历史provider缓存未变且本次无新增缓存、数据库或凭据。

normal-browser.json、脚本、日志和截图是v66成功回读后的唯一普通Chromium导航：03:29:24Z仍ERR_CERT_AUTHORITY_INVALID，1GET/0游戏写入/0完整对局，已停止，无证书绕过。没有红黑蓝灰线上试玩通过结论，中立双色暂停。

recovery-preflight.json记录执行连接恢复后的只读状态、原有17份未保存材料、隐私筛查和实物复核。锁仍为空文件，但恢复前后mtime/inode不同，已如实记录；本轮保持恢复后的文件不动，不声称跨恢复身份全同。上级两篇说明文档解释完整发布及收尾边界。

大归档、旧v65目录及工作材料留在`/workspace/jc057-publication-20261018/`，不入Git。此次保存不重复发布、访问网站、合main、清库或触发Actions。
