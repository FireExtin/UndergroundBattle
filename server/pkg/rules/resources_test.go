package rules

import "testing"

// Purpose: Verifies turn-based resource initialization/refill for battle V4.

func TestCurrentPaymentEngineUsesPrototypeMode(t *testing.T) {
	engine := CurrentPaymentEngine()
	if engine == nil {
		t.Fatal("CurrentPaymentEngine() returned nil")
	}
	if engine.Mode() != PaymentModePrototype {
		t.Fatalf("payment mode = %q, want %q", engine.Mode(), PaymentModePrototype)
	}
}

func TestNewGameStateInitializesBothPlayersResources(t *testing.T) {
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
	if p2.Current != 1 || p2.Max != 1 {
		t.Fatalf("P2 resource = %#v, want current=1 max=1", p2)
	}
	if CurrentPaymentMode() != PaymentModePrototype {
		t.Fatalf("CurrentPaymentMode() = %q, want %q", CurrentPaymentMode(), PaymentModePrototype)
	}
}

func TestAdvancePhaseEndToMainRefillsBothPlayersResources(t *testing.T) {
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
	p1 := result.State.Turn.Resources["P1"]
	if p1.Current != 2 || p1.Max != 2 {
		t.Fatalf("P1 resource = %#v, want current=2 max=2", p1)
	}
}

func TestPaymentEnginePayCostRejectsWhenPoolIsInsufficient(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-payment-engine-pay",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           20260402,
	})
	state.Turn.Resources["P1"] = PlayerResourceState{Current: 1, Max: 1}

	engine := CurrentPaymentEngine()
	if engine.PayCost(&state, "P1", 2) {
		t.Fatal("PayCost succeeded, want insufficient-pool rejection")
	}

	pool := engine.ResourceView(state, "P1")
	if pool.Current != 1 || pool.Max != 1 {
		t.Fatalf("pool after failed payment = %#v, want unchanged current=1 max=1", pool)
	}
}
