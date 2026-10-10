# JZ48 独立机制审查

审查者 `/root/review_jz48_mechanism` 未参与本批实现。审查时间 2026-10-07 10:56 UTC；仅审固定提交 `02a69dc47142c4b48800f25af38ba3dae2b8e532`，基线 `2305aa17c53e73dd3683ae9fe1e654531777f16e`。工作树 `/tmp/jz48-mechanism-game`，分支 `codex/jz48-finite-2305-cloud`，审查前后 HEAD 一致且 porcelain 为空。

结论：在固定机制 diff、规则、独立聚焦 native/WASM、历史身份路由、字节核验、固定组合树与后附自然 UI 证据补核范围内，未发现 P0、P1、P2、P3 问题。可交父审查；此结论不授权发布或合并 main。engine44完整策略终局不在此结论范围内。

## 规则证据

实际完整查看云端原图 `/workspace/jc094-game/resource/ymsj-fun.github.io/cards/JZ48 街头劫匪.jpg`，SHA256 `a9a5da75938c22872cca524776356b10f2a0250370d3ba3ed3b31deb332e5b4f`。实际查看 `/tmp/jz48-mechanism-evidence/rules/printed-p16.png`、`printed-p21.png`：本方按当前操控者本人判定；友方才包括队友；2v2 本方能力不扩大到队友。

原图为费用1、黑忠诚1、角色·人类/罪犯、永久战斗1、防御1，无领域要求、无临时图标或独有。持续条件为当前地区另一个本人操控的正面罪犯角色，使本实例获得永久势力1、防御+1；自身、隐藏牌、资产、别区、队友与敌方均不满足，横置支持者仍有类别。

## 源码审查

- `rust-game/src/attributes.rs:12` 使用单一实时封闭条件，按实际来源实例所在地区和 live controller 查询；支持者 ID 不同、正面、角色、相同当前 controller，并经 `current_subtypes` 查罪犯。多个支持者仍只返回一个布尔加成，双方同名实例分别查询。
- `rust-game/src/engine.rs:396` 与 `:497` 共用该条件，分别接入永久势力和防御；横置沿用既有参与规则，伤痕在全部防御加成之后扣减。
- `rust-game/src/rules.rs:1033` 对整个 JZ48 Definition 做精确白名单；移植 modifier 到别牌、删改、重复、附加能力或关键词均拒绝。新增 modifier 无参数；没有通用绑定框架、新身份、效果队列或新的死亡路径。
- `rust-game/src/engine.rs:1778` 原有同时死亡集合先冻结各来源，再逐个离场并循环处理失去防御后的新致死集合；本提交未改该清算段。`control.rs`、`resolution.rs`、`model.rs` 与基线无 diff。墓地按 owner，清伤和回合控制到期仍走既有原子 cleanup。
- 原101普通卡定义逐项与2305相同，只新增JZ48为102；秘社8、预组5、世界与构筑规则由实际路由测试与冻结43目录对比核查。既有JZ49能力保留；其旧UI夹具明确绑定冻结43。

## 独立执行

执行证据目录 `/tmp/jz48-independent-mechanism-evidence`。首次 shell 缺 cargo PATH；定位已安装的 `/workspace/jz31-tools/cargo/bin` 后使用本云端原有 Rust1.90、离线依赖与共享目标缓存重新执行。第一次字节核验脚本错误地从原始 cards.json 读取不在该文件中的 societies 字段；修正审查脚本后完整核验通过。这两项是审查驱动错误，未修改产品源码。

独立重新编译固定源并运行19个JZ48 native测试：19通过、0失败、0忽略。测试采用单job、`CARGO_INCREMENTAL=0`、DEV/TEST DEBUG=0、STRIP=debuginfo，并输出新 oracle/frontend/chain 证据；没有改动冻结engine43审计程序或浏览器牌桌。

覆盖包括四席controller/地区矩阵、自身与队友排除、暗置/资产/非罪犯、横置与先手、多个及互相支持的JZ48、owner≠controller、移动与控制变化、控制到期和来源离场、清伤到期原子顺序、翻暗换身份清伤、付费妖火级联、同时死亡及owner墓地归属、恰好一点势力和队伍总分10、付费派遣与原子费用/忠诚拒绝、非法完整Definition移植、付费响应链重放和恢复、SQLite每步关闭重开、重复回执和commandId冲突。

