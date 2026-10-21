//! Five complete red prints. Choices use the native declaration/frame protocol.
use crate::{catalog, engine::RuleResult, model::*, rules::{self, *}};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const CARDS: &[&str] = &["JC051", "JC055", "JZ32", "JZ34", "JZ41"];
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RedObservation {
    pub event_id: String,
    pub turn: u32,
    pub card: Card,
}

#[cfg(test)]
mod host_fixture_tests {
    use super::*;
    use crate::unit_support::{field, game, restore};

    #[test]
    fn next20_red_jc051_selected_attachment_host_algorithm_fixture() {
        // JC048/JZ41 are explicitly selected-attachment fixtures for the host
        // algorithm, never presented as badges or added as new catalog entries.
        let mut g = game(); let source = field(&mut g, "JC051", 0);
        let (r,c) = g.in_play(&source).unwrap();
        let spec = definition("JC051").unwrap().abilities[0].clone();
        let mut f = g.make_frame(0,g.source_snapshot(c,r),&spec,vec![],vec![],None);
        f.frame_id=g.id(); f.cursor=1; f.guard=GuardState::Accepted;
        let region_attachment=g.make_card("JC048",0);
        assert_eq!(g.jc051_host_options(&f,&region_attachment).len(),g.regions.len());
        let character_attachment=g.make_card("JZ41",0);
        assert_eq!(g.jc051_host_options(&f,&character_attachment).len(),1);
        let other=field(&mut g,"JC125",1);
        assert_eq!(g.jc051_host_options(&f,&character_attachment).len(),2);
        g.board_mut(&other).unwrap().face_down=true;
        assert_eq!(g.jc051_host_options(&f,&character_attachment).len(),1);
        assert!(g.jc051_finish_attach(&f,&character_attachment,"region:0").is_err());
        let old=region_attachment.id.clone(); g.players[0].deck.push(region_attachment.clone());
        g.jc051_finish_attach(&f,&region_attachment,"region:1").unwrap();
        assert_eq!(g.attachments[0].host_id,g.regions[1].card.id);
        assert_ne!(g.attachments[0].card.id,old); restore(&mut g);
        g.regions.iter_mut().for_each(|r| r.cards.clear());
        assert!(g.jc051_host_options(&f,&character_attachment).is_empty());
        let selected=character_attachment.id.clone(); g.players[0].deck.push(character_attachment);
        let random=g.random; g.jc051_choose_badge(&f,&[selected.clone()]).unwrap();
        assert!(g.players[0].deck.iter().any(|c| c.id==selected)); assert!(g.players[0].hand.is_empty());
        assert_ne!(g.random,random); assert!(g.pending.is_none()); restore(&mut g);
    }
}
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JC051Badge | Op::JC055PrepareTime { .. } | Op::JC055Discount
        | Op::JZ32Hourglass | Op::JZ34SacrificeDestroy | Op::JZ34DeathInfluence { .. }
        | Op::JZ41DamageWound { .. } => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted { exhausted, ready, .. } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn related_ability(a: &AbilitySpec) -> bool {
    a.key.starts_with("next20-red-") || matches!(a.event, Some(Event::JZ34OtherDeath | Event::JZ41HostDamage)) || owns(&a.ops) || a.modes.iter().any(|m| owns(&m.ops))
        || a.costs.iter().any(|c| matches!(c, Cost::JC055RemoveTime))
        || a.targets.iter().any(|t| matches!(t.predicate, Some(TargetPredicate::JC055CharacterOrAttachment | TargetPredicate::JZ34DestroyedRegion)))
}
fn related(f: &ResolutionFrame) -> bool {
    CARDS.contains(&f.source.card.definition.as_str()) || f.ability_key.starts_with("next20-red-")
        || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op)))
        || f.already_paid.iter().any(|p| matches!(p, PaidCost::JC055Time(_)))
}
pub(crate) fn definition(id: &str) -> Option<Definition> {
    let trigger = |key: &str, label: &str, ops, event| rules::ability(key, label, Timing::Fast, vec![], vec![], ops, Some(event));
    Some(match id {
        "JC051" => rules::with_abilities(vec![trigger("next20-red-jc051-badge", "现身触发：检索徽章附属，结附于合理目标并洗牌", vec![Op::JC051Badge], Event::Reveal)]),
        "JC055" => {
            let mut t = rules::target(Zone::Board, EntityKind::Any, Relation::Any, Range::Anywhere);
            t.predicate = Some(TargetPredicate::JC055CharacterOrAttachment);
            let prep = rules::ability("next20-red-jc055-prepare", "准备步骤触发：在目标角色或附属上放置一个时间标志", Timing::Fast,
                vec![], vec![t], vec![Op::JC055PrepareTime { event_id: None }], Some(Event::Prepare));
            let mut action = rules::ability("next20-red-jc055-discount", "行动：移除本方X个时间标志，下张打出的灾难牌减费X", Timing::Standard,
                vec![Cost::JC055RemoveTime], vec![], vec![Op::JC055Discount], None);
            action.activation_only = true;
            rules::with_abilities(vec![prep, action])
        }
        "JZ32" => {
            let mut a = rules::ability("next20-red-jz32-hourglass", "行动1、牺牲：将本方墓地的诅咒沙漏结附于原地区", Timing::Standard,
                vec![Cost::Assets(1), Cost::SacrificeSource], vec![], vec![Op::JZ32Hourglass], None);
            a.activation_only = true; rules::with_abilities(vec![a])
        }
        "JZ34" => {
            let mut a = trigger("next20-red-jz34-death", "另一个角色死去：在目标已毁灭地区放置一个本方势力标志", vec![Op::JZ34DeathInfluence { observation: None }], Event::JZ34OtherDeath);
            let mut t = rules::target(Zone::Region, EntityKind::Any, Relation::Any, Range::Anywhere);
            t.predicate = Some(TargetPredicate::JZ34DestroyedRegion); a.targets.push(t);
            rules::with_abilities(vec![trigger("next20-red-jz34-enter", "进场触发：牺牲另一个本地区的角色，然后毁灭本地区", vec![Op::JZ34SacrificeDestroy], Event::Enter), a])
        }
        "JZ41" => {
            let mut d = rules::attachment_definition(rules::target(Zone::Board, EntityKind::Character, Relation::Any, Range::Anywhere));
            d.abilities.push(trigger("next20-red-jz41-forecast", "进场触发：预测2", vec![Op::Forecast { player: PlayerRef::Actor, count: 2 }], Event::Enter));
            d.abilities.push(trigger("next20-red-jz41-wound", "受此结附的角色受到伤害：对该角色造成1点创伤", vec![Op::JZ41DamageWound { observation: None }], Event::JZ41HostDamage)); d
        }
        _ => return None,
    })
}
fn canonical_ability(id: &str, a: &AbilitySpec) -> RuleResult<AbilitySpec> {
    let mut expected = definition(id).and_then(|d| d.abilities.into_iter().find(|e| e.key == a.key)).ok_or("非准入红牌程序")?;
    match a.ops.as_slice() {
        [Op::JZ34DeathInfluence { observation }] if a.key == "next20-red-jz34-death" => expected.ops = vec![Op::JZ34DeathInfluence { observation: observation.clone() }],
        [Op::JZ41DamageWound { observation }] if a.key == "next20-red-jz41-wound" => expected.ops = vec![Op::JZ41DamageWound { observation: observation.clone() }],
        [Op::JC055PrepareTime { event_id }] if a.key == "next20-red-jc055-prepare" => expected.ops = vec![Op::JC055PrepareTime { event_id: event_id.clone() }],
        _ => {},
    }
    Ok(expected)
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if !related_ability(a) && !(id == "JZ41" && a.key == "attach") { return Ok(()); }
    if !same(a, &canonical_ability(id, a)?) { return Err("红牌须保留完整固定程序、费用与目标".into()); }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if let Some(expected) = definition(id) {
        if !same(d, &expected) { return Err("红牌整张定义被修改".into()); }
    } else if d.abilities.iter().any(related_ability) { return Err("红牌有限程序不能移植".into()); }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if !CARDS.contains(&id) { return Ok(()); }
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new(&a.kind) },
        "reveal" => Action { card_id: a.card_id.clone(), ..Action::new("reveal") },
        "play" if id == "JZ41" => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), ..Action::new("play") },
        "activate" if matches!(id, "JC055" | "JZ32") => Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), ..Action::new("activate") },
        _ => return Err("红牌操作种类不符".into()),
    };
    if *a != expected { return Err("红牌声明包含非准入载荷".into()); }
    Ok(())
}
impl Game {
    pub(crate) fn jz32_destroyed(&self, c: &Card) -> bool {
        c.definition == "JZ32" && !c.face_down && self.board(&c.id).is_some_and(|(r, _)| self.regions[r].destroyed && !self.regions[r].vacant)
    }
    pub(crate) fn jc051_aura(&self, c: &Card) -> u32 {
        if c.face_down || catalog::card(&c.definition).color != "红" || catalog::card(&c.definition).kind != "character" { return 0; }
        self.regions.iter().flat_map(|r| &r.cards).filter(|s| s.definition == "JC051" && !s.face_down && s.id != c.id && s.controller == c.controller).count() as u32
    }
    fn red_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if !CARDS.contains(&c.definition.as_str()) || c.id.is_empty() || actor >= self.players.len() || c.controller != actor || c.owner >= self.players.len()
            || s.region.is_some_and(|r| r >= self.regions.len()) || s.observed_death.is_some() || s.observed_mobility_grant.is_some() || s.observed_exposure.is_some()
            || s.region.is_some() != s.source_region_instance.is_some()
            || s.source_region_instance.as_ref().is_some_and(|id| id.is_empty() || id == &c.id
                || self.regions.iter().enumerate().any(|(r,region)| region.card.id == *id && Some(r) != s.region))
            || (c.definition != "JZ41" && s.attachment_host_instance.is_some())
            || self.in_play(&c.id).is_some_and(|(_, now)| now.definition != c.definition || now.owner != c.owner) {
            return Err("红牌冻结来源、地区或持有者不符".into());
        }
        self.mobility_immutable_identity(c)?;
        Ok(())
    }
    fn red_observation(&self, actor: usize, s: &SourceSnapshot, ops: &[Op]) -> RuleResult<Option<(String, String)>> {
        let (o, death) = match ops {
            [Op::JZ34DeathInfluence { observation }] => (observation.as_ref().ok_or("死亡观察缺失")?, true),
            [Op::JZ41DamageWound { observation }] => (observation.as_ref().ok_or("伤害观察缺失")?, false),
            _ => return Ok(None),
        };
        let c = &o.card;
        if o.event_id.is_empty() || o.event_id == c.id || o.event_id == s.card.id || o.turn != self.turn || c.id.is_empty() || c.face_down
            || c.owner >= self.players.len() || c.controller >= self.players.len()
            || catalog::catalog().cards.iter().find(|d| d.id == c.definition).is_none_or(|d| d.kind != "character")
            || actor != s.card.controller || s.card.face_down || s.play_source.is_some()
            || (death && (s.card.definition != "JZ34" || c.id == s.card.id || s.region.is_none()))
            || (!death && (s.card.definition != "JZ41" || s.attachment_host_instance.as_ref() != Some(&c.id) || c.damage == 0))
            || self.in_play(&c.id).is_some_and(|(_, now)| now.definition != c.definition || now.owner != c.owner) {
            return Err("红牌事件观察与原角色实例不符".into());
        }
        self.mobility_immutable_identity(c)?;
        Ok(Some((s.card.id.clone(), o.event_id.clone())))
    }
    pub(crate) fn validate_next20_red_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? || (!CARDS.contains(&d.source.card.definition.as_str()) && !related_ability(&d.ability)) { return Ok(()); }
        self.red_source(d.actor, &d.source)?;
        let expected = match d.ability.key.as_str() {
            "renown" if d.source.card.definition != "JZ41" => crate::renown::renown_ability(d.source.source_region_instance.as_deref().unwrap_or("")),
            "jc089-combat-glory" if d.source.card.definition != "JZ41" => rules::jc089_combat_glory_ability(d.source.source_region_instance.as_deref().unwrap_or("")),
            _ => canonical_ability(&d.source.card.definition,&d.ability)?,
        };
        if !same(&d.ability,&expected) { return Err("红牌声明必须是原完整触发或已准入奖励".into()); }
        if d.ability.event.is_none() || d.source.card.face_down || d.source.play_source.is_some() || d.source.region.is_none()
            || (d.source.card.definition == "JZ41" && d.source.attachment_host_instance.as_ref().is_none_or(|id| id.is_empty())) { return Err("红牌触发声明来源或事件错误".into()); }
        self.red_observation(d.actor, &d.source, &d.ability.ops)?;
        Ok(())
    }
    pub(crate) fn validate_next20_red_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? || !related(f) { return Ok(()); }
        if crate::jc035::preparation_frame(f) { return self.validate_jc035_frame(f); }
        self.red_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() || f.cursor > f.steps.len() || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked) && !(f.ability_key == "attach" && matches!(f.guard, GuardState::Accepted)))
            || (f.cursor > 0 && !matches!(f.guard, GuardState::Accepted)) { return Err("红牌帧身份、守卫或游标错误".into()); }
        let printed = matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= catalog::card(&f.source.card.definition).cost as usize
            && ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len());
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if f.source.card.definition == "JZ41" || !f.steps.is_empty() || !f.targets.is_empty() || !printed || f.source.region.is_none()
                || f.chosen_region != f.source.region || (f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play()))
                || (f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some())) { return Err("红牌出场帧错误".into()); }
            return Ok(());
        }
        if f.ability_key != "attach" && (f.source.region.is_none()
            || (f.source.card.definition == "JZ41" && f.source.attachment_host_instance.as_ref().is_none_or(|id| id.is_empty()))) {
            return Err("红牌场内能力缺少原地区或宿主快照".into());
        }
        let mut a = definition(&f.source.card.definition).and_then(|d| d.abilities.into_iter().find(|a| a.key == f.ability_key))
            .or_else(|| (f.ability_key == "renown" && f.source.card.definition != "JZ41").then(|| crate::renown::renown_ability(f.source.source_region_instance.as_deref().unwrap_or(""))))
            .or_else(|| (f.ability_key == "jc089-combat-glory" && f.source.card.definition != "JZ41").then(|| rules::jc089_combat_glory_ability(f.source.source_region_instance.as_deref().unwrap_or(""))))
            .ok_or("未知红牌帧")?;
        if matches!(f.ability_key.as_str(), "next20-red-jz34-death" | "next20-red-jz41-wound") {
            self.red_observation(f.actor, &f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
            a.ops = f.steps.iter().map(|s| s.op.clone()).collect(); validate_ability(&f.source.card.definition, &a)?;
        }
        let expected = a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect::<Vec<_>>();
        if !same(&expected, &f.steps) || f.source.card.face_down || f.source.play_source.is_some() || f.chosen_region.is_some()
            || f.targets.len() != a.targets.len() || f.targets.iter().zip(&a.targets).any(|(t, s)| !same(&t.spec, s) || t.id.is_empty()
                || t.public.instance_id != t.id || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some()
                || (s.zone == Zone::Region) != t.region_instance.is_some()
                || (s.zone == Zone::Region && t.id.strip_prefix("region:").and_then(|id| id.parse::<usize>().ok()).is_none_or(|r|
                    r >= self.regions.len() || t.id != format!("region:{r}") || t.public.kind != "region" || t.public.region != Some(r)
                    || t.public.owner.is_some() || t.public.controller.is_some() || t.region_instance.as_ref().is_none_or(|id| id.is_empty())))
                || (s.zone == Zone::Board && (t.public.kind != "character" || t.public.region.is_none() || t.public.owner.is_none() || t.public.controller.is_none()))) { return Err("红牌帧程序或目标错误".into()); }
        match f.ability_key.as_str() {
            "attach" if printed && f.source.region.is_none() && f.source.attachment_host_instance.is_none() => {},
            "next20-red-jc055-discount" => self.validate_jc055_paid(f)?,
            "next20-red-jz32-hourglass" => {
                if !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids), PaidCost::Sacrificed { old_instance, controller, owner }]
                    if ids.len() == 1 && !ids[0].is_empty() && old_instance == &f.source.card.id && *controller == f.actor && *owner == f.source.card.owner)
                    || f.source.region.is_none() { return Err("先知的支付与牺牲收据错误".into()); }
            }
            _ if a.event.is_some() && f.already_paid.is_empty() => {},
            _ => return Err("红牌费用收据错误".into()),
        }
        Ok(())
    }
    fn jc055_time_options(&self, actor: usize) -> Vec<ChoiceOption> {
        self.in_play_cards().into_iter().filter(|(_, c)| c.controller == actor && !c.face_down && c.time_markers > 0
            && matches!(catalog::card(&c.definition).kind.as_str(), "character" | "attachment"))
            .flat_map(|(_, c)| (0..c.time_markers).map(move |n| ChoiceOption { id: format!("{}/time/{n}", c.id),
                label: format!("{}的第{}个时间标志", catalog::card(&c.definition).name, n + 1), card: None })).collect()
    }
    pub(crate) fn jc055_begin_payment(&mut self, source: SourceSnapshot, a: &Action) -> RuleResult<()> {
        validate_action("JC055", a)?;
        let options = self.jc055_time_options(source.card.controller); let max = options.len();
        self.choice(source.card.controller, "jc055-time-payment", "选择移除的时间标志（可选择0个）".into(), options, 0, max, None,
            ChoiceResolution::JC055TimePayment { source }); Ok(())
    }
    pub(crate) fn jc055_finish_payment(&mut self, source: SourceSnapshot, a: &Action, selected: &[String]) -> RuleResult<()> {
        if *a != (Action { choice_id: a.choice_id.clone(), selected: Some(selected.to_vec()), ..Action::new("choose") }) { return Err("时间支付只能提交选项".into()); }
        let actor = source.card.controller;
        let options = self.jc055_time_options(actor);
        let mut counts = BTreeMap::<String, u32>::new();
        for id in selected {
            if !options.iter().any(|o| o.id == *id) { return Err("时间标志已失效".into()); }
            let (carrier, _) = id.rsplit_once("/time/").ok_or("时间标志标识错误")?;
            *counts.entry(carrier.into()).or_default() += 1;
        }
        let mut payments = vec![];
        for (id, n) in counts {
            let (region, c) = self.in_play(&id).ok_or("时间载体已离场")?;
            let remaining = c.time_markers.checked_sub(n).ok_or("时间不足")?;
            payments.push(TimeMarkerPayment { carrier: c.clone(), region, attachment_host: self.attachments.iter().find(|a| a.card.id == id).map(|a| a.host_id.clone()), remaining });
            self.board_mut(&id).ok_or("时间载体错误")?.time_markers = remaining;
        }
        let spec = definition("JC055").unwrap().abilities[1].clone();
        let f = self.make_frame(actor, source, &spec, vec![], vec![PaidCost::JC055Time(payments)], None);
        self.dispatch_frame(f, spec.label, None, ResponsePolicy::Respondable); Ok(())
    }
    fn validate_jc055_paid(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let [PaidCost::JC055Time(ps)] = f.already_paid.as_slice() else { return Err("JC055缺少时间支付收据".into()) };
        let mut ids = BTreeSet::new();
        for p in ps {
            let c = &p.carrier;
            if c.id.is_empty() || !ids.insert(&c.id) || c.face_down || c.controller != f.actor || c.owner >= self.players.len()
                || p.remaining >= c.time_markers || p.region.is_some_and(|r| r >= self.regions.len())
                || !matches!(catalog::card(&c.definition).kind.as_str(), "character" | "attachment")
                || (catalog::card(&c.definition).kind == "attachment") != p.attachment_host.is_some()
                || p.attachment_host.as_ref().is_some_and(|h| h.is_empty() || h == &c.id)
                || (c.id == f.source.card.id && (!same(c, &f.source.card) || p.region != f.source.region))
                || self.in_play(&c.id).is_some_and(|(_, now)| now.definition != c.definition || now.owner != c.owner) { return Err("JC055时间费用载体或数量错误".into()); }
        }
        Ok(())
    }
    pub(crate) fn jc055_discount(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        self.validate_jc055_paid(f)?;
        let [PaidCost::JC055Time(ps)] = f.already_paid.as_slice() else { unreachable!() };
        let amount = ps.iter().try_fold(0u32, |n, p| n.checked_add(p.carrier.time_markers - p.remaining)).ok_or("时间费用数量溢出")?;
        if amount > 0 { self.modifiers.push(CostModifier { actor: f.actor, filter: CardFilter::JC055Disaster, amount, expires_turn: self.turn, uses: 1, paid_reveal: false }); }
        Ok(())
    }
    pub(crate) fn jc055_prepare_time(&mut self, f: &ResolutionFrame) {
        if let Some(t) = f.targets.first().filter(|t| self.valid_bound_target(f.actor, &f.source, t)) {
            if let Some(c) = self.board_mut(&t.id) { c.time_markers = c.time_markers.saturating_add(1); }
        }
    }
    fn red_attach(&mut self, actor: usize, card: Card, host: String) {
        let mut card = self.reset_zone_card(card); card.controller = actor;
        let id = card.id.clone(); let definition = card.definition.clone();
        self.attachments.push(Attachment { card, host_id: host }); self.settle_controls();
        self.enter_triggers(actor, &definition, &id, false);
    }
    fn jc051_badge_options(&self, f: &ResolutionFrame) -> Vec<ChoiceOption> {
        self.players[f.actor].deck.iter().filter(|c| {
            let d = catalog::card(&c.definition); d.kind == "attachment" && d.subtypes.iter().any(|s| s == "徽章")
        }).map(|c| self.option(c, f.actor, None, None)).collect()
    }
    fn jc051_host_options(&self, f: &ResolutionFrame, badge: &Card) -> Vec<ChoiceOption> {
        let Some(a) = &rules::definition(&badge.definition).attachment else { return vec![]; };
        self.binding_options(f.actor, &self.source_snapshot(badge, None), &a.host)
    }
    pub(crate) fn jc051_search(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let options = self.jc051_badge_options(&f);
        if options.is_empty() { self.shuffle_player(f.actor); return Ok(()); }
        self.choice(f.actor, "search", "选择一张徽章附属".into(), options, 0, 1, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::JC051Badge }); Ok(())
    }
    pub(crate) fn jc051_choose_badge(&mut self, f: &ResolutionFrame, selected: &[String]) -> RuleResult<()> {
        let Some(id) = selected.first() else { self.shuffle_player(f.actor); return Ok(()); };
        let badge = self.players[f.actor].deck.iter().find(|c| c.id == *id).ok_or("徽章已离开牌库")?.clone();
        let hosts = self.jc051_host_options(f, &badge);
        if hosts.is_empty() { self.shuffle_player(f.actor); }
        else if hosts.len() == 1 { self.jc051_finish_attach(f, &badge, &hosts[0].id)?; }
        else {
            self.choice(f.actor, "target", "选择徽章的合理结附目标".into(), hosts, 1, 1, None,
                ChoiceResolution::Frame { frame: Box::new(f.clone()), choice: FrameChoice::JC051Host { badge } });
        }
        Ok(())
    }
    pub(crate) fn jc051_finish_attach(&mut self, f: &ResolutionFrame, badge: &Card, selected: &str) -> RuleResult<()> {
        if !self.jc051_host_options(f, badge).iter().any(|o| o.id == selected) { return Err("徽章宿主不合理".into()) }
        let pos = self.players[f.actor].deck.iter().position(|c| same(c, badge)).ok_or("徽章快照已失效")?;
        let host = selected.strip_prefix("region:").and_then(|s| s.parse::<usize>().ok()).map_or_else(|| selected.to_owned(), |r| self.regions[r].card.id.clone());
        let card = self.players[f.actor].deck.remove(pos); self.red_attach(f.actor, card, host);
        self.shuffle_player(f.actor); Ok(())
    }
    fn jz32_options(&self, f: &ResolutionFrame) -> Vec<ChoiceOption> {
        if self.original_region(&f.source).is_none() { return vec![]; }
        self.players[f.actor].graveyard.iter().filter(|c| c.definition == "JC048").map(|c| self.option(c, f.actor, None, None)).collect()
    }
    pub(crate) fn jz32_search(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let options = self.jz32_options(&f); if options.is_empty() { return Ok(()); }
        self.choice(f.actor, "graveyard", "选择本方墓地的诅咒沙漏".into(), options, 1, 1, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::JZ32Hourglass }); Ok(())
    }
    pub(crate) fn jz32_finish(&mut self, f: &ResolutionFrame, selected: &[String]) -> RuleResult<()> {
        let Some(r) = self.original_region(&f.source) else { return Ok(()); };
        let id = selected.first().ok_or("请选择诅咒沙漏")?;
        let pos = self.players[f.actor].graveyard.iter().position(|c| c.id == *id && c.definition == "JC048").ok_or("沙漏已离墓")?;
        let c = self.players[f.actor].graveyard.remove(pos); self.red_attach(f.actor, c, self.regions[r].card.id.clone()); Ok(())
    }
    fn jz34_sacrifice_options(&self, f: &ResolutionFrame) -> Vec<ChoiceOption> {
        self.original_region(&f.source).into_iter().flat_map(|r| self.regions[r].cards.iter().filter(|c|
            c.controller == f.actor && !c.face_down && c.id != f.source.card.id && catalog::card(&c.definition).kind == "character")
            .map(move |c| self.option(c, f.actor, Some(r), None))).collect()
    }
    pub(crate) fn jz34_enter(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let options = self.jz34_sacrifice_options(&f);
        if options.is_empty() { self.jz34_destroy(&f); return Ok(()); }
        self.choice(f.actor, "sacrifice", "牺牲另一个本地区的本方角色".into(), options, 1, 1, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::JZ34Sacrifice }); Ok(())
    }
    fn jz34_destroy(&mut self, f: &ResolutionFrame) {
        if let Some(r) = self.original_region(&f.source) { self.regions[r].destroyed = true; }
    }
    pub(crate) fn jz34_finish_sacrifice(&mut self, f: &ResolutionFrame, selected: &[String]) -> RuleResult<()> {
        if selected.len() != 1 || !self.jz34_sacrifice_options(f).iter().any(|o| o.id == selected[0]) { return Err("祭品不是另一个本地区的本方角色".into()) }
        self.remove_dead(&selected[0], RemovalCause::Sacrifice); self.jz34_destroy(f); Ok(())
    }
    pub(crate) fn jz34_observers(&self) -> Vec<SourceSnapshot> {
        self.regions.iter().enumerate().flat_map(|(r, reg)| reg.cards.iter().filter(|c| c.definition == "JZ34" && !c.face_down).map(move |c| self.source_snapshot(c, Some(r)))).collect()
    }
    pub(crate) fn jz34_observe_death(&mut self, observers: &[SourceSnapshot], dead: &Card) {
        if observers.iter().all(|s| s.card.id == dead.id) { return; }
        let observation = RedObservation { event_id: self.id(), turn: self.turn, card: dead.clone() };
        for s in observers.iter().filter(|s| s.card.id != dead.id) {
            let mut ability = definition("JZ34").unwrap().abilities[1].clone();
            ability.ops = vec![Op::JZ34DeathInfluence { observation: Some(observation.clone()) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: s.card.controller, source: s.clone(), ability } });
        }
    }
    pub(crate) fn jz41_observe_damage(&mut self, target: &str) {
        let Some((_, c)) = self.board(target).filter(|(_, c)| !c.face_down && catalog::card(&c.definition).kind == "character") else { return; };
        let card = c.clone();
        let sources = self.attachments.iter().filter(|a| a.card.definition == "JZ41" && !a.card.face_down && a.host_id == target && self.attachment_host_valid(a))
            .map(|a| self.source_snapshot(&a.card, self.attachment_region(a))).collect::<Vec<_>>();
        if sources.is_empty() { return; }
        let observation = RedObservation { event_id: self.id(), turn: self.turn, card };
        for source in sources {
            let mut ability = definition("JZ41").unwrap().abilities[2].clone();
            ability.ops = vec![Op::JZ41DamageWound { observation: Some(observation.clone()) }];
            self.effects.push_back(Effect::Declare { declaration: Declaration { actor: source.card.controller, source, ability } });
        }
    }
    pub(crate) fn jz41_wound(&mut self, f: &ResolutionFrame) {
        let [Step { op: Op::JZ41DamageWound { observation: Some(o) }, .. }] = f.steps.as_slice() else { return; };
        if let Some(c) = self.board_mut(&o.card.id).filter(|c| !c.face_down && c.definition == o.card.definition && c.owner == o.card.owner) { c.wounds = c.wounds.saturating_add(1); }
        else { return; }
        if let Some((r, c)) = self.board(&o.card.id) { self.emit_event(c.controller, self.source_snapshot(c, Some(r)), Event::ReceiveWound); }
        self.settle_deaths();
    }
    pub(crate) fn validate_next20_red_state(&self) -> RuleResult<()> {
        for m in self.modifiers.iter().filter(|m| matches!(m.filter, CardFilter::JC055Disaster)) {
            if m.actor >= self.players.len() || m.amount == 0 || m.uses > 1 || m.paid_reveal || m.expires_turn != self.turn { return Err("JC055灾难减费范围或有效期被改写".into()); }
        }
        let mut observations = BTreeSet::new(); let mut frames = BTreeSet::new();
        let mut remember = |actor: usize, source: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> {
            if let Some(key) = self.red_observation(actor, source, ops)? { if !observations.insert(key) { return Err("红牌事件观察重复执行".into()); } }
            Ok(())
        };
        for s in &self.stack {
            if let Some(f) = &s.frame {
                self.validate_next20_red_frame(f)?;
                if !related(f) || self.validated_cross_card_frame(f)? { continue; }
                if !frames.insert(&f.frame_id) || s.id != f.frame_id || s.controller != f.actor || s.target != f.targets.first().map(|t| t.id.clone())
                    || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) { return Err("红牌响应堆叠外壳错误".into()); }
                let transaction = matches!(f.ability_key.as_str(), "deploy" | "reveal" | "attach");
                if transaction != s.card.is_some() || s.card.as_ref().is_some_and(|c| c.definition != f.source.card.definition || c.owner != f.source.card.owner || c.controller != f.actor || c.face_down)
                    || s.reveal != (f.ability_key == "reveal") || s.deploy_region != (if matches!(f.ability_key.as_str(), "deploy" | "reveal") { f.chosen_region } else { None }) { return Err("红牌响应实体或出场地区错误".into()); }
                remember(f.actor, &f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
            } else if s.card.as_ref().is_some_and(|c| CARDS.contains(&c.definition.as_str())) { return Err("红牌缺少执行帧".into()); }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } => { self.validate_next20_red_declaration(d)?; remember(d.actor, &d.source, &d.ability.ops)?; }
                Effect::Frame { frame } if related(frame) && !self.validated_cross_card_frame(frame)? => return Err("红牌不能绕过响应堆叠".into()),
                _ => {},
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::JC055TimePayment { source } => {
                    self.red_source(p.seat, source)?;
                    let options = self.jc055_time_options(p.seat);
                    if self.status != "playing" || self.players[p.seat].eliminated || !self.stack.is_empty() || self.window != Some(Window::Action(self.team(p.seat))) || self.priority_team != self.team(p.seat)
                        || source.card.definition != "JC055" || source.card.face_down || source.play_source.is_some()
                        || self.in_play(&source.card.id).is_none_or(|(r,c)| !same(&self.source_snapshot(c,r), source)) { return Err("JC055支付前来源已变更".into()); }
                    self.red_pending(p, p.seat, "jc055-time-payment", "选择移除的时间标志（可选择0个）", &options, 0, options.len())?;
                }
                ChoiceResolution::Declare { declaration: d, stage } if CARDS.contains(&d.source.card.definition.as_str()) || related_ability(&d.ability) => {
                    self.validate_next20_red_declaration(d)?; remember(d.actor, &d.source, &d.ability.ops)?;
                    let targeted = !d.ability.targets.is_empty();
                    let options = if targeted { self.binding_options(d.actor, &d.source, &d.ability.targets[0]) }
                        else { vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }] };
                    if (targeted && !matches!(stage, DeclareChoice::Target)) || (!targeted && !matches!(stage, DeclareChoice::Accept)) { return Err("红牌触发选择阶段错误".into()); }
                    self.red_pending(p, d.actor, "trigger", &format!("{}：{}", catalog::card(&d.source.card.definition).name, d.ability.label), &options, 0, 1)?;
                }
                ChoiceResolution::Frame { frame: f, choice } if related(f) => {
                    self.validate_next20_red_frame(f)?; remember(f.actor, &f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?;
                    if !frames.insert(&f.frame_id) || f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) { return Err("红牌续体游标或帧重复".into()); }
                    let (key, kind, title, options, min, max) = match choice {
                        FrameChoice::JC051Badge => ("next20-red-jc051-badge", "search", "选择一张徽章附属", self.jc051_badge_options(f), 0, 1),
                        FrameChoice::JC051Host { badge } => {
                            if !self.jc051_badge_options(f).iter().any(|o| o.id == badge.id) || self.players[f.actor].deck.iter().find(|c| c.id == badge.id).is_none_or(|c| !same(c,badge)) { return Err("徽章选择快照被改写".into()); }
                            ("next20-red-jc051-badge", "target", "选择徽章的合理结附目标", self.jc051_host_options(f,badge), 1, 1)
                        }
                        FrameChoice::JZ32Hourglass => ("next20-red-jz32-hourglass", "graveyard", "选择本方墓地的诅咒沙漏", self.jz32_options(f), 1, 1),
                        FrameChoice::JZ34Sacrifice => ("next20-red-jz34-enter", "sacrifice", "牺牲另一个本地区的本方角色", self.jz34_sacrifice_options(f), 1, 1),
                        FrameChoice::Forecast { seat } if f.ability_key == "next20-red-jz41-forecast" && *seat == f.actor => {
                            let expected = self.players[*seat].deck.iter().take(2).map(|c| self.option(c,*seat,None,None)).collect::<Vec<_>>();
                            if p.choice.id.is_empty() || p.seat != *seat || p.choice.player_id != player_id(*seat) || p.choice.kind != "investigation" || p.choice.title != format!("预测 {}",expected.len()) || p.choice.allow_decline != Some(false) || !same(&p.choice.options,&expected)
                                || p.choice.min != Some(expected.len()) || p.choice.max != Some(expected.len()) || p.choice.amount != Some(expected.len() as u32) || !p.choice.preview_cards.is_empty() { return Err("衰败之环预测续体被改写".into()); }
                            return Ok(());
                        }
                        _ => return Err("红牌非准入续体".into()),
                    };
                    if f.ability_key != key { return Err("红牌续体不属于原程序".into()); }
                    self.red_pending(p,f.actor,kind,title,&options,min,max)?;
                }
                _ => {},
            }
        }
        Ok(())
    }
    fn red_pending(&self, p: &Pending, actor: usize, kind: &str, title: &str, options: &[ChoiceOption], min: usize, max: usize) -> RuleResult<()> {
        if p.choice.id.is_empty() || p.seat != actor || p.choice.player_id != player_id(actor) || p.choice.kind != kind || p.choice.title != title
            || p.choice.min != Some(min) || p.choice.max != Some(max) || p.choice.allow_decline != Some(min == 0) || p.choice.amount.is_some()
            || !p.choice.preview_cards.is_empty() || !same(&p.choice.options, &options.to_vec()) { return Err("红牌待选身份、选项或数量被改写".into()); }
        Ok(())
    }
}
