package rules

import (
	"fmt"
	"slices"
	"sort"
)

// Purpose: Implements the minimal continuous-effect registry, deterministic layer order, and commit-time recalculation.

func ResolveConflict(a, b ContinuousEffect) *ContinuousEffect {
	return nil
}

func RecalculateContinuousEffects(state GameState) GameState {
	working := cloneGameState(state)
	registry := &working.Board.Continuous
	if registry.InProgress {
		registry.CycleGuardTrips++
		registry.PendingRecalculation = false
		return working
	}

	registry.InProgress = true
	registry.PendingRecalculation = false
	registry.FullRecalculationCount++

	resetDerivedCardState(&working)
	pruneExpiredAttachments(&working)
	pruneExpiredContinuousEffects(&working)

	prodEffects := BuildContinuousEffectsFromTemplates(working, BuildProductionContinuousEffectTemplates())
	existingEffects := cloneContinuousEffects(working.Board.Continuous.Active)
	effects := append(existingEffects, prodEffects...)

	sortContinuousEffects(effects)
	for _, effect := range effects {
		applyContinuousEffect(&working, effect)
	}
	applyDerivedBoardSemantics(&working)

	registry = &working.Board.Continuous
	registry.InProgress = false
	registry.PendingRecalculation = false
	return working
}

func maybeRecalculateContinuousEffects(state GameState, revision Revision) GameState {
	if !state.Board.Continuous.PendingRecalculation {
		return state
	}

	recalculated := RecalculateContinuousEffects(state)
	recalculated.Board.Continuous.LastAppliedRevision = revision.Number
	recalculated.Board.Continuous.PendingRecalculation = false
	return recalculated
}

func requestContinuousRecalculation(state *GameState) {
	if state == nil {
		return
	}

	registry := &state.Board.Continuous
	if registry.InProgress {
		registry.CycleGuardTrips++
		registry.PendingRecalculation = true
		return
	}

	registry.PendingRecalculation = true
}

func registerContinuousEffect(state GameState, operation Operation, effect EffectSpec) GameState {
	if operation.Source == nil {
		return state
	}

	targetCardID := runtimeTargetCardID(operation, effect)
	if targetCardID == "" || findCardIndex(state, targetCardID) == -1 {
		return state
	}

	working := cloneGameState(state)
	attachmentID := ""

	// Attachment tracking must support both real board entities and fixture-only
	// sources such as BQ022, which currently do not materialize as table cards.
	if operation.Source.BasicType == "附属" {
		builder := NewAttachment(working).
			To(targetCardID).
			AtRevision(working.Revision.Number).
			WithBasicType(operation.Source.BasicType).
			FromDefinition(operation.Source.CardID).
			FromOperation(operation.ID)

		if findCardIndex(working, operation.CardID) != -1 {
			builder = builder.From(operation.CardID)
		}

		newState, err := builder.Create()
		if err == nil {
			working = newState
			attachmentID = working.Board.Attachments.Active[len(working.Board.Attachments.Active)-1].ID
		}
	}

	registry := &working.Board.Continuous
	registry.NextTimestamp++

	continuous := ContinuousEffect{
		ID:                fmt.Sprintf("ce:%s:%d", operation.ID, registry.NextTimestamp),
		SourceOperationID: operation.ID,
		SourceCardID:      operation.CardID,
		AttachmentID:      attachmentID,
		ControllerID:      operation.ActorID,
		TargetCardID:      targetCardID,
		Layer:             layerForEffect(effect.Kind),
		EffectKind:        effect.Kind,
		DurationKind:      normalizedContinuousDuration(operation.Source.DurationKind),
		DependencyKey:     nil,
		Timestamp:         registry.NextTimestamp,
		Stat:              effect.Stat,
		Keyword:           effect.Keyword,
	}

	if effect.Amount != nil {
		continuous.Amount = *effect.Amount
	}
	if continuous.DurationKind == "turn" {
		continuous.ExpiresAtTurn = working.Turn.TurnNumber
	}

	registry.Active = append(registry.Active, continuous)

	requestContinuousRecalculation(&working)
	return working
}

