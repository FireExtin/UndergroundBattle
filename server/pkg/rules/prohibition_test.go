package rules

import (
	"testing"
)

// TestProhibitionCheckerEmptyStateAllowsAll verifies that with no prohibition sources, all actions are allowed.
func TestProhibitionCheckerEmptyStateAllowsAll(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-empty",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	checker := BuildProhibitionChecker(state)
	targetCategory := TargetCategory{
		BasicTypes: []string{"事务"},
	}

	result := checker.Check(state, "P1", targetCategory)
	if result.Prohibited {
		t.Fatalf("expected no prohibition with empty state, got prohibited by %s", result.SourceCardName)
	}
}

// TestProhibitionCheckerMatchesSourceCondition verifies that source conditions are properly checked.
func TestProhibitionCheckerMatchesSourceCondition(t *testing.T) {
	tests := []struct {
		name        string
		card        CardState
		shouldMatch bool
	}{
		{
			name: "ready on table",
			card: CardState{
				CardID:       "XQ22-1",
				DefinitionID: "XQ22",
				Zone:         CardZoneTable,
				Exhausted:    false,
				Destroyed:    false,
			},
			shouldMatch: true,
		},
		{
			name: "exhausted on table",
			card: CardState{
				CardID:       "XQ22-1",
				DefinitionID: "XQ22",
				Zone:         CardZoneTable,
				Exhausted:    true,
				Destroyed:    false,
			},
			shouldMatch: false,
		},
		{
			name: "destroyed on table",
			card: CardState{
				CardID:       "XQ22-1",
				DefinitionID: "XQ22",
				Zone:         CardZoneTable,
				Exhausted:    false,
				Destroyed:    true,
			},
			shouldMatch: false,
		},
		{
			name: "ready in hand",
			card: CardState{
				CardID:       "XQ22-1",
				DefinitionID: "XQ22",
				Zone:         CardZoneHand,
				Exhausted:    false,
				Destroyed:    false,
			},
			shouldMatch: false,
		},
		{
			name: "wrong definition ID",
			card: CardState{
				CardID:       "OTHER-1",
				DefinitionID: "OTHER",
				Zone:         CardZoneTable,
				Exhausted:    false,
				Destroyed:    false,
			},
			shouldMatch: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			state := NewGameState(InitialStateConfig{
				GameID:         "test-condition",
				ActivePlayerID: "P1",
				PlayerIDs:      []string{"P1", "P2"},
			})
			state.Board.Cards = []CardState{tt.card}

			checker := BuildProhibitionChecker(state)
			targetCategory := TargetCategory{
				BasicTypes: []string{"事务"},
			}

			result := checker.Check(state, "P1", targetCategory)

			if tt.shouldMatch && !result.Prohibited {
				t.Fatalf("expected prohibition to match, but it didn't")
			}
			if !tt.shouldMatch && result.Prohibited {
				t.Fatalf("expected prohibition not to match, but it matched %s", result.SourceCardName)
			}
		})
	}
}

// TestProhibitionCheckerRespectsScope verifies that scope restrictions are properly enforced.
func TestProhibitionCheckerRespectsScope(t *testing.T) {
	// Create a state with XQ22 controlled by P1
	state := NewGameState(InitialStateConfig{
		GameID:         "test-scope",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:       "XQ22-1",
			DefinitionID: "XQ22",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
			ControllerID: "P1",
		},
	}

	checker := BuildProhibitionChecker(state)
	targetCategory := TargetCategory{
		BasicTypes: []string{"事务"},
	}

	// XQ22 has AllPlayers scope, so both P1 and P2 should be prohibited
	resultP1 := checker.Check(state, "P1", targetCategory)
	if !resultP1.Prohibited {
		t.Fatal("expected P1 to be prohibited by XQ22 with AllPlayers scope")
	}

	resultP2 := checker.Check(state, "P2", targetCategory)
	if !resultP2.Prohibited {
		t.Fatal("expected P2 to be prohibited by XQ22 with AllPlayers scope")
	}
}

