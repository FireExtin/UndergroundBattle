package rules

import (
	"encoding/json"
	"errors"
	"reflect"
	"strings"
	"testing"
)

// Purpose: Freezes the M0 sandbox baseline as golden scenarios and guards replay, projection, legality, and invariant surfaces.

func TestNewM0SandboxStateProvidesCanonicalBaseline(t *testing.T) {
	state := NewM0SandboxState()

	if state.GameID != "game-sandbox-live" {
		t.Fatalf("game id = %q, want %q", state.GameID, "game-sandbox-live")
	}

	if state.Revision.Number != 0 {
		t.Fatalf("revision number = %d, want 0", state.Revision.Number)
	}

	if state.Turn.ActivePlayerID != "P1" {
		t.Fatalf("active player = %q, want %q", state.Turn.ActivePlayerID, "P1")
	}

	if len(state.Board.Cards) != 7 {
		t.Fatalf("card count = %d, want 7", len(state.Board.Cards))
	}

	if cardStateByID(t, state, "P1-HAND-SECRET").Name != "Secret Archive" {
		t.Fatalf("P1 secret card = %#v, want Secret Archive", cardStateByID(t, state, "P1-HAND-SECRET"))
	}

	if cardStateByID(t, state, "P2-TABLE-1").Counters.Damage != 1 {
		t.Fatalf("P2 table card damage = %d, want 1", cardStateByID(t, state, "P2-TABLE-1").Counters.Damage)
	}

	for index, cardID := range []string{"REGION-1", "REGION-2", "REGION-3"} {
		card := cardStateByID(t, state, cardID)
		if card.Kind != CardKindRegion {
			t.Fatalf("%s kind = %q, want %q", cardID, card.Kind, CardKindRegion)
		}
		if card.Zone != CardZoneTable {
			t.Fatalf("%s zone = %q, want %q", cardID, card.Zone, CardZoneTable)
		}
		if card.RegionOrder != index+1 {
			t.Fatalf("%s region order = %d, want %d", cardID, card.RegionOrder, index+1)
		}
		if !card.Revealed {
			t.Fatalf("%s revealed = false, want true", cardID)
		}
	}

	if cardStateByID(t, state, "P1-TABLE-1").RegionCardID != "REGION-1" {
		t.Fatalf("P1-TABLE-1 region card = %q, want %q", cardStateByID(t, state, "P1-TABLE-1").RegionCardID, "REGION-1")
	}
	if cardStateByID(t, state, "P2-TABLE-1").RegionCardID != "REGION-2" {
		t.Fatalf("P2-TABLE-1 region card = %q, want %q", cardStateByID(t, state, "P2-TABLE-1").RegionCardID, "REGION-2")
	}
}

func TestLoadM0ScenariosReadsCommittedFixtures(t *testing.T) {
	scenarios, err := LoadM0Scenarios()
	if err != nil {
		t.Fatalf("LoadM0Scenarios returned error: %v", err)
	}

	if len(scenarios) < 8 {
		t.Fatalf("scenario count = %d, want at least 8", len(scenarios))
	}

	for _, scenario := range scenarios {
		if scenario.InitialState != M0SandboxInitialStateID {
			t.Fatalf("scenario %q initialState = %q, want %q", scenario.ID, scenario.InitialState, M0SandboxInitialStateID)
		}
	}
}

func TestRunM0ScenariosMatchesGoldenSnapshots(t *testing.T) {
	scenarios, err := LoadM0Scenarios()
	if err != nil {
		t.Fatalf("LoadM0Scenarios returned error: %v", err)
	}

	for _, scenario := range scenarios {
		scenario := scenario
		t.Run(scenario.ID, func(t *testing.T) {
			t.Parallel()

			result, err := RunM0Scenario(scenario)
			if err != nil {
				t.Fatalf("RunM0Scenario returned error: %v", err)
			}

			if diffs := CompareScenarioExpectation(result, scenario.Expectations); len(diffs) != 0 {
				t.Fatalf("scenario diffs:\n%s", strings.Join(diffs, "\n"))
			}
		})
	}
}

func TestReplayMatchesM0ScenarioCommittedStates(t *testing.T) {
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

			replayed, err := ReplayActions(NewM0SandboxState(), scenario.Actions)
			if err != nil {
				t.Fatalf("ReplayActions(actions) returned error: %v", err)
			}

			if !reflect.DeepEqual(result.State, replayed) {
				t.Fatalf("replayed state mismatch\nscenario = %#v\nreplayed = %#v", result.State, replayed)
			}

			replayedFromHistory, err := ReplayActions(NewM0SandboxState(), result.State.History.Actions)
			if err != nil {
				t.Fatalf("ReplayActions(history actions) returned error: %v", err)
			}

			if !reflect.DeepEqual(result.State, replayedFromHistory) {
				t.Fatalf("history replay mismatch\nscenario = %#v\nhistory = %#v", result.State, replayedFromHistory)
			}
		})
	}
}

