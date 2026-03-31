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

	// 直接从 result 中查找卡片
	var target *CardState
	for i := range result.Board.Cards {
		if result.Board.Cards[i].CardID == "target-1" {
			target = &result.Board.Cards[i]
			break
		}
	}
	if target == nil {
		t.Fatal("target card should still exist after discardCard effect")
	}
	if target.Zone != CardZoneDiscard {
		t.Fatalf("target.Zone = %q, want %q", target.Zone, CardZoneDiscard)
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

	// 直接从 state 中查找卡片
	var target *CardState
	for i := range state.Board.Cards {
		if state.Board.Cards[i].CardID == "target-1" {
			target = &state.Board.Cards[i]
			break
		}
	}
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
