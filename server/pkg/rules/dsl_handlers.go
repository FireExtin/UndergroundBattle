package rules

// Purpose: Separates DSL declaration from execution by routing each effect kind through a Go-side handler registry.

type dslEffectHandler func(GameState, Operation, EffectSpec) GameState

var dslEffectHandlers = map[string]dslEffectHandler{
	"drawCards":      applyDrawCardsEffect,
	"inspectHand":    applyInspectHandEffect,
	"exhaust":        applyExhaustEffect,
	"addKeyword":     applyAddKeywordEffect,
	"modifyStat":     applyModifyStatEffect,
	"placeInfluence": applyPlaceInfluenceEffect,
	"dealDamage":     applyDealDamageEffect,
	"discardCard":    applyDiscardCardEffect,
}

func dslEffectHandlerFor(kind string) (dslEffectHandler, bool) {
	handler, ok := dslEffectHandlers[kind]
	return handler, ok
}
