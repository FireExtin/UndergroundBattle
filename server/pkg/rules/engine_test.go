package rules

import (
	"errors"
	"reflect"
	"testing"
)

// Purpose: Verifies priority, stack, replay, revision, and structured legality behavior in the Go rules kernel.

const (
	testCardFastStack       = "BQ005"
	testCardDirect          = "BQ010"
	testCardScriptStack     = "BQ013"
	testCardScriptPermanent = "BQ024"
	testCardXQ03            = "XQ03"
	testCardXQ34            = "XQ34"
	testCardJZ74            = "JZ74"
	testCardWM088           = "WM088"
	testCardWM090           = "WM090"
)

func TestSubmitActionAcceptsLegalQueueOperation(t *testing.T) {
	initial := NewGameState(InitialStateConfig{
		GameID:         "game-legal",
		ActivePlayerID: "P1",
		Seed:           7,
	})

	result, err := SubmitAction(initial, Action{
		ID:      "act-queue-1",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
		CardID:  testCardFastStack,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Operation.Kind != OperationKindCardEffect {
		t.Fatalf("operation kind = %q, want %q", result.Operation.Kind, OperationKindCardEffect)
	}

	if result.Event.Kind != EventKindOperationEnqueued {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindOperationEnqueued)
	}

	if len(result.State.Board.Stack) != 1 {
		t.Fatalf("stack depth = %d, want 1", len(result.State.Board.Stack))
	}

	if result.State.Board.Stack[0].Label != "多重梦境迷宫" {
		t.Fatalf("stack label = %q, want %q", result.State.Board.Stack[0].Label, "多重梦境迷宫")
	}

	if result.State.Board.Stack[0].CardID != testCardFastStack {
		t.Fatalf("stack cardId = %q, want %q", result.State.Board.Stack[0].CardID, testCardFastStack)
	}

	if result.State.Board.Stack[0].Source == nil {
		t.Fatal("expected queued operation to carry fixture source metadata")
	}

	if result.State.Board.Stack[0].Source.LogicID != "cards.bq005.multi-dream-maze" {
		t.Fatalf("logic id = %q, want %q", result.State.Board.Stack[0].Source.LogicID, "cards.bq005.multi-dream-maze")
	}

	if result.State.Board.Stack[0].Source.BasicType != "角色" {
		t.Fatalf("basicType = %q, want %q", result.State.Board.Stack[0].Source.BasicType, "角色")
	}

	if result.State.Board.Stack[0].Source.ExecutionKind != CardExecutionDSL {
		t.Fatalf("execution kind = %q, want %q", result.State.Board.Stack[0].Source.ExecutionKind, CardExecutionDSL)
	}

	if result.State.Turn.Priority.CurrentPlayerID != "P2" {
		t.Fatalf("priority holder = %q, want %q", result.State.Turn.Priority.CurrentPlayerID, "P2")
	}

	if result.State.Turn.Priority.WindowKind != PriorityWindowResponse {
		t.Fatalf("priority window = %q, want %q", result.State.Turn.Priority.WindowKind, PriorityWindowResponse)
	}

	if result.State.Turn.Phase.Step != StepAction {
		t.Fatalf("phase step = %q, want %q", result.State.Turn.Phase.Step, StepAction)
	}

	if result.Accepted.Type != "ActionAccepted" {
		t.Fatalf("accepted type = %q, want %q", result.Accepted.Type, "ActionAccepted")
	}

	if result.Patched.Type != "StatePatched" {
		t.Fatalf("patched type = %q, want %q", result.Patched.Type, "StatePatched")
	}
}

func TestSubmitActionRejectsIllegalAction(t *testing.T) {
	initial := NewGameState(InitialStateConfig{
		GameID:         "game-illegal",
		ActivePlayerID: "P1",
		Seed:           7,
	})

	action := Action{
		ID:      "act-illegal-1",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	}

	_, err := SubmitAction(initial, action)
	if err == nil {
		t.Fatal("SubmitAction unexpectedly accepted an illegal action")
	}

	var legality *LegalityError
	if !errors.As(err, &legality) {
		t.Fatalf("expected LegalityError, got %T", err)
	}

	if legality.Code != ReasonCodeStackFailedEmpty {
		t.Fatalf("legality code = %q, want %q", legality.Code, ReasonCodeStackFailedEmpty)
	}

	rejected := NewActionRejected(action, legality.Result)
	if rejected.Type != "ActionRejected" {
		t.Fatalf("rejected type = %q, want %q", rejected.Type, "ActionRejected")
	}

	if rejected.Legality.ReasonCode != ReasonCodeStackFailedEmpty {
		t.Fatalf("rejected reason code = %q, want %q", rejected.Legality.ReasonCode, ReasonCodeStackFailedEmpty)
	}
}

func TestRevisionMonotonicallyIncrements(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-revision",
		ActivePlayerID: "P1",
		Seed:           11,
	})

	actions := []Action{
		queueCardAction("act-1", "P1", testCardFastStack),
		{
			ID:      "act-2",
			ActorID: "P1",
			Kind:    ActionKindResolveTopStack,
		},
		{
			ID:      "act-3",
			ActorID: "P1",
			Kind:    ActionKindAdvancePhase,
		},
	}

	for index, action := range actions {
		result, err := SubmitAction(state, action)
		if err != nil {
			t.Fatalf("SubmitAction(%q) returned error: %v", action.ID, err)
		}

		wantRevision := index + 1
		if result.Revision.Number != wantRevision {
			t.Fatalf("revision number = %d, want %d", result.Revision.Number, wantRevision)
		}

		state = result.State
	}

	if len(state.History.Revisions) != 3 {
		t.Fatalf("revision log length = %d, want 3", len(state.History.Revisions))
	}
}

