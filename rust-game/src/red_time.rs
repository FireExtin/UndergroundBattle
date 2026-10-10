//! Closed JZ30 printed time program; XQ18 is excluded pending a ruling. Zone/reset ownership stays in the
//! shared engine; no generic marker interpreter or configurable X is admitted.
use crate::{catalog, engine::RuleResult, model::*, rules::{self, *}};
use std::{collections::BTreeSet, sync::OnceLock};

const JZ30_ENTRY: &str = "entry-one-time";
const JZ30_DEATH: &str = "death-frozen-time-forecast";

fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}

fn admitted_card(c: &Card) -> RuleResult<()> {
    static IDS: OnceLock<BTreeSet<&'static str>> = OnceLock::new();
    let ids = IDS.get_or_init(|| catalog::catalog().cards.iter().map(|d| d.id.as_str())
        .chain(catalog::catalog().societies.iter().map(|d| d.card.id.as_str())).collect());
    if !ids.contains(c.definition.as_str()) {
        return Err("时间程序投影前发现非准入卡牌定义".into());
    }
    Ok(())
}

fn admitted_source(source: &SourceSnapshot) -> RuleResult<()> {
    admitted_card(&source.card)?;
    if let Some(dead) = &source.observed_death { admitted_card(&dead.card)?; }
    Ok(())
}

pub(crate) fn definition(id: &str) -> Option<Definition> {
    match id {
        "JZ30" => {
            let mut d = rules::with_abilities(vec![
                rules::ability(JZ30_ENTRY, "进场触发：放置一个时间标志", Timing::Fast,
                    vec![], vec![], vec![Op::JZ30AddOneTimeToOriginalSource], Some(Event::Enter)),
                rules::ability(JZ30_DEATH, "死亡触发：预测死亡时的时间标志数量", Timing::Fast,
                    vec![], vec![], vec![Op::JZ30ForecastFrozenTime], Some(Event::Death)),
            ]);
            d.traits.public = true;
            Some(d)
        }
        _ => None,
    }
}

pub(crate) fn contains_red_time_op(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JZ30AddOneTimeToOriginalSource | Op::JZ30ForecastFrozenTime => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => contains_red_time_op(body),
        Op::IfTargetExhausted { exhausted, ready, .. } => contains_red_time_op(std::slice::from_ref(exhausted))
            || contains_red_time_op(std::slice::from_ref(ready)),
        _ => false,
    })
}

fn relevant_ability(card: &str, a: &AbilitySpec) -> bool {
    definition(card).is_some() || matches!(a.key.as_str(), JZ30_ENTRY | JZ30_DEATH)
        || contains_red_time_op(&a.ops) || a.modes.iter().any(|m| contains_red_time_op(&m.ops))
}

// The shared declaration reducer flattens a selected mode before target choice.
// Admit exactly its resulting shape as well as the original printed program.
pub(crate) fn validate_red_time_ability(card: &str, a: &AbilitySpec) -> RuleResult<()> {
    if !relevant_ability(card, a) { return Ok(()); }
    let d = definition(card).ok_or("红色时间程序或目标谓词不能移植到其他卡牌")?;
    if d.abilities.iter().any(|canonical| {
        same(a, canonical) || canonical.modes.iter().any(|mode| {
            let mut flat = canonical.clone();
            flat.label = format!("{}：{}", flat.label, mode.label);
            flat.targets = mode.targets.clone();
            flat.ops = mode.ops.clone();
            flat.modes.clear();
            same(a, &flat)
        })
    }) || crate::jz22::runtime_ability_is_admitted(a) { return Ok(()); }
    Err("只准入完整印刷红色时间能力、已选模式或既有获授奖励".into())
}

pub(crate) fn validate_ability(card: &str, a: &AbilitySpec) -> RuleResult<()> {
    validate_red_time_ability(card, a)
}

