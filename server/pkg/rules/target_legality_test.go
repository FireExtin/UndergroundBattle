package rules

import (
	"testing"
)

// TestTargetLegalityXQ31RestrictsEnemyTargets verifies that XQ31 prevents enemy actors from targeting prestige allies.
// RED TEST: This test will fail initially because we haven't implemented TargetLegalityChecker yet.
func TestTargetLegalityXQ31RestrictsEnemyTargets(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-target-legality",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add XQ31 on P1's side
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

	// Add a prestige ally (should be protected by XQ31)
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

	// Add a non-prestige ally (should NOT be protected)
	nonPrestigeAlly := CardState{
		CardID:         "ALLY-2",
		DefinitionID:   "ALLY2",
		Name:           "普通盟友",
		Zone:           CardZoneTable,
		Exhausted:      false,
		Destroyed:      false,
		ControllerID:   "P1",
		PrintedKeywords: []string{},
	}

	state.Board.Cards = []CardState{xq31Card, prestigeAlly, nonPrestigeAlly}

	// Build the target legality checker with XQ31 rule
	xq31Rule := TargetLegalityRule{
		SourceDefinitionID: "XQ31",
		SourceCondition: CardCondition{
			Zone:         CardZoneTable,
			Ready:        true,
			NotDestroyed: true,
		},
		AffectedTargetCondition: TargetCondition{
			Keywords: []string{"声望"},
			Side:     SideAlly,
		},
		ActorRestriction: ProhibitionScope{
			Kind: ProhibitionScopeOpponentsOnly,
		},
		Description: "XQ31: Enemies can't target prestige allies",
	}

	checker := NewTargetLegalityChecker([]TargetLegalityRule{xq31Rule})

	// Test 1: Enemy (P2) tries to target prestige ally - SHOULD BE RESTRICTED
	result1 := checker.CheckTargetCard(state, "P2", "ALLY-1")
	if result1.CanTarget {
		t.Fatal("expected P2 (enemy) to NOT be able to target prestige ally ALLY-1")
	}
	if result1.SourceCardID != "XQ31-1" {
		t.Fatalf("expected source to be XQ31-1, got %s", result1.SourceCardID)
	}

	// Test 2: Enemy (P2) tries to target non-prestige ally - SHOULD BE ALLOWED
	result2 := checker.CheckTargetCard(state, "P2", "ALLY-2")
	if !result2.CanTarget {
		t.Fatal("expected P2 (enemy) to be able to target non-prestige ally ALLY-2")
	}

	// Test 3: Ally (P1) tries to target prestige ally - SHOULD BE ALLOWED
	result3 := checker.CheckTargetCard(state, "P1", "ALLY-1")
	if !result3.CanTarget {
		t.Fatal("expected P1 (ally) to be able to target prestige ally ALLY-1")
	}

	// Test 4: Ally (P1) tries to target non-prestige ally - SHOULD BE ALLOWED
	result4 := checker.CheckTargetCard(state, "P1", "ALLY-2")
	if !result4.CanTarget {
		t.Fatal("expected P1 (ally) to be able to target non-prestige ally ALLY-2")
	}
}

// TestTargetLegalityXQ31InactiveWhenExhausted verifies that XQ31 doesn't protect when exhausted.
func TestTargetLegalityXQ31InactiveWhenExhausted(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-xq31-exhausted",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add EXHAUSTED XQ31 on P1's side
	xq31Card := CardState{
		CardID:         "XQ31-1",
		DefinitionID:   "XQ31",
		Name:           "莫兰大主教",
		Zone:           CardZoneTable,
		Exhausted:      true, // Exhausted!
		Destroyed:      false,
		ControllerID:   "P1",
		PrintedKeywords: []string{"领袖", "公开", "声望"},
	}

	// Add a prestige ally
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

	checker := BuildTargetLegalityChecker(state)

	// Enemy (P2) should be able to target prestige ally because XQ31 is exhausted
	result := checker.CheckTargetCard(state, "P2", "ALLY-1")
	if !result.CanTarget {
		t.Fatal("expected P2 to be able to target prestige ally when XQ31 is exhausted")
	}
}

