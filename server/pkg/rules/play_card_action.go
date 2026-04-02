package rules

import (
	"fmt"
	"strings"
)

// Purpose: Implements hand-to-board/event deployment via play_card without introducing client-side adjudication.

const (
	playModeFaceUp   = "face_up"
	playModeFaceDown = "face_down"
)

func checkPlayCardActionLegality(state GameState, action Action, sourceLookup cardOperationSourceLookup) LegalityResult {
	if action.CardID == "" {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"action.cardId",
			nil,
		)
	}

	cardIndex := findCardIndex(state, action.CardID)
	if cardIndex == -1 {
		return legalityFailure(
			ReasonCodeTargetFailedMissing,
			"rules.target.card_missing",
			"board.cards",
			map[string]string{"cardId": action.CardID},
		)
	}

	card := state.Board.Cards[cardIndex]
	if card.OwnerID != action.ActorID || card.Zone != CardZoneHand || card.Destroyed {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.play_card.source_not_in_hand",
			"board.cards",
			map[string]string{"cardId": action.CardID},
		)
	}

	costLegality := checkPlayCardCost(state, action, card)
	if !costLegality.OK {
		return costLegality
	}

	switch card.Kind {
	case CardKindCharacter:
		mode := normalizePlayMode(action.PlayMode)
		if mode == "" {
			return legalityFailure(
				ReasonCodeRulesFailedInvalidState,
				"rules.play_card.mode_invalid",
				"action.playMode",
				map[string]string{"playMode": action.PlayMode},
			)
		}
		if len(state.Board.Stack) != 0 {
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
		if currentPriorityWindowKind(state) != PriorityWindowAction {
			return legalityFailure(
				ReasonCodeLegalityFailedActionWindowRequired,
				"rules.legality.action_window_required",
				"turn.priority.window",
				map[string]string{
					"windowKind": string(currentPriorityWindowKind(state)),
				},
			)
		}
		if action.TargetRegionCardID == "" {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.card_missing",
				"action.targetRegionCardId",
				nil,
			)
		}
		if _, ok := findRegionCard(state, action.TargetRegionCardID); !ok {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.card_missing",
				"board.cards",
				map[string]string{"targetRegionCardId": action.TargetRegionCardID},
			)
		}
		return okLegalityResult()
	case CardKindAsset:
		if len(state.Board.Stack) != 0 {
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
		if currentPriorityWindowKind(state) != PriorityWindowAction {
			return legalityFailure(
				ReasonCodeLegalityFailedActionWindowRequired,
				"rules.legality.action_window_required",
				"turn.priority.window",
				map[string]string{
					"windowKind": string(currentPriorityWindowKind(state)),
				},
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
		hostIndex := findCardIndex(state, action.TargetCardID)
		if hostIndex == -1 {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.card_missing",
				"board.cards",
				map[string]string{"targetCardId": action.TargetCardID},
			)
		}
		host := state.Board.Cards[hostIndex]
		if host.Zone != CardZoneTable || host.Destroyed || host.OwnerID != action.ActorID || host.Kind == CardKindRegion {
			return legalityFailure(
				ReasonCodeTargetFailedProhibited,
				"rules.play_card.asset_host_invalid",
				"action.targetCardId",
				map[string]string{"targetCardId": action.TargetCardID},
			)
		}
		return okLegalityResult()
	case CardKindEvent:
		definitionID := strings.TrimSpace(card.DefinitionID)
		if definitionID == "" {
			return legalityFailure(
				ReasonCodeRulesFailedCardLogicMissing,
				"rules.card_logic.missing",
				"board.cards.definitionId",
				map[string]string{"cardId": card.CardID},
			)
		}

		source, found, err := lookupCardOperationSourceWithLookup(sourceLookup, definitionID)
		if err != nil {
			return legalityFailure(
				ReasonCodeRulesFailedCardLogicUnavailable,
				"rules.card_logic.unavailable",
				"shared.contracts.fixtures",
				map[string]string{"cardId": definitionID, "error": err.Error()},
			)
		}
		if !found {
			return legalityFailure(
				ReasonCodeRulesFailedCardLogicMissing,
				"rules.card_logic.missing",
				"shared.contracts.fixtures",
				map[string]string{"cardId": definitionID},
			)
		}
		if source.BasicType != "事务" {
			return legalityFailure(
				ReasonCodeRulesFailedInvalidState,
				"rules.play_card.event_source_invalid",
				"shared.contracts.fixtures",
				map[string]string{
					"cardId":    definitionID,
					"basicType": source.BasicType,
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
					"cardId":     definitionID,
				},
			)
		}

		if requiresPlayCardPlayerTarget(source) && strings.TrimSpace(action.TargetPlayerID) == "" {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.player_missing",
				"action.targetPlayerId",
				nil,
			)
		}
		if requiresPlayCardRegionTarget(source) {
			if action.TargetCardID == "" {
				return legalityFailure(
					ReasonCodeTargetFailedMissing,
					"rules.target.card_missing",
					"action.targetCardId",
					nil,
				)
			}
			if _, ok := findRegionCard(state, action.TargetCardID); !ok {
				return legalityFailure(
					ReasonCodeTargetFailedMissing,
					"rules.target.card_missing",
					"board.cards",
					map[string]string{"targetCardId": action.TargetCardID},
				)
			}
		}

		return okLegalityResult()
	default:
		return legalityFailure(
			ReasonCodeRulesFailedInvalidState,
			"rules.play_card.unsupported_kind",
			"board.cards.kind",
			map[string]string{
				"cardId": action.CardID,
				"kind":   string(card.Kind),
			},
		)
	}
}

