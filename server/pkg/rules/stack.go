package rules

import "fmt"

// Purpose: Defines the minimal stack engine used by the authoritative rules pipeline.

// StackEngine manages stack push/pop behavior without owning mutable state.
type StackEngine struct{}

func (engine StackEngine) Push(state GameState, operation Operation) (GameState, Operation) {
	working := cloneGameState(state)
	pending := operation
	pending.Status = OperationStatusPending
	working.Board.Stack = append(working.Board.Stack, pending)
	return working, pending
}

func (engine StackEngine) PopTop(state GameState) (GameState, Operation, error) {
	if len(state.Board.Stack) == 0 {
		return GameState{}, Operation{}, fmt.Errorf("%s", ReasonCodeStackFailedEmpty)
	}

	working := cloneGameState(state)
	pending := working.Board.Stack[len(working.Board.Stack)-1]
	working.Board.Stack = working.Board.Stack[:len(working.Board.Stack)-1]
	return working, pending, nil
}
