package rules

// Purpose: Extracts generic preflight legality checks from engine orchestration.

func checkActionPreflightLegality(state GameState, action Action) LegalityResult {
	if actionRequiresPriority(action.Kind) && action.ActorID != currentPriorityPlayerID(state) {
		return legalityFailure(
			ReasonCodeLegalityFailedNotYourPriority,
			"rules.legality.not_your_priority",
			"turn.priority",
			map[string]string{
				"actorId":          action.ActorID,
				"priorityPlayerId": currentPriorityPlayerID(state),
			},
		)
	}

	if actionRequiresEmptyStack(action.Kind) && len(state.Board.Stack) != 0 {
		return legalityFailure(
			ReasonCodeLegalityFailedStackNotEmpty,
			"rules.legality.stack_not_empty",
			"board.stack",
			map[string]string{
				"stackDepth": intString(len(state.Board.Stack)),
				"actionKind": string(action.Kind),
			},
		)
	}

	if action.TargetPlayerID != "" && !containsString(state.Players, action.TargetPlayerID) {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.player_missing",
			"action.targetPlayerId",
			map[string]string{
				"targetPlayerId": action.TargetPlayerID,
			},
		)
	}

	if action.TargetCardID != "" && !hasCardID(state, action.TargetCardID) {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"action.targetCardId",
			map[string]string{
				"targetCardId": action.TargetCardID,
			},
		)
	}

	return okLegalityResult()
}
