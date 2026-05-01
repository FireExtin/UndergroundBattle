package rules

// Purpose: Defines the serializable state, log, and pipeline data structures for the minimal Go rules kernel.

// ActionKind names the minimal actions supported by the skeleton rules pipeline.
type ActionKind string

const (
	ActionKindAdvancePhase            ActionKind = "advance_phase"
	ActionKindRevealCard              ActionKind = "reveal_card"
	ActionKindInspectCard             ActionKind = "inspect_card"
	ActionKindPassPriority            ActionKind = "pass_priority"
	ActionKindQueueOperation          ActionKind = "queue_operation"
	ActionKindDeclareAttack           ActionKind = "declare_attack"
	ActionKindDeclareInvestigation    ActionKind = "declare_investigation"
	ActionKindResolveTopStack         ActionKind = "resolve_top_stack"
	ActionKindRollSeededRandom        ActionKind = "roll_seeded_random"
	ActionKindSetMarker               ActionKind = "set_marker"    // 设置标记物
	ActionKindRemoveMarker            ActionKind = "remove_marker" // 移除标记物
	ActionKindSetFaceDown             ActionKind = "set_face_down" // 设置卡牌为面朝下
	ActionKindUseFirstPlayerPrivilege ActionKind = "use_first_player_privilege"
	ActionKindMoveCard                ActionKind = "move_card"
	ActionKindSetCardMarker           ActionKind = "set_card_marker"
	ActionKindRemoveCardMarker        ActionKind = "remove_card_marker"
)

// OperationKind names the minimal operation types built from actions.
type OperationKind string

const (
	OperationKindAdvancePhase            OperationKind = "advance_phase"
	OperationKindRevealCard              OperationKind = "reveal_card"
	OperationKindInspectCard             OperationKind = "inspect_card"
	OperationKindPassPriority            OperationKind = "pass_priority"
	OperationKindStackedEffect           OperationKind = "stacked_effect"
	OperationKindCardEffect              OperationKind = "card_effect"
	OperationKindDeclareAttack           OperationKind = "declare_attack"
	OperationKindDeclareInvestigation    OperationKind = "declare_investigation"
	OperationKindResolveTopStack         OperationKind = "resolve_top_stack"
	OperationKindRollRandom              OperationKind = "roll_seeded_random"
	OperationKindSetMarker               OperationKind = "set_marker"
	OperationKindRemoveMarker            OperationKind = "remove_marker"
	OperationKindSetFaceDown             OperationKind = "set_face_down"
	OperationKindUseFirstPlayerPrivilege OperationKind = "use_first_player_privilege"
	OperationKindMoveCard                OperationKind = "move_card"
	OperationKindSetCardMarker           OperationKind = "set_card_marker"
	OperationKindRemoveCardMarker        OperationKind = "remove_card_marker"
)

// OperationStatus describes whether an operation is pending on the stack or already resolved.
type OperationStatus string

const (
	OperationStatusBuilt    OperationStatus = "built"
	OperationStatusPending  OperationStatus = "pending"
	OperationStatusResolved OperationStatus = "resolved"
)

// PaymentKind identifies the minimal authoritative payment source consumed by an action.
type PaymentKind string

const (
	PaymentKindMarker PaymentKind = "marker"
)

// PaymentRecord stores one minimal, replayable payment fact.
type PaymentRecord struct {
	Kind       PaymentKind `json:"kind"`
	MarkerType string      `json:"markerType,omitempty"`
	Amount     int         `json:"amount,omitempty"`
}

// EventKind identifies the committed state transition emitted by the rules pipeline.
type EventKind string

