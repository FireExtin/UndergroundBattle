package rules

import "fmt"

// Purpose: Extracts queue_operation legality and build logic from engine orchestration paths.

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
