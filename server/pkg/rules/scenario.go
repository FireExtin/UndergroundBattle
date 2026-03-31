package rules

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"slices"
	"strings"
)

// Purpose: Loads committed M0 scenario fixtures and evaluates their stable observable snapshots against the authoritative rules kernel.

type Scenario struct {
	ID           string              `json:"id"`
	Description  string              `json:"description"`
	InitialState string              `json:"initialState"`
	Actions      []Action            `json:"actions"`
	Expectations ScenarioExpectation `json:"expectations"`
}

type ScenarioExpectation struct {
	Revision      int                           `json:"revision"`
	Turn          ScenarioTurnExpectation       `json:"turn"`
	Priority      ScenarioPriorityExpectation   `json:"priority"`
	Stack         []ScenarioOperationSnapshot   `json:"stack"`
	Resolved      []ScenarioOperationSnapshot   `json:"resolved"`
	ActionLog     []ScenarioActionLogEntry      `json:"actionLog"`
	Views         map[string]ScenarioView       `json:"views"`
	LastRejection *ScenarioRejectionExpectation `json:"lastRejection,omitempty"`
}

type ScenarioTurnExpectation struct {
	Phase PhaseName `json:"phase"`
	Step  StepName  `json:"step"`
}

type ScenarioPriorityExpectation struct {
	CurrentPlayerID string             `json:"currentPlayerId"`
	WindowKind      PriorityWindowKind `json:"windowKind"`
	PassCount       int                `json:"passCount"`
}

type ScenarioOperationSnapshot struct {
	Kind         OperationKind   `json:"kind"`
	Status       OperationStatus `json:"status"`
	Label        string          `json:"label,omitempty"`
	CardID       string          `json:"cardId,omitempty"`
	TargetCardID string          `json:"targetCardId,omitempty"`
}

type ScenarioActionLogEntry struct {
	ID   string     `json:"id"`
	Kind ActionKind `json:"kind"`
}

type ScenarioView struct {
	Cards []ScenarioCardSnapshot `json:"cards"`
}

type ScenarioCardSnapshot struct {
	OwnerID    string           `json:"ownerId"`
	Zone       CardZone         `json:"zone"`
	Visibility string           `json:"visibility"`
	CardID     string           `json:"cardId,omitempty"`
	Name       string           `json:"name,omitempty"`
	Revealed   bool             `json:"revealed"`
	Exhausted  bool             `json:"exhausted"`
	Destroyed  bool             `json:"destroyed"`
	Keywords   []string         `json:"keywords,omitempty"`
	Stats      CardNumericStats `json:"stats"`
	Counters   CardCounters     `json:"counters"`
}

type ScenarioRejectionExpectation struct {
	ReasonCode ReasonCode        `json:"reasonCode"`
	MessageKey string            `json:"messageKey"`
	Hook       string            `json:"hook"`
	Context    map[string]string `json:"context,omitempty"`
}

type ScenarioResult struct {
	State         GameState
	Views         ProjectionBundle
	LastRejection *LegalityResult
}

// LoadM0Scenarios reads all committed M0 scenario files from server/pkg/rules/testdata/m0.
func LoadM0Scenarios() ([]Scenario, error) {
	dir, err := m0ScenarioDirectory()
	if err != nil {
		return nil, err
	}

	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, err
	}

	scenarios := make([]Scenario, 0, len(entries))
	for _, entry := range entries {
		if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".json") {
			continue
		}

		path := filepath.Join(dir, entry.Name())
		data, err := os.ReadFile(path)
		if err != nil {
			return nil, err
		}

		var scenario Scenario
		if err := json.Unmarshal(data, &scenario); err != nil {
			return nil, fmt.Errorf("decode scenario %s: %w", path, err)
		}

		scenarios = append(scenarios, scenario)
	}

	slices.SortFunc(scenarios, func(left Scenario, right Scenario) int {
		return strings.Compare(left.ID, right.ID)
	})

	return scenarios, nil
}

// RunScenario replays one action script from the provided initial state and returns the final state plus final projections.
func RunScenario(initial GameState, actions []Action) (ScenarioResult, error) {
	state := cloneGameState(initial)
	for _, action := range actions {
		result, err := SubmitAction(state, action)
		if err != nil {
			var legality *LegalityError
			if errors.As(err, &legality) {
				views := NewProjectionEngine().Generate(state)
				last := legality.Result
				return ScenarioResult{
					State:         state,
					Views:         views,
					LastRejection: &last,
				}, err
			}

			return ScenarioResult{}, err
		}

		state = result.State
	}

	return ScenarioResult{
		State: state,
		Views: NewProjectionEngine().Generate(state),
	}, nil
}

