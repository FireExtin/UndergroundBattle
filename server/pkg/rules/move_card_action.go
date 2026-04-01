package rules

// Purpose: Provides a minimal, deterministic move-card action based on adjacent region order.

func checkMoveCardActionLegality(state GameState, action Action) LegalityResult {
	if action.CardID == "" || action.TargetCardID == "" {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"action.cardId",
			nil,
		)
	}

	moverIndex := findCardIndex(state, action.CardID)
	if moverIndex == -1 {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"board.cards",
			map[string]string{"cardId": action.CardID},
		)
	}
	mover := state.Board.Cards[moverIndex]
	if mover.OwnerID != action.ActorID || mover.Zone != CardZoneTable || mover.Destroyed {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.move.prohibited",
			"action.cardId",
			map[string]string{"cardId": action.CardID},
		)
	}
	if mover.RegionCardID == "" {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.move.source_region_missing",
			"card.regionCardId",
			map[string]string{"cardId": action.CardID},
		)
	}

	sourceRegion, ok := findRegionCard(state, mover.RegionCardID)
	if !ok {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.move.source_region_missing",
			"board.cards",
			map[string]string{"sourceRegionCardId": mover.RegionCardID},
		)
	}
	targetRegion, ok := findRegionCard(state, action.TargetCardID)
	if !ok {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"board.cards",
			map[string]string{"targetCardId": action.TargetCardID},
		)
	}
	if sourceRegion.CardID == targetRegion.CardID {
		return legalityFailure(
			ReasonCodeTargetFailedProhibited,
			"rules.move.target_same_region",
			"action.targetCardId",
			map[string]string{"targetCardId": action.TargetCardID},
		)
	}
	if !regionsAreAdjacent(sourceRegion.RegionOrder, targetRegion.RegionOrder) {
		return legalityFailure(
			ReasonCodeTargetFailedProhibited,
			"rules.move.target_not_adjacent",
			"action.targetCardId",
			map[string]string{
				"sourceRegionCardId": sourceRegion.CardID,
				"targetCardId":       action.TargetCardID,
			},
		)
	}

	return okLegalityResult()
}

func executeMoveCard(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	index := findCardIndex(working, operation.CardID)
	if index == -1 {
		return GameState{}, Operation{}, Event{}, &LegalityError{
			Result: legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.card_missing",
				"board.cards",
				map[string]string{"cardId": operation.CardID},
			),
			Code:       ReasonCodeTargetFailedMissing,
			Message:    "move card target missing",
			MessageKey: "rules.target.card_missing",
		}
	}

	moveCardToRegion(&working.Board.Cards[index], operation.TargetCardID)
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindCardMoved,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		StackDepth:       len(working.Board.Stack),
		ResolvedTargetID: operation.TargetCardID,
		TargetCardID:     operation.TargetCardID,
		RevisionNumber:   0,
	}, nil
}

func findRegionCard(state GameState, cardID string) (CardState, bool) {
	index := findCardIndex(state, cardID)
	if index == -1 {
		return CardState{}, false
	}
	card := state.Board.Cards[index]
	if card.Kind != CardKindRegion || card.Zone != CardZoneTable || card.Destroyed {
		return CardState{}, false
	}
	return card, true
}

func regionsAreAdjacent(left int, right int) bool {
	if left <= 0 || right <= 0 {
		return false
	}
	delta := left - right
	if delta < 0 {
		delta = -delta
	}
	return delta == 1
}
