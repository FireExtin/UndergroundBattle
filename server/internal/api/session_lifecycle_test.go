package api

import (
	"testing"
)

func TestSessionLifecycle_Transitions(t *testing.T) {
	session := &SandboxSession{} // lifecycle starts as empty/Reset

	// Valid: Reset -> Setup Step 1
	err := session.Transition(newSetupLifecycle(1))
	if err != nil {
		t.Fatalf("expected Reset -> Setup 1 to be valid, got %v", err)
	}

	// Invalid: Setup 1 -> Setup 3
	err = session.Transition(newSetupLifecycle(3))
	if err == nil {
		t.Fatal("expected Setup 1 -> Setup 3 to be invalid")
	}

	// Valid: Setup 1 -> Setup 2
	err = session.Transition(newSetupLifecycle(2))
	if err != nil {
		t.Fatalf("expected Setup 1 -> Setup 2 to be valid, got %v", err)
	}

	// Valid: Setup 7 -> MatchActive (assuming step 7 is final setup)
	session.lifecycle.SetupStep = 7
	err = session.Transition(newMatchActiveLifecycle())
	if err != nil {
		t.Fatalf("expected Setup 7 -> MatchActive to be valid, got %v", err)
	}

	// Valid: MatchActive -> MatchFinished
	err = session.Transition(newMatchFinishedLifecycle(nil, 100))
	if err != nil {
		t.Fatalf("expected MatchActive -> MatchFinished to be valid, got %v", err)
	}

	// Invalid: MatchFinished -> MatchActive
	err = session.Transition(newMatchActiveLifecycle())
	if err == nil {
		t.Fatal("expected MatchFinished -> MatchActive to be invalid")
	}

	// Valid: Any -> Reset
	err = session.Transition(newResetLifecycle())
	if err != nil {
		t.Fatalf("expected any -> Reset to be valid, got %v", err)
	}
}

func TestSetupStateProjectsLifecycleFromSessionSource(t *testing.T) {
	session := &SandboxSession{
		lifecycle: newMatchActiveLifecycle(),
		setup: SetupState{
			Active:      true,
			Completed:   true,
			CurrentStep: 7,
			Lifecycle:   newResetLifecycle(),
		},
	}

	state := session.SetupState()
	if state.Lifecycle.Kind != SessionLifecycleMatchActive {
		t.Fatalf("setup lifecycle kind = %q, want %q", state.Lifecycle.Kind, SessionLifecycleMatchActive)
	}
}
