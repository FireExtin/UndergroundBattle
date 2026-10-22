//! BQ053 removes a distinct personal Society marker. This does not implement or
//! admit the absent Shoggoth Society, its core, commands, or marker generation.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use serde::Serialize;
use std::collections::BTreeSet;
pub(crate) const ENTER: &str = "batch4-bq053-enter";
pub(crate) const LEAVE: &str = "batch4-bq053-leave";
fn same<T: Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut d = with_abilities(vec![
        ability(ENTER, "进场：移除本方秘社失控1", Timing::Fast, vec![], vec![], vec![Op::BQ053RemoveLossControl], Some(Event::Enter)),
        ability(LEAVE, "离场：移除本方秘社失控1", Timing::Fast, vec![], vec![], vec![Op::BQ053RemoveLossControl], Some(Event::Leave)),
    ]);
    d.traits.public = true;
    d.traits.guard = 1;
    d
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    let expected = match a.kind.as_str() {
        "deploy" => Action { card_id: a.card_id.clone(), region: a.region, ..Action::new("deploy") },
        "reveal" => Action { card_id: a.card_id.clone(), ..Action::new("reveal") },
        _ => return Err("公开老看守只能正面派遣或在翻暗后现身".into()),
    };
    if *a != expected {
        return Err("老看守不能附加触发选择或额外载荷".into());
    }
    Ok(())
}
pub(crate) fn validate_choice_action(p: &Pending, a: &Action) -> RuleResult<()> {
    if matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.source.card.definition == "BQ053")
        && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
        return Err("老看守触发只接受原发动或放弃选择".into());
    }
    Ok(())
}
impl Game {
    fn bq053_source(&self, actor: usize, s: &SourceSnapshot) -> RuleResult<()> {
        let c = &s.card;
        if c.definition != "BQ053" || c.id.is_empty() || c.owner >= self.players.len() || actor >= self.players.len() || c.controller != actor
            || s.region.is_some_and(|r| r >= self.regions.len()) || s.death_echo.is_some() || s.observed_death.is_some()
            || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some() || s.attachment_host_instance.is_some()
            || s.region.zip(s.source_region_instance.as_ref()).is_some_and(|(r, id)| self.regions.iter().enumerate().any(|(other, live)| live.card.id == *id && other != r))
            || s.play_source.is_none() && self.batch3_physical_cards().any(|live| live.id == c.id) && self.board(&c.id).is_none() {
            return Err("老看守须保留原角色、操控者及地区实体".into());
        }
        self.mobility_immutable_identity(c)
    }
    fn bq053_expected(&self, s: &SourceSnapshot, key: &str) -> RuleResult<AbilitySpec> {
        if let Some(a) = definition().abilities.into_iter().find(|a| a.key == key) { return Ok(a); }
        let region = s.source_region_instance.as_deref().ok_or("老看守奖励缺少原地区")?;
        match key {
            "renown" => Ok(crate::renown::renown_ability(region)),
            "jc089-combat-glory" => Ok(crate::rules::jc089_combat_glory_ability(region)),
            _ => Err("老看守未知程序".into()),
        }
    }
    pub(crate) fn validate_bq053_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.bq053_source(d.actor, &d.source)?;
        let s = &d.source;
        let a = self.bq053_expected(s, &d.ability.key)?;
        if s.card.face_down || s.play_source.is_some() || s.region.is_none() || s.source_region_instance.as_ref().is_none_or(|id| id.is_empty() || *id == s.card.id)
            || !same(&a, &d.ability) || a.key == LEAVE && self.in_play(&s.card.id).is_some() {
            return Err("老看守须保留完整实际进场或离场触发".into());
        }
        Ok(())
    }
    pub(crate) fn validate_bq053_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        self.bq053_source(f.actor, &f.source)?;
        if f.frame_id.is_empty() || f.cursor > f.steps.len() || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor > 0 && !matches!(f.guard, GuardState::Accepted) { return Err("老看守帧身份、游标或守卫无效".into()); }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty() || !f.targets.is_empty() || f.chosen_region != f.source.region
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= 1 && ids.iter().all(|id| !id.is_empty() && *id != f.source.card.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
                || f.ability_key == "deploy" && (f.source.card.face_down || !f.source.valid_character_play())
                || f.ability_key == "reveal" && (!f.source.card.face_down || f.source.play_source.is_some()) {
                return Err("老看守派遣与现身须保留原地区、实际支付及空程序".into());
            }
            return Ok(());
        }
        let a = self.bq053_expected(&f.source, &f.ability_key)?;
        self.validate_bq053_declaration(&Declaration { actor: f.actor, source: f.source.clone(), ability: a.clone() })?;
        if f.chosen_region.is_some() || !f.already_paid.is_empty() || !f.targets.is_empty()
            || f.steps.len() != a.ops.len() || f.steps.iter().zip(&a.ops).any(|(s, op)| s.context != f.actor || !same(&s.op, op)) {
            return Err("老看守的本人标志移除或奖励程序被改写".into());
        }
        Ok(())
    }
    pub(crate) fn validate_bq053_state(&self) -> RuleResult<()> {
        if self.players.iter().any(|p| p.society_zone.loss_control > 3) { return Err("秘社失控标志不能超过规则上限3".into()); }
        let mut seen = BTreeSet::new();
        let mut frames = BTreeSet::new();
        let mut remember = |s: &SourceSnapshot, key: &str| -> RuleResult<()> {
            if matches!(key, ENTER | LEAVE) && !seen.insert((s.card.id.clone(), key.to_string())) { return Err("同一老看守的同次进离场触发不能重复".into()); }
            Ok(())
        };
        for item in &self.stack {
            if let Some(f) = &item.frame {
                if f.source.card.definition != "BQ053" || self.validated_cross_card_frame(f)? { continue; }
                self.validate_bq053_frame(f)?;
                remember(&f.source, &f.ability_key)?;
                let play = matches!(f.ability_key.as_str(), "deploy" | "reveal");
                if !frames.insert(&f.frame_id) || item.id != f.frame_id || item.controller != f.actor || f.cursor != 0
                    || !matches!(f.guard, GuardState::Unchecked) || item.card.is_some() != play || item.reveal != (f.ability_key == "reveal")
                    || item.deploy_region != if play { f.chosen_region } else { None } || item.target.is_some()
                    || item.card.as_ref().is_some_and(|c| c.definition != "BQ053" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor) {
                    return Err("老看守堆叠外壳与原程序不符".into());
                }
            }
        }
        for e in &self.effects {
            match e {
                Effect::Declare { declaration: d } if d.source.card.definition == "BQ053" && !self.validated_cross_card_declaration(d)? => {
                    self.validate_bq053_declaration(d)?; remember(&d.source, &d.ability.key)?;
                },
                Effect::Frame { frame: f } if f.source.card.definition == "BQ053" && !self.validated_cross_card_frame(f)? => return Err("老看守触发不能绕过响应".into()),
                _ => {},
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration: d, stage } if d.source.card.definition == "BQ053" && !self.validated_cross_card_declaration(d)? => {
                    self.validate_bq053_declaration(d)?; remember(&d.source, &d.ability.key)?;
                    let expected = Choice { id: p.choice.id.clone(), kind: "trigger".into(), title: format!("{}：{}", catalog::card("BQ053").name, d.ability.label),
                        description: "选择符合数量限制的选项，然后确认。".into(), player_id: player_id(d.actor),
                        options: vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }], preview_cards: vec![],
                        min: Some(0), max: Some(1), amount: None, allow_decline: Some(true) };
                    if !matches!(stage, DeclareChoice::Accept) || p.seat != d.actor || p.choice.id.is_empty() || !same(&p.choice, &expected) {
                        return Err("老看守只由原操控者接受或放弃触发".into());
                    }
                },
                ChoiceResolution::Frame { frame: f, .. } if f.source.card.definition == "BQ053" && !self.validated_cross_card_frame(f)? => return Err("老看守没有额外结算选择".into()),
                _ => {},
            }
        }
        Ok(())
    }
}
