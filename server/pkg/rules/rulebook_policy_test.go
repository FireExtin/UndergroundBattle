package rules

import "testing"

func TestRulebookPolicy_C01_DrawStepQuickWindowPolicy(t *testing.T) {
	policy := DefaultDrawStepPolicy()

	if !policy.DrawWithoutStack {
		t.Fatal("draw step should resolve draw action outside stack")
	}
	if !policy.DrawTriggersEnterStack {
		t.Fatal("draw-step triggers should enter stack")
	}
	if policy.PostDrawWindow != PriorityWindowAction {
		t.Fatalf("post draw window = %q, want %q", policy.PostDrawWindow, PriorityWindowAction)
	}
}

func TestRulebookPolicy_C02_RecoveryOrder(t *testing.T) {
	order := DefaultRecoveryStepOrder()
	want := []RecoveryStepPhase{
		RecoveryStepPhaseFirstPlayerDiscardToLimit,
		RecoveryStepPhaseSecondPlayerDiscardToLimit,
		RecoveryStepPhaseClearDamageAndEndTurnEffects,
		RecoveryStepPhaseTransferFirstPlayer,
	}

	if len(order) != len(want) {
		t.Fatalf("recovery order length = %d, want %d", len(order), len(want))
	}

	for i := range want {
		if order[i] != want[i] {
			t.Fatalf("recovery order[%d] = %q, want %q", i, order[i], want[i])
		}
	}
}

func TestRulebookPolicy_C03_RegionWinOrder(t *testing.T) {
	order := DefaultRegionWinOrder()
	want := []RegionWinStep{
		RegionWinStepClearRegionUnits,
		RegionWinStepMoveRegionToScore,
		RegionWinStepRefillRegionSlot,
		RegionWinStepEnqueueRegionWinTriggers,
		RegionWinStepOpenFastWindow,
	}

	if len(order) != len(want) {
		t.Fatalf("region-win order length = %d, want %d", len(order), len(want))
	}

	for i := range want {
		if order[i] != want[i] {
			t.Fatalf("region-win order[%d] = %q, want %q", i, order[i], want[i])
		}
	}
}

func TestRulebookPolicy_C04_SecretDeployIsRespondable(t *testing.T) {
	lookup := cardOperationSourceLookupFunc(func(cardID string) (CardOperationSource, bool, error) {
		switch cardID {
		case "SECRET_DEPLOY":
			return CardOperationSource{
				CardID:            cardID,
				CardName:          "秘密派遣",
				BasicType:         "角色",
				Speed:             "slow",
				RequiresStack:     true,
				ExecutionKind:     CardExecutionDSL,
				PureDSLExecutable: true,
			}, true, nil
		case "FAST_RESPONSE":
			return CardOperationSource{
				CardID:            cardID,
				CardName:          "快速响应",
				BasicType:         "事务",
				Speed:             "reaction",
				RequiresStack:     true,
				ExecutionKind:     CardExecutionDSL,
				PureDSLExecutable: true,
			}, true, nil
		default:
			return CardOperationSource{}, false, nil
		}
	})

	state := NewGameState(InitialStateConfig{GameID: "c04-secret-respond", ActivePlayerID: "P1"})

	deployLegality := checkQueueOperationActionLegality(state, Action{
		ID:      "act-c04-deploy",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
		CardID:  "SECRET_DEPLOY",
	}, lookup)
	if !deployLegality.OK {
		t.Fatalf("secret deploy should be legal in action window, got %+v", deployLegality)
	}

	op := Operation{ID: "op:act-c04-deploy", ActionID: "act-c04-deploy", ActorID: "P1", Status: OperationStatusBuilt}
	if err := buildQueueOperationFromAction(Action{ID: "act-c04-deploy", ActorID: "P1", Kind: ActionKindQueueOperation, CardID: "SECRET_DEPLOY"}, lookup, &op); err != nil {
		t.Fatalf("buildQueueOperationFromAction returned error: %v", err)
	}

	nextState, _, _, err := executeOperation(state, op)
	if err != nil {
		t.Fatalf("executeOperation returned error: %v", err)
	}

	if len(nextState.Board.Stack) != 1 {
		t.Fatalf("stack depth = %d, want 1 after secret deploy", len(nextState.Board.Stack))
	}
	if nextState.Turn.Priority.WindowKind != PriorityWindowResponse {
		t.Fatalf("priority window = %q, want %q", nextState.Turn.Priority.WindowKind, PriorityWindowResponse)
	}
	if nextState.Turn.Priority.CurrentPlayerID != "P2" {
		t.Fatalf("priority player = %q, want %q", nextState.Turn.Priority.CurrentPlayerID, "P2")
	}

	responseLegality := checkQueueOperationActionLegality(nextState, Action{
		ID:      "act-c04-response",
		ActorID: "P2",
		Kind:    ActionKindQueueOperation,
		CardID:  "FAST_RESPONSE",
	}, lookup)
	if !responseLegality.OK {
		t.Fatalf("response should be legal with non-empty stack, got %+v", responseLegality)
	}
}

