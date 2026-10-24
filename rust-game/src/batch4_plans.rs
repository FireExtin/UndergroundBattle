//! Printed Plan rules: personal Society host, conditional secret play, standard
//! unlimited paid advancement, and forced threshold reveal/sacrifice/effect.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const CARDS: &[&str] = &["WM045", "WM046", "WM047", "XG21"];
pub(crate) const ADVANCE: &str = "batch4-plan-advance";
const COMPLETE: &str = "batch4-plan-complete";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretPlay { pub turn: u32, pub controller: usize, pub concealment: u32 }
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn is_plan(c: &Card) -> bool { CARDS.contains(&c.definition.as_str()) }
pub(crate) fn secret(c: &Card) -> bool { c.face_down && is_plan(c) }
pub(crate) fn threshold(id: &str) -> Option<u32> { match id { "WM045" => Some(3), "WM046" => Some(2), "WM047" => Some(5), _ => None } }
pub(crate) fn label(c: &Card) -> &str { if secret(c) { "秘密计划" } else { &catalog::card(&c.definition).name } }
pub(crate) fn definition(id: &str) -> Option<Definition> {
    if !CARDS.contains(&id) { return None; }
    let mut d = crate::rules::attachment_definition(target(Zone::Society, EntityKind::Any, Relation::ControlledByActor, Range::Anywhere));
    d.abilities[0].ops = vec![Op::PlanPlayed { secrecy: None }];
    let mut advance = ability(ADVANCE, "推进1", Timing::Standard, vec![Cost::Assets(1)], vec![], vec![Op::PlanAdvance], None);
    advance.activation_only = true;
    d.abilities.push(advance);
    if id == "XG21" {
        let mut search = ability(crate::batch4_phantom::KEY, "牺牲本计划：检索费用小于推进数的声望角色置于地区一并洗牌", Timing::Standard,
            vec![Cost::SacrificeSource], vec![], vec![Op::XG21RenownSearch], None);
        search.activation_only = true;
        d.abilities.push(search);
    } else {
        d.abilities.push(ability(COMPLETE, "达成推进：展示并牺牲计划，发动效果", Timing::Fast, vec![], vec![], vec![Op::PlanComplete], Some(Event::PlanReached)));
    }
    Some(d)
}
pub(crate) fn canonical(id: &str, a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition(id)?.abilities.into_iter().find(|b| b.key == a.key)?;
    if a.key == "attach" { if let [Op::PlanPlayed { secrecy }] = a.ops.as_slice() { expected.ops = vec![Op::PlanPlayed { secrecy: secrecy.clone() }]; } }
    Some(expected)
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "play" => {
            if a.option.as_deref().is_some_and(|v| v != "secret-plan") { return Err("计划只有正面或秘密打出方式".into()); }
            if a.ability_id.as_deref().is_some_and(|v| v != "attach") { return Err("计划打出须使用完整结附程序".into()); }
            Action { card_id: a.card_id.clone(), target_id: a.target_id.clone(), option: a.option.clone(), ability_id: a.ability_id.clone(), ..Action::new("play") }
        },
        "activate" if matches!(a.ability_id.as_deref(), Some(ADVANCE | crate::batch4_phantom::KEY)) => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), ..Action::new("activate") },
        _ => return Err("计划只有结附和可推进标准能力".into()),
    };
    if *a != expected { return Err("计划操作不能带额外目标、支付或私有载荷".into()); } Ok(())
}
impl Game {
    pub(crate) fn plan_play_spec(&self, actor: usize, c: &mut Card, a: &Action, spec: &mut AbilitySpec) -> RuleResult<()> {
        if !is_plan(c) { return Ok(()); }
        if a.option.as_deref() == Some("secret-plan") {
            let concealment = self.players[actor].society_zone.secrecy;
            if concealment == 0 { return Err("秘密计划需要本人秘社上的隐秘标志".into()); }
            c.face_down = true;
            spec.ops = vec![Op::PlanPlayed { secrecy: Some(SecretPlay { turn: self.turn, controller: actor, concealment }) }];
        }
        Ok(())
    }
    fn validate_plan_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if !is_plan(c) || c.id.is_empty() || actor >= self.players.len() || c.owner >= self.players.len() || c.controller != actor
            || s.region.is_some() || s.source_region_instance.is_some() || s.play_source.is_some()
            || s.death_echo.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
        { return Err("计划来源必须保留原私人秘社区与完整身份".into()); }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_plan_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.validate_plan_source(d.actor, &d.source)?;
        let c = &d.source.card;
        let limit = threshold(&c.definition).ok_or("无完成阈值的计划没有强制完成声明")?;
        let expected = definition(&c.definition).unwrap().abilities.pop().unwrap();
        if c.face_down || c.advancement_markers < limit || !same(&d.ability, &expected)
            || d.source.attachment_host_instance.as_deref() != Some(format!("society:p{}", d.actor).as_str())
        { return Err("计划强制触发须保留实际达成阈值后的公开快照".into()); } Ok(())
    }
    pub(crate) fn validate_plan_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_plan_source(f.actor, &f.source)?;
        let c = &f.source.card;
        let mut a = definition(&c.definition).unwrap().abilities.into_iter().find(|a| a.key == f.ability_key).ok_or("未知计划程序")?;
        if f.ability_key == "attach" { if let [Step { op: Op::PlanPlayed { secrecy }, .. }] = f.steps.as_slice() { a.ops = vec![Op::PlanPlayed { secrecy: secrecy.clone() }]; } }
        let expected = a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect::<Vec<_>>();
        if f.frame_id.is_empty() || f.frame_id == c.id || f.chosen_region.is_some() || !same(&expected, &f.steps) || f.cursor > f.steps.len()
            || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked) && !(f.ability_key == "attach" && matches!(f.guard, GuardState::Accepted | GuardState::Cancelled)))
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)
            || f.targets.len() != a.targets.len() || f.targets.iter().zip(&a.targets).any(|(t, s)| !same(&t.spec, s))
        { return Err("计划帧、程序、游标、守卫或目标不完整".into()); }
        let paid = |n: usize, allow_reduction: bool| matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if (f.money_units(0) == n || allow_reduction && f.money_units(0) < n) && ids.iter().all(|id| !id.is_empty() && *id != c.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len());
        if f.ability_key == "attach" {
            if c.advancement_markers != 0 || f.source.attachment_host_instance.is_some() || !paid(1, true) { return Err("计划打出来源或费用不符".into()); }
            let [Op::PlanPlayed { secrecy }] = a.ops.as_slice() else { return Err("计划缺少完整打出程序".into()); };
            if c.face_down != secrecy.is_some() || secrecy.as_ref().is_some_and(|p| p.turn != self.turn || p.controller != f.actor || p.concealment == 0) { return Err("秘密计划缺少本人隐秘条件的实际打出收据".into()); }
            let t = &f.targets[0];
            if t.id != format!("society:p{}", f.actor) || t.region_instance.is_some() || t.public.instance_id != t.id || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some() || t.public.region.is_some() { return Err("计划只能结附本人稳定秘社区".into()); }
        } else {
            if f.source.attachment_host_instance.as_deref() != Some(format!("society:p{}", f.actor).as_str()) { return Err("计划推进与完成必须保留原秘社区".into()); }
            if f.ability_key == ADVANCE {
                if threshold(&c.definition).is_some_and(|limit| c.advancement_markers >= limit) || !paid(1, false) { return Err("推进必须只支付一次1且未达阈值".into()); }
            } else if f.ability_key == crate::batch4_phantom::KEY {
                if c.definition != "XG21" || c.face_down
                    || !matches!(f.already_paid.as_slice(), [PaidCost::Sacrificed { old_instance, controller, owner }]
                        if *old_instance == c.id && *controller == f.actor && *owner == c.owner)
                    || self.batch3_physical_cards().any(|live| live.id == c.id)
                { return Err("幻影英雄须保留正面原计划与一次真实自身牺牲费用".into()); }
            } else {
                if !f.already_paid.is_empty() { return Err("强制完成计划没有额外支付".into()); }
                self.validate_plan_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a })?;
            }
        }
        if self.batch3_physical_cards().any(|live| live.id == c.id) && (f.ability_key == "attach" || !self.attachments.iter().any(|p| p.card.id == c.id)) { return Err("计划不能由私有区同UID冒充原来源".into()); }
        Ok(())
    }
    pub(crate) fn plan_forced_declare(&mut self, d: &Declaration) -> RuleResult<bool> {
        if d.ability.event != Some(Event::PlanReached) { return Ok(false); }
        self.validate_plan_declaration(d)?;
        if !self.players[d.actor].eliminated {
            let f = self.make_frame(d.actor, d.source.clone(), &d.ability, vec![], vec![], None);
            self.dispatch_frame(f, format!("{}：达成推进", label(&d.source.card)), None, ResponsePolicy::Respondable);
        } Ok(true)
    }
    pub(crate) fn plan_advance(&mut self, f: &ResolutionFrame) {
        if self.attachments.iter().any(|a| a.card.id == f.source.card.id && a.card.controller == f.actor && Some(&a.host_id) == f.source.attachment_host_instance.as_ref()) { self.advance_plan_instance(f.actor, &f.source.card.id); }
    }
    pub(crate) fn advance_plan_instance(&mut self, actor: usize, id: &str) {
        let Some(index) = self.attachments.iter().position(|a| a.card.id == id && a.card.controller == actor && a.host_id == format!("society:p{actor}") && is_plan(&a.card)) else { return; };
        let before = self.attachments[index].card.advancement_markers;
        self.attachments[index].card.advancement_markers = before.saturating_add(1);
        self.observe_plan_advancement(id, before);
    }
    /// Project ruling 2026-10-22: one atomic 1 -> 3 move crosses Tianyan's
    /// threshold once. Retain its actual counter value in the event snapshot.
    pub(crate) fn observe_plan_advancement(&mut self, id: &str, before: u32) {
        let Some(index) = self.attachments.iter().position(|a| a.card.id == id && is_plan(&a.card)
            && a.host_id == format!("society:p{}", a.card.controller)) else { return; };
        let Some(limit) = threshold(&self.attachments[index].card.definition) else { return; };
        let queued = self.stack.iter().any(|i| i.frame.as_ref().is_some_and(|f| f.source.card.id == id && f.ability_key == COMPLETE))
            || self.effects.iter().any(|e| matches!(e, Effect::Declare { declaration: d } if d.source.card.id == id && d.ability.key == COMPLETE));
        if before < limit && self.attachments[index].card.advancement_markers >= limit && !queued {
            self.attachments[index].card.face_down = false;
            let source = self.source_snapshot(&self.attachments[index].card, None);
            self.note(format!("{} 达成推进{}，展示计划", label(&source.card), limit));
            self.emit_event(source.card.controller, source, Event::PlanReached);
        }
    }
    pub(crate) fn plan_complete(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.remove_dead(&f.source.card.id, RemovalCause::Sacrifice);
        match f.source.card.definition.as_str() {
            "WM045" => self.world_hide(None, true),
            "WM046" => {
                let n = self.in_play_cards().into_iter().filter(|(_, c)| !is_plan(c) && self.effective_marker_count(c, crate::batch4_markers::MarkerKind::Lock) > 0 && (c.face_down || catalog::card(&c.definition).kind == "character")).count();
                self.draw(f.actor, n)?;
            },
            "WM047" => {
                let deaths = self.in_play_cards().into_iter().filter(|(_, c)| !c.face_down && catalog::card(&c.definition).kind == "character" && self.current_magic_count(c) > 0)
                    .map(|(r, c)| (c.id.clone(), RemovalCause::Destroy, self.source_snapshot(c, r))).collect();
                self.destroy_wave_as(f.actor, deaths);
            },
            _ => return Err("非本批固定计划完成程序".into()),
        } Ok(())
    }
    pub(crate) fn validate_plan_state(&self) -> RuleResult<()> {
        for c in self.batch3_physical_cards().filter(|c| c.advancement_markers > 0) {
            if !self.marker_move_carrier(&c.id) { return Err("推进标志只能属于有效在场角色或附属".into()); }
        }
        let mut frames = BTreeSet::new(); let mut completed = BTreeSet::new();
        for a in self.attachments.iter().filter(|a| is_plan(&a.card)) {
            if a.host_id != format!("society:p{}", a.card.controller) || !self.attachment_host_valid(a) { return Err("计划失去本人稳定秘社区宿主".into()); }
        }
        let mut completion = |s: &SourceSnapshot, key: &str| -> RuleResult<()> { if key == COMPLETE && !completed.insert(s.card.id.clone()) { return Err("同一计划达成阈值不能重复完成".into()); } Ok(()) };
        for item in &self.stack { if let Some(f) = &item.frame { if is_plan(&f.source.card) {
            self.validate_plan_frame(f)?;
            if !frames.insert(&f.frame_id) || item.id != f.frame_id || item.controller != f.actor || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                || item.card.is_some() != (f.ability_key == "attach") || item.reveal || item.deploy_region.is_some() || item.target != f.targets.first().map(|t| t.id.clone())
                || item.card.as_ref().is_some_and(|c| c.definition != f.source.card.definition || c.face_down != f.source.card.face_down || c.owner != f.source.card.owner || c.controller != f.actor || c.advancement_markers != 0)
            { return Err("计划响应外壳、秘密朝向或实例不符".into()); }
            completion(&f.source, &f.ability_key)?;
        } } }
        for effect in &self.effects { if let Effect::Declare { declaration: d } = effect { if is_plan(&d.source.card) { self.validate_plan_declaration(d)?; completion(&d.source, &d.ability.key)?; } } }
        if self.pending.as_ref().is_some_and(|p| matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if is_plan(&d.source.card))) { return Err("计划达成推进是强制触发，不能改成可选选择".into()); }
        for a in self.attachments.iter().filter(|a| is_plan(&a.card) && threshold(&a.card.definition).is_some_and(|limit| a.card.advancement_markers >= limit)) {
            if a.card.face_down || !completed.contains(&a.card.id) { return Err("已达阈值计划必须公开并保留唯一强制完成程序".into()); }
        }
        self.validate_phantom_search_state()?;
        Ok(())
    }
}
