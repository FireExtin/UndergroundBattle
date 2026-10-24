//! WM064 uses the existing attachment, subtype-sacrifice and direct-reveal
//! mechanisms. Continuous grants query the original live human host.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;
pub(crate) const ID: &str = "WM064";
pub(crate) const FAST: &str = "import20-wm064-reveal-criminal";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut host = target(Zone::Board, EntityKind::Character, Relation::ControlledByActor, Range::Anywhere);
    host.subtype = Some("人类".into());
    let mut play = ability("attach", "结附目标本方人类角色", Timing::Standard, vec![], vec![host.clone()], vec![], None); play.play_only = true;
    let mut fast = ability(FAST, "快速行动1，横置并牺牲罪犯：若原本方暗藏者正面是罪犯则现身", Timing::Fast,
        vec![Cost::Assets(1), Cost::ExhaustSource, Cost::SacrificeControlledSubtype(vec!["罪犯".into()])],
        vec![target(Zone::Board, EntityKind::Hidden, Relation::ControlledByActor, Range::Anywhere)], vec![Op::WM064RevealCriminal], None);
    fast.activation_only = true;
    Definition { abilities: vec![play, fast], modifiers: vec![StaticModifier::WM064HostLeaderRenownCriminal],
        attachment: Some(AttachmentSpec { controls_host: false, host_subtype_change: SubtypeChange::None, host,
            host_icons: Icons { investigation: 1, combat: 2, influence: 1 }, host_temporary_icons: None,
            host_barrier: false, host_defense_bonus: 2, host_leaves: HostLeaveDestination::OwnerGraveyard }), ..Definition::default() }
}
fn owns(ops: &[Op]) -> bool { ops.iter().any(|op| match op {
    Op::WM064RevealCriminal => true,
    Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => owns(body),
    Op::IfTargetExhausted { exhausted, ready, .. } => owns(std::slice::from_ref(exhausted)) || owns(std::slice::from_ref(ready)),
    _ => false,
}) }
pub(crate) fn related(id: &str, a: &AbilitySpec) -> bool { id == ID || a.key == FAST || owns(&a.ops) || a.modes.iter().any(|m| owns(&m.ops)) }
pub(crate) fn frame_related(f: &ResolutionFrame) -> bool { f.source.card.definition == ID || f.ability_key == FAST || f.steps.iter().any(|s| owns(std::slice::from_ref(&s.op))) }
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if id != ID || !definition().abilities.iter().any(|b| same(a, b)) { return Err("巨蟒纹身只准入完整本方人类结附或罪犯现身程序".into()); } Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if id == ID && !same(d, &definition()) || id != ID && (d.abilities.iter().any(|a| related(id, a))
        || d.modifiers.iter().any(|m| matches!(m, StaticModifier::WM064HostLeaderRenownCriminal))) {
        return Err("巨蟒纹身完整定义和宿主授予不能改写或移植".into());
    } Ok(())
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "play" if a.ability_id.as_deref().is_none_or(|k| k == "attach") => Action { card_id: a.card_id.clone(), target_id: a.target_id.clone(), ability_id: a.ability_id.clone(), ..Action::new("play") },
        "activate" if a.ability_id.as_deref() == Some(FAST) => Action { card_id: a.card_id.clone(), target_id: a.target_id.clone(), ability_id: a.ability_id.clone(), cost_selected: a.cost_selected.clone(), ..Action::new("activate") },
        _ => return Err("巨蟒纹身没有该行动程序".into()),
    };
    if *a != expected { return Err("巨蟒纹身不可夹带额外目标、地区、模式或选择".into()); } Ok(())
}
impl Game {
    // Called inside current_subtypes, so do not recursively call host validity
    // or current_subtypes. The caller supplies the current Human condition.
    pub(crate) fn wm064_live_tattoo(&self, c: &Card) -> bool {
        !c.face_down && catalog::has_kind(catalog::card(&c.definition), "character")
            && self.board(&c.id).is_some_and(|(_, live)| live == c)
            && self.attachments.iter().any(|a| a.card.definition == ID && !a.card.face_down
                && a.host_id == c.id && a.card.controller == c.controller && !self.players[c.controller].eliminated)
    }
    pub(crate) fn wm064_status_grant(&self, c: &Card) -> bool {
        self.wm064_live_tattoo(c) && self.current_subtypes(c).iter().any(|s| s == "人类")
    }
    pub(crate) fn wm064_reveal_criminal(&mut self, f: &ResolutionFrame) -> RuleResult<()> {
        let id = &f.targets.first().ok_or("巨蟒纹身缺少原暗藏者目标")?.id;
        if self.board(id).is_some_and(|(_, c)| c.face_down && c.controller == f.actor
            && catalog::card(&c.definition).subtypes.iter().any(|s| s == "罪犯")) {
            self.world_reveal(f.actor, id, false)?;
        } Ok(())
    }
    pub(crate) fn validate_wm064_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        let s = &f.source; let c = &s.card;
        if f.actor >= self.players.len() || c.definition != ID || c.face_down || c.owner >= self.players.len() || c.controller != f.actor
            || !self.mystic_instance_valid(&c.id) || !self.mystic_instance_valid(&f.frame_id) || f.frame_id == c.id
            || s.play_source.is_some() || s.death_echo.is_some() || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || f.chosen_region.is_some() || f.cursor > f.steps.len() || f.targets.len() != 1
            || f.steps.iter().any(|s| s.context != f.actor)
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked) && !(f.steps.is_empty() && matches!(f.guard, GuardState::Accepted | GuardState::Cancelled))
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted) { return Err("巨蟒纹身须保留原来源、玩家、程序和目标".into()); }
        self.mobility_immutable_identity(c)?;
        let spec = definition().abilities.into_iter().find(|a| a.key == f.ability_key).ok_or("未知巨蟒纹身程序")?;
        if !same(&f.targets[0].spec, &spec.targets[0]) || !same(&f.steps.iter().map(|s| s.op.clone()).collect::<Vec<_>>(), &spec.ops) {
            return Err("巨蟒纹身的完整目标条件或操作被改写".into());
        }
        if f.ability_key == "attach" {
            if s.region.is_some() || s.source_region_instance.is_some() || s.attachment_host_instance.is_some()
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= 6 && ids.iter().all(|id| !id.is_empty() && *id != c.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
                return Err("巨蟒纹身结附须保留实际支付与场外来源".into());
            }
        } else {
            self.validate_batch20_payment(f, &spec.costs)?;
            if s.region.is_none_or(|r| r >= self.regions.len()) || s.source_region_instance.as_ref().is_none_or(|id| !self.mystic_instance_valid(id))
                || s.attachment_host_instance.as_ref().is_none_or(|id| !self.mystic_instance_valid(id) || *id == c.id) {
                return Err("巨蟒纹身发动须保留原宿主与原地区".into());
            }
            let [PaidCost::Assets(assets), PaidCost::Exhausted(_), PaidCost::SacrificedSubtype { card: paid, .. }] = f.already_paid.as_slice() else { return Err("巨蟒纹身支付次序无效".into()); };
            if paid.id == c.id || assets.iter().any(|a| *a == paid.id || *a == c.id) || self.batch3_physical_cards().any(|now| now.id == paid.id) {
                return Err("巨蟒纹身费用须为实际不同资产与已牺牲罪犯".into());
            }
            self.mobility_immutable_identity(paid)?;
        } Ok(())
    }
    pub(crate) fn validate_wm064_state(&self) -> RuleResult<()> {
        for item in &self.stack { if let Some(f) = &item.frame { if frame_related(f) && !self.validated_cross_card_frame(f)? {
            self.validate_wm064_frame(f)?;
            let play = f.ability_key == "attach";
            if item.id != f.frame_id || item.controller != f.actor || item.target != f.targets.first().map(|t| t.id.clone())
                || item.card.is_some() != play || item.reveal || item.deploy_region.is_some() || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked) {
                return Err("巨蟒纹身响应堆叠必须保留原外壳".into());
            }
        } } }
        for e in &self.effects { match e {
            Effect::Declare { declaration: d } if related(&d.source.card.definition, &d.ability) => return Err("巨蟒纹身没有自主进场触发".into()),
            Effect::Frame { frame: f } if frame_related(f) => self.validate_wm064_frame(f)?,
            _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration: d, .. } if related(&d.source.card.definition, &d.ability) => return Err("巨蟒纹身没有触发选择".into()),
            ChoiceResolution::Frame { frame: f, .. } if frame_related(f) => return Err("巨蟒纹身没有原子结算中的额外选择".into()),
            _ => {},
        } } Ok(())
    }
}
