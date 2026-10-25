//! Finite pre-grave departure continuation. Death observation is unchanged;
//! only an XG25's destination awaits its ordinary optional trigger/responses.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DeathDestination {
    Grave { instance: String },
    Sealed { instance: String, host: String },
    Removed,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PendingDeath {
    pub turn: u32,
    pub source: SourceSnapshot,
    pub transition_instance: String,
    pub card: Option<Card>,
    pub baseline_stack: Vec<String>,
    pub declaration_choice: Option<String>,
    pub destination: Option<DeathDestination>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeathWait {
    pub source: SourceSnapshot,
    pub destination: Option<DeathDestination>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DeathCallerKind {
    Frame,
    JZ24 {
        remaining_players: Vec<usize>,
    },
    JZ35 {
        returns: Vec<crate::jz35::HostReturn>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeathCaller {
    pub frame: Box<ResolutionFrame>,
    pub waits: Vec<DeathWait>,
    pub kind: DeathCallerKind,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
impl Game {
    pub(crate) fn death_instances(&self) -> BTreeSet<String> {
        self.death_replacements
            .iter()
            .map(|d| d.source.card.id.clone())
            .collect()
    }
    pub(crate) fn hold_pregrave_death(&mut self, source: &SourceSnapshot, card: Card) {
        debug_assert!(source.card.definition == "XG25" && !source.card.face_down);
        self.death_replacements.push(PendingDeath {
            turn: self.turn,
            source: source.clone(),
            transition_instance: card.id.clone(),
            card: Some(card),
            baseline_stack: self.stack.iter().map(|s| s.id.clone()).collect(),
            declaration_choice: None,
            destination: None,
        });
    }
    pub(crate) fn death_boundary_waits(&self, id: &str) -> bool {
        self.death_replacements
            .iter()
            .find(|d| d.source.card.id == id)
            .is_some_and(|d| {
                d.destination.is_none()
                    && self.stack.iter().any(|s| !d.baseline_stack.contains(&s.id))
            })
    }
    /// Freeze the actual lower stack when this optional trigger is offered,
    /// after payment has created its original caller and any earlier sibling
    /// declarations. No card-specific receipt exception or second stack is used.
    pub(crate) fn record_pregrave_choice(&mut self) -> RuleResult<()> {
        let Some((source, choice)) = self.pending.as_ref().and_then(|p| match &p.resolution {
            ChoiceResolution::Declare { declaration, .. }
                if declaration.ability.key == crate::next20_black_wraiths::SEAL =>
            {
                Some((declaration.source.card.id.clone(), p.choice.id.clone()))
            }
            _ => None,
        }) else {
            return Ok(());
        };
        let baseline = self.stack.iter().map(|s| s.id.clone()).collect();
        let d = self
            .death_replacements
            .iter_mut()
            .find(|d| d.source.card.id == source)
            .ok_or("死亡待选缺少原离场")?;
        if d.declaration_choice.is_some() {
            return Err("同次死亡不能重复声明替代能力".into());
        }
        d.baseline_stack = baseline;
        d.declaration_choice = Some(choice);
        Ok(())
    }
    pub(crate) fn decline_pregrave_declaration(&mut self, d: &Declaration) -> RuleResult<()> {
        if d.ability.key == crate::next20_black_wraiths::SEAL {
            self.validate_xg25_death_source(d.actor, &d.source)?;
            self.commit_pregrave_fallback(&d.source.card.id)?;
        }
        Ok(())
    }
    pub(crate) fn cancel_pregrave_frame(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        if f.ability_key == crate::next20_black_wraiths::SEAL {
            self.validate_xg25_death_source(f.actor, &f.source)?;
            self.commit_pregrave_fallback(&f.source.card.id)?;
        }
        Ok(())
    }
    /// Preserve the exact old caller cursor, payment and frozen source. The
    /// destination barriers already precede its old queue tail (including Bury).
    pub(crate) fn defer_death_caller(
        &mut self,
        frame: ResolutionFrame,
        kind: DeathCallerKind,
        before: &BTreeSet<String>,
    ) -> bool {
        let waits = self
            .death_replacements
            .iter()
            .filter(|d| !before.contains(&d.source.card.id))
            .map(|d| DeathWait {
                source: d.source.clone(),
                destination: d.destination.clone(),
            })
            .collect::<Vec<_>>();
        if waits.is_empty() {
            return false;
        }
        let at=self.effects.iter().rposition(|e|matches!(e,Effect::DeathFinish{death_instance} if waits.iter().any(|w|w.source.card.id==*death_instance))).expect("new held departures have destination barriers")+1;
        self.effects.insert(
            at,
            Effect::DeathCaller {
                caller: Box::new(DeathCaller {
                    frame: Box::new(frame),
                    waits,
                    kind,
                }),
            },
        );
        true
    }
    fn pregrave_source(&self, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != "XG25"
            || c.face_down
            || c.owner >= self.players.len()
            || c.controller >= self.players.len()
            || !self.mystic_instance_valid(&c.id)
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance
                .as_deref()
                .is_none_or(|id| !self.mystic_instance_valid(id))
            || s.play_source.is_some()
            || s.attachment_host_instance.is_some()
            || s.death_echo.is_some()
            || s.observed_death.is_some()
            || s.observed_exposure.is_some()
            || s.observed_mobility_grant.is_some()
            || self.batch3_physical_cards().any(|live| live.id == c.id)
        {
            return Err("待定死亡必须属于已离场的原正面伥鬼及其原地区".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_xg25_death_source(
        &self,
        actor: usize,
        s: &SourceSnapshot,
    ) -> RuleResult<()> {
        self.pregrave_source(s)?;
        let d = self
            .death_replacements
            .iter()
            .find(|d| d.source.card.id == s.card.id)
            .ok_or("伥鬼死亡触发缺少真实待定离场")?;
        if actor != s.card.controller
            || d.turn != self.turn
            || !same(&d.source, s)
            || d.card.is_none()
            || d.destination.is_some()
        {
            return Err("伥鬼死亡触发更换了原操控者、来源或去向阶段".into());
        }
        Ok(())
    }
    pub(crate) fn validate_xg25_death_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_xg25_death_source(f.actor, &f.source)?;
        let choice = self
            .death_replacements
            .iter()
            .find(|d| d.source.card.id == f.source.card.id)
            .and_then(|d| d.declaration_choice.as_deref())
            .ok_or("死亡响应缺少原目标待选实例")?;
        if !self
            .mystic_instance_number(choice)
            .zip(self.mystic_instance_number(&f.frame_id))
            .is_some_and(|(choice, frame)| choice.checked_add(1) == Some(frame))
        {
            return Err("死亡响应必须承接原目标待选的下一实例".into());
        }
        let a = crate::next20_black_wraiths::definition("XG25")
            .unwrap()
            .abilities
            .into_iter()
            .find(|a| a.key == crate::next20_black_wraiths::SEAL)
            .unwrap();
        if f.ability_key != a.key
            || !self
                .mystic_instance_number(&f.source.card.id)
                .zip(self.mystic_instance_number(&f.frame_id))
                .is_some_and(|(s, f)| s < f)
            || f.steps.len() != 1
            || f.steps[0].context != f.actor
            || !same(&f.steps[0].op, &a.ops[0])
            || f.targets.len() != 1
            || !same(&f.targets[0].spec, &a.targets[0])
            || !f.already_paid.is_empty()
            || !f.resource_payments.is_empty()
            || f.chosen_region.is_some()
            || f.cursor > 1
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
        {
            return Err("伥鬼死亡封印须保留原目标、程序、游标及免费收据".into());
        }
        let t = &f.targets[0];
        if !self.mystic_instance_valid(&t.id)
            || t.public.instance_id != t.id
            || t.public.kind != "character"
            || !t.public.valid
            || t.public.status != "valid"
            || t.public.invalid_reason.is_some()
            || t.region_instance.is_some()
        {
            return Err("伥鬼死亡目标必须保留声明时的原角色".into());
        }
        Ok(())
    }
    pub(crate) fn resolve_xg25_death(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_xg25_death_frame(f)?;
        let host = &f.targets[0].id;
        // Card-text inference accepted as test premise: the condition is in the
        // effect body, so response-time loss of the moon falls back to grave.
        if self.xg25_replaces_death(&f.source)
            && self.seal_host_valid(host)
            && self.valid_bound_target(f.actor, &f.source, &f.targets[0])
        {
            let index = self
                .death_replacements
                .iter()
                .position(|d| d.source.card.id == f.source.card.id)
                .unwrap();
            let c = self.death_replacements[index].card.take().unwrap();
            self.seal_card_on_valid_host(c, f.actor, host);
            let instance = self.sealed_cards.last().unwrap().card.id.clone();
            self.death_replacements[index].destination = Some(DeathDestination::Sealed {
                instance,
                host: host.clone(),
            });
        } else {
            self.commit_pregrave_fallback(&f.source.card.id)?;
        }
        Ok(())
    }
    fn commit_pregrave_fallback(&mut self, id: &str) -> RuleResult<()> {
        let index = self
            .death_replacements
            .iter()
            .position(|d| d.source.card.id == id)
            .ok_or("待定死亡回退缺少原实例")?;
        if let Some(c) = self.death_replacements[index].card.take() {
            let owner = c.owner;
            let instance = c.id.clone();
            let destination = if self.players[owner].eliminated {
                DeathDestination::Removed
            } else {
                self.commit_owner_grave(c, None);
                DeathDestination::Grave { instance }
            };
            self.death_replacements[index].destination = Some(destination);
        }
        Ok(())
    }
    pub(crate) fn finish_pregrave_death(&mut self, id: &str) -> RuleResult<()> {
        if self.death_boundary_waits(id) {
            return Err("死亡去向须等待原响应链结束".into());
        }
        // Decline, no candidate, cancelled/terminated frame, or lost target all
        // use the same real fallback. No exception is used as a continuation.
        self.commit_pregrave_fallback(id)?;
        let index = self
            .death_replacements
            .iter()
            .position(|d| d.source.card.id == id)
            .ok_or("死亡去向不可重复收束")?;
        let completed = self.death_replacements.remove(index);
        let result = completed.destination.ok_or("死亡缺少最终去向")?;
        for e in &mut self.effects {
            if let Effect::DeathCaller { caller } = e {
                for w in &mut caller.waits {
                    if w.source.card.id == id {
                        w.destination = Some(result.clone());
                    }
                }
                if let DeathCallerKind::JZ35 { returns } = &mut caller.kind {
                    for r in returns {
                        if r.death.deferred_death.as_deref() == Some(id) {
                            r.death.deferred_death = None;
                            r.death.grave_instance = match &result {
                                DeathDestination::Grave { instance } => instance.clone(),
                                _ => String::new(),
                            };
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn validate_destination(
        &self,
        s: &SourceSnapshot,
        d: &DeathDestination,
        transition: Option<&str>,
    ) -> RuleResult<()> {
        let id = match d {
            DeathDestination::Grave { instance } => {
                if transition.is_some_and(|id| id != instance) {
                    return Err("入墓去向必须保留原换号实例".into());
                }
                instance
            }
            DeathDestination::Sealed { instance, host } => {
                if !self.mystic_instance_valid(host)
                    || transition.is_some_and(|id| {
                        !self
                            .mystic_instance_number(id)
                            .zip(self.mystic_instance_number(instance))
                            .is_some_and(|(a, b)| a < b)
                    })
                {
                    return Err("封印去向载体或实例无效".into());
                }
                instance
            }
            DeathDestination::Removed => {
                return if self.players[s.card.owner].eliminated {
                    Ok(())
                } else {
                    Err("尚存玩家的死亡牌不能无故移除".into())
                }
            }
        };
        if !self
            .mystic_instance_number(&s.card.id)
            .zip(self.mystic_instance_number(id))
            .is_some_and(|(a, b)| a < b)
            || self
                .batch3_physical_cards()
                .any(|c| c.id == *id && (c.definition != "XG25" || c.owner != s.card.owner))
        {
            return Err("死亡最终去向更换了原牌身份或拥有者".into());
        }
        Ok(())
    }
    pub(crate) fn validate_death_caller(&self, c: &DeathCaller) -> RuleResult<()> {
        let f = &c.frame;
        if c.waits.is_empty()
            || f.actor >= self.players.len()
            || !matches!(f.guard, GuardState::Accepted)
            || f.cursor == 0
            || f.cursor > f.steps.len()
        {
            return Err("死亡原调用方缺少已执行游标及原去向收据".into());
        }
        if f.source.card.controller != f.actor
            || f.source.card.owner >= self.players.len()
            || !self
                .mystic_instance_number(&f.source.card.id)
                .zip(self.mystic_instance_number(&f.frame_id))
                .is_some_and(|(a, b)| a < b)
        {
            return Err("死亡原调用方更换了原来源、操控者或响应实例".into());
        }
        self.mobility_immutable_identity(&f.source.card)?;
        let mut proof = (**f).clone();
        proof.cursor = 0;
        proof.guard = GuardState::Unchecked;
        self.validate_pack_a_frame(&proof)?;
        let mut ids = BTreeSet::new();
        for w in &c.waits {
            self.pregrave_source(&w.source)?;
            if !ids.insert(&w.source.card.id) {
                return Err("原调用方重复等待同次死亡".into());
            }
            if let Some(result) = &w.destination {
                self.validate_destination(&w.source, result, None)?;
            } else if self
                .death_replacements
                .iter()
                .find(|d| d.source.card.id == w.source.card.id)
                .is_none_or(|d| !same(&d.source, &w.source))
            {
                return Err("原调用方等待了不存在或异源的死亡".into());
            }
        }
        match &c.kind {
            DeathCallerKind::Frame => {
                let canonical = definition(&f.source.card.definition)
                    .abilities
                    .iter()
                    .any(|a| {
                        a.key == f.ability_key
                            && same(
                                &self
                                    .make_frame(
                                        f.actor,
                                        f.source.clone(),
                                        a,
                                        f.targets.clone(),
                                        f.already_paid.clone(),
                                        f.chosen_region,
                                    )
                                    .steps,
                                &f.steps,
                            )
                            && a.targets.len() == f.targets.len()
                            && a.targets
                                .iter()
                                .zip(&f.targets)
                                .all(|(a, t)| same(a, &t.spec))
                    });
                if f.cursor >= f.steps.len() || !canonical {
                    return Err("死亡续帧须保留完整原程序及尚未执行的操作".into());
                }
            }
            DeathCallerKind::JZ24 { remaining_players } => {
                if f.source.card.definition != "JZ24"
                    || f.ability_key != "local-enemy-sacrifice"
                    || f.cursor != 1
                    || f.steps.len() != 1
                    || !matches!(f.steps[0].op, Op::JZ24LocalSacrificeSnapshot)
                    || !f.already_paid.is_empty()
                    || !f.targets.is_empty()
                    || remaining_players
                        .iter()
                        .any(|s| *s >= self.players.len() || self.team(*s) == self.team(f.actor))
                    || remaining_players.iter().collect::<BTreeSet<_>>().len()
                        != remaining_players.len()
                {
                    return Err("卡迪纳追迹人死亡后须续接原敌方名单".into());
                }
            }
            DeathCallerKind::JZ35 { returns } => {
                self.validate_jz35_frame(&proof)?;
                if f.cursor != 1
                    || returns
                        .iter()
                        .map(|r| &r.death.old_instance)
                        .collect::<BTreeSet<_>>()
                        .len()
                        != returns.len()
                {
                    return Err("JZ35死亡后返回名单重复".into());
                }
                for r in returns {
                    if r.death.owner >= self.players.len()
                        || r.region >= self.regions.len()
                        || !self.mystic_instance_valid(&r.region_instance)
                        || !self.mystic_instance_valid(&r.death.old_instance)
                        || !catalog::catalog()
                            .cards
                            .iter()
                            .any(|d| d.id == r.definition && d.kind == "character")
                    {
                        return Err("JZ35原返回实体、拥有者或地区无效".into());
                    }
                    if let Some(id) = &r.death.deferred_death {
                        if !r.death.grave_instance.is_empty()
                            || !c.waits.iter().any(|w| {
                                w.source.card.id == *id
                                    && w.destination.is_none()
                                    && w.source.card.owner == r.death.owner
                                    && w.source.card.definition == r.definition
                            })
                        {
                            return Err("JZ35不得以虚构墓地实例跳过待定死亡".into());
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn resume_death_caller(&mut self, c: DeathCaller) -> RuleResult<()> {
        self.validate_death_caller(&c)?;
        if c.waits.iter().any(|w| w.destination.is_none()) {
            return Err("原死亡去向未决，不能恢复调用方".into());
        }
        if self.players[c.frame.actor].eliminated {
            return Ok(());
        }
        match c.kind {
            DeathCallerKind::Frame => self.resolve_frame(*c.frame),
            DeathCallerKind::JZ24 { remaining_players } => {
                self.jz24_next(*c.frame, remaining_players)
            }
            DeathCallerKind::JZ35 { returns } => self.jz35_return_hosts(&returns),
        }
    }
    pub(crate) fn validate_death_continuations(&self) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        let mut held = BTreeSet::new();
        let mut callers = BTreeSet::new();
        for d in &self.death_replacements {
            self.pregrave_source(&d.source)?;
            if self.status!="playing"||d.turn!=self.turn||!ids.insert(&d.source.card.id)||!held.insert(&d.transition_instance)||!self.mystic_instance_number(&d.source.card.id).zip(self.mystic_instance_number(&d.transition_instance)).is_some_and(|(a,b)|a<b)
                ||d.baseline_stack.iter().collect::<BTreeSet<_>>().len()!=d.baseline_stack.len()||d.baseline_stack.iter().any(|id|!self.mystic_instance_number(id).zip(self.mystic_instance_number(d.declaration_choice.as_ref().unwrap_or(&d.transition_instance))).is_some_and(|(a,b)|a<b))
                ||self.effects.iter().filter(|e|matches!(e,Effect::DeathFinish{death_instance} if *death_instance==d.source.card.id)).count()!=1{return Err("待定死亡来源、原堆栈边界或唯一收束续体无效".into());}
            if d.declaration_choice.as_deref().is_some_and(|choice| {
                !self
                    .mystic_instance_number(&d.transition_instance)
                    .zip(self.mystic_instance_number(choice))
                    .is_some_and(|(held, choice)| held < choice)
            }) {
                return Err("死亡原目标待选必须晚于实际离场换号".into());
            }
            match (&d.card, &d.destination) {
                (Some(c), None) => {
                    let mut expected = d.source.card.clone();
                    expected.id = d.transition_instance.clone();
                    expected.controller = expected.owner;
                    expected.face_down = false;
                    expected.exhausted = false;
                    expected.damage = 0;
                    expected.wounds = 0;
                    expected.shield = 0;
                    expected.lock_markers = 0;
                    expected.time_markers = 0;
                    expected.charge_markers = 0;
                    expected.advancement_markers = 0;
                    expected.permanent_combat_markers = 0;
                    expected.permanent_defense_markers = 0;
                    expected.named_character = None;
                    expected.chosen_number = None;
                    expected.chosen_subtype = None;
                    if !same(c, &expected)
                        || self
                            .batch3_physical_cards()
                            .filter(|v| v.id == c.id)
                            .count()
                            != 1
                    {
                        return Err("待定死亡必须独占完整重置后的真实牌".into());
                    }
                }
                (None, Some(result)) => {
                    self.validate_destination(&d.source, result, Some(&d.transition_instance))?
                }
                _ => return Err("死亡牌与最终去向必须且只能处于一个阶段".into()),
            }
            let mut live = 0;
            for e in &self.effects {
                if let Effect::Declare { declaration } = e {
                    if declaration.ability.key == crate::next20_black_wraiths::SEAL
                        && declaration.source.card.id == d.source.card.id
                    {
                        self.validate_xg25_death_source(declaration.actor, &declaration.source)?;
                        if d.declaration_choice.is_some() {
                            return Err("死亡不能重放已经提供的原声明".into());
                        }
                        live += 1;
                    }
                }
            }
            for item in &self.stack {
                if let Some(f) = &item.frame {
                    if f.ability_key == crate::next20_black_wraiths::SEAL
                        && f.source.card.id == d.source.card.id
                    {
                        self.validate_xg25_death_frame(f)?;
                        live += 1;
                    }
                }
            }
            if let Some(p) = &self.pending {
                if let ChoiceResolution::Declare { declaration, .. } = &p.resolution {
                    if declaration.ability.key == crate::next20_black_wraiths::SEAL
                        && declaration.source.card.id == d.source.card.id
                    {
                        self.validate_xg25_death_source(declaration.actor, &declaration.source)?;
                        if d.declaration_choice.as_deref() != Some(&p.choice.id) {
                            return Err("死亡待选更换了原选择实例".into());
                        }
                        live += 1;
                    }
                }
            }
            if live > 1
                || d.destination.is_some() && live != 0
                || d.card.is_some()
                    && live == 0
                    && !self.death_boundary_waits(&d.source.card.id)
                    && !self.players[d.source.card.controller].eliminated
            {
                return Err("待定死亡不得复制、跳过或重放原触发响应".into());
            }
        }
        for (position, e) in self.effects.iter().enumerate() {
            match e{
            Effect::DeathFinish{death_instance} if !ids.contains(death_instance)=>return Err("死亡收束续体没有原待定离场".into()),
            Effect::DeathFinish{death_instance} if self.effects.iter().skip(position+1).any(|e| matches!(e,Effect::Declare{declaration:d} if d.ability.key == crate::next20_black_wraiths::SEAL && d.source.card.id == *death_instance)) => return Err("死亡去向不能越过尚未声明的原触发".into()),
            Effect::DeathCaller{caller}=>{self.validate_death_caller(caller)?;
                for w in &caller.waits { if w.destination.is_none() && self.effects.iter().position(|e| matches!(e,Effect::DeathFinish{death_instance} if *death_instance == w.source.card.id)).is_none_or(|i| i >= position) { return Err("原调用方不能越过未决死亡的原收束边界".into()); } }
                if !callers.insert(&caller.frame.frame_id)||self.stack.iter().any(|s|s.id==caller.frame.frame_id)||self.effects.iter().any(|e|matches!(e,Effect::Frame{frame} if frame.frame_id==caller.frame.frame_id)){return Err("死亡原调用方不能重复续帧或同时在堆栈执行".into());}},_=>(),
        }
        }
        Ok(())
    }
    pub(crate) fn cancel_pregrave_owner(&mut self, owner: usize) {
        for d in &mut self.death_replacements {
            if d.source.card.owner == owner {
                d.card = None;
                d.destination = Some(DeathDestination::Removed);
            }
        }
        self.stack.retain(|s| {
            s.frame.as_ref().is_none_or(|f| {
                f.ability_key != crate::next20_black_wraiths::SEAL || f.source.card.owner != owner
            })
        });
        self.effects.retain(|e|!matches!(e,Effect::Declare{declaration:d} if d.ability.key==crate::next20_black_wraiths::SEAL&&d.source.card.owner==owner));
        if self.pending.as_ref().is_some_and(|p|matches!(&p.resolution,ChoiceResolution::Declare{declaration:d,..} if d.ability.key==crate::next20_black_wraiths::SEAL&&d.source.card.owner==owner)){self.pending=None;}
    }
}