func checkPlayCardCost(state GameState, action Action, card CardState) LegalityResult {
	// Cost system is intentionally deferred; keep this hook for PN-BASE-001.
	return okLegalityResult()
}

func executePlayCard(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	cardIndex := findCardIndex(working, operation.CardID)
	if cardIndex == -1 {
		return GameState{}, Operation{}, Event{}, &LegalityError{
			Result: legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.card_missing",
				"board.cards",
				map[string]string{"cardId": operation.CardID},
			),
			Code:       ReasonCodeTargetFailedMissing,
			Message:    "play card source missing",
			MessageKey: "rules.target.card_missing",
		}
	}

	card := &working.Board.Cards[cardIndex]
	switch card.Kind {
	case CardKindCharacter:
		playMode := normalizePlayMode(operation.PlayMode)
		deployCardToTable(card, operation.TargetRegionCardID, playMode == playModeFaceDown)
		reopenPhaseStep(&working.Turn)
		resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
		operation.Status = OperationStatusResolved
		return working, operation, Event{
			ID:               "evt:" + operation.ActionID,
			ActionID:         operation.ActionID,
			OperationID:      operation.ID,
			Kind:             EventKindCardPlayed,
			Phase:            working.Turn.Phase.Name,
			Step:             working.Turn.Phase.Step,
			PriorityPlayerID: currentPriorityPlayerID(working),
			PriorityWindow:   currentPriorityWindowKind(working),
			PassCount:        working.Turn.Priority.PassCount,
			ResolvedTargetID: operation.CardID,
			SourceCardID:     operation.CardID,
			TargetCardID:     operation.TargetRegionCardID,
			StackDepth:       len(working.Board.Stack),
			RevisionNumber:   0,
		}, nil
	case CardKindAsset:
		hostIndex := findCardIndex(working, operation.TargetCardID)
		if hostIndex == -1 {
			return GameState{}, Operation{}, Event{}, fmt.Errorf("%s", ReasonCodeTargetFailedMissing)
		}
		host := working.Board.Cards[hostIndex]
		deployCardToTable(card, host.RegionCardID, false)
		nextState, _, err := attachToHost(working, attachmentTransitionSpec{
			SourceCardID:       card.CardID,
			SourceDefinitionID: card.DefinitionID,
			SourceOperationID:  operation.ID,
			TargetCardID:       host.CardID,
			HostCardID:         host.CardID,
			Revision:           working.Revision.Number,
			BasicType:          "附属",
		})
		if err != nil {
			return GameState{}, Operation{}, Event{}, err
		}
		working = nextState
		reopenPhaseStep(&working.Turn)
		resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
		operation.Status = OperationStatusResolved
		return working, operation, Event{
			ID:               "evt:" + operation.ActionID,
			ActionID:         operation.ActionID,
			OperationID:      operation.ID,
			Kind:             EventKindCardPlayed,
			Phase:            working.Turn.Phase.Name,
			Step:             working.Turn.Phase.Step,
			PriorityPlayerID: currentPriorityPlayerID(working),
			PriorityWindow:   currentPriorityWindowKind(working),
			PassCount:        working.Turn.Priority.PassCount,
			ResolvedTargetID: operation.CardID,
			SourceCardID:     operation.CardID,
			TargetCardID:     operation.TargetCardID,
			StackDepth:       len(working.Board.Stack),
			RevisionNumber:   0,
		}, nil
	case CardKindEvent:
		moveCardToDiscard(card)
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

		nextState, resolvedOperation, err := resolveCardEffect(working, operation)
		if err != nil {
			return GameState{}, Operation{}, Event{}, err
		}
		reopenPhaseStep(&nextState.Turn)
		resetPriorityWindow(&nextState.Turn, operation.ActorID, PriorityWindowAction)
		resolvedTargetID := resolvedOperation.ID
		if resolvedOperation.TargetCardID != "" {
			resolvedTargetID = resolvedOperation.TargetCardID
		}
		return nextState, resolvedOperation, Event{
			ID:               "evt:" + operation.ActionID,
			ActionID:         operation.ActionID,
			OperationID:      operation.ID,
			Kind:             EventKindOperationResolved,
			Phase:            nextState.Turn.Phase.Name,
			Step:             nextState.Turn.Phase.Step,
			PriorityPlayerID: currentPriorityPlayerID(nextState),
			PriorityWindow:   currentPriorityWindowKind(nextState),
			PassCount:        nextState.Turn.Priority.PassCount,
			ResolvedTargetID: resolvedTargetID,
			SourceCardID:     operation.CardID,
			TargetCardID:     operation.TargetCardID,
			StackDepth:       len(nextState.Board.Stack),
			RevisionNumber:   0,
		}, nil
	default:
		return GameState{}, Operation{}, Event{}, fmt.Errorf("%s", ReasonCodeRulesFailedInvalidState)
	}
}

