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
	turn.Phase.Step = StepAction
	turn.Phase.StepEnded = false
}

func closePhaseStep(turn *TurnState) {
	turn.Phase.Step = StepEnded
	turn.Phase.StepEnded = true
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
