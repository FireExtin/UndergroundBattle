//! Source-complete JC037: Leader uses the existing controller rule; its
//! continuous investigation bonus is derived from live controlled hidden cards.
use crate::{
    engine::RuleResult,
    model::*,
    rules::{Definition, StaticModifier},
};

pub(crate) fn definition() -> Definition {
    Definition {
        modifiers: vec![StaticModifier::JC037ControlledHiddenInvestigation],
        ..Default::default()
    }
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    let uses = d
        .modifiers
        .iter()
        .any(|m| matches!(m, StaticModifier::JC037ControlledHiddenInvestigation));
    if (id == "JC037" || uses)
        && (id != "JC037"
            || serde_json::to_value(d).unwrap() != serde_json::to_value(definition()).unwrap())
    {
        return Err("JC037只准入完整持续定义，不能改写或移植".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn jc037_controlled_hidden(&self, c: &Card) -> u32 {
        if c.face_down
            || !crate::rules::definition(&c.definition)
                .modifiers
                .iter()
                .any(|m| matches!(m, StaticModifier::JC037ControlledHiddenInvestigation))
            || !self.board(&c.id).is_some_and(|(_, live)| {
                !live.face_down
                    && live.definition == c.definition
                    && live.controller == c.controller
            })
        {
            return 0;
        }
        u32::try_from(
            self.regions
                .iter()
                .flat_map(|r| &r.cards)
                .filter(|other| other.face_down && other.controller == c.controller)
                .count(),
        )
        .unwrap_or(u32::MAX)
    }
}
