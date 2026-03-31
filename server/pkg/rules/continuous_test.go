package rules

import (
	"reflect"
	"testing"
)

// Purpose: Verifies minimal continuous-effect recalculation, commit integration, and DSL effect routing behavior.

func TestRecalculateContinuousEffectsAppliesNumericModifier(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:       "card-numeric-1",
			Name:         "Numeric Target",
			OwnerID:      "P1",
			Zone:         CardZoneTable,
			Revealed:     true,
			PrintedStats: CardNumericStats{Defense: 1},
		},
	}
	state.Board.Continuous = ContinuousEffectRegistry{
		Active: []ContinuousEffect{
			{
				ID:           "ce:numeric-1",
				Layer:        LayerNumeric,
				EffectKind:   "modifyStat",
				TargetCardID: "card-numeric-1",
				Stat:         "defense",
				Amount:       2,
				DurationKind: "permanent",
				Timestamp:    1,
			},
		},
	}

	recalculated := RecalculateContinuousEffects(state)
	target := cardStateByID(t, recalculated, "card-numeric-1")

	if target.EffectiveStats.Defense != 3 {
		t.Fatalf("effective defense = %d, want 3", target.EffectiveStats.Defense)
	}
}

func TestRecalculateContinuousEffectsProhibitionPrecedesPermission(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:   "card-policy-1",
			Name:     "Policy Target",
			OwnerID:  "P1",
			Zone:     CardZoneTable,
			Revealed: true,
		},
	}
	state.Board.Continuous = ContinuousEffectRegistry{
		Active: []ContinuousEffect{
			{
				ID:           "ce:prohibit-1",
				Layer:        LayerProhibition,
				EffectKind:   "prohibitPermission",
				TargetCardID: "card-policy-1",
				Permission:   "activate",
				DurationKind: "permanent",
				Timestamp:    1,
			},
			{
				ID:           "ce:permit-1",
				Layer:        LayerPermission,
				EffectKind:   "grantPermission",
				TargetCardID: "card-policy-1",
				Permission:   "activate",
				DurationKind: "permanent",
				Timestamp:    2,
			},
		},
	}

	recalculated := RecalculateContinuousEffects(state)
	if isCardActionAllowed(recalculated, "card-policy-1", "activate") {
		t.Fatal("expected prohibition to override permission for activate")
	}
}

func TestTurnDurationContinuousEffectExpiresAfterTurnEnds(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:       "card-turn-1",
			Name:         "Turn Target",
			OwnerID:      "P1",
			Zone:         CardZoneTable,
			Revealed:     true,
			PrintedStats: CardNumericStats{Defense: 1},
		},
	}
	state.Board.Continuous = ContinuousEffectRegistry{
		Active: []ContinuousEffect{
			{
				ID:            "ce:turn-1",
				Layer:         LayerNumeric,
				EffectKind:    "modifyStat",
				TargetCardID:  "card-turn-1",
				Stat:          "defense",
				Amount:        2,
				DurationKind:  "turn",
				ExpiresAtTurn: 1,
				Timestamp:     1,
			},
		},
	}

	currentTurn := RecalculateContinuousEffects(state)
	if cardStateByID(t, currentTurn, "card-turn-1").EffectiveStats.Defense != 3 {
		t.Fatalf("turn-1 effective defense = %d, want 3", cardStateByID(t, currentTurn, "card-turn-1").EffectiveStats.Defense)
	}

	nextTurn := cloneGameState(currentTurn)
	nextTurn.Turn.TurnNumber = 2
	expired := RecalculateContinuousEffects(nextTurn)
	target := cardStateByID(t, expired, "card-turn-1")

	if target.EffectiveStats.Defense != 1 {
		t.Fatalf("turn-2 effective defense = %d, want 1", target.EffectiveStats.Defense)
	}

	if len(expired.Board.Continuous.Active) != 0 {
		t.Fatalf("active continuous effects = %d, want 0", len(expired.Board.Continuous.Active))
	}
}

