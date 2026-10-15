# 隐秘世界单线程整合审查（WIP）

输入均在本次 `git ls-remote --heads origin` 后显式 fetch 并核对：

| 输入 | 精确提交 |
| --- | --- |
| 当前 main / 共同基线 | `3d642f0c1e6e5ac021cd40560086df35638bfe4c` |
| JC060 / ABI75 | `e53e8b017f0ec974c6cf3d8f446b6541771cf4a5` |
| JC095 / ABI76（含 JC060） | `dbdee5d6014c260f9833c162bf85aff4b491b690` |
| 恢复完整六提交 | `799717557a3536ebae75c472e1e5c4a58ac12fc9` |
| UI 完整八提交 | `9f3556d0983d5887b20ba61f3ae571bcd00defe2` |
| JC046 / ABI77 待接续 WIP | `a56827586d58666812bf48f6a3e0a22aecdf9895` |

输出分支：`codex/hidden-world-serial-integration-review-20261015`。使用现有普通克隆，不建 worktree，不开启并行开发子任务。仓库及祖先目录、环境 `.agents` / `.codex` 未找到可读 `AGENTS.md` / `SKILL.md`；遵循 `docs/TASK_DOCUMENTATION_POLICY.md`。本说明及后续证据记录实际组合结果，不能继承原分支测试数字作为组合通过证明。

## ABI76 整合

从 JC095 精确 SHA 新建审查分支，先以 merge 保留恢复完整历史，再 merge UI 完整历史。

- 恢复合入时仅 `CurrentLifecycleChoices.test.jsx` 自动三方融合；保留 ABI76 fixture 引用及真实 WASM SHA 检查，加入草稿刷新断言。
- UI 合入发生两个真实冲突：`ChoicePanel.tsx` 和 `ChoiceRecovery.test.tsx`。采用已包含恢复 `f12f47d` 草稿基底和其后交互改进的 UI 最终版本；未覆盖恢复独有的 API、`useGame`、入桌回执、保存失败重试及同席标签提示。
- `choiceDraftStorage.ts`、其测试、`Table.tsx` 两路内容相同；生命周期测试两路改动相同并已保留当前 ABI 版本。UI 新增 `currentDomains` 阅读，保留领域隐藏边界和印刷值。
- 未修改任何历史 opaque 状态版本或旧 fixture 内容。

此检查点尚未完成组合回归或生产构建，保持 WIP。依赖与构建缓存放在 Git 忽略目录或仓库外；不提交凭据、数据库或缓存。

## 接续验证准备

本机锁定工具链为 Rust 1.90.0、wasm32-unknown-unknown、wasm-bindgen-cli 0.2.104，入口 `/workspace/tooling/env.sh`；构建统一经过仓库现有存储/资源守卫，缓存只用一个磁盘 target。

新增 `web/src/game/JC046Integration.test.jsx`，用真实当前 Native37 场景中的强制选择验证四席处理、来源牺牲后继续、领袖暂停续体、两个独立现身帧；测试精确命令、完整返回与各席视图，并覆盖整合后的单选换选、草稿重挂载、不可跳过和自动让过不代选。新增 `tools/frontend/serial-integration-qa.mjs` 用现有 Vite/Playwright 对当前77投影做桌面/手机本地显示检查；它阻止 API 请求并明确不计作线上试玩。两项在此准备提交尚未运行，仍是 WIP。

已完成的本轮检查：来源42项、输入生成守卫22项；JC046静态复审14份源码指纹全部匹配。完整 Native workspace 回归正在执行，不跳过原长对局。新证据根 `/workspace/serial-evidence-20261015`；不沿用被中止日志作为通过证明。

## JC046 接续与发布边界

按 `docs/EARLY_STOP_HANDOFF_2026-10-15.md`、`docs/evidence/jc046-2026-10-15/README.md` 的原文恢复。JC046 保存点缺少真实 ABI77 当前输入及 WASM；原先被 SIGTERM 中止的 Native 回归不能称为全量通过。先在新目录生成原始证据，再完成 Native/WASM/当前输入/前端/Worker 验证。

JC095 历史 Native1139、Web971 是修正后的跨轮唯一通过数，不能表述为 ABI76 单次全绿。恢复历史124与后续19/32不相加；历史 ABI74 重建 hash 差异仍未解决，行为一致不等于身份相同。

此轮没有独立复审、发布、main 合并或线上自然试玩。交接所列公开站仍为 v53；后续只有成功部署且身份一致的版本可计作线上试玩。中立双色继续暂停。需要独立复审时将精确批次材料交父会话串行安排。
