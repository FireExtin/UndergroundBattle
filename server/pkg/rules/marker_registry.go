package rules

// Purpose: Owns player-level and card-level marker registry data structures.

// MarkerRegistry tracks all player markers (e.g., secret society markers).
type MarkerRegistry struct {
	// ByPlayer maps playerID -> markerType -> amount
	ByPlayer map[string]map[string]int `json:"byPlayer"`
}

// GetMarker returns the amount of a specific marker type for a player.
func (r MarkerRegistry) GetMarker(playerID, markerType string) int {
	if r.ByPlayer == nil {
		return 0
	}
	playerMarkers, ok := r.ByPlayer[playerID]
	if !ok {
		return 0
	}
	return playerMarkers[markerType]
}

// SetMarker sets the amount of a specific marker type for a player.
func (r *MarkerRegistry) SetMarker(playerID, markerType string, amount int) {
	if r.ByPlayer == nil {
		r.ByPlayer = make(map[string]map[string]int)
	}
	if r.ByPlayer[playerID] == nil {
		r.ByPlayer[playerID] = make(map[string]int)
	}
	if amount <= 0 {
		delete(r.ByPlayer[playerID], markerType)
	} else {
		r.ByPlayer[playerID][markerType] = amount
	}
}

// CardMarkerRegistry tracks markers placed on specific cards.
type CardMarkerRegistry struct {
	// ByCard maps cardID -> markerType -> amount
	ByCard map[string]map[string]int `json:"byCard"`
}

// GetMarker returns the amount of a specific marker type on a card.
func (r CardMarkerRegistry) GetMarker(cardID, markerType string) int {
	if r.ByCard == nil {
		return 0
	}
	cardMarkers, ok := r.ByCard[cardID]
	if !ok {
		return 0
	}
	return cardMarkers[markerType]
}

// SetMarker sets the amount of a specific marker type on a card.
func (r *CardMarkerRegistry) SetMarker(cardID, markerType string, amount int) {
	if r.ByCard == nil {
		r.ByCard = make(map[string]map[string]int)
	}
	if r.ByCard[cardID] == nil {
		r.ByCard[cardID] = make(map[string]int)
	}
	if amount <= 0 {
		delete(r.ByCard[cardID], markerType)
	} else {
		r.ByCard[cardID][markerType] = amount
	}
}
