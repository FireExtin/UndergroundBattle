//! WM043: both fast modes and its optional, personal Sentinel shown-card effect.
use crate::{engine::RuleResult, model::*, mystic_contest::{ContestOutcome, MysticPhase}, rules::*};

pub(crate) const PLAY: &str = "wm043-precision-strike";
pub(crate) const SHOWN: &str = "wm043-shown-lock";
pub(crate) const CHOICE: &str = "wm043-shown-target";
const TITLE: &str = "精准打击：可以在目标角色或暗藏者上放置一个锁定标志";
fn post_config(id:&str)->Option<(&'static str,&'static str,&'static str)>{match id{
    "WM043"=>Some((SHOWN,CHOICE,TITLE)),
    "CQ18"=>Some((crate::import20_sentinel::SHOWN,"import20-cq18-shown-target","任秋雨：可以在目标角色或暗藏者上放置一个锁定标志")),_=>None}}
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn lock_target() -> TargetSlotSpec {
    target(Zone::Board, EntityKind::CharacterOrHidden, Relation::Any, Range::Anywhere)
}
pub(crate) fn definition() -> Definition {
    let mut locked = lock_target();
    locked.predicate = Some(TargetPredicate::WM043LockedTarget);
    let mut a = ability(PLAY, "快速行动：锁定或消灭已锁定目标", Timing::Fast,
        vec![], vec![], vec![], None);
    a.play_only = true;
    a.modes = vec![
        Mode { key: "lock".into(), label: "放置锁定1".into(), targets: vec![lock_target()], ops: vec![Op::WM043Lock] },
        Mode { key: "destroy".into(), label: "消灭已锁定目标".into(), targets: vec![locked], ops: vec![Op::Destroy(EntityRef::Target(0))] },
    ];
    with_abilities(vec![a])
}
pub(crate) fn related(a: &AbilitySpec) -> bool {
    a.key.starts_with("wm043-") || a.ops.iter().chain(a.modes.iter().flat_map(|m| &m.ops))
        .any(|op| matches!(op, Op::WM043Lock | Op::WM043ShownLock { .. }))
        || a.targets.iter().chain(a.modes.iter().flat_map(|m| &m.targets))
            .any(|t| t.predicate == Some(TargetPredicate::WM043LockedTarget))
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    let expected = definition().abilities.remove(0);
    if id == "WM043" && (same(a, &expected) || expected.modes.iter().any(|m| {
        let mut selected = expected.clone();
        selected.targets = m.targets.clone(); selected.ops = m.ops.clone(); selected.modes.clear();
        same(a, &selected)
    })) { Ok(()) } else { Err("精准打击须保留原卡的一项完整快速行动".into()) }
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    if a.kind != "play" || *a != (Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(),
        target_id: a.target_id.clone(), option: a.option.clone(), ..Action::new("play") }) {
        return Err("精准打击只声明一项模式及一个目标".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn wm043_lock(&mut self, id: &str) {
        if let Some(c) = self.board_mut(id) { c.lock_markers = c.lock_markers.saturating_add(1); }
    }
    pub(crate) fn wm043_shown_after(&mut self, o: &ContestOutcome, index: usize) -> RuleResult<()> {
        let shown = o.shown.get(index).ok_or("精准打击展示索引无效")?;
        let Some((key,kind,title))=post_config(&shown.card.definition)else{return Ok(());};
        if !self.wm044_has_sentinel(shown.seat) { return Ok(()); }
        let source = self.source_snapshot(&shown.card, None);
        let options = self.binding_options(shown.seat, &source, &lock_target());
        if options.is_empty() { return Ok(()); }
        let f = ResolutionFrame { resource_payments: vec![], frame_id: self.id(), ability_key: key.into(), actor: shown.seat, source,
            targets: vec![], already_paid: vec![], guard: GuardState::Accepted, cursor: 1,
            steps: vec![Step { context: shown.seat, op: Op::WM043ShownLock { contest_id: o.contest_id.clone(), index } }],
            chosen_region: None };
        let Some(state) = &mut self.mystic_contest else { return Err("精准打击后置缺少原对抗".into()); };
        let MysticPhase::After { post_frame, .. } = &mut state.phase else { return Err("精准打击须在展示后结算".into()); };
        *post_frame = Some(Box::new(f.clone()));
        self.choice(shown.seat, kind, title.into(), options, 0, 1, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::WM043ShownTarget });
        Ok(())
    }
    pub(crate) fn validate_wm043_post(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let [Step { context, op: Op::WM043ShownLock { contest_id, index } }] = f.steps.as_slice()
            else { return Err("精准打击后置须保留原展示收据".into()); };
        let state = self.mystic_states().find(|s| s.contest_id == *contest_id).ok_or("精准打击缺少原对抗层")?;
        let MysticPhase::After { outcome, next_shown, post_frame } = &state.phase
            else { return Err("精准打击后置阶段不符".into()); };
        let shown = outcome.shown.get(*index).ok_or("精准打击原展示索引不符")?;
        let Some((key,_,_))=post_config(&shown.card.definition)else{return Err("展示后锁定来源不是原印刷卡".into());};
        if f.ability_key != key || f.actor != shown.seat
            || *context != f.actor || *next_shown != index + 1 || f.cursor != 1
            || !matches!(f.guard, GuardState::Accepted) || !f.targets.is_empty() || !f.already_paid.is_empty()
            || f.chosen_region.is_some() || !same(&f.source, &self.source_snapshot(&shown.card, None))
            || post_frame.as_ref().is_none_or(|p| !same(p.as_ref(), f)) {
            return Err("精准打击只能消费同次本人展示的唯一后置选择".into());
        }
        Ok(())
    }
    pub(crate) fn wm043_choose_shown(&mut self, f: &ResolutionFrame, chooser: usize, a: &Action, selected: &[String]) -> RuleResult<()> {
        self.validate_wm043_post(f)?;
        if chooser != f.actor || selected.len() > 1 || !self.wm044_has_sentinel(chooser)
            || *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
            return Err("精准打击后置只由原哨兵玩家选择一次或放弃".into());
        }
        if let Some(id) = selected.first() {
            if !self.valid_binding(chooser, &f.source, &lock_target(), id) { return Err("精准打击后置目标不合法".into()); }
            // The shown-card choice bypasses accept_frame_guard; a shield still
            // ends this enemy effect successfully, leaving the contest to resume.
            if self.shield_stops(chooser, id) { return Ok(()); }
            self.wm043_lock(id);
        }
        Ok(())
    }
}
