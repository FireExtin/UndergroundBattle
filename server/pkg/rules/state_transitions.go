package rules

import "fmt"

// Purpose: Centralizes authoritative state transitions so rules execution avoids ad-hoc field mutation.

type attachmentTransitionSpec struct {
	SourceCardID       string
	SourceDefinitionID string
	SourceOperationID  string
	TargetCardID       string
	HostCardID         string
	Revision           int
	BasicType          string
}

func moveCardToDiscard(card *CardState) {
	if card == nil {
		return
	}

	card.Destroyed = true
	card.Zone = CardZoneDiscard
	card.Revealed = true
}

func revealFaceDown(card *CardState) {
	if card == nil {
		return
	}

	card.FaceDown = false
	card.Revealed = true
}

func markCardInspected(card *CardState, inspectorID string) {
	if card == nil || inspectorID == "" {
		return
	}

	if containsString(card.InspectedBy, inspectorID) {
		return
	}

	card.InspectedBy = append(card.InspectedBy, inspectorID)
}

func appendGeneratedDrawCard(state *GameState, operationID string, ownerID string, sequence int) {
	if state == nil || operationID == "" || ownerID == "" || sequence <= 0 {
		return
	}

	state.Board.Cards = append(state.Board.Cards, CardState{
		CardID:         fmt.Sprintf("draw:%s:%d", operationID, sequence),
		Name:           "",
		OwnerID:        ownerID,
		Zone:           CardZoneHand,
		VisibleToOwner: true,
		Revealed:       false,
		Exhausted:      false,
	})
}

func appendRandomResult(state *GameState, result RandomResult) {
	if state == nil {
		return
	}

	state.Board.RandomResults = append(state.Board.RandomResults, result)
}

func appendResolvedOperation(state *GameState, operation Operation) {
	if state == nil {
		return
	}

	state.Board.Resolved = append(state.Board.Resolved, cloneOperation(operation))
}

func exhaustCard(card *CardState) {
	if card == nil {
		return
	}

	card.Exhausted = true
}

func addDamageCounter(card *CardState, amount int) {
	if card == nil || amount <= 0 {
		return
	}

	card.Counters.Damage += amount
}

func addInfluenceCounter(card *CardState, controllerID string, amount int) {
	if card == nil || amount <= 0 {
		return
	}

	card.Counters.Influence += amount
	if card.Kind != CardKindRegion {
		return
	}

	if card.InfluenceByPlayer == nil {
		card.InfluenceByPlayer = map[string]int{}
	}
	card.InfluenceByPlayer[controllerID] += amount
}

func setMarker(state *GameState, playerID string, markerType string, amount int) {
	if state == nil {
		return
	}

	state.Board.Markers.SetMarker(playerID, markerType, amount)
}

func addMarkerCount(state *GameState, playerID string, markerType string, amount int) {
	if state == nil || amount <= 0 {
		return
	}

	current := state.Board.Markers.GetMarker(playerID, markerType)
	setMarker(state, playerID, markerType, current+amount)
}

func removeMarkerCount(state *GameState, playerID string, markerType string, amount int) {
	if state == nil || amount <= 0 {
		return
	}

	current := state.Board.Markers.GetMarker(playerID, markerType)
	next := current - amount
	if next < 0 {
		next = 0
	}
	setMarker(state, playerID, markerType, next)
}

func attachToHost(state GameState, spec attachmentTransitionSpec) (GameState, string, error) {
	builder := NewAttachment(state).
		To(spec.TargetCardID).
		AtRevision(spec.Revision).
		WithBasicType(spec.BasicType).
		FromDefinition(spec.SourceDefinitionID).
		FromOperation(spec.SourceOperationID)
	if spec.SourceCardID != "" {
		builder = builder.From(spec.SourceCardID)
	}
	if spec.HostCardID != "" {
		builder = builder.Host(spec.HostCardID)
	}

	nextState, err := builder.Create()
	if err != nil {
		return state, "", err
	}

	last := len(nextState.Board.Attachments.Active) - 1
	if last < 0 {
		return nextState, "", nil
	}

	return nextState, nextState.Board.Attachments.Active[last].ID, nil
}
