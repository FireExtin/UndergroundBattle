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
// Only the two existing exact granted reward programs are runtime exceptions.
// A key string alone never admits a program or changes its original region.
pub(crate) fn runtime_ability_is_admitted(a: &rules::AbilitySpec) -> bool {
    let [Op::PlaceInfluence {
        region_instance,
        amount: 1,
    }] = a.ops.as_slice()
    else {
        return false;
    };
    if region_instance.is_empty() {
        return false;
    }
    let expected = match a.key.as_str() {
        "renown" => crate::renown::renown_ability(region_instance),
        "jc089-combat-glory" => rules::jc089_combat_glory_ability(region_instance),
        _ => return false,
    };
    serde_json::to_value(a).unwrap() == serde_json::to_value(expected).unwrap()
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
        if frame.source.card.definition != "JZ22" {
            if frame.ability_key == "entry-low-hand-influence"
                || frame
                    .steps
                    .iter()
                    .any(|s| contains_jz22_op(std::slice::from_ref(&s.op)))
            {
                return Err("JZ22进场程序不能移植到其他来源或能力".into());
            }
            return Ok(());
        }
        if matches!(frame.ability_key.as_str(), "deploy" | "reveal") {
            if !frame.steps.is_empty()
                || !frame.targets.is_empty()
                || frame.cursor != 0
                || !matches!(frame.guard, GuardState::Unchecked)
                || frame.actor >= self.players.len()
                || frame.source.card.owner >= self.players.len()
                || frame.source.card.controller != frame.actor
                || frame.source.card.id.is_empty()
                || frame.source.region.is_none_or(|r| r >= self.regions.len())
                || frame.chosen_region != frame.source.region
                || frame
                    .source
                    .source_region_instance
                    .as_ref()
                    .is_none_or(|id| id.is_empty())
                || frame.source.observed_death.is_some()
                || frame.source.attachment_host_instance.is_some()
            {
                return Err("JZ22派遣或现身空程序与来源不符".into());
            }
            let origin_matches = if frame.ability_key == "deploy" {
                !frame.source.card.face_down
                    && (frame.source.play_source == Some(PlaySource::Hand)
                        || frame.source.play_source == Some(PlaySource::Graveyard)
                            && rules::definition("JZ22").graveyard_face_up)
            } else {
                frame.source.card.face_down && frame.source.play_source.is_none()
            };
            if !origin_matches {
                return Err("JZ22派遣或现身的原来源标记不符".into());
            }
            return Ok(());
        }
        jz22_source(self, frame.actor, &frame.source)?;
        let region = frame.source.source_region_instance.as_deref().unwrap();
        let a = match frame.ability_key.as_str() {
            "entry-low-hand-influence" => rules::definition("JZ22").abilities[0].clone(),
            "renown" => crate::renown::renown_ability(region),
            "jc089-combat-glory" => rules::jc089_combat_glory_ability(region),
            _ => return Err("JZ22来源不允许其他程序或能力别名".into()),
        };
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
                && (pending_choice || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked))
            {
                return Err("JZ22原子程序不能保存已执行游标、已通过守卫或帧选择".into());
            }
            Ok(())
        };
        let declaration = |d: &Declaration| -> RuleResult<()> {
            if d.source.card.definition == "JZ22"
                || d.ability.key == "entry-low-hand-influence"
                || contains_jz22_op(&d.ability.ops)
                || d.ability.modes.iter().any(|m| contains_jz22_op(&m.ops))
            {
                rules::validate_ability(&d.source.card.definition, &d.ability)?;
                jz22_source(self, d.actor, &d.source)?;
                if let [Op::PlaceInfluence {
                    region_instance, ..
                }] = d.ability.ops.as_slice()
                {
                    if d.source.source_region_instance.as_ref() != Some(region_instance) {
                        return Err("JZ22获授奖励只能绑定冻结的原地区".into());
                    }
                }
            }
            Ok(())
        };
        for s in &self.stack {
            if let Some(f) = &s.frame {
                frame(f, false)?;
                if f.source.card.definition == "JZ22"
                    && (s.controller != f.actor || s.id != f.frame_id)
                {
                    return Err("JZ22堆栈对象与冻结帧不符".into());
                }
                if f.source.card.definition == "JZ22"
                    && matches!(f.ability_key.as_str(), "deploy" | "reveal")
                    && (s
                        .card
                        .as_ref()
                        .is_none_or(|c| c.definition != "JZ22" || c.owner != f.source.card.owner)
                        || s.deploy_region != f.source.region
                        || s.reveal != (f.ability_key == "reveal"))
                {
                    return Err("JZ22派遣或现身的堆栈实体/地区不符".into());
                }
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
