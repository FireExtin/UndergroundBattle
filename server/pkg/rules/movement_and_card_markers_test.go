package rules

import (
	"errors"
	"testing"
)

func TestMoveCardAction_MovesBetweenAdjacentRegions(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "move-card-adjacent",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:      "region-1",
			Name:        "Region 1",
			Kind:        CardKindRegion,
			Zone:        CardZoneTable,
			Revealed:    true,
			RegionOrder: 1,
		},
		{
			CardID:      "region-2",
			Name:        "Region 2",
			Kind:        CardKindRegion,
			Zone:        CardZoneTable,
			Revealed:    true,
			RegionOrder: 2,
		},
		{
			CardID:      "region-3",
			Name:        "Region 3",
			Kind:        CardKindRegion,
			Zone:        CardZoneTable,
			Revealed:    true,
			RegionOrder: 3,
		},
		{
			CardID:         "p1-mover",
			Name:           "Mover",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			RegionCardID:   "region-1",
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Combat: 1, Defense: 1},
			EffectiveStats: CardNumericStats{Combat: 1, Defense: 1},
		},
	}

	result, err := SubmitAction(state, Action{
		ID:           "act-move-adjacent",
		ActorID:      "P1",
		Kind:         ActionKindMoveCard,
		CardID:       "p1-mover",
		TargetCardID: "region-2",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Event.Kind != EventKindCardMoved {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindCardMoved)
	}
	moved := cardStateByID(t, result.State, "p1-mover")
	if moved.RegionCardID != "region-2" {
		t.Fatalf("moved region = %q, want %q", moved.RegionCardID, "region-2")
	}
}

func TestMoveCardAction_RejectsNonAdjacentMove(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "move-card-non-adjacent",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{CardID: "region-1", Kind: CardKindRegion, Zone: CardZoneTable, Revealed: true, RegionOrder: 1},
		{CardID: "region-2", Kind: CardKindRegion, Zone: CardZoneTable, Revealed: true, RegionOrder: 2},
		{CardID: "region-3", Kind: CardKindRegion, Zone: CardZoneTable, Revealed: true, RegionOrder: 3},
		{
			CardID:         "p1-mover",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			RegionCardID:   "region-1",
			VisibleToOwner: true,
			Revealed:       true,
		},
	}

	_, err := SubmitAction(state, Action{
		ID:           "act-move-non-adjacent",
		ActorID:      "P1",
		Kind:         ActionKindMoveCard,
		CardID:       "p1-mover",
		TargetCardID: "region-3",
	})
	if err == nil {
		t.Fatal("expected non-adjacent move to be rejected")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}
	if legalityErr.Code != ReasonCodeTargetFailedProhibited {
		t.Fatalf("code = %q, want %q", legalityErr.Code, ReasonCodeTargetFailedProhibited)
	}
}

func TestCardMarkerRegistry_SetRemoveAndProjection(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "card-markers-registry",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:         "p1-table-card",
			Name:           "Marker Host",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Combat: 1, Defense: 2},
			EffectiveStats: CardNumericStats{Combat: 1, Defense: 2},
		},
	}

	state = mustSubmit(t, state, Action{
		ID:           "act-set-card-marker",
		ActorID:      "P1",
		Kind:         ActionKindSetCardMarker,
		TargetCardID: "p1-table-card",
		MarkerType:   "locked",
		MarkerAmount: 2,
	})
	if got := state.Board.CardMarkers.GetMarker("p1-table-card", "locked"); got != 2 {
		t.Fatalf("locked marker = %d, want 2", got)
	}

	state = mustSubmit(t, state, Action{
		ID:           "act-remove-card-marker",
		ActorID:      "P1",
		Kind:         ActionKindRemoveCardMarker,
		TargetCardID: "p1-table-card",
		MarkerType:   "locked",
		MarkerAmount: 1,
	})
	if got := state.Board.CardMarkers.GetMarker("p1-table-card", "locked"); got != 1 {
		t.Fatalf("locked marker = %d, want 1 after remove", got)
	}

	views := NewProjectionEngine().Generate(state)
	card := cardViewByID(t, views.Players["P1"].Board.Cards, "p1-table-card")
	if card.Markers["locked"] != 1 {
		t.Fatalf("projected card marker = %d, want 1", card.Markers["locked"])
	}
}

func TestCardMarkerRegistry_RejectsInvalidRequests(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "card-markers-invalid",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:         "p1-card-invalid",
			Name:           "Marker Host",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
	}

	_, err := SubmitAction(state, Action{
		ID:           "act-set-card-marker-missing-target",
		ActorID:      "P1",
		Kind:         ActionKindSetCardMarker,
		MarkerType:   "locked",
		MarkerAmount: 1,
	})
	if err == nil {
		t.Fatal("expected missing target card to be rejected")
	}

	state = mustSubmit(t, state, Action{
		ID:           "act-set-card-marker-valid",
		ActorID:      "P1",
		Kind:         ActionKindSetCardMarker,
		TargetCardID: "p1-card-invalid",
		MarkerType:   "locked",
		MarkerAmount: 1,
	})

	_, err = SubmitAction(state, Action{
		ID:           "act-remove-card-marker-too-much",
		ActorID:      "P1",
		Kind:         ActionKindRemoveCardMarker,
		TargetCardID: "p1-card-invalid",
		MarkerType:   "locked",
		MarkerAmount: 2,
	})
	if err == nil {
		t.Fatal("expected removing too many card markers to be rejected")
	}
}
