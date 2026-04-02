package rules

import "strings"

// Purpose: Provides the minimal turn-based resource model used by battle V4 play_card legality.

const maxTurnResource = 10

// InitializeTurnResources ensures player resource entries exist and refills the current active player.
func InitializeTurnResources(state *GameState) {
	if state == nil {
		return
	}
	ensureTurnResourceEntries(&state.Turn, state.Players)
	RefillActivePlayerResources(state)
}

// RefillActivePlayerResources refills the active player's resource pool to turn-based capacity.
func RefillActivePlayerResources(state *GameState) {
	if state == nil {
		return
	}
	ensureTurnResourceEntries(&state.Turn, state.Players)
	playerID := strings.TrimSpace(state.Turn.ActivePlayerID)
	if playerID == "" {
		return
	}
	capacity := turnResourceCapacity(state.Turn.TurnNumber)
	state.Turn.Resources[playerID] = PlayerResourceState{
		Current: capacity,
		Max:     capacity,
	}
}

func turnResourceCapacity(turnNumber int) int {
	if turnNumber < 0 {
		return 0
	}
	if turnNumber > maxTurnResource {
		return maxTurnResource
	}
	return turnNumber
}

func ensureTurnResourceEntries(turn *TurnState, players []string) {
	if turn == nil {
		return
	}
	if turn.Resources == nil {
		turn.Resources = make(map[string]PlayerResourceState, len(players))
	}
	for _, playerID := range players {
		playerID = strings.TrimSpace(playerID)
		if playerID == "" {
			continue
		}
		if _, ok := turn.Resources[playerID]; ok {
			continue
		}
		turn.Resources[playerID] = PlayerResourceState{}
	}
}

func currentPlayerResource(state GameState, playerID string) PlayerResourceState {
	if state.Turn.Resources == nil {
		return PlayerResourceState{}
	}
	return state.Turn.Resources[playerID]
}

func payPlayerResourceCost(state *GameState, playerID string, required int) bool {
	if state == nil {
		return false
	}
	if required <= 0 {
		return true
	}

	ensureTurnResourceEntries(&state.Turn, state.Players)
	pool := state.Turn.Resources[playerID]
	if pool.Current < required {
		return false
	}
	pool.Current -= required
	if pool.Current < 0 {
		pool.Current = 0
	}
	state.Turn.Resources[playerID] = pool
	return true
}
