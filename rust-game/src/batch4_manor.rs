//! XQ15 is an ownership rule, including non-board character cards. Printed
//! definitions and their owner never change when control is transferred.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::Serialize;
use std::collections::BTreeSet;
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut d = crate::rules::attachment_definition(target(Zone::Society, EntityKind::Any, Relation::ControlledByActor, Range::Anywhere));
    d.modifiers.push(StaticModifier::XQ15OwnerVampireSlave);
    d
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    if a.kind != "play" || a.ability_id.as_deref().is_some_and(|id| id != "attach")
        || *a != (Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), ..Action::new("play") }) {
        return Err("爱弗罗德庄园只可结附于本方秘社".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn xq15_owner_aura(&self, c: &Card) -> bool {
        !c.face_down && c.owner < self.players.len() && catalog::card(&c.definition).kind == "character"
            && !self.players.iter().any(|p| p.assets.iter().any(|a| a.id == c.id))
            && !self.sealed_cards.iter().any(|s| s.card.id == c.id)
            && self.attachments.iter().any(|a| a.card.definition == "XQ15" && !a.card.face_down
                && a.card.controller == c.owner && self.society_host_seat(&a.host_id) == Some(c.owner)
                && self.attachment_host_valid(a))
    }
    pub(crate) fn xq15_owner_subtypes(&self, c: &Card, result: &mut Vec<String>) {
        if !self.xq15_owner_aura(c) { return; }
        let human = result.iter().any(|s| s == "人类");
        if human {
            result.retain(|s| s != "人类");
            if !result.iter().any(|s| s == "吸血鬼") { result.push("吸血鬼".into()); }
        } else if !result.iter().any(|s| s == "吸血鬼" || s == "奴仆") {
            result.push("奴仆".into());
        }
    }
    pub(crate) fn validate_xq15_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let s = &f.source; let c = &s.card;
        let id = |id: &str| id.strip_prefix('i').and_then(|n| n.parse::<u64>().ok()).filter(|n| *n > 0 && *n <= self.sequence);
        if c.definition != "XQ15" || c.face_down || id(&c.id).is_none() || c.owner >= self.players.len() || f.actor >= self.players.len() || c.controller != f.actor
            || s.region.is_some() || s.source_region_instance.is_some() || s.play_source.is_some() || s.attachment_host_instance.is_some()
            || s.death_echo.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || id(&f.frame_id).is_none_or(|n| n <= id(&c.id).unwrap()) || f.ability_key != "attach" || f.chosen_region.is_some()
            || !f.steps.is_empty() || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked | GuardState::Accepted | GuardState::Cancelled)
            || self.batch3_physical_cards().any(|live| live.id == c.id)
            || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= 2 && ids.iter().all(|id| !id.is_empty() && *id != c.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
            return Err("庄园结附须保留原牌、操控者、空程序及实际支付".into());
        }
        self.mobility_immutable_identity(c)?;
        let [t] = f.targets.as_slice() else { return Err("庄园只能结附于原本方秘社".into()); };
        let host = format!("society:p{}", f.actor);
        if !same(&t.spec, &definition().abilities[0].targets[0]) || t.id != host || t.region_instance.is_some()
            || t.public.instance_id != host || t.public.kind != "society" || t.public.region.is_some() || !t.public.valid || t.public.status != "valid"
            || t.public.invalid_reason.is_some() || t.public.owner.as_deref() != Some(&player_id(f.actor)) || t.public.controller.as_deref() != Some(&player_id(f.actor))
            || t.public.label != format!("{}的秘社", self.players[f.actor].name) {
            return Err("庄园冻结目标不能换成他人秘社或地区".into());
        }
        Ok(())
    }
    pub(crate) fn validate_xq15_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame {
                if f.source.card.definition != "XQ15" { continue; }
                self.validate_xq15_frame(f)?;
                if !frames.insert(&f.frame_id) || item.id != f.frame_id || item.controller != f.actor || !matches!(f.guard, GuardState::Unchecked)
                    || item.deploy_region.is_some() || item.reveal || item.target.as_ref() != f.targets.first().map(|t| &t.id)
                    || item.card.as_ref().is_none_or(|c| {
                        let sequence = |id: &str| id.strip_prefix('i').and_then(|n| n.parse::<u64>().ok());
                        // Ordinary play retires the hand UID before constructing
                        // the response frame. The stack holds that fresh card.
                        c.definition != "XQ15" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor
                            || sequence(&c.id).is_none_or(|n| sequence(&f.source.card.id).is_none_or(|old| n <= old)
                                || sequence(&f.frame_id).is_none_or(|frame| n >= frame))
                    }) {
                    return Err("庄园堆叠外壳与实际结附帧不符".into());
                }
            }
        }
        if self.effects.iter().any(|e| match e {
            Effect::Declare { declaration: d } => d.source.card.definition == "XQ15",
            Effect::Frame { frame: f } => f.source.card.definition == "XQ15",
            _ => false,
        }) || self.pending.as_ref().is_some_and(|p| match &p.resolution {
            ChoiceResolution::Declare { declaration: d, .. } => d.source.card.definition == "XQ15",
            ChoiceResolution::Frame { frame: f, .. } => f.source.card.definition == "XQ15",
            _ => false,
        }) { return Err("庄园没有额外声明、直接执行或待选续体".into()); }
        Ok(())
    }
}
