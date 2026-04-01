package rules

import "testing"

func TestEvaluateActionAbilityKindProhibition_BlocksMatchingActionKind(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "permission-ability-kind-block",
		ActivePlayerID: "P1",
	})
	state.Board.Cards = []CardState{
		{
			CardID:       "xq01-source-1",
			DefinitionID: "XQ01",
			Name:         "Region Silence Source",
			ControllerID: "P1",
			Zone:         CardZoneTable,
			Revealed:     true,
		},
	}

	checker := NewScopedProhibitionChecker([]ProhibitionRule{
		{
			SourceDefinitionID: "XQ01",
			SourceCondition: CardCondition{
				Zone:         CardZoneTable,
				Ready:        true,
				NotDestroyed: true,
			},
			Scope: ProhibitionScope{
				Kind: ProhibitionScopeOpponentsOnly,
			},
			TargetCategory: TargetCategory{
				ActionKinds: []ActionKind{ActionKindDeclareAttack},
				Condition: &TargetCondition{
					AbilityKinds: []string{"action"},
				},
			},
		},
	})

	targetCard := CardState{
		CardID: "target-1",
		Kind:   CardKindCharacter,
	}

	legality := evaluateActionAbilityKindProhibition(
		state,
		"P2",
		ActionKindDeclareAttack,
		targetCard,
		checker,
	)
	if legality.OK {
		t.Fatal("expected declare_attack to be blocked by action ability-kind prohibition")
	}
	if legality.ReasonCode != ReasonCodeLegalityFailedActionProhibited {
		t.Fatalf("reason code = %q, want %q", legality.ReasonCode, ReasonCodeLegalityFailedActionProhibited)
	}
}

func TestEvaluateActionAbilityKindProhibition_IgnoresMismatchedAbilityKind(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "permission-ability-kind-mismatch",
		ActivePlayerID: "P1",
	})
	state.Board.Cards = []CardState{
		{
			CardID:       "xq01-source-1",
			DefinitionID: "XQ01",
			Name:         "Region Silence Source",
			ControllerID: "P1",
			Zone:         CardZoneTable,
			Revealed:     true,
		},
	}

	checker := NewScopedProhibitionChecker([]ProhibitionRule{
		{
			SourceDefinitionID: "XQ01",
			SourceCondition: CardCondition{
				Zone:         CardZoneTable,
				Ready:        true,
				NotDestroyed: true,
			},
			Scope: ProhibitionScope{
				Kind: ProhibitionScopeOpponentsOnly,
			},
			TargetCategory: TargetCategory{
				ActionKinds: []ActionKind{ActionKindDeclareAttack},
				Condition: &TargetCondition{
					AbilityKinds: []string{"trigger"},
				},
			},
		},
	})

	targetCard := CardState{
		CardID: "target-1",
		Kind:   CardKindCharacter,
	}

	legality := evaluateActionAbilityKindProhibition(
		state,
		"P2",
		ActionKindDeclareAttack,
		targetCard,
		checker,
	)
	if !legality.OK {
		t.Fatalf("expected no prohibition when abilityKinds mismatch, got %+v", legality)
	}
}