func TestCommitRecordsActionLog(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-history",
		ActivePlayerID: "P1",
		Seed:           13,
	})

	action := Action{
		ID:      "act-history-1",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	}

	result, err := SubmitAction(state, action)
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if len(result.State.History.Actions) != 1 {
		t.Fatalf("action log length = %d, want 1", len(result.State.History.Actions))
	}

	if !reflect.DeepEqual(result.State.History.Actions[0], action) {
		t.Fatalf("recorded action = %#v, want %#v", result.State.History.Actions[0], action)
	}
}

func TestReplayActionLogProducesSameState(t *testing.T) {
	initial := NewGameState(InitialStateConfig{
		GameID:         "game-replay",
		ActivePlayerID: "P1",
		Seed:           21,
	})

	actions := []Action{
		queueCardAction("act-replay-1", "P1", testCardFastStack),
		{
			ID:      "act-replay-2",
			ActorID: "P1",
			Kind:    ActionKindResolveTopStack,
		},
		queueCardAction("act-replay-2b", "P1", testCardDirect),
		{
			ID:        "act-replay-3",
			ActorID:   "P1",
			Kind:      ActionKindRollSeededRandom,
			RandomMax: 6,
		},
		{
			ID:      "act-replay-4",
			ActorID: "P1",
			Kind:    ActionKindAdvancePhase,
		},
	}

	finalState, err := ReplayActions(initial, actions)
	if err != nil {
		t.Fatalf("ReplayActions returned error: %v", err)
	}

	replayedState, err := ReplayActions(initial, finalState.History.Actions)
	if err != nil {
		t.Fatalf("ReplayActions(history actions) returned error: %v", err)
	}

	if !reflect.DeepEqual(finalState, replayedState) {
		t.Fatalf("replayed state mismatch\nfinal   = %#v\nreplayed = %#v", finalState, replayedState)
	}
}

func TestSameSeedProducesSameRandomResult(t *testing.T) {
	leftInitial := NewGameState(InitialStateConfig{
		GameID:         "game-rng-left",
		ActivePlayerID: "P1",
		Seed:           99,
	})
	rightInitial := NewGameState(InitialStateConfig{
		GameID:         "game-rng-right",
		ActivePlayerID: "P1",
		Seed:           99,
	})

	action := Action{
		ID:        "act-rng-1",
		ActorID:   "P1",
		Kind:      ActionKindRollSeededRandom,
		RandomMax: 6,
	}

	left, err := SubmitAction(leftInitial, action)
	if err != nil {
		t.Fatalf("left SubmitAction returned error: %v", err)
	}

	right, err := SubmitAction(rightInitial, action)
	if err != nil {
		t.Fatalf("right SubmitAction returned error: %v", err)
	}

	if len(left.State.Board.RandomResults) != 1 || len(right.State.Board.RandomResults) != 1 {
		t.Fatalf("unexpected random result lengths: left=%d right=%d", len(left.State.Board.RandomResults), len(right.State.Board.RandomResults))
	}

	if left.State.Board.RandomResults[0].Value != right.State.Board.RandomResults[0].Value {
		t.Fatalf("random value mismatch: left=%d right=%d", left.State.Board.RandomResults[0].Value, right.State.Board.RandomResults[0].Value)
	}

	if !reflect.DeepEqual(left.State.RNG, right.State.RNG) {
		t.Fatalf("rng state mismatch: left=%#v right=%#v", left.State.RNG, right.State.RNG)
	}
}

func TestPassTransfersPriority(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-pass-transfer",
		ActivePlayerID: "P1",
		Seed:           1,
	})

	result, err := SubmitAction(state, Action{
		ID:      "act-pass-1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Event.Kind != EventKindPriorityPassed {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindPriorityPassed)
	}

	if result.State.Turn.Priority.CurrentPlayerID != "P2" {
		t.Fatalf("priority holder = %q, want %q", result.State.Turn.Priority.CurrentPlayerID, "P2")
	}

	if result.State.Turn.Priority.PassCount != 1 {
		t.Fatalf("pass count = %d, want 1", result.State.Turn.Priority.PassCount)
	}

	if result.State.Turn.Priority.WindowKind != PriorityWindowAction {
		t.Fatalf("priority window = %q, want %q", result.State.Turn.Priority.WindowKind, PriorityWindowAction)
	}
}

