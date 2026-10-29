//! Personal in-play headquarters, outside all regions.
use crate::{catalog::CardDefinition, engine::RuleResult, model::*};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SocietyZone {
    #[serde(default, skip_serializing_if = "crate::model::is_zero")]
    pub exposure: u32,
    #[serde(default, skip_serializing_if = "crate::model::is_zero")]
    pub secrecy: u32,
    /// Shoggoth's separate loss-of-control marker, not exposure or destruction.
    #[serde(default, skip_serializing_if = "crate::model::is_zero")]
    pub loss_control: u32,
    pub card: Option<Card>,
    // The stable per-seat source survives card instance changes and ready resets.
    // Only starting a new game replaces this zone and clears these ability keys.
    #[serde(default)]
    pub used_once_per_game: BTreeSet<String>,
    /// Internal MSCQ02 event receipts; never projected as once-per-game abilities.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub(crate) mscq02_observed: BTreeSet<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocietyZoneView {
    pub exposure: u32,
    pub secrecy: u32,
    #[serde(default, skip_serializing_if = "crate::model::is_zero")]
    pub loss_control: u32,
    pub hand_limit: usize,
    pub id: String,
    pub player_id: String,
    pub card: Option<CardView>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SocietyDeckConstraint {
    RedPurpleNeutral,
    BlackNeutralOrDeath,
    Shoggoth,
    BlackWhiteNeutralCorpse,
    BlueNeutralOrPrintedVampireSlave,
    BlueYellowNeutralOrPrintedVampireSpecter,
    GrayYellowNeutralOrPrintedAgentMediaInternet,
    RedNeutralOrPrintedDisasterTimeFate,
    MinimumColor { color: String, count: usize },
    GreenNeutralOrPrintedHumanCombat,
    ClockmakersOffColorMaximum,
    GrayNeutralOrNoMagic,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SocietyDefinition {
    #[serde(flatten)]
    pub card: CardDefinition,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub subtitle: String,
    // Society cards have no printed play cost. The legacy u32 is not a printed fee.
    #[serde(default)]
    pub printed_cost: Option<u32>,
    pub starting_hand: usize,
    pub deck_constraints: Vec<SocietyDeckConstraint>,
    // An unresolved condition is closed, never interpreted as free permission or a no-op payoff.
    pub unresolved_abilities: BTreeMap<String, String>,
}

pub fn definition(id: &str) -> RuleResult<&'static SocietyDefinition> {
    crate::catalog::catalog()
        .societies
        .iter()
        .find(|s| s.card.id == id)
        .ok_or_else(|| format!("未知或未开放秘社：{id}"))
}

pub(crate) fn ensure_activation_condition(id: &str, ability: &str) -> RuleResult<()> {
    if let Some(society) = crate::catalog::catalog()
        .societies
        .iter()
        .find(|s| s.card.id == id)
    {
        if let Some(issue) = society.unresolved_abilities.get(ability) {
            return Err(format!("该秘社能力条件尚待裁定：{issue}"));
        }
    }
    Ok(())
}

pub(crate) fn validate_construction(
    society: &SocietyDefinition,
    entries: &[crate::catalog::DeckEntry],
    cards: &[CardDefinition],
) -> RuleResult<()> {
    for constraint in &society.deck_constraints {
        match constraint {
            SocietyDeckConstraint::RedPurpleNeutral=>{if society.card.id!="MSJZ04"||entries.iter().filter_map(|e|cards.iter().find(|c|c.id==e.card_id)).any(|c|!matches!(c.color.as_str(),"红"|"紫"|"中立")){return Err("破茧者构筑仅红紫中立".into());}},
            SocietyDeckConstraint::BlackNeutralOrDeath => {if society.card.id!="MSJC16"||entries.iter().filter_map(|e|cards.iter().find(|c|c.id==e.card_id)).any(|c|!matches!(c.color.as_str(),"黑"|"中立")&&c.magic_icon!=crate::rules::MagicIcon::Death){return Err("颂亡者构筑仅黑、中立或死亡领域牌".into());}},
            SocietyDeckConstraint::RedNeutralOrPrintedDisasterTimeFate => {
                if society.card.id!="MSCQ02"||entries.iter().filter_map(|e|cards.iter().find(|c|c.id==e.card_id)).any(|c|!matches!(c.color.as_str(),"红"|"中立")&&!c.subtypes.iter().any(|s|matches!(s.as_str(),"灾难"|"时间"|"命运"))){return Err("诺尔维构筑仅红、中立及灾难、时间、命运牌".into());}
            },
            SocietyDeckConstraint::GrayYellowNeutralOrPrintedAgentMediaInternet => {
                if society.card.id!="MSWM02"||entries.iter().filter_map(|e|cards.iter().find(|c|c.id==e.card_id)).any(|c|!matches!(c.color.as_str(),"灰"|"黄"|"中立")&&!c.subtypes.iter().any(|s|matches!(s.as_str(),"特工"|"媒体"|"互联网"))){return Err("公众安全委员会构筑仅灰黄中立或其他派系印刷特工、媒体、互联网牌".into());}
            },
            SocietyDeckConstraint::BlueYellowNeutralOrPrintedVampireSpecter => {
                if society.card.id!="MSWM01" || entries.iter().filter_map(|e|cards.iter().find(|c|c.id==e.card_id)).any(|c| !matches!(c.color.as_str(),"蓝"|"黄"|"中立") && !(c.kind=="character" && c.subtypes.iter().any(|s|matches!(s.as_str(),"吸血鬼"|"阴魂")))) {return Err("曼纽构筑仅蓝黄中立及其他派系吸血鬼或阴魂角色".into());}
            }
            SocietyDeckConstraint::BlueNeutralOrPrintedVampireSlave => {
                if society.card.id != "MSJC12" || entries.iter().filter_map(|e|cards.iter().find(|c|c.id==e.card_id)).any(|c| !matches!(c.color.as_str(),"蓝"|"中立") && !(c.kind=="character" && c.subtypes.iter().any(|s|matches!(s.as_str(),"吸血鬼"|"奴仆")))) { return Err("爱弗罗德构筑仅允许蓝、中立及其他派系印刷吸血鬼或奴仆角色".into()); }
            },
            SocietyDeckConstraint::BlackWhiteNeutralCorpse => {
                if society.card.id!="MSBQ03"||entries.iter().filter_map(|e|cards.iter().find(|c|c.id==e.card_id)).any(|c|!matches!(c.color.as_str(),"黑"|"白"|"中立")){return Err("圣甲虫药业构筑仅黑白中立".into());}
            },
            SocietyDeckConstraint::Shoggoth => {
                if society.card.id!="MSBQ02"||entries.iter().any(|e|cards.iter().find(|c|c.id==e.card_id).is_none_or(|c|!matches!(c.color.as_str(),"灰"|"红"|"中立"))) {return Err("修格斯构筑仅允许灰、红、中立牌".into());}
                let commands:usize=entries.iter().filter(|e|cards.iter().any(|c|c.id==e.card_id&&c.subtypes.iter().any(|s|s=="命令"))).map(|e|e.count).sum();if commands<12{return Err("修格斯牌组须至少12张命令".into());}
            },
            SocietyDeckConstraint::ClockmakersOffColorMaximum => {
                if society.card.id != "MSJC13" { return Err("计时者构筑限制不能移植".into()); }
                let off: usize = entries.iter().filter(|e| cards.iter().any(|c| c.id == e.card_id
                    && !["红", "中立"].contains(&c.color.as_str()))).map(|e|e.count).sum();
                if off > 12 { return Err(format!("计时者非红且非中立牌合计不能超过12张，当前{off}张")); }
            }
            SocietyDeckConstraint::GrayNeutralOrNoMagic => {
                if society.card.id != "MSJC14" { return Err("哨兵构筑条件不能移植".into()); }
                if let Some(c) = entries.iter().filter_map(|e| cards.iter().find(|c| c.id == e.card_id))
                    .find(|c| !["灰", "中立"].contains(&c.color.as_str()) && c.magic_icon != crate::rules::MagicIcon::None) {
                    return Err(format!("哨兵构筑不允许具有魔法领域图标的异色牌：{}", c.name));
                }
            }
            SocietyDeckConstraint::GreenNeutralOrPrintedHumanCombat => {
                if society.card.id != "MSJC11" {
                    return Err("此有限混搭构筑条件仅开放给S.P.T.执行部".into());
                }
                if let Some(c) = entries
                    .iter()
                    .filter_map(|e| cards.iter().find(|c| c.id == e.card_id))
                    .find(|c| {
                        !["绿", "中立"].contains(&c.color.as_str())
                            && !(c.kind == "character"
                                && c.subtypes.iter().any(|s| s == "人类")
                                && c.permanent_icons.combat > 0)
                    })
                {
                    return Err(format!(
                        "S.P.T.执行部构筑不允许{}：异色牌须为印刷人类角色且具有永久战斗图标",
                        c.name
                    ));
                }
            }
            SocietyDeckConstraint::MinimumColor { color, count } => {
                let actual: usize = entries
                    .iter()
                    .filter(|e| cards.iter().any(|c| c.id == e.card_id && c.color == *color))
                    .map(|e| e.count)
                    .sum();
                if actual < *count {
                    return Err(format!(
                        "秘社构筑要求至少{count}张{color}牌，当前{actual}张"
                    ));
                }
            }
        }
    }
    Ok(())
}

impl Game {
    pub(crate) fn society_host_seat(&self, id: &str) -> Option<usize> {
        let seat = id.strip_prefix("society:p")?.parse::<usize>().ok()?;
        // Headquarters is a per-seat zone; choosing a society card is optional.
        self.players.get(seat).filter(|p| !p.eliminated
            && id == format!("society:p{seat}")).map(|_| seat)
    }
    pub(crate) fn society_usage(&self, source: &str) -> Option<&BTreeSet<String>> {
        self.players
            .iter()
            .find(|p| p.society_zone.card.as_ref().is_some_and(|c| c.id == source))
            .map(|p| &p.society_zone.used_once_per_game)
    }

    pub(crate) fn consume_society_usage(&mut self, source: &str, ability: &str) -> RuleResult<()> {
        let zone = self
            .players
            .iter_mut()
            .find(|p| p.society_zone.card.as_ref().is_some_and(|c| c.id == source))
            .map(|p| &mut p.society_zone)
            .ok_or("每局限次能力须来自稳定秘社来源")?;
        if !zone.used_once_per_game.insert(ability.into()) {
            return Err("此能力本局已发动过".into());
        }
        Ok(())
    }

    pub(crate) fn society_card(&self, id: &str) -> Option<&Card> {
        self.players
            .iter()
            .filter_map(|p| p.society_zone.card.as_ref())
            .find(|c| c.id == id)
    }

    // Ability sources are deliberately broader than region-targetable board entities.
    pub(crate) fn ability_source(&self, id: &str) -> Option<(Option<usize>, &Card)> {
        self.in_play(id)
            .or_else(|| self.society_card(id).map(|c| (None, c)))
    }

    pub(crate) fn ability_source_mut(&mut self, id: &str) -> Option<&mut Card> {
        if self.society_card(id).is_some() {
            self.players
                .iter_mut()
                .filter_map(|p| p.society_zone.card.as_mut())
                .find(|c| c.id == id)
        } else {
            self.board_mut(id)
        }
    }

    pub(crate) fn society_source_zone(&self, id: Option<&str>) -> Option<String> {
        let id = id?;
        if let Some(c) = self.society_card(id) {
            return Some(format!("society:{}", player_id(c.owner)));
        }
        self.attachments.iter().find(|a| a.card.id == id)
            .and_then(|a| self.society_host_seat(&a.host_id).map(|_| a.host_id.clone()))
    }

    pub(crate) fn society_views(&self, viewer: usize) -> Vec<SocietyZoneView> {
        self.players
            .iter()
            .map(|p| SocietyZoneView {
                exposure: p.society_zone.exposure,
                secrecy: p.society_zone.secrecy,
                loss_control: p.society_zone.loss_control,
                hand_limit: self.hand_limit(p.seat),
                id: format!("society:{}", player_id(p.seat)),
                player_id: player_id(p.seat),
                card: p
                    .society_zone
                    .card
                    .as_ref()
                    .map(|c| self.card_view(c, viewer, None, Some("society"))),
            })
            .collect()
    }
}

#[cfg(not(feature = "society-fixtures"))]
fn red_gray_society(id: &str, name: &str, subtitle: &str, color: &str, subtype: &str) -> SocietyDefinition {
    let mut card: CardDefinition = serde_json::from_value(serde_json::json!({
        "id":id,"name":name,"kind":"society","type":format!("秘社/{subtype}"),
        "subtypes":[subtype],"color":color,"society":name,"unique":true,"supported":true,
        "text":format!("持续：构筑时你的牌组中需包含25张或更多{color}色派系的牌。行动3，横置：若你具有先手标志，则抓一张牌。行动4，横置：从你的牌库中寻找一张{color}色独有牌，展示该牌后置于你的手中，然后将你的牌库洗牌。该能力每局游戏只能发动一次。")
    })).expect("verified whole original MSJC04/MSJC05");
    card.abilities = crate::rules::definition(id).abilities.iter().map(|a| crate::catalog::AbilitySummary {
        key: a.key.clone(), label: a.label.clone(), timing: "standard".into(),
        costs: a.costs.clone(), triggered: false,
    }).collect();
    SocietyDefinition {
        card, subtitle: subtitle.into(), printed_cost: None, starting_hand: 6,
        deck_constraints: vec![SocietyDeckConstraint::MinimumColor { color: color.into(), count: 25 }],
        unresolved_abilities: BTreeMap::new(),
    }
}

pub(crate) fn definitions() -> Vec<SocietyDefinition> {
    #[cfg(not(feature = "society-fixtures"))]
    {
        let mut card: CardDefinition = serde_json::from_value(serde_json::json!({
            "id":"MSJC09","name":"秘社","kind":"society","type":"秘社",
            "color":"中立","society":"-","unique":true,"supported":true,
            "text":"行动3，横置：若你具有【先手标志】，则抓一张牌。"
        }))
        .expect("verified MSJC09 printed fields");
        card.abilities = crate::rules::definition("MSJC09")
            .abilities
            .iter()
            .map(|a| crate::catalog::AbilitySummary {
                key: a.key.clone(),
                label: a.label.clone(),
                timing: "standard".into(),
                costs: a.costs.clone(),
                triggered: false,
            })
            .collect();
        let msjc09 = SocietyDefinition {
            card,
            subtitle: "未知的聚会".into(),
            printed_cost: None,
            starting_hand: 6,
            deck_constraints: vec![],
            unresolved_abilities: BTreeMap::new(),
        };
        let mut card: CardDefinition = serde_json::from_value(serde_json::json!({
            "id":"MSJC01","name":"帷幕守望","kind":"society","type":"秘社/法师结社",
            "subtypes":["法师结社"],"color":"黄","society":"帷幕守望","unique":true,"supported":true,
            "text":"构筑：你的牌组中需包含25张或更多黄色派系牌。行动3，横置：若你具有【先手标志】，则抓一张牌。行动4，横置：从你的牌库中寻找一张黄色独有牌，展示该牌后置于你的手中，然后将你的牌库洗牌。该能力每局游戏只能发动一次。"
        })).expect("verified printed MSJC01 fields");
        card.abilities = crate::rules::definition("MSJC01")
            .abilities
            .iter()
            .map(|a| crate::catalog::AbilitySummary {
                key: a.key.clone(),
                label: a.label.clone(),
                timing: "standard".into(),
                costs: a.costs.clone(),
                triggered: false,
            })
            .collect();
        let msjc01 = SocietyDefinition {
            card,
            subtitle: "世界守护者".into(),
            printed_cost: None,
            starting_hand: 6,
            deck_constraints: vec![SocietyDeckConstraint::MinimumColor {
                color: "黄".into(),
                count: 25,
            }],
            unresolved_abilities: BTreeMap::new(),
        };
        let mut card: CardDefinition = serde_json::from_value(serde_json::json!({
            "id":"MSJC07","name":"方碑序列","kind":"society","type":"秘社/法师结社",
            "subtypes":["法师结社"],"color":"黑","society":"方碑序列","unique":true,"supported":true,
            "text":"构筑：你的牌组中需包含25张或更多黑色派系牌。行动3，横置：若你具有【先手标志】，则抓一张牌。行动4，横置：从你的牌库中寻找一张黑色独有牌，展示该牌后置于你的手中，然后将你的牌库洗牌。该能力每局游戏只能发动一次。"
        })).expect("verified printed MSJC07 fields");
        card.abilities = crate::rules::definition("MSJC07")
            .abilities
            .iter()
            .map(|a| crate::catalog::AbilitySummary {
                key: a.key.clone(),
                label: a.label.clone(),
                timing: "standard".into(),
                costs: a.costs.clone(),
                triggered: false,
            })
            .collect();
        let mut saint_card: CardDefinition = serde_json::from_value(serde_json::json!({
            "id":"MSJC06","name":"圣贤","kind":"society","type":"秘社/群体",
            "subtypes":["群体"],"color":"白","society":"圣贤","unique":true,"supported":true,
            "text":"构筑：你的牌组中需包含25张或更多白色派系牌。行动3，横置：若你具有【先手标志】，则抓一张牌。行动4，横置：从你的牌库中寻找一张白色独有牌，展示该牌后置于你的手中，然后将你的牌库洗牌。该能力每局游戏只能发动一次。"
        })).expect("verified printed MSJC06 fields");
        saint_card.abilities = crate::rules::definition("MSJC06")
            .abilities
            .iter()
            .map(|a| crate::catalog::AbilitySummary {
                key: a.key.clone(),
                label: a.label.clone(),
                timing: "standard".into(),
                costs: a.costs.clone(),
                triggered: false,
            })
            .collect();
        let mut dream_card: CardDefinition = serde_json::from_value(serde_json::json!({
            "id":"MSJC08","name":"梦境行者","kind":"society","type":"秘社/群体/梦境",
            "subtypes":["群体","梦境"],"color":"紫","society":"梦境行者","unique":true,"supported":true,
            "text":"构筑：你的牌组中需包含25张或更多紫色派系牌。行动3，横置：若你具有【先手标志】，则抓一张牌。行动4，横置：从你的牌库中寻找一张紫色独有牌，展示该牌后置于你的手中，然后将你的牌库洗牌。该能力每局游戏只能发动一次。"
        })).expect("verified whole original MSJC08 fields and confirmed subtitle");
        dream_card.abilities = crate::rules::definition("MSJC08")
            .abilities
            .iter()
            .map(|a| crate::catalog::AbilitySummary {
                key: a.key.clone(),
                label: a.label.clone(),
                timing: "standard".into(),
                costs: a.costs.clone(),
                triggered: false,
            })
            .collect();
        let mut execution_card: CardDefinition = serde_json::from_value(serde_json::json!({
            "id":"MSJC11","name":"S.P.T.执行部","kind":"society","type":"秘社/企业/部门",
            "subtypes":["企业","部门"],"color":"绿","society":"猎魔人","unique":true,"supported":true,
            "text":"持续：构筑时你的牌组中只能包含绿色和中立派系的牌，以及其他派系带有永久战斗能力图标的人类角色牌。行动3，横置：目标角色获得杀伤1直到回合结束。行动3，横置：选择目标地区，所有位于该地区，且带有永久战斗能力图标的本方角色获得撤回直到回合结束。"
        })).expect("verified whole original MSJC11 fields");
        execution_card.abilities = crate::rules::definition("MSJC11")
            .abilities
            .iter()
            .map(|a| crate::catalog::AbilitySummary {
                key: a.key.clone(),
                label: a.label.clone(),
                timing: "standard".into(),
                costs: a.costs.clone(),
                triggered: false,
            })
            .collect();
        let mut hunter_card: CardDefinition = serde_json::from_value(serde_json::json!({
            "id":"MSJC02","name":"猎魔人","kind":"society","type":"秘社/群体","subtypes":["群体"],"color":"绿","society":"猎魔人","unique":true,"supported":true,
            "text":"持续：构筑时你的牌组中需包含25张或更多绿色派系的牌。行动3，横置：若你具有【先手标志】，则抓一张牌。行动4，横置：从你的牌库中寻找一张绿色独有牌，展示该牌后置于你的手中，然后将你的牌库洗牌。该能力每局游戏只能发动一次。"
        })).expect("verified whole original MSJC02");
        hunter_card.abilities = crate::rules::definition("MSJC02")
            .abilities
            .iter()
            .map(|a| crate::catalog::AbilitySummary {
                key: a.key.clone(),
                label: a.label.clone(),
                timing: "standard".into(),
                costs: a.costs.clone(),
                triggered: false,
            })
            .collect();
        let mut throne_card: CardDefinition = serde_json::from_value(serde_json::json!({
            "id":"MSJC03","name":"王座会","kind":"society","type":"秘社/法师结社/吸血鬼","subtypes":["法师结社","吸血鬼"],"color":"蓝","society":"王座会","unique":true,"supported":true,
            "text":"持续：构筑时你的牌组中需包含25张或更多蓝色派系的牌。行动3，横置：若你具有先手标志，则抓一张牌。行动4，横置：自你的牌库中寻找一张蓝色独有牌，将该牌展示后加入你的手牌，然后将你的牌库洗牌。该能力每局游戏只能发动一次。"
        })).expect("verified whole original MSJC03");
        throne_card.abilities = crate::rules::definition("MSJC03").abilities.iter().map(|a| crate::catalog::AbilitySummary {
            key:a.key.clone(),label:a.label.clone(),timing:"standard".into(),costs:a.costs.clone(),triggered:false,
        }).collect();
        let mut societies = vec![
            msjc09,
            msjc01,
            SocietyDefinition {
                card,
                subtitle: "恐怖同盟".into(),
                printed_cost: None,
                starting_hand: 6,
                deck_constraints: vec![SocietyDeckConstraint::MinimumColor {
                    color: "黑".into(),
                    count: 25,
                }],
                unresolved_abilities: BTreeMap::new(),
            },
            SocietyDefinition {
                card: saint_card,
                subtitle: "热爱之道".into(),
                printed_cost: None,
                starting_hand: 6,
                deck_constraints: vec![SocietyDeckConstraint::MinimumColor {
                    color: "白".into(),
                    count: 25,
                }],
                unresolved_abilities: BTreeMap::new(),
            },
            SocietyDefinition {
                card: dream_card,
                subtitle: "幻梦呓语".into(),
                printed_cost: None,
                starting_hand: 6,
                deck_constraints: vec![SocietyDeckConstraint::MinimumColor {
                    color: "紫".into(),
                    count: 25,
                }],
                unresolved_abilities: BTreeMap::new(),
            },
            SocietyDefinition {
                card: execution_card,
                subtitle: "直属特遣队".into(),
                printed_cost: None,
                starting_hand: 6,
                deck_constraints: vec![SocietyDeckConstraint::GreenNeutralOrPrintedHumanCombat],
                unresolved_abilities: BTreeMap::new(),
            },
            SocietyDefinition {
                card: hunter_card,
                subtitle: "猎杀异种战团".into(),
                printed_cost: None,
                starting_hand: 6,
                deck_constraints: vec![SocietyDeckConstraint::MinimumColor {
                    color: "绿".into(),
                    count: 25,
                }],
                unresolved_abilities: BTreeMap::new(),
            },
            SocietyDefinition {
                card: throne_card,
                subtitle: "通往至高王座".into(),
                printed_cost: None,
                starting_hand: 6,
                deck_constraints: vec![SocietyDeckConstraint::MinimumColor { color: "蓝".into(), count: 25 }],
                unresolved_abilities: BTreeMap::new(),
            },
        ];
        societies.extend([
            red_gray_society("MSJC04", "鸣钟教派", "末日钟的侍奉者", "红", "秘密教派"),
            red_gray_society("MSJC05", "国家机构", "政府权力", "灰", "机构"),
            crate::batch4_clock::society(),
        ]);
        societies.push(crate::sentinel::society());
        societies.push(crate::msbq02::society());
    societies.push(crate::msjz04::society());
    societies.push(crate::msjc12::society());
    societies.push(crate::mswm01::society());
        societies.push(crate::mswm02::society());
        societies.push(crate::msbq03::society());
        societies.push(crate::mscq02::society());
        societies.push(crate::msjc16::society());
        societies
    }
    #[cfg(feature = "society-fixtures")]
    {
        [("FIXTURE_SOCIETY_SIX",6,None,false), ("FIXTURE_SOCIETY_FOUR",4,Some(("中立",25)),false), ("FIXTURE_SOCIETY_PENDING",6,None,true)]
            .into_iter().map(|(id,starting_hand,minimum,pending)| {
                let mut card: CardDefinition = serde_json::from_value(serde_json::json!({
                    "id":id,"name":id,"kind":"society","text":"Internal foundation fixture; not a printed playable card.","unique":true,"supported":true
                })).expect("typed internal society fixture");
                card.abilities = crate::rules::definition(id).abilities.iter().map(|a| crate::catalog::AbilitySummary {
                    key:a.key.clone(),label:a.label.clone(),timing:"standard".into(),costs:a.costs.clone(),triggered:false,
                }).collect();
                SocietyDefinition { card, subtitle:String::new(), printed_cost:None, starting_hand,
                    deck_constraints: minimum.map(|(color,count)| SocietyDeckConstraint::MinimumColor { color:color.into(),count }).into_iter().collect(),
                    unresolved_abilities: if pending { BTreeMap::from([("fixture-draw".into(),"U13-test-unresolved".into())]) } else { BTreeMap::new() },
                }
            }).collect()
    }
}

#[cfg(all(test, not(feature = "society-fixtures")))]
#[path = "society_msjc09_tests.rs"]
mod printed_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{catalog, deck};

    #[test]
    fn every_seat_has_an_empty_non_region_zone_and_registry_matches_support_flag() {
        let mut g = Game::new(
            "society-empty".into(),
            "invite".into(),
            "teams".into(),
            "A".into(),
            "watchers".into(),
            9,
        )
        .unwrap();
        for name in ["B", "C", "D"] {
            g.join(name.into(), "watchers".into()).unwrap();
        }
        for seat in 0..4 {
            let v = g.view(seat);
            assert_eq!(v.society_zones.len(), 4);
            for (i, z) in v.society_zones.iter().enumerate() {
                assert_eq!(z.id, format!("society:p{i}"));
                assert!(z.card.is_none());
            }
        }
        let mut d = deck::preset("watchers").unwrap();
        d.society_id = Some("MSJC09".into());
        assert_eq!(
            deck::validate(d).is_ok(),
            !cfg!(feature = "society-fixtures")
        );
        assert!(!catalog::catalog().cards.iter().any(|d| d.kind == "society"));
        #[cfg(feature = "society-fixtures")]
        assert!(catalog::catalog()
            .societies
            .iter()
            .all(|d| d.card.id.starts_with("FIXTURE_")));
        #[cfg(not(feature = "society-fixtures"))]
        {
            assert_eq!(
                catalog::catalog()
                    .societies
                    .iter()
                    .map(|s| s.card.id.as_str())
                    .collect::<Vec<_>>(),
                vec!["MSJC09", "MSJC01", "MSJC07", "MSJC06", "MSJC08", "MSJC11", "MSJC02", "MSJC03", "MSJC04", "MSJC05", "MSJC13", "MSJC14", "MSBQ02", "MSJZ04", "MSJC12", "MSWM01", "MSWM02", "MSBQ03", "MSCQ02", "MSJC16"]
            );
        }
        assert_eq!(
            deck::build_rules().society_supported,
            !catalog::catalog().societies.is_empty()
        );
    }

    #[cfg(feature = "society-fixtures")]
    mod enabled {
        use super::*;
        use crate::{
            room::{RoomCommand, RoomEnvelope, SessionAction},
            rules,
        };

        fn draft(id: Option<&str>) -> deck::DeckDraft {
            let mut d = deck::preset("watchers").unwrap();
            d.cards = vec![catalog::DeckEntry {
                card_id: "JC125".into(),
                count: 50,
            }];
            d.society_id = id.map(str::to_owned);
            d
        }
        fn lobby() -> Game {
            let mut g = Game::new_with_deck(
                "society-fixture".into(),
                "invite".into(),
                "teams".into(),
                "A".into(),
                draft(Some("FIXTURE_SOCIETY_SIX")),
                9,
            )
            .unwrap();
            for (name, id) in [
                ("B", None),
                ("C", Some("FIXTURE_SOCIETY_FOUR")),
                ("D", Some("FIXTURE_SOCIETY_PENDING")),
            ] {
                g.join_with_deck(name.into(), draft(id)).unwrap();
            }
            g
        }
        fn started() -> Game {
            let mut g = lobby();
            for seat in 0..4 {
                g.apply(seat, Action::new("ready")).unwrap();
            }
            g.apply(0, Action::new("start")).unwrap();
            while let Some(p) = g.pending.clone() {
                g.apply(
                    p.seat,
                    Action {
                        choice_id: Some(p.choice.id),
                        selected: Some(vec![]),
                        ..Action::new("choose")
                    },
                )
                .unwrap();
            }
            g
        }
        fn action(g: &Game) -> Action {
            Action {
                card_id: Some(g.players[0].society_zone.card.as_ref().unwrap().id.clone()),
                ability_id: Some("fixture-draw".into()),
                ..Action::new("activate")
            }
        }
        fn pay_layout(g: &mut Game) {
            g.window = Some(Window::Action(0));
            g.priority_team = 0;
            g.active_team = 0;
            g.passed.clear();
            let c = g.make_card("JC125", 0);
            g.players[0].assets.push(c);
        }

        #[test]
        fn optional_society_is_separate_from_fifty_and_color_constraint_counts_copies() {
            let d = draft(Some("FIXTURE_SOCIETY_FOUR"));
            assert_eq!(deck::validate(d.clone()).unwrap().cards[0].count, 50);
            let mut small = d.clone();
            small.cards[0].count = 49;
            assert!(deck::validate(small).unwrap_err().contains("50"));
            let mut stuffed = d.clone();
            stuffed.cards.push(catalog::DeckEntry {
                card_id: "FIXTURE_SOCIETY_SIX".into(),
                count: 1,
            });
            assert!(deck::validate(stuffed).is_err());
            // Typed, test-only white player entry makes the exact 24/25 boundary buildable.
            let mut white = catalog::card("JC125").clone();
            white.id = "FIXTURE_WHITE_PLAYER".into();
            white.name = "fixture white".into();
            white.color = "白".into();
            let mut registry = catalog::catalog().cards.clone();
            registry.push(white);
            for (neutral, valid) in [(24, false), (25, true)] {
                let mut candidate = d.clone();
                candidate.cards = vec![
                    catalog::DeckEntry {
                        card_id: "JC125".into(),
                        count: neutral,
                    },
                    catalog::DeckEntry {
                        card_id: "FIXTURE_WHITE_PLAYER".into(),
                        count: 50 - neutral,
                    },
                ];
                assert_eq!(
                    deck::validate_with_cards(candidate, &registry).is_ok(),
                    valid
                );
            }
        }

        #[test]
        fn start_atomically_reveals_all_choices_with_personal_hand_sizes_and_mulligan_privacy() {
            let mut g = lobby();
            for seat in 0..4 {
                let v = g.view(seat);
                assert!(v.society_zones.iter().all(|z| z.card.is_none()));
                assert_eq!(
                    v.your_deck.unwrap().society_id,
                    g.players[seat].deck_snapshot.as_ref().unwrap().society_id
                );
                g.apply(seat, Action::new("ready")).unwrap();
            }
            g.apply(0, Action::new("start")).unwrap();
            assert_eq!(
                g.players.iter().map(|p| p.hand.len()).collect::<Vec<_>>(),
                vec![6, 6, 4, 6]
            );
            assert_eq!(
                g.players.iter().map(|p| p.deck.len()).collect::<Vec<_>>(),
                vec![44, 44, 46, 44]
            );
            for seat in 0..4 {
                let v = g.view(seat);
                assert_eq!(
                    v.society_zones.iter().filter(|z| z.card.is_some()).count(),
                    3
                );
                assert!(v
                    .society_zones
                    .iter()
                    .filter_map(|z| z.card.as_ref())
                    .all(|c| !c.face_down && c.region.is_none() && c.kind == "society"));
                assert!(v.hand.iter().all(|c| c.owner == player_id(seat)));
                assert_eq!(
                    v.pending_choice.is_some(),
                    seat == g.pending.as_ref().unwrap().seat
                );
            }
            let restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
            for s in 0..4 {
                assert_eq!(
                    serde_json::to_value(g.view(s)).unwrap(),
                    serde_json::to_value(restored.view(s)).unwrap()
                );
            }
        }

        #[test]
        fn society_action_pays_once_and_recovers_response_frame_and_all_four_projections() {
            let mut g = started();
            pay_layout(&mut g);
            let a = action(&g);
            let id = a.card_id.clone().unwrap();
            let legal = g
                .legal_actions(0)
                .into_iter()
                .find(|l| l.action == a)
                .unwrap();
            assert_eq!(legal.source_zone_id.as_deref(), Some("society:p0"));
            let room = RoomEnvelope::from_game(g);
            let before = room.players[0].hand.len();
            let cmd = RoomCommand {
                command_id: "society-paid".into(),
                expected_version: room.revision,
                action: SessionAction::Game { action: a.clone() },
            };
            let transition = room.transition(0, Some(cmd), 1000).unwrap();
            assert_eq!(transition.outcome, "accepted");
            let mut recovered = RoomEnvelope::from_persisted(&transition.state).unwrap();
            assert!(recovered.players[0].assets[0].exhausted);
            assert!(recovered.society_card(&id).unwrap().exhausted);
            assert_eq!(recovered.players[0].hand.len(), before);
            let frame = recovered.stack[0].frame.as_ref().unwrap();
            assert!(frame.source.region.is_none());
            assert_eq!(frame.already_paid.len(), 2);
            assert_eq!(
                serde_json::to_string(&room.replay_events(&transition.journal).unwrap()).unwrap(),
                transition.state
            );
            for seat in 0..4 {
                assert_eq!(
                    serde_json::to_value(transition.view.game.society_zones.clone()).unwrap(),
                    serde_json::to_value(recovered.game.view(seat).society_zones).unwrap()
                );
            }
            // Continue the unchanged game reducer after serialization; no repeated payment.
            while !recovered.game.stack.is_empty() {
                let seat = recovered
                    .game
                    .living(recovered.game.priority_team)
                    .into_iter()
                    .find(|s| !recovered.game.passed.contains(s))
                    .unwrap();
                recovered.game.apply(seat, Action::new("pass")).unwrap();
            }
            assert_eq!(recovered.players[0].hand.len(), before + 1);
            assert!(recovered.society_card(&id).unwrap().exhausted);
            assert!(recovered.game.apply(0, a).is_err());
            assert_eq!(recovered.players[0].hand.len(), before + 1);
        }

        #[test]
        fn wrong_seat_insufficient_payment_and_unresolved_conditions_are_atomic_noops() {
            let mut g = started();
            g.window = Some(Window::Action(0));
            g.priority_team = 0;
            g.active_team = 0;
            g.passed.clear();
            let a = action(&g);
            for seat in [0, 1, 2, 3] {
                let before = serde_json::to_string(&g).unwrap();
                assert!(g.apply(seat, a.clone()).is_err());
                assert_eq!(serde_json::to_string(&g).unwrap(), before);
            }
            g.window = Some(Window::Action(1));
            g.priority_team = 1;
            g.active_team = 1;
            let c = g.make_card("JC125", 3);
            g.players[3].assets.push(c);
            let pending = Action {
                card_id: Some(g.players[3].society_zone.card.as_ref().unwrap().id.clone()),
                ability_id: Some("fixture-draw".into()),
                ..Action::new("activate")
            };
            for first in [0, 1] {
                g.first_team = first;
                let before = serde_json::to_string(&g).unwrap();
                assert!(g.apply(3, pending.clone()).unwrap_err().contains("U13"));
                assert_eq!(serde_json::to_string(&g).unwrap(), before);
                assert!(!g.legal_actions(3).iter().any(|l| l.action == pending));
            }
        }

        #[test]
        fn society_is_not_a_board_target_asset_or_removable_and_resets_same_instance() {
            let mut g = started();
            pay_layout(&mut g);
            let id = g.players[0].society_zone.card.as_ref().unwrap().id.clone();
            assert!(g.board(&id).is_none());
            assert!(g.remove_board(&id).is_none());
            g.remove_dead(&id, RemovalCause::Destroy);
            g.return_hand(&id);
            g.to_bottom(&id);
            assert!(g.society_card(&id).is_some());
            let source = g.source_snapshot(g.society_card(&id).unwrap(), None);
            let spec = rules::TargetSlotSpec {
                zone: rules::Zone::Board,
                kind: rules::EntityKind::Any,
                relation: rules::Relation::Any,
                range: rules::Range::Anywhere,
                subtype: None,
                printed_subtype: false,
                subtypes_any: vec![],
                printed_cost_max: None,
                predicate: None,
                equipment_host: false,
                requires_magic: false,
                exclude_source: false,
                exclude_attachment_host: false,
                attachment_host_condition: None,
                min: 1,
                max: 1,
            };
            assert!(!g.valid_binding(0, &source, &spec, &id));
            for kind in ["asset", "deploy", "conceal", "reveal"] {
                let before = serde_json::to_string(&g).unwrap();
                assert!(g
                    .apply(
                        0,
                        Action {
                            card_id: Some(id.clone()),
                            region: Some(0),
                            ..Action::new(kind)
                        }
                    )
                    .is_err());
                assert_eq!(serde_json::to_string(&g).unwrap(), before);
            }
            g.players[0].society_zone.card.as_mut().unwrap().exhausted = true;
            g.effects.push_back(Effect::NextTurn);
            g.drive().unwrap();
            assert!(!g.society_card(&id).unwrap().exhausted);
            let restored = Game::from_persisted(&serde_json::to_string(&g).unwrap()).unwrap();
            assert!(restored.society_card(&id).is_some());
        }
    }
}
