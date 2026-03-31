package api

import (
	"reflect"
	"testing"

	"undergroundbattle/server/pkg/rules"
)

// Purpose: Verifies the live HTTP sandbox session reuses the canonical M0 baseline state from the rules package.

func TestNewSandboxSessionUsesCanonicalM0State(t *testing.T) {
	session := NewSandboxSession()
	want := rules.NewM0SandboxState()

	if !reflect.DeepEqual(session.state, want) {
		t.Fatalf("session state mismatch\nsession = %#v\nwant = %#v", session.state, want)
	}
}

func TestSandboxSessionResetRestoresCanonicalM0State(t *testing.T) {
	session := NewSandboxSession()
	_, err := session.SubmitAction(rules.Action{
		ID:      "act-session-reset-1",
		ActorID: "P1",
		Kind:    rules.ActionKindRevealCard,
		CardID:  "P1-HAND-SECRET",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	messages, err := session.Reset()
	if err != nil {
		t.Fatalf("Reset returned error: %v", err)
	}

	want := rules.NewM0SandboxState()
	if !reflect.DeepEqual(session.state, want) {
		t.Fatalf("reset state mismatch\nsession = %#v\nwant = %#v", session.state, want)
	}
	if len(messages) != len(want.Players)+1 {
		t.Fatalf("reset bootstrap messages = %d, want %d", len(messages), len(want.Players)+1)
	}
	for _, message := range messages {
		if message.Name != "StatePatched" {
			t.Fatalf("message name = %q, want %q", message.Name, "StatePatched")
		}
	}
}
