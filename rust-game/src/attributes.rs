//! Finite attribute queries. Resolved turn bonuses use exact entity identities;
//! ordinary granted icons are independent of the printed initiative icons.
use crate::{
    catalog::card,
    model::{Card, Game, Icons},
    rules::{IconCondition, MagicIcon},
};

impl Game {
    pub fn spirit_protected(&self, c: &Card) -> bool {
        if c.face_down
            || !crate::rules::definition(&c.definition).traits.spirit
            || card(&c.definition).kind != "character"
        {
            return false;
        }
        let Some((region, _)) = self.board(&c.id) else {
            return false;
        };
        let mut domains = [0usize; 2];
        for (r, other) in self.in_play_cards() {
            if r == region
                && !other.face_down
                && card(&other.definition).magic_icon != MagicIcon::None
            {
                domains[self.team(other.controller)] += 1;
            }
        }
        let team = self.team(c.controller);
        domains[1 - team] < domains[team]
    }

    pub(crate) fn printed_defense_override(&self, c: &Card) -> Option<u32> {
        if c.face_down || card(&c.definition).kind != "character" {
            return None;
        }
        self.turn_attribute_modifiers.iter().rev().find_map(|m| {
            (m.target_instance == c.id && m.expires_turn == self.turn)
                .then_some(m.printed_defense_override)
                .flatten()
        })
    }
    pub fn damage_prevented(&self, c: &Card) -> bool {
        !c.face_down
            && card(&c.definition).kind == "character"
            && self.turn_attribute_modifiers.iter().any(|m| {
                m.target_instance == c.id && m.expires_turn == self.turn && m.prevents_damage
            })
    }
    pub(crate) fn turn_attribute_bonus(&self, c: &Card) -> (u32, Icons) {
        if c.face_down || card(&c.definition).kind != "character" {
            return (0, Icons::default());
        }
        self.turn_attribute_modifiers
            .iter()
            .filter(|m| m.target_instance == c.id && m.expires_turn == self.turn)
            .fold((0, Icons::default()), |(defense, icons), m| {
                (defense + m.defense_bonus, icons.add(m.ordinary_icons))
            })
    }
    pub(crate) fn prune_turn_attribute_modifiers(&mut self) {
        let present = self
            .regions
            .iter()
            .flat_map(|r| r.cards.iter())
            .filter(|c| !c.face_down && card(&c.definition).kind == "character")
            .map(|c| c.id.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        self.turn_attribute_modifiers.retain(|m| {
            m.expires_turn == self.turn && present.contains(m.target_instance.as_str())
        });
    }
    pub(crate) fn actor_has_asset_domain(
        &self,
        actor: usize,
        magic: &MagicIcon,
        minimum: usize,
    ) -> bool {
        self.players[actor]
            .assets
            .iter()
            .filter(|asset| card(&asset.definition).magic_icon == *magic)
            .count()
            >= minimum
    }
    pub(crate) fn icon_condition(
        &self,
        c: &Card,
        region: usize,
        condition: &IconCondition,
    ) -> bool {
        if c.face_down
            || !self
                .regions
                .get(region)
                .is_some_and(|r| r.cards.iter().any(|x| x.id == c.id))
        {
            return false;
        }
        match condition {
            IconCondition::AssetDomain { magic, minimum } => {
                self.actor_has_asset_domain(c.controller, magic, *minimum)
            }
            IconCondition::RegionInfluence { friendly, minimum } => {
                let team = self.team(c.controller);
                self.regions[region].influence[if *friendly { team } else { 1 - team }] >= *minimum
            }
        }
    }
}
