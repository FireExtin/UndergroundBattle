# BQ035 蓝色幽冥骑士：机制实施中间保存点

输入：父会话要求继续红、黑、蓝、灰完整机制小批次，中立和双色暂停，未审内容保存在现有独立进度分支。本批承接 `BLUE_NEXT_BATCH_DESIGN_2026-10-15.md`，未重复实施已完成的 JC037／JC060／JC095／JC046。

原图及霸权 P18、P21 已实际查看，并读取旧手册和 FAQ 的机动／威名条目；确认费用5、蓝忠诚1、血领域1、吸血鬼／猎手、防御3、永久战斗2／势力1、临时调查1。只修正 BQ035 来源规格的忠诚遗漏，不改来源记录的 `selectionEnabled=false`／`gameAcceptance=notAccepted`。

当前写入：整卡定义、两项当前场面持续查询、对抗开始统一捕获机动及 BQ035 历史授予见证、完整移动程序与声明／待选／响应帧守卫，以及相关来源专属验证器的有限委托。候选身份为 ABI78／pool71，144 张目录牌。原图按哈希原样加入，旧扫描记录保留。

截至 2026-10-16 11:20 UTC：首轮 `cargo check --locked -p hegemony-server --lib` 通过。BQ035 定向 Native 曾有 22/22 通过；随后连续 Room 单测通过，包含 11 条情景、309 个真实 Room 命令及每步恢复／全席视图／journal 重放，3 次预期命令拒绝。新卡正面组件 3/3、保留内核输入检查 5/5 通过。这些是分轮、重叠范围，不能累加。

第二次完整 Native workspace 命令在 lib 阶段结束：**1121 通过、5 失败**，后续 integration 未执行。失败为 attachment 普通移动、engine board move、engine local targets、region aura moving 的机动事件来源／阶段校验，以及 catalog 134 对旧断言 133。原始日志 `/workspace/bq035-evidence-20261016/native-full-second.log`；不预先认定全部是 fixture 问题，更不能改松验证器以抹掉失败。第一次完整命令则因 examples 三处 SourceSnapshot 新字段编译失败，已补 None，原失败日志仍保留。旧 ABI77 前端 oracle 加载器两轮均未通过（Vite URL 改写后，当前仍报外部 file URL 模块未找到）；这不属于机制行为通过。

Native 首轮负例 helper 和连续 Room forecast driver 的失败及修正也保留原日志。source 全组首轮 43/44，修正 BQ030 历史基线排除新卡后仅该组 4/4 重跑，完整 source 组未重跑。当前最终的 WASM 等价、Worker/D1、前端整轮、production build 均未完成，不能发布。

兼容性必做：最小 ABI78／77 路由和 build 接线已写入，但尚未完整验证。v58 精确原 WASM SHA-256 为 `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c`。早期 WIP `855133b` 曾把配套生成 JS／gzip WASM 纳入 Git；根据父会话最新“排除构建产物”指示，本次删除这些运行文件，完整字节保留在 Git 外 `/workspace/undergroundbattle-build/frozen-kernels/abi77/`，仓库仅保留身份清单与哈希验证／staging helper，并忽略这些运行文件。不重写旧提交。缺少原输入或哈希不同必须失败，不下载、不重编译替换历史内核。默认 pkg 仍为已发布77，未生成候选78。旧测试已显式选择77 oracle，绝不能把这些检查称为78机制回归。

旧全量扫描生成器遇到已有卡的空 per-pool source 而中止，没有覆盖注册表；本批仅按来源规格有界追加 BQ035 原图和两个索引。后续来源测试须核对完整旧目录和所有既有扫描不变，不能把这次失败算成通过。

本保存点是 WIP，不是准入、独立复审或发布批准。正式站仍为父会话已成功部署的 v58；本批没有部署、main 合并、worktree、并行开发、数据库操作或证书绕过。线上试玩的证书阻断与本批离线实现分别记录。

11:17 UTC 收到新的优先需求：让过双方可见谁／发生时阶段／地区，无地区标全局，覆盖自动及响应窗口和双端刷新。当前 BQ035 工作完整保存后串行切换到该需求；未完成部分继续保持 WIP。10:30:57.167988 UTC 的正常浏览器线上检查再次得到 `net::ERR_CERT_AUTHORITY_INVALID`，未进入游戏、未发 POST、没有试玩结果，独立证据见 `SITES_V58_LIVE_CHECK_2026-10-16_1030Z.md`。
