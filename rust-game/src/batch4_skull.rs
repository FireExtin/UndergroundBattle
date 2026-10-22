//! JZ40 keeps the attached region's physical identity after paying sacrifice.
//! Time placement targets a character/attachment; destruction tests that original
//! host region, including markers physically on its card and nested attachments.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::Serialize;
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "batch4-jz40-time-destroy-host";
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut d = attachment_definition(target(Zone::Region, EntityKind::Any, Relation::Any, Range::Anywhere));
    let mut t = target(Zone::Board, EntityKind::Any, Relation::Any, Range::Anywhere);
    t.predicate = Some(TargetPredicate::JZ40CharacterOrAttachment);
    let mut a = ability(KEY, "牺牲：目标角色或附属时间1，原宿主地区时间至少4则毁灭", Timing::Standard,
        vec![Cost::SacrificeSource], vec![t], vec![Op::JZ40TimeAndDestroyHost], None);
    a.activation_only = true;
    d.abilities.push(a);
    d
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "play" if a.ability_id.as_deref().is_none_or(|k| k == "attach") =>
            Action { card_id:a.card_id.clone(), region:a.region, ability_id:a.ability_id.clone(), ..Action::new("play") },
        "activate" if a.ability_id.as_deref() == Some(KEY) =>
            Action { card_id:a.card_id.clone(), target_id:a.target_id.clone(), ability_id:a.ability_id.clone(), ..Action::new("activate") },
        _ => return Err("蚀刻头骨只有结附与固定标准牺牲能力".into()),
    };
    if *a != expected { return Err("蚀刻头骨不能重选原宿主、数量或追加费用载荷".into()); }
    Ok(())
}
impl Game {
    pub(crate) fn jz40_target(&self, source: &SourceSnapshot, c: &Card) -> bool {
        source.card.definition == "JZ40"
            && ((!c.face_down && catalog::card(&c.definition).kind == "character")
                || self.attachments.iter().any(|a| a.card.id == c.id && self.attachment_host_valid(a)
                    && (!c.face_down || crate::batch4_plans::secret(c)) && catalog::card(&c.definition).kind == "attachment"))
    }
    pub(crate) fn validate_jz40_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let s = &f.source; let c = &s.card;
        if c.definition != "JZ40" || c.face_down || c.id.is_empty() || c.owner >= self.players.len() || f.actor >= self.players.len()
            || c.controller != f.actor || s.play_source.is_some() || s.death_echo.is_some() || s.observed_death.is_some()
            || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() || f.frame_id.is_empty() || f.frame_id == c.id
            || f.cursor > f.steps.len() || f.ability_key != "attach" && f.chosen_region.is_some()
            || f.cursor == 0 && !matches!(f.guard,GuardState::Unchecked)
                && !(f.ability_key == "attach" && matches!(f.guard,GuardState::Accepted | GuardState::Cancelled))
            || f.cursor > 0 && !matches!(f.guard,GuardState::Accepted) {
            return Err("蚀刻头骨须保留原实体、操控者及有限执行帧".into());
        }
        self.mobility_immutable_identity(c)?;
        let a = definition().abilities.into_iter().find(|a| a.key == f.ability_key).ok_or("未知蚀刻头骨程序")?;
        let steps = a.ops.iter().map(|op| Step {context:f.actor,op:op.clone()}).collect::<Vec<_>>();
        if !same(&steps,&f.steps) || f.targets.len() != 1 || !same(&f.targets[0].spec,&a.targets[0]) {
            return Err("蚀刻头骨程序、环境与原目标不完整".into());
        }
        let t = &f.targets[0];
        let valid_player = |id: &str| id.strip_prefix('p').and_then(|n| n.parse::<usize>().ok())
            .is_some_and(|n| n < self.players.len() && player_id(n) == id);
        if t.id.is_empty() || t.public.instance_id != t.id || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some() {
            return Err("蚀刻头骨目标摘要被改写".into());
        }
        if f.ability_key == "attach" {
            let r = t.id.strip_prefix("region:").and_then(|n| n.parse::<usize>().ok()).filter(|r| *r < self.regions.len()).ok_or("头骨须指定地区")?;
            if f.chosen_region != Some(r) || t.id != format!("region:{r}") || t.public.kind != "region" || t.public.region != Some(r)
                || t.public.owner.is_some() || t.public.controller.is_some() || t.region_instance.as_ref().is_none_or(|id| id.is_empty())
                || s.region.is_some() || s.source_region_instance.is_some() || s.attachment_host_instance.is_some()
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)] if ids.len() <= 4 && ids.iter().all(|id| !id.is_empty() && *id != c.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("头骨结附须保留印刷费用与目标地区实体".into());
            }
        } else {
            let r = s.region.filter(|r| *r < self.regions.len()).ok_or("头骨牺牲缺少原宿主地区")?;
            let host = s.source_region_instance.as_ref().filter(|id| !id.is_empty() && *id != &c.id).ok_or("头骨缺少原宿主UID")?;
            if s.attachment_host_instance.as_ref() != Some(host)
                || self.regions.iter().enumerate().any(|(other,live)| live.card.id == *host && other != r)
                || t.region_instance.is_some() || !matches!(t.public.kind.as_str(),"character" | "attachment")
                || t.public.owner.as_deref().is_none_or(|id| !valid_player(id)) || t.public.controller.as_deref().is_none_or(|id| !valid_player(id))
                || !matches!(f.already_paid.as_slice(),[PaidCost::Sacrificed {old_instance,controller,owner}] if old_instance == &c.id && *controller == f.actor && *owner == c.owner) {
                return Err("头骨必须保留原宿主与只牺牲自身的支付收据".into());
            }
        }
        if self.batch3_physical_cards().any(|live| live.id == c.id) {
            return Err("已支付的头骨旧UID不能被其他区域实体冒充".into());
        }
        Ok(())
    }
    pub(crate) fn jz40_resolve(&mut self, f: &ResolutionFrame) {
        if let Some(c) = self.board_mut(&f.targets[0].id) {
            c.time_markers = c.time_markers.saturating_add(1);
        }
        if let Some(r) = self.original_region(&f.source) {
            if self.physical_region_time_markers(r) >= 4 { self.regions[r].destroyed = true; }
        }
    }
    pub(crate) fn validate_jz40_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame {
                if f.source.card.definition != "JZ40" || self.validated_cross_card_frame(f)? { continue; }
                self.validate_jz40_frame(f)?;
                if !frames.insert(&f.frame_id) || item.id != f.frame_id || item.controller != f.actor || f.cursor != 0
                    || !matches!(f.guard,GuardState::Unchecked) || item.reveal || item.deploy_region.is_some()
                    || item.target != Some(f.targets[0].id.clone()) || item.card.is_some() != (f.ability_key == "attach")
                    || item.card.as_ref().is_some_and(|c| c.definition != "JZ40" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor) {
                    return Err("头骨堆叠外壳与固定帧不一致".into());
                }
            }
        }
        for effect in &self.effects {
            if matches!(effect, Effect::Frame {frame} if frame.source.card.definition == "JZ40" && !self.validated_cross_card_frame(frame)?) {
                return Err("头骨原子效果不能停在未绑定的执行队列".into());
            }
        }
        if self.pending.as_ref().is_some_and(|p| matches!(&p.resolution,ChoiceResolution::Frame {frame,..} if frame.source.card.definition == "JZ40")) {
            return Err("头骨没有可改写的结算后选择".into());
        }
        Ok(())
    }
}
