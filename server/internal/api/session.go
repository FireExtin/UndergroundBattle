package api

import (
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"slices"
	"strings"
	"sync"
	"time"

	"undergroundbattle/server/pkg/rules"
)

// Purpose: Holds one in-memory sandbox match session and converts authoritative dispatch outputs into protocol envelopes for the web debugger.

const protocolVersion = "0.1.0"
const defaultMatchReportDirectory = "runtime/match-reports"

type protocolEnvelope struct {
	Version   string          `json:"version"`
	Kind      string          `json:"kind"`
	MessageID string          `json:"messageId"`
	Name      string          `json:"name"`
	Revision  *int            `json:"revision,omitempty"`
	Payload   json.RawMessage `json:"payload"`
}

type MatchReport struct {
	GameID           string `json:"gameId"`
	Revision         int    `json:"revision"`
	WinnerPlayerID   string `json:"winnerPlayerId,omitempty"`
	EndReason        string `json:"endReason,omitempty"`
	FinishedRevision int    `json:"finishedRevision"`
	GeneratedAt      string `json:"generatedAt"`
	Path             string `json:"path"`
	Content          string `json:"content"`
}

type SandboxSessionOptions struct {
	Logger          *log.Logger
	ReportDirectory string
	Now             func() time.Time
}

type SandboxSession struct {
	mu                sync.Mutex
	state             rules.GameState
	projector         *rules.ProjectionEngine
	setup             SetupState
	setupRuntime      setupRuntimeState
	messages          []protocolEnvelope
	nextMessageNumber int
	logger            *log.Logger
	reportDirectory   string
	now               func() time.Time
	latestReport      *MatchReport
}

func NewSandboxSession() *SandboxSession {
	return NewSandboxSessionWithOptions(SandboxSessionOptions{})
}

func NewSandboxSessionWithOptions(options SandboxSessionOptions) *SandboxSession {
	logger := options.Logger
	if logger == nil {
		logger = log.Default()
	}

	reportDirectory := strings.TrimSpace(options.ReportDirectory)
	if reportDirectory == "" {
		reportDirectory = defaultMatchReportDirectory
	}

	now := options.Now
	if now == nil {
		now = time.Now
	}

	session := &SandboxSession{
		nextMessageNumber: 1,
		logger:            logger,
		reportDirectory:   reportDirectory,
		now:               now,
	}
	if _, err := session.resetLocked(); err != nil {
		panic(err)
	}
	return session
}

func (session *SandboxSession) Messages() []protocolEnvelope {
	session.mu.Lock()
	defer session.mu.Unlock()

	return cloneProtocolEnvelopes(session.messages)
}

func (session *SandboxSession) SubmitAction(action rules.Action) ([]protocolEnvelope, error) {
	session.mu.Lock()
	defer session.mu.Unlock()
	if session.setup.Active && !session.setup.Completed {
		return nil, errSetupNotCompleted
	}

	beforeMatch := session.state.Match
	result, err := rules.SubmitActionWithProjection(session.state, action, session.projector)
	if err != nil {
		var legality *rules.LegalityError
		if errors.As(err, &legality) {
			session.logActionRejected(action, legality.Result)
			return session.appendDispatchBatch(rules.BuildRejectedDispatchBatch(action, legality.Result))
		}

		return nil, err
	}

	session.state = result.State
	session.logActionAccepted(result.Accepted, result.State)
	if beforeMatch.Status != rules.MatchStatusFinished && result.State.Match.Status == rules.MatchStatusFinished {
		session.logMatchFinished(result.State)
		if err := session.generateMatchReportLocked(result.State); err != nil {
			session.logError("match_report_write_failed gameId=%s revision=%d err=%v", result.State.GameID, result.State.Revision.Number, err)
		}
	}
	return session.appendDispatchBatch(result.Dispatch)
}

func (session *SandboxSession) Reset() ([]protocolEnvelope, error) {
	session.mu.Lock()
	defer session.mu.Unlock()

	return session.resetLocked()
}

func (session *SandboxSession) LatestReport() (MatchReport, bool) {
	session.mu.Lock()
	defer session.mu.Unlock()

	if session.latestReport == nil {
		return MatchReport{}, false
	}

	return *session.latestReport, true
}

