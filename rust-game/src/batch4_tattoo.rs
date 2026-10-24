//! JZ39: a human host gains combat, kill and Demon; payment is frozen before
//! sacrificing its own host can remove the tattoo. The enemy chooses a victim.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "batch4-jz39-demon-sacrifice";
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DemonPayment {
    pub card: Card,
    pub subtypes: Vec<String>,
    pub tattoos: Vec<Attachment>,
}
pub(crate) fn definition() -> Definition {
    let mut host = target(Zone::Board, EntityKind::Character, Relation::Any, Range::Anywhere);
    host.subtype = Some("人类".into());
    let mut d = crate::rules::attachment_definition(host);
    d.attachment.as_mut().unwrap().host_icons = Icons { combat: 1, ..Icons::default() };
    d.modifiers.push(StaticModifier::JZ39HostKillAndDemon);
    let mut a = ability(KEY, "行动1、横置、牺牲本方恶魔：目标敌方玩家牺牲一个角色", Timing::Standard,
        vec![Cost::Assets(1), Cost::ExhaustSource, Cost::JZ39SacrificeDemon],
        vec![target(Zone::Player, EntityKind::Any, Relation::EnemyTeam, Range::Anywhere)],
        vec![Op::JZ39EnemySacrifice], None);
    a.activation_only = true; d.abilities.push(a); d
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "play" => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), ..Action::new("play") },
        "activate" => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), cost_selected: a.cost_selected.clone(), ..Action::new("activate") },
        _ => return Err("恶魔刺青只可打出结附或发动固定行动".into()),
    };
    let key = if a.kind == "play" { "attach" } else { KEY };
    if *a != expected || a.ability_id.as_deref().is_some_and(|id| id != key) {
        return Err("恶魔刺青行动不能夹带其他载荷或程序".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn jz39_host_kill(&self, c: &Card) -> u32 {
        self.attachments.iter().filter(|a| a.card.definition == "JZ39" && !a.card.face_down
            && a.host_id == c.id && self.attachment_host_valid(a)).count() as u32
    }
    pub(crate) fn jz39_demons(&self, actor: usize) -> Vec<String> {
        self.regions.iter().flat_map(|r| &r.cards).filter(|c| !c.face_down && c.controller == actor
            && catalog::card(&c.definition).kind == "character" && self.current_subtypes(c).iter().any(|s| s == "恶魔"))
            .map(|c| c.id.clone()).collect()
    }
    pub(crate) fn pay_jz39_demon(&mut self, actor: usize, a: &Action) -> RuleResult<PaidCost> {
        let ids = a.cost_selected.as_ref().filter(|ids| ids.len() == 1).ok_or("请选择一个本方恶魔角色支付")?;
        if !self.jz39_demons(actor).contains(&ids[0]) { return Err("恶魔牺牲费用必须来自本方当前明置恶魔".into()); }
        let c = self.board(&ids[0]).unwrap().1;
        let payment = DemonPayment { card: c.clone(), subtypes: self.current_subtypes(c),
            tattoos: self.attachments.iter().filter(|t| t.card.definition == "JZ39" && !t.card.face_down
                && t.host_id == c.id && self.attachment_host_valid(t)).cloned().collect() };
        self.remove_dead(&ids[0], RemovalCause::Sacrifice);
        Ok(PaidCost::JZ39Demon(Box::new(payment)))
    }
    fn validate_jz39_payment(&self, actor: usize, p: &DemonPayment) -> RuleResult<()> {
        let c = &p.card;
        let print = catalog::catalog().cards.iter().find(|d| d.id == c.definition && catalog::runtime_implemented(d)).ok_or("恶魔费用不是当前实现牌")?;
        let mut seen = BTreeSet::new();
        if c.id.is_empty() || c.face_down || c.controller != actor || c.owner >= self.players.len() || print.kind != "character"
            || !p.subtypes.iter().any(|s| s == "恶魔") || p.subtypes.iter().any(|s| s.is_empty() || !seen.insert(s))
            || self.batch3_physical_cards().any(|live| live.id == c.id)
            || (!print.subtypes.iter().any(|s| s == "恶魔") && (!print.subtypes.iter().any(|s| s == "人类") || !p.subtypes.iter().any(|s| s == "人类") || p.tattoos.is_empty()))
        { return Err("恶魔牺牲费用缺少实际身份、类别或消费记录".into()); }
        self.mobility_immutable_identity(c)?;
        let mut ids = BTreeSet::new();
        for t in &p.tattoos {
            if t.card.definition != "JZ39" || t.card.id.is_empty() || t.card.id == c.id || !ids.insert(&t.card.id)
                || t.card.face_down || t.card.owner >= self.players.len() || t.card.controller >= self.players.len() || t.host_id != c.id
                || !p.subtypes.iter().any(|s| s == "人类")
            { return Err("恶魔类别授予须保留真实人类宿主及刺青".into()); }
            self.mobility_immutable_identity(&t.card)?;
        }
        Ok(())
    }
    pub(crate) fn validate_jz39_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let s = &f.source; let c = &s.card;
        if c.definition != "JZ39" || c.id.is_empty() || c.face_down || c.owner >= self.players.len() || f.actor >= self.players.len() || c.controller != f.actor
            || s.play_source.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || s.region.is_some_and(|r| r >= self.regions.len()) || f.frame_id.is_empty() || f.frame_id == c.id || f.chosen_region.is_some()
            || f.cursor > f.steps.len()
            || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
                && !(f.ability_key == "attach" && f.steps.is_empty() && matches!(f.guard, GuardState::Accepted | GuardState::Cancelled)))
            || (f.cursor > 0 && !matches!(f.guard, GuardState::Accepted))
            || s.source_region_instance.as_ref().is_some_and(|id| id.is_empty() || *id == c.id)
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r,id)| self.regions.iter().enumerate().any(|(other,live)| live.card.id == *id && r != other))
        { return Err("恶魔刺青来源、玩家、帧或冻结地区无效".into()); }
        self.mobility_immutable_identity(c)?;
        if self.batch3_physical_cards().any(|live| live.id == c.id)
            && (f.ability_key == "attach" || !self.attachments.iter().any(|a| a.card.id == c.id)) {
            return Err("恶魔刺青冻结来源不能由手牌、资产或墓地中的同UID替代".into());
        }
        let d = definition(); let a = d.abilities.iter().find(|a| a.key == f.ability_key).ok_or("恶魔刺青未知程序")?;
        let steps = a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect::<Vec<_>>();
        if !same(&steps, &f.steps) || f.targets.len() != 1 || !same(&f.targets[0].spec, &a.targets[0]) {
            return Err("恶魔刺青必须保留完整程序、环境及目标".into());
        }
        let t = &f.targets[0];
        if t.id.is_empty() || t.region_instance.is_some() || t.public.instance_id != t.id || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some() {
            return Err("恶魔刺青目标摘要无效".into());
        }
        if f.ability_key == "attach" {
            if s.region.is_some() || s.source_region_instance.is_some() || s.attachment_host_instance.is_some()
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= 2 && ids.iter().all(|id| !id.is_empty() && *id != c.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
                || t.public.kind != "character" || t.public.region.is_none_or(|r| r >= self.regions.len()) || t.public.owner.is_none() || t.public.controller.is_none()
            { return Err("恶魔刺青结附帧的支付、来源或人类目标无效".into()); }
        } else {
            if s.region.is_none() || s.source_region_instance.is_none() || s.attachment_host_instance.as_ref().is_none_or(|id| id.is_empty() || *id == c.id) || c.exhausted {
                return Err("恶魔刺青发动须保留原宿主与横置前来源".into());
            }
            let [PaidCost::Assets(assets), PaidCost::Exhausted(id), PaidCost::JZ39Demon(payment)] = f.already_paid.as_slice() else { return Err("恶魔刺青费用必须依次为1、横置与恶魔牺牲".into()); };
            if assets.len() != 1 || assets[0].is_empty() || assets[0] == c.id || id != &c.id || assets[0] == payment.card.id {
                return Err("恶魔刺青费用不能重复消费或更换来源".into());
            }
            self.validate_jz39_payment(f.actor, payment)?;
            let seat = t.id.strip_prefix('p').and_then(|p| p.parse::<usize>().ok()).filter(|p| *p < self.players.len()).ok_or("目标敌方玩家不存在")?;
            if t.id != player_id(seat) || self.team(seat) == self.team(f.actor) || t.public.kind != "player" || t.public.region.is_some()
                || t.public.owner.is_some() || t.public.controller.as_deref() != Some(&t.id) || t.public.label != self.players[seat].name {
                return Err("恶魔刺青只能绑定原敌方玩家".into());
            }
        }
        Ok(())
    }
    fn jz39_enemy(&self, f: &ResolutionFrame) -> RuleResult<usize> {
        self.validate_jz39_frame(f)?;
        if f.ability_key != KEY { return Err("恶魔刺青牺牲续体来源不符".into()); }
        f.targets[0].id.strip_prefix('p').and_then(|p| p.parse().ok()).ok_or("敌方玩家无效".into())
    }
    fn jz39_victims(&self, seat: usize) -> Vec<ChoiceOption> {
        self.in_play_cards().into_iter().filter(|(_,c)| !c.face_down && c.controller == seat && catalog::card(&c.definition).kind == "character")
            .map(|(r,c)| self.option(c,seat,r,None)).collect()
    }
    pub(crate) fn jz39_enemy_sacrifice(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let seat = self.jz39_enemy(&f)?; let options = self.jz39_victims(seat);
        if !options.is_empty() { self.choice(seat, "sacrifice", "恶魔刺青：牺牲一个角色".into(), options, 1, 1, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::JZ39EnemySacrifice }); }
        Ok(())
    }
    pub(crate) fn jz39_finish_sacrifice(&mut self, f: &ResolutionFrame, chooser: usize, a: &Action, selected: &[String]) -> RuleResult<()> {
        let seat = self.jz39_enemy(f)?;
        let expected = Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") };
        if *a != expected || chooser != seat || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted)
            || selected.len() != 1 || !self.jz39_victims(seat).iter().any(|o| o.id == selected[0]) {
            return Err("目标敌方玩家须自行牺牲一个当前角色".into());
        }
        self.remove_dead(&selected[0], RemovalCause::Sacrifice); Ok(())
    }
    pub(crate) fn validate_jz39_state(&self) -> RuleResult<()> {
        let mut ids = BTreeSet::new(); let mut payments = BTreeSet::new();
        let mut remember = |f: &ResolutionFrame| -> RuleResult<()> {
            self.validate_jz39_frame(f)?;
            if !ids.insert(f.frame_id.clone()) || f.already_paid.iter().any(|p| matches!(p, PaidCost::JZ39Demon(d) if !payments.insert(d.card.id.clone()))) {
                return Err("恶魔刺青不能重复堆叠帧或牺牲费用".into());
            } Ok(())
        };
        for item in &self.stack { if let Some(f) = &item.frame { if f.source.card.definition == "JZ39" {
            remember(f)?;
            if item.id != f.frame_id || item.controller != f.actor || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                || item.card.is_some() != (f.ability_key == "attach") || item.deploy_region.is_some() || item.reveal || item.target.as_ref() != Some(&f.targets[0].id)
                || item.card.as_ref().is_some_and(|c| c.face_down || c.definition != "JZ39" || c.owner != f.source.card.owner || c.controller != f.actor)
            { return Err("恶魔刺青响应堆叠外壳无效".into()); }
        } } }
        for e in &self.effects { if matches!(e, Effect::Frame { frame } if frame.source.card.definition == "JZ39") { return Err("恶魔刺青不能绕过响应堆叠".into()); } }
        if let Some(p) = &self.pending { if let ChoiceResolution::Frame { frame: f, choice } = &p.resolution {
            if matches!(choice, FrameChoice::JZ39EnemySacrifice) && f.source.card.definition != "JZ39" { return Err("恶魔刺青选择不能移植来源".into()); }
            if f.source.card.definition == "JZ39" {
            remember(f)?; let seat = self.jz39_enemy(f)?;
            let expected = Choice { id: p.choice.id.clone(), kind: "sacrifice".into(), title: "恶魔刺青：牺牲一个角色".into(),
                description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(seat), options: self.jz39_victims(seat),
                preview_cards: vec![], min: Some(1), max: Some(1), amount: None, allow_decline: Some(false) };
            if !matches!(choice, FrameChoice::JZ39EnemySacrifice) || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted)
                || p.seat != seat || p.choice.id.is_empty() || expected.options.is_empty() || !same(&p.choice,&expected) {
                return Err("恶魔刺青敌方牺牲选择被改写".into());
            }
        } } }
        Ok(())
    }
}
