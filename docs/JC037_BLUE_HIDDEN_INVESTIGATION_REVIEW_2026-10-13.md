# JC037 蓝色单卡候选：领袖与当前操控暗藏者的永久调查

意图为 `user_requested`。从已独审接受的 main `0939eab8f0ac971fc375183315313f4459a4e489` 在同一受控云克隆建立 `codex/jc037-blue-hidden-investigation-review-20261013`，初始工作区干净，无其他编辑进程；未新建克隆或 worktree。用户授权继续红、黑、蓝、灰单卡批次，中立和双色暂停。本候选须交父端独审，未获合 main 或发布授权。没有操作 Air、私有站点、Library、原始包上传或权限接口。

## 原图、规则和范围

只新增 **JC037 克里斯蒂娜·爱弗罗德／寂静女士**。实际查看原图 `resource/ymsj-fun.github.io/cards/JC037 克里斯蒂娜·爱弗罗德.jpg`，印刷 `037/135`，SHA256 `6d4199bd890373683c100b78cebca1cb6e291346b3c8a4fc7b25beea1f2feece`。完整固定规格在 `docs/factions/card-specifications.json#cards.JC037`，无字段阻塞或待裁定项；原139定义、规则注册、卡图映射均不含该牌，远端无同名候选分支。未重做历史灰色专项。

费用6、蓝忠诚3、鲜血领域、吸血鬼、防御3、金色独有名；永久战斗2、永久势力2、临时势力1。完整正文为“领袖；持续：你每操控一个暗藏者，克里斯蒂娜·爱弗罗德就获得一个永久调查1。”原图和完整规格保持，不改研究目录的历史 `selectionEnabled/gameAcceptance` 标记。

规则依据为基础手册印刷P1的暗藏者、本方，P2的持续及操控者，P3的独有，P6的领袖，P7的“你”；附属控制和拥有者沿用既有规则。实际读取手册与勘误，未检索到JC037专属勘误。手册SHA256 `2a771874efbdb3ad8a339ea01ff62888766090d075fe2e013632c79de1aeba35`，勘误SHA256 `916aa6a9f50ac7e737b517358e0c4270f715d0ed7391d55c8fcb629392a71d38`。

`jc037.rs` 绑定完整、不可改写或移植的静态定义，调查增量只读取场上暗藏者的当前操控者；跨地区计数，队友不合并，拥有者不决定计数。横置暗藏者仍被计数；增量属于永久调查，非先手回合仍有效。来源横置后当前属性仍存在，但既有规则令其不参与对抗；来源潜伏或离场立即失去正面持续能力。没有额外的可选声明、响应帧或缓存暗牌身份。`engine.rs` 在既有静态修正循环中增加一处永久调查计算调用，不为无关角色额外查表；现有领袖、赢区、空库、声望机制未改。

领袖复用已审的按当前操控者分组、强制选择并牺牲一个的处理，独有名复用既有同名在场限制。构筑沿用现行规则：金色独有不等于“唯一”关键词，本卡通常同名上限为3；Native验证合法三张新局和第四张拒绝，没有创造新构筑限制。

卡池139→140（129→130玩家牌，10地区），蓝15→16。`tools/cards/jc037-source.test.mjs` 固定旧139定义的完整规范化哈希 `8a45c42f19aed17a700793ab46ee89f5eff99b2073bdc85dfc03f1c4318246b0` 及既有牌组、世界和其他元数据。卡图增量仅一张原图逐字节副本、来源清单及客户端URL；当前扫描140张、清单151项。

## 单批次身份和真实输入

统一身份为 `hegemony-pdf-v1` / `limited-v2.67-jc037-hidden-investigation-candidate` / `rust-v0.2.74-jc037-hidden-investigation-candidate`，普通运行与隔离 fixture 身份仍区分。Sites 冻结元组同步74/67，严格 lazy 校验、版本拒绝及数据库定义保持。旧72/73输入不覆盖、不迁移。

当前WASM通过既有磁盘/内存守卫重建，2,837,852字节，SHA256 **`340b6cdc06605d9b3a4760984073b65b46f475db0d76af9dc38657704c10e7ac`**。这是本云环境源码重建产物，不能冒充 Air 原始包。旧云源码重建73包（`acf154df…11ad`）另行保留，并用其真实 `newGame` 生成明确标注来源的 `source-built-native73-lobby.json`；HTTP命令版本字段使用Native返回的数值0，未改写不透明状态。当前HTTP全入口及D1重开后拒绝旧73，数据不变。

Native导出8条JC037连续轨迹：付费派遣及暂停恢复、付费现身、秘密派遣、真实JC129控制、真实XQ16潜伏、真实JC075回手、不同领袖进场与强制牺牲空选拒绝，以及两人模式真实付费派遣。每步执行真实Room命令、精确重放真实日志，并恢复Game和Room两个严格加载器。`jc037-current.py` 连接原始导出并核对实际当前ABI catalog及版本，生成相同内容的前端/Worker输入；保留原始文件SHA，不修动作或期望状态。

