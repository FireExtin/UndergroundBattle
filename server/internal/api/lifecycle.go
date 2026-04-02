package api

// Purpose: Defines the single authoritative sandbox session lifecycle used by setup, match, and reset flows.

type SessionLifecycleKind string

const (
	SessionLifecycleReset         SessionLifecycleKind = "reset"
	SessionLifecycleSetup         SessionLifecycleKind = "setup"
	SessionLifecycleMatchActive   SessionLifecycleKind = "match_active"
	SessionLifecycleMatchFinished SessionLifecycleKind = "match_finished"
)

type SessionLifecycle struct {
	Kind             SessionLifecycleKind `json:"kind"`
	SetupStep        int                  `json:"setupStep,omitempty"`
	FinishedRevision int                  `json:"finishedRevision,omitempty"`
	ReportPath       string               `json:"reportPath,omitempty"`
}

func newResetLifecycle() SessionLifecycle {
	return SessionLifecycle{Kind: SessionLifecycleReset}
}

func newSetupLifecycle(step int) SessionLifecycle {
	return SessionLifecycle{
		Kind:      SessionLifecycleSetup,
		SetupStep: step,
	}
}

func newMatchActiveLifecycle() SessionLifecycle {
	return SessionLifecycle{Kind: SessionLifecycleMatchActive}
}

func newMatchFinishedLifecycle(report *MatchReport, finishedRevision int) SessionLifecycle {
	lifecycle := SessionLifecycle{
		Kind:             SessionLifecycleMatchFinished,
		FinishedRevision: finishedRevision,
	}
	if report != nil {
		lifecycle.ReportPath = report.Path
	}
	return lifecycle
}
