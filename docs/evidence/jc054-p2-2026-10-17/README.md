# JC054 两类P2：修前反例与修后验证（待串行复审）

原送审点 `f842af3a76d5e482b91a20f0cba7ec62ec3940da`，最小守卫源码点 `db15514a1400cfec1319a20c821bcd779766aff5`。完整说明见 [P2跟进](../../JC054_P2_REVIEW_FOLLOWUP_2026-10-17.md)。原81 raw为7f701ec9…bb7ca，修后为669403d5…4e0583；二者完整SHA分别列在报告和交接中，不能互换身份或结果。

- `original81-inputs.fixture.json.gz`：8份原始Room字符串（3正例、5个单字段坏档），无损压缩，不重新序列化字符串内的opaque状态。原文SHA和字节数见 `original81-inputs-index.json`。
- `original81-reproduction.json`：仅堆栈数值shield 0→1的派遣／现身均被原81接受，各4条参考命令后带shield1进场。
- `original81-reverse-source-reproduction.json`：容器JC054、来源定义改JC125，原81恢复仍接受。
- `original81-successor-reproduction.json`：同UID后继只改明置，冻结见证不改；原81接受，4次让过后坏输入的后继死亡、正例存活。
- `before-http-reproduction.json`：原固定时钟Worker真实本地HTTP/workerd/WASM/D1的5坏档和3正例均200；读取未改变opaque状态或写commands／journal。
- `after-http-reproduction.json`：同5坏档在修后全部400，同3正例仍200，无状态、commands或journal写入；`single-field-input-closure.json` 核对每份坏档恰好改变一个叶字段。
- `source-and-normal-flow-closure.json`：修前／修后各127项输入与各自精确Git匹配，原有8场完整正常观察逐字节一致。
- `*.log.gz` 与 `frontend-targeted-fixed.json.gz`：无损压缩原始日志／结果；修前Native0/4中的两个时间标志布局错误及两个真实接受漏洞如实保留，修后26/26、独立WASM对照和前端5/5分开计数。

原完整Native输入和编译物保存在 `/workspace/jc054-evidence-20261017/p2-stack-card/`，原核与原黄金输入在其 `original81-kernel/`，未被新构建覆盖。Git不含WASM、Worker、客户端构建、D1、缓存或凭据；这里的坏档是合成测试字符串，不是真实用户数据库。

最终完整Native **1179/1179**、完整Worker **269/269** 已自然结束；JC054定向26/26、HTTP/D1 26/26和四核路由4/4与整轮重叠，不相加。新81独立Native/WASM为8场景／214命令／888投影／44坏档／528恢复调用。前端相关5/5及production构建过，162客户端仍逐字节等于原送审；没有重跑完整前端、原浏览器流程或society-fixtures配置，也不借原81结果冒称新跑。

[验证汇总](validation.json)、[源码闭合](source-closure.json)、[168文件运行清单](runtime-fixed-manifest.json) 和 [无损日志索引](log-index.json) 固定最终范围。生产真实时钟与本地固定时钟测试Worker身份分别列明。Git只保存源码、合成输入及必要证据，本批未发布、未合main，仍需父会话串行独审。
