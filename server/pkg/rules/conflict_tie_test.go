package rules

import "testing"

// Tests for PN-ACT-005: tie + first-player-privilege integrated into conflict flow.

func TestTieWithFirstPlayerPrivilegeWins(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-conflict-tie-priv",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           42,
	})

	state.Board.Cards = []CardState{
		{
			CardID:            "region-1",
			Name:              "Region One",
			Kind:              CardKindRegion,
			OwnerID:           "TABLE",
			Zone:              CardZoneTable,
			Revealed:          true,
			RegionOrder:       1,
			InfluenceByPlayer: map[string]int{"P1": 2, "P2": 2},
		},
	}

	state.Turn.Conflict = ConflictState{
		RegionOrder:               1,
		RegionCardID:              "region-1",
		FirstPlayerPrivilegeOwner: "P1",
	}

	winner, err := DetermineRegionConflictWinner(state, "region-1")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if winner != "P1" {
		t.Fatalf("winner = %q, want %q (privilege owner)", winner, "P1")
	}
}

func TestTieFallsBackToPriorityLeaderWhenNoPrivilege(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-conflict-tie-priority",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           43,
	})

	state.Board.Cards = []CardState{
		{
			CardID:            "region-1",
			Name:              "Region One",
			Kind:              CardKindRegion,
			OwnerID:           "TABLE",
			Zone:              CardZoneTable,
			Revealed:          true,
			RegionOrder:       1,
			InfluenceByPlayer: map[string]int{"P1": 3, "P2": 3},
		},
	}

	state.Turn.Conflict = ConflictState{
		RegionOrder:            1,
		RegionCardID:           "region-1",
		PriorityLeaderPlayerID: "P2",
	}

	winner, err := DetermineRegionConflictWinner(state, "region-1")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if winner != "P2" {
		t.Fatalf("winner = %q, want %q (priority leader)", winner, "P2")
	}
}

func TestTieWithNoTiebreakerReturnsNoWinner(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-conflict-tie-none",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           44,
	})

	state.Board.Cards = []CardState{
		{
			CardID:            "region-1",
			Name:              "Region One",
			Kind:              CardKindRegion,
			OwnerID:           "TABLE",
			Zone:              CardZoneTable,
			Revealed:          true,
			RegionOrder:       1,
			InfluenceByPlayer: map[string]int{"P1": 1, "P2": 1},
		},
	}

	state.Turn.Conflict = ConflictState{
		RegionOrder:  1,
		RegionCardID: "region-1",
	}

	winner, err := DetermineRegionConflictWinner(state, "region-1")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if winner != "" {
		t.Fatalf("winner = %q, want empty (no tiebreaker)", winner)
	}
}
