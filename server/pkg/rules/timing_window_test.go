package rules

import "testing"

// TestFastActionAllowedInMainPhase 测试：fast action 在 main phase 允许
func TestFastActionAllowedInMainPhase(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-fast-action",
		ActivePlayerID: "P1",
	})

	// 检查是否可以在 main phase 发动 fast action
	canPlay := canPlayFastAction(state, "P1")
	if !canPlay {
		t.Fatal("fast action should be allowed in main phase")
	}
}

// TestReactionRequiresNonEmptyStack 测试：reaction 需要 stack 非空
func TestReactionRequiresNonEmptyStack(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-reaction",
		ActivePlayerID: "P1",
	})

	// Stack 为空时，reaction 应该被拒绝
	canReact := canPlayReaction(state, "P1")
	if canReact {
		t.Fatal("reaction should not be allowed when stack is empty")
	}

	// 添加一个操作到 stack
	state.Board.Stack = []Operation{
		{ID: "op-1", ActorID: "P2", Kind: "queue_operation"},
	}

	// Stack 非空时，reaction 应该被允许
	canReact = canPlayReaction(state, "P1")
	if !canReact {
		t.Fatal("reaction should be allowed when stack is not empty")
	}
}

// TestReactionAllowedOnOpponentAction 测试：reaction 可以在对手动作时发动
func TestReactionAllowedOnOpponentAction(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-reaction-opponent",
		ActivePlayerID: "P1",
	})

	// P2 的操作在 stack 中
	state.Board.Stack = []Operation{
		{ID: "op-1", ActorID: "P2", Kind: "queue_operation"},
	}

	// P1 应该可以对 P2 的动作做出反应
	canReact := canPlayReaction(state, "P1")
	if !canReact {
		t.Fatal("P1 should be able to react to P2's action")
	}
}

// Helper functions

func canPlayFastAction(state GameState, playerID string) bool {
	// Fast action 可以在 main phase 发动 (Phase.Name == "main")
	return state.Turn.Phase.Name == "main"
}

func canPlayReaction(state GameState, playerID string) bool {
	// Reaction 需要 stack 非空
	return len(state.Board.Stack) > 0
}
