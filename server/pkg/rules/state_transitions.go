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
	card.FaceDown = false
	card.RegionCardID = ""
}

func moveCardToScore(card *CardState) {
	if card == nil {
		return
	}

	card.Destroyed = false
	card.Zone = CardZoneScore
	card.Revealed = true
	card.FaceDown = false
	card.Exhausted = false
	card.RegionCardID = ""
}

func moveCardToDeckBottom(card *CardState) {
	if card == nil {
		return
	}

	card.Destroyed = false
	card.Zone = CardZoneDeck
	card.Revealed = false
	card.FaceDown = false
	card.Exhausted = false
	card.RegionCardID = ""
}

func drawCardFromDeck(card *CardState) {
	if card == nil {
		return
	}

	card.Destroyed = false
	card.Zone = CardZoneHand
	card.Revealed = false
	card.FaceDown = false
}

func revealFaceDown(card *CardState) {
	if card == nil {
		return
	}

	card.FaceDown = false
	card.Revealed = true
}

func setFaceDown(card *CardState) {
	if card == nil {
		return
	}

	card.FaceDown = true
	card.Revealed = false
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

	if card.BaseInfluenceByPlayer == nil {
		card.BaseInfluenceByPlayer = map[string]int{}
	}
	card.BaseInfluenceByPlayer[controllerID] += amount
	if card.InfluenceByPlayer == nil {
		card.InfluenceByPlayer = map[string]int{}
	}
	card.InfluenceByPlayer[controllerID] += amount
	card.RegionInfluenceDerived = false
}

func addShieldCounter(card *CardState, amount int) {
	if card == nil || amount <= 0 {
		return
	}

	card.Counters.Shield += amount
}

func consumeShieldCounter(card *CardState, amount int) bool {
	if card == nil || amount <= 0 {
		return false
	}
	if card.Counters.Shield < amount {
		return false
	}

	card.Counters.Shield -= amount
	if card.Counters.Shield < 0 {
		card.Counters.Shield = 0
	}
	return true
}

func setMarker(state *GameState, playerID string, markerType string, amount int) {
	if state == nil {
		return
	}

	state.Board.Markers.SetMarker(playerID, markerType, amount)
}

func setCardMarker(state *GameState, cardID string, markerType string, amount int) {
	if state == nil || cardID == "" {
		return
	}

	state.Board.CardMarkers.SetMarker(cardID, markerType, amount)
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

func addCardMarkerCount(state *GameState, cardID string, markerType string, amount int) {
	if state == nil || cardID == "" || amount <= 0 {
		return
	}
	current := state.Board.CardMarkers.GetMarker(cardID, markerType)
	setCardMarker(state, cardID, markerType, current+amount)
}

func removeCardMarkerCount(state *GameState, cardID string, markerType string, amount int) {
	if state == nil || cardID == "" || amount <= 0 {
		return
	}
	current := state.Board.CardMarkers.GetMarker(cardID, markerType)
	next := current - amount
	if next < 0 {
		next = 0
	}
	setCardMarker(state, cardID, markerType, next)
}

func moveCardToRegion(card *CardState, regionCardID string) {
	if card == nil || regionCardID == "" {
		return
	}

	card.RegionCardID = regionCardID
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
