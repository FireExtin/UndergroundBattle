package rules

// Purpose: Hosts card-marker-specific legality and execution paths.

func checkCardMarkerActionLegality(state GameState, action Action) LegalityResult {
	if action.TargetCardID == "" {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"action.targetCardId",
			nil,
		)
	}
	if action.MarkerType == "" {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.marker_type_missing",
			"action.markerType",
			nil,
		)
	}
	if !hasCardID(state, action.TargetCardID) {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"board.cards",
			map[string]string{"targetCardId": action.TargetCardID},
		)
	}

	switch action.Kind {
	case ActionKindSetCardMarker:
		if action.MarkerAmount <= 0 {
			return legalityFailure(
				ReasonCodeRulesFailedRandomMaxInvalid,
				"rules.marker.amount_invalid",
				"action.markerAmount",
				nil,
			)
		}
	case ActionKindRemoveCardMarker:
		if action.MarkerAmount < 0 {
			return legalityFailure(
				ReasonCodeRulesFailedRandomMaxInvalid,
				"rules.marker.amount_invalid",
				"action.markerAmount",
				nil,
			)
		}
		current := state.Board.CardMarkers.GetMarker(action.TargetCardID, action.MarkerType)
		if current <= 0 {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.marker.not_enough",
				"board.cardMarkers",
				map[string]string{
					"targetCardId": action.TargetCardID,
					"markerType":   action.MarkerType,
				},
			)
		}
		if action.MarkerAmount > current {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.marker.not_enough",
				"board.cardMarkers",
				map[string]string{
					"targetCardId":    action.TargetCardID,
					"markerType":      action.MarkerType,
					"currentAmount":   intString(current),
					"requestedAmount": intString(action.MarkerAmount),
				},
			)
		}
	}

	return okLegalityResult()
}

func executeSetCardMarker(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	setCardMarker(&working, operation.TargetCardID, operation.MarkerType, operation.MarkerAmount)
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindCardMarkerSet,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		StackDepth:       len(working.Board.Stack),
		TargetCardID:     operation.TargetCardID,
		ResolvedTargetID: operation.TargetCardID,
		MarkerType:       operation.MarkerType,
		MarkerAmount:     operation.MarkerAmount,
		RevisionNumber:   0,
	}, nil
}

func executeRemoveCardMarker(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	current := working.Board.CardMarkers.GetMarker(operation.TargetCardID, operation.MarkerType)
	removeAmount := operation.MarkerAmount
	if removeAmount <= 0 || removeAmount > current {
		removeAmount = current
	}
	next := current - removeAmount
	if next < 0 {
		next = 0
	}

	setCardMarker(&working, operation.TargetCardID, operation.MarkerType, next)
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindCardMarkerRemoved,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		StackDepth:       len(working.Board.Stack),
		TargetCardID:     operation.TargetCardID,
		ResolvedTargetID: operation.TargetCardID,
		MarkerType:       operation.MarkerType,
		MarkerAmount:     next,
		RevisionNumber:   0,
	}, nil
}
