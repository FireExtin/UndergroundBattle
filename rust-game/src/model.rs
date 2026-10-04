use crate::society::{SocietyZone, SocietyZoneView};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Icons {
    pub investigation: u32,
    pub combat: u32,
    pub influence: u32,
}
impl Icons {
    pub fn add(self, rhs: Self) -> Self {
        Self {
            investigation: self.investigation + rhs.investigation,
            combat: self.combat + rhs.combat,
            influence: self.influence + rhs.influence,
        }
    }
    pub fn at(self, contest: usize) -> u32 {
        match contest {
            0 => self.investigation,
            1 => self.combat,
            _ => self.influence,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choice_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocations: Option<BTreeMap<String, u32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_selected: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deck_draft: Option<crate::deck::DeckDraft>,
}
impl Action {
    pub fn new(kind: &str) -> Self {
        Self {
            kind: kind.into(),
            ..Self::default()
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalAction {
    #[serde(flatten)]
    pub action: Action,
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_zone_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_damage_prevention: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_barrier: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_renown: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_subtypes: Option<Vec<String>>,
    pub instance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_id: Option<String>,
    pub name: String,
    pub owner: String,
    pub controller: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<usize>,
    pub exhausted: bool,
    pub face_down: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_cost: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icons: Option<Icons>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defense: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shield: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wounds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magic: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used_once_per_game: Option<Vec<String>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentView {
    #[serde(flatten)]
    pub card: CardView,
    pub host_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceOption {
    pub id: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<CardView>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub description: String,
    pub player_id: String,
    pub options: Vec<ChoiceOption>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_decline: Option<bool>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerView {
    pub id: String,
    pub seat: usize,
    pub name: String,
    pub team: usize,
    pub deck_id: String,
    #[serde(default)]
    pub deck_name: String,
    pub ready: bool,
    pub eliminated: bool,
    pub hand_count: usize,
    pub deck_count: usize,
    pub score: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionView {
    pub id: String,
    pub index: usize,
    pub card_id: String,
    pub name: String,
    pub threshold: u32,
    pub points: u32,
    pub influence: [u32; 2],
    #[serde(default)]
    pub icons_by_team: [Icons; 2],
    #[serde(default)]
    pub skip_confrontation: bool,
    pub characters: Vec<CardView>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackView {
    pub id: String,
    pub label: String,
    pub controller: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability_id: Option<String>,
    pub targets: Vec<String>,
    pub target_summaries: Vec<TargetSummary>,
    pub resolution_state: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetSummary {
    pub instance_id: String,
    pub label: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<usize>,
    pub valid: bool,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub version: u64,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Versions {
    pub rules: String,
    pub card_pool: String,
    pub engine: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaitingChoice {
    pub player_id: String,
    pub kind: String,
    pub title: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub room_id: String,
    pub invite_code: String,
    pub version: u64,
    pub mode: String,
    pub status: String,
    pub you: String,
    pub players: Vec<PlayerView>,
    pub first_team: usize,
    pub active_team: usize,
    pub priority_team: usize,
    pub turn: u32,
    pub phase: String,
    pub step: String,
    pub win_score: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub winner_team: Option<usize>,
    pub regions: Vec<RegionView>,
    #[serde(default)]
    pub society_zones: Vec<SocietyZoneView>,
    #[serde(default)]
    pub attachments: Vec<AttachmentView>,
    pub hand: Vec<CardView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_deck_top: Option<CardView>,
    pub assets: Vec<CardView>,
    pub graveyard: Vec<CardView>,
    pub score_cards: Vec<CardView>,
    pub stack: Vec<StackView>,
    pub pending_choice: Option<Choice>,
    pub legal_actions: Vec<LegalAction>,
    pub log: Vec<LogEntry>,
    pub versions: Versions,
    pub waiting_choice: Option<WaitingChoice>,
    #[serde(default)]
    pub your_deck: Option<crate::deck::DeckDraft>,
    #[serde(default)]
    pub world_deck_count: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub definition: String,
    pub owner: usize,
    pub controller: usize,
    pub exhausted: bool,
    pub face_down: bool,
    pub damage: u32,
    pub wounds: u32,
    pub shield: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub card: Card,
    pub host_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub seat: usize,
    pub name: String,
    pub deck_id: String,
    #[serde(default)]
    pub deck_snapshot: Option<crate::deck::DeckDraft>,
    #[serde(default)]
    pub society_zone: SocietyZone,
    pub ready: bool,
    pub eliminated: bool,
    pub hand: Vec<Card>,
    pub deck: Vec<Card>,
    pub assets: Vec<Card>,
    pub graveyard: Vec<Card>,
    pub score_cards: Vec<Card>,
    pub asset_used: bool,
    pub conceal_used: bool,
}
impl Player {
    pub fn new(seat: usize, name: String, deck_id: String) -> Self {
        Self {
            seat,
            name,
            deck_id,
            deck_snapshot: None,
            society_zone: SocietyZone::default(),
            ready: false,
            eliminated: false,
            hand: vec![],
            deck: vec![],
            assets: vec![],
            graveyard: vec![],
            score_cards: vec![],
            asset_used: false,
            conceal_used: false,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Region {
    pub card: Card,
    pub influence: [u32; 2],
    pub cards: Vec<Card>,
    pub skip: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegionReturnBatch {
    pub region: usize,
    pub hand: Vec<String>,
    pub bottom: Vec<Vec<String>>,
    pub orders: Vec<Option<Vec<String>>>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Window {
    Prepare,
    Draw,
    Action(usize),
    Mobility,
    Before(usize, usize),
    After(usize, usize),
    Win(usize, usize),
    End,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Effect {
    RegionConfrontationsEnded {
        region: usize,
        region_instance: String,
    },
    Declare {
        declaration: Declaration,
    },
    Frame {
        frame: Box<ResolutionFrame>,
    },
    Draw {
        seat: usize,
        count: usize,
    },
    Mulligan {
        seat: usize,
    },
    Discard {
        seat: usize,
        amount: usize,
        redraw: bool,
        optional: bool,
    },
    Forecast {
        seat: usize,
        amount: usize,
        draw: bool,
    },
    Damage {
        seat: usize,
        region: usize,
        amount: u32,
    },
    Recipient {
        team: usize,
        region: usize,
        contest: usize,
        amount: u32,
        seats: Vec<usize>,
    },
    Award {
        seat: usize,
        region: usize,
    },
    Bottom {
        seat: usize,
        region: usize,
    },
    PrepareRegionReturn {
        region: usize,
    },
    CommitRegionReturn {
        region: usize,
    },
    Score {
        seat: usize,
        region: usize,
    },
    Cleanup,
    FinishCleanup,
    Bury {
        card: Card,
    },
    NextTurn,
    Unique {
        seat: usize,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StackItem {
    pub id: String,
    pub label: String,
    pub controller: usize,
    pub card: Option<Card>,
    pub deploy_region: Option<usize>,
    pub reveal: bool,
    pub target: Option<String>,
    pub frame: Option<ResolutionFrame>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceSnapshot {
    pub card: Card,
    pub region: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub play_source: Option<PlaySource>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlaySource {
    Hand,
    Graveyard,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundTarget {
    pub id: String,
    pub spec: crate::rules::TargetSlotSpec,
    pub public: TargetSummary,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PaidCost {
    Assets(Vec<String>),
    Exhausted(String),
    Sacrificed {
        old_instance: String,
        controller: usize,
        owner: usize,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GuardState {
    Unchecked,
    Accepted,
    Cancelled,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    pub context: usize,
    pub op: crate::rules::Op,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResolutionFrame {
    pub frame_id: String,
    pub ability_key: String,
    pub actor: usize,
    pub source: SourceSnapshot,
    pub targets: Vec<BoundTarget>,
    pub already_paid: Vec<PaidCost>,
    pub guard: GuardState,
    pub cursor: usize,
    pub steps: Vec<Step>,
    pub chosen_region: Option<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Declaration {
    pub actor: usize,
    pub source: SourceSnapshot,
    pub ability: crate::rules::AbilitySpec,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DeclareChoice {
    Accept,
    Mode,
    Target,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FrameChoice {
    Forecast {
        seat: usize,
    },
    Discard {
        seat: usize,
        redraw: bool,
    },
    Search {
        seat: usize,
        to_top: bool,
        #[serde(default)]
        visibility: crate::rules::SearchVisibility,
    },
    Sacrifice {
        seat: usize,
    },
    FreeReveal {
        seat: usize,
        require_loyalty: bool,
    },
    SacrificeDraw {
        seat: usize,
    },
    Region,
    GraveyardEntry {
        seat: usize,
        region: usize,
    },
    SimultaneousSearch {
        filter: crate::rules::CardFilter,
        participants: Vec<usize>,
        committed: Vec<(usize, Option<String>)>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostModifier {
    pub actor: usize,
    pub filter: crate::rules::CardFilter,
    pub amount: u32,
    pub expires_turn: u32,
    pub uses: u32,
}
/// A resolved bonus belongs to this exact in-play instance, not its owner,
/// controller, printed definition, or the spell's later graveyard instance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnAttributeModifier {
    pub target_instance: String,
    pub defense_bonus: u32,
    pub ordinary_icons: Icons,
    #[serde(default, skip_serializing_if = "is_false")]
    pub grants_renown: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub prevents_damage: bool,
    pub expires_turn: u32,
}
pub(crate) fn is_false(value: &bool) -> bool {
    !value
}
// Finite project interpretation: last still-valid resolved control wins.
// This ordering is a project ruling, not a claim about the old FAQ.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlLifetime {
    TurnEnd { turn: u32 },
    Attached { source_instance: String },
    SourceLeaves { source_instance: String },
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubtypeChange {
    #[default]
    None,
    HumanToVampire,
    AddSlave,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ControlEffect {
    pub target_instance: String,
    pub recipient: usize,
    pub lifetime: ControlLifetime,
    pub subtype_change: SubtypeChange,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ControlBaseline {
    pub target_instance: String,
    pub controller: usize,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum RemovalCause {
    Sacrifice,
    Destroy,
    Lethal,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChoiceResolution {
    Declare {
        declaration: Declaration,
        stage: DeclareChoice,
    },
    Frame {
        frame: Box<ResolutionFrame>,
        choice: FrameChoice,
    },
    Mulligan,
    Discard {
        redraw: bool,
    },
    Forecast {
        draw: bool,
    },
    Damage {
        region: usize,
    },
    Recipient {
        region: usize,
        contest: usize,
        amount: u32,
    },
    Bottom {
        region: usize,
    },
    Unique,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pending {
    pub choice: Choice,
    pub seat: usize,
    pub resolution: ChoiceResolution,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    pub state_schema: u32,
    pub room_id: String,
    pub invite_code: String,
    pub mode: String,
    pub version: u64,
    pub status: String,
    pub players: Vec<Player>,
    pub seed: u64,
    pub random: u64,
    pub sequence: u64,
    pub first_team: usize,
    pub active_team: usize,
    pub priority_team: usize,
    pub turn: u32,
    pub winner_team: Option<usize>,
    pub regions: Vec<Region>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub region_return: Option<RegionReturnBatch>,
    pub world: Vec<Card>,
    pub stack: Vec<StackItem>,
    pub pending: Option<Pending>,
    pub effects: VecDeque<Effect>,
    pub window: Option<Window>,
    pub passed: BTreeSet<usize>,
    pub team_passed: [bool; 2],
    pub privilege_used: bool,
    pub log: Vec<LogEntry>,
    pub versions: Versions,
    pub modifiers: Vec<CostModifier>,
    #[serde(default)]
    pub turn_attribute_modifiers: Vec<TurnAttributeModifier>,
    #[serde(default)]
    pub control_effects: Vec<ControlEffect>,
    #[serde(default)]
    pub control_baselines: Vec<ControlBaseline>,
}
pub fn player_id(seat: usize) -> String {
    format!("p{seat}")
}
impl Game {
    pub fn from_persisted(state: &str) -> Result<Self, String> {
        let header: serde_json::Value =
            serde_json::from_str(state).map_err(|_| "持久状态不是有效JSON")?;
        if header.get("state_schema").and_then(|v| v.as_u64()) != Some(2)
            || header["versions"]["engine"].as_str() != Some(crate::catalog::ENGINE_VERSION)
            || header["versions"]["cardPool"].as_str() != Some(crate::catalog::POOL_VERSION)
            || header["versions"]["rules"].as_str() != Some(crate::catalog::RULES_VERSION)
        {
            return Err(format!(
                "持久局版本与当前规则核不兼容（需要schema=2、engine={}、cardPool={}）；旧局须使用原版本binary/数据库，不支持静默迁移",
                crate::catalog::ENGINE_VERSION,
                crate::catalog::POOL_VERSION
            ));
        }
        serde_json::from_str(state).map_err(|_| "v2持久状态字段无效".into())
    }
}