const (
	EventKindOperationEnqueued        EventKind = "operation_enqueued"
	EventKindOperationResolved        EventKind = "operation_resolved"
	EventKindPhaseAdvanced            EventKind = "phase_advanced"
	EventKindCardInspected            EventKind = "card_inspected"
	EventKindCardRevealed             EventKind = "card_revealed"
	EventKindPriorityPassed           EventKind = "priority_passed"
	EventKindRandomGenerated          EventKind = "random_generated"
	EventKindStepEnded                EventKind = "step_ended"
	EventKindAttackDeclared           EventKind = "attack_declared"
	EventKindDamageApplied            EventKind = "damage_applied"
	EventKindCardDestroyed            EventKind = "card_destroyed"
	EventKindInvestigationApplied     EventKind = "investigation_applied"
	EventKindMarkerSet                EventKind = "marker_set"
	EventKindMarkerRemoved            EventKind = "marker_removed"
	EventKindFaceDownSet              EventKind = "face_down_set"
	EventKindShieldConsumed           EventKind = "shield_consumed"
	EventKindFirstPlayerPrivilegeUsed EventKind = "first_player_privilege_used"
	EventKindCardMoved                EventKind = "card_moved"
	EventKindCardMarkerSet            EventKind = "card_marker_set"
	EventKindCardMarkerRemoved        EventKind = "card_marker_removed"
)

// PhaseName identifies the current minimal turn phase.
type PhaseName string

const (
	PhaseMain PhaseName = "main"
	PhaseEnd  PhaseName = "end"
)

// StepName identifies the finer-grained step inside one phase.
type StepName string

const (
	StepAction StepName = "action"
	StepEnded  StepName = "ended"
)

// PriorityWindowKind identifies whether players are in a normal action window, response window, or closed step.
type PriorityWindowKind string

const (
	PriorityWindowAction   PriorityWindowKind = "action"
	PriorityWindowResponse PriorityWindowKind = "response"
	PriorityWindowClosed   PriorityWindowKind = "closed"
)

// CardExecutionKind identifies whether a card operation should route through pure DSL handling or a script entry.
type CardExecutionKind string

const (
	CardExecutionDSL    CardExecutionKind = "dsl"
	CardExecutionScript CardExecutionKind = "script"
)

// ContinuousLayer identifies the minimal continuous-effect application bucket.
type ContinuousLayer string

const (
	LayerProhibition ContinuousLayer = "prohibition"
	LayerPermission  ContinuousLayer = "permission"
	LayerCost        ContinuousLayer = "cost"
	LayerNumeric     ContinuousLayer = "numeric"
	LayerActionQuota ContinuousLayer = "action_quota"
)

// MatchStatus identifies whether the current match is still active or already terminal.
type MatchStatus string

const (
	MatchStatusActive   MatchStatus = "active"
	MatchStatusFinished MatchStatus = "finished"
)

// MatchEndReason identifies the minimal reason the current match reached a terminal state.
type MatchEndReason string

const (
	MatchEndReasonNone             MatchEndReason = ""
	MatchEndReasonVictoryThreshold MatchEndReason = "victory_threshold"
	MatchEndReasonDeckOut          MatchEndReason = "deck_out"
	MatchEndReasonDeckOutDraw      MatchEndReason = "deck_out_draw"
)

const (
	markerTypeResource = "resource"
)

// CardNumericStats stores printed and effective numeric values used by the minimal rules kernel.
type CardNumericStats struct {
	Combat        int `json:"combat"`
	Defense       int `json:"defense"`
	Influence     int `json:"influence"`
	Investigation int `json:"investigation"`
}

// CardCounters stores direct board counters that are not derived through continuous layers.
type CardCounters struct {
	Damage    int `json:"damage"`
	Influence int `json:"influence"`
	Shield    int `json:"shield,omitempty"`
}

// ContinuousEffect is the serializable minimal continuous-effect record stored in authoritative state.
type ContinuousEffect struct {
	ID                string          `json:"id"`
	SourceOperationID string          `json:"sourceOperationId,omitempty"`
	SourceCardID      string          `json:"sourceCardId,omitempty"`
	AttachmentID      string          `json:"attachmentId,omitempty"`
	BindingEntityID   string          `json:"bindingEntityId,omitempty"`
	ControllerID      string          `json:"controllerId,omitempty"`
	TargetCardID      string          `json:"targetCardId,omitempty"`
	Layer             ContinuousLayer `json:"layer"`
	EffectKind        string          `json:"effectKind"`
	DurationKind      string          `json:"durationKind"`
	ExpiresAtTurn     int             `json:"expiresAtTurn,omitempty"`
	DependencyKey     []string        `json:"dependencyKey,omitempty"`
	Timestamp         int64           `json:"timestamp"`
	Stat              string          `json:"stat,omitempty"`
	Amount            int             `json:"amount,omitempty"`
	Keyword           string          `json:"keyword,omitempty"`
	Permission        string          `json:"permission,omitempty"`
}