func TestDoublePassResolvesTopStackWhenStackNonEmpty(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-double-pass-resolve",
		ActivePlayerID: "P1",
		Seed:           2,
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-stack-1",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
		CardID:  testCardFastStack,
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-pass-2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	})

	result, err := SubmitAction(state, Action{
		ID:      "act-pass-3",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Event.Kind != EventKindOperationResolved {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindOperationResolved)
	}

	if result.Event.ResolvedTargetID != "op:act-stack-1" {
		t.Fatalf("resolved target = %q, want %q", result.Event.ResolvedTargetID, "op:act-stack-1")
	}

	if len(result.State.Board.Stack) != 0 {
		t.Fatalf("stack depth = %d, want 0", len(result.State.Board.Stack))
	}

	if len(result.State.Board.Resolved) != 1 {
		t.Fatalf("resolved count = %d, want 1", len(result.State.Board.Resolved))
	}

	if result.State.Board.Resolved[0].CardID != testCardFastStack {
		t.Fatalf("resolved cardId = %q, want %q", result.State.Board.Resolved[0].CardID, testCardFastStack)
	}

	if result.State.Turn.Priority.WindowKind != PriorityWindowAction {
		t.Fatalf("priority window = %q, want %q", result.State.Turn.Priority.WindowKind, PriorityWindowAction)
	}
}

func TestDoublePassEndsStepWhenStackEmpty(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-double-pass-end",
		ActivePlayerID: "P1",
		Seed:           3,
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-pass-empty-1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})

	result, err := SubmitAction(state, Action{
		ID:      "act-pass-empty-2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Event.Kind != EventKindStepEnded {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindStepEnded)
	}

	if !result.State.Turn.Phase.StepEnded {
		t.Fatal("expected phase step to be marked ended")
	}

	if result.State.Turn.Priority.CurrentPlayerID != "P1" {
		t.Fatalf("priority holder = %q, want %q", result.State.Turn.Priority.CurrentPlayerID, "P1")
	}

	if result.State.Turn.Priority.WindowKind != PriorityWindowClosed {
		t.Fatalf("priority window = %q, want %q", result.State.Turn.Priority.WindowKind, PriorityWindowClosed)
	}

	if result.State.Turn.Phase.Step != StepEnded {
		t.Fatalf("phase step = %q, want %q", result.State.Turn.Phase.Step, StepEnded)
	}
}

func TestIllegalActionReturnsMachineReadableReasonCode(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-reason-code",
		ActivePlayerID: "P1",
		Seed:           4,
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-pass-machine-1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})

	action := Action{
		ID:      "act-bad-priority",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	}

	legality := CheckLegality(state, action)
	if legality.OK {
		t.Fatal("expected legality check to fail")
	}

	if legality.ReasonCode != ReasonCodeLegalityFailedNotYourPriority {
		t.Fatalf("reason code = %q, want %q", legality.ReasonCode, ReasonCodeLegalityFailedNotYourPriority)
	}

	rejected := NewActionRejected(action, legality)
	if rejected.Legality.ReasonCode != ReasonCodeLegalityFailedNotYourPriority {
		t.Fatalf("rejected reason code = %q, want %q", rejected.Legality.ReasonCode, ReasonCodeLegalityFailedNotYourPriority)
	}
}

func TestStackResolvesLastInFirstOut(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-lifo",
		ActivePlayerID: "P1",
		Seed:           5,
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-lifo-1",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
		CardID:  testCardScriptStack,
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-lifo-2",
		ActorID: "P2",
		Kind:    ActionKindQueueOperation,
		CardID:  testCardFastStack,
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-lifo-3",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-lifo-4",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	})

	if len(state.Board.Resolved) != 2 {
		t.Fatalf("resolved count = %d, want 2", len(state.Board.Resolved))
	}

	if state.Board.Resolved[0].CardID != testCardFastStack {
		t.Fatalf("first resolved cardId = %q, want %q", state.Board.Resolved[0].CardID, testCardFastStack)
	}

	if state.Board.Resolved[1].CardID != testCardScriptStack {
		t.Fatalf("second resolved cardId = %q, want %q", state.Board.Resolved[1].CardID, testCardScriptStack)
	}
}

func TestStandardActionFailsWhenOpponentHasPriority(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-priority-legality",
		ActivePlayerID: "P1",
		Seed:           6,
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-pass-priority-1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})

	legality := CheckLegality(state, Action{
		ID:      "act-priority-check",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})

	if legality.OK {
		t.Fatal("expected legality check to fail")
	}

	if legality.ReasonCode != ReasonCodeLegalityFailedNotYourPriority {
		t.Fatalf("reason code = %q, want %q", legality.ReasonCode, ReasonCodeLegalityFailedNotYourPriority)
	}
}

func TestStandardActionFailsWhenStackIsNotEmpty(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-stack-legality",
		ActivePlayerID: "P1",
		Seed:           7,
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-stack-check-1",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
		CardID:  testCardFastStack,
	})

	legality := CheckLegality(state, Action{
		ID:      "act-stack-check-2",
		ActorID: "P2",
		Kind:    ActionKindAdvancePhase,
	})

	if legality.OK {
		t.Fatal("expected legality check to fail")
	}

	if legality.ReasonCode != ReasonCodeLegalityFailedStackNotEmpty {
		t.Fatalf("reason code = %q, want %q", legality.ReasonCode, ReasonCodeLegalityFailedStackNotEmpty)
	}
}

