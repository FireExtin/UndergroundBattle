package rules

import (
	"encoding/json"
	"reflect"
	"strings"
	"testing"
)

// Purpose: Verifies player-specific projection generation and hidden-information isolation.

func TestOwnHiddenCardVisibleButOpponentCannotSeeIt(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-projection-own-hidden",
		ActivePlayerID: "P1",
		Seed:           1,
	})
	state.Board.Cards = []CardState{
		{
			CardID:         "card-1",
			Name:           "Secret Archive",
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Revealed:       false,
		},
	}

	engine := NewProjectionEngine()
	views := engine.Generate(state)

	selfCard := onlyCard(t, views.Players["P1"])
	if selfCard.Name != "Secret Archive" || selfCard.CardID != "card-1" {
		t.Fatalf("self card view = %#v, want visible card identity", selfCard)
	}

	opponentCard := onlyCard(t, views.Players["P2"])
	if opponentCard.Name != "" || opponentCard.CardID != "" {
		t.Fatalf("opponent card view leaked hidden info: %#v", opponentCard)
	}

	if reflect.DeepEqual(views.Players["P1"], views.Players["P2"]) {
		t.Fatal("expected player views to differ for hidden information")
	}
}

func TestRevealedCardBecomesPublicAfterNextCommit(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-projection-reveal",
		ActivePlayerID: "P1",
		Seed:           2,
	})
	state.Board.Cards = []CardState{
		{
			CardID:         "card-2",
			Name:           "Open Sigil",
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Revealed:       false,
		},
	}

	engine := NewProjectionEngine()
	result, err := SubmitActionWithProjection(state, Action{
		ID:      "act-reveal-1",
		ActorID: "P1",
		Kind:    ActionKindRevealCard,
		CardID:  "card-2",
	}, engine)
	if err != nil {
		t.Fatalf("SubmitActionWithProjection returned error: %v", err)
	}

	if result.Event.Kind != EventKindCardRevealed {
		t.Fatalf("event kind = %q, want %q", result.Event.Kind, EventKindCardRevealed)
	}

	left := onlyCard(t, result.Views.Players["P1"])
	right := onlyCard(t, result.Views.Players["P2"])
	if left.Name != "Open Sigil" || right.Name != "Open Sigil" {
		t.Fatalf("revealed card must be visible to both players: left=%#v right=%#v", left, right)
	}
}

func TestInspectedCardVisibleOnlyToInspectingPlayer(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-projection-inspect",
		ActivePlayerID: "P1",
		Seed:           3,
	})
	state.Board.Cards = []CardState{
		{
			CardID:         "card-3",
			Name:           "Buried Omen",
			OwnerID:        "P2",
			Zone:           CardZoneDeck,
			VisibleToOwner: false,
			Revealed:       false,
		},
	}

	engine := NewProjectionEngine()
	result, err := SubmitActionWithProjection(state, Action{
		ID:      "act-inspect-1",
		ActorID: "P1",
		Kind:    ActionKindInspectCard,
		CardID:  "card-3",
	}, engine)
	if err != nil {
		t.Fatalf("SubmitActionWithProjection returned error: %v", err)
	}

	inspectorCard := onlyCard(t, result.Views.Players["P1"])
	if inspectorCard.Name != "Buried Omen" || inspectorCard.CardID != "card-3" {
		t.Fatalf("inspector view = %#v, want visible card identity", inspectorCard)
	}

	opponentCard := onlyCard(t, result.Views.Players["P2"])
	if opponentCard.Name != "" || opponentCard.CardID != "" {
		t.Fatalf("opponent view leaked inspected-only info: %#v", opponentCard)
	}
}

func TestSpectatorViewDoesNotExposeHiddenInformation(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-projection-spectator",
		ActivePlayerID: "P1",
		Seed:           4,
	})
	state.Board.Cards = []CardState{
		{
			CardID:         "card-4",
			Name:           "Hidden Gospel",
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Revealed:       false,
		},
	}

	views := NewProjectionEngine().Generate(state)
	card := onlySpectatorCard(t, views.Spectator)
	if card.Name != "" || card.CardID != "" {
		t.Fatalf("spectator view leaked hidden info: %#v", card)
	}

	payload, err := json.Marshal(views.Spectator)
	if err != nil {
		t.Fatalf("json.Marshal returned error: %v", err)
	}

	if strings.Contains(string(payload), "Hidden Gospel") {
		t.Fatalf("spectator JSON leaked hidden card name: %s", string(payload))
	}
}