// Attachment represents an attachment relationship between two cards.
type Attachment struct {
	ID                 string `json:"id"`
	SourceCardID       string `json:"sourceCardId,omitempty"`
	SourceDefinitionID string `json:"sourceDefinitionId,omitempty"`
	SourceOperationID  string `json:"sourceOperationId,omitempty"`
	TargetCardID       string `json:"targetCardId"`
	HostCardID         string `json:"hostCardId,omitempty"` // 宿主卡片ID，用于宿主离场联动
	CreatedAtRevision  int    `json:"createdAtRevision"`
}

// AttachmentRegistry tracks all active attachments.
type AttachmentRegistry struct {
	Active           []Attachment `json:"active"`
	NextAttachmentID int          `json:"nextAttachmentId"`
}

// ContinuousEffectRegistry stores active effects plus recalculation bookkeeping.
type ContinuousEffectRegistry struct {
	Active                 []ContinuousEffect `json:"active"`
	PendingRecalculation   bool               `json:"pendingRecalculation"`
	InProgress             bool               `json:"inProgress"`
	FullRecalculationCount int                `json:"fullRecalculationCount"`
	CycleGuardTrips        int                `json:"cycleGuardTrips"`
	LastAppliedRevision    int                `json:"lastAppliedRevision,omitempty"`
	NextTimestamp          int64              `json:"nextTimestamp"`
}

// InitialStateConfig configures the deterministic initial game state used by tests and replay.
type InitialStateConfig struct {
	GameID         string   `json:"gameId"`
	ActivePlayerID string   `json:"activePlayerId"`
	PlayerIDs      []string `json:"playerIds"`
	Seed           uint64   `json:"seed"`
}

// GameState is the authoritative serializable state snapshot.
type GameState struct {
	GameID   string       `json:"gameId"`
	Players  []string     `json:"players"`
	Revision Revision     `json:"revision"`
	Match    MatchState   `json:"match"`
	Turn     TurnState    `json:"turn"`
	Board    BoardState   `json:"board"`
	Score    ScoreState   `json:"score"`
	History  HistoryState `json:"history"`
	RNG      RNGState     `json:"rng"`
}

// FullState is the authoritative server-only truth source. Clients must consume projections instead of this structure.
type FullState = GameState

// Action is the player intent submitted into the authoritative pipeline.
type Action struct {
	ID             string     `json:"id"`
	ActorID        string     `json:"actorId"`
	Kind           ActionKind `json:"kind"`
	CardID         string     `json:"cardId,omitempty"`
	TargetPlayerID string     `json:"targetPlayerId,omitempty"`
	TargetCardID   string     `json:"targetCardId,omitempty"`
	MarkerType     string     `json:"markerType,omitempty"`
	MarkerAmount   int        `json:"markerAmount,omitempty"`
	OperationLabel string     `json:"operationLabel,omitempty"`
	RandomMax      int        `json:"randomMax,omitempty"`
}

// EffectSpec is the executable subset of the shared CardLogic effect payload copied into Go-side operations.
type EffectSpec struct {
	Kind      string `json:"kind"`
	TargetRef string `json:"targetRef"`
	Amount    *int   `json:"amount,omitempty"`
	Stat      string `json:"stat,omitempty"`
	Keyword   string `json:"keyword,omitempty"`
}

// CardOperationSource records which shared CardLogic fixture produced an operation.
type CardOperationSource struct {
	CardID            string            `json:"cardId"`
	CardName          string            `json:"cardName"`
	SourcePath        string            `json:"sourcePath"`
	BasicType         string            `json:"basicType,omitempty"`
	LogicID           string            `json:"logicId"`
	Speed             string            `json:"speed"`
	TargetKinds       []string          `json:"targetKinds"`
	RequiresStack     bool              `json:"requiresStack"`
	ExecutionKind     CardExecutionKind `json:"executionKind"`
	DurationKind      string            `json:"durationKind"`
	ScriptID          *string           `json:"scriptId,omitempty"`
	RequiresScript    bool              `json:"requiresScript"`
	PureDSLExecutable bool              `json:"pureDSLExecutable"`
	Effects           []EffectSpec      `json:"effects"`
	EffectKinds       []string          `json:"effectKinds"`
}

