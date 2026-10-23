//! The red Clockmakers dependency: fixed construction and both paid actions.
//! Uses the shared target guard, cost receipts and simultaneous departure wave.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;

pub(crate) const ADD: &str = "clock-msjc13-add-time";
pub(crate) const SWEEP: &str = "clock-msjc13-time-sweep";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn time_target() -> TargetSlotSpec {
    let mut t = target(Zone::Board, EntityKind::Any, Relation::Any, Range::Anywhere);
    t.predicate = Some(TargetPredicate::ClockCharacterOrAttachment);
    t
}
pub(crate) fn society_definition() -> Definition {
    let mut add = ability(ADD, "行动2，横置：目标角色或附属放时间1", Timing::Standard,
        vec![Cost::Assets(2), Cost::ExhaustSource], vec![time_target()], vec![Op::ClockAddTime], None);
    let mut sweep = ability(SWEEP, "行动3，横置：消灭所有时间至少3的公开角色或附属，每个直接消灭抓1", Timing::Standard,
        vec![Cost::Assets(3), Cost::ExhaustSource], vec![], vec![Op::ClockSweep], None);
    add.activation_only = true;
    sweep.activation_only = true;
    with_abilities(vec![add, sweep])
}
pub(crate) fn society() -> crate::society::SocietyDefinition {
    let mut card: catalog::CardDefinition = serde_json::from_value(serde_json::json!({
        "id":"MSJC13", "name":"计时者", "kind":"society", "type":"秘社/秘密教派/分支",
        "subtypes":["秘密教派","分支"], "color":"红", "society":"计时者", "unique":true,
        "supported":false,
        "text":"持续：构筑时你的牌组中，非红且非中立派系的牌不能超过12张。行动2，横置：在目标角色或附属上放置一个时间标志。行动3，横置：消灭所有其上有三个或更多时间标志的角色和附属，然后每有一个角色或附属因此法被消灭，就抓一张牌。"
    })).expect("pinned complete MSJC13 original");
    card.abilities = society_definition().abilities.iter().map(|a| catalog::AbilitySummary {
        key:a.key.clone(), label:a.label.clone(), timing:"standard".into(), costs:a.costs.clone(), triggered:false,
    }).collect();
    crate::society::SocietyDefinition {
        card, subtitle:"终末的阴谋家".into(), printed_cost:None, starting_hand:6,
        deck_constraints:vec![crate::society::SocietyDeckConstraint::ClockmakersOffColorMaximum],
        unresolved_abilities:Default::default(),
    }
}
fn owns(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::ClockAddTime | Op::ClockSweep => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
        Op::IfTargetExhausted { exhausted, ready, .. } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
        _ => false,
    })
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    crate::batch4_clock_spell::validate_ability(id,a)?;
    if id=="WM037" { return Ok(()); }
    let relevant = id == "MSJC13" || a.key.starts_with("clock-msjc13-") || owns(&a.ops)
        || a.modes.iter().any(|m| owns(&m.ops))
        || a.targets.iter().chain(a.modes.iter().flat_map(|m| &m.targets))
            .any(|t| t.predicate == Some(TargetPredicate::ClockCharacterOrAttachment));
    if relevant && (id != "MSJC13" || !society_definition().abilities.iter().any(|b| same(a,b))) {
        return Err("计时者只能使用完整的两项印刷行动，时间程序不能移植".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if id=="WM037" && !same(d,&crate::batch4_clock_spell::definition()) { return Err("戴森完整定义被改写".into()); }
    if id == "MSJC13" && !same(d,&society_definition()) { return Err("计时者完整定义被改写".into()); }
    for a in &d.abilities { validate_ability(id,a)?; }
    Ok(())
}
pub(crate) fn validate_action(id: &str, a: &Action) -> RuleResult<()> {
    if id != "MSJC13" { return Ok(()); }
    let expected = Action { card_id:a.card_id.clone(), ability_id:a.ability_id.clone(), target_id:a.target_id.clone(), ..Action::new("activate") };
    if *a != expected || !matches!(a.ability_id.as_deref(),Some(ADD|SWEEP))
        || (a.ability_id.as_deref()==Some(SWEEP) && a.target_id.is_some()) {
        return Err("计时者行动只能提交原秘社、能力及所需目标".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn clock_time_target(&self, s:&SourceSnapshot, c:&Card) -> bool {
        matches!(s.card.definition.as_str(),"MSJC13"|"WM037") && (!c.face_down || crate::batch4_plans::secret(c))
            && self.marker_move_carrier(&c.id)
    }
    pub(crate) fn clock_add_time(&mut self, f:&ResolutionFrame) {
        if let Some(c)=f.targets.first().and_then(|t|self.board_mut(&t.id)) {
            c.time_markers=c.time_markers.saturating_add(1);
        }
    }
    pub(crate) fn clock_sweep(&mut self, f:&ResolutionFrame) -> RuleResult<()> {
        // Freeze qualifying physical instances before any host leaves. Only
        // qualifying objects count; an unqualified collateral child does not.
        let deaths=self.in_play_cards().into_iter().filter(|(_,c)| self.marker_move_carrier(&c.id) && self.effective_time_markers(c)>=3)
            .map(|(r,c)|(c.id.clone(),RemovalCause::Destroy,self.source_snapshot(c,r))).collect::<Vec<_>>();
        let ids=deaths.iter().map(|(id,_,_)|id.clone()).collect::<BTreeSet<_>>();
        self.destroy_wave_as(f.actor,deaths);
        let destroyed=ids.iter().filter(|id|self.in_play(id).is_none()).count();
        self.draw(f.actor,destroyed)
    }
    pub(crate) fn validate_clock_frame(&self, f:&ResolutionFrame) -> RuleResult<()> {
        if f.source.card.definition=="WM037" || f.ability_key.starts_with("clock-wm037-") { return self.validate_wm037_frame(f); }
        if f.source.card.definition!="MSJC13" && !f.steps.iter().any(|s|owns(std::slice::from_ref(&s.op)))
            && !f.ability_key.starts_with("clock-msjc13-") { return Ok(()); }
        let a=society_definition().abilities.into_iter().find(|a|a.key==f.ability_key).ok_or("未知计时者行动")?;
        let c=&f.source.card;
        let paid=matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids),PaidCost::Exhausted(id)]
            if *id==c.id && ids.len()<=if a.key==ADD{2}else{3}
                && ids.iter().all(|id|self.mystic_instance_valid(id) && *id!=c.id)
                && ids.iter().collect::<BTreeSet<_>>().len()==ids.len());
        if c.definition!="MSJC13" || c.face_down || f.actor>=self.players.len() || c.owner!=f.actor || c.controller!=f.actor
            || !self.mystic_instance_valid(&c.id) || !self.mystic_instance_valid(&f.frame_id) || c.id==f.frame_id
            || f.source.region.is_some() || f.source.source_region_instance.is_some() || f.source.attachment_host_instance.is_some()
            || f.source.play_source.is_some() || f.source.death_echo.is_some() || f.source.observed_death.is_some()
            || f.source.observed_exposure.is_some() || f.source.observed_mobility_grant.is_some()
            || f.chosen_region.is_some() || f.cursor>a.ops.len()
            || f.cursor==0 && !matches!(f.guard,GuardState::Unchecked)
            || f.cursor>0 && !matches!(f.guard,GuardState::Accepted) || !paid
            || f.steps.len()!=a.ops.len() || f.steps.iter().zip(&a.ops).any(|(s,op)|s.context!=f.actor || !same(&s.op,op))
            || f.targets.len()!=a.targets.len() || f.targets.iter().zip(&a.targets).any(|(t,spec)|!same(&t.spec,spec)
                || t.id.is_empty() || t.public.instance_id!=t.id || !t.public.valid || t.public.status!="valid" || t.public.invalid_reason.is_some()) {
            return Err("计时者执行须保留原秘社、费用、目标及完整程序".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_clock_state(&self)->RuleResult<()> {
        self.validate_wm037_state()?;
        for item in &self.stack { if let Some(f)=&item.frame { self.validate_clock_frame(f)?; } }
        for e in &self.effects { if let Effect::Frame{frame}=e { self.validate_clock_frame(frame)?; } }
        if let Some(Pending{resolution:ChoiceResolution::Frame{frame,..},..})=&self.pending { self.validate_clock_frame(frame)?; }
        Ok(())
    }
}
