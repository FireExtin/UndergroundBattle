package rules

import "strings"

// Purpose: Implements build_asset as a dedicated standard action that moves one hand card into the asset zone.

const markerTypeBuildAssetUsed = "build_asset_used"

func checkBuildAssetActionLegality(state GameState, action Action) LegalityResult {
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
			"rules.build_asset.source_not_in_hand",
			"board.cards",
			map[string]string{"cardId": action.CardID},
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

	if state.Board.Markers.GetMarker(action.ActorID, markerTypeBuildAssetUsed) > 0 {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.build_asset.once_per_turn",
			"board.markers",
			map[string]string{
				"actorId": action.ActorID,
			},
		)
	}

	if cardDisallowsBuildAsset(card) {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.build_asset.non_asset_prohibited",
			"board.cards",
			map[string]string{
				"cardId": action.CardID,
			},
		)
	}

	return okLegalityResult()
}

func executeBuildAsset(state GameState, operation Operation) (GameState, Operation, Event, error) {
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
			Message:    "build asset source missing",
			MessageKey: "rules.target.card_missing",
		}
	}

	if working.Board.Markers.GetMarker(operation.ActorID, markerTypeBuildAssetUsed) > 0 {
		return GameState{}, Operation{}, Event{}, &LegalityError{
			Result: legalityFailure(
				ReasonCodeLegalityFailedActionProhibited,
				"rules.build_asset.once_per_turn",
				"board.markers",
				map[string]string{"actorId": operation.ActorID},
			),
			Code:       ReasonCodeLegalityFailedActionProhibited,
			Message:    "build asset once per turn",
			MessageKey: "rules.build_asset.once_per_turn",
		}
	}

	card := &working.Board.Cards[cardIndex]
	if cardDisallowsBuildAsset(*card) {
		return GameState{}, Operation{}, Event{}, &LegalityError{
			Result: legalityFailure(
				ReasonCodeLegalityFailedActionProhibited,
				"rules.build_asset.non_asset_prohibited",
				"board.cards",
				map[string]string{"cardId": operation.CardID},
			),
			Code:       ReasonCodeLegalityFailedActionProhibited,
			Message:    "build asset prohibited by non-asset restriction",
			MessageKey: "rules.build_asset.non_asset_prohibited",
		}
	}

	card.Zone = CardZoneAsset
	card.Kind = CardKindAsset
	card.RegionCardID = ""
	card.RegionOrder = 0
	card.RegionScore = 0
	card.FaceDown = false
	card.Revealed = true
	card.VisibleToOwner = true
	card.Destroyed = false

	setMarker(&working, operation.ActorID, markerTypeBuildAssetUsed, 1)
	operation.Status = OperationStatusResolved
	requestContinuousRecalculation(&working)

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindAssetBuilt,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		ResolvedTargetID: operation.CardID,
		SourceCardID:     operation.CardID,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
	}, nil
}

func cardDisallowsBuildAsset(card CardState) bool {
	for _, keyword := range card.PrintedKeywords {
		if strings.TrimSpace(keyword) == "非资产" {
			return true
		}
	}
	for _, keyword := range card.EffectiveKeywords {
		if strings.TrimSpace(keyword) == "非资产" {
			return true
		}
	}

	return strings.Contains(card.Description, "非资产")
}
