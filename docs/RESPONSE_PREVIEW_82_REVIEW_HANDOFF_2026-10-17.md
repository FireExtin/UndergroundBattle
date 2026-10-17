# ABI82 连锁前只读来源预览：串行独审交接（未发布）

源码审查点：`398e220721f9e14c4f523530183c6288b7749096`，分支 `codex/stage-pass-progress-20261016`。本批不增加卡牌、规则、支付路径或D1迁移，只在新82房间投影中增加可选来源摘要，并让现有UI在真实BeginResponse之前显示候选类型与来源。前一源码检查点`9e549277…`标WIP，随后此点仅修正JZ46测试的全局当前身份导入；Rust／生产前后端及169个运行文件均未变。最后完整Native1183／前端1283／sites279均通过，最终原始日志与身份见同批证据提交。完成后仍须父会话串行独审，不自动发布或合main。

公开站仍是v63，官方source `90bfb25faa4c142f13140e141b6c8dfe2ec9355a`、deployment `appgdep_6ad34a79d1808191b2c31056c5e9652b`。本批没有修改该官方源、可见性、权限、D1或发布版本，也没有启动Actions。11:17:01普通浏览器小时复查仍为`ERR_CERT_AUTHORITY_INVALID`（1GET／0POST／未到UI）；这里的成功浏览器检查全部是本地固定时钟合成牌桌。

## 实现与边界

- `rust-game/src/room.rs`：在原本已经计算好的本席`GameView.legalActions`被Room的“未Begin只保留pass”投影过滤之前，选取现有响应kind，按`(kind,cardId,abilityId)`去重。只给未决定、可Begin、未暂停且真实截止时间尚未到的本席，其他席位各算各自投影。
- `previewSources`最多包含`kind`、可选实例`cardId`及可选`abilityId`；不含目标、费用选择、标签、卡牌定义或可提交的LegalAction。无额外试算、intent、ID分配、支付或`&mut self`调用。实际提交／报价仍要求真正的composing intent；伪造未Begin的Submit依旧拒绝。
- `web/src/game/{types.ts,api.ts,actionHints.ts,Table.tsx}`：兼容旧字段缺失，只用现有卡牌目录的能力timing分类；只高亮当前投影中可见且由本席控制的暗置来源，不从目标推断来源，不解析卡面规则。预览高亮不进入`legalActions`或提交集合；文案明确先连锁再提交。busy／不确定回执／离线／暂停／选择中关闭提示。本地截止计时只收起提示，不发请求、不延长服务器时限。
- 原hand／己方暗置／资产／附属／秘社等来源展示继续复用原分类和实例映射；没有改动自动让过、Begin、Pass、Cancel、Quote或Submit协议。

测试布局的实际预览为：

```json
[{"abilityId":"destroy-human","cardId":"i251","kind":"play"},
 {"cardId":"i244","kind":"reveal"}]
```

这个字段只是来源摘要；同一时点实际`legalActions`仍只有pass，`myIntentId`缺失，点选两类来源都不能直接响应。普通`state` HTTP原有到期`Tick`仍可能持久化；“只读”指新增投影，不把到期轮询或普通HTTP服务统称零写入。新HTTP测试分别核对到期前的零写入与5000ms真实Tick。

## 六核身份与旧房保留

当前新房是pool74／engine82，卡池147张及整个catalog除engine身份外逐字段等于v63。Room schema仍为3；新Room拒绝旧engine身份，路由按完整原tuple选择原核，不改写旧state或回执。

| 内核 | SHA256 |
| --- | --- |
| 新82 | `1479c3d2ac98fa0f3c0aefff4eba409f2f84a28f437833cbb0a654332b752d37` |
| 发布81 | `669403d547a0ce451570e7953195315949a8ed8252ede34aa9269a607a4e0583` |
| 发布80 | `0229f75b7af79035f2a4025fc11c6de6f3f0b895127cc8b537f829268d030472` |
| 发布79 | `8fdc829ab25d0eb5d68d973489fafff6b70e0da8b7837621a3ef86d52360f0fd` |
| 发布78 | `9511a72c960cf3e0fa249d62669724875d475ab245de4f9616615f38df43fd5f` |
| 发布77 | `b0e6e59864ca064338050fe47a4b5439c32aacfcb6d257840bbd49976327953c` |

