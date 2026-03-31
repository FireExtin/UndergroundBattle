# Legality Framework Hardening V1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stabilize the current `XQ22` / `XQ31` legality surface by extracting a production rule catalog and shared legality matchers without changing the current gameplay semantics.

**Architecture:** Keep [`engine.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/engine.go) as the orchestration layer only. Move production rule declarations into a tiny catalog file, move duplicated source-condition and actor-scope matching into shared helpers, and keep [`prohibition.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/prohibition.go) and [`target_legality.go`](/Users/ddd/Downloads/UndergroundBattle/server/pkg/rules/target_legality.go) as thin evaluators over those shared pieces. This plan does not implement `XQ01`, full attachment permanents, or new UI work.

**Tech Stack:** Go, `testing`, current rules kernel under `server/pkg/rules`, docs in `docs/`

---

## Scope Guardrails

- In scope:
  - production-only legality rule catalog for `XQ22` and `XQ31`
  - shared matcher helpers for `CardCondition` and actor scope
  - regression coverage for `queue_operation` vs role-action boundaries
  - doc sync for the new structure
- Out of scope:
  - `XQ01` region / ability-kind semantics
  - full `XQ31` numeric aura (`+1 defense`) implementation
  - attachment / permanent lifecycle V1
  - web debugger / transport / websocket / persistence work

## File Map

- Create: `server/pkg/rules/legality_catalog.go`
- Create: `server/pkg/rules/legality_catalog_test.go`
- Create: `server/pkg/rules/legality_shared.go`
- Create: `server/pkg/rules/legality_shared_test.go`
- Modify: `server/pkg/rules/prohibition.go`
- Modify: `server/pkg/rules/target_legality.go`
- Modify: `server/pkg/rules/prohibition_test.go`
- Modify: `server/pkg/rules/role_actions_test.go`
- Modify: `docs/HANDOVER_TRAE_2026-04-01.md`
- Modify: `docs/NEXT_GEN_RULE_PLAN.md`

## Success Criteria

- `engine.go` no longer contains card-definition-specific legality matching logic.
- Production rule registration for `XQ22` and `XQ31` lives in one obvious file.
- `prohibition.go` and `target_legality.go` do not each keep their own copy of source-condition and actor-scope matching logic.
- `declare_attack` and `declare_investigation` remain unaffected by `XQ31`.
- `queue_operation` continues to respect `XQ22` and `XQ31`.
- `go test ./server/...`, `cd tools/fixture-tools && npm test`, and `cd web && npm test` all stay green.

### Task 1: Add a Production Rule Catalog API

**Files:**
- Create: `server/pkg/rules/legality_catalog_test.go`
- Create: `server/pkg/rules/legality_catalog.go`
- Test: `server/pkg/rules/legality_catalog_test.go`

- [ ] **Step 1: Write the failing catalog tests**

```go
package rules

import "testing"

func TestBuildProductionProhibitionRules(t *testing.T) {
	rules := BuildProductionProhibitionRules()
	if len(rules) != 1 {
		t.Fatalf("production prohibition rule count = %d, want 1", len(rules))
	}

	rule := rules[0]
	if rule.SourceDefinitionID != "XQ22" {
		t.Fatalf("prohibition sourceDefinitionId = %q, want XQ22", rule.SourceDefinitionID)
	}
	if len(rule.TargetCategory.BasicTypes) != 1 || rule.TargetCategory.BasicTypes[0] != "事务" {
		t.Fatalf("prohibition basicTypes = %#v, want [事务]", rule.TargetCategory.BasicTypes)
	}
}

func TestBuildProductionTargetLegalityRules(t *testing.T) {
	rules := BuildProductionTargetLegalityRules()
	if len(rules) != 1 {
		t.Fatalf("production target-legality rule count = %d, want 1", len(rules))
	}

	rule := rules[0]
	if rule.SourceDefinitionID != "XQ31" {
		t.Fatalf("target legality sourceDefinitionId = %q, want XQ31", rule.SourceDefinitionID)
	}
	if len(rule.AffectedTargetCondition.Keywords) != 1 || rule.AffectedTargetCondition.Keywords[0] != "声望" {
		t.Fatalf("target legality keywords = %#v, want [声望]", rule.AffectedTargetCondition.Keywords)
	}
	if rule.AffectedTargetCondition.Side != SideAlly {
		t.Fatalf("target legality side = %q, want %q", rule.AffectedTargetCondition.Side, SideAlly)
	}
}
```

