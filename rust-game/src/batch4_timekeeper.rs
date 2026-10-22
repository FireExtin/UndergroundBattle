//! XG16: optional entry time, current destroyed-region Prepare count, and
//! personal/enemy time defence. Ally scope follows Hegemony printed16/21.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::Serialize;
use std::collections::BTreeSet;

pub(crate) const ENTRY: &str = "batch4-xg16-entry-time";
pub(crate) const PREPARE: &str = "batch4-xg16-prepare-time";
fn same<T: Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
pub(crate) fn definition() -> Definition {
    let mut t = target(Zone::Board, EntityKind::Any, Relation::Any, Range::Anywhere);
    t.predicate = Some(TargetPredicate::XG16CharacterOrAttachment);
    let mut d = with_abilities(vec![
        ability(ENTRY, "进场：时间1", Timing::Fast, vec![], vec![],
            vec![Op::XG16EntryTime], Some(Event::Enter)),
        ability(PREPARE, "准备：目标角色或附属放置已毁灭地区数量的时间", Timing::Fast,
            vec![], vec![t], vec![Op::XG16PrepareTime { event_id: None }], Some(Event::Prepare)),
    ]);
    d.traits.public = true;
    d.modifiers.push(StaticModifier::XG16TimeDefense);
    d
}
pub(crate) fn canonical(a: &AbilitySpec) -> Option<AbilitySpec> {
    let mut expected = definition().abilities.into_iter().find(|b| b.key == a.key)?;
    if a.key == PREPARE { if let [Op::XG16PrepareTime { event_id }] = a.ops.as_slice() { expected.ops = vec![Op::XG16PrepareTime { event_id: event_id.clone() }]; } }
    Some(expected)
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "deploy" | "conceal" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new(&a.kind) },
        "reveal" => Action { card_id: a.card_id.clone(), ..Action::new("reveal") },
        _ => return Err("计时人戴森只有派遣与现身操作".into()),
    };
    if *a != expected { return Err("计时人戴森不能附带额外费用、数量或载荷".into()); }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.source.card.definition == "XG16")
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
        return Err("计时人戴森触发只接受原待选，不接受额外费用或数量".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn xg16_time_target(&self, source: &SourceSnapshot, c: &Card) -> bool {
        source.card.definition == "XG16"
            && ((!c.face_down && catalog::card(&c.definition).kind == "character")
                || self.attachments.iter().any(|a| a.card.id == c.id
                    && (!c.face_down || crate::batch4_plans::secret(c))
                    && catalog::card(&c.definition).kind == "attachment" && self.attachment_host_valid(a)))
    }
    pub(crate) fn xg16_time_defense(&self, c: &Card) -> (u32, u32) {
        if c.face_down || catalog::card(&c.definition).kind != "character" { return (0, 0); }
        let n = self.effective_time_markers(c);
        // The normal no-time path does not scan the board for aura sources.
        if n == 0 || !self.in_play(&c.id).is_some_and(|(_, live)| live == c) { return (0, 0); }
        self.regions.iter().filter(|r| !r.vacant).flat_map(|r| &r.cards)
            .filter(|s| s.definition == "XG16" && !s.face_down && !self.players[s.controller].eliminated)
            .fold((0u32, 0u32), |(bonus, penalty), s| {
                if s.controller == c.controller { (bonus.saturating_add(n), penalty) }
                else if self.team(s.controller) != self.team(c.controller) { (bonus, penalty.saturating_add(n)) }
                else { (bonus, penalty) }
            })
    }
    fn xg16_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != "XG16" || c.id.is_empty() || actor >= self.players.len()
            || c.owner >= self.players.len() || c.controller != actor
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance.as_ref().is_some_and(|id| id.is_empty() || *id == c.id)
            || s.attachment_host_instance.is_some() || s.death_echo.is_some()
            || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || s.play_source.is_some() && !s.valid_character_play() {
            return Err("计时人戴森来源、操控者或原地区无效".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_xg16_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.xg16_source(d.actor, &d.source)?;
        if d.ability.event == Some(Event::Prepare) { return crate::jc035::validate_ability("XG16", &d.ability); }
        if d.source.card.face_down || d.source.play_source.is_some()
            || canonical(&d.ability).is_none_or(|a| !same(&a, &d.ability)) {
            return Err("计时人戴森须保留原进场或准备触发".into());
        }
        Ok(())
    }
    pub(crate) fn validate_xg16_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if crate::jc035::preparation_frame(f) { return self.validate_jc035_frame(f); }
        self.xg16_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() || f.frame_id == f.source.card.id || f.cursor != 0
            || !matches!(f.guard, GuardState::Unchecked) { return Err("计时人戴森帧身份或阶段无效".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || !f.targets.is_empty() || f.chosen_region != f.source.region
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= 6
                    && ids.iter().all(|id| !id.is_empty() && *id != f.source.card.id)
                    && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some()) {
                return Err("计时人戴森派遣/现身须保留原地区和实际支付".into());
            }
            return Ok(());
        }
        let a = definition().abilities.into_iter().find(|a| a.key == f.ability_key).ok_or("未知计时人戴森程序")?;
        self.validate_xg16_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a.clone() })?;
        if !same(&f.steps, &a.ops.iter().map(|op| Step { context: f.actor, op: op.clone() }).collect::<Vec<_>>())
            || !f.already_paid.is_empty() || f.chosen_region.is_some() || f.targets.len() != a.targets.len()
            || f.targets.iter().zip(&a.targets).any(|(t, spec)| !same(&t.spec, spec)) {
            return Err("计时人戴森程序、目标或费用无效".into());
        }
        if let Some(t) = f.targets.first() {
            let player = |id: &Option<String>| id.as_deref().and_then(|s| s.strip_prefix('p'))
                .and_then(|s| s.parse::<usize>().ok()).is_some_and(|n| n < self.players.len() && *id == Some(player_id(n)));
            if t.id.is_empty() || t.public.instance_id != t.id || !t.public.valid || t.public.status != "valid"
                || t.public.invalid_reason.is_some() || t.region_instance.is_some()
                || !matches!(t.public.kind.as_str(), "character" | "attachment")
                || !player(&t.public.owner) || !player(&t.public.controller)
                || t.public.region.is_some_and(|r| r >= self.regions.len())
                || t.public.kind == "character" && t.public.region.is_none() {
                return Err("计时人戴森须保留原角色/附属目标摘要".into());
            }
        }
        Ok(())
    }
    pub(crate) fn xg16_entry_time(&mut self, f: &ResolutionFrame) {
        if let Some(c) = self.board_mut(&f.source.card.id).filter(|c| c.definition == "XG16" && !c.face_down) {
            c.time_markers = c.time_markers.saturating_add(1);
        }
    }
    pub(crate) fn xg16_prepare_time(&mut self, f: &ResolutionFrame) {
        let n = self.regions.iter().filter(|r| !r.vacant && r.destroyed).count() as u32;
        if let Some(c) = f.targets.first().and_then(|t| self.board_mut(&t.id)) {
            c.time_markers = c.time_markers.saturating_add(n);
        }
    }
    pub(crate) fn validate_xg16_state(&self) -> RuleResult<()> {
        let related = |s: &SourceSnapshot| s.card.definition == "XG16";
        let mut frames = BTreeSet::new();
        for item in &self.stack {
            if let Some(f) = &item.frame { if related(&f.source) && !self.validated_cross_card_frame(f)? && !crate::jc035::preparation_frame(f) {
                self.validate_xg16_frame(f)?;
                let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
                if !frames.insert(f.frame_id.clone()) || item.id != f.frame_id || item.controller != f.actor
                    || item.card.is_some() != play || item.reveal != (f.ability_key == "reveal")
                    || item.deploy_region != if play { f.source.region } else { None }
                    || item.target != f.targets.first().map(|t| t.id.clone())
                    || item.card.as_ref().is_some_and(|c| c.definition != "XG16" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor) {
                    return Err("计时人戴森堆叠外壳与原帧不符".into());
                }
            } }
        }
        for e in &self.effects { match e {
            Effect::Declare { declaration: d } if related(&d.source) && !self.validated_cross_card_declaration(d)? => self.validate_xg16_declaration(d)?,
            Effect::Frame { frame: f } if related(&f.source) && !self.validated_cross_card_frame(f)? => return Err("计时人戴森不能绕过响应保存直接帧".into()),
            _ => {},
        } }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Declare { declaration: d, stage } if related(&d.source) && !self.validated_cross_card_declaration(d)? => {
                self.validate_xg16_declaration(d)?;
                let (options, expected_stage) = if let Some(slot) = d.ability.targets.first() {
                    (self.binding_options(d.actor, &d.source, slot), DeclareChoice::Target)
                } else { (vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }], DeclareChoice::Accept) };
                let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(),
                    title: format!("{}：{}", catalog::card("XG16").name, d.ability.label),
                    description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor), options,
                    preview_cards: vec![], min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                if !same(stage, &expected_stage) || p.seat != d.actor || p.choice.id.is_empty()
                    || expected.options.is_empty() || !same(&p.choice, &expected) { return Err("计时人戴森待选内容无效".into()); }
            },
            ChoiceResolution::Frame { frame: f, .. } if related(&f.source) && !self.validated_cross_card_frame(f)? => return Err("计时人戴森没有帧内续体".into()),
            _ => {},
        } }
        Ok(())
    }
}
