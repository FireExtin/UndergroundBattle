# JC089 / JZ51–JZ53 原图候选独立审查

固定候选：`cf2b9657c9f4e0ba8dcb808aaa1e0c2afc80a0e4`。基线：`99a208c0e6dc73669292396268840a0d9034547a`。审查者 `/root/review_jc089_jz51_jz53_card_faces` 是本批新审查者，未参与实现。审查和实际原图查看均在 UndergroundBattle 云端执行；未操作 GitHub、main、Sites、Library 或官方包下载。

结论：固定原图候选在以下审查范围内未发现 P0、P1、P2 或 P3 问题。可进入父会话组合审查；这份结论不批准任何机制准入或发布操作。组合提交尚未通知，组合后的绑定将在收到确切提交后补充。

## 逐项审查结果

| 级别 | 结果 | 核对出处 |
| --- | --- | --- |
| P0 | 无发现 | 全部9文件差异；Rust、WASM、Sites 子树均与基线相同。没有运行数据、权限、服务或发布改动。 |
| P1 | 无发现 | `CardTile.visibleCard` 先抹除非操控者的隐藏 print / cardId，`ReadModal` 使用投影后的 cardId 查图，`ArchiveArtwork` 对隐藏卡与资产不渲染原图。上述生产文件没有修改。独立运行的隐藏视图测试包含 owner p1 ≠ controller p0 四席场景。 |
| P2 | 无发现 | 四个新增 JPEG 的 canonical 文件、Git blob、服务文件字节与 manifest SHA256 完全相同；400×560 JPEG；旧111条 manifest 和111张旧图逐字节保留；115条 URL 映射与 manifest 一一相符。 |
| P3 | 无发现 | 新40项原图测试、旧 JZ55 计数由111更新115的必要变化、相关76项回归及 TypeScript 均独立通过；四张实际完整原图、全部24张展示截图和 FAQ 印刷P4均实际查看。 |

差异只有：说明文档、`web/public/card-scans.json`、四张 `web/public/cards/{JC089,JZ51,JZ52,JZ53}.jpg`、`JC089JZ51JZ53CardFaces.test.jsx`、`JZ55UniqueDestroy.test.jsx`、`cardScans.ts`，共9文件、148行增加/1行删除。没有新的 reader 行为、服务端投影、牌池或规则改动。

## 主来源、印刷字段与字节绑定

四张原图实际从 `/workspace/jc094-game/resource/ymsj-fun.github.io/cards/` 打开整图查看。以下摘要由原图观察后与候选说明/测试/展示夹具交叉核对，字节由独立脚本核对。JPEG 原样复制，没有裁切、重压缩或新绘图。

| ID | 原图 SHA256 | 字节 | 实际观察到的主要印刷字段 |
| --- | --- | ---: | --- |
| JC089 毒血诅咒 | `83853a1a4c3e6ecd9644f7bb5c843e70585d421f1792a0e4a141ebbe867d12f7` | 312412 | 费用2，黑忠诚1，附属·诅咒，鲜血领域1；结附目标角色，给予宿主防御−1、永久战斗1、临时战斗1和威名。这些不是附属本身的图标或防御。089/135，酒杯系列符号。 |
| JZ51 温迪戈 | `1b0311913bbf4a26c0e9e2dc2c0d6b656bd802cc6e831350f6f2500edf577dc0` | 521395 | 费用5，黑忠诚1，不死生物·食尸鬼，死亡领域1；永久图标0/2/0，临时调查1，防御3。牌库顶8张寻找任意数量无名尸体并置于本地区、其余洗回；墓地行动封印本方墓地3张无名尸体到本方秘社，自己从墓地进场。白名、51/76。 |
| JZ52 大维齐尔迈哈穆德 | `7f9ad8adb68bdd84877ebdead2bc13ed04983099977333d2106486405c6478a3` | 521413 | 费用7，黑忠诚4，人类·法师，死亡领域1；永久0/2/2，临时调查3，防御5。金色独有名、副名魔特之子、领袖；阻止从手牌正面打出角色，允许从墓地正面打出死亡领域角色如同在手中。52/76。 |
| JZ53 尸舞舒拉密兹 | `a84d896c1b552dabda05a1fc12c55224275a31b63a8408446508775235cc4623` | 509401 | 费用4，黑忠诚2，人类·法师，死亡领域1；永久0/0/1，临时调查1/势力1，防御1。金色独有名、副名死灵术士；进场取回己方墓地目标死亡角色到手；角色离开自己墓地时在目标地区生成行尸。53/76。 |

