//! Shared declaration, payment, target guard and resumable finite operation interpreter.
use crate::{
    catalog::card,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::{BTreeMap, BTreeSet};

impl Game {
    fn frame_region(&self, frame: &ResolutionFrame, region: RegionRef) -> Option<usize> {
        let region = match region {
            RegionRef::SourceRegion => frame.source.region,
            RegionRef::Chosen => frame.chosen_region,
            RegionRef::Target(i) => frame
                .targets
                .get(i)
                .and_then(|t| t.id.strip_prefix("region:"))
                .and_then(|s| s.parse().ok()),
        };
        region.filter(|r| *r < self.regions.len())
    }
    // Non-target queries deliberately do not apply targetability, barriers or shields.
    fn matching_board(&self, frame: &ResolutionFrame, selector: &BoardSelector) -> Vec<String> {
        self.regions
            .iter()
            .enumerate()
            .flat_map(|(r, region)| region.cards.iter().map(move |c| (r, c)))
            .filter(|(r, c)| {
                let d = card(&c.definition);
                let kind = match selector.kind {
                    EntityKind::Any => true,
                    EntityKind::Character => !c.face_down && d.kind == "character",
                    EntityKind::Hidden => c.face_down,
                    EntityKind::CharacterOrHidden => c.face_down || d.kind == "character",
                };
                let relation = match selector.relation {
                    Relation::Any => true,
                    Relation::ControlledByActor => c.controller == frame.actor,
                    Relation::OwnedByActor => c.owner == frame.actor,
                    Relation::EnemyTeam => self.is_enemy(frame.actor, c),
                };
                kind && relation
                    && selector
                        .region
                        .is_none_or(|wanted| self.frame_region(frame, wanted) == Some(*r))
                    && selector
                        .subtype
                        .as_ref()
                        .is_none_or(|s| !c.face_down && d.subtypes.contains(s))
            })
            .map(|(_, c)| c.id.clone())
            .collect()
    }
    pub(crate) fn rule_action_candidates(
        &self,
        actor: usize,
        c: &Card,
        region: Option<usize>,
        kind: &str,
    ) -> Vec<(Action, String)> {
        let source = self.source_snapshot(c, region);
        let mut result = vec![];
        for ability in rules::definition(&c.definition)
            .abilities
            .iter()
            .filter(|a| a.event.is_none())
        {
            let modes = if ability.modes.is_empty() {
                vec![None]
            } else {
                ability.modes.iter().map(Some).collect()
            };
            for mode in modes {
                let slots = mode.map_or(&ability.targets, |m| &m.targets);
                if slots.len() > 1 {
                    continue;
                }
                let targets = slots
                    .first()
                    .map(|s| {
                        self.binding_options(actor, &source, s)
                            .into_iter()
                            .map(Some)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_else(|| vec![None]);
                let ops = mode.map_or(&ability.ops, |m| &m.ops);
                let regions = if ops
                    .iter()
                    .any(|op| matches!(op, Op::Move(_, Destination::HiddenInChosenRegion)))
                {
                    (0..self.regions.len()).map(Some).collect()
                } else {
                    vec![None]
                };
                let costs = if ability
                    .costs
                    .iter()
                    .any(|c| matches!(c, Cost::SacrificeSelectedControlledCharacter))
                {
                    self.regions
                        .iter()
                        .flat_map(|r| r.cards.iter())
                        .filter(|c| {
                            c.controller == actor
                                && !c.face_down
                                && card(&c.definition).kind == "character"
                        })
                        .map(|c| Some(vec![c.id.clone()]))
                        .collect()
                } else {
                    vec![None]
                };
                for target in &targets {
                    for region in &regions {
                        for cost in &costs {
                            let mut a = Action {
                                card_id: Some(c.id.clone()),
                                ability_id: Some(ability.key.clone()),
                                option: mode.map(|m| m.key.clone()),
                                region: *region,
                                cost_selected: cost.clone(),
                                ..Action::new(kind)
                            };
                            if let Some(t) = target {
                                if slots[0].zone == Zone::Region {
                                    a.region =
                                        t.id.strip_prefix("region:").and_then(|s| s.parse().ok());
                                } else {
                                    a.target_id = Some(t.id.clone());
                                }
                            }
                            let label = format!(
                                "{}：{}{}{}",
                                card(&c.definition).name,
                                mode.map_or(ability.label.as_str(), |m| m.label.as_str()),
                                target
                                    .as_ref()
                                    .map(|t| format!(
                                        " → {}{} [{}]",
                                        t.label,
                                        t.card
                                            .as_ref()
                                            .and_then(|c| c.region)
                                            .map(|r| format!("·地区{}", r + 1))
                                            .unwrap_or_default(),
                                        t.id
                                    ))
                                    .unwrap_or_default(),
                                cost.as_ref()
                                    .and_then(|ids| ids.first())
                                    .and_then(|id| self.board(id).map(|(r, c)| format!(
                                        "（费用：牺牲{}·地区{} [{}]）",
                                        card(&c.definition).name,
                                        r + 1,
                                        c.id
                                    )))
                                    .unwrap_or_default()
                            );
                            result.push((a, label));
                        }
                    }
                }
            }
        }
        result
    }
    pub(crate) fn public_target(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        spec: &TargetSlotSpec,
        id: &str,
    ) -> TargetSummary {
        let located = self.board(id).map(|(r, c)| (Some(r), c)).or_else(|| {
            self.players
                .iter()
                .flat_map(|p| p.graveyard.iter())
                .find(|c| c.id == id)
                .map(|c| (None, c))
        });
        let (label, kind, owner, controller, region) = if let Some((region, c)) = located {
            let label = if c.face_down {
                format!("暗藏者·{}", self.players[c.controller].name)
            } else {
                card(&c.definition).name.clone()
            };
            (
                label,
                if c.face_down {
                    "hidden"
                } else {
                    card(&c.definition).kind.as_str()
                }
                .into(),
                Some(player_id(c.owner)),
                Some(player_id(c.controller)),
                region,
            )
        } else if let Some(seat) = id
            .strip_prefix('p')
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|s| *s < self.players.len())
        {
            (
                self.players[seat].name.clone(),
                "player".into(),
                None,
                Some(player_id(seat)),
                None,
            )
        } else if let Some(r) = id
            .strip_prefix("region:")
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|r| *r < self.regions.len())
        {
            (
                card(&self.regions[r].card.definition).name.clone(),
                "region".into(),
                None,
                None,
                Some(r),
            )
        } else {
            ("目标".into(), "unknown".into(), None, None, None)
        };
        TargetSummary {
            instance_id: id.into(),
            label,
            kind,
            owner,
            controller,
            region,
            valid: self.valid_binding(actor, source, spec, id),
            status: "valid".into(),
            invalid_reason: None,
        }
    }
    pub(crate) fn target_summary(
        &self,
        frame: &ResolutionFrame,
        target: &BoundTarget,
    ) -> TargetSummary {
        let mut summary = target.public.clone();
        if self.board(&target.id).is_some()
            || self
                .players
                .iter()
                .any(|p| p.graveyard.iter().any(|c| c.id == target.id))
        {
            let current = self.public_target(frame.actor, &frame.source, &target.spec, &target.id);
            summary.owner = current.owner;
            summary.controller = current.controller;
            summary.region = current.region;
            if summary.kind != "hidden" {
                summary.label = current.label;
                summary.kind = current.kind;
            }
        }
        if matches!(frame.guard, GuardState::Accepted) {
            summary.valid = true;
            summary.status = "guardAccepted".into();
            return summary;
        }
        summary.valid = self.valid_binding(frame.actor, &frame.source, &target.spec, &target.id);
        if !summary.valid {
            let present = match target.spec.zone {
                Zone::Board => self.board(&target.id).is_some(),
                Zone::Graveyard => self
                    .players
                    .iter()
                    .any(|p| p.graveyard.iter().any(|c| c.id == target.id)),
                Zone::Player | Zone::Region => true,
            };
            summary.status = if present { "changed" } else { "missing" }.into();
            summary.invalid_reason = Some(
                if present {
                    "目标不再满足地区、类别或操控条件"
                } else {
                    "原目标已离场"
                }
                .into(),
            );
        }
        summary
    }
    pub(crate) fn frame_view(
        &self,
        frame: &ResolutionFrame,
        label: String,
        state: &str,
    ) -> StackView {
        StackView {
            id: frame.frame_id.clone(),
            label,
            controller: player_id(frame.actor),
            card_id: Some(frame.source.card.definition.clone()),
            target_id: frame.targets.first().map(|t| t.id.clone()),
            ability_id: Some(frame.ability_key.clone()),
            targets: frame.targets.iter().map(|t| t.id.clone()).collect(),
            target_summaries: frame
                .targets
                .iter()
                .map(|t| self.target_summary(frame, t))
                .collect(),
            resolution_state: state.into(),
        }
    }
    pub(crate) fn source_snapshot(&self, c: &Card, region: Option<usize>) -> SourceSnapshot {
        SourceSnapshot {
            card: c.clone(),
            region,
        }
    }
    pub(crate) fn filter_card(&self, filter: &CardFilter, c: &Card) -> bool {
        let d = card(&c.definition);
        match filter {
            CardFilter::Any => true,
            CardFilter::Kind(k) => d.kind == *k,
            CardFilter::SocietyOrMagic { society, magic } => {
                d.society == *society || d.magic_icon == *magic
            }
        }
    }
    pub(crate) fn effective_cost(&self, actor: usize, c: &Card) -> u32 {
        card(&c.definition).cost.saturating_sub(
            self.modifiers
                .iter()
                .filter(|m| {
                    m.actor == actor
                        && m.uses > 0
                        && m.expires_turn == self.turn
                        && self.filter_card(&m.filter, c)
                })
                .map(|m| m.amount)
                .sum(),
        )
    }
    pub(crate) fn pay_printed(
        &mut self,
        actor: usize,
        c: &Card,
        face_up_play: bool,
    ) -> RuleResult<Vec<PaidCost>> {
        let amount = if face_up_play {
            self.effective_cost(actor, c)
        } else {
            card(&c.definition).cost
        };
        let ids = self.players[actor]
            .assets
            .iter()
            .filter(|a| !a.exhausted)
            .take(amount as usize)
            .map(|a| a.id.clone())
            .collect();
        self.pay(actor, amount)?;
        let used = self
            .modifiers
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                m.actor == actor
                    && m.uses > 0
                    && m.expires_turn == self.turn
                    && self.filter_card(&m.filter, c)
            })
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        if face_up_play {
            for i in used {
                self.modifiers[i].uses -= 1;
            }
        }
        Ok(vec![PaidCost::Assets(ids)])
    }
    pub(crate) fn ability_for_action(
        &self,
        definition: &str,
        a: &Action,
    ) -> RuleResult<AbilitySpec> {
        let mut candidates = rules::definition(definition)
            .abilities
            .iter()
            .filter(|s| s.event.is_none());
        let spec = if let Some(key) = &a.ability_id {
            candidates.find(|s| s.key == *key)
        } else {
            let first = candidates.next();
            if candidates.next().is_some() {
                return Err("请选择具体能力".into());
            }
            first
        }
        .ok_or("该牌无可发动的行动能力")?;
        let mut spec = spec.clone();
        if !spec.modes.is_empty() {
            let mode = spec
                .modes
                .iter()
                .find(|m| a.option.as_deref() == Some(&m.key))
                .ok_or("请选择模式")?;
            spec.targets = mode.targets.clone();
            spec.ops = mode.ops.clone();
            spec.modes.clear();
        }
        Ok(spec)
    }
    pub(crate) fn check_timing(&self, actor: usize, spec: &AbilitySpec) -> RuleResult<()> {
        if !self.can_fast(actor) {
            return Err("当前没有快速行动权".into());
        }
        match spec.timing {
            Timing::Standard if !self.can_standard(actor) => Err("该能力为标准行动".into()),
            Timing::ActionFast if !matches!(self.window, Some(Window::Action(_))) => {
                Err("该能力只可在行动阶段发动".into())
            }
            _ => Ok(()),
        }
    }
    pub(crate) fn valid_binding(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        spec: &TargetSlotSpec,
        id: &str,
    ) -> bool {
        match spec.zone {
            Zone::Player => id
                .strip_prefix('p')
                .and_then(|s| s.parse::<usize>().ok())
                .is_some_and(|s| s < self.players.len() && !self.players[s].eliminated),
            Zone::Region => id
                .strip_prefix("region:")
                .and_then(|s| s.parse::<usize>().ok())
                .is_some_and(|r| {
                    r < self.regions.len()
                        && (spec.range != Range::Mobility
                            || source.region.is_some_and(|from| {
                                r != from && (self.mode == "duel" || r.abs_diff(from) == 1)
                            }))
                }),
            Zone::Board | Zone::Graveyard => {
                let located = if spec.zone == Zone::Board {
                    self.board(id).map(|(r, c)| (Some(r), c))
                } else {
                    self.players
                        .iter()
                        .flat_map(|p| p.graveyard.iter())
                        .find(|c| c.id == id)
                        .map(|c| (None, c))
                };
                located.is_some_and(|(region, c)| {
                    let d = card(&c.definition);
                    let kind = match spec.kind {
                        EntityKind::Any => true,
                        EntityKind::Character => !c.face_down && d.kind == "character",
                        EntityKind::Hidden => c.face_down,
                        EntityKind::CharacterOrHidden => c.face_down || d.kind == "character",
                    };
                    let relation = match spec.relation {
                        Relation::Any => true,
                        Relation::ControlledByActor => c.controller == actor,
                        Relation::OwnedByActor => c.owner == actor,
                        Relation::EnemyTeam => self.is_enemy(actor, c),
                    };
                    kind && relation
                        && (spec.range != Range::SourceRegion || region == source.region)
                        && spec
                            .subtype
                            .as_ref()
                            .is_none_or(|s| !c.face_down && d.subtypes.contains(s))
                        && (spec.zone != Zone::Board || self.targetable(actor, c))
                })
            }
        }
    }
    pub(crate) fn binding_options(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        spec: &TargetSlotSpec,
    ) -> Vec<ChoiceOption> {
        let options: Vec<ChoiceOption> = match spec.zone {
            Zone::Board => self
                .regions
                .iter()
                .enumerate()
                .flat_map(|(r, reg)| reg.cards.iter().map(move |c| (r, c)))
                .map(|(r, c)| self.option(c, actor, Some(r), None))
                .collect(),
            Zone::Graveyard => self
                .players
                .iter()
                .flat_map(|p| p.graveyard.iter())
                .map(|c| self.option(c, actor, None, None))
                .collect(),
            Zone::Player => self
                .players
                .iter()
                .map(|p| ChoiceOption {
                    id: player_id(p.seat),
                    label: p.name.clone(),
                    card: None,
                })
                .collect(),
            Zone::Region => self
                .regions
                .iter()
                .enumerate()
                .map(|(r, reg)| ChoiceOption {
                    id: format!("region:{r}"),
                    label: card(&reg.card.definition).name.clone(),
                    card: None,
                })
                .collect(),
        };
        options
            .into_iter()
            .filter(|o| self.valid_binding(actor, source, spec, &o.id))
            .collect()
    }
    pub(crate) fn bind_action(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        spec: &AbilitySpec,
        a: &Action,
    ) -> RuleResult<Vec<BoundTarget>> {
        if spec.targets.len() > 1 || spec.targets.iter().any(|s| s.min != 1 || s.max != 1) {
            return Err("尚未核定的多目标规则不可开放".into());
        }
        spec.targets
            .iter()
            .map(|slot| {
                let id = if slot.zone == Zone::Region {
                    format!("region:{}", a.region.ok_or("请选择地区")?)
                } else {
                    a.target_id.clone().ok_or("请选择目标")?
                };
                if !self.valid_binding(actor, source, slot, &id) {
                    return Err("目标不存在或不满足目标条件".into());
                }
                let public = self.public_target(actor, source, slot, &id);
                Ok(BoundTarget {
                    id,
                    spec: slot.clone(),
                    public,
                })
            })
            .collect()
    }
    pub(crate) fn pay_ability_costs(
        &mut self,
        actor: usize,
        source: &SourceSnapshot,
        spec: &AbilitySpec,
        a: &Action,
    ) -> RuleResult<Vec<PaidCost>> {
        let mut paid = vec![];
        for cost in &spec.costs {
            match cost {
                Cost::Assets(n) => {
                    let ids = self.players[actor]
                        .assets
                        .iter()
                        .filter(|c| !c.exhausted)
                        .take(*n as usize)
                        .map(|c| c.id.clone())
                        .collect();
                    self.pay(actor, *n)?;
                    paid.push(PaidCost::Assets(ids));
                }
                Cost::ExhaustSource => {
                    let c = self
                        .board_mut(&source.card.id)
                        .filter(|c| c.controller == actor && !c.face_down && !c.exhausted)
                        .ok_or("来源不可横置支付")?;
                    c.exhausted = true;
                    paid.push(PaidCost::Exhausted(c.id.clone()));
                }
                Cost::SacrificeSource | Cost::SacrificeSelectedControlledCharacter => {
                    let id = if matches!(cost, Cost::SacrificeSource) {
                        source.card.id.clone()
                    } else {
                        let ids = a.cost_selected.as_ref().ok_or("请选择额外牺牲费用")?;
                        if ids.len() != 1 {
                            return Err("须牺牲一个自己操控的角色".into());
                        }
                        ids[0].clone()
                    };
                    let (_, c) = self
                        .board(&id)
                        .filter(|(_, c)| {
                            c.controller == actor
                                && !c.face_down
                                && card(&c.definition).kind == "character"
                        })
                        .ok_or("牺牲费用须为自己操控的角色")?;
                    paid.push(PaidCost::Sacrificed {
                        old_instance: id.clone(),
                        controller: c.controller,
                        owner: c.owner,
                    });
                    self.remove_dead(&id, RemovalCause::Sacrifice);
                }
            }
        }
        Ok(paid)
    }
    pub(crate) fn make_frame(
        &self,
        actor: usize,
        source: SourceSnapshot,
        spec: &AbilitySpec,
        targets: Vec<BoundTarget>,
        paid: Vec<PaidCost>,
        chosen_region: Option<usize>,
    ) -> ResolutionFrame {
        let mut steps = vec![];
        for op in &spec.ops {
            if let Op::ForEachLivingPlayer(ops) = op {
                for p in self.players.iter().filter(|p| !p.eliminated) {
                    for op in ops {
                        steps.push(Step {
                            context: p.seat,
                            op: op.clone(),
                        });
                    }
                }
            } else {
                steps.push(Step {
                    context: actor,
                    op: op.clone(),
                });
            }
        }
        ResolutionFrame {
            frame_id: String::new(),
            ability_key: spec.key.clone(),
            actor,
            source,
            targets,
            already_paid: paid,
            guard: GuardState::Unchecked,
            cursor: 0,
            steps,
            chosen_region,
        }
    }
    pub(crate) fn push_frame(
        &mut self,
        mut frame: ResolutionFrame,
        label: String,
        transaction: Option<Card>,
    ) {
        let target = frame.targets.first().map(|t| t.id.clone());
        let actor = frame.actor;
        self.push_stack(actor, label, transaction, None, false, target);
        frame.frame_id = self.stack.last().unwrap().id.clone();
        self.stack.last_mut().unwrap().frame = Some(frame);
    }
    pub(crate) fn emit_event(&mut self, actor: usize, source: SourceSnapshot, event: Event) {
        for spec in rules::definition(&source.card.definition)
            .abilities
            .iter()
            .filter(|s| s.event == Some(event))
        {
            self.effects.push_back(Effect::Declare {
                declaration: Declaration {
                    actor,
                    source: source.clone(),
                    ability: spec.clone(),
                },
            });
        }
    }
    pub(crate) fn declare_trigger(&mut self, declaration: Declaration) -> RuleResult<()> {
        let actor = declaration.actor;
        let spec = &declaration.ability;
        if self.players[actor].eliminated {
            return Ok(());
        }
        if spec.requires_ready_source
            && !self
                .board(&declaration.source.card.id)
                .is_some_and(|(_, c)| !c.face_down && !c.exhausted)
        {
            return Ok(());
        }
        let title = format!(
            "{}：{}",
            card(&declaration.source.card.definition).name,
            spec.label
        );
        let (options, stage) = if !spec.modes.is_empty() {
            (
                spec.modes
                    .iter()
                    .map(|m| ChoiceOption {
                        id: m.key.clone(),
                        label: m.label.clone(),
                        card: None,
                    })
                    .collect(),
                DeclareChoice::Mode,
            )
        } else if let Some(slot) = spec.targets.first() {
            (
                self.binding_options(actor, &declaration.source, slot),
                DeclareChoice::Target,
            )
        } else {
            (
                vec![ChoiceOption {
                    id: "accept".into(),
                    label: "发动触发能力".into(),
                    card: None,
                }],
                DeclareChoice::Accept,
            )
        };
        if !options.is_empty() {
            self.choice(
                actor,
                "trigger",
                title,
                options,
                0,
                1,
                None,
                ChoiceResolution::Declare { declaration, stage },
            );
        }
        Ok(())
    }
    pub(crate) fn choose_declaration(
        &mut self,
        mut declaration: Declaration,
        stage: DeclareChoice,
        selected: Vec<String>,
    ) -> RuleResult<()> {
        let Some(selected) = selected.first() else {
            return Ok(());
        };
        let targets = match stage {
            DeclareChoice::Accept => vec![],
            DeclareChoice::Target => {
                let slot = declaration.ability.targets[0].clone();
                if !self.valid_binding(declaration.actor, &declaration.source, &slot, selected) {
                    return Err("触发目标已失效".into());
                }
                let public =
                    self.public_target(declaration.actor, &declaration.source, &slot, selected);
                vec![BoundTarget {
                    id: selected.clone(),
                    spec: slot,
                    public,
                }]
            }
            DeclareChoice::Mode => {
                let mode = declaration
                    .ability
                    .modes
                    .iter()
                    .find(|m| m.key == *selected)
                    .ok_or("模式无效")?
                    .clone();
                declaration.ability.label =
                    format!("{}：{}", declaration.ability.label, mode.label);
                declaration.ability.targets = mode.targets;
                declaration.ability.ops = mode.ops;
                declaration.ability.modes.clear();
                if !declaration.ability.targets.is_empty() {
                    return self.declare_trigger(declaration);
                }
                vec![]
            }
        };
        let label = format!(
            "{}：{}",
            card(&declaration.source.card.definition).name,
            declaration.ability.label
        );
        let frame = self.make_frame(
            declaration.actor,
            declaration.source,
            &declaration.ability,
            targets,
            vec![],
            None,
        );
        self.push_frame(frame, label, None);
        Ok(())
    }
    fn frame_entity(frame: &ResolutionFrame, entity: EntityRef) -> Option<&str> {
        match entity {
            EntityRef::Source => Some(&frame.source.card.id),
            EntityRef::Target(slot) => frame.targets.get(slot).map(|t| t.id.as_str()),
        }
    }
    fn frame_player(
        &self,
        frame: &ResolutionFrame,
        context: usize,
        player: PlayerRef,
    ) -> RuleResult<usize> {
        match player {
            PlayerRef::Actor => Ok(frame.actor),
            PlayerRef::Context => Ok(context),
            PlayerRef::Target(i) => frame
                .targets
                .get(i)
                .and_then(|t| t.id.strip_prefix('p'))
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|s| *s < self.players.len())
                .ok_or("玩家引用无效".into()),
        }
    }
    pub(crate) fn reset_zone_card(&mut self, mut c: Card) -> Card {
        c = self.fresh(c);
        c.controller = c.owner;
        c.face_down = false;
        c.exhausted = false;
        c.damage = 0;
        c.wounds = 0;
        c.shield = 0;
        c
    }
    pub(crate) fn remove_dead(&mut self, target: &str, cause: RemovalCause) {
        if let Some((region, c)) = self.remove_board(target) {
            let snapshot = self.source_snapshot(&c, Some(region));
            let controller = c.controller;
            let character = !c.face_down && card(&c.definition).kind == "character";
            let owner = c.owner;
            let c = self.reset_zone_card(c);
            self.players[owner].graveyard.push(c);
            self.note(format!(
                "{}：{}",
                card(&snapshot.card.definition).name,
                match cause {
                    RemovalCause::Sacrifice => "牺牲",
                    RemovalCause::Destroy => "消灭",
                    RemovalCause::Lethal => "死亡",
                }
            ));
            if character {
                self.emit_event(controller, snapshot, Event::Death);
            }
        }
    }
    fn take_entity(&mut self, id: &str) -> Option<Card> {
        if let Some((_, c)) = self.remove_board(id) {
            return Some(c);
        }
        for p in &mut self.players {
            if let Some(i) = p.graveyard.iter().position(|c| c.id == id) {
                return Some(p.graveyard.remove(i));
            }
        }
        None
    }
    pub(crate) fn resolve_frame(&mut self, mut frame: ResolutionFrame) -> RuleResult<()> {
        if matches!(frame.guard, GuardState::Unchecked) {
            if frame
                .targets
                .iter()
                .any(|t| !self.valid_binding(frame.actor, &frame.source, &t.spec, &t.id))
            {
                frame.guard = GuardState::Cancelled;
                self.note(format!(
                    "{}：原目标{}已离场或不再满足目标条件，整个卡牌或能力效果取消；费用不退",
                    card(&frame.source.card.definition).name,
                    frame
                        .targets
                        .iter()
                        .map(|t| t.public.label.as_str())
                        .collect::<Vec<_>>()
                        .join("、")
                ));
                return Ok(());
            }
            for target in &frame.targets {
                if target.spec.zone == Zone::Board && self.shield_stops(frame.actor, &target.id) {
                    frame.guard = GuardState::Cancelled;
                    return Ok(());
                }
            }
            frame.guard = GuardState::Accepted;
        }
        if matches!(frame.guard, GuardState::Cancelled) {
            return Ok(());
        }
        while frame.cursor < frame.steps.len() && self.status == "playing" {
            let step = frame.steps[frame.cursor].clone();
            frame.cursor += 1;
            match step.op {
                Op::Exhaust(entity) => {
                    if let Some(id) = Self::frame_entity(&frame, entity) {
                        if let Some(c) = self.board_mut(id) {
                            c.exhausted = true;
                        }
                    }
                }
                Op::HealWounds(entity) => {
                    if let Some(id) = Self::frame_entity(&frame, entity) {
                        if let Some(c) = self.board_mut(id) {
                            c.wounds = 0;
                        }
                    }
                }
                Op::Move(entity, destination) => {
                    if let Some(id) = Self::frame_entity(&frame, entity) {
                        if let Some(c) = self.take_entity(id) {
                            let mut c = self.reset_zone_card(c);
                            let owner = c.owner;
                            match destination {
                                Destination::OwnerHand => self.players[owner].hand.push(c),
                                Destination::ActorHand => self.players[frame.actor].hand.push(c),
                                Destination::OwnerDeckBottom => self.players[owner].deck.push(c),
                                Destination::HiddenInChosenRegion => {
                                    c.face_down = true;
                                    let r = frame.chosen_region.ok_or("缺少派遣地区")?;
                                    self.regions[r].cards.push(c);
                                }
                            }
                        }
                    }
                }
                Op::Hide(entity) => {
                    if let Some(id) = Self::frame_entity(&frame, entity) {
                        if let Some((r, c)) = self.remove_board(id) {
                            let mut c = self.fresh(c);
                            c.face_down = true;
                            c.damage = 0;
                            c.wounds = 0;
                            c.shield = 0;
                            self.regions[r].cards.push(c);
                        }
                    }
                }
                Op::MoveOnBoard { entity, region } => {
                    if let (Some(id), Some(region)) = (
                        Self::frame_entity(&frame, entity),
                        self.frame_region(&frame, region),
                    ) {
                        if let Some((_, c)) = self.remove_board(id) {
                            let actor = c.controller;
                            let definition = c.definition.clone();
                            let id = c.id.clone();
                            self.regions[region].cards.push(c);
                            self.enter_triggers(actor, &definition, &id, false);
                        }
                    }
                }
                Op::Destroy(entity) => {
                    if let Some(id) = Self::frame_entity(&frame, entity) {
                        self.remove_dead(id, RemovalCause::Destroy);
                    }
                }
                Op::Draw { player, count, end } => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    if end == DeckEnd::Top {
                        self.draw(seat, count)?;
                    } else {
                        for _ in 0..count {
                            if self.players[seat].eliminated {
                                break;
                            }
                            if let Some(c) = self.players[seat].deck.pop() {
                                let c = self.fresh(c);
                                self.players[seat].hand.push(c);
                            } else {
                                self.eliminate(seat);
                                break;
                            }
                        }
                    }
                }
                Op::MoveBottomToHand { player, count } => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    for _ in 0..count {
                        if let Some(c) = self.players[seat].deck.pop() {
                            let c = self.fresh(c);
                            self.players[seat].hand.push(c);
                        } else {
                            break;
                        }
                    }
                }
                Op::Forecast { player, count } => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    let options = self.players[seat]
                        .deck
                        .iter()
                        .take(count)
                        .map(|c| self.option(c, seat, None, None))
                        .collect::<Vec<_>>();
                    if !options.is_empty() {
                        let n = options.len();
                        self.choice(
                            seat,
                            "investigation",
                            format!("预测 {n}"),
                            options,
                            n,
                            n,
                            Some(n as u32),
                            ChoiceResolution::Frame {
                                frame: Box::new(frame),
                                choice: FrameChoice::Forecast { seat },
                            },
                        );
                        return Ok(());
                    }
                }
                Op::Discard {
                    player,
                    count,
                    redraw,
                    optional,
                } => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    if !self.players[seat].eliminated && !self.players[seat].hand.is_empty() {
                        let n = count
                            .unwrap_or(self.players[seat].hand.len())
                            .min(self.players[seat].hand.len());
                        let options = self.players[seat]
                            .hand
                            .iter()
                            .map(|c| self.option(c, seat, None, None))
                            .collect();
                        self.choice(
                            seat,
                            "discard",
                            if redraw {
                                "弃任意手牌并抓等量".into()
                            } else {
                                format!("弃 {n} 张手牌")
                            },
                            options,
                            if optional { 0 } else { n },
                            n,
                            None,
                            ChoiceResolution::Frame {
                                frame: Box::new(frame),
                                choice: FrameChoice::Discard { seat, redraw },
                            },
                        );
                        return Ok(());
                    }
                }
                Op::Search {
                    player,
                    filter,
                    to_top,
                    optional,
                } => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    if !self.players[seat].eliminated {
                        let options = self.players[seat]
                            .deck
                            .iter()
                            .filter(|c| self.filter_card(&filter, c))
                            .map(|c| self.option(c, seat, None, None))
                            .collect::<Vec<_>>();
                        if options.is_empty() {
                            self.shuffle_player(seat);
                            self.note(format!(
                                "{} 检索：没有符合条件的牌",
                                self.players[seat].name
                            ));
                        } else {
                            self.choice(
                                seat,
                                "search",
                                "检索并洗牌".into(),
                                options,
                                if optional { 0 } else { 1 },
                                1,
                                None,
                                ChoiceResolution::Frame {
                                    frame: Box::new(frame),
                                    choice: FrameChoice::Search { seat, to_top },
                                },
                            );
                            return Ok(());
                        }
                    }
                }
                Op::SacrificeChosen(player) => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    let options = self
                        .regions
                        .iter()
                        .enumerate()
                        .flat_map(|(r, reg)| reg.cards.iter().map(move |c| (r, c)))
                        .filter(|(_, c)| {
                            c.controller == seat
                                && !c.face_down
                                && card(&c.definition).kind == "character"
                        })
                        .map(|(r, c)| self.option(c, seat, Some(r), None))
                        .collect::<Vec<_>>();
                    if !options.is_empty() {
                        self.choice(
                            seat,
                            "target",
                            "选择一个自己操控的角色牺牲".into(),
                            options,
                            1,
                            1,
                            None,
                            ChoiceResolution::Frame {
                                frame: Box::new(frame),
                                choice: FrameChoice::Sacrifice { seat },
                            },
                        );
                        return Ok(());
                    }
                }
                Op::ExhaustMatching(selector) => {
                    let ids = self.matching_board(&frame, &selector);
                    for id in ids {
                        if let Some(c) = self.board_mut(&id) {
                            c.exhausted = true;
                        }
                    }
                }
                Op::DamageMatching { selector, amount } => {
                    let targets = self
                        .matching_board(&frame, &selector)
                        .into_iter()
                        .map(|id| (id, amount))
                        .collect();
                    self.damage(targets)?;
                }
                Op::SkipRegionThisRound(region) => {
                    if let Some(r) = self.frame_region(&frame, region) {
                        self.regions[r].skip = true;
                    }
                }
                Op::CostReduction { filter, amount } => self.modifiers.push(CostModifier {
                    actor: frame.actor,
                    filter,
                    amount,
                    expires_turn: self.turn,
                    uses: 1,
                }),
                Op::ForEachLivingPlayer(_) => return Err("禁止嵌套的玩家遍历声明".into()),
            }
            self.settle_deaths();
        }
        Ok(())
    }
    pub(crate) fn choose_frame(
        &mut self,
        frame: Box<ResolutionFrame>,
        choice: FrameChoice,
        a: Action,
        selected: Vec<String>,
        option_ids: BTreeSet<String>,
    ) -> RuleResult<()> {
        match choice {
            FrameChoice::Forecast { seat } => {
                let top = a.top.unwrap_or_default();
                let bottom = a.bottom.unwrap_or_default();
                let all = top.iter().chain(&bottom).cloned().collect::<Vec<_>>();
                if all.iter().cloned().collect::<BTreeSet<_>>() != option_ids
                    || all.len() != option_ids.len()
                {
                    return Err("必须完整排列预测牌".into());
                }
                let mut inspected = std::mem::take(&mut self.players[seat].deck);
                let remainder = inspected.split_off(all.len());
                let mut map = inspected
                    .into_iter()
                    .map(|c| (c.id.clone(), c))
                    .collect::<BTreeMap<_, _>>();
                let mut pile = vec![];
                for id in top {
                    pile.push(map.remove(&id).ok_or("预测牌失效")?);
                }
                pile.extend(remainder);
                for id in bottom {
                    pile.push(map.remove(&id).ok_or("预测牌失效")?);
                }
                self.players[seat].deck = pile;
            }
            FrameChoice::Discard { seat, redraw } => {
                let count = selected.len();
                for id in selected {
                    let c = self.remove_hand(seat, &id)?;
                    let c = self.reset_zone_card(c);
                    self.players[seat].graveyard.push(c);
                }
                if redraw {
                    self.draw(seat, count)?;
                }
            }
            FrameChoice::Search { seat, to_top } => {
                let c = selected
                    .first()
                    .and_then(|id| self.players[seat].deck.iter().position(|c| c.id == *id))
                    .map(|i| self.players[seat].deck.remove(i));
                self.shuffle_player(seat);
                if let Some(c) = c {
                    let c = self.fresh(c);
                    if to_top {
                        self.players[seat].deck.insert(0, c);
                    } else {
                        self.note(format!(
                            "{} 展示检索的 {}",
                            self.players[seat].name,
                            card(&c.definition).name
                        ));
                        self.players[seat].hand.push(c);
                    }
                }
            }
            FrameChoice::Sacrifice { seat } => {
                let id = selected.first().ok_or("必须选择角色牺牲")?;
                if !self.board(id).is_some_and(|(_, c)| {
                    c.controller == seat && !c.face_down && card(&c.definition).kind == "character"
                }) {
                    return Err("牺牲角色已不满足操控条件".into());
                }
                self.remove_dead(id, RemovalCause::Sacrifice);
            }
        }
        self.effects.push_front(Effect::Frame { frame });
        Ok(())
    }
}
