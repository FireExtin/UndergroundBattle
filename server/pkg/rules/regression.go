package rules

import (
	"encoding/json"
	"fmt"
	"reflect"
	"slices"
	"sort"
)

// Purpose: Provides reusable replay-diff and invariant-validation helpers for the M0 regression harness.

// DiffScenarioResults compares two scenario executions over the stable M0 snapshot surface.
func DiffScenarioResults(left ScenarioResult, right ScenarioResult) []string {
	return DiffScenarioExpectations(SnapshotScenarioResult(left), SnapshotScenarioResult(right))
}

// DiffScenarioExpectations compares two stable scenario snapshots and returns deterministic path-oriented diffs.
func DiffScenarioExpectations(left ScenarioExpectation, right ScenarioExpectation) []string {
	leftValue := scenarioExpectationAsJSONValue(left)
	rightValue := scenarioExpectationAsJSONValue(right)

	var diffs []string
	compareJSONValues("", leftValue, rightValue, &diffs)
	return diffs
}

// ValidateScenarioResultInvariants checks history, projection, and continuous-effect bookkeeping invariants for one replay result.
func ValidateScenarioResultInvariants(result ScenarioResult) []string {
	issues := make([]string, 0)
	issues = append(issues, validateHistoryInvariants(result.State)...)
	issues = append(issues, validateContinuousRegistryInvariants(result.State)...)
	issues = append(issues, validateProjectionBundleInvariants(result.State, result.Views)...)
	return issues
}

func validateHistoryInvariants(state GameState) []string {
	issues := make([]string, 0)
	revision := state.Revision.Number

	if len(state.History.Actions) != revision {
		issues = append(issues, fmt.Sprintf("history.actions length = %d, want %d", len(state.History.Actions), revision))
	}
	if len(state.History.Operations) != revision {
		issues = append(issues, fmt.Sprintf("history.operations length = %d, want %d", len(state.History.Operations), revision))
	}
	if len(state.History.Events) != revision {
		issues = append(issues, fmt.Sprintf("history.events length = %d, want %d", len(state.History.Events), revision))
	}
	if len(state.History.Revisions) != revision {
		issues = append(issues, fmt.Sprintf("history.revisions length = %d, want %d", len(state.History.Revisions), revision))
	}

	seenActionIDs := map[string]struct{}{}
	for index, action := range state.History.Actions {
		if action.ID == "" {
			issues = append(issues, fmt.Sprintf("history.actions[%d].id is empty", index))
			continue
		}
		if _, exists := seenActionIDs[action.ID]; exists {
			issues = append(issues, fmt.Sprintf("history.actions[%d].id %q duplicated", index, action.ID))
			continue
		}
		seenActionIDs[action.ID] = struct{}{}
	}

	for index, revisionEntry := range state.History.Revisions {
		want := index + 1
		if revisionEntry.Number != want {
			issues = append(issues, fmt.Sprintf("history.revisions[%d].number = %d, want %d", index, revisionEntry.Number, want))
		}
	}

	if revision > 0 && len(state.History.Revisions) > 0 {
		last := state.History.Revisions[len(state.History.Revisions)-1]
		if last != state.Revision {
			issues = append(issues, fmt.Sprintf("state.revision = %#v, want %#v", state.Revision, last))
		}
	}

	for index, operation := range state.Board.Stack {
		if operation.Status != OperationStatusPending {
			issues = append(issues, fmt.Sprintf("board.stack[%d].status = %q, want %q", index, operation.Status, OperationStatusPending))
		}
	}

	for index, operation := range state.Board.Resolved {
		if operation.Status != OperationStatusResolved {
			issues = append(issues, fmt.Sprintf("board.resolved[%d].status = %q, want %q", index, operation.Status, OperationStatusResolved))
		}
	}

	return issues
}

func validateContinuousRegistryInvariants(state GameState) []string {
	registry := state.Board.Continuous
	issues := make([]string, 0)

	if registry.InProgress {
		issues = append(issues, "board.continuous.inProgress should be false after scenario execution")
	}

	if registry.LastAppliedRevision > state.Revision.Number {
		issues = append(issues, fmt.Sprintf("board.continuous.lastAppliedRevision = %d, want <= %d", registry.LastAppliedRevision, state.Revision.Number))
	}

	if registry.FullRecalculationCount > state.Revision.Number {
		issues = append(issues, fmt.Sprintf("board.continuous.fullRecalculationCount = %d, want <= %d", registry.FullRecalculationCount, state.Revision.Number))
	}

	return issues
}