FAQ `resource/ymsj-fun.github.io/public/docs/隐秘世界勘误及释疑.pdf` 的 Git blob 字节独立核对：516140字节，SHA256 `916aa6a9f50ac7e737b517358e0c4270f715d0ed7391d55c8fcb629392a71d38`。实际打开实现者渲染的印刷第4页，看到毒血诅咒把防御1角色降至死亡的例子。FAQ 历史编号078与当前089/135原图按卡名/效果绑定；本原图分支没有实现该效果。

## 实际准入与库存

独立使用已审 engine44 WASM 执行 `initSync` / `catalog()`：`rust-v0.2.44-jz48-criminal-condition-candidate`、`limited-v2.41-jz48-criminal-condition-candidate`、`hegemony-pdf-v1`，102 ordinary（含10地区，即92非地区）、8会社、5预设。JZ48/JZ49仍在实际目录；JC089、JZ50/JZ51/JZ52/JZ53全部不在 ordinary 或会社目录。JC089机制是另一个候选，本原图分支不承载其准入。

实际 WASM 为2191175字节，SHA256 `6cf6dca6a06844c84e38fbed2a084145bc0e63ae3897de18b864814c314d11f9`。Rust 子树 `b4f921ae4b1e1cb45266f5072667c532343cd001`、完整 WASM 子树 `9cdff1379436567e1bae70d8c73f832f81a88793`、Sites 子树 `54993f3790128a5952d1d38b2fdc7e73e6f766d3` 均与基线相同。

计数由实际 ABI、115条原图 manifest 和 Git 中685条 research records 独立计算：排除TK007先手标志为684牌面/组件ID；574组件未准入；569原图未注册，其中568有记录来源，TK011唯一仍缺同ID canonical 原图。684包括会社、地区、衍生物、变体等，不是684张可构筑玩家牌。未修改历史研究索引标签，也没有为TK011补造原图。

## 独立执行验证

在固定工作树 `/tmp/jz51-jz53-card-faces-game` 独立运行：

```sh
cd /tmp/jz51-jz53-card-faces-game/web
npm test -- src/game/JC089JZ51JZ53CardFaces.test.jsx src/game/JZ48JZ50CardFaces.test.jsx src/game/JZ49CardFace.test.jsx src/game/JZ49Slow.test.jsx src/game/JZ48Criminal.test.jsx src/game/JZ55UniqueDestroy.test.jsx src/game/ArchivePresentation.test.tsx src/game/ReadModal.test.tsx --reporter=default --reporter=json --outputFile=/tmp/review-jc089-jz51-jz53-card-faces-evidence/focused-tests.json
npm run typecheck
```

8文件116项通过、0失败；生产 app/node TypeScript 检查通过。输出 JSON 明确写入本地独立证据目录，没有修改产品。已有 cloud `web/node_modules` 与已审 engine44 `pkg` 被短时链接；package-lock 逐字节一致。完成后两个 ignored 链接移除，实际 HEAD仍为固定cf2b9657且 `git status --porcelain` 为空，未改源码、checkout或依赖内容。

独立运行 `inspect-catalog.mjs` 与 `check-source-preservation.py` 成功。`source-preservation-and-review-bindings.json` 绑定实际源码、测试、111张旧图、4张新图、每张PNG、browser driver/fixture/results与本次运行日志的 SHA256/字节/Git blob；记录实际查过的图片，不把实现者日志写成独立重跑。

## 展示证据与边界

独立逐张打开24张PNG：4卡×桌面1440×1000/手机390×844×手牌、文字reader、完整original。完整牌面包括费用、忠诚、正文、底部编号与画师署名；无裁切。桌面 compact 显示图片，手机沿用原布局隐藏compact图片，文字与完整original均可读。

审阅实际 `card-faces-browser.mjs`、React夹具、定义JSON、`browser-results.json` 和 browser log：实现者使用单browser/context/page顺序执行8次公开阅读及32个隐藏视图（2viewport×4viewer×4card），0 API请求、0 pageerror。数值报告显示原图decode400×560、`object-fit: contain`、image box落在两viewport内；隐藏compact都无图片/卡面标识，仅currentcontroller p0可阅读隐藏print，owner p1及p2/p3不可。独立116项DOM测试重复覆盖这些权限关系。

这是生产Table/ReadModal的**合成展示夹具**；其中卡定义是印刷信息展示输入，实例ID/roomId等明确为LOCAL fixture，无自然房间、服务器投影、席位凭证、数据库或游戏写入。独立审查者没有新开browser或把展示截图当作自然对局。未验收JC089机制、JZ50–JZ53机制、2v2战局、完整策略终局或服务器hidden projection；这些不属于本9文件原图差异的结论。

