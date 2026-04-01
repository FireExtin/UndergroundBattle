package rules

import (
	"reflect"
	"testing"
)

func TestDSLEffectHandlerRegistryIncludesSupportedKinds(t *testing.T) {
	kinds := []string{
		"drawCards",
		"inspectHand",
		"exhaust",
		"addKeyword",
		"modifyStat",
		"placeInfluence",
		"dealDamage",
		"discardCard",
	}

	for _, kind := range kinds {
		if _, ok := dslEffectHandlerFor(kind); !ok {
			t.Fatalf("expected handler registry to include kind %q", kind)
		}
	}
}

func TestApplyDSLEffectUnknownKindIsNoop(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "dsl-handler-unknown-kind",
		ActivePlayerID: "P1",
		Seed:           41,
	})

	result := applyDSLEffect(state, Operation{
		ID:      "op-dsl-handler-unknown-kind",
		ActorID: "P1",
	}, EffectSpec{
		Kind: "unknown-effect-kind",
	})

	if !reflect.DeepEqual(state, result) {
		t.Fatalf("unknown DSL effect should be noop\nbefore = %#v\nafter  = %#v", state, result)
	}
}
