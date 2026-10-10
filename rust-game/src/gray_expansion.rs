//! Complete, finite JZ44/JZ45 programs. Integration stays in the shared engine.
use crate::{
    engine::RuleResult,
    model::*,
    rules::{self, ability, target, with_abilities, *},
};

pub(crate) fn definition(id: &str) -> Option<Definition> {
    match id {
        "JZ44" => {
            let mut criminal = target(
                Zone::Board,
                EntityKind::Character,
                Relation::Any,
                Range::SourceRegion,
            );
            criminal.subtype = Some("罪犯".into());
            let mut d = with_abilities(vec![ability(
                "entry-hide-criminal-draw",
                "进场触发：潜伏罪犯，然后抓一张牌",
                Timing::Fast,
                vec![],
                vec![criminal],
                vec![
                    Op::Hide(EntityRef::Target(0)),
                    Op::Draw {
                        player: PlayerRef::Actor,
                        count: 1,
                        end: DeckEnd::Top,
                    },
                ],
                Some(Event::Enter),
            )]);
            d.traits.public = true;
            d.traits.guard = 1;
            d.traits.city_play_only = true;
            Some(d)
        }
        "JZ45" => {
            let local = target(
                Zone::Board,
                EntityKind::CharacterOrHidden,
                Relation::Any,
                Range::SourceRegion,
            );
            let mut locked = local.clone();
            locked.predicate = Some(TargetPredicate::JZ45LockedLocalTarget);
            let mut entry = ability(
                "entry-lock-or-destroy",
                "进场触发：选择锁定或消灭",
                Timing::Fast,
                vec![],
                vec![],
                vec![],
                Some(Event::Enter),
            );
            entry.modes = vec![
                Mode {
                    key: "lock".into(),
                    label: "放置一个锁定标志".into(),
                    targets: vec![local],
                    ops: vec![Op::JZ45LockLocalTarget],
                },
                Mode {
                    key: "destroy-locked".into(),
                    label: "消灭有锁定标志的目标".into(),
                    targets: vec![locked],
                    ops: vec![Op::Destroy(EntityRef::Target(0))],
                },
            ];
            // The existing JC014 mobility program: only a ready source, adjacent
            // regions in teams mode, optional ConfrontationStart declaration.
            let mut mobility = ability(
                "mobility",
                "机动",
                Timing::Fast,
                vec![],
                vec![target(
                    Zone::Region,
                    EntityKind::Any,
                    Relation::Any,
                    Range::Mobility,
                )],
                vec![Op::MoveOnBoard {
                    entity: EntityRef::Source,
                    region: RegionRef::Target(0),
                }],
                Some(Event::ConfrontationStart),
            );
            mobility.requires_ready_source = true;
            let mut d = with_abilities(vec![entry, mobility]);
            d.traits.public = true;
            d.traits.city_play_only = true;
            Some(d)
        }
        _ => None,
    }
}

