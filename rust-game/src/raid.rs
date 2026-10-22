//! JC038's additive, event-time raid grant reuses the reveal/target/response flow.
use crate::{catalog, engine::RuleResult, model::*, rules::{self, *}};
use std::collections::BTreeSet;
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
fn uses(ops: &[Op]) -> bool { ops.iter().any(|o| match o {
    Op::JC038GrantedRaid { .. } => true,
    Op::ForEachLivingPlayer(b) | Op::ForEachLivingPlayerFromActor(b) => uses(b),
    Op::IfTargetExhausted { exhausted, ready, .. } => uses(&[*exhausted.clone(), *ready.clone()]),
    _ => false,
}) }
pub(crate) fn related(a: &AbilitySpec) -> bool { a.key == "jc038-granted-raid" || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops)) }
pub(crate) fn related_frame(f: &ResolutionFrame) -> bool { f.ability_key == "jc038-granted-raid" || f.steps.iter().any(|s| uses(&[s.op.clone()])) }
fn ability(grants: Vec<Card>, printed: u32) -> AbilitySpec {
    rules::ability("jc038-granted-raid", &format!("袭击{}", printed + grants.len() as u32), Timing::Fast, vec![],
        vec![rules::target(Zone::Board, EntityKind::Character, Relation::Any, Range::SourceRegion)],
        vec![Op::JC038GrantedRaid { grants }], Some(Event::Reveal))
}
fn printed_amount(id: &str) -> u32 {
    rules::definition(id).abilities.iter().filter(|a| a.event == Some(Event::Reveal) && a.key.starts_with("raid-"))
        .filter_map(|a| match a.ops.as_slice() { [Op::DamageTarget { slot: 0, amount }] => Some(*amount), _ => None }).sum()
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id == "XG11" { return Err("阴魂术士袭击必须保留隐秘门槛与授予的完整观察".into()); }
    let [Op::JC038GrantedRaid { grants }] = a.ops.as_slice() else { return Err("授予袭击程序无效".into()); };
    if grants.is_empty() || !same(a, &ability(grants.clone(), printed_amount(id))) { return Err("授予袭击须合并完整印刷袭击及真实授予".into()); }
    Ok(())
}
impl Game {
    pub(crate) fn emit_jc038_raid(&mut self, source: &SourceSnapshot) -> bool {
        let c = &source.card;
        if c.definition == "XG11" { return false; }
        if c.face_down || catalog::card(&c.definition).kind != "character" || catalog::card(&c.definition).color != "蓝" { return false; }
        let grants = self.regions.iter().flat_map(|r| &r.cards).filter(|g| g.definition == "JC038"
            && !g.face_down && g.id != c.id && self.team(g.controller) == self.team(c.controller)).cloned().collect::<Vec<_>>();
        if grants.is_empty() { return false; }
        let mut source = source.clone();
        source.source_region_instance = source.region.map(|r| self.regions[r].card.id.clone());
        self.effects.push_back(Effect::Declare { declaration: Declaration { actor: c.controller, source,
            ability: ability(grants, printed_amount(&c.definition)) } });
        true
    }
    fn validate_raid_source(&self, actor: usize, s: &SourceSnapshot, grants: &[Card]) -> RuleResult<()> {
        let c = &s.card; let mut seen = BTreeSet::new();
        if c.definition == "XG11" || actor >= self.players.len() || c.controller != actor || c.owner >= self.players.len() || c.id.is_empty()
            || c.face_down || catalog::card(&c.definition).kind != "character" || catalog::card(&c.definition).color != "蓝"
            || s.region.is_none_or(|r| r >= self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|s| s.is_empty())
            || s.observed_death.is_some() || s.observed_mobility_grant.is_some() || s.attachment_host_instance.is_some() || s.play_source.is_some()
            || grants.is_empty() || grants.iter().any(|g| g.definition != "JC038" || g.face_down || g.id.is_empty() || g.id == c.id
                || !seen.insert(&g.id) || g.owner >= self.players.len() || g.controller >= self.players.len()
                || self.team(g.controller) != self.team(actor)) { return Err("袭击来源、原地区或友方授予观察无效".into()); }
        self.mobility_immutable_identity(c)?;
        for g in grants { self.mobility_immutable_identity(g)?; }
        Ok(())
    }
    fn validated_raid_declaration(&self, d: &Declaration) -> RuleResult<bool> {
        if !related(&d.ability) { return Ok(false); }
        validate_ability(&d.source.card.definition, &d.ability)?;
        let [Op::JC038GrantedRaid { grants }] = d.ability.ops.as_slice() else { unreachable!() };
        self.validate_raid_source(d.actor, &d.source, grants)?;
        Ok(true)
    }
    fn validated_raid_frame(&self, f: &ResolutionFrame) -> RuleResult<bool> {
        if !related_frame(f) { return Ok(false); }
        let [Step { context, op: Op::JC038GrantedRaid { grants } }] = f.steps.as_slice() else { return Err("授予袭击帧程序无效".into()); };
        self.validate_raid_source(f.actor, &f.source, grants)?;
        if f.frame_id.is_empty() || f.ability_key != "jc038-granted-raid" || *context != f.actor || f.cursor != 0
            || !matches!(f.guard, GuardState::Unchecked) || !f.already_paid.is_empty() || f.chosen_region.is_some() || f.targets.len() != 1 {
            return Err("授予袭击帧身份、游标、费用或目标数量无效".into());
        }
        let t = &f.targets[0];
        if !same(&t.spec, &ability(grants.clone(), printed_amount(&f.source.card.definition)).targets[0])
            || t.id.is_empty() || t.region_instance.is_some() || t.public.instance_id != t.id || !t.public.valid || t.public.status != "valid"
            || t.public.invalid_reason.is_some() || t.public.kind != "character" || t.public.region != f.source.region {
            return Err("授予袭击原地区目标摘要无效".into());
        }
        Ok(true)
    }
    // Existing per-card validators must admit independently validated programs
    // granted to arbitrary recipients, without accepting arbitrary source abilities.
    pub(crate) fn validated_cross_card_frame(&self, f: &ResolutionFrame) -> RuleResult<bool> {
        Ok(self.validated_mobility_frame(f)? || self.validated_raid_frame(f)?)
    }
    pub(crate) fn validated_cross_card_declaration(&self, d: &Declaration) -> RuleResult<bool> {
        Ok(self.validated_mobility_declaration(d)? || self.validated_raid_declaration(d)?)
    }
    pub(crate) fn jc038_raid_resolve(&mut self, f: &ResolutionFrame, grants: &[Card]) -> RuleResult<()> {
        let n = printed_amount(&f.source.card.definition).saturating_add(grants.len() as u32);
        self.damage([(f.targets[0].id.clone(), n)].into_iter().collect())
    }
    pub(crate) fn validate_raid_state(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame { if self.validated_raid_frame(f)? {
                if !seen.insert(f.source.card.id.clone()) || item.id != f.frame_id || item.controller != f.actor
                    || item.card.is_some() || item.deploy_region.is_some() || item.reveal || item.target.as_ref() != Some(&f.targets[0].id) {
                    return Err("授予袭击堆叠外壳或重复实例不符".into());
                }
            } }
        }
        for e in &self.effects { match e {
            Effect::Declare { declaration:d } if self.validated_raid_declaration(d)? => {
                if !seen.insert(d.source.card.id.clone()) { return Err("重复授予袭击事件".into()); }
            }
            Effect::Frame { frame } if related_frame(frame) => return Err("授予袭击不能绕过堆叠".into()),
            _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration:d, stage } if self.validated_raid_declaration(d)? => {
                if !seen.insert(d.source.card.id.clone()) || !matches!(stage, DeclareChoice::Target)
                    || p.seat != d.actor || p.choice.player_id != player_id(d.actor) || p.choice.kind != "trigger"
                    || p.choice.min != Some(0) || p.choice.max != Some(1) || p.choice.allow_decline != Some(true)
                    || !same(&p.choice.options, &self.binding_options(d.actor, &d.source, &d.ability.targets[0])) {
                    return Err("授予袭击待选无效".into());
                }
            }
            ChoiceResolution::Frame { frame, .. } if related_frame(frame) => return Err("袭击没有帧内待选".into()),
            _ => {},
        } }
        Ok(())
    }
}
