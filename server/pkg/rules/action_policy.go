package rules

import "slices"

// Purpose: Defines authoritative action policies and rules metadata shared with projections and UI consumers.

type ActionActorConstraint string

const (
	ActionActorConstraintAny            ActionActorConstraint = "any"
	ActionActorConstraintPriorityPlayer ActionActorConstraint = "priority_player"
	ActionActorConstraintActivePlayer   ActionActorConstraint = "active_player"
)

type ActionFieldName string

const (
	ActionFieldNameCardID             ActionFieldName = "cardId"
	ActionFieldNameTargetPlayerID     ActionFieldName = "targetPlayerId"
	ActionFieldNameTargetCardID       ActionFieldName = "targetCardId"
	ActionFieldNameTargetRegionCardID ActionFieldName = "targetRegionCardId"
	ActionFieldNamePlayMode           ActionFieldName = "playMode"
	ActionFieldNameMarkerType         ActionFieldName = "markerType"
	ActionFieldNameMarkerAmount       ActionFieldName = "markerAmount"
	ActionFieldNameRandomMax          ActionFieldName = "randomMax"
)

type ActionFieldRequirement string

const (
	ActionFieldRequirementOptional  ActionFieldRequirement = "optional"
	ActionFieldRequirementRequired  ActionFieldRequirement = "required"
	ActionFieldRequirementForbidden ActionFieldRequirement = "forbidden"
)

type ActionFieldRule struct {
	Field       ActionFieldName        `json:"field"`
	Requirement ActionFieldRequirement `json:"requirement"`
	SourceKinds []CardKind             `json:"sourceKinds,omitempty"`
	MinimumInt  int                    `json:"minimumInt,omitempty"`
}

type ActionCardKindConstraint struct {
	Kind                 CardKind `json:"kind"`
	RequiresEmptyStack   bool     `json:"requiresEmptyStack"`
	RequiresActionWindow bool     `json:"requiresActionWindow"`
}

type ActionPolicy struct {
	ActionKind           ActionKind                 `json:"actionKind"`
	ActorConstraint      ActionActorConstraint      `json:"actorConstraint"`
	RequiresPriority     bool                       `json:"requiresPriority"`
	RequiresEmptyStack   bool                       `json:"requiresEmptyStack"`
	RequiresActionWindow bool                       `json:"requiresActionWindow"`
	FieldRules           []ActionFieldRule          `json:"fieldRules,omitempty"`
	CardKindConstraints  []ActionCardKindConstraint `json:"cardKindConstraints,omitempty"`
}

type LoyaltyColorAlias struct {
	Canonical string   `json:"canonical"`
	Aliases   []string `json:"aliases,omitempty"`
}

type LoyaltyMetadata struct {
	ColorAliases []LoyaltyColorAlias `json:"colorAliases"`
}

type ProjectionContract struct {
	HiddenCardPreserves []string `json:"hiddenCardPreserves"`
}

type RulesMetadata struct {
	ActionPolicies []ActionPolicy     `json:"actionPolicies"`
	Loyalty        LoyaltyMetadata    `json:"loyalty"`
	Projection     ProjectionContract `json:"projection"`
}

var defaultActionPolicies = []ActionPolicy{
	{ActionKind: ActionKindAdvancePhase, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true},
	{ActionKind: ActionKindRevealCard, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameCardID, Requirement: ActionFieldRequirementRequired}}},
	{ActionKind: ActionKindInspectCard, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameCardID, Requirement: ActionFieldRequirementRequired}}},
	{ActionKind: ActionKindPassPriority, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true},
	{ActionKind: ActionKindQueueOperation, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameCardID, Requirement: ActionFieldRequirementRequired}}},
	{ActionKind: ActionKindDeclareAttack, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameCardID, Requirement: ActionFieldRequirementRequired}, {Field: ActionFieldNameTargetCardID, Requirement: ActionFieldRequirementRequired}}},
	{ActionKind: ActionKindDeclareInvestigation, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameCardID, Requirement: ActionFieldRequirementRequired}, {Field: ActionFieldNameTargetCardID, Requirement: ActionFieldRequirementRequired}}},
	{ActionKind: ActionKindResolveTopStack, ActorConstraint: ActionActorConstraintAny},
	{ActionKind: ActionKindRollSeededRandom, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameRandomMax, Requirement: ActionFieldRequirementRequired, MinimumInt: 1}}},
	{ActionKind: ActionKindSetMarker, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameTargetPlayerID, Requirement: ActionFieldRequirementOptional}, {Field: ActionFieldNameMarkerType, Requirement: ActionFieldRequirementRequired}, {Field: ActionFieldNameMarkerAmount, Requirement: ActionFieldRequirementRequired, MinimumInt: 1}}},
	{ActionKind: ActionKindRemoveMarker, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameTargetPlayerID, Requirement: ActionFieldRequirementOptional}, {Field: ActionFieldNameMarkerType, Requirement: ActionFieldRequirementRequired}, {Field: ActionFieldNameMarkerAmount, Requirement: ActionFieldRequirementRequired}}},
	{ActionKind: ActionKindSetFaceDown, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameCardID, Requirement: ActionFieldRequirementRequired}}},
	{ActionKind: ActionKindUseFirstPlayerPrivilege, ActorConstraint: ActionActorConstraintActivePlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true},
	{ActionKind: ActionKindMoveCard, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameCardID, Requirement: ActionFieldRequirementRequired}, {Field: ActionFieldNameTargetCardID, Requirement: ActionFieldRequirementRequired}}},
	{ActionKind: ActionKindSetCardMarker, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameTargetCardID, Requirement: ActionFieldRequirementRequired}, {Field: ActionFieldNameMarkerType, Requirement: ActionFieldRequirementRequired}, {Field: ActionFieldNameMarkerAmount, Requirement: ActionFieldRequirementRequired, MinimumInt: 1}}},
	{ActionKind: ActionKindRemoveCardMarker, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameTargetCardID, Requirement: ActionFieldRequirementRequired}, {Field: ActionFieldNameMarkerType, Requirement: ActionFieldRequirementRequired}, {Field: ActionFieldNameMarkerAmount, Requirement: ActionFieldRequirementRequired}}},
	{ActionKind: ActionKindPlayCard, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameCardID, Requirement: ActionFieldRequirementRequired}, {Field: ActionFieldNamePlayMode, Requirement: ActionFieldRequirementOptional}, {Field: ActionFieldNameTargetPlayerID, Requirement: ActionFieldRequirementOptional}, {Field: ActionFieldNameTargetRegionCardID, Requirement: ActionFieldRequirementRequired, SourceKinds: []CardKind{CardKindCharacter}}, {Field: ActionFieldNameTargetCardID, Requirement: ActionFieldRequirementRequired, SourceKinds: []CardKind{CardKindAsset}}}, CardKindConstraints: []ActionCardKindConstraint{{Kind: CardKindCharacter, RequiresEmptyStack: true, RequiresActionWindow: true}, {Kind: CardKindAsset, RequiresEmptyStack: true, RequiresActionWindow: true}}},
	{ActionKind: ActionKindBuildAsset, ActorConstraint: ActionActorConstraintPriorityPlayer, RequiresPriority: true, RequiresEmptyStack: true, RequiresActionWindow: true, FieldRules: []ActionFieldRule{{Field: ActionFieldNameCardID, Requirement: ActionFieldRequirementRequired}}},
}

