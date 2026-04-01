package rules

import "reflect"

// Purpose: Hosts the authoritative submit/replay orchestration pipeline so engine.go stays focused on rules primitives.

type submitInternalOptions struct {
	projector            *ProjectionEngine
	enforceDeterminism   bool
	replayForDeterminism func(GameState, Action) (SubmitResult, error)
	cardSourceLookup     cardOperationSourceLookup
}

type submitPipelineState struct {
	preState GameState
	action   Action
	options  submitInternalOptions

	operation Operation
	event     Event

	postExecuteState GameState
	result           SubmitResult
}

type submitPipelinePhase func(*submitPipelineState) error

// SubmitAction runs the legality -> operation -> stack/direct resolution -> event -> commit pipeline.
func SubmitAction(state GameState, action Action) (SubmitResult, error) {
	return submitActionInternal(state, action, submitInternalOptions{
		projector:          NewProjectionEngine(),
		enforceDeterminism: true,
	})
}

// SubmitActionWithProjection runs the same pipeline as SubmitAction but exposes projection generation for tests and callers.
func SubmitActionWithProjection(state GameState, action Action, projector *ProjectionEngine) (SubmitResult, error) {
	return submitActionInternal(state, action, submitInternalOptions{
		projector:          projector,
		enforceDeterminism: true,
	})
}

func submitActionInternal(state GameState, action Action, options submitInternalOptions) (SubmitResult, error) {
	pipeline := submitPipelineState{
		preState: state,
		action:   action,
		options:  options,
	}

	phases := []submitPipelinePhase{
		submitLegalityPhase,
		submitBuildAndExecutePhase,
		submitCommitPhase,
		submitInvariantGuardPhase,
		submitDeterminismGuardPhase,
	}

	for _, phase := range phases {
		if err := phase(&pipeline); err != nil {
			return SubmitResult{}, err
		}
	}

	return pipeline.result, nil
}

func submitLegalityPhase(pipeline *submitPipelineState) error {
	legality := checkLegalityWithLookup(pipeline.preState, pipeline.action, pipeline.options.cardSourceLookup)
	if legality.OK {
		return nil
	}

	return newLegalityError(legality)
}

func submitBuildAndExecutePhase(pipeline *submitPipelineState) error {
	operation, err := buildOperationWithLookup(pipeline.preState, pipeline.action, pipeline.options.cardSourceLookup)
	if err != nil {
		return err
	}

	working := cloneGameState(pipeline.preState)
	working, operation, event, err := executeOperation(working, operation)
	if err != nil {
		return err
	}

	pipeline.postExecuteState = working
	pipeline.operation = operation
	pipeline.event = event
	return nil
}

func submitCommitPhase(pipeline *submitPipelineState) error {
	pipeline.result = commitState(
		pipeline.postExecuteState,
		pipeline.action,
		pipeline.operation,
		pipeline.event,
		pipeline.options.projector,
	)
	return nil
}

func submitInvariantGuardPhase(pipeline *submitPipelineState) error {
	if !DefaultInvariantConfig.Enabled {
		return nil
	}

	results := CheckAllInvariants(pipeline.result.State, DefaultInvariantConfig)
	for _, invariantResult := range results {
		if invariantResult.Passed {
			continue
		}

		invariantError := legalityFailure(
			ReasonCodeRulesFailedInvariantViolated,
			"rules.invariant.violated",
			"invariant.check",
			map[string]string{
				"actionId":      pipeline.action.ID,
				"invariantName": invariantResult.Name,
				"message":       invariantResult.Message,
			},
		)
		return newLegalityError(invariantError)
	}

	return nil
}

func submitDeterminismGuardPhase(pipeline *submitPipelineState) error {
	if !pipeline.options.enforceDeterminism {
		return nil
	}

	replay := pipeline.options.replayForDeterminism
	if replay == nil {
		replay = submitActionWithoutProjection
	}

	replayed, err := replay(pipeline.preState, pipeline.action)
	if err != nil {
		return newLegalityError(legalityFailure(
			ReasonCodeRulesFailedInvariantViolated,
			"rules.replay.non_deterministic",
			"replay.determinism",
			map[string]string{
				"actionId":    pipeline.action.ID,
				"replayError": err.Error(),
			},
		))
	}

	if statesMatchDeterministic(pipeline.result.State, replayed.State) {
		return nil
	}

	return newLegalityError(legalityFailure(
		ReasonCodeRulesFailedInvariantViolated,
		"rules.replay.non_deterministic",
		"replay.determinism",
		map[string]string{
			"actionId":       pipeline.action.ID,
			"revision":       intString(pipeline.result.State.Revision.Number),
			"replayRevision": intString(replayed.State.Revision.Number),
		},
	))
}

// ReplayActions replays an action log against an initial snapshot.
func ReplayActions(initial GameState, actions []Action) (GameState, error) {
	replayed := cloneGameState(initial)
	for _, action := range actions {
		result, err := submitActionWithoutProjection(replayed, action)
		if err != nil {
			return GameState{}, err
		}

		replayed = result.State
	}

	return replayed, nil
}

func submitActionWithoutProjection(state GameState, action Action) (SubmitResult, error) {
	return submitActionInternal(state, action, submitInternalOptions{
		projector:          nil,
		enforceDeterminism: false,
	})
}

func statesMatchDeterministic(left GameState, right GameState) bool {
	return reflect.DeepEqual(left, right)
}