## 交付

独立脱敏证据目录：`/tmp/review-jc089-jz51-jz53-card-faces-evidence/`。本报告：`/tmp/jc089-jz51-jz53-card-faces-independent-review.md`。原图和展示证据留在本地；本报告及summary没有真实房间、凭证、运行快照或数据库内容。


## 固定组合84a36d6补充审查

上述cf2b9657报告正文完整保留，以下是父会话通知固定组合后的新增结论。实际审查产品 `84a36d624528ab61f752e4ad9f05fdeb1839d7e4`，目录 `/tmp/jc089-combined-review-game`。它将机制19e594f0、独立P2边界修复a96a5595、原图cf2b9657及JZ50用户B裁定文档组合；本审查负责原图/reader/身份和计数绑定，机制正确性由另一个独立审查者负责。

实际读取基线到组合的63文件差异目录，以及原图9文件和直接相关的 `CardTile.tsx`、`types.ts`、JC089/JZ48 UI测试、`catalog.rs`、新增JC089 registry字段与JZ50裁定文档。独立脚本逐一读取候选Git blob和组合本地文件：全部9个已审原图路径的blob ID及字节完全相等cf2b9657，包括原图说明文档、manifest、4张JPEG、两个原图测试文件和URL映射。旧111条manifest及111张旧JPEG仍与99a208c0逐字节相同，新115条注册不变；4个新增JPEG仍为canonical SHA/400×560。

`ArchivePresentation.tsx` 与 `ReadModal.tsx` 在组合中保持已审字节。组合增加的 `currentCombatGlory` 字段在 `visibleCard` 非操控者隐藏投影中显式抹除，屏幕威名标记受 `!faceDown` 约束；原图与hidden print的授权路径没有放宽。JC089正文/独有/费用/鲜血忠诚与实际已看原图一致，附属本身的永久/临时图标仍均为0，宿主属性由独立机制路径处理。JZ50文档明确是用户项目裁定、不是原始FAQ补文，本组合没有JZ50准入。

独立只读加载组合actual pkg WASM并执行catalog：2198305字节，SHA256 `afc1b42523111dc242fd21850da9af243112a619c9f759bc60cbabac045883f7`；运行身份 `rust-v0.2.45-jc089-poison-blood-candidate` / `limited-v2.42-jc089-poison-blood-candidate` / `hegemony-pdf-v1`。103 ordinary包含10地区（93非地区），8会社，合计111准入组件。独立实际catalog比较确认JZ48、JZ49定义与冻结44等价，world/societies/decks/deckBuildRules亦相同；冻结44 WASM为已审2191175字节和`6cf6dca6…16f9`完整SHA。JC089实际准入，JZ50/JZ51/JZ52/JZ53仍只注册原图。

重新用actual45 ABI、组合manifest和Git归档records计算：684牌面/组件ID、111准入、573待玩法准入、115已注册原图、569未注册原图，TK011唯一缺来源。该独立值与root `candidate-fixed-identities.json` / `combined-catalog-and-originals.json`一致；两份root证据本次SHA/字节绑定已记录，不把root断言当作独立计算。

补核未发现P0、P1、P2或P3问题。证据为 `combined-art-binding-review.json`、`combined-actual-catalog-summary.json`、`combined-preserved-catalog.json`，含全部9路径blob+SHA/字节、实际ABI、root证据绑定；可复核的独立命令是 `node inspect-combined-catalog.mjs`、`python3 check-combined-art-bindings.py`、`node check-combined-preserved-catalog.mjs`，均位于独立证据目录且通过。没有重复116组件测试或打开新browser，没有读取真实entrycredentials、roomIds、完整状态或数据库。

工作树HEAD在补核前后均是固定84a36d6。此时只有root明确管理的 `sites/node_modules` / `web/node_modules` 两个未跟踪链接；审查者没有添加、移除或改变任何root文件/链接。root全套Web测试/build及后续真实两席UI仍在执行，本补核不提前声明其通过；实际组合UI截图尚待通知，既有24张合成展示截图的边界继续适用。


## 固定组合真实UI与最终审查范围

收到root新增证据后，独立审查者实际打开全部8张**新增自然流程**截图，并逐张重算文件SHA256/字节，与 `/tmp/jc089-combined-evidence/natural-ui-integrity.json` 中对应条目完全一致。它们绑定产品 `84a36d624528ab61f752e4ad9f05fdeb1839d7e4`；与前述24张合成展示截图分别统计，不相互替代。

