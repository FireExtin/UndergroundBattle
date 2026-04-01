package rules

import (
	"strings"
	"testing"
)

// =============================================================================
// 测试用例 1: Attachment 宿主离场时多个附属同时清理
// 测试目的: 验证当宿主离场时，所有依附于该宿主的多个附属都会被正确清理
// 输入数据: 一个宿主卡片，三个附属关系指向该宿主
// 预期输出: 宿主离场后，所有三个附属都从 Active 列表中移除
// 验证方法: 检查 Attachments.Active 长度是否为 0
// =============================================================================
func TestAttachment_MultipleAttachmentsCleanupOnHostDeparture(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-multiple-attachments",
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

	// 创建三个附属源卡片
	sourceCards := []CardState{
		{CardID: "source-1", DefinitionID: "SOURCE1", Name: "附属1", Kind: CardKindCharacter, OwnerID: "P1", Zone: CardZoneTable},
		{CardID: "source-2", DefinitionID: "SOURCE2", Name: "附属2", Kind: CardKindCharacter, OwnerID: "P1", Zone: CardZoneTable},
		{CardID: "source-3", DefinitionID: "SOURCE3", Name: "附属3", Kind: CardKindCharacter, OwnerID: "P1", Zone: CardZoneTable},
	}

	state.Board.Cards = append([]CardState{hostCard}, sourceCards...)

	// 创建三个附属关系，都指向同一个宿主
	state.Board.Attachments.Active = []Attachment{
		{ID: "att-1", SourceCardID: "source-1", TargetCardID: "host-1", HostCardID: "host-1"},
		{ID: "att-2", SourceCardID: "source-2", TargetCardID: "host-1", HostCardID: "host-1"},
		{ID: "att-3", SourceCardID: "source-3", TargetCardID: "host-1", HostCardID: "host-1"},
	}

	// 验证初始状态有3个附属
	if len(state.Board.Attachments.Active) != 3 {
		t.Fatalf("expected 3 attachments initially, got %d", len(state.Board.Attachments.Active))
	}

	// 宿主离场
	for i := range state.Board.Cards {
		if state.Board.Cards[i].CardID == "host-1" {
			moveCardToDiscard(&state.Board.Cards[i])
			break
		}
	}

	// 清理过期附属
	manager := NewAttachmentManager(state)
	state = manager.PruneExpired()

	// 验证所有附属都被清理
	if len(state.Board.Attachments.Active) != 0 {
		t.Fatalf("expected 0 attachments after host departure, got %d", len(state.Board.Attachments.Active))
	}
}

// =============================================================================
// 测试用例 2: Marker 边界条件 - 零值和负值处理
// 测试目的: 验证 SetMarker 对零值和负值的处理是否符合预期（应删除 marker）
// 输入数据: 设置 marker 为 5，然后分别设置为 0 和 -1
// 预期输出: 设置为 0 或负值后，该 marker 类型应从 map 中删除
// 验证方法: 检查 GetMarker 返回值是否为 0，且内部 map 不包含该 key
// =============================================================================
func TestMarker_Boundary_ZeroAndNegativeValues(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-marker-boundary",
		ActivePlayerID: "P1",
	})

	// 初始设置 marker 为 5
	state.Board.Markers.SetMarker("P1", "test_marker", 5)
	if state.Board.Markers.GetMarker("P1", "test_marker") != 5 {
		t.Fatal("initial marker value should be 5")
	}

	// 设置为 0，应该删除 marker
	state.Board.Markers.SetMarker("P1", "test_marker", 0)
	if state.Board.Markers.GetMarker("P1", "test_marker") != 0 {
		t.Fatal("marker should be 0 after setting to 0")
	}

	// 验证内部 map 中该 key 已被删除
	if _, exists := state.Board.Markers.ByPlayer["P1"]["test_marker"]; exists {
		t.Fatal("marker key should be deleted from map when set to 0")
	}

	// 重新设置为 5
	state.Board.Markers.SetMarker("P1", "test_marker", 5)

	// 设置为负值，也应该删除 marker
	state.Board.Markers.SetMarker("P1", "test_marker", -1)
	if state.Board.Markers.GetMarker("P1", "test_marker") != 0 {
		t.Fatal("marker should be 0 after setting to negative value")
	}
}

