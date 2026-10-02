use crate::model::Icons;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const RULES_VERSION: &str = "hegemony-pdf-v1";
pub const POOL_VERSION: &str = "limited-v2.1";
pub const ENGINE_VERSION: &str = "rust-v0.2.2";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardDefinition {
    pub id: String,
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub cost: u32,
    #[serde(default)]
    pub loyalty: Vec<String>,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub magic: String,
    #[serde(default)]
    pub society: String,
    #[serde(default)]
    pub subtypes: Vec<String>,
    #[serde(skip_deserializing)]
    pub magic_icon: crate::rules::MagicIcon,
    #[serde(skip_deserializing)]
    pub rule_traits: crate::rules::Traits,
    #[serde(skip_deserializing)]
    pub abilities: Vec<AbilitySummary>,
    pub text: String,
    #[serde(default)]
    pub permanent_icons: Icons,
    #[serde(default)]
    pub temporary_icons: Icons,
    #[serde(default)]
    pub defense: Option<u32>,
    #[serde(default)]
    pub threshold: Option<u32>,
    #[serde(default)]
    pub points: Option<u32>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub unique: bool,
    #[serde(default = "yes")]
    pub supported: bool,
}
impl Default for crate::rules::MagicIcon {
    fn default() -> Self {
        Self::None
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilitySummary {
    pub key: String,
    pub label: String,
    pub timing: String,
    pub costs: Vec<crate::rules::Cost>,
    pub triggered: bool,
}
fn yes() -> bool {
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckEntry {
    pub card_id: String,
    pub count: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub card_count: usize,
    pub cards: Vec<DeckEntry>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub rules_version: String,
    pub card_pool_version: String,
    pub engine_version: String,
    pub decks: Vec<DeckDefinition>,
    pub cards: Vec<CardDefinition>,
    #[serde(skip_serializing)]
    pub world: Vec<DeckEntry>,
}

pub fn catalog() -> &'static Catalog {
    static VALUE: OnceLock<Catalog> = OnceLock::new();
    VALUE.get_or_init(|| {
        let raw: serde_json::Value = serde_json::from_str(include_str!("../data/cards.json"))
            .expect("verified printed card registry JSON");
        let list = raw
            .as_array()
            .or_else(|| raw.get("cards").and_then(|x| x.as_array()))
            .expect("registry cards array");
        let cards: Vec<CardDefinition> = list
            .iter()
            .map(|v| {
                let mut definition: CardDefinition =
                    serde_json::from_value(v.clone()).expect("normalized registry entry");
                definition.permanent_icons =
                    serde_json::from_value(v["icons"]["permanent"].clone())
                        .expect("permanent icons");
                definition.temporary_icons =
                    serde_json::from_value(v["icons"]["temporary"].clone())
                        .expect("temporary icons");
                definition.r#type = v["subtypes"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .join("/")
                    })
                    .unwrap_or_default();
                definition.magic_icon = match definition.magic.as_str() {
                    "" => crate::rules::MagicIcon::None,
                    "鲜血" | "血" => crate::rules::MagicIcon::Blood,
                    "心灵" => crate::rules::MagicIcon::Mind,
                    other => crate::rules::MagicIcon::Other(other.into()),
                };
                let rules = crate::rules::definition(&definition.id);
                definition.rule_traits = rules.traits.clone();
                definition.abilities = rules
                    .abilities
                    .iter()
                    .map(|a| AbilitySummary {
                        key: a.key.clone(),
                        label: a.label.clone(),
                        timing: match a.timing {
                            crate::rules::Timing::Standard => "standard",
                            crate::rules::Timing::Fast => "fast",
                            crate::rules::Timing::ActionFast => "actionFast",
                        }
                        .into(),
                        costs: a.costs.clone(),
                        triggered: a.event.is_some(),
                    })
                    .collect();
                definition
            })
            .collect();
        let decks =
            serde_json::from_value(raw["decks"].clone()).expect("verified curated deck list");
        let world = serde_json::from_value(raw["world"].clone()).expect("curated world entries");
        Catalog {
            rules_version: RULES_VERSION.into(),
            card_pool_version: POOL_VERSION.into(),
            engine_version: ENGINE_VERSION.into(),
            decks,
            cards,
            world,
        }
    })
}
pub fn card(id: &str) -> &'static CardDefinition {
    catalog()
        .cards
        .iter()
        .find(|d| d.id == id)
        .unwrap_or_else(|| panic!("unsupported card {id}"))
}
pub fn deck(id: &str) -> Option<&'static DeckDefinition> {
    catalog().decks.iter().find(|d| d.id == id)
}
