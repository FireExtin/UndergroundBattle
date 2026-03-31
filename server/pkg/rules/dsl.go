package rules

import (
	"fmt"
	"strings"
)

// Purpose: Applies the current minimal executable subset of pure CardLogic DSL effects to GameState.

func applyDSLEffect(state GameState, operation Operation, effect EffectSpec) GameState {
	switch effect.Kind {
	case "drawCards":
		return applyDrawCardsEffect(state, operation, effect)
	case "inspectHand":
		return applyInspectHandEffect(state, operation, effect)
	case "exhaust":
		return applyExhaustEffect(state, operation, effect)
	case "addKeyword":
		return applyAddKeywordEffect(state, operation, effect)
	case "modifyStat":
		return applyModifyStatEffect(state, operation, effect)
	case "placeInfluence":
		return applyPlaceInfluenceEffect(state, operation, effect)
	case "dealDamage":
		return applyDealDamageEffect(state, operation, effect)
	default:
		return state
	}
}

func applyDrawCardsEffect(state GameState, operation Operation, effect EffectSpec) GameState {
	if effect.TargetRef != "controller" || effect.Amount == nil || *effect.Amount <= 0 {
		return state
	}

	working := cloneGameState(state)
	startSequence := countGeneratedDrawCards(working, operation.ID)
	for offset := 0; offset < *effect.Amount; offset++ {
		working.Board.Cards = append(working.Board.Cards, CardState{
			CardID:         fmt.Sprintf("draw:%s:%d", operation.ID, startSequence+offset+1),
			Name:           "",
			OwnerID:        operation.ActorID,
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Revealed:       false,
			Exhausted:      false,
		})
	}

	return working
}

func applyInspectHandEffect(state GameState, operation Operation, effect EffectSpec) GameState {
	targetPlayerID := runtimeTargetPlayerID(operation, effect)
	if targetPlayerID == "" {
		return state
	}

	working := cloneGameState(state)
	for index, card := range working.Board.Cards {
		if card.OwnerID != targetPlayerID || card.Zone != CardZoneHand {
			continue
		}

		if !containsString(card.InspectedBy, operation.ActorID) {
			working.Board.Cards[index].InspectedBy = append(working.Board.Cards[index].InspectedBy, operation.ActorID)
		}
	}

	return working
}

func applyExhaustEffect(state GameState, operation Operation, effect EffectSpec) GameState {
	targetCardID := runtimeTargetCardID(operation, effect)
	if targetCardID == "" {
		return state
	}

	index := findCardIndex(state, targetCardID)
	if index == -1 {
		return state
	}

	working := cloneGameState(state)
	working.Board.Cards[index].Exhausted = true
	return working
}

func applyAddKeywordEffect(state GameState, operation Operation, effect EffectSpec) GameState {
	if effect.Keyword == "" {
		return state
	}

	return registerContinuousEffect(state, operation, effect)
}

func applyModifyStatEffect(state GameState, operation Operation, effect EffectSpec) GameState {
	if effect.Stat == "" || effect.Amount == nil {
		return state
	}

	return registerContinuousEffect(state, operation, effect)
}

func applyPlaceInfluenceEffect(state GameState, operation Operation, effect EffectSpec) GameState {
	targetCardID := runtimeTargetCardID(operation, effect)
	if targetCardID == "" || effect.Amount == nil || *effect.Amount <= 0 {
		return state
	}

	index := findCardIndex(state, targetCardID)
	if index == -1 {
		return state
	}

	working := cloneGameState(state)
	working.Board.Cards[index].Counters.Influence += *effect.Amount
	if working.Board.Cards[index].Kind == CardKindRegion {
		if working.Board.Cards[index].InfluenceByPlayer == nil {
			working.Board.Cards[index].InfluenceByPlayer = map[string]int{}
		}
		working.Board.Cards[index].InfluenceByPlayer[operation.ActorID] += *effect.Amount
		refreshAllRegionControl(&working)
	}
	return working
}

func applyDealDamageEffect(state GameState, operation Operation, effect EffectSpec) GameState {
	targetCardID := runtimeTargetCardID(operation, effect)
	if targetCardID == "" || effect.Amount == nil || *effect.Amount <= 0 {
		return state
	}

	index := findCardIndex(state, targetCardID)
	if index == -1 {
		return state
	}

	working := cloneGameState(state)
	working.Board.Cards[index].Counters.Damage += *effect.Amount
	requestContinuousRecalculation(&working)
	return working
}

func runtimeTargetPlayerID(operation Operation, effect EffectSpec) string {
	switch effect.TargetRef {
	case "controller":
		return operation.ActorID
	case "selected":
		return operation.TargetPlayerID
	default:
		return ""
	}
}

func runtimeTargetCardID(operation Operation, effect EffectSpec) string {
	switch effect.TargetRef {
	case "controller":
		return operation.CardID
	case "selected":
		return operation.TargetCardID
	default:
		return ""
	}
}

func countGeneratedDrawCards(state GameState, operationID string) int {
	prefix := "draw:" + operationID + ":"
	count := 0
	for _, card := range state.Board.Cards {
		if strings.HasPrefix(card.CardID, prefix) {
			count++
		}
	}

	return count
}
