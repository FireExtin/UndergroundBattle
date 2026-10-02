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
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardView {
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
    pub hand: Vec<CardView>,
    pub assets: Vec<CardView>,
    pub graveyard: Vec<CardView>,
    pub score_cards: Vec<CardView>,
    pub stack: Vec<StackView>,
    pub pending_choice: Option<Choice>,
    pub legal_actions: Vec<LegalAction>,
    pub log: Vec<LogEntry>,
    pub versions: Versions,
    pub waiting_choice: Option<WaitingChoice>,
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
pub struct Player {
    pub seat: usize,
    pub name: String,
    pub deck_id: String,
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
    Exhaust {
        actor: usize,
        target: String,
        hidden: bool,
        source: Option<String>,
        #[serde(default)]
        region: Option<usize>,
    },
    Heal {
        actor: usize,
        target: String,
        #[serde(default)]
        region: Option<usize>,
    },
    Police {
        actor: usize,
        region: usize,
    },
    Funeral {
        actor: usize,
        target: String,
    },
    Ceasefire {
        region: usize,
    },
    Chase {
        actor: usize,
        target: String,
        hide: bool,
    },
    Recover {
        seat: usize,
        target: String,
    },
    Summon {
        seat: usize,
        target: String,
        region: usize,
    },
    Move {
        target: String,
        region: usize,
    },
    ReturnHand {
        target: String,
    },
    Search {
        seat: usize,
        kind: String,
    },
    Death {
        seat: usize,
    },
    Trigger {
        seat: usize,
        label: String,
        effect: Box<Effect>,
    },
    TargetTrigger {
        seat: usize,
        label: String,
        source: String,
        mode: String,
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
    Score {
        seat: usize,
        region: usize,
    },
    RegionTrigger {
        seat: usize,
        definition: String,
    },
    Cleanup,
    Bury {
        card: Card,
    },
    NextTurn,
    Unique {
        seat: usize,
    },
    GlobalDamage,
    WorldMode {
        seat: usize,
    },
    WorldAll {
        draw: bool,
    },
    Swap {
        seat: usize,
        selected: Vec<String>,
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
    pub effect: Option<Effect>,
    pub target: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChoiceResolution {
    Mulligan,
    Discard {
        redraw: bool,
    },
    Forecast {
        draw: bool,
    },
    Trigger {
        effect: Effect,
        label: String,
    },
    TargetTrigger {
        source: String,
        mode: String,
        label: String,
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
    Search {
        to_top: bool,
    },
    ReturnHand,
    Mobility {
        source: String,
    },
    Death,
    Unique,
    WorldMode,
    Swap,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pending {
    pub choice: Choice,
    pub seat: usize,
    pub resolution: ChoiceResolution,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
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
}
pub fn player_id(seat: usize) -> String {
    format!("p{seat}")
}