func TestSetFaceDownFailsWhenStackIsNotEmpty(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-set-face-down-stack-legality",
		ActivePlayerID: "P1",
		Seed:           8,
	})
	state.Board.Cards = append(state.Board.Cards, CardState{
		CardID:       "table-face-down-target",
		DefinitionID: "TEST_CARD",
		Name:         "Face Down Target",
		Kind:         CardKindCharacter,
		OwnerID:      "P2",
		Zone:         CardZoneTable,
	})

	state = mustSubmit(t, state, Action{
		ID:      "act-set-face-down-stack-check-1",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
		CardID:  testCardFastStack,
	})

	legality := CheckLegality(state, Action{
		ID:      "act-set-face-down-stack-check-2",
		ActorID: "P2",
		Kind:    ActionKindSetFaceDown,
		CardID:  "table-face-down-target",
	})

	if legality.OK {
		t.Fatal("expected legality check to fail")
	}

	if legality.ReasonCode != ReasonCodeLegalityFailedStackNotEmpty {
		t.Fatalf("reason code = %q, want %q", legality.ReasonCode, ReasonCodeLegalityFailedStackNotEmpty)
	}
}

func TestFastCardCanBePlayedDuringResponseWindow(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-fast-response",
		ActivePlayerID: "P1",
		Seed:           11,
	})

	state = mustSubmit(t, state, queueCardAction("act-fast-response-1", "P1", testCardScriptStack))

	result, err := SubmitAction(state, queueCardAction("act-fast-response-2", "P2", testCardFastStack))
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if len(result.State.Board.Stack) != 2 {
		t.Fatalf("stack depth = %d, want 2", len(result.State.Board.Stack))
	}

	if result.State.Board.Stack[1].CardID != testCardFastStack {
		t.Fatalf("top stack cardId = %q, want %q", result.State.Board.Stack[1].CardID, testCardFastStack)
	}

	if result.State.Turn.Priority.CurrentPlayerID != "P1" {
		t.Fatalf("priority holder = %q, want %q", result.State.Turn.Priority.CurrentPlayerID, "P1")
	}

	if result.State.Turn.Priority.WindowKind != PriorityWindowResponse {
		t.Fatalf("priority window = %q, want %q", result.State.Turn.Priority.WindowKind, PriorityWindowResponse)
	}
}

func TestSlowCardFailsOutsideActionWindow(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-slow-window",
		ActivePlayerID: "P1",
		Seed:           12,
	})

	state = mustSubmit(t, state, queueCardAction("act-slow-window-1", "P1", testCardScriptStack))

	legality := CheckLegality(state, queueCardAction("act-slow-window-2", "P2", testCardDirect))
	if legality.OK {
		t.Fatal("expected legality failure")
	}

	if legality.ReasonCode != ReasonCodeLegalityFailedActionWindowRequired {
		t.Fatalf("reason code = %q, want %q", legality.ReasonCode, ReasonCodeLegalityFailedActionWindowRequired)
	}
}

func TestResolveTopStackKeepsResponseWindowWhenStackStillNonEmpty(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-response-after-resolve",
		ActivePlayerID: "P1",
		Seed:           13,
	})

	state = mustSubmit(t, state, queueCardAction("act-response-after-resolve-1", "P1", testCardScriptStack))
	state = mustSubmit(t, state, queueCardAction("act-response-after-resolve-2", "P2", testCardFastStack))

	result, err := SubmitAction(state, Action{
		ID:      "act-response-after-resolve-3",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if len(result.State.Board.Stack) != 1 {
		t.Fatalf("stack depth = %d, want 1", len(result.State.Board.Stack))
	}

	if result.State.Turn.Priority.CurrentPlayerID != "P1" {
		t.Fatalf("priority holder = %q, want %q", result.State.Turn.Priority.CurrentPlayerID, "P1")
	}

	if result.State.Turn.Priority.WindowKind != PriorityWindowResponse {
		t.Fatalf("priority window = %q, want %q", result.State.Turn.Priority.WindowKind, PriorityWindowResponse)
	}
}

func TestQueueOperationDirectlyResolvesNonStackFixture(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-direct-card",
		ActivePlayerID: "P1",
		Seed:           8,
	})

	result, err := SubmitAction(state, queueCardAction("act-direct-card", "P1", testCardDirect))
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Event.Kind != EventKindOperationResolved {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindOperationResolved)
	}

	if len(result.State.Board.Stack) != 0 {
		t.Fatalf("stack depth = %d, want 0", len(result.State.Board.Stack))
	}

	if len(result.State.Board.Resolved) != 1 {
		t.Fatalf("resolved count = %d, want 1", len(result.State.Board.Resolved))
	}

	resolved := result.State.Board.Resolved[0]
	if resolved.CardID != testCardDirect {
		t.Fatalf("resolved cardId = %q, want %q", resolved.CardID, testCardDirect)
	}

	if resolved.Source == nil {
		t.Fatal("expected direct card operation to carry source metadata")
	}

	if !reflect.DeepEqual(resolved.Source.TargetKinds, []string{"player"}) {
		t.Fatalf("targetKinds = %v, want [player]", resolved.Source.TargetKinds)
	}

	if resolved.Source.PureDSLExecutable != true {
		t.Fatal("expected read-minds fixture to stay pure DSL executable")
	}

	if resolved.Source.BasicType != "事务" {
		t.Fatalf("basicType = %q, want %q", resolved.Source.BasicType, "事务")
	}

	if resolved.Source.ExecutionKind != CardExecutionDSL {
		t.Fatalf("execution kind = %q, want %q", resolved.Source.ExecutionKind, CardExecutionDSL)
	}
}

