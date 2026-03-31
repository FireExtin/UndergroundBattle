package api

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"undergroundbattle/server/pkg/rules"
)

// Purpose: Verifies the minimal HTTP sandbox exposes projected protocol envelopes and action submission without leaking FullState.

type statePatchedPayload struct {
	AudienceKind  string `json:"audienceKind"`
	AudienceID    string `json:"audienceId,omitempty"`
	PlayerView    *struct {
		Board struct {
			Cards []struct {
				CardID     string `json:"cardId,omitempty"`
				Name       string `json:"name,omitempty"`
				OwnerID    string `json:"ownerId"`
				Visibility string `json:"visibility"`
			} `json:"cards"`
		} `json:"board"`
	} `json:"playerView,omitempty"`
	SpectatorView *struct {
		Board struct {
			Cards []struct {
				CardID     string `json:"cardId,omitempty"`
				Name       string `json:"name,omitempty"`
				OwnerID    string `json:"ownerId"`
				Visibility string `json:"visibility"`
			} `json:"cards"`
		} `json:"board"`
	} `json:"spectatorView,omitempty"`
}

type actionRejectedPayload struct {
	Legality struct {
		ReasonCode string `json:"reasonCode"`
		MessageKey string `json:"messageKey"`
		Hook       string `json:"hook"`
	} `json:"legality"`
}

func TestHandlerBootstrapsProjectedMessages(t *testing.T) {
	session := NewSandboxSession()
	handler := NewHandler(session, "")

	response := performRequest(t, handler, http.MethodGet, "/api/debugger/messages", nil)
	if response.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", response.Code, http.StatusOK)
	}

	var messages []protocolEnvelope
	decodeResponseJSON(t, response, &messages)

	if len(messages) != 3 {
		t.Fatalf("message count = %d, want 3", len(messages))
	}

	p1Patch := findEnvelopeByAudience(t, messages, "player", "P1")
	p2Patch := findEnvelopeByAudience(t, messages, "player", "P2")
	spectatorPatch := findEnvelopeByAudience(t, messages, "spectator", "")

	if p1Patch.Kind != "view" || p1Patch.Name != "StatePatched" {
		t.Fatalf("P1 patch envelope = %#v, want StatePatched view", p1Patch)
	}

	if p1Patch.Revision == nil || *p1Patch.Revision != 0 {
		t.Fatalf("P1 patch revision = %#v, want 0", p1Patch.Revision)
	}

	p1Cards := decodeStatePatchedPayload(t, p1Patch).PlayerView.Board.Cards
	if p1Cards[0].CardID != "P1-HAND-SECRET" || p1Cards[0].Name != "Secret Archive" {
		t.Fatalf("P1 first card = %#v, want visible own hidden card", p1Cards[0])
	}

	p2Cards := decodeStatePatchedPayload(t, p2Patch).PlayerView.Board.Cards
	if p2Cards[0].CardID != "" || p2Cards[0].Name != "" || p2Cards[0].Visibility != "hidden" {
		t.Fatalf("P2 first card = %#v, want hidden opponent card", p2Cards[0])
	}

	spectatorCards := decodeStatePatchedPayload(t, spectatorPatch).SpectatorView.Board.Cards
	if spectatorCards[0].CardID != "" || spectatorCards[0].Name != "" || spectatorCards[0].Visibility != "hidden" {
		t.Fatalf("spectator first card = %#v, want hidden card", spectatorCards[0])
	}
}

func TestHandlerSubmitsActionsAndAppendsDispatchMessages(t *testing.T) {
	session := NewSandboxSession()
	handler := NewHandler(session, "")

	response := performRequest(t, handler, http.MethodPost, "/api/debugger/actions", rules.Action{
		ID:             "act-http-1",
		ActorID:        "P1",
		Kind:           rules.ActionKindQueueOperation,
		CardID:         "BQ010",
		TargetPlayerID: "P2",
	})
	if response.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", response.Code, http.StatusOK)
	}

	var batch []protocolEnvelope
	decodeResponseJSON(t, response, &batch)

	if len(batch) != 4 {
		t.Fatalf("batch message count = %d, want 4", len(batch))
	}

	p1Patch := findEnvelopeByAudience(t, batch, "player", "P1")
	p1Cards := decodeStatePatchedPayload(t, p1Patch).PlayerView.Board.Cards
	if !containsVisibleCard(p1Cards, "Black Ledger") {
		t.Fatalf("P1 patch cards = %#v, want inspected opponent hand card to become visible", p1Cards)
	}

	historyResponse := performRequest(t, handler, http.MethodGet, "/api/debugger/messages", nil)
	var history []protocolEnvelope
	decodeResponseJSON(t, historyResponse, &history)

	if len(history) != 7 {
		t.Fatalf("history message count = %d, want 7", len(history))
	}
}

