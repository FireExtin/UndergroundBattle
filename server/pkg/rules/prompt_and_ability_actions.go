package rules

import "fmt"

// Purpose: Implements prompt resolution, face-down reveal, and explicit activated abilities.

type AbilitySpeed string

const (
	AbilitySpeedAction AbilitySpeed = "action"
	AbilitySpeedQuick  AbilitySpeed = "quick"
)

type AbilityCost struct {
	Resource int  `json:"resource,omitempty"`
	Exhaust  bool `json:"exhaust,omitempty"`
}

type AbilityDefinition struct {
	ID                 string       `json:"id"`
	SourceDefinitionID string       `json:"sourceDefinitionId"`
	Speed              AbilitySpeed `json:"speed"`
	RequiresStack      bool         `json:"requiresStack"`
	Cost               AbilityCost  `json:"cost"`
}

var defaultAbilityRegistry = map[string]AbilityDefinition{
	"JC003.quick.exhaust_target": {
		ID:                 "JC003.quick.exhaust_target",
		SourceDefinitionID: "JC003",
		Speed:              AbilitySpeedQuick,
		RequiresStack:      true,
		Cost: AbilityCost{
			Resource: 2,
			Exhaust:  true,
		},
	},
}

func lookupAbilityDefinition(abilityID string) (AbilityDefinition, bool) {
	ability, ok := defaultAbilityRegistry[abilityID]
	return ability, ok
}

func checkResolvePromptActionLegality(state GameState, action Action) LegalityResult {
	if state.Turn.PendingPrompt == nil {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.prompt.missing",
			"turn.pendingPrompt",
			nil,
		)
	}
	if action.PromptID == "" || state.Turn.PendingPrompt.ID != action.PromptID {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.prompt.id_mismatch",
			"action.promptId",
			nil,
		)
	}
	if state.Turn.PendingPrompt.OwnerPlayerID != action.ActorID {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.prompt.owner_required",
			"turn.pendingPrompt.ownerPlayerId",
			nil,
		)
	}

	switch state.Turn.PendingPrompt.Kind {
	case PromptKindInvestigationReward:
		return checkInvestigationPromptOrdering(state.Turn.PendingPrompt, action.TopCardIDs, action.BottomCardIDs)
	case PromptKindBattleDamage:
		return checkBattleDamageAssignmentsLegality(*state.Turn.PendingPrompt, action.DamageAssignments)
	default:
		return okLegalityResult()
	}
}

func checkInvestigationPromptOrdering(prompt *PromptState, topCardIDs []string, bottomCardIDs []string) LegalityResult {
	if prompt == nil {
		return legalityFailure(ReasonCodeLegalityFailedActionProhibited, "rules.prompt.missing", "turn.pendingPrompt", nil)
	}
	seen := make(map[string]int, len(prompt.PeekCardIDs))
	for _, cardID := range topCardIDs {
		seen[cardID]++
	}
	for _, cardID := range bottomCardIDs {
		seen[cardID]++
	}
	for _, cardID := range prompt.PeekCardIDs {
		if seen[cardID] != 1 {
			return legalityFailure(
				ReasonCodeLegalityFailedActionProhibited,
				"rules.prompt.order_invalid",
				"action.topCardIds",
				map[string]string{"cardId": cardID},
			)
		}
	}
	if len(seen) != len(prompt.PeekCardIDs) {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.prompt.order_invalid",
			"action.topCardIds",
			nil,
		)
	}
	return okLegalityResult()
}

func checkBattleDamageAssignmentsLegality(prompt PromptState, assignments []DamageAssignment) LegalityResult {
	total := 0
	allowed := make(map[string]struct{}, len(prompt.EligibleTargetIDs))
	for _, targetID := range prompt.EligibleTargetIDs {
		allowed[targetID] = struct{}{}
	}
	for _, assignment := range assignments {
		if assignment.Amount <= 0 {
			return legalityFailure(ReasonCodeLegalityFailedActionProhibited, "rules.prompt.damage_invalid", "action.damageAssignments", nil)
		}
		if _, ok := allowed[assignment.TargetCardID]; !ok {
			return legalityFailure(ReasonCodeLegalityFailedActionProhibited, "rules.prompt.target_invalid", "action.damageAssignments", nil)
		}
		total += assignment.Amount
	}
	if total != prompt.RemainingAmount {
		return legalityFailure(ReasonCodeLegalityFailedActionProhibited, "rules.prompt.damage_total_invalid", "action.damageAssignments", nil)
	}
	return okLegalityResult()
}

