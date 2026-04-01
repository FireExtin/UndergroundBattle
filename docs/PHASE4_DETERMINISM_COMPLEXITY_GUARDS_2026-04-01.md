# PHASE4: Determinism + Complexity Guards (2026-04-01)

## Purpose

Add two cross-cutting safety rails in the Go rules engine:

1. 强制可回放确定性（commit-time replay determinism guard）
2. 复杂度预算机制（core file line-budget gate）

These are guardrails, not new gameplay features.

## 1) Commit-Time Replay Determinism Guard

### Behavior

- Public submit paths now enforce determinism:
  - `SubmitAction`
  - `SubmitActionWithProjection`
- After one action commits, engine replays the same action from the same pre-state (internal no-projection path), then compares full `GameState`.
- If replay state differs, action is rejected as:
  - `ReasonCode`: `RULES_FAILED_INVARIANT_VIOLATED`
  - `MessageKey`: `rules.replay.non_deterministic`
  - `Hook`: `replay.determinism`

### Scope

- Replay API path (`ReplayActions`) uses an internal submit path that **does not** recursively re-run determinism guard (to avoid recursion and doubled replay nesting).
- Invariant checks still run as before.

### Tests

- `TestSubmitActionInternalDeterminismGuardPassesForStableReplay`
- `TestSubmitActionInternalRejectsDeterminismMismatch`

## 2) Core Rules Complexity Budget Gate

Added `TestCoreRulesComplexityBudgets` to block uncontrolled growth in hotspot files:

- `engine.go <= 1250`
- `types.go <= 730`
- `continuous.go <= 450`
- `attachment.go <= 280`
- `projection.go <= 340`

The test fails once any file exceeds its line budget.

## Budget Update Rule

If a budget needs to be raised:

1. Prefer extracting module boundaries first.
2. If still necessary, raise the budget in test with a short rationale in the same commit.
3. Update this doc to keep rationale visible for handover/review.

## 3) Orchestrator-Lite Extraction (Submit/Replay)

To continue complexity reduction after adding guards, the submit/replay entry pipeline was extracted from `engine.go` into `submit_pipeline.go` as explicit phases:

- legality
- build + execute
- commit
- invariant guard
- determinism guard

`engine.go` now stays focused on rules primitives (legality/build/execute helpers), while orchestration flow lives in one dedicated file.

Practical impact:

- lower file pressure on `engine.go` (dropped from ~1240 lines to ~1110 lines)
- clearer insertion point for future cross-cutting guards (no more ad-hoc growth in engine core)
- no behavior change; all existing tests pass

## 4) Unified State Transition APIs

Added centralized transition helpers in `state_transitions.go`:

- `moveCardToDiscard`
- `revealFaceDown`
- `attachToHost`
- `setMarker`
- `addMarkerCount` / `removeMarkerCount`

And refactored core execution paths to use them (instead of scattered field writes):

- reveal path (`executeRevealCard`)
- role action exhaust/damage/influence writes
- DSL exhaust/damage/influence/discard writes
- attachment creation route inside continuous registration
- marker action execution route (`set_marker` / `remove_marker`)

Marker actions are now integrated into authoritative action pipeline end-to-end:

- legality checks
- operation build
- operation execution
- structured events (`marker_set`, `marker_removed`)
- replay/history/projection through normal commit flow

## 5) Rule Declaration vs Execution Registry Split

`DSL/fixture` remains declarative (`kind`, `targetRef`, params).  
Execution dispatch is now registry-driven in Go:

- `dsl_effect_handlers` map in `dsl_handlers.go`
- `applyDSLEffect` resolves handler by effect kind and executes implementation

This removes the execution switch from DSL entry and keeps control flow in Go handlers only.

## 6) Fine-Grained Effect Binding IDs

Added `ContinuousEffect.BindingEntityID` and binding-aware lifecycle checks:

- `attachment:<id>`
- `card:<cardId>`
- `operation:<operationId>`

Pruning now evaluates binding identity first (with backward-compatible fallback), preventing coarse source-level over-pruning.

Extra extraction:

- moved binding lifecycle helpers into `continuous_binding.go` to keep `continuous.go` within complexity budget.
