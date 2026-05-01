package rules

import "slices"

// Purpose: Clones state so the rules pipeline behaves like a pure transform instead of mutating caller-owned snapshots.

func cloneGameState(state GameState) GameState {
	cloned := state
	cloned.Players = slices.Clone(state.Players)
	cloned.Board = cloneBoardState(state.Board)
	cloned.Score = cloneScoreState(state.Score)
	cloned.History = cloneHistoryState(state.History)
	return cloned
}

func cloneBoardState(state BoardState) BoardState {
	cloned := state
	cloned.Stack = cloneOperations(state.Stack)
	cloned.Resolved = cloneOperations(state.Resolved)
	cloned.RandomResults = slices.Clone(state.RandomResults)
	cloned.Cards = cloneCardStates(state.Cards)
	cloned.Continuous = cloneContinuousRegistry(state.Continuous)
	cloned.Attachments = cloneAttachmentRegistry(state.Attachments)
	cloned.Markers = cloneMarkerRegistry(state.Markers)
	cloned.CardMarkers = cloneCardMarkerRegistry(state.CardMarkers)
	return cloned
}

func cloneHistoryState(state HistoryState) HistoryState {
	cloned := state
	cloned.Actions = cloneActions(state.Actions)
	cloned.Operations = cloneOperations(state.Operations)
	cloned.Events = cloneEvents(state.Events)
	cloned.Revisions = slices.Clone(state.Revisions)
	return cloned
}

func cloneActions(actions []Action) []Action {
	cloned := make([]Action, 0, len(actions))
	for _, action := range actions {
		cloned = append(cloned, cloneAction(action))
	}
	return cloned
}

func cloneAction(action Action) Action {
	cloned := action
	cloned.Choices = cloneChoiceRecords(action.Choices)
	return cloned
}

func cloneOperations(operations []Operation) []Operation {
	cloned := make([]Operation, 0, len(operations))
	for _, operation := range operations {
		cloned = append(cloned, cloneOperation(operation))
	}

	return cloned
}

func cloneOperation(operation Operation) Operation {
	cloned := operation
	cloned.Payment = clonePaymentRecord(operation.Payment)
	cloned.Choices = cloneChoiceRecords(operation.Choices)
	cloned.Source = cloneCardOperationSource(operation.Source)
	return cloned
}

func cloneCardOperationSource(source *CardOperationSource) *CardOperationSource {
	if source == nil {
		return nil
	}

	cloned := *source
	cloned.TargetKinds = slices.Clone(source.TargetKinds)
	cloned.Effects = cloneEffectSpecs(source.Effects)
	cloned.EffectKinds = slices.Clone(source.EffectKinds)
	cloned.ScriptID = cloneOptionalString(source.ScriptID)
	return &cloned
}

func cloneEffectSpecs(effects []EffectSpec) []EffectSpec {
	cloned := make([]EffectSpec, 0, len(effects))
	for _, effect := range effects {
		next := effect
		next.Amount = cloneOptionalInt(effect.Amount)
		cloned = append(cloned, next)
	}

	return cloned
}

func cloneEvents(events []Event) []Event {
	cloned := make([]Event, 0, len(events))
	for _, event := range events {
		cloned = append(cloned, cloneEvent(event))
	}
	return cloned
}

func cloneEvent(event Event) Event {
	cloned := event
	cloned.RandomValue = cloneOptionalInt(event.RandomValue)
	cloned.Payment = clonePaymentRecord(event.Payment)
	cloned.Choices = cloneChoiceRecords(event.Choices)
	return cloned
}

func clonePaymentRecord(record *PaymentRecord) *PaymentRecord {
	if record == nil {
		return nil
	}

	cloned := *record
	return &cloned
}

func cloneChoiceRecords(records []ChoiceRecord) []ChoiceRecord {
	if len(records) == 0 {
		return nil
	}

	cloned := make([]ChoiceRecord, 0, len(records))
	for _, record := range records {
		cloned = append(cloned, record)
	}
	return cloned
}

func cloneOptionalInt(value *int) *int {
	if value == nil {
		return nil
	}

	cloned := *value
	return &cloned
}

func cloneOptionalString(value *string) *string {
	if value == nil {
		return nil
	}

	cloned := *value
	return &cloned
}

func cloneCardStates(cards []CardState) []CardState {
	cloned := make([]CardState, 0, len(cards))
	for _, card := range cards {
		next := card
		next.InspectedBy = slices.Clone(card.InspectedBy)
		next.PrintedKeywords = slices.Clone(card.PrintedKeywords)
		next.EffectiveKeywords = slices.Clone(card.EffectiveKeywords)
		next.InfluenceByPlayer = cloneIntMap(card.InfluenceByPlayer)
		next.Permissions = slices.Clone(card.Permissions)
		next.Prohibitions = slices.Clone(card.Prohibitions)
		next.RequiredPermissions = slices.Clone(card.RequiredPermissions)
		cloned = append(cloned, next)
	}

	return cloned
}

func cloneContinuousRegistry(registry ContinuousEffectRegistry) ContinuousEffectRegistry {
	cloned := registry
	cloned.Active = cloneContinuousEffects(registry.Active)
	return cloned
}

func cloneContinuousEffects(effects []ContinuousEffect) []ContinuousEffect {
	cloned := make([]ContinuousEffect, 0, len(effects))
	for _, effect := range effects {
		next := effect
		next.DependencyKey = slices.Clone(effect.DependencyKey)
		cloned = append(cloned, next)
	}

	return cloned
}

func cloneAttachmentRegistry(registry AttachmentRegistry) AttachmentRegistry {
	cloned := registry
	cloned.Active = cloneAttachments(registry.Active)
	return cloned
}

func cloneAttachments(attachments []Attachment) []Attachment {
	cloned := make([]Attachment, 0, len(attachments))
	for _, attachment := range attachments {
		cloned = append(cloned, attachment)
	}

	return cloned
}

func cloneMarkerRegistry(registry MarkerRegistry) MarkerRegistry {
	cloned := registry
	cloned.ByPlayer = cloneNestedIntMap(registry.ByPlayer)
	return cloned
}

func cloneCardMarkerRegistry(registry CardMarkerRegistry) CardMarkerRegistry {
	cloned := registry
	cloned.ByCard = cloneNestedIntMap(registry.ByCard)
	return cloned
}

func cloneScoreState(state ScoreState) ScoreState {
	cloned := state
	cloned.ByPlayer = cloneIntMap(state.ByPlayer)
	return cloned
}

func cloneIntMap(values map[string]int) map[string]int {
	if len(values) == 0 {
		return nil
	}

	cloned := make(map[string]int, len(values))
	for key, value := range values {
		cloned[key] = value
	}

	return cloned
}

func cloneNestedIntMap(values map[string]map[string]int) map[string]map[string]int {
	if len(values) == 0 {
		return nil
	}

	cloned := make(map[string]map[string]int, len(values))
	for key, nested := range values {
		cloned[key] = cloneIntMap(nested)
	}

	return cloned
}
