package rules

import "sort"

// Purpose: Implements rulebook-accurate phase/substep advancement for action rights and conflict windows.

func applyRulebookAdvance(state GameState, operation Operation) GameState {
	working := cloneGameState(state)

	if working.Turn.Phase.Name == PhaseMain && working.Turn.Phase.StepEnded {
		switch working.Turn.Phase.Step {
		case StepFirstPlayerAction:
			working.Turn.Phase = phaseState(PhaseMain)
			working.Turn.Phase.Step = StepSecondPlayerAction
			resetPriorityToCurrentStepLeader(&working, PriorityWindowAction)
			requestContinuousRecalculation(&working)
			return working
		case StepSecondPlayerAction:
			if startConflictPhase(&working) {
				requestContinuousRecalculation(&working)
				return working
			}
		}
	}

	if working.Turn.Phase.Name == PhaseConflict && working.Turn.Phase.StepEnded {
		advanceConflictPhase(&working)
		requestContinuousRecalculation(&working)
		return working
	}

	previousPhase := working.Turn.Phase.Name
	nextPhase := PhaseEnd
	switch previousPhase {
	case PhaseEnd:
		nextPhase = PhaseMain
	default:
		nextPhase = PhaseEnd
	}

	working.Turn.Phase = phaseState(nextPhase)
	working.Turn.PendingPrompt = nil
	if nextPhase != PhaseConflict {
		working.Turn.Conflict = ConflictState{}
	}

	if previousPhase == PhaseEnd && nextPhase == PhaseMain {
		applyEndToMainRulebookFlow(&working, operation)
		if engine := CurrentPaymentEngine(); engine != nil {
			engine.RefillForTurn(&working)
		}
	}

	resetPriorityToCurrentStepLeader(&working, PriorityWindowAction)
	return working
}

func startConflictPhase(state *GameState) bool {
	if state == nil {
		return false
	}

	regions := orderedConflictRegions(*state)
	if len(regions) == 0 {
		state.Turn.Phase = phaseState(PhaseEnd)
		state.Turn.Conflict = ConflictState{}
		state.Turn.PendingPrompt = nil
		resetPriorityToCurrentStepLeader(state, PriorityWindowAction)
		return false
	}

	state.Turn.Phase = phaseState(PhaseConflict)
	state.Turn.PendingPrompt = nil
	state.Turn.Conflict = ConflictState{
		RegionOrder:            regions[0].RegionOrder,
		RegionCardID:           regions[0].CardID,
		Stage:                  ConflictStagePreInvestigationFast,
		PriorityLeaderPlayerID: state.Turn.ActivePlayerID,
	}
	resetPriorityToCurrentStepLeader(state, PriorityWindowAction)
	return true
}

func advanceConflictPhase(state *GameState) {
	if state == nil {
		return
	}

	state.Turn.Phase = phaseState(PhaseConflict)
	state.Turn.PendingPrompt = nil

	switch state.Turn.Conflict.Stage {
	case ConflictStagePreInvestigationFast:
		resolveInvestigationContest(state)
	case ConflictStagePostInvestigationFast:
		setConflictStage(state, ConflictStagePreBattleFast)
	case ConflictStagePreBattleFast:
		resolveBattleContest(state)
	case ConflictStagePostBattleFast:
		setConflictStage(state, ConflictStagePreInfluenceFast)
	case ConflictStagePreInfluenceFast:
		resolveInfluenceContest(state)
	case ConflictStagePostInfluenceFast:
		advanceConflictRegionOrEnd(state)
	default:
		advanceConflictRegionOrEnd(state)
	}
}

func setConflictStage(state *GameState, stage ConflictStage) {
	if state == nil {
		return
	}

	state.Turn.Phase = phaseState(PhaseConflict)
	state.Turn.Conflict.Stage = stage
	state.Turn.Conflict.PriorityLeaderPlayerID = state.Turn.ActivePlayerID
	state.Turn.Conflict.PendingPromptID = ""
	if state.Turn.PendingPrompt != nil {
		state.Turn.Conflict.PendingPromptID = state.Turn.PendingPrompt.ID
	}
	resetPriorityToCurrentStepLeader(state, PriorityWindowAction)
}

func resolveInvestigationContest(state *GameState) {
	if state == nil {
		return
	}

	winnerID, difference := resolveContestByKind(*state, state.Turn.Conflict.RegionCardID, contestKindInvestigation)
	if winnerID != "" && difference > 0 {
		prompt := openInvestigationRewardPrompt(*state, winnerID, difference)
		if prompt != nil {
			state.Turn.PendingPrompt = prompt
			state.Turn.Conflict.PendingPromptID = prompt.ID
			setConflictStage(state, ConflictStageInvestigationRewardPrompt)
			state.Turn.PendingPrompt = prompt
			state.Turn.Conflict.PendingPromptID = prompt.ID
			resetPriorityWindow(&state.Turn, winnerID, PriorityWindowAction)
			return
		}
	}

	state.Turn.PendingPrompt = nil
	setConflictStage(state, ConflictStagePostInvestigationFast)
}

