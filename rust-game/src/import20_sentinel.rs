//! CQ18/CQ19: bounded Sentinel programs on the existing interpreter and grants.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const CARDS: &[&str] = &["CQ18", "CQ19"];
pub(crate) const ENTRY: &str = "import20-cq18-entry-locks";
pub(crate) const LEAVE: &str = "import20-cq18-locked-departure";
pub(crate) const CHARGE_ENTRY: &str = "import20-cq19-sentinel-entry";
pub(crate) const CHARGE: &str = "import20-cq19-charge";
pub(crate) const FIRE: &str = "import20-cq19-fire";
pub(crate) const SHOWN: &str = "import20-cq18-shown-lock";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChargePayment {
    pub amount: u32,
    pub source: Card,
    pub attachments: Vec<Attachment>,
    pub taken: Vec<(String, u32)>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockedDeparture {
    pub event_id: String,
    pub turn: u32,
    pub departed: SourceSnapshot,
    pub attachments: Vec<Attachment>,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    if id == "CQ18" {
        return Some(with_abilities(vec![
            ability(
                ENTRY,
                "进场：本地区所有敌方角色与暗藏者各获得锁定1",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::CQ18EntryLocks],
                Some(Event::Enter),
            ),
            ability(
                LEAVE,
                "已锁定敌方角色或暗藏者离场：抓1",
                Timing::Fast,
                vec![],
                vec![],
                vec![Op::CQ18LeaveDraw { observation: None }],
                Some(Event::CQ18LockedEnemyLeft),
            ),
        ]));
    }
    if id != "CQ19" {
        return None;
    }
    let mut d = attachment_definition(target(
        Zone::Society,
        EntityKind::Any,
        Relation::ControlledByActor,
        Range::Anywhere,
    ));
    d.abilities.push(ability(
        CHARGE_ENTRY,
        "进场：若本方秘社是哨兵，充能1",
        Timing::Fast,
        vec![],
        vec![],
        vec![Op::CQ19AddCharge],
        Some(Event::Enter),
    ));
    for (key, label, costs, targets, ops) in [
        (
            CHARGE,
            "行动，横置：充能1",
            vec![Cost::ExhaustSource],
            vec![],
            vec![Op::CQ19AddCharge],
        ),
        (
            FIRE,
            "行动，横置并移除X充能：对目标角色造成X伤害",
            vec![Cost::ExhaustSource, Cost::RemoveChargeX],
            vec![target(
                Zone::Board,
                EntityKind::Character,
                Relation::Any,
                Range::Anywhere,
            )],
            vec![Op::CQ19Fire],
        ),
    ] {
        let mut a = ability(key, label, Timing::Standard, costs, targets, ops, None);
        a.activation_only = true;
        d.abilities.push(a);
    }
    Some(d)
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|o| match o {
        Op::CQ19AddCharge | Op::CQ19Fire | Op::CQ18EntryLocks | Op::CQ18LeaveDraw { .. } => true,
        Op::ForEachLivingPlayer(b) | Op::ForEachLivingPlayerFromActor(b) => owns(b),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(id: &str, a: &AbilitySpec) -> bool {
    CARDS.contains(&id)
        || a.key.starts_with("import20-cq18-")
        || a.key.starts_with("import20-cq19-")
        || owns(&a.ops)
        || a.modes.iter().any(|m| owns(&m.ops))
        || a.costs.iter().any(|c| matches!(c, Cost::RemoveChargeX))
}
pub(crate) fn frame_related(f: &ResolutionFrame) -> bool {
    CARDS.contains(&f.source.card.definition.as_str())
        || f.ability_key.starts_with("import20-cq18-")
        || f.ability_key.starts_with("import20-cq19-")
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    let mut expected = definition(id)
        .and_then(|d| d.abilities.into_iter().find(|p| p.key == a.key))
        .ok_or("哨兵双卡只有完整原始程序")?;
    if let [Op::CQ18LeaveDraw { observation }] = a.ops.as_slice() {
        expected.ops = vec![Op::CQ18LeaveDraw {
            observation: observation.clone(),
        }];
    }
    if !same(&expected, a) {
        return Err("哨兵双卡不能改写原时序、费用、目标或完整程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if let Some(expected) = definition(id) {
        if !same(&expected, d) {
            return Err("哨兵双卡完整定义不能删改".into());
        }
    } else if d.abilities.iter().any(|a| related(id, a)) {
        return Err("哨兵双卡程序不能静态移植".into());
    }
    Ok(())
}
pub(crate) fn validate_action(a: &AbilitySpec, x: &Action) -> RuleResult<()> {
    if x.kind != "activate" && x.kind != "play" {
        return Err("天基武器只按原结附或行动命令发动".into());
    }
    if x.amount.is_some() != (a.key == FIRE)
        || *x
            != (Action {
                card_id: x.card_id.clone(),
                ability_id: x.ability_id.clone(),
                target_id: x.target_id.clone(),
                amount: x.amount,
                ..Action::new(&x.kind)
            })
    {
        return Err("天基武器只接受原来源、目标及声明的X数量".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn sentinel_pair_trigger_available(&self, d: &Declaration) -> bool {
        d.source.card.definition != "CQ19"
            || d.ability.key != CHARGE_ENTRY
            || self.wm044_has_sentinel(d.actor)
    }
    pub(crate) fn cq19_amount_range(&self, a: &Action) -> Option<AmountRange> {
        if a.kind != "activate" || a.ability_id.as_deref() != Some(FIRE) {
            return None;
        }
        let (_, c) = self.in_play(a.card_id.as_deref()?)?;
        (c.definition == "CQ19").then(|| AmountRange {
            min: 0,
            max: self.effective_marker_count(c, crate::batch4_markers::MarkerKind::Charge),
        })
    }
    pub(crate) fn cq19_pay_charge(
        &mut self,
        source: &SourceSnapshot,
        a: &Action,
    ) -> RuleResult<PaidCost> {
        if source.card.definition != "CQ19" || a.ability_id.as_deref() != Some(FIRE) {
            return Err("充能支付只属于原天基武器".into());
        }
        let amount = a.amount.ok_or("请声明要移除的X个充能")?;
        let (_, c) = self.in_play(&source.card.id).ok_or("原天基武器已离场")?;
        if amount > self.effective_marker_count(c, crate::batch4_markers::MarkerKind::Charge) {
            return Err("充能数量不足".into());
        }
        let ids = self.marker_carriers(c);
        let attachments = self
            .attachments
            .iter()
            .filter(|a| ids.contains(&a.card.id))
            .cloned()
            .collect();
        let mut payment = ChargePayment {
            amount,
            source: source.card.clone(),
            attachments,
            taken: vec![],
        };
        let mut remaining = amount;
        for id in ids {
            let c = self.board_mut(&id).unwrap();
            let n = remaining.min(c.charge_markers);
            if n > 0 {
                payment.taken.push((id, n));
                c.charge_markers -= n;
                remaining -= n;
            }
        }
        if remaining != 0 {
            return Err("充能物理承载者改变".into());
        }
        Ok(PaidCost::CQ19ChargeRemoved(Box::new(payment)))
    }
    fn sentinel_pair_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        if !CARDS.contains(&s.card.definition.as_str())
            || actor >= self.players.len()
            || s.card.controller != actor
            || s.card.owner >= self.players.len()
            || !self.mystic_instance_valid(&s.card.id)
            || s.observed_death.is_some()
            || s.observed_exposure.is_some()
            || s.observed_mobility_grant.is_some()
            || s.death_echo.is_some()
        {
            return Err("哨兵双卡原来源或操控者不符".into());
        }
        self.mobility_immutable_identity(&s.card)
    }
    pub(crate) fn validate_sentinel_pair_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? {
            return Ok(());
        }
        self.sentinel_pair_source(d.actor, &d.source)?;
        validate_ability(&d.source.card.definition, &d.ability)?;
        if d.ability.event.is_none() || d.source.card.face_down || d.source.play_source.is_some() {
            return Err("哨兵双卡声明须来自原实际触发".into());
        }
        if d.source.card.definition == "CQ18" {
            if d.source.region.is_none() || d.source.source_region_instance.is_none() {
                return Err("任秋雨触发须保留原地区".into());
            }
            if d.ability.key == LEAVE {
                self.cq18_departure_valid(d.actor, &d.ability.ops)?;
            }
        } else if d.ability.key != CHARGE_ENTRY
            || d.source.attachment_host_instance.as_deref()
                != Some(format!("society:p{}", d.actor).as_str())
        {
            return Err("天基武器进场须保留原本人秘社宿主".into());
        }
        Ok(())
    }
    fn cq19_charge_valid(&self, f: &ResolutionFrame, p: &ChargePayment) -> RuleResult<()> {
        if !same(&p.source, &f.source.card) {
            return Err("充能收据须保留支付前原来源".into());
        }
        let mut cards = vec![&p.source];
        let mut seen = BTreeSet::from([p.source.id.as_str()]);
        let mut left = p.attachments.iter().collect::<Vec<_>>();
        while !left.is_empty() {
            let before = left.len();
            left.retain(|a| {
                if seen.contains(a.host_id.as_str()) && seen.insert(a.card.id.as_str()) {
                    cards.push(&a.card);
                    false
                } else {
                    true
                }
            });
            if left.len() == before {
                return Err("充能收据须为原宿主的有限实际附属链".into());
            }
        }
        if p.attachments.iter().any(|a| {
            a.card.face_down && !crate::batch4_plans::secret(&a.card)
                || !catalog::catalog()
                    .cards
                    .iter()
                    .any(|d| d.id == a.card.definition && catalog::has_kind(d, "attachment"))
        }) {
            return Err("充能物理承载者不是合法明置附属".into());
        }
        let mut ids = BTreeSet::new();
        let mut total = 0u32;
        for (id, n) in &p.taken {
            let c = cards
                .iter()
                .find(|c| c.id == *id)
                .ok_or("充能支付承载者不在原链")?;
            if *n == 0 || *n > c.charge_markers || !ids.insert(id) {
                return Err("充能必须按原实际不同载体移除".into());
            }
            total = total.checked_add(*n).ok_or("充能支付溢出")?;
        }
        if total != p.amount {
            return Err("充能支付总数必须等于原X".into());
        }
        Ok(())
    }
    pub(crate) fn validate_sentinel_pair_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_resource_frame(f)?;
        if self.validated_cross_card_frame(f)? {
            return Ok(());
        }
        self.sentinel_pair_source(f.actor, &f.source)?;
        if !self.mystic_instance_valid(&f.frame_id)
            || f.frame_id == f.source.card.id
            || f.cursor > f.steps.len()
            || f.cursor == 0
                && !matches!(f.guard, GuardState::Unchecked)
                && !(f.steps.is_empty()
                    && matches!(f.guard, GuardState::Accepted | GuardState::Cancelled))
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)
        {
            return Err("哨兵双卡帧阶段或身份无效".into());
        }
        if f.source.card.definition == "CQ18"
            && matches!(f.ability_key.as_str(), "deploy" | "reveal")
        {
            if !f.steps.is_empty()
                || f.source.region != f.chosen_region
                || f.source.region.is_none()
                || !f.targets.is_empty()
                || f.ability_key == "deploy"
                    && (!f.source.valid_character_play() || f.source.card.face_down)
                || f.ability_key == "reveal" && !f.source.card.face_down
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=4&&ids.iter().all(|id|f.source.valid_printed_payment_id(id)))
            {
                return Err("任秋雨进场须保留实际费用与原地区".into());
            }
            return Ok(());
        }
        let mut a = definition(&f.source.card.definition)
            .and_then(|d| d.abilities.into_iter().find(|a| a.key == f.ability_key))
            .ok_or("未知哨兵双卡执行程序")?;
        if f.ability_key == "attach" {
            if !f.steps.is_empty()
                || f.source.region.is_some()
                || f.source.attachment_host_instance.is_some()
                || f.targets.len() != 1
                || !same(&f.targets[0].spec, &a.targets[0])
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]if f.money_units(0)<=4&&ids.iter().all(|id|f.source.valid_printed_payment_id(id)))
            {
                return Err("天基武器结附须保留原本人秘社及实际支付".into());
            }
            return Ok(());
        }
        a.ops = f.steps.iter().map(|s| s.op.clone()).collect();
        validate_ability(&f.source.card.definition, &a)?;
        if f.chosen_region.is_some()
            || f.targets.len() != a.targets.len()
            || f.targets
                .iter()
                .zip(&a.targets)
                .any(|(t, s)| !same(&t.spec, s))
            || f.steps.iter().any(|s| s.context != f.actor)
        {
            return Err("哨兵双卡不能更换目标或执行环境".into());
        }
        if a.event.is_some() {
            if !f.already_paid.is_empty() {
                return Err("哨兵双卡触发没有额外费用".into());
            }
            self.validate_sentinel_pair_declaration(&Declaration {
                actor: f.actor,
                source: f.source.clone(),
                ability: a,
            })?;
        } else {
            if f.source.card.definition != "CQ19"
                || f.source.card.exhausted
                || f.source.attachment_host_instance.as_deref()
                    != Some(format!("society:p{}", f.actor).as_str())
            {
                return Err("天基武器须实际横置原本人秘社附属".into());
            }
            match f.already_paid.as_slice() {
                [PaidCost::Exhausted(id)] if *id == f.source.card.id && f.ability_key == CHARGE => {
                }
                [PaidCost::Exhausted(id), PaidCost::CQ19ChargeRemoved(p)]
                    if *id == f.source.card.id && f.ability_key == FIRE =>
                {
                    self.cq19_charge_valid(f, p)?
                }
                _ => return Err("天基武器须保留一次真实横置及X充能收据".into()),
            }
        }
        Ok(())
    }
    pub(crate) fn cq19_add_charge(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_sentinel_pair_frame(f)?;
        if f.ability_key == CHARGE_ENTRY && !self.wm044_has_sentinel(f.actor) {
            return Ok(());
        }
        if let Some(c) = self
            .board_mut(&f.source.card.id)
            .filter(|c| c.definition == "CQ19" && !c.face_down)
        {
            c.charge_markers = c
                .charge_markers
                .checked_add(1)
                .ok_or("充能标志超过当前表示范围")?;
        }
        Ok(())
    }
    pub(crate) fn cq19_fire(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_sentinel_pair_frame(f)?;
        let Some(PaidCost::CQ19ChargeRemoved(p)) = f.already_paid.get(1) else {
            return Err("缺少天基武器实际充能费用".into());
        };
        self.damage([(f.targets[0].id.clone(), p.amount)].into_iter().collect())
    }
    pub(crate) fn cq18_entry_locks(&mut self, _f: &ResolutionFrame) -> RuleResult<()> {
        Err("CQ18 entry WIP".into())
    }
    pub(crate) fn cq18_leave_draw(&mut self, _f: &ResolutionFrame) -> RuleResult<()> {
        Err("CQ18 leave WIP".into())
    }
    fn cq18_departure_valid(&self, _actor: usize, _ops: &[Op]) -> RuleResult<()> {
        Err("CQ18 departure WIP".into())
    }
    pub(crate) fn validate_sentinel_pair_state(&self) -> RuleResult<()> {
        for item in &self.stack {
            if let Some(f) = &item.frame {
                if frame_related(f) && !self.validated_cross_card_frame(f)? {
                    self.validate_sentinel_pair_frame(f)?;
                    let play = matches!(f.ability_key.as_str(), "attach" | "deploy" | "reveal");
                    if item.id != f.frame_id
                        || item.controller != f.actor
                        || item.card.is_some() != play
                        || item.reveal != (f.ability_key == "reveal")
                        || f.cursor != 0
                        || !matches!(f.guard, GuardState::Unchecked)
                        || item.target != f.targets.first().map(|t| t.id.clone())
                    {
                        return Err("哨兵双卡响应帧外壳须保持实际原来源与目标".into());
                    }
                }
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d }
                    if related(&d.source.card.definition, &d.ability)
                        && !self.validated_cross_card_declaration(d)? =>
                {
                    self.validate_sentinel_pair_declaration(d)?
                }
                Effect::Frame { frame: f }
                    if frame_related(f) && !self.validated_cross_card_frame(f)? =>
                {
                    self.validate_sentinel_pair_frame(f)?
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } if related(&d.source.card.definition, &d.ability)
                    && !self.validated_cross_card_declaration(d)? =>
                {
                    self.validate_sentinel_pair_declaration(d)?;
                    let expected = Choice {
                        id: p.choice.id.clone(),
                        kind: "trigger".into(),
                        title: format!(
                            "{}：{}",
                            catalog::card(&d.source.card.definition).name,
                            d.ability.label
                        ),
                        description: "选择符合数量限制的选项，然后确认。".into(),
                        player_id: player_id(d.actor),
                        options: vec![ChoiceOption {
                            id: "accept".into(),
                            label: "发动触发能力".into(),
                            card: None,
                        }],
                        preview_cards: vec![],
                        min: Some(0),
                        max: Some(1),
                        amount: None,
                        allow_decline: Some(true),
                    };
                    if p.seat != d.actor
                        || p.choice.id.is_empty()
                        || !matches!(stage, DeclareChoice::Accept)
                        || !same(&p.choice, &expected)
                    {
                        return Err("哨兵双卡触发只接受原玩家一次发动或放弃".into());
                    }
                }
                ChoiceResolution::Frame { frame: f, .. }
                    if frame_related(f) && !self.validated_cross_card_frame(f)? =>
                {
                    if f.ability_key != SHOWN {
                        return Err("哨兵双卡原子能力没有临时帧内选择".into());
                    }
                    self.validate_wm043_post(f)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
