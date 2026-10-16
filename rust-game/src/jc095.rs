//! Complete JC095: printed Leader/Renown and optional original-region conversion.
//! The frozen region/actor convention is this project's JC095 interpretation,
//! not a claim of a JC095-specific official FAQ.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::BTreeSet;
const KEY: &str = "jc095-convert-enemy-original-region-influence";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JC095ConvertEnemyInfluenceInOriginalRegion => true,
        Op::ForEachLivingPlayer(body) | Op::ForEachLivingPlayerFromActor(body) => uses(body),
        Op::IfTargetExhausted {
            exhausted, ready, ..
        } => uses(std::slice::from_ref(exhausted)) || uses(std::slice::from_ref(ready)),
        _ => false,
    })
}
fn own_ability(a: &AbilitySpec) -> bool {
    a.key == KEY || uses(&a.ops) || a.modes.iter().any(|m| uses(&m.ops))
}
fn related(f: &ResolutionFrame) -> bool {
    f.source.card.definition == "JC095"
        || f.ability_key == KEY
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
pub(crate) fn definition() -> Definition {
    let mut d = rules::with_abilities(vec![rules::ability(
        KEY,
        "进场触发：移除原地区所有敌方势力并放置等量本方势力",
        Timing::Fast,
        vec![],
        vec![],
        vec![Op::JC095ConvertEnemyInfluenceInOriginalRegion],
        Some(Event::Enter),
    )]);
    d.traits.renown = true;
    d
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if own_ability(a) && (id != "JC095" || !same(a, &definition().abilities[0])) {
        return Err("只准入JC095完整原地区进场势力转换程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "JC095" && !same(d, &definition()))
        || (id != "JC095" && d.abilities.iter().any(own_ability))
    {
        return Err("JC095完整定义不能改写或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_character_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "JC095"
        && (a.target_id.is_some()
            || a.ability_id.is_some()
            || a.option.is_some()
            || a.choice_id.is_some()
            || a.selected.is_some()
            || a.top.is_some()
            || a.bottom.is_some()
            || a.allocations.is_some()
            || a.cost_selected.is_some()
            || a.deck_draft.is_some()
            || (a.kind == "reveal" && a.region.is_some()))
    {
        return Err("JC095派遣、秘密派遣和现身没有额外目标或程序".into());
    }
    Ok(())
}
impl Game {
    pub(crate) fn jc095_original_region_live(&self, source: &SourceSnapshot) -> bool {
        source
            .region
            .and_then(|r| self.regions.get(r))
            .is_some_and(|r| {
                !r.vacant && source.source_region_instance.as_ref() == Some(&r.card.id)
            })
    }
    pub(crate) fn jc095_convert_enemy_influence(&mut self, f: &ResolutionFrame) {
        if !self.jc095_original_region_live(&f.source) {
            return;
        }
        let region = f.source.region.unwrap();
        let enemy = 1 - self.team(f.actor);
        // Read after all responses. Removing first is essential: merely placing
        // N friendly markers would cancel the enemy N and produce zero.
        let amount = self.regions[region].influence[enemy];
        if amount == 0 {
            return;
        }
        self.regions[region].influence[enemy] = 0;
        self.place_influence(f.actor, region, amount);
        // Existing WinContext owns any already-decided victory. The ordinary
        // Award guard prevents a second award for that region incarnation.
        self.note(format!(
            "唐·格鲁尼将地区{}的{}个敌方势力转为本方势力",
            region + 1,
            amount
        ));
    }
    fn validate_jc095_source(&self, s: &SourceSnapshot, actor: usize) -> RuleResult<()> {
        if s.card.definition != "JC095"
            || s.card.id.is_empty()
            || actor >= self.players.len()
            || s.card.owner >= self.players.len()
            || s.card.controller != actor
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance
                .as_ref()
                .is_none_or(|id| id.is_empty())
            || s.attachment_host_instance.is_some()
            || s.observed_death.is_some()
        {
            return Err("JC095冻结来源、操控者或原地区身份无效".into());
        }
        // A departed source need not exist now. If its UID still names an entity,
        // printed definition and owner are immutable; current control may change.
        let owner_cards = self.players.iter().flat_map(|p| {
            p.hand
                .iter()
                .chain(&p.deck)
                .chain(&p.assets)
                .chain(&p.graveyard)
                .chain(&p.score_cards)
                .chain(p.society_zone.card.iter())
        });
        let contradicts_source = self
            .regions
            .iter()
            .flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(owner_cards)
            .chain(&self.world)
            .chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.stack.iter().filter_map(|item| item.card.as_ref()))
            .chain(self.sealed_cards.iter().map(|sealed| &sealed.card))
            .chain(self.effects.iter().filter_map(|effect| {
                if let Effect::Bury { card } = effect {
                    Some(card)
                } else {
                    None
                }
            }))
            .any(|card| {
                card.id == s.card.id
                    && (card.definition != s.card.definition || card.owner != s.card.owner)
            });
        if contradicts_source {
            return Err("JC095冻结来源UID与已存在实体的印刷定义或拥有者矛盾".into());
        }
        Ok(())
    }
    fn jc095_trigger(&self, key: &str, source: &SourceSnapshot) -> RuleResult<AbilitySpec> {
        let region = source
            .source_region_instance
            .as_deref()
            .ok_or("JC095缺少原地区身份")?;
        match key {
            KEY => Ok(definition().abilities[0].clone()),
            "renown" => Ok(crate::renown::renown_ability(region)),
            "jc089-combat-glory" => Ok(rules::jc089_combat_glory_ability(region)),
            _ => Err("JC095未知触发程序".into()),
        }
    }
    pub(crate) fn validate_jc095_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_mobility_frame(f)? { return Ok(()); }
        if !related(f) {
            return Ok(());
        }
        self.validate_jc095_source(&f.source, f.actor)?;
        if f.frame_id.is_empty()
            || f.cursor != 0
            || !f.targets.is_empty()
            || !matches!(f.guard, GuardState::Unchecked)
        {
            return Err("JC095帧身份、游标或多余目标无效".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty()
                || f.chosen_region != f.source.region
                || (f.ability_key == "deploy"
                    && (f.source.card.face_down || f.source.play_source != Some(PlaySource::Hand)))
                || (f.ability_key == "reveal"
                    && (!f.source.card.face_down || f.source.play_source.is_some()))
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]
                    if ids.len()<=7 && ids.iter().all(|id|!id.is_empty())
                        && ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
            {
                return Err("JC095只准入真实派遣或付费现身空程序".into());
            }
            return Ok(());
        }
        let expected = self.jc095_trigger(&f.ability_key, &f.source)?;
        if f.source.card.face_down
            || f.source.play_source.is_some()
            || !f.already_paid.is_empty()
            || f.chosen_region.is_some()
            || !same(
                &f.steps,
                &expected
                    .ops
                    .iter()
                    .map(|op| Step {
                        context: f.actor,
                        op: op.clone(),
                    })
                    .collect::<Vec<_>>(),
            )
        {
            return Err("JC095仅准入固定原地区进场程序或既有获授奖励".into());
        }
        Ok(())
    }
    fn validate_jc095_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_mobility_declaration(d)? { return Ok(()); }
        if d.source.card.definition != "JC095" && !own_ability(&d.ability) {
            return Ok(());
        }
        self.validate_jc095_source(&d.source, d.actor)?;
        if d.source.card.face_down
            || d.source.play_source.is_some()
            || !same(&d.ability, &self.jc095_trigger(&d.ability.key, &d.source)?)
        {
            return Err("JC095进场声明或获授奖励不符".into());
        }
        Ok(())
    }
    pub(crate) fn validate_jc095_state(&self) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        for c in self
            .regions
            .iter()
            .flat_map(|r| &r.cards)
            .filter(|c| c.definition == "JC095")
        {
            if c.id.is_empty()
                || !ids.insert(&c.id)
                || c.owner >= self.players.len()
                || c.controller >= self.players.len()
            {
                return Err("JC095在场实例或座位无效".into());
            }
        }
        for s in &self.stack {
            if let Some(f) = &s.frame {
                if self.validated_mobility_frame(f)? { continue; }
                self.validate_jc095_frame(f)?;
                if related(f) || s.card.as_ref().is_some_and(|c| c.definition == "JC095") {
                    if !related(f)
                        || !matches!(f.guard, GuardState::Unchecked)
                        || s.id != f.frame_id
                        || s.controller != f.actor
                        || s.target.is_some()
                    {
                        return Err("JC095未结算堆栈外壳无效".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if s.card.as_ref().is_none_or(|c| {
                            c.definition != "JC095"
                                || c.face_down
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                        }) || s.deploy_region != f.source.region
                            || s.reveal != (f.ability_key == "reveal")
                        {
                            return Err("JC095派遣/现身卡和地区不符".into());
                        }
                    } else if s.card.is_some() || s.deploy_region.is_some() || s.reveal {
                        return Err("JC095触发堆栈不能伪造进场卡".into());
                    }
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "JC095") {
                return Err("JC095进场缺少冻结帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame } if crate::mobility::related_frame(frame) => return Err("机动不能绕过响应队列".into()),
                Effect::Frame { frame: f } => {
                    self.validate_jc095_frame(f)?;
                    if (f.source.card.definition == "JC095"
                        && matches!(f.ability_key.as_str(), "deploy" | "reveal"))
                        || f.ability_key == KEY
                        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
                    {
                        return Err("JC095触发不能绕过响应堆栈或重复执行".into());
                    }
                }
                Effect::Declare { declaration: d } => self.validate_jc095_declaration(d)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration, .. } if self.validated_mobility_declaration(declaration)? => {
                    self.validate_mobility_state()?;
                }
                ChoiceResolution::Frame { frame, .. } if crate::mobility::related_frame(frame) => return Err("机动没有帧内续体".into()),
                ChoiceResolution::Frame { frame: f, .. } if related(f) => {
                    return Err("JC095没有可保存的结算续体待选".into())
                }
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } => {
                    self.validate_jc095_declaration(d)?;
                    if d.source.card.definition == "JC095"
                        && (!matches!(stage, DeclareChoice::Accept)
                            || p.seat != d.actor
                            || p.choice.player_id != player_id(d.actor)
                            || p.choice.id.is_empty()
                            || p.choice.kind != "trigger"
                            || p.choice.title
                                != format!("{}：{}", catalog::card("JC095").name, d.ability.label)
                            || p.choice.description != "选择符合数量限制的选项，然后确认。"
                            || p.choice.min != Some(0)
                            || p.choice.max != Some(1)
                            || p.choice.allow_decline != Some(true)
                            || p.choice.amount.is_some()
                            || !p.choice.preview_cards.is_empty()
                            || !same(
                                &p.choice.options,
                                &vec![ChoiceOption {
                                    id: "accept".into(),
                                    label: "发动触发能力".into(),
                                    card: None,
                                }],
                            ))
                    {
                        return Err("JC095可选触发的完整待选元数据无效".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
