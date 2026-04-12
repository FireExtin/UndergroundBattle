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

func TestSandboxSessionWritesLiveTraceBeforeMatchFinish(t *testing.T) {
	traceDir := t.TempDir()
	session := NewSandboxSessionWithOptions(SandboxSessionOptions{
		TraceDirectory: traceDir,
		Now: func() time.Time {
			return time.Date(2026, time.April, 3, 10, 0, 0, 0, time.UTC)
		},
	})

	_, err := session.SubmitAction(rules.Action{
		ID:      "act-trace-rejected-priority",
		ActorID: "P2",
		Kind:    rules.ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction(rejected) returned error: %v", err)
	}

	trace, ok := session.LatestTrace()
	if !ok {
		t.Fatal("LatestTrace() = not found, want generated trace")
	}
	if trace.Path == "" {
		t.Fatal("trace path is empty")
	}
	if trace.EntryCount < 1 {
		t.Fatalf("trace entry count = %d, want >= 1", trace.EntryCount)
	}
	if !strings.Contains(trace.Content, "action_rejected") {
		t.Fatalf("trace content missing action_rejected:\n%s", trace.Content)
	}
	if !strings.Contains(trace.Content, "rules.legality.not_your_priority") {
		t.Fatalf("trace content missing rejection reason:\n%s", trace.Content)
	}
}

func TestSandboxSessionTraceIncludesSetupTransitions(t *testing.T) {
	traceDir := t.TempDir()
	session := NewSandboxSessionWithOptions(SandboxSessionOptions{
		TraceDirectory: traceDir,
		Now: func() time.Time {
			return time.Date(2026, time.April, 3, 11, 0, 0, 0, time.UTC)
		},
	})

	_, err := session.StartSetup(SetupStartInput{Seed: 20260403})
	if err != nil {
		t.Fatalf("StartSetup returned error: %v", err)
	}
	_, err = session.AdvanceSetup(SetupAdvanceInput{})
	if err != nil {
		t.Fatalf("AdvanceSetup returned error: %v", err)
	}

	trace, ok := session.LatestTrace()
	if !ok {
		t.Fatal("LatestTrace() = not found, want generated trace")
	}
	if !strings.Contains(trace.Content, "setup_started") {
		t.Fatalf("trace content missing setup_started:\n%s", trace.Content)
	}
	if !strings.Contains(trace.Content, "setup_advanced") {
		t.Fatalf("trace content missing setup_advanced:\n%s", trace.Content)
	}
}

func TestRenderTraceStateSnapshotIncludesConflictAndPromptDetails(t *testing.T) {
	state := rules.NewGameState(rules.InitialStateConfig{
		GameID:         "trace-conflict-details",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Phase = rules.PhaseState{Name: rules.PhaseConflict, Step: rules.StepAction, AllowsStack: true}
	state.Turn.Priority = rules.PriorityState{
		CurrentPlayerID:    "P2",
		PassCount:          1,
		LastPassedPlayerID: "P1",
		WindowKind:         rules.PriorityWindowResponse,
	}
	state.Turn.Conflict = rules.ConflictState{
		RegionOrder:               2,
		RegionCardID:              "region-2",
		Stage:                     rules.ConflictStageBattleDamagePrompt,
		PriorityLeaderPlayerID:    "P1",
		FirstPlayerPrivilegeOwner: "P1",
		PendingPromptID:           "prompt:battle_damage:region-2",
	}
	state.Turn.PendingPrompt = &rules.PromptState{
		ID:                "prompt:battle_damage:region-2",
		Kind:              rules.PromptKindBattleDamage,
		OwnerPlayerID:     "P1",
		RegionCardID:      "region-2",
		EligibleTargetIDs: []string{"unit-1", "unit-2"},
		RemainingAmount:   2,
		Difference:        2,
	}
	state.Board.Stack = []rules.Operation{{ID: "stack-1"}}

	snapshot := renderTraceStateSnapshot(state)
	for _, fragment := range []string{
		"Priority Window: response",
		"Conflict: region=region-2 order=2 stage=battle_damage_prompt leader=P1 privilege=P1 pendingPrompt=prompt:battle_damage:region-2",
		"Pending Prompt: id=prompt:battle_damage:region-2 kind=battle_damage owner=P1 region=region-2 diff=2 remaining=2 eligible=2 peek=0",
		"Stack Depth: 1",
	} {
		if !strings.Contains(snapshot, fragment) {
			t.Fatalf("snapshot missing %q:\n%s", fragment, snapshot)
		}
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

func TestSandboxSessionLifecycleTransitions(t *testing.T) {
	session := NewSandboxSessionWithOptions(SandboxSessionOptions{
		ReportDirectory: t.TempDir(),
	})

	if got := session.SetupState().Lifecycle.Kind; got != SessionLifecycleReset {
		t.Fatalf("initial lifecycle = %q, want %q", got, SessionLifecycleReset)
	}

	state, err := session.StartSetup(SetupStartInput{Seed: 20260403})
	if err != nil {
		t.Fatalf("StartSetup returned error: %v", err)
	}
	if state.Lifecycle.Kind != SessionLifecycleSetup {
		t.Fatalf("lifecycle kind after start = %q, want %q", state.Lifecycle.Kind, SessionLifecycleSetup)
	}
	if state.Lifecycle.SetupStep != 1 {
		t.Fatalf("setup lifecycle step after start = %d, want 1", state.Lifecycle.SetupStep)
	}

	state, err = session.AdvanceSetup(SetupAdvanceInput{})
	if err != nil {
		t.Fatalf("AdvanceSetup(step=1) returned error: %v", err)
	}
	if state.Lifecycle.SetupStep != 2 {
		t.Fatalf("setup lifecycle step after advance = %d, want 2", state.Lifecycle.SetupStep)
	}

	for step := 2; step <= 7; step++ {
		state, err = session.AdvanceSetup(SetupAdvanceInput{})
		if err != nil {
			t.Fatalf("AdvanceSetup(step=%d) returned error: %v", step, err)
		}
	}
	if state.Lifecycle.Kind != SessionLifecycleMatchActive {
		t.Fatalf("lifecycle kind after setup completion = %q, want %q", state.Lifecycle.Kind, SessionLifecycleMatchActive)
	}

	_, err = session.Reset()
	if err != nil {
		t.Fatalf("Reset returned error before canonical finish path: %v", err)
	}
	if got := session.SetupState().Lifecycle.Kind; got != SessionLifecycleReset {
		t.Fatalf("lifecycle kind after intermediate reset = %q, want %q", got, SessionLifecycleReset)
	}

	prepareSinglePointWin(t, session)
	_, err = session.SubmitAction(rules.Action{
		ID:      "act-session-lifecycle-finish",
		ActorID: "P1",
		Kind:    rules.ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	finished := session.SetupState().Lifecycle
	if finished.Kind != SessionLifecycleMatchFinished {
		t.Fatalf("lifecycle kind after finish = %q, want %q", finished.Kind, SessionLifecycleMatchFinished)
	}
	if finished.FinishedRevision <= 0 {
		t.Fatalf("finished revision = %d, want > 0", finished.FinishedRevision)
	}
	if finished.ReportPath == "" {
		t.Fatal("finished lifecycle report path is empty")
	}

	_, err = session.Reset()
	if err != nil {
		t.Fatalf("Reset returned error: %v", err)
	}
	if got := session.SetupState().Lifecycle.Kind; got != SessionLifecycleReset {
		t.Fatalf("lifecycle kind after reset = %q, want %q", got, SessionLifecycleReset)
	}
}

func TestSandboxSessionSetupRejectsDuplicateSocietyChoicesAtStepOne(t *testing.T) {
	session := NewSandboxSession()

	_, err := session.StartSetup(SetupStartInput{
		Seed:        20260403,
		P1Societies: []string{"帷幕守望", "帷幕守望"},
		P2Societies: []string{"王座会", "国家机构"},
	})
	if err != nil {
		t.Fatalf("StartSetup returned error: %v", err)
	}

	_, err = session.AdvanceSetup(SetupAdvanceInput{
		P1Societies: []string{"帷幕守望", "帷幕守望"},
		P2Societies: []string{"王座会", "国家机构"},
	})
	if err == nil {
		t.Fatal("AdvanceSetup succeeded with duplicate societies, want early rejection")
	}
	if !strings.Contains(err.Error(), "society_duplicate") {
		t.Fatalf("AdvanceSetup error = %q, want society_duplicate", err.Error())
	}
}

func TestSandboxSessionSetupRejectsSingleSocietyChoiceAtStepOne(t *testing.T) {
	session := NewSandboxSession()

	_, err := session.StartSetup(SetupStartInput{
		Seed:        20260403,
		P1Societies: []string{"帷幕守望"},
		P2Societies: []string{"王座会", "国家机构"},
	})
	if err != nil {
		t.Fatalf("StartSetup returned error: %v", err)
	}

	_, err = session.AdvanceSetup(SetupAdvanceInput{
		P1Societies: []string{"帷幕守望"},
		P2Societies: []string{"王座会", "国家机构"},
	})
	if err == nil {
		t.Fatal("AdvanceSetup succeeded with one-society selection, want early rejection")
	}
	if !strings.Contains(err.Error(), "society_count_invalid") {
		t.Fatalf("AdvanceSetup error = %q, want society_count_invalid", err.Error())
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
		card.BaseInfluenceByPlayer = map[string]int{"P1": 1}
		regionFound = true
		break
	}

	if !regionFound {
		t.Fatal("REGION-1 not found in canonical state")
	}
}