func TestContinuousEffectIsRemovedWhenSourceLeavesTable(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "card-source-1",
			Name:           "Effect Source",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
		{
			CardID:         "card-target-1",
			Name:           "Effect Target",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Defense: 1},
		},
	}
	state.Board.Continuous = ContinuousEffectRegistry{
		Active: []ContinuousEffect{
			{
				ID:           "ce:source-leaves-1",
				SourceCardID: "card-source-1",
				Layer:        LayerNumeric,
				EffectKind:   "modifyStat",
				TargetCardID: "card-target-1",
				Stat:         "defense",
				Amount:       2,
				DurationKind: "permanent",
				Timestamp:    1,
			},
		},
	}

	current := RecalculateContinuousEffects(state)
	if cardStateByID(t, current, "card-target-1").EffectiveStats.Defense != 3 {
		t.Fatalf("effective defense while source is in play = %d, want 3", cardStateByID(t, current, "card-target-1").EffectiveStats.Defense)
	}

	sourceLeaves := cloneGameState(current)
	sourceIndex := findCardIndex(sourceLeaves, "card-source-1")
	sourceLeaves.Board.Cards[sourceIndex].Zone = CardZoneDiscard
	sourceLeaves.Board.Cards[sourceIndex].Destroyed = true

	cleaned := RecalculateContinuousEffects(sourceLeaves)
	target := cardStateByID(t, cleaned, "card-target-1")

	if target.EffectiveStats.Defense != 1 {
		t.Fatalf("effective defense after source leaves table = %d, want 1", target.EffectiveStats.Defense)
	}

	if len(cleaned.Board.Continuous.Active) != 0 {
		t.Fatalf("active continuous effects = %d, want 0", len(cleaned.Board.Continuous.Active))
	}
}

func TestCommitRecalculatesContinuousEffectsOnceWhenOneOperationRegistersMultipleEffects(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "card-commit-1",
			Name:           "Commit Target",
			OwnerID:        "P2",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Defense: 1},
		},
	}

	action := Action{
		ID:           "act-continuous-commit-1",
		ActorID:      "P1",
		Kind:         ActionKindQueueOperation,
		CardID:       "MANUAL-CONTINUOUS",
		TargetCardID: "card-commit-1",
	}
	operation := manualDSLCardEffectOperation(
		"op:act-continuous-commit-1",
		"act-continuous-commit-1",
		"P1",
		"MANUAL-CONTINUOUS",
		"card-commit-1",
		"turn",
		[]EffectSpec{
			{
				Kind:      "addKeyword",
				TargetRef: "selected",
				Keyword:   "blackBlade",
			},
			{
				Kind:      "modifyStat",
				TargetRef: "selected",
				Stat:      "defense",
				Amount:    intPtr(2),
			},
		},
	)

	working, resolved, event, err := executeOperation(state, operation)
	if err != nil {
		t.Fatalf("executeOperation returned error: %v", err)
	}

	result := commitState(working, action, resolved, event, nil)
	target := cardStateByID(t, result.State, "card-commit-1")

	if result.State.Board.Continuous.FullRecalculationCount != 1 {
		t.Fatalf("full recalculation count = %d, want 1", result.State.Board.Continuous.FullRecalculationCount)
	}

	if !containsString(target.EffectiveKeywords, "blackBlade") {
		t.Fatalf("effective keywords = %v, want blackBlade", target.EffectiveKeywords)
	}

	if target.EffectiveStats.Defense != 3 {
		t.Fatalf("effective defense = %d, want 3", target.EffectiveStats.Defense)
	}
}

func TestRecalculateContinuousEffectsIsIdempotent(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:          "card-idempotent-1",
			Name:            "Idempotent Target",
			OwnerID:         "P1",
			Zone:            CardZoneTable,
			VisibleToOwner:  true,
			Revealed:        true,
			PrintedKeywords: []string{"printed"},
			PrintedStats:    CardNumericStats{Defense: 1},
		},
	}
	state.Board.Continuous = ContinuousEffectRegistry{
		Active: []ContinuousEffect{
			{
				ID:           "ce:idem-1",
				Layer:        LayerNumeric,
				EffectKind:   "addKeyword",
				TargetCardID: "card-idempotent-1",
				Keyword:      "blackBlade",
				DurationKind: "permanent",
				Timestamp:    1,
			},
			{
				ID:           "ce:idem-2",
				Layer:        LayerNumeric,
				EffectKind:   "modifyStat",
				TargetCardID: "card-idempotent-1",
				Stat:         "defense",
				Amount:       2,
				DurationKind: "permanent",
				Timestamp:    2,
			},
		},
	}

	left := RecalculateContinuousEffects(state)
	right := RecalculateContinuousEffects(state)

	if !reflect.DeepEqual(left, right) {
		t.Fatalf("recalculated states differ\nleft  = %#v\nright = %#v", left, right)
	}
}

