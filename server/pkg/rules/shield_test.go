package rules

import "testing"

// Purpose: Verifies Shield V1 behavior for enemy-targeted card/ability actions.

func TestQueueOperationEnemyTargetConsumesShieldAndCancelsEffect(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-shield-queue-enemy",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           31,
	})
	state.Board.Cards = []CardState{
		{
			CardID:         "shield-target-1",
			Name:           "Shielded Target",
			Kind:           CardKindCharacter,
			OwnerID:        "P2",
			ControllerID:   "P2",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			Counters: CardCounters{
				Shield: 2,
			},
		},
	}

	result, err := SubmitAction(state, Action{
		ID:           "act-shield-queue-enemy",
		ActorID:      "P1",
		Kind:         ActionKindQueueOperation,
		CardID:       testCardFastStack,
		TargetCardID: "shield-target-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Event.Kind != EventKindShieldConsumed {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindShieldConsumed)
	}
	if result.Operation.Status != OperationStatusResolved {
		t.Fatalf("operation status = %q, want %q", result.Operation.Status, OperationStatusResolved)
	}
	if len(result.State.Board.Stack) != 0 {
		t.Fatalf("stack depth = %d, want 0 when shield terminates the effect", len(result.State.Board.Stack))
	}
	if len(result.State.Board.Resolved) != 1 {
		t.Fatalf("resolved count = %d, want 1 for shield-terminated card effect", len(result.State.Board.Resolved))
	}

	target := cardStateByID(t, result.State, "shield-target-1")
	if target.Counters.Shield != 1 {
		t.Fatalf("target shield = %d, want 1", target.Counters.Shield)
	}
	if target.Exhausted {
		t.Fatal("target should not be exhausted when shield terminates the effect")
	}
}

func TestQueueOperationFriendlyTargetDoesNotConsumeShield(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-shield-queue-friendly",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           32,
	})
	state.Board.Cards = []CardState{
		{
			CardID:         "friendly-shield-target-1",
			Name:           "Friendly Shielded",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			ControllerID:   "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			Counters: CardCounters{
				Shield: 2,
			},
		},
	}

	queued, err := SubmitAction(state, Action{
		ID:           "act-shield-queue-friendly",
		ActorID:      "P1",
		Kind:         ActionKindQueueOperation,
		CardID:       testCardFastStack,
		TargetCardID: "friendly-shield-target-1",
	})
	if err != nil {
		t.Fatalf("SubmitAction(queue) returned error: %v", err)
	}

	if queued.Event.Kind != EventKindOperationEnqueued {
		t.Fatalf("queue event kind = %q, want %q", queued.Event.Kind, EventKindOperationEnqueued)
	}
	if got := cardStateByID(t, queued.State, "friendly-shield-target-1").Counters.Shield; got != 2 {
		t.Fatalf("target shield after queue = %d, want 2", got)
	}

	resolved, err := SubmitAction(queued.State, Action{
		ID:      "act-shield-queue-friendly-resolve",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	})
	if err != nil {
		t.Fatalf("SubmitAction(resolve) returned error: %v", err)
	}

	target := cardStateByID(t, resolved.State, "friendly-shield-target-1")
	if !target.Exhausted {
		t.Fatal("target should be exhausted after normal effect resolution")
	}
	if target.Counters.Shield != 2 {
		t.Fatalf("target shield after resolve = %d, want 2", target.Counters.Shield)
	}
}

func TestDeclareAttackEnemyTargetConsumesShieldAndCancelsDamage(t *testing.T) {
	state := newRoleActionTestState()
	state.Board.Cards = []CardState{
		testCharacterCard("p1-attacker-shield", "P1", CardNumericStats{Combat: 2, Defense: 2}),
		func() CardState {
			target := testCharacterCard("p2-defender-shield", "P2", CardNumericStats{Combat: 1, Defense: 4})
			target.ControllerID = "P2"
			target.Counters = CardCounters{
				Shield: 1,
			}
			return target
		}(),
	}

	result, err := SubmitAction(state, Action{
		ID:           "act-attack-shield-enemy",
		ActorID:      "P1",
		Kind:         ActionKindDeclareAttack,
		CardID:       "p1-attacker-shield",
		TargetCardID: "p2-defender-shield",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Event.Kind != EventKindShieldConsumed {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindShieldConsumed)
	}

	attacker := cardStateByID(t, result.State, "p1-attacker-shield")
	if !attacker.Exhausted {
		t.Fatal("attacker should still be exhausted when attack gets terminated by shield")
	}

	target := cardStateByID(t, result.State, "p2-defender-shield")
	if target.Counters.Damage != 0 {
		t.Fatalf("target damage = %d, want 0", target.Counters.Damage)
	}
	if target.Counters.Shield != 0 {
		t.Fatalf("target shield = %d, want 0", target.Counters.Shield)
	}
	if target.Destroyed {
		t.Fatal("target should not be destroyed when shield terminates attack effect")
	}
}