func isCardActionAllowed(state GameState, cardID string, permission string) bool {
	index := findCardIndex(state, cardID)
	if index == -1 {
		return false
	}

	card := state.Board.Cards[index]
	if containsString(card.Prohibitions, permission) {
		return false
	}

	if containsString(card.RequiredPermissions, permission) && !containsString(card.Permissions, permission) {
		return false
	}

	if containsString(card.Permissions, permission) {
		return true
	}

	return true
}

func resetDerivedCardState(state *GameState) {
	for index := range state.Board.Cards {
		card := &state.Board.Cards[index]
		card.EffectiveKeywords = slices.Clone(card.PrintedKeywords)
		card.EffectiveStats = card.PrintedStats
		card.Permissions = nil
		card.Prohibitions = nil
		card.CostAdjustment = 0
		card.ActionQuota = 0
	}
}

func pruneExpiredContinuousEffects(state *GameState) {
	registry := &state.Board.Continuous
	if len(registry.Active) == 0 {
		return
	}

	kept := make([]ContinuousEffect, 0, len(registry.Active))
	removed := false
	for _, effect := range registry.Active {
		if effect.DurationKind == "turn" && effect.ExpiresAtTurn > 0 && effect.ExpiresAtTurn < state.Turn.TurnNumber {
			removed = true
			continue
		}
		if !continuousEffectAttachmentIsStillActive(*state, effect) {
			removed = true
			continue
		}
		if !continuousEffectSourceIsStillActive(*state, effect) {
			removed = true
			continue
		}

		kept = append(kept, effect)
	}

	if removed {
		registry.Active = kept
		requestContinuousRecalculation(state)
	}
}

func pruneExpiredAttachments(state *GameState) {
	manager := NewAttachmentManager(*state)
	*state = manager.PruneExpired()
}

func continuousEffectAttachmentIsStillActive(state GameState, effect ContinuousEffect) bool {
	if effect.AttachmentID == "" {
		return true
	}

	for _, attachment := range state.Board.Attachments.Active {
		if attachment.ID == effect.AttachmentID {
			return true
		}
	}

	return false
}

func continuousEffectSourceIsStillActive(state GameState, effect ContinuousEffect) bool {
	if effect.SourceCardID == "" {
		return true
	}

	index := findCardIndex(state, effect.SourceCardID)
	if index == -1 {
		// Some continuous effects come from fixture-only cards that are not represented as board instances.
		return true
	}

	source := state.Board.Cards[index]
	return source.Zone == CardZoneTable && !source.Destroyed
}

func sortContinuousEffects(effects []ContinuousEffect) {
	sort.SliceStable(effects, func(left, right int) bool {
		leftEffect := effects[left]
		rightEffect := effects[right]
		leftOrder := continuousLayerOrder(leftEffect.Layer)
		rightOrder := continuousLayerOrder(rightEffect.Layer)
		if leftOrder != rightOrder {
			return leftOrder < rightOrder
		}

		if leftEffect.Timestamp != rightEffect.Timestamp {
			return leftEffect.Timestamp < rightEffect.Timestamp
		}

		return leftEffect.ID < rightEffect.ID
	})
}

func continuousLayerOrder(layer ContinuousLayer) int {
	switch layer {
	case LayerProhibition:
		return 0
	case LayerPermission:
		return 1
	case LayerCost:
		return 2
	case LayerNumeric:
		return 3
	case LayerActionQuota:
		return 4
	default:
		return 99
	}
}

