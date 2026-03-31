package contracts

import "slices"

// ParseFixtureLogic translates a shared fixture into the minimal Go-side interpretation needed by contract tests.
func ParseFixtureLogic(fixture Fixture) ParsedCardLogic {
	scriptID := cloneOptionalString(fixture.Input.Logic.ScriptID)
	requiresScript := scriptID != nil && *scriptID != ""
	effectKinds := make([]string, 0, len(fixture.Input.Logic.Effects))
	for _, effect := range fixture.Input.Logic.Effects {
		effectKinds = append(effectKinds, effect.Kind)
	}

	return ParsedCardLogic{
		CardID:            fixture.CardID,
		CardName:          fixture.Card.Name,
		SourcePath:        fixture.Card.SourcePath,
		BasicType:         fixture.Card.BasicType,
		LogicID:           fixture.Input.Logic.ID,
		SchemaVersion:     fixture.Input.Logic.SchemaVersion,
		Speed:             fixture.Input.Logic.Speed,
		TargetKinds:       slices.Clone(fixture.Input.Logic.TargetKinds),
		RequiresStack:     fixture.Input.Logic.RequiresStack,
		DurationKind:      fixture.Input.Logic.DurationKind,
		ScriptID:          scriptID,
		RequiresScript:    requiresScript,
		PureDSLExecutable: !requiresScript,
		Effects:           slices.Clone(fixture.Input.Logic.Effects),
		EffectKinds:       effectKinds,
	}
}

func cloneOptionalString(value *string) *string {
	if value == nil {
		return nil
	}

	cloned := *value
	return &cloned
}
