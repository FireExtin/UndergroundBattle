//! BQ041 freezes a successful Blood attachment to a personal character in its
//! region. The targeted enemy, rather than the observer, pays or discards.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "batch4-bq041-blood-demand";
const PAY_TITLE: &str = "欢宴：支付2，否则弃一张手牌";
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BloodAttachment { pub event_id: String, pub turn: u32, pub attachment: Attachment, pub host: SourceSnapshot }

pub(crate) fn definition() -> Definition {
    let mut d = crate::rules::attachment_definition(target(Zone::Region, EntityKind::Any, Relation::Any, Range::Anywhere));
    d.traits.retreat = true;
    d.abilities.push(ability(KEY, "鲜血结附本地区本方角色：目标敌方支付2或弃1", Timing::Fast, vec![],
        vec![target(Zone::Player, EntityKind::Any, Relation::EnemyTeam, Range::Anywhere)],
        vec![Op::BQ041BloodDemand { observation: None }], Some(Event::BQ041BloodAttached)));
    d
}
pub(crate) fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition().abilities.into_iter().find(|b| b.key == a.key)?;
    if let [Op::BQ041BloodDemand { observation }] = a.ops.as_slice() {
        if a.key == KEY { expected.ops = vec![Op::BQ041BloodDemand { observation: observation.clone() }]; }
    }
    Some(expected)
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    if a.kind != "play" || a.ability_id.as_deref().is_some_and(|k| k != "attach")
        || *a != (Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), region: a.region, ..Action::new("play") }) {
        return Err("欢宴只可按原地区目标打出结附".into());
    }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.source.card.definition == "BQ041")
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
        return Err("欢宴声明只能提交原目标选择，不能附带费用或其他载荷".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn bq041_observe_attachment(&mut self, id: &str) -> usize {
        let Some(blood) = self.attachments.iter().find(|a| a.card.id == id && !a.card.face_down
            && catalog::card(&a.card.definition).name == "鲜血" && self.attachment_host_valid(a)).cloned() else { return 0; };
        let Some((Some(r), host)) = self.in_play(&blood.host_id).filter(|(_, c)| !c.face_down && catalog::card(&c.definition).kind == "character") else { return 0; };
        let actor = host.controller;
        if self.players[actor].eliminated { return 0; }
        let mut host = self.source_snapshot(host, Some(r));
        host.source_region_instance = Some(self.regions[r].card.id.clone());
        let sources = self.attachments.iter().filter(|a| a.card.definition == "BQ041" && !a.card.face_down
            && a.card.controller == actor && self.attachment_host_valid(a) && a.host_id == self.regions[r].card.id)
            .map(|a| self.source_snapshot(&a.card, Some(r))).collect::<Vec<_>>();
        let count = sources.len();
        if count == 0 { return 0; }
        let observation = BloodAttachment { event_id: self.id(), turn: self.turn, attachment: blood, host };
        for source in sources {
            let mut a = definition().abilities.remove(1);
            a.ops = vec![Op::BQ041BloodDemand { observation: Some(Box::new(observation.clone())) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor, source, ability: a } });
        }
        count
    }
    fn bq041_instance(&self, id: &str) -> Option<u64> {
        id.strip_prefix('i').and_then(|s| s.parse::<u64>().ok()).filter(|n| *n > 0 && *n <= self.sequence && id == format!("i{n}"))
    }
    fn bq041_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != "BQ041" || self.bq041_instance(&c.id).is_none() || c.face_down
            || actor >= self.players.len() || c.controller != actor || c.owner >= self.players.len()
            || s.play_source.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || s.region.is_some_and(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_some_and(|id| self.bq041_instance(id).is_none() || *id == c.id)
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r,id)| self.regions.iter().enumerate().any(|(other,live)| live.card.id == *id && r != other)) {
            return Err("欢宴来源与冻结地区身份无效".into());
        }
        self.mobility_immutable_identity(c)
    }
    fn bq041_observation<'a>(&self, actor: usize, s: &SourceSnapshot, ops: &'a [Op]) -> RuleResult<&'a BloodAttachment> {
        let [Op::BQ041BloodDemand { observation: Some(o) }] = ops else { return Err("欢宴缺少实际鲜血结附观察".into()); };
        let n = self.bq041_instance(&o.event_id).ok_or("欢宴事件身份无效")?;
        let a = &o.attachment; let h = &o.host; let uid = s.source_region_instance.as_ref().ok_or("欢宴缺少原地区")?;
        let blood = catalog::catalog().cards.iter().find(|d| d.id == a.card.definition);
        let host = catalog::catalog().cards.iter().find(|d| d.id == h.card.definition);
        if o.turn != self.turn || self.batch3_physical_cards().any(|c| c.id == o.event_id)
            || s.region.is_none() || s.attachment_host_instance.as_ref() != Some(uid)
            || blood.is_none_or(|d| !catalog::runtime_implemented(d) || d.name != "鲜血" || d.kind != "attachment")
            || a.card.face_down || a.card.owner >= self.players.len() || a.card.controller >= self.players.len()
            || h.card.face_down || host.is_none_or(|d| !catalog::runtime_implemented(d) || d.kind != "character") || h.card.controller != actor || h.card.owner >= self.players.len()
            || h.region != s.region || h.source_region_instance.as_ref() != Some(uid) || a.host_id != h.card.id
            || h.play_source.is_some() || h.observed_death.is_some() || h.observed_exposure.is_some() || h.observed_mobility_grant.is_some()
            || [s.card.id.as_str(), a.card.id.as_str(), h.card.id.as_str(), uid.as_str()].iter().any(|id| self.bq041_instance(id).is_none_or(|v| v >= n))
            || [s.card.id.as_str(), a.card.id.as_str(), h.card.id.as_str(), uid.as_str()].iter().collect::<BTreeSet<_>>().len() != 4 {
            return Err("欢宴必须保留同原地区本方角色与实际鲜血附属事件".into());
        }
        self.mobility_immutable_identity(&a.card)?; self.mobility_immutable_identity(&h.card)?;
        for (c, attachment) in [(&s.card, true), (&a.card, true), (&h.card, false)] {
            if self.batch3_physical_cards().any(|live| live.id == c.id)
                && !(if attachment { self.attachments.iter().any(|a| a.card.id == c.id) } else { self.in_play(&c.id).is_some() }) {
                return Err("欢宴历史来源不能由私有区同UID替代".into());
            }
        }
        Ok(o)
    }
    pub(crate) fn validate_bq041_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.bq041_source(d.actor, &d.source)?;
        if d.ability.key != KEY || canonical(&d.ability).is_none_or(|a| !same(&a, &d.ability)) { return Err("欢宴触发完整程序被改写".into()); }
        self.bq041_observation(d.actor, &d.source, &d.ability.ops)?; Ok(())
    }
    pub(crate) fn validate_bq041_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.bq041_source(f.actor, &f.source)?;
        if self.bq041_instance(&f.frame_id).is_none() || f.frame_id == f.source.card.id || f.targets.len() != 1
            || f.cursor > f.steps.len() || (f.cursor > 0 && !matches!(f.guard, GuardState::Accepted))
            || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
                && !(f.ability_key == "attach" && f.steps.is_empty() && matches!(f.guard, GuardState::Accepted | GuardState::Cancelled))) {
            return Err("欢宴帧、游标或目标数量无效".into());
        }
        let t = &f.targets[0];
        if t.public.instance_id != t.id || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some() { return Err("欢宴冻结目标摘要无效".into()); }
        if f.ability_key == "attach" {
            let r = t.id.strip_prefix("region:").and_then(|s| s.parse::<usize>().ok()).filter(|r| *r < self.regions.len()).ok_or("欢宴地区目标无效")?;
            if !f.steps.is_empty() || !same(&t.spec, &definition().abilities[0].targets[0]) || t.id != format!("region:{r}")
                || f.chosen_region != Some(r) || t.public.kind != "region" || t.public.region != Some(r)
                || t.region_instance.as_ref().is_none_or(|id| self.bq041_instance(id).is_none())
                || f.source.region.is_some() || f.source.source_region_instance.is_some() || f.source.attachment_host_instance.is_some()
                || self.batch3_physical_cards().any(|c| c.id == f.source.card.id)
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= 2 && ids.iter().all(|id| !id.is_empty() && *id != f.source.card.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("欢宴结附须保留原地区、支付与空程序".into());
            }
        } else {
            let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>();
            let mut a = definition().abilities.remove(1); a.ops = ops;
            self.validate_bq041_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a.clone() })?;
            let seat = t.id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok()).filter(|s| *s < self.players.len()).ok_or("欢宴敌方目标无效")?;
            if f.ability_key != KEY || !f.already_paid.is_empty() || f.chosen_region.is_some() || f.steps.iter().any(|s| s.context != f.actor)
                || !same(&t.spec, &a.targets[0]) || t.id != player_id(seat) || self.team(seat) == self.team(f.actor)
                || t.region_instance.is_some() || t.public.kind != "player" || t.public.region.is_some() || t.public.owner.is_some()
                || t.public.controller.as_deref() != Some(&t.id) || t.public.label != self.players[seat].name {
                return Err("欢宴只能保留原敌方玩家及完整鲜血触发程序".into());
            }
        }
        Ok(())
    }
    fn bq041_enemy(&self, f: &ResolutionFrame) -> RuleResult<usize> {
        self.validate_bq041_frame(f)?;
        if f.ability_key != KEY || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) { return Err("欢宴支付只可来自原触发结算".into()); }
        f.targets[0].id.strip_prefix('p').and_then(|s| s.parse().ok()).ok_or("欢宴目标不存在".into())
    }
    fn bq041_pay_options(&self, seat: usize) -> Vec<ChoiceOption> {
        let mut options = vec![];
        if self.resources(seat) >= 2 { options.push(ChoiceOption { id: "pay".into(), label: "支付2".into(), card: None }); }
        options.push(ChoiceOption { id: "decline".into(), label: "不支付，弃一张手牌".into(), card: None }); options
    }
    fn bq041_discard_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let seat = self.bq041_enemy(&f)?;
        if !self.players[seat].eliminated && !self.players[seat].hand.is_empty() {
            let options = self.players[seat].hand.iter().map(|c| self.option(c, seat, None, None)).collect();
            self.choice(seat, "discard", "弃 1 张手牌".into(), options, 1, 1, None,
                ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::Discard { seat, redraw: false } });
        }
        Ok(())
    }
    pub(crate) fn bq041_demand(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let seat = self.bq041_enemy(&f)?;
        if self.players[seat].eliminated { return Ok(()); }
        if self.resources(seat) < 2 { return self.bq041_discard_start(f); }
        self.choice(seat, "bq041-payment", PAY_TITLE.into(), self.bq041_pay_options(seat), 1, 1, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::BQ041Payment }); Ok(())
    }
    pub(crate) fn bq041_choice_payload(&self, f: &ResolutionFrame, chooser: usize, a: &Action) -> RuleResult<()> {
        if f.source.card.definition != "BQ041" { return Ok(()); }
        if chooser != self.bq041_enemy(f)? || *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
            return Err("欢宴支付或私有弃牌须由原敌方玩家提交原选项".into());
        }
        Ok(())
    }
    pub(crate) fn bq041_pay_complete(&mut self, f: ResolutionFrame, chooser: usize, a: &Action, selected: &[String]) -> RuleResult<()> {
        self.bq041_choice_payload(&f, chooser, a)?;
        if selected.len() != 1 || !self.bq041_pay_options(chooser).iter().any(|o| o.id == selected[0]) { return Err("欢宴支付选项无效".into()); }
        if selected[0] == "pay" { self.pay(chooser, 2) } else { self.bq041_discard_start(f) }
    }
    pub(crate) fn validate_bq041_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new(); let mut events = BTreeSet::new();
        let mut remember = |s: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            if let [Op::BQ041BloodDemand { observation: Some(o) }] = ops {
                if !events.insert((s.card.id.clone(), o.event_id.clone())) { return Err("同一欢宴不能重复消费同次鲜血结附".into()); }
            }
            Ok(())
        };
        for item in &self.stack { if let Some(f) = &item.frame { if f.source.card.definition == "BQ041" {
            self.validate_bq041_frame(f)?; remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
            if !frames.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                || item.card.is_some() != (f.ability_key == "attach") || item.reveal || item.deploy_region.is_some() || item.target.as_ref() != Some(&f.targets[0].id)
                || item.card.as_ref().is_some_and(|c| c.definition != "BQ041" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor) {
                return Err("欢宴响应堆叠的来源、目标或帧重复".into());
            }
        } } }
        for e in &self.effects { match e {
            Effect::Declare { declaration: d } if d.source.card.definition == "BQ041" => { self.validate_bq041_declaration(d)?; remember(&d.source, &d.ability.ops)?; },
            Effect::Frame { frame: f } if f.source.card.definition == "BQ041" => return Err("欢宴不能保存跳过响应的续体".into()),
            _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration: d, stage } if d.source.card.definition == "BQ041" => {
                self.validate_bq041_declaration(d)?; remember(&d.source, &d.ability.ops)?;
                let options = self.binding_options(d.actor, &d.source, &d.ability.targets[0]);
                let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(), title: format!("{}：{}", catalog::card("BQ041").name, d.ability.label),
                    description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor), options, preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                if !matches!(stage, DeclareChoice::Target) || p.seat != d.actor || p.choice.id.is_empty() || expected.options.is_empty() || !same(&p.choice, &expected) { return Err("欢宴敌方目标声明被改写".into()); }
            },
            ChoiceResolution::Frame { frame: f, choice } if f.source.card.definition == "BQ041" => {
                let seat = self.bq041_enemy(f)?; remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
                if !frames.insert(f.frame_id.clone()) || p.seat != seat || p.choice.id.is_empty() || self.players[seat].eliminated { return Err("欢宴结算待选玩家或帧重复".into()); }
                let (kind, title, options) = match choice {
                    FrameChoice::BQ041Payment if self.resources(seat) >= 2 => ("bq041-payment", PAY_TITLE, self.bq041_pay_options(seat)),
                    FrameChoice::Discard { seat: target, redraw: false } if *target == seat => ("discard", "弃 1 张手牌", self.players[seat].hand.iter().map(|c| self.option(c, seat, None, None)).collect()),
                    _ => return Err("欢宴只准入原支付或私有弃牌阶段".into()),
                };
                let expected = Choice { id: p.choice.id.clone(), kind: kind.into(), title: title.into(), description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(seat), options,
                    preview_cards: vec![], min: Some(1), max: Some(1), amount: None, allow_decline: Some(false) };
                if expected.options.is_empty() || !same(&p.choice, &expected) { return Err("欢宴支付或弃牌的原选项被改写".into()); }
            },
            ChoiceResolution::Frame { choice: FrameChoice::BQ041Payment, .. } => return Err("欢宴支付不能移植来源".into()),
            _ => {},
        } }
        Ok(())
    }
}
