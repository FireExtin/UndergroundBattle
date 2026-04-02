package rules

import (
	"fmt"
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
			CardMarkers: CardMarkerRegistry{
				ByCard: make(map[string]map[string]int),
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
	if engine := CurrentPaymentEngine(); engine != nil {
		engine.Initialize(&state)
	}
	return state
}

// CheckLegality returns a structured machine-readable legality result instead of a plain text error.
func CheckLegality(state GameState, action Action) LegalityResult {
	return checkLegalityWithLookup(state, action, nil)
}

func checkLegalityWithLookup(state GameState, action Action, sourceLookup cardOperationSourceLookup) LegalityResult {
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
		context := map[string]string{
			"endReason": string(state.Match.EndReason),
		}
		if state.Match.WinnerPlayerID != "" {
			context["winnerPlayerId"] = state.Match.WinnerPlayerID
		}
		return legalityFailure(
			ReasonCodeRulesFailedGameAlreadyOver,
			"rules.game.already_over",
			"match.status",
			context,
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

	preflightLegality := checkActionPreflightLegality(state, action)
	if !preflightLegality.OK {
		return preflightLegality
	}

	targetLegality := checkQueueOperationTargetLegality(state, action)
	if !targetLegality.OK {
		return targetLegality
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

		permissionLegality := checkCardActionPermissionLegality(state, action.ActorID, action.CardID, action.Kind)
		if !permissionLegality.OK {
			return permissionLegality
		}

		return okLegalityResult()
	case ActionKindPassPriority:
		return okLegalityResult()
	case ActionKindSetMarker, ActionKindRemoveMarker:
		return checkMarkerActionLegality(state, action)
	case ActionKindSetCardMarker, ActionKindRemoveCardMarker:
		return checkCardMarkerActionLegality(state, action)
	case ActionKindUseFirstPlayerPrivilege:
		return checkFirstPlayerPrivilegeActionLegality(state, action)
	case ActionKindPlayCard:
		return checkPlayCardActionLegality(state, action, sourceLookup)
	case ActionKindBuildAsset:
		return checkBuildAssetActionLegality(state, action)
	case ActionKindMoveCard:
		return checkMoveCardActionLegality(state, action)
	case ActionKindDeclareAttack:
		return checkRoleActionLegality(state, action, CardKindCharacter)
	case ActionKindDeclareInvestigation:
		return checkRoleActionLegality(state, action, CardKindRegion)
	case ActionKindQueueOperation:
		return checkQueueOperationActionLegality(state, action, sourceLookup)
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

		permissionLegality := checkCardActionPermissionLegality(state, action.ActorID, action.CardID, action.Kind)
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
	return buildOperationWithLookup(state, action, nil)
}

func buildOperationWithLookup(state GameState, action Action, sourceLookup cardOperationSourceLookup) (Operation, error) {
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
	case ActionKindSetMarker:
		operation.Kind = OperationKindSetMarker
		operation.MarkerType = action.MarkerType
		operation.MarkerAmount = action.MarkerAmount
	case ActionKindRemoveMarker:
		operation.Kind = OperationKindRemoveMarker
		operation.MarkerType = action.MarkerType
		operation.MarkerAmount = action.MarkerAmount
	case ActionKindSetCardMarker:
		operation.Kind = OperationKindSetCardMarker
		operation.MarkerType = action.MarkerType
		operation.MarkerAmount = action.MarkerAmount
		operation.TargetCardID = action.TargetCardID
	case ActionKindRemoveCardMarker:
		operation.Kind = OperationKindRemoveCardMarker
		operation.MarkerType = action.MarkerType
		operation.MarkerAmount = action.MarkerAmount
		operation.TargetCardID = action.TargetCardID
	case ActionKindMoveCard:
		operation.Kind = OperationKindMoveCard
		operation.CardID = action.CardID
		operation.TargetCardID = action.TargetCardID
		operation.Label = "move_card"
	case ActionKindPlayCard:
		operation.Kind = OperationKindPlayCard
		operation.CardID = action.CardID
		operation.TargetCardID = action.TargetCardID
		operation.TargetPlayerID = action.TargetPlayerID
		operation.TargetRegionCardID = action.TargetRegionCardID
		operation.PlayMode = action.PlayMode
		operation.Label = "play_card"
		if cardIndex := findCardIndex(state, action.CardID); cardIndex >= 0 && cardIndex < len(state.Board.Cards) {
			card := state.Board.Cards[cardIndex]
			lookupID := card.DefinitionID
			if lookupID == "" {
				lookupID = card.CardID
			}
			if lookupID != "" && (card.Kind == CardKindEvent || card.Kind == CardKindAsset) {
				source, found, err := lookupCardOperationSourceWithLookup(sourceLookup, lookupID)
				if err != nil {
					return Operation{}, err
				}
				if found {
					operation.Source = &source
					if card.Kind == CardKindEvent {
						operation.RequiresStack = source.RequiresStack
						if source.CardName != "" {
							operation.Label = source.CardName
						}
					}
				}
			}
		}
	case ActionKindBuildAsset:
		operation.Kind = OperationKindBuildAsset
		operation.CardID = action.CardID
		operation.Label = "build_asset"
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
		if err := buildQueueOperationFromAction(action, sourceLookup, &operation); err != nil {
			return Operation{}, err
		}
	case ActionKindResolveTopStack:
		operation.Kind = OperationKindResolveTopStack
	case ActionKindRollSeededRandom:
		operation.Kind = OperationKindRollRandom
		operation.RandomMax = action.RandomMax
	case ActionKindSetFaceDown:
		operation.Kind = OperationKindSetFaceDown
		operation.CardID = action.CardID
	case ActionKindUseFirstPlayerPrivilege:
		operation.Kind = OperationKindUseFirstPlayerPrivilege
	default:
		return Operation{}, fmt.Errorf("unsupported action kind %q", action.Kind)
	}

	return operation, nil
}

func executeOperation(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)

	if shieldState, shieldOperation, shieldEvent, intercepted := tryResolveShieldInterception(working, operation); intercepted {
		return shieldState, shieldOperation, shieldEvent, nil
	}

	if operation.Kind == OperationKindPlayCard {
		return executePlayCard(working, operation)
	}

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
	case OperationKindSetMarker:
		return executeSetMarker(working, operation)
	case OperationKindRemoveMarker:
		return executeRemoveMarker(working, operation)
	case OperationKindSetCardMarker:
		return executeSetCardMarker(working, operation)
	case OperationKindRemoveCardMarker:
		return executeRemoveCardMarker(working, operation)
	case OperationKindDeclareAttack:
		return executeDeclareAttack(working, operation)
	case OperationKindBuildAsset:
		return executeBuildAsset(working, operation)
	case OperationKindMoveCard:
		return executeMoveCard(working, operation)
	case OperationKindDeclareInvestigation:
		return executeDeclareInvestigation(working, operation)
	case OperationKindCardEffect:
		return executeCardEffect(working, operation)
	case OperationKindResolveTopStack:
		return executeResolveTopStack(working, operation)
	case OperationKindRollRandom:
		return executeRollRandom(working, operation)
	case OperationKindSetFaceDown:
		return executeSetFaceDown(working, operation)
	case OperationKindUseFirstPlayerPrivilege:
		return executeUseFirstPlayerPrivilege(working, operation)
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

	revealFaceDown(&working.Board.Cards[index])
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

	markCardInspected(&working.Board.Cards[index], operation.ActorID)
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
	appendRandomResult(&working, RandomResult{
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

func executeSetFaceDown(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	index := findCardIndex(working, operation.CardID)
	if index == -1 {
		return GameState{}, Operation{}, Event{}, fmt.Errorf("%s", ReasonCodeTargetFailedMissing)
	}

	setFaceDown(&working.Board.Cards[index])
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
		applyEndToMainRulebookFlow(&working, operation)
		if engine := CurrentPaymentEngine(); engine != nil {
			engine.RefillForTurn(&working)
		}
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
	case ActionKindAdvancePhase,
		ActionKindRevealCard,
		ActionKindInspectCard,
		ActionKindSetFaceDown,
		ActionKindUseFirstPlayerPrivilege,
		ActionKindBuildAsset,
		ActionKindMoveCard,
		ActionKindDeclareAttack,
		ActionKindDeclareInvestigation,
		ActionKindRollSeededRandom:
		return true
	case ActionKindSetMarker, ActionKindRemoveMarker, ActionKindSetCardMarker, ActionKindRemoveCardMarker:
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

func resolveStackedOperation(state GameState, operation Operation) (GameState, Operation, error) {
	switch operation.Kind {
	case OperationKindCardEffect:
		return resolveCardEffect(state, operation)
	case OperationKindPlayCard:
		return resolveStackedPlayCard(state, operation)
	default:
		resolved := markOperationResolved(operation)
		return finalizeResolvedOperation(state, resolved), resolved, nil
	}
}

func postResolutionWindowKind(state GameState) PriorityWindowKind {
	if len(state.Board.Stack) != 0 {
		return PriorityWindowResponse
	}

	return PriorityWindowAction
}
