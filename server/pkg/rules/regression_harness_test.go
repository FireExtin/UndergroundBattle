package rules

import (
	"slices"
	"strings"
	"testing"
)

// Purpose: Verifies replay-diff diagnostics and invariant validators that turn M0 scenarios into a reusable regression harness.

func TestDiffScenarioResultsHighlightsStableSurfaceChanges(t *testing.T) {
	scenarios, err := LoadM0Scenarios()
	if err != nil {
		t.Fatalf("LoadM0Scenarios returned error: %v", err)
	}

	scenario, ok := findScenarioByID(scenarios, "read-minds-direct-resolve")
	if !ok {
		t.Fatal("expected read-minds-direct-resolve scenario")
	}

	left, err := RunM0Scenario(scenario)
	if err != nil {
		t.Fatalf("RunM0Scenario(left) returned error: %v", err)
	}

	right := ScenarioResult{
		State: cloneGameState(left.State),
	}
	right.State.Revision.Number++
	right.State.History.Actions[0].ID = "act-mutated"
	cardIndex := findCardIndex(right.State, "P1-HAND-SECRET")
	right.State.Board.Cards[cardIndex].Revealed = true
	right.Views = NewProjectionEngine().Generate(right.State)

	diffs := DiffScenarioResults(left, right)
	if len(diffs) == 0 {
		t.Fatal("expected replay diff to report stable-surface changes")
	}

	joined := strings.Join(diffs, "\n")
	for _, fragment := range []string{
		"revision",
		"actionLog[0].id",
		"views.P2.cards[0].visibility",
	} {
		if !strings.Contains(joined, fragment) {
			t.Fatalf("diff output missing %q:\n%s", fragment, joined)
		}
	}
}

func TestSuccessfulM0ScenariosSatisfyRegressionInvariants(t *testing.T) {
	scenarios, err := LoadM0Scenarios()
	if err != nil {
		t.Fatalf("LoadM0Scenarios returned error: %v", err)
	}

	for _, scenario := range scenarios {
		if scenario.Expectations.LastRejection != nil {
			continue
		}

		scenario := scenario
		t.Run(scenario.ID, func(t *testing.T) {
			t.Parallel()

			result, err := RunM0Scenario(scenario)
			if err != nil {
				t.Fatalf("RunM0Scenario returned error: %v", err)
			}

			issues := ValidateScenarioResultInvariants(result)
			if len(issues) != 0 {
				t.Fatalf("invariant issues:\n%s", strings.Join(issues, "\n"))
			}
		})
	}
}

func TestValidateScenarioResultInvariantsReportsProjectionLeakAndHistoryDrift(t *testing.T) {
	result, err := RunScenario(NewM0SandboxState(), []Action{
		{
			ID:      "act-invariant-1",
			ActorID: "P1",
			Kind:    ActionKindRevealCard,
			CardID:  "P1-HAND-SECRET",
		},
	})
	if err != nil {
		t.Fatalf("RunScenario returned error: %v", err)
	}

	result.State.History.Revisions = nil
	result.Views.Players["P2"] = clonePlayerView(result.Views.Players["P2"])
	result.Views.Players["P2"].Board.Cards[0].Name = "Leaked Secret"
	result.Views.Players["P2"].Board.Cards[0].CardID = "P1-HAND-SECRET"
	result.Views.Players["P2"].Board.Cards[0].Visibility = "visible"

	issues := ValidateScenarioResultInvariants(result)
	if len(issues) == 0 {
		t.Fatal("expected invariant validator to report issues")
	}

	joined := strings.Join(issues, "\n")
	for _, fragment := range []string{
		"history.revisions length",
		"views.P2.cards[0]",
	} {
		if !strings.Contains(joined, fragment) {
			t.Fatalf("invariant issues missing %q:\n%s", fragment, joined)
		}
	}
}

func TestValidateScenarioResultInvariantsAcceptsRejectedScenarioWithoutCommit(t *testing.T) {
	scenarios, err := LoadM0Scenarios()
	if err != nil {
		t.Fatalf("LoadM0Scenarios returned error: %v", err)
	}

	scenario, ok := findScenarioByID(scenarios, "illegal-not-your-priority")
	if !ok {
		t.Fatal("expected illegal-not-your-priority scenario")
	}

	result, err := RunM0Scenario(scenario)
	if err != nil {
		t.Fatalf("RunM0Scenario returned error: %v", err)
	}

	issues := ValidateScenarioResultInvariants(result)
	if len(issues) != 0 {
		t.Fatalf("rejected scenario should still satisfy non-commit invariants: %v", issues)
	}
}

func TestDiffScenarioResultsReturnsNoDiffForSameScenarioResult(t *testing.T) {
	scenarios, err := LoadM0Scenarios()
	if err != nil {
		t.Fatalf("LoadM0Scenarios returned error: %v", err)
	}

	scenario, ok := findScenarioByID(scenarios, "bootstrap-projections")
	if !ok {
		t.Fatal("expected bootstrap-projections scenario")
	}

	result, err := RunM0Scenario(scenario)
	if err != nil {
		t.Fatalf("RunM0Scenario returned error: %v", err)
	}

	if diffs := DiffScenarioResults(result, result); len(diffs) != 0 {
		t.Fatalf("expected no replay diff, got %v", diffs)
	}
}

func TestDiffScenarioResultsSortsViewKeysDeterministically(t *testing.T) {
	left := ScenarioResult{Views: ProjectionBundle{Players: map[string]PlayerViewState{}}}
	right := ScenarioResult{Views: ProjectionBundle{Players: map[string]PlayerViewState{}}}
	right.Views.Players["P2"] = PlayerViewState{}
	right.Views.Players["P1"] = PlayerViewState{}

	diffs := DiffScenarioResults(left, right)
	if len(diffs) == 0 {
		t.Fatal("expected diff output for added views")
	}

	if !slices.IsSortedFunc(diffs, strings.Compare) && !strings.Contains(strings.Join(diffs, "\n"), "views.P1") {
		t.Fatalf("expected deterministic ordered diff output, got %v", diffs)
	}
}
