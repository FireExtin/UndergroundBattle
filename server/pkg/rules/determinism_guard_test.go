package rules

import (
	"errors"
	"testing"
)

func TestSubmitActionInternalDeterminismGuardPassesForStableReplay(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-determinism-guard-stable",
		ActivePlayerID: "P1",
		Seed:           31,
	})

	_, err := submitActionInternal(state, Action{
		ID:      "act-determinism-guard-stable",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	}, submitInternalOptions{
		enforceDeterminism: true,
	})
	if err != nil {
		t.Fatalf("submitActionInternal returned error: %v", err)
	}
}

func TestSubmitActionInternalRejectsDeterminismMismatch(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-determinism-guard-mismatch",
		ActivePlayerID: "P1",
		Seed:           32,
	})

	action := Action{
		ID:      "act-determinism-guard-mismatch",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	}

	_, err := submitActionInternal(state, action, submitInternalOptions{
		enforceDeterminism: true,
		replayForDeterminism: func(state GameState, action Action) (SubmitResult, error) {
			replayed, err := submitActionWithoutProjection(state, action)
			if err != nil {
				return SubmitResult{}, err
			}
			replayed.State.Revision.Number++
			return replayed, nil
		},
	})
	if err == nil {
		t.Fatal("expected determinism guard to reject mismatch")
	}

	var legalityErr *LegalityError
	if !errors.As(err, &legalityErr) {
		t.Fatalf("expected LegalityError, got %T", err)
	}

	if legalityErr.Code != ReasonCodeRulesFailedInvariantViolated {
		t.Fatalf("error code = %q, want %q", legalityErr.Code, ReasonCodeRulesFailedInvariantViolated)
	}

	if legalityErr.Hook != "replay.determinism" {
		t.Fatalf("error hook = %q, want %q", legalityErr.Hook, "replay.determinism")
	}

	if legalityErr.Context["actionId"] != action.ID {
		t.Fatalf("error context actionId = %q, want %q", legalityErr.Context["actionId"], action.ID)
	}
}
