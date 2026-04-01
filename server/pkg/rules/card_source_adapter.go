package rules

import (
	"slices"

	internalcontracts "undergroundbattle/server/internal/contracts"
	contractspkg "undergroundbattle/server/pkg/contracts"
)

// Purpose: Isolates card fixture lookup behind a rules-local adapter boundary so engine orchestration does not own fixture loading details.

type cardOperationSourceLookup interface {
	Lookup(cardID string) (CardOperationSource, bool, error)
}

type cardOperationSourceLookupFunc func(cardID string) (CardOperationSource, bool, error)

func (fn cardOperationSourceLookupFunc) Lookup(cardID string) (CardOperationSource, bool, error) {
	return fn(cardID)
}

type fixtureCatalogCardOperationSourceLookup struct{}

var defaultCardOperationSourceLookup cardOperationSourceLookup = fixtureCatalogCardOperationSourceLookup{}

func resolveCardOperationSourceLookup(lookup cardOperationSourceLookup) cardOperationSourceLookup {
	if lookup != nil {
		return lookup
	}

	return defaultCardOperationSourceLookup
}

func lookupCardOperationSource(cardID string) (CardOperationSource, bool, error) {
	return lookupCardOperationSourceWithLookup(nil, cardID)
}

func lookupCardOperationSourceWithLookup(lookup cardOperationSourceLookup, cardID string) (CardOperationSource, bool, error) {
	return resolveCardOperationSourceLookup(lookup).Lookup(cardID)
}

func (fixtureCatalogCardOperationSourceLookup) Lookup(cardID string) (CardOperationSource, bool, error) {
	catalog, err := internalcontracts.LoadDefaultFixtureCatalog()
	if err != nil {
		return CardOperationSource{}, false, err
	}

	fixture, ok := catalog.Find(cardID)
	if !ok {
		return CardOperationSource{}, false, nil
	}

	return cardOperationSourceFromFixture(fixture), true, nil
}

func cardOperationSourceFromFixture(fixture contractspkg.Fixture) CardOperationSource {
	parsed := contractspkg.ParseFixtureLogic(fixture)
	return CardOperationSource{
		CardID:            parsed.CardID,
		CardName:          parsed.CardName,
		SourcePath:        parsed.SourcePath,
		BasicType:         parsed.BasicType,
		LogicID:           parsed.LogicID,
		Speed:             parsed.Speed,
		TargetKinds:       slices.Clone(parsed.TargetKinds),
		RequiresStack:     parsed.RequiresStack,
		ExecutionKind:     cardExecutionKind(parsed.RequiresScript),
		DurationKind:      parsed.DurationKind,
		ScriptID:          cloneOptionalString(parsed.ScriptID),
		RequiresScript:    parsed.RequiresScript,
		PureDSLExecutable: parsed.PureDSLExecutable,
		Effects:           effectSpecsFromParsed(parsed.Effects),
		EffectKinds:       slices.Clone(parsed.EffectKinds),
	}
}

func cardExecutionKind(requiresScript bool) CardExecutionKind {
	if requiresScript {
		return CardExecutionScript
	}

	return CardExecutionDSL
}

func effectSpecsFromParsed(effects []contractspkg.BasicEffect) []EffectSpec {
	specs := make([]EffectSpec, 0, len(effects))
	for _, effect := range effects {
		specs = append(specs, EffectSpec{
			Kind:      effect.Kind,
			TargetRef: effect.TargetRef,
			Amount:    cloneOptionalInt(effect.Amount),
			Stat:      effect.Stat,
			Keyword:   effect.Keyword,
		})
	}

	return specs
}
