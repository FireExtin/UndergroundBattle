package rules

import (
	"fmt"
	"slices"

	internalcontracts "undergroundbattle/server/internal/contracts"
	contractspkg "undergroundbattle/server/pkg/contracts"
)

// Purpose: Implements the authoritative action pipeline with structured legality, priority passing, and stack resolution.

var defaultStackEngine StackEngine

// NewGameState builds the deterministic initial state used by the minimal rules kernel.
func NewGameState(config InitialStateConfig) GameState {
	gameID := config.GameID
	if gameID == "" {
		gameID = "game-1"
	}

	activePlayerID := config.ActivePlayerID
	if activePlayerID == "" {
		activePlayerID = "P1"
	}

	players := normalizedPlayerIDs(config, activePlayerID)

	state := GameState{
		GameID:  gameID,
		Players: players,
		Revision: Revision{
			Number: 0,
		},
		Match: MatchState{
			Status: MatchStatusActive,
		},
		Turn: TurnState{
			TurnNumber:     1,
			ActivePlayerID: activePlayerID,
			Phase:          phaseState(PhaseMain),
		},
		Board: BoardState{
			Stack:         []Operation{},
			Resolved:      []Operation{},
			RandomResults: []RandomResult{},
			Cards:         []CardState{},
			Continuous: ContinuousEffectRegistry{
				Active: []ContinuousEffect{},
			},
			Attachments: AttachmentRegistry{
				Active:           []Attachment{},
				NextAttachmentID: 1,
			},
			Markers: MarkerRegistry{
				ByPlayer: make(map[string]map[string]int),
			},
		},
		Score: newScoreState(players),
		History: HistoryState{
			Actions:    []Action{},
			Operations: []Operation{},
			Events:     []Event{},
			Revisions:  []Revision{},
		},
		RNG: RNGState{
			Seed:      config.Seed,
			State:     config.Seed,
			DrawCount: 0,
		},
	}

	resetPriorityWindow(&state.Turn, activePlayerID, PriorityWindowAction)
	return state
}

// SubmitAction runs the legality -> operation -> stack/direct resolution -> event -> commit pipeline.
func SubmitAction(state GameState, action Action) (SubmitResult, error) {
	return SubmitActionWithProjection(state, action, NewProjectionEngine())
}

// SubmitActionWithProjection runs the same pipeline as SubmitAction but exposes projection generation for tests and callers.
func SubmitActionWithProjection(state GameState, action Action, projector *ProjectionEngine) (SubmitResult, error) {
	legality := CheckLegality(state, action)
	if !legality.OK {
		return SubmitResult{}, newLegalityError(legality)
	}

	operation, err := BuildOperation(state, action)
	if err != nil {
		return SubmitResult{}, err
	}

	working := cloneGameState(state)
	working, operation, event, err := executeOperation(working, operation)
	if err != nil {
		return SubmitResult{}, err
	}

	result := commitState(working, action, operation, event, projector)

	// Check invariants on the committed state so history/revision bookkeeping and
	// commit-time recalculation are validated together.
	if DefaultInvariantConfig.Enabled {
		results := CheckAllInvariants(result.State, DefaultInvariantConfig)
		for _, result := range results {
			if !result.Passed {
				invariantError := legalityFailure(
					ReasonCodeRulesFailedInvariantViolated,
					"rules.invariant.violated",
					"invariant.check",
					map[string]string{
						"actionId":      action.ID,
						"invariantName": result.Name,
						"message":       result.Message,
					},
				)
				return SubmitResult{}, newLegalityError(invariantError)
			}
		}
	}

	return result, nil
}

// ReplayActions replays an action log against an initial snapshot.
func ReplayActions(initial GameState, actions []Action) (GameState, error) {
	replayed := cloneGameState(initial)
	for _, action := range actions {
		result, err := submitActionWithoutProjection(replayed, action)
		if err != nil {
			return GameState{}, err
		}

		replayed = result.State
	}

	return replayed, nil
}

func submitActionWithoutProjection(state GameState, action Action) (SubmitResult, error) {
	return SubmitActionWithProjection(state, action, nil)
}