// TestProhibitionCheckerMatchesTargetCategory verifies that target category matching works correctly.
func TestProhibitionCheckerMatchesTargetCategory(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-target",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:       "XQ22-1",
			DefinitionID: "XQ22",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
		},
	}

	checker := BuildProhibitionChecker(state)

	tests := []struct {
		name           string
		basicType      string
		shouldProhibit bool
	}{
		{
			name:           "event card (事务)",
			basicType:      "事务",
			shouldProhibit: true,
		},
		{
			name:           "character card (角色)",
			basicType:      "角色",
			shouldProhibit: false,
		},
		{
			name:           "region card (地区)",
			basicType:      "地区",
			shouldProhibit: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			targetCategory := TargetCategory{
				BasicTypes: []string{tt.basicType},
			}

			result := checker.Check(state, "P1", targetCategory)

			if tt.shouldProhibit && !result.Prohibited {
				t.Fatalf("expected %s to be prohibited, but it wasn't", tt.basicType)
			}
			if !tt.shouldProhibit && result.Prohibited {
				t.Fatalf("expected %s not to be prohibited, but it was", tt.basicType)
			}
		})
	}
}

func TestProhibitionCheckerMatchesDynamicSourceRegionCondition(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-source-region",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:       "XQ01-1",
			DefinitionID: "XQ01",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
			ControllerID: "P1",
			RegionCardID: "region-1",
		},
	}

	checker := NewScopedProhibitionChecker([]ProhibitionRule{
		{
			SourceDefinitionID: "XQ01",
			SourceCondition: CardCondition{
				Zone:         CardZoneTable,
				Ready:        true,
				NotDestroyed: true,
			},
			Scope: ProhibitionScope{
				Kind: ProhibitionScopeOpponentsOnly,
			},
			TargetCategory: TargetCategory{
				ActionKinds: []ActionKind{ActionKindDeclareAttack},
				Condition: &TargetCondition{
					Kinds:        []CardKind{CardKindCharacter},
					RegionID:     targetRegionScopeSource,
					AbilityKinds: []string{"action"},
				},
			},
		},
	})

	sameRegion := checker.Check(state, "P2", TargetCategory{
		ActionKinds: []ActionKind{ActionKindDeclareAttack},
		Condition: &TargetCondition{
			Kinds:        []CardKind{CardKindCharacter},
			RegionID:     "region-1",
			AbilityKinds: []string{"action"},
		},
	})
	if !sameRegion.Prohibited {
		t.Fatal("expected same-region target category to be prohibited")
	}

	otherRegion := checker.Check(state, "P2", TargetCategory{
		ActionKinds: []ActionKind{ActionKindDeclareAttack},
		Condition: &TargetCondition{
			Kinds:        []CardKind{CardKindCharacter},
			RegionID:     "region-2",
			AbilityKinds: []string{"action"},
		},
	})
	if otherRegion.Prohibited {
		t.Fatal("expected different-region target category not to be prohibited")
	}
}

// TestProhibitionCheckerMultipleSources verifies that the checker handles multiple prohibition sources.
func TestProhibitionCheckerMultipleSources(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-multi",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add two XQ22 cards (both should be active)
	state.Board.Cards = []CardState{
		{
			CardID:       "XQ22-1",
			DefinitionID: "XQ22",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
			ControllerID: "P1",
		},
		{
			CardID:       "XQ22-2",
			DefinitionID: "XQ22",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
			ControllerID: "P2",
		},
	}

	checker := BuildProhibitionChecker(state)
	targetCategory := TargetCategory{
		BasicTypes: []string{"事务"},
	}

	result := checker.Check(state, "P1", targetCategory)
	if !result.Prohibited {
		t.Fatal("expected prohibition with multiple XQ22 sources")
	}

	// Verify that we got one of the XQ22 cards as the source
	if result.SourceCardID != "XQ22-1" && result.SourceCardID != "XQ22-2" {
		t.Fatalf("expected source to be one of the XQ22 cards, got %s", result.SourceCardID)
	}
}

