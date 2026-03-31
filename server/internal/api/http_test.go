package api

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"reflect"
	"testing"

	"undergroundbattle/server/pkg/rules"
)

// Purpose: Verifies the minimal HTTP sandbox exposes the reset route and restores canonical bootstrap state.

func TestResetEndpointRestoresCanonicalSandboxState(t *testing.T) {
	session := NewSandboxSession()
	_, err := session.SubmitAction(rules.Action{
		ID:      "act-http-reset-1",
		ActorID: "P1",
		Kind:    rules.ActionKindRevealCard,
		CardID:  "P1-HAND-SECRET",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	handler := NewHandler(session, "")
	request := httptest.NewRequest(http.MethodPost, "/api/debugger/reset", nil)
	recorder := httptest.NewRecorder()

	handler.ServeHTTP(recorder, request)

	if recorder.Code != http.StatusOK {
		t.Fatalf("status code = %d, want %d", recorder.Code, http.StatusOK)
	}

	var messages []protocolEnvelope
	if err := json.Unmarshal(recorder.Body.Bytes(), &messages); err != nil {
		t.Fatalf("json.Unmarshal returned error: %v", err)
	}

	want := rules.NewM0SandboxState()
	if !reflect.DeepEqual(session.state, want) {
		t.Fatalf("reset state mismatch\nsession = %#v\nwant = %#v", session.state, want)
	}
	if len(messages) != len(want.Players)+1 {
		t.Fatalf("reset messages = %d, want %d", len(messages), len(want.Players)+1)
	}
}
