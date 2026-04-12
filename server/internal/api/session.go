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
const defaultMatchTraceDirectory = "runtime/match-traces"

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

type MatchTrace struct {
	GameID     string `json:"gameId"`
	StartedAt  string `json:"startedAt"`
	UpdatedAt  string `json:"updatedAt"`
	Path       string `json:"path"`
	EntryCount int    `json:"entryCount"`
	Content    string `json:"content"`
}

type SandboxSessionOptions struct {
	Logger          *log.Logger
	ReportDirectory string
	TraceDirectory  string
	Now             func() time.Time
}

type SandboxSession struct {
	mu                sync.Mutex
	state             rules.GameState
	projector         *rules.ProjectionEngine
	lifecycle         SessionLifecycle
	setup             SetupState
	setupRuntime      setupRuntimeState
	messages          []protocolEnvelope
	nextMessageNumber int
	logger            *log.Logger
	reportDirectory   string
	traceDirectory    string
	now               func() time.Time
	latestReport      *MatchReport
	latestTrace       *MatchTrace
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
	reportDirectory = resolveSandboxPath(reportDirectory)
	traceDirectory := strings.TrimSpace(options.TraceDirectory)
	if traceDirectory == "" {
		traceDirectory = defaultMatchTraceDirectory
	}
	traceDirectory = resolveSandboxPath(traceDirectory)

	now := options.Now
	if now == nil {
		now = time.Now
	}

	session := &SandboxSession{
		nextMessageNumber: 1,
		logger:            logger,
		reportDirectory:   reportDirectory,
		traceDirectory:    traceDirectory,
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
	if session.lifecycle.Kind == SessionLifecycleSetup {
		return nil, errSetupNotCompleted
	}

	session.appendMatchTraceEntryLocked("action_submitted", map[string]any{
		"id":                 action.ID,
		"actorId":            action.ActorID,
		"kind":               action.Kind,
		"cardId":             action.CardID,
		"targetCardId":       action.TargetCardID,
		"targetPlayerId":     action.TargetPlayerID,
		"targetRegionCardId": action.TargetRegionCardID,
		"playMode":           action.PlayMode,
	}, &session.state)

	beforeMatch := session.state.Match
	result, err := rules.SubmitActionWithProjection(session.state, action, session.projector)
	if err != nil {
		var legality *rules.LegalityError
		if errors.As(err, &legality) {
			session.logActionRejected(action, legality.Result)
			session.appendMatchTraceEntryLocked("action_rejected", map[string]any{
				"actionId":    action.ID,
				"actorId":     action.ActorID,
				"kind":        action.Kind,
				"reasonCode":  legality.Result.ReasonCode,
				"messageKey":  legality.Result.MessageKey,
				"hook":        legality.Result.Hook,
				"context":     legality.Result.Context,
				"message":     legality.Message,
				"errorString": err.Error(),
			}, &session.state)
			return session.appendDispatchBatch(rules.BuildRejectedDispatchBatch(action, legality.Result))
		}

		return nil, err
	}

	session.state = result.State
	session.logActionAccepted(result.Accepted, result.State)
	session.appendMatchTraceEntryLocked("action_accepted", map[string]any{
		"action":    result.Accepted.Action,
		"operation": result.Accepted.Operation,
		"event":     result.Accepted.Event,
		"revision":  result.Accepted.Revision,
	}, &session.state)
	if beforeMatch.Status != rules.MatchStatusFinished && result.State.Match.Status == rules.MatchStatusFinished {
		session.logMatchFinished(result.State)
		if err := session.generateMatchReportLocked(result.State); err != nil {
			session.logError("match_report_write_failed gameId=%s revision=%d err=%v", result.State.GameID, result.State.Revision.Number, err)
		}
		finishedRevision := result.State.Match.FinishedAtRevision
		if finishedRevision <= 0 {
			finishedRevision = result.State.Revision.Number
		}
		if err := session.Transition(newMatchFinishedLifecycle(session.latestReport, finishedRevision)); err != nil {
			session.logError("match_finished_transition_failed gameId=%s err=%v", result.State.GameID, err)
			return nil, err
		}
		session.appendMatchTraceEntryLocked("match_finished", map[string]any{
			"winner":           result.State.Match.WinnerPlayerID,
			"endReason":        result.State.Match.EndReason,
			"finishedRevision": result.State.Match.FinishedAtRevision,
			"latestReportPath": latestReportPath(session.latestReport),
		}, &session.state)
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

func (session *SandboxSession) LatestTrace() (MatchTrace, bool) {
	session.mu.Lock()
	defer session.mu.Unlock()

	if session.latestTrace == nil {
		return MatchTrace{}, false
	}

	trace := *session.latestTrace
	content, err := os.ReadFile(trace.Path)
	if err == nil {
		trace.Content = string(content)
	}
	return trace, true
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
	_ = session.Transition(newResetLifecycle())
	session.setupRuntime = setupRuntimeState{}
	session.latestReport = nil
	session.latestTrace = nil
	session.nextMessageNumber = 1
	if err := session.startMatchTraceLocked(state, "reset"); err != nil {
		session.logError("match_trace_start_failed gameId=%s err=%v", state.GameID, err)
	}
	session.appendMatchTraceEntryLocked("session_reset", map[string]any{
		"reason": "api.debugger.reset",
	}, &session.state)
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

func (session *SandboxSession) startMatchTraceLocked(state rules.GameState, trigger string) error {
	startedAt := session.now().UTC()
	if err := os.MkdirAll(session.traceDirectory, 0o755); err != nil {
		return err
	}

	fileName := fmt.Sprintf("%s-%s.log", sanitizePathSegment(state.GameID), startedAt.Format("20060102T150405Z"))
	tracePath := filepath.Join(session.traceDirectory, fileName)
	absolutePath, err := filepath.Abs(tracePath)
	if err != nil {
		absolutePath = tracePath
	}

	var builder strings.Builder
	builder.WriteString("# Match Trace\n\n")
	builder.WriteString(fmt.Sprintf("- Game ID: %s\n", state.GameID))
	builder.WriteString(fmt.Sprintf("- Started At: %s\n", startedAt.Format(time.RFC3339)))
	builder.WriteString(fmt.Sprintf("- Trigger: %s\n", emptyAsDash(trigger)))
	builder.WriteString("\n")
	if err := os.WriteFile(absolutePath, []byte(builder.String()), 0o644); err != nil {
		return err
	}

	session.latestTrace = &MatchTrace{
		GameID:     state.GameID,
		StartedAt:  startedAt.Format(time.RFC3339),
		UpdatedAt:  startedAt.Format(time.RFC3339),
		Path:       absolutePath,
		EntryCount: 0,
	}
	return nil
}

func (session *SandboxSession) appendMatchTraceEntryLocked(kind string, payload any, state *rules.GameState) {
	if session.latestTrace == nil {
		return
	}

	at := session.now().UTC()
	var builder strings.Builder

	// Game化的日志格式
	switch kind {
	case "action_submitted":
		if data, ok := payload.(map[string]any); ok {
			actorID := data["actorId"].(string)
			var actionKind string
			kindVal := data["kind"]
			switch v := kindVal.(type) {
			case string:
				actionKind = v
			default:
				actionKind = fmt.Sprintf("%v", v)
			}
			builder.WriteString(fmt.Sprintf("## %s %s\n\n", at.Format(time.RFC3339), "action_submitted"))
			builder.WriteString(fmt.Sprintf("**%s** 发起了动作: %s\n\n", actorID, formatActionKind(actionKind)))
			if cardID, ok := data["cardId"].(string); ok && cardID != "" {
				builder.WriteString(fmt.Sprintf("- 卡牌: %s\n", cardID))
			}
			if targetCardID, ok := data["targetCardId"].(string); ok && targetCardID != "" {
				builder.WriteString(fmt.Sprintf("- 目标: %s\n", targetCardID))
			}
			builder.WriteString("\n")
		}
	case "action_accepted":
		if data, ok := payload.(map[string]any); ok {
			var actorID, actionKind string
			actionVal := data["action"]
			switch v := actionVal.(type) {
			case map[string]any:
				actorID = v["actorId"].(string)
				kindVal := v["kind"]
				switch kv := kindVal.(type) {
				case string:
					actionKind = kv
				default:
					actionKind = fmt.Sprintf("%v", kv)
				}
			default:
				// 处理其他类型，如 rules.Action
				actorID = "unknown"
				actionKind = fmt.Sprintf("%v", v)
			}
			builder.WriteString(fmt.Sprintf("## %s %s\n\n", at.Format(time.RFC3339), "action_accepted"))
			builder.WriteString(fmt.Sprintf("✅ **%s** 的动作已接受: %s\n\n", actorID, formatActionKind(actionKind)))
			eventVal := data["event"]
			var eventKind string
			switch v := eventVal.(type) {
			case map[string]any:
				eventKindVal := v["kind"]
				switch kv := eventKindVal.(type) {
				case string:
					eventKind = kv
				default:
					eventKind = fmt.Sprintf("%v", kv)
				}
			default:
				eventKind = fmt.Sprintf("%v", v)
			}
			builder.WriteString(fmt.Sprintf("- 事件: %s\n", formatEventKind(eventKind)))
			builder.WriteString("\n")
		}
	case "action_rejected":
		if data, ok := payload.(map[string]any); ok {
			// 提取 actorID
			actorID := "unknown"
			if id, ok := data["actorId"].(string); ok {
				actorID = id
			}

			// 提取 actionKind
			var actionKind string
			kindVal := data["kind"]
			switch v := kindVal.(type) {
			case string:
				actionKind = v
			default:
				actionKind = fmt.Sprintf("%v", v)
			}

			// 提取拒绝原因
			reasonMessage := ""
			if msg, ok := data["message"].(string); ok {
				reasonMessage = msg
			}

			builder.WriteString(fmt.Sprintf("## %s %s\n\n", at.Format(time.RFC3339), "action_rejected"))
			builder.WriteString(fmt.Sprintf("❌ **%s** 的动作被拒绝: %s\n\n", actorID, formatActionKind(actionKind)))
			if reasonMessage != "" {
				builder.WriteString(fmt.Sprintf("- 原因: %s\n", reasonMessage))
			}
			builder.WriteString("\n")
		}
	case "match_finished":
		if data, ok := payload.(map[string]any); ok {
			winner := data["winner"].(string)
			var endReason string
			endReasonVal := data["endReason"]
			switch v := endReasonVal.(type) {
			case string:
				endReason = v
			default:
				endReason = fmt.Sprintf("%v", v)
			}
			builder.WriteString(fmt.Sprintf("## %s %s\n\n", at.Format(time.RFC3339), "match_finished"))
			builder.WriteString("🏆 **游戏结束**\n\n")
			builder.WriteString(fmt.Sprintf("- 获胜者: %s\n", emptyAsDash(winner)))
			builder.WriteString(fmt.Sprintf("- 结束原因: %s\n\n", endReason))
		}
	default:
		// 保持原有格式
		builder.WriteString(fmt.Sprintf("## %s %s\n\n", at.Format(time.RFC3339), emptyAsDash(kind)))
		if payload != nil {
			if data, err := json.MarshalIndent(payload, "", "  "); err == nil {
				builder.WriteString("```json\n")
				builder.Write(data)
				builder.WriteString("\n```\n\n")
			}
		}
	}

	// 状态快照 - 简化版
	if state != nil {
		builder.WriteString(renderGameStateSnapshot(*state))
		builder.WriteString("\n")
	}

	file, err := os.OpenFile(session.latestTrace.Path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o644)
	if err != nil {
		session.logError("match_trace_append_open_failed path=%s err=%v", session.latestTrace.Path, err)
		return
	}
	defer func() {
		_ = file.Close()
	}()
	if _, err := file.WriteString(builder.String()); err != nil {
		session.logError("match_trace_append_write_failed path=%s err=%v", session.latestTrace.Path, err)
		return
	}

	session.latestTrace.UpdatedAt = at.Format(time.RFC3339)
	session.latestTrace.EntryCount++
}

func renderTraceStateSnapshot(state rules.GameState) string {
	var builder strings.Builder
	builder.WriteString("### State Snapshot\n\n")
	builder.WriteString(fmt.Sprintf("- Revision: %d\n", state.Revision.Number))
	builder.WriteString(fmt.Sprintf("- Match: %s\n", state.Match.Status))
	builder.WriteString(fmt.Sprintf("- Winner: %s\n", emptyAsDash(state.Match.WinnerPlayerID)))
	builder.WriteString(fmt.Sprintf("- Turn: %d\n", state.Turn.TurnNumber))
	builder.WriteString(fmt.Sprintf("- Active Player: %s\n", emptyAsDash(state.Turn.ActivePlayerID)))
	builder.WriteString(fmt.Sprintf("- Priority Player: %s\n", emptyAsDash(state.Turn.Priority.CurrentPlayerID)))
	builder.WriteString(fmt.Sprintf("- Priority Window: %s\n", emptyAsDash(string(state.Turn.Priority.WindowKind))))
	builder.WriteString(fmt.Sprintf("- Phase: %s/%s\n", state.Turn.Phase.Name, state.Turn.Phase.Step))
	builder.WriteString(fmt.Sprintf("- Conflict: %s\n", traceConflictSummary(state)))
	builder.WriteString(fmt.Sprintf("- Pending Prompt: %s\n", tracePendingPromptSummary(state)))
	builder.WriteString(fmt.Sprintf("- Pass Count: %d\n", state.Turn.Priority.PassCount))
	builder.WriteString(fmt.Sprintf("- Stack Depth: %d\n", len(state.Board.Stack)))
	builder.WriteString(fmt.Sprintf("- Resources: %s\n", traceResourceSummary(state)))
	builder.WriteString(fmt.Sprintf("- Score: %s\n", formatScore(state)))
	builder.WriteString(fmt.Sprintf("- Board: %s\n", traceBoardSummary(state)))
	return builder.String()
}

func traceConflictSummary(state rules.GameState) string {
	conflict := state.Turn.Conflict
	if conflict.RegionCardID == "" && conflict.Stage == "" && conflict.PriorityLeaderPlayerID == "" && conflict.PendingPromptID == "" {
		return "-"
	}

	return fmt.Sprintf(
		"region=%s order=%d stage=%s leader=%s privilege=%s pendingPrompt=%s",
		emptyAsDash(conflict.RegionCardID),
		conflict.RegionOrder,
		emptyAsDash(string(conflict.Stage)),
		emptyAsDash(conflict.PriorityLeaderPlayerID),
		emptyAsDash(conflict.FirstPlayerPrivilegeOwner),
		emptyAsDash(conflict.PendingPromptID),
	)
}

func tracePendingPromptSummary(state rules.GameState) string {
	prompt := state.Turn.PendingPrompt
	if prompt == nil {
		return "-"
	}

	return fmt.Sprintf(
		"id=%s kind=%s owner=%s region=%s diff=%d remaining=%d eligible=%d peek=%d",
		emptyAsDash(prompt.ID),
		emptyAsDash(string(prompt.Kind)),
		emptyAsDash(prompt.OwnerPlayerID),
		emptyAsDash(prompt.RegionCardID),
		prompt.Difference,
		prompt.RemainingAmount,
		len(prompt.EligibleTargetIDs),
		len(prompt.PeekCardIDs),
	)
}

func traceResourceSummary(state rules.GameState) string {
	if len(state.Turn.Resources) == 0 {
		return "-"
	}
	parts := make([]string, 0, len(state.Players))
	for _, playerID := range state.Players {
		resource := state.Turn.Resources[playerID]
		parts = append(parts, fmt.Sprintf("%s:%d/%d", playerID, resource.Current, resource.Max))
	}
	return strings.Join(parts, " ")
}

func traceBoardSummary(state rules.GameState) string {
	type zoneCounter struct {
		hand    int
		deck    int
		table   int
		asset   int
		discard int
		score   int
	}
	byPlayer := make(map[string]zoneCounter, len(state.Players))
	for _, playerID := range state.Players {
		byPlayer[playerID] = zoneCounter{}
	}

	regions := 0
	hiddenOnTable := 0
	for _, card := range state.Board.Cards {
		if card.Kind == rules.CardKindRegion && card.Zone == rules.CardZoneTable && !card.Destroyed {
			regions++
		}
		if card.Zone == rules.CardZoneTable && card.FaceDown && !card.Destroyed {
			hiddenOnTable++
		}
		counter := byPlayer[card.OwnerID]
		switch card.Zone {
		case rules.CardZoneHand:
			counter.hand++
		case rules.CardZoneDeck:
			counter.deck++
		case rules.CardZoneTable:
			counter.table++
		case rules.CardZoneAsset:
			counter.asset++
		case rules.CardZoneDiscard:
			counter.discard++
		case rules.CardZoneScore:
			counter.score++
		}
		byPlayer[card.OwnerID] = counter
	}

	parts := make([]string, 0, len(state.Players)+2)
	for _, playerID := range state.Players {
		counter := byPlayer[playerID]
		parts = append(parts, fmt.Sprintf("%s{hand=%d deck=%d table=%d asset=%d discard=%d score=%d}",
			playerID, counter.hand, counter.deck, counter.table, counter.asset, counter.discard, counter.score))
	}
	parts = append(parts, fmt.Sprintf("regions=%d", regions))
	parts = append(parts, fmt.Sprintf("hiddenTableCards=%d", hiddenOnTable))
	return strings.Join(parts, " ")
}

func latestReportPath(report *MatchReport) string {
	if report == nil {
		return ""
	}
	return report.Path
}

func formatActionKind(kind string) string {
	switch kind {
	case "play_card":
		return "打出卡牌"
	case "build_asset":
		return "构建资产"
	case "pass_priority":
		return "放弃优先权"
	case "declare_attack":
		return "发起攻击"
	case "declare_investigation":
		return "发起调查"
	case "reveal_card":
		return "揭示卡牌"
	case "inspect_card":
		return "检视卡牌"
	case "set_marker":
		return "设置标记物"
	case "remove_marker":
		return "移除标记物"
	case "set_face_down":
		return "设置面朝下"
	default:
		return kind
	}
}

func formatEventKind(kind string) string {
	switch kind {
	case "card_played":
		return "卡牌已打出"
	case "asset_built":
		return "资产已构建"
	case "priority_passed":
		return "优先权已传递"
	case "investigation_applied":
		return "调查已应用"
	case "card_revealed":
		return "卡牌已揭示"
	case "card_inspected":
		return "卡牌已检视"
	case "marker_set":
		return "标记物已设置"
	case "marker_removed":
		return "标记物已移除"
	case "face_down_set":
		return "已设置面朝下"
	default:
		return kind
	}
}

func renderGameStateSnapshot(state rules.GameState) string {
	var builder strings.Builder
	builder.WriteString("### 游戏状态\n\n")
	builder.WriteString(fmt.Sprintf("- 回合: %d\n", state.Turn.TurnNumber))
	builder.WriteString(fmt.Sprintf("- 阶段: %s/%s\n", state.Turn.Phase.Name, state.Turn.Phase.Step))
	builder.WriteString(fmt.Sprintf("- 当前玩家: %s\n", emptyAsDash(state.Turn.ActivePlayerID)))
	builder.WriteString(fmt.Sprintf("- 优先权: %s\n", emptyAsDash(state.Turn.Priority.CurrentPlayerID)))
	builder.WriteString(fmt.Sprintf("- 分数: %s\n", formatScore(state)))
	builder.WriteString(fmt.Sprintf("- 资源: %s\n", traceResourceSummary(state)))
	builder.WriteString(fmt.Sprintf("- 战场: %s\n", traceBoardSummary(state)))
	return builder.String()
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

func resolveSandboxPath(path string) string {
	trimmed := strings.TrimSpace(path)
	if trimmed == "" {
		return ""
	}
	if filepath.IsAbs(trimmed) {
		return trimmed
	}
	root, err := detectRepositoryRoot()
	if err != nil || root == "" {
		return trimmed
	}
	return filepath.Join(root, trimmed)
}

func detectRepositoryRoot() (string, error) {
	workingDirectory, err := os.Getwd()
	if err != nil {
		return "", err
	}
	workingDirectory, err = filepath.Abs(workingDirectory)
	if err != nil {
		return "", err
	}

	cursor := workingDirectory
	for {
		goModPath := filepath.Join(cursor, "go.mod")
		if _, err := os.Stat(goModPath); err == nil {
			return cursor, nil
		}
		parent := filepath.Dir(cursor)
		if parent == cursor {
			return "", fmt.Errorf("repository root not found")
		}
		cursor = parent
	}
}
