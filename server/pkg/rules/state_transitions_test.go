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