func TestQueueOperationRejectsMissingCardFixture(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-missing-card-fixture",
		ActivePlayerID: "P1",
		Seed:           9,
	})

	action := queueCardAction("act-missing-card-fixture", "P1", "ZZ999")
	legality := CheckLegality(state, action)
	if legality.OK {
		t.Fatal("expected legality failure")
	}

	if legality.ReasonCode != ReasonCodeRulesFailedCardLogicMissing {
		t.Fatalf("reason code = %q, want %q", legality.ReasonCode, ReasonCodeRulesFailedCardLogicMissing)
	}

	_, err := SubmitAction(state, action)
	if err == nil {
		t.Fatal("SubmitAction unexpectedly accepted an unknown card fixture")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}

	if legalityErr.Code != ReasonCodeRulesFailedCardLogicMissing {
		t.Fatalf("error code = %q, want %q", legalityErr.Code, ReasonCodeRulesFailedCardLogicMissing)
	}
}

func TestQueueOperationMarksScriptEntryPointFromFixture(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-script-entry",
		ActivePlayerID: "P1",
		Seed:           10,
	})

	result, err := SubmitAction(state, queueCardAction("act-script-entry", "P1", testCardScriptPermanent))
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Operation.Source == nil {
		t.Fatal("expected fixture source metadata")
	}

	if !result.Operation.Source.RequiresScript {
		t.Fatal("expected scripted fixture to set requiresScript")
	}

	if result.Operation.Source.PureDSLExecutable {
		t.Fatal("scripted fixture must not be treated as pure DSL executable")
	}

	if result.Operation.Source.ScriptID == nil || *result.Operation.Source.ScriptID != "scripts.bq024.spinal-bio-armor" {
		t.Fatalf("scriptId = %v, want scripts.bq024.spinal-bio-armor", result.Operation.Source.ScriptID)
	}

	if result.Operation.Source.ExecutionKind != CardExecutionScript {
		t.Fatalf("execution kind = %q, want %q", result.Operation.Source.ExecutionKind, CardExecutionScript)
	}
}

func TestStackResolutionPreservesDSLAndScriptEntryKinds(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-entry-kinds",
		ActivePlayerID: "P1",
		Seed:           14,
	})

	state = mustSubmit(t, state, queueCardAction("act-entry-kinds-1", "P1", testCardScriptStack))
	state = mustSubmit(t, state, queueCardAction("act-entry-kinds-2", "P2", testCardFastStack))
	state = mustSubmit(t, state, Action{
		ID:      "act-entry-kinds-3",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-entry-kinds-4",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	})

	if len(state.Board.Resolved) != 2 {
		t.Fatalf("resolved count = %d, want 2", len(state.Board.Resolved))
	}

	if state.Board.Resolved[0].Source == nil || state.Board.Resolved[0].Source.ExecutionKind != CardExecutionDSL {
		t.Fatalf("first resolved execution kind = %#v, want %q", state.Board.Resolved[0].Source, CardExecutionDSL)
	}

	if state.Board.Resolved[1].Source == nil || state.Board.Resolved[1].Source.ExecutionKind != CardExecutionScript {
		t.Fatalf("second resolved execution kind = %#v, want %q", state.Board.Resolved[1].Source, CardExecutionScript)
	}
}

func TestDirectDSLCardAppliesInspectAndDrawEffects(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-dsl-direct-effects",
		ActivePlayerID: "P1",
		Seed:           15,
	})
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "p2-hand-1",
			Name:           "Hidden Intel",
			OwnerID:        "P2",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
		CardState{
			CardID:         "p2-hand-2",
			Name:           "Second Secret",
			OwnerID:        "P2",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
	)

	result, err := SubmitAction(state, Action{
		ID:             "act-dsl-direct-effects",
		ActorID:        "P1",
		Kind:           ActionKindQueueOperation,
		CardID:         testCardDirect,
		TargetPlayerID: "P2",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if len(result.State.Board.Resolved) != 1 {
		t.Fatalf("resolved count = %d, want 1", len(result.State.Board.Resolved))
	}

	drawnCards := cardsOwnedInZone(result.State, "P1", CardZoneHand)
	if len(drawnCards) != 1 {
		t.Fatalf("drawn hand count = %d, want 1", len(drawnCards))
	}

	target := cardStateByID(t, result.State, "p2-hand-1")
	if !containsString(target.InspectedBy, "P1") {
		t.Fatalf("inspectedBy = %v, want P1 included", target.InspectedBy)
	}

	playerOneView, ok := result.Views.Players["P1"]
	if !ok {
		t.Fatal("expected P1 view to be generated")
	}

	visibleCount := 0
	for _, card := range playerOneView.Board.Cards {
		if card.OwnerID == "P2" && card.Zone == CardZoneHand && card.Visibility == "visible" {
			visibleCount++
		}
	}
	if visibleCount != 2 {
		t.Fatalf("visible inspected hand cards = %d, want 2", visibleCount)
	}
}

func TestStackedDSLCardExhaustsSelectedTargetOnResolution(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-dsl-exhaust",
		ActivePlayerID: "P1",
		Seed:           16,
	})
	state.Board.Cards = append(state.Board.Cards, CardState{
		CardID:         "table-char-1",
		Name:           "Frontline Operative",
		OwnerID:        "P2",
		Zone:           CardZoneTable,
		VisibleToOwner: true,
		Revealed:       true,
	})

	queued := mustSubmit(t, state, Action{
		ID:           "act-dsl-exhaust-queue",
		ActorID:      "P1",
		Kind:         ActionKindQueueOperation,
		CardID:       testCardFastStack,
		TargetCardID: "table-char-1",
	})

	if cardStateByID(t, queued, "table-char-1").Exhausted {
		t.Fatal("target should not be exhausted before stack resolution")
	}

	result, err := SubmitAction(queued, Action{
		ID:      "act-dsl-exhaust-resolve",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	target := cardStateByID(t, result.State, "table-char-1")
	if !target.Exhausted {
		t.Fatal("expected target to become exhausted after DSL stack resolution")
	}

	playerOneView := result.Views.Players["P1"]
	projected := cardViewByID(t, playerOneView.Board.Cards, "table-char-1")
	if !projected.Exhausted {
		t.Fatal("expected visible exhausted state to reach player projection")
	}
}

func TestXQ03ExhaustsSelectedTargetOnResolution(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-xq03-exhaust",
		ActivePlayerID: "P1",
		Seed:           17,
	})
	state.Board.Cards = append(state.Board.Cards, CardState{
		CardID:         "xq03-target-1",
		Name:           "Pinned Target",
		OwnerID:        "P2",
		Zone:           CardZoneTable,
		VisibleToOwner: true,
		Revealed:       true,
	})

	queued := mustSubmit(t, state, Action{
		ID:           "act-xq03-queue",
		ActorID:      "P1",
		Kind:         ActionKindQueueOperation,
		CardID:       testCardXQ03,
		TargetCardID: "xq03-target-1",
	})

	if cardStateByID(t, queued, "xq03-target-1").Exhausted {
		t.Fatal("target should stay ready until XQ03 resolves")
	}

	resolved := mustSubmit(t, queued, Action{
		ID:      "act-xq03-resolve",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	})

	if !cardStateByID(t, resolved, "xq03-target-1").Exhausted {
		t.Fatal("expected XQ03 to exhaust the selected target")
	}
}

