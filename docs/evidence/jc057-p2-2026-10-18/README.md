# JC057 P2：修前实际复现与修后精确见证（待父串行独审）

修复代码保存点`5a1a40dee9506b073f3ddd6dc4c796c3f7c72647`已正常推送至`codex/stage-pass-progress-20261016`。审查增量2261d773→5a1a40d，唯一生产源码是jc057.rs；后续仅补证据。runtime-manifest.json绑定该代码与177构建输出/174运行文件，133个源码/测试脚本及新fixture与提交内容吻合。

复现基线是2261d773，旧ABI84 WASM191c05fc、fixture0832c399。原revision13、choiceId i249、actor0检查点仅把pending.seat改1，就允许席位1代为跳过；再改playerId也允许其代选p1。修后同一输入字符串在恢复时被拒绝，原行动者跳过/指定目标仍接受至14，其他席位仍拒绝且保持13。原始输出见repro-before.json.gz、repro-after.json.gz，原输入和哈希见repro-inputs.json.gz；范围仅是持久化坏档，不是正常HTTP可上传任意状态的断言。

修后生产改动仅jc057.rs的Declare待选守卫。131编译输入中，仅jc057.rs及既有jc057_tests.rs变化。原10场景完整命令、状态、日志、四视图完全相同，原检查点逐字节一致，见native-legitimate-delta.json。新增声望、威名和通过JC036/XQ14/BQ035真实授予的机动3场景，保留规范Accept/Target流程。

| 验证 | 实际结果及范围 |
| --- | --- |
| Native首次编译 | 新测试中的E0502借用冲突，退出101，无测试执行；原日志保留。 |
| Native修正后 | 定向14/14通过，0失败/忽略，工作区其余测试被筛选；不称完整Native重跑。 |
| 机动兼容 | 既有全准入人类/吸血鬼真实授予机动测试1/1通过，所有实例沿原校验器继续。 |
| 新ABI84/WASM | 131输入，13场景124命令20次预期拒绝548视图逐项一致；24坏档的四席view、choose和poll均与Native恢复错误一致。 |
| 同输入修后复现 | 两种修前可代选的坏档都在恢复拒绝，合法原检查点仍可继续，见repro-after。 |
| 后端构建 | release WASM、production后端dry-run及固定时钟测试包装全部自然0，未构建前端或发布。 |
| 实际HTTP/D1及路由 | 定向4文件53/53通过，自然退出0，失败/取消/跳过/待办均0，耗时253033.731279ms。13合法场景逐一比较完整状态/日志/四视图，并在每条接受命令后D1重开，原收据重试一致；24坏档在D1重开前后各四席GET及一次choose共240个HTTP400，房间/收据/日志零改写。另16项覆盖77–83精确路由、原83见证39命令180视图、固定身份及8WASM产物清单。见affected-result.json和原始日志。 |

新WASM：3217477B，SHA256 `31d832d9f9873381f9cb6f4714fd800feef53c9993dd132f32eccdd72ca47b31`。新fixture：1051514B，SHA256 `bf6ab32c67db7017b202544ccf5e0d3d5b30b9fdfae64583b407ed436b36e77b`。Native raw：14103141B，SHA256 `f9e51f9e56f62b23b9feff1aedd32626150da425a707ee4b7fbf4367f0c52069`。全部131源哈希见build-identity.json；仅更新本批未获放行的84见证，没有重签任何冻结77–83输入或产物。

既有前端1298、布局50及7场本地浏览器证据按父复审要求复用：165客户端构建文件逐字节不变，web源码未变。既有完整Native1213和修前完整Worker315也仅为其各自保存点的历史证据，不声称本修复重跑了完整套件。此前299/315失败轮保留，不相加。

所有构建/测试经过既有build-storage磁盘/内存门禁，未清缓存/数据库。包内177构建输出含174运行文件（165客户端、8WASM、server/index.js）和3辅助文件（hosting.json、sourceMap、README）；测试包装仅固定本地服务时钟，生产仍真实时钟。旧pkg/generated/server整体保存在`/workspace/jc057-p2-20261018/pre-fix-outputs`，DB留本地，不入Git。原codex-index-refresh.lock身份保持。

定向Worker运行前后核验133个输入/测试脚本哈希、全部177构建输出和固定时钟Worker的9个运行文件均未变；原83 fixture仍ee2bc666。新production Worker为175795B、SHA256 `6a92fb1b6794bc94d31c976bc2e7e77a9b172504516c07b877b89ac81ead9b46`。review-delta.json列出相对2261唯一5个程序/测试/见证路径，以及新当前WASM换名和Worker/sourceMap/README产物差异；runtime-identities.json列出七个旧核JS/WASM身份。

FAQ原PDF实物高分辨率确认“监控员（基础050）”，此前060为误读，见faq-source.json和faq-monitor-header.png。原PDF与JC057的57/135卡图未改。

输出为本地候选，2261未放行，新修复也待父会话按精确SHA串行独立复审；没有访问网站、合main、部署或触发Actions。公网仍以既有已审v65/ABI83身份为准，本地组件/浏览器结果不算线上试玩。
