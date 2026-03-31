package rules

import "slices"

// Purpose: Builds the minimal structured protocol messages emitted by the authoritative rules kernel.

func okLegalityResult() LegalityResult {
	return LegalityResult{OK: true}
}

func legalityFailure(code ReasonCode, messageKey string, hook string, context map[string]string) LegalityResult {
	return LegalityResult{
		OK:         false,
		ReasonCode: code,
		MessageKey: messageKey,
		Hook:       hook,
		Context:    cloneContext(context),
	}
}

func newLegalityError(result LegalityResult) *LegalityError {
	return &LegalityError{
		Result:     result,
		Code:       result.ReasonCode,
		Message:    result.MessageKey,
		MessageKey: result.MessageKey,
		Hook:       result.Hook,
		Context:    cloneContext(result.Context),
	}
}

// NewActionAccepted builds the minimal accepted-action protocol payload.
func NewActionAccepted(action Action, operation Operation, event Event, revision Revision) ActionAccepted {
	return ActionAccepted{
		Type:      "ActionAccepted",
		Action:    action,
		Operation: operation,
		Event:     event,
		Revision:  revision,
	}
}

// NewActionRejected builds the minimal rejected-action protocol payload.
func NewActionRejected(action Action, legality LegalityResult) ActionRejected {
	return ActionRejected{
		Type:     "ActionRejected",
		Action:   action,
		Legality: legality,
	}
}

// NewStatePatchedForPlayer builds the minimal audience-specific patch payload for one player.
func NewStatePatchedForPlayer(views ProjectionBundle, viewerPlayerID string, event Event, revision Revision) StatePatched {
	view, ok := views.Players[viewerPlayerID]
	if !ok {
		return StatePatched{
			Type:         "StatePatched",
			AudienceKind: string(DispatchAudiencePlayer),
			AudienceID:   viewerPlayerID,
			Revision:     revision,
			Event:        event,
		}
	}

	cloned := clonePlayerView(view)
	return StatePatched{
		Type:         "StatePatched",
		AudienceKind: string(DispatchAudiencePlayer),
		AudienceID:   viewerPlayerID,
		Revision:     revision,
		Event:        event,
		PlayerView:   &cloned,
	}
}

// NewStatePatchedForSpectator builds the minimal audience-specific patch payload for public spectators.
func NewStatePatchedForSpectator(views ProjectionBundle, event Event, revision Revision) StatePatched {
	cloned := cloneSpectatorView(views.Spectator)
	return StatePatched{
		Type:          "StatePatched",
		AudienceKind:  string(DispatchAudienceSpectator),
		Revision:      revision,
		Event:         event,
		SpectatorView: &cloned,
	}
}

// BuildCommitDispatchBatch converts one committed action result into a per-client dispatch batch.
func BuildCommitDispatchBatch(result SubmitResult) DispatchBatch {
	messages := make([]ClientDispatch, 0, len(result.Views.Players)+2)

	accepted := result.Accepted
	messages = append(messages, ClientDispatch{
		Kind:           DispatchPayloadActionAccepted,
		Target:         DispatchTarget{Kind: DispatchAudiencePlayer, ID: accepted.Action.ActorID},
		ActionAccepted: &accepted,
	})

	playerIDs := make([]string, 0, len(result.Views.Players))
	for playerID := range result.Views.Players {
		playerIDs = append(playerIDs, playerID)
	}
	slices.Sort(playerIDs)

	for _, playerID := range playerIDs {
		patch := NewStatePatchedForPlayer(result.Views, playerID, result.Event, result.Revision)
		messages = append(messages, ClientDispatch{
			Kind:         DispatchPayloadStatePatched,
			Target:       DispatchTarget{Kind: DispatchAudiencePlayer, ID: playerID},
			StatePatched: &patch,
		})
	}

	spectatorPatch := NewStatePatchedForSpectator(result.Views, result.Event, result.Revision)
	messages = append(messages, ClientDispatch{
		Kind:         DispatchPayloadStatePatched,
		Target:       DispatchTarget{Kind: DispatchAudienceSpectator},
		StatePatched: &spectatorPatch,
	})

	return DispatchBatch{
		Revision: result.Revision,
		Messages: messages,
	}
}

// BuildRejectedDispatchBatch converts one rejected action into a per-client rejection batch.
func BuildRejectedDispatchBatch(action Action, legality LegalityResult) DispatchBatch {
	rejected := NewActionRejected(action, legality)
	return DispatchBatch{
		Messages: []ClientDispatch{
			{
				Kind:           DispatchPayloadActionRejected,
				Target:         DispatchTarget{Kind: DispatchAudiencePlayer, ID: action.ActorID},
				ActionRejected: &rejected,
			},
		},
	}
}

func cloneContext(context map[string]string) map[string]string {
	if len(context) == 0 {
		return nil
	}

	cloned := make(map[string]string, len(context))
	for key, value := range context {
		cloned[key] = value
	}

	return cloned
}

func clonePlayerView(view PlayerViewState) PlayerViewState {
	cloned := view
	cloned.Board = cloneViewBoardState(view.Board)
	return cloned
}

func cloneSpectatorView(view SpectatorViewState) SpectatorViewState {
	cloned := view
	cloned.Board = cloneViewBoardState(view.Board)
	return cloned
}

func cloneViewBoardState(board ViewBoardState) ViewBoardState {
	cloned := board
	cloned.Stack = cloneOperations(board.Stack)
	cloned.Resolved = cloneOperations(board.Resolved)
	cloned.RandomResults = append([]RandomResult(nil), board.RandomResults...)
	cloned.Cards = cloneCardViews(board.Cards)
	return cloned
}

func cloneCardViews(cards []CardView) []CardView {
	cloned := make([]CardView, 0, len(cards))
	for _, card := range cards {
		next := card
		next.Keywords = slices.Clone(card.Keywords)
		cloned = append(cloned, next)
	}

	return cloned
}