原81的127项Native绑定、raw原文SHA `ec28c25693e2d74db2bd6ec766fc7b6f8d94d938ca21abd5a8d51ebf1afc425e`、gzip黄金文件SHA `13c5728ff7342130ad95201134b84cd011d340f86e9a56cd909e90c0ea787c82`全部保留。`current81-kernel.mjs`现在只读冻结原81；`tools/jc054-oracle.mjs`固定原绑定和原字节，不能用82覆盖或重签历史Native81。原8场／214命令／888投影／44坏档／528恢复调用仍逐项对照原81。

生产以及当前本地固定时钟Worker都核对六份WASM。既有pass-history专项仍用单独的原77专用wrapper，7个HTTP／service相关源码与v63记录时完全相同；该历史依赖单独记录，不冒称新82六核运行物。部分继承测试标题中的candidate80／engine63是旧场景命名，实际路由和catalog断言按原房间身份及当前全局82核对。固定时钟构建器现将已有server目录完整改名保留，再输出到空目录，避免Wrangler遗留未引用模块。旧含7份WASM的测试目录仍完整保留；最终完整Worker和浏览器使用独立的`/workspace/response-preview-evidence-20261017/clean-worker82/server`。生产`sites/src/index.mjs`仍使用真实服务器时钟，测试时钟绑定只存在隔离wrapper。

## 独立参考输入与验证

新Native82由真实Rust测试导出2,405,888字节原文，SHA `e1b08ea112e7f81d369c13f21743700470474800eb54d4c3d7de11d9249543b2`，128项编译输入全部匹配源码审查点；压缩fixture175,690字节，SHA `0242797053882360449e6d6922c6eb8652229b4086bdefc7da4ce961deba4da8`。Node未重序列化或修改opaque状态。3场景覆盖预览后真实现身、暂停／4999ms无操作／5000ms到期，以及同伴让过／Begin／取消；共8命令（1拒绝）、2轮询、12次纯view观察、100份四席投影和2报价（1拒绝），完整状态、结果、视图及错误与新82 WASM一致。

| 检查 | 本批结果 |
| --- | --- |
| 新Native专项 | 4/4，包含在完整Native中 |
| 完整Native | 1183/1183，0失败／0忽略，924.44秒（WASM包本机lib另0用例） |
| 完整前端 | 1283/1283，135文件，378.994秒 |
| 完整sites套件（HTTP/workerd/WASM/D1及单元检查） | 最后单轮279/279，0失败／0跳过，992.807秒；首完整轮267/279另列 |
| 源码／卡面／路由专项 | 65/65（其中路由与完整Worker重叠，不相加） |
| 冻结81专项 | 2/2，原81完整对照通过 |
| 新82 Native/WASM | 上述3／8／2／12／100／2完整相等 |
| 构建 | 两个TS配置、Vite、六核production Worker dry-run通过 |
| 本地生产界面浏览器 | Chrome151，两尺寸各4席，16项流程断言、10个可达位置命中、0页面错误 |

浏览器使用实际production GameApp资源、监听HTTP、workerd／六核WASM和本地合成D1；没有mock API或改写opaque状态。1440×1000及1366×768各验证：预览与刷新0POST、己方手牌／暗置来源和对手隐私、客户端到期只收提示、手动Pass、真实Begin、刷新保留intent、真实Submit现身支付4资产、原命令重试回执。每尺寸UI POST为`[2,1,0,0]`，即席0 Begin＋Submit、席1 Pass。重复点击是同一任务内两次真实DOM`click()`事件，独立核对原命令精确回执；没有将其称为物理双击。初始顶部截图未拍全下缘手牌，最终补了普通滚动后的可达截图和`elementFromPoint`命中证据，无需改动生产布局。

