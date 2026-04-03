package rules

import "slices"

// Purpose: Provides deterministic priority transfer helpers for the minimal response window implementation.

func normalizedPlayerIDs(config InitialStateConfig, activePlayerID string) []string {
	if len(config.PlayerIDs) == 0 {
		other := "P2"
		if activePlayerID == "P2" {
			other = "P1"
		}
		return []string{activePlayerID, other}
	}

	players := slices.Clone(config.PlayerIDs)
	if !slices.Contains(players, activePlayerID) {
		players = append([]string{activePlayerID}, players...)
	}

	if len(players) == 1 {
		other := "P2"
		if players[0] == "P2" {
			other = "P1"
		}
		players = append(players, other)
	}

	return players
}

func syncPriority(turn *TurnState, currentPlayerID string, passCount int, lastPassedPlayerID string, windowKind PriorityWindowKind) {
	turn.PriorityPlayerID = currentPlayerID
	turn.Priority = PriorityState{
		CurrentPlayerID:    currentPlayerID,
		PassCount:          passCount,
		LastPassedPlayerID: lastPassedPlayerID,
		WindowKind:         windowKind,
	}
}

func resetPriorityWindow(turn *TurnState, currentPlayerID string, windowKind PriorityWindowKind) {
	syncPriority(turn, currentPlayerID, 0, "", windowKind)
}

func closePriorityWindow(turn *TurnState, currentPlayerID string) {
	syncPriority(turn, currentPlayerID, 0, "", PriorityWindowClosed)
}

func reopenPhaseStep(turn *TurnState) {
	turn.Phase.StepEnded = false
	if turn.Phase.Step == StepEnded {
		turn.Phase.Step = StepAction
	}
}

func closePhaseStep(turn *TurnState) {
	turn.Phase.StepEnded = true
}

func currentStepPriorityLeader(state GameState) string {
	if state.Turn.Phase.Name == PhaseConflict && state.Turn.Conflict.PriorityLeaderPlayerID != "" {
		return state.Turn.Conflict.PriorityLeaderPlayerID
	}
	if state.Turn.Phase.Name == PhaseMain && state.Turn.Phase.Step == StepSecondPlayerAction {
		return nextPriorityPlayerID(state, state.Turn.ActivePlayerID)
	}
	if state.Turn.ActivePlayerID != "" {
		return state.Turn.ActivePlayerID
	}
	return currentPriorityPlayerID(state)
}

func resetPriorityToCurrentStepLeader(state *GameState, windowKind PriorityWindowKind) {
	if state == nil {
		return
	}
	resetPriorityWindow(&state.Turn, currentStepPriorityLeader(*state), windowKind)
}

func nextPriorityPlayerID(state GameState, currentPlayerID string) string {
	if len(state.Players) == 0 {
		return currentPlayerID
	}

	index := slices.Index(state.Players, currentPlayerID)
	if index == -1 {
		return state.Players[0]
	}

	return state.Players[(index+1)%len(state.Players)]
}