对上述新 native oracle 独立运行 `jz48_compare.mjs`：59转换（8拒绝）、282保存检查点、13步连续付费响应链、1416四席投影与native完全一致；当前44拒绝旧43状态，冻结43仍能读取其状态。

独立 `node --test test/kernel-routing.test.mjs`：3通过；39历史tuple及5异常身份拒绝，拒绝状态进入当前解释器次数为0；当前新房流程与原始u64 seed保持；生产生成模块及Worker只含1个当前44 WASM。

独立逐字节对比199个历史文件与2305 git blobs，零差异；冻结43五文件逐字节等于已发布2305工作树pkg。当前44 WASM 2,191,175字节，SHA256 `6cf6dca6a06844c84e38fbed2a084145bc0e63ae3897de18b864814c314d11f9`；冻结43 WASM 2,186,939字节，SHA256 `3b0bee7b1c6788bc0a299338322f1c07d6108161a75e4999ac066971659520bb`。实际Worker WASM与当前pkg字节一致。

证据：`native-focused.log`、`wasm-compare.json`、`kernel-routing.log`、`byte-and-scope-proof.json`。其前三项SHA256分别为 `8cfa2f9ae13a7bc49335200d7c0af6acc99047d29b6f1e089f72efeda1ca5b20`、`453dbb4455d78fbd14511a072eb76a41be6d86e1d84d2ae58fe538368254f2ad`、`63668b60fee258941fe85d15c69ac1f8060b90954c4e92dc99927ff1da71d9ea`。

## 取证与限制

另读实现者固定提交的完整日志/JSON：native545（核心513+集成32）、Web537、Worker25均通过；当前44原JZ49 native回归包含于全套；冻结43 JZ49 WASM回归3616投影通过。这些全套结果是实现者日志取证，不声称由本审查者重跑。Web65个测试文件可由JSON的testResults核实；Vitest报告104 suites包括嵌套suite，不能把104当文件数。

本审查独立聚焦WASM比对使用明确的离线native初态；该部分SQLite也是明确的本地合成fixture。审查者没有开浏览器、改变原有engine43四席牌桌、访问生产房间、改产品/commit/推送、上传Library或发布。后附独立组合及自然UI补核明确记录其新增覆盖。P0–P3缺陷列表：空。

## 固定组合补核

父会话提供组合产品提交 `c98bf422795c48d5842011dd5b65a97e647d3f14` 后，独立核查 `/tmp/jz48-combined-review-game` / `codex/jz48-combined-review-20261007`。其 Rust tree `b4f921ae4b1e1cb45266f5072667c532343cd001`、整个 rust-game-wasm tree `9cdff1379436567e1bae70d8c73f832f81a88793`、WASM src subtree `188680abc5cd379e03ee2bd8ba85c22662b49335`、Sites tree `54993f3790128a5952d1d38b2fdc7e73e6f766d3` 均与已审机制02a69dc完全一致。`9cdff...` 指整个 WASM 树，不能将其标为 src subtree。

组合在机制之上只叠加7个文件，每一个git blob均与已独立卡面审查的 `65ad3b97ec3bc7d3e6f567ad40dfda05d02cb190` 相同：卡面说明文档、scan manifest、JZ48/JZ50原图、JZ48JZ50卡面测试、JZ55卡面计数断言、scan registry。没有新增JZ50机制或改动engine44产品代码。已读卡面独立报告；该报告无P0–P3且明确本地展示fixture的范围。

组合pkg与Sites generated的当前44 WASM均为已核hash `6cf6dca6a06844c84e38fbed2a084145bc0e63ae3897de18b864814c314d11f9`，字节互相一致。组合跟踪文件diff为空；有父会话运行测试用 `sites/node_modules`、`web/node_modules` 两个未跟踪依赖链接，故此刻不将整体工作树称为clean。本审查不删除父运行中链接。机器证据 `/tmp/jz48-independent-mechanism-evidence/combined-tree-proof.json`。

此次树补核未发现组合引入的P0–P3问题。无需重复相同Rust/WASM/Sites树全套测试。父组合Web/Worker及自然JZ48 UI的最终结果已在后附节独立取证，不将初次树核查时的运行状态留作最终阻塞。

## 自然 UI 与组合验证补核

父本次补充证据均对应固定产品 `c98bf422795c48d5842011dd5b65a97e647d3f14`。已读 `/tmp/jz48-combined-evidence/combined-verification.json`、Web JSON、Worker/build日志：66个Web文件557项全部通过，0失败/待定；Worker25通过、0跳过；TypeScript/Vite/Worker构建通过。这是父组合实际执行结果的独立取证，本审查者未重复这些全套测试。