func TestM0ProjectionJSONDoesNotLeakHiddenInformation(t *testing.T) {
	scenarios, err := LoadM0Scenarios()
	if err != nil {
		t.Fatalf("LoadM0Scenarios returned error: %v", err)
	}

	for _, scenario := range scenarios {
		scenario := scenario
		t.Run(scenario.ID, func(t *testing.T) {
			t.Parallel()

			result, err := RunM0Scenario(scenario)
			if err != nil {
				t.Fatalf("RunM0Scenario returned error: %v", err)
			}

			playerTwo, ok := result.Views.Players["P2"]
			if !ok {
				t.Fatal("expected P2 view")
			}

			spectator := result.Views.Spectator

			p2JSON, err := json.Marshal(playerTwo)
			if err != nil {
				t.Fatalf("json.Marshal(P2 view) returned error: %v", err)
			}

			spectatorJSON, err := json.Marshal(spectator)
			if err != nil {
				t.Fatalf("json.Marshal(spectator view) returned error: %v", err)
			}

			if strings.Contains(string(p2JSON), "\"cardId\":\"P1-HAND-SECRET\"") && !strings.Contains(string(p2JSON), "\"name\":\"Secret Archive\"") {
				t.Fatalf("P2 JSON leaked hidden card identity unexpectedly: %s", string(p2JSON))
			}

			if strings.Contains(string(spectatorJSON), "Black Ledger") && scenario.ID != "reveal-own-secret-public" {
				t.Fatalf("spectator JSON leaked hidden hand card unexpectedly: %s", string(spectatorJSON))
			}
		})
	}
}

