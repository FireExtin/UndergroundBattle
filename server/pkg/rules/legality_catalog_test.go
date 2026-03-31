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
