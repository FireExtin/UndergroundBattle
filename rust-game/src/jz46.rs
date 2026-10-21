//! Closed JZ46 program: inspect one original-region hidden card privately,
//! then exhaust and lock it iff its actual front is a criminal. No reveal.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;
const PLAY: &str = "jz46-attach-region";
const INSPECT: &str = "jz46-inspect-hidden";
const CHOICE: &str = "jz46-hidden-inspect";
const TITLE: &str = "狴犴铜雕：检视所选暗藏者";
const DESCRIPTION: &str = "仅你可见。确认后完成铜雕的结算。";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JZ46InspectHiddenCriminal => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "JZ46"
        || matches!(f.ability_key.as_str(), PLAY | INSPECT)
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
pub(crate) fn definition() -> Definition {
    let host = target(
        Zone::Region,
        EntityKind::Any,
        Relation::Any,
        Range::Anywhere,
    );
    let mut play = ability(
        PLAY,
        "标准行动：结附于目标地区",
        Timing::Standard,
        vec![],
        vec![host.clone()],
        vec![],
        None,
    );
    play.play_only = true;
    let mut inspect = ability(
        INSPECT,
        "行动：横置，检视本地区目标暗藏者，锁定罪犯",
        Timing::Standard,
        vec![Cost::ExhaustSource],
        vec![target(
            Zone::Board,
            EntityKind::Hidden,
            Relation::Any,
            Range::SourceRegion,
        )],
        vec![Op::JZ46InspectHiddenCriminal],
        None,
    );
    inspect.activation_only = true;
    Definition {
        abilities: vec![play, inspect],
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
        ..Default::default()
    }
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id == "JZ46"
        || matches!(a.key.as_str(), PLAY | INSPECT)
        || uses(&a.ops)
        || a.modes.iter().any(|m| uses(&m.ops))
    {
        if id != "JZ46"
            || !definition()
                .abilities
                .iter()
                .any(|expected| same(a, expected))
        {
            return Err("JZ46只准入完整地区结附及私密检视行动，不能移植或改写".into());
        }
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "JZ46" && !same(d, &definition()))
        || (id != "JZ46"
            && d.abilities.iter().any(|a| {
                matches!(a.key.as_str(), PLAY | INSPECT)
                    || uses(&a.ops)
                    || a.modes.iter().any(|m| uses(&m.ops))
            }))
    {
        return Err("JZ46整卡定义不能拆分、改写或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if id != "JZ46" {
        return Ok(());
    }
    if a.option.is_some()
        || a.choice_id.is_some()
        || a.selected.is_some()
        || a.top.is_some()
        || a.bottom.is_some()
        || a.allocations.is_some()
        || a.cost_selected.is_some()
        || a.deck_draft.is_some()
        || (a.kind == "play"
            && (a.region.is_none()
                || a.target_id.is_some()
                || a.ability_id.as_deref().is_some_and(|k| k != PLAY)))
        || (a.kind == "activate"
            && (a.region.is_some()
                || a.target_id.is_none()
                || a.ability_id.as_deref() != Some(INSPECT)))
    {
        return Err("JZ46只接受完整地区结附或单个暗藏者检视行动".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn validate_jz46_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !related(f) {
            return Ok(());
        }
        let c = &f.source.card;
        if c.definition != "JZ46"
            || c.id.is_empty()
            || c.face_down
            || f.frame_id.is_empty()
            || f.actor >= self.players.len()
            || c.owner >= self.players.len()
            || c.controller != f.actor
            || f.source.observed_death.is_some()
            || f.source.observed_mobility_grant.is_some()
            || f.source.play_source.is_some()
            || f.targets.len() != 1
        {
            return Err("JZ46冻结来源、行动者或唯一目标无效".into());
        }
        let d = definition();
        let t = &f.targets[0];
        if f.ability_key == PLAY {
            let region =
                t.id.strip_prefix("region:")
                    .and_then(|s| s.parse::<usize>().ok())
                    .filter(|r| *r < self.regions.len() && t.id == format!("region:{r}"));
            if f.source.region.is_some()
                || f.source.source_region_instance.is_some()
                || f.source.attachment_host_instance.is_some()
                || !f.steps.is_empty()
                || f.cursor != 0
                || region.is_none()
                || f.chosen_region != region
                || !same(&t.spec, &d.abilities[0].targets[0])
                || t.region_instance.as_ref().is_none_or(|id| id.is_empty())
                || t.public.instance_id != t.id
                || t.public.kind != "region"
                || t.public.region != region
                || t.public.owner.is_some()
                || t.public.controller.is_some()
                || !t.public.valid
                || t.public.status != "valid"
                || t.public.invalid_reason.is_some()
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= 2
                    && ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
            {
                return Err("JZ46地区结附、原宿主或支付帧无效".into());
            }
        } else if f.ability_key == INSPECT {
            let seat = |s: &Option<String>| {
                s.as_deref()
                    .and_then(|id| id.strip_prefix('p'))
                    .and_then(|n| n.parse::<usize>().ok())
                    .filter(|n| {
                        *n < self.players.len() && s.as_deref() == Some(format!("p{n}").as_str())
                    })
            };
            let public_controller = seat(&t.public.controller);
            if f.source.region.is_none_or(|r| r >= self.regions.len())
                || f.source
                    .source_region_instance
                    .as_ref()
                    .is_none_or(|id| id.is_empty())
                || f.source.attachment_host_instance != f.source.source_region_instance
                || c.exhausted
                || f.chosen_region.is_some()
                || t.id.is_empty()
                || t.id == c.id
                || t.region_instance.is_some()
                || !same(&t.spec, &d.abilities[1].targets[0])
                || t.public.instance_id != t.id
                || t.public.kind != "hidden"
                || t.public.region != f.source.region
                || seat(&t.public.owner).is_none()
                || public_controller.is_none()
                || public_controller
                    .is_some_and(|s| t.public.label != format!("暗藏者·{}", self.players[s].name))
                || !t.public.valid
                || t.public.status != "valid"
                || t.public.invalid_reason.is_some()
                || !matches!(f.already_paid.as_slice(), [PaidCost::Exhausted(id)] if id == &c.id)
                || f.steps.len() != 1
                || f.steps[0].context != f.actor
                || !same(&f.steps[0].op, &d.abilities[1].ops[0])
                || !((f.cursor == 0 && matches!(f.guard, GuardState::Unchecked))
                    || (f.cursor == 1 && matches!(f.guard, GuardState::Accepted)))
            {
                return Err("JZ46私密检视、原地区、横置费用或完整程序无效".into());
            }
        } else {
            return Err("JZ46不能使用其他能力帧".into());
        }
        Ok(())
    }
    fn jz46_inspected_valid(&self, f: &ResolutionFrame, inspected: &Card) -> bool {
        f.ability_key == INSPECT
            && f.cursor == 1
            && matches!(f.guard, GuardState::Accepted)
            && inspected.face_down
            && inspected.id == f.targets[0].id
            && inspected.owner < self.players.len()
            && inspected.controller < self.players.len()
            && catalog::catalog()
                .cards
                .iter()
                .any(|d| d.id == inspected.definition && d.supported && d.kind == "character")
            && self.valid_bound_target(f.actor, &f.source, &f.targets[0])
            && self
                .board(&inspected.id)
                .is_some_and(|(_, live)| same(live, inspected))
    }
    pub(crate) fn jz46_inspect_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_jz46_frame(&f)?;
        let inspected = self
            .board(&f.targets[0].id)
            .ok_or("JZ46原目标已失效")?
            .1
            .clone();
        if !self.jz46_inspected_valid(&f, &inspected) {
            return Err("JZ46检视检查点无效".into());
        }
        self.choice(
            f.actor,
            CHOICE,
            TITLE.into(),
            vec![],
            0,
            0,
            None,
            ChoiceResolution::Frame {
                frame: Box::new(f),
                choice: FrameChoice::JZ46InspectHidden { inspected },
            },
        );
        let p = self.pending.as_mut().unwrap();
        p.choice.description = DESCRIPTION.into();
        p.choice.allow_decline = Some(false);
        Ok(())
    }
    pub(crate) fn jz46_inspect_complete(
        &mut self,
        f: &ResolutionFrame,
        inspected: Card,
        chooser: usize,
        a: &Action,
        selected: &[String],
    ) -> RuleResult<()> {
        self.validate_jz46_frame(f)?;
        if chooser != f.actor
            || !selected.is_empty()
            || !self.jz46_inspected_valid(f, &inspected)
            || a.kind != "choose"
            || a.card_id.is_some()
            || a.region.is_some()
            || a.target_id.is_some()
            || a.ability_id.is_some()
            || a.option.is_some()
            || a.top.is_some()
            || a.bottom.is_some()
            || a.allocations.is_some()
            || a.cost_selected.is_some()
            || a.deck_draft.is_some()
        {
            return Err("JZ46检视确认者或冻结暗藏者无效".into());
        }
        // Hidden cards have no public subtype. Inspect the actual front, without
        // turning it face up or publishing its name/subtypes to any other seat.
        if catalog::card(&inspected.definition)
            .subtypes
            .iter()
            .any(|s| s == "罪犯")
        {
            let c = self.board_mut(&inspected.id).ok_or("JZ46原暗藏者已失效")?;
            c.exhausted = true;
            c.lock_markers = c.lock_markers.saturating_add(1);
        }
        Ok(())
    }
    pub(crate) fn jz46_private_choice(&self, p: &Pending, viewer: usize) -> Choice {
        let mut choice = p.choice.clone();
        if let ChoiceResolution::Frame {
            frame,
            choice: FrameChoice::JZ46InspectHidden { inspected },
        } = &p.resolution
        {
            if viewer == frame.actor {
                let mut front = inspected.clone();
                front.face_down = false;
                choice.preview_cards =
                    vec![self.card_view(&front, viewer, frame.source.region, None)];
            }
        }
        choice
    }
    pub(crate) fn validate_jz46_state(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        let mut frames = BTreeSet::new();
        if self
            .regions
            .iter()
            .any(|r| r.cards.iter().any(|c| c.definition == "JZ46"))
        {
            return Err("JZ46只能作为真实地区附属在场，不能充当角色或暗藏者".into());
        }
        for a in self
            .attachments
            .iter()
            .filter(|a| a.card.definition == "JZ46")
        {
            if a.card.id.is_empty()
                || !seen.insert(&a.card.id)
                || a.card.face_down
                || a.card.owner >= self.players.len()
                || a.card.controller >= self.players.len()
                || !self.attachment_host_valid(a)
            {
                return Err("JZ46在场实例或真实地区宿主无效".into());
            }
        }
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_jz46_frame(f)?;
                if related(f) {
                    if !frames.insert(&f.frame_id) {
                        return Err("JZ46不能重复同一冻结程序".into());
                    }
                    let play = f.ability_key == PLAY;
                    if item.id != f.frame_id
                        || item.controller != f.actor
                        || f.cursor != 0
                        || !matches!(f.guard, GuardState::Unchecked)
                        || item.reveal
                        || item.deploy_region.is_some()
                        || item.target != f.targets.first().map(|t| t.id.clone())
                        || (if play {
                            item.card.as_ref().is_none_or(|c| {
                                // Hand -> stack fresh() changes the UID and
                                // clears these markers; every other Card field
                                // survives and later enters from this container.
                                let mut expected = f.source.card.clone();
                                expected.id = c.id.clone();
                                expected.time_markers = 0;
                                expected.lock_markers = 0;
                                !same(c, &expected)
                            })
                        } else {
                            item.card.is_some()
                        })
                    {
                        return Err("JZ46堆栈容器或待响应检查点无效".into());
                    }
                }
            } else if item.card.as_ref().is_some_and(|c| c.definition == "JZ46") {
                return Err("JZ46结附缺少冻结帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame } => {
                    self.validate_jz46_frame(frame)?;
                    if related(frame) {
                        return Err("JZ46不能绕过响应堆栈".into());
                    }
                }
                Effect::Declare { declaration }
                    if declaration.source.card.definition == "JZ46"
                        || matches!(declaration.ability.key.as_str(), PLAY | INSPECT)
                        || uses(&declaration.ability.ops)
                        || declaration.ability.modes.iter().any(|m| uses(&m.ops)) =>
                {
                    return Err("JZ46没有触发声明".into())
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Frame { frame: f, choice } => {
                    self.validate_jz46_frame(f)?;
                    if related(f) || matches!(choice, FrameChoice::JZ46InspectHidden { .. }) {
                        if !frames.insert(&f.frame_id) {
                            return Err("JZ46不能同时保留同一堆栈和检视续体".into());
                        }
                        let FrameChoice::JZ46InspectHidden { inspected } = choice else {
                            return Err("JZ46不能使用其他待选续体".into());
                        };
                        if !self.jz46_inspected_valid(f, inspected)
                            || p.seat != f.actor
                            || self.players[f.actor].eliminated
                            || p.choice.id.is_empty()
                            || p.choice.player_id != format!("p{}", f.actor)
                            || p.choice.kind != CHOICE
                            || p.choice.title != TITLE
                            || p.choice.description != DESCRIPTION
                            || !p.choice.options.is_empty()
                            || p.choice.min != Some(0)
                            || p.choice.max != Some(0)
                            || p.choice.amount.is_some()
                            || p.choice.allow_decline != Some(false)
                            || !p.choice.preview_cards.is_empty()
                        {
                            return Err("JZ46私密检视的原牌、选择者或暂停元数据无效".into());
                        }
                    }
                }
                ChoiceResolution::Declare { declaration, .. }
                    if declaration.source.card.definition == "JZ46"
                        || matches!(declaration.ability.key.as_str(), PLAY | INSPECT)
                        || uses(&declaration.ability.ops)
                        || declaration.ability.modes.iter().any(|m| uses(&m.ops)) =>
                {
                    return Err("JZ46没有触发声明".into())
                }
                _ => {}
            }
        }
        Ok(())
    }
}
