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
