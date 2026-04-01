package rules

import (
	"fmt"
	"strings"
	"testing"
)

func TestRulebookFlow_C01_EndToMainDrawsOneCardPerPlayer(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "c01-draw-step",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Phase = phaseState(PhaseEnd)
	resetPriorityWindow(&state.Turn, "P1", PriorityWindowAction)

	result, err := SubmitAction(state, Action{
		ID:      "act-c01-end-to-main",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.State.Turn.Phase.Name != PhaseMain {
		t.Fatalf("phase = %q, want %q", result.State.Turn.Phase.Name, PhaseMain)
	}
	if result.State.Turn.TurnNumber != 2 {
		t.Fatalf("turn number = %d, want 2", result.State.Turn.TurnNumber)
	}
	if result.State.Turn.ActivePlayerID != "P2" {
		t.Fatalf("active player = %q, want %q", result.State.Turn.ActivePlayerID, "P2")
	}
	if result.State.Turn.Priority.CurrentPlayerID != "P2" {
		t.Fatalf("priority player = %q, want %q", result.State.Turn.Priority.CurrentPlayerID, "P2")
	}
	if result.State.Turn.Priority.WindowKind != PriorityWindowAction {
		t.Fatalf("priority window = %q, want %q", result.State.Turn.Priority.WindowKind, PriorityWindowAction)
	}
	if result.State.Turn.FirstPlayerPrivilegeUsed {
		t.Fatal("first-player privilege usage should reset to false at the start of a new turn")
	}

	if got := countCardsInZoneByOwner(result.State, CardZoneHand, "P1"); got != 1 {
		t.Fatalf("P1 hand count = %d, want 1", got)
	}
	if got := countCardsInZoneByOwner(result.State, CardZoneHand, "P2"); got != 1 {
		t.Fatalf("P2 hand count = %d, want 1", got)
	}
}

func TestRulebookFlow_C02_EndToMainRecoveryDiscardAndDamageCleanup(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "c02-recovery",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Phase = phaseState(PhaseEnd)
	resetPriorityWindow(&state.Turn, "P1", PriorityWindowAction)

	for i := 1; i <= 10; i++ {
		state.Board.Cards = append(state.Board.Cards,
			CardState{
				CardID:         fmt.Sprintf("p1-h%d", i),
				Name:           fmt.Sprintf("P1 Hand %d", i),
				OwnerID:        "P1",
				Zone:           CardZoneHand,
				Kind:           CardKindUnknown,
				VisibleToOwner: true,
			},
			CardState{
				CardID:         fmt.Sprintf("p2-h%d", i),
				Name:           fmt.Sprintf("P2 Hand %d", i),
				OwnerID:        "P2",
				Zone:           CardZoneHand,
				Kind:           CardKindUnknown,
				VisibleToOwner: true,
			},
		)
	}
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "p1-table-dmg",
			Name:           "P1 Damaged",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			Kind:           CardKindCharacter,
			VisibleToOwner: true,
			Revealed:       true,
			Counters:       CardCounters{Damage: 3},
		},
		CardState{
			CardID:         "p2-table-dmg",
			Name:           "P2 Damaged",
			OwnerID:        "P2",
			Zone:           CardZoneTable,
			Kind:           CardKindCharacter,
			VisibleToOwner: true,
			Revealed:       true,
			Counters:       CardCounters{Damage: 2},
		},
	)

	result, err := SubmitAction(state, Action{
		ID:      "act-c02-end-to-main",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if got := countCardsInZoneByOwner(result.State, CardZoneHand, "P1"); got != 8 {
		t.Fatalf("P1 hand count = %d, want 8 (discard to 7 then draw 1)", got)
	}
	if got := countCardsInZoneByOwner(result.State, CardZoneHand, "P2"); got != 8 {
		t.Fatalf("P2 hand count = %d, want 8 (discard to 7 then draw 1)", got)
	}

	for _, cardID := range []string{"p1-h8", "p1-h9", "p1-h10", "p2-h8", "p2-h9", "p2-h10"} {
		card := cardStateByID(t, result.State, cardID)
		if card.Zone != CardZoneDiscard {
			t.Fatalf("%s zone = %q, want %q after recovery discard", cardID, card.Zone, CardZoneDiscard)
		}
	}

	if cardStateByID(t, result.State, "p1-table-dmg").Counters.Damage != 0 {
		t.Fatal("P1 table character damage should be cleared in recovery")
	}
	if cardStateByID(t, result.State, "p2-table-dmg").Counters.Damage != 0 {
		t.Fatal("P2 table character damage should be cleared in recovery")
	}
}

func TestRulebookFlow_C03_RegionWinFlowMovesRegionAndRefills(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "c03-region-win",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Phase = phaseState(PhaseEnd)
	resetPriorityWindow(&state.Turn, "P1", PriorityWindowAction)

	region := testRegionCard("region-win-1")
	region.PrintedStats.Influence = 2 // Use printed influence as region win threshold in minimal model.
	region.EffectiveStats.Influence = 2
	region.InfluenceByPlayer = map[string]int{"P1": 2, "P2": 0}
	region.Counters.Influence = 2
	region.ControllerID = "P1"
	state.Board.Cards = []CardState{
		region,
		{
			CardID:         "region-win-unit-1",
			Name:           "Region Unit",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			RegionCardID:   "region-win-1",
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Combat: 1, Defense: 1},
			EffectiveStats: CardNumericStats{Combat: 1, Defense: 1},
		},
	}
	state.Board.Attachments.Active = []Attachment{
		{
			ID:                 "att:region-win-1",
			SourceDefinitionID: "TEST:ATTACHMENT",
			TargetCardID:       "region-win-unit-1",
			HostCardID:         "region-win-1",
			CreatedAtRevision:  0,
		},
	}

	result, err := SubmitAction(state, Action{
		ID:      "act-c03-end-to-main",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	won := cardStateByID(t, result.State, "region-win-1")
	if won.Zone != CardZoneScore {
		t.Fatalf("won region zone = %q, want %q", won.Zone, CardZoneScore)
	}
	if won.Destroyed {
		t.Fatal("won region should not be marked destroyed after moving to score zone")
	}
	if result.State.Score.ByPlayer["P1"] != 1 {
		t.Fatalf("P1 score = %d, want 1 from region win", result.State.Score.ByPlayer["P1"])
	}
	if got := cardStateByID(t, result.State, "region-win-unit-1").Zone; got != CardZoneDiscard {
		t.Fatalf("region unit zone = %q, want %q after region win cleanup", got, CardZoneDiscard)
	}
	if len(result.State.Board.Attachments.Active) != 0 {
		t.Fatalf("attachments should be pruned before score-zone move, got %#v", result.State.Board.Attachments.Active)
	}

	refillFound := false
	for _, card := range result.State.Board.Cards {
		if card.Kind == CardKindRegion && card.Zone == CardZoneTable && strings.HasPrefix(card.CardID, "region:auto:") {
			refillFound = true
			break
		}
	}
	if !refillFound {
		t.Fatal("expected an auto-refilled region card on table after region win")
	}
}

func TestRulebookFlow_C03_EffectiveThresholdOverridesPrintedAndPreventsEarlyWin(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "c03-effective-threshold",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Phase = phaseState(PhaseEnd)
	resetPriorityWindow(&state.Turn, "P1", PriorityWindowAction)

	region := testRegionCard("region-threshold")
	region.PrintedStats.Influence = 2
	region.EffectiveStats.Influence = 3
	region.InfluenceByPlayer = map[string]int{"P1": 2, "P2": 0}
	region.Counters.Influence = 2
	region.ControllerID = "P1"
	state.Board.Cards = []CardState{region}

	result, err := SubmitAction(state, Action{
		ID:      "act-c03-effective-threshold",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	card := cardStateByID(t, result.State, "region-threshold")
	if card.Zone != CardZoneTable {
		t.Fatalf("region zone = %q, want %q when effective threshold is not met", card.Zone, CardZoneTable)
	}
	if result.State.Score.ByPlayer["P1"] != 1 {
		t.Fatalf("P1 score = %d, want 1 from controlled-region scoring only", result.State.Score.ByPlayer["P1"])
	}
	for _, c := range result.State.Board.Cards {
		if strings.HasPrefix(c.CardID, "region:auto:") {
			t.Fatalf("unexpected auto region refill %q when threshold not met", c.CardID)
		}
	}
}

func TestRulebookFlow_C05C06_FirstPlayerPrivilegeBreaksNonZeroTieOnce(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "c05c06-privilege",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	region := testRegionCard("region-tie")
	region.InfluenceByPlayer = map[string]int{"P1": 2, "P2": 2}
	region.Counters.Influence = 4
	state.Board.Cards = []CardState{region}

	state.Board.Markers.SetMarker("P1", markerTypeFirstPlayerPrivilegeRequest, 1)
	refreshAllRegionControl(&state)

	if got := cardStateByID(t, state, "region-tie").ControllerID; got != "P1" {
		t.Fatalf("controller after privilege = %q, want %q", got, "P1")
	}
	if got := state.Board.Markers.GetMarker("P1", markerTypeFirstPlayerPrivilegeUsed); got != 1 {
		t.Fatalf("privilege used marker = %d, want 1", got)
	}
	if got := state.Board.Markers.GetMarker("P1", markerTypeFirstPlayerPrivilegeRequest); got != 0 {
		t.Fatalf("privilege request marker should be cleared, got %d", got)
	}
	if !state.Turn.FirstPlayerPrivilegeUsed {
		t.Fatal("turn.firstPlayerPrivilegeUsed should be true after privilege is consumed")
	}

	state.Board.Markers.SetMarker("P1", markerTypeFirstPlayerPrivilegeRequest, 1)
	refreshAllRegionControl(&state)
	if got := state.Board.Markers.GetMarker("P1", markerTypeFirstPlayerPrivilegeUsed); got != 1 {
		t.Fatalf("privilege used marker should stay 1 after second tie, got %d", got)
	}
}

func TestRulebookFlow_C05_ZeroVsZeroDoesNotConsumePrivilege(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "c05-zero-vs-zero",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	region := testRegionCard("region-zero")
	region.InfluenceByPlayer = map[string]int{"P1": 0, "P2": 0}
	region.Counters.Influence = 0
	state.Board.Cards = []CardState{region}

	state.Board.Markers.SetMarker("P1", markerTypeFirstPlayerPrivilegeRequest, 1)
	refreshAllRegionControl(&state)

	if got := cardStateByID(t, state, "region-zero").ControllerID; got != "" {
		t.Fatalf("controller = %q, want empty on 0v0 not-occurred", got)
	}
	if got := state.Board.Markers.GetMarker("P1", markerTypeFirstPlayerPrivilegeUsed); got != 0 {
		t.Fatalf("privilege used marker = %d, want 0 on not-occurred contest", got)
	}
}

func TestRulebookFlow_EndToMainResetsPrivilegeMarkersForAllPlayers(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "c06-reset-privilege-markers",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Phase = phaseState(PhaseEnd)
	resetPriorityWindow(&state.Turn, "P1", PriorityWindowAction)
	state.Turn.FirstPlayerPrivilegeUsed = true
	state.Board.Markers.SetMarker("P1", markerTypeFirstPlayerPrivilegeRequest, 1)
	state.Board.Markers.SetMarker("P1", markerTypeFirstPlayerPrivilegeUsed, 1)
	state.Board.Markers.SetMarker("P2", markerTypeFirstPlayerPrivilegeRequest, 1)
	state.Board.Markers.SetMarker("P2", markerTypeFirstPlayerPrivilegeUsed, 1)

	result, err := SubmitAction(state, Action{
		ID:      "act-c06-reset-privilege-markers",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.State.Turn.FirstPlayerPrivilegeUsed {
		t.Fatal("turn.firstPlayerPrivilegeUsed should be false on new turn")
	}
	for _, playerID := range []string{"P1", "P2"} {
		if got := result.State.Board.Markers.GetMarker(playerID, markerTypeFirstPlayerPrivilegeRequest); got != 0 {
			t.Fatalf("%s request marker = %d, want 0", playerID, got)
		}
		if got := result.State.Board.Markers.GetMarker(playerID, markerTypeFirstPlayerPrivilegeUsed); got != 0 {
			t.Fatalf("%s used marker = %d, want 0", playerID, got)
		}
	}
}

func TestRulebookFlow_C14_DrawFromEmptyDeckSinglePlayerLoses(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "c14-single-empty-deck",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Phase = phaseState(PhaseEnd)
	resetPriorityWindow(&state.Turn, "P1", PriorityWindowAction)
	state.Board.Cards = append(state.Board.Cards, CardState{
		CardID:         "p2-deck-1",
		Name:           "P2 Deck Card",
		OwnerID:        "P2",
		Zone:           CardZoneDeck,
		VisibleToOwner: false,
		Revealed:       false,
	})

	result, err := SubmitAction(state, Action{
		ID:      "act-c14-single-empty-deck",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.State.Match.Status != MatchStatusFinished {
		t.Fatalf("match status = %q, want %q", result.State.Match.Status, MatchStatusFinished)
	}
	if result.State.Match.EndReason != MatchEndReasonDeckOut {
		t.Fatalf("match end reason = %q, want %q", result.State.Match.EndReason, MatchEndReasonDeckOut)
	}
	if result.State.Match.WinnerPlayerID != "P2" {
		t.Fatalf("winner = %q, want %q", result.State.Match.WinnerPlayerID, "P2")
	}
	if result.State.Score.WinnerPlayerID != "P2" {
		t.Fatalf("score winner = %q, want %q", result.State.Score.WinnerPlayerID, "P2")
	}
	if got := countCardsInZoneByOwner(result.State, CardZoneHand, "P2"); got != 1 {
		t.Fatalf("P2 hand count = %d, want 1 after successful draw", got)
	}
}

func TestRulebookFlow_C14_DrawFromEmptyDeckBothPlayersDrawMatch(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "c14-both-empty-deck",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Phase = phaseState(PhaseEnd)
	resetPriorityWindow(&state.Turn, "P1", PriorityWindowAction)
	// Deck model is active because a deck card exists, but both players will fail to draw simultaneously.
	state.Board.Cards = append(state.Board.Cards, CardState{
		CardID:         "deck-anchor-other-owner",
		Name:           "Deck Anchor",
		OwnerID:        "WORLD",
		Zone:           CardZoneDeck,
		VisibleToOwner: false,
		Revealed:       false,
	})

	result, err := SubmitAction(state, Action{
		ID:      "act-c14-both-empty-deck",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.State.Match.Status != MatchStatusFinished {
		t.Fatalf("match status = %q, want %q", result.State.Match.Status, MatchStatusFinished)
	}
	if result.State.Match.EndReason != MatchEndReasonDeckOutDraw {
		t.Fatalf("match end reason = %q, want %q", result.State.Match.EndReason, MatchEndReasonDeckOutDraw)
	}
	if result.State.Match.WinnerPlayerID != "" {
		t.Fatalf("winner = %q, want empty on simultaneous deck-out", result.State.Match.WinnerPlayerID)
	}
	if result.State.Score.WinnerPlayerID != "" {
		t.Fatalf("score winner = %q, want empty on simultaneous deck-out", result.State.Score.WinnerPlayerID)
	}
}

func TestRegionControl_RefreshRegionControlWrapperStillComputesNonTieControl(t *testing.T) {
	region := testRegionCard("region-wrapper")
	region.InfluenceByPlayer = map[string]int{"P1": 2, "P2": 1}

	refreshRegionControl(&region)

	if region.ControllerID != "P1" {
		t.Fatalf("controller = %q, want %q", region.ControllerID, "P1")
	}
}

func TestRulebookFlow_NextAutoRegionIndexSkipsInvalidAndReturnsMaxPlusOne(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "c03-region-index",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Board.Cards = []CardState{
		testRegionCard("region:auto:2"),
		testRegionCard("region:auto:7"),
		testRegionCard("region:auto:bad"),
		testRegionCard("region:manual"),
	}

	if got := nextAutoRegionIndex(state); got != 8 {
		t.Fatalf("nextAutoRegionIndex = %d, want 8", got)
	}
}

func countCardsInZoneByOwner(state GameState, zone CardZone, ownerID string) int {
	count := 0
	for _, card := range state.Board.Cards {
		if card.Zone == zone && card.OwnerID == ownerID {
			count++
		}
	}
	return count
}
