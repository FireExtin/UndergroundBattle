package rules

func BuildProductionProhibitionRules() []ProhibitionRule {
	return []ProhibitionRule{
		XQ22ProhibitionRule,
	}
}

func BuildProductionTargetLegalityRules() []TargetLegalityRule {
	return []TargetLegalityRule{
		XQ31TargetLegalityRule,
	}
}