func resolveStackedPlayCard(state GameState, operation Operation) (GameState, Operation, error) {
	if operation.Source != nil && operation.Source.BasicType == "事务" {
		return resolveCardEffect(state, operation)
	}
	resolved := markOperationResolved(operation)
	return finalizeResolvedOperation(state, resolved), resolved, nil
}

func deployCardToTable(card *CardState, regionCardID string, faceDown bool) {
	if card == nil {
		return
	}
	card.Destroyed = false
	card.Zone = CardZoneTable
	card.RegionCardID = regionCardID
	card.Exhausted = false
	card.VisibleToOwner = true
	card.FaceDown = faceDown
	card.Revealed = !faceDown
}

func normalizePlayMode(raw string) string {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" {
		return playModeFaceUp
	}
	switch trimmed {
	case playModeFaceUp, playModeFaceDown:
		return trimmed
	default:
		return ""
	}
}

func requiresPlayCardPlayerTarget(source CardOperationSource) bool {
	for _, targetKind := range source.TargetKinds {
		if targetKind == "player" {
			return true
		}
	}
	return false
}

func requiresPlayCardRegionTarget(source CardOperationSource) bool {
	for _, targetKind := range source.TargetKinds {
		if targetKind == "region" {
			return true
		}
	}
	return false
}
