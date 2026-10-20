//! Whole JC035: optional preparation reward, bound to the observed region instance.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "jc035-preparation-influence";
const ORDER_KIND: &str = "jc035-preparation-order";
const ORDER_TITLE: &str = "传媒魔咒：选择下一项准备触发";
const ORDER_DESCRIPTION: &str = "选择下一项发动并入栈；可放弃剩余触发。后入栈的先结算。";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparationHand {
    pub seat: usize,
    pub count: usize,
    pub eliminated: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PreparationDecision {
    Declined,
    Accepted { frame_id: String },
    Resolved { frame_id: String },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparationEntry {
    pub source: SourceSnapshot,
    pub decision: Option<PreparationDecision>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreparationObservation {
    pub turn: u32,
    pub first_team: usize,
    pub event_id: Option<String>,
    pub hands: Vec<PreparationHand>,
    pub entries: Vec<PreparationEntry>,
}

fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JC035PreparedInfluence { .. } => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted { exhausted, ready, .. } =>
            uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn own(a: &AbilitySpec) -> bool {
    a.key == KEY || a.event == Some(Event::Prepare) || uses(&a.ops)
        || a.modes.iter().any(|m| uses(&m.ops))
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "JC035" || f.ability_key == KEY
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
pub(crate) fn definition() -> Definition {
    let host = target(Zone::Region, EntityKind::Any, Relation::Any, Range::Anywhere);
    let mut attach = ability("attach", "结附目标地区", Timing::Standard,
        vec![], vec![host.clone()], vec![], None);
    attach.play_only = true;
    Definition {
        abilities: vec![attach, ability(KEY, "准备步骤触发：本地区放置1本方势力", Timing::Fast,
            vec![], vec![], vec![Op::JC035PreparedInfluence { event_id: None }], Some(Event::Prepare))],
        attachment: Some(AttachmentSpec {
            controls_host: false, host_subtype_change: SubtypeChange::None, host,
            host_icons: Icons::default(), host_temporary_icons: None, host_barrier: false,
            host_defense_bonus: 0, host_leaves: HostLeaveDestination::OwnerGraveyard,
        }),
        ..Default::default()
    }
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if !own(a) { return Ok(()); }
    let mut normalized = a.clone();
    match normalized.ops.as_mut_slice() {
        [Op::JC035PreparedInfluence { event_id }] => *event_id = None,
        _ => return Err("JC035只准入完整准备触发程序".into()),
    }
    if id != "JC035" || !same(&normalized, &definition().abilities[1]) {
        return Err("JC035准备触发不能改写、嵌套或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if id == "JC035" && !same(d, &definition())
        || id != "JC035" && d.abilities.iter().any(own) {
        return Err("JC035必须保留整张地区附属与可选准备能力".into());
    }
    Ok(())
}
pub(crate) fn validate_play_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "JC035" && (*a != (Action {
        card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), region: a.region,
        ..Action::new("play")
    }) || a.ability_id.as_deref().is_some_and(|key| key != "attach")) {
        return Err("JC035只声明本牌和结附地区，不声明准备触发或额外载荷".into());
    }
    Ok(())
}

impl Game {
    pub(crate) fn jc035_region_cards(&self, region: usize) -> impl Iterator<Item = &Card> {
        self.attachments.iter().filter(move |a| a.card.definition == "JC035" && !a.card.face_down
            && self.attachment_host_valid(a) && self.attachment_region(a) == Some(region)).map(|a| &a.card)
    }
    pub(crate) fn jc035_current_icons(&self, c: &Card, region: usize) -> Icons {
        if !self.jc035_region_cards(region).any(|live| same(live, c)) { return Icons::default(); }
        let d = catalog::card("JC035");
        d.permanent_icons.add(if self.team(c.controller) == self.first_team { d.temporary_icons } else { Icons::default() })
    }
    fn jc035_physical_cards(&self) -> impl Iterator<Item = &Card> {
        self.regions.iter().flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(self.players.iter().flat_map(|p| p.hand.iter().chain(&p.deck).chain(&p.assets)
                .chain(&p.graveyard).chain(&p.score_cards).chain(p.society_zone.card.iter())))
            .chain(&self.world).chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.sealed_cards.iter().map(|s| &s.card))
            .chain(self.stack.iter().filter_map(|s| s.card.as_ref()))
            .chain(self.effects.iter().filter_map(|e| if let Effect::Bury { card } = e { Some(card) } else { None }))
    }
    fn jc035_source(&self, s: &SourceSnapshot, actor: usize, prepared: bool) -> RuleResult<()> {
        if s.card.definition != "JC035" || s.card.id.is_empty() || s.card.face_down
            || actor >= self.players.len() || s.card.controller != actor || s.card.owner >= self.players.len()
            || s.observed_death.is_some() || s.observed_mobility_grant.is_some() || s.play_source.is_some()
            || self.jc035_physical_cards().any(|c| c.id == s.card.id
                && (c.definition != s.card.definition || c.owner != s.card.owner || c.face_down)) {
            return Err("JC035冻结来源、拥有者或操控席位不符".into());
        }
        if prepared {
            let r = s.region.filter(|r| *r < self.regions.len()).ok_or("JC035准备来源缺少原地区")?;
            let id = s.source_region_instance.as_deref().filter(|id| !id.is_empty() && *id != s.card.id)
                .ok_or("JC035准备来源缺少原地区UID")?;
            if s.attachment_host_instance.as_deref() != Some(id)
                || self.regions.iter().enumerate().any(|(index, region)| region.card.id == id && index != r)
                || self.jc035_physical_cards().any(|c| c.id == id && catalog::card(&c.definition).kind != "region") {
                return Err("JC035必须绑定原结附地区实体，不能改成其他区域或替代地区".into());
            }
        } else if s.region.is_some() || s.source_region_instance.is_some() || s.attachment_host_instance.is_some() {
            return Err("JC035手牌结附来源不能夹带准备地区快照".into());
        }
        Ok(())
    }
    fn jc035_context(&self) -> RuleResult<&PreparationObservation> {
        let o = self.jc035_preparation.as_ref().ok_or("JC035缺少准备步骤观察")?;
        if o.turn == 0 || o.turn > self.turn || o.first_team > 1 || o.hands.len() != self.players.len()
            || o.hands.iter().enumerate().any(|(seat, h)| h.seat != seat
                || h.eliminated && !self.players[seat].eliminated)
            || (!o.entries.is_empty() && (o.turn != self.turn || o.first_team != self.first_team))
            || (o.entries.is_empty() != o.event_id.is_none())
            || o.event_id.as_ref().is_some_and(|id| id.is_empty()) {
            return Err("JC035准备观察的回合、席位或事件身份无效".into());
        }
        let mut sources = BTreeSet::new();
        let mut frames = BTreeSet::new();
        for entry in &o.entries {
            let s = &entry.source; let actor = s.card.controller;
            self.jc035_source(s, actor, true)?;
            if !sources.insert(s.card.id.clone()) || o.hands[actor].eliminated
                || !o.hands.iter().any(|h| !h.eliminated && self.team(h.seat) != self.team(actor) && h.count <= 3) {
                return Err("JC035准备资格只来自当时存活的敌方席位，不能重复来源".into());
            }
            if let Some(PreparationDecision::Accepted { frame_id } | PreparationDecision::Resolved { frame_id }) = &entry.decision {
                if frame_id.is_empty() || frame_id == &s.card.id || o.event_id.as_ref() == Some(frame_id)
                    || !frames.insert(frame_id.clone()) {
                    return Err("JC035每项发动必须绑定独立且唯一的响应帧".into());
                }
            }
        }
        Ok(o)
    }
    fn jc035_member(&self, source: &SourceSnapshot, event_id: &str) -> RuleResult<&PreparationEntry> {
        let o = self.jc035_context()?;
        if o.event_id.as_deref() != Some(event_id) { return Err("JC035程序不属于本次准备事件".into()); }
        o.entries.iter().find(|e| same(&e.source, source)).ok_or("JC035程序来源不在原准备观察中".into())
    }
    fn jc035_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if d.source.card.definition != "JC035" && !own(&d.ability) { return Ok(()); }
        validate_ability(&d.source.card.definition, &d.ability)?;
        let [Op::JC035PreparedInfluence { event_id: Some(event_id) }] = d.ability.ops.as_slice() else {
            return Err("JC035运行触发必须携带原准备事件".into());
        };
        if d.actor != d.source.card.controller || self.jc035_member(&d.source, event_id)?.decision.is_some() {
            return Err("JC035触发声明已处理或行动者不符".into());
        }
        Ok(())
    }
    pub(crate) fn jc035_begin_preparation(&mut self) {
        if self.jc035_preparation.as_ref().is_some_and(|o| o.turn == self.turn) { return; }
        let hands = self.players.iter().map(|p| PreparationHand {
            seat: p.seat, count: p.hand.len(), eliminated: p.eliminated,
        }).collect::<Vec<_>>();
        let mut sources = self.attachments.iter()
            .filter(|a| a.card.definition == "JC035" && !a.card.face_down && self.attachment_host_valid(a))
            .filter(|a| !self.players[a.card.controller].eliminated
                && hands.iter().any(|h| !h.eliminated && self.team(h.seat) != self.team(a.card.controller) && h.count <= 3))
            .filter_map(|a| self.attachment_region(a).map(|r| self.source_snapshot(&a.card, Some(r))))
            .collect::<Vec<_>>();
        // Seat ordering between teammates is the existing project contract;
        // each controller explicitly chooses its own simultaneous order.
        sources.sort_by_key(|s| (self.team(s.card.controller) != self.first_team, s.card.controller, s.card.id.clone()));
        let event_id = (!sources.is_empty()).then(|| self.id());
        self.jc035_preparation = Some(PreparationObservation {
            turn: self.turn, first_team: self.first_team, event_id: event_id.clone(), hands,
            entries: sources.iter().cloned().map(|source| PreparationEntry { source, decision: None }).collect(),
        });
        for source in sources {
            let mut ability = definition().abilities[1].clone();
            ability.ops = vec![Op::JC035PreparedInfluence { event_id: event_id.clone() }];
            self.effects.push_back(Effect::Declare { declaration: Declaration {
                actor: source.card.controller, source, ability,
            }});
        }
    }
    fn jc035_options(&self, ds: &[Declaration]) -> Vec<ChoiceOption> {
        ds.iter().map(|d| {
            let mut o = self.option(&d.source.card, d.actor, d.source.region, None);
            o.label = format!("地区{}：放置1本方势力", d.source.region.unwrap() + 1);
            o
        }).collect()
    }
    fn jc035_offer_order(&mut self, ds: Vec<Declaration>) {
        self.choice(ds[0].actor, ORDER_KIND, ORDER_TITLE.into(), self.jc035_options(&ds), 0, 1, None,
            ChoiceResolution::JC035PreparationOrder { declarations: ds });
        self.pending.as_mut().unwrap().choice.description = ORDER_DESCRIPTION.into();
    }
    pub(crate) fn jc035_declare(&mut self, d: Declaration) -> RuleResult<bool> {
        self.jc035_declaration(&d)?;
        if !own(&d.ability) { return Ok(false); }
        let mut ds = vec![d];
        while self.effects.front().is_some_and(|e| matches!(e, Effect::Declare { declaration: next }
            if next.actor == ds[0].actor && next.ability.key == KEY)) {
            if let Some(Effect::Declare { declaration: next }) = self.effects.pop_front() {
                self.jc035_declaration(&next)?; ds.push(next);
            }
        }
        if self.players[ds[0].actor].eliminated {
            self.jc035_decline(&ds);
        } else { self.jc035_offer_order(ds); }
        Ok(true)
    }
    fn jc035_decline(&mut self, ds: &[Declaration]) {
        if let Some(o) = &mut self.jc035_preparation {
            for entry in &mut o.entries {
                if ds.iter().any(|d| d.source.card.id == entry.source.card.id) {
                    entry.decision = Some(PreparationDecision::Declined);
                }
            }
        }
    }
    pub(crate) fn jc035_choose_order(&mut self, mut ds: Vec<Declaration>, a: &Action, selected: &[String]) -> RuleResult<()> {
        if *a != (Action { choice_id: a.choice_id.clone(), selected: Some(selected.to_vec()), ..Action::new("choose") })
            || selected.len() > 1 || ds.is_empty() {
            return Err("JC035准备选择只可发动一项或放弃剩余项".into());
        }
        for d in &ds { self.jc035_declaration(d)?; }
        if selected.is_empty() { self.jc035_decline(&ds); return Ok(()); }
        let index = ds.iter().position(|d| d.source.card.id == selected[0]).ok_or("JC035准备来源不在待选列表")?;
        let d = ds.remove(index);
        let frame = self.make_frame(d.actor, d.source.clone(), &d.ability, vec![], vec![], None);
        self.dispatch_frame(frame, format!("{}：{}", catalog::card("JC035").name, d.ability.label), None, ResponsePolicy::Respondable);
        let frame_id = self.stack.last().unwrap().id.clone();
        self.jc035_preparation.as_mut().unwrap().entries.iter_mut().find(|e| e.source.card.id == d.source.card.id)
            .unwrap().decision = Some(PreparationDecision::Accepted { frame_id });
        if !ds.is_empty() { self.jc035_offer_order(ds); }
        Ok(())
    }
    pub(crate) fn jc035_reject_optional(&self, d: &Declaration) -> RuleResult<()> {
        if d.source.card.definition == "JC035" || own(&d.ability) {
            return Err("JC035准备触发必须使用原观察的排序/可选队列".into());
        }
        Ok(())
    }
    pub(crate) fn validate_jc035_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if !related(f) { return Ok(()); }
        if f.frame_id.is_empty() { return Err("JC035缺少帧身份".into()); }
        if f.ability_key == "attach" {
            self.jc035_source(&f.source, f.actor, false)?;
            if self.jc035_physical_cards().any(|c| c.id == f.source.card.id) {
                return Err("JC035结附的原手牌实体必须已经离开物理区域".into());
            }
            let [t] = f.targets.as_slice() else { return Err("JC035结附须恰好一个地区目标".into()); };
            let r = t.id.strip_prefix("region:").and_then(|s| s.parse::<usize>().ok())
                .filter(|r| *r < self.regions.len() && t.id == format!("region:{r}"))
                .ok_or("JC035结附地区标识无效")?;
            let region_id = t.region_instance.as_deref().filter(|id| !id.is_empty() && *id != f.source.card.id)
                .ok_or("JC035结附缺少原地区实体")?;
            if !same(&t.spec, &definition().abilities[0].targets[0]) || t.public.instance_id != t.id
                || t.public.kind != "region" || t.public.region != Some(r) || t.public.owner.is_some()
                || t.public.controller.is_some() || !t.public.valid || t.public.status != "valid"
                || t.public.invalid_reason.is_some() || !f.steps.is_empty() || f.cursor != 0
                || f.chosen_region != Some(r)
                || self.regions.iter().enumerate().any(|(index, reg)| reg.card.id == region_id && index != r)
                || self.jc035_physical_cards().any(|c| c.id == region_id
                    && (catalog::card(&c.definition).kind != "region" || catalog::card(&c.definition).name != t.public.label))
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= 3
                    && ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("JC035结附的完整费用、地区或公开绑定被改写".into());
            }
            return Ok(());
        }
        let [step] = f.steps.as_slice() else { return Err("JC035准备帧只执行一次固定效果".into()); };
        let Op::JC035PreparedInfluence { event_id: Some(event_id) } = &step.op else {
            return Err("JC035准备帧缺少有限观察程序".into());
        };
        let member = self.jc035_member(&f.source, event_id)?;
        let accepted = match &member.decision {
            Some(PreparationDecision::Accepted { frame_id }) => frame_id == &f.frame_id,
            Some(PreparationDecision::Resolved { frame_id }) => frame_id == &f.frame_id && f.cursor == 1,
            _ => false,
        };
        if f.ability_key != KEY || f.actor != f.source.card.controller || step.context != f.actor
            || !f.targets.is_empty() || !f.already_paid.is_empty() || f.chosen_region.is_some() || !accepted
            || f.cursor > 1 || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked))
            || (f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)) {
            return Err("JC035准备帧的来源、响应状态、游标或程序不符".into());
        }
        Ok(())
    }
    pub(crate) fn jc035_resolve(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_jc035_frame(f)?;
        if f.ability_key != KEY || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("JC035准备效果必须经过已接受的原响应帧".into());
        }
        let entry = self.jc035_preparation.as_mut().unwrap().entries.iter_mut()
            .find(|e| e.source.card.id == f.source.card.id).unwrap();
        if !matches!(&entry.decision, Some(PreparationDecision::Accepted { frame_id }) if frame_id == &f.frame_id) {
            return Err("JC035准备效果不能重复执行".into());
        }
        entry.decision = Some(PreparationDecision::Resolved { frame_id: f.frame_id.clone() });
        if let Some(r) = f.source.region.filter(|r| self.regions.get(*r).is_some_and(|region|
            !region.vacant && f.source.source_region_instance.as_ref() == Some(&region.card.id))) {
            self.place_influence(f.actor, r, 1);
        }
        Ok(())
    }
    pub(crate) fn validate_jc035_state(&self) -> RuleResult<()> {
        if self.regions.iter().any(|r| r.card.definition == "JC035"
            || r.cards.iter().any(|c| c.definition == "JC035")) {
            return Err("JC035在场内只能是公开的地区附属，不能成为角色或地区".into());
        }
        let physical = self.jc035_physical_cards().filter(|c| c.definition == "JC035").collect::<Vec<_>>();
        for c in physical {
            if c.id.is_empty() || c.owner >= self.players.len() || c.controller >= self.players.len()
                || self.jc035_physical_cards().filter(|other| other.id == c.id).count() != 1 {
                return Err("JC035物理实体必须有合法拥有者、操控者和唯一UID".into());
            }
        }
        for a in self.attachments.iter().filter(|a| a.card.definition == "JC035") {
            if a.card.face_down || self.players[a.card.owner].eliminated
                || self.players[a.card.controller].eliminated || !self.attachment_host_valid(a) {
                return Err("JC035场内附属必须公开、由存活玩家持有并结附实际地区".into());
            }
        }
        if self.window == Some(Window::Prepare) && self.jc035_preparation.is_none()
            && self.attachments.iter().any(|a| a.card.definition == "JC035") {
            return Err("JC035准备步骤不能丢失整份观察及其声明".into());
        }
        if self.jc035_preparation.is_some() { self.jc035_context()?; }
        let mut seen = BTreeSet::new();
        let mut remember = |s: &SourceSnapshot| -> RuleResult<()> {
            if !seen.insert(s.card.id.clone()) { return Err("JC035同一准备来源不能重复出现在多个容器".into()); }
            Ok(())
        };
        for item in &self.stack {
            let Some(f) = &item.frame else {
                if item.card.as_ref().is_some_and(|c| c.definition == "JC035") {
                    return Err("JC035堆栈不能缺少冻结帧".into());
                }
                continue;
            };
            self.validate_jc035_frame(f)?;
            if !related(f) {
                if item.card.as_ref().is_some_and(|c| c.definition == "JC035") {
                    return Err("JC035实体不能挂接其他程序".into());
                }
                continue;
            }
            if item.id != f.frame_id || item.controller != f.actor || item.deploy_region.is_some()
                || item.reveal || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                || item.target != f.targets.first().map(|t| t.id.clone()) {
                return Err("JC035堆栈外壳与原帧不符".into());
            }
            if f.ability_key == "attach" {
                if item.label != catalog::card("JC035").name || item.card.as_ref().is_none_or(|c| {
                    let mut expected = f.source.card.clone(); expected.id = c.id.clone();
                    expected.time_markers = 0; expected.lock_markers = 0;
                    c.id.is_empty() || c.id == f.source.card.id || !same(c, &expected)
                }) { return Err("JC035结附事务实体不符".into()); }
            } else {
                remember(&f.source)?;
                if item.card.is_some() || item.label != format!("{}：{}", catalog::card("JC035").name, definition().abilities[1].label) {
                    return Err("JC035准备能力不能夹带事务实体或改写展示".into());
                }
            }
        }
        for effect in &self.effects {
            match effect {
                Effect::Declare { declaration: d } => {
                    self.jc035_declaration(d)?;
                    if own(&d.ability) { remember(&d.source)?; }
                },
                Effect::Frame { frame } if related(frame) => return Err("JC035不能绕过响应堆栈直接执行".into()),
                _ => {},
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::JC035PreparationOrder { declarations: ds } => {
                    if ds.is_empty() { return Err("JC035准备待选不能为空".into()); }
                    for d in ds {
                        self.jc035_declaration(d)?; remember(&d.source)?;
                        if d.actor != p.seat { return Err("JC035准备排序不能跨操控席位".into()); }
                    }
                    let expected = Choice { id: p.choice.id.clone(), kind: ORDER_KIND.into(), title: ORDER_TITLE.into(),
                        description: ORDER_DESCRIPTION.into(), player_id: player_id(p.seat), options: self.jc035_options(ds),
                        preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                    if self.status != "playing" || p.seat >= self.players.len() || self.players[p.seat].eliminated
                        || p.choice.id.is_empty() || !same(&p.choice, &expected) {
                        return Err("JC035准备待选必须绑定原操控者与完整选项和数量".into());
                    }
                },
                ChoiceResolution::Declare { declaration: d, .. }
                    if d.source.card.definition == "JC035" || own(&d.ability) =>
                        return Err("JC035准备触发不能改成其他待选阶段".into()),
                ChoiceResolution::Frame { frame, .. } if related(frame) =>
                    return Err("JC035固定准备效果没有中途额外待选".into()),
                _ => {},
            }
        }
        // The forward checks bind every container to its observation and reject
        // duplicates. A live unfinished observation must also have a container.
        // Elimination and finish legitimately cancel containers without rewriting
        // these historical decisions; leaving or changing control does not.
        if self.status == "playing" {
            if let Some(observation) = &self.jc035_preparation {
                for entry in &observation.entries {
                    if !self.players[entry.source.card.controller].eliminated
                        && matches!(&entry.decision, None | Some(PreparationDecision::Accepted { .. }))
                        && !seen.contains(&entry.source.card.id)
                    {
                        return Err("JC035未完成准备观察必须对应原声明、待选或响应帧".into());
                    }
                }
            }
        }
        Ok(())
    }
}
