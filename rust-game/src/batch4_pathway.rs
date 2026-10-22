//! XQ13 converts the recipient's permanent/temporary icon categories. All
//! Pathways share one personal-control limit, independently of gold names.
use crate::{catalog, engine::RuleResult, model::*, rules::*};
use std::collections::BTreeSet;
const TITLE: &str = "道途：选择一个道途牺牲，直到只操控一个";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn definition() -> Definition {
    let mut d = crate::rules::attachment_definition(target(Zone::Society, EntityKind::Any, Relation::ControlledByActor, Range::Anywhere));
    d.modifiers.push(StaticModifier::XQ13VampireIconConversion); d
}
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    if a.kind != "play" || a.ability_id.as_deref().is_some_and(|k| k != "attach")
        || *a != (Action { card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(), ..Action::new("play") }) {
        return Err("焚血之道只能结附于本方秘社".into());
    }
    Ok(())
}
impl Game {
    fn pathway(&self, c: &Card) -> bool {
        !c.face_down && catalog::card(&c.definition).subtypes.iter().any(|t| t == "道途")
    }
    fn pathway_options(&self, seat: usize) -> Vec<ChoiceOption> {
        if seat >= self.players.len() { return vec![]; }
        self.in_play_cards().into_iter().filter(|(_, c)| c.controller == seat && self.pathway(c))
            .map(|(r,c)| self.option(c, seat, r, None)).collect()
    }
    pub(crate) fn queue_pathway_checks(&mut self) {
        for seat in 0..self.players.len() {
            if self.pathway_options(seat).len() > 1
                && !self.effects.iter().any(|e| matches!(e, Effect::Unique { seat: s } if *s == seat))
                && !self.pending.as_ref().is_some_and(|p| p.seat == seat && matches!(p.resolution, ChoiceResolution::Unique)) {
                self.effects.push_front(Effect::Unique { seat });
            }
        }
    }
    pub(crate) fn offer_pathway_choice(&mut self, seat: usize) -> bool {
        let options = self.pathway_options(seat); if options.len() <= 1 { return false; }
        self.effects.push_front(Effect::Unique { seat });
        self.choice(seat, "target", TITLE.into(), options, 1, 1, None, ChoiceResolution::Unique); true
    }
    pub(crate) fn validate_pathway_choice(&self) -> RuleResult<()> {
        if let Some(p) = &self.pending { if matches!(p.resolution, ChoiceResolution::Unique) {
            let options = self.pathway_options(p.seat);
            let relevant = p.choice.title == TITLE || p.choice.options.iter().any(|o| self.in_play(&o.id).is_some_and(|(_,c)| self.pathway(c)));
            if relevant {
                let expected = Choice { id: p.choice.id.clone(), kind: "target".into(), title: TITLE.into(), description: "选择符合数量限制的选项，然后确认。".into(),
                    player_id: player_id(p.seat), options, preview_cards: vec![], min: Some(1), max: Some(1), amount: None, allow_decline: Some(false) };
                if p.choice.id.is_empty() || expected.options.len() < 2 || !same(&p.choice, &expected) { return Err("道途上限必须由原操控者从全部真实道途中牺牲一张".into()); }
            }
        } }
        for seat in 0..self.players.len() {
            if self.pathway_options(seat).len() > 1 && !self.effects.iter().any(|e| matches!(e, Effect::Unique { seat: s } if *s == seat))
                && !self.pending.as_ref().is_some_and(|p| p.seat == seat && matches!(p.resolution, ChoiceResolution::Unique)) {
                return Err("多个道途缺少必须完成的个人控制上限检查".into());
            }
        }
        Ok(())
    }
    pub(crate) fn pathway_choice_payload(&self, p: &Pending, a: &Action) -> RuleResult<()> {
        if matches!(p.resolution, ChoiceResolution::Unique) && p.choice.title == TITLE
            && *a != (Action { choice_id: a.choice_id.clone(), selected: a.selected.clone(), ..Action::new("choose") }) {
            return Err("道途牺牲只接受原待选实例".into());
        }
        Ok(())
    }
    pub(crate) fn xq13_convert_icons(&self, c: &Card, mut parts: (Icons, Icons, Icons)) -> (Icons, Icons, Icons) {
        if c.face_down || catalog::card(&c.definition).kind != "character"
            || !self.in_play(&c.id).is_some_and(|(_, live)| !live.face_down && live.controller == c.controller)
            || !self.current_subtypes(c).iter().any(|s| s == "吸血鬼")
            || !self.attachments.iter().any(|a| a.card.definition == "XQ13" && !a.card.face_down && a.card.controller == c.controller && self.attachment_host_valid(a)) {
            return parts;
        }
        let amount = parts.0.investigation.saturating_add(parts.0.influence).saturating_add(parts.1.investigation).saturating_add(parts.1.influence);
        parts.0.combat = parts.0.combat.saturating_add(amount);
        parts.0.investigation = 0; parts.0.influence = 0; parts.1.investigation = 0; parts.1.influence = 0;
        parts
    }
    pub(crate) fn validate_xq13_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let s = &f.source; let c = &s.card;
        if c.definition != "XQ13" || c.id.is_empty() || c.face_down || f.actor >= self.players.len() || c.controller != f.actor || c.owner >= self.players.len()
            || s.region.is_some() || s.source_region_instance.is_some() || s.attachment_host_instance.is_some() || s.play_source.is_some()
            || s.observed_death.is_some() || s.observed_exposure.is_some() || s.observed_mobility_grant.is_some()
            || f.frame_id.is_empty() || f.frame_id == c.id || f.ability_key != "attach" || f.targets.len() != 1 || !f.steps.is_empty() || f.cursor != 0 || f.chosen_region.is_some()
            || self.batch3_physical_cards().any(|live| live.id == c.id)
            || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)] if ids.len() <= 1 && ids.iter().all(|id| !id.is_empty() && *id != c.id) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()) {
            return Err("焚血之道须保留本方秘社结附及真实支付空程序".into());
        }
        self.mobility_immutable_identity(c)?;
        let t = &f.targets[0]; let host = format!("society:p{}", f.actor);
        if !same(&t.spec, &definition().abilities[0].targets[0]) || t.id != host || t.region_instance.is_some() || t.public.instance_id != host
            || t.public.kind != "society" || t.public.region.is_some() || !t.public.valid || t.public.status != "valid" || t.public.invalid_reason.is_some()
            || t.public.controller.as_deref() != Some(&player_id(f.actor)) || t.public.owner.as_deref() != Some(&player_id(f.actor)) {
            return Err("焚血之道不能改为敌方秘社或地区目标".into());
        }
        Ok(())
    }
    pub(crate) fn validate_xq13_state(&self) -> RuleResult<()> {
        self.validate_pathway_choice()?; let mut ids = BTreeSet::new();
        for item in &self.stack { if let Some(f) = &item.frame { if f.source.card.definition == "XQ13" {
            self.validate_xq13_frame(f)?;
            if !ids.insert(&f.frame_id) || item.id != f.frame_id || item.controller != f.actor || !matches!(f.guard, GuardState::Unchecked)
                || item.deploy_region.is_some() || item.reveal || item.target.as_ref() != Some(&f.targets[0].id)
                || item.card.as_ref().is_none_or(|c| c.definition != "XQ13" || c.face_down || c.owner != f.source.card.owner || c.controller != f.actor) {
                return Err("焚血之道响应堆叠外壳无效".into());
            }
        } } }
        if self.effects.iter().any(|e| matches!(e, Effect::Declare { declaration: d } if d.source.card.definition == "XQ13") || matches!(e, Effect::Frame { frame: f } if f.source.card.definition == "XQ13"))
            || self.pending.as_ref().is_some_and(|p| matches!(&p.resolution, ChoiceResolution::Frame { frame: f, .. } if f.source.card.definition == "XQ13") || matches!(&p.resolution, ChoiceResolution::Declare { declaration: d, .. } if d.source.card.definition == "XQ13")) {
            return Err("焚血之道没有额外触发或可保存续体".into());
        }
        Ok(())
    }
}
