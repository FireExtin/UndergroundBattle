# JC035 / ABI89 本地独立视觉复审

结论：**通过本次限定范围的独立视觉复审及既有证据一致性核验，未发现新增的具体阻断问题。** 最终截图未呈现前三地区的我方势力行，因此不能声称从像素直接读到了 1/1/1；这些结果由精确 Native 原始视图及既有 HTTP/D1 逐步比较支持。该截图覆盖限制不等于已证明界面永久遮挡或无法访问。

复审者：独立子代理 `/root/jc035_independent_visual_review`。本代理未参与 JC035 实现或实现者自查；本报告由实际本地看图、只读代码与原始材料交叉核验形成，没有以作者的 `passed` 字段替代审查。完成于 2026-10-20 UTC。

## 审查动作与身份

- 使用本地 `tools.view_image({detail:"original"})` 打开四张原始 PNG，每张再通过 `image(result.image_url,"original")` 实际显示并观察像素；没有裁切、重采样、生成替代图或仅凭哈希宣告看图。
- 四张均为 RGB、1366×900；记录的浏览器 viewport 为 1366×768，驱动 `fullPage:true`，两者一致而非分辨率矛盾。
- 仅阅读仓库、原图、pkg、生产/frozen 相关材料并计算身份；未运行测试、构建、WASM、浏览器或网站；未上传、访问 Library、改凭据或权限、提交推送、发布或合 main。新建文件仅为本目录的 `report.md` 和 `review.json`。
- 实际仓库 `/workspace/UndergroundBattle`；分支 `codex/jc035-blue-preparation-review-20261019`；审查时 HEAD 为 `d3105cd0e8a45ab2de0fc2ea5feef9016d14f6d4`，工作区干净。驱动、选牌组件、16 项前端测试源码、草稿存储源码、JC035 机制源码和 Native 夹具分别与代码/见证检查点 `100161b628e56b9153041d952ac0f5d83428ca88` 的 Git 对象逐字节相同。

## 逐张原始像素观察

### 1. original-jc035.png

[原图](/workspace/jc035-recovery-20261019/p2-observation-integrity-20261019/after/browser-four-clients-p2-01/original-jc035.png)

567,912B；SHA256 `294dfb5ad8f362c04643ccf2d8671d841202a062c298bb2fff7bc4e12dd5004f`。

直接看见中央“卡牌档案·放大阅读”面板，当前为“原始牌面”页；上下文为 P0 拥有、附着于地区死者之城、地区 1、同名对象 #i256。完整卡面已加载，未被页脚截去：标题“传媒魔咒”、费用 3、左侧蓝色忠诚方块、类型“附属·结界/媒体”、规则“结附于目标地区”及准备步骤触发正文可辨认，正文明确包含对手手牌少于或等于三张、可以在本地区放置一个本方势力标志。调查及心灵符号可见。画面底部自身手牌为四张无知路人；背景其他界面失焦，不能用其证明其他席位细节。

原图与目录字段交叉后支持 3 费、蓝忠诚 1、非独有、附属/结界/媒体、心灵 1、临时调查 1。非独有与图标语义不单靠“小图上未出现某符号”推断，另由原始 catalog 和测试字段佐证。页脚提示结合当前状态修正查看；原始牌面不能独自证明当前实际贡献图标。

### 2. restored-pending-draft.png

[原图](/workspace/jc035-recovery-20261019/p2-observation-integrity-20261019/after/browser-four-clients-p2-01/restored-pending-draft.png)

470,749B；SHA256 `64a73ca3078a15fa245fb71ff797c8b119313c7610b176428387067fa258d19d`。

直接看见“传媒魔咒：选择下一项准备触发”，说明可选择下一项发动入栈、放弃剩余触发、后入栈先结算。计数为“已选 1/1”。触发 1 为 P0 拥有、地区 1、#i256，显示“换选此项”；触发 2 为 P0 拥有、地区 2、#i257，有明显金色选中边框和“已选·再点取消”。两项费用 3、名称、类型、蓝忠诚与心灵文字可见；下方固定栏显示“下一项入栈：地区2：放置1本方势力”，跳过和金色确认按钮均清楚可辨。

