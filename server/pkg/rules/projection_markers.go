package rules

// Purpose: Isolates marker projection helpers from projection.go to keep file complexity budgets stable.

func projectMarkersForPlayer(full FullState, playerID string) map[string]int {
	if full.Board.Markers.ByPlayer == nil {
		return nil
	}
	playerMarkers, ok := full.Board.Markers.ByPlayer[playerID]
	if !ok || len(playerMarkers) == 0 {
		return nil
	}
	result := make(map[string]int)
	for markerType, amount := range playerMarkers {
		result[markerType] = amount
	}
	return result
}

func projectMarkersForSpectator(full FullState) map[string]int {
	if full.Board.Markers.ByPlayer == nil {
		return nil
	}
	totalMarkers := make(map[string]int)
	hasMarkers := false
	for _, playerMarkers := range full.Board.Markers.ByPlayer {
		for markerType, amount := range playerMarkers {
			totalMarkers[markerType] += amount
			hasMarkers = true
		}
	}
	if !hasMarkers {
		return nil
	}
	return totalMarkers
}
