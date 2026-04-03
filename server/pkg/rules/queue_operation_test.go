package rules

import (
	"testing"
)

func TestExecuteQueueOperation(t *testing.T) {
	t.Run("execute_queue_operation_ignores_payment_for_debug_flow", func(t *testing.T) {
		state := NewGameState(InitialStateConfig{
			ActivePlayerID: "P1",
			PlayerIDs:      []string{"P1", "P2"},
		})
		state.Turn.Resources["P1"] = PlayerResourceState{Current: 0, Max: 10}

		op := Operation{
			ID:       "op-1",
			ActionID: "act-1",
			ActorID:  "P1",
			Kind:     OperationKindCardEffect,
			CardID:   "C1",
			Source: &CardOperationSource{
				CardID:        "C1",
				CardName:      "Test Card",
				Cost:          2,
				ExecutionKind: CardExecutionDSL,
			},
		}

		nextState, _, _, err := executeQueueOperation(state, op)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		pool := nextState.Turn.Resources["P1"]
		if pool.Current != 0 {
			t.Fatalf("expected resources to remain 0 in debug flow, got %d", pool.Current)
		}
	})

	t.Run("execute_queue_operation_enqueues_when_requires_stack", func(t *testing.T) {
		state := NewGameState(InitialStateConfig{
			ActivePlayerID: "P1",
			PlayerIDs:      []string{"P1", "P2"},
		})
		state.Board = BoardState{
			Stack:    make([]Operation, 0),
			Cards:    make([]CardState, 0),
			Resolved: make([]Operation, 0),
		}
		state.Turn.Resources["P1"] = PlayerResourceState{Current: 0, Max: 10}

		op := Operation{
			ID:            "op-1",
			ActionID:      "act-1",
			ActorID:       "P1",
			Kind:          OperationKindCardEffect,
			CardID:        "C1",
			RequiresStack: true,
			Source: &CardOperationSource{
				CardID:        "C1",
				CardName:      "Test Card",
				Cost:          1,
				RequiresStack: true,
				ExecutionKind: CardExecutionDSL,
			},
		}

		nextState, updatedOp, event, err := executeQueueOperation(state, op)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if len(nextState.Board.Stack) != 1 {
			t.Fatalf("expected stack size 1, got %d", len(nextState.Board.Stack))
		}
		if updatedOp.Status != OperationStatusPending {
			t.Fatalf("expected status %s, got %s", OperationStatusPending, updatedOp.Status)
		}
		if event.Kind != EventKindOperationEnqueued {
			t.Fatalf("expected event %s, got %s", EventKindOperationEnqueued, event.Kind)
		}

		pool := nextState.Turn.Resources["P1"]
		if pool.Current != 0 {
			t.Fatalf("expected resources to remain 0 in debug flow, got %d", pool.Current)
		}
	})
}
