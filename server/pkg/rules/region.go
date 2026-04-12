package rules

// Purpose: Implements the first minimal region-control, scoring, and victory helpers for the playable loop.

const defaultVictoryThreshold = 3

func newScoreState(players []string) ScoreState {
	byPlayer := make(map[string]int, len(players))
	for _, playerID := range players {
		byPlayer[playerID] = 0
	}

	return ScoreState{
		ByPlayer:         byPlayer,
		VictoryThreshold: defaultVictoryThreshold,
	}
}

func refreshAllRegionControl(state *GameState) {
	if state == nil {
		return
	}

	for index := range state.Board.Cards {
		card := &state.Board.Cards[index]
		if card.Kind != CardKindRegion {
			continue
		}

		refreshRegionControlWithState(state, card)
	}
}

func refreshRegionControlWithState(state *GameState, card *CardState) {
	if card == nil {
		return
	}

	if card.Kind != CardKindRegion || card.Zone != CardZoneTable || card.Destroyed {
		card.ControllerID = ""
		return
	}

	totalInfluence := deriveEffectiveRegionInfluence(state, *card)
	card.InfluenceByPlayer = totalInfluence
	card.Counters.Influence = sumInfluence(totalInfluence)
	card.EffectiveStats.Influence = card.Counters.Influence

	bestPlayerID := ""
	bestInfluence := 0
	tied := false
	for playerID, influence := range totalInfluence {
		if influence <= 0 {
			continue
		}

		if bestPlayerID == "" || influence > bestInfluence {
			bestPlayerID = playerID
			bestInfluence = influence
			tied = false
			continue
		}

		if influence == bestInfluence {
			tied = true
		}
	}

	if bestPlayerID == "" {
		card.ControllerID = ""
		return
	}

	if !tied {
		card.ControllerID = bestPlayerID
		return
	}

	if state == nil || len(state.Players) == 0 {
		card.ControllerID = ""
		return
	}

	firstPlayerID := state.Turn.ActivePlayerID
	if firstPlayerID == "" {
		firstPlayerID = state.Players[0]
	}
	requested := state.Board.Markers.GetMarker(firstPlayerID, markerTypeFirstPlayerPrivilegeRequest) > 0
	alreadyUsed := state.Board.Markers.GetMarker(firstPlayerID, markerTypeFirstPlayerPrivilegeUsed) > 0
	privilege := ApplyFirstPlayerPrivilege(ResolveContestOutcome(bestInfluence, bestInfluence), alreadyUsed || !requested)
	if privilege.Allowed {
		card.ControllerID = firstPlayerID
		setMarker(state, firstPlayerID, markerTypeFirstPlayerPrivilegeUsed, 1)
		setMarker(state, firstPlayerID, markerTypeFirstPlayerPrivilegeRequest, 0)
		state.Turn.FirstPlayerPrivilegeUsed = true
		return
	}

	card.ControllerID = ""
}

func deriveReadyCharacterRegionInfluence(state GameState, regionCardID string) map[string]int {
	if regionCardID == "" {
		return nil
	}

	result := map[string]int{}
	for _, card := range state.Board.Cards {
		if card.Kind != CardKindCharacter || card.Zone != CardZoneTable || card.Destroyed || card.Exhausted {
			continue
		}
		if card.RegionCardID != regionCardID {
			continue
		}
		if card.OwnerID == "" {
			continue
		}
		influence := card.EffectiveStats.Influence
		if influence <= 0 {
			continue
		}
		result[card.OwnerID] += influence
	}
	if len(result) == 0 {
		return nil
	}
	return result
}

func deriveEffectiveRegionInfluence(state *GameState, region CardState) map[string]int {
	baseInfluence := cloneIntMap(region.BaseInfluenceByPlayer)
	if state == nil {
		return baseInfluence
	}
	return mergeInfluence(baseInfluence, deriveReadyCharacterRegionInfluence(*state, region.CardID))
}

func mergeInfluence(base map[string]int, dynamic map[string]int) map[string]int {
	if len(base) == 0 && len(dynamic) == 0 {
		return nil
	}

	merged := cloneIntMap(base)
	if merged == nil {
		merged = map[string]int{}
	}
	for playerID, amount := range dynamic {
		merged[playerID] += amount
	}
	return merged
}

func refreshRegionControl(card *CardState) {
	refreshRegionControlWithState(nil, card)
}

func awardControlledRegionPoints(state *GameState) {
	if state == nil {
		return
	}

	ensureScoreStatePlayers(state)
	refreshAllRegionControl(state)
	for _, card := range state.Board.Cards {
		if card.Kind != CardKindRegion || card.Zone != CardZoneTable || card.Destroyed || card.ControllerID == "" {
			continue
		}

		state.Score.ByPlayer[card.ControllerID]++
	}
}

func evaluateWinner(state *GameState) {
	if state == nil {
		return
	}

	ensureScoreStatePlayers(state)
	threshold := state.Score.VictoryThreshold
	if threshold <= 0 {
		threshold = defaultVictoryThreshold
		state.Score.VictoryThreshold = threshold
	}

	bestPlayerID := ""
	bestScore := 0
	for _, playerID := range state.Players {
		score := state.Score.ByPlayer[playerID]
		if score < threshold {
			continue
		}
		if bestPlayerID == "" || score > bestScore {
			bestPlayerID = playerID
			bestScore = score
		}
	}

	state.Score.WinnerPlayerID = bestPlayerID
	if bestPlayerID == "" {
		return
	}

	state.Match.Status = MatchStatusFinished
	state.Match.EndReason = MatchEndReasonVictoryThreshold
	state.Match.WinnerPlayerID = bestPlayerID
}

func ensureScoreStatePlayers(state *GameState) {
	if state == nil {
		return
	}

	if state.Score.ByPlayer == nil {
		state.Score.ByPlayer = map[string]int{}
	}
	if state.Score.VictoryThreshold <= 0 {
		state.Score.VictoryThreshold = defaultVictoryThreshold
	}

	for _, playerID := range state.Players {
		if _, ok := state.Score.ByPlayer[playerID]; !ok {
			state.Score.ByPlayer[playerID] = 0
		}
	}
}

func sumInfluence(values map[string]int) int {
	total := 0
	for _, value := range values {
		if value > 0 {
			total += value
		}
	}

	return total
}