func TestRulebookPolicy_C04_ReactionRequiresStack(t *testing.T) {
	lookup := cardOperationSourceLookupFunc(func(cardID string) (CardOperationSource, bool, error) {
		if cardID != "FAST_RESPONSE" {
			return CardOperationSource{}, false, nil
		}
		return CardOperationSource{
			CardID:            cardID,
			CardName:          "快速响应",
			BasicType:         "事务",
			Speed:             "reaction",
			RequiresStack:     true,
			ExecutionKind:     CardExecutionDSL,
			PureDSLExecutable: true,
		}, true, nil
	})

	state := NewGameState(InitialStateConfig{GameID: "c04-reaction-requires-stack", ActivePlayerID: "P1"})

	legality := checkQueueOperationActionLegality(state, Action{
		ID:      "act-c04-no-stack",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
		CardID:  "FAST_RESPONSE",
	}, lookup)
	if legality.OK {
		t.Fatal("reaction should be illegal when stack is empty")
	}
	if legality.ReasonCode != ReasonCodeLegalityFailedResponseWindowRequired {
		t.Fatalf("reason code = %q, want %q", legality.ReasonCode, ReasonCodeLegalityFailedResponseWindowRequired)
	}
}

func TestRulebookPolicy_C05_ZeroVsZeroIsNotTie(t *testing.T) {
	if got := ResolveContestOutcome(0, 0); got != ContestOutcomeNotOccurred {
		t.Fatalf("ResolveContestOutcome(0,0) = %q, want %q", got, ContestOutcomeNotOccurred)
	}

	if got := ResolveContestOutcome(2, 2); got != ContestOutcomeTie {
		t.Fatalf("ResolveContestOutcome(2,2) = %q, want %q", got, ContestOutcomeTie)
	}

	if got := ResolveContestOutcome(3, 1); got != ContestOutcomeActorWin {
		t.Fatalf("ResolveContestOutcome(3,1) = %q, want %q", got, ContestOutcomeActorWin)
	}

	if got := ResolveContestOutcome(1, 3); got != ContestOutcomeActorLose {
		t.Fatalf("ResolveContestOutcome(1,3) = %q, want %q", got, ContestOutcomeActorLose)
	}
}

func TestRulebookPolicy_C06_FirstPlayerPrivilegeConditions(t *testing.T) {
	applied := ApplyFirstPlayerPrivilege(ContestOutcomeTie, false)
	if !applied.Allowed {
		t.Fatal("first-player privilege should be allowed on unresolved tie")
	}
	if !applied.Consumed {
		t.Fatal("first-player privilege should be consumed when applied")
	}
	if applied.Outcome != ContestOutcomeActorWin {
		t.Fatalf("privilege outcome = %q, want %q", applied.Outcome, ContestOutcomeActorWin)
	}
	if applied.VirtualAdvantage != 1 {
		t.Fatalf("virtual advantage = %d, want 1", applied.VirtualAdvantage)
	}

	deniedUsed := ApplyFirstPlayerPrivilege(ContestOutcomeTie, true)
	if deniedUsed.Allowed {
		t.Fatal("first-player privilege should be denied after already used")
	}

	deniedNotTie := ApplyFirstPlayerPrivilege(ContestOutcomeActorWin, false)
	if deniedNotTie.Allowed {
		t.Fatal("first-player privilege should be denied when contest is not tie")
	}

	deniedNotOccurred := ApplyFirstPlayerPrivilege(ContestOutcomeNotOccurred, false)
	if deniedNotOccurred.Allowed {
		t.Fatal("first-player privilege should be denied when contest did not occur")
	}
}
