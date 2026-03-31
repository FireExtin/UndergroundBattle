package rules

import (
	"testing"
)

func TestAttachmentBuilder_Create_Success(t *testing.T) {
	state := newAttachmentTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "source-card",
			Name:           "Source",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
		{
			CardID:         "target-card",
			Name:           "Target",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
	}

	newState, err := NewAttachment(state).
		From("source-card").
		To("target-card").
		AtRevision(1).
		WithBasicType("附属").
		Create()

	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}

	if len(newState.Board.Attachments.Active) != 1 {
		t.Fatalf("expected 1 attachment, got %d", len(newState.Board.Attachments.Active))
	}

	att := newState.Board.Attachments.Active[0]
	if att.SourceCardID != "source-card" {
		t.Errorf("source = %q, want %q", att.SourceCardID, "source-card")
	}
	if att.TargetCardID != "target-card" {
		t.Errorf("target = %q, want %q", att.TargetCardID, "target-card")
	}
}

func TestAttachmentBuilder_Create_InvalidBasicType(t *testing.T) {
	state := newAttachmentTestState()
	state.Board.Cards = []CardState{
		{
			CardID:  "source-card",
			Zone:    CardZoneTable,
			Revealed: true,
		},
		{
			CardID:  "target-card",
			Zone:    CardZoneTable,
			Revealed: true,
		},
	}

	_, err := NewAttachment(state).
		From("source-card").
		To("target-card").
		AtRevision(1).
		WithBasicType("角色"). // 不是附属
		Create()

	if err == nil {
		t.Fatal("expected error for non-attachment basic type")
	}
}

func TestAttachmentBuilder_Create_SourceNotOnTable(t *testing.T) {
	state := newAttachmentTestState()
	state.Board.Cards = []CardState{
		{
			CardID: "source-card",
			Zone:   CardZoneDiscard, // 不在桌面
		},
		{
			CardID:  "target-card",
			Zone:    CardZoneTable,
			Revealed: true,
		},
	}

	_, err := NewAttachment(state).
		From("source-card").
		To("target-card").
		AtRevision(1).
		WithBasicType("附属").
		Create()

	if err == nil {
		t.Fatal("expected error when source not on table")
	}
}

func TestAttachmentBuilder_Create_TargetDestroyed(t *testing.T) {
	state := newAttachmentTestState()
	state.Board.Cards = []CardState{
		{
			CardID:  "source-card",
			Zone:    CardZoneTable,
			Revealed: true,
		},
		{
			CardID:    "target-card",
			Zone:      CardZoneTable,
			Revealed:  true,
			Destroyed: true, // 已被摧毁
		},
	}

	_, err := NewAttachment(state).
		From("source-card").
		To("target-card").
		AtRevision(1).
		WithBasicType("附属").
		Create()

	if err == nil {
		t.Fatal("expected error when target destroyed")
	}
}

func TestAttachmentBuilder_CanCreate(t *testing.T) {
	state := newAttachmentTestState()
	state.Board.Cards = []CardState{
		{
			CardID:  "source-card",
			Zone:    CardZoneTable,
			Revealed: true,
		},
		{
			CardID:  "target-card",
			Zone:    CardZoneTable,
			Revealed: true,
		},
	}

	builder := NewAttachment(state).
		From("source-card").
		To("target-card").
		AtRevision(1).
		WithBasicType("附属")

	if !builder.CanCreate() {
		t.Fatal("expected CanCreate to return true")
	}

	// Test with invalid target
	builder2 := NewAttachment(state).
		From("source-card").
		To("non-existent").
		AtRevision(1).
		WithBasicType("附属")

	if builder2.CanCreate() {
		t.Fatal("expected CanCreate to return false for non-existent target")
	}
}

