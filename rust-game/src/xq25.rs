//! Complete XQ25: only a live, face-up, untapped region attachment adds one.
use crate::{
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::BTreeSet;
const KEY: &str = "xq25-attach-region";
pub(crate) fn validate_play_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "XQ25"
        && (a.region.is_none()
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
        return Err("XQ25只接受地区结附行动，不能携带额外选择或支付程序".into());
    }
    Ok(())
}
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition() -> Definition {
    let host = rules::target(
        Zone::Region,
        EntityKind::Any,
        Relation::Any,
        Range::Anywhere,
    );
    let mut play = rules::ability(
        KEY,
        "标准行动：结附于目标地区",
        Timing::Standard,
        vec![],
        vec![host.clone()],
        vec![],
        None,
    );
    play.play_only = true;
    Definition {
        abilities: vec![play],
        modifiers: vec![StaticModifier::XQ25UntappedRegionConcealSurcharge],
        attachment: Some(AttachmentSpec {
            controls_host: false,
            host_subtype_change: SubtypeChange::None,
            host,
            host_icons: Icons::default(),
            host_temporary_icons: None,
            host_barrier: false,
            host_defense_bonus: 0,
            host_leaves: HostLeaveDestination::OwnerGraveyard,
        }),
        ..Definition::default()
    }
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if (id == "XQ25" || a.key == KEY) && (id != "XQ25" || !same(a, &definition().abilities[0])) {
        return Err("只准入XQ25完整地区结附行动".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "XQ25" && !same(d, &definition()))
        || (id != "XQ25"
            && (d.abilities.iter().any(|a| a.key == KEY)
                || d.modifiers
                    .iter()
                    .any(|m| matches!(m, StaticModifier::XQ25UntappedRegionConcealSurcharge))))
    {
        return Err("XQ25地区结附或秘密派遣加费定义不能改写或移植".into());
    }
    Ok(())
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "XQ25" || f.ability_key == KEY
}
impl Game {
    pub(crate) fn secret_deploy_cost(&self, region: usize) -> u32 {
        let Some(r) = self.regions.get(region).filter(|r| !r.vacant) else {
            return 1;
        };
        self.attachments
            .iter()
            .filter(|a| {
                a.card.definition == "XQ25"
                    && !a.card.face_down
                    && !a.card.exhausted
                    && a.host_id == r.card.id
                    && self.attachment_host_valid(a)
            })
            .fold(1u32, |cost, _| cost.saturating_add(1))
    }
    pub(crate) fn validate_xq25_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        if !related(f) {
            return Ok(());
        }
        let c = &f.source.card;
        if c.definition != "XQ25"
            || c.id.is_empty()
            || f.frame_id.is_empty()
            || f.actor >= self.players.len()
            || c.owner >= self.players.len()
            || c.controller != f.actor
            || c.face_down
            || f.ability_key != KEY
            || f.source.region.is_some()
            || f.source.source_region_instance.is_some()
            || f.source.play_source.is_some()
            || f.source.attachment_host_instance.is_some()
            || f.source.observed_death.is_some()
            || !f.steps.is_empty()
            || f.cursor != 0
            || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]
                if f.money_units(0)<=2&&ids.iter().all(|id|!id.is_empty())&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
        {
            return Err("XQ25来源、支付或空结附程序无效".into());
        }
        let t = f.targets.first().ok_or("XQ25缺少目标地区")?;
        let r =
            t.id.strip_prefix("region:")
                .and_then(|n| n.parse::<usize>().ok())
                .filter(|r| *r < self.regions.len() && t.id == format!("region:{r}"));
        if f.targets.len() != 1
            || r.is_none()
            || f.chosen_region != r
            || !same(&t.spec, &definition().abilities[0].targets[0])
            || t.region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty() || id == &c.id)
            || t.public.instance_id != t.id
            || t.public.kind != "region"
            || t.public.region != r
            || t.public.owner.is_some()
            || t.public.controller.is_some()
            || !t.public.valid
            || t.public.invalid_reason.is_some()
        {
            return Err("XQ25原地区目标或冻结实例无效".into());
        }
        Ok(())
    }
    pub(crate) fn validate_xq25_state(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        for a in self
            .attachments
            .iter()
            .filter(|a| a.card.definition == "XQ25")
        {
            if a.card.id.is_empty()
                || !seen.insert(&a.card.id)
                || a.card.owner >= self.players.len()
                || a.card.controller >= self.players.len()
                || a.card.face_down
                || !self.attachment_host_valid(a)
            {
                return Err("XQ25在场实例、操控者或实际地区宿主无效".into());
            }
        }
        for s in &self.stack {
            if let Some(f) = &s.frame {
                if self.validated_cross_card_frame(f)? { continue; }
                self.validate_xq25_frame(f)?;
                if related(f) || s.card.as_ref().is_some_and(|c| c.definition == "XQ25") {
                    if f.source.card.definition != "XQ25"
                        || !matches!(f.guard, GuardState::Unchecked)
                        || s.id != f.frame_id
                        || s.controller != f.actor
                        || s.deploy_region.is_some()
                        || s.reveal
                        || s.target != f.targets.first().map(|t| t.id.clone())
                        || s.card.as_ref().is_none_or(|c| {
                            c.definition != "XQ25"
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                                || c.face_down
                        })
                    {
                        return Err("XQ25实体结附卡或堆栈外壳无效".into());
                    }
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "XQ25") {
                return Err("XQ25结附堆栈缺少冻结帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame } if crate::mobility::related_frame(frame) => return Err("机动不能绕过响应队列".into()),
                Effect::Frame { frame: f } => {
                    self.validate_xq25_frame(f)?;
                    if related(f) {
                        return Err("XQ25结附不能绕过响应堆栈".into());
                    }
                }
                Effect::Declare { declaration: d }
                    if d.source.card.definition == "XQ25" || d.ability.key == KEY =>
                {
                    return Err("XQ25没有触发声明".into())
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration, .. } if self.validated_cross_card_declaration(declaration)? => {
                    self.validate_mobility_state()?;
                }
                ChoiceResolution::Frame { frame, .. } if crate::mobility::related_frame(frame) => return Err("机动没有帧内续体".into()),
                ChoiceResolution::Declare { declaration: d, .. }
                    if d.source.card.definition == "XQ25" || d.ability.key == KEY =>
                {
                    return Err("XQ25没有触发待选".into())
                }
                ChoiceResolution::Frame { frame: f, .. } if related(f) => {
                    return Err("XQ25没有结附续体待选".into())
                }
                _ => {}
            }
        }
        Ok(())
    }
}
