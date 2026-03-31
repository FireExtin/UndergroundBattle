package rules

// Purpose: Implements scoped prohibition checking for card effects like XQ22.

// ScopedProhibitionChecker evaluates prohibition rules against the current game state.
type ScopedProhibitionChecker struct {
	rules []ProhibitionRule
}

// NewScopedProhibitionChecker creates a new checker with the given rules.
func NewScopedProhibitionChecker(rules []ProhibitionRule) *ScopedProhibitionChecker {
	return &ScopedProhibitionChecker{
		rules: rules,
	}
}

// Check evaluates whether a given action by an actor is prohibited.
// It returns a ProhibitionMatchResult indicating whether the action is prohibited and which rule matched.
func (c *ScopedProhibitionChecker) Check(
	state GameState,
	actorID string,
	targetCategory TargetCategory,
) ProhibitionMatchResult {
	for _, rule := range c.rules {
		// Find all source cards that match this rule's definition and condition
		for _, card := range state.Board.Cards {
			if !c.matchesSourceCondition(card, rule) {
				continue
			}

			// Check if the scope applies to this actor
			if !c.matchesScope(state, card, actorID, rule.Scope) {
				continue
			}

			// Check if the target category matches
			if !c.matchesTargetCategory(targetCategory, rule.TargetCategory) {
				continue
			}

			// All conditions match - this action is prohibited
			return ProhibitionMatchResult{
				Prohibited:     true,
				MatchedRule:    &rule,
				SourceCardID:   card.CardID,
				SourceCardName: card.Name,
			}
		}
	}

	// No prohibition rules matched
	return ProhibitionMatchResult{
		Prohibited: false,
	}
}

// matchesSourceCondition checks if a card satisfies the source condition of a prohibition rule.
func (c *ScopedProhibitionChecker) matchesSourceCondition(card CardState, rule ProhibitionRule) bool {
	// Must match the definition ID
	if card.DefinitionID != rule.SourceDefinitionID {
		return false
	}

	condition := rule.SourceCondition

	// Check zone requirement
	if condition.Zone != "" && card.Zone != condition.Zone {
		return false
	}

	// Check ready requirement
	if condition.Ready && card.Exhausted {
		return false
	}

	// Check not destroyed requirement
	if condition.NotDestroyed && card.Destroyed {
		return false
	}

	// Check revealed requirement (if specified)
	if condition.Revealed && !card.Revealed {
		return false
	}

	return true
}

// matchesScope checks if the prohibition scope applies to the given actor.
func (c *ScopedProhibitionChecker) matchesScope(
	state GameState,
	sourceCard CardState,
	actorID string,
	scope ProhibitionScope,
) bool {
	switch scope.Kind {
	case ProhibitionScopeAllPlayers:
		return true

	case ProhibitionScopeOpponentsOnly:
		// Prohibition applies only to opponents of the source controller
		return sourceCard.ControllerID != "" && actorID != sourceCard.ControllerID

	case ProhibitionScopeControllerOnly:
		// Prohibition applies only to the controller of the source
		return sourceCard.ControllerID == actorID

	default:
		return false
	}
}

// matchesTargetCategory checks if the target category matches the prohibition rule.
func (c *ScopedProhibitionChecker) matchesTargetCategory(
	actual TargetCategory,
	prohibited TargetCategory,
) bool {
	// Check basic types
	if len(prohibited.BasicTypes) > 0 {
		matched := false
		for _, prohibitedType := range prohibited.BasicTypes {
			for _, actualType := range actual.BasicTypes {
				if prohibitedType == actualType {
					matched = true
					break
				}
			}
			if matched {
				break
			}
		}
		if !matched {
			return false
		}
	}

	// Check action kinds
	if len(prohibited.ActionKinds) > 0 {
		matched := false
		for _, prohibitedKind := range prohibited.ActionKinds {
			for _, actualKind := range actual.ActionKinds {
				if prohibitedKind == actualKind {
					matched = true
					break
				}
			}
			if matched {
				break
			}
		}
		if !matched {
			return false
		}
	}

	return true
}

// Predefined prohibition rules for known cards.
// These can later be moved to fixture configuration.

// XQ22ProhibitionRule defines the prohibition effect of card XQ22 (州议员贝伦·希恩斯).
// When XQ22 is ready on the table, all players are prohibited from playing event cards (事务).
var XQ22ProhibitionRule = ProhibitionRule{
	SourceDefinitionID: "XQ22",
	SourceCondition: CardCondition{
		Zone:         CardZoneTable,
		Ready:        true,
		NotDestroyed: true,
	},
	Scope: ProhibitionScope{
		Kind: ProhibitionScopeAllPlayers,
	},
	TargetCategory: TargetCategory{
		BasicTypes: []string{"事务"},
	},
	Description: "XQ22: While ready, prohibits all players from playing event cards",
}

// BuildProhibitionChecker creates a checker with all active prohibition rules.
// This is the entry point for legality checks.
func BuildProhibitionChecker(state GameState) *ScopedProhibitionChecker {
	// Currently hardcoded rules - can be made dynamic later
	rules := []ProhibitionRule{
		XQ22ProhibitionRule,
	}

	return NewScopedProhibitionChecker(rules)
}
