package rules

import (
	"testing"
)

// TestReplaySimpleSequence verifies that ReplayActions correctly replays a simple action sequence.
func TestReplaySimpleSequence(t *testing.T) {
	// Given: Initial state
	initialState := NewGameState(InitialStateConfig{
		GameID:         "replay-test",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Execute actions and record them
	actions := []Action{
		{ID: "a1", ActorID: "P1", Kind: ActionKindPassPriority},
		{ID: "a2", ActorID: "P2", Kind: ActionKindPassPriority},
	}

	state := initialState
	for _, action := range actions {
		result, err := SubmitAction(state, action)
		if err != nil {
			t.Fatalf("action %s failed: %v", action.ID, err)
		}
		state = result.State
	}

	finalState := state

	// When: Replay the same actions
	replayedState, err := ReplayActions(initialState, actions)
	if err != nil {
		t.Fatalf("replay failed: %v", err)
	}

	// Then: Verify replayed state matches final state
	if replayedState.Revision.Number != finalState.Revision.Number {
		t.Fatalf("revision mismatch: expected %d, got %d",
			finalState.Revision.Number, replayedState.Revision.Number)
	}

	// Verify priority player is the same
	if replayedState.Turn.Priority.CurrentPlayerID != finalState.Turn.Priority.CurrentPlayerID {
		t.Fatalf("priority player mismatch: expected %s, got %s",
			finalState.Turn.Priority.CurrentPlayerID,
			replayedState.Turn.Priority.CurrentPlayerID)
	}
}

// TestReplayDeterminism verifies that replay produces deterministic results.
func TestReplayDeterminism(t *testing.T) {
	// Given: Initial state and actions
	initialState := NewGameState(InitialStateConfig{
		GameID:         "replay-determinism",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	actions := []Action{
		{ID: "a1", ActorID: "P1", Kind: ActionKindPassPriority},
		{ID: "a2", ActorID: "P2", Kind: ActionKindPassPriority},
	}

	// When: Replay multiple times
	replayed1, err := ReplayActions(initialState, actions)
	if err != nil {
		t.Fatalf("first replay failed: %v", err)
	}

	replayed2, err := ReplayActions(initialState, actions)
	if err != nil {
		t.Fatalf("second replay failed: %v", err)
	}

	// Then: Both replays should produce identical results
	if replayed1.Revision.Number != replayed2.Revision.Number {
		t.Fatalf("determinism failed: revision %d != %d",
			replayed1.Revision.Number, replayed2.Revision.Number)
	}

	if replayed1.Turn.Priority.CurrentPlayerID != replayed2.Turn.Priority.CurrentPlayerID {
		t.Fatalf("determinism failed: priority player %s != %s",
			replayed1.Turn.Priority.CurrentPlayerID,
			replayed2.Turn.Priority.CurrentPlayerID)
	}
}

// TestReplayWithInvariants verifies that invariants hold during replay.
func TestReplayWithInvariants(t *testing.T) {
	// Given: Initial state with some cards
	initialState := NewGameState(InitialStateConfig{
		GameID:         "replay-invariants",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	initialState.Board.Cards = []CardState{
		{CardID: "card-1", Zone: CardZoneTable, ControllerID: "P1"},
		{CardID: "card-2", Zone: CardZoneHand, ControllerID: "P2"},
	}

	actions := []Action{
		{ID: "a1", ActorID: "P1", Kind: ActionKindPassPriority},
		{ID: "a2", ActorID: "P2", Kind: ActionKindPassPriority},
	}

	// When: Replay with invariant checking
	state := cloneGameState(initialState)
	for i, action := range actions {
		result, err := submitActionWithoutProjection(state, action)
		if err != nil {
			t.Fatalf("action %s failed: %v", action.ID, err)
		}
		state = result.State

		// Check invariants after each action
		invariantResults := CheckAllInvariants(state, DefaultInvariantConfig)
		for _, invResult := range invariantResults {
			if !invResult.Passed {
				t.Fatalf("after action %d (%s): invariant %s failed: %s",
					i, action.ID, invResult.Name, invResult.Message)
			}
		}
	}

	// Then: Final state should be valid
	finalResults := CheckAllInvariants(state, DefaultInvariantConfig)
	for _, result := range finalResults {
		if !result.Passed {
			t.Fatalf("final invariant %s failed: %s", result.Name, result.Message)
		}
	}
}

// TestReplayEmptyActions verifies that replay with empty actions returns initial state.
func TestReplayEmptyActions(t *testing.T) {
	initialState := NewGameState(InitialStateConfig{
		GameID:         "replay-empty",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// When: Replay with no actions
	replayedState, err := ReplayActions(initialState, []Action{})
	if err != nil {
		t.Fatalf("replay with empty actions failed: %v", err)
	}

	// Then: State should be identical to initial
	if replayedState.Revision.Number != initialState.Revision.Number {
		t.Fatalf("empty replay should preserve revision: expected %d, got %d",
			initialState.Revision.Number, replayedState.Revision.Number)
	}
}

// TestReplayWithCards verifies replay with card state changes.
func TestReplayWithCards(t *testing.T) {
	initialState := NewGameState(InitialStateConfig{
		GameID:         "replay-cards",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	// Add cards to track
	initialState.Board.Cards = []CardState{
		{CardID: "card-1", Zone: CardZoneTable, ControllerID: "P1", Exhausted: false},
		{CardID: "card-2", Zone: CardZoneTable, ControllerID: "P2", Exhausted: false},
	}

	actions := []Action{
		{ID: "a1", ActorID: "P1", Kind: ActionKindPassPriority},
		{ID: "a2", ActorID: "P2", Kind: ActionKindPassPriority},
	}

	// Execute and record final state
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

	// Verify card count is the same
	if len(replayedState.Board.Cards) != len(finalState.Board.Cards) {
		t.Fatalf("card count mismatch: expected %d, got %d",
			len(finalState.Board.Cards), len(replayedState.Board.Cards))
	}

	// Verify revision matches
	if replayedState.Revision.Number != finalState.Revision.Number {
		t.Fatalf("revision mismatch: expected %d, got %d",
			finalState.Revision.Number, replayedState.Revision.Number)
	}
}
