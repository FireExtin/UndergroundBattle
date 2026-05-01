package rules

import "testing"

func TestMoveCardToDiscardTransition(t *testing.T) {
	card := CardState{
		CardID:   "card-discard-transition-1",
		Zone:     CardZoneTable,
		Revealed: false,
	}

	moveCardToDiscard(&card)

	if card.Zone != CardZoneDiscard {
		t.Fatalf("card zone = %q, want %q", card.Zone, CardZoneDiscard)
	}
	if !card.Destroyed {
		t.Fatal("card destroyed = false, want true")
	}
	if !card.Revealed {
		t.Fatal("card revealed = false, want true")
	}
}

func TestRevealFaceDownTransition(t *testing.T) {
	card := CardState{
		CardID:   "card-reveal-transition-1",
		FaceDown: true,
		Revealed: false,
	}

	revealFaceDown(&card)

	if card.FaceDown {
		t.Fatal("card faceDown = true, want false")
	}
	if !card.Revealed {
		t.Fatal("card revealed = false, want true")
	}
}

func TestMarkCardInspectedTransitionDeduplicatesInspector(t *testing.T) {
	card := CardState{
		CardID:      "card-inspect-transition-1",
		InspectedBy: []string{"P1"},
	}

	markCardInspected(&card, "P2")
	markCardInspected(&card, "P2")
	markCardInspected(&card, "")
	markCardInspected(nil, "P3")

	if len(card.InspectedBy) != 2 {
		t.Fatalf("inspectedBy len = %d, want 2", len(card.InspectedBy))
	}
	if !containsString(card.InspectedBy, "P1") || !containsString(card.InspectedBy, "P2") {
		t.Fatalf("inspectedBy = %#v, want [P1 P2]", card.InspectedBy)
	}
}

func TestAppendGeneratedDrawCardTransition(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "state-transition-draw",
		ActivePlayerID: "P1",
		Seed:           53,
	})

	appendGeneratedDrawCard(&state, "op:draw-transition", "P1", 2)

	if len(state.Board.Cards) != 1 {
		t.Fatalf("cards count = %d, want 1", len(state.Board.Cards))
	}

	card := state.Board.Cards[0]
	if card.CardID != "draw:op:draw-transition:2" {
		t.Fatalf("cardId = %q, want %q", card.CardID, "draw:op:draw-transition:2")
	}
	if card.OwnerID != "P1" || card.Zone != CardZoneHand {
		t.Fatalf("draw card owner/zone = (%q,%q), want (%q,%q)", card.OwnerID, card.Zone, "P1", CardZoneHand)
	}
	if !card.VisibleToOwner || card.Revealed || card.Exhausted {
		t.Fatalf("draw card visibility flags = %#v, expected visibleToOwner=true/revealed=false/exhausted=false", card)
	}
}

func TestAppendGeneratedDrawCardTransitionRejectsInvalidInput(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "state-transition-draw-guard",
		ActivePlayerID: "P1",
		Seed:           54,
	})

	appendGeneratedDrawCard(&state, "", "P1", 1)
	appendGeneratedDrawCard(&state, "op:draw-transition", "", 1)
	appendGeneratedDrawCard(&state, "op:draw-transition", "P1", 0)
	appendGeneratedDrawCard(nil, "op:draw-transition", "P1", 1)

	if len(state.Board.Cards) != 0 {
		t.Fatalf("cards count = %d, want 0 for invalid input", len(state.Board.Cards))
	}
}

func TestAppendRandomResultTransition(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "state-transition-random",
		ActivePlayerID: "P1",
		Seed:           55,
	})

	appendRandomResult(&state, RandomResult{
		ActionID:    "act-random-transition",
		OperationID: "op-random-transition",
		DrawIndex:   3,
		Value:       6,
	})

	if len(state.Board.RandomResults) != 1 {
		t.Fatalf("random result count = %d, want 1", len(state.Board.RandomResults))
	}
	got := state.Board.RandomResults[0]
	if got.ActionID != "act-random-transition" || got.OperationID != "op-random-transition" || got.DrawIndex != 3 || got.Value != 6 {
		t.Fatalf("random result = %#v, want action/op/draw/value preserved", got)
	}
}

