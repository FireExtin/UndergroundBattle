package rules

// Purpose: Defines the replaceable payment engine seam while keeping the current prototype resource model intact.

type PaymentMode string

const (
	PaymentModePrototype PaymentMode = "prototype"
)

type PaymentEngine interface {
	Mode() PaymentMode
	Initialize(*GameState)
	RefillForTurn(*GameState)
	ResourceView(GameState, string) PlayerResourceState
	PayCost(*GameState, string, int) bool
}

type PaymentMetadata struct {
	Mode PaymentMode `json:"mode"`
}

var defaultPaymentEngine PaymentEngine = PrototypePaymentEngine{}

func CurrentPaymentEngine() PaymentEngine {
	return defaultPaymentEngine
}

func CurrentPaymentMode() PaymentMode {
	engine := CurrentPaymentEngine()
	if engine == nil {
		return ""
	}
	return engine.Mode()
}
