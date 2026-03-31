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

// TestGoldenScenario_XQ31GrantsDefenseToPrestigeAllies verifies that XQ31 grants +1 defense to all allied prestige characters.
// Scenario: XQ31 本方声望角色 +1 防御力
func TestGoldenScenario_XQ31GrantsDefenseToPrestigeAllies(t *testing.T) {
	// Given: P1 has XQ31, a prestige ally, and a non-prestige ally on the table
	state := NewGameState(InitialStateConfig{
		GameID:         "golden-xq31-defense",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add XQ31 to P1's table (4 defense printed)
	xq31Card := CardState{
		CardID:          "XQ31-1",
		DefinitionID:    "XQ31",
		Name:            "莫兰大主教",
		Zone:            CardZoneTable,
		Exhausted:       false,
		Destroyed:       false,
		ControllerID:    "P1",
		PrintedKeywords: []string{"领袖", "公开", "声望"},
		PrintedStats:    CardNumericStats{Combat: 1, Defense: 4},
	}

	// Add prestige ally to P1's table (2 defense printed)
	prestigeAlly := CardState{
		CardID:          "PRESTIGE-ALLY-1",
		DefinitionID:    "ALLY",
		Name:            "声望盟友",
		Zone:            CardZoneTable,
		Exhausted:       false,
		Destroyed:       false,
		ControllerID:    "P1",
		PrintedKeywords: []string{"声望"},
		PrintedStats:    CardNumericStats{Combat: 1, Defense: 2},
	}

	// Add non-prestige ally to P1's table (2 defense printed)
	nonPrestigeAlly := CardState{
		CardID:          "NON-PRESTIGE-ALLY-1",
		DefinitionID:    "ALLY",
		Name:            "非声望盟友",
		Zone:            CardZoneTable,
		Exhausted:       false,
		Destroyed:       false,
		ControllerID:    "P1",
		PrintedKeywords: []string{},
		PrintedStats:    CardNumericStats{Combat: 1, Defense: 2},
	}

	// Add enemy prestige character to P2's table (2 defense printed)
	enemyPrestige := CardState{
		CardID:          "ENEMY-PRESTIGE-1",
		DefinitionID:    "ENEMY",
		Name:            "敌方声望",
		Zone:            CardZoneTable,
		Exhausted:       false,
		Destroyed:       false,
		ControllerID:    "P2",
		PrintedKeywords: []string{"声望"},
		PrintedStats:    CardNumericStats{Combat: 1, Defense: 2},
	}

	state.Board.Cards = []CardState{xq31Card, prestigeAlly, nonPrestigeAlly, enemyPrestige}

	// When: Recalculate continuous effects
	recalculated := RecalculateContinuousEffects(state)

	// Then: Verify defense values
	var xq31, prestigeAllyAfter, nonPrestigeAllyAfter, enemyPrestigeAfter CardState
	for _, card := range recalculated.Board.Cards {
		switch card.CardID {
		case "XQ31-1":
			xq31 = card
		case "PRESTIGE-ALLY-1":
			prestigeAllyAfter = card
		case "NON-PRESTIGE-ALLY-1":
			nonPrestigeAllyAfter = card
		case "ENEMY-PRESTIGE-1":
			enemyPrestigeAfter = card
		}
	}

	if xq31.EffectiveStats.Defense != 5 {
		t.Fatalf("XQ31 effective defense = %d, want 5 (4 + 1)", xq31.EffectiveStats.Defense)
	}
	if prestigeAllyAfter.EffectiveStats.Defense != 3 {
		t.Fatalf("prestige ally effective defense = %d, want 3 (2 + 1)", prestigeAllyAfter.EffectiveStats.Defense)
	}
	if nonPrestigeAllyAfter.EffectiveStats.Defense != 2 {
		t.Fatalf("non-prestige ally effective defense = %d, want 2 (no buff)", nonPrestigeAllyAfter.EffectiveStats.Defense)
	}
	if enemyPrestigeAfter.EffectiveStats.Defense != 2 {
		t.Fatalf("enemy prestige effective defense = %d, want 2 (no buff)", enemyPrestigeAfter.EffectiveStats.Defense)
	}
}

// TestGoldenScenario_XQ01SilencesAttackAndInvestigation verifies that XQ01 prevents all characters from attacking and investigating.
// Scenario: XQ01 沉默所有角色的攻击和调查
func TestGoldenScenario_XQ01SilencesAttackAndInvestigation(t *testing.T) {
	// Given: P1 has XQ01 ready on the table
	state := NewGameState(InitialStateConfig{
		GameID:         "golden-xq01",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add XQ01 to P1's table
	xq01Card := CardState{
		CardID:          "XQ01-1",
		DefinitionID:    "XQ01",
		Name:            "联会禁音使",
		Zone:            CardZoneTable,
		Exhausted:       false,
		Destroyed:       false,
		ControllerID:    "P1",
		PrintedKeywords: []string{"角色"},
		Kind:            CardKindCharacter,
	}

	// Add P1's character to the table
	allyCard := CardState{
		CardID:          "ALLY-1",
		DefinitionID:    "ALLY",
		Name:            "本方角色",
		Zone:            CardZoneTable,
		Exhausted:       false,
		Destroyed:       false,
		ControllerID:    "P1",
		PrintedKeywords: []string{"角色"},
		Kind:            CardKindCharacter,
	}

	// Add P2's character to the table
	enemyCard := CardState{
		CardID:          "ENEMY-1",
		DefinitionID:    "ENEMY",
		Name:            "敌方角色",
		Zone:            CardZoneTable,
		Exhausted:       false,
		Destroyed:       false,
		ControllerID:    "P2",
		PrintedKeywords: []string{"角色"},
		Kind:            CardKindCharacter,
	}

	state.Board.Cards = []CardState{xq01Card, allyCard, enemyCard}

	// Recalculate continuous effects to apply XQ01's silence
	state = RecalculateContinuousEffects(state)

	// Verify that all characters have attack and investigate prohibited
	var allyAfter CardState
	var enemyAfter CardState
	for _, card := range state.Board.Cards {
		switch card.CardID {
		case "ALLY-1":
			allyAfter = card
		case "ENEMY-1":
			enemyAfter = card
		}
	}

	if !containsString(allyAfter.Prohibitions, "attack") {
		t.Fatalf("ally prohibitions = %v, want contains \"attack\"", allyAfter.Prohibitions)
	}
	if !containsString(allyAfter.Prohibitions, "investigate") {
		t.Fatalf("ally prohibitions = %v, want contains \"investigate\"", allyAfter.Prohibitions)
	}
	if !containsString(enemyAfter.Prohibitions, "attack") {
		t.Fatalf("enemy prohibitions = %v, want contains \"attack\"", enemyAfter.Prohibitions)
	}
	if !containsString(enemyAfter.Prohibitions, "investigate") {
		t.Fatalf("enemy prohibitions = %v, want contains \"investigate\"", enemyAfter.Prohibitions)
	}
}
