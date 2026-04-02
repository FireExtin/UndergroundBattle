package rules

import (
	"errors"
	"testing"
)

// Purpose: Covers play_card deployment across character/asset/event cards plus invalid edge paths.

func TestPlayCardDeploysCharacterToRegionFaceUp(t *testing.T) {
	state := basePlayCardState()
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p1-char-1",
			DefinitionID:   "DQJC001",
			Name:           "行动员",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Revealed:       false,
			PrintedStats: CardNumericStats{
				Combat:        1,
				Defense:       1,
				Investigation: 1,
			},
			EffectiveStats: CardNumericStats{
				Combat:        1,
				Defense:       1,
				Investigation: 1,
			},
		},
	)

	result, err := SubmitAction(state, Action{
		ID:                 "act-play-char-face-up",
		ActorID:            "P1",
		Kind:               ActionKindPlayCard,
		CardID:             "p1-char-1",
		PlayMode:           "face_up",
		TargetRegionCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}
	if result.Operation.Kind != OperationKindPlayCard {
		t.Fatalf("operation kind = %q, want %q", result.Operation.Kind, OperationKindPlayCard)
	}
	if result.Event.Kind != EventKindCardPlayed {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindCardPlayed)
	}

	deployed := cardStateByID(t, result.State, "p1-char-1")
	if deployed.Zone != CardZoneTable {
		t.Fatalf("zone = %q, want %q", deployed.Zone, CardZoneTable)
	}
	if deployed.RegionCardID != "region-1" {
		t.Fatalf("regionCardId = %q, want %q", deployed.RegionCardID, "region-1")
	}
	if deployed.FaceDown {
		t.Fatal("expected face-up deployment")
	}
	if !deployed.Revealed {
		t.Fatal("expected character to be revealed after face-up deployment")
	}
}