| 实际查看的新增PNG | 记录版本 | SHA256 |
| --- | ---: | --- |
| `jc089-before-curse-pair-desktop.png` | 217 | `7148d5fe326e2a34730ede1f98853f1b6c2371eb8436e946623933d04641d74c` |
| `jc089-natural-original.png` | 217 | `faa6ae0663057c5ab1119de6800e79421dbf176b2d564d79020b4db5cdc06d13` |
| `target-confirm-1.png` | 217 | `94f81aecd3b816ff0bee1d4dc3835f00b0accaeabdc81b29d33589cb2b0732bb` |
| `jc089-first-curse-attributes-desktop.png` | 220 | `12a7d7f3533aab4308323389c33e8dc76f355e402b5c7d4675317a302916c3c1` |
| `jc089-glory-one-marker-desktop.png` | 258 | `2155026a56eef73a33866f8e05bf2e62c59b5cd4440b68126c106725ef472bbb` |
| `lost-ack-recovered.png` | 272 | `aaf9c5af4dbba6398fe3d27b7b6a7e753ea5f8aa0d59eda6b8f4eef758f79155` |
| `jc089-second-curse-death-desktop.png` | 274 | `a19e33d45ba529b8b3686599f7f9fd72a1a188f96f5e5aa443ddd02232c8a6bf` |
| `jc089-survivor-mobile-reader.png` | 274 | `edc6bf880ac070160eae5beb3b7fc5b668beb71844fdecbc850a89bc91b5e943` |

可见检查结果：初始两张JZ48分别有加成防御2/势力1；JC089确实在实际手牌中，原图reader显示完整canonical牌面，包括089/135和画师署名。实际driver在打开原图后断言src `/cards/JC089.jpg`、解码400×560、computed `object-fit: contain`；这段driver断言已审，截图视觉显示全图。目标确认截图显示已选宿主、确认和取消选目标按钮；driver取消分支明确前后比较state/commands/journal完全相同，root回执记录1次成功取消，本审查未独立读取数据库重算它。

首附属截图显示宿主获得威名、当前防御1、附属1和战斗加成；实际战斗截图地区势力标志为1。恢复截图显示真实已提交毒血诅咒的响应窗口；第二附属结算后截图显示宿主与两张附属均不再留场，墓地数量3，幸存JZ48失去支持，当前防御1/永久势力0。手机reader明确显示幸存者当前0/1/0与印刷0/1/0、当前防御1与印刷防御1、完整持续正文。

手机driver实际viewport为390×844；该PNG使用 `fullPage:true`，文件尺寸390×1426。图中reader正文和底部属性完整未裁切；这张完整页面截图不独立证明所有元素的viewport DOM几何边界，本审查不将其当作另一次手机browser运行。没有新增browser、运行操作或读取entrycredentials/roomIds/完整快照/数据库。

root的脱敏integrity描述实际2席自然流程273条唯一命令、274个连续journal/version、turn6、playing；277个已接受HTTP回执含4个重复回执，1次取消、2次付费附属、4次lostACK，保留原HTTP记录无重建。WASM/native只读回放与数据库字节不变由root审计通过；本审查绑定和引用其结果，没有自行读取journal/数据库或将其写成独立回放。playing状态不是完整策略终局，engine43合作终局与此engine45序列无替代关系。

独立重算root `combined-web-tests.log`、`combined-worker-tests.log`、`combined-build.log` 的SHA256，与 `combined-validation-summary.json`完全相同，并实际检查最终测试汇总：Web68文件607项、Worker25项/0失败，TypeScript/Vite/Worker dry-run通过。它们是root组合全套运行的日志，独立审查者未重跑组合全套；本审查自己跑过的测试仍是cf2原图组件116项与TypeScript。实际组合server产物也独立读字节核对：唯一当前WASM2198305字节/SHA256 `afc1b42523111dc242fd21850da9af243112a619c9f759bc60cbabac045883f7`；Worker `index.js`47263字节/SHA256 `384050c24c4e79af52ed002036aa9400aa1e9aff077dd4b154a0f1b4bdbb71ea`。没有历史WASM混入该current-only build。

最终范围内未发现新增P0–P3问题；原图、组合绑定与实际截图审查通过。独立机制正确性、完整策略/平衡和服务器隐藏信息投影由相应独立机制/运行验收承担，本报告不扩张到这些范围。最新脱敏证据是 `natural-ui-screenshots-actually-viewed.json` 和 `natural-ui-and-validation-review.json`；新增8图的hash、driver、root汇总、组合日志和actual build均绑定。当前原图任务只覆盖既定JC089/JZ51/JZ52/JZ53，没有启动额外BULK原图任务或生成TK011。
