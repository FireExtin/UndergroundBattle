package rules

import "testing"

// TestAttachmentHostDepartureCleanup 测试：宿主离场时，附属按规则离场
func TestAttachmentHostDepartureCleanup(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-attachment-host-departure",
		ActivePlayerID: "P1",
	})

	// 创建宿主卡片
	hostCard := CardState{
		CardID:       "host-1",
		DefinitionID: "HOST",
		Name:         "宿主卡牌",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Destroyed:    false,
	}

	// 创建附属源卡片（提供 continuous effect）
	sourceCard := CardState{
		CardID:       "source-1",
		DefinitionID: "SOURCE",
		Name:         "附属源",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Destroyed:    false,
	}

	state.Board.Cards = []CardState{hostCard, sourceCard}

	// 创建附属关系：source-1 -> host-1
	state.Board.Attachments.Active = []Attachment{
		{
			ID:           "att-1",
			SourceCardID: "source-1",
			TargetCardID: "host-1",
			HostCardID:   "host-1", // 宿主关系
		},
	}

	// 模拟宿主离场（进入 discard）
	for i := range state.Board.Cards {
		if state.Board.Cards[i].CardID == "host-1" {
			moveCardToDiscard(&state.Board.Cards[i])
			break
		}
	}

	// 调用 pruneExpiredAttachments 清理过期附属
	manager := NewAttachmentManager(state)
	state = manager.PruneExpired()

	// 验证：附属应该被移除
	if len(state.Board.Attachments.Active) != 0 {
		t.Fatalf("expected 0 attachments after host departure, got %d", len(state.Board.Attachments.Active))
	}
}

// TestAttachmentSourceDepartureEffectInvalidation 测试：附属离场时，continuous effect 同步失效
func TestAttachmentSourceDepartureEffectInvalidation(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-attachment-source-departure",
		ActivePlayerID: "P1",
	})

	// 创建宿主卡片
	hostCard := CardState{
		CardID:       "host-1",
		DefinitionID: "HOST",
		Name:         "宿主卡牌",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Destroyed:    false,
	}

	// 创建附属源卡片（提供 continuous effect）
	sourceCard := CardState{
		CardID:       "source-1",
		DefinitionID: "SOURCE",
		Name:         "附属源",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Destroyed:    false,
	}

	state.Board.Cards = []CardState{hostCard, sourceCard}

	// 创建附属关系
	state.Board.Attachments.Active = []Attachment{
		{
			ID:           "att-1",
			SourceCardID: "source-1",
			TargetCardID: "host-1",
			HostCardID:   "host-1",
		},
	}

	// 创建 continuous effect（由 source-1 提供）
	state.Board.Continuous.Active = []ContinuousEffect{
		{
			ID:           "eff-1",
			SourceCardID: "source-1",
			AttachmentID: "att-1",
			TargetCardID: "host-1",
			Layer:        LayerNumeric,
			EffectKind:   "modifyStat",
			Stat:         "defense",
			Amount:       1,
			DurationKind: "permanent",
		},
	}

	// 模拟附属源离场（进入 discard）
	for i := range state.Board.Cards {
		if state.Board.Cards[i].CardID == "source-1" {
			moveCardToDiscard(&state.Board.Cards[i])
			break
		}
	}

	// 调用 pruneExpiredAttachments 清理
	manager := NewAttachmentManager(state)
	state = manager.PruneExpired()

	// 验证：continuous effect 应该被移除
	if len(state.Board.Continuous.Active) != 0 {
		t.Fatalf("expected 0 continuous effects after source departure, got %d", len(state.Board.Continuous.Active))
	}
}