func validateProjectionBundleInvariants(state GameState, views ProjectionBundle) []string {
	issues := make([]string, 0)

	if views.Revision != state.Revision {
		issues = append(issues, fmt.Sprintf("views.revision = %#v, want %#v", views.Revision, state.Revision))
	}

	playerIDs := slices.Clone(state.Players)
	slices.Sort(playerIDs)
	for _, playerID := range playerIDs {
		view, ok := views.Players[playerID]
		if !ok {
			issues = append(issues, fmt.Sprintf("views.%s missing", playerID))
			continue
		}

		if view.GameID != state.GameID {
			issues = append(issues, fmt.Sprintf("views.%s.gameId = %q, want %q", playerID, view.GameID, state.GameID))
		}
		if view.ViewerPlayerID != playerID {
			issues = append(issues, fmt.Sprintf("views.%s.viewerPlayerId = %q, want %q", playerID, view.ViewerPlayerID, playerID))
		}
		if view.Revision != state.Revision {
			issues = append(issues, fmt.Sprintf("views.%s.revision = %#v, want %#v", playerID, view.Revision, state.Revision))
		}
		if !reflect.DeepEqual(view.Turn, state.Turn) {
			issues = append(issues, fmt.Sprintf("views.%s.turn = %#v, want %#v", playerID, view.Turn, state.Turn))
		}
		issues = append(issues, validateProjectedBoard(playerID, state, view.Board, false)...)
	}

	if views.Spectator.GameID != state.GameID {
		issues = append(issues, fmt.Sprintf("views.spectator.gameId = %q, want %q", views.Spectator.GameID, state.GameID))
	}
	if views.Spectator.Revision != state.Revision {
		issues = append(issues, fmt.Sprintf("views.spectator.revision = %#v, want %#v", views.Spectator.Revision, state.Revision))
	}
	if !reflect.DeepEqual(views.Spectator.Turn, state.Turn) {
		issues = append(issues, fmt.Sprintf("views.spectator.turn = %#v, want %#v", views.Spectator.Turn, state.Turn))
	}
	issues = append(issues, validateProjectedBoard("spectator", state, views.Spectator.Board, true)...)

	return issues
}

func validateProjectedBoard(viewerID string, state GameState, board ViewBoardState, spectator bool) []string {
	issues := make([]string, 0)

	if !reflect.DeepEqual(board.Stack, state.Board.Stack) {
		issues = append(issues, fmt.Sprintf("views.%s.stack mismatch", viewerID))
	}
	if !reflect.DeepEqual(board.Resolved, state.Board.Resolved) {
		issues = append(issues, fmt.Sprintf("views.%s.resolved mismatch", viewerID))
	}
	if !reflect.DeepEqual(board.RandomResults, state.Board.RandomResults) {
		issues = append(issues, fmt.Sprintf("views.%s.randomResults mismatch", viewerID))
	}
	if len(board.Cards) != len(state.Board.Cards) {
		issues = append(issues, fmt.Sprintf("views.%s.cards length = %d, want %d", viewerID, len(board.Cards), len(state.Board.Cards)))
		return issues
	}

	for index, fullCard := range state.Board.Cards {
		viewCard := board.Cards[index]
		path := fmt.Sprintf("views.%s.cards[%d]", viewerID, index)
		shouldBeVisible := fullCard.Revealed
		if !spectator {
			shouldBeVisible = cardVisibleToPlayer(fullCard, viewerID)
		}

		if shouldBeVisible {
			if viewCard.Visibility != "visible" {
				issues = append(issues, fmt.Sprintf("%s.visibility = %q, want %q", path, viewCard.Visibility, "visible"))
			}
			if viewCard.CardID != fullCard.CardID {
				issues = append(issues, fmt.Sprintf("%s.cardId = %q, want %q", path, viewCard.CardID, fullCard.CardID))
			}
			if viewCard.Name != fullCard.Name {
				issues = append(issues, fmt.Sprintf("%s.name = %q, want %q", path, viewCard.Name, fullCard.Name))
			}
			if viewCard.Revealed != fullCard.Revealed {
				issues = append(issues, fmt.Sprintf("%s.revealed = %t, want %t", path, viewCard.Revealed, fullCard.Revealed))
			}
			if viewCard.Exhausted != fullCard.Exhausted {
				issues = append(issues, fmt.Sprintf("%s.exhausted = %t, want %t", path, viewCard.Exhausted, fullCard.Exhausted))
			}
			if viewCard.Destroyed != fullCard.Destroyed {
				issues = append(issues, fmt.Sprintf("%s.destroyed = %t, want %t", path, viewCard.Destroyed, fullCard.Destroyed))
			}
			if !reflect.DeepEqual(viewCard.Keywords, visibleKeywords(fullCard)) {
				issues = append(issues, fmt.Sprintf("%s.keywords = %#v, want %#v", path, viewCard.Keywords, visibleKeywords(fullCard)))
			}
			if viewCard.Stats != visibleStats(fullCard) {
				issues = append(issues, fmt.Sprintf("%s.stats = %#v, want %#v", path, viewCard.Stats, visibleStats(fullCard)))
			}
			if viewCard.Counters != fullCard.Counters {
				issues = append(issues, fmt.Sprintf("%s.counters = %#v, want %#v", path, viewCard.Counters, fullCard.Counters))
			}
		} else {
			if viewCard.Visibility != "hidden" {
				issues = append(issues, fmt.Sprintf("%s.visibility = %q, want %q", path, viewCard.Visibility, "hidden"))
			}
			if viewCard.CardID != "" {
				issues = append(issues, fmt.Sprintf("%s.cardId = %q, want empty", path, viewCard.CardID))
			}
			if viewCard.Name != "" {
				issues = append(issues, fmt.Sprintf("%s.name = %q, want empty", path, viewCard.Name))
			}
			if viewCard.Revealed {
				issues = append(issues, fmt.Sprintf("%s.revealed = true, want false", path))
			}
			if viewCard.Exhausted {
				issues = append(issues, fmt.Sprintf("%s.exhausted = true, want false", path))
			}
			if viewCard.Destroyed {
				issues = append(issues, fmt.Sprintf("%s.destroyed = true, want false", path))
			}
			if len(viewCard.Keywords) != 0 {
				issues = append(issues, fmt.Sprintf("%s.keywords = %#v, want empty", path, viewCard.Keywords))
			}
			if viewCard.Stats != (CardNumericStats{}) {
				issues = append(issues, fmt.Sprintf("%s.stats = %#v, want zero stats", path, viewCard.Stats))
			}
			if viewCard.Counters != (CardCounters{}) {
				issues = append(issues, fmt.Sprintf("%s.counters = %#v, want zero counters", path, viewCard.Counters))
			}
		}
	}

	return issues
}