func TestRecalculateContinuousEffectsPreventsRecursiveLoopWhenEffectsExpire(t *testing.T) {
	state := newContinuousTestState()
	state.Turn.TurnNumber = 2
	state.Board.Cards = []CardState{
		{
			CardID:       "card-cycle-1",
			Name:         "Cycle Target",
			OwnerID:      "P1",
			Zone:         CardZoneTable,
			Revealed:     true,
			PrintedStats: CardNumericStats{Defense: 1},
		},
	}
	state.Board.Continuous = ContinuousEffectRegistry{
		Active: []ContinuousEffect{
			{
				ID:            "ce:cycle-1",
				Layer:         LayerNumeric,
				EffectKind:    "modifyStat",
				TargetCardID:  "card-cycle-1",
				Stat:          "defense",
				Amount:        2,
				DurationKind:  "turn",
				ExpiresAtTurn: 1,
				Timestamp:     1,
			},
		},
		PendingRecalculation: true,
	}

	recalculated := RecalculateContinuousEffects(state)
	target := cardStateByID(t, recalculated, "card-cycle-1")

	if recalculated.Board.Continuous.FullRecalculationCount != 1 {
		t.Fatalf("full recalculation count = %d, want 1", recalculated.Board.Continuous.FullRecalculationCount)
	}

	if recalculated.Board.Continuous.CycleGuardTrips != 1 {
		t.Fatalf("cycle guard trips = %d, want 1", recalculated.Board.Continuous.CycleGuardTrips)
	}

	if len(recalculated.Board.Continuous.Active) != 0 {
		t.Fatalf("active continuous effects = %d, want 0", len(recalculated.Board.Continuous.Active))
	}

	if target.EffectiveStats.Defense != 1 {
		t.Fatalf("effective defense = %d, want 1", target.EffectiveStats.Defense)
	}
}

func TestPureDSLAttachmentRegistersContinuousKeywordEffect(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "card-keyword-1",
			Name:           "Keyword Target",
			OwnerID:        "P2",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
	}

	result, err := SubmitAction(state, Action{
		ID:           "act-keyword-fixture-1",
		ActorID:      "P1",
		Kind:         ActionKindQueueOperation,
		CardID:       "BQ022",
		TargetCardID: "card-keyword-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if len(result.State.Board.Continuous.Active) != 1 {
		t.Fatalf("active continuous effects = %d, want 1", len(result.State.Board.Continuous.Active))
	}
	if len(result.State.Board.Attachments.Active) != 1 {
		t.Fatalf("active attachments = %d, want 1", len(result.State.Board.Attachments.Active))
	}

	effect := result.State.Board.Continuous.Active[0]
	if effect.EffectKind != "addKeyword" {
		t.Fatalf("effect kind = %q, want addKeyword", effect.EffectKind)
	}

	if effect.Layer != LayerNumeric {
		t.Fatalf("effect layer = %q, want %q", effect.Layer, LayerNumeric)
	}

	target := cardStateByID(t, result.State, "card-keyword-1")
	if !containsString(target.EffectiveKeywords, "blackBlade") {
		t.Fatalf("effective keywords = %v, want blackBlade", target.EffectiveKeywords)
	}

	attachment := result.State.Board.Attachments.Active[0]
	if attachment.SourceDefinitionID != "BQ022" {
		t.Fatalf("attachment sourceDefinitionId = %q, want BQ022", attachment.SourceDefinitionID)
	}
	if attachment.SourceOperationID != result.Operation.ID {
		t.Fatalf("attachment sourceOperationId = %q, want %q", attachment.SourceOperationID, result.Operation.ID)
	}
	if attachment.TargetCardID != "card-keyword-1" {
		t.Fatalf("attachment targetCardId = %q, want card-keyword-1", attachment.TargetCardID)
	}
}

