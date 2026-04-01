# Review 记录（最近 3 个 Commit）

## 范围

- `61352b0` `feat(rules): 实现多个游戏机制V1版本`
- `b2a2ba2` `feat(rules): 统一 discard 语义并添加 discardCard DSL effect`
- `c5ee1fe` `feat(rules): 添加资产牌基础支持`

## 结论摘要

- `b2a2ba2` / `c5ee1fe`：总体方向正确。
- `61352b0`：发现 2 个实现级风险，均已修复并补回归测试。

## 发现与修复

### [P1] `MarkerRegistry` 未深拷贝，状态快照存在 map 别名污染风险

**问题描述**

- `BoardState` 新增 `Markers` 后，`cloneBoardState()` 没有克隆该字段，导致 `cloneGameState()` 之后的 marker 写操作可能回写到旧状态快照。

**修复**

- 在 `cloneBoardState()` 中接入 `cloneMarkerRegistry()`。
- 新增 `cloneNestedIntMap()`，对 `map[string]map[string]int` 做深拷贝。
- 文件：`server/pkg/rules/clone.go`

**新增测试**

- `TestMarkerCloneIsolation`
- 文件：`server/pkg/rules/marker_test.go`

---

### [P1] 附属 prune 按 `SourceCardID` 粗粒度删 effect，会误删同源仍有效效果

**问题描述**

- `AttachmentManager.PruneExpired()` 在清理过期附属后，曾按 `SourceCardID` 删除 continuous effects。
- 这会误删：
  - 同一 source 但绑定在其他仍有效 attachment 上的 effect
  - 同一 source 的非 attachment effect

**修复**

- 改为按“被移除的 `AttachmentID`”精准删除 effect。
- 仅删除 `effect.AttachmentID` 命中的 continuous effect，不再做 source 级别清洗。
- 文件：`server/pkg/rules/attachment.go`

**新增测试**

- `TestPruneExpiredAttachmentDoesNotRemoveOtherEffectsFromSameSource`
- 同时修正 `TestAttachmentSourceDepartureEffectInvalidation` 的测试建模，使其使用真实 attachment-bound effect（补 `AttachmentID`）。
- 文件：`server/pkg/rules/attachment_lifecycle_test.go`

## 本轮验证

已执行并通过：

1. `go test ./server/pkg/rules -run 'TestMarkerCloneIsolation|TestPruneExpiredAttachmentDoesNotRemoveOtherEffectsFromSameSource|TestAttachmentHostDepartureCleanup|TestAttachmentSourceDepartureEffectInvalidation|TestDiscardCardDSLEffectNoopForNonTableTarget|TestDiscardCardDSLEffectTriggersContinuousCleanupForTemplateEffects' -count=1`
2. `go test ./server/...`
3. `(cd tools/fixture-tools && npm test)`
4. `(cd web && npm test)`

## 备注

- 本次修复未扩展新机制范围，仅收敛了状态一致性与生命周期清理精度。
- 仍建议后续把 `marker` 与 `face-down/reveal` 从“测试辅助函数层”逐步接到正式 action/legality 管道，避免机制只停留在测试语义。
