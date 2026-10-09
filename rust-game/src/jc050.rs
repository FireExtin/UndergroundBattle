//! The complete JC050 transaction reuses region choice and simultaneous removal.
use crate::{engine::RuleResult, model::*, rules::Op};

pub(crate) fn contains_jc050_op(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JC050DestroyChosenRegionCharacters => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => {
            contains_jc050_op(body)
        }
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => {
            contains_jc050_op(std::slice::from_ref(exhausted))
                || contains_jc050_op(std::slice::from_ref(ready))
        }
        _ => false,
    })
}
fn relevant(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "JC050"
        || f.ability_key == "region-destruction"
        || f.steps
            .iter()
            .any(|s| contains_jc050_op(std::slice::from_ref(&s.op)))
}
impl Game {
    pub(crate) fn validate_jc050_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !relevant(f) {
            return Ok(());
        }
        let c = &f.source.card;
        if c.definition != "JC050"
            || f.actor >= self.players.len()
            || c.controller != f.actor
            || c.owner >= self.players.len()
            || c.id.is_empty()
            || !f.targets.is_empty()
        {
            return Err("JC050必须是原事务的无目标帧".into());
        }
        // A transaction cannot use character deployment. Its genuine paid
        // hidden reveal is the existing empty program in its original region.
        if f.ability_key == "reveal"
            && f.steps.is_empty()
            && f.cursor == 0
            && matches!(f.guard, GuardState::Unchecked)
            && c.face_down
            && f.source.region.is_some_and(|r| r < self.regions.len())
            && f.chosen_region == f.source.region
            && f.source.source_region_instance.is_none()
            && f.source.play_source.is_none()
            && f.source.observed_death.is_none()
            && f.source.attachment_host_instance.is_none()
        {
            return Ok(());
        }
        if f.ability_key != "region-destruction"
            || c.face_down
            || f.cursor > 1
            || f.steps.len() != 2
            || !matches!(f.steps[0].op, Op::ChooseRegion)
            || !matches!(f.steps[1].op, Op::JC050DestroyChosenRegionCharacters)
            || f.steps.iter().any(|s| s.context != f.actor)
            || (f.cursor == 0
                && (!matches!(f.guard, GuardState::Unchecked) || f.chosen_region.is_some()))
            || (f.cursor == 1 && !matches!(f.guard, GuardState::Accepted))
            || f.chosen_region.is_some_and(|r| r >= self.regions.len())
        {
            return Err("JC050原地区选择/消灭程序或游标不符".into());
        }
        Ok(())
    }
    pub(crate) fn validate_jc050_state(&self) -> RuleResult<()> {
        for s in &self.stack {
            if let Some(f) = &s.frame {
                self.validate_jc050_frame(f)?;
                if relevant(f)
                    && (f.cursor != 0
                        || s.controller != f.actor
                        || s.id != f.frame_id
                        || s.card.as_ref().is_none_or(|c| {
                            c.definition != "JC050" || c.owner != f.source.card.owner
                        }))
                {
                    return Err("JC050堆栈必须保留尚未执行的原事务".into());
                }
                if relevant(f)
                    && (s.target.is_some()
                        || (f.ability_key == "region-destruction"
                            && (s.deploy_region.is_some() || s.reveal))
                        || (f.ability_key == "reveal"
                            && (!s.reveal || s.deploy_region != f.source.region)))
                {
                    return Err("JC050原事务/现身堆栈的地区、现身或目标标记不符".into());
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "JC050") {
                return Err("JC050堆栈缺少原事务帧".into());
            }
        }
        let declaration = |d: &Declaration| -> RuleResult<()> {
            if d.source.card.definition == "JC050"
                || d.ability.key == "region-destruction"
                || contains_jc050_op(&d.ability.ops)
                || d.ability.modes.iter().any(|m| contains_jc050_op(&m.ops))
            {
                return Err("JC050事务不能移植为触发声明".into());
            }
            Ok(())
        };
        for e in &self.effects {
            match e {
                Effect::Frame { frame } => {
                    self.validate_jc050_frame(frame)?;
                    if relevant(frame) && frame.cursor != 0 {
                        return Err("JC050不能排入已执行帧".into());
                    }
                }
                Effect::Declare { declaration: d } => declaration(d)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Frame { frame, choice } => {
                    self.validate_jc050_frame(frame)?;
                    if relevant(frame)
                        && (frame.cursor != 1
                            || frame.chosen_region.is_some()
                            || !matches!(choice, FrameChoice::Region)
                            || p.seat != frame.actor
                            || p.choice.player_id != format!("p{}", frame.actor)
                            || p.choice.kind != "target"
                            || p.choice.amount.is_some()
                            || !p.choice.preview_cards.is_empty()
                            || p.choice.allow_decline != Some(false)
                            || p.choice.title != "选择要消灭其中角色与暗藏者的地区"
                            || p.choice.description != "选择符合数量限制的选项，然后确认。"
                            || p.choice.min != Some(1)
                            || p.choice.max != Some(1)
                            || p.choice.options.len() != self.regions.len()
                            || p.choice
                                .options
                                .iter()
                                .enumerate()
                                .any(|(r, o)| o.id != format!("region:{r}") || o.card.is_some()))
                    {
                        return Err("JC050只能等待原操控者选择地区".into());
                    }
                }
                ChoiceResolution::Declare { declaration: d, .. } => declaration(d)?,
                _ => {}
            }
        }
        Ok(())
    }
}