- [ ] **Step 2: Run the tests to confirm they fail**

Run: `go test ./server/pkg/rules -run 'TestBuildProduction(Prohibition|TargetLegality)Rules' -count=1`
Expected: FAIL with `undefined: BuildProductionProhibitionRules` and `undefined: BuildProductionTargetLegalityRules`

- [ ] **Step 3: Implement the catalog file**

```go
package rules

func BuildProductionProhibitionRules() []ProhibitionRule {
	return []ProhibitionRule{
		XQ22ProhibitionRule,
	}
}

func BuildProductionTargetLegalityRules() []TargetLegalityRule {
	return []TargetLegalityRule{
		XQ31TargetLegalityRule,
	}
}
```

- [ ] **Step 4: Wire the current builders to the catalog**

```go
func BuildProhibitionChecker(state GameState) *ScopedProhibitionChecker {
	return NewScopedProhibitionChecker(BuildProductionProhibitionRules())
}

func BuildTargetLegalityChecker(state GameState) *TargetLegalityChecker {
	return NewTargetLegalityChecker(BuildProductionTargetLegalityRules())
}
```

- [ ] **Step 5: Run the targeted tests to verify they pass**

Run: `go test ./server/pkg/rules -run 'TestBuildProduction(Prohibition|TargetLegality)Rules' -count=1`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add server/pkg/rules/legality_catalog.go server/pkg/rules/legality_catalog_test.go server/pkg/rules/prohibition.go server/pkg/rules/target_legality.go
git commit -m "refactor: add legality production rule catalog"
```

### Task 2: Extract Shared Source and Scope Matching

**Files:**
- Create: `server/pkg/rules/legality_shared_test.go`
- Create: `server/pkg/rules/legality_shared.go`
- Modify: `server/pkg/rules/prohibition.go`
- Modify: `server/pkg/rules/target_legality.go`
- Test: `server/pkg/rules/legality_shared_test.go`

- [ ] **Step 1: Write the failing helper tests**

```go
package rules

import "testing"

func TestCardMatchesDefinitionAndCondition(t *testing.T) {
	card := CardState{
		CardID:       "xq22-1",
		DefinitionID: "XQ22",
		Zone:         CardZoneTable,
		Exhausted:    false,
		Destroyed:    false,
		Revealed:     true,
	}

	if !cardMatchesDefinitionAndCondition(card, "XQ22", CardCondition{
		Zone:         CardZoneTable,
		Ready:        true,
		NotDestroyed: true,
		Revealed:     true,
	}) {
		t.Fatal("expected ready revealed XQ22 on table to match")
	}

	if cardMatchesDefinitionAndCondition(card, "XQ31", CardCondition{}) {
		t.Fatal("expected mismatched definitionId to fail")
	}
}

func TestScopeAppliesToActor(t *testing.T) {
	source := CardState{ControllerID: "P1"}

	if !scopeAppliesToActor(source, "P2", ProhibitionScope{Kind: ProhibitionScopeOpponentsOnly}) {
		t.Fatal("expected opponents-only scope to apply to P2")
	}
	if scopeAppliesToActor(source, "P1", ProhibitionScope{Kind: ProhibitionScopeOpponentsOnly}) {
		t.Fatal("expected opponents-only scope not to apply to controller")
	}
	if !scopeAppliesToActor(source, "P1", ProhibitionScope{Kind: ProhibitionScopeControllerOnly}) {
		t.Fatal("expected controller-only scope to apply to P1")
	}
}
```

- [ ] **Step 2: Run the tests to confirm they fail**

Run: `go test ./server/pkg/rules -run 'Test(CardMatchesDefinitionAndCondition|ScopeAppliesToActor)' -count=1`
Expected: FAIL with `undefined: cardMatchesDefinitionAndCondition` and `undefined: scopeAppliesToActor`

- [ ] **Step 3: Implement the shared helper file**

```go
package rules