func (session *SandboxSession) appendDispatchBatch(batch rules.DispatchBatch) ([]protocolEnvelope, error) {
	envelopes := make([]protocolEnvelope, 0, len(batch.Messages))
	for _, dispatch := range batch.Messages {
		envelope, err := session.newEnvelopeForDispatch(dispatch)
		if err != nil {
			return nil, err
		}

		envelopes = append(envelopes, envelope)
	}

	session.messages = append(session.messages, cloneProtocolEnvelopes(envelopes)...)
	return cloneProtocolEnvelopes(envelopes), nil
}

func (session *SandboxSession) materializeBootstrapMessages(views rules.ProjectionBundle) ([]protocolEnvelope, error) {
	revisionNumber := session.state.Revision.Number
	bootstrapEvent := rules.Event{
		ID:               "evt:bootstrap",
		Kind:             rules.EventKind("bootstrap"),
		RevisionNumber:   revisionNumber,
		Phase:            session.state.Turn.Phase.Name,
		Step:             session.state.Turn.Phase.Step,
		PriorityPlayerID: session.state.Turn.Priority.CurrentPlayerID,
		PriorityWindow:   session.state.Turn.Priority.WindowKind,
		PassCount:        session.state.Turn.Priority.PassCount,
		StackDepth:       len(session.state.Board.Stack),
	}

	envelopes := make([]protocolEnvelope, 0, len(session.state.Players)+1)
	for _, playerID := range session.state.Players {
		payload := rules.NewStatePatchedForPlayer(views, playerID, bootstrapEvent, session.state.Revision)
		envelope, err := session.newEnvelope("view", "StatePatched", &revisionNumber, payload)
		if err != nil {
			return nil, err
		}

		envelopes = append(envelopes, envelope)
	}

	spectatorPayload := rules.NewStatePatchedForSpectator(views, bootstrapEvent, session.state.Revision)
	envelope, err := session.newEnvelope("view", "StatePatched", &revisionNumber, spectatorPayload)
	if err != nil {
		return nil, err
	}

	envelopes = append(envelopes, envelope)
	return envelopes, nil
}

func (session *SandboxSession) newEnvelopeForDispatch(dispatch rules.ClientDispatch) (protocolEnvelope, error) {
	switch dispatch.Kind {
	case rules.DispatchPayloadActionAccepted:
		if dispatch.ActionAccepted == nil {
			return protocolEnvelope{}, fmt.Errorf("missing ActionAccepted payload")
		}
		revision := dispatch.ActionAccepted.Revision.Number
		return session.newEnvelope("event", string(dispatch.Kind), &revision, dispatch.ActionAccepted)
	case rules.DispatchPayloadActionRejected:
		if dispatch.ActionRejected == nil {
			return protocolEnvelope{}, fmt.Errorf("missing ActionRejected payload")
		}
		return session.newEnvelope("event", string(dispatch.Kind), nil, dispatch.ActionRejected)
	case rules.DispatchPayloadStatePatched:
		if dispatch.StatePatched == nil {
			return protocolEnvelope{}, fmt.Errorf("missing StatePatched payload")
		}
		revision := dispatch.StatePatched.Revision.Number
		return session.newEnvelope("view", string(dispatch.Kind), &revision, dispatch.StatePatched)
	default:
		return protocolEnvelope{}, fmt.Errorf("unsupported dispatch kind %q", dispatch.Kind)
	}
}

func (session *SandboxSession) newEnvelope(kind string, name string, revision *int, payload any) (protocolEnvelope, error) {
	data, err := json.Marshal(payload)
	if err != nil {
		return protocolEnvelope{}, err
	}

	envelope := protocolEnvelope{
		Version:   protocolVersion,
		Kind:      kind,
		MessageID: fmt.Sprintf("msg-%06d", session.nextMessageNumber),
		Name:      name,
		Payload:   append(json.RawMessage(nil), data...),
	}
	if revision != nil {
		cloned := *revision
		envelope.Revision = &cloned
	}

	session.nextMessageNumber++
	return envelope, nil
}

func cloneProtocolEnvelopes(messages []protocolEnvelope) []protocolEnvelope {
	cloned := make([]protocolEnvelope, 0, len(messages))
	for _, message := range messages {
		next := message
		next.Payload = append(json.RawMessage(nil), message.Payload...)
		if message.Revision != nil {
			revision := *message.Revision
			next.Revision = &revision
		}
		cloned = append(cloned, next)
	}

	return cloned
}