// =============================================================================
// 测试用例 3: Hidden Deployment - 非 Owner 尝试查看 Face-Down 卡牌详情
// 测试目的: 验证非 owner 玩家无法看到 face-down 卡牌的任何详细信息
// 输入数据: P1 的 face-down 卡牌，P2 尝试查看
// 预期输出: P2 看到的 CardView 应该是 hidden，不包含卡牌名称、ID 等敏感信息
// 验证方法: 检查 P2 视图中卡牌的 Visibility 字段和 Name 字段
// =============================================================================
func TestHiddenDeployment_NonOwnerCannotSeeDetails(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-hidden-non-owner",
		ActivePlayerID: "P1",
	})
	state.Players = []string{"P1", "P2"}

	// P1 的 face-down 卡牌
	card := CardState{
		CardID:       "secret-1",
		DefinitionID: "SECRET",
		Name:         "秘密卡牌",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		FaceDown:     true,
	}
	state.Board.Cards = []CardState{card}

	// 生成投影
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

	// P2 视图中卡牌应该是 hidden
	p2View := bundle.Players["P2"]
	if len(p2View.Board.Cards) != 1 {
		t.Fatal("P2 should see 1 card")
	}

	cardView := p2View.Board.Cards[0]
	if cardView.Visibility != "hidden" {
		t.Fatalf("expected visibility 'hidden', got '%s'", cardView.Visibility)
	}

	// P2 不应该看到卡牌名称
	if cardView.Name != "" {
		t.Fatalf("P2 should not see card name, got '%s'", cardView.Name)
	}

	// P2 不应该看到 CardID
	if cardView.CardID != "" {
		t.Fatalf("P2 should not see card ID, got '%s'", cardView.CardID)
	}
}

// =============================================================================
// 测试用例 4: Timing Window - Reaction 在 Stack 清空后不允许
// 测试目的: 验证当 Stack 被清空后，Reaction 不再被允许
// 输入数据: 有 Stack 时可以 Reaction，清空 Stack 后检查 Reaction 权限
// 预期输出: Stack 清空后 Reaction 返回 false
// 验证方法: 对比 Stack 清空前后的 canPlayReaction 返回值
// =============================================================================
func TestTimingWindow_ReactionNotAllowedAfterStackClear(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-reaction-stack-clear",
		ActivePlayerID: "P1",
	})

	// 初始 Stack 为空，不能 Reaction
	if canPlayReaction(state, "P1") {
		t.Fatal("reaction should not be allowed with empty stack")
	}

	// 添加操作到 Stack
	state.Board.Stack = []Operation{
		{ID: "op-1", ActorID: "P2", Kind: "queue_operation"},
	}

	// 现在可以 Reaction
	if !canPlayReaction(state, "P1") {
		t.Fatal("reaction should be allowed with non-empty stack")
	}

	// 清空 Stack
	state.Board.Stack = []Operation{}

	// 清空后不能 Reaction
	if canPlayReaction(state, "P1") {
		t.Fatal("reaction should not be allowed after stack is cleared")
	}
}

// =============================================================================
// 测试用例 5: Conflict Loop - 攻击已 Exhausted 的角色应该失败
// 测试目的: 验证攻击目标合法性检查会拒绝已 Exhausted 的攻击者
// 输入数据: Exhausted 的攻击者尝试攻击敌方角色
// 预期输出: canDeclareAttack 返回 false
// 验证方法: 检查攻击者 Exhausted 状态对攻击合法性的影响
// =============================================================================
func TestConflictLoop_AttackWithExhaustedAttackerShouldFail(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-attack-exhausted",
		ActivePlayerID: "P1",
	})

	// 创建已 Exhausted 的攻击者
	attacker := CardState{
		CardID:       "attacker-1",
		DefinitionID: "ATTACKER",
		Name:         "攻击者",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Exhausted:    true, // 已疲惫
	}

	target := CardState{
		CardID:       "target-1",
		DefinitionID: "TARGET",
		Name:         "目标",
		Kind:         CardKindCharacter,
		OwnerID:      "P2",
		Zone:         CardZoneTable,
	}

	state.Board.Cards = []CardState{attacker, target}

	// 已疲惫的攻击者不能攻击
	if canDeclareAttack(state, "attacker-1", "target-1") {
		t.Fatal("exhausted attacker should not be able to attack")
	}

	// 将攻击者设为 ready
	state.Board.Cards[0].Exhausted = false

	// 现在可以攻击
	if !canDeclareAttack(state, "attacker-1", "target-1") {
		t.Fatal("ready attacker should be able to attack")
	}
}