func TestPureDSLAttachmentPrunesContinuousEffectWhenTargetLeavesTable(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "card-keyword-1",
			Name:           "Keyword Target",
			OwnerID:        "P2",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
	}

	result, err := SubmitAction(state, Action{
		ID:           "act-keyword-fixture-prune",
		ActorID:      "P1",
		Kind:         ActionKindQueueOperation,
		CardID:       "BQ022",
		TargetCardID: "card-keyword-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	leavesTable := cloneGameState(result.State)
	targetIndex := findCardIndex(leavesTable, "card-keyword-1")
	leavesTable.Board.Cards[targetIndex].Zone = CardZoneDiscard
	leavesTable.Board.Cards[targetIndex].Destroyed = true

	recalculated := RecalculateContinuousEffects(leavesTable)

	if len(recalculated.Board.Attachments.Active) != 0 {
		t.Fatalf("active attachments after target leaves = %d, want 0", len(recalculated.Board.Attachments.Active))
	}
	if len(recalculated.Board.Continuous.Active) != 0 {
		t.Fatalf("active continuous effects after target leaves = %d, want 0", len(recalculated.Board.Continuous.Active))
	}

	target := cardStateByID(t, recalculated, "card-keyword-1")
	if containsString(target.EffectiveKeywords, "blackBlade") {
		t.Fatalf("effective keywords after target leaves = %v, blackBlade should be gone", target.EffectiveKeywords)
	}
}

func TestResolveDSLCardRoutesDamageAndInfluenceToCounters(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "card-counter-1",
			Name:           "Counter Target",
			OwnerID:        "P2",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
	}

	damageState, _, err := resolveDSLCardEffect(
		state,
		manualDSLCardEffectOperation(
			"op:damage-1",
			"act:damage-1",
			"P1",
			"MANUAL-DAMAGE",
			"card-counter-1",
			"none",
			[]EffectSpec{
				{
					Kind:      "dealDamage",
					TargetRef: "selected",
					Amount:    intPtr(4),
				},
			},
		),
	)
	if err != nil {
		t.Fatalf("resolveDSLCardEffect(damage) returned error: %v", err)
	}

	if cardStateByID(t, damageState, "card-counter-1").Counters.Damage != 4 {
		t.Fatalf("damage counter = %d, want 4", cardStateByID(t, damageState, "card-counter-1").Counters.Damage)
	}

	if len(damageState.Board.Continuous.Active) != 0 {
		t.Fatalf("damage path should not register continuous effects: %d active", len(damageState.Board.Continuous.Active))
	}

	influenceState, _, err := resolveDSLCardEffect(
		state,
		manualDSLCardEffectOperation(
			"op:influence-1",
			"act:influence-1",
			"P1",
			"MANUAL-INFLUENCE",
			"card-counter-1",
			"none",
			[]EffectSpec{
				{
					Kind:      "placeInfluence",
					TargetRef: "selected",
					Amount:    intPtr(2),
				},
			},
		),
	)
	if err != nil {
		t.Fatalf("resolveDSLCardEffect(influence) returned error: %v", err)
	}

	if cardStateByID(t, influenceState, "card-counter-1").Counters.Influence != 2 {
		t.Fatalf("influence counter = %d, want 2", cardStateByID(t, influenceState, "card-counter-1").Counters.Influence)
	}

	if len(influenceState.Board.Continuous.Active) != 0 {
		t.Fatalf("influence path should not register continuous effects: %d active", len(influenceState.Board.Continuous.Active))
	}
}

