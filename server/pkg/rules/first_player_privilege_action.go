package rules

// Purpose: Hosts legality and execution flow for the explicit first-player privilege action.

const (
	firstPlayerPrivilegePaymentChoiceKind   = "pay_first_player_privilege_cost"
	firstPlayerPrivilegePaymentChoiceOption = "resource_marker"
)

func checkFirstPlayerPrivilegeActionLegality(state GameState, action Action) LegalityResult {
	if action.ActorID != state.Turn.ActivePlayerID {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.first_player_privilege.not_first_player",
			"turn.activePlayerId",
			map[string]string{
				"activePlayerId": state.Turn.ActivePlayerID,
			},
		)
	}

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

	choice, accepted := acceptedFirstPlayerPrivilegePaymentChoice(operation.ActorID, operation.Choices)
	if !accepted {
		return GameState{}, Operation{}, Event{}, &LegalityError{
			Result: legalityFailure(
				ReasonCodeCostFailedUnpaid,
				"rules.cost.unpaid",
				"choice.first_player_privilege_payment",
				nil,
			),
			Code:       ReasonCodeCostFailedUnpaid,
			Message:    "first-player privilege cost unpaid",
			MessageKey: "rules.cost.unpaid",
		}
	}

	payment, paid := payFirstPlayerPrivilegeCost(&working, operation.ActorID)
	if !paid {
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
	operation.Payment = clonePaymentRecord(payment)
	operation.Choices = []ChoiceRecord{choice}

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
		Payment:          clonePaymentRecord(payment),
		Choices:          []ChoiceRecord{choice},
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

func payFirstPlayerPrivilegeCost(state *GameState, actorID string) (*PaymentRecord, bool) {
	if state == nil || actorID == "" {
		return nil, false
	}

	if state.Board.Markers.GetMarker(actorID, markerTypeResource) < 1 {
		return nil, false
	}

	removeMarkerCount(state, actorID, markerTypeResource, 1)
	return &PaymentRecord{
		Kind:       PaymentKindMarker,
		MarkerType: markerTypeResource,
		Amount:     1,
	}, true
}

func buildFirstPlayerPrivilegePaymentChoice(actorID string) ChoiceRecord {
	return ChoiceRecord{
		Kind:     firstPlayerPrivilegePaymentChoiceKind,
		PlayerID: actorID,
		OptionID: firstPlayerPrivilegePaymentChoiceOption,
		Accepted: true,
	}
}

func acceptedFirstPlayerPrivilegePaymentChoice(actorID string, choices []ChoiceRecord) (ChoiceRecord, bool) {
	for _, choice := range choices {
		if choice.Kind != firstPlayerPrivilegePaymentChoiceKind {
			continue
		}
		if choice.PlayerID != actorID {
			continue
		}
		if choice.OptionID != "" && choice.OptionID != firstPlayerPrivilegePaymentChoiceOption {
			continue
		}
		if !choice.Accepted {
			return ChoiceRecord{}, false
		}

		normalized := choice
		normalized.OptionID = firstPlayerPrivilegePaymentChoiceOption
		return normalized, true
	}

	return ChoiceRecord{}, false
}
