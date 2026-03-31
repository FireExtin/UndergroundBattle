package rules

import (
	"testing"
)

// TestGoldenScenario_XQ22BlocksEventCard verifies that XQ22 prevents event cards from being played.
// Scenario: XQ22 禁止事务卡
func TestGoldenScenario_XQ22BlocksEventCard(t *testing.T) {
	// Given: P1 has XQ22 ready on the table
	state := NewGameState(InitialStateConfig{
		GameID:         "golden-xq22",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add XQ22 to P1's table
	xq22Card := CardState{
		CardID:         "XQ22-1",
		DefinitionID:   "XQ22",
		Name:           "州议员贝伦·希恩斯",
		Zone:           CardZoneTable,
		Exhausted:      false,
		Destroyed:      false,
		ControllerID:   "P1",
		PrintedKeywords: []string{"角色"},
	}
	state.Board.Cards = []CardState{xq22Card}

	// Use prohibition checker directly to verify the rule
	checker := BuildProhibitionChecker(state)
	targetCategory := TargetCategory{
		BasicTypes: []string{"事务"},
	}

	// When: P2 tries to play an event card
	result := checker.Check(state, "P2", targetCategory)

	// Then: Should be prohibited
	if !result.Prohibited {
		t.Fatal("expected event cards to be prohibited when XQ22 is present")
	}

	// Verify source is XQ22
	if result.SourceCardID != "XQ22-1" {
		t.Fatalf("expected source to be XQ22-1, got %s", result.SourceCardID)
	}
}

// TestGoldenScenario_XQ31ProtectsPrestigeAlly verifies that XQ31 prevents enemies from targeting prestige allies.
// Scenario: XQ31 保护声望盟友
func TestGoldenScenario_XQ31ProtectsPrestigeAlly(t *testing.T) {
	// Given: P1 has XQ31 and a prestige ally
	state := NewGameState(InitialStateConfig{
		GameID:         "golden-xq31",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add XQ31 to P1's table
	xq31Card := CardState{
		CardID:         "XQ31-1",
		DefinitionID:   "XQ31",
		Name:           "莫兰大主教",
		Zone:           CardZoneTable,
		Exhausted:      false,
		Destroyed:      false,
		ControllerID:   "P1",
		PrintedKeywords: []string{"领袖", "公开", "声望"},
	}

	// Add prestige ally to P1's table
	prestigeAlly := CardState{
		CardID:         "ALLY-1",
		DefinitionID:   "ALLY",
		Name:           "声望盟友",
		Zone:           CardZoneTable,
		Exhausted:      false,
		Destroyed:      false,
		ControllerID:   "P1",
		PrintedKeywords: []string{"声望"},
	}

	state.Board.Cards = []CardState{xq31Card, prestigeAlly}

	// Use target legality checker directly
	checker := BuildTargetLegalityChecker(state)

	// When: P2 tries to target the prestige ally
	result := checker.CheckTargetCard(state, "P2", "ALLY-1")

	// Then: Should not be allowed to target
	if result.CanTarget {
		t.Fatal("expected P2 to NOT be able to target prestige ally protected by XQ31")
	}

	// Verify source is XQ31
	if result.SourceCardID != "XQ31-1" {
		t.Fatalf("expected source to be XQ31-1, got %s", result.SourceCardID)
	}
}

// TestGoldenScenario_FullGameTurn verifies a complete game turn with stack resolution.
// Scenario: 完整游戏回合
func TestGoldenScenario_FullGameTurn(t *testing.T) {
	// Given: Initial game state with P1 having priority
	state := NewGameState(InitialStateConfig{
		GameID:         "golden-full-turn",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Verify initial state
	if state.Turn.Priority.CurrentPlayerID != "P1" {
		t.Fatalf("expected P1 to have priority, got %s", state.Turn.Priority.CurrentPlayerID)
	}

	initialRevision := state.Revision.Number

	// Step 1: P1 passes priority
	action1 := Action{
		ID:      "action-1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	}

	result1, err := SubmitAction(state, action1)
	if err != nil {
		t.Fatalf("action 1 should succeed: %v", err)
	}
	state = result1.State

	// Verify revision incremented
	if state.Revision.Number != initialRevision+1 {
		t.Fatalf("expected revision %d, got %d", initialRevision+1, state.Revision.Number)
	}

	// Step 2: P2 passes priority
	action2 := Action{
		ID:      "action-2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	}

	result2, err := SubmitAction(state, action2)
	if err != nil {
		t.Fatalf("action 2 should succeed: %v", err)
	}
	state = result2.State

	// Verify revision incremented
	if state.Revision.Number != initialRevision+2 {
		t.Fatalf("expected revision %d, got %d", initialRevision+2, state.Revision.Number)
	}

	// Verify invariants pass
	results := CheckAllInvariants(state, DefaultInvariantConfig)
	for _, result := range results {
		if !result.Passed {
			t.Fatalf("invariant %s failed: %s", result.Name, result.Message)
		}
	}
}

// TestGoldenScenario_XQ22AllowsNonEventCards verifies that XQ22 doesn't block non-event cards.
func TestGoldenScenario_XQ22AllowsNonEventCards(t *testing.T) {
	// Given: P1 has XQ22 ready on the table
	state := NewGameState(InitialStateConfig{
		GameID:         "golden-xq22-allows",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add XQ22 to P1's table
	xq22Card := CardState{
		CardID:         "XQ22-1",
		DefinitionID:   "XQ22",
		Name:           "州议员贝伦·希恩斯",
		Zone:           CardZoneTable,
		Exhausted:      false,
		Destroyed:      false,
		ControllerID:   "P1",
		PrintedKeywords: []string{"角色"},
	}
	state.Board.Cards = []CardState{xq22Card}

	// When: P2 tries to play a character card (not event)
	checker := BuildProhibitionChecker(state)
	targetCategory := TargetCategory{
		BasicTypes: []string{"角色"}, // Character, not event
	}

	result := checker.Check(state, "P2", targetCategory)

	// Then: Should NOT be prohibited
	if result.Prohibited {
		t.Fatal("Character cards should NOT be prohibited by XQ22")
	}
}

// TestGoldenScenario_XQ31AllowsAllyToTargetPrestige verifies that allies can target their own prestige.
func TestGoldenScenario_XQ31AllowsAllyToTargetPrestige(t *testing.T) {
	// Given: P1 has XQ31 and a prestige ally
	state := NewGameState(InitialStateConfig{
		GameID:         "golden-xq31-ally",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add XQ31 to P1's table
	xq31Card := CardState{
		CardID:         "XQ31-1",
		DefinitionID:   "XQ31",
		Name:           "莫兰大主教",
		Zone:           CardZoneTable,
		Exhausted:      false,
		Destroyed:      false,
		ControllerID:   "P1",
		PrintedKeywords: []string{"领袖", "公开", "声望"},
	}

	// Add prestige ally to P1's table
	prestigeAlly := CardState{
		CardID:         "ALLY-1",
		DefinitionID:   "ALLY",
		Name:           "声望盟友",
		Zone:           CardZoneTable,
		Exhausted:      false,
		Destroyed:      false,
		ControllerID:   "P1",
		PrintedKeywords: []string{"声望"},
	}

	state.Board.Cards = []CardState{xq31Card, prestigeAlly}

	// When: P1 (ally) tries to target their own prestige ally
	checker := BuildTargetLegalityChecker(state)
	result := checker.CheckTargetCard(state, "P1", "ALLY-1")

	// Then: Should be allowed (P1 is not enemy of P1)
	if !result.CanTarget {
		t.Fatal("P1 should be able to target their own prestige ally")
	}
}

// TestGoldenScenario_RevisionConsistency verifies that revision numbers increment correctly.
func TestGoldenScenario_RevisionConsistency(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "golden-revision",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	initialRevision := state.Revision.Number

	// Execute action as P1
	action1 := Action{
		ID:      "action-1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	}

	result1, err := SubmitAction(state, action1)
	if err != nil {
		t.Fatalf("action 1 should succeed: %v", err)
	}
	state = result1.State

	// Verify revision incremented
	if state.Revision.Number != initialRevision+1 {
		t.Fatalf("expected revision %d, got %d", initialRevision+1, state.Revision.Number)
	}

	// Execute action as P2
	action2 := Action{
		ID:      "action-2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	}

	result2, err := SubmitAction(state, action2)
	if err != nil {
		t.Fatalf("action 2 should succeed: %v", err)
	}
	state = result2.State

	// Verify revision incremented again
	if state.Revision.Number != initialRevision+2 {
		t.Fatalf("expected revision %d, got %d", initialRevision+2, state.Revision.Number)
	}
}

// TestGoldenScenario_InvariantsAfterActions verifies invariants hold after multiple actions.
func TestGoldenScenario_InvariantsAfterActions(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "golden-invariants",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add some cards
	state.Board.Cards = []CardState{
		{CardID: "card-1", Zone: CardZoneTable, ControllerID: "P1"},
		{CardID: "card-2", Zone: CardZoneHand, ControllerID: "P2"},
	}

	// Execute multiple actions (alternating to respect priority)
	actions := []Action{
		{ID: "a1", ActorID: "P1", Kind: ActionKindPassPriority},
		{ID: "a2", ActorID: "P2", Kind: ActionKindPassPriority},
	}

	for _, action := range actions {
		result, err := SubmitAction(state, action)
		if err != nil {
			t.Fatalf("action %s should succeed: %v", action.ID, err)
		}
		state = result.State

		// Check invariants after each action
		invariantResults := CheckAllInvariants(state, DefaultInvariantConfig)
		for _, invResult := range invariantResults {
			if !invResult.Passed {
				t.Fatalf("after action %s: invariant %s failed: %s", action.ID, invResult.Name, invResult.Message)
			}
		}
	}
}
