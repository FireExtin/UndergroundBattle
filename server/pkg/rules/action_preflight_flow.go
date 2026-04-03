package rules

// Purpose: Extracts generic preflight legality checks from engine orchestration.

func checkActionPreflightLegality(state GameState, action Action) LegalityResult {
	policy, found := ActionPolicyForKind(action.Kind)
	if found {
		if legality := checkActionPolicyActorConstraint(state, action, policy); !legality.OK {
			return legality
		}
		if policy.RequiresPriority && action.ActorID != currentPriorityPlayerID(state) {
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
		if policy.RequiresEmptyStack && len(state.Board.Stack) != 0 {
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
		if policy.RequiresActionWindow && currentPriorityWindowKind(state) != PriorityWindowAction {
			return legalityFailure(
				ReasonCodeLegalityFailedActionWindowRequired,
				"rules.legality.action_window_required",
				"turn.priority.window",
				map[string]string{
					"windowKind": string(currentPriorityWindowKind(state)),
				},
			)
		}
		if len(policy.CardKindConstraints) > 0 && action.CardID != "" {
			cardIndex := findCardIndex(state, action.CardID)
			if cardIndex != -1 {
				card := state.Board.Cards[cardIndex]
				for _, constraint := range policy.CardKindConstraints {
					if constraint.Kind == card.Kind {
						if constraint.RequiresEmptyStack && len(state.Board.Stack) != 0 {
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
						if constraint.RequiresActionWindow && currentPriorityWindowKind(state) != PriorityWindowAction {
							return legalityFailure(
								ReasonCodeLegalityFailedActionWindowRequired,
								"rules.legality.action_window_required",
								"turn.priority.window",
								map[string]string{
									"windowKind": string(currentPriorityWindowKind(state)),
								},
							)
						}
						break
					}
				}
			}
		}
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

func checkActionPolicyActorConstraint(state GameState, action Action, policy ActionPolicy) LegalityResult {
	switch policy.ActorConstraint {
	case ActionActorConstraintAny:
		return okLegalityResult()
	case ActionActorConstraintPriorityPlayer:
		return okLegalityResult()
	case ActionActorConstraintActivePlayer:
		if action.ActorID == state.Turn.ActivePlayerID {
			return okLegalityResult()
		}
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.legality.active_player_required",
			"turn.activePlayerId",
			map[string]string{
				"activePlayerId": state.Turn.ActivePlayerID,
				"actorId":        action.ActorID,
				"actionKind":     string(action.Kind),
			},
		)
	default:
		return okLegalityResult()
	}
}
