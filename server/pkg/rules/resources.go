package rules

import "strings"

// Purpose: Provides the current authoritative payment implementation for battle actions.

const maxTurnResource = 10

type GamePaymentEngine struct{}

func (GamePaymentEngine) Initialize(state *GameState) {
	if state == nil {
		return
	}
	ensureTurnResourceEntries(&state.Turn, state.Players)
	refillTurnResourcesForAllPlayers(state)
}

func (GamePaymentEngine) RefillForTurn(state *GameState) {
	refillTurnResourcesForAllPlayers(state)
}

func (GamePaymentEngine) ResourceView(state GameState, playerID string) PlayerResourceState {
	if state.Turn.Resources == nil {
		return PlayerResourceState{}
	}
	return state.Turn.Resources[playerID]
}

func (GamePaymentEngine) PayCost(state *GameState, playerID string, required int) bool {
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

func (GamePaymentEngine) OnStepEnd(state *GameState) {
	// Current battle flow keeps resources until explicit refill/next-turn overwrite.
}

// InitializeTurnResources ensures player resource entries exist and refills each player's turn pool.
func InitializeTurnResources(state *GameState) {
	engine := CurrentPaymentEngine()
	if engine == nil {
		return
	}
	engine.Initialize(state)
}

// RefillActivePlayerResources refills the current shared battle resource view.
// Current rules kernel still uses a simplified shared-pool model for playable battle flow.
func RefillActivePlayerResources(state *GameState) {
	engine := CurrentPaymentEngine()
	if engine == nil {
		return
	}
	engine.RefillForTurn(state)
}

func refillTurnResourcesForAllPlayers(state *GameState) {
	if state == nil {
		return
	}
	ensureTurnResourceEntries(&state.Turn, state.Players)
	capacity := turnResourceCapacity(state.Turn.TurnNumber)
	for _, playerID := range state.Players {
		playerID = strings.TrimSpace(playerID)
		if playerID == "" {
			continue
		}
		state.Turn.Resources[playerID] = PlayerResourceState{
			Current: capacity,
			Max:     capacity,
		}
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
	engine := CurrentPaymentEngine()
	if engine == nil {
		return PlayerResourceState{}
	}
	return engine.ResourceView(state, playerID)
}

func payPlayerResourceCost(state *GameState, playerID string, required int) bool {
	engine := CurrentPaymentEngine()
	if engine == nil {
		return false
	}
	return engine.PayCost(state, playerID, required)
}
