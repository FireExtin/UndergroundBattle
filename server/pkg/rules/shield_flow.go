package rules

// Purpose: Implements Shield V1 interception for enemy-targeted card/ability actions.

const shieldMarkerType = "shield"

func tryResolveShieldInterception(state GameState, operation Operation) (GameState, Operation, Event, bool) {
	if !operationEligibleForShieldInterception(operation) {
		return state, operation, Event{}, false
	}

	targetIndex := findCardIndex(state, operation.TargetCardID)
	if targetIndex < 0 {
		return state, operation, Event{}, false
	}

	target := state.Board.Cards[targetIndex]
	if !canConsumeShieldForEnemyTarget(operation.ActorID, target) {
		return state, operation, Event{}, false
	}

	working := cloneGameState(state)
	if !consumeShieldCounter(&working.Board.Cards[targetIndex], 1) {
		return state, operation, Event{}, false
	}

	intercepted := cloneOperation(operation)
	switch intercepted.Kind {
	case OperationKindCardEffect:
		intercepted = markOperationResolved(intercepted)
		working = finalizeResolvedOperation(working, intercepted)
	case OperationKindDeclareAttack:
		attackerIndex := findCardIndex(working, intercepted.CardID)
		if attackerIndex >= 0 {
			exhaustCard(&working.Board.Cards[attackerIndex])
		}
		intercepted.Status = OperationStatusResolved
	default:
		intercepted.Status = OperationStatusResolved
	}

	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, intercepted.ActorID, PriorityWindowAction)

	targetPlayerID := cardControllerOrOwner(working.Board.Cards[targetIndex])
	return working, intercepted, Event{
		ID:               "evt:" + intercepted.ActionID,
		ActionID:         intercepted.ActionID,
		OperationID:      intercepted.ID,
		Kind:             EventKindShieldConsumed,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		ResolvedTargetID: intercepted.TargetCardID,
		SourceCardID:     intercepted.CardID,
		TargetCardID:     intercepted.TargetCardID,
		TargetPlayerID:   targetPlayerID,
		AppliedAmount:    1,
		MarkerType:       shieldMarkerType,
		MarkerAmount:     working.Board.Cards[targetIndex].Counters.Shield,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
	}, true
}

func operationEligibleForShieldInterception(operation Operation) bool {
	if operation.TargetCardID == "" {
		return false
	}

	switch operation.Kind {
	case OperationKindCardEffect, OperationKindDeclareAttack:
		return true
	default:
		return false
	}
}

func canConsumeShieldForEnemyTarget(actorID string, target CardState) bool {
	if actorID == "" {
		return false
	}
	if target.Zone != CardZoneTable || target.Destroyed {
		return false
	}
	if target.Counters.Shield <= 0 {
		return false
	}

	controllerID := cardControllerOrOwner(target)
	if controllerID == "" {
		return false
	}

	return controllerID != actorID
}

func cardControllerOrOwner(card CardState) string {
	if card.ControllerID != "" {
		return card.ControllerID
	}
	return card.OwnerID
}
