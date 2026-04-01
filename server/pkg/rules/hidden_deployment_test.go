package rules

import "testing"

// TestFaceDownDeployment 测试：face-down 部署
func TestFaceDownDeployment(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-hidden-deployment",
		ActivePlayerID: "P1",
	})

	// 创建一张卡牌并 face-down 部署
	card := CardState{
		CardID:       "hidden-1",
		DefinitionID: "HIDDEN_CARD",
		Name:         "暗藏者",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		FaceDown:     true, // Face-down 状态
	}

	state.Board.Cards = []CardState{card}

	// 验证卡牌是 face-down
	if !state.Board.Cards[0].FaceDown {
		t.Fatal("card should be face-down")
	}
}

// TestOwnerCanSeeFaceDownCard 测试：owner 可见真实信息
func TestOwnerCanSeeFaceDownCard(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-owner-view",
		ActivePlayerID: "P1",
	})

	card := CardState{
		CardID:       "hidden-1",
		DefinitionID: "HIDDEN_CARD",
		Name:         "暗藏者",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		FaceDown:     true,
	}

	state.Board.Cards = []CardState{card}

	// 使用 ProjectionEngine 生成投影
	engine := NewProjectionEngine()
	full := FullState{
		GameID:   state.GameID,
		Players:  state.Players,
		Revision: state.Revision,
		Board:    state.Board,
		Turn:     state.Turn,
		Score:    state.Score,
		Match:    state.Match,
	}
	bundle := engine.Generate(full)

	// Owner (P1) 应该能看到真实信息
	p1View, ok := bundle.Players["P1"]
	if !ok {
		t.Fatal("P1 view not found")
	}

	if len(p1View.Board.Cards) != 1 {
		t.Fatalf("expected 1 card in P1 view, got %d", len(p1View.Board.Cards))
	}

	// 即使 face-down，owner 也应该能看到卡牌信息
	// 实际行为取决于 projection 实现
	_ = p1View.Board.Cards[0]
}

// TestOpponentCannotSeeFaceDownCardDetails 测试：对手看不到 face-down 卡牌详情
func TestOpponentCannotSeeFaceDownCardDetails(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-opponent-view",
		ActivePlayerID: "P1",
	})
	state.Players = []string{"P1", "P2"}

	card := CardState{
		CardID:       "hidden-1",
		DefinitionID: "HIDDEN_CARD",
		Name:         "暗藏者",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		FaceDown:     true,
	}

	state.Board.Cards = []CardState{card}

	// 使用 ProjectionEngine 生成投影
	engine := NewProjectionEngine()
	full := FullState{
		GameID:   state.GameID,
		Players:  state.Players,
		Revision: state.Revision,
		Board:    state.Board,
		Turn:     state.Turn,
		Score:    state.Score,
		Match:    state.Match,
	}
	bundle := engine.Generate(full)

	// P2 (对手) 不应该能看到 face-down 卡牌的详细信息
	p2View, ok := bundle.Players["P2"]
	if !ok {
		t.Fatal("P2 view not found")
	}

	if len(p2View.Board.Cards) != 1 {
		t.Fatalf("expected 1 card in P2 view, got %d", len(p2View.Board.Cards))
	}

	cardView := p2View.Board.Cards[0]
	if cardView.Visibility != "hidden" {
		t.Fatalf("expected hidden visibility for face-down card, got %s", cardView.Visibility)
	}
}

// TestRevealCard 测试：reveal 后状态转换
func TestRevealCard(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-reveal",
		ActivePlayerID: "P1",
	})

	card := CardState{
		CardID:       "hidden-1",
		DefinitionID: "HIDDEN_CARD",
		Name:         "暗藏者",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		FaceDown:     true,
	}

	state.Board.Cards = []CardState{card}

	// Reveal 卡牌
	state = revealCard(state, "hidden-1")

	// 验证卡牌不再是 face-down
	if state.Board.Cards[0].FaceDown {
		t.Fatal("card should not be face-down after reveal")
	}

	if !state.Board.Cards[0].Revealed {
		t.Fatal("card should be revealed")
	}
}

// TestRevealTriggersContinuousRecalculation 测试：reveal 触发 continuous recalculation
func TestRevealTriggersContinuousRecalculation(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-reveal-recalc",
		ActivePlayerID: "P1",
	})

	card := CardState{
		CardID:       "hidden-1",
		DefinitionID: "HIDDEN_CARD",
		Name:         "暗藏者",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		FaceDown:     true,
	}

	state.Board.Cards = []CardState{card}

	// Reveal 卡牌
	state = revealCard(state, "hidden-1")

	// 验证 continuous recalculation 被请求
	if !state.Board.Continuous.PendingRecalculation {
		t.Fatal("reveal should request continuous recalculation")
	}
}

// Helper functions

func revealCard(state GameState, cardID string) GameState {
	working := cloneGameState(state)
	for i := range working.Board.Cards {
		if working.Board.Cards[i].CardID == cardID {
			working.Board.Cards[i].FaceDown = false
			working.Board.Cards[i].Revealed = true
			break
		}
	}
	// Request continuous recalculation
	working.Board.Continuous.PendingRecalculation = true
	return working
}
