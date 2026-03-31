package rules

import "fmt"

// Purpose: Provides the deterministic seeded RNG used by replayable rules execution.

const (
	rngMultiplier uint64 = 6364136223846793005
	rngIncrement  uint64 = 1442695040888963407
)

// NextRandom advances the RNG and returns a bounded deterministic value.
func NextRandom(state RNGState, maxExclusive int) (RNGState, int, error) {
	if maxExclusive <= 0 {
		return state, 0, fmt.Errorf("maxExclusive must be > 0")
	}

	next := state
	next.State = next.State*rngMultiplier + rngIncrement
	next.DrawCount++

	value := int(next.State % uint64(maxExclusive))
	return next, value, nil
}