func TestTriggerableReasonCodesHaveStableCoverage(t *testing.T) {
	testCases := []struct {
		name string
		code ReasonCode
		run  func(t *testing.T) LegalityResult
	}{
		{
			name: "action id missing",
			code: ReasonCodeLegalityFailedActionIDMissing,
			run: func(t *testing.T) LegalityResult {
				return CheckLegality(NewM0SandboxState(), Action{
					ActorID: "P1",
					Kind:    ActionKindAdvancePhase,
				})
			},
		},
		{
			name: "actor id missing",
			code: ReasonCodeLegalityFailedActorIDMissing,
			run: func(t *testing.T) LegalityResult {
				return CheckLegality(NewM0SandboxState(), Action{
					ID:   "act-missing-actor",
					Kind: ActionKindAdvancePhase,
				})
			},
		},
		{
			name: "action id duplicate",
			code: ReasonCodeLegalityFailedActionIDDuplicate,
			run: func(t *testing.T) LegalityResult {
				state := mustSubmit(t, NewM0SandboxState(), Action{
					ID:      "act-dup-1",
					ActorID: "P1",
					Kind:    ActionKindAdvancePhase,
				})
				return CheckLegality(state, Action{
					ID:      "act-dup-1",
					ActorID: "P1",
					Kind:    ActionKindAdvancePhase,
				})
			},
		},
		{
			name: "not your priority",
			code: ReasonCodeLegalityFailedNotYourPriority,
			run: func(t *testing.T) LegalityResult {
				return CheckLegality(NewM0SandboxState(), Action{
					ID:      "act-priority",
					ActorID: "P2",
					Kind:    ActionKindAdvancePhase,
				})
			},
		},
		{
			name: "stack not empty",
			code: ReasonCodeLegalityFailedStackNotEmpty,
			run: func(t *testing.T) LegalityResult {
				state := mustSubmit(t, NewM0SandboxState(), Action{
					ID:           "act-stack-not-empty-1",
					ActorID:      "P1",
					Kind:         ActionKindQueueOperation,
					CardID:       "BQ005",
					TargetCardID: "P2-TABLE-1",
				})
				return CheckLegality(state, Action{
					ID:      "act-stack-not-empty-2",
					ActorID: "P2",
					Kind:    ActionKindAdvancePhase,
				})
			},
		},
		{
			name: "stack closed",
			code: ReasonCodeLegalityFailedStackClosed,
			run: func(t *testing.T) LegalityResult {
				state := NewM0SandboxState()
				state.Turn.Phase = phaseState(PhaseEnd)
				return CheckLegality(state, Action{
					ID:      "act-stack-closed",
					ActorID: "P1",
					Kind:    ActionKindQueueOperation,
					CardID:  "BQ010",
				})
			},
		},
		{
			name: "action window required",
			code: ReasonCodeLegalityFailedActionWindowRequired,
			run: func(t *testing.T) LegalityResult {
				state := mustSubmit(t, NewM0SandboxState(), Action{
					ID:           "act-window-1",
					ActorID:      "P1",
					Kind:         ActionKindQueueOperation,
					CardID:       "BQ005",
					TargetCardID: "P2-TABLE-1",
				})
				return CheckLegality(state, Action{
					ID:             "act-window-2",
					ActorID:        "P2",
					Kind:           ActionKindQueueOperation,
					CardID:         "BQ010",
					TargetPlayerID: "P1",
				})
			},
		},
		{
			name: "permission required",
			code: ReasonCodeLegalityFailedPermissionRequired,
			run: func(t *testing.T) LegalityResult {
				state := NewM0SandboxState()
				index := findCardIndex(state, "P2-HAND-SECRET")
				state.Board.Cards[index].RequiredPermissions = []string{"inspect"}
				return CheckLegality(state, Action{
					ID:      "act-permission-required",
					ActorID: "P1",
					Kind:    ActionKindInspectCard,
					CardID:  "P2-HAND-SECRET",
				})
			},
		},
		{
			name: "action prohibited",
			code: ReasonCodeLegalityFailedActionProhibited,
			run: func(t *testing.T) LegalityResult {
				state := NewM0SandboxState()
				index := findCardIndex(state, "P2-HAND-SECRET")
				state.Board.Cards[index].Prohibitions = []string{"inspect"}
				return CheckLegality(state, Action{
					ID:      "act-action-prohibited",
					ActorID: "P1",
					Kind:    ActionKindInspectCard,
					CardID:  "P2-HAND-SECRET",
				})
			},
		},
		{
			name: "target missing",
			code: ReasonCodeTargetFailedMissing,
			run: func(t *testing.T) LegalityResult {
				return CheckLegality(NewM0SandboxState(), Action{
					ID:      "act-target-missing",
					ActorID: "P1",
					Kind:    ActionKindRevealCard,
				})
			},
		},
		{
			name: "stack empty",
			code: ReasonCodeStackFailedEmpty,
			run: func(t *testing.T) LegalityResult {
				return CheckLegality(NewM0SandboxState(), Action{
					ID:      "act-stack-empty",
					ActorID: "P1",
					Kind:    ActionKindResolveTopStack,
				})
			},
		},
		{
			name: "card logic missing",
			code: ReasonCodeRulesFailedCardLogicMissing,
			run: func(t *testing.T) LegalityResult {
				return CheckLegality(NewM0SandboxState(), Action{
					ID:      "act-card-logic-missing",
					ActorID: "P1",
					Kind:    ActionKindQueueOperation,
					CardID:  "UNKNOWN-CARD",
				})
			},
		},
		{
			name: "unknown action kind",
			code: ReasonCodeRulesFailedUnknownActionKind,
			run: func(t *testing.T) LegalityResult {
				return CheckLegality(NewM0SandboxState(), Action{
					ID:      "act-unknown-action",
					ActorID: "P1",
					Kind:    ActionKind("totally_unknown"),
				})
			},
		},
		{
			name: "random max invalid",
			code: ReasonCodeRulesFailedRandomMaxInvalid,
			run: func(t *testing.T) LegalityResult {
				return CheckLegality(NewM0SandboxState(), Action{
					ID:        "act-random-invalid",
					ActorID:   "P1",
					Kind:      ActionKindRollSeededRandom,
					RandomMax: 0,
				})
			},
		},
		{
			name: "step ended",
			code: ReasonCodeRulesFailedStepEnded,
			run: func(t *testing.T) LegalityResult {
				state := mustSubmit(t, NewM0SandboxState(), Action{
					ID:      "act-step-ended-1",
					ActorID: "P1",
					Kind:    ActionKindPassPriority,
				})
				state = mustSubmit(t, state, Action{
					ID:      "act-step-ended-2",
					ActorID: "P2",
					Kind:    ActionKindPassPriority,
				})
				return CheckLegality(state, Action{
					ID:      "act-step-ended-3",
					ActorID: "P1",
					Kind:    ActionKindRevealCard,
					CardID:  "P1-HAND-SECRET",
				})
			},
		},
	}

	for _, testCase := range testCases {
		testCase := testCase
		t.Run(testCase.name, func(t *testing.T) {
			legality := testCase.run(t)
			if legality.OK {
				t.Fatal("expected legality to fail")
			}

			if legality.ReasonCode != testCase.code {
				t.Fatalf("reason code = %q, want %q", legality.ReasonCode, testCase.code)
			}
		})
	}
}

