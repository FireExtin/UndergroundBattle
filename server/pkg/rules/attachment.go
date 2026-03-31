package rules

import (
	"fmt"
)

// Purpose: Implements the attachment lifecycle management with builder pattern.

// AttachmentBuilder provides a fluent interface for creating attachments.
type AttachmentBuilder struct {
	state      GameState
	sourceID   string
	targetID   string
	revision   int
	basicType  string
}

// NewAttachment creates a new AttachmentBuilder.
func NewAttachment(state GameState) *AttachmentBuilder {
	return &AttachmentBuilder{
		state: state,
	}
}

// From sets the source card ID.
func (b *AttachmentBuilder) From(sourceID string) *AttachmentBuilder {
	b.sourceID = sourceID
	return b
}

// To sets the target card ID.
func (b *AttachmentBuilder) To(targetID string) *AttachmentBuilder {
	b.targetID = targetID
	return b
}

// AtRevision sets the revision number.
func (b *AttachmentBuilder) AtRevision(revision int) *AttachmentBuilder {
	b.revision = revision
	return b
}

// WithBasicType sets the basic type (must be "附属" for attachments).
func (b *AttachmentBuilder) WithBasicType(basicType string) *AttachmentBuilder {
	b.basicType = basicType
	return b
}

// CanCreate checks if the attachment can be created without modifying state.
func (b *AttachmentBuilder) CanCreate() bool {
	// Must be an attachment type
	if b.basicType != "附属" {
		return false
	}

	// Validate source card
	if !b.isCardValid(b.sourceID) {
		return false
	}

	// Validate target card
	if !b.isCardValid(b.targetID) {
		return false
	}

	return true
}

// Create creates the attachment if valid and returns the new state.
// Returns error if attachment cannot be created.
func (b *AttachmentBuilder) Create() (GameState, error) {
	if !b.CanCreate() {
		return b.state, fmt.Errorf("cannot create attachment: invalid source or target")
	}

	working := cloneGameState(b.state)
	registry := &working.Board.Attachments
	registry.NextAttachmentID++

	attachment := Attachment{
		ID:                fmt.Sprintf("att:%d", registry.NextAttachmentID),
		SourceCardID:      b.sourceID,
		TargetCardID:      b.targetID,
		CreatedAtRevision: b.revision,
	}

	registry.Active = append(registry.Active, attachment)
	return working, nil
}

// isCardValid checks if a card is valid for attachment (on table and not destroyed).
func (b *AttachmentBuilder) isCardValid(cardID string) bool {
	index := findCardIndex(b.state, cardID)
	if index == -1 {
		return false
	}
	card := b.state.Board.Cards[index]
	return card.Zone == CardZoneTable && !card.Destroyed
}

// AttachmentManager provides high-level attachment operations.
type AttachmentManager struct {
	state GameState
}

// NewAttachmentManager creates a new AttachmentManager.
func NewAttachmentManager(state GameState) *AttachmentManager {
	return &AttachmentManager{state: state}
}

// PruneExpired removes attachments where source or target is no longer valid.
func (am *AttachmentManager) PruneExpired() GameState {
	working := cloneGameState(am.state)
	registry := &working.Board.Attachments

	if len(registry.Active) == 0 {
		return working
	}

	kept := make([]Attachment, 0, len(registry.Active))
	for _, attachment := range registry.Active {
		if am.isAttachmentStillActive(attachment) {
			kept = append(kept, attachment)
		}
	}

	registry.Active = kept
	return working
}

// isAttachmentStillActive checks if both source and target cards are still valid.
func (am *AttachmentManager) isAttachmentStillActive(attachment Attachment) bool {
	return am.isCardValid(attachment.SourceCardID) && am.isCardValid(attachment.TargetCardID)
}

// isCardValid checks if a card is valid (on table and not destroyed).
func (am *AttachmentManager) isCardValid(cardID string) bool {
	index := findCardIndex(am.state, cardID)
	if index == -1 {
		return false
	}
	card := am.state.Board.Cards[index]
	return card.Zone == CardZoneTable && !card.Destroyed
}

// GetAttachmentsForTarget returns all attachments targeting a specific card.
func (am *AttachmentManager) GetAttachmentsForTarget(targetID string) []Attachment {
	var result []Attachment
	for _, attachment := range am.state.Board.Attachments.Active {
		if attachment.TargetCardID == targetID {
			result = append(result, attachment)
		}
	}
	return result
}

// GetAttachmentsFromSource returns all attachments from a specific source card.
func (am *AttachmentManager) GetAttachmentsFromSource(sourceID string) []Attachment {
	var result []Attachment
	for _, attachment := range am.state.Board.Attachments.Active {
		if attachment.SourceCardID == sourceID {
			result = append(result, attachment)
		}
	}
	return result
}
