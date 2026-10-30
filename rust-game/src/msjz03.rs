//! Original identity only: closed clock-group WIP, not a completed society.
//! No initial construction and no executable actions until the group closes.
use crate::{catalog,society::{SocietyDefinition,SocietyDeckConstraint}};
pub(crate) const ID:&str="MSJZ03";
pub(crate) fn society()->SocietyDefinition{
 let card:catalog::CardDefinition=serde_json::from_value(serde_json::json!({"id":ID,"name":"末日钟","kind":"society","subtypes":["圣物","永恒意志"],"color":"红","society":"末日钟","unique":true,"supported":false,"text":"不能作为初始秘社牌。持续：不能打出或付费翻面非灾难类别的牌。持续：不能获得分数；若计分区有三个或更多毁灭标志，赢得游戏。行动横置：目标角色、附属或地区放置时间标志，然后该地区时间至少四则毁灭。行动横置、弃一张牌：目标地区生成钟摆祭司角色指示物。"})).unwrap();
 SocietyDefinition{card,subtitle:"熵之子".into(),printed_cost:None,starting_hand:0,deck_constraints:vec![SocietyDeckConstraint::NotStartingSociety],unresolved_abilities:[("clock-group-wip".into(),"工程未完成：转换入口、持续规则及两项行动；不是待用户裁定".into())].into_iter().collect()}
}
