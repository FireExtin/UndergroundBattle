package rules

import (
	"testing"
)

func TestCheckQueueOperationActionLegalityRejectsMissingCardID(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "queue-op-legality-missing-card",
		ActivePlayerID: "P1",
		Seed:           58,
	})

	legality := checkQueueOperationActionLegality(state, Action{
		ID:      "act-queue-missing-card",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
	}, nil)

	if legality.OK {
		t.Fatal("expected legality failure for missing action.cardId")
	}
	if legality.ReasonCode != ReasonCodeTargetFailedMissing {
		t.Fatalf("legality reasonCode = %q, want %q", legality.ReasonCode, ReasonCodeTargetFailedMissing)
	}
}

func TestBuildQueueOperationFromActionReturnsSourceMetadata(t *testing.T) {
	operation := Operation{
		ID:       "op:queue-build",
		ActionID: "act-queue-build",
		ActorID:  "P1",
		Status:   OperationStatusBuilt,
	}

	err := buildQueueOperationFromAction(Action{
		ID:      "act-queue-build",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
		CardID:  "INJECTED-CARD",
	}, cardOperationSourceLookupFunc(func(cardID string) (CardOperationSource, bool, error) {
		return CardOperationSource{
			CardID:            cardID,
			CardName:          "Injected Queue Card",
			BasicType:         "事务",
			RequiresStack:     true,
			ExecutionKind:     CardExecutionDSL,
			PureDSLExecutable: true,
		}, true, nil
	}), &operation)
	if err != nil {
		t.Fatalf("buildQueueOperationFromAction returned error: %v", err)
	}

	if operation.Kind != OperationKindCardEffect {
		t.Fatalf("operation kind = %q, want %q", operation.Kind, OperationKindCardEffect)
	}
	if operation.CardID != "INJECTED-CARD" {
		t.Fatalf("operation cardId = %q, want %q", operation.CardID, "INJECTED-CARD")
	}
	if operation.Label != "Injected Queue Card" {
		t.Fatalf("operation label = %q, want %q", operation.Label, "Injected Queue Card")
	}
	if operation.Source == nil || operation.Source.CardID != "INJECTED-CARD" {
		t.Fatalf("operation source = %#v, want injected source metadata", operation.Source)
	}
}