这是可读、可区分来源的静态选中画面。图片本身不能证明曾经重开 D1 或刷新恢复；该时序由驱动在捕获前的 dispose/start、四席 reload 和原快照比较支持。选项卡下部在固定栏以外的内容未全部呈现，不声称看到了这些隐藏文本或阅读按钮。

### 3. next-source-after-reopen.png

[原图](/workspace/jc035-recovery-20261019/p2-observation-integrity-20261019/after/browser-four-clients-p2-01/next-source-after-reopen.png)

455,210B；SHA256 `bfc7fbe5d9884808ce29098e6c69d7ae647fb281c7d16e0fddecfce23b14d21f`。

同一类准备来源对话框里只剩触发 1：P0 拥有、地区 1、#i256。计数“已选 0/1”，选项为“点击选择”，没有上一张的金色选中边框；确认按钮灰显，独立的跳过按钮可见，页脚没有“下一项入栈”已选摘要。由此直接支持“该画面没有沿用 #i257 草稿”。卡内可见的“先手”调查 1 是印刷临时图标栏；不是地区当前实际对抗贡献。Native 当前先手为另一团队，因此其有效对抗图标为 0，与卡内这一印刷栏不矛盾。

按钮的真实 disabled 属性、下一 choice 的产生、双击只发一条请求等动态结论另由组件断言和已记录驱动佐证，不能仅凭灰色像素完成这些证明。

### 4. resolved-three-regions.png

[原图](/workspace/jc035-recovery-20261019/p2-observation-integrity-20261019/after/browser-four-clients-p2-01/resolved-three-regions.png)

996,718B；SHA256 `e14be73d19903fcfb8160ad96056dd5273616006755ddbc85616e966c6690855`。

直接看见牌桌已无待选对话框：第 2 回合、开始阶段/重置与准备，先手与优先权 P2/P3、行动团队 P0/P1、等待 P2/P3 行动；最近让过显示 P1。五地区按序为死者之城、沉没的废墟、佛罗伦萨、死者之城、莫斯科。前三地区各有传媒魔咒附属按钮，旁边数字为 1、1、2；通过 `Table.tsx:169` 可知它们是操控席位徽记，不能误读成势力标志或副本数。P2 手 3、P3 手 4、P1 手牌背面及 P0 的四张正面手牌可见，没有在本截图中看到对方手牌正面。

前三地区我方势力行处于中央滚动区未呈现的下部，视觉上在下方玩家区边缘被截断。**没有从本张像素直接看到前三地区势力 1/1/1，也没有从图片读到最终 UID。** 第四、第五地区可见我方势力 0。代码 `Table.tsx:469` 明确将中央地区作为可上下滚动区域并给出滚动提示；`lantern-table.css:388` 为 `overflow-y:auto`，`:395` 显式为有地区附属的卡列提供四行布局。因此本证据说明未滚动截图覆盖不足，没有证明永久遮挡、无法滚动或操作被阻断。本复审不运行浏览器去补这项交互证据。

## 捕获顺序与动态证据交叉核验

[驱动](/workspace/UndergroundBattle/tools/frontend/jc035-http-four-clients-qa.mjs:52) 与原始 [report.json](/workspace/jc035-recovery-20261019/p2-observation-integrity-20261019/after/browser-four-clients-p2-01/report.json) 相符，且仓库中的浏览器报告副本与原文件逐字节相同。

