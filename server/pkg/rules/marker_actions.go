package rules

// Purpose: Hosts marker-specific legality and execution paths to keep engine.go focused on shared flow.

func checkMarkerActionLegality(state GameState, action Action) LegalityResult {
	if action.MarkerType == "" {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.marker_type_missing",
			"action.markerType",
			nil,
		)
	}

	switch action.Kind {
	case ActionKindSetMarker:
		if action.MarkerAmount <= 0 {
			return legalityFailure(
				ReasonCodeRulesFailedRandomMaxInvalid,
				"rules.marker.amount_invalid",
				"action.markerAmount",
				nil,
			)
		}
	case ActionKindRemoveMarker:
		if action.MarkerAmount < 0 {
			return legalityFailure(
				ReasonCodeRulesFailedRandomMaxInvalid,
				"rules.marker.amount_invalid",
				"action.markerAmount",
				nil,
			)
		}

		targetPlayerID := markerTargetPlayerID(action.ActorID, action.TargetPlayerID)
		currentAmount := state.Board.Markers.GetMarker(targetPlayerID, action.MarkerType)
		if currentAmount <= 0 {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.marker.not_enough",
				"board.markers",
				map[string]string{
					"playerId":      targetPlayerID,
					"markerType":    action.MarkerType,
					"currentAmount": intString(currentAmount),
				},
			)
		}
		if action.MarkerAmount > currentAmount {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.marker.not_enough",
				"board.markers",
				map[string]string{
					"playerId":        targetPlayerID,
					"markerType":      action.MarkerType,
					"currentAmount":   intString(currentAmount),
					"requestedAmount": intString(action.MarkerAmount),
				},
			)
		}
	}

	return okLegalityResult()
}

func executeSetMarker(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	targetPlayerID := markerTargetPlayerID(operation.ActorID, operation.TargetPlayerID)
	setMarker(&working, targetPlayerID, operation.MarkerType, operation.MarkerAmount)
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindMarkerSet,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		StackDepth:       len(working.Board.Stack),
		TargetPlayerID:   targetPlayerID,
		ResolvedTargetID: targetPlayerID,
		MarkerType:       operation.MarkerType,
		MarkerAmount:     operation.MarkerAmount,
		RevisionNumber:   0,
	}, nil
}

func executeRemoveMarker(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	targetPlayerID := markerTargetPlayerID(operation.ActorID, operation.TargetPlayerID)

	currentAmount := working.Board.Markers.GetMarker(targetPlayerID, operation.MarkerType)
	removeAmount := operation.MarkerAmount
	if removeAmount <= 0 || removeAmount > currentAmount {
		removeAmount = currentAmount
	}
	newAmount := currentAmount - removeAmount
	if newAmount < 0 {
		newAmount = 0
	}

	setMarker(&working, targetPlayerID, operation.MarkerType, newAmount)
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindMarkerRemoved,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		StackDepth:       len(working.Board.Stack),
		TargetPlayerID:   targetPlayerID,
		ResolvedTargetID: targetPlayerID,
		MarkerType:       operation.MarkerType,
		MarkerAmount:     newAmount,
		RevisionNumber:   0,
	}, nil
}

func markerTargetPlayerID(actorID string, targetPlayerID string) string {
	if targetPlayerID != "" {
		return targetPlayerID
	}
	return actorID
}
