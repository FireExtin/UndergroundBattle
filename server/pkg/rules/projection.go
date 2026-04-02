package rules

import "slices"

// Purpose: Defines player-specific and spectator-specific projections derived from the authoritative FullState after commit.

// CardZone identifies the minimal location used by the projection rules.
type CardZone string

const (
	CardZoneDeck    CardZone = "deck"
	CardZoneHand    CardZone = "hand"
	CardZoneTable   CardZone = "table"
	CardZoneAsset   CardZone = "asset"
	CardZoneDiscard CardZone = "discard"
	CardZoneScore   CardZone = "score"
)

// CardKind identifies the minimal role a card plays in legality and board semantics.
type CardKind string

const (
	CardKindUnknown   CardKind = "unknown"
	CardKindCharacter CardKind = "character"
	CardKindRegion    CardKind = "region"
	CardKindAsset     CardKind = "asset"
	CardKindEvent     CardKind = "event"
)

// CardState is the authoritative hidden-information record stored only in FullState.
type CardState struct {
	CardID              string           `json:"cardId"`
	DefinitionID        string           `json:"definitionId,omitempty"`
	Name                string           `json:"name"`
	Description         string           `json:"description,omitempty"`
	FAQ                 string           `json:"faq,omitempty"`
	Cost                int              `json:"cost,omitempty"`
	Color               string           `json:"color,omitempty"`
	Loyalty             string           `json:"loyalty,omitempty"`
	Kind                CardKind         `json:"kind,omitempty"`
	OwnerID             string           `json:"ownerId"`
	Zone                CardZone         `json:"zone"`
	RegionCardID        string           `json:"regionCardId,omitempty"`
	RegionOrder         int              `json:"regionOrder,omitempty"`
	RegionScore         int              `json:"regionScore,omitempty"`
	VisibleToOwner      bool             `json:"visibleToOwner"`
	Revealed            bool             `json:"revealed"`
	FaceDown            bool             `json:"faceDown,omitempty"` // 暗藏部署状态
	Exhausted           bool             `json:"exhausted"`
	InspectedBy         []string         `json:"inspectedBy,omitempty"`
	PrintedKeywords     []string         `json:"printedKeywords,omitempty"`
	EffectiveKeywords   []string         `json:"effectiveKeywords,omitempty"`
	PrintedStats        CardNumericStats `json:"printedStats"`
	EffectiveStats      CardNumericStats `json:"effectiveStats"`
	Counters            CardCounters     `json:"counters"`
	InfluenceByPlayer   map[string]int   `json:"influenceByPlayer,omitempty"`
	ControllerID        string           `json:"controllerId,omitempty"`
	Permissions         []string         `json:"permissions,omitempty"`
	Prohibitions        []string         `json:"prohibitions,omitempty"`
	RequiredPermissions []string         `json:"requiredPermissions,omitempty"`
	CostAdjustment      int              `json:"costAdjustment,omitempty"`
	ActionQuota         int              `json:"actionQuota,omitempty"`
	Destroyed           bool             `json:"destroyed"`
}

// CardView is the client-safe projection of a single card record.
type CardView struct {
	CardID       string           `json:"cardId,omitempty"`
	Name         string           `json:"name,omitempty"`
	Description  string           `json:"description,omitempty"`
	FAQ          string           `json:"faq,omitempty"`
	Cost         int              `json:"cost,omitempty"`
	Color        string           `json:"color,omitempty"`
	Loyalty      string           `json:"loyalty,omitempty"`
	OwnerID      string           `json:"ownerId"`
	Zone         CardZone         `json:"zone"`
	Kind         string           `json:"kind,omitempty"`
	RegionCardID string           `json:"regionCardId,omitempty"`
	RegionOrder  int              `json:"regionOrder,omitempty"`
	RegionScore  int              `json:"regionScore,omitempty"`
	Visibility   string           `json:"visibility"`
	Revealed     bool             `json:"revealed"`
	FaceDown     bool             `json:"faceDown,omitempty"` // 是否面朝下
	Exhausted    bool             `json:"exhausted"`
	Destroyed    bool             `json:"destroyed"`
	Keywords     []string         `json:"keywords,omitempty"`
	Stats        CardNumericStats `json:"stats"`
	Counters     CardCounters     `json:"counters"`
	Markers      map[string]int   `json:"markers,omitempty"`
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
	Match          MatchState     `json:"match"`
	Turn           TurnState      `json:"turn"`
	Score          ScoreState     `json:"score"`
	Board          ViewBoardState `json:"board"`
	Markers        map[string]int `json:"markers,omitempty"` // 玩家可见的标记物
}