// TestAttachmentTrackingV0NoRegression 测试：现有 Attachment tracking V0 不退化
func TestAttachmentTrackingV0NoRegression(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-attachment-v0",
		ActivePlayerID: "P1",
	})

	// 创建宿主和源卡片
	hostCard := CardState{
		CardID:       "host-1",
		DefinitionID: "HOST",
		Name:         "宿主卡牌",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Destroyed:    false,
	}

	sourceCard := CardState{
		CardID:       "source-1",
		DefinitionID: "SOURCE",
		Name:         "附属源",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Destroyed:    false,
	}

	state.Board.Cards = []CardState{hostCard, sourceCard}

	// 使用 AttachmentBuilder 创建附属（V0 方式，不带 HostCardID）
	builder := NewAttachment(state).
		From("source-1").
		To("host-1").
		AtRevision(1).
		WithBasicType("附属")

	if !builder.CanCreate() {
		t.Fatal("AttachmentBuilder.CanCreate() should return true")
	}

	newState, err := builder.Create()
	if err != nil {
		t.Fatalf("AttachmentBuilder.Create() failed: %v", err)
	}

	// 验证：附属应该被创建
	if len(newState.Board.Attachments.Active) != 1 {
		t.Fatalf("expected 1 attachment, got %d", len(newState.Board.Attachments.Active))
	}

	// 验证：TargetCardID 应该被设置
	att := newState.Board.Attachments.Active[0]
	if att.TargetCardID != "host-1" {
		t.Fatalf("expected TargetCardID = 'host-1', got %q", att.TargetCardID)
	}
}

// TestPruneExpiredAttachmentDoesNotRemoveOtherEffectsFromSameSource ensures we only
// remove effects tied to removed attachment IDs, not every effect from the same source.
func TestPruneExpiredAttachmentDoesNotRemoveOtherEffectsFromSameSource(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-attachment-prune-precision",
		ActivePlayerID: "P1",
	})

	state.Board.Cards = []CardState{
		{
			CardID:    "source-1",
			Kind:      CardKindCharacter,
			Zone:      CardZoneTable,
			OwnerID:   "P1",
			Destroyed: false,
		},
		{
			CardID:    "host-expired",
			Kind:      CardKindCharacter,
			Zone:      CardZoneDiscard, // invalid host/target for attachment
			OwnerID:   "P1",
			Destroyed: true,
		},
		{
			CardID:    "host-active",
			Kind:      CardKindCharacter,
			Zone:      CardZoneTable,
			OwnerID:   "P1",
			Destroyed: false,
		},
	}

	state.Board.Attachments.Active = []Attachment{
		{ID: "att-expired", SourceCardID: "source-1", TargetCardID: "host-expired", HostCardID: "host-expired"},
		{ID: "att-active", SourceCardID: "source-1", TargetCardID: "host-active", HostCardID: "host-active"},
	}

	state.Board.Continuous.Active = []ContinuousEffect{
		// Bound to expired attachment: should be removed.
		{ID: "ce-expired", SourceCardID: "source-1", AttachmentID: "att-expired", TargetCardID: "host-expired"},
		// Bound to active attachment: should stay.
		{ID: "ce-active-attachment", SourceCardID: "source-1", AttachmentID: "att-active", TargetCardID: "host-active"},
		// Not attachment-bound but same source: should also stay.
		{ID: "ce-active-non-attachment", SourceCardID: "source-1", TargetCardID: "host-active"},
	}

	pruned := NewAttachmentManager(state).PruneExpired()

	if len(pruned.Board.Attachments.Active) != 1 {
		t.Fatalf("attachments after prune = %d, want 1", len(pruned.Board.Attachments.Active))
	}
	if pruned.Board.Attachments.Active[0].ID != "att-active" {
		t.Fatalf("remaining attachment = %q, want %q", pruned.Board.Attachments.Active[0].ID, "att-active")
	}

	foundExpired := false
	foundActiveAttachment := false
	foundActiveNonAttachment := false
	for _, effect := range pruned.Board.Continuous.Active {
		switch effect.ID {
		case "ce-expired":
			foundExpired = true
		case "ce-active-attachment":
			foundActiveAttachment = true
		case "ce-active-non-attachment":
			foundActiveNonAttachment = true
		}
	}

	if foundExpired {
		t.Fatal("expired attachment effect should be removed")
	}
	if !foundActiveAttachment {
		t.Fatal("active attachment effect should be kept")
	}
	if !foundActiveNonAttachment {
		t.Fatal("non-attachment effect from same source should not be removed")
	}
}
