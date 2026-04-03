package rules

import (
	"errors"
	"testing"
)

func TestBuildAssetMovesHandCardIntoAssetZone(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "build-asset-success",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           20260402,
	})
	state.Board.Cards = append(state.Board.Cards, CardState{
		CardID:         "p1-hand-build",
		DefinitionID:   "DQJC001",
		Name:           "待建立资产",
		Kind:           CardKindCharacter,
		OwnerID:        "P1",
		Zone:           CardZoneHand,
		VisibleToOwner: true,
	})

	result, err := SubmitAction(state, Action{
		ID:      "act-build-asset-success",
		ActorID: "P1",
		Kind:    ActionKind("build_asset"),
		CardID:  "p1-hand-build",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Event.Kind != EventKind("asset_built") {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKind("asset_built"))
	}

	card := cardStateByID(t, result.State, "p1-hand-build")
	if card.Zone != CardZone("asset") {
		t.Fatalf("zone = %q, want %q", card.Zone, CardZone("asset"))
	}
	if card.Kind != CardKindAsset {
		t.Fatalf("kind = %q, want %q", card.Kind, CardKindAsset)
	}
}

func TestBuildAssetRejectsSecondUseInSameTurn(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "build-asset-once",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           20260402,
	})
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "p1-hand-build-1",
			Name:           "资产1",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
		CardState{
			CardID:         "p1-hand-build-2",
			Name:           "资产2",
			Kind:           CardKindAsset,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
	)

	state = mustSubmit(t, state, Action{
		ID:      "act-build-asset-first",
		ActorID: "P1",
		Kind:    ActionKind("build_asset"),
		CardID:  "p1-hand-build-1",
	})

	_, err := SubmitAction(state, Action{
		ID:      "act-build-asset-second",
		ActorID: "P1",
		Kind:    ActionKind("build_asset"),
		CardID:  "p1-hand-build-2",
	})
	if err == nil {
		t.Fatal("SubmitAction succeeded, want once-per-turn rejection")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}
	if legalityErr.Code != ReasonCodeLegalityFailedActionProhibited {
		t.Fatalf("error code = %q, want %q", legalityErr.Code, ReasonCodeLegalityFailedActionProhibited)
	}
	if legalityErr.MessageKey != "rules.build_asset.once_per_turn" {
		t.Fatalf("message key = %q, want %q", legalityErr.MessageKey, "rules.build_asset.once_per_turn")
	}
}

func TestBuildAssetAllowsUseAgainOnNextOwnTurn(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "build-asset-next-turn",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           20260402,
	})
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "p1-hand-build-a",
			Name:           "资产A",
			Kind:           CardKindAsset,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
		CardState{
			CardID:         "p1-hand-build-b",
			Name:           "资产B",
			Kind:           CardKindAsset,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
	)

	state = mustSubmit(t, state, Action{
		ID:      "act-build-asset-turn1",
		ActorID: "P1",
		Kind:    ActionKindBuildAsset,
		CardID:  "p1-hand-build-a",
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-build-asset-to-end-p1",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-build-asset-to-main-p2",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-build-asset-to-end-p2",
		ActorID: "P2",
		Kind:    ActionKindAdvancePhase,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-build-asset-to-main-p1",
		ActorID: "P2",
		Kind:    ActionKindAdvancePhase,
	})

	_, err := SubmitAction(state, Action{
		ID:      "act-build-asset-turn3",
		ActorID: "P1",
		Kind:    ActionKindBuildAsset,
		CardID:  "p1-hand-build-b",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error on next own turn: %v", err)
	}
}

func TestBuildAssetAllowsWhenActorHasPriorityButIsNotActivePlayer(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "build-asset-not-active",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           20260402,
	})
	state.Turn.Priority.CurrentPlayerID = "P2"
	state.Board.Cards = append(state.Board.Cards, CardState{
		CardID:         "p2-hand-build",
		Name:           "资产来源",
		Kind:           CardKindAsset,
		OwnerID:        "P2",
		Zone:           CardZoneHand,
		VisibleToOwner: true,
	})

	result, err := SubmitAction(state, Action{
		ID:      "act-build-asset-not-active",
		ActorID: "P2",
		Kind:    ActionKindBuildAsset,
		CardID:  "p2-hand-build",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}
	if result.Event.Kind != EventKindAssetBuilt {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindAssetBuilt)
	}
}

func TestBuildAssetDoesNotChargePrintedCardCost(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "build-asset-zero-cost",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           20260403,
	})
	state.Turn.Resources["P1"] = PlayerResourceState{Current: 1, Max: 1}
	state.Board.Cards = append(state.Board.Cards, CardState{
		CardID:         "p1-hand-expensive-build",
		DefinitionID:   "JC090",
		Name:           "灵魂契约",
		Cost:           2,
		Kind:           CardKindCharacter,
		OwnerID:        "P1",
		Zone:           CardZoneHand,
		VisibleToOwner: true,
	})

	result, err := SubmitAction(state, Action{
		ID:      "act-build-asset-zero-cost",
		ActorID: "P1",
		Kind:    ActionKindBuildAsset,
		CardID:  "p1-hand-expensive-build",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	card := cardStateByID(t, result.State, "p1-hand-expensive-build")
	if card.Zone != CardZoneAsset {
		t.Fatalf("zone = %q, want %q", card.Zone, CardZoneAsset)
	}

	pool := result.State.Turn.Resources["P1"]
	if pool.Current != 1 || pool.Max != 1 {
		t.Fatalf("resources after build_asset = %#v, want unchanged current=1 max=1", pool)
	}
}