func applyContinuousEffect(state *GameState, effect ContinuousEffect) {
	index := findCardIndex(*state, effect.TargetCardID)
	if index == -1 {
		return
	}

	card := &state.Board.Cards[index]
	switch effect.Layer {
	case LayerProhibition:
		if effect.EffectKind == "prohibitPermission" && effect.Permission != "" && !containsString(card.Prohibitions, effect.Permission) {
			card.Prohibitions = append(card.Prohibitions, effect.Permission)
		}
	case LayerPermission:
		if effect.EffectKind == "grantPermission" && effect.Permission != "" && !containsString(card.Permissions, effect.Permission) {
			card.Permissions = append(card.Permissions, effect.Permission)
		}
	case LayerCost:
		card.CostAdjustment += effect.Amount
	case LayerNumeric:
		applyNumericLayerEffect(card, effect)
	case LayerActionQuota:
		card.ActionQuota += effect.Amount
	}
}

func applyNumericLayerEffect(card *CardState, effect ContinuousEffect) {
	switch effect.EffectKind {
	case "addKeyword":
		if effect.Keyword != "" && !containsString(card.EffectiveKeywords, effect.Keyword) {
			card.EffectiveKeywords = append(card.EffectiveKeywords, effect.Keyword)
		}
	case "modifyStat":
		switch effect.Stat {
		case "combat":
			card.EffectiveStats.Combat += effect.Amount
		case "defense":
			card.EffectiveStats.Defense += effect.Amount
		case "influence":
			card.EffectiveStats.Influence += effect.Amount
		case "investigation":
			card.EffectiveStats.Investigation += effect.Amount
		}
	}
}

func applyDerivedBoardSemantics(state *GameState) {
	for index := range state.Board.Cards {
		card := &state.Board.Cards[index]
		if card.Zone != CardZoneTable {
			continue
		}

		if card.Counters.Damage >= lethalDefenseThreshold(card.EffectiveStats.Defense) {
			moveCardToDiscard(card)
		}
	}

	refreshAllRegionControl(state)
}

func moveCardToDiscard(card *CardState) {
	card.Destroyed = true
	card.Zone = CardZoneDiscard
	card.Revealed = true
}

func lethalDefenseThreshold(defense int) int {
	if defense <= 0 {
		return 1
	}

	return defense
}

func layerForEffect(effectKind string) ContinuousLayer {
	switch effectKind {
	case "grantPermission":
		return LayerPermission
	case "prohibitPermission":
		return LayerProhibition
	default:
		return LayerNumeric
	}
}

func normalizedContinuousDuration(durationKind string) string {
	switch durationKind {
	case "turn", "permanent":
		return durationKind
	default:
		return "permanent"
	}
}

// BuildContinuousEffectsFromTemplates builds active continuous effects from production templates
// based on the current game state.
func BuildContinuousEffectsFromTemplates(state GameState, templates []ContinuousEffectTemplate) []ContinuousEffect {
	var effects []ContinuousEffect
	timestamp := state.Board.Continuous.NextTimestamp

	for _, sourceCard := range state.Board.Cards {
		for _, template := range templates {
			if !cardMatchesDefinitionAndCondition(sourceCard, template.SourceDefinitionID, template.SourceCondition) {
				continue
			}

			for _, targetCard := range state.Board.Cards {
				if targetCard.Zone != CardZoneTable || targetCard.Destroyed {
					continue
				}

				if !cardMatchesTargetCondition(sourceCard, targetCard, template.TargetCondition) {
					continue
				}

				timestamp++
				effect := ContinuousEffect{
					ID:           fmt.Sprintf("ce:prod:%s:%s:%d", template.SourceDefinitionID, sourceCard.CardID, timestamp),
					SourceCardID: sourceCard.CardID,
					ControllerID: sourceCard.ControllerID,
					TargetCardID: targetCard.CardID,
					Layer:        template.Layer,
					EffectKind:   template.EffectKind,
					DurationKind: template.DurationKind,
					Timestamp:    timestamp,
					Stat:         template.Stat,
					Amount:       template.Amount,
					Keyword:      template.Keyword,
					Permission:   template.Permission,
				}
				effects = append(effects, effect)
			}
		}
	}

	return effects
}
