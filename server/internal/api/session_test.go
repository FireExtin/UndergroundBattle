package api

import (
	"bytes"
	"encoding/json"
	"log"
	"os"
	"reflect"
	"strings"
	"testing"
	"time"

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

func TestSandboxSessionResetBootstrapMessagesUseResetRevision(t *testing.T) {
	session := NewSandboxSession()
	_, err := session.SubmitAction(rules.Action{
		ID:      "act-session-reset-revision-1",
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

	for index, message := range messages {
		if message.Revision == nil {
			t.Fatalf("messages[%d].revision is nil, want 0", index)
		}

		if *message.Revision != 0 {
			t.Fatalf("messages[%d].revision = %d, want 0", index, *message.Revision)
		}

		var payload rules.StatePatched
		if err := json.Unmarshal(message.Payload, &payload); err != nil {
			t.Fatalf("json.Unmarshal(messages[%d]) returned error: %v", index, err)
		}

		if payload.Revision.Number != 0 {
			t.Fatalf("payload revision = %d, want 0", payload.Revision.Number)
		}

		if payload.Event.RevisionNumber != 0 {
			t.Fatalf("payload event revision = %d, want 0", payload.Event.RevisionNumber)
		}
	}
}

func TestSandboxSessionLogsAcceptedRejectedAndFinishedAtInfoLevel(t *testing.T) {
	var logBuffer bytes.Buffer
	reportDir := t.TempDir()
	session := NewSandboxSessionWithOptions(SandboxSessionOptions{
		Logger:          log.New(&logBuffer, "", 0),
		ReportDirectory: reportDir,
		Now: func() time.Time {
			return time.Date(2026, time.April, 2, 12, 34, 56, 0, time.UTC)
		},
	})

	_, err := session.SubmitAction(rules.Action{
		ID:      "act-rejected-priority",
		ActorID: "P2",
		Kind:    rules.ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction(rejected) returned error: %v", err)
	}

	prepareSinglePointWin(t, session)
	_, err = session.SubmitAction(rules.Action{
		ID:      "act-finish-match",
		ActorID: "P1",
		Kind:    rules.ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction(finished) returned error: %v", err)
	}

	logOutput := logBuffer.String()
	for _, needle := range []string{
		"action_rejected",
		"action_accepted",
		"match_finished",
		"winner=P1",
		"revision=1",
	} {
		if !strings.Contains(logOutput, needle) {
			t.Fatalf("log output missing %q\nlogs:\n%s", needle, logOutput)
		}
	}

	report, ok := session.LatestReport()
	if !ok {
		t.Fatal("LatestReport() = not found, want generated report")
	}

	if report.Path == "" {
		t.Fatal("report path is empty")
	}
	if _, err := os.Stat(report.Path); err != nil {
		t.Fatalf("os.Stat(report.Path) returned error: %v", err)
	}
	if !strings.Contains(report.Content, "# Match Report") {
		t.Fatalf("report content missing title:\n%s", report.Content)
	}
	if !strings.Contains(report.Content, "Winner: P1") {
		t.Fatalf("report content missing winner:\n%s", report.Content)
	}
	if !strings.Contains(report.Content, "act-finish-match") {
		t.Fatalf("report content missing action timeline:\n%s", report.Content)
	}
}

func TestSandboxSessionResetClearsLatestReport(t *testing.T) {
	session := NewSandboxSessionWithOptions(SandboxSessionOptions{
		ReportDirectory: t.TempDir(),
	})
	prepareSinglePointWin(t, session)

	_, err := session.SubmitAction(rules.Action{
		ID:      "act-reset-report-finish",
		ActorID: "P1",
		Kind:    rules.ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}
	if _, ok := session.LatestReport(); !ok {
		t.Fatal("LatestReport() = not found, want generated report")
	}

	_, err = session.Reset()
	if err != nil {
		t.Fatalf("Reset returned error: %v", err)
	}
	if _, ok := session.LatestReport(); ok {
		t.Fatal("LatestReport unexpectedly exists after reset")
	}
}

func TestSandboxSessionSubmitStillSucceedsWhenReportWriteFails(t *testing.T) {
	reportFile, err := os.CreateTemp(t.TempDir(), "report-dir-is-file-*.tmp")
	if err != nil {
		t.Fatalf("os.CreateTemp returned error: %v", err)
	}
	if err := reportFile.Close(); err != nil {
		t.Fatalf("reportFile.Close returned error: %v", err)
	}

	var logBuffer bytes.Buffer
	session := NewSandboxSessionWithOptions(SandboxSessionOptions{
		Logger:          log.New(&logBuffer, "", 0),
		ReportDirectory: reportFile.Name(),
	})
	prepareSinglePointWin(t, session)

	messages, err := session.SubmitAction(rules.Action{
		ID:      "act-finish-report-fail",
		ActorID: "P1",
		Kind:    rules.ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	foundAccepted := false
	for _, message := range messages {
		if message.Name == "ActionAccepted" {
			foundAccepted = true
			break
		}
	}
	if !foundAccepted {
		t.Fatalf("messages did not include ActionAccepted: %#v", messages)
	}

	if _, ok := session.LatestReport(); ok {
		t.Fatal("LatestReport unexpectedly exists after write failure")
	}
	if !strings.Contains(logBuffer.String(), "match_report_write_failed") {
		t.Fatalf("expected write failure log, got:\n%s", logBuffer.String())
	}
}

func TestSandboxSessionSetupFlowBlocksActionsUntilCompleted(t *testing.T) {
	session := NewSandboxSession()

	_, err := session.StartSetup(SetupStartInput{Seed: 20260402})
	if err != nil {
		t.Fatalf("StartSetup returned error: %v", err)
	}

	_, err = session.SubmitAction(rules.Action{
		ID:      "act-setup-blocked",
		ActorID: "P1",
		Kind:    rules.ActionKindPassPriority,
	})
	if err == nil {
		t.Fatal("SubmitAction succeeded before setup completion, want setup_not_completed error")
	}
	if !strings.Contains(err.Error(), "setup_not_completed") {
		t.Fatalf("SubmitAction error = %q, want setup_not_completed", err.Error())
	}

	for step := 1; step <= 7; step++ {
		_, err = session.AdvanceSetup(SetupAdvanceInput{})
		if err != nil {
			t.Fatalf("AdvanceSetup(step=%d) returned error: %v", step, err)
		}
	}

	actorID := session.state.Turn.Priority.CurrentPlayerID
	if actorID == "" {
		actorID = "P1"
	}
	_, err = session.SubmitAction(rules.Action{
		ID:      "act-setup-finished",
		ActorID: actorID,
		Kind:    rules.ActionKindPassPriority,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error after setup completion: %v", err)
	}
}

func TestSandboxSessionSetupStateTracksCurrentStep(t *testing.T) {
	session := NewSandboxSession()

	state, err := session.StartSetup(SetupStartInput{Seed: 42})
	if err != nil {
		t.Fatalf("StartSetup returned error: %v", err)
	}
	if !state.Active {
		t.Fatal("setup state should be active after start")
	}
	if state.CurrentStep != 1 {
		t.Fatalf("current step = %d, want 1", state.CurrentStep)
	}

	state, err = session.AdvanceSetup(SetupAdvanceInput{})
	if err != nil {
		t.Fatalf("AdvanceSetup returned error: %v", err)
	}
	if state.CurrentStep != 2 {
		t.Fatalf("current step = %d, want 2", state.CurrentStep)
	}

	for step := 2; step <= 6; step++ {
		state, err = session.AdvanceSetup(SetupAdvanceInput{})
		if err != nil {
			t.Fatalf("AdvanceSetup(step=%d) returned error: %v", step, err)
		}
	}
	if state.Completed {
		t.Fatal("setup should not be completed before advancing step 7")
	}
	if state.CurrentStep != 7 {
		t.Fatalf("current step = %d, want 7 before final advance", state.CurrentStep)
	}
	if isSetupStepCompleted(state.Steps, 7) {
		t.Fatal("setup step 7 should not be marked completed before final advance")
	}

	state, err = session.AdvanceSetup(SetupAdvanceInput{})
	if err != nil {
		t.Fatalf("AdvanceSetup(step=7) returned error: %v", err)
	}
	if !state.Completed {
		t.Fatal("setup should be completed after step 7")
	}
	if state.CurrentStep != 7 {
		t.Fatalf("current step = %d, want 7", state.CurrentStep)
	}
	if !isSetupStepCompleted(state.Steps, 7) {
		t.Fatal("setup step 7 should be marked completed after final advance")
	}
}

func isSetupStepCompleted(steps []SetupStepStatus, targetStep int) bool {
	for _, step := range steps {
		if step.Step == targetStep {
			return step.Completed
		}
	}
	return false
}

func prepareSinglePointWin(t *testing.T, session *SandboxSession) {
	t.Helper()

	session.mu.Lock()
	defer session.mu.Unlock()

	session.state.Turn.Phase = rules.PhaseState{
		Name:        rules.PhaseEnd,
		Step:        rules.StepAction,
		AllowsStack: false,
		StepEnded:   false,
	}
	session.state.Score.VictoryThreshold = 1

	regionFound := false
	for index := range session.state.Board.Cards {
		card := &session.state.Board.Cards[index]
		if card.CardID != "REGION-1" {
			continue
		}
		card.InfluenceByPlayer = map[string]int{"P1": 1}
		regionFound = true
		break
	}

	if !regionFound {
		t.Fatal("REGION-1 not found in canonical state")
	}
}
