//! TK010 printed abilities and TK010-only fee ruling; lifecycle lives in generated.rs.
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

pub(crate) fn generated(c: &Card) -> bool { c.generated.as_ref().is_some_and(|o|o.kind==crate::generated::GeneratedKind::Tentacle) }

// “触手角色” names the character. TK010's printed subtypes are 突变体/器官.
pub(crate) fn character(c: &Card) -> bool {
    !c.face_down && catalog::has_kind(catalog::card(&c.definition), "character")
        && catalog::card(&c.definition).name == "触手"
}

impl Game {
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

    pub(crate) fn validate_tentacle_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if f.source.card.definition==ID && matches!(f.ability_key.as_str(), "deploy" | "conceal" | "reveal") {
            return Err("生成触手不能恢复普通打出或付费现身帧".into());
        }
        Ok(())
    }
    pub(crate) fn validate_tentacle_play(&self, item: &StackItem) -> RuleResult<()> {
        // Effect-driven free flipping stays on the board; it is not a paid
        // character transaction. Inspect the actual card before trusting frame.
        if item.card.as_ref().is_some_and(|c|c.definition==ID) {
            return Err("生成触手不能作为普通打出或付费现身事务".into());
        }
        if let Some(f) = &item.frame { self.validate_tentacle_frame(f)?; }
        Ok(())
    }
    pub(crate) fn validate_tentacle_state(&self) -> RuleResult<()> {
        for item in &self.stack { self.validate_tentacle_play(item)?; }
        for e in &self.effects { match e {
            Effect::Frame{frame} | Effect::JZ57AfterMill{frame} => self.validate_tentacle_frame(frame)?,
            Effect::DeathCaller{caller} => self.validate_tentacle_frame(&caller.frame)?,
            _ => ()
        }}
        if let Some(p) = &self.pending { if let ChoiceResolution::Frame{frame,..} = &p.resolution { self.validate_tentacle_frame(frame)?; } }

        Ok(())
    }
}
