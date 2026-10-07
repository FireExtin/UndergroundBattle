# JC089 独立机制审查：最终结果

最终产品组合 `84a36d624528ab61f752e4ad9f05fdeb1839d7e4`，机制修复 `a96a5595d42b58ecf2d47abd4d4a87b3d6c7d1ae`：原P2反例已闭环，修复与真实双席UI证据补核均通过，没有未解决的P0/P1/P2/P3问题。以下原19候选P2结论是已关闭的历史记录，原完整初审和闭环阶段报告均另行冻结保存。真实UI是第6回合的流程验收，不是完整策略终局，也不是JC018特定自然对局。

# JC089 独立机制审查（进行中）

审查者为本批新独立 reviewer；没有修改产品、部署、推送或访问 Library。当前已审固定机制提交为 `19e594f00004c16c95cc1846c4ad3d252c5b0021`，基线 `99a208c0e6dc73669292396268840a0d9034547a`。先前 `c9821cb65d088aa41b0d90c1e8de34289697b9a7` 与 19e594f 的 Rust、WASM、Web、Sites 产品树完全相同，只有候选文档一处 UI 范围文字不同。工作树实际可读、审查后仍干净。

## 当前结论：P2 阻断，需修复再审

**[P2] 移除最后诅咒会让同一印刷威名再次作为声望奖励。** `rust-game/src/renown.rs` 的 `has_renown` 只在 `has_jc089_glory` 当前为真时抑制 JC018 旧的 `traits.renown` 绑定。JC018 赢战斗并接受 JC089 威名后，在响应中付费打出 JC005 裂解术消灭最后诅咒，已冻结威名仍合法结算 1 点；此时 JC018 的 `currentRenown` 恢复为 true。随后势力对抗平手，全地区对抗结束又提供一次声望选择，接受后本方影响力从 1 变为 2，没有任何独立真实声望授予。

审查者用披露的离线 fixture 只初始化一次测试布局，随后通过当前生产 WASM 的 31 条合法 `applyRoom` 命令（含真实付费 `BeginResponse`、`SubmitResponse`、让过和选择）独立复现，没有修改自然 UI 房间或生产数据库。驱动和脱敏结果分别位于 `/tmp/review-jc089-mechanism-evidence/removal-boundary-repro.mjs`、`removal-boundary-result.json`。此前候选中保留移除诅咒后旧基线的说明，不能豁免这条由新 JC089 奖励直接暴露的重复计分路径。根研发已收到阻断结果；本报告尚未接受任何修复候选。

最小修复建议：有限纠正 JC018 原印刷威名的误映射，移除其错误 `traits.renown`，让固定的 JC018 威名与真实 JC089 附属授予共用一份现有战斗奖励筛选和非叠加声明。独立 `grants_renown` 保留。无需新历史标志、队列或通用关键词框架。JC018 原图、目录印刷属性及历史引擎字节应保留。

## 实际读取的规则与代码

读取完整 45 文件差异，含当前生产规则、属性、对抗奖励、运行时声明验证、视图隐藏字段、Web 显示及所有测试改动；冻结 WASM/JS 用字节核查。实际查看 JC089、JC018 原图，《霸权说明书》印刷18/实体19、印刷15/实体17、2v2 印刷21/实体22，《规则手册》印刷4/实体6、5/实体7、6/实体8、9/实体11，FAQ 整页2、4。

原图与规则明确支持：JC089 费用2、黑色忠诚1、血脉/鲜血领域1、结附任意目标角色、附主防御-1、一个永久与一个临时战斗图标及威名；威名在赢得战斗后可选发动，不要求敌方参与或击杀，同地区不叠加。声望在全部对抗结束后比较未横置角色数量，二者是不同关键词。附属不随附主改变操控者，横置不抹除持续属性，附主离场或翻暗消灭附属。已入堆栈的效果可在来源离场后继续生效。