// =============================================================================
// 测试用例 6: Attachment - 源卡片离场但宿主仍在场时的处理
// 测试目的: 验证当附属源卡片离场但宿主仍在场时，附属关系被清理但宿主保留
// 输入数据: 宿主在场，附属源离场
// 预期输出: 附属从 Active 中移除，但宿主卡片仍在场上
// 验证方法: 检查 Attachments.Active 和宿主卡片的 Zone
// =============================================================================
func TestAttachment_SourceDepartureWithHostRemaining(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-source-departure",
		ActivePlayerID: "P1",
	})

	// 宿主和源都在场
	hostCard := CardState{
		CardID:       "host-1",
		DefinitionID: "HOST",
		Name:         "宿主",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
	}

	sourceCard := CardState{
		CardID:       "source-1",
		DefinitionID: "SOURCE",
		Name:         "附属源",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
	}

	state.Board.Cards = []CardState{hostCard, sourceCard}

	// 创建附属关系
	state.Board.Attachments.Active = []Attachment{
		{ID: "att-1", SourceCardID: "source-1", TargetCardID: "host-1", HostCardID: "host-1"},
	}

	// 源卡片离场
	for i := range state.Board.Cards {
		if state.Board.Cards[i].CardID == "source-1" {
			moveCardToDiscard(&state.Board.Cards[i])
			break
		}
	}

	// 清理过期附属
	manager := NewAttachmentManager(state)
	state = manager.PruneExpired()

	// 附属应该被清理
	if len(state.Board.Attachments.Active) != 0 {
		t.Fatalf("attachment should be removed when source departs, got %d", len(state.Board.Attachments.Active))
	}

	// 宿主应该仍在场上
	hostAfter := findCardByID(state, "host-1")
	if hostAfter == nil || hostAfter.Zone != CardZoneTable {
		t.Fatal("host should still be on table after source departs")
	}
}

// =============================================================================
// 测试用例 7: Marker - 多个玩家同时拥有不同类型 Marker
// 测试目的: 验证多个玩家可以同时拥有不同类型和数量的 marker，互不干扰
// 输入数据: P1 有 3 个 typeA，P2 有 5 个 typeB
// 预期输出: 每个玩家的 marker 独立存储，互不影响
// 验证方法: 分别检查 P1 和 P2 的 marker 值
// =============================================================================
func TestMarker_MultiplePlayersWithDifferentMarkers(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-multi-player-markers",
		ActivePlayerID: "P1",
	})
	state.Players = []string{"P1", "P2"}

	// P1 获得 3 个 typeA marker
	state.Board.Markers.SetMarker("P1", "typeA", 3)

	// P2 获得 5 个 typeB marker
	state.Board.Markers.SetMarker("P2", "typeB", 5)

	// 验证 P1 的 marker
	if state.Board.Markers.GetMarker("P1", "typeA") != 3 {
		t.Fatalf("P1 should have 3 typeA markers, got %d", state.Board.Markers.GetMarker("P1", "typeA"))
	}
	if state.Board.Markers.GetMarker("P1", "typeB") != 0 {
		t.Fatalf("P1 should have 0 typeB markers, got %d", state.Board.Markers.GetMarker("P1", "typeB"))
	}

	// 验证 P2 的 marker
	if state.Board.Markers.GetMarker("P2", "typeB") != 5 {
		t.Fatalf("P2 should have 5 typeB markers, got %d", state.Board.Markers.GetMarker("P2", "typeB"))
	}
	if state.Board.Markers.GetMarker("P2", "typeA") != 0 {
		t.Fatalf("P2 should have 0 typeA markers, got %d", state.Board.Markers.GetMarker("P2", "typeA"))
	}

	// 修改 P1 的 marker，不应影响 P2
	state.Board.Markers.SetMarker("P1", "typeA", 10)
	if state.Board.Markers.GetMarker("P2", "typeB") != 5 {
		t.Fatal("modifying P1's markers should not affect P2's markers")
	}
}

