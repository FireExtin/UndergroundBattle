//! XQ27 has one printed non-targeted standard transaction and no resolution choice.
use crate::{engine::RuleResult, model::*, rules::Op};
pub(crate) fn contains_xq27_op(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::XQ27DestroyAllHidden => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => {
            contains_xq27_op(body)
        }
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => {
            contains_xq27_op(std::slice::from_ref(exhausted))
                || contains_xq27_op(std::slice::from_ref(ready))
        }
        _ => false,
    })
}
fn relevant(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "XQ27"
        || f.ability_key == "hidden-sweep"
        || f.steps
            .iter()
            .any(|s| contains_xq27_op(std::slice::from_ref(&s.op)))
}
fn declaration(d: &Declaration) -> RuleResult<()> {
    if d.source.card.definition == "XQ27"
        || d.ability.key == "hidden-sweep"
        || contains_xq27_op(&d.ability.ops)
        || d.ability.modes.iter().any(|m| contains_xq27_op(&m.ops))
    {
        return Err("XQ27事务不能移植为触发声明".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn validate_xq27_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !relevant(f) {
            return Ok(());
        }
        let c = &f.source.card;
        if c.definition != "XQ27"
            || f.actor >= self.players.len()
            || c.controller != f.actor
            || c.owner >= self.players.len()
            || c.id.is_empty()
            || !f.targets.is_empty()
            || f.cursor != 0
            || !matches!(f.guard, GuardState::Unchecked)
            || f.source.source_region_instance.is_some()
            || f.source.attachment_host_instance.is_some()
            || f.source.play_source.is_some()
            || f.source.observed_death.is_some()
        {
            return Err("XQ27原事务来源、目标或执行状态不符".into());
        }
        // Paying to reveal a hidden transaction uses the existing empty reveal
        // program. It does not cast its printed standard action.
        if f.ability_key == "reveal"
            && c.face_down
            && f.steps.is_empty()
            && f.source.region.is_some_and(|r| r < self.regions.len())
            && f.chosen_region == f.source.region
        {
            return Ok(());
        }
        if f.ability_key != "hidden-sweep"
            || c.face_down
            || f.source.region.is_some()
            || f.chosen_region.is_some()
            || f.steps.len() != 1
            || !matches!(f.steps[0].op, Op::XQ27DestroyAllHidden)
            || f.steps[0].context != f.actor
        {
            return Err("XQ27只准入原无目标消灭程序或真实付费现身".into());
        }
        Ok(())
    }
    pub(crate) fn validate_xq27_state(&self) -> RuleResult<()> {
        for s in &self.stack {
            if let Some(f) = &s.frame {
                self.validate_xq27_frame(f)?;
                if relevant(f)
                    && (s.controller != f.actor
                        || s.id != f.frame_id
                        || s.target.is_some()
                        || s.card.as_ref().is_none_or(|c| {
                            c.definition != "XQ27"
                                || c.face_down
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                        })
                        || (f.ability_key == "hidden-sweep"
                            && (s.deploy_region.is_some() || s.reveal))
                        || (f.ability_key == "reveal"
                            && (!s.reveal || s.deploy_region != f.source.region)))
                {
                    return Err("XQ27原事务堆栈卡、地区或现身标记不符".into());
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "XQ27") {
                return Err("XQ27堆栈缺少原事务帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame } => self.validate_xq27_frame(frame)?,
                Effect::Declare { declaration: d } => declaration(d)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Frame { frame, .. } if relevant(frame) => {
                    return Err("XQ27原程序不能暂停为选择或重复执行".into())
                }
                ChoiceResolution::Declare { declaration: d, .. } => declaration(d)?,
                _ => {}
            }
        }
        Ok(())
    }
}
