//! Complete JC054: mandatory, responsive conceal burst and end-step conceal.
//! Finite observations belong to the closed operations, never to arbitrary actions.
use crate::{catalog, engine::RuleResult, model::*, rules::{self, *}};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const BURST: &str = "jc054-conceal-burst";
pub(crate) const END: &str = "jc054-end-conceal";
const ORDER_KIND: &str = "jc054-trigger-order";
const ORDER_TITLE: &str = "蒙古死亡蠕虫：选择下一项强制触发入栈";
const ORDER_DESCRIPTION: &str = "必须将全部触发入栈；后入栈的先结算。";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConcealObservation {
    pub event_id: String,
    pub turn: u32,
    pub successor: Card,
    pub baseline: Option<ControlBaseline>,
    pub baseline_eliminated: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndStepObservation {
    pub event_id: String,
    pub turn: u32,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn key(key: &str) -> bool { matches!(key, BURST | END) }
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JC054ConcealBurst { .. } | Op::JC054EndHide { .. } => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted { exhausted, ready, .. } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn own_ability(a: &AbilitySpec) -> bool {
    key(&a.key) || matches!(a.event, Some(Event::Hide | Event::EndStep))
        || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops))
}
fn intrinsic(f: &ResolutionFrame) -> bool {
    key(&f.ability_key) || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
fn related(f: &ResolutionFrame) -> bool { f.source.card.definition == "JC054" || intrinsic(f) }
pub(crate) fn definition() -> Definition {
    rules::with_abilities(vec![
        rules::ability(BURST, "潜伏强制触发：对本地区所有其他角色造成1点伤害", Timing::Fast,
            vec![], vec![], vec![Op::JC054ConcealBurst { observation: None }], Some(Event::Hide)),
        rules::ability(END, "结束步骤强制触发：潜伏", Timing::Fast,
            vec![], vec![], vec![Op::JC054EndHide { observation: None }], Some(Event::EndStep)),
    ])
}
pub(crate) fn validate_ability(id: &str, ability: &AbilitySpec) -> RuleResult<()> {
    if !own_ability(ability) { return Ok(()); }
    let mut normalized = ability.clone();
    match normalized.ops.as_mut_slice() {
        [Op::JC054ConcealBurst { observation }] => *observation = None,
        [Op::JC054EndHide { observation }] => *observation = None,
        _ => return Err("JC054只准入两条完整印刷强制触发".into()),
    }
    if id != "JC054" || !definition().abilities.iter().any(|a| same(a, &normalized)) {
        return Err("JC054完整强制触发不能改写或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if id == "JC054" && !same(d, &definition()) || id != "JC054" && d.abilities.iter().any(own_ability) {
        return Err("JC054必须同时保留潜伏伤害与结束潜伏两条能力".into());
    }
    Ok(())
}
pub(crate) fn validate_character_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "JC054" && (a.target_id.is_some() || a.ability_id.is_some() || a.option.is_some()
        || a.choice_id.is_some() || a.selected.is_some() || a.top.is_some() || a.bottom.is_some()
        || a.allocations.is_some() || a.cost_selected.is_some() || a.deck_draft.is_some()
        || a.kind == "reveal" && a.region.is_some()) {
        return Err("JC054派遣、秘密派遣和现身不接受额外程序".into());
    }
    Ok(())
}
fn observation(op: &Op) -> Option<(&str, u32)> {
    match op {
        Op::JC054ConcealBurst { observation: Some(o) } => Some((&o.event_id, o.turn)),
        Op::JC054EndHide { observation: Some(o) } => Some((&o.event_id, o.turn)),
        _ => None,
    }
}
fn declaration_event(d: &Declaration) -> Option<&str> {
    d.ability.ops.first().and_then(observation).map(|(id, _)| id)
}

impl Game {
    fn jc054_physical_cards(&self) -> impl Iterator<Item = &Card> {
        self.regions.iter().flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(self.players.iter().flat_map(|p| p.hand.iter().chain(&p.deck).chain(&p.assets)
                .chain(&p.graveyard).chain(&p.score_cards).chain(p.society_zone.card.iter())))
            .chain(&self.world).chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.sealed_cards.iter().map(|c| &c.card))
            .chain(self.stack.iter().filter_map(|s| s.card.as_ref()))
            .chain(self.effects.iter().filter_map(|e| if let Effect::Bury { card } = e { Some(card) } else { None }))
    }
    fn jc054_source(&self, source: &SourceSnapshot, actor: usize) -> RuleResult<()> {
        if source.card.definition != "JC054" || source.card.id.is_empty()
            || actor >= self.players.len() || source.card.owner >= self.players.len() || source.card.controller != actor
            || source.region.is_none_or(|r| r >= self.regions.len())
            || source.source_region_instance.as_ref().is_none_or(|id| id.is_empty())
            || source.attachment_host_instance.is_some() || source.observed_death.is_some()
            || source.observed_mobility_grant.is_some()
            || self.jc054_physical_cards().any(|c| c.id == source.card.id
                && (c.definition != source.card.definition || c.owner != source.card.owner)) {
            return Err("JC054冻结来源、座位、拥有者或地区身份不符".into());
        }
        Ok(())
    }
    fn jc054_operation(&self, source: &SourceSnapshot, actor: usize, ability_key: &str, op: &Op) -> RuleResult<()> {
        self.jc054_source(source, actor)?;
        if source.card.face_down || source.play_source.is_some() {
            return Err("JC054强制触发必须来自当时明置的角色".into());
        }
        let (event_id, turn) = observation(op).ok_or("JC054运行触发缺少真实有限事件")?;
        if event_id.is_empty() || turn != self.turn { return Err("JC054事件身份或回合不符".into()); }
        match op {
            Op::JC054ConcealBurst { observation: Some(o) } if ability_key == BURST => {
                let baseline = o.baseline.as_ref().map_or(source.card.controller, |b| b.controller);
                if baseline >= self.players.len() || o.baseline.as_ref().is_some_and(|b| b.target_instance != source.card.id)
                    || o.baseline_eliminated && !self.players[baseline].eliminated {
                    return Err("JC054潜伏前的操控基线无效".into());
                }
                let mut expected = source.card.clone();
                expected.id = o.successor.id.clone(); expected.face_down = true;
                expected.time_markers = 0; expected.lock_markers = 0;
                expected.damage = 0; expected.wounds = 0; expected.shield = 0;
                expected.controller = if o.baseline_eliminated { source.card.owner } else { baseline };
                let successor_still_hidden = self.regions.iter().any(|r| !r.vacant
                    && r.cards.iter().any(|c| c.id == o.successor.id && c.face_down));
                if expected.id.is_empty() || expected.id == source.card.id || !same(&expected, &o.successor)
                    || self.jc054_physical_cards().any(|c| c.id == source.card.id
                        || c.id == o.successor.id && (c.owner != o.successor.owner || c.definition != "JC054" || !successor_still_hidden)) {
                    return Err("JC054潜伏必须保留朝向、清理旧实体并生成真实暗藏者".into());
                }
            }
            Op::JC054EndHide { .. } if ability_key == END && self.jc054_end_step_turn == Some(turn) => {}
            _ => return Err("JC054触发键和有限程序或结束时机不符".into()),
        }
        Ok(())
    }
    fn jc054_validate_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? { return Ok(()); }
        if !own_ability(&d.ability) && d.source.card.definition != "JC054" { return Ok(()); }
        if !own_ability(&d.ability) {
            self.jc054_source(&d.source, d.actor)?;
            if d.source.card.face_down || d.source.play_source.is_some() {
                return Err("JC054获授奖励必须来自当时明置的在场角色".into());
            }
            let expected = self.jc054_granted(&d.ability.key, &d.source)?;
            if !same(&expected, &d.ability) { return Err("JC054获授触发被改写".into()); }
            return Ok(());
        }
        validate_ability(&d.source.card.definition, &d.ability)?;
        let [op] = d.ability.ops.as_slice() else { return Err("JC054强制触发程序数量无效".into()); };
        self.jc054_operation(&d.source, d.actor, &d.ability.key, op)
    }
    fn jc054_granted(&self, ability_key: &str, source: &SourceSnapshot) -> RuleResult<AbilitySpec> {
        let region = source.source_region_instance.as_deref().ok_or("JC054获授奖励缺少地区身份")?;
        match ability_key {
            "renown" => Ok(crate::renown::renown_ability(region)),
            "jc089-combat-glory" => Ok(rules::jc089_combat_glory_ability(region)),
            _ => Err("JC054未知获授程序".into()),
        }
    }
    pub(crate) fn jc054_observe_hide(&mut self, source: SourceSnapshot, successor: Card,
        baseline: Option<ControlBaseline>, baseline_eliminated: bool, event: Option<&str>) {
        let event_id = event.map(str::to_owned).unwrap_or_else(|| self.id());
        let mut ability = definition().abilities[0].clone();
        ability.ops = vec![Op::JC054ConcealBurst { observation: Some(ConcealObservation {
            event_id, turn: self.turn, successor, baseline, baseline_eliminated,
        }) }];
        self.effects.push_back(Effect::Declare { declaration: Declaration { actor: source.card.controller, source, ability } });
    }
    pub(crate) fn jc054_begin_end_step(&mut self) {
        if self.jc054_end_step_turn == Some(self.turn) { return; }
        self.jc054_end_step_turn = Some(self.turn);
        let sources = self.regions.iter().enumerate().filter(|(_, r)| !r.vacant)
            .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .filter(|(_, c)| c.definition == "JC054" && !c.face_down && !self.players[c.controller].eliminated)
            .map(|(r, c)| self.source_snapshot(c, Some(r))).collect::<Vec<_>>();
        if sources.is_empty() { return; }
        let event_id = self.id();
        for source in sources {
            let mut ability = definition().abilities[1].clone();
            ability.ops = vec![Op::JC054EndHide { observation: Some(EndStepObservation { event_id: event_id.clone(), turn: self.turn }) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: source.card.controller, source, ability } });
        }
        self.jc054_sort_batch(&event_id);
    }
    /// Sort only declarations from this observation, preserving unrelated queue positions.
    pub(crate) fn jc054_sort_batch(&mut self, event_id: &str) {
        let mut declarations = self.effects.iter().filter_map(|e| match e {
            Effect::Declare { declaration: d } if declaration_event(d) == Some(event_id) => Some(d.clone()), _ => None,
        }).collect::<Vec<_>>();
        declarations.sort_by_key(|d| (self.team(d.actor) != self.first_team, d.actor, d.source.card.id.clone()));
        let mut ordered = declarations.into_iter();
        for e in &mut self.effects {
            if matches!(e, Effect::Declare { declaration: d } if declaration_event(d) == Some(event_id)) {
                *e = Effect::Declare { declaration: ordered.next().unwrap() };
            }
        }
    }
    fn jc054_push_trigger(&mut self, d: Declaration) {
        let label = format!("{}：{}", catalog::card("JC054").name, d.ability.label);
        let frame = self.make_frame(d.actor, d.source, &d.ability, vec![], vec![], None);
        self.dispatch_frame(frame, label, None, ResponsePolicy::Respondable);
    }
    fn jc054_order_options(&self, ds: &[Declaration]) -> Vec<ChoiceOption> {
        ds.iter().map(|d| {
            let mut o = self.option(&d.source.card, d.actor, d.source.region, None);
            o.label = format!("{} · 地区{} · #{}", d.ability.label, d.source.region.unwrap() + 1, d.source.card.id);
            o
        }).collect()
    }
    fn jc054_offer_order(&mut self, ds: Vec<Declaration>) {
        let options = self.jc054_order_options(&ds);
        self.choice(ds[0].actor, ORDER_KIND, ORDER_TITLE.into(), options, 1, 1, None,
            ChoiceResolution::JC054TriggerOrder { declarations: ds });
        self.pending.as_mut().unwrap().choice.description = ORDER_DESCRIPTION.into();
    }
    pub(crate) fn jc054_declare(&mut self, d: Declaration) -> RuleResult<bool> {
        self.jc054_validate_declaration(&d)?;
        if !own_ability(&d.ability) { return Ok(false); }
        if self.players[d.actor].eliminated { return Ok(true); }
        let mut ds = vec![d];
        while self.effects.front().is_some_and(|e| matches!(e, Effect::Declare { declaration: next }
            if next.actor == ds[0].actor && next.ability.key == ds[0].ability.key
                && declaration_event(next) == declaration_event(&ds[0]))) {
            if let Some(Effect::Declare { declaration: next }) = self.effects.pop_front() {
                self.jc054_validate_declaration(&next)?; ds.push(next);
            }
        }
        if ds.len() > 1 { self.jc054_offer_order(ds); } else { self.jc054_push_trigger(ds.remove(0)); }
        Ok(true)
    }
    pub(crate) fn jc054_choose_order(&mut self, mut ds: Vec<Declaration>, a: &Action, selected: &[String]) -> RuleResult<()> {
        if a.kind != "choose" || selected.len() != 1 || a.card_id.is_some() || a.target_id.is_some()
            || a.region.is_some() || a.ability_id.is_some() || a.option.is_some() || a.top.is_some()
            || a.bottom.is_some() || a.allocations.is_some() || a.cost_selected.is_some() || a.deck_draft.is_some() {
            return Err("JC054强制触发排序只接受一项，不可跳过或附带其他行动".into());
        }
        let index = ds.iter().position(|d| d.source.card.id == selected[0]).ok_or("JC054排序来源已失效")?;
        self.jc054_push_trigger(ds.remove(index));
        if ds.len() > 1 { self.jc054_offer_order(ds); }
        else if let Some(d) = ds.pop() { self.jc054_push_trigger(d); }
        Ok(())
    }
    pub(crate) fn jc054_reject_optional(&self, d: &Declaration) -> RuleResult<()> {
        self.jc054_validate_declaration(d)?;
        if own_ability(&d.ability) { return Err("JC054强制触发不可改为可选发动".into()); }
        Ok(())
    }
    pub(crate) fn validate_jc054_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? || !related(f) { return Ok(()); }
        self.jc054_source(&f.source, f.actor)?;
        if f.frame_id.is_empty() || !f.targets.is_empty() { return Err("JC054帧身份或多余目标无效".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if f.cursor != 0 || !f.steps.is_empty() || f.chosen_region != f.source.region
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some())
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)]
                    if ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("JC054只准入真实付费派遣／现身空程序".into());
            }
            return Ok(());
        }
        if f.source.card.face_down || f.source.play_source.is_some() || f.chosen_region.is_some()
            || !f.already_paid.is_empty() || f.cursor > 1 || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted) {
            return Err("JC054触发帧的费用、游标或来源不符".into());
        }
        if intrinsic(f) {
            let [step] = f.steps.as_slice() else { return Err("JC054强制触发只能执行一个固定步骤".into()); };
            if step.context != f.actor { return Err("JC054执行上下文不符".into()); }
            self.jc054_operation(&f.source, f.actor, &f.ability_key, &step.op)
        } else {
            let expected = self.jc054_granted(&f.ability_key, &f.source)?;
            let steps = expected.ops.into_iter().map(|op| Step { context: f.actor, op }).collect::<Vec<_>>();
            if !same(&steps, &f.steps) { return Err("JC054获授奖励程序被改写".into()); }
            Ok(())
        }
    }
    pub(crate) fn jc054_resolve(&mut self, f: &ResolutionFrame, op: &Op) -> RuleResult<()> {
        self.jc054_operation(&f.source, f.actor, &f.ability_key, op)?;
        match op {
            Op::JC054ConcealBurst { .. } => {
                if let Some(region) = f.source.region.filter(|r| self.regions.get(*r).is_some_and(|reg|
                    !reg.vacant && f.source.source_region_instance.as_ref() == Some(&reg.card.id))) {
                    let allocations = self.region_characters(region).filter(|c| !c.face_down
                        && c.id != f.source.card.id && catalog::card(&c.definition).kind == "character")
                        .map(|c| (c.id.clone(), 1)).collect::<BTreeMap<_, _>>();
                    self.damage(allocations)?;
                }
            }
            Op::JC054EndHide { .. } => {
                if self.board(&f.source.card.id).is_some_and(|(_, c)| !c.face_down) {
                    self.hide_in_place(&f.source.card.id);
                }
            }
            _ => return Err("非JC054固定程序".into()),
        }
        Ok(())
    }
    pub(crate) fn validate_jc054_state(&self) -> RuleResult<()> {
        // Validate untriggered entities too: end-step observation indexes their
        // controllers before a declaration/frame exists to carry a witness.
        let mut instances = BTreeSet::new();
        let mut occurrences = BTreeMap::<&str, usize>::new();
        for c in self.jc054_physical_cards() {
            *occurrences.entry(&c.id).or_default() += 1;
            if c.definition == "JC054" {
                if c.id.is_empty() || c.owner >= self.players.len() || c.controller >= self.players.len() {
                    return Err("JC054实体身份、拥有者或操控席位无效".into());
                }
                instances.insert(c.id.as_str());
            }
        }
        if instances.iter().any(|id| occurrences.get(id).copied().unwrap_or(0) != 1) {
            return Err("JC054实体UID不能跨实体或区域重复".into());
        }
        if self.jc054_end_step_turn.is_some_and(|t| t != self.turn)
            || self.window == Some(Window::End) && self.jc054_end_step_turn != Some(self.turn) {
            return Err("JC054结束步骤观察回合标记无效".into());
        }
        let mut events = BTreeSet::new();
        let mut record = |d: &Declaration| -> RuleResult<()> {
            self.jc054_validate_declaration(d)?;
            if own_ability(&d.ability) && !events.insert((d.actor, d.source.card.id.clone(), d.ability.key.clone(), declaration_event(d).unwrap().to_owned())) {
                return Err("JC054同一历史触发不能在多个容器重复".into());
            }
            Ok(())
        };
        let frames = self.stack.iter().filter_map(|s| s.frame.as_ref()).chain(self.effects.iter().filter_map(|e|
            if let Effect::Frame { frame } = e { Some(frame.as_ref()) } else { None }))
            .chain(self.pending.as_ref().and_then(|p| if let ChoiceResolution::Frame { frame, .. } = &p.resolution { Some(frame.as_ref()) } else { None })).collect::<Vec<_>>();
        let relevant = !instances.is_empty() || frames.iter().any(|f| related(f))
            || self.effects.iter().any(|e| matches!(e,Effect::Declare { declaration: d }
                if d.source.card.definition == "JC054" || own_ability(&d.ability)))
            || self.pending.as_ref().is_some_and(|p| match &p.resolution {
                ChoiceResolution::JC054TriggerOrder { .. } => true,
                ChoiceResolution::Declare { declaration: d, .. } => d.source.card.definition == "JC054" || own_ability(&d.ability),
                _ => false,
            });
        let end_context = self.window == Some(Window::End)
            || self.win_contexts.iter().any(|c| c.resume_window == Window::End);
        if relevant && self.status != "finished" && self.jc054_end_step_turn.is_some() != end_context {
            return Err("JC054结束观察标记与当前步骤或嵌套赢区恢复链不符".into());
        }
        for s in &self.stack {
            if let Some(f) = &s.frame {
                if s.card.as_ref().is_some_and(|c| c.definition == "JC054") && f.source.card.definition != "JC054" {
                    return Err("JC054堆栈实体与冻结来源归属不符".into());
                }
                self.validate_jc054_frame(f)?;
                if related(f) && !crate::mobility::related_frame(f) {
                    if s.id != f.frame_id || s.controller != f.actor || s.target.is_some() || f.cursor != 0
                        || !matches!(f.guard, GuardState::Unchecked)
                        || frames.iter().filter(|other| other.frame_id == f.frame_id).count() != 1 {
                        return Err("JC054未结算堆栈外壳或重复帧无效".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if s.card.as_ref().is_none_or(|c| {
                            let mut expected=f.source.card.clone();
                            expected.id=c.id.clone();expected.time_markers=0;expected.lock_markers=0;expected.face_down=false;
                            c.id == f.source.card.id || !same(c,&expected)
                        })
                            || s.deploy_region != f.source.region || s.reveal != (f.ability_key == "reveal") {
                            return Err("JC054派遣／现身事务卡不符".into());
                        }
                    } else if s.card.is_some() || s.deploy_region.is_some() || s.reveal { return Err("JC054触发不能携带事务卡".into()); }
                    if intrinsic(f) {
                        let mut ability = definition().abilities[if f.ability_key == BURST { 0 } else { 1 }].clone();
                        ability.ops = f.steps.iter().map(|s| s.op.clone()).collect();
                        record(&Declaration { actor: f.actor, source: f.source.clone(), ability })?;
                    }
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "JC054")
                || definition().abilities.iter().any(|a| s.label == format!("{}：{}", catalog::card("JC054").name, a.label)) {
                return Err("JC054堆栈缺少冻结帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration } => record(declaration)?,
                Effect::Frame { frame } => {
                    self.validate_jc054_frame(frame)?;
                    if related(frame) { return Err("JC054不能绕过响应队列或重放已执行步骤".into()); }
                }
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::JC054TriggerOrder { declarations: ds } => {
                    if ds.len() < 2 { return Err("JC054排序待选必须包含多个触发".into()); }
                    for d in ds { record(d)?; }
                    if ds.iter().any(|d| !own_ability(&d.ability) || d.actor != p.seat || d.ability.key != ds[0].ability.key
                        || declaration_event(d) != declaration_event(&ds[0]))
                        || p.seat >= self.players.len() || self.players[p.seat].eliminated || p.choice.id.is_empty()
                        || p.choice.player_id != player_id(p.seat) || p.choice.kind != ORDER_KIND || p.choice.title != ORDER_TITLE
                        || p.choice.description != ORDER_DESCRIPTION || p.choice.min != Some(1) || p.choice.max != Some(1)
                        || p.choice.allow_decline != Some(false) || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty()
                        || !same(&p.choice.options, &self.jc054_order_options(ds)) {
                        return Err("JC054强制排序来源、选项或选择者不符".into());
                    }
                }
                ChoiceResolution::Declare { declaration, .. } => { self.jc054_reject_optional(declaration)?; }
                ChoiceResolution::Frame { frame, .. } if related(frame) => return Err("JC054固定原子程序没有续体待选".into()),
                _ => {}
            }
        }
        Ok(())
    }
}