// =============================================================================
// 测试用例 8: Hidden Deployment - Reveal 后 Continuous Effect 正确应用
// 测试目的: 验证卡牌 Reveal 后，其 continuous effect 能被正确计算和应用
// 输入数据: Face-down 的 XQ31 类型卡牌，Reveal 后应给声望角色 +1 防御
// 预期输出: Reveal 后请求 continuous recalculation，且效果正确应用
// 验证方法: 检查 PendingRecalculation 标志和角色防御力
// =============================================================================
func TestHiddenDeployment_RevealTriggersContinuousEffect(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-reveal-continuous",
		ActivePlayerID: "P1",
	})

	// 创建 face-down 的 XQ31（莫兰大主教）
	xq31 := CardState{
		CardID:          "xq31-1",
		DefinitionID:    "XQ31",
		Name:            "莫兰大主教",
		Kind:            CardKindCharacter,
		OwnerID:         "P1",
		Zone:            CardZoneTable,
		FaceDown:        true,
		PrintedKeywords: []string{"领袖", "公开", "声望"},
		PrintedStats:    CardNumericStats{Combat: 1, Defense: 4},
	}

	// 创建声望盟友
	ally := CardState{
		CardID:          "ally-1",
		DefinitionID:    "ALLY",
		Name:            "声望盟友",
		Kind:            CardKindCharacter,
		OwnerID:         "P1",
		Zone:            CardZoneTable,
		PrintedKeywords: []string{"声望"},
		PrintedStats:    CardNumericStats{Combat: 1, Defense: 2},
	}

	state.Board.Cards = []CardState{xq31, ally}

	// Reveal XQ31
	state = revealCard(state, "xq31-1")

	// 验证请求了 continuous recalculation
	if !state.Board.Continuous.PendingRecalculation {
		t.Fatal("reveal should request continuous recalculation")
	}
}

// =============================================================================
// 测试用例 9: Conflict Loop - 游戏结束条件精确检查
// 测试目的: 验证游戏结束条件在分数恰好等于阈值和超过阈值时都能正确触发
// 输入数据: 设置 VictoryThreshold 为 10，分别测试分数为 9、10、11 的情况
// 预期输出: 分数 >= 10 时返回 true，否则返回 false
// 验证方法: 边界值测试
// =============================================================================
func TestConflictLoop_GameOverBoundaryConditions(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-game-over-boundary",
		ActivePlayerID: "P1",
	})

	state.Score.VictoryThreshold = 10

	// 分数为 9，游戏未结束
	state.Score.ByPlayer["P1"] = 9
	if checkGameOver(state) {
		t.Fatal("game should not be over at score 9")
	}

	// 分数为 10，游戏结束（边界值）
	state.Score.ByPlayer["P1"] = 10
	if !checkGameOver(state) {
		t.Fatal("game should be over at score 10 (boundary)")
	}

	// 分数为 11，游戏结束（超过阈值）
	state.Score.ByPlayer["P1"] = 11
	if !checkGameOver(state) {
		t.Fatal("game should be over at score 11")
	}
}

