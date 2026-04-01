package rules

// Purpose: Extracts action-permission legality helpers from engine orchestration.

func checkCardActionPermissionLegality(state GameState, cardID string, kind ActionKind) LegalityResult {
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

	return okLegalityResult()
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
