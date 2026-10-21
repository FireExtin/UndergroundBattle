//! Interrogate binds three independent targets before payment. The existing
//! selected field carries the one repression player; it is not a paid cost.
use crate::{batch20, engine::RuleResult, model::*, rules::*};
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool { serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap() }
pub(crate) fn validate_action(a: &Action) -> RuleResult<()> {
    if a.kind != "play" || a.region.is_none() || a.target_id.is_none()
        || a.selected.as_ref().is_none_or(|s| s.len() != 1)
        || *a != (Action { card_id: a.card_id.clone(), target_id: a.target_id.clone(), region: a.region,
            selected: a.selected.clone(), ability_id: a.ability_id.clone(), ..Action::new("play") }) {
        return Err("审问嫌疑人须先指定地区、牺牲方对手及独立的遏制玩家".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn jc065_action_candidates(&self, actor: usize, c: &Card) -> Vec<(Action,String)> {
        let source = self.source_snapshot(c, None);
        let ability = batch20::definition("JC065").unwrap().abilities.remove(0);
        let regions = self.binding_options(actor, &source, &ability.targets[0]);
        let opponents = self.binding_options(actor, &source, &ability.targets[1]);
        let players = self.binding_options(actor, &source, &ability.targets[2]);
        let mut out = vec![];
        for r in &regions { for opponent in &opponents { for player in &players {
            out.push((Action { card_id: Some(c.id.clone()), ability_id: Some(ability.key.clone()),
                region: r.id.strip_prefix("region:").and_then(|s| s.parse().ok()), target_id: Some(opponent.id.clone()),
                selected: Some(vec![player.id.clone()]), ..Action::new("play") },
                format!("审问嫌疑人：遏制 {} → {}；牺牲方 {}", player.label, r.label, opponent.label)));
        } } }
        out
    }
    pub(crate) fn jc065_bind_action(&self, actor: usize, source: &SourceSnapshot, spec: &AbilitySpec, a: &Action) -> RuleResult<Vec<BoundTarget>> {
        validate_action(a)?;
        let ids = [format!("region:{}", a.region.unwrap()), a.target_id.clone().unwrap(), a.selected.as_ref().unwrap()[0].clone()];
        spec.targets.iter().zip(ids).enumerate().map(|(i,(slot,id))| {
            if !self.valid_binding(actor, source, slot, &id) { return Err("审问的地区、对手或遏制玩家无效".into()); }
            Ok(BoundTarget { public: self.public_target(actor, source, slot, &id), id, spec: slot.clone(),
                region_instance: (i == 0).then(|| self.regions[a.region.unwrap()].card.id.clone()) })
        }).collect()
    }
    pub(crate) fn validate_jc065_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        let r = f.targets.first().and_then(|t| t.id.strip_prefix("region:")).and_then(|s| s.parse::<usize>().ok());
        if f.ability_key != "batch20-jc065-interrogate" || f.targets.len() != 3 || r != f.chosen_region
            || r.is_none_or(|r| r >= self.regions.len()) || f.targets[0].region_instance.as_ref().is_none_or(|s| s.is_empty())
            || f.source.region.is_some() || f.source.source_region_instance.is_some() {
            return Err("审问的固定程序、原地区或来源无效".into());
        }
        for i in [1,2] {
            let s = f.targets[i].id.strip_prefix('p').and_then(|s| s.parse::<usize>().ok()).filter(|s| *s < self.players.len()).ok_or("审问玩家目标无效")?;
            if f.targets[i].id != player_id(s) || (i == 1 && self.team(s) == self.team(f.actor)) { return Err("牺牲方必须是目标对手".into()); }
        }
        Ok(())
    }
    fn jc065_sacrifice_options(&self, seat: usize, region: usize) -> Vec<ChoiceOption> {
        self.regions[region].cards.iter().filter(|c| c.controller == seat && c.face_down)
            .map(|c| self.option(c, seat, Some(region), None)).collect()
    }
    pub(crate) fn jc065_sacrifice_start(&mut self, f: ResolutionFrame) -> RuleResult<()> {
        let seat = f.targets[1].id[1..].parse::<usize>().map_err(|_| "牺牲方无效")?;
        let region = f.chosen_region.ok_or("审问地区缺失")?;
        let options = self.jc065_sacrifice_options(seat, region);
        if options.is_empty() { return self.resolve_frame(f); }
        let uid = f.targets[0].region_instance.clone().ok_or("审问原地区缺失")?;
        self.choice(seat, "sacrifice", "审问嫌疑人：牺牲本地区一个本方暗藏者".into(), options, 1, 1, None,
            ChoiceResolution::Frame { frame: Box::new(f), choice: FrameChoice::JC065Sacrifice { seat, region, region_instance: uid } });
        Ok(())
    }
    pub(crate) fn validate_jc065_pending(&self, p: &Pending, f: &ResolutionFrame, seat: usize, region: usize, uid: &str) -> RuleResult<()> {
        self.validate_batch20_frame(f)?;
        if f.cursor != 1 || !matches!(f.guard, GuardState::Accepted) || seat >= self.players.len()
            || f.targets[1].id != player_id(seat) || f.chosen_region != Some(region) || region >= self.regions.len()
            || f.targets[0].region_instance.as_deref() != Some(uid) || self.regions[region].card.id != uid || self.regions[region].vacant
            || p.seat != seat || p.choice.player_id != player_id(seat) || p.choice.kind != "sacrifice"
            || p.choice.title != "审问嫌疑人：牺牲本地区一个本方暗藏者" || p.choice.min != Some(1) || p.choice.max != Some(1)
            || p.choice.allow_decline != Some(false) || p.choice.amount.is_some() || p.choice.options.is_empty()
            || !same(&p.choice.options, &self.jc065_sacrifice_options(seat, region)) {
            return Err("审问牺牲的选择者、原地区或暗藏者选项无效".into());
        }
        Ok(())
    }
    pub(crate) fn jc065_sacrifice_complete(&mut self, f: ResolutionFrame, chooser: usize, seat: usize, region: usize, uid: &str, selected: &[String]) -> RuleResult<()> {
        self.validate_batch20_frame(&f)?;
        if chooser != seat || selected.len() != 1 || self.regions.get(region).is_none_or(|r| r.card.id != uid)
            || !self.jc065_sacrifice_options(seat, region).iter().any(|o| o.id == selected[0]) { return Err("须牺牲自己在原地区的一个暗藏者".into()); }
        self.remove_dead(&selected[0], RemovalCause::Sacrifice);
        self.resolve_frame(f)
    }
}