// TestProhibitionCheckerNoBasicType verifies that cards without basic type are not affected.
func TestProhibitionCheckerNoBasicType(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-no-type",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:       "XQ22-1",
			DefinitionID: "XQ22",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
		},
	}

	checker := BuildProhibitionChecker(state)
	targetCategory := TargetCategory{
		BasicTypes: []string{""}, // Empty basic type
	}

	result := checker.Check(state, "P1", targetCategory)
	if result.Prohibited {
		t.Fatal("expected no prohibition for empty basic type")
	}
}

// TestProhibitionCheckerReturnsRuleInfo verifies that the result contains correct rule information.
func TestProhibitionCheckerReturnsRuleInfo(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-info",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:       "XQ22-ABC",
			DefinitionID: "XQ22",
			Name:         "州议员贝伦·希恩斯",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
		},
	}

	checker := BuildProhibitionChecker(state)
	targetCategory := TargetCategory{
		BasicTypes: []string{"事务"},
	}

	result := checker.Check(state, "P1", targetCategory)

	if !result.Prohibited {
		t.Fatal("expected prohibition")
	}

	if result.SourceCardID != "XQ22-ABC" {
		t.Fatalf("expected SourceCardID to be XQ22-ABC, got %s", result.SourceCardID)
	}

	if result.SourceCardName != "州议员贝伦·希恩斯" {
		t.Fatalf("expected SourceCardName to be 州议员贝伦·希恩斯, got %s", result.SourceCardName)
	}

	if result.MatchedRule == nil {
		t.Fatal("expected MatchedRule to be set")
	}

	if result.MatchedRule.SourceDefinitionID != "XQ22" {
		t.Fatalf("expected rule SourceDefinitionID to be XQ22, got %s", result.MatchedRule.SourceDefinitionID)
	}
}

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

func TestProhibitionCheckerTargetConditionAbilityKindsMismatchDoesNotProhibit(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-prohibition-condition-ability-mismatch",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:       "COND-1",
			DefinitionID: "COND",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
			ControllerID: "P1",
		},
	}

	checker := NewScopedProhibitionChecker([]ProhibitionRule{
		{
			SourceDefinitionID: "COND",
			SourceCondition: CardCondition{
				Zone:         CardZoneTable,
				Ready:        true,
				NotDestroyed: true,
			},
			Scope: ProhibitionScope{
				Kind: ProhibitionScopeAllPlayers,
			},
			TargetCategory: TargetCategory{
				BasicTypes: []string{"事务"},
				Condition: &TargetCondition{
					AbilityKinds: []string{"action"},
				},
			},
		},
	})

	result := checker.Check(state, "P2", TargetCategory{
		BasicTypes: []string{"事务"},
		Condition: &TargetCondition{
			AbilityKinds: []string{"trigger"},
		},
	})
	if result.Prohibited {
		t.Fatalf("expected no prohibition when abilityKinds mismatch, got source %q", result.SourceCardID)
	}
}

func TestProhibitionCheckerTargetConditionRegionAndAbilityMatchProhibits(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-prohibition-condition-region-ability-match",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:       "COND-1",
			DefinitionID: "COND",
			Zone:         CardZoneTable,
			Exhausted:    false,
			Destroyed:    false,
			ControllerID: "P1",
		},
	}

	checker := NewScopedProhibitionChecker([]ProhibitionRule{
		{
			SourceDefinitionID: "COND",
			SourceCondition: CardCondition{
				Zone:         CardZoneTable,
				Ready:        true,
				NotDestroyed: true,
			},
			Scope: ProhibitionScope{
				Kind: ProhibitionScopeAllPlayers,
			},
			TargetCategory: TargetCategory{
				BasicTypes: []string{"事务"},
				Condition: &TargetCondition{
					RegionID:     "region-a",
					AbilityKinds: []string{"action"},
				},
			},
		},
	})

	result := checker.Check(state, "P2", TargetCategory{
		BasicTypes: []string{"事务"},
		Condition: &TargetCondition{
			RegionID:     "region-a",
			AbilityKinds: []string{"action"},
		},
	})
	if !result.Prohibited {
		t.Fatal("expected prohibition when region/ability conditions both match")
	}
}