// TestTargetLegalityXQ31InactiveWhenDestroyed verifies that XQ31 doesn't protect when destroyed.
func TestTargetLegalityXQ31InactiveWhenDestroyed(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-xq31-destroyed",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add DESTROYED XQ31 on P1's side
	xq31Card := CardState{
		CardID:         "XQ31-1",
		DefinitionID:   "XQ31",
		Name:           "莫兰大主教",
		Zone:           CardZoneTable,
		Exhausted:      false,
		Destroyed:      true, // Destroyed!
		ControllerID:   "P1",
		PrintedKeywords: []string{"领袖", "公开", "声望"},
	}

	// Add a prestige ally
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

	checker := BuildTargetLegalityChecker(state)

	// Enemy (P2) should be able to target prestige ally because XQ31 is destroyed
	result := checker.CheckTargetCard(state, "P2", "ALLY-1")
	if !result.CanTarget {
		t.Fatal("expected P2 to be able to target prestige ally when XQ31 is destroyed")
	}
}

// TestTargetLegalityXQ31ProtectsOnlyAllyPrestige verifies that XQ31 only protects ally prestige, not enemy prestige.
func TestTargetLegalityXQ31ProtectsOnlyAllyPrestige(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-xq31-enemy-prestige",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add XQ31 on P1's side
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

	// Add P1's prestige ally (should be protected)
	allyPrestige := CardState{
		CardID:         "ALLY-1",
		DefinitionID:   "ALLY",
		Name:           "本方声望盟友",
		Zone:           CardZoneTable,
		Exhausted:      false,
		Destroyed:      false,
		ControllerID:   "P1",
		PrintedKeywords: []string{"声望"},
	}

	// Add P2's prestige character (should NOT be protected by P1's XQ31)
	enemyPrestige := CardState{
		CardID:         "ENEMY-1",
		DefinitionID:   "ENEMY",
		Name:           "敌方声望角色",
		Zone:           CardZoneTable,
		Exhausted:      false,
		Destroyed:      false,
		ControllerID:   "P2",
		PrintedKeywords: []string{"声望"},
	}

	state.Board.Cards = []CardState{xq31Card, allyPrestige, enemyPrestige}

	checker := BuildTargetLegalityChecker(state)

	// P1 should NOT be able to target ally prestige (protected by XQ31)
	result1 := checker.CheckTargetCard(state, "P1", "ALLY-1")
	if !result1.CanTarget {
		t.Fatal("expected P1 to be able to target own prestige ally (P1 is not enemy of P1)")
	}

	// P1 should be able to target enemy prestige (not protected by XQ31)
	result2 := checker.CheckTargetCard(state, "P1", "ENEMY-1")
	if !result2.CanTarget {
		t.Fatal("expected P1 to be able to target enemy prestige (not protected by XQ31)")
	}

	// P2 should NOT be able to target ally prestige (protected by XQ31)
	result3 := checker.CheckTargetCard(state, "P2", "ALLY-1")
	if result3.CanTarget {
		t.Fatal("expected P2 to NOT be able to target P1's prestige ally")
	}

	// P2 should be able to target own prestige (not protected by XQ31)
	result4 := checker.CheckTargetCard(state, "P2", "ENEMY-1")
	if !result4.CanTarget {
		t.Fatal("expected P2 to be able to target own prestige (not protected by own XQ31)")
	}
}

// TestTargetLegalityEmptyStateAllowsAll verifies that empty state allows all targeting.
func TestTargetLegalityEmptyStateAllowsAll(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-empty-state",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// No XQ31 on the table
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

	state.Board.Cards = []CardState{prestigeAlly}

	checker := BuildTargetLegalityChecker(state)

	// Enemy (P2) should be able to target prestige ally because no XQ31
	result := checker.CheckTargetCard(state, "P2", "ALLY-1")
	if !result.CanTarget {
		t.Fatal("expected P2 to be able to target prestige ally when no XQ31 present")
	}
}

// TestTargetLegalityNonExistentTarget verifies behavior when target doesn't exist.
func TestTargetLegalityNonExistentTarget(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-non-existent",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add XQ31 on P1's side
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

	state.Board.Cards = []CardState{xq31Card}

	checker := BuildTargetLegalityChecker(state)

	// Try to target a non-existent card - should default to allowing (or handle error gracefully)
	result := checker.CheckTargetCard(state, "P2", "NON-EXISTENT")
	if !result.CanTarget {
		t.Fatal("expected to allow targeting non-existent card (default behavior)")
	}
}