对有限实现的其他审查未发现新问题：所有正防御增益汇总后再扣除诅咒数；每一真实附属实例分别加属性；角色横置抑制参与、诅咒横置不取消持续效果；战斗前冻结真实附主实例和当时操控者，`PlaceInfluence` 的真实地区实例与来源地区下标共同守卫，地区同下标替换不收获旧奖励。定义完整白名单拒绝 modifier 移植，印刷定义拒绝 `CombatWon`，运行时固定完整形状允许冻结来源已离场的存读。2v2 最小合资格操控者/既有桌面顺序是项目限定政策，报告没有把它声称为官方指定投票规则。

## 审查者独立执行的检查（19e594f）

- 原生专项 23 项全部通过，使用已有 Rust1.90 工具链、locked/offline/jobs1 与共享 `/tmp/jz49-native-target`；完整执行日志 `native-focused.log`。
- Web JC089 专项 9 项全部通过；两项 TypeScript 检查通过；日志 `web-focused.log`、`typecheck.log`。先比较 package-lock 一致，再暂借依赖链接；链接已移除，产品工作树保持干净。
- 当前 WASM 对原生 oracle 的完整独立重跑通过：579 条命令、5 条原子拒绝、1299 个存读检查点、13 条连续真实响应链、7564 份四席投影。状态 u64 保持 opaque 文本，没有通过 JS 解析重建。日志 `wasm-compare.json`。
- 独立逐文件核查 204 个基线历史文件与基线 Git blob 完全一致；5 个新增44冻结文件与已发布 Site36 的实际44 pkg 完全一致，共40个历史目录、209文件。当前构建字节清单9条实际匹配，包括 WASM 2,197,622 字节、SHA256 `ffbabae8bdb36352ae11acab613365ea3bdcdb0681d246a8d11b354864f01099`。记录 `historical-byte-verification.json`。
- 实际读取研发全套日志：568项原生、566项 Web、25项 Worker/D1 均通过。审查者没有无必要重复完整原生和 Worker 检查；上述全套不能替代修复后最终验证。

## 尚未执行或尚未绑定

本批真实自然浏览器操作、HTTP/SQLite 重开和其原生/WASM 回放尚未提供；不能声称 UI 已通过。原图 `cf2b9657c9f4e0ba8dcb808aaa1e0c2afc80a0e4` 是另批，本机制审查不代替原图审查。根研发已收到 owner 对 JZ50 搜索0张仍洗牌选择 B 的裁定，该机制属于后续独立批次，没有加入本固定差异。

## 原31命令冻结补充

原反例脚本、一次初始化的 fixture 和旧错误结果逐字节复制至 `/tmp/review-jc089-mechanism-evidence/frozen31/`；脚本 SHA256 `4d19e7b14d27d35186b98b45ed5952804cfdc099596c450d0c71772c47d5322f`，fixture SHA256 `1bbf3caa420d799ccc1e8d774dce3537377972a09bc8c195d1c1582cb23550c6`。新增的派生日志捕获脚本只记录输入，未更改原始脚本、布局、费用、目标、付款或响应顺序；再次执行旧生产 WASM 31/31 接受，结果与原错误结果逐字节一致。

精确命令流 `frozen31/command-stream.json` 共31条，9202字节，SHA256 `df861682a177b6d7db036127c329c7eca58a122ade492c0fcb41156abaf2fbb8`；每条含座位、原 commandId、原 expectedVersion、action 和 serverNow。第27条为错误声望的接受，第28–31条为其响应让过。修复闭环将使用同一初始 fixture opaque 文本和同一31输入，逐条记录实际接受／拒绝；不会修改费用和目标，也不会重新造已消失的 choice。原脚本明确期待错误声望，因此不应被要求在正确产品上原断言全通过；会使用单独的正确断言核查该错误选择不存在／拒绝、总奖励只有1。最终修复提交及其闭环尚待提供。

## 修复闭环：a96a559（机制审查通过，自然UI待补）

