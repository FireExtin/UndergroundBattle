//! Closed XQ37 entry program using the existing frozen source region and influence primitive.
use crate::{
    engine::RuleResult,
    model::*,
    rules::{self, Op},
};
pub(crate) fn contains_xq37_op(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::XQ37EntryInfluenceIfPresent => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => {
            contains_xq37_op(body)
        }
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => {
            contains_xq37_op(std::slice::from_ref(exhausted))
                || contains_xq37_op(std::slice::from_ref(ready))
        }
        _ => false,
    })
}
fn relevant(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "XQ37"
        || f.ability_key == "entry-existing-influence"
        || f.steps
            .iter()
            .any(|s| contains_xq37_op(std::slice::from_ref(&s.op)))
}
impl Game {
    pub(crate) fn xq37_entry_influence(&mut self, f: &ResolutionFrame) {
        // The already triggered effect stays in the original real entry region.
        // Its conditional count is evaluated after responses, using the frozen actor's team.
        if let Some(r) = self.blue_original_region(f) {
            if self.regions[r].influence[self.team(f.actor)] > 0 {
                self.place_influence(f.actor, r, 1);
            }
        }
    }
    pub(crate) fn validate_xq37_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !relevant(f) {
            return Ok(());
        }
        let c = &f.source.card;
        if c.definition != "XQ37"
            || f.actor >= self.players.len()
            || c.owner >= self.players.len()
            || c.controller != f.actor
            || c.id.is_empty()
            || f.frame_id.is_empty()
            || f.source.region.is_none_or(|r| r >= self.regions.len())
            || f.source
                .source_region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty())
            || f.source.observed_death.is_some()
            || f.source.attachment_host_instance.is_some()
            || !f.targets.is_empty()
            || f.cursor != 0
            || !matches!(f.guard, GuardState::Unchecked)
        {
            return Err("XQ37冻结来源、地区实例或原子执行状态不符".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty()
                || f.chosen_region != f.source.region
                || (f.ability_key == "deploy"
                    && (c.face_down || f.source.play_source != Some(PlaySource::Hand)))
                || (f.ability_key == "reveal" && (!c.face_down || f.source.play_source.is_some()))
            {
                return Err("XQ37仅准入真实手牌派遣或付费现身空程序".into());
            }
            return Ok(());
        }
        if c.face_down
            || f.source.play_source.is_some()
            || !f.already_paid.is_empty()
            || f.chosen_region.is_some()
        {
            return Err("XQ37进场声明不能带有暗藏、支付或预声明地区".into());
        }
        let region = f.source.source_region_instance.as_deref().unwrap();
        let a = match f.ability_key.as_str() {
            "entry-existing-influence" => rules::definition("XQ37").abilities[0].clone(),
            "renown" => crate::renown::renown_ability(region),
            "jc089-combat-glory" => rules::jc089_combat_glory_ability(region),
            _ => return Err("XQ37不允许其他程序或能力别名".into()),
        };
        if f.steps.len() != 1
            || f.steps[0].context != f.actor
            || serde_json::to_value(&f.steps[0].op).unwrap()
                != serde_json::to_value(&a.ops[0]).unwrap()
        {
            return Err("XQ37仅准入完整原子进场程序或原获授奖励".into());
        }
        Ok(())
    }
    pub(crate) fn validate_xq37_state(&self) -> RuleResult<()> {
        let declaration = |d: &Declaration| -> RuleResult<()> {
            if d.source.card.definition != "XQ37"
                && d.ability.key != "entry-existing-influence"
                && !contains_xq37_op(&d.ability.ops)
                && !d.ability.modes.iter().any(|m| contains_xq37_op(&m.ops))
            {
                return Ok(());
            }
            rules::validate_ability(&d.source.card.definition, &d.ability)?;
            let c = &d.source.card;
            if c.definition != "XQ37"
                || d.actor >= self.players.len()
                || c.owner >= self.players.len()
                || c.controller != d.actor
                || c.face_down
                || c.id.is_empty()
                || d.source.region.is_none_or(|r| r >= self.regions.len())
                || d.source
                    .source_region_instance
                    .as_ref()
                    .is_none_or(|id| id.is_empty())
                || d.source.observed_death.is_some()
                || d.source.attachment_host_instance.is_some()
                || d.source.play_source.is_some()
            {
                return Err("XQ37进场声明的冻结来源或地区不符".into());
            }
            if let [Op::PlaceInfluence {
                region_instance, ..
            }] = d.ability.ops.as_slice()
            {
                if d.source.source_region_instance.as_ref() != Some(region_instance) {
                    return Err("XQ37获授奖励原地区不符".into());
                }
            }
            Ok(())
        };
        for s in &self.stack {
            if let Some(f) = &s.frame {
                self.validate_xq37_frame(f)?;
                if relevant(f) {
                    if s.controller != f.actor || s.id != f.frame_id || s.target.is_some() {
                        return Err("XQ37堆栈实体与冻结帧不符".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if s.card.as_ref().is_none_or(|c| {
                            c.definition != "XQ37"
                                || c.face_down
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                        }) || s.deploy_region != f.source.region
                            || s.reveal != (f.ability_key == "reveal")
                        {
                            return Err("XQ37原派遣/现身堆栈卡或地区不符".into());
                        }
                    } else if s.card.is_some() || s.deploy_region.is_some() || s.reveal {
                        return Err("XQ37触发堆栈不能伪造派遣或现身卡".into());
                    }
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "XQ37") {
                return Err("XQ37派遣/现身堆栈缺少原子帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame } => self.validate_xq37_frame(frame)?,
                Effect::Declare { declaration: d } => declaration(d)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Frame { frame, .. } if relevant(frame) => {
                    return Err("XQ37原子程序不能暂停为帧选择".into())
                }
                ChoiceResolution::Declare { declaration: d, .. } => declaration(d)?,
                _ => {}
            }
        }
        Ok(())
    }
}