// CheckLegality returns a structured machine-readable legality result instead of a plain text error.
func CheckLegality(state GameState, action Action) LegalityResult {
	if action.ID == "" {
		return legalityFailure(
			ReasonCodeLegalityFailedActionIDMissing,
			"rules.legality.action_id_missing",
			"action.id",
			nil,
		)
	}

	if action.ActorID == "" {
		return legalityFailure(
			ReasonCodeLegalityFailedActorIDMissing,
			"rules.legality.actor_id_missing",
			"action.actorId",
			nil,
		)
	}

	if hasActionID(state, action.ID) {
		return legalityFailure(
			ReasonCodeLegalityFailedActionIDDuplicate,
			"rules.legality.action_id_duplicate",
			"history.actions",
			map[string]string{
				"actionId": action.ID,
			},
		)
	}

	if state.Match.Status == MatchStatusFinished {
		if state.Match.WinnerPlayerID == "" {
			return legalityFailure(
				ReasonCodeRulesFailedInvalidState,
				"rules.game.invalid_state",
				"match.winnerPlayerId",
				map[string]string{
					"status":    string(state.Match.Status),
					"endReason": string(state.Match.EndReason),
				},
			)
		}
		return legalityFailure(
			ReasonCodeRulesFailedGameAlreadyOver,
			"rules.game.already_over",
			"match.status",
			map[string]string{
				"winnerPlayerId": state.Match.WinnerPlayerID,
				"endReason":      string(state.Match.EndReason),
			},
		)
	}

	if state.Turn.Phase.StepEnded && action.Kind != ActionKindAdvancePhase {
		return legalityFailure(
			ReasonCodeRulesFailedStepEnded,
			"rules.phase.step_ended",
			"turn.phase",
			map[string]string{
				"phase": string(state.Turn.Phase.Name),
			},
		)
	}

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

	// Check target legality for card operation targets (not for role actions like attack/investigation)
	// XQ31 "不能成为目标" should only block card/ability targeting, not role actions.
	if action.TargetCardID != "" && action.Kind == ActionKindQueueOperation {
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
	}

	switch action.Kind {
	case ActionKindAdvancePhase:
		return okLegalityResult()
	case ActionKindRevealCard, ActionKindInspectCard:
		if action.CardID == "" {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.card_missing",
				"action.cardId",
				nil,
			)
		}

		if !hasCardID(state, action.CardID) {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.card_missing",
				"board.cards",
				map[string]string{
					"cardId": action.CardID,
				},
			)
		}

		permissionLegality := checkCardActionPermissionLegality(state, action.CardID, action.Kind)
		if !permissionLegality.OK {
			return permissionLegality
		}

		return okLegalityResult()
	case ActionKindPassPriority:
		return okLegalityResult()
	case ActionKindDeclareAttack:
		return checkRoleActionLegality(state, action, CardKindCharacter)
	case ActionKindDeclareInvestigation:
		return checkRoleActionLegality(state, action, CardKindRegion)
	case ActionKindQueueOperation:
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

		source, found, err := lookupCardOperationSource(action.CardID)
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
	case ActionKindResolveTopStack:
		if len(state.Board.Stack) == 0 {
			return legalityFailure(
				ReasonCodeStackFailedEmpty,
				"rules.stack.empty",
				"board.stack",
				nil,
			)
		}

		return okLegalityResult()
	case ActionKindRollSeededRandom:
		if action.RandomMax <= 0 {
			return legalityFailure(
				ReasonCodeRulesFailedRandomMaxInvalid,
				"rules.random.max_invalid",
				"action.randomMax",
				nil,
			)
		}

		return okLegalityResult()
	case ActionKindSetMarker:
		if action.MarkerType == "" {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.marker_type_missing",
				"action.markerType",
				nil,
			)
		}

		if action.MarkerAmount <= 0 {
			return legalityFailure(
				ReasonCodeRulesFailedRandomMaxInvalid,
				"rules.marker.amount_invalid",
				"action.markerAmount",
				nil,
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

		return okLegalityResult()
	case ActionKindRemoveMarker:
		if action.MarkerType == "" {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.marker_type_missing",
				"action.markerType",
				nil,
			)
		}

		if action.MarkerAmount < 0 {
			return legalityFailure(
				ReasonCodeRulesFailedRandomMaxInvalid,
				"rules.marker.amount_invalid",
				"action.markerAmount",
				nil,
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

		// 检查目标玩家是否有足够的标记物可以移除
		targetPlayerID := action.TargetPlayerID
		if targetPlayerID == "" {
			targetPlayerID = action.ActorID
		}

		currentAmount := state.Board.Markers.GetMarker(targetPlayerID, action.MarkerType)
		if currentAmount <= 0 {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.marker.not_enough",
				"board.markers",
				map[string]string{
					"playerId":      targetPlayerID,
					"markerType":    action.MarkerType,
					"currentAmount": intString(currentAmount),
				},
			)
		}

		if action.MarkerAmount > currentAmount {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.marker.not_enough",
				"board.markers",
				map[string]string{
					"playerId":        targetPlayerID,
					"markerType":      action.MarkerType,
					"currentAmount":   intString(currentAmount),
					"requestedAmount": intString(action.MarkerAmount),
				},
			)
		}

		return okLegalityResult()
	case ActionKindSetFaceDown:
		if action.CardID == "" {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.card_missing",
				"action.cardId",
				nil,
			)
		}

		if !hasCardID(state, action.CardID) {
			return legalityFailure(
				ReasonCodeTargetFailedMissing,
				"rules.target.card_missing",
				"board.cards",
				map[string]string{
					"cardId": action.CardID,
				},
			)
		}

		permissionLegality := checkCardActionPermissionLegality(state, action.CardID, action.Kind)
		if !permissionLegality.OK {
			return permissionLegality
		}

		return okLegalityResult()
	default:
		return legalityFailure(
			ReasonCodeRulesFailedUnknownActionKind,
			"rules.action.unknown_kind",
			"action.kind",
			map[string]string{
				"actionKind": string(action.Kind),
			},
		)
	}
}

