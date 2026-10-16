//! JZ35 only: one atomic sacrifice/return effect, with actual departure receipts.
use crate::{
    catalog,
    death_observers::CharacterDeathReceipt,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::{BTreeMap, BTreeSet};
const KEY: &str = "jz35-sacrifice-return-controlled-hosts";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn has_op(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JZ35SacrificeReturnControlledHosts => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => has_op(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => has_op(std::slice::from_ref(exhausted)) || has_op(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "JZ35"
        || f.ability_key == KEY
        || f.steps.iter().any(|s| has_op(std::slice::from_ref(&s.op)))
}
fn declaration_related(d: &Declaration) -> bool {
    d.source.card.definition == "JZ35"
        || d.ability.key == KEY
        || has_op(&d.ability.ops)
        || d.ability.modes.iter().any(|m| has_op(&m.ops))
}
pub(crate) fn definition() -> Definition {
    let mut play = rules::ability(
        KEY,
        "标准行动：牺牲所有本方宿主并返回原地区",
        Timing::Standard,
        vec![],
        vec![],
        vec![Op::JZ35SacrificeReturnControlledHosts],
        None,
    );
    play.play_only = true;
    Definition {
        abilities: vec![play],
        ..Definition::default()
    }
}
pub(crate) fn validate_play_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "JZ35"
        && (a.region.is_some()
            || a.target_id.is_some()
            || a.ability_id.as_deref().is_some_and(|key| key != KEY)
            || a.option.is_some()
            || a.choice_id.is_some()
            || a.selected.is_some()
            || a.top.is_some()
            || a.bottom.is_some()
            || a.allocations.is_some()
            || a.cost_selected.is_some()
            || a.deck_draft.is_some())
    {
        return Err("JZ35不指定目标、地区或额外牺牲费用".into());
    }
    Ok(())
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if (id == "JZ35" || a.key == KEY || has_op(&a.ops) || a.modes.iter().any(|m| has_op(&m.ops)))
        && (id != "JZ35" || !same(a, &definition().abilities[0]))
    {
        return Err("只准入完整JZ35本方宿主牺牲返回程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "JZ35" && !same(d, &definition()))
        || (id != "JZ35"
            && d.abilities
                .iter()
                .any(|a| a.key == KEY || has_op(&a.ops) || a.modes.iter().any(|m| has_op(&m.ops))))
    {
        return Err("JZ35完整定义不能改写或移植".into());
    }
    Ok(())
}

/// Ephemeral links captured in this single effect; not a save-state continuation.
#[derive(Clone, Debug)]
pub(crate) struct HostReturn {
    pub death: CharacterDeathReceipt,
    pub definition: String,
    pub region: usize,
    pub region_instance: String,
}
impl Game {
    pub(crate) fn jz35_sacrifice_hosts(&mut self, actor: usize) -> Vec<HostReturn> {
        // Resolve current control/type after all responses; freeze the whole wave
        // before a departure removes an attachment or a control grant.
        let mut locations = BTreeMap::new();
        let mut deaths = Vec::new();
        for (r, region) in self.regions.iter().enumerate().filter(|(_, r)| !r.vacant) {
            for c in &region.cards {
                if c.controller == actor
                    && !c.face_down
                    && catalog::card(&c.definition).kind == "character"
                    && self.current_subtypes(c).iter().any(|s| s == "宿主")
                {
                    locations.insert(
                        c.id.clone(),
                        (r, region.card.id.clone(), c.definition.clone()),
                    );
                    deaths.push((
                        c.id.clone(),
                        RemovalCause::Sacrifice,
                        self.source_snapshot(c, Some(r)),
                    ));
                }
            }
        }
        self.remove_death_batch_with_receipts(deaths)
            .into_iter()
            .map(|death| {
                let (region, region_instance, definition) = locations
                    .remove(&death.old_instance)
                    .expect("actual death receipt belongs to the frozen host wave");
                HostReturn {
                    death,
                    definition,
                    region,
                    region_instance,
                }
            })
            .collect()
    }
    pub(crate) fn jz35_return_hosts(&mut self, returns: &[HostReturn]) -> RuleResult<()> {
        for ret in returns {
            let original_region = self
                .regions
                .get(ret.region)
                .is_some_and(|r| !r.vacant && r.card.id == ret.region_instance);
            let actual_grave_card = self.players.get(ret.death.owner).is_some_and(|p| {
                p.graveyard.iter().any(|c| {
                    c.id == ret.death.grave_instance
                        && c.owner == ret.death.owner
                        && c.definition == ret.definition
                        && catalog::card(&c.definition).kind == "character"
                })
            });
            // No other copy can substitute for a missing/redirected card or region.
            if original_region && actual_grave_card {
                // General rule: owner controls a new entry unless the card says otherwise.
                self.world_graveyard_entry(ret.death.owner, ret.region, &ret.death.grave_instance)?;
            }
        }
        Ok(())
    }
    pub(crate) fn jz35_resolve(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        let returns = self.jz35_sacrifice_hosts(f.actor);
        self.jz35_return_hosts(&returns)
        // Declaration processing waits until the entire effect completes.
    }
    pub(crate) fn validate_jz35_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_mobility_frame(f)? { return Ok(()); }
        if !related(f) {
            return Ok(());
        }
        let c = &f.source.card;
        if c.definition != "JZ35"
            || c.id.is_empty()
            || c.face_down
            || f.actor >= self.players.len()
            || c.owner >= self.players.len()
            || c.controller != f.actor
            || f.frame_id.is_empty()
            || f.ability_key != KEY
            || f.cursor != 0
            || f.source.region.is_some()
            || f.source.source_region_instance.is_some()
            || f.source.play_source.is_some()
            || f.source.attachment_host_instance.is_some()
            || f.source.observed_death.is_some()
            || f.chosen_region.is_some()
            || !f.targets.is_empty()
            || !same(
                &f.steps,
                &vec![Step {
                    context: f.actor,
                    op: Op::JZ35SacrificeReturnControlledHosts,
                }],
            )
            || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)]
                if ids.len() <= 2 && ids.iter().all(|id| !id.is_empty())
                    && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
        {
            return Err("JZ35冻结来源、支付或固定牺牲返回帧无效".into());
        }
        Ok(())
    }
    pub(crate) fn validate_jz35_state(&self) -> RuleResult<()> {
        for s in &self.stack {
            if let Some(f) = &s.frame {
                if self.validated_mobility_frame(f)? { continue; }
                self.validate_jz35_frame(f)?;
                if related(f) || s.card.as_ref().is_some_and(|c| c.definition == "JZ35") {
                    if !related(f)
                        || !matches!(f.guard, GuardState::Unchecked)
                        || s.id != f.frame_id
                        || s.controller != f.actor
                        || s.deploy_region.is_some()
                        || s.reveal
                        || s.target.is_some()
                        || s.card.as_ref().is_none_or(|c| {
                            c.definition != "JZ35"
                                || c.face_down
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                        })
                    {
                        return Err("JZ35事务必须保留未结算的响应堆栈外壳".into());
                    }
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "JZ35") {
                return Err("JZ35事务缺少固定帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame } if crate::mobility::related_frame(frame) => return Err("机动不能绕过响应队列".into()),
                Effect::Frame { frame } => {
                    self.validate_jz35_frame(frame)?;
                    if related(frame) {
                        return Err("JZ35不能绕过响应堆栈或重复执行".into());
                    }
                }
                Effect::Declare { declaration } if declaration_related(declaration) => {
                    return Err("JZ35不是可触发声明".into())
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration, .. } if self.validated_mobility_declaration(declaration)? => {
                    self.validate_mobility_state()?;
                }
                ChoiceResolution::Frame { frame, .. } if crate::mobility::related_frame(frame) => return Err("机动没有帧内续体".into()),
                ChoiceResolution::Frame { frame, .. } if related(frame) => {
                    return Err("JZ35无可持久化的牺牲返回续体".into())
                }
                ChoiceResolution::Declare { declaration, .. }
                    if declaration_related(declaration) =>
                {
                    return Err("JZ35无触发待选".into())
                }
                _ => {}
            }
        }
        Ok(())
    }
}
