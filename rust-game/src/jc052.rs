//! Complete JC052: printed Leader and one non-targeted original-region entry wave.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::BTreeSet;
const KEY: &str = "jc052-destroy-other-original-region-characters";
const LEADER_TITLE: &str = "领袖：选择本方领袖角色中的一张牺牲";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::JC052DestroyOtherOriginalRegionCharacters => true,
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
    f.source.card.definition == "JC052"
        || f.ability_key == KEY
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
pub(crate) fn definition() -> Definition {
    rules::with_abilities(vec![rules::ability(
        KEY,
        "进场触发：消灭原地区的所有其他角色和暗藏者",
        Timing::Fast,
        vec![],
        vec![],
        vec![Op::JC052DestroyOtherOriginalRegionCharacters],
        Some(Event::Enter),
    )])
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if own_ability(a) && (id != "JC052" || !same(a, &definition().abilities[0])) {
        return Err("只准入JC052完整原地区进场消灭程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "JC052" && !same(d, &definition()))
        || (id != "JC052" && d.abilities.iter().any(own_ability))
    {
        return Err("JC052完整定义不能改写或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_character_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "JC052"
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
        return Err("JC052派遣、秘密派遣和现身没有额外目标或程序".into());
    }
    Ok(())
}
pub(crate) fn printed_leader(id: &str) -> bool {
    let c = catalog::card(id);
    c.kind == "character" && c.keywords.iter().any(|k| k.contains("领袖"))
}
/// A rules contract over current control and printed metadata, not card names/owners.
pub(crate) fn leader_instances<'a>(
    seat: usize,
    candidates: impl IntoIterator<Item = (&'a Card, bool)>,
) -> Vec<String> {
    candidates
        .into_iter()
        .filter(|(c, leader)| *leader && !c.face_down && c.controller == seat)
        .map(|(c, _)| c.id.clone())
        .collect()
}
impl Game {
    pub(crate) fn current_leader(&self, c: &Card) -> bool {
        !c.face_down && printed_leader(&c.definition) && (c.definition != "BQ087" || self.players.get(c.controller).is_some_and(|p|p.society_zone.exposure>0))
    }
    pub(crate) fn leader_options(&self, seat: usize) -> Vec<ChoiceOption> {
        if seat >= self.players.len() {
            return vec![];
        }
        let in_play = self.in_play_cards();
        let ids = leader_instances(
            seat,
            in_play
                .iter()
                .map(|(_, c)| (*c, self.current_leader(c))),
        );
        in_play
            .into_iter()
            .filter(|(_, c)| ids.contains(&c.id))
            .map(|(r, c)| self.option(c, seat, r, None))
            .collect()
    }
    pub(crate) fn queue_leader_checks(&mut self) {
        for seat in 0..self.players.len() {
            if self.leader_options(seat).len() > 1
                && !self
                    .effects
                    .iter()
                    .any(|e| matches!(e, Effect::Unique {seat:s} if *s==seat))
            {
                self.effects.push_front(Effect::Unique { seat });
            }
        }
    }
    pub(crate) fn offer_leader_choice(&mut self, seat: usize) -> bool {
        let options = self.leader_options(seat);
        if options.len() <= 1 {
            return false;
        }
        // Recheck after each sacrifice before queued entry declarations.
        self.effects.push_front(Effect::Unique { seat });
        self.choice(
            seat,
            "target",
            LEADER_TITLE.into(),
            options,
            1,
            1,
            None,
            ChoiceResolution::Unique,
        );
        true
    }
    pub(crate) fn jc052_original_region_live(&self, source: &SourceSnapshot) -> bool {
        source
            .region
            .and_then(|r| self.regions.get(r))
            .is_some_and(|r| {
                !r.vacant && source.source_region_instance.as_ref() == Some(&r.card.id)
            })
    }
    pub(crate) fn jc052_destroy_others(&mut self, f: &ResolutionFrame) {
        if !self.jc052_original_region_live(&f.source) {
            return;
        }
        let region = f.source.region.unwrap();
        let deaths = self.region_characters(region)
            .filter(|c| {
                c.id != f.source.card.id
                    && (c.face_down || catalog::card(&c.definition).kind == "character")
            })
            .map(|c| {
                (
                    c.id.clone(),
                    RemovalCause::Destroy,
                    self.source_snapshot(c, Some(region)),
                )
            })
            .collect();
        self.remove_death_batch(deaths);
    }
    fn validate_jc052_source(&self, s: &SourceSnapshot, actor: usize) -> RuleResult<()> {
        if s.card.definition != "JC052"
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
            return Err("JC052冻结来源、操控者或原地区身份无效".into());
        }
        // A departed source need not exist now. If its UID still names an entity,
        // printed definition and owner are immutable; current control may change.
        let owner_cards = self.players.iter().flat_map(|p| {
            p.hand.iter().chain(&p.deck).chain(&p.assets).chain(&p.graveyard)
                .chain(&p.score_cards).chain(p.society_zone.card.iter())
        });
        let contradicts_source = self.regions.iter()
            .flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(owner_cards)
            .chain(&self.world)
            .chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.stack.iter().filter_map(|item| item.card.as_ref()))
            .chain(self.sealed_cards.iter().map(|sealed| &sealed.card))
            .chain(self.effects.iter().filter_map(|effect| {
                if let Effect::Bury { card } = effect { Some(card) } else { None }
            }))
            .any(|card| card.id == s.card.id
                && (card.definition != s.card.definition || card.owner != s.card.owner));
        if contradicts_source {
            return Err("JC052冻结来源UID与已存在实体的印刷定义或拥有者矛盾".into());
        }
        Ok(())
    }
    fn jc052_trigger(&self, key: &str, source: &SourceSnapshot) -> RuleResult<AbilitySpec> {
        let region = source
            .source_region_instance
            .as_deref()
            .ok_or("JC052缺少原地区身份")?;
        match key {
            KEY => Ok(definition().abilities[0].clone()),
            "renown" => Ok(crate::renown::renown_ability(region)),
            "jc089-combat-glory" => Ok(rules::jc089_combat_glory_ability(region)),
            _ => Err("JC052未知触发程序".into()),
        }
    }
    pub(crate) fn validate_jc052_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_cross_card_frame(f)? { return Ok(()); }
        if !related(f) {
            return Ok(());
        }
        self.validate_jc052_source(&f.source, f.actor)?;
        if f.frame_id.is_empty() || f.cursor != 0 || !f.targets.is_empty() {
            return Err("JC052帧身份、游标或多余目标无效".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if !f.steps.is_empty()
                || f.chosen_region != f.source.region
                || (f.ability_key == "deploy"
                    && (f.source.card.face_down || !f.source.valid_character_play()))
                || (f.ability_key == "reveal"
                    && (!f.source.card.face_down || f.source.play_source.is_some()))
                || !matches!(f.already_paid.as_slice(),[PaidCost::Assets(ids)]
                    if ids.len()<=7 && ids.iter().all(|id|!id.is_empty())
                        && ids.iter().collect::<BTreeSet<_>>().len()==ids.len())
            {
                return Err("JC052只准入真实派遣或付费现身空程序".into());
            }
            return Ok(());
        }
        let expected = self.jc052_trigger(&f.ability_key, &f.source)?;
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
            return Err("JC052仅准入固定原地区进场程序或既有获授奖励".into());
        }
        Ok(())
    }
    fn validate_jc052_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_cross_card_declaration(d)? { return Ok(()); }
        if d.source.card.definition != "JC052" && !own_ability(&d.ability) {
            return Ok(());
        }
        self.validate_jc052_source(&d.source, d.actor)?;
        if d.source.card.face_down
            || d.source.play_source.is_some()
            || !same(&d.ability, &self.jc052_trigger(&d.ability.key, &d.source)?)
        {
            return Err("JC052进场声明或获授奖励不符".into());
        }
        Ok(())
    }
    pub(crate) fn validate_jc052_state(&self) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        for c in self
            .regions
            .iter()
            .flat_map(|r| &r.cards)
            .filter(|c| c.definition == "JC052")
        {
            if c.id.is_empty()
                || !ids.insert(&c.id)
                || c.owner >= self.players.len()
                || c.controller >= self.players.len()
            {
                return Err("JC052在场实例或座位无效".into());
            }
        }
        for s in &self.stack {
            if let Some(f) = &s.frame {
                if self.validated_cross_card_frame(f)? { continue; }
                self.validate_jc052_frame(f)?;
                if related(f) || s.card.as_ref().is_some_and(|c| c.definition == "JC052") {
                    if !related(f)
                        || !matches!(f.guard, GuardState::Unchecked)
                        || s.id != f.frame_id
                        || s.controller != f.actor
                        || s.target.is_some()
                    {
                        return Err("JC052未结算堆栈外壳无效".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if s.card.as_ref().is_none_or(|c| {
                            c.definition != "JC052"
                                || c.face_down
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                        }) || s.deploy_region != f.source.region
                            || s.reveal != (f.ability_key == "reveal")
                        {
                            return Err("JC052派遣/现身卡和地区不符".into());
                        }
                    } else if s.card.is_some() || s.deploy_region.is_some() || s.reveal {
                        return Err("JC052触发堆栈不能伪造进场卡".into());
                    }
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "JC052") {
                return Err("JC052进场缺少冻结帧".into());
            }
        }
        for e in &self.effects {
            match e {
                Effect::Frame { frame } if crate::mobility::related_frame(frame) => return Err("机动不能绕过响应队列".into()),
                Effect::Frame { frame: f } => {
                    self.validate_jc052_frame(f)?;
                    if (f.source.card.definition == "JC052"
                        && matches!(f.ability_key.as_str(), "deploy" | "reveal"))
                        || f.ability_key == KEY
                        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
                    {
                        return Err("JC052触发不能绕过响应堆栈或重复执行".into());
                    }
                }
                Effect::Declare { declaration: d } => self.validate_jc052_declaration(d)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration, .. } if self.validated_cross_card_declaration(declaration)? => {
                    self.validate_mobility_state()?;
                }
                ChoiceResolution::Frame { frame, .. } if crate::mobility::related_frame(frame) => return Err("机动没有帧内续体".into()),
                ChoiceResolution::Frame { frame: f, .. } if related(f) => {
                    return Err("JC052没有可保存的结算续体待选".into())
                }
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } => {
                    self.validate_jc052_declaration(d)?;
                    if d.source.card.definition == "JC052"
                        && (!matches!(stage, DeclareChoice::Accept)
                            || p.seat != d.actor
                            || p.choice.player_id != player_id(d.actor)
                            || p.choice.id.is_empty()
                            || p.choice.kind != "trigger"
                            || p.choice.title
                                != format!("{}：{}", catalog::card("JC052").name, d.ability.label)
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
                        return Err("JC052可选触发的完整待选元数据无效".into());
                    }
                }
                ChoiceResolution::Unique => {
                    let expected = self.leader_options(p.seat);
                    let relevant = p.choice.title == LEADER_TITLE
                        || expected.len() > 1
                        || p.choice.options.iter().any(|o| {
                            self.board(&o.id)
                                .is_some_and(|(_, c)| self.current_leader(c))
                        });
                    if relevant
                        && (p.seat >= self.players.len()
                            || expected.len() < 2
                            || p.choice.id.is_empty()
                            || p.choice.player_id != player_id(p.seat)
                            || p.choice.kind != "target"
                            || p.choice.title != LEADER_TITLE
                            || p.choice.description != "选择符合数量限制的选项，然后确认。"
                            || p.choice.min != Some(1)
                            || p.choice.max != Some(1)
                            || p.choice.allow_decline != Some(false)
                            || p.choice.amount.is_some()
                            || !p.choice.preview_cards.is_empty()
                            || !same(&p.choice.options, &expected))
                    {
                        return Err("领袖强制牺牲待选座位、数量或真实选项无效".into());
                    }
                }
                _ => {}
            }
        }
        for seat in 0..self.players.len() {
            if self.leader_options(seat).len() > 1
                && !self
                    .effects
                    .iter()
                    .any(|e| matches!(e,Effect::Unique {seat:s} if *s==seat))
                && !self.pending.as_ref().is_some_and(|p| {
                    p.seat == seat && matches!(p.resolution, ChoiceResolution::Unique)
                })
            {
                return Err("多领袖必须保留强制牺牲检查或待选".into());
            }
        }
        Ok(())
    }
}
