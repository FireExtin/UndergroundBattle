package rules

import "testing"

func TestBuildProductionProhibitionRules(t *testing.T) {
	rules := BuildProductionProhibitionRules()
	if len(rules) != 1 {
		t.Fatalf("production prohibition rule count = %d, want 1", len(rules))
	}

	rule := rules[0]
	if rule.SourceDefinitionID != "XQ22" {
		t.Fatalf("prohibition sourceDefinitionId = %q, want XQ22", rule.SourceDefinitionID)
	}
	if len(rule.TargetCategory.BasicTypes) != 1 || rule.TargetCategory.BasicTypes[0] != "事务" {
		t.Fatalf("prohibition basicTypes = %#v, want [事务]", rule.TargetCategory.BasicTypes)
	}
}

func TestBuildProductionTargetLegalityRules(t *testing.T) {
	rules := BuildProductionTargetLegalityRules()
	if len(rules) != 1 {
		t.Fatalf("production target-legality rule count = %d, want 1", len(rules))
	}

	rule := rules[0]
	if rule.SourceDefinitionID != "XQ31" {
		t.Fatalf("target legality sourceDefinitionId = %q, want XQ31", rule.SourceDefinitionID)
	}
	if len(rule.AffectedTargetCondition.Keywords) != 1 || rule.AffectedTargetCondition.Keywords[0] != "声望" {
		t.Fatalf("target legality keywords = %#v, want [声望]", rule.AffectedTargetCondition.Keywords)
	}
	if rule.AffectedTargetCondition.Side != SideAlly {
		t.Fatalf("target legality side = %q, want %q", rule.AffectedTargetCondition.Side, SideAlly)
	}
}

func TestBuildProductionContinuousEffectTemplates(t *testing.T) {
	templates := BuildProductionContinuousEffectTemplates()
	if len(templates) != 1 {
		t.Fatalf("production continuous effect template count = %d, want 1", len(templates))
	}

	foundXQ31 := false

	for _, template := range templates {
		switch template.SourceDefinitionID {
		case "XQ31":
			foundXQ31 = true
			if template.SourceCondition.Zone != CardZoneTable {
				t.Fatalf("XQ31 continuous effect sourceCondition.Zone = %q, want %q", template.SourceCondition.Zone, CardZoneTable)
			}
			if !template.SourceCondition.Ready {
				t.Fatal("XQ31 continuous effect sourceCondition.Ready = false, want true")
			}
			if !template.SourceCondition.NotDestroyed {
				t.Fatal("XQ31 continuous effect sourceCondition.NotDestroyed = false, want true")
			}
			if template.Layer != LayerNumeric {
				t.Fatalf("XQ31 continuous effect layer = %q, want %q", template.Layer, LayerNumeric)
			}
			if template.EffectKind != "modifyStat" {
				t.Fatalf("XQ31 continuous effect effectKind = %q, want modifyStat", template.EffectKind)
			}
			if template.Stat != "defense" {
				t.Fatalf("XQ31 continuous effect stat = %q, want defense", template.Stat)
			}
			if template.Amount != 1 {
				t.Fatalf("XQ31 continuous effect amount = %d, want 1", template.Amount)
			}
			if len(template.TargetCondition.Kinds) != 1 || template.TargetCondition.Kinds[0] != CardKindCharacter {
				t.Fatalf("XQ31 continuous effect targetCondition.Kinds = %#v, want [character]", template.TargetCondition.Kinds)
			}
			if template.TargetCondition.Side != SideAlly {
				t.Fatalf("XQ31 continuous effect targetCondition.Side = %q, want %q", template.TargetCondition.Side, SideAlly)
			}
			if len(template.TargetCondition.Keywords) != 1 || template.TargetCondition.Keywords[0] != "声望" {
				t.Fatalf("XQ31 continuous effect targetCondition.Keywords = %#v, want [声望]", template.TargetCondition.Keywords)
			}
		case "XQ01":
			t.Fatal("XQ01 should remain deferred until region-scoped silence prerequisites exist")
		}
	}

	if !foundXQ31 {
		t.Fatal("expected XQ31 continuous effect template")
	}
}
