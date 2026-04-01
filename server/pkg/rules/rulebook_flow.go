package rules

import (
	"fmt"
	"strconv"
	"strings"
)

// Purpose: Applies extracted rulebook flow policies to the minimal end->main transition path.

const (
	defaultHandLimit = 7

	markerTypeFirstPlayerPrivilegeRequest = "first_player_privilege_request"
	markerTypeFirstPlayerPrivilegeUsed    = "first_player_privilege_used"
)

func applyEndToMainRulebookFlow(state *GameState, operation Operation) {
	if state == nil {
		return
	}

	applyRecoveryStep(state)
	resolveRegionWins(state)
	awardControlledRegionPoints(state)

	state.Turn.TurnNumber++
	evaluateWinner(state)

	resetFirstPlayerPrivilegeMarkers(state)
	applyDrawStep(state, operation)
}

func applyRecoveryStep(state *GameState) {
	if state == nil {
		return
	}

	firstPlayerID := state.Turn.ActivePlayerID
	secondPlayerID := nextPriorityPlayerID(*state, firstPlayerID)
	for _, phase := range DefaultRecoveryStepOrder() {
		switch phase {
		case RecoveryStepPhaseFirstPlayerDiscardToLimit:
			discardExcessHandCards(state, firstPlayerID, defaultHandLimit)
		case RecoveryStepPhaseSecondPlayerDiscardToLimit:
			discardExcessHandCards(state, secondPlayerID, defaultHandLimit)
		case RecoveryStepPhaseClearDamageAndEndTurnEffects:
			clearTableCharacterDamage(state)
		case RecoveryStepPhaseTransferFirstPlayer:
			state.Turn.ActivePlayerID = secondPlayerID
		}
	}
}

func discardExcessHandCards(state *GameState, playerID string, handLimit int) {
	if state == nil || playerID == "" {
		return
	}
	if handLimit < 0 {
		handLimit = 0
	}

	handIndices := make([]int, 0)
	for index, card := range state.Board.Cards {
		if card.OwnerID == playerID && card.Zone == CardZoneHand {
			handIndices = append(handIndices, index)
		}
	}
	excess := len(handIndices) - handLimit
	if excess <= 0 {
		return
	}

	// Deterministic discard choice: latest cards in board-order hand slice are discarded first.
	for cursor := len(handIndices) - 1; cursor >= 0 && excess > 0; cursor-- {
		card := &state.Board.Cards[handIndices[cursor]]
		card.Zone = CardZoneDiscard
		card.Destroyed = true
		card.Revealed = true
		card.FaceDown = false
		excess--
	}
}

func clearTableCharacterDamage(state *GameState) {
	if state == nil {
		return
	}

	for index := range state.Board.Cards {
		card := &state.Board.Cards[index]
		if card.Zone != CardZoneTable || card.Kind != CardKindCharacter || card.Destroyed {
			continue
		}
		card.Counters.Damage = 0
	}
}

func applyDrawStep(state *GameState, operation Operation) {
	if state == nil {
		return
	}

	policy := DefaultDrawStepPolicy()
	if !policy.DrawWithoutStack {
		return
	}

	for index, playerID := range state.Players {
		appendGeneratedDrawCard(state, operation.ID+":draw", playerID, index+1)
	}
}

func resolveRegionWins(state *GameState) {
	if state == nil {
		return
	}

	nextAutoRegionIndex := nextAutoRegionIndex(*state)
	initialLen := len(state.Board.Cards)
	for index := 0; index < initialLen; index++ {
		region := &state.Board.Cards[index]
		if region.Kind != CardKindRegion || region.Zone != CardZoneTable || region.Destroyed {
			continue
		}
		threshold := regionWinThreshold(*region)
		if threshold <= 0 {
			continue
		}

		refreshRegionControlWithState(state, region)
		if region.ControllerID == "" || region.Counters.Influence < threshold {
			continue
		}

		winnerID := region.ControllerID
		moveCardToDiscard(region)
		state.Score.ByPlayer[winnerID]++

		state.Board.Cards = append(state.Board.Cards, CardState{
			CardID:         fmt.Sprintf("region:auto:%d", nextAutoRegionIndex),
			Name:           "Auto Region",
			Kind:           CardKindRegion,
			Zone:           CardZoneTable,
			VisibleToOwner: false,
			Revealed:       true,
		})
		nextAutoRegionIndex++
	}
}

func regionWinThreshold(region CardState) int {
	if region.EffectiveStats.Influence > 0 {
		return region.EffectiveStats.Influence
	}
	if region.PrintedStats.Influence > 0 {
		return region.PrintedStats.Influence
	}
	return 0
}

func nextAutoRegionIndex(state GameState) int {
	next := 1
	for _, card := range state.Board.Cards {
		if !strings.HasPrefix(card.CardID, "region:auto:") {
			continue
		}
		indexString := strings.TrimPrefix(card.CardID, "region:auto:")
		index, err := strconv.Atoi(indexString)
		if err != nil {
			continue
		}
		if index >= next {
			next = index + 1
		}
	}
	return next
}

func resetFirstPlayerPrivilegeMarkers(state *GameState) {
	if state == nil {
		return
	}
	state.Turn.FirstPlayerPrivilegeUsed = false
	for _, playerID := range state.Players {
		setMarker(state, playerID, markerTypeFirstPlayerPrivilegeUsed, 0)
		setMarker(state, playerID, markerTypeFirstPlayerPrivilegeRequest, 0)
	}
}