func TestAppendResolvedOperationTransitionClonesOperation(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "state-transition-resolved",
		ActivePlayerID: "P1",
		Seed:           56,
	})
	source := &CardOperationSource{CardID: "CARD-1"}
	operation := Operation{
		ID:     "op-resolved-transition",
		CardID: "card-resolved-transition",
		Source: source,
	}

	appendResolvedOperation(&state, operation)

	if len(state.Board.Resolved) != 1 {
		t.Fatalf("resolved count = %d, want 1", len(state.Board.Resolved))
	}
	if state.Board.Resolved[0].ID != operation.ID {
		t.Fatalf("resolved operation id = %q, want %q", state.Board.Resolved[0].ID, operation.ID)
	}

	operation.ID = "mutated-op"
	if state.Board.Resolved[0].ID != "op-resolved-transition" {
		t.Fatalf("resolved operation should be detached from caller mutation, got %q", state.Board.Resolved[0].ID)
	}

	source.CardID = "MUTATED-CARD"
	if state.Board.Resolved[0].Source == nil || state.Board.Resolved[0].Source.CardID != "CARD-1" {
		t.Fatalf("resolved source should be deep-cloned, got %#v", state.Board.Resolved[0].Source)
	}
}

func TestNextRevisionForCommitTransition(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "state-transition-revision",
		ActivePlayerID: "P1",
		Seed:           57,
	})
	state.Revision.Number = 4

	revision := nextRevisionForCommit(state, Action{ID: "act-5"}, Operation{ID: "op-5"}, Event{ID: "evt-5"})

	if revision.Number != 5 {
		t.Fatalf("revision number = %d, want 5", revision.Number)
	}
	if revision.ActionID != "act-5" || revision.OperationID != "op-5" || revision.EventID != "evt-5" {
		t.Fatalf("revision linkage = %#v, want act/op/event ids preserved", revision)
	}
}

func TestAppendCommitHistoryTransitionClonesOperationAndEvent(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "state-transition-history",
		ActivePlayerID: "P1",
		Seed:           58,
	})
	source := &CardOperationSource{CardID: "CARD-1"}
	operation := Operation{
		ID:      "op-history-1",
		Source:  source,
		Payment: &PaymentRecord{Kind: PaymentKindMarker, MarkerType: markerTypeResource, Amount: 1},
	}
	event := Event{
		ID:      "evt-history-1",
		Payment: &PaymentRecord{Kind: PaymentKindMarker, MarkerType: markerTypeResource, Amount: 1},
	}
	revision := Revision{Number: 1, ActionID: "act-history-1", OperationID: operation.ID, EventID: event.ID}

	appendCommitHistory(&state, Action{ID: "act-history-1"}, operation, event, revision)

	if len(state.History.Actions) != 1 || state.History.Actions[0].ID != "act-history-1" {
		t.Fatalf("history actions = %#v, want committed action", state.History.Actions)
	}
	if len(state.History.Operations) != 1 || state.History.Operations[0].ID != operation.ID {
		t.Fatalf("history operations = %#v, want committed operation", state.History.Operations)
	}
	if len(state.History.Events) != 1 || state.History.Events[0].ID != event.ID {
		t.Fatalf("history events = %#v, want committed event", state.History.Events)
	}
	if len(state.History.Revisions) != 1 || state.History.Revisions[0].Number != revision.Number {
		t.Fatalf("history revisions = %#v, want committed revision", state.History.Revisions)
	}
	if state.Revision.Number != revision.Number {
		t.Fatalf("state revision = %d, want %d", state.Revision.Number, revision.Number)
	}

	operation.ID = "mutated-op"
	source.CardID = "mutated-card"
	event.ID = "mutated-evt"
	operation.Payment.MarkerType = "mutated-payment"
	event.Payment.MarkerType = "mutated-payment"
	if state.History.Operations[0].ID != "op-history-1" {
		t.Fatalf("history operation should be detached from caller mutation, got %q", state.History.Operations[0].ID)
	}
	if state.History.Operations[0].Source == nil || state.History.Operations[0].Source.CardID != "CARD-1" {
		t.Fatalf("history operation source should be deep-cloned, got %#v", state.History.Operations[0].Source)
	}
	if state.History.Operations[0].Payment == nil || state.History.Operations[0].Payment.MarkerType != markerTypeResource {
		t.Fatalf("history operation payment should be deep-cloned, got %#v", state.History.Operations[0].Payment)
	}
	if state.History.Events[0].ID != "evt-history-1" {
		t.Fatalf("history event should be detached from caller mutation, got %q", state.History.Events[0].ID)
	}
	if state.History.Events[0].Payment == nil || state.History.Events[0].Payment.MarkerType != markerTypeResource {
		t.Fatalf("history event payment should be deep-cloned, got %#v", state.History.Events[0].Payment)
	}
}

