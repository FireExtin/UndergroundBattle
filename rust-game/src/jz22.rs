//! One closed entry program; no new persistent binding or choice mechanism.
use crate::{
    engine::RuleResult,
    model::*,
    rules::{self, Op},
};

pub(crate) fn contains_jz22_op(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JZ22LowHandInfluenceInSourceRegion => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => {
            contains_jz22_op(body)
        }
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => {
            contains_jz22_op(std::slice::from_ref(exhausted))
                || contains_jz22_op(std::slice::from_ref(ready))
        }
        _ => false,
    })
}
fn jz22_source(g: &Game, actor: usize, source: &SourceSnapshot) -> RuleResult<()> {
    if actor >= g.players.len()
        || source.card.owner >= g.players.len()
        || source.card.controller != actor
        || source.card.definition != "JZ22"
        || source.card.face_down
        || source.card.id.is_empty()
        || source.region.is_none_or(|r| r >= g.regions.len())
        || source
            .source_region_instance
            .as_ref()
            .is_none_or(|id| id.is_empty())
        || source.observed_death.is_some()
        || source.attachment_host_instance.is_some()
        || source.play_source.is_some()
    {
        return Err("JZ22进场来源、冻结行动者或原地区实例不符".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn jz22_low_hand_influence(&mut self, frame: &ResolutionFrame) {
        // Same living enemy-team predicate as JZ24, evaluated after responses.
        let qualifying = self.players.iter().any(|p| {
            !p.eliminated && self.team(p.seat) != self.team(frame.actor) && p.hand.len() <= 3
        });
        if qualifying {
            if let Some(r) = self.blue_original_region(frame) {
                self.place_influence(frame.actor, r, 1);
            }
        }
    }
    pub(crate) fn validate_jz22_frame(&self, frame: &ResolutionFrame) -> RuleResult<()> {
        let printed = frame.source.card.definition == "JZ22"
            && frame.ability_key == "entry-low-hand-influence";
        if !printed
            && !frame
                .steps
                .iter()
                .any(|s| contains_jz22_op(std::slice::from_ref(&s.op)))
        {
            return Ok(());
        }
        if !printed {
            return Err("JZ22进场程序不能移植到其他来源或能力".into());
        }
        jz22_source(self, frame.actor, &frame.source)?;
        let a = &rules::definition("JZ22").abilities[0];
        if frame.frame_id.is_empty()
            || frame.cursor > 1
            || frame.steps.len() != 1
            || frame.steps[0].context != frame.actor
            || !frame.targets.is_empty()
            || !frame.already_paid.is_empty()
            || serde_json::to_value(&frame.steps[0].op).unwrap()
                != serde_json::to_value(&a.ops[0]).unwrap()
        {
            return Err("JZ22完整原子程序、游标或支付记录不符".into());
        }
        Ok(())
    }
    pub(crate) fn validate_jz22_state(&self) -> RuleResult<()> {
        let frame = |f: &ResolutionFrame, pending_choice: bool| -> RuleResult<()> {
            self.validate_jz22_frame(f)?;
            if f.source.card.definition == "JZ22"
                && f.ability_key == "entry-low-hand-influence"
                && (pending_choice || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked))
            {
                return Err("JZ22原子程序不能保存已执行游标、已通过守卫或帧选择".into());
            }
            Ok(())
        };
        let declaration = |d: &Declaration| -> RuleResult<()> {
            if (d.source.card.definition == "JZ22" && d.ability.key != "renown")
                || contains_jz22_op(&d.ability.ops)
                || d.ability.modes.iter().any(|m| contains_jz22_op(&m.ops))
            {
                rules::validate_ability(&d.source.card.definition, &d.ability)?;
                jz22_source(self, d.actor, &d.source)?;
            }
            Ok(())
        };
        for s in &self.stack {
            if let Some(f) = &s.frame {
                frame(f, false)?;
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame: f } => frame(f, false)?,
                Effect::Declare { declaration: d } => declaration(d)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Frame { frame: f, .. } => frame(f, true)?,
                ChoiceResolution::Declare { declaration: d, .. } => declaration(d)?,
                _ => {}
            }
        }
        Ok(())
    }
}