// BuildOperation normalizes a legal action into a single authoritative operation.
func BuildOperation(state GameState, action Action) (Operation, error) {
	operation := Operation{
		ID:             "op:" + action.ID,
		ActionID:       action.ID,
		ActorID:        action.ActorID,
		TargetPlayerID: action.TargetPlayerID,
		TargetCardID:   action.TargetCardID,
		Status:         OperationStatusBuilt,
	}

	switch action.Kind {
	case ActionKindAdvancePhase:
		nextPhase, err := nextPhaseName(state.Turn.Phase.Name)
		if err != nil {
			return Operation{}, err
		}
		operation.Kind = OperationKindAdvancePhase
		operation.NextPhase = nextPhase
	case ActionKindRevealCard:
		operation.Kind = OperationKindRevealCard
		operation.CardID = action.CardID
	case ActionKindInspectCard:
		operation.Kind = OperationKindInspectCard
		operation.CardID = action.CardID
	case ActionKindPassPriority:
		operation.Kind = OperationKindPassPriority
	case ActionKindDeclareAttack:
		operation.Kind = OperationKindDeclareAttack
		operation.CardID = action.CardID
		operation.TargetCardID = action.TargetCardID
		operation.Label = "declare_attack"
	case ActionKindDeclareInvestigation:
		operation.Kind = OperationKindDeclareInvestigation
		operation.CardID = action.CardID
		operation.TargetCardID = action.TargetCardID
		operation.Label = "declare_investigation"
	case ActionKindQueueOperation:
		source, found, err := lookupCardOperationSource(action.CardID)
		if err != nil {
			return Operation{}, err
		}
		if !found {
			return Operation{}, fmt.Errorf("%s", ReasonCodeRulesFailedCardLogicMissing)
		}
		operation.Kind = OperationKindCardEffect
		operation.RequiresStack = source.RequiresStack
		operation.CardID = action.CardID
		operation.Label = source.CardName
		operation.Source = &source
	case ActionKindResolveTopStack:
		operation.Kind = OperationKindResolveTopStack
	case ActionKindRollSeededRandom:
		operation.Kind = OperationKindRollRandom
		operation.RandomMax = action.RandomMax
	case ActionKindSetMarker:
		operation.Kind = OperationKindSetMarker
		operation.MarkerType = action.MarkerType
		operation.MarkerAmount = action.MarkerAmount
		operation.TargetPlayerID = action.TargetPlayerID
	case ActionKindRemoveMarker:
		operation.Kind = OperationKindRemoveMarker
		operation.MarkerType = action.MarkerType
		operation.MarkerAmount = action.MarkerAmount
		operation.TargetPlayerID = action.TargetPlayerID
	case ActionKindSetFaceDown:
		operation.Kind = OperationKindSetFaceDown
		operation.CardID = action.CardID
	default:
		return Operation{}, fmt.Errorf("unsupported action kind %q", action.Kind)
	}

	return operation, nil
}

