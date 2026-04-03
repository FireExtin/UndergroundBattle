package rules

// Purpose: Keeps phase-step-priority window enums small and isolated from the large state payload file.

type PhaseName string

const (
	PhaseMain     PhaseName = "main"
	PhaseConflict PhaseName = "conflict"
	PhaseEnd      PhaseName = "end"
)

type StepName string

const (
	StepAction             StepName = "action"
	StepFirstPlayerAction  StepName = "first_player_action"
	StepSecondPlayerAction StepName = "second_player_action"
	StepEnded              StepName = "ended"
)

type PriorityWindowKind string

const (
	PriorityWindowAction   PriorityWindowKind = "action"
	PriorityWindowResponse PriorityWindowKind = "response"
	PriorityWindowClosed   PriorityWindowKind = "closed"
)