1. 从 `same-controller-order` 的精确 Native 待选状态开始；四个独立 Chromium context 分别绑定席位，真实本地监听 HTTP → workerd → 十三个原始 WASM 模块 → 持久化合成 D1。不是线上实局，起始布局为合成状态，不代表普通建房流程从头运行。
2. `:53–55` 打开阅读器，先断言文字、再切原始牌面，确认 `/cards/JC035.jpg` 成功加载后捕获第一图；关闭阅读器后 D1 快照不变。
3. `:61–65` 首先点击另一来源，再换选真实命令要求的 #i257，断言仅一个 aria-pressed=true、已选摘要存在；先重开 workerd/D1，再四席 reload 并比较持久化快照，确认 #i257 仍选中后捕获第二图。草稿为本标签页 sessionStorage 内容；这不证明换设备或关闭整个浏览器后恢复。
4. `:68–75` 对确认执行双击，核对唯一新增 POST、精确 action/expectedVersion、WASM/Native 结果、D1 状态、journal 及四席投影。首个选择后再次重开并 reload，捕获第三图。
5. 三项声明及十二次让过共 15 条 UI 命令结束后，第三次重开；15 条同 command 重送均得到原回执且快照不变，四席 reload 后捕获第四图。报告记录 `reopened=3`、`originalReceiptsAfterReopen=15`、`pageErrors=[]`，保留原始退出记录为自然退出 0。

报告 `posts=[10,8,6,6]` 合计 30 包含最终 15 次原回执重送；不能把它误算成 30 次新状态变更。驱动逐步只发一次新增请求的断言与双击场景相对应。此浏览器场景三次均选择来源，没有实际点击跳过；跳过的机制及 UI 提交由其他 Native 场景和 13 项组件向量支持。

本代理独立解析 gzip 的 `wrapper.raw.json`，按 Python 整数处理，没有重新序列化或改签 opaque 状态。UTF8 原文 19,576,994B、SHA256 `85a5b7bf3bdb89f162d8a37efb2bade6e00387287f459456b5124be2cd68e1a9`。起点为场景 offset 4、version 12，状态 SHA256 `1e7d8e10e7aab07299f07cf19a596e1b352f072e682c0cf37dd66c0a71cc5e63`；截出的 15 个 expected.state 哈希逐条等于报告 15 个哈希。

选择序列为：P0 在 choice i260 选择 #i257/地区 2；P0 在新 choice i262 选择 #i256/地区 1；P1 在 choice i264 选择 #i258/地区 3。第一次选择后的新待选仅包含 #i256，与第三图吻合。最终 version 27，四席投影均无 pendingChoice/waitingChoice，stack 为空；前三地区 influence 均 `[1,0]`，第四、第五为 `[0,0]`；原地区实体依次 i239/i201/i202，来源仍绑定 #i256/#i257/#i258。图中其余可读回合、地区、手牌数、先手和行动权与 Native 视图一致。最终 1/1/1 属于数据与既有动态比较结论，未升级成直接像素结论。

## 机制、选牌与权限/隐私范围

[机制](/workspace/UndergroundBattle/rust-game/src/jc035.rs:191) 在准备开始记录存活席位手牌数，按敌我与当时 ≤3 资格产生来源。后续增长不撤销资格，是本批已明确的项目规则；本复审没有新增结算复检要求，也不称它与 XQ37 完全相同或官方专属 FAQ。`:218–268` 来源选项绑定原卡及原地区；允许接受一项或放弃剩余，dispatch 到可响应帧，之后再给新的 choice；`:323–337` 每帧仅放 1 点于仍匹配原 UID 的地区。

[ChoicePanel](/workspace/UndergroundBattle/web/src/game/ChoicePanel.tsx:89) 对 JC035 triggerOrder 要求明确选中后才能确认；`:98–101` 支持点击已选取消、单选直接替换；`:202` 给出触发编号、拥有者、地区、实例标记；`:211–214` 分别呈现已选摘要、跳过与确认。其 key 使用 choice 指纹，草稿指纹包含 choice.id/playerId/选项 UID；存储按标签页、玩家范围、room、viewer 绑定，[存储逻辑](/workspace/UndergroundBattle/web/src/game/choiceDraftStorage.ts:12) 与刷新保留、下一 choice 清空一致。

既有 [JC035Native89.test.jsx](/workspace/UndergroundBattle/web/src/game/JC035Native89.test.jsx:14) 的 16 项是 13 个真实 Native 选牌/放弃向量，加上重挂草稿/下一来源、非行动者与内部观察、整卡阅读三项。原始前端日志记录 16/16、自然退出 0；本代理复算解压日志哈希与退出记录一致，没有重跑。浏览器原始日志及退出记录也已如此核对。

