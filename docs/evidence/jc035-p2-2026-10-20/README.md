# JC035 P2恢复完整性：修前实证与修复验证

当前仍为WIP；精确反例四链路修后拒绝已验证，完整74项HTTP、新浏览器证据及父独审仍待完成。旧Worker全量失败及完整文件补验结果保留，不为本P2重跑未受影响范围。

修前源码为`ecf2236e53817c011f503042bc71eb3c48cc4861`（产品代码仍为5056e4e），141输入见旧绑定`0e98a5cb…267fe98`。旧ABI89 WASM为`02e36dc9…fcac640`，旧187项生产清单为`0ba0f278…ceaaa77`。这些身份只能用于修前对照，修后必须重新绑定。

## 修前已执行实证

- 独审给定`actor-0.steps[4].state`精确33229B，SHA256 `98aa2e997a51b959a646e2a9e09e5cde46a1ab47c833bb32898932e4c66a8425`。只把`game.pending`改为null，观察未决定、所有玩家存活、stack/effects及pacing.window为空。原始状态及改变后状态均保留，JSON变换没有丢失u64精度。
- 独立Native探针实际调用Game与Room两loader：原始合法状态接受；丢pending状态也错误接受。另从真实已接受状态仅清空stack及对应pacing.window，保留Accepted记录，两loader也错误接受。见[原始结果](before/native-two-loaders.json)。Native探针和链接库实物保留在执行环境，Git仅存代码、哈希和结果。
- 原WASM及原十三核Worker实测：两个坏状态均可四席读取，D1重开前后共各8次HTTP200；pauseRoom也返回200，并各写入1条commands和1条journal。见[未决定项](before/missing-pending.http-wasm-reproduction.json)、[Accepted项](before/accepted-missing-frame.http-wasm-reproduction.json)和[runs](runs/)。这里两个探针“通过”表示确实复现漏洞，不表示产品功能通过。

八份原始/改变后Game、Room输入以原字节gzip保存于`before/`；完整修前日志、自然退出及前后141/运行载体输入回执分别保留。

## 最小修复与合法边界

保留既有容器→观察绑定及拒重复检查，再反向要求进行中的存活观察者，其None/Accepted记录必须出现在已核验容器中。出局及终局会合法取消容器，Declined/Resolved也可无活动容器；不泵状态、不重建触发、不要求来源仍在场或当前操控者不变。

当前新增3类坏档（丢pending、局部丢待选声明、Accepted丢帧）及11种合法恢复对照，包括声明队列、待选、响应帧、真正Declined/Resolved、出局/终局None及Accepted、来源离场、冻结操控者，以及实际待选命令续行。Native、当前89重建、见证、WASM、HTTP零写入和新生产身份结果会在验证自然结束后另列，不能用修前身份替代。

首轮`native-jc035-fixed-01`自然退出101，13通过、2失败；两失败均由同一新增离场夹具触发原有完整待选选项校验。该夹具在仍有待选时直接清理宿主，不用于证明正常响应中的来源离场。已将对照改为接受后的帧仍在而来源离场，产品反向守卫不变；首轮日志及源绑定v1保留，v2重新验证。额外手工JSON旧探针与宿主清理布局不同，不能混作同一证据。

## 规则依据与像素独审

JC035准备时取样、不因手牌后续增减追溯资格，是本批明确的项目规则。XQ37结算仍复检地区势力，不作为完全相同契约或官方专属FAQ的依据。

父报告旧四PNG的官方Library上传返回401，独审者尚未看像素；没有绕过认证，也不冒称已完成像素独审。修后取新截图时将记录新候选及图片身份，并通过官方Library流程传递，实际结果单独保存。

## Native02自然结果与待补链路

原会话57400于2026-10-20 00:35:55 UTC自然退出0，15/15通过，0失败、1261项过滤；本次只跑JC035明确范围。见[runs/native-jc035-fixed-02.exit.json](runs/native-jc035-fixed-02.exit.json)。13场景、145命令、4拒绝、620投影、50坏档、11合法恢复及1条真实待选续行均实际执行，141项源码及原ABI77–88未变。原始Native输出19,576,994B，SHA256 `85a5b7bf3bdb89f162d8a37efb2bade6e00387287f459456b5124be2cd68e1a9`，保留环境供新WASM比较。修后精确反例独立双loader、WASM/HTTP零写入、新89/Worker/完整生产清单与浏览器证据尚未完成，不能提前放行。

## 00:56新核四链路与产物检查点（WIP）

精确修前八份输入哈希均未变。新Game/Room独立探针拒绝两坏档，两个合法原件接受；新WASM的view/apply/poll均拒绝相同坏档。新十三核Worker对每例D1重开前后各8次读取和2次命令均返回400，共20次400，房间、commands及journal不变且后两者始终各0条。见`after/*http-wasm-rejection.json`。

新ABI89 3,471,247B/SHA256 `df890ed0b596ca245a39bc8466270edc2e8f40f92ef8dd34359534342ccf24de`；标准Worker `695c17c25667d165df429351ab65414dfd7d749e4b5c9a7837a7cfc851e4fb45`；本地时钟Worker `eeb4af7441cb2c4546c934e01b748694c5e37f19a9657d764de2437a4e458221`，仅测试不得发布。完整生产187文件/142,257,627B清单SHA256 `28b461d38f3d0140ef7413c62509bbd58be46f88dea5f4141bcab054f6bc4da8`，见`after/`。184旧路径逐字节相同，全部客户端文件相同；旧5目录556文件完整改名保留，旧时钟Worker硬链接未覆写。

三份新Native/WASM实际比较383条普通轨迹命令、81坏档、11合法恢复及1真实续行，才写入新当前89夹具；普通13+9+owner全部state/命令/日志/投影仍与修前相同。前端指定文件16/16、路由14/14（含238条原88命令）、保留核HTTP1/1、MSJC03入桌1/1和精确反例2/2均自然退出0。原Native01失败保留。

当前下一阶段为完整74项JC035 HTTP/D1和新四客户端浏览器/PNG正式传递；这些尚未宣告通过。父独审及旧环境实物12旧核最终哈希仍是未来发布门槛，没有发布或合main。
