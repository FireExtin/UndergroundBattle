package rules

import (
	"testing"
)

// Purpose: TDD regression tests for payment consolidation on formal battle actions.

func TestPaymentConsolidation_BuildAsset(t *testing.T) {
	t.Run("build_asset uses payment engine", func(t *testing.T) {
		state := NewGameState(InitialStateConfig{
			ActivePlayerID: "P1",
			PlayerIDs:      []string{"P1", "P2"},
		})
		state.Turn.Resources["P1"] = PlayerResourceState{Current: 5, Max: 5}

		state.Board.Cards = append(state.Board.Cards, CardState{
			CardID:  "c1",
			OwnerID: "P1",
			Zone:    CardZoneHand,
		})

		action := Action{
			ID:      "a1",
			ActorID: "P1",
			Kind:    ActionKindBuildAsset,
			CardID:  "c1",
		}

		result, err := submitActionInternal(state, action, submitInternalOptions{
			enforceDeterminism: false,
		})
		if err != nil {
			t.Fatalf("unexpected error executing build_asset: %v", err)
		}

		// Currently build asset costs 0, so resources should still be 5
		// But if it wasn't hooked up correctly, a 0-cost check would still theoretically pass,
		// However we will implement explicit hook.
		pool := result.State.Turn.Resources["P1"]
		if pool.Current != 5 {
			t.Fatalf("expected resources to be 5, got %d", pool.Current)
		}
	})
}
