package rules

import "strings"

const (
	bindingEntityAttachmentPrefix = "attachment:"
	bindingEntityCardPrefix       = "card:"
	bindingEntityOperationPrefix  = "operation:"
)

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

func continuousEffectBindingIsStillActive(state GameState, effect ContinuousEffect) bool {
	if effect.BindingEntityID != "" {
		switch {
		case strings.HasPrefix(effect.BindingEntityID, bindingEntityAttachmentPrefix):
			attachmentID := strings.TrimPrefix(effect.BindingEntityID, bindingEntityAttachmentPrefix)
			return attachmentIDStillActive(state, attachmentID)
		case strings.HasPrefix(effect.BindingEntityID, bindingEntityCardPrefix):
			cardID := strings.TrimPrefix(effect.BindingEntityID, bindingEntityCardPrefix)
			return cardEntityStillActive(state, cardID)
		case strings.HasPrefix(effect.BindingEntityID, bindingEntityOperationPrefix):
			// Operation-bound effects intentionally do not depend on source card presence.
			return true
		}
	}

	// Backward-compatible fallback for effects created before BindingEntityID rollout.
	if !continuousEffectAttachmentIsStillActive(state, effect) {
		return false
	}
	return continuousEffectSourceIsStillActive(state, effect)
}

func attachmentIDStillActive(state GameState, attachmentID string) bool {
	if attachmentID == "" {
		return false
	}

	for _, attachment := range state.Board.Attachments.Active {
		if attachment.ID == attachmentID {
			return true
		}
	}

	return false
}

func cardEntityStillActive(state GameState, cardID string) bool {
	index := findCardIndex(state, cardID)
	if index == -1 {
		return false
	}

	card := state.Board.Cards[index]
	return card.Zone == CardZoneTable && !card.Destroyed
}

func bindingEntityForAttachment(attachmentID string) string {
	if attachmentID == "" {
		return ""
	}

	return bindingEntityAttachmentPrefix + attachmentID
}

func bindingEntityForCard(cardID string) string {
	if cardID == "" {
		return ""
	}

	return bindingEntityCardPrefix + cardID
}

func bindingEntityForOperation(operationID string) string {
	if operationID == "" {
		return ""
	}

	return bindingEntityOperationPrefix + operationID
}

func sourceCardEntityID(state GameState, operation Operation) string {
	if operation.CardID == "" {
		return ""
	}

	if findCardIndex(state, operation.CardID) == -1 {
		return ""
	}

	return operation.CardID
}