func resolveBattleContest(state *GameState) {
	if state == nil {
		return
	}

	winnerID, difference := resolveContestByKind(*state, state.Turn.Conflict.RegionCardID, contestKindBattle)
	if winnerID != "" && difference > 0 {
		eligible := eligibleBattleDamageTargets(*state, state.Turn.Conflict.RegionCardID, winnerID)
		if len(eligible) != 0 {
			prompt := &PromptState{
				ID:                "prompt:battle_damage:" + state.Turn.Conflict.RegionCardID,
				Kind:              PromptKindBattleDamage,
				OwnerPlayerID:     winnerID,
				RegionCardID:      state.Turn.Conflict.RegionCardID,
				EligibleTargetIDs: eligible,
				RemainingAmount:   difference,
				Difference:        difference,
			}
			state.Turn.PendingPrompt = prompt
			state.Turn.Conflict.PendingPromptID = prompt.ID
			setConflictStage(state, ConflictStageBattleDamagePrompt)
			state.Turn.PendingPrompt = prompt
			state.Turn.Conflict.PendingPromptID = prompt.ID
			resetPriorityWindow(&state.Turn, winnerID, PriorityWindowAction)
			return
		}
	}

	state.Turn.PendingPrompt = nil
	setConflictStage(state, ConflictStagePostBattleFast)
}

func resolveInfluenceContest(state *GameState) {
	if state == nil {
		return
	}

	winnerID, difference := resolveContestByKind(*state, state.Turn.Conflict.RegionCardID, contestKindInfluence)
	if winnerID != "" && difference > 0 {
		index := findCardIndex(*state, state.Turn.Conflict.RegionCardID)
		if index >= 0 {
			addInfluenceCounter(&state.Board.Cards[index], winnerID, difference)
			refreshAllRegionControl(state)
		}
	}
	setConflictStage(state, ConflictStagePostInfluenceFast)
}

func advanceConflictRegionOrEnd(state *GameState) {
	if state == nil {
		return
	}

	regions := orderedConflictRegions(*state)
	currentIndex := -1
	for index, region := range regions {
		if region.CardID == state.Turn.Conflict.RegionCardID {
			currentIndex = index
			break
		}
	}

	if currentIndex >= 0 && currentIndex+1 < len(regions) {
		nextRegion := regions[currentIndex+1]
		state.Turn.Phase = phaseState(PhaseConflict)
		state.Turn.PendingPrompt = nil
		state.Turn.Conflict = ConflictState{
			RegionOrder:            nextRegion.RegionOrder,
			RegionCardID:           nextRegion.CardID,
			Stage:                  ConflictStagePreInvestigationFast,
			PriorityLeaderPlayerID: state.Turn.ActivePlayerID,
		}
		resetPriorityToCurrentStepLeader(state, PriorityWindowAction)
		return
	}

	state.Turn.Phase = phaseState(PhaseEnd)
	state.Turn.Conflict = ConflictState{}
	state.Turn.PendingPrompt = nil
	resetPriorityToCurrentStepLeader(state, PriorityWindowAction)
}

type contestKind string

const (
	contestKindInvestigation contestKind = "investigation"
	contestKindBattle        contestKind = "battle"
	contestKindInfluence     contestKind = "influence"
)

func resolveContestByKind(state GameState, regionCardID string, kind contestKind) (string, int) {
	if regionCardID == "" || len(state.Players) < 2 {
		return "", 0
	}

	first := state.Players[0]
	second := state.Players[1]
	firstValue := contestValueForPlayer(state, regionCardID, first, kind)
	secondValue := contestValueForPlayer(state, regionCardID, second, kind)

	switch ResolveContestOutcome(firstValue, secondValue) {
	case ContestOutcomeActorWin:
		return first, firstValue - secondValue
	case ContestOutcomeActorLose:
		return second, secondValue - firstValue
	default:
		return "", 0
	}
}

func contestValueForPlayer(state GameState, regionCardID string, playerID string, kind contestKind) int {
	total := 0
	for _, card := range state.Board.Cards {
		if card.OwnerID != playerID || card.Zone != CardZoneTable || card.Destroyed || card.RegionCardID != regionCardID {
			continue
		}
		if card.Kind != CardKindCharacter {
			continue
		}
		if card.Exhausted {
			continue
		}

		switch kind {
		case contestKindInvestigation:
			if card.FaceDown {
				continue
			}
			total += maxInt(card.EffectiveStats.Investigation, 0)
		case contestKindBattle:
			if card.FaceDown {
				continue
			}
			total += maxInt(card.EffectiveStats.Combat, 0)
		case contestKindInfluence:
			if card.FaceDown {
				total++
				continue
			}
			total += maxInt(card.EffectiveStats.Influence, 0)
		}
	}
	return total
}

