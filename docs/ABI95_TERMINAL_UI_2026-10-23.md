# ABI95 终局提示最小修复

固定源码 `b7066bd9fe9fb3faa7ba9583eb1a7cbd57912103`，直接父提交 `5a5911d64da37d1978776ef5f5760da6d5086ffb`。原 ABI95 证据头 `51a336c4a42fd67495dca8d48dcfdd374e67f0ed`。

将线上 `845f0d8c050b4302e0e3473ccc8ea13156c41522` 已有的 AutoPass 终局优先提示和两条原测试原样移入。两文件补丁逐字节一致，产品改动仅 AutoPass.tsx 的提示优先级；连续让过动作条件、计时器、服务端和卡牌机制均无变化。使用当前权威终局状态覆盖旧的“请先完成选择”，组件重挂载后保持一致。

本任务的普通独立克隆位于 `/home/ddd/Documents/Codex/2026-10-23/task-6/UndergroundBattle-abi95-terminal-ui`，无 worktree/alternates。原作者最新归属已声明只读检查，本任务独占此补丁；WM037/MSJC13、WM044/MSJC14 仍归原任务。当前项目台账35/50属于后续原生进度，不计入本固定ABI95的33/50台账。

作者验证：固定ABI95加原两条回归测试先出现2失败；修后连续让过、恢复、自动让过及终局摘要4文件44项通过。验证本人/他人选择后终局即时提示、按钮禁用、不发送动作、活动回调关闭和组件重挂载一致。两份 TypeScript 检查与生产构建通过；单核CPU31、nice19、单worker、禁止文件并行，离线复制依赖到本克隆。

构建来自干净源码提交。267项旧冻结工件已逐一校验；新清单仍267项，仅替换 index.html 与带hash的JS，其余265项保持一致，其中260项为客户端文件，包括CSS、原始卡图和元数据。WASM/server/hosting使用已有冻结字节，本轮未重新构建或重跑Native/WASM/Worker；已通过的原ABI95平台独审继续保留原范围。新的两文件客户端delta和完整目标清单供原部署任务组合，未执行部署。

本轮是作者验证，不是新候选独审或真实浏览器验收。“刷新”证据为组件卸载再挂载，未声称浏览器刷新或选择到胜利瞬间的浏览器验收。独审只需复核这次最小UI集成及构建身份；真实浏览器验收由原部署流程继续。未混入ABI96/97、其他新卡、损坏存档扩展、公开准入调整或整分支merge。

证据见 `evidence/abi95-terminal-ui-20261023/verification.json`、`input-audit.json`、`artifact-composition.json`、`deployment-artifacts.json`、原样补丁和gzip运行回执/日志。源码及测试、必要证据文档入库；依赖、构建缓存、凭据与数据库不入bundle。没有push、main修改、CI或上线操作。
