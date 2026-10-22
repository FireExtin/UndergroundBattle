//! XG11 follows the manual's additive Raid rule. Personal Moon tiers and live
//! JC038 grants are frozen together once at actual face-up reveal entry.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub(crate) const KEY: &str = "batch4-xg11-reveal-raid";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevealRaid { pub event_id: String, pub turn: u32, pub secrecy: u32, pub grants: Vec<Card> }
fn amount(o: &RevealRaid) -> u32 { o.secrecy.min(3).saturating_add(o.grants.len() as u32) }
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    with_abilities(vec![ability(KEY, "现身：按本方隐秘门槛及实际授予合并袭击", Timing::Fast, vec![], vec![target(Zone::Board, EntityKind::Character, Relation::Any, Range::SourceRegion)], vec![Op::XG11RevealRaid { observation: None }], Some(Event::Reveal))])
}
pub(crate) fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    if a.key != KEY { return None; }
    let mut expected = definition().abilities.remove(0);
    if let [Op::XG11RevealRaid { observation }] = a.ops.as_slice() {
        expected.ops = vec![Op::XG11RevealRaid { observation: observation.clone() }];
        if let Some(o) = observation { expected.label = format!("袭击{}", amount(o)); }
    } Some(expected)
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new(&a.kind) },
        "reveal" => Action { card_id: a.card_id.clone(), ..Action::new("reveal") },
        _ => return Err("阴魂术士只有正常派遣、秘密派遣及现身操作".into()),
    };
    if *a != expected { return Err("阴魂术士不能预选袭击目标或添加额外载荷".into()); } Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.source.card.definition == "XG11")
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) { return Err("阴魂术士袭击待选只接受原目标或放弃".into()); } Ok(())
}
impl Game {
    pub(crate) fn xg11_reveal_ability(&mut self, source: &SourceSnapshot) -> Option<AbilitySpec> {
        let c = &source.card; if c.definition != "XG11" || c.face_down { return None; }
        let secrecy = self.players[c.controller].society_zone.secrecy;
        let grants = self.regions.iter().flat_map(|r| &r.cards).filter(|g| g.definition == "JC038" && !g.face_down && g.id != c.id && self.team(g.controller) == self.team(c.controller)).cloned().collect::<Vec<_>>();
        if secrecy == 0 && grants.is_empty() { return None; }
        let o = RevealRaid { event_id: self.id(), turn: self.turn, secrecy, grants };
        let mut a = definition().abilities.remove(0); a.label = format!("袭击{}", amount(&o)); a.ops = vec![Op::XG11RevealRaid { observation: Some(o) }]; Some(a)
    }
    fn xg11_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != "XG11" || c.id.is_empty() || c.owner >= self.players.len() || actor >= self.players.len() || c.controller != actor
            || s.death_echo.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() || s.attachment_host_instance.is_some()
            || s.region.is_none_or(|r| r >= self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|id| id.is_empty() || *id == c.id)
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r, id)| self.regions.iter().enumerate().any(|(other, live)| live.card.id == *id && r != other))
            || s.play_source.is_none() && self.batch3_physical_cards().any(|live| live.id == c.id) && self.board(&c.id).is_none()
        { return Err("阴魂术士须保留原角色、控制者及地区身份".into()); } self.mobility_immutable_identity(c)
    }
    fn xg11_observation(&self, actor: usize, source: &SourceSnapshot, ops: &[Op]) -> RuleResult<()> {
        let [Op::XG11RevealRaid { observation: Some(o) }] = ops else { return Err("阴魂术士缺少实际现身时的隐秘与袭击授予观察".into()); };
        let mut ids = BTreeSet::new();
        if o.turn != self.turn || amount(o) == 0 || o.event_id == source.card.id || o.event_id.strip_prefix('i').and_then(|v| v.parse::<u64>().ok()).is_none_or(|n| n == 0 || n > self.sequence)
            || o.grants.iter().any(|g| g.definition != "JC038" || g.face_down || g.id.is_empty() || g.id == source.card.id || g.owner >= self.players.len() || g.controller >= self.players.len() || self.team(g.controller) != self.team(actor) || !ids.insert(g.id.clone())) { return Err("阴魂术士袭击须保留本人隐秘计数与不同友方实际授予来源".into()); }
        for g in &o.grants { self.mobility_immutable_identity(g)?; } Ok(())
    }
    fn xg11_expected(&self, s: &SourceSnapshot, key: &str, ops: &[Op]) -> RuleResult<AbilitySpec> {
        if key == "renown" || key == "jc089-combat-glory" {
            let region = s.source_region_instance.as_deref().ok_or("阴魂术士奖励缺少原地区")?;
            return Ok(if key == "renown" { crate::renown::renown_ability(region) } else { crate::rules::jc089_combat_glory_ability(region) });
        }
        if key != KEY { return Err("阴魂术士未知程序".into()); }
        let mut a = definition().abilities.remove(0); a.ops = ops.to_vec();
        if let [Op::XG11RevealRaid { observation: Some(o) }] = ops { a.label = format!("袭击{}", amount(o)); } Ok(a)
    }
    pub(crate) fn validate_xg11_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.xg11_source(d.actor, &d.source)?;
        let a = self.xg11_expected(&d.source, &d.ability.key, &d.ability.ops)?;
        if d.source.card.face_down || d.source.play_source.is_some() || !same(&a, &d.ability) { return Err("阴魂术士须保留完整现身袭击或既有奖励".into()); }
        if a.key == KEY { self.xg11_observation(d.actor, &d.source, &a.ops)?; if canonical(&a).is_none_or(|b| !same(&a, &b)) { return Err("阴魂术士固定袭击程序或目标被改写".into()); } } Ok(())
    }
    pub(crate) fn validate_xg11_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.xg11_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() || f.frame_id == f.source.card.id || f.cursor > f.steps.len() || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)) || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted) { return Err("阴魂术士帧、游标或守卫无效".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || !f.targets.is_empty() || f.chosen_region != f.source.region
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= 4 && ids.iter().all(|id| !id.is_empty() && *id != f.source.card.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some()) { return Err("阴魂术士派遣/现身必须保留原地区及实际支付".into()); } return Ok(());
        }
        let ops = f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(); let a = self.xg11_expected(&f.source, &f.ability_key, &ops)?;
        self.validate_xg11_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a.clone() })?;
        if f.steps.iter().any(|s| s.context != f.actor) || !same(&ops, &a.ops) || !f.already_paid.is_empty() || f.chosen_region.is_some() || f.targets.len() != a.targets.len() || f.targets.iter().zip(&a.targets).any(|(t, s)| !same(&t.spec, s)) { return Err("阴魂术士袭击目标、程序或环境不完整".into()); }
        if f.ability_key == KEY { let t = &f.targets[0];
            if t.id.is_empty() || t.region_instance.is_some() || t.public.instance_id != t.id || t.public.kind != "character" || t.public.region != f.source.region || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some() || t.public.owner.is_none() || t.public.controller.is_none() { return Err("阴魂术士袭击须保留原地区的目标角色".into()); }
        } Ok(())
    }
    pub(crate) fn xg11_raid_resolve(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        let [Step { op: Op::XG11RevealRaid { observation: Some(o) }, .. }] = f.steps.as_slice() else { return Err("阴魂术士袭击结算缺少原观察".into()); };
        self.damage([(f.targets[0].id.clone(), amount(o))].into_iter().collect())
    }
    pub(crate) fn validate_xg11_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new(); let mut events = BTreeSet::new();
        let mut remember = |s: &SourceSnapshot, ops: &[Op]| -> RuleResult<()> { if let [Op::XG11RevealRaid { observation: Some(o) }] = ops { if !events.insert((s.card.id.clone(), o.event_id.clone())) { return Err("同次现身不能重复阴魂术士袭击".into()); } } Ok(()) };
        for item in &self.stack { if let Some(f) = &item.frame { if f.source.card.definition == "XG11" && !self.validated_cross_card_frame(f)? {
            self.validate_xg11_frame(f)?; remember(&f.source, &f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>())?; let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
            if !frames.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) || item.card.is_some() != play || item.reveal != (f.ability_key == "reveal") || item.deploy_region != (if play { f.source.region } else { None }) || item.target != f.targets.first().map(|t| t.id.clone())
                || item.card.as_ref().is_some_and(|c| c.definition != "XG11" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor) { return Err("阴魂术士堆叠外壳、实例或支付被改写".into()); }
        } } }
        for e in &self.effects { match e {
            Effect::Declare { declaration: d } if d.source.card.definition == "XG11" && !self.validated_cross_card_declaration(d)? => { self.validate_xg11_declaration(d)?; remember(&d.source, &d.ability.ops)?; },
            Effect::Frame { frame: f } if f.source.card.definition == "XG11" && !self.validated_cross_card_frame(f)? => return Err("阴魂术士不能保存绕过响应的直接帧".into()),
            _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration: d, stage } if d.source.card.definition == "XG11" && !self.validated_cross_card_declaration(d)? => {
                self.validate_xg11_declaration(d)?; remember(&d.source, &d.ability.ops)?;
                let (options, expected_stage) = if let Some(slot) = d.ability.targets.first() { (self.binding_options(d.actor, &d.source, slot), DeclareChoice::Target) } else { (vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }], DeclareChoice::Accept) };
                let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(), title: format!("{}：{}", catalog::card("XG11").name, d.ability.label), description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor), options, preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                if !same(stage, &expected_stage) || p.seat != d.actor || p.choice.id.is_empty() || expected.options.is_empty() || !same(&p.choice, &expected) { return Err("阴魂术士待选只接受原地区角色目标或放弃".into()); }
            },
            ChoiceResolution::Frame { frame: f, .. } if f.source.card.definition == "XG11" && !self.validated_cross_card_frame(f)? => return Err("阴魂术士袭击为原子操作，不允许额外续体".into()),
            _ => {},
        } } Ok(())
    }
}
