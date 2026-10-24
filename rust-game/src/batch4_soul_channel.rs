//! XQ20 uses two ordered, pre-paid entity targets. Counter writes are one batch;
//! no target, payment or resolution engine is duplicated here.
use crate::{batch4_markers::MarkerKind, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "batch4-xq20-move-markers";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut t = target(Zone::Board, EntityKind::Any, Relation::Any, Range::Anywhere);
    t.predicate = Some(TargetPredicate::XQ20MarkerCarrier);
    let mut a = ability(KEY, "标准行动：移动全部指示物到同一操控者的另一个角色或附属", Timing::Standard,
        vec![], vec![t.clone(), t], vec![Op::XQ20MoveMarkers], None);
    a.play_only = true;
    with_abilities(vec![a])
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    if a.kind != "play" || a.ability_id.as_deref().is_some_and(|k| k != KEY)
        || a.selected.as_ref().is_none_or(|ids| ids.len() != 2 || ids[0] == ids[1])
        || *a != (Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), selected: a.selected.clone(), ..Action::new("play") }) {
        return Err("灵魂通道须在打出前按顺序指定两个不同目标，不能追加数量、费用或目标载荷".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn xq20_target(&self, source: &SourceSnapshot, c: &Card) -> bool {
        source.card.definition == "XQ20" && self.marker_move_carrier(&c.id)
    }
    pub(crate) fn xq20_action_candidates(&self, actor: usize, c: &Card) -> Vec<(Action, String)> {
        let source = self.source_snapshot(c, None);
        let d = definition(); let options = self.binding_options(actor, &source, &d.abilities[0].targets[0]);
        let mut out = vec![];
        for from in &options { for to in &options {
            if self.prepare_marker_move(&from.id, &to.id).is_ok() {
                out.push((Action { card_id: Some(c.id.clone()), ability_id: Some(KEY.into()),
                    selected: Some(vec![from.id.clone(), to.id.clone()]), ..Action::new("play") },
                    format!("灵魂通道：{} [{}] → {} [{}]", from.label, from.id, to.label, to.id)));
            }
        } }
        out
    }
    pub(crate) fn xq20_bind(&self, actor: usize, source: &SourceSnapshot, a: &AbilitySpec, action: &Action) -> RuleResult<Vec<BoundTarget>> {
        validate_action(action)?;
        let ids = action.selected.as_ref().unwrap();
        self.prepare_marker_move(&ids[0], &ids[1])?;
        ids.iter().zip(&a.targets).map(|(id, spec)| {
            if !self.valid_binding(actor, source, spec, id) { return Err("灵魂通道目标不满足当前角色或附属条件".into()); }
            Ok(BoundTarget { id: id.clone(), public: self.public_target(actor, source, spec, id), spec: spec.clone(), region_instance: None })
        }).collect()
    }
    /// A response may change control without changing either UID. The pair
    /// condition must pass before the shared guard consumes any shield.
    pub(crate) fn xq20_pair_current(&self, f: &ResolutionFrame) -> bool {
        f.targets.len() == 2 && self.prepare_marker_move(&f.targets[0].id, &f.targets[1].id).is_ok()
    }
    pub(crate) fn xq20_resolve(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        let prepared = self.prepare_marker_move(&f.targets[0].id, &f.targets[1].id)?;
        self.commit_marker_move_counters(prepared)
    }
    pub(crate) fn validate_xq20_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let s = &f.source; let c = &s.card; let d = definition(); let a = &d.abilities[0];
        if c.definition != "XQ20" || c.id.is_empty() || c.face_down || c.owner >= self.players.len()
            || f.actor >= self.players.len() || c.controller != f.actor || c.damage != 0
            || MarkerKind::ALL.into_iter().any(|k| k.count(c) != 0)
            || s.region.is_some() || s.source_region_instance.is_some() || s.attachment_host_instance.is_some()
            || s.play_source.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some()
            || s.observed_mobility_grant.is_some() || s.death_echo.is_some()
            || f.frame_id.is_empty() || f.frame_id == c.id || f.ability_key != KEY || f.chosen_region.is_some()
            || f.cursor > 1 || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
            || !same(&f.steps, &vec![Step { context: f.actor, op: Op::XQ20MoveMarkers }])
            || f.targets.len() != 2 || f.targets[0].id == f.targets[1].id
            || self.batch3_physical_cards().any(|live| live.id == c.id) {
            return Err("灵魂通道的来源、双目标、阶段或完整程序不符".into());
        }
        self.mobility_immutable_identity(c)?;
        for (t, spec) in f.targets.iter().zip(&a.targets) {
            if t.id.is_empty() || t.id == c.id || t.id == f.frame_id || !same(&t.spec, spec)
                || t.region_instance.is_some() || t.public.instance_id != t.id || !t.public.valid
                || t.public.status != "valid" || t.public.invalid_reason.is_some()
                || !matches!(t.public.kind.as_str(), "character" | "attachment")
                || t.public.region.is_some_and(|r| r >= self.regions.len())
                || t.public.owner.as_deref().is_none_or(|id| !self.players.iter().any(|p| player_id(p.seat) == id))
                || t.public.controller.as_deref().is_none_or(|id| !self.players.iter().any(|p| player_id(p.seat) == id)) {
                return Err("灵魂通道必须保留原实体目标与公开摘要".into());
            }
        }
        if f.targets[0].public.controller != f.targets[1].public.controller
            || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= 2
                && ids.iter().all(|id| !id.is_empty() && *id != c.id && *id != f.frame_id && f.targets.iter().all(|t| t.id != *id))
                && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
            return Err("灵魂通道须保留同一原操控者及一次印刷费用收据".into());
        }
        Ok(())
    }
    pub(crate) fn validate_xq20_state(&self) -> RuleResult<()> {
        let mut seen = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame { if f.source.card.definition == "XQ20" {
                self.validate_xq20_frame(f)?;
                if !seen.insert(&f.frame_id) || item.id != f.frame_id || item.controller != f.actor || item.reveal
                    || item.deploy_region.is_some() || item.target.as_ref() != Some(&f.targets[0].id) || f.cursor != 0
                    || !matches!(f.guard, GuardState::Unchecked) || item.card.as_ref().is_none_or(|c| {
                        let mut expected = f.source.card.clone(); expected.id = c.id.clone();
                        c.id.is_empty() || c.id == f.source.card.id || c.id == f.frame_id || *c != expected
                    }) {
                    return Err("灵魂通道响应外壳与原程序不符".into());
                }
            } }
        }
        // The single counter operation does not pause for another player choice.
        Ok(())
    }
}