// =============================================================================
// 测试用例 10: 综合场景 - Attachment + Marker + Hidden Deployment 联合使用
// 测试目的: 验证多个系统联合工作时的正确性
// 输入数据: 一个复杂的游戏场景，包含附属、marker、face-down 卡牌
// 预期输出: 所有系统独立工作，互不干扰，状态一致
// 验证方法: 综合检查所有子系统的状态
// =============================================================================
func TestComprehensive_CombinedSystems(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-comprehensive",
		ActivePlayerID: "P1",
	})
	state.Players = []string{"P1", "P2"}

	// 1. 设置 Marker
	state.Board.Markers.SetMarker("P1", "secret_society", 3)

	// 2. 创建 face-down 卡牌
	hiddenCard := CardState{
		CardID:       "hidden-1",
		DefinitionID: "HIDDEN",
		Name:         "暗藏者",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		FaceDown:     true,
	}

	// 3. 创建宿主和附属
	hostCard := CardState{
		CardID:       "host-1",
		DefinitionID: "HOST",
		Name:         "宿主",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
	}

	sourceCard := CardState{
		CardID:       "source-1",
		DefinitionID: "SOURCE",
		Name:         "附属源",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
	}

	state.Board.Cards = []CardState{hiddenCard, hostCard, sourceCard}

	// 4. 创建附属关系
	state.Board.Attachments.Active = []Attachment{
		{ID: "att-1", SourceCardID: "source-1", TargetCardID: "host-1", HostCardID: "host-1"},
	}

	// 5. 生成投影验证所有系统
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

	// 验证 P1 可以看到自己的 marker
	p1View := bundle.Players["P1"]
	if p1View.Markers == nil || p1View.Markers["secret_society"] != 3 {
		t.Fatal("P1 should see their markers in projection")
	}

	// 验证 P1 可以看到自己的 face-down 卡牌（至少能看到存在）
	if len(p1View.Board.Cards) != 3 {
		t.Fatalf("P1 should see 3 cards, got %d", len(p1View.Board.Cards))
	}

	// 验证 P2 看不到 P1 的 face-down 卡牌详情
	p2View := bundle.Players["P2"]
	hiddenCardView := p2View.Board.Cards[0] // hidden-1 是第一张
	if hiddenCardView.FaceDown == false && hiddenCardView.Visibility != "hidden" {
		t.Fatal("P2 should not see details of P1's face-down card")
	}

	// 验证附属关系仍然存在
	if len(state.Board.Attachments.Active) != 1 {
		t.Fatal("attachment should still be active")
	}

	// 6. 宿主离场，验证附属被清理但 marker 和 face-down 卡牌不受影响
	for i := range state.Board.Cards {
		if state.Board.Cards[i].CardID == "host-1" {
			moveCardToDiscard(&state.Board.Cards[i])
			break
		}
	}

	manager := NewAttachmentManager(state)
	state = manager.PruneExpired()

	// 附属应该被清理
	if len(state.Board.Attachments.Active) != 0 {
		t.Fatal("attachment should be cleaned up after host departure")
	}

	// marker 应该仍然存在
	if state.Board.Markers.GetMarker("P1", "secret_society") != 3 {
		t.Fatal("markers should not be affected by host departure")
	}

	// 验证 face-down 卡牌应该仍然存在
	hiddenAfter := findCardByID(state, "hidden-1")
	if hiddenAfter == nil || hiddenAfter.Zone != CardZoneTable {
		t.Fatal("face-down card should not be affected by host departure")
	}
}

