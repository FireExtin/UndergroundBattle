package api

import (
	"bytes"
	"encoding/json"
	"log"
	"net/http"
	"net/http/httptest"
	"reflect"
	"testing"
	"time"

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

func TestLatestReportEndpointReturns404WhenNoReportExists(t *testing.T) {
	session := NewSandboxSessionWithOptions(SandboxSessionOptions{
		ReportDirectory: t.TempDir(),
	})
	handler := NewHandler(session, "")
	request := httptest.NewRequest(http.MethodGet, "/api/debugger/reports/latest", nil)
	recorder := httptest.NewRecorder()

	handler.ServeHTTP(recorder, request)

	if recorder.Code != http.StatusNotFound {
		t.Fatalf("status code = %d, want %d", recorder.Code, http.StatusNotFound)
	}

	var payload map[string]string
	if err := json.Unmarshal(recorder.Body.Bytes(), &payload); err != nil {
		t.Fatalf("json.Unmarshal returned error: %v", err)
	}
	if payload["error"] != "report_not_found" {
		t.Fatalf("error payload = %q, want %q", payload["error"], "report_not_found")
	}
}

func TestLatestReportEndpointReturnsMostRecentReport(t *testing.T) {
	var logBuffer bytes.Buffer
	session := NewSandboxSessionWithOptions(SandboxSessionOptions{
		Logger:          log.New(&logBuffer, "", 0),
		ReportDirectory: t.TempDir(),
		Now: func() time.Time {
			return time.Date(2026, time.April, 2, 13, 0, 0, 0, time.UTC)
		},
	})
	prepareSinglePointWin(t, session)

	_, err := session.SubmitAction(rules.Action{
		ID:      "act-http-report-finish",
		ActorID: "P1",
		Kind:    rules.ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	handler := NewHandler(session, "")
	request := httptest.NewRequest(http.MethodGet, "/api/debugger/reports/latest", nil)
	recorder := httptest.NewRecorder()

	handler.ServeHTTP(recorder, request)

	if recorder.Code != http.StatusOK {
		t.Fatalf("status code = %d, want %d", recorder.Code, http.StatusOK)
	}

	var payload MatchReport
	if err := json.Unmarshal(recorder.Body.Bytes(), &payload); err != nil {
		t.Fatalf("json.Unmarshal returned error: %v", err)
	}
	if payload.Path == "" {
		t.Fatal("payload.path is empty")
	}
	if payload.GameID != "game-sandbox-live" {
		t.Fatalf("payload.gameId = %q, want %q", payload.GameID, "game-sandbox-live")
	}
	if payload.Revision != 1 {
		t.Fatalf("payload.revision = %d, want %d", payload.Revision, 1)
	}
}

func TestSetupEndpointsStartAdvanceAndState(t *testing.T) {
	session := NewSandboxSession()
	handler := NewHandler(session, "")

	startRequest := httptest.NewRequest(http.MethodPost, "/api/battle/setup/start", bytes.NewBufferString(`{"seed":20260402}`))
	startRecorder := httptest.NewRecorder()
	handler.ServeHTTP(startRecorder, startRequest)
	if startRecorder.Code != http.StatusOK {
		t.Fatalf("start status code = %d, want %d", startRecorder.Code, http.StatusOK)
	}

	stateRequest := httptest.NewRequest(http.MethodGet, "/api/battle/setup/state", nil)
	stateRecorder := httptest.NewRecorder()
	handler.ServeHTTP(stateRecorder, stateRequest)
	if stateRecorder.Code != http.StatusOK {
		t.Fatalf("state status code = %d, want %d", stateRecorder.Code, http.StatusOK)
	}

	var setupState SetupState
	if err := json.Unmarshal(stateRecorder.Body.Bytes(), &setupState); err != nil {
		t.Fatalf("json.Unmarshal setup state returned error: %v", err)
	}
	if setupState.CurrentStep != 1 {
		t.Fatalf("setup current step = %d, want 1", setupState.CurrentStep)
	}

	advanceRequest := httptest.NewRequest(http.MethodPost, "/api/battle/setup/advance", bytes.NewBufferString(`{}`))
	advanceRecorder := httptest.NewRecorder()
	handler.ServeHTTP(advanceRecorder, advanceRequest)
	if advanceRecorder.Code != http.StatusOK {
		t.Fatalf("advance status code = %d, want %d", advanceRecorder.Code, http.StatusOK)
	}

	if _, err := session.SubmitAction(rules.Action{ID: "act-http-setup-gate", ActorID: "P1", Kind: rules.ActionKindPassPriority}); err == nil {
		t.Fatal("SubmitAction succeeded while setup is incomplete, want error")
	}
}
