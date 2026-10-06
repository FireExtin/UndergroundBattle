# JZ31 真实响应链补充

生产基线为父已接受核心实现的 `b1a036fe13012f0b9484e50c1e265655659c2cad`。本补充只增测试、WASM对照脚本与记录，生产规则、engine35产物、Sites源码均未改。仍只在 UndergroundBattle 云端执行，没有推送、合入main或发布。

## 一条连续的实际会话链

测试 `jz31_actual_composed_response_hides_another_instance_before_frozen_death_effect` 只有一个明确的四席初始布局。此后每步均调用真实 `RoomEnvelope.transition`，下一步使用其完整返回state，经 `from_persisted` 验证并逐字保持；不重建响应窗口、不直接突变棋盘。这是初始夹具后的真实协议链，不是自然公网开局，也不是进程重启测试。

席0实际施放JC129，夺取席2拥有的JZ31 `i239`；接着实际施放JC091摧毁它。选择发动死亡能力时，冻结actor/controller=0、owner=2、原地区索引2及真实地区实例 `i206`。席0/1实际让过后，席2在当前死亡窗口执行一次 `beginResponse`，确认composing及intent，再执行一次 `submitResponse`，以JC063的合法hide模式隐藏地区内另一张席2控制的JZ31 `i240`。

堆栈顺序为底部JZ31死亡frame `i260`、顶部JC063响应frame `i262`。响应先结算，另一实例成为新的隐藏实例，影响力仍 `[0,0]`；随后死亡效果结算，原地区为死亡时actor0的队伍增加恰好一点，变为 `[1,0]`。原死亡实例／controller／owner／地区index+真实instance的完整frame在让过、开始编辑、提交响应、响应结算期间逐项相等。墓地归owner2，另一同卡实例没有触发死亡。

一次原生定向测试通过（1 passed；其余466规则测试被筛选，未全量重跑）。同一条链在原候选35 WASM上复跑，23条命令的完整transition与92份四席view均等同原生：3 game、18 passResponse、1 beginResponse、1 submitResponse。链有24个不同持久化state，全部连续，没有被解释为24次进程重启。

WASM仍为2,177,325 bytes，SHA256 `4ceb4f410ff96796888f2568b426a56866f8db743a8cebf41f41d9e47e98ee7b`，没有为测试重新构建。原498通过仍来自 `native-workspace-complete.log`，没有宣称新499项全量通过；原514仅为272个不同state的序列化检查。

保留首次筛选错误（0测试）、首次实际失败（误认为死亡声明后先归对方priority；实际先归actor队伍）及修正后成功日志。修正只给测试补上席0/1真实让过，未修改priority规则。

复现：

```sh
JZ31_RESPONSE_EVIDENCE_DIR=/absolute/evidence cargo test --locked -p hegemony-server --lib jz31_tests::jz31_actual_composed_response_hides_another_instance_before_frozen_death_effect -- --exact --nocapture
node rust-game-wasm/tests/jz31_response_compare.mjs /absolute/evidence/response-chain.json
```

## 仓库Sites的只读缺口

直接阅读 `sites/src/kernel.mjs`、`kernel-router.mjs`、`scripts/build.mjs`、`scripts/stage-source.mjs`、`test/kernel-routing.test.mjs` 和README：最高明确冻结路由为engine10，当前槽位未固定版本；README与路由回归仍明确要求当前engine11/pool8。build/stage只复制当前pkg及legacy1至10。即使当前pkg是候选35，也不能据此声称该旧适配已支持现网34或正式35。

游戏仓库另有legacy14至33及legacy26-resource-policy冻结文件，但该旧Sites loader/build/stage没有引用它们。游戏仓库缺少legacy11/12/13、legacy25-resource-policy与精确原34 WASM。不能把“已存在历史文件”当作“实际已路由”。

父最新已定位现网Site32独立源码：HEAD `867438926278c0da577deacb30b8d27a669974f5`、tree `c8d845d4b68b0dadeca4eec9739c5e380bbb5166`，583 tracked files；精确原34 WASM为blob `7fd15fe4af28b6e42a990a6c5a782cee4ac40cec`。父报告适配相对旧sites多／改12文件，含lazy-kernel及兼容测试，并保存上述额外冻结。此段是父提供的事实，尚未在本云端消费执行器取到独立源码。

相对现网34，最少需拿到这份精确独立源码及其适配12文件、其缺失冻结模块（含原34），然后以原发布清单核实兼容路由。原34 WASM期望2,171,802 bytes／SHA256 `b17c350a65ba68291a9ff8c44f28382298932abb17a502a35cb9b6a73620482e`；本地源码重建不能替代它。无需手工猜测重建完整583文件或重取旧大包。

独立源码交接分支尚待父取得用户授权。本轮没有再请求公共URL、尝试Library下载、修改Sites、运行Site构建或保存／发布版本。后续装配等待精确Site32基线；当前只读缺口报告不构成发布审查。