func scenarioExpectationAsJSONValue(expectation ScenarioExpectation) any {
	data, err := json.Marshal(expectation)
	if err != nil {
		panic(err)
	}

	var value any
	if err := json.Unmarshal(data, &value); err != nil {
		panic(err)
	}

	return value
}

func compareJSONValues(path string, left any, right any, diffs *[]string) {
	switch leftTyped := left.(type) {
	case map[string]any:
		rightTyped, ok := right.(map[string]any)
		if !ok {
			*diffs = append(*diffs, fmt.Sprintf("%s: %s != %s", normalizedPath(path), formatDiffValue(left), formatDiffValue(right)))
			return
		}

		keySet := make(map[string]struct{}, len(leftTyped)+len(rightTyped))
		for key := range leftTyped {
			keySet[key] = struct{}{}
		}
		for key := range rightTyped {
			keySet[key] = struct{}{}
		}

		keys := make([]string, 0, len(keySet))
		for key := range keySet {
			keys = append(keys, key)
		}
		sort.Strings(keys)

		for _, key := range keys {
			nextPath := key
			if path != "" {
				nextPath = path + "." + key
			}
			leftValue, leftOK := leftTyped[key]
			rightValue, rightOK := rightTyped[key]
			if !leftOK || !rightOK {
				*diffs = append(*diffs, fmt.Sprintf("%s: %s != %s", normalizedPath(nextPath), formatDiffValue(leftValue), formatDiffValue(rightValue)))
				continue
			}
			compareJSONValues(nextPath, leftValue, rightValue, diffs)
		}
	case []any:
		rightTyped, ok := right.([]any)
		if !ok {
			*diffs = append(*diffs, fmt.Sprintf("%s: %s != %s", normalizedPath(path), formatDiffValue(left), formatDiffValue(right)))
			return
		}

		if len(leftTyped) != len(rightTyped) {
			*diffs = append(*diffs, fmt.Sprintf("%s length: %d != %d", normalizedPath(path), len(leftTyped), len(rightTyped)))
		}

		limit := len(leftTyped)
		if len(rightTyped) < limit {
			limit = len(rightTyped)
		}
		for index := 0; index < limit; index++ {
			nextPath := fmt.Sprintf("%s[%d]", normalizedPath(path), index)
			compareJSONValues(nextPath, leftTyped[index], rightTyped[index], diffs)
		}
	default:
		if !reflect.DeepEqual(left, right) {
			*diffs = append(*diffs, fmt.Sprintf("%s: %s != %s", normalizedPath(path), formatDiffValue(left), formatDiffValue(right)))
		}
	}
}

func normalizedPath(path string) string {
	if path == "" {
		return "<root>"
	}

	return path
}

func formatDiffValue(value any) string {
	if value == nil {
		return "null"
	}

	return fmt.Sprintf("%v", value)
}
