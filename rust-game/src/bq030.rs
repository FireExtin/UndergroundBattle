//! Closed BQ030 attachment observation and optional simultaneous declarations.
//!
//! The caller records a successful attachment after continuous control changes
//! and before any departure cleanup. The immutable event survives removal of
//! its attachment, host, or observers; only declaration order is mutable.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const ABILITY_KEY: &str = "attachment-observer-draw-one";
const ORDER_KIND: &str = "bq030-attachment-order";
const ORDER_TITLE: &str = "裁定官：选择下一项结附触发入栈";
const ORDER_DESCRIPTION: &str =
    "选择下一位裁定官的触发能力入栈；后入栈的先结算。跳过会放弃本次结附剩余的触发能力。";
const ACCEPT_DESCRIPTION: &str = "选择符合数量限制的选项，然后确认。";

/// A finite historical attachment event, carried by the admitted atomic Op.
/// `observers` never changes. `remaining` belongs to a declaration carrier, or
/// contains exactly the selected observer in an already declared draw frame.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachmentEvent {
    pub event_id: String,
    pub attachment: Card,
    pub host: SourceSnapshot,
    pub observers: Vec<SourceSnapshot>,
    pub remaining: Vec<String>,
}

fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}

/// Construct directly: registry initialization must not re-enter its OnceLock.
pub(crate) fn definition() -> Definition {
    rules::with_abilities(vec![rules::ability(
        ABILITY_KEY,
        "结附触发：抓一张牌",
        Timing::Fast,
        vec![],
        vec![],
        vec![Op::BQ030AttachmentDraw { observation: None }],
        Some(Event::AttachmentCommittedObserved),
    )])
}

pub(crate) fn contains_op(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::BQ030AttachmentDraw { .. } => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => contains_op(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => {
            contains_op(std::slice::from_ref(exhausted)) || contains_op(std::slice::from_ref(ready))
        }
        _ => false,
    })
}

pub(crate) fn is_attachment_declaration(d: &Declaration) -> bool {
    d.ability.key == ABILITY_KEY
        || d.ability.event == Some(Event::AttachmentCommittedObserved)
        || contains_op(&d.ability.ops)
        || d.ability.modes.iter().any(|m| contains_op(&m.ops))
}

fn relevant_declaration(d: &Declaration) -> bool {
    d.source.card.definition == "BQ030" || is_attachment_declaration(d)
}

fn relevant_frame(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "BQ030"
        || f.ability_key == ABILITY_KEY
        || f.steps
            .iter()
            .any(|s| contains_op(std::slice::from_ref(&s.op)))
}

/// Permit only the exact static program or its finite historical payload.
/// The Game validators below require Some for every runtime declaration/frame.
pub(crate) fn validate_ability(card_id: &str, a: &AbilitySpec) -> RuleResult<()> {
    let relevant = card_id == "BQ030"
        || a.key == ABILITY_KEY
        || a.event == Some(Event::AttachmentCommittedObserved)
        || contains_op(&a.ops)
        || a.modes.iter().any(|m| contains_op(&m.ops));
    if !relevant {
        return Ok(());
    }
    if card_id == "BQ030" && crate::jz22::runtime_ability_is_admitted(a) {
        return Ok(());
    }
    let mut normalized = a.clone();
    if let [Op::BQ030AttachmentDraw { observation }] = normalized.ops.as_mut_slice() {
        *observation = None;
    }
    if card_id != "BQ030" || !same(&normalized, &definition().abilities[0]) {
        return Err("裁定官仅准入完整印刷结附触发和既有获授奖励程序".into());
    }
    Ok(())
}

fn event_in_ability(a: &AbilitySpec) -> RuleResult<&AttachmentEvent> {
    let [Op::BQ030AttachmentDraw {
        observation: Some(event),
    }] = a.ops.as_slice()
    else {
        return Err("裁定官运行触发缺少完整结附事件".into());
    };
    Ok(event)
}

fn event_in_frame(f: &ResolutionFrame) -> RuleResult<&AttachmentEvent> {
    let [Step {
        op: Op::BQ030AttachmentDraw {
            observation: Some(event),
        },
        ..
    }] = f.steps.as_slice()
    else {
        return Err("裁定官运行帧缺少完整原子结附事件".into());
    };
    Ok(event)
}