func TestStampFinishedMatchRevisionTransition(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "state-transition-match-finish",
		ActivePlayerID: "P1",
		Seed:           59,
	})
	state.Match.Status = MatchStatusFinished

	stampFinishedMatchRevision(&state, Revision{Number: 7})
	if state.Match.FinishedAtRevision != 7 {
		t.Fatalf("finishedAtRevision = %d, want 7", state.Match.FinishedAtRevision)
	}

	stampFinishedMatchRevision(&state, Revision{Number: 9})
	if state.Match.FinishedAtRevision != 7 {
		t.Fatalf("finishedAtRevision should stay 7 once stamped, got %d", state.Match.FinishedAtRevision)
	}
}

func TestAttachToHostTransitionCreatesAttachment(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "state-transition-attach",
		ActivePlayerID: "P1",
		Seed:           51,
	})
	state.Board.Cards = []CardState{
		{
			CardID:         "attach-source-1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
		{
			CardID:         "attach-host-1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
		},
	}

	next, attachmentID, err := attachToHost(state, attachmentTransitionSpec{
		SourceCardID:       "attach-source-1",
		SourceDefinitionID: "BQ022",
		SourceOperationID:  "op:attach-state-transition-1",
		TargetCardID:       "attach-host-1",
		HostCardID:         "attach-host-1",
		Revision:           3,
		BasicType:          "附属",
	})
	if err != nil {
		t.Fatalf("attachToHost returned error: %v", err)
	}

	if attachmentID == "" {
		t.Fatal("attachmentID is empty")
	}
	if len(next.Board.Attachments.Active) != 1 {
		t.Fatalf("attachments count = %d, want 1", len(next.Board.Attachments.Active))
	}
	if next.Board.Attachments.Active[0].ID != attachmentID {
		t.Fatalf("attachment id = %q, want %q", next.Board.Attachments.Active[0].ID, attachmentID)
	}
}

func TestMarkerTransitions(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "state-transition-marker",
		ActivePlayerID: "P1",
		Seed:           52,
	})

	setMarker(&state, "P1", "secret_society", 2)
	if got := state.Board.Markers.GetMarker("P1", "secret_society"); got != 2 {
		t.Fatalf("setMarker result = %d, want 2", got)
	}

	addMarkerCount(&state, "P1", "secret_society", 3)
	if got := state.Board.Markers.GetMarker("P1", "secret_society"); got != 5 {
		t.Fatalf("addMarkerCount result = %d, want 5", got)
	}

	removeMarkerCount(&state, "P1", "secret_society", 4)
	if got := state.Board.Markers.GetMarker("P1", "secret_society"); got != 1 {
		t.Fatalf("removeMarkerCount result = %d, want 1", got)
	}
}

func TestShieldCounterTransitions(t *testing.T) {
	card := CardState{
		CardID: "card-shield-transition-1",
	}

	addShieldCounter(&card, 2)
	if card.Counters.Shield != 2 {
		t.Fatalf("shield after add = %d, want 2", card.Counters.Shield)
	}

	consumed := consumeShieldCounter(&card, 1)
	if !consumed {
		t.Fatal("consumeShieldCounter should succeed when enough shield exists")
	}
	if card.Counters.Shield != 1 {
		t.Fatalf("shield after consume = %d, want 1", card.Counters.Shield)
	}

	consumed = consumeShieldCounter(&card, 2)
	if consumed {
		t.Fatal("consumeShieldCounter should fail when shield is insufficient")
	}
	if card.Counters.Shield != 1 {
		t.Fatalf("shield should stay unchanged on failed consume, got %d", card.Counters.Shield)
	}

	addShieldCounter(nil, 1)
	if consumeShieldCounter(nil, 1) {
		t.Fatal("consumeShieldCounter(nil) should return false")
	}
}
