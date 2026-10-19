# JC035新环境恢复WIP证据

验证代码固定为`5056e4e65f0e1edacf617ce12175d67f86eee0fc`。本目录仅记录恢复、构建、已自然完成的两项定向复验，以及现有完整Worker的中途前缀；不是完整资格通过或发布回执。

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
