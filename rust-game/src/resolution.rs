//! Shared declaration, payment, target guard and resumable finite operation interpreter.
use crate::{
    catalog::card,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::{BTreeMap, BTreeSet};

impl Game {
    fn attachment_host_matches(
        &self,
        actor: usize,
        id: &str,
        condition: &AttachmentHostCondition,
    ) -> bool {
        let Some(attachment) = self.attachments.iter().find(|a| a.card.id == id) else {
            return false;
        };
        if self.attachment_region(attachment).is_none() {
            return false;
        }
        match condition {
            AttachmentHostCondition::CharacterOrActorAssetDomain { magic } => {
                self.board(&attachment.host_id).is_some_and(|(_, host)| {
                    !host.face_down && card(&host.definition).kind == "character"
                }) || self.actor_has_asset_domain(actor, magic, 1)
            }
        }
    }
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
        self.in_play_cards()
            .into_iter()
            .filter(|(r, c)| {
                let d = card(&c.definition);
                let kind = match selector.kind {
                    EntityKind::Any => true,
                    EntityKind::Character => !c.face_down && d.kind == "character",
                    EntityKind::Attachment => !c.face_down && d.kind == "attachment",
                    EntityKind::Hidden => c.face_down,
                    EntityKind::CharacterOrHidden => c.face_down || d.kind == "character",
                };
                let relation = match selector.relation {
                    Relation::Any => true,
                    Relation::ControlledByActor => c.controller == frame.actor,
                    Relation::OwnedByActor => c.owner == frame.actor,
                    Relation::FriendlyTeam => !self.is_enemy(frame.actor, c),
                    Relation::EnemyTeam => self.is_enemy(frame.actor, c),
                };
                kind && relation
                    && selector
                        .region
                        .is_none_or(|wanted| self.frame_region(frame, wanted) == Some(*r))
                    && selector
                        .subtype
                        .as_ref()
                        .is_none_or(|s| !c.face_down && self.current_subtypes(c).contains(s))
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
            .filter(|a| {
                !(kind == "play" && a.activation_only) && !(kind == "activate" && a.play_only)
            })
        {
            let modes = if ability.modes.is_empty() {
                vec![None]
            } else {
                ability.modes.iter().map(Some).collect()
            };
            for mode in modes {
                let slots = mode.map_or(&ability.targets, |m| &m.targets);
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
                } else if ability
                    .costs
                    .iter()
                    .any(|c| matches!(c, Cost::DiscardSelectedHandCard))
                {
                    self.players[actor]
                        .hand
                        .iter()
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
                                        " → {}{}{} [{}]",
                                        t.label,
                                        t.card
                                            .as_ref()
                                            .and_then(|c| c.region)
                                            .map(|r| format!("·地区{}", r + 1))
                                            .unwrap_or_default(),
                                        t.card
                                            .as_ref()
                                            .map(|c| format!(
                                                "·拥有者{}",
                                                self.players[c
                                                    .owner
                                                    .strip_prefix('p')
                                                    .and_then(|s| s.parse::<usize>().ok())
                                                    .unwrap()]
                                                .name
                                            ))
                                            .unwrap_or_default(),
                                        t.id
                                    ))
                                    .unwrap_or_default(),
                                cost.as_ref()
                                    .and_then(|ids| ids.first())
                                    .and_then(|id| self.players[actor]
                                        .hand
                                        .iter()
                                        .find(|c| c.id == *id)
                                        .filter(|_| ability
                                            .costs
                                            .iter()
                                            .any(|c| matches!(c, Cost::DiscardSelectedHandCard)))
                                        .map(|c| format!(
                                            "（费用：弃掉{} [{}]）",
                                            card(&c.definition).name,
                                            c.id
                                        ))
                                        .or_else(|| self.board(id).map(|(r, c)| format!(
                                            "（费用：牺牲{}·地区{} [{}]）",
                                            card(&c.definition).name,
                                            r + 1,
                                            c.id
                                        ))))
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
        let asset = (spec.zone == Zone::AttachmentOrAsset)
            .then(|| self.asset_location(id))
            .flatten();
        let located = asset
            .map(|(s, i)| (None, &self.players[s].assets[i]))
            .or_else(|| self.board(id).map(|(r, c)| (Some(r), c)))
            .or_else(|| {
                self.players
                    .iter()
                    .flat_map(|p| p.graveyard.iter())
                    .find(|c| c.id == id)
                    .map(|c| (None, c))
            });
        let (label, kind, owner, controller, region) = if let Some((region, c)) = located {
            let label = if asset.is_some() {
                "资产".into()
            } else if c.face_down {
                format!("暗藏者·{}", self.players[c.controller].name)
            } else {
                card(&c.definition).name.clone()
            };
            (
                label,
                if asset.is_some() {
                    "asset"
                } else if c.face_down {
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
        if (target.spec.zone == Zone::AttachmentOrAsset
            && self.asset_location(&target.id).is_some())
            || self.board(&target.id).is_some()
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
        summary.valid = self.valid_bound_target(frame.actor, &frame.source, target);
        if !summary.valid {
            let present = match target.spec.zone {
                Zone::AttachmentOrAsset => {
                    self.asset_location(&target.id).is_some()
                        || self.attachments.iter().any(|a| a.card.id == target.id)
                }
                Zone::Board => self.board(&target.id).is_some(),
                Zone::Graveyard => self
                    .players
                    .iter()
                    .any(|p| p.graveyard.iter().any(|c| c.id == target.id)),
                Zone::Region => target
                    .region_instance
                    .as_ref()
                    .is_none_or(|id| self.regions.iter().any(|r| r.card.id == *id)),
                Zone::Player => true,
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
            source_region_instance: if matches!(c.definition.as_str(), "JC032" | "JZ24" | "JZ31" | "BQ104" | "XQ48") {
                region.and_then(|r| self.regions.get(r)).map(|r| r.card.id.clone())
            } else { None },
            attachment_host_instance: rules::definition(&c.definition)
                .abilities
                .iter()
                .any(|a| {
                    a.ops
                        .iter()
                        .any(|op| matches!(op, Op::ModifyAttachmentHostUntilTurnEnd))
                })
                .then(|| {
                    self.attachments
                        .iter()
                        .find(|a| a.card.id == c.id)
                        .map(|a| a.host_id.clone())
                })
                .flatten(),
            play_source: None,
        }
    }
    pub(crate) fn filter_card(&self, filter: &CardFilter, c: &Card) -> bool {
        filter.matches(card(&c.definition))
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
    pub(crate) fn paid_reveal_cost(&self, actor: usize, c: &Card) -> u32 {
        card(&c.definition).cost.saturating_sub(
            self.modifiers
                .iter()
                .filter(|m| {
                    m.paid_reveal
                        && m.actor == actor
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
            self.paid_reveal_cost(actor, c)
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
                    && (face_up_play || m.paid_reveal)
            })
            .map(|(i, _)| i)
            .collect::<Vec<_>>();
        for i in used {
            self.modifiers[i].uses -= 1;
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
            .filter(|s| s.event.is_none())
            .filter(|s| {
                !(a.kind == "play" && s.activation_only) && !(a.kind == "activate" && s.play_only)
            });
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
        crate::society::ensure_activation_condition(definition, &spec.key)?;
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
            Zone::AttachmentOrAsset => {
                self.asset_location(id)
                    .is_some_and(|(s, i)| !self.players[s].assets[i].face_down)
                    || self
                        .attachments
                        .iter()
                        .find(|a| a.card.id == id)
                        .is_some_and(|a| {
                            !a.card.face_down
                                && self.board(id).is_some()
                                && self.targetable(actor, &a.card)
                        })
            }
            Zone::Player => id
                .strip_prefix('p')
                .and_then(|s| s.parse::<usize>().ok())
                .is_some_and(|s| {
                    s < self.players.len()
                        && !self.players[s].eliminated
                        && match spec.relation {
                            Relation::Any => true,
                            Relation::EnemyTeam => self.team(s) != self.team(actor),
                            Relation::FriendlyTeam => self.team(s) == self.team(actor),
                            Relation::ControlledByActor | Relation::OwnedByActor => s == actor,
                        }
                }),
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
                        EntityKind::Attachment => !c.face_down && d.kind == "attachment",
                        EntityKind::Hidden => c.face_down,
                        EntityKind::CharacterOrHidden => c.face_down || d.kind == "character",
                    };
                    let relation = match spec.relation {
                        Relation::Any => true,
                        Relation::ControlledByActor => c.controller == actor,
                        Relation::OwnedByActor => c.owner == actor,
                        Relation::FriendlyTeam => !self.is_enemy(actor, c),
                        Relation::EnemyTeam => self.is_enemy(actor, c),
                    };
                    kind && relation
                        && spec
                            .attachment_host_condition
                            .as_ref()
                            .is_none_or(|condition| {
                                self.attachment_host_matches(actor, id, condition)
                            })
                        && (!spec.requires_magic
                            || (!c.face_down && d.magic_icon != MagicIcon::None))
                        && (!spec.exclude_source || c.id != source.card.id)
                        && (!spec.exclude_attachment_host
                            || self
                                .attachments
                                .iter()
                                .find(|a| a.card.id == source.card.id)
                                .is_some_and(|a| a.host_id != c.id))
                        && (!spec.equipment_host
                            || !rules::definition(&c.definition).traits.cannot_be_equipped)
                        && spec
                            .printed_cost_max
                            .is_none_or(|max| !c.face_down && d.cost <= max)
                        && spec
                            .predicate
                            .as_ref()
                            .is_none_or(|predicate| match predicate {
                                rules::TargetPredicate::JC015NonHumanPrintedCostAtLeastThree => {
                                    source.card.definition == "JC015"
                                        && !c.face_down
                                        && d.cost >= 3
                                        && !self.current_subtypes(c).iter().any(|s| s == "人类")
                                }
                                rules::TargetPredicate::JZ55UniqueCharacter => {
                                    source.card.definition == "JZ55"
                                        && !c.face_down && d.kind == "character" && d.unique
                                }
                            })
                        && (spec.range != Range::SourceRegion || region == source.region)
                        && spec.subtype.as_ref().is_none_or(|s| {
                            !c.face_down && self.target_subtypes(c, spec).contains(s)
                        })
                        && (spec.subtypes_any.is_empty()
                            || (!c.face_down
                                && spec
                                    .subtypes_any
                                    .iter()
                                    .any(|s| self.target_subtypes(c, spec).contains(s))))
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
            Zone::AttachmentOrAsset => self
                .attachments
                .iter()
                .filter_map(|a| {
                    self.board(&a.card.id)
                        .map(|(r, c)| self.option(c, actor, Some(r), None))
                })
                .chain(
                    self.players
                        .iter()
                        .flat_map(|p| p.assets.iter())
                        .map(|c| self.option(c, actor, None, Some("asset"))),
                )
                .collect(),
            Zone::Board => self
                .in_play_cards()
                .into_iter()
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
    pub(crate) fn valid_bound_target(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        target: &BoundTarget,
    ) -> bool {
        let exact_region = Self::requires_region_instance(source, &target.spec);
        self.valid_binding(actor, source, &target.spec, &target.id)
            && (!exact_region
                || target.region_instance.as_ref().is_some_and(|instance| {
                    target
                        .id
                        .strip_prefix("region:")
                        .and_then(|s| s.parse::<usize>().ok())
                        .and_then(|r| self.regions.get(r))
                        .is_some_and(|r| r.card.id == *instance)
                }))
    }
    fn requires_region_instance(source: &SourceSnapshot, slot: &TargetSlotSpec) -> bool {
        slot.zone == Zone::Region
            && (source.card.definition == "MSJC11"
                || rules::definition(&source.card.definition)
                    .attachment
                    .as_ref()
                    .is_some_and(|a| a.host.zone == Zone::Region))
    }
    pub(crate) fn bind_action(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        spec: &AbilitySpec,
        a: &Action,
    ) -> RuleResult<Vec<BoundTarget>> {
        rules::validate_ability(&source.card.definition, spec)?;
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
                    region_instance: Self::requires_region_instance(source, slot)
                        .then(|| self.regions[a.region.unwrap()].card.id.clone()),
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
        if spec.per_turn_limit.is_some_and(|limit| {
            self.turn_ability_usage.iter().any(|u| {
                u.source_instance == source.card.id
                    && u.ability_key == spec.key
                    && u.turn == self.turn
                    && u.uses >= limit
            })
        }) {
            return Err("此能力本回合已达发动上限".into());
        }
        if spec.once_per_game {
            let used = self
                .society_usage(&source.card.id)
                .ok_or("每局限次能力须来自稳定秘社来源")?;
            if used.contains(&spec.key) {
                return Err("此能力本局已发动过".into());
            }
        }
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
                        .ability_source_mut(&source.card.id)
                        .filter(|c| c.controller == actor && !c.face_down && !c.exhausted)
                        .ok_or("来源不可横置支付")?;
                    c.exhausted = true;
                    paid.push(PaidCost::Exhausted(c.id.clone()));
                }
                Cost::DiscardSelectedHandCard => {
                    let ids = a.cost_selected.as_ref().ok_or("请选择弃手牌费用")?;
                    if ids.len() != 1 {
                        return Err("须弃一张自己的手牌".into());
                    }
                    paid.push(self.discard_hand_card(actor, &ids[0])?);
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
                                && (card(&c.definition).kind == "character"
                                    || (matches!(cost, Cost::SacrificeSource)
                                        && card(&c.definition).kind == "attachment"
                                        && self.attachments.iter().any(|a| a.card.id == c.id)))
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
        if spec.once_per_game {
            self.consume_society_usage(&source.card.id, &spec.key)?;
        }
        if spec.per_turn_limit.is_some() {
            if let Some(usage) = self.turn_ability_usage.iter_mut().find(|u| {
                u.source_instance == source.card.id
                    && u.ability_key == spec.key
                    && u.turn == self.turn
            }) {
                usage.uses += 1;
            } else {
                self.turn_ability_usage.push(TurnAbilityUsage {
                    source_instance: source.card.id.clone(),
                    ability_key: spec.key.clone(),
                    turn: self.turn,
                    uses: 1,
                });
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
            if let Op::ForEachLivingPlayer(ops) | Op::ForEachLivingPlayerFromActor(ops) = op {
                let start = if matches!(op, Op::ForEachLivingPlayerFromActor(_)) {
                    actor
                } else {
                    0
                };
                for seat in
                    (0..self.players.len()).map(|offset| (start + offset) % self.players.len())
                {
                    if self.players[seat].eliminated {
                        continue;
                    }
                    for op in ops {
                        steps.push(Step {
                            context: seat,
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
    pub(crate) fn dispatch_frame(
        &mut self,
        mut frame: ResolutionFrame,
        label: String,
        transaction: Option<Card>,
        policy: ResponsePolicy,
    ) {
        match policy {
            ResponsePolicy::Respondable => self.push_frame(frame, label, transaction),
            ResponsePolicy::Immediate => {
                // The same guarded, paid, resumable frame runs without a response
                // window. Independent events emitted by paying its costs remain
                // queued and are declared after this immediate effect completes.
                frame.frame_id = self.id();
                self.reset_passes();
                self.note(format!("立即结算（不可响应）：{label}"));
                if let Some(card) = transaction {
                    self.effects.push_front(Effect::Bury { card });
                }
                self.effects.push_front(Effect::Frame {
                    frame: Box::new(frame),
                });
            }
        }
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
        rules::validate_ability(&declaration.source.card.definition, &declaration.ability)?;
        let actor = declaration.actor;
        let spec = &declaration.ability;
        if self.players[actor].eliminated {
            return Ok(());
        }
        if spec.event == Some(Event::HandDiscard) && self.resources(actor) < 1 {
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
        rules::validate_ability(&declaration.source.card.definition, &declaration.ability)?;
        let Some(selected) = selected.first() else {
            return Ok(());
        };
        let targets = match stage {
            DeclareChoice::Accept => vec![],
            DeclareChoice::Target => {
                let slot = declaration
                    .ability
                    .targets
                    .first()
                    .cloned()
                    .ok_or_else(|| {
                        format!(
                            "cardId={} abilityKey={}: target choice requires a target slot",
                            declaration.source.card.definition, declaration.ability.key
                        )
                    })?;
                if !self.valid_binding(declaration.actor, &declaration.source, &slot, selected) {
                    return Err("触发目标已失效".into());
                }
                let public =
                    self.public_target(declaration.actor, &declaration.source, &slot, selected);
                vec![BoundTarget {
                    region_instance: None,
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
        let paid = if declaration.ability.event == Some(Event::HandDiscard) {
            self.pay_ability_costs(
                declaration.actor,
                &declaration.source,
                &declaration.ability,
                &Action::new("trigger"),
            )?
        } else {
            vec![]
        };
        let frame = self.make_frame(
            declaration.actor,
            declaration.source,
            &declaration.ability,
            targets,
            paid,
            None,
        );
        self.dispatch_frame(frame, label, None, declaration.ability.response_policy);
        Ok(())
    }
    fn frame_entity(frame: &ResolutionFrame, entity: EntityRef) -> Option<&str> {
        match entity {
            EntityRef::Source => Some(&frame.source.card.id),
            EntityRef::Target(slot) => frame.targets.get(slot).map(|t| t.id.as_str()),
        }
    }
    // A discard is neither a play nor a death. Ownership of the graveyard is
    // independent of the player whose hand was used, including paid discards.
    pub(crate) fn discard_hand_card(&mut self, holder: usize, id: &str) -> RuleResult<PaidCost> {
        let c = self.remove_hand(holder, id)?;
        let owner = c.owner;
        let old_instance = c.id.clone();
        let mut source = self.source_snapshot(&c, None);
        source.card.controller = holder;
        let c = self.reset_zone_card(c);
        self.players[owner].graveyard.push(c);
        self.emit_event(holder, source, Event::HandDiscard);
        Ok(PaidCost::Discarded {
            old_instance,
            holder,
            owner,
        })
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
        self.remove_dead_with_snapshot(target, cause, None);
    }
    fn asset_location(&self, id: &str) -> Option<(usize, usize)> {
        self.players
            .iter()
            .enumerate()
            .find_map(|(s, p)| p.assets.iter().position(|c| c.id == id).map(|i| (s, i)))
    }
    pub(crate) fn remove_dead_with_snapshot(
        &mut self,
        target: &str,
        cause: RemovalCause,
        simultaneous_source: Option<SourceSnapshot>,
    ) {
        // Host departure can remove an attached control grant. Freeze before it;
        // an already captured simultaneous lethal-set snapshot remains authoritative.
        let snapshot = simultaneous_source.or_else(|| self.board(target)
            .map(|(region, c)| self.source_snapshot(c, Some(region))));
        if let Some((_, c)) = self.leave_board(target) {
            let snapshot = snapshot.expect("board departure has a pre-departure snapshot");
            let controller = snapshot.card.controller;
            let character =
                !snapshot.card.face_down && card(&snapshot.card.definition).kind == "character";
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
        if let Some((_, c)) = self.leave_board(id) {
            return Some(c);
        }
        for p in &mut self.players {
            if let Some(i) = p.graveyard.iter().position(|c| c.id == id) {
                return Some(p.graveyard.remove(i));
            }
        }
        None
    }
    pub(crate) fn accept_frame_guard(&mut self, frame: &mut ResolutionFrame) -> bool {
        if matches!(frame.guard, GuardState::Unchecked) {
            if frame
                .targets
                .iter()
                .any(|t| !self.valid_bound_target(frame.actor, &frame.source, t))
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
                return false;
            }
            for target in &frame.targets {
                if matches!(target.spec.zone, Zone::Board | Zone::AttachmentOrAsset)
                    && self.shield_stops(frame.actor, &target.id)
                {
                    frame.guard = GuardState::Cancelled;
                    return false;
                }
            }
            frame.guard = GuardState::Accepted;
        }
        if matches!(frame.guard, GuardState::Cancelled) {
            return false;
        }
        true
    }
    pub(crate) fn resolve_frame(&mut self, mut frame: ResolutionFrame) -> RuleResult<()> {
        if !self.accept_frame_guard(&mut frame) {
            return Ok(());
        }
        while frame.cursor < frame.steps.len() && self.status == "playing" {
            let step = frame.steps[frame.cursor].clone();
            frame.cursor += 1;
            match step.op {
                Op::JZ50SearchDeathToGraveyard => return self.jz50_search_start(frame),
                Op::BQ104SearchEmployeeHiddenInSourceRegion => return self.entry_search_start(frame, true),
                Op::XQ48SearchPassersIntoSourceRegion => return self.entry_search_start(frame, false),
                Op::SealOneActorHandCardOnTarget => return self.choose_hand_seal(frame),
                Op::DestroyTargetIfSealed => {
                    let host = &frame.targets.first().ok_or("缺少封印载体目标")?.id;
                    if self.sealed_cards.iter().any(|s| s.host_id == *host) {
                        self.remove_dead(host, RemovalCause::Destroy);
                    }
                }
                Op::JC032TopSixVampireHidden => return self.jc032_start(frame),
                Op::JZ24LocalSacrificeSnapshot => return self.jz24_start(frame),
                Op::GainControl {
                    slot,
                    until_source_leaves,
                    subtype_change,
                } => {
                    let target = frame.targets.get(slot).ok_or("缺少控制目标")?.id.clone();
                    let lifetime = if until_source_leaves {
                        ControlLifetime::SourceLeaves {
                            source_instance: frame.source.card.id.clone(),
                        }
                    } else {
                        ControlLifetime::TurnEnd { turn: self.turn }
                    };
                    self.add_control(&target, frame.actor, lifetime, subtype_change);
                }
                Op::ModifyTargetUntilTurnEnd {
                    slot,
                    defense_bonus,
                    ordinary_icons,
                    grants_renown,
                } => {
                    let target = frame.targets.get(slot).ok_or("缺少属性修正目标")?;
                    self.turn_attribute_modifiers.push(TurnAttributeModifier {
                        kill_bonus: 0,
                        grants_retreat: false,
                        target_instance: target.id.clone(),
                        defense_bonus,
                        printed_defense_override: None,
                        ordinary_icons,
                        grants_renown,
                        prevents_damage: false,
                        expires_turn: self.turn,
                    });
                }
                Op::GrantTargetKillUntilTurnEnd { slot } => {
                    self.grant_execution_traits(
                        frame.targets.get(slot).ok_or("缺少杀伤目标")?.id.clone(),
                        1,
                        false,
                    );
                }
                Op::GrantRegionRetreatUntilTurnEnd { slot } => {
                    let target = frame.targets.get(slot).ok_or("缺少撤回地区目标")?;
                    let region = target
                        .id
                        .strip_prefix("region:")
                        .and_then(|s| s.parse::<usize>().ok())
                        .ok_or("撤回地区无效")?;
                    let recipients: Vec<_> = self.regions[region]
                        .cards
                        .iter()
                        .filter(|c| {
                            c.controller == frame.actor
                                && self.current_permanent_combat(c, region) > 0
                        })
                        .map(|c| c.id.clone())
                        .collect();
                    for id in recipients {
                        self.grant_execution_traits(id, 0, true);
                    }
                }
                Op::RepressOpponentOne { slot } => {
                    let seat = self.frame_player(&frame, step.context, PlayerRef::Target(slot))?;
                    let team = self.team(seat);
                    let regions: Vec<_> = self
                        .regions
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| r.influence[team] > 0)
                        .map(|(r, region)| (r, region.card.id.clone()))
                        .collect();
                    if !regions.is_empty() {
                        let options = regions
                            .iter()
                            .map(|(r, _)| ChoiceOption {
                                id: format!("region:{r}"),
                                label: format!(
                                    "地区{}：{}（本方势力{}）",
                                    r + 1,
                                    card(&self.regions[*r].card.definition).name,
                                    self.regions[*r].influence[team]
                                ),
                                card: None,
                            })
                            .collect();
                        self.choice(
                            seat,
                            "target",
                            "遏制1：选择移除本方势力的地区".into(),
                            options,
                            1,
                            1,
                            None,
                            ChoiceResolution::Frame {
                                frame: Box::new(frame),
                                choice: FrameChoice::RepressOne { seat, regions },
                            },
                        );
                        return Ok(());
                    }
                }
                Op::MoveActorDeckTopToAsset => {
                    let seat = frame.actor;
                    if !self.players[seat].deck.is_empty() {
                        let c = self.players[seat].deck.remove(0);
                        let c = self.reset_zone_card(c);
                        self.note(format!(
                            "{} 将牌库顶的 {} 正面置入资产区",
                            self.players[seat].name,
                            card(&c.definition).name
                        ));
                        self.players[seat].assets.push(c);
                    }
                }
                Op::WoundTarget { slot, amount } => {
                    let id = frame.targets.get(slot).ok_or("缺少创伤目标")?.id.clone();
                    if amount > 0 {
                        if let Some(c) = self.board_mut(&id) {
                            c.wounds += amount;
                            let (region, c) = self.board(&id).unwrap();
                            // Capture the recipient's controller and source at the
                            // actual event, before lethal departure resets its instance.
                            let controller = c.controller;
                            let source = self.source_snapshot(c, Some(region));
                            self.emit_event(controller, source, Event::ReceiveWound);
                        }
                    }
                }
                Op::DamageTarget { slot, amount } => {
                    let target = frame.targets.get(slot).ok_or("缺少伤害目标")?;
                    self.damage(std::collections::BTreeMap::from([(
                        target.id.clone(),
                        amount,
                    )]))?;
                }
                Op::PreventTargetDamageUntilTurnEnd { slot } => {
                    let target = frame.targets.get(slot).ok_or("缺少伤害防护目标")?;
                    self.turn_attribute_modifiers.push(TurnAttributeModifier {
                        target_instance: target.id.clone(),
                        defense_bonus: 0,
                        kill_bonus: 0,
                        grants_retreat: false,
                        printed_defense_override: None,
                        ordinary_icons: Icons::default(),
                        grants_renown: false,
                        prevents_damage: true,
                        expires_turn: self.turn,
                    });
                }
                Op::ReattachSource { slot } => {
                    let target = frame.targets.get(slot).ok_or("缺少转移结附目标")?;
                    self.reattach_source(&frame.source.card.id, &target.id);
                }
                Op::ModifyAttachmentHostUntilTurnEnd => {
                    if let Some(id) = frame.source.attachment_host_instance.as_ref().filter(|id| {
                        self.board(id).is_some_and(|(_, c)| {
                            !c.face_down && card(&c.definition).kind == "character"
                        })
                    }) {
                        self.turn_attribute_modifiers.push(TurnAttributeModifier {
                            target_instance: id.clone(),
                            defense_bonus: 0,
                            kill_bonus: 0,
                            grants_retreat: false,
                            printed_defense_override: None,
                            ordinary_icons: Icons {
                                combat: 1,
                                influence: 1,
                                ..Default::default()
                            },
                            grants_renown: false,
                            prevents_damage: false,
                            expires_turn: self.turn,
                        });
                    }
                }
                Op::SetLocalMagicPrintedDefenseToOneUntilTurnEnd => {
                    let ids = self
                        .matching_board(
                            &frame,
                            &BoardSelector {
                                kind: EntityKind::Character,
                                relation: Relation::Any,
                                region: Some(RegionRef::SourceRegion),
                                subtype: None,
                            },
                        )
                        .into_iter()
                        .filter(|id| {
                            self.board(id).is_some_and(|(_, c)| {
                                card(&c.definition).magic_icon != MagicIcon::None
                            })
                        })
                        .collect::<Vec<_>>();
                    for id in ids {
                        self.turn_attribute_modifiers.push(TurnAttributeModifier {
                            target_instance: id,
                            defense_bonus: 0,
                            kill_bonus: 0,
                            grants_retreat: false,
                            printed_defense_override: Some(1),
                            ordinary_icons: Icons::default(),
                            grants_renown: false,
                            prevents_damage: false,
                            expires_turn: self.turn,
                        });
                    }
                }
                Op::PlaceOneInfluenceInSourceRegion => {
                    if let Some(region) = frame.source.region.filter(|&region| {
                        self.regions.get(region).is_some_and(|r|
                            frame.source.source_region_instance.as_ref() == Some(&r.card.id))
                    }) {
                        self.place_influence(frame.actor, region, 1);
                    }
                }
                Op::PlaceInfluence {
                    region_instance,
                    amount,
                } => {
                    if let Some(region) = frame.source.region.filter(|&region| {
                        self.regions
                            .get(region)
                            .is_some_and(|r| r.card.id == region_instance)
                    }) {
                        self.place_influence(frame.actor, region, amount);
                    }
                }
                Op::IfTargetExhausted {
                    slot,
                    exhausted,
                    ready,
                } => {
                    // Do not capture this state at declaration. The shared frame
                    // guard has already checked identity, range and protection.
                    if let Some((_, target)) = frame
                        .targets
                        .get(slot)
                        .and_then(|target| self.board(&target.id))
                    {
                        let op = if target.exhausted { *exhausted } else { *ready };
                        frame.steps.insert(
                            frame.cursor,
                            Step {
                                context: step.context,
                                op,
                            },
                        );
                    }
                    // Run the selected existing atomic operation without an
                    // extra death-settlement boundary between query and effect.
                    continue;
                }
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
                        let controller = self.controller_after_reset(id);
                        if let Some((r, c)) = self.leave_board(id) {
                            let mut c = self.fresh(c);
                            if let Some(controller) = controller {
                                c.controller = controller;
                            }
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
                        if self.board(id).is_some_and(|(from, _)| from != region) {
                            if let Some((_, c)) = self.remove_board(id) {
                                let actor = c.controller;
                                let source = self.source_snapshot(&c, Some(region));
                                self.regions[region].cards.push(c);
                                self.emit_event(actor, source, Event::EnterRegion);
                            }
                        }
                    }
                }
                Op::DestroyPublicAttachmentOrAsset { slot } => {
                    let id = frame
                        .targets
                        .get(slot)
                        .ok_or("缺少附属或资产目标")?
                        .id
                        .clone();
                    if let Some((s, i)) = self.asset_location(&id) {
                        let c = self.players[s].assets.remove(i);
                        let owner = c.owner;
                        self.note(format!("资产{}：消灭", card(&c.definition).name));
                        let c = self.reset_zone_card(c);
                        self.players[owner].graveyard.push(c);
                    } else {
                        self.remove_dead(&id, RemovalCause::Destroy);
                    }
                }
                Op::Destroy(entity) => {
                    if let Some(id) = Self::frame_entity(&frame, entity) {
                        self.remove_dead(id, RemovalCause::Destroy);
                    }
                }
                Op::DrawIfActorHasInitiative { count } => {
                    if self.team(frame.actor) == self.first_team {
                        self.draw(frame.actor, count)?;
                    }
                }
                Op::RevealHandAndOfferSourceSacrifice { player } => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    let revealed = self.players[seat].hand.clone();
                    self.note(format!(
                        "{} 展示手牌：{}",
                        self.players[seat].name,
                        revealed
                            .iter()
                            .map(|c| card(&c.definition).name.as_str())
                            .collect::<Vec<_>>()
                            .join("、")
                    ));
                    if !revealed.is_empty() {
                        let options = revealed
                            .iter()
                            .map(|c| self.option(c, frame.actor, None, None))
                            .collect();
                        let available = self.board(&frame.source.card.id).is_some_and(|(_, c)| {
                            c.controller == frame.actor
                                && !c.face_down
                                && card(&c.definition).kind == "character"
                        });
                        self.choice(
                            frame.actor,
                            "revealed-hand-discard",
                            "可选择一张并牺牲好奇的黑客，令该玩家弃牌".into(),
                            options,
                            0,
                            usize::from(available),
                            None,
                            ChoiceResolution::Frame {
                                frame: Box::new(frame),
                                choice: FrameChoice::RevealedHandDiscard { seat, revealed },
                            },
                        );
                        return Ok(());
                    }
                }
                Op::RandomHandToOwnerDeckTop { player } => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    if !self.players[seat].hand.is_empty() {
                        let i = self.random_below(self.players[seat].hand.len());
                        let c = self.players[seat].hand.remove(i);
                        let owner = c.owner;
                        self.note(format!(
                            "{} 随机展示 {}，置于其拥有者牌库顶",
                            self.players[seat].name,
                            card(&c.definition).name
                        ));
                        let c = self.reset_zone_card(c);
                        self.players[owner].deck.insert(0, c);
                    }
                }
                Op::MoveDeckTopToGraveyard { player, count } => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    for _ in 0..count {
                        if self.players[seat].deck.is_empty() {
                            break;
                        }
                        let c = self.players[seat].deck.remove(0);
                        let owner = c.owner;
                        let c = self.reset_zone_card(c);
                        self.players[owner].graveyard.push(c);
                    }
                }
                Op::OfferShuffleIfActorControlsDreamDemon { player } => {
                    if self.regions.iter().flat_map(|r| &r.cards).any(|c| {
                        c.controller == frame.actor
                            && !c.face_down
                            && card(&c.definition).kind == "character"
                            && self.current_subtypes(c).iter().any(|t| t == "梦魔")
                    }) {
                        let seat = self.frame_player(&frame, step.context, player)?;
                        self.choice(
                            frame.actor,
                            "optional-shuffle",
                            "是否令目标玩家洗牌？".into(),
                            vec![ChoiceOption {
                                id: "shuffle".into(),
                                label: "洗牌".into(),
                                card: None,
                            }],
                            0,
                            1,
                            None,
                            ChoiceResolution::Frame {
                                frame: Box::new(frame),
                                choice: FrameChoice::OptionalShuffle { seat },
                            },
                        );
                        return Ok(());
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
                    visibility,
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
                                    choice: FrameChoice::Search {
                                        seat,
                                        to_top,
                                        visibility,
                                    },
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
                    paid_reveal: false,
                }),
                Op::CostReductionOnFaceUpOrPaidReveal { filter, amount } => {
                    self.modifiers.push(CostModifier {
                        actor: frame.actor,
                        filter,
                        amount,
                        expires_turn: self.turn,
                        uses: 1,
                        paid_reveal: true,
                    })
                }
                Op::FreeReveal {
                    player,
                    require_loyalty,
                } => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    if self.world_free_reveal_choice(frame.clone(), seat, require_loyalty) {
                        return Ok(());
                    }
                }
                Op::SacrificeAndDraw(player) => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    if self.world_sacrifice_choice(frame.clone(), seat) {
                        return Ok(());
                    }
                }
                Op::ChooseRegion => {
                    self.world_region_choice(frame);
                    return Ok(());
                }
                Op::GraveyardEntry { player, region } => {
                    let seat = self.frame_player(&frame, step.context, player)?;
                    let region = self
                        .frame_region(&frame, region)
                        .ok_or("缺少墓地进场地区")?;
                    if self.world_graveyard_choice(frame.clone(), seat, region) {
                        return Ok(());
                    }
                }
                Op::HideMatching {
                    except_subtype,
                    mix_hidden,
                } => self.world_hide(except_subtype.as_deref(), mix_hidden),
                Op::SimultaneousSearch { filter } => {
                    let participants = self
                        .players
                        .iter()
                        .filter(|p| !p.eliminated)
                        .map(|p| p.seat)
                        .collect();
                    self.world_search_next(frame, filter, participants, vec![])?;
                    return Ok(());
                }
                Op::ForEachLivingPlayer(_) | Op::ForEachLivingPlayerFromActor(_) => {
                    return Err("禁止嵌套的玩家遍历声明".into())
                }
            }
            self.settle_deaths();
        }
        Ok(())
    }
    pub(crate) fn choose_frame(
        &mut self,
        mut frame: Box<ResolutionFrame>,
        choice: FrameChoice,
        chooser: usize,
        a: Action,
        selected: Vec<String>,
        option_ids: BTreeSet<String>,
    ) -> RuleResult<()> {
        match choice {
            FrameChoice::JZ50DeathSearch => {
                if chooser != frame.actor {
                    return Err("墓穴食尸鬼检索的选择者无效".into());
                }
                self.jz50_search_complete(&frame, &selected)?;
            }
            FrameChoice::BQ104EmployeeSearch | FrameChoice::XQ48PasserSearch => {
                if chooser != frame.actor { return Err("进场检索的选择者无效".into()); }
                self.entry_search_complete(&frame, &selected, matches!(choice, FrameChoice::BQ104EmployeeSearch))?;
            }
            FrameChoice::HandSeal { seat, host_id } => {
                if chooser != seat || frame.actor != seat
                    || frame.targets.first().is_none_or(|t| t.id != host_id) {
                    return Err("封印手牌的选择者无效".into());
                }
                let id = selected.first().ok_or("必须选择一张手牌封印")?;
                self.seal_hand_card(seat, id, &host_id)?;
            }
            FrameChoice::JC032TopSix { inspected_ids } => {
                self.jc032_complete(&frame, inspected_ids, &selected)?;
            }
            FrameChoice::JZ24Sacrifice { remaining_players } => {
                return self.jz24_complete(*frame, chooser, remaining_players, &selected);
            }
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
                    self.discard_hand_card(seat, &id)?;
                }
                if redraw {
                    self.draw(seat, count)?;
                }
            }
            FrameChoice::RevealedHandDiscard { seat, .. } => {
                if let Some(id) = selected.first() {
                    if !self.board(&frame.source.card.id).is_some_and(|(_, c)| {
                        c.controller == frame.actor
                            && !c.face_down
                            && card(&c.definition).kind == "character"
                    }) {
                        return Err("原来源实例不可作为牺牲支付".into());
                    }
                    if !self.players[seat].hand.iter().any(|c| c.id == *id) {
                        return Err("展示手牌实例已失效".into());
                    }
                    self.remove_dead(&frame.source.card.id, RemovalCause::Sacrifice);
                    self.discard_hand_card(seat, id)?;
                }
            }
            FrameChoice::OptionalShuffle { seat } => {
                if !selected.is_empty() {
                    self.shuffle_player(seat);
                }
            }
            FrameChoice::Search {
                seat,
                to_top,
                visibility,
            } => {
                let c = selected
                    .first()
                    .and_then(|id| self.players[seat].deck.iter().position(|c| c.id == *id))
                    .map(|i| self.players[seat].deck.remove(i));
                if to_top {
                    self.shuffle_player(seat);
                }
                if let Some(c) = c {
                    let c = self.fresh(c);
                    if to_top {
                        self.players[seat].deck.insert(0, c);
                    } else {
                        if visibility == SearchVisibility::Reveal {
                            self.note(format!(
                                "{} 展示检索的 {}",
                                self.players[seat].name,
                                card(&c.definition).name
                            ));
                        } else {
                            self.note(format!("{} 完成私密检索", self.players[seat].name));
                        }
                        self.players[seat].hand.push(c);
                    }
                }
                if !to_top {
                    self.shuffle_player(seat);
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
            FrameChoice::FreeReveal {
                seat,
                require_loyalty,
            } => {
                if let Some(id) = selected.first() {
                    self.world_reveal(seat, id, require_loyalty)?;
                }
            }
            FrameChoice::SacrificeDraw { seat } => {
                let id = selected.first().ok_or("须选择一个角色牺牲")?;
                let (region, c) = self
                    .board(id)
                    .filter(|(_, c)| {
                        c.controller == seat
                            && !c.face_down
                            && card(&c.definition).kind == "character"
                    })
                    .ok_or("牺牲角色已失效")?;
                let amount = self.defense(c, region) as usize;
                self.remove_dead(id, RemovalCause::Sacrifice);
                self.draw(seat, amount)?;
            }
            FrameChoice::RepressOne { seat, regions } => {
                let region = selected
                    .first()
                    .and_then(|id| id.strip_prefix("region:"))
                    .and_then(|s| s.parse::<usize>().ok())
                    .ok_or("缺少遏制地区")?;
                let original = regions
                    .iter()
                    .find(|(r, _)| *r == region)
                    .ok_or("遏制地区不属于原选项")?;
                let team = self.team(seat);
                let r = self
                    .regions
                    .get_mut(region)
                    .filter(|r| r.card.id == original.1 && r.influence[team] > 0)
                    .ok_or("遏制地区或势力已失效")?;
                r.influence[team] -= 1;
                self.note(format!(
                    "{} 从地区{}移除1个本方势力标志",
                    self.players[seat].name,
                    region + 1
                ));
            }
            FrameChoice::Region => {
                let region = selected
                    .first()
                    .and_then(|id| id.strip_prefix("region:"))
                    .and_then(|r| r.parse::<usize>().ok())
                    .filter(|r| *r < self.regions.len())
                    .ok_or("地区选择已失效")?;
                frame.chosen_region = Some(region);
            }
            FrameChoice::GraveyardEntry { seat, region } => {
                self.world_graveyard_entry(
                    seat,
                    region,
                    selected.first().ok_or("须选择墓地角色")?,
                )?;
            }
            FrameChoice::SimultaneousSearch {
                filter,
                participants,
                mut committed,
            } => {
                let seat = *participants
                    .get(committed.len())
                    .ok_or("检索承诺阶段已失效")?;
                committed.push((seat, selected.first().cloned()));
                self.world_search_next(*frame, filter, participants, committed)?;
                return Ok(());
            }
        }
        self.effects.push_front(Effect::Frame { frame });
        Ok(())
    }
}
