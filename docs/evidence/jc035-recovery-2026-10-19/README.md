# JC035新环境恢复WIP证据

验证代码固定为`5056e4e65f0e1edacf617ce12175d67f86eee0fc`。本目录记录恢复、构建、两项定向复验、完整Worker的原始中途前缀及自然终态，以及后续独立补验；当前补验尚未完成，不是完整资格通过或发布回执。

- 迁移包13,882,939B，SHA256 `02a4d8fb4ababa881589b80fff7252ac0406063f99defcd0d75574b468b75c43`；27个原始文件13,841,070B均验证并恢复，13个实际ABI目录匹配。见[runtime-restoration-verification.json](runtime-restoration-verification.json)和[runtime-abi-identities.json](runtime-abi-identities.json)。私有Library传输标识、签名URL、xattrs和二进制载体没有纳入Git。
- 生产构建自然通过；186个执行/静态文件与旧候选完全相同，唯一不同路径是Wrangler写入当前UTC的`server/README.md`。旧README字节未迁移、未重构或覆盖。原整清单对照的差异与生成器定位分别保留在[初次对照](production-rebuild-verification.json)和[差异分析](production-rebuild-delta-analysis.json)。本次187项清单见[production-recovered.manifest.json](production-recovered.manifest.json)，SHA256 `0ba0f278b10e65f71908acdb27b2c5366c3387aaa3616bfe676931577ceaaa77`。
- 标准Worker仍为`ba792492721a5bbf96a5af5bada80c9d3d3d72411b50d9aaaa92f7cd5184f15d`；本地测试Worker仍为`6dc7cf9e217f6be438b8b422e66a91ff725f837e45beacddab8a500a0108e567`，与原四客户端浏览器验证的字节相同。构建后空间为27.317GiB。
- `jc035-entry.test.mjs`完整文件1/1通过；四席命名完整牌组、六张起手、D1重开和原回执恢复均断言通过。原失败的黄色MSJC01只能凑39张；本次已有的单行MSJC03修正通过，没有新改断言、业务或预算。
- 原样`JC066 ABI86 HTTP target-reveal`单例1/1通过，19条接受命令、160次投影检查。原socket异常未复现，关闭根因仍未证实；没有添加重试或修改预算。
- 原完整轮720/722、2失败的日志及回执仍保留在旧证据目录，没有合计为全绿。本次完整69文件Worker于18:37:29 UTC启动，至2026-10-19T19:36:18.639891+00:00仅观察到60条通过、0条失败；JC035的13条正常HTTP轨迹已通过，47种坏档矩阵仍在执行。自然退出回执尚未生成。见[小时检查点](hour-checkpoint.json)和[原日志前缀](runs/worker-full-recovered.20261019T193618Z.partial.log.gz)。

已完成阶段的原日志以gzip保存，解压后的SHA256对应各自`.exit.json`中的`logSha256`，输入前后摘要原样保留。`record-run.py`是本轮实际使用的记录器，业务代码和测试没有修改。完整Native1358、前端1431、Native/WASM383命令/78坏档以及四客户端15命令的旧通过范围未重跑；编译141输入、实际WASM、Worker和前端资产字节均保持原身份。

待现有Worker自然结束后核对真实全量汇总、输入和候选清单，再补齐最终资格材料并交父串行独审。没有部署、合main、新worktree或历史内核重编译。

## 20:17首条新失败检查点

同一原运行已完成JC035的13条正常HTTP/D1轨迹和47种坏档检查，继续到JC041矩阵时出现`JC041 corrupted discard-6 restores HTTP400 with no D1 writes`失败标记，耗时26514.679158ms。20:17:20 UTC的[有界日志检查点](first-new-failure-checkpoint.json)仅观察160条通过、1条失败标记；随后相邻用例继续通过，尚无自然退出回执。当前没有错误类型/原因链/栈/断言差异，不能归因于旧socket、业务或资源；保留原socket关闭根因未证实的结论。这里只补证据和WIP状态，没有重试、改预算或修改活动测试/业务。