func executeOperation(state GameState, operation Operation) (GameState, Operation, Event, error) {
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

	switch operation.Kind {
	case OperationKindAdvancePhase:
		working = applyPhaseAdvance(working, operation)
		operation.Status = OperationStatusResolved

		return working, operation, Event{
			ID:               "evt:" + operation.ActionID,
			ActionID:         operation.ActionID,
			OperationID:      operation.ID,
			Kind:             EventKindPhaseAdvanced,
			Phase:            working.Turn.Phase.Name,
			Step:             working.Turn.Phase.Step,
			PriorityPlayerID: currentPriorityPlayerID(working),
			PriorityWindow:   currentPriorityWindowKind(working),
			PassCount:        working.Turn.Priority.PassCount,
			StackDepth:       len(working.Board.Stack),
			RevisionNumber:   0,
		}, nil
	case OperationKindRevealCard:
		return executeRevealCard(working, operation)
	case OperationKindInspectCard:
		return executeInspectCard(working, operation)
	case OperationKindPassPriority:
		return executePassPriority(working, operation)
	case OperationKindDeclareAttack:
		return executeDeclareAttack(working, operation)
	case OperationKindDeclareInvestigation:
		return executeDeclareInvestigation(working, operation)
	case OperationKindCardEffect:
		return executeCardEffect(working, operation)
	case OperationKindResolveTopStack:
		return executeResolveTopStack(working, operation)
	case OperationKindRollRandom:
		return executeRollRandom(working, operation)
	case OperationKindSetMarker:
		return executeSetMarker(working, operation)
	case OperationKindRemoveMarker:
		return executeRemoveMarker(working, operation)
	case OperationKindSetFaceDown:
		return executeSetFaceDown(working, operation)
	default:
		return GameState{}, Operation{}, Event{}, fmt.Errorf("unsupported operation kind %q", operation.Kind)
	}
}

func executeCardEffect(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working, operation, err := resolveCardEffect(state, operation)
	if err != nil {
		return GameState{}, Operation{}, Event{}, err
	}
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)

	resolvedTargetID := operation.ID
	if operation.CardID != "" {
		resolvedTargetID = operation.CardID
	}

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindOperationResolved,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		ResolvedTargetID: resolvedTargetID,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
	}, nil
}

func executeRevealCard(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	index := findCardIndex(working, operation.CardID)
	if index == -1 {
		return GameState{}, Operation{}, Event{}, fmt.Errorf("%s", ReasonCodeTargetFailedMissing)
	}

	working.Board.Cards[index].Revealed = true
	working.Board.Cards[index].FaceDown = false // 同时设置 FaceDown = false
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindCardRevealed,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		ResolvedTargetID: operation.CardID,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
	}, nil
}

func executeInspectCard(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	index := findCardIndex(working, operation.CardID)
	if index == -1 {
		return GameState{}, Operation{}, Event{}, fmt.Errorf("%s", ReasonCodeTargetFailedMissing)
	}

	if !containsString(working.Board.Cards[index].InspectedBy, operation.ActorID) {
		working.Board.Cards[index].InspectedBy = append(working.Board.Cards[index].InspectedBy, operation.ActorID)
	}
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindCardInspected,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		ResolvedTargetID: operation.CardID,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
	}, nil
}

