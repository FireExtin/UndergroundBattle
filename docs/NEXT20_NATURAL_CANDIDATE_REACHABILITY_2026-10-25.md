# NEXT20 三卡候选合法构筑自然轨迹 · 2026-10-25

固定测试源码 `64e76dfcdfae8be35fd2b68f53e9c855d0efb98d`。本片只新增 `cfg(test)`、线程作用域内的 BQ034/CQ25/XG25 构筑许可及自然轨迹测试；正式目录、其他关闭牌和生产准入均不变。仍是277条目录、233张受支持，BQ009/BQ033及这三张均 `supported:false`。测试作用域只改变验证用目录副本的三个 supported 位，复用原构筑校验，退出及 panic 时恢复。生产 release 没有该许可入口。

| 自然路径 | 种子 | Native Room 命令 | 最后回合 | 有限 WASM/D1 命令 |
| --- | ---: | ---: | ---: | ---: |
| BQ034 进场饮血及牺牲附属反制 | 1603 | 386 | 8 | 383 |
| BQ034 展示后选择本方吸血鬼饮血 | 1603 | 332 | 7 | 329 |
| CQ25/XG25 月标、死亡封印、封印后移动、非空检索 | 1723 | 357 | 7 | 354 |

Native 本片 **4项通过/0失败**（3条路径加候选作用域/复制限制），2561项旧 library 测试过滤，未跑旧完整套件。固定源码执行的命令、源文件哈希、原始日志见 [Native receipt](evidence/next20-natural-reachability-20261025/logs/fixed-native-06-receipt.json)。1075条命令每步实际调用 Room.transition、from_persisted 并记录双席视图和实际 reducer 重复探针；暂停/恢复也是普通 SessionAction 命令。

有限 WASM 与实际本地 workerd/D1 三案均通过：**1066条命令、2132个双席视图、1066个实际 reducer 重复探针**；Worker逐条比较完整状态、journal、视图和原始 HTTP 重复收据，每案4次D1重开。Worker三项耗时86.751秒，0失败/0跳过。新WASM release、UI TypeScript/Vite和Worker dry-run构建通过。WASM SHA256：`ec71c2a3958a86cb5fbb5b47e45d2bd570c6c6549804a2cfd443dd426989a2dc`；完整源码指纹：`ae97682563cb9b45806da72117c36d4c391dd7092c9c9fa88c95b224065943b5`。

## 构筑和真实来源

每席卡组均50张，同名最多三张（包括无知路人也只用三张），BQ034仅一张；填充牌还排除了原数据标 unique 的牌。用原 Game.new_with_deck/join_with_deck 创建候选大厅，保留两副完整卡组、种子、创建后和加入后状态；其后 ready/start、换牌、逐回合建立资产、抽牌和全部行动都走真实 Room 命令。没有运行中 make_card、填资产/墓地、改牌库或赋月标。三案分别实际建立了16、14、14张资产。源码与自动审计见 [自然来源审计](evidence/next20-natural-reachability-20261025/natural-reachability-audit.json)。

BQ034 两案的 XQ14 均先从合法手牌正常结附于 JC029，再由敌方 JC005 消灭，实际进入拥有者墓地。进场案随后由 BQ034 饮血并在被 JC102 指定后牺牲该鲜血、终止原事务，角色保留且伤害0；展示案用实际 BQ090 手牌行动启动隐秘对抗，展示 BQ034，选择真实 JC029并饮用上述墓地鲜血。

黑卡案由 CQ07 实际派遣和进场触发把隐秘从0变1（命令265）。XG25 被 BQ090 消灭后，合法选择敌方角色作为载体；封印牌拥有者0、载体操控者1。拥有者0实际支付1发动封印后移动（命令345，命令347结算，从相邻地区1移动至0，限次记录uses=1）。随后消灭移入的敌方角色，CQ25触发，从本方**牌库**非空检索XG25并封到伥虎上（命令354）。这些均不依赖 BQ009/BQ033。

## 候选准入与跨端边界

这是隔离候选构筑可达性，不是公开准入。每案前3条 Native ready/start 命令使用候选构筑许可；生产 WASM 的对应 start 实际拒绝，保持完整回滚。有限 WASM/D1 明确从 Native 真实开局后的状态接入，重放其余命令。**9条 Native setup命令不宣称具有生产HTTP准入/收据覆盖**；D1初始状态来自已记录的真实候选开局，后续没有人工填入资产、墓地或标志。公开 create/join/start 三卡可玩性没有被开启或宣称。

本片不覆盖三卡所有边界。CQ25来源离场检索、死亡封印失败入墓等先前定向证据仍保留原来源，本片自然路径不冒充这些分支的自然证明。完整 Native/UI/Worker/WASM483旧套件、新浏览器试玩、远端Cloudflare D1验收均 **NOT_RUN** 于本源码；原9afd8a57完整验证保持原SHA，不重签。

## 保留的失败与待复核项

全部初始失败、部分轨迹及其精确编译输入均保留：最初误以为目录已限制BQ034两张，实际接口允许两张；测试牌组填充池不足；预测选择漏 top/bottom；普通战斗漏 allocations。这些脚本修正未改规则内核。早期编译输入恢复副本均以运行前捕获SHA256核验，未改Native输出或日志。

另有来源一致性问题：原始BQ034记录 `unique:true`、keywords为空，但当前目录 `deckCopyLimit=3`。所有本片牌组只用一张，普通四张超限反例仍通过；没有擅自改变唯一规则或生产目录。该差异须父端复核，详见 [记录](evidence/next20-natural-reachability-20261025/known-source-consistency.json)。BQ009/BQ033封印供给仍明确缺失、未补供给机制。

首个Worker构建尝试通过依赖符号链接写原只读Vite缓存，EROFS后停止。没有提权或重试被拒目录；随后只在已授权Task7目录复制依赖、排除临时缓存，使用独立副本构建成功。失败原日志和路径保留。实际数据包不含依赖、缓存、数据库或构建产物。

## 本地交接

完整原始/部分Native轨迹、三份严格指纹候选包、全部失败和脚本保存在独立证据归档：`/home/ddd/Documents/Codex/2026-10-24/task-7/checkpoints/next20-natural-evidence-20261025.tar.gz`；SHA256 `342028519218e3d247f544ac36ae806fff4cd54330e0b9c2cab7cc9fbe7c9b04`。逐文件哈希见 [index](evidence/next20-natural-reachability-20261025/natural-evidence-index.json)。源码/测试在现有开发分支正常提交，保留全部原history；最终本地bundle及归档提交SHA记录于本轮交接JSON。

原桌97EC1A99E58C、双席、原32输入、网站和部署未动。没有push、GitHub CI、main合并、发布或新增并行开发写者。main与两个review远端ref只读核对未变；progress分支此次连接器断开，未重试或换路；workflow白名单拒绝未重试。父端继续审查，三卡保持关闭。
