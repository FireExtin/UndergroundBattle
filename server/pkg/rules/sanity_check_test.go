package rules

import (
	"testing"
)

// TestSanityCheck_AllInvariantsPass verifies that all invariants pass on a valid state.
func TestSanityCheck_AllInvariantsPass(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "sanity-invariants",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add some valid cards
	state.Board.Cards = []CardState{
		{CardID: "card-1", Zone: CardZoneTable, ControllerID: "P1"},
		{CardID: "card-2", Zone: CardZoneHand, ControllerID: "P2"},
		{CardID: "card-3", Zone: CardZoneDeck, ControllerID: "P1"},
	}

	results := CheckAllInvariants(state, DefaultInvariantConfig)

	for _, result := range results {
		if !result.Passed {
			t.Fatalf("invariant %s failed: %s", result.Name, result.Message)
		}
	}
}

// TestSanityCheck_AllGoldenScenariosPass runs all golden scenarios.
func TestSanityCheck_AllGoldenScenariosPass(t *testing.T) {
	// This test serves as a meta-test that runs all golden scenarios
	// Individual scenarios are tested in golden_scenario_test.go
	// This ensures they all pass together

	scenarios := []struct {
		name string
		fn   func(*testing.T)
	}{
		{"XQ22BlocksEventCard", TestGoldenScenario_XQ22BlocksEventCard},
		{"XQ31ProtectsPrestigeAlly", TestGoldenScenario_XQ31ProtectsPrestigeAlly},
		{"FullGameTurn", TestGoldenScenario_FullGameTurn},
		{"XQ22AllowsNonEventCards", TestGoldenScenario_XQ22AllowsNonEventCards},
		{"XQ31AllowsAllyToTargetPrestige", TestGoldenScenario_XQ31AllowsAllyToTargetPrestige},
		{"RevisionConsistency", TestGoldenScenario_RevisionConsistency},
		{"InvariantsAfterActions", TestGoldenScenario_InvariantsAfterActions},
	}

	for _, scenario := range scenarios {
		t.Run(scenario.name, scenario.fn)
	}
}

// TestSanityCheck_AllReplayTestsPass runs all replay tests.
func TestSanityCheck_AllReplayTestsPass(t *testing.T) {
	// This test serves as a meta-test that runs all replay tests
	// Individual tests are in replay_test.go

	tests := []struct {
		name string
		fn   func(*testing.T)
	}{
		{"SimpleSequence", TestReplaySimpleSequence},
		{"Determinism", TestReplayDeterminism},
		{"WithInvariants", TestReplayWithInvariants},
		{"EmptyActions", TestReplayEmptyActions},
		{"WithCards", TestReplayWithCards},
	}

	for _, test := range tests {
		t.Run(test.name, test.fn)
	}
}

// TestSanityCheck_ProhibitionFramework verifies the prohibition framework works.
func TestSanityCheck_ProhibitionFramework(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "sanity-prohibition",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add XQ22
	state.Board.Cards = []CardState{
		{
			CardID:         "XQ22-1",
			DefinitionID:   "XQ22",
			Name:           "州议员贝伦·希恩斯",
			Zone:           CardZoneTable,
			Exhausted:      false,
			Destroyed:      false,
			ControllerID:   "P1",
			PrintedKeywords: []string{"角色"},
		},
	}

	checker := BuildProhibitionChecker(state)

	// Event cards should be prohibited
	eventResult := checker.Check(state, "P2", TargetCategory{BasicTypes: []string{"事务"}})
	if !eventResult.Prohibited {
		t.Fatal("event cards should be prohibited by XQ22")
	}

	// Character cards should not be prohibited
	charResult := checker.Check(state, "P2", TargetCategory{BasicTypes: []string{"角色"}})
	if charResult.Prohibited {
		t.Fatal("character cards should NOT be prohibited by XQ22")
	}
}

// TestSanityCheck_TargetLegalityFramework verifies the target legality framework works.
func TestSanityCheck_TargetLegalityFramework(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "sanity-target-legality",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add XQ31 and prestige ally
	state.Board.Cards = []CardState{
		{
			CardID:         "XQ31-1",
			DefinitionID:   "XQ31",
			Name:           "莫兰大主教",
			Zone:           CardZoneTable,
			Exhausted:      false,
			Destroyed:      false,
			ControllerID:   "P1",
			PrintedKeywords: []string{"领袖", "公开", "声望"},
		},
		{
			CardID:         "ALLY-1",
			DefinitionID:   "ALLY",
			Name:           "声望盟友",
			Zone:           CardZoneTable,
			Exhausted:      false,
			Destroyed:      false,
			ControllerID:   "P1",
			PrintedKeywords: []string{"声望"},
		},
	}

	checker := BuildTargetLegalityChecker(state)

	// Enemy should not be able to target prestige ally
	enemyResult := checker.CheckTargetCard(state, "P2", "ALLY-1")
	if enemyResult.CanTarget {
		t.Fatal("enemy should NOT be able to target prestige ally protected by XQ31")
	}

	// Ally should be able to target own prestige ally
	allyResult := checker.CheckTargetCard(state, "P1", "ALLY-1")
	if !allyResult.CanTarget {
		t.Fatal("ally should be able to target own prestige ally")
	}
}

// TestSanityCheck_ActionSubmission verifies that actions can be submitted.
func TestSanityCheck_ActionSubmission(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "sanity-actions",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	initialRevision := state.Revision.Number

	// Submit a pass priority action
	action := Action{
		ID:      "action-1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	}

	result, err := SubmitAction(state, action)
	if err != nil {
		t.Fatalf("action submission failed: %v", err)
	}

	// Verify revision incremented
	if result.State.Revision.Number != initialRevision+1 {
		t.Fatalf("revision should increment: expected %d, got %d",
			initialRevision+1, result.State.Revision.Number)
	}

	// Verify invariants still pass
	invariantResults := CheckAllInvariants(result.State, DefaultInvariantConfig)
	for _, invResult := range invariantResults {
		if !invResult.Passed {
			t.Fatalf("after action: invariant %s failed: %s", invResult.Name, invResult.Message)
		}
	}
}

// TestSanityCheck_ReplaySystem verifies the replay system works end-to-end.
func TestSanityCheck_ReplaySystem(t *testing.T) {
	initialState := NewGameState(InitialStateConfig{
		GameID:         "sanity-replay",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	actions := []Action{
		{ID: "a1", ActorID: "P1", Kind: ActionKindPassPriority},
		{ID: "a2", ActorID: "P2", Kind: ActionKindPassPriority},
	}

	// Execute actions
	state := initialState
	for _, action := range actions {
		result, err := SubmitAction(state, action)
		if err != nil {
			t.Fatalf("action %s failed: %v", action.ID, err)
		}
		state = result.State
	}
	finalState := state

	// Replay
	replayedState, err := ReplayActions(initialState, actions)
	if err != nil {
		t.Fatalf("replay failed: %v", err)
	}

	// Verify replay matches
	if replayedState.Revision.Number != finalState.Revision.Number {
		t.Fatalf("replay mismatch: revision %d != %d",
			replayedState.Revision.Number, finalState.Revision.Number)
	}
}
