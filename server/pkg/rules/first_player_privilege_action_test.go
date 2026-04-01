package rules

import (
	"errors"
	"testing"
)

func TestFirstPlayerPrivilegeAction_ConsumesOnNonZeroTie(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "first-player-privilege-consume",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:            "region-privilege",
			Name:              "Region",
			Kind:              CardKindRegion,
			Zone:              CardZoneTable,
			Revealed:          true,
			InfluenceByPlayer: map[string]int{"P1": 2, "P2": 2},
		},
	}

	result, err := SubmitAction(state, Action{
		ID:      "act-first-player-privilege",
		ActorID: "P1",
		Kind:    ActionKindUseFirstPlayerPrivilege,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Event.Kind != EventKindFirstPlayerPrivilegeUsed {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindFirstPlayerPrivilegeUsed)
	}
	region := cardStateByID(t, result.State, "region-privilege")
	if region.ControllerID != "P1" {
		t.Fatalf("region controller = %q, want %q", region.ControllerID, "P1")
	}
	if got := result.State.Board.Markers.GetMarker("P1", markerTypeFirstPlayerPrivilegeUsed); got != 1 {
		t.Fatalf("used marker = %d, want 1", got)
	}
	if got := result.State.Board.Markers.GetMarker("P1", markerTypeFirstPlayerPrivilegeRequest); got != 0 {
		t.Fatalf("request marker = %d, want 0 after consume", got)
	}
	if !result.State.Turn.FirstPlayerPrivilegeUsed {
		t.Fatal("turn.firstPlayerPrivilegeUsed should be true after action resolves")
	}
}

func TestFirstPlayerPrivilegeAction_RejectsWhenNoBreakableTie(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "first-player-privilege-no-tie",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:            "region-no-tie",
			Name:              "Region",
			Kind:              CardKindRegion,
			Zone:              CardZoneTable,
			Revealed:          true,
			InfluenceByPlayer: map[string]int{"P1": 3, "P2": 1},
		},
	}

	_, err := SubmitAction(state, Action{
		ID:      "act-first-player-privilege-no-tie",
		ActorID: "P1",
		Kind:    ActionKindUseFirstPlayerPrivilege,
	})
	if err == nil {
		t.Fatal("expected first-player privilege without tie to be rejected")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}
	if legalityErr.Code != ReasonCodeLegalityFailedActionProhibited {
		t.Fatalf("code = %q, want %q", legalityErr.Code, ReasonCodeLegalityFailedActionProhibited)
	}
}

func TestFirstPlayerPrivilegeAction_RejectsSecondUseSameTurn(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "first-player-privilege-twice",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:            "region-tie-twice",
			Name:              "Region",
			Kind:              CardKindRegion,
			Zone:              CardZoneTable,
			Revealed:          true,
			InfluenceByPlayer: map[string]int{"P1": 2, "P2": 2},
		},
	}

	state = mustSubmit(t, state, Action{
		ID:      "act-first-player-privilege-first",
		ActorID: "P1",
		Kind:    ActionKindUseFirstPlayerPrivilege,
	})
	if !state.Turn.FirstPlayerPrivilegeUsed {
		t.Fatal("first use should mark turn.firstPlayerPrivilegeUsed=true")
	}

	_, err := SubmitAction(state, Action{
		ID:      "act-first-player-privilege-second",
		ActorID: "P1",
		Kind:    ActionKindUseFirstPlayerPrivilege,
	})
	if err == nil {
		t.Fatal("expected second privilege use in same turn to be rejected")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}
	if legalityErr.Code != ReasonCodeLegalityFailedActionProhibited {
		t.Fatalf("code = %q, want %q", legalityErr.Code, ReasonCodeLegalityFailedActionProhibited)
	}
}