func TestFastStackDrawFixturesDrawCardsOnResolution(t *testing.T) {
	testCases := []struct {
		name   string
		cardID string
	}{
		{name: "XQ34", cardID: testCardXQ34},
		{name: "JZ74", cardID: testCardJZ74},
	}

	for _, testCase := range testCases {
		testCase := testCase
		t.Run(testCase.name, func(t *testing.T) {
			state := NewGameState(InitialStateConfig{
				GameID:         "game-" + testCase.cardID,
				ActivePlayerID: "P1",
				Seed:           18,
			})

			queued := mustSubmit(t, state, Action{
				ID:      "act-" + testCase.cardID + "-queue",
				ActorID: "P1",
				Kind:    ActionKindQueueOperation,
				CardID:  testCase.cardID,
			})

			if len(cardsOwnedInZone(queued, "P1", CardZoneHand)) != 0 {
				t.Fatal("draw should not happen before stack resolution")
			}

			resolved := mustSubmit(t, queued, Action{
				ID:      "act-" + testCase.cardID + "-resolve",
				ActorID: "P1",
				Kind:    ActionKindResolveTopStack,
			})

			if len(cardsOwnedInZone(resolved, "P1", CardZoneHand)) != 1 {
				t.Fatalf("drawn hand count = %d, want 1", len(cardsOwnedInZone(resolved, "P1", CardZoneHand)))
			}

			if len(resolved.Board.Resolved) != 1 || resolved.Board.Resolved[0].CardID != testCase.cardID {
				t.Fatalf("resolved cards = %#v, want one resolved %q", resolved.Board.Resolved, testCase.cardID)
			}
		})
	}
}

