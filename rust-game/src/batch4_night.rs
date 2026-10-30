//! JZ56 remembers one public subtype per in-play instance. Its continuous
//! bonuses use the current controller, current subtype and current grave size.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "batch4-jz56-choose-undead";
const TITLE: &str = "亡者之夜：选择食尸鬼、行尸或骷髅类别";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UndeadKind { Ghoul, Zombie, Skeleton }
impl UndeadKind {
    pub(crate) fn label(self) -> &'static str {
        match self { Self::Ghoul => "食尸鬼", Self::Zombie => "行尸", Self::Skeleton => "骷髅" }
    }
    fn id(self) -> &'static str {
        match self { Self::Ghoul => "ghoul", Self::Zombie => "zombie", Self::Skeleton => "skeleton" }
    }
}
const TYPES: [UndeadKind; 3] = [UndeadKind::Ghoul, UndeadKind::Zombie, UndeadKind::Skeleton];
fn options() -> Vec<ChoiceOption> {
    TYPES.into_iter().map(|kind| ChoiceOption { id: kind.id().into(), label: kind.label().into(), card: None }).collect()
}
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut d = attachment_definition(target(Zone::Society, EntityKind::Any, Relation::ControlledByActor, Range::Anywhere));
    d.abilities.push(ability(KEY, "进场：选择食尸鬼、行尸或骷髅类别", Timing::Fast,
        vec![], vec![], vec![Op::JZ56ChooseUndead], Some(Event::Enter)));
    d.modifiers.push(StaticModifier::JZ56ChosenUndeadAura);
    d
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    if a.kind != "play" || a.ability_id.as_deref().is_some_and(|key| key != "attach")
        || *a != (Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), ..Action::new("play") }) {
        return Err("亡者之夜只可结附于本方秘社".into());
    }
    Ok(())
}
impl Game {
    /// (temporary influence and defense, permanent combat and wound).
    pub(crate) fn jz56_aura(&self, c: &Card) -> (u32, u32) {
        if c.face_down || c.controller >= self.players.len()
            || !catalog::has_kind(catalog::card(&c.definition), "character")
            || !self.attachments.iter().any(|a| a.card.definition == "JZ56" && a.card.controller == c.controller && !a.card.face_down && a.card.chosen_subtype.is_some())
            || !self.in_play(&c.id).is_some_and(|(_, now)| now == c) { return (0, 0); }
        let subtypes = self.current_subtypes(c);
        let n = self.attachments.iter().filter(|a| a.card.definition == "JZ56" && !a.card.face_down
            && a.card.controller == c.controller && self.attachment_host_valid(a)
            && a.card.chosen_subtype.is_some_and(|kind| subtypes.iter().any(|s| s == kind.label()))).count() as u32;
        let empowered=self.attachments.iter().filter(|a| a.card.definition=="JZ56" && !a.card.face_down && a.card.controller==c.controller && self.attachment_host_valid(a) && a.card.chosen_subtype.is_some_and(|kind|subtypes.iter().any(|s|s==kind.label())) && self.bq095_reference_cards(&a.card).len()>=7).count() as u32;
        (n, empowered)
    }
    fn jz56_source(&self, actor: usize, s: &SourceSnapshot, attaching: bool) -> RuleResult<()> {
        let c = &s.card;
        // A finished choice and an unfinished entry event cannot coexist for
        // the same live instance. A departed source's historical event is legal.
        if !attaching && self.attachments.iter().any(|a| a.card.id == c.id && a.card.chosen_subtype.is_some()) {
            return Err("亡者之夜原实例已经选择类别，不能保留未完成选择".into());
        }
        if c.definition != "JZ56" || c.face_down || c.chosen_subtype.is_some()
            || actor >= self.players.len() || c.controller != actor || c.owner >= self.players.len()
            || self.mystic_instance_number(&c.id).is_none() || s.region.is_some() || s.source_region_instance.is_some()
            || s.play_source.is_some() || s.death_echo.is_some() || s.observed_death.is_some()
            || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || (attaching && s.attachment_host_instance.is_some())
            || (!attaching && s.attachment_host_instance.as_deref() != Some(format!("society:p{actor}").as_str()))
            || self.batch3_physical_cards().any(|now| now.id == c.id && (attaching
                || !self.attachments.iter().any(|a| a.card.id == now.id && a.card.definition == "JZ56"))) {
            return Err("亡者之夜须保留原牌、原操控者和本人秘社宿主".into());
        }
        self.mobility_immutable_identity(c)
    }
    pub(crate) fn validate_jz56_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.jz56_source(d.actor, &d.source, false)?;
        if !same(&d.ability, &definition().abilities[1]) { return Err("亡者之夜只接受完整进场选择程序".into()); }
        Ok(())
    }
    pub(crate) fn validate_jz56_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let attach = f.ability_key == "attach";
        self.jz56_source(f.actor, &f.source, attach)?;
        if !self.mystic_instance_number(&f.source.card.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(s, f)| s < f)
            || f.chosen_region.is_some() { return Err("亡者之夜帧身份或地区载荷无效".into()); }
        if attach {
            if !f.steps.is_empty() || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked | GuardState::Accepted | GuardState::Cancelled)
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if f.money_units(0) <= 2
                    && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()
                    && ids.iter().all(|id| self.mystic_instance_number(id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(n, frame)| n < frame) && *id != f.source.card.id)) {
                return Err("亡者之夜结附须保留实际支付和空程序".into());
            }
            let [t] = f.targets.as_slice() else { return Err("亡者之夜须绑定唯一秘社".into()); };
            let host = format!("society:p{}", f.actor);
            if !same(&t.spec, &definition().abilities[0].targets[0]) || t.id != host || t.region_instance.is_some()
                || t.public.instance_id != host || t.public.kind != "society" || t.public.region.is_some()
                || t.public.status != "valid" || !t.public.valid || t.public.invalid_reason.is_some()
                || t.public.owner.as_deref() != Some(player_id(f.actor).as_str())
                || t.public.controller.as_deref() != Some(player_id(f.actor).as_str())
                || t.public.label != format!("{}的秘社", self.players[f.actor].name) {
                return Err("亡者之夜原秘社目标被改写".into());
            }
        } else if f.ability_key != KEY || !f.targets.is_empty() || !f.already_paid.is_empty()
            || !same(&f.steps, &vec![Step { context: f.actor, op: Op::JZ56ChooseUndead }]) || f.cursor > 1
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted) {
            return Err("亡者之夜进场选择不能改变程序、费用或游标".into());
        }
        Ok(())
    }
    pub(crate) fn jz56_choose_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        self.validate_jz56_frame(&f)?;
        if !self.attachments.iter().any(|a| a.card.id == f.source.card.id && a.card.chosen_subtype.is_none() && self.attachment_host_valid(a)) { return Ok(()); }
        self.choice(f.actor, KEY, TITLE.into(), options(), 1, 1, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::JZ56Undead });
        Ok(())
    }
    pub(crate) fn jz56_choose_finish(&mut self, f: &ResolutionFrame, chooser: usize, a: &Action, selected: &[String]) -> RuleResult<()> {
        self.validate_jz56_frame(f)?;
        if chooser != f.actor || f.ability_key != KEY || f.cursor != 1 || selected.len() != 1
            || *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
            return Err("亡者之夜只能选择一个原不死类别".into());
        }
        let kind = TYPES.into_iter().find(|kind| kind.id() == selected[0]).ok_or("不是亡者之夜的三种类别")?;
        if let Some(c) = self.attachments.iter_mut().find(|a| a.card.id == f.source.card.id).map(|a| &mut a.card) {
            if c.chosen_subtype.is_some() { return Err("亡者之夜类别只能选择一次".into()); }
            c.chosen_subtype = Some(kind);
            self.note(format!("亡者之夜：选择{}类别", kind.label()));
        }
        self.settle_deaths();
        Ok(())
    }
    pub(crate) fn validate_jz56_state(&self) -> RuleResult<()> {
        self.validate_combat_wound_state()?;
        for c in self.batch3_physical_cards().filter(|c| c.chosen_subtype.is_some()) {
            if c.definition != "JZ56" || c.face_down || !self.attachments.iter().any(|a| &a.card == c && self.attachment_host_valid(a)) {
                return Err("亡者之夜类别只能留在原公开场内实例".into());
            }
        }
        let mut frames = BTreeSet::new();
        let mut entry_sources = BTreeSet::new();
        let remember = |f: &ResolutionFrame, frames: &mut BTreeSet<String>, entry_sources: &mut BTreeSet<String>| -> RuleResult<()> {
            self.validate_jz56_frame(f)?;
            if !frames.insert(f.frame_id.clone()) || f.ability_key == KEY && !entry_sources.insert(f.source.card.id.clone()) {
                return Err("亡者之夜不能重复保留选择帧".into());
            }
            Ok(())
        };
        for item in &self.stack { if let Some(f) = &item.frame { if f.source.card.definition == "JZ56" {
            remember(f, &mut frames, &mut entry_sources)?;
            if item.id != f.frame_id || item.controller != f.actor || f.cursor != 0 || !matches!(f.guard, GuardState::Unchecked)
                || item.deploy_region.is_some() || item.reveal || item.target != f.targets.first().map(|t| t.id.clone())
                || item.card.is_some() != (f.ability_key == "attach")
                || item.card.as_ref().is_some_and(|c| c.definition != "JZ56" || c.face_down || c.chosen_subtype.is_some()
                    || c.owner != f.source.card.owner || c.controller != f.actor
                    || !self.mystic_instance_number(&c.id).zip(self.mystic_instance_number(&f.frame_id)).is_some_and(|(card, frame)| card.checked_add(1) == Some(frame))) {
                return Err("亡者之夜响应堆叠外壳不符".into());
            }
        } } }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } if d.source.card.definition == "JZ56" => {
                    self.validate_jz56_declaration(d)?;
                    if !entry_sources.insert(d.source.card.id.clone()) { return Err("亡者之夜不能重复进场声明".into()); }
                }
                Effect::Frame { frame } if frame.source.card.definition == "JZ56" => return Err("亡者之夜不能绕过响应直接续帧".into()),
                _ => (),
            }
        }
        if let Some(p) = &self.pending { match &p.resolution {
            ChoiceResolution::Frame { frame: f, choice } => {
                if matches!(choice, FrameChoice::JZ56Undead) && f.source.card.definition != "JZ56" { return Err("亡者之夜待选不能移植来源".into()); }
                if f.source.card.definition == "JZ56" {
                    remember(f, &mut frames, &mut entry_sources)?;
                    let expected = Choice { id: p.choice.id.clone(), kind: KEY.into(), title: TITLE.into(),
                        description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(f.actor),
                        options: options(), preview_cards: vec![], min: Some(1), max: Some(1), amount: None, allow_decline: Some(false) };
                    if !matches!(choice, FrameChoice::JZ56Undead) || f.cursor != 1 || p.seat != f.actor || p.choice.id.is_empty()
                        || !same(&p.choice, &expected) { return Err("亡者之夜待选须保留原操控者及固定三项".into()); }
                }
            }
            ChoiceResolution::Declare { declaration: d, stage } if d.source.card.definition == "JZ56" => {
                self.validate_jz56_declaration(d)?;
                if !entry_sources.insert(d.source.card.id.clone()) { return Err("亡者之夜不能重复进场声明".into()); }
                // The live source may leave during a response; the frozen event remains legal.
                if !matches!(stage, DeclareChoice::Accept) || p.seat != d.actor || p.choice.id.is_empty()
                    || p.choice.player_id != player_id(d.actor) || p.choice.kind != "trigger"
                    || p.choice.title != format!("亡者之夜：{}", d.ability.label)
                    || p.choice.min != Some(0) || p.choice.max != Some(1) || p.choice.allow_decline != Some(true)
                    || p.choice.amount.is_some() || !p.choice.preview_cards.is_empty()
                    || !same(&p.choice.options, &vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }]) {
                    return Err("亡者之夜进场声明外壳不符".into());
                }
            }
            _ => (),
        } }
        Ok(())
    }
}