pub(crate) fn contains_gray_expansion_op(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JZ45LockLocalTarget => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => {
            contains_gray_expansion_op(body)
        }
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => {
            contains_gray_expansion_op(std::slice::from_ref(exhausted))
                || contains_gray_expansion_op(std::slice::from_ref(ready))
        }
        _ => false,
    })
}
fn has_predicate(targets: &[TargetSlotSpec]) -> bool {
    targets
        .iter()
        .any(|t| t.predicate == Some(TargetPredicate::JZ45LockedLocalTarget))
}
fn own_key(key: &str) -> bool {
    matches!(key, "entry-hide-criminal-draw" | "entry-lock-or-destroy")
}
fn selected_modes(id: &str) -> Vec<AbilitySpec> {
    let Some(d) = definition(id) else {
        return vec![];
    };
    d.abilities
        .iter()
        .flat_map(|a| {
            a.modes.iter().map(|m| {
                let mut selected = a.clone();
                selected.label = format!("{}：{}", selected.label, m.label);
                selected.targets = m.targets.clone();
                selected.ops = m.ops.clone();
                selected.modes.clear();
                selected
            })
        })
        .collect()
}
pub(crate) fn ability_is_admitted(id: &str, a: &AbilitySpec) -> bool {
    definition(id).is_some_and(|d| {
        d.abilities
            .iter()
            .chain(selected_modes(id).iter())
            .any(|printed| {
                serde_json::to_value(a).unwrap() == serde_json::to_value(printed).unwrap()
            })
    })
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    let relevant = relevant_ability(id, a);
    if relevant
        && !ability_is_admitted(id, a)
        && !(definition(id).is_some() && crate::jz22::runtime_ability_is_admitted(a))
    {
        return Err(format!(
            "{id}: only complete JZ44/JZ45 printed or selected-mode programs are admitted"
        ));
    }
    Ok(())
}
fn relevant_ability(id: &str, a: &AbilitySpec) -> bool {
    definition(id).is_some()
        || own_key(&a.key)
        || contains_gray_expansion_op(&a.ops)
        || a.modes.iter().any(|m| contains_gray_expansion_op(&m.ops))
        || has_predicate(&a.targets)
        || a.modes.iter().any(|m| has_predicate(&m.targets))
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if let Some(printed) = definition(id) {
        if serde_json::to_value(d).unwrap() != serde_json::to_value(printed).unwrap() {
            return Err(format!(
                "{id}: only the complete printed gray expansion definition is admitted"
            ));
        }
    }
    Ok(())
}
fn relevant_frame(f: &ResolutionFrame) -> bool {
    definition(&f.source.card.definition).is_some()
        || own_key(&f.ability_key)
        || f.steps
            .iter()
            .any(|s| contains_gray_expansion_op(std::slice::from_ref(&s.op)))
        || f.targets
            .iter()
            .any(|t| has_predicate(std::slice::from_ref(&t.spec)))
}
fn validate_source(
    g: &Game,
    actor: usize,
    s: &SourceSnapshot,
    allow_hidden: bool,
    require_region_instance: bool,
) -> RuleResult<()> {
    if definition(&s.card.definition).is_none()
        || actor >= g.players.len()
        || s.card.owner >= g.players.len()
        || s.card.controller != actor
        || s.card.id.is_empty()
        || (!allow_hidden && s.card.face_down)
        || s.region.is_none_or(|r| r >= g.regions.len())
        || (require_region_instance
            && s.source_region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty()))
        || s.source_region_instance
            .as_ref()
            .is_some_and(|id| id.is_empty())
        || s.observed_death.is_some()
        || s.attachment_host_instance.is_some()
    {
        return Err("灰色扩充来源、冻结行动者或原地区实例不符".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn jz45_lock_local_target(&mut self, frame: &ResolutionFrame) -> RuleResult<()> {
        let target = frame.targets.first().ok_or("JZ45缺少锁定目标")?;
        if let Some(c) = self.board_mut(&target.id) {
            c.lock_markers = c.lock_markers.saturating_add(1);
        }
        Ok(())
    }
    pub(crate) fn validate_gray_expansion_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !relevant_frame(f) {
            return Ok(());
        }
        validate_source(
            self,
            f.actor,
            &f.source,
            f.ability_key == "reveal",
            f.ability_key != "mobility",
        )?;
        if f.frame_id.is_empty() || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) {
            return Err("灰色扩充原子程序不能恢复已执行游标或已通过守卫".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            let paid_valid = matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)]
                if ids.iter().all(|id| !id.is_empty())
                    && ids.iter().collect::<std::collections::BTreeSet<_>>().len() == ids.len());
            if !f.steps.is_empty()
                || !f.targets.is_empty()
                || !paid_valid
                || f.chosen_region != f.source.region
                || (f.ability_key == "deploy"
                    && (f.source.card.face_down || f.source.play_source != Some(PlaySource::Hand)))
                || (f.ability_key == "reveal"
                    && (!f.source.card.face_down || f.source.play_source.is_some()))
            {
                return Err("灰色扩充仅准入真实手牌派遣或付费现身空程序".into());
            }
            return Ok(());
        }
        if f.source.play_source.is_some() || !f.already_paid.is_empty() || f.chosen_region.is_some()
        {
            return Err("灰色扩充触发程序不得夹带支付、打出来源或选择地区".into());
        }
        let mut candidates = definition(&f.source.card.definition).unwrap().abilities;
        candidates.extend(selected_modes(&f.source.card.definition));
        if let Some(instance) = f.source.source_region_instance.as_deref() {
            candidates.push(crate::renown::renown_ability(instance));
            candidates.push(rules::jc089_combat_glory_ability(instance));
        }
        let same_program = candidates
            .iter()
            .filter(|a| a.key == f.ability_key && a.modes.is_empty())
            .any(|a| {
                a.ops.len() == f.steps.len()
                    && a.targets.len() == f.targets.len()
                    && a.ops.iter().zip(&f.steps).all(|(op, s)| {
                        s.context == f.actor
                            && serde_json::to_value(op).unwrap()
                                == serde_json::to_value(&s.op).unwrap()
                    })
                    && a.targets.iter().zip(&f.targets).all(|(spec, t)| {
                        !t.id.is_empty()
                            && t.public.instance_id == t.id
                            && t.region_instance.is_none()
                            && serde_json::to_value(spec).unwrap()
                                == serde_json::to_value(&t.spec).unwrap()
                    })
            });
        if !same_program {
            return Err("灰色扩充完整程序、目标或冻结行动者不符".into());
        }
        Ok(())
    }
    pub(crate) fn validate_gray_expansion_state(&self) -> RuleResult<()> {
        let frame = |f: &ResolutionFrame, pending: bool| -> RuleResult<()> {
            self.validate_gray_expansion_frame(f)?;
            if relevant_frame(f) && pending {
                return Err("灰色扩充原子程序不能保存帧内选择".into());
            }
            Ok(())
        };
        let declaration = |d: &Declaration| -> RuleResult<()> {
            let relevant = relevant_ability(&d.source.card.definition, &d.ability);
            if relevant {
                validate_ability(&d.source.card.definition, &d.ability)?;
                validate_source(self, d.actor, &d.source, false, d.ability.key != "mobility")?;
                if d.source.play_source.is_some() {
                    return Err("灰色扩充触发声明不能夹带手牌或墓地打出标记".into());
                }
                if let [Op::PlaceInfluence {
                    region_instance, ..
                }] = d.ability.ops.as_slice()
                {
                    if d.source.source_region_instance.as_ref() != Some(region_instance) {
                        return Err("灰色扩充获授奖励必须绑定冻结原地区".into());
                    }
                }
            }
            Ok(())
        };
        for s in &self.stack {
            if s.frame.is_none()
                && s.card
                    .as_ref()
                    .is_some_and(|c| definition(&c.definition).is_some())
            {
                return Err("灰色扩充派遣或现身堆栈必须携带完整冻结帧".into());
            }
            if let Some(f) = &s.frame {
                frame(f, false)?;
                if relevant_frame(f) {
                    if s.controller != f.actor
                        || s.id != f.frame_id
                        || s.target.as_ref() != f.targets.first().map(|t| &t.id)
                    {
                        return Err("灰色扩充堆栈对象与冻结帧不符".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if s.card.as_ref().is_none_or(|c| {
                            c.definition != f.source.card.definition
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                                || c.face_down
                                || c.id.is_empty()
                                || c.id == f.source.card.id
                        }) || s.deploy_region != f.source.region
                            || s.reveal != (f.ability_key == "reveal")
                        {
                            return Err("灰色扩充打出实体、地区或现身标记不符".into());
                        }
                    } else if s.card.is_some() || s.deploy_region.is_some() || s.reveal {
                        return Err("灰色扩充触发堆栈不得携带实体、派遣地区或现身标记".into());
                    }
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
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } => {
                    declaration(d)?;
                    if relevant_ability(&d.source.card.definition, &d.ability) {
                        let a = &d.ability;
                        let expected = match stage {
                            DeclareChoice::Mode if !a.modes.is_empty() => a
                                .modes
                                .iter()
                                .map(|m| ChoiceOption {
                                    id: m.key.clone(),
                                    label: m.label.clone(),
                                    card: None,
                                })
                                .collect(),
                            DeclareChoice::Target if a.modes.is_empty() && a.targets.len() == 1 => {
                                self.binding_options(d.actor, &d.source, &a.targets[0])
                            }
                            DeclareChoice::Accept if a.modes.is_empty() && a.targets.is_empty() => {
                                vec![ChoiceOption {
                                    id: "accept".into(),
                                    label: "发动触发能力".into(),
                                    card: None,
                                }]
                            }
                            _ => return Err("灰色扩充待选阶段与完整能力形状不符".into()),
                        };
                        let title = format!(
                            "{}：{}",
                            crate::catalog::card(&d.source.card.definition).name,
                            a.label
                        );
                        if expected.is_empty()
                            || p.seat != d.actor
                            || p.choice.player_id != player_id(d.actor)
                            || p.choice.id.is_empty()
                            || p.choice.kind != "trigger"
                            || p.choice.title != title
                            || p.choice.description != "选择符合数量限制的选项，然后确认。"
                            || p.choice.min != Some(0)
                            || p.choice.max != Some(1)
                            || p.choice.allow_decline != Some(true)
                            || p.choice.amount.is_some()
                            || !p.choice.preview_cards.is_empty()
                            || serde_json::to_value(&p.choice.options).unwrap()
                                != serde_json::to_value(expected).unwrap()
                        {
                            return Err("灰色扩充待选玩家、阶段、元数据或完整选项不符".into());
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
