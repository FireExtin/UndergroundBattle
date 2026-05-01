package rules

// Purpose: Extracts action-permission legality helpers from engine orchestration.

// checkCardActionPermissionLegality 检查卡牌动作权限合法性
// actorID string // 执行动作的玩家ID，用于能力类型禁止检查
// cardID string // 目标卡牌ID
// kind ActionKind // 动作类型
func checkCardActionPermissionLegality(state GameState, actorID string, cardID string, kind ActionKind) LegalityResult {
	permission := permissionForActionKind(kind)
	if permission == "" {
		return okLegalityResult()
	}

	index := findCardIndex(state, cardID)
	if index == -1 {
		return okLegalityResult()
	}

	card := state.Board.Cards[index]
	if containsString(card.Prohibitions, permission) {
		return legalityFailure(
			ReasonCodeLegalityFailedActionProhibited,
			"rules.legality.action_prohibited",
			"board.cards.prohibitions",
			map[string]string{
				"cardId":     cardID,
				"permission": permission,
				"actionKind": string(kind),
			},
		)
	}

	if containsString(card.RequiredPermissions, permission) && !containsString(card.Permissions, permission) {
		return legalityFailure(
			ReasonCodeLegalityFailedPermissionRequired,
			"rules.legality.permission_required",
			"board.cards.permissions",
			map[string]string{
				"cardId":     cardID,
				"permission": permission,
				"actionKind": string(kind),
			},
		)
	}

	abilityKindLegality := evaluateActionAbilityKindProhibition(
		state,
		actorID,
		kind,
		card,
		BuildProhibitionChecker(state),
	)
	if !abilityKindLegality.OK {
		return abilityKindLegality
	}

	return okLegalityResult()
}

func evaluateActionAbilityKindProhibition(
	state GameState,
	actorID string,
	kind ActionKind,
	targetCard CardState,
	checker *ScopedProhibitionChecker,
) LegalityResult {
	if checker == nil {
		return okLegalityResult()
	}

	abilityKinds := abilityKindsForActionKind(kind)
	if len(abilityKinds) == 0 {
		return okLegalityResult()
	}

	targetKeywords := targetCard.EffectiveKeywords
	if len(targetKeywords) == 0 {
		targetKeywords = targetCard.PrintedKeywords
	}

	targetCategory := TargetCategory{
		ActionKinds: []ActionKind{kind},
		Condition: &TargetCondition{
			Kinds:        []CardKind{targetCard.Kind},
			Keywords:     targetKeywords,
			RegionID:     targetCard.RegionCardID,
			AbilityKinds: abilityKinds,
		},
	}

	result := checker.Check(state, actorID, targetCategory)
	if !result.Prohibited {
		return okLegalityResult()
	}

	return legalityFailure(
		ReasonCodeLegalityFailedActionProhibited,
		"rules.legality.action_prohibited",
		"board.cards",
		map[string]string{
			"cardId":              targetCard.CardID,
			"actionKind":          string(kind),
			"abilityKind":         abilityKinds[0],
			"prohibitingCardId":   result.SourceCardID,
			"prohibitingCardName": result.SourceCardName,
		},
	)
}

func abilityKindsForActionKind(kind ActionKind) []string {
	switch kind {
	case ActionKindDeclareAttack,
		ActionKindDeclareInvestigation:
		return []string{"action"}
	default:
		return nil
	}
}

func permissionForActionKind(kind ActionKind) string {
	switch kind {
	case ActionKindInspectCard:
		return "inspect"
	case ActionKindRevealCard:
		return "reveal"
	case ActionKindSetFaceDown:
		return "set_face_down"
	case ActionKindDeclareAttack:
		return "attack"
	case ActionKindDeclareInvestigation:
		return "investigate"
	default:
		return ""
	}
}
