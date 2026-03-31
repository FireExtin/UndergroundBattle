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

// XQ31ContinuousEffectTemplate defines the continuous effect for card XQ31 (莫兰大主教).
// When XQ31 is ready on the table, all allied prestige characters gain +1 defense.
var XQ31ContinuousEffectTemplate = ContinuousEffectTemplate{
	SourceDefinitionID: "XQ31",
	SourceCondition: CardCondition{
		Zone:         CardZoneTable,
		Ready:        true,
		NotDestroyed: true,
	},
	TargetCondition: TargetCondition{
		Kinds:    []CardKind{CardKindCharacter},
		Keywords: []string{"声望"},
		Side:     SideAlly,
	},
	Layer:        LayerNumeric,
	EffectKind:   "modifyStat",
	DurationKind: "permanent",
	Stat:         "defense",
	Amount:       1,
	Description:  "XQ31: Allied prestige characters gain +1 defense",
}

func BuildProductionContinuousEffectTemplates() []ContinuousEffectTemplate {
	return []ContinuousEffectTemplate{
		XQ31ContinuousEffectTemplate,
	}
}