func executePassPriority(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	operation.Status = OperationStatusResolved
	passCount := working.Turn.Priority.PassCount + 1
	requiredPasses := consecutivePassLimit(working)

	if passCount < requiredPasses {
		syncPriority(
			&working.Turn,
			nextPriorityPlayerID(working, operation.ActorID),
			passCount,
			operation.ActorID,
			currentPriorityWindowKind(working),
		)

		return working, operation, Event{
			ID:               "evt:" + operation.ActionID,
			ActionID:         operation.ActionID,
			OperationID:      operation.ID,
			Kind:             EventKindPriorityPassed,
			Phase:            working.Turn.Phase.Name,
			Step:             working.Turn.Phase.Step,
			PriorityPlayerID: currentPriorityPlayerID(working),
			PriorityWindow:   currentPriorityWindowKind(working),
			PassCount:        working.Turn.Priority.PassCount,
			StackDepth:       len(working.Board.Stack),
			RevisionNumber:   0,
		}, nil
	}

	if len(working.Board.Stack) != 0 {
		poppedState, pending, err := defaultStackEngine.PopTop(working)
		if err != nil {
			return GameState{}, Operation{}, Event{}, err
		}

		working, resolvedTarget, err := resolveStackedOperation(poppedState, pending)
		if err != nil {
			return GameState{}, Operation{}, Event{}, err
		}
		reopenPhaseStep(&working.Turn)
		resetPriorityWindow(&working.Turn, working.Turn.ActivePlayerID, postResolutionWindowKind(working))

		return working, operation, Event{
			ID:               "evt:" + operation.ActionID,
			ActionID:         operation.ActionID,
			OperationID:      operation.ID,
			Kind:             EventKindOperationResolved,
			Phase:            working.Turn.Phase.Name,
			Step:             working.Turn.Phase.Step,
			PriorityPlayerID: currentPriorityPlayerID(working),
			PriorityWindow:   currentPriorityWindowKind(working),
			PassCount:        working.Turn.Priority.PassCount,
			ResolvedTargetID: resolvedTarget.ID,
			StackDepth:       len(working.Board.Stack),
			RevisionNumber:   0,
		}, nil
	}

	closePhaseStep(&working.Turn)
	closePriorityWindow(&working.Turn, working.Turn.ActivePlayerID)

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindStepEnded,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
		StepEnded:        true,
	}, nil
}

func executeResolveTopStack(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working, pending, err := defaultStackEngine.PopTop(state)
	if err != nil {
		return GameState{}, Operation{}, Event{}, err
	}

	working, resolvedTarget, err := resolveStackedOperation(working, pending)
	if err != nil {
		return GameState{}, Operation{}, Event{}, err
	}
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, working.Turn.ActivePlayerID, postResolutionWindowKind(working))
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindOperationResolved,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		ResolvedTargetID: resolvedTarget.ID,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
	}, nil
}

func executeRollRandom(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	nextRNG, value, err := NextRandom(working.RNG, operation.RandomMax)
	if err != nil {
		return GameState{}, Operation{}, Event{}, err
	}

	working.RNG = nextRNG
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	working.Board.RandomResults = append(working.Board.RandomResults, RandomResult{
		ActionID:    operation.ActionID,
		OperationID: operation.ID,
		DrawIndex:   nextRNG.DrawCount,
		Value:       value,
	})
	operation.Status = OperationStatusResolved
	randomValue := value

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindRandomGenerated,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		StackDepth:       len(working.Board.Stack),
		RandomValue:      &randomValue,
		RevisionNumber:   0,
	}, nil
}

func executeSetMarker(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	targetPlayerID := operation.TargetPlayerID
	if targetPlayerID == "" {
		targetPlayerID = operation.ActorID
	}

	working.Board.Markers.SetMarker(targetPlayerID, operation.MarkerType, operation.MarkerAmount)
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindMarkerSet,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		StackDepth:       len(working.Board.Stack),
		TargetPlayerID:   targetPlayerID,
		MarkerType:       operation.MarkerType,
		MarkerAmount:     operation.MarkerAmount,
		RevisionNumber:   0,
	}, nil
}

