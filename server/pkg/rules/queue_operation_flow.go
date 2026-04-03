package rules

import "fmt"

// Purpose: Extracts queue_operation legality and build logic from engine orchestration paths.

func checkQueueOperationTargetLegality(state GameState, action Action) LegalityResult {
	// XQ31 "不能成为目标" should only block card/ability targeting on queue_operation.
	// Role actions like declare_attack / declare_investigation are intentionally out of scope.
	if action.Kind != ActionKindQueueOperation || action.TargetCardID == "" {
		return okLegalityResult()
	}

	targetLegalityChecker := BuildTargetLegalityChecker(state)
	targetResult := targetLegalityChecker.CheckTargetCard(state, action.ActorID, action.TargetCardID)
	if !targetResult.CanTarget {
		return legalityFailure(
			ReasonCodeTargetFailedProhibited,
			"rules.target.prohibited",
			"action.targetCardId",
			map[string]string{
				"targetCardId":        action.TargetCardID,
				"prohibitingCardId":   targetResult.SourceCardID,
				"prohibitingCardName": targetResult.SourceCardName,
			},
		)
	}

	return okLegalityResult()
}

func checkQueueOperationActionLegality(state GameState, action Action, sourceLookup cardOperationSourceLookup) LegalityResult {
	if !state.Turn.Phase.AllowsStack {
		return legalityFailure(
			ReasonCodeLegalityFailedStackClosed,
			"rules.legality.stack_closed",
			"turn.phase",
			map[string]string{
				"phase": string(state.Turn.Phase.Name),
			},
		)
	}

	if action.CardID == "" {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"action.cardId",
			nil,
		)
	}

	source, found, err := lookupCardOperationSourceWithLookup(sourceLookup, action.CardID)
	if err != nil {
		return legalityFailure(
			ReasonCodeRulesFailedCardLogicUnavailable,
			"rules.card_logic.unavailable",
			"shared.contracts.fixtures",
			map[string]string{
				"cardId": action.CardID,
				"error":  err.Error(),
			},
		)
	}

	if !found {
		return legalityFailure(
			ReasonCodeRulesFailedCardLogicMissing,
			"rules.card_logic.missing",
			"shared.contracts.fixtures",
			map[string]string{
				"cardId": action.CardID,
			},
		)
	}

	windowLegality := checkCardWindowLegality(state, source)
	if !windowLegality.OK {
		return windowLegality
	}

	playLegality := checkQueuedCardPlayLegality(state, action.ActorID, source)
	if !playLegality.OK {
		return playLegality
	}

	if !source.RequiresStack && len(state.Board.Stack) != 0 {
		return legalityFailure(
			ReasonCodeLegalityFailedStackNotEmpty,
			"rules.legality.stack_not_empty",
			"board.stack",
			map[string]string{
				"stackDepth": intString(len(state.Board.Stack)),
				"cardId":     action.CardID,
			},
		)
	}

	return okLegalityResult()
}

func buildQueueOperationFromAction(action Action, sourceLookup cardOperationSourceLookup, operation *Operation) error {
	if operation == nil {
		return fmt.Errorf("buildQueueOperationFromAction received nil operation")
	}

	source, found, err := lookupCardOperationSourceWithLookup(sourceLookup, action.CardID)
	if err != nil {
		return err
	}
	if !found {
		return fmt.Errorf("%s", ReasonCodeRulesFailedCardLogicMissing)
	}

	operation.Kind = OperationKindCardEffect
	operation.RequiresStack = source.RequiresStack
	operation.CardID = action.CardID
	operation.Label = source.CardName
	operation.Source = &source

	return nil
}

func executeQueueOperation(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)

	if operation.RequiresStack {
		working, operation = defaultStackEngine.Push(working, operation)
		reopenPhaseStep(&working.Turn)
		resetPriorityWindow(&working.Turn, nextPriorityPlayerID(working, operation.ActorID), PriorityWindowResponse)

		return working, operation, Event{
			ID:               "evt:" + operation.ActionID,
			ActionID:         operation.ActionID,
			OperationID:      operation.ID,
			Kind:             EventKindOperationEnqueued,
			Phase:            working.Turn.Phase.Name,
			Step:             working.Turn.Phase.Step,
			PriorityPlayerID: currentPriorityPlayerID(working),
			PriorityWindow:   currentPriorityWindowKind(working),
			PassCount:        working.Turn.Priority.PassCount,
			StackDepth:       len(working.Board.Stack),
			RevisionNumber:   0,
		}, nil
	}

	return executeCardEffect(working, operation)
}

func checkCardWindowLegality(state GameState, source CardOperationSource) LegalityResult {
	windowKind := currentPriorityWindowKind(state)
	switch source.Speed {
	case "slow":
		if windowKind != PriorityWindowAction {
			return legalityFailure(
				ReasonCodeLegalityFailedActionWindowRequired,
				"rules.legality.action_window_required",
				"turn.priority.window",
				map[string]string{
					"cardId":     source.CardID,
					"speed":      source.Speed,
					"windowKind": string(windowKind),
				},
			)
		}
	case "reaction":
		if windowKind != PriorityWindowResponse || len(state.Board.Stack) == 0 {
			return legalityFailure(
				ReasonCodeLegalityFailedResponseWindowRequired,
				"rules.legality.response_window_required",
				"turn.priority.window",
				map[string]string{
					"cardId":     source.CardID,
					"speed":      source.Speed,
					"windowKind": string(windowKind),
					"stackDepth": intString(len(state.Board.Stack)),
				},
			)
		}
	case "fast":
		if windowKind == PriorityWindowClosed {
			return legalityFailure(
				ReasonCodeLegalityFailedActionWindowRequired,
				"rules.legality.action_window_required",
				"turn.priority.window",
				map[string]string{
					"cardId":     source.CardID,
					"speed":      source.Speed,
					"windowKind": string(windowKind),
				},
			)
		}
	}

	return okLegalityResult()
}

func checkQueuedCardPlayLegality(state GameState, actorID string, source CardOperationSource) LegalityResult {
	targetCategory := TargetCategory{
		BasicTypes: []string{source.BasicType},
	}

	checker := BuildProhibitionChecker(state)
	result := checker.Check(state, actorID, targetCategory)

	if result.Prohibited {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.legality.action_prohibited",
			"board.cards",
			map[string]string{
				"cardId":              source.CardID,
				"basicType":           source.BasicType,
				"prohibitingCardId":   result.SourceCardID,
				"prohibitingCardName": result.SourceCardName,
			},
		)
	}

	return okLegalityResult()
}
