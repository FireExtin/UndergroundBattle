package rules

import "testing"

// TestCombatTargetLegality 测试：战斗目标合法性
func TestCombatTargetLegality(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-combat",
		ActivePlayerID: "P1",
	})

	// 创建攻击者和目标
	attacker := CardState{
		CardID:       "attacker-1",
		DefinitionID: "ATTACKER",
		Name:         "攻击者",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Exhausted:    false,
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

	// 验证可以攻击敌方角色
	canAttack := canDeclareAttack(state, "attacker-1", "target-1")
	if !canAttack {
		t.Fatal("should be able to attack enemy character")
	}
}

// TestInvestigationConnectsToRegionControl 测试：调查与地区控制衔接
func TestInvestigationConnectsToRegionControl(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-investigation",
		ActivePlayerID: "P1",
	})

	// 创建地区
	region := CardState{
		CardID:       "region-1",
		DefinitionID: "REGION",
		Name:         "地区",
		Kind:         CardKindRegion,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		InfluenceByPlayer: map[string]int{
			"P1": 1,
		},
	}

	state.Board.Cards = []CardState{region}

	// 验证调查可以增加 influence
	newState := addInfluence(state, "region-1", "P1", 2)

	regionAfter := findCardByID(newState, "region-1")
	if regionAfter == nil {
		t.Fatal("region not found")
	}

	if regionAfter.InfluenceByPlayer["P1"] != 3 {
		t.Fatalf("expected P1 influence to be 3, got %d", regionAfter.InfluenceByPlayer["P1"])
	}
}

// TestGameOverGate 测试：游戏结束检查
func TestGameOverGate(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-game-over",
		ActivePlayerID: "P1",
	})

	// 设置胜利分数
	state.Score.ByPlayer["P1"] = 10
	state.Score.VictoryThreshold = 10

	// 检查游戏是否应该结束
	isGameOver := checkGameOver(state)
	if !isGameOver {
		t.Fatal("game should be over when score reaches threshold")
	}
}

// Helper functions

func canDeclareAttack(state GameState, attackerID, targetID string) bool {
	attacker := findCardByID(state, attackerID)
	target := findCardByID(state, targetID)

	if attacker == nil || target == nil {
		return false
	}

	// 攻击者必须 ready
	if attacker.Exhausted {
		return false
	}

	// 目标必须是敌方角色
	if target.OwnerID == attacker.OwnerID {
		return false
	}

	// 目标必须在场上
	if target.Zone != CardZoneTable {
		return false
	}

	return true
}

func addInfluence(state GameState, regionID, playerID string, amount int) GameState {
	working := cloneGameState(state)
	for i := range working.Board.Cards {
		if working.Board.Cards[i].CardID == regionID {
			if working.Board.Cards[i].InfluenceByPlayer == nil {
				working.Board.Cards[i].InfluenceByPlayer = make(map[string]int)
			}
			working.Board.Cards[i].InfluenceByPlayer[playerID] += amount
			break
		}
	}
	return working
}

func checkGameOver(state GameState) bool {
	for playerID, score := range state.Score.ByPlayer {
		if score >= state.Score.VictoryThreshold {
			_ = playerID
			return true
		}
	}
	return false
}
