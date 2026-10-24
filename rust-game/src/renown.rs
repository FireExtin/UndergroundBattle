//! Project 2v2 interpretation: compare team totals and merge renown into one
//! optional reward. Hegemony win-before-renown ordering preserves the old flow.
use crate::{
    catalog::card,
    engine::RuleResult,
    model::*,
    rules::{self, AbilitySpec, Event, Op, ResponsePolicy, Timing},
};

pub(crate) fn reward_key(key: &str) -> bool {
    matches!(key, "renown" | "jc089-combat-glory")
}

pub(crate) fn reward_related(a: &AbilitySpec) -> bool {
    reward_key(&a.key)
        || matches!(a.event, Some(Event::RegionConfrontationsEnded | Event::CombatWon))
}

pub(crate) fn renown_ability(region_instance: &str) -> AbilitySpec {
    AbilitySpec {
        play_only: false,
        activation_only: false,
        key: "renown".into(),
        label: "声望：本地区额外放置一个势力标志".into(),
        timing: Timing::Fast,
        response_policy: ResponsePolicy::Respondable,
        costs: vec![],
        targets: vec![],
        ops: vec![Op::PlaceInfluence {
            region_instance: region_instance.into(),
            amount: 1,
        }],
        event: Some(Event::RegionConfrontationsEnded),
        modes: vec![],
        requires_ready_source: false,
        once_per_game: false,
        per_turn_limit: None,
    }
}

impl Game {
    // These two rewards belong to the observed keyword event, independently of
    // the recipient's printed abilities. Keep their complete one-influence
    // program and original region closed rather than bypassing per-card checks
    // on a key string alone. A later response may remove the source or grant.
    fn validate_reward_source(
        &self,
        actor: usize,
        source: &SourceSnapshot,
        region_instance: &str,
    ) -> RuleResult<()> {
        let c = &source.card;
        if actor >= self.players.len()
            || c.controller != actor
            || c.owner >= self.players.len()
            || c.id.is_empty()
            || c.face_down
            || !crate::catalog::catalog().cards.iter().any(|d|
                d.id == c.definition && crate::catalog::has_kind(d, "character"))
            || source.region.is_none_or(|r| r >= self.regions.len())
            || region_instance.is_empty()
            || region_instance == c.id
            || source.source_region_instance.as_deref() != Some(region_instance)
            || source.play_source.is_some()
            || source.death_echo.is_some()
            || source.observed_death.is_some()
            || source.observed_exposure.is_some()
            || source.observed_mobility_grant.is_some()
            || source.attachment_host_instance.as_ref().is_some_and(|host|
                c.definition != "BQ026" || host.is_empty() || *host == c.id)
            || self.regions.iter().enumerate().any(|(i,r)|
                r.card.id == region_instance && Some(i) != source.region)
        {
            return Err("声望/威名奖励须保留实际角色、行动者和冻结的原地区".into());
        }
        self.mobility_immutable_identity(c)
    }

    pub(crate) fn validated_reward_declaration(&self, d: &Declaration) -> RuleResult<bool> {
        if !reward_related(&d.ability) { return Ok(false); }
        if !crate::jz22::runtime_ability_is_admitted(&d.ability) {
            return Err("声望/威名只允许完整无费用的原地区一个势力奖励".into());
        }
        let [Op::PlaceInfluence { region_instance, .. }] = d.ability.ops.as_slice() else { unreachable!() };
        self.validate_reward_source(d.actor, &d.source, region_instance)?;
        Ok(true)
    }

    pub(crate) fn validated_reward_frame(&self, f: &ResolutionFrame) -> RuleResult<bool> {
        if !reward_key(&f.ability_key) { return Ok(false); }
        let [Step { context, op: Op::PlaceInfluence { region_instance, amount: 1 } }] = f.steps.as_slice()
            else { return Err("声望/威名帧必须保留一个原地区势力奖励".into()); };
        self.validate_reward_source(f.actor, &f.source, region_instance)?;
        if f.frame_id.is_empty() || *context != f.actor || f.cursor > 1
            || f.cursor == 0 && !matches!(f.guard, GuardState::Unchecked)
            || f.cursor == 1 && !matches!(f.guard, GuardState::Accepted)
            || !f.targets.is_empty() || !f.already_paid.is_empty()
            || !f.resource_payments.is_empty() || f.chosen_region.is_some()
        {
            return Err("声望/威名帧的上下文、费用、目标或执行位置不符".into());
        }
        Ok(true)
    }

