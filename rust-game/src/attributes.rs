//! Finite attribute queries. Resolved turn bonuses use exact entity identities;
//! ordinary granted icons are independent of the printed initiative icons.
use crate::{
    catalog::card,
    model::{Card, Game, Icons},
    rules::{IconCondition, MagicIcon, StaticModifier},
};

impl Game {
    // One printed JC030 condition, shared by its permanent icon and added
    // subtype. Only a live face-up bat's current controller's assets count.
    pub(crate) fn jc030_blood_assets_active(&self, c: &Card) -> bool {
        if c.definition != "JC030" || c.face_down
            || !crate::rules::definition("JC030").modifiers.iter().any(|m| matches!(m, StaticModifier::JC030BloodAssetsVampireAndInvestigation)) {
            return false;
        }
        let Some((_, live)) = self.board(&c.id) else { return false; };
        !live.face_down && live.definition == "JC030"
            && self.players[live.controller].assets.iter()
                .filter(|a| card(&a.definition).magic_icon == MagicIcon::Blood)
                .take(2).count() == 2
    }
    // Exactly two admitted source-bound continuous scalars. Printed metadata
    // stays immutable; exhaustion does not erase asset domain icons.
    pub(crate) fn green_source_attribute_bonus(&self, c: &Card, region: usize) -> (u32, u32) {
        if c.face_down
            || card(&c.definition).kind != "character"
            || !self
                .regions
                .get(region)
                .is_some_and(|r| r.cards.iter().any(|live| live.id == c.id))
        {
            return (0, 0);
        }
        let modifiers = &crate::rules::definition(&c.definition).modifiers;
        if c.definition == "LC30"
            && modifiers
                .iter()
                .any(|m| matches!(m, StaticModifier::LC30WeaponsCombatAndDefense))
        {
            let n = self
                .attachments
                .iter()
                .filter(|a| {
                    !a.card.face_down
                        && a.host_id == c.id
                        && card(&a.card.definition)
                            .subtypes
                            .iter()
                            .any(|s| s == "武器")
                        && self.attachment_host_valid(a)
                })
                .count() as u32;
            return (n, n);
        }
        if c.definition == "JC018"
            && modifiers
                .iter()
                .any(|m| matches!(m, StaticModifier::JC018MindAssetsCombat))
        {
            let n = self.players[c.controller]
                .assets
                .iter()
                .filter(|a| card(&a.definition).magic_icon == MagicIcon::Mind)
                .count() as u32;
            return (n, 0);
        }
        (0, 0)
    }
    pub(crate) fn execution_turn_grants(&self, c: &Card) -> (u32, bool) {
        if c.face_down || card(&c.definition).kind != "character" {
            return (0, false);
        }
        self.turn_attribute_modifiers
            .iter()
            .filter(|m| m.target_instance == c.id && m.expires_turn == self.turn)
            .fold((0, false), |(kill, retreat), m| {
                (kill + m.kill_bonus, retreat || m.grants_retreat)
            })
    }
    pub(crate) fn effective_kill(&self, c: &Card) -> u32 {
        if c.face_down || card(&c.definition).kind != "character" {
            return 0;
        }
        crate::rules::definition(&c.definition).traits.kill + self.execution_turn_grants(c).0
    }
    pub(crate) fn has_retreat(&self, c: &Card) -> bool {
        !c.face_down
            && (crate::rules::definition(&c.definition).traits.retreat
                || self.execution_turn_grants(c).1)
    }
    pub(crate) fn grant_execution_traits(
        &mut self,
        id: String,
        kill_bonus: u32,
        grants_retreat: bool,
    ) {
        self.turn_attribute_modifiers
            .push(crate::model::TurnAttributeModifier {
                target_instance: id,
                defense_bonus: 0,
                kill_bonus,
                grants_retreat,
                printed_defense_override: None,
                ordinary_icons: Icons::default(),
                grants_renown: false,
                prevents_damage: false,
                expires_turn: self.turn,
            });
    }
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
