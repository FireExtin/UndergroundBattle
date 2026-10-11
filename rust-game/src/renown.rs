//! Project 2v2 interpretation: compare team totals and merge renown into one
//! optional reward. Hegemony win-before-renown ordering preserves the old flow.
use crate::{
    catalog::card,
    model::{Card, Declaration, Effect, Game, Window},
    rules::{self, AbilitySpec, Event, Op, ResponsePolicy, Timing},
};

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
    pub(crate) fn jc089_combat_glory(&self, team: usize, region: usize) -> Option<Declaration> {
        // Only the fully admitted JC018 print and real JC089 host grant qualify.
        // Merge their identical 威名 into one optional combat-time reward.
        let source = self.regions[region].cards.iter().filter(|c|
            self.team(c.controller) == team && !self.players[c.controller].eliminated
                && self.icons(c, region).combat > 0 && self.has_combat_glory(c))
            .min_by_key(|c| c.controller)?;
        Some(Declaration {
            actor: source.controller,
            source: self.source_snapshot(source, Some(region)),
            ability: rules::jc089_combat_glory_ability(&self.regions[region].card.id),
        })
    }
    pub(crate) fn has_renown(&self, c: &Card) -> bool {
        !c.face_down
            && card(&c.definition).kind == "character"
            && (rules::definition(&c.definition).traits.renown
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
            || !self
                .regions
                .get(region)
                .is_some_and(|r| !r.vacant && r.card.id == region_instance && !r.skip)
        {
            return;
        }
        let mut counts = [0u32; 2];
        let eligible = self.regions[region]
            .cards
            .iter()
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
        let source = self.source_snapshot(source, Some(region));
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
