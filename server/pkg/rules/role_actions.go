package rules

// Purpose: Implements the first playable-loop role actions for attacking characters and investigating regions.

func checkRoleActionLegality(state GameState, action Action, requiredTargetKind CardKind) LegalityResult {
	if action.CardID == "" {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"action.cardId",
			nil,
		)
	}

	if action.TargetCardID == "" {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"action.targetCardId",
			nil,
		)
	}

	actorIndex := findCardIndex(state, action.CardID)
	if actorIndex == -1 {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"board.cards",
			map[string]string{"cardId": action.CardID},
		)
	}

	targetIndex := findCardIndex(state, action.TargetCardID)
	if targetIndex == -1 {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"board.cards",
			map[string]string{"targetCardId": action.TargetCardID},
		)
	}

	actorCard := state.Board.Cards[actorIndex]
	if actorCard.OwnerID != action.ActorID || actorCard.Kind != CardKindCharacter || actorCard.Zone != CardZoneTable || actorCard.Destroyed || actorCard.Exhausted {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.legality.actor_card_invalid",
			"board.cards",
			map[string]string{
				"cardId":     action.CardID,
				"actionKind": string(action.Kind),
			},
		)
	}

	permissionLegality := checkCardActionPermissionLegality(state, action.CardID, action.Kind)
	if !permissionLegality.OK {
		return permissionLegality
	}

	targetCard := state.Board.Cards[targetIndex]
	if targetCard.Zone != CardZoneTable || targetCard.Kind != requiredTargetKind || targetCard.Destroyed {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"board.cards",
			map[string]string{
				"targetCardId": action.TargetCardID,
				"actionKind":   string(action.Kind),
			},
		)
	}

	return okLegalityResult()
}

func executeDeclareAttack(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	attackerIndex := findCardIndex(working, operation.CardID)
	targetIndex := findCardIndex(working, operation.TargetCardID)
	damage := appliedCombatDamage(working.Board.Cards[attackerIndex])

	working.Board.Cards[attackerIndex].Exhausted = true
	working.Board.Cards[targetIndex].Counters.Damage += damage
	destroyedCardID := ""
	eventKind := EventKindDamageApplied
	if cardWillBeDestroyed(working.Board.Cards[targetIndex]) {
		eventKind = EventKindCardDestroyed
		destroyedCardID = operation.TargetCardID
	}

	requestContinuousRecalculation(&working)
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             eventKind,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		ResolvedTargetID: operation.TargetCardID,
		SourceCardID:     operation.CardID,
		TargetCardID:     operation.TargetCardID,
		AppliedAmount:    damage,
		DestroyedCardID:  destroyedCardID,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
	}, nil
}

func executeDeclareInvestigation(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	investigatorIndex := findCardIndex(working, operation.CardID)
	targetIndex := findCardIndex(working, operation.TargetCardID)
	applied := appliedInvestigation(working.Board.Cards[investigatorIndex])

	working.Board.Cards[investigatorIndex].Exhausted = true
	working.Board.Cards[targetIndex].Counters.Influence += applied
	if working.Board.Cards[targetIndex].Kind == CardKindRegion {
		if working.Board.Cards[targetIndex].InfluenceByPlayer == nil {
			working.Board.Cards[targetIndex].InfluenceByPlayer = map[string]int{}
		}
		working.Board.Cards[targetIndex].InfluenceByPlayer[operation.ActorID] += applied
		refreshAllRegionControl(&working)
	}
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindInvestigationApplied,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		ResolvedTargetID: operation.TargetCardID,
		SourceCardID:     operation.CardID,
		TargetCardID:     operation.TargetCardID,
		AppliedAmount:    applied,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
	}, nil
}

func appliedCombatDamage(card CardState) int {
	if card.EffectiveStats.Combat > 0 {
		return card.EffectiveStats.Combat
	}

	return 0
}

func appliedInvestigation(card CardState) int {
	if card.EffectiveStats.Investigation > 0 {
		return card.EffectiveStats.Investigation
	}

	return 0
}

func cardWillBeDestroyed(card CardState) bool {
	if card.Zone != CardZoneTable {
		return false
	}

	return card.Counters.Damage >= lethalDefenseThreshold(card.EffectiveStats.Defense)
}