func TestInspectCardRejectedWhenContinuousProhibitionBlocksAction(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "card-inspect-prohibited-1",
			Name:           "Forbidden Archive",
			OwnerID:        "P2",
			Zone:           CardZoneDeck,
			VisibleToOwner: false,
		},
	}
	state.Board.Continuous = ContinuousEffectRegistry{
		Active: []ContinuousEffect{
			{
				ID:           "ce:prohibit-inspect-1",
				Layer:        LayerProhibition,
				EffectKind:   "prohibitPermission",
				TargetCardID: "card-inspect-prohibited-1",
				Permission:   "inspect",
				DurationKind: "permanent",
				Timestamp:    1,
			},
		},
	}

	locked := RecalculateContinuousEffects(state)
	_, err := SubmitAction(locked, Action{
		ID:      "act-inspect-prohibited-1",
		ActorID: "P1",
		Kind:    ActionKindInspectCard,
		CardID:  "card-inspect-prohibited-1",
	})
	if err == nil {
		t.Fatal("expected inspect action to be rejected by prohibition")
	}

	legality, ok := err.(*LegalityError)
	if !ok {
		t.Fatalf("expected LegalityError, got %T", err)
	}

	if legality.Code != ReasonCodeLegalityFailedActionProhibited {
		t.Fatalf("legality code = %q, want %q", legality.Code, ReasonCodeLegalityFailedActionProhibited)
	}
}

func TestInspectCardRequiresGrantedPermission(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:              "card-inspect-required-1",
			Name:                "Locked Archive",
			OwnerID:             "P2",
			Zone:                CardZoneDeck,
			VisibleToOwner:      false,
			RequiredPermissions: []string{"inspect"},
		},
	}

	_, err := SubmitAction(state, Action{
		ID:      "act-inspect-required-1",
		ActorID: "P1",
		Kind:    ActionKindInspectCard,
		CardID:  "card-inspect-required-1",
	})
	if err == nil {
		t.Fatal("expected inspect action to require an explicit permission grant")
	}

	legality, ok := err.(*LegalityError)
	if !ok {
		t.Fatalf("expected LegalityError, got %T", err)
	}

	if legality.Code != ReasonCodeLegalityFailedPermissionRequired {
		t.Fatalf("legality code = %q, want %q", legality.Code, ReasonCodeLegalityFailedPermissionRequired)
	}
}

func TestInspectCardSucceedsWhenPermissionIsGranted(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:              "card-inspect-granted-1",
			Name:                "Grantable Archive",
			OwnerID:             "P2",
			Zone:                CardZoneDeck,
			VisibleToOwner:      false,
			RequiredPermissions: []string{"inspect"},
		},
	}
	state.Board.Continuous = ContinuousEffectRegistry{
		Active: []ContinuousEffect{
			{
				ID:           "ce:grant-inspect-1",
				Layer:        LayerPermission,
				EffectKind:   "grantPermission",
				TargetCardID: "card-inspect-granted-1",
				Permission:   "inspect",
				DurationKind: "permanent",
				Timestamp:    1,
			},
		},
	}

	granted := RecalculateContinuousEffects(state)
	result, err := SubmitAction(granted, Action{
		ID:      "act-inspect-granted-1",
		ActorID: "P1",
		Kind:    ActionKindInspectCard,
		CardID:  "card-inspect-granted-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	target := cardStateByID(t, result.State, "card-inspect-granted-1")
	if !containsString(target.InspectedBy, "P1") {
		t.Fatalf("inspectedBy = %v, want P1 included", target.InspectedBy)
	}
}

func TestDamageEqualToEffectiveDefenseDestroysCard(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "card-lethal-1",
			Name:           "Lethal Target",
			OwnerID:        "P2",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Defense: 2},
		},
	}

	action := Action{
		ID:           "act-lethal-damage-1",
		ActorID:      "P1",
		Kind:         ActionKindQueueOperation,
		CardID:       "MANUAL-DAMAGE",
		TargetCardID: "card-lethal-1",
	}
	operation := manualDSLCardEffectOperation(
		"op:act-lethal-damage-1",
		"act-lethal-damage-1",
		"P1",
		"MANUAL-DAMAGE",
		"card-lethal-1",
		"none",
		[]EffectSpec{
			{
				Kind:      "dealDamage",
				TargetRef: "selected",
				Amount:    intPtr(2),
			},
		},
	)

	working, resolved, event, err := executeOperation(state, operation)
	if err != nil {
		t.Fatalf("executeOperation returned error: %v", err)
	}

	result := commitState(working, action, resolved, event, nil)
	target := cardStateByID(t, result.State, "card-lethal-1")

	if !target.Destroyed {
		t.Fatal("expected target to be destroyed by lethal damage")
	}

	if target.Zone != CardZoneDiscard {
		t.Fatalf("zone = %q, want %q", target.Zone, CardZoneDiscard)
	}
}

