package rules

import "strings"

// Purpose: Provides the minimal turn-based resource model used by battle V4 play_card legality.

const maxTurnResource = 10

type PrototypePaymentEngine struct{}

func (PrototypePaymentEngine) Mode() PaymentMode {
	return PaymentModePrototype
}

func (PrototypePaymentEngine) Initialize(state *GameState) {
	if state == nil {
		return
	}
	ensureTurnResourceEntries(&state.Turn, state.Players)
	refillTurnResourcesForAllPlayers(state)
}

func (PrototypePaymentEngine) RefillForTurn(state *GameState) {
	refillTurnResourcesForAllPlayers(state)
}

func (PrototypePaymentEngine) ResourceView(state GameState, playerID string) PlayerResourceState {
	if state.Turn.Resources == nil {
		return PlayerResourceState{}
	}
	return state.Turn.Resources[playerID]
}

func (PrototypePaymentEngine) PayCost(state *GameState, playerID string, required int) bool {
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

// InitializeTurnResources ensures player resource entries exist and refills each player's turn pool.
func InitializeTurnResources(state *GameState) {
	engine := CurrentPaymentEngine()
	if engine == nil {
		return
	}
	engine.Initialize(state)
}

// RefillActivePlayerResources keeps the historical API surface while refilling both players.
// The playable battle prototype allows both players to spend resources when they have priority.
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