func executeResolvePrompt(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	prompt := working.Turn.PendingPrompt
	if prompt == nil {
		return GameState{}, Operation{}, Event{}, fmt.Errorf("%s", ReasonCodeLegalityFailedActionProhibited)
	}

	switch prompt.Kind {
	case PromptKindInvestigationReward:
		reorderPeekedCardsAndDraw(&working, prompt.OwnerPlayerID, prompt.PeekCardIDs, operation.TopCardIDs, operation.BottomCardIDs)
		setConflictStage(&working, ConflictStagePostInvestigationFast)
	case PromptKindBattleDamage:
		for _, assignment := range operation.DamageAssignments {
			index := findCardIndex(working, assignment.TargetCardID)
			if index == -1 {
				continue
			}
			addDamageCounter(&working.Board.Cards[index], assignment.Amount)
		}
		requestContinuousRecalculation(&working)
		setConflictStage(&working, ConflictStagePostBattleFast)
	default:
		return GameState{}, Operation{}, Event{}, fmt.Errorf("%s", ReasonCodeLegalityFailedActionProhibited)
	}

	working.Turn.PendingPrompt = nil
	working.Turn.Conflict.PendingPromptID = ""
	operation.Status = OperationStatusResolved

	return working, operation, Event{
		ID:               "evt:" + operation.ActionID,
		ActionID:         operation.ActionID,
		OperationID:      operation.ID,
		Kind:             EventKindPromptResolved,
		Phase:            working.Turn.Phase.Name,
		Step:             working.Turn.Phase.Step,
		PriorityPlayerID: currentPriorityPlayerID(working),
		PriorityWindow:   currentPriorityWindowKind(working),
		PassCount:        working.Turn.Priority.PassCount,
		StackDepth:       len(working.Board.Stack),
		ResolvedTargetID: operation.PromptID,
		RevisionNumber:   0,
	}, nil
}

func checkRevealFaceDownActionLegality(state GameState, action Action) LegalityResult {
	if action.CardID == "" {
		return legalityFailure(ReasonCodeTargetFailedMissing, "rules.target.card_missing", "action.cardId", nil)
	}
	index := findCardIndex(state, action.CardID)
	if index == -1 {
		return legalityFailure(ReasonCodeTargetFailedMissing, "rules.target.card_missing", "board.cards", map[string]string{"cardId": action.CardID})
	}
	card := state.Board.Cards[index]
	if card.OwnerID != action.ActorID || card.Zone != CardZoneTable || card.Destroyed || !card.FaceDown {
		return legalityFailure(ReasonCodeLegalityFailedActionProhibited, "rules.reveal_face_down.prohibited", "board.cards", map[string]string{"cardId": action.CardID})
	}

	required := effectivePlayCardCost(card)
	pool := currentPlayerResource(state, action.ActorID)
	if pool.Current < required {
		return legalityFailure(
			ReasonCodeCostFailedUnpaid,
			"rules.cost.unpaid",
			"turn.resources",
			map[string]string{
				"actorId":  action.ActorID,
				"cardId":   action.CardID,
				"required": intString(required),
				"current":  intString(pool.Current),
			},
		)
	}

	return checkPlayCardLoyalty(state, Action{ActorID: action.ActorID}, card)
}

