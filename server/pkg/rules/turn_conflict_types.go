package rules

// Purpose: Keeps conflict-step and prompt payload models out of the core mega-types file.

type DamageAssignment struct {
	TargetCardID string `json:"targetCardId"`
	Amount       int    `json:"amount"`
}

type ConflictStage string

const (
	ConflictStageNone                      ConflictStage = ""
	ConflictStagePreInvestigationFast      ConflictStage = "pre_investigation_fast"
	ConflictStageInvestigationRewardPrompt ConflictStage = "investigation_reward_prompt"
	ConflictStagePostInvestigationFast     ConflictStage = "post_investigation_fast"
	ConflictStagePreBattleFast             ConflictStage = "pre_battle_fast"
	ConflictStageBattleDamagePrompt        ConflictStage = "battle_damage_prompt"
	ConflictStagePostBattleFast            ConflictStage = "post_battle_fast"
	ConflictStagePreInfluenceFast          ConflictStage = "pre_influence_fast"
	ConflictStagePostInfluenceFast         ConflictStage = "post_influence_fast"
)

type ConflictState struct {
	RegionOrder            int           `json:"regionOrder,omitempty"`
	RegionCardID           string        `json:"regionCardId,omitempty"`
	Stage                  ConflictStage `json:"stage,omitempty"`
	PriorityLeaderPlayerID string        `json:"priorityLeaderPlayerId,omitempty"`
	PendingPromptID        string        `json:"pendingPromptId,omitempty"`
}

type PromptKind string

const (
	PromptKindInvestigationReward PromptKind = "investigation_reward"
	PromptKindBattleDamage        PromptKind = "battle_damage"
)

type PromptState struct {
	ID                string     `json:"id"`
	Kind              PromptKind `json:"kind"`
	OwnerPlayerID     string     `json:"ownerPlayerId"`
	RegionCardID      string     `json:"regionCardId,omitempty"`
	PeekCardIDs       []string   `json:"peekCardIds,omitempty"`
	EligibleTargetIDs []string   `json:"eligibleTargetIds,omitempty"`
	RemainingAmount   int        `json:"remainingAmount,omitempty"`
	Difference        int        `json:"difference,omitempty"`
}
