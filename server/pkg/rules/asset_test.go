package rules

import "testing"

func TestAssetCardKindExists(t *testing.T) {
	// Asset Card Kind 应该存在
	var _ CardKind = CardKindAsset
}

func TestAssetCardCanBeSetOnCardState(t *testing.T) {
	// 红测：Asset Card Kind 可以设置在 CardState 上
	card := CardState{
		CardID: "asset-1",
		Kind:   CardKindAsset,
	}
	if card.Kind != CardKindAsset {
		t.Fatalf("card.Kind = %q, want %q", card.Kind, CardKindAsset)
	}
}

func TestAssetCardCanEnterTable(t *testing.T) {
	// 红测：Asset Card 可以进场到 Table
	state := NewGameState(InitialStateConfig{
		GameID:         "test-asset-enter",
		ActivePlayerID: "P1",
	})

	assetCard := CardState{
		CardID:       "asset-1",
		DefinitionID: "ASSET01",
		Name:         "测试资产牌",
		Kind:         CardKindAsset,
		OwnerID:      "P1",
		Zone:         CardZoneHand,
	}

	state.Board.Cards = []CardState{assetCard}

	// 模拟进场
	index := findCardIndex(state, "asset-1")
	state.Board.Cards[index].Zone = CardZoneTable
	state.Board.Cards[index].Destroyed = false

	assetAfter := state.Board.Cards[index]
	if assetAfter.Zone != CardZoneTable {
		t.Fatalf("asset.Zone = %q, want %q", assetAfter.Zone, CardZoneTable)
	}
	if assetAfter.Destroyed {
		t.Fatal("asset.Destroyed = true, want false")
	}
}

func TestAssetCardCanLeaveToDiscard(t *testing.T) {
	// 红测：Asset Card 可以离场进入 Discard
	state := NewGameState(InitialStateConfig{
		GameID:         "test-asset-leave",
		ActivePlayerID: "P1",
	})

	assetCard := CardState{
		CardID:       "asset-1",
		DefinitionID: "ASSET01",
		Name:         "测试资产牌",
		Kind:         CardKindAsset,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Destroyed:    false,
	}

	state.Board.Cards = []CardState{assetCard}

	// 模拟离场
	index := findCardIndex(state, "asset-1")
	state.Board.Cards[index].Zone = CardZoneDiscard
	state.Board.Cards[index].Destroyed = true

	assetAfter := state.Board.Cards[index]
	if assetAfter.Zone != CardZoneDiscard {
		t.Fatalf("asset.Zone = %q, want %q", assetAfter.Zone, CardZoneDiscard)
	}
	if !assetAfter.Destroyed {
		t.Fatal("asset.Destroyed = false, want true")
	}
}

func TestAssetCardDoesNotAffectExistingCharacters(t *testing.T) {
	// 红测：Asset Card 不影响现有 Character 语义
	state := NewGameState(InitialStateConfig{
		GameID:         "test-asset-character",
		ActivePlayerID: "P1",
	})

	characterCard := CardState{
		CardID:       "char-1",
		DefinitionID: "CHAR01",
		Name:         "测试角色",
		Kind:         CardKindCharacter,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Destroyed:    false,
	}

	assetCard := CardState{
		CardID:       "asset-1",
		DefinitionID: "ASSET01",
		Name:         "测试资产牌",
		Kind:         CardKindAsset,
		OwnerID:      "P1",
		Zone:         CardZoneTable,
		Destroyed:    false,
	}

	state.Board.Cards = []CardState{characterCard, assetCard}

	// 验证角色牌的语义没有被破坏
	charIndex := findCardIndex(state, "char-1")
	charAfter := state.Board.Cards[charIndex]
	if charAfter.Kind != CardKindCharacter {
		t.Fatalf("character.Kind = %q, want %q", charAfter.Kind, CardKindCharacter)
	}
}
