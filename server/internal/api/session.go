package api

import (
	"encoding/json"
	"errors"
	"fmt"
	"sync"

	"undergroundbattle/server/pkg/rules"
)

// Purpose: Holds one in-memory sandbox match session and converts authoritative dispatch outputs into protocol envelopes for the web debugger.

const protocolVersion = "0.1.0"

type protocolEnvelope struct {
	Version   string          `json:"version"`
	Kind      string          `json:"kind"`
	MessageID string          `json:"messageId"`
	Name      string          `json:"name"`
	Revision  *int            `json:"revision,omitempty"`
	Payload   json.RawMessage `json:"payload"`
}

type SandboxSession struct {
	mu                sync.Mutex
	state             rules.GameState
	projector         *rules.ProjectionEngine
	messages          []protocolEnvelope
	nextMessageNumber int
}

func NewSandboxSession() *SandboxSession {
	session := &SandboxSession{
		nextMessageNumber: 1,
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

	result, err := rules.SubmitActionWithProjection(session.state, action, session.projector)
	if err != nil {
		var legality *rules.LegalityError
		if errors.As(err, &legality) {
			return session.appendDispatchBatch(rules.BuildRejectedDispatchBatch(action, legality.Result))
		}

		return nil, err
	}

	session.state = result.State
	return session.appendDispatchBatch(result.Dispatch)
}

func (session *SandboxSession) Reset() ([]protocolEnvelope, error) {
	session.mu.Lock()
	defer session.mu.Unlock()

	return session.resetLocked()
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
	session.nextMessageNumber = 1
	messages, err := session.materializeBootstrapMessages(views)
	if err != nil {
		return nil, err
	}

	session.state = state
	session.projector = projector
	session.messages = cloneProtocolEnvelopes(messages)
	session.nextMessageNumber = len(messages) + 1
	return cloneProtocolEnvelopes(messages), nil
}
