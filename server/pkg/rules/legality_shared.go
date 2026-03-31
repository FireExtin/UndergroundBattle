package rules

import "slices"

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

func cardMatchesTargetCondition(sourceCard CardState, targetCard CardState, condition TargetCondition) bool {
	if condition.Side != "" {
		isAlly := sourceCard.ControllerID != "" && targetCard.ControllerID == sourceCard.ControllerID
		if condition.Side == SideAlly && !isAlly {
			return false
		}
		if condition.Side == SideEnemy && isAlly {
			return false
		}
	}

	if len(condition.Keywords) > 0 {
		targetKeywords := targetCard.EffectiveKeywords
		if len(targetKeywords) == 0 {
			targetKeywords = targetCard.PrintedKeywords
		}
		matched := false
		for _, reqKeyword := range condition.Keywords {
			if slices.Contains(targetKeywords, reqKeyword) {
				matched = true
				break
			}
		}
		if !matched {
			return false
		}
	}

	return true
}
