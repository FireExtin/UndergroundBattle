# RULES_ROADMAP_CONTEXT_2026-03-31

Purpose: preserves the current rules-core roadmap as context for subsequent web debugger and rules-engine tasks.

## Current Baseline

- 已有最小规则核、priority、stack、projection、continuous effect。
- 已有最小 permission hook：`inspect_card`。
- 已有最小数值语义：`dealDamage` 会结合 `EffectiveStats.Defense` 做致命判定。

## Planned Next Rules Milestones

### 1. Permission / Legality V2

- 把 permission hook 从 `inspect_card` 扩到 `reveal_card`、`queue_operation` 和后续角色动作入口。
- 固化 `RequiredPermissions`、`Prohibitions`、`ActionQuota`。
- 拒绝码统一为：
  - `LEGALITY_FAILED_PERMISSION_REQUIRED`
  - `LEGALITY_FAILED_ACTION_PROHIBITED`
  - `LEGALITY_FAILED_ACTION_QUOTA_EXCEEDED`

### 2. Character Stats Semantics V1

- 新增最小角色动作：`declare_attack`、`declare_investigation`。
- `combat` 用于攻击伤害值，`defense` 用于致命阈值，`investigation` 用于放置影响力。
- `modifyStat` 必须真实影响这些动作读取的数值。

### 3. Continuous Effect Lifecycle V1.1

- source card 离场后，其持续效果在下一次 commit 重算中移除。
- 继续保持每次 commit 最多一次完整重算。
- 暂不实现完整 dependency graph。

### 4. DSL Fixture Gate 扩展

- 新增命中新语义的 fixture：授予检视、禁止发动、本回合数值修正、攻击伤害、调查影响力。
- TS 做 schema/fixture gate，Go 做 contract test 和 rules test。

## Why This Context Exists

- Web 调试器必须围绕“Go 为唯一语义权威”来展示状态。
- 当前前端工作只消费 mock 或未来真实 protocol messages，不直接裁定规则。
- 后续所有 Web 调试和规则扩展，都以这份 roadmap 作为当前阶段背景。
