//! Shared declaration, payment, target guard and resumable finite operation interpreter.
use crate::{
    catalog::card,
    engine::RuleResult,
    model::*,
    rules::{self, *},
};
use std::collections::{BTreeMap, BTreeSet};

impl Game {
    // The operation beneath authenticated movement programs. It preserves the
    // existing entity and attachments, and emits EnterRegion only. Primitive
    // unit fixtures call this operation rather than inventing a legal ability.
    pub(crate) fn move_on_board_instance(&mut self, id: &str, region: usize) {
        if self.attached_character(id) { return; }
        let followers = self.attachments.iter().filter(|a| a.card.definition == "BQ026" && a.host_id == id && !a.card.face_down).map(|a| a.card.id.clone()).collect::<Vec<_>>();
        if self.board(id).is_some_and(|(from, c)| from != region && !self.jc064_blocks_character(c, region)) {
            if let Some((_, c)) = self.remove_board(id) {
                let actor = c.controller;
                let source = self.source_snapshot(&c, Some(region));
                self.regions[region].cards.push(c);
                // A hidden card has no printed character abilities. Moving it
                // must not announce its front or emit that front's entry trigger.
                if !source.card.face_down {
                    self.emit_event(actor, source, Event::EnterRegion);
                    for id in followers { if let Some((r,c)) = self.board(&id) { let s=self.source_snapshot(c,Some(r)); self.emit_event(c.controller,s,Event::EnterRegion); } }
                }
            }
        }
    }
    // Shared with XQ24; preserve the existing single-character Hide cleanup,
    // identity reset, orientation and region rather than creating a second rule.
    pub(crate) fn hide_in_place(&mut self, id: &str) -> bool {
        self.hide_in_place_successor(id).is_some()
    }
    // Return the actual newly allocated object for effects that explicitly
    // continue to refer to the card after it has become a new hidden entity.
    pub(crate) fn hide_in_place_successor(&mut self, id: &str) -> Option<Card> {
        self.hide_in_place_successor_in_batch(id, None)
    }
    pub(crate) fn hide_in_place_successor_in_batch(&mut self, id: &str, event: Option<&str>) -> Option<Card> {
        let observation = self.board(id).filter(|(_, c)| c.definition == "JC054" && !c.face_down).map(|(r, c)| {
            let baseline = self.control_baselines.iter().find(|b| b.target_instance == id).cloned();
            let base_seat = baseline.as_ref().map_or(c.controller, |b| b.controller);
            (self.source_snapshot(c, Some(r)), baseline, self.players[base_seat].eliminated)
        });
        let controller = self.controller_after_reset(id);
        if let Some((r, c)) = self.leave_board(id) {
            let mut c = self.fresh(c);
            if let Some(controller) = controller { c.controller = controller; }
            c.face_down = true;
            c.damage = 0;
            c.wounds = 0;
            c.shield = 0;
            self.regions[r].cards.push(c.clone());
            if let Some((source, baseline, eliminated)) = observation {
                self.jc054_observe_hide(source, c.clone(), baseline, eliminated, event);
            }
            return Some(c);
        }
        None
    }
    fn attachment_host_matches(
        &self,
        actor: usize,
        id: &str,
        condition: &AttachmentHostCondition,
    ) -> bool {
        let Some(attachment) = self.attachments.iter().find(|a| a.card.id == id) else {
            return false;
        };
        let live_society = rules::definition(&attachment.card.definition).attachment.as_ref()
            .is_some_and(|spec| spec.host.zone == Zone::Society) && self.attachment_host_valid(attachment);
        if self.attachment_region(attachment).is_none() && !live_society {
            return false;
        }
        match condition {
            AttachmentHostCondition::CQ24Host { .. } => false, // handled before zone dispatch below
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
        region.filter(|r| self.region_live(*r))
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
                    EntityKind::Attachment => (!c.face_down || crate::batch4_plans::secret(c)) && crate::catalog::has_kind(d, "attachment"),
                    EntityKind::Hidden => c.face_down && !crate::batch4_plans::is_plan(c),
                    EntityKind::CharacterOrHidden => !crate::batch4_plans::is_plan(c) && (c.face_down || d.kind == "character"),
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
                        .is_none_or(|wanted| self.frame_region(frame, wanted).is_some_and(|wanted| *r == Some(wanted)))
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
        if c.definition=="CQ28" && kind=="play" { return vec![(Action{card_id:Some(c.id.clone()),..Action::new("play")},"咒欲之门：先声明两次同项及目标".into())]; }
        if c.definition == "MSJC14" && kind == "activate" { return self.sentinel_candidates(actor, c); }
        if c.definition == "JC065" && kind == "play" { return self.jc065_action_candidates(actor, c); }
        if c.definition == "BQ089" && kind == "play" { return self.bq089_action_candidates(actor, c); }
        if c.definition == "JC062" && kind == "play" { return self.jc062_action_candidates(actor, c); }
        if c.definition == "XQ20" && kind == "play" { return self.xq20_action_candidates(actor, c); }
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
                    (0..self.regions.len()).filter(|&r| self.region_live(r)).map(Some).collect()
                } else {
                    vec![None]
                };
                let costs = if ability
                    .costs
                    .iter()
                    .any(|c| matches!(c, Cost::SacrificeSelectedControlledCharacter | Cost::SacrificeSelectedOtherLocalControlledCharacter))
                {
                    let local_other = ability.costs.iter().any(|cost| matches!(cost, Cost::SacrificeSelectedOtherLocalControlledCharacter));
                    self.in_play_cards().into_iter()
                        .filter(|(r, c)| {
                            c.controller == actor
                                && !c.face_down
                                && card(&c.definition).kind == "character"
                                && (!local_other || (*r == source.region && c.id != source.card.id))
                        })
                        .map(|(_, c)| Some(vec![c.id.clone()]))
                        .collect()
                } else if ability.costs.iter().any(|c| matches!(c, Cost::IndustryExhaustOther)) {
                    self.industry_cost_candidates(actor,&source.card.id).into_iter().map(|id|Some(vec![id])).collect()
                } else if ability.costs.iter().any(|c| matches!(c, Cost::BQ036SacrificeAttachment)) {
                    self.bq036_contest_cost_candidates(actor).into_iter().map(|id| Some(vec![id])).collect()
                } else if ability.costs.iter().any(|c| matches!(c, Cost::JZ39SacrificeDemon)) {
                    self.jz39_demons(actor).into_iter().map(|id| Some(vec![id])).collect()
                } else if let Some(required) = ability.costs.iter().find_map(|cost| if let Cost::SacrificeControlledSubtype(subtypes) = cost { Some(subtypes) } else { None }) {
                    self.regions.iter().flat_map(|r| &r.cards).filter(|c| !c.face_down && c.controller == actor
                        && card(&c.definition).kind == "character" && required.iter().any(|s| self.current_subtypes(c).contains(s)))
                        .map(|c| Some(vec![c.id.clone()])).collect()
                } else if ability.costs.iter().any(|c| matches!(c, Cost::RemoveOneTimeFromSource | Cost::Batch3RemoveTime)) {
                    self.wm034_time_payments(&source).into_iter().map(|p| Some(vec![p.carrier.id])).collect()
                } else if ability.costs.iter().any(|c| matches!(c, Cost::RemoveOneTimeFromSelectedControlledCard)) {
                    self.jc044_time_payments(actor).into_iter().map(|p| Some(vec![p.carrier.id])).collect()
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
                            if ability.key == crate::import20_sentinel::FIRE { a.amount=Some(self.effective_marker_count(c,crate::batch4_markers::MarkerKind::Charge)); }
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
                                    .and_then(|id| ability.costs.iter().any(|c| matches!(c, Cost::RemoveOneTimeFromSelectedControlledCard))
                                        .then(|| self.jc044_time_payments(actor).into_iter().find(|p| p.carrier.id == *id)).flatten()
                                        .map(|p| format!("（费用：从{} [{}]移除1个时间标志）", card(&p.carrier.definition).name, p.carrier.id))
                                        .or_else(|| self.players[actor]
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
                                        )))))
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
            .or_else(|| self.in_play(id))
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
            } else if crate::batch4_plans::secret(c) {
                "秘密计划".into()
            } else if c.face_down {
                format!("暗藏者·{}", self.players[c.controller].name)
            } else {
                card(&c.definition).name.clone()
            };
            (
                label,
                if asset.is_some() {
                    "asset"
                } else if crate::batch4_plans::secret(c) {
                    "attachment"
                } else if c.face_down {
                    "hidden"
                } else {
                    if (spec.kind == EntityKind::Attachment || spec.zone == Zone::AttachmentOrAsset) && crate::catalog::has_kind(card(&c.definition), "attachment") { "attachment" } else { card(&c.definition).kind.as_str() }
                }
                .into(),
                Some(player_id(c.owner)),
                Some(player_id(c.controller)),
                region,
            )
        } else if let Some(seat) = self.society_host_seat(id) {
            (format!("{}的秘社", self.players[seat].name), "society".into(),
                Some(player_id(seat)), Some(player_id(seat)), None)
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
            .filter(|r| self.region_live(*r))
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
            || self.in_play(&target.id).is_some()
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
                Zone::Board => self.in_play(&target.id).is_some(),
                Zone::Society => self.society_host_seat(&target.id).is_some(),
                Zone::Graveyard => self
                    .players
                    .iter()
                    .any(|p| p.graveyard.iter().any(|c| c.id == target.id)),
                Zone::Region => target.id.strip_prefix("region:")
                    .and_then(|r| r.parse::<usize>().ok())
                    .and_then(|r| self.regions.get(r))
                    .is_some_and(|r| !r.vacant && target.region_instance.as_ref().is_none_or(|id| r.card.id == *id)),
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
        // A hidden-front trigger does not reveal its print through metadata.
        let hidden_trap=(frame.source.card.definition=="JZ26" && frame.source.card.face_down
            && frame.ability_key=="batch3-jz26-trap") || crate::batch4_plans::secret(&frame.source.card);
        StackView {
            id: frame.frame_id.clone(),
            label,
            controller: player_id(frame.actor),
            card_id: (!hidden_trap).then(||frame.source.card.definition.clone()),
            target_id: frame.targets.first().map(|t| t.id.clone()),
            ability_id: (!hidden_trap).then(||frame.ability_key.clone()),
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
            death_echo: None,
            observed_exposure: None,
            observed_death: None,
                                        observed_mobility_grant: None,            card: {
                let mut snapshot = c.clone();
                if c.definition == "JZ30" && !c.face_down && self.board(&c.id).is_some() {
                    snapshot.time_markers = self.effective_time_markers(c);
                }
                snapshot
            },
            region,
            source_region_instance: if c.definition==crate::cq10::ID || c.definition==crate::bq084::ID || c.definition==crate::bq001::ID || crate::next20_black_wraiths::CARDS.contains(&c.definition.as_str()) || crate::next20_blue_vampires::CARDS.contains(&c.definition.as_str()) || c.definition=="BQ057" || c.definition=="CQ18" || crate::import20_industry::CARDS.contains(&c.definition.as_str()) || c.definition == crate::import20_carmen::ID || c.definition == crate::import20_tattoo::ID || c.definition == crate::import20_cultist::ID || crate::import20_broker::CARDS.contains(&c.definition.as_str()) || crate::import20_vampires::CARDS.contains(&c.definition.as_str()) || crate::import20_observers::CARDS.contains(&c.definition.as_str()) || crate::batch4_vampires::CARDS.contains(&c.definition.as_str()) || crate::batch3::IMPLEMENTING.contains(&c.definition.as_str()) || crate::batch4::IMPLEMENTING.contains(&c.definition.as_str()) || crate::pack_a_contest::CARDS.contains(&c.definition.as_str()) || matches!(c.definition.as_str(), "BQ077" | "BQ085" | "CQ26" | "XG27" | "JC032" | "JZ24" | "JZ31" | "BQ104" | "XQ48" | "JZ43" | "JZ22" | "XQ37" | "JZ44" | "JZ45" | "XQ24" | "WM059" | "BQ078" | "XQ18" | "JZ30" | "BQ028" | "BQ030" | "JC061" | "JC087" | "JC094" | "WM056" | "WM023" | "JC052" | "JC046" | "JC054" | "JC095" | "BQ035" | "JZ46" | "JC057" | "JC044" | "JC066" | "JC041" | "XQ23" | "JC035" | "JC043" | "BQ054" | "JC048" | "WM034" | "BQ091" | "BQ094" | "BQ080" | "JZ57" | "CQ16" | "JC038" | "JC067" | "JZ33" | "JZ52" | "JC040" | "JZ23" | "JZ28" | "XG10" | "CQ08" | "BQ059" | "CQ17" | "JC051" | "JC055" | "JZ32" | "JZ34" | "JZ41" | "WM039" | "WM040") {
                region.and_then(|r| self.regions.get(r)).filter(|r| !r.vacant).map(|r| r.card.id.clone())
            } else { None },
            attachment_host_instance: (c.definition=="XG20" || c.definition=="CQ19" || crate::import20_industry::CARDS.contains(&c.definition.as_str()) || c.definition == crate::import20_tattoo::ID || matches!(c.definition.as_str(), "BQ039" | "WM028" | "WM044" | "BQ038" | "JZ40" | "JZ56" | "XQ15" | "WM045" | "WM046" | "WM047" | "XG21" | "JZ38" | "BQ041" | "XG12" | "JZ39" | "BQ026" | "XQ26" | "JC064" | "XG28" | "XQ39" | "JC090" | "WM038" | "JZ46" | "JC035" | "JC048" | "BQ091" | "JC068" | "BQ094" | "JZ57" | "JC033" | "JC034" | "JZ29" | "JC039" | "JZ41") || rules::definition(&c.definition)
                .abilities
                .iter()
                .any(|a| {
                    a.ops
                        .iter()
                        .any(|op| matches!(op, Op::ModifyAttachmentHostUntilTurnEnd))
                }))
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
        card(&c.definition).cost.saturating_sub(self.wm024_cost_reduction(actor, c)).saturating_sub(self.xg27_curse_reduction(actor, c)).saturating_sub(if matches!(c.definition.as_str(), "JZ33" | "XG14" | "XG15") { 2 * self.destroyed_region_count() } else { 0 }).saturating_sub(
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
        card(&c.definition).cost.saturating_sub(self.wm024_cost_reduction(actor, c)).saturating_sub(self.wm025_reveal_reduction(actor, c)).saturating_sub(self.jc033_reveal_reduction(actor)).saturating_sub(
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
        let payment = self.capture_money(actor, amount)?;
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
        Ok(vec![payment])
    }
    pub(crate) fn ability_for_action(
        &self,
        definition: &str,
        a: &Action,
    ) -> RuleResult<AbilitySpec> {
        crate::import20_observers::validate_action(definition, a)?;
        crate::import20_contest::validate_action(definition, a)?;
        crate::import20_industry::validate_action(definition,a)?;
        if definition == "WM043" { crate::wm043::validate_action(a)?; }
        crate::next20_blue_gray::validate_action(definition, a)?;
        if definition == "XQ23" && *a != (Action {
            card_id: a.card_id.clone(), target_id: a.target_id.clone(), ability_id: a.ability_id.clone(),
            ..Action::new(&a.kind)
        }) {
            return Err("XQ23行动只声明原来源和一个本方资产目标".into());
        }
        if definition == "WM023" && *a != (Action {
            card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), region: a.region,
            ..Action::new(&a.kind)
        }) {
            return Err("WM023行动只声明一个地区，不声明手牌目标或额外选项".into());
        }
        if definition == "WM056" && *a != (Action {
            card_id: a.card_id.clone(),
            ability_id: a.ability_id.clone(),
            ..Action::new(&a.kind)
        }) {
            return Err("WM056移动势力行动不声明目标、地区或额外选择".into());
        }
        if definition == "XQ39" && *a != (Action {
            card_id: a.card_id.clone(), ability_id: a.ability_id.clone(), target_id: a.target_id.clone(),
            ..Action::new(&a.kind)
        }) {
            return Err("方碑印记仅声明本牌、印刷能力及其单个目标".into());
        }
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
        if matches!(spec.attachment_host_condition, Some(AttachmentHostCondition::CQ24Host { .. })) {
            return self.cq24_host_binding(actor, source, spec, id);
        }
        if spec.predicate == Some(TargetPredicate::Next20XG25AdjacentEnemy) { return self.xg25_adjacent_target(actor,source,id); }
        match spec.zone {
            Zone::Society => self.society_host_seat(id).is_some_and(|seat| match spec.relation {
                Relation::Any => true,
                Relation::ControlledByActor | Relation::OwnedByActor => seat == actor,
                Relation::FriendlyTeam => self.team(seat) == self.team(actor),
                Relation::EnemyTeam => self.team(seat) != self.team(actor),
            }),
            Zone::AttachmentOrAsset => {
                if spec.predicate == Some(TargetPredicate::XQ23ActorAsset) {
                    return source.card.definition == "XQ23" && self.asset_location(id)
                        .is_some_and(|(seat, index)| seat == actor && self.players[seat].assets[index].controller == actor
                            && !self.players[seat].assets[index].face_down);
                }
                self.asset_location(id)
                    .is_some_and(|(s, i)| !self.players[s].assets[i].face_down)
                    || self.board(id).is_some_and(|(_, c)| !c.face_down && c.definition == "BQ026" && self.targetable(actor, c) && self.bq026_spell_targetable(actor, source, c))
                    || self
                        .attachments
                        .iter()
                        .find(|a| a.card.id == id)
                        .is_some_and(|a| {
                            (!a.card.face_down || crate::batch4_plans::secret(&a.card))
                                && self.attachment_host_valid(a)
                                && self.targetable(actor, &a.card) && self.bq026_spell_targetable(actor, source, &a.card)
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
                    self.region_live(r)
                        && (spec.predicate != Some(TargetPredicate::BQ077WaterRegion)
                            || (source.card.definition == "BQ077" && card(&self.regions[r].card.definition).subtypes.iter().any(|s| s == "水域")))
                        && (spec.predicate != Some(TargetPredicate::Batch4DestroyedRegion) || (source.card.definition == "XG14" && self.regions[r].destroyed))
                        && (spec.predicate != Some(TargetPredicate::JZ34DestroyedRegion) || (source.card.definition == "JZ34" && self.regions[r].destroyed))
                        && (spec.range != Range::Mobility
                            || source.region.is_some_and(|from| {
                                self.regions_adjacent(r, from) && !self.jc064_blocks_character(&source.card, r)
                            }))
                }),
            Zone::Board | Zone::Graveyard => {
                let located = if spec.zone == Zone::Board {
                    self.in_play(id)
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
                        EntityKind::Attachment => (!c.face_down || crate::batch4_plans::secret(c)) && crate::catalog::has_kind(d, "attachment"),
                        EntityKind::Hidden => c.face_down && !crate::batch4_plans::is_plan(c),
                        EntityKind::CharacterOrHidden => !crate::batch4_plans::is_plan(c) && (c.face_down || d.kind == "character"),
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
                            || (!c.face_down && self.current_magic_count(c) > 0))
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
                            .is_none_or(|max| (!c.face_down && d.cost <= max)
                                || (max >= 1 && crate::import20_carmen::secret_plan_cost_target(source, spec, c)))
                        && spec
                            .predicate
                            .as_ref()
                            .is_none_or(|predicate| match predicate {
                                TargetPredicate::Next20XG25AdjacentEnemy => self.xg25_adjacent_target(actor,source,&c.id),
                                TargetPredicate::PackAOwnGraveCharacter => self.pack_a_grave_target(actor, source, c),
                                TargetPredicate::XQ21MarkedCarrier => self.xq21_target(source, c),
                                TargetPredicate::XQ20MarkerCarrier => self.xq20_target(source, c),
                                TargetPredicate::XG17PersonalPlan => source.card.definition == "XG17" && crate::batch4_plans::is_plan(c) && self.attachments.iter().any(|a| a.card.id == c.id && a.host_id == format!("society:p{actor}") && self.attachment_host_valid(a)),
                                TargetPredicate::BQ084OwnGraveCharacter => self.bq084_grave_target(actor,source,c),
                                TargetPredicate::Batch3OwnGraveCharacter => self.batch3_own_grave_character(actor, source, c),
                                TargetPredicate::WM044HiddenOrSecretPlan => source.card.definition == "WM044" && c.face_down && (crate::batch4_plans::secret(c) || self.board(&c.id).is_some()),
                                TargetPredicate::WM057HiddenOrSecretPlan => source.card.definition == "WM057" && c.face_down && (crate::batch4_plans::secret(c) || self.board(&c.id).is_some()),
                                TargetPredicate::JC039BlueCharacter => source.card.definition == "JC039" && !c.face_down && d.color == "蓝",
                                TargetPredicate::BQ077WaterRegion => false,
                                TargetPredicate::JZ34DestroyedRegion | TargetPredicate::Batch4DestroyedRegion => false,
                                TargetPredicate::JC055CharacterOrAttachment => source.card.definition == "JC055" && !c.face_down && matches!(d.kind.as_str(), "character" | "attachment"),
                                TargetPredicate::JC043SocietyOrOriginalRegion | TargetPredicate::BQ042SocietyOrOriginalRegion => {
                                    matches!(source.card.definition.as_str(), "JC043" | "BQ042") && self.attachments.iter().find(|a| a.card.id == c.id)
                                        .is_some_and(|a| self.attachment_host_valid(a) && (self.society_host_seat(&a.host_id).is_some()
                                            || source.region.and_then(|r| self.regions.get(r)).is_some_and(|r|
                                                !r.vacant && source.source_region_instance.as_ref() == Some(&r.card.id)
                                                    && a.host_id == r.card.id)))
                                }
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
                                TargetPredicate::WM043LockedTarget => source.card.definition == "WM043" && self.sentinel_has_effective_lock(c),
                                TargetPredicate::SentinelLockedAnchor => source.card.definition == "MSJC14" && self.board(&c.id).is_some() && self.sentinel_has_effective_lock(c),
                                rules::TargetPredicate::JC069LockedAnchor => {
                                    source.card.definition == "JC069" && self.effective_marker_count(c, crate::batch4_markers::MarkerKind::Lock) > 0
                                }
                                rules::TargetPredicate::XQ23ActorAsset => false,
                                rules::TargetPredicate::XG16CharacterOrAttachment => self.xg16_time_target(source, c),
                                rules::TargetPredicate::JZ36CharacterOrAttachment => self.jz36_target(source, c),
                                rules::TargetPredicate::JZ40CharacterOrAttachment => self.jz40_target(source, c),
                                rules::TargetPredicate::ClockCharacterOrAttachment => self.clock_time_target(source,c),
                                rules::TargetPredicate::XQ18CharacterOrAttachment => {
                                    self.xq18_character_or_attachment(source, c)
                                }
                                rules::TargetPredicate::JZ45LockedLocalTarget => {
                                    source.card.definition == "JZ45" && self.effective_marker_count(c, crate::batch4_markers::MarkerKind::Lock) > 0
                                }
                            })
                        && (spec.range != Range::SourceRegion || region == source.region)
                        && (!(spec.range == Range::SourceRegion && source.source_region_instance.is_some()) && !matches!(source.card.definition.as_str(), "JZ43" | "JZ44" | "JZ45" | "WM059" | "JC061" | "JC087" | "JZ46" | "JC066" | "JC041" | "BQ054" | "CQ16") || spec.range != Range::SourceRegion
                            || source.region.and_then(|r| self.regions.get(r))
                                .zip(source.source_region_instance.as_ref())
                                .is_some_and(|(region, instance)| !region.vacant && region.card.id == *instance))
                        && spec.subtype.as_ref().is_none_or(|s| {
                            !c.face_down && self.target_subtypes(c, spec).contains(s)
                        })
                        && (spec.subtypes_any.is_empty()
                            || (!c.face_down
                                && spec
                                    .subtypes_any
                                    .iter()
                                    .any(|s| self.target_subtypes(c, spec).contains(s))))
                        && (spec.zone != Zone::Board || (self.targetable(actor, c) && self.bq026_spell_targetable(actor, source, c)))
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
            Zone::Society => self.players.iter().map(|p| ChoiceOption {
                id: format!("society:p{}", p.seat), label: format!("{}的秘社", p.name), card: None,
            }).collect(),
            Zone::AttachmentOrAsset => self
                .attachments
                .iter()
                .filter_map(|a| {
                    self.in_play(&a.card.id)
                        .map(|(r, c)| self.option(c, actor, r, Some("attachment")))
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
                .map(|(r, c)| self.option(c, actor, r, None))
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
                .filter(|(_, reg)| !reg.vacant)
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
                        .is_some_and(|r| !r.vacant && r.card.id == *instance)
                }))
    }
    fn requires_region_instance(source: &SourceSnapshot, slot: &TargetSlotSpec) -> bool {
        slot.zone == Zone::Region
            && ((source.card.definition == "CQ24" && matches!(slot.attachment_host_condition, Some(AttachmentHostCondition::CQ24Host { .. }))) || source.card.definition == "XG14" || matches!(source.card.definition.as_str(), "LC16" | "JZ36" | "BQ089" | "MSJC11" | "JC033" | "JC065")
                || (source.card.definition == "JZ34" && slot.predicate == Some(TargetPredicate::JZ34DestroyedRegion))
                || (source.card.definition == "BQ077" && slot.predicate == Some(TargetPredicate::BQ077WaterRegion))
                // WM023's own hand-entry action freezes a region UID. A fully
                // canonical granted mobility destination retains the shared
                // slot contract, independently of the recipient's print.
                || (source.card.definition == "WM023"
                    && !crate::mobility::canonical_target(slot))
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
        if source.card.definition=="CQ28" { return self.cq28_bind(actor,source,spec,a); }
        if source.card.definition=="CQ19" { crate::import20_sentinel::validate_action(spec,a)?; }
        if source.card.definition == "CQ24" && spec.targets.len()==2 { return self.cq24_bind(actor,source,spec,a); }
        if source.card.definition == "MSJC14" { return self.sentinel_bind(actor, source, spec, a); }
        if source.card.definition == "JC065" { return self.jc065_bind_action(actor, source, spec, a); }
        if source.card.definition == "JC062" { return self.jc062_bind(actor, source, spec, a); }
        if source.card.definition == "XQ20" { return self.xq20_bind(actor, source, spec, a); }
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
                // The FAQ forbids choosing an in-place movement. This is an
                // activation check; later response movement does not add an
                // unprinted distance condition to the target's frame guard.
                if slot.predicate == Some(TargetPredicate::JC069LockedAnchor)
                    && self.board(&source.card.id).zip(self.board(&id))
                        .is_some_and(|((from, _), (to, _))| from == to)
                {
                    return Err("不能原地移动".into());
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
        self.xg13_check_cost(source, a)?;
        self.validate_wm038_activation_cost(actor, source, spec, a)?;
        self.jc044_check_activation(source, a)?;
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
        let borrowed=self.wm057_capture_borrow(actor,source,spec,a)?;
        let allow_borrow=borrowed.is_some();
        let mut paid = vec![];
        for cost in &spec.costs {
            match cost {
                Cost::CQ15ReturnSword(g) => paid.push(self.cq15_return_cost(actor,source,g)?),
                Cost::Honor(n) => paid.push(self.pete_pay_honor(actor,source,spec,*n)?),
                Cost::RemoveChargeX => paid.push(self.cq19_pay_charge(source,a)?),
                Cost::IndustryExhaustOther => paid.push(self.industry_pay_other(actor,source,a)?),
                Cost::BQ036SacrificeAttachment => paid.push(self.bq036_pay_attachment(actor, a)?),
                Cost::XQ35SealCharacter => paid.push(self.xq35_pay(actor,source,a)?),
                Cost::JZ51SealThreeCorpses => return Err("温迪戈封印费用必须通过墓地专属待选支付".into()),
                Cost::JZ36SacrificeBatch => paid.push(self.jz36_pay(actor, a)?),
                Cost::JZ39SacrificeDemon => paid.push(self.pay_jz39_demon(actor, a)?),
                Cost::JC055RemoveTime => return Err("JC055须通过时间标志支付选择".into()),
                Cost::SacrificeControlledSubtype(subtypes) => paid.push(self.pay_subtype_sacrifice(actor, subtypes, a)?),
                Cost::Batch3RemoveTime => paid.push(self.batch3_pay_time(source, a)?),
                Cost::RemoveOneTimeFromSource => paid.push(self.wm034_pay_time(source, a)?),                Cost::RemoveOneTimeFromSelectedControlledCard => paid.push(self.jc044_pay_time(actor, a)?),
                Cost::Assets(n) => {
                    paid.push(self.capture_money(actor, *n)?);
                }
                Cost::ExhaustSource => {
                    let c = self
                        .ability_source_mut(&source.card.id)
                        .filter(|c| (c.controller == actor || allow_borrow && c.id==source.card.id && c.controller==source.card.controller) && !c.face_down && !c.exhausted)
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
                Cost::SacrificeSource | Cost::SacrificeSelectedControlledCharacter | Cost::SacrificeSelectedOtherLocalControlledCharacter => {
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
                        .in_play(&id)
                        .filter(|(r, c)| {
                            c.controller == actor
                                && !c.face_down
                                && (!matches!(cost, Cost::SacrificeSelectedOtherLocalControlledCharacter)
                                    || (*r == source.region && c.id != source.card.id))
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
        if let Some(permission)=borrowed {paid.push(PaidCost::WM057Borrowed(permission));}
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
                    // WM061 retains all original seats, including no-op slots
                    // for eliminated players, so restore can authenticate the
                    // entire printed all-player program after any response.
                    if self.players[seat].eliminated
                        && !(source.card.definition == "WM061" && spec.key == crate::pack_a_contest::EARTH) {
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
        let (already_paid,resource_payments)=crate::step_resources::unpack_payments(paid);
        ResolutionFrame {
            resource_payments,
            frame_id: String::new(),
            ability_key: spec.key.clone(),
            actor,
            source,
            targets,
            already_paid,
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
        let previous = self.effects.len();
        let observe_disaster = transaction.is_some();
        let observe_spell = transaction.as_ref().is_some_and(|c| card(&c.definition).kind == "spell");
        let target = frame.targets.first().map(|t| t.id.clone());
        let actor = frame.actor;
        self.push_stack(actor, label, transaction, None, false, target);
        frame.frame_id = self.stack.last().unwrap().id.clone();
        if observe_spell { self.cq07_observe_spell_targets(&frame); }
        if observe_disaster { self.pete_observe_attachment(&frame); self.wm058_observe_card(&frame); self.bq034_observe_card(&frame); self.observe_batch4_disaster(&frame); }
        self.bq038_observe_raid(&frame);
        self.batch3_register_entry_order(&frame.frame_id, previous);
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
                // Preserve the established immediate-card ordering: capture
                // target events now, execute the card without a response item,
                // then declare observers. A later tax finds no stack card.
                if transaction.is_some() { self.pete_observe_attachment(&frame); self.wm058_observe_card(&frame); self.bq034_observe_card(&frame); }
                if transaction.is_some() { self.observe_batch4_disaster(&frame); }
                if transaction.as_ref().is_some_and(|c| card(&c.definition).kind == "spell") {
                    self.cq07_observe_spell_targets(&frame);
                }
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
        if event == Event::ConfrontationStart {
            let ability = self.xg12_event_ability(actor,&source).unwrap_or_else(crate::mobility::ability);
            let declaration = Declaration { actor, source, ability };
            if self.validate_mobility_event_source(&declaration).is_ok()
                && !self.turn_ability_usage.iter().any(|u| u.ability_key == "mobility"
                    && u.source_instance == declaration.source.card.id && u.turn == self.turn)
                && !self.effects.iter().any(|e| matches!(e, Effect::Declare { declaration: d }
                    if d.ability.key == "mobility" && d.source.card.id == declaration.source.card.id))
                && !self.stack.iter().any(|s| s.frame.as_ref().is_some_and(|f|
                    f.ability_key == "mobility" && f.source.card.id == declaration.source.card.id))
                && !self.pending.as_ref().is_some_and(|p| matches!(&p.resolution,
                    ChoiceResolution::Declare { declaration: d, .. }
                    if d.ability.key == "mobility" && d.source.card.id == declaration.source.card.id)) {
                self.turn_ability_usage.push(TurnAbilityUsage {
                    source_instance: declaration.source.card.id.clone(), ability_key: "mobility".into(),
                    turn: self.turn, uses: 1,
                });
                self.effects.push_back(Effect::Declare { declaration });
            }
            return;
        }
        // User ruling: freeze successful attachment facts before death cleanup.
        // enter_triggers runs after the attachment's continuous control change.
        if event == Event::Enter
            && self.attachments.iter().any(|a| a.card.id == source.card.id)
        {
            self.observe_attachment_committed(&source.card.id);
            self.observe_bq080_attachment_entry(&source.card.id);
            self.observe_industry_entry(&source);
        }
        if event == Event::Reveal { self.observe_jz29_vampire_reveal(&source);
            self.observe_jz23_vampire_reveal(&source); }
        let granted_raid = event == Event::Reveal && self.emit_jc038_raid(&source);
        let specs = rules::definition(&source.card.definition).abilities.iter()
            .filter(|s| s.event == Some(event) && !(granted_raid && s.key.starts_with("raid-"))).cloned().collect::<Vec<_>>();
        for mut spec in specs {
            if source.card.definition == "WM024" && spec.key == crate::import20_vampires::RAID {
                spec = self.wm024_reveal_ability(&source);
            }
            if source.card.definition == "XG11" && spec.key == crate::batch4_spectral::KEY {
                let Some(captured) = self.xg11_reveal_ability(&source) else { continue; };
                spec = captured;
            }
            if spec.key == "batch3-bq026-attach" { spec.ops = vec![Op::BQ026Attach { observation: Some(crate::batch3_bats::RegionEntry { event_id: self.id(), turn: self.turn }) }]; }
            // XQ37's printed if-condition must hold at the actual entry event.
            // Do not defer this gate to declaration processing: earlier queued
            // rewards or responses must not retroactively qualify an entry.
            if source.card.definition == "XQ37"
                && spec.key == "entry-existing-influence"
                && !source
                    .region
                    .is_some_and(|r| self.regions[r].influence[self.team(actor)] > 0)
            {
                continue;
            }
            self.effects.push_back(Effect::Declare {
                declaration: Declaration {
                    actor,
                    source: source.clone(),
                    ability: spec.clone(),
                },
            });
        }
    }
    pub(crate) fn declare_trigger(&mut self,declaration:Declaration)->RuleResult<()> {
        if self.batch3_entry_order_declare(&declaration)? { return Ok(()); }
        self.declare_trigger_ordered(declaration)
    }
    pub(crate) fn declare_trigger_ordered(&mut self, declaration: Declaration) -> RuleResult<()> {
        self.validated_cross_card_declaration(&declaration)?;
        self.validate_cq10_declaration(&declaration)?;
        self.validate_bq084_declaration(&declaration)?;
        self.validate_bq001_declaration(&declaration)?;
        self.validate_anchor_declaration(&declaration)?;
        if crate::next20_blue_vampires::related(&declaration.ability) { self.validate_next20_blue_declaration(&declaration)?; }
        if crate::next20_black_wraiths::related(&declaration.ability) { self.validate_next20_black_declaration(&declaration)?; }
        self.validate_batch20_declaration(&declaration)?;
        self.validate_next_blue_gray_declaration(&declaration)?;
        self.validate_black_next20_declaration(&declaration)?;
        self.validate_next20_red_declaration(&declaration)?;
        self.validate_batch3_declaration(&declaration)?;
        self.validate_batch4_declaration(&declaration)?;
        self.validate_xq23_declaration(&declaration)?;
        if self.pack_a_forced_declare(&declaration)? || self.omega_forced_declare(&declaration)? || self.plan_forced_declare(&declaration)? || self.batch3_forced_declare(&declaration)? || self.batch20_declare_forced(&declaration)? || self.next_blue_gray_declare_forced(&declaration)? { return Ok(()); }
        if self.jc054_declare(declaration.clone())? { return Ok(()); }
        if self.jc041_declare(declaration.clone())? { return Ok(()); }
        if self.jc035_declare(declaration.clone())? { return Ok(()); }
        if self.jc046_declare(declaration.clone())? {
            return Ok(());
        }
        if self.bq030_declare(declaration.clone())? {
            return Ok(());
        }
        rules::validate_ability(&declaration.source.card.definition, &declaration.ability)?;
        let actor = declaration.actor;
        let spec = &declaration.ability;
        if self.players[actor].eliminated {
            self.decline_pregrave_declaration(&declaration)?;
            return Ok(());
        }
        if self.death_observer_limit_reached(&declaration) {
            return Ok(());
        }
        if (spec.event == Some(Event::HandDiscard) || declaration.source.card.definition == "JZ23") && self.resources(actor) < spec.costs.iter().filter_map(|c|
            if let Cost::Assets(n) = c { Some(*n) } else { None }).sum() {
            return Ok(());
        }
        if spec.requires_ready_source
            && !self
                .board(&declaration.source.card.id)
                .is_some_and(|(_, c)| !c.face_down && !c.exhausted)
        {
            return Ok(());
        }
        if (!self.wm044_trigger_available(&declaration) || !self.sentinel_pair_trigger_available(&declaration)) || !self.black_next20_trigger_available(&declaration) || !self.vampire_trigger_available(&declaration) { return Ok(()); }
        let title = format!(
            "{}：{}",
            crate::batch3_trap::source_label(&declaration.source),
            spec.label
        );
        let (options, stage) = if !spec.modes.is_empty() {
            (
                spec.modes
                    .iter()
                    .filter(|m| declaration.source.card.definition != "WM057" || spec.key != crate::import20_broker::WIN
                        || m.targets.first().is_none_or(|t| !self.binding_options(actor, &declaration.source, t).is_empty()))
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
            self.record_pregrave_choice()?;
        } else {
            self.decline_pregrave_declaration(&declaration)?;
        }
        Ok(())
    }
    pub(crate) fn choose_declaration(
        &mut self,
        mut declaration: Declaration,
        stage: DeclareChoice,
        selected: Vec<String>,
    ) -> RuleResult<()> {
        self.validated_cross_card_declaration(&declaration)?;
        self.validate_cq10_declaration(&declaration)?;
        self.validate_bq084_declaration(&declaration)?;
        self.validate_bq001_declaration(&declaration)?;
        self.validate_anchor_declaration(&declaration)?;
        if crate::next20_blue_vampires::related(&declaration.ability) { self.validate_next20_blue_declaration(&declaration)?; }
        if crate::next20_black_wraiths::related(&declaration.ability) { self.validate_next20_black_declaration(&declaration)?; }
        self.validate_batch20_declaration(&declaration)?;
        self.validate_next_blue_gray_declaration(&declaration)?;
        self.validate_black_next20_declaration(&declaration)?;
        self.validate_next20_red_declaration(&declaration)?;
        self.validate_batch3_declaration(&declaration)?;
        self.validate_batch4_declaration(&declaration)?;
        self.validate_xq23_declaration(&declaration)?;
        self.batch3_reject_optional(&declaration)?;
        self.omega_reject_optional(&declaration)?;
        self.batch20_reject_optional(&declaration)?;
        self.jc046_reject_optional_declaration(&declaration)?;
        self.jc041_reject_optional(&declaration)?;
        self.jc035_reject_optional(&declaration)?;
        self.jc054_reject_optional(&declaration)?;
        rules::validate_ability(&declaration.source.card.definition, &declaration.ability)?;
        if declaration.source.card.definition == crate::import20_carmen::ID && declaration.ability.key == crate::import20_carmen::ENTRY {
            if !matches!(stage, DeclareChoice::Target) { return Err("卡门须先声明原敌方附属".into()); }
            return self.cq24_begin_host(declaration, selected);
        }
        if crate::bq030::is_attachment_declaration(&declaration) {
            return self.bq030_choose(declaration, stage, selected);
        }
        let Some(selected) = selected.first() else {
            self.decline_pregrave_declaration(&declaration)?;
            self.wm058_decline(&declaration)?;
            return Ok(());
        };
        let targets = match stage {
            DeclareChoice::AttachmentOrder => return Err("结附触发排序来源无效".into()),
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
                    region_instance: Self::requires_region_instance(&declaration.source, &slot).then(|| selected.strip_prefix("region:").and_then(|s| s.parse::<usize>().ok()).and_then(|r| self.regions.get(r)).map(|r| r.card.id.clone())).flatten(),
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
            crate::batch3_trap::source_label(&declaration.source),
            declaration.ability.label
        );
        let paid = if declaration.ability.key == crate::batch4_vampires::ATTACH_DISCARD || declaration.source.card.definition == "JZ23" || (declaration.source.card.definition == "XG27" && declaration.ability.event == Some(Event::BlackCharacterDeathObserved))
            || declaration.ability.event == Some(Event::HandDiscard)
            || (declaration.source.card.definition == "JC031"
                && declaration.ability.event == Some(Event::CharacterDeathObserved))
            || (declaration.source.card.definition == "JC061"
                && declaration.ability.key == "enter-region-paid-hidden-destroy"
                && declaration.ability.event == Some(Event::EnterRegion)) {
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
        Ok(self.discard_hand_cards(holder, &[id.into()])?.remove(0))
    }
    pub(crate) fn discard_hand_cards(&mut self, holder: usize, ids: &[String]) -> RuleResult<Vec<PaidCost>> {
        let cards = ids.iter().map(|id| self.players[holder].hand.iter().find(|c| c.id == *id).cloned().ok_or_else(|| "弃牌引用失效".into())).collect::<RuleResult<Vec<_>>>()?;
        if ids.iter().collect::<std::collections::BTreeSet<_>>().len() != ids.len() { return Err("弃牌重复".into()); }
        let observers = self.capture_discard_observers(holder);
        let paid = ids.iter().map(|id| self.discard_hand_card_unobserved(holder, id)).collect::<RuleResult<Vec<_>>>()?;
        self.emit_discard_observed(holder, cards, observers);
        Ok(paid)
    }
    fn discard_hand_card_unobserved(&mut self, holder: usize, id: &str) -> RuleResult<PaidCost> {
        let c = self.remove_hand(holder, id)?;
        let owner = c.owner;
        let old_instance = c.id.clone();
        let mut source = self.source_snapshot(&c, None);
        source.card.controller = holder;
        let c = self.reset_zone_card(c);
        self.commit_owner_grave(c, None);
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
        if let Some((region, c)) = self.in_play(target) {
            self.remove_death_batch(vec![(target.into(), cause, self.source_snapshot(c, region))]);
        }
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
        ghoul_sources: Option<&[SourceSnapshot]>,
        cq18_observations: Option<Vec<Declaration>>,
    ) -> Option<crate::death_observers::CharacterDeathReceipt> {
        // Host departure can remove an attached control grant. Freeze before it;
        // an already captured simultaneous lethal-set snapshot remains authoritative.
        let snapshot = simultaneous_source.or_else(|| self.in_play(target)
            .map(|(region, c)| self.source_snapshot(c, region)));
        let was_renown = self.in_play(target).is_some_and(|(_, c)| self.has_renown(c));
        if let Some((_, c)) = self.leave_in_play_observed(target,cq18_observations) {
            let snapshot = snapshot.expect("board departure has a pre-departure snapshot");
            let controller = snapshot.card.controller;
            let character =
                !snapshot.card.face_down && card(&snapshot.card.definition).kind == "character";
            let owner = c.owner;
            let c = self.reset_zone_card(c);
            let deferred = character && snapshot.card.definition == "XG25";
            let grave_instance = if deferred { String::new() } else { c.id.clone() };
            if deferred { self.hold_pregrave_death(&snapshot, c); }
            else { self.commit_owner_grave(c, ghoul_sources); }
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
                return Some(crate::death_observers::CharacterDeathReceipt {
                    old_instance: target.into(),
                    grave_instance,
                    deferred_death: deferred.then(|| target.to_owned()),
                    owner,
                    was_renown,
                });
            }
        }
        None
    }
    fn take_entity(&mut self, id: &str) -> Option<Card> {
        if let Some((_, c)) = self.leave_in_play(id) {
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
                || frame.source.card.definition == "XQ20" && !self.xq20_pair_current(frame)
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
        self.validate_xq19_frame(&frame)?;
        self.validate_cq28_frame(&frame)?;
        self.validate_resource_frame(&frame)?;
        self.validate_exposure_frame(&frame)?;
        self.validated_cross_card_frame(&frame)?;
        self.validate_red_time_frame(&frame)?;
        self.validate_clock_frame(&frame)?;
        self.validate_blue_expansion_frame(&frame)?;
        self.validate_black_expansion_frame(&frame)?;
        self.validate_black_next20_frame(&frame)?;
        self.validate_wm056_frame(&frame)?;
        self.validate_wm062_frame(&frame)?;
        self.validate_wm023_frame(&frame)?;
        self.validate_xq25_frame(&frame)?;
        self.validate_jz46_frame(&frame)?;
        self.validate_wm038_frame(&frame)?;
        self.validate_jz35_frame(&frame)?;
        self.validate_next20_black_play_frame(&frame)?;
        if crate::next20_black_wraiths::frame_related(&frame) { self.validate_next20_black_program_frame(&frame)?; }
        self.validate_cq10_frame(&frame)?;
        self.validate_bq084_frame(&frame)?;
        self.validate_xq35_frame(&frame)?;
        self.validate_bq001_frame(&frame)?;
        self.validate_anchor_frame(&frame)?;
        self.validate_next20_blue_play_frame(&frame)?;
        if crate::next20_blue_vampires::frame_related(&frame) { self.validate_next20_blue_frame(&frame)?; }
        self.validate_batch20_frame(&frame)?;
        self.validate_next_blue_gray_frame(&frame)?;
        self.validate_next20_red_frame(&frame)?;
        self.validate_batch3_frame(&frame)?;
        self.validate_batch4_frame(&frame)?;
        self.validate_jc052_frame(&frame)?;
        self.validate_jc046_frame(&frame)?;
        self.validate_jc054_frame(&frame)?;
        self.validate_jc095_frame(&frame)?;
        self.validate_bq035_frame(&frame)?;
        self.validate_gray_expansion_frame(&frame)?;
        self.validate_gray_lock_frame(&frame)?;
        self.validate_jz22_frame(&frame)?;
        self.validate_jc050_frame(&frame)?;
        self.validate_xq27_frame(&frame)?;
        self.validate_xq37_frame(&frame)?;
        self.validate_jc044_frame(&frame)?;
        self.validate_jc066_frame(&frame)?;
        self.validate_jc041_frame(&frame)?;
        self.validate_xq23_frame(&frame)?;
        self.validate_jc035_frame(&frame)?;
        if !self.accept_frame_guard(&mut frame) {
            self.cancel_pregrave_frame(&frame)?;
            self.jc035_cancel_guarded_frame(&frame)?;
            return Ok(());
        }
        while frame.cursor < frame.steps.len() && self.status == "playing" {
            let before_deaths = self.death_instances();
            let step = frame.steps[frame.cursor].clone();
            frame.cursor += 1;
            match step.op {
                Op::Next20Wraith(program) => match program {
                    crate::next20_black_wraiths::WraithProgram::TigerSearch { .. } => return self.cq25_search_start(frame),
                    crate::next20_black_wraiths::WraithProgram::GhostMove { .. } => self.xg25_move(&frame)?,
                    crate::next20_black_wraiths::WraithProgram::GhostSeal => self.resolve_xg25_death(&frame)?,
                },
                Op::Next20Blue(program) => match program {
                    crate::next20_blue_vampires::BlueProgram::RandomDiscard => self.bq033_random_discard(&frame)?,
                    crate::next20_blue_vampires::BlueProgram::EntryFeed => { let recipient = frame.source.card.id.clone(); return self.bq034_feed_start(frame, recipient); },
                    crate::next20_blue_vampires::BlueProgram::Counter { .. } => return self.bq034_counter_start(frame),
                    crate::next20_blue_vampires::BlueProgram::ShownFeed { .. } => return Err("展示后饮血须沿原对抗后置续体".into()),
                },
                Op::XQ19FromHand { .. } => self.xq19_place_hand(&frame)?,
                Op::CQ28Mode {mode,slot} => self.cq28_resolve_mode(&frame,mode,slot)?,
                Op::CQ15DestroySearch { .. } => return self.cq15_destroy_search(frame),
                Op::BQ039BattleDiscard { .. } => return self.batch20_combat_discard_start(frame),
                Op::CQ24TakeAttachment { .. } => self.cq24_take_attachment(&frame)?,
                Op::WM064RevealCriminal => self.wm064_reveal_criminal(&frame)?,
                Op::XG20ChooseNumber => return self.xg20_number_start(frame),
                Op::PeteTax { .. } => return self.pete_tax_start(frame),
                Op::PeteFromAsset => self.pete_place_from_asset(&frame)?,
                Op::CQ19AddCharge => self.cq19_add_charge(&frame)?,
                Op::CQ19Fire => self.cq19_fire(&frame)?,
                Op::CQ18EntryLocks => self.cq18_entry_locks(&frame)?,
                Op::CQ18LeaveDraw { .. } => self.cq18_leave_draw(&frame)?,
                Op::IndustryStepResource => self.industry_gain_resource(&frame)?,
                Op::IndustryMill { .. } => {self.validate_industry_frame(&frame)?;self.mill_cards_with_cause(frame.actor,1,Some(&frame))?;},
                Op::IndustryInvestigation { .. } => self.industry_resolve_bonus(&frame)?,
                Op::XG13EntryTime { .. } => self.xg13_add_time(&frame)?,
                Op::BQ064TakeSamples => self.import20_take_samples(frame.actor)?,
                Op::XG11RevealRaid { .. } => self.xg11_raid_resolve(&frame)?,
                Op::BQ053RemoveLossControl => {
                    let n = &mut self.players[frame.actor].society_zone.loss_control;
                    *n = n.saturating_sub(1);
                },
                Op::WM029TurnAura => self.wm029_resolve(&frame)?,
                Op::JZ56ChooseUndead => self.jz56_choose_start(frame.clone())?,
                Op::WM058Tax { .. } => { self.wm058_demand(frame)?; return Ok(()); },
                Op::WM058RemoveEnemyInfluence => self.wm058_reveal(&frame),
                Op::XG17AdvancePlan { .. } => self.xg17_advance_target(&frame),
                Op::XG18RenownPlay { .. } => self.omega_resolve(&frame)?,
                Op::XG16EntryTime => self.xg16_entry_time(&frame),
                Op::XG16PrepareTime { .. } => self.jc035_resolve(&frame)?,
                Op::BQ038ContestBonus { .. } => return self.bq038_resolve_bonus(frame),
                Op::BQ027ShownFeed { .. } | Op::BQ031WinFeed { .. } | Op::BQ027FeedTarget | Op::BQ032RevealFeed | Op::BQ038RaidFeed { .. } => return self.vampire_feed_start(frame),
                Op::BQ031AttachmentDiscard { .. } => return self.bq031_discard(frame),
                Op::BQ036TopSix => return self.bq036_search_start(frame),
                Op::MysticContest { actor_bonus } => {
                    if frame.source.card.definition=="WM037" {return self.wm037_offer_contest(frame);}
                    return self.mystic_contest_start(frame,actor_bonus);
                },
                Op::WM037ShownTime { .. } => return Err("戴森展示后置只能经原对抗待选执行".into()),
                Op::MysticWin { .. } => {},
                Op::BQ081GraveBottomDestroy => return self.bq081_entry_start(frame),
                Op::WM057Win { .. } => {},
                Op::WM057Repress => return self.wm057_repress_start(frame),
                Op::WM057Inspect => return self.wm057_inspect_start(frame),
                Op::WM025MysticInfluence { .. } => self.wm025_influence(frame.actor),
                Op::WM024RevealRaid { .. } => self.wm024_raid_resolve(&frame)?,
                Op::MysticDestroyedInfluence => self.pack_a_destroyed_influence(&frame),
                Op::XQ21CopyMarker => return self.xq21_marker_start(frame),
                Op::XQ20MoveMarkers => self.xq20_resolve(&frame)?,
                Op::LC16PrepareSearch { .. } => self.jc035_resolve(&frame)?,
                Op::JZ51TopEightCorpses => return self.jz51_search_start(frame),
                Op::CQ10Entry { .. } => return self.cq10_run(frame),
                Op::CQ10Repeat => return self.cq10_repeat(frame),
                Op::CQ10Prepare { .. } => self.jc035_resolve(&frame)?,
                Op::BQ084AnchorEntry => return self.bq084_start(frame),
                Op::XQ35GraveReturn => self.xq35_return(&frame)?,
                Op::JZ51GraveEntry => return self.jz51_grave_entry(frame),
                Op::JZ36TimeAndDestroyTargetRegion => self.jz36_resolve(&frame)?,
                Op::JZ40TimeAndDestroyHost => self.jz40_resolve(&frame),
                Op::PlanPlayed { .. } => {},
                Op::PlanAdvance => self.plan_advance(&frame),
                Op::PlanComplete => self.plan_complete(&frame)?,
                Op::XG21RenownSearch => return self.phantom_search_start(frame),
                Op::NextBlueGraySearch | Op::JZ28DiscardSearch { .. } => return self.next_blue_gray_search_start(frame),
                Op::JZ23RevealReward { observation } => self.jz23_reward(observation.as_ref().ok_or("缺少现身观察")?),
                Op::CQ08Prepare { .. } => { self.jc035_resolve(&frame)?; return self.next_blue_gray_search_start(frame); },
                Op::BQ059EntryShield => { if let Some(c) = self.board_mut(&frame.source.card.id).filter(|c| !c.face_down) { c.shield = c.shield.saturating_add(2); } },
                Op::BlackDeckToGraveyard { .. } => self.black_deck_recovery(&frame)?,
                Op::BlackDeathRecovery { observation } => self.black_death_recovery(&frame, observation.as_ref().ok_or("黑色死亡事件缺少快照")?)?,
                Op::XQ39GraveCharacterToDeckTop => self.xq39_grave_to_top(&frame)?,
                Op::JC051Badge => return self.jc051_search(frame),
                Op::JC055Discount => self.jc055_discount(&frame)?,
                Op::JZ32Hourglass => return self.jz32_search(frame),
                Op::JZ34SacrificeDestroy => return self.jz34_enter(frame),
                Op::JZ34DeathInfluence { .. } => { if let Some(r) = frame.targets.first().filter(|t| self.valid_bound_target(frame.actor, &frame.source, t)).and_then(|t| t.id.strip_prefix("region:")).and_then(|s| s.parse::<usize>().ok()) { self.place_influence(frame.actor,r,1); } },
                Op::JZ41DamageWound { .. } => self.jz41_wound(&frame),
                Op::WM039Analysis => self.wm039_analysis(&frame),
                Op::WM040Heal => self.wm040_heal(&frame),
                Op::WM040Repress => return self.wm040_repress_start(frame),
                Op::WM042Exposure { add } => self.wm042_exposure(add),
                Op::JC065SacrificeHidden => return self.jc065_sacrifice_start(frame),
                Op::JC065RepressOne => return self.batch20_repress_start(frame),
                Op::JZ29ReturnOnVampireReveal { .. } => {
                    if let Some((_, c)) = self.leave_in_play(&frame.source.card.id) {
                        let c = self.reset_zone_card(c); self.players[c.owner].hand.push(c);
                    }
                }
                Op::JC034CombatDiscard { .. } => return self.batch20_combat_discard_start(frame),
                Op::JC040RepressDiscard { .. } => return self.batch20_repress_start(frame),
                Op::JC038GrantedRaid { grants } => self.jc038_raid_resolve(&frame, &grants)?,
                Op::Batch20SearchEntry => return self.batch20_search_start(frame),
                Op::CQ16LockLocalCharacter => {
                    if let Some(c) = frame.targets.first().and_then(|t| self.board_mut(&t.id)) { c.lock_markers = c.lock_markers.saturating_add(1); }
                }
                Op::BQ080HideForAttachment { .. } => { self.hide_in_place(&frame.source.card.id); }
                Op::JZ57MillAndOptionalDraw => return self.jz57_resolve(frame),
                Op::BQ054LockTarget => {
                    if let Some(c) = frame.targets.first().and_then(|t| self.board_mut(&t.id)) {
                        c.lock_markers = c.lock_markers.saturating_add(1);
                    }
                }
                Op::JZ26Trap { .. } => self.jz26_resolve(&frame)?,
                Op::BQ026Attach { .. } => self.bq026_attach(&frame),
                Op::BQ029SelfFeed { .. } => return self.bq029_feed_start(frame),
                Op::BQ037FeedPair { .. } => return self.bq037_feed_start(frame),
                Op::BQ037RevealVampire => return self.bq037_reveal_start(frame),
                Op::XQ26Name => return self.xq26_name_start(frame),
                Op::XQ26Wanted { observation } => self.xq26_wanted_resolve(&frame,observation.as_ref().ok_or("缺少通缉进场观察")?),
                Op::XG14DisasterInfluence { .. } => self.xg14_influence(&frame),
                Op::XG15DestroyOriginalRegion => self.xg15_destroy_region(&frame),
                Op::JZ38RepeatDeath { .. } => self.statue_repeat_death(&frame)?,
                Op::BQ041BloodDemand { .. } => return self.bq041_demand(frame),
                Op::WM060RevealInfluence => self.wm060_reveal_influence(&frame),
                Op::WM060CriminalObservation { .. } => {},
                Op::WM060RepressTwo => return self.wm060_repress_start(frame),
                Op::XG12Reattach => self.reattach_source(&frame.source.card.id, &frame.targets[0].id),
                Op::XG12GrantedMobility { .. } => {
                    if let Some(region) = self.frame_region(&frame, RegionRef::Target(0)) {
                        self.move_on_board_instance(&frame.source.card.id, region);
                    }
                },
                Op::JZ39EnemySacrifice => return self.jz39_enemy_sacrifice(frame),
                Op::CQ07EnterMoon => {
                    if self.players[frame.actor].society_zone.secrecy == 0 { self.add_society_markers(frame.actor,0,1); }
                },
                Op::CQ07SpellTargeted { .. } => self.draw(frame.actor,1)?,
                Op::WM041PublicPlay { .. } => { self.add_society_markers(frame.actor,1,0); self.draw(frame.actor,1)?; },
                Op::BQ079CorpseIcons { .. } => self.bq079_reward(&frame.source),
                Op::WM035HostDeathDraw { .. } => self.draw(frame.actor, 1)?,
                Op::XQ22Orientation => return self.xq22_orientation_start(frame),
                Op::WM044Inspect { .. } => { self.wm044_inspect_start(frame)?; return Ok(()); },
                Op::WM044SentinelLock => self.wm044_lock(&frame),
                Op::JC062Inspect => { if self.jc062_inspect_start(frame.clone())? { return Ok(()); } },
                Op::JZ25SealRandomHand => self.jz25_seal_random_hand(&frame)?,
                Op::XG28GraveReturn => self.xg28_return(&frame)?,
                Op::BQ089GraveEntry => self.bq089_reanimate(&frame)?,
                Op::Batch3Prepare { .. } => self.jc035_resolve(&frame)?,
                Op::XG26AttachmentBarrier => self.xg26_grant_barrier(&frame)?,
                Op::WM033ReturnSpellOrDisaster => return self.wm033_start(frame),
                Op::JC043DestroyAttachment | Op::BQ042DestroyAttachment => {
                    if let Some(t) = frame.targets.first() { self.destroy_from_frame(&frame, &t.id); }
                }
                Op::JZ33ScorchDestroyedRegions => self.jz33_scorch()?,
                Op::JZ37DestroyWorld => self.jz37_destroy_world(frame.actor),
                Op::JC052DestroyOtherOriginalRegionCharacters => self.jc052_destroy_others(&frame),
                Op::JC095ConvertEnemyInfluenceInOriginalRegion => self.jc095_convert_enemy_influence(&frame),
                Op::JZ35SacrificeReturnControlledHosts => self.jz35_resolve(&frame)?,
                Op::WM038AddOneTimeToOriginalHost => self.wm038_add_time(&frame),
                Op::WM023PutUniqueMageOrVampireFromHand => return self.wm023_start(frame),
                Op::WM056MoveOneAdjacentInfluence => return self.wm056_start(frame),
                Op::WM062HideCriminalThenOptionalAdjacentMove => return self.wm062_start(frame),
                ref op @ (Op::JC054ConcealBurst { .. } | Op::JC054EndHide { .. }) => self.jc054_resolve(&frame, op)?,
                Op::ClockAddTime => self.clock_add_time(&frame),
                Op::ClockSweep => self.clock_sweep(&frame)?,
                Op::XQ18AddOneTimeToTarget => self.xq18_add_one_time(&frame),
                Op::XQ18ChooseTimeCarrier => return self.xq18_choose_carrier(frame),
                Op::JZ30AddOneTimeToOriginalSource => self.jz30_add_one_time(&frame),
                Op::JZ30ForecastFrozenTime => {
                    if self.jz30_forecast_frozen_time(&frame)? { return Ok(()); }
                }
                Op::BQ028InspectTargetHandAttachments => return self.bq028_inspect_start(frame),
                Op::JZ46InspectHiddenCriminal => return self.jz46_inspect_start(frame),
                Op::JC057InspectTargetHand => return self.jc057_inspect_start(frame),
                Op::JC066SealLocalTargetOnSource => self.jc066_seal_local_target(&frame)?,
                Op::XQ23ReturnAssetThenBecomeAsset => self.xq23_return_and_become_asset(&frame)?,
                Op::WM028Prepare { .. } | Op::JC035PreparedInfluence { .. } | Op::Batch20Prepare { .. } | Op::JC055PrepareTime { .. } | Op::BQ085Prepare { .. } => {
                    self.jc035_resolve(&frame)?;
                    if frame.source.card.definition == "WM028" { return self.mystic_contest_start(frame, 0); }
                },
                Op::WM034AddOneTime => { if let Some(c) = self.board_mut(&frame.source.card.id).filter(|c| !c.face_down) { c.time_markers = c.time_markers.saturating_add(1); } }
                Op::WM034DestroyOtherCharacters => self.wm034_destroy_others(&frame),
                Op::JC041RevealOriginalHidden => self.jc041_reveal_hidden(&frame)?,
                Op::BQ030AttachmentDraw { .. } => self.bq030_draw_one(&frame)?,
                Op::BQ078ReturnNamelessCorpse => return self.bq078_return_start(frame),
                Op::JC087HideTargetThenInfluence => self.jc087_hide_then_influence(&frame)?,
                Op::JZ45LockLocalTarget => self.jz45_lock_local_target(&frame)?,
                Op::XQ24HideCheapCharactersInSourceRegion => self.xq24_hide_cheap_characters(&frame),
                Op::XQ44SearchDreamSealOnTarget => return self.deck_seal_search_start(frame, true),
                Op::JZ02SearchSpaceSpellSealOnSource => return self.deck_seal_search_start(frame, false),
                Op::JZ50SearchDeathToGraveyard => return self.jz50_search_start(frame),
                Op::BQ104SearchEmployeeHiddenInSourceRegion => return self.entry_search_start(frame, true),
                Op::XQ48SearchPassersIntoSourceRegion => return self.entry_search_start(frame, false),
                Op::BQ001SealStarHandOnSociety => return self.bq001_start(frame),
                Op::SealOneActorHandCardOnTarget => return self.choose_hand_seal(frame),
                Op::DestroyTargetIfSealed => {
                    let host = &frame.targets.first().ok_or("缺少封印载体目标")?.id;
                    if self.sealed_cards.iter().any(|s| s.host_id == *host) {
                        self.destroy_from_frame(&frame, host);
                    }
                }
                Op::JC032TopSixVampireHidden => return self.jc032_start(frame),
                Op::JZ24LocalSacrificeSnapshot => return self.jz24_start(frame),
                Op::JZ22LowHandInfluenceInSourceRegion => self.jz22_low_hand_influence(&frame),
                Op::XQ37EntryInfluenceIfPresent => self.xq37_entry_influence(&frame),
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
                        grants_barrier: false, prevents_damage: false,
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
                Op::JC044AddOneTimeToOriginalSource => self.jc044_add_time(&frame)?,
                Op::RepressOpponentOne { slot } => {
                    let seat = self.frame_player(&frame, step.context, PlayerRef::Target(slot))?;
                    let team = self.team(seat);
                    let regions: Vec<_> = self
                        .regions
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| !r.vacant && r.influence[team] > 0)
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
                    self.place_wounds(BTreeMap::from([(id, amount)]))?;
                }
                Op::DamageTarget { slot, amount } => {
                    let target = frame.targets.get(slot).ok_or("缺少伤害目标")?;
                    self.damage(std::collections::BTreeMap::from([(
                        target.id.clone(),
                        amount,
                    )]))?;
                }
                Op::WM043Lock => { if let Some(t) = frame.targets.first() { self.wm043_lock(&t.id); } },
                Op::WM043ShownLock { .. } => return Err("精准打击展示后置只由当前选择消费".into()),
                Op::SentinelLock => {
                    if let Some(c) = frame.targets.first().and_then(|t| self.board_mut(&t.id)) { c.lock_markers = c.lock_markers.saturating_add(1); }
                }
                Op::SentinelMove => {
                    let mover = &frame.targets[0].id;
                    if let Some(to) = self.board(&frame.targets[1].id).map(|(r, _)| r) { self.move_on_board_instance(mover, to); }
                }
                Op::JC069LockTarget => {
                    let target = frame.targets.first().ok_or("缺少锁定目标")?;
                    if let Some(c) = self.board_mut(&target.id) {
                        c.lock_markers = c.lock_markers.saturating_add(1);
                    }
                }
                Op::JZ43LockOrDamageLocalTarget => {
                    let target = frame.targets.first().ok_or("缺少锁定目标")?;
                    if self.board(&target.id).is_some_and(|(_, c)| self.effective_marker_count(c, crate::batch4_markers::MarkerKind::Lock) > 0) {
                        self.damage(BTreeMap::from([(target.id.clone(), 1)]))?;
                    } else if let Some(c) = self.board_mut(&target.id) {
                        c.lock_markers = c.lock_markers.saturating_add(1);
                    }
                }
                Op::JC069ChaseLockedTarget => {
                    let target = frame.targets.first().ok_or("缺少追逐目标")?;
                    let destination = self.board(&target.id).map(|(r, _)| r);
                    // Self-reference follows the original board instance. A
                    // leave/flip/reentry gives a new ID and cannot inherit this.
                    if let Some((from, c)) = self.board(&frame.source.card.id)
                        .filter(|(_, c)| !c.face_down && card(&c.definition).kind == "character")
                    {
                        let id = c.id.clone();
                        if let Some(to) = destination.filter(|to| *to != from) {
                            self.move_on_board_instance(&id, to);
                        }
                        self.turn_attribute_modifiers.push(TurnAttributeModifier {
                            target_instance: id, defense_bonus: 0, kill_bonus: 0,
                            grants_retreat: false, printed_defense_override: None,
                            ordinary_icons: Icons { investigation: 1, ..Default::default() },
                            grants_renown: false, grants_barrier: false, prevents_damage: false, expires_turn: self.turn,
                        });
                    }
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
                        grants_barrier: false, prevents_damage: true,
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
                            grants_barrier: false, prevents_damage: false,
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
                                self.current_magic_count(c) > 0
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
                            grants_barrier: false, prevents_damage: false,
                            expires_turn: self.turn,
                        });
                    }
                }
                Op::PlaceOneInfluenceInSourceRegion => {
                    if let Some(region) = frame.source.region.filter(|&region| {
                        self.regions.get(region).is_some_and(|r|
                            !r.vacant && frame.source.source_region_instance.as_ref() == Some(&r.card.id))
                    }) {
                        self.place_influence(frame.actor, region, 1);
                    }
                }
                Op::JC045AddOneTimeToOriginalSource => {
                    if let Some(live) = self.board_mut(&frame.source.card.id)
                        .filter(|c| c.definition == "JC045" && !c.face_down)
                    {
                        live.time_markers = live.time_markers.saturating_add(1);
                    }
                }
                Op::JC090PlaceOneInfluenceInOriginalAttachedRegion => {
                    if let Some(region) = frame.source.region.filter(|&r| self.regions.get(r)
                        .is_some_and(|region| !region.vacant && frame.source.attachment_host_instance.as_ref() == Some(&region.card.id)
                            && frame.source.source_region_instance.as_ref() == Some(&region.card.id)))
                    {
                        self.place_influence(frame.actor, region, 1);
                    }
                }
                Op::JC031DiscardForObservedDeath => {
                    let seat = self.frame_player(&frame, step.context, PlayerRef::Target(0))?;
                    if !self.players[seat].eliminated && !self.players[seat].hand.is_empty() {
                        if frame.source.observed_death.as_ref().is_some_and(|d| d.vampire) {
                            let i = self.random_below(self.players[seat].hand.len());
                            let id = self.players[seat].hand[i].id.clone();
                            self.discard_effect_cards(seat, &[id], &frame, false)?;
                        } else {
                            let options = self.players[seat].hand.iter()
                                .map(|c| self.option(c, seat, None, None)).collect();
                            self.choice(seat, "discard", "弃 1 张手牌".into(), options, 1, 1, None,
                                ChoiceResolution::Frame { frame: Box::new(frame),
                                    choice: FrameChoice::Discard { seat, redraw: false } });
                            return Ok(());
                        }
                    }
                }
                Op::PlaceInfluence {
                    region_instance,
                    amount,
                } => {
                    if let Some(region) = frame.source.region.filter(|&region| {
                        self.regions
                            .get(region)
                            .is_some_and(|r| !r.vacant && r.card.id == region_instance)
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
                    if matches!(destination, Destination::HiddenInChosenRegion)
                        && frame.chosen_region.is_none_or(|r| !self.region_live(r)) {
                        continue;
                    }
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
                                    self.apply_hidden_entry_slow(&mut c);
                                    self.regions[r].cards.push(c);
                                }
                            }
                        }
                    }
                }
                Op::Hide(entity) => {
                    if let Some(id) = Self::frame_entity(&frame, entity) {
                        self.hide_in_place(id);
                    }
                }
                Op::MoveOnBoard { entity, region } => {
                    if let (Some(id), Some(region)) = (
                        Self::frame_entity(&frame, entity),
                        self.frame_region(&frame, region),
                    ) {
                        self.move_on_board_instance(id, region);
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
                        self.note(format!("资产{}：消灭", card(&c.definition).name));
                        let c = self.reset_zone_card(c);
                        self.commit_owner_grave(c, None);
                    } else {
                        self.destroy_from_frame(&frame, &id);
                    }
                }
                Op::XQ27DestroyAllHidden => {
                    let game = &*self;
                    let deaths = game.regions.iter().enumerate().flat_map(|(r, region)|
                        region.cards.iter().filter(|c| c.face_down)
                            .map(move |c| (c.id.clone(), RemovalCause::Destroy, game.source_snapshot(c, Some(r)))))
                        .collect();
                    self.destroy_wave_as(frame.actor,deaths);
                }
                Op::JC050DestroyChosenRegionCharacters => {
                    let region = frame.chosen_region.ok_or("JC050缺少已选择地区")?;
                    let deaths = self.regions.get(region).ok_or("JC050地区已失效")?
                        .cards.iter().filter(|c| c.face_down || card(&c.definition).kind == "character")
                        .map(|c| (c.id.clone(), RemovalCause::Destroy, self.source_snapshot(c, Some(region))))
                        .collect();
                    self.destroy_wave_as(frame.actor,deaths);
                }
                Op::Destroy(entity) => {
                    if let Some(id) = Self::frame_entity(&frame, entity) {
                        self.destroy_from_frame(&frame, id);
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
                    self.mill_cards_with_cause(seat, count, Some(&frame))?;
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
                                if frame.ability_key == crate::jc041::REVEAL { crate::jc041::DISCARD_TITLE.into() }
                                else { format!("弃 {n} 张手牌") }
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
            if self.pending.is_some() {
                if frame.cursor < frame.steps.len() { self.effects.push_front(Effect::Frame { frame: Box::new(frame) }); }
                return Ok(());
            }
            if frame.cursor < frame.steps.len() && self.defer_death_caller(frame.clone(), crate::death_continuation::DeathCallerKind::Frame, &before_deaths) { return Ok(()); }
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
        let before_deaths = self.death_instances();
        self.validate_jc046_choice(&frame, &choice, chooser, &a, &selected)?;
        self.validate_jc041_choice(&frame, &choice, chooser, &a, &selected)?;
        self.wm060_choice_payload(&frame, chooser, &a)?;
        self.bq041_choice_payload(&frame, chooser, &a)?;
        self.statue_choice_payload(&frame, &choice, chooser, &a)?;
        match choice {
            FrameChoice::Next20Wraith(crate::next20_black_wraiths::WraithChoice::TigerSearch) => return self.cq25_search_finish(&frame,chooser,&selected),
            FrameChoice::Next20Blue(choice) => return match choice {
                crate::next20_blue_vampires::BlueChoice::ShownTarget => self.bq034_shown_target(*frame,chooser,&selected),
                crate::next20_blue_vampires::BlueChoice::Blood { recipient } => self.bq034_feed_finish(&frame,&recipient,chooser,&selected),
                crate::next20_blue_vampires::BlueChoice::SacrificeAttachment => self.bq034_counter_finish(&frame,chooser,&selected),
            },
            FrameChoice::CQ15Search { search } => return self.cq15_search_finish(&frame,&search,chooser,&a,&selected),
            FrameChoice::XG20Number => return self.xg20_number_finish(&frame,chooser,&a),
            FrameChoice::PetePay => return self.pete_pay_finish(*frame,chooser,&a,&selected),
            FrameChoice::PeteDraw => return self.pete_draw_finish(&frame,chooser,&a,&selected),
            FrameChoice::WM037Contest => return self.wm037_choose_contest(*frame,chooser,&a,&selected),
            FrameChoice::WM037TimeTarget => return self.wm037_choose_time(&frame,chooser,&a,&selected),
            FrameChoice::MysticCommit { contest_id } => return self.mystic_commit(&frame, &contest_id, chooser, &selected),
            FrameChoice::MysticReward { contest_id } => return self.mystic_reward(&frame, &contest_id, chooser, &selected),
            FrameChoice::WM058Payment => return self.wm058_payment(*frame, chooser, &a, &selected),
            FrameChoice::BQ041Payment => return self.bq041_pay_complete(*frame, chooser, &a, &selected),
            FrameChoice::WM060Repress { regions } => return self.wm060_repress_complete(&frame, chooser, &a, &regions, &selected),
            FrameChoice::JZ39EnemySacrifice => return self.jz39_finish_sacrifice(&frame, chooser, &a, &selected),
            FrameChoice::JZ56Undead => return self.jz56_choose_finish(&frame, chooser, &a, &selected),
            FrameChoice::BQ027ShownTarget => return self.bq027_choose_shown_target(*frame, chooser, &selected),
            FrameChoice::VampireBlood => return self.vampire_feed_finish(&frame, chooser, &selected),
            FrameChoice::BQ036TopSix { inspected_ids } => return self.bq036_search_finish(&frame, &inspected_ids, &selected),
            FrameChoice::BloodFeed { state } => return self.blood_feed_complete(&frame,state,chooser,&a,&selected),
            FrameChoice::BQ037Reveal => return self.bq037_reveal_complete(&frame,chooser,&a,&selected),
            FrameChoice::XQ26Name => return self.xq26_name_complete(&frame, chooser, &a, &selected),
            FrameChoice::BQ079Upkeep => return self.bq079_upkeep_complete(&frame, chooser, &a, &selected),
            FrameChoice::JC064Upkeep => return self.jc064_upkeep_complete(&frame, chooser, &a, &selected),
            FrameChoice::BQ081GraveCharacter => self.bq081_entry_complete(&frame, chooser, &selected)?,
            FrameChoice::WM057Opponent => return self.wm057_repress_choose(*frame, chooser, &selected),
            FrameChoice::WM057InspectMay => return self.wm057_inspect_choose(*frame, chooser, &selected),
            FrameChoice::WM057Inspection { .. } => self.wm057_inspect_complete(&frame, chooser, &selected)?,
            FrameChoice::XQ22Orientation => self.xq22_orientation_complete(&frame, chooser, &a, &selected)?,
            FrameChoice::WM043ShownTarget => return self.wm043_choose_shown(&frame, chooser, &a, &selected),
            FrameChoice::WM044Inspection { inspected } => self.wm044_inspect_complete(&frame, chooser, &a, &selected, &inspected)?,
            FrameChoice::JC062Inspection { inspected } => self.jc062_inspect_complete(&frame, chooser, &a, &selected, &inspected)?,
            FrameChoice::WM033GraveSpell => return self.wm033_finish(&frame, chooser, &a, &selected),
            FrameChoice::JC051Badge => return self.jc051_choose_badge(&frame,&selected),
            FrameChoice::JC051Host { badge } => return self.jc051_finish_attach(&frame,&badge,selected.first().ok_or("请选择徽章宿主")?),
            FrameChoice::JZ32Hourglass => return self.jz32_finish(&frame,&selected),
            FrameChoice::JZ34Sacrifice => return self.jz34_finish_sacrifice(&frame,&selected),
            FrameChoice::WM040Repress { seat, regions, .. } => return self.wm040_repress_complete(seat, &regions, &selected),
            FrameChoice::JC065Sacrifice { seat, region, region_instance } => return self.jc065_sacrifice_complete(*frame, chooser, seat, region, &region_instance, &selected),
            FrameChoice::Batch20Repress { seat, regions } => return self.batch20_repress_complete(&frame, chooser, seat, &regions, &selected),
            FrameChoice::NextBlueGraySearch { inspected_ids } => return self.next_blue_gray_search_complete(&frame, chooser, &inspected_ids, &selected),
            FrameChoice::Batch20SearchEntry => return self.batch20_search_complete(&frame, chooser, &selected),
            FrameChoice::Batch20OptionalDraw => return self.batch20_optional_draw(&frame, chooser, &selected),
            FrameChoice::JC068Upkeep => return self.jc068_upkeep_complete(&frame, chooser, &selected),
            FrameChoice::WM062AdjacentMove { before, hidden, baseline, region, region_instance, destinations } => {
                self.wm062_complete(&frame, chooser, &a, &selected, &before, &hidden, baseline.as_ref(), region, &region_instance, &destinations)?;
            }
            FrameChoice::WM023HandEntry => self.wm023_complete(&frame, chooser, &selected)?,
            FrameChoice::XQ18TimeCarrier { target_instance } => {
                self.xq18_remove_from_carrier(&frame, chooser, &target_instance, &selected)?;
            }
            FrameChoice::JZ46InspectHidden { inspected } => {
                self.jz46_inspect_complete(&frame, inspected, chooser, &a, &selected)?;
            }
            FrameChoice::JC057InspectHand { seat, inspected } => {
                self.jc057_inspect_complete(&frame, seat, &inspected, chooser, &a, &selected)?;
            }
            FrameChoice::BQ028InspectAttachments { seat, inspected } => {
                self.bq028_inspect_complete(&frame, seat, inspected, &selected)?;
            }
            FrameChoice::BQ078CorpseReturn => {
                self.bq078_return_complete(&frame, chooser, &selected)?;
            }
            FrameChoice::XQ44DreamSealSearch | FrameChoice::JZ02SpaceSealSearch => {
                if chooser != frame.actor { return Err("牌库封印检索选择者无效".into()); }
                self.deck_seal_search_complete(&frame, &selected, matches!(choice, FrameChoice::XQ44DreamSealSearch))?;
            }
            FrameChoice::LC16Search => return self.lc16_search_complete(&frame, chooser, &a, &selected),
            FrameChoice::JZ50DeathSearch => {
                if chooser != frame.actor {
                    return Err("墓穴食尸鬼检索的选择者无效".into());
                }
                self.jz50_search_complete(&frame, &selected)?;
            }
            FrameChoice::XG21RenownSearch => {
                self.phantom_search_complete(&frame, &selected)?;
                return Ok(());
            }
            FrameChoice::BQ104EmployeeSearch | FrameChoice::XQ48PasserSearch => {
                if chooser != frame.actor { return Err("进场检索的选择者无效".into()); }
                self.entry_search_complete(&frame, &selected, matches!(choice, FrameChoice::BQ104EmployeeSearch))?;
            }
            FrameChoice::CQ10Discard => return self.cq10_discard(*frame,chooser,&selected),
            FrameChoice::BQ084Anchor => return self.bq084_finish(*frame,chooser,&a,&selected),
            FrameChoice::BQ001StarHand => return self.bq001_complete(*frame,chooser,&a,&selected),
            FrameChoice::HandSeal { seat, host_id } => {
                if chooser != seat || frame.actor != seat
                    || frame.targets.first().is_none_or(|t| t.id != host_id) {
                    return Err("封印手牌的选择者无效".into());
                }
                let id = selected.first().ok_or("必须选择一张手牌封印")?;
                self.seal_hand_card(seat, id, &host_id)?;
            }
            FrameChoice::XQ21Marker { kinds } => self.xq21_marker_finish(&frame, &kinds, &selected)?,
            FrameChoice::JZ51TopEight { inspected_ids } => self.jz51_search_finish(&frame,&inspected_ids,&selected)?,
            FrameChoice::JZ51EntryRegion { regions } => self.jz51_region_finish(&frame,&regions,&selected)?,
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
            FrameChoice::BQ085Upkeep => { self.bq085_upkeep_complete(&frame, &selected)?; return Ok(()); }
            FrameChoice::Discard { seat, redraw } => {
                if frame.cursor < frame.steps.len() { self.effects.push_front(Effect::Frame { frame: frame.clone() }); }
                self.discard_effect_cards(seat, &selected, &frame, redraw)?;
                return Ok(());
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
                    if frame.cursor < frame.steps.len() { self.effects.push_front(Effect::Frame { frame: frame.clone() }); }
                    self.discard_effect_cards(seat, std::slice::from_ref(id), &frame, false)?;
                    return Ok(());
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
                    .filter(|r| !r.vacant && r.card.id == original.1 && r.influence[team] > 0)
                    .ok_or("遏制地区或势力已失效")?;
                r.influence[team] -= 1;
                self.note(format!(
                    "{} 从地区{}移除1个本方势力标志",
                    self.players[seat].name,
                    region + 1
                ));
            }
            FrameChoice::WM056InfluenceOrigin { regions } => {
                self.wm056_move(&frame, chooser, &regions, &selected)?;
            }
            FrameChoice::Region => {
                let region = selected
                    .first()
                    .and_then(|id| id.strip_prefix("region:"))
                    .and_then(|r| r.parse::<usize>().ok())
                    .filter(|r| self.region_live(*r))
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
        if frame.cursor < frame.steps.len() && self.defer_death_caller((*frame).clone(), crate::death_continuation::DeathCallerKind::Frame, &before_deaths) { return Ok(()); }
        self.effects.push_front(Effect::Frame { frame });
        Ok(())
    }
}