pub(crate) fn validate_definition(card: &str, d: &Definition) -> RuleResult<()> {
    if let Some(expected) = definition(card) {
        if !same(d, &expected) { return Err("红色时间卡只能绑定完整印刷定义".into()); }
    } else if d.abilities.iter().any(|a| relevant_ability(card, a)) {
        return Err("红色时间能力或目标谓词不能移植到其他定义".into());
    }
    Ok(())
}

fn relevant_frame(f: &ResolutionFrame) -> bool {
    definition(&f.source.card.definition).is_some()
        || matches!(f.ability_key.as_str(), JZ30_ENTRY | JZ30_DEATH)
        || f.steps.iter().any(|s| contains_red_time_op(std::slice::from_ref(&s.op)))
}

impl Game {
    // from_persisted deserializes first. Check definitions before binding_options
    // or option can call catalog::card while projecting attacker-supplied state.
    fn validate_red_time_catalogue(&self) -> RuleResult<()> {
        for c in self.regions.iter().flat_map(|r| &r.cards).chain(self.regions.iter().map(|r| &r.card))
            .chain(&self.world).chain(self.players.iter().flat_map(|p| p.hand.iter().chain(&p.deck)
                .chain(&p.assets).chain(&p.graveyard).chain(&p.score_cards).chain(p.society_zone.card.iter())))
            .chain(self.attachments.iter().map(|a| &a.card)).chain(self.sealed_cards.iter().map(|s| &s.card))
            .chain(self.stack.iter().filter_map(|s| s.card.as_ref())) { admitted_card(c)?; }
        for item in &self.stack { if let Some(f) = &item.frame { admitted_source(&f.source)?; } }
        for e in &self.effects {
            match e {
                Effect::Bury { card } => admitted_card(card)?,
                Effect::Declare { declaration: d } => admitted_source(&d.source)?,
                Effect::Frame { frame: f } => admitted_source(&f.source)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration: d, .. } => admitted_source(&d.source)?,
                ChoiceResolution::Frame { frame: f, .. } => admitted_source(&f.source)?,
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn effective_time_markers(&self, c: &Card) -> u32 {
        self.attachments.iter().filter(|a| a.host_id == c.id && !a.card.face_down
            && self.attachment_host_valid(a))
            .fold(c.time_markers, |n, a| n.saturating_add(a.card.time_markers))
    }

    fn red_time_source(&self, actor: usize, source: &SourceSnapshot, face_down: bool) -> RuleResult<()> {
        let c = &source.card;
        if definition(&c.definition).is_none() || actor >= self.players.len()
            || c.owner >= self.players.len() || c.controller != actor || c.id.is_empty()
            || c.face_down != face_down || source.region.is_none_or(|r| r >= self.regions.len())
            || source.observed_death.is_some() || source.attachment_host_instance.is_some()
            || source.source_region_instance.as_ref().is_some_and(|id| id.is_empty())
            || (face_down && c.time_markers != 0)
        { return Err("红色时间能力的冻结来源、行动者或原地区无效".into()); }
        Ok(())
    }

    pub(crate) fn jz30_add_one_time(&mut self, f: &ResolutionFrame) {
        if let Some(c) = self.board_mut(&f.source.card.id)
            .filter(|c| c.definition == "JZ30" && !c.face_down) {
            c.time_markers = c.time_markers.saturating_add(1);
        }
    }

    // Called after the interpreter increments cursor and accepts the frame guard.
    // Returning true means the original frame is paused in the existing Forecast
    // continuation; returning false means X=0 or an empty deck completed at once.
    pub(crate) fn jz30_forecast_frozen_time(&mut self, f: &ResolutionFrame) -> RuleResult<bool> {
        self.validate_red_time_catalogue()?;
        self.validate_red_time_frame(f)?;
        let seat = f.actor;
        let count = f.source.card.time_markers as usize;
        let options = self.players[seat].deck.iter().take(count)
            .map(|c| self.option(c, seat, None, None)).collect::<Vec<_>>();
        if options.is_empty() { return Ok(false); }
        let n = options.len();
        self.choice(seat, "investigation", format!("预测 {n}"), options, n, n, Some(n as u32),
            ChoiceResolution::Frame { frame: Box::new(f.clone()), choice: FrameChoice::Forecast { seat } });
        Ok(true)
    }

    pub(crate) fn validate_red_time_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_red_time_catalogue()?;
        admitted_source(&f.source)?;
        if !relevant_frame(f) { return Ok(()); }
        let deploy = matches!(f.ability_key.as_str(), "deploy" | "reveal");
        self.red_time_source(f.actor, &f.source, deploy && f.ability_key == "reveal")?;
        if f.frame_id.is_empty() { return Err("红色时间帧缺少实例身份".into()); }
        if deploy {
            if !f.steps.is_empty() || !f.targets.is_empty() || f.cursor != 0
                || !matches!(f.guard, GuardState::Unchecked) || f.chosen_region != f.source.region
                || f.source.card.time_markers != 0
                || (f.ability_key == "deploy" && f.source.play_source != Some(PlaySource::Hand))
                || (f.ability_key == "reveal" && f.source.play_source.is_some())
            { return Err("红色时间角色仅准入真实手牌派遣或付费现身空程序".into()); }
            return Ok(());
        }
        if f.source.play_source.is_some() || !f.already_paid.is_empty() || f.chosen_region.is_some()
            || f.steps.len() != 1 || f.steps[0].context != f.actor {
            return Err("红色时间触发必须为完整免费原子程序".into());
        }
        let mut expected = definition(&f.source.card.definition).unwrap().abilities.into_iter()
            .filter(|a| a.key == f.ability_key).flat_map(|a| {
                if a.modes.is_empty() { vec![(a.targets, a.ops)] }
                else { a.modes.into_iter().map(|m| (m.targets, m.ops)).collect() }
            }).collect::<Vec<_>>();
        if let Op::PlaceInfluence { region_instance, amount: 1 } = &f.steps[0].op {
            if f.source.source_region_instance.as_ref() != Some(region_instance) {
                return Err("红色角色获授奖励不能改绑到其他地区实例".into());
            }
            let a = match f.ability_key.as_str() {
                "renown" => Some(crate::renown::renown_ability(region_instance)),
                "jc089-combat-glory" => Some(rules::jc089_combat_glory_ability(region_instance)),
                _ => None,
            };
            if let Some(a) = a.filter(|_| !region_instance.is_empty()) { expected.push((a.targets, a.ops)); }
        }
        if !expected.iter().any(|(targets, ops)| same(&f.steps[0].op, &ops[0])
            && targets.len() == f.targets.len() && targets.iter().zip(&f.targets).all(|(s, t)|
                same(s, &t.spec) && !t.id.is_empty() && t.region_instance.is_none()
                    && t.public.instance_id == t.id && matches!(t.public.kind.as_str(), "character" | "attachment"))) {
            return Err("红色时间帧的程序、目标形状或能力别名无效".into());
        }
        let forecast = f.source.card.definition == "JZ30" && f.ability_key == JZ30_DEATH;
        if !((f.cursor == 0 && matches!(f.guard, GuardState::Unchecked))
            || (forecast && f.cursor == 1 && matches!(f.guard, GuardState::Accepted))) {
            return Err("红色时间原子程序游标或预测续体守卫无效".into());
        }
        Ok(())
    }

    pub(crate) fn validate_red_time_state(&self) -> RuleResult<()> {
        self.validate_red_time_catalogue()?;
        let declaration = |d: &Declaration| -> RuleResult<()> {
            if !relevant_ability(&d.source.card.definition, &d.ability) { return Ok(()); }
            validate_red_time_ability(&d.source.card.definition, &d.ability)?;
            self.red_time_source(d.actor, &d.source, false)?;
            if d.source.play_source.is_some() { return Err("红色时间触发不能携带出牌区域".into()); }
            if let [Op::PlaceInfluence { region_instance, .. }] = d.ability.ops.as_slice() {
                if d.source.source_region_instance.as_ref() != Some(region_instance) {
                    return Err("红色角色获授奖励声明只能绑定冻结原地区".into());
                }
            }
            Ok(())
        };
        let atomic = |f: &ResolutionFrame| -> RuleResult<()> {
            self.validate_red_time_frame(f)?;
            if relevant_frame(f) && (f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)) {
                return Err("已执行红色时间程序只能保存在合法预测待选中".into());
            }
            Ok(())
        };
        for item in &self.stack {
            if let Some(f) = &item.frame {
                atomic(f)?;
                if relevant_frame(f) {
                    if item.id != f.frame_id || item.controller != f.actor
                        || item.target != f.targets.first().map(|t| t.id.clone()) {
                        return Err("红色时间堆栈容器与冻结帧不符".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if item.card.as_ref().is_none_or(|c| c.definition != f.source.card.definition
                            || c.owner != f.source.card.owner || c.controller != f.actor || c.time_markers != 0
                            || c.face_down) || item.deploy_region != f.source.region
                            || item.reveal != (f.ability_key == "reveal") {
                            return Err("红色时间角色派遣堆栈实体无效".into());
                        }
                    } else if item.card.is_some() || item.deploy_region.is_some() || item.reveal {
                        return Err("红色时间触发堆栈不能携带实体或派遣标志".into());
                    }
                }
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } => declaration(d)?,
                Effect::Frame { frame: f } => atomic(f)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration: d, stage } => {
                    declaration(d)?;
                    if relevant_ability(&d.source.card.definition, &d.ability) {
                        let a = &d.ability;
                        let expected = match stage {
                            DeclareChoice::Mode if !a.modes.is_empty() => a.modes.iter()
                                .map(|m| ChoiceOption { id: m.key.clone(), label: m.label.clone(), card: None }).collect(),
                            DeclareChoice::Target if a.modes.is_empty() && a.targets.len() == 1 =>
                                self.binding_options(d.actor, &d.source, &a.targets[0]),
                            DeclareChoice::Accept if a.modes.is_empty() && a.targets.is_empty() =>
                                vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }],
                            _ => return Err("红色时间声明阶段与能力形状不符".into()),
                        };
                        if expected.is_empty() || p.seat != d.actor || p.choice.player_id != format!("p{}", d.actor)
                            || p.choice.kind != "trigger" || p.choice.min != Some(0) || p.choice.max != Some(1)
                            || p.choice.allow_decline != Some(true) || p.choice.amount.is_some()
                            || !p.choice.preview_cards.is_empty() || !same(&p.choice.options, &expected) {
                            return Err("红色时间触发待选内容无效".into());
                        }
                    }
                }
                ChoiceResolution::Frame { frame: f, choice } if relevant_frame(f) => {
                    self.validate_red_time_frame(f)?;
                    if !matches!(choice, FrameChoice::Forecast { seat } if *seat == f.actor)
                        || f.source.card.definition != "JZ30" || f.ability_key != JZ30_DEATH
                        || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted)
                        || p.seat != f.actor || p.choice.player_id != format!("p{}", f.actor) {
                        return Err("JZ30预测待选的续体或选择者无效".into());
                    }
                    let options = self.players[f.actor].deck.iter().take(f.source.card.time_markers as usize)
                        .map(|c| self.option(c, f.actor, None, None)).collect::<Vec<_>>();
                    let n = options.len();
                    if n == 0 || p.choice.kind != "investigation" || p.choice.min != Some(n)
                        || p.choice.max != Some(n) || p.choice.amount != Some(n as u32)
                        || p.choice.allow_decline != Some(false) || !p.choice.preview_cards.is_empty()
                        || !same(&p.choice.options, &options) {
                        return Err("JZ30预测数量或真实牌库顶序不符".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