func TestHandlerReturnsStructuredRejectionEnvelope(t *testing.T) {
	session := NewSandboxSession()
	handler := NewHandler(session, "")

	response := performRequest(t, handler, http.MethodPost, "/api/debugger/actions", rules.Action{
		ID:      "act-http-reject-1",
		ActorID: "P2",
		Kind:    rules.ActionKindAdvancePhase,
	})
	if response.Code != http.StatusOK {
		t.Fatalf("status = %d, want %d", response.Code, http.StatusOK)
	}

	var batch []protocolEnvelope
	decodeResponseJSON(t, response, &batch)

	if len(batch) != 1 {
		t.Fatalf("batch message count = %d, want 1", len(batch))
	}

	if batch[0].Name != "ActionRejected" || batch[0].Kind != "event" {
		t.Fatalf("rejection envelope = %#v, want ActionRejected event", batch[0])
	}

	var payload actionRejectedPayload
	if err := json.Unmarshal(batch[0].Payload, &payload); err != nil {
		t.Fatalf("json.Unmarshal(rejection payload) returned error: %v", err)
	}

	if payload.Legality.ReasonCode != string(rules.ReasonCodeLegalityFailedNotYourPriority) {
		t.Fatalf("reasonCode = %q, want %q", payload.Legality.ReasonCode, rules.ReasonCodeLegalityFailedNotYourPriority)
	}
}

func performRequest(t *testing.T, handler http.Handler, method string, path string, body any) *httptest.ResponseRecorder {
	t.Helper()

	var requestBody *bytes.Reader
	if body == nil {
		requestBody = bytes.NewReader(nil)
	} else {
		data, err := json.Marshal(body)
		if err != nil {
			t.Fatalf("json.Marshal(body) returned error: %v", err)
		}
		requestBody = bytes.NewReader(data)
	}

	request := httptest.NewRequest(method, path, requestBody)
	if body != nil {
		request.Header.Set("Content-Type", "application/json")
	}

	response := httptest.NewRecorder()
	handler.ServeHTTP(response, request)
	return response
}

func decodeResponseJSON[T any](t *testing.T, response *httptest.ResponseRecorder, target *T) {
	t.Helper()

	if err := json.Unmarshal(response.Body.Bytes(), target); err != nil {
		t.Fatalf("json.Unmarshal(response) returned error: %v", err)
	}
}

func findEnvelopeByAudience(t *testing.T, messages []protocolEnvelope, audienceKind string, audienceID string) protocolEnvelope {
	t.Helper()

	for _, message := range messages {
		if message.Name != "StatePatched" {
			continue
		}

		payload := decodeStatePatchedPayload(t, message)
		if payload.AudienceKind == audienceKind && payload.AudienceID == audienceID {
			return message
		}
	}

	t.Fatalf("StatePatched envelope not found for audienceKind=%q audienceID=%q", audienceKind, audienceID)
	return protocolEnvelope{}
}

func decodeStatePatchedPayload(t *testing.T, message protocolEnvelope) statePatchedPayload {
	t.Helper()

	var payload statePatchedPayload
	if err := json.Unmarshal(message.Payload, &payload); err != nil {
		t.Fatalf("json.Unmarshal(StatePatched payload) returned error: %v", err)
	}

	return payload
}

func containsVisibleCard(cards []struct {
	CardID     string `json:"cardId,omitempty"`
	Name       string `json:"name,omitempty"`
	OwnerID    string `json:"ownerId"`
	Visibility string `json:"visibility"`
}, name string) bool {
	for _, card := range cards {
		if card.Name == name && card.Visibility == "visible" {
			return true
		}
	}

	return false
}