权限和隐私结论限于这批 Native/组件/本地浏览器证据：组件测试检查非选择者没有选牌控件；四客户端驱动在 reload 时按四席 Native pendingChoice 核对对话框数量，并在每条命令后比较四席完整公开投影。本代理静态遍历 620 个 Native 公开视图，未见内部 `jc035_preparation` 键、另一玩家的 pendingChoice 或自身 hand 数组夹入他人拥有的牌。最终截图只证明其所显示的席位视角没有对手手牌正面。

这些不构成真实多设备凭证隔离、任意恶意请求或整个应用安全性的独立验证。Native 的四次拒绝实际为 `room_paused`，不把它们误称为四次越权选牌拒绝。四个浏览器 context 的真实动态结果来自已保存运行，本代理没有再次执行。

## 固定产物身份

本代理自行读取并复算以下身份，均匹配委托指定值：

| 项目 | 字节数/范围 | SHA256 |
| --- | --- | --- |
| 当前 ABI89 WASM | 3,471,247B | `df890ed0b596ca245a39bc8466270edc2e8f40f92ef8dd34359534342ccf24de` |
| 标准生产 `sites/dist/server/index.js` | 264,190B | `695c17c25667d165df429351ab65414dfd7d749e4b5c9a7837a7cfc851e4fb45` |
| 本地时钟测试 `worker.js` | 251,078B，仅既有本地测试用途 | `eeb4af7441cb2c4546c934e01b748694c5e37f19a9657d764de2437a4e458221` |
| 完整生产清单 | 46,909B；187 文件/142,257,627B | `28b461d38f3d0140ef7413c62509bbd58be46f88dea5f4141bcab054f6bc4da8` |
| `web/public/cards/JC035.jpg` | 341,309B | `e7f2cff71f3a511d5065ece513db9013d02e5685a7d7ee899de4afbc109fe319` |
| 浏览器驱动 | 11,937B | `e01a230afd69efc650f0856519cd67ca4df7263a3e5263d2b9a083cd29ac275b` |
| 原始浏览器报告 | 6,344B | `32598211ebe5abb4b2332d76108e4612b2807c9d68cc9033be1a7fe5e6fa499f` |

另逐项核对当前 pkg 的全部 5 文件、生产清单全部 187 文件、时钟 Worker 目录中报告列出的 13 个模块，以及 Native 141 个编译输入，均无缺失、大小或哈希差异。生产 Worker 与时钟 Worker 身份严格区分；本报告不将测试包装认定为待发生产文件，也不替代旧环境实际载体的最终核验。

## 既有失败、限制与剩余门槛

[完整 HTTP 汇总](/workspace/UndergroundBattle/docs/evidence/jc035-p2-2026-10-20/after/http-jc035-complete.summary.json) 仍为原完整轮 73/74、自然退出 1；ordinary-attach 为同步代理 fetch 的 `SocketError: other side closed`，单项同源补验 1/1、自然退出 0。报告明确 `oneUnfilteredGreen74Run=false`、socket 关闭根因未证明。不能合写成一轮 74 全绿，也不能称根因已修复。50 坏档 500 次 HTTP400、11 合法恢复及 1 真实续行保持其既有证据范围；本次没有重新执行。旧完整 Worker 717/722 及其各文件补验同样不能被本次视觉通过覆盖。

没有新增视觉或一致性阻断。剩余事项为父代理最终综合裁定与任何未来发布前的固定候选/原 ABI77–88 实际载体身份核验；此处不擅自新增全量重跑要求，也未取得发布或合 main 的授权。前三地区势力数的直接像素覆盖、中央区域滚动后的可读性、其他分辨率/移动端以及其他角色截图未在这四张原图中独立证明，不能扩展本次通过范围。

本次是用户明确授权的同环境本地独立复审，已实际完成；旧材料里“待独立像素审查”不再描述本代理的这次工作。既有 Library 401 交付记录保持历史原样，本次没有尝试传输，也没有声称 Library 上传成功。没有线上实局验证。所有旧证据及仓库文件保持未改写。
