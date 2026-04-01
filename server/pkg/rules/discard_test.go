package rules

import "testing"

func TestDiscardCardDSLEffectExists(t *testing.T) {
	// 红测：discardCard DSL effect 应该能被处理
	state := NewGameState(InitialStateConfig{
		GameID:         "test-discard-dsl",
		ActivePlayerID: "P1",
	})

	targetCard := CardState{
		CardID:       "target-1",
		DefinitionID: "TARGET",
		Name:         "目标卡牌",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Destroyed:    false,
	}

	state.Board.Cards = []CardState{targetCard}

	effect := EffectSpec{
		Kind:      "discardCard",
		TargetRef: "selected",
	}

	operation := Operation{
		ID:           "op-test-discard",
		TargetCardID: "target-1",
		ActorID:      "P1",
	}

	result := applyDSLEffect(state, operation, effect)

	target := findCardByID(result, "target-1")
	if target == nil {
		t.Fatal("target card should still exist after discardCard effect")
	}
	if target.Zone != CardZoneDiscard {
		t.Fatalf("target.Zone = %q, want %q", target.Zone, CardZoneDiscard)
	}
	if !result.Board.Continuous.PendingRecalculation {
		t.Fatal("discardCard should request continuous recalculation")
	}
}

func findCardByID(state GameState, cardID string) *CardState {
	for i := range state.Board.Cards {
		if state.Board.Cards[i].CardID == cardID {
			return &state.Board.Cards[i]
		}
	}
	return nil
}

func TestLethalDamageUsesMoveCardToDiscard(t *testing.T) {
	// 红测：致命伤害应该使用 moveCardToDiscard
	state := NewGameState(InitialStateConfig{
		GameID:         "test-lethal-discard",
		ActivePlayerID: "P1",
	})

	targetCard := CardState{
		CardID:         "target-1",
		DefinitionID:   "TARGET",
		Name:           "目标卡牌",
		Kind:           CardKindCharacter,
		OwnerID:        "P1",
		Zone:           CardZoneTable,
		Destroyed:      false,
		Counters:       CardCounters{Damage: 3},
		PrintedStats:   CardNumericStats{Defense: 2},
		EffectiveStats: CardNumericStats{Defense: 2},
	}

	state.Board.Cards = []CardState{targetCard}

	// 模拟 applyDerivedBoardSemantics 的调用
	applyDerivedBoardSemantics(&state)

	target := findCardByID(state, "target-1")
	if target == nil {
		t.Fatal("target card should still exist after lethal damage")
	}
	if target.Zone != CardZoneDiscard {
		t.Fatalf("target.Zone = %q, want %q", target.Zone, CardZoneDiscard)
	}
	if !target.Destroyed {
		t.Fatal("target.Destroyed = false, want true")
	}
	if !target.Revealed {
		t.Fatal("target.Revealed = false, want true")
	}
}

func TestDiscardCardDSLEffectNoopForNonTableTarget(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-discard-dsl-non-table",
		ActivePlayerID: "P1",
	})

	targetCard := CardState{
		CardID:       "target-1",
		DefinitionID: "TARGET",
		Name:         "手牌目标",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneHand,
		Destroyed:    false,
		Revealed:     false,
	}
	state.Board.Cards = []CardState{targetCard}

	result := applyDSLEffect(state, Operation{
		ID:           "op-test-discard-non-table",
		TargetCardID: "target-1",
		ActorID:      "P1",
	}, EffectSpec{
		Kind:      "discardCard",
		TargetRef: "selected",
	})

	target := findCardByID(result, "target-1")
	if target == nil {
		t.Fatal("target card should still exist")
	}
	if target.Zone != CardZoneHand {
		t.Fatalf("target.Zone = %q, want %q (non-table target should be noop)", target.Zone, CardZoneHand)
	}
	if target.Destroyed {
		t.Fatal("target.Destroyed = true, want false for non-table noop")
	}
	if target.Revealed {
		t.Fatal("target.Revealed = true, want false for non-table noop")
	}
	if result.Board.Continuous.PendingRecalculation {
		t.Fatal("non-table noop should not request continuous recalculation")
	}
}

func TestDiscardCardDSLEffectTriggersContinuousCleanupForTemplateEffects(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-discard-dsl-cleanup",
		ActivePlayerID: "P1",
	})
	state.Board.Cards = []CardState{
		{
			CardID:          "xq31-1",
			DefinitionID:    "XQ31",
			Name:            "莫兰大主教",
			Kind:            CardKindCharacter,
			OwnerID:         "P1",
			ControllerID:    "P1",
			Zone:            CardZoneTable,
			PrintedKeywords: []string{"领袖", "公开", "声望"},
			PrintedStats:    CardNumericStats{Combat: 1, Defense: 4},
		},
		{
			CardID:          "ally-1",
			DefinitionID:    "ALLY",
			Name:            "声望盟友",
			Kind:            CardKindCharacter,
			OwnerID:         "P1",
			ControllerID:    "P1",
			Zone:            CardZoneTable,
			PrintedKeywords: []string{"声望"},
			PrintedStats:    CardNumericStats{Combat: 1, Defense: 2},
		},
	}

	recalculated := RecalculateContinuousEffects(state)
	if card := findCardByID(recalculated, "ally-1"); card == nil || card.EffectiveStats.Defense != 3 {
		t.Fatalf("expected ally defense buff before discard, got %#v", card)
	}

	discarded := applyDSLEffect(recalculated, Operation{
		ID:           "op-test-discard-cleanup",
		TargetCardID: "xq31-1",
		ActorID:      "P1",
	}, EffectSpec{
		Kind:      "discardCard",
		TargetRef: "selected",
	})

	if !discarded.Board.Continuous.PendingRecalculation {
		t.Fatal("discarding a source card should request continuous recalculation")
	}

	committed := maybeRecalculateContinuousEffects(discarded, Revision{Number: 1})
	if card := findCardByID(committed, "ally-1"); card == nil || card.EffectiveStats.Defense != 2 {
		t.Fatalf("expected ally defense buff removed after source discard, got %#v", card)
	}
}
