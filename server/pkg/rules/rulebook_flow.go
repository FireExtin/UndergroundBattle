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
			readyTablePermanents(state)
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
		moveCardToDiscard(card)
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

func readyTablePermanents(state *GameState) {
	if state == nil {
		return
	}

	for index := range state.Board.Cards {
		card := &state.Board.Cards[index]
		if card.Zone != CardZoneTable || card.Destroyed {
			continue
		}
		if card.Kind != CardKindCharacter && card.Kind != CardKindAsset {
			continue
		}
		card.Exhausted = false
	}
}

func applyDrawStep(state *GameState, operation Operation) {
	if state == nil {
		return
	}
	if state.Match.Status == MatchStatusFinished {
		return
	}

	policy := DefaultDrawStepPolicy()
	if !policy.DrawWithoutStack {
		return
	}

	operationID := operation.ID + ":draw"
	if hasExplicitDeckModel(*state) {
		failedPlayers := drawOneFromDeckPerPlayer(state)
		if len(failedPlayers) == 0 {
			return
		}
		applyDeckOutResult(state, failedPlayers)
		return
	}

	for index, playerID := range state.Players {
		appendGeneratedDrawCard(state, operationID, playerID, index+1)
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

		cleanupWonRegionTableState(state, region.CardID)
		winnerID := region.ControllerID
		wonRegionOrder := region.RegionOrder
		moveCardToScore(region)
		state.Score.ByPlayer[winnerID]++

		if !refillRegionSlotFromWorldDeck(state, wonRegionOrder) {
			state.Board.Cards = append(state.Board.Cards, CardState{
				CardID:         fmt.Sprintf("region:auto:%d", nextAutoRegionIndex),
				Name:           "Auto Region",
				Kind:           CardKindRegion,
				Zone:           CardZoneTable,
				VisibleToOwner: false,
				Revealed:       true,
				RegionOrder:    wonRegionOrder,
			})
			nextAutoRegionIndex++
		}
	}
}

func refillRegionSlotFromWorldDeck(state *GameState, regionOrder int) bool {
	if state == nil {
		return false
	}

	deckIndex := topWorldRegionDeckIndex(*state)
	if deckIndex < 0 {
		return false
	}

	replacement := &state.Board.Cards[deckIndex]
	replacement.Destroyed = false
	replacement.Zone = CardZoneTable
	replacement.Revealed = true
	replacement.FaceDown = false
	replacement.Exhausted = false
	replacement.RegionCardID = ""
	replacement.RegionOrder = regionOrder
	return true
}

func cleanupWonRegionTableState(state *GameState, regionCardID string) {
	if state == nil || regionCardID == "" {
		return
	}

	for index := range state.Board.Cards {
		card := &state.Board.Cards[index]
		if card.Kind == CardKindRegion || card.Zone != CardZoneTable || card.Destroyed {
			continue
		}
		if card.RegionCardID != regionCardID {
			continue
		}
		moveCardToDiscard(card)
	}

	pruned := NewAttachmentManager(*state).PruneExpired()
	state.Board.Attachments = pruned.Board.Attachments
	state.Board.Continuous.Active = pruned.Board.Continuous.Active
}

func hasExplicitDeckModel(state GameState) bool {
	for _, card := range state.Board.Cards {
		if card.Zone == CardZoneDeck {
			return true
		}
	}
	return false
}

func drawOneFromDeckPerPlayer(state *GameState) []string {
	if state == nil {
		return nil
	}

	failed := make([]string, 0)
	for _, playerID := range state.Players {
		index := topDeckCardIndex(*state, playerID)
		if index < 0 {
			failed = append(failed, playerID)
			continue
		}
		drawCardFromDeck(&state.Board.Cards[index])
	}

	return failed
}

func topDeckCardIndex(state GameState, playerID string) int {
	for index := len(state.Board.Cards) - 1; index >= 0; index-- {
		card := state.Board.Cards[index]
		if card.OwnerID != playerID || card.Zone != CardZoneDeck || card.Destroyed {
			continue
		}
		return index
	}
	return -1
}

func topWorldRegionDeckIndex(state GameState) int {
	for index := len(state.Board.Cards) - 1; index >= 0; index-- {
		card := state.Board.Cards[index]
		if card.Kind != CardKindRegion || card.Zone != CardZoneDeck || card.Destroyed {
			continue
		}
		return index
	}
	return -1
}

func applyDeckOutResult(state *GameState, failedPlayers []string) {
	if state == nil || len(failedPlayers) == 0 {
		return
	}

	state.Match.Status = MatchStatusFinished
	state.Match.WinnerPlayerID = ""
	state.Score.WinnerPlayerID = ""

	if len(failedPlayers) == len(state.Players) {
		state.Match.EndReason = MatchEndReasonDeckOutDraw
		return
	}

	winner := survivingPlayerID(state.Players, failedPlayers)
	state.Match.EndReason = MatchEndReasonDeckOut
	state.Match.WinnerPlayerID = winner
	state.Score.WinnerPlayerID = winner
}

func survivingPlayerID(players []string, failedPlayers []string) string {
	failedSet := make(map[string]bool, len(failedPlayers))
	for _, playerID := range failedPlayers {
		failedSet[playerID] = true
	}
	for _, playerID := range players {
		if !failedSet[playerID] {
			return playerID
		}
	}
	return ""
}

func regionWinThreshold(region CardState) int {
	// Region scoring now uses explicit region score fields as the authoritative
	// victory threshold. Dynamic EffectiveStats.Influence represents current
	// control pressure on the table and must not be reused as the win threshold.
	if region.RegionScore > 0 {
		return region.RegionScore
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
		setMarker(state, playerID, markerTypeBuildAssetUsed, 0)
	}
}
