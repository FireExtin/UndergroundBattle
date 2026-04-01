package rules

// Purpose: Encodes rulebook conflict-resolution policies as deterministic, testable primitives.

// DrawStepPolicy captures the draw-step handling decision used by the rulebook extraction.
type DrawStepPolicy struct {
	DrawWithoutStack       bool
	DrawTriggersEnterStack bool
	PostDrawWindow         PriorityWindowKind
}

// DefaultDrawStepPolicy returns the current draw-step policy:
// draw action itself does not enter stack, triggers do, then an action window opens.
func DefaultDrawStepPolicy() DrawStepPolicy {
	return DrawStepPolicy{
		DrawWithoutStack:       true,
		DrawTriggersEnterStack: true,
		PostDrawWindow:         PriorityWindowAction,
	}
}

// RecoveryStepPhase identifies one ordered phase inside the recovery step.
type RecoveryStepPhase string

const (
	RecoveryStepPhaseFirstPlayerDiscardToLimit    RecoveryStepPhase = "first_player_discard_to_limit"
	RecoveryStepPhaseSecondPlayerDiscardToLimit   RecoveryStepPhase = "second_player_discard_to_limit"
	RecoveryStepPhaseClearDamageAndEndTurnEffects RecoveryStepPhase = "clear_damage_and_end_turn_effects"
	RecoveryStepPhaseTransferFirstPlayer          RecoveryStepPhase = "transfer_first_player"
)

// DefaultRecoveryStepOrder returns the canonical recovery ordering selected in the extraction.
func DefaultRecoveryStepOrder() []RecoveryStepPhase {
	return []RecoveryStepPhase{
		RecoveryStepPhaseFirstPlayerDiscardToLimit,
		RecoveryStepPhaseSecondPlayerDiscardToLimit,
		RecoveryStepPhaseClearDamageAndEndTurnEffects,
		RecoveryStepPhaseTransferFirstPlayer,
	}
}

// RegionWinStep identifies one ordered action in region-win resolution.
type RegionWinStep string

const (
	RegionWinStepClearRegionUnits         RegionWinStep = "clear_region_units"
	RegionWinStepMoveRegionToScore        RegionWinStep = "move_region_to_score"
	RegionWinStepRefillRegionSlot         RegionWinStep = "refill_region_slot"
	RegionWinStepEnqueueRegionWinTriggers RegionWinStep = "enqueue_region_win_triggers"
	RegionWinStepOpenFastWindow           RegionWinStep = "open_fast_window"
)

// DefaultRegionWinOrder returns the canonical region-win order selected in the extraction.
func DefaultRegionWinOrder() []RegionWinStep {
	return []RegionWinStep{
		RegionWinStepClearRegionUnits,
		RegionWinStepMoveRegionToScore,
		RegionWinStepRefillRegionSlot,
		RegionWinStepEnqueueRegionWinTriggers,
		RegionWinStepOpenFastWindow,
	}
}

// ContestOutcome is the normalized outcome for icon-count contests.
type ContestOutcome string

const (
	ContestOutcomeNotOccurred ContestOutcome = "not_occurred"
	ContestOutcomeTie         ContestOutcome = "tie"
	ContestOutcomeActorWin    ContestOutcome = "actor_win"
	ContestOutcomeActorLose   ContestOutcome = "actor_lose"
)

// ResolveContestOutcome resolves one icon contest from the acting player's perspective.
// Both sides zero means the contest did not occur, not a tie.
func ResolveContestOutcome(actorIcons int, opponentIcons int) ContestOutcome {
	if actorIcons < 0 {
		actorIcons = 0
	}
	if opponentIcons < 0 {
		opponentIcons = 0
	}

	if actorIcons == 0 && opponentIcons == 0 {
		return ContestOutcomeNotOccurred
	}
	if actorIcons == opponentIcons {
		return ContestOutcomeTie
	}
	if actorIcons > opponentIcons {
		return ContestOutcomeActorWin
	}
	return ContestOutcomeActorLose
}

// FirstPlayerPrivilegeResult captures the resolved result of trying to use first-player privilege.
type FirstPlayerPrivilegeResult struct {
	Allowed          bool
	Consumed         bool
	Outcome          ContestOutcome
	VirtualAdvantage int
}

// ApplyFirstPlayerPrivilege applies the first-player privilege to a tie when available.
func ApplyFirstPlayerPrivilege(outcome ContestOutcome, alreadyUsed bool) FirstPlayerPrivilegeResult {
	if alreadyUsed || outcome != ContestOutcomeTie {
		return FirstPlayerPrivilegeResult{
			Allowed:          false,
			Consumed:         false,
			Outcome:          outcome,
			VirtualAdvantage: 0,
		}
	}

	return FirstPlayerPrivilegeResult{
		Allowed:          true,
		Consumed:         true,
		Outcome:          ContestOutcomeActorWin,
		VirtualAdvantage: 1,
	}
}
