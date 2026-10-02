use crate::model::Icons;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const RULES_VERSION: &str = "hegemony-pdf-v1";
pub const POOL_VERSION: &str = "limited-v1";
pub const ENGINE_VERSION: &str = "rust-v0.1.0";

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
                definition
            })
            .collect();
        let decks =
            serde_json::from_value(raw["decks"].clone()).expect("verified curated deck list");
        Catalog {
            rules_version: RULES_VERSION.into(),
            card_pool_version: POOL_VERSION.into(),
            engine_version: ENGINE_VERSION.into(),
            decks,
            cards,
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
