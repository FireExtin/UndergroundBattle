//! Fourth batch is isolated from the frozen ABI94 twenty-card candidate.
//! Only reviewed pilots are executable here; deck construction stays disabled.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const IMPLEMENTING: &[&str] = &["XG14", "XG15"];
const KEY: &str = "batch4-xg14-disaster";
const XG15_KEY: &str = "batch4-xg15-destroy-region";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DisasterPlay {
    pub event_id: String,
    pub turn: u32,
    pub played: SourceSnapshot,
    pub ability_key: String,
}
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition(id: &str) -> Option<Definition> {
    let mut d = match id {
        "XG14" => {
            let mut t = target(Zone::Region, EntityKind::Any, Relation::Any, Range::Anywhere);
            t.predicate = Some(TargetPredicate::Batch4DestroyedRegion);
            with_abilities(vec![ability(KEY, "打出灾难牌：目标已毁灭地区放置本方势力1", Timing::Fast,
                vec![], vec![t], vec![Op::XG14DisasterInfluence { observation: None }], Some(Event::DisasterPlayed))])
        },
        "XG15" => {
            let mut d = with_abilities(vec![ability(XG15_KEY, "现身：毁灭本地区", Timing::Fast,
                vec![], vec![], vec![Op::XG15DestroyOriginalRegion], Some(Event::Reveal))]);
            d.traits.cannot_be_equipped = true;
            d.modifiers.push(StaticModifier::XG15WaterDoubleIcons);
            d
        },
        _ => return None,
    };
    d.traits.chaos = true;
    Some(d)
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::XG14DisasterInfluence { .. } | Op::XG15DestroyOriginalRegion => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted { exhausted, ready, .. } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    a.key.starts_with("batch4-") || a.event == Some(Event::DisasterPlayed)
        || owns(&a.ops) || a.modes.iter().any(|m| owns(&m.ops))
        || a.targets.iter().any(|t| t.predicate == Some(TargetPredicate::Batch4DestroyedRegion))
}
fn frame_related(f: &ResolutionFrame) -> bool {
    IMPLEMENTING.contains(&f.source.card.definition.as_str()) || f.ability_key.starts_with("batch4-")
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
}
fn canonical(id: &str, a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition(id)?.abilities.into_iter().find(|p| p.key == a.key)?;
    if let ("XG14", [Op::XG14DisasterInfluence { observation }]) = (id, a.ops.as_slice()) {
        expected.ops = vec![Op::XG14DisasterInfluence { observation: observation.clone() }];
    }
    Some(expected)
}
fn reward(source: &SourceSnapshot, key: &str) -> Option<AbilitySpec> {
    let region = source.source_region_instance.as_deref()?;
    match key {
        "renown" => Some(crate::renown::renown_ability(region)),
        "jc089-combat-glory" => Some(crate::rules::jc089_combat_glory_ability(region)),
        _ => None,
    }
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if let Some(expected) = definition(id) {
        if !same(d, &expected) { return Err("第四批必须保留完整固定定义".into()); }
    } else if d.traits.chaos || d.abilities.iter().any(related) || d.modifiers.iter().any(|m| matches!(m, StaticModifier::XG15WaterDoubleIcons)) {
        return Err("第四批混乱或灾难程序不能移植".into());
    }
    Ok(())
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if related(a) && canonical(id, a).is_none_or(|expected| !same(a, &expected)) {
        return Err("第四批能力程序、来源或目标被改写".into());
    }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if !IMPLEMENTING.contains(&id) { return Ok(()); }
    let expected = match a.kind.as_str() {
        "deploy" | "reveal" => Action { card_id: a.card_id.clone(), ..Action::new(&a.kind) },
        "conceal" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new("conceal") },
        _ => return Err("本批混乱角色只有派遣、秘密派遣与现身操作".into()),
    };
    if *a != expected { return Err("混乱角色不能预选正面进场地区或附带其他载荷".into()); }
    Ok(())
}

