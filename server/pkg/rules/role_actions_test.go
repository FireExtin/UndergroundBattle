package rules

import (
	"errors"
	"reflect"
	"testing"
)

// Purpose: Verifies the first playable-loop role actions use authoritative stats, exhaustion, and replay-safe structured events.

func TestDeclareAttackAppliesCombatDamageAndExhaustsAttacker(t *testing.T) {
	state := newRoleActionTestState()
	state.Board.Cards = []CardState{
		testCharacterCard("p1-attacker", "P1", CardNumericStats{Combat: 2, Defense: 2}),
		testCharacterCard("p2-defender", "P2", CardNumericStats{Combat: 1, Defense: 4}),
	}

	result, err := SubmitAction(state, Action{
		ID:           "act-attack-1",
		ActorID:      "P1",
		Kind:         ActionKindDeclareAttack,
		CardID:       "p1-attacker",
		TargetCardID: "p2-defender",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Operation.Kind != OperationKindDeclareAttack {
		t.Fatalf("operation kind = %q, want %q", result.Operation.Kind, OperationKindDeclareAttack)
	}

	if result.Event.Kind != EventKindDamageApplied {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindDamageApplied)
	}

	if result.Event.SourceCardID != "p1-attacker" {
		t.Fatalf("event sourceCardId = %q, want %q", result.Event.SourceCardID, "p1-attacker")
	}

	if result.Event.ResolvedTargetID != "p2-defender" {
		t.Fatalf("event resolvedTargetId = %q, want %q", result.Event.ResolvedTargetID, "p2-defender")
	}

	if result.Event.AppliedAmount != 2 {
		t.Fatalf("event appliedAmount = %d, want 2", result.Event.AppliedAmount)
	}

	attacker := cardStateByID(t, result.State, "p1-attacker")
	if !attacker.Exhausted {
		t.Fatal("expected attacker to become exhausted")
	}

	defender := cardStateByID(t, result.State, "p2-defender")
	if defender.Counters.Damage != 2 {
		t.Fatalf("defender damage = %d, want 2", defender.Counters.Damage)
	}

	projected := cardViewByID(t, result.Views.Players["P1"].Board.Cards, "p1-attacker")
	if !projected.Exhausted {
		t.Fatal("expected projected attacker to be exhausted")
	}
}

func TestDeclareAttackCanDestroyTargetUsingEffectiveCombat(t *testing.T) {
	state := newRoleActionTestState()
	state.Board.Cards = []CardState{
		testCharacterCard("p1-attacker", "P1", CardNumericStats{Combat: 1, Defense: 2}),
		testCharacterCard("p2-defender", "P2", CardNumericStats{Combat: 1, Defense: 3}),
	}
	state.Board.Continuous.Active = []ContinuousEffect{
		{
			ID:           "ce:combat-buff",
			Layer:        LayerNumeric,
			EffectKind:   "modifyStat",
			TargetCardID: "p1-attacker",
			Stat:         "combat",
			Amount:       2,
			DurationKind: "permanent",
			Timestamp:    1,
		},
	}
	state = RecalculateContinuousEffects(state)

	result, err := SubmitAction(state, Action{
		ID:           "act-attack-lethal",
		ActorID:      "P1",
		Kind:         ActionKindDeclareAttack,
		CardID:       "p1-attacker",
		TargetCardID: "p2-defender",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Event.Kind != EventKindCardDestroyed {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindCardDestroyed)
	}

	if result.Event.AppliedAmount != 3 {
		t.Fatalf("event appliedAmount = %d, want 3", result.Event.AppliedAmount)
	}

	if result.Event.DestroyedCardID != "p2-defender" {
		t.Fatalf("destroyedCardId = %q, want %q", result.Event.DestroyedCardID, "p2-defender")
	}

	defender := cardStateByID(t, result.State, "p2-defender")
	if !defender.Destroyed {
		t.Fatal("expected target to be destroyed")
	}
	if defender.Zone != CardZoneDiscard {
		t.Fatalf("target zone = %q, want %q", defender.Zone, CardZoneDiscard)
	}
}

func TestDeclareAttackRejectsExhaustedAttacker(t *testing.T) {
	state := newRoleActionTestState()
	attacker := testCharacterCard("p1-attacker", "P1", CardNumericStats{Combat: 2, Defense: 2})
	attacker.Exhausted = true
	state.Board.Cards = []CardState{
		attacker,
		testCharacterCard("p2-defender", "P2", CardNumericStats{Combat: 1, Defense: 4}),
	}

	_, err := SubmitAction(state, Action{
		ID:           "act-attack-exhausted",
		ActorID:      "P1",
		Kind:         ActionKindDeclareAttack,
		CardID:       "p1-attacker",
		TargetCardID: "p2-defender",
	})
	if err == nil {
		t.Fatal("expected exhausted attacker to be rejected")
	}

	var legality *LegalityError
	if !errors.As(err, &legality) {
		t.Fatalf("expected LegalityError, got %T", err)
	}

	if legality.Code != ReasonCodeLegalityFailedActionProhibited {
		t.Fatalf("legality code = %q, want %q", legality.Code, ReasonCodeLegalityFailedActionProhibited)
	}
}

func TestDeclareInvestigationPlacesInfluenceOnRegionAndExhaustsInvestigator(t *testing.T) {
	state := newRoleActionTestState()
	state.Board.Cards = []CardState{
		testCharacterCard("p1-investigator", "P1", CardNumericStats{Combat: 1, Defense: 2, Investigation: 2}),
		testRegionCard("region-1"),
	}

	result, err := SubmitAction(state, Action{
		ID:           "act-investigation-1",
		ActorID:      "P1",
		Kind:         ActionKindDeclareInvestigation,
		CardID:       "p1-investigator",
		TargetCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Operation.Kind != OperationKindDeclareInvestigation {
		t.Fatalf("operation kind = %q, want %q", result.Operation.Kind, OperationKindDeclareInvestigation)
	}

	if result.Event.Kind != EventKindInvestigationApplied {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindInvestigationApplied)
	}

	if result.Event.AppliedAmount != 2 {
		t.Fatalf("event appliedAmount = %d, want 2", result.Event.AppliedAmount)
	}

	investigator := cardStateByID(t, result.State, "p1-investigator")
	if !investigator.Exhausted {
		t.Fatal("expected investigator to become exhausted")
	}

	region := cardStateByID(t, result.State, "region-1")
	if region.Counters.Influence != 2 {
		t.Fatalf("region influence = %d, want 2", region.Counters.Influence)
	}
}

func TestDeclareInvestigationUsesEffectiveInvestigationStat(t *testing.T) {
	state := newRoleActionTestState()
	state.Board.Cards = []CardState{
		testCharacterCard("p1-investigator", "P1", CardNumericStats{Combat: 1, Defense: 2, Investigation: 1}),
		testRegionCard("region-1"),
	}
	state.Board.Continuous.Active = []ContinuousEffect{
		{
			ID:           "ce:investigation-buff",
			Layer:        LayerNumeric,
			EffectKind:   "modifyStat",
			TargetCardID: "p1-investigator",
			Stat:         "investigation",
			Amount:       2,
			DurationKind: "permanent",
			Timestamp:    1,
		},
	}
	state = RecalculateContinuousEffects(state)

	result, err := SubmitAction(state, Action{
		ID:           "act-investigation-buffed",
		ActorID:      "P1",
		Kind:         ActionKindDeclareInvestigation,
		CardID:       "p1-investigator",
		TargetCardID: "region-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if cardStateByID(t, result.State, "region-1").Counters.Influence != 3 {
		t.Fatalf("region influence = %d, want 3", cardStateByID(t, result.State, "region-1").Counters.Influence)
	}
}

func TestRoleActionReplayProducesSameState(t *testing.T) {
	initial := newRoleActionTestState()
	initial.Board.Cards = []CardState{
		testCharacterCard("p1-attacker", "P1", CardNumericStats{Combat: 2, Defense: 2, Investigation: 1}),
		testCharacterCard("p2-defender", "P2", CardNumericStats{Combat: 1, Defense: 4}),
		testRegionCard("region-1"),
	}

	actions := []Action{
		{
			ID:           "act-role-replay-1",
			ActorID:      "P1",
			Kind:         ActionKindDeclareAttack,
			CardID:       "p1-attacker",
			TargetCardID: "p2-defender",
		},
		{
			ID:      "act-role-replay-2",
			ActorID: "P1",
			Kind:    ActionKindAdvancePhase,
		},
	}

	finalState, err := ReplayActions(initial, actions)
	if err != nil {
		t.Fatalf("ReplayActions returned error: %v", err)
	}

	replayed, err := ReplayActions(initial, finalState.History.Actions)
	if err != nil {
		t.Fatalf("ReplayActions(history) returned error: %v", err)
	}

	if !reflect.DeepEqual(finalState, replayed) {
		t.Fatalf("replayed state mismatch\nfinal = %#v\nreplayed = %#v", finalState, replayed)
	}
}

func newRoleActionTestState() GameState {
	return NewGameState(InitialStateConfig{
		GameID:         "game-role-actions",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           23,
	})
}

func testCharacterCard(cardID string, ownerID string, stats CardNumericStats) CardState {
	return CardState{
		CardID:         cardID,
		Name:           cardID,
		Kind:           CardKindCharacter,
		OwnerID:        ownerID,
		Zone:           CardZoneTable,
		VisibleToOwner: true,
		Revealed:       true,
		PrintedStats:   stats,
		EffectiveStats: stats,
	}
}

func testRegionCard(cardID string) CardState {
	return CardState{
		CardID:         cardID,
		Name:           "Forgotten District",
		Kind:           CardKindRegion,
		OwnerID:        "",
		Zone:           CardZoneTable,
		VisibleToOwner: false,
		Revealed:       true,
	}
}