fn runtime_ability(event: AttachmentEvent) -> AbilitySpec {
    let mut a = definition().abilities.remove(0);
    a.ops = vec![Op::BQ030AttachmentDraw {
        observation: Some(Box::new(event)),
    }];
    a
}

fn plain_snapshot(card: &Card, region: usize) -> SourceSnapshot {
    SourceSnapshot {
        card: card.clone(),
        region: Some(region),
        observed_death: None,
        source_region_instance: None,
        attachment_host_instance: None,
        play_source: None,
    }
}

impl Game {
    /// Successful actual attachment only. No targeting attempt, entry of the
    /// observer itself, host movement, or unchanged reattachment calls this.
    pub(crate) fn observe_attachment_committed(&mut self, attachment_id: &str) {
        let Some(attachment) = self.attachments.iter().find(|a| a.card.id == attachment_id) else {
            return;
        };
        if attachment.card.face_down || !self.attachment_host_valid(attachment) {
            return;
        }
        let Some((region, host)) = self.board(&attachment.host_id) else {
            return;
        };
        if host.face_down || catalog::card(&host.definition).kind != "character" {
            return;
        }
        let actor = host.controller;
        if self.players.get(actor).is_none_or(|p| p.eliminated) {
            return;
        }
        let observers: Vec<_> = self
            .regions
            .iter()
            .enumerate()
            .flat_map(|(r, region)| {
                region
                    .cards
                    .iter()
                    .filter(move |c| {
                        c.definition == "BQ030" && !c.face_down && c.controller == actor
                    })
                    .map(move |c| plain_snapshot(c, r))
            })
            .collect();
        let Some(source) = observers.first().cloned() else {
            return;
        };
        let attachment = attachment.card.clone();
        let host = plain_snapshot(host, region);
        let remaining = observers.iter().map(|s| s.card.id.clone()).collect();
        let event = AttachmentEvent {
            event_id: self.id(),
            attachment,
            host,
            observers,
            remaining,
        };
        self.effects.push_back(Effect::Declare {
            declaration: Declaration {
                actor,
                source,
                ability: runtime_ability(event),
            },
        });
    }

    fn bq030_instance_number(&self, id: &str) -> Option<u64> {
        id.strip_prefix('i')
            .and_then(|s| s.parse::<u64>().ok())
            .filter(|n| *n > 0 && *n <= self.sequence && id == format!("i{n}"))
    }

