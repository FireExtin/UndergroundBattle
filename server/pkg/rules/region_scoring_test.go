package rules

import (
	"reflect"
	"testing"
)

// Purpose: Verifies the first minimal region contest, scoring, and victory loop built on top of role actions.

func TestDeclareInvestigationTracksPerPlayerRegionInfluence(t *testing.T) {
	state := newRoleActionTestState()
	state.Board.Cards = []CardState{
		testCharacterCard("p1-investigator", "P1", CardNumericStats{Combat: 1, Defense: 2, Investigation: 2}),
		testRegionCard("region-1"),
	}

	result, err := SubmitAction(state, Action{
		ID:           "act-region-influence-1",
		ActorID:      "P1",
		Kind:         ActionKindDeclareInvestigation,
		CardID:       "p1-investigator",
		TargetCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	region := cardStateByID(t, result.State, "region-1")
	if region.InfluenceByPlayer["P1"] != 2 {
		t.Fatalf("P1 region influence = %d, want 2", region.InfluenceByPlayer["P1"])
	}
	if region.Counters.Influence != 2 {
		t.Fatalf("region total influence = %d, want 2", region.Counters.Influence)
	}
	if region.ControllerID != "P1" {
		t.Fatalf("region controller = %q, want %q", region.ControllerID, "P1")
	}
}

func TestEndOfTurnAwardsPointToControlledRegionOwner(t *testing.T) {
	state := newRoleActionTestState()
	state.Board.Cards = []CardState{
		testRegionCard("region-1"),
	}
	state.Board.Cards[0].InfluenceByPlayer = map[string]int{"P1": 2, "P2": 1}
	refreshAllRegionControl(&state)

	state = mustSubmit(t, state, Action{
		ID:      "act-score-advance-1",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	result, err := SubmitAction(state, Action{
		ID:      "act-score-advance-2",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.State.Turn.TurnNumber != 2 {
		t.Fatalf("turn number = %d, want 2", result.State.Turn.TurnNumber)
	}
	if result.State.Score.ByPlayer["P1"] != 1 {
		t.Fatalf("P1 score = %d, want 1", result.State.Score.ByPlayer["P1"])
	}
	if result.State.Score.ByPlayer["P2"] != 0 {
		t.Fatalf("P2 score = %d, want 0", result.State.Score.ByPlayer["P2"])
	}
}

func TestEndOfTurnTieDoesNotAwardPoint(t *testing.T) {
	state := newRoleActionTestState()
	state.Board.Cards = []CardState{
		testRegionCard("region-1"),
	}
	state.Board.Cards[0].InfluenceByPlayer = map[string]int{"P1": 2, "P2": 2}
	refreshAllRegionControl(&state)

	state = mustSubmit(t, state, Action{
		ID:      "act-score-tie-1",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	result, err := SubmitAction(state, Action{
		ID:      "act-score-tie-2",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.State.Score.ByPlayer["P1"] != 0 || result.State.Score.ByPlayer["P2"] != 0 {
		t.Fatalf("scores = %#v, want no points on tie", result.State.Score.ByPlayer)
	}

	region := cardStateByID(t, result.State, "region-1")
	if region.ControllerID != "" {
		t.Fatalf("controller = %q, want empty on tie", region.ControllerID)
	}
}

func TestVictoryIsDeclaredWhenThresholdReached(t *testing.T) {
	state := newRoleActionTestState()
	state.Score.VictoryThreshold = 2
	state.Board.Cards = []CardState{
		testRegionCard("region-1"),
	}
	state.Board.Cards[0].InfluenceByPlayer = map[string]int{"P1": 3}
	refreshAllRegionControl(&state)

	for _, action := range []Action{
		{ID: "act-victory-1", ActorID: "P1", Kind: ActionKindAdvancePhase},
		{ID: "act-victory-2", ActorID: "P1", Kind: ActionKindAdvancePhase},
		{ID: "act-victory-3", ActorID: "P2", Kind: ActionKindAdvancePhase},
		{ID: "act-victory-4", ActorID: "P2", Kind: ActionKindAdvancePhase},
	} {
		var err error
		state, err = nextStateAfter(state, action)
		if err != nil {
			t.Fatalf("SubmitAction(%q) returned error: %v", action.ID, err)
		}
	}

	if state.Score.ByPlayer["P1"] != 2 {
		t.Fatalf("P1 score = %d, want 2", state.Score.ByPlayer["P1"])
	}
	if state.Score.WinnerPlayerID != "P1" {
		t.Fatalf("winner = %q, want %q", state.Score.WinnerPlayerID, "P1")
	}
}

func TestEndOfTurnRotatesActivePlayerToNextPlayer(t *testing.T) {
	state := newRoleActionTestState()

	state = mustSubmit(t, state, Action{
		ID:      "act-rotate-turn-1",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	result, err := SubmitAction(state, Action{
		ID:      "act-rotate-turn-2",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.State.Turn.ActivePlayerID != "P2" {
		t.Fatalf("active player = %q, want %q", result.State.Turn.ActivePlayerID, "P2")
	}
	if result.State.Turn.Priority.CurrentPlayerID != "P2" {
		t.Fatalf("priority player = %q, want %q", result.State.Turn.Priority.CurrentPlayerID, "P2")
	}
}

func TestRegionScoringReplayProducesSameState(t *testing.T) {
	initial := newRoleActionTestState()
	initial.Board.Cards = []CardState{
		testRegionCard("region-1"),
	}
	initial.Board.Cards[0].InfluenceByPlayer = map[string]int{"P1": 2}
	refreshAllRegionControl(&initial)

	actions := []Action{
		{ID: "act-replay-score-1", ActorID: "P1", Kind: ActionKindAdvancePhase},
		{ID: "act-replay-score-2", ActorID: "P1", Kind: ActionKindAdvancePhase},
	}

	finalState, err := ReplayActions(initial, actions)
	if err != nil {
		t.Fatalf("ReplayActions returned error: %v", err)
	}

	replayed, err := ReplayActions(initial, finalState.History.Actions)
	if err != nil {
		t.Fatalf("ReplayActions(history) returned error: %v", err)
	}

	if !reflect.DeepEqual(finalState, replayed) {
		t.Fatalf("replayed state mismatch\nfinal = %#v\nreplayed = %#v", finalState, replayed)
	}
}

func nextStateAfter(state GameState, action Action) (GameState, error) {
	result, err := SubmitAction(state, action)
	if err != nil {
		return GameState{}, err
	}

	return result.State, nil
}
