package rules

import (
	"encoding/json"
	"strings"
	"testing"
)

// Purpose: Verifies per-client dispatch batches are built from projection outputs without leaking cross-audience hidden information.

func TestCommitDispatchBatchBuildsPerClientMessages(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-dispatch-commit",
		ActivePlayerID: "P1",
		Seed:           1,
	})
	state.Board.Cards = []CardState{
		{
			CardID:         "card-dispatch-1",
			Name:           "Veiled Archive",
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Revealed:       false,
		},
	}

	result, err := SubmitActionWithProjection(state, Action{
		ID:      "act-dispatch-1",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	}, NewProjectionEngine())
	if err != nil {
		t.Fatalf("SubmitActionWithProjection returned error: %v", err)
	}

	if len(result.Dispatch.Messages) != 4 {
		t.Fatalf("dispatch message count = %d, want 4", len(result.Dispatch.Messages))
	}

	accepted := findDispatchMessage(t, result.Dispatch, DispatchPayloadActionAccepted, DispatchAudiencePlayer, "P1")
	if accepted.ActionAccepted == nil {
		t.Fatal("expected ActionAccepted payload for actor")
	}

	if accepted.ActionAccepted.Revision.Number != result.Revision.Number {
		t.Fatalf("accepted revision = %d, want %d", accepted.ActionAccepted.Revision.Number, result.Revision.Number)
	}

	selfPatch := findDispatchMessage(t, result.Dispatch, DispatchPayloadStatePatched, DispatchAudiencePlayer, "P1")
	selfCard := onlyPatchedPlayerCard(t, selfPatch)
	if selfCard.Name != "Veiled Archive" || selfCard.CardID != "card-dispatch-1" {
		t.Fatalf("self patch card = %#v, want visible hidden card", selfCard)
	}

	opponentPatch := findDispatchMessage(t, result.Dispatch, DispatchPayloadStatePatched, DispatchAudiencePlayer, "P2")
	opponentCard := onlyPatchedPlayerCard(t, opponentPatch)
	if opponentCard.Name != "" || opponentCard.CardID != "" {
		t.Fatalf("opponent patch leaked hidden info: %#v", opponentCard)
	}

	spectatorPatch := findDispatchMessage(t, result.Dispatch, DispatchPayloadStatePatched, DispatchAudienceSpectator, "")
	spectatorCard := onlyPatchedSpectatorCard(t, spectatorPatch)
	if spectatorCard.Name != "" || spectatorCard.CardID != "" {
		t.Fatalf("spectator patch leaked hidden info: %#v", spectatorCard)
	}
}

func TestRejectedDispatchBatchTargetsOnlyActor(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-dispatch-reject",
		ActivePlayerID: "P1",
		Seed:           2,
	})

	action := Action{
		ID:      "act-dispatch-reject-1",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	}
	legality := CheckLegality(state, action)
	if legality.OK {
		t.Fatal("expected legality failure")
	}

	batch := BuildRejectedDispatchBatch(action, legality)
	if len(batch.Messages) != 1 {
		t.Fatalf("dispatch message count = %d, want 1", len(batch.Messages))
	}

	message := batch.Messages[0]
	if message.Kind != DispatchPayloadActionRejected {
		t.Fatalf("dispatch kind = %q, want %q", message.Kind, DispatchPayloadActionRejected)
	}

	if message.Target.Kind != DispatchAudiencePlayer || message.Target.ID != "P1" {
		t.Fatalf("dispatch target = %#v, want player P1", message.Target)
	}

	if message.ActionRejected == nil {
		t.Fatal("expected ActionRejected payload")
	}

	if message.ActionRejected.Legality.ReasonCode != ReasonCodeStackFailedEmpty {
		t.Fatalf("reason code = %q, want %q", message.ActionRejected.Legality.ReasonCode, ReasonCodeStackFailedEmpty)
	}
}

func TestPerClientDispatchEnvelopeDoesNotLeakHiddenInfoToWrongAudience(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-dispatch-hidden",
		ActivePlayerID: "P1",
		Seed:           3,
	})
	state.Board.Cards = []CardState{
		{
			CardID:         "card-dispatch-2",
			Name:           "Black Ledger",
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Revealed:       false,
		},
	}

	result, err := SubmitActionWithProjection(state, Action{
		ID:      "act-dispatch-hidden-1",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	}, NewProjectionEngine())
	if err != nil {
		t.Fatalf("SubmitActionWithProjection returned error: %v", err)
	}

	opponentPatch := findDispatchMessage(t, result.Dispatch, DispatchPayloadStatePatched, DispatchAudiencePlayer, "P2")
	opponentPayload, err := json.Marshal(opponentPatch)
	if err != nil {
		t.Fatalf("json.Marshal(opponentPatch) returned error: %v", err)
	}

	if strings.Contains(string(opponentPayload), "Black Ledger") {
		t.Fatalf("opponent envelope leaked hidden card name: %s", string(opponentPayload))
	}

	spectatorPatch := findDispatchMessage(t, result.Dispatch, DispatchPayloadStatePatched, DispatchAudienceSpectator, "")
	spectatorPayload, err := json.Marshal(spectatorPatch)
	if err != nil {
		t.Fatalf("json.Marshal(spectatorPatch) returned error: %v", err)
	}

	if strings.Contains(string(spectatorPayload), "Black Ledger") {
		t.Fatalf("spectator envelope leaked hidden card name: %s", string(spectatorPayload))
	}
}

func findDispatchMessage(
	t *testing.T,
	batch DispatchBatch,
	kind DispatchPayloadKind,
	audienceKind DispatchAudienceKind,
	audienceID string,
) ClientDispatch {
	t.Helper()

	for _, message := range batch.Messages {
		if message.Kind == kind && message.Target.Kind == audienceKind && message.Target.ID == audienceID {
			return message
		}
	}

	t.Fatalf("dispatch message not found: kind=%q audienceKind=%q audienceID=%q", kind, audienceKind, audienceID)
	return ClientDispatch{}
}

func onlyPatchedPlayerCard(t *testing.T, message ClientDispatch) CardView {
	t.Helper()

	if message.StatePatched == nil || message.StatePatched.PlayerView == nil {
		t.Fatal("expected player StatePatched payload")
	}

	cards := message.StatePatched.PlayerView.Board.Cards
	if len(cards) != 1 {
		t.Fatalf("player patch card count = %d, want 1", len(cards))
	}

	return cards[0]
}

func onlyPatchedSpectatorCard(t *testing.T, message ClientDispatch) CardView {
	t.Helper()

	if message.StatePatched == nil || message.StatePatched.SpectatorView == nil {
		t.Fatal("expected spectator StatePatched payload")
	}

	cards := message.StatePatched.SpectatorView.Board.Cards
	if len(cards) != 1 {
		t.Fatalf("spectator patch card count = %d, want 1", len(cards))
	}

	return cards[0]
}