func TestCommitDispatchBatchJSONDoesNotContainFullState(t *testing.T) {
	result, err := SubmitActionWithProjection(NewM0SandboxState(), Action{
		ID:      "act-dispatch-no-fullstate",
		ActorID: "P1",
		Kind:    ActionKindRevealCard,
		CardID:  "P1-HAND-SECRET",
	}, NewProjectionEngine())
	if err != nil {
		t.Fatalf("SubmitActionWithProjection returned error: %v", err)
	}

	data, err := json.Marshal(result.Dispatch)
	if err != nil {
		t.Fatalf("json.Marshal(dispatch) returned error: %v", err)
	}

	payload := string(data)
	for _, forbidden := range []string{"\"history\":", "\"rng\":", "\"continuous\":", "\"inspectedBy\":"} {
		if strings.Contains(payload, forbidden) {
			t.Fatalf("dispatch payload leaked full-state field %q: %s", forbidden, payload)
		}
	}
}

func FuzzCheckLegalityDoesNotPanic(f *testing.F) {
	f.Add("", "", "", "", "", 0)
	f.Add("act-1", "P1", string(ActionKindAdvancePhase), "", "", 0)
	f.Add("act-2", "P2", string(ActionKindQueueOperation), "BQ005", "P2-TABLE-1", 0)

	f.Fuzz(func(t *testing.T, actionID string, actorID string, rawKind string, cardID string, targetCardID string, randomMax int) {
		state := NewM0SandboxState()
		action := Action{
			ID:           actionID,
			ActorID:      actorID,
			Kind:         ActionKind(rawKind),
			CardID:       cardID,
			TargetCardID: targetCardID,
			RandomMax:    randomMax,
		}

		_ = CheckLegality(state, action)
	})
}

func FuzzProjectionEngineGenerateDoesNotLeakHiddenCardNames(f *testing.F) {
	f.Add(false, false)
	f.Add(true, false)

	f.Fuzz(func(t *testing.T, revealed bool, inspected bool) {
		state := NewM0SandboxState()
		index := findCardIndex(state, "P1-HAND-SECRET")
		state.Board.Cards[index].Revealed = revealed
		if inspected {
			state.Board.Cards[index].InspectedBy = []string{"P2"}
		}

		views := NewProjectionEngine().Generate(state)
		p2JSON, err := json.Marshal(views.Players["P2"])
		if err != nil {
			t.Fatalf("json.Marshal(P2 view) returned error: %v", err)
		}

		if !revealed && !inspected && strings.Contains(string(p2JSON), "Secret Archive") {
			t.Fatalf("hidden card name leaked to P2: %s", string(p2JSON))
		}
	})
}

func FuzzRecalculateContinuousEffectsRemainsDeterministic(f *testing.F) {
	f.Add(0, 0)
	f.Add(2, 1)

	f.Fuzz(func(t *testing.T, amount int, turn int) {
		state := NewM0SandboxState()
		if amount < -10 || amount > 10 {
			t.Skip()
		}
		if turn < 1 || turn > 5 {
			t.Skip()
		}

		state.Turn.TurnNumber = turn
		state.Board.Cards = append(state.Board.Cards, CardState{
			CardID:         "fuzz-target",
			Name:           "Fuzz Target",
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Defense: 2},
		})
		state.Board.Continuous.Active = append(state.Board.Continuous.Active, ContinuousEffect{
			ID:           "ce:fuzz",
			Layer:        LayerNumeric,
			EffectKind:   "modifyStat",
			TargetCardID: "fuzz-target",
			Stat:         "defense",
			Amount:       amount,
			DurationKind: "permanent",
			Timestamp:    1,
		})

		left := RecalculateContinuousEffects(state)
		right := RecalculateContinuousEffects(state)
		if !reflect.DeepEqual(left, right) {
			t.Fatalf("recalculated states differ\nleft  = %#v\nright = %#v", left, right)
		}
	})
}

func TestRunM0ScenarioSurfacesStructuredLegalityErrors(t *testing.T) {
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

	if result.LastRejection == nil {
		t.Fatal("expected structured rejection")
	}

	if result.LastRejection.ReasonCode != ReasonCodeLegalityFailedNotYourPriority {
		t.Fatalf("reason code = %q, want %q", result.LastRejection.ReasonCode, ReasonCodeLegalityFailedNotYourPriority)
	}
}

func TestRunM0ScenarioReturnsLegalityErrorWhenActionRejected(t *testing.T) {
	state := NewM0SandboxState()
	_, err := RunScenario(state, []Action{{
		ID:      "act-rejected",
		ActorID: "P2",
		Kind:    ActionKindAdvancePhase,
	}})
	if err == nil {
		t.Fatal("expected RunScenario to return error")
	}

	var legality *LegalityError
	if !errors.As(err, &legality) {
		t.Fatalf("expected LegalityError, got %T", err)
	}
}

func findScenarioByID(scenarios []Scenario, scenarioID string) (Scenario, bool) {
	for _, scenario := range scenarios {
		if scenario.ID == scenarioID {
			return scenario, true
		}
	}

	return Scenario{}, false
}
