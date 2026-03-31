package rules

import "slices"

// Purpose: Defines player-specific and spectator-specific projections derived from the authoritative FullState after commit.

// CardZone identifies the minimal location used by the projection rules.
type CardZone string

const (
	CardZoneDeck    CardZone = "deck"
	CardZoneHand    CardZone = "hand"
	CardZoneTable   CardZone = "table"
	CardZoneDiscard CardZone = "discard"
)

// CardKind identifies the minimal role a card plays in legality and board semantics.
type CardKind string

const (
	CardKindUnknown   CardKind = "unknown"
	CardKindCharacter CardKind = "character"
	CardKindRegion    CardKind = "region"
)

// CardState is the authoritative hidden-information record stored only in FullState.
type CardState struct {
	CardID              string           `json:"cardId"`
	Name                string           `json:"name"`
	Kind                CardKind         `json:"kind,omitempty"`
	OwnerID             string           `json:"ownerId"`
	Zone                CardZone         `json:"zone"`
	VisibleToOwner      bool             `json:"visibleToOwner"`
	Revealed            bool             `json:"revealed"`
	Exhausted           bool             `json:"exhausted"`
	InspectedBy         []string         `json:"inspectedBy,omitempty"`
	PrintedKeywords     []string         `json:"printedKeywords,omitempty"`
	EffectiveKeywords   []string         `json:"effectiveKeywords,omitempty"`
	PrintedStats        CardNumericStats `json:"printedStats"`
	EffectiveStats      CardNumericStats `json:"effectiveStats"`
	Counters            CardCounters     `json:"counters"`
	Permissions         []string         `json:"permissions,omitempty"`
	Prohibitions        []string         `json:"prohibitions,omitempty"`
	RequiredPermissions []string         `json:"requiredPermissions,omitempty"`
	CostAdjustment      int              `json:"costAdjustment,omitempty"`
	ActionQuota         int              `json:"actionQuota,omitempty"`
	Destroyed           bool             `json:"destroyed"`
}

// CardView is the client-safe projection of a single card record.
type CardView struct {
	CardID     string           `json:"cardId,omitempty"`
	Name       string           `json:"name,omitempty"`
	OwnerID    string           `json:"ownerId"`
	Zone       CardZone         `json:"zone"`
	Visibility string           `json:"visibility"`
	Revealed   bool             `json:"revealed"`
	Exhausted  bool             `json:"exhausted"`
	Destroyed  bool             `json:"destroyed"`
	Keywords   []string         `json:"keywords,omitempty"`
	Stats      CardNumericStats `json:"stats"`
	Counters   CardCounters     `json:"counters"`
}

// ViewBoardState is the client-safe projection of board information.
type ViewBoardState struct {
	Stack         []Operation    `json:"stack"`
	Resolved      []Operation    `json:"resolved"`
	RandomResults []RandomResult `json:"randomResults"`
	Cards         []CardView     `json:"cards"`
}

// PlayerViewState is the client-safe projection generated for one specific player.
type PlayerViewState struct {
	GameID         string         `json:"gameId"`
	ViewerPlayerID string         `json:"viewerPlayerId"`
	Revision       Revision       `json:"revision"`
	Turn           TurnState      `json:"turn"`
	Board          ViewBoardState `json:"board"`
}

// SpectatorViewState is the public-only projection generated for non-player observers.
type SpectatorViewState struct {
	GameID   string         `json:"gameId"`
	Revision Revision       `json:"revision"`
	Turn     TurnState      `json:"turn"`
	Board    ViewBoardState `json:"board"`
}

// ProjectionBundle collects all post-commit projections for a single revision.
type ProjectionBundle struct {
	Revision  Revision                   `json:"revision"`
	Players   map[string]PlayerViewState `json:"players"`
	Spectator SpectatorViewState         `json:"spectator"`
}

// ProjectionEngine derives client-safe views from FullState after a commit completes.
type ProjectionEngine struct {
	generationCount int
}