已读 `natural-ui-e2e.mjs`、`natural-ui-harness.mjs`、`natural-ui.log`、自然UI摘要，以及本地原始 progress steps、HTTP记录、最终snapshot内原始commands/journal。实际查看手牌打开原图、两实例互助桌面、摧毁后桌面这3张截图；另实际查看手机幸存者当前阅读和原图2张截图。未新开浏览器或复制牌桌。

该自然 run 通过真实UI建立合法50张自定义牌组（JC125×44、JC091×3、JZ48×3）、create/join/ready/start、保留手牌、资产与付费派遣、地区目标确认和实际付费《谋杀》。测试wrapper仅在普通 `RoomService.create` 的8字节seed输入固定4586；其余ID/token/nonce调用原生随机数；harness源码没有直接更新房间state或注入引擎布局。初始持久state为revision0/lobby/空场，seed4586；create HTTP body的牌组计数与构筑一致。正常命令日志显示逐回合推进至第3回合。

独立交叉核验：120个steps命令ID唯一、均200；原始HTTP有122个POST（create/join各1及120命令）全部200；120个步骤、120个HTTP命令、120个DB回执及120个journal Command事件的seat/完整command/完整view回执/version逐项相同。121条journal由1次Join和120次SessionEvents构成，版本1…121连续。原始凭证、entry response、完整state、DB及私有snapshot只在本地读取，未打印或加入报告。

可直接由原始views核验的实际规则转换：v16单张本人JZ48势力0/防御1；v65同地区两张本人JZ48实例ID不同且均势力1/防御2；目标动作来自实际手牌JC091《谋杀》，指向第二张互助实例，3个原未横置资产变为横置；v121幸存者仍是第一实例，立即回到势力0/防御1，owner p0墓地恰有1张JZ48。桌面截图与上述状态相符。未出现新增P0–P3问题。

两次目标取消的执行路径比较取消前后room state/commands/journal完全一致，再重新选目标确认；自然driver成功日志证明该断言执行完成。SQLite dispose/reopen前后整个snapshot一致并两席reload的driver断言也完成。这里是执行者driver和实际记录交叉取证，不声称审查者另行点击取消或重开浏览器。

独立重新执行正常WASM原journal回放：120 Command事件、121条journal，最终原始持久state字节完全相等。另独立以SQLite mode=ro核对现存同一D1的room/commands/journal与final-snapshot完全相同，随后实际执行冻结engine44原生 `hegemony-audit` 只读重放（SHA256 `4092c3da5054f4c08956826564327ef5f73d077fd8365ecc1437d077ed908673`）；version121、journal121、status playing，persisted/replayed digest均为 `0b1ee65f705f53db3e9150fac0819bd630e017d5a142ec24048edb9746a610a0`。此次独立核验前后实际DB与WAL SHA256保持不变，未写入游戏数据。

手机补充driver恢复同一房间同一席位的已有会话，在390×844视口把地区横向滚动到幸存实例，打开当前阅读和原图。result记录version121、势力0/防御1、原图400×560且contain、POST=0、前后DB snapshot相等；已实际查看两张full-page截图，当前与印刷图标/防御及完整原图可辨。此项是同一自然牌桌结束后只读阅读验证。

新增独立脱敏证据：`/tmp/jz48-independent-mechanism-evidence/natural-ui-independent-cross-check.json`、`natural-ui-wasm-replay.json`、`screenshots-actually-viewed.json`。审查脚本首次误从view读取不存在的resources字段；按既有资源实现改为检查3个未横置资产→3个横置资产后全部核验通过，未改产品。

范围界线：此次实际自然UI覆盖正常构筑/准入、付费派遣、原图阅读、同地区同controller两张JZ48互助、实际付费摧毁导致即时失去支持、取消目标、保存重开与桌面/手机阅读。owner≠controller、2v2队友排除、隐藏/横置/其他地区、移动/控制到期、清伤原子顺序、伤害/伤痕导致级联及同时死亡、阈值总分10、非法Definition、重复回执/付费响应等仍由先前明确的native/WASM/SQLite离线规则fixture验证，不声称已在此自然牌桌逐项点击。该牌桌终态仍playing、turn3，不是engine44完整策略终局；root另一个原engine43四席牌桌的终局工作与本批分开。没有发布、合main、上传Library或导出私有运行资料。