func TestAttachmentManager_PruneExpired(t *testing.T) {
	state := newAttachmentTestState()
	state.Board.Cards = []CardState{
		{
			CardID:  "source-valid",
			Zone:    CardZoneTable,
			Revealed: true,
		},
		{
			CardID:  "target-valid",
			Zone:    CardZoneTable,
			Revealed: true,
		},
		{
			CardID:    "source-destroyed",
			Zone:      CardZoneTable,
			Revealed:  true,
			Destroyed: true,
		},
		{
			CardID:  "target-in-discard",
			Zone:    CardZoneDiscard,
			Revealed: true,
		},
	}
	state.Board.Attachments = AttachmentRegistry{
		Active: []Attachment{
			{ID: "att:1", SourceCardID: "source-valid", TargetCardID: "target-valid"},           // 有效
			{ID: "att:2", SourceCardID: "source-destroyed", TargetCardID: "target-valid"},      // 过期（源被摧毁）
			{ID: "att:3", SourceCardID: "source-valid", TargetCardID: "target-in-discard"},     // 过期（目标不在桌面）
		},
		NextAttachmentID: 3,
	}

	manager := NewAttachmentManager(state)
	newState := manager.PruneExpired()

	if len(newState.Board.Attachments.Active) != 1 {
		t.Fatalf("expected 1 attachment after pruning, got %d", len(newState.Board.Attachments.Active))
	}

	if newState.Board.Attachments.Active[0].ID != "att:1" {
		t.Errorf("expected att:1 to remain, got %s", newState.Board.Attachments.Active[0].ID)
	}
}

func TestAttachmentManager_GetAttachmentsForTarget(t *testing.T) {
	state := newAttachmentTestState()
	state.Board.Attachments = AttachmentRegistry{
		Active: []Attachment{
			{ID: "att:1", SourceCardID: "source-1", TargetCardID: "target-a"},
			{ID: "att:2", SourceCardID: "source-2", TargetCardID: "target-a"},
			{ID: "att:3", SourceCardID: "source-3", TargetCardID: "target-b"},
		},
	}

	manager := NewAttachmentManager(state)
	attachments := manager.GetAttachmentsForTarget("target-a")

	if len(attachments) != 2 {
		t.Fatalf("expected 2 attachments for target-a, got %d", len(attachments))
	}
}

func TestAttachmentManager_GetAttachmentsFromSource(t *testing.T) {
	state := newAttachmentTestState()
	state.Board.Attachments = AttachmentRegistry{
		Active: []Attachment{
			{ID: "att:1", SourceCardID: "source-a", TargetCardID: "target-1"},
			{ID: "att:2", SourceCardID: "source-a", TargetCardID: "target-2"},
			{ID: "att:3", SourceCardID: "source-b", TargetCardID: "target-3"},
		},
	}

	manager := NewAttachmentManager(state)
	attachments := manager.GetAttachmentsFromSource("source-a")

	if len(attachments) != 2 {
		t.Fatalf("expected 2 attachments from source-a, got %d", len(attachments))
	}
}

// TestAttachmentCreationFailureStillCreatesContinuousEffect verifies that when
// attachment creation fails (e.g., target destroyed), the continuous effect is
// still created. This is intentional - attachment is optional metadata.
func TestAttachmentCreationFailureStillCreatesContinuousEffect(t *testing.T) {
	state := newAttachmentTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "source-card",
			Name:           "Source",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
		{
			CardID:    "target-card",
			Name:      "Target",
			OwnerID:   "P1",
			Zone:      CardZoneTable,
			Revealed:  true,
			Destroyed: true, // Target is destroyed - attachment should fail
		},
	}

	operation := manualAttachmentOperation(
		"op:test",
		"act:test",
		"P1",
		"source-card",
		"target-card",
	)

	// Call registerContinuousEffect which should create the continuous effect
	// even though attachment creation will fail
	result := registerContinuousEffect(state, operation, EffectSpec{
		Kind:      "addKeyword",
		TargetRef: "selected",
		Keyword:   "testKeyword",
	})

	// Continuous effect should still be created
	if len(result.Board.Continuous.Active) != 1 {
		t.Fatalf("expected 1 continuous effect, got %d", len(result.Board.Continuous.Active))
	}

	// Attachment should NOT be created (target is destroyed)
	if len(result.Board.Attachments.Active) != 0 {
		t.Fatalf("expected 0 attachments (creation failed), got %d", len(result.Board.Attachments.Active))
	}

	// Verify the continuous effect has the correct properties
	effect := result.Board.Continuous.Active[0]
	if effect.SourceCardID != "source-card" {
		t.Errorf("effect source = %q, want %q", effect.SourceCardID, "source-card")
	}
	if effect.TargetCardID != "target-card" {
		t.Errorf("effect target = %q, want %q", effect.TargetCardID, "target-card")
	}
	if effect.Keyword != "testKeyword" {
		t.Errorf("effect keyword = %q, want %q", effect.Keyword, "testKeyword")
	}
}

func newAttachmentTestState() GameState {
	return NewGameState(InitialStateConfig{
		GameID:         "game-attachment",
		ActivePlayerID: "P1",
		Seed:           42,
	})
}