// Operation is the normalized work item built from an action before it is queued or resolved.
type Operation struct {
	ID             string               `json:"id"`
	ActionID       string               `json:"actionId"`
	ActorID        string               `json:"actorId"`
	Kind           OperationKind        `json:"kind"`
	Status         OperationStatus      `json:"status"`
	RequiresStack  bool                 `json:"requiresStack"`
	CardID         string               `json:"cardId,omitempty"`
	TargetPlayerID string               `json:"targetPlayerId,omitempty"`
	TargetCardID   string               `json:"targetCardId,omitempty"`
	MarkerType     string               `json:"markerType,omitempty"`
	MarkerAmount   int                  `json:"markerAmount,omitempty"`
	Label          string               `json:"label,omitempty"`
	RandomMax      int                  `json:"randomMax,omitempty"`
	NextPhase      PhaseName            `json:"nextPhase,omitempty"`
	Payment        *PaymentRecord       `json:"payment,omitempty"`
	Source         *CardOperationSource `json:"source,omitempty"`
}

// Event records the committed result of a single action pipeline run.
type Event struct {
	ID               string             `json:"id"`
	ActionID         string             `json:"actionId"`
	OperationID      string             `json:"operationId"`
	Kind             EventKind          `json:"kind"`
	RevisionNumber   int                `json:"revisionNumber"`
	Phase            PhaseName          `json:"phase,omitempty"`
	Step             StepName           `json:"step,omitempty"`
	PriorityPlayerID string             `json:"priorityPlayerId,omitempty"`
	PriorityWindow   PriorityWindowKind `json:"priorityWindow,omitempty"`
	PassCount        int                `json:"passCount,omitempty"`
	ResolvedTargetID string             `json:"resolvedTargetId,omitempty"`
	SourceCardID     string             `json:"sourceCardId,omitempty"`
	TargetCardID     string             `json:"targetCardId,omitempty"`
	AppliedAmount    int                `json:"appliedAmount,omitempty"`
	DestroyedCardID  string             `json:"destroyedCardId,omitempty"`
	MarkerType       string             `json:"markerType,omitempty"`
	MarkerAmount     int                `json:"markerAmount,omitempty"`
	StackDepth       int                `json:"stackDepth"`
	RandomValue      *int               `json:"randomValue,omitempty"`
	Payment          *PaymentRecord     `json:"payment,omitempty"`
	StepEnded        bool               `json:"stepEnded,omitempty"`
	TargetPlayerID   string             `json:"targetPlayerId,omitempty"` // 目标玩家ID
}

// Revision monotonically identifies each committed state transition.
type Revision struct {
	Number      int    `json:"number"`
	ActionID    string `json:"actionId,omitempty"`
	OperationID string `json:"operationId,omitempty"`
	EventID     string `json:"eventId,omitempty"`
}

// HistoryState stores the replayable logs produced by the authoritative pipeline.
type HistoryState struct {
	Actions    []Action    `json:"actions"`
	Operations []Operation `json:"operations"`
	Events     []Event     `json:"events"`
	Revisions  []Revision  `json:"revisions"`
}

// ScoreState stores the public score race used by the first minimal playable loop.
type ScoreState struct {
	ByPlayer         map[string]int `json:"byPlayer"`
	VictoryThreshold int            `json:"victoryThreshold"`
	WinnerPlayerID   string         `json:"winnerPlayerId,omitempty"`
}

// MatchState stores the explicit lifecycle status of one authoritative match.
type MatchState struct {
	Status             MatchStatus    `json:"status"`
	EndReason          MatchEndReason `json:"endReason,omitempty"`
	WinnerPlayerID     string         `json:"winnerPlayerId,omitempty"`
	FinishedAtRevision int            `json:"finishedAtRevision,omitempty"`
}

