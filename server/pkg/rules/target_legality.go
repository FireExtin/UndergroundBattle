package rules

// Purpose: Implements target legality checking (for XQ31 "不能成为目标"一类效果.

import "slices"

// TargetLegalityChecker evaluates whether a target can be acted upon.
type TargetLegalityChecker struct {
	rules []TargetLegalityRule
}

// NewTargetLegalityChecker creates a new checker with the given rules.
func NewTargetLegalityChecker(rules []TargetLegalityRule) *TargetLegalityChecker {
	return &TargetLegalityChecker{
		rules: rules,
	}
}

// CheckTargetCard checks if the given card can be targeted by the actor.
func (c *TargetLegalityChecker) CheckTargetCard(
	state GameState,
	actorID string,
	targetCardID string,
) TargetLegalityMatchResult {
	// Find the target card first
	var targetCard *CardState
	for i, card := range state.Board.Cards {
		if card.CardID == targetCardID {
			targetCard = &state.Board.Cards[i]
			break
		}
	}

	if targetCard == nil {
		// Target card not found - default to allowing (or handle error)
		return TargetLegalityMatchResult{
			CanTarget: true,
		}
	}

	// Iterate all rules and all source cards
	for _, rule := range c.rules {
		for i, sourceCard := range state.Board.Cards {
			if !c.matchesSourceCondition(sourceCard, rule) {
				continue
			}

			if !c.matchesActorRestriction(state, sourceCard, actorID, rule.ActorRestriction) {
				continue
			}

			if !c.matchesAffectedTarget(*targetCard, rule) {
				continue
			}

			// Check side condition if specified
			if rule.AffectedTargetCondition.Side != "" {
				if !c.matchesSide(sourceCard, *targetCard, rule.AffectedTargetCondition.Side) {
					continue
				}
			}

			// All conditions match - cannot target
			return TargetLegalityMatchResult{
				CanTarget:      false,
				MatchedRule:    &rule,
				SourceCardID:   state.Board.Cards[i].CardID,
				SourceCardName: state.Board.Cards[i].Name,
			}
		}
	}

	// No rules matched - can target
	return TargetLegalityMatchResult{
		CanTarget: true,
	}
}

// matchesSourceCondition checks if a card satisfies the source condition of a target legality rule.
func (c *TargetLegalityChecker) matchesSourceCondition(
	card CardState,
	rule TargetLegalityRule,
) bool {
	return cardMatchesDefinitionAndCondition(card, rule.SourceDefinitionID, rule.SourceCondition)
}

// matchesActorRestriction checks if the actor restriction applies to the given actor.
func (c *TargetLegalityChecker) matchesActorRestriction(
	state GameState,
	sourceCard CardState,
	actorID string,
	restriction ProhibitionScope,
) bool {
	return scopeAppliesToActor(sourceCard, actorID, restriction)
}

// matchesAffectedTarget checks if the target satisfies the affected target condition.
func (c *TargetLegalityChecker) matchesAffectedTarget(
	targetCard CardState,
	rule TargetLegalityRule,
) bool {
	condition := rule.AffectedTargetCondition

	// Check side requirement
	if condition.Side != "" {
		// Side check requires a source card to compare against
		// This will be handled in the main Check function
	}

	// Check keywords requirement
	if len(condition.Keywords) > 0 {
		matched := false
		targetKeywords := targetCard.EffectiveKeywords
		if len(targetKeywords) == 0 {
			targetKeywords = targetCard.PrintedKeywords
		}
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

	// Check region ID requirement (reserved for XQ01)
	if condition.RegionID != "" {
		// Region check requires region ID matching will be implemented when region model is in place
		// For now, skip this check
	}

	// Check ability kinds requirement (reserved for XQ01)
	if len(condition.AbilityKinds) > 0 {
		// Ability kinds check will be implemented when ability type model is in place
		// For now, skip this check
	}

	return true
}

// matchesSide checks if the target side matches.
func (c *TargetLegalityChecker) matchesSide(
	sourceCard CardState,
	targetCard CardState,
	side SideKind,
) bool {
	switch side {
	case SideAlly:
		return sourceCard.ControllerID != "" && sourceCard.ControllerID == targetCard.ControllerID
	case SideEnemy:
		return sourceCard.ControllerID != "" && sourceCard.ControllerID != targetCard.ControllerID
	default:
		return true
	}
}

// XQ31TargetLegalityRule defines the target legality rule for card XQ31 (莫兰大主教).
// When XQ31 is ready on the table, enemies cannot target prestige allies.
var XQ31TargetLegalityRule = TargetLegalityRule{
	SourceDefinitionID: "XQ31",
	SourceCondition: CardCondition{
		Zone:         CardZoneTable,
		Ready:        true,
		NotDestroyed: true,
	},
	AffectedTargetCondition: TargetCondition{
		Keywords: []string{"声望"},
		Side:     SideAlly,
	},
	ActorRestriction: ProhibitionScope{
		Kind: ProhibitionScopeOpponentsOnly,
	},
	Description: "XQ31: Enemies cannot target prestige allies",
}

// BuildTargetLegalityChecker creates a checker with all active target legality rules.
// This is the entry point for target legality checks.
func BuildTargetLegalityChecker(state GameState) *TargetLegalityChecker {
	return NewTargetLegalityChecker(BuildProductionTargetLegalityRules())
}
