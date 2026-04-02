package rules

import "testing"

func TestProjectionCarriesRulesMetadata(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-rules-metadata",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	views := NewProjectionEngine().Generate(state)
	metadata := views.Players["P1"].RulesMetadata

	playCardPolicy, ok := findActionPolicy(metadata.ActionPolicies, ActionKindPlayCard)
	if !ok {
		t.Fatal("play_card action policy missing from projected metadata")
	}
	if playCardPolicy.ActorConstraint != ActionActorConstraintPriorityPlayer {
		t.Fatalf("play_card actor constraint = %q, want %q", playCardPolicy.ActorConstraint, ActionActorConstraintPriorityPlayer)
	}
	if !hasActionFieldRule(playCardPolicy, ActionFieldNameCardID, ActionFieldRequirementRequired, nil) {
		t.Fatal("play_card policy missing required cardId field rule")
	}
	if !hasActionFieldRule(playCardPolicy, ActionFieldNameTargetRegionCardID, ActionFieldRequirementRequired, []CardKind{CardKindCharacter}) {
		t.Fatal("play_card policy missing character targetRegionCardId field rule")
	}
	if !hasActionFieldRule(playCardPolicy, ActionFieldNameTargetCardID, ActionFieldRequirementRequired, []CardKind{CardKindAsset}) {
		t.Fatal("play_card policy missing asset targetCardId field rule")
	}

	privilegePolicy, ok := findActionPolicy(metadata.ActionPolicies, ActionKindUseFirstPlayerPrivilege)
	if !ok {
		t.Fatal("use_first_player_privilege policy missing")
	}
	if privilegePolicy.ActorConstraint != ActionActorConstraintActivePlayer {
		t.Fatalf("privilege actor constraint = %q, want %q", privilegePolicy.ActorConstraint, ActionActorConstraintActivePlayer)
	}

	if len(metadata.Loyalty.ColorAliases) == 0 {
		t.Fatal("loyalty color aliases missing from projected metadata")
	}
	if metadata.Payment.Mode != PaymentModePrototype {
		t.Fatalf("payment mode metadata = %q, want %q", metadata.Payment.Mode, PaymentModePrototype)
	}
	if !containsString(metadata.Projection.HiddenCardPreserves, "regionCardId") {
		t.Fatalf("projection contract hiddenCardPreserves = %#v, want regionCardId", metadata.Projection.HiddenCardPreserves)
	}
}

func TestCheckLegalityUsesActionPolicyActivePlayerConstraint(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "game-action-policy-active-player",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Priority.CurrentPlayerID = "P2"
	state.Board.Cards = []CardState{
		{
			CardID:            "region-policy-tie",
			Name:              "Region",
			Kind:              CardKindRegion,
			Zone:              CardZoneTable,
			Revealed:          true,
			InfluenceByPlayer: map[string]int{"P1": 1, "P2": 1},
		},
	}

	legality := CheckLegality(state, Action{
		ID:      "act-action-policy-active-player",
		ActorID: "P2",
		Kind:    ActionKindUseFirstPlayerPrivilege,
	})
	if legality.OK {
		t.Fatal("legality unexpectedly succeeded for non-active player")
	}
	if legality.Hook != "turn.activePlayerId" {
		t.Fatalf("legality hook = %q, want %q", legality.Hook, "turn.activePlayerId")
	}
	if legality.MessageKey != "rules.legality.active_player_required" {
		t.Fatalf("message key = %q, want %q", legality.MessageKey, "rules.legality.active_player_required")
	}
}

func findActionPolicy(policies []ActionPolicy, kind ActionKind) (ActionPolicy, bool) {
	for _, policy := range policies {
		if policy.ActionKind == kind {
			return policy, true
		}
	}
	return ActionPolicy{}, false
}

func hasActionFieldRule(policy ActionPolicy, field ActionFieldName, requirement ActionFieldRequirement, sourceKinds []CardKind) bool {
	for _, rule := range policy.FieldRules {
		if rule.Field != field || rule.Requirement != requirement {
			continue
		}
		if len(sourceKinds) == 0 && len(rule.SourceKinds) == 0 {
			return true
		}
		if sameCardKinds(rule.SourceKinds, sourceKinds) {
			return true
		}
	}
	return false
}

func sameCardKinds(left []CardKind, right []CardKind) bool {
	if len(left) != len(right) {
		return false
	}
	for index := range left {
		if left[index] != right[index] {
			return false
		}
	}
	return true
}
