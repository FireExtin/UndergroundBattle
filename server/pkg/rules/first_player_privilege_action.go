package rules

// Purpose: Hosts legality and execution flow for the explicit first-player privilege action.

func checkFirstPlayerPrivilegeActionLegality(state GameState, action Action) LegalityResult {
	if state.Turn.FirstPlayerPrivilegeUsed ||
		state.Board.Markers.GetMarker(action.ActorID, markerTypeFirstPlayerPrivilegeUsed) > 0 {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.first_player_privilege.already_used",
			"turn.firstPlayerPrivilegeUsed",
			nil,
		)
	}

	if !hasBreakableFirstPlayerTie(state, action.ActorID) {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.first_player_privilege.no_tie",
			"board.cards",
			nil,
		)
	}

	return okLegalityResult()
}

func executeUseFirstPlayerPrivilege(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)

	if !payFirstPlayerPrivilegeCost(&working, operation.ActorID) {
		return GameState{}, Operation{}, Event{}, &LegalityError{
			Result: legalityFailure(
				ReasonCodeCostFailedUnpaid,
				"rules.cost.unpaid",
				"cost.first_player_privilege",
				nil,
			),
			Code:       ReasonCodeCostFailedUnpaid,
			Message:    "first-player privilege cost unpaid",
			MessageKey: "rules.cost.unpaid",
		}
	}

	setMarker(&working, operation.ActorID, markerTypeFirstPlayerPrivilegeRequest, 1)
	refreshAllRegionControl(&working)
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindFirstPlayerPrivilegeUsed,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		StackDepth:       len(working.Board.Stack),
		TargetPlayerID:   operation.ActorID,
		ResolvedTargetID: operation.ActorID,
		RevisionNumber:   0,
	}, nil
}

func hasBreakableFirstPlayerTie(state GameState, firstPlayerID string) bool {
	if firstPlayerID == "" {
		return false
	}

	for _, card := range state.Board.Cards {
		if card.Kind != CardKindRegion || card.Zone != CardZoneTable || card.Destroyed {
			continue
		}
		if hasBreakableTieOnRegion(state, card, firstPlayerID) {
			return true
		}
	}

	return false
}

func hasBreakableTieOnRegion(state GameState, region CardState, firstPlayerID string) bool {
	top := 0
	tiedPlayers := 0

	for _, playerID := range state.Players {
		value := region.InfluenceByPlayer[playerID]
		if value <= 0 {
			continue
		}
		if value > top {
			top = value
			tiedPlayers = 1
			continue
		}
		if value == top {
			tiedPlayers++
		}
	}

	if top <= 0 || tiedPlayers < 2 {
		return false
	}

	return region.InfluenceByPlayer[firstPlayerID] == top
}

func payFirstPlayerPrivilegeCost(_ *GameState, _ string) bool {
	// Cost model hook. Current minimal engine does not track per-step resource pools yet.
	return true
}
