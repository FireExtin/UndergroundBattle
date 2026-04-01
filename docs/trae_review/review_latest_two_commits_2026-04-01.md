# Review 记录（最近 2 个 Commit）

## 范围

- `b2a2ba2` `feat(rules): 统一 discard 语义并添加 discardCard DSL effect`
- `c5ee1fe` `feat(rules): 添加资产牌基础支持`

## 结论摘要

- `c5ee1fe`：未发现会导致当前行为回归的实现级 bug。
- `b2a2ba2`：发现 1 个实质语义问题，已修复并补回归测试。

## 发现与修复

### [P1] `discardCard` 未请求 continuous 重算，可能导致临时脏状态

**问题描述**

- `discardCard` 会直接把目标送入 `discard`，但之前没有调用 `requestContinuousRecalculation`。
- 结果：若被弃置卡是 continuous source（例如 `XQ31`），同一次提交后目标卡的 `EffectiveStats` 可能短暂保留旧 buff，直到下一次触发重算。

**修复**

- 在 `applyDiscardCardEffect()` 中，目标成功进入 `discard` 后显式请求 continuous 重算。
- 文件：`server/pkg/rules/dsl.go`

### [P2] `discardCard` 对非 `table` 目标也生效，语义越界

**问题描述**

- 之前实现允许把任何 zone 的目标直接变为 `discard + revealed`。
- 这会把 `discardCard` 从“在场离场”扩大到“任意区直接转移”，并引入隐藏信息边界风险（不必要 reveal）。

**修复**

- 将 V1.5 行为收紧：`discardCard` 仅对 `Zone == table` 且未销毁目标生效；否则 no-op。
- 文件：`server/pkg/rules/dsl.go`

## 新增/更新测试

- `server/pkg/rules/discard_test.go`
  - `TestDiscardCardDSLEffectExists`
    - 增加断言：`discardCard` 会设置 `PendingRecalculation`
  - `TestDiscardCardDSLEffectNoopForNonTableTarget`（新增）
    - 验证非 `table` 目标 no-op，且不触发重算
  - `TestDiscardCardDSLEffectTriggersContinuousCleanupForTemplateEffects`（新增）
    - 验证丢弃 `XQ31` source 后，重算能移除对盟友的防御 buff

## 验证结果

已执行并通过：

1. `go test ./server/pkg/rules -run 'TestDiscardCardDSLEffect|TestLethalDamageUsesMoveCardToDiscard|TestDiscardCardDSLEffectNoopForNonTableTarget|TestDiscardCardDSLEffectTriggersContinuousCleanupForTemplateEffects' -count=1`
2. `go test ./server/...`
3. `(cd tools/fixture-tools && npm test)`
4. `(cd web && npm test)`

## 备注

- 本次修复保持了 `Discard/Graveyard V1.5` 的“最小可扩展”边界，没有扩展到复活/回手/坟墓资源系统。
