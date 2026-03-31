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

// XQ01SilenceAttackTemplate defines the continuous effect for card XQ01 (联会禁音使).
// When XQ01 is ready on the table, all characters cannot attack.
var XQ01SilenceAttackTemplate = ContinuousEffectTemplate{
	SourceDefinitionID: "XQ01",
	SourceCondition: CardCondition{
		Zone:         CardZoneTable,
		Ready:        true,
		NotDestroyed: true,
	},
	TargetCondition: TargetCondition{},
	Layer:           LayerProhibition,
	EffectKind:      "prohibitPermission",
	DurationKind:    "permanent",
	Permission:      "attack",
	Description:     "XQ01: All characters cannot attack",
}

// XQ01SilenceInvestigateTemplate defines the continuous effect for card XQ01 (联会禁音使).
// When XQ01 is ready on the table, all characters cannot investigate.
var XQ01SilenceInvestigateTemplate = ContinuousEffectTemplate{
	SourceDefinitionID: "XQ01",
	SourceCondition: CardCondition{
		Zone:         CardZoneTable,
		Ready:        true,
		NotDestroyed: true,
	},
	TargetCondition: TargetCondition{},
	Layer:           LayerProhibition,
	EffectKind:      "prohibitPermission",
	DurationKind:    "permanent",
	Permission:      "investigate",
	Description:     "XQ01: All characters cannot investigate",
}

func BuildProductionContinuousEffectTemplates() []ContinuousEffectTemplate {
	return []ContinuousEffectTemplate{
		XQ31ContinuousEffectTemplate,
		XQ01SilenceAttackTemplate,
		XQ01SilenceInvestigateTemplate,
	}
}