func (session *SandboxSession) resetLocked() ([]protocolEnvelope, error) {
	state := rules.NewM0SandboxState()
	projector := rules.NewProjectionEngine()
	views := projector.Generate(state)

	session.state = state
	session.projector = projector
	session.setup = SetupState{}
	session.setupRuntime = setupRuntimeState{}
	session.latestReport = nil
	session.nextMessageNumber = 1
	messages, err := session.materializeBootstrapMessages(views)
	if err != nil {
		return nil, err
	}

	session.messages = cloneProtocolEnvelopes(messages)
	session.nextMessageNumber = len(messages) + 1
	return cloneProtocolEnvelopes(messages), nil
}

func (session *SandboxSession) logActionAccepted(accepted rules.ActionAccepted, state rules.GameState) {
	session.logInfo(
		"action_accepted actor=%s action=%s event=%s revision=%d phase=%s priority=%s stackDepth=%d score=%s",
		accepted.Action.ActorID,
		accepted.Action.Kind,
		accepted.Event.Kind,
		accepted.Revision.Number,
		accepted.Event.Phase,
		accepted.Event.PriorityPlayerID,
		accepted.Event.StackDepth,
		formatScore(state),
	)
}

func (session *SandboxSession) logActionRejected(action rules.Action, legality rules.LegalityResult) {
	session.logInfo(
		"action_rejected actor=%s action=%s reasonCode=%s messageKey=%s context=%s",
		action.ActorID,
		action.Kind,
		legality.ReasonCode,
		legality.MessageKey,
		formatContext(legality.Context),
	)
}

func (session *SandboxSession) logMatchFinished(state rules.GameState) {
	session.logInfo(
		"match_finished winner=%s endReason=%s finishedRevision=%d finalScore=%s",
		state.Match.WinnerPlayerID,
		state.Match.EndReason,
		state.Match.FinishedAtRevision,
		formatScore(state),
	)
}

func (session *SandboxSession) logInfo(format string, args ...any) {
	if session.logger == nil {
		return
	}
	session.logger.Printf("info "+format, args...)
}

func (session *SandboxSession) logError(format string, args ...any) {
	if session.logger == nil {
		return
	}
	session.logger.Printf("error "+format, args...)
}

func (session *SandboxSession) generateMatchReportLocked(state rules.GameState) error {
	reportTime := session.now().UTC()
	if err := os.MkdirAll(session.reportDirectory, 0o755); err != nil {
		return err
	}

	finishedRevision := state.Match.FinishedAtRevision
	if finishedRevision <= 0 {
		finishedRevision = state.Revision.Number
	}
	timestamp := reportTime.Format("20060102T150405Z")
	fileName := fmt.Sprintf("%s-rev%d-%s.md", sanitizePathSegment(state.GameID), finishedRevision, timestamp)
	reportPath := filepath.Join(session.reportDirectory, fileName)
	content := buildMatchReportMarkdown(state, reportTime)
	if err := os.WriteFile(reportPath, []byte(content), 0o644); err != nil {
		return err
	}

	absolutePath, err := filepath.Abs(reportPath)
	if err != nil {
		absolutePath = reportPath
	}

	session.latestReport = &MatchReport{
		GameID:           state.GameID,
		Revision:         state.Revision.Number,
		WinnerPlayerID:   state.Match.WinnerPlayerID,
		EndReason:        string(state.Match.EndReason),
		FinishedRevision: finishedRevision,
		GeneratedAt:      reportTime.Format(time.RFC3339),
		Path:             absolutePath,
		Content:          content,
	}
	return nil
}

