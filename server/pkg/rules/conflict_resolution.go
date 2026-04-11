package rules

import (
	"fmt"
)

// DetermineRegionConflictWinner computes the winner for a region conflict.
// Returns the winning playerID string, or empty string if no winner (tie with no tiebreaker).
// Error is returned when the region card cannot be found.
func DetermineRegionConflictWinner(state GameState, regionCardID string) (string, error) {
	index := findCardIndex(state, regionCardID)
	if index == -1 {
		return "", fmt.Errorf("region card %q not found", regionCardID)
	}

	region := state.Board.Cards[index]

	// Aggregate influence by player
	topScore := -1
	topPlayers := []string{}
	if region.InfluenceByPlayer != nil {
		for pid, score := range region.InfluenceByPlayer {
			if score > topScore {
				topScore = score
				topPlayers = []string{pid}
			} else if score == topScore {
				topPlayers = append(topPlayers, pid)
			}
		}
	} else {
		// No per-player influence information -> treat as no winner
		return "", nil
	}

	if len(topPlayers) == 0 {
		return "", nil
	}

	// Clear single-winner case
	if len(topPlayers) == 1 {
		return topPlayers[0], nil
	}

	// Tie: consult first-player privilege, then priority leader, then fallback to no winner.
	conflict := state.Turn.Conflict
	if conflict.FirstPlayerPrivilegeOwner != "" {
		return conflict.FirstPlayerPrivilegeOwner, nil
	}
	if conflict.PriorityLeaderPlayerID != "" {
		return conflict.PriorityLeaderPlayerID, nil
	}

	return "", nil
}
