package rules

import "testing"

func TestCardMatchesDefinitionAndCondition(t *testing.T) {
	card := CardState{
		CardID:       "xq22-1",
		DefinitionID: "XQ22",
		Zone:         CardZoneTable,
		Exhausted:    false,
		Destroyed:    false,
		Revealed:     true,
	}

	if !cardMatchesDefinitionAndCondition(card, "XQ22", CardCondition{
		Zone:         CardZoneTable,
		Ready:        true,
		NotDestroyed: true,
		Revealed:     true,
	}) {
		t.Fatal("expected ready revealed XQ22 on table to match")
	}

	if cardMatchesDefinitionAndCondition(card, "XQ31", CardCondition{}) {
		t.Fatal("expected mismatched definitionId to fail")
	}
}

func TestScopeAppliesToActor(t *testing.T) {
	source := CardState{ControllerID: "P1"}

	if !scopeAppliesToActor(source, "P2", ProhibitionScope{Kind: ProhibitionScopeOpponentsOnly}) {
		t.Fatal("expected opponents-only scope to apply to P2")
	}
	if scopeAppliesToActor(source, "P1", ProhibitionScope{Kind: ProhibitionScopeOpponentsOnly}) {
		t.Fatal("expected opponents-only scope not to apply to controller")
	}
	if !scopeAppliesToActor(source, "P1", ProhibitionScope{Kind: ProhibitionScopeControllerOnly}) {
		t.Fatal("expected controller-only scope to apply to P1")
	}
}