func TestPlayCardDeploysCharacterFaceDown(t *testing.T) {
	state := basePlayCardState()
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p1-char-secret",
			DefinitionID:   "DQJC002",
			Name:           "潜行者",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
	)

	result, err := SubmitAction(state, Action{
		ID:                 "act-play-char-face-down",
		ActorID:            "P1",
		Kind:               ActionKindPlayCard,
		CardID:             "p1-char-secret",
		PlayMode:           "face_down",
		TargetRegionCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	deployed := cardStateByID(t, result.State, "p1-char-secret")
	if deployed.Zone != CardZoneTable {
		t.Fatalf("zone = %q, want %q", deployed.Zone, CardZoneTable)
	}
	if !deployed.FaceDown {
		t.Fatal("expected face-down deployment")
	}
	if deployed.Revealed {
		t.Fatal("face-down deployment must not be publicly revealed")
	}
}

func TestPlayCardDeploysAssetToHostAndCreatesAttachment(t *testing.T) {
	state := basePlayCardState()
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p1-host-1",
			Name:           "宿主角色",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionCardID:   "region-1",
		},
		CardState{
			CardID:         "p1-asset-1",
			DefinitionID:   "BQ022",
			Name:           "合金指虎",
			Kind:           CardKindAsset,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
	)

	result, err := SubmitAction(state, Action{
		ID:           "act-play-asset",
		ActorID:      "P1",
		Kind:         ActionKindPlayCard,
		CardID:       "p1-asset-1",
		TargetCardID: "p1-host-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	asset := cardStateByID(t, result.State, "p1-asset-1")
	if asset.Zone != CardZoneTable {
		t.Fatalf("asset zone = %q, want %q", asset.Zone, CardZoneTable)
	}
	if asset.RegionCardID != "region-1" {
		t.Fatalf("asset regionCardId = %q, want %q", asset.RegionCardID, "region-1")
	}
	if len(result.State.Board.Attachments.Active) == 0 {
		t.Fatal("expected attachment to be created")
	}
	attachment := result.State.Board.Attachments.Active[0]
	if attachment.SourceCardID != "p1-asset-1" {
		t.Fatalf("attachment source = %q, want %q", attachment.SourceCardID, "p1-asset-1")
	}
	if attachment.TargetCardID != "p1-host-1" {
		t.Fatalf("attachment target = %q, want %q", attachment.TargetCardID, "p1-host-1")
	}
}

func TestPlayCardResolvesEventFromHand(t *testing.T) {
	state := basePlayCardState()
	state.Board.Cards = append(state.Board.Cards, CardState{
		CardID:         "p1-event-direct",
		DefinitionID:   testCardDirect,
		Name:           "读心术",
		Kind:           CardKindEvent,
		OwnerID:        "P1",
		Zone:           CardZoneHand,
		VisibleToOwner: true,
	})

	result, err := SubmitAction(state, Action{
		ID:             "act-play-event-direct",
		ActorID:        "P1",
		Kind:           ActionKindPlayCard,
		CardID:         "p1-event-direct",
		TargetPlayerID: "P2",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}
	if result.Event.Kind != EventKindOperationResolved {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindOperationResolved)
	}
	if len(result.State.Board.Stack) != 0 {
		t.Fatalf("stack depth = %d, want 0", len(result.State.Board.Stack))
	}
	card := cardStateByID(t, result.State, "p1-event-direct")
	if card.Zone != CardZoneDiscard {
		t.Fatalf("zone = %q, want %q", card.Zone, CardZoneDiscard)
	}
	if !card.Destroyed {
		t.Fatal("event in discard should be flagged destroyed")
	}
}

func TestPlayCardEnqueuesStackedEventAndResolvesAfterDoublePass(t *testing.T) {
	state := basePlayCardState()
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p1-event-stack",
			DefinitionID:   testCardScriptStack,
			Name:           "召现雷霆",
			Kind:           CardKindEvent,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
	)

	queued := mustSubmit(t, state, Action{
		ID:           "act-play-event-stack",
		ActorID:      "P1",
		Kind:         ActionKindPlayCard,
		CardID:       "p1-event-stack",
		TargetCardID: "region-1",
	})

	if len(queued.Board.Stack) != 1 {
		t.Fatalf("stack depth = %d, want 1", len(queued.Board.Stack))
	}
	card := cardStateByID(t, queued, "p1-event-stack")
	if card.Zone != CardZoneDiscard {
		t.Fatalf("zone = %q, want %q after enqueue", card.Zone, CardZoneDiscard)
	}

	afterP2Pass := mustSubmit(t, queued, Action{
		ID:      "act-play-event-stack-pass-p2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	})
	if afterP2Pass.Turn.Priority.CurrentPlayerID != "P1" {
		t.Fatalf("priority holder = %q, want P1", afterP2Pass.Turn.Priority.CurrentPlayerID)
	}

	resolved, err := SubmitAction(afterP2Pass, Action{
		ID:      "act-play-event-stack-pass-p1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}
	if resolved.Event.Kind != EventKindOperationResolved {
		t.Fatalf("event kind = %q, want %q", resolved.Event.Kind, EventKindOperationResolved)
	}
	if len(resolved.State.Board.Stack) != 0 {
		t.Fatalf("stack depth = %d, want 0 after resolution", len(resolved.State.Board.Stack))
	}
}

func TestPlayCardRejectsInvalidInputs(t *testing.T) {
	state := basePlayCardState()
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p1-char-hand",
			Name:           "角色手牌",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
		CardState{
			CardID:         "p1-char-table",
			Name:           "角色场上",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionCardID:   "region-1",
		},
	)

	cases := []struct {
		name   string
		action Action
		code   ReasonCode
	}{
		{
			name: "card not in hand",
			action: Action{
				ID:      "act-play-invalid-zone",
				ActorID: "P1",
				Kind:    ActionKindPlayCard,
				CardID:  "p1-char-table",
			},
			code: ReasonCodeLegalityFailedActionProhibited,
		},
		{
			name: "character missing target region",
			action: Action{
				ID:      "act-play-missing-region",
				ActorID: "P1",
				Kind:    ActionKindPlayCard,
				CardID:  "p1-char-hand",
			},
			code: ReasonCodeTargetFailedMissing,
		},
		{
			name: "character invalid play mode",
			action: Action{
				ID:                 "act-play-invalid-mode",
				ActorID:            "P1",
				Kind:               ActionKindPlayCard,
				CardID:             "p1-char-hand",
				TargetRegionCardID: "region-1",
				PlayMode:           "invalid_mode",
			},
			code: ReasonCodeRulesFailedInvalidState,
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			_, err := SubmitAction(state, tc.action)
			if err == nil {
				t.Fatal("SubmitAction unexpectedly accepted invalid play_card action")
			}

			var legalityErr *LegalityError
			if !errors.As(err, &legalityErr) {
				t.Fatalf("expected LegalityError, got %T", err)
			}
			if legalityErr.Code != tc.code {
				t.Fatalf("error code = %q, want %q", legalityErr.Code, tc.code)
			}
		})
	}
}

func TestPlayCardRejectsWhenCostUnpaid(t *testing.T) {
	state := basePlayCardState()
	state.Turn.Resources["P1"] = PlayerResourceState{Current: 1, Max: 1}
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p1-costly-character",
			Name:           "高费角色",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Cost:           3,
		},
	)

	_, err := SubmitAction(state, Action{
		ID:                 "act-play-card-cost-unpaid",
		ActorID:            "P1",
		Kind:               ActionKindPlayCard,
		CardID:             "p1-costly-character",
		PlayMode:           "face_up",
		TargetRegionCardID: "region-1",
	})
	if err == nil {
		t.Fatal("SubmitAction succeeded, want cost unpaid error")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}
	if legalityErr.Code != ReasonCodeCostFailedUnpaid {
		t.Fatalf("error code = %q, want %q", legalityErr.Code, ReasonCodeCostFailedUnpaid)
	}
}

func TestPlayCardDeductsResourceAfterSuccessfulDeployment(t *testing.T) {
	state := basePlayCardState()
	state.Turn.Resources["P1"] = PlayerResourceState{Current: 4, Max: 4}
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p1-cost-character",
			Name:           "中费角色",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Cost:           2,
		},
	)

	result, err := SubmitAction(state, Action{
		ID:                 "act-play-card-cost-paid",
		ActorID:            "P1",
		Kind:               ActionKindPlayCard,
		CardID:             "p1-cost-character",
		PlayMode:           "face_up",
		TargetRegionCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	pool := result.State.Turn.Resources["P1"]
	if pool.Current != 2 {
		t.Fatalf("resource current = %d, want 2", pool.Current)
	}
	if pool.Max != 4 {
		t.Fatalf("resource max = %d, want 4", pool.Max)
	}
}

func TestPlayCardFaceDownUsesSecretDeployCostAndSkipsLoyalty(t *testing.T) {
	state := basePlayCardState()
	state.Turn.Resources["P1"] = PlayerResourceState{Current: 1, Max: 1}
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p1-secret-heavy",
			Name:           "高费暗藏角色",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Cost:           6,
			Loyalty:        "红色红色红色",
		},
	)

	result, err := SubmitAction(state, Action{
		ID:                 "act-play-card-secret-deploy",
		ActorID:            "P1",
		Kind:               ActionKindPlayCard,
		CardID:             "p1-secret-heavy",
		PlayMode:           "face_down",
		TargetRegionCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	deployed := cardStateByID(t, result.State, "p1-secret-heavy")
	if deployed.Zone != CardZoneTable {
		t.Fatalf("zone = %q, want %q", deployed.Zone, CardZoneTable)
	}
	if !deployed.FaceDown {
		t.Fatal("expected face-down deployment")
	}

	pool := result.State.Turn.Resources["P1"]
	if pool.Current != 0 {
		t.Fatalf("resource current = %d, want 0 after secret deploy cost", pool.Current)
	}
}

func TestPlayCardFaceDownIgnoresLoyaltyRequirement(t *testing.T) {
	state := basePlayCardState()
	state.Turn.Resources["P1"] = PlayerResourceState{Current: 10, Max: 10}
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p1-secret-loyal",
			Name:           "暗藏忠诚角色",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Cost:           6,
			Loyalty:        "红色红色红色",
		},
	)

	result, err := SubmitAction(state, Action{
		ID:                 "act-play-card-secret-loyalty-ignore",
		ActorID:            "P1",
		Kind:               ActionKindPlayCard,
		CardID:             "p1-secret-loyal",
		PlayMode:           "face_down",
		TargetRegionCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	pool := result.State.Turn.Resources["P1"]
	if pool.Current != 9 {
		t.Fatalf("resource current = %d, want 9 after secret deploy", pool.Current)
	}
}

func TestPlayCardCharacterAllowsWhenActorHasPriorityButIsNotActivePlayer(t *testing.T) {
	state := basePlayCardState()
	state.Turn.ActivePlayerID = "P1"
	state.Turn.Priority.CurrentPlayerID = "P2"
	state.Turn.Resources["P2"] = PlayerResourceState{Current: 10, Max: 10}
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p2-char-non-active",
			Name:           "非当前行动玩家角色",
			Kind:           CardKindCharacter,
			OwnerID:        "P2",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Cost:           1,
		},
	)

	result, err := SubmitAction(state, Action{
		ID:                 "act-play-char-not-active",
		ActorID:            "P2",
		Kind:               ActionKindPlayCard,
		CardID:             "p2-char-non-active",
		PlayMode:           "face_down",
		TargetRegionCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	deployed := cardStateByID(t, result.State, "p2-char-non-active")
	if deployed.Zone != CardZoneTable {
		t.Fatalf("zone = %q, want %q", deployed.Zone, CardZoneTable)
	}
	if !deployed.FaceDown {
		t.Fatal("expected face-down deployment")
	}
}

func TestPlayCardFaceDownAllowsAfterBuildAssetWhenPriorityTransfersToP2(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "play-card-p2-after-build-asset",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           20260402,
	})
	state.Turn.Priority.CurrentPlayerID = "P2"
	state.Turn.PriorityPlayerID = "P2"
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p2-build-source",
			Name:           "P2 建立资产来源",
			Kind:           CardKindAsset,
			OwnerID:        "P2",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
		CardState{
			CardID:         "p2-secret-character",
			Name:           "P2 暗置角色",
			Kind:           CardKindCharacter,
			OwnerID:        "P2",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Cost:           2,
			Color:          "黄",
		},
	)

	result, err := SubmitAction(state, Action{
		ID:      "act-p2-build-asset",
		ActorID: "P2",
		Kind:    ActionKindBuildAsset,
		CardID:  "p2-build-source",
	})
	if err != nil {
		t.Fatalf("build_asset failed: %v", err)
	}

	_, err = SubmitAction(result.State, Action{
		ID:                 "act-p2-play-secret",
		ActorID:            "P2",
		Kind:               ActionKindPlayCard,
		CardID:             "p2-secret-character",
		PlayMode:           "face_down",
		TargetRegionCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("play_card face_down failed after build_asset: %v", err)
	}
}

