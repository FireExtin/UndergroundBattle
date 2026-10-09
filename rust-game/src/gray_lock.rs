//! Restore/execute guards for only the two admitted gray lock cards.
use crate::{engine::RuleResult, model::*, rules::{self, Op, TargetPredicate}};

fn contains_lock_op(op: &Op) -> bool {
    match op {
        Op::JC069LockTarget | Op::JC069ChaseLockedTarget | Op::JZ43LockOrDamageLocalTarget => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => body.iter().any(contains_lock_op),
        Op::IfTargetExhausted { exhausted, ready, .. } => contains_lock_op(exhausted) || contains_lock_op(ready),
        _ => false,
    }
}
impl Game {
    fn validate_persisted_gray_lock_frame(&self, frame: &ResolutionFrame, pending_choice: bool) -> RuleResult<()> {
        self.validate_gray_lock_frame(frame)?;
        // These three closed programs each execute one atomic operation. They
        // never suspend after accepting the target guard or create a frame choice.
        // Preserve the runtime validator separately: an executing frame may have
        // Accepted internally, but no persisted gray frame can reach that state.
        if matches!(frame.source.card.definition.as_str(), "JC069" | "JZ43")
            && (pending_choice || !matches!(frame.guard, GuardState::Unchecked) || frame.cursor != 0)
        { return Err("灰色锁定原子程序不能保存已通过守卫、已执行游标或帧选择".into()); }
        Ok(())
    }
    pub(crate) fn validate_gray_lock_frame(&self, frame: &ResolutionFrame) -> RuleResult<()> {
        let printed = matches!(frame.source.card.definition.as_str(), "JC069" | "JZ43")
            .then(|| rules::definition(&frame.source.card.definition).abilities.iter()
                .find(|a| a.key == frame.ability_key)).flatten();
        let relevant = printed.is_some() || frame.steps.iter().any(|s| contains_lock_op(&s.op))
            || frame.targets.iter().any(|t| t.spec.predicate == Some(TargetPredicate::JC069LockedAnchor));
        if !relevant { return Ok(()); }
        let a = printed.ok_or("灰色锁定程序不能移植到其他来源或能力")?;
        let source = &frame.source;
        let valid_paid = if frame.ability_key == "chase-locked" {
            matches!(frame.already_paid.as_slice(), [PaidCost::Assets(ids)]
                if ids.len() == 2 && ids[0] != ids[1] && ids.iter().all(|id| !id.is_empty()))
        } else { frame.already_paid.is_empty() };
        if frame.actor >= self.players.len() || source.card.owner >= self.players.len()
            || source.card.controller != frame.actor || source.card.face_down || source.card.id.is_empty()
            || source.region.is_none_or(|r| r >= self.regions.len())
            || (source.card.definition == "JZ43" && source.source_region_instance.as_ref().is_none_or(|id| id.is_empty()))
            || frame.frame_id.is_empty() || frame.steps.len() != 1 || frame.cursor > 1
            || frame.steps[0].context != frame.actor
            || serde_json::to_value(&frame.steps[0].op).unwrap() != serde_json::to_value(&a.ops[0]).unwrap()
            || frame.targets.len() != 1 || frame.targets[0].id.is_empty()
            || frame.targets[0].public.instance_id != frame.targets[0].id
            || frame.targets[0].region_instance.is_some()
            || serde_json::to_value(&frame.targets[0].spec).unwrap() != serde_json::to_value(&a.targets[0]).unwrap()
            || !valid_paid
        { return Err("灰色锁定完整程序、实例、目标或冻结行动者不符".into()); }
        Ok(())
    }
    pub(crate) fn validate_gray_lock_state(&self) -> RuleResult<()> {
        let declaration = |d: &Declaration| -> RuleResult<()> {
            let relevant = matches!(d.source.card.definition.as_str(), "JC069" | "JZ43")
                || d.ability.ops.iter().any(contains_lock_op)
                || d.ability.modes.iter().any(|m| m.ops.iter().any(contains_lock_op))
                || d.ability.targets.iter().chain(d.ability.modes.iter().flat_map(|m| &m.targets))
                    .any(|t| t.predicate == Some(TargetPredicate::JC069LockedAnchor));
            if relevant {
                rules::validate_ability(&d.source.card.definition, &d.ability)?;
                if d.actor >= self.players.len() || d.source.card.controller != d.actor
                    || d.source.card.owner >= self.players.len() || d.source.card.face_down
                    || d.source.card.id.is_empty() || d.source.region.is_none_or(|r| r >= self.regions.len())
                    || (d.source.card.definition == "JZ43" && d.source.source_region_instance.as_ref().is_none_or(|id| id.is_empty()))
                { return Err("灰色锁定声明实例或冻结行动者不符".into()); }
            }
            Ok(())
        };
        for item in &self.stack {
            if let Some(frame) = &item.frame { self.validate_persisted_gray_lock_frame(frame, false)?; }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame } => self.validate_persisted_gray_lock_frame(frame, false)?,
                Effect::Declare { declaration: d } => declaration(d)?,
                _ => {},
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Frame { frame, .. } => self.validate_persisted_gray_lock_frame(frame, true)?,
                ChoiceResolution::Declare { declaration: d, .. } => declaration(d)?,
                _ => {},
            }
        }
        Ok(())
    }
}