func eligibleBattleDamageTargets(state GameState, regionCardID string, winnerPlayerID string) []string {
	loserPlayerID := opposingPlayerID(state, winnerPlayerID)
	if loserPlayerID == "" {
		return nil
	}

	targets := make([]string, 0)
	for _, card := range state.Board.Cards {
		if card.OwnerID != loserPlayerID || card.Zone != CardZoneTable || card.Destroyed || card.RegionCardID != regionCardID {
			continue
		}
		if card.Kind != CardKindCharacter {
			continue
		}
		targets = append(targets, card.CardID)
	}
	return targets
}

func openInvestigationRewardPrompt(state GameState, winnerPlayerID string, difference int) *PromptState {
	peek := topDeckCardIDs(state, winnerPlayerID, difference)
	if len(peek) == 0 {
		return nil
	}
	return &PromptState{
		ID:            "prompt:investigation_reward:" + winnerPlayerID + ":" + state.Turn.Conflict.RegionCardID,
		Kind:          PromptKindInvestigationReward,
		OwnerPlayerID: winnerPlayerID,
		RegionCardID:  state.Turn.Conflict.RegionCardID,
		PeekCardIDs:   peek,
		Difference:    difference,
	}
}

func orderedConflictRegions(state GameState) []CardState {
	regions := make([]CardState, 0)
	for _, card := range state.Board.Cards {
		if card.Kind == CardKindRegion && card.Zone == CardZoneTable && !card.Destroyed {
			regions = append(regions, card)
		}
	}
	sort.SliceStable(regions, func(i, j int) bool {
		if regions[i].RegionOrder == regions[j].RegionOrder {
			return regions[i].CardID < regions[j].CardID
		}
		return regions[i].RegionOrder < regions[j].RegionOrder
	})
	return regions
}

func topDeckCardIDs(state GameState, playerID string, count int) []string {
	if count <= 0 {
		return nil
	}

	ids := make([]string, 0, count)
	for index := len(state.Board.Cards) - 1; index >= 0 && len(ids) < count; index-- {
		card := state.Board.Cards[index]
		if card.OwnerID != playerID || card.Zone != CardZoneDeck || card.Destroyed {
			continue
		}
		ids = append(ids, card.CardID)
	}
	return ids
}

func reorderPeekedCardsAndDraw(state *GameState, playerID string, peekCardIDs []string, topCardIDs []string, bottomCardIDs []string) {
	if state == nil {
		return
	}

	peekSet := make(map[string]struct{}, len(peekCardIDs))
	for _, cardID := range peekCardIDs {
		peekSet[cardID] = struct{}{}
	}

	deckIndices := make([]int, 0)
	deckCards := make([]CardState, 0)
	for index, card := range state.Board.Cards {
		if card.OwnerID != playerID || card.Zone != CardZoneDeck || card.Destroyed {
			continue
		}
		deckIndices = append(deckIndices, index)
		deckCards = append(deckCards, card)
	}

	cardByID := make(map[string]CardState, len(deckCards))
	remaining := make([]CardState, 0, len(deckCards))
	for _, card := range deckCards {
		cardByID[card.CardID] = card
		if _, ok := peekSet[card.CardID]; ok {
			continue
		}
		remaining = append(remaining, card)
	}

	ordered := make([]CardState, 0, len(deckCards))
	for _, cardID := range bottomCardIDs {
		if card, ok := cardByID[cardID]; ok {
			ordered = append(ordered, card)
		}
	}
	ordered = append(ordered, remaining...)
	for index := len(topCardIDs) - 1; index >= 0; index-- {
		cardID := topCardIDs[index]
		if card, ok := cardByID[cardID]; ok {
			ordered = append(ordered, card)
		}
	}

	for index, boardIndex := range deckIndices {
		if index >= len(ordered) {
			break
		}
		state.Board.Cards[boardIndex] = ordered[index]
	}

	topIndex := topDeckCardIndex(*state, playerID)
	if topIndex >= 0 {
		drawCardFromDeck(&state.Board.Cards[topIndex])
	}
}

func opposingPlayerID(state GameState, playerID string) string {
	for _, candidate := range state.Players {
		if candidate != playerID {
			return candidate
		}
	}
	return ""
}

func maxInt(left int, right int) int {
	if left > right {
		return left
	}
	return right
}
