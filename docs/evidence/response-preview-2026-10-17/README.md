# 连锁前只读来源预览：ABI82独立审查证据

最终源码审查点 `398e220721f9e14c4f523530183c6288b7749096`。Rust、前端生产代码和169个运行文件与运行验证时的 `9e549277e98396d7e7ade38a8fa3825d37fa7950` 完全相同；后续仅修正JZ46测试对全局当前catalog的身份导入。发布仍是v63，本批未发布、未合main、未启动Actions。范围、失败和限制见 [交接](../../RESPONSE_PREVIEW_82_REVIEW_HANDOFF_2026-10-17.md)。

- `validation.json`：最终完整轮的实际计数与边界；历史专项和完整轮重叠，不累加。
- `source-closure.json`：128项Native编译输入逐一匹配源码审查点；新Native原文保存在合成黄金输入 `rust-game-wasm/tests/responsePreviewNative82.fixture.json.gz` 内，未用Node改写opaque字符串。
- `runtime-manifest.json`：162客户端＋7服务端运行文件；6份指定WASM，原77–81字节不变。编译产物在Git外。
- `reproduction.json`：本次实际运行命令、固定时钟wrapper与工具链。旧ABI77 pass-history专用依赖另见 `historical-pass-history-wrapper.json`。
- `browser/report.json`：普通Chrome151，两尺寸各四席，实际production GameApp＋监听HTTP/workerd/WASM＋合成本地D1；0页面错误。10次位置命中与预览／刷新0POST、真实Pass／Begin／Submit、支付和原回执均有原始记录。重复点击是同步两次DOM click，不冒称物理双击。
- `browser/screenshots-index.json`：原10图的完整SHA与字节数；Git包含两个尺寸各一张连锁前手牌可达图和一张真实composing图，共4张。
- `published-v63-tls-recheck/`：11:17:01安全边界小时复查，官方v63身份后普通TLS访问，仍证书错误，1GET尝试／0POST／未到UI；与82本地结果分开。

完整Native1183与前端1283分别自然结束；新82 Native/WASM为3场／8命令／2轮询／12纯view／100投影／2报价，逐完整结果相等。旧81原黄金文件SHA仍为13c5728f…787c82，8场／214命令／888投影／44坏档／528恢复调用仍只对照原发布81。`wip-status.json`及早期round1/2日志是历史检查点，不代表最终状态。

失败与停止均保留：首次HTTP测试持久时钟字段名错误；首次TS编译中新增测试误用exact；浏览器首轮旧测试目录残留未引用WASM导致预检停止，第二轮误断言展示属性；旧输出完整保留后改干净六核。受影响的首次Worker运行主动停止；干净完整首轮267/279、12败全部为JZ46全局身份期望仍取历史81，修正一个测试导入后重跑完整套件。详见带`before-test-identity-fix`的原始结果及`worker-round1-stop`记录，不把这些轮次统称全绿。

所有gzip无损保存原始日志／JSON，`log-index.json`列原文与gzip身份。没有把数据库、WASM、Worker、客户端构建、缓存或凭据写入Git；opaque样例与席位信息均为显式本地合成输入。本批未重跑society-fixtures配置，未进行远端D1验收或线上对局，也不解决历史ABI74重建身份差异。
