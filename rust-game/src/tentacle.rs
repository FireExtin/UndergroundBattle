//! TK010 only: project lifecycle ruling of 2026-10-29, not a general token rule.
use crate::{catalog, engine::RuleResult, model::*};

pub(crate) const ID: &str = "TK010";

pub(crate) fn definition() -> crate::rules::Definition {
    let mut d = crate::rules::with_abilities(vec![]);
    d.modifiers.push(crate::rules::StaticModifier::TentacleCoreSupport);
    d
}

pub(crate) fn validate_definition(id: &str, d: &crate::rules::Definition) -> RuleResult<()> {
    if (id == ID || d.modifiers.iter().any(|m| matches!(m, crate::rules::StaticModifier::TentacleCoreSupport)))
        && (id != ID || serde_json::to_value(d).unwrap() != serde_json::to_value(definition()).unwrap()) {
        return Err("触手须保留原持续能力，不能移植到其他牌".into());
    }
    Ok(())
}

pub(crate) fn generated(c: &Card) -> bool { c.definition == ID }

// “触手角色” names the character. TK010's printed subtypes are 突变体/器官.
pub(crate) fn character(c: &Card) -> bool {
    !c.face_down && catalog::has_kind(catalog::card(&c.definition), "character")
        && catalog::card(&c.definition).name == "触手"
}

impl Game {
    /// Call only after normal host/departure cleanup. Never prune physical zones
    /// afterwards: that would already have emitted false arrival observations.
    pub(crate) fn tentacle_disappears(&mut self, c: &Card) -> bool {
        if !generated(c) { return false; }
        self.note("生成触手离场消失，不进入实体牌区".into());
        true
    }

    pub(crate) fn tentacle_core_defense(&self, c: &Card) -> u32 {
        if c.face_down || !self.board(&c.id).is_some_and(|(_, live)| live == c)
            || !self.selected_core_card(c) || c.controller != c.owner { return 0; }
        self.regions.iter().filter(|r| !r.vacant).flat_map(|r| &r.cards)
            .filter(|t| generated(t) && !t.face_down && t.controller == c.controller).count() as u32
    }

    pub(crate) fn tentacle_temporary_combat(&self, c: &Card, region: usize) -> u32 {
        if !generated(c) || c.face_down || !self.board(&c.id).is_some_and(|(r, live)| r == region && live == c) { return 0; }
        if self.region_characters(region).any(|core| !core.face_down
            && core.controller == c.controller && core.owner == c.controller && self.selected_core_card(core)) { 2 } else { 0 }
    }

    pub(crate) fn validate_tentacle_state(&self) -> RuleResult<()> {
        // Retention exceptions are board (including face down), sealed, and a
        // real reveal transaction/burial continuation. No physical deck card.
        for c in self.batch3_physical_cards().filter(|c| generated(c)) {
            let retained = self.regions.iter().any(|r| !r.vacant && r.cards.iter().any(|v| std::ptr::eq(v,c)))
                || self.sealed_cards.iter().any(|s| std::ptr::eq(&s.card,c))
                || self.stack.iter().any(|s| s.reveal && s.card.as_ref().is_some_and(|v| std::ptr::eq(v,c)))
                || self.effects.iter().any(|e| matches!(e, Effect::Bury {card} if std::ptr::eq(card,c)));
            if !retained { return Err("生成触手不能存在于手牌、牌库、墓地或其他实体牌区".into()); }
        }
        Ok(())
    }
}
