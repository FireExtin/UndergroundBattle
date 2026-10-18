# JC044 堆栈冻结来源反向身份校验 P2 修复

本次只处理父会话独审指出的恢复缺口；旧完整源码 `1979d2420bd1644c8d6b04a16ff2caa23608b9c5`（文档保存点 `4f5ba75c33142e0a9d34c54b8e1d983766b83fa2`）**未获放行**。独立分支仍为 `codex/jc044-time-repress-review-20261018`，其他独审仍在进行。当前为 WIP 验证保存点，HTTP/D1 最后验收待完成；没有发布或合 main。

输入是原 `jc044Native85.fixture.json.gz` 解压包装的 `raw.json` → `entry-deploy-skip.steps[0].expected.state`。revision=9，stack[0].id=i245，item.card=JC044/i244，frame.source.card=JC044/i243。强类型 Rust 只把 `/game/stack/0/frame/source/card/definition` 从 JC044 改成已准入 LC19，逐项断言其他字段不变；修前 Game/Room 以及原 WASM 均接受。四次真实 PassResponse 后，`resolve_stack` 仍依据 item.card 将 JC044 以正常新 UID i246 入场。现身入口同样成立。JavaScript 没有解析或重写不透明 Room 字符串。

生产修复仅在 `validate_jc044_state` 原 `if related(f)` 后补两行对称分支：容器仍夹带 JC044 而冻结程序属于其他牌时，返回“JC044不能挂接其他牌的堆栈程序”。沿用 JC057 既有模式，未修改支付、入场结算或其他规则。单测以派遣/现身×接受/跳过四种正常流程证明恢复及真实响应续行；新两份坏档直接来自原审查输入，旧 35 份坏档和原 13 场景的完整状态、日志、四席视图全部逐字节保留。133 编译输入仅改变 JC044 模块及其测试。

当前已完成：Native 适用 18/18；原始两份坏档 Game/Room 拒绝且合法续行逐项不变；新 WASM 的原始两档四席 view/apply/poll 共 12 入口全部拒绝；完整 Native/WASM 13 场景、118 命令、17 预期拒绝、524 视图、37 坏档一致；相关前端 45/45；TypeScript/Vite/Wrangler dry-run 通过。全量套件没有重跑，旧完整套件失败和修正边界继续见[历史证据矩阵](evidence/jc044-2026-10-18/README.md)。首个测试筛选误配为 0 项不计验收，改为完整模块名后原新回归 1 项按预期失败；探针私有方法/新 UID 假设错误的失败轮全部保留。

候选仍为 ABI85/pool77，149 牌/10 秘社。新 WASM `c02da9cbdfd99fcbfe3b4e0b82cc206ef8572edf2f58017ffd7a58bee2d0f9c1`（3263215B）；fixture `4ad888e66386e390662f19cac5582d16dd36047b2111b90ec89590a84f8ccd63`（998943B）；Native raw `f43a7e8a93b3ad77f29ce63d7fe861aa325248f8f4630a6798405327262df027`。旧候选 6ab520… 及 fixture41be2c… 已整份保留，ABI77–84 原核/见证不覆盖、不重签。原件与构建日志在 `/workspace/jc044-stack-source-review-20261018`，后续最终源码/运行身份及 HTTP/D1 回执将独立绑定入文档证据。

本小时已审 v66 的普通浏览器检查见[06:26 回执](SITES_V66_HOURLY_2026-10-18_0626.md)：证书阻断后停止，1 GET/0 写入/0 线上对局。组件、本地 HTTP 和本地浏览器均不计线上试玩。候选不发布；无新子任务、worktree、Actions 或 exchange 部署。
