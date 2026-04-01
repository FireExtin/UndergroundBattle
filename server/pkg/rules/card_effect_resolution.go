package rules

import "fmt"

// Purpose: Keeps card-effect resolution logic outside engine.go so the engine remains orchestration-first.

func resolveCardEffect(state GameState, operation Operation) (GameState, Operation, error) {
	if operation.Source == nil {
		return GameState{}, Operation{}, fmt.Errorf("%s", ReasonCodeRulesFailedCardLogicUnavailable)
	}

	switch operation.Source.ExecutionKind {
	case CardExecutionDSL:
		return resolveDSLCardEffect(state, operation)
	case CardExecutionScript:
		return resolveScriptCardEffect(state, operation)
	default:
		return GameState{}, Operation{}, fmt.Errorf("%s", ReasonCodeRulesFailedCardLogicUnavailable)
	}
}

func resolveDSLCardEffect(state GameState, operation Operation) (GameState, Operation, error) {
	if operation.Source == nil {
		return GameState{}, Operation{}, fmt.Errorf("%s", ReasonCodeRulesFailedCardLogicUnavailable)
	}

	working := cloneGameState(state)
	for _, effect := range operation.Source.Effects {
		working = applyDSLEffect(working, operation, effect)
	}

	resolved := markOperationResolved(operation)
	return finalizeResolvedOperation(working, resolved), resolved, nil
}

func resolveScriptCardEffect(state GameState, operation Operation) (GameState, Operation, error) {
	resolved := markOperationResolved(operation)
	return finalizeResolvedOperation(state, resolved), resolved, nil
}

func finalizeResolvedOperation(state GameState, operation Operation) GameState {
	working := cloneGameState(state)
	appendResolvedOperation(&working, operation)
	return working
}

func markOperationResolved(operation Operation) Operation {
	resolved := cloneOperation(operation)
	resolved.Status = OperationStatusResolved
	return resolved
}
