package rules

// Purpose: Keeps turn/prompt privacy projection logic out of projection.go's line budget hotspot.

func projectTurnForPlayer(full FullState, viewerPlayerID string) TurnState {
	turn := cloneTurnState(full.Turn)
	sanitizePromptForViewer(&turn, viewerPlayerID, false)
	return turn
}

func projectTurnForSpectator(full FullState) TurnState {
	turn := cloneTurnState(full.Turn)
	sanitizePromptForViewer(&turn, "", true)
	return turn
}

func sanitizePromptForViewer(turn *TurnState, viewerPlayerID string, spectator bool) {
	if turn == nil || turn.PendingPrompt == nil {
		return
	}

	ownerVisible := !spectator && viewerPlayerID != "" && turn.PendingPrompt.OwnerPlayerID == viewerPlayerID
	if ownerVisible {
		return
	}

	prompt := clonePromptState(turn.PendingPrompt)
	if prompt == nil {
		return
	}
	prompt.PeekCardIDs = nil
	prompt.EligibleTargetIDs = nil
	turn.PendingPrompt = prompt
}
