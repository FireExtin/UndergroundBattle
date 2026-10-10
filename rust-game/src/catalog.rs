use crate::model::Icons;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const RULES_VERSION: &str = "hegemony-pdf-v1";
pub const POOL_VERSION: &str = "limited-v2.53-four-faction-engine-candidate";
#[cfg(not(feature = "society-fixtures"))]
pub const ENGINE_VERSION: &str = "rust-v0.2.58-four-faction-engine-candidate";
#[cfg(feature = "society-fixtures")]
pub const ENGINE_VERSION: &str = "rust-v0.2.58-four-faction-engine-fixture";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardDefinition {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
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
    #[serde(skip_deserializing)]
    pub deck_copy_limit: Option<usize>,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    pub societies: Vec<crate::society::SocietyDefinition>,
    pub deck_build_rules: crate::deck::DeckBuildRules,
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
                // Preserve old definitions; expose only individually reviewed subtitles.
                if !["JZ27", "JC096", "JC114", "XQ38", "LC06", "LC30", "JZ02", "WM059"]
                    .contains(&definition.id.as_str())
                {
                    definition.subtitle = None;
                }
                definition.magic_icon = match definition.magic.as_str() {
                    "" => crate::rules::MagicIcon::None,
                    "鲜血" | "血" => crate::rules::MagicIcon::Blood,
                    "心灵" => crate::rules::MagicIcon::Mind,
                    "死亡" => crate::rules::MagicIcon::Death,
                    other => crate::rules::MagicIcon::Other(other.into()),
                };
                let rules = crate::rules::definition(&definition.id);
                definition.rule_traits = rules.traits.clone();
                definition.deck_copy_limit = crate::deck::copy_limit(&definition);
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
        let world: Vec<DeckEntry> =
            serde_json::from_value(raw["world"].clone()).expect("curated world entries");
        validate_base_world(&world, &cards).expect("one complete printed base-world set");
        Catalog {
            rules_version: RULES_VERSION.into(),
            card_pool_version: POOL_VERSION.into(),
            engine_version: ENGINE_VERSION.into(),
            decks,
            cards,
            societies: crate::society::definitions(),
            deck_build_rules: crate::deck::build_rules(),
            world,
        }
    })
}
pub(crate) fn validate_base_world(
    world: &[DeckEntry],
    cards: &[CardDefinition],
) -> Result<(), String> {
    let expected = (107..=116)
        .map(|n| format!("DQJC{n}"))
        .collect::<std::collections::BTreeSet<_>>();
    let mut ids = std::collections::BTreeSet::new();
    let mut names = std::collections::BTreeSet::new();
    for entry in world {
        let definition = cards
            .iter()
            .find(|c| c.id == entry.card_id)
            .ok_or("世界牌未注册")?;
        if entry.count != 1
            || definition.kind != "region"
            || !definition.supported
            || !ids.insert(entry.card_id.clone())
            || !names.insert(definition.name.clone())
        {
            return Err("基础世界必须使用十种原地区各一张，不能重复填充".into());
        }
    }
    if ids != expected {
        return Err("基础世界缺少或替换了原地区".into());
    }
    Ok(())
}
pub fn card(id: &str) -> &'static CardDefinition {
    catalog()
        .cards
        .iter()
        .find(|d| d.id == id)
        .or_else(|| {
            catalog()
                .societies
                .iter()
                .find(|d| d.card.id == id)
                .map(|d| &d.card)
        })
        .unwrap_or_else(|| panic!("unsupported card {id}"))
}
pub fn deck(id: &str) -> Option<&'static DeckDefinition> {
    catalog().decks.iter().find(|d| d.id == id)
}