func cardMatchesDefinitionAndCondition(card CardState, definitionID string, condition CardCondition) bool {
	if card.DefinitionID != definitionID {
		return false
	}
	if condition.Zone != "" && card.Zone != condition.Zone {
		return false
	}
	if condition.Ready && card.Exhausted {
		return false
	}
	if condition.NotDestroyed && card.Destroyed {
		return false
	}
	if condition.Revealed && !card.Revealed {
		return false
	}
	return true
}

func scopeAppliesToActor(sourceCard CardState, actorID string, scope ProhibitionScope) bool {
	switch scope.Kind {
	case ProhibitionScopeAllPlayers:
		return true
	case ProhibitionScopeOpponentsOnly:
		return sourceCard.ControllerID != "" && actorID != sourceCard.ControllerID
	case ProhibitionScopeControllerOnly:
		return sourceCard.ControllerID != "" && sourceCard.ControllerID == actorID
	default:
		return false
	}
}
```

- [ ] **Step 4: Replace duplicated logic in the evaluators**

```go
func (c *ScopedProhibitionChecker) matchesSourceCondition(card CardState, rule ProhibitionRule) bool {
	return cardMatchesDefinitionAndCondition(card, rule.SourceDefinitionID, rule.SourceCondition)
}

func (c *ScopedProhibitionChecker) matchesScope(
	state GameState,
	sourceCard CardState,
	actorID string,
	scope ProhibitionScope,
) bool {
	return scopeAppliesToActor(sourceCard, actorID, scope)
}

func (c *TargetLegalityChecker) matchesSourceCondition(card CardState, rule TargetLegalityRule) bool {
	return cardMatchesDefinitionAndCondition(card, rule.SourceDefinitionID, rule.SourceCondition)
}

func (c *TargetLegalityChecker) matchesActorRestriction(
	state GameState,
	sourceCard CardState,
	actorID string,
	restriction ProhibitionScope,
) bool {
	return scopeAppliesToActor(sourceCard, actorID, restriction)
}
```

- [ ] **Step 5: Run the helper and existing legality tests**

Run: `go test ./server/pkg/rules -run 'Test(CardMatchesDefinitionAndCondition|ScopeAppliesToActor|ProhibitionChecker|TargetLegalityXQ31)' -count=1`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add server/pkg/rules/legality_shared.go server/pkg/rules/legality_shared_test.go server/pkg/rules/prohibition.go server/pkg/rules/target_legality.go
git commit -m "refactor: share legality source and scope matchers"
```

### Task 3: Lock the Engine Boundary Around XQ31

**Files:**
- Modify: `server/pkg/rules/role_actions_test.go`
- Modify: `server/pkg/rules/prohibition_test.go`
- Test: `server/pkg/rules/role_actions_test.go`
- Test: `server/pkg/rules/prohibition_test.go`

- [ ] **Step 1: Add a regression test for `declare_investigation`**

```go
func TestDeclareInvestigationIgnoresXQ31TargetLegalityRestriction(t *testing.T) {
	state := newRoleActionTestState()
	state.Turn.Priority.CurrentPlayerID = "P2"
	state.Turn.ActivePlayerID = "P2"

	investigator := testCharacterCard("p2-investigator", "P2", CardNumericStats{Combat: 1, Defense: 2, Investigation: 1})
	xq31 := testCharacterCard("xq31-1", "P1", CardNumericStats{Combat: 1, Defense: 4})
	xq31.DefinitionID = "XQ31"
	xq31.ControllerID = "P1"
	xq31.PrintedKeywords = []string{"领袖", "公开", "声望"}
	region := testRegionCard("region-1")

	state.Board.Cards = []CardState{investigator, xq31, region}

	result, err := SubmitAction(state, Action{
		ID:           "act-investigation-xq31-boundary",
		ActorID:      "P2",
		Kind:         ActionKindDeclareInvestigation,
		CardID:       "p2-investigator",
		TargetCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}
	if result.Event.Kind != EventKindInvestigationApplied {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindInvestigationApplied)
	}
}
```