最终独立审查固定修复提交 `a96a5595d42b58ecf2d47abd4d4a87b3d6c7d1ae`，相对原候选19e594f读取全部18文件差异与新修复说明。旧首次报告完整保存在 `/tmp/review-jc089-mechanism-evidence/initial-review-before-a96a559.md`，原P2历史、原脚本、原fixture、冻结31输入及旧19包均未覆写或删除。本次未修改产品。

**原P2已关闭；修复差异没有发现未解决的P0/P1/P2/P3问题，可进入新的组合本地自然UI验收。** `jc018_definition` 删除伪 `traits.renown`；有限 `has_combat_glory` 只接受在场真实JC018固定完整定义及真实JC089结附授予。两种来源共享一份既有战斗获胜声明，同地区不叠加。没有扫描任意卡的关键词授予通用能力，没有新历史奖励标志、队列或身份机制。独立真实JC074声望仍有效，JC018原图与印刷目录字段保留。无诅咒JC018现在按原印刷战斗时点正确奖励，这是关闭新交互漏洞必要的有限纠正。

审查者新写的回放驱动 `/tmp/review-jc089-mechanism-evidence/replay-frozen31-closure.mjs` 逐条使用同一初始opaque fixture文本及原31条命令的seat/commandId/expectedVersion/action/serverNow，未改任何输入。旧包31条全部接受、最终势力[2,0]；修复包第1–26条全部接受，第27条原伪声望accept返回 `invalid_action`（当前没有行动权），第28–31条原版本返回 `version_conflict`。全部5条拒绝的state字符串与输入state逐字节一致、journal为空；最终势力[1,0]，附属0、pendingChoice为空，JC018印刷威名为true、伪声望为false。原脚本的错误断言没有变成通过标准，未重造不存在的选择。逐条结果见 `exact31-closure-independent.json`。

修复新增的正常合法流程测试没有提交伪声望选择：付费JC059支持、JC089结附、JC005在威名结算前或结算后移除最后诅咒，均只有一次威名。另经真实让过推进完整回合、清理、先手轮替：第一次拒绝不补发，下一轮赢战斗可以再次正常发动，没有同轮手工重开战斗或错误once-per-game机制。印刷与授予两种来源均测试控制权/来源离场、同下标地区替换及响应前后；隐藏/横置/无参与/失败/平手/其他对抗不触发。

独立执行通过：26项原生专项（22.49s）、10项Web专项、两项TypeScript检查；最新WASM对照938条命令、5条原子拒绝、2033检查点、13条真实付费响应链、11936份四席投影。日志和结果在 `closure-a96a559/`。完整研发571项Native、567项Web、25项Worker/D1日志由固定候选证据提供；没有把修复前旧568/566日志当成最终验证。

独立核查209历史文件与固定19 Git blob逐字节一致；9条当前构建artifact均匹配实际字节。当前WASM 2,198,305字节，SHA256 `afc1b42523111dc242fd21850da9af243112a619c9f759bc60cbabac045883f7`；原生审计二进制SHA256 `f77a6eb184af636a410c0a0fbdfdf85af8422914e57d672e42708f69d79f470a`，67项构建来源清单与a96 Git文件及实际文件一致。

根组合产品 `84a36d624528ab61f752e4ad9f05fdeb1839d7e4` 的Rust/WASM/Sites源树逐ID等于a96：Rust `6a7a2afe39e719925a2879d01a1c8df91bd17e35`，WASM `acd84940795276c1d9b9226eb27230c8976bbba0`，Sites `67de5b05acf41ad066c6e5b6487d6d09e2bd8214`。原图cf2是另批独立审查范围。临时依赖链接已移除，a96 worktree clean。

