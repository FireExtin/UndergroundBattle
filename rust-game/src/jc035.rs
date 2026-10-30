//! One frozen preparation event and controller-ordered queue for the admitted preparation cards.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "jc035-preparation-influence";
const ORDER_KIND: &str = "jc035-preparation-order";
const ORDER_TITLE: &str = "选择下一项准备触发";
const ORDER_DESCRIPTION: &str = "选择下一项发动并入栈；强制触发必须入栈，可放弃剩余可选触发。后入栈的先结算。";

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
    Cancelled { frame_id: String },
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
        Op::BQ095Prepare { .. } | Op::WM026Prepare { .. } | Op::CQ27Prepare { .. } | Op::MSJC16Prepare { .. } | Op::BQ052Prepare { .. } | Op::WM028Prepare { .. } | Op::LC16PrepareSearch { .. } | Op::XG16PrepareTime { .. } | Op::Batch3Prepare { .. } | Op::JC035PreparedInfluence { .. } | Op::Batch20Prepare { .. } | Op::BQ085Prepare { .. } | Op::JC055PrepareTime { .. } | Op::CQ08Prepare { .. } | Op::CQ10Prepare { .. } => true,
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
    f.source.card.definition == "JC035" || f.ability_key == KEY || f.ability_key.starts_with("batch20-prepare-")
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
pub(crate) fn preparation_card(id: &str) -> bool { matches!(id, "BQ095" | "ZHWM026" | "CQ27" | "MSJC16" | "BQ052" | "WM028" | "LC16" | "XG16" | "BQ079" | "BQ087" | "JC064" | "XG28" | "JC035" | "JC048" | "WM034" | "BQ091" | "JC068" | "BQ085" | "JC055" | "CQ08" | "CQ10") }
pub(crate) fn preparation_forced(id: &str) -> bool { matches!(id, "WM028" | "BQ079" | "BQ087" | "JC064" | "JC048" | "WM034" | "JC068" | "BQ085") }
pub(crate) fn preparation_frame(f: &ResolutionFrame) -> bool { f.ability_key == KEY || f.ability_key.starts_with("batch20-prepare-") || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op))) }
fn event(ops: &[Op]) -> Option<&str> { match ops {
    [Op::BQ095Prepare { event_id }] | [Op::WM026Prepare { event_id }] | [Op::CQ27Prepare { event_id }] | [Op::MSJC16Prepare { event_id }] | [Op::BQ052Prepare { event_id }] | [Op::WM028Prepare { event_id }] | [Op::LC16PrepareSearch { event_id }] | [Op::XG16PrepareTime { event_id }] | [Op::Batch3Prepare { event_id }] | [Op::JC035PreparedInfluence { event_id }] | [Op::Batch20Prepare { event_id }] | [Op::BQ085Prepare { event_id }] | [Op::JC055PrepareTime { event_id }] | [Op::CQ08Prepare { event_id }] | [Op::CQ10Prepare { event_id }] => event_id.as_deref(), _ => None,
}}
fn program(id: &str, event_id: Option<String>) -> RuleResult<AbilitySpec> {
    let mut a = if id == "BQ095" {crate::bq095::definition().abilities[2].clone()} else if id == "ZHWM026" {crate::wm026::definition(true).abilities[0].clone()} else if id == "JC035" { definition().abilities[1].clone() }
        else if id == "JC055" { crate::next20_red::definition(id).unwrap().abilities[0].clone() }
        else if id == "MSJC16" { crate::msjc16::definition().abilities[0].clone() }
        else if id == "BQ052" { crate::bq052::definition().abilities[1].clone() }
        else if id == "CQ27" { crate::cq27::definition().abilities[0].clone() }
        else if id == "CQ10" { crate::cq10::definition().abilities[1].clone() }
        else if id == "CQ08" { crate::next20_blue_gray::definition(id).unwrap().abilities[1].clone() }
        else { crate::batch20::definition(id).or_else(|| crate::black_next20::definition(id)).or_else(|| crate::batch3::definition(id)).or_else(|| crate::batch4::definition(id)).or_else(|| crate::import20_observers::definition(id)).and_then(|d| d.abilities.into_iter().find(|a| a.event == Some(Event::Prepare)))
            .ok_or("非准入准备来源")? };
    a.ops = if id == "BQ095" {vec![Op::BQ095Prepare{event_id}]} else if id == "ZHWM026" {vec![Op::WM026Prepare{event_id}]} else if id == "CQ27" {vec![Op::CQ27Prepare{event_id}]} else if id == "MSJC16" {vec![Op::MSJC16Prepare{event_id}]} else if id == "BQ052" { vec![Op::BQ052Prepare { event_id }] } else if id == "WM028" { vec![Op::WM028Prepare { event_id }] } else if id == "LC16" { vec![Op::LC16PrepareSearch { event_id }] } else if id == "XG16" { vec![Op::XG16PrepareTime { event_id }] } else if matches!(id, "XG28" | "JC064" | "BQ087" | "BQ079") { vec![Op::Batch3Prepare { event_id }] } else if id == "JC035" { vec![Op::JC035PreparedInfluence { event_id }] } else if id == "BQ085" { vec![Op::BQ085Prepare { event_id }] } else if id == "JC055" { vec![Op::JC055PrepareTime { event_id }] } else if id == "CQ10" { vec![Op::CQ10Prepare { event_id }] } else if id == "CQ08" { vec![Op::CQ08Prepare { event_id }] } else { vec![Op::Batch20Prepare { event_id }] };
    Ok(a)
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if !own(a) { return Ok(()); }
    if !preparation_card(id) || !same(a, &program(id, event(&a.ops).map(str::to_owned))?) {
        return Err("准备触发只能使用来源对应的完整固定程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if id == "JC035" && !same(d, &definition())
        || !preparation_card(id) && d.abilities.iter().any(own) {
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
    fn jc035_physical_cards(&self) -> impl Iterator<Item = &Card> {
        self.regions.iter().flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(self.players.iter().flat_map(|p| p.hand.iter().chain(&p.deck).chain(&p.assets)
                .chain(&p.graveyard).chain(self.bq095_headquarters.iter().filter(|z|z.seat==p.seat).flat_map(|z|&z.cards)).chain(&p.score_cards).chain(p.society_zone.card.iter())))
            .chain(&self.world).chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.sealed_cards.iter().map(|s| &s.card))
            .chain(self.stack.iter().filter_map(|s| s.card.as_ref()))
            .chain(self.effects.iter().filter_map(|e| if let Effect::Bury { card } = e { Some(card) } else { None }))
    }
    fn jc035_source(&self, s: &SourceSnapshot, actor: usize, prepared: bool) -> RuleResult<()> {
        if !preparation_card(&s.card.definition) || s.card.id.is_empty() || s.card.face_down
            || actor >= self.players.len() || s.card.controller != actor || s.card.owner >= self.players.len()
            || s.observed_death.is_some() || s.observed_mobility_grant.is_some() || s.observed_exposure.is_some() || s.play_source.is_some()
            || self.jc035_physical_cards().any(|c| c.id == s.card.id
                && (crate::wm026::printed_id(&c.definition) != crate::wm026::printed_id(&s.card.definition) || c.owner != s.card.owner || (c.face_down && s.card.definition != "JC055"))) {
            return Err("JC035冻结来源、拥有者或操控席位不符".into());
        }
        if prepared && s.card.definition=="MSJC16" {if s.region.is_some()||s.source_region_instance.is_some()||s.attachment_host_instance.is_some(){return Err("颂亡者准备来源只能原秘社".into());}} else if prepared && matches!(s.card.definition.as_str(), "BQ095" | "BQ052" | "JC068" | "WM028") {
            let host = s.attachment_host_instance.as_deref().ok_or("秘社准备来源缺少宿主")?;
            if host != format!("society:p{actor}") || s.region.is_some() || s.source_region_instance.is_some() {
                return Err("秘社准备触发不得伪装地区宿主".into());
            }
        } else if prepared {
            let r = s.region.filter(|r| *r < self.regions.len()).ok_or("JC035准备来源缺少原地区")?;
            let id = s.source_region_instance.as_deref().filter(|id| !id.is_empty() && *id != s.card.id)
                .ok_or("JC035准备来源缺少原地区UID")?;
            if s.attachment_host_instance.as_deref() != (if matches!(s.card.definition.as_str(), "ZHWM026" | "CQ27" | "CQ10" | "LC16" | "WM034" | "JC055" | "BQ085" | "CQ08" | "BQ087" | "BQ079" | "XG16") { None } else { Some(id) })
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
        let mut sisters_used = BTreeSet::new();
        for entry in &o.entries {
            let s = &entry.source; let actor = s.card.controller;
            self.jc035_source(s, actor, true)?;
            if !sources.insert(s.card.id.clone()) || o.hands[actor].eliminated
                || (s.card.definition=="MSJC16" && o.turn!=1)
                || (s.card.definition == "WM028" && self.team(actor) != o.first_team)
                || (matches!(s.card.definition.as_str(),"JC035"|"ZHWM026") && !o.hands.iter().any(|h| !h.eliminated && self.team(h.seat) != self.team(actor) && h.count <= 3))
                || (preparation_forced(&s.card.definition) && !self.players[actor].eliminated && matches!(entry.decision, Some(PreparationDecision::Declined))) {
                return Err("JC035准备资格只来自当时存活的敌方席位，不能重复来源".into());
            }
            if s.card.definition == "CQ08" && o.entries.iter().filter(|e| e.source.card.definition == "CQ08" && e.source.card.controller == actor).count() < 3 {
                return Err("三姐妹准备触发缺少当时同一操控者的三个公开同名来源".into());
            }
            if let Some(PreparationDecision::Accepted { frame_id } | PreparationDecision::Resolved { frame_id } | PreparationDecision::Cancelled { frame_id }) = &entry.decision {
                if s.card.definition == "CQ08" && !sisters_used.insert(actor) { return Err("三姐妹同名能力每操控者每回合只能发动一次".into()); }
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
    pub(crate) fn jc035_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if d.source.card.definition != "JC035" && !own(&d.ability) { return Ok(()); }
        validate_ability(&d.source.card.definition, &d.ability)?;
        let event_id = event(&d.ability.ops).ok_or("运行触发必须携带原准备事件")?;
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
        let mut sources = self.in_play_cards().into_iter().chain(self.players.iter().filter_map(|p|p.society_zone.card.as_ref().filter(|c|c.definition=="MSJC16").map(|c|(None,c))))
            .filter(|(_, c)| preparation_card(&c.definition) && !c.face_down && !self.players[c.controller].eliminated)
            .filter(|(_, c)| !matches!(c.definition.as_str(),"JC035"|"ZHWM026") || hands.iter().any(|h|
                !h.eliminated && self.team(h.seat) != self.team(c.controller) && h.count <= 3))
            .filter(|(_,c)|c.definition!="MSJC16"||self.turn==1)
            .filter(|(_, c)| c.definition != "CQ08" || self.cq08_count(c.controller) >= 3)
            .filter(|(_, c)| c.definition != "WM028" || self.team(c.controller) == self.first_team)
            .filter(|(_, c)| c.definition != "BQ087" || self.bq095_reference_cards(c).iter().any(|grave| grave.owner==self.bq095_reference_seat(c) && catalog::catalog().cards.iter().any(|d|d.id==grave.definition && d.kind=="character")))
            .map(|(region, c)| self.source_snapshot(c, region)).collect::<Vec<_>>();
        // Seat ordering between teammates is the existing project contract;
        // each controller explicitly chooses its own simultaneous order.
        sources.sort_by_key(|s| (self.team(s.card.controller) != self.first_team, s.card.controller, s.card.id.clone()));
        let event_id = (!sources.is_empty()).then(|| self.id());
        self.jc035_preparation = Some(PreparationObservation {
            turn: self.turn, first_team: self.first_team, event_id: event_id.clone(), hands,
            entries: sources.iter().cloned().map(|source| PreparationEntry { source, decision: None }).collect(),
        });
        for source in sources {
            let ability = program(&source.card.definition, event_id.clone()).expect("admitted preparation source");
            self.effects.push_back(Effect::Declare { declaration: Declaration {
                actor: source.card.controller, source, ability,
            }});
        }
    }
    fn jc035_options(&self, ds: &[Declaration]) -> Vec<ChoiceOption> {
        ds.iter().map(|d| {
            let mut o = self.option(&d.source.card, d.actor, d.source.region, None);
            o.label = format!("{}：{}", catalog::card(&d.source.card.definition).name, d.ability.label);
            o
        }).collect()
    }
    fn jc035_offer_order(&mut self, ds: Vec<Declaration>) {
        self.choice(ds[0].actor, ORDER_KIND, ORDER_TITLE.into(), self.jc035_options(&ds), usize::from(ds.iter().any(|d| preparation_forced(&d.source.card.definition))), 1, None,
            ChoiceResolution::JC035PreparationOrder { declarations: ds });
        self.pending.as_mut().unwrap().choice.description = ORDER_DESCRIPTION.into();
    }
    pub(crate) fn jc035_declare(&mut self, d: Declaration) -> RuleResult<bool> {
        self.jc035_declaration(&d)?;
        if !own(&d.ability) { return Ok(false); }
        let mut ds = vec![d];
        while self.effects.front().is_some_and(|e| matches!(e, Effect::Declare { declaration: next }
            if next.actor == ds[0].actor && own(&next.ability))) {
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
        if selected.is_empty() {
            if ds.iter().any(|d| preparation_forced(&d.source.card.definition)) { return Err("剩余强制准备触发不能放弃".into()); }
            self.jc035_decline(&ds); return Ok(());
        }
        let index = ds.iter().position(|d| d.source.card.id == selected[0]).ok_or("JC035准备来源不在待选列表")?;
        let d = ds.remove(index);
        if d.source.card.definition == "CQ08" {
            let siblings = ds.iter().filter(|other| other.source.card.definition == "CQ08" && other.actor == d.actor).cloned().collect::<Vec<_>>();
            self.jc035_decline(&siblings);
            ds.retain(|other| other.source.card.definition != "CQ08" || other.actor != d.actor);
        }
        if matches!(d.source.card.definition.as_str(), "CQ10" | "LC16" | "JC055" | "BQ087" | "XG16") {
            let options = self.binding_options(d.actor, &d.source, &d.ability.targets[0]);
            if options.is_empty() {
                self.jc035_decline(&[d]); if !ds.is_empty() { self.jc035_offer_order(ds); }
            } else {
                let (kind,title)=if d.source.card.definition=="CQ10" {("cq10-preparation-target","选择朱苏德拉准备效果的目标地区")} else if d.source.card.definition=="BQ087" {("batch3-bq087-target","选择本方墓地角色，结算时进入随机地区")} else if d.source.card.definition=="LC16" {("lc16-preparation-target","选择江月准备效果的目标地区")} else {("jc055-preparation-target","选择放置时间标志的角色或附属")};
                self.choice(d.actor, kind, title.into(), options, 1, 1, None,
                    ChoiceResolution::JC055PreparationTarget { declaration: d, remaining: ds });
            }
            return Ok(());
        }
        let frame = self.make_frame(d.actor, d.source.clone(), &d.ability, vec![], vec![], None);
        self.dispatch_frame(frame, format!("{}：{}", catalog::card(&d.source.card.definition).name, d.ability.label), None, ResponsePolicy::Respondable);
        let frame_id = self.stack.last().unwrap().id.clone();
        self.jc035_preparation.as_mut().unwrap().entries.iter_mut().find(|e| e.source.card.id == d.source.card.id)
            .unwrap().decision = Some(PreparationDecision::Accepted { frame_id });
        if !ds.is_empty() { self.jc035_offer_order(ds); }
        Ok(())
    }
    pub(crate) fn jc055_preparation_target(&mut self, d: Declaration, ds: Vec<Declaration>, a: &Action, selected: &[String]) -> RuleResult<()> {
        self.jc035_declaration(&d)?;
        for next in &ds { self.jc035_declaration(next)?; }
        if !matches!(d.source.card.definition.as_str(), "CQ10" | "LC16" | "JC055" | "BQ087" | "XG16") || selected.len() != 1 || *a != (Action { choice_id: a.choice_id.clone(), selected: Some(selected.to_vec()), ..Action::new("choose") }) {
            return Err("JC055准备目标不能夹带额外声明".into());
        }
        if let Some(id) = selected.first() {
            let spec = &d.ability.targets[0];
            if !self.valid_binding(d.actor, &d.source, spec, id) { return Err("JC055准备目标失效".into()); }
            let chosen_region = if matches!(d.source.card.definition.as_str(), "LC16" | "CQ10") { Some(id.strip_prefix("region:").and_then(|s|s.parse::<usize>().ok()).ok_or("江月须指定地区")?) } else { None };
            let target = BoundTarget { id: id.clone(), spec: spec.clone(), public: self.public_target(d.actor,&d.source,spec,id), region_instance: chosen_region.map(|r|self.regions[r].card.id.clone()) };
            let f = self.make_frame(d.actor,d.source.clone(),&d.ability,vec![target],vec![],chosen_region);
            self.dispatch_frame(f,format!("{}：{}",catalog::card(&d.source.card.definition).name,d.ability.label),None,ResponsePolicy::Respondable);
            let frame_id = self.stack.last().unwrap().id.clone();
            self.jc035_preparation.as_mut().unwrap().entries.iter_mut().find(|e| e.source.card.id == d.source.card.id).unwrap().decision = Some(PreparationDecision::Accepted { frame_id });
        } else { self.jc035_decline(&[d]); }
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
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= 3
                    && ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("JC035结附的完整费用、地区或公开绑定被改写".into());
            }
            return Ok(());
        }
        let [step] = f.steps.as_slice() else { return Err("JC035准备帧只执行一次固定效果".into()); };
        let event_id = event(std::slice::from_ref(&step.op)).ok_or("准备帧缺少有限观察程序")?;
        let expected = program(&f.source.card.definition, Some(event_id.into()))?;
        let member = self.jc035_member(&f.source, event_id)?;
        let accepted = match &member.decision {
            Some(PreparationDecision::Accepted { frame_id }) => frame_id == &f.frame_id,
            Some(PreparationDecision::Resolved { frame_id }) => frame_id == &f.frame_id && f.cursor == 1,
            _ => false,
        };
        if f.ability_key != expected.key || !same(&step.op, &expected.ops[0]) || f.actor != f.source.card.controller || step.context != f.actor
            || (if f.source.card.definition == "LC16" { !self.lc16_bound_region(f) } else if f.source.card.definition == "CQ10" { !self.cq10_bound_region(f) } else {
                f.targets.len() != expected.targets.len() || f.targets.iter().zip(&expected.targets).any(|(t,s)|
                    !same(&t.spec,s) || t.id.is_empty() || t.public.instance_id != t.id || t.region_instance.is_some()
                        || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some()
                        || !matches!(t.public.kind.as_str(), "character" | "attachment") || t.public.owner.is_none() || t.public.controller.is_none())
                    || f.chosen_region.is_some()
            })
            || !f.already_paid.is_empty() || !accepted
            || f.cursor > 1 || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked))
            || (f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)) {
            return Err("JC035准备帧的来源、响应状态、游标或程序不符".into());
        }
        Ok(())
    }
    pub(crate) fn jc035_cancel_guarded_frame(&mut self,f:&ResolutionFrame)->RuleResult<()> {
        if !preparation_frame(f) { return Ok(()); }
        if f.cursor!=0 || !matches!(f.guard,GuardState::Cancelled) { return Err("准备取消必须来自原目标守卫".into()); }
        let entry=self.jc035_preparation.as_mut().and_then(|o|o.entries.iter_mut().find(|e|e.source.card.id==f.source.card.id)).ok_or("取消缺少原准备来源")?;
        if !matches!(&entry.decision,Some(PreparationDecision::Accepted{frame_id}) if frame_id==&f.frame_id) { return Err("取消不能改写其他准备收据".into()); }
        entry.decision=Some(PreparationDecision::Cancelled{frame_id:f.frame_id.clone()});Ok(())
    }
    pub(crate) fn jc035_resolve(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_jc035_frame(f)?;
        if !preparation_frame(f) || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) {
            return Err("JC035准备效果必须经过已接受的原响应帧".into());
        }
        let entry = self.jc035_preparation.as_mut().unwrap().entries.iter_mut()
            .find(|e| e.source.card.id == f.source.card.id).unwrap();
        if !matches!(&entry.decision, Some(PreparationDecision::Accepted { frame_id }) if frame_id == &f.frame_id) {
            return Err("JC035准备效果不能重复执行".into());
        }
        entry.decision = Some(PreparationDecision::Resolved { frame_id: f.frame_id.clone() });
        if matches!(f.source.card.definition.as_str(), "CQ08" | "WM028") { return Ok(()); }
        if f.source.card.definition=="MSJC16" {return self.mill_cards_with_cause(f.actor,3,Some(f));}
        if f.source.card.definition == "ZHWM026" {return self.wm026_prepare_resolve(f);}
        if f.source.card.definition == "CQ27" {return self.cq27_prepare(f);}
        if f.source.card.definition == "BQ095" {self.effects.push_front(Effect::Discard{seat:f.actor,amount:1,redraw:false,optional:false});self.effects.push_front(Effect::Draw{seat:f.actor,count:2});return Ok(());}
        if f.source.card.definition == "BQ052" { return self.bq052_generate(f.actor); }
        if f.source.card.definition == "LC16" { return self.lc16_search_start(f); }
        if f.source.card.definition == "CQ10" { return self.cq10_prepare(f); }
        if f.source.card.definition == "XG28" { return self.mill_cards_with_cause(f.actor, 1, Some(f)); }
        if f.source.card.definition == "JC064" { return self.jc064_upkeep_start(f); }
        if f.source.card.definition == "BQ079" { return self.bq079_upkeep_start(f); }
        if f.source.card.definition == "BQ087" {
            let regions=self.regions.iter().enumerate().filter(|(_,r)|!r.vacant).map(|(i,_)|i).collect::<Vec<_>>();
            if !regions.is_empty() { let region=regions[self.random_below(regions.len())]; self.own_graveyard_entry(&f.source.card,f.actor,region,&f.targets[0].id)?; }
            return Ok(());
        }
        if f.source.card.definition == "BQ085" { return self.bq085_upkeep_start(f.clone()); }
        if f.source.card.definition == "JC055" { self.jc055_prepare_time(f); return Ok(()); }
        if f.source.card.definition == "XG16" { self.xg16_prepare_time(f); return Ok(()); }
        if f.source.card.definition != "JC035" { return self.batch20_prepare_resolve(f); }
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
            && (self.in_play_cards().iter().any(|(_, c)| preparation_card(&c.definition)) || self.turn==1&&self.players.iter().any(|p|!p.eliminated&&p.society_zone.card.as_ref().is_some_and(|c|c.definition=="MSJC16"&&!c.face_down))) {
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
                    expected.time_markers = 0; expected.charge_markers = 0; expected.lock_markers = 0;
                    c.id.is_empty() || c.id == f.source.card.id || !same(c, &expected)
                }) { return Err("JC035结附事务实体不符".into()); }
            } else {
                remember(&f.source)?;
                if item.card.is_some() || item.label != format!("{}：{}", catalog::card(&f.source.card.definition).name, program(&f.source.card.definition, None)?.label) {
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
                Effect::Frame { frame } if related(frame) => {
                    // A completed WM028 step resumes after its original contest;
                    // cursor1 cannot execute or repay the preparation operation.
                    if frame.source.card.definition != "WM028"
                        || frame.ability_key != crate::import20_observers::HALL_PREP || frame.cursor != 1
                        || !matches!(frame.guard, GuardState::Accepted) {
                        return Err("JC035不能绕过响应堆栈直接执行".into());
                    }
                    self.validate_jc035_frame(frame)?; remember(&frame.source)?;
                },
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
                        preview_cards: vec![], min: Some(usize::from(ds.iter().any(|d| preparation_forced(&d.source.card.definition)))), max: Some(1), amount: None, allow_decline: Some(!ds.iter().any(|d| preparation_forced(&d.source.card.definition))) };
                    if self.status != "playing" || p.seat >= self.players.len() || self.players[p.seat].eliminated
                        || p.choice.id.is_empty() || !same(&p.choice, &expected) {
                        return Err("JC035准备待选必须绑定原操控者与完整选项和数量".into());
                    }
                },
                ChoiceResolution::JC055PreparationTarget { declaration: d, remaining: ds } => {
                    for entry in std::iter::once(d).chain(ds) {
                        self.jc035_declaration(entry)?; remember(&entry.source)?;
                        if entry.actor != p.seat { return Err("准备目标续体不能跨操控席位".into()); }
                    }
                    let options = self.binding_options(d.actor,&d.source,&d.ability.targets[0]);
                    let (kind,title)=if d.source.card.definition=="CQ10" {("cq10-preparation-target","选择朱苏德拉准备效果的目标地区")} else if d.source.card.definition=="BQ087" {("batch3-bq087-target","选择本方墓地角色，结算时进入随机地区")} else if d.source.card.definition=="LC16" {("lc16-preparation-target","选择江月准备效果的目标地区")} else {("jc055-preparation-target","选择放置时间标志的角色或附属")};
                    if !matches!(d.source.card.definition.as_str(), "CQ10" | "LC16" | "JC055" | "BQ087" | "XG16") || p.choice.id.is_empty() || p.choice.player_id != player_id(d.actor)
                        || p.choice.kind != kind || p.choice.title != title
                        || p.choice.min != Some(1) || p.choice.max != Some(1) || p.choice.allow_decline != Some(false)
                        || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&p.choice.options,&options) {
                        return Err("JC055准备目标待选错误".into());
                    }
                },
                ChoiceResolution::Declare { declaration: d, .. }
                    if d.source.card.definition == "JC035" || own(&d.ability) =>
                        return Err("JC035准备触发不能改成其他待选阶段".into()),
                ChoiceResolution::Frame { frame, choice: FrameChoice::NextBlueGraySearch { inspected_ids } } if preparation_frame(frame) => {
                    self.validate_next_blue_gray_search(p, frame, inspected_ids)?;
                    remember(&frame.source)?;
                },
                ChoiceResolution::Frame { frame, choice: FrameChoice::BQ085Upkeep } if preparation_frame(frame) => {
                    self.validate_jc035_frame(frame)?;
                    self.validate_bq085_upkeep(p, frame)?;
                    remember(&frame.source)?;
                },
                ChoiceResolution::Frame { frame, choice: FrameChoice::JC068Upkeep } if preparation_frame(frame) => {
                    self.validate_jc035_frame(frame)?;
                    self.validate_jc068_upkeep(p, frame)?;
                    remember(&frame.source)?;
                },
                ChoiceResolution::Frame { frame, choice: FrameChoice::LC16Search } if preparation_frame(frame) => {
                    self.validate_lc16_pending(p,frame)?; remember(&frame.source)?;
                },
                ChoiceResolution::Frame {frame,choice:FrameChoice::CQ27Bottom} if preparation_frame(frame)=>{self.validate_cq27_pending(p,frame)?;remember(&frame.source)?;},
                ChoiceResolution::Frame { frame, choice: FrameChoice::BQ079Upkeep } if preparation_frame(frame) => {
                    self.validate_bq079_pending(p, frame)?; remember(&frame.source)?;
                },
                ChoiceResolution::Frame { frame, choice: FrameChoice::JC064Upkeep } if preparation_frame(frame) => {
                    self.validate_jc035_frame(frame)?; self.validate_jc064_pending(p, frame)?; remember(&frame.source)?;
                },
                ChoiceResolution::Frame { frame, choice: FrameChoice::MysticCommit { .. } | FrameChoice::MysticReward { .. } }
                    if frame.source.card.definition == "WM028" && frame.ability_key == crate::import20_observers::HALL_PREP => {
                    self.validate_jc035_frame(frame)?;
                    if frame.cursor != 1 || !matches!(frame.guard, GuardState::Accepted) {
                        return Err("阴影厅堂对抗必须属于已消费的原准备帧".into());
                    }
                    // The shared mystic validator authenticates the private
                    // chooser/options against this exact contest frame.
                    remember(&frame.source)?;
                },
                ChoiceResolution::Frame { frame, .. } if related(frame) =>
                    return Err("固定准备效果没有其他额外待选".into()),
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
