# JC066 探针发现隔离证据

基线 `a8bf844c30cf988abb25af73e15951849b5fc93a`；产品源码仍为 `fa94ec406a9e8163ed9b42756181bd629d3a129f`。本轮只移动专用探针并修正相对辅助模块导入、更新说明；仍待父会话其余独审，不发布。

- `discovery.json`：默认 package 脚本、55 文件展开列表、探针被排除且默认文件不引用专用 case-file 变量。
- `runs/default-discovery-smoke.*`：四项专用变量显式 unset，原 glob 全部模块加载，名称过滤只执行一项既有身份断言。55 个 Node 文件/断言结果通过不代表 55 文件完整回归。
- `runs/probe-before.*`、`runs/probe-after.*`：移动后的显式探针各 4/4。旧实物接受漏洞，新实物拒绝且重开前后零额外写入。使用相同坏状态字符串，旧版通过仅表示成功复现漏洞。
- `isolation.json`、`verify-isolation.py`：核对 135 编译输入和现有 WASM/夹具、181 生产文件、20 生成文件、13 测试 Worker 文件未变；无重建。
- `validation-summary.json`：本轮三次自然命令的准确范围与限制。`manifest.json`：本目录除 manifest 本身之外的字节数/SHA256。

当前显式命令见 [专用探针说明](../../../sites/probes/README.md)。旧产品证据回执不改写；本轮原件在 `/workspace/jc066-p2-probe-evidence-20261018`，旧 Native/WASM/Worker 原件沿用 `/workspace/jc066-p2-evidence-20261018`。Git 无数据库、WASM 二进制、缓存、凭据或真实私人数据。本轮不重复网站检查、无线上试玩。