func TestPlayCardRejectsWhenLoyaltyRequirementUnmet(t *testing.T) {
	state := basePlayCardState()
	state.Turn.Resources["P1"] = PlayerResourceState{Current: 4, Max: 4}
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p1-yellow-support",
			Name:           "黄色附属",
			Kind:           CardKindAsset,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			Color:          "黄",
			RegionCardID:   "region-1",
		},
		CardState{
			CardID:         "p1-loyal-card",
			Name:           "忠诚角色",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Cost:           1,
			Loyalty:        "黄色黄色",
		},
	)

	_, err := SubmitAction(state, Action{
		ID:                 "act-play-card-loyalty-fail",
		ActorID:            "P1",
		Kind:               ActionKindPlayCard,
		CardID:             "p1-loyal-card",
		PlayMode:           "face_up",
		TargetRegionCardID: "region-1",
	})
	if err == nil {
		t.Fatal("SubmitAction succeeded, want loyalty unmet error")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}
	if legalityErr.Code != ReasonCodeLegalityFailedActionProhibited {
		t.Fatalf("error code = %q, want %q", legalityErr.Code, ReasonCodeLegalityFailedActionProhibited)
	}
	if legalityErr.MessageKey != "rules.play_card.loyalty_unmet" {
		t.Fatalf("message key = %q, want %q", legalityErr.MessageKey, "rules.play_card.loyalty_unmet")
	}
}