继承的有限Native生产器也在74规则下实际执行，形成24份当前前端输入和20份当前Sites输入。旧57种子只用于明确标注的离线布局准备，随后由Native产生当前转移，不是存档迁移。新增当前文件名，所有历史JSON保持；XQ24非法容器仍由真实Native的uint64最大种子生产，检查工具按实际当前catalog版本取匹配输入。

JC037本地Worker/D1测试以同一可注入0ms时钟的RoomService执行连续首动作和后续命令，核对完整不透明状态、日志、四席投影、响应堆栈/选择/暂停重开及原回执。**HTTP仅验证已保存的重复回执**，没有把这些首动作写成HTTP测试。本地JS API及真实workerd/D1 SQLite不是远端Cloudflare D1或多人浏览器验收。JC052既有Worker案例的首动作同样保留注入0ms与HTTP重复回执的区别。

所有Cargo命令经 `tools/build-storage.mjs`，复用磁盘target/tmp、单Cargo构建进程，不删除旧缓存。Native示例和测试导出的存储保护由调用方守卫提供；生成入口的13组轻量守卫测试不等于Rust示例自身具备磁盘守卫。后端仅Wrangler `--dry-run`，明确记录 `deployed:false`。

## 验证与复跑

准入红灯证实原卡不在池，计数红灯证实应为2而实际0；实现后通过。开发过程各轮日志和准备错误输入另行保留，最终结果单列，不合称一次全绿。最终专项Native16、来源33、串行生成入口守卫13通过；JC037 Native/WASM为8场景、43完整命令、196座位投影、1次强制空选拒绝，全状态和结果一致。其余九张既有卡的当前Native/WASM完整比较及XQ24合法/非法容器检查通过。

最终源码Native全量 **1033/1033**通过（0失败、忽略）。首轮六个旧139总数断言，以及次轮一个旧129玩家牌断言，已精确同步为140/130，均保留原日志；完整最终轮单独通过。前端全量 **969/969（105文件）**通过；首轮唯一失败为卡图清单150→151的精确数量同步，原图/展示断言保持，最终完整轮单独通过。完整普通Worker/D1 **120/120**通过（0失败、取消、跳过），含八条JC037轨迹及真实旧73拒绝。TypeScript双配置检查与Vite构建通过；Sites严格绑定74/67、卡池140及pkg/generated/Worker三处WASM字节相同，独立负向旧64/72/73路由不进入当前Reducer。串行生成守卫 **13/13**通过；Native运行期间的资源不足拒绝日志另存，没有绕过1GiB条件或删缓存。原始资料、数据库及全部旧版输入1143个受保护blob已与0939逐项核对不变。

复跑先选新的空证据目录；不要清理或复用旧目录。专项为：

```sh
source /workspace/tooling/env.sh
JC037_NATIVE_DIR=<fresh>/native node tools/build-storage.mjs --output-dir <fresh>/native -- cargo test --locked -p hegemony-server --lib jc037 -- --test-threads=1
node tools/build-storage.mjs --output-dir rust-game-wasm/pkg -- bash rust-game-wasm/build.sh
python3 tools/fixture-tools/jc037-current.py <fresh>/native
node rust-game-wasm/tests/jc037_compare.mjs web/src/game/jc037Native74.fixture.json
```

若当前输入已存在且内容不同，单卡连接器拒绝覆盖。全量当前输入入口保留原名称 `regenerate_native72_inputs.py`，精确支持72/73/74身份组合并拒绝任意未登记身份；已有非空输出、链接、低资源、碰撞日志及后续空间不足均拒绝。74运行会生成新的前端和Sites输入；在保留既有当前输入的审查副本中使用，不能直接覆盖已审文件。随后以新的后端输出目录执行守卫下的 `sites/scripts/build-backend-test.mjs`，并在sites目录运行 `node --test --test-concurrency=1 test/*.test.mjs`。

外置证据在 `/workspace/jc037-blue-review-0939`，含逐轮原始日志、真实Native导出、当前源码重建ABI、旧73来源与完整保持性清单。新候选仍待父端独审，真实多人试玩及私有站点更新、原始包上传尚未执行，需各自明确授权。

## 审查范围分类

本批次127路径（原126另加一项JZ55当前卡图清单精确数量断言）：生产源码/数据/身份7，卡图与映射3，当前ABI74输入44及真实旧ABI73负向输入1，测试66，Native生成/校验工具4，本地预览脚本1，本说明1。配套路径的多数变化为当前文件引用、精确版本/卡池数量断言及新Native输入，不是新增其他卡牌。测试分类包含Native专项、现有六项目录数量同步、来源保持、WASM对照、前端及Worker/D1；`engine.rs` 中的目录断言计入其单一源码路径，不重复计数。

逐路径清单和SHA在外置交付清单单列。构建pkg/generated/dist、Cargo/npm缓存、Python字节码、原始运行日志和真实凭据不提交。历史原图、研究规范、旧JSON和数据库定义保持，只有指定JC037及其完整身份闭环进入候选。