func executeRemoveMarker(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	targetPlayerID := operation.TargetPlayerID
	if targetPlayerID == "" {
		targetPlayerID = operation.ActorID
	}

	currentAmount := working.Board.Markers.GetMarker(targetPlayerID, operation.MarkerType)
	removeAmount := operation.MarkerAmount
	if removeAmount <= 0 || removeAmount > currentAmount {
		removeAmount = currentAmount
	}

	newAmount := currentAmount - removeAmount
	if newAmount < 0 {
		newAmount = 0
	}

	working.Board.Markers.SetMarker(targetPlayerID, operation.MarkerType, newAmount)
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindMarkerRemoved,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		StackDepth:       len(working.Board.Stack),
		TargetPlayerID:   targetPlayerID,
		MarkerType:       operation.MarkerType,
		MarkerAmount:     newAmount,
		RevisionNumber:   0,
	}, nil
}

func executeSetFaceDown(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	index := findCardIndex(working, operation.CardID)
	if index == -1 {
		return GameState{}, Operation{}, Event{}, fmt.Errorf("%s", ReasonCodeTargetFailedMissing)
	}

	working.Board.Cards[index].FaceDown = true
	working.Board.Cards[index].Revealed = false
	reopenPhaseStep(&working.Turn)
	resetPriorityWindow(&working.Turn, operation.ActorID, PriorityWindowAction)
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindFaceDownSet,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		ResolvedTargetID: operation.CardID,
		StackDepth:       len(working.Board.Stack),
		RevisionNumber:   0,
	}, nil
}

func applyPhaseAdvance(state GameState, operation Operation) GameState {
	working := cloneGameState(state)
	previousPhase := working.Turn.Phase.Name
	working.Turn.Phase = phaseState(operation.NextPhase)

	if previousPhase == PhaseEnd && operation.NextPhase == PhaseMain {
		awardControlledRegionPoints(&working)
		working.Turn.TurnNumber++
		evaluateWinner(&working)
		working.Turn.ActivePlayerID = nextPriorityPlayerID(working, working.Turn.ActivePlayerID)
	}

	resetPriorityWindow(&working.Turn, working.Turn.ActivePlayerID, PriorityWindowAction)
	requestContinuousRecalculation(&working)
	return working
}

func commitState(state GameState, action Action, operation Operation, event Event, projector *ProjectionEngine) SubmitResult {
	committed := cloneGameState(state)

	revision := Revision{
		Number:      committed.Revision.Number + 1,
		ActionID:    action.ID,
		OperationID: operation.ID,
		EventID:     event.ID,
	}

	event.RevisionNumber = revision.Number

	committed.History.Actions = append(committed.History.Actions, action)
	committed.History.Operations = append(committed.History.Operations, operation)
	committed.History.Events = append(committed.History.Events, cloneEvent(event))
	committed.History.Revisions = append(committed.History.Revisions, revision)
	committed.Revision = revision
	if committed.Match.Status == MatchStatusFinished && committed.Match.FinishedAtRevision == 0 {
		committed.Match.FinishedAtRevision = revision.Number
	}
	committed = maybeRecalculateContinuousEffects(committed, revision)
	views := ProjectionBundle{}
	if projector != nil {
		views = projector.Generate(committed)
	}

	result := SubmitResult{
		State:     committed,
		Operation: operation,
		Event:     event,
		Revision:  revision,
		Views:     views,
	}
	result.Accepted = NewActionAccepted(action, operation, event, revision)
	result.Patched = NewStatePatchedForPlayer(views, action.ActorID, event, revision)
	if len(views.Players) != 0 || views.Spectator.GameID != "" {
		result.Dispatch = BuildCommitDispatchBatch(result)
	}

	return result
}

func hasActionID(state GameState, actionID string) bool {
	for _, existing := range state.History.Actions {
		if existing.ID == actionID {
			return true
		}
	}

	return false
}