var defaultRulesMetadata = RulesMetadata{
	ActionPolicies: cloneActionPolicies(defaultActionPolicies),
	Loyalty: LoyaltyMetadata{
		ColorAliases: cloneLoyaltyColorAliases(loyaltyColorAliases()),
	},
	Projection: ProjectionContract{
		HiddenCardPreserves: []string{"ownerId", "zone", "regionCardId", "regionOrder", "faceDown", "destroyed"},
	},
}

func DefaultRulesMetadata() RulesMetadata {
	return cloneRulesMetadata(defaultRulesMetadata)
}

func ActionPolicyForKind(kind ActionKind) (ActionPolicy, bool) {
	for _, policy := range defaultActionPolicies {
		if policy.ActionKind == kind {
			return cloneActionPolicy(policy), true
		}
	}
	return ActionPolicy{}, false
}

func cloneRulesMetadata(metadata RulesMetadata) RulesMetadata {
	return RulesMetadata{
		ActionPolicies: cloneActionPolicies(metadata.ActionPolicies),
		Loyalty: LoyaltyMetadata{
			ColorAliases: cloneLoyaltyColorAliases(metadata.Loyalty.ColorAliases),
		},
		Projection: ProjectionContract{
			HiddenCardPreserves: append([]string(nil), metadata.Projection.HiddenCardPreserves...),
		},
	}
}

func cloneActionPolicies(policies []ActionPolicy) []ActionPolicy {
	cloned := make([]ActionPolicy, 0, len(policies))
	for _, policy := range policies {
		cloned = append(cloned, cloneActionPolicy(policy))
	}
	return cloned
}

func cloneActionPolicy(policy ActionPolicy) ActionPolicy {
	return ActionPolicy{
		ActionKind:           policy.ActionKind,
		ActorConstraint:      policy.ActorConstraint,
		RequiresPriority:     policy.RequiresPriority,
		RequiresEmptyStack:   policy.RequiresEmptyStack,
		RequiresActionWindow: policy.RequiresActionWindow,
		FieldRules:           cloneActionFieldRules(policy.FieldRules),
		CardKindConstraints:  cloneActionCardKindConstraints(policy.CardKindConstraints),
	}
}

func cloneActionCardKindConstraints(constraints []ActionCardKindConstraint) []ActionCardKindConstraint {
	cloned := make([]ActionCardKindConstraint, 0, len(constraints))
	for _, c := range constraints {
		cloned = append(cloned, ActionCardKindConstraint{
			Kind:                 c.Kind,
			RequiresEmptyStack:   c.RequiresEmptyStack,
			RequiresActionWindow: c.RequiresActionWindow,
		})
	}
	return cloned
}

func cloneActionFieldRules(rules []ActionFieldRule) []ActionFieldRule {
	cloned := make([]ActionFieldRule, 0, len(rules))
	for _, rule := range rules {
		cloned = append(cloned, ActionFieldRule{
			Field:       rule.Field,
			Requirement: rule.Requirement,
			SourceKinds: slices.Clone(rule.SourceKinds),
			MinimumInt:  rule.MinimumInt,
		})
	}
	return cloned
}

func cloneLoyaltyColorAliases(aliases []LoyaltyColorAlias) []LoyaltyColorAlias {
	cloned := make([]LoyaltyColorAlias, 0, len(aliases))
	for _, alias := range aliases {
		cloned = append(cloned, LoyaltyColorAlias{
			Canonical: alias.Canonical,
			Aliases:   append([]string(nil), alias.Aliases...),
		})
	}
	return cloned
}