// TurnState captures the minimum turn owner, priority owner, and phase needed by the skeleton rules kernel.
type TurnState struct {
	TurnNumber               int           `json:"turnNumber"`
	ActivePlayerID           string        `json:"activePlayerId"`
	PriorityPlayerID         string        `json:"priorityPlayerId"`
	FirstPlayerPrivilegeUsed bool          `json:"firstPlayerPrivilegeUsed,omitempty"`
	Priority                 PriorityState `json:"priority"`
	Phase                    PhaseState    `json:"phase"`
}

// PriorityState captures the current action holder plus consecutive pass tracking.
type PriorityState struct {
	CurrentPlayerID    string             `json:"currentPlayerId"`
	PassCount          int                `json:"passCount"`
	LastPassedPlayerID string             `json:"lastPassedPlayerId,omitempty"`
	WindowKind         PriorityWindowKind `json:"windowKind"`
}

// PhaseState captures the current phase plus whether new stack objects may be created.
type PhaseState struct {
	Name        PhaseName `json:"name"`
	Step        StepName  `json:"step"`
	AllowsStack bool      `json:"allowsStack"`
	StepEnded   bool      `json:"stepEnded"`
}

// BoardState stores the minimal stack, resolved operations, and deterministic random outputs.
type BoardState struct {
	Stack         []Operation              `json:"stack"`
	Resolved      []Operation              `json:"resolved"`
	RandomResults []RandomResult           `json:"randomResults"`
	Cards         []CardState              `json:"cards"`
	Continuous    ContinuousEffectRegistry `json:"continuous"`
	Attachments   AttachmentRegistry       `json:"attachments"`
	Markers       MarkerRegistry           `json:"markers"` // 秘社标记物注册表
	CardMarkers   CardMarkerRegistry       `json:"cardMarkers"`
}

// RandomResult records each deterministic RNG draw committed into history-visible board state.
type RandomResult struct {
	ActionID    string `json:"actionId"`
	OperationID string `json:"operationId"`
	DrawIndex   int    `json:"drawIndex"`
	Value       int    `json:"value"`
}

// RNGState is the seed-backed random source required for deterministic replay.
type RNGState struct {
	Seed      uint64 `json:"seed"`
	State     uint64 `json:"state"`
	DrawCount int    `json:"drawCount"`
}

// SubmitResult exposes the committed artifacts produced by a successful pipeline run.
type SubmitResult struct {
	State     GameState        `json:"state"`
	Operation Operation        `json:"operation"`
	Event     Event            `json:"event"`
	Revision  Revision         `json:"revision"`
	Accepted  ActionAccepted   `json:"accepted"`
	Patched   StatePatched     `json:"patched"`
	Views     ProjectionBundle `json:"views"`
	Dispatch  DispatchBatch    `json:"dispatch"`
}

// DispatchAudienceKind identifies the connection-facing audience category for one dispatch envelope.
type DispatchAudienceKind string

const (
	DispatchAudiencePlayer    DispatchAudienceKind = "player"
	DispatchAudienceSpectator DispatchAudienceKind = "spectator"
)

// DispatchPayloadKind identifies the protocol payload carried by one per-client envelope.
type DispatchPayloadKind string

const (
	DispatchPayloadActionAccepted DispatchPayloadKind = "ActionAccepted"
	DispatchPayloadActionRejected DispatchPayloadKind = "ActionRejected"
	DispatchPayloadStatePatched   DispatchPayloadKind = "StatePatched"
)

// DispatchTarget identifies which audience should receive a given protocol envelope.
type DispatchTarget struct {
	Kind DispatchAudienceKind `json:"kind"`
	ID   string               `json:"id,omitempty"`
}

// ClientDispatch is one per-client protocol envelope ready for transport-layer delivery.
type ClientDispatch struct {
	Kind           DispatchPayloadKind `json:"kind"`
	Target         DispatchTarget      `json:"target"`
	ActionAccepted *ActionAccepted     `json:"actionAccepted,omitempty"`
	ActionRejected *ActionRejected     `json:"actionRejected,omitempty"`
	StatePatched   *StatePatched       `json:"statePatched,omitempty"`
}