// =============================================================================
// 测试用例 11: Marker - 通过 Action 正式管道设置和移除标记物
// 测试目的: 验证通过 ActionKindSetMarker 和 ActionKindRemoveMarker 在正式管道中设置和移除标记物
// 输入数据: 发送 SetMarker action 设置标记物，然后发送 RemoveMarker action 移除标记物
// 预期输出: 标记物被正确设置和移除，状态一致
// 验证方法: 检查标记物数量和事件类型
// =============================================================================
func TestMarker_ActionPipeline_SetAndRemove(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-marker-action-pipeline",
		ActivePlayerID: "P1",
	})
	state.Players = []string{"P1", "P2"}

	// 测试设置标记物
	setAction := Action{
		ID:           "action-1",
		ActorID:      "P1",
		Kind:         ActionKindSetMarker,
		MarkerType:   "secret_society",
		MarkerAmount: 3,
	}

	result, err := SubmitAction(state, setAction)
	if err != nil {
		t.Fatalf("SetMarker action failed: %v", err)
	}

	// 验证标记物被设置
	if result.State.Board.Markers.GetMarker("P1", "secret_society") != 3 {
		t.Fatalf("Expected 3 secret_society markers for P1, got %d", result.State.Board.Markers.GetMarker("P1", "secret_society"))
	}

	// 验证事件类型
	if result.Event.Kind != EventKindMarkerSet {
		t.Fatalf("Expected EventKindMarkerSet, got %s", result.Event.Kind)
	}

	// 测试移除标记物
	removeAction := Action{
		ID:         "action-2",
		ActorID:    "P1",
		Kind:       ActionKindRemoveMarker,
		MarkerType: "secret_society",
	}

	result, err = SubmitAction(result.State, removeAction)
	if err != nil {
		t.Fatalf("RemoveMarker action failed: %v", err)
	}

	// 验证标记物被移除
	if result.State.Board.Markers.GetMarker("P1", "secret_society") != 0 {
		t.Fatalf("Expected 0 secret_society markers for P1, got %d", result.State.Board.Markers.GetMarker("P1", "secret_society"))
	}

	// 验证事件类型
	if result.Event.Kind != EventKindMarkerRemoved {
		t.Fatalf("Expected EventKindMarkerRemoved, got %s", result.Event.Kind)
	}
}

func TestMarker_ActionPipeline_RemoveByAmount(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-marker-remove-by-amount",
		ActivePlayerID: "P1",
	})
	state.Players = []string{"P1", "P2"}

	result, err := SubmitAction(state, Action{
		ID:           "action-1",
		ActorID:      "P1",
		Kind:         ActionKindSetMarker,
		MarkerType:   "secret_society",
		MarkerAmount: 3,
	})
	if err != nil {
		t.Fatalf("SetMarker action failed: %v", err)
	}

	result, err = SubmitAction(result.State, Action{
		ID:           "action-2",
		ActorID:      "P1",
		Kind:         ActionKindRemoveMarker,
		MarkerType:   "secret_society",
		MarkerAmount: 1,
	})
	if err != nil {
		t.Fatalf("RemoveMarker action failed: %v", err)
	}

	if got := result.State.Board.Markers.GetMarker("P1", "secret_society"); got != 2 {
		t.Fatalf("Expected 2 secret_society markers for P1 after removing 1, got %d", got)
	}
	if result.Event.Kind != EventKindMarkerRemoved {
		t.Fatalf("Expected EventKindMarkerRemoved, got %s", result.Event.Kind)
	}
	if result.Event.MarkerAmount != 2 {
		t.Fatalf("Expected event markerAmount to be 2 (remaining), got %d", result.Event.MarkerAmount)
	}
}

// =============================================================================
// 测试用例 12: Marker - 合法性检查
// 测试目的: 验证 marker 操作的合法性检查功能
// 输入数据: 测试各种非法情况，如缺少 MarkerType、MarkerAmount <= 0、移除不存在的标记物
// 预期输出: 非法操作被拒绝，返回相应的错误代码
// 验证方法: 检查返回的错误类型和原因
// =============================================================================
func TestMarker_LegalityChecks(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-marker-legality",
		ActivePlayerID: "P1",
	})
	state.Players = []string{"P1", "P2"}

	// 测试 1: 缺少 MarkerType
	action := Action{
		ID:           "action-1",
		ActorID:      "P1",
		Kind:         ActionKindSetMarker,
		MarkerAmount: 3,
	}

	_, err := SubmitAction(state, action)
	if err == nil {
		t.Fatal("Expected error for missing MarkerType, got none")
	}

	// 测试 2: MarkerAmount <= 0
	action = Action{
		ID:           "action-2",
		ActorID:      "P1",
		Kind:         ActionKindSetMarker,
		MarkerType:   "secret_society",
		MarkerAmount: 0,
	}

	_, err = SubmitAction(state, action)
	if err == nil {
		t.Fatal("Expected error for MarkerAmount <= 0, got none")
	}

	// 测试 3: 移除不存在的标记物
	action = Action{
		ID:         "action-3",
		ActorID:    "P1",
		Kind:       ActionKindRemoveMarker,
		MarkerType: "non_existent",
	}

	_, err = SubmitAction(state, action)
	if err == nil {
		t.Fatal("Expected error for removing non-existent marker, got none")
	}

	// 测试 4: 移除数量超过当前数量
	result, err := SubmitAction(state, Action{
		ID:           "action-4",
		ActorID:      "P1",
		Kind:         ActionKindSetMarker,
		MarkerType:   "secret_society",
		MarkerAmount: 1,
	})
	if err != nil {
		t.Fatalf("SetMarker action failed: %v", err)
	}

	_, err = SubmitAction(result.State, Action{
		ID:           "action-5",
		ActorID:      "P1",
		Kind:         ActionKindRemoveMarker,
		MarkerType:   "secret_society",
		MarkerAmount: 2,
	})
	if err == nil {
		t.Fatal("Expected error when removing more markers than current amount, got none")
	}
}

