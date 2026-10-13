# JC052 终焉的诗寇蒂：单卡候选独审交付

基线 `9890163a093621f814f9768712e2663de2fd5239`，分支 `codex/jc052-red-original-region-review-20261013`。只新增这张完整红色角色；旧138张定义及牌组、世界、其他卡池元数据不改。卡池139张，版本 `rust-v0.2.73-jc052-original-region-candidate` / `limited-v2.66-jc052-original-region-candidate`。没有发布、线上房间修改或旧存档迁移。

## 可追溯卡源

原图：[JC052 终焉的诗寇蒂.jpg](../resource/ymsj-fun.github.io/cards/JC052%20终焉的诗寇蒂.jpg)，SHA256 `3b42cde3d48132e0ecbf73dd7ee78c6379c2319a0b83cb7304f24033ab361d33`。原图实读编号 **052/135**、酒杯系列符号；勘误中的旧基础编号 **047** 只用于按相同名称定位，不把二者视作同一印次。固定规格为 `docs/factions/card-specifications.json.cards.JC052`；其 fields/sourceVerification/edition 规范化哈希 `45cb9fdb7a38fd4f2f61169b230ecfcc813f0e6b01db7468c942c5e44a223520`，没有阻塞字段或未决问题。

名称金色、副名称“剪断命运者”；红色、角色·人类·圣者；费用7、四红忠诚、一个星辰领域，永久调查1／战斗0／势力2，临时调查1／战斗0／势力0，防御1，独有。星辰继续使用现有 `MagicIcon::Other("星辰")`，没有增加领域枚举或改旧卡。

完整原图规则：

> 领袖
> 【进场触发】消灭所有本地区的其他角色和暗藏者。

公开扫描副本 `web/public/cards/JC052.jpg` 与原图逐字节相同；卡面专项测试同时核文字、忠诚、领域、副名称和原图入口。来源测试固定旧138张规范化哈希 `56c4165cd44e7d80bb222da6619c0ad9faf0c059162997bdefda140db133e4df` 与其他元数据哈希 `a0ca39bf9ba052ac13c2e64e7698bb1d5f051d38f5a75a380d54424811a357f0`。

## 规则依据及本轮补足的领袖基础

1. [隐秘世界规则手册](../resource/ymsj-fun.github.io/public/docs/隐秘世界规则手册.pdf)，**PDF第8页／印刷第6页“领袖”**：

   > 领袖是一个持续能力。每位玩家同一时间只能控制一个领袖角色在场。当另一个领袖角色在该玩家的操控下进场时，若他已经操控一个领袖角色，则他必须选择并牺牲其中一个。

   同页“来源”说明加入堆叠的能力不随来源被消灭而失效。文件SHA256 `2a771874efbdb3ad8a339ea01ff62888766090d075fe2e013632c79de1aeba35`。

2. [隐秘世界勘误及释疑](../resource/ymsj-fun.github.io/public/docs/隐秘世界勘误及释疑.pdf)，**PDF第3页／印刷第3页“终焉的诗寇蒂（基础047）”**：

   > 当终焉的诗寇蒂的进场能力触发时，即使诗寇蒂已被消灭或移动至其他地区，仍然会消灭其进场时那个地区上的所有角色和暗藏者。

   文件SHA256 `916aa6a9f50ac7e737b517358e0c4270f715d0ed7391d55c8fcb629392a71d38`。

3. 规则手册 **PDF第12页／印刷第10页“消灭和牺牲”**：牺牲只能选择自己操控的牌，移入拥有者墓地；牺牲与消灭不同。该页“友方”明确包括队友。本轮领袖遵循“每位玩家／控制／自己操控”，**没有把队友的领袖纳入本席牺牲组**。正常2v2的0、1席是队友，各自可保留一个领袖；拥有者不决定限制，当前操控者决定限制。

此前暂缓的基础现在以印刷关键词分组补足：`printed_leader`、`leader_instances` 按当前操控席筛选正面在场角色，忽略名称和拥有者。隐藏的JC052是暗藏者，不计正面领袖。借用既有 `Effect::Unique` 的强制单选、`ChoiceResolution::Unique` 的牺牲执行和同时死亡基础，没有新持久化选择类型、没有改任何旧卡定义为领袖。同名独有检查保留，领袖检查先执行。每次牺牲后重新检查，三名领袖也必须依次减到一名；控制变化及其失效后的死亡清理再次排入持续检查。

异名分组测试给 `leader_instances` 传入两种名称的显式印刷元数据布尔值；**这是纯分组契约探针，LC23实际定义未改变，也不声称它被准入为领袖**。真实生产状态的多领袖测试全部使用JC052、实际操控权状态及既有强制选择。

## 独审核对点

