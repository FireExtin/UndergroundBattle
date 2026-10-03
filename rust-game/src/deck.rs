//! Printed-name construction rules and immutable per-room deck specifications.
use crate::catalog::{self, CardDefinition, DeckEntry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const MINIMUM_CARDS: usize = 50;
pub const SERVICE_CARD_CAPACITY: usize = 2048;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckDraft {
    pub id: String,
    pub name: String,
    pub description: String,
    pub society_id: Option<String>,
    pub cards: Vec<DeckEntry>,
    pub rules_version: String,
    pub card_pool_version: String,
    pub engine_version: String,
    pub updated_at: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckBuildRules {
    pub minimum_cards: usize,
    pub usual_name_copy_limit: usize,
    pub unique_name_copy_limit: usize,
    pub copy_limit_by_printed_name: bool,
    pub faction_limit: Option<usize>,
    pub society_supported: bool,
    pub service_card_capacity: usize,
    pub capacity_is_service_limit: bool,
}
pub fn build_rules() -> DeckBuildRules {
    DeckBuildRules {
        minimum_cards: MINIMUM_CARDS,
        usual_name_copy_limit: 3,
        unique_name_copy_limit: 1,
        copy_limit_by_printed_name: true,
        faction_limit: None,
        society_supported: cfg!(feature = "society-fixtures"),
        service_card_capacity: SERVICE_CARD_CAPACITY,
        capacity_is_service_limit: true,
    }
}
pub fn copy_limit(card: &CardDefinition) -> Option<usize> {
    if card.keywords.iter().any(|keyword| keyword == "唯一") {
        Some(1)
    } else if card.rule_traits.unlimited_copies {
        None
    } else {
        Some(3)
    }
}
pub fn validate(draft: DeckDraft) -> Result<DeckDraft, String> {
    validate_with_cards(draft, &catalog::catalog().cards)
}
/// The caller supplies a typed registry; game entry points always use the active catalog.
pub fn validate_with_cards(
    mut draft: DeckDraft,
    cards: &[CardDefinition],
) -> Result<DeckDraft, String> {
    if draft.rules_version != catalog::RULES_VERSION
        || draft.card_pool_version != catalog::POOL_VERSION
        || draft.engine_version != catalog::ENGINE_VERSION
    {
        return Err("卡组规则、卡池或引擎版本不匹配，请按当前目录重新保存".into());
    }
    let society = draft
        .society_id
        .as_deref()
        .map(crate::society::definition)
        .transpose()?;
    draft.id = draft.id.trim().into();
    draft.name = draft.name.trim().into();
    if draft.id.is_empty()
        || draft.id.chars().count() > 100
        || draft.name.is_empty()
        || draft.name.chars().count() > 80
        || draft.description.chars().count() > 1000
        || draft.updated_at.chars().count() > 100
    {
        return Err("卡组id/名称不能为空且须符合服务文本长度限制".into());
    }
    if draft.cards.len() > SERVICE_CARD_CAPACITY {
        return Err("卡组条目超过服务容量2048；这不是原作规则的最大牌数".into());
    }
    let mut by_id = BTreeMap::<String, usize>::new();
    let mut by_name = BTreeMap::<String, (usize, Option<usize>)>::new();
    let mut total = 0usize;
    for entry in &draft.cards {
        if entry.count == 0 {
            return Err("每个卡组条目的数量必须大于0".into());
        }
        let card = cards
            .iter()
            .find(|card| card.id == entry.card_id)
            .ok_or_else(|| format!("未知或未开放卡牌：{}", entry.card_id))?;
        if !card.supported
            || card.kind == "region"
            || (card.kind != "character" && card.kind != "spell" && card.kind != "attachment")
        {
            return Err(format!("{}不能加入当前玩家卡组", card.name));
        }
        total = total
            .checked_add(entry.count)
            .ok_or("卡组数量溢出，超出服务容量")?;
        let count = by_id.entry(card.id.clone()).or_default();
        *count = count.checked_add(entry.count).ok_or("同一卡牌数量溢出")?;
        let group = by_name
            .entry(card.name.clone())
            .or_insert((0, copy_limit(card)));
        group.0 = group.0.checked_add(entry.count).ok_or("同名卡牌数量溢出")?;
        group.1 = match (group.1, copy_limit(card)) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (None, _) | (_, None) => None,
        };
        // A printed Unique applies to the whole name even across card versions.
        if cards
            .iter()
            .any(|c| c.name == card.name && c.keywords.iter().any(|k| k == "唯一"))
        {
            group.1 = Some(1);
        }
    }
    if total > SERVICE_CARD_CAPACITY {
        return Err("卡组超过服务容量2048；这不是原作规则的最大牌数".into());
    }
    if total < MINIMUM_CARDS {
        return Err("卡组至少需要50张牌".into());
    }
    for (name, (count, limit)) in by_name {
        if limit.is_some_and(|limit| count > limit) {
            return Err(format!(
                "同名卡牌{name}合计{count}张，最多{}张",
                limit.unwrap()
            ));
        }
    }
    draft.cards = by_id
        .into_iter()
        .map(|(card_id, count)| DeckEntry { card_id, count })
        .collect();
    if let Some(society) = society {
        crate::society::validate_construction(society, &draft.cards, cards)?;
    }
    Ok(draft)
}
pub fn preset(id: &str) -> Result<DeckDraft, String> {
    let deck = catalog::deck(id).ok_or("未知预组")?;
    validate(DeckDraft {
        id: deck.id.clone(),
        name: deck.name.clone(),
        description: deck.description.clone(),
        society_id: None,
        cards: deck.cards.clone(),
        rules_version: catalog::RULES_VERSION.into(),
        card_pool_version: catalog::POOL_VERSION.into(),
        engine_version: catalog::ENGINE_VERSION.into(),
        updated_at: String::new(),
    })
}
