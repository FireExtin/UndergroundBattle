# Requested files to add to the chat before I make edits

I will follow the TDD-first, minimal-implementation workflow described in README.md, AGENTS.md, docs/ARCHITECTURE_PRINCIPLES.md and docs/NEXT_GEN_RULE_PLAN.md.
You already added:
- server/pkg/rules/engine.go
- server/pkg/rules/projection.go
- server/pkg/rules/turn_conflict_types.go

To proceed with Iteration 1 (PN-ACT-005 / PN-ACT-006 / PN-ACT-009) I will need the following additional files to be added to the chat so I can create failing tests and then implement minimal fixes:

1) New implementation files (minimal, focused):
- server/pkg/rules/conflict_state_machine.go
  - Purpose: implement conflict-phase state transitions, tie handling, and first-player privilege transfer points.

- server/pkg/rules/combat_resolution.go
  - Purpose: minimal intercept selection, damage assignment, lethal check, and removal pipeline used by the conflict flow.

2) New test files (TDD first — I will add tests that initially fail, then implement code to pass them):
- server/pkg/rules/conflict_tie_test.go
- server/pkg/rules/conflict_privilege_test.go
- server/pkg/rules/combat_resolution_test.go
- server/pkg/rules/damage_lethal_test.go
- server/pkg/rules/trace_playback_test.go

3) Golden traces (fixtures used by trace_playback_test.go):
- runtime/match-traces/iter1-tie-intercept.log
- runtime/match-traces/iter1-combat.log

Notes and rationale:
- I will not edit any other files unless you explicitly add them to the chat or grant permission.
- After you add the files above I will:
  1. Create failing unit tests (in the listed test files).
  2. Implement minimal code in conflict_state_machine.go and combat_resolution.go to make tests pass.
  3. Run local verification and iterate until green.
  4. Update docs/NEXT_GEN_RULE_PLAN.md to mark PN-ACT-005 (and related tasks) as completed when validated.
- If you prefer an even smaller initial scope, tell me which of the above to postpone (e.g., skip golden traces for now) and I'll only request the essential files.

Suggested single command to run after I push changes locally (optional):
- go test ./server/...

Please add the listed files to the chat so I can produce SEARCH/REPLACE patches (tests first) against them.
