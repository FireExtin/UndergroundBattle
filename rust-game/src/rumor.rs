//! A graveyard transfer pauses before a rumor card leaves its original zone.
//! Reuse the normal pending choice/effects/sealed-card machinery, including UID reset.
use crate::{catalog, engine::RuleResult, exposure::same, model::*, rules::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraveyardTransfer {
    pub holder: usize,
    pub from_deck: bool,
    pub redraw: bool,
    pub originals: Vec<Card>,
    pub cursor: usize,
    pub outcomes: Vec<TransferOutcome>,
    pub discard_observers: Vec<SourceSnapshot>,
    pub cause: Option<Box<ResolutionFrame>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransferOutcome {
    Grave(Card),
    Sealed { card: Card, host_id: String },
}

impl Game {
    fn rumor_eligible(&self, c: &Card) -> bool {
        !c.face_down && (c.definition == "WM042"
            || (matches!(c.definition.as_str(), "WM033" | "BQ087") && self.players.get(c.controller).is_some_and(|p| p.society_zone.secrecy > 0)))
    }
    fn rumor_title(c: &Card) -> String {
        format!("{}：可改为封印在目标角色上", catalog::card(&c.definition).name)
    }
    fn rumor_options(&self, c: &Card) -> Vec<ChoiceOption> {
        self.binding_options(
            c.owner,
            &self.source_snapshot(c, None),
            &crate::rules::target(
                Zone::Board,
                EntityKind::Character,
                Relation::Any,
                Range::Anywhere,
            ),
        )
    }
    pub(crate) fn discard_effect_cards(
        &mut self,
        holder: usize,
        ids: &[String],
        cause: &ResolutionFrame,
        redraw: bool,
    ) -> RuleResult<()> {
        let originals = ids
            .iter()
            .map(|id| {
                self.players[holder]
                    .hand
                    .iter()
                    .find(|c| c.id == *id)
                    .cloned()
                    .ok_or_else(|| "效应弃牌引用失效".into())
            })
            .collect::<RuleResult<Vec<_>>>()?;
        if ids.iter().collect::<BTreeSet<_>>().len() != ids.len() {
            return Err("效应弃牌引用重复".into());
        }
        let observers = self.capture_discard_observers(holder);
        self.advance_graveyard_transfer(GraveyardTransfer {
            holder,
            from_deck: false,
            redraw,
            originals,
            cursor: 0,
            outcomes: vec![],
            discard_observers: observers,
            cause: Some(Box::new(cause.clone())),
        })
    }
    pub(crate) fn mill_cards_with_cause(
        &mut self,
        seat: usize,
        count: usize,
        cause: Option<&ResolutionFrame>,
    ) -> RuleResult<()> {
        if self.players[seat].eliminated {
            return Ok(());
        }
        let originals = self.players[seat]
            .deck
            .iter()
            .take(count)
            .cloned()
            .collect();
        self.advance_graveyard_transfer(GraveyardTransfer {
            holder: seat,
            from_deck: true,
            redraw: false,
            originals,
            cursor: 0,
            outcomes: vec![],
            discard_observers: vec![],
            cause: cause.map(|f| Box::new(f.clone())),
        })
    }
    fn take_transfer_card(&mut self, batch: &GraveyardTransfer) -> RuleResult<Card> {
        let original = batch
            .originals
            .get(batch.cursor)
            .ok_or("传闻批次游标越界")?;
        let zone = if batch.from_deck {
            &mut self.players[batch.holder].deck
        } else {
            &mut self.players[batch.holder].hand
        };
        let i = zone
            .iter()
            .position(|c| c == original)
            .ok_or("传闻原手牌或牌库实例已失效")?;
        Ok(zone.remove(i))
    }
    fn commit_transfer_grave(&mut self, batch: &mut GraveyardTransfer) -> RuleResult<()> {
        let c = self.take_transfer_card(batch)?;
        let mut source = self.source_snapshot(&c, None);
        source.card.controller = batch.holder;
        let c = self.reset_zone_card(c);
        self.commit_owner_grave(c.clone(), None);
        batch.outcomes.push(TransferOutcome::Grave(c));
        batch.cursor += 1;
        if !batch.from_deck {
            self.emit_event(batch.holder, source, Event::HandDiscard);
        }
        Ok(())
    }
    pub(crate) fn advance_graveyard_transfer(
        &mut self,
        mut batch: GraveyardTransfer,
    ) -> RuleResult<()> {
        while batch.cursor < batch.originals.len() {
            let c = &batch.originals[batch.cursor];
            if self.rumor_eligible(c) && !self.players[c.owner].eliminated {
                let options = self.rumor_options(c);
                if !options.is_empty() {
                    if batch.cause.is_none() {
                        return Err("可暂停的传闻须有真实效应来源".into());
                    }
                    self.choice(
                        c.owner,
                        "rumor",
                        Self::rumor_title(c),
                        options,
                        0,
                        1,
                        None,
                        ChoiceResolution::Rumor { batch },
                    );
                    return Ok(());
                }
            }
            self.commit_transfer_grave(&mut batch)?;
        }
        let graves: Vec<_> = batch
            .outcomes
            .iter()
            .filter_map(|o| {
                if let TransferOutcome::Grave(c) = o {
                    Some(c.clone())
                } else {
                    None
                }
            })
            .collect();
        if batch.redraw && !graves.is_empty() {
            self.effects.push_front(Effect::Draw {
                seat: batch.holder,
                count: graves.len(),
            });
        }
        if batch.from_deck {
            self.observe_black_deck_to_graveyard(batch.holder, &graves);
        } else {
            let discarded = batch
                .originals
                .into_iter()
                .zip(batch.outcomes)
                .filter_map(|(c, o)| matches!(o, TransferOutcome::Grave(_)).then_some(c))
                .collect();
            self.emit_discard_observed(batch.holder, discarded, batch.discard_observers);
        }
        Ok(())
    }
    pub(crate) fn rumor_complete(
        &mut self,
        mut batch: GraveyardTransfer,
        chooser: usize,
        selected: &[String],
    ) -> RuleResult<()> {
        let c = batch.originals.get(batch.cursor).ok_or("传闻游标无效")?;
        if chooser != c.owner || !self.rumor_eligible(c) || selected.len() > 1 {
            return Err("传闻必须由原牌拥有者选择".into());
        }
        if let Some(host) = selected.first() {
            if !self.rumor_options(c).iter().any(|o| &o.id == host) {
                return Err("传闻目标已经失效".into());
            }
            if self.shield_stops(chooser, host) {
                self.commit_transfer_grave(&mut batch)?;
            } else {
                let c = self.take_transfer_card(&batch)?;
                self.seal_card_on_valid_host(c, chooser, host);
                let sealed = self.sealed_cards.last().ok_or("传闻未完成封印")?;
                batch.outcomes.push(TransferOutcome::Sealed {
                    card: sealed.card.clone(),
                    host_id: host.clone(),
                });
                batch.cursor += 1;
            }
        } else {
            self.commit_transfer_grave(&mut batch)?;
        }
        self.advance_graveyard_transfer(batch)
    }
    // Only a real paused transfer may retain the unfinished original frame.
    // The batch validator authenticates its whole program and exact queue count.
    pub(crate) fn rumor_frame_continuation_valid(&self, f: &ResolutionFrame) -> bool {
        f.cursor < f.steps.len()
            && self.pending.as_ref().is_some_and(|p| match &p.resolution {
                ChoiceResolution::Rumor { batch } => batch
                    .cause
                    .as_ref()
                    .is_some_and(|cause| same(cause.as_ref(), f)),
                _ => false,
            })
    }
    pub(crate) fn validate_rumor_state(&self) -> RuleResult<()> {
        let Some(p) = &self.pending else {
            return Ok(());
        };
        let ChoiceResolution::Rumor { batch: b } = &p.resolution else {
            if p.choice.kind == "rumor" {
                return Err("传闻待选缺少传闻续体".into());
            }
            return Ok(());
        };
        if b.holder >= self.players.len()
            || self.players[b.holder].eliminated
            || b.originals.is_empty()
            || b.cursor >= b.originals.len()
            || b.cursor != b.outcomes.len()
            || b.originals.iter().any(|c| {
                c.id.is_empty()
                    || c.owner >= self.players.len()
                    || self.players[c.owner].eliminated
                    || c.controller >= self.players.len()
                    || c.face_down
                    || !catalog::catalog()
                        .cards
                        .iter()
                        .any(|d| d.id == c.definition)
            })
            || b.originals
                .iter()
                .map(|c| &c.id)
                .collect::<BTreeSet<_>>()
                .len()
                != b.originals.len()
        {
            return Err("传闻批次原牌、拥有者或游标无效".into());
        }
        let current = &b.originals[b.cursor];
        let options = self.rumor_options(current);
        if !self.rumor_eligible(current)
            || p.seat != current.owner
            || p.choice.player_id != player_id(current.owner)
            || p.choice.id.is_empty()
            || p.choice.kind != "rumor"
            || p.choice.title != Self::rumor_title(current)
            || p.choice.min != Some(0)
            || p.choice.max != Some(1)
            || p.choice.allow_decline != Some(true)
            || p.choice.amount.is_some()
            || !p.choice.preview_cards.is_empty()
            || options.is_empty()
            || !same(&p.choice.options, &options)
        {
            return Err("传闻待选拥有者、目标或元数据无效".into());
        }
        let zone = if b.from_deck {
            &self.players[b.holder].deck
        } else {
            &self.players[b.holder].hand
        };
        if b.from_deck
            && !same(
                &b.originals[b.cursor..].to_vec(),
                &zone
                    .iter()
                    .take(b.originals.len() - b.cursor)
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        {
            return Err("传闻磨牌必须保留原批次牌库顶顺序".into());
        }
        let mut outcome_ids = BTreeSet::new();
        for c in &b.originals[b.cursor..] {
            if zone.iter().filter(|live| *live == c).count() != 1 {
                return Err("传闻尚未替换的牌必须仍在原手牌或牌库中".into());
            }
        }
        for (old, outcome) in b.originals.iter().zip(&b.outcomes) {
            let c = match outcome {
                TransferOutcome::Grave(c) => {
                    if c.owner >= self.players.len() {
                        return Err("入墓收据拥有者无效".into());
                    }
                    if !self.players[c.owner].graveyard.iter().any(|live| live == c) {
                        return Err("传闻批次实际入墓收据不符".into());
                    }
                    c
                }
                TransferOutcome::Sealed { card, host_id } => {
                    if !matches!(old.definition.as_str(), "WM042" | "WM033" | "BQ087")
                        || !self
                            .sealed_cards
                            .iter()
                            .any(|s| s.host_id == *host_id && s.card == *card)
                    {
                        return Err("传闻封印替换收据不符".into());
                    }
                    card
                }
            };
            if !outcome_ids.insert(&c.id)
                || c.owner != old.owner
                || c.definition != old.definition
                || c.id == old.id
                || c.controller != c.owner
                || c.face_down
                || c.exhausted
                || c.damage > 0
                || c.wounds > 0
                || c.shield > 0
                || c.lock_markers > 0
                || c.time_markers > 0
                || zone.iter().any(|live| live.id == old.id)
            {
                return Err("传闻区域转移未重置身份".into());
            }
        }
        if b.from_deck && (!b.discard_observers.is_empty() || b.redraw) {
            return Err("磨牌不得产生弃牌观察".into());
        }
        if !b.from_deck
            && !same(
                &b.discard_observers,
                &self.capture_discard_observers(b.holder),
            )
        {
            return Err("传闻批次的弃牌观察者不符".into());
        }
        let f = b.cause.as_ref().ok_or("传闻替换缺少原效应")?;
        if f.actor >= self.players.len()
            || f.source.card.controller != f.actor
            || f.source.card.owner >= self.players.len()
            || f.source.card.id.is_empty()
            || f.source.card.face_down
            || !catalog::catalog()
                .cards
                .iter()
                .any(|c| c.id == f.source.card.definition)
            || f.source.region.is_some_and(|r| r >= self.regions.len())
            || f.source.play_source.is_some()
            || f.source.observed_mobility_grant.is_some()
            || f.source.observed_exposure.is_some()
            || f.chosen_region.is_some()
            || f.frame_id.is_empty()
            || !matches!(f.guard, GuardState::Accepted)
            || f.cursor == 0
            || f.cursor > f.steps.len()
        {
            return Err("传闻原效应游标或行动者无效".into());
        }
        let continuations = self
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Frame { frame } if frame.frame_id == f.frame_id => Some(frame),
                _ => None,
            })
            .collect::<Vec<_>>();
        let unfinished = usize::from(f.cursor < f.steps.len());
        if continuations.len() != unfinished
            || continuations
                .iter()
                .any(|frame| !same(frame.as_ref(), f.as_ref()))
        {
            return Err("传闻续体必须精确保留且只执行一次".into());
        }
        let after_mill = self
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::JZ57AfterMill { frame } if frame.frame_id == f.frame_id => Some(frame),
                _ => None,
            })
            .collect::<Vec<_>>();
        if after_mill.len() != usize::from(f.ability_key == "batch20-jz57-mill")
            || after_mill
                .iter()
                .any(|frame| !same(frame.as_ref(), f.as_ref()))
        {
            return Err("骷髅抓牌续体必须精确保留且只执行一次".into());
        }
        self.validate_batch20_frame(f)?;
        self.validate_jc035_frame(f)?;
        self.validate_jc041_frame(f)?;
        self.validate_black_next20_frame(f)?;
        self.validate_next_blue_gray_frame(f)?;
        self.validate_batch3_frame(f)?;
        self.validate_pack_a_frame(f)?;
        self.validate_cq28_frame(f)?;
        if f.source.observed_death.is_some() {
            // The original observer was accepted before the random/nonrandom
            // discard. Authenticate its captured death and paid-once limit at
            // the original entry cursor without re-running its guard or RNG.
            let mut original = (**f).clone();
            original.cursor = 0;
            original.guard = GuardState::Unchecked;
            self.validate_observer_frame(&original, None)?;
        }
        // Authenticate the full original program, including every later step.
        // Runtime batch observations keep their existing stricter validators.
        let observer_discard = match (f.source.card.definition.as_str(), f.ability_key.as_str(), f.steps.as_slice()) {
            ("JZ23", "next-bg-jz23-reveal", [Step { op: Op::JZ23RevealReward { observation: Some(_) }, .. }, Step { op: Op::Discard { .. }, .. }]) => {
                f.cursor == 2 && !b.from_deck && !b.redraw && b.originals.len() == 1
                    && f.targets.first().is_some_and(|t| t.id == player_id(b.holder))
            },
            ("CQ26", "black20-cq26-loot", [Step { op: Op::BlackDeckToGraveyard { observation: Some(_) }, .. }, Step { op: Op::Discard { .. }, .. }]) => {
                f.cursor == 2 && !b.from_deck && b.redraw && b.originals.len() == 1 && b.holder == f.actor
            },
            _ => false,
        };
        let dynamic = observer_discard || self.pack_a_rumor_frame(f)? || matches!(
            f.steps.get(f.cursor - 1).map(|s| &s.op),
            Some(Op::JC034CombatDiscard { .. } | Op::Batch20Prepare { .. } | Op::Batch3Prepare { .. })
        );
        if !dynamic {
            let candidates = crate::rules::definition(&f.source.card.definition)
                .abilities
                .iter()
                .flat_map(|a| {
                    let mut v = vec![a.clone()];
                    for m in &a.modes {
                        let mut selected = a.clone();
                        selected.targets = m.targets.clone();
                        selected.ops = m.ops.clone();
                        selected.modes.clear();
                        v.push(selected);
                    }
                    v
                })
                .collect::<Vec<_>>();
            if !candidates.iter().any(|a| {
                let expected = self.make_frame(
                    f.actor,
                    f.source.clone(),
                    a,
                    f.targets.clone(),
                    f.already_paid.clone(),
                    f.chosen_region,
                );
                let payment = if a.event.is_some() {
                    f.already_paid.is_empty()
                } else if f.source.card.definition == "JC114" {
                    matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids), PaidCost::Exhausted(id)]
                        if f.money_units(0) == 1 && ids.iter().all(|id| !id.is_empty()) && id == &f.source.card.id && !f.source.card.exhausted)
                } else if a.costs.is_empty() {
                    matches!(f.already_paid.as_slice(), [PaidCost::Assets(ids)]
                        if f.money_units(0) <= catalog::card(&f.source.card.definition).cost as usize
                            && ids.iter().all(|id| !id.is_empty()) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len())
                } else {
                    // Existing nontrivial JC038/JZ57 sacrifice is authenticated above
                    // by validate_batch20_frame, including captured actual subtype.
                    matches!(f.source.card.definition.as_str(), "JC038" | "JZ57")
                };
                a.key == f.ability_key
                    && payment
                    && a.modes.is_empty()
                    && same(&expected.steps, &f.steps)
                    && same(
                        &a.targets,
                        &f.targets.iter().map(|t| t.spec.clone()).collect::<Vec<_>>(),
                    )
            }) {
                return Err("传闻续体不能移植或改写原完整程序".into());
            }
        }
        if f.targets
            .iter()
            .any(|t| t.id.is_empty() || t.public.instance_id != t.id)
        {
            return Err("传闻原效应目标身份无效".into());
        }
        let step = &f.steps[f.cursor - 1];
        let seat = |p: PlayerRef| match p {
            PlayerRef::Actor => Some(f.actor),
            PlayerRef::Context => Some(step.context),
            PlayerRef::Target(i) => f
                .targets
                .get(i)
                .and_then(|t| t.id.strip_prefix('p'))
                .and_then(|s| s.parse::<usize>().ok()),
        };
        let allowed = match &step.op {
            Op::CQ28Mode { mode: 0, .. } => {
                b.from_deck && !b.redraw
                    && f.source.card.definition == "CQ28"
                    && f.ability_key == crate::import20_gate::KEY
                    && b.holder == f.actor && b.originals.len() <= 6
            }
            Op::IndustryMill { .. } => {
                b.from_deck && !b.redraw
                    && f.source.card.definition == "BQ093"
                    && f.ability_key == crate::import20_industry::MILL
                    && b.holder == f.actor && b.originals.len() == 1
            }
            Op::MoveDeckTopToGraveyard { player, count } => {
                b.from_deck && seat(*player) == Some(b.holder) && b.originals.len() <= *count
            }
            Op::JZ57MillAndOptionalDraw => {
                b.from_deck
                    && f.source.card.definition == "JZ57"
                    && seat(PlayerRef::Target(0)) == Some(b.holder)
                    && b.originals.len() <= 2
            }
            Op::Batch20Prepare { .. } => {
                b.from_deck
                    && f.source.card.definition == "BQ091"
                    && b.holder == f.actor
                    && b.originals.len() == 1
            }
            Op::Batch3Prepare { .. } => {
                b.from_deck && f.source.card.definition == "XG28"
                    && b.holder == f.actor && b.originals.len() == 1
            }
            Op::Discard {
                player,
                count,
                redraw,
                ..
            } => {
                b.redraw == *redraw
                    && !b.from_deck
                    && seat(*player) == Some(b.holder)
                    && count.is_none_or(|n| b.originals.len() <= n)
            }
            Op::JC034CombatDiscard { .. } | Op::JC031DiscardForObservedDeath => {
                !b.from_deck
                    && seat(PlayerRef::Target(0)) == Some(b.holder)
                    && b.originals.len() == 1
            }
            Op::RevealHandAndOfferSourceSacrifice { player } => {
                !b.from_deck && seat(*player) == Some(b.holder) && b.originals.len() == 1
            }
            _ => false,
        };
        if !allowed {
            return Err("传闻原效应不能产生该区域转移批次".into());
        }
        Ok(())
    }
}