func TestPlaceInfluenceOnRegionTracksControllerForActor(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		testRegionCard("region-dsl-1"),
	}

	action := Action{
		ID:           "act-region-dsl-1",
		ActorID:      "P1",
		Kind:         ActionKindQueueOperation,
		CardID:       "MANUAL-INFLUENCE",
		TargetCardID: "region-dsl-1",
	}
	operation := manualDSLCardEffectOperation(
		"op:act-region-dsl-1",
		"act-region-dsl-1",
		"P1",
		"MANUAL-INFLUENCE",
		"region-dsl-1",
		"none",
		[]EffectSpec{
			{
				Kind:      "placeInfluence",
				TargetRef: "selected",
				Amount:    intPtr(2),
			},
		},
	)

	working, resolved, event, err := executeOperation(state, operation)
	if err != nil {
		t.Fatalf("executeOperation returned error: %v", err)
	}

	result := commitState(working, action, resolved, event, nil)
	region := cardStateByID(t, result.State, "region-dsl-1")

	if region.Counters.Influence != 2 {
		t.Fatalf("region influence = %d, want 2", region.Counters.Influence)
	}
	if region.InfluenceByPlayer["P1"] != 2 {
		t.Fatalf("P1 region influence = %d, want 2", region.InfluenceByPlayer["P1"])
	}
	if region.ControllerID != "P1" {
		t.Fatalf("controller = %q, want %q", region.ControllerID, "P1")
	}
}

func TestModifyStatCanPreventLethalDamageUntilEffectExpires(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "card-buffed-1",
			Name:           "Buffed Target",
			OwnerID:        "P2",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Defense: 2},
			Counters:       CardCounters{Damage: 3},
		},
	}
	state.Board.Continuous = ContinuousEffectRegistry{
		Active: []ContinuousEffect{
			{
				ID:            "ce:defense-buff-1",
				Layer:         LayerNumeric,
				EffectKind:    "modifyStat",
				TargetCardID:  "card-buffed-1",
				Stat:          "defense",
				Amount:        2,
				DurationKind:  "turn",
				ExpiresAtTurn: 1,
				Timestamp:     1,
			},
		},
		PendingRecalculation: true,
	}

	currentTurn := RecalculateContinuousEffects(state)
	target := cardStateByID(t, currentTurn, "card-buffed-1")
	if target.Destroyed {
		t.Fatal("expected defense buff to keep target alive this turn")
	}

	if target.EffectiveStats.Defense != 4 {
		t.Fatalf("effective defense = %d, want 4", target.EffectiveStats.Defense)
	}

	nextTurn := cloneGameState(currentTurn)
	nextTurn.Turn.TurnNumber = 2
	expired := RecalculateContinuousEffects(nextTurn)
	expiredTarget := cardStateByID(t, expired, "card-buffed-1")

	if !expiredTarget.Destroyed {
		t.Fatal("expected target to be destroyed after buff expires")
	}

	if expiredTarget.Zone != CardZoneDiscard {
		t.Fatalf("zone = %q, want %q", expiredTarget.Zone, CardZoneDiscard)
	}
}

func newContinuousTestState() GameState {
	return NewGameState(InitialStateConfig{
		GameID:         "game-continuous",
		ActivePlayerID: "P1",
		Seed:           17,
	})
}

func manualDSLCardEffectOperation(
	operationID string,
	actionID string,
	actorID string,
	cardID string,
	targetCardID string,
	durationKind string,
	effects []EffectSpec,
) Operation {
	return Operation{
		ID:            operationID,
		ActionID:      actionID,
		ActorID:       actorID,
		Kind:          OperationKindCardEffect,
		Status:        OperationStatusBuilt,
		CardID:        cardID,
		TargetCardID:  targetCardID,
		RequiresStack: false,
		Source: &CardOperationSource{
			CardID:            cardID,
			CardName:          "Manual DSL",
			LogicID:           "manual.dsl",
			Speed:             "slow",
			TargetKinds:       []string{"character"},
			RequiresStack:     false,
			ExecutionKind:     CardExecutionDSL,
			DurationKind:      durationKind,
			RequiresScript:    false,
			PureDSLExecutable: true,
			Effects:           effects,
		},
	}
}