func hasCardID(state GameState, cardID string) bool {
	return findCardIndex(state, cardID) >= 0
}

func findCardIndex(state GameState, cardID string) int {
	for index, card := range state.Board.Cards {
		if card.CardID == cardID {
			return index
		}
	}

	return -1
}

func phaseState(name PhaseName) PhaseState {
	switch name {
	case PhaseMain:
		return PhaseState{Name: PhaseMain, Step: StepAction, AllowsStack: true, StepEnded: false}
	case PhaseEnd:
		return PhaseState{Name: PhaseEnd, Step: StepAction, AllowsStack: false, StepEnded: false}
	default:
		return PhaseState{Name: name, Step: StepAction, AllowsStack: false, StepEnded: false}
	}
}

func nextPhaseName(current PhaseName) (PhaseName, error) {
	switch current {
	case PhaseMain:
		return PhaseEnd, nil
	case PhaseEnd:
		return PhaseMain, nil
	default:
		return "", fmt.Errorf("unsupported phase %q", current)
	}
}

func currentPriorityPlayerID(state GameState) string {
	if state.Turn.Priority.CurrentPlayerID != "" {
		return state.Turn.Priority.CurrentPlayerID
	}

	return state.Turn.PriorityPlayerID
}

func currentPriorityWindowKind(state GameState) PriorityWindowKind {
	if state.Turn.Priority.WindowKind != "" {
		return state.Turn.Priority.WindowKind
	}

	return PriorityWindowAction
}

func actionRequiresPriority(kind ActionKind) bool {
	switch kind {
	case ActionKindResolveTopStack:
		return false
	default:
		return true
	}
}

func actionRequiresEmptyStack(kind ActionKind) bool {
	switch kind {
	case ActionKindAdvancePhase, ActionKindRevealCard, ActionKindInspectCard, ActionKindDeclareAttack, ActionKindDeclareInvestigation, ActionKindRollSeededRandom:
		return true
	default:
		return false
	}
}

func consecutivePassLimit(state GameState) int {
	if len(state.Players) < 2 {
		return 2
	}

	return len(state.Players)
}

func intString(value int) string {
	return fmt.Sprintf("%d", value)
}

func containsString(values []string, target string) bool {
	for _, value := range values {
		if value == target {
			return true
		}
	}

	return false
}

func checkCardActionPermissionLegality(state GameState, cardID string, kind ActionKind) LegalityResult {
	permission := permissionForActionKind(kind)
	if permission == "" {
		return okLegalityResult()
	}

	index := findCardIndex(state, cardID)
	if index == -1 {
		return okLegalityResult()
	}

	card := state.Board.Cards[index]
	if containsString(card.Prohibitions, permission) {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.legality.action_prohibited",
			"board.cards.prohibitions",
			map[string]string{
				"cardId":     cardID,
				"permission": permission,
				"actionKind": string(kind),
			},
		)
	}

	if containsString(card.RequiredPermissions, permission) && !containsString(card.Permissions, permission) {
		return legalityFailure(
			ReasonCodeLegalityFailedPermissionRequired,
			"rules.legality.permission_required",
			"board.cards.permissions",
			map[string]string{
				"cardId":     cardID,
				"permission": permission,
				"actionKind": string(kind),
			},
		)
	}

	return okLegalityResult()
}