// DispatchBatch is the authoritative per-client output produced from one action result.
type DispatchBatch struct {
	Revision Revision         `json:"revision"`
	Messages []ClientDispatch `json:"messages"`
}

// ReasonCode is the machine-readable rejection code shared by the authoritative rules kernel.
type ReasonCode string

const (
	ReasonCodeLegalityFailedActionIDMissing        ReasonCode = "LEGALITY_FAILED_ACTION_ID_MISSING"
	ReasonCodeLegalityFailedActorIDMissing         ReasonCode = "LEGALITY_FAILED_ACTOR_ID_MISSING"
	ReasonCodeLegalityFailedActionIDDuplicate      ReasonCode = "LEGALITY_FAILED_ACTION_ID_DUPLICATE"
	ReasonCodeLegalityFailedNotYourPriority        ReasonCode = "LEGALITY_FAILED_NOT_YOUR_PRIORITY"
	ReasonCodeLegalityFailedStackNotEmpty          ReasonCode = "LEGALITY_FAILED_STACK_NOT_EMPTY"
	ReasonCodeLegalityFailedStackClosed            ReasonCode = "LEGALITY_FAILED_STACK_CLOSED"
	ReasonCodeLegalityFailedActionWindowRequired   ReasonCode = "LEGALITY_FAILED_ACTION_WINDOW_REQUIRED"
	ReasonCodeLegalityFailedResponseWindowRequired ReasonCode = "LEGALITY_FAILED_RESPONSE_WINDOW_REQUIRED"
	ReasonCodeLegalityFailedOperationLabelMissing  ReasonCode = "LEGALITY_FAILED_OPERATION_LABEL_MISSING"
	ReasonCodeLegalityFailedPermissionRequired     ReasonCode = "LEGALITY_FAILED_PERMISSION_REQUIRED"
	ReasonCodeLegalityFailedActionProhibited       ReasonCode = "LEGALITY_FAILED_ACTION_PROHIBITED"
	ReasonCodeTargetFailedMissing                  ReasonCode = "TARGET_FAILED_MISSING"
	ReasonCodeTargetFailedProhibited               ReasonCode = "TARGET_FAILED_PROHIBITED"
	ReasonCodeCostFailedUnpaid                     ReasonCode = "COST_FAILED_UNPAID"
	ReasonCodeStackFailedEmpty                     ReasonCode = "STACK_FAILED_EMPTY"
	ReasonCodeRulesFailedCardLogicMissing          ReasonCode = "RULES_FAILED_CARD_LOGIC_MISSING"
	ReasonCodeRulesFailedCardLogicUnavailable      ReasonCode = "RULES_FAILED_CARD_LOGIC_UNAVAILABLE"
	ReasonCodeRulesFailedGameAlreadyOver           ReasonCode = "RULES_FAILED_GAME_ALREADY_OVER"
	ReasonCodeRulesFailedInvalidState              ReasonCode = "RULES_FAILED_INVALID_STATE"
	ReasonCodeRulesFailedUnknownActionKind         ReasonCode = "RULES_FAILED_UNKNOWN_ACTION_KIND"
	ReasonCodeRulesFailedRandomMaxInvalid          ReasonCode = "RULES_FAILED_RANDOM_MAX_INVALID"
	ReasonCodeRulesFailedStepEnded                 ReasonCode = "RULES_FAILED_STEP_ENDED"
	ReasonCodeRulesFailedInvariantViolated         ReasonCode = "RULES_FAILED_INVARIANT_VIOLATED"
)

// LegalityResult is the structured legality response returned by the authoritative checker.
type LegalityResult struct {
	OK         bool              `json:"ok"`
	ReasonCode ReasonCode        `json:"reasonCode,omitempty"`
	MessageKey string            `json:"messageKey,omitempty"`
	Hook       string            `json:"hook,omitempty"`
	Context    map[string]string `json:"context,omitempty"`
}