// NewProjectionEngine constructs a fresh projection engine for one rules session or one test harness.
func NewProjectionEngine() *ProjectionEngine {
	return &ProjectionEngine{}
}

// GenerationCount exposes how many post-commit projection batches the engine has produced.
func (engine *ProjectionEngine) GenerationCount() int {
	if engine == nil {
		return 0
	}

	return engine.generationCount
}

// Generate derives all player and spectator views from one committed FullState.
func (engine *ProjectionEngine) Generate(full FullState) ProjectionBundle {
	if engine != nil {
		engine.generationCount++
	}

	playerViews := make(map[string]PlayerViewState, len(full.Players))
	for _, playerID := range full.Players {
		playerViews[playerID] = PlayerViewState{
			GameID:         full.GameID,
			ViewerPlayerID: playerID,
			Revision:       full.Revision,
			Turn:           full.Turn,
			Board:          projectBoardForPlayer(full, playerID),
		}
	}

	return ProjectionBundle{
		Revision: full.Revision,
		Players:  playerViews,
		Spectator: SpectatorViewState{
			GameID:   full.GameID,
			Revision: full.Revision,
			Turn:     full.Turn,
			Board:    projectBoardForSpectator(full),
		},
	}
}

func projectBoardForPlayer(full FullState, viewerPlayerID string) ViewBoardState {
	view := baseBoardProjection(full)
	for _, card := range full.Board.Cards {
		view.Cards = append(view.Cards, projectCardForPlayer(card, viewerPlayerID))
	}

	return view
}

func projectBoardForSpectator(full FullState) ViewBoardState {
	view := baseBoardProjection(full)
	for _, card := range full.Board.Cards {
		view.Cards = append(view.Cards, projectCardForSpectator(card))
	}

	return view
}

func baseBoardProjection(full FullState) ViewBoardState {
	return ViewBoardState{
		Stack:         cloneOperations(full.Board.Stack),
		Resolved:      cloneOperations(full.Board.Resolved),
		RandomResults: slices.Clone(full.Board.RandomResults),
		Cards:         make([]CardView, 0, len(full.Board.Cards)),
	}
}

func projectCardForPlayer(card CardState, viewerPlayerID string) CardView {
	if cardVisibleToPlayer(card, viewerPlayerID) {
		return visibleCardView(card)
	}

	return hiddenCardView(card)
}

func projectCardForSpectator(card CardState) CardView {
	if card.Revealed {
		return visibleCardView(card)
	}

	return hiddenCardView(card)
}

func cardVisibleToPlayer(card CardState, viewerPlayerID string) bool {
	if card.Revealed {
		return true
	}

	if card.OwnerID == viewerPlayerID && card.VisibleToOwner {
		return true
	}

	return slices.Contains(card.InspectedBy, viewerPlayerID)
}

func visibleCardView(card CardState) CardView {
	return CardView{
		CardID:     card.CardID,
		Name:       card.Name,
		OwnerID:    card.OwnerID,
		Zone:       card.Zone,
		Visibility: "visible",
		Revealed:   card.Revealed,
		Exhausted:  card.Exhausted,
		Destroyed:  card.Destroyed,
		Keywords:   visibleKeywords(card),
		Stats:      visibleStats(card),
		Counters:   card.Counters,
	}
}

func hiddenCardView(card CardState) CardView {
	return CardView{
		OwnerID:    card.OwnerID,
		Zone:       card.Zone,
		Visibility: "hidden",
		Revealed:   false,
		Exhausted:  false,
	}
}

func visibleKeywords(card CardState) []string {
	if len(card.EffectiveKeywords) != 0 {
		return slices.Clone(card.EffectiveKeywords)
	}

	return slices.Clone(card.PrintedKeywords)
}

func visibleStats(card CardState) CardNumericStats {
	if card.EffectiveStats != (CardNumericStats{}) || card.PrintedStats == (CardNumericStats{}) {
		return card.EffectiveStats
	}

	return card.PrintedStats
}
