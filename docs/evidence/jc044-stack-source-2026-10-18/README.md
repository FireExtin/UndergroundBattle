# JC044 堆栈来源反向身份 P2 证据

> 后续状态：父会话已确认JC044/ABI85完整独审通过、P2关闭，官方v67发布成功。见[发布回执](../../SITES_V67_JC044_PUBLICATION_2026-10-18.md)。以下保留发布前验收及其原边界；线上普通浏览器仍证书阻断，0局。

本地适用验证完成，待父会话串行复审；候选整体未放行、未发布。源码固定为f34424b8a699f74f742748a5cdbd4c73529bbe94。1979旧源码未获放行，本目录只证明本次P2，其他独审仍在进行。

| 范围 | 实际结果 |
| --- | --- |
| 原始修前复现 | 原fixture41be2c… → raw.json → entry-deploy-skip.steps[0].expected.state；revision9/i245/itemJC044/i244/sourceJC044/i243。Native只改source.definition为LC19，Game/Room和原6ab WASM接受；4次真实PassResponse后新UIDi246的JC044入场。现身同样复现。两个完整不透明输入在original-review-inputs.json.gz。 |
| 修前单测与探针边界 | 首个--exact筛选为0项，不计验收；改为完整模块名后原新回归1项按预期失败。探针私有board方法导致编译失败，以及未考虑fresh新UID导致的两轮断言失败均保留。最终Native修前两入口复现自然0。 |
| 最小修复 | jc044.rs仅补两行反向容器拒绝，参照JC057；没有改支付或结算。新单测真实派遣/现身×接受/跳过均续行。 |
| Native | 适用18/18自然0，含现有机动兼容。原13场景和原35坏档完整输出不变；新两份坏档与原复现字节相同。133输入仅JC044模块及测试变更。原两份Game/Room修后拒绝，合法原始续行完全相同。 |
| WASM | 新c02核：原始两档四席view/apply/poll共12入口拒绝；完整13场景118命令17预期拒绝524视图及37坏档对照通过。未用JS解析/重写不透明Room。 |
| 相关前端 | 原有相关45/45，实际3文件；完整前端未重跑。没有产品UI改动。 |
| 生产/身份 | TypeScript/Vite/Wrangler dry-run通过，960输入与源码Git字节核对，179生产输出=176运行+3辅助，166客户端/9核，12本地时钟包装。166客户端与77–84八核逐字节等于1979。生产真实Date.now，所有输入/产物在HTTP验收前后不变。 |
| 实际HTTP/D1 | 六文件69/69自然0，0跳过/取消。13正常场景101接受/17拒绝、980HTTP投影，接受命令逐一D1重开与原收据一致。37坏档370次HTTP400（本次新两档20），重开前后房间/commands/journal零额外写入。仅真实本地workerd/SQLite D1，未做远端D1验收。 |
| 连接恢复 | 一次只读调用短暂断连后恢复，同一PID43681、递增日志、源码保存点、干净Git和原锁均核实；测试未重启，自然完成。 |
| 浏览器边界 | c02核没有另跑本地四浏览器；旧6ab两场为旧身份历史证据，不改挂。当前13合法路径已由Native/WASM、相关组件及真实HTTP/D1核验。已审v66在06:26正常浏览器一次GET证书阻断，0游戏写入/0线上局，未绕过或重复访问。 |

身份、完整输入/输出哈希见runtime-manifest.json，实际HTTP细分见http-d1-coverage.json；所有本目录载荷及文档哈希见manifest.json。源码/构建/测试原件在/workspace/jc044-stack-source-review-20261018；D1留在受保护磁盘临时目录，不入Git；日志gz均为无损归档。原ABI77–84见证不覆盖或重签。没有发布、合main、Actions、exchange部署或并行子任务。

新WASM c02da9cbdfd99fcbfe3b4e0b82cc206ef8572edf2f58017ffd7a58bee2d0f9c1；fixture4ad888e66386e390662f19cac5582d16dd36047b2111b90ec89590a84f8ccd63；rawf43a7e8a93b3ad77f29ce63d7fe861aa325248f8f4630a6798405327262df027。全量旧测试失败/修正边界继续见../jc044-2026-10-18/README.md，未将多轮计数相加或称完整全绿。