// ActionAccepted is the minimal protocol payload emitted when an action commits successfully.
type ActionAccepted struct {
	Type      string    `json:"type"`
	Action    Action    `json:"action"`
	Operation Operation `json:"operation"`
	Event     Event     `json:"event"`
	Revision  Revision  `json:"revision"`
}

// ActionRejected is the minimal protocol payload emitted when an action is rejected before commit.
type ActionRejected struct {
	Type     string         `json:"type"`
	Action   Action         `json:"action"`
	Legality LegalityResult `json:"legality"`
}

// StatePatched is the minimal protocol payload emitted after each successful commit.
type StatePatched struct {
	Type          string              `json:"type"`
	AudienceKind  string              `json:"audienceKind"`
	AudienceID    string              `json:"audienceId,omitempty"`
	Revision      Revision            `json:"revision"`
	Event         Event               `json:"event"`
	PlayerView    *PlayerViewState    `json:"playerView,omitempty"`
	SpectatorView *SpectatorViewState `json:"spectatorView,omitempty"`
}

// LegalityError is returned when an action is rejected before any state mutation is committed.
type LegalityError struct {
	Result     LegalityResult    `json:"result"`
	Code       ReasonCode        `json:"code"`
	Message    string            `json:"message"`
	MessageKey string            `json:"messageKey"`
	Hook       string            `json:"hook,omitempty"`
	Context    map[string]string `json:"context,omitempty"`
}

func (err *LegalityError) Error() string {
	return string(err.Code) + ": " + err.Message
}

// ProhibitionScopeKind defines who is affected by a prohibition rule.
type ProhibitionScopeKind string

const (
	// ProhibitionScopeAllPlayers means the prohibition applies to all players.
	ProhibitionScopeAllPlayers ProhibitionScopeKind = "all_players"
	// ProhibitionScopeOpponentsOnly means the prohibition applies only to opponents of the source controller.
	ProhibitionScopeOpponentsOnly ProhibitionScopeKind = "opponents_only"
	// ProhibitionScopeControllerOnly means the prohibition applies only to the controller of the source.
	ProhibitionScopeControllerOnly ProhibitionScopeKind = "controller_only"
)

// CardCondition defines the conditions a source card must satisfy for a prohibition to be active.
type CardCondition struct {
	Zone         CardZone `json:"zone,omitempty"`         // Card must be in this zone
	Ready        bool     `json:"ready,omitempty"`        // Card must be ready (not exhausted)
	NotDestroyed bool     `json:"notDestroyed,omitempty"` // Card must not be destroyed
	Revealed     bool     `json:"revealed,omitempty"`     // Card must be revealed (optional)
}

// ProhibitionScope defines the scope of players affected by a prohibition.
type ProhibitionScope struct {
	Kind ProhibitionScopeKind `json:"kind"`
}

// TargetCondition defines additional conditions on the target being acted upon.
// This is a reserved extension point for future use (e.g., XQ31/XQ01).
type TargetCondition struct {
	// Kinds defines required card kinds on the target (e.g. character-only aura filters).
	Kinds []CardKind `json:"kinds,omitempty"`

	// Keywords defines required keywords on the target (reserved for XQ31: "声望")
	Keywords []string `json:"keywords,omitempty"`

	// RegionID defines the region scope (reserved for XQ01: "本地区")
	RegionID string `json:"regionId,omitempty"`

	// AbilityKinds defines which ability kinds are affected (reserved for XQ01: "触发能力", "行动能力")
	AbilityKinds []string `json:"abilityKinds,omitempty"`

	// Side defines whether the target must be ally or enemy (reserved for XQ31: "本方", "敌方")
	Side SideKind `json:"side,omitempty"`
}

// SideKind defines whether a target is considered ally or enemy.
type SideKind string

const (
	// SideAlly means the target is an ally (same controller as source).
	SideAlly SideKind = "ally"
	// SideEnemy means the target is an enemy (different controller from source).
	SideEnemy SideKind = "enemy"
)

// TargetCategory defines what kinds of targets are prohibited.
type TargetCategory struct {
	BasicTypes  []string         `json:"basicTypes,omitempty"`  // Prohibited card basic types (e.g., "事务")
	ActionKinds []ActionKind     `json:"actionKinds,omitempty"` // Prohibited action kinds
	Condition   *TargetCondition `json:"condition,omitempty"`   // Additional target conditions (reserved extension)
}

