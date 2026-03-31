package rules

import (
	"testing"
)

// TestMultiCardProhibition verifies that the prohibition framework supports multiple cards.
// Uses explicit rule construction to avoid polluting production BuildProhibitionChecker.
func TestMultiCardProhibition(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-multi",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add both XQ22 and TEST01 to the board
	state.Board.Cards = []CardState{
		{
			CardID:       "XQ22-1",
			DefinitionID: "XQ22",
			Name:         "州议员贝伦·希恩斯",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
			ControllerID: "P1",
		},
		{
			CardID:       "TEST01-1",
			DefinitionID: "TEST01",
			Name:         "测试禁角色卡",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
			ControllerID: "P1",
		},
	}

	// Build checker with explicit test rules (not using production BuildProhibitionChecker)
	rules := []ProhibitionRule{
		XQ22ProhibitionRule,
		TEST01ProhibitionRule,
	}
	checker := NewScopedProhibitionChecker(rules)

	// Verify XQ22 still prohibits event cards (事务)
	targetEvent := TargetCategory{
		BasicTypes: []string{"事务"},
	}
	resultEvent := checker.Check(state, "P1", targetEvent)
	if !resultEvent.Prohibited {
		t.Fatal("expected event cards (事务) to be prohibited by XQ22")
	}

	// Verify TEST01 prohibits character cards (角色)
	targetCharacter := TargetCategory{
		BasicTypes: []string{"角色"},
	}
	resultCharacter := checker.Check(state, "P1", targetCharacter)
	if !resultCharacter.Prohibited {
		t.Fatal("expected character cards (角色) to be prohibited by TEST01")
	}

	// Verify non-event/non-character cards are allowed
	targetRegion := TargetCategory{
		BasicTypes: []string{"地区"},
	}
	resultRegion := checker.Check(state, "P1", targetRegion)
	if resultRegion.Prohibited {
		t.Fatal("expected region cards (地区) to be allowed")
	}
}

// TestMultiCardProhibitionDifferentScopes verifies that different scopes work correctly.
func TestMultiCardProhibitionDifferentScopes(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-multi-scope",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add TEST02 (opponents only scope)
	state.Board.Cards = []CardState{
		{
			CardID:       "TEST02-1",
			DefinitionID: "TEST02",
			Name:         "测试仅对手禁用卡",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
			ControllerID: "P1",
		},
	}

	// Build checker with explicit test rules
	rules := []ProhibitionRule{
		TEST02ProhibitionRule,
	}
	checker := NewScopedProhibitionChecker(rules)

	targetCategory := TargetCategory{
		BasicTypes: []string{"事务"},
	}

	// P2 (opponent) should be prohibited
	resultP2 := checker.Check(state, "P2", targetCategory)
	if !resultP2.Prohibited {
		t.Fatal("expected P2 (opponent) to be prohibited by TEST02")
	}

	// P1 (controller) should NOT be prohibited
	resultP1 := checker.Check(state, "P1", targetCategory)
	if resultP1.Prohibited {
		t.Fatal("expected P1 (controller) to NOT be prohibited by TEST02")
	}
}

// TestProhibitionCheckerControllerOnlyScope verifies that controller_only scope works correctly.
func TestProhibitionCheckerControllerOnlyScope(t *testing.T) {
	// Create a test rule for controller_only (we'll create it in the checker for this test)
	state := NewGameState(InitialStateConfig{
		GameID:         "test-controller-only",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Create TEST03 (controller only scope)
	state.Board.Cards = []CardState{
		{
			CardID:       "TEST03-1",
			DefinitionID: "TEST03",
			Name:         "测试仅控制者禁用卡",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
			ControllerID: "P1",
		},
	}

	// Build a checker with TEST03 rule
	testRule := ProhibitionRule{
		SourceDefinitionID: "TEST03",
		SourceCondition: CardCondition{
			Zone:         CardZoneTable,
			Ready:        true,
			NotDestroyed: true,
		},
		Scope: ProhibitionScope{
			Kind: ProhibitionScopeControllerOnly,
		},
		TargetCategory: TargetCategory{
			BasicTypes: []string{"事务"},
		},
		Description: "TEST03: Only controller can't play event cards",
	}

	checker := NewScopedProhibitionChecker([]ProhibitionRule{testRule})

	targetCategory := TargetCategory{
		BasicTypes: []string{"事务"},
	}

	// P1 (controller) should be prohibited
	resultP1 := checker.Check(state, "P1", targetCategory)
	if !resultP1.Prohibited {
		t.Fatal("expected P1 (controller) to be prohibited by TEST03")
	}

	// P2 (opponent) should NOT be prohibited
	resultP2 := checker.Check(state, "P2", targetCategory)
	if resultP2.Prohibited {
		t.Fatal("expected P2 (opponent) to NOT be prohibited by TEST03")
	}
}

// TestProhibitionCheckerEmptyControllerID verifies that empty ControllerID doesn't cause issues.
func TestProhibitionCheckerEmptyControllerID(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-empty-controller",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add TEST02 with empty ControllerID
	state.Board.Cards = []CardState{
		{
			CardID:       "TEST02-1",
			DefinitionID: "TEST02",
			Name:         "测试仅对手禁用卡",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
			ControllerID: "", // Empty!
		},
	}

	// Build checker with explicit test rules
	rules := []ProhibitionRule{
		TEST02ProhibitionRule,
	}
	checker := NewScopedProhibitionChecker(rules)

	targetCategory := TargetCategory{
		BasicTypes: []string{"事务"},
	}

	// Both players should NOT be prohibited (empty ControllerID means rule doesn't apply)
	resultP1 := checker.Check(state, "P1", targetCategory)
	if resultP1.Prohibited {
		t.Fatal("expected P1 to NOT be prohibited with empty ControllerID")
	}

	resultP2 := checker.Check(state, "P2", targetCategory)
	if resultP2.Prohibited {
		t.Fatal("expected P2 to NOT be prohibited with empty ControllerID")
	}
}
