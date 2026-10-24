//! Scarab Industry: exact subtitle costs, actual entry/sealing observations,
//! and one-step monetary credits. No synthetic asset or second payment engine.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const CARDS: &[&str] = &["BQ092", "BQ093"];
pub(crate) const FUND: &str = "import20-bq092-step-resource";
pub(crate) const HAND: &str = "import20-bq092-hand-contest";
pub(crate) const MILL: &str = "import20-bq093-entry-mill";
pub(crate) const SEALED: &str = "import20-bq093-sealed-investigation";
const SUBTITLE: &str = "圣甲虫产业";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndustryEvent {
    pub event_id: String,
    pub turn: u32,
    pub subject: Card,
    pub host: Option<SourceSnapshot>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndustryBonus {
    pub target_instance: String,
    pub turn: u32,
    pub source: Box<ResolutionFrame>,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn industry(c: &Card) -> bool {
    !c.face_down
        && catalog::card(&c.definition).subtitle.as_deref() == Some(SUBTITLE)
        && catalog::has_kind(catalog::card(&c.definition), "attachment")
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    if !CARDS.contains(&id) {
        return None;
    }
    let mut d = attachment_definition(target(
        Zone::Region,
        EntityKind::Any,
        Relation::Any,
        Range::Anywhere,
    ));
    if id == "BQ092" {
        let mut a = ability(
            FUND,
            "快速行动：横置本牌及另一个本方产业附属，获得本步骤费用1",
            Timing::Fast,
            vec![Cost::ExhaustSource, Cost::IndustryExhaustOther],
            vec![],
            vec![Op::IndustryStepResource],
            None,
        );
        a.activation_only = true;
        d.abilities.push(a);
        d.abilities
            .push(crate::import20_contest::hand_ability(id).unwrap());
    } else {
        d.abilities.push(ability(
            MILL,
            "本方圣甲虫产业附属进场：将本人牌库顶置于墓地",
            Timing::Fast,
            vec![],
            vec![],
            vec![Op::IndustryMill { observation: None }],
            Some(Event::IndustryAttachmentEntered),
        ));
        d.abilities.push(ability(
            SEALED,
            "角色牌封印到本方产业附属：该附属本回合获得调查1",
            Timing::Fast,
            vec![],
            vec![],
            vec![Op::IndustryInvestigation { observation: None }],
            Some(Event::IndustryCharacterSealed),
        ));
    }
    Some(d)
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|o| match o {
        Op::IndustryStepResource | Op::IndustryMill { .. } | Op::IndustryInvestigation { .. } => {
            true
        }
        Op::ForEachLivingPlayer(b) | Op::ForEachLivingPlayerFromActor(b) => owns(b),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(id: &str, a: &AbilitySpec) -> bool {
    CARDS.contains(&id)
        || [FUND, HAND, MILL, SEALED].contains(&a.key.as_str())
        || owns(&a.ops)
        || a.modes.iter().any(|m| owns(&m.ops))
        || a.costs
            .iter()
            .any(|c| matches!(c, Cost::IndustryExhaustOther))
}
pub(crate) fn frame_related(f: &ResolutionFrame) -> bool {
    CARDS.contains(&f.source.card.definition.as_str())
        || [FUND, HAND, MILL, SEALED].contains(&f.ability_key.as_str())
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
}
fn canonical(id: &str, a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition(id)?
        .abilities
        .into_iter()
        .find(|b| b.key == a.key)?;
    if matches!(a.key.as_str(), MILL | SEALED) {
        expected.ops = a.ops.clone();
        match a.ops.as_slice() {
            [Op::IndustryMill { .. }] if a.key == MILL => {}
            [Op::IndustryInvestigation { .. }] if a.key == SEALED => {}
            _ => return None,
        }
    }
    Some(expected)
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if canonical(id, a).is_none_or(|b| !same(a, &b)) {
        return Err("产业能力须保留完整原费用、事件和操作".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if let Some(expected) = definition(id) {
        if !same(d, &expected) {
            return Err("产业完整定义不能改写".into());
        }
    } else if d.abilities.iter().any(|a| related(id, a)) {
        return Err("产业程序不能移植".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if !CARDS.contains(&id) {
        return Ok(());
    }
    let expected = match a.kind.as_str() {
        "play" if a.ability_id.as_deref().is_none_or(|k| k == "attach") => Action {
            card_id: a.card_id.clone(),
            region: a.region,
            ability_id: a.ability_id.clone(),
            ..Action::new("play")
        },
        "activate" if id == "BQ092" && a.ability_id.as_deref() == Some(FUND) => Action {
            card_id: a.card_id.clone(),
            ability_id: a.ability_id.clone(),
            cost_selected: a.cost_selected.clone(),
            ..Action::new("activate")
        },
        "activate" if id == "BQ092" && a.ability_id.as_deref() == Some(HAND) => Action {
            card_id: a.card_id.clone(),
            ability_id: a.ability_id.clone(),
            ..Action::new("activate")
        },
        _ => return Err("产业只接受原结附、快速费用或神秘学博弈行动".into()),
    };
    if *a != expected {
        return Err("产业行动不可夹带额外载荷".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn industry_cost_candidates(&self, actor: usize, source: &str) -> Vec<String> {
        self.attachments
            .iter()
            .filter(|a| {
                a.card.id != source
                    && a.card.controller == actor
                    && !a.card.exhausted
                    && industry(&a.card)
                    && self.attachment_host_valid(a)
            })
            .map(|a| a.card.id.clone())
            .collect()
    }
    pub(crate) fn industry_pay_other(
        &mut self,
        actor: usize,
        source: &SourceSnapshot,
        a: &Action,
    ) -> RuleResult<PaidCost> {
        let ids = a
            .cost_selected
            .as_ref()
            .filter(|v| v.len() == 1)
            .ok_or("须横置另一个本方圣甲虫产业附属")?;
        let (_, c) = self
            .in_play(&ids[0])
            .filter(|(_, c)| {
                c.id != source.card.id
                    && c.controller == actor
                    && !c.exhausted
                    && industry(c)
                    && self
                        .attachments
                        .iter()
                        .any(|att| att.card.id == c.id && self.attachment_host_valid(att))
            })
            .ok_or("产业横置费用须为另一个未横置本方产业附属")?;
        let receipt = self.source_snapshot(c, self.in_play(&c.id).and_then(|(r, _)| r));
        self.board_mut(&ids[0]).unwrap().exhausted = true;
        Ok(PaidCost::IndustryExhausted(Box::new(receipt)))
    }
    fn industry_sources(&self, actor: usize) -> Vec<SourceSnapshot> {
        self.attachments
            .iter()
            .filter(|a| {
                a.card.definition == "BQ093"
                    && a.card.controller == actor
                    && !a.card.face_down
                    && self.attachment_host_valid(a)
            })
            .map(|a| self.source_snapshot(&a.card, self.attachment_region(a)))
            .collect()
    }
    pub(crate) fn observe_industry_entry(&mut self, subject: &SourceSnapshot) {
        if !industry(&subject.card) {
            return;
        }
        let sources = self.industry_sources(subject.card.controller);
        if sources.is_empty() {
            return;
        }
        let event = IndustryEvent {
            event_id: self.id(),
            turn: self.turn,
            subject: subject.card.clone(),
            host: None,
        };
        self.industry_declare(sources, event, false);
    }
    pub(crate) fn observe_industry_sealed(&mut self, sealed: &Card, host_id: &str) {
        if catalog::card(&sealed.definition).kind != "character" {
            return;
        }
        let Some(a) = self
            .attachments
            .iter()
            .find(|a| a.card.id == host_id && industry(&a.card) && self.attachment_host_valid(a))
        else {
            return;
        };
        let host = self.source_snapshot(&a.card, self.attachment_region(a));
        let sources = self.industry_sources(a.card.controller);
        if sources.is_empty() {
            return;
        }
        let event = IndustryEvent {
            event_id: self.id(),
            turn: self.turn,
            subject: sealed.clone(),
            host: Some(host),
        };
        self.industry_declare(sources, event, true);
    }
    fn industry_declare(
        &mut self,
        sources: Vec<SourceSnapshot>,
        event: IndustryEvent,
        sealed: bool,
    ) {
        for source in sources {
            let key = if sealed { SEALED } else { MILL };
            let mut ability = definition("BQ093")
                .unwrap()
                .abilities
                .into_iter()
                .find(|a| a.key == key)
                .unwrap();
            ability.ops = vec![if sealed {
                Op::IndustryInvestigation {
                    observation: Some(Box::new(event.clone())),
                }
            } else {
                Op::IndustryMill {
                    observation: Some(Box::new(event.clone())),
                }
            }];
            self.effects.push_back(Effect::Declare {
                declaration: Declaration {
                    actor: source.card.controller,
                    source,
                    ability,
                },
            });
        }
    }
    fn industry_observation(&self, actor: usize, ops: &[Op]) -> RuleResult<()> {
        let (e, sealed) = match ops {
            [Op::IndustryMill {
                observation: Some(e),
            }] => (e, false),
            [Op::IndustryInvestigation {
                observation: Some(e),
            }] => (e, true),
            _ => return Err("产业观察缺少实际事件".into()),
        };
        if e.turn != self.turn
            || !self.mystic_instance_valid(&e.event_id)
            || e.subject.owner >= self.players.len()
            || e.subject.controller >= self.players.len()
        {
            return Err("产业事件身份无效".into());
        }
        self.mobility_immutable_identity(&e.subject)?;
        if sealed {
            let h = e.host.as_ref().ok_or("产业封印缺少原宿主")?;
            if catalog::card(&e.subject.definition).kind != "character"
                || h.card.controller != actor
                || !industry(&h.card)
            {
                return Err("仅角色封印到本方产业附属才触发".into());
            }
            self.mobility_immutable_identity(&h.card)?;
        } else if e.host.is_some() || !industry(&e.subject) || e.subject.controller != actor {
            return Err("仅本方产业附属进场才触发".into());
        }
        Ok(())
    }
    pub(crate) fn validate_industry_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if d.source.card.definition != "BQ093"
            || d.source.card.controller != d.actor
            || d.source.card.face_down
        {
            return Err("产业观察须来自原样本陈列室".into());
        }
        validate_ability("BQ093", &d.ability)?;
        self.mobility_immutable_identity(&d.source.card)?;
        self.industry_observation(d.actor, &d.ability.ops)
    }
    pub(crate) fn validate_industry_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_resource_frame(f)?;
        if f.ability_key == HAND {
            return self.validate_import20_frame(f);
        }
        let id = f.source.card.definition.as_str();
        let mut a = definition(id)
            .and_then(|d| d.abilities.into_iter().find(|a| a.key == f.ability_key))
            .ok_or("未知产业帧")?;
        if f.actor >= self.players.len()
            || f.source.card.controller != f.actor
            || f.source.card.face_down
            || f.cursor > f.steps.len()
            || f.steps.iter().any(|s| s.context != f.actor)
            || f.cursor == 0
                && !matches!(f.guard, GuardState::Unchecked)
                && !(f.steps.is_empty()
                    && matches!(f.guard, GuardState::Accepted | GuardState::Cancelled))
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)
        {
            return Err("产业来源、阶段或执行席位无效".into());
        }
        self.mobility_immutable_identity(&f.source.card)?;
        if f.ability_key == "attach" {
            if f.targets.len() != 1
                || !same(&f.targets[0].spec, &a.targets[0])
                || !f.steps.is_empty()
                || f.source.region.is_some()
                || f.source.attachment_host_instance.is_some()
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=catalog::card(id).cost as usize&&ids.iter().all(|id|f.source.valid_printed_payment_id(id))&&ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
            {
                return Err("产业结附须保留原地区和实际支付".into());
            }
            return Ok(());
        }
        if !f.targets.is_empty() || f.chosen_region.is_some() {
            return Err("产业能力不另选效果目标".into());
        }
        a.ops = f.steps.iter().map(|s| s.op.clone()).collect();
        validate_ability(id, &a)?;
        if f.ability_key == FUND {
            if f.source.card.exhausted
                || f.source.attachment_host_instance.is_none()
                || !matches!(f.already_paid.as_slice(),[PaidCost::Exhausted(id),PaidCost::IndustryExhausted(other)]if id==&f.source.card.id&&other.card.id!=*id&&!other.card.exhausted&&other.card.controller==f.actor&&industry(&other.card)&&other.attachment_host_instance.is_some())
            {
                return Err("基金中心须保留两个不同本方产业实际横置费用".into());
            }
        } else {
            if !f.already_paid.is_empty() {
                return Err("产业触发没有额外费用".into());
            }
            self.industry_observation(f.actor, &a.ops)?;
        }
        Ok(())
    }
    pub(crate) fn industry_gain_resource(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_industry_frame(f)?;
        if f.ability_key != FUND {
            return Err("非基金中心费用效果".into());
        }
        let step = self.resource_step().ok_or("当前没有有效步骤")?;
        let credit = crate::step_resources::StepCredit {
            id: self.id(),
            actor: f.actor,
            turn: self.turn,
            step,
            source: Box::new(f.clone()),
        };
        self.step_resources.push(credit);
        Ok(())
    }
    pub(crate) fn industry_resolve_bonus(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_industry_frame(f)?;
        let [Step {
            op: Op::IndustryInvestigation {
                observation: Some(e),
            },
            ..
        }] = f.steps.as_slice()
        else {
            return Err("产业调查缺少原封印事件".into());
        };
        let h = e.host.as_ref().unwrap();
        if self.in_play(&h.card.id).is_some_and(|(_, c)| industry(c)) {
            self.industry_bonuses.push(IndustryBonus {
                target_instance: h.card.id.clone(),
                turn: self.turn,
                source: Box::new(f.clone()),
            });
        }
        Ok(())
    }
    pub(crate) fn industry_investigation(&self, c: &Card) -> u32 {
        if !industry(c) {
            return 0;
        }
        self.industry_bonuses
            .iter()
            .filter(|b| b.target_instance == c.id && b.turn == self.turn)
            .count() as u32
    }
    pub(crate) fn validate_industry_state(&self) -> RuleResult<()> {
        self.validate_step_resource_state()?;
        for b in &self.industry_bonuses {
            if b.turn != self.turn || b.source.ability_key != SEALED {
                return Err("产业调查增益回合或来源无效".into());
            }
            self.validate_industry_frame(&b.source)?;
        }
        for item in &self.stack {
            if let Some(f) = &item.frame {
                if frame_related(f) {
                    self.validate_industry_frame(f)?;
                }
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame } if frame_related(frame) => {
                    self.validate_industry_frame(frame)?
                }
                Effect::Declare { declaration: d }
                    if related(&d.source.card.definition, &d.ability) =>
                {
                    self.validate_industry_declaration(d)?
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Frame { frame, .. } if frame_related(frame) => {
                    self.validate_industry_frame(frame)?
                }
                ChoiceResolution::Declare { declaration: d, .. }
                    if related(&d.source.card.definition, &d.ability) =>
                {
                    self.validate_industry_declaration(d)?
                }
                _ => {}
            }
        }
        Ok(())
    }
}