- [ ] **Step 2: Add a regression test proving production prohibition builders ignore test-only rules**

```go
func TestBuildProhibitionCheckerIgnoresTestOnlyRules(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-production-prohibition-only",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:       "TEST01-1",
			DefinitionID: "TEST01",
			Zone:         CardZoneTable,
			ControllerID: "P1",
		},
	}

	result := BuildProhibitionChecker(state).Check(state, "P2", TargetCategory{
		BasicTypes: []string{"角色"},
	})
	if result.Prohibited {
		t.Fatalf("expected production checker to ignore TEST01, got source %q", result.SourceCardID)
	}
}
```

- [ ] **Step 3: Run the focused regression tests**

Run: `go test ./server/pkg/rules -run 'Test(DeclareInvestigationIgnoresXQ31TargetLegalityRestriction|BuildProhibitionCheckerIgnoresTestOnlyRules)' -count=1`
Expected: PASS

- [ ] **Step 4: If anything fails, fix only the minimal boundary code**

```go
// Keep this boundary intact:
if action.TargetCardID != "" && action.Kind == ActionKindQueueOperation {
	targetLegalityChecker := BuildTargetLegalityChecker(state)
	targetResult := targetLegalityChecker.CheckTargetCard(state, action.ActorID, action.TargetCardID)
	// ...
}
```

- [ ] **Step 5: Commit**

```bash
git add server/pkg/rules/role_actions_test.go server/pkg/rules/prohibition_test.go server/pkg/rules/engine.go
git commit -m "test: lock legality engine boundaries"
```

### Task 4: Sync Docs and Run Full Verification

**Files:**
- Modify: `docs/HANDOVER_TRAE_2026-04-01.md`
- Modify: `docs/NEXT_GEN_RULE_PLAN.md`

- [ ] **Step 1: Update the handover doc**

```md
- legality production rules now live in `server/pkg/rules/legality_catalog.go`
- shared source-condition and actor-scope matching now live in `server/pkg/rules/legality_shared.go`
- `XQ31` remains queue-operation-only targeting; role actions stay outside this gate
- `XQ01` remains deferred pending region / ability-kind model
```

- [ ] **Step 2: Update the roadmap doc**

```md
- Phase 3 legality hardening now has a dedicated production rule catalog
- duplicated legality matcher logic has been collapsed into shared helpers
- next follow-up remains `XQ31` numeric aura completion or `XQ01` prerequisite design, not UI work
```

- [ ] **Step 3: Run Go verification**

Run: `go test ./server/...`
Expected: PASS

- [ ] **Step 4: Run TypeScript verification**

Run: `cd tools/fixture-tools && npm test`
Expected: PASS

Run: `cd web && npm test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add docs/HANDOVER_TRAE_2026-04-01.md docs/NEXT_GEN_RULE_PLAN.md
git commit -m "docs: sync legality framework hardening"
```

## Follow-On After This Plan

Do not roll straight into more card count after this plan. The recommended next sequence is:

1. Finish the other half of `XQ31` with a minimal continuous numeric aura for allied prestige characters.
2. Write a spec for `XQ01` region-scoped ability silence before touching code.
3. Only then start `Attachment / Permanent Model V1`, where `BQ022` becomes a real table permanent instead of attachment-tracking metadata.

## Final Verification Bundle

Run this bundle before handing work back:

```bash
go test ./server/...
cd tools/fixture-tools && npm test
cd ../web && npm test
```
