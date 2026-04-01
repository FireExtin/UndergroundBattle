package rules

import (
	"reflect"
	"testing"
)

func TestLookupCardOperationSourceUsesConfiguredAdapter(t *testing.T) {
	original := defaultCardOperationSourceLookup
	t.Cleanup(func() {
		defaultCardOperationSourceLookup = original
	})

	expected := CardOperationSource{
		CardID:        "ADAPTER-CARD",
		CardName:      "Adapter Card",
		BasicType:     "角色",
		ExecutionKind: CardExecutionDSL,
	}

	called := false
	defaultCardOperationSourceLookup = cardOperationSourceLookupFunc(func(cardID string) (CardOperationSource, bool, error) {
		called = true
		if cardID != "ADAPTER-CARD" {
			t.Fatalf("adapter cardId = %q, want %q", cardID, "ADAPTER-CARD")
		}
		return expected, true, nil
	})

	source, found, err := lookupCardOperationSource("ADAPTER-CARD")
	if err != nil {
		t.Fatalf("lookupCardOperationSource returned error: %v", err)
	}
	if !called {
		t.Fatal("expected configured adapter to be called")
	}
	if !found {
		t.Fatal("expected source to be found")
	}
	if !reflect.DeepEqual(source, expected) {
		t.Fatalf("source = %#v, want %#v", source, expected)
	}
}

func TestSubmitActionInternalUsesInjectedCardSourceLookup(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-injected-card-source-lookup",
		ActivePlayerID: "P1",
		Seed:           57,
	})

	amount := 1
	injectedSource := CardOperationSource{
		CardID:            "INJECTED-CARD",
		CardName:          "Injected Card",
		SourcePath:        "test://injected-card",
		BasicType:         "事务",
		LogicID:           "test.injected-card",
		Speed:             "slow",
		TargetKinds:       []string{},
		RequiresStack:     false,
		ExecutionKind:     CardExecutionDSL,
		DurationKind:      "immediate",
		RequiresScript:    false,
		PureDSLExecutable: true,
		Effects: []EffectSpec{
			{
				Kind:      "drawCards",
				TargetRef: "controller",
				Amount:    &amount,
			},
		},
		EffectKinds: []string{"drawCards"},
	}

	called := false
	result, err := submitActionInternal(state, Action{
		ID:      "act-injected-card-source-lookup",
		ActorID: "P1",
		Kind:    ActionKindQueueOperation,
		CardID:  "INJECTED-CARD",
	}, submitInternalOptions{
		projector:          nil,
		enforceDeterminism: false,
		cardSourceLookup: cardOperationSourceLookupFunc(func(cardID string) (CardOperationSource, bool, error) {
			called = true
			if cardID != "INJECTED-CARD" {
				t.Fatalf("injected lookup cardId = %q, want %q", cardID, "INJECTED-CARD")
			}
			return injectedSource, true, nil
		}),
	})
	if err != nil {
		t.Fatalf("submitActionInternal returned error: %v", err)
	}
	if !called {
		t.Fatal("expected injected card source lookup to be called")
	}
	if result.Operation.Source == nil {
		t.Fatal("expected operation source metadata from injected lookup")
	}
	if result.Operation.Source.CardID != "INJECTED-CARD" {
		t.Fatalf("operation source cardId = %q, want %q", result.Operation.Source.CardID, "INJECTED-CARD")
	}
}