截至本补充，根新的组合完整Web/build/Worker及自然双席UI、HTTP收据、同SQLite重开、原生只读审计与WASM逐journal回放尚未提交审查者，本报告不声称其完成。自然UI将聚焦真实付费JC089/JZ48的属性/威名/死亡/清理/重试等流程；它不会冒充JC018特定自然局或完整策略终局。JC018反例闭环来自本冻结31输入与26专项。Site发布和main合并不属于本审查授权。

## 最终真实UI证据补核（84a36d）

审查者读取根研发实际浏览器driver/harness、全部原始HTTP/最终SQLite快照/journal/progress、原生只读回放证明与组合完整验证摘要。实际查看8张截图：结附前同实例对照、JC089原图、目标确认／取消、第一诅咒属性、战斗威名恰好1点、丢失ACK恢复、第二诅咒致死与清理、移动端幸存者阅读。原图完整显示，未裁剪文本；附咒角色显示威名，存档恢复后的幸存JZ48回到防御1/势力0。目标选择取消与付款流程使用真实UI。

独立pure WASM使用原真实create请求的合法custom50及seed4952重建初始状态，初始opaque存档逐字节等于实际DB initial_state，证实普通建房路径与唯一受控seed一致。随后独立回放1次JoinWithDeck和273条普通命令；全部273条WASM生成的响应视图逐条等于真实DB receipt，最终opaque state逐字节一致。最终状态SHA256 `3c0f8d0783037b11b1c10a01973581299978258fc24c7eaa8c1b4054ffc4509c`，273命令、274连续journal、version274、turn6、status playing。两席最终手牌投影只包含本人的手牌；UI截图中的敌方手牌保持牌背。原生审计摘要也绑定同一最终digest及固定f77二进制，审查者没有无必要重跑原生审计。

独立全量核查279条原始HTTP记录：2条建房／入席，277条成功commands响应，对应273唯一命令和4条重复响应；同一命令实际出现5次，每次payload／response相同。每条HTTP响应、每条浏览器progress步骤、每条journal里的Command payload／seat／version均对应唯一持久化receipt。没有倒填或重建HTTP记录，driver从最初create实时记录。

独立以SQLite URI `mode=ro`、SELECT-only一致读取rooms、commands、journal、seats、entry_receipts五类实际表，与最终快照逐项相同；DB/WAL当前字节哈希等于根原生审计的before/after证明，本次独立读取后仍不变。原始房间ID、entry凭据、全状态、QA数据库和HTTP body留在本地，没有写入本报告或仓库交付摘要。

真实流程包含两次合法付费JC089结附、一次纯UI取消目标、第一诅咒后的战斗威名恰好增加1点；第二诅咒让附主防御归0死亡，两诅咒均进入owner墓地，幸存JZ48失去同控制者另一罪犯支持，回到防御1／势力0。丢失4次ACK后，真实UI保留pending命令、跨reload使用原commandId并取得原收据，没有新增D1命令／日志。两席reload与同SQLite重开deep equality实际driver断言通过。最终地区标志包含普通势力奖励，因此本UI仅声明威名这一步额外1点，不把最终地区全部标志误写成1点。

组合完整607项Web／68文件、25项Worker/D1日志实际读取并核对SHA；TypeScript/Vite/Worker build摘要与9条当前构建artifact实际字节匹配，Worker只有1个当前WASM，SHA为 `afc1b42523111dc242fd21850da9af243112a619c9f759bc60cbabac045883f7`。8张实际截图的字节和SHA均匹配根摘要。原图批次cf2的完整审查仍由独立原图报告负责；本补充核验真实JC089原图阅读与机制流程，不冒充其他三张原图的自然实战准入。

独立证明和驱动：`/tmp/review-jc089-mechanism-evidence/natural-ui-replay-independent.json`、`natural-ui-integrity-independent.json`、`verify-natural-ui-replay.mjs`、`verify-natural-ui-integrity.py`。最终脱敏摘要：`final-independent-review-summary.json`。固定机制、旧19候选、cf2原图、冻结31输入和QA数据库均保留，未改产品或发布／合并main。