func intPtr(value int) *int {
	return &value
}

func TestAttachmentNotCreatedWhenTargetNotOnTable(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "attachment-source",
			Name:           "Test Attachment",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
		{
			CardID:         "target-in-discard",
			Name:           "Target in Discard",
			OwnerID:        "P1",
			Zone:           CardZoneDiscard,
			VisibleToOwner: true,
			Revealed:       true,
		},
	}

	operation := manualAttachmentOperation(
		"op:attach-test",
		"act:attach-test",
		"P1",
		"attachment-source",
		"target-in-discard",
	)

	result := registerContinuousEffect(state, operation, EffectSpec{
		Kind:      "addKeyword",
		TargetRef: "selected",
		Keyword:   "test",
	})

	if len(result.Board.Attachments.Active) != 0 {
		t.Fatalf("expected no attachment created when target is in discard, got %d", len(result.Board.Attachments.Active))
	}
}

func TestAttachmentNotCreatedWhenTargetDestroyed(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "attachment-source",
			Name:           "Test Attachment",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
		{
			CardID:         "target-destroyed",
			Name:           "Target Destroyed",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			Destroyed:      true,
		},
	}

	operation := manualAttachmentOperation(
		"op:attach-test",
		"act:attach-test",
		"P1",
		"attachment-source",
		"target-destroyed",
	)

	result := registerContinuousEffect(state, operation, EffectSpec{
		Kind:      "addKeyword",
		TargetRef: "selected",
		Keyword:   "test",
	})

	if len(result.Board.Attachments.Active) != 0 {
		t.Fatalf("expected no attachment created when target is destroyed, got %d", len(result.Board.Attachments.Active))
	}
}

func TestAttachmentCreatedWhenTargetValid(t *testing.T) {
	state := newContinuousTestState()
	state.Board.Cards = []CardState{
		{
			CardID:         "attachment-source",
			Name:           "Test Attachment",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
		{
			CardID:         "target-valid",
			Name:           "Target Valid",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			Destroyed:      false,
		},
	}

	operation := manualAttachmentOperation(
		"op:attach-test",
		"act:attach-test",
		"P1",
		"attachment-source",
		"target-valid",
	)

	result := registerContinuousEffect(state, operation, EffectSpec{
		Kind:      "addKeyword",
		TargetRef: "selected",
		Keyword:   "test",
	})

	if len(result.Board.Attachments.Active) != 1 {
		t.Fatalf("expected 1 attachment created when target is valid, got %d", len(result.Board.Attachments.Active))
	}

	attachment := result.Board.Attachments.Active[0]
	if attachment.SourceCardID != "attachment-source" {
		t.Errorf("attachment source = %q, want %q", attachment.SourceCardID, "attachment-source")
	}
	if attachment.TargetCardID != "target-valid" {
		t.Errorf("attachment target = %q, want %q", attachment.TargetCardID, "target-valid")
	}
}

func manualAttachmentOperation(
	operationID string,
	actionID string,
	actorID string,
	cardID string,
	targetCardID string,
) Operation {
	return Operation{
		ID:            operationID,
		ActionID:      actionID,
		ActorID:       actorID,
		Kind:          OperationKindCardEffect,
		Status:        OperationStatusBuilt,
		CardID:        cardID,
		TargetCardID:  targetCardID,
		RequiresStack: false,
		Source: &CardOperationSource{
			CardID:            cardID,
			CardName:          "Test Attachment",
			LogicID:           "test.attachment",
			Speed:             "slow",
			TargetKinds:       []string{"character"},
			BasicType:         "附属",
			RequiresStack:     false,
			ExecutionKind:     CardExecutionDSL,
			DurationKind:      "permanent",
			RequiresScript:    false,
			PureDSLExecutable: true,
			Effects: []EffectSpec{
				{
					Kind:      "addKeyword",
					TargetRef: "selected",
					Keyword:   "test",
				},
			},
		},
	}
}
