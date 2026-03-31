package rules

import "slices"

// Purpose: Defines the canonical M0 sandbox initial state shared by the rules regression harness and the live HTTP sandbox.

const M0SandboxInitialStateID = "m0_sandbox_v1"

// NewM0SandboxState returns the single canonical engine-owned initial state for the live sandbox and M0 regression scenarios.
func NewM0SandboxState() GameState {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-sandbox-live",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
		Seed:           42,
	})

	state.Board.Cards = []CardState{
		{
			CardID:         "P1-HAND-SECRET",
			Name:           "Secret Archive",
			Kind:           CardKindUnknown,
			OwnerID:        "P1",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Revealed:       false,
		},
		{
			CardID:         "P1-TABLE-1",
			Name:           "Dream Sentinel",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats: CardNumericStats{
				Combat:        2,
				Defense:       2,
				Influence:     0,
				Investigation: 1,
			},
			EffectiveStats: CardNumericStats{
				Combat:        2,
				Defense:       2,
				Influence:     0,
				Investigation: 1,
			},
		},
		{
			CardID:         "P2-HAND-SECRET",
			Name:           "Black Ledger",
			Kind:           CardKindUnknown,
			OwnerID:        "P2",
			Zone:           CardZoneHand,
			VisibleToOwner: true,
			Revealed:       false,
		},
		{
			CardID:            "P2-TABLE-1",
			Name:              "Frontline Adept",
			Kind:              CardKindCharacter,
			OwnerID:           "P2",
			Zone:              CardZoneTable,
			VisibleToOwner:    true,
			Revealed:          true,
			PrintedKeywords:   []string{"blackBlade"},
			EffectiveKeywords: []string{"blackBlade"},
			PrintedStats: CardNumericStats{
				Combat:        2,
				Defense:       3,
				Influence:     0,
				Investigation: 1,
			},
			EffectiveStats: CardNumericStats{
				Combat:        2,
				Defense:       3,
				Influence:     0,
				Investigation: 1,
			},
			Counters: CardCounters{
				Damage:    1,
				Influence: 0,
			},
		},
	}
	state.Board.Continuous.Active = slices.Clone(state.Board.Continuous.Active)
	return state
}
