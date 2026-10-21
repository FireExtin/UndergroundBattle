//! Isolated complete JC046 candidate. A mandatory Reveal uses the existing
//! responsive stack and the seat-zero ForEachLivingPlayer sacrifice program.
use crate::{
    catalog,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::BTreeSet;

pub(crate) const KEY: &str = "jc046-reveal-each-sacrifice";
const TITLE: &str = "选择一个自己操控的角色牺牲";
fn same<T: serde::Serialize>(a: &T, b: &T) -> bool {
    serde_json::to_value(a).unwrap() == serde_json::to_value(b).unwrap()
}
fn uses(ops: &[Op]) -> bool {
    ops.iter().any(|op| match op {
        Op::SacrificeChosen(PlayerRef::Context) => true,
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
    f.source.card.definition == "JC046"
        || f.ability_key == KEY
        || f.steps.iter().any(|s| uses(std::slice::from_ref(&s.op)))
}
pub(crate) fn definition() -> Definition {
    let mut d = rules::with_abilities(vec![rules::ability(
        KEY,
        "现身强制触发：每位玩家各牺牲一个角色",
        Timing::Fast,
        vec![],
        vec![],
        vec![Op::ForEachLivingPlayer(vec![Op::SacrificeChosen(
            PlayerRef::Context,
        )])],
        Some(Event::Reveal),
    )]);
    d.traits.kill = 2;
    d
}
pub(crate) fn validate_ability(id: &str, a: &AbilitySpec) -> RuleResult<()> {
    if own_ability(a) && (id != "JC046" || !same(a, &definition().abilities[0])) {
        return Err("只准入JC046完整强制现身、全玩家牺牲程序".into());
    }
    Ok(())
}
pub(crate) fn validate_definition(id: &str, d: &Definition) -> RuleResult<()> {
    if (id == "JC046" && !same(d, &definition()))
        || (id != "JC046" && d.abilities.iter().any(own_ability))
    {
        return Err("JC046杀伤2与完整现身程序不能改写或移植".into());
    }
    Ok(())
}
pub(crate) fn validate_character_action(id: &str, a: &Action) -> RuleResult<()> {
    if id == "JC046"
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
        return Err("JC046派遣、秘密派遣和现身没有额外目标或程序".into());
    }
    Ok(())
}
impl Game {
    fn validate_jc046_source(&self, s: &SourceSnapshot, actor: usize) -> RuleResult<()> {
        if s.card.definition != "JC046"
            || s.card.id.is_empty()
            || actor >= self.players.len()
            || s.card.owner >= self.players.len()
            || s.card.controller != actor
            || s.region.is_none_or(|r| r >= self.regions.len())
            || s.source_region_instance
                .as_ref()
                .is_some_and(|id| id.is_empty())
            || s.attachment_host_instance.is_some()
            || s.observed_death.is_some()
        {
            return Err("JC046冻结来源、操控者或地区身份无效".into());
        }
        // Owner/print are immutable for an existing UID. Current control, region,
        // readiness and the continued presence of the source are not required.
        let owner_cards = self.players.iter().flat_map(|p| {
            p.hand
                .iter()
                .chain(&p.deck)
                .chain(&p.assets)
                .chain(&p.graveyard)
                .chain(&p.score_cards)
                .chain(p.society_zone.card.iter())
        });
        if self
            .regions
            .iter()
            .flat_map(|r| std::iter::once(&r.card).chain(&r.cards))
            .chain(owner_cards)
            .chain(&self.world)
            .chain(self.attachments.iter().map(|a| &a.card))
            .chain(self.stack.iter().filter_map(|s| s.card.as_ref()))
            .chain(self.sealed_cards.iter().map(|s| &s.card))
            .chain(self.effects.iter().filter_map(|e| match e {
                Effect::Bury { card } => Some(card),
                _ => None,
            }))
            .any(|c| {
                c.id == s.card.id && (c.definition != s.card.definition || c.owner != s.card.owner)
            })
        {
            return Err("JC046冻结来源UID与既有实体的印刷定义或拥有者矛盾".into());
        }
        Ok(())
    }
    fn jc046_trigger(&self, key: &str, s: &SourceSnapshot) -> RuleResult<AbilitySpec> {
        match key {
            KEY => Ok(definition().abilities[0].clone()),
            "renown" | "jc089-combat-glory" => {
                let region = s
                    .source_region_instance
                    .as_deref()
                    .filter(|id| !id.is_empty())
                    .ok_or("JC046获授奖励缺少冻结地区身份")?;
                Ok(if key == "renown" {
                    crate::renown::renown_ability(region)
                } else {
                    rules::jc089_combat_glory_ability(region)
                })
            }
            _ => Err("JC046未知触发程序".into()),
        }
    }
    fn validate_jc046_declaration(&self, d: &Declaration) -> RuleResult<()> {
        if self.validated_mobility_declaration(d)? { return Ok(()); }
        if d.source.card.definition != "JC046" && !own_ability(&d.ability) {
            return Ok(());
        }
        self.validate_jc046_source(&d.source, d.actor)?;
        if d.source.card.face_down
            || d.source.play_source.is_some()
            || !same(&d.ability, &self.jc046_trigger(&d.ability.key, &d.source)?)
        {
            return Err("JC046现身声明或获授奖励不符".into());
        }
        Ok(())
    }
    /// Mandatory means no Accept prompt, not an unrespondable resolution.
    pub(crate) fn jc046_declare(&mut self, d: Declaration) -> RuleResult<bool> {
        self.validate_jc046_declaration(&d)?;
        if !own_ability(&d.ability) {
            return Ok(false);
        }
        if self.players[d.actor].eliminated {
            return Ok(true);
        }
        let label = format!("{}：{}", catalog::card("JC046").name, d.ability.label);
        let frame = self.make_frame(d.actor, d.source, &d.ability, vec![], vec![], None);
        self.dispatch_frame(frame, label, None, ResponsePolicy::Respondable);
        Ok(true)
    }
    pub(crate) fn jc046_reject_optional_declaration(&self, d: &Declaration) -> RuleResult<()> {
        self.validate_jc046_declaration(d)?;
        if own_ability(&d.ability) {
            return Err("JC046强制现身没有可选发动待选".into());
        }
        Ok(())
    }
    fn jc046_stored_frames(&self) -> impl Iterator<Item = &ResolutionFrame> {
        self.stack
            .iter()
            .filter_map(|item| item.frame.as_ref())
            .chain(self.effects.iter().filter_map(|effect| match effect {
                Effect::Frame { frame } => Some(frame.as_ref()),
                _ => None,
            }))
            .chain(
                self.pending
                    .as_ref()
                    .and_then(|pending| match &pending.resolution {
                        ChoiceResolution::Frame { frame, .. } => Some(frame.as_ref()),
                        _ => None,
                    }),
            )
    }
    pub(crate) fn validate_jc046_frame(&self, f: &ResolutionFrame) -> RuleResult<()> {
        if self.validated_mobility_frame(f)? { return Ok(()); }
        if !related(f) {
            return Ok(());
        }
        // A frame moves between containers; it is never copied into a second
        // one. While inspecting stored state, the one identical reference is
        // allowed. During execution the current frame has already been removed,
        // so any remaining occurrence also rejects an in-memory replay.
        if self
            .jc046_stored_frames()
            .any(|other| other.frame_id == f.frame_id && !std::ptr::eq(other, f))
        {
            return Err("JC046帧身份不能在stack、effects或pending中重复".into());
        }
        self.validate_jc046_source(&f.source, f.actor)?;
        if f.frame_id.is_empty() || !f.targets.is_empty() {
            return Err("JC046帧身份或多余目标无效".into());
        }
        if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
            if f.cursor != 0
                || !f.steps.is_empty()
                || f.chosen_region != f.source.region
                || (f.ability_key == "deploy"
                    && (f.source.card.face_down || !f.source.valid_character_play()))
                || (f.ability_key == "reveal"
                    && (!f.source.card.face_down || f.source.play_source.is_some()))
                || !matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)]
                    if ids.len() <= 5 && ids.iter().all(|id| !id.is_empty())
                        && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
            {
                return Err("JC046只准入真实付费派遣或现身空程序".into());
            }
            return Ok(());
        }
        let expected = self.jc046_trigger(&f.ability_key, &f.source)?;
        if f.source.card.face_down
            || f.source.play_source.is_some()
            || !f.already_paid.is_empty()
            || f.chosen_region.is_some()
        {
            return Err("JC046触发不支付费用、不选目标或地区".into());
        }
        if f.ability_key == KEY {
            let seats = f.steps.iter().map(|s| s.context).collect::<Vec<_>>();
            if seats.is_empty()
                || !seats.contains(&f.actor)
                || seats.iter().any(|s| *s >= self.players.len())
                || seats.windows(2).any(|w| w[0] >= w[1])
                || self
                    .players
                    .iter()
                    .any(|p| !p.eliminated && !seats.contains(&p.seat))
                || f.steps
                    .iter()
                    .any(|s| !matches!(s.op, Op::SacrificeChosen(PlayerRef::Context)))
                || f.cursor > f.steps.len()
                || (f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked))
                || (f.cursor > 0 && !matches!(f.guard, GuardState::Accepted))
            {
                return Err("JC046必须保留0起递增的存活玩家冻结续体与真实游标".into());
            }
        } else if f.cursor != 0
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
            return Err("JC046既有获授奖励不能改写".into());
        }
        Ok(())
    }
    fn jc046_options(&self, seat: usize) -> Vec<ChoiceOption> {
        self.regions
            .iter()
            .enumerate()
            .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .filter(|(_, c)| {
                c.controller == seat
                    && !c.face_down
                    && catalog::card(&c.definition).kind == "character"
            })
            .map(|(r, c)| self.option(c, seat, Some(r), None))
            .collect()
    }
    fn validate_jc046_continuation(
        &self,
        f: &ResolutionFrame,
        choice: &FrameChoice,
        chooser: usize,
    ) -> RuleResult<()> {
        self.validate_jc046_frame(f)?;
        if !related(f) {
            return Ok(());
        }
        if f.ability_key != KEY
            || !matches!(f.guard, GuardState::Accepted)
            || f.cursor == 0
            || f.cursor > f.steps.len()
            || !matches!(choice, FrameChoice::Sacrifice { seat } if *seat == chooser && *seat == f.steps[f.cursor - 1].context)
        {
            return Err("JC046牺牲续体、选择者或游标不符".into());
        }
        Ok(())
    }
    pub(crate) fn validate_jc046_choice(
        &self,
        f: &ResolutionFrame,
        choice: &FrameChoice,
        chooser: usize,
        a: &Action,
        selected: &[String],
    ) -> RuleResult<()> {
        if !related(f) {
            return Ok(());
        }
        self.validate_jc046_continuation(f, choice, chooser)?;
        if a.kind != "choose"
            || a.card_id.is_some()
            || a.target_id.is_some()
            || a.region.is_some()
            || a.ability_id.is_some()
            || a.option.is_some()
            || a.top.is_some()
            || a.bottom.is_some()
            || a.allocations.is_some()
            || a.cost_selected.is_some()
            || a.deck_draft.is_some()
            || selected.len() != 1
            || !self
                .jc046_options(chooser)
                .iter()
                .any(|o| o.id == selected[0])
        {
            return Err("JC046必须且只能选择自己当前操控的一个明置角色牺牲".into());
        }
        Ok(())
    }
    fn validate_jc046_suspended_effect(&self, index: usize) -> RuleResult<()> {
        // A chosen sacrifice queues its frame at the front and apply() drains
        // it. Permit a retained frame only behind a complete forced conflict
        // choice and its checks, never as unrelated queued work.
        let p = self
            .pending
            .as_ref()
            .ok_or("JC046续体缺少真实强制冲突待选")?;
        if !matches!(p.resolution, ChoiceResolution::Unique) || p.seat >= self.players.len() {
            return Err("JC046 effects续体只能挂起于真实强制冲突待选".into());
        }
        let leaders = self.leader_options(p.seat);
        let is_leader = leaders.len() > 1;
        let (options, title) = if is_leader {
            (leaders, "领袖：选择本方领袖角色中的一张牺牲")
        } else {
            // Preserve the existing Unique fallback after a leader recheck.
            // This duplicates its current group selection, not its resolution.
            let in_play = self.in_play_cards();
            let name = in_play.iter().find_map(|(_, c)| {
                let name = &catalog::card(&c.definition).name;
                (c.controller == p.seat
                    && !c.face_down
                    && catalog::card(&c.definition).unique
                    && in_play
                        .iter()
                        .filter(|(_, other)| {
                            other.controller == p.seat
                                && !other.face_down
                                && catalog::card(&other.definition).name == *name
                        })
                        .count()
                        > 1)
                .then_some(name)
            });
            let options = in_play
                .iter()
                .filter(|(_, c)| {
                    c.controller == p.seat
                        && !c.face_down
                        && catalog::card(&c.definition).unique
                        && name == Some(&catalog::card(&c.definition).name)
                })
                .map(|(r, c)| self.option(c, p.seat, *r, None))
                .collect::<Vec<_>>();
            let title = if options
                .iter()
                .any(|o| o.card.as_ref().is_some_and(|c| c.kind == "attachment"))
            {
                "独有：选择同名独有牌中的一张牺牲"
            } else {
                "独有：选择同名独有角色中的一张牺牲"
            };
            (options, title)
        };
        let prefix_valid = self
            .effects
            .iter()
            .take(index)
            .all(|e| matches!(e, Effect::Unique { seat } if *seat < self.players.len()));
        let leader_recheck = !is_leader
            || (index > 0
                && matches!(self.effects.front(), Some(Effect::Unique { seat }) if *seat == p.seat));
        if !prefix_valid
            || !leader_recheck
            || options.len() < 2
            || p.choice.id.is_empty()
            || p.choice.player_id != player_id(p.seat)
            || p.choice.kind != "target"
            || p.choice.title != title
            || p.choice.description != "选择符合数量限制的选项，然后确认。"
            || p.choice.min != Some(1)
            || p.choice.max != Some(1)
            || p.choice.allow_decline != Some(false)
            || p.choice.amount.is_some()
            || !p.choice.preview_cards.is_empty()
            || !same(&p.choice.options, &options)
        {
            return Err("JC046挂起续体必须紧随完整强制冲突检查及其真实待选".into());
        }
        Ok(())
    }
    pub(crate) fn validate_jc046_state(&self) -> RuleResult<()> {
        let mut ids = BTreeSet::new();
        for c in self
            .regions
            .iter()
            .flat_map(|r| &r.cards)
            .filter(|c| c.definition == "JC046")
        {
            if c.id.is_empty()
                || !ids.insert(&c.id)
                || c.owner >= self.players.len()
                || c.controller >= self.players.len()
            {
                return Err("JC046在场实例或座位无效".into());
            }
        }
        for s in &self.stack {
            if let Some(f) = &s.frame {
                if self.validated_mobility_frame(f)? { continue; }
                self.validate_jc046_frame(f)?;
                if related(f) || s.card.as_ref().is_some_and(|c| c.definition == "JC046") {
                    if !related(f)
                        || !matches!(f.guard, GuardState::Unchecked)
                        || f.cursor != 0
                        || s.id != f.frame_id
                        || s.controller != f.actor
                        || s.target.is_some()
                    {
                        return Err("JC046未结算堆栈外壳无效".into());
                    }
                    if matches!(f.ability_key.as_str(), "deploy" | "reveal") {
                        if s.card.as_ref().is_none_or(|c| {
                            c.definition != "JC046"
                                || c.face_down
                                || c.owner != f.source.card.owner
                                || c.controller != f.actor
                        }) || s.deploy_region != f.source.region
                            || s.reveal != (f.ability_key == "reveal")
                        {
                            return Err("JC046派遣或现身卡和地区不符".into());
                        }
                    } else if s.card.is_some() || s.deploy_region.is_some() || s.reveal {
                        return Err("JC046触发堆栈不能伪造进场卡".into());
                    }
                }
            } else if s.card.as_ref().is_some_and(|c| c.definition == "JC046")
                || s.label
                    == format!(
                        "{}：{}",
                        catalog::card("JC046").name,
                        definition().abilities[0].label
                    )
            {
                return Err("JC046进场或强制现身触发缺少冻结帧".into());
            }
        }
        for (index, e) in self.effects.iter().enumerate() {
            match e {
                Effect::Frame { frame } if crate::mobility::related_frame(frame) => return Err("机动不能绕过响应队列".into()),
                Effect::Frame { frame: f } => {
                    self.validate_jc046_frame(f)?;
                    if related(f)
                        && (matches!(f.ability_key.as_str(), "deploy" | "reveal")
                            || (f.ability_key == KEY
                                && (f.cursor == 0 || !matches!(f.guard, GuardState::Accepted))))
                    {
                        return Err("JC046强制现身不能绕过响应堆栈或重放已执行牺牲".into());
                    }
                    if f.ability_key == KEY {
                        self.validate_jc046_suspended_effect(index)?;
                    }
                }
                Effect::Declare { declaration } => self.validate_jc046_declaration(declaration)?,
                _ => {}
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration, .. } if self.validated_mobility_declaration(declaration)? => {
                    self.validate_mobility_state()?;
                }
                ChoiceResolution::Frame { frame, .. } if crate::mobility::related_frame(frame) => return Err("机动没有帧内续体".into()),
                ChoiceResolution::Frame { frame: f, choice } if related(f) => {
                    self.validate_jc046_continuation(f, choice, p.seat)?;
                    let options = self.jc046_options(p.seat);
                    if options.is_empty()
                        || p.choice.id.is_empty()
                        || p.choice.player_id != player_id(p.seat)
                        || p.choice.kind != "target"
                        || p.choice.title != TITLE
                        || p.choice.description != "选择符合数量限制的选项，然后确认。"
                        || p.choice.min != Some(1)
                        || p.choice.max != Some(1)
                        || p.choice.allow_decline != Some(false)
                        || p.choice.amount.is_some()
                        || !p.choice.preview_cards.is_empty()
                        || !same(&p.choice.options, &options)
                    {
                        return Err("JC046强制牺牲待选元数据或完整选项无效".into());
                    }
                }
                ChoiceResolution::Declare {
                    declaration: d,
                    stage,
                } => {
                    self.jc046_reject_optional_declaration(d)?;
                    if d.source.card.definition == "JC046"
                        && (!matches!(stage, DeclareChoice::Accept)
                            || p.seat != d.actor
                            || p.choice.player_id != player_id(d.actor)
                            || p.choice.id.is_empty()
                            || p.choice.kind != "trigger"
                            || p.choice.title
                                != format!("{}：{}", catalog::card("JC046").name, d.ability.label)
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
                        return Err("JC046既有获授奖励的可选声明无效".into());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
