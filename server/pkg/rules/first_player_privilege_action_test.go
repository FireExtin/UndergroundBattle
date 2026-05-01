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
	state.Board.Markers.SetMarker("P1", markerTypeResource, 2)

	result, err := SubmitAction(state, Action{
		ID:      "act-first-player-privilege",
		ActorID: "P1",
		Kind:    ActionKindUseFirstPlayerPrivilege,
		Choices: []ChoiceRecord{buildFirstPlayerPrivilegePaymentChoice("P1")},
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
	if got := result.State.Board.Markers.GetMarker("P1", markerTypeResource); got != 1 {
		t.Fatalf("resource marker = %d, want 1 after paying cost", got)
	}
	if result.Operation.Payment == nil {
		t.Fatal("operation payment should be recorded")
	}
	if result.Operation.Payment.MarkerType != markerTypeResource || result.Operation.Payment.Amount != 1 {
		t.Fatalf("operation payment = %#v, want markerType=%q amount=1", result.Operation.Payment, markerTypeResource)
	}
	if result.Event.Payment == nil {
		t.Fatal("event payment should be recorded")
	}
	if result.Event.Payment.MarkerType != markerTypeResource || result.Event.Payment.Amount != 1 {
		t.Fatalf("event payment = %#v, want markerType=%q amount=1", result.Event.Payment, markerTypeResource)
	}
	if len(result.Operation.Choices) != 1 || result.Operation.Choices[0].Kind != firstPlayerPrivilegePaymentChoiceKind {
		t.Fatalf("operation choices = %#v, want first-player privilege payment choice", result.Operation.Choices)
	}
	if len(result.Event.Choices) != 1 || result.Event.Choices[0].Kind != firstPlayerPrivilegePaymentChoiceKind {
		t.Fatalf("event choices = %#v, want first-player privilege payment choice", result.Event.Choices)
	}
	if len(result.State.History.Actions) != 1 || len(result.State.History.Actions[0].Choices) != 1 {
		t.Fatalf("history action choices = %#v, want committed explicit choice", result.State.History.Actions)
	}
	if got := result.Views.Players["P1"].Markers[markerTypeResource]; got != 1 {
		t.Fatalf("projected resource marker = %d, want 1", got)
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
		Choices: []ChoiceRecord{buildFirstPlayerPrivilegePaymentChoice("P1")},
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

func TestFirstPlayerPrivilegeAction_RejectsWhenChoiceMissing(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "first-player-privilege-choice-missing",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:            "region-privilege-choice-missing",
			Name:              "Region",
			Kind:              CardKindRegion,
			Zone:              CardZoneTable,
			Revealed:          true,
			InfluenceByPlayer: map[string]int{"P1": 2, "P2": 2},
		},
	}
	state.Board.Markers.SetMarker("P1", markerTypeResource, 1)

	_, err := SubmitAction(state, Action{
		ID:      "act-first-player-privilege-choice-missing",
		ActorID: "P1",
		Kind:    ActionKindUseFirstPlayerPrivilege,
	})
	if err == nil {
		t.Fatal("expected first-player privilege without explicit payment choice to be rejected")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}
	if legalityErr.Code != ReasonCodeCostFailedUnpaid {
		t.Fatalf("code = %q, want %q", legalityErr.Code, ReasonCodeCostFailedUnpaid)
	}
}

func TestFirstPlayerPrivilegeAction_RejectsWhenCostUnpaid(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "first-player-privilege-unpaid",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		{
			CardID:            "region-privilege-unpaid",
			Name:              "Region",
			Kind:              CardKindRegion,
			Zone:              CardZoneTable,
			Revealed:          true,
			InfluenceByPlayer: map[string]int{"P1": 2, "P2": 2},
		},
	}

	_, err := SubmitAction(state, Action{
		ID:      "act-first-player-privilege-unpaid",
		ActorID: "P1",
		Kind:    ActionKindUseFirstPlayerPrivilege,
		Choices: []ChoiceRecord{buildFirstPlayerPrivilegePaymentChoice("P1")},
	})
	if err == nil {
		t.Fatal("expected first-player privilege without enough cost resources to be rejected")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}
	if legalityErr.Code != ReasonCodeCostFailedUnpaid {
		t.Fatalf("code = %q, want %q", legalityErr.Code, ReasonCodeCostFailedUnpaid)
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
	state.Board.Markers.SetMarker("P1", markerTypeResource, 2)

	state = mustSubmit(t, state, Action{
		ID:      "act-first-player-privilege-first",
		ActorID: "P1",
		Kind:    ActionKindUseFirstPlayerPrivilege,
		Choices: []ChoiceRecord{buildFirstPlayerPrivilegePaymentChoice("P1")},
	})
	if !state.Turn.FirstPlayerPrivilegeUsed {
		t.Fatal("first use should mark turn.firstPlayerPrivilegeUsed=true")
	}

	_, err := SubmitAction(state, Action{
		ID:      "act-first-player-privilege-second",
		ActorID: "P1",
		Kind:    ActionKindUseFirstPlayerPrivilege,
		Choices: []ChoiceRecord{buildFirstPlayerPrivilegePaymentChoice("P1")},
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

func TestFirstPlayerPrivilegeAction_ReplayPreservesPaymentSpend(t *testing.T) {
	initial := NewGameState(InitialStateConfig{
		GameID:         "first-player-privilege-replay",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	initial.Board.Cards = []CardState{
		{
			CardID:            "region-privilege-replay",
			Name:              "Region",
			Kind:              CardKindRegion,
			Zone:              CardZoneTable,
			Revealed:          true,
			InfluenceByPlayer: map[string]int{"P1": 2, "P2": 2},
		},
	}
	initial.Board.Markers.SetMarker("P1", markerTypeResource, 1)

	result, err := SubmitAction(initial, Action{
		ID:      "act-first-player-privilege-replay",
		ActorID: "P1",
		Kind:    ActionKindUseFirstPlayerPrivilege,
		Choices: []ChoiceRecord{buildFirstPlayerPrivilegePaymentChoice("P1")},
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	replayed, err := ReplayActions(initial, result.State.History.Actions)
	if err != nil {
		t.Fatalf("ReplayActions returned error: %v", err)
	}

	if got := replayed.Board.Markers.GetMarker("P1", markerTypeResource); got != 0 {
		t.Fatalf("replayed resource marker = %d, want 0", got)
	}
	if got := replayed.Board.Markers.GetMarker("P1", markerTypeFirstPlayerPrivilegeUsed); got != 1 {
		t.Fatalf("replayed used marker = %d, want 1", got)
	}
	if len(replayed.History.Actions) != 1 || len(replayed.History.Actions[0].Choices) != 1 {
		t.Fatalf("replayed history action choices = %#v, want explicit choice preserved", replayed.History.Actions)
	}
}
