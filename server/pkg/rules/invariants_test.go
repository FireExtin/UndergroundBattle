package rules

import (
	"testing"
)

// TestInvariantCardIDUniquePass verifies that unique card IDs pass.
func TestInvariantCardIDUniquePass(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-unique-pass",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	state.Board.Cards = []CardState{
		{CardID: "card-1", Zone: CardZoneTable},
		{CardID: "card-2", Zone: CardZoneTable},
		{CardID: "card-3", Zone: CardZoneHand},
	}

	if !InvariantCardIDUnique(state) {
		t.Fatal("expected unique card IDs to pass")
	}
}

// TestInvariantCardIDUniqueFail verifies that duplicate card IDs fail.
func TestInvariantCardIDUniqueFail(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-unique-fail",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	state.Board.Cards = []CardState{
		{CardID: "card-1", Zone: CardZoneTable},
		{CardID: "card-2", Zone: CardZoneTable},
		{CardID: "card-1", Zone: CardZoneHand}, // Duplicate!
	}

	if InvariantCardIDUnique(state) {
		t.Fatal("expected duplicate card IDs to fail")
	}
}

// TestInvariantCardZoneValidPass verifies that valid zones pass.
func TestInvariantCardZoneValidPass(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-zone-pass",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	state.Board.Cards = []CardState{
		{CardID: "card-1", Zone: CardZoneTable},
		{CardID: "card-2", Zone: CardZoneHand},
		{CardID: "card-3", Zone: CardZoneDiscard},
		{CardID: "card-4", Zone: CardZoneDeck},
	}

	if !InvariantCardZoneValid(state) {
		t.Fatal("expected valid zones to pass")
	}
}

// TestInvariantCardZoneValidFail verifies that invalid zones fail.
func TestInvariantCardZoneValidFail(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-zone-fail",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	state.Board.Cards = []CardState{
		{CardID: "card-1", Zone: CardZoneTable},
		{CardID: "card-2", Zone: CardZone("invalid_zone")}, // Invalid!
	}

	if InvariantCardZoneValid(state) {
		t.Fatal("expected invalid zone to fail")
	}
}

// TestInvariantPriorityPlayerValidPass verifies that valid priority player passes.
func TestInvariantPriorityPlayerValidPass(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-priority-pass",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	state.Turn.Priority.CurrentPlayerID = "P1"

	if !InvariantPriorityPlayerValid(state) {
		t.Fatal("expected valid priority player to pass")
	}
}

// TestInvariantPriorityPlayerValidEmptyPass verifies that empty priority player passes.
func TestInvariantPriorityPlayerValidEmptyPass(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-priority-empty",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	state.Turn.Priority.CurrentPlayerID = "" // Empty is allowed

	if !InvariantPriorityPlayerValid(state) {
		t.Fatal("expected empty priority player to pass")
	}
}

// TestInvariantPriorityPlayerValidFail verifies that invalid priority player fails.
func TestInvariantPriorityPlayerValidFail(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-priority-fail",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	state.Turn.Priority.CurrentPlayerID = "P3" // Not in players list!

	if InvariantPriorityPlayerValid(state) {
		t.Fatal("expected invalid priority player to fail")
	}
}

// TestInvariantStackDepthNonNegativePass verifies that non-negative stack depth passes.
func TestInvariantStackDepthNonNegativePass(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-stack-pass",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	state.Board.Stack = []Operation{
		{ID: "op-1"},
		{ID: "op-2"},
	}

	if !InvariantStackDepthNonNegative(state) {
		t.Fatal("expected non-negative stack depth to pass")
	}
}

// TestInvariantStackDepthNonNegativeEmptyPass verifies that empty stack passes.
func TestInvariantStackDepthNonNegativeEmptyPass(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-stack-empty",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	state.Board.Stack = []Operation{} // Empty stack

	if !InvariantStackDepthNonNegative(state) {
		t.Fatal("expected empty stack to pass")
	}
}

// TestInvariantRevisionConsistentPass verifies that consistent revision passes.
func TestInvariantRevisionConsistentPass(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-revision-pass",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Revision 0, no actions
	state.Revision.Number = 0
	state.History.Actions = []Action{}

	if !InvariantRevisionConsistent(state) {
		t.Fatal("expected revision 0 with no actions to pass")
	}

	// Revision 1, one action
	state.Revision.Number = 1
	state.History.Actions = []Action{{ID: "action-1"}}

	if !InvariantRevisionConsistent(state) {
		t.Fatal("expected revision 1 with one action to pass")
	}

	// Revision 2, one action (one ahead)
	state.Revision.Number = 2
	state.History.Actions = []Action{{ID: "action-1"}}

	if !InvariantRevisionConsistent(state) {
		t.Fatal("expected revision 2 with one action to pass (one ahead)")
	}
}

// TestInvariantRevisionConsistentFail verifies that inconsistent revision fails.
func TestInvariantRevisionConsistentFail(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-revision-fail",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Negative revision
	state.Revision.Number = -1
	state.History.Actions = []Action{}

	if InvariantRevisionConsistent(state) {
		t.Fatal("expected negative revision to fail")
	}

	// Revision 5, only 1 action (too far ahead)
	state.Revision.Number = 5
	state.History.Actions = []Action{{ID: "action-1"}}

	if InvariantRevisionConsistent(state) {
		t.Fatal("expected revision too far ahead to fail")
	}
}

// TestCheckAllInvariants verifies that CheckAllInvariants runs all checks.
func TestCheckAllInvariants(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-all-invariants",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	state.Board.Cards = []CardState{
		{CardID: "card-1", Zone: CardZoneTable},
	}
	state.Turn.Priority.CurrentPlayerID = "P1"

	results := CheckAllInvariants(state, DefaultInvariantConfig)

	if len(results) != 5 {
		t.Fatalf("expected 5 invariant results, got %d", len(results))
	}

	for _, result := range results {
		if !result.Passed {
			t.Fatalf("invariant %s failed: %s", result.Name, result.Message)
		}
	}
}

// TestCheckAllInvariantsDisabled verifies that disabled config returns nil.
func TestCheckAllInvariantsDisabled(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-disabled",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	config := InvariantConfig{Enabled: false}
	results := CheckAllInvariants(state, config)

	if results != nil {
		t.Fatal("expected nil results when disabled")
	}
}

// TestAssertInvariantsPass verifies that AssertInvariants doesn't panic on valid state.
func TestAssertInvariantsPass(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-assert-pass",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	state.Board.Cards = []CardState{
		{CardID: "card-1", Zone: CardZoneTable},
	}
	state.Turn.Priority.CurrentPlayerID = "P1"

	// Should not panic
	defer func() {
		if r := recover(); r != nil {
			t.Fatalf("AssertInvariants panicked on valid state: %v", r)
		}
	}()

	AssertInvariants(state)
}

// TestAssertInvariantsFail verifies that AssertInvariants panics on invalid state.
func TestAssertInvariantsFail(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-assert-fail",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Create duplicate card IDs
	state.Board.Cards = []CardState{
		{CardID: "card-1", Zone: CardZoneTable},
		{CardID: "card-1", Zone: CardZoneHand}, // Duplicate!
	}

	// Should panic
	defer func() {
		if r := recover(); r == nil {
			t.Fatal("expected AssertInvariants to panic on invalid state")
		}
	}()

	AssertInvariants(state)
}
