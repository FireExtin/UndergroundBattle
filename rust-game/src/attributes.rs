//! Finite pure conditional icon queries. Exhausted assets retain their domains.
use crate::{catalog::card, model::Card, model::Game, rules::IconCondition};

impl Game {
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
                self.players[c.controller]
                    .assets
                    .iter()
                    .filter(|asset| card(&asset.definition).magic_icon == *magic)
                    .count()
                    >= *minimum
            }
            IconCondition::RegionInfluence { friendly, minimum } => {
                let team = self.team(c.controller);
                self.regions[region].influence[if *friendly { team } else { 1 - team }] >= *minimum
            }
        }
    }
}