    pub(crate) fn validate_reward_state(&self) -> RuleResult<()> {
        for item in &self.stack {
            if let Some(f) = &item.frame {
                if self.validated_reward_frame(f)?
                    && (item.id != f.frame_id || item.controller != f.actor
                        || item.card.is_some() || item.target.is_some() || item.reveal
                        || item.deploy_region.is_some() || f.cursor != 0)
                {
                    return Err("声望/威名响应对象与冻结帧不符".into());
                }
            }
        }
        for effect in &self.effects {
            match effect {
                Effect::Declare { declaration } => { self.validated_reward_declaration(declaration)?; }
                Effect::Frame { frame } if self.validated_reward_frame(frame)? && frame.cursor != 0 =>
                    return Err("声望/威名原子奖励不能保留已执行续体".into()),
                _ => {},
            }
        }
        if let Some(p) = &self.pending {
            match &p.resolution {
                ChoiceResolution::Declare { declaration, stage }
                    if self.validated_reward_declaration(declaration)? => {
                    let options = vec![ChoiceOption { id: "accept".into(), label: "发动触发能力".into(), card: None }];
                    if !matches!(stage, DeclareChoice::Accept) || p.seat != declaration.actor
                        || p.choice.player_id != player_id(declaration.actor) || p.choice.kind != "trigger"
                        || p.choice.min != Some(0) || p.choice.max != Some(1) || p.choice.allow_decline != Some(true)
                        || serde_json::to_value(&p.choice.options).unwrap() != serde_json::to_value(options).unwrap()
                    {
                        return Err("声望/威名只允许原行动者选择发动或放弃".into());
                    }
                }
                ChoiceResolution::Frame { frame, .. } if self.validated_reward_frame(frame)? =>
                    return Err("声望/威名原子奖励没有帧内选择".into()),
                _ => {},
            }
        }
        Ok(())
    }

    pub(crate) fn jc089_combat_glory(&self, team: usize, region: usize) -> Option<Declaration> {
        // Only the fully admitted JC018 print and real JC089 host grant qualify.
        // Merge their identical 威名 into one optional combat-time reward.
        let source = self.region_characters(region).filter(|c|
            self.team(c.controller) == team && !self.players[c.controller].eliminated
                && self.icons(c, region).combat > 0 && self.has_combat_glory(c))
            .min_by_key(|c| c.controller)?;
        let mut snapshot = self.source_snapshot(source, Some(region));
        snapshot.source_region_instance = Some(self.regions[region].card.id.clone());
        Some(Declaration {
            actor: source.controller,
            source: snapshot,
            ability: rules::jc089_combat_glory_ability(&self.regions[region].card.id),
        })
    }
    pub(crate) fn has_renown(&self, c: &Card) -> bool {
        !c.face_down
            && card(&c.definition).kind == "character"
            && !self.wm041_renown_denied(c)
            && (self.wm064_status_grant(c) || self.wm029_renown(c) || self.wm041_renown_granted(c) || self.jz32_destroyed(c) || self.xg17_conditional_renown(c) || rules::definition(&c.definition).traits.renown
                || self.turn_attribute_modifiers.iter().any(|m| {
                    m.target_instance == c.id && m.expires_turn == self.turn && m.grants_renown
                }))
    }

    pub(crate) fn place_influence(&mut self, seat: usize, region: usize, amount: u32) {
        if !self.region_live(region) { return; }
        let team = self.team(seat);
        let enemy = 1 - team;
        let removed = amount.min(self.regions[region].influence[enemy]);
        self.regions[region].influence[enemy] -= removed;
        self.regions[region].influence[team] += amount - removed;
        if self.regions[region].influence[team]
            >= card(&self.regions[region].card.definition)
                .threshold
                .unwrap_or(3)
        {
            self.effects.push_back(Effect::Award {
                seat, region, region_instance: self.regions[region].card.id.clone(),
            });
        }
    }

    pub(crate) fn declare_region_renown(&mut self, region: usize, region_instance: &str) {
        // An ordinary influence Award is queued first and opens Win before this
        // effect. After(r,2) is also used after replacement: never emit there.
        if self.window != Some(Window::After(region, 2))
            || !self.influence_contests.iter().any(|p| p.region == region && p.region_instance == region_instance && p.ended && p.turn == self.turn)
            || !self
                .regions
                .get(region)
                .is_some_and(|r| !r.vacant && r.card.id == region_instance && !r.skip)
        {
            return;
        }
        let mut counts = [0u32; 2];
        let eligible = self.region_characters(region)
            .filter(|c| {
                !c.exhausted && !self.players[c.controller].eliminated && self.has_renown(c)
            })
            .collect::<Vec<_>>();
        for c in &eligible {
            counts[self.team(c.controller)] += 1;
        }
        if counts[0] == counts[1] {
            return;
        }
        let team = usize::from(counts[1] > counts[0]);
        // Existing trigger order chooses the first eligible controller, then
        // board order. One ordinary optional declaration, no team vote or
        // repeated per-card rewards, even if that controller declines.
        let source = eligible
            .into_iter()
            .filter(|c| self.team(c.controller) == team)
            .min_by_key(|c| c.controller)
            .unwrap();
        let actor = source.controller;
        let mut source = self.source_snapshot(source, Some(region));
        source.source_region_instance = Some(region_instance.into());
        let ability = renown_ability(region_instance);
        self.effects.push_front(Effect::Declare {
            declaration: Declaration {
                actor,
                source,
                ability,
            },
        });
    }
}