func executeRevealFaceDown(state GameState, operation Operation) (GameState, Operation, Event, error) {
	working := cloneGameState(state)
	index := findCardIndex(working, operation.CardID)
	if index == -1 {
		return GameState{}, Operation{}, Event{}, fmt.Errorf("%s", ReasonCodeTargetFailedMissing)
	}

	required := effectivePlayCardCost(working.Board.Cards[index])
	if !payPlayerResourceCost(&working, operation.ActorID, required) {
		return GameState{}, Operation{}, Event{}, &LegalityError{
			Result: legalityFailure(ReasonCodeCostFailedUnpaid, "rules.cost.unpaid", "turn.resources", nil),
			Code:   ReasonCodeCostFailedUnpaid,
		}
	}

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

func resolveStackedRevealFaceDown(state GameState, operation Operation) (GameState, Operation, error) {
	working := cloneGameState(state)
	index := findCardIndex(working, operation.CardID)
	if index == -1 {
		return GameState{}, Operation{}, fmt.Errorf("%s", ReasonCodeTargetFailedMissing)
	}
	revealFaceDown(&working.Board.Cards[index])
	resolved := markOperationResolved(operation)
	return finalizeResolvedOperation(working, resolved), resolved, nil
}

func checkActivateAbilityActionLegality(state GameState, action Action) LegalityResult {
	if action.CardID == "" || action.AbilityID == "" {
		return legalityFailure(ReasonCodeTargetFailedMissing, "rules.target.card_missing", "action.cardId", nil)
	}
	ability, ok := lookupAbilityDefinition(action.AbilityID)
	if !ok {
		return legalityFailure(ReasonCodeRulesFailedUnknownActionKind, "rules.ability.unknown", "action.abilityId", nil)
	}
	index := findCardIndex(state, action.CardID)
	if index == -1 {
		return legalityFailure(ReasonCodeTargetFailedMissing, "rules.target.card_missing", "board.cards", map[string]string{"cardId": action.CardID})
	}
	card := state.Board.Cards[index]
	if card.OwnerID != action.ActorID || card.Zone != CardZoneTable || card.Destroyed || card.DefinitionID != ability.SourceDefinitionID {
		return legalityFailure(ReasonCodeLegalityFailedActionProhibited, "rules.ability.source_invalid", "board.cards", nil)
	}
	if ability.Cost.Exhaust && card.Exhausted {
		return legalityFailure(ReasonCodeLegalityFailedActionProhibited, "rules.ability.source_exhausted", "board.cards", nil)
	}
	if ability.Speed == AbilitySpeedAction && currentPriorityWindowKind(state) != PriorityWindowAction {
		return legalityFailure(ReasonCodeLegalityFailedActionWindowRequired, "rules.legality.action_window_required", "turn.priority.window", nil)
	}
	if ability.Speed == AbilitySpeedQuick && currentPriorityWindowKind(state) == PriorityWindowClosed {
		return legalityFailure(ReasonCodeLegalityFailedActionProhibited, "rules.ability.window_closed", "turn.priority.window", nil)
	}
	pool := currentPlayerResource(state, action.ActorID)
	if pool.Current < ability.Cost.Resource {
		return legalityFailure(ReasonCodeCostFailedUnpaid, "rules.cost.unpaid", "turn.resources", map[string]string{"required": intString(ability.Cost.Resource), "current": intString(pool.Current)})
	}
	if action.TargetCardID != "" {
		targetIndex := findCardIndex(state, action.TargetCardID)
		if targetIndex == -1 {
			return legalityFailure(ReasonCodeTargetFailedMissing, "rules.target.card_missing", "board.cards", map[string]string{"targetCardId": action.TargetCardID})
		}
		target := state.Board.Cards[targetIndex]
		if target.Zone != CardZoneTable || target.Destroyed || target.Kind != CardKindCharacter {
			return legalityFailure(ReasonCodeTargetFailedProhibited, "rules.target.card_invalid", "action.targetCardId", nil)
		}
	}
	return okLegalityResult()
}

func executeActivateAbility(state GameState, operation Operation) (GameState, Operation, Event, error) {
	ability, ok := lookupAbilityDefinition(operation.AbilityID)
	if !ok {
		return GameState{}, Operation{}, Event{}, fmt.Errorf("%s", ReasonCodeRulesFailedUnknownActionKind)
	}

	working := cloneGameState(state)
	sourceIndex := findCardIndex(working, operation.CardID)
	if sourceIndex == -1 {
		return GameState{}, Operation{}, Event{}, fmt.Errorf("%s", ReasonCodeTargetFailedMissing)
	}
	if !payPlayerResourceCost(&working, operation.ActorID, ability.Cost.Resource) {
		return GameState{}, Operation{}, Event{}, &LegalityError{
			Result: legalityFailure(ReasonCodeCostFailedUnpaid, "rules.cost.unpaid", "turn.resources", nil),
			Code:   ReasonCodeCostFailedUnpaid,
		}
	}
	if ability.Cost.Exhaust {
		exhaustCard(&working.Board.Cards[sourceIndex])
	}

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

func resolveStackedActivatedAbility(state GameState, operation Operation) (GameState, Operation, error) {
	working := cloneGameState(state)

	switch operation.AbilityID {
	case "JC003.quick.exhaust_target":
		index := findCardIndex(working, operation.TargetCardID)
		if index >= 0 {
			target := working.Board.Cards[index]
			if target.Zone == CardZoneTable && !target.Destroyed {
				exhaustCard(&working.Board.Cards[index])
			}
		}
	default:
		return GameState{}, Operation{}, fmt.Errorf("%s", ReasonCodeRulesFailedUnknownActionKind)
	}

	resolved := markOperationResolved(operation)
	return finalizeResolvedOperation(working, resolved), resolved, nil
}