impl Game {
    pub(crate) fn observe_batch4_disaster(&mut self, f: &ResolutionFrame) {
        if !catalog::card(&f.source.card.definition).subtypes.iter().any(|s| s == "灾难") { return; }
        let sources = self.regions.iter().enumerate().filter(|(_, r)| !r.vacant)
            .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .filter(|(_, c)| c.definition == "XG14" && !c.face_down && c.controller == f.actor && !self.players[c.controller].eliminated)
            .map(|(r, c)| self.source_snapshot(c, Some(r))).collect::<Vec<_>>();
        for source in sources {
            let mut a = definition("XG14").unwrap().abilities.remove(0);
            a.ops = vec![Op::XG14DisasterInfluence { observation: Some(DisasterPlay {
                event_id: f.frame_id.clone(), turn: self.turn, played: f.source.clone(), ability_key: f.ability_key.clone(),
            }) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: f.actor, source, ability: a } });
        }
    }
    fn batch4_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        if !IMPLEMENTING.contains(&s.card.definition.as_str()) || s.card.id.is_empty()
            || actor >= self.players.len() || s.card.controller != actor || s.card.owner >= self.players.len()
            || s.attachment_host_instance.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || s.region.is_some_and(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_some_and(|id| id.is_empty() || *id == s.card.id)
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r, id)| self.regions.iter().enumerate().any(|(other, live)| live.card.id == *id && other != r))
            || self.batch3_physical_cards().any(|c| c.id == s.card.id && self.board(&c.id).is_none() && s.play_source.is_none())
        { return Err("第四批冻结来源或原地区身份无效".into()); }
        self.mobility_immutable_identity(&s.card)
    }
    fn xg14_observation(&self, actor: usize, source: &SourceSnapshot, ops: &[Op]) -> RuleResult<()> {
        let [Op::XG14DisasterInfluence { observation: Some(o) }] = ops else { return Err("朽灭者缺少实际打出灾难观察".into()); };
        let p = &o.played; let c = &p.card;
        let d = catalog::catalog().cards.iter().find(|d| d.id == c.definition && catalog::runtime_implemented(d))
            .ok_or("灾难观察不是当前实现牌")?;
        let valid_play = if d.kind == "character" {
            o.ability_key == "deploy" && p.valid_character_play()
        } else {
            matches!(d.kind.as_str(), "spell" | "attachment") && p.play_source.is_none() && p.region.is_none()
                && crate::rules::definition(&d.id).abilities.iter().any(|a| a.key == o.ability_key && a.event.is_none() && !a.activation_only)
        };
        let event = o.event_id.strip_prefix('i').and_then(|s| s.parse::<u64>().ok());
        if c.face_down || c.controller != actor || c.owner >= self.players.len() || c.id.is_empty() || c.id == source.card.id
            || !d.subtypes.iter().any(|t| t == "灾难") || !valid_play || o.turn != self.turn
            || event.is_none_or(|n| n == 0 || n > self.sequence) || o.event_id == c.id || o.event_id == source.card.id
            || p.attachment_host_instance.is_some() || p.observed_death.is_some() || p.observed_exposure.is_some() || p.observed_mobility_grant.is_some()
            || self.batch3_physical_cards().any(|live| live.id == c.id)
            || self.stack.iter().any(|item| item.id == o.event_id && item.frame.as_ref().is_none_or(|f|
                f.actor != actor || f.ability_key != o.ability_key || !same(&f.source, p)))
        { return Err("朽灭者必须保留个人实际打出、事件及灾难类别".into()); }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_batch4_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? { return Ok(()); }
        if !IMPLEMENTING.contains(&d.source.card.definition.as_str()) && !related(&d.ability) { return Ok(()); }
        self.batch4_source(d.actor, &d.source)?;
        let expected = reward(&d.source, &d.ability.key).or_else(|| canonical(&d.source.card.definition, &d.ability));
        if d.source.card.face_down || d.source.play_source.is_some() || d.source.region.is_none()
            || d.source.source_region_instance.is_none() || expected.is_none_or(|a| !same(&a, &d.ability))
        { return Err("第四批触发声明的来源或完整程序不符".into()); }
        if d.ability.key == KEY { self.xg14_observation(d.actor, &d.source, &d.ability.ops)?; }
        Ok(())
    }
    pub(crate) fn validate_batch4_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? || !frame_related(f) { return Ok(()); }
        self.batch4_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() || f.cursor > f.steps.len() || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked))
            || (f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)) { return Err("第四批执行帧身份、游标或守卫不符".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || !f.targets.is_empty()
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= catalog::card(&f.source.card.definition).cost as usize && ids.iter().all(|id| !id.is_empty() && *id != f.source.card.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play() || f.source.region.is_some() || f.chosen_region.is_some())
                || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some() || f.source.region.is_none() || f.chosen_region != f.source.region)
            { return Err("混乱派遣或现身帧不能更改地区、类型或支付".into()); }
            return Ok(());
        }
        let [Step { op, context }] = f.steps.as_slice() else { return Err("朽灭者必须保留单个有限操作".into()); };
        if *context != f.actor { return Err("朽灭者操作环境必须保留原玩家".into()); }
        let a = if let Some(a) = reward(&f.source, &f.ability_key) {
            if !same(&a.ops, &vec![op.clone()]) { return Err("第四批获授奖励不能改写操作".into()); }
            a
        } else {
            let mut a = definition(&f.source.card.definition).unwrap().abilities.remove(0); a.ops = vec![op.clone()]; a
        };
        self.validate_batch4_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a.clone() })?;
        if f.ability_key != a.key || f.targets.len() != a.targets.len() || !f.already_paid.is_empty() || f.chosen_region.is_some() {
            return Err("朽灭者的目标或操作外壳不符".into());
        }
        if a.targets.is_empty() { return Ok(()); }
        if !same(&f.targets[0].spec, &a.targets[0]) { return Err("朽灭者目标定义不符".into()); }
        let t = &f.targets[0];
        let r = t.id.strip_prefix("region:").and_then(|v| v.parse::<usize>().ok()).filter(|r| *r < self.regions.len()).ok_or("朽灭者地区目标无效")?;
        if t.public.instance_id != t.id || t.public.kind != "region" || t.public.region != Some(r) || !t.public.valid || t.public.status != "valid"
            || t.public.invalid_reason.is_some() || t.public.owner.is_some() || t.public.controller.is_some()
            || t.region_instance.as_ref().is_none_or(|id| id.is_empty() || *id == f.source.card.id)
            || self.regions.iter().enumerate().any(|(i, region)| Some(&region.card.id) == t.region_instance.as_ref() && i != r)
        { return Err("朽灭者目标须保留原已毁灭地区实体".into()); }
        Ok(())
    }
    pub(crate) fn validate_batch4_state(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        let mut frames = BTreeSet::new();
        let mut remember = |s: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            if let [Op::XG14DisasterInfluence { observation: Some(o) }] = ops {
                if !seen.insert((s.card.id.clone(), o.event_id.clone())) { return Err("同次打出灾难不能重复触发".into()); }
            }
            Ok(())
        };
        for item in &self.stack {
            if let Some(f) = &item.frame {
                self.validate_batch4_frame(f)?;
                if IMPLEMENTING.contains(&f.source.card.definition.as_str()) && !self.validated_cross_card_frame(f)? {
                    if !frames.insert(&f.frame_id) || item.id != f.frame_id || item.controller != f.actor || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                        || item.card.is_some() != matches!(f.ability_key.as_str(), "deploy" | "reveal")
                        || item.target != f.targets.first().map(|t| t.id.clone())
                        || item.card.as_ref().is_some_and(|c| c.definition != f.source.card.definition || c.owner != f.source.card.owner || c.controller != f.actor || c.face_down)
                        || item.reveal != (f.ability_key == "reveal") || item.deploy_region != (if item.reveal { f.source.region } else { None })
                    { return Err("第四批响应堆叠外壳被改写".into()); }
                    remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
                }
            } else if item.card.as_ref().is_some_and(|c| IMPLEMENTING.contains(&c.definition.as_str())) { return Err("第四批堆叠缺少完整帧".into()); }
        }
        for e in &self.effects { match e {
            Effect::Declare { declaration } => { self.validate_batch4_declaration(declaration)?; remember(&declaration.source, &declaration.ability.ops)?; },
            Effect::Frame { frame } if frame_related(frame) && !self.validated_cross_card_frame(frame)? => return Err("第四批不能绕过响应堆叠".into()),
            _ => {},
        } }
        if let Some(p) = &self.pending { if let ChoiceResolution::Declare { declaration: d, stage } = &p.resolution {
            self.validate_batch4_declaration(d)?;
            if IMPLEMENTING.contains(&d.source.card.definition.as_str()) && !self.validated_cross_card_declaration(d)? {
                let expected = match (d.ability.targets.as_slice(), stage) {
                    ([slot], DeclareChoice::Target) => self.binding_options(d.actor, &d.source, slot),
                    ([], DeclareChoice::Accept) => vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }],
                    _ => return Err("第四批待选阶段与完整程序不符".into()),
                };
                if p.seat != d.actor || p.choice.id.is_empty() || p.choice.kind != "trigger"
                    || p.choice.player_id != player_id(d.actor) || p.choice.title != format!("{}：{}", catalog::card(&d.source.card.definition).name, d.ability.label)
                    || p.choice.min != Some(0) || p.choice.max != Some(1) || p.choice.allow_decline != Some(true)
                    || p.choice.description != "选择符合数量限制的选项，然后确认。" || self.status != "playing" || self.players[d.actor].eliminated
                    || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty() || !same(&p.choice.options, &expected) || expected.is_empty()
                { return Err("朽灭者目标选择须保留原操控者和原选项".into()); }
                remember(&d.source, &d.ability.ops)?;
            }
        } else if matches!(&p.resolution, ChoiceResolution::Frame { frame, .. } if frame_related(frame) && !self.validated_cross_card_frame(frame)?) {
            return Err("第四批当前有限程序不含结算中选择".into());
        } }
        Ok(())
    }
    pub(crate) fn xg14_influence(&mut self, f: &ResolutionFrame) {
        if let Some(t) = f.targets.first().filter(|t| self.valid_bound_target(f.actor, &f.source, t)) {
            let r = t.id.strip_prefix("region:").unwrap().parse::<usize>().unwrap();
            let team = self.team(f.actor); self.regions[r].influence[team] = self.regions[r].influence[team].saturating_add(1);
        }
    }
    pub(crate) fn xg15_destroy_region(&mut self, f: &ResolutionFrame) {
        if let Some(r) = self.original_region(&f.source) { self.regions[r].destroyed = true; }
    }
    pub(crate) fn xg15_double_icons(&self, c: &Card, region: usize, parts: (Icons, Icons, Icons)) -> (Icons, Icons, Icons) {
        if c.definition != "XG15" || c.face_down || !self.region_live(region)
            || !self.board(&c.id).is_some_and(|(r,_)| r == region)
            || !catalog::card(&self.regions[region].card.definition).subtypes.iter().any(|s| s == "水域")
        { return parts; }
        let double = |i: Icons| Icons { combat: i.combat.saturating_mul(2), investigation: i.investigation.saturating_mul(2), influence: i.influence.saturating_mul(2) };
        (double(parts.0), double(parts.1), double(parts.2))
    }
}