// SpectatorViewState is the public-only projection generated for non-player observers.
type SpectatorViewState struct {
	GameID   string         `json:"gameId"`
	Revision Revision       `json:"revision"`
	Match    MatchState     `json:"match"`
	Turn     TurnState      `json:"turn"`
	Score    ScoreState     `json:"score"`
	Board    ViewBoardState `json:"board"`
	Markers  map[string]int `json:"markers,omitempty"` // 公开可见的标记物
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
			Match:          full.Match,
			Turn:           cloneTurnState(full.Turn),
			Score:          cloneScoreState(full.Score),
			Board:          projectBoardForPlayer(full, playerID),
			Markers:        projectMarkersForPlayer(full, playerID),
		}
	}

	return ProjectionBundle{
		Revision: full.Revision,
		Players:  playerViews,
		Spectator: SpectatorViewState{
			GameID:   full.GameID,
			Revision: full.Revision,
			Match:    full.Match,
			Turn:     cloneTurnState(full.Turn),
			Score:    cloneScoreState(full.Score),
			Board:    projectBoardForSpectator(full),
			Markers:  projectMarkersForSpectator(full),
		},
	}
}

func projectBoardForPlayer(full FullState, viewerPlayerID string) ViewBoardState {
	view := baseBoardProjection(full)
	for _, card := range full.Board.Cards {
		view.Cards = append(view.Cards, projectCardForPlayer(full, card, viewerPlayerID))
	}

	return view
}

func projectBoardForSpectator(full FullState) ViewBoardState {
	view := baseBoardProjection(full)
	for _, card := range full.Board.Cards {
		view.Cards = append(view.Cards, projectCardForSpectator(full, card))
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

func projectCardForPlayer(full FullState, card CardState, viewerPlayerID string) CardView {
	if cardVisibleToPlayer(card, viewerPlayerID) {
		return visibleCardView(card, projectCardMarkers(full, card.CardID))
	}

	return hiddenCardView(card)
}

func projectCardForSpectator(full FullState, card CardState) CardView {
	// Face-down cards are always hidden from spectators
	if card.FaceDown {
		return hiddenCardView(card)
	}

	if card.Revealed {
		return visibleCardView(card, projectCardMarkers(full, card.CardID))
	}

	return hiddenCardView(card)
}

func cardVisibleToPlayer(card CardState, viewerPlayerID string) bool {
	// Face-down cards are only visible to their owner
	if card.FaceDown {
		return card.OwnerID == viewerPlayerID
	}

	if card.Revealed {
		return true
	}

	if card.OwnerID == viewerPlayerID && card.VisibleToOwner {
		return true
	}

	return slices.Contains(card.InspectedBy, viewerPlayerID)
}

func visibleCardView(card CardState, markers map[string]int) CardView {
	return CardView{
		CardID:       card.CardID,
		Name:         card.Name,
		Description:  card.Description,
		FAQ:          card.FAQ,
		Cost:         card.Cost,
		Color:        card.Color,
		Loyalty:      card.Loyalty,
		OwnerID:      card.OwnerID,
		Zone:         card.Zone,
		Kind:         string(card.Kind),
		RegionCardID: card.RegionCardID,
		RegionOrder:  card.RegionOrder,
		RegionScore:  card.RegionScore,
		Visibility:   "visible",
		Revealed:     card.Revealed,
		FaceDown:     card.FaceDown,
		Exhausted:    card.Exhausted,
		Destroyed:    card.Destroyed,
		Keywords:     visibleKeywords(card),
		Stats:        visibleStats(card),
		Counters:     card.Counters,
		Markers:      markers,
	}
}

func hiddenCardView(card CardState) CardView {
	return CardView{
		OwnerID:      card.OwnerID,
		Zone:         card.Zone,
		RegionCardID: card.RegionCardID,
		Visibility:   "hidden",
		Revealed:     false,
		FaceDown:     card.FaceDown,
		Exhausted:    false,
	}
}

func projectCardMarkers(full FullState, cardID string) map[string]int {
	if cardID == "" || full.Board.CardMarkers.ByCard == nil {
		return nil
	}
	values, ok := full.Board.CardMarkers.ByCard[cardID]
	if !ok || len(values) == 0 {
		return nil
	}

	result := make(map[string]int, len(values))
	for markerType, amount := range values {
		result[markerType] = amount
	}
	return result
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
