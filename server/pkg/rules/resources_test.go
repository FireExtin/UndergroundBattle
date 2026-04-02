package rules

import "testing"

// Purpose: Verifies turn-based resource initialization/refill for battle V4.

func TestNewGameStateInitializesActivePlayerResources(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-resource-init",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           20260402,
	})

	p1 := state.Turn.Resources["P1"]
	if p1.Current != 1 || p1.Max != 1 {
		t.Fatalf("P1 resource = %#v, want current=1 max=1", p1)
	}
	p2 := state.Turn.Resources["P2"]
	if p2.Current != 0 || p2.Max != 0 {
		t.Fatalf("P2 resource = %#v, want current=0 max=0", p2)
	}
}

func TestAdvancePhaseEndToMainRefillsNextActivePlayerResources(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-resource-refill",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           20260402,
	})
	state.Turn.Phase = phaseState(PhaseEnd)
	state.Turn.Priority.CurrentPlayerID = "P1"
	state.Turn.PriorityPlayerID = "P1"
	state.Turn.Resources["P1"] = PlayerResourceState{Current: 0, Max: 1}
	state.Turn.Resources["P2"] = PlayerResourceState{Current: 0, Max: 0}

	result, err := SubmitAction(state, Action{
		ID:      "act-resource-advance",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.State.Turn.ActivePlayerID != "P2" {
		t.Fatalf("active player = %q, want P2", result.State.Turn.ActivePlayerID)
	}
	if result.State.Turn.TurnNumber != 2 {
		t.Fatalf("turn number = %d, want 2", result.State.Turn.TurnNumber)
	}
	p2 := result.State.Turn.Resources["P2"]
	if p2.Current != 2 || p2.Max != 2 {
		t.Fatalf("P2 resource = %#v, want current=2 max=2", p2)
	}
}