func TestWM088DrawsTwoCardsOnDirectResolution(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-wm088-draw",
		ActivePlayerID: "P1",
		Seed:           19,
	})

	result, err := SubmitAction(state, Action{
		ID:      "act-wm088",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
		CardID:  testCardWM088,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if len(result.State.Board.Resolved) != 1 {
		t.Fatalf("resolved count = %d, want 1", len(result.State.Board.Resolved))
	}

	if len(cardsOwnedInZone(result.State, "P1", CardZoneHand)) != 2 {
		t.Fatalf("drawn hand count = %d, want 2", len(cardsOwnedInZone(result.State, "P1", CardZoneHand)))
	}
}

func TestWM090InspectsTargetPlayerHandOnDirectResolution(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-wm090-inspect",
		ActivePlayerID: "P1",
		Seed:           20,
	})
	state.Board.Cards = append(state.Board.Cards,
		CardState{
			CardID:         "wm090-p2-hand-1",
			Name:           "First Secret",
			OwnerID:        "P2",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
		CardState{
			CardID:         "wm090-p2-hand-2",
			Name:           "Second Secret",
			OwnerID:        "P2",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
		},
	)

	result, err := SubmitAction(state, Action{
		ID:             "act-wm090",
		ActorID:        "P1",
		Kind:           ActionKindQueueOperation,
		CardID:         testCardWM090,
		TargetPlayerID: "P2",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	target := cardStateByID(t, result.State, "wm090-p2-hand-1")
	if !containsString(target.InspectedBy, "P1") {
		t.Fatalf("inspectedBy = %v, want P1 included", target.InspectedBy)
	}

	playerOneView := result.Views.Players["P1"]
	visibleCount := 0
	for _, card := range playerOneView.Board.Cards {
		if card.OwnerID == "P2" && card.Zone == CardZoneHand && card.Visibility == "visible" {
			visibleCount++
		}
	}

	if visibleCount != 2 {
		t.Fatalf("visible inspected hand cards = %d, want 2", visibleCount)
	}
}

func TestXQ22PreventsQueueOperationForEventCardsWhileReady(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-xq22-event-lock",
		ActivePlayerID: "P1",
		Seed:           21,
	})
	state.Board.Cards = append(state.Board.Cards, xq22TableCard("P1"))

	legality := CheckLegality(state, Action{
		ID:      "act-xq22-blocks-event",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
		CardID:  testCardDirect,
	})

	if legality.OK {
		t.Fatal("expected XQ22 to prohibit queueing event cards while ready")
	}

	if legality.ReasonCode != ReasonCodeLegalityFailedActionProhibited {
		t.Fatalf("reason code = %q, want %q", legality.ReasonCode, ReasonCodeLegalityFailedActionProhibited)
	}
}

func TestXQ22StillPreventsEventCardsWhenDisplayNameChanges(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-xq22-renamed",
		ActivePlayerID: "P1",
		Seed:           26,
	})
	card := xq22TableCard("P1")
	card.Name = "改名后的州议员"
	state.Board.Cards = append(state.Board.Cards, card)

	legality := CheckLegality(state, queueCardAction("act-xq22-renamed", "P1", testCardDirect))
	if legality.OK {
		t.Fatal("expected XQ22 definition to prohibit event cards even after display name changes")
	}

	if legality.ReasonCode != ReasonCodeLegalityFailedActionProhibited {
		t.Fatalf("reason code = %q, want %q", legality.ReasonCode, ReasonCodeLegalityFailedActionProhibited)
	}
}

func TestXQ22DoesNotPreventEventCardsForNameCollisionOnly(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-xq22-name-collision",
		ActivePlayerID: "P1",
		Seed:           27,
	})
	card := xq22TableCard("P1")
	card.DefinitionID = "NOT_XQ22"
	state.Board.Cards = append(state.Board.Cards, card)

	legality := CheckLegality(state, queueCardAction("act-xq22-name-collision", "P1", testCardDirect))
	if !legality.OK {
		t.Fatalf("expected name collision without XQ22 definition to stay legal, got %+v", legality)
	}
}

func TestXQ22PreventsBothPlayersFromQueueingEventCardsWhileReady(t *testing.T) {
	for _, actorID := range []string{"P1", "P2"} {
		t.Run(actorID, func(t *testing.T) {
			state := NewGameState(InitialStateConfig{
				GameID:         "game-xq22-both-players-" + actorID,
				ActivePlayerID: actorID,
				Seed:           22,
			})
			state.Board.Cards = append(state.Board.Cards, xq22TableCard("P1"))

			legality := CheckLegality(state, queueCardAction("act-xq22-both-"+actorID, actorID, testCardDirect))
			if legality.OK {
				t.Fatalf("expected XQ22 to prohibit %s from queueing event cards while ready", actorID)
			}

			if legality.ReasonCode != ReasonCodeLegalityFailedActionProhibited {
				t.Fatalf("reason code = %q, want %q", legality.ReasonCode, ReasonCodeLegalityFailedActionProhibited)
			}
		})
	}
}

func TestXQ22AllowsNonEventCardsWhileReady(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-xq22-non-event",
		ActivePlayerID: "P1",
		Seed:           23,
	})
	state.Board.Cards = append(state.Board.Cards, xq22TableCard("P1"))

	legality := CheckLegality(state, queueCardAction("act-xq22-non-event", "P1", testCardScriptPermanent))
	if !legality.OK {
		t.Fatalf("expected non-event card to remain legal, got %+v", legality)
	}
}

func TestXQ22AllowsEventCardsWhenInactive(t *testing.T) {
	cases := []struct {
		name string
		card CardState
	}{
		{
			name: "exhausted",
			card: func() CardState {
				card := xq22TableCard("P1")
				card.Exhausted = true
				return card
			}(),
		},
		{
			name: "destroyed",
			card: func() CardState {
				card := xq22TableCard("P1")
				card.Destroyed = true
				return card
			}(),
		},
		{
			name: "discarded",
			card: func() CardState {
				card := xq22TableCard("P1")
				card.Zone = CardZoneDiscard
				return card
			}(),
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			state := NewGameState(InitialStateConfig{
				GameID:         "game-xq22-inactive-" + tc.name,
				ActivePlayerID: "P1",
				Seed:           24,
			})
			state.Board.Cards = append(state.Board.Cards, tc.card)

			legality := CheckLegality(state, queueCardAction("act-xq22-inactive-"+tc.name, "P1", testCardDirect))
			if !legality.OK {
				t.Fatalf("expected inactive XQ22 not to prohibit event cards, got %+v", legality)
			}
		})
	}
}