    /// Actual card containers only; frozen sources are historical copies.
    fn bq030_physical_cards(&self) -> impl Iterator<Item = &Card> {
        self.players
            .iter()
            .flat_map(|p| {
                p.hand
                    .iter()
                    .chain(&p.deck)
                    .chain(&p.assets)
                    .chain(&p.graveyard)
                    .chain(&p.score_cards)
                    .chain(p.society_zone.card.iter())
            })
            .chain(self.regions.iter().flat_map(|r| r.cards.iter()))
            .chain(self.regions.iter().map(|r| &r.card))
            .chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.sealed_cards.iter().map(|s| &s.card))
            .chain(self.world.iter())
            .chain(self.stack.iter().filter_map(|s| s.card.as_ref()))
            .chain(self.effects.iter().filter_map(|e| match e {
                Effect::Bury { card } => Some(card),
                _ => None,
            }))
    }

    fn bq030_historical_card(&self, c: &Card, kind: &str) -> RuleResult<()> {
        if self.bq030_instance_number(&c.id).is_none()
            || c.owner >= self.players.len()
            || c.controller >= self.players.len()
            || c.face_down
            || !catalog::catalog()
                .cards
                .iter()
                .any(|d| d.id == c.definition && d.supported && d.kind == kind)
        {
            return Err("结附事件包含未知牌面、暗置对象或非法历史实例".into());
        }
        let mut live = self.bq030_physical_cards().filter(|live| live.id == c.id);
        if let Some(current) = live.next() {
            // Departure allocates a fresh identity. A surviving identity must
            // retain its printed definition, owner and original container kind;
            // its controller, position and other mutable state may have changed.
            if live.next().is_some()
                || current.definition != c.definition
                || current.owner != c.owner
                || (kind == "character"
                    && !self
                        .regions
                        .iter()
                        .any(|r| r.cards.iter().any(|live| live.id == c.id)))
                || (kind == "attachment" && !self.attachments.iter().any(|a| a.card.id == c.id))
            {
                return Err("结附历史实例与现存卡牌身份或容器不符".into());
            }
        }
        Ok(())
    }

    fn bq030_plain_character(&self, s: &SourceSnapshot) -> RuleResult<()> {
        self.bq030_historical_card(&s.card, "character")?;
        if s.region.is_none_or(|r| r >= self.regions.len())
            || s.observed_death.is_some()
            || s.source_region_instance.is_some()
            || s.attachment_host_instance.is_some()
            || s.play_source.is_some()
        {
            return Err("结附事件角色快照含其他程序的绑定".into());
        }
        Ok(())
    }

    fn bq030_validate_event(&self, event: &AttachmentEvent) -> RuleResult<()> {
        let event_number = self
            .bq030_instance_number(&event.event_id)
            .ok_or("结附事件标识无效")?;
        self.bq030_historical_card(&event.attachment, "attachment")?;
        self.bq030_plain_character(&event.host)?;
        if !rules::definition(&event.attachment.definition)
            .attachment
            .as_ref()
            .is_some_and(|a| a.host.zone == Zone::Board && a.host.kind == EntityKind::Character)
            || event.observers.is_empty()
            || event.remaining.is_empty()
        {
            return Err("裁定官事件必须为角色上的结附且具有剩余合格来源".into());
        }
        let actor = event.host.card.controller;
        let mut participants = BTreeSet::new();
        let mut frozen_ids = vec![&event.attachment.id, &event.host.card.id];
        if event.attachment.id == event.host.card.id {
            return Err("结附附属与宿主不能共用实例".into());
        }
        for observer in &event.observers {
            self.bq030_plain_character(observer)?;
            if observer.card.definition != "BQ030"
                || observer.card.controller != actor
                || !participants.insert(observer.card.id.as_str())
                || observer.card.id == event.attachment.id
                || (observer.card.id == event.host.card.id && !same(observer, &event.host))
            {
                return Err("裁定官事件来源必须为同操控方的不同明置裁定官".into());
            }
            frozen_ids.push(&observer.card.id);
        }
        if event.host.card.definition == "BQ030"
            && !participants.contains(event.host.card.id.as_str())
        {
            return Err("明置裁定官宿主必须保留在完整结附观察者中".into());
        }
        let mut remaining = BTreeSet::new();
        if event
            .remaining
            .iter()
            .any(|id| !participants.contains(id.as_str()) || !remaining.insert(id.as_str()))
            || frozen_ids.iter().any(|id| {
                self.bq030_instance_number(id)
                    .is_none_or(|n| n >= event_number)
            })
        {
            return Err("裁定官剩余来源或事件与牌面身份无效".into());
        }
        let historical_sources = self
            .stack
            .iter()
            .filter_map(|s| s.frame.as_ref().map(|f| &f.source))
            .chain(self.effects.iter().filter_map(|e| match e {
                Effect::Declare { declaration } => Some(&declaration.source),
                Effect::Frame { frame } => Some(&frame.source),
                _ => None,
            }))
            .chain(self.pending.as_ref().and_then(|p| match &p.resolution {
                ChoiceResolution::Declare { declaration, .. } => Some(&declaration.source),
                ChoiceResolution::Frame { frame, .. } => Some(&frame.source),
                _ => None,
            }));
        if self.bq030_physical_cards().any(|c| c.id == event.event_id)
            || historical_sources.into_iter().any(|s| {
                s.card.id == event.event_id
                    || s.observed_death
                        .as_ref()
                        .is_some_and(|d| d.card.id == event.event_id)
                    || s.source_region_instance.as_ref() == Some(&event.event_id)
                    || s.attachment_host_instance.as_ref() == Some(&event.event_id)
            })
            || self.stack.iter().any(|s| s.id == event.event_id)
            || self
                .effects
                .iter()
                .any(|e| matches!(e, Effect::Frame { frame } if frame.frame_id == event.event_id))
            || self.pending.as_ref().is_some_and(|p| {
                p.choice.id == event.event_id
                    || matches!(&p.resolution, ChoiceResolution::Frame { frame, .. }
                            if frame.frame_id == event.event_id)
            })
        {
            return Err("结附事件不能盗用当前卡牌、堆栈或选择的标识".into());
        }
        Ok(())
    }

    fn bq030_reward_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let mut plain = s.clone();
        plain.source_region_instance = None;
        self.bq030_plain_character(&plain)?;
        if s.card.definition != "BQ030"
            || s.card.controller != actor
            || s.source_region_instance
                .as_ref()
                .is_none_or(|id| self.bq030_instance_number(id).is_none() || id == &s.card.id)
        {
            return Err("裁定官获授奖励缺少冻结原地区或操控者".into());
        }
        Ok(())
    }

    pub(crate) fn validate_bq030_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if !relevant_declaration(d) {
            return Ok(());
        }
        validate_ability(&d.source.card.definition, &d.ability)?;
        if d.actor >= self.players.len() || d.source.card.controller != d.actor {
            return Err("裁定官声明操控者无效".into());
        }
        if crate::jz22::runtime_ability_is_admitted(&d.ability) {
            self.bq030_reward_source(d.actor, &d.source)?;
            let [Op::PlaceInfluence {
                region_instance, ..
            }] = d.ability.ops.as_slice()
            else {
                unreachable!();
            };
            if d.source.source_region_instance.as_ref() != Some(region_instance) {
                return Err("裁定官获授奖励只能绑定冻结原地区".into());
            }
            return Ok(());
        }
        let event = event_in_ability(&d.ability)?;
        self.bq030_validate_event(event)?;
        let carrier = event
            .observers
            .iter()
            .find(|s| Some(&s.card.id) == event.remaining.first())
            .ok_or("裁定官声明缺少剩余来源")?;
        if d.actor != event.host.card.controller || !same(&d.source, carrier) {
            return Err("裁定官声明来源与冻结结附事件不符".into());
        }
        Ok(())
    }

    fn bq030_order_options(&self, event: &AttachmentEvent, actor: usize) -> Vec<ChoiceOption> {
        event
            .remaining
            .iter()
            .map(|id| {
                let source = event.observers.iter().find(|s| s.card.id == *id).unwrap();
                self.option(&source.card, actor, source.region, None)
            })
            .collect()
    }

    /// Return true only when this module owns the finite attachment declaration.
    pub(crate) fn bq030_declare(&mut self, declaration: Declaration) -> RuleResult<bool> {
        if !is_attachment_declaration(&declaration) {
            return Ok(false);
        }
        self.validate_bq030_declaration(&declaration)?;
        let actor = declaration.actor;
        if self.players[actor].eliminated {
            return Ok(true);
        }
        let event = event_in_ability(&declaration.ability)?;
        let multiple = event.remaining.len() > 1;
        let (options, stage, kind, title, description) = if multiple {
            (
                self.bq030_order_options(event, actor),
                DeclareChoice::AttachmentOrder,
                ORDER_KIND,
                ORDER_TITLE.to_owned(),
                ORDER_DESCRIPTION,
            )
        } else {
            (
                vec![ChoiceOption {
                    id: "accept".into(),
                    label: "发动触发能力".into(),
                    card: None,
                }],
                DeclareChoice::Accept,
                "trigger",
                format!("裁定官：{}", declaration.ability.label),
                ACCEPT_DESCRIPTION,
            )
        };
        self.choice(
            actor,
            kind,
            title,
            options,
            0,
            1,
            None,
            ChoiceResolution::Declare { declaration, stage },
        );
        self.pending.as_mut().unwrap().choice.description = description.into();
        Ok(true)
    }

    pub(crate) fn bq030_choose(
        &mut self,
        declaration: Declaration,
        stage: DeclareChoice,
        selected: Vec<String>,
    ) -> RuleResult<()> {
        self.validate_bq030_declaration(&declaration)?;
        let mut event = event_in_ability(&declaration.ability)?.clone();
        if selected.len() > 1
            || self.players[declaration.actor].eliminated
            || !(if event.remaining.len() == 1 {
                matches!(stage, DeclareChoice::Accept)
            } else {
                matches!(stage, DeclareChoice::AttachmentOrder)
            })
        {
            return Err("裁定官结附触发排序阶段或选择数量无效".into());
        }
        let Some(selected) = selected.first() else {
            return Ok(());
        };
        let selected_id = if event.remaining.len() == 1 {
            if selected != "accept" {
                return Err("裁定官单个触发只能选择发动或跳过".into());
            }
            event.remaining[0].clone()
        } else {
            if !event.remaining.contains(selected) {
                return Err("裁定官只能选择本次结附尚未入栈的来源".into());
            }
            selected.clone()
        };
        let source = event
            .observers
            .iter()
            .find(|s| s.card.id == selected_id)
            .unwrap()
            .clone();
        let mut frame_event = event.clone();
        frame_event.remaining = vec![selected_id.clone()];
        let ability = runtime_ability(frame_event);
        let frame = self.make_frame(declaration.actor, source, &ability, vec![], vec![], None);
        self.dispatch_frame(
            frame,
            format!("裁定官：{}", ability.label),
            None,
            ResponsePolicy::Respondable,
        );
        event.remaining.retain(|id| *id != selected_id);
        if let Some(next) = event.remaining.first() {
            let source = event
                .observers
                .iter()
                .find(|s| &s.card.id == next)
                .unwrap()
                .clone();
            // One simultaneous event is fully declared before its first response.
            self.effects.push_front(Effect::Declare {
                declaration: Declaration {
                    actor: declaration.actor,
                    source,
                    ability: runtime_ability(event),
                },
            });
        }
        Ok(())
    }

    pub(crate) fn validate_bq030_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !relevant_frame(f) {
            return Ok(());
        }
        if f.source.card.definition != "BQ030"
            || self.bq030_instance_number(&f.frame_id).is_none()
            || self.bq030_instance_number(&f.frame_id)
                <= self.bq030_instance_number(&f.source.card.id)
            || f.actor >= self.players.len()
            || f.source.card.controller != f.actor
        {
            return Err("裁定官程序不能移植到其他来源或操控者".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            let s = &f.source;
            let mut plain = s.clone();
            plain.card.face_down = false;
            plain.play_source = None;
            plain.source_region_instance = None;
            self.bq030_plain_character(&plain)?;
            let [PaidCost::Assets(ids)] = f.already_paid.as_slice() else {
                return Err("裁定官派遣或现身必须保持实际资产支付记录".into());
            };
            let mut paid_ids = BTreeSet::new();
            if !f.steps.is_empty()
                || self
                    .bq030_physical_cards()
                    .any(|current| current.id == s.card.id)
                || s.card.time_markers != 0
                || !f.targets.is_empty()
                || f.chosen_region != s.region
                || s.source_region_instance
                    .as_ref()
                    .is_none_or(|id| self.bq030_instance_number(id).is_none() || id == &s.card.id)
                || f.cursor != 0
                || !matches!(f.guard, GuardState::Unchecked)
                || ids.len() > 2
                || ids.iter().any(|id| {
                    self.bq030_instance_number(id).is_none()
                        || id == &s.card.id
                        || !paid_ids.insert(id.as_str())
                })
                || (if f.ability_key == "deploy" {
                    s.card.face_down || s.play_source != Some(PlaySource::Hand)
                } else {
                    !s.card.face_down || s.play_source.is_some()
                })
            {
                return Err("裁定官派遣或现身来源、空程序或费用无效".into());
            }
            return Ok(());
        }
        if matches!(f.ability_key.as_str(), "renown" | "jc089-combat-glory") {
            self.bq030_reward_source(f.actor, &f.source)?;
            let region = f.source.source_region_instance.as_deref().unwrap();
            let expected = if f.ability_key == "renown" {
                crate::renown::renown_ability(region)
            } else {
                rules::jc089_combat_glory_ability(region)
            };
            if f.steps.len() != 1
                || f.steps[0].context != f.actor
                || !same(&f.steps[0].op, &expected.ops[0])
                || !f.targets.is_empty()
                || !f.already_paid.is_empty()
                || f.chosen_region.is_some()
                || !((f.cursor == 0 && matches!(f.guard, GuardState::Unchecked))
                    || (f.cursor == 1 && matches!(f.guard, GuardState::Accepted)))
            {
                return Err("裁定官获授奖励必须保持既有完整原子程序".into());
            }
            return Ok(());
        }
        let event = event_in_frame(f)?;
        self.bq030_validate_event(event)?;
        let selected = event
            .observers
            .iter()
            .find(|s| s.card.id == f.source.card.id);
        if f.ability_key != ABILITY_KEY
            || self.bq030_instance_number(&f.frame_id)
                <= self.bq030_instance_number(&event.event_id)
            || f.steps[0].context != f.actor
            || f.actor != event.host.card.controller
            || selected.is_none_or(|s| !same(s, &f.source))
            || event.remaining != vec![f.source.card.id.clone()]
            || !f.targets.is_empty()
            || !f.already_paid.is_empty()
            || f.chosen_region.is_some()
            || !((f.cursor == 0 && matches!(f.guard, GuardState::Unchecked))
                || (f.cursor == 1 && matches!(f.guard, GuardState::Accepted)))
        {
            return Err("裁定官完整抓牌帧、冻结来源、费用或游标被修改".into());
        }
        Ok(())
    }

    pub(crate) fn bq030_draw_one(&mut self, frame: &ResolutionFrame) -> RuleResult<()> {
        self.validate_bq030_frame(frame)?;
        if frame.ability_key != ABILITY_KEY
            || frame.cursor != 1
            || !matches!(frame.guard, GuardState::Accepted)
        {
            return Err("裁定官抓牌只能执行已通过守卫的完整原子触发".into());
        }
        self.draw(frame.actor, 1)
    }

    pub(crate) fn validate_bq030_state(&self) -> RuleResult<()> {
        let mut events: BTreeMap<String, AttachmentEvent> = BTreeMap::new();
        let mut selected: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut carriers: BTreeMap<String, AttachmentEvent> = BTreeMap::new();
        let mut register = |event: &AttachmentEvent, carrier: bool| -> RuleResult<()> {
            // All copies of an event keep the same immutable attachment, host,
            // and complete participant list. Only the remaining subset changes.
            if let Some(original) = events.get(event.event_id.as_str()) {
                if !same(&original.attachment, &event.attachment)
                    || !same(&original.host, &event.host)
                    || !same(&original.observers, &event.observers)
                {
                    return Err("同一结附事件的历史快照不一致".into());
                }
            } else {
                events.insert(event.event_id.clone(), event.clone());
            }
            if carrier {
                if carriers
                    .insert(event.event_id.clone(), event.clone())
                    .is_some()
                {
                    return Err("同一结附事件不能有重复的剩余声明".into());
                }
            } else {
                let set = selected.entry(event.event_id.clone()).or_default();
                if !set.insert(event.remaining[0].clone()) {
                    return Err("同一裁定官不能重复声明同一次结附触发".into());
                }
            }
            Ok(())
        };
        for item in &self.stack {
            if item.card.as_ref().is_some_and(|c| c.definition == "BQ030")
                && item
                    .frame
                    .as_ref()
                    .is_none_or(|f| f.source.card.definition != "BQ030")
            {
                return Err("裁定官交易必须保留自身来源的完整冻结程序".into());
            }
            if let Some(f) = &item.frame {
                self.validate_bq030_frame(f)?;
                if relevant_frame(f) {
                    let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
                    let expected_label = if play {
                        format!(
                            "{} 裁定官",
                            if f.ability_key == "deploy" {
                                "派遣"
                            } else {
                                "现身"
                            }
                        )
                    } else if f.ability_key == ABILITY_KEY {
                        format!("裁定官：{}", definition().abilities[0].label)
                    } else {
                        let r = f.source.source_region_instance.as_deref().unwrap();
                        let a = if f.ability_key == "renown" {
                            crate::renown::renown_ability(r)
                        } else {
                            rules::jc089_combat_glory_ability(r)
                        };
                        format!("裁定官：{}", a.label)
                    };
                    if item.id != f.frame_id
                        || item.controller != f.actor
                        || item.label != expected_label
                        || item.target.is_some()
                        || f.cursor != 0
                        || !matches!(f.guard, GuardState::Unchecked)
                        || (if play {
                            item.card.as_ref().is_none_or(|c| {
                                let mut expected = f.source.card.clone();
                                expected.id = c.id.clone();
                                expected.face_down = false;
                                expected.time_markers = 0;
                                expected.lock_markers = 0;
                                self.bq030_instance_number(&c.id).is_none_or(|n| {
                                    n <= self.bq030_instance_number(&f.source.card.id).unwrap()
                                        || n >= self.bq030_instance_number(&f.frame_id).unwrap()
                                }) || self
                                    .bq030_physical_cards()
                                    .filter(|current| current.id == c.id)
                                    .count()
                                    != 1
                                    || !same(c, &expected)
                            }) || item.deploy_region != f.source.region
                                || item.reveal != (f.ability_key == "reveal")
                        } else {
                            item.card.is_some() || item.deploy_region.is_some() || item.reveal
                        })
                    {
                        return Err("裁定官堆栈容器与冻结程序不一致".into());
                    }
                    if f.ability_key == ABILITY_KEY {
                        register(event_in_frame(f)?, false)?;
                    }
                }
            }
        }
        for effect in &self.effects {
            match effect {
                Effect::Declare { declaration: d } => {
                    self.validate_bq030_declaration(d)?;
                    if is_attachment_declaration(d) {
                        register(event_in_ability(&d.ability)?, true)?;
                    }
                }
                Effect::Frame { frame: f } => {
                    self.validate_bq030_frame(f)?;
                    if relevant_frame(f) {
                        // Deploy, reveal, draw and granted rewards are atomic.
                        // resolve_stack uses this container transiently, then
                        // the same command executes the complete program.
                        return Err("裁定官可响应原子程序不能保存为绕过堆栈的效果帧".into());
                    }
                }
                Effect::Bury { card } if card.definition == "BQ030" => {
                    return Err("裁定官角色不能保存为事务埋葬容器".into());
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } => {
                    self.validate_bq030_declaration(d)?;
                    if matches!(stage, DeclareChoice::AttachmentOrder)
                        && !is_attachment_declaration(d)
                    {
                        return Err("结附触发排序不能用于其他能力".into());
                    }
                    if relevant_declaration(d) {
                        let event = if is_attachment_declaration(d) {
                            let event = event_in_ability(&d.ability)?;
                            register(event, true)?;
                            Some(event)
                        } else {
                            None
                        };
                        let multiple = event.is_some_and(|e| e.remaining.len() > 1);
                        let options = if multiple {
                            self.bq030_order_options(event.unwrap(), d.actor)
                        } else {
                            vec![ChoiceOption {
                                id: "accept".into(),
                                label: "发动触发能力".into(),
                                card: None,
                            }]
                        };
                        let title = if multiple {
                            ORDER_TITLE.to_owned()
                        } else {
                            format!("裁定官：{}", d.ability.label)
                        };
                        if !(if multiple {
                            matches!(stage, DeclareChoice::AttachmentOrder)
                        } else {
                            matches!(stage, DeclareChoice::Accept)
                        }) || p.seat != d.actor
                            || self.players[d.actor].eliminated
                            || self.bq030_instance_number(&p.choice.id).is_none()
                            || p.choice.player_id != format!("p{}", d.actor)
                            || p.choice.kind != (if multiple { ORDER_KIND } else { "trigger" })
                            || p.choice.title != title
                            || p.choice.description
                                != (if multiple {
                                    ORDER_DESCRIPTION
                                } else {
                                    ACCEPT_DESCRIPTION
                                })
                            || p.choice.min != Some(0)
                            || p.choice.max != Some(1)
                            || p.choice.allow_decline != Some(true)
                            || p.choice.amount.is_some()
                            || !p.choice.preview_cards.is_empty()
                            || !same(&p.choice.options, &options)
                        {
                            return Err("裁定官待选来源、排序、选项或选择者被修改".into());
                        }
                    }
                }
                ChoiceResolution::Frame { frame: f, .. } if relevant_frame(f) => {
                    return Err("裁定官完整原子程序不能移入暂停选择容器".into());
                }
                _ => {}
            }
        }
        drop(register);
        for event in events.values() {
            if events.contains_key(&event.attachment.id)
                || events.contains_key(&event.host.card.id)
                || event
                    .observers
                    .iter()
                    .any(|s| events.contains_key(&s.card.id))
            {
                return Err("结附事件标识不能盗用另一事件记录的历史卡牌身份".into());
            }
        }
        for (id, event) in carriers {
            if selected
                .get(&id)
                .is_some_and(|set| event.remaining.iter().any(|id| set.contains(id.as_str())))
            {
                return Err("已入栈裁定官不能再次出现在同一事件剩余来源中".into());
            }
            let mut accounted: BTreeSet<_> = event.remaining.iter().cloned().collect();
            if let Some(set) = selected.get(&id) {
                accounted.extend(set.iter().cloned());
            }
            if accounted != event.observers.iter().map(|s| s.card.id.clone()).collect() {
                return Err("尚未完成声明的结附事件必须保留每位合格裁定官".into());
            }
        }
        Ok(())
    }
}