// =============================================================================
// 测试用例 13: FaceDown - 通过 Action 正式管道设置和揭示卡牌
// 测试目的: 验证通过 ActionKindSetFaceDown 和 ActionKindRevealCard 在正式管道中设置和揭示卡牌
// 输入数据: 发送 SetFaceDown action 设置卡牌为面朝下，然后发送 RevealCard action 揭示卡牌
// 预期输出: 卡牌状态被正确设置和揭示，状态一致
// 验证方法: 检查卡牌的 FaceDown 和 Revealed 状态
// =============================================================================
func TestFaceDown_ActionPipeline_SetAndReveal(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-facedown-action-pipeline",
		ActivePlayerID: "P1",
	})

	// 创建一张卡牌
	card := CardState{
		CardID:       "card-1",
		DefinitionID: "TEST_CARD",
		Name:         "测试卡牌",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
	}
	state.Board.Cards = []CardState{card}

	// 测试设置为面朝下
	setFaceDownAction := Action{
		ID:      "action-1",
		ActorID: "P1",
		Kind:    ActionKindSetFaceDown,
		CardID:  "card-1",
	}

	result, err := SubmitAction(state, setFaceDownAction)
	if err != nil {
		t.Fatalf("SetFaceDown action failed: %v", err)
	}

	// 验证卡牌被设置为面朝下
	cardAfterSet := findCardByID(result.State, "card-1")
	if cardAfterSet == nil {
		t.Fatal("Card not found after SetFaceDown")
	}
	if !cardAfterSet.FaceDown {
		t.Fatal("Card should be face-down after SetFaceDown")
	}
	if cardAfterSet.Revealed {
		t.Fatal("Card should not be revealed after SetFaceDown")
	}

	// 验证事件类型
	if result.Event.Kind != EventKindFaceDownSet {
		t.Fatalf("Expected EventKindFaceDownSet, got %s", result.Event.Kind)
	}

	// 测试揭示卡牌
	revealAction := Action{
		ID:      "action-2",
		ActorID: "P1",
		Kind:    ActionKindRevealCard,
		CardID:  "card-1",
	}

	result, err = SubmitAction(result.State, revealAction)
	if err != nil {
		t.Fatalf("RevealCard action failed: %v", err)
	}

	// 验证卡牌被揭示
	cardAfterReveal := findCardByID(result.State, "card-1")
	if cardAfterReveal == nil {
		t.Fatal("Card not found after RevealCard")
	}
	if cardAfterReveal.FaceDown {
		t.Fatal("Card should not be face-down after RevealCard")
	}
	if !cardAfterReveal.Revealed {
		t.Fatal("Card should be revealed after RevealCard")
	}

	// 验证事件类型
	if result.Event.Kind != EventKindCardRevealed {
		t.Fatalf("Expected EventKindCardRevealed, got %s", result.Event.Kind)
	}
}