func TestSubmitActionReturnsLegalityErrorWhenXQ22BlocksEventCards(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-xq22-submit-action",
		ActivePlayerID: "P1",
		Seed:           25,
	})
	state.Board.Cards = append(state.Board.Cards, xq22TableCard("P1"))

	_, err := SubmitAction(state, queueCardAction("act-xq22-submit-action", "P1", testCardDirect))
	if err == nil {
		t.Fatal("expected SubmitAction to reject event card while XQ22 is ready")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}

	if legalityErr.Code != ReasonCodeLegalityFailedActionProhibited {
		t.Fatalf("error code = %q, want %q", legalityErr.Code, ReasonCodeLegalityFailedActionProhibited)
	}
}

func TestSetMarkerActionUpdatesMarkerRegistry(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-set-marker",
		ActivePlayerID: "P1",
		Seed:           28,
	})

	result, err := SubmitAction(state, Action{
		ID:             "act-set-marker-1",
		ActorID:        "P1",
		Kind:           ActionKindSetMarker,
		TargetPlayerID: "P1",
		MarkerType:     "secret_society",
		MarkerAmount:   2,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.Operation.Kind != OperationKindSetMarker {
		t.Fatalf("operation kind = %q, want %q", result.Operation.Kind, OperationKindSetMarker)
	}

	if result.Event.Kind != EventKindMarkerSet {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindMarkerSet)
	}

	if got := result.State.Board.Markers.GetMarker("P1", "secret_society"); got != 2 {
		t.Fatalf("marker amount = %d, want 2", got)
	}
}

func TestRemoveMarkerActionRejectsOverRemoval(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-remove-marker",
		ActivePlayerID: "P1",
		Seed:           29,
	})

	state = mustSubmit(t, state, Action{
		ID:             "act-set-marker-before-remove",
		ActorID:        "P1",
		Kind:           ActionKindSetMarker,
		TargetPlayerID: "P1",
		MarkerType:     "secret_society",
		MarkerAmount:   2,
	})

	_, err := SubmitAction(state, Action{
		ID:             "act-remove-marker-1",
		ActorID:        "P1",
		Kind:           ActionKindRemoveMarker,
		TargetPlayerID: "P1",
		MarkerType:     "secret_society",
		MarkerAmount:   5,
	})
	if err == nil {
		t.Fatal("expected SubmitAction to reject marker over-removal")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}
	if legalityErr.Code != ReasonCodeTargetFailedMissing {
		t.Fatalf("error code = %q, want %q", legalityErr.Code, ReasonCodeTargetFailedMissing)
	}
}

func TestSetMarkerActionRejectsMissingMarkerType(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-set-marker-missing-type",
		ActivePlayerID: "P1",
		Seed:           30,
	})

	_, err := SubmitAction(state, Action{
		ID:             "act-set-marker-missing-type",
		ActorID:        "P1",
		Kind:           ActionKindSetMarker,
		TargetPlayerID: "P1",
		MarkerAmount:   1,
	})
	if err == nil {
		t.Fatal("expected SubmitAction to reject missing markerType")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}

	if legalityErr.Code != ReasonCodeTargetFailedMissing {
		t.Fatalf("error code = %q, want %q", legalityErr.Code, ReasonCodeTargetFailedMissing)
	}
}

func mustSubmit(t *testing.T, state GameState, action Action) GameState {
	t.Helper()

	result, err := SubmitAction(state, action)
	if err != nil {
		t.Fatalf("SubmitAction(%q) returned error: %v", action.ID, err)
	}

	return result.State
}

func queueCardAction(actionID string, actorID string, cardID string) Action {
	return Action{
		ID:      actionID,
		ActorID: actorID,
		Kind:    ActionKindQueueOperation,
		CardID:  cardID,
	}
}

func xq22TableCard(ownerID string) CardState {
	return CardState{
		CardID:         "xq22-table-1",
		DefinitionID:   "XQ22",
		Name:           "州议员贝伦·希恩斯",
		Kind:           CardKindCharacter,
		OwnerID:        ownerID,
		Zone:           CardZoneTable,
		VisibleToOwner: true,
		Revealed:       true,
	}
}

func cardsOwnedInZone(state GameState, ownerID string, zone CardZone) []CardState {
	var cards []CardState
	for _, card := range state.Board.Cards {
		if card.OwnerID == ownerID && card.Zone == zone {
			cards = append(cards, card)
		}
	}
	return cards
}

func cardStateByID(t *testing.T, state GameState, cardID string) CardState {
	t.Helper()

	for _, card := range state.Board.Cards {
		if card.CardID == cardID {
			return card
		}
	}

	t.Fatalf("card %q not found", cardID)
	return CardState{}
}

func cardViewByID(t *testing.T, cards []CardView, cardID string) CardView {
	t.Helper()

	for _, card := range cards {
		if card.CardID == cardID {
			return card
		}
	}

	t.Fatalf("projected card %q not found", cardID)
	return CardView{}
}