func TestPlayCardAcceptsWhenLoyaltyRequirementMet(t *testing.T) {
	state := basePlayCardState()
	state.Turn.Resources["P1"] = PlayerResourceState{Current: 4, Max: 4}
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "region-1",
			Name:           "地区1",
			Kind:           CardKindRegion,
			OwnerID:        "TABLE",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			RegionOrder:    1,
		},
		CardState{
			CardID:         "p1-yellow-asset-1",
			Name:           "黄色附属1",
			Kind:           CardKindAsset,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			Color:          "黄",
			RegionCardID:   "region-1",
		},
		CardState{
			CardID:         "p1-yellow-asset-2",
			Name:           "黄色附属2",
			Kind:           CardKindAsset,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			Color:          "黄色",
			RegionCardID:   "region-1",
		},
		CardState{
			CardID:         "p1-loyal-pass",
			Name:           "忠诚通过角色",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Cost:           1,
			Loyalty:        "黄色黄色",
		},
	)

	result, err := SubmitAction(state, Action{
		ID:                 "act-play-card-loyalty-pass",
		ActorID:            "P1",
		Kind:               ActionKindPlayCard,
		CardID:             "p1-loyal-pass",
		PlayMode:           "face_up",
		TargetRegionCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	deployed := cardStateByID(t, result.State, "p1-loyal-pass")
	if deployed.Zone != CardZoneTable {
		t.Fatalf("zone = %q, want %q", deployed.Zone, CardZoneTable)
	}
}

func TestParseLoyaltyRequirementsCountsMixedCanonicalAndAlias(t *testing.T) {
	requirements := parseLoyaltyRequirements("黄黄色")

	if requirements["黄色"] != 2 {
		t.Fatalf("黄色 requirement = %d, want 2", requirements["黄色"])
	}
}

func basePlayCardState() GameState {
	state := NewGameState(InitialStateConfig{
		GameID:         "test-play-card",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           20260402,
	})
	state.Board.Cards = []CardState{}
	state.Board.Attachments = AttachmentRegistry{
		Active:           []Attachment{},
		NextAttachmentID: 1,
	}
	state.Board.CardMarkers = CardMarkerRegistry{
		ByCard: map[string]map[string]int{},
	}
	return state
}