func permissionForActionKind(kind ActionKind) string {
	switch kind {
	case ActionKindInspectCard:
		return "inspect"
	case ActionKindRevealCard:
		return "reveal"
	case ActionKindSetFaceDown:
		return "set_face_down"
	case ActionKindDeclareAttack:
		return "attack"
	case ActionKindDeclareInvestigation:
		return "investigate"
	default:
		return ""
	}
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
	// Build target category from the card being played
	targetCategory := TargetCategory{
		BasicTypes: []string{source.BasicType},
	}

	// Use the prohibition checker to evaluate all active rules
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

func resolveStackedOperation(state GameState, operation Operation) (GameState, Operation, error) {
	switch operation.Kind {
	case OperationKindCardEffect:
		return resolveCardEffect(state, operation)
	default:
		resolved := markOperationResolved(operation)
		return finalizeResolvedOperation(state, resolved), resolved, nil
	}
}

func resolveCardEffect(state GameState, operation Operation) (GameState, Operation, error) {
	if operation.Source == nil {
		return GameState{}, Operation{}, fmt.Errorf("%s", ReasonCodeRulesFailedCardLogicUnavailable)
	}

	switch operation.Source.ExecutionKind {
	case CardExecutionDSL:
		return resolveDSLCardEffect(state, operation)
	case CardExecutionScript:
		return resolveScriptCardEffect(state, operation)
	default:
		return GameState{}, Operation{}, fmt.Errorf("%s", ReasonCodeRulesFailedCardLogicUnavailable)
	}
}

func resolveDSLCardEffect(state GameState, operation Operation) (GameState, Operation, error) {
	if operation.Source == nil {
		return GameState{}, Operation{}, fmt.Errorf("%s", ReasonCodeRulesFailedCardLogicUnavailable)
	}

	working := cloneGameState(state)
	for _, effect := range operation.Source.Effects {
		working = applyDSLEffect(working, operation, effect)
	}

	resolved := markOperationResolved(operation)
	return finalizeResolvedOperation(working, resolved), resolved, nil
}

func resolveScriptCardEffect(state GameState, operation Operation) (GameState, Operation, error) {
	resolved := markOperationResolved(operation)
	return finalizeResolvedOperation(state, resolved), resolved, nil
}

func finalizeResolvedOperation(state GameState, operation Operation) GameState {
	working := cloneGameState(state)
	working.Board.Resolved = append(working.Board.Resolved, operation)
	return working
}

func markOperationResolved(operation Operation) Operation {
	resolved := cloneOperation(operation)
	resolved.Status = OperationStatusResolved
	return resolved
}

func postResolutionWindowKind(state GameState) PriorityWindowKind {
	if len(state.Board.Stack) != 0 {
		return PriorityWindowResponse
	}

	return PriorityWindowAction
}

func lookupCardOperationSource(cardID string) (CardOperationSource, bool, error) {
	catalog, err := internalcontracts.LoadDefaultFixtureCatalog()
	if err != nil {
		return CardOperationSource{}, false, err
	}

	fixture, ok := catalog.Find(cardID)
	if !ok {
		return CardOperationSource{}, false, nil
	}

	return cardOperationSourceFromFixture(fixture), true, nil
}

func cardOperationSourceFromFixture(fixture contractspkg.Fixture) CardOperationSource {
	parsed := contractspkg.ParseFixtureLogic(fixture)
	return CardOperationSource{
		CardID:            parsed.CardID,
		CardName:          parsed.CardName,
		SourcePath:        parsed.SourcePath,
		BasicType:         parsed.BasicType,
		LogicID:           parsed.LogicID,
		Speed:             parsed.Speed,
		TargetKinds:       slices.Clone(parsed.TargetKinds),
		RequiresStack:     parsed.RequiresStack,
		ExecutionKind:     cardExecutionKind(parsed.RequiresScript),
		DurationKind:      parsed.DurationKind,
		ScriptID:          cloneOptionalString(parsed.ScriptID),
		RequiresScript:    parsed.RequiresScript,
		PureDSLExecutable: parsed.PureDSLExecutable,
		Effects:           effectSpecsFromParsed(parsed.Effects),
		EffectKinds:       slices.Clone(parsed.EffectKinds),
	}
}

func cardExecutionKind(requiresScript bool) CardExecutionKind {
	if requiresScript {
		return CardExecutionScript
	}

	return CardExecutionDSL
}

func effectSpecsFromParsed(effects []contractspkg.BasicEffect) []EffectSpec {
	specs := make([]EffectSpec, 0, len(effects))
	for _, effect := range effects {
		specs = append(specs, EffectSpec{
			Kind:      effect.Kind,
			TargetRef: effect.TargetRef,
			Amount:    cloneOptionalInt(effect.Amount),
			Stat:      effect.Stat,
			Keyword:   effect.Keyword,
		})
	}

	return specs
}
