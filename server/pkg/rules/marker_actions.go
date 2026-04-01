package rules

// Purpose: Hosts marker-specific legality and execution paths to keep engine.go focused on shared flow.

func checkMarkerActionLegality(action Action) LegalityResult {
	if action.TargetPlayerID == "" {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.player_missing",
			"action.targetPlayerId",
			nil,
		)
	}
	if action.MarkerType == "" {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.marker_missing",
			"action.markerType",
			nil,
		)
	}
	if action.MarkerAmount <= 0 {
		return legalityFailure(
			ReasonCodeRulesFailedInvalidState,
			"rules.marker.amount_invalid",
			"action.markerAmount",
			map[string]string{
				"markerAmount": intString(action.MarkerAmount),
			},
		)
	}

	return okLegalityResult()
}

func executeSetMarker(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	setMarker(&working, operation.TargetPlayerID, operation.MarkerType, operation.MarkerAmount)
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
		ResolvedTargetID: operation.TargetPlayerID,
		MarkerType:       operation.MarkerType,
		MarkerAmount:     operation.MarkerAmount,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
	}, nil
}

func executeRemoveMarker(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	removeMarkerCount(&working, operation.TargetPlayerID, operation.MarkerType, operation.MarkerAmount)
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
		ResolvedTargetID: operation.TargetPlayerID,
		MarkerType:       operation.MarkerType,
		MarkerAmount:     operation.MarkerAmount,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
	}, nil
}
