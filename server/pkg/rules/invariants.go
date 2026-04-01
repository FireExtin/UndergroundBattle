package rules

// Purpose: Defines and checks game state invariants to ensure system health.

// InvariantFunc is a function that checks a specific invariant.
type InvariantFunc func(GameState) bool

// InvariantCheckResult contains the result of an invariant check.
type InvariantCheckResult struct {
	Name    string // Invariant name
	Passed  bool   // Whether the check passed
	Message string // Failure message (if failed)
}

// InvariantConfig controls invariant checking behavior.
type InvariantConfig struct {
	Enabled     bool // Whether to enable invariant checking
	PanicOnFail bool // Whether to panic on failure (for testing/debugging only)
}

// DefaultInvariantConfig is the default configuration for invariant checking.
var DefaultInvariantConfig = InvariantConfig{
	Enabled:     true,
	PanicOnFail: false,
}

// CheckAllInvariants runs all invariant checks and returns results.
func CheckAllInvariants(state GameState, config InvariantConfig) []InvariantCheckResult {
	if !config.Enabled {
		return nil
	}

	invariants := []struct {
		name string
		fn   InvariantFunc
	}{
		{"CardIDUnique", InvariantCardIDUnique},
		{"CardZoneValid", InvariantCardZoneValid},
		{"CardDestroyedStateValid", InvariantCardDestroyedStateValid},
		{"PriorityPlayerValid", InvariantPriorityPlayerValid},
		{"StackDepthNonNegative", InvariantStackDepthNonNegative},
		{"RevisionConsistent", InvariantRevisionConsistent},
	}

	results := make([]InvariantCheckResult, 0, len(invariants))
	for _, inv := range invariants {
		passed := inv.fn(state)
		result := InvariantCheckResult{
			Name:   inv.name,
			Passed: passed,
		}
		if !passed {
			result.Message = "invariant violated: " + inv.name
		}
		results = append(results, result)
	}

	return results
}

// AssertInvariants panics if any invariant fails (for testing/debugging).
func AssertInvariants(state GameState) {
	results := CheckAllInvariants(state, InvariantConfig{Enabled: true, PanicOnFail: true})
	for _, result := range results {
		if !result.Passed {
			panic(result.Message)
		}
	}
}

// InvariantCardIDUnique checks that all card IDs are unique.
func InvariantCardIDUnique(state GameState) bool {
	seen := make(map[string]bool)
	for _, card := range state.Board.Cards {
		if seen[card.CardID] {
			return false
		}
		seen[card.CardID] = true
	}
	return true
}

// InvariantCardZoneValid checks that all cards are in valid zones.
func InvariantCardZoneValid(state GameState) bool {
	validZones := map[CardZone]bool{
		CardZoneHand:    true,
		CardZoneTable:   true,
		CardZoneDiscard: true,
		CardZoneDeck:    true,
		CardZoneScore:   true,
	}
	for _, card := range state.Board.Cards {
		if !validZones[card.Zone] {
			return false
		}
	}
	return true
}

// InvariantCardDestroyedStateValid checks that card Destroyed state is consistent with Zone.
func InvariantCardDestroyedStateValid(state GameState) bool {
	for _, card := range state.Board.Cards {
		if card.Zone == CardZoneTable && card.Destroyed {
			return false
		}
		if card.Zone == CardZoneDiscard && !card.Destroyed {
			return false
		}
	}
	return true
}

// InvariantPriorityPlayerValid checks that the priority player exists in the players list.
// This checks the actual value that the engine will use (including fallback to PriorityPlayerID).
func InvariantPriorityPlayerValid(state GameState) bool {
	// Use the same logic as currentPriorityPlayerID in engine.go
	priorityPlayer := state.Turn.Priority.CurrentPlayerID
	if priorityPlayer == "" {
		priorityPlayer = state.Turn.PriorityPlayerID
	}
	if priorityPlayer == "" {
		return true // Empty means no priority (game might not have started)
	}
	for _, player := range state.Players {
		if player == priorityPlayer {
			return true
		}
	}
	return false
}

// InvariantStackDepthNonNegative checks that stack depth is non-negative.
func InvariantStackDepthNonNegative(state GameState) bool {
	return len(state.Board.Stack) >= 0
}

// InvariantRevisionConsistent checks that revision number is consistent with history.
func InvariantRevisionConsistent(state GameState) bool {
	// Revision number must be non-negative
	if state.Revision.Number < 0 {
		return false
	}
	// On committed authoritative state, revision must match action history exactly.
	expectedRevision := len(state.History.Actions)
	return state.Revision.Number == expectedRevision
}
