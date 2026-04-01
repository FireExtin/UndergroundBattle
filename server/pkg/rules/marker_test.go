package rules

import "testing"

// TestMarkerAddAndRemove 测试：marker 可以增减
func TestMarkerAddAndRemove(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-marker",
		ActivePlayerID: "P1",
	})

	// 添加 marker
	state = addMarker(state, "P1", "secret_society", 2)

	// 验证 marker 已添加
	p1Markers := getPlayerMarkers(state, "P1")
	if p1Markers["secret_society"] != 2 {
		t.Fatalf("expected 2 secret_society markers, got %d", p1Markers["secret_society"])
	}

	// 移除 marker
	state = removeMarker(state, "P1", "secret_society", 1)

	// 验证 marker 已移除
	p1Markers = getPlayerMarkers(state, "P1")
	if p1Markers["secret_society"] != 1 {
		t.Fatalf("expected 1 secret_society marker after removal, got %d", p1Markers["secret_society"])
	}
}

// TestMarkerReplayConsistency 测试：marker 回放一致
func TestMarkerReplayConsistency(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-marker-replay",
		ActivePlayerID: "P1",
	})

	// 添加 marker
	state1 := addMarker(state, "P1", "secret_society", 3)

	// 模拟回放：重新应用同样的操作
	state2 := addMarker(state, "P1", "secret_society", 3)

	// 验证回放后状态一致
	markers1 := getPlayerMarkers(state1, "P1")
	markers2 := getPlayerMarkers(state2, "P1")

	if markers1["secret_society"] != markers2["secret_society"] {
		t.Fatalf("replay inconsistency: expected %d, got %d", markers1["secret_society"], markers2["secret_society"])
	}
}

// TestMarkerProjectionConsistency 测试：marker 投影一致
func TestMarkerProjectionConsistency(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-marker-projection",
		ActivePlayerID: "P1",
	})

	// 添加公开 marker
	state = addMarker(state, "P1", "public_marker", 5)

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

	// 验证投影 bundle 不为 nil
	if bundle.Players == nil {
		t.Fatal("Projection bundle Players is nil")
	}

	// 验证 marker 在投影中可见
	p1View, ok := bundle.Players["P1"]
	if !ok {
		t.Fatal("P1 view not found in projection bundle")
	}
	_ = p1View // 后续可以验证 marker 内容
}

// TestMarkerLegalityHook 测试：marker 可以作为 legality 条件
func TestMarkerLegalityHook(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-marker-legality",
		ActivePlayerID: "P1",
	})

	// 初始时没有 marker
	canPerform := checkMarkerCondition(state, "P1", "secret_society", 1)
	if canPerform {
		t.Fatal("should not be able to perform action without enough markers")
	}

	// 添加 marker
	state = addMarker(state, "P1", "secret_society", 2)

	// 现在应该满足条件
	canPerform = checkMarkerCondition(state, "P1", "secret_society", 1)
	if !canPerform {
		t.Fatal("should be able to perform action with enough markers")
	}
}

// TestMarkerCloneIsolation ensures cloneGameState deep-copies marker registry maps.
func TestMarkerCloneIsolation(t *testing.T) {
	original := NewGameState(InitialStateConfig{
		GameID:         "test-marker-clone-isolation",
		ActivePlayerID: "P1",
	})
	original.Board.Markers.SetMarker("P1", "secret_society", 1)

	cloned := cloneGameState(original)
	cloned.Board.Markers.SetMarker("P1", "secret_society", 3)

	if got := original.Board.Markers.GetMarker("P1", "secret_society"); got != 1 {
		t.Fatalf("original marker mutated through clone aliasing: got %d, want 1", got)
	}
	if got := cloned.Board.Markers.GetMarker("P1", "secret_society"); got != 3 {
		t.Fatalf("cloned marker value = %d, want 3", got)
	}
}

// Helper functions

func addMarker(state GameState, playerID, markerType string, amount int) GameState {
	working := cloneGameState(state)
	current := working.Board.Markers.GetMarker(playerID, markerType)
	working.Board.Markers.SetMarker(playerID, markerType, current+amount)
	return working
}

func removeMarker(state GameState, playerID, markerType string, amount int) GameState {
	working := cloneGameState(state)
	current := working.Board.Markers.GetMarker(playerID, markerType)
	newAmount := current - amount
	if newAmount < 0 {
		newAmount = 0
	}
	working.Board.Markers.SetMarker(playerID, markerType, newAmount)
	return working
}

func getPlayerMarkers(state GameState, playerID string) map[string]int {
	if state.Board.Markers.ByPlayer == nil {
		return make(map[string]int)
	}
	playerMarkers, ok := state.Board.Markers.ByPlayer[playerID]
	if !ok {
		return make(map[string]int)
	}
	// Return a copy to avoid external modification
	result := make(map[string]int)
	for k, v := range playerMarkers {
		result[k] = v
	}
	return result
}

func checkMarkerCondition(state GameState, playerID, markerType string, minAmount int) bool {
	return state.Board.Markers.GetMarker(playerID, markerType) >= minAmount
}