func buildMatchReportMarkdown(state rules.GameState, reportTime time.Time) string {
	var builder strings.Builder
	builder.WriteString("# Match Report\n\n")
	builder.WriteString(fmt.Sprintf("- Game ID: %s\n", state.GameID))
	builder.WriteString(fmt.Sprintf("- Generated At: %s\n", reportTime.Format(time.RFC3339)))
	builder.WriteString(fmt.Sprintf("- Winner: %s\n", emptyAsDash(state.Match.WinnerPlayerID)))
	builder.WriteString(fmt.Sprintf("- End Reason: %s\n", emptyAsDash(string(state.Match.EndReason))))
	builder.WriteString(fmt.Sprintf("- Finished Revision: %d\n\n", state.Match.FinishedAtRevision))

	builder.WriteString("## Final Score\n\n")
	builder.WriteString("| Player | Score |\n")
	builder.WriteString("| --- | --- |\n")
	for _, playerID := range state.Players {
		builder.WriteString(fmt.Sprintf("| %s | %d |\n", playerID, state.Score.ByPlayer[playerID]))
	}
	builder.WriteString("\n")

	builder.WriteString("## Final Turn Snapshot\n\n")
	builder.WriteString(fmt.Sprintf("- Turn: %d\n", state.Turn.TurnNumber))
	builder.WriteString(fmt.Sprintf("- Active Player: %s\n", state.Turn.ActivePlayerID))
	builder.WriteString(fmt.Sprintf("- Priority Player: %s\n", state.Turn.Priority.CurrentPlayerID))
	builder.WriteString(fmt.Sprintf("- Phase: %s/%s\n\n", state.Turn.Phase.Name, state.Turn.Phase.Step))

	builder.WriteString("## Action Timeline\n\n")
	builder.WriteString("| Revision | Action ID | Actor | Action Kind | Operation ID | Operation Kind | Event Kind | Phase | Priority | StackDepth |\n")
	builder.WriteString("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n")
	length := minTimelineLength(state)
	for index := 0; index < length; index++ {
		revision := state.History.Revisions[index]
		action := state.History.Actions[index]
		operation := state.History.Operations[index]
		event := state.History.Events[index]
		builder.WriteString(fmt.Sprintf(
			"| %d | %s | %s | %s | %s | %s | %s | %s/%s | %s | %d |\n",
			revision.Number,
			action.ID,
			action.ActorID,
			action.Kind,
			operation.ID,
			operation.Kind,
			event.Kind,
			event.Phase,
			event.Step,
			event.PriorityPlayerID,
			event.StackDepth,
		))
	}

	builder.WriteString("\n## Board Snapshot\n\n")
	builder.WriteString(fmt.Sprintf("- Region cards on table: %d\n", countCards(state, rules.CardKindRegion, rules.CardZoneTable)))
	builder.WriteString(fmt.Sprintf("- Character cards on table: %d\n", countCards(state, rules.CardKindCharacter, rules.CardZoneTable)))
	builder.WriteString(fmt.Sprintf("- Discard cards: %d\n", countCardsByZone(state, rules.CardZoneDiscard)))
	builder.WriteString(fmt.Sprintf("- Score cards: %d\n", countCardsByZone(state, rules.CardZoneScore)))
	return builder.String()
}

func minTimelineLength(state rules.GameState) int {
	length := len(state.History.Revisions)
	length = min(length, len(state.History.Actions))
	length = min(length, len(state.History.Operations))
	length = min(length, len(state.History.Events))
	return length
}

func countCards(state rules.GameState, kind rules.CardKind, zone rules.CardZone) int {
	total := 0
	for _, card := range state.Board.Cards {
		if card.Kind == kind && card.Zone == zone && !card.Destroyed {
			total++
		}
	}
	return total
}

func countCardsByZone(state rules.GameState, zone rules.CardZone) int {
	total := 0
	for _, card := range state.Board.Cards {
		if card.Zone == zone && !card.Destroyed {
			total++
		}
	}
	return total
}

func sanitizePathSegment(raw string) string {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" {
		return "game"
	}

	var builder strings.Builder
	for _, r := range trimmed {
		switch {
		case r >= 'a' && r <= 'z':
			builder.WriteRune(r)
		case r >= 'A' && r <= 'Z':
			builder.WriteRune(r)
		case r >= '0' && r <= '9':
			builder.WriteRune(r)
		case r == '-' || r == '_':
			builder.WriteRune(r)
		default:
			builder.WriteRune('-')
		}
	}
	cleaned := strings.Trim(builder.String(), "-")
	if cleaned == "" {
		return "game"
	}
	return cleaned
}

func formatContext(context map[string]string) string {
	if len(context) == 0 {
		return "-"
	}

	keys := make([]string, 0, len(context))
	for key := range context {
		keys = append(keys, key)
	}
	slices.Sort(keys)

	parts := make([]string, 0, len(keys))
	for _, key := range keys {
		parts = append(parts, fmt.Sprintf("%s=%s", key, context[key]))
	}
	return strings.Join(parts, ",")
}

func formatScore(state rules.GameState) string {
	parts := make([]string, 0, len(state.Players))
	for _, playerID := range state.Players {
		parts = append(parts, fmt.Sprintf("%s:%d", playerID, state.Score.ByPlayer[playerID]))
	}
	return strings.Join(parts, ",")
}

func emptyAsDash(value string) string {
	if strings.TrimSpace(value) == "" {
		return "-"
	}
	return value
}