func TestProjectionGenerationDoesNotBreakRevisionOrReplay(t *testing.T) {
	initial := NewGameState(InitialStateConfig{
		GameID:         "game-projection-replay",
		ActivePlayerID: "P1",
		Seed:           5,
	})
	initial.Board.Cards = []CardState{
		{
			CardID:         "card-5",
			Name:           "Mirror Doctrine",
			OwnerID:        "P2",
			Zone:           CardZoneDeck,
			VisibleToOwner: false,
			Revealed:       false,
		},
	}
	state := cloneGameState(initial)

	engine := NewProjectionEngine()
	actions := []Action{
		{
			ID:      "act-proj-1",
			ActorID: "P1",
			Kind:    ActionKindInspectCard,
			CardID:  "card-5",
		},
		{
			ID:      "act-proj-2",
			ActorID: "P1",
			Kind:    ActionKindRevealCard,
			CardID:  "card-5",
		},
	}

	for _, action := range actions {
		result, err := SubmitActionWithProjection(state, action, engine)
		if err != nil {
			t.Fatalf("SubmitActionWithProjection(%q) returned error: %v", action.ID, err)
		}

		state = result.State
	}

	if state.Revision.Number != 2 {
		t.Fatalf("revision number = %d, want 2", state.Revision.Number)
	}

	replayed, err := ReplayActions(cloneGameState(initial), state.History.Actions)
	if err != nil {
		t.Fatalf("ReplayActions returned error: %v", err)
	}

	if state.Revision.Number != 2 {
		t.Fatalf("revision changed unexpectedly after projection generation: %d", state.Revision.Number)
	}

	if engine.GenerationCount() != 2 {
		t.Fatalf("projection generation count = %d, want 2", engine.GenerationCount())
	}

	if !reflect.DeepEqual(state, replayed) {
		t.Fatalf("projection generation changed replayed state\nstate   = %#v\nreplayed = %#v", state, replayed)
	}
}

func TestProjectionCarriesPublicScoreAndWinner(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-projection-score",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           7,
	})
	state.Score.ByPlayer["P1"] = 2
	state.Score.ByPlayer["P2"] = 1
	state.Score.WinnerPlayerID = "P1"

	views := NewProjectionEngine().Generate(state)

	if views.Players["P1"].Score.ByPlayer["P1"] != 2 {
		t.Fatalf("P1 projected score = %d, want 2", views.Players["P1"].Score.ByPlayer["P1"])
	}
	if views.Players["P2"].Score.ByPlayer["P2"] != 1 {
		t.Fatalf("P2 projected score = %d, want 1", views.Players["P2"].Score.ByPlayer["P2"])
	}
	if views.Spectator.Score.WinnerPlayerID != "P1" {
		t.Fatalf("spectator winner = %q, want %q", views.Spectator.Score.WinnerPlayerID, "P1")
	}
}

func TestProjectionRunsOnlyAfterCommitAndNotDuringLegality(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-projection-timing",
		ActivePlayerID: "P1",
		Seed:           6,
	})
	engine := NewProjectionEngine()

	legality := CheckLegality(state, Action{
		ID:      "act-proj-timing-illegal",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	})
	if legality.OK {
		t.Fatal("expected legality to fail")
	}

	if engine.GenerationCount() != 0 {
		t.Fatalf("projection generation count = %d, want 0 before submit", engine.GenerationCount())
	}

	_, err := SubmitActionWithProjection(state, Action{
		ID:      "act-proj-timing-illegal",
		ActorID: "P1",
		Kind:    ActionKindResolveTopStack,
	}, engine)
	if err == nil {
		t.Fatal("expected illegal submit to fail")
	}

	if engine.GenerationCount() != 0 {
		t.Fatalf("projection generation count = %d, want 0 after illegal submit", engine.GenerationCount())
	}

	result, err := SubmitActionWithProjection(state, Action{
		ID:      "act-proj-timing-ok",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	}, engine)
	if err != nil {
		t.Fatalf("SubmitActionWithProjection returned error: %v", err)
	}

	if engine.GenerationCount() != 1 {
		t.Fatalf("projection generation count = %d, want 1 after commit", engine.GenerationCount())
	}

	if result.Views.Revision.Number != result.Revision.Number {
		t.Fatalf("projection revision = %d, want %d", result.Views.Revision.Number, result.Revision.Number)
	}
}

func onlyCard(t *testing.T, view PlayerViewState) CardView {
	t.Helper()

	if len(view.Board.Cards) != 1 {
		t.Fatalf("card count = %d, want 1", len(view.Board.Cards))
	}

	return view.Board.Cards[0]
}

func onlySpectatorCard(t *testing.T, view SpectatorViewState) CardView {
	t.Helper()

	if len(view.Board.Cards) != 1 {
		t.Fatalf("card count = %d, want 1", len(view.Board.Cards))
	}

	return view.Board.Cards[0]
}
