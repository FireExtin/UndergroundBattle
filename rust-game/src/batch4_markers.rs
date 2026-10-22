//! XQ21 copies one existing marker category, never damage or a printed trait.
//! Marker inheritance follows manual P11; physical carriers stay distinct.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const KEY: &str = "batch4-xq21-copy-marker";
const KIND: &str = "xq21-marker";
const TITLE: &str = "选择要增加一枚的已有指示物标志类别";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerKind { Advancement, PermanentCombat, PermanentDefense, Time, Wound, Shield, Lock }
impl MarkerKind {
    const ALL: [Self; 7] = [Self::Advancement, Self::PermanentCombat, Self::PermanentDefense, Self::Time, Self::Wound, Self::Shield, Self::Lock];
    fn count(self, c: &Card) -> u32 { match self {
        Self::Advancement => c.advancement_markers, Self::PermanentCombat => c.permanent_combat_markers,
        Self::PermanentDefense => c.permanent_defense_markers, Self::Time => c.time_markers,
        Self::Wound => c.wounds, Self::Shield => c.shield, Self::Lock => c.lock_markers,
    } }
    fn field(self, c: &mut Card) -> &mut u32 { match self {
        Self::Advancement => &mut c.advancement_markers, Self::PermanentCombat => &mut c.permanent_combat_markers,
        Self::PermanentDefense => &mut c.permanent_defense_markers, Self::Time => &mut c.time_markers,
        Self::Wound => &mut c.wounds, Self::Shield => &mut c.shield, Self::Lock => &mut c.lock_markers,
    } }
    fn id(self) -> &'static str { match self {
        Self::Advancement => "advancement", Self::PermanentCombat => "permanent-combat", Self::PermanentDefense => "permanent-defense",
        Self::Time => "time", Self::Wound => "wound", Self::Shield => "shield", Self::Lock => "lock",
    } }
    fn label(self) -> &'static str { match self {
        Self::Advancement => "推进标志", Self::PermanentCombat => "永久战斗1标志", Self::PermanentDefense => "防御+1标志",
        Self::Time => "时间标志", Self::Wound => "伤口标志", Self::Shield => "护盾标志", Self::Lock => "锁定标志",
    } }
}
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
fn options(kinds: &[MarkerKind]) -> Vec<ChoiceOption> {
    kinds.iter().map(|k| ChoiceOption { id: k.id().into(), label: k.label().into(), card: None }).collect()
}
pub(crate) fn definition() -> Definition {
    let mut t = target(Zone::Board, EntityKind::Any, Relation::Any, Range::Anywhere);
    t.predicate = Some(TargetPredicate::XQ21MarkedCarrier);
    let mut a = ability(KEY, "快速行动：牺牲一个角色，增加目标已有的一类标志", Timing::Fast,
        vec![Cost::SacrificeSelectedControlledCharacter], vec![t], vec![Op::XQ21CopyMarker], None);
    a.play_only = true;
    with_abilities(vec![a])
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    if a.kind != "play" || a.ability_id.as_deref().is_some_and(|id| id != KEY)
        || *a != (Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), cost_selected: a.cost_selected.clone(), ..Action::new("play") }) {
        return Err("臣服于命运只接受原目标及一次额外牺牲费用".into());
    }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(p.resolution, ChoiceResolution::Frame { choice: FrameChoice::XQ21Marker { .. }, .. })
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
        return Err("标志类别选择不能重新指定目标、数量或费用".into());
    }
    Ok(())
}
impl Game {
    // Deduplicate physical carriers even when the chosen target is a dual-type
    // attached character. This reads markers; it never relocates attachments.
    fn xq21_marker_kinds(&self, id: &str) -> Vec<MarkerKind> {
        let Some((_, first)) = self.in_play(id) else { return vec![]; };
        let mut cards = vec![first];
        let mut seen = BTreeSet::from([first.id.as_str()]);
        let mut cursor = 0;
        while cursor < cards.len() {
            let parent = cards[cursor].id.clone(); cursor += 1;
            for a in &self.attachments {
                if a.host_id == parent && self.attachment_host_valid(a) && seen.insert(a.card.id.as_str()) { cards.push(&a.card); }
            }
        }
        MarkerKind::ALL.into_iter().filter(|kind| cards.iter().any(|c| kind.count(c) > 0)).collect()
    }
    pub(crate) fn xq21_target(&self, source: &SourceSnapshot, c: &Card) -> bool {
        source.card.definition == "XQ21"
            && ((!crate::batch4_plans::is_plan(c) && (c.face_down || catalog::card(&c.definition).kind == "character"))
                || self.attachments.iter().any(|a| a.card.id == c.id && self.attachment_host_valid(a)
                    && (!c.face_down || crate::batch4_plans::secret(c))))
            && !self.xq21_marker_kinds(&c.id).is_empty()
    }
    pub(crate) fn validate_xq21_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let c = &f.source.card; let d = definition(); let a = &d.abilities[0];
        if c.definition != "XQ21" || c.id.is_empty() || c.face_down || c.owner >= self.players.len()
            || f.actor >= self.players.len() || c.controller != f.actor || c.damage > 0 || MarkerKind::ALL.into_iter().any(|k| k.count(c) > 0)
            || f.source.region.is_some() || f.source.source_region_instance.is_some() || f.source.attachment_host_instance.is_some()
            || f.source.play_source.is_some() || f.source.observed_death.is_some() || f.source.observed_exposure.is_some()
            || f.source.observed_mobility_grant.is_some() || f.source.death_echo.is_some()
            || f.frame_id.is_empty() || f.frame_id == c.id || f.ability_key != KEY || f.chosen_region.is_some() || f.cursor > 1
            || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)) || (f.cursor == 1 && !matches!(f.guard, GuardState::Accepted))
            || !same(&f.steps, &vec![Step { context: f.actor, op: Op::XQ21CopyMarker }]) || f.targets.len() != 1
            || !same(&f.targets[0].spec, &a.targets[0]) || self.batch3_physical_cards().any(|live| live.id == c.id) {
            return Err("臣服于命运的来源、目标程序或执行阶段被改写".into());
        }
        self.mobility_immutable_identity(c)?;
        let t = &f.targets[0];
        if t.id.is_empty() || t.region_instance.is_some() || t.public.instance_id != t.id || !t.public.valid
            || t.public.status != "valid" || t.public.invalid_reason.is_some() || !matches!(t.public.kind.as_str(), "character" | "hidden" | "attachment")
            || t.public.region.is_some_and(|r| r >= self.regions.len())
            || t.public.owner.as_deref().is_none_or(|id| !self.players.iter().any(|p| player_id(p.seat) == id))
            || t.public.controller.as_deref().is_none_or(|id| !self.players.iter().any(|p| player_id(p.seat) == id)) {
            return Err("臣服于命运必须保留原实体目标摘要".into());
        }
        let [PaidCost::Assets(assets), PaidCost::Sacrificed { old_instance, controller, owner }] = f.already_paid.as_slice()
            else { return Err("臣服于命运须保留印刷费和一次牺牲收据".into()); };
        if assets.len() > 1 || assets.iter().any(|id| id.is_empty() || *id == c.id || id == old_instance || *id == t.id)
            || old_instance.is_empty() || *old_instance == c.id || *old_instance == f.frame_id || *controller != f.actor || *owner >= self.players.len()
            || self.batch3_physical_cards().any(|live| live.id == *old_instance) {
            return Err("臣服于命运的资产支付或已消费牺牲实体无效".into());
        }
        Ok(())
    }
    pub(crate) fn xq21_marker_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_xq21_frame(&f)?;
        let kinds = self.xq21_marker_kinds(&f.targets[0].id);
        if kinds.is_empty() { return Err("已接纳的标志复制目标没有可选类别".into()); }
        self.choice(f.actor, KIND, TITLE.into(), options(&kinds), 1, 1, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::XQ21Marker { kinds } });
        self.pending.as_mut().unwrap().choice.allow_decline = Some(false);
        Ok(())
    }
    pub(crate) fn xq21_marker_finish(&mut self, f: &ResolutionFrame, kinds: &[MarkerKind], selected: &[String]) -> RuleResult<()> {
        self.validate_xq21_frame(f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) || kinds != self.xq21_marker_kinds(&f.targets[0].id) || selected.len() != 1 {
            return Err("标志复制只能继续原待选程序一次".into());
        }
        let kind = kinds.iter().copied().find(|k| k.id() == selected[0]).ok_or("请选择原目标当前已有的标志类别")?;
        let id = &f.targets[0].id;
        if kind == MarkerKind::Advancement {
            let controller = self.in_play(id).ok_or("原计划已离场")?.1.controller;
            // Reuse normal threshold disclosure and the owner's forced trigger.
            self.advance_plan_instance(controller, id);
        } else {
            let c = self.board_mut(id).ok_or("原标志目标已离场")?;
            let n = kind.field(c); *n = n.checked_add(1).ok_or("指示物数量超过当前表示范围")?;
        }
        Ok(())
    }
    pub(crate) fn validate_xq21_state(&self) -> RuleResult<()> {
        let mut frames = BTreeSet::new();
        let relevant = |f: &ResolutionFrame| f.source.card.definition == "XQ21" || f.steps.iter().any(|s| matches!(s.op, Op::XQ21CopyMarker));
        for item in &self.stack { if let Some(f) = &item.frame { if relevant(f) {
            self.validate_xq21_frame(f)?;
            if !frames.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor || item.reveal || item.deploy_region.is_some()
                || item.target.as_ref() != Some(&f.targets[0].id) || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                || item.card.as_ref().is_none_or(|c| c.definition != "XQ21" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor) {
                return Err("臣服于命运的响应外壳被改写".into());
            }
        } } }
        for e in &self.effects { match e {
            Effect::Frame { frame: f } if relevant(f) => self.validate_xq21_frame(f)?,
            Effect::Declare { declaration: d } if d.source.card.definition == "XQ21" || d.ability.key == KEY => return Err("臣服于命运不是可移植的触发声明".into()),
            _ => (),
        } }
        let Some(p) = &self.pending else { return Ok(()); };
        match &p.resolution {
            ChoiceResolution::Frame { frame: f, choice: FrameChoice::XQ21Marker { kinds } } => {
                self.validate_xq21_frame(f)?;
                if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) || kinds.is_empty() || *kinds != self.xq21_marker_kinds(&f.targets[0].id)
                    || p.seat != f.actor || p.choice.player_id != player_id(f.actor) || p.choice.id.is_empty() || p.choice.kind != KIND || p.choice.title != TITLE
                    || p.choice.min != Some(1) || p.choice.max != Some(1) || p.choice.allow_decline != Some(false) || p.choice.amount.is_some()
                    || !p.choice.preview_cards.is_empty() || !same(&p.choice.options, &options(kinds)) {
                    return Err("臣服于命运的标志类别、原目标或选择者被改写".into());
                }
            },
            ChoiceResolution::Frame { frame: f, .. } if relevant(f) => return Err("臣服于命运不能移植其他待选程序".into()),
            _ if p.choice.kind == KIND => return Err("标志复制待选外壳与程序不符".into()),
            _ => (),
        }
        Ok(())
    }
}