## 21:23计数与资源检查点

[progress-counting-and-resource-checkpoint.json](progress-counting-and-resource-checkpoint.json)保留同一原运行的新日志前缀及只读核验：377个通过用例、1条失败标记，未自然退出。数字按完成的顶层测试用例计，不是文件或命令数；同一69文件以及当前378个用例的名称/顺序与旧轮对应前缀一致（只归一化已有秘社ID改名），预期全量722仍须末尾Node汇总确认。CPU配额/节流、OOM和可用磁盘是整个环境的观察值，不能判定失败根因。前缀耗时外推约5小时只是估计，不是超时、终止或重试条件。源码、测试、资源与预算均未改动。

## 21:42 JC066正常场景与小时检查点

[jc066-normal-hour-checkpoint.json](jc066-normal-hour-checkpoint.json)和独立有界日志前缀记录同一82121运行434个通过用例、1条JC041 discard-6失败标记，最新日志21:42:16 UTC，详细异常仍未输出，尚无自然退出。[jc066-normal-scenarios-in-full-run.json](jc066-normal-scenarios-in-full-run.json)逐项记录JC066全部19条正常场景通过，包括原失败target-reveal；87种非法状态矩阵仍在执行。原socket关闭根因仍未证实，没有新增重试或改预算，未把中途结果合计为完整资格通过。

## 22:05新增失败与前置缺口

[pass-history-failures-checkpoint.json](pass-history-failures-checkpoint.json)保留同一82121运行547个通过用例、5条失败标记的新前缀；新增4项均来自pass-history文件，尚无自然退出或具体异常。[pass-history-build-prerequisite-readonly.json](pass-history-build-prerequisite-readonly.json)确认该文件专用的本地Worker目录在新环境缺失，以及固定分支中已有的构建脚本和精确ABI77来源。缺口与失败异常的关联待最终报告核对；当前只读留证，不补建活动依赖、不重试、不修改源码或预算。

## 23:02自然终态与独立补验

原完整Worker自然退出1：69文件、722项，717通过、5失败，0取消/跳过；见[最终回执](runs/worker-full-recovered.exit.json)、[完整原日志](runs/worker-full-recovered.log.gz)和[失败原文](worker-full-recovered.failure-details.txt)。[完整性核验](full-worker-natural-exit-integrity.json)确认141编译输入、74个测试源码、27个载体前后相同，187项候选产物、13个测试WASM、两个Worker及旧失败证据不变；本轮原日志SHA256为`b6728a4c16b5fc195078241618c5f1ff5925eb871917e4a675cad7b9da5cdd6f`。722用例名称/顺序与旧轮仅有已知秘社夹具改名。

4项pass-history均为缺少专用Worker目录的ENOENT，属于恢复准备遗漏，产品断言未执行。23:05已有脚本dry-run补建完成，执行文件与原专用ABI77包装字节一致；见[构建回执](runs/pass-history-worker-recovered.exit.json)与[包装身份](pass-history-wrapper-before-tests.json)。完整4项文件正在复验。JC041的fetch/SocketError栈映射到[Miniflare同步绑定代理](jc041-transport-stack-mapping.json)，关闭根因仍未证明，待完整45项文件复验。保留原失败轮，后续结果单独记录，不把补验与原失败合为一轮全绿。

23:09更新：pass-history完整文件已自然退出0，4/4通过，0取消/跳过；[回执](runs/pass-history-complete-recovered.exit.json)、[完整原日志](runs/pass-history-complete-recovered.log.gz)和[前后核验](pass-history-complete-verification.json)记录37原Native77轨迹、322接受命令、224公开让过记录，以及双端整回合和CAS检查通过。漏建包装的环境遗漏已补齐并复验。前项结束后，[完整JC04145项启动核验](jc041-complete-followup-gate.json)与[原始启动回执](runs/jc041-complete-recovered.started.json)记录同依赖、同十三核、同预算的串行补验，结果仍待自然终态。
