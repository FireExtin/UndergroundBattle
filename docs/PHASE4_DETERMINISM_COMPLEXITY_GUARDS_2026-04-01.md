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
