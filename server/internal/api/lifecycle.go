package api

import "fmt"

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

func (session *SandboxSession) Transition(next SessionLifecycle) error {
	current := session.lifecycle

	if next.Kind == SessionLifecycleReset {
		session.lifecycle = next
		return nil
	}

	var err error
	switch current.Kind {
	case SessionLifecycleReset, "":
		if next.Kind == SessionLifecycleSetup && next.SetupStep == 1 {
			break
		}
		if next.Kind == SessionLifecycleMatchFinished {
			break
		}
		err = fmt.Errorf("invalid transition from reset to %s", next.Kind)
	case SessionLifecycleSetup:
		if next.Kind == SessionLifecycleSetup {
			if next.SetupStep != current.SetupStep+1 {
				err = fmt.Errorf("invalid setup step transition: %d to %d", current.SetupStep, next.SetupStep)
			}
		} else if next.Kind == SessionLifecycleMatchActive {
			if current.SetupStep != 7 {
				err = fmt.Errorf("invalid transition to match active from setup step %d", current.SetupStep)
			}
		} else {
			err = fmt.Errorf("invalid transition from setup to %s", next.Kind)
		}
	case SessionLifecycleMatchActive:
		if next.Kind == SessionLifecycleMatchActive {
			return nil // No-op if already active
		}
		if next.Kind != SessionLifecycleMatchFinished {
			err = fmt.Errorf("invalid transition from match active to %s", next.Kind)
		}
	case SessionLifecycleMatchFinished:
		err = fmt.Errorf("invalid transition from match finished to %s", next.Kind)
	}

	if err != nil {
		return err
	}

	session.lifecycle = next
	return nil
}
