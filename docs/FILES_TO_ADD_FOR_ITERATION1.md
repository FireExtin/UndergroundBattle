# Files I need you to add to the chat before I start editing

Please add the following new files (or existing test/trace files) to the chat so I can implement TDD-first changes for Iteration 1. I will not edit other repository files until these are present in the chat.

1) New source (minimal, focused implementation)
- server/pkg/rules/turn_conflict_resolution.go
  - Purpose: small new file to host conflict/tie resolution helpers and the minimal entrypoints that engine.go can call.
  - Will contain: tie resolution, first-player-privilege application helper, and minimal public functions used by tests.

2) New tests (failing tests first)
- server/pkg/rules/conflict_test.go
  - Contains TestConflictTieUsesFirstPlayerPrivilege (failing initially).
- server/pkg/rules/damage_test.go
  - Contains TestDamageAssignmentAndLethalResolution.
- server/pkg/rules/priority_test.go
  - Contains TestPriorityRestoredAfterStackResolution.
- server/pkg/rules/pending_prompt_test.go
  - Contains TestPendingPromptProjectionPrivacy.

3) Trace / golden artifacts (for QA / regression)
- runtime/match-traces/iteration1_tie_privilege.json
- runtime/match-traces/iteration1_damage_lethal.json
- runtime/match-traces/iteration1_priority_restore.json

Why I need each:
- The new source file keeps the engine.go changes minimal and isolated (TDD principle: smallest change).
- Tests drive the exact behavior required by the README / NEXT_GEN_RULE_PLAN: tie handling, damage/lethal resolution, and priority restoration after stack resolution.
- Traces are the golden artifacts for QA to replay and assert outcomes.

What I'll do after you add them:
1. Add failing tests (the files above) or adopt the ones you add if they already contain test stubs.
2. Implement the minimal functions in server/pkg/rules/turn_conflict_resolution.go and small, safe edits to engine.go (only if necessary) to make tests pass.
3. Run tests, iterate until green.
4. Produce the three trace files under runtime/match-traces/ and mark Iteration 1 progress -> update docs/NEXT_GEN_RULE_PLAN.md to "Partially Done" / "Done" accordingly.
5. Commit proposed changes as SEARCH/REPLACE blocks (you will apply them).

Minimal commands you can run locally to verify once I produce edits:
```bash
go test ./server/...
```
```bash
# (optional) run all rules package tests more specifically
go test ./server/pkg/rules -run TestConflictTieUsesFirstPlayerPrivilege
```

If you prefer, add the test files empty (with TODO stubs) and I will replace them with full failing tests in my next edit. Once you add the listed files to the chat, reply "added" and I'll produce the SEARCH/REPLACE edits to implement the tests and minimal code changes.
