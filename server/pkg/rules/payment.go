package rules

// Purpose: Defines the single authoritative payment engine seam for the current rules kernel.

type PaymentEngine interface {
	Initialize(*GameState)
	RefillForTurn(*GameState)
	ResourceView(GameState, string) PlayerResourceState
	PayCost(*GameState, string, int) bool
	OnStepEnd(*GameState)
}

var defaultPaymentEngine PaymentEngine = GamePaymentEngine{}

func CurrentPaymentEngine() PaymentEngine {
	return defaultPaymentEngine
}
