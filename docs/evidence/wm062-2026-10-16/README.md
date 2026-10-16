# WM062 本地实施证据（WIP）

当前候选为80／pool73／hegemony-pdf-v1，WASM `0229f75b7af79035f2a4025fc11c6de6f3f0b895127cc8b537f829268d030472`，3047680字节。Native专项第四轮12/12，14份连续场景、111命令、490席投影、18次预期命令拒绝及45固定坏档540次恢复与该WASM逐opaque状态、回执、投影及错误相等。`native-source-binding.json` 来自编译内嵌的125项输入，压缩原始输入在 `rust-game-wasm/tests/wm062Native80.fixture.json.gz`。

`defense-before-after*` 保留同一合法初始输入在修前ef68 WASM与修后Native／0229 WASM中的实际差异：5条真实Room命令后，旧核在移动待选时留下伤害1防御1的JZ48，新核已清理，所得暗藏者UID相同。没有改写任何版本头。ef68、原Native13场景及95b32536的完整125输入绑定均在Git外保留；ef68不是当前制品。

完整workspace、最终前端、Worker及最终本地真实浏览器继续执行，不能据本检查点放行发布或冒称线上试玩。原始日志、各轮失败和大记录均在 `/workspace/wm062-evidence-20261016/`。源码、测试和证据只含本地合成场景；无真实桌、数据库、凭据、缓存或编译运行文件入Git。最终范围会在完整批次交接中补齐。