| 场景 | 实现与专项证据 |
| --- | --- |
| 本方和队友 | `jc052_leader_groups_use_controller_and_ignore_names_owners_teammates_and_hidden`、`jc052_leaders_on_different_controllers_coexist_and_hidden_leader_is_not_a_conflict`；不同控制席互不冲突。他人拥有、由本席控制的领袖计入本席。 |
| 入场后的强制牺牲 | `jc052_duplicate_entry_mandatory_leader_choice_preserves_sacrificed_sources_trigger`；进场事件先冻结，领袖强制牺牲先于排队的可选进场声明；新来源被牺牲也保留声明。三领袖及控制改变专项逐次牺牲到一名。 |
| 原物理地区 | `SourceSnapshot` 同时记录原槽位和地区实例UID；清扫要求原UID仍占原槽且非空位。原地区已替换／空缺时无效果，不把旧事件投射到补位地区。`jc052_original_region_replacement_or_vacancy_is_noop_without_redirect` 是显式地区替换契约布局。 |
| 来源移动／离场 | 原波次从冻结地区选择结算时仍在当地的其他正面角色和暗藏者；只排除原来源UID。移动／回手原语测试与**实际JC075付费快速响应回手**分别覆盖；同名返回的新UID也是“其他”实体。不随来源当前地区迁移。 |
| 整区死亡 | 四席、各拥有者、敌我和暗藏者一并清理；非指定程序不受屏障／护盾拦截，附属进入其拥有者墓地。复用 `remove_death_batch` 冻结死亡旁观者；暗藏者死亡不产生角色死亡声明。 |
| 可选触发恢复 | min0/max1/allowDecline=true、完整accept选项／标题／说明／座位按固定定义核对。正常接受和放弃恢复后各执行一次，重复Room命令不重复执行；12种元数据篡改拒绝。 |
| 强制牺牲恢复 | min1/max1/allowDecline=false、当前真实领袖实例视图和座位逐项核对；正常恢复继续牺牲，不能空选放弃；9种待选篡改及删除全部强制检查的状态拒绝。 |
| 程序与响应外壳 | 付费派遣／现身保留资产支付、冻结来源及地区；固定触发只能通过响应堆栈。7种付费帧（含将真实JC052卡配伪造的非JC052来源帧）、7种触发帧，以及直接派遣Effect帧／重复触发Effect帧篡改由Game和Room加载器拒绝。既有获授声望／JC089威名有单独合法程序恢复专项。 |

未发现上述原图、规则手册与勘误之间需要用户再裁定的具体语义冲突。地区替换和异名领袖属于明确标注的原语／分组契约探针，不称作自然浏览器实玩。

## 验证与交接

本机已通过项目存储守卫编译并执行19项JC052 Native专项；卡面UI1项通过。另有当前完整Room命令的四个Native快照：派遣、接受、放弃、领袖牺牲。`tools/fixture-tools/jc052-current.py` 只连接完全相同的原始状态和实际接受的完整动作，拒绝旧ABI或无对应真实命令的输入。`rust-game-wasm/tests/jc052_compare.mjs` 比较完整命令结果、四席投影及非法恢复；`JC052Integration.test.jsx` 固定ABI73及其实际WASM哈希，动作ID用属性值精确相等比较，没有旧版本回退。

最终核心恢复守卫包含伪造付费帧和直接派遣Effect帧的补强。ABI73经项目存储守卫重建，WASM SHA256 **`6d2e6bca5e688b25fc47681a54954c9ba201401ab55a10b91b0167b9146c4fad`**。最终WASM对照通过228个完整Room命令、474个恢复快照、2808个四席投影和38个非法恢复状态；匹配的 `jc052Native73.fixture.json` 已从真实Native结果生成并登记。JC052 UI为10/10（卡面1项、实际命令4项、四席隐私与禁止自动选择4项、版本/哈希1项），来源与扫描30/30，TS/Vite构建通过。

最终源码的Native库测试 **1016/1016**，其中JC052专项 **19/19**；共享 `bq030_attachment_regression` **27/27**。编译均经 `node tools/build-storage.mjs -- cargo ...`，使用既有 `/tmp/undergroundbattle-engine65-build/target`，没有绕过1GiB下限。专项实际Cargo命令为 `cargo test -p hegemony-server --lib jc052 --locked --offline -- --test-threads=1`；库回归执行该当前编译产物的全部测试，共1016项。共享回归命令为 `cargo test -p hegemony-server --test bq030_attachment_regression --locked --offline -- --test-threads=1`。本卡UI实际命令为 `npm test -- src/game/JC052Face.test.jsx src/game/JC052Integration.test.jsx --maxWorkers=1`，构建为存储守卫下的 `npm --prefix web run build`。

重新导出时使用新的证据目录：设置 `GREEN_EVIDENCE_DIR=<new>/trace`、`JC052_EVIDENCE_DIR=<new>/native`、`JC052_INVALID_DIR=<new>/invalid` 后通过守卫运行上述专项，再运行当前ABI构建、fixture连接及WASM对照。不要把既有输出目录当作可清理缓存。

最终原始证据位于工作区外层 `development-evidence/jc052-20261013/final`；外层另存PDF原文页、来源SHA及基线保持报告。补强前的日志、ABI对照和UI输入仍保留在外层，不代替final目录中的最终结果。没有删除既有任何证据或缓存。

独立云QA、全量当前ABI前端输入刷新、真实浏览器试玩尚未进行，由父会话安排。21个历史Native72/UIV072输入文件逐字节保持；其他卡的旧UI集成套件仍固定在ABI72，**不能拿旧输入跑ABI73，当前没有全量前端通过声明**。在隔离QA源码副本中导出新的Native73输入并改对应套件引用后才能验证全量前端；不要直接运行写死72文件名的旧刷新入口覆盖历史文件。本卡的fixture生成命令为 `python3 tools/fixture-tools/jc052-current.py <本次原始证据目录> [新的输出文件]`，WASM对照命令为 `node rust-game-wasm/tests/jc052_compare.mjs <本次原始证据目录>`。
