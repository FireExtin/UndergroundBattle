package rules

import "testing"

func TestRulebookActionRights_MainStepsAdvanceFromFirstToSecondPlayer(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "rulebook-main-steps",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})

	if state.Turn.Phase.Name != PhaseMain {
		t.Fatalf("phase = %q, want %q", state.Turn.Phase.Name, PhaseMain)
	}
	if state.Turn.Phase.Step != StepFirstPlayerAction {
		t.Fatalf("step = %q, want %q", state.Turn.Phase.Step, StepFirstPlayerAction)
	}
	if state.Turn.Priority.CurrentPlayerID != "P1" {
		t.Fatalf("priority player = %q, want P1", state.Turn.Priority.CurrentPlayerID)
	}

	state = mustSubmit(t, state, Action{
		ID:      "act-main-pass-p1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-main-pass-p2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	})
	if !state.Turn.Phase.StepEnded {
		t.Fatal("expected first-player action step to end after two consecutive passes on empty stack")
	}

	state = mustSubmit(t, state, Action{
		ID:      "act-main-advance-second",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})

	if state.Turn.Phase.Name != PhaseMain {
		t.Fatalf("phase = %q, want %q after main-step advance", state.Turn.Phase.Name, PhaseMain)
	}
	if state.Turn.Phase.Step != StepSecondPlayerAction {
		t.Fatalf("step = %q, want %q", state.Turn.Phase.Step, StepSecondPlayerAction)
	}
	if state.Turn.Priority.CurrentPlayerID != "P2" {
		t.Fatalf("priority player = %q, want P2 at second-player action start", state.Turn.Priority.CurrentPlayerID)
	}
}

func TestRulebookConflict_InvestigationRewardPromptUsesDifferenceWithoutExhaustingParticipants(t *testing.T) {
	state := baseConflictPromptState()

	state = mustSubmit(t, state, Action{
		ID:      "act-conflict-pass-p1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-conflict-pass-p2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-conflict-advance-investigation",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})

	if state.Turn.Phase.Name != PhaseConflict {
		t.Fatalf("phase = %q, want %q", state.Turn.Phase.Name, PhaseConflict)
	}
	if state.Turn.Conflict.Stage != ConflictStageInvestigationRewardPrompt {
		t.Fatalf("conflict stage = %q, want %q", state.Turn.Conflict.Stage, ConflictStageInvestigationRewardPrompt)
	}
	if state.Turn.PendingPrompt == nil {
		t.Fatal("expected investigation reward prompt to be opened")
	}
	if state.Turn.PendingPrompt.Kind != PromptKindInvestigationReward {
		t.Fatalf("prompt kind = %q, want %q", state.Turn.PendingPrompt.Kind, PromptKindInvestigationReward)
	}
	if state.Turn.PendingPrompt.OwnerPlayerID != "P1" {
		t.Fatalf("prompt owner = %q, want P1", state.Turn.PendingPrompt.OwnerPlayerID)
	}
	if len(state.Turn.PendingPrompt.PeekCardIDs) != 1 {
		t.Fatalf("peek card count = %d, want 1 for icon difference 1", len(state.Turn.PendingPrompt.PeekCardIDs))
	}

	if got := cardStateByID(t, state, "p1-investigator").Exhausted; got {
		t.Fatal("investigation contest should not exhaust the participating character")
	}
	if got := cardStateByID(t, state, "p1-face-down").FaceDown; !got {
		t.Fatal("face-down participant should stay face-down during investigation resolution")
	}
}

func TestRulebookConflict_ResolveInvestigationRewardReordersAndDrawsExactlyOne(t *testing.T) {
	state := baseConflictPromptState()

	state = mustSubmit(t, state, Action{
		ID:      "act-investigation-pass-p1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-investigation-pass-p2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-investigation-open-prompt",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})

	prompt := state.Turn.PendingPrompt
	if prompt == nil || len(prompt.PeekCardIDs) != 1 {
		t.Fatalf("expected single-card investigation prompt, got %+v", prompt)
	}
	peekID := prompt.PeekCardIDs[0]

	result, err := SubmitAction(state, Action{
		ID:            "act-investigation-resolve-prompt",
		ActorID:       "P1",
		Kind:          ActionKindResolvePrompt,
		PromptID:      prompt.ID,
		TopCardIDs:    []string{},
		BottomCardIDs: []string{peekID},
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	if result.State.Turn.PendingPrompt != nil {
		t.Fatal("prompt should be cleared after investigation reward resolution")
	}
	if result.State.Turn.Conflict.Stage != ConflictStagePostInvestigationFast {
		t.Fatalf("conflict stage = %q, want %q", result.State.Turn.Conflict.Stage, ConflictStagePostInvestigationFast)
	}
	if got := countCardsInZoneByOwner(result.State, CardZoneHand, "P1"); got != 1 {
		t.Fatalf("P1 hand count = %d, want 1 after investigation reward draw", got)
	}
}

func TestRulebookConflict_ResolveBattleDamagePromptAppliesLethalCleanupAndReopensActionWindow(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "conflict-battle-damage-cleanup",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Phase.Name = PhaseConflict
	state.Turn.Conflict = ConflictState{
		RegionOrder:            1,
		RegionCardID:           "region-1",
		Stage:                  ConflictStagePreBattleFast,
		PriorityLeaderPlayerID: "P1",
	}
	resetPriorityWindow(&state.Turn, "P1", PriorityWindowAction)
	state.Board.Cards = append(state.Board.Cards,
		testRegionCard("region-1"),
		CardState{
			CardID:         "p1-battle-winner",
			DefinitionID:   "P1BAT",
			Name:           "战斗赢家",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			RegionCardID:   "region-1",
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Combat: 3, Defense: 3, Influence: 1},
			EffectiveStats: CardNumericStats{Combat: 3, Defense: 3, Influence: 1},
		},
		CardState{
			CardID:         "p2-battle-loser",
			DefinitionID:   "P2BAT",
			Name:           "战斗输家",
			Kind:           CardKindCharacter,
			OwnerID:        "P2",
			Zone:           CardZoneTable,
			RegionCardID:   "region-1",
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Combat: 1, Defense: 2, Influence: 2},
			EffectiveStats: CardNumericStats{Combat: 1, Defense: 2, Influence: 2},
		},
	)
	refreshAllRegionControl(&state)

	state = mustSubmit(t, state, Action{
		ID:      "act-battle-pass-p1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-battle-pass-p2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-battle-open-prompt",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})

	prompt := state.Turn.PendingPrompt
	if prompt == nil {
		t.Fatal("expected battle damage prompt to be opened")
	}
	if prompt.RemainingAmount != 2 {
		t.Fatalf("remaining damage = %d, want 2", prompt.RemainingAmount)
	}

	result, err := SubmitAction(state, Action{
		ID:       "act-battle-resolve-prompt",
		ActorID:  "P1",
		Kind:     ActionKindResolvePrompt,
		PromptID: prompt.ID,
		DamageAssignments: []DamageAssignment{
			{TargetCardID: "p2-battle-loser", Amount: 2},
		},
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	loser := cardStateByID(t, result.State, "p2-battle-loser")
	if !loser.Destroyed {
		t.Fatal("battle damage should move lethal target to destroyed state")
	}
	if loser.Zone != CardZoneDiscard {
		t.Fatalf("battle damage target zone = %q, want %q", loser.Zone, CardZoneDiscard)
	}
	if loser.RegionCardID != "" {
		t.Fatalf("battle damage target region = %q, want empty after leaving play", loser.RegionCardID)
	}
	if result.State.Turn.PendingPrompt != nil {
		t.Fatal("battle damage prompt should be cleared after resolution")
	}
	if result.State.Turn.Conflict.Stage != ConflictStagePostBattleFast {
		t.Fatalf("conflict stage = %q, want %q", result.State.Turn.Conflict.Stage, ConflictStagePostBattleFast)
	}
	if result.State.Turn.Priority.CurrentPlayerID != "P1" {
		t.Fatalf("priority player = %q, want P1 after battle prompt resolution", result.State.Turn.Priority.CurrentPlayerID)
	}
	if result.State.Turn.Priority.WindowKind != PriorityWindowAction {
		t.Fatalf("priority window = %q, want %q", result.State.Turn.Priority.WindowKind, PriorityWindowAction)
	}
	region := cardStateByID(t, result.State, "region-1")
	if region.ControllerID != "P1" {
		t.Fatalf("region controller = %q, want %q after lethal cleanup recalculation", region.ControllerID, "P1")
	}
}

func TestRulebookConflict_InvestigationPromptHidesPeekCardsFromOpponentAndSpectator(t *testing.T) {
	state := baseConflictPromptState()

	state = mustSubmit(t, state, Action{
		ID:      "act-prompt-hide-pass-p1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})
	state = mustSubmit(t, state, Action{
		ID:      "act-prompt-hide-pass-p2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	})

	result, err := SubmitAction(state, Action{
		ID:      "act-prompt-hide-open",
		ActorID: "P1",
		Kind:    ActionKindAdvancePhase,
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}

	p1Prompt := result.Views.Players["P1"].Turn.PendingPrompt
	if p1Prompt == nil || len(p1Prompt.PeekCardIDs) != 1 {
		t.Fatalf("owner prompt = %+v, want one visible peek card", p1Prompt)
	}

	p2Prompt := result.Views.Players["P2"].Turn.PendingPrompt
	if p2Prompt == nil {
		t.Fatal("opponent should still see prompt shell")
	}
	if len(p2Prompt.PeekCardIDs) != 0 {
		t.Fatalf("opponent peek cards = %v, want hidden", p2Prompt.PeekCardIDs)
	}

	spectatorPrompt := result.Views.Spectator.Turn.PendingPrompt
	if spectatorPrompt == nil {
		t.Fatal("spectator should still see prompt shell")
	}
	if len(spectatorPrompt.PeekCardIDs) != 0 {
		t.Fatalf("spectator peek cards = %v, want hidden", spectatorPrompt.PeekCardIDs)
	}
}

func TestRevealFaceDown_UsesStackAndPreservesExhaustedState(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "reveal-face-down-stack",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Resources["P1"] = PlayerResourceState{Current: 2, Max: 2}
	state.Board.Cards = append(state.Board.Cards, CardState{
		CardID:         "p1-face-down-reveal",
		DefinitionID:   "JC006",
		Name:           "常春藤派学者",
		Cost:           1,
		Kind:           CardKindCharacter,
		OwnerID:        "P1",
		Zone:           CardZoneTable,
		RegionCardID:   "region-1",
		VisibleToOwner: true,
		FaceDown:       true,
		Revealed:       false,
		Exhausted:      true,
		PrintedStats:   CardNumericStats{Influence: 2},
		EffectiveStats: CardNumericStats{Influence: 2},
	})
	state.Board.Cards = append(state.Board.Cards, testRegionCard("region-1"))

	queued, err := SubmitAction(state, Action{
		ID:      "act-reveal-face-down",
		ActorID: "P1",
		Kind:    ActionKindRevealFaceDown,
		CardID:  "p1-face-down-reveal",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}
	if queued.Event.Kind != EventKindOperationEnqueued {
		t.Fatalf("event kind = %q, want %q", queued.Event.Kind, EventKindOperationEnqueued)
	}
	if queued.State.Turn.Priority.WindowKind != PriorityWindowResponse {
		t.Fatalf("priority window = %q, want %q", queued.State.Turn.Priority.WindowKind, PriorityWindowResponse)
	}
	if got := queued.State.Turn.Resources["P1"].Current; got != 1 {
		t.Fatalf("resource current = %d, want 1 after paying reveal cost", got)
	}

	resolved := mustSubmit(t, queued.State, Action{
		ID:      "act-reveal-pass-p2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	})
	resolved = mustSubmit(t, resolved, Action{
		ID:      "act-reveal-pass-p1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})

	card := cardStateByID(t, resolved, "p1-face-down-reveal")
	if card.FaceDown {
		t.Fatal("card should be face up after stack resolution")
	}
	if !card.Revealed {
		t.Fatal("card should be revealed after stack resolution")
	}
	if !card.Exhausted {
		t.Fatal("revealed card should preserve exhausted state")
	}
}

func TestActivateAbility_JC003QuickAbilityPaysCostsAndExhaustsTargetThroughStack(t *testing.T) {
	state := NewGameState(InitialStateConfig{
		GameID:         "activate-ability-jc003",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Phase.Name = PhaseConflict
	state.Turn.Conflict = ConflictState{
		RegionOrder:            1,
		RegionCardID:           "region-1",
		Stage:                  ConflictStagePreInvestigationFast,
		PriorityLeaderPlayerID: "P1",
	}
	state.Turn.Resources["P1"] = PlayerResourceState{Current: 2, Max: 2}
	resetPriorityWindow(&state.Turn, "P1", PriorityWindowAction)
	state.Board.Cards = append(state.Board.Cards,
		testRegionCard("region-1"),
		CardState{
			CardID:         "p1-veil-guard",
			DefinitionID:   "JC003",
			Name:           "帷幕护卫",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			RegionCardID:   "region-1",
			VisibleToOwner: true,
			Revealed:       true,
		},
		CardState{
			CardID:         "p2-target-character",
			DefinitionID:   "TARGET",
			Name:           "目标角色",
			Kind:           CardKindCharacter,
			OwnerID:        "P2",
			Zone:           CardZoneTable,
			RegionCardID:   "region-1",
			VisibleToOwner: true,
			Revealed:       true,
		},
	)

	queued, err := SubmitAction(state, Action{
		ID:           "act-activate-jc003",
		ActorID:      "P1",
		Kind:         ActionKindActivateAbility,
		CardID:       "p1-veil-guard",
		AbilityID:    "JC003.quick.exhaust_target",
		TargetCardID: "p2-target-character",
	})
	if err != nil {
		t.Fatalf("SubmitAction returned error: %v", err)
	}
	if queued.Event.Kind != EventKindOperationEnqueued {
		t.Fatalf("event kind = %q, want %q", queued.Event.Kind, EventKindOperationEnqueued)
	}
	if !cardStateByID(t, queued.State, "p1-veil-guard").Exhausted {
		t.Fatal("source card should be exhausted as part of ability cost payment")
	}
	if got := queued.State.Turn.Resources["P1"].Current; got != 0 {
		t.Fatalf("resource current = %d, want 0 after paying quick ability cost", got)
	}

	resolved := mustSubmit(t, queued.State, Action{
		ID:      "act-activate-pass-p2",
		ActorID: "P2",
		Kind:    ActionKindPassPriority,
	})
	resolved = mustSubmit(t, resolved, Action{
		ID:      "act-activate-pass-p1",
		ActorID: "P1",
		Kind:    ActionKindPassPriority,
	})

	if !cardStateByID(t, resolved, "p2-target-character").Exhausted {
		t.Fatal("target should become exhausted after quick ability resolution")
	}
}

func baseConflictPromptState() GameState {
	state := NewGameState(InitialStateConfig{
		GameID:         "conflict-investigation-prompt",
		ActivePlayerID: "P1",
		PlayerIDs:      []string{"P1", "P2"},
	})
	state.Turn.Phase.Name = PhaseConflict
	state.Turn.Conflict = ConflictState{
		RegionOrder:            1,
		RegionCardID:           "region-1",
		Stage:                  ConflictStagePreInvestigationFast,
		PriorityLeaderPlayerID: "P1",
	}
	resetPriorityWindow(&state.Turn, "P1", PriorityWindowAction)
	state.Board.Cards = append(state.Board.Cards,
		testRegionCard("region-1"),
		CardState{
			CardID:         "p1-investigator",
			DefinitionID:   "P1INV",
			Name:           "调查员A",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			RegionCardID:   "region-1",
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Investigation: 2},
			EffectiveStats: CardNumericStats{Investigation: 2},
		},
		CardState{
			CardID:         "p1-face-down",
			DefinitionID:   "P1FD",
			Name:           "暗藏者",
			Kind:           CardKindCharacter,
			OwnerID:        "P1",
			Zone:           CardZoneTable,
			RegionCardID:   "region-1",
			VisibleToOwner: true,
			FaceDown:       true,
			Revealed:       false,
			PrintedStats:   CardNumericStats{Investigation: 9, Combat: 9, Influence: 9},
			EffectiveStats: CardNumericStats{Investigation: 9, Combat: 9, Influence: 9},
		},
		CardState{
			CardID:         "p2-investigator",
			DefinitionID:   "P2INV",
			Name:           "调查员B",
			Kind:           CardKindCharacter,
			OwnerID:        "P2",
			Zone:           CardZoneTable,
			RegionCardID:   "region-1",
			VisibleToOwner: true,
			Revealed:       true,
			PrintedStats:   CardNumericStats{Investigation: 1},
			EffectiveStats: CardNumericStats{Investigation: 1},
		},
		CardState{
			CardID:         "p1-deck-top-1",
			DefinitionID:   "D1",
			Name:           "牌库顶1",
			OwnerID:        "P1",
			Zone:           CardZoneDeck,
			VisibleToOwner: true,
		},
		CardState{
			CardID:         "p1-deck-top-2",
			DefinitionID:   "D2",
			Name:           "牌库顶2",
			OwnerID:        "P1",
			Zone:           CardZoneDeck,
			VisibleToOwner: true,
		},
	)
	return state
}