// RunM0Scenario replays one committed M0 scenario using the canonical M0 initial state and tolerates expected structured rejections.
func RunM0Scenario(scenario Scenario) (ScenarioResult, error) {
	if scenario.InitialState != M0SandboxInitialStateID {
		return ScenarioResult{}, fmt.Errorf("unsupported initial state %q", scenario.InitialState)
	}

	result, err := RunScenario(NewM0SandboxState(), scenario.Actions)
	if err != nil {
		var legality *LegalityError
		if errors.As(err, &legality) && scenario.Expectations.LastRejection != nil {
			return result, nil
		}

		return ScenarioResult{}, err
	}

	if scenario.Expectations.LastRejection != nil {
		return ScenarioResult{}, fmt.Errorf("scenario %q expected rejection but action sequence committed", scenario.ID)
	}

	return result, nil
}

// CompareScenarioExpectation reduces a live scenario result into the stable M0 snapshot surface and returns human-readable diffs.
func CompareScenarioExpectation(result ScenarioResult, expected ScenarioExpectation) []string {
	return DiffScenarioExpectations(expected, SnapshotScenarioResult(result))
}

// SnapshotScenarioResult reduces a live scenario result into the stable observable baseline used by M0 fixtures.
func SnapshotScenarioResult(result ScenarioResult) ScenarioExpectation {
	return ScenarioExpectation{
		Revision:      result.State.Revision.Number,
		Turn:          ScenarioTurnExpectation{Phase: result.State.Turn.Phase.Name, Step: result.State.Turn.Phase.Step},
		Priority:      ScenarioPriorityExpectation{CurrentPlayerID: result.State.Turn.Priority.CurrentPlayerID, WindowKind: result.State.Turn.Priority.WindowKind, PassCount: result.State.Turn.Priority.PassCount},
		Stack:         snapshotOperations(result.State.Board.Stack),
		Resolved:      snapshotOperations(result.State.Board.Resolved),
		ActionLog:     snapshotActionLog(result.State.History.Actions),
		Views:         snapshotViews(result.Views),
		LastRejection: snapshotRejection(result.LastRejection),
	}
}

func snapshotOperations(operations []Operation) []ScenarioOperationSnapshot {
	snapshots := make([]ScenarioOperationSnapshot, 0, len(operations))
	for _, operation := range operations {
		snapshots = append(snapshots, ScenarioOperationSnapshot{
			Kind:         operation.Kind,
			Status:       operation.Status,
			Label:        operation.Label,
			CardID:       operation.CardID,
			TargetCardID: operation.TargetCardID,
		})
	}

	return snapshots
}

func snapshotActionLog(actions []Action) []ScenarioActionLogEntry {
	snapshots := make([]ScenarioActionLogEntry, 0, len(actions))
	for _, action := range actions {
		snapshots = append(snapshots, ScenarioActionLogEntry{
			ID:   action.ID,
			Kind: action.Kind,
		})
	}

	return snapshots
}

func snapshotViews(bundle ProjectionBundle) map[string]ScenarioView {
	views := map[string]ScenarioView{
		"spectator": {Cards: snapshotCardViews(bundle.Spectator.Board.Cards)},
	}

	playerIDs := make([]string, 0, len(bundle.Players))
	for playerID := range bundle.Players {
		playerIDs = append(playerIDs, playerID)
	}
	slices.Sort(playerIDs)
	for _, playerID := range playerIDs {
		views[playerID] = ScenarioView{
			Cards: snapshotCardViews(bundle.Players[playerID].Board.Cards),
		}
	}

	return views
}

func snapshotCardViews(cards []CardView) []ScenarioCardSnapshot {
	snapshots := make([]ScenarioCardSnapshot, 0, len(cards))
	for _, card := range cards {
		snapshots = append(snapshots, ScenarioCardSnapshot{
			OwnerID:    card.OwnerID,
			Zone:       card.Zone,
			Visibility: card.Visibility,
			CardID:     card.CardID,
			Name:       card.Name,
			Revealed:   card.Revealed,
			Exhausted:  card.Exhausted,
			Destroyed:  card.Destroyed,
			Keywords:   slices.Clone(card.Keywords),
			Stats:      card.Stats,
			Counters:   card.Counters,
		})
	}

	return snapshots
}

func snapshotRejection(legality *LegalityResult) *ScenarioRejectionExpectation {
	if legality == nil {
		return nil
	}

	return &ScenarioRejectionExpectation{
		ReasonCode: legality.ReasonCode,
		MessageKey: legality.MessageKey,
		Hook:       legality.Hook,
		Context:    cloneContext(legality.Context),
	}
}

func m0ScenarioDirectory() (string, error) {
	_, currentFile, _, ok := runtime.Caller(0)
	if !ok {
		return "", fmt.Errorf("runtime.Caller failed")
	}

	return filepath.Clean(filepath.Join(filepath.Dir(currentFile), "testdata/m0")), nil
}