本批没有重跑`society-fixtures`特性配置或真实线上牌局；普通本地浏览器检查不等于已发布或线上试玩。历史恢复交接所述WASM重建身份差异，以及“10ABI74输入行为过不等于身份匹配”的限制，不在本批身份结论内；这里仅证明原字节冻结77–81及独立新82，不冒称解决那条历史身份问题。

## 失败、修正与未计入项

1. `18b1806`阶段前端专项首轮58/61，修正测试布局缺少对手及按钮定位后61/61；另一次重定向目录缺失未启动测试。原始材料保留在同目录WIP证据中。
2. 新HTTP专项首轮10个测试9过1败：到期场景测试读取opaque pacing时用了snake_case，实际持久JSON是`lastServerNowMs`。只改测试后该场景通过，后续干净六核完整轮已再次覆盖全部3场景。另一次shell工作目录错误在测试启动前退出。
3. production build首轮被新增前端测试误用Playwright的`exact`选项拦住，三处TS类型报错；修正测试选项后两个TS与完整构建通过，生产代码未改。
4. 浏览器第一轮在六核身份预检发现旧输出目录中一份未引用历史WASM，0浏览器／0请求；受影响的首轮完整Worker主动停止，不计完整通过。保存旧目录、改空输出后，最终完整Worker与浏览器均使用干净六核。
5. 干净六核完整Worker首轮自然结束267/279、12败，全在JZ46的全局catalog断言：它从已冻结为历史81的JC054 oracle取“当前身份”。源码点`398e220…`只改这一个测试导入为新82 oracle，历史79/78房间catalog和黄金输入未改。此失败发生在命令循环前，该轮不能称为全绿；最后完整sites套件已自然结束279/279，Native／前端／构建／浏览器原结果因生产源和产物完全未变而保留，不重复或混加。
6. 浏览器第二轮将纯展示的`data-card-actionable=true`误断言为false，在发送任何POST前失败。生产含义是可点选浏览／高亮，不是响应权限；修正测试后第三轮通过，第四轮补可达位置与手牌截图并完整通过。仍核对未Begin时只有pass及点击不提交。所有轮次日志保留，后续结果不与重叠专项相加。

## 精确材料与复核入口

- [最终验证](evidence/response-preview-2026-10-17/validation.json)、[原始日志索引](evidence/response-preview-2026-10-17/log-index.json)、[精确复现命令](evidence/response-preview-2026-10-17/reproduction.json)。
- [128项源码闭合](evidence/response-preview-2026-10-17/source-closure.json)、[169文件运行清单](evidence/response-preview-2026-10-17/runtime-manifest.json)。
- [已发布v63小时TLS复查](evidence/response-preview-2026-10-17/published-v63-tls-recheck/index.json)：官方只读身份与普通浏览器原始记录，和82本地结果分开。
- [原始浏览器结果](evidence/response-preview-2026-10-17/browser/report.json)、[全部原图SHA与4份随Git图片](evidence/response-preview-2026-10-17/browser/screenshots-index.json)。
- `rust-game-wasm/tests/responsePreviewNative82.fixture.json.gz`包含完整Native82原文；`node tools/build-storage.mjs -- node rust-game-wasm/tests/response-preview-compare.mjs`读取新82，`node tools/build-storage.mjs -- node rust-game-wasm/tests/jc054-compare.mjs`只读原冻结81。
- 新HTTP场景见`sites/test/response-preview-integration.test.mjs`，六核及整个catalog不变检查见`response-preview-kernel-routing.test.mjs`。原全部Native81场景也经过六核路由，不增加新字段。
- 原始大输入、编译输出和完整本地数据库位于Git外`/workspace/response-preview-evidence-20261017/`；Git只含源码、合成输入、必要日志／图片／身份材料，不含WASM、Worker、客户端构建、真实数据库、缓存或凭据。

下一步仅交父会话在此精确批次上串行独审。若审查通过，发布须另行指向已审定的精确来源；当前HEAD不会自动成为官方发布源。