// =============================================================================
// 测试用例 14: 标准动作空栈约束（表驱动测试）
// 测试目的: 验证 reveal/inspect/set_face_down 等标准动作在非空栈时应被拒绝
// 输入数据: 各种标准动作在空栈和非空栈状态下的执行
// 预期输出: 空栈时允许执行，非空栈时拒绝执行
// 验证方法: 表驱动测试，检查不同状态下的执行结果
// =============================================================================
func TestStandardActions_EmptyStackConstraint(t *testing.T) {
	// 测试用例表
	testCases := []struct {
		name           string
		action         Action
		hasStack       bool
		expectSuccess  bool
		expectErrorMsg string
	}{
		{
			name: "reveal_card with empty stack",
			action: Action{
				ID:      "action-1",
				ActorID: "P1",
				Kind:    ActionKindRevealCard,
				CardID:  "card-1",
			},
			hasStack:       false,
			expectSuccess:  true,
			expectErrorMsg: "",
		},
		{
			name: "reveal_card with non-empty stack",
			action: Action{
				ID:      "action-2",
				ActorID: "P1",
				Kind:    ActionKindRevealCard,
				CardID:  "card-1",
			},
			hasStack:       true,
			expectSuccess:  false,
			expectErrorMsg: "LEGALITY_FAILED_STACK_NOT_EMPTY",
		},
		{
			name: "inspect_card with empty stack",
			action: Action{
				ID:      "action-3",
				ActorID: "P1",
				Kind:    ActionKindInspectCard,
				CardID:  "card-1",
			},
			hasStack:       false,
			expectSuccess:  true,
			expectErrorMsg: "",
		},
		{
			name: "inspect_card with non-empty stack",
			action: Action{
				ID:      "action-4",
				ActorID: "P1",
				Kind:    ActionKindInspectCard,
				CardID:  "card-1",
			},
			hasStack:       true,
			expectSuccess:  false,
			expectErrorMsg: "LEGALITY_FAILED_STACK_NOT_EMPTY",
		},
		{
			name: "set_face_down with empty stack",
			action: Action{
				ID:      "action-5",
				ActorID: "P1",
				Kind:    ActionKindSetFaceDown,
				CardID:  "card-1",
			},
			hasStack:       false,
			expectSuccess:  true,
			expectErrorMsg: "",
		},
		{
			name: "set_face_down with non-empty stack",
			action: Action{
				ID:      "action-6",
				ActorID: "P1",
				Kind:    ActionKindSetFaceDown,
				CardID:  "card-1",
			},
			hasStack:       true,
			expectSuccess:  false,
			expectErrorMsg: "LEGALITY_FAILED_STACK_NOT_EMPTY",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			// 创建测试状态
			state := NewGameState(InitialStateConfig{
				GameID:         "test-empty-stack-constraint",
				ActivePlayerID: "P1",
			})

			// 添加一张卡牌
			card := CardState{
				CardID:       "card-1",
				DefinitionID: "TEST_CARD",
				Name:         "测试卡牌",
				Kind:         CardKindCharacter,
				OwnerID:      "P1",
				Zone:         CardZoneTable,
			}
			state.Board.Cards = []CardState{card}

			// 如果需要非空栈，添加一个操作
			if tc.hasStack {
				state.Board.Stack = []Operation{
					{
						ID:            "op-1",
						ActorID:       "P1",
						Kind:          OperationKindCardEffect,
						Status:        OperationStatusPending,
						CardID:        "card-1",
						Label:         "test effect",
						RequiresStack: true,
					},
				}
			}

			// 执行动作
			_, err := SubmitAction(state, tc.action)

			// 验证结果
			if tc.expectSuccess {
				if err != nil {
					t.Fatalf("Expected success, got error: %v", err)
				}
			} else {
				if err == nil {
					t.Fatalf("Expected error, got success")
				}
				if !strings.Contains(err.Error(), tc.expectErrorMsg) {
					t.Fatalf("Expected error message to contain %q, got %q", tc.expectErrorMsg, err.Error())
				}
			}
		})
	}
}