// ProhibitionRule defines a self-contained prohibition effect.
// It describes: when active (source condition), who affected (scope), what prohibited (target).
type ProhibitionRule struct {
	// SourceDefinitionID is the card definition ID that produces this prohibition (e.g., "XQ22").
	SourceDefinitionID string `json:"sourceDefinitionId"`

	// SourceCondition defines when the source card is considered "active" for this prohibition.
	SourceCondition CardCondition `json:"sourceCondition"`

	// Scope defines which players are affected by this prohibition.
	Scope ProhibitionScope `json:"scope"`

	// TargetCategory defines what is being prohibited.
	TargetCategory TargetCategory `json:"targetCategory"`

	// Description is a human-readable description of this prohibition (for debugging).
	Description string `json:"description,omitempty"`
}

// ProhibitionMatchResult contains the result of a prohibition check.
type ProhibitionMatchResult struct {
	Prohibited     bool             // Whether the action is prohibited
	MatchedRule    *ProhibitionRule // The rule that caused the prohibition (if any)
	SourceCardID   string           // The actual source card instance ID (for error messages)
	SourceCardName string           // The source card name (for error messages)
}

// TargetLegalityRule defines a self-contained rule for whether a target can be acted upon.
// This is a reserved extension point for future use (e.g., XQ31 "不能成为目标").
type TargetLegalityRule struct {
	// SourceDefinitionID is the card definition ID that produces this rule (e.g., "XQ31").
	SourceDefinitionID string `json:"sourceDefinitionId"`

	// SourceCondition defines when the source card is considered "active".
	SourceCondition CardCondition `json:"sourceCondition"`

	// AffectedTargetCondition defines which targets are affected by this rule.
	AffectedTargetCondition TargetCondition `json:"affectedTargetCondition"`

	// ActorRestriction defines which actors are restricted by this rule.
	ActorRestriction ProhibitionScope `json:"actorRestriction"`

	// Description is a human-readable description of this rule (for debugging).
	Description string `json:"description,omitempty"`
}

// ContinuousEffectTemplate defines a production continuous effect produced by a card.
// This is used to dynamically build continuous effects based on active cards on the board.
type ContinuousEffectTemplate struct {
	// SourceDefinitionID is the card definition ID that produces this effect (e.g., "XQ31").
	SourceDefinitionID string `json:"sourceDefinitionId"`

	// SourceCondition defines when the source card is considered "active" for this effect.
	SourceCondition CardCondition `json:"sourceCondition"`

	// TargetCondition defines which targets are affected by this effect.
	TargetCondition TargetCondition `json:"targetCondition"`

	// Layer is the continuous effect layer.
	Layer ContinuousLayer `json:"layer"`

	// EffectKind is the effect kind (e.g., "modifyStat").
	EffectKind string `json:"effectKind"`

	// DurationKind is how long the effect lasts (e.g., "permanent").
	DurationKind string `json:"durationKind"`

	// Stat is the stat to modify (if effectKind is "modifyStat").
	Stat string `json:"stat,omitempty"`

	// Amount is the amount to modify (if effectKind is "modifyStat").
	Amount int `json:"amount,omitempty"`

	// Keyword is the keyword to add/remove (if effectKind is "addKeyword" or "removeKeyword").
	Keyword string `json:"keyword,omitempty"`

	// Permission is the permission to grant/prohibit (if effectKind is "grantPermission" or "prohibitPermission").
	Permission string `json:"permission,omitempty"`

	// Description is a human-readable description of this effect (for debugging).
	Description string `json:"description,omitempty"`
}

// TargetLegalityMatchResult contains the result of a target legality check.
type TargetLegalityMatchResult struct {
	// CanTarget is true if the target can be acted upon.
	CanTarget bool

	// MatchedRule is the rule that caused the restriction (if any).
	MatchedRule *TargetLegalityRule

	// SourceCardID is the actual source card instance ID (for error messages).
	SourceCardID string

	// SourceCardName is the source card name (for error messages).
	SourceCardName string
}
