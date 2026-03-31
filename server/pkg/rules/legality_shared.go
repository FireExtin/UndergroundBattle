package rules

func cardMatchesDefinitionAndCondition(card CardState, definitionID string, condition CardCondition) bool {
	if card.DefinitionID != definitionID {
		return false
	}
	if condition.Zone != "" && card.Zone != condition.Zone {
		return false
	}
	if condition.Ready && card.Exhausted {
		return false
	}
	if condition.NotDestroyed && card.Destroyed {
		return false
	}
	if condition.Revealed && !card.Revealed {
		return false
	}
	return true
}

func scopeAppliesToActor(sourceCard CardState, actorID string, scope ProhibitionScope) bool {
	switch scope.Kind {
	case ProhibitionScopeAllPlayers:
		return true
	case ProhibitionScopeOpponentsOnly:
		return sourceCard.ControllerID != "" && actorID != sourceCard.ControllerID
	case ProhibitionScopeControllerOnly:
		return sourceCard.ControllerID != "" && sourceCard.ControllerID == actorID
	default:
		return false
	}
}
